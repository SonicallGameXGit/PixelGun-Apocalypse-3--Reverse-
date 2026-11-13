//! `GUILayout` widgets recorded for one frame, laid out by `layout.rs`, then drawn and hit-tested
//! (the lobby in `menu.rs` and the in-room menu in `room_menu.rs`; spec:
//! `docs/gameplay/menu.md`, "IMGUI layout", and `docs/gameplay/settings.md`).
//!
//! `A` is what a button does: the caller's action type.

use glam::{Vec2, Vec4};

use crate::gui::{ACTIVE, FOCUSED, Gui, GuiFont, GuiQuad, GuiStyle, HOVER, NamedSkin, ON_NORMAL};
use crate::layout::{Layout, LeafSize, Offsets};

/// Width of a vertical scroll bar when the skin's style has no fixed width.
const SCROLLBAR_WIDTH: f32 = 55.0;

pub enum WKind<A> {
    Label,
    Box,
    Button(A),
    /// A text field: the action that focuses it and its maximum length.
    Field(A, usize),
    Texture(String),
    /// A styled `BeginHorizontal(style)`: draws its background.
    Group,
    ScrollBegin(usize),
    ScrollEnd,
}

pub struct Widget<A> {
    pub entry: usize,
    pub kind: WKind<A>,
    pub text: String,
    pub style: String,
    /// `GUI.color` when the widget was called.
    pub color: Vec4,
    /// `GUI.enabled` when the widget was called.
    pub enabled: bool,
}

pub struct Frame<A> {
    pub layout: Layout,
    pub widgets: Vec<Widget<A>>,
    /// `(view entry, content entry)` of every scroll view, in order.
    pub scrolls: Vec<(usize, usize)>,
}

/// Records one frame of `GUILayout` calls.
pub struct Builder<'a, A> {
    pub layout: Layout,
    pub widgets: Vec<Widget<A>>,
    pub scrolls: Vec<(usize, usize)>,
    pub gui: &'a mut Gui,
    pub font: &'a mut GuiFont,
    pub skin: &'a NamedSkin,
    /// `GUI.color`.
    pub color: Vec4,
    /// `GUI.enabled`.
    pub enabled: bool,
}

