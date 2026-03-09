mod commands;
mod git;
mod recent;
mod theme;

use commands::{blame, branches, commits, diff, recent as recent_cmd, remotes, repo, search, stash, status, theme as theme_cmd};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Repository
            repo::get_cli_args,
            repo::open_repository,
            repo::init_repository,
            // Commits
            commits::load_commits,
            commits::get_commit_detail,
            commits::compute_graph,
            commits::create_commit,
            // Branches
            branches::create_branch,
            branches::checkout_branch,
            branches::delete_branch,
            branches::rename_branch,
            branches::merge_branch,
            branches::reset_to_commit,
            // Remotes
            remotes::fetch_remote,
            remotes::pull,
            remotes::push,
            // Diff
            diff::get_working_diff,
            diff::get_commit_diff,
            // Status
            status::get_status,
            status::stage_file,
            status::unstage_file,
            status::stage_all,
            status::unstage_all,
            status::discard_file,
            status::delete_file,
            status::add_to_gitignore,
            // Stash
            stash::stash_list,
            stash::stash_save,
            stash::stash_pop,
            stash::stash_apply,
            stash::stash_drop,
            // Blame
            blame::get_blame,
            // Search
            search::search_commits,
            // Theme
            theme_cmd::load_user_themes,
            theme_cmd::save_user_theme,
            theme_cmd::delete_user_theme,
            // Recent repos
            recent_cmd::load_recent_repos,
            recent_cmd::add_recent_repo,
            recent_cmd::remove_recent_repo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
