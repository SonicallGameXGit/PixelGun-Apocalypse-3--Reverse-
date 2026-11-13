# docs/

Distilled, evidence-backed knowledge about the original game. Keep each claim labelled as **observed**, **inferred** or **unknown**, with a pointer to the evidence (REA evidence ID, capture file, or asset path).

Existing:

- `original-build.md`: Unity WebGL build layers (brotli → UnityWebData → UnityFS) and IL2CPP metadata.
- `crazygames-build.md`: the live CrazyGames build (Unity 5.3.5, asm.js): files, data package, IL2CPP registrations, differences from Build-7, decompiled menu facts.
- `code-structure.md`: assemblies, game classes, scenes.
- `protocol.md`: multiplayer (PUN Classic on Photon Cloud): server settings, the recorded CrazyGames session, what this client sends online.
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
- `gameplay/remote-players.md`: other players' copies, teams, the kill feed, the `--dummy` client.
- `gameplay/match-flow.md`: the room lifecycle: modes and zombie maps, round timer, round end and restart, kill feed, respawns, late join, master client, leaving, kicks, disconnects.

Planned:

- `gameplay/`: progression, balance tables, chat.

Tooling for reading the original code (wasm2c, or asm.js for the CrazyGames build, + IL2CPP names): `tools/re/README.md`.
- `gameplay/menu.md`: boot, server window, lobby, room creation, loading fade and the in-room Tab menu of the CrazyGames build, with the client's implementation.
- `gameplay/settings.md`: the in-room menu (`RoomMultiplayerMenu`), sensitivity, quality, resolutions, the client volume, PlayerPrefs and `EnableHelper`.
