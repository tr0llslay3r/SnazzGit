use git2::{Repository, Sort};

use super::error::GitError;
use super::types::CommitInfo;

pub fn search_commits(
    path: &str,
    query: &str,
    max_results: usize,
) -> Result<Vec<CommitInfo>, GitError> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    revwalk.push_head()?;

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for oid_result in revwalk {
        if results.len() >= max_results {
            break;
        }
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let id_str = oid.to_string();

        let matches = message.to_lowercase().contains(&query_lower)
            || author_name.to_lowercase().contains(&query_lower)
            || author_email.to_lowercase().contains(&query_lower)
            || id_str.starts_with(&query_lower);

        if matches {
            let committer = commit.committer();
            results.push(CommitInfo {
                id: id_str.clone(),
                short_id: id_str[..8.min(id_str.len())].to_string(),
                message,
                summary: commit.summary().unwrap_or("").to_string(),
                author_name,
                author_email,
                author_time: author.when().seconds(),
                committer_name: committer.name().unwrap_or("").to_string(),
                committer_time: committer.when().seconds(),
                parent_ids: commit.parent_ids().map(|p| p.to_string()).collect(),
                refs: Vec::new(),
            });
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commits() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let alice = git2::Signature::now("Alice Smith", "alice@example.com").unwrap();
        let bob = git2::Signature::now("Bob Jones", "bob@example.com").unwrap();

        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let c1 = repo
            .commit(Some("HEAD"), &alice, &alice, "Add feature foo", &tree, &[])
            .unwrap();
        let parent1 = repo.find_commit(c1).unwrap();
        let tree_id2 = repo.index().unwrap().write_tree().unwrap();
        let tree2 = repo.find_tree(tree_id2).unwrap();
        let c2 = repo
            .commit(Some("HEAD"), &bob, &bob, "Fix bug bar", &tree2, &[&parent1])
            .unwrap();
        let parent2 = repo.find_commit(c2).unwrap();
        let tree_id3 = repo.index().unwrap().write_tree().unwrap();
        let tree3 = repo.find_tree(tree_id3).unwrap();
        repo.commit(Some("HEAD"), &alice, &alice, "Refactor baz", &tree3, &[&parent2])
            .unwrap();

        (dir, path)
    }

    #[test]
    fn test_search_by_message_keyword() {
        let (_dir, path) = init_repo_with_commits();
        let results = search_commits(&path, "feature", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].summary.contains("feature"));
    }

    #[test]
    fn test_search_by_author_name() {
        let (_dir, path) = init_repo_with_commits();
        let results = search_commits(&path, "alice", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_author_email() {
        let (_dir, path) = init_repo_with_commits();
        let results = search_commits(&path, "bob@example.com", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].summary.contains("bug"));
    }

    #[test]
    fn test_search_no_match() {
        let (_dir, path) = init_repo_with_commits();
        let results = search_commits(&path, "zzz_no_match_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_respects_max_results() {
        let (_dir, path) = init_repo_with_commits();
        // "a" matches all commits (alice, bar, baz, add, etc.)
        let results = search_commits(&path, "a", 2).unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let (_dir, path) = init_repo_with_commits();
        let lower = search_commits(&path, "alice", 10).unwrap();
        let upper = search_commits(&path, "ALICE", 10).unwrap();
        assert_eq!(lower.len(), upper.len());
    }

    #[test]
    fn test_search_by_sha_prefix() {
        let (_dir, path) = init_repo_with_commits();
        let all = search_commits(&path, "a", 20).unwrap();
        assert!(!all.is_empty());
        // Search by the first 4 chars of the first result's id
        let prefix = &all[0].id[..4];
        let by_sha = search_commits(&path, prefix, 10).unwrap();
        assert!(!by_sha.is_empty());
        assert!(by_sha.iter().any(|c| c.id == all[0].id));
    }
}
