//! Audio: Unity `AudioSource` playback as the original WebGL build does it (spec:
//! `docs/gameplay/audio.md`).
//!
//! Unity WebGL plays sounds through the browser's Web Audio API (`pga3-v18.framework.js`,
//! `_JS_Sound_*`): every AudioSource is a channel `source → [PannerNode] → GainNode → output`.
//! The panner exists only for 3D sources and has `rolloffFactor = 0`, so it only pans (Web Audio's
//! default `equalpower` model); distance attenuation and volume are computed by Unity and set on
//! the gain node. `Play` stops the channel's current sound first (`_JS_Sound_Play` calls
//! `_JS_Sound_Stop`). Clips are the original AAC streams, decoded once at load.
//!
//! The mixer runs on the audio device's thread; the game thread owns the AudioSources and pushes
//! gains to the mixer once per frame.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use glam::{Quat, Vec3};
use serde_json::Value;

use crate::assets::f32_of;

/// A decoded clip: interleaved f32 samples.
pub struct Clip {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub rate: u32,
}

impl Clip {
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels.max(1)
    }

    #[cfg(test)]
    pub fn length_seconds(&self) -> f32 {
        self.frames() as f32 / self.rate.max(1) as f32
    }
}

/// Decodes an `.m4a` (AAC in MP4) file. The encoder delay given by the container is skipped, as
/// browsers do when decoding (`decodeAudioData`).
pub fn decode_clip(path: &Path) -> Result<Clip> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("m4a");
    let probed = symphonia::default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;
    let track = format.default_track().context("no audio track")?;
    let track_id = track.id;
    let params = track.codec_params.clone();
    let mut decoder = symphonia::default::get_codecs().make(&params, &DecoderOptions::default())?;
    let mut samples = Vec::new();
    let mut channels = params.channels.map_or(1, |c| c.count());
    let mut rate = params.sample_rate.unwrap_or(44_100);
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(Error::ResetRequired) => break,
            Err(error) => return Err(error.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(Error::DecodeError(_)) => continue,
            Err(error) => return Err(error.into()),
        };
        let spec = *decoded.spec();
        channels = spec.channels.count();
        rate = spec.rate;
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buffer.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buffer.samples());
    }
    let delay = params.delay.unwrap_or(0) as usize * channels;
    if delay > 0 && delay < samples.len() {
        samples.drain(..delay);
    }
    Ok(Clip {
        samples,
        channels,
        rate,
    })
}

/// The `AudioSource` settings this client uses.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceSettings {
    pub volume: f32,
    /// `spatialBlend` (the `panLevelCustomCurve` value) > 0: positioned through the panner.
    pub spatial: bool,
    pub min_distance: f32,
    pub max_distance: f32,
    /// `rolloffMode`: 0 logarithmic, 1 linear (2 custom is not used by the sources ported).
    pub rolloff: i32,
    pub play_on_awake: bool,
    pub clip: Option<String>,
}

impl SourceSettings {
    /// From an `AudioSource` component's `data`.
    pub fn from_component(data: &Value) -> SourceSettings {
        let pan = data["panLevelCustomCurve"]["m_Curve"][0]["value"]
            .as_f64()
            .unwrap_or(0.0);
        SourceSettings {
            volume: f32_of(&data["m_Volume"]),
            spatial: pan > 0.0,
            min_distance: f32_of(&data["MinDistance"]),
            max_distance: f32_of(&data["MaxDistance"]),
            rolloff: data["rolloffMode"].as_i64().unwrap_or(0) as i32,
            play_on_awake: data["m_PlayOnAwake"].as_bool().unwrap_or(false),
            clip: data["m_audioClip"]["asset"].as_str().map(str::to_string),
        }
    }
}

/// Unity's distance attenuation for 3D sources (`Logarithmic`: `min / d`, linear: `(max - d) /
/// (max - min)`), constant beyond `maxDistance` for the logarithmic curve.
pub fn attenuation(settings: &SourceSettings, distance: f32) -> f32 {
    let min = settings.min_distance.max(1e-4);
    let max = settings.max_distance.max(min);
    if distance <= min {
        return 1.0;
    }
    match settings.rolloff {
        1 => ((max - distance) / (max - min)).clamp(0.0, 1.0),
        _ => min / distance.min(max),
    }
}

