//! Objects the weapons instantiate from `Resources` prefabs (spec: `docs/gameplay/weapons.md`):
//! `Bullet` (raycast-stepped bullets and the knife's damage probe) and `Projectile` (the
//! launcher's rocket, a continuous-collision `Rigidbody`). Impact and explosion prefabs are
//! reported as events and played by `effects.rs`, which also draws the trails.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use glam::{Mat4, Quat, Vec3};
use serde_json::Value;

use crate::anim::ClipLibrary;
use crate::assets::{f32_of, ref_of, tag_name};
use crate::physics::{Capsule, CollisionWorld, InitialOverlap, QueryFilter};
use crate::prefab::{PrefabInstance, resource_prefab_index};
use crate::unity::{from_to_rotation, look_rotation};

/// `Bullet` fields.
#[derive(Clone, Debug)]
pub struct BulletFields {
    pub speed: f32,
    pub life: f32,
    pub damage: i32,
    /// Prefab refs; the one whose GameObject name equals the hit collider's tag is spawned.
    pub impact_objects: Vec<String>,
}

/// `Projectile` fields and the rocket's `SphereCollider` (local).
#[derive(Clone, Debug)]
pub struct ProjectileFields {
    pub explosion: Option<String>,
    pub destroy_delay: f32,
    pub time_out: f32,
    pub radius: f32,
    pub center: Vec3,
}

#[derive(Clone, Debug)]
pub enum Behaviour {
    Bullet(BulletFields),
    Projectile(ProjectileFields),
    /// Nothing ported (particle effects).
    None,
}

/// A `Resources` prefab the weapons spawn.
pub struct Template {
    pub path: String,
    /// Root GameObject name (`impactObjects` are matched by it).
    pub name: String,
    pub prefab: PrefabInstance,
    pub behaviour: Behaviour,
}

/// Every prefab a weapon field refers to, loaded once.
pub struct Templates {
    pub list: Vec<Template>,
    by_ref: HashMap<String, usize>,
    /// TagManager custom tags (for the impact match by tag name).
    pub custom_tags: Vec<String>,
}

fn bullet_fields(f: &Value) -> BulletFields {
    BulletFields {
        speed: f32_of(&f["speed"]),
        life: f32_of(&f["life"]),
        damage: f["damage"].as_i64().unwrap_or(0) as i32,
        impact_objects: f["impactObjects"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| ref_of(r).map(|(r, _)| r.to_string()))
            .collect(),
    }
}

impl Templates {
    /// Loads the prefabs of `refs` and, recursively, the prefabs those refer to (impacts,
    /// explosions).
    pub fn load(
        assets: &Path,
        refs: &[String],
        custom_tags: Vec<String>,
        clips: &mut ClipLibrary,
    ) -> Result<Templates> {
        let index = resource_prefab_index(assets)?;
        let mut templates = Templates {
            list: Vec::new(),
            by_ref: HashMap::new(),
            custom_tags,
        };
        let mut by_path: HashMap<String, usize> = HashMap::new();
        let mut pending: Vec<String> = refs.to_vec();
        while let Some(reference) = pending.pop() {
            if templates.by_ref.contains_key(&reference) {
                continue;
            }
            let Some(path) = index.get(&reference) else {
                continue;
            };
            let slot = match by_path.get(path) {
                Some(&slot) => slot,
                None => {
                    let mut prefab = PrefabInstance::load_file(assets, path, clips)?;
                    prefab.update_world();
                    let behaviour = if let Some(f) = prefab.script_fields(0, "Bullet") {
                        Behaviour::Bullet(bullet_fields(f))
                    } else if let Some(f) = prefab.script_fields(0, "Projectile") {
                        let sphere = prefab.component(0, "SphereCollider");
                        Behaviour::Projectile(ProjectileFields {
                            explosion: ref_of(&f["explosion"]).map(|(r, _)| r.to_string()),
                            destroy_delay: f32_of(&f["destroyDelay"]),
                            time_out: f32_of(&f["timeOut"]),
                            radius: sphere.map_or(0.5, |s| f32_of(&s["m_Radius"])),
                            center: sphere.map_or(Vec3::ZERO, |s| {
                                let c = &s["m_Center"];
                                Vec3::new(f32_of(&c["x"]), f32_of(&c["y"]), f32_of(&c["z"]))
                            }),
                        })
                    } else {
                        Behaviour::None
                    };
                    match &behaviour {
                        Behaviour::Bullet(b) => pending.extend(b.impact_objects.iter().cloned()),
                        Behaviour::Projectile(p) => pending.extend(p.explosion.iter().cloned()),
                        Behaviour::None => {}
                    }
                    let slot = templates.list.len();
                    templates.list.push(Template {
                        path: path.clone(),
                        name: prefab.nodes[0].name.clone(),
                        prefab,
                        behaviour,
                    });
                    by_path.insert(path.clone(), slot);
                    slot
                }
            };
            templates.by_ref.insert(reference, slot);
        }
        Ok(templates)
    }

