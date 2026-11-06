//! Ports of the game scripts that drive animations (spec: `docs/gameplay/animation.md`):
//! `WeaponManager.Awake` / `TakeFirstWeapon`, `WeaponScript.selectWeapon`, `WeaponAnimation`
//! (first-person hands), the motion part of `WeaponScript.Update` (`Aiming`, `RotationRealism`,
//! `SmoothMove`) and `CharacterAnimation` (third-person body).

use glam::{EulerRot, Quat, Vec3};
use serde_json::Value;

use crate::anim::{ClipLibrary, WrapMode};
use crate::assets::f32_of;
use crate::prefab::PrefabInstance;

/// `WeaponAnimation`: clip names and speeds of one first-person weapon.
#[allow(
    dead_code,
    reason = "Shoot/Reload/TakeOut are used by WeaponScript firing and weapon switching (not ported yet)"
)]
#[derive(Clone, Debug)]
pub struct WeaponAnimation {
    /// Node carrying the component and its `Animation`.
    pub node: usize,
    pub idle: String,
    pub reload: String,
    pub shoot: String,
    pub take_in: String,
    pub take_out: String,
    pub fire_animation_speed: f32,
    pub take_in_out_speed: f32,
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

impl WeaponAnimation {
    fn from_fields(node: usize, f: &Value) -> WeaponAnimation {
        WeaponAnimation {
            node,
            idle: text(&f["Idle"]),
            reload: text(&f["Reload"]),
            shoot: text(&f["Shoot"]),
            take_in: text(&f["TakeIn"]),
            take_out: text(&f["TakeOut"]),
            fire_animation_speed: f32_of(&f["FireAnimationSpeed"]),
            take_in_out_speed: f32_of(&f["TakeInOutSpeed"]),
        }
    }

    /// `Awake`: `animation.Play(Idle); animation.wrapMode = WrapMode.Once`.
    pub fn awake(&self, prefab: &mut PrefabInstance) {
        if let Some(animation) = prefab.animation_on(self.node) {
            animation.play(&self.idle);
            animation.set_wrap_mode(WrapMode::Once);
        }
    }

    /// `Fire`: Rewind(Shoot), speed = FireAnimationSpeed, Play(Shoot).
    #[allow(dead_code, reason = "called by WeaponScript (not ported yet)")]
    pub fn fire(&self, prefab: &mut PrefabInstance) {
        if let Some(animation) = prefab.animation_on(self.node) {
            animation.rewind(&self.shoot);
            animation.set_speed(&self.shoot, self.fire_animation_speed);
            animation.play(&self.shoot);
        }
    }

    /// `Reloading(reloadTime)`: Stop(Reload), speed = clip length / reloadTime, Rewind, Play.
    #[allow(dead_code, reason = "called by WeaponScript (not ported yet)")]
    pub fn reloading(&self, prefab: &mut PrefabInstance, reload_time: f32) {
        if let Some(animation) = prefab.animation_on(self.node) {
            animation.stop(&self.reload);
            if let Some(index) = animation.state_index(&self.reload) {
                let length = animation.states[index].clip.length();
                animation.set_speed(&self.reload, length / reload_time);
            }
            animation.rewind(&self.reload);
            animation.play(&self.reload);
        }
    }

    fn take(&self, prefab: &mut PrefabInstance, clip: &str) {
        if let Some(animation) = prefab.animation_on(self.node) {
            animation.rewind(clip);
            animation.set_speed(clip, self.take_in_out_speed);
            animation.set_time(clip, 0.0);
            animation.play(clip);
        }
    }

    /// `takeIn`: Rewind, speed = TakeInOutSpeed, time = 0, Play(TakeIn).
    pub fn take_in(&self, prefab: &mut PrefabInstance) {
        let clip = self.take_in.clone();
        self.take(prefab, &clip);
    }

