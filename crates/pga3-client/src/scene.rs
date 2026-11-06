//! Scene loading from `assets/data/scenes/<Scene>.json`.
//!
//! Everything stays in Unity world space (left-handed, +Y up, +Z forward, metres) because
//! positions and rotations exchanged with the original servers use it.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use glam::{Mat4, Quat, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{color_of, f32_of, load_json, ref_of, vec4_of};

pub struct MeshRendererInstance {
    pub world: Mat4,
    pub mesh_asset: String,
    /// Material refs (keys of materials.json), index = submesh slot. `None` = empty slot.
    pub materials: Vec<Option<String>>,
}

pub struct DirectionalLight {
    /// Unity `-transform.forward`: direction *towards* the light (`_WorldSpaceLightPos0.xyz`).
    pub to_light: Vec3,
    /// `color * intensity` (gamma colour space, as `_LightColor0`).
    pub color: Vec3,
}

pub struct CameraInfo {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
    pub clear_skybox: bool,
    pub background: Vec4,
}

pub struct SpawnPoint {
    pub team: String,
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone, Copy)]
pub enum FogMode {
    Linear,
    Exponential,
    ExponentialSquared,
}

pub struct RenderSettings {
    pub fog: Option<FogMode>,
    pub fog_color: Vec4,
    pub fog_density: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    /// `AmbientMode`; every scene in this build uses 3 (Flat = `m_AmbientSkyColor`).
    pub ambient_mode: i64,
    pub ambient_sky: Vec4,
    pub skybox_material: Option<String>,
}

pub enum ColliderShape {
    /// Non-convex MeshCollider: triangles of an `assets/models/*.obj` mesh.
    Mesh { mesh_asset: String },
    /// BoxCollider in the node's local space.
    Box { center: Vec3, size: Vec3 },
}

pub struct ColliderInstance {
    pub world: Mat4,
    pub shape: ColliderShape,
    pub is_trigger: bool,
    /// GameObject layer (queries filter by layer mask).
    pub layer: u32,
    /// GameObject tag (`assets::tag_name` numbering), e.g. "Metal" on map geometry.
    pub tag: u32,
}

/// A `Ladder` MonoBehaviour: its trigger volume and `Ladder.Start`'s climb direction.
pub struct LadderInstance {
    /// World matrix of the Ladder GameObject (its BoxCollider trigger is in this space).
    pub world: Mat4,
    pub trigger_center: Vec3,
    pub trigger_size: Vec3,
    /// `ladderTop.position - ladderBottom.position`.
    pub climb_direction: Vec3,
}

pub struct Scene {
    pub name: String,
    pub render: RenderSettings,
    pub renderers: Vec<MeshRendererInstance>,
    /// Enabled colliders on active GameObjects (static map geometry, ladder volumes).
    pub colliders: Vec<ColliderInstance>,
    pub ladders: Vec<LadderInstance>,
    pub lights: Vec<DirectionalLight>,
    pub camera: Option<CameraInfo>,
    pub spawn_points: Vec<SpawnPoint>,
    /// Renderers whose mesh is not a published asset (e.g. Unity built-in primitives).
    pub skipped_renderers: usize,
}

struct Node<'a> {
    json: &'a Value,
    parent: Option<String>,
    local: Mat4,
    active: bool,
}

fn vec3_of(value: &Value) -> Vec3 {
    Vec3::new(
        f32_of(&value["x"]),
        f32_of(&value["y"]),
        f32_of(&value["z"]),
    )
}

fn quat_of(value: &Value) -> Quat {
    let v = vec4_of(value, ["x", "y", "z", "w"]);
    let q = Quat::from_xyzw(v.x, v.y, v.z, v.w);
    if q.length_squared() > 0.0 {
        q.normalize()
    } else {
        Quat::IDENTITY
    }
}

fn component<'a>(node: &'a Value, kind: &str) -> Option<&'a Value> {
    node["components"]
        .as_array()?
        .iter()
        .find(|c| c["type"] == kind)
        .map(|c| &c["data"])
}

fn local_matrix(node: &Value) -> Mat4 {
    let transform = component(node, "Transform").or_else(|| component(node, "RectTransform"));
    match transform {
        Some(t) => Mat4::from_scale_rotation_translation(
            vec3_of(&t["m_LocalScale"]),
            quat_of(&t["m_LocalRotation"]),
            vec3_of(&t["m_LocalPosition"]),
        ),
        None => Mat4::IDENTITY,
    }
}

fn enabled(data: &Value) -> bool {
    match &data["m_Enabled"] {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_i64() != Some(0),
        _ => true,
    }
}

