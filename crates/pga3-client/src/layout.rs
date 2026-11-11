//! Unity's `GUILayout` engine (the part the lobby and the room menu use), so that rows, gaps and
//! stretching come out where the original puts them (spec: `docs/gameplay/menu.md`, "IMGUI
//! layout").
//!
//! The scripts call `BeginHorizontal` / `Label` / `FlexibleSpace` ... in order. Here a frame's
//! calls are recorded as a tree of entries, laid out with the algorithm of Unity's
//! `GUILayoutGroup` (`CalcWidth`, `CalcHeight`, `SetHorizontal`, `SetVertical`) and then
//! replayed against the resulting rectangles.
//!
//! - A leaf has a minimum and maximum size (the content size, or a `GUILayout.Width` /
//!   `Height` option) and a stretch flag (from the style: `stretchWidth` and a `fixedWidth` of
//!   0; `FlexibleSpace` always stretches).
//! - Groups collapse the margins of neighbours to the larger one, and a group with a style (or a
//!   user-specified size) turns its children's margins into padding.
//! - Along a group's axis, children get a size between their minimum and maximum by a common
//!   factor `t`; once the space exceeds every maximum, the stretchable children share the rest.

/// A margin or padding: left, right, top, bottom.
pub type Offsets = [f32; 4];

#[derive(Clone, Debug)]
pub enum Kind {
    Leaf,
    Group {
        vertical: bool,
    },
    /// `GUILayout.Space` / `FlexibleSpace`: no margins, ignored by margin collapsing.
    Space,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub kind: Kind,
    pub children: Vec<usize>,
    pub min_w: f32,
    pub max_w: f32,
    pub min_h: f32,
    pub max_h: f32,
    pub stretch_w: i32,
    pub stretch_h: i32,
    pub margin: Offsets,
    /// The style's padding (groups with a style only).
    pub padding: Offsets,
    /// A group whose style is not `GUIStyle.none`, or whose size the script specified.
    pub styled: bool,
    /// `GUILayout.Width` / `Height` on a group: its size is exactly that whatever its children ask.
    user_w: Option<f32>,
    user_h: Option<f32>,
    pub rect: [f32; 4],
    child_min_w: f32,
    child_max_w: f32,
    child_min_h: f32,
    child_max_h: f32,
    stretch_count_x: i32,
    stretch_count_y: i32,
}

impl Entry {
    fn new(kind: Kind) -> Entry {
        Entry {
            kind,
            children: Vec::new(),
            min_w: 0.0,
            max_w: 0.0,
            min_h: 0.0,
            max_h: 0.0,
            stretch_w: 0,
            stretch_h: 0,
            margin: [0.0; 4],
            padding: [0.0; 4],
            styled: false,
            user_w: None,
            user_h: None,
            rect: [0.0; 4],
            child_min_w: 0.0,
            child_max_w: 0.0,
            child_min_h: 0.0,
            child_max_h: 0.0,
            stretch_count_x: 0,
            stretch_count_y: 0,
        }
    }
}

/// What a leaf asks for.
#[derive(Clone, Copy, Debug, Default)]
pub struct LeafSize {
    /// The content size (`GUIStyle.CalcSize`, padding included).
    pub size: (f32, f32),
    pub margin: Offsets,
    /// `style.stretchWidth && style.fixedWidth == 0`.
    pub stretch_w: bool,
    pub stretch_h: bool,
    /// `GUILayout.Width` / `Height` options.
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Default)]
pub struct Layout {
    pub entries: Vec<Entry>,
    stack: Vec<usize>,
}

impl Layout {
    /// A top-level group (`BeginArea`): `rect` is its fixed rectangle, `padding` its style's.
    pub fn new_area(rect: [f32; 4], padding: Offsets) -> Layout {
        let mut area = Entry::new(Kind::Group { vertical: true });
        area.styled = true;
        area.padding = padding;
        area.rect = rect;
        Layout {
            entries: vec![area],
            stack: vec![0],
        }
    }

    fn push(&mut self, entry: Entry) -> usize {
        let index = self.entries.len();
        self.entries.push(entry);
        if let Some(&parent) = self.stack.last() {
            self.entries[parent].children.push(index);
        }
        index
    }

