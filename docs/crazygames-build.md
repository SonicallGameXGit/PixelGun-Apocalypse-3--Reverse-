# CrazyGames build: structure

The live multiplayer target (`docs/protocol.md`, "Playing online"). All facts are **observed** from the files in `research/crazygames-build/`, unless marked otherwise. Build-7 (`docs/original-build.md`) is the PacoGames build the client was first ported from.

## Source

- Page: <https://games.crazygames.com/en_US/block-pixel-gun-apocalypse-3/index.html>. It was saved on 2026-11-07 to `research/source/crazygames-page/`.
- The page posts a `unity54config` message whose `loaderOptions` name these files in `https://files.crazygames.com/webgl/blocky-pixel-apocalypse-3/Release/` (downloaded 2026-11-07):

| File | Size | Content (gzip; name inside the gzip header) |
|------|------|---------|
| `pixelgunapocalypse3.2.datagz` | 13,239,298 | `pixelgunapocalypse3.2.data`, 58,831,132 bytes: the Emscripten file package |
| `pixelgunapocalypse3.2.jsgz` | 5,321,828 | `pixelgunapocalypse3.2.js`, 25.8 MB: asm.js code and Emscripten glue |
| `pixelgunapocalypse3.2.memgz` | 404,522 | `pixelgunapocalypse3.2.mem`, 1,880,818 bytes: static memory image |
| `UnityLoader.js` | 41,847 | loader, including the file package's table |

The gzip headers date the files to 2018-02-26. `PlayerSettings`: `productName "Block Pixel Gun Apocalypse 3"`, `bundleVersion "1.3"`.

## Engine

- **Unity 5.3.5f1** (serialized file headers).
- IL2CPP compiled to **asm.js** (not wasm).
- `global-metadata.dat` version **21**.

## Data package

The `.data` file has no header. `UnityLoader.js` lists its files as `new o(start, end, 0, 0).open("GET", "/path")`. There are 51 files, and together they cover every byte:
- `globalgamemanagers`, `level0`–`level19`, `sharedassets0`–`19.assets`, `resources.assets`;
- `Il2CppData/Metadata/global-metadata.dat`, `Resources/unity_default_resources`, `unity_builtin_extra`.

Unpack with `tools/extract/unpack_emscripten_package.py`.

## Scenes

These are the same 20 scenes as Build-7, from `Assets/A WEB crazy/`:
- `Main`;
- `Bridge`, `Castle`, `Crash`, `Credits`, `Fight`, `Pete`, `Scaz`, `Snow`, `Tunnels`, `Village`;
- the `ZM_*` variants of the maps.

## Code

Method names come from Il2CppDumper, run on the static memory wrapped as a wasm data section (`tools/re/asmjs_mem_to_wasm.py`).
- The registrations come from the asm.js call `OJ(1239772, 934140, 1870540)`, which is `il2cpp_codegen_register`: **CodeRegistration `0x12eadc`, MetadataRegistration `0xe40fc`**.
- The dumper finds only the first on its own, so the second is entered in manual mode.
- Output: `research/crazygames-il2cpp-dump/`.
- `tools/re/asmjs_index.py` maps 18,611 methods to asm.js functions through the per-signature tables (how-to: `tools/re/README.md`).

**`Assembly-CSharp` against Build-7:** 276 classes here, 441 there.
- The extra Build-7 classes are PacoGames' SDK (`Garter`, `HGarter*`), ads, `VersionChecker`, `ServerPaco` and PUN 1.91 helpers.
- This build adds the `InstantGui*` widgets, `Detonator*` effects, Easy Touch demo scripts, `InRoomRoundTimer` and `ServerTime`.
- The game scripts have the same methods in both builds. The exceptions are `ConnectMenu` (no `DoesRoomContainCode`/`GetRooms`/`GetSetRooms`/`Sort`), `Link`, `ServerChose` (no `ru`), `WeaponScript` (no `RaycastForZombies`) and `RoomMultiplayerMenu` (adds `Resolutions`).
- Method bodies have not been compared yet.

## Decompiled so far

- **`ConnectMenu`** (notes in `research/notes/crazygames/`):
  - `CreateRoom(name, isVisible true, isOpen true, maxPlayers, props, lobbyProps)`;
  - name = `newRoomName + ": " + gameMode + weapon tags + "PixelWarfare001v"`;
  - props `MapName` (the `AllMaps[i].mapName`), `RoundDuration` (minutes × 60), `GameMode`, `Password` (static `ConnectMenu.Password`);
  - lobby props = those four keys;
  - `OnJoinedRoom` starts `LoadMap(room.MapName)`.
  - There is no `Version` property, unlike Build-7 (no `VersionChecker`).
- **`RoomMultiplayerMenu.Awake`:**
  - sets player properties `TeamName = "Spectators"`, `Kills = 0`, `Deaths = 0`, one call each;
  - then, on the **master client**, room properties `RefTime = (float)PhotonNetwork.time`, `Team1Score = 0`, `Team2Score = 0`, one call each;
  - every other client unboxes `RefTime` as a float, so a room without it throws in `Awake` and the client shows no menu.
  - `PhotonNetwork.time` = `(uint)ServerTimeInMilliSeconds / 1000.0`.