/// Web Audio `PannerNode` equal-power gains for a source at `source` heard by a listener at
/// `listener` with Unity rotation `rotation`. Unity passes the listener's forward negated
/// (`_JS_Sound_SetListenerOrientation(-x, -y, -z, up)`), so the Web Audio "right" is Unity's +X.
/// Returns (left, right) gains for a mono input.
pub fn equal_power_gains(source: Vec3, listener: Vec3, rotation: Quat) -> (f32, f32) {
    let azimuth = panner_azimuth(source, listener, rotation);
    // Equal-power panning (Web Audio spec): azimuth folded into [-90, 90].
    let azimuth = if azimuth < -90.0 {
        -180.0 - azimuth
    } else if azimuth > 90.0 {
        180.0 - azimuth
    } else {
        azimuth
    };
    let x = (azimuth + 90.0) / 180.0;
    let angle = x * std::f32::consts::FRAC_PI_2;
    (angle.cos(), angle.sin())
}

/// The Web Audio spec's azimuth (degrees) of the source in the listener's frame.
fn panner_azimuth(source: Vec3, listener: Vec3, rotation: Quat) -> f32 {
    let to_source = source - listener;
    if to_source.length_squared() < 1e-12 {
        return 0.0;
    }
    let to_source = to_source.normalize();
    let forward = -(rotation * Vec3::Z);
    let up = rotation * Vec3::Y;
    let right = forward.cross(up).normalize_or_zero();
    let up = right.cross(forward).normalize_or_zero();
    let projected = (to_source - up * to_source.dot(up)).normalize_or_zero();
    let mut azimuth = projected.dot(right).clamp(-1.0, 1.0).acos().to_degrees();
    if projected.dot(forward) < 0.0 {
        azimuth = 360.0 - azimuth;
    }
    if (0.0..=270.0).contains(&azimuth) {
        90.0 - azimuth
    } else {
        450.0 - azimuth
    }
}

