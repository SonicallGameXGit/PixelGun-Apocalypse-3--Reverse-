//! Runtime instances of prefabs from `assets/data/prefabs/` (`resources/<name>.json`, the
//! `Resources.Load` prefabs, and `resources.assets/*.json`, prefabs referenced by scripts such as
//! bullets): the GameObject hierarchy (local transforms, active flags, layers), its renderers
//! (MeshRenderer and SkinnedMeshRenderer) and its legacy `Animation` components.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use glam::{Mat4, Quat, Vec3};
use serde_json::Value;

use crate::anim::{Animation, ClipLibrary, Trs, WrapMode};
use crate::assets::{f32_of, load_json, ref_of, vec4_of};
use crate::primitives::Primitive;

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub parent: Option<usize>,
    /// In `Transform.m_Children` order (= `GetChild` order).
    pub children: Vec<usize>,
    pub local: Trs,
    pub active_self: bool,
    pub layer: u32,
    /// `GameObject.tag` (TagManager number: 0-6 built-in, 20000+ custom).
    pub tag: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeshSource {
    /// Published mesh under `assets/` (`models/*.obj`).
    Asset(String),
    Builtin(Primitive),
}

#[derive(Clone, Debug)]
pub struct Skin {
    /// `m_Bones`, as node indices.
    pub bones: Vec<Option<usize>>,
}

#[derive(Clone, Debug)]
pub struct RendererInstance {
    pub node: usize,
    pub enabled: bool,
    pub mesh: MeshSource,
    /// Material refs (keys of materials.json), index = submesh slot.
    pub materials: Vec<Option<String>>,
    /// `Some` for a SkinnedMeshRenderer.
    pub skin: Option<Skin>,
}

#[derive(Clone, Copy, Debug)]
pub struct CameraSettings {
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
    pub culling_mask: u32,
    /// 1 = Skybox, 2 = Solid colour, 3 = Depth only, 4 = Nothing.
    pub clear_flags: i64,
}

#[derive(Clone)]
pub struct PrefabInstance {
    pub nodes: Vec<Node>,
    pub renderers: Vec<RendererInstance>,
    pub animations: Vec<Animation>,
    /// World matrices, filled by `update_world`.
    pub world: Vec<Mat4>,
    /// Nodes whose world matrix is driven from outside (the player root, the camera).
    pub world_override: Vec<Option<Mat4>>,
    /// Renderers skipped because their mesh is neither published nor a supported primitive.
    pub skipped_renderers: usize,
    json: Value,
    node_by_ref: HashMap<String, usize>,
    /// Component ref → (node, index into the node's `components`).
    /// Component ref → node carrying it.
    component_by_ref: HashMap<String, usize>,
}

fn vec3_of(value: &Value) -> Vec3 {
    Vec3::new(
        f32_of(&value["x"]),
        f32_of(&value["y"]),
        f32_of(&value["z"]),
    )
}

