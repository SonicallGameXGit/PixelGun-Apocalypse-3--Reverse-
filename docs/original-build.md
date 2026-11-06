# Original build: structure

All facts **observed** from files in `research/unity-build/` unless marked otherwise.

## Source

- Manifest: `https://www.pacogames.com/game/pixel-gun-apocalypse-3/Build-7/pga3-v18.json`
- Referenced files (same directory), downloaded 2026-11-06:

| File | Size | Content |
|------|------|---------|
| `pga3-v18.data.unityweb` | 10,404,686 | asset container |
| `pga3-v18.wasm.code.unityweb` | 4,789,069 | IL2CPP-compiled game code (WebAssembly MVP) |
| `pga3-v18.wasm.framework.unityweb` | 80,442 | Emscripten JS framework/glue |

Manifest: `companyName "DefaultCompany"`, `productName "Pixel Gun Apocalypse 3"`, `TOTAL_MEMORY 268435456` (256 MiB), `graphicsAPI ["WebGL 1.0", "WebGL 2.0"]`, background `#231F20`, splash style `Dark`.

## Engine

- **Unity 2018.3.14f1** (from the `UnityFS` bundle header of `data.unity3d`). `unity_default_resources` carries the string `2018.3.11f1` (built-in resource version; not the player version).
- Scripting backend: **IL2CPP** (wasm + `global-metadata.dat`), `scripting-runtime-version=latest` (.NET 4.x equivalent) from `boot.config`.

## Layer 1: `.unityweb` compression

Each `.unityweb` file is **brotli**. It begins with a brotli metadata block holding the ASCII marker `UnityWeb Compressed Content (brotli)`. Decompress with plain `brotli -d`.

Decompressed (in `research/unity-build/raw/`): `pga3-v18.data` (22.0 MB), `pga3-v18.wasm` (22.3 MB), `pga3-v18.framework.js` (512 KB).

## Layer 2: `UnityWebData1.0` container (`pga3-v18.data`)

Layout: magic `UnityWebData1.0\0` (16 bytes), `u32le headerSize`, then entries until `headerSize`: `u32le offset, u32le size, u32le nameLen, name[nameLen]`. Contents:

| Path | Size |
|------|------|
| `data.unity3d` | 12,301,882 |
| `Resources/unity_default_resources` | 3,376,860 |
| `Il2CppData/Metadata/global-metadata.dat` | 5,995,248 |
| `resources.resource` | 1,090,981 |
| `sharedassets0.resource` | 284,673 |
| `Managed/mono/2.0/machine.config` | 27,625 |
| `boot.config` | 123 |

`*.resource` files are raw streamed data (textures/audio payloads) referenced by offset from serialized files inside `data.unity3d`.

## Layer 3: `data.unity3d`: UnityFS bundle

Header: `UnityFS`, format version 6, `5.x.x`, `2018.3.14f1`, total size 12,301,882, blocks-info compressed 2,410 / uncompressed 8,726 bytes, flags `0x43` → compression type 3 = **LZ4HC**, block info at end-of-header. Contains `globalgamemanagers(.assets)`, `level0`–`level19` (scenes), `sharedassets0`–`19.assets`, `resources.assets`, `unity_builtin_extra`. Unpacked with UnityPy: 47,491 objects. See `docs/assets.md` and `docs/code-structure.md`.

## IL2CPP metadata

- `global-metadata.dat`: magic `0xFAB11BAF`, version **24**, header size **272 bytes**, i.e. the **v24.1** variant introduced in Unity 2018.3 (base v24 has 264 bytes).
- REA `inspect_il2cpp_metadata` rejects it (`unexpected-header-size: stringLiterals starts at 272, not immediately after the 264-byte header`; evidence `ev_9983a375…`). Il2CppDumper (v24.1-aware, WebGL wasm supported) succeeded: output in `research/il2cpp-dump/`, summary in `docs/code-structure.md`.
