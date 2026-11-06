//! Ports of the first-person weapon scripts (spec: `docs/gameplay/weapons.md`): `WeaponManager`
//! (selection and switching), `WeaponScript` (aiming, sway, firing, reloading, camera recoil and
//! its coroutines), `SniperScope` and `WeaponCrosshair`.
//!
//! Coroutines (`StartCoroutine` + `WaitForSeconds`) are explicit continuations: the code before
//! the first `yield` runs inside the call that starts the coroutine, the rest is queued and
//! resumed in `run_coroutines`, which the frame calls after every `Update` (where Unity resumes
//! `WaitForSeconds`). Deactivating a weapon's GameObject drops its queued coroutines, as Unity
//! stops the coroutines of an inactive MonoBehaviour.

use glam::{Mat4, Quat, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{f32_of, ref_of, tag_number};
use crate::character::WeaponAnimation;
pub use crate::gui::GuiQuad;
use crate::physics::CollisionWorld;
use crate::player::Player;
use crate::prefab::PrefabInstance;
use crate::unity::{Random, angle_axis, euler, lerp, look_rotation, slerp_rotation};

/// `RoomMultiplayerMenu.Awooledweops` (= `PhotonNetwork.room.name`): `WeaponManager` only
/// switches to slot n (n ≥ 1) when the room name contains `[WEP0{n+1}]`. Rooms are not
/// implemented yet; this name allows the six weapons of the player prefabs (docs/unknowns.md).
pub const ROOM_NAME: &str = "[WEP01][WEP02][WEP03][WEP04][WEP05][WEP06]";

/// What the weapon scripts read from `CF2Input` in one frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct WeaponInput {
    /// `GetAxis("Mouse X")`, `("Mouse Y")`, `("Vertical")`.
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub vertical: f32,
    /// `CharacterController.velocity.y`.
    pub velocity_y: f32,
    /// `GetMouseButton(0)`, `GetMouseButtonDown(0)`, `GetMouseButtonDown(1)`.
    pub fire_held: bool,
    pub fire_down: bool,
    pub aim_down: bool,
    /// `GetKeyDown(KeyCode.R)`, `GetKeyDown(KeyCode.O)`.
    pub reload_down: bool,
    pub auto_fire_toggle_down: bool,
    /// `GetKeyDown(KeyCode.Alpha0 + n)`.
    pub number_down: [bool; 10],
    /// `GetAxis("Mouse ScrollWheel")`.
    pub scroll: f32,
    /// `GetKey(KeyCode.M)` / `GetKey(KeyCode.N)`: next / previous weapon while held.
    pub next_held: bool,
    pub previous_held: bool,
}

/// Something a weapon script did that the rest of the client carries out.
#[derive(Clone, Debug, PartialEq)]
pub enum WeaponEvent {
    /// `Object.Instantiate(prefab, position, rotation)` of a bullet (`Bullet.doDamage` set) or
    /// of a projectile (`Rigidbody.velocity` set).
    Spawn {
        prefab: String,
        position: Vec3,
        rotation: Quat,
        velocity: Option<Vec3>,
    },
    /// `GetComponent<AudioSource>().clip = clip; .Play()` (asset path) on the WeaponManager's or
    /// a weapon's own AudioSource. `Play` restarts that source.
    Sound { source: SoundSource, clip: String },
}

/// Which AudioSource a weapon sound plays on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundSource {
    /// The `WeaponManager` GameObject's (`takeInAudio`).
    Manager,
    /// The weapon GameObject's (slot index in `allWeapons`).
    Weapon(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunType {
    MachineGun,
    GrenadeLauncher,
    ShotGun,
    Knife,
}

#[derive(Clone, Debug)]
pub struct AimVariables {
    pub aim_position: Vec3,
    pub smooth_time: f32,
    pub to_fov: i32,
    pub aim_bobbing_amount: f32,
    pub play_animation: bool,
}

#[derive(Clone, Debug)]
pub struct MachineGun {
    pub bullet: Option<String>,
    pub muzzle_flash: Option<usize>,
    pub fire_sound: Option<String>,
    pub reload_sound: Option<String>,
    pub point_light: Option<usize>,
    /// `pointLight.enabled` (point lights are not rendered yet).
    pub point_light_enabled: bool,
    pub fire_rate: f32,
    pub bullets_per_clip: i32,
    pub clips: i32,
    pub bullets_left: i32,
    pub reload_time: f32,
    pub no_aim_error_angle: f32,
    pub aim_error_angle: f32,
}

#[derive(Clone, Debug)]
pub struct ShotGun {
    pub bullet: Option<String>,
    pub fractions: i32,
    pub error_angle: f32,
    pub fire_rate: f32,
    pub reload_time: f32,
    pub fire_sound: Option<String>,
    pub reload_sound: Option<String>,
    pub bullets_per_clip: i32,
    pub bullets_left: i32,
    pub clips: i32,
    pub smoke: Option<usize>,
    /// `smoke.Play()` / `Stop()` state (applied to the particle system in `main.rs`).
    pub smoke_playing: bool,
}

#[derive(Clone, Debug)]
pub struct GrenadeLauncher {
    pub projectile: Option<String>,
    pub fire_sound: Option<String>,
    pub reload_sound: Option<String>,
    pub initial_speed: i32,
    pub shot_delay: f32,
    pub wait_before_reload: f32,
    pub reload_time: f32,
    pub ammo_count: i32,
}

#[derive(Clone, Debug)]
pub struct Knife {
    pub bullet: Option<String>,
    pub fire_sound: Option<String>,
    pub fire_rate: f32,
    pub delay_time: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct CameraRecoil {
    pub recoil_power: f32,
    pub shake_amount: f32,
    pub smooth: i32,
}

/// `SniperScope` (Sniper and Launcher): while aimed, `OnGUI` draws `scope_texture` over the
/// screen and deactivates the weapon model.
#[derive(Clone, Debug)]
pub struct SniperScope {
    pub scope_texture: Option<String>,
    pub objects_to_deactivate: Vec<usize>,
}

/// One `WeaponScript` (on a weapon GameObject under `WeaponManager`). Field names follow the
/// original; offsets and decompiled functions are listed in docs/gameplay/weapons.md.
#[derive(Clone, Debug)]
pub struct WeaponScript {
    pub node: usize,
    pub name: String,
    pub gun_type: GunType,
    pub in_sight: bool,
    pub auto_fire: bool,
    pub aimed: bool,
    pub fire: bool,
    pub can_aim: bool,
    pub is_reload: bool,
    pub no_bullets: bool,
    pub recoil: bool,
    pub can_fire: bool,
    pub single_fire: bool,
    pub aim: AimVariables,
    pub default_fov: f32,
    pub default_position: Vec3,
    pub current_fov: f32,
    pub current_position: Vec3,
    pub fire_point: Option<usize>,
    pub shot_gun: ShotGun,
    pub grenade_launcher: GrenadeLauncher,
    pub last_shot: f32,
    pub machine_gun: MachineGun,
    pub error_angle: f32,
    pub next_fire_time: f32,
    pub knife: Knife,
    pub rotation_amplitude: i32,
    pub rotation_smooth: i32,
    pub smooth_max_amount: f32,
    pub smooth_speed: i32,
    pub camera_recoil: CameraRecoil,
    pub cam_default_rotation: Quat,
    /// `camPos`: target of the Main Camera's local rotation. Zero until the first recoil.
    pub cam_pos: Quat,
    pub default_bobbing_amount: f32,
    pub weapon_animation: Option<WeaponAnimation>,
    pub scope: Option<SniperScope>,
    /// `Start` has run (on the first frame the GameObject is active).
    pub started: bool,
}

/// What a resumed coroutine continues with.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Routine {
    /// `WeaponManager.SwitchWeapons`, after `SwitchTime`.
    SwitchWeapons {
        from: usize,
        to: usize,
    },
    /// `machineGunOneShot` after the 1 s pause on an empty clip.
    MachineGunEmpty,
    MachineGunReloadDone,
    MuzzleFlashOff,
    /// End of `machineGunCameraRecoil` / `shotGunCameraRecoil` / `grenadeLauncherCameraRecoil`.
    CameraRecoilDone,
    ShotGunEmpty,
    ShotGunReloadDone,
    SmokeStop,
    /// `grenadeLauncherOneShot` after `shotDelay`.
    GrenadeShot,
    /// `grenadeLauncherReload` after `waitBeforeReload`, then after `reloadTime`.
    GrenadeReloadStart,
    GrenadeReloadDone,
    /// `knifeOneShot` after `delayTime`.
    KnifeHit,
}

#[derive(Clone, Copy, Debug)]
struct Coroutine {
    /// Weapon slot whose `WeaponScript` runs it; `None` = the `WeaponManager`.
    owner: Option<usize>,
    resume_at: f32,
    routine: Routine,
}

/// `WeaponCrosshair` (on `WeaponManager`).
#[derive(Clone, Debug)]
pub struct WeaponCrosshair {
    pub texture: Option<String>,
    pub length: f32,
    pub width: f32,
    pub dynamic_crosshair: bool,
    pub crosshair_responce: f32,
    pub default_distance: f32,
    pub smooth: f32,
    /// `crosshair`: shown (not while aiming).
    pub display: bool,
    pub distance: f32,
}

/// The player's per-frame state the weapon scripts read and write.
pub struct WeaponContext<'a> {
    pub prefab: &'a mut PrefabInstance,
    pub player: &'a mut Player,
    pub world: &'a CollisionWorld,
    pub input: &'a WeaponInput,
    pub dt: f32,
    /// `Time.time`.
    pub time: f32,
}

