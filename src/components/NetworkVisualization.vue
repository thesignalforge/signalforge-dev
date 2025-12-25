<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import * as THREE from 'three'

interface NetworkContainer {
  id: string
  name: string
  type: string
  networks: string[]
  ip: string
  ports: string
  health: string
  cpu: number
  mem: number
}

interface NetworkConnection {
  from: string
  to: string
  protocol: string
  network: string
}

interface NetworkTopology {
  containers: NetworkContainer[]
  connections: NetworkConnection[]
}

const networkData = ref<NetworkTopology | null>(null)
const selectedContainer = ref<NetworkContainer | null>(null)
const isDetailOpen = ref(false)
const loading = ref(true)
const error = ref<string | null>(null)

let scene: THREE.Scene
let camera: THREE.PerspectiveCamera
let renderer: THREE.WebGLRenderer
let nodes: Record<string, { mesh: THREE.Mesh; glow: THREE.Mesh; ring: THREE.Mesh | null; position: { x: number; y: number; z: number } }> = {}
let animationFrameId: number

const healthColors: Record<string, number> = {
  healthy: 0x00ff88,
  starting: 0x00d9ff,
  unhealthy: 0xff3344,
  stopped: 0x556677
}

const networkColors: Record<string, number> = {
  devstack: 0x00d9ff,
  bridge: 0xa855f7,
  custom: 0x10b981,
  host: 0xf97316
}

const CAMERA_DEFAULT_Z = 20
const CAMERA_ZOOMED_Z = 35

async function loadNetworkTopology() {
  try {
    loading.value = true
    error.value = null
    const data = await invoke<NetworkTopology>('get_network_topology')
    networkData.value = data
  } catch (e) {
    error.value = String(e)
    console.error('Failed to load network topology:', e)
  } finally {
    loading.value = false
  }
}

function initThreeJS() {
  const container = document.getElementById('three-container')
  if (!container) return

  scene = new THREE.Scene()
  scene.background = new THREE.Color(0x050810)
  scene.fog = new THREE.Fog(0x050810, 15, 60)

  camera = new THREE.PerspectiveCamera(
    60,
    container.clientWidth / container.clientHeight,
    0.1,
    1000
  )
  camera.position.set(0, 0, CAMERA_DEFAULT_Z)
  camera.lookAt(0, 0, 0)

  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
  renderer.setSize(container.clientWidth, container.clientHeight)
  renderer.setPixelRatio(window.devicePixelRatio)
  container.appendChild(renderer.domElement)

  const ambientLight = new THREE.AmbientLight(0xffffff, 0.3)
  scene.add(ambientLight)

  const pointLight1 = new THREE.PointLight(0x00d9ff, 1.5, 100)
  pointLight1.position.set(15, 15, 15)
  scene.add(pointLight1)

  const pointLight2 = new THREE.PointLight(0x00d9ff, 1, 100)
  pointLight2.position.set(-15, -15, -15)
  scene.add(pointLight2)

  const pointLight3 = new THREE.PointLight(0x00ff88, 0.8, 100)
  pointLight3.position.set(0, 20, 0)
  scene.add(pointLight3)

  const sphereGeometry = new THREE.SphereGeometry(8, 32, 32)
  const wireframeMaterial = new THREE.MeshBasicMaterial({
    color: 0x00d9ff,
    wireframe: true,
    transparent: true,
    opacity: 0.04
  })
  const wireframeSphere = new THREE.Mesh(sphereGeometry, wireframeMaterial)
  scene.add(wireframeSphere)

  createNetwork()
  setupInteraction()
  animate()

  window.addEventListener('resize', handleResize)
}

function generateFullerenePositions(count: number) {
  const positions: { x: number; y: number; z: number }[] = []
  const radius = 8
  const phi = (1 + Math.sqrt(5)) / 2

  const icoVertices = [
    [-1, phi, 0], [1, phi, 0], [-1, -phi, 0], [1, -phi, 0],
    [0, -1, phi], [0, 1, phi], [0, -1, -phi], [0, 1, -phi],
    [phi, 0, -1], [phi, 0, 1], [-phi, 0, -1], [-phi, 0, 1]
  ]

  for (const vertex of icoVertices) {
    const length = Math.sqrt(vertex[0] ** 2 + vertex[1] ** 2 + vertex[2] ** 2)
    positions.push({
      x: (vertex[0] / length) * radius,
      y: (vertex[1] / length) * radius,
      z: (vertex[2] / length) * radius
    })
  }

  const remaining = count - positions.length
  const goldenAngle = Math.PI * (3 - Math.sqrt(5))

  for (let i = 0; i < remaining; i++) {
    const y = 1 - (i / (remaining - 1 || 1)) * 2
    const radiusAtY = Math.sqrt(1 - y * y)
    const theta = goldenAngle * i

    const x = Math.cos(theta) * radiusAtY
    const z = Math.sin(theta) * radiusAtY

    positions.push({
      x: x * radius,
      y: y * radius,
      z: z * radius
    })
  }

  return positions
}

