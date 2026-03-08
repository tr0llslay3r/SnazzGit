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