    /// `takeOut`: as `takeIn` with TakeOut.
    #[allow(dead_code, reason = "called by WeaponScript (not ported yet)")]
    pub fn take_out(&self, prefab: &mut PrefabInstance) {
        let clip = self.take_out.clone();
        self.take(prefab, &clip);
    }
}

/// The motion settings and state of one `WeaponScript` (the weapon's GameObject, child of
/// `WeaponManager`). Firing, reloading and aiming input are not ported yet.
#[derive(Clone, Debug)]
pub struct WeaponScript {
    pub node: usize,
    /// `Aim.smoothTime` (+0x40 +20): `Aiming` lerps the position at `deltaTime / smoothTime`.
    aim_smooth_time: f32,
    /// `defaultPosition` (`Start`: the GameObject's local position).
    default_position: Vec3,
    /// `RotRealism.RotationAmplitude` / `.smooth` (ints).
    rotation_amplitude: i32,
    rotation_smooth: i32,
    /// `SmoothMovement.maxAmount` / `.Smooth` (int).
    smooth_max_amount: f32,
    smooth_speed: i32,
}

/// What `WeaponScript.Update` reads each frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct WeaponInput {
    /// `CF2Input.GetAxis("Mouse X")`, `("Mouse Y")`, `("Vertical")`.
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub vertical: f32,
    /// `CharacterController.velocity.y`.
    pub velocity_y: f32,
}

fn int_of(value: &Value) -> i32 {
    value.as_i64().unwrap_or(0) as i32
}

impl WeaponScript {
    fn from_fields(prefab: &PrefabInstance, node: usize, f: &Value) -> WeaponScript {
        WeaponScript {
            node,
            aim_smooth_time: f32_of(&f["Aim"]["smoothTime"]),
            default_position: prefab.nodes[node].local.position,
            rotation_amplitude: int_of(&f["RotRealism"]["RotationAmplitude"]),
            rotation_smooth: int_of(&f["RotRealism"]["smooth"]),
            smooth_max_amount: f32_of(&f["SmoothMovement"]["maxAmount"]),
            smooth_speed: int_of(&f["SmoothMovement"]["Smooth"]),
        }
    }

    /// The motion part of `WeaponScript.Update` (`f29942`): `Aiming`, `RotationRealism`,
    /// `SmoothMove`, in that order, on the weapon GameObject's local transform.
    pub fn update(&self, prefab: &mut PrefabInstance, input: &WeaponInput, dt: f32) {
        let local = &mut prefab.nodes[self.node].local;

        // Aiming (`f29943`), not-aimed branch: currentPosition = defaultPosition (the camera FOV
        // lerps to defaultFov, which nothing changes while not aiming).
        let t = (dt / self.aim_smooth_time).clamp(0.0, 1.0);
        local.position = local.position.lerp(self.default_position, t);

        // RotationRealism (`f29944`): tilt against the mouse beyond a 0.1 dead zone.
        let amplitude = self.rotation_amplitude as f32;
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
        // Quaternion.Euler(x, y, 0).
        let target = Quat::from_euler(
            EulerRot::YXZ,
            angle_y.to_radians(),
            angle_x.to_radians(),
            0.0,
        );
        let t = (dt * self.rotation_smooth as f32).clamp(0.0, 1.0);
        // Transform.localRotation stores a normalised quaternion.
        local.rotation = local.rotation.slerp(target, t).normalize();

        // SmoothMove (`f29945`): offsets from the vertical speed and the forward axis.
        let max = self.smooth_max_amount;
        let mut offset_y = 0.0;
        if input.velocity_y > max + 1.0 {
            offset_y = -max;
        }
        if input.velocity_y < -max - 1.0 {
            offset_y = max;
        }
        let offset_z = (-input.vertical).clamp(-max, max);
        let target = local.position + Vec3::new(0.0, offset_y, offset_z);
        let t = (dt * self.smooth_speed as f32).clamp(0.0, 1.0);
        local.position = local.position.lerp(target, t);
    }
}

/// First-person weapons of a player prefab (`WeaponManager` and its weapons).
pub struct WeaponManager {
    /// `allWeapons`: the WeaponScript GameObjects.
    pub all_weapons: Vec<usize>,
    pub index: usize,
    /// The `WeaponAnimation` of each weapon (`GetComponentInChildren`, may be missing).
    pub weapon_animations: Vec<Option<WeaponAnimation>>,
    /// The `WeaponScript` of each weapon.
    pub weapon_scripts: Vec<Option<WeaponScript>>,
}

impl WeaponManager {
    pub fn selected_animation(&self) -> Option<&WeaponAnimation> {
        self.weapon_animations.get(self.index)?.as_ref()
    }

