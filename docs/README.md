# docs/

Distilled, evidence-backed knowledge about the original game. Keep each claim labelled as **observed**, **inferred** or **unknown**, with a pointer to the evidence (REA evidence ID, capture file, or asset path).

Existing:

- `original-build.md`: Unity WebGL build layers (brotli → UnityWebData → UnityFS) and IL2CPP metadata.
- `code-structure.md`: assemblies, game classes, scenes.
- `protocol.md`: multiplayer (PUN Classic 1.91 / Photon Cloud), server settings, RPC list.
- `assets.md`: extraction pipeline, published asset counts, gaps.
- `architecture.md`: client crates and rationale, modules, coordinate conventions, renderer (shader ports), how to run.
- `unknowns.md`: open questions.
- `gameplay/movement.md`: player movement, input, mouse look, camera bob, CharacterController model (decompiled `FPScontroller` & co.).
- `gameplay/animation.md`: legacy clips, the native `Animation` component (Play/CrossFade/update), skinning, built-in meshes, `WeaponAnimation`/`CharacterAnimation` ports.
- `gameplay/weapons.md`: `WeaponManager` (switching, room tokens), `WeaponScript` (aiming, firing, reloading, recoil), bullets, rockets, crosshair, sniper scope.
- `gameplay/audio.md`: how the Unity WebGL build plays `AudioSource`s, attenuation, the sounds ported, footsteps.
- `gameplay/particles.md`: Shuriken particle systems, particle shaders, trails, impacts, the rocket explosion.
- `gameplay/hud.md`: the IMGUI model, font, `AmmoDisplay`, `WeaponManager.OnGUI`, the spawn-protection label.
- `gameplay/damage.md`: damage sources, `PlayerDamage`, death prefab, respawn.

Planned:

- `gameplay/`: modes, progression, balance tables, zombies, remote players.

Tooling for reading the original code (wasm2c + IL2CPP names): `tools/re/README.md`.
