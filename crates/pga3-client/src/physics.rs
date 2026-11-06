//! Static collision world and a Unity `CharacterController` equivalent.
//!
//! The original runs PhysX 3.x through Unity 2018.3. The map geometry is non-convex MeshColliders
//! (converted Source BSP) plus a few BoxColliders, all static, so the world is a triangle soup in a
//! uniform grid. Sweeps are exact capsule-vs-triangle time-of-impact tests (no substepping).
//!
//! The controller follows PhysX's character controller structure (`SweepTest::moveCharacter`):
//! the motion is split into an up pass (plus the step offset), a side pass (collide and slide) and a
//! down pass (minus the step offset), each sweeping the capsule inflated by the skin width (PhysX
//! contact offset). See `docs/gameplay/movement.md` for which parts are observed and which are
//! inferred and still need measuring against the original.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use glam::{Mat4, Quat, Vec3};

use crate::obj::parse_obj;
use crate::scene::{ColliderShape, Scene};

/// Gap kept between a swept shape and what it hit, so the next sweep starts outside it.
const CONTACT_EPSILON: f32 = 1e-4;
/// Grid cell size in metres.
const CELL: f32 = 4.0;

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    /// Unit front-face normal. Unity front faces are clockwise in its left-handed space, which
    /// makes `cross(b - a, c - a)` point out of the front side.
    pub normal: Vec3,
}

impl Triangle {
    fn new(a: Vec3, b: Vec3, c: Vec3) -> Option<Triangle> {
        let normal = (b - a).cross(c - a);
        let length = normal.length();
        if length < 1e-12 || !length.is_finite() {
            return None;
        }
        Some(Triangle {
            a,
            b,
            c,
            normal: normal / length,
        })
    }

    fn edges(&self) -> [(Vec3, Vec3); 3] {
        [(self.a, self.b), (self.b, self.c), (self.c, self.a)]
    }
}

/// Which colliders a query sees.
#[derive(Clone, Copy, PartialEq)]
pub enum Filter {
    /// Non-trigger colliders only (the CharacterController never collides with triggers).
    Solid,
    /// Triggers too (`Physics.queriesHitTriggers` is on in this build's PhysicsManager).
    All,
}

/// What a sweep does with triangles the shape already overlaps at its start position.
#[derive(Clone, Copy, PartialEq)]
pub enum InitialOverlap {
    /// Report a hit at distance 0 if the motion goes further in (controller sweeps).
    Block,
    /// Skip them, as Unity's `Physics.SphereCast` does.
    Ignore,
}

