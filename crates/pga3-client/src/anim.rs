//! Unity legacy animation: legacy `AnimationClip`s and the `Animation` component.
//!
//! Spec and evidence: `docs/gameplay/animation.md`. `Play`, `CrossFade`, the per-frame state
//! update (time, wrapping, fading, auto-stop) and `Animation.wrapMode` are ports of the native
//! Unity 2018.3 code decompiled from the original build (`f1685`, `f1694`, `f1730`, `f1670`,
//! `f1886`). Blending of several states is inferred (normalised weights, see the doc).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
use glam::{Quat, Vec3, Vec4};
use serde_json::Value;

use crate::assets::{f32_of, load_json};

/// Unity `WrapMode` (`Once` = `Clamp` = 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapMode {
    Default,
    Once,
    Loop,
    PingPong,
    ClampForever,
}

impl WrapMode {
    pub fn from_unity(value: i64) -> WrapMode {
        match value {
            1 => WrapMode::Once,
            2 => WrapMode::Loop,
            4 => WrapMode::PingPong,
            8 => WrapMode::ClampForever,
            _ => WrapMode::Default,
        }
    }
}

/// Local transform of a node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trs {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct Key {
    time: f32,
    value: Vec4,
    in_slope: Vec4,
    out_slope: Vec4,
}

/// A keyframed curve of up to 4 components (every key in this build is non-weighted with finite
/// slopes, and pre/post infinity is Clamp).
#[derive(Clone, Debug)]
pub struct Curve {
    keys: Vec<Key>,
}

impl Curve {
    /// Unity's `AnimationCurve` evaluation: cubic Hermite between keys, clamped outside.
    pub fn evaluate(&self, t: f32) -> Vec4 {
        let keys = &self.keys;
        let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
            return Vec4::ZERO;
        };
        if t <= first.time {
            return first.value;
        }
        if t >= last.time {
            return last.value;
        }
        let right = keys.partition_point(|k| k.time <= t);
        let (lhs, rhs) = (&keys[right - 1], &keys[right]);
        let dx = rhs.time - lhs.time;
        let (s, m0, m1) = if dx != 0.0 {
            ((t - lhs.time) / dx, lhs.out_slope * dx, rhs.in_slope * dx)
        } else {
            (0.0, Vec4::ZERO, Vec4::ZERO)
        };
        let s2 = s * s;
        let s3 = s2 * s;
        let a = 2.0 * s3 - 3.0 * s2 + 1.0;
        let b = s3 - 2.0 * s2 + s;
        let c = s3 - s2;
        let d = -2.0 * s3 + 3.0 * s2;
        lhs.value * a + m0 * b + m1 * c + rhs.value * d
    }

    fn range(&self) -> Option<(f32, f32)> {
        Some((self.keys.first()?.time, self.keys.last()?.time))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Position,
    Rotation,
    Scale,
}

#[derive(Clone, Debug)]
pub struct ClipCurve {
    /// Transform path relative to the `Animation` component's GameObject ("" = itself).
    pub path: String,
    pub channel: Channel,
    pub curve: Curve,
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub name: String,
    /// (first key time, last key time) over all curves.
    pub range: (f32, f32),
    pub wrap_mode: WrapMode,
    pub curves: Vec<ClipCurve>,
}

impl Clip {
    #[allow(dead_code, reason = "Animation API used by scripts not ported yet")]
    pub fn length(&self) -> f32 {
        self.range.1 - self.range.0
    }
}

fn vec4_value(value: &Value) -> Vec4 {
    Vec4::new(
        f32_of(&value["x"]),
        f32_of(&value["y"]),
        f32_of(&value["z"]),
        if value.get("w").is_some() {
            f32_of(&value["w"])
        } else {
            0.0
        },
    )
}

