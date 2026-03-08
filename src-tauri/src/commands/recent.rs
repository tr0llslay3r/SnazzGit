use crate::recent::{self, RecentRepo};

#[tauri::command]
pub async fn load_recent_repos() -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(recent::load_recent_repos)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_recent_repo(path: String, name: String) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recent::add_recent_repo(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_recent_repo(path: String) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recent::remove_recent_repo(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
