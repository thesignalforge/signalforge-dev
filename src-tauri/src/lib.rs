pub mod commands;
pub mod docker;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::check_docker_connection,
            commands::connect_docker,
            commands::list_containers,
            commands::start_container,
            commands::stop_container,
            commands::restart_container,
            commands::get_container_stats,
            commands::get_docker_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
