//! Pixel Gun Apocalypse 3 open-source client.
//!
//! Current milestone: everything of a match that runs on one client, before the network. Loads a
//! map from `assets/data/scenes/`, renders it with the original shading and spawns the local
//! player (original movement and collision) at its team's spawn point with its first-person
//! weapons (switching, aiming, firing, reloading, bullets and rockets), their sounds, impact /
//! explosion particles and trails, the HUD (ammo, weapon bar, HP, kill feed) and fall damage with
//! the death prefab and respawn. Other players are copies driven by what the network would carry
//! (`remote.rs`); without the network, `--dummy` adds a simulated second client whose player can
//! be shot and killed. A free-fly camera shows the player's own body as the others see it.
//!
//! Usage: `pga3-client [Scene] [--spawn N | --camera x,y,z,yaw,pitch [--walk]] [--fly] [--stats]
//! [--weapon KEY] [--fire] [--aim] [--screenshot out.png [--wait SECONDS]]
//! [--mode DM|TDM] [--team 1|2] [--name NAME] [--volume 0..1] [--quality 0..5] [--pause]
//! [--dummy [--dummy-team 1|2] [--dummy-spawn N] [--dummy-weapon SLOT] [--dummy-fire]]
//! [--online [--region eu] [--room TEXT | --create]] [--list-rooms [--region eu]]`
//! (`--online` joins the most populated room of the live CrazyGames build on the original Photon
//! servers, or `--room`'s, or creates one on the given map; the room's map is loaded.)
//! (`--weapon` presses that number key on the first frame, `--fire` holds the fire button and
//! clicks every 30 frames: for screenshots of the weapons. `--team 1` is "German-ZM", which plays
//! the `mercenarys` prefab, `--team 2` (default) "Mercenary" with `german`.)
//! Controls: click = capture mouse, Esc = release / quit, F = walk/fly toggle, Tab = next spawn.
//! Walking (original keys): WASD/arrows, Space jump, Left Shift run, C crouch, Left Ctrl prone.
//! Weapons (original keys): left mouse fire, right mouse aim, R reload, 1-6 / mouse wheel /
//! M, N select.
//! Flying: WASD, Space/C up/down, Shift fast.

mod anim;
mod assets;
mod audio;
mod character;
mod damage;
mod effects;
mod footsteps;
mod gfx;
mod gui;
mod hud;
mod imgui;
mod layout;
mod legacy_particles;
mod menu;
mod net;
mod obj;
mod online;
mod particles;
mod physics;
mod player;
mod prefab;
mod prefs;
mod primitives;
mod projectiles;
mod remote;
mod render;
mod room;
mod room_menu;
mod scene;
mod skin;
mod trail;
mod ui;
mod unity;
mod weapons;

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use assets::ref_of;
use glam::{EulerRot, Quat, Vec2, Vec3};
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext, PossiblyCurrentContext,
    Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde_json::Value;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

use anim::{ClipLibrary, Trs};
use audio::{Audio, SourceSettings};
use character::{CharacterAnimation, CharacterInput, SelectedWeapon};
use damage::{DamageEvent, PlayerDamage, RagdollController, SpawnProtection};
use effects::{EffectEvent, Effects, MeshCache, explosion_damage};
use footsteps::FpsSoundController;
use gui::{Gui, GuiFont, GuiSkin};
use hud::{AmmoDisplay, DefaultSkin};
use online::{NetIn, Online, OnlineOptions};
use physics::CollisionWorld;
use player::{FIXED_DELTA_TIME, FrameInput, KeyAxis, MAXIMUM_DELTA_TIME, Player};
use prefab::PrefabInstance;
use prefs::{PlayerPrefs, Settings, SharedSettings};
use projectiles::{ProjectileEvent, Projectiles, SpawnFlags, Targets, Templates};
use remote::{ActorId, LocalView, RemoteDeath, RemotePlayer, SyncEvent};
use render::{PrefabModels, SceneRenderer, View};
use room::{DummyOwner, DummyWeapon, OwnerEvent, PlayerProps, Room, SERIALIZE_INTERVAL};
use room_menu::{PlayerRow, RoomAction, RoomMenu, RoomView, SPECTATORS};
use scene::Scene;
use weapons::{SoundSource, SyncFire, WeaponContext, WeaponEvent, WeaponInput, WeaponManager};

/// The player prefabs `RoomMultiplayerMenu.SpawnPlayer` instantiates: `playerPrefab`
/// ("Mercenarys", for team 1) and `playerPrefabTwo` ("German", any other team).
const PLAYER_KINDS: [&str; 2] = ["mercenarys", "german"];

/// This client's Photon actor number (the dummies take the next ones).
const LOCAL_ACTOR: ActorId = 1;

/// `Input.GetAxis("Mouse X"/"Mouse Y")` sensitivity in this build's InputManager.
const MOUSE_AXIS_SENSITIVITY: f32 = 0.1;

struct Args {
    scene: String,
    spawn: Option<usize>,
    camera: Option<[f32; 5]>,
    fly: bool,
    /// With `--camera`: the player stands there (as after the F toggle) instead of flying.
    walk: bool,
    screenshot: Option<PathBuf>,
    /// Seconds to run before taking the screenshot (animations in progress).
    wait: f32,
    /// Print frame-time statistics every second.
    stats: bool,
    /// Number key pressed on the first frame.
    weapon: Option<usize>,
    /// Hold the fire button (and click every 30 frames).
    fire: bool,
    /// Press the aim button once, 0.6 s after start (after the weapon switch).
    aim: bool,
    /// Room property `GameMode`.
    mode: String,
    /// 1 or 2: `team_1` or `team_2`.
    team: u8,
    name: String,
    /// A simulated second client.
    dummy: bool,
    dummy_team: u8,
    dummy_spawn: Option<usize>,
    dummy_weapon: usize,
    dummy_fire: bool,
    /// Play on the original servers (the CrazyGames build's Photon application).
    online: bool,
    region: String,
    room: Option<String>,
    /// Print the region's rooms and exit.
    list_rooms: bool,
    /// With `--online`: create a new room on the scene instead of joining one.
    create: bool,
    /// Start in the menu at this screen (`--menu boot|main|connecting|lobby|create|loading`).
    menu: Option<String>,
    /// Sample rooms in the lobby while offline.
    demo_rooms: bool,
    /// The menu connects and creates a room on its own (tests).
    menu_auto: bool,
    /// Open the in-game Tab menu on start (screenshots).
    pause: bool,
    /// The client's master volume, saved in PlayerPrefs.
    volume: Option<f32>,
    /// The starting quality level (the room menu's list; not saved), for screenshots.
    quality: Option<usize>,
    /// A scene was named: skip the menu.
    scene_given: bool,
}

impl Args {
    /// The menu runs unless something asks for a game directly (a scene, `--online`, the
    /// offline test options).
    fn use_menu(&self) -> bool {
        if self.menu.is_some() || self.menu_auto {
            return true;
        }
        !(self.scene_given
            || self.dummy
            || self.online
            || self.camera.is_some()
            || self.spawn.is_some()
            || self.fly
            || self.weapon.is_some()
            || self.fire
            || self.aim)
    }
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        scene: "Bridge".to_string(),
        spawn: None,
        camera: None,
        fly: false,
        walk: false,
        screenshot: None,
        wait: 0.0,
        stats: false,
        weapon: None,
        fire: false,
        aim: false,
        mode: "DM".to_string(),
        team: 2,
        name: "Player".to_string(),
        dummy: false,
        dummy_team: 1,
        dummy_spawn: None,
        dummy_weapon: 2,
        dummy_fire: false,
        online: false,
        region: "eu".to_string(),
        room: None,
        list_rooms: false,
        create: false,
        menu: None,
        demo_rooms: false,
        menu_auto: false,
        pause: false,
        volume: None,
        quality: None,
        scene_given: false,
    };
    let team = |text: String| -> Result<u8> {
        match text.as_str() {
            "1" => Ok(1),
            "2" => Ok(2),
            _ => bail!("teams are 1 or 2"),
        }
    };
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--spawn" => args.spawn = Some(iter.next().context("--spawn N")?.parse()?),
            "--camera" => {
                let text = iter.next().context("--camera x,y,z,yaw,pitch")?;
                let values: Vec<f32> = text.split(',').map(str::parse).collect::<Result<_, _>>()?;
                let Ok(values) = <[f32; 5]>::try_from(values) else {
                    bail!("--camera needs 5 comma-separated numbers");
                };
                args.camera = Some(values);
            }
            "--fly" => args.fly = true,
            "--walk" => args.walk = true,
            "--wait" => args.wait = iter.next().context("--wait SECONDS")?.parse()?,
            "--stats" => args.stats = true,
            "--weapon" => {
                let key: usize = iter.next().context("--weapon KEY")?.parse()?;
                if key > 9 {
                    bail!("--weapon takes a number key, 0-9");
                }
                args.weapon = Some(key);
            }
            "--fire" => args.fire = true,
            "--aim" => args.aim = true,
            "--screenshot" => {
                args.screenshot = Some(PathBuf::from(iter.next().context("--screenshot PATH")?))
            }
            "--mode" => {
                args.mode = iter.next().context("--mode DM|TDM")?;
                if args.mode != "DM" && args.mode != "TDM" {
                    bail!("--mode is DM or TDM");
                }
            }
            "--team" => args.team = team(iter.next().context("--team 1|2")?)?,
            "--name" => args.name = iter.next().context("--name NAME")?,
            "--dummy" => args.dummy = true,
            "--dummy-team" => args.dummy_team = team(iter.next().context("--dummy-team 1|2")?)?,
            "--dummy-spawn" => {
                args.dummy_spawn = Some(iter.next().context("--dummy-spawn N")?.parse()?)
            }
            "--dummy-weapon" => {
                args.dummy_weapon = iter.next().context("--dummy-weapon SLOT")?.parse()?
            }
            "--dummy-fire" => args.dummy_fire = true,
            "--online" => args.online = true,
            "--list-rooms" => args.list_rooms = true,
            "--create" => args.create = true,
            "--menu" => args.menu = Some(iter.next().context("--menu SCREEN")?),
            "--demo-rooms" => args.demo_rooms = true,
            "--menu-auto" => args.menu_auto = true,
            "--pause" => args.pause = true,
            "--quality" => {
                args.quality = Some(iter.next().context("--quality 0..5")?.parse()?);
            }
            "--volume" => {
                let volume: f32 = iter.next().context("--volume 0..1")?.parse()?;
                if !(0.0..=1.0).contains(&volume) {
                    bail!("--volume is between 0 and 1");
                }
                args.volume = Some(volume);
            }
            "--region" => args.region = iter.next().context("--region us|eu|asia|jp|au|ru")?,
            "--room" => args.room = Some(iter.next().context("--room TEXT")?),
            other if other.starts_with("--") => bail!("unknown option {other}"),
            scene => {
                args.scene = scene.to_string();
                args.scene_given = true;
            }
        }
    }
    Ok(args)
}

/// Free-fly camera using Unity's conventions: yaw about +Y, pitch about +X (positive = down).
struct FlyCamera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl FlyCamera {
    fn rotation(&self) -> Quat {
        Quat::from_euler(
            EulerRot::YXZ,
            self.yaw.to_radians(),
            self.pitch.to_radians(),
            0.0,
        )
    }

    fn look(&mut self, rotation: Quat) {
        let forward = rotation * Vec3::Z;
        self.yaw = forward.x.atan2(forward.z).to_degrees();
        self.pitch = (-forward.y).clamp(-1.0, 1.0).asin().to_degrees();
    }
}

#[derive(Default)]
struct Input {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    fast: bool,
    run: bool,
    /// Keys that went down since the last frame (`GetKeyDown`).
    jump_pressed: bool,
    crouch_pressed: bool,
    prone_pressed: bool,
    mouse_delta: (f64, f64),
    grabbed: bool,
    /// Letter keys only, as `CF2Input.GetKey(KeyCode.W)` etc. in `CharacterAnimation`.
    key_w: bool,
    key_a: bool,
    key_s: bool,
    key_d: bool,
    key_r: bool,
    /// Mouse buttons (only while the cursor is captured) and the weapon keys.
    fire_held: bool,
    fire_pressed: bool,
    aim_pressed: bool,
    reload_pressed: bool,
    number_pressed: [bool; 10],
    /// `Input.GetAxis("Mouse ScrollWheel")` of this frame.
    scroll: f32,
    key_m: bool,
    key_n: bool,
}

/// A player prefab with what every instance of it shares.
struct PlayerKind {
    /// The prefab as loaded, before any script ran.
    pristine: PrefabInstance,
    models: PrefabModels,
    /// Its death prefab (`PlayerDamage.ragdoll`) and the GPU models.
    ragdoll: Option<(PrefabInstance, PrefabModels)>,
    /// `lookTarget`'s local transform (under `LookObject`), for the serialized look target.
    look_target_local: Trs,
}

