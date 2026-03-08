use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::git::error::GitError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

fn themes_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "snazzgit", "snazzgit").map(|dirs| dirs.config_dir().join("themes"))
}

pub fn load_user_themes() -> Result<Vec<Theme>, GitError> {
    let dir = match themes_dir() {
        Some(d) => d,
        None => return Ok(Vec::new()),
    };

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut themes = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(theme) = serde_json::from_str::<Theme>(&content) {
                    themes.push(theme);
                }
            }
        }
    }

    Ok(themes)
}

pub fn save_user_theme(theme: &Theme) -> Result<(), GitError> {
    let dir = themes_dir().ok_or_else(|| GitError::General("Cannot determine config directory".into()))?;
    fs::create_dir_all(&dir)?;

    let filename = theme
        .name
        .to_lowercase()
        .replace(' ', "-")
        + ".json";
    let path = dir.join(filename);
    let content = serde_json::to_string_pretty(theme)
        .map_err(|e| GitError::General(e.to_string()))?;
    fs::write(path, content)?;
    Ok(())
}

pub fn delete_user_theme(name: &str) -> Result<(), GitError> {
    let dir = themes_dir().ok_or_else(|| GitError::General("Cannot determine config directory".into()))?;
    let filename = name.to_lowercase().replace(' ', "-") + ".json";
    let path = dir.join(filename);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
