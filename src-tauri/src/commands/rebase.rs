use crate::git::rebase;

#[tauri::command]
pub async fn rebase_onto(path: String, upstream_ref: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || rebase::rebase_onto(&path, &upstream_ref))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rebase_abort(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || rebase::rebase_abort(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rebase_continue(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || rebase::rebase_continue(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn squash_commits(
    path: String,
    count: usize,
    message: String,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || rebase::squash_commits(&path, count, &message))
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
        let repo = git2::Repository::init(&path).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();
        (dir, path)
    }

    fn make_commit(path: &str, msg: &str, file: &str, content: &str) -> git2::Oid {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        std::fs::write(std::path::Path::new(path).join(file), content).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(file)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs).unwrap()
    }

    #[tokio::test]
    async fn test_squash_commits_command() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First", "a.txt", "a\n");
        make_commit(&path, "Second", "b.txt", "b\n");
        make_commit(&path, "Third", "c.txt", "c\n");

        let result = squash_commits(path, 2, "Squashed".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 40);
    }

    #[tokio::test]
    async fn test_squash_commits_too_few_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Only", "a.txt", "a\n");
        let result = squash_commits(path, 1, "nope".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rebase_onto_invalid_path() {
        let result = rebase_onto("/nonexistent/path".into(), "main".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rebase_abort_no_rebase_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "a.txt", "a\n");
        let result = rebase_abort(path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rebase_continue_no_rebase_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "a.txt", "a\n");
        let result = rebase_continue(path).await;
        assert!(result.is_err());
    }
}