/// `WeaponManager` and the weapons under it.
pub struct WeaponManager {
    pub switch_time: f32,
    pub take_in_audio: Option<String>,
    /// `allWeapons`: the weapon GameObjects.
    pub all_weapons: Vec<usize>,
    pub index: usize,
    /// `SelectedWeapon` = `allWeapons[index]`, refreshed each `Update`.
    pub selected_weapon: Option<usize>,
    /// The `WeaponScript` of each weapon.
    pub scripts: Vec<Option<WeaponScript>>,
    pub shownowepinroom: f32,
    /// "Available only at FULL game" message timer (set by the room rules, not ported).
    pub shownowepinroom2: f32,
    pub changewep: i32,
    pub index2: i32,
    pub text_delay: f32,
    pub can_switch: bool,
    pub auto_fire: bool,
    /// `SuperWeapons`: `Application.absoluteURL` contains "pacogames" (slot 9 / `[WEP10]`).
    pub super_weapons: bool,
    pub room_name: String,
    pub crosshair: Option<WeaponCrosshair>,
    /// The Main Camera node (`Camera.main`).
    pub main_camera: Option<usize>,
    /// `FPScontroller.tagsToShoot` as numeric tags.
    pub tags_to_shoot: Vec<u32>,
    pub random: Random,
    pub events: Vec<WeaponEvent>,
    coroutines: Vec<Coroutine>,
}

fn int_of(value: &Value) -> i32 {
    value.as_i64().unwrap_or(0) as i32
}

fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}

fn vec3_of(value: &Value) -> Vec3 {
    Vec3::new(
        f32_of(&value["x"]),
        f32_of(&value["y"]),
        f32_of(&value["z"]),
    )
}

/// A prefab/asset ref field (`null` when unassigned).
fn reference(value: &Value) -> Option<String> {
    ref_of(value).map(|(r, _)| r.to_string())
}

/// An `AudioClip` field, as the published asset path.
fn sound(value: &Value) -> Option<String> {
    value["asset"].as_str().map(str::to_string)
}

/// Sets the root and Main Camera world transforms of the local player prefab from the player
/// state, and the `WeaponManager` local position (`WalkSway`), so that transforms read
/// mid-frame (fire points, the weapon) match the original's live `Transform`s.
pub fn sync_avatar(
    prefab: &mut PrefabInstance,
    player: &Player,
    main_camera: Option<usize>,
    weapon_manager: Option<usize>,
) {
    prefab.world_override[0] = Some(Mat4::from_rotation_translation(
        player.rotation,
        player.position,
    ));
    if let Some(node) = main_camera {
        prefab.world_override[node] = Some(Mat4::from_rotation_translation(
            player.camera_rotation(),
            player.camera_position(),
        ));
    }
    if let Some(node) = weapon_manager {
        prefab.nodes[node].local.position = player.weapon_local_position;
    }
}

impl WeaponContext<'_> {
    fn sync_camera(&mut self, main_camera: Option<usize>) {
        sync_avatar(self.prefab, self.player, main_camera, None);
    }

    /// `WeaponScript.GetCenterOfScreen`: where the screen centre ray hits within 300 m, else
    /// 100 m along it.
    fn center_of_screen(&self) -> Vec3 {
        let origin = self.player.camera_position();
        let forward = self.player.camera_rotation() * Vec3::Z;
        match self.world.raycast(origin, forward, 300.0) {
            Some(hit) => hit.point,
            None => origin + forward * 100.0,
        }
    }

    fn position(&self, node: usize) -> Vec3 {
        self.prefab.world_of(node).w_axis.truncate()
    }

    /// `Transform.LookAt(target)` (world up).
    fn look_at(&mut self, node: usize, target: Vec3) {
        if let Some(rotation) = look_rotation(target - self.position(node), Vec3::Y) {
            self.prefab.set_world_rotation(node, rotation);
        }
    }

    fn set_active(&mut self, node: usize, active: bool) {
        self.prefab.nodes[node].active_self = active;
    }
}

impl WeaponScript {
    fn from_fields(prefab: &PrefabInstance, node: usize, f: &Value) -> WeaponScript {
        let a = &f["Aim"];
        let s = &f["ShotGun"];
        let g = &f["grenadeLauncher"];
        let m = &f["machineGun"];
        let k = &f["knife"];
        let r = &f["CameraRecoil"];
        let scope = prefab
            .script_fields(node, "SniperScope")
            .map(|fields| SniperScope {
                scope_texture: sound(&fields["scopeTexture"]),
                objects_to_deactivate: fields["objectsToDeactivate"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| prefab.node_of_ref(r))
                    .collect(),
            });
        WeaponScript {
            node,
            name: f["weaponName"].as_str().unwrap_or_default().to_string(),
            gun_type: match int_of(&f["GunType"]) {
                1 => GunType::GrenadeLauncher,
                2 => GunType::ShotGun,
                3 => GunType::Knife,
                _ => GunType::MachineGun,
            },
            in_sight: flag(&f["inSight"]),
            auto_fire: flag(&f["autoFire"]),
            aimed: flag(&f["aimed"]),
            fire: flag(&f["fire"]),
            can_aim: flag(&f["canAim"]),
            is_reload: flag(&f["isReload"]),
            no_bullets: flag(&f["noBullets"]),
            recoil: flag(&f["Recoil"]),
            can_fire: flag(&f["canFire"]),
            single_fire: flag(&f["singleFire"]),
            aim: AimVariables {
                aim_position: vec3_of(&a["aimPosition"]),
                smooth_time: f32_of(&a["smoothTime"]),
                to_fov: int_of(&a["toFov"]),
                aim_bobbing_amount: f32_of(&a["aimBobbingAmount"]),
                play_animation: flag(&a["playAnimation"]),
            },
            default_fov: 0.0,
            default_position: prefab.nodes[node].local.position,
            current_fov: 0.0,
            current_position: Vec3::ZERO,
            fire_point: prefab.node_of_ref(&f["firePoint"]),
            shot_gun: ShotGun {
                bullet: reference(&s["bullet"]),
                fractions: int_of(&s["fractions"]),
                error_angle: f32_of(&s["errorAngle"]),
                fire_rate: f32_of(&s["fireRate"]),
                reload_time: f32_of(&s["reloadTime"]),
                fire_sound: sound(&s["fireSound"]),
                reload_sound: sound(&s["reloadSound"]),
                bullets_per_clip: int_of(&s["bulletsPerClip"]),
                bullets_left: int_of(&s["bulletsLeft"]),
                clips: int_of(&s["clips"]),
                smoke: prefab.node_of_ref(&s["smoke"]),
                smoke_playing: false,
            },
            grenade_launcher: GrenadeLauncher {
                projectile: reference(&g["projectile"]),
                fire_sound: sound(&g["fireSound"]),
                reload_sound: sound(&g["reloadSound"]),
                initial_speed: int_of(&g["initialSpeed"]),
                shot_delay: f32_of(&g["shotDelay"]),
                wait_before_reload: f32_of(&g["waitBeforeReload"]),
                reload_time: f32_of(&g["reloadTime"]),
                ammo_count: int_of(&g["ammoCount"]),
            },
            last_shot: -10.0,
            machine_gun: MachineGun {
                bullet: reference(&m["bullet"]),
                muzzle_flash: prefab.node_of_ref(&m["muzzleFlash"]),
                fire_sound: sound(&m["fireSound"]),
                reload_sound: sound(&m["reloadSound"]),
                point_light: prefab.node_of_ref(&m["pointLight"]),
                point_light_enabled: false,
                fire_rate: f32_of(&m["fireRate"]),
                bullets_per_clip: int_of(&m["bulletsPerClip"]),
                clips: int_of(&m["clips"]),
                bullets_left: int_of(&m["bulletsLeft"]),
                reload_time: f32_of(&m["reloadTime"]),
                no_aim_error_angle: f32_of(&m["NoAimErrorAngle"]),
                aim_error_angle: f32_of(&m["AimErrorAngle"]),
            },
            error_angle: f32_of(&f["errorAngle"]),
            next_fire_time: 0.0,
            knife: Knife {
                bullet: reference(&k["bullet"]),
                fire_sound: sound(&k["fireSound"]),
                fire_rate: f32_of(&k["fireRate"]),
                delay_time: f32_of(&k["delayTime"]),
            },
            rotation_amplitude: int_of(&f["RotRealism"]["RotationAmplitude"]),
            rotation_smooth: int_of(&f["RotRealism"]["smooth"]),
            smooth_max_amount: f32_of(&f["SmoothMovement"]["maxAmount"]),
            smooth_speed: int_of(&f["SmoothMovement"]["Smooth"]),
            camera_recoil: CameraRecoil {
                recoil_power: f32_of(&r["recoilPower"]),
                shake_amount: f32_of(&r["shakeAmount"]),
                smooth: int_of(&r["smooth"]),
            },
            cam_default_rotation: Quat::IDENTITY,
            cam_pos: Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
            default_bobbing_amount: 0.0,
            weapon_animation: None,
            scope,
            started: false,
        }
    }

