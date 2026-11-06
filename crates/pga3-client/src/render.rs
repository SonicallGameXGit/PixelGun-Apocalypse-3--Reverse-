//! Scene renderer: ports of the original game's shaders (from the compiled GLSL ES programs in
//! `assets/shaders/`) on OpenGL 3.3 core.
//!
//! Colour pipeline matches the original build: gamma colour space (PlayerSettings
//! `m_ActiveColorSpace = 0`), no sRGB textures/framebuffer, quality level "Good" (1 pixel light,
//! no shadows, no MSAA).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use glow::HasContext;

use crate::assets::{Material, ShaderKind, TextureInfo, load_materials, load_texture_infos};
use crate::gfx::{GpuMesh, compile_program, upload_mesh, upload_texture};
use crate::obj::{MeshData, parse_obj};
use crate::prefab::{MeshSource, PrefabInstance};
use crate::primitives::Primitive;
use crate::scene::{FogMode, Scene};
use crate::skin::{SkinData, load_skin};
use crate::weapons::GuiQuad;

/// Shared part of the lit vertex shaders: `lit_outputs` does the projection, UVs, ambient and
/// fog of the compiled `Mobile/Diffuse` / `Diffuse` vertex programs.
const LIT_VS_COMMON: &str = r#"#version 330 core
in vec3 a_position;
in vec3 a_normal;
in vec2 a_uv;
uniform mat4 u_view_proj;
uniform vec4 u_main_st;
uniform vec3 u_ambient_linear;
uniform int u_fog_mode;
uniform vec4 u_fog_params;
out vec2 v_uv;
out vec3 v_normal;
out vec3 v_ambient;
out float v_fog;
void lit_outputs(vec4 world, vec3 world_normal) {
    gl_Position = u_view_proj * world;
    v_uv = a_uv * u_main_st.xy + u_main_st.zw;
    v_normal = normalize(world_normal);
    // ShadeSH9 (flat ambient = L0 term only), then Unity's LinearToGammaSpace approximation.
    vec3 sh = max(u_ambient_linear, vec3(0.0));
    v_ambient = max(exp2(log2(sh) * 0.416666657) * 1.05499995 - 0.0549999997, vec3(0.0));
    // UNITY_TRANSFER_FOG with clip-space z, as in the compiled GLES programs.
    float z = gl_Position.z;
    if (u_fog_mode == 1) {
        v_fog = z * u_fog_params.z + u_fog_params.w;
    } else if (u_fog_mode == 2) {
        v_fog = exp2(-z * u_fog_params.y);
    } else if (u_fog_mode == 3) {
        float t = z * u_fog_params.x;
        v_fog = exp2(-t * t);
    } else {
        v_fog = 1.0;
    }
}
"#;

const LIT_VS_MAIN: &str = r#"
uniform mat4 u_object_to_world;
uniform mat3 u_normal_matrix;
void main() {
    lit_outputs(u_object_to_world * vec4(a_position, 1.0), u_normal_matrix * a_normal);
}
"#;

/// GPU skinning (`SkinnedMeshRenderer`): `u_bones[i] = bone[i].localToWorldMatrix *
/// bindpose[i]`; weights are pre-limited to the quality level's bone count (`skin.rs`).
const SKINNED_VS_MAIN: &str = r#"
in vec4 a_bone_indices;
in vec4 a_bone_weights;
uniform mat4 u_bones[MAX_BONES];
void main() {
    mat4 skin = u_bones[int(a_bone_indices.x)] * a_bone_weights.x
        + u_bones[int(a_bone_indices.y)] * a_bone_weights.y
        + u_bones[int(a_bone_indices.z)] * a_bone_weights.z
        + u_bones[int(a_bone_indices.w)] * a_bone_weights.w;
    lit_outputs(skin * vec4(a_position, 1.0), mat3(skin) * a_normal);
}
"#;

/// Upper bound of `m_Bones` per skinned mesh in this build is 60.
const MAX_BONES: usize = 64;

const LIT_FS: &str = r#"#version 330 core
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_ambient;
in float v_fog;
uniform sampler2D u_main_tex;
uniform vec3 u_color;
uniform vec3 u_to_light;
uniform vec3 u_light_color;
uniform vec3 u_fog_color;
out vec4 frag_color;
void main() {
    vec3 albedo = texture(u_main_tex, v_uv).rgb * u_color;
    float ndl = max(dot(v_normal, u_to_light), 0.0);
    vec3 c = albedo * u_light_color * ndl + albedo * v_ambient;
    frag_color = vec4(mix(u_fog_color, c, clamp(v_fog, 0.0, 1.0)), 1.0);
}
"#;

const UNLIT_VS: &str = r#"#version 330 core
in vec3 a_position;
in vec3 a_normal;
in vec2 a_uv;
uniform mat4 u_object_to_world;
uniform mat4 u_view_proj;
uniform vec4 u_main_st;
out vec2 v_uv;
void main() {
    gl_Position = u_view_proj * (u_object_to_world * vec4(a_position, 1.0));
    v_uv = a_uv * u_main_st.xy + u_main_st.zw;
}
"#;