function createNetwork() {
  if (!networkData.value || networkData.value.containers.length === 0) return

  const positions = generateFullerenePositions(networkData.value.containers.length)

  networkData.value.containers.forEach((container, index) => {
    const pos = positions[index] || { x: 0, y: 0, z: 0 }
    const healthColor = healthColors[container.health] || 0x556677

    const geometry = new THREE.SphereGeometry(0.4, 32, 32)
    const material = new THREE.MeshPhongMaterial({
      color: healthColor,
      emissive: healthColor,
      emissiveIntensity: container.health === 'healthy' ? 0.6 : 0.4,
      shininess: 100,
      specular: 0xffffff
    })

    const mesh = new THREE.Mesh(geometry, material)
    mesh.position.set(pos.x, pos.y, pos.z)
    mesh.userData = { id: container.id }
    scene.add(mesh)

    const glowGeometry = new THREE.SphereGeometry(0.5, 32, 32)
    const glowMaterial = new THREE.MeshBasicMaterial({
      color: healthColor,
      transparent: true,
      opacity: 0.25
    })
    const glow = new THREE.Mesh(glowGeometry, glowMaterial)
    glow.position.copy(mesh.position)
    scene.add(glow)

    let ring: THREE.Mesh | null = null
    if (container.health === 'healthy') {
      const ringGeometry = new THREE.RingGeometry(0.6, 0.65, 32)
      const ringMaterial = new THREE.MeshBasicMaterial({
        color: healthColor,
        side: THREE.DoubleSide,
        transparent: true,
        opacity: 0.7
      })
      ring = new THREE.Mesh(ringGeometry, ringMaterial)
      ring.position.copy(mesh.position)
      ring.lookAt(camera.position)
      scene.add(ring)
    }

    nodes[container.id] = { mesh, glow, ring, position: pos }
  })

  networkData.value.connections.forEach(conn => {
    const fromNode = nodes[conn.from]
    const toNode = nodes[conn.to]

    if (!fromNode || !toNode) return

    const fromPos = fromNode.position
    const toPos = toNode.position

    const curve = new THREE.QuadraticBezierCurve3(
      new THREE.Vector3(fromPos.x, fromPos.y, fromPos.z),
      new THREE.Vector3(
        (fromPos.x + toPos.x) / 2,
        (fromPos.y + toPos.y) / 2 + 1,
        (fromPos.z + toPos.z) / 2
      ),
      new THREE.Vector3(toPos.x, toPos.y, toPos.z)
    )

    const points = curve.getPoints(50)
    const geometry = new THREE.BufferGeometry().setFromPoints(points)

    const networkColor = networkColors[conn.network] || 0x00d9ff
    const material = new THREE.LineBasicMaterial({
      color: networkColor,
      transparent: true,
      opacity: 0.5
    })

    const line = new THREE.Line(geometry, material)
    scene.add(line)
  })
}

function setupInteraction() {
  const raycaster = new THREE.Raycaster()
  const mouse = new THREE.Vector2()

  renderer.domElement.addEventListener('click', (event: MouseEvent) => {
    if (isDetailOpen.value) return

    const rect = renderer.domElement.getBoundingClientRect()
    mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1
    mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1

    raycaster.setFromCamera(mouse, camera)
    const meshes = Object.values(nodes).map((n) => n.mesh)
    const intersects = raycaster.intersectObjects(meshes)

    if (intersects.length > 0) {
      const clickedNode = intersects[0].object
      const container = networkData.value?.containers.find(
        c => c.id === clickedNode.userData.id
      )
      if (container) {
        showDetail(container)
      }
    }
  })

  let isDragging = false
  let previousMouse = { x: 0, y: 0 }

  renderer.domElement.addEventListener('mousedown', () => {
    if (!isDetailOpen.value) isDragging = true
  })
  renderer.domElement.addEventListener('mouseup', () => isDragging = false)

  renderer.domElement.addEventListener('mousemove', (event: MouseEvent) => {
    if (isDragging) {
      const deltaX = event.clientX - previousMouse.x
      const deltaY = event.clientY - previousMouse.y

      scene.rotation.y += deltaX * 0.005
      scene.rotation.x += deltaY * 0.005
    }

    previousMouse = { x: event.clientX, y: event.clientY }
  })

  renderer.domElement.addEventListener('wheel', (event: WheelEvent) => {
    if (isDetailOpen.value) return
    event.preventDefault()
    const delta = event.deltaY * 0.01
    camera.position.z += delta
    camera.position.z = Math.max(10, Math.min(35, camera.position.z))
  })
}

function showDetail(container: NetworkContainer) {
  selectedContainer.value = container
  isDetailOpen.value = true
  animateCamera(CAMERA_ZOOMED_Z, 600)
}

function hideDetail() {
  isDetailOpen.value = false
  selectedContainer.value = null
  animateCamera(CAMERA_DEFAULT_Z, 600)
}

function animateCamera(targetZ: number, duration: number) {
  const startZ = camera.position.z
  const startTime = Date.now()

  function update() {
    const elapsed = Date.now() - startTime
    const progress = Math.min(elapsed / duration, 1)

    const eased = progress < 0.5
      ? 4 * progress * progress * progress
      : 1 - Math.pow(-2 * progress + 2, 3) / 2

    camera.position.z = startZ + (targetZ - startZ) * eased

    if (progress < 1) {
      requestAnimationFrame(update)
    }
  }

  update()
}