/// Parses a legacy clip from `data/animations/AnimationClip/*.json`.
pub fn parse_clip(json: &Value) -> Result<Clip> {
    let data = json.get("data").unwrap_or(json);
    let mut curves = Vec::new();
    for (field, channel) in [
        ("m_PositionCurves", Channel::Position),
        ("m_RotationCurves", Channel::Rotation),
        ("m_ScaleCurves", Channel::Scale),
    ] {
        for entry in data[field].as_array().into_iter().flatten() {
            let keys = entry["curve"]["m_Curve"]
                .as_array()
                .context("curve without keys")?
                .iter()
                .map(|k| Key {
                    time: f32_of(&k["time"]),
                    value: vec4_value(&k["value"]),
                    in_slope: vec4_value(&k["inSlope"]),
                    out_slope: vec4_value(&k["outSlope"]),
                })
                .collect();
            curves.push(ClipCurve {
                path: entry["path"].as_str().unwrap_or_default().to_string(),
                channel,
                curve: Curve { keys },
            });
        }
    }
    let mut range: Option<(f32, f32)> = None;
    for c in &curves {
        if let Some((start, end)) = c.curve.range() {
            range = Some(match range {
                Some((a, b)) => (a.min(start), b.max(end)),
                None => (start, end),
            });
        }
    }
    Ok(Clip {
        name: data["m_Name"].as_str().unwrap_or_default().to_string(),
        range: range.unwrap_or((0.0, 0.0)),
        wrap_mode: WrapMode::from_unity(data["m_WrapMode"].as_i64().unwrap_or(0)),
        curves,
    })
}

/// Clips by reference (`resources.assets:260`), loaded on demand.
pub struct ClipLibrary {
    files: HashMap<String, PathBuf>,
    loaded: HashMap<String, Rc<Clip>>,
}

impl ClipLibrary {
    /// Indexes `data/animations/AnimationClip/<name>__<file>_<path_id>.json`.
    pub fn new(assets: &Path) -> Result<ClipLibrary> {
        let dir = assets.join("data/animations/AnimationClip");
        let mut files = HashMap::new();
        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?
        {
            let path = entry?.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some((_, source)) = stem.rsplit_once("__") else {
                continue;
            };
            let Some((file, path_id)) = source.rsplit_once('_') else {
                continue;
            };
            files.insert(format!("{file}:{path_id}"), path.clone());
        }
        Ok(ClipLibrary {
            files,
            loaded: HashMap::new(),
        })
    }

    pub fn get(&mut self, reference: &str) -> Result<Option<Rc<Clip>>> {
        if let Some(clip) = self.loaded.get(reference) {
            return Ok(Some(clip.clone()));
        }
        let Some(path) = self.files.get(reference) else {
            return Ok(None);
        };
        let clip = Rc::new(parse_clip(&load_json(path)?).with_context(|| reference.to_string())?);
        self.loaded.insert(reference.to_string(), clip.clone());
        Ok(Some(clip))
    }
}

/// Native `f1670`: maps a state time into the clip range for a wrap mode.
fn wrap_time(t: f32, (start, end): (f32, f32), mode: WrapMode) -> f32 {
    let length = end - start;
    let repeat = |t: f32| start + ((t - start) - length * ((t - start) / length).floor());
    let ping_pong = |t: f32| {
        let offset = t - start;
        let double = length * 2.0;
        let m = offset - double * (offset / double).floor() - length;
        start + (length - m.abs())
    };
    if end <= t {
        match mode {
            WrapMode::Once | WrapMode::ClampForever => end,
            WrapMode::Loop => repeat(t),
            WrapMode::PingPong => ping_pong(t),
            WrapMode::Default => t,
        }
    } else if t < start {
        match mode {
            WrapMode::Once | WrapMode::ClampForever => start,
            WrapMode::Loop => repeat(t),
            WrapMode::PingPong => ping_pong(t),
            WrapMode::Default => t,
        }
    } else {
        t
    }
}

/// One `AnimationState`. Field comments give the native offset in Unity's state object.
#[derive(Clone, Debug)]
pub struct AnimationState {
    pub name: String,
    pub clip: Rc<Clip>,
    /// Node index per clip curve (`None` = path not found, curve ignored).
    bindings: Vec<Option<usize>>,
    /// Flag 2.
    pub enabled: bool,
    /// +16.
    pub weight: f32,
    /// +44 (`speed`, 1 by default).
    pub speed: f32,
    /// +24, a double.
    pub time: f64,
    /// +20, the sampled (wrapped) time.
    wrapped_time: f32,
    /// +76.
    pub wrap_mode: WrapMode,
    /// +40.
    pub layer: i32,
    /// Flag 1, +60 target weight, +84 weight change per second.
    fading: bool,
    fade_target: f32,
    fade_speed: f32,
    /// Flag 4: stop when the fade reaches its target.
    stop_when_faded: bool,
    /// Flag 32: the automatic fade-out before the end of a Once clip has started.
    auto_fade_started: bool,
    /// +56: fade length of the `CrossFade`/`Play` that started the state.
    fade_length: f32,
    /// +32: animation clock at the last update (set when the state is enabled).
    last_update: f64,
}