    /// `WeaponScript.Update` of every active weapon.
    pub fn update(&self, prefab: &mut PrefabInstance, input: &WeaponInput, dt: f32) {
        for script in self.weapon_scripts.iter().flatten() {
            if prefab.active_in_hierarchy(script.node) {
                script.update(prefab, input, dt);
            }
        }
    }
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

/// `WeaponManager.Awake`: activates every child (their `Awake` runs: `WeaponAnimation.Awake`
/// plays Idle), deactivates them all again, then `TakeFirstWeapon(allWeapons[index])` activates
/// that weapon and sends it `selectWeapon` (→ `WeaponAnimation.takeIn`).
pub fn weapon_manager_awake(prefab: &mut PrefabInstance) -> Option<WeaponManager> {
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
    // WeaponScript.Awake (while active) caches GetComponentInChildren<WeaponAnimation>.
    let weapon_animations: Vec<Option<WeaponAnimation>> = all_weapons
        .iter()
        .map(|&w| weapon_animation_in_children(prefab, w))
        .collect();
    for &child in &children {
        prefab.nodes[child].active_self = false;
    }
    for &weapon in &all_weapons {
        prefab.nodes[weapon].active_self = false;
    }

    // `Start` runs on first activation, before anything moved the weapon: defaultPosition is
    // the prefab's local position.
    let weapon_scripts = all_weapons
        .iter()
        .map(|&w| {
            let fields = prefab.scripts(w, "WeaponScript").next()?["fields"].clone();
            Some(WeaponScript::from_fields(prefab, w, &fields))
        })
        .collect();
    let index = fields["index"].as_u64().unwrap_or(0) as usize;
    let manager = WeaponManager {
        all_weapons,
        index,
        weapon_animations,
        weapon_scripts,
    };
    // TakeFirstWeapon: SetActive(true), SendMessage("selectWeapon").
    if let Some(&weapon) = manager.all_weapons.get(index) {
        prefab.nodes[weapon].active_self = true;
        if let Some(weapon_animation) = manager.selected_animation() {
            weapon_animation.take_in(prefab);
        }
    }
    Some(manager)
}

/// `CharacterAnimation.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Stand,
    Crouch,
    Prone,
}

/// Inputs `CharacterAnimation.LateUpdate` reads: `FPScontroller` flags and
/// `CF2Input.GetKey(KeyCode.W/A/S/D/R)` (the letter keys only, not the arrows).
#[derive(Clone, Copy, Debug, Default)]
pub struct CharacterInput {
    pub crouch: bool,
    pub prone: bool,
    pub grounded: bool,
    pub walking: bool,
    pub running: bool,
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_r: bool,
}

/// `CharacterAnimation` (on the third-person body, e.g. `mercenary_minecraft`).
pub struct CharacterAnimation {
    /// Node of the body's `Animation`.
    pub node: usize,
    /// `Animations` slots, as clip names.
    pub clips: CharacterClips,
    pub action: Action,
    /// `animationSyncHelper`: the last cross-faded clip (synced to other players).
    pub animation_sync_helper: String,
    /// `animationType`: "Walking", "Running", "Crouch" or "Prone".
    pub animation_type: String,
}

/// `CharacterAnimation.animations` fields as clip names. The remaining slots (`pistolIdle`,
/// `knifeIdle`, `gunIdle`) are not referenced by any ported method; `configureAnimations` leaves
/// them (and `jumpPose`) at their default wrap mode.
#[derive(Clone, Debug, Default)]
pub struct CharacterClips {
    pub reload: Option<String>,
    pub fire_still: Option<String>,
    pub knife_hit: Option<String>,
    pub throw_nade: Option<String>,
    pub jump_pose: Option<String>,
    pub stay_idle: Option<String>,
    pub crouch_idle: Option<String>,
    pub prone_idle: Option<String>,
    pub walk_front: Option<String>,
    pub walk_back: Option<String>,
    pub walk_left: Option<String>,
    pub walk_right: Option<String>,
    pub run_front: Option<String>,
    pub crouch_front: Option<String>,
    pub crouch_left: Option<String>,
    pub crouch_right: Option<String>,
    pub crouch_back: Option<String>,
    pub prone_front: Option<String>,
    pub prone_left: Option<String>,
    pub prone_right: Option<String>,
    pub prone_back: Option<String>,
}

