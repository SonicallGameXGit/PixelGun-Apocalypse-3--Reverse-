//! Local player movement: ports of the original `FPScontroller` (a modified Unity
//! `CharacterMotor`), `FPSinput`, `FPSMouseLook`, `CameraBob` and `WalkSway`, decompiled from the
//! wasm build.
//! Spec with evidence: `docs/gameplay/movement.md`. Values come from the player prefab
//! (`assets/data/prefabs/resources/german.json`).
//!
//! Frame order matches Unity: `fixed_update` (0.02 s steps) → `update` → `late_update`, so input
//! read in `late_update` drives the next frame's movement, as in the original.

use std::path::Path;

use anyhow::{Context, Result};
use glam::{Quat, Vec3};
use serde_json::Value;

use crate::assets::{f32_of, load_json};
use crate::physics::{CharacterController, CollisionWorld, ControllerHit, capsule_overlaps_box};
use crate::scene::LadderInstance;

/// `Time.fixedDeltaTime` (TimeManager "Fixed Timestep").
pub const FIXED_DELTA_TIME: f32 = 0.02;
/// `Time.maximumDeltaTime` (TimeManager "Maximum Allowed Timestep").
pub const MAXIMUM_DELTA_TIME: f32 = 0.333_333_34;

/// Unity `Vector3 == Vector3`: squared distance below 1e-10.
fn approx_eq(a: Vec3, b: Vec3) -> bool {
    (a - b).length_squared() < 9.999_999_4e-11
}

/// Unity `Vector3.normalized` (zero below magnitude 1e-5).
fn normalized(v: Vec3) -> Vec3 {
    let length = v.length();
    if length > 1e-5 {
        v / length
    } else {
        Vec3::ZERO
    }
}

/// Unity `Mathf.Lerp` (t clamped to 0..1).
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Unity `Vector3.Slerp` for the jump direction.
fn slerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 || approx_eq(b, Vec3::ZERO) {
        return a * (1.0 - t) + b * t;
    }
    let (la, lb) = (a.length(), b.length());
    let (na, nb) = (a / la, b / lb);
    let angle = na.dot(nb).clamp(-1.0, 1.0).acos();
    let direction = if angle < 1e-6 {
        na
    } else {
        (na * ((1.0 - t) * angle).sin() + nb * (t * angle).sin()) / angle.sin()
    };
    direction * (la + (lb - la) * t)
}

/// A Unity `AnimationCurve` (Hermite keys), clamped outside its key range.
#[derive(Clone, Debug)]
pub struct Curve {
    keys: Vec<(f32, f32, f32, f32)>,
}

impl Curve {
    fn from_json(value: &Value) -> Curve {
        let keys = value["keys"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|k| {
                (
                    f32_of(&k["time"]),
                    f32_of(&k["value"]),
                    f32_of(&k["inSlope"]),
                    f32_of(&k["outSlope"]),
                )
            })
            .collect();
        Curve { keys }
    }

    fn evaluate(&self, time: f32) -> f32 {
        let (Some(first), Some(last)) = (self.keys.first(), self.keys.last()) else {
            return 0.0;
        };
        if time <= first.0 {
            return first.1;
        }
        if time >= last.0 {
            return last.1;
        }
        for pair in self.keys.windows(2) {
            let ((t0, v0, _, out0), (t1, v1, in1, _)) = (pair[0], pair[1]);
            if time <= t1 {
                let dt = t1 - t0;
                let s = (time - t0) / dt;
                let (s2, s3) = (s * s, s * s * s);
                return (2.0 * s3 - 3.0 * s2 + 1.0) * v0
                    + (s3 - 2.0 * s2 + s) * out0 * dt
                    + (-2.0 * s3 + 3.0 * s2) * v1
                    + (s3 - s2) * in1 * dt;
            }
        }
        last.1
    }
}

/// `FPScontroller.movement` (`CharacterMotorMovement` plus the game's speeds and stances).
#[derive(Clone, Debug)]
pub struct Movement {
    pub max_forward_speed: f32,
    pub max_sideways_speed: f32,
    pub max_backwards_speed: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub can_crouch: bool,
    pub crouch_speed: f32,
    pub crouch_height: f32,
    pub crouch_smooth: f32,
    pub can_prone: bool,
    pub prone_speed: f32,
    pub prone_height: f32,
    pub slope_speed_multiplier: Curve,
    pub max_ground_acceleration: f32,
    pub max_air_acceleration: f32,
    pub gravity: f32,
    pub max_fall_speed: f32,
    pub enable_gravity: bool,
    pub collision_flags: u8,
    pub velocity: Vec3,
    pub frame_velocity: Vec3,
    pub hit_point: Vec3,
    pub last_hit_point: Vec3,
}

/// `FPScontroller.jumping` (`CharacterMotorJumping`).
#[derive(Clone, Debug)]
pub struct Jumping {
    pub enabled: bool,
    pub base_height: f32,
    pub extra_height: f32,
    pub perp_amount: f32,
    pub steep_perp_amount: f32,
    pub jumping: bool,
    pub holding_jump_button: bool,
    pub last_start_time: f32,
    pub last_button_down_time: f32,
    pub jump_dir: Vec3,
}

/// `FPScontroller.sliding` (`CharacterMotorSliding`). Its `enabled` flag is never read by the
/// original code, so it is not kept.
#[derive(Clone, Debug)]
pub struct Sliding {
    pub sliding_speed: f32,
    pub sideways_control: f32,
    pub speed_control: f32,
}

/// State of `CameraBob` (Main Camera) or `WalkSway` (WeaponManager): both classes have the same
/// fields and share their constructor (`f29906`); they differ only in how prone is handled.
#[derive(Clone, Debug)]
pub struct Bob {
    pub walk_bobbing_speed: f32,
    pub run_bobbing_speed: f32,
    pub idle_bobbing_speed: f32,
    pub bobbing_amount: f32,
    pub smooth: f32,
    midpoint: Vec3,
    timer: f32,
    bobbing_speed: f32,
    current_amount: f32,
}

/// Messages the original sends with `SendMessage` (sounds, animation, damage).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerEvent {
    Jump,
    Fall,
    Land,
    /// `PlayerDamage.ApplyFallDamage(amount)`.
    FallDamage(f32),
}

/// Inputs `FPSinput.LateUpdate` hands to the controller.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInput {
    /// `CF2Input.GetAxis("Horizontal")` / `("Vertical")`.
    pub horizontal: f32,
    pub vertical: f32,
    /// `GetKeyDown(Space)`, `GetKey(LeftShift)`, `GetKeyDown(C)`, `GetKeyDown(LeftControl)`.
    pub jump_down: bool,
    /// `GetKey(Space)` (LadderPlayer lets go of the ladder while it is held).
    pub jump_held: bool,
    pub run_held: bool,
    pub crouch_down: bool,
    pub prone_down: bool,
    /// `Input.GetAxis("Mouse X")` / `("Mouse Y")`.
    pub mouse_x: f32,
    pub mouse_y: f32,
}