pub fn load_scene(assets: &Path, name: &str) -> Result<Scene> {
    let json = load_json(&assets.join(format!("data/scenes/{name}.json")))?;
    let nodes_json = json["nodes"].as_array().context("scene has no nodes")?;

    let mut nodes: HashMap<String, Node> = HashMap::new();
    for node in nodes_json {
        let reference = node["ref"]
            .as_str()
            .context("node without ref")?
            .to_string();
        nodes.insert(
            reference,
            Node {
                json: node,
                parent: node["parent"].as_str().map(str::to_string),
                local: local_matrix(node),
                active: node["active"].as_bool().unwrap_or(true),
            },
        );
    }

    // World matrix and activeInHierarchy, resolved through the parent chain.
    let resolve = |reference: &str| -> (Mat4, bool) {
        let mut world = Mat4::IDENTITY;
        let mut active = true;
        let mut current = Some(reference.to_string());
        while let Some(key) = current {
            let Some(node) = nodes.get(&key) else {
                break;
            };
            world = node.local * world;
            active &= node.active;
            current = node.parent.clone();
        }
        (world, active)
    };

    let settings = &json["settings"]["RenderSettings"];
    let render = RenderSettings {
        fog: if settings["m_Fog"].as_bool().unwrap_or(false) {
            match settings["m_FogMode"].as_i64() {
                Some(1) => Some(FogMode::Linear),
                Some(2) => Some(FogMode::Exponential),
                Some(3) => Some(FogMode::ExponentialSquared),
                _ => None,
            }
        } else {
            None
        },
        fog_color: color_of(&settings["m_FogColor"]),
        fog_density: f32_of(&settings["m_FogDensity"]),
        fog_start: f32_of(&settings["m_LinearFogStart"]),
        fog_end: f32_of(&settings["m_LinearFogEnd"]),
        ambient_mode: settings["m_AmbientMode"].as_i64().unwrap_or(3),
        ambient_sky: color_of(&settings["m_AmbientSkyColor"]),
        skybox_material: ref_of(&settings["m_SkyboxMaterial"]).map(|(r, _)| r.to_string()),
    };

    let mut scene = Scene {
        name: name.to_string(),
        render,
        renderers: Vec::new(),
        colliders: Vec::new(),
        ladders: Vec::new(),
        lights: Vec::new(),
        camera: None,
        spawn_points: Vec::new(),
        skipped_renderers: 0,
    };

    for node in nodes_json {
        let reference = node["ref"].as_str().unwrap_or_default();
        let (world, active) = resolve(reference);
        let node_name = node["name"].as_str().unwrap_or_default();

        if node_name.starts_with("SpawnPoint") {
            let parent_name = node["parent"]
                .as_str()
                .and_then(|p| nodes.get(p))
                .and_then(|p| p.json["name"].as_str())
                .unwrap_or_default();
            let (_, rotation, position) = world.to_scale_rotation_translation();
            scene.spawn_points.push(SpawnPoint {
                team: parent_name.trim_end_matches(" SpawnPoints").to_string(),
                position,
                rotation,
            });
        }
        if !active {
            continue;
        }

        if let (Some(filter), Some(renderer)) = (
            component(node, "MeshFilter"),
            component(node, "MeshRenderer"),
        ) && enabled(renderer)
        {
            match ref_of(&filter["m_Mesh"]).and_then(|(_, asset)| asset) {
                Some(mesh_asset) => scene.renderers.push(MeshRendererInstance {
                    world,
                    mesh_asset: mesh_asset.to_string(),
                    materials: renderer["m_Materials"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|m| ref_of(m).map(|(r, _)| r.to_string()))
                        .collect(),
                }),
                None => scene.skipped_renderers += 1,
            }
        }

        for collider in node["components"].as_array().into_iter().flatten() {
            let data = &collider["data"];
            if !enabled(data) {
                continue;
            }
            let shape = match collider["type"].as_str() {
                Some("MeshCollider") => match ref_of(&data["m_Mesh"]).and_then(|(_, a)| a) {
                    Some(mesh_asset) => ColliderShape::Mesh {
                        mesh_asset: mesh_asset.to_string(),
                    },
                    None => continue,
                },
                Some("BoxCollider") => ColliderShape::Box {
                    center: vec3_of(&data["m_Center"]),
                    size: vec3_of(&data["m_Size"]),
                },
                _ => continue,
            };
            scene.colliders.push(ColliderInstance {
                world,
                shape,
                is_trigger: data["m_IsTrigger"].as_bool().unwrap_or(false),
                layer: node["layer"].as_u64().unwrap_or(0) as u32,
                tag: node["tag"].as_u64().unwrap_or(0) as u32,
            });
        }

        let ladder = node["components"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|c| {
                c["type"] == "MonoBehaviour" && c["script"] == "Ladder" && c["enabled"] != false
            });
        let trigger = component(node, "BoxCollider");
        if let (Some(ladder), Some(trigger)) = (ladder, trigger) {
            let position_of = |field: &str| {
                let reference = ladder["fields"][field]["ref"].as_str().unwrap_or_default();
                resolve(reference).0.transform_point3(Vec3::ZERO)
            };
            scene.ladders.push(LadderInstance {
                world,
                trigger_center: vec3_of(&trigger["m_Center"]),
                trigger_size: vec3_of(&trigger["m_Size"]),
                climb_direction: position_of("ladderTop") - position_of("ladderBottom"),
            });
        }

        if let Some(light) = component(node, "Light") {
            // m_Type 1 = Directional.
            if enabled(light) && light["m_Type"].as_i64() == Some(1) {
                let color = color_of(&light["m_Color"]).truncate() * f32_of(&light["m_Intensity"]);
                let forward = world.transform_vector3(Vec3::Z).normalize_or_zero();
                scene.lights.push(DirectionalLight {
                    to_light: -forward,
                    color,
                });
            }
        }

        if let Some(camera) = component(node, "Camera")
            && enabled(camera)
            && scene.camera.is_none()
        {
            let (_, rotation, position) = world.to_scale_rotation_translation();
            scene.camera = Some(CameraInfo {
                position,
                rotation,
                fov_y_degrees: f32_of(&camera["field of view"]),
                near: f32_of(&camera["near clip plane"]),
                far: f32_of(&camera["far clip plane"]),
                clear_skybox: camera["m_ClearFlags"].as_i64() == Some(1),
                background: color_of(&camera["m_BackGroundColor"]),
            });
        }
    }
    Ok(scene)
}
