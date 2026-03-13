use git2::Repository;
use tauri::{AppHandle, Emitter};

use super::credentials::{make_credential_callback, Credentials};
use super::error::GitError;

#[derive(Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub received_objects: usize,
    pub total_objects: usize,
    pub indexed_deltas: usize,
    pub total_deltas: usize,
    pub received_bytes: usize,
}

pub fn fetch_remote(
    path: &str,
    remote_name: &str,
    credentials: Option<Credentials>,
    app_handle: Option<&AppHandle>,
) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(make_credential_callback(credentials));

    if let Some(handle) = app_handle {
        let handle = handle.clone();
        callbacks.transfer_progress(move |progress| {
            let _ = handle.emit(
                "git-progress",
                ProgressPayload {
                    received_objects: progress.received_objects(),
                    total_objects: progress.total_objects(),
                    indexed_deltas: progress.indexed_deltas(),
                    total_deltas: progress.total_deltas(),
                    received_bytes: progress.received_bytes(),
                },
            );
            true
        });
    }

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options.prune(git2::FetchPrune::On);
    remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;
    Ok(())
}

pub fn pull(
    path: &str,
    remote_name: &str,
    credentials: Option<Credentials>,
    app_handle: Option<&AppHandle>,
) -> Result<String, GitError> {
    fetch_remote(path, remote_name, credentials, app_handle)?;

    let repo = Repository::open(path)?;
    let head = repo.head()?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| GitError::General("Cannot determine current branch".into()))?
        .to_string();

    let remote_ref = format!("{}/{}", remote_name, branch_name);
    let fetch_head = repo.find_reference(&format!("refs/remotes/{}", remote_ref))?;
    let annotated = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _) = repo.merge_analysis(&[&annotated])?;

    if analysis.is_up_to_date() {
        return Ok("Already up to date".to_string());
    }

    if analysis.is_fast_forward() {
        let mut reference = repo.head()?;
        reference.set_target(annotated.id(), "Pull fast-forward")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok("Fast-forward".to_string());
    }

    if analysis.is_normal() {
        repo.merge(&[&annotated], None, None)?;
        let index = repo.index()?;
        if index.has_conflicts() {
            return Ok("Conflicts detected".to_string());
        }
        return Ok("Merged - commit to finalize".to_string());
    }

    Err(GitError::General("Unexpected merge analysis".into()))
}

pub fn push(
    path: &str,
    remote_name: &str,
    credentials: Option<Credentials>,
    app_handle: Option<&AppHandle>,
) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let head = repo.head()?;
    let refspec = head
        .name()
        .ok_or_else(|| GitError::General("Cannot determine HEAD ref".into()))?
        .to_string();

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(make_credential_callback(credentials));

    if let Some(handle) = app_handle {
        let handle = handle.clone();
        callbacks.push_transfer_progress(move |current, total, bytes| {
            let _ = handle.emit(
                "git-progress",
                ProgressPayload {
                    received_objects: current,
                    total_objects: total,
                    indexed_deltas: 0,
                    total_deltas: 0,
                    received_bytes: bytes,
                },
            );
        });
    }

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);
    remote.push(&[&refspec], Some(&mut push_options))?;
    Ok(())
}

pub fn force_push(
    path: &str,
    remote_name: &str,
    credentials: Option<Credentials>,
    app_handle: Option<&AppHandle>,
) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let head = repo.head()?;
    let refspec = head
        .name()
        .ok_or_else(|| GitError::General("Cannot determine HEAD ref".into()))?
        .to_string();

    let force_refspec = format!("+{refspec}");

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(make_credential_callback(credentials));

    if let Some(handle) = app_handle {
        let handle = handle.clone();
        callbacks.push_transfer_progress(move |current, total, bytes| {
            let _ = handle.emit(
                "git-progress",
                ProgressPayload {
                    received_objects: current,
                    total_objects: total,
                    indexed_deltas: 0,
                    total_deltas: 0,
                    received_bytes: bytes,
                },
            );
        });
    }

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);
    remote.push(&[&force_refspec], Some(&mut push_options))?;
    Ok(())
}

pub fn add_remote(path: &str, name: &str, url: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    repo.remote(name, url)?;
    Ok(())
}

pub fn remove_remote(path: &str, name: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    repo.remote_delete(name)?;
    Ok(())
}

