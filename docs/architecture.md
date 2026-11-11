# Architecture

Client: `crates/pga3-client` (binary). Procedural, data-oriented Rust; GL calls (`unsafe`) are confined to `gfx.rs` and `render.rs`.

## Dependencies (decisions)

| Crate | Use | Why |
|-------|-----|-----|
| `winit` 0.30 | window, input | de facto standard, Wayland + X11 |
| `glutin` 0.32 + `glutin-winit` 0.5 + `raw-window-handle` 0.6 | GL 3.3 core context | pairs with winit |
| `glow` 0.18 | OpenGL bindings | same API on desktop GL and WebGL2, keeps a wasm build possible |
| `glam` 0.34 | math | **chosen over nalgebra**: game-oriented, small, fast, has the quaternion/matrix ops Unity uses. Revisit only if physics needs nalgebra (e.g. rapier). |
| `image` 0.25 (png only) | texture loading | |
| `serde` / `serde_json` | `assets/data/*.json` | |
| `anyhow` | error handling | |
| `cpal` 0.15 | audio output | plain cross-platform device access; the mixer is ours (`audio.rs`) because the WebGL build's model (pan-only panner + gain set by Unity) must be reproduced exactly |
| `symphonia` 0.5 (`aac`, `isomp4`) | decoding the original `.m4a` clips | pure Rust; the clips stay byte-exact AAC in `assets/` |
| `fontdue` 0.9 | glyph rasterisation for `OnGUI` text | small, pure Rust, reads the published Liberation Sans (`assets/fonts/Arial.ttf`) |
| `tungstenite` 0.30 (`rustls-tls-webpki-roots`) + `rustls` 0.23 (`ring`) | Photon's secure WebSocket | synchronous, run on one thread per connection (`net/socket.rs`); pure-Rust TLS, no OpenSSL; Photon's own Protocol16 codec is ours (`net/protocol16.rs`) |

**Physics: custom** (`physics.rs`), not rapier: the game only needs static triangle-mesh/box queries plus Unity's CharacterController. That controller is PhysX 3.4's character controller, whose source is public (BSD-3-Clause, NVIDIAGameWorks/PhysX-3.4, `PhysXCharacterKinematic/src/CctCharacterController.cpp`). `physics.rs` ports its `moveCharacter` / `doSweepTest` / `collisionResponse` / `computeMTD` and the sweep rules of `GuSweepCapsuleTriangle.cpp` / `GuSweepsMesh.cpp` for the settings this game uses (details: `docs/gameplay/movement.md`). A general rigid-body engine would not reproduce that behaviour. Networking: `net/` (below), specified in `docs/protocol.md`.

## Modules

