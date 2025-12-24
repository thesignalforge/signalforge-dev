use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NginxVhost {
    pub id: String,
    pub server_name: String,
    pub document_root: String,
    pub php_enabled: bool,
    pub ssl_enabled: bool,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
    pub config_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NginxTestResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
}

fn get_nginx_conf_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("SignalforgeData")
        .join("nginx")
        .join("conf.d")
}

fn get_vhosts_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("signalforge-dev")
        .join("vhosts.json")
}

fn load_vhosts() -> Result<Vec<NginxVhost>, String> {
    let path = get_vhosts_file();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read vhosts: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse vhosts: {}", e))
}

fn save_vhosts(vhosts: &[NginxVhost]) -> Result<(), String> {
    let path = get_vhosts_file();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(vhosts)
        .map_err(|e| format!("Failed to serialize vhosts: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write vhosts: {}", e))?;

    Ok(())
}

fn generate_vhost_config_content(vhost: &NginxVhost) -> String {
    let mut config = String::new();

    // HTTP server block
    config.push_str("server {\n");
    config.push_str("    listen 80;\n");
    config.push_str(&format!("    server_name {};\n", vhost.server_name));

    if vhost.ssl_enabled {
        config.push_str(&format!("    return 301 https://{}$request_uri;\n", vhost.server_name));
        config.push_str("}\n\n");

        // HTTPS server block
        config.push_str("server {\n");
        config.push_str("    listen 443 ssl http2;\n");
        config.push_str(&format!("    server_name {};\n\n", vhost.server_name));

        if let (Some(cert), Some(key)) = (&vhost.ssl_cert_path, &vhost.ssl_key_path) {
            config.push_str(&format!("    ssl_certificate {};\n", cert));
            config.push_str(&format!("    ssl_certificate_key {};\n", key));
            config.push_str("    ssl_protocols TLSv1.2 TLSv1.3;\n");
            config.push_str("    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;\n");
            config.push_str("    ssl_prefer_server_ciphers off;\n\n");
        }
    }

    config.push_str(&format!("    root {};\n", vhost.document_root));
    config.push_str("    index index.php index.html index.htm;\n\n");

    config.push_str("    location / {\n");
    config.push_str("        try_files $uri $uri/ /index.php?$query_string;\n");
    config.push_str("    }\n\n");

    if vhost.php_enabled {
        config.push_str("    location ~ \\.php$ {\n");
        config.push_str("        fastcgi_pass php:9000;\n");
        config.push_str("        fastcgi_index index.php;\n");
        config.push_str("        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;\n");
        config.push_str("        include fastcgi_params;\n");
        config.push_str("    }\n\n");
    }

    config.push_str("    location ~ /\\.ht {\n");
    config.push_str("        deny all;\n");
    config.push_str("    }\n\n");

    config.push_str("    access_log /var/log/nginx/access.log;\n");
    config.push_str("    error_log /var/log/nginx/error.log;\n");
    config.push_str("}\n");

    config
}

#[tauri::command]
pub async fn list_vhosts() -> Result<Vec<NginxVhost>, String> {
    load_vhosts()
}

#[tauri::command]
pub async fn get_vhost(id: String) -> Result<NginxVhost, String> {
    let vhosts = load_vhosts()?;
    vhosts
        .into_iter()
        .find(|v| v.id == id)
        .ok_or_else(|| format!("Vhost not found: {}", id))
}

