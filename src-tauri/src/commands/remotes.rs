use crate::git::remote;
use tauri::AppHandle;

#[tauri::command]
pub async fn fetch_remote(
    app: AppHandle,
    path: String,
    remote_name: String,
) -> Result<(), String> {
    let handle = app.clone();
    tokio::task::spawn_blocking(move || remote::fetch_remote(&path, &remote_name, Some(&handle)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pull(
    app: AppHandle,
    path: String,
    remote_name: String,
) -> Result<String, String> {
    let handle = app.clone();
    tokio::task::spawn_blocking(move || remote::pull(&path, &remote_name, Some(&handle)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push(
    app: AppHandle,
    path: String,
    remote_name: String,
) -> Result<(), String> {
    let handle = app.clone();
    tokio::task::spawn_blocking(move || remote::push(&path, &remote_name, Some(&handle)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