impl AnimationState {
    fn wrap(&self, t: f32) -> f32 {
        wrap_time(t, self.clip.range, self.wrap_mode)
    }

    /// +52: Once/Default states stop at the end of the range (start when playing backwards).
    fn stop_time(&self) -> f32 {
        match self.wrap_mode {
            WrapMode::Default | WrapMode::Once => {
                if self.speed >= 0.0 {
                    self.clip.range.1
                } else {
                    self.clip.range.0
                }
            }
            _ => {
                if self.speed >= 0.0 {
                    f32::INFINITY
                } else {
                    f32::NEG_INFINITY
                }
            }
        }
    }

    fn stop(&mut self) {
        self.enabled = false;
        self.fading = false;
        self.stop_when_faded = false;
        self.auto_fade_started = false;
        self.time = 0.0;
        self.wrapped_time = self.wrap(0.0);
        self.weight = 0.0;
    }

    /// Native fade set-up shared by `CrossFade` and the automatic fade-out: keeps the existing
    /// speed when already fading to the same target faster.
    fn setup_fade(&mut self, target: f32, length: f32) {
        let difference = target - self.weight;
        let mut speed = if f64::from(length) > 0.001 {
            difference / length
        } else {
            difference * 100_000.0
        };
        if self.fading
            && (self.fade_target - target).abs() <= 1e-4
            && self.fade_speed.abs() > speed.abs()
        {
            speed = self.fade_speed;
        }
        if speed.abs() <= 1e-4 {
            speed = 100_000.0;
        }
        self.fade_speed = speed;
        self.fade_target = target;
        self.fading = true;
    }

    /// Native `f1694` + `f1730`. Returns true when the state stopped this frame.
    fn update(&mut self, now: f64) -> bool {
        let dt = (now - self.last_update) as f32;
        self.last_update = now;
        let delta = self.speed * dt;
        if delta != 0.0 {
            self.time += f64::from(delta);
            let (start, end) = self.clip.range;
            self.wrapped_time = match self.wrap_mode {
                WrapMode::Default => self.wrapped_time + delta,
                WrapMode::Once | WrapMode::ClampForever => {
                    (self.time.clamp(f64::from(start), f64::from(end))) as f32
                }
                WrapMode::Loop => {
                    let start = f64::from(start);
                    let length = f64::from(end) - start;
                    let offset = self.time - start;
                    (offset - length * (offset / length).floor() + start) as f32
                }
                WrapMode::PingPong => self.wrap(self.time as f32),
            };
        }

        // Automatic fade-out / stop at the end of Once and Default states.
        if !self.auto_fade_started && matches!(self.wrap_mode, WrapMode::Default | WrapMode::Once) {
            let overshoot = if self.speed >= 0.0 {
                self.time - f64::from(self.stop_time() - self.fade_length)
            } else {
                f64::from(self.stop_time() + self.fade_length) - self.time
            } as f32;
            if overshoot > 0.0 {
                self.setup_fade(0.0, self.fade_length);
                self.fade_target = 0.0;
                self.stop_when_faded = true;
                self.auto_fade_started = true;
                self.weight += self.fade_speed * overshoot;
                let still_fading = self.fade_length != 0.0
                    && !(self.fade_speed > 0.0 && self.weight > 0.0)
                    && !(self.fade_speed <= 0.0 && self.weight < 0.0);
                if !still_fading {
                    self.stop();
                    return true;
                }
            }
        }

        if self.fading {
            self.weight += self.fade_speed * dt;
            let reached = (self.fade_speed > 0.0 && self.weight > self.fade_target)
                || (self.fade_speed <= 0.0 && self.weight < self.fade_target);
            if reached {
                self.weight = self.fade_target;
                self.fading = false;
                if self.stop_when_faded {
                    self.stop();
                    return true;
                }
            }
        }
        false
    }
}

/// `PlayMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayMode {
    StopSameLayer,
    #[allow(dead_code, reason = "Animation API used by scripts not ported yet")]
    StopAll,
}

