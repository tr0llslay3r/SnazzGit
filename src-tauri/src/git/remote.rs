use git2::Repository;
use tauri::{AppHandle, Emitter};

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
    app_handle: Option<&AppHandle>,
) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut remote = repo.find_remote(remote_name)?;

    let mut callbacks = git2::RemoteCallbacks::new();
    setup_credentials(&mut callbacks);

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
    app_handle: Option<&AppHandle>,
) -> Result<String, GitError> {
    fetch_remote(path, remote_name, app_handle)?;

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
    setup_credentials(&mut callbacks);

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

fn setup_credentials(callbacks: &mut git2::RemoteCallbacks) {
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            git2::Cred::ssh_key_from_agent(username)
        } else if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            // Try git credential helper
            git2::Cred::credential_helper(
                &git2::Config::open_default()?,
                _url,
                username_from_url,
            )
        } else {
            git2::Cred::default()
        }
    });
}