/// The local player's prefab instance as the owner sees it (`localObjectsToDeactivate` applied):
/// only the first-person weapon under `Weapon Camera` is visible.
struct LocalAvatar {
    /// Index into `Running::kinds`.
    kind: usize,
    prefab: PrefabInstance,
    weapons: Option<WeaponManager>,
    main_camera: Option<usize>,
    weapon_camera: Option<usize>,
    /// Moved by `WalkSway` (`Player::weapon_local_position`).
    weapon_manager: Option<usize>,
    /// Prefabs the weapons (local and remote) spawn, their GPU models, and the live instances.
    templates: Templates,
    template_models: Vec<PrefabModels>,
    projectiles: Projectiles,
    scripts: AvatarScripts,
}

/// Per-life state of the local player's scripts (recreated on respawn).
struct AvatarScripts {
    damage: Option<PlayerDamage>,
    protection: SpawnProtection,
    ammo: Option<AmmoDisplay>,
    footsteps: Option<FpsSoundController>,
    /// `CharacterAnimation` on the (hidden) body: its strings are serialized to the others.
    character: Option<CharacterAnimation>,
    step_random: unity::Random,
    /// AudioSources: the root's (footsteps), the WeaponManager's, each weapon's (slot order).
    root_source: Option<u64>,
    manager_source: Option<(u64, usize)>,
    weapon_sources: Vec<Option<(u64, usize)>>,
    /// Particle emitters in the prefab (the shotgun's smoke): (node, emitter).
    systems: Vec<(usize, effects::NodeParticles)>,
    /// `PhotonNetwork.Destroy` ran (after death): the player object is gone.
    destroyed: bool,
}

/// A death prefab (`german (ragdol)` / `mercenarys (ragdol)`): an animated body with its own
/// camera (cleared on the other players' deaths).
struct Ragdoll {
    kind: usize,
    prefab: PrefabInstance,
    controller: Option<RagdollController>,
    sources: Vec<u64>,
    destroy_at: f32,
}

enum Mode {
    Walk,
    Fly,
}

/// Field order is drop order: GL objects, then the surface and context, the window last
/// (dropping the window first leaves the EGL surface dangling and crashes on exit).
struct Running {
    renderer: SceneRenderer,
    kinds: Vec<PlayerKind>,
    local: LocalAvatar,
    /// The local player as the other clients see it, fed by its own serialized state: drawn in
    /// fly mode.
    mirror: Option<RemotePlayer>,
    /// The other players' copies.
    remotes: Vec<RemotePlayer>,
    /// Simulated other clients (`--dummy`).
    dummies: Vec<DummyOwner>,
    /// The connection to the original servers (`--online`).
    online: Option<Online>,
    room: Room,
    /// This client's player properties.
    props: PlayerProps,
    /// Next `OnPhotonSerializeView` of the local player.
    serialize_at: f32,
    clips: ClipLibrary,
    scene: Scene,
    world: CollisionWorld,
    player: Player,
    mode: Mode,
    horizontal: KeyAxis,
    vertical: KeyAxis,
    fixed_time: f32,
    camera: FlyCamera,
    fov: (f32, f32, f32),
    input: Input,
    spawn_index: usize,
    last_frame: Instant,
    started: Instant,
    frames: u32,
    /// `--aim` already pressed.
    aimed: bool,
    /// `Time.time` for scene objects (projectiles, effects, the death prefab), which keep
    /// running while the player is dead; `fixed_clock` is `Time.fixedTime`.
    time: f32,
    fixed_clock: f32,
    /// This frame's unclamped delta time.
    last_dt: f32,
    audio: Audio,
    effects: Effects,
    meshes: MeshCache,
    gui: Gui,
    font: Option<GuiFont>,
    /// `PlayerDamage`/`AmmoDisplay`'s skin, the death prefab's, the kill feed's, Unity's default.
    fps_skin: Option<GuiSkin>,
    ragdoll_skin: Option<GuiSkin>,
    feed_skin: Option<GuiSkin>,
    default_skin: Option<DefaultSkin>,
    ragdolls: Vec<Ragdoll>,
    assets: PathBuf,
    dummy_spawn: Option<usize>,
    dummy_weapon: usize,
    /// Frame-time statistics since `stats_start`: count, sum, max, CPU time in update+render.
    stats_start: Instant,
    stats_frames: u32,
    stats_max: f32,
    stats_cpu: f32,
    /// Started from the menu: "Leave Room" goes back to the lobby.
    from_menu: bool,
    /// Tab and P are the room menu's keys (else the test keys: next spawn, print the camera).
    room_keys: bool,
    /// `RoomMultiplayerMenu` (the Tab menu).
    room_menu: Option<RoomMenu>,
    /// The room's full name.
    room_name: String,
    /// The map's `roomCamera` renders until the first `SpawnPlayer` deactivates it.
    room_camera: bool,
    /// The `EnableHelper` object's active state, as the room menu's last `Update` left it.
    helper_active: bool,
    /// `roundDuration` (s), `referenceTime`, `currentRoundTime`, and when `WaitBeforeCheck`
    /// clears `delayAction` (`Time.time`).
    round_duration: i32,
    reference_time: f32,
    round_time: f32,
    round_check_at: f32,
    /// `Screen.resolutions`.
    resolutions: Vec<(u32, u32)>,
    cursor: Vec2,
    /// The left button went down / up and the wheel turned since the last frame (room menu).
    menu_down: bool,
    menu_up: bool,
    menu_wheel: f32,
    /// `--menu-auto`: the team button pressed one second into the room (tests).
    auto_team: Option<u8>,
    settings: SharedSettings,
    gl: Rc<glow::Context>,
    window: Rc<Window>,
}

/// The window, its GL context and surface. Field order is drop order (the window last).
struct Platform {
    gl: Rc<glow::Context>,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    window: Rc<Window>,
}

/// The menu with what draws it (the `Main` scene's renderer).
struct MenuScreen {
    menu: menu::Menu,
    renderer: SceneRenderer,
    input: menu::MenuInput,
    mouse: Vec2,
    last_frame: Instant,
    started: Instant,
    frames: u32,
}

enum Screen {
    Menu(Box<MenuScreen>),
    Game(Box<Running>),
}

/// Field order is drop order: the screens release their GL objects before the platform.
struct App {
    args: Args,
    settings: SharedSettings,
    online: Option<Online>,
    screen: Option<Screen>,
    /// The menu while a game runs, for the way back.
    stash: Option<Box<MenuScreen>>,
    platform: Option<Platform>,
    error: Option<anyhow::Error>,
}

fn team_name(room: &Room, team: u8) -> String {
    if team == 1 {
        room.team_1.name.clone()
    } else {
        room.team_2.name.clone()
    }
}

fn create_platform(event_loop: &ActiveEventLoop, args: &Args) -> Result<Platform> {
    let title = if args.use_menu() {
        "Pixel Gun Apocalypse 3".to_string()
    } else {
        format!("Pixel Gun Apocalypse 3 — {}", args.scene)
    };
    let attributes = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(1280.0, 720.0));
    let template = ConfigTemplateBuilder::new().with_depth_size(24);
    let (window, config) = DisplayBuilder::new()
        .with_window_attributes(Some(attributes))
        .build(event_loop, template, |configs| {
            configs
                .reduce(|best, c| {
                    if c.num_samples() < best.num_samples() {
                        c
                    } else {
                        best
                    }
                })
                .expect("no GL config")
        })
        .map_err(|e| anyhow::anyhow!("creating window: {e}"))?;
    let window = window.context("no window")?;
    let display = config.display();
    let raw_handle = window.window_handle().ok().map(|h| h.as_raw());
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .with_profile(GlProfile::Core)
        .build(raw_handle);
    let not_current = unsafe { display.create_context(&config, &context_attributes) }?;
    let surface_attributes = window.build_surface_attributes(Default::default())?;
    let surface = unsafe { display.create_window_surface(&config, &surface_attributes) }?;
    let context = not_current.make_current(&surface)?;
    // On Wayland winit already paces RedrawRequested to the compositor's frame callbacks
    // (`pre_present_notify`); a blocking EGL swap on top of that waits a second refresh and
    // halves the frame rate. Elsewhere the swap interval is the vsync.
    let wayland = matches!(raw_handle, Some(RawWindowHandle::Wayland(_)));
    let interval = if wayland || std::env::var_os("PGA3_NO_VSYNC").is_some() {
        SwapInterval::DontWait
    } else {
        SwapInterval::Wait(NonZeroU32::MIN)
    };
    let _ = surface.set_swap_interval(&context, interval);
    let gl =
        unsafe { glow::Context::from_loader_function_cstr(|name| display.get_proc_address(name)) };

    Ok(Platform {
        gl: Rc::new(gl),
        surface,
        context,
        window: Rc::new(window),
    })
}

fn create_menu(platform: &Platform, args: &Args, settings: &SharedSettings) -> Result<MenuScreen> {
    let assets = assets::find_assets_dir()?;
    let scene = scene::load_scene(&assets, "Main")?;
    let mut renderer = SceneRenderer::new(&platform.gl, &assets, &scene)?;
    renderer.set_texture_limit(&platform.gl, settings.borrow().texture_limit());
    let mut menu = menu::Menu::new(
        &assets,
        settings.clone(),
        args.demo_rooms,
        &args.name,
        &args.region,
    )?;
    menu.screenshot = args.screenshot.is_some();
    menu.auto = args.menu_auto;
    if let Some(screen) = &args.menu
        && !menu.show(screen)
    {
        bail!("--menu SCREEN is boot, main, connecting, lobby, create or loading");
    }
    Ok(MenuScreen {
        menu,
        renderer,
        input: menu::MenuInput::default(),
        mouse: Vec2::ZERO,
        last_frame: Instant::now(),
        started: Instant::now(),
        frames: 0,
    })
}

/// How a game starts.
struct GameStart {
    /// The original flow: the room menu is open and no player exists until a team is chosen.
    from_menu: bool,
    /// The room's full name (offline: the menu's).
    room_name: String,
    /// `RoundDuration` (s) when no room property gives it.
    round_duration: i32,
}

/// `Screen.resolutions`: the monitor's video modes, unique sizes in ascending order.
fn screen_resolutions(window: &Window) -> Vec<(u32, u32)> {
    let mut sizes: Vec<(u32, u32)> = window
        .current_monitor()
        .map(|m| {
            m.video_modes()
                .map(|v| (v.size().width, v.size().height))
                .collect()
        })
        .unwrap_or_default();
    sizes.sort_unstable();
    sizes.dedup();
    if sizes.is_empty() {
        let size = window.inner_size();
        sizes.push((size.width, size.height));
    }
    sizes
}

