//! Skinning data of the skinned meshes (`assets/data/skins/<model stem>.json`, written by
//! `tools/extract/export_skins.py`) and Unity's per-quality bone limit.

use std::path::Path;

use anyhow::{Context, Result, bail};
use glam::Mat4;
use serde_json::Value;

use crate::assets::{f32_of, load_json};

pub struct SkinData {
    /// Mesh space → bone space, one per `m_Bones` entry.
    pub bind_poses: Vec<Mat4>,
    /// Per vertex, already limited to the quality level's bone count (see `limit_weights`).
    pub bone_indices: Vec<[u32; 4]>,
    pub bone_weights: Vec<[f32; 4]>,
}

/// Path of the skin sidecar of a published mesh (`models/Box005.obj` → `data/skins/Box005.json`).
pub fn skin_path(mesh_asset: &str) -> String {
    let stem = Path::new(mesh_asset)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    format!("data/skins/{stem}.json")
}

/// `QualitySettings.blendWeights` of the active quality level (1, 2 or 4 bones per vertex).
pub fn quality_bone_count(assets: &Path) -> Result<usize> {
    let json = load_json(&assets.join("data/settings/QualitySettings.json"))?;
    let data = json.get("data").unwrap_or(&json);
    let current = data["m_CurrentQuality"]
        .as_u64()
        .context("m_CurrentQuality")? as usize;
    let level = &data["m_QualitySettings"][current];
    Ok(match level["blendWeights"].as_u64() {
        Some(1) => 1,
        Some(2) => 2,
        _ => 4,
    })
}

/// Unity's skin-weight limit: keeps the first `bones` influences (Unity stores them sorted by
/// weight, largest first) and renormalises them; `OneBone` therefore uses influence 0 at full
/// weight.
pub fn limit_weights(indices: [u32; 4], weights: [f32; 4], bones: usize) -> ([u32; 4], [f32; 4]) {
    let bones = bones.clamp(1, 4);
    let sum: f32 = weights[..bones].iter().sum();
    let mut out_indices = [0; 4];
    let mut out_weights = [0.0; 4];
    for i in 0..bones {
        out_indices[i] = indices[i];
        out_weights[i] = if sum > 0.0 {
            weights[i] / sum
        } else if i == 0 {
            1.0
        } else {
            0.0
        };
    }
    (out_indices, out_weights)
}

fn row_major_matrix(value: &Value) -> Result<Mat4> {
    let values: Vec<f32> = value
        .as_array()
        .context("bind pose is not an array")?
        .iter()
        .map(f32_of)
        .collect();
    let Ok(m) = <[f32; 16]>::try_from(values) else {
        bail!("bind pose needs 16 values");
    };
    Ok(Mat4::from_cols_array(&m).transpose())
}

pub fn load_skin(assets: &Path, mesh_asset: &str, bones: usize) -> Result<SkinData> {
    let path = assets.join(skin_path(mesh_asset));
    let json = load_json(&path)?;
    let bind_poses = json["bind_poses"]
        .as_array()
        .context("bind_poses")?
        .iter()
        .map(row_major_matrix)
        .collect::<Result<Vec<_>>>()?;
    let indices = json["bone_indices"].as_array().context("bone_indices")?;
    let weights = json["bone_weights"].as_array().context("bone_weights")?;
    if indices.len() != weights.len() {
        bail!("{}: index/weight count mismatch", path.display());
    }
    let mut bone_indices = Vec::with_capacity(indices.len());
    let mut bone_weights = Vec::with_capacity(indices.len());
    for (i, w) in indices.iter().zip(weights) {
        let mut ix = [0u32; 4];
        let mut wx = [0f32; 4];
        for k in 0..4 {
            ix[k] = i[k].as_u64().unwrap_or(0) as u32;
            wx[k] = f32_of(&w[k]);
        }
        if ix.iter().any(|&b| b as usize >= bind_poses.len()) {
            bail!("{}: bone index out of range", path.display());
        }
        let (ix, wx) = limit_weights(ix, wx, bones);
        bone_indices.push(ix);
        bone_weights.push(wx);
    }
    Ok(SkinData {
        bind_poses,
        bone_indices,
        bone_weights,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    #[test]
    fn one_bone_quality_keeps_first_influence() {
        let (i, w) = limit_weights([3, 1, 0, 0], [0.7, 0.3, 0.0, 0.0], 1);
        assert_eq!(i, [3, 0, 0, 0]);
        assert_eq!(w, [1.0, 0.0, 0.0, 0.0]);
        let (_, w) = limit_weights([3, 1, 0, 0], [0.6, 0.2, 0.2, 0.0], 2);
        assert!((w[0] - 0.75).abs() < 1e-6 && (w[1] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn game_uses_one_bone_skinning() {
        // Quality level "Good" (current) has blendWeights = 1.
        let assets = find_assets_dir().unwrap();
        assert_eq!(quality_bone_count(&assets).unwrap(), 1);
    }

    #[test]
    fn skins_match_their_meshes() {
        let assets = find_assets_dir().unwrap();
        for mesh in ["models/Box005.obj", "models/Base_mesh.obj"] {
            let skin = load_skin(&assets, mesh, 4).unwrap();
            let text = std::fs::read_to_string(assets.join(mesh)).unwrap();
            let obj = crate::obj::parse_obj(&text).unwrap();
            assert_eq!(skin.bone_indices.len(), obj.positions.len(), "{mesh}");
            // Bind pose · bind-pose inverse sanity: matrices are invertible rigid transforms.
            for m in &skin.bind_poses {
                assert!((m.determinant().abs() - 1.0).abs() < 0.05, "{mesh}");
            }
        }
    }
}
