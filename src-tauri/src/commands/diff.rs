use crate::git::diff;
use crate::git::types::DiffFile;

#[tauri::command]
pub async fn get_working_diff(
    path: String,
    file_path: String,
    staged: bool,
) -> Result<DiffFile, String> {
    tokio::task::spawn_blocking(move || diff::get_working_diff(&path, &file_path, staged))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_diff(
    path: String,
    commit_id: String,
    file_path: Option<String>,
) -> Result<Vec<DiffFile>, String> {
    tokio::task::spawn_blocking(move || {
        diff::get_commit_diff(&path, &commit_id, file_path.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_file_at_ref(
    path: String,
    file_path: String,
    git_ref: Option<String>,
) -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(move || {
        diff::read_file_at_ref(&path, &file_path, git_ref.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn diff_refs(
    path: String,
    from_ref: String,
    to_ref: String,
) -> Result<Vec<DiffFile>, String> {
    tokio::task::spawn_blocking(move || diff::diff_refs(&path, &from_ref, &to_ref))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_file_commit() -> (tempfile::TempDir, String, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        std::fs::write(dir.path().join("hello.txt"), "hello\nworld\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("hello.txt")).unwrap();
        index.write().unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();
        (dir, path, oid.to_string())
    }

    #[tokio::test]
    async fn test_get_commit_diff_valid() {
        let (_dir, path, oid) = init_repo_with_file_commit();
        let result = get_commit_diff(path, oid, None).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_commit_diff_with_file_filter() {
        let (_dir, path, oid) = init_repo_with_file_commit();
        let result = get_commit_diff(path, oid, Some("hello.txt".into())).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_commit_diff_invalid_oid() {
        let (_dir, path, _) = init_repo_with_file_commit();
        let result =
            get_commit_diff(path, "0000000000000000000000000000000000000000".into(), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_working_diff_staged() {
        let (_dir, path, _) = init_repo_with_file_commit();
        std::fs::write(std::path::Path::new(&path).join("hello.txt"), "changed\n").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("hello.txt")).unwrap();
        index.write().unwrap();
        let result = get_working_diff(path, "hello.txt".into(), true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_working_diff_unstaged() {
        let (_dir, path, _) = init_repo_with_file_commit();
        std::fs::write(std::path::Path::new(&path).join("hello.txt"), "changed\n").unwrap();
        let result = get_working_diff(path, "hello.txt".into(), false).await;
        assert!(result.is_ok());
    }
}
