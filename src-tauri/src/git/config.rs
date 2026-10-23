use git2::{ConfigLevel, Repository};

use super::error::GitError;
use super::types::ConfigEntry;

pub fn get_config_entries(path: &str) -> Result<Vec<ConfigEntry>, GitError> {
    let repo = Repository::open(path)?;
    let config = repo.config()?;
    let local = config.open_level(ConfigLevel::Local)?;

    let mut entries = Vec::new();
    local.entries(None)?.for_each(|entry| {
        if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
            entries.push(ConfigEntry {
                name: name.to_string(),
                value: value.to_string(),
            });
        }
    })?;
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub fn set_config_entry(path: &str, key: &str, value: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut config = repo.config()?.open_level(ConfigLevel::Local)?;
    config.set_str(key, value)?;
    Ok(())
}

pub fn delete_config_entry(path: &str, key: &str) -> Result<(), GitError> {
    let repo = Repository::open(path)?;
    let mut config = repo.config()?.open_level(ConfigLevel::Local)?;
    config.remove(key)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        git2::Repository::init(&path).unwrap();
        (dir, path)
    }

    #[test]
    fn test_get_config_entries_reads_local_values() {
        let (_dir, path) = init_repo();
        {
            let repo = git2::Repository::open(&path).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test User").unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
        }
        let entries = get_config_entries(&path).unwrap();
        assert!(entries.iter().any(|e| e.name == "user.name" && e.value == "Test User"));
        assert!(entries.iter().any(|e| e.name == "user.email" && e.value == "test@test.com"));
    }

    #[test]
    fn test_get_config_entries_empty_repo_has_core_entries() {
        let (_dir, path) = init_repo();
        let entries = get_config_entries(&path).unwrap();
        // A freshly initialised repo always has at least core.* entries
        assert!(entries.iter().any(|e| e.name.starts_with("core.")));
    }

    #[test]
    fn test_get_config_entries_sorted_by_name() {
        let (_dir, path) = init_repo();
        {
            let repo = git2::Repository::open(&path).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("zzz.last", "z").unwrap();
            config.set_str("aaa.first", "a").unwrap();
        }
        let entries = get_config_entries(&path).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_set_config_entry_adds_new_key() {
        let (_dir, path) = init_repo();
        set_config_entry(&path, "custom.key", "hello").unwrap();
        let entries = get_config_entries(&path).unwrap();
        assert!(entries.iter().any(|e| e.name == "custom.key" && e.value == "hello"));
    }

    #[test]
    fn test_set_config_entry_overwrites_existing_key() {
        let (_dir, path) = init_repo();
        set_config_entry(&path, "custom.key", "first").unwrap();
        set_config_entry(&path, "custom.key", "second").unwrap();
        let entries = get_config_entries(&path).unwrap();
        let matches: Vec<_> = entries.iter().filter(|e| e.name == "custom.key").collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].value, "second");
    }

    #[test]
    fn test_delete_config_entry_removes_key() {
        let (_dir, path) = init_repo();
        set_config_entry(&path, "custom.key", "value").unwrap();
        delete_config_entry(&path, "custom.key").unwrap();
        let entries = get_config_entries(&path).unwrap();
        assert!(!entries.iter().any(|e| e.name == "custom.key"));
    }

    #[test]
    fn test_delete_config_entry_nonexistent_key_errors() {
        let (_dir, path) = init_repo();
        let result = delete_config_entry(&path, "does.not.exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_config_entries_invalid_path_errors() {
        let result = get_config_entries("/nonexistent/snazzgit_config_test_path");
        assert!(result.is_err());
    }
}
