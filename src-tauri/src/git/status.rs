use git2::{Repository, StatusOptions};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use super::error::GitError;
use super::types::{FileStatus, FileStatusType, WorkingTreeStatus};

pub fn get_status(path: &str) -> Result<WorkingTreeStatus, GitError> {
    let repo = Repository::open(path)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        if status.contains(git2::Status::WT_NEW) {
            untracked.push(path.clone());
        }

        // Staged changes
        if status.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            let file_status = if status.contains(git2::Status::INDEX_NEW) {
                FileStatusType::New
            } else if status.contains(git2::Status::INDEX_MODIFIED) {
                FileStatusType::Modified
            } else if status.contains(git2::Status::INDEX_DELETED) {
                FileStatusType::Deleted
            } else if status.contains(git2::Status::INDEX_RENAMED) {
                FileStatusType::Renamed
            } else {
                FileStatusType::Typechange
            };

            let old_path = entry
                .head_to_index()
                .and_then(|d| d.old_file().path().map(|p| p.to_string_lossy().to_string()));

            staged.push(FileStatus {
                path: path.clone(),
                status: file_status,
                old_path,
            });
        }

        // Unstaged changes (working tree)
        if status.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) {
            let file_status = if status.contains(git2::Status::WT_MODIFIED) {
                FileStatusType::Modified
            } else if status.contains(git2::Status::WT_DELETED) {
                FileStatusType::Deleted
            } else if status.contains(git2::Status::WT_RENAMED) {
                FileStatusType::Renamed
            } else {
                FileStatusType::Typechange
            };

            unstaged.push(FileStatus {
                path: path.clone(),
                status: file_status,
                old_path: None,
            });
        }

        // Conflicts
        if status.contains(git2::Status::CONFLICTED) {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Conflicted,
                old_path: None,
            });
        }
    }

    Ok(WorkingTreeStatus {
        staged,
        unstaged,
        untracked,
    })
}

pub fn stage_file(path: &str, file_path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new(file_path))?;
    index.write()?;
    Ok(())
}

pub fn unstage_file(path: &str, file_path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let head_commit = repo.head()?.peel_to_commit()?;
    repo.reset_default(Some(head_commit.as_object()), [file_path])?;
    Ok(())
}

