//! The player's `OnGUI` HUD scripts besides the crosshair and scope (spec:
//! `docs/gameplay/hud.md`): `AmmoDisplay` (`f65490`, `f65491`), `WeaponManager.OnGUI`
//! (`f29925`) and `PlayerNetworkController.OnGUI` (`f37905`).
//!
//! `OnGUI` runs once per GUI event; each frame has at least a Layout and a Repaint event, so the
//! state changes the scripts make inside `OnGUI` (the weapon bar's fade, the message timers) are
//! applied `EVENTS_PER_FRAME` times per frame.

use glam::Vec4;

use crate::gui::{Gui, GuiFont, GuiSkin};
use crate::prefab::PrefabInstance;
use crate::weapons::{GunType, WeaponManager};

pub const EVENTS_PER_FRAME: usize = 2;

/// `AmmoDisplay` state.
pub struct AmmoDisplay {
    pub display: bool,
    /// `currentWeapon` (slot in `allWeapons`).
    current_weapon: Option<usize>,
    /// The highlighted weapon's alpha (`color`, a float field).
    color: f32,
    bullets_left: i32,
    /// `clips` is a float field (printed with `float.ToString()`).
    clips: f32,
    /// The `WeaponScript` cached from `SelectedWeapon` (slot).
    weapon_script: Option<usize>,
}

impl AmmoDisplay {
    pub fn new(display: bool) -> AmmoDisplay {
        AmmoDisplay {
            display,
            current_weapon: None,
            color: 0.0,
            bullets_left: 0,
            clips: 0.0,
            weapon_script: None,
        }
    }

    /// `Update`.
    pub fn update(&mut self, manager: &WeaponManager, dt: f32) {
        let selected = selected_slot(manager);
        if selected.is_some() {
            self.weapon_script = selected;
        }
        let Some(script) = self
            .weapon_script
            .and_then(|slot| manager.scripts.get(slot))
            .and_then(Option::as_ref)
        else {
            return;
        };
        match script.gun_type {
            GunType::MachineGun => {
                self.bullets_left = script.machine_gun.bullets_left;
                self.clips = script.machine_gun.clips as f32;
            }
            GunType::ShotGun => {
                self.bullets_left = script.shot_gun.bullets_left;
                self.clips = script.shot_gun.clips as f32;
            }
            GunType::GrenadeLauncher => self.clips = script.grenade_launcher.ammo_count as f32,
            GunType::Knife => {}
        }
        if self.current_weapon != selected {
            self.color = lerp(self.color, 0.3, dt * 20.0);
            if self.color < 0.32 {
                self.current_weapon = selected;
            }
        }
    }

    /// `OnGUI` (all of a frame's events): the ammo box and the weapon bar.
    #[allow(clippy::too_many_arguments)]
    pub fn on_gui(
        &mut self,
        gui: &mut Gui,
        font: &mut GuiFont,
        skin: &GuiSkin,
        manager: &WeaponManager,
        dt: f32,
        w: f32,
        h: f32,
    ) {
        if !self.display {
            return;
        }
        let mut items = Vec::new();
        for _ in 0..EVENTS_PER_FRAME {
            items = self.bar_items(manager, dt);
        }
        gui.begin_script();
        gui.color = Vec4::new(1.0, 1.0, 1.0, 0.9);
        let rect = [
            ((w as i32) - 110) as f32,
            ((h as i32) - 55) as f32,
            100.0,
            45.0,
        ];
        if let Some(script) = self
            .weapon_script
            .and_then(|slot| manager.scripts.get(slot))
            .and_then(Option::as_ref)
        {
            let text = match script.gun_type {
                GunType::Knife => "\u{221e}".to_string(),
                GunType::GrenadeLauncher => format_float(self.clips),
                _ => format!("{} | {}", self.bullets_left, format_float(self.clips)),
            };
            gui.box_(font, rect, &text, &skin.box_style);
        }
        // `if (weaponManager)`: always true here.
        gui.horizontal_boxes(font, [0.0, 0.0, w, 30.0], &items, &skin.box_style, 30.0);
    }

