//! Another player's copy of a player prefab, as this client runs it (spec:
//! `docs/gameplay/remote-players.md`). Everything here is driven by what the network carries:
//! the `OnPhotonSerializeView` stream (`NetState`) and the `SyncWeaponsRPC` / `DoDamage` RPCs.
//!
//! Ported: `PlayerNetworkController`'s remote branch (`f37893` Awake, `f37894`
//! OnPhotonSerializeView, `f37896` Update, `f37902` SyncWeaponsRPC), `PlayerDamage`'s remote side
//! (`Awake` adds a `HitBox` per hit-box entry; `HitBox.ApplyDamage` → `TotalDamage` → `DoDamage`,
//! hit marker in `OnGUI`), `WeaponSync` (Start and the `sync*` coroutines) and `DrawPlayerName`.
//! `HeadLookController` stays on the copy, but the prefabs give it no bending segments, which
//! makes its `LateUpdate` an identity: not ported.

use glam::{Mat4, Quat, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{f32_of, ref_of};
use crate::audio::{Audio, SourceSettings};
use crate::gui::{Gui, GuiFont, GuiStyle};
use crate::physics::{Capsule, DynamicCollider, HitShape};
use crate::prefab::PrefabInstance;
use crate::projectiles::{SpawnFlags, Target, Targets};
use crate::render::View;
use crate::unity::{Random, euler, lerp};
use crate::weapons::{GunType, SyncFire};

/// A Photon actor (player) number.
pub type ActorId = i32;

/// `PlayerNetworkController.OnPhotonSerializeView` payload, in stream order.
#[derive(Clone, Debug, PartialEq)]
pub struct NetState {
    /// `transform.position` / `rotation` of the player root.
    pub position: Vec3,
    pub rotation: Quat,
    /// `gameObject.name` (`SpawnPlayer` names the object after the Photon player).
    pub name: String,
    /// `lookTarget.position` / `rotation` (world).
    pub look_target_position: Vec3,
    pub look_target_rotation: Quat,
    /// `PhotonNetwork.player.customProperties["TeamName"]`.
    pub team: String,
    /// `characterAnimation.animationSyncHelper`.
    pub animation: String,
    /// `characterAnimation.animationForHands`.
    pub hands_animation: String,
    /// `characterAnimation.activeWeapon`.
    pub weapon: String,
    /// `characterAnimation.animationType`.
    pub animation_type: String,
    /// `IsSpawnProtected`.
    pub spawn_protected: bool,
}

/// A collider `PlayerDamage.Awake` gave a `HitBox` (`maxDamage` = the entry's `damage`).
#[derive(Clone, Debug)]
pub struct HitBox {
    pub node: usize,
    pub max_damage: f32,
    pub shape: HitShape,
}

/// `WeaponSync` on a third-person weapon.
#[derive(Clone, Debug)]
struct WeaponSync {
    node: usize,
    /// `firstPersonWeapon`'s `GunType` and the fields `Start` copies from it.
    gun_type: Option<GunType>,
    fire_point: Option<usize>,
    bullet: Option<String>,
    projectile: Option<String>,
    fire_audio: Option<String>,
    /// Node whose renderer is the muzzle flash.
    muzzle_flash: Option<usize>,
    source: Option<u64>,
    started: bool,
    /// `WaitForSeconds(0.04)` of the flash.
    flash_off_at: Option<f32>,
    /// `firstPersonWeapon` fields, read at `Start`.
    first_person: Value,
}

/// The remote copy's `PlayerDamage` state.
#[derive(Clone, Debug)]
pub struct RemoteDamage {
    pub current_hp: f32,
    pub disable_damage: bool,
    pub we_killed: bool,
    /// Hit marker alpha (`fadeValue`).
    pub fade_value: f32,
    pub fade_value_b: f32,
    pub hit_mark_texture: Option<String>,
    pub ragdoll: Option<String>,
}

/// `DoDamage` killed the copy (`currentHp < 0`): the root's children were deactivated; the
/// caller instantiates `ragdoll` at the transform and sends it `clearCamera`.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteDeath {
    pub ragdoll: Option<String>,
    pub position: Vec3,
    pub rotation: Quat,
    /// `gameObject.name`.
    pub name: String,
    /// The damager is this client's player: `AddKillNotification`, Kills + 1, team score.
    pub killed_by_local: bool,
}

/// Something `WeaponSync` did.
#[derive(Clone, Debug, PartialEq)]
pub enum SyncEvent {
    /// `Instantiate` of a bullet (`doDamage = false`) or a projectile (`isRemote = true`, force
    /// along the fire point, collisions with the shooter ignored).
    Spawn {
        prefab: String,
        position: Vec3,
        rotation: Quat,
        velocity: Option<Vec3>,
        flags: SpawnFlags,
    },
    /// `fireAudio.Play()` on the weapon's AudioSource.
    Sound { source: u64, clip: String },
}

/// What the copy needs to know about this client and the room.
#[derive(Clone, Copy, Debug)]
pub struct LocalView<'a> {
    /// `rmm.gameMode`.
    pub game_mode: &'a str,
    /// This client's `TeamName` property.
    pub team: &'a str,
}

