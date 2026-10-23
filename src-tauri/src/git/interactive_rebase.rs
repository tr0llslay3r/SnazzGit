use git2::build::CheckoutBuilder;
use git2::{Oid, Repository};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use super::error::GitError;
use super::status;
use super::types::{
    FileStatusType, InteractiveRebaseStatus, RebaseAction, RebasePausedReason, RebaseTodoItem,
};

/// On-disk pause state for an in-progress interactive rebase. Never crosses the
/// Tauri IPC boundary (that's `InteractiveRebaseStatus`) — this is read/written
/// fresh on every command since each Tauri command opens its own `Repository`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractiveRebaseFile {
    original_branch_refname: String,
    original_head_oid: String,
    total_steps: usize,
    completed_steps: usize,
    commits_created: usize,
    pending_item: RebaseTodoItem,
    remaining_items: Vec<RebaseTodoItem>,
    paused_reason: RebasePausedReason,
}

fn state_file_path(repo: &Repository) -> PathBuf {
    repo.path().join("snazzgit").join("interactive-rebase-state.json")
}

fn save_state(repo: &Repository, state: &InteractiveRebaseFile) -> Result<(), GitError> {
    let file_path = state_file_path(repo);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content =
        serde_json::to_string_pretty(state).map_err(|e| GitError::General(e.to_string()))?;
    fs::write(file_path, content)?;
    Ok(())
}