    pub fn find(&self, reference: &str) -> Option<usize> {
        self.by_ref.get(reference).copied()
    }
}

/// Something a spawned object did.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectileEvent {
    /// `Instantiate` of an impact or explosion prefab (played by `effects.rs`).
    Effect {
        prefab: String,
        position: Vec3,
        rotation: Quat,
    },
    /// `SendMessageUpwards("ApplyDamage", damage)` on the hit collider. Only `HitBox` (on remote
    /// players) receives it, so locally nothing does.
    Damage { amount: i32, point: Vec3, tag: u32 },
}

#[derive(Clone, Debug)]
enum State {
    Bullet {
        started: bool,
        do_damage: bool,
        velocity: Vec3,
        new_pos: Vec3,
        old_pos: Vec3,
        has_hit: bool,
    },
    Projectile {
        started: bool,
        velocity: Vec3,
        kill_at: f32,
        killed: bool,
    },
    Inert,
}

/// One instantiated prefab.
pub struct Spawned {
    /// Unique per instantiation (for what is attached to it: trail, sound).
    pub id: u64,
    pub template: usize,
    pub position: Vec3,
    pub rotation: Quat,
    state: State,
    /// `Destroy(gameObject, t)`: removed at the end of the first frame with `time >= t`.
    destroy_at: f32,
}

#[derive(Default)]
pub struct Projectiles {
    pub objects: Vec<Spawned>,
    pub events: Vec<ProjectileEvent>,
    next_id: u64,
}

impl Projectiles {
    /// `Object.Instantiate(template, position, rotation)`; `velocity` is set on the
    /// `Rigidbody` right after (projectiles).
    pub fn spawn(
        &mut self,
        templates: &Templates,
        template: usize,
        position: Vec3,
        rotation: Quat,
        velocity: Option<Vec3>,
    ) {
        let state = match &templates.list[template].behaviour {
            Behaviour::Bullet(_) => State::Bullet {
                started: false,
                // The weapons set `doDamage = true` on every bullet they spawn.
                do_damage: true,
                velocity: Vec3::ZERO,
                new_pos: position,
                old_pos: position,
                has_hit: false,
            },
            Behaviour::Projectile(_) => State::Projectile {
                started: false,
                velocity: velocity.unwrap_or(Vec3::ZERO),
                kill_at: f32::INFINITY,
                killed: false,
            },
            Behaviour::None => State::Inert,
        };
        self.next_id += 1;
        self.objects.push(Spawned {
            id: self.next_id,
            template,
            position,
            rotation,
            state,
            destroy_at: f32::INFINITY,
        });
    }

    /// One `FixedUpdate` + physics step for the projectiles: `Projectile.FixedUpdate` faces the
    /// velocity, then the `Rigidbody` moves with continuous collision detection and
    /// `OnCollisionEnter` kills the projectile at the contact.
    pub fn fixed_update(
        &mut self,
        templates: &Templates,
        world: &CollisionWorld,
        dt: f32,
        time: f32,
    ) {
        for object in &mut self.objects {
            let Behaviour::Projectile(fields) = &templates.list[object.template].behaviour else {
                continue;
            };
            let State::Projectile {
                started,
                velocity,
                killed,
                ..
            } = &mut object.state
            else {
                continue;
            };
            if !*started || *killed {
                continue;
            }
            if let Some(rotation) = look_rotation(*velocity, Vec3::Y) {
                object.rotation = rotation;
            }
            let scale = templates.list[object.template].prefab.nodes[0].local.scale;
            let radius = fields.radius * scale.max_element();
            let center = object.position + object.rotation * (fields.center * scale);
            let distance = velocity.length() * dt;
            let direction = velocity.normalize_or_zero();
            if direction == Vec3::ZERO {
                continue;
            }
            let sphere = Capsule {
                p0: center,
                p1: center,
                radius,
            };
            // Rigidbody contacts ignore triggers.
            let hit = world.sweep_capsule(
                &sphere,
                direction,
                distance,
                QueryFilter::CONTROLLER,
                InitialOverlap::Block,
            );
            match hit {
                Some(hit) => {
                    object.position += direction * hit.distance;
                    // OnCollisionEnter: face the contact normal, then Kill.
                    object.rotation = from_to_rotation(Vec3::Y, hit.normal);
                    *killed = true;
                    kill(object, fields, templates, &mut self.events, time);
                }
                None => object.position += direction * distance,
            }
        }
    }

