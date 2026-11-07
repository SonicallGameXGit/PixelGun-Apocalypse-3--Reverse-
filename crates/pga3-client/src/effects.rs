//! The visual/audio side of the prefabs the weapons spawn (spec: `docs/gameplay/particles.md`):
//! impact effects (`WaitForDestroy` + play-on-awake `AudioSource`), the rocket explosion
//! (`CFX_AutoDestructShuriken` + `ExplosionDamage`), and the trails / sounds attached to bullets
//! and rockets while they fly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{Mat4, Quat, Vec3};
use serde_json::Value;

use crate::assets::{f32_of, ref_of};
use crate::audio::{Audio, SourceSettings};
use crate::legacy_particles::{LegacyEmitterDef, LegacyParticles};
use crate::obj::{MeshData, parse_obj};
use crate::particles::{ParticleSystem, ParticleSystemDef};
use crate::physics::CollisionWorld;
use crate::prefab::PrefabInstance;
use crate::projectiles::{Projectiles, Templates};
use crate::render::{DynamicBatch, View};
use crate::trail::{Trail, TrailDef};

/// `ExplosionDamage` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplosionFields {
    pub radius: f32,
    pub power: f32,
    pub damage: f32,
    pub timeout: f32,
    pub is_remote: bool,
}

/// `ExplosionDamage.Start`: damage to a collider whose bounds' closest point is `distance` from
/// the explosion (sent to `HitBox`es, which only the other players' copies have).
pub fn explosion_damage(fields: &ExplosionFields, distance: f32) -> f32 {
    (1.0 - (distance / fields.radius).clamp(0.0, 1.0)) * fields.damage
}

/// What a template's GameObjects carry besides what `projectiles.rs` simulates.
pub struct EffectDef {
    /// (node, system), in hierarchy order (the root's first).
    pub systems: Vec<(usize, Arc<ParticleSystemDef>)>,
    /// (node, settings) of the enabled AudioSources.
    pub sources: Vec<(usize, SourceSettings)>,
    pub trail: Option<Arc<TrailDef>>,
    /// `WaitForDestroy.lifeTime`.
    pub wait_for_destroy: Option<f32>,
    /// `CFX_AutoDestructShuriken.OnlyDeactivate` (`Some` when the script is present).
    pub auto_destruct: Option<bool>,
    pub explosion: Option<ExplosionFields>,
    /// Node world matrices relative to the root's (`root⁻¹ · node`), for placing instances.
    relative: Vec<Mat4>,
    root_scale: Vec3,
}

/// Loads `models/*.obj` meshes for mesh particles (cached).
pub struct MeshCache {
    assets: PathBuf,
    meshes: HashMap<String, Option<Arc<MeshData>>>,
}

impl MeshCache {
    pub fn new(assets: &Path) -> MeshCache {
        MeshCache {
            assets: assets.to_path_buf(),
            meshes: HashMap::new(),
        }
    }

    pub fn get(&mut self, value: &Value) -> Option<Arc<MeshData>> {
        let path = ref_of(value)?.1?.to_string();
        if let Some(mesh) = self.meshes.get(&path) {
            return mesh.clone();
        }
        let mesh = std::fs::read_to_string(self.assets.join(&path))
            .ok()
            .and_then(|text| parse_obj(&text).ok())
            .map(Arc::new);
        if mesh.is_none() {
            eprintln!("particle mesh {path} not loaded");
        }
        self.meshes.insert(path, mesh.clone());
        mesh
    }
}

/// The `ParticleSystem` of a node (with its renderer), if any.
pub fn particle_system_def(
    prefab: &PrefabInstance,
    node: usize,
    meshes: &mut MeshCache,
) -> Option<ParticleSystemDef> {
    let ps = prefab.component(node, "ParticleSystem")?;
    let renderer = prefab.component(node, "ParticleSystemRenderer");
    Some(ParticleSystemDef::from_components(ps, renderer, |m| {
        meshes.get(m)
    }))
}

/// A particle emitter on a player-prefab node: a Shuriken `ParticleSystem` (Build-7) or a legacy
/// `ParticleEmitter` (the CrazyGames build's shotgun smoke).
#[derive(Clone, Debug)]
pub enum NodeParticles {
    Shuriken(ParticleSystem),
    Legacy(Box<LegacyParticles>),
}

