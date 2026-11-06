# Player movement and collision

Implementation: `crates/pga3-client/src/player.rs` (scripts), `physics.rs` (CharacterController, collision world).

Labels: **observed** = read from decompiled code or extracted data; **inferred** = reasoned, not yet measured against the original; **unknown** = listed in `unknowns.md`.

## Evidence

- Values: player prefab `assets/data/prefabs/resources/german.json` (`mercenarys.json` and `zombie.json` are the other player prefabs), `assets/data/settings/{InputManager,TimeManager,PhysicsManager}.json`.
- Code: `pga3-v18.wasm` translated with `wasm2c` and annotated with `tools/re/` (see `tools/re/README.md`). The wasm function indices below are in the WABT index space (imports first). Decompiled listings are kept in `research/notes/wasm/` (gitignored): `fpscontroller_compact.c`, `input.c`, `camerabob.c`, `roommenu.c`, `spawnplayer.c`.

| Method | wasm function |
|--------|---------------|
| `FPScontroller.Awake` / `UpdateFunction` / `ApplyInputVelocityChange` / `ApplyGravityAndJumping` | f28856 / f28858 / f28859 / f28860 |
| `FPScontroller.FixedUpdate` / `Update` / `OnRunning` / `OffRunning` / `Crouch` / `Prone` | f28871 / f28872 / f28873 / f28874 / f28875 / f28876 |
| `FPScontroller.OnControllerColliderHit` / `.ctor` | f28879 / f28887 |
| `FPSinput.LateUpdate` | f28898 |
| `FPSMouseLook.Update` | f28849 |
| `CameraBob.Awake` / `FixedUpdate` / `.ctor` | f65539 / f65540 / f29906 |
| `RoomMultiplayerMenu.Start` / `.cctor` / `SpawnPlayer` | f29719 / f29739 / f29731 |

`FPScontroller` is Unity's stock `CharacterMotor` (Standard Assets) with game-specific running, crouch, prone, ladder and fall-damage code. The stock parts were checked line by line against the decompiled code. Moving platforms are disabled in the prefab and are not ported.

## Player prefab (observed)

`German` root, layer 2. Hierarchy used by movement: `German` (CharacterController, FPScontroller, FPSinput, FPSMouseLook axes 1) → `LookObject` (local (-0.01062, 1.9171467, -0.0956), FPSMouseLook axes 2) → `Main Camera` (local (0, 0, 0.178226), FOV 65, near 0.2, far 1000, CameraBob).

| Component | Values |
|-----------|--------|
| CharacterController | height 2.2, radius 0.36, center (0, 1.03, 0.06), slopeLimit 90, stepOffset 0.3, skinWidth 0.08, minMoveDistance 0 |
| movement | Walk 5.5, Run 8.5, CrouchSpeed 2.5, ProneSpeed 1.0, crouchHeight 1.5, proneHeight 0.7, crouchSmooth 0.1, maxGroundAcceleration 30, maxAirAcceleration 20, gravity 20, maxFallSpeed 100, slopeSpeedMultiplier flat 1 |
| jumping | baseHeight 1.6, extraHeight 1.8, perpAmount 0, steepPerpAmount 0 |
| fallDamage | fallLimit 0.85 s, multiplier 5 |
| CameraBob | walk 0.13, run 0.22, idle 0.01, amount 0.08, smooth 9 |
| LadderPlayer | climbSpeed 6, climbDownThreshold -0.6 |

## Frame order (observed)

- `useFixedUpdate` is never set (prefab and constructor), so `FPScontroller.Update` moves the player once per rendered frame with `Time.deltaTime`, capped at `Maximum Allowed Timestep` 0.33333334 s.
- `FixedUpdate` (0.02 s) runs fall-damage bookkeeping and `CameraBob`.
- `FPSinput.LateUpdate` writes the inputs, so a key press moves the player on the next frame.

## Input (observed)

- `FPSinput.LateUpdate`:
  - `dir = (CF2Input.GetAxis("Horizontal"), 0, GetAxis("Vertical"))`.
  - If `dir != 0`: `len = |dir|`, `dir /= len`, `dir *= min(1, len)²`.
  - `inputMoveDirection = transform.rotation * dir`.
- Keys:
  - jump: `GetKeyDown(Space)`;
  - run: `GetKey(LeftShift)`;
  - crouch: `GetKeyDown(C)`;
  - prone: `GetKeyDown(LeftControl)`.
- Because `inputJump` is a one-frame key-down, `ApplyGravityAndJumping` clears `holdingJumpButton` on the next frame. The `extraHeight` boost therefore never applies, and every jump peaks at `baseHeight` = 1.6 m (take-off speed √(2·1.6·20) = 8 m/s).
- Horizontal/Vertical are keyboard axes (arrows, alt keys a/d/s/w):
  - gravity 3, sensitivity 3, dead 0.001, snap on.
  - The client reproduces Unity's axis smoothing: move towards ±1 at 3/s, back to 0 at 3/s, and jump to 0 on reversal.
  - **Inferred**: `CF2Input` (ControlFreak2) falls back to Unity's `Input` axes on desktop.
