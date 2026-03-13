use crate::git::reflog::{self, ReflogEntry};

#[tauri::command]
pub async fn get_reflog(path: String, limit: usize) -> Result<Vec<ReflogEntry>, String> {
    tokio::task::spawn_blocking(move || reflog::get_reflog(&path, limit))
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

    fn make_commit(path: &str, msg: &str) {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs).unwrap();
    }

    #[tokio::test]
    async fn test_get_reflog_empty_repo() {
        let (_dir, path) = init_repo();
        let result = get_reflog(path, 100).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_reflog_with_commits() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First");
        make_commit(&path, "Second");
        let result = get_reflog(path, 100).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_reflog_respects_limit() {
        let (_dir, path) = init_repo();
        make_commit(&path, "A");
        make_commit(&path, "B");
        make_commit(&path, "C");
        let result = get_reflog(path, 2).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_reflog_invalid_path() {
        let result = get_reflog("/nonexistent/path".into(), 100).await;
        assert!(result.is_err());
    }
}