impl NodeParticles {
    /// The node's emitter, if any. `seed` seeds its random numbers.
    pub fn of_node(
        prefab: &PrefabInstance,
        node: usize,
        meshes: &mut MeshCache,
        seed: u32,
    ) -> Option<NodeParticles> {
        if let Some(def) = particle_system_def(prefab, node, meshes) {
            return Some(NodeParticles::Shuriken(ParticleSystem::new(
                Arc::new(def),
                seed,
            )));
        }
        let emitter = prefab.component(node, "EllipsoidParticleEmitter")?;
        let def = LegacyEmitterDef::from_components(
            emitter,
            prefab.component(node, "ParticleAnimator"),
            prefab.component(node, "ParticleRenderer"),
        );
        Some(NodeParticles::Legacy(Box::new(LegacyParticles::new(
            def, seed,
        ))))
    }

    /// `ParticleSystem.isEmitting` / `ParticleEmitter.emit`.
    pub fn emitting(&self) -> bool {
        match self {
            NodeParticles::Shuriken(system) => system.emitting,
            NodeParticles::Legacy(emitter) => emitter.emit,
        }
    }

    /// `ParticleSystem.Play()` / `Stop()`, or setting `ParticleEmitter.emit`.
    pub fn set_emitting(&mut self, on: bool) {
        match self {
            NodeParticles::Shuriken(system) if on => system.play(),
            NodeParticles::Shuriken(system) => system.stop(),
            NodeParticles::Legacy(emitter) => emitter.emit = on,
        }
    }

    pub fn simulate(&mut self, dt: f32, emitter: Mat4, world: Option<&CollisionWorld>) {
        match self {
            NodeParticles::Shuriken(system) => system.simulate(dt, emitter, world),
            NodeParticles::Legacy(particles) => particles.simulate(dt, emitter),
        }
    }

    pub fn geometry(&self, emitter: Mat4, view: &View, weapon_layer: bool) -> Option<DynamicBatch> {
        match self {
            NodeParticles::Shuriken(system) => system.geometry(emitter, view, weapon_layer),
            NodeParticles::Legacy(particles) => particles.geometry(emitter, view, weapon_layer),
        }
    }
}

impl EffectDef {
    /// `None` when the prefab has nothing this module plays.
    pub fn from_prefab(prefab: &PrefabInstance, meshes: &mut MeshCache) -> Option<EffectDef> {
        let mut systems = Vec::new();
        let mut sources = Vec::new();
        for node in 0..prefab.nodes.len() {
            if !prefab.active_in_hierarchy(node) {
                continue;
            }
            if let Some(def) = particle_system_def(prefab, node, meshes) {
                systems.push((node, Arc::new(def)));
            }
            for source in prefab.components(node, "AudioSource") {
                if source["m_Enabled"].as_i64().unwrap_or(1) != 0 {
                    sources.push((node, SourceSettings::from_component(source)));
                }
            }
        }
        let trail = prefab
            .component(0, "TrailRenderer")
            .map(|t| Arc::new(TrailDef::from_component(t)));
        let wait_for_destroy = prefab
            .script_fields(0, "WaitForDestroy")
            .map(|f| f32_of(&f["lifeTime"]));
        let auto_destruct = prefab
            .script_fields(0, "CFX_AutoDestructShuriken")
            .map(|f| f["OnlyDeactivate"].as_bool().unwrap_or(false));
        let explosion = prefab
            .script_fields(0, "ExplosionDamage")
            .map(|f| ExplosionFields {
                radius: f32_of(&f["explosionRadius"]),
                power: f32_of(&f["explosionPower"]),
                damage: f32_of(&f["explosionDamage"]),
                timeout: f32_of(&f["explosionTimeout"]),
                is_remote: f["isRemote"].as_bool().unwrap_or(false),
            });
        if systems.is_empty() && sources.is_empty() && trail.is_none() {
            return None;
        }
        let root_inverse = prefab.world_of(0).inverse();
        Some(EffectDef {
            systems,
            sources,
            trail,
            wait_for_destroy,
            auto_destruct,
            explosion,
            relative: (0..prefab.nodes.len())
                .map(|n| root_inverse * prefab.world_of(n))
                .collect(),
            root_scale: prefab.nodes[0].local.scale,
        })
    }
}

