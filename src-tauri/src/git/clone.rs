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
