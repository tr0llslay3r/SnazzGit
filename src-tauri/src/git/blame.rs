use git2::Repository;

use super::error::GitError;
use super::types::{BlameInfo, BlameLine};

pub fn get_blame(path: &str, file_path: &str) -> Result<BlameInfo, GitError> {
    let repo = Repository::open(path)?;
    let blame = repo.blame_file(std::path::Path::new(file_path), None)?;

    let mut lines = Vec::new();

    for hunk_idx in 0..blame.len() {
        let hunk = blame.get_index(hunk_idx).unwrap();
        let sig = hunk.final_signature();
        let commit_id = hunk.final_commit_id().to_string();
        let author = sig.name().unwrap_or("").to_string();
        let date = sig.when().seconds();

        for line_offset in 0..hunk.lines_in_hunk() {
            let line_number = hunk.final_start_line() + line_offset;
            lines.push(BlameLine {
                line_number: line_number as u32,
                commit_id: commit_id.clone(),
                author: author.clone(),
                date,
                content: String::new(), // Content is filled by reading the file
            });
        }
    }

    Ok(BlameInfo { lines })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn init_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        (dir, path)
    }

    fn make_commit(path: &str, file_name: &str, content: &str) -> git2::Oid {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        fs::write(PathBuf::from(path).join(file_name), content).unwrap();
        index.add_path(std::path::Path::new(file_name)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &parent_refs)
            .unwrap()
    }

    #[test]
    fn test_get_blame_basic() {
        let (_dir, path) = init_repo();
        make_commit(&path, "hello.txt", "line one\nline two\nline three\n");

        let info = get_blame(&path, "hello.txt").unwrap();
        // Three lines means at least one blame hunk covering all three
        assert!(!info.lines.is_empty());
        assert_eq!(info.lines[0].line_number, 1);
        assert_eq!(info.lines[0].author, "Test User");
        assert!(!info.lines[0].commit_id.is_empty());
    }

    #[test]
    fn test_get_blame_line_count() {
        let (_dir, path) = init_repo();
        make_commit(&path, "file.txt", "a\nb\nc\nd\n");

        let info = get_blame(&path, "file.txt").unwrap();
        assert_eq!(info.lines.len(), 4);
    }

    #[test]
    fn test_get_blame_multiple_commits() {
        let (_dir, path) = init_repo();
        make_commit(&path, "file.txt", "first line\n");
        make_commit(&path, "file.txt", "first line\nsecond line\n");

        let info = get_blame(&path, "file.txt").unwrap();
        assert_eq!(info.lines.len(), 2);
        // Lines from different commits should have different commit IDs
        assert_ne!(info.lines[0].commit_id, info.lines[1].commit_id);
    }

    #[test]
    fn test_get_blame_invalid_repo_errors() {
        let result = get_blame("/tmp/nonexistent_snazzgit_blame_xyz", "file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_blame_nonexistent_file_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, "real.txt", "content\n");

        let result = get_blame(&path, "ghost.txt");
        assert!(result.is_err());
    }
}
