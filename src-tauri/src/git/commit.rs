use git2::{Oid, Repository, Sort};
use std::collections::HashMap;

use super::error::GitError;
use super::types::{CommitInfo, RefInfo, RefType};

pub fn load_commits(
    path: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<CommitInfo>, GitError> {
    let repo = Repository::open(path)?;
    let ref_map = build_ref_map(&repo)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    if revwalk.push_head().is_err() {
        // Empty repo with no commits yet (unborn HEAD)
        return Ok(Vec::new());
    }

    // Also push all branch tips so we see all branches
    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
        for branch_result in branches {
            if let Ok((branch, _)) = branch_result {
                if let Some(oid) = branch.get().target() {
                    let _ = revwalk.push(oid);
                }
            }
        }
    }

    let commits: Vec<CommitInfo> = revwalk
        .skip(offset)
        .take(limit)
        .filter_map(|oid_result| oid_result.ok())
        .filter_map(|oid| {
            let commit = repo.find_commit(oid).ok()?;
            let id = oid.to_string();
            let short_id = id[..8.min(id.len())].to_string();
            let message = commit
                .message()
                .unwrap_or("")
                .to_string();
            let summary = commit
                .summary()
                .unwrap_or("")
                .to_string();
            let author = commit.author();
            let committer = commit.committer();
            let parent_ids: Vec<String> =
                commit.parent_ids().map(|p| p.to_string()).collect();
            let refs = ref_map.get(&oid).cloned().unwrap_or_default();

            Some(CommitInfo {
                id,
                short_id,
                message,
                summary,
                author_name: author.name().unwrap_or("").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                author_time: author.when().seconds(),
                committer_name: committer.name().unwrap_or("").to_string(),
                committer_time: committer.when().seconds(),
                parent_ids,
                refs,
            })
        })
        .collect();

    Ok(commits)
}

pub fn get_commit_detail(path: &str, commit_id: &str) -> Result<CommitInfo, GitError> {
    let repo = Repository::open(path)?;
    let oid = Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    let ref_map = build_ref_map(&repo)?;

    let id = oid.to_string();
    let short_id = id[..8.min(id.len())].to_string();
    let author = commit.author();
    let committer = commit.committer();
    let parent_ids: Vec<String> = commit.parent_ids().map(|p| p.to_string()).collect();
    let refs = ref_map.get(&oid).cloned().unwrap_or_default();

    Ok(CommitInfo {
        id,
        short_id,
        message: commit.message().unwrap_or("").to_string(),
        summary: commit.summary().unwrap_or("").to_string(),
        author_name: author.name().unwrap_or("").to_string(),
        author_email: author.email().unwrap_or("").to_string(),
        author_time: author.when().seconds(),
        committer_name: committer.name().unwrap_or("").to_string(),
        committer_time: committer.when().seconds(),
        parent_ids,
        refs,
    })
}

fn build_ref_map(repo: &Repository) -> Result<HashMap<Oid, Vec<RefInfo>>, GitError> {
    let mut map: HashMap<Oid, Vec<RefInfo>> = HashMap::new();

    // HEAD
    if let Ok(head) = repo.head() {
        if let Some(oid) = head.target() {
            map.entry(oid).or_default().push(RefInfo {
                name: "HEAD".to_string(),
                ref_type: RefType::Head,
            });
        }
    }

    // Branches
    if let Ok(branches) = repo.branches(None) {
        for branch_result in branches {
            if let Ok((branch, branch_type)) = branch_result {
                if let (Some(name), Some(oid)) = (
                    branch.name().ok().flatten(),
                    branch.get().target(),
                ) {
                    let ref_type = match branch_type {
                        git2::BranchType::Local => RefType::LocalBranch,
                        git2::BranchType::Remote => RefType::RemoteBranch,
                    };
                    map.entry(oid).or_default().push(RefInfo {
                        name: name.to_string(),
                        ref_type,
                    });
                }
            }
        }
    }

    // Tags
    repo.tag_foreach(|oid, name| {
        let name = String::from_utf8_lossy(name)
            .trim_start_matches("refs/tags/")
            .to_string();
        // Resolve annotated tags to their target commit
        let target_oid = repo
            .find_tag(oid)
            .ok()
            .and_then(|tag| tag.target().ok())
            .map(|obj| obj.id())
            .unwrap_or(oid);
        map.entry(target_oid).or_default().push(RefInfo {
            name,
            ref_type: RefType::Tag,
        });
        true
    })?;

    Ok(map)
}