function animate() {
  animationFrameId = requestAnimationFrame(animate)

  scene.rotation.y += 0.001

  const time = Date.now() * 0.001
  Object.entries(nodes).forEach(([id, node]) => {
    const container = networkData.value?.containers.find(c => c.id === id)

    if (container?.health === 'healthy' && node.ring) {
      node.ring.lookAt(camera.position)
      const scale = 1 + Math.sin(time * 2) * 0.1
      node.ring.scale.set(scale, scale, 1)
    }

    const glowScale = 1 + Math.sin(time + node.position.x) * 0.05
    node.glow.scale.set(glowScale, glowScale, glowScale)
  })

  renderer.render(scene, camera)
}

function handleResize() {
  const container = document.getElementById('three-container')
  if (!container) return

  camera.aspect = container.clientWidth / container.clientHeight
  camera.updateProjectionMatrix()
  renderer.setSize(container.clientWidth, container.clientHeight)
}

onMounted(async () => {
  await loadNetworkTopology()
  if (networkData.value && networkData.value.containers.length > 0) {
    initThreeJS()
  }

  const interval = setInterval(loadNetworkTopology, 5000)
  onUnmounted(() => clearInterval(interval))
})

onUnmounted(() => {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId)
  }
  window.removeEventListener('resize', handleResize)
})
</script>

<template>
  <div class="network-visualization">
    <div v-if="loading && !networkData" class="loading-state">
      <div class="loading-spinner"></div>
      <p>Loading network topology...</p>
    </div>

    <div v-else-if="error" class="error-state">
      <p>{{ error }}</p>
    </div>

    <div v-else-if="networkData && networkData.containers.length === 0" class="empty-state">
      <p>No containers found</p>
      <p class="text-dim">Start some Docker containers to see them here</p>
    </div>

    <div v-else id="three-container"></div>

    <!-- Detail Panel -->
    <div v-if="isDetailOpen && selectedContainer" class="detail-backdrop" @click="hideDetail">
      <div class="detail-panel" @click.stop>
        <div class="panel-header">CONTAINER ANALYSIS</div>
        <div class="panel-body">
          <div class="detail-title-bar">
            <div>
              <div class="detail-title">{{ selectedContainer.name }}</div>
              <div class="detail-subtitle">
                {{ selectedContainer.type.toUpperCase() }} |
                {{ selectedContainer.networks[0]?.toUpperCase() || 'NO NETWORK' }}
              </div>
            </div>
            <div
              class="detail-status-badge"
              :class="selectedContainer.health"
            >
              {{ selectedContainer.health.toUpperCase() }}
            </div>
            <button class="detail-close" @click="hideDetail">x</button>
          </div>

          <div class="detail-content">
            <div class="detail-section">
              <div class="detail-section-title">RESOURCE USAGE</div>
              <div class="stats-grid">
                <div class="stat-item">
                  <div class="stat-label">CPU USAGE</div>
                  <div class="stat-value">{{ selectedContainer.cpu.toFixed(1) }}%</div>
                </div>
                <div class="stat-item">
                  <div class="stat-label">MEMORY</div>
                  <div class="stat-value">{{ selectedContainer.mem }} MB</div>
                </div>
              </div>
            </div>

            <div class="detail-section">
              <div class="detail-section-title">CONFIGURATION</div>
              <div class="info-grid">
                <div class="info-row">
                  <span class="info-label">CONTAINER ID</span>
                  <span class="info-value">{{ selectedContainer.id.substring(0, 12) }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">IP ADDRESS</span>
                  <span class="info-value">{{ selectedContainer.ip }}</span>
                </div>
                <div class="info-row">
                  <span class="info-label">PORTS</span>
                  <span class="info-value">{{ selectedContainer.ports }}</span>
                </div>
              </div>
            </div>

            <div class="detail-section full-width">
              <div class="detail-section-title">NETWORKS</div>
              <div class="networks-list">
                <div
                  v-for="network in selectedContainer.networks"
                  :key="network"
                  class="network-badge"
                  :class="network"
                >
                  <div class="network-indicator"></div>
                  {{ network.toUpperCase() }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style src="@/assets/network-visualization.css"></style>

<style scoped>
.network-visualization {
  width: 100%;
  height: 100%;
  position: relative;
}

#three-container {
  width: 100%;
  height: 100%;
}

.loading-state,
.error-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-dim);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border);
  border-top-color: var(--cyan);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.text-dim {
  color: var(--text-dim);
  font-size: 12px;
  margin-top: 8px;
}

.detail-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(5, 8, 16, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.detail-panel {
  background: linear-gradient(135deg, rgba(18, 24, 32, 0.98), rgba(18, 24, 32, 0.95));
  border: 1px solid var(--cyan);
  border-radius: 8px;
  width: 90%;
  max-width: 800px;
  max-height: 80vh;
  overflow: auto;
  box-shadow: 0 0 40px rgba(0, 217, 255, 0.2);
}
</style>
