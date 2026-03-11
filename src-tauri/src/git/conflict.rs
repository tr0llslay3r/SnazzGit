use std::path::{Path, PathBuf};

use git2::Repository;

use super::error::GitError;
use super::types::{ConflictFile, ConflictHunk, ConflictHunkKind};

pub fn get_conflict_diff(repo_path: &str, file_path: &str) -> Result<ConflictFile, GitError> {
    let full_path = PathBuf::from(repo_path).join(file_path);
    let raw = std::fs::read_to_string(&full_path)?;

    let mut hunks: Vec<ConflictHunk> = Vec::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut current_start: u32 = 1;
    let mut line_number: u32 = 0;

    #[derive(PartialEq)]
    enum State {
        Context,
        Ours,
        Theirs,
    }

    let mut state = State::Context;
    let mut conflict_index: u32 = 0;
    let mut our_label = String::new();
    let mut their_label = String::new();

    for line in raw.lines() {
        line_number += 1;
        let trimmed = line.trim_end_matches('\r');

        if trimmed.starts_with("<<<<<<<") {
            // Flush context
            if !current_lines.is_empty() {
                hunks.push(ConflictHunk {
                    kind: ConflictHunkKind::Context,
                    lines: std::mem::take(&mut current_lines),
                    start_line: current_start,
                    conflict_index: None,
                });
            }
            // Capture our label (text after "<<<<<<< ")
            our_label = trimmed
                .strip_prefix("<<<<<<<")
                .unwrap_or("")
                .trim()
                .to_string();
            current_start = line_number + 1;
            state = State::Ours;
        } else if trimmed.starts_with("=======") && state == State::Ours {
            // Flush ours
            hunks.push(ConflictHunk {
                kind: ConflictHunkKind::Ours,
                lines: std::mem::take(&mut current_lines),
                start_line: current_start,
                conflict_index: Some(conflict_index),
            });
            current_start = line_number + 1;
            state = State::Theirs;
        } else if trimmed.starts_with(">>>>>>>") && state == State::Theirs {
            // Capture their label
            their_label = trimmed
                .strip_prefix(">>>>>>>")
                .unwrap_or("")
                .trim()
                .to_string();
            // Flush theirs
            hunks.push(ConflictHunk {
                kind: ConflictHunkKind::Theirs,
                lines: std::mem::take(&mut current_lines),
                start_line: current_start,
                conflict_index: Some(conflict_index),
            });
            conflict_index += 1;
            current_start = line_number + 1;
            state = State::Context;
        } else {
            // Preserve the original line with its newline
            current_lines.push(format!("{}\n", trimmed));
        }
    }

    // Flush remaining context
    if !current_lines.is_empty() {
        hunks.push(ConflictHunk {
            kind: ConflictHunkKind::Context,
            lines: current_lines,
            start_line: current_start,
            conflict_index: None,
        });
    }

    if conflict_index == 0
        && hunks
            .iter()
            .all(|h| matches!(h.kind, ConflictHunkKind::Context))
    {
        return Err(GitError::General(format!(
            "No conflict markers found in {}",
            file_path
        )));
    }

    Ok(ConflictFile {
        path: file_path.to_string(),
        hunks,
        our_label,
        their_label,
    })
}

pub fn resolve_with_stage(
    repo_path: &str,
    file_path: &str,
    use_ours: bool,
) -> Result<(), GitError> {
    let repo = Repository::open(repo_path)?;
    let index = repo.index()?;
    let stage = if use_ours { 2 } else { 3 };

    let entry = index.get_path(Path::new(file_path), stage).ok_or_else(|| {
        GitError::General(format!(
            "No stage {} entry for {} — file may not be conflicted",
            stage, file_path
        ))
    })?;

    let blob = repo.find_blob(entry.id)?;
    let content = blob.content().to_vec();
    drop(blob);
    drop(index);
    drop(repo);

    let full_path = PathBuf::from(repo_path).join(file_path);
    std::fs::write(&full_path, &content)?;

    // Stage the resolved file
    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;
    index.add_path(Path::new(file_path))?;
    index.write()?;

    Ok(())
}