impl<'a, A> Builder<'a, A> {
    /// `BeginArea(area)` / a window's client area with the style's `padding`.
    pub fn new(
        gui: &'a mut Gui,
        font: &'a mut GuiFont,
        skin: &'a NamedSkin,
        area: [f32; 4],
        padding: Offsets,
    ) -> Builder<'a, A> {
        Builder {
            layout: Layout::new_area(area, padding),
            widgets: Vec::new(),
            scrolls: Vec::new(),
            gui,
            font,
            skin,
            color: Vec4::ONE,
            enabled: true,
        }
    }

    /// Lays the recorded calls out.
    pub fn finish(self) -> Frame<A> {
        let mut layout = self.layout;
        layout.compute();
        Frame {
            layout,
            widgets: self.widgets,
            scrolls: self.scrolls,
        }
    }

    fn push(&mut self, entry: usize, kind: WKind<A>, text: &str, style: &str) {
        self.widgets.push(Widget {
            entry,
            kind,
            text: text.to_string(),
            style: style.to_string(),
            color: self.color,
            enabled: self.enabled,
        });
    }

    pub fn leaf(
        &mut self,
        kind: WKind<A>,
        text: &str,
        style: &str,
        width: Option<f32>,
        height: Option<f32>,
    ) -> usize {
        let s = self.skin.style(style);
        let size = self.gui.calc_size(self.font, text, &s);
        let entry = self.layout.leaf(LeafSize {
            size,
            margin: s.margin,
            stretch_w: s.stretch_width && s.fixed_width == 0.0,
            stretch_h: s.stretch_height && s.fixed_height == 0.0,
            width,
            height,
        });
        self.push(entry, kind, text, style);
        entry
    }

    /// A texture drawn over the last widget (the lobby's lock icon).
    pub fn overlay_texture(&mut self, texture: &str) {
        let entry = self.widgets.last().map_or(0, |w| w.entry);
        self.widgets.push(Widget {
            entry,
            kind: WKind::Texture(texture.to_string()),
            text: String::new(),
            style: "none".to_string(),
            color: Vec4::ONE,
            enabled: self.enabled,
        });
    }

    pub fn label(&mut self, text: &str, width: Option<f32>, height: Option<f32>) {
        self.leaf(WKind::Label, text, "label", width, height);
    }

    pub fn boxed(&mut self, text: &str, width: Option<f32>, height: Option<f32>) {
        self.leaf(WKind::Box, text, "box", width, height);
    }

    pub fn button(&mut self, text: &str, action: A, width: Option<f32>, height: Option<f32>) {
        self.leaf(WKind::Button(action), text, "button", width, height);
    }

    pub fn field(&mut self, focus: A, text: &str, max: usize, width: Option<f32>, height: f32) {
        self.leaf(
            WKind::Field(focus, max),
            text,
            "textfield",
            width,
            Some(height),
        );
    }

    pub fn horizontal(&mut self) {
        self.layout
            .begin_group(false, false, [0.0; 4], [0.0; 4], None, None);
    }

    pub fn vertical(&mut self) {
        self.layout
            .begin_group(true, false, [0.0; 4], [0.0; 4], None, None);
    }

    /// `BeginHorizontal(style, options)`.
    pub fn horizontal_styled(&mut self, style: &str, width: Option<f32>, height: Option<f32>) {
        let s = self.skin.style(style);
        let entry = self
            .layout
            .begin_group(false, true, s.padding, s.margin, width, height);
        self.push(entry, WKind::Group, "", style);
    }

    /// `BeginHorizontal("box")`.
    pub fn horizontal_box(&mut self) {
        self.horizontal_styled("box", None, None);
    }

    pub fn end(&mut self) {
        self.layout.end_group();
    }

    pub fn space(&mut self, pixels: f32) {
        self.layout.space(pixels);
    }

    pub fn flexible_space(&mut self) {
        self.layout.flexible_space();
    }

    /// `BeginScrollView` of the given size: the view group and, inside it, the content group
    /// (`content_width` is the view's minus the scroll bar's, when there is one).
    pub fn begin_scroll(&mut self, view: (f32, f32), content_width: f32) {
        let outer =
            self.layout
                .begin_group(true, false, [0.0; 4], [0.0; 4], Some(view.0), Some(view.1));
        let inner =
            self.layout
                .begin_group(true, false, [0.0; 4], [0.0; 4], Some(content_width), None);
        self.scrolls.push((outer, inner));
        let index = self.scrolls.len() - 1;
        self.push(outer, WKind::ScrollBegin(index), "", "");
    }

    pub fn end_scroll(&mut self) {
        self.layout.end_group();
        self.layout.end_group();
        self.push(0, WKind::ScrollEnd, "", "");
    }
}

/// The width of the skin's vertical scroll bar.
pub fn scrollbar_width(skin: &NamedSkin) -> f32 {
    let width = skin.style("verticalscrollbar").fixed_width;
    if width > 0.0 { width } else { SCROLLBAR_WIDTH }
}

pub fn contains(rect: [f32; 4], p: Vec2) -> bool {
    p.x >= rect[0] && p.x < rect[0] + rect[2] && p.y >= rect[1] && p.y < rect[1] + rect[3]
}

impl<A> Frame<A> {
    /// Whether scroll view `index` has more content than fits (a scroll bar shows).
    pub fn overflows(&self, index: usize) -> bool {
        let (view, content) = self.scrolls[index];
        self.layout.content_height(content) > self.layout.rect(view)[3] + 0.5
    }

    pub fn any_overflow(&self) -> bool {
        (0..self.scrolls.len()).any(|i| self.overflows(i))
    }

    /// Mouse wheel over a scroll view moves it; offsets stay within the content.
    pub fn scroll(&self, offsets: &mut [f32], mouse: Vec2, wheel: f32, step: f32) {
        for (i, &(view, content)) in self.scrolls.iter().enumerate() {
            let Some(offset) = offsets.get_mut(i) else {
                continue;
            };
            let rect = self.layout.rect(view);
            let max = (self.layout.content_height(content) - rect[3]).max(0.0);
            if contains(rect, mouse) {
                *offset -= wheel * step;
            }
            *offset = offset.clamp(0.0, max);
        }
    }

