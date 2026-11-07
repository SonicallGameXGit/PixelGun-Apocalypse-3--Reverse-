//! Static collision world and a Unity `CharacterController` equivalent.
//!
//! The original runs PhysX 3.4 through Unity 2018.3. The map geometry is non-convex MeshColliders
//! (converted Source BSP) plus a few BoxColliders, all static, so the world is a triangle soup in a
//! uniform grid. Sweeps are exact capsule-vs-triangle time-of-impact tests (no substepping).
//!
//! The controller is a port of the PhysX 3.4 character controller (BSD-3,
//! `PhysXCharacterKinematic/src/CctCharacterController.cpp`: `SweepTest::moveCharacter`,
//! `doSweepTest`, `collisionResponse`, `computeMTD`) for the settings this game uses. See
//! `docs/gameplay/movement.md`.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use glam::{Mat4, Quat, Vec3};

use crate::obj::parse_obj;
use crate::scene::{ColliderShape, Scene};

/// Grid cell size in metres.
const CELL: f32 = 4.0;
/// PhysX `GU_EPSILON_SAME_DISTANCE`: hits this close count as the same distance.
const EPSILON_SAME_DISTANCE: f32 = 1e-3;

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
pub struct QueryFilter {
    /// Bit `n` set = colliders on layer `n` are seen.
    pub layer_mask: u32,
    pub triggers: bool,
}

/// Unity's "Ignore Raycast" layer.
pub const IGNORE_RAYCAST_LAYER: u32 = 2;

impl QueryFilter {
    /// The CharacterController: every layer (this build's layer collision matrix is all ones),
    /// never triggers (PhysX's controller pre-filter drops trigger shapes).
    pub const CONTROLLER: QueryFilter = QueryFilter {
        layer_mask: u32::MAX,
        triggers: false,
    };
    /// `Physics.SphereCast` / `CapsuleCast` without a layer mask: `DefaultRaycastLayers` (all but
    /// Ignore Raycast), triggers included (`queriesHitTriggers` is on in PhysicsManager).
    pub const DEFAULT_RAYCAST: QueryFilter = QueryFilter {
        layer_mask: !(1 << IGNORE_RAYCAST_LAYER),
        triggers: true,
    };
}