/// `LadderPlayer` state.
#[derive(Clone, Debug)]
pub struct LadderPlayer {
    pub climb_speed: f32,
    pub climb_down_threshold: f32,
    pub climb_direction: Vec3,
    pub lateral_move: Vec3,
    pub ladder_movement: Vec3,
    /// Index into the scene's ladders.
    pub current_ladder: Option<usize>,
    pub latched_to_ladder: bool,
    /// Never set anywhere in the decompiled code.
    pub in_landing_pad: bool,
    pub trigger: bool,
    /// Whether the controller overlapped each ladder trigger after the last physics step
    /// (`OnTriggerEnter` fires on the change to true).
    pub inside_trigger: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct Player {
    // Transforms (Unity world space). Root rotation only changes through yaw.
    pub position: Vec3,
    pub rotation: Quat,
    /// `LookObject` local position and `FPSMouseLook.rotationY` (pitch, degrees, up positive).
    pub look_local_position: Vec3,
    pub rotation_y: f32,
    pub minimum_y: f32,
    pub maximum_y: f32,
    /// `Main Camera` local position (moved by `CameraBob`) and local rotation (turned by the
    /// weapons' camera recoil, `WeaponScript.cameraRecoilDo`).
    pub camera_local_position: Vec3,
    pub camera_local_rotation: Quat,
    /// `WeaponManager` local position (moved by `WalkSway`), under the Weapon Camera.
    pub weapon_local_position: Vec3,
    /// `CharacterController.velocity`: displacement of the last `Move` / its delta time.
    pub controller_velocity: Vec3,
    /// `FPSMouseLook.sensitivityX` of the root (axes 1) and `sensitivityY` of `LookObject`
    /// (axes 2): the prefab's values on the first frame, then `currentSensitivity`.
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    /// Both `FPSMouseLook`s are enabled (`EnableHelper` disables them while the menu is open).
    pub look_enabled: bool,
    /// `FPSMouseLook.weapScript`: the weapon node whose `WeaponScript` was cached.
    pub look_weapon: Option<usize>,
    /// Main Camera field of view (degrees), near and far planes.
    pub camera_fov: f32,
    pub camera_near: f32,
    pub camera_far: f32,

    pub controller: CharacterController,

    // FPScontroller fields.
    pub can_control: bool,
    pub use_fixed_update: bool,
    pub running: bool,
    pub walking: bool,
    pub can_run: bool,
    pub on_ladder: bool,
    pub input_move_direction: Vec3,
    pub input_jump: bool,
    pub input_run: bool,
    pub input_crouch: bool,
    pub input_prone: bool,
    pub movement: Movement,
    pub crouch: bool,
    pub standart_height: f32,
    pub center_y: f32,
    pub can_stand: bool,
    pub can_stand_crouch: bool,
    pub prone: bool,
    pub jumping: Jumping,
    pub sliding: Sliding,
    pub fall_limit: f32,
    pub fall_damage_multiplier: f32,
    pub grounded: bool,
    pub ground_normal: Vec3,
    pub last_ground_normal: Vec3,
    pub player_y_speed: f32,
    pub previous_position_y: f32,
    pub previous_grounded: bool,
    pub time_in_air: f32,

    pub camera_bob: Bob,
    /// `WalkSway` on `WeaponManager` (absent on prefabs without first-person weapons).
    pub walk_sway: Option<Bob>,
    pub ladder: LadderPlayer,
    /// `Time.time`.
    pub time: f32,
    pub events: Vec<PlayerEvent>,
}

fn component<'a>(nodes: &'a [Value], node: &str, script_or_type: &str) -> Result<&'a Value> {
    nodes
        .iter()
        .filter(|n| n["name"] == node)
        .flat_map(|n| n["components"].as_array().into_iter().flatten())
        .find(|c| c["script"] == script_or_type || c["type"] == script_or_type)
        .with_context(|| format!("{node}: no {script_or_type} component"))
}

fn local_position(nodes: &[Value], node: &str) -> Result<Vec3> {
    let t = &component(nodes, node, "Transform")?["data"]["m_LocalPosition"];
    Ok(Vec3::new(f32_of(&t["x"]), f32_of(&t["y"]), f32_of(&t["z"])))
}

fn local_rotation(nodes: &[Value], node: &str) -> Result<Quat> {
    let r = &component(nodes, node, "Transform")?["data"]["m_LocalRotation"];
    let q = Quat::from_xyzw(
        f32_of(&r["x"]),
        f32_of(&r["y"]),
        f32_of(&r["z"]),
        f32_of(&r["w"]),
    );
    Ok(if q.length_squared() > 0.0 {
        q.normalize()
    } else {
        Quat::IDENTITY
    })
}

fn bob_from_fields(fields: &Value, midpoint: Vec3) -> Bob {
    Bob {
        walk_bobbing_speed: f32_of(&fields["walkBobbingSpeed"]),
        run_bobbing_speed: f32_of(&fields["runBobbingSpeed"]),
        idle_bobbing_speed: f32_of(&fields["idleBobbingSpeed"]),
        bobbing_amount: f32_of(&fields["bobbingAmount"]),
        smooth: f32_of(&fields["smooth"]),
        // Awake: midpoint = localPosition.
        midpoint,
        timer: 0.0,
        bobbing_speed: 0.0,
        current_amount: 0.0,
    }
}

/// The sine wave both bob scripts start their `FixedUpdate` with: returns the target local
/// position (`Vector3.zero` when not grounded, as decompiled) and advances the timer.
fn bob_wave(bob: &mut Bob, grounded: bool) -> Vec3 {
    let wave_slice2 = (bob.timer * 2.0).sin();
    let wave_slice = bob.timer.sin();
    bob.timer += bob.bobbing_speed;
    if bob.timer > std::f32::consts::TAU {
        bob.timer -= std::f32::consts::TAU;
    }
    if wave_slice2 == 0.0 {
        bob.midpoint
    } else if grounded {
        // As decompiled: z stays 0, so a prefab z offset is lerped away.
        Vec3::new(
            bob.midpoint.x + wave_slice * bob.current_amount,
            bob.midpoint.y + wave_slice2 * bob.current_amount,
            0.0,
        )
    } else {
        Vec3::ZERO
    }
}

fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}

