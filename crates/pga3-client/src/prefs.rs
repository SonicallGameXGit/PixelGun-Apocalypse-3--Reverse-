//! `PlayerPrefs` on disk and the settings of a session (spec: `docs/gameplay/settings.md`).
//!
//! Unity WebGL keeps PlayerPrefs in the browser; this client keeps them in
//! `$XDG_DATA_HOME/pga3-client/prefs.json` (fallback `~/.local/share/...`), one JSON value per key,
//! typed as Unity types them: integers are `Int`, numbers with a decimal point `Float`.
//! Every `set_*` writes the file at once.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::Serialize;
use serde_json::Value;

use crate::assets::{f32_of, load_json};

/// `RoomMultiplayerMenu.sens` (`.cctor`), used until PlayerPrefs has "sens".
pub const DEFAULT_SENSITIVITY: f32 = 1.8;
/// The client's own master volume (not a key of the original).
pub const VOLUME_KEY: &str = "pga3-client.volume";

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Pref {
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Debug, Default)]
pub struct PlayerPrefs {
    /// `None`: nothing is read or written (no home directory, tests).
    path: Option<PathBuf>,
    values: BTreeMap<String, Pref>,
}

/// The default location of the prefs file.
pub fn default_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("pga3-client/prefs.json"))
}

impl PlayerPrefs {
    /// Reads `path` (missing or unreadable: empty prefs that will be written there).
    pub fn load(path: Option<PathBuf>) -> PlayerPrefs {
        let mut values = BTreeMap::new();
        if let Some(path) = &path
            && let Ok(Value::Object(map)) = load_json(path)
        {
            for (key, value) in map {
                let pref = match value {
                    Value::Number(n) => match n.as_i64() {
                        Some(i) => Pref::Int(i as i32),
                        None => Pref::Float(n.as_f64().unwrap_or(0.0) as f32),
                    },
                    Value::String(s) => Pref::String(s),
                    // Unity has no other types; `null` was an unset name of older versions.
                    _ => continue,
                };
                values.insert(key, pref);
            }
        }
        PlayerPrefs { path, values }
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let mut out = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut serializer = serde_json::Serializer::with_formatter(&mut out, formatter);
        if self.values.serialize(&mut serializer).is_err() {
            return;
        }
        out.push(b'\n');
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(error) = std::fs::write(path, out) {
            eprintln!("prefs: writing {}: {error}", path.display());
        }
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// `GetInt`: the default when the key is missing or holds another type.
    pub fn get_int(&self, key: &str, default: i32) -> i32 {
        match self.values.get(key) {
            Some(Pref::Int(value)) => *value,
            _ => default,
        }
    }

    pub fn get_float(&self, key: &str, default: f32) -> f32 {
        match self.values.get(key) {
            Some(Pref::Float(value)) => *value,
            _ => default,
        }
    }

    pub fn get_string(&self, key: &str, default: &str) -> String {
        match self.values.get(key) {
            Some(Pref::String(value)) => value.clone(),
            _ => default.to_string(),
        }
    }

    fn set(&mut self, key: &str, value: Pref) {
        if self.values.get(key) == Some(&value) {
            return;
        }
        self.values.insert(key.to_string(), value);
        self.save();
    }

    pub fn set_int(&mut self, key: &str, value: i32) {
        self.set(key, Pref::Int(value));
    }

    pub fn set_float(&mut self, key: &str, value: f32) {
        self.set(key, Pref::Float(value));
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        self.set(key, Pref::String(value.to_string()));
    }

    /// `PlayerPrefs.SetInt(key, PlayerPrefs.GetInt(key) + 1)`.
    pub fn increment(&mut self, key: &str) {
        let value = self.get_int(key, 0).wrapping_add(1);
        self.set_int(key, value);
    }
}

/// One `QualitySettings` level (the fields that change what the client draws).
#[derive(Clone, Debug, PartialEq)]
pub struct QualityLevel {
    pub name: String,
    /// `masterTextureLimit`: mip levels skipped.
    pub texture_quality: u32,
}

/// `QualitySettings.names` and the level a player build starts with (`m_CurrentQuality`).
pub fn quality_levels(assets: &Path) -> (Vec<QualityLevel>, usize) {
    let Ok(json) = load_json(&assets.join("data/settings/QualitySettings.json")) else {
        return (Vec::new(), 0);
    };
    let data = json.get("data").unwrap_or(&json);
    let levels: Vec<QualityLevel> = data["m_QualitySettings"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|level| QualityLevel {
            name: level["name"].as_str().unwrap_or_default().to_string(),
            texture_quality: f32_of(&level["textureQuality"]) as u32,
        })
        .collect();
    let current = data["m_CurrentQuality"].as_u64().unwrap_or(0) as usize;
    let current = current.min(levels.len().saturating_sub(1));
    (levels, current)
}

/// What the menus and the game share for the whole run.
#[derive(Debug)]
pub struct Settings {
    pub prefs: PlayerPrefs,
    /// `RoomMultiplayerMenu.sens` (a static: it keeps its value between rooms).
    pub sens: f32,
    pub quality_levels: Vec<QualityLevel>,
    /// `QualitySettings.GetQualityLevel()` (not saved).
    pub quality: usize,
    /// Client master volume 0..1.
    pub volume: f32,
}

pub type SharedSettings = Rc<RefCell<Settings>>;

impl Settings {
    pub fn new(assets: &Path, prefs: PlayerPrefs) -> Settings {
        let (quality_levels, quality) = quality_levels(assets);
        let volume = prefs.get_float(VOLUME_KEY, 1.0).clamp(0.0, 1.0);
        Settings {
            prefs,
            sens: DEFAULT_SENSITIVITY,
            quality_levels,
            quality,
            volume,
        }
    }