    /// Where widget `entry` is on screen inside scroll view `scroll` (`None`: scrolled out).
    fn visible_rect(
        &self,
        entry: usize,
        scroll: Option<usize>,
        offsets: &[f32],
    ) -> Option<[f32; 4]> {
        let mut rect = self.layout.rect(entry);
        if let Some(i) = scroll {
            rect[1] -= offsets.get(i).copied().unwrap_or(0.0);
            let clip = self.layout.rect(self.scrolls[i].0);
            if !contains(
                clip,
                Vec2::new(rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0),
            ) {
                // Mostly outside the view: only the visible part can be hit.
                let top = rect[1].max(clip[1]);
                let bottom = (rect[1] + rect[3]).min(clip[1] + clip[3]);
                if bottom <= top {
                    return None;
                }
                rect[1] = top;
                rect[3] = bottom - top;
            }
        }
        Some(rect)
    }

    /// The enabled button or field under the mouse (later widgets win, as drawn on top).
    pub fn hit(&self, offsets: &[f32], mouse: Vec2) -> Option<&A> {
        let mut scroll_index = None;
        let mut hit = None;
        for w in &self.widgets {
            match &w.kind {
                WKind::ScrollBegin(i) => scroll_index = Some(*i),
                WKind::ScrollEnd => scroll_index = None,
                WKind::Button(action) | WKind::Field(action, _) if w.enabled => {
                    if let Some(rect) = self.visible_rect(w.entry, scroll_index, offsets)
                        && contains(rect, mouse)
                    {
                        hit = Some(action);
                    }
                }
                _ => {}
            }
        }
        hit
    }

    /// The maximum length of the field `focus` focuses.
    pub fn field_max(&self, focus: &A) -> Option<usize>
    where
        A: PartialEq,
    {
        self.widgets.iter().find_map(|w| match &w.kind {
            WKind::Field(f, max) if f == focus => Some(*max),
            _ => None,
        })
    }
}

/// What drawing needs besides the frame.
pub struct DrawState<'s, A> {
    pub mouse: Vec2,
    /// The button the mouse went down on.
    pub pressed: Option<&'s A>,
    pub focus: Option<&'s A>,
    /// The caret shows while `blink` is in the first half of each second.
    pub blink: f32,
    pub offsets: &'s [f32],
    /// Multiplies every widget's alpha (the lobby while connecting).
    pub alpha: f32,
    /// Hover and press states show (false: the whole window is disabled).
    pub interactive: bool,
}

/// `GUI.enabled = false` draws at half alpha and without hover (inferred, see
/// `docs/gameplay/settings.md`).
const DISABLED_ALPHA: f32 = 0.5;

/// Draws the frame's widgets in call order; scroll views clip their content and get a bar when
/// it overflows.
pub fn draw<A: PartialEq>(
    gui: &mut Gui,
    font: &mut GuiFont,
    skin: &NamedSkin,
    frame: &Frame<A>,
    state: &DrawState<A>,
) {
    let mut clip: Option<(usize, usize)> = None;
    for w in &frame.widgets {
        let mut rect = frame.layout.rect(w.entry);
        if let Some((i, _)) = clip {
            rect[1] -= state.offsets.get(i).copied().unwrap_or(0.0);
        }
        let mut tint = w.color;
        tint.w *= state.alpha;
        if !w.enabled {
            tint.w *= DISABLED_ALPHA;
        }
        gui.color = tint;
        let style = skin.style(&w.style);
        match &w.kind {
            WKind::Label => gui.label(font, rect, &w.text, &style),
            WKind::Box => gui.box_(font, rect, &w.text, &style),
            WKind::Group => gui.styled(font, rect, "", &style, None),
            WKind::Texture(texture) => {
                gui.label(font, rect, "", &style);
                gui.draw_texture(rect, texture);
            }
            WKind::Button(action) => {
                let inside = clip.is_none_or(|(i, _)| {
                    contains(frame.layout.rect(frame.scrolls[i].0), state.mouse)
                });
                let over = state.interactive && w.enabled && inside && contains(rect, state.mouse);
                let button_state = if over && state.pressed == Some(action) {
                    Some(ACTIVE)
                } else if over {
                    Some(HOVER)
                } else {
                    None
                };
                gui.styled(font, rect, &w.text, &style, button_state);
            }
            WKind::Field(focus, _) => {
                let focused = state.focus == Some(focus);
                let mut text = w.text.clone();
                if focused && (state.blink * 2.0) as i32 % 2 == 0 {
                    text.push('|');
                }
                let field_state = if focused { Some(FOCUSED) } else { None };
                gui.styled(font, rect, &text, &style, field_state);
            }
            WKind::ScrollBegin(i) => clip = Some((*i, gui.quads.len())),
            WKind::ScrollEnd => {
                if let Some((i, from)) = clip.take() {
                    let view = frame.layout.rect(frame.scrolls[i].0);
                    clip_quads(&mut gui.quads, from, view);
                    gui.color = Vec4::new(1.0, 1.0, 1.0, state.alpha);
                    draw_scrollbar(gui, font, skin, frame, i, state.offsets);
                }
            }
        }
    }
}

