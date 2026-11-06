# Audio

Client: `crates/pga3-client/src/audio.rs` (mixer, AudioSources, listener), `footsteps.rs` (`FPSSoundController`). Decoding uses `symphonia` (AAC in MP4); output goes through `cpal`.

## How Unity WebGL plays sound

Evidence: `research/unity-build/unpacked/pga3-v18.framework.js`, the `_JS_Sound_*` functions (**observed**).

- Every `AudioSource` is one Web Audio channel: `source → [PannerNode] → GainNode → destination`.
- The `PannerNode` exists only for 3D sources (`spatialBlend` > 0). Its `rolloffFactor` is 0, so it only **pans**, with the default `equalpower` model. Unity computes distance attenuation and volume itself and writes the result to the gain node.
- `_JS_Sound_Play` calls `_JS_Sound_Stop` first, so `AudioSource.Play` restarts the source and does not layer sounds.
- The listener orientation is passed as `_JS_Sound_SetListenerOrientation(-forward, up)`. Web Audio "right" is therefore Unity's +X.
- Clips are decoded by the browser (`decodeAudioData`), which drops the AAC encoder delay given by the MP4 container. The client does the same.

## Distance attenuation (**inferred**, Unity's documented curves)

- Logarithmic rolloff: `min(1, minDistance / d)`, held constant beyond `maxDistance`.
- Linear rolloff: `(maxDistance - d) / (maxDistance - minDistance)`, clamped to 0..1.
- The result is multiplied by `volume`.

The custom curve (`rolloffMode` 2) is not used by any source the client ports. The exact formula Unity's native audio uses has not been decompiled (see `docs/unknowns.md`).

## Sources ported

| Sound | Source | Trigger |
|-------|--------|---------|
| Weapon take-in | `WeaponManager` GameObject (`takeInAudio`) | `TakeFirstWeapon` / switching (`docs/gameplay/weapons.md`) |
| Fire / reload | the weapon GameObject's AudioSource | `WeaponScript` (`fireSound`, `reloadSound`), set as `clip` and `Play()` |
| Footsteps | the player root's AudioSource | `FPSSoundController` (below) |
| Impacts, explosion | the spawned prefab's play-on-awake AudioSource | instantiation (`docs/gameplay/particles.md`) |
| Rocket in flight | the rocket prefab's AudioSource | instantiation; stops when the rocket is destroyed |
| Death | `german (ragdol)` play-on-awake `death.m4a` (2D) | death (`docs/gameplay/damage.md`) |

When a GameObject is deactivated or destroyed, its sources stop. This matches Unity.

## FPSSoundController (`f28852`..`f28855`)

- `Awake`: `StepLenght = walkStepLength`. The ctor sets `lastStep = -10`.
- `FixedUpdate`, unless prone:
  - walking && grounded && !crouch: `PlayStepSounds()`, then `StepLenght = walkStepLength`;
  - running && grounded: `PlayStepSounds()`, then `StepLenght = runStepLenght`;
  - walking && crouch && grounded: `PlayStepSounds()`, then `StepLenght = crouchStepLenght`.

  Each check runs independently, so the step length used is always the previous mode's.
- `PlayStepSounds`: if `StepLenght != 0` and `Time.time > StepLenght + lastStep`, set `clip = walkSounds[Random.Range(0, count)]`, call `Play()`, and set `lastStep = Time.time`.

The client runs it inside the fixed-step loop with the fixed clock, which is what `Time.time` returns inside `FixedUpdate`.

## Screenshots and tests

`--screenshot` runs and `cargo test` use a silent mixer: no output device is opened. Without an output device the client runs silently.