/// The `CharacterController` of the root (rockets collide with it).
#[derive(Clone, Copy, Debug)]
struct ControllerShape {
    center: Vec3,
    height: f32,
    radius: f32,
}

pub struct RemotePlayer {
    pub actor: ActorId,
    /// Index of the player prefab kind (the caller's list).
    pub kind: usize,
    pub prefab: PrefabInstance,
    // PlayerNetworkController (remote fields).
    correct_position: Vec3,
    correct_rotation: Quat,
    look_target_position: Vec3,
    look_target_rotation: Quat,
    /// `PlayerName`.
    pub player_name: String,
    /// `playerTeam`.
    pub team: String,
    current_animation: String,
    blended_animation: String,
    current_weapon_name: String,
    anim_type: String,
    pub spawn_protected: bool,
    /// `MovementAnimation` / `currentBlendedAnimation` / `prevWeap` (null until first set).
    movement_animation: Option<String>,
    current_blended_animation: Option<String>,
    prev_weapon: Option<String>,
    mixed_animations: Vec<String>,
    current_active_weapon: Option<usize>,
    look_target: Option<usize>,
    third_person_weapons: Option<usize>,
    shoulders: Vec<usize>,
    /// The headLookController's GameObject, which carries the body `Animation`.
    animation_node: Option<usize>,
    /// `animationSpeed`: walk, run, crouch, prone.
    animation_speed: [f32; 4],
    /// `gameObject.name`.
    pub name: String,
    // PlayerDamage / HitBox.
    pub damage: RemoteDamage,
    pub hit_boxes: Vec<HitBox>,
    // WeaponSync.
    weapon_syncs: Vec<WeaponSync>,
    // DrawPlayerName.
    pub draw_name_enabled: bool,
    position_display: usize,
    draw_name: String,
    controller: Option<ControllerShape>,
    random: Random,
}

/// `Vector3.Lerp` / `Quaternion.Lerp` (t clamped to [0, 1]).
fn lerp_vec(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a.lerp(b, t.clamp(0.0, 1.0))
}

fn lerp_quat(a: Quat, b: Quat, t: f32) -> Quat {
    a.lerp(b, t.clamp(0.0, 1.0)).normalize()
}

fn vec3_of(value: &Value) -> Vec3 {
    Vec3::new(
        f32_of(&value["x"]),
        f32_of(&value["y"]),
        f32_of(&value["z"]),
    )
}

/// `WeaponScript.GunType` of a first-person weapon's fields.
fn gun_type_of(fields: &Value) -> GunType {
    match fields["GunType"].as_i64().unwrap_or(0) {
        1 => GunType::GrenadeLauncher,
        2 => GunType::ShotGun,
        3 => GunType::Knife,
        _ => GunType::MachineGun,
    }
}

