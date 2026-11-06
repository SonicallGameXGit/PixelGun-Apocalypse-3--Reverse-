# Weapons

First-person weapons of the local player: `WeaponManager` (selection), `WeaponScript` (aiming, firing, reloading, recoil), `WeaponCrosshair`, `SniperScope`, and what the weapons spawn: `Bullet`, `Projectile` (rocket). Ports: `crates/pga3-client/src/weapons.rs` and `projectiles.rs`. Unity math helpers (`Quaternion.Euler/LookRotation/Slerp`, `Random`) are in `unity.rs`.

Conventions:

- `fNNNN` is a wasm function index in the original `pga3-v18.wasm`. Decompiled copies are in `research/notes/wasm/`: `w_weaponmanager.c`, `w_weaponscript.c`, `w_bullet.c`, `w_projectile.c`, `w_weaponcrosshair.c`, `w_sniperscope.c`.
- Field offsets are from `research/il2cpp-dump/dump.cs`.
- Values are from `assets/data/prefabs/resources/german.json` (the `german` player prefab) and `assets/data/prefabs/resources.assets/*.json` (spawned prefabs).
- Everything here is **observed** in the decompiled code unless marked **inferred** or **unknown**.

## Data

`WeaponManager` sits at `German/LookObject/Main Camera/Weapon Camera/WeaponManager`, with `index` 0 and `SwitchTime` 0.4. Its `allWeapons` are its six children:

| Slot | Weapon | GunType | singleFire | Rate | Clip / clips | Reload | Spawns |
|---|---|---|---|---|---|---|---|
| 0 | Machete | 3 Knife | yes | knife 0.2 (delay 0.07) | none | none | `KnifeDamage__383` |
| 1 | Pistol | 0 MachineGun | no | 0.2 | 10 / 60 | 1.3 | `Bullet__4166` |
| 2 | M242 | 0 MachineGun | no | 0.1 | 32 / 128 | 2.3 | `Bullet_fn58t__5073` |
| 3 | ShotGun | 2 ShotGun | yes | 1.2 | 10 / 100, 4 fractions | 0.8 | `Bullet__5072` |
| 4 | Sniper | 0 MachineGun | no | 1.6 | 5 / 55 | 3.0 | `Bullet Sniper__4167` |
| 5 | Launcher | 1 GrenadeLauncher | yes | shotDelay 0 | ammoCount 1 | none | `Roket cube__11042` |

Other per-weapon values (aim position and FOV, error angles, recoil, sway, sounds) are in the prefab JSON. The table only summarises them.

Weapon fields refer to prefabs by GameObject or **component** ref. For example, Machete's `knife.bullet` is `resources.assets:16268`, the `Transform` of `KnifeDamage`. `prefab::resource_prefab_index` maps every node and component ref of `prefabs/resources.assets/` to its file.

Spawned prefabs:

- **Bullets** are the built-in Cube at root scale 0.2. Their `BoxCollider` is disabled (`m_Enabled: false`), so bullets never collide with each other.
  - `Bullet` fields: speed, life, damage, `impactObjects` = [`Blood__356`, `Metal__357`/`__368`].
  - Pistol: 30 m/s, 40 dmg. M242: 35, 30. ShotGun: 35, 35. Sniper: 45, 300. All live 10 s.
  - `KnifeDamage` has no mesh: 25 m/s, life 0.15 s, damage 300.
- **Rocket** (`Roket cube`): Cube mesh at scale 0.7244.
  - `SphereCollider` radius 0.79, centre (0, 0, 0.04).
  - `Rigidbody`: mass 1, no gravity, `CollisionDetection` 1 (continuous).
  - `TrailRenderer`; inactive child `Sphere`.
  - `Projectile`: explosion `WFXMR_Explosion 1__4274`, `destroyDelay` 0, `timeOut` 20.
- **Impacts and explosion** are `ParticleSystem`s. Impacts have `WaitForDestroy` (0.3-3 s per prefab, `docs/gameplay/particles.md`). The explosion has `ExplosionDamage` (radius 6, power 15, damage 100).

## WeaponManager

**`Awake`** (`f29921`):

