use crate::git::status;
use crate::git::types::WorkingTreeStatus;

#[tauri::command]
pub async fn get_status(path: String) -> Result<WorkingTreeStatus, String> {
    tokio::task::spawn_blocking(move || status::get_status(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stage_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::stage_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unstage_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::unstage_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stage_all(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::stage_all(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unstage_all(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::unstage_all(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn discard_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::discard_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_file(path: String, file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::delete_file(&path, &file_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_gitignore(path: String, pattern: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || status::add_to_gitignore(&path, &pattern))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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

    fn write_file(repo_path: &str, name: &str, content: &str) {
        let full = std::path::Path::new(repo_path).join(name);
        let mut f = std::fs::File::create(full).unwrap();
        write!(f, "{}", content).unwrap();
    }

    #[tokio::test]
    async fn test_get_status_clean_repo() {
        let (_dir, path) = init_repo_with_commit();
        let result = get_status(path).await;
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.staged.is_empty());
        assert!(s.unstaged.is_empty());
    }

    #[tokio::test]
    async fn test_get_status_invalid_path() {
        let result = get_status("/nonexistent/path".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stage_file() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "new.txt", "hello");
        let result = stage_file(path.clone(), "new.txt".into()).await;
        assert!(result.is_ok());
        let s = get_status(path).await.unwrap();
        assert!(!s.staged.is_empty());
    }

    #[tokio::test]
    async fn test_unstage_file() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "staged.txt", "content");
        stage_file(path.clone(), "staged.txt".into()).await.unwrap();
        let result = unstage_file(path.clone(), "staged.txt".into()).await;
        assert!(result.is_ok());
        let s = get_status(path).await.unwrap();
        assert!(s.staged.is_empty());
    }

    #[tokio::test]
    async fn test_stage_all() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "a.txt", "aaa");
        write_file(&path, "b.txt", "bbb");
        let result = stage_all(path.clone()).await;
        assert!(result.is_ok());
        let s = get_status(path).await.unwrap();
        assert!(!s.staged.is_empty());
    }

    #[tokio::test]
    async fn test_unstage_all() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "a.txt", "aaa");
        stage_all(path.clone()).await.unwrap();
        let result = unstage_all(path.clone()).await;
        assert!(result.is_ok());
        let s = get_status(path).await.unwrap();
        assert!(s.staged.is_empty());
    }

    #[tokio::test]
    async fn test_add_to_gitignore_creates_file() {
        let (_dir, path) = init_repo_with_commit();
        let result = add_to_gitignore(path.clone(), "*.log".into()).await;
        assert!(result.is_ok());
        assert!(std::path::Path::new(&path).join(".gitignore").exists());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "to_delete.txt", "bye");
        let result = delete_file(path.clone(), "to_delete.txt".into()).await;
        assert!(result.is_ok());
        assert!(!std::path::Path::new(&path).join("to_delete.txt").exists());
    }

    #[tokio::test]
    async fn test_discard_file() {
        let (_dir, path) = init_repo_with_commit();
        // Commit a tracked file first
        write_file(&path, "tracked.txt", "original");
        stage_file(path.clone(), "tracked.txt".into()).await.unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Add file", &tree, &[&head])
            .unwrap();
        // Now modify and discard
        write_file(&path, "tracked.txt", "modified");
        let result = discard_file(path, "tracked.txt".into()).await;
        assert!(result.is_ok());
    }
}