/// Something an effect did that other systems react to.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectEvent {
    /// `ExplosionDamage.Start`: `OverlapSphere` + `SendMessageUpwards("ApplyDamage")` with
    /// `explosion_damage`, then `AddExplosionForce` on rigidbodies.
    Explosion {
        position: Vec3,
        fields: ExplosionFields,
    },
}

/// One live instance.
pub struct Effect {
    pub def: Arc<EffectDef>,
    pub position: Vec3,
    pub rotation: Quat,
    /// `Spawned::id` of the projectile this instance belongs to (trail/sound of a flying object).
    follow: Option<u64>,
    systems: Vec<ParticleSystem>,
    sources: Vec<u64>,
    trail: Option<Trail>,
    started: bool,
    destroy_at: f32,
    /// `CFX_AutoDestructShuriken.CheckIfAlive`: next `WaitForSeconds(0.5)` resume.
    cfx_check_at: f32,
    active: bool,
    /// `ExplosionDamage.isRemote` set after `Instantiate` (by a remote player's projectile).
    pub is_remote: bool,
}

impl Effect {
    fn root(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.def.root_scale, self.rotation, self.position)
    }

    fn node_world(&self, node: usize) -> Mat4 {
        self.root() * self.def.relative[node]
    }

    /// `ParticleSystem.IsAlive(true)` of the root system (with its children).
    fn alive(&self) -> bool {
        self.systems.iter().any(ParticleSystem::is_alive)
    }
}

#[derive(Default)]
pub struct Effects {
    /// Per template index (`None`: nothing to play).
    defs: HashMap<usize, Option<Arc<EffectDef>>>,
    pub instances: Vec<Effect>,
    pub events: Vec<EffectEvent>,
    seed: u32,
}

impl Effects {
    fn def(
        &mut self,
        templates: &Templates,
        template: usize,
        meshes: &mut MeshCache,
    ) -> Option<Arc<EffectDef>> {
        self.defs
            .entry(template)
            .or_insert_with(|| {
                EffectDef::from_prefab(&templates.list[template].prefab, meshes).map(Arc::new)
            })
            .clone()
    }

