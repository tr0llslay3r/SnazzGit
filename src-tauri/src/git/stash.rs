use git2::Repository;

use super::error::GitError;
use super::types::StashEntry;

pub fn stash_list(path: &str) -> Result<Vec<StashEntry>, GitError> {
    let repo = Repository::open(path)?;
    let mut entries = Vec::new();
    let mut repo = repo;
    repo.stash_foreach(|index, message, oid| {
        entries.push(StashEntry {
            index,
            message: message.to_string(),
            commit_id: oid.to_string(),
        });
        true
    })?;
    Ok(entries)
}

pub fn stash_save(path: &str, message: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut repo = repo;
    let sig = repo.signature()?;
    let msg = if message.is_empty() {
        None
    } else {
        Some(message)
    };
    repo.stash_save(&sig, msg.unwrap_or("WIP"), None)?;
    Ok(())
}

pub fn stash_pop(path: &str, index: usize) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut repo = repo;
    repo.stash_pop(index, None)?;
    Ok(())
}

pub fn stash_apply(path: &str, index: usize) -> Result<(), GitError> {
    let mut repo = Repository::open(path)?;
    repo.stash_apply(index, None)?;
    Ok(())
}

pub fn stash_drop(path: &str, index: usize) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut repo = repo;
    repo.stash_drop(index)?;
    Ok(())
}
