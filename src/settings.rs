use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

pub const MAX_TOP_MARGIN: f32 = 200.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Monitor device name (e.g. `\\.\DISPLAY2`); `None` = primary.
    pub monitor: Option<String>,
    /// Gap between the top of the work area and the capsule, in DIP.
    pub top_margin: f32,
    pub visible: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { monitor: None, top_margin: 8.0, visible: true }
    }
}

impl Settings {
    fn sanitized(mut self) -> Self {
        self.top_margin = if self.top_margin.is_finite() {
            self.top_margin.clamp(0.0, MAX_TOP_MARGIN)
        } else {
            Settings::default().top_margin
        };
        self
    }
}

pub fn default_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    base.join("DynamicIsland").join("settings.json")
}

/// Missing → defaults. Invalid JSON → defaults, original preserved as `.bak`.
pub fn load(path: &Path) -> Settings {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Settings::default(),
        Err(e) => {
            crate::log::warn(&format!("settings unreadable: {e}"));
            return Settings::default();
        }
    };
    match serde_json::from_str::<Settings>(&text) {
        Ok(s) => s.sanitized(),
        Err(e) => {
            crate::log::warn(&format!("settings invalid ({e}); using defaults"));
            let _ = fs::copy(path, path.with_extension("json.bak"));
            Settings::default()
        }
    }
}

/// Atomic write: temp file + rename (replaces existing on Windows).
pub fn save(path: &Path, s: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(s)?)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("di-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn missing_file_gives_defaults() {
        let d = tmp_dir("missing");
        assert_eq!(load(&d.join("settings.json")), Settings::default());
    }

    #[test]
    fn roundtrip_and_partial_json() {
        let d = tmp_dir("roundtrip");
        let p = d.join("settings.json");
        let s = Settings { monitor: Some(r"\\.\DISPLAY2".into()), top_margin: 20.0, visible: false };
        save(&p, &s).unwrap();
        assert_eq!(load(&p), s);
        fs::write(&p, r#"{"visible": false}"#).unwrap();
        assert_eq!(load(&p), Settings { visible: false, ..Settings::default() });
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let d = tmp_dir("corrupt");
        let p = d.join("settings.json");
        fs::write(&p, "{not json").unwrap();
        assert_eq!(load(&p), Settings::default());
        assert_eq!(fs::read_to_string(p.with_extension("json.bak")).unwrap(), "{not json");
    }

    #[test]
    fn margin_is_clamped() {
        let d = tmp_dir("clamp");
        let p = d.join("settings.json");
        fs::write(&p, r#"{"top_margin": -5}"#).unwrap();
        assert_eq!(load(&p).top_margin, 0.0);
        fs::write(&p, r#"{"top_margin": 999}"#).unwrap();
        assert_eq!(load(&p).top_margin, MAX_TOP_MARGIN);
    }
}
