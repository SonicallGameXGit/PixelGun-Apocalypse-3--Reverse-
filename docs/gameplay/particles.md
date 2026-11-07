# Particles, effects and trails

Client: `particles.rs` (Shuriken `ParticleSystem`), `trail.rs` (`TrailRenderer`), `effects.rs` (the spawned prefabs' effects), particle shaders in `render.rs`.

## What uses particles

| Prefab / object | Systems | Lifetime script |
|-----------------|---------|-----------------|
| Impacts (`Metal__*`, `Glass__392`, `Blood__356` in `assets/data/prefabs/resources.assets/`, chosen by hit tag in `docs/gameplay/weapons.md`) | bursts of billboards / meshes, play on awake | `WaitForDestroy` (`f29902`): `Destroy(gameObject, lifeTime)`, 0.3-3 s per prefab (`Metal__357` 0.5 s, `Blood__356` 0.8 s) |
| `WFXMR_Explosion 1` (rocket explosion) | several systems | `CFX_AutoDestructShuriken` (`f65523`..`f65525`) + `ExplosionDamage` (`f28826`..`f28828`) |
| Shotgun smoke (child of the shotgun) | one system | `Stop()` in `WeaponScript.Start`; `shotGunSmokeEffect`: `Play()` on each shot, `Stop()` 0.3 s later |
| Bullet, rocket | `TrailRenderer` (+ the rocket's flying AudioSource) | destroyed with the projectile |

**Example (observed in data):** `Metal__357` is a 100-particle burst of billboards using the lit cyan `PISTOL MAT` material, with size ≤ 0.15 and speed ≤ 8. These are the cyan squares seen when shooting metal.

## ParticleSystem (Shuriken, Unity 2018.3)

All values come from the prefabs' `ParticleSystem` and `ParticleSystemRenderer` data (`assets/data/prefabs/resources/*.json`).

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

### Particle shaders

The shader dump has no render state for these shaders. The states below come from Unity's built-in shader sources and the WFX shader sources (**inferred**).

| Shader | Blend | Colour |
|--------|-------|--------|
| `Mobile/Particles/Additive` | `SrcAlpha One` | tex · vertex colour |
| `Particles/Additive` | `SrcAlpha One` | 2 · `_TintColor` · tex · vertex colour |
| `Particles/Additive (Soft)` | `One OneMinusSrcColor` | tex · vertex colour, premultiplied |
| `Particles/Alpha Blended` | `SrcAlpha OneMinusSrcAlpha` | 2 · `_TintColor` · tex · vertex colour |
| `WFX/Additive Alpha8` | `SrcAlpha One` | 2 · `_TintColor` · vertex colour · tex.a |
| `WFX/Multiply Alpha8` | `Zero SrcColor` | tex.a · vertex colour |

- Fog fades additive shaders to black and the multiply shader to white, as in the built-in sources.
- Particles draw after opaques and decals, sorted back to front, with depth test on, ZWrite off and culling off.
- Systems on layer 9 (the shotgun smoke) draw in the Weapon Camera pass.

## TrailRenderer (bullet and rocket trails)

- A point expires `m_Time` seconds after it is added.
- A new point is added when the head (the transform's position) moves more than `m_MinVertexDistance` from the newest point.
- The strip is view-aligned: each vertex is offset along `tangent × toCamera`.
- Width (`widthCurve · widthMultiplier`), colour (`colorGradient`) and the texture u coordinate are evaluated at the point's distance fraction from the head (0) to the tail (1). This parameterisation is **inferred**: Unity documents width/colour "over the trail's length" without saying whether it uses distance or age.

## Rocket explosion (`WFXMR_Explosion 1`)

Decompiled order of events (**observed** in the code; the visible result is **inferred** and not yet compared with the original):

1. `CFX_AutoDestructShuriken.OnEnable` runs first and starts the coroutine `CheckIfAlive`: every 0.5 s, if no `ParticleSystem` in the object `IsAlive(true)`, it destroys the object (or deactivates it when `OnlyDeactivate`).
2. `ExplosionDamage.Start`:
   - applies damage: every collider in `radius` gets `ApplyDamage((1 - clamp01(dist / radius)) · damage)`, where `dist` is measured to the closest point of its bounds;
   - then calls `Stop()` on the particle systems before their first update;
   - waits 0.5 s, then calls `Play()`.
3. The first `CheckIfAlive` check comes at 0.5 s, before or in the same frame as the `Play()`. Nothing is alive, so the explosion object is destroyed.

**Result:** no particles are drawn, and the 2D explosion sound (`Pixel Explosion - roket.m4a`) is cut off at 0.5 s when its GameObject is destroyed. The client reproduces this. If the original shows an explosion, the order of the 0.5 s coroutine resumes is the thing to re-check (both coroutines wait `WaitForSeconds(0.5)`; Unity resumes them in start order).

Explosion damage only reaches `HitBox` receivers, which only remote players have (`docs/gameplay/damage.md`). The client applies `effects::explosion_damage` to every overlapped hit box of the other players' copies, each through the `SendMessageUpwards` chain (`docs/gameplay/remote-players.md`). An explosion of a remote rocket (`isRemote`) damages nothing.

Tests: the `Metal__357` impact plays its sounds and is destroyed after 0.5 s; the explosion never draws and is destroyed at 0.5 s; the rocket trail follows and dies with the rocket; curves, gradients, bursts and shapes.
