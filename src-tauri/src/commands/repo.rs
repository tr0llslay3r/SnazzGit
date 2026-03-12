use crate::git::repository;
use crate::git::types::RepoInfo;

#[tauri::command]
pub async fn get_cli_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

#[tauri::command]
pub async fn open_repository(path: String) -> Result<RepoInfo, String> {
    tokio::task::spawn_blocking(move || repository::open_repo(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn init_repository(path: String) -> Result<RepoInfo, String> {
    tokio::task::spawn_blocking(move || {
        git2::Repository::init(&path).map_err(crate::git::error::GitError::from)?;
        repository::open_repo(&path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        (dir, path)
    }

    #[tokio::test]
    async fn test_open_repository_valid() {
        let (_dir, path) = init_repo();
        let result = open_repository(path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_open_repository_invalid_path() {
        let result = open_repository("/tmp/nonexistent_snazzgit_xyz_99999".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_open_repository_returns_repo_info() {
        let (_dir, path) = init_repo();
        let info = open_repository(path.clone()).await.unwrap();
        assert!(!info.path.is_empty());
        assert!(!info.is_bare);
    }

    #[tokio::test]
    async fn test_init_repository() {
        let dir = tempfile::TempDir::new().unwrap();
        let new_path = dir.path().join("new_repo");
        std::fs::create_dir_all(&new_path).unwrap();
        let path_str = new_path.to_str().unwrap().to_string();
        let result = init_repository(path_str.clone()).await;
        assert!(result.is_ok());
        assert!(new_path.join(".git").exists());
    }

    #[tokio::test]
    async fn test_init_repository_already_a_repo() {
        let (_dir, path) = init_repo();
        // Re-initializing an existing repo is valid git behavior
        let result = init_repository(path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_cli_args() {
        let args = get_cli_args().await;
        // Just verify it returns without panicking; actual args vary by test runner
        let _ = args;
    }
}
