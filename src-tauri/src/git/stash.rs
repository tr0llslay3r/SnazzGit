use git2::Repository;

use super::error::GitError;
use super::diff::highlight_diff;
use super::types::{DiffFile, DiffHunk, DiffLine, DiffLineType, StashEntry};

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

pub fn stash_save(
    path: &str,
    message: &str,
    include_untracked: bool,
) -> Result<(), GitError> {
    let mut repo = Repository::open(path)?;
    let sig = repo.signature()?;
    let msg = if message.is_empty() { "WIP" } else { message };
    let flags = if include_untracked {
        Some(git2::StashFlags::INCLUDE_UNTRACKED)
    } else {
        None
    };
    repo.stash_save(&sig, msg, flags)?;
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

pub fn stash_diff(path: &str, stash_commit_id: &str) -> Result<Vec<DiffFile>, GitError> {
    let repo = Repository::open(path)?;
    let oid = git2::Oid::from_str(stash_commit_id)
        .map_err(|e| GitError::General(format!("Invalid stash commit ID: {}", e)))?;
    let stash_commit = repo.find_commit(oid)?;
    let stash_tree = stash_commit.tree()?;

    // The stash commit's first parent is the commit the stash was created from
    let parent = stash_commit.parent(0)?;
    let parent_tree = parent.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&stash_tree), None)?;

    let mut files: Vec<DiffFile> = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut current_hunk: Option<DiffHunk> = None;

    diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        let file_name = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        match line.origin() {
            'F' => {
                if let Some(mut prev_file) = current_file.take() {
                    if let Some(h) = current_hunk.take() {
                        prev_file.hunks.push(h);
                    }
                    files.push(prev_file);
                }
                current_file = Some(DiffFile {
                    path: file_name,
                    hunks: Vec::new(),
                });
            }
            'H' => {
                if let Some(ref mut file) = current_file {
                    if let Some(h) = current_hunk.take() {
                        file.hunks.push(h);
                    }
                }
                if let Some(h) = hunk {
                    current_hunk = Some(DiffHunk {
                        header: String::from_utf8_lossy(h.header()).trim().to_string(),
                        old_start: h.old_start(),
                        old_lines: h.old_lines(),
                        new_start: h.new_start(),
                        new_lines: h.new_lines(),
                        lines: Vec::new(),
                    });
                }
            }
            '+' | '-' | ' ' => {
                if let Some(ref mut hunk) = current_hunk {
                    let content = String::from_utf8_lossy(line.content()).to_string();
                    let line_type = match line.origin() {
                        '+' => DiffLineType::Addition,
                        '-' => DiffLineType::Deletion,
                        _ => DiffLineType::Context,
                    };
                    hunk.lines.push(DiffLine {
                        line_type,
                        content,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                        spans: Vec::new(),
                    });
                }
            }
            _ => {}
        }
        true
    })?;

    if let Some(mut file) = current_file {
        if let Some(h) = current_hunk {
            file.hunks.push(h);
        }
        files.push(file);
    }

    // Apply syntax highlighting
    for file in &mut files {
        highlight_diff(file);
    }

    Ok(files)
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
        stash_save(&path, "my stash", false).unwrap();
        let list = stash_list(&path).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].message.contains("my stash"));
        assert_eq!(list[0].index, 0);
    }

    #[test]
    fn test_stash_multiple_entries() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file1.txt", "first");
        stash_save(&path, "stash one", false).unwrap();
        stage_new_file(&path, "file2.txt", "second");
        stash_save(&path, "stash two", false).unwrap();
        let list = stash_list(&path).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_stash_drop_removes_entry() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file.txt", "content");
        stash_save(&path, "to drop", false).unwrap();
        stash_drop(&path, 0).unwrap();
        let list = stash_list(&path).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_stash_pop_restores_changes() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "popped.txt", "pop me");
        stash_save(&path, "pop test", false).unwrap();
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
        stash_save(&path, "apply test", false).unwrap();
        stash_apply(&path, 0).unwrap();
        let list = stash_list(&path).unwrap();
        // apply keeps the stash entry
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_stash_pop_invalid_index_errors() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file.txt", "content");
        stash_save(&path, "only stash", false).unwrap();
        let result = stash_pop(&path, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_stash_drop_invalid_index_errors() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file.txt", "content");
        stash_save(&path, "only stash", false).unwrap();
        let result = stash_drop(&path, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_stash_apply_invalid_index_errors() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file.txt", "content");
        stash_save(&path, "only stash", false).unwrap();
        let result = stash_apply(&path, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_stash_save_empty_message_defaults_to_wip() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "file.txt", "content");
        stash_save(&path, "", false).unwrap();
        let list = stash_list(&path).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].message.contains("WIP"));
    }

    #[test]
    fn test_stash_diff_shows_changes() {
        let (_dir, path) = init_repo_with_commit();
        stage_new_file(&path, "diffme.txt", "hello world");
        stash_save(&path, "diff test", false).unwrap();
        let list = stash_list(&path).unwrap();
        let commit_id = &list[0].commit_id;
        let files = stash_diff(&path, commit_id).unwrap();
        assert!(!files.is_empty());
        assert_eq!(files[0].path, "diffme.txt");
        assert!(!files[0].hunks.is_empty());
        // Should have at least one addition line
        let has_addition = files[0].hunks[0]
            .lines
            .iter()
            .any(|l| matches!(l.line_type, DiffLineType::Addition));
        assert!(has_addition);
    }

    #[test]
    fn test_stash_diff_invalid_commit_id_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = stash_diff(&path, "not-a-valid-oid");
        assert!(result.is_err());
    }

    #[test]
    fn test_stash_diff_nonexistent_commit_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = stash_diff(&path, "0000000000000000000000000000000000000000");
        assert!(result.is_err());
    }
}
