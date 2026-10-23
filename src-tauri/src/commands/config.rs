use crate::git::config;
use crate::git::types::ConfigEntry;

#[tauri::command]
pub async fn get_config_entries(path: String) -> Result<Vec<ConfigEntry>, String> {
    tokio::task::spawn_blocking(move || config::get_config_entries(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_config_entry(path: String, key: String, value: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || config::set_config_entry(&path, &key, &value))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_config_entry(path: String, key: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || config::delete_config_entry(&path, &key))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
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

    #[tokio::test]
    async fn test_get_config_entries_command() {
        let (_dir, path) = init_repo();
        let result = get_config_entries(path).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_set_config_entry_command() {
        let (_dir, path) = init_repo();
        let result = set_config_entry(path.clone(), "custom.key".into(), "value".into()).await;
        assert!(result.is_ok());
        let entries = get_config_entries(path).await.unwrap();
        assert!(entries.iter().any(|e| e.name == "custom.key" && e.value == "value"));
    }

    #[tokio::test]
    async fn test_delete_config_entry_command() {
        let (_dir, path) = init_repo();
        set_config_entry(path.clone(), "custom.key".into(), "value".into())
            .await
            .unwrap();
        let result = delete_config_entry(path.clone(), "custom.key".into()).await;
        assert!(result.is_ok());
        let entries = get_config_entries(path).await.unwrap();
        assert!(!entries.iter().any(|e| e.name == "custom.key"));
    }

    #[tokio::test]
    async fn test_get_config_entries_invalid_path() {
        let result = get_config_entries("/nonexistent".into()).await;
        assert!(result.is_err());
    }
}
