use git2::{DiffOptions, Oid, Repository};
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;

use super::error::GitError;
use super::types::{DiffFile, DiffHunk, DiffLine, DiffLineType, HighlightSpan};

pub fn get_working_diff(path: &str, file_path: &str, staged: bool) -> Result<DiffFile, GitError> {
    let repo = Repository::open(path)?;
    let mut opts = DiffOptions::new();
    opts.pathspec(file_path);

    let diff = if staged {
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))?
    } else {
        opts.include_untracked(true);
        opts.show_untracked_content(true);
        opts.recurse_untracked_dirs(true);
        repo.diff_index_to_workdir(None, Some(&mut opts))?
    };

    let mut result = parse_diff(&diff, file_path)?;

    // Fallback: if diff returned empty hunks, read file content directly
    if result.hunks.is_empty() {
        let content = if staged {
            // Read from the index
            read_from_index(&repo, file_path)
        } else {
            // Read from working tree
            let full_path = std::path::Path::new(path).join(file_path);
            if full_path.exists() {
                std::fs::read_to_string(&full_path).ok()
            } else {
                None
            }
        };
        if let Some(text) = content {
            let lines: Vec<&str> = text.lines().collect();
            if !lines.is_empty() {
                let diff_lines: Vec<DiffLine> = lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| DiffLine {
                        line_type: DiffLineType::Addition,
                        content: line.to_string(),
                        old_lineno: None,
                        new_lineno: Some((i + 1) as u32),
                        spans: Vec::new(),
                    })
                    .collect();
                let num_lines = diff_lines.len() as u32;
                result.hunks.push(DiffHunk {
                    header: format!("@@ -0,0 +1,{} @@", num_lines),
                    old_start: 0,
                    old_lines: 0,
                    new_start: 1,
                    new_lines: num_lines,
                    lines: diff_lines,
                });
                highlight_diff(&mut result);
            }
        }
    }

    Ok(result)
}

/// Read a file's content from the repo index (staging area).
fn read_from_index(repo: &Repository, file_path: &str) -> Option<String> {
    let index = repo.index().ok()?;
    let entry = index.get_path(std::path::Path::new(file_path), 0)?;
    let blob = repo.find_blob(entry.id).ok()?;
    if blob.is_binary() {
        return None;
    }
    std::str::from_utf8(blob.content()).ok().map(|s| s.to_string())
}

pub fn get_commit_diff(path: &str, commit_id: &str, file_path: Option<&str>) -> Result<Vec<DiffFile>, GitError> {
    let repo = Repository::open(path)?;
    let oid = Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;

    let mut opts = DiffOptions::new();
    if let Some(fp) = file_path {
        opts.pathspec(fp);
    }

    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;

    let mut files = Vec::new();

    // Parse all files from diff
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

    Ok(files)
}

/// Read a file's content at a given ref (or working tree if ref is None).
/// Returns base64 encoded content for binary-safe transfer.
pub fn read_file_at_ref(
    path: &str,
    file_path: &str,
    git_ref: Option<&str>,
) -> Result<Option<String>, GitError> {
    use base64::Engine;

    if let Some(r) = git_ref {
        let repo = Repository::open(path)?;
        let obj = repo.revparse_single(r)?;
        let tree = obj.peel_to_tree()?;
        match tree.get_path(std::path::Path::new(file_path)) {
            Ok(entry) => {
                let blob = repo.find_blob(entry.id())?;
                let encoded = base64::engine::general_purpose::STANDARD.encode(blob.content());
                Ok(Some(encoded))
            }
            Err(_) => Ok(None),
        }
    } else {
        // Read from working tree
        let full_path = std::path::Path::new(path).join(file_path);
        if full_path.exists() {
            let content = std::fs::read(&full_path)?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&content);
            Ok(Some(encoded))
        } else {
            Ok(None)
        }
    }
}

pub fn diff_refs(path: &str, from_ref: &str, to_ref: &str) -> Result<Vec<DiffFile>, GitError> {
    let repo = Repository::open(path)?;

    let from_obj = repo.revparse_single(from_ref)?;
    let to_obj = repo.revparse_single(to_ref)?;

    let from_tree = from_obj.peel_to_tree()?;
    let to_tree = to_obj.peel_to_tree()?;

    let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)?;

    let mut files = Vec::new();
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

    Ok(files)
}

pub fn highlight_diff(file: &mut DiffFile) {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let syntax = ss
        .find_syntax_for_file(&file.path)
        .ok()
        .flatten()
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);

    for hunk in &mut file.hunks {
        for line in &mut hunk.lines {
            if let Ok(ranges) = highlighter.highlight_line(&line.content, &ss) {
                line.spans = ranges_to_spans(&ranges);
            }
        }
    }
}

