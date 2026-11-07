//! The subset of Unity's immediate-mode GUI (`OnGUI`) the HUD scripts use (spec:
//! `docs/gameplay/hud.md`): `GUISkin` styles, `GUI.Box` / `GUI.Label` / `GUI.DrawTexture`, a
//! `GUILayout` row of boxes, `GUI.color`, text in the built-in Arial font (Liberation Sans) with
//! rich-text tags, and backgrounds drawn with their style border (9-slice).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use glam::Vec4;
use serde_json::Value;

use crate::assets::{f32_of, load_json};

/// Texture name of the glyph atlas in `GuiQuad::texture`.
pub const FONT_ATLAS: &str = "@font";
const ATLAS_SIZE: usize = 1024;

/// The built-in Arial `Font`: `m_FontSize` 13, `m_LineSpacing` 14.94873, `m_Ascent` 11.76855
/// (in pixels at size 13).
const DEFAULT_FONT_SIZE: i32 = 13;
const LINE_SPACING: f32 = 14.948_73 / 13.0;
const ASCENT: f32 = 11.768_555 / 13.0;

/// A textured screen rectangle (GUI pixels, origin top left). `uv` = (u0, v0, u1, v1) with v
/// growing downwards like GUI y.
#[derive(Clone, Debug, PartialEq)]
pub struct GuiQuad {
    pub texture: String,
    /// x, y, width, height.
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub color: Vec4,
}

impl GuiQuad {
    /// The whole texture over `rect`.
    pub fn texture(texture: &str, rect: [f32; 4], color: Vec4) -> GuiQuad {
        GuiQuad {
            texture: texture.to_string(),
            rect,
            uv: [0.0, 0.0, 1.0, 1.0],
            color,
        }
    }
}

/// `TextAnchor`.
fn anchor(alignment: i32) -> (f32, f32) {
    let h = (alignment % 3) as f32 * 0.5;
    let v = (alignment / 3) as f32 * 0.5;
    (h, v)
}

/// A `GUIStyle` (normal state only: the HUD never hovers or presses).
#[derive(Clone, Debug, PartialEq)]
pub struct GuiStyle {
    pub background: Option<String>,
    pub text_color: Vec4,
    /// left, right, top, bottom.
    pub border: [f32; 4],
    pub margin: [f32; 4],
    pub padding: [f32; 4],
    /// 0: the font's size.
    pub font_size: i32,
    /// `FontStyle`: 0 normal, 1 bold, 2 italic, 3 bold and italic.
    pub font_style: i32,
    /// `TextAnchor` 0..8 (upper left .. lower right).
    pub alignment: i32,
    pub word_wrap: bool,
    pub rich_text: bool,
    pub stretch_width: bool,
}

fn offsets(value: &Value) -> [f32; 4] {
    ["left", "right", "top", "bottom"].map(|k| value[k].as_f64().unwrap_or(0.0) as f32)
}

impl GuiStyle {
    pub fn from_json(value: &Value) -> GuiStyle {
        let normal = &value["normal"];
        let c = &normal["textColor"];
        GuiStyle {
            background: normal["background"]["asset"].as_str().map(str::to_string),
            text_color: Vec4::new(
                f32_of(&c["r"]),
                f32_of(&c["g"]),
                f32_of(&c["b"]),
                f32_of(&c["a"]),
            ),
            border: offsets(&value["border"]),
            margin: offsets(&value["margin"]),
            padding: offsets(&value["padding"]),
            font_size: value["fontSize"].as_i64().unwrap_or(0) as i32,
            font_style: value["fontStyle"].as_i64().unwrap_or(0) as i32,
            alignment: value["alignment"].as_i64().unwrap_or(0) as i32,
            word_wrap: value["wordWrap"].as_bool().unwrap_or(false),
            rich_text: value["richText"].as_bool().unwrap_or(false),
            stretch_width: value["stretchWidth"].as_bool().unwrap_or(false),
        }
    }
}

/// The styles of a `GUISkin` the HUD uses.
#[derive(Clone, Debug)]
pub struct GuiSkin {
    pub box_style: GuiStyle,
    pub label: GuiStyle,
}

