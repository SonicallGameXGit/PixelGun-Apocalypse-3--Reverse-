# Player movement and collision

Implementation: `crates/pga3-client/src/player.rs` (scripts), `physics.rs` (CharacterController, collision world, queries).

Labels: **observed** = read from decompiled code or extracted data; **inferred** = reasoned, not yet measured against the original; **unknown** = listed in `unknowns.md`.

## Evidence

- Values: player prefab `assets/data/prefabs/resources/german.json` (`mercenarys.json` and `zombie.json` are the other player prefabs), `assets/data/settings/{InputManager,TimeManager,PhysicsManager}.json`.
- Code: `pga3-v18.wasm` translated with `wasm2c` and annotated with `tools/re/` (see `tools/re/README.md`). The wasm function indices below are in the WABT index space (imports first). Decompiled listings are kept in `research/notes/wasm/` (gitignored): `fpscontroller_compact.c`, `input.c`, `camerabob.c`, `roommenu.c`, `spawnplayer.c`, `ladder.c`.
- Engine: Unity 2018.3's CharacterController and scene queries are PhysX 3.4. Its source is public under BSD-3-Clause: <https://github.com/NVIDIAGameWorks/PhysX-3.4>, files `PhysX_3.4/Source/PhysXCharacterKinematic/src/Cct*.cpp` and `PhysX_3.4/Source/GeomUtils/src/{GuInternal.h, GuMTD.cpp, sweep/GuSweepCapsuleTriangle.cpp, mesh/GuSweepsMesh.cpp}` (local copies in `research/notes/physx/`, gitignored).

| Method | wasm function |
|--------|---------------|
| `FPScontroller.Awake` / `UpdateFunction` / `ApplyInputVelocityChange` / `ApplyGravityAndJumping` | f28856 / f28858 / f28859 / f28860 |
| `FPScontroller.FixedUpdate` / `Update` / `OnRunning` / `OffRunning` / `Crouch` / `Prone` | f28871 / f28872 / f28873 / f28874 / f28875 / f28876 |
| `FPScontroller.OnControllerColliderHit` / `.ctor` | f28879 / f28887 |
| `FPSinput.LateUpdate` | f28898 |
| `FPSMouseLook.Update` | f28849 |
| `CameraBob.Awake` / `FixedUpdate` / `.ctor` | f65539 / f65540 / f29906 |
| `RoomMultiplayerMenu.Start` / `.cctor` / `SpawnPlayer` | f29719 / f29739 / f29731 |
| `FPScontroller.OnLadder` / `OffLadder` | f28877 / f28878 |
| `Ladder.Start` / `ClimbDirection` / `OnTriggerEnter` | f38664 / f38665 / f38666 |
| `LadderPlayer.LatchLadder` / `Start` / `UnlatchLadder` / `FixedUpdate` / `RaycastCheck` | f38667 / f38669 / f38670 / f38671 / f38672 |
| `Physics.SphereCast(origin, radius, direction, out hit, maxDistance)` | f62646 (layer mask `0xFFFFFFFB`, QueryTriggerInteraction.UseGlobal) |

`FPScontroller` is Unity's stock `CharacterMotor` (Standard Assets) with game-specific running, crouch, prone, ladder and fall-damage code. The stock parts were checked line by line against the decompiled code. Moving platforms are disabled in the prefab and are not ported.

## Player prefab (observed)

