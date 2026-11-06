//! Unity's built-in primitive meshes (`unity_default_resources`), generated in code.
//!
//! The game builds its weapons and much of its characters from Unity's Cube and Cylinder. Those
//! meshes belong to Unity, so they are not published in `assets/` (see `tools/extract`); this
//! module rebuilds them with Unity's vertex order, positions, normals and triangles. The test
//! compares against the local raw extraction (`research/extracted/Mesh`) when it exists.
//! Cube is exact; Cylinder positions/normals/triangles are exact and its UVs (an FBX import with
//! ~5e-4 offsets) match within 1e-5. Plane (10 × 10 quads, used by the weapons' red-dot sights) is
//! a regular grid whose triangle order comes from Unity's mesh optimiser, so its index list is
//! stored as data.

use crate::obj::MeshData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Primitive {
    Cube,
    Cylinder,
    Plane,
}

impl Primitive {
    /// Built-in mesh referenced as `unity_default_resources:<path_id>` in the extracted data.
    pub fn from_ref(reference: &str) -> Option<Primitive> {
        match reference {
            "unity_default_resources:10202" => Some(Primitive::Cube),
            "unity_default_resources:10206" => Some(Primitive::Cylinder),
            "unity_default_resources:10209" => Some(Primitive::Plane),
            _ => None,
        }
    }

    pub fn mesh(self) -> MeshData {
        match self {
            Primitive::Cube => cube(),
            Primitive::Cylinder => cylinder(),
            Primitive::Plane => plane(),
        }
    }
}

fn cube() -> MeshData {
    const P: f32 = 0.5;
    const N: f32 = -0.5;
    let positions = vec![
        [P, N, P],
        [N, N, P],
        [P, P, P],
        [N, P, P],
        [P, P, N],
        [N, P, N],
        [P, N, N],
        [N, N, N],
        [P, P, P],
        [N, P, P],
        [P, P, N],
        [N, P, N],
        [P, N, N],
        [P, N, P],
        [N, N, P],
        [N, N, N],
        [N, N, P],
        [N, P, P],
        [N, P, N],
        [N, N, N],
        [P, N, N],
        [P, P, N],
        [P, P, P],
        [P, N, P],
    ];
    let face_normals: [([f32; 3], usize); 7] = [
        ([0.0, 0.0, 1.0], 4),
        ([0.0, 1.0, 0.0], 2),
        ([0.0, 0.0, -1.0], 2),
        ([0.0, 1.0, 0.0], 2),
        ([0.0, 0.0, -1.0], 2),
        ([0.0, -1.0, 0.0], 4),
        ([-1.0, 0.0, 0.0], 4),
    ];
    let mut normals = Vec::new();
    for (normal, count) in face_normals {
        normals.extend(std::iter::repeat_n(normal, count));
    }
    normals.extend(std::iter::repeat_n([1.0, 0.0, 0.0], 4));
    let uvs = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
    ];
    let triangles = vec![
        0, 2, 3, 0, 3, 1, 8, 4, 5, 8, 5, 9, 10, 6, 7, 10, 7, 11, 12, 13, 14, 12, 14, 15, 16, 17,
        18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];
    MeshData {
        positions,
        normals,
        uvs,
        submeshes: vec![triangles],
    }
}