pub fn save_resolved_conflict(
    repo_path: &str,
    file_path: &str,
    content: &str,
) -> Result<(), GitError> {
    let full_path = PathBuf::from(repo_path).join(file_path);
    std::fs::write(&full_path, content.as_bytes())?;

    let repo = Repository::open(repo_path)?;
    let mut index = repo.index()?;
    index.add_path(Path::new(file_path))?;
    index.write()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_conflict_file(dir: &tempfile::TempDir, name: &str, content: &str) {
        std::fs::write(dir.path().join(name), content).unwrap();
    }

    fn make_commit(path: &str, file_name: &str, content: &str) {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        std::fs::write(PathBuf::from(path).join(file_name), content).unwrap();
        index.add_path(Path::new(file_name)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, "commit", &tree, &parent_refs)
            .unwrap();
    }

    #[test]
    fn test_get_conflict_diff_basic() {
        let dir = tempfile::TempDir::new().unwrap();
        let content = "<<<<<<< HEAD\nours line\n=======\ntheirs line\n>>>>>>> branch\n";
        write_conflict_file(&dir, "file.txt", content);

        let result = get_conflict_diff(dir.path().to_str().unwrap(), "file.txt").unwrap();
        assert_eq!(result.path, "file.txt");
        assert!(!result.hunks.is_empty());
        assert!(result
            .hunks
            .iter()
            .any(|h| matches!(h.kind, ConflictHunkKind::Ours)));
        assert!(result
            .hunks
            .iter()
            .any(|h| matches!(h.kind, ConflictHunkKind::Theirs)));
    }

    #[test]
    fn test_get_conflict_diff_no_conflict_errors() {
        let dir = tempfile::TempDir::new().unwrap();
        write_conflict_file(&dir, "clean.txt", "no conflicts here\njust normal text\n");

        let result = get_conflict_diff(dir.path().to_str().unwrap(), "clean.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_conflict_diff_multiple_conflicts() {
        let dir = tempfile::TempDir::new().unwrap();
        let content = concat!(
            "context line\n",
            "<<<<<<< HEAD\nours 1\n=======\ntheirs 1\n>>>>>>> b\n",
            "middle\n",
            "<<<<<<< HEAD\nours 2\n=======\ntheirs 2\n>>>>>>> b\n",
        );
        write_conflict_file(&dir, "multi.txt", content);

        let result = get_conflict_diff(dir.path().to_str().unwrap(), "multi.txt").unwrap();
        let ours_count = result
            .hunks
            .iter()
            .filter(|h| matches!(h.kind, ConflictHunkKind::Ours))
            .count();
        assert_eq!(ours_count, 2);
    }

    #[test]
    fn test_get_conflict_diff_labels() {
        let dir = tempfile::TempDir::new().unwrap();
        let content = "<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> feature-branch\n";
        write_conflict_file(&dir, "file.txt", content);

        let result = get_conflict_diff(dir.path().to_str().unwrap(), "file.txt").unwrap();
        assert_eq!(result.our_label, "HEAD");
        assert_eq!(result.their_label, "feature-branch");
    }

    #[test]
    fn test_get_conflict_diff_context_preserved() {
        let dir = tempfile::TempDir::new().unwrap();
        let content = "before\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> b\nafter\n";
        write_conflict_file(&dir, "file.txt", content);

        let result = get_conflict_diff(dir.path().to_str().unwrap(), "file.txt").unwrap();
        let context_hunks: Vec<_> = result
            .hunks
            .iter()
            .filter(|h| matches!(h.kind, ConflictHunkKind::Context))
            .collect();
        assert!(!context_hunks.is_empty());
    }

    #[test]
    fn test_save_resolved_conflict() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap();
        git2::Repository::init(path).unwrap();
        make_commit(path, "file.txt", "original\n");

        let resolved = "resolved content\n";
        save_resolved_conflict(path, "file.txt", resolved).unwrap();

        let on_disk = std::fs::read_to_string(dir.path().join("file.txt")).unwrap();
        assert_eq!(on_disk, resolved);
    }
}