/// What a sweep does with triangles the shape already overlaps at its start position.
#[derive(Clone, Copy, PartialEq)]
pub enum InitialOverlap {
    /// PhysX: report a hit at distance 0 (the controller then runs overlap recovery).
    Block,
    /// Skip them, as Unity's `Physics.SphereCast` / `CapsuleCast` do.
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

#[derive(Clone, Copy)]
struct TriangleInfo {
    trigger: bool,
    layer: u32,
    /// GameObject tag of the collider (`assets::tag_name` numbering).
    tag: u32,
    /// Lowest grid cell the triangle is listed in.
    min_cell: [i32; 3],
}

impl TriangleInfo {
    fn passes(self, filter: QueryFilter) -> bool {
        (filter.triggers || !self.trigger) && filter.layer_mask & (1 << self.layer) != 0
    }
}

pub struct CollisionWorld {
    triangles: Vec<Triangle>,
    info: Vec<TriangleInfo>,
    cells: HashMap<[i32; 3], Vec<u32>>,
    /// Inclusive range of grid cells holding triangles.
    cell_bounds: Option<([i32; 3], [i32; 3])>,
    /// Tag given to the triangles of the colliders added next (`add_mesh` / `add_box`).
    pub collider_tag: u32,
}

/// `RaycastHit` of `CollisionWorld::raycast`.
#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub distance: f32,
    pub point: Vec3,
    /// Face normal of the hit triangle (PhysX reports the face normal for mesh hits).
    pub normal: Vec3,
    /// Tag of the hit collider's GameObject.
    pub tag: u32,
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
            info: Vec::new(),
            cells: HashMap::new(),
            cell_bounds: None,
            collider_tag: 0,
        }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    fn add_triangle(&mut self, triangle: Triangle, trigger: bool, layer: u32) {
        let index = self.triangles.len() as u32;
        let min = cell_of(triangle.a.min(triangle.b).min(triangle.c));
        let info = TriangleInfo {
            trigger,
            layer,
            tag: self.collider_tag,
            min_cell: min,
        };
        let max = cell_of(triangle.a.max(triangle.b).max(triangle.c));
        self.cell_bounds = Some(match self.cell_bounds {
            None => (min, max),
            Some((lo, hi)) => (
                [lo[0].min(min[0]), lo[1].min(min[1]), lo[2].min(min[2])],
                [hi[0].max(max[0]), hi[1].max(max[1]), hi[2].max(max[2])],
            ),
        });
        for x in min[0]..=max[0] {
            for y in min[1]..=max[1] {
                for z in min[2]..=max[2] {
                    self.cells.entry([x, y, z]).or_default().push(index);
                }
            }
        }
        self.triangles.push(triangle);
        self.info.push(info);
    }

    /// Mesh triangles transformed to world space. A mirroring transform reverses the winding, so
    /// the corners are swapped back to keep the front face outside (as PhysX does).
    pub fn add_mesh(
        &mut self,
        world: Mat4,
        positions: &[Vec3],
        indices: &[u32],
        trigger: bool,
        layer: u32,
    ) {
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
                self.add_triangle(triangle, trigger, layer);
            }
        }
    }

    pub fn add_box(&mut self, world: Mat4, center: Vec3, size: Vec3, trigger: bool, layer: u32) {
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
                self.add_triangle(triangle, trigger, layer);
            }
        }
    }

    /// Calls `visit` for every triangle (once) whose grid cells overlap the box and that passes
    /// the filter.
    fn for_each_candidate(
        &self,
        min: Vec3,
        max: Vec3,
        filter: QueryFilter,
        mut visit: impl FnMut(&Triangle),
    ) {
        let (lo, hi) = (cell_of(min), cell_of(max));
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    let Some(list) = self.cells.get(&[x, y, z]) else {
                        continue;
                    };
                    for &index in list {
                        let info = self.info[index as usize];
                        // A triangle spanning several cells is visited only in the first cell
                        // of the query range that lists it.
                        let m = info.min_cell;
                        if [m[0].max(lo[0]), m[1].max(lo[1]), m[2].max(lo[2])] != [x, y, z]
                            || !info.passes(filter)
                        {
                            continue;
                        }
                        visit(&self.triangles[index as usize]);
                    }
                }
            }
        }
    }

    /// First hit of `capsule` moved along unit `direction` up to `max_distance`
    /// (`PxMeshQuery::sweep` as the controller and Unity's casts call it):
    /// - back faces (`normal · direction > 0`) are culled; `queriesHitBackfaces` is off;
    /// - among hits within `EPSILON_SAME_DISTANCE` the most opposing triangle wins
    ///   (PhysX `keepTriangle`);
    /// - a triangle the capsule already overlaps is a hit at distance 0 (`Block`) or skipped.
    pub fn sweep_capsule(
        &self,
        capsule: &Capsule,
        direction: Vec3,
        max_distance: f32,
        filter: QueryFilter,
        overlap: InitialOverlap,
    ) -> Option<SweepHit> {
        let end_p0 = capsule.p0 + direction * max_distance;
        let end_p1 = capsule.p1 + direction * max_distance;
        let pad = Vec3::splat(capsule.radius + EPSILON_SAME_DISTANCE);
        let min = capsule.p0.min(capsule.p1).min(end_p0).min(end_p1) - pad;
        let max = capsule.p0.max(capsule.p1).max(end_p0).max(end_p1) + pad;
        let mut best: Option<SweepHit> = None;
        let mut best_distance = max_distance;
        let mut best_alignment = 2.0;
        let mut overlapped: Option<SweepHit> = None;
        self.for_each_candidate(min, max, filter, |triangle| {
            if overlapped.is_some() || triangle.normal.dot(direction) > 0.0 {
                return;
            }
            let Some(hit) =
                sweep_capsule_triangle(capsule, direction, max_distance, triangle, overlap)
            else {
                return;
            };
            if hit.distance == 0.0 && overlap == InitialOverlap::Block {
                overlapped = Some(hit);
                return;
            }
            let alignment = -triangle.normal.dot(direction).abs();
            if keep_triangle(
                hit.distance,
                alignment,
                best_distance,
                best_alignment,
                max_distance,
            ) {
                best_distance = hit.distance;
                best_alignment = alignment;
                best = Some(hit);
            }
        });
        overlapped.or(best)
    }

    /// `Physics.SphereCast` with the default layer mask: colliders the sphere starts inside are
    /// not reported.
    pub fn sphere_cast(
        &self,
        origin: Vec3,
        radius: f32,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<SweepHit> {
        self.capsule_cast(origin, origin, radius, direction, max_distance)
    }

    /// `Physics.CapsuleCast(point1, point2, radius, direction, out hit, maxDistance)` with the
    /// default layer mask; `point1`/`point2` are the centres of the end spheres.
    pub fn capsule_cast(
        &self,
        point1: Vec3,
        point2: Vec3,
        radius: f32,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<SweepHit> {
        let capsule = Capsule {
            p0: point1,
            p1: point2,
            radius,
        };
        self.sweep_capsule(
            &capsule,
            direction,
            max_distance,
            QueryFilter::DEFAULT_RAYCAST,
            InitialOverlap::Ignore,
        )
    }

    /// `Physics.Raycast(origin, direction, out hit, maxDistance)` with the default layer mask
    /// (`QueryFilter::DEFAULT_RAYCAST`): the closest front face along the ray. `direction` need not
    /// be normalised (Unity normalises it); `max_distance` may be infinite. Back faces are culled
    /// (`queriesHitBackfaces` is off), so a ray starting inside a closed collider does not hit it.
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RayHit> {
        let direction = direction.normalize_or_zero();
        let (lo, hi) = self.cell_bounds?;
        if direction == Vec3::ZERO || max_distance.is_nan() || max_distance < 0.0 {
            return None;
        }
        // Clip the ray to the grid's bounds (slab test), then walk its cells in order
        // (Amanatides & Woo).
        let bounds_min = Vec3::new(lo[0] as f32, lo[1] as f32, lo[2] as f32) * CELL;
        let bounds_max =
            Vec3::new((hi[0] + 1) as f32, (hi[1] + 1) as f32, (hi[2] + 1) as f32) * CELL;
        let mut enter = 0.0f32;
        let mut exit = max_distance;
        for axis in 0..3 {
            if direction[axis] == 0.0 {
                if origin[axis] < bounds_min[axis] || origin[axis] > bounds_max[axis] {
                    return None;
                }
                continue;
            }
            let t0 = (bounds_min[axis] - origin[axis]) / direction[axis];
            let t1 = (bounds_max[axis] - origin[axis]) / direction[axis];
            enter = enter.max(t0.min(t1));
            exit = exit.min(t0.max(t1));
        }
        if enter > exit {
            return None;
        }
        let start = origin + direction * enter;
        let mut cell = cell_of(start);
        let mut step = [0i32; 3];
        let mut next = Vec3::splat(f32::INFINITY);
        let mut delta = Vec3::splat(f32::INFINITY);
        for axis in 0..3 {
            cell[axis] = cell[axis].clamp(lo[axis], hi[axis]);
            if direction[axis] == 0.0 {
                continue;
            }
            step[axis] = if direction[axis] > 0.0 { 1 } else { -1 };
            let wall = cell_of_axis(cell[axis], step[axis]) * CELL;
            next[axis] = (wall - origin[axis]) / direction[axis];
            delta[axis] = CELL / direction[axis].abs();
        }
        let mut best: Option<RayHit> = None;
        loop {
            if let Some(list) = self.cells.get(&cell) {
                for &index in list {
                    let info = self.info[index as usize];
                    if !info.passes(QueryFilter::DEFAULT_RAYCAST) {
                        continue;
                    }
                    let triangle = &self.triangles[index as usize];
                    let Some(t) = ray_triangle(origin, direction, triangle) else {
                        continue;
                    };
                    if t <= max_distance && best.is_none_or(|b| t < b.distance) {
                        best = Some(RayHit {
                            distance: t,
                            point: origin + direction * t,
                            normal: triangle.normal,
                            tag: info.tag,
                        });
                    }
                }
            }
            // Leave the cell through its nearest boundary.
            let axis = if next.x < next.y {
                if next.x < next.z { 0 } else { 2 }
            } else if next.y < next.z {
                1
            } else {
                2
            };
            let leave = next[axis];
            if best.is_some_and(|b| b.distance <= leave) || leave > exit {
                return best;
            }
            cell[axis] += step[axis];
            if cell[axis] < lo[axis] || cell[axis] > hi[axis] {
                return best;
            }
            next[axis] += delta[axis];
        }
    }

    /// PhysX `computePenetration` of a capsule against the mesh triangles, as `computeMTD`
    /// uses it: contacts of front-facing triangles inside the capsule, combined per axis as in
    /// `GuMTD.cpp processContacts`. Returns (push-out direction, depth).
    fn penetration(&self, capsule: &Capsule, filter: QueryFilter) -> Option<(Vec3, f32)> {
        let pad = Vec3::splat(capsule.radius);
        let min = capsule.p0.min(capsule.p1) - pad;
        let max = capsule.p0.max(capsule.p1) + pad;
        let (mut low, mut high) = (Vec3::ZERO, Vec3::ZERO);
        let mut count = 0;
        self.for_each_candidate(min, max, filter, |triangle| {
            let (on_segment, on_triangle) =
                closest_segment_triangle(capsule.p0, capsule.p1, triangle);
            if triangle.normal.dot(on_segment - triangle.a) <= 0.0 {
                return;
            }
            let offset = on_segment - on_triangle;
            let distance = offset.length();
            let separation = distance - capsule.radius;
            if separation >= 0.0 {
                return;
            }
            let normal = if distance > 1e-6 {
                offset / distance
            } else {
                triangle.normal
            };
            let depenetration = normal * separation;
            low = low.min(depenetration);
            high = high.max(depenetration);
            count += 1;
        });
        if count == 0 {
            return None;
        }
        let pick = |a: f32, b: f32| if a == 0.0 { b } else { a };
        let low1 = Vec3::new(
            pick(low.x, high.x),
            pick(low.y, high.y),
            pick(low.z, high.z),
        );
        let high1 = Vec3::new(
            pick(high.x, low.x),
            pick(high.y, low.y),
            pick(high.z, low.z),
        );
        let separation = (low1 + high1) * 0.5;
        if separation.length_squared() < 1e-10 {
            return None;
        }
        Some((-separation.normalize(), separation.length()))
    }
}

/// PhysX `keepTriangle` (GuInternal.h).
fn keep_triangle(
    distance: f32,
    alignment: f32,
    best_distance: f32,
    best_alignment: f32,
    max_distance: f32,
) -> bool {
    if distance > max_distance {
        return false;
    }
    let epsilon = EPSILON_SAME_DISTANCE * distance.max(best_distance).max(1.0);
    if distance < best_distance - epsilon {
        return true;
    }
    if distance < best_distance + epsilon && alignment < best_alignment {
        return true;
    }
    alignment == best_alignment && distance < best_distance
}

/// World coordinate (in cells) of the boundary a ray stepping `step` leaves `cell` through.
fn cell_of_axis(cell: i32, step: i32) -> f32 {
    if step > 0 {
        (cell + 1) as f32
    } else {
        cell as f32
    }
}

/// Ray / front face of a triangle (Möller–Trumbore): the distance along unit `direction`.
fn ray_triangle(origin: Vec3, direction: Vec3, triangle: &Triangle) -> Option<f32> {
    if triangle.normal.dot(direction) >= 0.0 {
        return None;
    }
    let e1 = triangle.b - triangle.a;
    let e2 = triangle.c - triangle.a;
    let p = direction.cross(e2);
    let determinant = e1.dot(p);
    if determinant.abs() < 1e-12 {
        return None;
    }
    let inverse = 1.0 / determinant;
    let s = origin - triangle.a;
    let u = s.dot(p) * inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = direction.dot(q) * inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inverse;
    (t >= 0.0).then_some(t)
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
                world.collider_tag = collider.tag;
                world.add_mesh(
                    collider.world,
                    positions,
                    indices,
                    collider.is_trigger,
                    collider.layer,
                );
            }
            ColliderShape::Box { center, size } => {
                world.collider_tag = collider.tag;
                world.add_box(
                    collider.world,
                    *center,
                    *size,
                    collider.is_trigger,
                    collider.layer,
                );
            }
        }
    }
    world.collider_tag = 0;
    Ok(world)
}

