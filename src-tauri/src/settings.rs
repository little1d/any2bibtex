use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default, rename = "semanticScholarApiKey")]
    pub semantic_scholar_api_key: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            semantic_scholar_api_key: None,
            theme: default_theme(),
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

pub fn get_settings() -> io::Result<Settings> {
    let path = settings_path()?;
    let Ok(content) = fs::read_to_string(path) else {
        return Ok(Settings::default());
    };
    let mut settings = serde_json::from_str::<Settings>(&content).unwrap_or_default();
    if settings.theme != "light" {
        settings.theme = "dark".to_string();
    }
    Ok(settings)
}

pub fn save_settings_patch(
    semantic_scholar_api_key: Option<String>,
    theme: Option<String>,
) -> io::Result<Settings> {
    let mut settings = get_settings()?;
    if let Some(api_key) = semantic_scholar_api_key {
        settings.semantic_scholar_api_key = if api_key.trim().is_empty() {
            None
        } else {
            Some(api_key)
        };
    }
    if let Some(next_theme) = theme {
        settings.theme = if next_theme == "light" {
            "light".to_string()
        } else {
            "dark".to_string()
        };
    }

    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(path, content)?;
    Ok(settings)
}

pub fn normalize_semantic_scholar_api_key(value: &str) -> String {
    let mut text = value.trim().to_string();
    if text.to_lowercase().starts_with("export ") {
        text = text[7..].trim().to_string();
    }
    let lowered = text.to_lowercase();
    if lowered.starts_with("semantic_scholar_api_key") {
        if let Some((_, rhs)) = text.split_once('=') {
            text = rhs.trim().to_string();
        }
    }
    text.trim_matches(['\'', '"']).trim().to_string()
}

fn settings_path() -> io::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "could not determine config directory")
    })?;
    Ok(base.join("any2bibtex").join("settings.json"))
}