impl RemotePlayer {
    /// `PhotonNetwork.Instantiate` on this client: the prefab at `position`/`rotation`, then the
    /// remote branches of the scripts' `Awake`, Animation `playAutomatically` and the
    /// AudioSources (`audio` is `None` for a silent copy).
    pub fn new(
        pristine: &PrefabInstance,
        kind: usize,
        actor: ActorId,
        position: Vec3,
        rotation: Quat,
        audio: Option<&mut Audio>,
    ) -> RemotePlayer {
        let mut prefab = pristine.clone();
        prefab.nodes[0].local.position = position;
        prefab.nodes[0].local.rotation = rotation;
        let pnc = prefab
            .script_fields(0, "PlayerNetworkController")
            .cloned()
            .unwrap_or(Value::Null);
        // GetComponentInChildren<WeaponManager>(): every child deactivated.
        if let Some((manager, _)) = prefab
            .find_scripts("WeaponManager")
            .into_iter()
            .find(|(n, _)| prefab.active_in_hierarchy(*n))
        {
            for child in prefab.nodes[manager].children.clone() {
                prefab.set_active(child, false);
            }
        }
        prefab.set_active_refs(&pnc["remoteObjectsToDeactivate"], false);
        let look_target = prefab.node_of_ref(&pnc["lookTarget"]);
        if let Some(node) = look_target {
            prefab.set_active(node, true);
        }
        let shoulders = [&pnc["shoulderR"], &pnc["shoulderL"]]
            .into_iter()
            .filter_map(|r| prefab.node_of_ref(r))
            .collect();
        let speed = &pnc["animationSpeed"];
        let animation_speed = [
            f32_of(&speed["walkSpeed"]),
            f32_of(&speed["runSpeed"]),
            f32_of(&speed["crouchSpeed"]),
            f32_of(&speed["proneSpeed"]),
        ];
        let damage_fields = prefab
            .script_fields(0, "PlayerDamage")
            .cloned()
            .unwrap_or(Value::Null);
        let hit_boxes = damage_fields["hitBoxes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entry| hit_box(&prefab, entry))
            .collect();
        let reference = |name: &str| ref_of(&damage_fields[name]).map(|(r, _)| r.to_string());
        let damage = RemoteDamage {
            current_hp: f32_of(&damage_fields["hp"]),
            disable_damage: damage_fields["disableDamage"].as_bool().unwrap_or(false),
            we_killed: false,
            fade_value: 0.0,
            fade_value_b: 0.0,
            hit_mark_texture: damage_fields["hitMarkTexture"]["asset"]
                .as_str()
                .map(str::to_string),
            ragdoll: reference("ragdoll"),
        };
        let position_display = prefab
            .script_fields(0, "DrawPlayerName")
            .and_then(|f| prefab.node_of_ref(&f["positionDisplay"]))
            .unwrap_or(0);
        let mut weapon_syncs = Vec::new();
        let mut audio = audio;
        for (node, component) in prefab.find_scripts("WeaponSync") {
            let f = &component["fields"];
            let first_person = prefab
                .node_of_ref(&f["firstPersonWeapon"])
                .and_then(|n| prefab.script_fields(n, "WeaponScript"))
                .cloned()
                .unwrap_or(Value::Null);
            let source = match (&mut audio, prefab.components(node, "AudioSource").next()) {
                (Some(audio), Some(data)) => {
                    let mut settings = SourceSettings::from_component(data);
                    // Start: `fireAudio.playOnAwake = false`.
                    settings.play_on_awake = false;
                    Some(audio.add_source(settings, prefab.world_of(node).w_axis.truncate()))
                }
                _ => None,
            };
            // The serialized values stay when `Start` finds nothing to copy.
            let serialized = |field: &str| ref_of(&f[field]).map(|(r, _)| r.to_string());
            weapon_syncs.push(WeaponSync {
                node,
                gun_type: None,
                fire_point: prefab.node_of_ref(&f["firePoint"]),
                bullet: serialized("bullet"),
                projectile: serialized("projectile"),
                fire_audio: f["fireAudio"]["asset"].as_str().map(str::to_string),
                muzzle_flash: prefab.node_of_ref(&f["muzzleFlash"]),
                source,
                started: false,
                flash_off_at: None,
                first_person,
            });
        }
        let controller = prefab
            .component(0, "CharacterController")
            .map(|c| ControllerShape {
                center: vec3_of(&c["m_Center"]),
                height: f32_of(&c["m_Height"]),
                radius: f32_of(&c["m_Radius"]),
            });
        prefab.play_automatically();
        prefab.update_world();
        RemotePlayer {
            actor,
            kind,
            animation_node: prefab.node_of_ref(&pnc["headLookController"]),
            third_person_weapons: prefab.node_of_ref(&pnc["thirdPersonWeapons"]),
            prefab,
            // Constructor values until the first OnPhotonSerializeView.
            correct_position: Vec3::new(0.0, -100.0, 0.0),
            correct_rotation: Quat::IDENTITY,
            look_target_position: Vec3::ZERO,
            look_target_rotation: Quat::IDENTITY,
            player_name: String::new(),
            team: String::new(),
            current_animation: String::new(),
            blended_animation: String::new(),
            current_weapon_name: String::new(),
            anim_type: String::new(),
            spawn_protected: pnc["IsSpawnProtected"].as_bool().unwrap_or(true),
            movement_animation: None,
            current_blended_animation: None,
            prev_weapon: None,
            mixed_animations: Vec::new(),
            current_active_weapon: None,
            look_target,
            shoulders,
            animation_speed,
            name: String::new(),
            damage,
            hit_boxes,
            weapon_syncs,
            draw_name_enabled: false,
            position_display,
            draw_name: String::new(),
            controller,
            random: Random::new(actor as u32 ^ 0x5eed),
        }
    }

    /// `OnPhotonSerializeView` while reading.
    pub fn receive(&mut self, state: &NetState) {
        self.correct_position = state.position;
        self.correct_rotation = state.rotation;
        self.player_name = state.name.clone();
        self.look_target_position = state.look_target_position;
        self.look_target_rotation = state.look_target_rotation;
        self.team = state.team.clone();
        self.current_animation = state.animation.clone();
        self.blended_animation = state.hands_animation.clone();
        self.current_weapon_name = state.weapon.clone();
        self.anim_type = state.animation_type.clone();
        self.spawn_protected = state.spawn_protected;
    }

    /// Every AudioSource of the copy (to remove them with it).
    pub fn sources(&self) -> impl Iterator<Item = u64> + '_ {
        self.weapon_syncs.iter().filter_map(|s| s.source)
    }

    /// One frame: the Start of newly active `WeaponSync`s, then the scripts' `Update`s
    /// (`PlayerNetworkController`, `PlayerDamage`, `DrawPlayerName`) and the flash coroutines.
    pub fn update(&mut self, dt: f32, time: f32, local: LocalView) {
        for index in 0..self.weapon_syncs.len() {
            let node = self.weapon_syncs[index].node;
            if !self.weapon_syncs[index].started && self.prefab.active_in_hierarchy(node) {
                self.weapon_sync_start(index);
            }
        }
        self.network_update(dt, local);
        self.damage.fade_value = lerp(self.damage.fade_value, 0.0, dt * 2.0);
        self.damage.fade_value_b = lerp(self.damage.fade_value_b, 0.0, dt * 2.0);
        self.draw_name = self.name.clone();
        for index in 0..self.weapon_syncs.len() {
            if let Some(at) = self.weapon_syncs[index].flash_off_at
                && time >= at
                && self
                    .prefab
                    .active_in_hierarchy(self.weapon_syncs[index].node)
            {
                self.weapon_syncs[index].flash_off_at = None;
                self.set_flash(index, false);
            }
        }
    }

