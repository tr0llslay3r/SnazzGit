use crate::git::tag;

#[tauri::command]
pub async fn create_tag(
    path: String,
    name: String,
    commit_id: String,
    message: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        tag::create_tag(&path, &name, &commit_id, message.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(path: String, name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || tag::delete_tag(&path, &name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commit() -> (tempfile::TempDir, String, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();
        (dir, path, oid.to_string())
    }

    #[tokio::test]
    async fn test_create_tag_command() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        let result = create_tag(path, "v1.0".into(), commit_id, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_tag_command() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        create_tag(path.clone(), "v1.0".into(), commit_id, None)
            .await
            .unwrap();
        let result = delete_tag(path, "v1.0".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_tag_invalid_path() {
        let result = create_tag("/nonexistent".into(), "v1.0".into(), "abc".into(), None).await;
        assert!(result.is_err());
    }
}
