use crate::git::commit;
use crate::git::graph;
use crate::git::types::{CommitInfo, GraphRow};

#[tauri::command]
pub async fn load_commits(
    path: String,
    limit: usize,
    offset: usize,
) -> Result<Vec<CommitInfo>, String> {
    tokio::task::spawn_blocking(move || commit::load_commits(&path, limit, offset))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_detail(path: String, commit_id: String) -> Result<CommitInfo, String> {
    tokio::task::spawn_blocking(move || commit::get_commit_detail(&path, &commit_id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compute_graph(commits: Vec<CommitInfo>) -> Result<Vec<GraphRow>, String> {
    tokio::task::spawn_blocking(move || graph::compute_graph(&commits))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_commit(
    path: String,
    message: String,
    amend: bool,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let repo = git2::Repository::open(&path).map_err(|e| e.to_string())?;
        let sig = repo.signature().map_err(|e| e.to_string())?;
        let mut index = repo.index().map_err(|e| e.to_string())?;
        let tree_oid = index.write_tree().map_err(|e| e.to_string())?;
        let tree = repo.find_tree(tree_oid).map_err(|e| e.to_string())?;

        if amend {
            let head = repo.head().map_err(|e| e.to_string())?;
            let head_commit = head.peel_to_commit().map_err(|e| e.to_string())?;
            let oid = head_commit
                .amend(
                    Some("HEAD"),
                    Some(&sig),
                    Some(&sig),
                    None,
                    Some(&message),
                    Some(&tree),
                )
                .map_err(|e| e.to_string())?;
            Ok(oid.to_string())
        } else {
            let parents = if let Ok(head) = repo.head() {
                vec![head.peel_to_commit().map_err(|e| e.to_string())?]
            } else {
                vec![]
            };
            let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_refs)
                .map_err(|e| e.to_string())?;
            Ok(oid.to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())?
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

    fn set_user_config(path: &str) {
        let repo = git2::Repository::open(path).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();
    }

    fn make_commit(path: &str, msg: &str) -> git2::Oid {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
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
    async fn test_load_commits_empty_repo() {
        let (_dir, path) = init_repo();
        let result = load_commits(path, 100, 0).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_load_commits_single() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First commit");
        let result = load_commits(path, 100, 0).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_load_commits_respects_limit() {
        let (_dir, path) = init_repo();
        make_commit(&path, "A");
        make_commit(&path, "B");
        make_commit(&path, "C");
        let result = load_commits(path, 2, 0).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_load_commits_invalid_path() {
        let result = load_commits("/nonexistent/path".into(), 100, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_commit_detail_valid() {
        let (_dir, path) = init_repo();
        let oid = make_commit(&path, "Detail test");
        let oid_str = oid.to_string();
        let result = get_commit_detail(path, oid_str.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, oid_str);
    }

    #[tokio::test]
    async fn test_get_commit_detail_invalid_id() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Seed");
        let result =
            get_commit_detail(path, "0000000000000000000000000000000000000000".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_graph_empty() {
        let result = compute_graph(vec![]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_compute_graph_with_commits() {
        let (_dir, path) = init_repo();
        make_commit(&path, "A");
        make_commit(&path, "B");
        let commits = load_commits(path, 100, 0).await.unwrap();
        let result = compute_graph(commits).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_create_commit_initial() {
        let (_dir, path) = init_repo();
        set_user_config(&path);
        let result = create_commit(path, "Initial commit".into(), false).await;
        assert!(result.is_ok());
        // Returns the new commit OID as a hex string
        assert_eq!(result.unwrap().len(), 40);
    }

    #[tokio::test]
    async fn test_create_commit_second() {
        let (_dir, path) = init_repo();
        set_user_config(&path);
        create_commit(path.clone(), "First".into(), false)
            .await
            .unwrap();
        let result = create_commit(path, "Second".into(), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_commit_amend() {
        let (_dir, path) = init_repo();
        set_user_config(&path);
        create_commit(path.clone(), "Original".into(), false)
            .await
            .unwrap();
        let result = create_commit(path, "Amended".into(), true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_commit_amend_on_empty_repo_fails() {
        let (_dir, path) = init_repo();
        set_user_config(&path);
        // amend=true with no HEAD should fail
        let result = create_commit(path, "Amend nothing".into(), true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_commit_invalid_path() {
        let result = create_commit("/nonexistent/path".into(), "msg".into(), false).await;
        assert!(result.is_err());
    }
}