/// Loads a `GUISkin` by object ref (`resources.assets:40959`) from
/// `assets/data/scriptable/UnityEngine.GUISkin/`.
pub fn load_skin(assets: &Path, reference: &str) -> Result<GuiSkin> {
    let dir = assets.join("data/scriptable/UnityEngine.GUISkin");
    let suffix = format!("__{}.json", reference.replace(':', "_"));
    let file = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(&suffix))
        })
        .with_context(|| format!("GUISkin {reference} not found in {}", dir.display()))?;
    let json = load_json(&file)?;
    let f = &json["fields"];
    Ok(GuiSkin {
        box_style: GuiStyle::from_json(&f["m_box"]),
        label: GuiStyle::from_json(&f["m_label"]),
    })
}

/// Unity's default `GameSkin` (`unity_default_resources:11000`), what `OnGUI` starts with.
pub const DEFAULT_SKIN: &str = "unity_default_resources:11000";

#[derive(Clone, Copy, Debug)]
struct Glyph {
    /// Atlas rectangle in pixels (x, y, w, h); empty for blank glyphs.
    atlas: [u32; 4],
    /// Bitmap offset from the pen position: x right, y down from the baseline to the top row.
    offset: (i32, i32),
    advance: f32,
}

/// Liberation Sans rasterised on demand into an alpha atlas (rows top-down, RGBA with white
/// colour and the coverage in alpha).
pub struct GuiFont {
    font: fontdue::Font,
    glyphs: HashMap<(char, i32, bool), Option<Glyph>>,
    pub atlas: Vec<u8>,
    pub atlas_size: u32,
    /// Bumped when the atlas changes (re-upload).
    pub version: u64,
    cursor: (u32, u32),
    row_height: u32,
}

impl GuiFont {
    pub fn load(assets: &Path) -> Result<GuiFont> {
        let path = assets.join("fonts/Arial.ttf");
        let data = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        Ok(GuiFont {
            font,
            glyphs: HashMap::new(),
            atlas: vec![0; ATLAS_SIZE * ATLAS_SIZE * 4],
            atlas_size: ATLAS_SIZE as u32,
            version: 1,
            cursor: (1, 1),
            row_height: 0,
        })
    }

    /// `Font.HasCharacter`: characters outside the embedded font are not drawn.
    pub fn has_character(&self, c: char) -> bool {
        self.font.lookup_glyph_index(c) != 0
    }

    fn glyph(&mut self, c: char, size: i32, bold: bool) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&(c, size, bold)) {
            return *g;
        }
        let glyph = self.rasterize(c, size, bold);
        self.glyphs.insert((c, size, bold), glyph);
        glyph
    }

    fn rasterize(&mut self, c: char, size: i32, bold: bool) -> Option<Glyph> {
        if !self.has_character(c) {
            return None;
        }
        let (metrics, coverage) = self.font.rasterize(c, size as f32);
        // Synthetic bold (the embedded font has no bold face): strokes widened by `extra` pixels.
        let extra = if bold {
            ((size as f32) / 24.0).ceil().max(1.0) as usize
        } else {
            0
        };
        let width = if metrics.width == 0 {
            0
        } else {
            metrics.width + extra
        };
        let height = metrics.height;
        // Unity rounds glyph advances (`m_ShouldRoundAdvanceValue`).
        let advance = (metrics.advance_width + extra as f32).round();
        let offset = (metrics.xmin, -(metrics.ymin + metrics.height as i32));
        if width == 0 || height == 0 {
            return Some(Glyph {
                atlas: [0, 0, 0, 0],
                offset,
                advance,
            });
        }
        let (w, h) = (width as u32, height as u32);
        if self.cursor.0 + w + 1 > self.atlas_size {
            self.cursor = (1, self.cursor.1 + self.row_height + 1);
            self.row_height = 0;
        }
        if self.cursor.1 + h + 1 > self.atlas_size {
            // Full: start over (rarely happens with the HUD's few sizes).
            self.glyphs.clear();
            self.atlas.iter_mut().for_each(|b| *b = 0);
            self.cursor = (1, 1);
            self.row_height = 0;
        }
        let (x0, y0) = self.cursor;
        for y in 0..height {
            for x in 0..width {
                let mut a = 0u8;
                for dx in 0..=extra {
                    if x >= dx && x - dx < metrics.width {
                        a = a.max(coverage[y * metrics.width + x - dx]);
                    }
                }
                let i = (((y0 as usize + y) * ATLAS_SIZE) + x0 as usize + x) * 4;
                self.atlas[i..i + 4].copy_from_slice(&[255, 255, 255, a]);
            }
        }
        self.cursor.0 += w + 1;
        self.row_height = self.row_height.max(h);
        self.version += 1;
        Some(Glyph {
            atlas: [x0, y0, w, h],
            offset,
            advance,
        })
    }
}

