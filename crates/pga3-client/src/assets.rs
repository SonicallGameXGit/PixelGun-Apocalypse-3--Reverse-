//! Access to the extracted original assets (`assets/`, see docs/assets.md).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use glam::{Vec2, Vec4};
use serde_json::Value;

/// Finds `assets/`: `$PGA3_ASSETS`, else the nearest `assets/manifest.json` walking up from the
/// current directory, else the repository root this crate was built from.
pub fn find_assets_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("PGA3_ASSETS") {
        return Ok(PathBuf::from(dir));
    }
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.extend(cwd.ancestors().map(|dir| dir.join("assets")));
    }
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets"));
    for dir in candidates {
        if dir.join("manifest.json").is_file() {
            return Ok(dir);
        }
    }
    bail!("assets/ not found (set PGA3_ASSETS)")
}

/// Reads an `assets/data` JSON file. The extractor (Python `json`) writes non-finite floats
/// (particle curve slopes) as the bare tokens `NaN`, `Infinity` and `-Infinity`, which are not
/// JSON: those files are re-read with the tokens quoted, and `f32_of` turns them back into floats.
pub fn load_json(path: &Path) -> Result<Value> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    match serde_json::from_str(&text) {
        Ok(value) => Ok(value),
        Err(_) => serde_json::from_str(&quote_non_finite(&text))
            .with_context(|| format!("parsing {}", path.display())),
    }
}

