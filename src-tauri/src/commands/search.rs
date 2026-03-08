use crate::git::search;
use crate::git::types::CommitInfo;

#[tauri::command]
pub async fn search_commits(
    path: String,
    query: String,
    max_results: usize,
) -> Result<Vec<CommitInfo>, String> {
    tokio::task::spawn_blocking(move || search::search_commits(&path, &query, max_results))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}
