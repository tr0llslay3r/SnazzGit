use git2::Repository;

use super::error::GitError;
use super::types::{SubmoduleInfo, SubmoduleStatus};

pub fn list_submodules(path: &str) -> Result<Vec<SubmoduleInfo>, GitError> {
    let repo = Repository::open(path)?;
    let mut result = Vec::new();

    for sm in repo.submodules()? {
        let head_id = sm.head_id().map(|oid| oid.to_string());
        let workdir_id = sm.workdir_id().map(|oid| oid.to_string());

        let status = if workdir_id.is_none() {
            SubmoduleStatus::Uninitialized
        } else if head_id == workdir_id {
            SubmoduleStatus::UpToDate
        } else {
            SubmoduleStatus::Modified
        };

        result.push(SubmoduleInfo {
            name: sm.name().unwrap_or_default().to_string(),
            path: sm.path().to_string_lossy().to_string(),
            url: sm.url().map(|s| s.to_string()),
            head_id,
            workdir_id,
            status,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
        (dir, path)
    }

    fn add_submodule(main_path: &str, sub_path: &str, name: &str) {
        let main_repo = git2::Repository::open(main_path).unwrap();
        let mut sm = main_repo.submodule(sub_path, Path::new(name), true).unwrap();
        sm.clone(None).unwrap();
        sm.add_to_index(true).unwrap();
        sm.add_finalize().unwrap();

        // Commit the new .gitmodules + gitlink so head_id() has something to report
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let mut index = main_repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = main_repo.find_tree(tree_id).unwrap();
        let parent = main_repo.head().unwrap().peel_to_commit().unwrap();
        main_repo
            .commit(Some("HEAD"), &sig, &sig, "Add submodule", &tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn test_list_submodules_empty_repo() {
        let (_dir, path) = init_repo_with_commit();
        let submodules = list_submodules(&path).unwrap();
        assert!(submodules.is_empty());
    }

    #[test]
    fn test_list_submodules_up_to_date() {
        let (_main_dir, main_path) = init_repo_with_commit();
        let (_sub_dir, sub_path) = init_repo_with_commit();
        add_submodule(&main_path, &sub_path, "sublib");

        let submodules = list_submodules(&main_path).unwrap();
        assert_eq!(submodules.len(), 1);
        let sm = &submodules[0];
        assert_eq!(sm.name, "sublib");
        assert_eq!(sm.path, "sublib");
        assert_eq!(sm.url.as_deref(), Some(sub_path.as_str()));
        assert!(sm.head_id.is_some());
        assert_eq!(sm.head_id, sm.workdir_id);
        assert!(matches!(sm.status, SubmoduleStatus::UpToDate));
    }

    #[test]
    fn test_list_submodules_invalid_path_errors() {
        let result = list_submodules("/nonexistent/snazzgit_submodule_test_path");
        assert!(result.is_err());
    }
}