#[derive(Clone, Copy, Debug)]
pub struct Capsule {
    pub p0: Vec3,
    pub p1: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SweepHit {
    pub distance: f32,
    /// Surface normal at the contact, pointing from the geometry towards the swept shape.
    pub normal: Vec3,
    /// Contact point on the geometry.
    pub point: Vec3,
}

pub struct CollisionWorld {
    triangles: Vec<Triangle>,
    trigger: Vec<bool>,
    cells: HashMap<[i32; 3], Vec<u32>>,
}

fn cell_of(p: Vec3) -> [i32; 3] {
    [
        (p.x / CELL).floor() as i32,
        (p.y / CELL).floor() as i32,
        (p.z / CELL).floor() as i32,
    ]
}

impl CollisionWorld {
    pub fn new() -> CollisionWorld {
        CollisionWorld {
            triangles: Vec::new(),
            trigger: Vec::new(),
            cells: HashMap::new(),
        }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub fn add_triangle(&mut self, triangle: Triangle, is_trigger: bool) {
        let index = self.triangles.len() as u32;
        let min = cell_of(triangle.a.min(triangle.b).min(triangle.c));
        let max = cell_of(triangle.a.max(triangle.b).max(triangle.c));
        for x in min[0]..=max[0] {
            for y in min[1]..=max[1] {
                for z in min[2]..=max[2] {
                    self.cells.entry([x, y, z]).or_default().push(index);
                }
            }
        }
        self.triangles.push(triangle);
        self.trigger.push(is_trigger);
    }

    /// Mesh triangles transformed to world space. A mirroring transform reverses the winding, so
    /// the corners are swapped back to keep the front face outside (as PhysX does).
    pub fn add_mesh(&mut self, world: Mat4, positions: &[Vec3], indices: &[u32], trigger: bool) {
        let mirrored = world.determinant() < 0.0;
        let points: Vec<Vec3> = positions
            .iter()
            .map(|p| world.transform_point3(*p))
            .collect();
        for corners in indices.as_chunks::<3>().0 {
            let [a, b, c] = corners.map(|i| points[i as usize]);
            let triangle = if mirrored {
                Triangle::new(a, c, b)
            } else {
                Triangle::new(a, b, c)
            };
            if let Some(triangle) = triangle {
                self.add_triangle(triangle, trigger);
            }
        }
    }

    pub fn add_box(&mut self, world: Mat4, center: Vec3, size: Vec3, trigger: bool) {
        let half = size * 0.5;
        let corner = |i: usize| {
            let sign = Vec3::new(
                if i & 1 == 0 { -1.0 } else { 1.0 },
                if i & 2 == 0 { -1.0 } else { 1.0 },
                if i & 4 == 0 { -1.0 } else { 1.0 },
            );
            world.transform_point3(center + half * sign)
        };
        let middle = world.transform_point3(center);
        const FACES: [[usize; 4]; 6] = [
            [0, 1, 3, 2],
            [4, 6, 7, 5],
            [0, 4, 5, 1],
            [2, 3, 7, 6],
            [0, 2, 6, 4],
            [1, 5, 7, 3],
        ];
        for face in FACES {
            for [i, j, k] in [[face[0], face[1], face[2]], [face[0], face[2], face[3]]] {
                let (a, b, c) = (corner(i), corner(j), corner(k));
                let Some(mut triangle) = Triangle::new(a, b, c) else {
                    continue;
                };
                // Outward normals, whatever the transform's handedness.
                if triangle.normal.dot((a + b + c) / 3.0 - middle) < 0.0 {
                    triangle = Triangle {
                        a,
                        b: c,
                        c: b,
                        normal: -triangle.normal,
                    };
                }
                self.add_triangle(triangle, trigger);
            }
        }
    }

    /// Triangle indices whose grid cells overlap the box.
    fn candidates(&self, min: Vec3, max: Vec3, filter: Filter) -> Vec<u32> {
        let (lo, hi) = (cell_of(min), cell_of(max));
        let mut out = Vec::new();
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    if let Some(list) = self.cells.get(&[x, y, z]) {
                        out.extend_from_slice(list);
                    }
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        if filter == Filter::Solid {
            out.retain(|&i| !self.trigger[i as usize]);
        }
        out
    }

    /// First hit of `capsule` moved along unit `direction` up to `max_distance`. Back faces are
    /// culled (`Physics.queriesHitBackfaces` is off; PhysX controller sweeps cull them too).
    pub fn sweep_capsule(
        &self,
        capsule: &Capsule,
        direction: Vec3,
        max_distance: f32,
        filter: Filter,
        overlap: InitialOverlap,
    ) -> Option<SweepHit> {
        let end = Capsule {
            p0: capsule.p0 + direction * max_distance,
            p1: capsule.p1 + direction * max_distance,
            radius: capsule.radius,
        };
        let pad = Vec3::splat(capsule.radius + CONTACT_EPSILON);
        let min = capsule.p0.min(capsule.p1).min(end.p0).min(end.p1) - pad;
        let max = capsule.p0.max(capsule.p1).max(end.p0).max(end.p1) + pad;
        let mut best: Option<SweepHit> = None;
        for index in self.candidates(min, max, filter) {
            let triangle = &self.triangles[index as usize];
            if triangle.normal.dot(direction) >= 0.0 {
                continue;
            }
            let hit = sweep_capsule_triangle(capsule, direction, max_distance, triangle, overlap);
            if let Some(hit) = hit
                && best.is_none_or(|b| hit.distance < b.distance)
            {
                best = Some(hit);
            }
        }
        best
    }

    /// `Physics.SphereCast`: colliders the sphere starts inside are not reported.
    pub fn sphere_cast(
        &self,
        origin: Vec3,
        radius: f32,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<SweepHit> {
        let capsule = Capsule {
            p0: origin,
            p1: origin,
            radius,
        };
        self.sweep_capsule(
            &capsule,
            direction,
            max_distance,
            Filter::All,
            InitialOverlap::Ignore,
        )
    }

    /// Deepest penetration of the capsule into front-facing solid triangles: (depth, push-out
    /// direction).
    fn deepest_penetration(&self, capsule: &Capsule) -> Option<(f32, Vec3)> {
        let pad = Vec3::splat(capsule.radius);
        let min = capsule.p0.min(capsule.p1) - pad;
        let max = capsule.p0.max(capsule.p1) + pad;
        let mut best: Option<(f32, Vec3)> = None;
        for index in self.candidates(min, max, Filter::Solid) {
            let triangle = &self.triangles[index as usize];
            let (on_segment, on_triangle) =
                closest_segment_triangle(capsule.p0, capsule.p1, triangle);
            if triangle.normal.dot(on_segment - triangle.a) <= 0.0 {
                continue;
            }
            let offset = on_segment - on_triangle;
            let distance = offset.length();
            let depth = capsule.radius - distance;
            if depth <= 0.0 {
                continue;
            }
            let normal = if distance > 1e-6 {
                offset / distance
            } else {
                triangle.normal
            };
            if best.is_none_or(|(d, _)| depth > d) {
                best = Some((depth, normal));
            }
        }
        best
    }
}

/// Builds the collision world of a scene: every enabled collider on an active GameObject.
pub fn build_world(assets: &Path, scene: &Scene) -> Result<CollisionWorld> {
    let mut world = CollisionWorld::new();
    let mut meshes: HashMap<&str, (Vec<Vec3>, Vec<u32>)> = HashMap::new();
    for collider in &scene.colliders {
        match &collider.shape {
            ColliderShape::Mesh { mesh_asset } => {
                if !meshes.contains_key(mesh_asset.as_str()) {
                    let path = assets.join(mesh_asset);
                    let text = std::fs::read_to_string(&path)
                        .with_context(|| format!("reading {}", path.display()))?;
                    let mesh = parse_obj(&text).with_context(|| format!("parsing {mesh_asset}"))?;
                    let positions = mesh.positions.iter().map(|p| Vec3::from(*p)).collect();
                    let indices = mesh.submeshes.concat();
                    meshes.insert(mesh_asset, (positions, indices));
                }
                let (positions, indices) = &meshes[mesh_asset.as_str()];
                world.add_mesh(collider.world, positions, indices, collider.is_trigger);
            }
            ColliderShape::Box { center, size } => {
                world.add_box(collider.world, *center, *size, collider.is_trigger)
            }
        }
    }
    Ok(world)
}

// ---------------------------------------------------------------------------------------------
// Geometry primitives

/// Closest point on triangle `abc` to `p` (Ericson, Real-Time Collision Detection 5.1.5).
fn closest_point_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    a + ab * (vb * denom) + ac * (vc * denom)
}

/// Closest points of segments `p1q1` and `p2q2` (Ericson 5.1.9).
fn closest_segments(p1: Vec3, q1: Vec3, p2: Vec3, q2: Vec3) -> (Vec3, Vec3) {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r = p1 - p2;
    let a = d1.dot(d1);
    let e = d2.dot(d2);
    let f = d2.dot(r);
    let (s, t);
    if a <= 1e-12 && e <= 1e-12 {
        return (p1, p2);
    }
    if a <= 1e-12 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = d1.dot(r);
        if e <= 1e-12 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = d1.dot(d2);
            let denom = a * e - b * b;
            let mut s0 = if denom > 1e-12 {
                ((b * f - c * e) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut t0 = (b * s0 + f) / e;
            if t0 < 0.0 {
                t0 = 0.0;
                s0 = (-c / a).clamp(0.0, 1.0);
            } else if t0 > 1.0 {
                t0 = 1.0;
                s0 = ((b - c) / a).clamp(0.0, 1.0);
            }
            s = s0;
            t = t0;
        }
    }
    (p1 + d1 * s, p2 + d2 * t)
}

fn inside_triangle(p: Vec3, triangle: &Triangle) -> bool {
    let n = triangle.normal;
    let tolerance = -1e-6;
    (triangle.b - triangle.a).cross(p - triangle.a).dot(n) >= tolerance
        && (triangle.c - triangle.b).cross(p - triangle.b).dot(n) >= tolerance
        && (triangle.a - triangle.c).cross(p - triangle.c).dot(n) >= tolerance
}

/// Closest points between segment `p0p1` and a triangle: (on segment, on triangle).
fn closest_segment_triangle(p0: Vec3, p1: Vec3, triangle: &Triangle) -> (Vec3, Vec3) {
    // The segment crossing the triangle gives distance 0.
    let s0 = triangle.normal.dot(p0 - triangle.a);
    let s1 = triangle.normal.dot(p1 - triangle.a);
    if (s0 <= 0.0) != (s1 <= 0.0) {
        let point = p0 + (p1 - p0) * (s0 / (s0 - s1));
        if inside_triangle(point, triangle) {
            return (point, point);
        }
    }
    let mut best = {
        let q = closest_point_triangle(p0, triangle.a, triangle.b, triangle.c);
        (p0, q, (p0 - q).length_squared())
    };
    let q = closest_point_triangle(p1, triangle.a, triangle.b, triangle.c);
    if (p1 - q).length_squared() < best.2 {
        best = (p1, q, (p1 - q).length_squared());
    }
    for (u, v) in triangle.edges() {
        let (on_segment, on_edge) = closest_segments(p0, p1, u, v);
        let d = (on_segment - on_edge).length_squared();
        if d < best.2 {
            best = (on_segment, on_edge, d);
        }
    }
    (best.0, best.1)
}

/// Unit-direction ray vs sphere; `None` when starting inside or moving away.
fn ray_sphere(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let m = origin - center;
    let b = m.dot(direction);
    let c = m.dot(m) - radius * radius;
    if c < 0.0 || b >= 0.0 {
        return None;
    }
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    Some((-b - discriminant.sqrt()).max(0.0))
}

/// Unit-direction ray vs the side of the finite cylinder `a..b` (no caps; the sphere tests cover
/// the ends). `None` when starting inside or moving away.
fn ray_cylinder(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, radius: f32) -> Option<f32> {
    let axis = b - a;
    let length_squared = axis.length_squared();
    if length_squared < 1e-12 {
        return None;
    }
    let m = origin - a;
    let m_axis = m.dot(axis) / length_squared;
    let d_axis = direction.dot(axis) / length_squared;
    let m_perp = m - axis * m_axis;
    let d_perp = direction - axis * d_axis;
    let qa = d_perp.dot(d_perp);
    if qa < 1e-12 {
        return None;
    }
    let qb = m_perp.dot(d_perp);
    let qc = m_perp.dot(m_perp) - radius * radius;
    if qc < 0.0 || qb >= 0.0 {
        return None;
    }
    let discriminant = qb * qb - qa * qc;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-qb - discriminant.sqrt()) / qa;
    let s = m_axis + d_axis * t;
    if !(0.0..=1.0).contains(&s) {
        return None;
    }
    Some(t.max(0.0))
}

/// Sphere at `center` swept along `direction` against the triangle's face, edges and vertices.
fn sweep_sphere_triangle(
    center: Vec3,
    radius: f32,
    direction: Vec3,
    triangle: &Triangle,
) -> Option<SweepHit> {
    let mut best: Option<SweepHit> = None;
    let mut consider = |hit: SweepHit| {
        if best.is_none_or(|b| hit.distance < b.distance) {
            best = Some(hit);
        }
    };
    let n = triangle.normal;
    let approach = n.dot(direction);
    let s0 = n.dot(center - triangle.a);
    if approach < 0.0 && s0 >= radius {
        let t = (s0 - radius) / -approach;
        let point = center + direction * t - n * radius;
        if inside_triangle(point, triangle) {
            consider(SweepHit {
                distance: t,
                normal: n,
                point,
            });
        }
    }
    for (u, v) in triangle.edges() {
        if let Some(t) = ray_cylinder(center, direction, u, v, radius) {
            let moved = center + direction * t;
            let (_, point) = closest_segments(moved, moved, u, v);
            consider(SweepHit {
                distance: t,
                normal: (moved - point).normalize_or(n),
                point,
            });
        }
    }
    for vertex in [triangle.a, triangle.b, triangle.c] {
        if let Some(t) = ray_sphere(center, direction, vertex, radius) {
            let moved = center + direction * t;
            consider(SweepHit {
                distance: t,
                normal: (moved - vertex).normalize_or(n),
                point: vertex,
            });
        }
    }
    best
}

/// Exact time of impact of a capsule swept against one triangle. Every first contact is between
/// a segment end and the triangle (sphere sweep), a triangle vertex and the segment's side, or a
/// triangle edge and the segment's interior (line-line distance reaching the radius).
fn sweep_capsule_triangle(
    capsule: &Capsule,
    direction: Vec3,
    max_distance: f32,
    triangle: &Triangle,
    overlap: InitialOverlap,
) -> Option<SweepHit> {
    let radius = capsule.radius;
    let (on_segment, on_triangle) = closest_segment_triangle(capsule.p0, capsule.p1, triangle);
    let offset = on_segment - on_triangle;
    if offset.length_squared() < radius * radius {
        return match overlap {
            InitialOverlap::Ignore => None,
            InitialOverlap::Block => {
                let normal = offset.normalize_or(triangle.normal);
                (normal.dot(direction) < -1e-4).then_some(SweepHit {
                    distance: 0.0,
                    normal,
                    point: on_triangle,
                })
            }
        };
    }

    let mut best: Option<SweepHit> = None;
    let mut consider = |hit: SweepHit| {
        if hit.distance <= max_distance && best.is_none_or(|b| hit.distance < b.distance) {
            best = Some(hit);
        }
    };
    for end in [capsule.p0, capsule.p1] {
        if let Some(hit) = sweep_sphere_triangle(end, radius, direction, triangle) {
            consider(hit);
        }
        if capsule.p0 == capsule.p1 {
            break;
        }
    }
    if capsule.p0 != capsule.p1 {
        let segment = capsule.p1 - capsule.p0;
        for vertex in [triangle.a, triangle.b, triangle.c] {
            if let Some(t) = ray_cylinder(vertex, -direction, capsule.p0, capsule.p1, radius) {
                let (on_moved, _) = closest_segments(
                    capsule.p0 + direction * t,
                    capsule.p1 + direction * t,
                    vertex,
                    vertex,
                );
                consider(SweepHit {
                    distance: t,
                    normal: (on_moved - vertex).normalize_or(triangle.normal),
                    point: vertex,
                });
            }
        }
        for (u, v) in triangle.edges() {
            let edge = v - u;
            let cross = segment.cross(edge);
            let length = cross.length();
            if length < 1e-9 {
                continue;
            }
            let mut axis = cross / length;
            let mut separation = axis.dot(capsule.p0 - u);
            if separation < 0.0 {
                axis = -axis;
                separation = -separation;
            }
            let closing = axis.dot(direction);
            if closing >= 0.0 || separation < radius {
                continue;
            }
            let t = (separation - radius) / -closing;
            // Both closest points must be interior; otherwise an end/vertex case applies.
            let r = capsule.p0 + direction * t - u;
            let (a, b, c, e, f) = (
                segment.dot(segment),
                segment.dot(edge),
                segment.dot(r),
                edge.dot(edge),
                edge.dot(r),
            );
            let denom = a * e - b * b;
            if denom <= 1e-12 {
                continue;
            }
            let s = (b * f - c * e) / denom;
            let w = (a * f - b * c) / denom;
            if !(0.0..=1.0).contains(&s) || !(0.0..=1.0).contains(&w) {
                continue;
            }
            consider(SweepHit {
                distance: t,
                normal: axis,
                point: u + edge * w,
            });
        }
    }
    best
}

// ---------------------------------------------------------------------------------------------
// CharacterController

pub const COLLIDED_SIDES: u8 = 1;
pub const COLLIDED_ABOVE: u8 = 2;
pub const COLLIDED_BELOW: u8 = 4;

/// Unity `ControllerColliderHit` (the fields the game reads).
#[derive(Clone, Copy, Debug)]
pub struct ControllerHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub move_direction: Vec3,
}

