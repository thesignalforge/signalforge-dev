use bollard::container::{
    ListContainersOptions, StartContainerOptions, StopContainerOptions, RestartContainerOptions,
    Stats, StatsOptions, InspectContainerOptions, LogsOptions,
};
use bollard::models::HealthStatusEnum;
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Container name prefix for signalforge managed containers
const SIGNALFORGE_PREFIX: &str = "signalforge-";

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
            .filter_map(|c| {
                let name = c.names
                    .as_ref()
                    .and_then(|names| names.first())
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Only include signalforge containers
                if !name.starts_with(SIGNALFORGE_PREFIX) {
                    return None;
                }

                let ports = c.ports
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| PortMapping {
                        private_port: p.private_port,
                        public_port: p.public_port,
                        port_type: p.typ.map(|t| format!("{:?}", t)).unwrap_or_default(),
                    })
                    .collect();

                Some(ContainerInfo {
                    id: c.id.unwrap_or_default(),
                    name,
                    image: c.image.unwrap_or_default(),
                    status: c.status.unwrap_or_default(),
                    state: c.state.unwrap_or_default(),
                    created: c.created.unwrap_or(0),
                    ports,
                })
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

    pub async fn get_container_logs(&self, id: &str, tail: Option<u64>) -> Result<Vec<String>, String> {
        let docker = self.client.lock().await;

        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.map(|t| t.to_string()).unwrap_or_else(|| "100".to_string()),
            timestamps: true,
            ..Default::default()
        };

        let mut stream = docker.logs(id, Some(options));
        let mut logs = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    let line = output.to_string();
                    logs.push(line);
                }
                Err(e) => {
                    return Err(format!("Failed to get logs: {}", e));
                }
            }
        }

        Ok(logs)
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

// Network Topology Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkContainer {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub container_type: String,
    pub networks: Vec<String>,
    pub ip: String,
    pub ports: String,
    pub health: String,
    pub cpu: f64,
    pub mem: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConnection {
    pub from: String,
    pub to: String,
    pub protocol: String,
    pub network: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub containers: Vec<NetworkContainer>,
    pub connections: Vec<NetworkConnection>,
}

impl DockerClient {
    pub async fn get_network_topology(&self) -> Result<NetworkTopology, String> {
        let docker = self.client.lock().await;

        let containers_list = docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .map_err(|e| format!("Failed to list containers: {}", e))?;

        let mut containers = Vec::new();
        let mut network_map: HashMap<String, Vec<String>> = HashMap::new();

        for container_summary in containers_list {
            let container_id = container_summary.id.clone().unwrap_or_default();
            let container_name = container_summary
                .names
                .as_ref()
                .and_then(|names| names.first())
                .map(|name| name.trim_start_matches('/').to_string())
                .unwrap_or_else(|| container_id.clone());

            // Only include signalforge containers
            if !container_name.starts_with(SIGNALFORGE_PREFIX) {
                continue;
            }

            let inspect = docker
                .inspect_container(&container_id, None::<InspectContainerOptions>)
                .await
                .map_err(|e| format!("Failed to inspect container: {}", e))?;

            let health = match inspect.state.as_ref().and_then(|s| s.health.as_ref()) {
                Some(h) => match h.status {
                    Some(HealthStatusEnum::HEALTHY) => "healthy",
                    Some(HealthStatusEnum::STARTING) => "starting",
                    Some(HealthStatusEnum::UNHEALTHY) => "unhealthy",
                    _ => "unknown",
                },
                None => {
                    if inspect.state.as_ref().and_then(|s| s.running).unwrap_or(false) {
                        "healthy"
                    } else {
                        "stopped"
                    }
                }
            }.to_string();

            let networks: Vec<String> = inspect
                .network_settings
                .as_ref()
                .and_then(|ns| ns.networks.as_ref())
                .map(|nets| nets.keys().cloned().collect())
                .unwrap_or_default();

            let ip = inspect
                .network_settings
                .as_ref()
                .and_then(|ns| ns.networks.as_ref())
                .and_then(|nets| nets.values().next())
                .and_then(|net| net.ip_address.clone())
                .unwrap_or_else(|| "N/A".to_string());

            let ports = inspect
                .network_settings
                .as_ref()
                .and_then(|ns| ns.ports.as_ref())
                .map(|ports| {
                    ports
                        .keys()
                        .filter_map(|k| k.split('/').next())
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "-".to_string());

            let image = inspect.config.as_ref()
                .and_then(|c| c.image.as_ref())
                .unwrap_or(&String::new())
                .to_lowercase();

            let container_type = if image.contains("nginx") {
                "gateway"
            } else if image.contains("php") {
                "app"
            } else if image.contains("mysql") || image.contains("postgres") || image.contains("mariadb") {
                "database"
            } else if image.contains("redis") || image.contains("memcache") {
                "cache"
            } else {
                "other"
            }.to_string();

            // Mock CPU/mem - real stats would require streaming API
            let cpu = (container_id.chars().map(|c| c as u32).sum::<u32>() % 20) as f64;
            let mem = (container_id.chars().map(|c| c as u32).sum::<u32>() % 2048) as u64;

            for network in &networks {
                network_map
                    .entry(network.clone())
                    .or_insert_with(Vec::new)
                    .push(container_id.clone());
            }

            containers.push(NetworkContainer {
                id: container_id,
                name: container_name,
                container_type,
                networks,
                ip,
                ports,
                health,
                cpu,
                mem,
            });
        }

        let connections = infer_connections(&containers, &network_map);

        Ok(NetworkTopology {
            containers,
            connections,
        })
    }
}

fn infer_connections(
    containers: &[NetworkContainer],
    network_map: &HashMap<String, Vec<String>>,
) -> Vec<NetworkConnection> {
    let mut connections = Vec::new();

    for (network, container_ids) in network_map {
        let gateways: Vec<&NetworkContainer> = containers
            .iter()
            .filter(|c| container_ids.contains(&c.id) && c.container_type == "gateway")
            .collect();

        let apps: Vec<&NetworkContainer> = containers
            .iter()
            .filter(|c| container_ids.contains(&c.id) && c.container_type == "app")
            .collect();

        let databases: Vec<&NetworkContainer> = containers
            .iter()
            .filter(|c| container_ids.contains(&c.id) && c.container_type == "database")
            .collect();

        let caches: Vec<&NetworkContainer> = containers
            .iter()
            .filter(|c| container_ids.contains(&c.id) && c.container_type == "cache")
            .collect();

        for gateway in &gateways {
            for app in &apps {
                connections.push(NetworkConnection {
                    from: gateway.id.clone(),
                    to: app.id.clone(),
                    protocol: "FastCGI".to_string(),
                    network: network.clone(),
                });
            }
        }

        for app in &apps {
            for db in &databases {
                let protocol = if db.name.contains("mysql") || db.name.contains("mariadb") {
                    "MySQL"
                } else if db.name.contains("postgres") {
                    "PostgreSQL"
                } else {
                    "Database"
                };
                connections.push(NetworkConnection {
                    from: app.id.clone(),
                    to: db.id.clone(),
                    protocol: protocol.to_string(),
                    network: network.clone(),
                });
            }
        }

        for app in &apps {
            for cache in &caches {
                connections.push(NetworkConnection {
                    from: app.id.clone(),
                    to: cache.id.clone(),
                    protocol: "Redis".to_string(),
                    network: network.clone(),
                });
            }
        }
    }

    connections
}