impl Player {
    /// Builds the player from a Resources prefab (`german`, `mercenarys`, `zombie`, ...).
    pub fn from_prefab(assets: &Path, prefab: &str) -> Result<Player> {
        let json = load_json(&assets.join(format!("data/prefabs/resources/{prefab}.json")))?;
        let nodes = json["nodes"].as_array().context("prefab has no nodes")?;
        let root = nodes
            .iter()
            .find(|n| n["parent"].is_null())
            .and_then(|n| n["name"].as_str())
            .context("prefab has no root")?
            .to_string();

        let cc = &component(nodes, &root, "CharacterController")?["data"];
        let center = &cc["m_Center"];
        let controller = CharacterController {
            height: f32_of(&cc["m_Height"]),
            radius: f32_of(&cc["m_Radius"]),
            center: Vec3::new(
                f32_of(&center["x"]),
                f32_of(&center["y"]),
                f32_of(&center["z"]),
            ),
            skin_width: f32_of(&cc["m_SkinWidth"]),
            step_offset: f32_of(&cc["m_StepOffset"]),
            slope_limit: f32_of(&cc["m_SlopeLimit"]),
            min_move_distance: f32_of(&cc["m_MinMoveDistance"]),
        };

        let fields = &component(nodes, &root, "FPScontroller")?["fields"];
        let m = &fields["movement"];
        let movement = Movement {
            max_forward_speed: f32_of(&m["maxForwardSpeed"]),
            max_sideways_speed: f32_of(&m["maxSidewaysSpeed"]),
            max_backwards_speed: f32_of(&m["maxBackwardsSpeed"]),
            walk_speed: f32_of(&m["WalkSpeed"]),
            run_speed: f32_of(&m["RunSpeed"]),
            can_crouch: flag(&m["canCrouch"]),
            crouch_speed: f32_of(&m["CrouchSpeed"]),
            crouch_height: f32_of(&m["crouchHeight"]),
            crouch_smooth: f32_of(&m["crouchSmooth"]),
            can_prone: flag(&m["canProne"]),
            prone_speed: f32_of(&m["ProneSpeed"]),
            prone_height: f32_of(&m["proneHeight"]),
            slope_speed_multiplier: Curve::from_json(&m["slopeSpeedMultiplier"]),
            max_ground_acceleration: f32_of(&m["maxGroundAcceleration"]),
            max_air_acceleration: f32_of(&m["maxAirAcceleration"]),
            gravity: f32_of(&m["gravity"]),
            max_fall_speed: f32_of(&m["maxFallSpeed"]),
            enable_gravity: flag(&m["enableGravity"]),
            collision_flags: 0,
            velocity: Vec3::ZERO,
            frame_velocity: Vec3::ZERO,
            hit_point: Vec3::ZERO,
            last_hit_point: Vec3::new(f32::INFINITY, 0.0, 0.0),
        };
        let j = &fields["jumping"];
        let jumping = Jumping {
            enabled: flag(&j["enabled"]),
            base_height: f32_of(&j["baseHeight"]),
            extra_height: f32_of(&j["extraHeight"]),
            perp_amount: f32_of(&j["perpAmount"]),
            steep_perp_amount: f32_of(&j["steepPerpAmount"]),
            jumping: false,
            holding_jump_button: false,
            last_start_time: 0.0,
            last_button_down_time: -100.0,
            jump_dir: Vec3::Y,
        };
        let s = &fields["sliding"];
        let sliding = Sliding {
            sliding_speed: f32_of(&s["slidingSpeed"]),
            sideways_control: f32_of(&s["sidewaysControl"]),
            speed_control: f32_of(&s["speedControl"]),
        };
        if flag(&fields["movingPlatform"]["enabled"]) {
            eprintln!("warning: {prefab}: moving platforms are not implemented");
        }

        let ladder = &component(nodes, &root, "LadderPlayer")?["fields"];
        let look = &component(nodes, "LookObject", "FPSMouseLook")?["fields"];
        let root_look = &component(nodes, &root, "FPSMouseLook")?["fields"];
        let bob = &component(nodes, "Main Camera", "CameraBob")?["fields"];
        let camera_local_position = local_position(nodes, "Main Camera")?;
        let walk_sway = component(nodes, "WeaponManager", "WalkSway").ok();
        let weapon_local_position = match walk_sway {
            Some(_) => local_position(nodes, "WeaponManager")?,
            None => Vec3::ZERO,
        };
        let camera = &component(nodes, "Main Camera", "Camera")?["data"];

        Ok(Player {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            look_local_position: local_position(nodes, "LookObject")?,
            rotation_y: 0.0,
            minimum_y: f32_of(&look["minimumY"]),
            maximum_y: f32_of(&look["maximumY"]),
            camera_local_position,
            camera_local_rotation: local_rotation(nodes, "Main Camera")?,
            weapon_local_position,
            controller_velocity: Vec3::ZERO,
            sensitivity_x: f32_of(&root_look["sensitivityX"]),
            sensitivity_y: f32_of(&look["sensitivityY"]),
            look_enabled: true,
            look_weapon: None,
            camera_fov: f32_of(&camera["field of view"]),
            camera_near: f32_of(&camera["near clip plane"]),
            camera_far: f32_of(&camera["far clip plane"]),
            // Awake: standartHeight / centerY from the controller; canRun and canStand true.
            standart_height: controller.height,
            center_y: controller.center.y,
            controller,
            can_control: flag(&fields["canControl"]),
            // Never set by the prefab or the constructor, so Update drives the movement.
            use_fixed_update: false,
            running: false,
            walking: false,
            can_run: true,
            on_ladder: false,
            input_move_direction: Vec3::ZERO,
            input_jump: false,
            input_run: false,
            input_crouch: false,
            input_prone: false,
            movement,
            crouch: flag(&fields["crouch"]),
            can_stand: true,
            can_stand_crouch: true,
            prone: flag(&fields["prone"]),
            jumping,
            sliding,
            fall_limit: f32_of(&fields["fallDamage"]["fallLimit"]),
            fall_damage_multiplier: f32_of(&fields["fallDamage"]["fallDamageMultiplier"]),
            grounded: true,
            ground_normal: Vec3::ZERO,
            last_ground_normal: Vec3::ZERO,
            player_y_speed: 0.0,
            previous_position_y: 0.0,
            previous_grounded: false,
            time_in_air: 0.0,
            camera_bob: bob_from_fields(bob, camera_local_position),
            walk_sway: walk_sway.map(|c| bob_from_fields(&c["fields"], weapon_local_position)),
            ladder: LadderPlayer {
                climb_speed: f32_of(&ladder["climbSpeed"]),
                climb_down_threshold: f32_of(&ladder["climbDownThreshold"]),
                climb_direction: Vec3::ZERO,
                lateral_move: Vec3::ZERO,
                ladder_movement: Vec3::ZERO,
                current_ladder: None,
                latched_to_ladder: false,
                in_landing_pad: false,
                trigger: false,
                inside_trigger: Vec::new(),
            },
            time: 0.0,
            events: Vec::new(),
        })
    }

    /// `RoomMultiplayerMenu.SpawnPlayer`: the prefab is instantiated at the spawn point's
    /// position and rotation.
    pub fn spawn(&mut self, position: Vec3, rotation: Quat) {
        self.position = position;
        self.rotation = rotation;
        self.previous_position_y = position.y;
        self.movement.velocity = Vec3::ZERO;
    }

    /// World rotation of `LookObject` (root yaw, then `FPSMouseLook` pitch).
    pub fn look_rotation(&self) -> Quat {
        self.rotation * Quat::from_rotation_x((-self.rotation_y).to_radians())
    }

    /// World rotation of the Main Camera: LookObject, then its own local rotation.
    pub fn camera_rotation(&self) -> Quat {
        self.look_rotation() * self.camera_local_rotation
    }

    /// World position of the Main Camera: root → LookObject → Main Camera.
    pub fn camera_position(&self) -> Vec3 {
        self.position
            + self.rotation * self.look_local_position
            + self.look_rotation() * self.camera_local_position
    }

    fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    // -----------------------------------------------------------------------------------------
    // Unity callbacks

    /// One fixed step: the `FixedUpdate`s of `FPScontroller` (fall damage), `LadderPlayer` and
    /// `CameraBob`, `WalkSway`, then the physics step's trigger events (`Ladder.OnTriggerEnter`).
    pub fn fixed_update(
        &mut self,
        world: &CollisionWorld,
        ladders: &[LadderInstance],
        input: &FrameInput,
    ) {
        self.fpscontroller_fixed_update();
        self.ladder_fixed_update(world, input);
        self.camera_bob_fixed_update(FIXED_DELTA_TIME);
        self.walk_sway_fixed_update(FIXED_DELTA_TIME);
        self.trigger_events(ladders);
    }

    fn fpscontroller_fixed_update(&mut self) {
        let dt = FIXED_DELTA_TIME;
        self.player_y_speed = (self.previous_position_y - self.position.y) / dt;
        if !self.grounded && self.player_y_speed > 0.0 {
            self.time_in_air += dt;
        }
        if self.on_ladder {
            self.time_in_air = 0.0;
        }
        self.previous_position_y = self.position.y;
        if self.grounded != self.previous_grounded {
            if self.grounded {
                if self.time_in_air > self.fall_limit {
                    self.events.push(PlayerEvent::FallDamage(
                        self.player_y_speed * self.fall_damage_multiplier,
                    ));
                }
                self.time_in_air = 0.0;
            }
            self.previous_grounded = self.grounded;
        }
    }

