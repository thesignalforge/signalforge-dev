use bollard::container::{
    ListContainersOptions, StartContainerOptions, StopContainerOptions, RestartContainerOptions,
    Stats, StatsOptions,
};
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: i64,
    pub ports: Vec<PortMapping>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub port_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx: u64,
    pub network_tx: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerInfo {
    pub containers_running: i64,
    pub containers_paused: i64,
    pub containers_stopped: i64,
    pub images: i64,
    pub docker_version: String,
    pub os_type: String,
    pub architecture: String,
    pub memory_total: i64,
    pub cpus: i64,
}

pub struct DockerClient {
    client: Arc<Mutex<Docker>>,
}

impl DockerClient {
    pub fn new() -> Result<Self, String> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

        Ok(Self {
            client: Arc::new(Mutex::new(docker)),
        })
    }

    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String> {
        let docker = self.client.lock().await;

        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };

        let containers = docker
            .list_containers(Some(options))
            .await
            .map_err(|e| format!("Failed to list containers: {}", e))?;

        let container_infos: Vec<ContainerInfo> = containers
            .into_iter()
            .map(|c| {
                let name = c.names
                    .unwrap_or_default()
                    .first()
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let ports = c.ports
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| PortMapping {
                        private_port: p.private_port,
                        public_port: p.public_port,
                        port_type: p.typ.map(|t| format!("{:?}", t)).unwrap_or_default(),
                    })
                    .collect();

                ContainerInfo {
                    id: c.id.unwrap_or_default(),
                    name,
                    image: c.image.unwrap_or_default(),
                    status: c.status.unwrap_or_default(),
                    state: c.state.unwrap_or_default(),
                    created: c.created.unwrap_or(0),
                    ports,
                }
            })
            .collect();

        Ok(container_infos)
    }

    pub async fn start_container(&self, id: &str) -> Result<(), String> {
        let docker = self.client.lock().await;
        docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| format!("Failed to start container: {}", e))
    }

    pub async fn stop_container(&self, id: &str) -> Result<(), String> {
        let docker = self.client.lock().await;
        docker
            .stop_container(id, Some(StopContainerOptions { t: 10 }))
            .await
            .map_err(|e| format!("Failed to stop container: {}", e))
    }

    pub async fn restart_container(&self, id: &str) -> Result<(), String> {
        let docker = self.client.lock().await;
        docker
            .restart_container(id, Some(RestartContainerOptions { t: 10 }))
            .await
            .map_err(|e| format!("Failed to restart container: {}", e))
    }

    pub async fn get_container_stats(&self, id: &str) -> Result<ContainerStats, String> {
        let docker = self.client.lock().await;

        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stream = docker.stats(id, Some(options));

        if let Some(result) = stream.next().await {
            let stats = result.map_err(|e| format!("Failed to get stats: {}", e))?;
            return Ok(calculate_stats(&stats));
        }

        Err("No stats available".to_string())
    }

    pub async fn get_docker_info(&self) -> Result<DockerInfo, String> {
        let docker = self.client.lock().await;

        let info = docker
            .info()
            .await
            .map_err(|e| format!("Failed to get Docker info: {}", e))?;

        Ok(DockerInfo {
            containers_running: info.containers_running.unwrap_or(0) as i64,
            containers_paused: info.containers_paused.unwrap_or(0) as i64,
            containers_stopped: info.containers_stopped.unwrap_or(0) as i64,
            images: info.images.unwrap_or(0) as i64,
            docker_version: info.server_version.unwrap_or_default(),
            os_type: info.os_type.unwrap_or_default(),
            architecture: info.architecture.unwrap_or_default(),
            memory_total: info.mem_total.unwrap_or(0),
            cpus: info.ncpu.unwrap_or(0) as i64,
        })
    }
}

fn calculate_stats(stats: &Stats) -> ContainerStats {
    let cpu_percent = calculate_cpu_percent(stats);

    let memory_usage = stats.memory_stats.usage.unwrap_or(0);
    let memory_limit = stats.memory_stats.limit.unwrap_or(1);
    let memory_percent = if memory_limit > 0 {
        (memory_usage as f64 / memory_limit as f64) * 100.0
    } else {
        0.0
    };

    let (network_rx, network_tx) = stats
        .networks
        .as_ref()
        .map(|networks| {
            networks.values().fold((0u64, 0u64), |(rx, tx), net| {
                (rx + net.rx_bytes, tx + net.tx_bytes)
            })
        })
        .unwrap_or((0, 0));

    ContainerStats {
        cpu_percent,
        memory_usage,
        memory_limit,
        memory_percent,
        network_rx,
        network_tx,
    }
}

fn calculate_cpu_percent(stats: &Stats) -> f64 {
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
        - stats.precpu_stats.cpu_usage.total_usage as f64;

    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    let cpu_count = stats
        .cpu_stats
        .online_cpus
        .unwrap_or(1) as f64;

    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * cpu_count * 100.0
    } else {
        0.0
    }
}