struct Voice {
    id: u64,
    clip: Arc<Clip>,
    /// Read position in source frames.
    position: f64,
    /// Gain applied to each input channel per output channel: [left, right] for mono input,
    /// or (for stereo input) the Web Audio stereo-panner gains.
    gain: Gain,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Gain {
    /// 2D: input played as is (mono up-mixed to both speakers), times volume.
    Direct(f32),
    /// 3D mono: per-speaker gains.
    Mono(f32, f32),
    /// 3D stereo (Web Audio equal-power stereo panning): (azimuth-derived gains, volume).
    Stereo { pan: f32, volume: f32 },
}

struct Mixer {
    voices: Vec<Voice>,
    output_rate: u32,
    /// `AudioListener.volume` (the project's `AudioManager.m_Volume`) times the client's own
    /// master volume (`docs/gameplay/settings.md`, "Volume").
    master: f32,
}

impl Default for Mixer {
    fn default() -> Mixer {
        Mixer {
            voices: Vec::new(),
            output_rate: 0,
            master: 1.0,
        }
    }
}

impl Mixer {
    /// Mixes into an interleaved buffer of `channels` channels (the first two are left/right).
    fn mix(&mut self, out: &mut [f32], channels: usize) {
        out.iter_mut().for_each(|s| *s = 0.0);
        if channels == 0 {
            return;
        }
        let frames = out.len() / channels;
        for voice in &mut self.voices {
            let clip = &voice.clip;
            let step = clip.rate as f64 / self.output_rate.max(1) as f64;
            let total = clip.frames();
            for frame in 0..frames {
                let index = voice.position as usize;
                if index >= total {
                    break;
                }
                let t = (voice.position - index as f64) as f32;
                let sample = |c: usize| {
                    let a = clip.samples[index * clip.channels + c];
                    let b = if index + 1 < total {
                        clip.samples[(index + 1) * clip.channels + c]
                    } else {
                        a
                    };
                    a + (b - a) * t
                };
                let (l_in, r_in) = if clip.channels >= 2 {
                    (sample(0), sample(1))
                } else {
                    let m = sample(0);
                    (m, m)
                };
                let (l, r) = match voice.gain {
                    Gain::Direct(v) => (l_in * v, r_in * v),
                    Gain::Mono(gl, gr) => (l_in * gl, l_in * gr),
                    Gain::Stereo { pan, volume } => {
                        // Web Audio stereo equal-power panning, pan in [-1, 1].
                        if pan <= 0.0 {
                            let x = (pan + 1.0) * std::f32::consts::FRAC_PI_2;
                            ((l_in + r_in * x.cos()) * volume, r_in * x.sin() * volume)
                        } else {
                            let x = pan * std::f32::consts::FRAC_PI_2;
                            (l_in * x.cos() * volume, (r_in + l_in * x.sin()) * volume)
                        }
                    }
                };
                out[frame * channels] += l;
                if channels > 1 {
                    out[frame * channels + 1] += r;
                }
                voice.position += step;
            }
        }
        self.voices
            .retain(|v| (v.position as usize) < v.clip.frames());
        if self.master != 1.0 {
            out.iter_mut().for_each(|s| *s *= self.master);
        }
    }
}

/// `AudioManager.m_Volume` (the project's initial `AudioListener.volume`).
fn project_volume(assets: &Path) -> f32 {
    crate::assets::load_json(&assets.join("data/settings/AudioManager.json"))
        .map_or(1.0, |json| json["m_Volume"].as_f64().unwrap_or(1.0) as f32)
}

struct Source {
    settings: SourceSettings,
    position: Vec3,
    /// Mixer voice of the current `Play`.
    voice: Option<u64>,
}

/// AudioListener + every AudioSource of the game, and the output device.
pub struct Audio {
    assets: PathBuf,
    mixer: Arc<Mutex<Mixer>>,
    /// Kept alive while playing; `None` when no device is available (silent).
    _stream: Option<cpal::Stream>,
    clips: HashMap<String, Option<Arc<Clip>>>,
    sources: HashMap<u64, Source>,
    next_id: u64,
    listener: (Vec3, Quat),
    /// `AudioManager.m_Volume`.
    project_volume: f32,
}

impl Audio {
    /// Opens the default output device; without one the game runs silently.
    pub fn new(assets: &Path) -> Audio {
        let mixer = Arc::new(Mutex::new(Mixer::default()));
        let stream = match open_stream(mixer.clone()) {
            Ok(stream) => Some(stream),
            Err(error) => {
                eprintln!("audio disabled: {error:#}");
                None
            }
        };
        Audio {
            assets: assets.to_path_buf(),
            mixer,
            _stream: stream,
            clips: HashMap::new(),
            sources: HashMap::new(),
            next_id: 1,
            listener: (Vec3::ZERO, Quat::IDENTITY),
            project_volume: project_volume(assets),
        }
    }

    /// No output device (tests, screenshots).
    pub fn silent(assets: &Path) -> Audio {
        Audio {
            assets: assets.to_path_buf(),
            mixer: Arc::new(Mutex::new(Mixer::default())),
            _stream: None,
            clips: HashMap::new(),
            sources: HashMap::new(),
            next_id: 1,
            listener: (Vec3::ZERO, Quat::IDENTITY),
            project_volume: project_volume(assets),
        }
    }

    /// The client-only master volume (0..1) on top of `AudioListener.volume`.
    pub fn set_volume(&mut self, volume: f32) {
        let master = self.project_volume * volume.clamp(0.0, 1.0);
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.master = master;
        }
    }

    /// Decodes a clip (cached; a clip that fails to decode is reported once and stays silent).
    pub fn clip(&mut self, path: &str) -> Option<Arc<Clip>> {
        if let Some(clip) = self.clips.get(path) {
            return clip.clone();
        }
        let clip = match decode_clip(&self.assets.join(path)) {
            Ok(clip) => Some(Arc::new(clip)),
            Err(error) => {
                eprintln!("audio clip {path}: {error:#}");
                None
            }
        };
        self.clips.insert(path.to_string(), clip.clone());
        clip
    }

