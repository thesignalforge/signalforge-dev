use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::net::TcpStream;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsmasqStatus {
    pub installed: bool,
    pub running: bool,
    pub config_path: Option<String>,
    pub sig_configured: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SigDomain {
    pub name: String,
    pub full_domain: String,
    pub ip_address: String,
    pub in_hosts: bool,
    pub in_dnsmasq: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsTestResult {
    pub domain: String,
    pub resolves: bool,
    pub ip_address: Option<String>,
    pub method: String,
}

fn get_domains_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("signalforge-dev")
        .join("sig_domains.json")
}

fn load_domains() -> Result<Vec<SigDomain>, String> {
    let path = get_domains_file();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read domains: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse domains: {}", e))
}

fn save_domains(domains: &[SigDomain]) -> Result<(), String> {
    let path = get_domains_file();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(domains)
        .map_err(|e| format!("Failed to serialize domains: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write domains: {}", e))?;

    Ok(())
}

fn get_dnsmasq_config_path() -> Option<PathBuf> {
    let possible_paths = vec![
        PathBuf::from("/etc/dnsmasq.d"),
        PathBuf::from("/usr/local/etc/dnsmasq.d"),
        PathBuf::from("/opt/homebrew/etc/dnsmasq.d"),
    ];

    possible_paths.into_iter().find(|p| p.exists())
}

fn check_dnsmasq_running() -> bool {
    let output = Command::new("pgrep")
        .arg("dnsmasq")
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

#[tauri::command]
pub async fn get_dnsmasq_status() -> Result<DnsmasqStatus, String> {
    // Check if dnsmasq is installed
    let installed = Command::new("which")
        .arg("dnsmasq")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let running = check_dnsmasq_running();

    let config_path = get_dnsmasq_config_path()
        .map(|p| p.to_string_lossy().to_string());

    // Check if .sig TLD is configured
    let sig_configured = if let Some(ref path) = config_path {
        let sig_conf = PathBuf::from(path).join("sig.conf");
        sig_conf.exists()
    } else {
        false
    };

    Ok(DnsmasqStatus {
        installed,
        running,
        config_path,
        sig_configured,
    })
}

#[tauri::command]
pub async fn configure_sig_tld() -> Result<String, String> {
    let config_dir = get_dnsmasq_config_path()
        .ok_or_else(|| "dnsmasq config directory not found. Please install dnsmasq first.".to_string())?;

    let sig_conf = config_dir.join("sig.conf");

    // Create the .sig TLD configuration
    let config_content = r#"# Signalforge Dev - .sig TLD configuration
# Route all .sig domains to localhost
address=/.sig/127.0.0.1
"#;

    fs::write(&sig_conf, config_content)
        .map_err(|e| format!("Failed to write dnsmasq config: {}. You may need sudo permissions.", e))?;

    // Try to restart dnsmasq
    let restart_result = Command::new("sudo")
        .args(["systemctl", "restart", "dnsmasq"])
        .output();

    match restart_result {
        Ok(output) if output.status.success() => {
            Ok("dnsmasq configured for .sig TLD and restarted successfully".to_string())
        }
        _ => {
            // Try brew services for macOS
            let brew_result = Command::new("brew")
                .args(["services", "restart", "dnsmasq"])
                .output();

            match brew_result {
                Ok(output) if output.status.success() => {
                    Ok("dnsmasq configured for .sig TLD and restarted successfully".to_string())
                }
                _ => {
                    Ok("dnsmasq configured. Please restart dnsmasq manually.".to_string())
                }
            }
        }
    }
}

#[tauri::command]
pub async fn list_sig_domains() -> Result<Vec<SigDomain>, String> {
    load_domains()
}

#[tauri::command]
pub async fn add_sig_domain(name: String, ip_address: Option<String>) -> Result<SigDomain, String> {
    let mut domains = load_domains()?;

    let full_domain = if name.ends_with(".sig") {
        name.clone()
    } else {
        format!("{}.sig", name)
    };

    // Check for duplicates
    if domains.iter().any(|d| d.full_domain == full_domain) {
        return Err(format!("Domain '{}' already exists", full_domain));
    }

    let ip = ip_address.unwrap_or_else(|| "127.0.0.1".to_string());

    // Add to /etc/hosts
    let add_to_hosts = add_hosts_entry_internal(&full_domain, &ip);

    let domain = SigDomain {
        name: name.trim_end_matches(".sig").to_string(),
        full_domain: full_domain.clone(),
        ip_address: ip,
        in_hosts: add_to_hosts.is_ok(),
        in_dnsmasq: false, // dnsmasq handles wildcards, individual entries not needed
    };

    domains.push(domain.clone());
    save_domains(&domains)?;

    if add_to_hosts.is_err() {
        return Ok(domain); // Return domain but note hosts wasn't updated
    }

    Ok(domain)
}

fn add_hosts_entry_internal(domain: &str, ip: &str) -> Result<(), String> {
    let hosts_path = PathBuf::from("/etc/hosts");

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| format!("Failed to read /etc/hosts: {}", e))?;

    // Check if entry already exists
    if content.lines().any(|line| line.contains(domain)) {
        return Ok(()); // Already exists
    }

    let new_entry = format!("\n{} {}", ip, domain);

    // Try to append using sudo
    let echo_cmd = format!("echo '{}' | sudo tee -a /etc/hosts", new_entry.trim());

    let output = Command::new("sh")
        .args(["-c", &echo_cmd])
        .output()
        .map_err(|e| format!("Failed to update /etc/hosts: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to update /etc/hosts. You may need to add the entry manually.".to_string())
    }
}

#[tauri::command]
pub async fn remove_sig_domain(name: String) -> Result<(), String> {
    let mut domains = load_domains()?;

    let full_domain = if name.ends_with(".sig") {
        name.clone()
    } else {
        format!("{}.sig", name)
    };

    let idx = domains
        .iter()
        .position(|d| d.full_domain == full_domain)
        .ok_or_else(|| format!("Domain not found: {}", full_domain))?;

    // Try to remove from /etc/hosts
    let _ = remove_hosts_entry_internal(&full_domain);

    domains.remove(idx);
    save_domains(&domains)?;

    Ok(())
}

fn remove_hosts_entry_internal(domain: &str) -> Result<(), String> {
    let hosts_path = PathBuf::from("/etc/hosts");

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| format!("Failed to read /etc/hosts: {}", e))?;

    let new_content: String = content
        .lines()
        .filter(|line| !line.contains(domain))
        .collect::<Vec<&str>>()
        .join("\n");

    // Write using sudo
    let cmd = format!("echo '{}' | sudo tee /etc/hosts", new_content);

    Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .map_err(|e| format!("Failed to update /etc/hosts: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn test_domain_resolution(domain: String) -> Result<DnsTestResult, String> {
    let full_domain = if domain.ends_with(".sig") {
        domain.clone()
    } else {
        format!("{}.sig", domain)
    };

    // Try to resolve using getent/host command
    let output = Command::new("getent")
        .args(["hosts", &full_domain])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let ip = stdout.split_whitespace().next().map(String::from);

            return Ok(DnsTestResult {
                domain: full_domain,
                resolves: true,
                ip_address: ip,
                method: "getent".to_string(),
            });
        }
    }

    // Fallback: try to connect to port 80
    let test_addr = format!("{}:80", full_domain);
    if TcpStream::connect(&test_addr).is_ok() {
        return Ok(DnsTestResult {
            domain: full_domain,
            resolves: true,
            ip_address: Some("127.0.0.1".to_string()),
            method: "tcp_connect".to_string(),
        });
    }

    Ok(DnsTestResult {
        domain: full_domain,
        resolves: false,
        ip_address: None,
        method: "none".to_string(),
    })
}

