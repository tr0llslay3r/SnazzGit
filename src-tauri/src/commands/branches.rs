use crate::git::branch;

#[tauri::command]
pub async fn create_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::create_branch(&path, &name, true))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn checkout_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::checkout_branch(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::delete_branch(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_branch(
    path: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::rename_branch(&path, &old_name, &new_name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_branch(path: String, source_branch: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || branch::merge_branch(&path, &source_branch))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_to_commit(path: String, commit_id: String, mode: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::reset_to_commit(&path, &commit_id, &mode))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
