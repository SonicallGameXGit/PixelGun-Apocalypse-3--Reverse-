//! The room around the players, without the network (spec: `docs/gameplay/remote-players.md`):
//! `RoomMultiplayerMenu` (`f29720` Awake: teams, colours, game mode; `SpawnPlayer`: team spawn
//! points and player prefab), `WhoKilledWho` (`f61870`..`f61887`: the kill feed) and a test
//! "dummy" owner that stands in for another client: it simulates that client's own copy of its
//! player (movement, `CharacterAnimation`, `PlayerDamage`, spawn protection, death and respawn)
//! and produces what the network would carry to this client.

use glam::{Quat, Vec3, Vec4};

use crate::anim::Trs;
use crate::character::{CharacterAnimation, CharacterInput, SelectedWeapon};
use crate::damage::{DamageEvent, PlayerDamage, RagdollController, SpawnProtection};
use crate::gui::{Gui, GuiFont, GuiSkin};
use crate::physics::CollisionWorld;
use crate::player::{FIXED_DELTA_TIME, FrameInput, Player, PlayerEvent};
use crate::prefab::PrefabInstance;
use crate::remote::{ActorId, NetState};
use crate::scene::{LadderInstance, Scene};
use crate::unity::Random;
use crate::weapons::SyncFire;

/// PUN's `sendIntervalOnSerialize` (100 ms, set in the `PhotonNetwork` constructor).
pub const SERIALIZE_INTERVAL: f32 = 0.1;

/// `RoomMultiplayerMenu.team_1` / `team_2`.
#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub color: Vec4,
    /// Indices into `Scene::spawn_points`.
    pub spawn_points: Vec<usize>,
}

/// One `WhoKilledWho.WhoKillWho` entry.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedEntry {
    pub killer: String,
    pub killed: String,
    pub middle: String,
    pub killer_color: Vec4,
    pub killed_color: Vec4,
    pub timer: f32,
}

/// A player's Photon custom properties as the room scripts use them.
#[derive(Clone, Debug)]
pub struct PlayerProps {
    pub actor: ActorId,
    /// `PhotonNetwork.player.name` (also the player GameObject's name).
    pub name: String,
    /// `TeamName`.
    pub team: String,
    pub kills: i32,
    pub deaths: i32,
}

pub struct Room {
    /// Room property `GameMode`: "DM" or "TDM".
    pub game_mode: String,
    pub team_1: Team,
    pub team_2: Team,
    /// `allSpawnPoints`: team 1's then team 2's.
    pub all_spawn_points: Vec<usize>,
    /// Room properties `Team1Score` / `Team2Score`.
    pub team1_score: i32,
    pub team2_score: i32,
    /// `WhoKilledWho.list`.
    pub feed: Vec<FeedEntry>,
    random: Random,
}

impl Room {
    /// `RoomMultiplayerMenu.Awake`: team names and colours (overriding the scene's values), the
    /// game mode, the spawn point lists (every scene's team_1 list is its "Team A SpawnPoints",
    /// team_2's its "Team B SpawnPoints").
    pub fn new(scene: &Scene, game_mode: &str) -> Room {
        let points = |team: &str| -> Vec<usize> {
            (0..scene.spawn_points.len())
                .filter(|&i| scene.spawn_points[i].team == team)
                .collect()
        };
        let team_1 = Team {
            name: "German-ZM".to_string(),
            color: Vec4::new(1.0, 0.921_568_63, 0.015_686_275, 1.0),
            spawn_points: points("Team A"),
        };
        let team_2 = Team {
            name: "Mercenary".to_string(),
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            spawn_points: points("Team B"),
        };
        let all_spawn_points = team_1
            .spawn_points
            .iter()
            .chain(&team_2.spawn_points)
            .copied()
            .collect();
        Room {
            game_mode: game_mode.to_string(),
            team_1,
            team_2,
            all_spawn_points,
            team1_score: 0,
            team2_score: 0,
            feed: Vec::new(),
            random: Random::new(11),
        }
    }