#[derive(Clone, Debug)]
pub struct CharacterController {
    pub height: f32,
    pub radius: f32,
    /// Local offset of the capsule centre from the transform position.
    pub center: Vec3,
    pub skin_width: f32,
    pub step_offset: f32,
    pub slope_limit: f32,
    pub min_move_distance: f32,
}

/// Side-pass iteration limit (PhysX `SweepTest::mMaxIter`; inferred value).
const MAX_SIDE_ITERATIONS: usize = 10;

impl CharacterController {
    fn capsule(&self, center: Vec3, inflate: f32) -> Capsule {
        // Unity clamps the height to at least the diameter (a sphere).
        let half = (self.height * 0.5 - self.radius).max(0.0);
        Capsule {
            p0: center - Vec3::Y * half,
            p1: center + Vec3::Y * half,
            radius: self.radius + inflate,
        }
    }

    fn world_center(&self, position: Vec3, rotation: Quat) -> Vec3 {
        position + rotation * self.center
    }

    /// `CharacterController.Move`. Moves `position` and returns the collision flags; every
    /// contact is appended to `hits` (Unity calls `OnControllerColliderHit` for each).
    pub fn move_by(
        &self,
        world: &CollisionWorld,
        position: &mut Vec3,
        rotation: Quat,
        motion: Vec3,
        hits: &mut Vec<ControllerHit>,
    ) -> u8 {
        if motion.length() < self.min_move_distance {
            return 0;
        }
        let mut center = self.world_center(*position, rotation);
        self.recover_from_overlap(world, &mut center);
        let start = center;
        let first_hit = hits.len();

        // Inferred: no auto-step while moving up (jumping).
        let step = if motion.y <= 0.0 {
            self.step_offset
        } else {
            0.0
        };
        let (mut flags, landed) = self.move_passes(world, &mut center, motion, step, hits);
        // Step validation (PhysX-style, inferred): landing on something higher than the step
        // offset above the original bottom means the step was not allowed; redo the move
        // without stepping.
        let original_bottom = self.capsule(start, 0.0).p0.y - self.radius;
        if step > 0.0
            && let Some(contact_height) = landed
            && contact_height > original_bottom + step
        {
            center = start;
            hits.truncate(first_hit);
            (flags, _) = self.move_passes(world, &mut center, motion, 0.0, hits);
        }
        *position = center - rotation * self.center;
        flags
    }

