use git2::Repository;

use super::error::GitError;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ReflogEntry {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub action: String,
    pub committer: String,
    pub time: i64,
}

pub fn get_reflog(path: &str, limit: usize) -> Result<Vec<ReflogEntry>, GitError> {
    let repo = Repository::open(path)?;
    let reflog = repo.reflog("HEAD")?;

    let entries: Vec<ReflogEntry> = reflog
        .iter()
        .take(limit)
        .map(|entry| {
            let id = entry.id_new().to_string();
            let short_id = id[..8.min(id.len())].to_string();
            let message = entry.message().unwrap_or("").to_string();
            // Extract action (e.g., "commit", "checkout", "rebase") from reflog message
            let action = message.split(':').next().unwrap_or("").to_string();
            ReflogEntry {
                id,
                short_id,
                message,
                action,
                committer: entry.committer().name().unwrap_or("").to_string(),
                time: entry.committer().when().seconds(),
            }
        })
        .collect();

    Ok(entries)
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
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs).unwrap()
    }

    #[test]
    fn test_reflog_empty_repo() {
        let (_dir, path) = init_repo();
        let entries = get_reflog(&path, 100).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_reflog_with_commits() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First");
        make_commit(&path, "Second");
        let entries = get_reflog(&path, 100).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_reflog_respects_limit() {
        let (_dir, path) = init_repo();
        make_commit(&path, "A");
        make_commit(&path, "B");
        make_commit(&path, "C");
        let entries = get_reflog(&path, 2).unwrap();
        assert_eq!(entries.len(), 2);
    }
}