/// A run of text with one size, style and colour.
#[derive(Clone, Debug, PartialEq)]
struct Run {
    text: String,
    size: i32,
    bold: bool,
    color: Option<Vec4>,
}

/// Unity's rich-text colour names (compared ignoring ASCII case), from the native table the
/// colour parser (`f11465`) indexes.
fn named_color(name: &str) -> Option<Vec4> {
    let rgba: u32 = match name.to_ascii_lowercase().as_str() {
        "red" => 0xff0000ff,
        "cyan" | "aqua" => 0x00ffffff,
        "blue" => 0x0000ffff,
        "darkblue" => 0x0000a0ff,
        "lightblue" => 0xadd8e6ff,
        "purple" => 0x800080ff,
        "yellow" => 0xffff00ff,
        "lime" => 0x00ff00ff,
        "fuchsia" | "magenta" => 0xff00ffff,
        "white" => 0xffffffff,
        "silver" => 0xc0c0c0ff,
        "grey" => 0x808080ff,
        "black" => 0x000000ff,
        "orange" => 0xffa500ff,
        "brown" => 0xa52a2aff,
        "maroon" => 0x800000ff,
        "green" => 0x008000ff,
        "olive" => 0x808000ff,
        "navy" => 0x000080ff,
        "teal" => 0x008080ff,
        "transparent" => 0x00000000,
        _ => return None,
    };
    let channel = |shift: u32| ((rgba >> shift) & 255) as f32 / 255.0;
    Some(Vec4::new(channel(24), channel(16), channel(8), channel(0)))
}

fn parse_color(value: &str) -> Option<Vec4> {
    if let Some(hex) = value.strip_prefix('#') {
        let digits = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
        let channel = |i: usize| digits(i).map(|d| d as f32 / 255.0);
        return match hex.len() {
            6 => Some(Vec4::new(channel(0)?, channel(2)?, channel(4)?, 1.0)),
            8 => Some(Vec4::new(
                channel(0)?,
                channel(2)?,
                channel(4)?,
                channel(6)?,
            )),
            _ => None,
        };
    }
    named_color(value)
}

/// The `<size=...>` value: leading whitespace skipped (observed, `f7515`), then an optional
/// sign and the leading digits (inferred); anything else is 0.
fn parse_size(value: &str) -> i32 {
    let text = value.trim_start_matches([' ', '\t', '\n', '\r', '\u{b}', '\u{c}']);
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.strip_prefix('+').unwrap_or(text)),
    };
    let end = digits
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(digits.len());
    digits[..end].parse::<i32>().map_or(0, |n| sign * n)
}

#[derive(Clone, Debug)]
enum Tag {
    Bold,
    Italic,
    Size(i32),
    Color(Vec4),
    /// `material` and `quad`: accepted and hidden, no visible effect in `OnGUI` text.
    Other,
}

/// Unity's rich-text tag names in its lexer's order (`f20035`, the table at 121500).
const RICH_TAGS: [&str; 10] = [
    "b", "i", "color", "size", "material", "quad", "x", "y", "width", "height",
];
const QUAD: usize = 5;

