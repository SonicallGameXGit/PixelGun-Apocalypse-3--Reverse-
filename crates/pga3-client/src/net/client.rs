//! Photon LoadBalancing client: name server → master server (lobby, room list) → game server
//! (the room), as `PhotonNetwork.ConnectToRegion` / `JoinLobby` / `JoinRoom` run it. Every step
//! follows the recorded CrazyGames session (`docs/protocol.md`):
//!
//! 1. Name server `wss://ns.exitgames.com:19093/`: `Authenticate {220 AppVersion, 224 AppId,
//!    210 region}` → `{230 master address, 221 secret}`.
//! 2. Master: `Authenticate {221 secret}`, `JoinLobby {}` → events `GameList` / `GameListUpdate`.
//! 3. `JoinGame {255 name}` on the master → `{230 game server address, 221 secret}`.
//! 4. Game server: `Authenticate {221}`, `JoinGame {255 name, 249 {255: player name}, 250 true}` →
//!    `{254 actor number, 249 actors' properties, 248 room properties, 252 actor list}`.

use std::time::Instant;

use anyhow::{Result, bail};

use super::photon::{self, Message, Params, cache, event, game_key, op, param};
use super::protocol16::Value;
use super::socket::{Socket, SocketEvent};
use crate::remote::ActorId;

/// What identifies a game build to Photon: rooms are only shared by clients with the same AppId
/// and AppVersion in the same region.
#[derive(Clone, Copy, Debug)]
pub struct AppProfile {
    pub name: &'static str,
    pub app_id: &'static str,
    /// `gameVersion + "_" + PhotonNetwork.versionPUN`.
    pub app_version: &'static str,
    pub name_server: &'static str,
}

/// A room as the lobby lists it.
#[derive(Clone, Debug)]
pub struct RoomInfo {
    pub name: String,
    /// The room's properties as listed (custom ones by string key, Photon's by byte key).
    pub props: Value,
    pub players: i32,
    pub max_players: i32,
    pub open: bool,
}

impl RoomInfo {
    fn from_props(name: &str, props: &Value) -> RoomInfo {
        RoomInfo {
            name: name.to_string(),
            props: props.clone(),
            players: props
                .get_byte(game_key::PLAYER_COUNT)
                .and_then(Value::as_i32)
                .unwrap_or(0),
            max_players: props
                .get_byte(game_key::MAX_PLAYERS)
                .and_then(Value::as_i32)
                .unwrap_or(0),
            open: props
                .get_byte(game_key::IS_OPEN)
                .and_then(Value::as_bool)
                .unwrap_or(true),
        }
    }

    pub fn prop_str(&self, key: &str) -> Option<&str> {
        self.props.get_str(key).and_then(Value::as_str)
    }
}

