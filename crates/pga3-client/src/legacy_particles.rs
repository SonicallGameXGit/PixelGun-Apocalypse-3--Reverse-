//! Unity's legacy particle system (`EllipsoidParticleEmitter` + `ParticleAnimator` +
//! `ParticleRenderer`, removed in Unity 2018.3), used by the CrazyGames build (Unity 5.3) for the
//! shotgun's "Smoke Trail" (spec: `docs/gameplay/particles.md`, "Legacy particles").
//!
//! The engine code is native and minified in the CrazyGames build, so the update rules follow
//! Unity's documented legacy behaviour; the parts that are inferred are listed in the spec and in
//! `docs/unknowns.md`. Fields the game's emitters leave at zero (ellipsoid, forces, rotation
//! axes) are still applied, in the simplest documented form.

use glam::{Mat4, Vec2, Vec3, Vec4};
use serde_json::Value;

use crate::assets::f32_of;
use crate::particles::{ParticleRandom, push_vertex};
use crate::render::{DYNAMIC_STRIDE, DynamicBatch, View};

/// `ParticleRenderer.particleRenderMode` (`m_StretchParticles`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyRenderMode {
    /// 0 billboard, 2 sorted billboard.
    Billboard,
    /// 3: stretched along the velocity.
    Stretch,
    /// 4: flat on the XZ plane, facing up.
    Horizontal,
    /// 5: facing the camera but kept upright (around world Y).
    Vertical,
}

#[derive(Clone, Debug)]
pub struct LegacyEmitterDef {
    pub emit: bool,
    pub min_size: f32,
    pub max_size: f32,
    pub min_energy: f32,
    pub max_energy: f32,
    /// Particles per second.
    pub min_emission: f32,
    pub max_emission: f32,
    pub world_velocity: Vec3,
    pub local_velocity: Vec3,
    pub rnd_velocity: Vec3,
    pub angular_velocity: f32,
    pub rnd_angular_velocity: f32,
    pub rnd_rotation: bool,
    pub world_space: bool,
    pub one_shot: bool,
    pub ellipsoid: Vec3,
    pub min_emitter_range: f32,
    // ParticleAnimator.
    pub animate_color: bool,
    pub color_animation: [Vec4; 5],
    pub size_grow: f32,
    pub force: Vec3,
    pub rnd_force: Vec3,
    pub damping: f32,
    // ParticleRenderer.
    pub renderer_enabled: bool,
    pub material: Option<String>,
    pub render_mode: LegacyRenderMode,
    pub length_scale: f32,
    pub velocity_scale: f32,
    pub max_particle_size: f32,
}

/// `ColorRGBA32` (red in the low byte).
fn rgba32(value: &Value) -> Vec4 {
    let rgba = value["rgba"].as_u64().unwrap_or(0xFFFF_FFFF);
    let channel = |i: u32| ((rgba >> (8 * i)) & 0xFF) as f32 / 255.0;
    Vec4::new(channel(0), channel(1), channel(2), channel(3))
}

fn vec3_of(value: &Value) -> Vec3 {
    Vec3::new(
        f32_of(&value["x"]),
        f32_of(&value["y"]),
        f32_of(&value["z"]),
    )
}