#[tauri::command]
pub async fn get_hosts_entries() -> Result<Vec<SigDomain>, String> {
    let hosts_path = PathBuf::from("/etc/hosts");

    let content = fs::read_to_string(&hosts_path)
        .map_err(|e| format!("Failed to read /etc/hosts: {}", e))?;

    let sig_entries: Vec<SigDomain> = content
        .lines()
        .filter(|line| !line.trim().starts_with('#') && line.contains(".sig"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let ip = parts[0].to_string();
                let domain = parts[1].to_string();
                if domain.ends_with(".sig") {
                    return Some(SigDomain {
                        name: domain.trim_end_matches(".sig").to_string(),
                        full_domain: domain,
                        ip_address: ip,
                        in_hosts: true,
                        in_dnsmasq: false,
                    });
                }
            }
            None
        })
        .collect();

    Ok(sig_entries)
}

#[tauri::command]
pub async fn get_dnsmasq_install_instructions() -> Result<String, String> {
    let os = std::env::consts::OS;

    let instructions = match os {
        "linux" => r#"## Install dnsmasq on Linux

### Ubuntu/Debian:
```bash
sudo apt install dnsmasq
```

### Arch Linux:
```bash
sudo pacman -S dnsmasq
```

### After installation:
1. The app will create `/etc/dnsmasq.d/sig.conf` automatically
2. Restart dnsmasq: `sudo systemctl restart dnsmasq`
3. Configure your system to use 127.0.0.1 as DNS resolver

### Configure NetworkManager (if using):
```bash
echo "dns=dnsmasq" | sudo tee /etc/NetworkManager/conf.d/dnsmasq.conf
sudo systemctl restart NetworkManager
```"#,

        "macos" => r#"## Install dnsmasq on macOS

### Using Homebrew:
```bash
brew install dnsmasq

# Start dnsmasq
sudo brew services start dnsmasq

# Configure resolver for .sig domains
sudo mkdir -p /etc/resolver
echo "nameserver 127.0.0.1" | sudo tee /etc/resolver/sig
```

After installation, click "Configure .sig TLD" to set up the configuration."#,

        _ => "Please install dnsmasq according to your system's package manager.",
    };

    Ok(instructions.to_string())
}
