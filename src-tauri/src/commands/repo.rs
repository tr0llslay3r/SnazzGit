use crate::git::repository;
use crate::git::types::RepoInfo;

#[tauri::command]
pub async fn get_cli_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

#[tauri::command]
pub async fn open_repository(path: String) -> Result<RepoInfo, String> {
    tokio::task::spawn_blocking(move || repository::open_repo(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn init_repository(path: String) -> Result<RepoInfo, String> {
    tokio::task::spawn_blocking(move || {
        git2::Repository::init(&path).map_err(crate::git::error::GitError::from)?;
        repository::open_repo(&path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