    /// `Object.Instantiate` of an effect prefab: `Awake` / `OnEnable` run here (play-on-awake
    /// systems and sources start, `WaitForDestroy` schedules the destroy). Returns the instance
    /// so the spawner can set its fields.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        templates: &Templates,
        template: usize,
        position: Vec3,
        rotation: Quat,
        follow: Option<u64>,
        time: f32,
        audio: &mut Audio,
        meshes: &mut MeshCache,
    ) -> Option<&mut Effect> {
        let def = self.def(templates, template, meshes)?;
        let mut effect = Effect {
            def: def.clone(),
            position,
            rotation,
            follow,
            systems: Vec::new(),
            sources: Vec::new(),
            trail: def.trail.clone().map(|t| Trail::new(t, position, time)),
            started: false,
            destroy_at: def
                .wait_for_destroy
                .map_or(f32::INFINITY, |life| time + life),
            cfx_check_at: time + 0.5,
            active: true,
            is_remote: false,
        };
        for (_, system) in &def.systems {
            self.seed = self.seed.wrapping_add(0x9E37_79B9);
            effect
                .systems
                .push(ParticleSystem::new(system.clone(), self.seed));
        }
        for (node, settings) in &def.sources {
            let at = effect.node_world(*node).w_axis.truncate();
            effect.sources.push(audio.add_source(settings.clone(), at));
        }
        self.instances.push(effect);
        self.instances.last_mut()
    }

    /// Creates the instances of newly spawned projectiles (their trail / flying sound) and
    /// destroys those whose projectile is gone.
    pub fn sync_projectiles(
        &mut self,
        templates: &Templates,
        projectiles: &Projectiles,
        time: f32,
        audio: &mut Audio,
        meshes: &mut MeshCache,
    ) {
        let live: HashMap<u64, (usize, Vec3, Quat)> = projectiles
            .objects
            .iter()
            .map(|o| (o.id, (o.template, o.position, o.rotation)))
            .collect();
        for effect in &mut self.instances {
            if let Some(id) = effect.follow {
                match live.get(&id) {
                    Some(&(_, position, rotation)) => {
                        effect.position = position;
                        effect.rotation = rotation;
                    }
                    None => effect.destroy_at = time,
                }
            }
        }
        let followed: Vec<u64> = self.instances.iter().filter_map(|e| e.follow).collect();
        for (id, (template, position, rotation)) in live {
            if !followed.contains(&id) {
                self.spawn(
                    templates,
                    template,
                    position,
                    rotation,
                    Some(id),
                    time,
                    audio,
                    meshes,
                );
            }
        }
    }

    /// One frame: `Start` of new instances, the scripts' coroutines, particle simulation,
    /// sources following their objects, trails, and the `Destroy`s due this frame.
    pub fn update(&mut self, dt: f32, time: f32, world: &CollisionWorld, audio: &mut Audio) {
        for effect in &mut self.instances {
            if !effect.started {
                effect.started = true;
                if let Some(mut fields) = effect.def.explosion {
                    // ExplosionDamage.Start (CrazyGames `Fbg`): damage and force, then, as the
                    // root has no legacy `ParticleEmitter` to switch on for 0.5 s,
                    // `Destroy(gameObject, explosionTimeout)`. The play-on-awake systems are left
                    // playing (Build-7 stopped them and played them again 0.5 s later).
                    fields.is_remote |= effect.is_remote;
                    self.events.push(EffectEvent::Explosion {
                        position: effect.position,
                        fields,
                    });
                    effect.destroy_at = effect.destroy_at.min(time + fields.timeout);
                }
            }
            // CheckIfAlive was started in OnEnable, before ExplosionDamage's Start: it resumes
            // first in the frame both waits end.
            if let Some(only_deactivate) = effect.def.auto_destruct
                && effect.active
                && time >= effect.cfx_check_at
            {
                effect.cfx_check_at += 0.5;
                if !effect.alive() {
                    if only_deactivate {
                        effect.active = false;
                    } else {
                        effect.destroy_at = effect.destroy_at.min(time);
                    }
                }
            }
            if effect.active {
                for (i, (node, _)) in effect.def.systems.iter().enumerate() {
                    let emitter = effect.node_world(*node);
                    effect.systems[i].simulate(dt, emitter, Some(world));
                }
            }
            for (i, (node, _)) in effect.def.sources.iter().enumerate() {
                audio.set_position(
                    effect.sources[i],
                    effect.node_world(*node).w_axis.truncate(),
                );
            }
            if let Some(trail) = &mut effect.trail {
                trail.update(effect.position, time);
            }
        }
        // Destroy (end of frame): the GameObject's sources stop with it.
        self.instances.retain(|effect| {
            let keep = time < effect.destroy_at;
            if !keep {
                for &id in &effect.sources {
                    audio.remove(id);
                }
            }
            keep
        });
    }

    /// Particle and trail geometry of the live instances.
    pub fn batches(&self, view: &View) -> Vec<DynamicBatch> {
        let mut out = Vec::new();
        for effect in &self.instances {
            if !effect.active {
                continue;
            }
            for (i, (node, _)) in effect.def.systems.iter().enumerate() {
                let emitter = effect.node_world(*node);
                out.extend(effect.systems[i].geometry(emitter, view, false));
            }
            if let Some(trail) = &effect.trail {
                out.extend(trail.geometry(view, false));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::ClipLibrary;
    use crate::assets::{find_assets_dir, load_tags};

    const PISTOL_BULLET: &str = "resources.assets:4141";
    const ROCKET: &str = "resources.assets:11017";

    fn setup() -> (Templates, Audio, MeshCache) {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        let tags = load_tags(&assets).unwrap();
        let refs = [PISTOL_BULLET.to_string(), ROCKET.to_string()];
        let templates = Templates::load(&assets, &refs, tags, &mut clips).unwrap();
        (templates, Audio::silent(&assets), MeshCache::new(&assets))
    }

    fn template(t: &Templates, name: &str) -> usize {
        t.list.iter().position(|t| t.name == name).unwrap()
    }

    #[test]
    fn metal_impact_plays_and_destroys_after_life_time() {
        let (t, mut audio, mut meshes) = setup();
        let world = CollisionWorld::new();
        let mut effects = Effects::default();
        effects.spawn(
            &t,
            template(&t, "Metal"),
            Vec3::ZERO,
            Quat::IDENTITY,
            None,
            0.0,
            &mut audio,
            &mut meshes,
        );
        let def = effects.instances[0].def.clone();
        assert_eq!(def.wait_for_destroy, Some(0.5));
        assert_eq!(def.sources.len(), 2);
        assert!(
            def.sources
                .iter()
                .all(|(_, s)| s.spatial && s.play_on_awake)
        );
        let mut time = 0.0;
        let mut frames = 0;
        while !effects.instances.is_empty() {
            time += 0.02;
            frames += 1;
            effects.update(0.02, time, &world, &mut audio);
            assert!(frames < 100);
        }
        assert_eq!(frames, 25);
    }

    #[test]
    fn explosion_plays_and_is_destroyed_after_its_timeout() {
        let (t, mut audio, mut meshes) = setup();
        let world = CollisionWorld::new();
        let mut effects = Effects::default();
        let explosion = template(&t, "WFXMR_Explosion 1");
        effects.spawn(
            &t,
            explosion,
            Vec3::ZERO,
            Quat::IDENTITY,
            None,
            0.0,
            &mut audio,
            &mut meshes,
        );
        assert_eq!(effects.instances[0].def.systems.len(), 9);
        let mut time = 0.0;
        let mut drawn = false;
        let mut frames = 0;
        while !effects.instances.is_empty() {
            time += 0.02;
            frames += 1;
            effects.update(0.02, time, &world, &mut audio);
            drawn |= effects
                .instances
                .iter()
                .any(|e| e.systems.iter().any(|s| !s.particles.is_empty()));
            assert!(frames < 200);
        }
        let Some(EffectEvent::Explosion { fields, .. }) = effects.events.first() else {
            panic!("no explosion event");
        };
        assert_eq!(
            *fields,
            ExplosionFields {
                radius: 6.0,
                power: 15.0,
                damage: 100.0,
                timeout: 2.0,
                is_remote: false,
            }
        );
        assert_eq!(explosion_damage(fields, 3.0), 50.0);
        // The systems play from awake; `Destroy(gameObject, explosionTimeout)` removes the
        // explosion 2 s after its `Start` (first update, 0.02 s), while CheckIfAlive still finds
        // particles alive (frame 101, or 102 with the summed float time).
        assert!(drawn);
        assert!((101..=102).contains(&frames), "{frames}");
    }

    #[test]
    fn rocket_trail_follows_and_dies_with_rocket() {
        let (t, mut audio, mut meshes) = setup();
        let world = CollisionWorld::new();
        let mut projectiles = Projectiles::default();
        let rocket = t.find(ROCKET).unwrap();
        projectiles.spawn(&t, rocket, Vec3::ZERO, Quat::IDENTITY, Some(Vec3::Z * 20.0));
        let mut effects = Effects::default();
        let mut time = 0.0;
        for _ in 0..10 {
            time += 0.02;
            projectiles.update(
                &t,
                &world,
                &crate::projectiles::Targets::default(),
                0.02,
                time,
            );
            projectiles.fixed_update(
                &t,
                &world,
                &crate::projectiles::Targets::default(),
                0.02,
                time,
            );
            effects.sync_projectiles(&t, &projectiles, time, &mut audio, &mut meshes);
            effects.update(0.02, time, &world, &mut audio);
        }
        assert_eq!(effects.instances.len(), 1);
        let view = View {
            position: Vec3::new(3.0, 0.0, 1.0),
            rotation: Quat::IDENTITY,
            fov_y_degrees: 60.0,
            near: 0.1,
            far: 100.0,
        };
        let batches = effects.batches(&view);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].material, "resources.assets:40");
        projectiles.objects.clear();
        effects.sync_projectiles(&t, &projectiles, time, &mut audio, &mut meshes);
        effects.update(0.02, time, &world, &mut audio);
        assert!(effects.instances.is_empty());
    }
}
