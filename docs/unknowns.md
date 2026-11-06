# Unknowns

| # | Question | Needed to resolve |
|---|----------|-------------------|
| 1 | Which body mesh a remote player shows (the `german` prefab carries several body meshes; `localObjectsToDeactivate` hides `Base_mesh`, `Root_jnt`, `Terrorist_02_mesh`, `remoteObjectsToDeactivate` hides `LookObject`, `SpawnObject`, `Base_mesh`; the client's remote view currently hides only `LookObject`) and how `HeadLookController`/`ThirdPersonWeaponControl` pose the body | Decompile `PlayerNetworkController.Start`/team setup and `HeadLookController`; screenshot a remote player in the original |
| 2 | Exact Photon connect flow, AppVersion string, room/player properties, serialization | Decompile the PUN/menu code with `tools/re/` + runtime capture (PUN 1.91 on Photon Cloud is established) |
| 3 | Whether GameArter overrides the Photon AppID at runtime / is required to play | Runtime capture |
| 4 | Remaining method bodies (`PlayerDamage`, `ExplosionDamage`, `AmmoDisplay`, `WeaponSync`). Decompilation works (`tools/re/`); movement, ladders and weapons are done (`docs/gameplay/movement.md`, `docs/gameplay/weapons.md`) | Decompile per system as it is implemented |
| 5 | Is there music beyond the 22.8 s lobby loop (the "OST")? | Play the original and compare; check network for streamed audio |
| 6 | Redistribution of bundled commercial fonts (Ebrima, Impact) | User decision |
| 7 | How Unity 2018.3 maps CharacterController settings onto the PhysX 3.4 controller (the controller itself is ported from PhysX source): cylinder height = height - 2·radius, `scaleCoeff` of the trigger proxy (0.8 assumed), overlap recovery on, `eEASY` climbing; also whether Unity's `CapsuleCast` really skips colliders it starts inside | Measure in the original: stand under 2.3 m / 2.4 m ceilings, walk into a 0.6 m and a 0.7 m wall, find where a ladder latches; compare positions |
| 8 | Mouse look scale: Unity WebGL `Mouse X/Y` units (raw pointer-lock pixels × 0.1 is assumed) and whether `CF2Input` axes fall back to Unity `Input` on desktop | Turn 1000 px in the original at sensitivity 1.8 and measure the angle |
| 9 | Legacy `Animation` blending of several states: per-property weight normalisation is inferred (single-state playback and cross-fades are decoded exactly, `docs/gameplay/animation.md`) | Decompile the native blend/bind code, or compare a mid-cross-fade pose with the original |
| 10 | `CharacterController.velocity` is taken as the last `Move`'s displacement / delta time (used by `WeaponScript.SmoothMove`) | Decompile the native `CharacterController::get_velocity` / `Move` |
| 11 | Room name: `WeaponManager` only allows slot n ≥ 1 when the Photon room name contains `[WEP0(n+1)]`. Rooms are not implemented; the client assumes `[WEP01]`…`[WEP06]` (all six weapons) and `SuperWeapons` = false (the client is not served from pacogames.com) | Capture real room names (room list) once networking exists |
| 12 | Native details the weapon port assumes: `Quaternion.Slerp` formula (incl. the zero `camPos` before the first recoil), `UnityEngine.Random` seeding/`Range` rounding, `GUI.color` reset to white each `OnGUI` frame, `Mobile/Particles/Additive` render state (Unity's built-in source, not in the shader dump) with missing vertex colours read as white, PhysX CCD contact position for the rocket (sphere sweep to first contact) | Decompile the native functions, or compare captures of the original (recoil sway, crosshair colour, muzzle flash brightness, rocket impact point) |
| 13 | Unity frame details: weapons instantiated during `Update` start next frame; `Destroy` takes effect at the end of the frame (before rendering); pending `Start`s run before any `Update` | Standard Unity behaviour (documented), not verified on this build |

Resolved:

- Skinning data (former row 1): exported by `tools/extract/export_skins.py` to `assets/data/skins/`; legacy `Animation` playback and GPU skinning implemented (`docs/gameplay/animation.md`).
- Renderer fidelity (former row 7): the user compared screenshots of the original and this client at the same spot on Bridge and found them identical; no visible issues on Castle.
