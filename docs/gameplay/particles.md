# Particles, effects and trails

Client: `particles.rs` (Shuriken `ParticleSystem`), `legacy_particles.rs` (legacy `ParticleEmitter`), `trail.rs` (`TrailRenderer`), `effects.rs` (the spawned prefabs' effects), particle shaders in `render.rs`. The data is the CrazyGames build's (Unity 5.3); the scripts follow its code (`docs/crazygames-build.md`, "Scripts against Build-7").

## What uses particles

| Prefab / object | Systems | Lifetime script |
|-----------------|---------|-----------------|
| Impacts (`Blood__320` and one `Metal__*` per bullet in `assets/data/prefabs/resources.assets/`, chosen by hit tag in `docs/gameplay/weapons.md`) | bursts of billboards / meshes, play on awake | `WaitForDestroy`: `Destroy(gameObject, lifeTime)`, 0.5-3 s per prefab (`Metal__330` 0.5 s, `Blood__320` 0.8 s) |
| `WFXMR_Explosion 1__4249` (rocket explosion) | 9 systems | `CFX_AutoDestructShuriken` + `ExplosionDamage` (below) |
| Shotgun "Smoke Trail" (child of the shotgun) | one **legacy** emitter | `shotGunAwake`: `emit = false`; `shotGunSmokeEffect`: `emit = true` on each shot, `false` 0.3 s later |
| Rocket | `TrailRenderer` (+ the flying AudioSource) | destroyed with the projectile |

**Example (observed in data):** `Metal__330` (the pistol's metal impact) is a burst of 100 billboards using the lit cyan `PISTOL MAT` material, size ≤ 0.15, speed ≤ 8, capped at **20** particles by `maxNumParticles` (Build-7's `Metal__357`: 100). These are the cyan squares seen when shooting metal.

## ParticleSystem (Shuriken)

All values come from the prefabs' `ParticleSystem` and `ParticleSystemRenderer` data (`assets/data/prefabs/resources/*.json`). The client reads the Unity 2018.3 layout; 5.3 data is upgraded first (next section).

- **Curves:**
  - `MinMaxCurve` states: 0 constant, 1 curve, 2 random between two curves, 3 random between two constants.
  - `MinMaxGradient` states: 0 colour, 1 gradient, 2 random between two colours, 3 random between two gradients, 4 random colour from a gradient.
  - Gradient key times are stored as `u16 / 65535`.
  - Curves are Hermite with the keys' in/out slopes.
- **Main module:** start delay, lifetime, speed, size, rotation, colour, gravity modifier (× `Physics.gravity` (0, -9.81, 0)), simulation space (local/world) and max particles.
- **Emission:** rate over time and bursts.
- **Shape:** sphere, hemisphere, cone and none, with random or spherical direction.
- **Over-lifetime modules:** velocity, force and limit velocity (dampen); size, colour and rotation over lifetime; rotation by speed.
- **Collision:** world mode, using raycasts against the collision world. Bounce, dampen and lifetime loss come from the data.
- **Renderer:**
  - billboard, stretched billboard (speed and length scale) and mesh (`assets/models/*.obj`) render modes;
  - `maxParticleSize` is a fraction of the screen height.
- **Randomness:** Unity's per-particle random numbers are internal to the engine. Individual particles differ from the original, just as they differ between two runs of the original. The distributions follow the data.
- **Play/Stop:**
  - `Stop()` is `StopEmitting`: live particles keep simulating until they die.
  - `IsAlive()` is emitting || particles > 0.

### Unity 5.3 layout

`particles.rs::upgrade_unity_5_3` rewrites a 5.3 `ParticleSystem` (detected by `EmissionModule.m_BurstCount`) the way Unity's serialization upgrade does. Each rule was checked against the 36 effects present in both builds (Build-7 holds Unity's own upgrade of the same data): after the upgrade every field the client reads is equal, except where the effect data itself was edited (collision `radiusScale`, `Metal` max particles) and box sizes of non-box shapes, which nothing reads (**observed**).

