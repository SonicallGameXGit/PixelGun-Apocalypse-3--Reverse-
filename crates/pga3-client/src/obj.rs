//! Loader for the OBJ meshes in `assets/models/` (written by UnityPy's exporter).
//!
//! UnityPy mirrors meshes into a right-handed space when writing OBJ: it negates X of positions
//! and normals and reverses each triangle's winding (`f c b a`). This loader undoes both, so the
//! returned mesh is in the original Unity mesh space (left-handed, +Y up, clockwise front faces).
//! Every face corner uses the same index for v/vt/vn (`f i/i/i`), so one index buffer suffices.
//! The first `g` line is the mesh name; each following `g <name>_<n>` starts submesh `n`.

use anyhow::{Context, Result, bail};

pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Triangle index lists, one per Unity submesh, in submesh order.
    pub submeshes: Vec<Vec<u32>>,
}

fn parse_floats<const N: usize>(parts: &mut std::str::SplitWhitespace) -> Result<[f32; N]> {
    let mut out = [0.0; N];
    for value in out.iter_mut() {
        let text = parts.next().context("missing component")?;
        *value = text
            .parse()
            .with_context(|| format!("bad number {text:?}"))?;
    }
    Ok(out)
}

fn parse_corner(text: &str) -> Result<u32> {
    let first = text.split('/').next().unwrap_or(text);
    let index: u32 = first
        .parse()
        .with_context(|| format!("bad face index {text:?}"))?;
    if index == 0 {
        bail!("OBJ indices are 1-based, got 0");
    }
    Ok(index - 1)
}

pub fn parse_obj(text: &str) -> Result<MeshData> {
    let mut mesh = MeshData {
        positions: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        submeshes: Vec::new(),
    };
    let mut groups_seen = 0;
    for (line_number, line) in text.lines().enumerate() {
        let mut parts = line.split_whitespace();
        let Some(tag) = parts.next() else {
            continue;
        };
        let result: Result<()> = (|| {
            match tag {
                "v" => {
                    let [x, y, z] = parse_floats::<3>(&mut parts)?;
                    mesh.positions.push([-x, y, z]);
                }
                "vn" => {
                    let [x, y, z] = parse_floats::<3>(&mut parts)?;
                    mesh.normals.push([-x, y, z]);
                }
                "vt" => mesh.uvs.push(parse_floats::<2>(&mut parts)?),
                "g" => {
                    groups_seen += 1;
                    if groups_seen > 1 {
                        mesh.submeshes.push(Vec::new());
                    }
                }
                "f" => {
                    let c = parse_corner(parts.next().context("face corner")?)?;
                    let b = parse_corner(parts.next().context("face corner")?)?;
                    let a = parse_corner(parts.next().context("face corner")?)?;
                    if mesh.submeshes.is_empty() {
                        mesh.submeshes.push(Vec::new());
                    }
                    let current = mesh.submeshes.last_mut().context("no submesh")?;
                    current.extend_from_slice(&[a, b, c]);
                }
                _ => {}
            }
            Ok(())
        })();
        result.with_context(|| format!("OBJ line {}", line_number + 1))?;
    }
    let count = mesh.positions.len();
    if let Some(bad) = mesh
        .submeshes
        .iter()
        .flatten()
        .find(|&&i| i as usize >= count)
    {
        bail!("face index {bad} out of range ({count} vertices)");
    }
    if mesh.normals.len() != count {
        mesh.normals = vec![[0.0, 1.0, 0.0]; count];
    }
    if mesh.uvs.len() != count {
        mesh.uvs = vec![[0.0, 0.0]; count];
    }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undoes_unitypy_mirroring() {
        let text = "g Quad\nv -1 2 3\nv -4 5 6\nv -7 8 9\nvt 0 0\nvt 1 0\nvt 1 1\n\
                    vn -1 0 0\nvn -1 0 0\nvn -1 0 0\ng Quad_0\nf 3/3/3 2/2/2 1/1/1\n\
                    g Quad_1\nf 1/1/1 2/2/2 3/3/3\n";
        let mesh = parse_obj(text).unwrap();
        assert_eq!(mesh.positions[0], [1.0, 2.0, 3.0]);
        assert_eq!(mesh.normals[0], [1.0, 0.0, 0.0]);
        assert_eq!(mesh.submeshes, vec![vec![0, 1, 2], vec![2, 1, 0]]);
    }
}
