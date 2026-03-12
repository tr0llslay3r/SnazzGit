use crate::git::search;
use crate::git::types::CommitInfo;

#[tauri::command]
pub async fn search_commits(
    path: String,
    query: String,
    max_results: usize,
) -> Result<Vec<CommitInfo>, String> {
    tokio::task::spawn_blocking(move || search::search_commits(&path, &query, max_results))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commits() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let make = |repo: &git2::Repository, msg: &str, parents: &[&git2::Commit]| {
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, parents)
                .unwrap()
        };
        let oid1 = make(&repo, "feat: add login", &[]);
        let c1 = repo.find_commit(oid1).unwrap();
        let oid2 = make(&repo, "fix: resolve logout bug", &[&c1]);
        let c2 = repo.find_commit(oid2).unwrap();
        make(&repo, "chore: update deps", &[&c2]);
        (dir, path)
    }

    #[tokio::test]
    async fn test_search_commits_matching() {
        let (_dir, path) = init_repo_with_commits();
        let result = search_commits(path, "login".into(), 10).await;
        assert!(result.is_ok());
        let commits = result.unwrap();
        assert_eq!(commits.len(), 1);
        assert!(commits[0].summary.contains("login"));
    }

    #[tokio::test]
    async fn test_search_commits_no_match() {
        let (_dir, path) = init_repo_with_commits();
        let result = search_commits(path, "zzznomatch".into(), 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_commits_respects_max_results() {
        let (_dir, path) = init_repo_with_commits();
        // "e" appears in all three commit messages
        let result = search_commits(path, "e".into(), 2).await;
        assert!(result.is_ok());
        assert!(result.unwrap().len() <= 2);
    }

    #[tokio::test]
    async fn test_search_commits_invalid_path() {
        let result = search_commits("/nonexistent".into(), "query".into(), 10).await;
        assert!(result.is_err());
    }
}
