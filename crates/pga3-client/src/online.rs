//! The game on the original servers: joins a room of the live CrazyGames build through the Photon
//! client (`net/`) and turns PUN traffic into what the room scripts and the remote copies need,
//! and back. Every message reproduces what the recorded original client sent
//! (`docs/protocol.md`, "Playing online"):
//!
//! - after joining: `TeamName = "Spectators"`, `Kills = 0`, `Deaths = 0`, then
//!   `networkAddMessage(name, "", "connected", "")` on the room's view 1;
//! - choosing a team: `TeamName`, `PhotonNetwork.Instantiate` of the team's player prefab
//!   (cached), `networkAddMessage(name, "", "joined <team>", "")`;
//! - the player's `OnPhotonSerializeView` every 100 ms (201; once a reliable 206 when nothing
//!   changed, then nothing until it changes: `UnreliableOnChange`);
//! - `SyncWeaponsRPC` / `DoDamage` / `networkAddMessage` RPCs, `Kills` / `Deaths` / team score
//!   properties, `Ping` every 5 s;
//! - death: the cached instantiation and RPCs removed, event 204 destroys the player object.
//!
//! PUN's own reactions are reproduced too (`NetworkingPeer.OnEvent`, decompiled): a joining
//! player resets every view's last serialized state so the full state reaches it
//! (`ResetPhotonViewsOnSerialize`), the master client follows the server's choice or the lowest
//! actor number (`CheckMasterClient`), and event 203 from the master client leaves the room.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use glam::{Quat, Vec3};

use crate::net::client::{AppProfile, Client, ClientEvent, NewRoom, RoomInfo};
use crate::net::photon::{cache, game_key};
use crate::net::protocol16::Value;
use crate::net::pun::{self, PunEvent};
use crate::remote::{ActorId, NetState};
use crate::room::{PLAYER_PREFABS, prefab_kind};
use crate::weapons::SyncFire;

/// Marker in this game's room names (`ConnectMenu`); other rooms of the AppId redirect elsewhere.
pub const ROOM_MARKER: &str = "PixelWarfare001v";
/// `PlayerPrefs`-free `Ping` update interval of the room scripts (recorded: every 5 s).
const PING_PROPERTY_INTERVAL: f32 = 5.0;

#[derive(Clone, Debug)]
pub struct OnlineOptions {
    pub region: String,
    pub name: String,
    /// Join the first room whose name contains this text.
    pub room: Option<String>,
    /// Create a room on this map when no room is joined.
    pub map: String,
    /// Always create a new room.
    pub create: bool,
    pub mode: String,
}

/// What the network asks of the game this frame.
#[derive(Clone, Debug)]
pub enum NetIn {
    /// Another player's `PhotonNetwork.Instantiate` of its player prefab (a `PLAYER_PREFABS`
    /// kind).
    Instantiate {
        actor: ActorId,
        kind: usize,
        position: Vec3,
        rotation: Quat,
    },
    State {
        actor: ActorId,
        state: NetState,
    },
    SyncWeapons {
        actor: ActorId,
        sync: SyncFire,
    },
    /// `DoDamage(amount, damager)` on the player object of `target` (possibly this client's).
    DoDamage {
        target: ActorId,
        amount: f32,
        damager: ActorId,
    },
    Message {
        killer: String,
        killed: String,
        middle: String,
        team: String,
    },
    /// The player object of `actor` was destroyed (death, or the player left).
    Destroy {
        actor: ActorId,
    },
    RoomScores {
        team1: Option<i32>,
        team2: Option<i32>,
    },
    /// `DoSwapTeams` on another player's object: its death prefab, camera cleared.
    SwapTeams {
        actor: ActorId,
    },
    /// Another player left the room (`OnPhotonPlayerDisconnected`, after PUN picked the master
    /// client).
    PlayerLeft {
        name: String,
    },
    /// This client is out of the room (`OnLeftRoom`): it left, or the master client closed its
    /// connection.
    LeftRoom,
    /// The connection ended (`OnDisconnectedFromPhoton`).
    Disconnected(String),
}

/// The custom properties of a player the room menu lists (`None`: not set).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActorProps {
    pub name: String,
    pub team: Option<String>,
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub ping: Option<i32>,
}