`German` root, layer 2 (Ignore Raycast), tag 6 (Unity's built-in "Player"). Hierarchy used by movement: `German` (CharacterController, FPScontroller, FPSinput, FPSMouseLook axes 1) → `LookObject` (local (-0.01062, 1.9171467, -0.0956), FPSMouseLook axes 2) → `Main Camera` (local (0, 0, 0.178226), FOV 65, near 0.2, far 1000, CameraBob).

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
- `FixedUpdate` (0.02 s) runs fall-damage bookkeeping, `LadderPlayer` and `CameraBob`; the physics step after it raises the trigger events (`Ladder.OnTriggerEnter`).
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
  - Afterwards `sensX = sensY = currentSensitivity = RoomMultiplayerMenu.sens`, ×0.25 while the first weapon the script
    cached is aiming. So the first frame turns with the prefab values (root `sensitivityX` 4.0, LookObject `sensitivityY`
    3.5) and every later frame with the previous frame's menu value (`docs/gameplay/settings.md`, "FPSMouseLook").
  - `sens` comes from PlayerPrefs `"sens"`, default **1.8** (`.cctor`); the room menu's `<`/`>` change it.
  - `EnableHelper` turns both scripts off (and `canControl`) while the room menu is open.
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

- `Physics.SphereCast` here uses the default layer mask (all layers but 2, Ignore Raycast). Every trigger in the maps (the ladder volumes) is on layer 2, so the cast only sees solid map geometry.
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

## Ladders (observed)

Maps with ladders: Crash, Fight, Pete, Scaz, Village (and their `ZM_` variants). Each `Ladder` GameObject (layer 2, tag "Ladder") has a trigger BoxCollider, a solid `MiddleCollider` child (layer 0) and `LadderBottom` / `LadderTop` transforms. On Crash the trigger is a 0.078 m slab 0.16-0.24 m in front of the 0.088 m MiddleCollider.

- `Ladder.Start`: `climbDirection = ladderTop.position - ladderBottom.position` (4.71 m straight up on Crash).
- `Ladder.OnTriggerEnter(other)`: if `other.transform.root.tag == "Player"`, `root.GetComponent<LadderPlayer>().LatchLadder(this)`. It fires only when the overlap starts, so a player who let go inside the volume does not latch again until leaving it.
- `LadderPlayer.LatchLadder`: store the ladder, `latchedToLadder = true`, copy its climb direction, `SendMessage("OnLadder")`, `trigger = true`.
- `FPScontroller.OnLadder`: `onLadder = true`, `inputMoveDirection = 0`, `movement.enableGravity = false`. While `onLadder`, `Update` clears grounded/crouch/prone each frame and running is off. `FPSinput` still writes the walking input, so `UpdateFunction` keeps moving the player horizontally (air control, no gravity), mostly into the MiddleCollider.
- `LadderPlayer.FixedUpdate` (only while latched):
  1. if `trigger`: `RaycastCheck()`;
  2. if `GetKey(Space)`: `UnlatchLadder()`. The method carries on, so that step still moves;
  3. `vertical = climbDirection.normalized * GetAxis("Vertical")`, negated when `Camera.main.forward.y <= climbDownThreshold` (-0.6, about 37° down);
  4. `lateral = transform.TransformDirection(GetAxis("Horizontal"), 0, inLandingPad ? GetAxis("Vertical") : 0)`; `inLandingPad` is never set;
  5. `controller.Move((vertical + lateral) * climbSpeed (6) * Time.deltaTime (0.02))`.
- `RaycastCheck`: `p = position + controller.center` (not rotated), `bottom = p - up * height/2`, `top = bottom + up * height`; `if (!Physics.CapsuleCast(bottom, top, radius, transform.forward, out hit, radius)) UnlatchLadder()`. The player lets go when nothing is within one radius in front, for example above the top of the MiddleCollider.
- `UnlatchLadder`: clear the latch, ladder and `trigger`, then `SendMessage("OffLadder", ladderMovement)`. `FPScontroller.OffLadder`: `onLadder = false`, `enableGravity = true`; it also computes the camera's forward direction and never uses it.
- **Inferred**: trigger overlap uses PhysX's kinematic proxy capsule, which is the controller capsule scaled by `PxControllerDesc::scaleCoeff` (default 0.8; Unity does not expose it). On Crash both 0.8 and 1.0 reach the trigger slab.

## CharacterController (PhysX 3.4 port)

Unity's `CharacterController` is the PhysX 3.4 capsule controller. `physics.rs` ports `SweepTest::moveCharacter` and `doSweepTest` (`CctCharacterController.cpp`) for this game's settings. Paths that need other settings (other controllers, obstacles, moving platforms, the walk experiment, constrained climbing) are left out.

- **Mapping from Unity** (inferred from Unity's documented behaviour; only the values are observed):
  - capsule radius 0.36;
  - cylinder height `2.2 - 2·0.36`;
  - contact offset = skinWidth 0.08;
  - stepOffset 0.3;
  - `slopeLimit = cos(90°)`, which is -4.4e-8 in float;
  - `minDist` = minMoveDistance 0;
  - climbing mode `eEASY` (the PhysX default);
  - manager defaults: overlap recovery on, precise sweeps on, prevent-vertical-sliding off.
- **Consequence of slopeLimit 90**: `testSlope(n) = n·up >= 0 && n·up < slopeLimit` is never true. So the down-pass check that redoes the move as a "walk experiment" never runs, and no surface is non-walkable. There is no step validation and no flattening of the slide normal.
- **moveCharacter**:
  - `stepOffset = 0` when the motion goes up (jumping).
  - The motion is split into `up = max(y, 0)`, `side = (x, 0, z)` and `down = min(y, 0)`.
  - If the side part is not almost zero (each component above 1e-6), the up pass gets `+stepOffset` and the down pass `-stepOffset`.
  - Passes:
    - **Up**: 1 iteration, or 10 when there is no side motion. If it collided, the step is clamped to the height actually gained. Flag Above.
    - **Side**: 10 iterations. Flag Sides.
    - **Down**: 1 iteration. Flag Below, only when the motion's y is ≤ 0.
- **doSweepTest** (one pass):
  - Loop up to the iteration limit:
    - Stop when the remaining length is ≤ `minDist`.
    - Stop when the new direction points against the pass direction (Quake 2 rule).
    - Sweep the real (not inflated) capsule for `length + contactOffset`.
    - No hit: move to the target.
    - Hit at distance 0 (the capsule already overlaps something): overlap recovery. `computeMTD` pushes the capsule inflated by the contact offset out of the geometry (up to 4 iterations; contacts combined per axis as in `GuMTD.cpp processContacts`) and ends the pass.
    - Otherwise:
      - move `distance - contactOffset` (only if positive);
      - `collisionResponse` with bump 0 and friction 1: target = current + (tangential part of the reflected direction) × remaining length, not normalised;
      - every hit calls `OnControllerColliderHit`.
- **Sweep rules** (`GuSweepCapsuleTriangle.cpp`, `GuSweepsMesh.cpp`, `GuInternal.h`):
  - triangles with `normal·direction > 0` are culled (back faces);
  - a triangle the capsule already intersects is a hit at distance 0;
  - among hits within 1e-3 (relative) of the closest, the triangle most opposed to the motion wins (`keepTriangle`).

  The client computes the exact time of impact (segment ends vs face, edges and vertices; triangle vertices vs the capsule side; triangle edges vs the segment) over a 4 m uniform grid. BoxColliders become 12 outward-facing triangles.
- **Results**:
  - The capsule rests one contact offset (0.08 m) above the floor; the root is 0.15 m above it.
  - Ledges up to 0.3 m are stepped onto directly.
  - Higher edges are climbed only by the slide over the rounded bottom (the contact must be below the bottom sphere's centre after the step-up pass, at most about 0.74 m).
    - `CharacterMotor` recomputes the velocity from the actual displacement every frame, so on a steep edge that slide creeps (0.15 mm per frame on Crash's 0.63 m walls) and acts as a wall unless you jump.
    - The same happens in the original, because the algorithm and inputs are the same.
  - The standing capsule needs 2.36 m of headroom (2.2 m + 2 × 0.08 m). Only `FPScontroller` changes `CharacterController.height` (checked: the only callers of `set_height` are `Update`, `Crouch` and `Prone`), so lower openings (for example 1.78 m on Castle, 1.89 m on Bridge) need crouching in the original too.
- **Unity queries** (`Physics.SphereCast`, `Physics.CapsuleCast`): the same sweep, default layer mask (all but Ignore Raycast), triggers included (`queriesHitTriggers`). Colliders the shape starts inside are not reported, as Unity's documentation states for these casts.

## Not ported yet

- Moving platforms (disabled in the prefab).
- `PlayerDamage`, footstep/jump sounds (`FPSSoundController` receives `OnJump` / `OnLand` / `OnFall`), and network sync of the movement.

## Tests

`cargo test -p pga3-client` covers:
- walk 5.5 / run 8.5 / backwards 2.75 m/s;
- jump apex 1.6 m;
- crouch to height 1.5 at 2.5 m/s;
- Unity axis smoothing;
- capsule sweep, slide along a wall, stepping onto a 0.2 m ledge, sliding over a 0.5 m edge, being stopped by a 1 m wall, walking under a 2.4 m ceiling;
- the SphereCast layer mask;
- capsule-box trigger overlap;
- ladders: latch on entering the trigger, climb at 6 m/s, climb down when looking down, let go with Space and fall back;
- every spawn point of every map: the player settles exactly at floor + 0.15 m and survives running in four directions.

Diagnostics (ignored by default, run with `cargo test --release -p pga3-client -- --ignored --nocapture <name>`):
- `movement_cost`: time per frame of the player update (7-34 µs);
- `report_unexplained_stops`: runs from every spawn in 16 directions and lists the stops with no wall at chest height, with the heights at which something is in front. Current result: 8 places (each also in the `ZM_` copy), all real geometry: 1.78-1.89 m openings and 0.63 m walls.
