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
            for (term, args) in &terminals {
                let result = std::process::Command::new(term)
                    .args(*args)
                    .current_dir(&path)
                    .spawn();
                if result.is_ok() {
                    return Ok(());
                }
            }
            Err("No terminal emulator found".to_string())
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