/// Quotes the `NaN` / `Infinity` / `-Infinity` tokens outside strings.
fn quote_non_finite(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 64);
    let mut in_string = false;
    let mut escaped = false;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if let Some(token) = ["-Infinity", "Infinity", "NaN"]
            .into_iter()
            .find(|t| rest.starts_with(t))
        {
            out.push('"');
            out.push_str(token);
            out.push('"');
            rest = &rest[token.len()..];
            continue;
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

pub fn f32_of(value: &Value) -> f32 {
    match value.as_str() {
        Some("Infinity") => f32::INFINITY,
        Some("-Infinity") => f32::NEG_INFINITY,
        Some("NaN") => f32::NAN,
        _ => value.as_f64().unwrap_or(0.0) as f32,
    }
}

pub fn vec4_of(value: &Value, keys: [&str; 4]) -> Vec4 {
    Vec4::new(
        f32_of(&value[keys[0]]),
        f32_of(&value[keys[1]]),
        f32_of(&value[keys[2]]),
        f32_of(&value[keys[3]]),
    )
}

pub fn color_of(value: &Value) -> Vec4 {
    vec4_of(value, ["r", "g", "b", "a"])
}

/// `{ "ref": "...", "asset": "..." }` reference → (ref, published asset path).
pub fn ref_of(value: &Value) -> Option<(&str, Option<&str>)> {
    let reference = value.get("ref")?.as_str()?;
    Some((reference, value.get("asset").and_then(Value::as_str)))
}

/// `TagManager` custom tags start at this number in the tag field of exported GameObjects
/// (Unity's internal numbering; built-in tags are below it).
const FIRST_CUSTOM_TAG: u32 = 20000;

/// The custom tags of `data/settings/TagManager.json`, for `tag_name`.
pub fn load_tags(assets: &Path) -> Result<Vec<String>> {
    let json = load_json(&assets.join("data/settings/TagManager.json"))?;
    Ok(json["tags"]
        .as_array()
        .context("TagManager has no tags")?
        .iter()
        .map(|t| t.as_str().unwrap_or_default().to_string())
        .collect())
}

/// `GameObject.tag` of a numeric tag: Unity's built-in tags, then `custom[tag - 20000]`.
pub fn tag_name(tag: u32, custom: &[String]) -> &str {
    match tag {
        0 => "Untagged",
        1 => "Respawn",
        2 => "Finish",
        3 => "EditorOnly",
        5 => "MainCamera",
        6 => "Player",
        7 => "GameController",
        _ => tag
            .checked_sub(FIRST_CUSTOM_TAG)
            .and_then(|i| custom.get(i as usize))
            .map_or("Untagged", String::as_str),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderKind {
    /// `Mobile/Diffuse`: Lambert, main directional light + SH ambient, no tint.
    MobileDiffuse,
    /// `Diffuse` / `Legacy Shaders/Diffuse`: same lighting, albedo tinted by `_Color`.
    Diffuse,
    /// `BlendedDecal`: unlit `_MainTex * _Color`, alpha test (a > 0), ZWrite off, offset -1,-1.
    BlendedDecal,
    /// Unlit transparent particle shaders (ZWrite off, Cull off, Transparent queue), e.g. the
    /// weapons' muzzle flashes, the red-dot sights, impacts and the explosion.
    Particle(ParticleShader),
    /// `Mobile/Skybox`: 6-sided skybox.
    Skybox,
    /// Anything not implemented yet; drawn as `Diffuse` and reported.
    Unsupported,
}

/// The unlit particle shaders of this build. Fragment programs are from the compiled GLSL ES
/// programs in `assets/shaders/` (non-soft-particle variants: `QualitySettings.softParticles` is
/// off at every level). The dump has no blend state; it is taken from the shaders' public sources
/// (Unity built-in shaders 2018.3, War FX) and is **inferred**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleShader {
    /// `Mobile/Particles/Additive`: `tex · colour`; `Blend SrcAlpha One`; fog to black.
    MobileAdditive,
    /// `Particles/Additive`: `2 · colour · _TintColor · tex`; `Blend SrcAlpha One`, fog to black.
    Additive,
    /// `Legacy Shaders/Particles/Additive (Soft)`: `t = tex · colour; (t.rgb · t.a, t.a)`;
    /// `Blend One OneMinusSrcColor`, fog to black.
    SoftAdditive,
    /// `Legacy Shaders/Particles/Alpha Blended`: `2 · colour · _TintColor · tex` (alpha
    /// clamped); `Blend SrcAlpha OneMinusSrcAlpha`, fog to the fog colour.
    AlphaBlended,
    /// `Hidden/Particles/Alpha Blended +100` (the shotgun smoke's `Smoke2`): the same program
    /// without fog variants or soft particles, queue Transparent+100.
    AlphaBlendedNoFog,
    /// `WFX/Additive Alpha8`: `2 · colour · _TintColor · (2 · tex.a · colour.a)`;
    /// `Blend SrcAlpha One`, fog to black.
    WfxAdditiveAlpha8,
    /// `WFX/Multiply Alpha8`: `tex.a · colour`; `Blend Zero SrcColor`, fog to white.
    WfxMultiplyAlpha8,
}

impl ParticleShader {
    /// `_TintColor` is read by these shaders (its default is (0.5, 0.5, 0.5, 0.5)).
    pub fn uses_tint(self) -> bool {
        matches!(
            self,
            ParticleShader::Additive
                | ParticleShader::AlphaBlended
                | ParticleShader::AlphaBlendedNoFog
                | ParticleShader::WfxAdditiveAlpha8
        )
    }
}

pub struct TexEnv {
    pub texture: Option<String>,
    pub scale: Vec2,
    pub offset: Vec2,
}

pub struct Material {
    pub shader_name: String,
    pub shader: ShaderKind,
    pub textures: HashMap<String, TexEnv>,
    pub colors: HashMap<String, Vec4>,
}

impl Material {
    pub fn color(&self, name: &str) -> Vec4 {
        self.colors.get(name).copied().unwrap_or(Vec4::ONE)
    }

    /// Texture path and `_ST` (scale.xy, offset.zw) of a texture property.
    pub fn texture(&self, name: &str) -> (Option<&str>, Vec4) {
        match self.textures.get(name) {
            Some(env) => (
                env.texture.as_deref(),
                Vec4::new(env.scale.x, env.scale.y, env.offset.x, env.offset.y),
            ),
            None => (None, Vec4::new(1.0, 1.0, 0.0, 0.0)),
        }
    }
}

fn shader_kind(name: &str) -> ShaderKind {
    match name {
        "Mobile/Diffuse" => ShaderKind::MobileDiffuse,
        "Diffuse" | "Legacy Shaders/Diffuse" => ShaderKind::Diffuse,
        "BlendedDecal" => ShaderKind::BlendedDecal,
        "Mobile/Particles/Additive" => ShaderKind::Particle(ParticleShader::MobileAdditive),
        "Particles/Additive" | "Legacy Shaders/Particles/Additive" => {
            ShaderKind::Particle(ParticleShader::Additive)
        }
        // Unity 5.3 names (CrazyGames build); 2017.1+ moved them under "Legacy Shaders/". Same
        // programs (docs/crazygames-build.md, "Assets").
        "Particles/Additive (Soft)" | "Legacy Shaders/Particles/Additive (Soft)" => {
            ShaderKind::Particle(ParticleShader::SoftAdditive)
        }
        "Particles/Alpha Blended" | "Legacy Shaders/Particles/Alpha Blended" => {
            ShaderKind::Particle(ParticleShader::AlphaBlended)
        }
        "Hidden/Particles/Alpha Blended +100" => {
            ShaderKind::Particle(ParticleShader::AlphaBlendedNoFog)
        }
        "WFX/Additive Alpha8" => ShaderKind::Particle(ParticleShader::WfxAdditiveAlpha8),
        "WFX/Multiply Alpha8" => ShaderKind::Particle(ParticleShader::WfxMultiplyAlpha8),
        "Mobile/Skybox" => ShaderKind::Skybox,
        _ => ShaderKind::Unsupported,
    }
}

/// Parses `data/materials.json` (keyed by ref).
pub fn load_materials(assets: &Path) -> Result<HashMap<String, Material>> {
    let json = load_json(&assets.join("data/materials.json"))?;
    let entries = json
        .as_object()
        .context("materials.json is not an object")?;
    let mut out = HashMap::new();
    for (key, data) in entries {
        let shader_name = data["m_Shader"]["name"].as_str().unwrap_or("").to_string();
        let props = &data["m_SavedProperties"];
        let mut textures = HashMap::new();
        for pair in props["m_TexEnvs"].as_array().into_iter().flatten() {
            let (Some(name), env) = (pair[0].as_str(), &pair[1]) else {
                continue;
            };
            textures.insert(
                name.to_string(),
                TexEnv {
                    texture: env["m_Texture"]["asset"].as_str().map(str::to_string),
                    scale: Vec2::new(f32_of(&env["m_Scale"]["x"]), f32_of(&env["m_Scale"]["y"])),
                    offset: Vec2::new(f32_of(&env["m_Offset"]["x"]), f32_of(&env["m_Offset"]["y"])),
                },
            );
        }
        let mut colors = HashMap::new();
        for pair in props["m_Colors"].as_array().into_iter().flatten() {
            if let Some(name) = pair[0].as_str() {
                colors.insert(name.to_string(), color_of(&pair[1]));
            }
        }
        out.insert(
            key.clone(),
            Material {
                shader: shader_kind(&shader_name),
                shader_name,
                textures,
                colors,
            },
        );
    }
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Point,
    Bilinear,
    Trilinear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrap {
    Repeat,
    Clamp,
    Mirror,
}

/// Sampler settings of a texture, from `data/textures.json`.
#[derive(Clone, Copy, Debug)]
pub struct TextureInfo {
    pub filter: Filter,
    pub wrap_u: Wrap,
    pub wrap_v: Wrap,
    pub mipmaps: bool,
}

impl Default for TextureInfo {
    fn default() -> Self {
        TextureInfo {
            filter: Filter::Bilinear,
            wrap_u: Wrap::Repeat,
            wrap_v: Wrap::Repeat,
            mipmaps: true,
        }
    }
}

fn wrap_of(value: &Value) -> Wrap {
    match value.as_str() {
        Some("clamp") => Wrap::Clamp,
        Some("mirror") | Some("mirror_once") => Wrap::Mirror,
        _ => Wrap::Repeat,
    }
}

pub fn load_texture_infos(assets: &Path) -> Result<HashMap<String, TextureInfo>> {
    let json = load_json(&assets.join("data/textures.json"))?;
    let entries = json.as_object().context("textures.json is not an object")?;
    let mut out = HashMap::new();
    for (path, info) in entries {
        let filter = match info["filter"].as_str() {
            Some("point") => Filter::Point,
            Some("trilinear") => Filter::Trilinear,
            _ => Filter::Bilinear,
        };
        out.insert(
            path.clone(),
            TextureInfo {
                filter,
                wrap_u: wrap_of(&info["wrap_u"]),
                wrap_v: wrap_of(&info["wrap_v"]),
                mipmaps: info["mip_count"].as_u64().unwrap_or(1) > 1,
            },
        );
    }
    Ok(out)
}