- Mouse look (`FPSMouseLook.Update`, skipped while `timeScale < 0.01`):
  - Root (axes 1): `Rotate(0, MouseX·sensX, 0)`.
  - LookObject (axes 2): `rotationY = clamp(rotationY + MouseY·sensY, -80, 80)`, `localEulerAngles = (-rotationY, y, 0)`.
  - Afterwards `sensX = sensY = currentSensitivity = RoomMultiplayerMenu.sens`, ×0.25 while the weapon is aiming.
  - `sens` comes from PlayerPrefs `"sens"`, default **1.8** (`.cctor`).
  - Mouse axes have sensitivity 0.1, so the client turns 0.1 × 1.8 = 0.18° per pixel. **Inferred**: Unity WebGL reports raw pointer-lock pixels; needs measuring.

## FPScontroller.Update (observed)

1. `UpdateFunction()` (when `!useFixedUpdate`).
2. Running:
   - Starts if `GetAxis("Vertical") > 0.1 && inputRun && canRun && !onLadder && Walking`, and then only if `canStand && canStandCrouch`.
   - `OnRunning` sets forward and sideways speed to Run, and `extraHeight = baseHeight + 0.15`.
   - Otherwise `OffRunning`: when not crouched or prone, forward/sideways = Walk, backwards = Walk × 0.5 (2.75), `extraHeight = baseHeight`.
3. `Walking = |velocity.x| > 0.01 || |velocity.z| > 0.01`.
4. If `canControl`:
   - Run `Crouch()` / `Prone()` when off a ladder; on a ladder, clear grounded, crouch and prone.
   - Then, if neither crouched nor prone and `height < standartHeight - 0.01`, lerp three values with `t = dt / crouchSmooth`:
     - controller height → `standartHeight`;
     - center.y → `centerY`;
     - LookObject local y → `standartHeight`. The target is 2.2, not the prefab's 1.917. After the first crouch the eye ends about 0.27 m higher, as decompiled.

### UpdateFunction (stock CharacterMotor plus one game change)

```
velocity = ApplyInputVelocityChange(movement.velocity)
if enableGravity:
    if (prone || crouch) && inputJump: return        // game-specific: that frame does not move
    velocity = ApplyGravityAndJumping(velocity)
offset = velocity·dt
pushDown = max(stepOffset, |offset.xz|)
if grounded: offset -= pushDown·up
groundNormal = 0; flags = controller.Move(offset)    // OnControllerColliderHit sets groundNormal
lastHitPoint = hitPoint; lastGroundNormal = groundNormal
oldH = velocity.xz
movement.velocity = (pos - lastPos)/dt
movement.velocity = oldH == 0 ? (0, v.y, 0) : oldH·clamp01(dot(newH, oldH)/|oldH|²) + v.y·up
if movement.velocity.y < velocity.y - 0.001:
    if movement.velocity.y < 0: movement.velocity.y = velocity.y else holdingJumpButton = false
if grounded && !(groundNormal.y > 0.01): grounded = false; OnFall; pos += pushDown·up
elif !grounded && groundNormal.y > 0.01: grounded = true; jumping = false; OnLand
```

- **ApplyInputVelocityChange**:
  - Desired velocity: `inputMoveDirection` in local space, times `MaxSpeedInDirection` (stock ellipse of forward/backwards vs sideways), times the slope curve (always 1).
  - Grounded: projected onto the ground plane (`Cross(Cross(up, v), n).normalized·|v|`). Airborne: `velocity.y = 0`.
  - The change is clamped to `(grounded ? 30 : 20)·dt`.
  - Grounded: `velocity.y = min(velocity.y, 0)`.
  - The steep-slope slide branch exists but never triggers: `slopeLimit` 90 → `cos 90° ≈ -4e-8`.
- **ApplyGravityAndJumping**:
  - Gravity: grounded `y = min(0, y) - 20·dt`; airborne `y = movement.velocity.y - 20·dt`, clamped at -100.
  - Jump when grounded and the press is under 0.2 s old:
    - `jumpDir = Slerp(up, groundNormal, perp = 0) = up`;
    - `y = 0`, then `+ jumpDir·√(2·baseHeight·gravity)`;
    - send `OnJump`.
- **OnControllerColliderHit**:
  - Condition: `normal.y > 0 && normal.y > groundNormal.y && moveDirection.y < 0`.
  - Then `groundNormal = normal`; the previous normal is reused when the hit point moved by less than √0.001.
  - Pushes rigidbodies (`pushPower` 3); there are none in the maps.