    fn set_flash(&mut self, index: usize, enabled: bool) {
        if let Some(node) = self.weapon_syncs[index].muzzle_flash {
            for renderer in &mut self.prefab.renderers {
                if renderer.node == node {
                    renderer.enabled = enabled;
                }
            }
        }
    }

    /// `WeaponSync.Start` (`f61818`, CrazyGames `Azg`): the muzzle flash renderer off, the shot
    /// settings copied from the first-person weapon by its `GunType`. Each value is copied only
    /// when the first-person one is set (`if (x) field = x`), so the serialized one stays
    /// otherwise.
    fn weapon_sync_start(&mut self, index: usize) {
        self.set_flash(index, false);
        let sync = &mut self.weapon_syncs[index];
        sync.started = true;
        let f = &sync.first_person;
        if f.is_null() {
            return;
        }
        let reference = |v: &Value| ref_of(v).map(|(r, _)| r.to_string());
        let sound = |v: &Value| v["asset"].as_str().map(str::to_string);
        let gun_type = gun_type_of(f);
        sync.gun_type = Some(gun_type);
        let (shot, audio) = match gun_type {
            GunType::MachineGun => (
                reference(&f["machineGun"]["bullet"]),
                sound(&f["machineGun"]["fireSound"]),
            ),
            GunType::ShotGun => (
                reference(&f["ShotGun"]["bullet"]),
                sound(&f["ShotGun"]["fireSound"]),
            ),
            GunType::GrenadeLauncher => (
                reference(&f["grenadeLauncher"]["projectile"]),
                sound(&f["grenadeLauncher"]["fireSound"]),
            ),
            GunType::Knife => (None, sound(&f["knife"]["fireSound"])),
        };
        if let Some(shot) = shot {
            if gun_type == GunType::GrenadeLauncher {
                sync.projectile = Some(shot);
            } else {
                sync.bullet = Some(shot);
            }
        }
        if let Some(audio) = audio {
            sync.fire_audio = Some(audio);
        }
    }

    /// `PlayerNetworkController.Update`, remote branch.
    fn network_update(&mut self, dt: f32, local: LocalView) {
        let root = &mut self.prefab.nodes[0].local;
        root.position = lerp_vec(root.position, self.correct_position, dt * 8.0);
        root.rotation = lerp_quat(root.rotation, self.correct_rotation, dt * 8.0);
        if let Some(target) = self.look_target {
            let parent = self.prefab.nodes[target]
                .parent
                .map_or(Mat4::IDENTITY, |p| self.prefab.world_of(p));
            let current = self.prefab.world_of(target).w_axis.truncate();
            let position = lerp_vec(current, self.look_target_position, dt * 8.0);
            self.prefab.nodes[target].local.position = parent.inverse().transform_point3(position);
            self.prefab
                .set_world_rotation(target, self.look_target_rotation);
        }
        if self.name != self.player_name {
            self.name = self.player_name.clone();
        }
        if local.game_mode == "TDM" {
            let teammate = self.team == local.team;
            self.damage.disable_damage = teammate;
            self.draw_name_enabled = teammate;
        } else {
            self.draw_name_enabled = false;
        }
        let Some(animation_node) = self.animation_node else {
            return;
        };
        if self.movement_animation.as_deref() != Some(&self.current_animation) {
            let name = self.current_animation.clone();
            self.movement_animation = Some(name.clone());
            let speed = match self.anim_type.as_str() {
                "Walking" => Some(self.animation_speed[0]),
                "Running" => Some(self.animation_speed[1]),
                "Crouch" => Some(self.animation_speed[2]),
                "Prone" => Some(self.animation_speed[3]),
                _ => None,
            };
            let Some(animation) = self.prefab.animation_on(animation_node) else {
                return;
            };
            let exists = animation.state_index(&name).is_some();
            if let Some(speed) = speed {
                if !exists {
                    // `animation[name].speed` on a missing state throws: Update ends here.
                    return;
                }
                animation.set_speed(&name, speed);
            }
            if let Some(state) = animation.state_mut(&name) {
                state.layer = 1;
            }
            animation.set_state_wrap_mode(&name, crate::anim::WrapMode::Loop);
        }
        let movement = self.movement_animation.clone().unwrap_or_default();
        if let Some(animation) = self.prefab.animation_on(animation_node) {
            // CrossFade(name): fadeLength 0.3, StopSameLayer (no-op for a missing state).
            animation.cross_fade(&movement, 0.3);
        }
        if self.current_blended_animation.as_deref() != Some(&self.blended_animation) {
            let name = self.blended_animation.clone();
            self.current_blended_animation = Some(name.clone());
            if !self.mixed_animations.contains(&name) && name != "Null" {
                let mut mixing = Vec::new();
                for &shoulder in &self.shoulders {
                    mixing.extend(self.prefab.subtree(shoulder));
                }
                if let Some(animation) = self.prefab.animation_on(animation_node)
                    && animation.state_index(&name).is_some()
                {
                    if let Some(state) = animation.state_mut(&name) {
                        state.layer = 4;
                    }
                    animation.set_state_wrap_mode(&name, crate::anim::WrapMode::Loop);
                    animation.add_mixing_transform(&name, &mixing);
                    self.mixed_animations.push(name);
                }
            }
        }
        let blended = self.current_blended_animation.clone().unwrap_or_default();
        if blended != "Null"
            && let Some(animation) = self.prefab.animation_on(animation_node)
        {
            animation.play(&blended);
        }
        if self.prev_weapon.as_deref() != Some(&self.current_weapon_name) {
            if let Some(weapons) = self.third_person_weapons {
                for child in self.prefab.nodes[weapons].children.clone() {
                    let matches = self.prefab.nodes[child].name == self.current_weapon_name;
                    self.prefab.set_active(child, matches);
                    if matches {
                        self.current_active_weapon = Some(child);
                    }
                }
            }
            self.prev_weapon = Some(self.current_weapon_name.clone());
        }
    }