fn draw_scrollbar<A>(
    gui: &mut Gui,
    font: &mut GuiFont,
    skin: &NamedSkin,
    frame: &Frame<A>,
    index: usize,
    offsets: &[f32],
) {
    if !frame.overflows(index) {
        return;
    }
    let (view, content) = frame.scrolls[index];
    let clip = frame.layout.rect(view);
    let total = frame.layout.content_height(content);
    let width = scrollbar_width(skin);
    let track = [clip[0] + clip[2] - width, clip[1], width, clip[3]];
    let bar = skin.style("verticalscrollbar");
    let thumb = skin.style("verticalscrollbarthumb");
    gui.styled(font, track, "", &bar, None);
    let ratio = clip[3] / total;
    let height = (track[3] * ratio).max(20.0);
    let offset = offsets.get(index).copied().unwrap_or(0.0);
    let top = track[1] + (track[3] - height) * (offset / (total - clip[3]));
    gui.styled(font, [track[0], top, width, height], "", &thumb, None);
}

/// Clips the quads from `from` on to `clip`, cutting their UVs proportionally.
pub fn clip_quads(quads: &mut Vec<GuiQuad>, from: usize, clip: [f32; 4]) {
    let mut kept: Vec<GuiQuad> = Vec::new();
    for q in quads.drain(from..) {
        let [x, y, w, h] = q.rect;
        let (x0, y0) = (x.max(clip[0]), y.max(clip[1]));
        let (x1, y1) = (
            (x + w).min(clip[0] + clip[2]),
            (y + h).min(clip[1] + clip[3]),
        );
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        let [u0, v0, u1, v1] = q.uv;
        let (fx0, fx1) = ((x0 - x) / w, (x1 - x) / w);
        let (fy0, fy1) = ((y0 - y) / h, (y1 - y) / h);
        kept.push(GuiQuad {
            rect: [x0, y0, x1 - x0, y1 - y0],
            uv: [
                u0 + (u1 - u0) * fx0,
                v0 + (v1 - v0) * fy0,
                u0 + (u1 - u0) * fx1,
                v0 + (v1 - v0) * fy1,
            ],
            ..q
        });
    }
    quads.extend(kept);
}

/// `GUI.Window(id, area, func, title)`'s frame (also `GUILayout.BeginArea(area, title, style)`):
/// `GUIStyle.Draw` puts the background over the area grown by the style's overflow and the title
/// inside the area less the padding; `focused` draws the `onNormal` state (a window the script
/// focuses with `GUI.FocusWindow`).
pub fn window_frame(
    gui: &mut Gui,
    font: &mut GuiFont,
    style: &GuiStyle,
    area: [f32; 4],
    title: &str,
    focused: bool,
) {
    let [ol, or, ot, ob] = style.overflow;
    let outer = [
        area[0] - ol,
        area[1] - ot,
        area[2] + ol + or,
        area[3] + ot + ob,
    ];
    let state = if focused { Some(ON_NORMAL) } else { None };
    gui.styled(font, outer, "", style, state);
    let mut text_style = style.clone();
    text_style.background = None;
    for s in &mut text_style.states {
        s.0 = None;
    }
    gui.styled(font, area, title, &text_style, state);
}
