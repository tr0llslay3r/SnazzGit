use crate::git::blame;
use crate::git::types::BlameInfo;

#[tauri::command]
pub async fn get_blame(path: String, file_path: String) -> Result<BlameInfo, String> {
    tokio::task::spawn_blocking(move || blame::get_blame(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