fn create(
    platform: &Platform,
    args: &Args,
    mut online: Option<Online>,
    settings: &SharedSettings,
    start: &GameStart,
) -> Result<Running> {
    let gl = platform.gl.clone();
    let window = platform.window.clone();
    let assets = assets::find_assets_dir()?;
    let loading = Instant::now();
    let scene = scene::load_scene(&assets, &args.scene)?;
    let mut renderer = SceneRenderer::new(&gl, &assets, &scene)?;
    renderer.set_texture_limit(&gl, settings.borrow().texture_limit());
    let world = physics::build_world(&assets, &scene)?;
    // Screenshot runs stay silent.
    let mut audio = if args.screenshot.is_some() {
        Audio::silent(&assets)
    } else {
        Audio::new(&assets)
    };
    audio.set_volume(settings.borrow().volume);
    {
        // RoomMultiplayerMenu.Start: the saved sensitivity.
        let mut s = settings.borrow_mut();
        if s.prefs.has_key("sens") {
            s.sens = s.prefs.get_float("sens", s.sens);
        }
    }
    let mut meshes = MeshCache::new(&assets);
    let mut clips = ClipLibrary::new(&assets)?;
    let kinds = load_kinds(&gl, &assets, &mut renderer, &mut clips)?;
    let mut room = Room::new(&scene, &args.mode);
    if let Some(online) = &online {
        room.team1_score = online.room_int("Team1Score").unwrap_or(0);
        room.team2_score = online.room_int("Team2Score").unwrap_or(0);
    }
    let team = if start.from_menu {
        SPECTATORS.to_string()
    } else {
        team_name(&room, args.team)
    };
    let props = PlayerProps {
        actor: online.as_ref().map_or(LOCAL_ACTOR, Online::actor),
        name: args.name.clone(),
        team,
        kills: 0,
        deaths: 0,
    };
    // WhoKilledWho.Start (PlayerJoinedRoom), then (test starts) the team menu's PlayerJoinedTeam.
    room.add_message(&props.name, "", "connected", "");
    if !start.from_menu {
        room.add_message(&props.name, "", &format!("joined {}", props.team), "");
    }
    if let Some(online) = &mut online {
        online.joined(&props.name);
    }
    let round_duration = online
        .as_ref()
        .and_then(|o| o.room_int("RoundDuration"))
        .unwrap_or(start.round_duration);
    let reference_time = online
        .as_ref()
        .and_then(|o| o.room_float("RefTime"))
        .unwrap_or(0.0);
    let room_name = match online.as_ref().map(Online::room_name) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => start.room_name.clone(),
    };
    let resolutions = screen_resolutions(&window);
    let room_menu = match gui::load_named_skin(&assets, room_menu::ROOM_SKIN) {
        Ok(skin) => {
            let mut menu = RoomMenu::new(skin, resolutions.len());
            menu.paused = start.from_menu || args.pause;
            Some(menu)
        }
        Err(error) => {
            eprintln!("room menu: {error:#}");
            None
        }
    };
    // SpawnPlayer(team): the spawn point and the prefab of the team.
    let (team_spawn, team_1) = room.spawn_point(&props.team);
    let kind = if team_1 { 0 } else { 1 };
    let custom_tags = assets::load_tags(&assets)?;
    let refs: Vec<String> = kinds
        .iter()
        .flat_map(|k| weapon_prefab_refs(&k.pristine))
        .collect();
    let templates = Templates::load(&assets, &refs, custom_tags, &mut clips)?;
    let template_models = templates
        .list
        .iter()
        .map(|t| renderer.prepare_prefab(&gl, &t.prefab))
        .collect::<Result<Vec<_>>>()?;
    let mut player = Player::from_prefab(&assets, PLAYER_KINDS[kind])?;
    let local = instantiate_local(
        &kinds,
        kind,
        templates,
        template_models,
        &player,
        &mut audio,
        &mut meshes,
        &mut clips,
    );
    let font = match GuiFont::load(&assets) {
        Ok(font) => Some(font),
        Err(error) => {
            eprintln!("GUI font: {error:#}");
            None
        }
    };
    let skin = |reference: Option<&str>| reference.and_then(|r| gui::load_skin(&assets, r).ok());
    let fps_skin = skin(
        local
            .scripts
            .damage
            .as_ref()
            .and_then(|d| d.skin.as_deref()),
    );
    let ragdoll_skin = kinds[kind].ragdoll.as_ref().and_then(|(prefab, _)| {
        let (_, c) = prefab
            .find_scripts("RagdollController")
            .into_iter()
            .next()?;
        skin(ref_of(&c["fields"]["guiSKin"]).map(|(r, _)| r))
    });
    let feed_skin = skin(scene.kill_feed_skin.as_deref());
    let default_skin = skin(Some(gui::DEFAULT_SKIN)).map(|skin| DefaultSkin { skin });
    println!(
        "collision: {} colliders, {} triangles",
        scene.colliders.len(),
        world.triangle_count()
    );
    println!(
        "{}: {} renderers ({} skipped), {} spawn points, {:?}, loaded in {:.2?}",
        scene.name,
        scene.renderers.len(),
        scene.skipped_renderers,
        scene.spawn_points.len(),
        renderer.stats,
        loading.elapsed()
    );

    let (fov, near, far) = match &scene.camera {
        Some(c) => (c.fov_y_degrees, c.near, c.far),
        None => (60.0, 0.3, 1000.0),
    };
    let mut camera = FlyCamera {
        position: Vec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
    };
    if let Some(c) = &scene.camera {
        camera.position = c.position;
        camera.look(c.rotation);
    }
    let spawn_index = args.spawn.or(team_spawn).unwrap_or(0);
    let mut mode = if args.fly || scene.spawn_points.is_empty() {
        Mode::Fly
    } else {
        Mode::Walk
    };
    place_at_spawn(&mut player, &mut camera, &scene, spawn_index);
    if let Some([x, y, z, yaw, pitch]) = args.camera {
        camera = FlyCamera {
            position: Vec3::new(x, y, z),
            yaw,
            pitch,
        };
        mode = Mode::Fly;
        if args.walk {
            drop_player_at_camera(&mut player, &camera);
            mode = Mode::Walk;
        }
    }
    let mut dummies = Vec::new();
    if args.dummy {
        let props = PlayerProps {
            actor: LOCAL_ACTOR + 1,
            name: "Dummy".to_string(),
            team: team_name(&room, args.dummy_team),
            kills: 0,
            deaths: 0,
        };
        dummies.push(DummyOwner::new(props, args.dummy_fire));
    }

    let mut running = Running {
        window,
        gl,
        from_menu: start.from_menu,
        room_keys: start.from_menu || args.pause,
        room_menu,
        room_name,
        room_camera: false,
        helper_active: !(start.from_menu || args.pause),
        round_duration,
        reference_time,
        round_time: 0.0,
        round_check_at: room_menu::ROUND_DELAY,
        resolutions,
        cursor: Vec2::ZERO,
        menu_down: false,
        menu_up: false,
        menu_wheel: 0.0,
        auto_team: None,
        settings: settings.clone(),
        scene,
        renderer,
        kinds,
        local,
        mirror: None,
        remotes: Vec::new(),
        dummies,
        online,
        room,
        props,
        serialize_at: 0.0,
        clips,
        world,
        player,
        mode,
        horizontal: KeyAxis::movement(),
        vertical: KeyAxis::movement(),
        fixed_time: 0.0,
        camera,
        fov: (fov, near, far),
        input: Input::default(),
        spawn_index,
        last_frame: Instant::now(),
        started: Instant::now(),
        frames: 0,
        aimed: false,
        stats_start: Instant::now(),
        stats_frames: 0,
        stats_max: 0.0,
        stats_cpu: 0.0,
        time: 0.0,
        fixed_clock: 0.0,
        last_dt: 0.0,
        audio,
        effects: Effects::default(),
        meshes,
        gui: Gui::new(&assets),
        font,
        fps_skin,
        ragdoll_skin,
        feed_skin,
        default_skin,
        ragdolls: Vec::new(),
        assets,
        dummy_spawn: args.dummy_spawn,
        dummy_weapon: args.dummy_weapon,
    };
    running.sync_local_avatar();
    if start.from_menu {
        // `Awake`: no player yet; the room camera shows the map.
        running.local.scripts.destroyed = true;
        running.local.scripts.remove_sources(&mut running.audio);
        running.room_camera = true;
    } else {
        running.create_mirror();
        running.instantiate_online();
        if let Some(online) = &running.online {
            let name = running.props.name.clone();
            online.send_message(&name, "", &format!("joined {}", running.props.team), "");
        }
    }
    for index in 0..running.dummies.len() {
        running.spawn_dummy(index);
    }
    Ok(running)
}

/// Loads both player prefabs with their death prefabs and GPU models.
fn load_kinds(
    gl: &glow::Context,
    assets: &std::path::Path,
    renderer: &mut SceneRenderer,
    clips: &mut ClipLibrary,
) -> Result<Vec<PlayerKind>> {
    let mut kinds = Vec::new();
    for name in PLAYER_KINDS {
        let pristine = PrefabInstance::load(assets, name, clips)?;
        let models = renderer.prepare_prefab(gl, &pristine)?;
        let ragdoll_ref = pristine
            .script_fields(0, "PlayerDamage")
            .and_then(|f| ref_of(&f["ragdoll"]).map(|(r, _)| r.to_string()));
        let ragdoll = match ragdoll_ref {
            Some(reference) => load_ragdoll(gl, assets, renderer, &reference, clips)?,
            None => None,
        };
        let look_target_local = pristine
            .script_fields(0, "PlayerNetworkController")
            .and_then(|f| pristine.node_of_ref(&f["lookTarget"]))
            .map_or(Trs::IDENTITY, |n| pristine.nodes[n].local);
        println!(
            "{name}: {} renderers ({} skipped), {} animations",
            pristine.renderers.len(),
            pristine.skipped_renderers,
            pristine.animations.len(),
        );
        kinds.push(PlayerKind {
            pristine,
            models,
            ragdoll,
            look_target_local,
        });
    }
    Ok(kinds)
}

/// The bullet / projectile prefab refs of every `WeaponScript` in a player prefab.
fn weapon_prefab_refs(prefab: &PrefabInstance) -> Vec<String> {
    prefab
        .find_scripts("WeaponScript")
        .into_iter()
        .flat_map(|(_, c)| {
            let f = &c["fields"];
            [
                &f["machineGun"]["bullet"],
                &f["ShotGun"]["bullet"],
                &f["grenadeLauncher"]["projectile"],
                &f["knife"]["bullet"],
            ]
            .into_iter()
            .filter_map(|r| ref_of(r).map(|(r, _)| r.to_string()))
            .collect::<Vec<_>>()
        })
        .collect()
}

/// The owner's instance of a player prefab with its start-up scripts run, in Unity's order:
/// `PlayerNetworkController.Awake` deactivations, Animation `playAutomatically`,
/// `WeaponManager.Awake` (weapon take-in), the scripts' `Awake`/`Start`, `CharacterAnimation.Start`.
#[allow(clippy::too_many_arguments)]
fn instantiate_local(
    kinds: &[PlayerKind],
    kind: usize,
    templates: Templates,
    template_models: Vec<PrefabModels>,
    player: &Player,
    audio: &mut Audio,
    meshes: &mut MeshCache,
    clips: &mut ClipLibrary,
) -> LocalAvatar {
    let mut prefab = kinds[kind].pristine.clone();
    let controller = prefab
        .script_fields(0, "PlayerNetworkController")
        .cloned()
        .unwrap_or(Value::Null);
    prefab.set_active_refs(&controller["localObjectsToDeactivate"], false);
    prefab.play_automatically();
    let weapons = weapons::weapon_manager_awake(&mut prefab);
    let mut scripts = AvatarScripts::start(&prefab, weapons.as_ref(), player, audio, meshes);
    scripts.character = CharacterAnimation::start(&mut prefab, clips);
    LocalAvatar {
        kind,
        main_camera: prefab.find("Main Camera"),
        weapon_camera: prefab.find("Weapon Camera"),
        weapon_manager: prefab.find("WeaponManager"),
        prefab,
        weapons,
        templates,
        template_models,
        projectiles: Projectiles::default(),
        scripts,
    }
}

/// The first AudioSource of a node, added to the mixer.
fn add_node_source(prefab: &PrefabInstance, node: usize, audio: &mut Audio) -> Option<u64> {
    let settings = SourceSettings::from_component(prefab.components(node, "AudioSource").next()?);
    Some(audio.add_source(settings, prefab.world_of(node).w_axis.truncate()))
}

impl AvatarScripts {
    /// `Awake`/`Start` of the scripts on a freshly instantiated owner's player.
    fn start(
        prefab: &PrefabInstance,
        weapons: Option<&WeaponManager>,
        player: &Player,
        audio: &mut Audio,
        meshes: &mut MeshCache,
    ) -> AvatarScripts {
        let damage = prefab
            .script_fields(0, "PlayerDamage")
            .map(|f| PlayerDamage::awake(f, player.camera_local_rotation));
        let manager_node = prefab
            .find_scripts("WeaponManager")
            .first()
            .map(|(n, _)| *n);
        let ammo = manager_node
            .and_then(|n| prefab.script_fields(n, "AmmoDisplay"))
            .map(|f| AmmoDisplay::new(f["display"].as_bool().unwrap_or(true)));
        let footsteps = prefab
            .script_fields(0, "FPSSoundController")
            .map(FpsSoundController::new);
        let root_source = add_node_source(prefab, 0, audio);
        let manager_source =
            manager_node.and_then(|n| add_node_source(prefab, n, audio).map(|id| (id, n)));
        let weapon_sources = weapons
            .map(|w| {
                w.all_weapons
                    .iter()
                    .map(|&n| add_node_source(prefab, n, audio).map(|id| (id, n)))
                    .collect()
            })
            .unwrap_or_default();
        let mut systems = Vec::new();
        for node in 0..prefab.nodes.len() {
            if let Some(mut system) =
                effects::NodeParticles::of_node(prefab, node, meshes, node as u32 + 1)
            {
                // `WeaponScript.Start` / `shotGunAwake`: `smoke.Stop()` / `smoke.emit = false`.
                system.set_emitting(false);
                systems.push((node, system));
            }
        }
        AvatarScripts {
            damage,
            protection: SpawnProtection::default(),
            ammo,
            footsteps,
            character: None,
            step_random: unity::Random::new(3),
            root_source,
            manager_source,
            weapon_sources,
            systems,
            destroyed: false,
        }
    }

    /// Every AudioSource of the player object is destroyed with it.
    fn remove_sources(&mut self, audio: &mut Audio) {
        let ids = self
            .root_source
            .take()
            .into_iter()
            .chain(self.manager_source.take().map(|(id, _)| id))
            .chain(self.weapon_sources.drain(..).flatten().map(|(id, _)| id));
        for id in ids {
            audio.remove(id);
        }
    }
}

/// Loads the death prefab a `PlayerDamage.ragdoll` ref names (a `Resources` prefab).
fn load_ragdoll(
    gl: &glow::Context,
    assets: &std::path::Path,
    renderer: &mut SceneRenderer,
    reference: &str,
    clips: &mut ClipLibrary,
) -> Result<Option<(PrefabInstance, PrefabModels)>> {
    let resources = assets::load_json(&assets.join("data/resources.json"))?;
    let name = resources.as_object().and_then(|map| {
        map.iter().find_map(|(name, objects)| {
            objects
                .as_array()?
                .iter()
                .any(|o| o["ref"] == reference)
                .then(|| name.clone())
        })
    });
    let Some(name) = name else {
        eprintln!("ragdoll prefab {reference} not found");
        return Ok(None);
    };
    let prefab = PrefabInstance::load(assets, &name, clips)?;
    let models = renderer.prepare_prefab(gl, &prefab)?;
    Ok(Some((prefab, models)))
}