pub fn rename_remote(path: &str, old_name: &str, new_name: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    repo.remote_rename(old_name, new_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        (dir, path)
    }

    fn add_commit(path: &str, msg: &str) {
        let repo = git2::Repository::open(path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&parent]).unwrap();
    }

    /// Set up a local "remote" by creating a bare repo and adding it as a remote
    /// to the working repo. Returns (working_dir, working_path, bare_dir, bare_path).
    fn setup_local_remote() -> (tempfile::TempDir, String, tempfile::TempDir, String) {
        // Create bare repo to act as remote
        let bare_dir = tempfile::TempDir::new().unwrap();
        let bare_path = bare_dir.path().to_str().unwrap().to_string();
        git2::Repository::init_bare(&bare_path).unwrap();

        // Create working repo with a commit
        let (work_dir, work_path) = init_repo_with_commit();

        // Add bare repo as "origin" remote
        let repo = git2::Repository::open(&work_path).unwrap();
        repo.remote("origin", &bare_path).unwrap();

        (work_dir, work_path, bare_dir, bare_path)
    }

    // --- fetch_remote tests ---

    #[test]
    fn test_fetch_invalid_path() {
        let result = fetch_remote("/nonexistent/path", "origin", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_invalid_remote_name() {
        let (_dir, path) = init_repo_with_commit();
        let result = fetch_remote(&path, "nonexistent", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_local_remote() {
        let (_work_dir, work_path, _bare_dir, _bare_path) = setup_local_remote();

        // Push initial commit to bare repo first
        push(&work_path, "origin", None, None).unwrap();

        // Fetch should succeed against local bare repo
        let result = fetch_remote(&work_path, "origin", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fetch_with_none_credentials() {
        let (_work_dir, work_path, _bare_dir, _bare_path) = setup_local_remote();
        push(&work_path, "origin", None, None).unwrap();

        // Fetching a local remote with None credentials should work fine
        let result = fetch_remote(&work_path, "origin", None, None);
        assert!(result.is_ok());
    }

    // --- push tests ---

    #[test]
    fn test_push_to_local_remote() {
        let (_work_dir, work_path, _bare_dir, bare_path) = setup_local_remote();

        // Push should succeed to local bare repo
        push(&work_path, "origin", None, None).unwrap();

        // Verify the commit landed in the bare repo
        let bare_repo = git2::Repository::open(&bare_path).unwrap();
        let head = bare_repo.head().unwrap();
        // Default branch name varies by system (main or master)
        let branch = head.shorthand().unwrap();
        assert!(branch == "main" || branch == "master");
    }

    #[test]
    fn test_push_invalid_remote() {
        let (_dir, path) = init_repo_with_commit();
        let result = push(&path, "nonexistent", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_push_no_head_errors() {
        // An empty repo (no commits) has no HEAD to push
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();

        let bare_dir = tempfile::TempDir::new().unwrap();
        let bare_path = bare_dir.path().to_str().unwrap().to_string();
        git2::Repository::init_bare(&bare_path).unwrap();
        repo.remote("origin", &bare_path).unwrap();

        let result = push(&path, "origin", None, None);
        assert!(result.is_err());
    }

    // --- pull tests ---

    #[test]
    fn test_pull_up_to_date() {
        let (_work_dir, work_path, _bare_dir, _bare_path) = setup_local_remote();

        // Push, then pull — should be up to date
        push(&work_path, "origin", None, None).unwrap();
        let result = pull(&work_path, "origin", None, None).unwrap();
        assert_eq!(result, "Already up to date");
    }

    #[test]
    fn test_pull_fast_forward() {
        let (_work_dir, work_path, _bare_dir, bare_path) = setup_local_remote();

        // Push initial commit
        push(&work_path, "origin", None, None).unwrap();

        // Create a second clone, add a commit, push it to bare
        let clone_dir = tempfile::TempDir::new().unwrap();
        let clone_path = clone_dir.path().to_str().unwrap().to_string();
        git2::build::RepoBuilder::new()
            .clone(&bare_path, std::path::Path::new(&clone_path))
            .unwrap();
        add_commit(&clone_path, "New commit from clone");
        push(&clone_path, "origin", None, None).unwrap();

        // Pull from the original working repo — should fast-forward
        let result = pull(&work_path, "origin", None, None).unwrap();
        assert_eq!(result, "Fast-forward");
    }

    #[test]
    fn test_pull_invalid_remote() {
        let (_dir, path) = init_repo_with_commit();
        let result = pull(&path, "nonexistent", None, None);
        assert!(result.is_err());
    }

    // --- ProgressPayload ---

    // --- remote management tests ---

    #[test]
    fn test_add_remote() {
        let (_dir, path) = init_repo_with_commit();
        add_remote(&path, "upstream", "https://example.com/repo.git").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_remote("upstream").is_ok());
    }

    #[test]
    fn test_add_duplicate_remote_fails() {
        let (_dir, path) = init_repo_with_commit();
        add_remote(&path, "upstream", "https://example.com/repo.git").unwrap();
        assert!(add_remote(&path, "upstream", "https://other.com/repo.git").is_err());
    }

    #[test]
    fn test_remove_remote() {
        let (_work_dir, work_path, _bare_dir, _) = setup_local_remote();
        remove_remote(&work_path, "origin").unwrap();
        let repo = git2::Repository::open(&work_path).unwrap();
        assert!(repo.find_remote("origin").is_err());
    }

    #[test]
    fn test_remove_nonexistent_remote_fails() {
        let (_dir, path) = init_repo_with_commit();
        assert!(remove_remote(&path, "nonexistent").is_err());
    }

    #[test]
    fn test_rename_remote() {
        let (_work_dir, work_path, _bare_dir, _) = setup_local_remote();
        rename_remote(&work_path, "origin", "upstream").unwrap();
        let repo = git2::Repository::open(&work_path).unwrap();
        assert!(repo.find_remote("upstream").is_ok());
        assert!(repo.find_remote("origin").is_err());
    }

    #[test]
    fn test_progress_payload_serialization() {
        let payload = ProgressPayload {
            received_objects: 10,
            total_objects: 20,
            indexed_deltas: 5,
            total_deltas: 10,
            received_bytes: 4096,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"received_objects\":10"));
        assert!(json.contains("\"total_objects\":20"));
        assert!(json.contains("\"received_bytes\":4096"));
    }
}
