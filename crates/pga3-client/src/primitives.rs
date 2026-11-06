//! Unity's built-in primitive meshes (`unity_default_resources`), generated in code.
//!
//! The game builds its weapons and much of its characters from Unity's Cube and Cylinder. Those
//! meshes belong to Unity, so they are not published in `assets/` (see `tools/extract`); this
//! module rebuilds them with Unity's vertex order, positions, normals and triangles. The test
//! compares against the local raw extraction (`research/extracted/Mesh`) when it exists.
//! Cube is exact; Cylinder positions/normals/triangles are exact and its UVs (an FBX import with
//! ~5e-4 offsets) match within 1e-5.

use crate::obj::MeshData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Primitive {
    Cube,
    Cylinder,
}

impl Primitive {
    /// Built-in mesh referenced as `unity_default_resources:<path_id>` in the extracted data.
    pub fn from_ref(reference: &str) -> Option<Primitive> {
        match reference {
            "unity_default_resources:10202" => Some(Primitive::Cube),
            "unity_default_resources:10206" => Some(Primitive::Cylinder),
            _ => None,
        }
    }

    pub fn mesh(self) -> MeshData {
        match self {
            Primitive::Cube => cube(),
            Primitive::Cylinder => cylinder(),
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
        for primitive in [Primitive::Cube, Primitive::Cylinder] {
            let mesh = primitive.mesh();
            let count = mesh.positions.len();
            assert_eq!(mesh.normals.len(), count);
            assert_eq!(mesh.uvs.len(), count);
            assert!(mesh.submeshes[0].iter().all(|&i| (i as usize) < count));
        }
        assert_eq!(Primitive::Cube.mesh().submeshes[0].len(), 36);
        assert_eq!(Primitive::Cylinder.mesh().positions.len(), 88);
        assert_eq!(Primitive::Cylinder.mesh().submeshes[0].len(), 240);
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
    }
}