fn try_load_state(repo: &Repository) -> Option<InteractiveRebaseFile> {
    let content = fs::read_to_string(state_file_path(repo)).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_state(repo: &Repository) -> Result<InteractiveRebaseFile, GitError> {
    try_load_state(repo)
        .ok_or_else(|| GitError::General("No interactive rebase in progress".to_string()))
}

fn delete_state(repo: &Repository) -> Result<(), GitError> {
    let file_path = state_file_path(repo);
    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

fn conflicted_files(path: &str) -> Result<Vec<String>, GitError> {
    let ws = status::get_status(path)?;
    Ok(ws
        .unstaged
        .into_iter()
        .filter(|f| matches!(f.status, FileStatusType::Conflicted))
        .map(|f| f.path)
        .collect())
}

/// Commit the currently-applied index (already clean, no conflicts) for a
/// Pick/Reword/Edit/Squash/Fixup step. Never called for Drop (never applied at
/// all) — the caller never reaches here for Drop items.
fn commit_step(repo: &Repository, item: &RebaseTodoItem) -> Result<Oid, GitError> {
    let orig_commit = repo.find_commit(Oid::from_str(&item.commit_id)?)?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let committer = repo
        .signature()
        .or_else(|_| git2::Signature::now("SnazzGit User", "snazzgit@local"))?;

    match item.action {
        RebaseAction::Pick | RebaseAction::Reword | RebaseAction::Edit => {
            // Preserve the original commit's author/time (matching `git rebase`'s
            // own behavior); Reword/Edit may override the message.
            let parent = repo.head()?.peel_to_commit()?;
            let author = orig_commit.author().to_owned();
            let message = item
                .message
                .clone()
                .unwrap_or_else(|| orig_commit.message().unwrap_or("").to_string());
            Ok(repo.commit(Some("HEAD"), &author, &committer, &message, &tree, &[&parent])?)
        }
        RebaseAction::Squash | RebaseAction::Fixup => {
            // Fold into the previous tip rather than stacking on top of it:
            // parent = previous tip's parent, author = previous tip's author
            // (matching `git commit --amend`'s default author-preservation).
            // libgit2 unconditionally rejects `update_ref: Some("HEAD")` here
            // (validate_tree_and_parents requires the ref's current target to
            // equal the new commit's first parent) since our first parent is
            // the tip's *grandparent*, not the tip itself — so we create the
            // commit untethered (update_ref: None) and move detached HEAD to
            // it ourselves via a soft reset, exactly like `squash_commits`.
            let prev_tip = repo.head()?.peel_to_commit()?;
            let author = prev_tip.author().to_owned();
            let grandparents: Vec<git2::Commit> = if prev_tip.parent_count() > 0 {
                vec![prev_tip.parent(0)?]
            } else {
                vec![]
            };
            let parent_refs: Vec<&git2::Commit> = grandparents.iter().collect();
            let message = match item.action {
                RebaseAction::Squash => item.message.clone().unwrap_or_else(|| {
                    format!(
                        "{}\n\n{}",
                        prev_tip.message().unwrap_or(""),
                        orig_commit.message().unwrap_or("")
                    )
                }),
                _ => prev_tip.message().unwrap_or("").to_string(),
            };
            let new_oid = repo.commit(None, &author, &committer, &message, &tree, &parent_refs)?;
            let new_obj = repo.find_object(new_oid, None)?;
            repo.reset(&new_obj, git2::ResetType::Soft, None)?;
            Ok(new_oid)
        }
        RebaseAction::Drop => {
            unreachable!("Drop items are skipped by the caller and never committed")
        }
    }
}

/// Replay the remaining todo items onto the (already detached) HEAD, pausing
/// and persisting state on a conflict or an Edit step, and reattaching the
/// original branch once everything has replayed cleanly.
fn run_sequence(
    repo: &Repository,
    original_branch_refname: &str,
    original_head_oid: &str,
    total_steps: usize,
    mut completed_steps: usize,
    mut commits_created: usize,
    mut queue: VecDeque<RebaseTodoItem>,
) -> Result<String, GitError> {
    while let Some(item) = queue.pop_front() {
        if matches!(item.action, RebaseAction::Drop) {
            completed_steps += 1;
            continue;
        }

        let commit = repo.find_commit(Oid::from_str(&item.commit_id)?)?;
        repo.cherrypick(&commit, None)?;

        if repo.index()?.has_conflicts() {
            save_state(
                repo,
                &InteractiveRebaseFile {
                    original_branch_refname: original_branch_refname.to_string(),
                    original_head_oid: original_head_oid.to_string(),
                    total_steps,
                    completed_steps,
                    commits_created,
                    pending_item: item,
                    remaining_items: queue.into_iter().collect(),
                    paused_reason: RebasePausedReason::Conflict,
                },
            )?;
            return Ok(format!(
                "Interactive rebase paused at step {}/{} due to conflicts. Resolve and continue.",
                completed_steps + 1,
                total_steps
            ));
        }

        if matches!(item.action, RebaseAction::Edit) {
            save_state(
                repo,
                &InteractiveRebaseFile {
                    original_branch_refname: original_branch_refname.to_string(),
                    original_head_oid: original_head_oid.to_string(),
                    total_steps,
                    completed_steps,
                    commits_created,
                    pending_item: item,
                    remaining_items: queue.into_iter().collect(),
                    paused_reason: RebasePausedReason::Edit,
                },
            )?;
            return Ok(format!(
                "Interactive rebase paused at step {}/{} for editing. Amend as needed, then continue.",
                completed_steps + 1,
                total_steps
            ));
        }

        commit_step(repo, &item)?;
        repo.cleanup_state()?;
        completed_steps += 1;
        // Squash/Fixup fold into the previous commit rather than adding a new
        // one, so they don't contribute to the final replayed-commit count.
        if !matches!(item.action, RebaseAction::Squash | RebaseAction::Fixup) {
            commits_created += 1;
        }
    }

    let tip_oid = repo
        .head()?
        .target()
        .ok_or_else(|| GitError::General("Detached HEAD has no target".to_string()))?;
    let mut branch_ref = repo.find_reference(original_branch_refname)?;
    branch_ref.set_target(tip_oid, "rebase (interactive): finish")?;
    repo.set_head(original_branch_refname)?;
    delete_state(repo)?;

    Ok(format!(
        "Rebase complete: {} commit{} replayed",
        commits_created,
        if commits_created == 1 { "" } else { "s" }
    ))
}

/// Start a custom, cherry-pick-based interactive rebase. Bypasses git2's
/// `Repository::rebase()` entirely since that API can only ever replay commits
/// as-is in ancestry order — it has no way to reorder, squash, fixup, or drop.
/// `base_commit_id` is the exclusive lower bound (the new parent of the first
/// todo item); `todo` must be oldest-first, matching `git rebase -i`'s
/// todo-file convention.
pub fn start_interactive_rebase(
    path: &str,
    base_commit_id: &str,
    todo: Vec<RebaseTodoItem>,
) -> Result<String, GitError> {
    if todo.is_empty() {
        return Err(GitError::General("Nothing to rebase".to_string()));
    }
    if matches!(todo[0].action, RebaseAction::Squash | RebaseAction::Fixup) {
        return Err(GitError::General(
            "Cannot squash/fixup the first commit — there is nothing before it to fold into"
                .to_string(),
        ));
    }

    if !status::is_clean(path)? {
        return Err(GitError::General(
            "Working tree is not clean. Commit or stash your changes before rebasing."
                .to_string(),
        ));
    }

    let repo = Repository::open(path)?;

    if repo.head_detached()? {
        return Err(GitError::General(
            "Cannot start an interactive rebase from a detached HEAD".to_string(),
        ));
    }

    let head_ref = repo.head()?;
    let original_branch_refname = head_ref
        .name()
        .ok_or_else(|| GitError::General("HEAD reference has no name".to_string()))?
        .to_string();
    let original_head_oid = head_ref
        .target()
        .ok_or_else(|| GitError::General("HEAD has no target".to_string()))?
        .to_string();

    let base_oid = Oid::from_str(base_commit_id)?;
    repo.find_commit(base_oid)?; // validate before mutating anything

    // Detach HEAD onto the base commit — this never touches the branch ref
    // itself, so the original branch stays exactly where it was until the
    // whole sequence finishes (or is aborted).
    repo.set_head_detached(base_oid)?;
    repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;

    run_sequence(
        &repo,
        &original_branch_refname,
        &original_head_oid,
        todo.len(),
        0,
        0,
        VecDeque::from(todo),
    )
}

/// Resume a paused interactive rebase: commit the currently-pending step using
/// whatever is in the index/working directory right now (a resolved conflict,
/// or user edits made during an Edit pause), then keep replaying.
pub fn interactive_rebase_continue(path: &str) -> Result<String, GitError> {
    let repo = Repository::open(path)?;
    let state = load_state(&repo)?;

    if repo.index()?.has_conflicts() {
        return Err(GitError::General(
            "Resolve all conflicts before continuing".to_string(),
        ));
    }

    commit_step(&repo, &state.pending_item)?;
    repo.cleanup_state()?;

    let commits_created = if matches!(
        state.pending_item.action,
        RebaseAction::Squash | RebaseAction::Fixup
    ) {
        state.commits_created
    } else {
        state.commits_created + 1
    };

    run_sequence(
        &repo,
        &state.original_branch_refname,
        &state.original_head_oid,
        state.total_steps,
        state.completed_steps + 1,
        commits_created,
        VecDeque::from(state.remaining_items),
    )
}

/// Abort a paused interactive rebase, restoring the repository to exactly how
/// it was before the rebase started. Safe to call even mid-conflict: the
/// branch ref was never touched (only the detached HEAD was advanced), so
/// this only needs to force-checkout back and reattach.
pub fn interactive_rebase_abort(path: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let state = load_state(&repo)?;

    let original_oid = Oid::from_str(&state.original_head_oid)?;
    repo.set_head_detached(original_oid)?;
    repo.checkout_head(Some(&mut CheckoutBuilder::new().force()))?;
    repo.set_head(&state.original_branch_refname)?;
    repo.cleanup_state()?;
    delete_state(&repo)?;
    Ok(())
}

/// Report whether an interactive rebase is currently paused, and why. Reads
/// purely from disk, so this works correctly even after an app restart.
pub fn get_interactive_rebase_status(path: &str) -> Result<InteractiveRebaseStatus, GitError> {
    let repo = Repository::open(path)?;
    let Some(state) = try_load_state(&repo) else {
        return Ok(InteractiveRebaseStatus {
            active: false,
            total_steps: 0,
            completed_steps: 0,
            paused_reason: None,
            conflicted_files: None,
        });
    };

    let conflicted = if matches!(state.paused_reason, RebasePausedReason::Conflict) {
        Some(conflicted_files(path)?)
    } else {
        None
    };

    Ok(InteractiveRebaseStatus {
        active: true,
        total_steps: state.total_steps,
        completed_steps: state.completed_steps,
        paused_reason: Some(state.paused_reason),
        conflicted_files: conflicted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();
        (dir, path)
    }

    fn make_commit(path: &str, msg: &str, file: &str, content: &str) -> Oid {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        std::fs::write(std::path::Path::new(path).join(file), content).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(file)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs)
            .unwrap()
    }

    fn pick(commit_id: Oid) -> RebaseTodoItem {
        RebaseTodoItem {
            commit_id: commit_id.to_string(),
            action: RebaseAction::Pick,
            message: None,
        }
    }

    fn history_messages(path: &str) -> Vec<String> {
        let repo = git2::Repository::open(path).unwrap();
        let mut walk = repo.revwalk().unwrap();
        walk.push_head().unwrap();
        walk.map(|oid| {
            repo.find_commit(oid.unwrap())
                .unwrap()
                .message()
                .unwrap_or("")
                .to_string()
        })
        .collect()
    }

    #[test]
    fn test_pick_only_reorder() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let b = make_commit(&path, "B", "b.txt", "b\n");
        let c = make_commit(&path, "C", "c.txt", "c\n");

        // Reorder: C before B.
        let result =
            start_interactive_rebase(&path, &base.to_string(), vec![pick(c), pick(b)]).unwrap();
        assert!(result.contains("Rebase complete"), "got: {result}");
        assert!(result.contains("2 commits"));

        let messages = history_messages(&path);
        assert_eq!(messages, vec!["B", "C", "Base"]);

        let repo = git2::Repository::open(&path).unwrap();
        assert!(!repo.head_detached().unwrap());
    }

    #[test]
    fn test_reword_changes_message_only() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "Original message", "a.txt", "a\n");

        let item = RebaseTodoItem {
            commit_id: a.to_string(),
            action: RebaseAction::Reword,
            message: Some("New message".to_string()),
        };
        start_interactive_rebase(&path, &base.to_string(), vec![item]).unwrap();

        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.message().unwrap(), "New message");

        let orig = repo.find_commit(a).unwrap();
        assert_eq!(head.author().name(), orig.author().name());
        assert_eq!(head.tree().unwrap().id(), orig.tree().unwrap().id());
    }

    #[test]
    fn test_squash_combines_into_one_commit() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "First", "a.txt", "a\n");
        let b = make_commit(&path, "Second", "b.txt", "b\n");

        let todo = vec![
            pick(a),
            RebaseTodoItem {
                commit_id: b.to_string(),
                action: RebaseAction::Squash,
                message: None,
            },
        ];
        let result = start_interactive_rebase(&path, &base.to_string(), todo).unwrap();
        assert!(result.contains("1 commit"), "got: {result}");

        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 1);
        assert_eq!(head.parent(0).unwrap().id(), base);
        let msg = head.message().unwrap();
        assert!(msg.contains("First") && msg.contains("Second"));
        assert!(std::path::Path::new(&path).join("a.txt").exists());
        assert!(std::path::Path::new(&path).join("b.txt").exists());
    }

    #[test]
    fn test_fixup_discards_message() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "Keep this message", "a.txt", "a\n");
        let b = make_commit(&path, "Discard this message", "b.txt", "b\n");

        let todo = vec![
            pick(a),
            RebaseTodoItem {
                commit_id: b.to_string(),
                action: RebaseAction::Fixup,
                message: None,
            },
        ];
        start_interactive_rebase(&path, &base.to_string(), todo).unwrap();

        let repo = git2::Repository::open(&path).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.message().unwrap(), "Keep this message");
        assert!(std::path::Path::new(&path).join("b.txt").exists());
    }

    #[test]
    fn test_drop_skips_commit_entirely() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");
        let b = make_commit(&path, "B", "b.txt", "b\n");
        let c = make_commit(&path, "C", "c.txt", "c\n");

        let todo = vec![
            pick(a),
            RebaseTodoItem {
                commit_id: b.to_string(),
                action: RebaseAction::Drop,
                message: None,
            },
            pick(c),
        ];
        let result = start_interactive_rebase(&path, &base.to_string(), todo).unwrap();
        assert!(result.contains("2 commits"), "got: {result}");

        assert!(!std::path::Path::new(&path).join("b.txt").exists());
        assert!(std::path::Path::new(&path).join("a.txt").exists());
        assert!(std::path::Path::new(&path).join("c.txt").exists());

        let messages = history_messages(&path);
        assert_eq!(messages, vec!["C", "A", "Base"]);
    }

    #[test]
    fn test_edit_pauses_then_continue_picks_up_extra_change() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");
        let b = make_commit(&path, "B", "b.txt", "b\n");

        let todo = vec![
            RebaseTodoItem {
                commit_id: a.to_string(),
                action: RebaseAction::Edit,
                message: None,
            },
            pick(b),
        ];
        let result = start_interactive_rebase(&path, &base.to_string(), todo).unwrap();
        assert!(result.contains("paused") && result.contains("editing"), "got: {result}");

        {
            let repo = git2::Repository::open(&path).unwrap();
            assert!(repo.head_detached().unwrap());
            let head = repo.head().unwrap().peel_to_commit().unwrap();
            assert_eq!(head.id(), base, "HEAD should not have advanced past base yet");
        }
        assert!(try_load_state(&git2::Repository::open(&path).unwrap()).is_some());

        // Simulate the user staging an extra change while paused on Edit.
        std::fs::write(std::path::Path::new(&path).join("a.txt"), "a-edited\n").unwrap();
        {
            let repo = git2::Repository::open(&path).unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("a.txt")).unwrap();
            index.write().unwrap();
        }

        let result = interactive_rebase_continue(&path).unwrap();
        assert!(result.contains("Rebase complete") && result.contains("2 commits"), "got: {result}");

        let content = std::fs::read_to_string(std::path::Path::new(&path).join("a.txt")).unwrap();
        assert_eq!(content, "a-edited\n");

        let repo = git2::Repository::open(&path).unwrap();
        assert!(!repo.head_detached().unwrap());
        assert!(try_load_state(&repo).is_none());
    }

    /// Build two commits that both modify the same line of the same file
    /// starting from a shared base (siblings, not a chain) — cherry-picking
    /// one after the other therefore conflicts. Returns (base, a, b).
    fn make_sibling_conflict(path: &str) -> (Oid, Oid, Oid) {
        let base = make_commit(path, "Base", "shared.txt", "line1\nline2\nline3\n");
        let a = make_commit(
            path,
            "A changes shared",
            "shared.txt",
            "line1\nA-CHANGE\nline3\n",
        );
        // Reset back to base so B is A's sibling, not A's child — otherwise
        // picking B onto A's result would just be a clean, non-conflicting
        // continuation of the same line of history.
        {
            let repo = git2::Repository::open(path).unwrap();
            repo.reset(
                repo.find_commit(base).unwrap().as_object(),
                git2::ResetType::Hard,
                None,
            )
            .unwrap();
        }
        let b = make_commit(
            path,
            "B changes shared",
            "shared.txt",
            "line1\nB-CHANGE\nline3\n",
        );
        (base, a, b)
    }

    #[test]
    fn test_conflict_pauses_then_continue() {
        let (_dir, path) = init_repo();
        let (base, a, b) = make_sibling_conflict(&path);

        let todo = vec![pick(a), pick(b)];
        let result = start_interactive_rebase(&path, &base.to_string(), todo).unwrap();
        assert!(result.contains("conflict") || result.contains("Conflict"), "got: {result}");

        {
            let repo = git2::Repository::open(&path).unwrap();
            assert!(repo.index().unwrap().has_conflicts());
        }

        // Resolve by writing out the merged content.
        std::fs::write(
            std::path::Path::new(&path).join("shared.txt"),
            "line1\nresolved\nline3\n",
        )
        .unwrap();
        {
            let repo = git2::Repository::open(&path).unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("shared.txt")).unwrap();
            index.write().unwrap();
        }

        let result = interactive_rebase_continue(&path).unwrap();
        assert!(result.contains("Rebase complete") && result.contains("2 commits"), "got: {result}");

        let repo = git2::Repository::open(&path).unwrap();
        assert!(!repo.index().unwrap().has_conflicts());
        assert!(try_load_state(&repo).is_none());
    }

    #[test]
    fn test_abort_mid_sequence_restores_original_state() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");
        let original_head = {
            let repo = git2::Repository::open(&path).unwrap();
            let oid = repo.head().unwrap().target().unwrap();
            oid
        };

        let todo = vec![RebaseTodoItem {
            commit_id: a.to_string(),
            action: RebaseAction::Edit,
            message: None,
        }];
        start_interactive_rebase(&path, &base.to_string(), todo).unwrap();

        interactive_rebase_abort(&path).unwrap();

        let repo = git2::Repository::open(&path).unwrap();
        assert!(!repo.head_detached().unwrap());
        assert_eq!(repo.head().unwrap().target().unwrap(), original_head);
        assert!(try_load_state(&repo).is_none());
        assert!(std::path::Path::new(&path).join("a.txt").exists());
    }

    #[test]
    fn test_dirty_working_tree_guard_rejects_before_mutating() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");
        let original_head = {
            let repo = git2::Repository::open(&path).unwrap();
            let oid = repo.head().unwrap().target().unwrap();
            oid
        };

        std::fs::write(std::path::Path::new(&path).join("dirty.txt"), "oops\n").unwrap();

        let result = start_interactive_rebase(&path, &base.to_string(), vec![pick(a)]);
        assert!(result.is_err());

        let repo = git2::Repository::open(&path).unwrap();
        assert!(!repo.head_detached().unwrap());
        assert_eq!(repo.head().unwrap().target().unwrap(), original_head);
    }

    #[test]
    fn test_squash_as_first_item_errors() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");

        let todo = vec![RebaseTodoItem {
            commit_id: a.to_string(),
            action: RebaseAction::Squash,
            message: None,
        }];
        let result = start_interactive_rebase(&path, &base.to_string(), todo);
        assert!(result.is_err());
    }

    #[test]
    fn test_detached_head_guard_rejects() {
        let (_dir, path) = init_repo();
        let base = make_commit(&path, "Base", "base.txt", "base\n");
        let a = make_commit(&path, "A", "a.txt", "a\n");

        {
            let repo = git2::Repository::open(&path).unwrap();
            repo.set_head_detached(a).unwrap();
        }

        let result = start_interactive_rebase(&path, &base.to_string(), vec![pick(a)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_status_reports_progress_and_conflicted_files() {
        let (_dir, path) = init_repo();
        let (base, a, b) = make_sibling_conflict(&path);

        let status = get_interactive_rebase_status(&path).unwrap();
        assert!(!status.active);

        start_interactive_rebase(&path, &base.to_string(), vec![pick(a), pick(b)]).unwrap();

        let status = get_interactive_rebase_status(&path).unwrap();
        assert!(status.active);
        assert_eq!(status.total_steps, 2);
        assert_eq!(status.completed_steps, 1);
        assert!(matches!(status.paused_reason, Some(RebasePausedReason::Conflict)));
        assert!(status
            .conflicted_files
            .unwrap()
            .iter()
            .any(|f| f == "shared.txt"));
    }
}
