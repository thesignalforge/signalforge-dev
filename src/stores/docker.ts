import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Demo mode - set to true to use mock data
const DEMO_MODE = false

export interface ContainerInfo {
  id: string
  name: string
  image: string
  status: string
  state: string
  created: number
  ports: PortMapping[]
}

export interface PortMapping {
  private_port: number
  public_port: number | null
  port_type: string
}

export interface ContainerStats {
  cpu_percent: number
  memory_usage: number
  memory_limit: number
  memory_percent: number
  network_rx: number
  network_tx: number
}

export interface DockerInfo {
  containers_running: number
  containers_paused: number
  containers_stopped: number
  images: number
  docker_version: string
  os_type: string
  architecture: string
  memory_total: number
  cpus: number
}

// Mock data for demo mode
const MOCK_CONTAINERS: ContainerInfo[] = [
  { id: 'a1b2c3d4e5f6', name: 'nginx', image: 'nginx:alpine', status: 'Up 2 hours', state: 'running', created: Math.floor(Date.now()/1000) - 7200, ports: [{ private_port: 80, public_port: 80, port_type: 'tcp' }, { private_port: 443, public_port: 443, port_type: 'tcp' }] },
  { id: 'b2c3d4e5f6a1', name: 'php-8.5-fpm', image: 'php:8.5-fpm-alpine', status: 'Up 2 hours', state: 'running', created: Math.floor(Date.now()/1000) - 7200, ports: [{ private_port: 9000, public_port: 9000, port_type: 'tcp' }] },
  { id: 'c3d4e5f6a1b2', name: 'php-8.5-cli', image: 'php:8.5-cli-alpine', status: 'Up 45 minutes', state: 'running', created: Math.floor(Date.now()/1000) - 2700, ports: [] },
  { id: 'd4e5f6a1b2c3', name: 'php-8.5-swoole', image: 'php:8.5-swoole', status: 'Up 2 hours', state: 'running', created: Math.floor(Date.now()/1000) - 7200, ports: [{ private_port: 9501, public_port: 9501, port_type: 'tcp' }] },
  { id: 'e5f6a1b2c3d4', name: 'mysql', image: 'mysql:8.0', status: 'Up 2 hours', state: 'running', created: Math.floor(Date.now()/1000) - 7200, ports: [{ private_port: 3306, public_port: 3306, port_type: 'tcp' }] },
  { id: 'f6a1b2c3d4e5', name: 'postgres', image: 'postgres:17-alpine', status: 'Exited (0) 1 hour ago', state: 'exited', created: Math.floor(Date.now()/1000) - 10800, ports: [{ private_port: 5432, public_port: 5432, port_type: 'tcp' }] },
  { id: 'a1b2c3d4e5f7', name: 'redis', image: 'redis:7-alpine', status: 'Up 2 hours', state: 'running', created: Math.floor(Date.now()/1000) - 7200, ports: [{ private_port: 6379, public_port: 6379, port_type: 'tcp' }] },
  { id: 'b2c3d4e5f6a2', name: 'dnsmasq', image: 'dnsmasq:latest', status: 'Exited (0) 3 hours ago', state: 'exited', created: Math.floor(Date.now()/1000) - 14400, ports: [{ private_port: 53, public_port: 53, port_type: 'udp' }] },
]

const MOCK_STATS: Map<string, ContainerStats> = new Map([
  ['a1b2c3d4e5f6', { cpu_percent: 0.8, memory_usage: 52428800, memory_limit: 1073741824, memory_percent: 4.9, network_rx: 15728640, network_tx: 8388608 }],
  ['b2c3d4e5f6a1', { cpu_percent: 12.4, memory_usage: 268435456, memory_limit: 1073741824, memory_percent: 25.0, network_rx: 104857600, network_tx: 52428800 }],
  ['c3d4e5f6a1b2', { cpu_percent: 45.2, memory_usage: 134217728, memory_limit: 1073741824, memory_percent: 12.5, network_rx: 5242880, network_tx: 2621440 }],
  ['d4e5f6a1b2c3', { cpu_percent: 8.7, memory_usage: 209715200, memory_limit: 1073741824, memory_percent: 19.5, network_rx: 78643200, network_tx: 41943040 }],
  ['e5f6a1b2c3d4', { cpu_percent: 3.2, memory_usage: 524288000, memory_limit: 2147483648, memory_percent: 24.4, network_rx: 209715200, network_tx: 104857600 }],
  ['a1b2c3d4e5f7', { cpu_percent: 1.1, memory_usage: 31457280, memory_limit: 536870912, memory_percent: 5.9, network_rx: 52428800, network_tx: 26214400 }],
])