fn local_trs(node: &Value) -> Trs {
    let transform = node["components"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c["type"] == "Transform" || c["type"] == "RectTransform")
        .map(|c| &c["data"]);
    match transform {
        Some(t) => {
            let q = vec4_of(&t["m_LocalRotation"], ["x", "y", "z", "w"]);
            let rotation = Quat::from_xyzw(q.x, q.y, q.z, q.w);
            Trs {
                position: vec3_of(&t["m_LocalPosition"]),
                rotation: if rotation.length_squared() > 0.0 {
                    rotation.normalize()
                } else {
                    Quat::IDENTITY
                },
                scale: vec3_of(&t["m_LocalScale"]),
            }
        }
        None => Trs {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    }
}

fn enabled_flag(data: &Value) -> bool {
    match &data["m_Enabled"] {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_i64() != Some(0),
        _ => true,
    }
}

fn mesh_source(mesh: &Value) -> Option<MeshSource> {
    let (reference, asset) = ref_of(mesh)?;
    match asset {
        Some(path) => Some(MeshSource::Asset(path.to_string())),
        None => Primitive::from_ref(reference).map(MeshSource::Builtin),
    }
}

impl Trs {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

impl PrefabInstance {
    /// An instance with no nodes (a placeholder).
    pub fn empty() -> PrefabInstance {
        PrefabInstance {
            nodes: Vec::new(),
            renderers: Vec::new(),
            animations: Vec::new(),
            world: Vec::new(),
            world_override: Vec::new(),
            skipped_renderers: 0,
            json: Value::Null,
            node_by_ref: HashMap::new(),
            component_by_ref: HashMap::new(),
        }
    }

    /// Loads `data/prefabs/resources/<name>.json` and builds its animations from `clips`.
    pub fn load(assets: &Path, name: &str, clips: &mut ClipLibrary) -> Result<PrefabInstance> {
        PrefabInstance::load_file(assets, &format!("prefabs/resources/{name}.json"), clips)
    }

    /// Loads a prefab file given relative to `assets/data/`.
    pub fn load_file(assets: &Path, path: &str, clips: &mut ClipLibrary) -> Result<PrefabInstance> {
        let json = load_json(&assets.join("data").join(path))?;
        let nodes_json = json["nodes"].as_array().context("prefab has no nodes")?;

        let mut node_by_ref = HashMap::new();
        let mut component_by_ref = HashMap::new();
        for (index, node) in nodes_json.iter().enumerate() {
            let reference = node["ref"].as_str().context("node without ref")?;
            node_by_ref.insert(reference.to_string(), index);
            for component in node["components"].as_array().into_iter().flatten() {
                if let Some(r) = component["ref"].as_str() {
                    component_by_ref.insert(r.to_string(), index);
                }
            }
        }
        let mut nodes = Vec::with_capacity(nodes_json.len());
        for node in nodes_json {
            nodes.push(Node {
                name: node["name"].as_str().unwrap_or_default().to_string(),
                parent: node["parent"]
                    .as_str()
                    .and_then(|p| node_by_ref.get(p).copied()),
                children: node["children"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|c| c.as_str().and_then(|r| node_by_ref.get(r).copied()))
                    .collect(),
                local: local_trs(node),
                active_self: node["active"].as_bool().unwrap_or(true),
                layer: node["layer"].as_u64().unwrap_or(0) as u32,
                tag: node["tag"].as_u64().unwrap_or(0) as u32,
            });
        }

        let mut instance = PrefabInstance {
            world: vec![Mat4::IDENTITY; nodes.len()],
            world_override: vec![None; nodes.len()],
            nodes,
            renderers: Vec::new(),
            animations: Vec::new(),
            skipped_renderers: 0,
            node_by_ref,
            component_by_ref,
            json: Value::Null,
        };

        for (index, node) in nodes_json.iter().enumerate() {
            let components = node["components"].as_array().cloned().unwrap_or_default();
            let find = |kind: &str| components.iter().find(|c| c["type"] == kind);
            if let (Some(filter), Some(renderer)) = (find("MeshFilter"), find("MeshRenderer")) {
                instance.add_renderer(index, &filter["data"]["m_Mesh"], &renderer["data"], None);
            }
            if let Some(renderer) = find("SkinnedMeshRenderer") {
                let data = &renderer["data"];
                let skin = Skin {
                    bones: data["m_Bones"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|b| instance.node_of_ref(b))
                        .collect(),
                };
                instance.add_renderer(index, &data["m_Mesh"], data, Some(skin));
            }
            if let Some(animation) = find("Animation") {
                let data = &animation["data"];
                let mut clip_list = Vec::new();
                for clip_ref in data["m_Animations"].as_array().into_iter().flatten() {
                    if let Some((reference, _)) = ref_of(clip_ref)
                        && let Some(clip) = clips.get(reference)?
                    {
                        clip_list.push(clip);
                    }
                }
                let default_clip = match ref_of(&data["m_Animation"]) {
                    Some((reference, _)) => clips.get(reference)?.map(|c| c.name.clone()),
                    None => None,
                };
                let nodes = &instance.nodes;
                let animation = Animation::new(
                    index,
                    WrapMode::from_unity(data["m_WrapMode"].as_i64().unwrap_or(0)),
                    data["m_PlayAutomatically"].as_bool().unwrap_or(false),
                    default_clip,
                    &clip_list,
                    |path| find_path(nodes, index, path),
                );
                if enabled_flag(data) {
                    instance.animations.push(animation);
                }
            }
        }
        instance.json = json;
        instance.update_world();
        Ok(instance)
    }

    fn add_renderer(&mut self, node: usize, mesh: &Value, renderer: &Value, skin: Option<Skin>) {
        let Some(mesh) = mesh_source(mesh) else {
            self.skipped_renderers += 1;
            return;
        };
        self.renderers.push(RendererInstance {
            node,
            enabled: enabled_flag(renderer),
            mesh,
            materials: renderer["m_Materials"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|m| ref_of(m).map(|(r, _)| r.to_string()))
                .collect(),
            skin,
        });
    }

    pub fn node_of_ref(&self, value: &Value) -> Option<usize> {
        let (reference, _) = ref_of(value)?;
        self.node_by_ref
            .get(reference)
            .copied()
            .or_else(|| self.component_by_ref.get(reference).copied())
    }

    /// MonoBehaviours with this script on a node: (component index, component JSON).
    pub fn scripts<'a>(&'a self, node: usize, script: &'a str) -> impl Iterator<Item = &'a Value> {
        self.json["nodes"][node]["components"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(move |c| c["script"] == script)
    }

    /// Every (node, component) carrying `script`, in hierarchy order.
    pub fn find_scripts<'a>(&'a self, script: &'a str) -> Vec<(usize, &'a Value)> {
        (0..self.nodes.len())
            .flat_map(|node| self.scripts(node, script).map(move |c| (node, c)))
            .collect()
    }

    /// Built-in component data (`Camera`, `Transform`, ...) of a node.
    pub fn component(&self, node: usize, kind: &str) -> Option<&Value> {
        self.json["nodes"][node]["components"]
            .as_array()?
            .iter()
            .find(|c| c["type"] == kind)
            .map(|c| &c["data"])
    }

    /// The data of the component `reference` on a node, if it has that type.
    pub fn component_by_ref(&self, node: usize, reference: &str, kind: &str) -> Option<&Value> {
        self.json["nodes"][node]["components"]
            .as_array()?
            .iter()
            .find(|c| c["ref"] == reference && c["type"] == kind)
            .map(|c| &c["data"])
    }

    /// Every built-in component of this type on a node (a node can carry several AudioSources).
    pub fn components<'a>(&'a self, node: usize, kind: &'a str) -> impl Iterator<Item = &'a Value> {
        self.json["nodes"][node]["components"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(move |c| c["type"] == kind)
            .map(|c| &c["data"])
    }

    /// Settings of the `Camera` on a node.
    pub fn camera(&self, node: usize) -> Option<CameraSettings> {
        let c = self.component(node, "Camera")?;
        Some(CameraSettings {
            fov_y_degrees: f32_of(&c["field of view"]),
            near: f32_of(&c["near clip plane"]),
            far: f32_of(&c["far clip plane"]),
            culling_mask: c["m_CullingMask"]["m_Bits"]
                .as_u64()
                .unwrap_or(u64::from(u32::MAX)) as u32,
            clear_flags: c["m_ClearFlags"].as_i64().unwrap_or(1),
        })
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.name == name)
    }

    /// `node` and every node below it (depth-first, `node` first).
    pub fn subtree(&self, node: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            out.push(current);
            stack.extend(self.nodes[current].children.iter().rev());
        }
        out
    }

    pub fn active_in_hierarchy(&self, node: usize) -> bool {
        let mut current = Some(node);
        while let Some(index) = current {
            if !self.nodes[index].active_self {
                return false;
            }
            current = self.nodes[index].parent;
        }
        true
    }

    /// Fields of the first `script` MonoBehaviour on a node.
    pub fn script_fields(&self, node: usize, script: &str) -> Option<&Value> {
        self.json["nodes"][node]["components"]
            .as_array()?
            .iter()
            .find(|c| c["script"] == script)
            .map(|c| &c["fields"])
    }

    /// `Transform.localToWorldMatrix` from the current local transforms (`world` is only
    /// refreshed by `update_world`; scripts read transforms mid-frame).
    pub fn world_of(&self, node: usize) -> Mat4 {
        if let Some(m) = self.world_override[node] {
            return m;
        }
        let local = self.nodes[node].local.matrix();
        match self.nodes[node].parent {
            Some(parent) => self.world_of(parent) * local,
            None => local,
        }
    }

    /// `Transform.rotation` (world) of a node.
    pub fn world_rotation(&self, node: usize) -> Quat {
        let (_, rotation, _) = self.world_of(node).to_scale_rotation_translation();
        rotation.normalize()
    }

    /// `Transform.rotation = rotation`: sets the local rotation relative to the parent.
    pub fn set_world_rotation(&mut self, node: usize, rotation: Quat) {
        let parent = match self.nodes[node].parent {
            Some(parent) => self.world_rotation(parent),
            None => Quat::IDENTITY,
        };
        self.nodes[node].local.rotation = (parent.inverse() * rotation).normalize();
    }

    pub fn animation_on(&mut self, node: usize) -> Option<&mut Animation> {
        self.animations.iter_mut().find(|a| a.node == node)
    }

    /// Recomputes `world` (parents come before children in the exported node order; a parent
    /// appearing later is handled by resolving recursively).
    pub fn update_world(&mut self) {
        let mut done = vec![false; self.nodes.len()];
        for index in 0..self.nodes.len() {
            self.resolve_world(index, &mut done);
        }
    }

    fn resolve_world(&mut self, index: usize, done: &mut [bool]) -> Mat4 {
        if done[index] {
            return self.world[index];
        }
        let world = match (self.world_override[index], self.nodes[index].parent) {
            (Some(m), _) => m,
            (None, Some(parent)) => {
                self.resolve_world(parent, done) * self.nodes[index].local.matrix()
            }
            (None, None) => self.nodes[index].local.matrix(),
        };
        self.world[index] = world;
        done[index] = true;
        world
    }

    /// Advances every active `Animation` and writes the poses into the local transforms
    /// (Unity runs this between Update and LateUpdate).
    pub fn update_animations(&mut self) {
        let mut locals: Vec<Trs> = self.nodes.iter().map(|n| n.local).collect();
        for index in 0..self.animations.len() {
            let node = self.animations[index].node;
            if self.active_in_hierarchy(node) {
                self.animations[index].update(&mut locals);
            }
        }
        for (node, local) in self.nodes.iter_mut().zip(locals) {
            node.local = local;
        }
    }

    pub fn begin_frame(&mut self, dt: f32) {
        for animation in &mut self.animations {
            animation.begin_frame(dt);
        }
    }

    /// `GameObject.SetActive` for each listed node ref (e.g. `localObjectsToDeactivate`).
    pub fn set_active_refs(&mut self, refs: &Value, active: bool) {
        let nodes: Vec<usize> = refs
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| self.node_of_ref(r))
            .collect();
        for node in nodes {
            self.nodes[node].active_self = active;
        }
    }

    /// `GameObject.SetActive`. A node that becomes active in the hierarchy enables the
    /// `Animation`s below it, and each one with `playAutomatically` plays its default clip
    /// again (inferred Unity behaviour; the Mercenary sniper's muzzle flash relies on it:
    /// `docs/gameplay/animation.md`).
    pub fn set_active(&mut self, node: usize, active: bool) {
        let was_active = self.active_in_hierarchy(node);
        self.nodes[node].active_self = active;
        if !was_active && self.active_in_hierarchy(node) {
            let subtree = self.subtree(node);
            self.play_automatically_in(&subtree);
        }
    }

    /// Plays the default clip of every `Animation` with `playAutomatically` on an active
    /// GameObject (Unity does this when the component is enabled).
    pub fn play_automatically(&mut self) {
        let all: Vec<usize> = (0..self.nodes.len()).collect();
        self.play_automatically_in(&all);
    }

    fn play_automatically_in(&mut self, nodes: &[usize]) {
        for index in 0..self.animations.len() {
            let node = self.animations[index].node;
            if !nodes.contains(&node) || !self.active_in_hierarchy(node) {
                continue;
            }
            let animation = &mut self.animations[index];
            if animation.play_automatically
                && let Some(clip) = animation.default_clip.clone()
            {
                // Starts from the clip's beginning, even when the GameObject was deactivated
                // mid-clip (inferred).
                animation.stop(&clip);
                animation.play(&clip);
            }
        }
    }
}

