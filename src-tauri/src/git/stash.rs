use git2::Repository;

use super::error::GitError;
use super::types::StashEntry;

pub fn stash_list(path: &str) -> Result<Vec<StashEntry>, GitError> {
    let mut repo = Repository::open(path)?;
    let mut entries = Vec::new();
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
    let mut repo = Repository::open(path)?;
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
    let mut repo = Repository::open(path)?;
    repo.stash_pop(index, None)?;
    Ok(())
}

pub fn stash_apply(path: &str, index: usize) -> Result<(), GitError> {
    let mut repo = Repository::open(path)?;
    repo.stash_apply(index, None)?;
    Ok(())
}

pub fn stash_drop(path: &str, index: usize) -> Result<(), GitError> {
    let mut repo = Repository::open(path)?;
    repo.stash_drop(index)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        // Write user config directly into repo config so repo.signature() works
        let cfg_path = repo.path().join("config");
        let mut cfg = git2::Config::open(&cfg_path).unwrap();
        cfg.set_str("user.name", "Test User").unwrap();
        cfg.set_str("user.email", "test@test.com").unwrap();
        drop(cfg);
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        (dir, path)
    }

    fn stage_new_file(repo_path: &str, name: &str, content: &str) {
        let p = std::path::Path::new(repo_path).join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let repo = git2::Repository::open(repo_path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(name)).unwrap();
        index.write().unwrap();
    }

    #[test]
    fn test_stash_list_empty_on_clean_repo() {
        let (_dir, path) = init_repo_with_commit();
        let list = stash_list(&path).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_stash_save_and_list() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "dirty.txt", "dirty");
        stash_save(&path, "my stash").unwrap();
        let list = stash_list(&path).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].message.contains("my stash"));
        assert_eq!(list[0].index, 0);
    }

    #[test]
    fn test_stash_multiple_entries() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file1.txt", "first");
        stash_save(&path, "stash one").unwrap();
        stage_new_file(&path, "file2.txt", "second");
        stash_save(&path, "stash two").unwrap();
        let list = stash_list(&path).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_stash_drop_removes_entry() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file.txt", "content");
        stash_save(&path, "to drop").unwrap();
        stash_drop(&path, 0).unwrap();
        let list = stash_list(&path).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_stash_pop_restores_changes() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "popped.txt", "pop me");
        stash_save(&path, "pop test").unwrap();
        // File should be gone from index now
        assert!(!std::path::Path::new(&path).join("popped.txt").exists());
        stash_pop(&path, 0).unwrap();
        let list = stash_list(&path).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_stash_apply_keeps_entry() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "apply.txt", "apply me");
        stash_save(&path, "apply test").unwrap();
        stash_apply(&path, 0).unwrap();
        let list = stash_list(&path).unwrap();
        // apply keeps the stash entry
        assert_eq!(list.len(), 1);
    }
}