    /// Adds an AudioSource; `playOnAwake` sources with a clip start playing.
    pub fn add_source(&mut self, settings: SourceSettings, position: Vec3) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let clip = settings.clip.clone();
        let play = settings.play_on_awake;
        self.sources.insert(
            id,
            Source {
                settings,
                position,
                voice: None,
            },
        );
        if play && let Some(clip) = clip {
            self.play(id, &clip);
        }
        id
    }

    /// `audio.clip = clip; audio.Play()`: restarts the source with `clip`.
    pub fn play(&mut self, id: u64, clip: &str) {
        let Some(data) = self.clip(clip) else {
            return;
        };
        self.stop(id);
        let Some(source) = self.sources.get_mut(&id) else {
            return;
        };
        source.settings.clip = Some(clip.to_string());
        let voice = self.next_id;
        self.next_id += 1;
        source.voice = Some(voice);
        let gain = gain_of(source, data.channels, self.listener);
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.voices.push(Voice {
                id: voice,
                clip: data,
                position: 0.0,
                gain,
            });
        }
    }

    /// `AudioSource.Stop` (also what deactivating or destroying its GameObject does).
    pub fn stop(&mut self, id: u64) {
        let Some(voice) = self.sources.get_mut(&id).and_then(|s| s.voice.take()) else {
            return;
        };
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.voices.retain(|v| v.id != voice);
        }
    }

    /// Destroys the source (its GameObject was destroyed).
    pub fn remove(&mut self, id: u64) {
        self.stop(id);
        self.sources.remove(&id);
    }

    pub fn set_position(&mut self, id: u64, position: Vec3) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.position = position;
        }
    }

    /// The AudioListener's transform (the Main Camera's), then the gains of every playing
    /// source (once per frame, as Unity's audio update).
    pub fn update(&mut self, listener_position: Vec3, listener_rotation: Quat) {
        self.listener = (listener_position, listener_rotation);
        let Ok(mut mixer) = self.mixer.lock() else {
            return;
        };
        for source in self.sources.values() {
            let Some(voice_id) = source.voice else {
                continue;
            };
            if let Some(voice) = mixer.voices.iter_mut().find(|v| v.id == voice_id) {
                voice.gain = gain_of(source, voice.clip.channels, self.listener);
            }
        }
    }
}

fn gain_of(source: &Source, channels: usize, listener: (Vec3, Quat)) -> Gain {
    let s = &source.settings;
    if !s.spatial {
        return Gain::Direct(s.volume);
    }
    let distance = source.position.distance(listener.0);
    let volume = s.volume * attenuation(s, distance);
    if channels >= 2 {
        let azimuth = panner_azimuth(source.position, listener.0, listener.1);
        let folded = if azimuth < -90.0 {
            -180.0 - azimuth
        } else if azimuth > 90.0 {
            180.0 - azimuth
        } else {
            azimuth
        };
        Gain::Stereo {
            pan: folded / 90.0,
            volume,
        }
    } else {
        let (l, r) = equal_power_gains(source.position, listener.0, listener.1);
        Gain::Mono(l * volume, r * volume)
    }
}

