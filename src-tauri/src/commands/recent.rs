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

    // These scenarios all share a single HOME override deliberately, in one
    // test. std::env::set_var is process-global, and cargo test runs tests
    // in parallel threads within the same process — so when each scenario
    // was its own #[tokio::test] setting its own HOME, they raced: one
    // test's temp HOME could be overwritten (or even deleted, once that
    // test's TempDir guard dropped) by another mid-flight, intermittently
    // failing with an I/O error depending on thread scheduling.
    #[tokio::test]
    async fn test_recent_repos_roundtrip() {
        let home = tempfile::TempDir::new().unwrap();
        unsafe { std::env::set_var("HOME", home.path()) };

        let result = load_recent_repos().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let result = add_recent_repo(path.clone(), "Test Repo".into()).await;
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert!(repos.iter().any(|r| r.path == path));

        // Adding the same path again updates it in place rather than
        // duplicating it.
        let repos = add_recent_repo(path.clone(), "Updated".into()).await.unwrap();
        assert_eq!(repos.iter().filter(|r| r.path == path).count(), 1);
        assert_eq!(repos[0].path, path);
        assert_eq!(repos[0].name, "Updated");

        let result = remove_recent_repo(path.clone()).await;
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert!(!repos.iter().any(|r| r.path == path));
    }
}
