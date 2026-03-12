use crate::theme::{self, Theme};

#[tauri::command]
pub async fn load_user_themes() -> Result<Vec<Theme>, String> {
    tokio::task::spawn_blocking(theme::load_user_themes)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_user_theme(theme_data: Theme) -> Result<(), String> {
    tokio::task::spawn_blocking(move || theme::save_user_theme(&theme_data))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_user_theme(name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || theme::delete_user_theme(&name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_load_user_themes_ok() {
        // Always succeeds even if the themes directory doesn't exist yet
        let result = load_user_themes().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_and_delete_user_theme() {
        let theme = Theme {
            name: "test-theme-snazzgit-unit".to_string(),
            colors: {
                let mut m = HashMap::new();
                m.insert("--color-bg".to_string(), "#000000".to_string());
                m
            },
        };
        save_user_theme(theme).await.unwrap();

        let themes = load_user_themes().await.unwrap();
        assert!(themes
            .iter()
            .any(|t| t.name == "test-theme-snazzgit-unit"));

        // Clean up
        delete_user_theme("test-theme-snazzgit-unit".into())
            .await
            .unwrap();
    }
}