/// `f20035`: the tag starting with the `<` at `at`, as (kind, closing, index after the name).
/// The name matches ignoring ASCII case and must be followed by `>`, by `=` (opening tags) or,
/// for `quad`, by a space.
fn lex_tag(chars: &[char], at: usize) -> Option<(usize, bool, usize)> {
    if chars.get(at) != Some(&'<') || at + 1 >= chars.len() {
        return None;
    }
    let closing = chars[at + 1] == '/';
    let start = if closing { at + 2 } else { at + 1 };
    for (kind, name) in RICH_TAGS.iter().enumerate() {
        let matches = name
            .chars()
            .enumerate()
            .all(|(k, c)| chars.get(start + k).map(char::to_ascii_lowercase) == Some(c));
        if !matches {
            continue;
        }
        let after = start + name.len();
        let Some(&next) = chars.get(after) else {
            continue;
        };
        if next == '>' || (!closing && next == '=') || (kind == QUAD && next == ' ') {
            return Some((kind, closing, after));
        }
    }
    None
}

/// `f20036`: the value after `=` up to the `>` (or the end of the text); `pos` ends on the `>`.
/// A value of 3+ characters enclosed in matching `'` or `"` loses them.
fn read_tag_value(chars: &[char], pos: &mut usize) -> String {
    let mut value = String::new();
    if chars.get(*pos) == Some(&'=') {
        *pos += 1;
        while *pos < chars.len() && chars[*pos] != '>' {
            value.push(chars[*pos]);
            *pos += 1;
        }
    }
    let inner: Vec<char> = value.chars().collect();
    if inner.len() > 2 && inner[0] == inner[inner.len() - 1] && matches!(inner[0], '\'' | '"') {
        return inner[1..inner.len() - 1].iter().collect();
    }
    value
}

/// One tag `f19944` recorded: the characters `start..=end`, its kind and its effect (`None` for
/// a closing tag).
struct TagSpan {
    start: usize,
    end: usize,
    kind: usize,
    tag: Option<Tag>,
}

/// `f20038`: tags must nest. `x`, `y`, `width` and `height` outside a `quad` count as closing
/// tags that match nothing.
fn tags_nest(spans: &[TagSpan]) -> bool {
    let mut stack = Vec::new();
    for span in spans {
        match (&span.tag, span.kind) {
            // A `quad` records its own closing entry.
            (Some(_), QUAD) => {}
            (Some(_), kind) => stack.push(kind),
            (None, kind) => {
                if kind > QUAD || stack.pop() != Some(kind) {
                    return false;
                }
            }
        }
    }
    stack.is_empty()
}

/// Splits rich text into runs the way Unity's text generator does (`f19944`): every tag is
/// collected first; if they do not nest, none of them applies and the whole text is shown.
/// A colour name Unity does not know (`<color=yelow>` in the spawn-protection label) still makes
/// a valid tag, with the colour white.
fn parse_rich(text: &str, rich: bool, size: i32, bold: bool) -> Vec<Run> {
    let base = Run {
        text: String::new(),
        size,
        bold,
        color: None,
    };
    if !rich {
        return vec![Run {
            text: text.to_string(),
            ..base
        }];
    }
    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut pos = 0;
    while pos < chars.len() {
        if let Some((kind, closing, mut end)) = lex_tag(&chars, pos) {
            let tag = if closing {
                None
            } else {
                match kind {
                    0 => Some(Tag::Bold),
                    1 => Some(Tag::Italic),
                    2 => {
                        let value = read_tag_value(&chars, &mut end);
                        Some(Tag::Color(parse_color(&value).unwrap_or(Vec4::ONE)))
                    }
                    3 => Some(Tag::Size(parse_size(&read_tag_value(&chars, &mut end)))),
                    4 => {
                        read_tag_value(&chars, &mut end);
                        Some(Tag::Other)
                    }
                    QUAD => {
                        // Its attributes (`f20037`) run to the `>`.
                        while end < chars.len() && chars[end] != '>' {
                            end += 1;
                        }
                        Some(Tag::Other)
                    }
                    _ => None,
                }
            };
            spans.push(TagSpan {
                start: pos,
                end,
                kind,
                tag,
            });
            pos = end;
        }
        pos += 1;
    }
    if !tags_nest(&spans) {
        spans.clear();
    }
    let mut runs: Vec<Run> = Vec::new();
    let mut stack: Vec<Tag> = Vec::new();
    let push = |runs: &mut Vec<Run>, stack: &[Tag], c: char| {
        let mut run = base.clone();
        for tag in stack {
            match tag {
                Tag::Bold => run.bold = true,
                // Assumed: a size of 0 or less keeps the enclosing size.
                Tag::Size(s) if *s > 0 => run.size = *s,
                Tag::Color(c) => run.color = Some(*c),
                Tag::Size(_) | Tag::Italic | Tag::Other => {}
            }
        }
        match runs.last_mut() {
            Some(last)
                if last.size == run.size && last.bold == run.bold && last.color == run.color =>
            {
                last.text.push(c)
            }
            _ => {
                run.text.push(c);
                runs.push(run);
            }
        }
    };
    let mut next = spans.iter().peekable();
    let mut i = 0;
    while i < chars.len() {
        if let Some(span) = next.next_if(|s| s.start == i) {
            match (&span.tag, span.kind) {
                (Some(_), QUAD) => {}
                (Some(tag), _) => stack.push(tag.clone()),
                (None, _) => {
                    stack.pop();
                }
            }
            i = span.end + 1;
            continue;
        }
        push(&mut runs, &stack, chars[i]);
        i += 1;
    }
    runs
}