    /// `BeginHorizontal` / `BeginVertical`. `padding` and `margin` are the group style's (zero
    /// for no style); `width` / `height` are the options.
    pub fn begin_group(
        &mut self,
        vertical: bool,
        styled: bool,
        padding: Offsets,
        margin: Offsets,
        width: Option<f32>,
        height: Option<f32>,
    ) -> usize {
        let mut entry = Entry::new(Kind::Group { vertical });
        entry.styled = styled || width.is_some() || height.is_some();
        entry.padding = padding;
        if styled {
            entry.margin = margin;
        }
        if let Some(w) = width {
            entry.min_w = w;
            entry.max_w = w;
            entry.user_w = Some(w);
        }
        if let Some(h) = height {
            entry.min_h = h;
            entry.max_h = h;
            entry.user_h = Some(h);
        }
        let index = self.push(entry);
        self.stack.push(index);
        index
    }

    pub fn end_group(&mut self) {
        self.stack.pop();
    }

    /// `GUILayout.Space(pixels)`: along the parent's axis.
    pub fn space(&mut self, pixels: f32) -> usize {
        let mut entry = Entry::new(Kind::Space);
        entry.min_w = pixels;
        entry.max_w = pixels;
        entry.min_h = pixels;
        entry.max_h = pixels;
        self.push(entry)
    }

    /// `GUILayout.FlexibleSpace()`: stretches along the parent's axis only.
    pub fn flexible_space(&mut self) -> usize {
        let vertical = self
            .stack
            .last()
            .is_some_and(|&p| matches!(self.entries[p].kind, Kind::Group { vertical: true }));
        let mut entry = Entry::new(Kind::Space);
        if vertical {
            entry.stretch_h = 1;
        } else {
            entry.stretch_w = 1;
        }
        self.push(entry)
    }

    pub fn leaf(&mut self, leaf: LeafSize) -> usize {
        let mut entry = Entry::new(Kind::Leaf);
        entry.margin = leaf.margin;
        entry.min_w = leaf.size.0;
        entry.max_w = leaf.size.0;
        entry.min_h = leaf.size.1;
        entry.max_h = leaf.size.1;
        entry.stretch_w = leaf.stretch_w as i32;
        entry.stretch_h = leaf.stretch_h as i32;
        if let Some(w) = leaf.width {
            entry.min_w = w;
            entry.max_w = w;
            entry.stretch_w = 0;
        }
        if let Some(h) = leaf.height {
            entry.min_h = h;
            entry.max_h = h;
            entry.stretch_h = 0;
        }
        self.push(entry)
    }

    /// Lays the whole tree out inside the area's rectangle.
    pub fn compute(&mut self) {
        let [x, y, w, h] = self.entries[0].rect;
        self.calc_width(0);
        self.calc_height(0);
        self.set_horizontal(0, x, w);
        self.set_vertical(0, y, h);
    }

    fn is_space(&self, i: usize) -> bool {
        matches!(self.entries[i].kind, Kind::Space)
    }

