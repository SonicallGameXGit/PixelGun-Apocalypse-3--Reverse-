# Damage, death and respawn

Client: `damage.rs` (`PlayerDamage`, spawn protection, `RagdollController`), driven from `main.rs`.

## Where damage comes from

- Bullets (`Bullet`) and explosions (`ExplosionDamage`) call `SendMessage("ApplyDamage", ...)` on what they hit. Only `HitBox.ApplyDamage` receives it, and it forwards the damage as the `DoDamage` RPC (**observed**).
- The owner's `HitBox`es are destroyed in `PlayerNetworkController.Awake`, so only **remote** players can be hit. This client's bullets and explosions damage the other players' copies (`docs/gameplay/remote-players.md`). The owner then receives the `DoDamage` RPC, which runs `DoDamage` below. Until networking exists, only the `--dummy` client sends and receives these RPCs.
- **Fall damage** is the only local source:
  - `FPScontroller` calls `PlayerDamage.ApplyFallDamage(dmg)` (`f37878`, values in `docs/gameplay/movement.md`);
  - that sends `DoDamage` to `PhotonTargets.All`, which PUN runs locally at once.

## PlayerDamage (`f37876`..`f37890`)

**Basics:**
- `hp = 200`.
- `Awake` stores the camera's default local rotation.

**`DoDamage(damage, ...)`** (`f37881`):
1. Returns if the player `IsSpawnProtected` (`PlayerNetworkController` +0x75) or already `weKilled`.
2. If `currentHp > 0`, start `doCameraShake`:
   - `camRot = Euler(Random.Range(-10,10), Random.Range(-10,10), 0)`;
   - wait 0.1 s;
   - `camRot = camDefaultRotation`.
3. `fadeValueB = 2` (the bloody screen), then `hp -= damage`.
4. If `hp < 0` (strictly):
   - deactivate the root's children;
   - instantiate the death prefab;
   - call its `RespawnAfter`;
   - `DestroyPlayer(0.2)`;
   - `hp = 0` and `weKilled = true`.

**`Update`** (`f37877`):
- The fades lerp to 0 at `dt·2`.
- The camera's local rotation slerps towards `camRot` at `dt·15`.
- `camRot` starts as the zero quaternion, which leaves the rotation unchanged until the first shake.

**`OnGUI`** (`f37886`, depth 10, skin 40923):
- `GUI.color = (1,1,1,fadeValueB)`, then `DrawTexture` of the full-screen bloody screen.
- `GUI.color = (1,1,1,0.9)`, then `Box(W-220, H-55, 100, 45, "HP | " + (int)hp)`.

## Death prefab `german (ragdol)` (`resources.assets:14496`)

- It is not a physics ragdoll. It is an animated body (an `Animation` component with clip 199, played automatically) with its own `Camera` at node 69. The client renders through that camera while the prefab exists.
- It has a play-on-awake 2D `death.m4a` and `WaitForDestroy` 4 s.

**`RagdollController`** (`f29667`..`f29672`, timer 2, skin 40929):
- `RespawnAfter` shows the GUI and calls `InvokeRepeating("_respawnAfter", 1, 1)`.
- Each call decrements the timer. At 0 it calls `clearCamera` and then `RoomMultiplayerMenu.SpawnPlayer`.
- `OnGUI`: label `"Respawn in: " + timer` at `(W/2-75, H/2-15, 150, 30)`.

## Death on the owner, respawn

- Deaths + 1 (the player property); a death from the player's own fall also sends `networkAddMessage(name, "", "fell down", "")`.
- `RoomMultiplayerMenu.SpawnPlayer(team)` instantiates a fresh player prefab of the team at one of its spawn points (rules in `docs/gameplay/remote-players.md`). The client instantiates a fresh copy (new `PlayerDamage`, weapons, spawn protection).
- The per-death `PlayerPrefs` statistics (persistent counters) are not kept.
