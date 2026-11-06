//! Pixel Gun Apocalypse 3 open-source client.
//!
//! Current milestone: weapons. Loads a map from `assets/data/scenes/`, renders it with the
//! original shading and spawns the local player (original movement and collision) at a spawn
//! point with its first-person weapons (switching, aiming, firing, reloading, bullets and
//! rockets); a free-fly camera is available for inspection and shows the player's animated
//! third-person body.
//!
//! Usage: `pga3-client [Scene] [--spawn N | --camera x,y,z,yaw,pitch] [--fly] [--stats]
//! [--weapon KEY] [--fire] [--screenshot out.png [--wait SECONDS]]`
//! (`--weapon` presses that number key on the first frame, `--fire` holds the fire button and
//! clicks every 30 frames: for screenshots of the weapons.)
//! Controls: click = capture mouse, Esc = release / quit, F = walk/fly toggle, Tab = next spawn.
//! Walking (original keys): WASD/arrows, Space jump, Left Shift run, C crouch, Left Ctrl prone.
//! Weapons (original keys): left mouse fire, right mouse aim, R reload, 1-6 / mouse wheel /
//! M, N select, O auto-fire toggle.
//! Flying: WASD, Space/C up/down, Shift fast.

mod anim;
mod assets;
mod character;
mod gfx;
mod obj;
mod physics;
mod player;
mod prefab;
mod primitives;
mod projectiles;
mod render;
mod scene;
mod skin;
mod unity;
mod weapons;

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use glam::{EulerRot, Mat4, Quat, Vec3};
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext, PossiblyCurrentContext,
    Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use character::{CharacterAnimation, CharacterInput};
use physics::CollisionWorld;
use player::{FIXED_DELTA_TIME, FrameInput, KeyAxis, MAXIMUM_DELTA_TIME, Player};
use prefab::PrefabInstance;
use projectiles::{ProjectileEvent, Projectiles, Templates};
use render::{PrefabModels, SceneRenderer, View};
use scene::Scene;
use weapons::{WeaponContext, WeaponEvent, WeaponInput, WeaponManager};

/// Player prefab spawned by the client (as `Player::from_prefab`).
const PLAYER_PREFAB: &str = "german";

/// `Input.GetAxis("Mouse X"/"Mouse Y")` sensitivity in this build's InputManager.
const MOUSE_AXIS_SENSITIVITY: f32 = 0.1;

struct Args {
    scene: String,
    spawn: Option<usize>,
    camera: Option<[f32; 5]>,
    fly: bool,
    screenshot: Option<PathBuf>,
    /// Seconds to run before taking the screenshot (animations in progress).
    wait: f32,
    /// Print frame-time statistics every second.
    stats: bool,
    /// Number key pressed on the first frame.
    weapon: Option<usize>,
    /// Hold the fire button (and click every 30 frames).
    fire: bool,
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        scene: "Bridge".to_string(),
        spawn: None,
        camera: None,
        fly: false,
        screenshot: None,
        wait: 0.0,
        stats: false,
        weapon: None,
        fire: false,
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
            "--screenshot" => {
                args.screenshot = Some(PathBuf::from(iter.next().context("--screenshot PATH")?))
            }
            other if other.starts_with("--") => bail!("unknown option {other}"),
            scene => args.scene = scene.to_string(),
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
    auto_fire_pressed: bool,
    number_pressed: [bool; 10],
    /// `Input.GetAxis("Mouse ScrollWheel")` of this frame.
    scroll: f32,
    key_m: bool,
    key_n: bool,
}

/// The local player's prefab instance as the owner sees it (`localObjectsToDeactivate` applied):
/// only the first-person weapon under `Weapon Camera` is visible.
struct LocalAvatar {
    prefab: PrefabInstance,
    models: PrefabModels,
    weapons: Option<WeaponManager>,
    main_camera: Option<usize>,
    weapon_camera: Option<usize>,
    /// Moved by `WalkSway` (`Player::weapon_local_position`).
    weapon_manager: Option<usize>,
    /// Prefabs the weapons spawn, their GPU models, and the live instances.
    templates: Templates,
    template_models: Vec<PrefabModels>,
    projectiles: Projectiles,
}

/// The same prefab as other players see it (`LookObject` hidden): the animated third-person
/// body, driven by `CharacterAnimation`. Drawn in fly mode as a debugging view.
struct BodyView {
    prefab: PrefabInstance,
    models: PrefabModels,
    character: Option<CharacterAnimation>,
}