/// Maps every GameObject and component ref of the `prefabs/resources.assets/*.json` prefabs to
/// that file (relative to `assets/data/`), so script fields such as `Bullet` or
/// `projectile` (refs to a prefab's GameObject or one of its components) find their prefab.
pub fn resource_prefab_index(assets: &Path) -> Result<HashMap<String, String>> {
    let directory = assets.join("data/prefabs/resources.assets");
    let mut index = HashMap::new();
    let mut entries: Vec<_> = std::fs::read_dir(&directory)
        .with_context(|| format!("reading {}", directory.display()))?
        .collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".json") {
            continue;
        }
        let path = format!("prefabs/resources.assets/{name}");
        let json = load_json(&entry.path())?;
        for node in json["nodes"].as_array().into_iter().flatten() {
            let refs = std::iter::once(&node["ref"]).chain(
                node["components"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|c| &c["ref"]),
            );
            for reference in refs.filter_map(Value::as_str) {
                index.insert(reference.to_string(), path.clone());
            }
        }
    }
    Ok(index)
}

fn find_path(nodes: &[Node], from: usize, path: &str) -> Option<usize> {
    let mut current = from;
    if path.is_empty() {
        return Some(current);
    }
    for part in path.split('/') {
        current = *nodes[current]
            .children
            .iter()
            .find(|&&c| nodes[c].name == part)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    #[test]
    fn player_prefab_binds_animations_and_skins() {
        let assets = find_assets_dir().unwrap();
        let mut clips = ClipLibrary::new(&assets).unwrap();
        let prefab = PrefabInstance::load(&assets, "german", &mut clips).unwrap();
        // Third-person body Animation with its clips bound to the Root_jnt skeleton.
        let body = prefab.find("mercenary_minecraft").unwrap();
        let animation = prefab.animations.iter().find(|a| a.node == body).unwrap();
        assert!(animation.state_index("Run").is_some());
        assert_eq!(animation.default_clip.as_deref(), Some("Weapon_Idle"));
        // Every skinned renderer resolves all its bones.
        let skinned: Vec<_> = prefab
            .renderers
            .iter()
            .filter(|r| r.skin.is_some())
            .collect();
        assert!(!skinned.is_empty());
        for renderer in skinned {
            let skin = renderer.skin.as_ref().unwrap();
            assert!(
                skin.bones.iter().all(Option::is_some),
                "{:?}",
                renderer.mesh
            );
        }
        // Weapons are made of Unity's built-in Cube/Cylinder.
        assert!(
            prefab
                .renderers
                .iter()
                .any(|r| r.mesh == MeshSource::Builtin(Primitive::Cube))
        );
    }
}