/// What `WeaponSync_Catcher.Fire` would send for the weapon in `allWeapons[slot]`, not aimed.
fn dummy_weapon(prefab: &PrefabInstance, slot: usize) -> Option<DummyWeapon> {
    let (_, manager) = prefab.find_scripts("WeaponManager").into_iter().next()?;
    let node = prefab.node_of_ref(manager["fields"]["allWeapons"].get(slot)?)?;
    let f = prefab.script_fields(node, "WeaponScript")?;
    let int = |v: &Value| v.as_i64().unwrap_or(0) as f32;
    let (sync, interval) = match f["GunType"].as_i64().unwrap_or(0) {
        1 => {
            let g = &f["grenadeLauncher"];
            let cycle = assets::f32_of(&g["shotDelay"])
                + assets::f32_of(&g["waitBeforeReload"])
                + assets::f32_of(&g["reloadTime"]);
            (SyncFire::GrenadeLauncher(int(&g["initialSpeed"])), cycle)
        }
        2 => (
            SyncFire::ShotGun(int(&f["ShotGun"]["fractions"])),
            assets::f32_of(&f["ShotGun"]["fireRate"]),
        ),
        3 => (SyncFire::Knife, assets::f32_of(&f["knife"]["fireRate"])),
        _ => (
            SyncFire::MachineGun(assets::f32_of(&f["machineGun"]["NoAimErrorAngle"])),
            assets::f32_of(&f["machineGun"]["fireRate"]),
        ),
    };
    Some(DummyWeapon {
        node,
        name: f["weaponName"].as_str().unwrap_or_default().to_string(),
        sync,
        interval,
    })
}

/// The delta time the player's frame uses (`maximumDeltaTime` clamps it while walking).
fn dt_frame_of(mode: &Mode, dt: f32) -> f32 {
    match mode {
        Mode::Walk => dt.min(MAXIMUM_DELTA_TIME),
        Mode::Fly => dt,
    }
}

/// Puts the player where the fly camera is, looking its way.
fn drop_player_at_camera(player: &mut Player, camera: &FlyCamera) {
    let eye = player.camera_position() - player.position;
    let yaw = Quat::from_rotation_y(camera.yaw.to_radians());
    player.spawn(camera.position - eye, yaw);
    // `FPSMouseLook.rotationY` grows upwards; the fly pitch grows downwards.
    player.rotation_y = (-camera.pitch).clamp(player.minimum_y, player.maximum_y);
}

fn place_at_spawn(player: &mut Player, camera: &mut FlyCamera, scene: &Scene, index: usize) {
    if scene.spawn_points.is_empty() {
        return;
    }
    let spawn = &scene.spawn_points[index % scene.spawn_points.len()];
    player.spawn(spawn.position, spawn.rotation);
    camera.position = player.camera_position();
    camera.look(player.camera_rotation());
    println!(
        "spawn point {} ({}) at {}",
        index % scene.spawn_points.len(),
        spawn.team,
        spawn.position
    );
}

/// Places a death prefab at a player's transform (`Instantiate(ragdoll, position, rotation)`)
/// with its AudioSources; `RespawnAfter` on the owner's death, `clearCamera` on another
/// player's.
fn spawn_ragdoll(
    kinds: &[PlayerKind],
    kind: usize,
    position: Vec3,
    rotation: Quat,
    time: f32,
    owner: bool,
    audio: &mut Audio,
) -> Option<Ragdoll> {
    let (template, _) = kinds[kind].ragdoll.as_ref()?;
    let mut prefab = template.clone();
    prefab.nodes[0].local.position = position;
    prefab.nodes[0].local.rotation = rotation;
    prefab.play_automatically();
    prefab.update_world();
    let mut sources = Vec::new();
    for node in 0..prefab.nodes.len() {
        if !prefab.active_in_hierarchy(node) {
            continue;
        }
        for data in prefab.components(node, "AudioSource") {
            let settings = SourceSettings::from_component(data);
            let at = prefab.world_of(node).w_axis.truncate();
            sources.push(audio.add_source(settings, at));
        }
    }
    let mut controller = prefab
        .find_scripts("RagdollController")
        .into_iter()
        .next()
        .map(|(_, c)| {
            let cam = prefab.node_of_ref(&c["fields"]["cam"]);
            RagdollController::new(&c["fields"], cam)
        });
    if let Some(c) = &mut controller {
        if owner {
            c.respawn_after(time);
        } else {
            // clearCamera: the death camera is destroyed.
            c.cam = None;
        }
    }
    let life = prefab
        .script_fields(0, "WaitForDestroy")
        .map_or(f32::INFINITY, |f| assets::f32_of(&f["lifeTime"]));
    Some(Ragdoll {
        kind,
        prefab,
        controller,
        sources,
        destroy_at: time + life,
    })
}