impl LegacyEmitterDef {
    /// From an `EllipsoidParticleEmitter` and the node's `ParticleAnimator` / `ParticleRenderer`.
    pub fn from_components(
        emitter: &Value,
        animator: Option<&Value>,
        renderer: Option<&Value>,
    ) -> LegacyEmitterDef {
        let a = animator.unwrap_or(&Value::Null);
        let r = renderer.unwrap_or(&Value::Null);
        LegacyEmitterDef {
            emit: emitter["m_Emit"].as_bool().unwrap_or(true),
            min_size: f32_of(&emitter["minSize"]),
            max_size: f32_of(&emitter["maxSize"]),
            min_energy: f32_of(&emitter["minEnergy"]),
            max_energy: f32_of(&emitter["maxEnergy"]),
            min_emission: f32_of(&emitter["minEmission"]),
            max_emission: f32_of(&emitter["maxEmission"]),
            world_velocity: vec3_of(&emitter["worldVelocity"]),
            local_velocity: vec3_of(&emitter["localVelocity"]),
            rnd_velocity: vec3_of(&emitter["rndVelocity"]),
            angular_velocity: f32_of(&emitter["angularVelocity"]),
            rnd_angular_velocity: f32_of(&emitter["rndAngularVelocity"]),
            rnd_rotation: emitter["rndRotation"].as_bool().unwrap_or(false),
            world_space: emitter["Simulate in Worldspace?"].as_bool().unwrap_or(true),
            one_shot: emitter["m_OneShot"].as_bool().unwrap_or(false),
            ellipsoid: vec3_of(&emitter["m_Ellipsoid"]),
            min_emitter_range: f32_of(&emitter["m_MinEmitterRange"]),
            animate_color: animator.is_some()
                && a["Does Animate Color?"].as_bool().unwrap_or(false),
            color_animation: std::array::from_fn(|i| rgba32(&a[format!("colorAnimation[{i}]")])),
            size_grow: f32_of(&a["sizeGrow"]),
            force: vec3_of(&a["force"]),
            rnd_force: vec3_of(&a["rndForce"]),
            damping: a["damping"].as_f64().unwrap_or(1.0) as f32,
            renderer_enabled: r["m_Enabled"].as_bool().unwrap_or(false),
            material: r["m_Materials"][0]["ref"].as_str().map(str::to_string),
            render_mode: match r["m_StretchParticles"].as_i64().unwrap_or(0) {
                3 => LegacyRenderMode::Stretch,
                4 => LegacyRenderMode::Horizontal,
                5 => LegacyRenderMode::Vertical,
                _ => LegacyRenderMode::Billboard,
            },
            length_scale: f32_of(&r["m_LengthScale"]),
            velocity_scale: f32_of(&r["m_VelocityScale"]),
            max_particle_size: r["m_MaxParticleSize"].as_f64().unwrap_or(0.25) as f32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LegacyParticle {
    /// World or emitter-local position (see `LegacyEmitterDef::world_space`).
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: f32,
    /// Degrees.
    pub rotation: f32,
    pub angular_velocity: f32,
    /// Remaining lifetime in seconds.
    pub energy: f32,
    pub start_energy: f32,
    pub color: Vec4,
}

/// A legacy emitter at runtime. `emit` is `ParticleEmitter.emit`: while false the live particles
/// keep animating and nothing new is emitted.
#[derive(Clone, Debug)]
pub struct LegacyParticles {
    pub def: LegacyEmitterDef,
    pub emit: bool,
    pub particles: Vec<LegacyParticle>,
    accumulator: f32,
    /// The emitter's world matrix at the previous update (new particles are spread between it and
    /// the current one).
    last_emitter: Option<Mat4>,
    random: ParticleRandom,
}

impl LegacyParticles {
    pub fn new(def: LegacyEmitterDef, seed: u32) -> LegacyParticles {
        LegacyParticles {
            emit: def.emit,
            def,
            particles: Vec::new(),
            accumulator: 0.0,
            last_emitter: None,
            random: ParticleRandom::new(seed),
        }
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.random.value()
    }

    fn signed(&mut self) -> Vec3 {
        Vec3::new(
            self.random.value() * 2.0 - 1.0,
            self.random.value() * 2.0 - 1.0,
            self.random.value() * 2.0 - 1.0,
        )
    }

    /// One frame of `dt` seconds; `emitter` is the GameObject's world matrix.
    pub fn simulate(&mut self, dt: f32, emitter: Mat4) {
        if dt <= 0.0 {
            return;
        }
        let def = self.def.clone();
        // ParticleAnimator: age, move, forces, damping, size growth, colour.
        let damping = def.damping.powf(dt);
        let rnd_force = def.rnd_force;
        for i in 0..self.particles.len() {
            let jitter = self.signed() * rnd_force;
            let p = &mut self.particles[i];
            p.energy -= dt;
            p.velocity += (def.force + jitter) * dt;
            p.velocity *= damping;
            p.position += p.velocity * dt;
            p.rotation += p.angular_velocity * dt;
            p.size += p.size * def.size_grow * dt;
        }
        self.particles.retain(|p| p.energy > 0.0);
        // Emission.
        let last = self.last_emitter.unwrap_or(emitter);
        self.last_emitter = Some(emitter);
        if self.emit {
            let count = if def.one_shot {
                self.emit = false;
                self.range(def.min_emission, def.max_emission).round()
            } else {
                self.accumulator += self.range(def.min_emission, def.max_emission) * dt;
                let whole = self.accumulator.floor();
                self.accumulator -= whole;
                whole
            } as usize;
            for i in 0..count {
                // Spread over the frame: particle i was emitted (count - i) / count of a frame ago.
                let t = (i + 1) as f32 / count as f32;
                let matrix = if def.one_shot {
                    emitter
                } else {
                    lerp_matrix(last, emitter, t)
                };
                let mut p = self.new_particle(&def, matrix);
                let late = (1.0 - t) * dt;
                p.energy -= late;
                p.position += p.velocity * late;
                if p.energy > 0.0 {
                    self.particles.push(p);
                }
            }
        }
        if def.animate_color {
            for p in &mut self.particles {
                p.color = animated_color(&def.color_animation, p.energy, p.start_energy);
            }
        }
    }

    fn new_particle(&mut self, def: &LegacyEmitterDef, emitter: Mat4) -> LegacyParticle {
        // A point in the ellipsoid, at least `min_emitter_range` (fraction of it) from the centre.
        let direction = self.random.unit_vector();
        let fraction = self.range(def.min_emitter_range, 1.0);
        let local_position = direction * def.ellipsoid * 0.5 * fraction;
        let local_velocity = def.local_velocity;
        let rnd = self.signed() * def.rnd_velocity;
        let (position, velocity) = if def.world_space {
            (
                emitter.transform_point3(local_position),
                def.world_velocity + emitter.transform_vector3(local_velocity) + rnd,
            )
        } else {
            let to_local = emitter.inverse();
            (
                local_position,
                to_local.transform_vector3(def.world_velocity) + local_velocity + rnd,
            )
        };
        let energy = self.range(def.min_energy, def.max_energy);
        let rotation = if def.rnd_rotation {
            self.random.value() * 360.0
        } else {
            0.0
        };
        let spin = self.random.value() * 2.0 - 1.0;
        LegacyParticle {
            position,
            velocity,
            size: self.range(def.min_size, def.max_size),
            rotation,
            angular_velocity: def.angular_velocity + spin * def.rnd_angular_velocity,
            energy,
            start_energy: energy,
            color: if def.animate_color {
                def.color_animation[0]
            } else {
                Vec4::ONE
            },
        }
    }

    pub fn geometry(&self, emitter: Mat4, view: &View, weapon_layer: bool) -> Option<DynamicBatch> {
        let def = &self.def;
        if !def.renderer_enabled || self.particles.is_empty() {
            return None;
        }
        let material = def.material.clone()?;
        let camera = view.position;
        let right = view.rotation * Vec3::X;
        let up = view.rotation * Vec3::Y;
        let forward = view.rotation * Vec3::Z;
        let half_tan = (view.fov_y_degrees.to_radians() * 0.5).tan();
        let mut vertices = Vec::with_capacity(self.particles.len() * 4 * DYNAMIC_STRIDE);
        let mut indices = Vec::new();
        let mut center = Vec3::ZERO;
        for p in &self.particles {
            let (position, velocity) = if def.world_space {
                (p.position, p.velocity)
            } else {
                (
                    emitter.transform_point3(p.position),
                    emitter.transform_vector3(p.velocity),
                )
            };
            center += position;
            let depth = (position - camera).dot(forward).max(1e-3);
            // `maxParticleSize`: fraction of the viewport height.
            let size = p.size.min(def.max_particle_size * 2.0 * depth * half_tan);
            if size <= 0.0 {
                continue;
            }
            let base = (vertices.len() / DYNAMIC_STRIDE) as u32;
            let (axis_x, axis_y, normal) = match def.render_mode {
                LegacyRenderMode::Billboard => (right, up, -forward),
                LegacyRenderMode::Vertical => {
                    let flat = Vec3::new(right.x, 0.0, right.z).normalize_or(Vec3::X);
                    (flat, Vec3::Y, flat.cross(Vec3::Y))
                }
                LegacyRenderMode::Horizontal => (Vec3::X, Vec3::Z, Vec3::Y),
                LegacyRenderMode::Stretch => {
                    let speed = velocity.length();
                    let axis = if speed > 1e-5 { velocity / speed } else { up };
                    let to_camera = (camera - position).normalize_or(forward);
                    let side = axis.cross(to_camera).normalize_or(right);
                    let length = size * def.length_scale + speed * def.velocity_scale;
                    let half = side * size * 0.5;
                    let tail = position - axis * length;
                    let c = p.color;
                    push_vertex(&mut vertices, tail - half, to_camera, Vec2::ZERO, c);
                    push_vertex(&mut vertices, tail + half, to_camera, Vec2::Y, c);
                    push_vertex(&mut vertices, position + half, to_camera, Vec2::ONE, c);
                    push_vertex(&mut vertices, position - half, to_camera, Vec2::X, c);
                    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
                    continue;
                }
            };
            let (s, c) = (-p.rotation.to_radians()).sin_cos();
            let corner = |x: f32, y: f32| {
                let (rx, ry) = (x * c - y * s, x * s + y * c);
                position + (axis_x * rx + axis_y * ry) * size
            };
            for (x, y) in [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
                push_vertex(
                    &mut vertices,
                    corner(x, y),
                    normal,
                    Vec2::new(x + 0.5, y + 0.5),
                    p.color,
                );
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
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

/// `ParticleAnimator.colorAnimation`: five keys spread evenly over the particle's life.
fn animated_color(keys: &[Vec4; 5], energy: f32, start_energy: f32) -> Vec4 {
    let age = if start_energy > 0.0 {
        (1.0 - energy / start_energy).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let t = age * 4.0;
    let index = (t.floor() as usize).min(3);
    keys[index].lerp(keys[index + 1], t - index as f32)
}

fn lerp_matrix(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(sa.lerp(sb, t), ra.slerp(rb, t), ta.lerp(tb, t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The shotgun's "Smoke Trail" (CrazyGames `german` prefab).
    fn smoke() -> LegacyParticles {
        let zero = json!({ "x": 0.0, "y": 0.0, "z": 0.0 });
        let def = LegacyEmitterDef::from_components(
            &json!({
                "m_Enabled": true, "m_Emit": true, "minSize": 0.3, "maxSize": 0.3,
                "minEnergy": 2.0, "maxEnergy": 2.0, "minEmission": 4.0, "maxEmission": 4.0,
                "worldVelocity": zero, "localVelocity": zero, "rndVelocity": zero,
                "emitterVelocityScale": 0.0, "tangentVelocity": zero, "angularVelocity": 0.0,
                "rndAngularVelocity": 0.0, "rndRotation": true, "Simulate in Worldspace?": true,
                "m_OneShot": false, "m_Ellipsoid": zero, "m_MinEmitterRange": 0.0,
            }),
            Some(&json!({
                "Does Animate Color?": true,
                "colorAnimation[0]": { "rgba": 2323711 },
                "colorAnimation[1]": { "rgba": 2789425987u32 },
                "colorAnimation[2]": { "rgba": 2051228483u32 },
                "colorAnimation[3]": { "rgba": 1413167931 },
                "colorAnimation[4]": { "rgba": 2960685 },
                "worldRotationAxis": zero, "localRotationAxis": zero, "sizeGrow": 0.8895081,
                "rndForce": zero, "force": zero, "damping": 1.0, "stopSimulation": false,
                "autodestruct": false,
            })),
            Some(&json!({
                "m_Enabled": true, "m_Materials": [{ "ref": "resources.assets:57" }],
                "m_StretchParticles": 5, "m_LengthScale": 0.81, "m_VelocityScale": -1.46,
                "m_MaxParticleSize": 1.1,
            })),
        );
        LegacyParticles::new(def, 3)
    }

    #[test]
    fn smoke_emits_while_on_and_fades_out() {
        let mut smoke = smoke();
        assert_eq!(smoke.def.render_mode, LegacyRenderMode::Vertical);
        // `shotGunAwake`: emit = false.
        smoke.emit = false;
        smoke.simulate(0.02, Mat4::IDENTITY);
        assert!(smoke.particles.is_empty());
        // `shotGunSmokeEffect`: emit for 0.3 s at 4 per second → 1 particle.
        smoke.emit = true;
        for _ in 0..15 {
            smoke.simulate(0.02, Mat4::IDENTITY);
        }
        smoke.emit = false;
        assert_eq!(smoke.particles.len(), 1);
        let p = &smoke.particles[0];
        // Born at the start of the colour animation (alpha 0) and growing.
        assert!(p.size > 0.3 && p.size < 0.35, "{}", p.size);
        assert!((0.0..360.0).contains(&p.rotation));
        // Half-way through its 2 s life: the middle key.
        let mid = animated_color(&smoke.def.color_animation, 1.0, 2.0);
        assert!((mid.w - 0x7A as f32 / 255.0).abs() < 1e-6);
        for _ in 0..100 {
            smoke.simulate(0.02, Mat4::IDENTITY);
        }
        assert!(smoke.particles.is_empty());
    }
}