| File | Role |
|------|------|
| `main.rs` | args, window/GL setup, event loop, walk/fly modes, screenshot mode |
| `assets.rs` | finds `assets/`, parses `data/materials.json` and `data/textures.json` |
| `scene.rs` | `data/scenes/<Scene>.json` → world matrices, renderers, directional lights, camera, spawn points, RenderSettings |
| `obj.rs` | OBJ meshes from `assets/models/` back into Unity mesh space |
| `physics.rs` | collision world (scene MeshColliders/BoxColliders in a grid, with layers, tags and trigger flags), exact capsule sweeps, `Raycast` (grid DDA, front faces), `SphereCast` / `CapsuleCast`, capsule-box trigger overlap, CharacterController (port of the PhysX 3.4 controller) |
| `player.rs` | local player: ports of `FPScontroller`, `FPSinput`, `FPSMouseLook`, `CameraBob`, `WalkSway`, `LadderPlayer` / `Ladder`; Unity frame order and keyboard axes (spec: `docs/gameplay/movement.md`) |
| `gfx.rs` | GL helpers: programs, mesh/texture upload, framebuffer readback |
| `render.rs` | scene renderer: ports of the original shaders; prefab models (static, GPU-skinned, `Mobile/Particles/Additive`) per camera culling mask, also at spawned-instance transforms; `OnGUI` texture quads |
| `prefab.rs` | instances a `data/prefabs/resources/<name>.json` prefab: node hierarchy, renderers (meshes, built-in primitives, skins), `Animation` components, cameras, world matrices |
| `anim.rs` | legacy `AnimationClip` evaluation and the `Animation` component (ports of Unity's native Play/CrossFade/update; spec: `docs/gameplay/animation.md`) |
| `skin.rs` | skin sidecars (`data/skins/`), quality bone limit (`blendWeights`) |
| `character.rs` | ports of `WeaponAnimation` (first-person hands) and `CharacterAnimation` (third-person body) |
| `weapons.rs` | ports of `WeaponManager` (selection, switching), `WeaponScript` (aiming, sway, firing, reloading, recoil; coroutines as explicit continuations), `WeaponCrosshair`, `SniperScope` (spec: `docs/gameplay/weapons.md`) |
| `projectiles.rs` | spawned `Resources` prefabs: `Bullet` (raycast stepping, impacts by tag), `Projectile` (rocket sphere sweep, explosion), destroy timing |
| `unity.rs` | Unity math (`Quaternion.Euler`/`AngleAxis`/`LookRotation`/`FromToRotation`/`Slerp`, `Mathf.Lerp`) and `UnityEngine.Random` (Xorshift128) |
| `primitives.rs` | Unity's built-in Cube/Cylinder, generated in code (not published) |
| `audio.rs` | `AudioSource`/`AudioListener` and the mixer, as the Unity WebGL build plays them (spec: `docs/gameplay/audio.md`) |
| `footsteps.rs` | port of `FPSSoundController` |
| `particles.rs` | Shuriken `ParticleSystem` simulation and geometry (spec: `docs/gameplay/particles.md`) |
| `trail.rs` | `TrailRenderer` (bullet and rocket trails) |
| `effects.rs` | the spawned prefabs' effects: impacts (`WaitForDestroy`), the explosion (`CFX_AutoDestructShuriken`, `ExplosionDamage`), projectile trails and sounds |
| `gui.rs` | IMGUI subset for `OnGUI`: skins, styles, boxes, labels, rich text, the built-in Arial font atlas (spec: `docs/gameplay/hud.md`) |
| `menu.rs` | the menu flow: server window, `ConnectMenu` lobby and room creation, `LoadMap` fade, PlayerPrefs, the in-game Tab menu (spec: `docs/gameplay/menu.md`) |
| `ui.rs` | the uGUI subset of the `Main` scene: canvas in the 3D scene, RectTransforms, Image, Text, Button animation, `TiltWindow` |
| `layout.rs` | Unity's `GUILayout` group algorithm for the lobby |
| `hud.rs` | ports of `AmmoDisplay`, `WeaponManager.OnGUI` and the spawn-protection label |
| `damage.rs` | ports of `PlayerDamage`, spawn protection and the death prefab's `RagdollController` (spec: `docs/gameplay/damage.md`) |
| `remote.rs` | another player's copy: the remote branches of `PlayerNetworkController` (serialized state, animation layers, third-person weapon), `PlayerDamage`/`HitBox` (`SendMessageUpwards` chain, `DoDamage`, hit marker), `WeaponSync`, `DrawPlayerName` (spec: `docs/gameplay/remote-players.md`) |
| `room.rs` | `RoomMultiplayerMenu` teams/spawn points, `WhoKilledWho` kill feed, the owner's serialized state, and the `--dummy` simulated client (spec: `docs/gameplay/remote-players.md`) |
| `net/protocol16.rs` | Photon's Protocol16 values (byte-exact codec) |
| `net/photon.rs` | Photon messages over WebSocket (`0xF3` framing), operation/parameter/event codes |
| `net/socket.rs` | one `wss://` connection on its own thread, Photon's 1 s ping, round-trip time and server clock |
| `net/client.rs` | LoadBalancing client: name server → master (lobby, room list) → game server, properties, `RaiseEvent` |
| `net/pun.rs` | PUN 1.60: the CrazyGames app profile, `RpcList`, RPC / serialize / instantiate / destroy events and the player stream |
| `online.rs` | the game on the real servers: room choice/creation and every message the original client sends, turned into/from game events (spec: `docs/protocol.md`, "Playing online") |

## Coordinate conventions (important for network compatibility)

- Game state stays in **Unity world space**: left-handed, +Y up, +Z forward, metres, Unity quaternions. Positions and rotations sent to/received from the original servers use this space, so no conversion is applied anywhere in game logic.
- Transforms: `local = T * R * S`, `world = parent * local`, quaternion → matrix with the usual formula (it is the same for Unity's left-handed convention).
- Rendering mirrors Unity on OpenGL: view = `scale(1, 1, -1) * inverse(cameraToWorld)`, projection = GL perspective (depth -1..1, vertical FOV), **front faces are clockwise** (`glFrontFace(GL_CW)`; flipped per draw for negative-determinant transforms).
- `assets/models/*.obj` are mirrored by UnityPy (X negated, winding reversed); `obj.rs` undoes both. PNGs are top-down, so textures are flipped on upload (GL/Unity sample row 0 at v = 0).

## Rendering (matches the original build's settings)

Evidence: `assets/data/settings/PlayerSettings.json` (`m_ActiveColorSpace` 0 = gamma), `QualitySettings.json` (default level 3 "Good": 1 pixel light, shadows off, no MSAA), compiled shader programs in `assets/shaders/`.

- Gamma colour space: no sRGB textures or framebuffer.
- `Mobile/Diffuse` / `Diffuse`: `albedo * _LightColor0 * max(0, N·L) + albedo * ambient`; `Diffuse` multiplies albedo by `_Color`. Main light = brightest directional light (`color * intensity`). Ambient: flat `m_AmbientSkyColor` converted to linear (as Unity stores SH) and back with Unity's `LinearToGammaSpace` approximation, as in the compiled vertex program (**inferred** that Unity linearises the ambient colour in gamma mode; verify with original screenshots).
- Fog (`FOG_LINEAR/EXP/EXP2`): `unity_FogParams` formulas with clip-space z, computed per vertex as in the compiled programs; the skybox is not fogged.
- `Mobile/Skybox`: 6 faces (Front +Z, Back -Z, Left +X, Right -X, Up +Y, Down -Y), drawn first without depth test/write.
- `BlendedDecal`: unlit `_MainTex * _Color`, alpha test `a > 0`, ZWrite off, polygon offset -1,-1, drawn after opaques.
- Texture sampling from `data/textures.json`: point → `NEAREST(_MIPMAP_NEAREST)`, bilinear → `LINEAR(_MIPMAP_NEAREST)`, trilinear → `LINEAR_MIPMAP_LINEAR`; wrap repeat/clamp/mirror. Mipmaps are generated by GL (Unity's own mip chain is not exported; box-filtered in both).
- Unity material slot rule: slot *i* draws submesh *i*; extra slots redraw the last submesh.
- Skinned meshes: GPU skinning with a bone palette (`bone.localToWorld · bindpose`, max 64), one bone per vertex as quality level "Good" (`blendWeights = 1`) dictates. Player prefab: Main Camera pass (mask 309), then depth clear and the Weapon Camera pass (layer 9) for the first-person rig. Details: `docs/gameplay/animation.md`.

Particles and trails (`Mobile/Particles/*`, `Particles/*`, `WFX/*`) are drawn as dynamic batches sorted back to front after opaques and decals (`docs/gameplay/particles.md`); `OnGUI` output is a list of textured quads drawn last. Not implemented yet: `Standard`, point/spot lights, the menus (uGUI/`OnGUI` menu screens).

## Running

```bash
cargo run --release -p pga3-client -- [Scene] [--spawn N | --camera x,y,z,yaw,pitch [--walk]] [--fly] [--stats] [--wait SECONDS] [--screenshot out.png]
    [--weapon KEY] [--fire] [--aim] [--mode DM|TDM] [--team 1|2] [--name NAME]
    [--dummy [--dummy-team 1|2] [--dummy-spawn N] [--dummy-weapon SLOT] [--dummy-fire]]
    [--online [--region eu] [--room TEXT] [--create]] [--list-rooms [--region eu]]
```

Scenes: `Bridge Castle Crash Fight Pete Scaz Snow Tunnels Village` and `ZM_*` variants. The game starts walking at spawn point N (default: a random spawn point of the player's team, as `SpawnPlayer` picks it) with the original controls: WASD/arrows, Space jump, Left Shift run, C crouch, Left Ctrl prone, mouse look. F toggles a free-fly camera (WASD, Space/C up/down, Shift fast; F again drops the player there); `--camera`/`--fly` start in fly mode; `--camera ... --walk` instead puts the walking player at that camera. Click captures the mouse, Esc releases/quits, Tab goes to the next spawn point, P prints the current `--camera` value. `--screenshot` renders a few frames, saves a PNG and exits; agents use it to verify rendering (`--wait S` delays the screenshot by S seconds, e.g. to capture an animation in progress). `--team 2` (default, team "Mercenary") plays the `german` prefab, `--team 1` ("German-ZM") `mercenarys`, whose Sniper and Launcher differ (`docs/gameplay/weapons.md`); `--mode` sets the room's game mode (DM default) and `--name` the player name. Walk mode shows its first-person weapons with the original weapon controls (left mouse fire, right mouse aim, R reload, 1-6 / wheel / M, N select; the mouse buttons only act while the cursor is captured), the crosshair and the sniper scope; fly mode also shows its third-person body as the other players see it (a copy fed by its own serialized state). `--dummy` adds a simulated second client whose player stands at its team's spawn point (`--dummy-spawn N` overrides it) holding `allWeapons[SLOT]` (`--dummy-weapon`, default 2 = M242) and, with `--dummy-fire`, firing it; it can be shot and killed, and respawns (`docs/gameplay/remote-players.md`). `--weapon KEY` presses a number key on the first frame and `--fire` holds fire (clicking every 30 frames), for screenshots of the weapons. Sound plays through the default output device (the game runs silently without one; `--screenshot` is always silent). `--stats` prints the frame rate, frame times and CPU time once per second. `--online` plays on the live CrazyGames build's Photon servers (region `--region`, default `eu`): it joins the most populated open room of this game (`--room TEXT` picks one by name) or creates one (always with `--create`), on the room's map and mode (the scene argument and `--mode` only apply to a created room; `DM` creates a `TDM` room). `--list-rooms` prints the region's rooms and exits. `PGA3_NET_DEBUG=1` prints incoming network events. The assets folder is found automatically or via `PGA3_ASSETS`.

Frame pacing: the client renders once per display refresh. On Wayland, winit paces `RedrawRequested` to the compositor's frame callbacks (`Window::pre_present_notify` before each swap), so the EGL swap interval is 0; a blocking swap on top of that waited a second refresh and halved the frame rate (72 fps on a 144 Hz monitor). On X11 and other platforms the swap interval 1 is the vsync. `PGA3_NO_VSYNC=1` renders uncapped (benchmarking). Measured on the maintainer's machine: 144 fps on a 144 Hz monitor, 0.1 ms CPU per frame, player movement 7-34 µs per frame.