    /// `Start` of the objects spawned last frame, then `Update` (`Bullet`) and the `Invoke`d
    /// `Projectile.Kill` time-outs.
    pub fn update(&mut self, templates: &Templates, world: &CollisionWorld, dt: f32, time: f32) {
        for object in &mut self.objects {
            let template = &templates.list[object.template];
            match (&template.behaviour, &mut object.state) {
                (
                    Behaviour::Bullet(fields),
                    State::Bullet {
                        started,
                        do_damage,
                        velocity,
                        new_pos,
                        old_pos,
                        has_hit,
                    },
                ) => {
                    if !*started {
                        // Start: newPos = oldPos = position, velocity = speed · forward,
                        // Destroy(gameObject, life).
                        *started = true;
                        *new_pos = object.position;
                        *old_pos = object.position;
                        *velocity = fields.speed * (object.rotation * Vec3::Z);
                        object.destroy_at = time + fields.life;
                    }
                    if *has_hit {
                        continue;
                    }
                    *new_pos += *velocity * dt;
                    let direction = *new_pos - *old_pos;
                    if let Some(hit) = world.raycast(*old_pos, direction, direction.length()) {
                        *new_pos = hit.point;
                        *has_hit = true;
                        let rotation = from_to_rotation(Vec3::Y, hit.normal);
                        let tag = tag_name(hit.tag, &templates.custom_tags);
                        for impact in &fields.impact_objects {
                            let Some(i) = templates.find(impact) else {
                                continue;
                            };
                            if templates.list[i].name == tag {
                                self.events.push(ProjectileEvent::Effect {
                                    prefab: templates.list[i].path.clone(),
                                    position: hit.point,
                                    rotation,
                                });
                            }
                        }
                        object.destroy_at = time;
                        if tag != "Player" && *do_damage {
                            self.events.push(ProjectileEvent::Damage {
                                amount: fields.damage,
                                point: hit.point,
                                tag: hit.tag,
                            });
                        }
                    }
                    *old_pos = object.position;
                    object.position = *new_pos;
                }
                (
                    Behaviour::Projectile(fields),
                    State::Projectile {
                        started,
                        kill_at,
                        killed,
                        ..
                    },
                ) => {
                    if !*started {
                        // Start: Invoke("Kill", timeOut).
                        *started = true;
                        *kill_at = time + fields.time_out;
                    } else if !*killed && time >= *kill_at {
                        *killed = true;
                        kill(object, fields, templates, &mut self.events, time);
                    }
                }
                _ => {}
            }
        }
    }

    /// End of frame: `Destroy`ed objects disappear before rendering.
    pub fn end_frame(&mut self, time: f32) {
        self.objects.retain(|o| time < o.destroy_at);
    }

    /// World matrices of the live objects: (template, root matrix with the prefab's scale).
    pub fn instances<'a>(
        &'a self,
        templates: &'a Templates,
    ) -> impl Iterator<Item = (usize, Mat4)> + 'a {
        self.objects.iter().map(move |o| {
            let scale = templates.list[o.template].prefab.nodes[0].local.scale;
            (
                o.template,
                Mat4::from_scale_rotation_translation(scale, o.rotation, o.position),
            )
        })
    }
}