enum Mode {
    Walk,
    Fly,
}

/// Field order is drop order: GL objects, then the surface and context, the window last
/// (dropping the window first leaves the EGL surface dangling and crashes on exit).
struct Running {
    renderer: SceneRenderer,
    local: LocalAvatar,
    body: BodyView,
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
    /// Frame-time statistics since `stats_start`: count, sum, max, CPU time in update+render.
    stats_start: Instant,
    stats_frames: u32,
    stats_max: f32,
    stats_cpu: f32,
    gl: glow::Context,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    window: Window,
}

struct App {
    args: Args,
    running: Option<Running>,
    error: Option<anyhow::Error>,
}

fn create(event_loop: &ActiveEventLoop, args: &Args) -> Result<Running> {
    let attributes = Window::default_attributes()
        .with_title(format!("Pixel Gun Apocalypse 3 — {}", args.scene))
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

    let assets = assets::find_assets_dir()?;
    let loading = Instant::now();
    let scene = scene::load_scene(&assets, &args.scene)?;
    let renderer = SceneRenderer::new(&gl, &assets, &scene)?;
    let world = physics::build_world(&assets, &scene)?;
    let mut player = Player::from_prefab(&assets, PLAYER_PREFAB)?;
    let mut renderer = renderer;
    let (local, body) = load_avatars(&gl, &assets, &mut renderer)?;
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
    let spawn_index = args.spawn.unwrap_or(0);
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
    }

    Ok(Running {
        window,
        surface,
        context,
        gl,
        scene,
        renderer,
        local,
        body,
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
        stats_start: Instant::now(),
        stats_frames: 0,
        stats_max: 0.0,
        stats_cpu: 0.0,
    })
}

/// Instantiates the player prefab twice (owner view and remote view) and runs the start-up
/// scripts in Unity's order: `PlayerNetworkController.Awake` deactivations, Animation
/// `playAutomatically`, `WeaponManager.Awake` (weapon take-in), `CharacterAnimation.Start`.
fn load_avatars(
    gl: &glow::Context,
    assets: &std::path::Path,
    renderer: &mut SceneRenderer,
) -> Result<(LocalAvatar, BodyView)> {
    let mut clips = anim::ClipLibrary::new(assets)?;
    let mut local = PrefabInstance::load(assets, PLAYER_PREFAB, &mut clips)?;
    let mut remote = PrefabInstance::load(assets, PLAYER_PREFAB, &mut clips)?;
    let controller = local
        .find_scripts("PlayerNetworkController")
        .first()
        .map(|(_, c)| c["fields"].clone());
    if let Some(fields) = &controller {
        local.set_active_refs(&fields["localObjectsToDeactivate"], false);
    }
    // Remote view: only LookObject (the first-person rig) is hidden. The original also hides
    // SpawnObject and Base_mesh and shows a team body mesh chosen elsewhere (docs/unknowns.md).
    if let Some(look) = remote.find("LookObject") {
        remote.nodes[look].active_self = false;
    }
    local.play_automatically();
    remote.play_automatically();
    let custom_tags = assets::load_tags(assets)?;
    let weapons = weapons::weapon_manager_awake(&mut local, &custom_tags);
    let character = CharacterAnimation::start(&mut remote, &mut clips);
    let spawned_refs = weapons.as_ref().map(spawned_prefabs).unwrap_or_default();
    let templates = Templates::load(assets, &spawned_refs, custom_tags, &mut clips)?;
    let template_models = templates
        .list
        .iter()
        .map(|t| renderer.prepare_prefab(gl, &t.prefab))
        .collect::<Result<Vec<_>>>()?;
    let local_models = renderer.prepare_prefab(gl, &local)?;
    let remote_models = renderer.prepare_prefab(gl, &remote)?;
    println!(
        "{PLAYER_PREFAB}: {} renderers ({} skipped), {} animations; weapons {:?}",
        local.renderers.len(),
        local.skipped_renderers,
        local.animations.len(),
        weapons.as_ref().map(|w| w
            .scripts
            .iter()
            .flatten()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>())
    );
    Ok((
        LocalAvatar {
            main_camera: local.find("Main Camera"),
            weapon_camera: local.find("Weapon Camera"),
            weapon_manager: local.find("WeaponManager"),
            prefab: local,
            models: local_models,
            weapons,
            templates,
            template_models,
            projectiles: Projectiles::default(),
        },
        BodyView {
            prefab: remote,
            models: remote_models,
            character,
        },
    ))
}