/// Builds one frame's `OnGUI` quads.
pub struct Gui {
    assets: PathBuf,
    pub quads: Vec<GuiQuad>,
    /// `GUI.color`.
    pub color: Vec4,
    texture_sizes: HashMap<String, Option<(u32, u32)>>,
}

impl Gui {
    pub fn new(assets: &Path) -> Gui {
        Gui {
            assets: assets.to_path_buf(),
            quads: Vec::new(),
            color: Vec4::ONE,
            texture_sizes: HashMap::new(),
        }
    }

    /// Start of a MonoBehaviour's `OnGUI`: `GUIUtility` resets `GUI.color` (and the skin, which
    /// the callers pass explicitly).
    pub fn begin_script(&mut self) {
        self.color = Vec4::ONE;
    }

    fn texture_size(&mut self, texture: &str) -> Option<(u32, u32)> {
        if let Some(size) = self.texture_sizes.get(texture) {
            return *size;
        }
        let size = image::image_dimensions(self.assets.join(texture)).ok();
        self.texture_sizes.insert(texture.to_string(), size);
        size
    }

    /// `GUI.DrawTexture(rect, texture)` (`ScaleToFit.StretchToFill`).
    pub fn draw_texture(&mut self, rect: [f32; 4], texture: &str) {
        self.quads.push(GuiQuad::texture(texture, rect, self.color));
    }

    /// A style's background over `rect`, borders unscaled (9-slice).
    fn background(&mut self, rect: [f32; 4], style: &GuiStyle) {
        let Some(texture) = style.background.clone() else {
            return;
        };
        let Some((tw, th)) = self.texture_size(&texture) else {
            return;
        };
        let (tw, th) = (tw as f32, th as f32);
        let [l, r, t, b] = style.border;
        let [x, y, w, h] = rect;
        let xs = [x, x + l, x + w - r, x + w];
        let ys = [y, y + t, y + h - b, y + h];
        let us = [0.0, l / tw, 1.0 - r / tw, 1.0];
        let vs = [0.0, t / th, 1.0 - b / th, 1.0];
        for j in 0..3 {
            for i in 0..3 {
                let (qw, qh) = (xs[i + 1] - xs[i], ys[j + 1] - ys[j]);
                if qw <= 0.0 || qh <= 0.0 {
                    continue;
                }
                self.quads.push(GuiQuad {
                    texture: texture.clone(),
                    rect: [xs[i], ys[j], qw, qh],
                    uv: [us[i], vs[j], us[i + 1], vs[j + 1]],
                    color: self.color,
                });
            }
        }
    }

    /// Width and height of `text` in `style` (`GUIStyle.CalcSize`, single line).
    pub fn calc_size(&mut self, font: &mut GuiFont, text: &str, style: &GuiStyle) -> (f32, f32) {
        let (w, h) = self.text_size(font, text, style);
        let [l, r, t, b] = style.padding;
        (w + l + r, h + t + b)
    }

