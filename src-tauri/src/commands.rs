use crate::docker::{ContainerInfo, ContainerStats, DockerClient, DockerInfo, NetworkTopology};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

pub struct AppState {
    pub docker: Arc<Mutex<Option<DockerClient>>>,
}

impl AppState {
    pub fn new() -> Self {
        let docker = DockerClient::new().ok();
        Self {
            docker: Arc::new(Mutex::new(docker)),
        }
    }
}

#[tauri::command]
pub async fn check_docker_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let docker = state.docker.lock().await;
    Ok(docker.is_some())
}

#[tauri::command]
pub async fn connect_docker(state: State<'_, AppState>) -> Result<bool, String> {
    let mut docker = state.docker.lock().await;
    match DockerClient::new() {
        Ok(client) => {
            *docker = Some(client);
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn list_containers(state: State<'_, AppState>) -> Result<Vec<ContainerInfo>, String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.list_containers().await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn start_container(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.start_container(&id).await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn stop_container(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.stop_container(&id).await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn restart_container(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.restart_container(&id).await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn get_container_stats(
    id: String,
    state: State<'_, AppState>,
) -> Result<ContainerStats, String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.get_container_stats(&id).await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn get_container_logs(
    id: String,
    tail: Option<u64>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.get_container_logs(&id, tail).await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn get_docker_info(state: State<'_, AppState>) -> Result<DockerInfo, String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.get_docker_info().await,
        None => Err("Docker is not connected".to_string()),
    }
}

#[tauri::command]
pub async fn get_network_topology(state: State<'_, AppState>) -> Result<NetworkTopology, String> {
    let docker = state.docker.lock().await;
    match docker.as_ref() {
        Some(client) => client.get_network_topology().await,
        None => Err("Docker is not connected".to_string()),
    }
}