    /// `SyncWeaponsRPC(functionName, value)`: `currentActiveWeapon.SendMessage(functionName,
    /// value, DontRequireReceiver)` on its `WeaponSync`. `WeaponSync.syncKnife(float temp)` takes
    /// the value, so the knife arrives too and `knifeOneHit` plays `fireAudio` (CrazyGames `Jzg` /
    /// `Kzg`; Build-7's `WeaponSync` has the same signature). `owner` is this copy's index in the
    /// caller's target list (for the projectile's `IgnoreCollision`).
    pub fn sync_weapons_rpc(&mut self, sync: SyncFire, time: f32, owner: usize) -> Vec<SyncEvent> {
        let mut events = Vec::new();
        let Some(weapon) = self.current_active_weapon else {
            return events;
        };
        let Some(index) = self.weapon_syncs.iter().position(|s| s.node == weapon) else {
            return events;
        };
        if !self.prefab.active_in_hierarchy(weapon) {
            return events;
        }
        match sync {
            SyncFire::MachineGun(error_angle) => {
                self.shots(index, 1, error_angle, time, &mut events);
            }
            SyncFire::ShotGun(fractions) => {
                self.shots(index, fractions as i32, 3.0, time, &mut events);
            }
            SyncFire::GrenadeLauncher(speed) => self.grenade(index, speed, owner, &mut events),
            // syncKnife → knifeOneHit: `clip = fireAudio; Play()` (no StopAllCoroutines).
            SyncFire::Knife => self.play_fire_audio(index, &mut events),
        }
        events
    }

    /// `syncMachineGun` / `syncShotGun`: `StopAllCoroutines`, then `machineGunShot` /
    /// `shotGunShot` up to the flash's `WaitForSeconds(0.04)`.
    fn shots(
        &mut self,
        index: usize,
        count: i32,
        spread: f32,
        time: f32,
        events: &mut Vec<SyncEvent>,
    ) {
        self.weapon_syncs[index].flash_off_at = None;
        let sync = self.weapon_syncs[index].clone();
        let Some(fire_point) = sync.fire_point else {
            return;
        };
        // `Instantiate(null)` throws before anything else happens.
        let Some(bullet) = sync.bullet.clone() else {
            return;
        };
        let saved = self.prefab.world_rotation(fire_point);
        for _ in 0..count {
            let rotation = euler(self.random.inside_unit_sphere() * spread) * saved;
            self.prefab.set_world_rotation(fire_point, rotation);
            events.push(SyncEvent::Spawn {
                prefab: bullet.clone(),
                position: self.prefab.world_of(fire_point).w_axis.truncate(),
                rotation: self.prefab.world_rotation(fire_point),
                velocity: None,
                flags: SpawnFlags {
                    do_damage: false,
                    is_remote: false,
                    ignore_owner: None,
                },
            });
            self.prefab.set_world_rotation(fire_point, saved);
        }
        self.play_fire_audio(index, events);
        if sync.muzzle_flash.is_some() {
            self.set_flash(index, true);
            self.weapon_syncs[index].flash_off_at = Some(time + 0.04);
        }
    }

    /// `syncGrenadeLauncher(speed)`: the projectile (`isRemote`) at the fire point, pushed by
    /// `AddForce(forward · speed · 50)` (mass 1: Δv = speed over one 0.02 s step), its collisions
    /// with the shooter's colliders ignored; then the sound.
    fn grenade(&mut self, index: usize, speed: f32, owner: usize, events: &mut Vec<SyncEvent>) {
        let sync = self.weapon_syncs[index].clone();
        let (Some(fire_point), Some(projectile)) = (sync.fire_point, sync.projectile) else {
            return;
        };
        let rotation = self.prefab.world_rotation(fire_point);
        events.push(SyncEvent::Spawn {
            prefab: projectile,
            position: self.prefab.world_of(fire_point).w_axis.truncate(),
            rotation,
            velocity: Some(rotation * Vec3::Z * speed),
            flags: SpawnFlags {
                do_damage: false,
                is_remote: true,
                ignore_owner: Some(owner),
            },
        });
        self.play_fire_audio(index, events);
    }

    fn play_fire_audio(&self, index: usize, events: &mut Vec<SyncEvent>) {
        let sync = &self.weapon_syncs[index];
        if let (Some(source), Some(clip)) = (sync.source, &sync.fire_audio) {
            events.push(SyncEvent::Sound {
                source,
                clip: clip.clone(),
            });
        }
    }

