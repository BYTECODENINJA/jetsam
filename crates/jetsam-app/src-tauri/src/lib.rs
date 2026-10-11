#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod commands;
mod daemon;
mod helper;
mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_inventory,
            commands::list_packages,
            commands::package_detail,
            commands::cleanup_suggestions,
            commands::usage_map,
            commands::ignore_list,
            commands::set_ignored,
            commands::set_opt_in,
            commands::wipe_data,
            commands::app_status,
            commands::dry_run,
            commands::apply_remove,
            commands::run_upgrade,
            commands::check_updates,
            commands::start_daemon,
            commands::helper_available,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("Jetsam");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