/// The legacy `Animation` component.
#[derive(Clone, Debug)]
pub struct Animation {
    /// Node of the GameObject carrying the component (curve paths are relative to it).
    pub node: usize,
    pub wrap_mode: WrapMode,
    pub play_automatically: bool,
    pub default_clip: Option<String>,
    pub states: Vec<AnimationState>,
    /// Animation clock (`Time.time`), advanced by `begin_frame`.
    pub clock: f64,
}

impl Animation {
    /// `resolve(path)` maps a curve path to a node index.
    pub fn new(
        node: usize,
        wrap_mode: WrapMode,
        play_automatically: bool,
        default_clip: Option<String>,
        clips: &[Rc<Clip>],
        resolve: impl Fn(&str) -> Option<usize>,
    ) -> Animation {
        let mut states = Vec::new();
        for clip in clips {
            if states.iter().any(|s: &AnimationState| s.name == clip.name) {
                continue;
            }
            let state_wrap = if clip.wrap_mode == WrapMode::Default {
                wrap_mode
            } else {
                clip.wrap_mode
            };
            let mut state = AnimationState {
                name: clip.name.clone(),
                clip: clip.clone(),
                bindings: clip.curves.iter().map(|c| resolve(&c.path)).collect(),
                enabled: false,
                weight: 0.0,
                speed: 1.0,
                time: 0.0,
                wrapped_time: 0.0,
                wrap_mode: state_wrap,
                layer: 0,
                fading: false,
                fade_target: 0.0,
                fade_speed: 0.0,
                stop_when_faded: false,
                auto_fade_started: false,
                fade_length: 0.0,
                last_update: 0.0,
            };
            state.wrapped_time = state.wrap(0.0);
            states.push(state);
        }
        Animation {
            node,
            wrap_mode,
            play_automatically,
            default_clip,
            states,
            clock: 0.0,
        }
    }

    pub fn state_index(&self, name: &str) -> Option<usize> {
        self.states.iter().position(|s| s.name == name)
    }

    pub fn state_mut(&mut self, name: &str) -> Option<&mut AnimationState> {
        self.states.iter_mut().find(|s| s.name == name)
    }

    #[allow(dead_code, reason = "Animation API used by scripts not ported yet")]
    pub fn is_playing(&self, name: &str) -> bool {
        self.states.iter().any(|s| s.name == name && s.enabled)
    }

    /// `Animation.wrapMode` setter (native `f1886`): applies to every state.
    pub fn set_wrap_mode(&mut self, mode: WrapMode) {
        self.wrap_mode = mode;
        for state in &mut self.states {
            state.wrap_mode = mode;
            state.wrapped_time = state.wrap(state.time as f32);
        }
    }

    /// `AnimationState.wrapMode` setter.
    pub fn set_state_wrap_mode(&mut self, name: &str, mode: WrapMode) {
        if let Some(state) = self.state_mut(name) {
            state.wrap_mode = mode;
            state.wrapped_time = state.wrap(state.time as f32);
        }
    }

    /// `AnimationState.time` setter (native `f1906`).
    pub fn set_time(&mut self, name: &str, time: f32) {
        if let Some(state) = self.state_mut(name) {
            state.time = f64::from(time);
            state.wrapped_time = state.wrap(time);
        }
    }

    pub fn set_speed(&mut self, name: &str, speed: f32) {
        if let Some(state) = self.state_mut(name) {
            state.speed = speed;
        }
    }

    /// `Animation.Rewind(name)`.
    pub fn rewind(&mut self, name: &str) {
        self.set_time(name, 0.0);
    }

    /// `Animation.Stop(name)`.
    #[allow(dead_code, reason = "Animation API used by scripts not ported yet")]
    pub fn stop(&mut self, name: &str) {
        if let Some(state) = self.state_mut(name) {
            state.stop();
        }
    }

    /// `Animation.Play(name)`: native `Play` is `CrossFade(name, 0)`.
    pub fn play(&mut self, name: &str) -> bool {
        self.cross_fade_mode(name, 0.0, PlayMode::StopSameLayer)
    }

    /// `Animation.CrossFade(name, fadeLength)`.
    pub fn cross_fade(&mut self, name: &str, fade_length: f32) -> bool {
        self.cross_fade_mode(name, fade_length, PlayMode::StopSameLayer)
    }

