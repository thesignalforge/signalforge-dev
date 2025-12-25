pub mod commands;
pub mod compose;
pub mod config;
pub mod dnsmasq;
pub mod docker;
pub mod filesystem;
pub mod mkcert;
pub mod nginx;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Docker commands
            commands::check_docker_connection,
            commands::connect_docker,
            commands::list_containers,
            commands::start_container,
            commands::stop_container,
            commands::restart_container,
            commands::get_container_stats,
            commands::get_docker_info,
            commands::get_network_topology,
            // Filesystem commands
            filesystem::list_directory,
            filesystem::list_directory_recursive,
            filesystem::read_file,
            filesystem::write_file,
            filesystem::create_directory,
            filesystem::delete_path,
            filesystem::path_exists,
            filesystem::get_home_dir,
            filesystem::get_app_data_dir,
            // Config commands
            config::get_app_config,
            config::save_app_config,
            config::ensure_directories,
            config::reset_app_config,
            // Compose commands
            compose::list_projects,
            compose::get_project,
            compose::create_project,
            compose::update_project,
            compose::delete_project,
            compose::get_compose_content,
            compose::save_compose_content,
            compose::compose_up,
            compose::compose_down,
            compose::compose_restart,
            compose::compose_status,
            // Nginx commands
            nginx::list_vhosts,
            nginx::get_vhost,
            nginx::create_vhost,
            nginx::update_vhost,
            nginx::delete_vhost,
            nginx::get_vhost_config,
            nginx::save_vhost_config,
            nginx::test_nginx_config,
            nginx::reload_nginx,
            nginx::generate_default_nginx_config,
            // mkcert commands
            mkcert::get_mkcert_status,
            mkcert::install_mkcert_ca,
            mkcert::generate_certificate,
            mkcert::list_certificates,
            mkcert::get_certificate,
            mkcert::delete_certificate,
            mkcert::get_mkcert_install_instructions,
            // dnsmasq commands
            dnsmasq::get_dnsmasq_status,
            dnsmasq::configure_sig_tld,
            dnsmasq::list_sig_domains,
            dnsmasq::add_sig_domain,
            dnsmasq::remove_sig_domain,
            dnsmasq::test_domain_resolution,
            dnsmasq::get_hosts_entries,
            dnsmasq::get_dnsmasq_install_instructions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