const DECAL_FS: &str = r#"#version 330 core
in vec2 v_uv;
uniform sampler2D u_main_tex;
uniform vec4 u_tint;
out vec4 frag_color;
void main() {
    vec4 c = texture(u_main_tex, v_uv) * u_tint;
    if (c.a <= 0.0) {
        discard;
    }
    frag_color = c;
}
"#;

/// `Mobile/Particles/Additive` (Unity's built-in fixed-function shader: `Blend SrcAlpha One`,
/// `Cull Off`, `ZWrite Off`, `Fog { Color (0,0,0,0) }`, `combine texture * primary`). Meshes
/// without vertex colours give `primary` = white.
const ADDITIVE_VS: &str = r#"#version 330 core
in vec3 a_position;
in vec3 a_normal;
in vec2 a_uv;
uniform mat4 u_object_to_world;
uniform mat4 u_view_proj;
uniform vec4 u_main_st;
uniform int u_fog_mode;
uniform vec4 u_fog_params;
out vec2 v_uv;
out float v_fog;
void main() {
    gl_Position = u_view_proj * (u_object_to_world * vec4(a_position, 1.0));
    v_uv = a_uv * u_main_st.xy + u_main_st.zw;
    float z = gl_Position.z;
    if (u_fog_mode == 1) {
        v_fog = z * u_fog_params.z + u_fog_params.w;
    } else if (u_fog_mode == 2) {
        v_fog = exp2(-z * u_fog_params.y);
    } else if (u_fog_mode == 3) {
        float t = z * u_fog_params.x;
        v_fog = exp2(-t * t);
    } else {
        v_fog = 1.0;
    }
}
"#;

const ADDITIVE_FS: &str = r#"#version 330 core
in vec2 v_uv;
in float v_fog;
uniform sampler2D u_main_tex;
out vec4 frag_color;
void main() {
    vec4 c = texture(u_main_tex, v_uv);
    // Fog towards the shader's fog colour, black.
    frag_color = vec4(c.rgb * clamp(v_fog, 0.0, 1.0), c.a);
}
"#;

/// `OnGUI` textures (`GUI.Box` with a background-only style, `GUI.DrawTexture`): GUI pixels,
/// origin at the top left, tinted by `GUI.color`, alpha blended.
const GUI_VS: &str = r#"#version 330 core
in vec2 a_position;
uniform vec4 u_rect;
uniform vec2 u_screen;
out vec2 v_uv;
void main() {
    vec2 pixel = u_rect.xy + a_position * u_rect.zw;
    gl_Position = vec4(pixel.x / u_screen.x * 2.0 - 1.0, 1.0 - pixel.y / u_screen.y * 2.0, 0.0, 1.0);
    // Texture row 0 (uploaded flipped) is the bottom; GUI y grows downwards.
    v_uv = vec2(a_position.x, 1.0 - a_position.y);
}
"#;

const GUI_FS: &str = r#"#version 330 core
in vec2 v_uv;
uniform sampler2D u_main_tex;
uniform vec4 u_tint;
out vec4 frag_color;
void main() {
    frag_color = texture(u_main_tex, v_uv) * u_tint;
}
"#;

const SKY_VS: &str = r#"#version 330 core
in vec3 a_position;
in vec2 a_uv;
uniform mat4 u_view_proj;
out vec2 v_uv;
void main() {
    // Drawn first without depth test/write, so the cube size only has to sit between the clip
    // planes (near 0.3); scene geometry then overwrites it.
    gl_Position = u_view_proj * vec4(a_position * 10.0, 1.0);
    v_uv = a_uv;
}
"#;

const SKY_FS: &str = r#"#version 330 core
in vec2 v_uv;
uniform sampler2D u_main_tex;
out vec4 frag_color;
void main() {
    frag_color = texture(u_main_tex, v_uv);
}
"#;

struct Program {
    handle: glow::Program,
    uniforms: HashMap<&'static str, glow::UniformLocation>,
}

impl Program {
    fn new(
        gl: &glow::Context,
        vs: &str,
        fs: &str,
        attributes: &[&str],
        names: &[&'static str],
    ) -> Result<Program> {
        let handle = compile_program(gl, vs, fs, attributes)?;
        let mut uniforms = HashMap::new();
        for &name in names {
            if let Some(location) = unsafe { gl.get_uniform_location(handle, name) } {
                uniforms.insert(name, location);
            }
        }
        Ok(Program { handle, uniforms })
    }

    fn u(&self, name: &str) -> Option<&glow::UniformLocation> {
        self.uniforms.get(name)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pass {
    Lit,
    Decal,
}

struct Draw {
    pass: Pass,
    vao: glow::VertexArray,
    offset: i32,
    count: i32,
    world: Mat4,
    normal: Mat3,
    mirrored: bool,
    texture: glow::Texture,
    main_st: Vec4,
    color: Vec4,
}

struct Skybox {
    mesh: GpuMesh,
    faces: [glow::Texture; 6],
}

pub struct View {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
}

impl View {
    /// Unity camera conventions on GL: `worldToCameraMatrix` flips Z (camera looks down -Z in GL
    /// view space), projection is the standard GL perspective with -1..1 depth.
    pub fn matrices(&self, aspect: f32) -> (Mat4, Mat4) {
        let camera_to_world = Mat4::from_rotation_translation(self.rotation, self.position);
        let view = Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0)) * camera_to_world.inverse();
        (
            view,
            gl_perspective(self.fov_y_degrees, aspect, self.near, self.far),
        )
    }
}

/// Unity's `Matrix4x4.Perspective` (OpenGL convention, depth -1..1), as used on WebGL.
fn gl_perspective(fov_y_degrees: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y_degrees.to_radians() * 0.5).tan();
    Mat4::from_cols(
        Vec4::new(f / aspect, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f, 0.0, 0.0),
        Vec4::new(0.0, 0.0, (far + near) / (near - far), -1.0),
        Vec4::new(0.0, 0.0, 2.0 * far * near / (near - far), 0.0),
    )
}

