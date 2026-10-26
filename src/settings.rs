//! User settings, stored as JSON in the platform's config folder.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Languages the UI is translated into: code, English name, native name
pub const LANGUAGES: &[(&str, &str, &str)] = &[("en", "English", "English"), ("nl", "Dutch", "Nederlands")];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Language code from `LANGUAGES`; `None` follows the system language
    #[serde(default)]
    pub language: Option<String>,
}

impl Settings {
    /// Location of the settings file, e.g. `~/.config/grandshop/settings.json` on Linux.
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "grandshop", "grandshop")
            .map(|dirs| dirs.config_dir().join("settings.json"))
    }

    /// Missing or unreadable settings give the defaults.
    pub fn load(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)
    }

    /// The language to show: the saved one, else the system language if translated, else English.
    pub fn effective_language(&self) -> &'static str {
        let wanted = self.language.clone().or_else(sys_locale::get_locale).unwrap_or_default();
        // "nl-BE" and "nl_BE.UTF-8" both mean Dutch
        let base = wanted.split(['-', '_', '.', '@']).next().unwrap_or_default();
        LANGUAGES
            .iter()
            .find(|(code, _, _)| code.eq_ignore_ascii_case(base))
            .map_or("en", |(code, _, _)| code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(Settings::load(&dir.path().join("nope.json")), Settings::default());
    }

    #[test]
    fn corrupt_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, b"{ not json").unwrap();
        assert_eq!(Settings::load(&path), Settings::default());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/settings.json");
        let settings = Settings { language: Some("nl".into()) };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
    }

    #[test]
    fn saved_language_wins_and_regions_are_ignored() {
        assert_eq!(Settings { language: Some("nl".into()) }.effective_language(), "nl");
        assert_eq!(Settings { language: Some("nl-BE".into()) }.effective_language(), "nl");
        assert_eq!(Settings { language: Some("fr".into()) }.effective_language(), "en");
    }
}