1. Activate every child, which runs `WeaponAnimation.Awake` and `WeaponScript.Awake`, the latter caching `GetComponentInChildren<WeaponAnimation>`.
2. Deactivate the children again.
3. `TakeFirstWeapon(allWeapons[index])` (`f29922`): play `takeInAudio`, `SetActive(true)`, `SendMessage("selectWeapon")`, then `canSwitch = true`.
4. Set every weapon's `autoFire = !manager.autoFire`. The constructor (`f29926`) sets the manager's `autoFire = true` and `textDelay = 1`, so the weapons start with `false` and "Auto fire on" shows for the first second (`docs/gameplay/hud.md`).

**`Update`** (`f29923`):

1. **O** (`GetKeyDown`): toggle `autoFire`, copy it to every weapon, set `textDelay = 1`.
2. Set `SelectedWeapon = allWeapons[index]`.
3. With more than one weapon, check the slots:
   - **Key `1`**, or `changewep == 0`: switch to slot 0 if `canSwitch` and `index != 0`.
   - **Keys `2`..`6`** (slots 1..5), or `changewep == slot`: needs `canSwitch`.
     - If the room name lacks `[WEP0n]` (n = slot + 1), set `shownowepinroom = 40` (a "weapon not allowed" message drawn by `WeaponManager.OnGUI`).
     - Otherwise switch if `index != slot`.
     - `allWeapons[slot]` past the end throws, which ends `Update`.
   - **Key `0`**, or `changewep == 9`: needs `canSwitch`, ten weapons, `[WEP10]`, `SuperWeapons` (the page URL contains "pacogames") and `index != 9`.
4. Set `changewep = -1`.
5. **Scroll up**, or **M** held (`GetKey`, so every frame), picks the next allowed slot into `changewep`:
   - `i2 = index`.
   - For i in 0..=8: if `i2 == i`, then either `changewep = i + 1` (when `[WEP(i+2)]` is present, `i != 8 || SuperWeapons`, and `changewep == -1`), or `i2 = i + 1`.
   - Then if `i2 == 9`: `changewep = 0` when `[WEP10]`, `SuperWeapons` and `changewep == -1`. Otherwise `i2 = 0; changewep = 0`.
6. **Scroll down**, or **N** held, picks the previous allowed slot:
   - Same pattern counting down: pass 1 for i in 9..=1, with the `SuperWeapons` check at i = 9.
   - Then the `i2 == 0` wrap: `changewep = 9`, else `i2 = 9`.
   - Pass 2 for i in 9..=2 without the super check.
   - Then `i2 == 1` → `[WEP01]`: `changewep = 0`, else `i2 = 0`.
7. `textDelay -= deltaTime`.

The chosen `changewep` is applied on the next frame by the slot checks in step 3. The token literals are `[WEP01]`..`[WEP09]` and `[WEP10]`. The room name is `RoomMultiplayerMenu.Awooledweops`.

**`SwitchWeapons(current, next)`** (`f29924`, coroutine `f29927`):

1. `canSwitch = false`. If `current.activeSelf`, send it `deselectWeapon`.
2. Wait `SwitchTime`.
3. Play `takeInAudio`, `current.SetActive(false)` (its coroutines stop), `next.SetActive(true)`, send `selectWeapon`, `canSwitch = true`.

`index` changes as soon as the coroutine starts.

`selectWeapon` (`f29967`): `canFire = true`, `canAim = (type != Knife)`, `aimed = false`, `WeaponAnimation.takeIn`.

`deselectWeapon` (`f29968`): `aimed = isReload = canFire = canAim = false`, `takeOut`.

## WeaponScript

**`Start`** (`f29936`) runs on the first frame the weapon is active, before any `Update` of that frame:

- Store `WalkSway.bobbingAmount` as the default.
- `camDefaultRotation` = Main Camera `localRotation`. `defaultFov` = `Camera.main.fieldOfView`. `defaultPosition` = `localPosition`.
- Per type:
  - MG: `bulletsLeft = bulletsPerClip`, muzzle flash inactive.
  - SG: `bulletsLeft = bulletsPerClip`, `smoke.Stop()`.
  - MG, GL and SG: `canAim = canFire = true`.
  - Knife: `canAim = false`, `canFire = true`.
- Initial field values:
  - `camPos` is an unassigned `Quaternion`, so (0,0,0,0).
  - `lastShot` = -10.

**`Update`** (`f29942`) runs in this order:

1. `RaycastForZombies` (`f29941`): `inSight` is true when a ray from the Main Camera along its forward axis (no distance limit) hits a collider whose tag is in `FPScontroller.tagsToShoot` (`["Blood"]`).
2. `Aiming` (`f29943`):
   - If `aimed && !running`, use `aimPosition`, `toFov`, `AimErrorAngle` and `aimBobbingAmount`.
   - Otherwise use the defaults and `NoAimErrorAngle`.
   - Set `WalkSway.bobbingAmount` to the chosen value.
   - `localPosition = Lerp(localPosition, pos, dt / smoothTime)`. FOV `= Mathf.Lerp(fov, target, dt / smoothTime)`.
3. `RotationRealism` and `SmoothMove`: see `docs/gameplay/animation.md`.
4. If `Recoil`, `cameraRecoilDo` (`f29946`): Main Camera `localRotation = Slerp(localRotation, camPos, dt · CameraRecoil.smooth)`.
   - While `camPos` is still the zero quaternion, the Slerp only scales the rotation and the `Transform` setter normalises it back.
5. Gun-type update:
   - MG (`f29947`): if `fire && !isReload`, `machineGunFire`. Otherwise `canRun = true` and the muzzle flash goes inactive. While reloading, `canAim = false`.
   - GL (`f29948`): like MG without the flash and `canAim` parts.
   - SG (`f29949`): like GL, plus `canAim = false` while reloading.
6. If `running`, `aimed = false`.
7. `InputUpdate` (`f29950`):
   - Right mouse down with `canAim && !running` toggles `aimed`.
   - `fire = (left held && canFire && !singleFire) || (autoFire && inSight)`.
   - Left mouse down with `canFire && !isReload && singleFire`:
     - MG → `machineGunFire`. On other frames the MG branch sets `canRun = true`.
     - GL → `grenadeLauncherFIre`. SG → `shotGunFire`. Knife → `StartCoroutine(knifeOneShot)`.
   - **R** down with `!isReload && machineGun.clips > 0` (the MG clip count is checked for both types):
     - MG with `bulletsLeft != bulletsPerClip` → `machineGunReload`.
     - SG with `ShotGun.bulletsLeft != ShotGun.bulletsPerClip` → `shotGunReload`.

**Fire loops**: `machineGunFire` (`f29951`), `shotGunFire` (`f29954`), and `grenadeLauncherFIre` (`f29953`, which also requires `canFire`).

```
if ammo != 0 {
    if time - rate > nextFireTime { nextFireTime = time - deltaTime }
    while nextFireTime < time && ammo != 0 {   // GL: ammoCount >= 1
        StartCoroutine(OneShot)
        nextFireTime += rate
    }
}
```

`rate` is `fireRate`, or `shotDelay` for the launcher. Shots fire at most every `rate` seconds, catching up on frames longer than `rate`.

**`GetCenterOfScreen`** (`f29969`): `Raycast(camera position, camera forward, 300)` returns the hit point, or `position + forward · 100`.

**`machineGunOneShot`** (`f61834`):

1. Save `firePoint.rotation`.
2. Set `firePoint.rotation = Euler(insideUnitSphere · errorAngle) · weapon.rotation`, then `LookAt(GetCenterOfScreen())`. The spread is overwritten, so MG bullets always fly at the screen centre; only the random draw remains.
3. Instantiate the bullet at the fire point (`doDamage = true`). Restore the rotation.
4. `lastShot = time`; `bulletsLeft--`; `fireSound`.
5. `machineGunMuzzleFlash` (`f61832`): flash `localRotation = AngleAxis(Random.Range(0, 359), Vector3.left)`, active, light enabled. After 0.04 s: inactive, light off.
6. Fire animation: when `aimed`, only if `Aim.playAnimation`; otherwise always.
7. If `Recoil`:
   - `mouseLook.rotationY += recoilPower`.
   - `machineGunCameraRecoil` (`f61830`): `camPos = Euler(Range(0, -shake), Range(-shake, shake), 0)`. After 0.05 s, `camPos = camDefaultRotation`.
8. If `clips > 0 && bulletsLeft == 0`: `noBullets = true`, wait 1 s, then `machineGunReload` unless already reloading.

**`machineGunReload`** (`f61836`):

