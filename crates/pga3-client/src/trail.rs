//! Unity `TrailRenderer` (2018.3; the 5.3 start/end width and five colours are converted) for the bullets' and the rocket's trails (spec:
//! `docs/gameplay/particles.md`): view-aligned strip, stretched texture, width curve and colour
//! gradient from the head (0) to the tail (1).

use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{f32_of, ref_of};
use crate::particles::{Curve, Gradient};
use crate::render::{DYNAMIC_STRIDE, DynamicBatch, View};

#[derive(Clone, Debug)]
pub struct TrailDef {
    /// `m_Time`: lifetime of a point in seconds.
    pub time: f32,
    pub min_vertex_distance: f32,
    pub width: Curve,
    pub width_multiplier: f32,
    pub color: Gradient,
    pub material: Option<String>,
    pub enabled: bool,
    pub emitting: bool,
}

impl TrailDef {
    /// From a `TrailRenderer` component's data (Unity 2018.3 or 5.3 layout).
    pub fn from_component(data: &Value) -> TrailDef {
        let p = &data["m_Parameters"];
        let (width, width_multiplier, color) = if data.get("m_StartWidth").is_some() {
            unity_5_3_width_and_color(data)
        } else {
            (
                Curve::from_json(&p["widthCurve"]),
                f32_of(&p["widthMultiplier"]),
                Gradient::from_json(&p["colorGradient"]),
            )
        };
        TrailDef {
            time: f32_of(&data["m_Time"]),
            min_vertex_distance: f32_of(&data["m_MinVertexDistance"]),
            width,
            width_multiplier,
            color,
            material: data["m_Materials"][0]
                .as_object()
                .and_then(|_| ref_of(&data["m_Materials"][0]))
                .map(|(r, _)| r.to_string()),
            enabled: data["m_Enabled"].as_bool().unwrap_or(false),
            emitting: data["m_Emitting"].as_bool().unwrap_or(true),
        }
    }
}

/// Unity 5.3 trail (CrazyGames build): `m_StartWidth` / `m_EndWidth` at the head / tail and five
/// `m_Colors` (`ColorRGBA32`) spread evenly from the head (0) to the tail (1), both interpolated
/// linearly. As a width curve (multiplier 1) and a gradient with the key times Unity's upgrade
/// writes (0, 16384, 32768, 49151, 65535 of 65535).
fn unity_5_3_width_and_color(data: &Value) -> (Curve, f32, Gradient) {
    let start = f32_of(&data["m_StartWidth"]);
    let end = f32_of(&data["m_EndWidth"]);
    let slope = end - start;
    let width = Curve::from_json(&serde_json::json!({ "m_Curve": [
        { "time": 0.0, "value": start, "inSlope": slope, "outSlope": slope },
        { "time": 1.0, "value": end, "inSlope": slope, "outSlope": slope },
    ] }));
    let mut gradient = serde_json::json!({ "m_NumColorKeys": 5, "m_NumAlphaKeys": 5, "m_Mode": 0 });
    for (i, time) in [0, 16384, 32768, 49151, 65535].into_iter().enumerate() {
        let rgba = data["m_Colors"][format!("m_Color[{i}]")]["rgba"]
            .as_u64()
            .unwrap_or(0xFFFF_FFFF);
        let channel = |c: u32| ((rgba >> (8 * c)) & 0xFF) as f64 / 255.0;
        gradient[format!("key{i}")] = serde_json::json!({
            "r": channel(0), "g": channel(1), "b": channel(2), "a": channel(3),
        });
        gradient[format!("ctime{i}")] = Value::from(time);
        gradient[format!("atime{i}")] = Value::from(time);
    }
    (width, 1.0, Gradient::from_json(&gradient))
}

/// Runtime trail of one object.
#[derive(Clone, Debug)]
pub struct Trail {
    pub def: Arc<TrailDef>,
    /// Committed points (oldest first) with the time they were added.
    points: Vec<(Vec3, f32)>,
    /// The transform's current position (the trail's head).
    head: Vec3,
}

impl Trail {
    /// The trail starts with a point at the spawn position.
    pub fn new(def: Arc<TrailDef>, position: Vec3, time: f32) -> Trail {
        Trail {
            def,
            points: vec![(position, time)],
            head: position,
        }
    }

    /// End of frame: expire old points, commit a new one once the head is more than
    /// `minVertexDistance` from the newest.
    pub fn update(&mut self, position: Vec3, time: f32) {
        let lifetime = self.def.time;
        self.points.retain(|(_, t)| time - t <= lifetime);
        self.head = position;
        let far = self
            .points
            .last()
            .is_none_or(|(p, _)| p.distance(position) > self.def.min_vertex_distance);
        if self.def.emitting && far {
            self.points.push((position, time));
        }
    }