/// `Projectile.Kill`: instantiate the explosion, destroy after `destroyDelay`.
fn kill(
    object: &mut Spawned,
    fields: &ProjectileFields,
    templates: &Templates,
    events: &mut Vec<ProjectileEvent>,
    time: f32,
) {
    if let Some(explosion) = fields.explosion.as_ref().and_then(|r| templates.find(r)) {
        events.push(ProjectileEvent::Effect {
            prefab: templates.list[explosion].path.clone(),
            position: object.position,
            rotation: object.rotation,
        });
    }
    object.destroy_at = time + fields.destroy_delay;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{find_assets_dir, load_tags};

    const DT: f32 = 1.0 / 60.0;
    const PISTOL_BULLET: &str = "resources.assets:4166";
    const ROCKET: &str = "resources.assets:11042";

    fn templates() -> Templates {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        let tags = load_tags(&assets).unwrap();
        let refs = [PISTOL_BULLET.to_string(), ROCKET.to_string()];
        Templates::load(&assets, &refs, tags, &mut clips).unwrap()
    }

    /// A 'Metal'-tagged wall whose front face is the plane z = 10.
    fn wall() -> CollisionWorld {
        let mut world = CollisionWorld::new();
        world.collider_tag = 20000;
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.0, 10.5),
            Vec3::new(20.0, 20.0, 1.0),
            false,
            0,
        );
        world
    }

    #[test]
    fn templates_follow_impacts_and_explosions() {
        let t = templates();
        let names: Vec<_> = t.list.iter().map(|t| t.name.as_str()).collect();
        for name in [
            "Bullet",
            "Roket cube",
            "Metal",
            "Blood",
            "WFXMR_Explosion 1",
        ] {
            assert!(names.contains(&name), "{name} in {names:?}");
        }
        let Behaviour::Bullet(b) = &t.list[t.find(PISTOL_BULLET).unwrap()].behaviour else {
            panic!("pistol bullet");
        };
        assert_eq!((b.speed, b.life, b.damage), (30.0, 10.0, 40));
    }

    #[test]
    fn bullet_hits_wall_and_spawns_matching_impact() {
        let t = templates();
        let world = wall();
        let mut p = Projectiles::default();
        let bullet = t.find(PISTOL_BULLET).unwrap();
        p.spawn(&t, bullet, Vec3::ZERO, Quat::IDENTITY, None);
        let mut time = 0.0;
        let mut frames = 0;
        while !p.objects.is_empty() {
            time += DT;
            frames += 1;
            p.update(&t, &world, DT, time);
            p.end_frame(time);
            assert!(frames < 100);
        }
        // 0.5 m per frame: the ray (two frames long) reaches z = 10 on frame 20.
        assert!((20..=21).contains(&frames), "{frames}");
        let metal = t.list.iter().find(|t| t.name == "Metal").unwrap();
        let effect = p
            .events
            .iter()
            .find_map(|e| match e {
                ProjectileEvent::Effect {
                    prefab, position, ..
                } => Some((prefab.clone(), *position)),
                _ => None,
            })
            .unwrap();
        assert_eq!(effect.0, metal.path);
        assert!((effect.1.z - 10.0).abs() < 1e-4);
        assert!(p.events.iter().any(|e| matches!(
            e,
            ProjectileEvent::Damage {
                amount: 40,
                tag: 20000,
                ..
            }
        )));
    }

    #[test]
    fn bullet_expires_after_life() {
        let t = templates();
        let world = CollisionWorld::new();
        let mut p = Projectiles::default();
        p.spawn(
            &t,
            t.find(PISTOL_BULLET).unwrap(),
            Vec3::ZERO,
            Quat::IDENTITY,
            None,
        );
        let mut time = 0.0;
        while time < 9.9 {
            time += 0.1;
            p.update(&t, &world, 0.1, time);
            p.end_frame(time);
        }
        assert_eq!(p.objects.len(), 1);
        for _ in 0..3 {
            time += 0.1;
            p.update(&t, &world, 0.1, time);
            p.end_frame(time);
        }
        assert!(p.objects.is_empty() && p.events.is_empty());
    }

    #[test]
    fn rocket_explodes_on_contact() {
        let t = templates();
        let world = wall();
        let mut p = Projectiles::default();
        let rocket = t.find(ROCKET).unwrap();
        p.spawn(&t, rocket, Vec3::ZERO, Quat::IDENTITY, Some(Vec3::Z * 20.0));
        let mut time = 0.0;
        p.update(&t, &world, DT, time);
        let mut steps = 0;
        while !p.objects.is_empty() {
            time += 0.02;
            steps += 1;
            p.fixed_update(&t, &world, 0.02, time);
            p.end_frame(time);
            assert!(steps < 100);
        }
        let Some(ProjectileEvent::Effect {
            prefab, position, ..
        }) = p.events.first()
        else {
            panic!("{:?}", p.events);
        };
        assert!(prefab.ends_with("WFXMR_Explosion 1__4274.json"), "{prefab}");
        // Sphere radius 0.79 · 0.7244, centre 0.04 · 0.7244 ahead of the root.
        let expected = 10.0 - 0.79 * 0.724_400_4 - 0.04 * 0.724_400_9;
        assert!((position.z - expected).abs() < 1e-3, "{position}");
    }

    #[test]
    fn rocket_times_out() {
        let t = templates();
        let world = CollisionWorld::new();
        let mut p = Projectiles::default();
        p.spawn(
            &t,
            t.find(ROCKET).unwrap(),
            Vec3::ZERO,
            Quat::IDENTITY,
            Some(Vec3::Z),
        );
        let mut time = 0.0;
        p.update(&t, &world, DT, time);
        while time < 20.05 {
            time += 0.1;
            p.update(&t, &world, 0.1, time);
            p.end_frame(time);
        }
        assert!(p.objects.is_empty());
        assert_eq!(p.events.len(), 1);
    }
}
