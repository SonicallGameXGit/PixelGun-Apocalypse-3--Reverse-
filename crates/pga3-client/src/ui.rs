//! The part of uGUI the `Main` scene uses (spec: `docs/gameplay/menu.md`, "The Main scene"):
//! a `Canvas` in "Screen Space - Camera" mode drawn by the GUI camera, `RectTransform` layout
//! from anchors, `Image`, `Text` (dynamic fonts), `Button` with its animator transitions,
//! `TiltWindow` and `PressHandler`.
//!
//! The canvas lives in the 3D scene: the GUI camera (perspective, fov 40) sees it at a plane
//! distance of 100, and the menu window sits 147 canvas units in front of that plane and tilts
//! with the mouse, so every quad is drawn through the camera's view-projection matrix.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{f32_of, load_json};
use crate::gui::{FONT_ATLAS, GuiFont, GuiQuad};
use crate::render::View;
use crate::unity;

/// `CanvasScaler.referenceResolution.y`, with `matchWidthOrHeight` 1 (match the height).
const REFERENCE_HEIGHT: f32 = 600.0;
/// `Canvas.planeDistance`.
const PLANE_DISTANCE: f32 = 100.0;
/// The button animator's transitions last 0.1 s (`AnyState -> Highlighted`, observed in the
/// `SF Button` controller).
const BUTTON_FADE: f32 = 0.1;

#[derive(Clone, Debug)]
pub struct UiImage {
    pub sprite: Option<String>,
    pub color: Vec4,
}

#[derive(Clone, Debug)]
pub struct UiText {
    pub text: String,
    pub font: Option<String>,
    pub size: i32,
    pub bold: bool,
    pub alignment: i32,
    pub color: Vec4,
    /// `m_HorizontalOverflow` 0 wraps, 1 overflows.
    pub wrap: bool,
    /// `m_VerticalOverflow` 0 truncates.
    pub truncate: bool,
}

/// A persistent call of a `UnityEvent`: the target component's ref and the method name.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub target: String,
    pub method: String,
}

#[derive(Clone, Debug, Default)]
pub struct UiButton {
    pub on_click: Vec<Call>,
    /// `PressHandler.OnPress`: fires on pointer down.
    pub on_press: Vec<Call>,
    /// 0..1: how far the highlight animation has run.
    pub highlight: f32,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub active: bool,
    anchor_min: Vec2,
    anchor_max: Vec2,
    anchored: Vec2,
    size_delta: Vec2,
    pivot: Vec2,
    local_z: f32,
    rotation: Quat,
    scale: Vec3,
    pub image: Option<UiImage>,
    pub text: Option<UiText>,
    pub button: Option<UiButton>,
    /// `TiltWindow.range`.
    tilt: Option<Vec2>,
    /// Computed by `Canvas::layout`: own rect in the node's local space (x, y up, width, height)
    /// and the world matrix.
    pub rect: [f32; 4],
    pub world: Mat4,
}

pub struct Canvas {
    pub nodes: Vec<Node>,
    by_ref: HashMap<String, usize>,
    root: usize,
    camera: View,
    /// `TiltWindow.mRot`.
    tilt: Vec2,
    pub aspect: f32,
}

fn vec2(v: &Value) -> Vec2 {
    Vec2::new(f32_of(&v["x"]), f32_of(&v["y"]))
}

fn color(v: &Value) -> Vec4 {
    Vec4::new(
        f32_of(&v["r"]),
        f32_of(&v["g"]),
        f32_of(&v["b"]),
        f32_of(&v["a"]),
    )
}