pub struct SceneRenderer {
    lit: Program,
    skinned: Program,
    decal: Program,
    sky: Program,
    additive: Program,
    gui: Program,
    /// Unit quad (0..1) for `gui`.
    gui_quad: GpuMesh,
    resources: Resources,
    materials: HashMap<String, Material>,
    /// `QualitySettings.blendWeights` of the active level.
    skin_bones: usize,
    draws: Vec<Draw>,
    skybox: Option<Skybox>,
    to_light: Vec3,
    light_color: Vec3,
    ambient_linear: Vec3,
    fog_mode: i32,
    fog_params: Vec4,
    fog_color: Vec3,
    pub clear_color: Vec4,
    pub stats: Stats,
}

#[derive(Default, Debug)]
pub struct Stats {
    pub meshes: usize,
    pub textures: usize,
    pub draws: usize,
    pub unsupported_shaders: HashMap<String, usize>,
    pub missing_textures: usize,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// GPU resources shared by the scene and every prepared prefab.
struct Resources {
    assets: PathBuf,
    texture_infos: HashMap<String, TextureInfo>,
    textures: HashMap<String, glow::Texture>,
    meshes: HashMap<String, GpuMesh>,
    builtin_meshes: HashMap<Primitive, GpuMesh>,
    /// Skinned meshes (position, normal, uv, bone indices, bone weights) by asset path.
    skinned_meshes: HashMap<String, (GpuMesh, Vec<Mat4>)>,
    white: glow::Texture,
    missing_textures: usize,
}

fn interleave(data: &MeshData, skin: Option<&SkinData>) -> Vec<f32> {
    let stride = if skin.is_some() { 16 } else { 8 };
    let mut vertices = Vec::with_capacity(data.positions.len() * stride);
    for i in 0..data.positions.len() {
        vertices.extend_from_slice(&data.positions[i]);
        vertices.extend_from_slice(&data.normals[i]);
        vertices.extend_from_slice(&data.uvs[i]);
        if let Some(skin) = skin {
            vertices.extend(skin.bone_indices[i].iter().map(|&b| b as f32));
            vertices.extend_from_slice(&skin.bone_weights[i]);
        }
    }
    vertices
}

impl Resources {
    fn texture(&mut self, gl: &glow::Context, path: Option<&str>) -> glow::Texture {
        let Some(path) = path else {
            return self.white;
        };
        if let Some(&texture) = self.textures.get(path) {
            return texture;
        }
        let info = self.texture_infos.get(path).copied().unwrap_or_default();
        let texture = match image::open(self.assets.join(path)) {
            Ok(img) => {
                // PNGs are top-down; GL (like Unity) samples row 0 at v = 0, the bottom.
                let rgba = image::imageops::flip_vertical(&img.to_rgba8());
                upload_texture(gl, rgba.width(), rgba.height(), rgba.as_raw(), &info)
                    .unwrap_or(self.white)
            }
            Err(error) => {
                eprintln!("texture {path}: {error}");
                self.missing_textures += 1;
                self.white
            }
        };
        self.textures.insert(path.to_string(), texture);
        texture
    }

    fn read_obj(&self, path: &str) -> Result<MeshData> {
        let text = std::fs::read_to_string(self.assets.join(path))
            .with_context(|| format!("reading {path}"))?;
        parse_obj(&text).with_context(|| format!("parsing {path}"))
    }

    fn mesh(&mut self, gl: &glow::Context, path: &str) -> Result<&GpuMesh> {
        if !self.meshes.contains_key(path) {
            let data = self.read_obj(path)?;
            let mesh = upload_mesh(gl, &interleave(&data, None), &[3, 3, 2], &data.submeshes)?;
            self.meshes.insert(path.to_string(), mesh);
        }
        Ok(&self.meshes[path])
    }

    fn builtin(&mut self, gl: &glow::Context, primitive: Primitive) -> Result<&GpuMesh> {
        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.builtin_meshes.entry(primitive)
        {
            let data = primitive.mesh();
            entry.insert(upload_mesh(
                gl,
                &interleave(&data, None),
                &[3, 3, 2],
                &data.submeshes,
            )?);
        }
        Ok(&self.builtin_meshes[&primitive])
    }