    /// `Start` (`f29936`).
    fn start(&mut self, ctx: &mut WeaponContext) {
        self.default_bobbing_amount = ctx
            .player
            .walk_sway
            .as_ref()
            .map_or(0.0, |w| w.bobbing_amount);
        self.cam_default_rotation = ctx.player.camera_local_rotation;
        self.default_fov = ctx.player.camera_fov;
        self.default_position = ctx.prefab.nodes[self.node].local.position;
        match self.gun_type {
            GunType::MachineGun => {
                self.machine_gun.bullets_left = self.machine_gun.bullets_per_clip;
                if let Some(flash) = self.machine_gun.muzzle_flash {
                    ctx.set_active(flash, false);
                }
                self.can_aim = true;
                self.can_fire = true;
            }
            GunType::GrenadeLauncher => {
                self.can_aim = true;
                self.can_fire = true;
            }
            GunType::ShotGun => {
                self.shot_gun.bullets_left = self.shot_gun.bullets_per_clip;
                self.shot_gun.smoke_playing = false;
                self.can_aim = true;
                self.can_fire = true;
            }
            GunType::Knife => {
                self.can_aim = false;
                self.can_fire = true;
            }
        }
        self.started = true;
    }

    /// `selectWeapon` (SendMessage from `WeaponManager`).
    pub fn select_weapon(&mut self, prefab: &mut PrefabInstance) {
        self.can_fire = true;
        self.can_aim = self.gun_type != GunType::Knife;
        self.aimed = false;
        if let Some(animation) = &self.weapon_animation {
            animation.take_in(prefab);
        }
    }

    /// `deselectWeapon`.
    pub fn deselect_weapon(&mut self, prefab: &mut PrefabInstance) {
        self.aimed = false;
        self.is_reload = false;
        self.can_fire = false;
        self.can_aim = false;
        if let Some(animation) = &self.weapon_animation {
            animation.take_out(prefab);
        }
    }
}

/// Work a weapon script queues while it runs.
struct Queue<'a> {
    slot: usize,
    time: f32,
    coroutines: &'a mut Vec<Coroutine>,
    events: &'a mut Vec<WeaponEvent>,
    random: &'a mut Random,
    main_camera: Option<usize>,
    tags_to_shoot: &'a [u32],
}

impl Queue<'_> {
    fn wait(&mut self, seconds: f32, routine: Routine) {
        self.coroutines.push(Coroutine {
            owner: Some(self.slot),
            resume_at: self.time + seconds,
            routine,
        });
    }

    fn sound(&mut self, clip: &Option<String>) {
        if let Some(clip) = clip {
            self.events.push(WeaponEvent::Sound {
                source: SoundSource::Weapon(self.slot),
                clip: clip.clone(),
            });
        }
    }

    fn spawn(&mut self, prefab: &Option<String>, position: Vec3, rotation: Quat) {
        if let Some(prefab) = prefab {
            self.events.push(WeaponEvent::Spawn {
                prefab: prefab.clone(),
                position,
                rotation,
                velocity: None,
            });
        }
    }
}

impl WeaponScript {
    /// `Update` (`f29942`).
    fn update(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        self.raycast_for_zombies(ctx, q);
        self.aiming(ctx);
        self.rotation_realism(ctx);
        self.smooth_move(ctx);
        if self.recoil {
            self.camera_recoil_do(ctx, q);
        }
        match self.gun_type {
            GunType::MachineGun => self.machine_gun_fixed_update(ctx, q),
            GunType::GrenadeLauncher => self.grenade_launcher_fixed_update(ctx, q),
            GunType::ShotGun => self.shot_gun_fixed_update(ctx, q),
            GunType::Knife => {}
        }
        if ctx.player.running {
            self.aimed = false;
        }
        self.input_update(ctx, q);
    }

    /// `RaycastForZombies`: `inSight` = the screen-centre ray hits a collider whose tag is in
    /// `FPScontroller.tagsToShoot` (["Blood"]).
    fn raycast_for_zombies(&mut self, ctx: &WeaponContext, q: &Queue) {
        let origin = ctx.player.camera_position();
        let forward = ctx.player.camera_rotation() * Vec3::Z;
        self.in_sight = ctx
            .world
            .raycast(origin, forward, f32::INFINITY)
            .is_some_and(|hit| q.tags_to_shoot.contains(&hit.tag));
    }

    /// `Aiming` (`f29943`).
    fn aiming(&mut self, ctx: &mut WeaponContext) {
        let bobbing = if self.aimed && !ctx.player.running {
            self.current_position = self.aim.aim_position;
            self.current_fov = self.aim.to_fov as f32;
            self.error_angle = self.machine_gun.aim_error_angle;
            self.aim.aim_bobbing_amount
        } else {
            self.current_position = self.default_position;
            self.current_fov = self.default_fov;
            self.error_angle = self.machine_gun.no_aim_error_angle;
            self.default_bobbing_amount
        };
        if let Some(sway) = &mut ctx.player.walk_sway {
            sway.bobbing_amount = bobbing;
        }
        let t = (ctx.dt / self.aim.smooth_time).clamp(0.0, 1.0);
        let local = &mut ctx.prefab.nodes[self.node].local;
        local.position = local.position.lerp(self.current_position, t);
        ctx.player.camera_fov = lerp(
            ctx.player.camera_fov,
            self.current_fov,
            ctx.dt / self.aim.smooth_time,
        );
    }

    /// `RotationRealism` (`f29944`): tilt against the mouse beyond a 0.1 dead zone.
    fn rotation_realism(&self, ctx: &mut WeaponContext) {
        let amplitude = self.rotation_amplitude as f32;
        let input = ctx.input;
        let angle_y = if input.mouse_x.abs() > 0.1 {
            amplitude * input.mouse_x
        } else {
            0.0
        };
        let angle_x = if input.mouse_y.abs() > 0.1 {
            -amplitude * input.mouse_y
        } else {
            0.0
        };
        let target = euler(Vec3::new(angle_x, angle_y, 0.0));
        let t = (ctx.dt * self.rotation_smooth as f32).clamp(0.0, 1.0);
        let local = &mut ctx.prefab.nodes[self.node].local;
        // Transform.localRotation stores a normalised quaternion.
        local.rotation = local.rotation.slerp(target, t).normalize();
    }

    /// `SmoothMove` (`f29945`): offsets from the vertical speed and the forward axis.
    fn smooth_move(&self, ctx: &mut WeaponContext) {
        let max = self.smooth_max_amount;
        let input = ctx.input;
        let mut offset_y = 0.0;
        if input.velocity_y > max + 1.0 {
            offset_y = -max;
        }
        if input.velocity_y < -max - 1.0 {
            offset_y = max;
        }
        let offset_z = (-input.vertical).clamp(-max, max);
        let local = &mut ctx.prefab.nodes[self.node].local;
        let target = local.position + Vec3::new(0.0, offset_y, offset_z);
        let t = (ctx.dt * self.smooth_speed as f32).clamp(0.0, 1.0);
        local.position = local.position.lerp(target, t);
    }

    /// `cameraRecoilDo` (`f29946`): the Main Camera's local rotation follows `camPos`.
    fn camera_recoil_do(&self, ctx: &mut WeaponContext, q: &Queue) {
        let player = &mut *ctx.player;
        player.camera_local_rotation = slerp_rotation(
            player.camera_local_rotation,
            self.cam_pos,
            ctx.dt * self.camera_recoil.smooth as f32,
        );
        ctx.sync_camera(q.main_camera);
    }

    /// `machineGunFixedUpdate` (called from `Update`).
    fn machine_gun_fixed_update(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.fire && !self.is_reload {
            self.machine_gun_fire(ctx, q);
        } else {
            ctx.player.can_run = true;
            if let Some(flash) = self.machine_gun.muzzle_flash {
                ctx.set_active(flash, false);
            }
        }
        if self.is_reload {
            self.can_aim = false;
        }
    }

    fn grenade_launcher_fixed_update(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.fire && !self.is_reload {
            self.grenade_launcher_fire(ctx, q);
        } else {
            ctx.player.can_run = true;
        }
    }