    /// The `HitBox`es a `SendMessageUpwards("ApplyDamage")` from `node` reaches: those on `node`
    /// and on each of its ancestors (active GameObjects only).
    pub fn hit_box_chain(&self, node: usize) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut current = Some(node);
        while let Some(n) = current {
            if self.prefab.active_in_hierarchy(n) {
                chain.extend(
                    self.hit_boxes
                        .iter()
                        .enumerate()
                        .filter(|(_, h)| h.node == n)
                        .map(|(i, _)| i),
                );
            }
            current = self.prefab.nodes[n].parent;
        }
        chain
    }

    /// `HitBox.ApplyDamage(damage)` → `TotalDamage(damage + maxDamage)`: unless damage is
    /// disabled, the hit marker and the `DoDamage` RPC to all (run here at once). Returns the
    /// amount sent (for the owner) and the death, if this copy died.
    pub fn apply_damage(
        &mut self,
        hit_box: usize,
        damage: f32,
        damager: ActorId,
        local: ActorId,
    ) -> (Option<f32>, Option<RemoteDeath>) {
        let total = damage + self.hit_boxes[hit_box].max_damage;
        if self.damage.disable_damage {
            return (None, None);
        }
        self.damage.fade_value = 2.0;
        (Some(total), self.do_damage(total, damager, local))
    }

    /// `DoDamage(damage, damager)` on a copy that is not ours.
    pub fn do_damage(
        &mut self,
        damage: f32,
        damager: ActorId,
        local: ActorId,
    ) -> Option<RemoteDeath> {
        if self.spawn_protected || self.damage.we_killed {
            return None;
        }
        self.damage.fade_value_b = 2.0;
        self.damage.current_hp -= damage;
        if self.damage.current_hp >= 0.0 {
            return None;
        }
        for child in self.prefab.nodes[0].children.clone() {
            self.prefab.set_active(child, false);
        }
        self.damage.current_hp = 0.0;
        self.damage.we_killed = true;
        let root = &self.prefab.nodes[0].local;
        Some(RemoteDeath {
            ragdoll: self.damage.ragdoll.clone(),
            position: root.position,
            rotation: root.rotation,
            name: self.name.clone(),
            killed_by_local: damager == local,
        })
    }

    /// The copy's hit boxes at their current transforms (`owner` = the caller's index).
    pub fn add_targets(&self, owner: usize, targets: &mut Targets) {
        for hit_box in &self.hit_boxes {
            if !self.prefab.active_in_hierarchy(hit_box.node) {
                continue;
            }
            targets.hit_boxes.push(Target {
                owner,
                node: hit_box.node,
                collider: DynamicCollider {
                    world: self.prefab.world[hit_box.node],
                    shape: hit_box.shape,
                    tag: self.prefab.nodes[hit_box.node].tag,
                },
            });
        }
        if let Some(c) = self.controller {
            let root = self.prefab.world[0];
            let half = (c.height * 0.5 - c.radius).max(0.0);
            targets.capsules.push((
                owner,
                Capsule {
                    p0: root.transform_point3(c.center - Vec3::Y * half),
                    p1: root.transform_point3(c.center + Vec3::Y * half),
                    radius: c.radius,
                },
            ));
        }
    }

    /// Moves the AudioSources with their weapons (a deactivated weapon's source stops).
    pub fn update_sources(&self, audio: &mut Audio) {
        for sync in &self.weapon_syncs {
            let Some(id) = sync.source else {
                continue;
            };
            if self.prefab.active_in_hierarchy(sync.node) {
                audio.set_position(id, self.prefab.world[sync.node].w_axis.truncate());
            } else {
                audio.stop(id);
            }
        }
    }

    /// `PlayerDamage.OnGUI` (not ours): the hit marker, faded by `fadeValue`, at
    /// `(Screen.width / 2 - 13, Screen.height / 2 - 13)` in integer arithmetic.
    pub fn hit_marker_gui(&self, gui: &mut Gui, w: f32, h: f32) {
        let Some(texture) = &self.damage.hit_mark_texture else {
            return;
        };
        gui.begin_script();
        gui.color = Vec4::new(1.0, 1.0, 1.0, self.damage.fade_value);
        let x = (w as i32 / 2 - 13) as f32;
        let y = (h as i32 / 2 - 13) as f32;
        gui.draw_texture([x, y, 26.0, 26.0], texture);
    }

    /// `DrawPlayerName.OnGUI` (enabled for TDM teammates only): the name over `positionDisplay`.
    pub fn draw_name_gui(
        &self,
        gui: &mut Gui,
        font: &mut GuiFont,
        label: &GuiStyle,
        view: &View,
        w: f32,
        h: f32,
    ) {
        if !self.draw_name_enabled || !self.prefab.active_in_hierarchy(0) {
            return;
        }
        gui.begin_script();
        gui.color = Vec4::new(0.1, 0.9, 0.5, 1.0);
        let at = self.prefab.world[self.position_display].w_axis.truncate();
        let s = world_to_screen_point(view, at, w, h);
        let offset = (s.z * 3.0).min(50.0);
        if s.z > 0.0 {
            // hpStyle: a new GUIStyle (normal weight, unlike the skin's bold label) with the
            // skin's font, the label's font size, white text, MiddleCenter.
            let style = GuiStyle {
                background: None,
                text_color: Vec4::ONE,
                border: [0.0; 4],
                margin: [0.0; 4],
                padding: [0.0; 4],
                font_style: 0,
                alignment: 4,
                word_wrap: false,
                stretch_width: false,
                ..label.clone()
            };
            gui.label(
                font,
                [s.x - 100.0, h - s.y - 5.0 - offset, 200.0, 30.0],
                &self.draw_name,
                &style,
            );
        }
    }
}