#[tauri::command]
pub async fn create_vhost(
    server_name: String,
    document_root: String,
    php_enabled: bool,
    ssl_enabled: bool,
    ssl_cert_path: Option<String>,
    ssl_key_path: Option<String>,
) -> Result<NginxVhost, String> {
    let mut vhosts = load_vhosts()?;

    // Check for duplicate server names
    if vhosts.iter().any(|v| v.server_name == server_name) {
        return Err(format!("Vhost with server name '{}' already exists", server_name));
    }

    let nginx_conf_dir = get_nginx_conf_dir();
    fs::create_dir_all(&nginx_conf_dir)
        .map_err(|e| format!("Failed to create nginx conf directory: {}", e))?;

    let id = Uuid::new_v4().to_string();
    let config_filename = format!("{}.conf", server_name.replace('.', "_"));
    let config_path = nginx_conf_dir.join(&config_filename);

    let vhost = NginxVhost {
        id,
        server_name,
        document_root,
        php_enabled,
        ssl_enabled,
        ssl_cert_path,
        ssl_key_path,
        config_path: config_path.to_string_lossy().to_string(),
    };

    // Generate and write config file
    let config_content = generate_vhost_config_content(&vhost);
    fs::write(&config_path, &config_content)
        .map_err(|e| format!("Failed to write vhost config: {}", e))?;

    vhosts.push(vhost.clone());
    save_vhosts(&vhosts)?;

    Ok(vhost)
}

#[tauri::command]
pub async fn update_vhost(vhost: NginxVhost) -> Result<NginxVhost, String> {
    let mut vhosts = load_vhosts()?;

    let idx = vhosts
        .iter()
        .position(|v| v.id == vhost.id)
        .ok_or_else(|| format!("Vhost not found: {}", vhost.id))?;

    // Regenerate config file
    let config_content = generate_vhost_config_content(&vhost);
    fs::write(&vhost.config_path, &config_content)
        .map_err(|e| format!("Failed to write vhost config: {}", e))?;

    vhosts[idx] = vhost.clone();
    save_vhosts(&vhosts)?;

    Ok(vhost)
}

#[tauri::command]
pub async fn delete_vhost(id: String) -> Result<(), String> {
    let mut vhosts = load_vhosts()?;

    let idx = vhosts
        .iter()
        .position(|v| v.id == id)
        .ok_or_else(|| format!("Vhost not found: {}", id))?;

    // Delete config file
    let config_path = PathBuf::from(&vhosts[idx].config_path);
    if config_path.exists() {
        fs::remove_file(&config_path)
            .map_err(|e| format!("Failed to delete vhost config: {}", e))?;
    }

    vhosts.remove(idx);
    save_vhosts(&vhosts)?;

    Ok(())
}

#[tauri::command]
pub async fn get_vhost_config(id: String) -> Result<String, String> {
    let vhost = get_vhost(id).await?;

    fs::read_to_string(&vhost.config_path)
        .map_err(|e| format!("Failed to read vhost config: {}", e))
}

#[tauri::command]
pub async fn save_vhost_config(id: String, content: String) -> Result<(), String> {
    let vhost = get_vhost(id).await?;

    fs::write(&vhost.config_path, content)
        .map_err(|e| format!("Failed to write vhost config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn test_nginx_config() -> Result<NginxTestResult, String> {
    let output = Command::new("docker")
        .args(["exec", "signalforge-nginx", "nginx", "-t"])
        .output()
        .map_err(|e| format!("Failed to test nginx config: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let errors: Vec<String> = stderr
        .lines()
        .filter(|line| line.contains("error") || line.contains("failed"))
        .map(String::from)
        .collect();

    Ok(NginxTestResult {
        success: output.status.success(),
        output: if output.status.success() { stdout } else { stderr.clone() },
        errors,
    })
}

#[tauri::command]
pub async fn reload_nginx() -> Result<String, String> {
    let output = Command::new("docker")
        .args(["exec", "signalforge-nginx", "nginx", "-s", "reload"])
        .output()
        .map_err(|e| format!("Failed to reload nginx: {}", e))?;

    if output.status.success() {
        Ok("Nginx reloaded successfully".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn generate_default_nginx_config() -> Result<String, String> {
    Ok(r#"server {
    listen 80 default_server;
    listen [::]:80 default_server;

    root /var/www/html/public;
    index index.php index.html index.htm;

    server_name _;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass php:9000;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }

    location ~ /\.ht {
        deny all;
    }
}
"#.to_string())
}