    /// Native `Animation::CrossFade(state, fadeLength, mode)` (`f1685`).
    pub fn cross_fade_mode(&mut self, name: &str, fade_length: f32, mode: PlayMode) -> bool {
        let Some(target) = self.state_index(name) else {
            return false;
        };
        let layer = self.states[target].layer;
        let fades = fade_length > 0.001;
        let now = self.clock;
        for (index, state) in self.states.iter_mut().enumerate() {
            if mode == PlayMode::StopSameLayer && state.layer != layer {
                continue;
            }
            if index != target {
                if !fades {
                    state.stop();
                } else {
                    state.setup_fade(0.0, fade_length);
                    state.stop_when_faded = true;
                }
                continue;
            }
            if !state.enabled {
                state.last_update = now;
                state.enabled = true;
            }
            if fades {
                state.setup_fade(1.0, fade_length);
                state.stop_when_faded = false;
            } else {
                state.weight = 1.0;
                state.fading = false;
                state.stop_when_faded = false;
            }
            state.auto_fade_started = false;
            state.fade_length = fade_length;
        }
        true
    }

    /// Advances the animation clock to this frame's `Time.time` (call before scripts' Update).
    pub fn begin_frame(&mut self, dt: f32) {
        self.clock += f64::from(dt);
    }

    /// Unity's per-frame animation update (after Update, before LateUpdate): advances enabled
    /// states, then writes the blended pose into `locals` (indexed by node).
    pub fn update(&mut self, locals: &mut [Trs]) {
        let now = self.clock;
        for state in &mut self.states {
            if state.enabled {
                state.update(now);
            }
        }
        self.sample(locals);
    }