    /// Up, side and down passes. Returns the flags and, if the down pass hit something, the
    /// height of its contact point.
    fn move_passes(
        &self,
        world: &CollisionWorld,
        center: &mut Vec3,
        motion: Vec3,
        mut step: f32,
        hits: &mut Vec<ControllerHit>,
    ) -> (u8, Option<f32>) {
        let side = Vec3::new(motion.x, 0.0, motion.z);
        let mut flags = 0;
        let up = motion.y.max(0.0) + step;
        if up > 0.0 {
            let start = center.y;
            if self.pass(world, center, Vec3::Y * up, 1, false, hits) {
                flags |= COLLIDED_ABOVE;
                // Only undo the part of the step that actually happened.
                step = step.min(center.y - start);
            }
        }
        if side.length_squared() > 0.0
            && self.pass(world, center, side, MAX_SIDE_ITERATIONS, true, hits)
        {
            flags |= COLLIDED_SIDES;
        }
        let down = motion.y.min(0.0) - step;
        let mut landed = None;
        if down < 0.0 && self.pass(world, center, Vec3::Y * down, 1, false, hits) {
            flags |= COLLIDED_BELOW;
            landed = hits.last().map(|h| h.point.y);
        }
        (flags, landed)
    }

    /// One collide-and-slide pass; true if anything was hit. `horizontal` (side pass) slides
    /// along the hit normal with its vertical part removed, so obstacles higher than the step
    /// offset act as walls instead of being climbed over the capsule's rounded bottom (inferred
    /// from PhysX's behaviour; needs measuring).
    #[allow(clippy::too_many_arguments)]
    fn pass(
        &self,
        world: &CollisionWorld,
        center: &mut Vec3,
        displacement: Vec3,
        max_iterations: usize,
        horizontal: bool,
        hits: &mut Vec<ControllerHit>,
    ) -> bool {
        let mut target = *center + displacement;
        let mut collided = false;
        for _ in 0..max_iterations {
            let remaining = target - *center;
            let length = remaining.length();
            if length <= 1e-6 {
                break;
            }
            let direction = remaining / length;
            let capsule = self.capsule(*center, self.skin_width);
            let Some(hit) = world.sweep_capsule(
                &capsule,
                direction,
                length,
                Filter::Solid,
                InitialOverlap::Block,
            ) else {
                *center = target;
                break;
            };
            collided = true;
            *center += direction * (hit.distance - CONTACT_EPSILON).max(0.0);
            hits.push(ControllerHit {
                point: hit.point,
                normal: hit.normal,
                move_direction: direction,
            });
            // PhysX collisionResponse with bump 0 and friction 1: keep the tangential part of
            // the direction, scaled by the distance still to go.
            let normal = if horizontal {
                let flat = Vec3::new(hit.normal.x, 0.0, hit.normal.z);
                if flat.length_squared() < 1e-6 {
                    break;
                }
                flat.normalize()
            } else {
                hit.normal
            };
            let amplitude = (target - *center).length();
            let tangent = direction - normal * direction.dot(normal);
            target = *center + tangent * amplitude;
        }
        collided
    }

