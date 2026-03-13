use crate::git::watcher::RepoWatcher;
use std::sync::Mutex;
use tauri::{AppHandle, State};

pub struct WatcherState(pub Mutex<Option<RepoWatcher>>);

#[tauri::command]
pub async fn start_watching(
    app: AppHandle,
    path: String,
    state: State<'_, WatcherState>,
) -> Result<(), String> {
    let watcher = RepoWatcher::new(&path, app)?;
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = Some(watcher);
    Ok(())
}

#[tauri::command]
pub async fn stop_watching(state: State<'_, WatcherState>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}