impl CharacterAnimation {
    /// Finds the component, resolves its clip slots and runs `Start` → `configureAnimations`.
    pub fn start(
        prefab: &mut PrefabInstance,
        library: &mut ClipLibrary,
    ) -> Option<CharacterAnimation> {
        let (node, component) = prefab
            .find_scripts("CharacterAnimation")
            .into_iter()
            .next()?;
        let slots = component["fields"]["Animations"].clone();
        // Each slot holds a clip ref; `CrossFade(clip.name)` addresses the state by clip name.
        let mut clip_name = |field: &str| -> Option<String> {
            let reference = slots[field]["ref"].as_str()?;
            library
                .get(reference)
                .ok()
                .flatten()
                .map(|c| c.name.clone())
        };
        let clips = CharacterClips {
            reload: clip_name("reload"),
            fire_still: clip_name("fireStill"),
            knife_hit: clip_name("knifeHit"),
            throw_nade: clip_name("throwNade"),
            jump_pose: clip_name("jumpPose"),
            stay_idle: clip_name("stayIdle"),
            crouch_idle: clip_name("crouchIdle"),
            prone_idle: clip_name("proneIdle"),
            walk_front: clip_name("walkFront"),
            walk_back: clip_name("walkBack"),
            walk_left: clip_name("walkLeft"),
            walk_right: clip_name("walkRight"),
            run_front: clip_name("runFront"),
            crouch_front: clip_name("crouchFront"),
            crouch_left: clip_name("crouchLeft"),
            crouch_right: clip_name("crouchRight"),
            crouch_back: clip_name("crouchBack"),
            prone_front: clip_name("proneFront"),
            prone_left: clip_name("proneLeft"),
            prone_right: clip_name("proneRight"),
            prone_back: clip_name("proneBack"),
        };
        // configureAnimations: every slot except jumpPose and the three hand idles loops.
        let looping = [
            &clips.reload,
            &clips.fire_still,
            &clips.knife_hit,
            &clips.throw_nade,
            &clips.stay_idle,
            &clips.crouch_idle,
            &clips.prone_idle,
            &clips.walk_front,
            &clips.walk_back,
            &clips.walk_left,
            &clips.walk_right,
            &clips.run_front,
            &clips.crouch_front,
            &clips.crouch_left,
            &clips.crouch_right,
            &clips.crouch_back,
            &clips.prone_front,
            &clips.prone_left,
            &clips.prone_right,
            &clips.prone_back,
        ];
        let names: Vec<String> = looping.into_iter().flatten().cloned().collect();
        if let Some(animation) = prefab.animation_on(node) {
            for name in names {
                animation.set_state_wrap_mode(&name, WrapMode::Loop);
            }
        }
        Some(CharacterAnimation {
            node,
            clips,
            action: Action::Stand,
            animation_sync_helper: String::new(),
            animation_type: String::new(),
        })
    }

    fn cross_fade(&mut self, prefab: &mut PrefabInstance, clip: &Option<String>) {
        let Some(name) = clip.clone() else {
            return;
        };
        if let Some(animation) = prefab.animation_on(self.node) {
            animation.cross_fade(&name, 0.2);
        }
        self.animation_sync_helper = name;
    }