    /// `SpawnPlayer(team)`: team 1 spawns at a random team-1 point in TDM, anywhere in DM, as
    /// `playerPrefab` (Mercenarys); any other team at a random team-2 point as
    /// `playerPrefabTwo` (German). Returns (spawn point, is `playerPrefab`).
    pub fn spawn_point(&mut self, team: &str) -> (Option<usize>, bool) {
        let team_1 = team == self.team_1.name;
        let list = if !team_1 {
            &self.team_2.spawn_points
        } else if self.game_mode == "TDM" {
            &self.team_1.spawn_points
        } else {
            &self.all_spawn_points
        };
        if list.is_empty() {
            return (None, team_1);
        }
        let index = self.random.range_int(0, list.len() as i32) as usize;
        (Some(list[index]), team_1)
    }

    /// `networkAddMessage(killer, killed, middleText, teamName)` (an RPC to all).
    pub fn add_message(&mut self, killer: &str, killed: &str, middle: &str, team: &str) {
        let (killer_color, killed_color) = if self.game_mode == "TDM" {
            if team.is_empty() {
                (Vec4::ONE, Vec4::ONE)
            } else if team == self.team_1.name {
                (self.team_1.color, self.team_2.color)
            } else {
                (self.team_2.color, self.team_1.color)
            }
        } else {
            (self.team_1.color, self.team_1.color)
        };
        self.feed.push(FeedEntry {
            killer: killer.to_string(),
            killed: killed.to_string(),
            middle: middle.to_string(),
            killer_color,
            killed_color,
            timer: 5.0,
        });
        if self.feed.len() > 5 {
            self.feed.remove(0);
        }
    }

    /// `AddKillNotification(killed)` (sent by `PlayerDamage` when this client's player killed
    /// `killed`): the feed message, the killer's Kills and the team score. (The original also
    /// increments a persistent `PlayerPrefs` "Kills" counter, which this client does not keep.)
    pub fn kill_notification(&mut self, killer: &mut PlayerProps, killed: &str) {
        let (name, team) = (killer.name.clone(), killer.team.clone());
        self.add_message(&name, killed, "killed", &team);
        killer.kills += 1;
        if team == self.team_1.name {
            self.team1_score += 1;
        } else if team == self.team_2.name {
            self.team2_score += 1;
        }
    }

    /// `WhoKilledWho.Update`: timers run down; an expired entry is removed (and, as in the
    /// original, the entry after it is skipped this frame).
    pub fn update(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.feed.len() {
            self.feed[i].timer -= dt;
            if self.feed[i].timer <= 0.0 {
                self.feed.remove(i);
            }
            i += 1;
        }
    }

    /// `WhoKilledWho.OnGUI`: right-aligned boxes in `BeginArea(W - W/3.5, 85, W/3.5, 400)`.
    /// The GUILayout spacing follows Unity's group rules from memory (docs/unknowns.md).
    pub fn feed_gui(&self, gui: &mut Gui, font: &mut GuiFont, skin: &GuiSkin, w: f32) {
        if self.feed.is_empty() {
            return;
        }
        gui.begin_script();
        let area = [w - w / 3.5, 85.0, w / 3.5, 400.0];
        let b = &skin.box_style;
        let l = &skin.label;
        let mut y = area[1] + b.margin[2];
        let dim = Vec4::new(1.0, 1.0, 1.0, 0.65);
        for entry in &self.feed {
            let texts = [&entry.killer, &entry.middle, &entry.killed];
            let sizes: Vec<(f32, f32)> = texts.iter().map(|t| gui.calc_size(font, t, l)).collect();
            let label_height = sizes.iter().map(|s| s.1).fold(0.0, f32::max);
            let height = b.padding[2] + l.margin[2] + label_height + l.margin[3] + b.padding[3];
            let x = area[0] + b.margin[0];
            let width = area[2] - b.margin[0] - b.margin[1];
            gui.color = dim;
            gui.box_(font, [x, y, width, height], "", b);
            // FlexibleSpace, killer, Space(5), middle, Space(5), killed, Space(10): laid out
            // from the right edge of the box's content.
            let mut right = x + width - b.padding[1] - 10.0;
            let colors = [entry.killer_color, dim, entry.killed_color];
            let mut rects = [[0.0; 4]; 3];
            for i in (0..3).rev() {
                let (tw, th) = sizes[i];
                right -= l.margin[1];
                rects[i] = [right - tw, y + b.padding[2] + l.margin[2], tw, th];
                right -= tw + l.margin[0];
                if i > 0 {
                    right -= 5.0;
                }
            }
            for i in 0..3 {
                gui.color = colors[i];
                gui.label(font, rects[i], texts[i], l);
            }
            y += height + b.margin[2].max(b.margin[3]);
        }
    }
}

