use git2::Repository;

use super::error::GitError;

pub fn create_tag(
    path: &str,
    name: &str,
    commit_id: &str,
    message: Option<&str>,
) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let oid = git2::Oid::from_str(commit_id)
        .map_err(|e| GitError::General(format!("Invalid commit ID: {}", e)))?;
    let commit = repo.find_commit(oid)?;

    if let Some(msg) = message {
        let sig = repo.signature()?;
        repo.tag(name, commit.as_object(), &sig, msg, false)?;
    } else {
        repo.tag_lightweight(name, commit.as_object(), false)?;
    }
    Ok(())
}

pub fn delete_tag(path: &str, name: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    repo.tag_delete(name)?;
    Ok(())
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
            .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
        (dir, path, oid.to_string())
    }

    #[test]
    fn test_create_lightweight_tag() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        create_tag(&path, "v1.0", &commit_id, None).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_reference("refs/tags/v1.0").is_ok());
    }

    #[test]
    fn test_create_annotated_tag() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        create_tag(&path, "v2.0", &commit_id, Some("Release 2.0")).unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let tag_ref = repo.find_reference("refs/tags/v2.0").unwrap();
        let tag = repo.find_tag(tag_ref.target().unwrap()).unwrap();
        assert_eq!(tag.message().unwrap(), "Release 2.0");
    }

    #[test]
    fn test_create_duplicate_tag_fails() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        create_tag(&path, "v1.0", &commit_id, None).unwrap();
        assert!(create_tag(&path, "v1.0", &commit_id, None).is_err());
    }

    #[test]
    fn test_delete_lightweight_tag() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        create_tag(&path, "v1.0", &commit_id, None).unwrap();
        delete_tag(&path, "v1.0").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_reference("refs/tags/v1.0").is_err());
    }

    #[test]
    fn test_delete_annotated_tag() {
        let (_dir, path, commit_id) = init_repo_with_commit();
        create_tag(&path, "v2.0", &commit_id, Some("Release")).unwrap();
        delete_tag(&path, "v2.0").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        assert!(repo.find_reference("refs/tags/v2.0").is_err());
    }

    #[test]
    fn test_delete_nonexistent_tag_fails() {
        let (_dir, path, _) = init_repo_with_commit();
        assert!(delete_tag(&path, "nonexistent").is_err());
    }

    #[test]
    fn test_create_tag_invalid_commit_fails() {
        let (_dir, path, _) = init_repo_with_commit();
        assert!(create_tag(&path, "v1.0", "not-a-sha", None).is_err());
    }
}
