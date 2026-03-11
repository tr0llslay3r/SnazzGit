use crate::git::credentials;

#[tauri::command]
pub async fn store_credentials(
    url: String,
    username: String,
    password: String,
) -> Result<(), String> {
    let creds = credentials::Credentials {
        username: username.clone(),
        password,
    };
    tokio::task::spawn_blocking(move || credentials::store_credentials(&url, &creds))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_stored_credentials(url: String, username: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || credentials::delete_credentials(&url, &username))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