| 5.3 | 2018.3 |
|-----|--------|
| colour `{rgba: u32}` (red in the low byte), also gradient keys | `{r, g, b, a}` = byte / 255 |
| `MinMaxCurve` without `minScalar`; "two constants" = `scalar` × first key of `minCurve` / `maxCurve` | `(minScalar, scalar)` |
| `moveWithTransform` bool, true = local | 0 local, 1 world |
| `speed` | `simulationSpeed` |
| `InitialModule.gravityModifier` float | constant `MinMaxCurve` |
| `EmissionModule.rate` + `m_Type` (0 time, 1 distance) | `rateOverTime` (distance is used by no system) |
| `m_BurstCount` ≤ 4 × (`time`, `cnt` min, `cntmax` max) | `m_Bursts` (count random between min and max, 1 cycle, probability 1) |
| shape types SphereShell 1, HemisphereShell 3 (both in the data), ConeShell 7, ConeVolumeShell 9, CircleEdge 11 | Sphere 0, Hemisphere 2, Cone 4, ConeVolume 8, Circle 10 with `radiusThickness` 0 (other types: 1) |
| shape `radius`, `arc` floats; `randomDirection` bool | `radius.value`, `arc.value`; `randomDirectionAmount` 1 / 0, `sphericalDirectionAmount` 0 |

`simulationSpeed` (1.67 on `WFXM_Nuke`) and `prewarm` (three looping systems: the menu scenes and `Bullet__323`) are not applied, as before; none of them is spawned by the ported gameplay.

## Legacy particles (`legacy_particles.rs`)

Unity 5.3 still has the pre-Shuriken components `EllipsoidParticleEmitter` + `ParticleAnimator` + `ParticleRenderer` (removed in 2018.3). In gameplay only the shotgun's "Smoke Trail" uses them (also `Metal__362`, `Glass__361`, `MachineGun sparks__360`, `RocketMissile__367` and old prefabs, none referenced by a used weapon). Its data (`german` / `mercenarys`, **observed**): emission 4 /s, energy 2 s, size 0.3, world space, random rotation, no velocities, ellipsoid 0; animator: colour animation on (5 keys, alpha 0 → 0.65 → 0.48 → 0.33 → 0), `sizeGrow` 0.8895, damping 1, no forces; renderer: `Smoke2` material, `m_StretchParticles` 5, `maxParticleSize` 1.1.

The engine code is native and minified in this build (names like `OJ`), so the rules follow Unity's documented legacy behaviour (**inferred**; listed in `docs/unknowns.md`):

- **Emission** while `emit`: `accumulator += Random(minEmission, maxEmission) · dt`, the whole part is emitted, spread over the frame (positions interpolated from the previous emitter matrix, pre-aged by the rest of the frame). One-shot emitters emit `Random(minEmission, maxEmission)` once.
- **New particle:** position in the ellipsoid (`m_Ellipsoid` · ½, at least `m_MinEmitterRange`); velocity `worldVelocity + rotated localVelocity + rndVelocity · [-1, 1]³`; energy `Random(minEnergy, maxEnergy)`; size `Random(minSize, maxSize)`; rotation `Random(0, 360)` with `rndRotation`; angular velocity `angularVelocity + rndAngularVelocity · [-1, 1]`.
- **Animator each frame:** energy −= dt (dead at 0); velocity += (`force` + `rndForce` jitter) · dt, × `damping`^dt; position += velocity · dt; `size += size · sizeGrow · dt`; colour = the five `colorAnimation` keys spread evenly over `1 − energy / startEnergy`.
- **Renderer:** `m_StretchParticles` 0/2 billboard, 3 stretched (length and velocity scale), 4 horizontal, 5 **vertical billboard** (faces the camera around world Y; the smoke). `maxParticleSize` is a fraction of the screen height, as for Shuriken.
- `emit = false` stops emission; live particles finish their life.

The smoke's `Smoke2` material uses `Hidden/Particles/Alpha Blended +100`: the `Particles/Alpha Blended` program in queue Transparent+100, compiled without fog variants, so the client draws it without fog.

### Particle shaders

The shader dump has no render state for these shaders. The states below come from Unity's built-in shader sources and the WFX shader sources (**inferred**).

| Shader | Blend | Colour |
|--------|-------|--------|
| `Mobile/Particles/Additive` | `SrcAlpha One` | tex · vertex colour |
| `Particles/Additive` | `SrcAlpha One` | 2 · `_TintColor` · tex · vertex colour |
| `Particles/Additive (Soft)` | `One OneMinusSrcColor` | tex · vertex colour, premultiplied |
| `Particles/Alpha Blended` | `SrcAlpha OneMinusSrcAlpha` | 2 · `_TintColor` · tex · vertex colour |
| `Hidden/Particles/Alpha Blended +100` | `SrcAlpha OneMinusSrcAlpha` (from its dump) | as above, no fog |
| `WFX/Additive Alpha8` | `SrcAlpha One` | 2 · `_TintColor` · vertex colour · tex.a |
| `WFX/Multiply Alpha8` | `Zero SrcColor` | tex.a · vertex colour |

