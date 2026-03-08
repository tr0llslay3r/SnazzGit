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
        let head_tree = repo.head()?.peel_to_tree()?;
        repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut opts))?
    } else {
        opts.include_untracked(true);
        opts.show_untracked_content(true);
        opts.recurse_untracked_dirs(true);
        repo.diff_index_to_workdir(None, Some(&mut opts))?
    };

    parse_diff(&diff, file_path)
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
    let deltas: Vec<_> = (0..diff.deltas().len()).collect();
    for _idx in &deltas {
        // We'll parse the whole diff at once
    }

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
