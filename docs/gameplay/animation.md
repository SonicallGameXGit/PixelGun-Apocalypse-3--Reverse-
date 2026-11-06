# Animation and skinning

Everything the player prefabs animate goes through Unity's **legacy** `Animation` component. Code: `crates/pga3-client/src/anim.rs` (clips + `Animation`), `prefab.rs` (prefab instancing), `skin.rs` + `render.rs` (skinning), `character.rs` (script ports), `primitives.rs` (Unity built-in meshes).

Evidence references:

- `fNNNN` is a wasm function index in the original `pga3-v18.wasm`. It can be read with `tools/re/wasm_func.py`, and decompiled copies are in `research/notes/wasm/`: `characteranimation.c`, `weaponanim.c`, `playernet.c`.
- Native Unity functions have no IL2CPP name. They were found through the icall table in `f62929`, which maps each `UnityEngine.Animation::*` icall string to a function pointer.

## Clips (observed)

- Every gameplay clip is legacy (`m_Legacy: true`), with data in `m_RotationCurves`, `m_PositionCurves` and `m_ScaleCurves`, keyed by a transform path relative to the `Animation`'s GameObject. Source: `assets/data/animations/AnimationClip/*.json`.
- All 21,565 keys in the build have `weightedMode` 0 and finite slopes; pre/post infinity is Clamp (2). There are no animation events.
- Evaluation is a cubic Hermite between keys `k0` and `k1`, with `dx = t1 - t0`, `s = (t - t0) / dx`, `m0 = outSlope0 · dx` and `m1 = inSlope1 · dx`:
  `v = (2s³-3s²+1)·v0 + (s³-2s²+s)·m0 + (s³-s²)·m1 + (-2s³+3s²)·v1`.
  Outside the key range the value is clamped. Rotation results are normalised.
- Clip range = (first key time, last key time) over all curves. `m_WrapMode` is the clip's default wrap mode.

## `Animation` component: native semantics (observed, decompiled)

Call chain: `CrossFade` (icall pointer 50) → `f1895` → `f1894` → `f1685`. `Play` → `f1891` → `f1892`, which is `CrossFade(name, 0)` with `PlayMode.StopSameLayer`.

`AnimationState` fields (byte offsets in the native struct):

| Offset | Field |
|--------|-------|
| +16 | weight |
| +20 | wrapped time (f32) |
| +24 | time (f64) |
| +32 | last update |
| +40 | layer |
| +44 | speed |
| +52 | stop time |
| +56 | fade length |
| +60 | fade target |
| +76 | wrap mode |
| +84 | fade speed |
| +96 / +100 | range |

Flags: 1 = fading, 2 = enabled, 4 = stop when faded, 32 = automatic fade-out started.

**CrossFade(name, fadeLength)** (`f1685`):

- *Other states in the same layer:*
  - with `fadeLength ≤ 0.001`, they stop immediately (time 0, weight 0);
  - otherwise they fade to weight 0 at speed `(0 - w) / fadeLength`, flagged stop-when-faded.
- *Target state:*
  - becomes enabled with `last_update = now`. Its time is **not** reset; that is what `Rewind` is for.
  - It fades to 1 at `(1 - w) / fadeLength`. If it was already fading to the same target faster, the faster speed is kept.
  - A speed ≤ 1e-4 becomes 100000, so a fade of 0 is instant.

**Per-frame update** (`f1694` + `f1730`; runs between `Update` and `LateUpdate`). For each enabled state:

- *Time:* `time += speed · (now - last_update)`, accumulated as a double.
- *Wrapped time* (`f1670`):
  - Once and ClampForever clamp to the range;
  - Loop wraps with `fmod`;
  - PingPong mirrors;
  - Default passes the time through.
- *Fading:* the weight moves towards the fade target and stops there. A state flagged stop-when-faded is disabled at weight 0.
- *Once/Default auto-stop:* `fade_length` seconds before the end of the range, the state starts fading out (flag 32). With fade 0 it stops at the end (time 0, weight 0). When playing backwards, the start of the range counts as the end.

**`Animation.wrapMode` setter** (`f1886`): sets every existing state's wrap mode, not just the default.

**Blending (inferred):** the per-property mechanics were not decoded.

- Weights are normalised per animated property.
- Quaternions are sign-aligned before summing, then normalised.
- Properties no enabled state animates keep their current value.

These rules reproduce single-state playback and cross-fades exactly. Multi-layer additive cases do not occur in the ported scripts.

**Not ported:** `m_CullingType` (BasedOnRenderers); the client always animates.

## Skinning (observed)

- **Bones per vertex:** `QualitySettings` level 3 "Good" (the current level) has `blendWeights = 1`. Every vertex therefore uses only its first, largest influence at weight 1; `skin::limit_weights` applies the rule.
- **Skin data:** `tools/extract/export_skins.py` writes bind poses, bone name hashes and per-vertex indices/weights to `assets/data/skins/<model>.json` for the 6 skinned meshes.
- **Bones:** `m_Bones` of the `SkinnedMeshRenderer` (in the prefab JSON) gives the bone transforms.
- **GPU skinning:** the palette is `bone.localToWorld · bindpose[i]`, up to 64 bones (the meshes use at most 60). The skinned vertex is already in world space, so the renderer's own transform is not applied, as in Unity.