    /// One `OnGUI` event's weapon bar: (weaponName, GUI.color) per weapon.
    fn bar_items(&mut self, manager: &WeaponManager, dt: f32) -> Vec<(String, Vec4)> {
        let selected = selected_slot(manager);
        let mut items = Vec::new();
        for slot in 0..manager.all_weapons.len() {
            let alpha = if Some(slot) == self.current_weapon && self.current_weapon == selected {
                self.color = lerp(self.color, 0.9, dt * 20.0);
                self.color
            } else {
                0.3
            };
            let name = manager
                .scripts
                .get(slot)
                .and_then(Option::as_ref)
                .map_or_else(String::new, |s| s.name.clone());
            items.push((name, Vec4::new(1.0, 1.0, 1.0, alpha)));
        }
        items
    }
}

/// `float.ToString()` for the HUD's whole-number values ("3", "-1"; "0.5" otherwise).
fn format_float(value: f32) -> String {
    if value.fract() == 0.0 && value.abs() < 1e7 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// `WeaponManager.SelectedWeapon` as a slot of `allWeapons` (the port keeps its node).
fn selected_slot(manager: &WeaponManager) -> Option<usize> {
    let node = manager.selected_weapon?;
    manager.all_weapons.iter().position(|&n| n == node)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// What `WeaponManager.OnGUI` changes in the default skin persists (Unity skins are shared
/// objects): the box style's font size.
pub struct DefaultSkin {
    pub skin: GuiSkin,
}

impl DefaultSkin {
    /// `WeaponManager.OnGUI` (all of a frame's events; CrazyGames `hxg`: only the two room
    /// messages, Build-7's "Auto fire on/off" label is not in this build).
    pub fn weapon_manager_gui(
        &mut self,
        gui: &mut Gui,
        font: &mut GuiFont,
        manager: &mut WeaponManager,
        w: f32,
    ) {
        let wi = w as i32;
        let yellow = Vec4::new(1.0, 0.921_568_6, 0.015_686_275, 1.0);
        let green = Vec4::new(0.0, 1.0, 0.0, 1.0);
        let mut draw = |gui: &mut Gui, font: &mut GuiFont, skin: &mut GuiSkin, last: bool| {
            gui.begin_script();
            let message_rect = |y: f32| [(wi / 2) as f32 - w * 0.5, y, w, 50.0];
            if manager.shownowepinroom2 > 0.0 {
                manager.shownowepinroom2 -= 0.1;
                gui.color = green;
                skin.box_style.font_size = 35;
                if last {
                    let text = "This Weapon Is Available Only At FULL Game !!!";
                    gui.box_(font, message_rect(140.0), text, &skin.box_style);
                }
                skin.box_style.font_size = 12;
            }
            if manager.shownowepinroom > 0.0 {
                manager.shownowepinroom -= 0.1;
                gui.color = yellow;
                skin.box_style.font_size = 38;
                if last {
                    let text = "This Weapon Is Disabled In This Room !!!";
                    gui.box_(font, message_rect(150.0), text, &skin.box_style);
                }
                skin.box_style.font_size = 12;
            }
        };
        for event in 0..EVENTS_PER_FRAME {
            draw(gui, font, &mut self.skin, event + 1 == EVENTS_PER_FRAME);
        }
    }

    /// `PlayerNetworkController.OnGUI` of the owner: the spawn-protection label.
    pub fn spawn_protection_gui(&self, gui: &mut Gui, font: &mut GuiFont, w: f32, h: f32) {
        gui.begin_script();
        let rect = [5.0, ((h as i32) - 38) as f32, ((w as i32) - 5) as f32, 50.0];
        gui.label(
            font,
            rect,
            crate::damage::SPAWN_PROTECTION_TEXT,
            &self.skin.label,
        );
    }
}

/// Whether the local player's GameObject (and so its `OnGUI` scripts) is active.
pub fn player_gui_active(prefab: &PrefabInstance, node: Option<usize>) -> bool {
    node.is_some_and(|n| prefab.active_in_hierarchy(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_text() {
        assert_eq!(format_float(3.0), "3");
        assert_eq!(format_float(0.0), "0");
        assert_eq!(format_float(-1.0), "-1");
        assert_eq!(format_float(0.5), "0.5");
    }
}
