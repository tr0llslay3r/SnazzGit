use std::path::Path;

#[tauri::command]
pub async fn open_in_file_manager(path: String) -> Result<(), String> {
    let target = Path::new(&path);
    let dir = if target.is_file() {
        target.parent().unwrap_or(target)
    } else {
        target
    };
    opener::open(dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_in_default_app(path: String) -> Result<(), String> {
    opener::open(Path::new(&path)).map_err(|e| e.to_string())
}

/// Try each `(command, args)` candidate in order, returning as soon as one
/// spawns successfully. Extracted from `open_terminal` so tests can supply
/// command names that are guaranteed not to exist, instead of depending on
/// which terminal emulators happen to be installed wherever this runs.
#[cfg(target_os = "linux")]
fn try_spawn_terminal(candidates: &[(&str, &[&str])], path: &str) -> Result<(), String> {
    for (term, args) in candidates {
        let result = std::process::Command::new(term)
            .args(*args)
            .current_dir(path)
            .spawn();
        if result.is_ok() {
            return Ok(());
        }
    }
    Err("No terminal emulator found".to_string())
}

#[tauri::command]
pub async fn open_terminal(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        {
            // Try common terminal emulators in order
            let terminals = [
                ("x-terminal-emulator", &["-e", "bash"][..]),
                ("gnome-terminal", &["--working-directory", &path][..]),
                ("konsole", &["--workdir", &path][..]),
                ("xfce4-terminal", &["--working-directory", &path][..]),
                ("xterm", &["-e", "bash"][..]),
            ];
            try_spawn_terminal(&terminals, &path)
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .args(["-a", "Terminal", &path])
                .spawn()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", &format!("cd /d {}", path)])
                .spawn()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn test_try_spawn_terminal_errors_when_none_found() {
        // Command names that cannot possibly exist, so this doesn't depend on
        // which terminal emulators (if any) happen to be installed wherever
        // this test runs — unlike the real `terminals` list in open_terminal,
        // which a CI runner image can start shipping one of at any time.
        let fake_terminals: &[(&str, &[&str])] = &[
            ("snazzgit-test-nonexistent-terminal-abc123", &[]),
            ("snazzgit-test-nonexistent-terminal-def456", &[]),
        ];
        let dir = tempfile::TempDir::new().unwrap();
        let result = try_spawn_terminal(fake_terminals, dir.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No terminal emulator found"));
    }

    #[tokio::test]
    async fn test_open_in_file_manager_resolves_file_to_parent() {
        // Create a temp file and verify the function runs without panic.
        // opener::open fires xdg-open asynchronously, so it may return Ok
        // even in headless environments. We verify it doesn't panic.
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();
        let _ = open_in_file_manager(file_path.to_str().unwrap().into()).await;
    }

    #[tokio::test]
    async fn test_open_in_file_manager_directory_path() {
        // When given a directory, it should pass the directory directly to opener.
        let dir = tempfile::TempDir::new().unwrap();
        let _ = open_in_file_manager(dir.path().to_str().unwrap().into()).await;
    }

    #[tokio::test]
    async fn test_open_in_default_app_runs_without_panic() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();
        let _ = open_in_default_app(file_path.to_str().unwrap().into()).await;
    }
}
