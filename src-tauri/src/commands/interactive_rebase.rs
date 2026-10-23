use crate::git::interactive_rebase;
use crate::git::types::{InteractiveRebaseStatus, RebaseTodoItem};

#[tauri::command]
pub async fn start_interactive_rebase(
    path: String,
    base_commit_id: String,
    todo: Vec<RebaseTodoItem>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        interactive_rebase::start_interactive_rebase(&path, &base_commit_id, todo)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn interactive_rebase_continue(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || interactive_rebase::interactive_rebase_continue(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn interactive_rebase_abort(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || interactive_rebase::interactive_rebase_abort(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_interactive_rebase_status(path: String) -> Result<InteractiveRebaseStatus, String> {
    tokio::task::spawn_blocking(move || interactive_rebase::get_interactive_rebase_status(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::RebaseAction;

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
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs)
            .unwrap()
    }

    #[tokio::test]
    async fn test_start_interactive_rebase_invalid_path_errors() {
        let todo = vec![RebaseTodoItem {
            commit_id: "deadbeef".repeat(5),
            action: RebaseAction::Pick,
            message: None,
        }];
        let result =
            start_interactive_rebase("/nonexistent/path".into(), "deadbeef".repeat(5), todo).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_interactive_rebase_reorder_end_to_end() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");
        let b = make_commit(&path, "B", "b.txt", "b\n");

        let todo = vec![
            RebaseTodoItem {
                commit_id: b.to_string(),
                action: RebaseAction::Pick,
                message: None,
            },
            RebaseTodoItem {
                commit_id: a.to_string(),
                action: RebaseAction::Pick,
                message: None,
            },
        ];
        let result = start_interactive_rebase(path, base.to_string(), todo).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Rebase complete"));
    }

    #[tokio::test]
    async fn test_interactive_rebase_continue_no_state_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "a.txt", "a\n");
        let result = interactive_rebase_continue(path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_interactive_rebase_abort_no_state_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "a.txt", "a\n");
        let result = interactive_rebase_abort(path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_interactive_rebase_status_inactive_by_default() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "a.txt", "a\n");
        let result = get_interactive_rebase_status(path).await.unwrap();
        assert!(!result.active);
    }

    #[tokio::test]
    async fn test_start_interactive_rebase_dirty_tree_errors() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");
        std::fs::write(std::path::Path::new(&path).join("dirty.txt"), "oops\n").unwrap();

        let todo = vec![RebaseTodoItem {
            commit_id: a.to_string(),
            action: RebaseAction::Pick,
            message: None,
        }];
        let result = start_interactive_rebase(path, base.to_string(), todo).await;
        assert!(result.is_err());
    }
}