pub fn stage_all(path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

pub fn unstage_all(path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let head_commit = repo.head()?.peel_to_commit()?;
    repo.reset_default(Some(head_commit.as_object()), ["*"])?;
    Ok(())
}

pub fn discard_file(path: &str, file_path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut checkout = git2::build::CheckoutBuilder::new();
    checkout.path(file_path).force();
    repo.checkout_head(Some(&mut checkout))?;
    Ok(())
}

pub fn delete_file(repo_path: &str, file_path: &str) -> Result<(), GitError> {
    let full_path = Path::new(repo_path).join(file_path);
    std::fs::remove_file(&full_path)?;
    Ok(())
}

pub fn add_to_gitignore(repo_path: &str, pattern: &str) -> Result<(), GitError> {
    let gitignore_path = Path::new(repo_path).join(".gitignore");

    // Read existing content to check for duplicates and trailing newline
    let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == pattern.trim()) {
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore_path)?;

    // Ensure we start on a new line
    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(file)?;
    }
    writeln!(file, "{}", pattern)?;
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
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        (dir, path)
    }

    fn write_file(repo_path: &str, name: &str, content: &str) {
        let p = std::path::Path::new(repo_path).join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_get_status_clean_repo() {
        let (_dir, path) = init_repo_with_commit();
        let status = get_status(&path).unwrap();
        assert!(status.staged.is_empty());
        assert!(status.unstaged.is_empty());
        assert!(status.untracked.is_empty());
    }

    #[test]
    fn test_untracked_file_appears_in_status() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "new_file.txt", "hello");
        let status = get_status(&path).unwrap();
        assert!(status.untracked.contains(&"new_file.txt".to_string()));
    }

    #[test]
    fn test_stage_file_moves_to_staged() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "staged.txt", "content");
        stage_file(&path, "staged.txt").unwrap();
        let status = get_status(&path).unwrap();
        assert!(!status.staged.is_empty());
        assert_eq!(status.staged[0].path, "staged.txt");
        assert!(matches!(status.staged[0].status, FileStatusType::New));
    }

    #[test]
    fn test_stage_file_not_in_untracked() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "staged.txt", "content");
        stage_file(&path, "staged.txt").unwrap();
        let status = get_status(&path).unwrap();
        assert!(!status.untracked.contains(&"staged.txt".to_string()));
    }

    #[test]
    fn test_stage_all_stages_multiple_files() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "a.txt", "a");
        write_file(&path, "b.txt", "b");
        stage_all(&path).unwrap();
        let status = get_status(&path).unwrap();
        assert_eq!(status.staged.len(), 2);
    }

    #[test]
    fn test_add_to_gitignore_creates_entry() {
        let (_dir, path) = init_repo_with_commit();
        add_to_gitignore(&path, "*.log").unwrap();
        let content =
            std::fs::read_to_string(std::path::Path::new(&path).join(".gitignore")).unwrap();
        assert!(content.contains("*.log"));
    }

    #[test]
    fn test_add_to_gitignore_no_duplicate() {
        let (_dir, path) = init_repo_with_commit();
        add_to_gitignore(&path, "*.log").unwrap();
        add_to_gitignore(&path, "*.log").unwrap();
        let content =
            std::fs::read_to_string(std::path::Path::new(&path).join(".gitignore")).unwrap();
        let count = content.lines().filter(|l| l.trim() == "*.log").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_add_multiple_patterns_to_gitignore() {
        let (_dir, path) = init_repo_with_commit();
        add_to_gitignore(&path, "*.log").unwrap();
        add_to_gitignore(&path, "target/").unwrap();
        let content =
            std::fs::read_to_string(std::path::Path::new(&path).join(".gitignore")).unwrap();
        assert!(content.contains("*.log"));
        assert!(content.contains("target/"));
    }

    #[test]
    fn test_delete_file_removes_it() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "to_delete.txt", "bye");
        delete_file(&path, "to_delete.txt").unwrap();
        assert!(!std::path::Path::new(&path).join("to_delete.txt").exists());
    }

    /// Write, stage, and commit a file on top of the existing HEAD.
    fn commit_file(repo_path: &str, name: &str, content: &str) {
        write_file(repo_path, name, content);
        stage_file(repo_path, name).unwrap();
        let repo = git2::Repository::open(repo_path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "add file", &tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn test_unstage_file_moves_back() {
        let (_dir, path) = init_repo_with_commit();
        commit_file(&path, "tracked.txt", "original");
        write_file(&path, "tracked.txt", "modified");
        stage_file(&path, "tracked.txt").unwrap();
        assert!(!get_status(&path).unwrap().staged.is_empty());
        unstage_file(&path, "tracked.txt").unwrap();
        let status = get_status(&path).unwrap();
        assert!(status.staged.is_empty());
        assert!(status.unstaged.iter().any(|f| f.path == "tracked.txt"));
    }

    #[test]
    fn test_unstage_all_clears_staged() {
        let (_dir, path) = init_repo_with_commit();
        write_file(&path, "a.txt", "a");
        write_file(&path, "b.txt", "b");
        stage_file(&path, "a.txt").unwrap();
        stage_file(&path, "b.txt").unwrap();
        assert_eq!(get_status(&path).unwrap().staged.len(), 2);
        unstage_all(&path).unwrap();
        assert!(get_status(&path).unwrap().staged.is_empty());
    }

    #[test]
    fn test_discard_file_reverts_to_head() {
        let (_dir, path) = init_repo_with_commit();
        commit_file(&path, "revert.txt", "original content");
        write_file(&path, "revert.txt", "changed content");
        discard_file(&path, "revert.txt").unwrap();
        let content =
            std::fs::read_to_string(std::path::Path::new(&path).join("revert.txt")).unwrap();
        assert_eq!(content, "original content");
    }

    #[test]
    fn test_get_status_modified_unstaged() {
        let (_dir, path) = init_repo_with_commit();
        commit_file(&path, "mod.txt", "original");
        write_file(&path, "mod.txt", "changed");
        let status = get_status(&path).unwrap();
        let modified = status.unstaged.iter().find(|f| f.path == "mod.txt");
        assert!(modified.is_some());
        assert!(matches!(modified.unwrap().status, FileStatusType::Modified));
    }
}
