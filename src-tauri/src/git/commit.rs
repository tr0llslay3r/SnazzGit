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
        for (branch, _) in branches.flatten() {
            if let Some(oid) = branch.get().target() {
                let _ = revwalk.push(oid);
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
        for (branch, branch_type) in branches.flatten() {
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

pub fn file_history(
    path: &str,
    file_path: &str,
    limit: usize,
) -> Result<Vec<CommitInfo>, GitError> {
    let repo = Repository::open(path)?;
    let ref_map = build_ref_map(&repo)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }

    let mut commits = Vec::new();
    let target = std::path::Path::new(file_path);

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;

        // Check if this commit touched the file
        let has_file = tree.get_path(target).is_ok();
        let parent_has_same = commit
            .parent(0)
            .ok()
            .and_then(|p| p.tree().ok())
            .map(|pt| {
                match (pt.get_path(target), tree.get_path(target)) {
                    (Ok(pe), Ok(te)) => pe.id() == te.id(),
                    (Err(_), Err(_)) => true,
                    _ => false,
                }
            })
            .unwrap_or(!has_file);

        if !parent_has_same {
            let id = oid.to_string();
            let short_id = id[..8.min(id.len())].to_string();
            let author = commit.author();
            let committer = commit.committer();
            let parent_ids: Vec<String> = commit.parent_ids().map(|p| p.to_string()).collect();
            let refs = ref_map.get(&oid).cloned().unwrap_or_default();

            commits.push(CommitInfo {
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
            });

            if commits.len() >= limit {
                break;
            }
        }
    }

    Ok(commits)
}

pub fn cherry_pick(path: &str, commit_id: &str) -> Result<String, GitError> {
    let repo = Repository::open(path)?;
    let oid = Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;

    repo.cherrypick(&commit, None)?;

    let index = repo.index()?;
    if index.has_conflicts() {
        return Ok("Cherry-pick has conflicts - resolve before committing".to_string());
    }

    // Auto-commit the cherry-pick
    let sig = repo.signature().or_else(|_| {
        git2::Signature::now(
            commit.author().name().unwrap_or("Unknown"),
            commit.author().email().unwrap_or("unknown@unknown"),
        )
    })?;
    let tree_id = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let head = repo.head()?.peel_to_commit()?;
    let message = commit.message().unwrap_or("Cherry-picked commit");
    let new_oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&head])?;

    repo.cleanup_state()?;

    Ok(new_oid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_empty_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        (dir, path)
    }

    fn make_commit_msg(path: &str, msg: &str) -> git2::Oid {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs).unwrap()
    }

    #[test]
    fn test_load_commits_empty_repo() {
        let (_dir, path) = init_empty_repo();
        let commits = load_commits(&path, 100, 0).unwrap();
        assert!(commits.is_empty());
    }

    #[test]
    fn test_load_commits_single() {
        let (_dir, path) = init_empty_repo();
        make_commit_msg(&path, "First commit");
        let commits = load_commits(&path, 100, 0).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].summary, "First commit");
        assert_eq!(commits[0].author_name, "Test User");
        assert_eq!(commits[0].author_email, "test@test.com");
    }

    #[test]
    fn test_load_commits_newest_first() {
        let (_dir, path) = init_empty_repo();
        make_commit_msg(&path, "First");
        make_commit_msg(&path, "Second");
        make_commit_msg(&path, "Third");
        let commits = load_commits(&path, 100, 0).unwrap();
        assert_eq!(commits.len(), 3);
        assert_eq!(commits[0].summary, "Third");
        assert_eq!(commits[2].summary, "First");
    }

    #[test]
    fn test_load_commits_respects_limit() {
        let (_dir, path) = init_empty_repo();
        make_commit_msg(&path, "A");
        make_commit_msg(&path, "B");
        make_commit_msg(&path, "C");
        let commits = load_commits(&path, 2, 0).unwrap();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_load_commits_respects_offset() {
        let (_dir, path) = init_empty_repo();
        make_commit_msg(&path, "A");
        make_commit_msg(&path, "B");
        make_commit_msg(&path, "C");
        let commits = load_commits(&path, 100, 1).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].summary, "B");
    }

    #[test]
    fn test_load_commits_short_id_is_8_chars() {
        let (_dir, path) = init_empty_repo();
        make_commit_msg(&path, "Short id test");
        let commits = load_commits(&path, 1, 0).unwrap();
        assert_eq!(commits[0].short_id.len(), 8);
        assert!(commits[0].id.starts_with(&commits[0].short_id));
    }

    #[test]
    fn test_load_commits_parent_ids() {
        let (_dir, path) = init_empty_repo();
        make_commit_msg(&path, "Root");
        make_commit_msg(&path, "Child");
        let commits = load_commits(&path, 100, 0).unwrap();
        // Child (index 0) should have Root (index 1) as parent
        assert_eq!(commits[0].parent_ids.len(), 1);
        assert_eq!(commits[0].parent_ids[0], commits[1].id);
        // Root has no parents
        assert!(commits[1].parent_ids.is_empty());
    }

    #[test]
    fn test_get_commit_detail_by_id() {
        let (_dir, path) = init_empty_repo();
        let oid = make_commit_msg(&path, "Detail test");
        let oid_str = oid.to_string();
        let detail = get_commit_detail(&path, &oid_str).unwrap();
        assert_eq!(detail.id, oid_str);
        assert_eq!(detail.summary, "Detail test");
        assert_eq!(detail.short_id, oid_str[..8].to_string());
    }

    #[test]
    fn test_get_commit_detail_invalid_id() {
        let (_dir, path) = init_empty_repo();
        let result = get_commit_detail(&path, "0000000000000000000000000000000000000000");
        assert!(result.is_err());
    }

    /// Creates a repo with a merge commit: root → main-tip + root → feat-tip → merge.
    fn init_repo_with_merge_commit() -> (tempfile::TempDir, String, git2::Oid) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let root_oid = repo
            .commit(Some("HEAD"), &sig, &sig, "Root", &tree, &[])
            .unwrap();
        let root_commit = repo.find_commit(root_oid).unwrap();

        let main_oid = repo
            .commit(Some("HEAD"), &sig, &sig, "Main", &tree, &[&root_commit])
            .unwrap();
        let main_commit = repo.find_commit(main_oid).unwrap();

        // Feature commit branched from root, does not advance HEAD
        let feat_oid = repo
            .commit(None, &sig, &sig, "Feature", &tree, &[&root_commit])
            .unwrap();
        let feat_commit = repo.find_commit(feat_oid).unwrap();

        let merge_oid = repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Merge",
                &tree,
                &[&main_commit, &feat_commit],
            )
            .unwrap();

        (dir, path, merge_oid)
    }

    #[test]
    fn test_load_commits_merge_commit_has_two_parents() {
        let (_dir, path, _) = init_repo_with_merge_commit();
        let commits = load_commits(&path, 100, 0).unwrap();
        let merge = commits.iter().find(|c| c.summary == "Merge").unwrap();
        assert_eq!(merge.parent_ids.len(), 2);
    }

    #[test]
    fn test_get_commit_detail_merge_commit_shows_both_parents() {
        let (_dir, path, merge_oid) = init_repo_with_merge_commit();
        let detail = get_commit_detail(&path, &merge_oid.to_string()).unwrap();
        assert_eq!(detail.parent_ids.len(), 2);
    }
}
