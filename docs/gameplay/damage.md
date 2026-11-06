# Damage, death and respawn

Client: `damage.rs` (`PlayerDamage`, spawn protection, `RagdollController`), driven from `main.rs`.

## Where damage comes from

- Bullets (`Bullet`) and explosions (`ExplosionDamage`) call `SendMessage("ApplyDamage", ...)` on what they hit. Only `HitBox.ApplyDamage` receives it, and it forwards the damage as the `DoDamage` RPC (**observed**).
- The owner's `HitBox`es are destroyed in `PlayerNetworkController.Awake`, so only **remote** players can be hit. Bullets and rockets therefore damage nothing locally until networking (Photon) exists.
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

**`OnGUI`** (`f37886`, depth 10, skin 40959):
- `GUI.color = (1,1,1,fadeValueB)`, then `DrawTexture` of the full-screen bloody screen.
- `GUI.color = (1,1,1,0.9)`, then `Box(W-220, H-55, 100, 45, "HP | " + (int)hp)`.

## Death prefab `german (ragdol)` (`resources.assets:14520`)

- It is not a physics ragdoll. It is an animated body (an `Animation` component with clip 199, played automatically) with its own `Camera` at node 69. The client renders through that camera while the prefab exists.
- It has a play-on-awake 2D `death.m4a` and `WaitForDestroy` 4 s.

**`RagdollController`** (`f29667`..`f29672`, timer 2, skin 40964):
- `RespawnAfter` shows the GUI and calls `InvokeRepeating("_respawnAfter", 1, 1)`.
- Each call decrements the timer. At 0 it calls `clearCamera` and then `RoomMultiplayerMenu.SpawnPlayer`.
- `OnGUI`: label `"Respawn in: " + timer` at `(W/2-75, H/2-15, 150, 30)`.

## Respawn (stand-in)

`SpawnPlayer` picks a random spawn point of the player's team and instantiates a fresh player prefab through PUN. The client has no rooms or teams yet, so it uses this stand-in:
- it instantiates a fresh copy of the local player prefab (new `PlayerDamage`, weapons, spawn protection) at a random spawn point of the current map;
- the per-death `PlayerPrefs` statistics are skipped.

The real selection depends on the room and team state (`docs/unknowns.md`).