    /// `FPScontroller.Update` followed by both `FPSMouseLook.Update`s.
    pub fn update(
        &mut self,
        world: &CollisionWorld,
        dt: f32,
        vertical_axis: f32,
        input: &FrameInput,
    ) {
        self.time += dt;
        if !self.use_fixed_update {
            self.update_function(world, dt);
        }

        if vertical_axis > 0.1 && self.input_run && self.can_run && !self.on_ladder && self.walking
        {
            if self.can_stand && self.can_stand_crouch {
                self.on_running();
            }
        } else {
            self.off_running();
        }
        let v = self.movement.velocity;
        self.walking = v.x > 0.01 || v.z > 0.01 || v.x < -0.01 || v.z < -0.01;

        if self.can_control {
            if self.movement.can_crouch && !self.on_ladder {
                self.crouch(world, dt);
            }
            if self.movement.can_prone && !self.on_ladder {
                self.prone(world, dt);
            }
            if self.on_ladder {
                self.grounded = false;
                self.crouch = false;
                self.prone = false;
            }
            if !self.crouch && !self.prone && self.controller.height < self.standart_height - 0.01 {
                let t = dt / self.movement.crouch_smooth;
                self.controller.height = lerp(self.controller.height, self.standart_height, t);
                self.controller.center.y = lerp(self.controller.center.y, self.center_y, t);
                // The original lerps the LookObject towards standartHeight (not its prefab y).
                self.look_local_position.y =
                    lerp(self.look_local_position.y, self.standart_height, t);
            }
        }

        if self.look_enabled {
            self.mouse_look(input);
        }
    }

    /// `FPSinput.LateUpdate`.
    pub fn late_update(&mut self, input: &FrameInput) {
        let mut direction = Vec3::new(input.horizontal, 0.0, input.vertical);
        if !approx_eq(direction, Vec3::ZERO) {
            let mut length = direction.length();
            direction /= length;
            length = length.min(1.0);
            length *= length;
            direction *= length;
        }
        self.input_move_direction = self.rotation * direction;
        self.input_jump = input.jump_down;
        self.input_run = input.run_held;
        self.input_crouch = input.crouch_down;
        self.input_prone = input.prone_down;
    }

    /// `FPSMouseLook.Update` on the root (axes 1: yaw) and on LookObject (axes 2: pitch), turning
    /// with the sensitivities the previous frame left (`docs/gameplay/settings.md`).
    fn mouse_look(&mut self, input: &FrameInput) {
        self.rotation *= Quat::from_rotation_y((input.mouse_x * self.sensitivity_x).to_radians());
        self.rotation_y = (self.rotation_y + input.mouse_y * self.sensitivity_y)
            .clamp(self.minimum_y, self.maximum_y);
    }

    /// The end of `FPSMouseLook.Update`: `weapScript` is cached from the first selected weapon
    /// (cleared without one) and `currentSensitivity = RoomMultiplayerMenu.sens`, x0.25 while
    /// that cached script is aimed, becomes both sensitivities for the next frame. The first
    /// weapon (the Machete) cannot aim, so in practice the factor never applies.
    /// `aimed_of(node)` is `weapScript.aimed` of the weapon at `node`.
    pub fn update_look_sensitivity(
        &mut self,
        sens: f32,
        selected: Option<usize>,
        aimed_of: impl Fn(usize) -> bool,
    ) {
        if !self.look_enabled {
            return;
        }
        match selected {
            Some(node) => {
                if self.look_weapon.is_none() {
                    self.look_weapon = Some(node);
                }
            }
            None => self.look_weapon = None,
        }
        let aimed = self.look_weapon.is_some_and(aimed_of);
        let current = if aimed { sens * 0.25 } else { sens };
        self.sensitivity_x = current;
        self.sensitivity_y = current;
    }

    // -----------------------------------------------------------------------------------------
    // Ladder / LadderPlayer

    /// `Ladder.OnTriggerEnter` for the triggers the controller started overlapping in this
    /// physics step (the root is tagged "Player", so it latches).
    fn trigger_events(&mut self, ladders: &[LadderInstance]) {
        let capsule = self.controller.proxy_capsule(self.position, self.rotation);
        self.ladder.inside_trigger.resize(ladders.len(), false);
        for (index, ladder) in ladders.iter().enumerate() {
            let inside = capsule_overlaps_box(
                &capsule,
                ladder.world,
                ladder.trigger_center,
                ladder.trigger_size,
            );
            if inside && !self.ladder.inside_trigger[index] {
                self.latch_ladder(index, ladder);
            }
            self.ladder.inside_trigger[index] = inside;
        }
    }

    fn latch_ladder(&mut self, index: usize, ladder: &LadderInstance) {
        self.ladder.current_ladder = Some(index);
        self.ladder.latched_to_ladder = true;
        self.ladder.climb_direction = ladder.climb_direction;
        self.on_ladder();
        self.ladder.trigger = true;
    }

    fn unlatch_ladder(&mut self) {
        self.ladder.latched_to_ladder = false;
        self.ladder.current_ladder = None;
        self.ladder.trigger = false;
        self.off_ladder();
    }

    /// `LadderPlayer.FixedUpdate`.
    fn ladder_fixed_update(&mut self, world: &CollisionWorld, input: &FrameInput) {
        if !self.ladder.latched_to_ladder {
            return;
        }
        if self.ladder.trigger {
            self.ladder_raycast_check(world);
        }
        if input.jump_held {
            self.unlatch_ladder();
        }
        // The original carries on after letting go: this step still moves.
        let mut vertical_move = normalized(self.ladder.climb_direction) * input.vertical;
        let camera_forward = self.camera_rotation() * Vec3::Z;
        let sign = if camera_forward.y > self.ladder.climb_down_threshold {
            1.0
        } else {
            -1.0
        };
        vertical_move *= sign;
        let lateral = if self.ladder.in_landing_pad {
            Vec3::new(input.horizontal, 0.0, input.vertical)
        } else {
            Vec3::new(input.horizontal, 0.0, 0.0)
        };
        self.ladder.lateral_move = self.rotation * lateral;
        self.ladder.ladder_movement = vertical_move + self.ladder.lateral_move;
        let motion = self.ladder.ladder_movement * self.ladder.climb_speed * FIXED_DELTA_TIME;
        self.controller_move(world, motion, FIXED_DELTA_TIME);
    }

    /// `LadderPlayer.RaycastCheck`: lets go when nothing is within one radius in front of the
    /// capsule (the ladder's solid `MiddleCollider`).
    fn ladder_raycast_check(&mut self, world: &CollisionWorld) {
        let cc = &self.controller;
        // transform.position + center, not rotated, as decompiled.
        let middle = self.position + cc.center;
        let bottom = middle - Vec3::Y * cc.height * 0.5;
        let top = bottom + Vec3::Y * cc.height;
        let forward = self.rotation * Vec3::Z;
        if world
            .capsule_cast(bottom, top, cc.radius, forward, cc.radius)
            .is_none()
        {
            self.unlatch_ladder();
        }
    }

    // -----------------------------------------------------------------------------------------
    // FPScontroller

    fn update_function(&mut self, world: &CollisionWorld, dt: f32) {
        let mut velocity = self.movement.velocity;
        velocity = self.apply_input_velocity_change(velocity, dt);
        if self.movement.enable_gravity {
            // Game-specific: no movement at all on a jump press while crouched or prone.
            if (self.prone || self.crouch) && self.input_jump {
                return;
            }
            velocity = self.apply_gravity_and_jumping(velocity, dt);
        }

        let last_position = self.position;
        let mut offset = velocity * dt;
        let push_down = self
            .controller
            .step_offset
            .max(Vec3::new(offset.x, 0.0, offset.z).length());
        if self.grounded {
            offset -= push_down * self.up();
        }

        self.ground_normal = Vec3::ZERO;
        self.movement.collision_flags = self.controller_move(world, offset, dt);
        self.movement.last_hit_point = self.movement.hit_point;
        self.last_ground_normal = self.ground_normal;

        let old_h_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
        self.movement.velocity = (self.position - last_position) / dt;
        let new_h_velocity = Vec3::new(self.movement.velocity.x, 0.0, self.movement.velocity.z);
        if approx_eq(old_h_velocity, Vec3::ZERO) {
            self.movement.velocity = Vec3::new(0.0, self.movement.velocity.y, 0.0);
        } else {
            let projected = new_h_velocity.dot(old_h_velocity) / old_h_velocity.length_squared();
            self.movement.velocity =
                old_h_velocity * projected.clamp(0.0, 1.0) + self.movement.velocity.y * Vec3::Y;
        }
        if self.movement.velocity.y < velocity.y - 0.001 {
            if self.movement.velocity.y < 0.0 {
                self.movement.velocity.y = velocity.y;
            } else {
                self.jumping.holding_jump_button = false;
            }
        }

        if self.grounded && !self.is_grounded_test() {
            self.grounded = false;
            self.events.push(PlayerEvent::Fall);
            self.position += push_down * self.up();
        } else if !self.grounded && self.is_grounded_test() {
            self.grounded = true;
            self.jumping.jumping = false;
            self.events.push(PlayerEvent::Land);
        }
    }

