# Pixel Gun Apocalypse 3: Open-Source Client

A clean-room, open-source reimplementation of **Pixel Gun Apocalypse 3** (PacoGames, originally Unity WebGL), written in **Rust + OpenGL**.

## Goal

Logical parity with the original game:

- identical visuals, audio, UI, controls, weapons and abilities;
- gameplay and physics matching the original (target 95%+);
- **multiplayer compatibility**: this client joins the real original servers and plays alongside players on the official client.

The implementation is independent. It has no Unity and no original code, only behaviour reproduced from observation.

## Status

Reverse engineering in progress; not playable yet. The client runs on the assets and code of the CrazyGames version of the game (Unity 5.3), whose servers it plays on; the PacoGames Build-7 (Unity 2018.3) it was first ported from differs in map textures and some script details ([`docs/crazygames-build.md`](docs/crazygames-build.md)). The client loads every map, renders it with the original shaders (textures, lighting, fog, skybox), and lets you walk, run, jump, crouch, go prone and climb ladders with the original movement code ported from the decompiled game. The player prefab is instanced with its animations and skinned meshes, and the third-person body follows the original `CharacterAnimation` logic (idle, walk, run, crouch, prone, jump). The six first-person weapons (Machete, Pistol, M242, ShotGun, Sniper, Launcher) work with the original `WeaponManager`/`WeaponScript` code: switching, aiming (FOV zoom, sniper scope), fire rates, spread, recoil, reloading, muzzle flashes, bullets with impacts, the rocket, the other team's grenade launcher, and the dynamic crosshair. Sounds (weapons, footsteps, impacts, death) play as the Unity WebGL build plays them; impacts and the rocket explosion are simulated Shuriken particle systems, the shotgun smoke a legacy particle emitter, the rocket trail a trail renderer; the HUD (HP, ammo, weapon bar, room messages and spawn-protection text) is a port of the original `OnGUI` scripts; fall damage kills the player, who respawns at a team spawn point after the original death sequence. Other players' copies are ported as the network would drive them (the serialized state, third-person animation and weapons, remote shots, hit boxes, `DoDamage`, kills, the kill feed and teammates' names); `--online` connects to the live servers of the CrazyGames version of the game (Photon), joins or creates a room and plays in it with the original messages, and `--dummy` adds a simulated second player to shoot at offline. Rounds end as in the original: the winner box ("<team> Won... Restarting" or "<player> is Winner... Restarting"), then scores reset and everyone respawns; the `ZM_*` maps play team 1 as zombies; players who join late see everyone, and leaving, being kicked or losing the connection fade back to the menu ([`docs/gameplay/match-flow.md`](docs/gameplay/match-flow.md)). The original menus lead into a game: the server window, the lobby with its room list and room creation, and the in-room Tab menu (team selection, player list, controls, mouse sensitivity, resolution and quality), with the player name, kills, deaths and sensitivity saved as PlayerPrefs on disk. Collision uses a port of the PhysX 3.4 character controller that the original runs, against the original map geometry. Done so far: the original build is unpacked, the code structure is recovered, the multiplayer stack is identified (PUN Classic on Photon Cloud) and its traffic recorded, and the assets are extracted into `assets/`: textures, sprites, audio, meshes, fonts, shaders, plus all scenes, prefabs, materials, settings and decoded script data (weapon stats, movement values, ...) as JSON. See [`docs/`](docs/).

## Assets

Original assets (textures, sounds, the OST, models, ...) are extracted from the Unity bundles into plain files under [`assets/`](assets/), never kept in packed bundles, so anyone can browse and reuse them.

## Layout

| Path | Purpose |
|------|---------|
| `crates/pga3-client/` | Game client (Rust + OpenGL) |
| `assets/` | Extracted original assets as regular files (`manifest.json` maps each to its Unity origin) |
| `tools/extract/` | Reproducible asset extraction pipeline |
| `tools/re/` | Helpers for reading the original game code (wasm2c + IL2CPP names) |
| `tools/net/` | Decoding recorded Photon traffic, building the protocol test fixture |
| `docs/` | Findings, protocol and gameplay specs, architecture |
| `research/` | Raw RE material (gitignored): saved page, original build, captures |
| `AGENTS.md` | Context and rules for AI agents |

## Reverse engineering

Done with the [REA (Reverse Engineer Anything)](https://github.com/morluto/rea) plugin. Original sources: <https://www.pacogames.com/game/pixel-gun-apocalypse-3/>. See [`AGENTS.md`](AGENTS.md) and [`docs/`](docs/) for the methodology.

## Build

```bash
cargo run --release -p pga3-client                # menu: server -> lobby (join or create a room) -> game; Tab in game: room menu
cargo run --release -p pga3-client -- Bridge      # straight into a map: Bridge Castle Crash Fight Pete Scaz Snow Tunnels Village, ZM_*
```

The menu connects to the live CrazyGames servers. `--menu boot|main|connecting|lobby|create|loading [--demo-rooms] --screenshot out.png` renders a menu screen offline (`docs/gameplay/menu.md`).

Click to capture the mouse. Original controls: WASD/arrows move, Space jump (and let go of a ladder), Left Shift run, C crouch, Left Ctrl prone; walk into a ladder to climb it, look down to climb down. Left mouse fire, right mouse aim, R reload, 1-6 / mouse wheel / M, N select a weapon. In a game started from the menu, Tab opens the room menu (pick a team to spawn) and P toggles fullscreen; in a game started directly, F toggles free-fly, Tab jumps to the next spawn point and P prints the camera. Esc releases the mouse / quits. `--volume 0..1` sets the client volume (saved), `--quality 0..5` the quality level, `--round-time SECONDS` the round length of a game started directly. `--team 1` plays the "German-ZM" team (the `Mercenarys` prefab, whose Sniper and Launcher are different weapons: a green-flash sniper and a grenade launcher); the default `--team 2` plays `German`. `--dummy` (with `--dummy-fire`) adds a simulated other player that can be shot and killed. `--online` plays on the real servers (`--list-rooms` shows the rooms, `--create` makes a new one, `--room TEXT` picks one). More options: [`docs/architecture.md`](docs/architecture.md).

Requires a recent stable Rust toolchain (see `rust-toolchain.toml`) and OpenGL 3.3. Crates: winit, glutin, glow, glam, image, serde_json, anyhow, cpal, symphonia, fontdue, tungstenite, rustls (rationale in `docs/architecture.md`).

## Legal

Pixel Gun Apocalypse 3 and its assets belong to their respective owners. This is a fan preservation project and is not affiliated with PacoGames or the original developers. Code is released under the [MIT License](LICENSE).