impl Running {
    fn set_grab(&mut self, grab: bool) {
        if grab {
            let ok = self
                .window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));
            if ok.is_err() {
                return;
            }
        } else {
            let _ = self.window.set_cursor_grab(CursorGrabMode::None);
        }
        self.window.set_cursor_visible(!grab);
        self.input.grabbed = grab;
    }

    fn update(&mut self, dt: f32) {
        let dt_frame = dt_frame_of(&self.mode, dt);
        self.last_dt = dt;
        self.local.prefab.begin_frame(dt_frame);
        for remote in self.remotes.iter_mut().chain(self.mirror.as_mut()) {
            remote.prefab.begin_frame(dt_frame);
        }
        for ragdoll in &mut self.ragdolls {
            ragdoll.prefab.begin_frame(dt_frame);
        }
        match self.mode {
            Mode::Walk => self.update_walk(dt),
            Mode::Fly => self.update_fly(dt),
        }
        self.late_animation_update();
        let i = &mut self.input;
        i.jump_pressed = false;
        i.crouch_pressed = false;
        i.prone_pressed = false;
        i.fire_pressed = false;
        i.aim_pressed = false;
        i.reload_pressed = false;
        i.number_pressed = [false; 10];
        i.scroll = 0.0;
    }

    /// One Unity frame for the local player: FixedUpdate steps, Update, LateUpdate.
    fn update_walk(&mut self, dt: f32) {
        let dt = dt.min(MAXIMUM_DELTA_TIME);
        let (dx, dy) = std::mem::take(&mut self.input.mouse_delta);
        let i = &self.input;
        let input = FrameInput {
            horizontal: self.horizontal.update(i.right, i.left, dt),
            vertical: self.vertical.update(i.forward, i.back, dt),
            jump_down: i.jump_pressed,
            jump_held: i.up,
            run_held: i.run,
            crouch_down: i.crouch_pressed,
            prone_down: i.prone_pressed,
            mouse_x: dx as f32 * MOUSE_AXIS_SENSITIVITY,
            // Unity's Mouse Y grows upwards; window coordinates grow downwards.
            mouse_y: -dy as f32 * MOUSE_AXIS_SENSITIVITY,
        };
        let weapon_input = WeaponInput {
            mouse_x: input.mouse_x,
            mouse_y: input.mouse_y,
            vertical: input.vertical,
            velocity_y: 0.0,
            fire_held: i.fire_held,
            fire_down: i.fire_pressed,
            aim_down: i.aim_pressed,
            reload_down: i.reload_pressed,
            number_down: i.number_pressed,
            scroll: i.scroll,
            next_held: i.key_m,
            previous_held: i.key_n,
        };
        self.step_player(dt, &input, weapon_input);
        self.camera.position = self.player.camera_position();
        self.camera.look(self.player.camera_rotation());
    }

    /// The moving colliders of the other players (bullets and rockets hit them).
    fn targets(&self) -> Targets {
        let mut targets = Targets::default();
        for (index, remote) in self.remotes.iter().enumerate() {
            remote.add_targets(index, &mut targets);
        }
        targets
    }

    /// FixedUpdate steps, Update, coroutines, the animation pass, LateUpdate.
    fn step_player(&mut self, dt: f32, input: &FrameInput, mut weapon_input: WeaponInput) {
        self.time += dt;
        let world_time = self.time;
        // Photon's dispatch: what the other clients sent reaches the copies first.
        self.network_frame(dt, world_time);
        let view = LocalView {
            game_mode: &self.room.game_mode,
            team: &self.props.team,
        };
        for remote in &mut self.remotes {
            remote.update(dt, world_time, view);
        }
        if let Some(mirror) = &mut self.mirror {
            mirror.update(dt, world_time, view);
        }
        let targets = self.targets();
        let alive = !self.local.scripts.destroyed;
        let local = &mut self.local;
        self.fixed_time += dt;
        while self.fixed_time >= FIXED_DELTA_TIME {
            self.fixed_time -= FIXED_DELTA_TIME;
            self.fixed_clock += FIXED_DELTA_TIME;
            if alive {
                // EnableHelper.FixedUpdate: the room menu's last Update decides.
                let active = self.helper_active;
                self.player.can_control = active;
                self.player.look_enabled = active;
                if let Some(weapons) = &mut local.weapons {
                    weapons.enable_helper(active);
                }
                self.player
                    .fixed_update(&self.world, &self.scene.ladders, input);
                // FPSSoundController.FixedUpdate.
                let scripts = &mut local.scripts;
                if let Some(steps) = &mut scripts.footsteps
                    && let Some(clip) =
                        steps.fixed_update(&self.player, self.fixed_clock, &mut scripts.step_random)
                    && let Some(source) = scripts.root_source
                {
                    self.audio.play(source, &clip);
                }
            }
            local.projectiles.fixed_update(
                &local.templates,
                &self.world,
                &targets,
                FIXED_DELTA_TIME,
                world_time,
            );
        }
        if alive {
            self.player.update(&self.world, dt, input.vertical, input);
            // The end of FPSMouseLook.Update: the sensitivity of the next frame.
            let sens = self.settings.borrow().sens;
            let weapons = local.weapons.as_ref();
            let selected = weapons.and_then(|w| w.selected_weapon);
            self.player.update_look_sensitivity(sens, selected, |node| {
                weapons
                    .and_then(|w| w.script_of(node))
                    .is_some_and(|s| s.aimed)
            });
        }
        let time = self.player.time;
        if alive {
            weapons::sync_avatar(
                &mut local.prefab,
                &self.player,
                local.main_camera,
                local.weapon_manager,
            );
        }
        // Objects spawned last frame run Start, then every Update.
        local
            .projectiles
            .update(&local.templates, &self.world, &targets, dt, world_time);
        let manager_active = alive && hud::player_gui_active(&local.prefab, local.weapon_manager);
        if let Some(weapons) = &mut local.weapons
            && manager_active
        {
            weapon_input.velocity_y = self.player.controller_velocity.y;
            let mut ctx = WeaponContext {
                prefab: &mut local.prefab,
                player: &mut self.player,
                world: &self.world,
                input: &weapon_input,
                dt,
                time,
            };
            weapons.update(&mut ctx);
            // WaitForSeconds resumes after every Update.
            weapons.run_coroutines(&mut ctx);
            for event in weapons.events.drain(..) {
                match event {
                    WeaponEvent::Spawn {
                        prefab,
                        position,
                        rotation,
                        velocity,
                    } => match local.templates.find(&prefab) {
                        Some(template) => local.projectiles.spawn_with(
                            &local.templates,
                            template,
                            position,
                            rotation,
                            velocity,
                            SpawnFlags::LOCAL,
                        ),
                        None => eprintln!("weapon spawned unknown prefab {prefab}"),
                    },
                    WeaponEvent::Sound { source, clip } => {
                        let id = match source {
                            SoundSource::Manager => local.scripts.manager_source.map(|(id, _)| id),
                            SoundSource::Weapon(slot) => local
                                .scripts
                                .weapon_sources
                                .get(slot)
                                .copied()
                                .flatten()
                                .map(|(id, _)| id),
                        };
                        if let Some(id) = id {
                            self.audio.play(id, &clip);
                        }
                    }
                    // `SyncWeaponsRPC` to the others.
                    WeaponEvent::SyncFire(sync) => {
                        if let Some(online) = &self.online {
                            online.sync_fire(sync);
                        }
                    }
                }
            }
            if let Some(ammo) = &mut local.scripts.ammo {
                ammo.update(weapons, dt);
            }
            // `smoke.Play()` / `Stop()` of the shotgun.
            for script in weapons.scripts.iter().flatten() {
                let Some(node) = script.shot_gun.smoke else {
                    continue;
                };
                for (n, system) in &mut local.scripts.systems {
                    if *n == node && script.shot_gun.smoke_playing != system.emitting() {
                        system.set_emitting(script.shot_gun.smoke_playing);
                    }
                }
            }
        }
        let mut hits = Vec::new();
        for event in local.projectiles.events.drain(..) {
            match event {
                ProjectileEvent::Effect {
                    prefab,
                    position,
                    rotation,
                    is_remote,
                } => {
                    if let Some(template) =
                        local.templates.list.iter().position(|t| t.path == prefab)
                        && let Some(effect) = self.effects.spawn(
                            &local.templates,
                            template,
                            position,
                            rotation,
                            None,
                            world_time,
                            &mut self.audio,
                            &mut self.meshes,
                        )
                    {
                        effect.is_remote = is_remote;
                    }
                }
                // `SendMessageUpwards("ApplyDamage")`: only the other players' hit boxes have a
                // receiver.
                ProjectileEvent::Damage {
                    amount,
                    target: Some((owner, node)),
                    ..
                } => hits.push((owner, node, amount as f32)),
                ProjectileEvent::Damage { target: None, .. } => {}
            }
        }
        let mut destroyed = false;
        if alive {
            let scripts = &mut local.scripts;
            scripts.protection.update(dt);
            if let Some(damage) = &mut scripts.damage {
                damage.update(dt, &mut self.player.camera_local_rotation);
                destroyed = damage.run_coroutines(time) == Some(DamageEvent::Destroyed);
            }
        }
        if destroyed {
            self.destroy_local();
        }
        let local = &mut self.local;
        local.projectiles.end_frame(world_time);
        self.effects.sync_projectiles(
            &local.templates,
            &local.projectiles,
            world_time,
            &mut self.audio,
            &mut self.meshes,
        );
        self.effects
            .update(dt, world_time, &self.world, &mut self.audio);
        for (owner, node, amount) in hits {
            self.send_message_upwards(owner, node, amount, world_time);
        }
        // `ExplosionDamage`: every collider in the sphere gets `ApplyDamage` (only the other
        // players' hit boxes receive it); rigidbodies (none in the maps) are pushed.
        let explosions = std::mem::take(&mut self.effects.events);
        for EffectEvent::Explosion { position, fields } in explosions {
            if fields.is_remote {
                continue;
            }
            let mut damaged = Vec::new();
            for (owner, remote) in self.remotes.iter().enumerate() {
                for hit_box in &remote.hit_boxes {
                    if !remote.prefab.active_in_hierarchy(hit_box.node) {
                        continue;
                    }
                    let collider = physics::DynamicCollider {
                        world: remote.prefab.world[hit_box.node],
                        shape: hit_box.shape,
                        tag: 0,
                    };
                    if collider.overlaps_sphere(position, fields.radius) {
                        let distance =
                            position.distance(collider.closest_point_on_bounds(position));
                        damaged.push((owner, hit_box.node, explosion_damage(&fields, distance)));
                    }
                }
            }
            for (owner, node, amount) in damaged {
                self.send_message_upwards(owner, node, amount, world_time);
            }
        }
        // Unity evaluates animations between Update and LateUpdate.
        self.local.prefab.update_animations();
        for remote in self.remotes.iter_mut().chain(self.mirror.as_mut()) {
            remote.prefab.update_animations();
        }
        for ragdoll in &mut self.ragdolls {
            ragdoll.prefab.update_animations();
        }
        if alive {
            self.player.late_update(input);
        }
        let events = std::mem::take(&mut self.player.events);
        for event in events {
            if let player::PlayerEvent::FallDamage(amount) = event
                && alive
            {
                let scripts = &mut self.local.scripts;
                let protected = scripts.protection.protected;
                let died = scripts
                    .damage
                    .as_mut()
                    .and_then(|d| d.apply_fall_damage(amount, protected, time));
                if let Some(DamageEvent::Died { .. }) = died {
                    self.die(world_time, true);
                }
            }
        }
        self.update_ragdolls(world_time);
        self.room.update(dt);
        self.room_update();
    }

    /// `PhotonNetwork.Destroy` of this client's player object.
    fn destroy_local(&mut self) {
        let scripts = &mut self.local.scripts;
        scripts.destroyed = true;
        if let Some(online) = &mut self.online {
            online.destroy();
        }
        scripts.remove_sources(&mut self.audio);
        self.mirror = None;
    }

    /// `RoomMultiplayerMenu.Update` / `FixedUpdate`: the `EnableHelper` object follows the
    /// menu, and the round timer runs once `WaitBeforeCheck` cleared `delayAction`.
    fn room_update(&mut self) {
        let paused = self.room_menu.as_ref().is_some_and(|m| m.paused);
        self.helper_active = !paused;
        if self.time >= self.round_check_at {
            let now = match &self.online {
                Some(online) => online.photon_time() as f32,
                None => self.time,
            };
            self.round_time = (self.round_duration as f32 - (now - self.reference_time)).max(0.0);
        }
    }

    /// The owner's death prefab: `RagdollController.Start` (with `gui` from `RespawnAfter` and
    /// its camera) counts a death in PlayerPrefs.
    fn add_owner_ragdoll(&mut self, ragdoll: Ragdoll) {
        if ragdoll
            .controller
            .as_ref()
            .is_some_and(|c| c.gui && c.cam.is_some())
        {
            self.settings.borrow_mut().prefs.increment("Deaths");
        }
        self.ragdolls.push(ragdoll);
    }

    /// `RoomMultiplayerMenu.SpawnPlayer(team)` from the menu: `TeamName`, the `EnableHelper`
    /// object on, the player at a spawn point of the team, the room camera off.
    fn spawn_player(&mut self, team: &str) {
        self.props.team = team.to_string();
        self.helper_active = true;
        self.room_camera = false;
        self.respawn();
    }

    /// `RoomMultiplayerMenu.SwapTeams(team)`: `TeamName`, then `PlayerDamage.SwapTeams` sends
    /// `DoSwapTeams` to all; on the owner it instantiates the death prefab with `RespawnAfter`
    /// and destroys the player (the respawn uses the new team).
    fn swap_teams(&mut self, team: &str) {
        self.props.team = team.to_string();
        if let Some(online) = &self.online {
            online.choose_team(team);
            online.swap_teams();
        }
        if let Some(ragdoll) = spawn_ragdoll(
            &self.kinds,
            self.local.kind,
            self.player.position,
            self.player.rotation,
            self.time,
            true,
            &mut self.audio,
        ) {
            self.add_owner_ragdoll(ragdoll);
        }
        self.destroy_local();
    }

    /// The room menu's open state (Tab, Resume, a team button): `Screen.lockCursor` follows.
    fn set_menu_open(&mut self, open: bool) {
        let Some(menu) = &mut self.room_menu else {
            return;
        };
        menu.paused = open;
        self.set_grab(!open);
        if open {
            self.input.fire_held = false;
        }
    }

    /// `Screen.fullScreen` (P): going fullscreen uses the selected resolution's video mode
    /// (borderless where exclusive modes do not exist, as on Wayland).
    fn set_fullscreen(&self, on: bool) {
        if !on {
            self.window.set_fullscreen(None);
            return;
        }
        let wayland = matches!(
            self.window.window_handle().map(|h| h.as_raw()),
            Ok(RawWindowHandle::Wayland(_))
        );
        let size = self
            .room_menu
            .as_ref()
            .and_then(|m| self.resolutions.get(m.resolution_index))
            .copied();
        let mode = self.window.current_monitor().and_then(|monitor| {
            monitor
                .video_modes()
                .filter(|v| Some((v.size().width, v.size().height)) == size)
                .max_by_key(|v| v.refresh_rate_millihertz())
        });
        let fullscreen = match mode {
            Some(mode) if !wayland => Fullscreen::Exclusive(mode),
            _ => Fullscreen::Borderless(None),
        };
        self.window.set_fullscreen(Some(fullscreen));
    }

    /// `PhotonNetwork.playerList` as the menu lists it: this client, then the others.
    fn room_players(&self) -> Vec<PlayerRow> {
        let mut rows = vec![PlayerRow {
            name: self.props.name.clone(),
            team: Some(self.props.team.clone()),
            kills: Some(self.props.kills),
            deaths: Some(self.props.deaths),
            ping: self.online.as_ref().and_then(|o| o.local_ping),
        }];
        if let Some(online) = &self.online {
            for (_, p) in online.players() {
                rows.push(PlayerRow {
                    name: p.name,
                    team: p.team,
                    kills: p.kills,
                    deaths: p.deaths,
                    ping: p.ping,
                });
            }
        }
        for dummy in &self.dummies {
            rows.push(PlayerRow {
                name: dummy.props.name.clone(),
                team: Some(dummy.props.team.clone()),
                kills: Some(dummy.props.kills),
                deaths: Some(dummy.props.deaths),
                ping: None,
            });
        }
        rows
    }

    /// The room menu's `OnGUI` (`draw`) or its mouse handling: the clicked button.
    fn room_gui(&mut self, w: f32, h: f32, draw: bool) -> Option<RoomAction> {
        let players = self.room_players();
        let settings = self.settings.clone();
        let settings = settings.borrow();
        let (Some(menu), Some(font)) = (&mut self.room_menu, &mut self.font) else {
            return None;
        };
        let view = RoomView {
            size: (w, h),
            room_name: &self.room_name,
            game_mode: &self.room.game_mode,
            team_1: &self.room.team_1,
            team_2: &self.room.team_2,
            team1_score: self.room.team1_score,
            team2_score: self.room.team2_score,
            round_time: self.round_time,
            players: &players,
            local_name: &self.props.name,
            local_team: &self.props.team,
            player_exists: !self.local.scripts.destroyed,
            resolutions: &self.resolutions,
            quality_levels: &settings.quality_levels,
            quality: settings.quality,
            sens: settings.sens,
        };
        if draw {
            menu.on_gui(&mut self.gui, font, &view, self.cursor);
            return None;
        }
        let events = room_menu::MouseEvents {
            position: self.cursor,
            down: self.menu_down,
            up: self.menu_up,
            wheel: self.menu_wheel,
        };
        self.menu_down = false;
        self.menu_up = false;
        self.menu_wheel = 0.0;
        let action = menu.input(&mut self.gui, font, &view, events);
        // Layout measured text only: nothing of it is drawn.
        self.gui.quads.clear();
        action
    }

    /// What a room menu button does. Returns true for "Leave Room".
    fn room_action(&mut self, action: RoomAction) -> bool {
        match action {
            RoomAction::Resolution(index) => {
                if let Some(menu) = &mut self.room_menu {
                    menu.resolution_index = index;
                }
                if self.window.fullscreen().is_some() {
                    self.set_fullscreen(true);
                }
            }
            RoomAction::Quality(index) => {
                let limit = {
                    let mut settings = self.settings.borrow_mut();
                    settings.quality = index;
                    settings.texture_limit()
                };
                self.renderer.set_texture_limit(&self.gl, limit);
            }
            RoomAction::Team(team) => {
                let team = team_name(&self.room, team);
                if self.local.scripts.destroyed {
                    self.spawn_player(&team);
                } else {
                    self.swap_teams(&team);
                }
                self.set_menu_open(false);
                // WhoKilledWho.PlayerJoinedTeam.
                let joined = if self.room.game_mode == "TDM" {
                    format!("joined {team}")
                } else {
                    "joined battle".to_string()
                };
                let name = self.props.name.clone();
                self.room.add_message(&name, "", &joined, "");
                if let Some(online) = &self.online {
                    online.send_message(&name, "", &joined, "");
                }
            }
            RoomAction::Resume => self.set_menu_open(false),
            RoomAction::Leave => return true,
            RoomAction::SensitivityDown | RoomAction::SensitivityUp => {
                let up = action == RoomAction::SensitivityUp;
                let mut settings = self.settings.borrow_mut();
                if let Some(sens) = room_menu::step_sensitivity(settings.sens, up) {
                    settings.sens = sens;
                    settings.prefs.set_float("sens", sens);
                }
            }
            RoomAction::PlayerList | RoomAction::Controls => {
                if let Some(menu) = &mut self.room_menu {
                    menu.select_tab(&action);
                }
            }
        }
        false
    }

    /// `SendMessageUpwards("ApplyDamage", amount)` from a node of another player's copy: every
    /// `HitBox` on it and its ancestors runs `TotalDamage`, whose `DoDamage` RPC also reaches
    /// that player's owner.
    fn send_message_upwards(&mut self, owner: usize, node: usize, amount: f32, time: f32) {
        let Some(remote) = self.remotes.get_mut(owner) else {
            return;
        };
        let actor = remote.actor;
        let mut deaths = Vec::new();
        let mut sent = Vec::new();
        for hit_box in remote.hit_box_chain(node) {
            let local = self.props.actor;
            let (total, death) = remote.apply_damage(hit_box, amount, local, local);
            sent.extend(total);
            deaths.extend(death);
        }
        for death in deaths {
            self.remote_died(owner, death, time);
        }
        for total in sent {
            if let Some(online) = &self.online {
                online.do_damage(actor, total);
            }
            if let Some(index) = self.dummies.iter().position(|d| d.props.actor == actor) {
                let local = self.props.actor;
                let events = self.dummies[index].receive_do_damage(total, local, time);
                self.owner_events(index, events, time);
            }
        }
    }

    /// `DoDamage` killed another player's copy: its death prefab (camera cleared) and, when this
    /// client's player was the damager, `AddKillNotification`.
    fn remote_died(&mut self, owner: usize, death: RemoteDeath, time: f32) {
        let kind = self.remotes[owner].kind;
        if let Some(ragdoll) = spawn_ragdoll(
            &self.kinds,
            kind,
            death.position,
            death.rotation,
            time,
            false,
            &mut self.audio,
        ) {
            self.ragdolls.push(ragdoll);
        }
        if death.killed_by_local {
            self.room.kill_notification(&mut self.props, &death.name);
            // AddKillNotification: PlayerPrefs "Kills" + 1.
            self.settings.borrow_mut().prefs.increment("Kills");
            if let Some(online) = &mut self.online {
                // AddKillNotification: the feed message to all, Kills + 1, the team's score.
                online.send_message(&self.props.name, &death.name, "killed", &self.props.team);
                online.set_score("Kills", self.props.kills);
                if self.props.team == self.room.team_1.name {
                    online.set_team_score("Team1Score", self.room.team1_score);
                } else if self.props.team == self.room.team_2.name {
                    online.set_team_score("Team2Score", self.room.team2_score);
                }
            }
        }
    }

    /// The simulated clients' frames and what they send, then what the servers delivered.
    fn network_frame(&mut self, dt: f32, time: f32) {
        let incoming = match &mut self.online {
            Some(online) => online.update(time),
            None => Vec::new(),
        };
        for event in incoming {
            if std::env::var_os("PGA3_NET_DEBUG").is_some() {
                eprintln!("net: {event:?}");
            }
            self.net_event(event, time);
        }
        for index in 0..self.dummies.len() {
            let (events, respawn) =
                self.dummies[index].update(&self.world, &self.scene.ladders, dt, time);
            self.owner_events(index, events, time);
            if respawn {
                self.spawn_dummy(index);
            }
        }
    }

    /// `SpawnPlayer` on a dummy's client.
    fn spawn_dummy(&mut self, index: usize) {
        let team = self.dummies[index].props.team.clone();
        let (point, team_1) = self.room.spawn_point(&team);
        let point = self.dummy_spawn.or(point).unwrap_or(0);
        let kind = if team_1 { 0 } else { 1 };
        let Some(spawn) = self.scene.spawn_points.get(point) else {
            return;
        };
        let (position, rotation) = (spawn.position, spawn.rotation);
        let player = match Player::from_prefab(&self.assets, PLAYER_KINDS[kind]) {
            Ok(player) => player,
            Err(error) => {
                eprintln!("dummy: {error:#}");
                return;
            }
        };
        let ragdoll = self.kinds[kind].ragdoll.as_ref().and_then(|(prefab, _)| {
            let (_, c) = prefab
                .find_scripts("RagdollController")
                .into_iter()
                .next()?;
            Some(RagdollController::new(&c["fields"], None))
        });
        let weapon = dummy_weapon(&self.kinds[kind].pristine, self.dummy_weapon);
        let events = self.dummies[index].spawn(
            player,
            &self.kinds[kind].pristine,
            &mut self.clips,
            ragdoll,
            weapon,
            position,
            rotation,
            team_1,
            self.time,
        );
        println!(
            "dummy {} ({}, {}) at spawn point {point}",
            self.dummies[index].props.name, team, PLAYER_KINDS[kind]
        );
        self.owner_events(index, events, self.time);
    }

    /// Delivers what dummy `index`'s client sent.
    fn owner_events(&mut self, index: usize, events: Vec<OwnerEvent>, time: f32) {
        let actor = self.dummies[index].props.actor;
        for event in events {
            let remote = self.remotes.iter().position(|r| r.actor == actor);
            match event {
                OwnerEvent::Instantiate {
                    position,
                    rotation,
                    team_1_prefab,
                } => {
                    if let Some(old) = remote {
                        self.remove_remote(old);
                    }
                    let kind = if team_1_prefab { 0 } else { 1 };
                    self.remotes.push(RemotePlayer::new(
                        &self.kinds[kind].pristine,
                        kind,
                        actor,
                        position,
                        rotation,
                        Some(&mut self.audio),
                    ));
                }
                OwnerEvent::Serialize(state) => {
                    if let Some(r) = remote {
                        self.remotes[r].receive(&state);
                    }
                }
                OwnerEvent::SyncWeapons(sync) => {
                    if let Some(r) = remote {
                        let events = self.remotes[r].sync_weapons_rpc(sync, time, r);
                        self.sync_events(events);
                    }
                }
                OwnerEvent::DoDamage(amount) => {
                    // An RPC to all: the sender runs it at once, then this client's copy.
                    let more = self.dummies[index].receive_do_damage(amount, actor, time);
                    if let Some(r) = remote
                        && let Some(death) =
                            self.remotes[r].do_damage(amount, actor, self.props.actor)
                    {
                        self.remote_died(r, death, time);
                    }
                    self.owner_events(index, more, time);
                }
                OwnerEvent::Message {
                    killer,
                    killed,
                    middle,
                    team,
                } => self.room.add_message(&killer, &killed, &middle, &team),
                OwnerEvent::Destroy => {
                    if let Some(r) = remote {
                        self.remove_remote(r);
                    }
                }
            }
        }
    }

    /// What a copy's `WeaponSync` did: remote shots and sounds.
    fn sync_events(&mut self, events: Vec<SyncEvent>) {
        let local = &mut self.local;
        for event in events {
            match event {
                SyncEvent::Spawn {
                    prefab,
                    position,
                    rotation,
                    velocity,
                    flags,
                } => match local.templates.find(&prefab) {
                    Some(template) => local.projectiles.spawn_with(
                        &local.templates,
                        template,
                        position,
                        rotation,
                        velocity,
                        flags,
                    ),
                    None => eprintln!("remote weapon spawned unknown prefab {prefab}"),
                },
                SyncEvent::Sound { source, clip } => self.audio.play(source, &clip),
            }
        }
    }

    /// `SpawnPlayer` on the network: `TeamName`, then `PhotonNetwork.Instantiate` of the team's
    /// player prefab at the spawn point.
    fn instantiate_online(&mut self) {
        let team_1 = self.local.kind == 0;
        let (position, rotation) = (self.player.position, self.player.rotation);
        if let Some(online) = &mut self.online {
            online.choose_team(&self.props.team);
            online.spawn(team_1, position, rotation);
        }
    }

    /// What the original servers delivered: the other players' objects, RPCs and scores.
    fn net_event(&mut self, event: NetIn, time: f32) {
        let remote = |remotes: &[RemotePlayer], actor: ActorId| {
            remotes.iter().position(|r| r.actor == actor)
        };
        match event {
            NetIn::Instantiate {
                actor,
                team_1_prefab,
                position,
                rotation,
            } => {
                if let Some(old) = remote(&self.remotes, actor) {
                    self.remove_remote(old);
                }
                let kind = if team_1_prefab { 0 } else { 1 };
                self.remotes.push(RemotePlayer::new(
                    &self.kinds[kind].pristine,
                    kind,
                    actor,
                    position,
                    rotation,
                    Some(&mut self.audio),
                ));
            }
            NetIn::State { actor, state } => {
                if let Some(r) = remote(&self.remotes, actor) {
                    self.remotes[r].receive(&state);
                }
            }
            NetIn::SyncWeapons { actor, sync } => {
                if let Some(r) = remote(&self.remotes, actor) {
                    let events = self.remotes[r].sync_weapons_rpc(sync, time, r);
                    self.sync_events(events);
                }
            }
            NetIn::DoDamage {
                target,
                amount,
                damager,
            } => {
                if target == self.props.actor {
                    self.local_do_damage(amount, damager, time);
                } else if let Some(r) = remote(&self.remotes, target)
                    && let Some(death) =
                        self.remotes[r].do_damage(amount, damager, self.props.actor)
                {
                    self.remote_died(r, death, time);
                }
            }
            NetIn::Message {
                killer,
                killed,
                middle,
                team,
            } => self.room.add_message(&killer, &killed, &middle, &team),
            NetIn::Destroy { actor } => {
                if let Some(r) = remote(&self.remotes, actor) {
                    self.remove_remote(r);
                }
            }
            NetIn::RoomScores { team1, team2 } => {
                if let Some(score) = team1 {
                    self.room.team1_score = score;
                }
                if let Some(score) = team2 {
                    self.room.team2_score = score;
                }
            }
            NetIn::SwapTeams { actor } => {
                // DoSwapTeams on another player's copy: its death prefab, camera cleared.
                if let Some(r) = remote(&self.remotes, actor) {
                    let kind = self.remotes[r].kind;
                    let (_, rotation, position) =
                        self.remotes[r].prefab.world[0].to_scale_rotation_translation();
                    if let Some(ragdoll) = spawn_ragdoll(
                        &self.kinds,
                        kind,
                        position,
                        rotation,
                        time,
                        false,
                        &mut self.audio,
                    ) {
                        self.ragdolls.push(ragdoll);
                    }
                }
            }
            NetIn::Disconnected(reason) => {
                eprintln!("online: disconnected: {reason}");
                self.online = None;
            }
        }
    }

    /// `PlayerDamage.DoDamage` on this client's player, sent by another client's hit.
    fn local_do_damage(&mut self, amount: f32, damager: ActorId, time: f32) {
        if self.local.scripts.destroyed {
            return;
        }
        let player_time = self.player.time;
        let scripts = &mut self.local.scripts;
        let protected = scripts.protection.protected;
        let died = scripts
            .damage
            .as_mut()
            .and_then(|d| d.do_damage(amount, protected, player_time));
        if let Some(DamageEvent::Died { .. }) = died {
            self.die(time, damager == self.props.actor);
        }
    }

    /// `PhotonNetwork.Destroy` of another player's object (its AudioSources go with it).
    fn remove_remote(&mut self, index: usize) {
        let remote = self.remotes.remove(index);
        for id in remote.sources() {
            self.audio.remove(id);
        }
    }

    /// `DoDamage` at `currentHp < 0` on the owner: the player's children are deactivated, the
    /// death prefab is instantiated at its transform and sent `RespawnAfter`, Deaths + 1, and
    /// (the damage being its own fall) `PlayerFellDown`.
    fn die(&mut self, time: f32, fell: bool) {
        if std::env::var_os("PGA3_NET_DEBUG").is_some() {
            eprintln!("net: local player died (fell: {fell}) at {time:.2}");
        }
        let children = self.local.prefab.nodes[0].children.clone();
        for child in children {
            self.local.prefab.set_active(child, false);
        }
        if let Some(ragdoll) = spawn_ragdoll(
            &self.kinds,
            self.local.kind,
            self.player.position,
            self.player.rotation,
            time,
            true,
            &mut self.audio,
        ) {
            self.add_owner_ragdoll(ragdoll);
        }
        self.props.deaths += 1;
        if let Some(online) = &self.online {
            online.set_score("Deaths", self.props.deaths);
        }
        if fell {
            let name = self.props.name.clone();
            self.room.add_message(&name, "", "fell down", "");
            if let Some(online) = &self.online {
                online.send_message(&name, "", "fell down", "");
            }
        }
    }

    /// The death prefabs' `InvokeRepeating` countdown (the owner's `SpawnPlayer` respawns) and
    /// their `WaitForDestroy`.
    fn update_ragdolls(&mut self, time: f32) {
        let mut spawn = false;
        let mut index = 0;
        while index < self.ragdolls.len() {
            let ragdoll = &mut self.ragdolls[index];
            spawn |= ragdoll.controller.as_mut().is_some_and(|c| c.update(time));
            if time >= ragdoll.destroy_at {
                for &id in &ragdoll.sources {
                    self.audio.remove(id);
                }
                self.ragdolls.remove(index);
            } else {
                index += 1;
            }
        }
        if spawn {
            self.respawn();
        }
    }

    /// `RoomMultiplayerMenu.SpawnPlayer(team)`: a new player prefab of the team at one of its
    /// spawn points.
    fn respawn(&mut self) {
        self.local.scripts.remove_sources(&mut self.audio);
        let (point, team_1) = self.room.spawn_point(&self.props.team);
        let kind = if team_1 { 0 } else { 1 };
        match Player::from_prefab(&self.assets, PLAYER_KINDS[kind]) {
            Ok(player) => self.player = player,
            Err(error) => eprintln!("respawn: {error:#}"),
        }
        self.spawn_index = point.unwrap_or(0);
        place_at_spawn(
            &mut self.player,
            &mut self.camera,
            &self.scene,
            self.spawn_index,
        );
        let templates = std::mem::take(&mut self.local.templates);
        let template_models = std::mem::take(&mut self.local.template_models);
        let projectiles = std::mem::take(&mut self.local.projectiles);
        self.local = instantiate_local(
            &self.kinds,
            kind,
            templates,
            template_models,
            &self.player,
            &mut self.audio,
            &mut self.meshes,
            &mut self.clips,
        );
        self.local.projectiles = projectiles;
        self.sync_local_avatar();
        self.create_mirror();
        self.instantiate_online();
    }

    /// The local prefab's transforms from the player state.
    fn sync_local_avatar(&mut self) {
        let local = &mut self.local;
        weapons::sync_avatar(
            &mut local.prefab,
            &self.player,
            local.main_camera,
            local.weapon_manager,
        );
        local.prefab.update_world();
    }

    /// The mirror copy of a freshly spawned local player, with its first serialized state.
    fn create_mirror(&mut self) {
        let kind = self.local.kind;
        let mut mirror = RemotePlayer::new(
            &self.kinds[kind].pristine,
            kind,
            self.props.actor,
            self.player.position,
            self.player.rotation,
            None,
        );
        mirror.receive(&self.local_state());
        self.mirror = Some(mirror);
        self.serialize_at = self.time + SERIALIZE_INTERVAL;
    }

    /// The local player's `OnPhotonSerializeView` (write).
    fn local_state(&self) -> remote::NetState {
        room::owner_state(
            &self.player,
            &self.kinds[self.local.kind].look_target_local,
            self.local.scripts.character.as_ref(),
            &self.props,
            self.local.scripts.protection.protected,
        )
    }

    fn update_fly(&mut self, dt: f32) {
        const SENSITIVITY: f32 = 0.15;
        let (dx, dy) = std::mem::take(&mut self.input.mouse_delta);
        self.camera.yaw += dx as f32 * SENSITIVITY;
        self.camera.pitch = (self.camera.pitch + dy as f32 * SENSITIVITY).clamp(-89.0, 89.0);

        let rotation = self.camera.rotation();
        let i = &self.input;
        let axis = |positive: bool, negative: bool| (positive as i32 - negative as i32) as f32;
        let wish = rotation * Vec3::new(axis(i.right, i.left), 0.0, axis(i.forward, i.back))
            + Vec3::Y * axis(i.up, i.down);
        let speed = if i.fast || i.run { 30.0 } else { 8.0 };
        self.camera.position += wish.normalize_or_zero() * speed * dt;
        // The player keeps being simulated without input, so the body seen from the fly camera
        // stands (and animates) where it was left.
        self.step_player(
            dt.min(MAXIMUM_DELTA_TIME),
            &FrameInput::default(),
            WeaponInput::default(),
        );
    }

    /// `CharacterAnimation.LateUpdate`, the local player's serialization, then the transforms
    /// rendering uses.
    fn late_animation_update(&mut self) {
        let p = &self.player;
        // In fly mode the movement keys drive the camera, not the player.
        let keys = matches!(self.mode, Mode::Walk);
        let i = &self.input;
        let input = CharacterInput {
            crouch: p.crouch,
            prone: p.prone,
            grounded: p.grounded,
            walking: p.walking,
            running: p.running,
            key_w: keys && i.key_w,
            key_a: keys && i.key_a,
            key_s: keys && i.key_s,
            key_d: keys && i.key_d,
            key_r: keys && i.key_r,
        };
        let local = &mut self.local;
        if !local.scripts.destroyed
            && let Some(character) = &mut local.scripts.character
        {
            let selected = local
                .weapons
                .as_ref()
                .map_or(SelectedWeapon::default(), |w| SelectedWeapon {
                    node: w.selected_weapon,
                    name: w.selected_script().map_or("", |s| s.name.as_str()),
                });
            character.late_update(&mut local.prefab, &input, selected);
        }
        if !self.local.scripts.destroyed && self.time >= self.serialize_at {
            self.serialize_at = self.time + SERIALIZE_INTERVAL;
            let state = self.local_state();
            if let Some(online) = &mut self.online {
                online.send_state(&state);
            }
            if let Some(mirror) = &mut self.mirror {
                mirror.receive(&state);
            }
        }
        self.sync_local_avatar();
        for remote in self.remotes.iter_mut().chain(self.mirror.as_mut()) {
            remote.prefab.update_world();
        }
        for remote in &self.remotes {
            remote.update_sources(&mut self.audio);
        }
        for ragdoll in &mut self.ragdolls {
            ragdoll.prefab.update_world();
        }
        self.update_avatar_systems(dt_frame_of(&self.mode, self.last_dt));
        self.update_listener();
    }

    /// The player prefab's particle systems (simulated while their GameObject is active) and its
    /// AudioSources (positions; a deactivated weapon's source stops).
    fn update_avatar_systems(&mut self, dt: f32) {
        let local = &mut self.local;
        if local.scripts.destroyed {
            return;
        }
        for (node, system) in &mut local.scripts.systems {
            if local.prefab.active_in_hierarchy(*node) {
                system.simulate(dt, local.prefab.world[*node], Some(&self.world));
            }
        }
        let scripts = &local.scripts;
        if let Some(id) = scripts.root_source {
            self.audio
                .set_position(id, local.prefab.world[0].w_axis.truncate());
        }
        for &(id, node) in scripts
            .manager_source
            .iter()
            .chain(scripts.weapon_sources.iter().flatten())
        {
            if local.prefab.active_in_hierarchy(node) {
                self.audio
                    .set_position(id, local.prefab.world[node].w_axis.truncate());
            } else {
                self.audio.stop(id);
            }
        }
    }

    /// The camera rendering the frame: the player's Main Camera, the local death prefab's camera
    /// while it exists, or the fly camera.
    fn main_view(&self) -> View {
        match self.mode {
            Mode::Walk => {
                for ragdoll in &self.ragdolls {
                    if let Some(cam) = ragdoll.controller.as_ref().and_then(|c| c.cam)
                        && let Some(camera) = ragdoll.prefab.camera(cam)
                    {
                        let (_, rotation, position) =
                            ragdoll.prefab.world[cam].to_scale_rotation_translation();
                        return View {
                            position,
                            rotation,
                            fov_y_degrees: camera.fov_y_degrees,
                            near: camera.near,
                            far: camera.far,
                        };
                    }
                }
                if self.room_camera
                    && let Some(c) = &self.scene.camera
                {
                    return View {
                        position: c.position,
                        rotation: c.rotation,
                        fov_y_degrees: c.fov_y_degrees,
                        near: c.near,
                        far: c.far,
                    };
                }
                View {
                    position: self.player.camera_position(),
                    rotation: self.player.camera_rotation(),
                    fov_y_degrees: self.player.camera_fov,
                    near: self.player.camera_near,
                    far: self.player.camera_far,
                }
            }
            Mode::Fly => View {
                position: self.camera.position,
                rotation: self.camera.rotation(),
                fov_y_degrees: self.fov.0,
                near: self.fov.1,
                far: self.fov.2,
            },
        }
    }

    /// The AudioListener follows the rendering camera.
    fn update_listener(&mut self) {
        let view = self.main_view();
        self.audio.update(view.position, view.rotation);
    }

    fn render(&mut self) {
        let size = self.window.inner_size();
        let view = self.main_view();
        let (w, h) = (size.width, size.height);
        self.renderer.draw(&self.gl, &view, w, h);
        // Spawned bullets and rockets (Default layer, seen by the Main Camera and the fly
        // camera).
        let local = &self.local;
        for (template, root) in local.projectiles.instances(&local.templates) {
            self.renderer.draw_models(
                &self.gl,
                &view,
                w,
                h,
                &local.templates.list[template].prefab,
                &local.template_models[template],
                u32::MAX,
                false,
                Some(root),
            );
        }
        for ragdoll in &self.ragdolls {
            if let Some((_, models)) = &self.kinds[ragdoll.kind].ragdoll {
                self.renderer.draw_models(
                    &self.gl,
                    &view,
                    w,
                    h,
                    &ragdoll.prefab,
                    models,
                    u32::MAX,
                    false,
                    None,
                );
            }
        }
        let alive = !self.local.scripts.destroyed;
        let main_mask = match self.mode {
            Mode::Walk => {
                let local = &self.local;
                local
                    .main_camera
                    .and_then(|n| local.prefab.camera(n))
                    .map_or(u32::MAX, |c| c.culling_mask)
            }
            Mode::Fly => u32::MAX,
        };
        for remote in &self.remotes {
            self.renderer.draw_models(
                &self.gl,
                &view,
                w,
                h,
                &remote.prefab,
                &self.kinds[remote.kind].models,
                main_mask,
                false,
                None,
            );
        }
        match self.mode {
            Mode::Walk => {
                let local = &self.local;
                if alive {
                    self.renderer.draw_models(
                        &self.gl,
                        &view,
                        w,
                        h,
                        &local.prefab,
                        &self.kinds[local.kind].models,
                        main_mask,
                        false,
                        None,
                    );
                }
                let batches = self.effects.batches(&view);
                self.renderer
                    .draw_dynamic(&self.gl, &view, w, h, &batches, false);
                // Weapon Camera (depth 2, clear flags Depth only, culling mask = layer 9).
                let local = &self.local;
                if let Some(node) = local.weapon_camera
                    && alive
                    && local.prefab.active_in_hierarchy(node)
                    && let Some(camera) = local.prefab.camera(node)
                {
                    let (_, rotation, position) =
                        local.prefab.world[node].to_scale_rotation_translation();
                    let weapon_view = View {
                        position,
                        rotation,
                        fov_y_degrees: camera.fov_y_degrees,
                        near: camera.near,
                        far: camera.far,
                    };
                    self.renderer.draw_models(
                        &self.gl,
                        &weapon_view,
                        w,
                        h,
                        &local.prefab,
                        &self.kinds[local.kind].models,
                        camera.culling_mask,
                        camera.clear_flags == 3,
                        None,
                    );
                    let batches: Vec<_> = local
                        .scripts
                        .systems
                        .iter()
                        .filter(|(n, _)| local.prefab.active_in_hierarchy(*n))
                        .filter_map(|(n, system)| {
                            let layer = local.prefab.nodes[*n].layer;
                            let weapon_layer = camera.culling_mask & (1 << layer) != 0;
                            system.geometry(local.prefab.world[*n], &weapon_view, weapon_layer)
                        })
                        .collect();
                    self.renderer
                        .draw_dynamic(&self.gl, &weapon_view, w, h, &batches, true);
                }
                self.draw_hud(w as f32, h as f32, &view);
                self.room_gui(w as f32, h as f32, true);
                self.renderer
                    .draw_gui(&self.gl, &self.gui.quads, self.font.as_ref(), w, h);
                self.gui.quads.clear();
            }
            Mode::Fly => {
                if let Some(mirror) = &self.mirror {
                    self.renderer.draw_models(
                        &self.gl,
                        &view,
                        w,
                        h,
                        &mirror.prefab,
                        &self.kinds[mirror.kind].models,
                        u32::MAX,
                        false,
                        None,
                    );
                }
                let batches = self.effects.batches(&view);
                self.renderer
                    .draw_dynamic(&self.gl, &view, w, h, &batches, false);
            }
        }
    }

    /// Every `OnGUI` of the frame, lowest `GUI.depth` last: `PlayerDamage` (10), then
    /// `DrawPlayerName` (2), then the scripts at the default depth 0.
    fn draw_hud(&mut self, w: f32, h: f32, view: &View) {
        let Some(font) = &mut self.font else {
            return;
        };
        let gui = &mut self.gui;
        let local = &mut self.local;
        let dt = self.last_dt;
        if !local.scripts.destroyed
            && let (Some(damage), Some(skin)) = (&local.scripts.damage, &self.fps_skin)
        {
            damage.on_gui(gui, font, skin, w, h);
        }
        if let Some(skin) = &self.fps_skin {
            for remote in &self.remotes {
                remote.draw_name_gui(gui, font, &skin.label, view, w, h);
            }
        }
        for ragdoll in &self.ragdolls {
            if let (Some(controller), Some(skin)) = (&ragdoll.controller, &self.ragdoll_skin) {
                controller.on_gui(gui, font, skin, w, h);
            }
        }
        if !local.scripts.destroyed {
            let manager_active = hud::player_gui_active(&local.prefab, local.weapon_manager);
            if let Some(weapons) = &mut local.weapons
                && manager_active
            {
                // SniperScope and WeaponCrosshair.
                let quads = weapons.on_gui(&mut local.prefab, w, h);
                gui.quads.extend(quads);
                if let (Some(ammo), Some(skin)) = (&mut local.scripts.ammo, &self.fps_skin) {
                    ammo.on_gui(gui, font, skin, weapons, dt, w, h);
                }
                if let Some(default) = &mut self.default_skin {
                    default.weapon_manager_gui(gui, font, weapons, w);
                }
            }
            if local.scripts.protection.protected
                && let Some(default) = &self.default_skin
            {
                default.spawn_protection_gui(gui, font, w, h);
            }
        }
        for remote in &self.remotes {
            remote.hit_marker_gui(gui, w, h);
        }
        if let Some(skin) = &self.feed_skin {
            self.room.feed_gui(gui, font, skin, w);
        }
    }
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }

    /// `LoadMap` finished: starts the game the menu chose.
    fn launch(&mut self, event_loop: &ActiveEventLoop, launch: menu::Launch) {
        self.args.scene = launch.map;
        if launch.online.is_some() || launch.mode == "TDM" || launch.mode == "DM" {
            self.args.mode = launch.mode;
        }
        self.args.name = launch.name;
        self.args.region = launch.region;
        let Some(platform) = &self.platform else {
            return;
        };
        let start = GameStart {
            from_menu: true,
            room_name: launch.room_name,
            round_duration: launch.round_duration,
        };
        match create(platform, &self.args, launch.online, &self.settings, &start) {
            Ok(mut running) => {
                running.auto_team = self.args.menu_auto.then_some(self.args.team);
                let menu = self.screen.take();
                if let Some(Screen::Menu(menu)) = menu {
                    self.stash = Some(menu);
                }
                self.screen = Some(Screen::Game(Box::new(running)));
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    /// "Leave Room": the game ends and the lobby comes back (`OnLeftRoom` loads `Main`); a game
    /// started without the menu ends the program.
    fn leave_room(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(Screen::Game(game)) = &mut self.screen {
            game.set_grab(false);
        }
        self.screen = None;
        let Some(mut menu) = self.stash.take() else {
            event_loop.exit();
            return;
        };
        if let Some(platform) = &self.platform {
            let limit = self.settings.borrow().texture_limit();
            menu.renderer.set_texture_limit(&platform.gl, limit);
        }
        menu.menu.return_from_game(&self.args.region);
        menu.last_frame = Instant::now();
        self.screen = Some(Screen::Menu(menu));
    }

    fn menu_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let Some(Screen::Menu(screen)) = self.screen.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                screen.mouse = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    screen.input.down = true;
                } else {
                    screen.input.up = true;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                screen.input.wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Backspace) => screen.input.backspace = true,
                    PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter) => {
                        screen.input.enter = true
                    }
                    PhysicalKey::Code(KeyCode::Tab) => screen.input.tab = true,
                    PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                    _ => {}
                }
                if let Some(text) = &event.text {
                    screen.input.chars.extend(text.chars());
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(platform) = &self.platform else {
                    return;
                };
                let now = Instant::now();
                let dt = (now - screen.last_frame).as_secs_f32().min(0.1);
                screen.last_frame = now;
                let size = platform.window.inner_size();
                let size_f = (size.width as f32, size.height as f32);
                screen.input.mouse = screen.mouse;
                let mut launch = screen.menu.update(dt, &screen.input, size_f);
                if launch.is_none() {
                    launch = screen.menu.offline_launch();
                }
                screen.input = menu::MenuInput::default();
                let (clear, quads) = screen.menu.frame(size_f);
                gfx::clear(&platform.gl, clear, size.width, size.height);
                screen.renderer.draw_gui(
                    &platform.gl,
                    &quads,
                    Some(&screen.menu.font),
                    size.width,
                    size.height,
                );
                screen.frames += 1;
                // `--menu-auto` runs take the screenshot in the room (`--wait` from its start).
                let ready = screen.frames >= 3
                    && screen.started.elapsed().as_secs_f32() >= self.args.wait
                    && !self.args.menu_auto;
                if let (Some(path), true) = (&self.args.screenshot, ready) {
                    save_screenshot(platform, path);
                    event_loop.exit();
                }
                platform.window.pre_present_notify();
                if let Err(error) = platform.surface.swap_buffers(&platform.context) {
                    eprintln!("swap_buffers: {error}");
                }
                if let Some(launch) = launch {
                    self.launch(event_loop, launch);
                }
            }
            _ => {}
        }
    }

    fn game_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let Some(Screen::Game(app)) = self.screen.as_mut() else {
            return;
        };
        let Some(platform) = &self.platform else {
            return;
        };
        let mut leave = false;
        match event {
            WindowEvent::Focused(false) => {
                app.set_grab(false);
                app.input.fire_held = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                app.cursor = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                let menu_open = app.room_menu.as_ref().is_some_and(|m| m.paused);
                if menu_open {
                    if button == MouseButton::Left {
                        if pressed {
                            app.menu_down = true;
                        } else {
                            app.menu_up = true;
                        }
                    }
                } else {
                    let i = &mut app.input;
                    if !i.grabbed {
                        // The first click only captures the cursor.
                        if pressed && button == MouseButton::Left {
                            app.set_grab(true);
                        }
                    } else {
                        match button {
                            MouseButton::Left => {
                                i.fire_held = pressed;
                                i.fire_pressed |= pressed;
                            }
                            MouseButton::Right => i.aim_pressed |= pressed,
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. }
                if app.room_menu.as_ref().is_some_and(|m| m.paused) =>
            {
                app.menu_wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
            }
            WindowEvent::MouseWheel { delta, .. } if app.input.grabbed => {
                // Unity reports one wheel notch as 0.1 on the "Mouse ScrollWheel" axis; only the
                // sign matters to WeaponManager.
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                app.input.scroll += lines * 0.1;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                let down = pressed && !repeat;
                if app.room_keys && down {
                    match code {
                        // RoomMultiplayerMenu.Update: Tab toggles the menu, P fullscreen.
                        KeyCode::Tab => {
                            let open = app.room_menu.as_ref().is_some_and(|m| !m.paused);
                            app.set_menu_open(open);
                            return;
                        }
                        KeyCode::KeyP => {
                            app.set_fullscreen(app.window.fullscreen().is_none());
                            return;
                        }
                        _ => {}
                    }
                }
                let i = &mut app.input;
                match code {
                    KeyCode::KeyW => i.key_w = pressed,
                    KeyCode::KeyS => i.key_s = pressed,
                    KeyCode::KeyA => i.key_a = pressed,
                    KeyCode::KeyD => i.key_d = pressed,
                    KeyCode::KeyR => {
                        i.key_r = pressed;
                        i.reload_pressed |= down;
                    }
                    KeyCode::KeyM => i.key_m = pressed,
                    KeyCode::KeyN => i.key_n = pressed,
                    KeyCode::Digit0 => i.number_pressed[0] |= down,
                    KeyCode::Digit1 => i.number_pressed[1] |= down,
                    KeyCode::Digit2 => i.number_pressed[2] |= down,
                    KeyCode::Digit3 => i.number_pressed[3] |= down,
                    KeyCode::Digit4 => i.number_pressed[4] |= down,
                    KeyCode::Digit5 => i.number_pressed[5] |= down,
                    KeyCode::Digit6 => i.number_pressed[6] |= down,
                    KeyCode::Digit7 => i.number_pressed[7] |= down,
                    KeyCode::Digit8 => i.number_pressed[8] |= down,
                    KeyCode::Digit9 => i.number_pressed[9] |= down,
                    _ => {}
                }
                match code {
                    KeyCode::KeyW | KeyCode::ArrowUp => i.forward = pressed,
                    KeyCode::KeyS | KeyCode::ArrowDown => i.back = pressed,
                    KeyCode::KeyA | KeyCode::ArrowLeft => i.left = pressed,
                    KeyCode::KeyD | KeyCode::ArrowRight => i.right = pressed,
                    KeyCode::Space => {
                        i.up = pressed;
                        i.jump_pressed |= down;
                    }
                    KeyCode::KeyC => {
                        i.down = pressed;
                        i.crouch_pressed |= down;
                    }
                    KeyCode::ControlLeft => i.prone_pressed |= down,
                    KeyCode::ShiftLeft => i.run = pressed,
                    KeyCode::ShiftRight => i.fast = pressed,
                    KeyCode::KeyF if down && !app.room_keys => {
                        app.mode = match app.mode {
                            Mode::Walk => Mode::Fly,
                            Mode::Fly => {
                                drop_player_at_camera(&mut app.player, &app.camera);
                                Mode::Walk
                            }
                        };
                    }
                    KeyCode::Tab if down && !app.room_keys => {
                        app.spawn_index += 1;
                        place_at_spawn(
                            &mut app.player,
                            &mut app.camera,
                            &app.scene,
                            app.spawn_index,
                        );
                    }
                    KeyCode::KeyP if down && !app.room_keys => {
                        let c = &app.camera;
                        println!(
                            "--camera {},{},{},{},{}",
                            c.position.x, c.position.y, c.position.z, c.yaw, c.pitch
                        );
                    }
                    KeyCode::Escape if pressed => {
                        if app.input.grabbed {
                            app.set_grab(false);
                        } else if !app.from_menu {
                            event_loop.exit();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - app.last_frame).as_secs_f32().min(0.1);
                app.last_frame = now;
                if app.frames == 0
                    && let Some(key) = self.args.weapon
                {
                    app.input.number_pressed[key] = true;
                }
                if self.args.aim && !app.aimed && app.started.elapsed().as_secs_f32() >= 0.6 {
                    app.aimed = true;
                    app.input.aim_pressed = true;
                }
                if self.args.fire {
                    app.input.fire_held = true;
                    // Single-fire weapons need clicks.
                    app.input.fire_pressed |= app.frames % 30 == 0;
                }
                let size = app.window.inner_size();
                if let Some(action) = app.room_gui(size.width as f32, size.height as f32, false) {
                    leave |= app.room_action(action);
                }
                if app.started.elapsed().as_secs_f32() >= 1.0
                    && let Some(team) = app.auto_team.take()
                {
                    leave |= app.room_action(RoomAction::Team(team));
                }
                app.update(dt);
                app.render();
                app.frames += 1;
                if self.args.stats {
                    app.stats_frames += 1;
                    app.stats_max = app.stats_max.max(dt);
                    app.stats_cpu += now.elapsed().as_secs_f32();
                    let elapsed = app.stats_start.elapsed().as_secs_f32();
                    if elapsed >= 1.0 {
                        let n = app.stats_frames as f32;
                        println!(
                            "{:.0} fps, frame {:.2} ms avg / {:.2} ms max, update+render CPU {:.2} ms",
                            n / elapsed,
                            elapsed / n * 1000.0,
                            app.stats_max * 1000.0,
                            app.stats_cpu / n * 1000.0
                        );
                        app.stats_start = Instant::now();
                        app.stats_frames = 0;
                        app.stats_max = 0.0;
                        app.stats_cpu = 0.0;
                    }
                }
                let ready =
                    app.frames >= 3 && app.started.elapsed().as_secs_f32() >= self.args.wait;
                if let (Some(path), true) = (&self.args.screenshot, ready) {
                    save_screenshot(platform, path);
                    event_loop.exit();
                }
                app.window.pre_present_notify();
                if let Err(error) = platform.surface.swap_buffers(&platform.context) {
                    eprintln!("swap_buffers: {error}");
                }
            }
            _ => {}
        }
        if leave {
            self.leave_room(event_loop);
        }
    }
}

fn save_screenshot(platform: &Platform, path: &std::path::Path) {
    let size = platform.window.inner_size();
    let pixels = gfx::read_framebuffer(&platform.gl, size.width, size.height);
    let result = image::save_buffer(
        path,
        &pixels,
        size.width,
        size.height,
        image::ColorType::Rgba8,
    );
    match result {
        Ok(()) => println!("screenshot saved to {}", path.display()),
        Err(error) => eprintln!("screenshot failed: {error}"),
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.platform.is_some() {
            return;
        }
        let platform = match create_platform(event_loop, &self.args) {
            Ok(platform) => platform,
            Err(error) => return self.fail(event_loop, error),
        };
        let screen = if self.args.use_menu() {
            create_menu(&platform, &self.args, &self.settings).map(|m| Screen::Menu(Box::new(m)))
        } else {
            let start = GameStart {
                from_menu: false,
                room_name: format!(
                    "Room 4721: {}[WEP02][WEP03][WEP04][WEP05][WEP06]{}",
                    self.args.mode,
                    online::ROOM_MARKER
                ),
                round_duration: 1200,
            };
            create(
                &platform,
                &self.args,
                self.online.take(),
                &self.settings,
                &start,
            )
            .map(|g| Screen::Game(Box::new(g)))
        };
        match screen {
            Ok(screen) => self.screen = Some(screen),
            Err(error) => self.fail(event_loop, error),
        }
        self.platform = Some(platform);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::Resized(size) => {
                if let (Some(platform), Some(w), Some(h)) = (
                    &self.platform,
                    NonZeroU32::new(size.width),
                    NonZeroU32::new(size.height),
                ) {
                    platform.surface.resize(&platform.context, w, h);
                }
            }
            _ => {}
        }
        match self.screen {
            Some(Screen::Menu(_)) => self.menu_event(event_loop, event),
            Some(Screen::Game(_)) => self.game_event(event_loop, event),
            None => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let (Some(Screen::Game(app)), DeviceEvent::MouseMotion { delta }) =
            (self.screen.as_mut(), event)
            && app.input.grabbed
        {
            app.input.mouse_delta.0 += delta.0;
            app.input.mouse_delta.1 += delta.1;
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Release GL resources while the window still exists.
        self.screen = None;
        self.stash = None;
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(platform) = &self.platform {
            platform.window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let mut args = parse_args()?;
    let event_loop = EventLoop::new()?;
    if args.list_rooms {
        return online::list_rooms(net::pun::CRAZYGAMES, &args.region, &args.name);
    }
    let mut online = None;
    if args.online {
        let assets = assets::find_assets_dir()?;
        let options = OnlineOptions {
            region: args.region.clone(),
            name: args.name.clone(),
            room: args.room.clone(),
            map: args.scene.clone(),
            create: args.create,
            mode: if args.mode == "DM" {
                "TDM".to_string()
            } else {
                args.mode.clone()
            },
        };
        let session = Online::join(net::pun::CRAZYGAMES, &options, &assets)?;
        if let Some(map) = session.map() {
            args.scene = map.to_string();
        }
        if let Some(mode) = session.game_mode() {
            args.mode = mode.to_string();
        }
        online = Some(session);
    }
    let assets = assets::find_assets_dir()?;
    let mut settings = Settings::new(&assets, PlayerPrefs::load(prefs::default_path()));
    if let Some(volume) = args.volume {
        settings.set_volume(volume);
    }
    if let Some(quality) = args.quality {
        if quality >= settings.quality_levels.len() {
            bail!(
                "--quality is 0..{}",
                settings.quality_levels.len().saturating_sub(1)
            );
        }
        settings.quality = quality;
    }
    let mut app = App {
        args,
        settings: Rc::new(std::cell::RefCell::new(settings)),
        online,
        screen: None,
        stash: None,
        platform: None,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    match app.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
