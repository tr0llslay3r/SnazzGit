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

#[tauri::command]
pub async fn export_commit_as_patch(
    path: String,
    commit_id: String,
    dest_path: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || diff::export_commit_as_patch(&path, &commit_id, &dest_path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_patch_file(path: String, patch_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || diff::apply_patch_file(&path, &patch_path))
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

    #[tokio::test]
    async fn test_diff_refs_command() {
        let (_dir, path, oid1) = init_repo_with_file_commit();
        // Make a second commit
        std::fs::write(std::path::Path::new(&path).join("hello.txt"), "changed\n").unwrap();
        let repo = git2::Repository::open(&path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("hello.txt")).unwrap();
        index.write().unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let oid2 = repo.commit(Some("HEAD"), &sig, &sig, "Second", &tree, &[&parent]).unwrap();

        let result = diff_refs(path, oid1, oid2.to_string()).await;
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "hello.txt");
    }

    #[tokio::test]
    async fn test_diff_refs_invalid_ref() {
        let (_dir, path, _) = init_repo_with_file_commit();
        let result = diff_refs(path, "nonexistent".into(), "HEAD".into()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_file_at_ref_command() {
        let (_dir, path, _) = init_repo_with_file_commit();
        let result = read_file_at_ref(path, "hello.txt".into(), Some("HEAD".into())).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_read_file_at_ref_nonexistent_file() {
        let (_dir, path, _) = init_repo_with_file_commit();
        let result = read_file_at_ref(path, "nofile.txt".into(), Some("HEAD".into())).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_read_file_at_ref_working_tree() {
        let (_dir, path, _) = init_repo_with_file_commit();
        let result = read_file_at_ref(path, "hello.txt".into(), None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_export_and_apply_patch_commands_round_trip() {
        let (_dir_a, path_a, _) = init_repo_with_file_commit();
        // Second commit in repo A to export
        std::fs::write(std::path::Path::new(&path_a).join("hello.txt"), "hello\nworld\nmore\n").unwrap();
        let repo_a = git2::Repository::open(&path_a).unwrap();
        let mut index = repo_a.index().unwrap();
        index.add_path(std::path::Path::new("hello.txt")).unwrap();
        index.write().unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo_a.find_tree(tree_id).unwrap();
        let parent = repo_a.head().unwrap().peel_to_commit().unwrap();
        let oid2 = repo_a.commit(Some("HEAD"), &sig, &sig, "Add a line", &tree, &[&parent]).unwrap();

        let patch_file = tempfile::NamedTempFile::new().unwrap();
        let dest = patch_file.path().to_str().unwrap().to_string();
        let export_result = export_commit_as_patch(path_a, oid2.to_string(), dest.clone()).await;
        assert!(export_result.is_ok());

        // Independent repo B at the pre-change state
        let (_dir_b, path_b, _) = init_repo_with_file_commit();
        let apply_result = apply_patch_file(path_b.clone(), dest).await;
        assert!(apply_result.is_ok());

        let applied = std::fs::read_to_string(std::path::Path::new(&path_b).join("hello.txt")).unwrap();
        assert_eq!(applied, "hello\nworld\nmore\n");
    }

    #[tokio::test]
    async fn test_export_commit_as_patch_invalid_path() {
        let patch_file = tempfile::NamedTempFile::new().unwrap();
        let result = export_commit_as_patch(
            "/nonexistent".into(),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            patch_file.path().to_str().unwrap().into(),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_apply_patch_file_invalid_path() {
        let patch_file = tempfile::NamedTempFile::new().unwrap();
        let result = apply_patch_file(
            "/nonexistent".into(),
            patch_file.path().to_str().unwrap().into(),
        )
        .await;
        assert!(result.is_err());
    }
}