const MOCK_DOCKER_INFO: DockerInfo = {
  containers_running: 6,
  containers_paused: 0,
  containers_stopped: 2,
  images: 24,
  docker_version: '27.4.0',
  os_type: 'linux',
  architecture: 'x86_64',
  memory_total: 34359738368,
  cpus: 16
}

export const useDockerStore = defineStore('docker', () => {
  const containers = ref<ContainerInfo[]>([])
  const dockerInfo = ref<DockerInfo | null>(null)
  const containerStats = ref<Map<string, ContainerStats>>(new Map())
  const isConnected = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const runningContainers = computed(() =>
    containers.value.filter((c) => c.state === 'running')
  )

  const stoppedContainers = computed(() =>
    containers.value.filter((c) => c.state !== 'running')
  )

  const totalCpu = computed(() => {
    let total = 0
    containerStats.value.forEach((stats) => {
      total += stats.cpu_percent
    })
    return total
  })

  const totalMemory = computed(() => {
    let total = 0
    containerStats.value.forEach((stats) => {
      total += stats.memory_usage
    })
    return total
  })

  async function checkConnection() {
    if (DEMO_MODE) {
      isConnected.value = true
      return true
    }
    try {
      isConnected.value = await invoke<boolean>('check_docker_connection')
    } catch (e) {
      isConnected.value = false
      error.value = String(e)
    }
    return isConnected.value
  }

  async function connect() {
    if (DEMO_MODE) {
      isConnected.value = true
      return
    }
    try {
      loading.value = true
      error.value = null
      isConnected.value = await invoke<boolean>('connect_docker')
    } catch (e) {
      isConnected.value = false
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadContainers() {
    if (DEMO_MODE) {
      containers.value = MOCK_CONTAINERS
      return
    }
    if (!isConnected.value) return

    try {
      loading.value = true
      error.value = null
      containers.value = await invoke<ContainerInfo[]>('list_containers')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadDockerInfo() {
    if (DEMO_MODE) {
      dockerInfo.value = MOCK_DOCKER_INFO
      return
    }
    if (!isConnected.value) return

    try {
      dockerInfo.value = await invoke<DockerInfo>('get_docker_info')
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadContainerStats(id: string) {
    if (DEMO_MODE) {
      const mockStats = MOCK_STATS.get(id)
      if (mockStats) {
        containerStats.value.set(id, mockStats)
      }
      return
    }
    if (!isConnected.value) return

    try {
      const stats = await invoke<ContainerStats>('get_container_stats', { id })
      containerStats.value.set(id, stats)
    } catch (e) {
      // Stats may fail for stopped containers, ignore silently
    }
  }

  async function loadAllStats() {
    const running = containers.value.filter((c) => c.state === 'running')
    await Promise.all(running.map((c) => loadContainerStats(c.id)))
  }

  async function startContainer(id: string) {
    try {
      loading.value = true
      await invoke('start_container', { id })
      await loadContainers()
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function stopContainer(id: string) {
    try {
      loading.value = true
      await invoke('stop_container', { id })
      await loadContainers()
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function restartContainer(id: string) {
    try {
      loading.value = true
      await invoke('restart_container', { id })
      await loadContainers()
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function refresh() {
    await loadContainers()
    await loadAllStats()
    await loadDockerInfo()
  }

  async function getContainerLogs(id: string, tail: number = 100): Promise<string[]> {
    if (DEMO_MODE) {
      return [
        '2024-01-15T10:30:00.000Z [INFO] Container started',
        '2024-01-15T10:30:01.000Z [INFO] Listening on port 80',
        '2024-01-15T10:30:02.000Z [INFO] Ready to accept connections',
      ]
    }
    try {
      return await invoke<string[]>('get_container_logs', { id, tail })
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    containers,
    dockerInfo,
    containerStats,
    isConnected,
    loading,
    error,
    runningContainers,
    stoppedContainers,
    totalCpu,
    totalMemory,
    checkConnection,
    connect,
    loadContainers,
    loadDockerInfo,
    loadContainerStats,
    loadAllStats,
    startContainer,
    stopContainer,
    restartContainer,
    refresh,
    getContainerLogs
  }
})
