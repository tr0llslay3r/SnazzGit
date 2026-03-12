use crate::recent::{self, RecentRepo};

#[tauri::command]
pub async fn load_recent_repos() -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(recent::load_recent_repos)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_recent_repo(path: String, name: String) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recent::add_recent_repo(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_recent_repo(path: String) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recent::remove_recent_repo(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_recent_repos_ok() {
        let home = tempfile::TempDir::new().unwrap();
        unsafe { std::env::set_var("HOME", home.path()) };
        let result = load_recent_repos().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_and_remove_recent_repo() {
        let home = tempfile::TempDir::new().unwrap();
        unsafe { std::env::set_var("HOME", home.path()) };

        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let result = add_recent_repo(path.clone(), "Test Repo".into()).await;
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert!(repos.iter().any(|r| r.path == path));

        let result = remove_recent_repo(path.clone()).await;
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert!(!repos.iter().any(|r| r.path == path));
    }

    #[tokio::test]
    async fn test_add_recent_repo_deduplicates() {
        let home = tempfile::TempDir::new().unwrap();
        unsafe { std::env::set_var("HOME", home.path()) };

        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        add_recent_repo(path.clone(), "First".into()).await.unwrap();
        let repos = add_recent_repo(path.clone(), "Updated".into()).await.unwrap();
        assert_eq!(repos.iter().filter(|r| r.path == path).count(), 1);
        assert_eq!(repos[0].path, path);
    }
}
