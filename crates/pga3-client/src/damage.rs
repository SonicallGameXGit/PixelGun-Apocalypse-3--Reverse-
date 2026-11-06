//! The local player's damage receiver and what follows a death (spec: `docs/gameplay/damage.md`):
//! `PlayerDamage` (`f37876`..`f37890`), `PlayerNetworkController`'s spawn protection
//! (`f37896`, `f37905`) and the death prefab's `RagdollController` (`f29667`..`f29672`).
//!
//! Damage arrives as the `DoDamage` RPC, which `PhotonTargets.All` also runs locally at once. The
//! only local source is fall damage (`ApplyFallDamage`): bullets and explosions send
//! `ApplyDamage`, which only `HitBox`es receive, and the owner's hit boxes are destroyed in
//! `Awake`.

use glam::{Quat, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{f32_of, ref_of};
use crate::gui::{Gui, GuiFont, GuiSkin};
use crate::unity::{Random, euler, slerp_rotation};

/// What `DoDamage` / its coroutines did.
#[derive(Clone, Debug, PartialEq)]
pub enum DamageEvent {
    /// `currentHp < 0`: every child of the player deactivated, the ragdoll prefab instantiated
    /// at the player's transform and sent `RespawnAfter`; the deaths property incremented.
    Died { ragdoll: Option<String> },
    /// `DestroyPlayer(0.2)` finished: `PhotonNetwork.Destroy(gameObject)`.
    Destroyed,
}

pub struct PlayerDamage {
    pub current_hp: f32,
    pub disable_damage: bool,
    pub we_killed: bool,
    /// Hit marker alpha (drawn for remote players only).
    pub fade_value: f32,
    /// Bloody screen alpha.
    pub fade_value_b: f32,
    /// `camRot`; `None` while it still holds its default (zero) quaternion, which the
    /// normalising Slerp leaves without effect.
    cam_rot: Option<Quat>,
    cam_default_rotation: Quat,
    /// `doCameraShake` waiting in `WaitForSeconds(0.1)` until this time.
    shake_until: Option<f32>,
    /// `DestroyPlayer` waiting until this time.
    destroy_at: Option<f32>,
    pub skin: Option<String>,
    pub bloody_screen: Option<String>,
    pub ragdoll: Option<String>,
    random: Random,
}

impl PlayerDamage {
    /// `Awake` of the owner's instance: `currentHp = hp`, `camDefaultRotation` = the Main
    /// Camera's local rotation, hit boxes destroyed.
    pub fn awake(fields: &Value, camera_local_rotation: Quat) -> PlayerDamage {
        let hp = f32_of(&fields["hp"]);
        let reference = |name: &str| ref_of(&fields[name]).map(|(r, _)| r.to_string());
        PlayerDamage {
            current_hp: hp,
            disable_damage: fields["disableDamage"].as_bool().unwrap_or(false),
            we_killed: false,
            fade_value: 0.0,
            fade_value_b: 0.0,
            cam_rot: None,
            cam_default_rotation: camera_local_rotation,
            shake_until: None,
            destroy_at: None,
            skin: reference("guiSKin"),
            bloody_screen: fields["bloodyScreen"]["asset"].as_str().map(str::to_string),
            ragdoll: reference("ragdoll"),
            random: Random::new(2),
        }
    }

    /// `Update`: the fades decay and the Main Camera's local rotation eases to `camRot`.
    pub fn update(&mut self, dt: f32, camera_local_rotation: &mut Quat) {
        self.fade_value = lerp(self.fade_value, 0.0, dt * 2.0);
        self.fade_value_b = lerp(self.fade_value_b, 0.0, dt * 2.0);
        if let Some(target) = self.cam_rot {
            *camera_local_rotation = slerp_rotation(*camera_local_rotation, target, dt * 15.0);
        }
    }

    /// `WaitForSeconds` resumes (after every `Update`).
    pub fn run_coroutines(&mut self, time: f32) -> Option<DamageEvent> {
        if let Some(until) = self.shake_until
            && time >= until
        {
            self.shake_until = None;
            self.cam_rot = Some(self.cam_default_rotation);
        }
        if let Some(at) = self.destroy_at
            && time >= at
        {
            self.destroy_at = None;
            return Some(DamageEvent::Destroyed);
        }
        None
    }

    /// `ApplyFallDamage` (owner only): the `DoDamage` RPC.
    pub fn apply_fall_damage(
        &mut self,
        damage: f32,
        spawn_protected: bool,
        time: f32,
    ) -> Option<DamageEvent> {
        self.do_damage(damage, spawn_protected, time)
    }

    /// `TotalDamage(damage)`: the bloody-screen flash and the `DoDamage` RPC unless damage is
    /// disabled. Called by `HitBox.ApplyDamage` on remote players' hit boxes (network milestone).
    #[allow(dead_code)]
    pub fn total_damage(
        &mut self,
        damage: f32,
        spawn_protected: bool,
        time: f32,
    ) -> Option<DamageEvent> {
        if self.disable_damage {
            return None;
        }
        self.fade_value = 2.0;
        self.do_damage(damage, spawn_protected, time)
    }

    /// `DoDamage(damage, damager)` on the owner's instance.
    fn do_damage(&mut self, damage: f32, spawn_protected: bool, time: f32) -> Option<DamageEvent> {
        if spawn_protected || self.we_killed {
            return None;
        }
        if self.current_hp > 0.0 {
            // StopAllCoroutines; StartCoroutine(doCameraShake()) runs to its first yield.
            self.destroy_at = None;
            let x = self.random.range_int(-10, 10) as f32;
            let y = self.random.range_int(-10, 10) as f32;
            self.cam_rot = Some(euler(Vec3::new(x, y, 0.0)));
            self.shake_until = Some(time + 0.1);
        }
        self.fade_value_b = 2.0;
        self.current_hp -= damage;
        if self.current_hp < 0.0 {
            self.destroy_at = Some(time + 0.2);
            self.current_hp = 0.0;
            self.we_killed = true;
            return Some(DamageEvent::Died {
                ragdoll: self.ragdoll.clone(),
            });
        }
        None
    }

    /// `OnGUI` of the owner: the bloody screen and the HP box (`GUI.depth` 10: under the other
    /// scripts' GUI).
    pub fn on_gui(&self, gui: &mut Gui, font: &mut GuiFont, skin: &GuiSkin, w: f32, h: f32) {
        gui.begin_script();
        gui.color = Vec4::new(1.0, 1.0, 1.0, self.fade_value_b);
        if let Some(texture) = &self.bloody_screen {
            gui.draw_texture([0.0, 0.0, w, h], texture);
        }
        gui.color = Vec4::new(1.0, 1.0, 1.0, 0.9);
        let text = format!("HP | {}", self.current_hp as i32);
        gui.box_(
            font,
            [w - 220.0, h - 55.0, 100.0, 45.0],
            &text,
            &skin.box_style,
        );
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// `PlayerNetworkController.IsSpawnProtected` / `SpawnProtectionTimer` (5 s from the
/// constructor) on the owner's instance.
pub struct SpawnProtection {
    pub protected: bool,
    timer: f32,
}

impl Default for SpawnProtection {
    fn default() -> SpawnProtection {
        SpawnProtection {
            protected: true,
            timer: 5.0,
        }
    }
}

/// The label `PlayerNetworkController.OnGUI` shows while protected. `yelow` is not a colour
/// name, so Unity keeps that tag as text.
pub const SPAWN_PROTECTION_TEXT: &str =
    "<size=32><color=yelow>>SPAWN PROTECTION ACTIVE</color></size>";

impl SpawnProtection {
    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.protected = false;
            self.timer = 0.0;
        } else {
            self.protected = true;
        }
    }
}

/// `RagdollController` of the death prefab.
pub struct RagdollController {
    pub timer: i32,
    pub gui: bool,
    /// `cam` (node), destroyed by `clearCamera`.
    pub cam: Option<usize>,
    /// `InvokeRepeating("_respawnAfter", 1, 1)` next call.
    next_call: Option<f32>,
    /// The component destroyed itself.
    destroyed: bool,
}

impl RagdollController {
    pub fn new(fields: &Value, cam: Option<usize>) -> RagdollController {
        RagdollController {
            timer: fields["timer"].as_i64().unwrap_or(0) as i32,
            gui: false,
            cam,
            next_call: None,
            destroyed: false,
        }
    }

    /// `RespawnAfter` (sent by `PlayerDamage` right after instantiating the prefab).
    pub fn respawn_after(&mut self, time: f32) {
        self.gui = true;
        self.next_call = Some(time + 1.0);
    }

    /// The `InvokeRepeating` calls due by `time`; true when `_respawnAfter` reached 0 and sent
    /// `SpawnPlayer` (after `clearCamera` and destroying itself).
    pub fn update(&mut self, time: f32) -> bool {
        let mut spawn = false;
        while let Some(at) = self.next_call
            && time >= at
            && !self.destroyed
        {
            self.next_call = Some(at + 1.0);
            self.timer -= 1;
            if self.timer == 0 {
                self.cam = None;
                self.destroyed = true;
                self.next_call = None;
                spawn = true;
            }
        }
        spawn
    }

    /// `OnGUI`: the countdown under the ragdoll skin.
    pub fn on_gui(&self, gui: &mut Gui, font: &mut GuiFont, skin: &GuiSkin, w: f32, h: f32) {
        if self.destroyed || !self.gui || self.cam.is_none() {
            return;
        }
        gui.begin_script();
        let rect = [
            ((w as i32) / 2 - 75) as f32,
            ((h as i32) / 2 - 15) as f32,
            150.0,
            30.0,
        ];
        let text = format!("Respawn in: {}", self.timer);
        gui.label(font, rect, &text, &skin.label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn damage() -> PlayerDamage {
        PlayerDamage::awake(
            &json!({ "hp": 200.0, "disableDamage": false, "ragdoll": { "ref": "resources.assets:14520" } }),
            Quat::IDENTITY,
        )
    }

    #[test]
    fn fall_damage_shakes_then_kills_below_zero() {
        let mut d = damage();
        assert_eq!(d.apply_fall_damage(50.0, false, 0.0), None);
        assert_eq!(d.current_hp, 150.0);
        assert_eq!(d.fade_value_b, 2.0);
        // The shake target is set, then reset to the default rotation after 0.1 s.
        assert!(d.cam_rot.is_some() && d.shake_until == Some(0.1));
        assert_eq!(d.run_coroutines(0.1), None);
        assert_eq!(d.cam_rot, Some(Quat::IDENTITY));
        // Exactly 0 HP is still alive (`currentHp < 0` kills).
        assert_eq!(d.apply_fall_damage(150.0, false, 1.0), None);
        assert_eq!(d.current_hp, 0.0);
        let died = d.apply_fall_damage(1.0, false, 2.0);
        assert_eq!(
            died,
            Some(DamageEvent::Died {
                ragdoll: Some("resources.assets:14520".to_string())
            })
        );
        assert!(d.we_killed && d.current_hp == 0.0);
        assert_eq!(d.apply_fall_damage(10.0, false, 2.1), None);
        assert_eq!(d.run_coroutines(2.1), None);
        assert_eq!(d.run_coroutines(2.2), Some(DamageEvent::Destroyed));
    }

    #[test]
    fn spawn_protection_blocks_damage_for_five_seconds() {
        let mut d = damage();
        let mut p = SpawnProtection::default();
        for _ in 0..249 {
            p.update(0.02);
        }
        assert!(p.protected);
        assert_eq!(d.apply_fall_damage(50.0, p.protected, 0.0), None);
        assert_eq!(d.current_hp, 200.0);
        p.update(0.03);
        assert!(!p.protected);
        d.apply_fall_damage(50.0, p.protected, 0.0);
        assert_eq!(d.current_hp, 150.0);
    }

    #[test]
    fn ragdoll_counts_down_and_spawns() {
        let mut r = RagdollController::new(&json!({ "timer": 2 }), Some(69));
        r.respawn_after(10.0);
        assert!(!r.update(10.9));
        assert_eq!(r.timer, 2);
        assert!(!r.update(11.0));
        assert_eq!(r.timer, 1);
        assert!(r.update(12.0));
        assert_eq!((r.timer, r.cam), (0, None));
        assert!(!r.update(20.0));
    }
}
