# Remote players, the room and the kill feed

Client: `remote.rs` (another player's copy), `room.rs` (`RoomMultiplayerMenu` teams/spawns, `WhoKilledWho`, the `--dummy` test client), wired in `main.rs`.

Every player exists once per client. The owner runs the `PhotonView.isMine` branch of the scripts; every other client runs the remote branch on a **copy**, driven only by the serialized stream and the RPCs. This page is that remote side. Its inputs come from Photon with `--online` (`online.rs`, `docs/protocol.md` "Playing online"), or from the in-process `--dummy` client.

Evidence: decompiled with `tools/re/` (function numbers below are wasm2c indices), field data in `assets/data/prefabs/resources/{german,mercenarys}.json`. **Observed** = read from code or data; **assumed** = marked as such.

## Teams, spawn points, prefabs (`RoomMultiplayerMenu`)

**`Awake` (`f29720`)** overrides the scene's values in every scene (observed):

| | name | colour | spawn list |
|---|---|---|---|
| `team_1` | `German-ZM` | `(1, 0.92156863, 0.015686275, 1)` (yellow) | the scene's "Team A SpawnPoints" |
| `team_2` | `Mercenary` | red | the scene's "Team B SpawnPoints" |

- `gameMode` comes from the room property `GameMode` (`DM` / `TDM`).

**`SpawnPlayer(team)`:**
- Team 1 spawns at a random team-1 point in TDM, or at any point in DM, as `playerPrefab`. That is `mercenarys`, despite the team name.
- Any other team spawns at a random team-2 point as `playerPrefabTwo` (`german`).
- The client's `--team 1|2` selects the team. The default is 2, which plays `german` as before.

## The copy: `PlayerNetworkController` remote branch

**Awake (`f37893`, not mine):**
- Every child of the `WeaponManager` is deactivated, so no first-person weapon runs.
- `remoteObjectsToDeactivate` is applied: in `german`, `LookObject`, `SpawnObject` and the root-level `Base_mesh` (node 483). The owner's `localObjectsToDeactivate` hides the other `Base_mesh`, `Root_jnt` and `Terrorist_02_mesh`.
- `lookTarget` is activated.
- `PlayerDamage.Awake` adds a `HitBox` to each `hitBoxes` entry's collider (12 per prefab, `damage = 1` each). The owner destroys them instead.
- The body under `mercenary_minecraft`, with its `Animation`, `CharacterAnimation` and `HeadLookController`, stays visible.

**OnPhotonSerializeView (`f37894`).** The stream order (owner writes, copy reads) is:
1. `transform.position`
2. `transform.rotation`
3. `gameObject.name`
4. `lookTarget.position`
5. `lookTarget.rotation`
6. `TeamName`
7. `animationSyncHelper` (movement clip)
8. `animationForHands`
9. `activeWeapon`
10. `animationType`
11. `IsSpawnProtected`

PUN sends it every 100 ms (`sendIntervalOnSerialize`).

**Update (`f37896`, remote branch), every frame:**
1. Position and rotation lerp to the received values at `dt·8`. So does `lookTarget`'s world position; its rotation is set directly.
2. In TDM, a teammate gets `disableDamage = true` and `DrawPlayerName` enabled. In DM the name is off.
3. **Movement clip.** When it changes:
   - set the speed by `animationType` (`Walking`, `Running`, `Crouch`, `Prone` → `animationSpeed[0..3]`);
   - set layer 1 and `WrapMode.Loop`.

   Then `CrossFade(clip)` runs every frame. Setting `.speed` on a missing state throws a NullReference, which ends that frame's Update (observed: the speed set is unguarded).
4. **Hands clip.** When it changes and is not `Null`:
   - set layer 4 and `Loop`;
   - add a mixing transform for both shoulders' subtrees, once per clip.

   Then it is `Play`ed every frame.
5. **Third-person weapon.** When `activeWeapon` changes, only the child of `ThirdPersonWeapons` with that name is active, and it becomes `currentActiveWeapon`.

**`HeadLookController`** stays on the copy. Its `segments` array is empty in both prefabs, and `Slerp(a, a)` with `FromToRotation(a, a)` leaves every bone as it is. The `LateUpdate` is therefore an identity, so it is not ported (observed).

**`CharacterAnimation` on the owner.** `ThirdPersonWeaponControl` picks:
- `Null` when prone;
- otherwise the idle of the list that contains the selected weapon:
  - `twoHandedWeapons`: gun idle;
  - `pistols`: pistol idle;
  - `knivesNades`: knife idle.

It writes the clip to `animationForHands` and the weapon name to `activeWeapon`; those are the strings serialized above. Legacy layering and mixing are in `docs/gameplay/animation.md`.

## Shots of another player: `WeaponSync`

**Flow:**
1. The owner's `WeaponSync_Catcher.Fire` sends `SyncWeaponsRPC(functionName, value)` to `PhotonTargets.Others`.
2. The copy then calls `currentActiveWeapon.SendMessage(functionName, value)`.

The client's `WeaponEvent::SyncFire` is that call. It is ignored locally because no other client exists yet.

**What each weapon sends:**

| weapon | sends | value |
|---|---|---|
| machine gun | `syncMachineGun` | the current error angle |
| shotgun | `syncShotGun` | `fractions` |
| grenade launcher | `syncGrenadeLauncher` | `initialSpeed` |
| knife | `syncKnife` | — |

**Unity `SendMessage` (native `f5630`):**
- It converts a boxed int to float, so the int values arrive as float.
- When the parameter count does not match, it logs an error and calls nothing. `syncKnife()` takes no parameter but is always sent one value, so **a remote knife never plays** (observed).

**On the copy:**
- **`Start` (`f61818`)** runs on the first frame the weapon is active. It turns the muzzle-flash renderer off and copies the shot settings from the matching first-person weapon by `GunType`.
- **`syncMachineGun(err)` / `syncShotGun(n)`:**
  - `StopAllCoroutines`;
  - 1 or `n` bullets from the fire point, spread by `err` or 3. The bullet is `Instantiate`d with `doDamage = false`; impacts only;
  - the flash on for `WaitForSeconds(0.04)`;
  - the fire sound.
- **`syncGrenadeLauncher(speed)`:**
  - the projectile with `isRemote = true`, pushed by `AddForce(forward·speed·50)`. With mass 1 that is Δv = `speed` over one 0.02 s step;
  - `IgnoreCollision` with the shooter's colliders;
  - the sound.

  A remote rocket explodes visually, but its `ExplosionDamage` does nothing (`isRemote`).

## Damage to another player: `HitBox` → `DoDamage`

**Hit detection.** Bullets (raycast) and explosions (`OverlapSphere`) of **this client's** player hit the copies' `HitBox` colliders:
- `BoxCollider` / `SphereCollider` on the bones, moving with the animation;
- rockets also hit the root `CharacterController` capsule.

**`SendMessageUpwards("ApplyDamage", dmg)`** runs on the hit collider's GameObject and **each ancestor**. Every `HitBox` on that chain runs `ApplyDamage`:
- One bullet on `LowerArm_Right_jnt` reaches the hit boxes of the lower arm, upper arm, spine, body and hips: 5 × (40 + 1) = 205 damage with the M242. One such hit kills a 200-HP player.
- A body hit reaches fewer hit boxes.
- An explosion does this for every overlapped hit box, with `ExplosionDamage`'s falloff from the closest point of the collider's bounds.

**`HitBox.ApplyDamage(d)` → `TotalDamage(d + maxDamage)`.** Unless `disableDamage` (TDM teammate):
1. the hit marker (`fadeValue = 2`);
2. the `DoDamage(total, damager)` RPC to `PhotonTargets.All`. The sender runs it at once on its copy; the owner runs it on its player (`docs/gameplay/damage.md`).

**`DoDamage` on a copy:**
1. It returns while spawn-protected (the synced flag) or already killed.
2. `hp -= damage`.
3. Below 0:
   - the root's children are deactivated;
   - the death prefab is instantiated with `clearCamera`, so no camera;
   - if the damager is this client's player, `AddKillNotification(name)` runs.

**`AddKillNotification`:**
- the feed message `killer killed killed`;
- the killer's `Kills` + 1;
- the team score + 1;
- a persistent `PlayerPrefs` "Kills" counter (kept in `prefs.json`, `docs/gameplay/settings.md`).

**`PlayerDamage.OnGUI` on a copy** draws only the hit marker: `textures/hitmarker_red.png`, 26 × 26 at the screen centre, alpha `fadeValue`, which fades at `dt·2`.

## DrawPlayerName

- It is enabled for TDM teammates only.
- `OnGUI` at depth 2:
  - `GUI.color = (0.1, 0.9, 0.5, 1)`;
  - `p = WorldToScreenPoint(positionDisplay)`, `offset = min(p.z·3, 50)`;
  - if `p.z > 0`, a label with the skin's font size, white, centred, at `(p.x-100, H-p.y-5-offset, 200, 30)`.

## Kill feed: `WhoKilledWho` (`f61870`..`f61887`)

- **Skin:** `WhoKilledWho.guiSkin` on `__Room` in every scene: `resources.assets:40925` ("1 win  rom_who kiled who 1") in the CrazyGames build (Build-7: 40955). The client reads it from the scene (`Scene::kill_feed_skin`).
- **`networkAddMessage(killer, killed, middle, team)`** is an RPC to all. It appends an entry with a 5 s timer and keeps at most 5 entries.
- **Colours:**
  - TDM, no team: both names white;
  - TDM, a team: the killer's team colour and the other team's colour;
  - DM: both names in team 1's yellow.
- **Messages:** `X connected` (`PlayerJoinedRoom`), `X joined <team>` (the team menu), `X killed Y`, `X fell down` (`PlayerFellDown`, own fall death).
- **`Update`:** timers run down. Removing an expired entry skips the next entry for that frame, as in the original loop.
- **`OnGUI`:**
  - `BeginArea(W - W/3.5, 85, W/3.5, 400)`;
  - one right-aligned box per entry: killer, `Space(5)`, middle, `Space(5)`, killed, `Space(10)`.

  The GUILayout spacing is reproduced from Unity's rules, not measured (`docs/unknowns.md`).

## The `--dummy` test client

`DummyOwner` (`room.rs`) is a second client in-process. It runs that client's **own** player:
- movement without input;
- `CharacterAnimation`;
- `PlayerDamage`;
- spawn protection;
- death, `RespawnAfter`, `SpawnPlayer`.

It sends this client what Photon would carry:
- `Instantiate`;
- the serialized state every 100 ms;
- `SyncWeaponsRPC` with `--dummy-fire`;
- `DoDamage` (its fall damage);
- `networkAddMessage`;
- `Destroy`.

This client's `DoDamage` RPCs reach the dummy's owner side, so it dies, shows `RespawnAfter`, and respawns at a team spawn point.

The dummy does not simulate hit detection against this client's player. Only this client's shots deal damage.

**Flags:**
- `--dummy`, `--dummy-team 1|2`, `--dummy-spawn N`;
- `--dummy-weapon SLOT` (index into `allWeapons`, default 2 = M242);
- `--dummy-fire`.

`--camera x,y,z,yaw,pitch --walk` puts the local player at a camera position, for screenshots of shooting the dummy. Fly mode also draws the local player's own copy (the "mirror"), fed by its serialized state.

**Assumption:** the first serialized state arrives with the instantiation, so the copy does not lerp from the constructor's `(0, -100, 0)` (`docs/unknowns.md`).