## Built-in meshes (observed)

The weapons and the characters' gear are built from Unity's built-in primitives, referenced as `unity_default_resources:<path_id>`:

| Primitive | path_id | References |
|-----------|---------|------------|
| Cube | 10202 | 1957 |
| Cylinder | 10206 | 296 |
| Plane | 102 | – |
| Sphere | 23 | – |

These meshes belong to Unity, so they are not published. `primitives.rs` regenerates Cube and Cylinder with Unity's vertex order. Its test compares them with the local raw extraction: Cube is exact, and the Cylinder UVs match within 1e-5. Plane and Sphere are not needed by the player prefabs yet.

## Script ports (observed, decompiled)

**`WeaponManager.Awake`** (`f29921`):

1. Activate every child. Each `WeaponAnimation.Awake` (`f29907`) runs `Play(Idle)`, then `animation.wrapMode = Once`.
2. Deactivate the children and every `allWeapons` entry again.
3. Call `TakeFirstWeapon(allWeapons[index])` (`f29922`): `SetActive(true)`, then `SendMessage("selectWeapon")`.

`WeaponScript.Awake` caches `GetComponentInChildren<WeaponAnimation>`, falling back to `SniperAnimation`. `selectWeapon` (`f29967`) then calls `takeIn`.

`WeaponAnimation` methods:

| Method | Function | Body |
|--------|----------|------|
| `takeIn` | `f29910` | `Rewind(TakeIn)`; `speed = TakeInOutSpeed`; `time = 0`; `Play(TakeIn)` |
| `takeOut` | `f29911` | same as `takeIn`, with `TakeOut` |
| `Fire` | `f29908` | `Rewind(Shoot)`; `speed = FireAnimationSpeed`; `Play(Shoot)` |
| `Reloading(t)` | `f29909` | `Stop(Reload)`; `speed = clip length / t`; `Rewind`; `Play` |

`Fire`, `Reloading` and `takeOut` are ported but not called yet (weapons are not implemented).

For `german`, `index` = 0 is the Machete. Its TakeIn clip lasts 0.933 s at speed 1.3 and is nearly static (a few millimetres of arm motion).

**`CharacterAnimation`** (`f65552` Start, `f65553` configureAnimations, `f65555` LateUpdate):

- **Start / configureAnimations:** every clip slot gets `WrapMode.Loop`, except `jumpPose`, `pistolIdle`, `knifeIdle` and `gunIdle`.
- **LateUpdate, action:** crouch → Crouch; prone without crouch → Prone; otherwise Stand.
- **LateUpdate, per action:** every choice below is `CrossFade(clip, 0.2)`.
  - **Stand:**
    1. not grounded → `jumpPose`;
    2. not walking → `stayIdle`;
    3. running → `runFront` while W is held;
    4. walking → W `walkFront`; A && !S `walkLeft`; D && !S `walkRight`; S `walkBack`.
  - **Crouch:** the same pattern with the `crouch*` clips.
  - **Prone:** idle cross-fades the **`reload`** slot, as in the original. The R key also cross-fades `reload` but records `proneBack` as the sync name.
- **Inputs:**
  - The keys are `CF2Input.GetKey(KeyCode.W/A/S/D/R)`: letter keys only, not the arrows.
  - The flags are `FPScontroller` fields: Running 0x19, Walking 0x1A, crouch 0x38, prone 0x46, grounded 0x5C.

`ThirdPersonWeaponControl` (`f65554`) is not ported.

**`PlayerNetworkController.Awake`** (`f37893`):

- `localObjectsToDeactivate` = Base_mesh, Root_jnt, Terrorist_02_mesh: the owner's body is invisible, and only the first-person rig under `Weapon Camera` is seen.
- `remoteObjectsToDeactivate` = LookObject, SpawnObject, Base_mesh.

## Cameras (observed, `german` prefab)

| Camera | FOV | near | far | depth | clear | culling mask |
|--------|-----|------|-----|-------|-------|--------------|
| Main Camera | 65 | 0.2 | 1000 | 1 | Skybox | 309 (not layer 9) |
| Weapon Camera | 50 | 0.01 | 2 | 2 | Depth only | 512 (layer 9 only) |

The Weapon Camera's local transform is identity under the Main Camera. The client draws the scene and prefab with the Main Camera, then clears depth and draws layer 9 with the Weapon Camera.

## Client frame order

1. `begin_frame`: the animation clock advances.
2. FixedUpdate and Update of the player.
3. `Animation` update and sampling.
4. `FPScontroller` LateUpdate.
5. `CharacterAnimation.LateUpdate`.
6. Root and camera transforms are applied, then world matrices are computed.

**CharacterAnimation timing (inferred):** its CrossFade calls in LateUpdate take effect at the next animation update, as in Unity.

Fly mode keeps simulating the player with no input, so the third-person body seen from the fly camera stands at its last position and plays `stayIdle`.

## Open points

These are tracked in `docs/unknowns.md`:

- Which team body mesh a remote player shows.
- The exact blending normalisation for weights summing to less than 1.
- `HeadLookController`, `WalkSway`, `WeaponScript` sway/aim, and remote animation playback (`PlayerNetworkController`).
- Particle shaders: prefab renderers using them are skipped for now.
