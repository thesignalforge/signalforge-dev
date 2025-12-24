use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub compose_path: String,
    pub services: Vec<ServiceConfig>,
    pub volumes: Vec<VolumeMapping>,
    pub environment: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub image: String,
    pub enabled: bool,
    pub ports: Vec<PortMapping>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

fn get_projects_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("signalforge-dev")
        .join("projects.json")
}

fn load_projects() -> Result<Vec<Project>, String> {
    let path = get_projects_file();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read projects: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse projects: {}", e))
}

fn save_projects(projects: &[Project]) -> Result<(), String> {
    let path = get_projects_file();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(projects)
        .map_err(|e| format!("Failed to serialize projects: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write projects: {}", e))?;

    Ok(())
}

fn default_services() -> Vec<ServiceConfig> {
    vec![
        ServiceConfig {
            name: "nginx".to_string(),
            image: "nginx:latest".to_string(),
            enabled: true,
            ports: vec![
                PortMapping { host: 80, container: 80 },
                PortMapping { host: 443, container: 443 },
            ],
            environment: HashMap::new(),
        },
        ServiceConfig {
            name: "php".to_string(),
            image: "php:8.4-fpm".to_string(),
            enabled: true,
            ports: vec![],
            environment: HashMap::from([
                ("PHP_MEMORY_LIMIT".to_string(), "256M".to_string()),
                ("PHP_POST_MAX_SIZE".to_string(), "100M".to_string()),
                ("PHP_UPLOAD_MAX_FILESIZE".to_string(), "100M".to_string()),
            ]),
        },
        ServiceConfig {
            name: "mysql".to_string(),
            image: "mysql:8".to_string(),
            enabled: true,
            ports: vec![PortMapping { host: 3306, container: 3306 }],
            environment: HashMap::from([
                ("MYSQL_ROOT_PASSWORD".to_string(), "secret".to_string()),
                ("MYSQL_DATABASE".to_string(), "app".to_string()),
                ("MYSQL_USER".to_string(), "app".to_string()),
                ("MYSQL_PASSWORD".to_string(), "secret".to_string()),
            ]),
        },
        ServiceConfig {
            name: "postgres".to_string(),
            image: "postgres:17".to_string(),
            enabled: false,
            ports: vec![PortMapping { host: 5432, container: 5432 }],
            environment: HashMap::from([
                ("POSTGRES_DB".to_string(), "app".to_string()),
                ("POSTGRES_USER".to_string(), "app".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "secret".to_string()),
            ]),
        },
        ServiceConfig {
            name: "redis".to_string(),
            image: "redis:latest".to_string(),
            enabled: true,
            ports: vec![PortMapping { host: 6379, container: 6379 }],
            environment: HashMap::new(),
        },
    ]
}

#[tauri::command]
pub async fn list_projects() -> Result<Vec<Project>, String> {
    load_projects()
}

#[tauri::command]
pub async fn get_project(id: String) -> Result<Project, String> {
    let projects = load_projects()?;
    projects
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("Project not found: {}", id))
}

#[tauri::command]
pub async fn create_project(name: String, root_path: String) -> Result<Project, String> {
    let mut projects = load_projects()?;

    // Check for duplicate names
    if projects.iter().any(|p| p.name == name) {
        return Err(format!("Project with name '{}' already exists", name));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    // Create project directory structure
    let project_config_dir = PathBuf::from(&root_path).join(".signalforge");
    fs::create_dir_all(&project_config_dir)
        .map_err(|e| format!("Failed to create project config directory: {}", e))?;

    let compose_path = project_config_dir.join("docker-compose.yml");

    let project = Project {
        id,
        name,
        root_path: root_path.clone(),
        compose_path: compose_path.to_string_lossy().to_string(),
        services: default_services(),
        volumes: vec![VolumeMapping {
            host_path: root_path,
            container_path: "/var/www/html".to_string(),
            read_only: false,
        }],
        environment: HashMap::new(),
        created_at: now,
        updated_at: now,
    };

    // Generate initial docker-compose.yml
    let compose_content = generate_compose_content(&project)?;
    fs::write(&compose_path, compose_content)
        .map_err(|e| format!("Failed to write docker-compose.yml: {}", e))?;

    projects.push(project.clone());
    save_projects(&projects)?;

    Ok(project)
}

#[tauri::command]
pub async fn update_project(project: Project) -> Result<Project, String> {
    let mut projects = load_projects()?;

    let idx = projects
        .iter()
        .position(|p| p.id == project.id)
        .ok_or_else(|| format!("Project not found: {}", project.id))?;

    let mut updated = project.clone();
    updated.updated_at = Utc::now().timestamp();

    // Regenerate docker-compose.yml
    let compose_content = generate_compose_content(&updated)?;
    fs::write(&updated.compose_path, compose_content)
        .map_err(|e| format!("Failed to write docker-compose.yml: {}", e))?;

    projects[idx] = updated.clone();
    save_projects(&projects)?;

    Ok(updated)
}

#[tauri::command]
pub async fn delete_project(id: String) -> Result<(), String> {
    let mut projects = load_projects()?;

    let idx = projects
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| format!("Project not found: {}", id))?;

    // Remove .signalforge directory
    let project = &projects[idx];
    let config_dir = PathBuf::from(&project.root_path).join(".signalforge");
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir)
            .map_err(|e| format!("Failed to remove project config: {}", e))?;
    }

    projects.remove(idx);
    save_projects(&projects)?;

    Ok(())
}