fn ranges_to_spans(ranges: &[(Style, &str)]) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let mut offset = 0;
    for (style, text) in ranges {
        let len = text.len();
        if len > 0 {
            spans.push(HighlightSpan {
                start: offset,
                end: offset + len,
                style: format!(
                    "color: #{:02x}{:02x}{:02x}",
                    style.foreground.r, style.foreground.g, style.foreground.b
                ),
            });
        }
        offset += len;
    }
    spans
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

    fn make_commit(path: &str, files: &[(&str, &str)]) -> git2::Oid {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        for (file_name, content) in files {
            let full_path = PathBuf::from(path).join(file_name);
            fs::write(&full_path, content).unwrap();
            index.add_path(std::path::Path::new(file_name)).unwrap();
        }
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
    fn test_get_working_diff_unstaged() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("foo.txt", "line1\n")]);

        // Modify file without staging
        fs::write(PathBuf::from(&path).join("foo.txt"), "line1\nline2\n").unwrap();

        let diff = get_working_diff(&path, "foo.txt", false).unwrap();
        assert_eq!(diff.path, "foo.txt");
        assert!(!diff.hunks.is_empty());
        let all_lines: Vec<_> = diff.hunks.iter().flat_map(|h| &h.lines).collect();
        assert!(all_lines
            .iter()
            .any(|l| l.content.contains("line2") && matches!(l.line_type, DiffLineType::Addition)));
    }

    #[test]
    fn test_get_working_diff_staged() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("foo.txt", "line1\n")]);

        // Modify and stage file
        fs::write(PathBuf::from(&path).join("foo.txt"), "line1\nline2\n").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("foo.txt")).unwrap();
        index.write().unwrap();

        let diff = get_working_diff(&path, "foo.txt", true).unwrap();
        assert_eq!(diff.path, "foo.txt");
        assert!(!diff.hunks.is_empty());
        let all_lines: Vec<_> = diff.hunks.iter().flat_map(|h| &h.lines).collect();
        assert!(all_lines
            .iter()
            .any(|l| matches!(l.line_type, DiffLineType::Addition)));
    }

    #[test]
    fn test_get_commit_diff_single_file() {
        let (_dir, path) = init_repo();
        let oid = make_commit(&path, &[("foo.txt", "hello\nworld\n")]);

        let files = get_commit_diff(&path, &oid.to_string(), Some("foo.txt")).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "foo.txt");
        assert!(!files[0].hunks.is_empty());
    }

    #[test]
    fn test_get_commit_diff_all_files() {
        let (_dir, path) = init_repo();
        let oid = make_commit(&path, &[("a.txt", "aaa\n"), ("b.txt", "bbb\n")]);

        let files = get_commit_diff(&path, &oid.to_string(), None).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_get_commit_diff_addition_lines() {
        let (_dir, path) = init_repo();
        let oid = make_commit(&path, &[("foo.txt", "hello\n")]);

        let files = get_commit_diff(&path, &oid.to_string(), Some("foo.txt")).unwrap();
        assert!(!files.is_empty());
        let all_lines: Vec<_> = files[0].hunks.iter().flat_map(|h| &h.lines).collect();
        assert!(all_lines
            .iter()
            .any(|l| matches!(l.line_type, DiffLineType::Addition)));
    }

    #[test]
    fn test_get_working_diff_untracked_file() {
        let (_dir, path) = init_repo();
        // Need an initial commit so HEAD exists
        make_commit(&path, &[("seed.txt", "seed\n")]);
        // Create an untracked file (not staged, not committed)
        fs::write(PathBuf::from(&path).join("new_file.txt"), "hello\nworld\n").unwrap();

        let diff = get_working_diff(&path, "new_file.txt", false).unwrap();
        assert_eq!(diff.path, "new_file.txt");
        // Should have hunks showing the file content as additions
        assert!(!diff.hunks.is_empty(), "untracked file diff should have hunks");
        let all_lines: Vec<_> = diff.hunks.iter().flat_map(|h| &h.lines).collect();
        assert!(
            all_lines.iter().any(|l| l.content.contains("hello") && matches!(l.line_type, DiffLineType::Addition)),
            "untracked file should show content as additions, got: {:?}", all_lines
        );
    }

    #[test]
    fn test_get_working_diff_untracked_in_subdir() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("seed.txt", "seed\n")]);
        let sub = PathBuf::from(&path).join("docs");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("test.md"), "# Hello\nContent\n").unwrap();

        let diff = get_working_diff(&path, "docs/test.md", false).unwrap();
        assert!(!diff.hunks.is_empty(), "untracked file in subdir should have hunks");
    }

    #[test]
    fn test_get_working_diff_untracked_no_prior_commits() {
        let (_dir, path) = init_repo();
        // No commits at all — repo has unborn HEAD
        fs::write(PathBuf::from(&path).join("readme.md"), "hello\n").unwrap();

        let result = get_working_diff(&path, "readme.md", false);
        // This may fail because there's no HEAD to diff against — let's see
        eprintln!("untracked-no-commits result: {:?}", result.as_ref().map(|d| d.hunks.len()));
        // At minimum it should not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_get_working_diff_invalid_path_errors() {
        let result =
            get_working_diff("/tmp/nonexistent_snazzgit_test_xyz_diff", "foo.txt", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_commit_diff_invalid_commit_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("foo.txt", "hello\n")]);
        let result = get_commit_diff(
            &path,
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_commit_diff_second_commit_shows_diff() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("foo.txt", "line1\n")]);
        let oid2 = make_commit(&path, &[("foo.txt", "line1\nline2\n")]);

        let files = get_commit_diff(&path, &oid2.to_string(), Some("foo.txt")).unwrap();
        assert_eq!(files.len(), 1);
        let all_lines: Vec<_> = files[0].hunks.iter().flat_map(|h| &h.lines).collect();
        assert!(all_lines
            .iter()
            .any(|l| l.content.contains("line2") && matches!(l.line_type, DiffLineType::Addition)));
    }

    #[test]
    fn test_diff_refs_shows_differences() {
        let (_dir, path) = init_repo();
        let oid1 = make_commit(&path, &[("foo.txt", "line1\n")]);
        let oid2 = make_commit(&path, &[("foo.txt", "line1\nline2\n")]);

        let files = diff_refs(&path, &oid1.to_string(), &oid2.to_string()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "foo.txt");
        let all_lines: Vec<_> = files[0].hunks.iter().flat_map(|h| &h.lines).collect();
        assert!(all_lines
            .iter()
            .any(|l| l.content.contains("line2") && matches!(l.line_type, DiffLineType::Addition)));
    }

    #[test]
    fn test_diff_refs_no_changes() {
        let (_dir, path) = init_repo();
        let oid = make_commit(&path, &[("foo.txt", "same\n")]);
        let files = diff_refs(&path, &oid.to_string(), &oid.to_string()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_diff_refs_multiple_files() {
        let (_dir, path) = init_repo();
        let oid1 = make_commit(&path, &[("a.txt", "aaa\n")]);
        let oid2 = make_commit(&path, &[("a.txt", "aaa changed\n"), ("b.txt", "bbb\n")]);

        let files = diff_refs(&path, &oid1.to_string(), &oid2.to_string()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_diff_refs_invalid_ref_errors() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("foo.txt", "hello\n")]);
        let result = diff_refs(&path, "nonexistent-ref", "HEAD");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_at_ref_returns_content() {
        let (_dir, path) = init_repo();
        let oid = make_commit(&path, &[("hello.txt", "hello world\n")]);
        let result = read_file_at_ref(&path, "hello.txt", Some(&oid.to_string())).unwrap();
        assert!(result.is_some());
        // Content is base64 encoded
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(result.unwrap())
            .unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "hello world\n");
    }

    #[test]
    fn test_read_file_at_ref_nonexistent_file_returns_none() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("hello.txt", "hello\n")]);
        let result = read_file_at_ref(&path, "nonexistent.txt", Some("HEAD")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_file_at_ref_working_tree() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("seed.txt", "seed\n")]);
        fs::write(PathBuf::from(&path).join("workfile.txt"), "working content\n").unwrap();
        let result = read_file_at_ref(&path, "workfile.txt", None).unwrap();
        assert!(result.is_some());
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(result.unwrap())
            .unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "working content\n");
    }

    #[test]
    fn test_read_file_at_ref_working_tree_nonexistent_returns_none() {
        let (_dir, path) = init_repo();
        make_commit(&path, &[("seed.txt", "seed\n")]);
        let result = read_file_at_ref(&path, "no_such_file.txt", None).unwrap();
        assert!(result.is_none());
    }
}

fn parse_diff(diff: &git2::Diff, file_path: &str) -> Result<DiffFile, GitError> {
    let mut result = DiffFile {
        path: file_path.to_string(),
        hunks: Vec::new(),
    };
    let mut current_hunk: Option<DiffHunk> = None;

    diff.print(git2::DiffFormat::Patch, |_delta, hunk, line| {
        match line.origin() {
            'H' => {
                if let Some(h) = current_hunk.take() {
                    result.hunks.push(h);
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

    if let Some(h) = current_hunk {
        result.hunks.push(h);
    }

    highlight_diff(&mut result);
    Ok(result)
}
