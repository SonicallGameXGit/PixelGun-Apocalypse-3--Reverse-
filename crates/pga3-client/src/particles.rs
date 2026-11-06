//! Unity `ParticleSystem` ("Shuriken", 2018.3) for the effects this game spawns: impacts, the
//! rocket explosion and the shotgun's smoke (spec: `docs/gameplay/particles.md`).
//!
//! The modules and modes these prefabs enable are implemented: main module (start delay,
//! lifetime, speed, size, rotation, colour, gravity, simulation space, max particles), emission
//! (rate over time, bursts), shape (sphere, hemisphere, cone, none; random/spherical direction),
//! velocity / force / limit velocity over lifetime, size / colour / rotation over lifetime,
//! rotation by speed, world collision, and the billboard, stretched billboard and mesh render
//! modes with the renderer's max particle size. Unity's own particle random numbers are internal
//! to the engine, so individual particles differ from the original run to run (as they do between
//! two runs of the original); the distributions follow the data.

use std::sync::Arc;

use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use serde_json::Value;

use crate::assets::f32_of;
use crate::obj::MeshData;
use crate::physics::CollisionWorld;
use crate::render::{DYNAMIC_STRIDE, DynamicBatch, View};

/// `Physics.gravity` (`PhysicsManager.m_Gravity`).
pub const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

/// A Unity `AnimationCurve` (`m_Curve` Hermite keys), clamped outside its keys.
#[derive(Clone, Debug, Default)]
pub struct Curve {
    keys: Vec<(f32, f32, f32, f32)>,
}

impl Curve {
    pub fn from_json(value: &Value) -> Curve {
        Curve {
            keys: value["m_Curve"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|k| {
                    (
                        f32_of(&k["time"]),
                        f32_of(&k["value"]),
                        f32_of(&k["inSlope"]),
                        f32_of(&k["outSlope"]),
                    )
                })
                .collect(),
        }
    }

    pub fn evaluate(&self, time: f32) -> f32 {
        let (Some(first), Some(last)) = (self.keys.first(), self.keys.last()) else {
            return 0.0;
        };
        if time <= first.0 {
            return first.1;
        }
        if time >= last.0 {
            return last.1;
        }
        for pair in self.keys.windows(2) {
            let ((t0, v0, _, out0), (t1, v1, in1, _)) = (pair[0], pair[1]);
            if time <= t1 {
                // Infinite tangents are stepped keys.
                if !out0.is_finite() || !in1.is_finite() {
                    return v0;
                }
                let dt = t1 - t0;
                if dt <= 0.0 {
                    return v1;
                }
                let s = (time - t0) / dt;
                let (s2, s3) = (s * s, s * s * s);
                return (2.0 * s3 - 3.0 * s2 + 1.0) * v0
                    + (s3 - 2.0 * s2 + s) * out0 * dt
                    + (-2.0 * s3 + 3.0 * s2) * v1
                    + (s3 - s2) * in1 * dt;
            }
        }
        last.1
    }
}

/// `MinMaxCurve` (`minMaxState` 0 constant, 1 curve, 2 random between two curves, 3 random
/// between two constants). Curves are scaled by `scalar`.
#[derive(Clone, Debug)]
pub enum MinMaxCurve {
    Constant(f32),
    Curve(f32, Curve),
    TwoCurves(f32, Curve, Curve),
    TwoConstants(f32, f32),
}

impl MinMaxCurve {
    pub fn from_json(value: &Value) -> MinMaxCurve {
        let scalar = f32_of(&value["scalar"]);
        match value["minMaxState"].as_i64().unwrap_or(0) {
            1 => MinMaxCurve::Curve(scalar, Curve::from_json(&value["maxCurve"])),
            2 => MinMaxCurve::TwoCurves(
                scalar,
                Curve::from_json(&value["minCurve"]),
                Curve::from_json(&value["maxCurve"]),
            ),
            3 => MinMaxCurve::TwoConstants(f32_of(&value["minScalar"]), scalar),
            _ => MinMaxCurve::Constant(scalar),
        }
    }

    /// `t` = normalised time, `random` in [0, 1) picks between the min and max.
    pub fn evaluate(&self, t: f32, random: f32) -> f32 {
        match self {
            MinMaxCurve::Constant(c) => *c,
            MinMaxCurve::TwoConstants(a, b) => a + (b - a) * random,
            MinMaxCurve::Curve(s, c) => s * c.evaluate(t),
            MinMaxCurve::TwoCurves(s, a, b) => {
                let (a, b) = (a.evaluate(t), b.evaluate(t));
                s * (a + (b - a) * random)
            }
        }
    }
}

/// A Unity `Gradient` (up to 8 colour and 8 alpha keys, times in 1/65535).
#[derive(Clone, Debug, Default)]
pub struct Gradient {
    colors: Vec<(f32, Vec3)>,
    alphas: Vec<(f32, f32)>,
    /// `m_Mode` 1: fixed (no blending).
    fixed: bool,
}