    fn apply_input_velocity_change(&mut self, mut velocity: Vec3, dt: f32) -> Vec3 {
        if !self.can_control {
            self.input_move_direction = Vec3::ZERO;
        }
        let mut desired_velocity;
        if self.grounded && self.too_steep() {
            desired_velocity =
                normalized(Vec3::new(self.ground_normal.x, 0.0, self.ground_normal.z));
            let projected_move_dir = project(self.input_move_direction, desired_velocity);
            desired_velocity = desired_velocity
                + projected_move_dir * self.sliding.speed_control
                + (self.input_move_direction - projected_move_dir) * self.sliding.sideways_control;
            desired_velocity *= self.sliding.sliding_speed;
        } else {
            desired_velocity = self.desired_horizontal_velocity();
        }
        if self.grounded {
            desired_velocity =
                adjust_ground_velocity_to_normal(desired_velocity, self.ground_normal);
        } else {
            velocity.y = 0.0;
        }
        let max_velocity_change = self.max_acceleration(self.grounded) * dt;
        let mut change = desired_velocity - velocity;
        if change.length_squared() > max_velocity_change * max_velocity_change {
            change = normalized(change) * max_velocity_change;
        }
        if self.grounded || self.can_control {
            velocity += change;
        }
        if self.grounded {
            velocity.y = velocity.y.min(0.0);
        }
        velocity
    }

    fn apply_gravity_and_jumping(&mut self, mut velocity: Vec3, dt: f32) -> Vec3 {
        let gravity = self.movement.gravity;
        if !self.input_jump || !self.can_control {
            self.jumping.holding_jump_button = false;
            self.jumping.last_button_down_time = -100.0;
        }
        if self.input_jump && self.jumping.last_button_down_time < 0.0 && self.can_control {
            self.jumping.last_button_down_time = self.time;
        }
        if self.grounded {
            velocity.y = velocity.y.min(0.0) - gravity * dt;
        } else {
            velocity.y = self.movement.velocity.y - gravity * dt;
            if self.jumping.jumping
                && self.jumping.holding_jump_button
                && self.time
                    < self.jumping.last_start_time
                        + self.jumping.extra_height
                            / self.calculate_jump_vertical_speed(self.jumping.base_height)
            {
                velocity += self.jumping.jump_dir * gravity * dt;
            }
            velocity.y = velocity.y.max(-self.movement.max_fall_speed);
        }
        if self.grounded {
            if self.jumping.enabled
                && self.can_control
                && self.time - self.jumping.last_button_down_time < 0.2
            {
                self.grounded = false;
                self.jumping.jumping = true;
                self.jumping.last_start_time = self.time;
                self.jumping.last_button_down_time = -100.0;
                self.jumping.holding_jump_button = true;
                let perp = if self.too_steep() {
                    self.jumping.steep_perp_amount
                } else {
                    self.jumping.perp_amount
                };
                self.jumping.jump_dir = slerp(self.up(), self.ground_normal, perp);
                velocity.y = 0.0;
                velocity += self.jumping.jump_dir
                    * self.calculate_jump_vertical_speed(self.jumping.base_height);
                self.events.push(PlayerEvent::Jump);
            } else {
                self.jumping.holding_jump_button = false;
            }
        }
        velocity
    }

    /// `CharacterController.Move`, with `OnControllerColliderHit` for every contact.
    /// `CharacterController.Move`; also records `CharacterController.velocity` (the move's
    /// displacement over the current delta time).
    fn controller_move(&mut self, world: &CollisionWorld, motion: Vec3, dt: f32) -> u8 {
        let mut hits = Vec::new();
        let before = self.position;
        let flags =
            self.controller
                .move_by(world, &mut self.position, self.rotation, motion, &mut hits);
        if dt > 0.0 {
            self.controller_velocity = (self.position - before) / dt;
        }
        for hit in &hits {
            self.on_controller_collider_hit(hit);
        }
        flags
    }

    /// `FPScontroller.OnLadder` (SendMessage from `LadderPlayer.LatchLadder`).
    fn on_ladder(&mut self) {
        self.on_ladder = true;
        self.input_move_direction = Vec3::ZERO;
        self.movement.enable_gravity = false;
    }

    /// `FPScontroller.OffLadder` (its camera-forward computation is unused).
    fn off_ladder(&mut self) {
        self.on_ladder = false;
        self.movement.enable_gravity = true;
    }

    fn on_controller_collider_hit(&mut self, hit: &ControllerHit) {
        if hit.normal.y > 0.0 && hit.normal.y > self.ground_normal.y && hit.move_direction.y < 0.0 {
            if (hit.point - self.movement.last_hit_point).length_squared() > 0.001
                || approx_eq(self.last_ground_normal, Vec3::ZERO)
            {
                self.ground_normal = hit.normal;
            } else {
                self.ground_normal = self.last_ground_normal;
            }
            self.movement.hit_point = hit.point;
            self.movement.frame_velocity = Vec3::ZERO;
        }
    }

    fn is_grounded_test(&self) -> bool {
        self.ground_normal.y > 0.01
    }

    fn too_steep(&self) -> bool {
        self.ground_normal.y <= self.controller.slope_limit.to_radians().cos()
    }

    fn calculate_jump_vertical_speed(&self, target_jump_height: f32) -> f32 {
        (2.0 * target_jump_height * self.movement.gravity).sqrt()
    }

    fn max_acceleration(&self, grounded: bool) -> f32 {
        if grounded {
            self.movement.max_ground_acceleration
        } else {
            self.movement.max_air_acceleration
        }
    }

    fn desired_horizontal_velocity(&self) -> Vec3 {
        let local = self.rotation.inverse() * self.input_move_direction;
        let mut max_speed = self.max_speed_in_direction(local);
        if self.grounded {
            let slope_angle = normalized(self.movement.velocity).y.asin().to_degrees();
            max_speed *= self.movement.slope_speed_multiplier.evaluate(slope_angle);
        }
        self.rotation * (local * max_speed)
    }

    fn max_speed_in_direction(&self, desired: Vec3) -> f32 {
        if approx_eq(desired, Vec3::ZERO) {
            return 0.0;
        }
        let m = &self.movement;
        let z_multiplier = if desired.z > 0.0 {
            m.max_forward_speed
        } else {
            m.max_backwards_speed
        } / m.max_sideways_speed;
        let temp = normalized(Vec3::new(desired.x, 0.0, desired.z / z_multiplier));
        Vec3::new(temp.x, 0.0, temp.z * z_multiplier).length() * m.max_sideways_speed
    }

    fn on_running(&mut self) {
        self.running = true;
        self.movement.max_forward_speed = self.movement.run_speed;
        self.movement.max_sideways_speed = self.movement.run_speed;
        self.jumping.extra_height = self.jumping.base_height + 0.15;
    }

