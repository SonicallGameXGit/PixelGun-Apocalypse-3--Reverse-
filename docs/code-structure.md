# Original code structure (IL2CPP dump)

Source: Il2CppDumper output in `research/il2cpp-dump/` (`dump.cs`, `script.json`, `il2cpp.h`, `stringliteral.json`, `DummyDll/`). Names, fields, field offsets, method signatures and wasm function indices are recovered; method **bodies** are not. Those require decompiling `pga3-v18.wasm` (use `script.json` to name functions).

## Assemblies of interest

| Assembly | TypeDefIndex range | Content |
|----------|--------------------|---------|
| `Assembly-CSharp` | 4602+ (520 types) | game code |
| `Assembly-CSharp-firstpass` | 4415–4601 | PUN Classic 1.91 and other plugins |
| `Photon3Unity3D` | 4343–4414 | Photon client library |
| `GarterProtection`, `Garter*`/`HGarter*` classes | – | GameArter (PacoGames platform) SDK: auth, progress, ads, anti-piracy |
| `Newtonsoft.Json` | – | JSON |

## Game classes (Assembly-CSharp)

The core gameplay matches the structure of the **NSdesignGames "FPS Kit 2.0"** Unity asset (**inferred** from class/field names; not yet verified against the kit's source). Main groups:

- **Movement/input**: `FPScontroller` (+ `FPScontrollerMovement`, `Jumping`, `MovingPlatform`, `Sliding`, `Pushing`, `FallDamage`), `FPSinput`, `FPSMouseLook`, `Ladder`, `LadderPlayer`, `CameraBob`, `WalkSway`.
- **Weapons**: `WeaponManager`, `WeaponScript` (`gunType`, `shotGun`, `GrenadeLauncher`, `MachineGun`, `Knife`, `AimVariables`, `cameraRecoil`), `Bullet`, `Projectile`, `ExplosionDamage`, `WeaponPickUp`, `WeaponItem`, `SniperScope`, `WeaponCrosshair`, `AmmoDisplay`.
- **Damage/health**: `PlayerDamage` (+ `HitBoxes`), `HitBox`, `SpawnProtection`, `RagdollController`, `WhoKilledWho` (kill feed).
- **Networking/menus**: `ServerChose` (region), `ConnectMenu` (lobby, room creation, maps, levels/ranks), `RoomMultiplayerMenu` (in-room, teams), `PlayerNetworkController`, `WeaponSync`, `MultiplayerChat`, `DrawPlayerName`.
- **Characters**: `CharacterAnimation`, `HeadLookController`.
- **Effects**: WFX/CFX war-effects packs.

## Scenes (BuildSettings)

| Index | Scene | Index | Scene |
|-------|-------|-------|-------|
| 0 | Main (menu/lobby) | 10 | Village |
| 1 | Bridge | 11 | ZM_Bridge |
| 2 | Castle | 12 | ZM_Castle |
| 3 | Crash | 13 | ZM_Crash |
| 4 | Credits | 14 | ZM_Fight |
| 5 | Fight | 15 | ZM_Pete |
| 6 | Pete | 16 | ZM_Scaz |
| 7 | Scaz | 17 | ZM_Snow |
| 8 | Snow | 18 | ZM_Tunnels |
| 9 | Tunnels | 19 | ZM_Village |

`ZM_*` = zombie-mode variants. Scene paths are under `Assets/- - A WEB paco/`. `PlayerSettings.bundleVersion = 1.7`.