    /// Skinned mesh and its bind poses.
    fn skinned(
        &mut self,
        gl: &glow::Context,
        path: &str,
        bones: usize,
    ) -> Result<&(GpuMesh, Vec<Mat4>)> {
        if !self.skinned_meshes.contains_key(path) {
            let data = self.read_obj(path)?;
            let skin = load_skin(&self.assets, path, bones)?;
            if skin.bone_indices.len() != data.positions.len() {
                anyhow::bail!(
                    "{path}: skin has {} vertices, mesh {}",
                    skin.bone_indices.len(),
                    data.positions.len()
                );
            }
            let mesh = upload_mesh(
                gl,
                &interleave(&data, Some(&skin)),
                &[3, 3, 2, 4, 4],
                &data.submeshes,
            )?;
            self.skinned_meshes
                .insert(path.to_string(), (mesh, skin.bind_poses));
        }
        Ok(&self.skinned_meshes[path])
    }
}

/// One draw of a prepared prefab: a (renderer, material slot) pair.
struct ModelDraw {
    renderer: usize,
    vao: glow::VertexArray,
    offset: i32,
    count: i32,
    texture: glow::Texture,
    main_st: Vec4,
    color: Vec4,
    /// Bind poses for skinned renderers.
    bind_poses: Option<Vec<Mat4>>,
    /// `Mobile/Particles/Additive` (drawn after the opaque models).
    additive: bool,
}

/// GPU-side data of a `PrefabInstance`, from `SceneRenderer::prepare_prefab`.
pub struct PrefabModels {
    draws: Vec<ModelDraw>,
}

fn skybox_mesh(gl: &glow::Context) -> Result<GpuMesh> {
    // Unity 6-sided skybox faces: Front +Z, Back -Z, Left +X, Right -X, Up +Y, Down -Y.
    // Each face is seen upright from inside with "right" = increasing u.
    let faces: [fn(f32, f32) -> [f32; 3]; 6] = [
        |u, v| [u, v, 1.0],
        |u, v| [-u, v, -1.0],
        |u, v| [1.0, v, -u],
        |u, v| [-1.0, v, u],
        |u, v| [u, 1.0, -v],
        |u, v| [u, -1.0, v],
    ];
    let mut vertices = Vec::new();
    let mut submeshes = Vec::new();
    for (face, position) in faces.iter().enumerate() {
        let base = (face * 4) as u32;
        for (u, v) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            vertices.extend_from_slice(&position(u, v));
            vertices.extend_from_slice(&[(u + 1.0) * 0.5, (v + 1.0) * 0.5]);
        }
        submeshes.push(vec![base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    upload_mesh(gl, &vertices, &[3, 2], &submeshes)
}

impl SceneRenderer {
    pub fn new(gl: &glow::Context, assets: &Path, scene: &Scene) -> Result<SceneRenderer> {
        let attributes = ["a_position", "a_normal", "a_uv"];
        let lit_uniforms = [
            "u_object_to_world",
            "u_normal_matrix",
            "u_view_proj",
            "u_main_st",
            "u_ambient_linear",
            "u_fog_mode",
            "u_fog_params",
            "u_main_tex",
            "u_color",
            "u_to_light",
            "u_light_color",
            "u_fog_color",
            "u_bones",
        ];
        let lit = Program::new(
            gl,
            &format!("{LIT_VS_COMMON}{LIT_VS_MAIN}"),
            LIT_FS,
            &attributes,
            &lit_uniforms,
        )?;
        let skinned_vs = format!(
            "{LIT_VS_COMMON}{}",
            SKINNED_VS_MAIN.replace("MAX_BONES", &MAX_BONES.to_string())
        );
        let skinned = Program::new(
            gl,
            &skinned_vs,
            LIT_FS,
            &[
                "a_position",
                "a_normal",
                "a_uv",
                "a_bone_indices",
                "a_bone_weights",
            ],
            &lit_uniforms,
        )?;
        let decal = Program::new(
            gl,
            UNLIT_VS,
            DECAL_FS,
            &attributes,
            &[
                "u_object_to_world",
                "u_view_proj",
                "u_main_st",
                "u_main_tex",
                "u_tint",
            ],
        )?;
        let sky = Program::new(
            gl,
            SKY_VS,
            SKY_FS,
            &["a_position", "a_uv"],
            &["u_view_proj", "u_main_tex"],
        )?;

        let additive = Program::new(
            gl,
            ADDITIVE_VS,
            ADDITIVE_FS,
            &attributes,
            &[
                "u_object_to_world",
                "u_view_proj",
                "u_main_st",
                "u_main_tex",
                "u_fog_mode",
                "u_fog_params",
            ],
        )?;
        let gui = Program::new(
            gl,
            GUI_VS,
            GUI_FS,
            &["a_position"],
            &["u_rect", "u_screen", "u_main_tex", "u_tint"],
        )?;
        let gui_quad = upload_mesh(
            gl,
            &[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
            &[2],
            &[vec![0, 1, 2, 0, 2, 3]],
        )?;

        let materials = load_materials(assets)?;
        let white = upload_texture(gl, 1, 1, &[255, 255, 255, 255], &TextureInfo::default())?;
        let mut loader = Resources {
            assets: assets.to_path_buf(),
            texture_infos: load_texture_infos(assets)?,
            textures: HashMap::new(),
            meshes: HashMap::new(),
            builtin_meshes: HashMap::new(),
            skinned_meshes: HashMap::new(),
            white,
            missing_textures: 0,
        };
        let mut stats = Stats::default();
        let mut draws = Vec::new();

        for renderer in &scene.renderers {
            let (vao, submeshes) = {
                let mesh = loader.mesh(gl, &renderer.mesh_asset)?;
                (mesh.vao, mesh.submeshes.clone())
            };
            if submeshes.is_empty() {
                continue;
            }
            // Unity: material slot i draws submesh i; extra slots redraw the last submesh.
            for (slot, material_ref) in renderer.materials.iter().enumerate() {
                let Some(material) = material_ref.as_ref().and_then(|r| materials.get(r)) else {
                    continue;
                };
                let (offset, count) = submeshes[slot.min(submeshes.len() - 1)];
                let (pass, color) = match material.shader {
                    ShaderKind::MobileDiffuse => (Pass::Lit, Vec4::ONE),
                    ShaderKind::Diffuse => (Pass::Lit, material.color("_Color")),
                    ShaderKind::BlendedDecal => (Pass::Decal, material.color("_Color")),
                    ShaderKind::Skybox
                    | ShaderKind::ParticlesAdditive
                    | ShaderKind::Unsupported => {
                        *stats
                            .unsupported_shaders
                            .entry(material.shader_name.clone())
                            .or_default() += 1;
                        (Pass::Lit, material.color("_Color"))
                    }
                };
                let (texture_path, main_st) = material.texture("_MainTex");
                let texture = loader.texture(gl, texture_path);
                draws.push(Draw {
                    pass,
                    vao,
                    offset,
                    count,
                    world: renderer.world,
                    normal: Mat3::from_mat4(renderer.world).inverse().transpose(),
                    mirrored: renderer.world.determinant() < 0.0,
                    texture,
                    main_st,
                    color,
                });
            }
        }

        // Camera clear flags 1 = Skybox; otherwise only the background colour is cleared.
        let clears_skybox = scene.camera.as_ref().is_none_or(|c| c.clear_skybox);
        let skybox_material = scene
            .render
            .skybox_material
            .as_ref()
            .and_then(|r| materials.get(r));
        let skybox = match skybox_material {
            Some(material) if clears_skybox => Some(Self::load_skybox(gl, &mut loader, material)?),
            _ => None,
        };
        if scene.render.ambient_mode != 3 {
            eprintln!(
                "ambient mode {} not implemented, using flat ambient",
                scene.render.ambient_mode
            );
        }

        let main_light = scene
            .lights
            .iter()
            .max_by(|a, b| a.color.dot(Vec3::ONE).total_cmp(&b.color.dot(Vec3::ONE)));
        let render = &scene.render;
        let ambient = render.ambient_sky.truncate();
        let ambient_linear = Vec3::new(
            srgb_to_linear(ambient.x),
            srgb_to_linear(ambient.y),
            srgb_to_linear(ambient.z),
        );
        let (fog_mode, fog_params) = match render.fog {
            None => (0, Vec4::ZERO),
            Some(mode) => {
                let range = render.fog_end - render.fog_start;
                let params = Vec4::new(
                    render.fog_density * 1.201_122_4,
                    render.fog_density * std::f32::consts::LOG2_E,
                    if range != 0.0 { -1.0 / range } else { 0.0 },
                    if range != 0.0 {
                        render.fog_end / range
                    } else {
                        1.0
                    },
                );
                let mode = match mode {
                    FogMode::Linear => 1,
                    FogMode::Exponential => 2,
                    FogMode::ExponentialSquared => 3,
                };
                (mode, params)
            }
        };

        stats.meshes = loader.meshes.len();
        stats.textures = loader.textures.len();
        stats.draws = draws.len();
        stats.missing_textures = loader.missing_textures;
        // Opaque first, then decals (Transparent queue).
        draws.sort_by_key(|d| d.pass == Pass::Decal);

        Ok(SceneRenderer {
            lit,
            skinned,
            decal,
            sky,
            additive,
            gui,
            gui_quad,
            resources: loader,
            materials,
            skin_bones: crate::skin::quality_bone_count(assets)?,
            draws,
            skybox,
            to_light: main_light.map(|l| l.to_light).unwrap_or(Vec3::Y),
            light_color: main_light.map(|l| l.color).unwrap_or(Vec3::ZERO),
            ambient_linear,
            fog_mode,
            fog_params,
            fog_color: render.fog_color.truncate(),
            clear_color: scene
                .camera
                .as_ref()
                .map(|c| c.background)
                .unwrap_or(Vec4::new(0.0, 0.0, 0.0, 1.0)),
            stats,
        })
    }

    fn load_skybox(
        gl: &glow::Context,
        loader: &mut Resources,
        material: &Material,
    ) -> Result<Skybox> {
        let names = [
            "_FrontTex",
            "_BackTex",
            "_LeftTex",
            "_RightTex",
            "_UpTex",
            "_DownTex",
        ];
        let faces = names.map(|name| loader.texture(gl, material.texture(name).0));
        Ok(Skybox {
            mesh: skybox_mesh(gl)?,
            faces,
        })
    }

    pub fn draw(&self, gl: &glow::Context, view: &View, width: u32, height: u32) {
        let aspect = width as f32 / height.max(1) as f32;
        let (view_matrix, projection) = view.matrices(aspect);
        let view_proj = projection * view_matrix;
        let c = self.clear_color;
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear_color(c.x, c.y, c.z, c.w);
            gl.depth_mask(true);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.disable(glow::BLEND);
            gl.active_texture(glow::TEXTURE0);

            // Skybox (Mobile/Skybox: Cull Off, ZWrite Off) behind everything.
            if let Some(skybox) = &self.skybox {
                let rotation_only = Mat4::from_mat3(Mat3::from_mat4(view_matrix));
                let p = &self.sky;
                gl.use_program(Some(p.handle));
                gl.uniform_matrix_4_f32_slice(
                    p.u("u_view_proj"),
                    false,
                    &(projection * rotation_only).to_cols_array(),
                );
                gl.uniform_1_i32(p.u("u_main_tex"), 0);
                gl.disable(glow::CULL_FACE);
                gl.disable(glow::DEPTH_TEST);
                gl.bind_vertex_array(Some(skybox.mesh.vao));
                for (face, &(offset, count)) in skybox.mesh.submeshes.iter().enumerate() {
                    gl.bind_texture(glow::TEXTURE_2D, Some(skybox.faces[face]));
                    gl.draw_elements(glow::TRIANGLES, count, glow::UNSIGNED_INT, offset);
                }
                gl.enable(glow::CULL_FACE);
                gl.enable(glow::DEPTH_TEST);
            }

            // Opaque, Mobile/Diffuse / Diffuse.
            let p = &self.lit;
            gl.use_program(Some(p.handle));
            gl.uniform_matrix_4_f32_slice(p.u("u_view_proj"), false, &view_proj.to_cols_array());
            gl.uniform_3_f32(
                p.u("u_ambient_linear"),
                self.ambient_linear.x,
                self.ambient_linear.y,
                self.ambient_linear.z,
            );
            gl.uniform_1_i32(p.u("u_fog_mode"), self.fog_mode);
            let f = self.fog_params;
            gl.uniform_4_f32(p.u("u_fog_params"), f.x, f.y, f.z, f.w);
            gl.uniform_3_f32(
                p.u("u_fog_color"),
                self.fog_color.x,
                self.fog_color.y,
                self.fog_color.z,
            );
            gl.uniform_3_f32(
                p.u("u_to_light"),
                self.to_light.x,
                self.to_light.y,
                self.to_light.z,
            );
            gl.uniform_3_f32(
                p.u("u_light_color"),
                self.light_color.x,
                self.light_color.y,
                self.light_color.z,
            );
            gl.uniform_1_i32(p.u("u_main_tex"), 0);
            for d in self.draws.iter().filter(|d| d.pass == Pass::Lit) {
                gl.front_face(if d.mirrored { glow::CCW } else { glow::CW });
                gl.uniform_matrix_4_f32_slice(
                    p.u("u_object_to_world"),
                    false,
                    &d.world.to_cols_array(),
                );
                gl.uniform_matrix_3_f32_slice(
                    p.u("u_normal_matrix"),
                    false,
                    &d.normal.to_cols_array(),
                );
                gl.uniform_4_f32(
                    p.u("u_main_st"),
                    d.main_st.x,
                    d.main_st.y,
                    d.main_st.z,
                    d.main_st.w,
                );
                gl.uniform_3_f32(p.u("u_color"), d.color.x, d.color.y, d.color.z);
                gl.bind_texture(glow::TEXTURE_2D, Some(d.texture));
                gl.bind_vertex_array(Some(d.vao));
                gl.draw_elements(glow::TRIANGLES, d.count, glow::UNSIGNED_INT, d.offset);
            }

            // BlendedDecal: ZWrite Off, Offset -1, -1, alpha test.
            let p = &self.decal;
            gl.use_program(Some(p.handle));
            gl.uniform_matrix_4_f32_slice(p.u("u_view_proj"), false, &view_proj.to_cols_array());
            gl.uniform_1_i32(p.u("u_main_tex"), 0);
            gl.depth_mask(false);
            gl.enable(glow::POLYGON_OFFSET_FILL);
            gl.polygon_offset(-1.0, -1.0);
            for d in self.draws.iter().filter(|d| d.pass == Pass::Decal) {
                gl.front_face(if d.mirrored { glow::CCW } else { glow::CW });
                gl.uniform_matrix_4_f32_slice(
                    p.u("u_object_to_world"),
                    false,
                    &d.world.to_cols_array(),
                );
                gl.uniform_4_f32(
                    p.u("u_main_st"),
                    d.main_st.x,
                    d.main_st.y,
                    d.main_st.z,
                    d.main_st.w,
                );
                gl.uniform_4_f32(p.u("u_tint"), d.color.x, d.color.y, d.color.z, d.color.w);
                gl.bind_texture(glow::TEXTURE_2D, Some(d.texture));
                gl.bind_vertex_array(Some(d.vao));
                gl.draw_elements(glow::TRIANGLES, d.count, glow::UNSIGNED_INT, d.offset);
            }
            gl.disable(glow::POLYGON_OFFSET_FILL);
            gl.depth_mask(true);
            gl.bind_vertex_array(None);
        }
    }
}

impl SceneRenderer {
    /// Uploads the meshes/textures of a prefab instance's renderers.
    pub fn prepare_prefab(
        &mut self,
        gl: &glow::Context,
        prefab: &PrefabInstance,
    ) -> Result<PrefabModels> {
        let mut draws = Vec::new();
        for (index, renderer) in prefab.renderers.iter().enumerate() {
            let (vao, submeshes, bind_poses) = match (&renderer.mesh, &renderer.skin) {
                (MeshSource::Asset(path), Some(_)) => {
                    let (mesh, poses) = self.resources.skinned(gl, path, self.skin_bones)?;
                    (mesh.vao, mesh.submeshes.clone(), Some(poses.clone()))
                }
                (MeshSource::Asset(path), None) => {
                    let mesh = self.resources.mesh(gl, path)?;
                    (mesh.vao, mesh.submeshes.clone(), None)
                }
                (MeshSource::Builtin(primitive), _) => {
                    let mesh = self.resources.builtin(gl, *primitive)?;
                    (mesh.vao, mesh.submeshes.clone(), None)
                }
            };
            if submeshes.is_empty() {
                continue;
            }
            for (slot, material_ref) in renderer.materials.iter().enumerate() {
                let Some(material) = material_ref.as_ref().and_then(|r| self.materials.get(r))
                else {
                    continue;
                };
                let (color, additive) = match material.shader {
                    ShaderKind::MobileDiffuse => (Vec4::ONE, false),
                    ShaderKind::Diffuse => (material.color("_Color"), false),
                    ShaderKind::ParticlesAdditive => (Vec4::ONE, true),
                    // Other prefab shaders are transparent particle effects (`Legacy
                    // Shaders/Particles/Alpha Blended`); drawing them as opaque Diffuse would be
                    // wrong, so they are skipped and reported until those shaders are ported.
                    _ => {
                        *self
                            .stats
                            .unsupported_shaders
                            .entry(material.shader_name.clone())
                            .or_default() += 1;
                        continue;
                    }
                };
                let (offset, count) = submeshes[slot.min(submeshes.len() - 1)];
                let (texture_path, main_st) = material.texture("_MainTex");
                let texture = self.resources.texture(gl, texture_path);
                draws.push(ModelDraw {
                    renderer: index,
                    vao,
                    offset,
                    count,
                    texture,
                    main_st,
                    color,
                    bind_poses: bind_poses.clone(),
                    additive,
                });
            }
        }
        Ok(PrefabModels { draws })
    }

    fn set_lighting(&self, gl: &glow::Context, p: &Program, view_proj: &Mat4) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(p.u("u_view_proj"), false, &view_proj.to_cols_array());
            let a = self.ambient_linear;
            gl.uniform_3_f32(p.u("u_ambient_linear"), a.x, a.y, a.z);
            gl.uniform_1_i32(p.u("u_fog_mode"), self.fog_mode);
            let f = self.fog_params;
            gl.uniform_4_f32(p.u("u_fog_params"), f.x, f.y, f.z, f.w);
            let c = self.fog_color;
            gl.uniform_3_f32(p.u("u_fog_color"), c.x, c.y, c.z);
            let l = self.to_light;
            gl.uniform_3_f32(p.u("u_to_light"), l.x, l.y, l.z);
            let c = self.light_color;
            gl.uniform_3_f32(p.u("u_light_color"), c.x, c.y, c.z);
            gl.uniform_1_i32(p.u("u_main_tex"), 0);
        }
    }

    /// Draws the renderers of `prefab` whose GameObject is active and whose layer is in the
    /// camera's culling mask. `clear_depth` = camera clear flags "Depth only" (3), as the
    /// player's `Weapon Camera` uses. `root` places the prefab's root elsewhere (instances of a
    /// shared template: its root world matrix is replaced, the rest of the hierarchy follows).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_models(
        &self,
        gl: &glow::Context,
        view: &View,
        width: u32,
        height: u32,
        prefab: &PrefabInstance,
        models: &PrefabModels,
        culling_mask: u32,
        clear_depth: bool,
        root: Option<Mat4>,
    ) {
        let aspect = width as f32 / height.max(1) as f32;
        let (view_matrix, projection) = view.matrices(aspect);
        let view_proj = projection * view_matrix;
        let visible = |renderer: usize| {
            let r = &prefab.renderers[renderer];
            let layer = prefab.nodes[r.node].layer;
            r.enabled && culling_mask & (1 << layer) != 0 && prefab.active_in_hierarchy(r.node)
        };
        let relocate = root.map(|m| m * prefab.world[0].inverse());
        let world_of = |node: usize| match relocate {
            Some(m) => m * prefab.world[node],
            None => prefab.world[node],
        };
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
            if clear_depth {
                gl.depth_mask(true);
                gl.clear(glow::DEPTH_BUFFER_BIT);
            }
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.disable(glow::BLEND);
            gl.active_texture(glow::TEXTURE0);

            let p = &self.lit;
            gl.use_program(Some(p.handle));
            self.set_lighting(gl, p, &view_proj);
            for d in models
                .draws
                .iter()
                .filter(|d| d.bind_poses.is_none() && !d.additive)
            {
                if !visible(d.renderer) {
                    continue;
                }
                let world = world_of(prefab.renderers[d.renderer].node);
                let normal = Mat3::from_mat4(world).inverse().transpose();
                gl.front_face(if world.determinant() < 0.0 {
                    glow::CCW
                } else {
                    glow::CW
                });
                gl.uniform_matrix_4_f32_slice(
                    p.u("u_object_to_world"),
                    false,
                    &world.to_cols_array(),
                );
                gl.uniform_matrix_3_f32_slice(
                    p.u("u_normal_matrix"),
                    false,
                    &normal.to_cols_array(),
                );
                self.draw_model(gl, p, d);
            }

            let p = &self.skinned;
            gl.use_program(Some(p.handle));
            self.set_lighting(gl, p, &view_proj);
            gl.front_face(glow::CW);
            let mut palette = vec![0.0f32; MAX_BONES * 16];
            for d in models.draws.iter() {
                let Some(bind_poses) = &d.bind_poses else {
                    continue;
                };
                if !visible(d.renderer) {
                    continue;
                }
                let Some(skin) = &prefab.renderers[d.renderer].skin else {
                    continue;
                };
                for (i, bind_pose) in bind_poses.iter().enumerate().take(MAX_BONES) {
                    let bone = skin.bones.get(i).copied().flatten();
                    let matrix = match bone {
                        Some(node) => world_of(node) * *bind_pose,
                        None => Mat4::IDENTITY,
                    };
                    palette[i * 16..i * 16 + 16].copy_from_slice(&matrix.to_cols_array());
                }
                gl.uniform_matrix_4_f32_slice(p.u("u_bones"), false, &palette);
                self.draw_model(gl, p, d);
            }

            // Mobile/Particles/Additive: Blend SrcAlpha One, ZWrite Off, Cull Off.
            if models
                .draws
                .iter()
                .any(|d| d.additive && visible(d.renderer))
            {
                let p = &self.additive;
                gl.use_program(Some(p.handle));
                gl.uniform_matrix_4_f32_slice(
                    p.u("u_view_proj"),
                    false,
                    &view_proj.to_cols_array(),
                );
                gl.uniform_1_i32(p.u("u_fog_mode"), self.fog_mode);
                let f = self.fog_params;
                gl.uniform_4_f32(p.u("u_fog_params"), f.x, f.y, f.z, f.w);
                gl.uniform_1_i32(p.u("u_main_tex"), 0);
                gl.disable(glow::CULL_FACE);
                gl.enable(glow::BLEND);
                gl.blend_func(glow::SRC_ALPHA, glow::ONE);
                gl.depth_mask(false);
                for d in models.draws.iter().filter(|d| d.additive) {
                    if !visible(d.renderer) {
                        continue;
                    }
                    let world = world_of(prefab.renderers[d.renderer].node);
                    gl.uniform_matrix_4_f32_slice(
                        p.u("u_object_to_world"),
                        false,
                        &world.to_cols_array(),
                    );
                    let s = d.main_st;
                    gl.uniform_4_f32(p.u("u_main_st"), s.x, s.y, s.z, s.w);
                    gl.bind_texture(glow::TEXTURE_2D, Some(d.texture));
                    gl.bind_vertex_array(Some(d.vao));
                    gl.draw_elements(glow::TRIANGLES, d.count, glow::UNSIGNED_INT, d.offset);
                }
                gl.depth_mask(true);
                gl.disable(glow::BLEND);
                gl.enable(glow::CULL_FACE);
            }
            gl.bind_vertex_array(None);
        }
    }

    /// Draws `OnGUI` quads over the frame (`GuiQuad::rect` in GUI pixels of a `width` ×
    /// `height` screen).
    pub fn draw_gui(&mut self, gl: &glow::Context, quads: &[GuiQuad], width: u32, height: u32) {
        if quads.is_empty() {
            return;
        }
        let textures: Vec<glow::Texture> = quads
            .iter()
            .map(|q| self.resources.texture(gl, Some(&q.texture)))
            .collect();
        let p = &self.gui;
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::CULL_FACE);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.active_texture(glow::TEXTURE0);
            gl.use_program(Some(p.handle));
            gl.uniform_2_f32(p.u("u_screen"), width as f32, height as f32);
            gl.uniform_1_i32(p.u("u_main_tex"), 0);
            gl.bind_vertex_array(Some(self.gui_quad.vao));
            let (offset, count) = self.gui_quad.submeshes[0];
            for (quad, texture) in quads.iter().zip(textures) {
                let [x, y, w, h] = quad.rect;
                gl.uniform_4_f32(p.u("u_rect"), x, y, w, h);
                let c = quad.color;
                gl.uniform_4_f32(p.u("u_tint"), c.x, c.y, c.z, c.w);
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.draw_elements(glow::TRIANGLES, count, glow::UNSIGNED_INT, offset);
            }
            gl.bind_vertex_array(None);
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::CULL_FACE);
        }
    }

    fn draw_model(&self, gl: &glow::Context, p: &Program, d: &ModelDraw) {
        unsafe {
            let s = d.main_st;
            gl.uniform_4_f32(p.u("u_main_st"), s.x, s.y, s.z, s.w);
            gl.uniform_3_f32(p.u("u_color"), d.color.x, d.color.y, d.color.z);
            gl.bind_texture(glow::TEXTURE_2D, Some(d.texture));
            gl.bind_vertex_array(Some(d.vao));
            gl.draw_elements(glow::TRIANGLES, d.count, glow::UNSIGNED_INT, d.offset);
        }
    }
}