    /// Writes the weighted pose of every enabled state into the bound nodes. Properties no
    /// enabled state animates keep their value; weights are normalised per property.
    pub fn sample(&self, locals: &mut [Trs]) {
        #[derive(Clone, Copy, Default)]
        struct Accumulator {
            position: Vec3,
            position_weight: f32,
            rotation: Vec4,
            rotation_weight: f32,
            scale: Vec3,
            scale_weight: f32,
        }
        let mut accumulators: HashMap<usize, Accumulator> = HashMap::new();
        for state in self.states.iter().filter(|s| s.enabled && s.weight > 0.0) {
            let w = state.weight;
            for (curve, binding) in state.clip.curves.iter().zip(&state.bindings) {
                let Some(node) = *binding else {
                    continue;
                };
                if node >= locals.len() {
                    continue;
                }
                let value = curve.curve.evaluate(state.wrapped_time);
                let acc = accumulators.entry(node).or_default();
                match curve.channel {
                    Channel::Position => {
                        acc.position += value.truncate() * w;
                        acc.position_weight += w;
                    }
                    Channel::Scale => {
                        acc.scale += value.truncate() * w;
                        acc.scale_weight += w;
                    }
                    Channel::Rotation => {
                        // Unity normalises a quaternion curve's evaluated value.
                        let q = value.normalize_or_zero();
                        let aligned = if acc.rotation_weight > 0.0 && acc.rotation.dot(q) < 0.0 {
                            -q
                        } else {
                            q
                        };
                        acc.rotation += aligned * w;
                        acc.rotation_weight += w;
                    }
                }
            }
        }
        for (node, acc) in accumulators {
            let local = &mut locals[node];
            if acc.position_weight > 0.0 {
                local.position = acc.position / acc.position_weight;
            }
            if acc.scale_weight > 0.0 {
                local.scale = acc.scale / acc.scale_weight;
            }
            if acc.rotation_weight > 0.0 {
                let q = acc.rotation.normalize_or_zero();
                if q != Vec4::ZERO {
                    local.rotation = Quat::from_vec4(q);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(time: f32, value: f32, slope: f32) -> Key {
        Key {
            time,
            value: Vec4::splat(value),
            in_slope: Vec4::splat(slope),
            out_slope: Vec4::splat(slope),
        }
    }

    fn clip(name: &str, end: f32) -> Rc<Clip> {
        Rc::new(Clip {
            name: name.to_string(),
            range: (0.0, end),
            wrap_mode: WrapMode::Default,
            curves: vec![ClipCurve {
                path: String::new(),
                channel: Channel::Position,
                curve: Curve {
                    keys: vec![key(0.0, 0.0, 0.0), key(end, 1.0, 0.0)],
                },
            }],
        })
    }

    fn animation(wrap: WrapMode) -> Animation {
        Animation::new(
            0,
            wrap,
            false,
            None,
            &[clip("A", 1.0), clip("B", 2.0)],
            |path| (path.is_empty()).then_some(0),
        )
    }

    fn identity() -> Vec<Trs> {
        vec![Trs {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }]
    }

    #[test]
    fn hermite_matches_linear_slopes() {
        // Slope 1 between (0,0) and (1,1) is the straight line.
        let curve = Curve {
            keys: vec![key(0.0, 0.0, 1.0), key(1.0, 1.0, 1.0)],
        };
        for t in [0.0, 0.25, 0.5, 0.9] {
            assert!((curve.evaluate(t).x - t).abs() < 1e-6);
        }
        assert_eq!(curve.evaluate(-1.0).x, 0.0);
        assert_eq!(curve.evaluate(2.0).x, 1.0);
        // Flat tangents give smoothstep.
        let flat = Curve {
            keys: vec![key(0.0, 0.0, 0.0), key(1.0, 1.0, 0.0)],
        };
        assert!((flat.evaluate(0.25).x - 0.15625).abs() < 1e-6);
    }

    #[test]
    fn wrap_modes() {
        let r = (0.0, 2.0);
        assert_eq!(wrap_time(2.5, r, WrapMode::Once), 2.0);
        assert_eq!(wrap_time(2.5, r, WrapMode::ClampForever), 2.0);
        assert!((wrap_time(2.5, r, WrapMode::Loop) - 0.5).abs() < 1e-6);
        assert!((wrap_time(2.5, r, WrapMode::PingPong) - 1.5).abs() < 1e-6);
        assert_eq!(wrap_time(2.5, r, WrapMode::Default), 2.5);
    }

    #[test]
    fn play_does_not_advance_on_its_first_frame_and_stops_at_the_end() {
        let mut anim = animation(WrapMode::Once);
        let mut locals = identity();
        anim.begin_frame(0.1);
        anim.play("A");
        anim.update(&mut locals);
        assert_eq!(anim.states[0].time, 0.0);
        anim.begin_frame(0.5);
        anim.update(&mut locals);
        assert!((anim.states[0].time - 0.5).abs() < 1e-6);
        assert!(anim.is_playing("A"));
        anim.begin_frame(0.6);
        anim.update(&mut locals);
        assert!(!anim.is_playing("A"), "Once stops past the end");
        assert_eq!(anim.states[0].time, 0.0, "stopping rewinds");
    }

    #[test]
    fn cross_fade_is_linear_even_when_repeated_every_frame() {
        let mut anim = animation(WrapMode::Loop);
        let mut locals = identity();
        anim.play("A");
        anim.update(&mut locals);
        for frame in 1..=10 {
            anim.begin_frame(0.02);
            anim.cross_fade("B", 0.2);
            anim.update(&mut locals);
            // B is enabled during frame 1, so it first advances in frame 2; the fade speed
            // stays 1 / 0.2 although CrossFade is called again every frame.
            let b = anim.states[1].weight;
            let expected = ((frame - 1) as f32 * 0.1).min(1.0);
            assert!((b - expected).abs() < 1e-4, "frame {frame}: {b}");
        }
        anim.begin_frame(0.05);
        anim.cross_fade("B", 0.2);
        anim.update(&mut locals);
        assert!(!anim.is_playing("A"), "faded-out state stops");
        assert_eq!(anim.states[1].weight, 1.0);
    }

    #[test]
    fn set_wrap_mode_applies_to_all_states() {
        let mut anim = animation(WrapMode::Default);
        anim.set_wrap_mode(WrapMode::Loop);
        assert!(anim.states.iter().all(|s| s.wrap_mode == WrapMode::Loop));
    }

    #[test]
    fn single_state_pose_is_written() {
        let mut anim = animation(WrapMode::Loop);
        let mut locals = identity();
        anim.play("B");
        anim.begin_frame(1.0);
        anim.update(&mut locals);
        anim.begin_frame(1.0);
        anim.update(&mut locals);
        // t = 2 (end of B) wraps to 0 under Loop... B was enabled at clock 0, so it ran 2 s.
        assert!((anim.states[1].wrapped_time - 0.0).abs() < 1e-6);
        assert_eq!(locals[0].position.x, 0.0);
    }
}