    fn shot_gun_fixed_update(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.fire && !self.is_reload {
            self.shot_gun_fire(ctx, q);
        } else {
            ctx.player.can_run = true;
        }
        if self.is_reload {
            self.can_aim = false;
        }
    }

    /// `InputUpdate` (`f29950`).
    fn input_update(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        let input = *ctx.input;
        if input.aim_down && self.can_aim && !ctx.player.running {
            self.aimed = !self.aimed;
        }
        self.fire = (input.fire_held && self.can_fire && !self.single_fire)
            || (self.auto_fire && self.in_sight);
        let single_shot = input.fire_down && self.can_fire && !self.is_reload && self.single_fire;
        match self.gun_type {
            GunType::MachineGun => {
                if single_shot {
                    self.machine_gun_fire(ctx, q);
                } else {
                    ctx.player.can_run = true;
                }
            }
            GunType::GrenadeLauncher => {
                if single_shot {
                    self.grenade_launcher_fire(ctx, q);
                }
            }
            GunType::ShotGun => {
                if single_shot {
                    self.shot_gun_fire(ctx, q);
                }
            }
            GunType::Knife => {
                if single_shot {
                    self.knife_one_shot(ctx, q);
                }
            }
        }
        // R: the clip check reads `machineGun.clips` for both gun types.
        if input.reload_down && !self.is_reload && self.machine_gun.clips > 0 {
            if self.gun_type == GunType::MachineGun
                && self.machine_gun.bullets_left != self.machine_gun.bullets_per_clip
            {
                self.machine_gun_reload(ctx, q);
            }
            if self.gun_type == GunType::ShotGun
                && self.shot_gun.bullets_left != self.shot_gun.bullets_per_clip
            {
                self.shot_gun_reload(ctx, q);
            }
        }
    }

    /// The fire-rate loop of `machineGunFire`, `shotGunFire` and `grenadeLauncherFIre`: one shot
    /// per elapsed `rate` since `nextFireTime`, catching up at most to the previous frame.
    fn shots_due(
        &mut self,
        time: f32,
        dt: f32,
        rate: f32,
        mut has_ammo: impl FnMut(&Self) -> bool,
        mut shoot: impl FnMut(&mut Self),
    ) {
        if time - rate > self.next_fire_time {
            self.next_fire_time = time - dt;
        }
        while self.next_fire_time < time && has_ammo(self) {
            shoot(self);
            self.next_fire_time += rate;
        }
    }

    fn machine_gun_fire(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.machine_gun.bullets_left == 0 {
            return;
        }
        let rate = self.machine_gun.fire_rate;
        self.shots_due(
            ctx.time,
            ctx.dt,
            rate,
            |s| s.machine_gun.bullets_left != 0,
            |s| s.machine_gun_one_shot(ctx, q),
        );
    }

    fn shot_gun_fire(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.shot_gun.bullets_left == 0 {
            return;
        }
        let rate = self.shot_gun.fire_rate;
        self.shots_due(
            ctx.time,
            ctx.dt,
            rate,
            |s| s.shot_gun.bullets_left != 0,
            |s| s.shot_gun_one_shot(ctx, q),
        );
    }

    fn grenade_launcher_fire(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.grenade_launcher.ammo_count == 0 || !self.can_fire {
            return;
        }
        let rate = self.grenade_launcher.shot_delay;
        self.shots_due(
            ctx.time,
            ctx.dt,
            rate,
            |s| s.grenade_launcher.ammo_count >= 1,
            |s| s.grenade_launcher_one_shot(ctx, q),
        );
    }

    /// The weapon animation's `Fire`: while aimed only if `Aim.playAnimation`.
    fn fire_animation(&self, ctx: &mut WeaponContext) {
        if self.aimed && !self.aim.play_animation {
            return;
        }
        if let Some(animation) = &self.weapon_animation {
            animation.fire(ctx.prefab);
        }
    }

    /// Recoil kick: `mouseLook.rotationY += recoilPower`.
    fn kick(&self, ctx: &mut WeaponContext) {
        ctx.player.rotation_y += self.camera_recoil.recoil_power;
    }

    /// `machineGunCameraRecoil` up to its `yield`.
    fn machine_gun_camera_recoil(&mut self, q: &mut Queue) {
        let shake = self.camera_recoil.shake_amount;
        let x = q.random.range(0.0, -shake);
        let y = q.random.range(-shake, shake);
        self.cam_pos = euler(Vec3::new(x, y, 0.0));
        q.wait(0.05, Routine::CameraRecoilDone);
    }

    /// `shotGunCameraRecoil` / `grenadeLauncherCameraRecoil` up to their `yield`.
    fn heavy_camera_recoil(&mut self, q: &mut Queue) {
        let shake = self.camera_recoil.shake_amount;
        let x = q.random.range(-shake * 1.5, -shake);
        let y = q.random.range(shake / 3.0, shake * 0.5);
        self.cam_pos = euler(Vec3::new(x, y, 0.0));
        q.wait(0.1, Routine::CameraRecoilDone);
    }

    /// `machineGunOneShot` up to its `yield` (`f61834`).
    fn machine_gun_one_shot(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        let Some(fire_point) = self.fire_point else {
            return;
        };
        let saved = ctx.prefab.world_rotation(fire_point);
        // The spread rotation is replaced by the LookAt right away (only the random draw
        // remains of it).
        let spread = euler(q.random.inside_unit_sphere() * self.error_angle)
            * ctx.prefab.world_rotation(self.node);
        ctx.prefab.set_world_rotation(fire_point, spread);
        let center = ctx.center_of_screen();
        ctx.look_at(fire_point, center);
        let position = ctx.position(fire_point);
        let rotation = ctx.prefab.world_rotation(fire_point);
        q.spawn(&self.machine_gun.bullet, position, rotation);
        ctx.prefab.set_world_rotation(fire_point, saved);
        self.last_shot = ctx.time;
        self.machine_gun.bullets_left -= 1;
        q.sound(&self.machine_gun.fire_sound);
        self.machine_gun_muzzle_flash(ctx, q);
        self.fire_animation(ctx);
        if self.recoil {
            self.kick(ctx);
            self.machine_gun_camera_recoil(q);
        }
        if self.machine_gun.clips > 0 && self.machine_gun.bullets_left == 0 {
            self.no_bullets = true;
            q.wait(1.0, Routine::MachineGunEmpty);
        }
    }

    /// `machineGunMuzzleFlash` up to its `yield`.
    fn machine_gun_muzzle_flash(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if let Some(flash) = self.machine_gun.muzzle_flash {
            let roll = q.random.range_int(0, 359);
            ctx.prefab.nodes[flash].local.rotation = angle_axis(roll as f32, Vec3::NEG_X);
            ctx.set_active(flash, true);
        }
        if self.machine_gun.point_light.is_some() {
            self.machine_gun.point_light_enabled = true;
        }
        q.wait(0.04, Routine::MuzzleFlashOff);
    }

    /// `machineGunReload` up to its `yield`.
    fn machine_gun_reload(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        self.is_reload = true;
        self.aimed = false;
        self.can_aim = false;
        if let Some(animation) = &self.weapon_animation {
            animation.reloading(ctx.prefab, self.machine_gun.reload_time);
        }
        q.sound(&self.machine_gun.reload_sound);
        q.wait(self.machine_gun.reload_time, Routine::MachineGunReloadDone);
    }

    /// `shotGunOneShot` up to its `yield` (`f61840`).
    fn shot_gun_one_shot(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        let Some(fire_point) = self.fire_point else {
            return;
        };
        let center = ctx.center_of_screen();
        ctx.look_at(fire_point, center);
        let saved = ctx.prefab.world_rotation(fire_point);
        for _ in 0..self.shot_gun.fractions {
            let rotation = euler(q.random.inside_unit_sphere() * self.shot_gun.error_angle)
                * ctx.prefab.world_rotation(self.node);
            ctx.prefab.set_world_rotation(fire_point, rotation);
            let position = ctx.position(fire_point);
            let rotation = ctx.prefab.world_rotation(fire_point);
            q.spawn(&self.shot_gun.bullet, position, rotation);
        }
        ctx.prefab.set_world_rotation(fire_point, saved);
        self.last_shot = ctx.time;
        q.sound(&self.shot_gun.fire_sound);
        self.shot_gun.bullets_left -= 1;
        self.fire_animation(ctx);
        // shotGunSmokeEffect up to its yield.
        if self.shot_gun.smoke.is_some() {
            self.shot_gun.smoke_playing = true;
            q.wait(0.3, Routine::SmokeStop);
        }
        if self.recoil {
            self.heavy_camera_recoil(q);
            self.kick(ctx);
        }
        if self.shot_gun.clips > 0 && self.shot_gun.bullets_left == 0 {
            self.no_bullets = true;
            q.wait(1.0, Routine::ShotGunEmpty);
        }
    }

