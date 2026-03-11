use tauri::{AppHandle, Emitter};

use super::credentials::{make_credential_callback, Credentials};
use super::error::GitError;
use super::remote::ProgressPayload;

/// Clone a repository from a URL to a local path.
pub fn clone_repo(
    url: &str,
    path: &str,
    credentials: Option<Credentials>,
    app_handle: Option<&AppHandle>,
) -> Result<String, GitError> {
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

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    let repo = builder
        .clone(url, std::path::Path::new(path))
        .map_err(|e| GitError::General(format!("Clone failed: {}", e)))?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::General("Cloned repository has no working directory".into()))?
        .to_string_lossy()
        .to_string();

    Ok(workdir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Detect the default branch name used by git on this system.
    fn default_branch_name() -> String {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let sig = git2::Signature::now("Test", "t@t.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
        let name = repo.head().unwrap().shorthand().unwrap().to_string();
        name
    }

    fn init_bare_with_commit() -> (tempfile::TempDir, String, String) {
        let branch = default_branch_name();
        let src_dir = tempfile::TempDir::new().unwrap();
        let src_path = src_dir.path().to_str().unwrap().to_string();
        let bare_dir = tempfile::TempDir::new().unwrap();
        let bare_path = bare_dir.path().to_str().unwrap().to_string();
        git2::Repository::init_bare(&bare_path).unwrap();

        {
            let repo = git2::Repository::init(&src_path).unwrap();
            let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
            drop(tree);

            let refspec = format!("refs/heads/{b}:refs/heads/{b}", b = branch);
            let mut remote = repo.remote("origin", &bare_path).unwrap();
            let mut push_opts = git2::PushOptions::new();
            remote.push(&[&refspec], Some(&mut push_opts)).unwrap();
        }

        (bare_dir, bare_path, branch)
    }

    #[test]
    fn test_clone_local_repo() {
        let (_bare_dir, bare_path, branch) = init_bare_with_commit();
        let dest_dir = tempfile::TempDir::new().unwrap();
        let dest_path = dest_dir.path().join("cloned");
        let dest_str = dest_path.to_str().unwrap();

        let result = clone_repo(&bare_path, dest_str, None, None);
        assert!(result.is_ok());

        let workdir = result.unwrap();
        let repo = git2::Repository::open(&workdir).unwrap();
        let head = repo.head().unwrap();
        assert_eq!(head.shorthand().unwrap(), branch);
    }

    #[test]
    fn test_clone_returns_workdir_path() {
        let (_bare_dir, bare_path, _branch) = init_bare_with_commit();
        let dest_dir = tempfile::TempDir::new().unwrap();
        let dest_path = dest_dir.path().join("myrepo");
        let dest_str = dest_path.to_str().unwrap();

        let workdir = clone_repo(&bare_path, dest_str, None, None).unwrap();
        assert!(workdir.contains("myrepo"));
    }

    #[test]
    fn test_clone_invalid_url_fails() {
        let dest_dir = tempfile::TempDir::new().unwrap();
        let dest_path = dest_dir.path().join("cloned");
        let dest_str = dest_path.to_str().unwrap();

        let result = clone_repo("/nonexistent/path/to/repo", dest_str, None, None);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Clone failed"));
    }

    #[test]
    fn test_clone_to_existing_nonempty_dir_fails() {
        let (_bare_dir, bare_path, _branch) = init_bare_with_commit();
        let dest_dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dest_dir.path().join("blocker.txt"), "hello").unwrap();

        let result = clone_repo(&bare_path, dest_dir.path().to_str().unwrap(), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_preserves_commit_history() {
        let (_bare_dir, bare_path, _branch) = init_bare_with_commit();
        let dest_dir = tempfile::TempDir::new().unwrap();
        let dest_path = dest_dir.path().join("cloned");
        let dest_str = dest_path.to_str().unwrap();

        let workdir = clone_repo(&bare_path, dest_str, None, None).unwrap();
        let repo = git2::Repository::open(&workdir).unwrap();
        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head_commit.message().unwrap(), "Initial commit");
    }

    #[test]
    fn test_clone_with_none_credentials() {
        let (_bare_dir, bare_path, _branch) = init_bare_with_commit();
        let dest_dir = tempfile::TempDir::new().unwrap();
        let dest_path = dest_dir.path().join("cloned");
        let dest_str = dest_path.to_str().unwrap();

        let result = clone_repo(&bare_path, dest_str, None, None);
        assert!(result.is_ok());
    }
}
