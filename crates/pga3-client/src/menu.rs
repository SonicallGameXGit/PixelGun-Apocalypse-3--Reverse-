//! The menu flow of the CrazyGames build (spec: `docs/gameplay/menu.md`): the `Main` scene's
//! "Choose Server" window (`ServerChose`, uGUI in `ui.rs`), then the `ConnectMenu` lobby (IMGUI
//! with the "lobby swat" skin, laid out by `layout.rs`) with its room list and room creation,
//! then the fade into the room's map (`LoadMap`).
//!
//! The menu owns the Photon connection while it runs; joining a room hands the connected
//! `Client` over to the game (`Online::from_events`).

use std::path::Path;

use anyhow::Result;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use serde_json::Value as Json;

use crate::assets::{self, load_json};
use crate::gui::{Gui, GuiFont, GuiQuad, NamedSkin};
use crate::imgui::{self, Builder, DrawState, Frame, WKind};
use crate::net::client::{Client, ClientEvent, NewRoom};
use crate::net::protocol16::Value;
use crate::online::{Online, ROOM_MARKER};
use crate::prefs::SharedSettings;
use crate::render::View;
use crate::ui::Canvas;

const LOADING_TEXTURE: &str = "textures/loading crazy games.png";
const LOCK_TEXTURE: &str = "textures/Lock.png";
const BACKGROUND_TEXTURE: &str = "textures/1.png";
const LOBBY_SKIN: &str = "sharedassets0.assets:141";
/// The server dialog's `ServerChose` buttons (`ConnectToRegion` codes: `eu` 0, `us` 1, `asia` 2).
const REGIONS: [(&str, &str); 3] = [("eu", "eu"), ("us", "us"), ("asia", "asia")];
/// `LoadMap` waits one second before it starts loading the scene.
const LOAD_DELAY: f32 = 1.0;
/// `FadeScreen`: `fadeValue += deltaTime * 15`.
const FADE_SPEED: f32 = 15.0;
/// How long the lock icon's wrong-password flash lasts: `redtimer` is set to 100 and drops by
/// `deltaTime * 80`.
const RED_TIMER: f32 = 100.0;
const RED_SPEED: f32 = 80.0;
/// The boot screen's minimum duration (the host page's loader in the original).
const BOOT_TIME: f32 = 0.6;
const SCROLL_STEP: f32 = 40.0;

