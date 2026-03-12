use git2::{BranchType, ResetType, Repository};

use super::error::GitError;

pub fn create_branch(path: &str, name: &str, from_head: bool) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let _ = from_head; // TODO: support creating branch from non-HEAD commit
    let commit = repo.head()?.peel_to_commit()?;
    repo.branch(name, &commit, false)?;
    Ok(())
}

pub fn checkout_branch(path: &str, name: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let (object, reference) = repo.revparse_ext(&format!("refs/heads/{}", name))?;
    repo.checkout_tree(&object, None)?;
    if let Some(reference) = reference {
        repo.set_head(reference.name().unwrap_or(&format!("refs/heads/{}", name)))?;
    } else {
        repo.set_head(&format!("refs/heads/{}", name))?;
    }
    Ok(())
}

pub fn delete_branch(path: &str, name: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut branch = repo.find_branch(name, BranchType::Local)?;
    branch.delete()?;
    Ok(())
}

pub fn rename_branch(path: &str, old_name: &str, new_name: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut branch = repo.find_branch(old_name, BranchType::Local)?;
    branch.rename(new_name, false)?;
    Ok(())
}

pub fn merge_branch(path: &str, source_branch: &str) -> Result<String, GitError> {
    let repo = Repository::open(path)?;
    let branch = repo.find_branch(source_branch, BranchType::Local)?;
    let annotated = repo.reference_to_annotated_commit(branch.get())?;
    let (analysis, _) = repo.merge_analysis(&[&annotated])?;

    if analysis.is_up_to_date() {
        return Ok("Already up to date".to_string());
    }

    if analysis.is_fast_forward() {
        let target_oid = annotated.id();
        let mut reference = repo.head()?;
        reference.set_target(target_oid, "Fast-forward merge")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok("Fast-forward".to_string());
    }

    if analysis.is_normal() {
        repo.merge(&[&annotated], None, None)?;
        // Check for conflicts
        let index = repo.index()?;
        if index.has_conflicts() {
            return Ok("Conflicts detected - resolve before committing".to_string());
        }
        return Ok("Merge completed - commit to finalize".to_string());
    }

    Err(GitError::General("Merge analysis returned unexpected result".to_string()))
}

pub fn checkout_remote_branch(
    path: &str,
    remote_branch: &str,
    local_name: &str,
    track: bool,
) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let branch = repo.find_branch(remote_branch, BranchType::Remote)?;
    let commit = branch.get().peel_to_commit()?;
    let mut local_branch = repo.branch(local_name, &commit, false)?;
    if track {
        local_branch.set_upstream(Some(remote_branch))?;
    }
    let (object, reference) = repo.revparse_ext(&format!("refs/heads/{}", local_name))?;
    repo.checkout_tree(&object, None)?;
    if let Some(reference) = reference {
        repo.set_head(reference.name().unwrap_or(&format!("refs/heads/{}", local_name)))?;
    } else {
        repo.set_head(&format!("refs/heads/{}", local_name))?;
    }
    Ok(())
}

