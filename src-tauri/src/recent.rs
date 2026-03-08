use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::git::error::GitError;

const MAX_RECENT: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRepo {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RecentReposFile {
    repos: Vec<RecentRepo>,
}

fn config_file() -> Option<PathBuf> {
    ProjectDirs::from("com", "snazzgit", "snazzgit")
        .map(|dirs| dirs.config_dir().join("recent-repos.json"))
}

pub fn load_recent_repos() -> Result<Vec<RecentRepo>, GitError> {
    let Some(path) = config_file() else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let file: RecentReposFile =
        serde_json::from_str(&content).map_err(|e| GitError::General(e.to_string()))?;
    Ok(file.repos)
}

pub fn add_recent_repo(path: &str, name: &str) -> Result<Vec<RecentRepo>, GitError> {
    let mut repos = load_recent_repos().unwrap_or_default();

    // Remove existing entry with the same path
    repos.retain(|r| r.path != path);

    // Insert at front
    repos.insert(
        0,
        RecentRepo {
            path: path.to_string(),
            name: name.to_string(),
        },
    );

    // Limit size
    repos.truncate(MAX_RECENT);

    // Save
    let config_path =
        config_file().ok_or_else(|| GitError::General("Cannot determine config directory".into()))?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = RecentReposFile {
        repos: repos.clone(),
    };
    let content =
        serde_json::to_string_pretty(&file).map_err(|e| GitError::General(e.to_string()))?;
    fs::write(config_path, content)?;

    Ok(repos)
}

pub fn remove_recent_repo(path: &str) -> Result<Vec<RecentRepo>, GitError> {
    let mut repos = load_recent_repos().unwrap_or_default();
    repos.retain(|r| r.path != path);

    let config_path =
        config_file().ok_or_else(|| GitError::General("Cannot determine config directory".into()))?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = RecentReposFile {
        repos: repos.clone(),
    };
    let content =
        serde_json::to_string_pretty(&file).map_err(|e| GitError::General(e.to_string()))?;
    fs::write(config_path, content)?;

    Ok(repos)
}