1. `isReload = true`, `aimed = canAim = false`.
2. `WeaponAnimation.Reloading(reloadTime)`, `reloadSound`.
3. Wait `reloadTime`.
4. If `clips > 0`:
   - `need = perClip - left`.
   - If `clips > need`: `clips -= need; left += need`. Else: `left += clips; clips = 0`.
   - Then `noBullets = isReload = false`, `canAim = canRun = true`.

With `clips == 0`, `isReload` stays true. **Observed**; the original has the same dead end.

**`shotGunOneShot`** (`f61840`):

1. `firePoint.LookAt(center)` and save the rotation.
2. For each of the `fractions`: set `firePoint.rotation = Euler(insideUnitSphere · errorAngle) · weapon.rotation` and spawn a bullet. Pellets spread around the weapon's facing, not the screen centre.
3. Restore the rotation.
4. `lastShot`, `fireSound`, `bulletsLeft--`, animation.
5. Smoke `Play()`, then `Stop()` after 0.3 s.
6. If `Recoil`:
   - `shotGunCameraRecoil`: `camPos = Euler(Range(-1.5·shake, -shake), Range(shake/3, 0.5·shake), 0)`, reset after 0.1 s.
   - Then `rotationY += recoilPower`.
7. Empty clip: as for the MG, with `shotGunReload`.

`shotGunReload` (`f61842`) matches the MG reload on the `ShotGun` fields, but it never sets `canAim = false`.

**`grenadeLauncherOneShot`** (`f61823`):

- If `shotDelay > 0`: animation, recoil (`rotationY`, then the shotgun-style camera recoil), start `grenadeLauncherReload`, wait `shotDelay`.
- Then launch:
  - Instantiate the projectile at the fire point. No `LookAt`, so it uses the fire point's own rotation.
  - `isRemote = false`.
  - `Rigidbody.velocity = weapon.TransformDirection(0, 0, initialSpeed)`.
  - `IgnoreCollision` with the root's colliders.
  - `lastShot`, `ammoCount--`, `fireSound`.
- If `shotDelay == 0`: animation, recoil, and `grenadeLauncherReload` only while `ammoCount > 0`.
- The Launcher has `ammoCount` 1. Nothing refills it, so it fires one rocket per life.

`grenadeLauncherReload` (`f61826`):

1. `isReload = true`; wait `waitBeforeReload`.
2. `aimed = false`, `Reloading(reloadTime)`, `reloadSound`; wait `reloadTime`.
3. `isReload = false`.

**`knifeOneShot`** (`f61828`):

1. Only if `time > knife.fireRate + lastShot`.
2. `firePoint.LookAt(center)` (kept), `fireSound`, `WeaponAnimation.Fire`.
3. Wait `delayTime`.
4. Spawn `KnifeDamage` at the fire point (`doDamage = true`). `lastShot = time`.

## Bullet and Projectile

**`Bullet.Start`** (`f65509`):

- `newPos = oldPos = position`, `velocity = speed · forward`.
- `Destroy(gameObject, life)`.

**`Bullet.Update`** (`f65510`), while `!hasHit`:

1. `newPos += velocity · dt`.
2. `Raycast(oldPos, newPos - oldPos, |newPos - oldPos|)`. `oldPos` lags one frame, so each ray covers two frames of travel.
3. On a hit:
   - `newPos = hit.point`, `hasHit = true`.
   - The impact rotation is `FromToRotation(up, normal)`.
   - Push the hit `Rigidbody` (none on maps).
   - Instantiate every `impactObjects` prefab whose GameObject **name equals the hit collider's tag** (map colliders are tagged `Metal`).
   - `Destroy(gameObject)`.
   - If the tag is not `Player` and `doDamage`, `SendMessageUpwards("ApplyDamage", damage)`.
4. `oldPos = position`; `position = newPos`.

**`Projectile`**:

- `Start` (`f29626`): `Invoke("Kill", timeOut)`.
- `FixedUpdate` (`f29627`): `rotation = LookRotation(velocity)`.
- `OnCollisionEnter` (`f29628`): `rotation = FromToRotation(up, contact normal)`, then `Kill`.
- `Kill` (`f29631`): instantiate the explosion, destroy after `destroyDelay`.

The client sweeps the rocket's sphere (world radius 0.79 · 0.7244) through each fixed step and stops at the contact. Rigidbody contacts ignore triggers.

## Crosshair and scope