    fn calc_width(&mut self, i: usize) {
        let children = self.entries[i].children.clone();
        for &c in &children {
            if matches!(self.entries[c].kind, Kind::Group { .. }) {
                self.calc_width(c);
            }
        }
        let Kind::Group { vertical } = self.entries[i].kind else {
            return;
        };
        let (mut min_w, mut max_w) = (0.0f32, 0.0f32);
        let mut stretch = 0;
        let (mut first_m, mut last_m) = (0.0f32, 0.0f32);
        let mut first = true;
        if vertical {
            for &c in &children {
                let e = &self.entries[c];
                if !self.is_space(c) {
                    let [ml, mr, _, _] = e.margin;
                    if first {
                        first_m = ml;
                        last_m = mr;
                        first = false;
                    } else {
                        first_m = first_m.min(ml);
                        last_m = last_m.min(mr);
                    }
                    min_w = min_w.max(e.min_w + ml + mr);
                    max_w = max_w.max(e.max_w + ml + mr);
                }
                stretch += e.stretch_w;
            }
            min_w -= first_m + last_m;
            max_w -= first_m + last_m;
        } else {
            let mut prev_right = 0.0f32;
            for &c in &children {
                let e = &self.entries[c];
                if !self.is_space(c) {
                    let gap = if first {
                        first = false;
                        0.0
                    } else {
                        prev_right.max(e.margin[0])
                    };
                    min_w += e.min_w + gap;
                    max_w += e.max_w + gap;
                    prev_right = e.margin[1];
                } else {
                    min_w += e.min_w;
                    max_w += e.max_w;
                }
                stretch += e.stretch_w;
            }
            if let (Some(&f), Some(&l)) = (children.first(), children.last()) {
                first_m = self.entries[f].margin[0];
                last_m = self.entries[l].margin[1];
            }
        }
        let e = &mut self.entries[i];
        let (pad_l, pad_r) = if e.styled {
            (e.padding[0].max(first_m), e.padding[1].max(last_m))
        } else {
            e.margin[0] = first_m;
            e.margin[1] = last_m;
            (0.0, 0.0)
        };
        e.child_min_w = min_w;
        e.child_max_w = max_w;
        e.stretch_count_x = stretch;
        e.min_w = e.min_w.max(min_w + pad_l + pad_r);
        // A user-specified width fixes the maximum as well; otherwise the children's maximum.
        e.max_w = if e.max_w > 0.0 && e.max_w == e.min_w {
            e.max_w
        } else {
            e.max_w.max(max_w + pad_l + pad_r).max(e.min_w)
        };
        e.stretch_w = if e.styled && e.max_w == e.min_w && e.max_w > 0.0 && stretch == 0 {
            0
        } else {
            stretch.min(1)
        };
        if let Some(w) = e.user_w {
            e.min_w = w;
            e.max_w = w;
            e.stretch_w = 0;
        }
    }

    fn calc_height(&mut self, i: usize) {
        let children = self.entries[i].children.clone();
        for &c in &children {
            if matches!(self.entries[c].kind, Kind::Group { .. }) {
                self.calc_height(c);
            }
        }
        let Kind::Group { vertical } = self.entries[i].kind else {
            return;
        };
        let (mut min_h, mut max_h) = (0.0f32, 0.0f32);
        let mut stretch = 0;
        let (mut first_m, mut last_m) = (0.0f32, 0.0f32);
        let mut first = true;
        if !vertical {
            for &c in &children {
                let e = &self.entries[c];
                if !self.is_space(c) {
                    let [_, _, mt, mb] = e.margin;
                    if first {
                        first_m = mt;
                        last_m = mb;
                        first = false;
                    } else {
                        first_m = first_m.min(mt);
                        last_m = last_m.min(mb);
                    }
                    min_h = min_h.max(e.min_h + mt + mb);
                    max_h = max_h.max(e.max_h + mt + mb);
                }
                stretch += e.stretch_h;
            }
            min_h -= first_m + last_m;
            max_h -= first_m + last_m;
        } else {
            let mut prev_bottom = 0.0f32;
            for &c in &children {
                let e = &self.entries[c];
                if !self.is_space(c) {
                    let gap = if first {
                        first = false;
                        0.0
                    } else {
                        prev_bottom.max(e.margin[2])
                    };
                    min_h += e.min_h + gap;
                    max_h += e.max_h + gap;
                    prev_bottom = e.margin[3];
                } else {
                    min_h += e.min_h;
                    max_h += e.max_h;
                }
                stretch += e.stretch_h;
            }
            if let (Some(&f), Some(&l)) = (children.first(), children.last()) {
                first_m = self.entries[f].margin[2];
                last_m = self.entries[l].margin[3];
            }
        }
        let e = &mut self.entries[i];
        let (pad_t, pad_b) = if e.styled {
            (e.padding[2].max(first_m), e.padding[3].max(last_m))
        } else {
            e.margin[2] = first_m;
            e.margin[3] = last_m;
            (0.0, 0.0)
        };
        e.child_min_h = min_h;
        e.child_max_h = max_h;
        e.stretch_count_y = stretch;
        e.min_h = e.min_h.max(min_h + pad_t + pad_b);
        e.max_h = if e.max_h > 0.0 && e.max_h == e.min_h {
            e.max_h
        } else {
            e.max_h.max(max_h + pad_t + pad_b).max(e.min_h)
        };
        e.stretch_h = if e.styled && e.max_h == e.min_h && e.max_h > 0.0 && stretch == 0 {
            0
        } else {
            stretch.min(1)
        };
        if let Some(h) = e.user_h {
            e.min_h = h;
            e.max_h = h;
            e.stretch_h = 0;
        }
    }

