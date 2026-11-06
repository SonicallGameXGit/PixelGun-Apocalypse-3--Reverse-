//! Ports of the game scripts that drive animations (spec: `docs/gameplay/animation.md`):
//! `WeaponAnimation` (first-person hands, driven by `weapons.rs`) and `CharacterAnimation`
//! (third-person body).

use serde_json::Value;

use crate::anim::{ClipLibrary, WrapMode};
use crate::assets::f32_of;
use crate::prefab::PrefabInstance;

/// `WeaponAnimation`: clip names and speeds of one first-person weapon.
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
    pub fn from_fields(node: usize, f: &Value) -> WeaponAnimation {
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
    pub fn fire(&self, prefab: &mut PrefabInstance) {
        if let Some(animation) = prefab.animation_on(self.node) {
            animation.rewind(&self.shoot);
            animation.set_speed(&self.shoot, self.fire_animation_speed);
            animation.play(&self.shoot);
        }
    }

    /// `Reloading(reloadTime)`: Stop(Reload), speed = clip length / reloadTime, Rewind, Play.
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
    pub fn take_out(&self, prefab: &mut PrefabInstance) {
        let clip = self.take_out.clone();
        self.take(prefab, &clip);
    }
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
