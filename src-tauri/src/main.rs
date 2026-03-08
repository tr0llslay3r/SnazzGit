// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix WebKitGTK crash on Wayland compositors
    #[cfg(target_os = "linux")]
    {
        // SAFETY: Called before any threads are spawned in main()
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }

    snazzgit_lib::run();
}