    /// `LateUpdate` (decompiled `CharacterAnimation$$LateUpdate`).
    pub fn late_update(&mut self, prefab: &mut PrefabInstance, i: &CharacterInput) {
        self.action = match (i.crouch, i.prone) {
            (false, false) => Action::Stand,
            (false, true) => Action::Prone,
            (true, _) => Action::Crouch,
        };
        let clips = self.clips.clone();
        match self.action {
            Action::Stand => {
                if !i.grounded {
                    self.cross_fade(prefab, &clips.jump_pose);
                } else if !i.walking {
                    self.cross_fade(prefab, &clips.stay_idle);
                } else if i.running {
                    self.animation_type = "Running".to_string();
                    if i.key_w {
                        self.cross_fade(prefab, &clips.run_front);
                    }
                } else {
                    self.animation_type = "Walking".to_string();
                    if i.key_w {
                        self.cross_fade(prefab, &clips.walk_front);
                    } else if i.key_a && !i.key_s {
                        self.cross_fade(prefab, &clips.walk_left);
                    } else if i.key_d && !i.key_s {
                        self.cross_fade(prefab, &clips.walk_right);
                    } else if i.key_s {
                        // (The original also tests `GetKey(R) && !GetKey(R)`, which is never
                        // true, before this branch.)
                        self.cross_fade(prefab, &clips.walk_back);
                    }
                }
            }
            Action::Crouch => {
                self.animation_type = "Crouch".to_string();
                if !i.walking {
                    self.cross_fade(prefab, &clips.crouch_idle);
                } else if i.key_w {
                    self.cross_fade(prefab, &clips.crouch_front);
                } else if i.key_a && !i.key_s {
                    self.cross_fade(prefab, &clips.crouch_left);
                } else if i.key_d && !i.key_s {
                    self.cross_fade(prefab, &clips.crouch_right);
                } else if i.key_s {
                    self.cross_fade(prefab, &clips.crouch_back);
                }
            }
            Action::Prone => {
                self.animation_type = "Prone".to_string();
                // Prone idle cross-fades the `reload` slot (as in the original).
                if !i.walking {
                    self.cross_fade(prefab, &clips.reload);
                } else if i.key_w {
                    self.cross_fade(prefab, &clips.prone_front);
                } else if i.key_a && !i.key_s {
                    self.cross_fade(prefab, &clips.prone_left);
                } else if i.key_d && !i.key_s {
                    self.cross_fade(prefab, &clips.prone_right);
                } else if i.key_s {
                    self.cross_fade(prefab, &clips.prone_back);
                } else if i.key_r {
                    // Cross-fades `reload` but records `proneBack` as the sync name.
                    let reload = clips.reload.clone();
                    self.cross_fade(prefab, &reload);
                    if let Some(name) = clips.prone_back {
                        self.animation_sync_helper = name;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    #[test]
    fn weapon_script_sways_with_mouse_and_movement() {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        let mut prefab = PrefabInstance::load(&assets, "german", &mut clips).unwrap();
        let manager = weapon_manager_awake(&mut prefab).unwrap();
        let script = manager.weapon_scripts[manager.index].clone().unwrap();
        let default = prefab.nodes[script.node].local.position;
        let dt = 1.0 / 60.0;
        // Mouse right: the weapon turns towards Euler(0, amplitude · x, 0) = 2° · 1.
        let turning = WeaponInput {
            mouse_x: 1.0,
            ..Default::default()
        };
        for _ in 0..120 {
            manager.update(&mut prefab, &turning, dt);
        }
        let (y, x, _) = prefab.nodes[script.node]
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
        for _ in 0..120 {
            manager.update(&mut prefab, &walking, dt);
        }
        let z = prefab.nodes[script.node].local.position.z - default.z;
        assert!(z < -0.005 && z > -0.1, "{z}");
        // Idle again: back to the default position and rotation.
        for _ in 0..300 {
            manager.update(&mut prefab, &WeaponInput::default(), dt);
        }
        let local = prefab.nodes[script.node].local;
        assert!((local.position - default).length() < 1e-4);
        assert!(
            local.rotation.angle_between(Quat::IDENTITY) < 1e-3,
            "{:?}",
            local.rotation
        );
    }

    #[test]
    fn first_weapon_takes_in_then_stops() {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        let mut prefab = PrefabInstance::load(&assets, "german", &mut clips).unwrap();
        let manager = weapon_manager_awake(&mut prefab).unwrap();
        let weapon = manager.selected_animation().unwrap().clone();
        let take_in = weapon.take_in.clone();
        let length = {
            let animation = prefab.animation_on(weapon.node).unwrap();
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
        prefab.update_animations();
        let start = poses(&prefab);
        prefab.begin_frame(0.1);
        prefab.update_animations();
        assert_ne!(start, poses(&prefab), "TakeIn moves the hands");
        // Past the end (length / speed), Once stops the state.
        prefab.begin_frame(length / weapon.take_in_out_speed);
        prefab.update_animations();
        assert!(
            !prefab
                .animation_on(weapon.node)
                .unwrap()
                .is_playing(&take_in)
        );
        eprintln!("TakeIn length {length}");
    }
}
