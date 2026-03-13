use git2::Repository;

use super::error::GitError;

/// Standard rebase: rebase current branch onto the given upstream ref.
pub fn rebase_onto(path: &str, upstream_ref: &str) -> Result<String, GitError> {
    let repo = Repository::open(path)?;

    let upstream = repo.revparse_single(upstream_ref)?;
    let upstream_annotated = repo.find_annotated_commit(upstream.id())?;

    let mut rebase = repo.rebase(None, Some(&upstream_annotated), None, None)?;

    let sig = repo.signature().or_else(|_| {
        git2::Signature::now("SnazzGit User", "snazzgit@local")
    })?;

    let mut count = 0;
    while rebase.next().is_some() {
        let index = repo.index()?;
        if index.has_conflicts() {
            return Ok(format!(
                "Rebase paused at step {} due to conflicts. Resolve conflicts and continue.",
                count + 1
            ));
        }
        rebase.commit(None, &sig, None)?;
        count += 1;
    }

    rebase.finish(Some(&sig))?;
    Ok(format!("Rebase complete: {} commit{} replayed", count, if count == 1 { "" } else { "s" }))
}

/// Abort an in-progress rebase.
pub fn rebase_abort(path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut rebase = repo.open_rebase(None)?;
    rebase.abort()?;
    Ok(())
}

/// Continue an in-progress rebase (after conflict resolution).
pub fn rebase_continue(path: &str) -> Result<String, GitError> {
    let repo = Repository::open(path)?;
    let sig = repo.signature().or_else(|_| {
        git2::Signature::now("SnazzGit User", "snazzgit@local")
    })?;

    let mut rebase = repo.open_rebase(None)?;

    // Commit the currently resolved step
    rebase.commit(None, &sig, None)?;

    let mut count = 1;
    while rebase.next().is_some() {
        let index = repo.index()?;
        if index.has_conflicts() {
            return Ok(format!(
                "Rebase paused at step {} due to conflicts. Resolve and continue.",
                count + 1
            ));
        }
        rebase.commit(None, &sig, None)?;
        count += 1;
    }

    rebase.finish(Some(&sig))?;
    Ok(format!("Rebase complete: {} remaining commit{} replayed", count, if count == 1 { "" } else { "s" }))
}

/// Squash the last N commits into one with the given message.
pub fn squash_commits(path: &str, count: usize, message: &str) -> Result<String, GitError> {
    if count < 2 {
        return Err(GitError::General("Must squash at least 2 commits".to_string()));
    }

    let repo = Repository::open(path)?;
    let head = repo.head()?.peel_to_commit()?;

    // Walk back `count - 1` parents to find the base
    let mut base = head.clone();
    for _ in 0..(count - 1) {
        base = base.parent(0).map_err(|_| {
            GitError::General("Not enough commits to squash".to_string())
        })?;
    }

    let sig = repo.signature().or_else(|_| {
        git2::Signature::now("SnazzGit User", "snazzgit@local")
    })?;

    // Create a new commit with the same tree as HEAD but parented on base's parent
    let tree = head.tree()?;
    let parents: Vec<git2::Commit> = if base.parent_count() > 0 {
        vec![base.parent(0)?]
    } else {
        vec![]
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    // Create commit without updating HEAD (since parent isn't current HEAD)
    let new_oid = repo.commit(None, &sig, &sig, message, &tree, &parent_refs)?;

    // Reset HEAD to the new commit
    let new_commit = repo.find_object(new_oid, None)?;
    repo.reset(&new_commit, git2::ResetType::Soft, None)?;

    Ok(new_oid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        // Set user config
        let repo = git2::Repository::open(&path).unwrap();
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

    #[test]
    fn test_squash_commits() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First", "a.txt", "a\n");
        make_commit(&path, "Second", "b.txt", "b\n");
        make_commit(&path, "Third", "c.txt", "c\n");

        let result = squash_commits(&path, 2, "Squashed").unwrap();
        assert_eq!(result.len(), 40); // OID hex

        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.message().unwrap(), "Squashed");
        // Should have one parent (the first commit)
        assert_eq!(head.parent_count(), 1);
    }

    #[test]
    fn test_squash_too_few() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First", "a.txt", "a\n");
        let result = squash_commits(&path, 1, "nope");
        assert!(result.is_err());
    }

    #[test]
    fn test_squash_not_enough_commits() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First", "a.txt", "a\n");
        let result = squash_commits(&path, 5, "nope");
        assert!(result.is_err());
    }

    #[test]
    fn test_rebase_onto_fast_forward() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "base.txt", "base\n");

        // Create a feature branch from base
        {
            let repo = git2::Repository::open(&path).unwrap();
            let head = repo.head().unwrap().peel_to_commit().unwrap();
            repo.branch("feature", &head, false).unwrap();
        }

        // Add commits on master
        make_commit(&path, "Master1", "m1.txt", "m1\n");
        make_commit(&path, "Master2", "m2.txt", "m2\n");

        // Checkout feature
        {
            let repo = git2::Repository::open(&path).unwrap();
            let (obj, reference) = repo.revparse_ext("refs/heads/feature").unwrap();
            repo.checkout_tree(&obj, None).unwrap();
            repo.set_head(reference.unwrap().name().unwrap()).unwrap();
        }

        // Add a commit on feature
        make_commit(&path, "Feature1", "f1.txt", "f1\n");

        // Rebase feature onto default branch
        let repo = git2::Repository::open(&path).unwrap();
        let main_name = if repo.find_branch("main", git2::BranchType::Local).is_ok() {
            "main"
        } else {
            "master"
        };
        drop(repo);
        let result = rebase_onto(&path, main_name).unwrap();
        assert!(result.contains("Rebase complete"));
        assert!(result.contains("1 commit"));
    }

    #[test]
    fn test_rebase_onto_invalid_ref_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "base.txt", "base\n");
        let result = rebase_onto(&path, "nonexistent-branch");
        assert!(result.is_err());
    }

    #[test]
    fn test_rebase_abort_no_rebase_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "Base", "base.txt", "base\n");
        // No rebase in progress, should error
        let result = rebase_abort(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_squash_preserves_tree() {
        let (_dir, path) = init_repo();
        make_commit(&path, "First", "a.txt", "a\n");
        make_commit(&path, "Second", "b.txt", "b\n");
        make_commit(&path, "Third", "c.txt", "c\n");

        squash_commits(&path, 3, "Squashed all").unwrap();

        // All files should still exist
        assert!(std::path::Path::new(&path).join("a.txt").exists());
        assert!(std::path::Path::new(&path).join("b.txt").exists());
        assert!(std::path::Path::new(&path).join("c.txt").exists());

        // HEAD message should be the squash message
        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.message().unwrap(), "Squashed all");
    }
}