impl Gradient {
    pub fn from_json(value: &Value) -> Gradient {
        let count = |key: &str| value[key].as_u64().unwrap_or(0) as usize;
        let key = |i: usize| &value[format!("key{i}")];
        let time = |name: &str, i: usize| {
            value[format!("{name}{i}")].as_f64().unwrap_or(0.0) as f32 / 65535.0
        };
        Gradient {
            colors: (0..count("m_NumColorKeys").min(8))
                .map(|i| {
                    let k = key(i);
                    (
                        time("ctime", i),
                        Vec3::new(f32_of(&k["r"]), f32_of(&k["g"]), f32_of(&k["b"])),
                    )
                })
                .collect(),
            alphas: (0..count("m_NumAlphaKeys").min(8))
                .map(|i| (time("atime", i), f32_of(&key(i)["a"])))
                .collect(),
            fixed: value["m_Mode"].as_i64() == Some(1),
        }
    }

    pub fn evaluate(&self, t: f32) -> Vec4 {
        let color = sample_keys(&self.colors, t, self.fixed, Vec3::ONE, |a, b, s| {
            a + (b - a) * s
        });
        let alpha = sample_keys(&self.alphas, t, self.fixed, 1.0, |a, b, s| a + (b - a) * s);
        color.extend(alpha)
    }
}

fn sample_keys<T: Copy>(
    keys: &[(f32, T)],
    t: f32,
    fixed: bool,
    default: T,
    mix: impl Fn(T, T, f32) -> T,
) -> T {
    let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
        return default;
    };
    if t <= first.0 {
        return first.1;
    }
    for pair in keys.windows(2) {
        let ((t0, v0), (t1, v1)) = (pair[0], pair[1]);
        if t <= t1 {
            if fixed {
                return v1;
            }
            let span = t1 - t0;
            return if span <= 0.0 {
                v1
            } else {
                mix(v0, v1, (t - t0) / span)
            };
        }
    }
    last.1
}

/// `MinMaxGradient` (0 colour, 1 gradient, 2 random between two colours, 3 random between two
/// gradients, 4 random colour from a gradient).
#[derive(Clone, Debug)]
pub enum MinMaxGradient {
    Color(Vec4),
    Gradient(Gradient),
    TwoColors(Vec4, Vec4),
    TwoGradients(Gradient, Gradient),
    RandomColor(Gradient),
}

fn color_of(value: &Value) -> Vec4 {
    Vec4::new(
        f32_of(&value["r"]),
        f32_of(&value["g"]),
        f32_of(&value["b"]),
        f32_of(&value["a"]),
    )
}

impl MinMaxGradient {
    pub fn from_json(value: &Value) -> MinMaxGradient {
        match value["minMaxState"].as_i64().unwrap_or(0) {
            1 => MinMaxGradient::Gradient(Gradient::from_json(&value["maxGradient"])),
            2 => MinMaxGradient::TwoColors(
                color_of(&value["minColor"]),
                color_of(&value["maxColor"]),
            ),
            3 => MinMaxGradient::TwoGradients(
                Gradient::from_json(&value["minGradient"]),
                Gradient::from_json(&value["maxGradient"]),
            ),
            4 => MinMaxGradient::RandomColor(Gradient::from_json(&value["maxGradient"])),
            _ => MinMaxGradient::Color(color_of(&value["maxColor"])),
        }
    }