    pub fn geometry(&self, view: &View, weapon_layer: bool) -> Option<DynamicBatch> {
        let def = &self.def;
        let material = def.material.clone()?;
        if !def.enabled {
            return None;
        }
        // Head first.
        let mut line: Vec<Vec3> = vec![self.head];
        for (p, _) in self.points.iter().rev() {
            if line.last().is_some_and(|last| last.distance(*p) > 1e-5) {
                line.push(*p);
            }
        }
        if line.len() < 2 {
            return None;
        }
        let mut lengths = vec![0.0f32];
        for pair in line.windows(2) {
            let last = *lengths.last().unwrap_or(&0.0);
            lengths.push(last + pair[0].distance(pair[1]));
        }
        let total = lengths.last().copied().unwrap_or(0.0).max(1e-6);
        let mut vertices = Vec::with_capacity(line.len() * 2 * DYNAMIC_STRIDE);
        let mut indices = Vec::new();
        for (i, p) in line.iter().enumerate() {
            let prev = line[i.saturating_sub(1)];
            let next = line[(i + 1).min(line.len() - 1)];
            let tangent = (prev - next).normalize_or(Vec3::Z);
            let to_camera = (view.position - *p).normalize_or(Vec3::Y);
            let side = tangent.cross(to_camera).normalize_or(Vec3::X);
            let t = lengths[i] / total;
            let half = def.width.evaluate(t) * def.width_multiplier * 0.5;
            let color = def.color.evaluate(t);
            for (offset, v) in [(half, 0.0), (-half, 1.0)] {
                let q = *p + side * offset;
                push(&mut vertices, q, to_camera, Vec2::new(t, v), color);
            }
            if i > 0 {
                let base = (i as u32 - 1) * 2;
                indices.extend([base, base + 1, base + 3, base, base + 3, base + 2]);
            }
        }
        let center = line.iter().copied().sum::<Vec3>() / line.len() as f32;
        Some(DynamicBatch {
            material,
            vertices,
            indices,
            weapon_layer,
            distance: center.distance(view.position),
        })
    }
}

fn push(out: &mut Vec<f32>, p: Vec3, n: Vec3, uv: Vec2, c: Vec4) {
    out.extend_from_slice(&[p.x, p.y, p.z, n.x, n.y, n.z, uv.x, uv.y, c.x, c.y, c.z, c.w]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Quat;
    use serde_json::json;

    fn def() -> Arc<TrailDef> {
        Arc::new(TrailDef::from_component(&json!({
            "m_Enabled": true,
            "m_Time": 0.1,
            "m_MinVertexDistance": 0.1,
            "m_Emitting": true,
            "m_Materials": [{ "ref": "resources.assets:50" }],
            "m_Parameters": {
                "widthMultiplier": 1.0,
                "widthCurve": { "m_Curve": [
                    { "time": 0.0, "value": 0.2, "inSlope": 0.0, "outSlope": 0.0 },
                    { "time": 1.0, "value": 0.0, "inSlope": 0.0, "outSlope": 0.0 },
                ] },
                "colorGradient": {
                    "key0": { "r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0 },
                    "key1": { "r": 1.0, "g": 1.0, "b": 1.0, "a": 0.0 },
                    "ctime0": 0, "ctime1": 65535, "atime0": 0, "atime1": 65535,
                    "m_Mode": 0, "m_NumColorKeys": 2, "m_NumAlphaKeys": 2,
                },
            },
        })))
    }

    /// The CrazyGames rocket trail (`Roket cube__11017`, Unity 5.3 layout).
    #[test]
    fn unity_5_3_trail_widths_and_colors() {
        let def = TrailDef::from_component(&json!({
            "m_Enabled": true,
            "m_Time": 1.0,
            "m_StartWidth": 0.25,
            "m_EndWidth": 0.1,
            "m_Colors": {
                "m_Color[0]": { "rgba": 4294967295u32 },
                "m_Color[1]": { "rgba": 4294967295u32 },
                "m_Color[2]": { "rgba": 4294967295u32 },
                "m_Color[3]": { "rgba": 4294967295u32 },
                "m_Color[4]": { "rgba": 16777215 },
            },
            "m_MinVertexDistance": 0.1,
            "m_Autodestruct": true,
        }));
        assert!(def.emitting);
        assert_eq!(def.width_multiplier, 1.0);
        assert!((def.width.evaluate(0.0) - 0.25).abs() < 1e-6);
        assert!((def.width.evaluate(0.5) - 0.175).abs() < 1e-6);
        assert!((def.width.evaluate(1.0) - 0.1).abs() < 1e-6);
        assert_eq!(def.color.evaluate(0.5), Vec4::ONE);
        // Alpha fades over the last quarter.
        assert!((def.color.evaluate(0.875).w - 0.5).abs() < 1e-3);
        assert_eq!(def.color.evaluate(1.0), Vec4::new(1.0, 1.0, 1.0, 0.0));
    }

    #[test]
    fn points_expire_and_strip_faces_camera() {
        let mut trail = Trail::new(def(), Vec3::ZERO, 0.0);
        for i in 1..=10 {
            trail.update(Vec3::Z * i as f32 * 0.5, i as f32 * 0.02);
        }
        // Points older than 0.1 s are gone: those from 0.1 s .. 0.2 s remain (6), plus none
        // duplicated at the head.
        assert_eq!(trail.points.len(), 6);
        let view = View {
            position: Vec3::new(5.0, 0.0, 2.5),
            rotation: Quat::IDENTITY,
            fov_y_degrees: 60.0,
            near: 0.1,
            far: 100.0,
        };
        let batch = trail.geometry(&view, false).unwrap();
        assert_eq!(batch.vertices.len(), 6 * 2 * DYNAMIC_STRIDE);
        // Head width 0.2 across Y (perpendicular to the trail and to the camera direction).
        let (a, b) = (
            &batch.vertices[0..3],
            &batch.vertices[DYNAMIC_STRIDE..DYNAMIC_STRIDE + 3],
        );
        assert!(((a[1] - b[1]).abs() - 0.2).abs() < 1e-4, "{a:?} {b:?}");
        // Alpha fades to 0 at the tail.
        let last = batch.vertices.len() - DYNAMIC_STRIDE;
        assert!(batch.vertices[last + 11].abs() < 1e-5);
    }
}
