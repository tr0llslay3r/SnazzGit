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

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        (dir, path)
    }

    fn make_commit_in_repo(path: &str) {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &parent_refs).unwrap();
    }

    #[test]
    fn test_open_nonexistent_path() {
        let result = open_repo("/tmp/nonexistent_snazzgit_test_xyz_99999");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_valid_repo_with_commit() {
        let (_dir, path) = init_repo();
        make_commit_in_repo(&path);
        let info = open_repo(&path).unwrap();
        assert!(!info.path.is_empty());
        assert!(!info.is_bare);
        assert!(!info.name.is_empty());
    }

    #[test]
    fn test_open_repo_current_branch() {
        let (_dir, path) = init_repo();
        make_commit_in_repo(&path);
        let info = open_repo(&path).unwrap();
        // A freshly initialised repo should be on "main" or "master"
        assert!(info.current_branch.is_some());
    }

    #[test]
    fn test_open_repo_has_local_branch() {
        let (_dir, path) = init_repo();
        make_commit_in_repo(&path);
        let info = open_repo(&path).unwrap();
        assert!(!info.branches.is_empty());
        let local_branches: Vec<_> = info.branches.iter().filter(|b| !b.is_remote).collect();
        assert!(!local_branches.is_empty());
    }

    #[test]
    fn test_open_repo_empty_remotes() {
        let (_dir, path) = init_repo();
        make_commit_in_repo(&path);
        let info = open_repo(&path).unwrap();
        assert!(info.remotes.is_empty());
    }

    #[test]
    fn test_open_repo_stash_count_zero() {
        let (_dir, path) = init_repo();
        make_commit_in_repo(&path);
        let info = open_repo(&path).unwrap();
        assert_eq!(info.stash_count, 0);
    }
}
