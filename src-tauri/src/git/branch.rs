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