pub fn reset_to_commit(path: &str, commit_id: &str, mode: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let oid = git2::Oid::from_str(commit_id)
        .map_err(|e| GitError::General(format!("Invalid commit ID: {}", e)))?;
    let commit = repo.find_commit(oid)?;
    let reset_type = match mode {
        "soft" => ResetType::Soft,
        "mixed" => ResetType::Mixed,
        "hard" => ResetType::Hard,
        _ => return Err(GitError::General(format!("Unknown reset mode: {}", mode))),
    };
    repo.reset(commit.as_object(), reset_type, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        (dir, path)
    }

    fn add_commit(path: &str, msg: &str) {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&parent]).unwrap();
    }

    #[test]
    fn test_create_branch() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(&path, "feature", true).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_branch("feature", git2::BranchType::Local).is_ok());
    }

    #[test]
    fn test_create_branch_duplicate_fails() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(&path, "dup", true).unwrap();
        assert!(create_branch(&path, "dup", true).is_err());
    }

    #[test]
    fn test_checkout_branch() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(&path, "feature", true).unwrap();
        checkout_branch(&path, "feature").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap();
        assert_eq!(head.shorthand().unwrap(), "feature");
    }

    #[test]
    fn test_delete_branch() {
        let (_dir, path) = init_repo_with_commit();
        // Create the branch while still on the default branch so it is not HEAD
        create_branch(&path, "to-delete", true).unwrap();
        delete_branch(&path, "to-delete").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_branch("to-delete", git2::BranchType::Local).is_err());
    }

    #[test]
    fn test_rename_branch() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(&path, "old-name", true).unwrap();
        rename_branch(&path, "old-name", "new-name").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_branch("new-name", git2::BranchType::Local).is_ok());
        assert!(repo.find_branch("old-name", git2::BranchType::Local).is_err());
    }

    #[test]
    fn test_reset_to_commit_soft() {
        let (_dir, path) = init_repo_with_commit();
        let repo = git2::Repository::open(&path).unwrap();
        let first_id = repo.head().unwrap().target().unwrap().to_string();
        drop(repo);
        add_commit(&path, "Second commit");
        reset_to_commit(&path, &first_id, "soft").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert_eq!(repo.head().unwrap().target().unwrap().to_string(), first_id);
    }

    #[test]
    fn test_reset_to_commit_hard() {
        let (_dir, path) = init_repo_with_commit();
        let repo = git2::Repository::open(&path).unwrap();
        let first_id = repo.head().unwrap().target().unwrap().to_string();
        drop(repo);
        add_commit(&path, "Second commit");
        reset_to_commit(&path, &first_id, "hard").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert_eq!(repo.head().unwrap().target().unwrap().to_string(), first_id);
    }

    #[test]
    fn test_reset_invalid_mode_errors() {
        let (_dir, path) = init_repo_with_commit();
        let repo = git2::Repository::open(&path).unwrap();
        let head_id = repo.head().unwrap().target().unwrap().to_string();
        drop(repo);
        let result = reset_to_commit(&path, &head_id, "bogus");
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_invalid_commit_id_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = reset_to_commit(&path, "not-a-valid-sha", "hard");
        assert!(result.is_err());
    }

    /// Create a bare "remote" repo with a commit, then clone it so the clone
    /// has `origin/master` as a remote branch. Returns (remote_dir, clone_dir, clone_path).
    fn init_repo_with_remote() -> (tempfile::TempDir, tempfile::TempDir, String) {
        // Create bare remote repo with one commit
        let remote_dir = tempfile::TempDir::new().unwrap();
        let remote_path = remote_dir.path().to_str().unwrap();
        let remote_repo = git2::Repository::init_bare(remote_path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = {
            let mut index = remote_repo.index().unwrap();
            index.write_tree().unwrap()
        };
        {
            let tree = remote_repo.find_tree(tree_id).unwrap();
            remote_repo
                .commit(Some("refs/heads/master"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }
        {
            let head_commit = remote_repo
                .find_reference("refs/heads/master")
                .unwrap()
                .peel_to_commit()
                .unwrap();
            remote_repo.branch("feature-x", &head_commit, false).unwrap();
        }
        drop(remote_repo);

        // Clone it
        let clone_dir = tempfile::TempDir::new().unwrap();
        let clone_path = clone_dir.path().to_str().unwrap().to_string();
        git2::build::RepoBuilder::new()
            .clone(remote_path, std::path::Path::new(&clone_path))
            .unwrap();

        (remote_dir, clone_dir, clone_path)
    }

    #[test]
    fn test_checkout_remote_branch_basic() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        checkout_remote_branch(&path, "origin/feature-x", "feature-x", true).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap();
        assert_eq!(head.shorthand().unwrap(), "feature-x");
        // Verify local branch exists
        assert!(repo.find_branch("feature-x", git2::BranchType::Local).is_ok());
    }

    #[test]
    fn test_checkout_remote_branch_with_tracking() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        checkout_remote_branch(&path, "origin/feature-x", "feature-x", true).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let branch = repo.find_branch("feature-x", git2::BranchType::Local).unwrap();
        let upstream = branch.upstream().unwrap();
        assert_eq!(upstream.name().unwrap().unwrap(), "origin/feature-x");
    }

    #[test]
    fn test_checkout_remote_branch_without_tracking() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        checkout_remote_branch(&path, "origin/feature-x", "feature-x", false).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let branch = repo.find_branch("feature-x", git2::BranchType::Local).unwrap();
        // No upstream should be set
        assert!(branch.upstream().is_err());
    }

    #[test]
    fn test_checkout_remote_branch_custom_local_name() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        checkout_remote_branch(&path, "origin/feature-x", "my-local-name", true).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap();
        assert_eq!(head.shorthand().unwrap(), "my-local-name");
        assert!(repo.find_branch("my-local-name", git2::BranchType::Local).is_ok());
    }

    #[test]
    fn test_checkout_remote_branch_points_to_same_commit() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        let repo = git2::Repository::open(&path).unwrap();
        let remote_oid = repo
            .find_branch("origin/feature-x", git2::BranchType::Remote)
            .unwrap()
            .get()
            .target()
            .unwrap();
        drop(repo);

        checkout_remote_branch(&path, "origin/feature-x", "feature-x", false).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let local_oid = repo.head().unwrap().target().unwrap();
        assert_eq!(local_oid, remote_oid);
    }

    #[test]
    fn test_checkout_remote_branch_nonexistent_fails() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        let result = checkout_remote_branch(&path, "origin/no-such-branch", "local", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_checkout_remote_branch_duplicate_local_name_fails() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        // First checkout creates the local branch
        checkout_remote_branch(&path, "origin/feature-x", "feature-x", false).unwrap();
        // Second checkout with same local name should fail
        let result = checkout_remote_branch(&path, "origin/feature-x", "feature-x", false);
        assert!(result.is_err());
    }

    fn current_branch(path: &str) -> String {
        let repo = git2::Repository::open(path).unwrap();
        let head = repo.head().unwrap();
        head.shorthand().unwrap().to_string()
    }

    #[test]
    fn test_merge_branch_already_up_to_date() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(&path, "feature", true).unwrap();
        let result = merge_branch(&path, "feature").unwrap();
        assert_eq!(result, "Already up to date");
    }

    #[test]
    fn test_merge_branch_fast_forward() {
        let (_dir, path) = init_repo_with_commit();
        let main_branch = current_branch(&path);
        create_branch(&path, "feature", true).unwrap();
        checkout_branch(&path, "feature").unwrap();
        add_commit(&path, "commit on feature");
        checkout_branch(&path, &main_branch).unwrap();
        let result = merge_branch(&path, "feature").unwrap();
        assert_eq!(result, "Fast-forward");
    }

    #[test]
    fn test_merge_branch_diverged_no_conflicts() {
        let (_dir, path) = init_repo_with_commit();
        let main_branch = current_branch(&path);
        create_branch(&path, "feature", true).unwrap();
        add_commit(&path, "main commit");
        checkout_branch(&path, "feature").unwrap();
        add_commit(&path, "feature commit");
        checkout_branch(&path, &main_branch).unwrap();
        let result = merge_branch(&path, "feature").unwrap();
        assert_eq!(result, "Merge completed - commit to finalize");
    }

    #[test]
    fn test_merge_branch_nonexistent_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = merge_branch(&path, "no-such-branch");
        assert!(result.is_err());
    }
}