    fn off_running(&mut self) {
        self.running = false;
        if !self.crouch && !self.prone {
            self.movement.max_forward_speed = self.movement.walk_speed;
            self.movement.max_sideways_speed = self.movement.walk_speed;
            self.movement.max_backwards_speed = self.movement.walk_speed * 0.5;
            self.jumping.extra_height = self.jumping.base_height;
        }
    }

    /// Lerps the controller height, its centre and the LookObject towards a stance height.
    fn lerp_stance(&mut self, height: f32, speed: f32, dt: f32) {
        let t = dt / self.movement.crouch_smooth;
        self.controller.height = lerp(self.controller.height, height, t);
        self.controller.center.y = lerp(self.controller.center.y, height * 0.5, t);
        self.look_local_position.y = lerp(self.look_local_position.y, height, t);
        self.movement.max_forward_speed = speed;
        self.movement.max_sideways_speed = speed;
        self.movement.max_backwards_speed = speed;
    }

    fn crouch(&mut self, world: &CollisionWorld, dt: f32) {
        if self.input_crouch && !self.running && !self.on_ladder && self.can_stand {
            self.prone = false;
            self.crouch = !self.crouch;
        }
        let blocked = world
            .sphere_cast(
                self.position,
                self.controller.radius,
                self.up(),
                self.standart_height,
            )
            .is_some();
        if blocked {
            if self.crouch {
                self.can_stand = false;
            }
            self.can_stand_crouch = false;
        } else {
            if self.input_jump && self.crouch {
                self.crouch = false;
            }
            if self.running && self.crouch {
                self.crouch = false;
            }
            if self.crouch {
                self.can_stand = true;
            }
            self.can_stand_crouch = true;
        }
        let height = self.movement.crouch_height;
        if self.crouch
            && !self.prone
            && !(self.controller.height < height + 0.01 && self.controller.height > height - 0.01)
        {
            self.lerp_stance(height, self.movement.crouch_speed, dt);
        }
    }

    fn prone(&mut self, world: &CollisionWorld, dt: f32) {
        if self.input_prone && !self.running && !self.on_ladder && (self.can_stand || self.crouch) {
            self.crouch = false;
            self.prone = !self.prone;
            if self.prone {
                self.can_stand_crouch = true;
            } else {
                self.crouch = true;
            }
            if self.can_stand_crouch {
                self.crouch = false;
            }
        }
        if self.input_jump && self.prone && self.can_stand {
            self.prone = false;
            self.crouch = !self.can_stand_crouch;
        }
        if self.prone || self.running {
            let blocked = world
                .sphere_cast(
                    self.position,
                    self.controller.radius,
                    self.up(),
                    self.movement.crouch_height * 0.9,
                )
                .is_some();
            if blocked {
                if self.prone {
                    self.can_stand = false;
                }
            } else {
                if self.running && self.prone {
                    self.prone = false;
                    self.crouch = !self.can_stand_crouch;
                }
                if self.prone {
                    self.can_stand = true;
                }
            }
        }
        if self.prone && !self.crouch && self.controller.height > self.movement.prone_height + 0.01
        {
            self.lerp_stance(self.movement.prone_height, self.movement.prone_speed, dt);
        }
    }

    // -----------------------------------------------------------------------------------------
    // CameraBob

    fn camera_bob_fixed_update(&mut self, dt: f32) {
        if self.prone {
            return;
        }
        let walking = self.walking;
        let running = self.running;
        let bob = &mut self.camera_bob;
        // timeScale is always 1 here; the original rescales by fixedDeltaTime / 0.02 otherwise.
        let (walk, run, idle) = (
            bob.walk_bobbing_speed,
            bob.run_bobbing_speed,
            bob.idle_bobbing_speed,
        );

        let target = bob_wave(bob, self.grounded);
        if walking && !running {
            bob.bobbing_speed = walk;
            bob.current_amount = bob.bobbing_amount;
        } else if running {
            bob.bobbing_speed = run;
            bob.current_amount = bob.bobbing_amount;
        }
        if !running && !walking {
            bob.bobbing_speed = idle;
            bob.current_amount = bob.bobbing_amount * 0.3;
        }
        let t = (dt * bob.smooth).clamp(0.0, 1.0);
        self.camera_local_position = self.camera_local_position.lerp(target, t);
    }

    // -----------------------------------------------------------------------------------------
    // WalkSway (`f29905`): CameraBob's wave on the WeaponManager, except that prone uses the idle
    // sway instead of skipping the update.

    fn walk_sway_fixed_update(&mut self, dt: f32) {
        let (walking, running, prone, grounded) =
            (self.walking, self.running, self.prone, self.grounded);
        let Some(bob) = &mut self.walk_sway else {
            return;
        };
        // timeScale is always 1 here; the original rescales by fixedDeltaTime / 0.02 otherwise.
        let target = bob_wave(bob, grounded);
        if walking && !running && !prone {
            bob.bobbing_speed = bob.walk_bobbing_speed;
            bob.current_amount = bob.bobbing_amount;
        }
        if running {
            bob.bobbing_speed = bob.run_bobbing_speed;
            bob.current_amount = bob.bobbing_amount;
        }
        if (!running && !walking) || prone {
            bob.bobbing_speed = bob.idle_bobbing_speed;
            bob.current_amount = bob.bobbing_amount * 0.3;
        }
        // `Time.deltaTime` inside FixedUpdate is the fixed step.
        let t = (dt * bob.smooth).clamp(0.0, 1.0);
        self.weapon_local_position = self.weapon_local_position.lerp(target, t);
    }
}

/// Unity `Vector3.Project`.
fn project(vector: Vec3, on_normal: Vec3) -> Vec3 {
    let sqr = on_normal.length_squared();
    if sqr < f32::EPSILON {
        Vec3::ZERO
    } else {
        on_normal * (vector.dot(on_normal) / sqr)
    }
}

fn adjust_ground_velocity_to_normal(h_velocity: Vec3, ground_normal: Vec3) -> Vec3 {
    let sideways = Vec3::Y.cross(h_velocity);
    normalized(sideways.cross(ground_normal)) * h_velocity.length()
}

/// Unity keyboard axis (`InputManager` button axis): moves towards the pressed direction at
/// `sensitivity` units/s, back to 0 at `gravity`, snapping through 0 on reversal.
#[derive(Clone, Copy, Debug)]
pub struct KeyAxis {
    pub sensitivity: f32,
    pub gravity: f32,
    pub dead: f32,
    pub snap: bool,
    value: f32,
}

impl KeyAxis {
    /// `Horizontal` / `Vertical` in this build's InputManager.
    pub fn movement() -> KeyAxis {
        KeyAxis {
            sensitivity: 3.0,
            gravity: 3.0,
            dead: 0.001,
            snap: true,
            value: 0.0,
        }
    }

    pub fn update(&mut self, positive: bool, negative: bool, dt: f32) -> f32 {
        let target = positive as i32 as f32 - negative as i32 as f32;
        if target != 0.0 {
            if self.snap && self.value != 0.0 && self.value.signum() != target {
                self.value = 0.0;
            }
            self.value = move_towards(self.value, target, self.sensitivity * dt);
        } else {
            self.value = move_towards(self.value, 0.0, self.gravity * dt);
        }
        if self.value.abs() < self.dead {
            0.0
        } else {
            self.value
        }
    }
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;
    use crate::scene::LadderInstance;
    use glam::Mat4;

