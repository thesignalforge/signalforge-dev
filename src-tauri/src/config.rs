use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub projects_dir: String,
    pub ssl_dir: String,
    pub nginx_conf_dir: String,
    pub default_network_subnet: String,
    pub default_php_version: String,
    pub default_mysql_version: String,
    pub default_postgres_version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let signalforge_dir = home.join("SignalforgeData");

        Self {
            projects_dir: signalforge_dir.join("projects").to_string_lossy().to_string(),
            ssl_dir: signalforge_dir.join("ssl").to_string_lossy().to_string(),
            nginx_conf_dir: signalforge_dir.join("nginx").to_string_lossy().to_string(),
            default_network_subnet: "172.25.0.0/16".to_string(),
            default_php_version: "8.4".to_string(),
            default_mysql_version: "8".to_string(),
            default_postgres_version: "17".to_string(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("signalforge-dev")
        .join("config.json")
}

#[tauri::command]
pub async fn get_app_config() -> Result<AppConfig, String> {
    let config_path = get_config_path();

    if !config_path.exists() {
        let config = AppConfig::default();
        save_app_config_internal(&config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

#[tauri::command]
pub async fn save_app_config(config: AppConfig) -> Result<(), String> {
    save_app_config_internal(&config)
}

fn save_app_config_internal(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();

    // Create parent directories
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn ensure_directories() -> Result<(), String> {
    let config = get_app_config().await?;

    let dirs = vec![
        &config.projects_dir,
        &config.ssl_dir,
        &config.nginx_conf_dir,
    ];

    for dir in dirs {
        fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn reset_app_config() -> Result<AppConfig, String> {
    let config = AppConfig::default();
    save_app_config_internal(&config)?;
    Ok(config)
}
