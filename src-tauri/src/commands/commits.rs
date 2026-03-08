use crate::git::commit;
use crate::git::graph;
use crate::git::types::{CommitInfo, GraphRow};

#[tauri::command]
pub async fn load_commits(
    path: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<CommitInfo>, String> {
    tokio::task::spawn_blocking(move || commit::load_commits(&path, limit, offset))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_detail(path: String, commit_id: String) -> Result<CommitInfo, String> {
    tokio::task::spawn_blocking(move || commit::get_commit_detail(&path, &commit_id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compute_graph(commits: Vec<CommitInfo>) -> Result<Vec<GraphRow>, String> {
    tokio::task::spawn_blocking(move || graph::compute_graph(&commits))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_commit(
    path: String,
    message: String,
    amend: bool,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let repo = git2::Repository::open(&path).map_err(|e| e.to_string())?;
        let sig = repo.signature().map_err(|e| e.to_string())?;
        let mut index = repo.index().map_err(|e| e.to_string())?;
        let tree_oid = index.write_tree().map_err(|e| e.to_string())?;
        let tree = repo.find_tree(tree_oid).map_err(|e| e.to_string())?;

        if amend {
            let head = repo.head().map_err(|e| e.to_string())?;
            let head_commit = head.peel_to_commit().map_err(|e| e.to_string())?;
            let oid = head_commit
                .amend(
                    Some("HEAD"),
                    Some(&sig),
                    Some(&sig),
                    None,
                    Some(&message),
                    Some(&tree),
                )
                .map_err(|e| e.to_string())?;
            Ok(oid.to_string())
        } else {
            let parents = if let Ok(head) = repo.head() {
                vec![head.peel_to_commit().map_err(|e| e.to_string())?]
            } else {
                vec![]
            };
            let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_refs)
                .map_err(|e| e.to_string())?;
            Ok(oid.to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