    #[test]
    fn mouse_look_sensitivity_follows_the_menu_a_frame_later() {
        let assets = find_assets_dir().unwrap();
        let mut player = Player::from_prefab(&assets, "german").unwrap();
        // The prefab's values on the first frame.
        assert_eq!((player.sensitivity_x, player.sensitivity_y), (4.0, 3.5));
        let input = FrameInput {
            mouse_x: 1.0,
            mouse_y: 1.0,
            ..FrameInput::default()
        };
        player.mouse_look(&input);
        assert!((player.rotation_y - 3.5).abs() < 1e-6);
        // Then `RoomMultiplayerMenu.sens`, x0.25 while the first cached weapon is aimed.
        player.update_look_sensitivity(1.8, Some(7), |_| false);
        assert_eq!((player.sensitivity_x, player.sensitivity_y), (1.8, 1.8));
        player.update_look_sensitivity(2.0, Some(9), |node| node == 7);
        assert_eq!(player.look_weapon, Some(7));
        assert_eq!(player.sensitivity_x, 0.5);
        // No selected weapon clears the cache; a disabled look changes nothing.
        player.update_look_sensitivity(2.0, None, |_| true);
        assert_eq!((player.look_weapon, player.sensitivity_x), (None, 2.0));
        player.look_enabled = false;
        player.update_look_sensitivity(3.0, Some(1), |_| false);
        assert_eq!(player.sensitivity_x, 2.0);
    }

    fn flat_world() -> CollisionWorld {
        let mut world = CollisionWorld::new();
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(400.0, 1.0, 400.0),
            false,
            0,
        );
        world
    }

    fn player_on_floor() -> (Player, CollisionWorld) {
        let assets = find_assets_dir().expect("assets");
        let mut player = Player::from_prefab(&assets, "german").expect("prefab");
        player.spawn(Vec3::new(0.0, 0.5, 0.0), Quat::IDENTITY);
        (player, flat_world())
    }

    fn step(player: &mut Player, world: &CollisionWorld, input: &FrameInput, dt: f32) {
        player.update(world, dt, input.vertical, input);
        player.late_update(input);
    }

    #[test]
    fn climbs_ladder_and_lets_go_with_space() {
        let (mut player, mut world) = player_on_floor();
        // Laid out like the maps' ladders (Crash): a solid 0.088 m MiddleCollider with a
        // 0.078 m trigger slab 0.16-0.24 m in front of it.
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 2.4, 1.044),
            Vec3::new(1.15, 4.8, 0.088),
            false,
            0,
        );
        let ladder = LadderInstance {
            world: Mat4::IDENTITY,
            trigger_center: Vec3::new(0.0, 2.6, 0.803),
            trigger_size: Vec3::new(1.7, 5.2, 0.078),
            climb_direction: Vec3::new(0.0, 4.7, 0.0),
        };
        world.add_box(
            Mat4::IDENTITY,
            ladder.trigger_center,
            ladder.trigger_size,
            true,
            crate::physics::IGNORE_RAYCAST_LAYER,
        );
        let ladders = [ladder];
        let dt = FIXED_DELTA_TIME;
        let frame = |player: &mut Player, input: &FrameInput| {
            player.fixed_update(&world, &ladders, input);
            step(player, &world, input, dt);
        };
        let forward = FrameInput {
            vertical: 1.0,
            ..Default::default()
        };
        for _ in 0..50 {
            frame(&mut player, &FrameInput::default());
        }
        let ground = player.position.y;
        while !player.on_ladder {
            frame(&mut player, &forward);
        }
        assert!(player.ladder.latched_to_ladder && !player.movement.enable_gravity);
        let start = player.position.y;
        for _ in 0..10 {
            frame(&mut player, &forward);
        }
        // climbSpeed 6 m/s along the ladder: 10 fixed steps = 1.2 m.
        let climbed = player.position.y - start;
        assert!((climbed - 1.2).abs() < 0.01, "climbed {climbed}");
        // Looking down past climbDownThreshold (-0.6) makes forward climb down.
        player.rotation_y = -60.0;
        for _ in 0..5 {
            frame(&mut player, &forward);
        }
        let down = player.position.y - start - climbed;
        assert!((down + 0.6).abs() < 0.01, "down {down}");
        player.rotation_y = 0.0;

        let jump = FrameInput {
            jump_held: true,
            ..forward
        };
        frame(&mut player, &jump);
        assert!(!player.on_ladder && player.movement.enable_gravity);
        for _ in 0..200 {
            frame(&mut player, &FrameInput::default());
        }
        assert!((player.position.y - ground).abs() < 1e-3, "fell back down");
    }

    #[test]
    fn walks_at_walk_speed_and_runs_at_run_speed() {
        let (mut player, world) = player_on_floor();
        let dt = 1.0 / 60.0;
        let walk = FrameInput {
            vertical: 1.0,
            ..Default::default()
        };
        for _ in 0..120 {
            step(&mut player, &world, &walk, dt);
        }
        assert!(player.grounded);
        assert!(
            (player.movement.velocity.z - 5.5).abs() < 1e-3,
            "{:?}",
            player.movement.velocity
        );
        let run = FrameInput {
            run_held: true,
            ..walk
        };
        for _ in 0..120 {
            step(&mut player, &world, &run, dt);
        }
        assert!(player.running);
        assert!(
            (player.movement.velocity.z - 8.5).abs() < 1e-3,
            "{:?}",
            player.movement.velocity
        );
        // Backwards is half the walk speed.
        let back = FrameInput {
            vertical: -1.0,
            ..Default::default()
        };
        for _ in 0..120 {
            step(&mut player, &world, &back, dt);
        }
        assert!(
            (player.movement.velocity.z + 2.75).abs() < 1e-3,
            "{:?}",
            player.movement.velocity
        );
    }

    #[test]
    fn jump_reaches_base_height() {
        let (mut player, world) = player_on_floor();
        let dt = 1.0 / 240.0;
        let idle = FrameInput::default();
        for _ in 0..240 {
            step(&mut player, &world, &idle, dt);
        }
        let ground = player.position.y;
        let jump = FrameInput {
            jump_down: true,
            ..Default::default()
        };
        step(&mut player, &world, &jump, dt);
        let mut apex = ground;
        let mut events = Vec::new();
        for _ in 0..480 {
            step(&mut player, &world, &idle, dt);
            apex = apex.max(player.position.y);
            events.append(&mut player.events);
        }
        // GetKeyDown lasts one frame, so the extra height never applies: apex = baseHeight.
        assert!((apex - ground - 1.6).abs() < 0.05, "apex {}", apex - ground);
        assert!(events.contains(&PlayerEvent::Jump) && events.contains(&PlayerEvent::Land));
        assert!((player.position.y - ground).abs() < 1e-3);
    }

    #[test]
    fn crouch_lowers_controller_and_speed() {
        let (mut player, world) = player_on_floor();
        let dt = 1.0 / 60.0;
        let press = FrameInput {
            crouch_down: true,
            ..Default::default()
        };
        step(&mut player, &world, &press, dt);
        let walk = FrameInput {
            vertical: 1.0,
            ..Default::default()
        };
        for _ in 0..120 {
            step(&mut player, &world, &walk, dt);
        }
        assert!(player.crouch);
        assert!((player.controller.height - 1.5).abs() < 0.011);
        assert!(
            (player.movement.velocity.z - 2.5).abs() < 1e-3,
            "{:?}",
            player.movement.velocity
        );
    }

    /// Largest |offset| of the WalkSway position over `steps` fixed steps with `input`.
    fn sway_extent(
        player: &mut Player,
        world: &CollisionWorld,
        input: &FrameInput,
        steps: usize,
    ) -> f32 {
        let mut extent = 0.0f32;
        for _ in 0..steps {
            player.fixed_update(world, &[], input);
            step(player, world, input, FIXED_DELTA_TIME);
            extent = extent.max(player.weapon_local_position.length());
        }
        extent
    }

    #[test]
    fn walk_sway_moves_the_weapon() {
        let (mut player, world) = player_on_floor();
        let idle = FrameInput::default();
        // Settle, then a full idle period: 2π / 0.005 rad per step = 1257 steps.
        sway_extent(&mut player, &world, &idle, 100);
        let idle_extent = sway_extent(&mut player, &world, &idle, 1300);
        // Amount 0.02 · 0.3 = 0.006 m (x and y each), lerped at 6 · 0.02 per step.
        assert!(idle_extent > 0.003 && idle_extent < 0.0085, "{idle_extent}");
        let walk = FrameInput {
            vertical: 1.0,
            ..Default::default()
        };
        let walk_extent = sway_extent(&mut player, &world, &walk, 300);
        assert!(player.walking);
        assert!(walk_extent > 0.01 && walk_extent < 0.0285, "{walk_extent}");
    }

    #[test]
    fn key_axis_matches_unity_smoothing() {
        let mut axis = KeyAxis::movement();
        let dt = 0.1;
        assert!((axis.update(true, false, dt) - 0.3).abs() < 1e-6);
        // Snap: reversing jumps through zero.
        assert!((axis.update(false, true, dt) + 0.3).abs() < 1e-6);
        assert!((axis.update(false, false, dt) - 0.0).abs() < 1e-6);
    }

    /// Every spawn point of every map: the player settles on the floor and can walk around
    /// without falling through the BSP collision meshes.
    #[test]
    fn spawns_stand_on_every_map() {
        let assets = find_assets_dir().expect("assets");
        let mut checked = 0;
        for entry in std::fs::read_dir(assets.join("data/scenes")).expect("scenes") {
            let path = entry.expect("entry").path();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("name")
                .to_string();
            let scene = crate::scene::load_scene(&assets, &name).expect("scene");
            if scene.spawn_points.is_empty() {
                continue;
            }
            let world = crate::physics::build_world(&assets, &scene).expect("world");
            for (index, spawn) in scene.spawn_points.iter().enumerate() {
                let mut player = Player::from_prefab(&assets, "german").expect("prefab");
                player.spawn(spawn.position, spawn.rotation);
                let dt = 1.0 / 60.0;
                let run = |player: &mut Player, input: FrameInput, frames: usize| {
                    for _ in 0..frames {
                        player.fixed_update(&world, &scene.ladders, &input);
                        step(player, &world, &input, dt);
                    }
                };
                run(&mut player, FrameInput::default(), 120);
                assert!(
                    player.grounded,
                    "{name} spawn {index}: not grounded at {}",
                    player.position
                );
                let settled = player.position.y;
                // Spawn points float 1-3 m above the floor; the player drops onto it and rests
                // with the capsule bottom (0.07 m below the root) one skin width above it.
                let floor = world
                    .sphere_cast(spawn.position + Vec3::Y, 0.1, -Vec3::Y, 50.0)
                    .map(|h| h.point.y)
                    .expect("floor below spawn");
                let rest = floor + player.controller.skin_width
                    - (player.controller.center.y - player.controller.height * 0.5);
                assert!(
                    (settled - rest).abs() < 0.01,
                    "{name} spawn {index}: settled at {settled}, expected {rest}"
                );
                for (h, v) in [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.3)] {
                    let input = FrameInput {
                        horizontal: h,
                        vertical: v,
                        run_held: true,
                        ..Default::default()
                    };
                    run(&mut player, input, 90);
                }
                run(&mut player, FrameInput::default(), 120);
                assert!(
                    player.position.y > settled - 20.0 && player.grounded,
                    "{name} spawn {index}: fell to {} (settled {settled})",
                    player.position
                );
                checked += 1;
            }
        }
        assert!(checked > 100, "only {checked} spawns checked");
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use crate::assets::find_assets_dir;

    #[test]
    #[ignore]
    fn movement_cost() {
        let assets = find_assets_dir().expect("assets");
        for name in ["Bridge", "Crash", "Castle"] {
            let scene = crate::scene::load_scene(&assets, name).expect("scene");
            let world = crate::physics::build_world(&assets, &scene).expect("world");
            let mut player = Player::from_prefab(&assets, "german").expect("prefab");
            let spawn = &scene.spawn_points[0];
            player.spawn(spawn.position, spawn.rotation);
            let input = FrameInput {
                vertical: 1.0,
                run_held: true,
                mouse_x: 3.0,
                ..Default::default()
            };
            let frames = 2000;
            let start = std::time::Instant::now();
            for _ in 0..frames {
                player.fixed_update(&world, &scene.ladders, &input);
                player.update(&world, 1.0 / 60.0, input.vertical, &input);
                player.late_update(&input);
            }
            let per = start.elapsed() / frames;
            eprintln!(
                "{name}: {per:?} per frame, {} triangles",
                world.triangle_count()
            );
        }
    }
}