    pub fn evaluate(&self, t: f32, random: f32) -> Vec4 {
        match self {
            MinMaxGradient::Color(c) => *c,
            MinMaxGradient::Gradient(g) => g.evaluate(t),
            MinMaxGradient::TwoColors(a, b) => a.lerp(*b, random),
            MinMaxGradient::TwoGradients(a, b) => a.evaluate(t).lerp(b.evaluate(t), random),
            MinMaxGradient::RandomColor(g) => g.evaluate(random),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeType {
    /// Shape module disabled: from the origin along +Z.
    None,
    Sphere,
    Hemisphere,
    /// Cone, emitting from its base.
    Cone,
    /// Other shapes (not used by the ported effects) emit like a point.
    Other,
}

#[derive(Clone, Debug)]
pub struct Shape {
    pub kind: ShapeType,
    pub radius: f32,
    pub radius_thickness: f32,
    pub angle: f32,
    pub arc: f32,
    pub random_direction: f32,
    pub spherical_direction: f32,
}

#[derive(Clone, Debug)]
pub struct Burst {
    pub time: f32,
    pub count: MinMaxCurve,
    pub cycles: i32,
    pub interval: f32,
    pub probability: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    Billboard,
    Stretched,
    Mesh,
    /// Horizontal / vertical billboards (unused here) are drawn as view billboards.
    Other,
}

/// The `ParticleSystem` + `ParticleSystemRenderer` of one GameObject.
#[derive(Clone, Debug)]
pub struct ParticleSystemDef {
    pub duration: f32,
    pub looping: bool,
    pub play_on_awake: bool,
    pub start_delay: MinMaxCurve,
    /// `moveWithTransform` 1: particles live in world space.
    pub world_space: bool,
    pub start_lifetime: MinMaxCurve,
    pub start_speed: MinMaxCurve,
    pub start_color: MinMaxGradient,
    pub start_size: MinMaxCurve,
    pub start_rotation: MinMaxCurve,
    pub gravity_modifier: MinMaxCurve,
    pub max_particles: usize,
    pub rate_over_time: Option<MinMaxCurve>,
    pub bursts: Vec<Burst>,
    pub shape: Shape,
    pub velocity: Option<(MinMaxCurve, MinMaxCurve, MinMaxCurve, bool)>,
    pub force: Option<(MinMaxCurve, MinMaxCurve, MinMaxCurve, bool)>,
    /// Limit velocity: (magnitude, dampen).
    pub limit_velocity: Option<(MinMaxCurve, f32)>,
    pub size_over_lifetime: Option<MinMaxCurve>,
    pub color_over_lifetime: Option<MinMaxGradient>,
    /// Angular velocity over lifetime (radians/s).
    pub rotation_over_lifetime: Option<MinMaxCurve>,
    /// Rotation by speed: (angular velocity, speed range).
    pub rotation_by_speed: Option<(MinMaxCurve, Vec2)>,
    /// World collision: (dampen, bounce, lifetime loss, radius scale).
    pub collision: Option<(f32, f32, f32, f32)>,
    // Renderer.
    pub render_mode: RenderMode,
    pub material: Option<String>,
    pub mesh: Option<Arc<MeshData>>,
    pub max_particle_size: f32,
    pub velocity_scale: f32,
    pub length_scale: f32,
    /// `m_RenderAlignment` 2 (local): mesh particles follow the system's rotation.
    pub align_local: bool,
    pub renderer_enabled: bool,
}

fn module<'a>(data: &'a Value, name: &str) -> Option<&'a Value> {
    let m = &data[name];
    (m["enabled"].as_bool() == Some(true)).then_some(m)
}

fn curve(value: &Value) -> MinMaxCurve {
    MinMaxCurve::from_json(value)
}

impl ParticleSystemDef {
    /// From a `ParticleSystem` and its `ParticleSystemRenderer` component data. `mesh` loads the
    /// renderer's mesh for the mesh render mode.
    pub fn from_components(
        ps: &Value,
        renderer: Option<&Value>,
        mesh: impl FnOnce(&Value) -> Option<Arc<MeshData>>,
    ) -> ParticleSystemDef {
        let initial = &ps["InitialModule"];
        let shape = match module(ps, "ShapeModule") {
            None => Shape {
                kind: ShapeType::None,
                radius: 0.0,
                radius_thickness: 1.0,
                angle: 0.0,
                arc: 360.0,
                random_direction: 0.0,
                spherical_direction: 0.0,
            },
            Some(s) => Shape {
                kind: match s["type"].as_i64().unwrap_or(0) {
                    0 => ShapeType::Sphere,
                    2 => ShapeType::Hemisphere,
                    4 => ShapeType::Cone,
                    _ => ShapeType::Other,
                },
                radius: f32_of(&s["radius"]["value"]),
                radius_thickness: f32_of(&s["radiusThickness"]),
                angle: f32_of(&s["angle"]),
                arc: f32_of(&s["arc"]["value"]),
                random_direction: f32_of(&s["randomDirectionAmount"]),
                spherical_direction: f32_of(&s["sphericalDirectionAmount"]),
            },
        };
        let emission = module(ps, "EmissionModule");
        let xyz = |m: &Value, world: &str| {
            (
                curve(&m["x"]),
                curve(&m["y"]),
                curve(&m["z"]),
                m[world].as_bool().unwrap_or(false),
            )
        };
        let r = renderer.unwrap_or(&Value::Null);
        let render_mode = match r["m_RenderMode"].as_i64().unwrap_or(0) {
            0 => RenderMode::Billboard,
            1 => RenderMode::Stretched,
            4 => RenderMode::Mesh,
            _ => RenderMode::Other,
        };
        let mesh = if render_mode == RenderMode::Mesh {
            mesh(&r["m_Mesh"])
        } else {
            None
        };
        ParticleSystemDef {
            duration: f32_of(&ps["lengthInSec"]),
            looping: ps["looping"].as_bool().unwrap_or(false),
            play_on_awake: ps["playOnAwake"].as_bool().unwrap_or(true),
            start_delay: curve(&ps["startDelay"]),
            world_space: ps["moveWithTransform"].as_i64() == Some(1),
            start_lifetime: curve(&initial["startLifetime"]),
            start_speed: curve(&initial["startSpeed"]),
            start_color: MinMaxGradient::from_json(&initial["startColor"]),
            start_size: curve(&initial["startSize"]),
            start_rotation: curve(&initial["startRotation"]),
            gravity_modifier: curve(&initial["gravityModifier"]),
            max_particles: initial["maxNumParticles"].as_u64().unwrap_or(1000) as usize,
            rate_over_time: emission.map(|e| curve(&e["rateOverTime"])),
            bursts: emission
                .and_then(|e| e["m_Bursts"].as_array())
                .into_iter()
                .flatten()
                .map(|b| Burst {
                    time: f32_of(&b["time"]),
                    count: curve(&b["countCurve"]),
                    cycles: b["cycleCount"].as_i64().unwrap_or(1) as i32,
                    interval: f32_of(&b["repeatInterval"]),
                    probability: f32_of(&b["probability"]),
                })
                .collect(),
            shape,
            velocity: module(ps, "VelocityModule").map(|m| xyz(m, "inWorldSpace")),
            force: module(ps, "ForceModule").map(|m| xyz(m, "inWorldSpace")),
            limit_velocity: module(ps, "ClampVelocityModule")
                .map(|m| (curve(&m["magnitude"]), f32_of(&m["dampen"]))),
            size_over_lifetime: module(ps, "SizeModule").map(|m| curve(&m["curve"])),
            color_over_lifetime: module(ps, "ColorModule")
                .map(|m| MinMaxGradient::from_json(&m["gradient"])),
            rotation_over_lifetime: module(ps, "RotationModule").map(|m| curve(&m["curve"])),
            rotation_by_speed: module(ps, "RotationBySpeedModule").map(|m| {
                (
                    curve(&m["curve"]),
                    Vec2::new(f32_of(&m["range"]["x"]), f32_of(&m["range"]["y"])),
                )
            }),
            collision: module(ps, "CollisionModule")
                .filter(|m| m["type"].as_i64() == Some(1))
                .map(|m| {
                    (
                        curve(&m["m_Dampen"]).evaluate(0.0, 0.0),
                        curve(&m["m_Bounce"]).evaluate(0.0, 0.0),
                        curve(&m["m_EnergyLossOnCollision"]).evaluate(0.0, 0.0),
                        f32_of(&m["radiusScale"]),
                    )
                }),
            render_mode,
            material: r["m_Materials"][0]["ref"].as_str().map(str::to_string),
            mesh,
            max_particle_size: if r.is_null() {
                0.5
            } else {
                f32_of(&r["m_MaxParticleSize"])
            },
            velocity_scale: f32_of(&r["m_VelocityScale"]),
            length_scale: f32_of(&r["m_LengthScale"]),
            align_local: r["m_RenderAlignment"].as_i64() == Some(2),
            renderer_enabled: r["m_Enabled"].as_bool().unwrap_or(false),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Particle {
    /// World or emitter-local position (see `ParticleSystemDef::world_space`).
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f32,
    pub lifetime: f32,
    pub size: f32,
    pub color: Vec4,
    /// Radians.
    pub rotation: f32,
    /// Per-particle random values for the "random between" modes.
    random: [f32; 8],
    /// Stuck after a fully damped collision.
    resting: bool,
}

/// Small xorshift generator for particle randomness (Unity's particle RNG is internal).
#[derive(Clone, Debug)]
pub struct ParticleRandom(u32);

impl ParticleRandom {
    pub fn new(seed: u32) -> ParticleRandom {
        ParticleRandom(seed.max(1))
    }

    pub fn value(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn unit_vector(&mut self) -> Vec3 {
        let z = self.value() * 2.0 - 1.0;
        let a = self.value() * std::f32::consts::TAU;
        let r = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(r * a.cos(), r * a.sin(), z)
    }
}

/// Runtime state of one system.
#[derive(Clone, Debug)]
pub struct ParticleSystem {
    pub def: Arc<ParticleSystemDef>,
    pub particles: Vec<Particle>,
    /// Playback time since `Play` (minus the start delay).
    pub time: f32,
    delay: f32,
    pub playing: bool,
    pub emitting: bool,
    bursts_done: Vec<i32>,
    rate_accumulator: f32,
    random: ParticleRandom,
}

impl ParticleSystem {
    pub fn new(def: Arc<ParticleSystemDef>, seed: u32) -> ParticleSystem {
        let bursts = def.bursts.len();
        let mut system = ParticleSystem {
            def,
            particles: Vec::new(),
            time: 0.0,
            delay: 0.0,
            playing: false,
            emitting: false,
            bursts_done: vec![0; bursts],
            rate_accumulator: 0.0,
            random: ParticleRandom::new(seed),
        };
        if system.def.play_on_awake {
            system.play();
        }
        system
    }

    /// `Play`: (re)starts emission from time 0; live particles are kept.
    pub fn play(&mut self) {
        if self.playing && self.emitting {
            return;
        }
        self.time = 0.0;
        let r = self.random.value();
        self.delay = self.def.start_delay.evaluate(0.0, r);
        self.bursts_done.iter_mut().for_each(|b| *b = 0);
        self.rate_accumulator = 0.0;
        self.playing = true;
        self.emitting = true;
    }

    /// `Stop()` = `ParticleSystemStopBehavior.StopEmitting`: live particles finish.
    pub fn stop(&mut self) {
        self.emitting = false;
        self.playing = false;
    }

    /// `IsAlive`: still emitting or has live particles.
    pub fn is_alive(&self) -> bool {
        self.emitting || !self.particles.is_empty()
    }

    /// One frame of `dt` seconds. `emitter` is the system GameObject's world matrix.
    pub fn simulate(&mut self, dt: f32, emitter: Mat4, world: Option<&CollisionWorld>) {
        if dt <= 0.0 {
            return;
        }
        let def = self.def.clone();
        let (scale, rotation, translation) = emitter.to_scale_rotation_translation();
        // Gravity and world-space modules in the simulation space.
        let to_sim = if def.world_space {
            Quat::IDENTITY
        } else {
            rotation.inverse()
        };
        // Existing particles.
        for p in &mut self.particles {
            p.age += dt;
        }
        self.particles.retain(|p| p.age < p.lifetime);
        for i in 0..self.particles.len() {
            let mut p = self.particles[i].clone();
            self.step_particle(&mut p, dt, &def, to_sim, emitter, world);
            self.particles[i] = p;
        }
        // Emission.
        if self.emitting {
            let start = self.time;
            let mut remaining = dt;
            if self.delay > 0.0 {
                let used = remaining.min(self.delay);
                self.delay -= used;
                remaining -= used;
            }
            if remaining > 0.0 {
                let end = start + remaining;
                let mut emits: Vec<(f32, usize)> = Vec::new();
                for (b, burst) in def.bursts.iter().enumerate() {
                    while self.bursts_done[b] < burst.cycles.max(1) {
                        let at = burst.time + burst.interval * self.bursts_done[b] as f32;
                        let in_range = if start == 0.0 {
                            at <= end
                        } else {
                            at > start && at <= end
                        };
                        if !in_range || at > def.duration && !def.looping {
                            break;
                        }
                        self.bursts_done[b] += 1;
                        if self.random.value() <= burst.probability {
                            let count = burst
                                .count
                                .evaluate(at / def.duration.max(1e-4), self.random.value());
                            emits.push((at, count.round().max(0.0) as usize));
                        }
                    }
                }
                if let Some(rate) = &def.rate_over_time {
                    let r = self.random.value();
                    self.rate_accumulator +=
                        rate.evaluate(start / def.duration.max(1e-4), r) * remaining;
                    let whole = self.rate_accumulator.floor();
                    self.rate_accumulator -= whole;
                    if whole > 0.0 {
                        emits.push((end, whole as usize));
                    }
                }
                for (at, count) in emits {
                    for _ in 0..count {
                        if self.particles.len() >= def.max_particles {
                            break;
                        }
                        let mut p = self.emit(&def, scale, rotation, translation, at);
                        // Simulate the part of the frame after its emission.
                        let late = (end - at).clamp(0.0, remaining);
                        p.age = late;
                        if p.age < p.lifetime {
                            self.step_particle(&mut p, late, &def, to_sim, emitter, world);
                            self.particles.push(p);
                        }
                    }
                }
                self.time = end;
                if !def.looping && self.time >= def.duration {
                    self.emitting = false;
                    self.playing = false;
                } else if def.looping && self.time >= def.duration {
                    self.time -= def.duration;
                    self.bursts_done.iter_mut().for_each(|b| *b = 0);
                }
            }
        }
    }

    fn emit(
        &mut self,
        def: &ParticleSystemDef,
        scale: Vec3,
        rotation: Quat,
        translation: Vec3,
        at: f32,
    ) -> Particle {
        let t = (at / def.duration.max(1e-4)).clamp(0.0, 1.0);
        let r = &mut self.random;
        let (local_position, mut direction) = emit_shape(&def.shape, r);
        if def.shape.random_direction > 0.0 {
            let random = r.unit_vector();
            direction = direction
                .lerp(random, def.shape.random_direction)
                .normalize_or_zero();
        }
        if def.shape.spherical_direction > 0.0 {
            let radial = local_position.normalize_or(direction);
            direction = direction
                .lerp(radial, def.shape.spherical_direction)
                .normalize_or_zero();
        }
        // Scaling mode "Shape": the transform's scale applies to the shape only.
        let shaped = local_position * scale;
        let speed = def.start_speed.evaluate(t, r.value());
        let (position, velocity) = if def.world_space {
            (
                translation + rotation * shaped,
                rotation * direction * speed,
            )
        } else {
            (shaped, direction * speed)
        };
        let mut random = [0.0; 8];
        for value in &mut random {
            *value = r.value();
        }
        Particle {
            position,
            velocity,
            age: 0.0,
            lifetime: def.start_lifetime.evaluate(t, r.value()).max(1e-4),
            size: def.start_size.evaluate(t, r.value()),
            color: def.start_color.evaluate(t, r.value()),
            rotation: def.start_rotation.evaluate(t, r.value()),
            random,
            resting: false,
        }
    }

    fn step_particle(
        &self,
        p: &mut Particle,
        dt: f32,
        def: &ParticleSystemDef,
        to_sim: Quat,
        emitter: Mat4,
        world: Option<&CollisionWorld>,
    ) {
        if dt <= 0.0 {
            return;
        }
        let t = (p.age / p.lifetime).clamp(0.0, 1.0);
        let gravity = def.gravity_modifier.evaluate(t, p.random[0]);
        if !p.resting {
            p.velocity += to_sim * GRAVITY * gravity * dt;
        }
        if let Some((x, y, z, in_world)) = &def.force {
            let f = Vec3::new(
                x.evaluate(t, p.random[1]),
                y.evaluate(t, p.random[1]),
                z.evaluate(t, p.random[1]),
            );
            p.velocity += if *in_world {
                to_sim * f
            } else {
                space_of_local(def, emitter, f)
            } * dt;
        }
        if let Some((limit, dampen)) = &def.limit_velocity {
            let limit = limit.evaluate(t, p.random[2]);
            let speed = p.velocity.length();
            if speed > limit && speed > 0.0 {
                let reduced = speed - (speed - limit) * dampen;
                p.velocity *= reduced / speed;
            }
        }
        let mut animated = Vec3::ZERO;
        if let Some((x, y, z, in_world)) = &def.velocity {
            let v = Vec3::new(
                x.evaluate(t, p.random[3]),
                y.evaluate(t, p.random[4]),
                z.evaluate(t, p.random[5]),
            );
            animated = if *in_world {
                to_sim * v
            } else {
                space_of_local(def, emitter, v)
            };
        }
        let total = if p.resting {
            Vec3::ZERO
        } else {
            p.velocity + animated
        };
        let old = p.position;
        p.position += total * dt;
        if let (Some((dampen, bounce, loss, radius_scale)), Some(world)) = (def.collision, world) {
            let to_world = |v: Vec3| {
                if def.world_space {
                    v
                } else {
                    emitter.transform_point3(v)
                }
            };
            let (a, b) = (to_world(old), to_world(p.position));
            let delta = b - a;
            let distance = delta.length();
            if distance > 1e-6
                && let Some(hit) = world.raycast(a, delta, distance + p.size * radius_scale)
            {
                let n = hit.normal;
                let inside = hit.point + n * (p.size * radius_scale).max(1e-4);
                p.position = if def.world_space {
                    inside
                } else {
                    emitter.inverse().transform_point3(inside)
                };
                let normal_part = n * p.velocity.dot(n);
                let tangent = p.velocity - normal_part;
                p.velocity = (tangent - normal_part * bounce) * (1.0 - dampen);
                p.lifetime -= p.lifetime * loss;
                if p.velocity.length_squared() < 1e-8 {
                    p.resting = true;
                }
            }
        }
        let mut angular = 0.0;
        if let Some(curve) = &def.rotation_over_lifetime {
            angular += curve.evaluate(t, p.random[6]);
        }
        if let Some((curve, range)) = &def.rotation_by_speed {
            let span = (range.y - range.x).max(1e-4);
            let s = ((total.length() - range.x) / span).clamp(0.0, 1.0);
            angular += curve.evaluate(s, p.random[7]);
        }
        p.rotation += angular * dt;
    }

    /// Current size and colour of a particle (start values times the over-lifetime modules).
    fn appearance(&self, p: &Particle) -> (f32, Vec4) {
        let t = (p.age / p.lifetime).clamp(0.0, 1.0);
        let mut size = p.size;
        if let Some(curve) = &self.def.size_over_lifetime {
            size *= curve.evaluate(t, p.random[2]);
        }
        let mut color = p.color;
        if let Some(gradient) = &self.def.color_over_lifetime {
            color *= gradient.evaluate(t, p.random[1]);
        }
        (size, color)
    }

    /// Builds the renderer's geometry as seen from `view`.
    pub fn geometry(&self, emitter: Mat4, view: &View, weapon_layer: bool) -> Option<DynamicBatch> {
        let (camera, camera_rotation, fov_y_degrees) =
            (view.position, view.rotation, view.fov_y_degrees);
        let def = &self.def;
        if !def.renderer_enabled || self.particles.is_empty() {
            return None;
        }
        let material = def.material.clone()?;
        let mut vertices = Vec::with_capacity(self.particles.len() * 4 * DYNAMIC_STRIDE);
        let mut indices = Vec::new();
        let right = camera_rotation * Vec3::X;
        let up = camera_rotation * Vec3::Y;
        let forward = camera_rotation * Vec3::Z;
        let half_tan = (fov_y_degrees.to_radians() * 0.5).tan();
        let (_, system_rotation, _) = emitter.to_scale_rotation_translation();
        let mut center = Vec3::ZERO;
        for p in &self.particles {
            let position = if def.world_space {
                p.position
            } else {
                emitter.transform_point3(p.position)
            };
            center += position;
            let (mut size, color) = self.appearance(p);
            if size <= 0.0 || color.w <= 0.0 && color.truncate() == Vec3::ZERO {
                continue;
            }
            let base = (vertices.len() / DYNAMIC_STRIDE) as u32;
            match def.render_mode {
                RenderMode::Mesh => {
                    let Some(mesh) = &def.mesh else {
                        continue;
                    };
                    let rotation = if def.align_local {
                        system_rotation
                    } else {
                        Quat::IDENTITY
                    } * Quat::from_rotation_z(-p.rotation);
                    let normal_matrix = Mat3::from_quat(rotation);
                    for i in 0..mesh.positions.len() {
                        let v = Vec3::from(mesh.positions[i]);
                        let n = normal_matrix * Vec3::from(mesh.normals[i]);
                        push_vertex(
                            &mut vertices,
                            position + rotation * (v * size),
                            n,
                            Vec2::from(mesh.uvs[i]),
                            color,
                        );
                    }
                    for submesh in &mesh.submeshes {
                        indices.extend(submesh.iter().map(|i| base + i));
                    }
                }
                RenderMode::Stretched => {
                    let velocity = if def.world_space {
                        p.velocity
                    } else {
                        emitter.transform_vector3(p.velocity)
                    };
                    let speed = velocity.length();
                    let axis = if speed > 1e-5 { velocity / speed } else { up };
                    let to_camera = (camera - position).normalize_or(forward);
                    let side = axis.cross(to_camera).normalize_or(right);
                    let depth = (position - camera).dot(forward).max(1e-3);
                    size = size.min(def.max_particle_size * 2.0 * depth * half_tan);
                    let length = size * def.length_scale + speed * def.velocity_scale;
                    let half = side * size * 0.5;
                    // The particle is the quad's head; it trails back along the velocity.
                    let head = position;
                    let tail = position - axis * length;
                    let n = to_camera;
                    push_vertex(&mut vertices, tail - half, n, Vec2::new(0.0, 0.0), color);
                    push_vertex(&mut vertices, tail + half, n, Vec2::new(0.0, 1.0), color);
                    push_vertex(&mut vertices, head + half, n, Vec2::new(1.0, 1.0), color);
                    push_vertex(&mut vertices, head - half, n, Vec2::new(1.0, 0.0), color);
                    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
                }
                RenderMode::Billboard | RenderMode::Other => {
                    let depth = (position - camera).dot(forward).max(1e-3);
                    size = size.min(def.max_particle_size * 2.0 * depth * half_tan);
                    // Positive rotation turns the quad clockwise on screen.
                    let (s, c) = (-p.rotation).sin_cos();
                    let corner = |x: f32, y: f32| {
                        let (rx, ry) = (x * c - y * s, x * s + y * c);
                        position + (right * rx + up * ry) * size
                    };
                    let n = -forward;
                    push_vertex(
                        &mut vertices,
                        corner(-0.5, -0.5),
                        n,
                        Vec2::new(0.0, 0.0),
                        color,
                    );
                    push_vertex(
                        &mut vertices,
                        corner(0.5, -0.5),
                        n,
                        Vec2::new(1.0, 0.0),
                        color,
                    );
                    push_vertex(
                        &mut vertices,
                        corner(0.5, 0.5),
                        n,
                        Vec2::new(1.0, 1.0),
                        color,
                    );
                    push_vertex(
                        &mut vertices,
                        corner(-0.5, 0.5),
                        n,
                        Vec2::new(0.0, 1.0),
                        color,
                    );
                    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
        center /= self.particles.len() as f32;
        Some(DynamicBatch {
            material,
            vertices,
            indices,
            weapon_layer,
            distance: center.distance(camera),
        })
    }
}

/// A local-space module vector in the simulation space.
fn space_of_local(def: &ParticleSystemDef, emitter: Mat4, v: Vec3) -> Vec3 {
    if def.world_space {
        let (_, rotation, _) = emitter.to_scale_rotation_translation();
        rotation * v
    } else {
        v
    }
}

fn push_vertex(out: &mut Vec<f32>, p: Vec3, n: Vec3, uv: Vec2, c: Vec4) {
    out.extend_from_slice(&[p.x, p.y, p.z, n.x, n.y, n.z, uv.x, uv.y, c.x, c.y, c.z, c.w]);
}

/// Shape module: (local position, direction).
fn emit_shape(shape: &Shape, r: &mut ParticleRandom) -> (Vec3, Vec3) {
    match shape.kind {
        ShapeType::None | ShapeType::Other => (Vec3::ZERO, Vec3::Z),
        ShapeType::Sphere | ShapeType::Hemisphere => {
            let mut d = r.unit_vector();
            if shape.kind == ShapeType::Hemisphere {
                d.z = d.z.abs();
            }
            // Radius thickness 0 = surface, 1 = whole volume.
            let fraction = 1.0 - shape.radius_thickness * (1.0 - r.value().cbrt());
            (d * shape.radius * fraction, d)
        }
        ShapeType::Cone => {
            let phi = r.value() * shape.arc.to_radians();
            let fraction = 1.0 - shape.radius_thickness * (1.0 - r.value().sqrt());
            let (s, c) = phi.sin_cos();
            let position = Vec3::new(c, s, 0.0) * shape.radius * fraction;
            let tilt = shape.angle.to_radians() * fraction;
            let direction = Vec3::new(c * tilt.sin(), s * tilt.sin(), tilt.cos());
            (position, direction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn constant(v: f32) -> Value {
        json!({ "minMaxState": 0, "scalar": v, "minScalar": v })
    }

    fn burst_def(count: f32, world_space: bool) -> ParticleSystemDef {
        let ps = json!({
            "lengthInSec": 1.0,
            "looping": false,
            "playOnAwake": true,
            "startDelay": constant(0.0),
            "moveWithTransform": if world_space { 1 } else { 0 },
            "InitialModule": {
                "enabled": true,
                "startLifetime": constant(1.0),
                "startSpeed": constant(2.0),
                "startColor": { "minMaxState": 0, "maxColor": { "r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0 } },
                "startSize": constant(0.5),
                "startRotation": constant(0.0),
                "gravityModifier": constant(0.0),
                "maxNumParticles": 50,
            },
            "EmissionModule": {
                "enabled": true,
                "rateOverTime": constant(0.0),
                "m_Bursts": [{
                    "time": 0.0,
                    "countCurve": constant(count),
                    "cycleCount": 1,
                    "repeatInterval": 0.01,
                    "probability": 1.0,
                }],
            },
            "ShapeModule": {
                "enabled": true,
                "type": 0,
                "radius": { "value": 1.0 },
                "radiusThickness": 0.0,
                "angle": 25.0,
                "arc": { "value": 360.0 },
                "randomDirectionAmount": 0.0,
                "sphericalDirectionAmount": 0.0,
            },
        });
        ParticleSystemDef::from_components(&ps, None, |_| None)
    }

    #[test]
    fn hermite_curve_and_gradient() {
        let c = Curve::from_json(&json!({ "m_Curve": [
            { "time": 0.0, "value": 0.0, "inSlope": 0.0, "outSlope": 1.0 },
            { "time": 1.0, "value": 1.0, "inSlope": 1.0, "outSlope": 0.0 },
        ] }));
        assert!((c.evaluate(0.5) - 0.5).abs() < 1e-6);
        assert_eq!(c.evaluate(2.0), 1.0);
        let g = Gradient::from_json(&json!({
            "key0": { "r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0 },
            "key1": { "r": 0.0, "g": 0.0, "b": 1.0, "a": 0.0 },
            "ctime0": 0, "ctime1": 65535, "atime0": 0, "atime1": 65535,
            "m_Mode": 0, "m_NumColorKeys": 2, "m_NumAlphaKeys": 2,
        }));
        let mid = g.evaluate(0.5);
        assert!((mid - Vec4::new(0.5, 0.0, 0.5, 0.5)).length() < 1e-4);
    }

    #[test]
    fn burst_emits_on_first_frame_and_dies() {
        let def = Arc::new(burst_def(10.0, true));
        let mut system = ParticleSystem::new(def, 7);
        assert!(system.is_alive() && system.particles.is_empty());
        system.simulate(0.02, Mat4::IDENTITY, None);
        assert_eq!(system.particles.len(), 10);
        // Sphere surface, radius 1, moving outwards at 2 m/s.
        for p in &system.particles {
            let d = p.position.length();
            assert!((d - 1.04).abs() < 1e-3, "{d}");
        }
        for _ in 0..60 {
            system.simulate(0.02, Mat4::IDENTITY, None);
        }
        assert!(!system.is_alive());
    }

    #[test]
    fn stop_before_first_update_emits_nothing() {
        let mut system = ParticleSystem::new(Arc::new(burst_def(5.0, false)), 3);
        system.stop();
        system.simulate(0.02, Mat4::IDENTITY, None);
        assert!(system.particles.is_empty() && !system.is_alive());
        system.play();
        system.simulate(0.02, Mat4::IDENTITY, None);
        assert_eq!(system.particles.len(), 5);
    }

    #[test]
    fn max_particles_caps_bursts() {
        let mut system = ParticleSystem::new(Arc::new(burst_def(100.0, true)), 1);
        system.simulate(0.02, Mat4::IDENTITY, None);
        assert_eq!(system.particles.len(), 50);
    }
}
