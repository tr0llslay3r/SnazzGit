use git2::{Repository, Sort};

use super::error::GitError;
use super::types::CommitInfo;

pub fn search_commits(
    path: &str,
    query: &str,
    max_results: usize,
) -> Result<Vec<CommitInfo>, GitError> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    revwalk.push_head()?;

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for oid_result in revwalk {
        if results.len() >= max_results {
            break;
        }
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let id_str = oid.to_string();

        let matches = message.to_lowercase().contains(&query_lower)
            || author_name.to_lowercase().contains(&query_lower)
            || author_email.to_lowercase().contains(&query_lower)
            || id_str.starts_with(&query_lower);

        if matches {
            let committer = commit.committer();
            results.push(CommitInfo {
                id: id_str.clone(),
                short_id: id_str[..8.min(id_str.len())].to_string(),
                message,
                summary: commit.summary().unwrap_or("").to_string(),
                author_name,
                author_email,
                author_time: author.when().seconds(),
                committer_name: committer.name().unwrap_or("").to_string(),
                committer_time: committer.when().seconds(),
                parent_ids: commit.parent_ids().map(|p| p.to_string()).collect(),
                refs: Vec::new(),
            });
        }
    }

    Ok(results)
}