#[cfg(test)]
mod stuck_sweep {
    use super::*;
    use crate::assets::find_assets_dir;

    /// Diagnostic (`cargo test --release -- --ignored report_unexplained_stops --nocapture`):
    /// runs from every spawn in 16 directions and reports stops with nothing in front at chest
    /// height, with the heights at which something is in front (knee-high walls, beams).
    #[test]
    #[ignore]
    fn report_unexplained_stops() {
        let assets = find_assets_dir().expect("assets");
        let mut names: Vec<String> = std::fs::read_dir(assets.join("data/scenes"))
            .expect("scenes")
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter_map(|n| n.strip_suffix(".json").map(str::to_string))
            .collect();
        names.sort();
        let mut total = 0;
        for name in names {
            let scene = crate::scene::load_scene(&assets, &name).expect("scene");
            if scene.spawn_points.is_empty() {
                continue;
            }
            let world = crate::physics::build_world(&assets, &scene).expect("world");
            let mut stops = 0;
            for spawn in &scene.spawn_points {
                for direction in 0..16 {
                    let mut player = Player::from_prefab(&assets, "german").expect("prefab");
                    let yaw =
                        Quat::from_rotation_y(direction as f32 * std::f32::consts::TAU / 16.0);
                    player.spawn(spawn.position, yaw);
                    let input = FrameInput {
                        vertical: 1.0,
                        run_held: true,
                        ..Default::default()
                    };
                    let mut history = Vec::new();
                    for frame in 0..360 {
                        if frame % 3 == 0 {
                            player.fixed_update(&world, &scene.ladders, &input);
                        }
                        let i = if frame < 60 {
                            FrameInput::default()
                        } else {
                            input
                        };
                        player.update(&world, 1.0 / 144.0, i.vertical, &i);
                        player.late_update(&i);
                        history.push(player.position);
                    }
                    let last = history[history.len() - 1];
                    let before = history[history.len() - 72];
                    let moved = Vec3::new(last.x - before.x, 0.0, last.z - before.z).length();
                    let forward = player.rotation * Vec3::Z;
                    let chest = last + Vec3::Y * 1.2;
                    let wall = world.sphere_cast(chest - forward * 0.3, 0.3, forward, 1.2);
                    if moved < 0.05 && wall.is_none() && player.grounded {
                        stops += 1;
                        // Height of what is in front (a low wall or a head-height beam).
                        let probes: Vec<f32> = [0.2f32, 0.45, 0.7, 0.95, 1.7, 2.1, 2.3]
                            .into_iter()
                            .filter(|h| {
                                let origin = last + Vec3::Y * *h - forward * 0.2;
                                world.sphere_cast(origin, 0.15, forward, 1.0).is_some()
                            })
                            .collect();
                        eprintln!(
                            "{name}: stopped at {last} facing {forward}, blocked at {probes:?} m"
                        );
                    }
                }
            }
            total += stops;
        }
        eprintln!("stops without a chest-height wall: {total}");
    }
}