    /// Pushes the capsule out of geometry it starts inside (Unity's overlap recovery).
    fn recover_from_overlap(&self, world: &CollisionWorld, center: &mut Vec3) {
        for _ in 0..4 {
            let capsule = self.capsule(*center, 0.0);
            let Some((depth, normal)) = world.deepest_penetration(&capsule) else {
                break;
            };
            *center += normal * (depth + CONTACT_EPSILON);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor_world() -> CollisionWorld {
        let mut world = CollisionWorld::new();
        // A 40x40 m floor at y = 0 (front face up) and a wall at z = 5 facing -Z.
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(40.0, 1.0, 40.0),
            false,
        );
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 2.0, 5.5),
            Vec3::new(40.0, 4.0, 1.0),
            false,
        );
        world
    }

    fn controller() -> CharacterController {
        CharacterController {
            height: 2.2,
            radius: 0.36,
            center: Vec3::new(0.0, 1.03, 0.06),
            skin_width: 0.08,
            step_offset: 0.3,
            slope_limit: 90.0,
            min_move_distance: 0.0,
        }
    }

    #[test]
    fn sphere_cast_hits_floor_face() {
        let world = floor_world();
        let hit = world
            .sphere_cast(Vec3::new(1.0, 3.0, 0.0), 0.5, -Vec3::Y, 10.0)
            .expect("hit");
        assert!((hit.distance - 2.5).abs() < 1e-4, "{hit:?}");
        assert!((hit.normal - Vec3::Y).length() < 1e-4);
    }

    #[test]
    fn controller_lands_on_floor_with_skin_gap() {
        let world = floor_world();
        let cc = controller();
        let mut position = Vec3::new(0.0, 1.0, 0.0);
        let mut hits = Vec::new();
        let flags = cc.move_by(
            &world,
            &mut position,
            Quat::IDENTITY,
            -Vec3::Y * 3.0,
            &mut hits,
        );
        assert_eq!(flags & COLLIDED_BELOW, COLLIDED_BELOW);
        // Capsule bottom = position.y + 1.03 - 1.1, resting skin width above the floor.
        let bottom = position.y + 1.03 - 1.1;
        assert!((bottom - 0.08).abs() < 1e-3, "bottom {bottom}");
        assert!(hits.iter().all(|h| h.normal.y > 0.99));
    }

    #[test]
    fn controller_slides_along_wall() {
        let world = floor_world();
        let cc = controller();
        let mut position = Vec3::new(0.0, 0.2, 3.0);
        let mut hits = Vec::new();
        let flags = cc.move_by(
            &world,
            &mut position,
            Quat::IDENTITY,
            Vec3::new(2.0, -0.3, 4.0),
            &mut hits,
        );
        assert_ne!(flags & COLLIDED_SIDES, 0);
        // Front of the capsule (z + 0.06 + radius + skin) stops at the wall (z = 5).
        let front = position.z + 0.06 + 0.36 + 0.08;
        assert!((front - 5.0).abs() < 1e-3, "front {front}");
        assert!((position.x - 2.0).abs() < 1e-3, "slid to x {}", position.x);
    }

    #[test]
    fn controller_steps_up_low_ledge_only() {
        let mut world = CollisionWorld::new();
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(40.0, 1.0, 40.0),
            false,
        );
        // A 0.2 m ledge from z = 1 to 9 and a 0.5 m one from z = -1 to -9.
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.1, 5.0),
            Vec3::new(4.0, 0.2, 8.0),
            false,
        );
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.25, -5.0),
            Vec3::new(4.0, 0.5, 8.0),
            false,
        );
        let cc = controller();
        let rest = 0.08 - (1.03 - 1.1);
        let mut hits = Vec::new();

        let mut position = Vec3::new(0.0, rest, 0.0);
        for _ in 0..40 {
            cc.move_by(
                &world,
                &mut position,
                Quat::IDENTITY,
                Vec3::new(0.0, -0.3, 0.1),
                &mut hits,
            );
        }
        assert!(
            (position.y - (rest + 0.2)).abs() < 1e-3,
            "on the 0.2 m ledge: {position}"
        );

        let mut position = Vec3::new(0.0, rest, 0.0);
        for _ in 0..40 {
            cc.move_by(
                &world,
                &mut position,
                Quat::IDENTITY,
                Vec3::new(0.0, -0.3, -0.1),
                &mut hits,
            );
        }
        assert!(
            (position.y - rest).abs() < 1e-3,
            "blocked by the 0.5 m ledge: {position}"
        );
        assert!(position.z > -1.0, "{position}");
    }
}