    fn set_horizontal(&mut self, i: usize, x: f32, width: f32) {
        self.entries[i].rect[0] = x;
        self.entries[i].rect[2] = width;
        let Kind::Group { vertical } = self.entries[i].kind else {
            return;
        };
        let children = self.entries[i].children.clone();
        let styled = self.entries[i].styled;
        let padding = self.entries[i].padding;
        let margin = self.entries[i].margin;
        if vertical {
            if styled {
                for &c in &children {
                    let e = &self.entries[c];
                    let left = e.margin[0].max(padding[0]);
                    let w = width - e.margin[1].max(padding[1]) - left;
                    let (stretch, min, max) = (e.stretch_w, e.min_w, e.max_w);
                    let w = if stretch != 0 {
                        w
                    } else {
                        w.clamp(min, max.max(min))
                    };
                    self.set_horizontal(c, x + left, w);
                }
            } else {
                let x0 = x - margin[0];
                let w0 = width + margin[0] + margin[1];
                for &c in &children {
                    let e = &self.entries[c];
                    let (ml, mr) = (e.margin[0], e.margin[1]);
                    let (stretch, min, max) = (e.stretch_w, e.min_w, e.max_w);
                    let w = w0 - ml - mr;
                    let w = if stretch != 0 {
                        w
                    } else {
                        w.clamp(min, max.max(min))
                    };
                    self.set_horizontal(c, x0 + ml, w);
                }
            }
        } else {
            let (mut x, mut width) = (x, width);
            if styled {
                let mut pl = padding[0];
                let mut pr = padding[1];
                if let (Some(&f), Some(&l)) = (children.first(), children.last()) {
                    pl = pl.max(self.entries[f].margin[0]);
                    pr = pr.max(self.entries[l].margin[1]);
                }
                x += pl;
                width -= pl + pr;
            }
            let (cmin, cmax, stretch_count) = {
                let e = &self.entries[i];
                (e.child_min_w, e.child_max_w, e.stretch_count_x)
            };
            let t = if cmin != cmax {
                ((width - cmin) / (cmax - cmin)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let extra = if width > cmax && stretch_count > 0 {
                (width - cmax) / stretch_count as f32
            } else {
                0.0
            };
            let mut prev_right = 0.0f32;
            let mut first = true;
            for &c in &children {
                let e = &self.entries[c];
                let mut w = e.min_w + (e.max_w - e.min_w) * t;
                w += extra * e.stretch_w as f32;
                if !self.is_space(c) {
                    let left = if first {
                        first = false;
                        0.0
                    } else {
                        e.margin[0]
                    };
                    x += prev_right.max(left);
                    prev_right = e.margin[1];
                }
                self.set_horizontal(c, x.round(), w.round());
                x += w;
            }
        }
    }

    fn set_vertical(&mut self, i: usize, y: f32, height: f32) {
        self.entries[i].rect[1] = y;
        self.entries[i].rect[3] = height;
        let Kind::Group { vertical } = self.entries[i].kind else {
            return;
        };
        let children = self.entries[i].children.clone();
        let styled = self.entries[i].styled;
        let padding = self.entries[i].padding;
        let margin = self.entries[i].margin;
        if !vertical {
            if styled {
                for &c in &children {
                    let e = &self.entries[c];
                    let top = e.margin[2].max(padding[2]);
                    let h = height - e.margin[3].max(padding[3]) - top;
                    let (stretch, min, max) = (e.stretch_h, e.min_h, e.max_h);
                    let h = if stretch != 0 {
                        h
                    } else {
                        h.clamp(min, max.max(min))
                    };
                    self.set_vertical(c, y + top, h);
                }
            } else {
                let y0 = y - margin[2];
                let h0 = height + margin[2] + margin[3];
                for &c in &children {
                    let e = &self.entries[c];
                    let (mt, mb) = (e.margin[2], e.margin[3]);
                    let (stretch, min, max) = (e.stretch_h, e.min_h, e.max_h);
                    let h = h0 - mt - mb;
                    let h = if stretch != 0 {
                        h
                    } else {
                        h.clamp(min, max.max(min))
                    };
                    self.set_vertical(c, y0 + mt, h);
                }
            }
        } else {
            let (mut y, mut height) = (y, height);
            if styled {
                let mut pt = padding[2];
                let mut pb = padding[3];
                if let (Some(&f), Some(&l)) = (children.first(), children.last()) {
                    pt = pt.max(self.entries[f].margin[2]);
                    pb = pb.max(self.entries[l].margin[3]);
                }
                y += pt;
                height -= pt + pb;
            }
            let (cmin, cmax, stretch_count) = {
                let e = &self.entries[i];
                (e.child_min_h, e.child_max_h, e.stretch_count_y)
            };
            let t = if cmin != cmax {
                ((height - cmin) / (cmax - cmin)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let extra = if height > cmax && stretch_count > 0 {
                (height - cmax) / stretch_count as f32
            } else {
                0.0
            };
            let mut prev_bottom = 0.0f32;
            let mut first = true;
            for &c in &children {
                let e = &self.entries[c];
                let mut h = e.min_h + (e.max_h - e.min_h) * t;
                h += extra * e.stretch_h as f32;
                if !self.is_space(c) {
                    let top = if first {
                        first = false;
                        0.0
                    } else {
                        e.margin[2]
                    };
                    y += prev_bottom.max(top);
                    prev_bottom = e.margin[3];
                }
                self.set_vertical(c, y.round(), h.round());
                y += h;
            }
        }
    }

    pub fn rect(&self, entry: usize) -> [f32; 4] {
        self.entries[entry].rect
    }

    /// The natural height of a group's children (a scroll view's content).
    pub fn content_height(&self, entry: usize) -> f32 {
        self.entries[entry].child_min_h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(w: f32, h: f32, stretch: bool) -> LeafSize {
        LeafSize {
            size: (w, h),
            stretch_w: stretch,
            ..LeafSize::default()
        }
    }

    #[test]
    fn stretching_label_leaves_the_fixed_field_its_width() {
        // Area 400 wide: [Label (stretch, 50), TextField (fixed 100)] with a 34 px label margin.
        let mut l = Layout::new_area([10.0, 20.0, 400.0, 100.0], [0.0; 4]);
        l.begin_group(false, false, [0.0; 4], [0.0; 4], None, None);
        let label = l.leaf(LeafSize {
            margin: [0.0, 34.0, 0.0, 0.0],
            ..leaf(50.0, 18.0, true)
        });
        let field = l.leaf(LeafSize {
            width: Some(100.0),
            height: Some(22.0),
            ..LeafSize::default()
        });
        l.end_group();
        l.compute();
        let (a, b) = (l.rect(label), l.rect(field));
        assert_eq!(b[2], 100.0);
        assert_eq!(b[0] + b[2], 410.0);
        assert_eq!(a[0], 10.0);
        assert_eq!(a[0] + a[2] + 34.0, b[0]);
        // Row height: the tallest child.
        assert_eq!(l.rect(1)[3], 22.0);
    }

    #[test]
    fn flexible_space_pushes_buttons_to_the_edges() {
        let mut l = Layout::new_area([0.0, 0.0, 300.0, 100.0], [0.0; 4]);
        l.begin_group(false, false, [0.0; 4], [0.0; 4], None, None);
        l.flexible_space();
        let button = l.leaf(LeafSize {
            width: Some(100.0),
            height: Some(40.0),
            ..LeafSize::default()
        });
        l.end_group();
        l.compute();
        assert_eq!(l.rect(button)[0], 200.0);
    }
}