/// `Camera.WorldToScreenPoint`: pixels from the bottom left, z = depth along the view.
pub fn world_to_screen_point(view: &View, point: Vec3, w: f32, h: f32) -> Vec3 {
    let local = view.rotation.inverse() * (point - view.position);
    let tan = (view.fov_y_degrees.to_radians() * 0.5).tan();
    let aspect = w / h.max(1.0);
    if local.z == 0.0 {
        return Vec3::new(w * 0.5, h * 0.5, 0.0);
    }
    let x = local.x / (local.z * tan * aspect);
    let y = local.y / (local.z * tan);
    Vec3::new((x + 1.0) * 0.5 * w, (y + 1.0) * 0.5 * h, local.z)
}

/// A `hitBoxes` entry: its collider (`BoxCollider` or `SphereCollider` component ref).
fn hit_box(prefab: &PrefabInstance, entry: &Value) -> Option<HitBox> {
    let node = prefab.node_of_ref(&entry["box"])?;
    let (reference, _) = ref_of(&entry["box"])?;
    let shape = ["BoxCollider", "SphereCollider"]
        .into_iter()
        .find_map(|kind| {
            let data = prefab.component_by_ref(node, reference, kind)?;
            Some(match kind {
                "BoxCollider" => HitShape::Box {
                    center: vec3_of(&data["m_Center"]),
                    size: vec3_of(&data["m_Size"]),
                },
                _ => HitShape::Sphere {
                    center: vec3_of(&data["m_Center"]),
                    radius: f32_of(&data["m_Radius"]),
                },
            })
        })?;
    Some(HitBox {
        node,
        max_damage: f32_of(&entry["damage"]),
        shape,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::ClipLibrary;
    use crate::assets::find_assets_dir;

    fn german() -> PrefabInstance {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        PrefabInstance::load(&assets, "german", &mut clips).unwrap()
    }

    fn state() -> NetState {
        NetState {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            name: "Enemy".to_string(),
            look_target_position: Vec3::new(1.0, 3.5, 4.0),
            look_target_rotation: Quat::IDENTITY,
            team: "Mercenary".to_string(),
            animation: "Run".to_string(),
            hands_animation: "Null".to_string(),
            weapon: "M242".to_string(),
            animation_type: "Running".to_string(),
            spawn_protected: false,
        }
    }

    const DM: LocalView = LocalView {
        game_mode: "DM",
        team: "German-ZM",
    };

    #[test]
    fn remote_awake_hides_the_first_person_rig_and_adds_hit_boxes() {
        let pristine = german();
        let remote = RemotePlayer::new(&pristine, 0, 2, Vec3::ZERO, Quat::IDENTITY, None);
        let p = &remote.prefab;
        assert!(!p.nodes[p.find("LookObject").unwrap()].active_self);
        assert!(!p.nodes[p.find("SpawnObject").unwrap()].active_self);
        // The body mesh under mercenary_minecraft stays visible.
        assert!(p.active_in_hierarchy(430));
        assert_eq!(remote.hit_boxes.len(), 12);
        assert!(matches!(remote.hit_boxes[0].shape, HitShape::Sphere { .. }));
        assert_eq!(
            remote.weapon_syncs.len(),
            14,
            "6 on the body, 8 under Bad_legs"
        );
    }

    #[test]
    fn serialized_state_drives_position_animation_and_weapon() {
        let pristine = german();
        let mut remote = RemotePlayer::new(&pristine, 0, 2, Vec3::ZERO, Quat::IDENTITY, None);
        remote.receive(&state());
        remote.update(0.02, 0.02, DM);
        let root = remote.prefab.nodes[0].local.position;
        assert!(
            (root - Vec3::new(1.0, 2.0, 3.0) * 0.16).length() < 1e-5,
            "{root}"
        );
        assert_eq!(remote.name, "Enemy");
        let animation = remote.prefab.animation_on(429).unwrap();
        let run = animation.state_index("Run").unwrap();
        assert_eq!(animation.states[run].layer, 1);
        assert!(animation.states[run].enabled);
        // Only the M242 third-person weapon is active.
        let weapons: Vec<_> = [444, 447, 451, 464, 471, 473]
            .iter()
            .map(|&n| remote.prefab.nodes[n].active_self)
            .collect();
        assert_eq!(weapons, [false, false, true, false, false, false]);
    }

    #[test]
    fn hands_animation_plays_on_layer_four_on_the_arms() {
        let pristine = german();
        let mut remote = RemotePlayer::new(&pristine, 0, 2, Vec3::ZERO, Quat::IDENTITY, None);
        let mut s = state();
        let gun_idle = {
            let assets = find_assets_dir().unwrap();
            let mut clips = ClipLibrary::new(&assets).unwrap();
            clips
                .get("resources.assets:226")
                .unwrap()
                .unwrap()
                .name
                .clone()
        };
        s.hands_animation = gun_idle.clone();
        remote.receive(&s);
        remote.update(0.02, 0.02, DM);
        let animation = remote.prefab.animation_on(429).unwrap();
        let state = &animation.states[animation.state_index(&gun_idle).unwrap()];
        assert_eq!(state.layer, 4);
        assert!(state.enabled);
    }

    #[test]
    fn damage_runs_up_the_hit_box_chain_and_kills() {
        let pristine = german();
        let mut remote = RemotePlayer::new(&pristine, 0, 2, Vec3::ZERO, Quat::IDENTITY, None);
        remote.receive(&state());
        remote.update(0.02, 0.02, DM);
        // LowerArm_Right_jnt → UpperArm_Right, Spine, Body, Hips.
        let chain = remote.hit_box_chain(441);
        let nodes: Vec<_> = chain.iter().map(|&i| remote.hit_boxes[i].node).collect();
        assert_eq!(nodes, [441, 440, 434, 433, 432]);
        let mut death = None;
        let mut sent = Vec::new();
        for i in chain {
            let (amount, died) = remote.apply_damage(i, 40.0, 1, 1);
            sent.extend(amount);
            death = death.or(died);
        }
        // Five RPCs of 41: 200 - 205 < 0 on the fifth.
        assert_eq!(sent, [41.0; 5]);
        let death = death.unwrap();
        assert!(death.killed_by_local);
        assert_eq!(death.name, "Enemy");
        assert!(remote.damage.we_killed && remote.damage.fade_value == 2.0);
        assert!(!remote.prefab.active_in_hierarchy(430));
        // Spawn protection (synced) blocks DoDamage.
        let mut protected = RemotePlayer::new(&pristine, 0, 3, Vec3::ZERO, Quat::IDENTITY, None);
        assert_eq!(protected.do_damage(500.0, 1, 1), None);
        assert_eq!(protected.damage.current_hp, 200.0);
    }

    #[test]
    fn weapon_sync_fires_remote_bullets_and_sets_up_the_knife() {
        let pristine = german();
        let mut remote = RemotePlayer::new(&pristine, 0, 2, Vec3::ZERO, Quat::IDENTITY, None);
        remote.receive(&state());
        remote.update(0.02, 0.02, DM);
        // The M242 became active this frame: its Start runs next frame.
        assert!(
            remote
                .sync_weapons_rpc(SyncFire::MachineGun(1.0), 0.03, 0)
                .is_empty()
        );
        remote.update(0.02, 0.04, DM);
        let events = remote.sync_weapons_rpc(SyncFire::MachineGun(1.0), 0.04, 0);
        assert!(matches!(
            &events[0],
            SyncEvent::Spawn { flags, velocity: None, .. } if !flags.do_damage
        ));
        let flash = remote
            .prefab
            .renderers
            .iter()
            .find(|r| r.node == 463)
            .unwrap();
        assert!(flash.enabled, "muzzle flash on for 0.04 s");
        remote.update(0.02, 0.08, DM);
        let flash = remote
            .prefab
            .renderers
            .iter()
            .find(|r| r.node == 463)
            .unwrap();
        assert!(!flash.enabled);
        let mut s = state();
        s.weapon = "Machete".to_string();
        remote.receive(&s);
        remote.update(0.02, 0.1, DM);
        remote.update(0.02, 0.12, DM);
        // Start replaced the serialized fireAudio (SuperPunch) by the first-person knife's
        // fireSound.
        assert_eq!(
            remote.weapon_syncs[0].fire_audio.as_deref(),
            Some("audio/fire melee_fire.m4a")
        );
    }

    #[test]
    fn remote_knife_plays_its_sound() {
        let assets = find_assets_dir().unwrap();
        let mut audio = Audio::silent(&assets);
        let pristine = german();
        let mut remote = RemotePlayer::new(
            &pristine,
            0,
            2,
            Vec3::ZERO,
            Quat::IDENTITY,
            Some(&mut audio),
        );
        let mut s = state();
        s.weapon = "Machete".to_string();
        remote.receive(&s);
        remote.update(0.02, 0.02, DM);
        remote.update(0.02, 0.04, DM);
        let events = remote.sync_weapons_rpc(SyncFire::Knife, 0.04, 0);
        assert!(matches!(
            events.as_slice(),
            [SyncEvent::Sound { clip, .. }] if clip == "audio/fire melee_fire.m4a"
        ));
    }

    #[test]
    fn world_to_screen_point_centre() {
        let view = View {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            fov_y_degrees: 60.0,
            near: 0.3,
            far: 100.0,
        };
        let s = world_to_screen_point(&view, Vec3::new(0.0, 0.0, 5.0), 800.0, 600.0);
        assert_eq!(s, Vec3::new(400.0, 300.0, 5.0));
        let up = world_to_screen_point(
            &view,
            Vec3::new(0.0, 5.0 * 30f32.to_radians().tan(), 5.0),
            800.0,
            600.0,
        );
        assert!((up.y - 600.0).abs() < 1e-3);
    }
}