fn open_stream(mixer: Arc<Mutex<Mixer>>) -> Result<cpal::Stream> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("no audio output device")?;
    let supported = device.default_output_config()?;
    let format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let channels = config.channels as usize;
    if let Ok(mut m) = mixer.lock() {
        m.output_rate = config.sample_rate.0;
    }
    let error = |e| eprintln!("audio stream error: {e}");
    let stream = match format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if let Ok(mut m) = mixer.lock() {
                    m.mix(data, channels);
                }
            },
            error,
            None,
        )?,
        cpal::SampleFormat::I16 => {
            let mut scratch = Vec::new();
            device.build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    scratch.resize(data.len(), 0.0);
                    if let Ok(mut m) = mixer.lock() {
                        m.mix(&mut scratch, channels);
                    }
                    for (o, s) in data.iter_mut().zip(&scratch) {
                        *o = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                    }
                },
                error,
                None,
            )?
        }
        cpal::SampleFormat::U16 => {
            let mut scratch = Vec::new();
            device.build_output_stream(
                &config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    scratch.resize(data.len(), 0.0);
                    if let Ok(mut m) = mixer.lock() {
                        m.mix(&mut scratch, channels);
                    }
                    for (o, s) in data.iter_mut().zip(&scratch) {
                        *o = ((s.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16;
                    }
                },
                error,
                None,
            )?
        }
        other => anyhow::bail!("unsupported sample format {other:?}"),
    };
    stream.play()?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    #[test]
    fn decodes_every_clip() {
        let assets = find_assets_dir().unwrap();
        let mut count = 0;
        for entry in std::fs::read_dir(assets.join("audio")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "m4a") {
                let clip = decode_clip(&path).unwrap();
                assert!(clip.frames() > 0, "{}", path.display());
                assert!(clip.channels == 1 || clip.channels == 2);
                count += 1;
            }
        }
        assert_eq!(count, 48);
        let lobby = decode_clip(&assets.join("audio/muzica lobby.m4a")).unwrap();
        assert!(
            (lobby.length_seconds() - 22.76).abs() < 0.05,
            "{}",
            lobby.length_seconds()
        );
    }

    #[test]
    fn panner_matches_web_audio_equal_power() {
        let listener = Vec3::ZERO;
        let rotation = Quat::IDENTITY;
        // Straight ahead: centre (cos 45°, sin 45°).
        let (l, r) = equal_power_gains(Vec3::Z * 5.0, listener, rotation);
        assert!((l - r).abs() < 1e-5 && (l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5);
        // Unity +X (right) is fully right; behind is centred again.
        let (l, r) = equal_power_gains(Vec3::X, listener, rotation);
        assert!(l.abs() < 1e-5 && (r - 1.0).abs() < 1e-5);
        let (l, r) = equal_power_gains(-Vec3::X, listener, rotation);
        assert!((l - 1.0).abs() < 1e-5 && r.abs() < 1e-5);
        let (l, r) = equal_power_gains(-Vec3::Z, listener, rotation);
        assert!((l - r).abs() < 1e-5);
        // Turned 90° to the right (yaw +90°), a source at +X is ahead.
        let turned = Quat::from_rotation_y(90f32.to_radians());
        let (l, r) = equal_power_gains(Vec3::X, listener, turned);
        assert!((l - r).abs() < 1e-4, "{l} {r}");
    }

    #[test]
    fn logarithmic_rolloff() {
        let s = SourceSettings {
            volume: 1.0,
            spatial: true,
            min_distance: 1.0,
            max_distance: 500.0,
            rolloff: 0,
            play_on_awake: false,
            clip: None,
        };
        assert_eq!(attenuation(&s, 0.5), 1.0);
        assert!((attenuation(&s, 4.0) - 0.25).abs() < 1e-6);
        assert!((attenuation(&s, 1000.0) - 1.0 / 500.0).abs() < 1e-6);
    }

    #[test]
    fn play_restarts_and_mixes() {
        let assets = find_assets_dir().unwrap();
        let mut audio = Audio::silent(&assets);
        if let Ok(mut m) = audio.mixer.lock() {
            m.output_rate = 44_100;
        }
        let settings = SourceSettings {
            volume: 1.0,
            spatial: false,
            min_distance: 1.0,
            max_distance: 500.0,
            rolloff: 0,
            play_on_awake: false,
            clip: None,
        };
        let id = audio.add_source(settings, Vec3::ZERO);
        audio.play(id, "audio/Pistol Fire.m4a");
        audio.play(id, "audio/Pistol Fire.m4a");
        assert_eq!(audio.mixer.lock().unwrap().voices.len(), 1);
        let mut out = vec![0.0; 4096];
        audio.mixer.lock().unwrap().mix(&mut out, 2);
        assert!(out.iter().any(|s| s.abs() > 1e-3));
        audio.remove(id);
        assert!(audio.mixer.lock().unwrap().voices.is_empty());
    }
}
