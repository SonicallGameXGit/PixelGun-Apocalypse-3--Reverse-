# HUD (OnGUI)

Client: `gui.rs` (IMGUI subset, font, skins), `hud.rs` (`AmmoDisplay`, `WeaponManager.OnGUI`, spawn protection), `damage.rs` (`PlayerDamage.OnGUI`, `RagdollController.OnGUI`). The crosshair and the sniper scope are covered in `docs/gameplay/weapons.md`.

## IMGUI model

- `OnGUI` runs once per GUI event, and every frame has at least a Layout and a Repaint event. State that a script changes inside `OnGUI` is therefore changed twice per frame (`hud::EVENTS_PER_FRAME = 2`). This affects the weapon bar's fade and the `shownowepinroom` timers. It is **inferred** from Unity's documented behaviour, assuming no input events in the frame.
- `GUI.skin` and `GUI.color` reset to the default skin and white at the start of each script's `OnGUI`.
- `GUISkin` styles are shared objects, so `fontSize` and `alignment` changes made by `WeaponManager.OnGUI` to the default skin persist.
- **Skins:**
  - `resources.assets:40959` is the `AmmoDisplay` / `PlayerDamage` skin;
  - `resources.assets:40964` is the `RagdollController` skin;
  - the default is `unity_default_resources:11000`.

  Skins are published as JSON in `assets/data/scriptable/UnityEngine.GUISkin/`.
- Box and label backgrounds are drawn 9-slice with the style's `border`. Text is placed with the style's padding and alignment.
- **Font:**
  - Unity's built-in "Arial" is Liberation Sans 2.00.1 (OFL). It is published as `assets/fonts/Arial.ttf`, with the licence in `OFL-LiberationSans.txt`.
  - Built-in metrics: `m_FontSize` 13, line spacing 14.94873, ascent 11.768555.
  - Bold is synthetic.
  - Glyphs are rasterised with `fontdue` into a 1024² atlas.
- **Rich text:** `<b>`, `<i>`, `<size=N>` and `<color=...>` are supported. A tag with an invalid value is shown as literal text (**inferred**).

## Draw order (client)

1. `PlayerDamage` (depth 10): the bloody screen and HP.
2. `RagdollController` label, while dead.
3. Crosshair or scope.
4. `AmmoDisplay`.
5. `WeaponManager` texts.
6. Spawn-protection label.

Unity's order between scripts with the same depth is not established (`docs/unknowns.md`).

## AmmoDisplay (`f65489`..`f65491`)

**`Update`:**
- Caches the `WeaponScript` of `SelectedWeapon`.
- Copies `bulletsLeft` and `clips`:
  - machine gun and shotgun: both;
  - grenade launcher: `clips = ammoCount`;
  - knife: nothing.
- If `currentWeapon != SelectedWeapon`: `color = Lerp(color, 0.3, dt·20)`, and once `color < 0.32`, `currentWeapon = SelectedWeapon`.

**`OnGUI`** uses skin 40959:
- `GUI.color = (1,1,1,0.9)`.
- Box at `(W-110, H-55, 100, 45)`:
  - knife: `"∞"`;
  - grenade launcher: `clips`;
  - otherwise `bulletsLeft + " | " + clips`.

  `clips` is a float field printed with `float.ToString()`.
- `if (weaponManager)`: a `GUILayout` horizontal row of boxes, one per weapon (`weaponName`, `Height(30)`), across the top of the screen.
  - The current weapon's box uses `GUI.color` alpha `color = Lerp(color, 0.9, dt·20)`.
  - The others use alpha 0.3.

The knife's `"∞"` (U+221E) is not in the built-in font, so the box is empty. This is **inferred**: Unity's font fallback in WebGL is not established.

## WeaponManager.OnGUI (`f29925`)

All of this uses the default skin.

**Auto-fire text:**
- If `textDelay > 0`: `GUI.color` yellow `(1, 0.9215686, 0.0156863)`.
- `Label` style `fontSize = 30` and `alignment = UpperCenter`.
- Label at `(W/2-150, H/3-25, 300, 50)`: "Auto fire on" or "Auto fire off".
- `textDelay` starts at 1 (constructor, `f29926`), so "Auto fire on" shows for the first second.

**`shownowepinroom2 > 0`** (only set in the full-game check, unused here):
- Decrements by 0.1 per call.
- Green colour, box `fontSize = 35`.
- Box at `(W/2 - W/2, 140, W, 50)`: "This Weapon Is Available Only At FULL Game !!!".
- Then box `fontSize = 12`.

**`shownowepinroom > 0`** (set to 40 by a slot the room does not allow, `docs/gameplay/weapons.md`):
- Decrements by 0.1 per call.
- Yellow colour, box `fontSize = 38`.
- Box at y 150: "This Weapon Is Disabled In This Room !!!".
- Then box `fontSize = 12`.

Constructor values: `autoFire = true`. `Awake` therefore sets every weapon's `autoFire = false`. The client earlier had both reversed; this is now fixed.

## Spawn protection (`PlayerNetworkController.OnGUI`, `f37905`)

- While the owner is spawn-protected, a default-skin label is drawn at `(5, H-38, W-5, 50)` with the string literal `"<size=32><color=yelow>>SPAWN PROTECTION ACTIVE</color></size>"`.
- `yelow` is not a valid colour, so the `<color>` tags are shown as text at size 32 (**inferred**, to be compared with the original).
- Protection lasts 5 s from the constructor (`PlayerNetworkController.Update`, `f37896`).