fn calls(event: &Value) -> Vec<Call> {
    event["m_PersistentCalls"]["m_Calls"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|c| {
                    Some(Call {
                        target: c["m_Target"]["ref"].as_str()?.to_string(),
                        method: c["m_MethodName"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

impl Canvas {
    /// Reads the `Canvas` subtree of `scenes/Main.json`. `camera` is the GUI camera.
    pub fn load(assets: &Path, camera: View) -> Result<Canvas> {
        let scene = load_json(&assets.join("data/scenes/Main.json"))?;
        let list = scene["nodes"].as_array().context("Main.json nodes")?;
        let index: HashMap<&str, &Value> = list
            .iter()
            .filter_map(|n| Some((n["ref"].as_str()?, n)))
            .collect();
        let root_ref = list
            .iter()
            .find(|n| {
                n["name"] == "Canvas"
                    && n["components"].as_array().is_some_and(|c| {
                        c.iter()
                            .any(|c| c["type"] == "Canvas" && c["data"]["m_RenderMode"] == 1)
                    })
            })
            .and_then(|n| n["ref"].as_str())
            .context("no screen-space Canvas in Main.json")?;
        let mut canvas = Canvas {
            nodes: Vec::new(),
            by_ref: HashMap::new(),
            root: 0,
            camera,
            tilt: Vec2::ZERO,
            aspect: 16.0 / 9.0,
        };
        canvas.add(&index, root_ref, None)?;
        Ok(canvas)
    }

    fn add(
        &mut self,
        index: &HashMap<&str, &Value>,
        reference: &str,
        parent: Option<usize>,
    ) -> Result<usize> {
        let Some(n) = index.get(reference) else {
            bail!("node {reference} missing");
        };
        let mut node = Node {
            name: n["name"].as_str().unwrap_or("").to_string(),
            parent,
            children: Vec::new(),
            active: n["active"].as_bool().unwrap_or(true),
            anchor_min: Vec2::splat(0.5),
            anchor_max: Vec2::splat(0.5),
            anchored: Vec2::ZERO,
            size_delta: Vec2::ZERO,
            pivot: Vec2::splat(0.5),
            local_z: 0.0,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            image: None,
            text: None,
            button: None,
            tilt: None,
            rect: [0.0; 4],
            world: Mat4::IDENTITY,
        };
        for c in n["components"].as_array().into_iter().flatten() {
            let ty = c["type"].as_str().unwrap_or("");
            if ty == "RectTransform" {
                let d = &c["data"];
                node.anchor_min = vec2(&d["m_AnchorMin"]);
                node.anchor_max = vec2(&d["m_AnchorMax"]);
                node.anchored = vec2(&d["m_AnchoredPosition"]);
                node.size_delta = vec2(&d["m_SizeDelta"]);
                node.pivot = vec2(&d["m_Pivot"]);
                node.local_z = f32_of(&d["m_LocalPosition"]["z"]);
                let q = &d["m_LocalRotation"];
                node.rotation = Quat::from_xyzw(
                    f32_of(&q["x"]),
                    f32_of(&q["y"]),
                    f32_of(&q["z"]),
                    f32_of(&q["w"]),
                );
                let s = &d["m_LocalScale"];
                node.scale = Vec3::new(f32_of(&s["x"]), f32_of(&s["y"]), f32_of(&s["z"]));
                continue;
            }
            if ty != "MonoBehaviour" {
                continue;
            }
            let f = &c["fields"];
            match c["script"].as_str().unwrap_or("") {
                "UnityEngine.UI.Image" => {
                    node.image = Some(UiImage {
                        sprite: f["m_Sprite"]["asset"].as_str().map(str::to_string),
                        color: color(&f["m_Color"]),
                    });
                }
                "UnityEngine.UI.Text" => {
                    let d = &f["m_FontData"];
                    let style = d["m_FontStyle"].as_i64().unwrap_or(0);
                    node.text = Some(UiText {
                        text: f["m_Text"].as_str().unwrap_or("").to_string(),
                        font: d["m_Font"]["asset"].as_str().map(str::to_string),
                        size: d["m_FontSize"].as_i64().unwrap_or(14) as i32,
                        bold: style == 1 || style == 3,
                        alignment: d["m_Alignment"].as_i64().unwrap_or(0) as i32,
                        color: color(&f["m_Color"]),
                        wrap: d["m_HorizontalOverflow"].as_i64() == Some(0),
                        truncate: d["m_VerticalOverflow"].as_i64() == Some(0),
                    });
                }
                "UnityEngine.UI.Button" => {
                    let button = node.button.get_or_insert_with(UiButton::default);
                    button.on_click = calls(&f["m_OnClick"]);
                }
                "PressHandler" => {
                    let button = node.button.get_or_insert_with(UiButton::default);
                    button.on_press = calls(&f["OnPress"]);
                }
                // The Window's TiltWindow is serialized disabled, so it never runs there.
                "TiltWindow" if c["enabled"].as_bool().unwrap_or(true) => {
                    node.tilt = Some(vec2(&f["range"]));
                }
                _ => {}
            }
        }
        let id = self.nodes.len();
        self.by_ref.insert(reference.to_string(), id);
        self.nodes.push(node);
        if let Some(p) = parent {
            self.nodes[p].children.push(id);
        }
        for child in n["children"].as_array().into_iter().flatten() {
            if let Some(r) = child.as_str() {
                // Only RectTransform children belong to the canvas (`MenuManager` etc. are
                // plain Transforms that hold scripts).
                let is_ui = index.get(r).is_some_and(|c| {
                    c["components"]
                        .as_array()
                        .is_some_and(|l| l.iter().any(|x| x["type"] == "RectTransform"))
                });
                if is_ui {
                    self.add(index, r, Some(id))?;
                }
            }
        }
        Ok(id)
    }

    fn in_hierarchy(&self, mut node: usize) -> bool {
        loop {
            if !self.nodes[node].active {
                return false;
            }
            match self.nodes[node].parent {
                Some(p) => node = p,
                None => return true,
            }
        }
    }

    /// Canvas scale: world units per canvas unit (the camera frustum's height at the plane
    /// distance over the canvas height).
    fn canvas_scale(&self) -> f32 {
        2.0 * PLANE_DISTANCE * (self.camera.fov_y_degrees.to_radians() * 0.5).tan()
            / REFERENCE_HEIGHT
    }

    /// Computes rects and world matrices. `mouse` is the cursor in window pixels (y down) for
    /// `TiltWindow`; `dt` advances its smoothing.
    pub fn layout(&mut self, aspect: f32, screen: (f32, f32), mouse: Vec2, dt: f32) {
        self.aspect = aspect;
        // TiltWindow.Update: mRot = Lerp(mRot, clamp((mouse - half) / half), dt * 5).
        let half = Vec2::new(screen.0 * 0.5, screen.1 * 0.5);
        let target = Vec2::new(
            ((mouse.x - half.x) / half.x).clamp(-1.0, 1.0),
            // Input.mousePosition has y up.
            (((screen.1 - mouse.y) - half.y) / half.y).clamp(-1.0, 1.0),
        );
        self.tilt = self.tilt.lerp(target, (dt * 5.0).clamp(0.0, 1.0));
        let scale = self.canvas_scale();
        let size = Vec2::new(REFERENCE_HEIGHT * aspect, REFERENCE_HEIGHT);
        let center =
            self.camera.position + self.camera.rotation * Vec3::new(0.0, 0.0, PLANE_DISTANCE);
        let root = self.root;
        self.nodes[root].rect = [-size.x * 0.5, -size.y * 0.5, size.x, size.y];
        self.nodes[root].world =
            Mat4::from_scale_rotation_translation(Vec3::splat(scale), self.camera.rotation, center);
        self.layout_children(root);
    }

    fn layout_children(&mut self, parent: usize) {
        let [px, py, pw, ph] = self.nodes[parent].rect;
        let psize = Vec2::new(pw, ph);
        let pmin = Vec2::new(px, py);
        let pworld = self.nodes[parent].world;
        for child in self.nodes[parent].children.clone() {
            let tilt = self.tilt;
            let n = &mut self.nodes[child];
            let size = (n.anchor_max - n.anchor_min) * psize + n.size_delta;
            let anchor_ref =
                pmin + (n.anchor_min + (n.anchor_max - n.anchor_min) * n.pivot) * psize;
            let pos = anchor_ref + n.anchored;
            let mut rotation = n.rotation;
            if let Some(range) = n.tilt {
                // TiltWindow: start * Euler(-mRot.y * range.y, mRot.x * range.x, 0).
                rotation *= unity::euler(Vec3::new(-tilt.y * range.y, tilt.x * range.x, 0.0));
            }
            n.rect = [-n.pivot.x * size.x, -n.pivot.y * size.y, size.x, size.y];
            n.world = pworld
                * Mat4::from_scale_rotation_translation(
                    n.scale,
                    rotation,
                    Vec3::new(pos.x, pos.y, n.local_z),
                );
            self.layout_children(child);
        }
    }

    /// The camera's view-projection matrix for the current aspect.
    pub fn view_projection(&self) -> Mat4 {
        let (view, proj) = self.camera.matrices(self.aspect);
        proj * view
    }

    /// The topmost active button under the cursor (window pixels, y down).
    pub fn button_at(&self, mouse: Vec2, screen: (f32, f32)) -> Option<usize> {
        let inverse = self.view_projection().inverse();
        let ndc = Vec2::new(
            mouse.x / screen.0 * 2.0 - 1.0,
            1.0 - mouse.y / screen.1 * 2.0,
        );
        let near = inverse.project_point3(Vec3::new(ndc.x, ndc.y, -1.0));
        let far = inverse.project_point3(Vec3::new(ndc.x, ndc.y, 1.0));
        let mut hit = None;
        for (i, n) in self.nodes.iter().enumerate() {
            if n.button.is_none() || !self.in_hierarchy(i) {
                continue;
            }
            let to_local = n.world.inverse();
            let a = to_local.transform_point3(near);
            let b = to_local.transform_point3(far);
            if (b.z - a.z).abs() < 1e-9 {
                continue;
            }
            let t = -a.z / (b.z - a.z);
            let p = a + (b - a) * t;
            let [x, y, w, h] = n.rect;
            if p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h {
                hit = Some(i);
            }
        }
        hit
    }

    /// Advances the buttons' highlight animation: `hovered` runs towards 1, the others back
    /// to 0 (the animator's 0.1 s transitions).
    pub fn animate(&mut self, hovered: Option<usize>, dt: f32) {
        for (i, n) in self.nodes.iter_mut().enumerate() {
            if let Some(b) = &mut n.button {
                let target = if Some(i) == hovered { 1.0 } else { 0.0 };
                let step = dt / BUTTON_FADE;
                b.highlight = if b.highlight < target {
                    (b.highlight + step).min(target)
                } else {
                    (b.highlight - step).max(target)
                };
            }
        }
    }

    /// The highlight of the button a node belongs to (`Highlighted` and `Pressed` set the
    /// `Background` image's colour to opaque white; returning to `Normal` restores the
    /// serialized colour).
    fn highlight_of(&self, mut node: usize) -> Option<f32> {
        let mut prev_is_background = self.nodes[node].name == "Background";
        while let Some(p) = self.nodes[node].parent {
            if let Some(b) = &self.nodes[p].button
                && prev_is_background
            {
                return Some(b.highlight);
            }
            prev_is_background = self.nodes[p].name == "Background";
            node = p;
        }
        None
    }

    /// Every drawn quad, in the canvas's depth-first order.
    pub fn quads(&self, font: &mut GuiFont, screen: (f32, f32)) -> Vec<GuiQuad> {
        let vp = self.view_projection();
        let mut quads = Vec::new();
        self.draw_node(self.root, font, screen, vp, &mut quads);
        quads
    }

    fn draw_node(
        &self,
        i: usize,
        font: &mut GuiFont,
        screen: (f32, f32),
        vp: Mat4,
        out: &mut Vec<GuiQuad>,
    ) {
        let n = &self.nodes[i];
        if !n.active {
            return;
        }
        let mvp = vp * n.world * Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0));
        let [x, y, w, h] = n.rect;
        // Rects are y up; the shader wants y down from the top edge.
        let rect = [x, -(y + h), w, h];
        if let Some(image) = &n.image
            && let Some(sprite) = &image.sprite
        {
            let mut c = image.color;
            if let Some(t) = self.highlight_of(i) {
                c.w += (1.0 - c.w) * t;
            }
            if c.w > 0.0 {
                out.push(GuiQuad {
                    texture: sprite.clone(),
                    rect,
                    uv: [0.0, 0.0, 1.0, 1.0],
                    color: c,
                    mvp: Some(mvp),
                });
            }
        }
        if let Some(text) = &n.text {
            // Pixels on screen per canvas unit at this node (perspective included).
            let ppu = {
                let a = vp * n.world * Vec4::new(0.0, 0.0, 0.0, 1.0);
                let b = vp * n.world * Vec4::new(1.0, 0.0, 0.0, 1.0);
                let pa = Vec2::new(a.x / a.w * screen.0 * 0.5, a.y / a.w * screen.1 * 0.5);
                let pb = Vec2::new(b.x / b.w * screen.0 * 0.5, b.y / b.w * screen.1 * 0.5);
                pa.distance(pb).max(0.01)
            };
            text_quads(font, text, rect, ppu, mvp, out);
        }
        for &c in &n.children {
            self.draw_node(c, font, screen, vp, out);
        }
    }
}

/// Lays a `Text` out in `rect` (canvas units, y down) and appends one quad per glyph. The font
/// is rasterised at the on-screen size (`ppu` pixels per unit) like a dynamic font under a
/// scaled canvas, and laid out in those pixels.
fn text_quads(
    font: &mut GuiFont,
    text: &UiText,
    rect: [f32; 4],
    ppu: f32,
    mvp: Mat4,
    out: &mut Vec<GuiQuad>,
) {
    if text.text.is_empty() {
        return;
    }
    let face = font.face(text.font.as_deref());
    let size = ((text.size as f32 * ppu).round() as i32).max(1);
    let avail_w = rect[2] * ppu;
    let avail_h = rect[3] * ppu;
    let atlas = font.atlas_size as f32;
    // Greedy word wrap into lines of (glyph, pen) lists.
    struct Pen {
        pen: f32,
        glyph: crate::gui::Glyph,
    }
    let mut lines: Vec<(Vec<Pen>, f32)> = vec![(Vec::new(), 0.0)];
    let mut word: Vec<Pen> = Vec::new();
    let mut word_w = 0.0f32;
    let flush = |lines: &mut Vec<(Vec<Pen>, f32)>, word: &mut Vec<Pen>, word_w: &mut f32| {
        if word.is_empty() {
            return;
        }
        let (line, width) = lines.last_mut().unwrap_or_else(|| unreachable!());
        for mut p in word.drain(..) {
            p.pen += *width;
            line.push(p);
        }
        *width += *word_w;
        *word_w = 0.0;
    };
    for c in text.text.chars() {
        if c == '\n' {
            flush(&mut lines, &mut word, &mut word_w);
            lines.push((Vec::new(), 0.0));
            continue;
        }
        let Some(glyph) = font.glyph(face, c, size, text.bold) else {
            continue;
        };
        if c == ' ' {
            flush(&mut lines, &mut word, &mut word_w);
            let (line, width) = lines.last_mut().unwrap_or_else(|| unreachable!());
            line.push(Pen { pen: *width, glyph });
            *width += glyph.advance;
            continue;
        }
        // Wrap before a word that does not fit on a non-empty line.
        let line_w = lines.last().map_or(0.0, |l| l.1);
        if text.wrap
            && line_w + word_w + glyph.advance > avail_w
            && !lines.last().is_some_and(|l| l.0.is_empty())
            && word_w + glyph.advance <= avail_w
        {
            lines.push((Vec::new(), 0.0));
        }
        word.push(Pen { pen: word_w, glyph });
        word_w += glyph.advance;
    }
    flush(&mut lines, &mut word, &mut word_w);
    let line_h = font.line_height(face, size);
    let ascent = font.ascent(face, size);
    // Vertical truncation: lines that do not fit are dropped (the first always stays).
    if text.truncate {
        let fit = ((avail_h / line_h).floor() as usize).max(1);
        lines.truncate(fit);
    }
    let total = lines.len() as f32 * line_h;
    let (ax, ay) = (
        (text.alignment % 3) as f32 * 0.5,
        (text.alignment / 3) as f32 * 0.5,
    );
    let top = (avail_h - total) * ay;
    for (k, (line, width)) in lines.iter().enumerate() {
        // Trailing spaces do not count towards the line's width.
        let trailing: f32 = line
            .iter()
            .rev()
            .take_while(|p| p.glyph.atlas[2] == 0)
            .map(|p| p.glyph.advance)
            .sum();
        let width = width - trailing;
        let x0 = (avail_w - width) * ax;
        let baseline = top + k as f32 * line_h + ascent;
        for p in line {
            let [gx, gy, gw, gh] = p.glyph.atlas;
            if gw == 0 {
                continue;
            }
            let px = x0 + p.pen + p.glyph.offset.0 as f32;
            let py = baseline + p.glyph.offset.1 as f32;
            out.push(GuiQuad {
                texture: FONT_ATLAS.to_string(),
                rect: [
                    rect[0] + px / ppu,
                    rect[1] + py / ppu,
                    gw as f32 / ppu,
                    gh as f32 / ppu,
                ],
                uv: [
                    gx as f32 / atlas,
                    gy as f32 / atlas,
                    (gx + gw) as f32 / atlas,
                    (gy + gh) as f32 / atlas,
                ],
                color: text.color,
                mvp: Some(mvp),
            });
        }
    }
}