/// `OnPhotonSerializeView` written by the owner of `player`.
pub fn owner_state(
    player: &Player,
    look_target_local: &Trs,
    character: Option<&CharacterAnimation>,
    props: &PlayerProps,
    spawn_protected: bool,
) -> NetState {
    let look_position = player.position + player.rotation * player.look_local_position;
    let text =
        |f: fn(&CharacterAnimation) -> &String| character.map(f).cloned().unwrap_or_default();
    NetState {
        position: player.position,
        rotation: player.rotation,
        name: props.name.clone(),
        look_target_position: look_position + player.look_rotation() * look_target_local.position,
        look_target_rotation: player.look_rotation() * look_target_local.rotation,
        team: props.team.clone(),
        animation: text(|c| &c.animation_sync_helper),
        hands_animation: text(|c| &c.animation_for_hands),
        weapon: text(|c| &c.active_weapon),
        animation_type: text(|c| &c.animation_type),
        spawn_protected,
    }
}

/// What the dummy's client sent this frame, in order.
#[derive(Clone, Debug, PartialEq)]
pub enum OwnerEvent {
    /// `PhotonNetwork.Instantiate` of its player prefab (`mercenarys` when `team_1_prefab`).
    Instantiate {
        position: Vec3,
        rotation: Quat,
        team_1_prefab: bool,
    },
    Serialize(NetState),
    /// `SyncWeaponsRPC` to the others.
    SyncWeapons(SyncFire),
    /// `DoDamage` to all (its own fall damage).
    DoDamage(f32),
    /// `networkAddMessage` to all.
    Message {
        killer: String,
        killed: String,
        middle: String,
        team: String,
    },
    /// `PhotonNetwork.Destroy` of its player.
    Destroy,
}

/// The dummy's selected weapon as `CharacterAnimation` and `WeaponSync_Catcher` see it.
#[derive(Clone, Debug)]
pub struct DummyWeapon {
    pub node: usize,
    pub name: String,
    /// What `WeaponSync_Catcher.Fire` sends for it (aimless values).
    pub sync: SyncFire,
    /// Seconds between shots when `--dummy-fire` is on (the weapon's fire rate).
    pub interval: f32,
}

/// Another client, simulated: its player as its owner runs it (no input).
pub struct DummyOwner {
    pub props: PlayerProps,
    pub player: Option<Player>,
    prefab: PrefabInstance,
    character: Option<CharacterAnimation>,
    damage: Option<PlayerDamage>,
    protection: SpawnProtection,
    ragdoll: Option<RagdollController>,
    pub weapon: Option<DummyWeapon>,
    pub fire: bool,
    look_target_local: Trs,
    fixed_time: f32,
    serialize_at: f32,
    fire_at: f32,
    joined: bool,
}

impl DummyOwner {
    pub fn new(props: PlayerProps, fire: bool) -> DummyOwner {
        DummyOwner {
            props,
            player: None,
            prefab: PrefabInstance::empty(),
            character: None,
            damage: None,
            protection: SpawnProtection::default(),
            ragdoll: None,
            weapon: None,
            fire,
            look_target_local: Trs::IDENTITY,
            fixed_time: 0.0,
            serialize_at: 0.0,
            fire_at: 0.0,
            joined: false,
        }
    }