**`WeaponCrosshair`** sits on `WeaponManager`: texture `CrosshairTexture`, length 10, width 1, dynamic, `crosshairResponce` 50, `defaultDistance` 20, smooth 0.3.

`Awake` (`f29913`) creates a `GUIStyle` with `normal.background` = the texture and no border.

`Update` (`f29914`), with `r = crosshairResponce` and `d = defaultDistance`:

```
if dynamic {
    if ws && (mouseDown || ws.fire) {
        if !ws.singleFire {
            distance = lerp(distance, ws.fire && !ws.noBullets ? r·2 : d, dt/smooth)
        } else if mouseDown && ws.canFire && !ws.isReload && !ws.noBullets {
            if distance < r·4 { distance += r }
        } else {
            distance = lerp(distance, d, dt/smooth)
        }
    } else {
        distance = lerp(distance, walking ? r + |movement.velocity|·2 : d, dt/smooth)
    }
} else {
    distance = d
}
crosshair = !ws.aimed
```

`OnGUI` (`f29915`):

- Draws only when `distance <= (int)(Screen.height / 2)` and the crosshair is shown.
- `GUI.color = red` while `ws.inSight`.
- Four `GUI.Box`es with the style, as `(x, y, w, h)`:
  - left: `((W-dist)/2 - len, (H-wid)/2, len, wid)`
  - right: `((W+dist)/2, (H-wid)/2, len, wid)`
  - top: `((W-wid)/2, (H-dist)/2 - len, wid, len)`
  - bottom: `((W-wid)/2, (H+dist)/2, wid, len)`

**`SniperScope.OnGUI`** (`f29809`) is on the Sniper and the Launcher:

- While aimed: `GUI.DrawTexture(((int)(W/2) - H·1.8·0.5, H/2 - H/2, H·1.8, H), scopeTexture)` and `SetActive(false)` on `objectsToDeactivate` (the weapon model).
- Otherwise: `SetActive(true)` on them.

## Rendering

- The muzzle flash is `SpecialEffects/MuzzleFlash` (layer 9, Weapon Camera): mesh `models/Sphere.obj`, material `MuzzleFlash_1` (`Mobile/Particles/Additive`).
  - The compiled shader dump has no render state. The client uses the state of Unity's built-in shader source (**inferred**, not dumped): `Blend SrcAlpha One`, `Cull Off`, `ZWrite Off`, `Fog { Color (0,0,0,0) }`, `combine texture * primary`.
  - The mesh has no vertex colours, so `primary` is taken as white (**inferred**).
- The point light next to it is inactive with its `Light` disabled, so it is never seen.
- Spawned bullets and rockets are drawn with their templates' models at the instance transforms.
- `OnGUI` quads are alpha blended over the frame, with GUI pixels origin at the top left.

## Frame order in the client

1. FixedUpdate steps: player, then rocket sweeps.
2. Player `Update`.
3. Pending `Start`s of the spawned objects, then the `Bullet` updates and `Projectile` time-outs.
4. Pending `WeaponScript.Start`s, then `WeaponManager.Update`, then every active `WeaponScript.Update`, then `WeaponCrosshair.Update`.
5. Coroutines whose `WaitForSeconds` elapsed (`resume_at <= time`, oldest first).
6. Weapon spawns are instantiated. They start next frame, as objects instantiated during `Update` do.
7. Animations, then `LateUpdate`.
8. Destroyed objects are removed before rendering.

Coroutines of a weapon are dropped when its GameObject is deactivated.

## Not implemented

- Network sync (`WeaponSync`), third-person weapon visuals, and damage to other players (`HitBox` receivers exist only on remote players, `docs/gameplay/damage.md`).

Sounds, particles, trails, the explosion, `AmmoDisplay` and the `WeaponManager.OnGUI` texts are covered in `docs/gameplay/audio.md`, `docs/gameplay/particles.md` and `docs/gameplay/hud.md`.

Tests (`cargo test`) cover:

- Weapon data and refs.
- Sway.
- Switch timing (0.4 s) and room tokens.
- The scroll chain.
- Pistol fire rate, ammo and reload timing.
- Knife delay.
- The Launcher's single rocket.
- Crosshair geometry and the sniper scope.
- Bullet raycast and impact selection.
- Bullet life.
- Rocket contact and time-out.
- Raycast hit, miss and back-face behaviour.
