use crate::git::clone;
use crate::git::credentials::Credentials;
use tauri::AppHandle;

#[tauri::command]
pub async fn clone_repo(
    app: AppHandle,
    url: String,
    path: String,
    credentials: Option<Credentials>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || clone::clone_repo(&url, &path, credentials, Some(&app)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