    /// `shotGunReload` up to its `yield` (unlike the machine gun's, it leaves `canAim`).
    fn shot_gun_reload(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        self.is_reload = true;
        self.aimed = false;
        if let Some(animation) = &self.weapon_animation {
            animation.reloading(ctx.prefab, self.shot_gun.reload_time);
        }
        q.sound(&self.shot_gun.reload_sound);
        q.wait(self.shot_gun.reload_time, Routine::ShotGunReloadDone);
    }

    /// `grenadeLauncherOneShot` (`f61823`).
    fn grenade_launcher_one_shot(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if self.grenade_launcher.shot_delay > 0.0 {
            self.fire_animation(ctx);
            if self.recoil {
                self.kick(ctx);
                self.heavy_camera_recoil(q);
            }
            self.grenade_launcher_reload(q);
            q.wait(self.grenade_launcher.shot_delay, Routine::GrenadeShot);
            return;
        }
        self.grenade_shot(ctx, q);
    }

    /// The part of `grenadeLauncherOneShot` after the optional `shotDelay` wait.
    fn grenade_shot(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        let Some(fire_point) = self.fire_point else {
            return;
        };
        if let Some(projectile) = &self.grenade_launcher.projectile {
            // velocity = transform.TransformDirection(0, 0, initialSpeed) of the weapon.
            let speed = self.grenade_launcher.initial_speed as f32;
            let velocity = ctx.prefab.world_rotation(self.node) * Vec3::new(0.0, 0.0, speed);
            q.events.push(WeaponEvent::Spawn {
                prefab: projectile.clone(),
                position: ctx.position(fire_point),
                rotation: ctx.prefab.world_rotation(fire_point),
                velocity: Some(velocity),
            });
        }
        self.last_shot = ctx.time;
        self.grenade_launcher.ammo_count -= 1;
        q.sound(&self.grenade_launcher.fire_sound);
        if self.grenade_launcher.shot_delay == 0.0 {
            self.fire_animation(ctx);
            if self.recoil {
                self.kick(ctx);
                self.heavy_camera_recoil(q);
            }
            if self.grenade_launcher.ammo_count > 0 {
                self.grenade_launcher_reload(q);
            }
        }
    }

    /// `grenadeLauncherReload` up to its first `yield` (it never refills `ammoCount`).
    fn grenade_launcher_reload(&mut self, q: &mut Queue) {
        self.is_reload = true;
        q.wait(
            self.grenade_launcher.wait_before_reload,
            Routine::GrenadeReloadStart,
        );
    }

    /// `knifeOneShot` up to its `yield` (`f61828`).
    fn knife_one_shot(&mut self, ctx: &mut WeaponContext, q: &mut Queue) {
        if ctx.time <= self.knife.fire_rate + self.last_shot {
            return;
        }
        let Some(fire_point) = self.fire_point else {
            return;
        };
        let center = ctx.center_of_screen();
        ctx.look_at(fire_point, center);
        q.sound(&self.knife.fire_sound);
        if let Some(animation) = &self.weapon_animation {
            animation.fire(ctx.prefab);
        }
        q.wait(self.knife.delay_time, Routine::KnifeHit);
    }

    /// Continues a coroutine of this script after its wait.
    fn resume(&mut self, routine: Routine, ctx: &mut WeaponContext, q: &mut Queue) {
        match routine {
            Routine::SwitchWeapons { .. } => {}
            Routine::MachineGunEmpty => {
                if !self.is_reload {
                    self.machine_gun_reload(ctx, q);
                }
            }
            Routine::MachineGunReloadDone => {
                let m = &mut self.machine_gun;
                if m.clips > 0 {
                    let needed = m.bullets_per_clip - m.bullets_left;
                    if m.clips > needed {
                        m.clips -= needed;
                        m.bullets_left += needed;
                    } else {
                        m.bullets_left += m.clips;
                        m.clips = 0;
                    }
                    self.no_bullets = false;
                    self.is_reload = false;
                    self.can_aim = true;
                    ctx.player.can_run = true;
                }
            }
            Routine::MuzzleFlashOff => {
                if let Some(flash) = self.machine_gun.muzzle_flash {
                    ctx.set_active(flash, false);
                }
                if self.machine_gun.point_light.is_some() {
                    self.machine_gun.point_light_enabled = false;
                }
            }
            Routine::CameraRecoilDone => self.cam_pos = self.cam_default_rotation,
            Routine::ShotGunEmpty => {
                if !self.is_reload {
                    self.shot_gun_reload(ctx, q);
                }
            }
            Routine::ShotGunReloadDone => {
                let s = &mut self.shot_gun;
                if s.clips > 0 {
                    let needed = s.bullets_per_clip - s.bullets_left;
                    if s.clips > needed {
                        s.clips -= needed;
                        s.bullets_left += needed;
                    } else {
                        s.bullets_left += s.clips;
                        s.clips = 0;
                    }
                    self.no_bullets = false;
                    self.is_reload = false;
                    self.can_aim = true;
                    ctx.player.can_run = true;
                }
            }
            Routine::SmokeStop => self.shot_gun.smoke_playing = false,
            Routine::GrenadeShot => self.grenade_shot(ctx, q),
            Routine::GrenadeReloadStart => {
                self.aimed = false;
                if let Some(animation) = &self.weapon_animation {
                    animation.reloading(ctx.prefab, self.grenade_launcher.reload_time);
                }
                q.sound(&self.grenade_launcher.reload_sound);
                q.wait(
                    self.grenade_launcher.reload_time,
                    Routine::GrenadeReloadDone,
                );
            }
            Routine::GrenadeReloadDone => self.is_reload = false,
            Routine::KnifeHit => {
                if let Some(fire_point) = self.fire_point {
                    let position = ctx.position(fire_point);
                    let rotation = ctx.prefab.world_rotation(fire_point);
                    q.spawn(&self.knife.bullet, position, rotation);
                }
                self.last_shot = ctx.time;
            }
        }
    }
}

impl WeaponCrosshair {
    fn from_fields(f: &Value) -> WeaponCrosshair {
        WeaponCrosshair {
            texture: sound(&f["crosshairTexture"]),
            length: f32_of(&f["length"]),
            width: f32_of(&f["width"]),
            dynamic_crosshair: flag(&f["dynamicCrosshair"]),
            crosshair_responce: f32_of(&f["crosshairResponce"]),
            default_distance: f32_of(&f["defaultDistance"]),
            smooth: f32_of(&f["smooth"]),
            display: false,
            distance: 0.0,
        }
    }

    /// `Update` (`f29914`) with the selected weapon's script.
    fn update(&mut self, script: Option<&WeaponScript>, ctx: &WeaponContext) {
        let t = ctx.dt / self.smooth;
        let response = self.crosshair_responce;
        if self.dynamic_crosshair {
            let mouse_down = ctx.input.fire_down;
            match script {
                Some(ws) if mouse_down || ws.fire => {
                    if !ws.single_fire {
                        let target = if ws.fire && !ws.no_bullets {
                            response * 2.0
                        } else {
                            self.default_distance
                        };
                        self.distance = lerp(self.distance, target, t);
                    } else if mouse_down && ws.can_fire && !ws.is_reload && !ws.no_bullets {
                        if self.distance < response * 4.0 {
                            self.distance += response;
                        }
                    } else {
                        self.distance = lerp(self.distance, self.default_distance, t);
                    }
                }
                _ => {
                    let target = if ctx.player.walking {
                        response + ctx.player.movement.velocity.length() * 2.0
                    } else {
                        self.default_distance
                    };
                    self.distance = lerp(self.distance, target, t);
                }
            }
        } else {
            self.distance = self.default_distance;
        }
        if let Some(ws) = script {
            self.display = !ws.aimed;
        }
    }

    /// `OnGUI` (`f29915`): four boxes around the centre, red while `inSight`.
    fn on_gui(&self, script: Option<&WeaponScript>, width: f32, height: f32) -> Vec<GuiQuad> {
        let Some(texture) = &self.texture else {
            return Vec::new();
        };
        if self.distance > ((height as i32) / 2) as f32 || !self.display {
            return Vec::new();
        }
        let color = if script.is_some_and(|ws| ws.in_sight) {
            Vec4::new(1.0, 0.0, 0.0, 1.0)
        } else {
            Vec4::ONE
        };
        let (d, l, w) = (self.distance, self.length, self.width);
        [
            [(width - d) * 0.5 - l, (height - w) * 0.5, l, w],
            [(width + d) * 0.5, (height - w) * 0.5, l, w],
            [(width - w) * 0.5, (height - d) * 0.5 - l, w, l],
            [(width - w) * 0.5, (height + d) * 0.5, w, l],
        ]
        .into_iter()
        .map(|rect| GuiQuad::texture(texture, rect, color))
        .collect()
    }
}

