# Pixel Gun Apocalypse 3: Open-Source Client

A clean-room, open-source reimplementation of **Pixel Gun Apocalypse 3** (PacoGames, originally Unity WebGL), written in **Rust + OpenGL**.

## Goal

Logical parity with the original game:

- identical visuals, audio, UI, controls, weapons and abilities;
- gameplay and physics matching the original (target 95%+);
- **multiplayer compatibility**: this client joins the real original servers and plays alongside players on the official client.

The implementation is independent. It has no Unity and no original code, only behaviour reproduced from observation.

## Status

Reverse engineering in progress; not playable yet. The client loads every map, renders it identically to the original (textures, lighting, fog, skybox), and lets you walk, run, jump, crouch and go prone with the original movement code ported from the decompiled game, on collision with the original map geometry. Done so far: the original build is unpacked, the code structure is recovered, the multiplayer stack is identified (PUN Classic 1.91 on Photon Cloud), and the assets are extracted into `assets/`: textures, sprites, audio, meshes, fonts, shaders, plus all scenes, prefabs, materials, settings and decoded script data (weapon stats, movement values, ...) as JSON. See [`docs/`](docs/).

## Assets

Original assets (textures, sounds, the OST, models, ...) are extracted from the Unity bundles into plain files under [`assets/`](assets/), never kept in packed bundles, so anyone can browse and reuse them.

## Layout

| Path | Purpose |
|------|---------|
| `crates/pga3-client/` | Game client (Rust + OpenGL) |
| `assets/` | Extracted original assets as regular files (`manifest.json` maps each to its Unity origin) |
| `tools/extract/` | Reproducible asset extraction pipeline |
| `tools/re/` | Helpers for reading the original game code (wasm2c + IL2CPP names) |
| `docs/` | Findings, protocol and gameplay specs, architecture |
| `research/` | Raw RE material (gitignored): saved page, original build, captures |
| `AGENTS.md` | Context and rules for AI agents |

## Reverse engineering

Done with the [REA (Reverse Engineer Anything)](https://github.com/morluto/rea) plugin. Original sources: <https://www.pacogames.com/game/pixel-gun-apocalypse-3/>. See [`AGENTS.md`](AGENTS.md) and [`docs/`](docs/) for the methodology.

## Build

```bash
cargo run -p pga3-client -- Bridge          # any map: Bridge Castle Crash Fight Pete Scaz Snow Tunnels Village, ZM_*
```

Click to capture the mouse. Original controls: WASD/arrows move, Space jump, Left Shift run, C crouch, Left Ctrl prone. F toggles free-fly, Tab jumps to the next spawn point, Esc releases the mouse / quits. More options: [`docs/architecture.md`](docs/architecture.md).

Requires a recent stable Rust toolchain (see `rust-toolchain.toml`) and OpenGL 3.3. Crates: winit, glutin, glow, glam, image, serde_json, anyhow (rationale in `docs/architecture.md`).

## Legal

Pixel Gun Apocalypse 3 and its assets belong to their respective owners. This is a fan preservation project and is not affiliated with PacoGames or the original developers. Code is released under the [MIT License](LICENSE).