/// A room to create (`PhotonNetwork.CreateRoom(name, RoomOptions, null)`).
#[derive(Clone, Debug)]
pub struct NewRoom {
    pub name: String,
    pub max_players: u8,
    /// `customRoomProperties`, in insertion order.
    pub props: Vec<(String, Value)>,
    /// `customRoomPropertiesForLobby`.
    pub lobby_props: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum ClientEvent {
    /// Joined the master's lobby: the room list follows.
    InLobby,
    /// `rooms` changed.
    RoomList,
    /// In the room on the game server.
    Joined {
        /// Every actor in the room (this client included) with its properties.
        actors: Vec<(ActorId, Value)>,
        room_props: Value,
    },
    /// Another player entered the room (event `Join`).
    ActorJoined { actor: ActorId, props: Value },
    /// A player left the room (event `Leave`).
    ActorLeft { actor: ActorId },
    /// Changed player properties (event `PropertiesChanged` with a target actor).
    PlayerProps { actor: ActorId, props: Value },
    /// Changed room properties.
    RoomProps(Value),
    /// A custom event (PUN's RPC, instantiation, serialization, destruction).
    Custom {
        code: u8,
        sender: ActorId,
        content: Value,
    },
    /// The connection failed or ended.
    Failed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Server {
    Name,
    Master,
    Game,
}

#[derive(Clone, Debug)]
enum Pending {
    Join(String),
    Create(NewRoom),
}

pub struct Client {
    pub profile: AppProfile,
    pub region: String,
    pub player_name: String,
    socket: Option<Socket>,
    server: Server,
    secret: Option<String>,
    master_address: Option<String>,
    pending: Option<Pending>,
    pub rooms: Vec<RoomInfo>,
    pub actor: ActorId,
    pub in_room: bool,
    epoch: Instant,
}

impl Client {
    /// Starts `ConnectToRegion(region)`.
    pub fn connect(profile: AppProfile, region: &str, player_name: &str) -> Client {
        let epoch = Instant::now();
        Client {
            profile,
            region: region.to_string(),
            player_name: player_name.to_string(),
            socket: Some(Socket::open(profile.name_server, epoch)),
            server: Server::Name,
            secret: None,
            master_address: None,
            pending: None,
            rooms: Vec::new(),
            actor: 0,
            in_room: false,
            epoch,
        }
    }

    /// `PhotonNetwork.GetPing()`.
    pub fn ping(&self) -> i32 {
        self.socket.as_ref().map_or(0, Socket::rtt)
    }

    /// `PhotonNetwork.ServerTimestamp`.
    pub fn server_timestamp(&self) -> i32 {
        self.socket.as_ref().map_or(0, Socket::server_timestamp)
    }

    fn send(&self, message: Message) {
        if let Some(socket) = &self.socket {
            socket.send(&message);
        }
    }

    fn switch_to(&mut self, server: Server, address: &str) {
        let url = if server == Server::Game || address.ends_with('/') {
            address.to_string()
        } else {
            format!("{address}/")
        };
        self.server = server;
        self.socket = Some(Socket::open(&url, self.epoch));
    }

    /// `PhotonNetwork.JoinRoom(name)` (from the lobby).
    pub fn join_room(&mut self, name: &str) -> Result<()> {
        if self.server != Server::Master {
            bail!("not in the lobby");
        }
        self.pending = Some(Pending::Join(name.to_string()));
        self.send(Message::request(
            op::JOIN_GAME,
            vec![(param::ROOM_NAME, Value::str(name))],
        ));
        Ok(())
    }

    /// `PhotonNetwork.CreateRoom(name, options, null)` (from the lobby).
    pub fn create_room(&mut self, room: NewRoom) -> Result<()> {
        if self.server != Server::Master {
            bail!("not in the lobby");
        }
        let params = vec![
            (param::ROOM_NAME, Value::str(&room.name)),
            (param::GAME_PROPERTIES, game_properties(&room)),
        ];
        self.pending = Some(Pending::Create(room));
        self.send(Message::request(op::CREATE_GAME, params));
        Ok(())
    }

    /// `PhotonPlayer.SetCustomProperties` of this client's player.
    pub fn set_player_props(&self, props: Vec<(String, Value)>) {
        self.send(Message::request(
            op::SET_PROPERTIES,
            vec![
                (param::PROPERTIES, string_table(props)),
                (param::ACTOR_NR, Value::Int(self.actor)),
                (param::BROADCAST, Value::Bool(true)),
            ],
        ));
    }

    /// `Room.SetCustomProperties`.
    pub fn set_room_props(&self, props: Vec<(String, Value)>) {
        self.send(Message::request(
            op::SET_PROPERTIES,
            vec![
                (param::PROPERTIES, string_table(props)),
                (param::BROADCAST, Value::Bool(true)),
            ],
        ));
    }

    /// `OpRaiseEvent(code, content, options)`: to the other actors, optionally cached or removing
    /// cached events of `targets`.
    pub fn raise_event(
        &self,
        code: u8,
        content: Value,
        cache_option: Option<u8>,
        targets: Option<Vec<ActorId>>,
    ) {
        let mut params: Params = vec![(param::CODE, Value::Byte(code)), (param::DATA, content)];
        if let Some(c) = cache_option {
            params.push((param::CACHE, Value::Byte(c)));
        }
        if let Some(targets) = targets {
            params.push((
                param::ACTOR_LIST,
                Value::Array(
                    super::protocol16::INT,
                    targets.into_iter().map(Value::Int).collect(),
                ),
            ));
        }
        self.send(Message::request(op::RAISE_EVENT, params));
    }

    /// Removes cached events of this client (`RemoveFromRoomCache` with this actor as filter).
    pub fn remove_cached(&self, code: u8, content: Value) {
        self.raise_event(
            code,
            content,
            Some(cache::REMOVE_FROM_ROOM_CACHE),
            Some(vec![self.actor]),
        );
    }

    /// Processes what arrived; call once per frame.
    pub fn poll(&mut self) -> Vec<ClientEvent> {
        let mut out = Vec::new();
        let events = match &self.socket {
            Some(socket) => socket.poll(),
            None => return out,
        };
        let server = self.server;
        for event in events {
            match event {
                SocketEvent::Message(message) => {
                    if let Err(error) = self.handle(message, &mut out) {
                        out.push(ClientEvent::Failed(format!("{error:#}")));
                        self.socket = None;
                    }
                }
                SocketEvent::Closed(reason) => {
                    out.push(ClientEvent::Failed(format!(
                        "{:?} server connection ended: {reason}",
                        self.server
                    )));
                    self.socket = None;
                    self.in_room = false;
                }
            }
            // A server switch replaced the socket: the old one's remaining events are moot.
            if self.socket.is_none() || self.server != server {
                break;
            }
        }
        out
    }

    fn handle(&mut self, message: Message, out: &mut Vec<ClientEvent>) -> Result<()> {
        match message {
            Message::InitResponse => self.authenticate(),
            Message::Response {
                op,
                code,
                debug,
                params,
                ..
            } => {
                if code != 0 {
                    bail!(
                        "{:?} server: operation {op} failed with {code}: {}",
                        self.server,
                        debug.unwrap_or_default()
                    );
                }
                self.response(op, params, out)?;
            }
            Message::Event { code, params } => self.event(code, params, out),
            Message::Request { .. } => {}
        }
        Ok(())
    }

    fn authenticate(&self) {
        let params = match (self.server, &self.secret) {
            (Server::Name, _) => vec![
                (param::APP_VERSION, Value::str(self.profile.app_version)),
                (param::APP_ID, Value::str(self.profile.app_id)),
                (param::REGION, Value::str(&self.region)),
            ],
            (_, Some(secret)) => vec![(param::SECRET, Value::str(secret))],
            (_, None) => vec![],
        };
        self.send(Message::request(op::AUTHENTICATE, params));
    }

    fn response(&mut self, code: u8, params: Params, out: &mut Vec<ClientEvent>) -> Result<()> {
        if let Some(secret) = photon::get(&params, param::SECRET).and_then(Value::as_str) {
            self.secret = Some(secret.to_string());
        }
        let address = photon::get(&params, param::ADDRESS)
            .and_then(Value::as_str)
            .map(str::to_string);
        match (self.server, code) {
            (Server::Name, op::AUTHENTICATE) => {
                let Some(address) = address else {
                    bail!("name server sent no master address");
                };
                self.master_address = Some(address.clone());
                self.switch_to(Server::Master, &address);
            }
            (Server::Master, op::AUTHENTICATE) => {
                self.send(Message::request(op::JOIN_LOBBY, vec![]));
            }
            (Server::Master, op::JOIN_LOBBY) => out.push(ClientEvent::InLobby),
            (Server::Master, op::JOIN_GAME | op::CREATE_GAME) => {
                let Some(address) = address else {
                    bail!("master sent no game server address");
                };
                self.switch_to(Server::Game, &address);
            }
            (Server::Game, op::AUTHENTICATE) => self.enter_room(),
            (Server::Game, op::JOIN_GAME | op::CREATE_GAME) => {
                self.actor = photon::get(&params, param::ACTOR_NR)
                    .and_then(Value::as_i32)
                    .unwrap_or(0);
                let actors = photon::get(&params, param::PLAYER_PROPERTIES)
                    .and_then(Value::entries)
                    .map(|entries| {
                        entries
                            .iter()
                            .filter_map(|(k, v)| Some((k.as_i32()?, v.clone())))
                            .collect()
                    })
                    .unwrap_or_default();
                let room_props = photon::get(&params, param::GAME_PROPERTIES)
                    .cloned()
                    .unwrap_or(Value::Hashtable(Vec::new()));
                self.in_room = true;
                out.push(ClientEvent::Joined { actors, room_props });
            }
            _ => {}
        }
        Ok(())
    }

    /// The game server's join/create of the pending room, with this player's name.
    fn enter_room(&mut self) {
        let player = Value::Hashtable(vec![(
            Value::Byte(photon::PLAYER_NAME_KEY),
            Value::str(&self.player_name),
        )]);
        let message = match &self.pending {
            Some(Pending::Join(name)) => Message::request(
                op::JOIN_GAME,
                vec![
                    (param::ROOM_NAME, Value::str(name)),
                    (param::PLAYER_PROPERTIES, player),
                    (param::BROADCAST, Value::Bool(true)),
                ],
            ),
            Some(Pending::Create(room)) => Message::request(
                op::CREATE_GAME,
                vec![
                    (param::ROOM_NAME, Value::str(&room.name)),
                    (param::GAME_PROPERTIES, game_properties(room)),
                    (param::PLAYER_PROPERTIES, player),
                    (param::BROADCAST, Value::Bool(true)),
                    (param::CLEANUP_CACHE_ON_LEAVE, Value::Bool(true)),
                ],
            ),
            None => return,
        };
        self.send(message);
    }

    fn event(&mut self, code: u8, params: Params, out: &mut Vec<ClientEvent>) {
        let actor_of = |params: &Params| {
            photon::get(params, param::ACTOR_NR)
                .and_then(Value::as_i32)
                .unwrap_or(0)
        };
        match (self.server, code) {
            (Server::Master, event::GAME_LIST | event::GAME_LIST_UPDATE) => {
                if code == event::GAME_LIST {
                    self.rooms.clear();
                }
                if let Some(entries) =
                    photon::get(&params, param::GAME_LIST).and_then(Value::entries)
                {
                    for (name, props) in entries {
                        let Some(name) = name.as_str() else { continue };
                        self.rooms.retain(|r| r.name != name);
                        let removed = props
                            .get_byte(game_key::REMOVED)
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        if !removed {
                            self.rooms.push(RoomInfo::from_props(name, props));
                        }
                    }
                }
                out.push(ClientEvent::RoomList);
            }
            (Server::Game, event::JOIN) => {
                let actor = actor_of(&params);
                if actor != self.actor {
                    out.push(ClientEvent::ActorJoined {
                        actor,
                        props: photon::get(&params, param::PLAYER_PROPERTIES)
                            .cloned()
                            .unwrap_or(Value::Hashtable(Vec::new())),
                    });
                }
            }
            (Server::Game, event::LEAVE) => out.push(ClientEvent::ActorLeft {
                actor: actor_of(&params),
            }),
            (Server::Game, event::PROPERTIES_CHANGED) => {
                let props = photon::get(&params, param::PROPERTIES)
                    .cloned()
                    .unwrap_or(Value::Hashtable(Vec::new()));
                match photon::get(&params, param::TARGET_ACTOR).and_then(Value::as_i32) {
                    Some(actor) if actor > 0 => out.push(ClientEvent::PlayerProps { actor, props }),
                    _ => out.push(ClientEvent::RoomProps(props)),
                }
            }
            // Custom events (Photon reserves the codes from 224 up).
            (Server::Game, code) if code < 224 => {
                out.push(ClientEvent::Custom {
                    code,
                    sender: actor_of(&params),
                    content: photon::get(&params, param::DATA)
                        .cloned()
                        .unwrap_or(Value::Null),
                });
            }
            _ => {}
        }
    }
}

fn string_table(props: Vec<(String, Value)>) -> Value {
    Value::Hashtable(
        props
            .into_iter()
            .map(|(k, v)| (Value::String(k), v))
            .collect(),
    )
}

/// `RoomOptions` as game properties: Photon's byte keys, then the custom properties.
fn game_properties(room: &NewRoom) -> Value {
    let mut entries = vec![
        (
            Value::Byte(game_key::MAX_PLAYERS),
            Value::Byte(room.max_players),
        ),
        (Value::Byte(game_key::IS_VISIBLE), Value::Bool(true)),
        (Value::Byte(game_key::IS_OPEN), Value::Bool(true)),
        (
            Value::Byte(game_key::PROPS_LISTED_IN_LOBBY),
            Value::Array(
                super::protocol16::STRING,
                room.lobby_props.iter().map(|p| Value::str(p)).collect(),
            ),
        ),
    ];
    for (key, value) in &room.props {
        entries.push((Value::String(key.clone()), value.clone()));
    }
    Value::Hashtable(entries)
}