/// 20 segments, radius 0.5, height 2. Side vertex `i` sits at angle `-162° + 18°·i` (x = cos,
/// z = sin); the side texture wraps twice around, the caps are planar-mapped.
fn cylinder() -> MeshData {
    const SEGMENTS: usize = 20;
    // UV constants of Unity's Cylinder (an imported FBX).
    const U0: f32 = 0.000694;
    const U_STEP: f32 = (0.999749 - U0) / 10.0;
    const V_BOTTOM: f32 = 0.000454;
    const V_TOP: f32 = 0.999431;
    const CAP_CENTER: [f32; 2] = [0.500224, 0.499944];
    const CAP_RADIUS: f32 = 0.496116;

    let angle = |i: usize| (-162.0 + 18.0 * i as f32).to_radians();
    let ring = |i: usize, y: f32| {
        let a = angle(i);
        [0.5 * a.cos(), y, 0.5 * a.sin()]
    };
    let side_normal = |i: usize| {
        let a = angle(i);
        [a.cos(), 0.0, a.sin()]
    };
    let side_u = |i: usize| U0 + ((i % 10) + 1) as f32 * U_STEP;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut push = |p: [f32; 3], n: [f32; 3], uv: [f32; 2]| {
        positions.push(p);
        normals.push(n);
        uvs.push(uv);
    };
    // 0..20 bottom side ring, 20..40 top side ring.
    for (y, v) in [(-1.0, V_BOTTOM), (1.0, V_TOP)] {
        for i in 0..SEGMENTS {
            push(ring(i, y), side_normal(i), [side_u(i), v]);
        }
    }
    // 40/41 cap centres, 42..46 seam duplicates at 0° and 180° with u = U0, 46/47 copies of 20/0.
    push([0.0, -1.0, 0.0], [0.0, -1.0, 0.0], CAP_CENTER);
    push([0.0, 1.0, 0.0], [0.0, 1.0, 0.0], CAP_CENTER);
    push([0.5, -1.0, 0.0], [1.0, 0.0, 0.0], [U0, V_BOTTOM]);
    push([0.5, 1.0, 0.0], [1.0, 0.0, 0.0], [U0, V_TOP]);
    push([-0.5, -1.0, 0.0], [-1.0, 0.0, 0.0], [U0, V_BOTTOM]);
    push([-0.5, 1.0, 0.0], [-1.0, 0.0, 0.0], [U0, V_TOP]);
    push(ring(0, 1.0), side_normal(0), [side_u(0), V_TOP]);
    push(ring(0, -1.0), side_normal(0), [side_u(0), V_BOTTOM]);
    // 48..68 bottom cap ring (segment order 1, 0, 2, 3, ..., 19), 68..88 top cap ring.
    let bottom_order = [1, 0].into_iter().chain(2..SEGMENTS);
    for i in bottom_order {
        let a = angle(i);
        let uv = [
            CAP_CENTER[0] + CAP_RADIUS * a.cos(),
            CAP_CENTER[1] - CAP_RADIUS * a.sin(),
        ];
        push(ring(i, -1.0), [0.0, -1.0, 0.0], uv);
    }
    for i in 0..SEGMENTS {
        let a = angle(i);
        let uv = [
            CAP_CENTER[0] - CAP_RADIUS * a.cos(),
            CAP_CENTER[1] - CAP_RADIUS * a.sin(),
        ];
        push(ring(i, 1.0), [0.0, 1.0, 0.0], uv);
    }

    let mut triangles: Vec<u32> = Vec::new();
    let mut quad =
        |a: u32, b: u32, c: u32, d: u32| triangles.extend_from_slice(&[a, b, c, a, c, d]);
    for i in 0..9 {
        quad(i, 20 + i, 21 + i, i + 1);
    }
    quad(42, 43, 30, 10);
    for i in 10..19 {
        quad(i, 20 + i, 21 + i, i + 1);
    }
    quad(44, 45, 46, 47);
    triangles.extend_from_slice(&[48, 40, 49]);
    triangles.extend_from_slice(&[50, 40, 48]);
    for k in 51..68 {
        triangles.extend_from_slice(&[k, 40, k - 1]);
    }
    triangles.extend_from_slice(&[49, 40, 67]);
    for j in 0..19 {
        triangles.extend_from_slice(&[68 + j, 41, 69 + j]);
    }
    triangles.extend_from_slice(&[87, 41, 68]);

    MeshData {
        positions,
        normals,
        uvs,
        submeshes: vec![triangles],
    }
}

