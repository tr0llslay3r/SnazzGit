use crate::git::stash;
use crate::git::types::StashEntry;

#[tauri::command]
pub async fn stash_list(path: String) -> Result<Vec<StashEntry>, String> {
    tokio::task::spawn_blocking(move || stash::stash_list(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_save(path: String, message: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_save(&path, &message))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_pop(path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_pop(&path, index))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_apply(path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_apply(&path, index))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stash_drop(path: String, index: usize) -> Result<(), String> {
    tokio::task::spawn_blocking(move || stash::stash_drop(&path, index))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a repo with a committed tracked file, then modifies it so
    /// the working tree is dirty (required for stash_save with default flags).
    fn init_repo_dirty() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();
        std::fs::write(dir.path().join("file.txt"), "original").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("file.txt")).unwrap();
        index.write().unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();
        std::fs::write(dir.path().join("file.txt"), "modified").unwrap();
        (dir, path)
    }

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();
        (dir, path)
    }

    #[tokio::test]
    async fn test_stash_list_empty() {
        let (_dir, path) = init_repo_with_commit();
        let result = stash_list(path).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stash_save_and_list() {
        let (_dir, path) = init_repo_dirty();
        stash_save(path.clone(), "my stash".into()).await.unwrap();
        let list = stash_list(path).await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].message.contains("my stash"));
    }

    #[tokio::test]
    async fn test_stash_drop() {
        let (_dir, path) = init_repo_dirty();
        stash_save(path.clone(), "drop me".into()).await.unwrap();
        let result = stash_drop(path.clone(), 0).await;
        assert!(result.is_ok());
        assert!(stash_list(path).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stash_pop() {
        let (_dir, path) = init_repo_dirty();
        stash_save(path.clone(), "pop me".into()).await.unwrap();
        let result = stash_pop(path.clone(), 0).await;
        assert!(result.is_ok());
        assert!(stash_list(path).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_stash_apply() {
        let (_dir, path) = init_repo_dirty();
        stash_save(path.clone(), "apply me".into()).await.unwrap();
        let result = stash_apply(path.clone(), 0).await;
        assert!(result.is_ok());
        // apply keeps the stash entry
        assert_eq!(stash_list(path).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_stash_save_invalid_path() {
        let result = stash_save("/nonexistent".into(), "msg".into()).await;
        assert!(result.is_err());
    }
}