/// `WeaponManager.Awake` (`f29921`): activates every child (their `Awake`s run:
/// `WeaponAnimation.Awake` plays Idle, `WeaponScript.Awake` caches its `WeaponAnimation`),
/// deactivates them all again, then `TakeFirstWeapon(allWeapons[index])` activates that weapon
/// and sends it `selectWeapon`; finally every weapon's `autoFire = !manager.autoFire`.
pub fn weapon_manager_awake(
    prefab: &mut PrefabInstance,
    custom_tags: &[String],
) -> Option<WeaponManager> {
    let (node, component) = prefab.find_scripts("WeaponManager").into_iter().next()?;
    let fields = component["fields"].clone();
    let children = prefab.nodes[node].children.clone();

    for &child in &children {
        prefab.nodes[child].active_self = true;
    }
    let mut awoken = Vec::new();
    for &child in &children {
        let mut stack = vec![child];
        while let Some(n) = stack.pop() {
            if !prefab.active_in_hierarchy(n) {
                continue;
            }
            if let Some(c) = prefab
                .scripts(n, "WeaponAnimation")
                .find(|c| c["enabled"] != false)
            {
                awoken.push(WeaponAnimation::from_fields(n, &c["fields"]));
            }
            stack.extend(prefab.nodes[n].children.iter().rev());
        }
    }
    for weapon_animation in &awoken {
        weapon_animation.awake(prefab);
    }

    let all_weapons: Vec<usize> = fields["allWeapons"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| prefab.node_of_ref(r))
        .collect();
    let mut scripts: Vec<Option<WeaponScript>> = all_weapons
        .iter()
        .map(|&w| {
            let fields = prefab.script_fields(w, "WeaponScript")?.clone();
            let mut script = WeaponScript::from_fields(prefab, w, &fields);
            // WeaponScript.Awake (while active): GetComponentInChildren<WeaponAnimation>.
            script.weapon_animation = weapon_animation_in_children(prefab, w);
            Some(script)
        })
        .collect();
    for &child in &children {
        prefab.nodes[child].active_self = false;
    }
    for &weapon in &all_weapons {
        prefab.nodes[weapon].active_self = false;
    }

    let root = 0;
    let tags_to_shoot = prefab
        .script_fields(root, "FPScontroller")
        .and_then(|f| f["tagsToShoot"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|t| tag_number(t.as_str()?, custom_tags))
        .collect();
    let crosshair = prefab
        .script_fields(node, "WeaponCrosshair")
        .map(WeaponCrosshair::from_fields);
    let index = fields["index"].as_u64().unwrap_or(0) as usize;
    let mut manager = WeaponManager {
        switch_time: f32_of(&fields["SwitchTime"]),
        take_in_audio: sound(&fields["takeInAudio"]),
        all_weapons,
        index,
        selected_weapon: None,
        scripts: Vec::new(),
        shownowepinroom: f32_of(&fields["shownowepinroom"]),
        shownowepinroom2: f32_of(&fields["shownowepinroom2"]),
        changewep: int_of(&fields["changewep"]),
        index2: int_of(&fields["index2"]),
        // Private fields: the constructor's values (`f29926`).
        text_delay: 1.0,
        can_switch: false,
        auto_fire: true,
        // The client is not served from pacogames.com.
        super_weapons: false,
        room_name: ROOM_NAME.to_string(),
        crosshair,
        main_camera: prefab.find("Main Camera"),
        tags_to_shoot,
        random: Random::new(1),
        events: Vec::new(),
        coroutines: Vec::new(),
    };
    // TakeFirstWeapon: takeInAudio, SetActive(true), SendMessage("selectWeapon").
    if let Some(&weapon) = manager.all_weapons.get(index) {
        if let Some(clip) = &manager.take_in_audio {
            manager.events.push(WeaponEvent::Sound {
                source: SoundSource::Manager,
                clip: clip.clone(),
            });
        }
        prefab.nodes[weapon].active_self = true;
        if let Some(script) = &mut scripts[index] {
            script.select_weapon(prefab);
        }
        manager.can_switch = true;
    }
    let auto_fire = !manager.auto_fire;
    for script in scripts.iter_mut().flatten() {
        script.auto_fire = auto_fire;
    }
    manager.scripts = scripts;
    Some(manager)
}

/// First enabled `WeaponAnimation` at or below `root` on an active GameObject
/// (`GetComponentInChildren<WeaponAnimation>()`, depth-first).
fn weapon_animation_in_children(prefab: &PrefabInstance, root: usize) -> Option<WeaponAnimation> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if !prefab.active_in_hierarchy(node) {
            continue;
        }
        if let Some(c) = prefab
            .scripts(node, "WeaponAnimation")
            .find(|c| c["enabled"] != false)
        {
            return Some(WeaponAnimation::from_fields(node, &c["fields"]));
        }
        stack.extend(prefab.nodes[node].children.iter().rev());
    }
    None
}

impl WeaponManager {
    pub fn selected_script(&self) -> Option<&WeaponScript> {
        self.scripts.get(self.index)?.as_ref()
    }

    fn has_token(&self, number: usize) -> bool {
        self.room_name.contains(&format!("[WEP{number:02}]"))
    }

    /// One frame of the weapon scripts: the pending `WeaponScript.Start`s, `WeaponManager.Update`,
    /// every active weapon's `WeaponScript.Update`, then
    /// `WeaponCrosshair.Update`. `run_coroutines` follows once the frame's other `Update`s ran.
    pub fn update(&mut self, ctx: &mut WeaponContext) {
        // Pending `Start`s run before any `Update` of the frame.
        for script in self.scripts.iter_mut().flatten() {
            if !script.started && ctx.prefab.active_in_hierarchy(script.node) {
                script.start(ctx);
            }
        }
        self.manager_update(ctx);
        for slot in 0..self.scripts.len() {
            let Some(mut script) = self.scripts[slot].take() else {
                continue;
            };
            if ctx.prefab.active_in_hierarchy(script.node) {
                let mut q = Queue {
                    slot,
                    time: ctx.time,
                    coroutines: &mut self.coroutines,
                    events: &mut self.events,
                    random: &mut self.random,
                    main_camera: self.main_camera,
                    tags_to_shoot: &self.tags_to_shoot,
                };
                script.update(ctx, &mut q);
            }
            self.scripts[slot] = Some(script);
        }
        if let Some(mut crosshair) = self.crosshair.take() {
            crosshair.update(self.selected_script(), ctx);
            self.crosshair = Some(crosshair);
        }
    }

    /// `WeaponManager.Update` (`f29923`).
    fn manager_update(&mut self, ctx: &mut WeaponContext) {
        let input = *ctx.input;
        if input.auto_fire_toggle_down {
            self.auto_fire = !self.auto_fire;
            for script in self.scripts.iter_mut().flatten() {
                script.auto_fire = self.auto_fire;
                self.text_delay = 1.0;
            }
        }
        self.selected_weapon = self.all_weapons.get(self.index).copied();
        if self.all_weapons.len() > 1 {
            // Number keys, or `changewep` from last frame's scroll.
            if (input.number_down[1] || self.changewep == 0) && self.can_switch && self.index != 0 {
                self.switch_to(0, ctx);
            }
            for slot in 1..=5 {
                if !(input.number_down[slot + 1] || self.changewep == slot as i32)
                    || !self.can_switch
                {
                    continue;
                }
                if !self.has_token(slot + 1) {
                    self.shownowepinroom = 40.0;
                } else if self.index != slot {
                    if slot >= self.all_weapons.len() {
                        // `allWeapons[slot]` throws: the rest of Update is skipped.
                        return;
                    }
                    self.switch_to(slot, ctx);
                }
            }
            if (input.number_down[0] || self.changewep == 9)
                && self.can_switch
                && self.all_weapons.len() == 10
                && self.has_token(10)
                && self.super_weapons
                && self.index != 9
            {
                self.switch_to(9, ctx);
            }
            self.changewep = -1;
            if input.scroll > 0.0 || input.next_held {
                self.scroll_up();
            }
            if input.scroll < 0.0 || input.previous_held {
                self.scroll_down();
            }
        }
        self.text_delay -= ctx.dt;
    }

    /// Scroll up / M: the next slot whose room token is present, as `changewep`.
    fn scroll_up(&mut self) {
        self.index2 = self.index as i32;
        for i in 0..=8 {
            if self.index2 != i {
                continue;
            }
            let allowed = self.has_token(i as usize + 2) && (i != 8 || self.super_weapons);
            if allowed && self.changewep == -1 {
                self.changewep = i + 1;
            } else {
                self.index2 = i + 1;
            }
        }
        if self.index2 == 9 {
            if self.has_token(10) && self.super_weapons && self.changewep == -1 {
                self.changewep = 0;
            } else {
                self.index2 = 0;
                self.changewep = 0;
            }
        }
    }

    /// Scroll down / N: the previous slot whose room token is present, wrapping once.
    fn scroll_down(&mut self) {
        self.index2 = self.index as i32;
        let step = |manager: &mut WeaponManager, i: i32, super_check: bool| {
            if manager.index2 != i {
                return;
            }
            let allowed = manager.has_token(i as usize) && (!super_check || manager.super_weapons);
            if allowed && manager.changewep == -1 {
                manager.changewep = i - 1;
            } else {
                manager.index2 = i - 1;
            }
        };
        for i in (1..=9).rev() {
            step(self, i, i == 9);
        }
        if self.index2 == 0 {
            if self.has_token(10) && self.super_weapons && self.changewep == -1 {
                self.changewep = 9;
            } else {
                self.index2 = 9;
            }
        }
        for i in (1..=9).rev() {
            step(self, i, false);
        }
    }