    fn text_size(&mut self, font: &mut GuiFont, text: &str, style: &GuiStyle) -> (f32, f32) {
        let lines = self.layout_lines(font, text, style, f32::INFINITY);
        let width = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let height = lines.iter().map(|l| l.height).sum();
        (width, height)
    }

    fn layout_lines(
        &mut self,
        font: &mut GuiFont,
        text: &str,
        style: &GuiStyle,
        wrap_width: f32,
    ) -> Vec<Line> {
        let size = if style.font_size > 0 {
            style.font_size
        } else {
            DEFAULT_FONT_SIZE
        };
        let bold = style.font_style == 1 || style.font_style == 3;
        let runs = parse_rich(text, style.rich_text, size, bold);
        let mut lines = vec![Line::default()];
        for run in runs {
            for (k, part) in run.text.split('\n').enumerate() {
                if k > 0 {
                    lines.push(Line::default());
                }
                for c in part.chars() {
                    let Some(glyph) = font.glyph(c, run.size, run.bold) else {
                        continue;
                    };
                    let line = lines.last_mut().unwrap_or_else(|| unreachable!());
                    if style.word_wrap
                        && line.width + glyph.advance > wrap_width
                        && !line.glyphs.is_empty()
                    {
                        lines.push(Line::default());
                    }
                    let line = lines.last_mut().unwrap_or_else(|| unreachable!());
                    line.glyphs.push((line.width, glyph, run.size, run.color));
                    line.width += glyph.advance;
                    line.size = line.size.max(run.size);
                }
            }
        }
        for line in &mut lines {
            if line.size == 0 {
                line.size = size;
            }
            line.height = (line.size as f32 * LINE_SPACING).round();
        }
        lines
    }

    /// Text of `style` inside `rect` (padding applied), aligned by `style.alignment`.
    fn content(&mut self, font: &mut GuiFont, rect: [f32; 4], text: &str, style: &GuiStyle) {
        let [l, r, t, b] = style.padding;
        let inner = [rect[0] + l, rect[1] + t, rect[2] - l - r, rect[3] - t - b];
        let lines = self.layout_lines(font, text, style, inner[2]);
        let total: f32 = lines.iter().map(|l| l.height).sum();
        let (ax, ay) = anchor(style.alignment);
        let mut y = (inner[1] + (inner[3] - total) * ay).round();
        let base_color = style.text_color * self.color;
        let atlas = font.atlas_size as f32;
        for line in &lines {
            let x0 = (inner[0] + (inner[2] - line.width) * ax).round();
            let baseline = y + (line.size as f32 * ASCENT).round();
            for &(pen, glyph, _, color) in &line.glyphs {
                let [gx, gy, gw, gh] = glyph.atlas;
                if gw == 0 {
                    continue;
                }
                let color = color.map_or(base_color, |c| c * self.color);
                let px = x0 + pen + glyph.offset.0 as f32;
                let py = baseline + glyph.offset.1 as f32;
                self.quads.push(GuiQuad {
                    texture: FONT_ATLAS.to_string(),
                    rect: [px, py, gw as f32, gh as f32],
                    uv: [
                        gx as f32 / atlas,
                        gy as f32 / atlas,
                        (gx + gw) as f32 / atlas,
                        (gy + gh) as f32 / atlas,
                    ],
                    color,
                });
            }
            y += line.height;
        }
    }

    /// `GUI.Box(rect, text)` with the skin's box style.
    pub fn box_(&mut self, font: &mut GuiFont, rect: [f32; 4], text: &str, style: &GuiStyle) {
        self.background(rect, style);
        self.content(font, rect, text, style);
    }

    /// `GUI.Label(rect, text, style)`.
    pub fn label(&mut self, font: &mut GuiFont, rect: [f32; 4], text: &str, style: &GuiStyle) {
        self.background(rect, style);
        self.content(font, rect, text, style);
    }

