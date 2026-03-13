use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct RepoWatcher {
    _watcher: RecommendedWatcher,
}

impl RepoWatcher {
    pub fn new(path: &str, app_handle: AppHandle) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher
            .watch(Path::new(path), RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch path: {}", e))?;

        let handle = app_handle.clone();
        std::thread::spawn(move || {
            let mut pending = false;
            let mut last_emit = std::time::Instant::now();
            loop {
                match rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(event) => {
                        // Skip .git internal changes that aren't relevant
                        let dominated_by_git_noise = event.paths.iter().all(|p| {
                            let p_str = p.to_string_lossy();
                            // Allow .git/refs, HEAD, index changes (branch/commit/stage ops)
                            if p_str.contains(".git/refs")
                                || p_str.contains(".git/HEAD")
                                || p_str.contains(".git/index")
                            {
                                return false; // not noise — allow these
                            }
                            p_str.contains(".git/") || p_str.contains(".git\\")
                        });
                        if dominated_by_git_noise {
                            continue;
                        }
                        pending = true;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Drain phase: if we have a pending change and enough time
                        // has passed since last emit, fire now.
                        if pending && last_emit.elapsed() >= Duration::from_millis(300) {
                            let _ = handle.emit("fs-changed", ());
                            last_emit = std::time::Instant::now();
                            pending = false;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                // Also emit immediately if pending and debounce window elapsed
                if pending && last_emit.elapsed() >= Duration::from_millis(500) {
                    let _ = handle.emit("fs-changed", ());
                    last_emit = std::time::Instant::now();
                    pending = false;
                }
            }
        });

        Ok(RepoWatcher { _watcher: watcher })
    }
}