const MAX_PLAYERS: [u8; 8] = [2, 4, 6, 8, 10, 12, 14, 16];
const TIMES: [i32; 4] = [5, 10, 15, 20];
const WEAPONS: [(&str, &str); 5] = [
    ("[WEP02]", "Pistol"),
    ("[WEP03]", "M242"),
    ("[WEP04]", "Shotgun"),
    ("[WEP05]", "Sniper"),
    ("[WEP06]", "Launcher"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The host page's loading screen.
    Boot,
    /// `Main`: the "Choose Server" window.
    Servers,
    /// `ConnectMenu`: the room list.
    Lobby,
    /// `ConnectMenu`: creating a room.
    Create,
    /// `LoadMap`: the fade to the room's map.
    Loading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    PlayerName,
    Search,
    Password,
    RoomName,
    RoomPassword,
}

#[derive(Clone, Debug, PartialEq)]
enum Action {
    Focus(Field),
    ChangeServer,
    CreateRoom,
    Join(usize),
    Lock,
    MainMenu,
    Continue,
    Time(usize),
    Max(usize),
    Mode(&'static str),
    Map(usize),
    Weapon(usize),
}

/// What the player did this frame, in window pixels (y down).
#[derive(Default)]
pub struct MenuInput {
    pub mouse: Vec2,
    /// Left button went down / up this frame.
    pub down: bool,
    pub up: bool,
    pub wheel: f32,
    pub chars: Vec<char>,
    pub backspace: bool,
    pub enter: bool,
    pub tab: bool,
}

/// How the menu ends: the game to start.
pub struct Launch {
    pub online: Option<Online>,
    pub map: String,
    pub mode: String,
    pub name: String,
    pub region: String,
    /// The room's full name.
    pub room_name: String,
    /// `RoundDuration` (s) for an offline room.
    pub round_duration: i32,
}

#[derive(Clone, Debug)]
struct RoomRow {
    name: String,
    map: String,
    players: i32,
    max_players: i32,
    password: String,
}

pub struct Menu {
    pub phase: Phase,
    pub font: GuiFont,
    gui: Gui,
    skin: NamedSkin,
    canvas: Canvas,
    lobby_view: View,
    scene_view: View,
    settings: SharedSettings,
    /// `ConnectMenu`'s statics read from PlayerPrefs in `Start`.
    clan: String,
    kills: i32,
    deaths: i32,
    game_plays: i32,
    client: Option<Client>,
    region: String,
    /// The lobby is shown but the connection is not up yet (`Connecting...`).
    connecting: bool,
    demo_rooms: Vec<RoomRow>,
    maps: Vec<(String, String)>,
    levels: Vec<i64>,
    rank_textures: Vec<String>,
    // Text fields.
    player_name: String,
    search: String,
    password: String,
    room_name: String,
    room_password: String,
    focus: Option<Field>,
    blink: f32,
    red_timer: f32,
    // Create-room settings.
    time_index: usize,
    max_index: usize,
    mode: &'static str,
    map_index: usize,
    weapons: [bool; 5],
    scroll: [f32; 2],
    pressed: Option<Action>,
    // Fade to the map.
    fade: f32,
    load_timer: f32,
    frames_loading: u32,
    joined: Option<(Client, Vec<ClientEvent>)>,
    launch_map: String,
    launch_mode: String,
    launch_room: String,
    boot_timer: f32,
    last_mouse: Vec2,
    pub screenshot: bool,
    /// `--menu-auto`: connects and creates a default room without input (tests).
    pub auto: bool,
    auto_created: bool,
}

fn random(low: u32, high: u32) -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    low + nanos.wrapping_mul(2_654_435_761) % (high - low)
}

/// A camera of the `Main` scene.
fn main_camera(scene: &Json, name: &str, fallback: View) -> View {
    let Some(nodes) = scene["nodes"].as_array() else {
        return fallback;
    };
    let node = nodes.iter().find(|n| n["name"] == name);
    let Some(node) = node else {
        return fallback;
    };
    let mut view = fallback;
    for c in node["components"].as_array().into_iter().flatten() {
        if c["type"] == "Camera" {
            let d = &c["data"];
            view.fov_y_degrees = assets::f32_of(&d["field of view"]);
            view.near = assets::f32_of(&d["near clip plane"]);
            view.far = assets::f32_of(&d["far clip plane"]);
        }
    }
    view
}

impl Menu {
    pub fn new(
        assets: &Path,
        settings: SharedSettings,
        demo_rooms: bool,
        name: &str,
        region: &str,
    ) -> Result<Menu> {
        let scene = load_json(&assets.join("data/scenes/Main.json"))?;
        // Cameras: the GUI camera (`level0:83`) draws the canvas, `__Lobby1/Camera` the lobby.
        let menu_view = View {
            position: Vec3::new(0.0, 12.534, -100.0),
            rotation: Quat::IDENTITY,
            fov_y_degrees: 40.0,
            near: 0.3,
            far: 1000.0,
        };
        // `Main Camera` (draws the background): the GUI camera is 12.534 higher.
        let scene_view = View {
            position: Vec3::new(0.0, 0.0, -100.0),
            ..menu_view
        };
        let lobby_view = View {
            position: Vec3::new(0.0, 113.0, -247.0),
            rotation: Quat::from_xyzw(0.1736, 0.0, 0.0, 0.9848).normalize(),
            fov_y_degrees: 21.0,
            near: 0.3,
            far: 600.0,
        };
        let lobby_view = main_camera(&scene, "Camera", lobby_view);
        let canvas = Canvas::load(
            assets,
            View {
                position: menu_view.position,
                rotation: menu_view.rotation,
                fov_y_degrees: menu_view.fov_y_degrees,
                near: menu_view.near,
                far: menu_view.far,
            },
        )?;
        let skin = crate::gui::load_named_skin(assets, LOBBY_SKIN)?;
        let mut maps = Vec::new();
        let mut levels = Vec::new();
        let mut rank_textures = Vec::new();
        for node in scene["nodes"].as_array().into_iter().flatten() {
            for c in node["components"].as_array().into_iter().flatten() {
                if c["script"] != "ConnectMenu" {
                    continue;
                }
                let f = &c["fields"];
                for m in f["allMaps"].as_array().into_iter().flatten() {
                    maps.push((
                        m["mapName"].as_str().unwrap_or("").to_string(),
                        m["mapPreview"]["asset"].as_str().unwrap_or("").to_string(),
                    ));
                }
                for l in f["Level"].as_array().into_iter().flatten() {
                    levels.push(l["Level_Kills_Required"].as_i64().unwrap_or(0));
                }
                for r in f["RankTextures"].as_array().into_iter().flatten() {
                    rank_textures.push(r["asset"].as_str().unwrap_or("").to_string());
                }
            }
        }
        let mut player_name = format!("Player {}", random(111, 9999));
        let saved = {
            let s = settings.borrow();
            s.prefs
                .has_key("PlayerName")
                .then(|| s.prefs.get_string("PlayerName", ""))
        };
        if let Some(saved) = saved {
            player_name = saved;
        }
        let player_name = if name == "Player" {
            player_name
        } else {
            name.to_string()
        };
        let demo = if demo_rooms {
            vec![
                RoomRow {
                    name: format!("Room 4721: TDM[WEP02][WEP03][WEP04][WEP05][WEP06]{ROOM_MARKER}"),
                    map: "Bridge".to_string(),
                    players: 7,
                    max_players: 16,
                    password: String::new(),
                },
                RoomRow {
                    name: format!("Fast frags: DM[WEP02][WEP04]{ROOM_MARKER}"),
                    map: "Snow".to_string(),
                    players: 3,
                    max_players: 8,
                    password: String::new(),
                },
                RoomRow {
                    name: format!(
                        "Friends only: TDM[WEP02][WEP03][WEP04][WEP05][WEP06]{ROOM_MARKER}"
                    ),
                    map: "Castle".to_string(),
                    players: 2,
                    max_players: 4,
                    password: "secret".to_string(),
                },
            ]
        } else {
            Vec::new()
        };
        let mut menu = Menu {
            phase: Phase::Boot,
            font: GuiFont::load(assets)?,
            gui: Gui::new(assets),
            skin,
            canvas,
            lobby_view,
            scene_view,
            settings,
            clan: String::new(),
            kills: 0,
            deaths: 0,
            game_plays: 0,
            client: None,
            region: region.to_string(),
            connecting: false,
            demo_rooms: demo,
            maps,
            levels,
            rank_textures,
            player_name,
            search: String::new(),
            password: String::new(),
            room_name: format!("Room {}", random(111, 9999)),
            room_password: String::new(),
            focus: None,
            blink: 0.0,
            red_timer: 0.0,
            time_index: 3,
            max_index: 7,
            mode: "TDM",
            map_index: 0,
            weapons: [true; 5],
            scroll: [0.0; 2],
            pressed: None,
            fade: 0.0,
            load_timer: 0.0,
            frames_loading: 0,
            joined: None,
            launch_map: String::new(),
            launch_mode: String::new(),
            launch_room: String::new(),
            boot_timer: 0.0,
            last_mouse: Vec2::ZERO,
            screenshot: false,
            auto: false,
            auto_created: false,
        };
        menu.start();
        Ok(menu)
    }

    /// `ConnectMenu.Start` (each time `Main` loads): `GamePlays + 1`; the clan, kills and deaths
    /// of PlayerPrefs.
    fn start(&mut self) {
        let mut settings = self.settings.borrow_mut();
        let prefs = &mut settings.prefs;
        prefs.increment("GamePlays");
        self.game_plays = prefs.get_int("GamePlays", 0);
        self.clan = prefs.get_string("Clan", "");
        self.kills = prefs.get_int("Kills", 0);
        self.deaths = prefs.get_int("Deaths", 0);
    }

    /// Menu screens for `--menu` (screenshots): jumps to `screen` without a connection.
    pub fn show(&mut self, screen: &str) -> bool {
        match screen {
            "boot" => self.phase = Phase::Boot,
            "main" | "servers" => self.phase = Phase::Servers,
            "connecting" => {
                self.phase = Phase::Lobby;
                self.connecting = true;
            }
            "lobby" => self.phase = Phase::Lobby,
            "create" => self.phase = Phase::Create,
            "loading" => {
                self.phase = Phase::Loading;
                self.fade = 1.0;
            }
            _ => return false,
        }
        self.boot_timer = -1.0e9;
        true
    }

    /// Back from a game: the connection is re-established and the lobby shown (the original
    /// stays connected to the master and loads `Main`, whose `ServerChose` skips the dialog).
    /// The menu scene after `OnDisconnectedFromPhoton`: not connected, so `ServerChose` shows the
    /// server dialog again (`ServerChose.OnDisconnectedFromPhoton` also clears the region).
    pub fn return_disconnected(&mut self) {
        self.client = None;
        self.phase = Phase::Servers;
        self.connecting = false;
        self.fade = 0.0;
        self.joined = None;
        self.boot_timer = BOOT_TIME;
        self.pressed = None;
        self.focus = None;
        self.start();
    }

    pub fn return_from_game(&mut self, region: &str) {
        self.region = region.to_string();
        self.client = Some(Client::connect(
            crate::net::pun::CRAZYGAMES,
            region,
            &self.player_name,
        ));
        self.phase = Phase::Lobby;
        self.connecting = true;
        self.fade = 0.0;
        self.joined = None;
        self.boot_timer = BOOT_TIME;
        self.pressed = None;
        self.focus = None;
        self.start();
    }

    fn rooms(&self) -> Vec<RoomRow> {
        if self.client.is_none() {
            return self.demo_rooms.clone();
        }
        let Some(client) = &self.client else {
            return Vec::new();
        };
        client
            .rooms
            .iter()
            .map(|r| RoomRow {
                name: r.name.clone(),
                map: r.prop_str("MapName").unwrap_or("").to_string(),
                players: r.players,
                max_players: r.max_players,
                password: r.prop_str("Password").unwrap_or("").to_string(),
            })
            .collect()
    }

    /// The rooms `ConnectMenu.ShowConnectMenu` lists: this game's, matching the search.
    fn listed_rooms(&self) -> Vec<RoomRow> {
        let search = self.search.to_lowercase();
        self.rooms()
            .into_iter()
            .filter(|r| r.name.contains(ROOM_MARKER))
            .filter(|r| r.name.to_lowercase().contains(&search))
            .collect()
    }

    fn weapons_allowed(&self) -> String {
        WEAPONS
            .iter()
            .zip(self.weapons)
            .filter(|(_, on)| *on)
            .map(|((tag, _), _)| *tag)
            .collect()
    }

    fn level(&self) -> usize {
        let mut level = 1;
        for (i, required) in self.levels.iter().enumerate() {
            if i64::from(self.kills) >= *required {
                level = i + 1;
            }
        }
        level
    }

    fn strip_spaces(text: &str) -> String {
        text.replace(' ', "")
    }

    /// `CheckPlayerNameAndRoom`.
    fn check_names(&mut self) {
        self.player_name = Menu::strip_spaces(&self.player_name);
        if self.player_name.is_empty() {
            self.player_name = format!("Player{}", random(111, 999));
        }
        self.clan = Menu::strip_spaces(&self.clan);
        self.room_name = Menu::strip_spaces(&self.room_name);
        if self.room_name.is_empty() {
            self.room_name = format!("RoomName{}", random(111, 999));
        }
        let mut settings = self.settings.borrow_mut();
        settings.prefs.set_string("PlayerName", &self.player_name);
        settings.prefs.set_string("Clan", &self.clan);
    }

    fn connect(&mut self, region: &str) {
        self.region = region.to_string();
        self.client = Some(Client::connect(
            crate::net::pun::CRAZYGAMES,
            region,
            &self.player_name,
        ));
        self.connecting = true;
    }

    fn open_url(url: &str) {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }

    fn start_loading(&mut self, client: Client, events: Vec<ClientEvent>, map: &str, mode: &str) {
        self.joined = Some((client, events));
        self.launch_map = map.to_string();
        self.launch_mode = mode.to_string();
        self.phase = Phase::Loading;
        self.fade = 0.0;
        self.load_timer = 0.0;
        self.frames_loading = 0;
    }

    /// One frame of logic. Returns the game to start once `LoadMap`'s delay is over.
    pub fn update(&mut self, dt: f32, input: &MenuInput, size: (f32, f32)) -> Option<Launch> {
        self.blink += dt;
        self.last_mouse = input.mouse;
        if self.red_timer > 0.0 {
            self.red_timer -= dt * RED_SPEED;
        }
        self.poll_network();
        if self.auto {
            self.run_auto();
        }
        match self.phase {
            Phase::Boot => {
                self.boot_timer += dt;
                if self.boot_timer >= BOOT_TIME {
                    self.phase = if self.client.is_some() {
                        Phase::Lobby
                    } else {
                        Phase::Servers
                    };
                }
            }
            Phase::Servers => self.update_servers(dt, input, size),
            Phase::Lobby | Phase::Create => self.update_lobby(input, size),
            Phase::Loading => {
                self.fade = (self.fade + dt * FADE_SPEED).min(1.0);
                self.load_timer += dt;
                self.frames_loading += 1;
                // The fade is drawn at least twice before the (blocking) scene load starts.
                if self.load_timer >= LOAD_DELAY && self.frames_loading > 2 {
                    return self.finish();
                }
            }
        }
        None
    }

    /// `--menu-auto`: the clicks of a player who picks Europe and creates a room.
    fn run_auto(&mut self) {
        match self.phase {
            // With `--demo-rooms` the room is created offline.
            Phase::Servers if self.client.is_none() && !self.demo_rooms.is_empty() => {
                self.phase = Phase::Lobby;
            }
            Phase::Servers if self.client.is_none() => {
                let region = self.region.clone();
                self.connect(&region);
            }
            Phase::Lobby if !self.connecting && !self.auto_created => {
                self.auto_created = true;
                self.perform(Action::CreateRoom);
                self.perform(Action::Continue);
            }
            _ => {}
        }
    }

    fn finish(&mut self) -> Option<Launch> {
        let (client, events) = self.joined.take()?;
        let online = Online::from_events(client, events);
        Some(Launch {
            online: Some(online),
            map: self.launch_map.clone(),
            mode: self.launch_mode.clone(),
            name: self.player_name.clone(),
            region: self.region.clone(),
            room_name: self.launch_room.clone(),
            round_duration: TIMES[self.time_index] * 60,
        })
    }

    /// `OnConnectedToPhoton` / `OnJoinedRoom` / `OnDisconnectedFromPhoton`.
    fn poll_network(&mut self) {
        let Some(client) = &mut self.client else {
            return;
        };
        let events = client.poll();
        let mut joined = None;
        let mut failed = false;
        let mut kept = Vec::new();
        for event in events {
            match &event {
                ClientEvent::InLobby => {
                    self.connecting = false;
                    if self.phase == Phase::Servers {
                        // ServerChose.Update: connected -> hide the dialog, show the lobby.
                        self.phase = Phase::Lobby;
                    }
                }
                ClientEvent::Joined { .. } => joined = Some(event),
                ClientEvent::Failed(reason) => {
                    eprintln!("menu: {reason}");
                    failed = true;
                }
                _ => kept.push(event),
            }
        }
        if let Some(event) = joined
            && let Some(client) = self.client.take()
        {
            let (map, mode) = match &event {
                ClientEvent::Joined { room_props, .. } => (
                    room_props
                        .get_str("MapName")
                        .and_then(Value::as_str)
                        .unwrap_or("Bridge")
                        .to_string(),
                    room_props
                        .get_str("GameMode")
                        .and_then(Value::as_str)
                        .unwrap_or("TDM")
                        .to_string(),
                ),
                _ => unreachable!(),
            };
            kept.insert(0, event);
            self.start_loading(client, kept, &map, &mode);
            return;
        }
        if failed {
            // OnDisconnectedFromPhoton: back to the server dialog.
            self.client = None;
            self.connecting = false;
            if self.phase != Phase::Loading {
                self.phase = Phase::Servers;
            }
        }
    }

    fn update_servers(&mut self, dt: f32, input: &MenuInput, size: (f32, f32)) {
        let aspect = size.0 / size.1;
        self.canvas.layout(aspect, size, input.mouse, dt);
        let hovered = self.canvas.button_at(input.mouse, size);
        self.canvas.animate(hovered, dt);
        if !input.down {
            return;
        }
        let Some(node) = hovered else {
            return;
        };
        let calls = {
            let n = &self.canvas.nodes[node];
            match &n.button {
                Some(b) => b
                    .on_click
                    .iter()
                    .chain(b.on_press.iter())
                    .cloned()
                    .collect(),
                None => Vec::<crate::ui::Call>::new(),
            }
        };
        for call in calls {
            if let Some((_, region)) = REGIONS.iter().find(|(m, _)| *m == call.method) {
                self.connect(region);
            } else if let Some(url) = link_url(&call.method) {
                Menu::open_url(url);
            }
        }
    }

    // ---------------------------------------------------------------------------------------
    // The lobby (IMGUI)
    // ---------------------------------------------------------------------------------------

    fn area(size: (f32, f32)) -> [f32; 4] {
        [size.0 / 2.0 - 370.0, size.1 / 2.0 - 237.0, 750.0, 475.0]
    }

    fn build_lobby(&mut self, size: (f32, f32), bar: f32) -> Frame<Action> {
        let window = self.skin.style("window");
        let rows = self.listed_rooms();
        let level = self.level();
        let area = Menu::area(size);
        let mut b = Builder::new(
            &mut self.gui,
            &mut self.font,
            &self.skin,
            area,
            window.padding,
        );
        match self.phase {
            Phase::Create => create_widgets(
                &mut b,
                &self.room_name,
                &self.room_password,
                self.time_index,
                self.max_index,
                self.mode,
                self.map_index,
                &self.weapons,
                &self.maps,
                bar,
            ),
            _ => list_widgets(
                &mut b,
                &self.player_name,
                &self.search,
                &self.password,
                self.red_timer > 0.0,
                &rows,
                (self.kills, self.deaths, self.game_plays),
                level,
                &self.rank_textures,
                bar,
            ),
        }
        b.finish()
    }

    /// Lays the lobby out (with a scroll bar when a scroll view overflows) and handles the
    /// mouse and keyboard.
    fn lobby_frame(&mut self, size: (f32, f32)) -> Frame<Action> {
        let frame = self.build_lobby(size, 0.0);
        if frame.any_overflow() {
            let bar = imgui::scrollbar_width(&self.skin);
            return self.build_lobby(size, bar);
        }
        frame
    }

    fn update_lobby(&mut self, input: &MenuInput, size: (f32, f32)) {
        let frame = self.lobby_frame(size);
        let enabled = !self.connecting;
        let mouse = input.mouse;
        frame.scroll(&mut self.scroll, mouse, input.wheel, SCROLL_STEP);
        // Typing.
        if let Some(field) = self.focus {
            let max = frame.field_max(&Action::Focus(field)).unwrap_or(17);
            let text = self.field_mut(field);
            for c in &input.chars {
                if !c.is_control() && text.chars().count() < max {
                    text.push(*c);
                }
            }
            if input.backspace {
                text.pop();
            }
            if input.enter || input.tab {
                self.focus = None;
            }
        }
        if !enabled {
            return;
        }
        let hit = frame.hit(&self.scroll, mouse).cloned();
        if input.down {
            if !matches!(hit, Some(Action::Focus(_))) {
                self.focus = None;
            }
            self.pressed = hit.clone();
        }
        if input.up {
            let pressed = self.pressed.take();
            if let Some(action) = pressed
                && Some(&action) == hit.as_ref()
            {
                self.perform(action);
            }
        }
    }

    fn field_mut(&mut self, field: Field) -> &mut String {
        match field {
            Field::PlayerName => &mut self.player_name,
            Field::Search => &mut self.search,
            Field::Password => &mut self.password,
            Field::RoomName => &mut self.room_name,
            Field::RoomPassword => &mut self.room_password,
        }
    }

    fn perform(&mut self, action: Action) {
        match action {
            Action::Focus(field) => {
                self.focus = Some(field);
                self.blink = 0.0;
            }
            Action::ChangeServer => {
                // chooseserverondisconect: PhotonNetwork.Disconnect, then the dialog returns.
                self.client = None;
                self.connecting = false;
                self.phase = Phase::Servers;
            }
            Action::CreateRoom => {
                self.check_names();
                self.phase = Phase::Create;
                self.scroll = [0.0; 2];
            }
            Action::MainMenu => {
                self.phase = Phase::Lobby;
                self.scroll = [0.0; 2];
            }
            Action::Lock => self.red_timer = RED_TIMER,
            Action::Join(i) => {
                let rows = self.listed_rooms();
                let Some(row) = rows.get(i).cloned() else {
                    return;
                };
                self.check_names();
                self.launch_room = row.name.clone();
                if let Some(client) = &mut self.client {
                    client.player_name = self.player_name.clone();
                    if let Err(error) = client.join_room(&row.name) {
                        eprintln!("menu: join: {error:#}");
                    }
                } else {
                    // Offline demo: no connection to hand over.
                    self.launch_map = row.map.clone();
                    self.phase = Phase::Loading;
                    self.fade = 0.0;
                    self.load_timer = 0.0;
                    self.frames_loading = 0;
                    self.joined = None;
                }
            }
            Action::Continue => {
                self.check_names();
                let name = format!(
                    "{}: {}{}{}",
                    self.room_name,
                    self.mode,
                    self.weapons_allowed(),
                    ROOM_MARKER
                );
                self.launch_room = name.clone();
                let map = self
                    .maps
                    .get(self.map_index)
                    .map_or("Bridge".to_string(), |m| m.0.clone());
                let props = vec![
                    ("MapName".to_string(), Value::str(&map)),
                    (
                        "RoundDuration".to_string(),
                        Value::Int(TIMES[self.time_index] * 60),
                    ),
                    ("GameMode".to_string(), Value::str(self.mode)),
                    ("Password".to_string(), Value::str(&self.room_password)),
                ];
                let room = NewRoom {
                    name,
                    max_players: MAX_PLAYERS[self.max_index],
                    props,
                    lobby_props: ["MapName", "RoundDuration", "GameMode", "Password"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                };
                if let Some(client) = &mut self.client {
                    client.player_name = self.player_name.clone();
                    if let Err(error) = client.create_room(room) {
                        eprintln!("menu: create: {error:#}");
                    }
                } else {
                    self.launch_map = map;
                    self.phase = Phase::Loading;
                    self.fade = 0.0;
                    self.load_timer = 0.0;
                    self.frames_loading = 0;
                    self.joined = None;
                }
            }
            Action::Time(i) => self.time_index = i,
            Action::Max(i) => self.max_index = i,
            Action::Mode(mode) => self.mode = mode,
            Action::Map(i) => self.map_index = i,
            Action::Weapon(i) => self.weapons[i] = !self.weapons[i],
        }
    }

    /// An offline launch (no server): the demo rooms and `--menu` runs.
    pub fn offline_launch(&mut self) -> Option<Launch> {
        if self.phase != Phase::Loading || self.joined.is_some() || self.client.is_some() {
            return None;
        }
        if self.load_timer >= LOAD_DELAY && self.frames_loading > 2 && !self.launch_map.is_empty() {
            return Some(Launch {
                online: None,
                map: self.launch_map.clone(),
                mode: self.mode.to_string(),
                name: self.player_name.clone(),
                region: self.region.clone(),
                room_name: self.launch_room.clone(),
                round_duration: TIMES[self.time_index] * 60,
            });
        }
        None
    }

    // ---------------------------------------------------------------------------------------
    // Drawing
    // ---------------------------------------------------------------------------------------

    /// The clear colour and quads of this frame.
    pub fn frame(&mut self, size: (f32, f32)) -> ([f32; 3], Vec<GuiQuad>) {
        self.gui.quads.clear();
        self.gui.color = Vec4::ONE;
        match self.phase {
            Phase::Boot => {
                self.fullscreen(size, 1.0);
                ([0.0; 3], std::mem::take(&mut self.gui.quads))
            }
            Phase::Servers => {
                let mut quads = Vec::new();
                let aspect = size.0 / size.1;
                quads.push(background_quad(&self.scene_view, aspect));
                quads.extend(self.canvas.quads(&mut self.font, size));
                ([0.0616, 0.0949, 0.147], quads)
            }
            Phase::Lobby | Phase::Create => {
                let aspect = size.0 / size.1;
                let frame = self.lobby_frame(size);
                let mut quads = vec![background_quad(&self.lobby_view, aspect)];
                self.draw_lobby(&frame, size);
                quads.append(&mut self.gui.quads);
                ([0.0; 3], quads)
            }
            Phase::Loading => {
                let aspect = size.0 / size.1;
                let mut quads = Vec::new();
                if self.client.is_none() && self.joined.is_none() {
                    // Offline demo: the lobby still shows under the fade.
                    quads.push(background_quad(&self.lobby_view, aspect));
                } else {
                    quads.push(background_quad(&self.lobby_view, aspect));
                }
                self.fullscreen(size, self.fade);
                quads.append(&mut self.gui.quads);
                ([0.0; 3], quads)
            }
        }
    }

    /// `FadeScreen`: the loading picture over the whole window with `alpha`.
    fn fullscreen(&mut self, size: (f32, f32), alpha: f32) {
        self.gui.color = Vec4::new(1.0, 1.0, 1.0, alpha);
        self.gui
            .draw_texture([0.0, 0.0, size.0, size.1], LOADING_TEXTURE);
        self.gui.color = Vec4::ONE;
    }

    fn draw_lobby(&mut self, frame: &Frame<Action>, size: (f32, f32)) {
        let enabled = !self.connecting;
        let alpha = if enabled { 1.0 } else { 0.5 };
        let window = self.skin.style("window");
        let rooms = self.listed_rooms();
        let players: i32 = rooms.iter().map(|r| r.players).sum();
        let title = format!(
            "Rooms: {}                              Players: {}",
            rooms.len(),
            players
        );
        self.gui.color = Vec4::new(1.0, 1.0, 1.0, alpha);
        let area = Menu::area(size);
        imgui::window_frame(&mut self.gui, &mut self.font, &window, area, &title, false);
        let focus = self.focus.map(Action::Focus);
        let state = DrawState {
            mouse: self.last_mouse,
            pressed: self.pressed.as_ref(),
            focus: focus.as_ref(),
            blink: self.blink,
            offsets: &self.scroll,
            alpha,
            interactive: enabled,
        };
        imgui::draw(&mut self.gui, &mut self.font, &self.skin, frame, &state);
        self.gui.color = Vec4::ONE;
        if self.connecting {
            // `GUI.enabled` stays false until connected: the "Connecting..." box on top.
            let rect = [size.0 / 2.0 - 75.0, size.1 / 2.0 - 15.0, 150.0, 30.0];
            let style = self.skin.style("box");
            self.gui.box_(&mut self.font, rect, "Connecting...", &style);
        }
    }
}

fn link_url(method: &str) -> Option<&'static str> {
    match method {
        "OpenCrazy" => Some("https://www.crazygames.com/t/mentolatux"),
        "OpenFacebook" => Some("https://www.facebook.com/MultiplayerAndroidGames/"),
        "OpenIoanMita2016" => Some("https://play.google.com/store/apps/dev?id=5903514895660400064"),
        "OpenYoutube" => Some("https://www.youtube.com/channel/UCNxM6ECVEYNLXJepiyWqA6A"),
        "OpenPaco" => Some("http://www.pacogames.com/links/pixel-gun-apocalypse-3.php?more"),
        _ => None,
    }
}

/// The `webgl` background cube's front face as seen by `view`: `textures/1.png` on a plane of
/// the cube's size at its front (documented as inferred in `docs/gameplay/menu.md`).
fn background_quad(view: &View, aspect: f32) -> GuiQuad {
    let (v, p) = view.matrices(aspect);
    // Cube: position (-2.8, 2, 83.3), size 277.59 x 141.66 x 7.07; front face at z - 3.535.
    let model = Mat4::from_translation(Vec3::new(-2.8, 2.0, 83.3 - 3.535))
        * Mat4::from_scale(Vec3::new(277.59, -141.66, 1.0));
    let tint = 1.557;
    GuiQuad {
        texture: BACKGROUND_TEXTURE.to_string(),
        rect: [-0.5, -0.5, 1.0, 1.0],
        uv: [0.0, 0.0, 1.0, 1.0],
        color: Vec4::new(tint, tint, tint, 1.0),
        mvp: Some(p * v * model),
    }
}

// -------------------------------------------------------------------------------------------
// Widget lists (one function per `ShowConnectMenu` mode)
// -------------------------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn list_widgets(
    b: &mut Builder<Action>,
    name: &str,
    search: &str,
    password: &str,
    red: bool,
    rooms: &[RoomRow],
    (kills, deaths, game_plays): (i32, i32, i32),
    level: usize,
    ranks: &[String],
    bar: f32,
) {
    b.space(10.0);
    b.horizontal();
    b.label("Player Name: ", None, None);
    b.field(
        Action::Focus(Field::PlayerName),
        name,
        15,
        Some(390.0),
        22.0,
    );
    b.end();
    b.horizontal();
    b.label("Search room name: ", None, None);
    b.field(Action::Focus(Field::Search), search, 17, Some(360.0), 22.0);
    b.end();
    b.horizontal();
    if red {
        b.color = Vec4::new(1.0, 0.0, 0.0, 1.0);
    }
    b.label("Password: ", Some(80.0), None);
    b.field(Action::Focus(Field::Password), password, 17, None, 22.0);
    b.color = Vec4::ONE;
    b.end();
    b.begin_scroll((660.0, 225.0), 660.0 - bar);
    for (i, room) in rooms.iter().enumerate() {
        b.horizontal_box();
        b.label(&strip_room_name(&room.name), Some(170.0), None);
        b.label(&room.map, Some(90.0), None);
        b.label(
            &format!("{}/{}", room.players, room.max_players),
            Some(40.0),
            None,
        );
        b.flexible_space();
        if !room.password.is_empty() && room.password != password {
            b.leaf(
                WKind::Button(Action::Lock),
                "",
                "button",
                Some(30.0),
                Some(30.0),
            );
            b.overlay_texture(LOCK_TEXTURE);
        }
        b.button("Join Room", Action::Join(i), Some(180.0), None);
        b.end();
    }
    b.end_scroll();
    b.space(5.0);
    b.horizontal();
    b.boxed(&format!("Level: {level}"), None, Some(32.0));
    let rank = ranks
        .get(level.saturating_sub(1))
        .cloned()
        .unwrap_or_default();
    b.leaf(WKind::Texture(rank), "", "box", Some(62.0), Some(62.0));
    b.boxed(&format!("Kills: {}", kills), None, Some(32.0));
    b.boxed(&format!("Deaths: {}", deaths), None, Some(32.0));
    b.boxed(&format!("Game plays: {game_plays}"), None, Some(32.0));
    b.end();
    b.flexible_space();
    b.horizontal();
    b.flexible_space();
    b.button(
        "Change Server",
        Action::ChangeServer,
        Some(230.0),
        Some(55.0),
    );
    b.button("Create Room", Action::CreateRoom, Some(230.0), Some(55.0));
    b.end();
}