pub struct Online {
    pub client: Client,
    pub room_props: Value,
    /// The player object (view id) of each actor that has one.
    views: HashMap<ActorId, i32>,
    names: HashMap<ActorId, String>,
    /// Every actor's properties as the server reported them.
    players: HashMap<ActorId, ActorProps>,
    /// The `Ping` this client last set on itself.
    pub local_ping: Option<i32>,
    /// This client's player object.
    local_view: Option<i32>,
    last_view: i32,
    last_state: Option<NetState>,
    reliable_sent: bool,
    ping_property_at: f32,
    warned: Vec<String>,
    /// `NetworkingPeer.mMasterClientId`.
    master_id: ActorId,
    /// What arrived together with the join answer (the room's cached events), handled on the
    /// first `update` once the scene exists (PUN holds them while `isMessageQueueRunning` is off).
    backlog: Vec<ClientEvent>,
}

fn scene_exists(assets: &Path, map: &str) -> bool {
    assets.join(format!("data/scenes/{map}.json")).is_file()
}

/// Waits for `ready` while polling, failing on a connection error or after `timeout`.
fn wait_for(
    client: &mut Client,
    timeout: Duration,
    what: &str,
    mut ready: impl FnMut(&Client, &ClientEvent) -> bool,
) -> Result<Vec<ClientEvent>> {
    let start = Instant::now();
    let mut seen = Vec::new();
    loop {
        // The whole batch is kept: what came with the answer (e.g. the room's cached events
        // right after the join) belongs to the caller too.
        let mut done = false;
        for event in client.poll() {
            if let ClientEvent::Failed(reason) = &event {
                bail!("{what}: {reason}");
            }
            done |= ready(client, &event);
            seen.push(event);
        }
        if done {
            return Ok(seen);
        }
        if start.elapsed() > timeout {
            bail!("{what}: no answer within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The rooms of this game the client can enter (open, not full, on a map this client has).
fn joinable<'a>(rooms: &'a [RoomInfo], assets: &Path) -> Vec<&'a RoomInfo> {
    rooms
        .iter()
        .filter(|r| r.name.contains(ROOM_MARKER) && r.open)
        .filter(|r| r.max_players == 0 || r.players < r.max_players)
        .filter(|r| {
            r.prop_str("MapName")
                .is_some_and(|m| scene_exists(assets, m))
        })
        .filter(|r| r.prop_str("Password").is_none_or(str::is_empty))
        .collect()
}

pub fn print_rooms(rooms: &[RoomInfo]) {
    println!("online: {} rooms", rooms.len());
    for room in rooms {
        println!(
            "  {:>2}/{:<2} {:<10} {:<4} {}",
            room.players,
            room.max_players,
            room.prop_str("MapName").unwrap_or("?"),
            room.prop_str("GameMode").unwrap_or("?"),
            room.name
        );
    }
}

/// Connects to the region's lobby and prints its rooms (`--list-rooms`).
pub fn list_rooms(profile: AppProfile, region: &str, name: &str) -> Result<()> {
    println!(
        "online: connecting to {} ({}, region {region})",
        profile.name, profile.app_version
    );
    let mut client = Client::connect(profile, region, name);
    wait_for(
        &mut client,
        Duration::from_secs(15),
        "joining the lobby",
        |_, e| matches!(e, ClientEvent::InLobby),
    )?;
    let _ = wait_for(&mut client, Duration::from_secs(3), "room list", |_, e| {
        matches!(e, ClientEvent::RoomList)
    });
    print_rooms(&client.rooms);
    Ok(())
}

impl Online {
    /// Connects to the region's master, picks or creates a room and joins it (blocking: before
    /// the scene loads, as the original loads the room's map after `OnJoinedRoom`).
    pub fn join(profile: AppProfile, options: &OnlineOptions, assets: &Path) -> Result<Online> {
        println!(
            "online: connecting to {} ({}, region {})",
            profile.name, profile.app_version, options.region
        );
        let mut client = Client::connect(profile, &options.region, &options.name);
        let timeout = Duration::from_secs(15);
        wait_for(&mut client, timeout, "joining the lobby", |_, e| {
            matches!(e, ClientEvent::InLobby)
        })?;
        // The room list arrives right after the lobby join.
        let _ = wait_for(&mut client, Duration::from_secs(3), "room list", |_, e| {
            matches!(e, ClientEvent::RoomList)
        });
        print_rooms(&client.rooms);
        let choice = {
            let rooms = joinable(&client.rooms, assets);
            match &options.room {
                _ if options.create => None,
                Some(text) => client
                    .rooms
                    .iter()
                    .find(|r| r.name.contains(text.as_str()))
                    .map(|r| r.name.clone()),
                None => rooms
                    .iter()
                    .max_by_key(|r| r.players)
                    .map(|r| r.name.clone()),
            }
        };
        match choice {
            Some(name) => {
                println!("online: joining {name}");
                client.join_room(&name)?;
            }
            None => {
                if options.room.is_some() {
                    bail!("no room matches --room");
                }
                let room = new_room(&options.map, &options.mode);
                println!("online: no open room; creating {}", room.name);
                client.create_room(room)?;
            }
        }
        let events = wait_for(&mut client, timeout, "joining the room", |_, e| {
            matches!(e, ClientEvent::Joined { .. })
        })?;
        Ok(Online::from_events(client, events))
    }

    /// Wraps a client that has just received `Joined` (the menu drives the connection itself).
    pub fn from_events(client: Client, events: Vec<ClientEvent>) -> Online {
        let mut online = Online {
            client,
            room_props: Value::Hashtable(Vec::new()),
            views: HashMap::new(),
            names: HashMap::new(),
            players: HashMap::new(),
            local_ping: None,
            local_view: None,
            last_view: 0,
            last_state: None,
            reliable_sent: false,
            ping_property_at: 0.0,
            warned: Vec::new(),
            master_id: 0,
            backlog: Vec::new(),
        };
        let mut joined = false;
        for event in events {
            match event {
                ClientEvent::Joined {
                    actors, room_props, ..
                } if !joined => {
                    joined = true;
                    for (actor, props) in &actors {
                        if std::env::var_os("PGA3_NET_DEBUG").is_some() {
                            eprintln!("net: actor {actor} {props:?}");
                        }
                        online.update_names(*actor, props);
                    }
                    online.room_props = room_props;
                    // A server-side master client is a room property; without one PUN takes
                    // the lowest actor number.
                    let lowest = online
                        .names
                        .keys()
                        .copied()
                        .chain([online.client.actor])
                        .min()
                        .unwrap_or(online.client.actor);
                    online.master_id = online.room_master().unwrap_or(lowest);
                }
                ClientEvent::Joined { .. } | ClientEvent::InLobby | ClientEvent::RoomList => {}
                other if joined => online.backlog.push(other),
                _ => {}
            }
        }
        if std::env::var_os("PGA3_NET_DEBUG").is_some() {
            eprintln!("net: room properties {:?}", online.room_props);
            eprintln!("net: master client: {}", online.master_id);
        }
        println!(
            "online: joined as actor {} with {} other players, map {}",
            online.client.actor,
            online.names.len(),
            online.map().unwrap_or("?")
        );
        online
    }

    pub fn actor(&self) -> ActorId {
        self.client.actor
    }

    /// `PhotonNetwork.isMasterClient`.
    pub fn is_master(&self) -> bool {
        self.master_id == self.actor()
    }

    /// The room property 248 (the server-side master client), when the server keeps one.
    fn room_master(&self) -> Option<ActorId> {
        self.room_props
            .get_byte(game_key::MASTER_CLIENT_ID)
            .and_then(Value::as_i32)
            .filter(|&m| m > 0)
    }

    pub fn map(&self) -> Option<&str> {
        self.room_props.get_str("MapName").and_then(Value::as_str)
    }

    pub fn game_mode(&self) -> Option<&str> {
        self.room_props.get_str("GameMode").and_then(Value::as_str)
    }

    pub fn room_int(&self, key: &str) -> Option<i32> {
        self.room_props.get_str(key).and_then(Value::as_i32)
    }

    pub fn room_float(&self, key: &str) -> Option<f32> {
        self.room_props.get_str(key).and_then(Value::as_f32)
    }

    fn update_names(&mut self, actor: ActorId, props: &Value) {
        let player = self.players.entry(actor).or_default();
        if let Some(name) = props
            .get_byte(crate::net::photon::PLAYER_NAME_KEY)
            .and_then(Value::as_str)
        {
            self.names.insert(actor, name.to_string());
            player.name = name.to_string();
        }
        if let Some(team) = props.get_str("TeamName").and_then(Value::as_str) {
            player.team = Some(team.to_string());
        }
        let int = |key: &str| props.get_str(key).and_then(Value::as_i32);
        for (key, field) in [
            ("Kills", &mut player.kills),
            ("Deaths", &mut player.deaths),
            ("Ping", &mut player.ping),
        ] {
            if let Some(value) = int(key) {
                *field = Some(value);
            }
        }
    }

    /// The other players in the room by actor number, with their properties.
    pub fn players(&self) -> Vec<(ActorId, ActorProps)> {
        let mut players: Vec<(ActorId, ActorProps)> = self
            .players
            .iter()
            .filter(|(actor, _)| **actor != self.actor())
            .map(|(actor, props)| (*actor, props.clone()))
            .collect();
        players.sort_by_key(|(actor, _)| *actor);
        players
    }

    /// The room's full name.
    pub fn room_name(&self) -> &str {
        &self.client.room_name
    }

    fn timestamp(&self) -> i32 {
        self.client.server_timestamp()
    }

    fn rpc(&self, view_id: i32, method: &str, args: Vec<Value>) {
        let content = pun::rpc(view_id, method, args, self.timestamp());
        self.client.raise_event(pun::RPC, content, None, None);
    }

    /// `WhoKilledWho.networkAddMessage` to all (the caller shows it locally).
    pub fn send_message(&self, killer: &str, killed: &str, middle: &str, team: &str) {
        self.rpc(
            pun::ROOM_VIEW,
            "networkAddMessage",
            vec![
                Value::str(killer),
                Value::str(killed),
                Value::str(middle),
                Value::str(team),
            ],
        );
    }

    /// What the original sends right after `OnJoinedRoom`.
    pub fn joined(&mut self, name: &str) {
        self.client
            .set_player_props(vec![("TeamName".to_string(), Value::str("Spectators"))]);
        self.client
            .set_player_props(vec![("Kills".to_string(), Value::Int(0))]);
        self.client
            .set_player_props(vec![("Deaths".to_string(), Value::Int(0))]);
        // `RoomMultiplayerMenu.Awake`: the master client starts the round. The other clients
        // unbox `RefTime` in their `Awake`, so a room without it shows them no menu at all.
        if self.is_master() {
            self.start_round(self.photon_time() as f32);
        }
        self.send_message(name, "", "connected", "");
    }

    /// The master client's new round (`Awake`, `Restart`): `RefTime`, then both team scores
    /// at 0, each its own `SetCustomProperties`.
    pub fn start_round(&mut self, reference_time: f32) {
        self.set_room_prop("RefTime", Value::Float(reference_time));
        self.set_room_prop("Team1Score", Value::Int(0));
        self.set_room_prop("Team2Score", Value::Int(0));
    }

    /// `PhotonNetwork.LeaveRoom()`; `NetIn::LeftRoom` follows.
    pub fn leave_room(&mut self) {
        self.client.leave_room();
    }

    /// `PhotonNetwork.time`: the server clock in seconds, `(uint)ServerTimestamp / 1000.0`.
    pub fn photon_time(&self) -> f64 {
        self.timestamp() as u32 as f64 / 1000.0
    }

    /// `Room.SetCustomProperties` of one key; PUN also updates its own copy at once.
    fn set_room_prop(&mut self, key: &str, value: Value) {
        if let Value::Hashtable(room) = &mut self.room_props {
            room.retain(|(k, _)| k.as_str() != Some(key));
            room.push((Value::str(key), value.clone()));
        }
        self.client.set_room_props(vec![(key.to_string(), value)]);
    }

    /// The team menu's choice: `TeamName` (the spawn and the "joined" message follow).
    pub fn choose_team(&self, team: &str) {
        self.client
            .set_player_props(vec![("TeamName".to_string(), Value::str(team))]);
    }

    pub fn set_score(&self, key: &str, value: i32) {
        self.client
            .set_player_props(vec![(key.to_string(), Value::Int(value))]);
    }

    pub fn set_team_score(&mut self, key: &str, value: i32) {
        self.set_room_prop(key, Value::Int(value));
    }

    /// `PhotonNetwork.Instantiate` of this client's player (a new view id each time).
    pub fn spawn(&mut self, kind: usize, position: Vec3, rotation: Quat) {
        self.last_view = self.last_view % (pun::MAX_VIEW_IDS - 1) + 1;
        let view_id = self.actor() * pun::MAX_VIEW_IDS + self.last_view;
        let prefab = PLAYER_PREFABS[kind].name;
        let content = pun::instantiate(prefab, position, rotation, view_id, self.timestamp());
        self.client.raise_event(
            pun::INSTANTIATION,
            content,
            Some(cache::ADD_TO_ROOM_CACHE),
            None,
        );
        self.local_view = Some(view_id);
        self.last_state = None;
        self.reliable_sent = false;
    }

    /// `PhotonNetwork.Destroy` of this client's player object.
    pub fn destroy(&mut self) {
        let Some(view_id) = self.local_view.take() else {
            return;
        };
        self.client
            .remove_cached(pun::INSTANTIATION, pun::view_key(7, view_id));
        self.client
            .raise_event(pun::DESTROY, pun::view_key(0, view_id), None, None);
        self.client.raise_event(
            pun::RPC,
            pun::view_key(0, view_id),
            Some(cache::REMOVE_FROM_ROOM_CACHE),
            None,
        );
    }

    /// The player's `OnPhotonSerializeView`, at PUN's serialization interval.
    pub fn send_state(&mut self, state: &NetState) {
        let Some(view_id) = self.local_view else {
            return;
        };
        let Some(code) = self.serialize_code(state) else {
            return;
        };
        let content = pun::serialize(view_id, pun::state_data(state), self.timestamp());
        self.client.raise_event(code, content, None, None);
    }

    /// `ViewSynchronization.UnreliableOnChange`: a changed state goes out unreliably; the
    /// first unchanged one once reliably, then nothing until it changes.
    fn serialize_code(&mut self, state: &NetState) -> Option<u8> {
        let code = if self.last_state.as_ref() == Some(state) {
            if self.reliable_sent {
                return None;
            }
            self.reliable_sent = true;
            pun::SEND_SERIALIZE_RELIABLE
        } else {
            self.reliable_sent = false;
            pun::SEND_SERIALIZE
        };
        self.last_state = Some(state.clone());
        Some(code)
    }

    /// `WeaponSync_Catcher.Fire`: `SyncWeaponsRPC` to the others.
    pub fn sync_fire(&self, sync: SyncFire) {
        if let Some(view_id) = self.local_view {
            self.rpc(view_id, "SyncWeaponsRPC", pun::sync_fire_args(sync));
        }
    }

    /// `PlayerDamage.SwapTeams`: `RpcSecure("DoSwapTeams", All)` on this client's player (the
    /// caller runs it locally).
    pub fn swap_teams(&self) {
        if let Some(view_id) = self.local_view {
            self.rpc(view_id, "DoSwapTeams", Vec::new());
        }
    }

    /// `HitBox.TotalDamage`: `DoDamage(amount, PhotonNetwork.player)` on `target`'s player object.
    pub fn do_damage(&self, target: ActorId, amount: f32) {
        if let Some(&view_id) = self.views.get(&target) {
            self.rpc(
                view_id,
                "DoDamage",
                vec![Value::Float(amount), Value::player(self.actor())],
            );
        }
    }

    fn warn_once(&mut self, text: String) {
        if !self.warned.contains(&text) {
            eprintln!("online: {text}");
            self.warned.push(text);
        }
    }

    /// Processes the network; `time` is `Time.time`.
    pub fn update(&mut self, time: f32) -> Vec<NetIn> {
        let mut out = Vec::new();
        if time >= self.ping_property_at && self.client.in_room {
            self.ping_property_at = time + PING_PROPERTY_INTERVAL;
            let ping = self.client.ping();
            self.set_score("Ping", ping);
            self.local_ping = Some(ping);
        }
        let mut events = std::mem::take(&mut self.backlog);
        events.extend(self.client.poll());
        for event in events {
            self.client_event(event, &mut out);
        }
        out
    }

    /// One event of the Photon client, as `NetworkingPeer.OnEvent` / `OnOperationResponse`
    /// handle it.
    fn client_event(&mut self, event: ClientEvent, out: &mut Vec<NetIn>) {
        match event {
            ClientEvent::Custom {
                code,
                sender,
                content,
            } => {
                for pun_event in pun::decode(code, &content) {
                    self.pun_event(pun_event, sender, out);
                }
            }
            ClientEvent::ActorJoined { actor, props } => {
                // `AddNewPlayer`, then `ResetPhotonViewsOnSerialize`: every view sends its
                // whole state again, so the newcomer sees players who stand still.
                self.update_names(actor, &props);
                self.last_state = None;
                self.reliable_sent = false;
            }
            ClientEvent::ActorLeft { actor, master } => {
                // `HandleEventLeave`: the master client first (the server's choice, else
                // `CheckMasterClient`), the player's objects, then `OnPhotonPlayerDisconnected`.
                match master {
                    Some(id) => self.master_id = id,
                    None if self.master_id == actor || self.master_id <= 0 => {
                        self.master_id = self
                            .names
                            .keys()
                            .copied()
                            .chain([self.actor()])
                            .filter(|&a| a != actor)
                            .min()
                            .unwrap_or(self.actor());
                    }
                    None => {}
                }
                let name = self.names.remove(&actor).unwrap_or_default();
                self.players.remove(&actor);
                if self.views.remove(&actor).is_some() {
                    out.push(NetIn::Destroy { actor });
                }
                out.push(NetIn::PlayerLeft { name });
            }
            ClientEvent::PlayerProps { actor, props } => self.update_names(actor, &props),
            ClientEvent::RoomProps(props) => {
                let int = |key: &str| props.get_str(key).and_then(Value::as_i32);
                let (team1, team2) = (int("Team1Score"), int("Team2Score"));
                if team1.is_some() || team2.is_some() {
                    out.push(NetIn::RoomScores { team1, team2 });
                }
                if let (Value::Hashtable(room), Some(changed)) =
                    (&mut self.room_props, props.entries())
                {
                    for (key, value) in changed {
                        room.retain(|(k, _)| k != key);
                        room.push((key.clone(), value.clone()));
                    }
                }
                if let Some(master) = self.room_master() {
                    self.master_id = master;
                }
            }
            ClientEvent::LeftRoom => out.push(NetIn::LeftRoom),
            ClientEvent::Failed(reason) => out.push(NetIn::Disconnected(reason)),
            ClientEvent::InLobby | ClientEvent::RoomList | ClientEvent::Joined { .. } => {}
        }
    }

    fn actor_of_view(&self, view_id: i32) -> Option<ActorId> {
        self.views
            .iter()
            .find(|(_, v)| **v == view_id)
            .map(|(a, _)| *a)
    }

    fn pun_event(&mut self, event: PunEvent, sender: ActorId, out: &mut Vec<NetIn>) {
        match event {
            PunEvent::Instantiate {
                view_id,
                prefab,
                position,
                rotation,
            } => {
                let actor = pun::owner_of(view_id);
                let Some(kind) = prefab_kind(&prefab) else {
                    self.warn_once(format!("unknown prefab {prefab} instantiated"));
                    return;
                };
                if actor == self.actor() {
                    return;
                }
                self.views.insert(actor, view_id);
                out.push(NetIn::Instantiate {
                    actor,
                    kind,
                    position,
                    rotation,
                });
            }
            PunEvent::Serialize { view_id, data } => {
                let Some(actor) = self.actor_of_view(view_id) else {
                    return;
                };
                match pun::state_of(&data) {
                    Some(state) => out.push(NetIn::State { actor, state }),
                    None => self.warn_once(format!("unexpected stream of view {view_id}")),
                }
            }
            PunEvent::Rpc {
                view_id,
                method,
                args,
            } => match method.as_str() {
                "SyncWeaponsRPC" => {
                    if let (Some(actor), Some(sync)) =
                        (self.actor_of_view(view_id), pun::sync_fire_of(&args))
                    {
                        out.push(NetIn::SyncWeapons { actor, sync });
                    }
                }
                "DoDamage" => {
                    let target = if Some(view_id) == self.local_view {
                        Some(self.actor())
                    } else {
                        self.actor_of_view(view_id)
                    };
                    let amount = args.first().and_then(Value::as_f32);
                    let damager = args.get(1).and_then(Value::as_player).unwrap_or(sender);
                    if let (Some(target), Some(amount)) = (target, amount) {
                        out.push(NetIn::DoDamage {
                            target,
                            amount,
                            damager,
                        });
                    }
                }
                "DoSwapTeams" => {
                    if let Some(actor) = self.actor_of_view(view_id) {
                        out.push(NetIn::SwapTeams { actor });
                    }
                }
                "networkAddMessage" => {
                    let text = |i: usize| {
                        args.get(i)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string()
                    };
                    out.push(NetIn::Message {
                        killer: text(0),
                        killed: text(1),
                        middle: text(2),
                        team: text(3),
                    });
                }
                other => self.warn_once(format!("RPC {other} is not handled")),
            },
            PunEvent::Destroy { view_id } => {
                if let Some(actor) = self.actor_of_view(view_id) {
                    self.views.remove(&actor);
                    out.push(NetIn::Destroy { actor });
                }
            }
            PunEvent::DestroyPlayer { actor } => {
                if self.views.remove(&actor).is_some() {
                    out.push(NetIn::Destroy { actor });
                }
            }
            PunEvent::CloseConnection => {
                if sender == self.master_id {
                    self.leave_room();
                } else {
                    self.warn_once(format!(
                        "actor {sender}, not the master client, asked this client to leave"
                    ));
                }
            }
        }
    }
}

/// A room as `ConnectMenu` creates it (defaults: TDM, 20 minutes, all weapons). The CrazyGames
/// build lists `MapName`, `RoundDuration`, `GameMode`, `Password` in the lobby (recorded room).
pub fn new_room(map: &str, mode: &str) -> NewRoom {
    let number = 111
        + (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos())
            % (9999 - 111)) as i32;
    NewRoom {
        name: format!("Room {number}: {mode}[WEP02][WEP03][WEP04][WEP05][WEP06]{ROOM_MARKER}"),
        max_players: 16,
        props: vec![
            ("MapName".to_string(), Value::str(map)),
            ("RoundDuration".to_string(), Value::Int(1200)),
            ("GameMode".to_string(), Value::str(mode)),
            ("Password".to_string(), Value::str("")),
        ],
        lobby_props: ["MapName", "RoundDuration", "GameMode", "Password"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::photon::PLAYER_NAME_KEY;

    fn named(name: &str) -> Value {
        Value::Hashtable(vec![(Value::Byte(PLAYER_NAME_KEY), Value::str(name))])
    }

    fn state(name: &str, x: f32) -> NetState {
        NetState {
            position: Vec3::new(x, 1.0, 2.0),
            rotation: Quat::IDENTITY,
            name: name.to_string(),
            look_target_position: Vec3::ZERO,
            look_target_rotation: Quat::IDENTITY,
            team: "Mercenary".to_string(),
            animation: "idle".to_string(),
            hands_animation: "Null".to_string(),
            weapon: "M242".to_string(),
            animation_type: "Walking".to_string(),
            spawn_protected: false,
        }
    }

    /// This client (`actor`) just joined a room holding `others`, with what came after the join
    /// answer in the same batch.
    fn joined(
        actor: ActorId,
        others: &[(ActorId, &str)],
        room: Value,
        rest: Vec<ClientEvent>,
    ) -> Online {
        let mut events = vec![ClientEvent::Joined {
            actors: others.iter().map(|(a, n)| (*a, named(n))).collect(),
            room_props: room,
        }];
        events.extend(rest);
        Online::from_events(Client::offline(actor), events)
    }

    fn instantiation(actor: ActorId, prefab: &str) -> ClientEvent {
        ClientEvent::Custom {
            code: pun::INSTANTIATION,
            sender: actor,
            content: pun::instantiate(
                prefab,
                Vec3::new(1.0, 2.0, 3.0),
                Quat::IDENTITY,
                actor * pun::MAX_VIEW_IDS + 1,
                0,
            ),
        }
    }

    #[test]
    fn cached_events_that_came_with_the_join_answer_are_kept() {
        // The bug: the room's cached instantiation arrives in the same batch as the join
        // answer and was dropped, so the late joiner never saw players already in the room.
        let serialize = ClientEvent::Custom {
            code: pun::SEND_SERIALIZE_RELIABLE,
            sender: 1,
            content: pun::serialize(1001, pun::state_data(&state("A", 5.0)), 0),
        };
        let mut online = joined(
            2,
            &[(1, "A")],
            Value::Hashtable(Vec::new()),
            vec![instantiation(1, "German"), serialize],
        );
        let incoming = online.update(0.0);
        assert!(matches!(
            incoming[0],
            NetIn::Instantiate {
                actor: 1,
                kind: 1,
                ..
            }
        ));
        match &incoming[1] {
            NetIn::State { actor: 1, state } => assert_eq!(state.position.x, 5.0),
            other => panic!("expected the stream, got {other:?}"),
        }
        assert!(online.update(0.1).is_empty(), "the backlog is handled once");
    }

    #[test]
    fn a_joining_player_gets_the_whole_state_again() {
        let mut online = joined(1, &[], Value::Hashtable(Vec::new()), Vec::new());
        let still = state("B", 0.0);
        assert_eq!(online.serialize_code(&still), Some(pun::SEND_SERIALIZE));
        assert_eq!(
            online.serialize_code(&still),
            Some(pun::SEND_SERIALIZE_RELIABLE)
        );
        assert_eq!(
            online.serialize_code(&still),
            None,
            "UnreliableOnChange: nothing new"
        );
        let mut out = Vec::new();
        online.client_event(
            ClientEvent::ActorJoined {
                actor: 2,
                props: named("C"),
            },
            &mut out,
        );
        assert_eq!(
            online.serialize_code(&still),
            Some(pun::SEND_SERIALIZE),
            "ResetPhotonViewsOnSerialize"
        );
    }

    #[test]
    fn the_master_client_follows_the_server_or_the_lowest_actor() {
        let room = Value::Hashtable(vec![(
            Value::Byte(game_key::MASTER_CLIENT_ID),
            Value::Int(1),
        )]);
        let mut online = joined(3, &[(1, "A"), (2, "B")], room, Vec::new());
        assert!(!online.is_master());
        let mut out = Vec::new();
        // The server names the new master client.
        online.client_event(
            ClientEvent::ActorLeft {
                actor: 1,
                master: Some(3),
            },
            &mut out,
        );
        assert!(online.is_master());
        assert!(matches!(&out[0], NetIn::PlayerLeft { name } if name == "A"));

        // Without the server's choice: `CheckMasterClient`, the lowest remaining actor.
        let mut online = joined(
            3,
            &[(1, "A"), (2, "B")],
            Value::Hashtable(Vec::new()),
            Vec::new(),
        );
        assert_eq!(online.master_id, 1);
        online.client_event(
            ClientEvent::ActorLeft {
                actor: 2,
                master: None,
            },
            &mut out,
        );
        assert_eq!(online.master_id, 1, "a leaving non-master changes nothing");
        online.client_event(
            ClientEvent::ActorLeft {
                actor: 1,
                master: None,
            },
            &mut out,
        );
        assert!(online.is_master());

        // A room created by this client.
        assert!(joined(1, &[], Value::Hashtable(Vec::new()), Vec::new()).is_master());
    }

    #[test]
    fn a_leaving_player_takes_its_object_with_it() {
        let mut online = joined(2, &[(1, "A")], Value::Hashtable(Vec::new()), Vec::new());
        let mut out = Vec::new();
        online.client_event(instantiation(1, "Zombie"), &mut out);
        assert!(matches!(out[0], NetIn::Instantiate { kind: 2, .. }));
        online.client_event(
            ClientEvent::ActorLeft {
                actor: 1,
                master: None,
            },
            &mut out,
        );
        assert!(matches!(out[1], NetIn::Destroy { actor: 1 }));
        assert!(matches!(&out[2], NetIn::PlayerLeft { name } if name == "A"));
        assert!(online.players().is_empty());
        online.client_event(instantiation(1, "Tank"), &mut out);
        assert_eq!(out.len(), 3, "an unknown prefab is ignored");
    }
}
