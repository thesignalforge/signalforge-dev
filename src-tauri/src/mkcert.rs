use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MkcertStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub ca_installed: bool,
    pub ca_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Certificate {
    pub domain: String,
    pub cert_path: String,
    pub key_path: String,
    pub created_at: i64,
    pub is_wildcard: bool,
}

fn get_ssl_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("SignalforgeData")
        .join("ssl")
}

fn get_certs_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("signalforge-dev")
        .join("certificates.json")
}

fn load_certificates() -> Result<Vec<Certificate>, String> {
    let path = get_certs_file();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read certificates: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse certificates: {}", e))
}

fn save_certificates(certs: &[Certificate]) -> Result<(), String> {
    let path = get_certs_file();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(certs)
        .map_err(|e| format!("Failed to serialize certificates: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write certificates: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_mkcert_status() -> Result<MkcertStatus, String> {
    // Check if mkcert is installed
    let version_output = Command::new("mkcert")
        .arg("-version")
        .output();

    let installed = version_output.as_ref().map(|o| o.status.success()).unwrap_or(false);
    let version = version_output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    // Check if CA is installed
    let ca_installed = if installed {
        let caroot_output = Command::new("mkcert")
            .arg("-CAROOT")
            .output()
            .ok();

        caroot_output
            .as_ref()
            .map(|o| {
                let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                PathBuf::from(&path).join("rootCA.pem").exists()
            })
            .unwrap_or(false)
    } else {
        false
    };

    let ca_path = if ca_installed {
        Command::new("mkcert")
            .arg("-CAROOT")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };

    Ok(MkcertStatus {
        installed,
        version,
        ca_installed,
        ca_path,
    })
}

#[tauri::command]
pub async fn install_mkcert_ca() -> Result<String, String> {
    let output = Command::new("mkcert")
        .arg("-install")
        .output()
        .map_err(|e| format!("Failed to install mkcert CA: {}", e))?;

    if output.status.success() {
        Ok("CA installed successfully. You may need to restart your browser.".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn generate_certificate(domain: String, wildcard: bool) -> Result<Certificate, String> {
    let ssl_dir = get_ssl_dir();
    fs::create_dir_all(&ssl_dir)
        .map_err(|e| format!("Failed to create SSL directory: {}", e))?;

    // Determine the actual domain to generate cert for
    let cert_domain = if wildcard {
        format!("*.{}", domain)
    } else {
        domain.clone()
    };

    // Clean filename (remove * for wildcard)
    let filename_base = domain.replace('.', "_");
    let cert_path = ssl_dir.join(format!("{}.crt", filename_base));
    let key_path = ssl_dir.join(format!("{}.key", filename_base));

    // Build mkcert command
    let mut args = vec![
        "-cert-file".to_string(),
        cert_path.to_string_lossy().to_string(),
        "-key-file".to_string(),
        key_path.to_string_lossy().to_string(),
    ];

    // Add domain and wildcard if needed
    args.push(domain.clone());
    if wildcard {
        args.push(cert_domain.clone());
    }

    let output = Command::new("mkcert")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to generate certificate: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let cert = Certificate {
        domain: domain.clone(),
        cert_path: cert_path.to_string_lossy().to_string(),
        key_path: key_path.to_string_lossy().to_string(),
        created_at: Utc::now().timestamp(),
        is_wildcard: wildcard,
    };

    // Save to certificates list
    let mut certs = load_certificates()?;

    // Remove existing cert for same domain
    certs.retain(|c| c.domain != domain);
    certs.push(cert.clone());
    save_certificates(&certs)?;

    Ok(cert)
}

#[tauri::command]
pub async fn list_certificates() -> Result<Vec<Certificate>, String> {
    let certs = load_certificates()?;

    // Filter out certificates whose files no longer exist
    let valid_certs: Vec<Certificate> = certs
        .into_iter()
        .filter(|c| {
            PathBuf::from(&c.cert_path).exists() && PathBuf::from(&c.key_path).exists()
        })
        .collect();

    Ok(valid_certs)
}

#[tauri::command]
pub async fn get_certificate(domain: String) -> Result<Certificate, String> {
    let certs = load_certificates()?;
    certs
        .into_iter()
        .find(|c| c.domain == domain)
        .ok_or_else(|| format!("Certificate not found for domain: {}", domain))
}

#[tauri::command]
pub async fn delete_certificate(domain: String) -> Result<(), String> {
    let mut certs = load_certificates()?;

    let idx = certs
        .iter()
        .position(|c| c.domain == domain)
        .ok_or_else(|| format!("Certificate not found: {}", domain))?;

    let cert = &certs[idx];

    // Delete certificate files
    let cert_path = PathBuf::from(&cert.cert_path);
    let key_path = PathBuf::from(&cert.key_path);

    if cert_path.exists() {
        fs::remove_file(&cert_path)
            .map_err(|e| format!("Failed to delete certificate file: {}", e))?;
    }

    if key_path.exists() {
        fs::remove_file(&key_path)
            .map_err(|e| format!("Failed to delete key file: {}", e))?;
    }

    certs.remove(idx);
    save_certificates(&certs)?;

    Ok(())
}

#[tauri::command]
pub async fn get_mkcert_install_instructions() -> Result<String, String> {
    let os = std::env::consts::OS;

    let instructions = match os {
        "linux" => r#"## Install mkcert on Linux

### Ubuntu/Debian:
```bash
sudo apt install libnss3-tools
wget https://github.com/FiloSottile/mkcert/releases/download/v1.4.4/mkcert-v1.4.4-linux-amd64
sudo mv mkcert-v1.4.4-linux-amd64 /usr/local/bin/mkcert
sudo chmod +x /usr/local/bin/mkcert
```

### Arch Linux:
```bash
sudo pacman -S mkcert
```

### Using Homebrew (Linux):
```bash
brew install mkcert
```

After installation, run `mkcert -install` to install the CA."#,

        "macos" => r#"## Install mkcert on macOS

### Using Homebrew (recommended):
```bash
brew install mkcert
brew install nss  # if you use Firefox
```

After installation, run `mkcert -install` to install the CA."#,

        "windows" => r#"## Install mkcert on Windows

### Using Chocolatey:
```powershell
choco install mkcert
```

### Using Scoop:
```powershell
scoop install mkcert
```

After installation, run `mkcert -install` to install the CA."#,

        _ => "Please visit https://github.com/FiloSottile/mkcert for installation instructions.",
    };

    Ok(instructions.to_string())
}