/// Triangles of Unity's Plane (vertex `11 · row + column`), in the original order.
#[rustfmt::skip]
const PLANE_TRIANGLES: [u32; 600] = [
    0, 11, 12, 0, 12, 1, 1, 12, 13, 1, 13, 2, 2, 13, 14, 2, 14, 3, 3, 14, 15, 3, 15, 4, 4, 15, 16,
    4, 16, 5, 5, 16, 17, 5, 17, 6, 6, 17, 18, 6, 18, 7, 7, 18, 19, 7, 19, 8, 8, 19, 20, 8, 20, 9, 9,
    20, 21, 9, 21, 10, 14, 26, 15, 15, 26, 27, 15, 27, 16, 16, 27, 28, 16, 28, 17, 17, 28, 29, 17,
    29, 18, 18, 29, 30, 18, 30, 19, 19, 30, 31, 19, 31, 20, 20, 31, 32, 20, 32, 21, 26, 38, 27, 26,
    37, 38, 25, 37, 26, 14, 25, 26, 13, 25, 14, 13, 24, 25, 12, 24, 13, 12, 23, 24, 11, 23, 12, 11,
    22, 23, 22, 34, 23, 22, 33, 34, 23, 34, 35, 23, 35, 24, 24, 35, 36, 24, 36, 25, 25, 36, 37, 31,
    43, 32, 33, 45, 34, 33, 44, 45, 34, 45, 46, 34, 46, 35, 35, 46, 47, 35, 47, 36, 36, 47, 48, 36,
    48, 37, 37, 48, 49, 37, 49, 38, 38, 49, 50, 38, 50, 39, 27, 38, 39, 27, 39, 28, 28, 39, 40, 28,
    40, 29, 29, 40, 41, 29, 41, 30, 30, 41, 42, 30, 42, 31, 31, 42, 43, 39, 50, 51, 39, 51, 40, 40,
    51, 52, 40, 52, 41, 41, 52, 53, 41, 53, 42, 42, 53, 54, 42, 54, 43, 50, 62, 51, 50, 61, 62, 51,
    62, 63, 51, 63, 52, 52, 63, 64, 52, 64, 53, 53, 64, 65, 53, 65, 54, 61, 73, 62, 61, 72, 73, 60,
    72, 61, 49, 60, 61, 48, 60, 49, 48, 59, 60, 47, 59, 48, 47, 58, 59, 46, 58, 47, 46, 57, 58, 45,
    57, 46, 45, 56, 57, 44, 56, 45, 44, 55, 56, 55, 67, 56, 55, 66, 67, 56, 67, 68, 56, 68, 57, 57,
    68, 69, 57, 69, 58, 58, 69, 70, 58, 70, 59, 59, 70, 71, 59, 71, 60, 60, 71, 72, 66, 78, 67, 66,
    77, 78, 67, 78, 79, 67, 79, 68, 68, 79, 80, 68, 80, 69, 69, 80, 81, 69, 81, 70, 70, 81, 82, 70,
    82, 71, 71, 82, 83, 71, 83, 72, 72, 83, 84, 72, 84, 73, 73, 84, 85, 73, 85, 74, 62, 73, 74, 62,
    74, 63, 63, 74, 75, 63, 75, 64, 64, 75, 76, 64, 76, 65, 74, 85, 86, 74, 86, 75, 75, 86, 87, 75,
    87, 76, 80, 92, 81, 81, 92, 93, 81, 93, 82, 82, 93, 94, 82, 94, 83, 83, 94, 95, 83, 95, 84, 84,
    95, 96, 84, 96, 85, 85, 96, 97, 85, 97, 86, 86, 97, 98, 86, 98, 87, 92, 104, 93, 92, 103, 104,
    91, 103, 92, 80, 91, 92, 79, 91, 80, 79, 90, 91, 78, 90, 79, 78, 89, 90, 77, 89, 78, 77, 88, 89,
    88, 100, 89, 88, 99, 100, 89, 100, 101, 89, 101, 90, 90, 101, 102, 90, 102, 91, 91, 102, 103,
    97, 109, 98, 99, 111, 100, 99, 110, 111, 100, 111, 112, 100, 112, 101, 101, 112, 113, 101, 113,
    102, 102, 113, 114, 102, 114, 103, 103, 114, 115, 103, 115, 104, 104, 115, 116, 104, 116, 105,
    93, 104, 105, 93, 105, 94, 94, 105, 106, 94, 106, 95, 95, 106, 107, 95, 107, 96, 96, 107, 108,
    96, 108, 97, 97, 108, 109, 105, 116, 117, 105, 117, 106, 106, 117, 118, 106, 118, 107, 107, 118,
    119, 107, 119, 108, 108, 119, 120, 108, 120, 109, 49, 61, 50
];