    /// `masterTextureLimit` of the current level.
    pub fn texture_limit(&self) -> u32 {
        self.quality_levels
            .get(self.quality)
            .map_or(0, |l| l.texture_quality)
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.prefs.set_float(VOLUME_KEY, self.volume);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::find_assets_dir;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pga3-prefs-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("prefs.json")
    }

    #[test]
    fn types_survive_a_reload() {
        let path = temp_path("types");
        let mut prefs = PlayerPrefs::load(Some(path.clone()));
        assert!(!prefs.has_key("sens"));
        prefs.set_float("sens", 1.8);
        prefs.set_int("Kills", 3);
        prefs.increment("Kills");
        prefs.increment("Deaths");
        prefs.set_string("PlayerName", "Tester");
        prefs.set_float("whole", 2.0);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"sens\": 1.8"), "{text}");
        assert!(text.contains("\"whole\": 2.0"), "{text}");
        let prefs = PlayerPrefs::load(Some(path.clone()));
        assert_eq!(prefs.get_float("sens", 0.0), 1.8);
        assert_eq!(prefs.get_int("Kills", 0), 4);
        assert_eq!(prefs.get_int("Deaths", 0), 1);
        assert_eq!(prefs.get_string("PlayerName", ""), "Tester");
        assert_eq!(prefs.get_float("whole", 0.0), 2.0);
        // Another type than asked: the default (Unity's typed storage).
        assert_eq!(prefs.get_int("sens", 7), 7);
        assert_eq!(prefs.get_float("Kills", 0.5), 0.5);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn older_files_load() {
        let path = temp_path("old");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "{ \"PlayerName\": null, \"Clan\": \"\", \"Kills\": 5, \"GamePlays\": 2 }",
        )
        .unwrap();
        let prefs = PlayerPrefs::load(Some(path.clone()));
        assert!(!prefs.has_key("PlayerName"));
        assert_eq!(prefs.get_int("Kills", 0), 5);
        assert_eq!(prefs.get_string("Clan", "x"), "");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn quality_levels_of_the_build() {
        let assets = find_assets_dir().unwrap();
        let (levels, current) = quality_levels(&assets);
        let names: Vec<&str> = levels.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Fastest",
                "Fast",
                "Simple",
                "Good",
                "Beautiful",
                "Fantastic"
            ]
        );
        assert_eq!(current, 3);
        let limits: Vec<u32> = levels.iter().map(|l| l.texture_quality).collect();
        assert_eq!(limits, [1, 2, 2, 1, 0, 1]);
    }
}
