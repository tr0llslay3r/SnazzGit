use crate::git::blame;
use crate::git::types::BlameInfo;

#[tauri::command]
pub async fn get_blame(path: String, file_path: String) -> Result<BlameInfo, String> {
    tokio::task::spawn_blocking(move || blame::get_blame(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_file_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        std::fs::write(dir.path().join("blame_me.txt"), "line one\nline two\n").unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("blame_me.txt"))
            .unwrap();
        index.write().unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();
        (dir, path)
    }

    #[tokio::test]
    async fn test_get_blame_valid() {
        let (_dir, path) = init_repo_with_file_commit();
        let result = get_blame(path, "blame_me.txt".into()).await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.lines.is_empty());
        assert_eq!(info.lines[0].author, "Test User");
    }

    #[tokio::test]
    async fn test_get_blame_nonexistent_file() {
        let (_dir, path) = init_repo_with_file_commit();
        let result = get_blame(path, "no_such_file.txt".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_blame_invalid_repo() {
        let result = get_blame("/nonexistent".into(), "file.txt".into()).await;
        assert!(result.is_err());
    }
}