    /// `GUILayout.BeginArea(area)` + `BeginHorizontal()` + one `GUILayout.Box(text,
    /// GUILayout.Height(height))` per item (with its `GUI.color`) + the matching ends. Boxes keep
    /// their style margins (adjacent margins collapse to the larger) and the stretchable ones
    /// share the width left over after their content size.
    pub fn horizontal_boxes(
        &mut self,
        font: &mut GuiFont,
        area: [f32; 4],
        items: &[(String, Vec4)],
        style: &GuiStyle,
        height: f32,
    ) {
        if items.is_empty() {
            return;
        }
        let [ml, mr, mt, _] = style.margin;
        let widths: Vec<f32> = items
            .iter()
            .map(|(text, _)| self.calc_size(font, text, style).0)
            .collect();
        let gaps = ml + mr + ml.max(mr) * (items.len() - 1) as f32;
        let min: f32 = widths.iter().sum();
        let extra = if style.stretch_width {
            ((area[2] - gaps - min) / items.len() as f32).max(0.0)
        } else {
            0.0
        };
        let mut x = area[0] + ml;
        for ((text, color), width) in items.iter().zip(widths) {
            let w = width + extra;
            self.color = *color;
            self.box_(font, [x, area[1] + mt, w, height], text, style);
            x += w + ml.max(mr);
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Line {
    /// (pen x, glyph, size, rich-text colour).
    glyphs: Vec<(f32, Glyph, i32, Option<Vec4>)>,
    width: f32,
    height: f32,
    size: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    #[test]
    fn rich_text_tags_and_invalid_values() {
        let runs = parse_rich("<size=32><color=red>A</color></size>B", true, 13, false);
        assert_eq!(runs.len(), 2);
        assert_eq!((runs[0].size, runs[0].color), (32, named_color("red")));
        assert_eq!((runs[1].text.as_str(), runs[1].size), ("B", 13));
        // An unknown colour name is still a tag, coloured white (Unity's f11465 leaves -1).
        let runs = parse_rich("<size=32><color=yelow>>X</color></size>", true, 13, false);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, ">X");
        assert_eq!((runs[0].size, runs[0].color), (32, Some(Vec4::ONE)));
        // Tags that do not nest: none applies.
        let text = "<b>x<i>y</b></i>";
        assert_eq!(parse_rich(text, true, 13, false)[0].text, text);
        assert_eq!(parse_rich("<b>open", true, 13, false)[0].text, "<b>open");
        // Names ignore case; quoted values lose their quotes; unknown tags are text.
        let runs = parse_rich("<COLOR=\"Red\">a</Color><u>b</u>", true, 13, false);
        assert_eq!(
            (runs[0].text.as_str(), runs[0].color),
            ("a", named_color("red"))
        );
        assert_eq!(runs[1].text, "<u>b</u>");
        assert_eq!(named_color("transparent"), Some(Vec4::ZERO));
        // Rich text off: everything is text.
        assert_eq!(parse_rich("<b>x</b>", false, 13, false)[0].text, "<b>x</b>");
    }

    #[test]
    fn skins_and_font_load() {
        let assets = find_assets_dir().unwrap();
        let skin = load_skin(&assets, "resources.assets:40959").unwrap();
        assert_eq!(skin.box_style.font_size, 22);
        assert_eq!(skin.box_style.alignment, 4);
        assert_eq!(skin.box_style.border, [6.0; 4]);
        assert_eq!(
            skin.box_style.background.as_deref(),
            Some("textures/PartyUnitFrame_Avatar_Background.png")
        );
        let default = load_skin(&assets, DEFAULT_SKIN).unwrap();
        assert_eq!(default.label.font_size, 0);
        let mut font = GuiFont::load(&assets).unwrap();
        assert!(font.has_character('7') && font.has_character('|'));
        // The knife's "∞" is not in the embedded font subset.
        assert!(!font.has_character('\u{221e}'));
        let mut gui = Gui::new(&assets);
        let (w, h) = gui.calc_size(&mut font, "30 | 4", &skin.box_style);
        assert!(w > 50.0 && w < 90.0, "{w}");
        assert_eq!(h, 25.0 + 8.0);
        gui.box_(
            &mut font,
            [10.0, 10.0, 100.0, 45.0],
            "30 | 4",
            &skin.box_style,
        );
        // 9 background slices + 4 visible glyphs (spaces draw nothing).
        assert_eq!(gui.quads.len(), 9 + 4);
    }
}