/// The bullet / projectile prefab refs of every weapon.
fn spawned_prefabs(manager: &WeaponManager) -> Vec<String> {
    manager
        .scripts
        .iter()
        .flatten()
        .flat_map(|s| {
            [
                s.machine_gun.bullet.clone(),
                s.shot_gun.bullet.clone(),
                s.grenade_launcher.projectile.clone(),
                s.knife.bullet.clone(),
            ]
        })
        .flatten()
        .collect()
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
        let dt_frame = match self.mode {
            Mode::Walk => dt.min(MAXIMUM_DELTA_TIME),
            Mode::Fly => dt,
        };
        self.local.prefab.begin_frame(dt_frame);
        self.body.prefab.begin_frame(dt_frame);
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
        i.auto_fire_pressed = false;
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
            auto_fire_toggle_down: i.auto_fire_pressed,
            number_down: i.number_pressed,
            scroll: i.scroll,
            next_held: i.key_m,
            previous_held: i.key_n,
        };
        self.step_player(dt, &input, weapon_input);
        self.camera.position = self.player.camera_position();
        self.camera.look(self.player.camera_rotation());
    }

    /// FixedUpdate steps, Update, coroutines, the animation pass, LateUpdate.
    fn step_player(&mut self, dt: f32, input: &FrameInput, mut weapon_input: WeaponInput) {
        let local = &mut self.local;
        self.fixed_time += dt;
        while self.fixed_time >= FIXED_DELTA_TIME {
            self.fixed_time -= FIXED_DELTA_TIME;
            self.player
                .fixed_update(&self.world, &self.scene.ladders, input);
            local.projectiles.fixed_update(
                &local.templates,
                &self.world,
                FIXED_DELTA_TIME,
                self.player.time,
            );
        }
        self.player.update(&self.world, dt, input.vertical, input);
        let time = self.player.time;
        weapons::sync_avatar(
            &mut local.prefab,
            &self.player,
            local.main_camera,
            local.weapon_manager,
        );
        // Objects spawned last frame run Start, then every Update.
        local
            .projectiles
            .update(&local.templates, &self.world, dt, time);
        if let Some(weapons) = &mut local.weapons {
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
                        Some(template) => local.projectiles.spawn(
                            &local.templates,
                            template,
                            position,
                            rotation,
                            velocity,
                        ),
                        None => eprintln!("weapon spawned unknown prefab {prefab}"),
                    },
                    // Audio is not implemented yet.
                    WeaponEvent::Sound(_) => {}
                }
            }
        }
        for event in local.projectiles.events.drain(..) {
            if let ProjectileEvent::Damage { amount, point, tag } = event {
                // No damage receivers (PlayerDamage, zombies) yet.
                let _ = (amount, point, tag);
            }
        }
        local.projectiles.end_frame(time);
        // Unity evaluates animations between Update and LateUpdate.
        self.local.prefab.update_animations();
        self.body.prefab.update_animations();
        self.player.late_update(input);
        for event in self.player.events.drain(..) {
            if let player::PlayerEvent::FallDamage(amount) = event {
                println!("fall damage {amount:.1}");
            }
        }
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

    /// `CharacterAnimation.LateUpdate`, then the transforms rendering uses.
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
        if let Some(character) = &mut self.body.character {
            character.late_update(&mut self.body.prefab, &input);
        }
        let root = Mat4::from_rotation_translation(self.player.rotation, self.player.position);
        self.body.prefab.world_override[0] = Some(root);
        weapons::sync_avatar(
            &mut self.local.prefab,
            &self.player,
            self.local.main_camera,
            self.local.weapon_manager,
        );
        self.local.prefab.update_world();
        self.body.prefab.update_world();
    }

    fn render(&mut self) {
        let size = self.window.inner_size();
        let view = match self.mode {
            Mode::Walk => View {
                position: self.player.camera_position(),
                rotation: self.player.camera_rotation(),
                fov_y_degrees: self.player.camera_fov,
                near: self.player.camera_near,
                far: self.player.camera_far,
            },
            Mode::Fly => View {
                position: self.camera.position,
                rotation: self.camera.rotation(),
                fov_y_degrees: self.fov.0,
                near: self.fov.1,
                far: self.fov.2,
            },
        };
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
        match self.mode {
            Mode::Walk => {
                let local = &self.local;
                let main_mask = local
                    .main_camera
                    .and_then(|n| local.prefab.camera(n))
                    .map_or(u32::MAX, |c| c.culling_mask);
                self.renderer.draw_models(
                    &self.gl,
                    &view,
                    w,
                    h,
                    &local.prefab,
                    &local.models,
                    main_mask,
                    false,
                    None,
                );
                // Weapon Camera (depth 2, clear flags Depth only, culling mask = layer 9).
                if let Some(node) = local.weapon_camera
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
                        &local.models,
                        camera.culling_mask,
                        camera.clear_flags == 3,
                        None,
                    );
                }
                // OnGUI (SniperScope, WeaponCrosshair) over the cameras.
                if let Some(weapons) = &self.local.weapons {
                    let quads = weapons.on_gui(&mut self.local.prefab, w as f32, h as f32);
                    self.renderer.draw_gui(&self.gl, &quads, w, h);
                }
            }
            Mode::Fly => {
                self.renderer.draw_models(
                    &self.gl,
                    &view,
                    w,
                    h,
                    &self.body.prefab,
                    &self.body.models,
                    u32::MAX,
                    false,
                    None,
                );
            }
        }
    }
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_none() {
            match create(event_loop, &self.args) {
                Ok(running) => self.running = Some(running),
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(app) = self.running.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(w), Some(h)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    app.surface.resize(&app.context, w, h);
                }
            }
            WindowEvent::Focused(false) => {
                app.set_grab(false);
                app.input.fire_held = false;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                let i = &mut app.input;
                if !i.grabbed {
                    // The first click only captures the cursor.
                    if pressed && button == MouseButton::Left {
                        app.set_grab(true);
                    }
                    return;
                }
                match button {
                    MouseButton::Left => {
                        i.fire_held = pressed;
                        i.fire_pressed |= pressed;
                    }
                    MouseButton::Right => i.aim_pressed |= pressed,
                    _ => {}
                }
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
                    KeyCode::KeyO => i.auto_fire_pressed |= down,
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
                    KeyCode::KeyF if down => {
                        app.mode = match app.mode {
                            Mode::Walk => Mode::Fly,
                            Mode::Fly => {
                                // Drop the player where the fly camera is.
                                let eye = app.player.camera_position() - app.player.position;
                                let yaw = Quat::from_rotation_y(app.camera.yaw.to_radians());
                                app.player.spawn(app.camera.position - eye, yaw);
                                app.player.rotation_y = 0.0;
                                Mode::Walk
                            }
                        };
                    }
                    KeyCode::Tab if down => {
                        app.spawn_index += 1;
                        place_at_spawn(
                            &mut app.player,
                            &mut app.camera,
                            &app.scene,
                            app.spawn_index,
                        );
                    }
                    KeyCode::KeyP if pressed && !repeat => {
                        let c = &app.camera;
                        println!(
                            "--camera {},{},{},{},{}",
                            c.position.x, c.position.y, c.position.z, c.yaw, c.pitch
                        );
                    }
                    KeyCode::Escape if pressed => {
                        if app.input.grabbed {
                            app.set_grab(false);
                        } else {
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
                if self.args.fire {
                    app.input.fire_held = true;
                    // Single-fire weapons need clicks.
                    app.input.fire_pressed |= app.frames % 30 == 0;
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
                    let size = app.window.inner_size();
                    let pixels = gfx::read_framebuffer(&app.gl, size.width, size.height);
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
                    event_loop.exit();
                }
                app.window.pre_present_notify();
                if let Err(error) = app.surface.swap_buffers(&app.context) {
                    eprintln!("swap_buffers: {error}");
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let (Some(app), DeviceEvent::MouseMotion { delta }) = (self.running.as_mut(), event)
            && app.input.grabbed
        {
            app.input.mouse_delta.0 += delta.0;
            app.input.mouse_delta.1 += delta.1;
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // Release GL resources while the window still exists.
        self.running = None;
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(app) = &self.running {
            app.window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let event_loop = EventLoop::new()?;
    let mut app = App {
        args,
        running: None,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    match app.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
