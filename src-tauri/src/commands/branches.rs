use crate::git::branch;

#[tauri::command]
pub async fn create_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::create_branch(&path, &name, true))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn checkout_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::checkout_branch(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::delete_branch(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_branch(
    path: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::rename_branch(&path, &old_name, &new_name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_branch(path: String, source_branch: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || branch::merge_branch(&path, &source_branch))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn checkout_remote_branch(
    path: String,
    remote_branch: String,
    local_name: String,
    track: bool,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        branch::checkout_remote_branch(&path, &remote_branch, &local_name, track)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn force_delete_branch(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::force_delete_branch(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_upstream(
    path: String,
    branch_name: String,
    upstream: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        branch::set_upstream(&path, &branch_name, upstream.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_to_commit(path: String, commit_id: String, mode: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || branch::reset_to_commit(&path, &commit_id, &mode))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
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
        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();
        (dir, path)
    }

    #[tokio::test]
    async fn test_create_branch() {
        let (_dir, path) = init_repo_with_commit();
        let result = create_branch(path, "feature".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_branch_invalid_path() {
        let result = create_branch("/nonexistent".into(), "feature".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_checkout_branch() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(path.clone(), "dev".into()).await.unwrap();
        let result = checkout_branch(path, "dev".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_checkout_branch_nonexistent_fails() {
        let (_dir, path) = init_repo_with_commit();
        let result = checkout_branch(path, "does-not-exist".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_branch() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(path.clone(), "to-delete".into()).await.unwrap();
        let result = delete_branch(path, "to-delete".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rename_branch() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(path.clone(), "old-name".into()).await.unwrap();
        let result = rename_branch(path, "old-name".into(), "new-name".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_to_commit_soft() {
        let (_dir, path) = init_repo_with_commit();
        let repo = git2::Repository::open(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let first = repo.head().unwrap().peel_to_commit().unwrap();
        let tree = first.tree().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Second", &tree, &[&first])
            .unwrap();
        let first_oid = first.id().to_string();
        let result = reset_to_commit(path, first_oid, "soft".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_to_commit_invalid_oid() {
        let (_dir, path) = init_repo_with_commit();
        let result = reset_to_commit(
            path,
            "0000000000000000000000000000000000000000".into(),
            "soft".into(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_merge_branch_already_up_to_date() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(path.clone(), "feature".into()).await.unwrap();
        let result = merge_branch(path, "feature".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Already up to date");
    }

    #[tokio::test]
    async fn test_merge_branch_nonexistent_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = merge_branch(path, "no-such-branch".into()).await;
        assert!(result.is_err());
    }

    /// Set up a bare "remote" and clone it to get remote tracking branches.
    fn init_repo_with_remote() -> (tempfile::TempDir, tempfile::TempDir, String) {
        let remote_dir = tempfile::TempDir::new().unwrap();
        let remote_path = remote_dir.path().to_str().unwrap();
        {
            let remote_repo = git2::Repository::init_bare(remote_path).unwrap();
            let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
            let tree_id = remote_repo.index().unwrap().write_tree().unwrap();
            let tree = remote_repo.find_tree(tree_id).unwrap();
            remote_repo
                .commit(Some("refs/heads/master"), &sig, &sig, "Initial", &tree, &[])
                .unwrap();
            let head_commit = remote_repo
                .find_reference("refs/heads/master")
                .unwrap()
                .peel_to_commit()
                .unwrap();
            remote_repo.branch("feature-remote", &head_commit, false).unwrap();
        }

        let clone_dir = tempfile::TempDir::new().unwrap();
        let clone_path = clone_dir.path().to_str().unwrap().to_string();
        git2::build::RepoBuilder::new()
            .clone(remote_path, std::path::Path::new(&clone_path))
            .unwrap();

        (remote_dir, clone_dir, clone_path)
    }

    #[tokio::test]
    async fn test_checkout_remote_branch_command() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        let result = checkout_remote_branch(
            path.clone(),
            "origin/feature-remote".into(),
            "feature-local".into(),
            true,
        ).await;
        assert!(result.is_ok());
        let repo = git2::Repository::open(&path).unwrap();
        assert_eq!(repo.head().unwrap().shorthand().unwrap(), "feature-local");
    }

    #[tokio::test]
    async fn test_checkout_remote_branch_nonexistent_errors() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        let result = checkout_remote_branch(
            path,
            "origin/no-such-branch".into(),
            "local".into(),
            false,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_force_delete_branch_command() {
        let (_dir, path) = init_repo_with_commit();
        create_branch(path.clone(), "to-force-delete".into()).await.unwrap();
        let result = force_delete_branch(path, "to-force-delete".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_force_delete_branch_nonexistent_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = force_delete_branch(path, "nope".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_upstream_command() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        checkout_remote_branch(
            path.clone(),
            "origin/feature-remote".into(),
            "feat".into(),
            false,
        ).await.unwrap();
        let result = set_upstream(path, "feat".into(), Some("origin/feature-remote".into())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_upstream_unset() {
        let (_remote_dir, _clone_dir, path) = init_repo_with_remote();
        checkout_remote_branch(
            path.clone(),
            "origin/feature-remote".into(),
            "feat".into(),
            true,
        ).await.unwrap();
        let result = set_upstream(path, "feat".into(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_upstream_nonexistent_branch_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = set_upstream(path, "nope".into(), Some("origin/main".into())).await;
        assert!(result.is_err());
    }
}