    /// Starts `SwitchWeapons(allWeapons[index], allWeapons[slot])` and sets `index = slot`.
    fn switch_to(&mut self, slot: usize, ctx: &mut WeaponContext) {
        let from = self.index;
        self.index = slot;
        self.can_switch = false;
        let current = self.all_weapons[from];
        if ctx.prefab.nodes[current].active_self
            && let Some(script) = &mut self.scripts[from]
        {
            script.deselect_weapon(ctx.prefab);
        }
        self.coroutines.push(Coroutine {
            owner: None,
            resume_at: ctx.time + self.switch_time,
            routine: Routine::SwitchWeapons { from, to: slot },
        });
    }

    /// Resumes the coroutines whose `WaitForSeconds` elapsed, oldest deadline first.
    pub fn run_coroutines(&mut self, ctx: &mut WeaponContext) {
        loop {
            let due = self
                .coroutines
                .iter()
                .enumerate()
                .filter(|(_, c)| c.resume_at <= ctx.time)
                .min_by(|a, b| a.1.resume_at.total_cmp(&b.1.resume_at))
                .map(|(i, _)| i);
            let Some(i) = due else {
                return;
            };
            let coroutine = self.coroutines.remove(i);
            match (coroutine.owner, coroutine.routine) {
                (None, Routine::SwitchWeapons { from, to }) => self.finish_switch(from, to, ctx),
                (Some(slot), routine) => {
                    let Some(mut script) = self.scripts[slot].take() else {
                        continue;
                    };
                    let mut q = Queue {
                        slot,
                        time: ctx.time,
                        coroutines: &mut self.coroutines,
                        events: &mut self.events,
                        random: &mut self.random,
                        main_camera: self.main_camera,
                        tags_to_shoot: &self.tags_to_shoot,
                    };
                    script.resume(routine, ctx, &mut q);
                    self.scripts[slot] = Some(script);
                }
                (None, _) => {}
            }
        }
    }

    /// The end of `SwitchWeapons` (`f29927`).
    fn finish_switch(&mut self, from: usize, to: usize, ctx: &mut WeaponContext) {
        if let Some(clip) = &self.take_in_audio {
            self.events.push(WeaponEvent::Sound {
                source: SoundSource::Manager,
                clip: clip.clone(),
            });
        }
        ctx.set_active(self.all_weapons[from], false);
        // An inactive MonoBehaviour's coroutines stop.
        self.coroutines.retain(|c| c.owner != Some(from));
        ctx.set_active(self.all_weapons[to], true);
        if let Some(script) = &mut self.scripts[to] {
            script.select_weapon(ctx.prefab);
        }
        self.can_switch = true;
    }