/// Whether a capsule overlaps a BoxCollider (`center`/`size` in the space of `world`). The box
/// is the oriented box PhysX builds from the collider: axes = the matrix columns, extents scaled
/// by their lengths.
pub fn capsule_overlaps_box(capsule: &Capsule, world: Mat4, center: Vec3, size: Vec3) -> bool {
    let origin = world.transform_point3(center);
    let columns = [
        world.x_axis.truncate(),
        world.y_axis.truncate(),
        world.z_axis.truncate(),
    ];
    let axes = columns.map(|c| c.normalize_or_zero());
    let half = Vec3::new(
        columns[0].length() * size.x.abs() * 0.5,
        columns[1].length() * size.y.abs() * 0.5,
        columns[2].length() * size.z.abs() * 0.5,
    );
    let local = |p: Vec3| {
        let d = p - origin;
        Vec3::new(axes[0].dot(d), axes[1].dot(d), axes[2].dot(d))
    };
    let (a, b) = (local(capsule.p0), local(capsule.p1));
    let distance_squared = |t: f32| {
        let p = a + (b - a) * t;
        let outside = (p.abs() - half).max(Vec3::ZERO);
        outside.length_squared()
    };
    // The distance from a point moving along a segment to a convex box is convex in t.
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..40 {
        let m1 = lo + (hi - lo) / 3.0;
        let m2 = hi - (hi - lo) / 3.0;
        if distance_squared(m1) <= distance_squared(m2) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    let best = distance_squared(0.0)
        .min(distance_squared(1.0))
        .min(distance_squared((lo + hi) * 0.5));
    best <= capsule.radius * capsule.radius
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
            // PhysX `setInitialOverlapResults`: distance 0, normal against the motion.
            InitialOverlap::Block => Some(SweepHit {
                distance: 0.0,
                normal: -direction,
                point: on_triangle,
            }),
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
// CharacterController (PhysX 3.4 capsule controller)

pub const COLLIDED_SIDES: u8 = 1;
pub const COLLIDED_ABOVE: u8 = 2;
pub const COLLIDED_BELOW: u8 = 4;

/// PhysX `MAX_ITER`: collide-and-slide iterations per pass.
const MAX_ITER: usize = 10;
/// PhysX `computeMTD` iteration limit.
const MTD_ITERATIONS: usize = 4;
/// PhysX `PxControllerDesc::scaleCoeff` default: the kinematic proxy shape (what triggers see)
/// is the capsule scaled by this. Inferred: Unity does not expose it.
pub const PROXY_SCALE: f32 = 0.8;

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
    /// PhysX contact offset.
    pub skin_width: f32,
    pub step_offset: f32,
    /// Degrees. 90 in this game: PhysX gets `cos 90° ≈ -4.4e-8`, so `testSlope` never reports a
    /// non-walkable triangle and the slope/step validation of `moveCharacter` never runs.
    pub slope_limit: f32,
    /// PhysX `minDist`.
    pub min_move_distance: f32,
}

/// PhysX `Ps::isAlmostZero`.
fn is_almost_zero(v: Vec3) -> bool {
    v.x.abs() <= 1e-6 && v.y.abs() <= 1e-6 && v.z.abs() <= 1e-6
}

impl CharacterController {
    /// The PhysX capsule: Unity's height includes the end caps; it is clamped to a sphere.
    pub fn capsule(&self, center: Vec3) -> Capsule {
        let half = (self.height * 0.5 - self.radius).max(0.0);
        Capsule {
            p0: center - Vec3::Y * half,
            p1: center + Vec3::Y * half,
            radius: self.radius,
        }
    }

    /// The kinematic proxy shape the trigger volumes are tested against.
    pub fn proxy_capsule(&self, position: Vec3, rotation: Quat) -> Capsule {
        let center = self.world_center(position, rotation);
        let half = (self.height * 0.5 - self.radius).max(0.0) * PROXY_SCALE;
        Capsule {
            p0: center - Vec3::Y * half,
            p1: center + Vec3::Y * half,
            radius: self.radius * PROXY_SCALE,
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
        let mut center = self.world_center(*position, rotation);
        let flags = self.move_character(world, &mut center, motion, hits);
        *position = center - rotation * self.center;
        flags
    }

    /// `SweepTest::moveCharacter` for this game's settings (static geometry only, no other
    /// controllers, `slopeLimit` 90 so no walk experiment, `eEASY` climbing).
    fn move_character(
        &self,
        world: &CollisionWorld,
        center: &mut Vec3,
        direction: Vec3,
        hits: &mut Vec<ControllerHit>,
    ) -> u8 {
        let original_height = center.y;
        // Auto-step is cancelled while moving up.
        let mut step_offset = if direction.y > 0.0 {
            0.0
        } else {
            self.step_offset
        };
        let mut up = Vec3::ZERO;
        let mut down = Vec3::ZERO;
        if direction.y <= 0.0 {
            down.y = direction.y;
        } else {
            up.y = direction.y;
        }
        let side = Vec3::new(direction.x, 0.0, direction.z);
        let side_is_zero = is_almost_zero(side);
        if !side_is_zero {
            up.y += step_offset;
        }

        let mut flags = 0;
        let max_iter_up = if side_is_zero { MAX_ITER } else { 1 };
        let (moved, collisions) = self.do_sweep_test(world, center, up, max_iter_up, hits);
        if moved && collisions > 0 {
            flags |= COLLIDED_ABOVE;
            // Don't undo more than the up pass did.
            let delta = center.y - original_height;
            if delta < step_offset {
                step_offset = delta;
            }
        }

        let (moved, collisions) = self.do_sweep_test(world, center, side, MAX_ITER, hits);
        if moved && collisions > 0 {
            flags |= COLLIDED_SIDES;
        }

        if !side_is_zero {
            down.y -= step_offset;
        }
        let (moved, collisions) = self.do_sweep_test(world, center, down, 1, hits);
        if moved && collisions > 0 && direction.y <= 0.0 {
            flags |= COLLIDED_BELOW;
        }
        flags
    }

    /// `SweepTest::doSweepTest`: collide and slide along `direction`. Returns (moved at all,
    /// number of collisions).
    fn do_sweep_test(
        &self,
        world: &CollisionWorld,
        center: &mut Vec3,
        direction: Vec3,
        max_iterations: usize,
        hits: &mut Vec<ControllerHit>,
    ) -> (bool, usize) {
        if direction == Vec3::ZERO {
            return (false, 0);
        }
        let mut has_moved = false;
        let mut collisions = 0;
        let mut current = *center;
        let mut target = current + direction;
        for _ in 0..max_iterations {
            let offset = target - current;
            let length = offset.length();
            if length <= self.min_move_distance {
                break;
            }
            let current_direction = offset / length;
            // Quake 2: stop dead if the slide turned against the original motion.
            if current_direction.dot(direction) <= 0.0 {
                break;
            }
            has_moved = true;

            let capsule = self.capsule(current);
            let Some(hit) = world.sweep_capsule(
                &capsule,
                current_direction,
                length + self.skin_width,
                QueryFilter::CONTROLLER,
                InitialOverlap::Block,
            ) else {
                current = target;
                break;
            };

            if hit.distance == 0.0 {
                // Overlap recovery (on by default in the PhysX controller manager).
                *center = self.compute_mtd(world, current);
                return (has_moved, collisions + 1);
            }

            collisions += 1;
            hits.push(ControllerHit {
                point: hit.point,
                normal: hit.normal,
                move_direction: current_direction,
            });

            if hit.distance > self.skin_width {
                current += current_direction * (hit.distance - self.skin_width);
            }

            // collisionResponse with bump 0 and friction 1: keep the tangential part of the
            // reflected direction, scaled by the distance still to go.
            let normal = hit.normal;
            let reflected =
                (current_direction - normal * 2.0 * current_direction.dot(normal)).normalize();
            let tangent = reflected - normal * reflected.dot(normal);
            let amplitude = (target - current).length();
            target = current + tangent * amplitude;
        }
        *center = current;
        (has_moved, collisions)
    }

    /// `computeMTD`: pushes the capsule inflated by the contact offset out of the geometry.
    fn compute_mtd(&self, world: &CollisionWorld, center: Vec3) -> Vec3 {
        let mut p = center;
        for _ in 0..MTD_ITERATIONS {
            let mut capsule = self.capsule(p);
            capsule.radius += self.skin_width;
            let Some((direction, depth)) = world.penetration(&capsule, QueryFilter::CONTROLLER)
            else {
                break;
            };
            p += direction * depth;
        }
        p
    }
}

/// Shape of a primitive collider in its GameObject's local space (`BoxCollider` /
/// `SphereCollider`), for the colliders that move with an animated body (players' hit boxes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitShape {
    Box { center: Vec3, size: Vec3 },
    Sphere { center: Vec3, radius: f32 },
}

/// A primitive collider at its current world transform.
#[derive(Clone, Copy, Debug)]
pub struct DynamicCollider {
    pub world: Mat4,
    pub shape: HitShape,
    /// Tag of the collider's GameObject.
    pub tag: u32,
}

impl DynamicCollider {
    /// World centre and radius of a sphere (PhysX scales the radius by the largest axis scale).
    fn sphere(&self, center: Vec3, radius: f32) -> (Vec3, f32) {
        let scale = Vec3::new(
            self.world.x_axis.truncate().length(),
            self.world.y_axis.truncate().length(),
            self.world.z_axis.truncate().length(),
        );
        (
            self.world.transform_point3(center),
            radius * scale.max_element(),
        )
    }

    /// `Physics.Raycast` against this collider: the first entry point within `max_distance`.
    /// A ray starting inside the shape does not hit it (PhysX primitives).
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RayHit> {
        let direction = direction.normalize_or_zero();
        if direction == Vec3::ZERO || max_distance <= 0.0 {
            return None;
        }
        match self.shape {
            HitShape::Box { center, size } => {
                let inverse = self.world.inverse();
                let o = inverse.transform_point3(origin) - center;
                let d = inverse.transform_vector3(direction * max_distance);
                let half = size * 0.5;
                if o.abs().cmple(half).all() {
                    return None;
                }
                // Slab test in local space; t runs over [0, 1] along the segment.
                let mut t_enter = 0.0f32;
                let mut t_exit = 1.0f32;
                let mut axis = 0;
                for i in 0..3 {
                    if d[i].abs() < 1e-12 {
                        if o[i].abs() > half[i] {
                            return None;
                        }
                        continue;
                    }
                    let a = (-half[i] - o[i]) / d[i];
                    let b = (half[i] - o[i]) / d[i];
                    let (near, far) = if a < b { (a, b) } else { (b, a) };
                    if near > t_enter {
                        t_enter = near;
                        axis = i;
                    }
                    t_exit = t_exit.min(far);
                    if t_enter > t_exit {
                        return None;
                    }
                }
                let mut local_normal = Vec3::ZERO;
                local_normal[axis] = -d[axis].signum();
                // Normals transform with the inverse transpose.
                let normal = inverse
                    .transpose()
                    .transform_vector3(local_normal)
                    .normalize_or_zero();
                let distance = t_enter * max_distance;
                Some(RayHit {
                    distance,
                    point: origin + direction * distance,
                    normal,
                    tag: self.tag,
                })
            }
            HitShape::Sphere { center, radius } => {
                let (c, r) = self.sphere(center, radius);
                let m = origin - c;
                let c_term = m.length_squared() - r * r;
                if c_term <= 0.0 {
                    return None;
                }
                let b = m.dot(direction);
                let discriminant = b * b - c_term;
                if b > 0.0 || discriminant < 0.0 {
                    return None;
                }
                let distance = -b - discriminant.sqrt();
                if distance > max_distance {
                    return None;
                }
                let point = origin + direction * distance;
                Some(RayHit {
                    distance,
                    point,
                    normal: (point - c).normalize_or_zero(),
                    tag: self.tag,
                })
            }
        }
    }

    /// `Collider.bounds` (world axis-aligned box): (min, max).
    pub fn bounds(&self) -> (Vec3, Vec3) {
        match self.shape {
            HitShape::Box { center, size } => {
                let half = size * 0.5;
                let mut min = Vec3::splat(f32::INFINITY);
                let mut max = Vec3::splat(f32::NEG_INFINITY);
                for corner in 0..8 {
                    let sign = Vec3::new(
                        if corner & 1 == 0 { -1.0 } else { 1.0 },
                        if corner & 2 == 0 { -1.0 } else { 1.0 },
                        if corner & 4 == 0 { -1.0 } else { 1.0 },
                    );
                    let p = self.world.transform_point3(center + half * sign);
                    min = min.min(p);
                    max = max.max(p);
                }
                (min, max)
            }
            HitShape::Sphere { center, radius } => {
                let (c, r) = self.sphere(center, radius);
                (c - Vec3::splat(r), c + Vec3::splat(r))
            }
        }
    }

    /// `Collider.ClosestPointOnBounds(point)`.
    pub fn closest_point_on_bounds(&self, point: Vec3) -> Vec3 {
        let (min, max) = self.bounds();
        point.clamp(min, max)
    }

    /// Whether `Physics.OverlapSphere(center, radius)` reports this collider.
    pub fn overlaps_sphere(&self, center: Vec3, radius: f32) -> bool {
        match self.shape {
            HitShape::Box { center: c, size } => {
                // Closest point of the oriented box, along its (scaled) world axes.
                let world_center = self.world.transform_point3(c);
                let relative = center - world_center;
                let mut closest = world_center;
                for (i, axis) in [self.world.x_axis, self.world.y_axis, self.world.z_axis]
                    .iter()
                    .enumerate()
                {
                    let axis = axis.truncate();
                    let length = axis.length();
                    if length <= 0.0 {
                        continue;
                    }
                    let unit = axis / length;
                    let extent = size[i] * 0.5 * length;
                    closest += unit * relative.dot(unit).clamp(-extent, extent);
                }
                closest.distance_squared(center) <= radius * radius
            }
            HitShape::Sphere {
                center: c,
                radius: r,
            } => {
                let (c, r) = self.sphere(c, r);
                c.distance(center) <= r + radius
            }
        }
    }
}

/// A sphere moving from `center` along the unit `direction` for `distance` against a capsule:
/// the travelled distance and the contact normal (pointing from the capsule). Starting inside
/// reports a hit at 0.
pub fn sweep_sphere_capsule(
    center: Vec3,
    radius: f32,
    direction: Vec3,
    distance: f32,
    capsule: &Capsule,
) -> Option<SweepHit> {
    let r = radius + capsule.radius;
    let closest_on_axis = |p: Vec3| {
        let axis = capsule.p1 - capsule.p0;
        let length_squared = axis.length_squared();
        let t = if length_squared > 0.0 {
            ((p - capsule.p0).dot(axis) / length_squared).clamp(0.0, 1.0)
        } else {
            0.0
        };
        capsule.p0 + axis * t
    };
    let start_axis = closest_on_axis(center);
    if center.distance_squared(start_axis) <= r * r {
        return Some(SweepHit {
            distance: 0.0,
            normal: (center - start_axis).normalize_or(-direction),
            point: center,
        });
    }
    // Ray against the infinite cylinder, then the two end spheres; keep the nearest.
    let mut best: Option<f32> = None;
    let mut consider = |t: f32| {
        if (0.0..=distance).contains(&t) && best.is_none_or(|b| t < b) {
            best = Some(t);
        }
    };
    let axis = capsule.p1 - capsule.p0;
    let length = axis.length();
    if length > 0.0 {
        let a_dir = axis / length;
        let m = center - capsule.p0;
        let d_perp = direction - a_dir * direction.dot(a_dir);
        let m_perp = m - a_dir * m.dot(a_dir);
        let a = d_perp.length_squared();
        let b = m_perp.dot(d_perp);
        let c = m_perp.length_squared() - r * r;
        let discriminant = b * b - a * c;
        if a > 1e-12 && discriminant >= 0.0 {
            let t = (-b - discriminant.sqrt()) / a;
            let along = (m + direction * t).dot(a_dir);
            if (0.0..=length).contains(&along) {
                consider(t);
            }
        }
    }
    for end in [capsule.p0, capsule.p1] {
        let m = center - end;
        let b = m.dot(direction);
        let c = m.length_squared() - r * r;
        let discriminant = b * b - c;
        if discriminant >= 0.0 {
            consider(-b - discriminant.sqrt());
        }
    }
    let t = best?;
    let at = center + direction * t;
    let normal = (at - closest_on_axis(at)).normalize_or(-direction);
    Some(SweepHit {
        distance: t,
        normal,
        point: at - normal * radius,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_box_and_sphere_raycasts() {
        let unit_box = DynamicCollider {
            world: Mat4::from_rotation_translation(
                Quat::from_rotation_y(0.3),
                Vec3::new(0.0, 0.0, 5.0),
            ),
            shape: HitShape::Box {
                center: Vec3::ZERO,
                size: Vec3::ONE,
            },
            tag: 7,
        };
        let hit = unit_box.raycast(Vec3::ZERO, Vec3::Z, 10.0).unwrap();
        // The rotated box's front face at x = 0 lies 0.5 / cos(0.3) in front of its centre.
        assert!(
            (hit.distance - (5.0 - 0.5 / 0.3f32.cos())).abs() < 1e-4,
            "{hit:?}"
        );
        assert!(hit.normal.z < -0.9 && hit.tag == 7);
        assert!(
            unit_box.raycast(Vec3::ZERO, Vec3::Z, 4.0).is_none(),
            "too short"
        );
        assert!(
            unit_box
                .raycast(Vec3::new(0.0, 0.0, 5.0), Vec3::Z, 10.0)
                .is_none(),
            "starting inside"
        );
        let scaled_sphere = DynamicCollider {
            world: Mat4::from_scale_rotation_translation(
                Vec3::new(1.0, 2.0, 1.0),
                Quat::IDENTITY,
                Vec3::new(0.0, 0.0, 5.0),
            ),
            shape: HitShape::Sphere {
                center: Vec3::ZERO,
                radius: 0.5,
            },
            tag: 0,
        };
        let hit = scaled_sphere.raycast(Vec3::ZERO, Vec3::Z, 10.0).unwrap();
        assert!((hit.distance - 4.0).abs() < 1e-5, "radius scaled by 2");
        assert_eq!(
            scaled_sphere.closest_point_on_bounds(Vec3::ZERO),
            Vec3::new(0.0, 0.0, 4.0)
        );
        assert!(unit_box.overlaps_sphere(Vec3::new(0.0, 0.0, 3.9), 0.6));
        assert!(!unit_box.overlaps_sphere(Vec3::new(0.0, 0.0, 3.0), 0.6));
    }

    #[test]
    fn sphere_sweeps_against_capsule() {
        let capsule = Capsule {
            p0: Vec3::new(0.0, 0.5, 5.0),
            p1: Vec3::new(0.0, 1.5, 5.0),
            radius: 0.5,
        };
        let hit =
            sweep_sphere_capsule(Vec3::new(0.0, 1.0, 0.0), 0.25, Vec3::Z, 10.0, &capsule).unwrap();
        assert!((hit.distance - 4.25).abs() < 1e-5, "{hit:?}");
        assert!(hit.normal.z < -0.99);
        // Over the top: hits the upper end sphere.
        let hit =
            sweep_sphere_capsule(Vec3::new(0.0, 2.2, 0.0), 0.25, Vec3::Z, 10.0, &capsule).unwrap();
        let expected = 5.0 - (0.75f32 * 0.75 - 0.7 * 0.7).sqrt();
        assert!((hit.distance - expected).abs() < 1e-4, "{hit:?}");
        assert!(
            sweep_sphere_capsule(Vec3::new(2.0, 1.0, 0.0), 0.25, Vec3::Z, 10.0, &capsule).is_none()
        );
    }

    const FLOOR_LAYER: u32 = 0;

    fn floor_world() -> CollisionWorld {
        let mut world = CollisionWorld::new();
        // A 40x40 m floor at y = 0 (front face up) and a wall at z = 5 facing -Z.
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(40.0, 1.0, 40.0),
            false,
            FLOOR_LAYER,
        );
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 2.0, 5.5),
            Vec3::new(40.0, 4.0, 1.0),
            false,
            FLOOR_LAYER,
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

    /// Root height of a controller resting on a floor at y = 0.
    const REST: f32 = 0.08 - (1.03 - 1.1);

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
    fn sphere_cast_skips_ignore_raycast_triggers() {
        let mut world = floor_world();
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(1.0, 1.5, 0.0),
            Vec3::new(1.0, 0.1, 1.0),
            true,
            IGNORE_RAYCAST_LAYER,
        );
        let hit = world
            .sphere_cast(Vec3::new(1.0, 3.0, 0.0), 0.2, -Vec3::Y, 10.0)
            .expect("hit");
        assert!((hit.distance - 2.8).abs() < 1e-4, "{hit:?}");
    }

    #[test]
    fn raycast_hits_closest_front_face_with_its_tag() {
        let mut world = floor_world();
        world.collider_tag = 20000;
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 1.0, 2.5),
            Vec3::new(1.0, 1.0, 1.0),
            false,
            FLOOR_LAYER,
        );
        // Along +Z the tagged box (front face z = 2) comes before the wall (z = 5).
        let hit = world
            .raycast(Vec3::new(0.0, 1.0, -3.0), Vec3::Z * 7.0, f32::INFINITY)
            .expect("hit");
        assert!((hit.distance - 5.0).abs() < 1e-4, "{hit:?}");
        assert!((hit.point - Vec3::new(0.0, 1.0, 2.0)).length() < 1e-4);
        assert!((hit.normal + Vec3::Z).length() < 1e-4);
        assert_eq!(hit.tag, 20000);
        // Beside it, the untagged wall; short of it, nothing.
        let hit = world
            .raycast(Vec3::new(3.0, 1.0, -3.0), Vec3::Z, 100.0)
            .expect("hit");
        assert!((hit.distance - 8.0).abs() < 1e-4 && hit.tag == 0, "{hit:?}");
        assert!(
            world
                .raycast(Vec3::new(3.0, 1.0, -3.0), Vec3::Z, 7.9)
                .is_none()
        );
        // Straight down onto the floor (zero x/z direction components).
        let hit = world
            .raycast(Vec3::new(-7.3, 3.0, 1.1), -Vec3::Y, 10.0)
            .expect("hit");
        assert!((hit.distance - 3.0).abs() < 1e-4, "{hit:?}");
    }

    #[test]
    fn raycast_from_inside_a_box_misses_it() {
        let world = floor_world();
        // Starting inside the floor slab going up: only back faces are ahead.
        assert!(
            world
                .raycast(Vec3::new(0.0, -0.5, 0.0), Vec3::Y, 50.0)
                .is_none()
        );
        // Leaving the wall through its back face towards +Z: nothing beyond.
        assert!(
            world
                .raycast(Vec3::new(0.0, 2.0, 5.5), Vec3::Z, 50.0)
                .is_none()
        );
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
        assert!((position.y - REST).abs() < 1e-4, "root {}", position.y);
        assert!(hits.iter().all(|h| h.normal.y > 0.99));
    }

    #[test]
    fn controller_slides_along_wall() {
        let world = floor_world();
        let cc = controller();
        let mut position = Vec3::new(0.0, REST, 3.0);
        let mut hits = Vec::new();
        let flags = cc.move_by(
            &world,
            &mut position,
            Quat::IDENTITY,
            Vec3::new(2.0, -0.3, 4.0),
            &mut hits,
        );
        assert_ne!(flags & COLLIDED_SIDES, 0);
        // PhysX stops the contact offset short along the motion; the slide then runs parallel
        // to the wall, so the capsule ends at most a contact offset from it.
        let front = position.z + 0.06 + 0.36;
        assert!(front < 5.0 && front > 5.0 - 0.08, "front {front}");
        assert!(position.x > 1.5, "slid to x {}", position.x);
        assert!((position.y - REST).abs() < 1e-4, "root {}", position.y);
    }

    #[test]
    fn controller_steps_up_low_ledge_and_climbs_rounded_edges() {
        let mut world = CollisionWorld::new();
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(40.0, 1.0, 40.0),
            false,
            FLOOR_LAYER,
        );
        // A 0.2 m ledge from z = 1 to 9, a 0.5 m one from z = -1 to -9 and a 1 m wall at x = 3.
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.1, 5.0),
            Vec3::new(4.0, 0.2, 8.0),
            false,
            FLOOR_LAYER,
        );
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 0.25, -5.0),
            Vec3::new(4.0, 0.5, 8.0),
            false,
            FLOOR_LAYER,
        );
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(5.0, 0.5, 0.0),
            Vec3::new(4.0, 1.0, 40.0),
            false,
            FLOOR_LAYER,
        );
        let cc = controller();
        let mut hits = Vec::new();
        let mut walk = |motion: Vec3| {
            let mut position = Vec3::new(0.0, REST, 0.0);
            for _ in 0..60 {
                cc.move_by(&world, &mut position, Quat::IDENTITY, motion, &mut hits);
            }
            position
        };

        let position = walk(Vec3::new(0.0, -0.3, 0.1));
        assert!(
            (position.y - (REST + 0.2)).abs() < 1e-3,
            "0.2 m: {position}"
        );
        // With slopeLimit 90 PhysX never rejects a step: the rounded bottom rides up edges
        // that the up pass brings below the capsule's equator.
        let position = walk(Vec3::new(0.0, -0.3, -0.1));
        assert!(
            (position.y - (REST + 0.5)).abs() < 1e-3,
            "0.5 m: {position}"
        );
        let position = walk(Vec3::new(0.1, -0.3, 0.0));
        assert!((position.y - REST).abs() < 1e-3, "1 m wall: {position}");
        assert!(position.x < 3.0 - 0.36, "{position}");
    }

    #[test]
    fn controller_walks_under_low_ceiling() {
        let mut world = floor_world();
        // Ceiling 2.4 m up (bottom face down): leaves 2.4 - 2.2 - 0.08 = 0.12 m of headroom.
        world.add_box(
            Mat4::IDENTITY,
            Vec3::new(0.0, 2.9, 0.0),
            Vec3::new(4.0, 1.0, 4.0),
            false,
            FLOOR_LAYER,
        );
        let cc = controller();
        let mut position = Vec3::new(0.0, REST, -4.0);
        let mut hits = Vec::new();
        for _ in 0..100 {
            cc.move_by(
                &world,
                &mut position,
                Quat::IDENTITY,
                Vec3::new(0.0, -0.3, 0.1),
                &mut hits,
            );
        }
        assert!(position.z > 4.0, "stuck under the ceiling at {position}");
        assert!((position.y - REST).abs() < 1e-3, "{position}");
    }

    #[test]
    fn capsule_box_overlap() {
        let capsule = Capsule {
            p0: Vec3::new(0.0, 0.5, 0.0),
            p1: Vec3::new(0.0, 1.5, 0.0),
            radius: 0.3,
        };
        let world = Mat4::from_rotation_y(0.5);
        assert!(capsule_overlaps_box(
            &capsule,
            world,
            Vec3::new(0.0, 1.0, 0.7),
            Vec3::new(1.0, 1.0, 1.0)
        ));
        assert!(!capsule_overlaps_box(
            &capsule,
            world,
            Vec3::new(0.0, 1.0, 0.9),
            Vec3::new(1.0, 1.0, 1.0)
        ));
    }
}