/// 10 × 10 m, 11 × 11 vertices on y = 0 facing +Y. Vertex `11 · row + column` sits at
/// (5 - column, 0, 5 - row) with UV (column / 10, row / 10).
fn plane() -> MeshData {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    for row in 0..11 {
        for column in 0..11 {
            positions.push([5.0 - column as f32, 0.0, 5.0 - row as f32]);
            uvs.push([column as f32 * 0.1, row as f32 * 0.1]);
        }
    }
    MeshData {
        normals: vec![[0.0, 1.0, 0.0]; positions.len()],
        positions,
        uvs,
        submeshes: vec![PLANE_TRIANGLES.to_vec()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::parse_obj;

    fn reference(name: &str) -> Option<MeshData> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../research/extracted/Mesh")
            .join(name);
        let text = std::fs::read_to_string(path).ok()?;
        Some(parse_obj(&text).expect("reference OBJ parses"))
    }

    fn assert_close(generated: &MeshData, original: &MeshData, uv_tolerance: f32) {
        assert_eq!(generated.positions.len(), original.positions.len());
        assert_eq!(generated.submeshes, original.submeshes);
        for i in 0..generated.positions.len() {
            for k in 0..3 {
                assert!(
                    (generated.positions[i][k] - original.positions[i][k]).abs() < 1e-5,
                    "position {i}"
                );
                assert!(
                    (generated.normals[i][k] - original.normals[i][k]).abs() < 1e-4,
                    "normal {i}"
                );
            }
            for k in 0..2 {
                assert!(
                    (generated.uvs[i][k] - original.uvs[i][k]).abs() <= uv_tolerance,
                    "uv {i}: {:?} vs {:?}",
                    generated.uvs[i],
                    original.uvs[i]
                );
            }
        }
    }

    #[test]
    fn shapes_are_consistent() {
        for primitive in [Primitive::Cube, Primitive::Cylinder, Primitive::Plane] {
            let mesh = primitive.mesh();
            let count = mesh.positions.len();
            assert_eq!(mesh.normals.len(), count);
            assert_eq!(mesh.uvs.len(), count);
            assert!(mesh.submeshes[0].iter().all(|&i| (i as usize) < count));
        }
        assert_eq!(Primitive::Cube.mesh().submeshes[0].len(), 36);
        assert_eq!(Primitive::Cylinder.mesh().positions.len(), 88);
        assert_eq!(Primitive::Cylinder.mesh().submeshes[0].len(), 240);
        assert_eq!(Primitive::Plane.mesh().positions.len(), 121);
    }

    /// Runs only where the raw extraction exists (it is gitignored).
    #[test]
    fn matches_unity_builtin_meshes() {
        if let Some(original) = reference("Cube__10202.obj") {
            assert_close(&Primitive::Cube.mesh(), &original, 0.0);
        }
        if let Some(original) = reference("Cylinder.obj") {
            assert_close(&Primitive::Cylinder.mesh(), &original, 1e-5);
        }
        if let Some(original) = reference("Plane.obj") {
            assert_close(&Primitive::Plane.mesh(), &original, 1e-6);
        }
    }
}
