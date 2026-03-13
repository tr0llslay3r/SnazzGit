use crate::git::credentials::Credentials;
use crate::git::remote;
use tauri::AppHandle;

#[tauri::command]
pub async fn force_push(
    app: AppHandle,
    path: String,
    remote_name: String,
    credentials: Option<Credentials>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        remote::force_push(&path, &remote_name, credentials, Some(&app))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_remote(path: String, name: String, url: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || remote::add_remote(&path, &name, &url))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_remote(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || remote::remove_remote(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_remote(
    path: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || remote::rename_remote(&path, &old_name, &new_name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_remote(
    app: AppHandle,
    path: String,
    remote_name: String,
    credentials: Option<Credentials>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        remote::fetch_remote(&path, &remote_name, credentials, Some(&app))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pull(
    app: AppHandle,
    path: String,
    remote_name: String,
    credentials: Option<Credentials>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        remote::pull(&path, &remote_name, credentials, Some(&app))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn push(
    app: AppHandle,
    path: String,
    remote_name: String,
    credentials: Option<Credentials>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        remote::push(&path, &remote_name, credentials, Some(&app))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