- Fog fades additive shaders to black and the multiply shader to white, as in the built-in sources.
- Particles draw after opaques and decals, sorted back to front, with depth test on, ZWrite off and culling off.
- Systems on layer 9 (the shotgun smoke) draw in the Weapon Camera pass.
- In the CrazyGames build (Unity 5.3) the built-in particle shaders are named without the `Legacy Shaders/` prefix that 2017.1 added; the programs are the same.

## TrailRenderer (rocket trail)

Unity 5.3 stores `m_StartWidth` / `m_EndWidth` and five `m_Colors` (`ColorRGBA32`) instead of `m_Parameters`. `trail.rs` turns them into a width curve from start (head) to end (tail) with multiplier 1 and a five-key gradient at the times Unity's upgrade writes (0, 16384, 32768, 49151, 65535 of 65535; **observed** in Build-7's upgraded copy of the same trail). Unity's upgrade gives the width curve flat tangents; the client interpolates the width linearly, as the 5.3 renderer is documented to (**inferred**). The rocket trail: 1 s, width 0.25 → 0.1, white fading out over the last quarter.

- A point expires `m_Time` seconds after it is added.
- A new point is added when the head (the transform's position) moves more than `m_MinVertexDistance` from the newest point.
- The strip is view-aligned: each vertex is offset along `tangent × toCamera`.
- Width (`widthCurve · widthMultiplier`), colour (`colorGradient`) and the texture u coordinate are evaluated at the point's distance fraction from the head (0) to the tail (1). This parameterisation is **inferred**: Unity documents width/colour "over the trail's length" without saying whether it uses distance or age.

## Rocket explosion (`WFXMR_Explosion 1__4249`)

Order of events in the CrazyGames code (**observed**: `research/notes/crazygames/explosiondamage.js`, `cfx_autodestructshuriken.js`; the visible result is **inferred** and not yet compared with a recording):

1. The nine systems play on awake.
2. `CFX_AutoDestructShuriken.OnEnable` starts `CheckIfAlive`: every 0.5 s, if the root `ParticleSystem` is not `IsAlive(true)` (children included), destroy the object (or deactivate it when `OnlyDeactivate`).
3. `ExplosionDamage.Start`:
   - every collider in `radius` gets `SendMessageUpwards("ApplyDamage", (1 - clamp01(dist / radius)) · damage)` unless `isRemote`, with `dist` measured to the closest point of its bounds;
   - every rigidbody in `radius` gets `AddExplosionForce(power, position, radius, 3)`;
   - if the object has a legacy `ParticleEmitter`: `emit = true`, wait 0.5 s, `emit = false` (this prefab has none);
   - `Destroy(gameObject, explosionTimeout)` (2 s).

**Result:** the explosion plays and is removed 2 s after its `Start` (its particles live up to 3 s, so `CheckIfAlive` does not end it first); the 2D explosion sound (`Pixel Explosion - roket.m4a`) plays until then.

Build-7 differed: its `ExplosionDamage.Start` called `Stop()` on the systems before their first update and `Play()` 0.5 s later, and `CheckIfAlive`, resuming first, found nothing alive at 0.5 s and destroyed the object, so no particles were drawn and the sound was cut at 0.5 s.

Explosion damage only reaches `HitBox` receivers, which only remote players have (`docs/gameplay/damage.md`). The client applies `effects::explosion_damage` to every overlapped hit box of the other players' copies, each through the `SendMessageUpwards` chain (`docs/gameplay/remote-players.md`). An explosion of a remote rocket (`isRemote`) damages nothing.

Tests: the `Metal__330` impact plays its sounds and is destroyed after 0.5 s; the explosion draws and is destroyed 2 s after its `Start`; the rocket trail follows and dies with the rocket; curves, gradients, bursts and shapes; the 5.3 upgrade (colours, two-constant curves, bursts, shell shapes, gravity, simulation space); the 5.3 trail (widths, colours); the legacy smoke (emission while on, growth, colour animation, death).
