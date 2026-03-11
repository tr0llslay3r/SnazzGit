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
}