    /// The dummy's owner-side `DoDamage` (the RPC from this client's copy of it).
    pub fn receive_do_damage(
        &mut self,
        amount: f32,
        damager: ActorId,
        time: f32,
    ) -> Vec<OwnerEvent> {
        let mut events = Vec::new();
        let protected = self.protection.protected;
        let Some(damage) = &mut self.damage else {
            return events;
        };
        if let Some(DamageEvent::Died { .. }) = damage.do_damage(amount, protected, time) {
            // Its death prefab's RespawnAfter, Deaths + 1, PlayerFellDown when self-inflicted.
            if let Some(ragdoll) = &mut self.ragdoll {
                ragdoll.respawn_after(time);
            }
            self.props.deaths += 1;
            if damager == self.props.actor {
                events.push(OwnerEvent::Message {
                    killer: self.props.name.clone(),
                    killed: String::new(),
                    middle: "fell down".to_string(),
                    team: String::new(),
                });
            }
        }
        events
    }

    /// `SpawnPlayer` on the dummy's client: its new player prefab at `spawn` (a fresh copy of
    /// `pristine` for `CharacterAnimation`).
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        mut player: Player,
        pristine: &PrefabInstance,
        clips: &mut crate::anim::ClipLibrary,
        ragdoll: Option<RagdollController>,
        weapon: Option<DummyWeapon>,
        position: Vec3,
        rotation: Quat,
        team_1_prefab: bool,
        time: f32,
    ) -> Vec<OwnerEvent> {
        let mut events = Vec::new();
        if !self.joined {
            // WhoKilledWho.Start (PlayerJoinedRoom) and the team menu's PlayerJoinedTeam.
            self.joined = true;
            for middle in [
                "connected".to_string(),
                format!("joined {}", self.props.team),
            ] {
                events.push(OwnerEvent::Message {
                    killer: self.props.name.clone(),
                    killed: String::new(),
                    middle,
                    team: String::new(),
                });
            }
        }
        player.spawn(position, rotation);
        let mut prefab = pristine.clone();
        prefab.play_automatically();
        self.character = CharacterAnimation::start(&mut prefab, clips);
        self.look_target_local = prefab
            .script_fields(0, "PlayerNetworkController")
            .and_then(|f| prefab.node_of_ref(&f["lookTarget"]))
            .map_or(Trs::IDENTITY, |n| prefab.nodes[n].local);
        self.damage = prefab
            .script_fields(0, "PlayerDamage")
            .map(|f| PlayerDamage::awake(f, player.camera_local_rotation));
        self.prefab = prefab;
        self.protection = SpawnProtection::default();
        self.ragdoll = ragdoll;
        self.weapon = weapon;
        self.fixed_time = 0.0;
        events.push(OwnerEvent::Instantiate {
            position,
            rotation,
            team_1_prefab,
        });
        // Assumed: the first `OnPhotonSerializeView` reaches the others with the instantiation,
        // so the copy never lerps from the constructor's position (docs/unknowns.md).
        events.push(OwnerEvent::Serialize(owner_state(
            &player,
            &self.look_target_local,
            self.character.as_ref(),
            &self.props,
            self.protection.protected,
        )));
        self.serialize_at = time + SERIALIZE_INTERVAL;
        self.player = Some(player);
        events
    }

    /// One frame of the dummy's client. Returns its messages and whether its `RagdollController`
    /// asked for `SpawnPlayer`.
    pub fn update(
        &mut self,
        world: &CollisionWorld,
        ladders: &[LadderInstance],
        dt: f32,
        time: f32,
    ) -> (Vec<OwnerEvent>, bool) {
        let mut events = Vec::new();
        let respawn = self.ragdoll.as_mut().is_some_and(|r| r.update(time));
        let Some(player) = &mut self.player else {
            return (events, respawn);
        };
        let input = FrameInput::default();
        self.fixed_time += dt;
        while self.fixed_time >= FIXED_DELTA_TIME {
            self.fixed_time -= FIXED_DELTA_TIME;
            player.fixed_update(world, ladders, &input);
        }
        player.update(world, dt, 0.0, &input);
        self.protection.update(dt);
        if let Some(damage) = &mut self.damage {
            let mut camera = player.camera_local_rotation;
            damage.update(dt, &mut camera);
            if damage.run_coroutines(time) == Some(DamageEvent::Destroyed) {
                events.push(OwnerEvent::Destroy);
                self.player = None;
                return (events, respawn);
            }
        }
        player.late_update(&input);
        for event in std::mem::take(&mut player.events) {
            if let PlayerEvent::FallDamage(amount) = event {
                events.push(OwnerEvent::DoDamage(amount));
            }
        }
        let alive = !self.damage.as_ref().is_some_and(|d| d.we_killed);
        if let Some(character) = &mut self.character {
            let input = CharacterInput {
                crouch: player.crouch,
                prone: player.prone,
                grounded: player.grounded,
                walking: player.walking,
                running: player.running,
                ..CharacterInput::default()
            };
            let selected = SelectedWeapon {
                node: self.weapon.as_ref().map(|w| w.node),
                name: self.weapon.as_ref().map_or("", |w| w.name.as_str()),
            };
            character.late_update(&mut self.prefab, &input, selected);
        }
        if alive
            && self.fire
            && let Some(weapon) = &self.weapon
            && time >= self.fire_at
        {
            self.fire_at = time + weapon.interval.max(0.05);
            events.push(OwnerEvent::SyncWeapons(weapon.sync));
        }
        if time >= self.serialize_at {
            self.serialize_at = time + SERIALIZE_INTERVAL;
            events.push(OwnerEvent::Serialize(owner_state(
                player,
                &self.look_target_local,
                self.character.as_ref(),
                &self.props,
                self.protection.protected,
            )));
        }
        (events, respawn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    fn room(mode: &str) -> Room {
        let assets = find_assets_dir().unwrap();
        let scene = crate::scene::load_scene(&assets, "Bridge").unwrap();
        Room::new(&scene, mode)
    }

    #[test]
    fn spawn_points_follow_team_and_mode() {
        let mut dm = room("DM");
        assert_eq!(dm.team_1.spawn_points.len(), 4);
        assert_eq!(dm.team_2.spawn_points.len(), 4);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            let (point, team_1) = dm.spawn_point("German-ZM");
            assert!(team_1);
            seen.insert(point.unwrap());
        }
        assert_eq!(seen.len(), 8, "DM: team 1 spawns anywhere");
        let mut tdm = room("TDM");
        for _ in 0..100 {
            let (point, _) = tdm.spawn_point("German-ZM");
            assert!(tdm.team_1.spawn_points.contains(&point.unwrap()));
            let (point, team_1) = tdm.spawn_point("Mercenary");
            assert!(!team_1 && tdm.team_2.spawn_points.contains(&point.unwrap()));
        }
    }

    #[test]
    fn kill_feed_colours_limit_and_expiry() {
        let mut r = room("TDM");
        let mut killer = PlayerProps {
            actor: 1,
            name: "A".to_string(),
            team: "Mercenary".to_string(),
            kills: 0,
            deaths: 0,
        };
        r.kill_notification(&mut killer, "B");
        assert_eq!(killer.kills, 1);
        assert_eq!(r.team2_score, 1);
        let e = &r.feed[0];
        assert_eq!(
            (e.killer.as_str(), e.middle.as_str(), e.killed.as_str()),
            ("A", "killed", "B")
        );
        assert_eq!(
            (e.killer_color, e.killed_color),
            (r.team_2.color, r.team_1.color)
        );
        for i in 0..6 {
            r.add_message(&format!("P{i}"), "", "connected", "");
        }
        assert_eq!(r.feed.len(), 5);
        assert_eq!(r.feed[0].killer, "P1");
        assert_eq!(r.feed[0].killer_color, Vec4::ONE);
        r.update(5.0);
        // Removing index 0 skips the next entry for this frame.
        assert_eq!(r.feed.len(), 2);
        r.update(0.01);
        assert_eq!(r.feed.len(), 2, "the skipped entries keep running");
        assert!((r.feed[0].timer - 4.99).abs() < 1e-5);
    }
}
