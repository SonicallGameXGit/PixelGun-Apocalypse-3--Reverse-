# Parity checklist: scripts against the CrazyGames build

Audit of 2026-11-07. It lists every `MonoBehaviour` script that `assets/data/` attaches to a scene or prefab (63 scripts) and says whether `crates/pga3-client/src` ports it. Nothing was implemented for this audit.

## How this was built

- **Script inventory (observed):** every `MonoBehaviour` component in `assets/data/scenes/*.json` and `assets/data/prefabs/**`, grouped by `script`. Counts in the tables are scenes (of 20) and prefabs.
- **Method names (observed):** `research/crazygames-il2cpp-dump/dump.cs` (CrazyGames build, Unity 5.3.5). They are the original method names.
- **Port status (observed, by grep):** each script and method name searched in `crates/pga3-client/src` and `docs/`. "done" means the client has code and a spec for it. A grep hit is not proof that every method is exact: `docs/unknowns.md` #34 lists the ported scripts that were never compared with their CrazyGames bodies. They are marked **CG?** below.
- **Fresh decompilation (observed):** `EnableHelper`, `WeaponItem` (and `WeaponPickUp`'s call list) were read with `tools/re/asmjs_func.py` for this audit.
- **Marks:** `done` = ported; `partial` = some methods or branches ported; `missing` = not ported. **CG verified** = compared with the CrazyGames body (`yes`, or `no` = ported from Build-7 or the wasm and not yet compared).
- **Scope of play:** online play uses the `german` and `mercenarys` player prefabs and the room prefab `__Room` (`docs/gameplay/remote-players.md`). Scripts that exist only on other prefabs are marked *unreachable*.

Summary: the in-match gameplay scripts are ported. What is missing is the UI around the match (the full room menu, scoreboard, round flow, chat), the `Credits` scene, and a handful of scripts on prefabs that online play never loads.

## 1. Missing or partial, ranked by player visibility

Rank 1 is the most visible.

| Rank | Script (method names) | Status | What the player loses | Evidence / notes |
|------|-----------------------|--------|-----------------------|------------------|
| 1 | `RoomMultiplayerMenu`: `Start`, `Update`, `FixedUpdate`, `OnGUI`, `SortPlayers`, `WaitBeforeCheck`, `RoundEnded`, `Restart`, `SwapTeams`, `SpawnPlayerTwo`, `FadeScreen`, `MainMenu`, `Resolutions`, `QualityWindow`, `LeaveRoom`, `LoadMap`, `OnDisconnectedFromPhoton`, `OnLeftRoom` | **partial** (2026-11-11): `Awake`, `Start`, `Update`, `FixedUpdate`, `OnGUI`, `MainMenu` (team menu, player list with kills/deaths/ping, controls, sensitivity, `Resolutions`, `QualityWindow`, Resume, Leave Room), `SortPlayers`, `WaitBeforeCheck` and the round timer, `SpawnPlayer`, `SwapTeams` are done (`room_menu.rs`, spec `docs/gameplay/settings.md`); not done: `RoundEnded`, `Restart`, `SpawnPlayerTwo`, the leave `FadeScreen` | The in-room UI: the team-selection menu (the client picks the team with `--team`), the scoreboard (`SortPlayers`), the round timer and round end, `Restart`/`SwapTeams` between rounds, the pause/options menu (resolution, quality, leave room), the screen fade. Biggest gap in a match. | `docs/unknowns.md` #4 and #32. `research/notes/crazygames/roommultiplayermenu.js` (200 KB) is already decompiled, so reading it is cheap. `Resolutions` is new in this build (`docs/crazygames-build.md`). |
| 2 | `ConnectMenu`: `Start`, `Update`, `OnGUI`, `ShowConnectMenu`, `FadeScreen`, `LoadMap`, `CheckPlayerNameAndRoom`, `OnJoinedRoom`, `OnJoinedLobby`, `OnLeftRoom`, `OnPhotonJoinRoomFailed`, `OnPhotonCreateRoomFailed`, `OnPhotonPlayerConnected`, `OnConnectedToPhoton`, `OnDisconnectedFromPhoton` | **done** (2026-11-08): `menu.rs` (player name, search, password, room list with locks, level row, room creation with maps, weapons, mode, time, players, `LoadMap` fade); spec `docs/gameplay/menu.md` | The lobby: player name field, room list, map and mode pickers, room creation form, password, error messages, levels and ranks. Today everything is a CLI flag (`--online`, `--room`, `--create`, `--list-rooms`). | Decompiled in `research/notes/crazygames/connectmenu.js`; protocol in `docs/protocol.md`. Scenes: `Main`, `Credits` + the `__lobby1` prefab. |
| 3 | `ServerChose`: `Update`, `us`, `eu`, `au`, `asia`, `jp`, `chooseserverondisconect`, `OnDisconnectedFromPhoton` | **done** (2026-11-08): the three region buttons (`eu`, `us`, `asia`), disconnect returns to the dialog; `jp`/`au` have no buttons in this build | The region buttons and reconnect to another region after a disconnect. | Decompiled in `research/notes/crazygames/serverchose.js` (no `ru` in this build). Part of the lobby. |
| 4 | `MultiplayerChat`: `Awake`, `Update`, `OnGUI`, `SendChat`, `SendChatMessage`, `AddMessage`, `OnLeftRoom`, `OnJoinedRoom`, `OnCreatedRoom`, `ClearChat` | **missing** (on `__Room`, 18 scenes) | In-match text chat, both reading and sending. | `docs/unknowns.md` #32 ("chat"). Needs the chat RPC name and format from a capture or the decompilation. |
| 5 | `Main` and `Credits` scene UI (uGUI): `PanelManager` (`OnEnable`, `OpenPanel`, `FindFirstEnabledSelectable`, `CloseCurrent`, `DisablePanelDeleyed`, `SetSelected`, `Start`), `PressHandler` (`OnPointerDown`), `Link` (`OpenPaco`, `OpenCrazy`, `OpenIoanMita2016`, `OpenYoutube`, `OpenFacebook`, `openWindow`), `X Redirect Link`, `ApplicationManager` (`Quit`), `TiltWindow` (`Start`, `Update`), `UnityAdsHelper` (`Awake`), `UnityAdsOnLoad`, and the stock `UnityEngine.UI.*` / `EventSystems.*` components | **partial** (2026-11-08): `ui.rs` draws the `Main` server window (Image, Text, Button with the highlight animation, `TiltWindow`, `Link` URLs); not done: `PanelManager` panels, the 43 `Animator`s beyond the button clips, the menu particle system and music, the `Credits` scene | The main menu screens, the Credits scene, the menu panel animation (43 `Animator`s, `docs/assets.md`). `Link`, ads and `ApplicationManager` are web-platform glue. | Only `Main` and `Credits` hold these. The UI clips are Mecanim (`docs/assets.md`), so this is the only place the client would need an `Animator`. |
| 6 | `EnableHelper`: `Start`, `FixedUpdate` | **done** (2026-11-11, `docs/gameplay/settings.md`) | Pausing the controls while a room menu is open. | **Observed:** `Start` stores `GameObject.FindWithTag("EnableHelper")` (every map and `__Room` has one such object). `FixedUpdate`: while that object is active, `FPScontroller.canControl`, `WeaponManager` and its child, and both `FPSMouseLook`s are forced **on**; while it is inactive they are forced **off**. So the menus of rank 1 and 4 switch it. It is only needed once those menus exist. |
| 7 | `PlayerDamage`: `SwapTeams`, `DoSwapTeams` | **partial** (the rest of `PlayerDamage` is done) | The team swap at round end. The RPC name is already known and ignored in `net/pun.rs`. | `docs/unknowns.md` #32 (`DoSwapTeams`). Depends on rank 1's round end. |
| 8 | `PlayerNetworkController`: `SetOnVehicle`, `TeleportPosition` | **partial** (the rest is done) | Probably nothing: the RPC names are known (`net/pun.rs`) and ignored. Whether any code in this build sends them is **unknown**. | Search the CrazyGames code for callers before deciding. |
| 9 | `WeaponPickUp`: `Start`, `Update`, `AssignWeapon`, `OnGUI`; `WeaponItem`: `OnTriggerEnter`, `OnTriggerExit` | **missing** (`WeaponPickUp` is on both live player prefabs, enabled; `WeaponItem` only on the 9 `weaponsforpick/*` prefabs) | Picking up a dropped weapon, a pickup hint, throwing a weapon (`throwForce` 350), ammo top-up (`pickAmmoMultiply`, `reserveAmmoLimit`). **Observed:** `WeaponItem.OnTriggerEnter` calls `AssignWeapon` on a root tagged `Player`; `WeaponPickUp` calls `WeaponManager.SwitchWeapons`, `Instantiate`, `Rigidbody.AddForce`, `GUILayout` and `Object.Destroy`. **Not found:** any scene or player prefab that instantiates a `WeaponItem` prefab. If nothing spawns one, the player never sees this. | Reachability is **unknown**; settle it by finding who instantiates `weaponsforpick/*` (a `PhotonNetwork.Instantiate` string in `dump`/`stringliteral.json`). |
| 10 | `TouchController` (`InitController`, `PollController`, `UpdateController`, `DrawControllerGUI`, ... Easy Touch) | **missing** (18 scenes) | Mobile touch controls only. Not shown on desktop. | Touch textures changed in this build (`docs/crazygames-build.md`). Low priority unless mobile is a goal. |

## 2. Not visible in a match (missing, kept for completeness)

These are attached to the data but cannot be seen by an online player, or only exist on prefabs that online play never loads.

| Script (method names) | Status | Why it is not visible |
|-----------------------|--------|----------------------|
| `Flashlight` (`Start`, `Update`, `flashLightOnOff`) | missing | Only on `old/*` and the single-player character prefabs, *unreachable* online. |
| `AmmoDeactivator` (`Start`, `FixedUpdate`) | missing | Same prefabs as `Flashlight`. |
| `SniperAnimation` (`Awake`, `Fire`, `Reloading`, `takeIn`, `takeOut`) | missing | Only on `old/*` and the single-player prefabs. `WeaponAnimation`, the same code for the live prefabs, is done. |
| `MainMenu` (`Start`, `Update`, `OnGUI`, `MainMenuDisplay`, `Resolutions`, `QualityWindow`, `Pause`) | missing | Single-player `_mainmenu` prefab. |
| `SlowMotionEffect` (`Update`, `OnGUI`) | missing | On `_mainmenu`. |
| `FramesPerSecond` (`Start`, `Update`, `OnGUI`) | missing | Debug counter on `_fps`. |
| `Detonator` (`Awake`, `FillDefaultMaterials`, `Start`, `Update`, `UpdateComponents`, `Explode`, `Reset`, `Default*Material`), `DetonatorFireball`, `DetonatorShockwave` | missing | Only on the `Detonator-Wide` and `Detonator-Heatwave(Pro)` prefabs. The Launcher's projectile in `german` is `resources.assets:38601` and `docs/gameplay/weapons.md` lists its explosion as `WFXMR_Explosion 1`, so these prefabs are not reached by it. They are the explosion of the `RocketMissile__367` projectile (and `Detonator-Heatwave(Pro)` of `Grenade__337`). Not checked: whether any live weapon or mode uses those two projectiles. |
| `WFX_ParticleMeshBillboard` (`Awake`, `OnWillRenderObject`) | missing | On 3 effect prefabs (`ExplosiveSmokeGround Small`, `Metal`, `WFXM_Nuke`). `Metal` is a bullet-impact effect and may be visible: the script orients a mesh particle towards the camera. Check before ranking it as not visible. |

## 3. Done

| Script | Methods (original names) | CG verified | Where |
|--------|--------------------------|-------------|-------|
| `FPScontroller` | `Awake`, `setupBools`, `UpdateFunction`, `FixedUpdate`, `Update`, `Prone`, `Crouch`, `OnRunning`, `OffRunning`, `OnLadder`, `OffLadder`, `ApplyInputVelocityChange`, `ApplyGravityAndJumping`, `OnControllerColliderHit`, `GetDesiredHorizontalVelocity`, `AdjustGroundVelocityToNormal`, `IsGroundedTest`, `GetMaxAcceleration`, `CalculateJumpVerticalSpeed`, `IsJumping`, `IsSliding`, `IsTouchingCeiling`, `IsGrounded`, `TooSteep`, `GetDirection`, `SetControllable`, `MaxSpeedInDirection`, `SetVelocity` (moving platforms: `SubtractNewPlatformVelocity`, `MoveWithPlatform` are not ported; disabled in the prefab) | no | `player.rs`, `physics.rs`; `docs/gameplay/movement.md` |
| `FPSinput` | `Awake`, `LateUpdate` | no | `player.rs` |
| `FPSMouseLook` | `Awake`, `Update`, `Recoil` | yes (`docs/gameplay/settings.md`) | `player.rs` |
| `CameraBob` | `Awake`, `FixedUpdate` | no | `player.rs` |
| `WalkSway` | `Awake`, `FixedUpdate` | no (Build-7; unknowns #34 omits it) | `player.rs` |
| `LadderPlayer`, `Ladder` | `Start`, `LatchLadder`, `UnlatchLadder`, `FixedUpdate`, `RaycastCheck`; `Start`, `ClimbDirection`, `OnTriggerEnter` | no | `player.rs`, `scene.rs` |
| `FPSSoundController` | `Awake`, `FixedUpdate`, `PlayStepSounds` | no | `footsteps.rs`; `docs/gameplay/audio.md` |
| `WeaponManager` | `Awake`, `Update`, `TakeFirstWeapon`, `SwitchWeapons`, `OnGUI` | yes | `weapons.rs`, `hud.rs` |
| `WeaponScript` | `Awake`, `Start`, `Update`, `InputUpdate`, all `machineGun*`, `grenadeLauncher*`, `shotGun*`, `knife*`, `Aiming`, `cameraRecoilDo`, `RotationRealism`, `SmoothMove`, `selectWeapon`, `deselectWeapon`, `GetCenterOfScreen` | yes | `weapons.rs` |
| `WeaponCrosshair`, `SniperScope` | `Awake`, `Update`, `OnGUI`; `Awake`, `OnGUI` | yes | `weapons.rs` |
| `Bullet`, `Projectile`, `ExplosionDamage`, `CFX_AutoDestructShuriken` | `Start`, `Update`; `Start`, `FixedUpdate`, `OnCollisionEnter`, `Kill`; `Start`; `OnEnable`, `CheckIfAlive` | yes | `projectiles.rs`, `effects.rs` |
| `WaitForDestroy` | `Awake` | no | `effects.rs` |
| `AmmoDisplay` | `Awake`, `Update`, `OnGUI` | no | `hud.rs` |
| `WeaponAnimation` | `Awake`, `Fire`, `Reloading`, `takeIn`, `takeOut` | no | `character.rs`, `weapons.rs`; `docs/gameplay/animation.md` |
| `CharacterAnimation` | `Start`, `LateUpdate`, `ThirdPersonWeaponControl`, `configureAnimations` | no | `character.rs` |
| `HeadLookController` | `Start`, `LateUpdate`, `AngleAroundAxis` | no | Identity on both prefabs (empty `segments`), not run (`docs/gameplay/remote-players.md`) |
| `PlayerDamage` | `Awake`, `Update`, `ApplyFallDamage`, `TotalDamage`, `DoDamage`, `DestroyPlayer`, `OnGUI`, `doCameraShake` | no | `damage.rs`; `docs/gameplay/damage.md` |
| `RagdollController` | `Start`, `OnGUI`, `RespawnAfter`, `_respawnAfter`, `clearCamera`, `sleepRigidbody` | no | `damage.rs` |
| `HitBox` | `ApplyDamage` | no | `remote.rs`, `damage.rs` |
| `PlayerNetworkController` | `Awake`, `OnPhotonSerializeView`, `Start`, `Update`, `ReDeactivatePlayerObjects`, `syncMachineGun`, `syncShotGun`, `syncGrenadeLauncher`, `syncKnife`, `SyncWeaponsRPC`, `OnGUI` | no | `remote.rs`, `net/pun.rs`, `hud.rs` |
| `WeaponSync`, `WeaponSync_Catcher` | `Start`, `syncMachineGun`, `machineGunShot`, `syncShotGun`, `shotGunShot`, `syncGrenadeLauncher`, `grenadeLauncherShot`, `syncKnife`, `knifeOneHit`; `Awake`, `Fire` | no | `remote.rs`, `weapons.rs`; a remote knife never plays (observed) |
| `DrawPlayerName` | `Awake`, `Update`, `OnGUI` | no | `remote.rs` |
| `WhoKilledWho` | `Start`, `Update`, `OnGUI`, `AddKillNotification`, `PlayerJoinedRoom`, `PlayerJoinedTeam`, `PlayerFellDown`, `networkAddMessage` (`PlayerLeftRoom`, `OnPhotonPlayerDisconnected`, `OnLeftRoom`, `OnJoinedRoom`, `OnCreatedRoom` are not ported: nobody leaves in this client's view) | no | `room.rs`; the persistent `PlayerPrefs` "Kills" counter is kept (`prefs.rs`) |
| `PhotonView` | `Awake`, `RPC`, `SerializeView`, `DeserializeView`, ownership calls | n/a (PUN library) | `net/pun.rs` |

Two scripts are in the dump but **not attached** to anything in `assets/data`, so they need no port: `SpawnProtection` (`Start`, `Update`, `GetHit`; the protection logic the client uses lives in `PlayerNetworkController`) and `InRoomRoundTimer`/`ServerTime` (the round timer is read from the room by `RoomMultiplayerMenu`, rank 1).

## 4. Order of work I suggest

1. Rank 1 (`RoomMultiplayerMenu`): the menu, ranks 6 and 7 are done; the round end (`RoundEnded`, `Restart`) remains (`docs/unknowns.md` #32).
2. Rank 4 (`MultiplayerChat`), once its RPC is known from a capture.
3. Rank 2 and 3 (lobby): done, see `docs/gameplay/menu.md`.
4. Settle the two "unknown reachability" items (rank 8, rank 9, `WFX_ParticleMeshBillboard` on `Metal`) with a grep before spending any time on them.
5. In parallel, the "CG verified: no" rows against their CrazyGames bodies (`docs/unknowns.md` #34).

## 5. Doc drift found during the audit

- `docs/gameplay/movement.md`, "Not ported yet", still says `PlayerDamage`, footstep sounds and network sync are not ported. They are (`damage.rs`, `footsteps.rs`, `remote.rs`).
- `docs/gameplay/weapons.md`, "Not implemented", still says `SyncWeaponsRPC` is not sent over the network; `net/pun.rs` has `sync_fire_args`.
- `docs/code-structure.md` describes Build-7's layout (bundle version 1.7, scene paths `- - A WEB paco/`); the CrazyGames build is 1.3 with `Assets/A WEB crazy/`.
- `docs/unknowns.md` #34 lists the uncompared ported scripts but omits `WalkSway`, `FPSSoundController` and `WeaponSync_Catcher`.

These are listed, not fixed, since this task was an audit only.