### Crouch / Prone (observed)

- **Crouch**:
  - C toggles `crouch` when not running, off ladders and `canStand`; this also clears prone.
  - Every frame: `Physics.SphereCast(position, radius, up, standartHeight)` from the feet.
  - Hit: `canStand = false` (if crouched) and `canStandCrouch = false`.
  - No hit: jump or running clears crouch; `canStand = canStandCrouch = true`.
  - While crouched and the height is not within ±0.01 of 1.5, lerp:
    - height → 1.5, center.y → 0.75, LookObject y → 1.5;
    - all three speeds = 2.5.
  - The speeds are only written while the height is still changing.
- **Prone**:
  - Left Ctrl toggles `prone` when not running, off ladders and `canStand || crouch`.
  - Afterwards `crouch = !prone && !canStandCrouch`.
  - Jump while prone (if `canStand`) leaves prone; it crouches if `!canStandCrouch`.
  - While prone or running: sphere cast up to `crouchHeight × 0.9`. Running stands up from prone when nothing is above.
  - Lerp towards height 0.7 (center 0.35, LookObject 0.7); speeds = 1.0.

### FixedUpdate: fall damage (observed)

- `playerYSpeed = (previousY - y)/0.02`; `timeInAir += 0.02` while airborne and falling.
- On landing with `timeInAir > 0.85`: `PlayerDamage.ApplyFallDamage(playerYSpeed × 5)`.
- `ApplyFallDamage` itself is not decompiled yet; the client only reports the amount.

### CameraBob.FixedUpdate (observed, skipped while prone)

- Waves: `s1 = sin(timer)`, `s2 = sin(2·timer)`; `timer += bobbingSpeed` (wraps at 2π).
- Target: zero, or `midpoint` when `s2 == 0`. When grounded: `(mid.x + s1·amount, mid.y + s2·amount, 0)`. The z is 0, so the camera's 0.178 m forward offset is lerped away while bobbing, as decompiled.
- Speed and amount from the state:
  - walking: 0.13 / 0.08;
  - running: 0.22 / 0.08;
  - idle: 0.01 / 0.024.
- `localPosition = Lerp(localPosition, target, deltaTime·smooth)`. Inside FixedUpdate, deltaTime is 0.02.

### Spawning (observed)

`RoomMultiplayerMenu.SpawnPlayer` instantiates the player prefab (`PhotonNetwork.Instantiate`) at a random spawn point of the team, with the spawn point's position and rotation. Spawn points float 1–3 m above the floor; the player drops onto it.

## CharacterController (PhysX) model

Unity's `CharacterController` is the PhysX 3.x capsule controller.

- **Observed**:
  - capsule height/radius/center and skin width (contact offset) from the prefab;
  - collision flags Sides 1, Above 2, Below 4;
  - map colliders are non-convex MeshColliders (BSP meshes) and BoxColliders;
  - `queriesHitBackfaces` off, `queriesHitTriggers` on.
- **Inferred** from the PhysX controller's design, to be measured against the original:
  - The motion is split into up (+ stepOffset), side and down (- stepOffset) passes.
  - Each pass sweeps the capsule inflated by the skin width, and slides (`friction 1, bump 0`) in the side pass (10 iterations max). The up and down passes do not slide.
  - The capsule rests one skin width (0.08 m) above the floor. With center.y 1.03, the root sits 0.15 m above it.
  - No auto-step while moving up.
  - In the side pass, the slide normal is made horizontal.
  - Step validation: if the down pass lands on a contact higher than the original bottom + stepOffset, the move is redone without the step. Ledges above 0.3 m therefore block.
  - Overlap recovery pushes the capsule out of penetrations before moving.
  - Back faces are ignored; the capsule's centre offset rotates with the root's yaw.
- The client's sweeps are exact capsule-triangle time-of-impact tests:
  - segment ends vs face, edges and vertices;
  - triangle vertices vs the capsule side;
  - triangle edges vs the segment.

  The world is a 4 m uniform grid of triangles.
- `Physics.SphereCast` (crouch checks) ignores colliders the sphere starts inside. It includes triggers, so ladder volumes also block standing up.

## Not ported yet

- Ladders (`Ladder` trigger volumes with `MiddleCollider` boxes, `LadderPlayer`), moving platforms (disabled).
- Aim sensitivity (weapons), `PlayerDamage`, footstep/jump sounds (`FPSSoundController` receives `OnJump` / `OnLand` / `OnFall`), and network sync of the movement.

## Tests

`cargo test -p pga3-client` covers:
- walk 5.5 / run 8.5 / backwards 2.75 m/s;
- jump apex 1.6 m;
- crouch to height 1.5 at 2.5 m/s;
- Unity axis smoothing;
- capsule sweep, slide and step;
- every spawn point of every map: the player settles exactly at floor + 0.15 m and survives running in four directions.
