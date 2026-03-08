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