    /// The `OnGUI`s of the weapon scripts for a `width` × `height` screen: `SniperScope` (scope
    /// texture and model hiding while aimed) and `WeaponCrosshair`.
    pub fn on_gui(&self, prefab: &mut PrefabInstance, width: f32, height: f32) -> Vec<GuiQuad> {
        let mut quads = Vec::new();
        for script in self.scripts.iter().flatten() {
            let Some(scope) = &script.scope else {
                continue;
            };
            if !prefab.active_in_hierarchy(script.node) {
                continue;
            }
            if script.aimed {
                if let Some(texture) = &scope.scope_texture {
                    let (w, h) = (width as i32, height as i32);
                    quads.push(GuiQuad::texture(
                        texture,
                        [
                            (w / 2) as f32 - h as f32 * 1.8 * 0.5,
                            // The original computes `h / 2 - h / 2`.
                            0.0,
                            h as f32 * 1.8,
                            h as f32,
                        ],
                        Vec4::ONE,
                    ));
                }
                for &node in &scope.objects_to_deactivate {
                    prefab.nodes[node].active_self = false;
                }
            } else {
                for &node in &scope.objects_to_deactivate {
                    prefab.nodes[node].active_self = true;
                }
            }
        }
        if let Some(crosshair) = &self.crosshair {
            quads.extend(crosshair.on_gui(self.selected_script(), width, height));
        }
        quads
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::ClipLibrary;
    use crate::assets::{find_assets_dir, load_tags};
    use crate::prefab::resource_prefab_index;
    use glam::EulerRot;

    const DT: f32 = 1.0 / 60.0;

    struct Rig {
        prefab: PrefabInstance,
        player: Player,
        world: CollisionWorld,
        manager: WeaponManager,
        time: f32,
    }

    fn rig() -> Rig {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        let mut prefab = PrefabInstance::load(&assets, "german", &mut clips).unwrap();
        let tags = load_tags(&assets).unwrap();
        let manager = weapon_manager_awake(&mut prefab, &tags).unwrap();
        let mut player = Player::from_prefab(&assets, "german").unwrap();
        player.spawn(Vec3::ZERO, Quat::IDENTITY);
        Rig {
            prefab,
            player,
            world: CollisionWorld::new(),
            manager,
            time: 0.0,
        }
    }

    impl Rig {
        /// One frame: the weapon scripts' Update, the coroutines, the animations.
        fn frame(&mut self, input: &WeaponInput) -> Vec<WeaponEvent> {
            self.time += DT;
            let weapon_manager = self.prefab.find("WeaponManager");
            sync_avatar(
                &mut self.prefab,
                &self.player,
                self.manager.main_camera,
                weapon_manager,
            );
            self.prefab.begin_frame(DT);
            let mut ctx = WeaponContext {
                prefab: &mut self.prefab,
                player: &mut self.player,
                world: &self.world,
                input,
                dt: DT,
                time: self.time,
            };
            self.manager.update(&mut ctx);
            self.manager.run_coroutines(&mut ctx);
            self.prefab.update_animations();
            std::mem::take(&mut self.manager.events)
        }

        fn frames(&mut self, n: usize, input: &WeaponInput) -> Vec<WeaponEvent> {
            (0..n).flat_map(|_| self.frame(input)).collect()
        }

        fn script(&self, slot: usize) -> &WeaponScript {
            self.manager.scripts[slot].as_ref().unwrap()
        }

        fn active(&self, slot: usize) -> bool {
            self.prefab.nodes[self.manager.all_weapons[slot]].active_self
        }

        /// Presses number key `key` and waits for the switch to finish.
        fn select(&mut self, key: usize) {
            let mut press = WeaponInput::default();
            press.number_down[key] = true;
            self.frame(&press);
            self.frames(30, &WeaponInput::default());
            assert_eq!(self.manager.index, key - 1);
            assert!(self.active(key - 1));
        }
    }

    fn spawns(events: &[WeaponEvent]) -> Vec<&WeaponEvent> {
        events
            .iter()
            .filter(|e| matches!(e, WeaponEvent::Spawn { .. }))
            .collect()
    }

    #[test]
    fn weapons_load_in_prefab_order() {
        let rig = rig();
        let names: Vec<_> = rig
            .manager
            .scripts
            .iter()
            .map(|s| s.as_ref().unwrap().name.as_str())
            .collect();
        assert_eq!(
            names,
            ["Machete", "Pistol", "M242", "ShotGun", "Sniper", "Launcher"]
        );
        // The constructor sets autoFire; Awake gives weapons autoFire = !manager.autoFire.
        assert!(rig.manager.auto_fire && !rig.script(0).auto_fire);
        assert_eq!(rig.manager.text_delay, 1.0);
        assert_eq!(rig.manager.tags_to_shoot, [20001]);
        assert!(rig.active(0) && !rig.active(1));
        // Bullet refs (component refs too) find their Resources prefab.
        let assets = find_assets_dir().unwrap();
        let index = resource_prefab_index(&assets).unwrap();
        let path = |r: &Option<String>| index[r.as_ref().unwrap()].clone();
        let prefix = "prefabs/resources.assets/";
        assert_eq!(
            path(&rig.script(0).knife.bullet),
            format!("{prefix}KnifeDamage__383.json")
        );
        assert_eq!(
            path(&rig.script(1).machine_gun.bullet),
            format!("{prefix}Bullet__4166.json")
        );
        assert_eq!(
            path(&rig.script(5).grenade_launcher.projectile),
            format!("{prefix}Roket cube__11042.json")
        );
    }

    #[test]
    fn weapon_script_sways_with_mouse_and_movement() {
        let mut rig = rig();
        let node = rig.script(0).node;
        let default = rig.prefab.nodes[node].local.position;
        // Mouse right: the weapon turns towards Euler(0, amplitude · x, 0) = 2° · 1.
        let turning = WeaponInput {
            mouse_x: 1.0,
            ..Default::default()
        };
        rig.frames(120, &turning);
        let (y, x, _) = rig.prefab.nodes[node]
            .local
            .rotation
            .to_euler(EulerRot::YXZ);
        assert!(
            (y.to_degrees() - 2.0).abs() < 1e-3 && x.abs() < 1e-5,
            "{y} {x}"
        );
        // Walking forward pulls the weapon back: Aiming (towards the default at dt / 0.05) and
        // SmoothMove (z - 0.1 at dt · 3) settle in between.
        let walking = WeaponInput {
            vertical: 1.0,
            ..Default::default()
        };
        rig.frames(120, &walking);
        let z = rig.prefab.nodes[node].local.position.z - default.z;
        assert!(z < -0.005 && z > -0.1, "{z}");
        // Idle again: back to the default position and rotation.
        rig.frames(300, &WeaponInput::default());
        let local = rig.prefab.nodes[node].local;
        assert!((local.position - default).length() < 1e-4);
        assert!(
            local.rotation.angle_between(Quat::IDENTITY) < 1e-3,
            "{:?}",
            local.rotation
        );
    }

    #[test]
    fn first_weapon_takes_in_then_stops() {
        let mut rig = rig();
        let weapon = rig.script(0).weapon_animation.clone().unwrap();
        let take_in = weapon.take_in.clone();
        let length = {
            let animation = rig.prefab.animation_on(weapon.node).unwrap();
            assert!(animation.is_playing(&take_in));
            assert!(
                !animation.is_playing(&weapon.idle),
                "Play(TakeIn) stops Idle"
            );
            let state = &animation.states[animation.state_index(&take_in).unwrap()];
            assert_eq!(state.speed, weapon.take_in_out_speed);
            state.clip.length()
        };
        let poses =
            |prefab: &PrefabInstance| -> Vec<_> { prefab.nodes.iter().map(|n| n.local).collect() };
        rig.prefab.update_animations();
        let start = poses(&rig.prefab);
        rig.prefab.begin_frame(0.1);
        rig.prefab.update_animations();
        assert_ne!(start, poses(&rig.prefab), "TakeIn moves the hands");
        // Past the end (length / speed), Once stops the state.
        rig.prefab.begin_frame(length / weapon.take_in_out_speed);
        rig.prefab.update_animations();
        assert!(
            !rig.prefab
                .animation_on(weapon.node)
                .unwrap()
                .is_playing(&take_in)
        );
    }

    #[test]
    fn switching_takes_switch_time() {
        let mut rig = rig();
        let mut press = WeaponInput::default();
        press.number_down[2] = true;
        rig.frame(&press);
        // `index` changes at once; the GameObjects swap when WaitForSeconds(0.4) resumes.
        assert_eq!(rig.manager.index, 1);
        assert!(!rig.manager.can_switch);
        assert!(!rig.script(0).can_fire, "deselectWeapon");
        rig.frames(23, &WeaponInput::default());
        assert!(rig.active(0) && !rig.active(1));
        rig.frames(2, &WeaponInput::default());
        assert!(!rig.active(0) && rig.active(1));
        assert!(rig.manager.can_switch && rig.script(1).can_fire);
        // Slots without a `[WEPnn]` room token (7+) and the current slot do nothing.
        let mut press = WeaponInput::default();
        press.number_down[7] = true;
        press.number_down[2] = true;
        rig.frame(&press);
        assert!(rig.manager.can_switch && rig.manager.index == 1);
    }

    #[test]
    fn scroll_cycles_through_room_slots() {
        let mut rig = rig();
        // Down from slot 0 wraps to the last slot the room allows (5).
        rig.manager.scroll_down();
        assert_eq!(rig.manager.changewep, 5);
        rig.manager.changewep = -1;
        rig.manager.index = 5;
        // Up from 5: no [WEP07]..[WEP10], back to 0.
        rig.manager.scroll_up();
        assert_eq!(rig.manager.changewep, 0);
        rig.manager.changewep = -1;
        rig.manager.index = 2;
        rig.manager.scroll_up();
        assert_eq!(rig.manager.changewep, 3);
        rig.manager.changewep = -1;
        rig.manager.scroll_down();
        assert_eq!(rig.manager.changewep, 1);
    }

    #[test]
    fn pistol_fires_at_fire_rate_and_reloads() {
        let mut rig = rig();
        rig.select(2);
        let fire = WeaponInput {
            fire_held: true,
            fire_down: true,
            ..Default::default()
        };
        let mut events = rig.frame(&fire);
        let hold = WeaponInput {
            fire_held: true,
            ..Default::default()
        };
        events.extend(rig.frames(59, &hold));
        let shots = spawns(&events).len();
        // fireRate 0.2: shots at 0, 0.2, ... within one second.
        assert!((5..=6).contains(&shots), "{shots}");
        let pistol = rig.script(1);
        assert_eq!(pistol.machine_gun.bullets_left, 10 - shots as i32);
        assert!(rig.player.rotation_y > 0.0, "recoil kicks the view up");
        // Empty the clip.
        let mut frames = 0;
        while rig.script(1).machine_gun.bullets_left > 0 {
            rig.frame(&hold);
            frames += 1;
            assert!(frames < 200);
        }
        assert!(rig.script(1).no_bullets);
        let last_shot = rig.script(1).last_shot;
        // Wait 1 s, then reload for reloadTime 1.3 s.
        while rig.time < last_shot + 2.25 {
            rig.frame(&WeaponInput::default());
        }
        assert!(rig.script(1).is_reload && !rig.script(1).can_aim);
        rig.frames(8, &WeaponInput::default());
        let pistol = rig.script(1);
        assert!(!pistol.is_reload && !pistol.no_bullets && pistol.can_aim);
        assert_eq!(pistol.machine_gun.bullets_left, 10);
        assert_eq!(pistol.machine_gun.clips, 50);
    }

    #[test]
    fn knife_hits_after_delay_time() {
        let mut rig = rig();
        rig.frames(30, &WeaponInput::default());
        let click = WeaponInput {
            fire_held: true,
            fire_down: true,
            ..Default::default()
        };
        let events = rig.frame(&click);
        assert!(spawns(&events).is_empty());
        assert!(events.contains(&WeaponEvent::Sound {
            source: SoundSource::Weapon(0),
            clip: "audio/fire melee_fire.m4a".to_string(),
        }));
        // delayTime 0.07: the KnifeDamage probe spawns on the 5th frame (4/60 < 0.07 < 5/60).
        let mut spawned_at = None;
        for frame in 1..10 {
            if !spawns(&rig.frame(&WeaponInput::default())).is_empty() {
                spawned_at = Some(frame);
                break;
            }
        }
        assert_eq!(spawned_at, Some(5));
        // fireRate 0.2 from the hit: an early click is ignored.
        assert!(rig.frame(&click).is_empty());
    }

    #[test]
    fn launcher_fires_its_single_rocket() {
        let mut rig = rig();
        rig.select(6);
        let click = WeaponInput {
            fire_held: true,
            fire_down: true,
            ..Default::default()
        };
        let events = rig.frame(&click);
        let spawned = spawns(&events);
        assert_eq!(spawned.len(), 1);
        let WeaponEvent::Spawn { velocity, .. } = spawned[0] else {
            unreachable!()
        };
        let launcher = rig.script(5);
        let speed = launcher.grenade_launcher.initial_speed as f32;
        assert!((velocity.unwrap().length() - speed).abs() < 1e-3);
        assert_eq!(launcher.grenade_launcher.ammo_count, 0);
        // ammoCount 1, no reload: nothing more to fire.
        rig.frames(120, &WeaponInput::default());
        assert!(spawns(&rig.frame(&click)).is_empty());
    }

    #[test]
    fn crosshair_settles_and_sniper_scope_replaces_it() {
        let mut rig = rig();
        rig.frames(120, &WeaponInput::default());
        let quads = rig.manager.on_gui(&mut rig.prefab, 1280.0, 720.0);
        assert_eq!(quads.len(), 4);
        // defaultDistance 20, length 10, width 1.
        assert!(
            (quads[0].rect[0] - 620.0).abs() < 0.05 && quads[0].rect[1] == 359.5,
            "{:?}",
            quads[0]
        );
        rig.select(5);
        let aim = WeaponInput {
            aim_down: true,
            ..Default::default()
        };
        rig.frame(&aim);
        rig.frames(60, &WeaponInput::default());
        let sniper = rig.script(4);
        assert!(sniper.aimed);
        assert!((rig.player.camera_fov - sniper.aim.to_fov as f32).abs() < 1e-2);
        let quads = rig.manager.on_gui(&mut rig.prefab, 1280.0, 720.0);
        assert_eq!(quads.len(), 1, "scope only, no crosshair");
        assert_eq!(quads[0].rect, [640.0 - 648.0, 0.0, 1296.0, 720.0]);
        let hidden = &rig.script(4).scope.as_ref().unwrap().objects_to_deactivate;
        assert!(hidden.iter().all(|&n| !rig.prefab.nodes[n].active_self));
    }
}