#[allow(clippy::too_many_arguments)]
fn create_widgets(
    b: &mut Builder<Action>,
    room_name: &str,
    room_password: &str,
    time: usize,
    max: usize,
    mode: &str,
    map: usize,
    weapons: &[bool; 5],
    maps: &[(String, String)],
    bar: f32,
) {
    let green = Vec4::new(0.0, 1.0, 0.0, 1.0);
    b.horizontal();
    b.label("Room Name: ", Some(130.0), None);
    b.field(Action::Focus(Field::RoomName), room_name, 17, None, 22.0);
    b.end();
    b.space(5.0);
    b.horizontal();
    b.label("Password: ", Some(130.0), None);
    b.field(
        Action::Focus(Field::RoomPassword),
        room_password,
        17,
        None,
        22.0,
    );
    b.end();
    b.space(5.0);
    b.horizontal();
    b.label("Round Time: ", Some(130.0), None);
    for (i, minutes) in TIMES.iter().enumerate() {
        b.color = if i == time { green } else { Vec4::ONE };
        b.button(
            &minutes.to_string(),
            Action::Time(i),
            Some(55.0),
            Some(25.0),
        );
    }
    b.color = Vec4::ONE;
    b.end();
    b.space(5.0);
    b.horizontal();
    b.label("Max Players: ", Some(130.0), None);
    for (i, players) in MAX_PLAYERS.iter().enumerate() {
        b.color = if i == max { green } else { Vec4::ONE };
        b.button(&players.to_string(), Action::Max(i), Some(55.0), Some(25.0));
    }
    b.color = Vec4::ONE;
    b.end();
    b.space(5.0);
    b.horizontal();
    b.label("Game Mode: ", Some(130.0), None);
    b.color = if mode == "TDM" { green } else { Vec4::ONE };
    b.button(
        "TeamDeathmatch",
        Action::Mode("TDM"),
        Some(300.0),
        Some(30.0),
    );
    b.color = if mode == "DM" { green } else { Vec4::ONE };
    b.button("Deathmatch", Action::Mode("DM"), Some(200.0), Some(30.0));
    b.color = Vec4::ONE;
    b.end();
    b.space(5.0);
    b.horizontal();
    b.begin_scroll((320.0, 200.0), 320.0 - bar);
    for (i, (name, _)) in maps.iter().enumerate() {
        b.color = if i == map { green } else { Vec4::ONE };
        b.button(name, Action::Map(i), None, Some(50.0));
    }
    b.color = Vec4::ONE;
    b.end_scroll();
    b.space(5.0);
    b.vertical();
    for (i, (_, label)) in WEAPONS.iter().enumerate() {
        b.color = if weapons[i] {
            green
        } else {
            Vec4::new(1.0, 0.0, 0.0, 1.0)
        };
        b.button(label, Action::Weapon(i), Some(150.0), Some(28.0));
    }
    b.color = Vec4::ONE;
    b.end();
    let preview = maps.get(map).map(|m| m.1.clone()).unwrap_or_default();
    b.leaf(
        WKind::Texture(preview),
        "",
        "label",
        Some(230.0),
        Some(160.0),
    );
    b.end();
    b.flexible_space();
    b.horizontal();
    b.button("Main Menu", Action::MainMenu, Some(370.0), Some(40.0));
    b.flexible_space();
    b.button("Continue", Action::Continue, Some(370.0), Some(40.0));
    b.end();
}

/// The room name as listed: `[WEP0n]` tags removed and the trailing marker cut (16 chars).
fn strip_room_name(name: &str) -> String {
    let mut text = name.to_string();
    for n in 1..=9 {
        text = text.replace(&format!("[WEP0{n}]"), "");
    }
    let keep = text.chars().count().saturating_sub(16);
    text.chars().take(keep).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_names_lose_tags_and_marker() {
        let name = format!("Room 12: TDM[WEP02][WEP03]{ROOM_MARKER}");
        assert_eq!(strip_room_name(&name), "Room 12: TDM");
    }
}
