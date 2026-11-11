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

## Assets

Extracted with the Build-7 pipeline (`tools/extract/README.md`, "CrazyGames build") into `assets/`, which now holds this build. The Unity 5.3 differences the tools handle are in `docs/assets.md`, "Unity 5.3 serialization". `tools/extract/compare_assets.py` compares two published trees by content (pixels for images, bytes otherwise); the report is `research/notes/crazygames/asset-diff.json` (**observed**, 2026-11-07):

| Type | Build-7 | CrazyGames | Same | Notes |
|------|---------|------------|------|-------|
| Mesh | 173 | 172 | 172 | Build-7 also has `Icosphere` |
| Font | 12 | 7 | 7 | Build-7 also has the "fortness", Impact and PacoGames fonts |
| AudioClip | 48 | 48 | 0 | the same sounds encoded separately (correlation 0.87-1.0 at zero lag); some sample rates differ |
| Texture2D | 277 | 397 | 111 | 5 of them renamed; see below |
| Sprite | 41 | 11 | 3 | Build-7's extra sprites are the PacoGames SDK / social UI |

- **Map textures differ completely.** This build textures each map with its own materials (`Bridge_material_N`, ...); Build-7 replaced them with Minecraft-style block textures. Map geometry (meshes, colliders, spawn points) is the same.
- Other texture changes: touch-control textures (Easy Touch), `gun_4_rocke tinta`, `Heatwave`, `Smoke`, `hitmarker_red`, the rocket texture; skyboxes are 128² (64² in Build-7). `detonator/textures/heatdistort` is a new Resources texture; the PacoGames SDK prefabs are gone.
- The kill-feed `GUISkin` (`WhoKilledWho.guiSkin`) is a different asset; the client reads it from the scene instead of a fixed ref.
- **Weapons:** the `WeaponScript` data is identical except the crosshair (`WeaponCrosshair`: texture `Crosshair 16x15`, width 4) and the fields this build does not have (`tagsToShoot`, `autoFire`, `inSight`). Bullet, knife and rocket prefabs have the same values; impacts and the explosion are Shuriken systems in both builds.
- **Particles:** the same effects, serialized with the 5.3 `ParticleSystem` layout; upgraded to the 2018.3 layout on load, every read field matches Build-7 except real data edits (collision `radiusScale` 1.0 vs Build-7's 0.01 on `Blood`, `Metal`, `WFXMR_Explosion*`; `Metal` max particles 20 vs 100). The shotgun's "Smoke Trail" is a **legacy** particle emitter here (`docs/gameplay/particles.md`).

## Scripts against Build-7

Compared method by method, CrazyGames asm.js (`tools/re/asmjs_func.py`, output in `research/notes/crazygames/`) against the Build-7 decompilation (**observed**):

- **`WeaponManager`:** no auto fire (`autoFire`, `textDelay`, the O key toggle, `Awake` setting the weapons' `autoFire`) and no "Auto fire on/off" label in `OnGUI` (`hxg` draws only the two room messages). Number keys (`Input.GetKeyDown("1")`...), scrolling, switching and `TakeFirstWeapon` are the same.
- **`WeaponScript`:** no `RaycastForZombies` / `inSight`, so `fire = held && canFire && !singleFire`. The shotgun smoke is a legacy `ParticleEmitter`: `shotGunAwake` sets `emit = false`; `shotGunSmokeEffect` sets `emit = true`, waits 0.3 s, sets `emit = false` (Build-7: `Play()` / `Stop()`). Everything else is the same: `Start`, `Awake`, the fire loops, one-shots, reloads, camera recoil, `RotationRealism`, muzzle flash, knife and launcher.
- **`WeaponCrosshair`:** no red `GUI.color` while `inSight`.
- **`Projectile.Kill`:** `GetComponentInChildren<ParticleEmitter>().emit = false` instead of `ParticleSystem.Stop()`; the rocket has only a `TrailRenderer`, so neither does anything.
- **`ExplosionDamage.Start`** (`Fbg`): damage (`SendMessageUpwards("ApplyDamage")` unless `isRemote`) and `AddExplosionForce(power, position, radius, 3)` as in Build-7; then, if the object has a legacy `ParticleEmitter`, `emit = true` for 0.5 s, and `Destroy(gameObject, explosionTimeout)`. It never stops or plays the Shuriken systems, so **the explosion is visible** in this build (Build-7 stopped them before their first update and `CheckIfAlive` destroyed the object at 0.5 s).
- **`CFX_AutoDestructShuriken`:** same (`CheckIfAlive` every 0.5 s on `GetComponent<ParticleSystem>().IsAlive(true)`).
- **`Bullet`, `SniperScope`:** identical.

The client follows this build (`docs/gameplay/weapons.md`, `docs/gameplay/particles.md`, `docs/gameplay/hud.md`).

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
- Method bodies: the weapon scripts are compared ("Scripts against Build-7" below); the others not yet.

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

Menu scenes and scripts (`Main`, `ConnectMenu`, `ServerChose`, the in-room menu) are specified in `docs/gameplay/menu.md`.
