use git2::Repository;
use std::path::Path;

use super::error::GitError;
use super::types::{BranchInfo, RepoInfo};

pub fn open_repo(path: &str) -> Result<RepoInfo, GitError> {
    let repo_path = Path::new(path);
    if !repo_path.exists() {
        return Err(GitError::RepoNotFound(path.to_string()));
    }

    let repo = Repository::discover(repo_path)?;
    let workdir = repo
        .workdir()
        .unwrap_or(repo.path())
        .to_string_lossy()
        .to_string();
    let name = Path::new(&workdir)
        .file_name()
        .or_else(|| Path::new(&workdir).parent()?.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let current_branch = get_current_branch(&repo);
    let branches = get_branches(&repo)?;
    let remotes = get_remotes(&repo)?;
    let tags = get_tags(&repo)?;
    let stash_count = get_stash_count(&workdir);

    Ok(RepoInfo {
        path: workdir.trim_end_matches('/').to_string(),
        name,
        current_branch,
        is_bare: repo.is_bare(),
        branches,
        remotes,
        tags,
        stash_count,
    })
}

fn get_current_branch(repo: &Repository) -> Option<String> {
    if repo.head_detached().unwrap_or(false) {
        let head = repo.head().ok()?;
        let oid = head.target()?;
        return Some(format!("(detached {})", &oid.to_string()[..8]));
    }
    let head = repo.head().ok()?;
    head.shorthand().map(|s| s.to_string())
}

fn get_branches(repo: &Repository) -> Result<Vec<BranchInfo>, GitError> {
    let mut branches = Vec::new();
    for branch_result in repo.branches(None)? {
        let (branch, branch_type) = branch_result?;
        let name = branch
            .name()?
            .unwrap_or("unknown")
            .to_string();
        let is_head = branch.is_head();
        let is_remote = branch_type == git2::BranchType::Remote;
        let upstream = branch
            .upstream()
            .ok()
            .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));
        let commit_id = branch
            .get()
            .target()
            .map(|oid| oid.to_string())
            .unwrap_or_default();

        branches.push(BranchInfo {
            name,
            is_head,
            is_remote,
            upstream,
            commit_id,
        });
    }
    Ok(branches)
}

fn get_remotes(repo: &Repository) -> Result<Vec<String>, GitError> {
    let remotes = repo.remotes()?;
    Ok(remotes.iter().filter_map(|r| r.map(|s| s.to_string())).collect())
}

fn get_tags(repo: &Repository) -> Result<Vec<String>, GitError> {
    let mut tags = Vec::new();
    repo.tag_names(None)?.iter().for_each(|t| {
        if let Some(name) = t {
            tags.push(name.to_string());
        }
    });
    Ok(tags)
}

fn get_stash_count(path: &str) -> usize {
    let mut count = 0;
    if let Ok(mut repo) = Repository::open(path) {
        let _ = repo.stash_foreach(|_, _, _| {
            count += 1;
            true
        });
    }
    count
}
