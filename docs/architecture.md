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

**Physics: custom** (`physics.rs`), not rapier: the game only needs static triangle-mesh/box queries plus Unity's CharacterController. That controller is PhysX 3.4's character controller, whose source is public (BSD-3-Clause, NVIDIAGameWorks/PhysX-3.4, `PhysXCharacterKinematic/src/CctCharacterController.cpp`). `physics.rs` ports its `moveCharacter` / `doSweepTest` / `collisionResponse` / `computeMTD` and the sweep rules of `GuSweepCapsuleTriangle.cpp` / `GuSweepsMesh.cpp` for the settings this game uses (details: `docs/gameplay/movement.md`). A general rigid-body engine would not reproduce that behaviour. Audio and networking crates are not chosen yet (see AGENTS.md "Tech stack").

## Modules

| File | Role |
|------|------|
| `main.rs` | args, window/GL setup, event loop, walk/fly modes, screenshot mode |
| `assets.rs` | finds `assets/`, parses `data/materials.json` and `data/textures.json` |
| `scene.rs` | `data/scenes/<Scene>.json` → world matrices, renderers, directional lights, camera, spawn points, RenderSettings |
| `obj.rs` | OBJ meshes from `assets/models/` back into Unity mesh space |
| `physics.rs` | collision world (scene MeshColliders/BoxColliders in a grid, with layers and trigger flags), exact capsule sweeps, `SphereCast` / `CapsuleCast`, capsule-box trigger overlap, CharacterController (port of the PhysX 3.4 controller) |
| `player.rs` | local player: ports of `FPScontroller`, `FPSinput`, `FPSMouseLook`, `CameraBob`, `WalkSway`, `LadderPlayer` / `Ladder`; Unity frame order and keyboard axes (spec: `docs/gameplay/movement.md`) |
| `gfx.rs` | GL helpers: programs, mesh/texture upload, framebuffer readback |
| `render.rs` | scene renderer: ports of the original shaders; prefab models (static and GPU-skinned) per camera culling mask |
| `prefab.rs` | instances a `data/prefabs/resources/<name>.json` prefab: node hierarchy, renderers (meshes, built-in primitives, skins), `Animation` components, cameras, world matrices |
| `anim.rs` | legacy `AnimationClip` evaluation and the `Animation` component (ports of Unity's native Play/CrossFade/update; spec: `docs/gameplay/animation.md`) |
| `skin.rs` | skin sidecars (`data/skins/`), quality bone limit (`blendWeights`) |
| `character.rs` | ports of `WeaponManager`/`WeaponAnimation` (first-person hands), `WeaponScript`'s weapon motion (`Aiming`, `RotationRealism`, `SmoothMove`) and `CharacterAnimation` (third-person body) |
| `primitives.rs` | Unity's built-in Cube/Cylinder, generated in code (not published) |

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

Not implemented yet: particles (`Mobile/Particles/*`, `WFX/*`; prefab renderers using them are skipped), `Standard`, point/spot lights, UI.

## Running

```bash
cargo run --release -p pga3-client -- [Scene] [--spawn N | --camera x,y,z,yaw,pitch] [--fly] [--stats] [--wait SECONDS] [--screenshot out.png]
```

Scenes: `Bridge Castle Crash Fight Pete Scaz Snow Tunnels Village` and `ZM_*` variants. The game starts walking at spawn point N (default 0) with the original controls: WASD/arrows, Space jump, Left Shift run, C crouch, Left Ctrl prone, mouse look. F toggles a free-fly camera (WASD, Space/C up/down, Shift fast; F again drops the player there); `--camera`/`--fly` start in fly mode. Click captures the mouse, Esc releases/quits, Tab goes to the next spawn point, P prints the current `--camera` value. `--screenshot` renders a few frames, saves a PNG and exits; agents use it to verify rendering (`--wait S` delays the screenshot by S seconds, e.g. to capture an animation in progress). The local player is the `german` prefab: walk mode shows its first-person weapon (Machete), fly mode also shows its animated third-person body at the player's position. `--stats` prints the frame rate, frame times and CPU time once per second. The assets folder is found automatically or via `PGA3_ASSETS`.

Frame pacing: the client renders once per display refresh. On Wayland, winit paces `RedrawRequested` to the compositor's frame callbacks (`Window::pre_present_notify` before each swap), so the EGL swap interval is 0; a blocking swap on top of that waited a second refresh and halved the frame rate (72 fps on a 144 Hz monitor). On X11 and other platforms the swap interval 1 is the vsync. `PGA3_NO_VSYNC=1` renders uncapped (benchmarking). Measured on the maintainer's machine: 144 fps on a 144 Hz monitor, 0.1 ms CPU per frame, player movement 7-34 µs per frame.