fn generate_compose_content(project: &Project) -> Result<String, String> {
    let enabled_services: Vec<&ServiceConfig> = project.services.iter().filter(|s| s.enabled).collect();

    let mut content = String::from("version: '3.9'\n\nservices:\n");

    for service in &enabled_services {
        content.push_str(&format!("  {}:\n", service.name));
        content.push_str(&format!("    image: {}\n", service.image));
        content.push_str(&format!("    container_name: {}-{}\n", project.name.to_lowercase().replace(' ', "-"), service.name));

        // Ports
        if !service.ports.is_empty() {
            content.push_str("    ports:\n");
            for port in &service.ports {
                content.push_str(&format!("      - \"{}:{}\"\n", port.host, port.container));
            }
        }

        // Volumes for nginx and php
        if service.name == "nginx" || service.name == "php" {
            content.push_str("    volumes:\n");
            for vol in &project.volumes {
                let ro = if vol.read_only { ":ro" } else { "" };
                content.push_str(&format!("      - {}:{}{}\n", vol.host_path, vol.container_path, ro));
            }
        }

        // Named volumes for databases
        if service.name == "mysql" {
            content.push_str("    volumes:\n");
            content.push_str("      - mysql_data:/var/lib/mysql\n");
        } else if service.name == "postgres" {
            content.push_str("    volumes:\n");
            content.push_str("      - postgres_data:/var/lib/postgresql/data\n");
        } else if service.name == "redis" {
            content.push_str("    volumes:\n");
            content.push_str("      - redis_data:/data\n");
        }

        // Environment
        if !service.environment.is_empty() {
            content.push_str("    environment:\n");
            for (key, value) in &service.environment {
                content.push_str(&format!("      - {}={}\n", key, value));
            }
        }

        // Network
        content.push_str("    networks:\n");
        content.push_str("      - signalforge\n");

        // Dependencies
        if service.name == "nginx" && enabled_services.iter().any(|s| s.name == "php") {
            content.push_str("    depends_on:\n");
            content.push_str("      - php\n");
        }

        content.push_str("    restart: unless-stopped\n\n");
    }

    // Networks
    content.push_str("networks:\n");
    content.push_str("  signalforge:\n");
    content.push_str("    driver: bridge\n");
    content.push_str("    ipam:\n");
    content.push_str("      config:\n");
    content.push_str("        - subnet: 172.25.0.0/16\n\n");

    // Volumes
    content.push_str("volumes:\n");
    if enabled_services.iter().any(|s| s.name == "mysql") {
        content.push_str("  mysql_data:\n");
    }
    if enabled_services.iter().any(|s| s.name == "postgres") {
        content.push_str("  postgres_data:\n");
    }
    if enabled_services.iter().any(|s| s.name == "redis") {
        content.push_str("  redis_data:\n");
    }

    Ok(content)
}

#[tauri::command]
pub async fn get_compose_content(project_id: String) -> Result<String, String> {
    let project = get_project(project_id).await?;

    fs::read_to_string(&project.compose_path)
        .map_err(|e| format!("Failed to read docker-compose.yml: {}", e))
}

#[tauri::command]
pub async fn save_compose_content(project_id: String, content: String) -> Result<(), String> {
    let project = get_project(project_id).await?;

    fs::write(&project.compose_path, content)
        .map_err(|e| format!("Failed to write docker-compose.yml: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn compose_up(project_id: String) -> Result<String, String> {
    let project = get_project(project_id).await?;

    let output = Command::new("docker")
        .args(["compose", "-f", &project.compose_path, "up", "-d"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn compose_down(project_id: String) -> Result<String, String> {
    let project = get_project(project_id).await?;

    let output = Command::new("docker")
        .args(["compose", "-f", &project.compose_path, "down"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn compose_restart(project_id: String) -> Result<String, String> {
    let project = get_project(project_id).await?;

    let output = Command::new("docker")
        .args(["compose", "-f", &project.compose_path, "restart"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub async fn compose_status(project_id: String) -> Result<String, String> {
    let project = get_project(project_id).await?;

    let output = Command::new("docker")
        .args(["compose", "-f", &project.compose_path, "ps", "--format", "json"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
