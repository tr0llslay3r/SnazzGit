use crate::git::submodule;
use crate::git::types::SubmoduleInfo;

#[tauri::command]
pub async fn list_submodules(path: String) -> Result<Vec<SubmoduleInfo>, String> {
    tokio::task::spawn_blocking(move || submodule::list_submodules(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
        (dir, path)
    }

    #[tokio::test]
    async fn test_list_submodules_command_empty() {
        let (_dir, path) = init_repo_with_commit();
        let result = list_submodules(path).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_submodules_command_invalid_path() {
        let result = list_submodules("/nonexistent".into()).await;
        assert!(result.is_err());
    }
}
