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

interface Impulse {
  mesh: THREE.Mesh
  progress: number
  speed: number
  startPos: THREE.Vector3
  endPos: THREE.Vector3
  reverse: boolean
}

const networkData = ref<NetworkTopology | null>(null)
const selectedContainer = ref<NetworkContainer | null>(null)
const isDetailOpen = ref(false)
const loading = ref(true)
const error = ref<string | null>(null)

let scene: THREE.Scene
let camera: THREE.PerspectiveCamera
let renderer: THREE.WebGLRenderer
let animationFrameId: number
let labelContainer: HTMLDivElement

// Sphere geometry data
const SPHERE_RADIUS = 8
const sphereEdges: { start: THREE.Vector3; end: THREE.Vector3; line: THREE.Line }[] = []
const sphereImpulses: Impulse[] = []

// Container nodes
const containerNodes: Map<string, {
  position: THREE.Vector3
  mesh: THREE.Mesh
  glow: THREE.Mesh
  labelDiv: HTMLDivElement
}> = new Map()

// Connection impulses (between containers)
const connectionImpulses: Impulse[] = []

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

function createGeodesicSphere(): THREE.Vector3[] {
  // Create icosahedron vertices for geodesic sphere
  const phi = (1 + Math.sqrt(5)) / 2
  const vertices: THREE.Vector3[] = []

  // Icosahedron base vertices
  const icoVertices = [
    [-1, phi, 0], [1, phi, 0], [-1, -phi, 0], [1, -phi, 0],
    [0, -1, phi], [0, 1, phi], [0, -1, -phi], [0, 1, -phi],
    [phi, 0, -1], [phi, 0, 1], [-phi, 0, -1], [-phi, 0, 1]
  ]

  // Normalize to sphere radius
  for (const v of icoVertices) {
    const vec = new THREE.Vector3(v[0], v[1], v[2]).normalize().multiplyScalar(SPHERE_RADIUS)
    vertices.push(vec)
  }

  // Icosahedron faces (triangles)
  const faces = [
    [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
    [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
    [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
    [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1]
  ]

  // Subdivide each face once for more vertices
  const subdividedVertices: THREE.Vector3[] = [...vertices]
  const edgeMap = new Map<string, number>()

  function getMidpoint(i1: number, i2: number): number {
    const key = i1 < i2 ? `${i1}-${i2}` : `${i2}-${i1}`
    if (edgeMap.has(key)) return edgeMap.get(key)!

    const v1 = subdividedVertices[i1]
    const v2 = subdividedVertices[i2]
    const mid = new THREE.Vector3().addVectors(v1, v2).normalize().multiplyScalar(SPHERE_RADIUS)
    const idx = subdividedVertices.length
    subdividedVertices.push(mid)
    edgeMap.set(key, idx)
    return idx
  }

  const newFaces: number[][] = []
  for (const [a, b, c] of faces) {
    const ab = getMidpoint(a, b)
    const bc = getMidpoint(b, c)
    const ca = getMidpoint(c, a)
    newFaces.push([a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca])
  }

  return subdividedVertices
}

function initThreeJS() {
  const container = document.getElementById('three-container')
  if (!container) return

  // Create label container
  labelContainer = document.createElement('div')
  labelContainer.style.position = 'absolute'
  labelContainer.style.top = '0'
  labelContainer.style.left = '0'
  labelContainer.style.width = '100%'
  labelContainer.style.height = '100%'
  labelContainer.style.pointerEvents = 'none'
  labelContainer.style.overflow = 'hidden'
  container.appendChild(labelContainer)

  scene = new THREE.Scene()
  scene.background = new THREE.Color(0x050810)

  camera = new THREE.PerspectiveCamera(
    60,
    container.clientWidth / container.clientHeight,
    0.1,
    1000
  )
  camera.position.set(0, 0, 20)
  camera.lookAt(0, 0, 0)

  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
  renderer.setSize(container.clientWidth, container.clientHeight)
  renderer.setPixelRatio(window.devicePixelRatio)
  container.appendChild(renderer.domElement)

  // Ambient light
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.2)
  scene.add(ambientLight)

  // Create geodesic sphere wireframe with impulses
  createSphereWireframe()

  // Place containers at vertices
  placeContainerNodes()

  // Create connection lines between containers
  createConnectionLines()

  // Setup interaction
  setupInteraction()

  // Start animation
  animate()

  window.addEventListener('resize', handleResize)
}

function createSphereWireframe() {
  const vertices = createGeodesicSphere()

  // Create edges from the geodesic structure
  const phi = (1 + Math.sqrt(5)) / 2
  const edgeLength = SPHERE_RADIUS * 2 / phi * 1.2 // Approximate edge length

  const addedEdges = new Set<string>()

  for (let i = 0; i < vertices.length; i++) {
    for (let j = i + 1; j < vertices.length; j++) {
      const dist = vertices[i].distanceTo(vertices[j])
      if (dist < edgeLength) {
        const key = `${i}-${j}`
        if (!addedEdges.has(key)) {
          addedEdges.add(key)

          // Create line
          const geometry = new THREE.BufferGeometry().setFromPoints([vertices[i], vertices[j]])
          const material = new THREE.LineBasicMaterial({
            color: 0x1a3a4a,
            transparent: true,
            opacity: 0.4
          })
          const line = new THREE.Line(geometry, material)
          scene.add(line)

          sphereEdges.push({
            start: vertices[i].clone(),
            end: vertices[j].clone(),
            line
          })
        }
      }
    }
  }

  // Create impulses traveling along sphere edges
  for (let i = 0; i < 30; i++) {
    const edge = sphereEdges[Math.floor(Math.random() * sphereEdges.length)]
    createSphereImpulse(edge.start, edge.end)
  }
}

function createSphereImpulse(start: THREE.Vector3, end: THREE.Vector3) {
  const geometry = new THREE.SphereGeometry(0.08, 8, 8)
  const material = new THREE.MeshBasicMaterial({
    color: 0x00d9ff,
    transparent: true,
    opacity: 0.6
  })
  const mesh = new THREE.Mesh(geometry, material)
  mesh.position.copy(start)
  scene.add(mesh)

  sphereImpulses.push({
    mesh,
    progress: Math.random(),
    speed: 0.002 + Math.random() * 0.003,
    startPos: start.clone(),
    endPos: end.clone(),
    reverse: Math.random() > 0.5
  })
}

function placeContainerNodes() {
  if (!networkData.value) return

  const vertices = createGeodesicSphere()
  const containers = networkData.value.containers

  // Assign containers to vertices
  containers.forEach((container, index) => {
    const vertex = vertices[index % vertices.length]

    // Main node (white dot) - smaller size
    const geometry = new THREE.SphereGeometry(0.15, 32, 32)
    const material = new THREE.MeshBasicMaterial({
      color: 0xffffff,
      transparent: false
    })
    const mesh = new THREE.Mesh(geometry, material)
    mesh.position.copy(vertex)
    mesh.userData = { id: container.id, name: container.name }
    scene.add(mesh)

    // Glow effect - smaller to match node
    const glowGeometry = new THREE.SphereGeometry(0.25, 32, 32)
    const glowMaterial = new THREE.MeshBasicMaterial({
      color: 0x00d9ff,
      transparent: true,
      opacity: 0.3
    })
    const glow = new THREE.Mesh(glowGeometry, glowMaterial)
    glow.position.copy(vertex)
    scene.add(glow)

    // Create HTML label
    const labelDiv = document.createElement('div')
    labelDiv.className = 'node-label'
    labelDiv.textContent = container.name.replace('signalforge-', '')
    labelDiv.style.cssText = `
      position: absolute;
      color: #00d9ff;
      font-family: 'Monaco', 'Consolas', monospace;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 1px;
      text-shadow: 0 0 10px rgba(0, 217, 255, 0.8), 0 0 20px rgba(0, 217, 255, 0.4);
      white-space: nowrap;
      pointer-events: none;
    `
    labelContainer.appendChild(labelDiv)

    containerNodes.set(container.id, { position: vertex, mesh, glow, labelDiv })
  })
}

function createConnectionLines() {
  if (!networkData.value) return

  const connections = networkData.value.connections

  connections.forEach(conn => {
    const fromNode = containerNodes.get(conn.from)
    const toNode = containerNodes.get(conn.to)

    if (!fromNode || !toNode) return

    // Create thick line between connected containers
    const curve = new THREE.QuadraticBezierCurve3(
      fromNode.position,
      new THREE.Vector3(
        (fromNode.position.x + toNode.position.x) / 2,
        (fromNode.position.y + toNode.position.y) / 2 + 1,
        (fromNode.position.z + toNode.position.z) / 2
      ),
      toNode.position
    )

    const points = curve.getPoints(50)
    const geometry = new THREE.BufferGeometry().setFromPoints(points)
    const material = new THREE.LineBasicMaterial({
      color: 0x00d9ff,
      transparent: true,
      opacity: 0.7,
      linewidth: 2
    })
    const line = new THREE.Line(geometry, material)
    scene.add(line)

    // Create impulses on connection lines
    for (let i = 0; i < 3; i++) {
      createConnectionImpulse(fromNode.position, toNode.position)
    }
  })
}

function createConnectionImpulse(start: THREE.Vector3, end: THREE.Vector3) {
  const geometry = new THREE.SphereGeometry(0.12, 8, 8)
  const material = new THREE.MeshBasicMaterial({
    color: 0x00ffaa,
    transparent: true,
    opacity: 0.9
  })
  const mesh = new THREE.Mesh(geometry, material)
  scene.add(mesh)

  connectionImpulses.push({
    mesh,
    progress: Math.random(),
    speed: 0.005 + Math.random() * 0.005,
    startPos: start.clone(),
    endPos: end.clone(),
    reverse: Math.random() > 0.5
  })
}

function updateLabels() {
  containerNodes.forEach((node) => {
    // Get the actual world position of the mesh after scene rotation
    const worldPos = new THREE.Vector3()
    node.mesh.getWorldPosition(worldPos)

    const vector = worldPos.clone()
    vector.project(camera)

    const x = (vector.x * 0.5 + 0.5) * renderer.domElement.clientWidth
    const y = (-vector.y * 0.5 + 0.5) * renderer.domElement.clientHeight

    // Position label above the node
    node.labelDiv.style.transform = `translate(-50%, -100%) translate(${x}px, ${y - 15}px)`

    // Hide if behind camera
    node.labelDiv.style.opacity = vector.z < 1 ? '1' : '0'
  })
}

function updateImpulses() {
  // Update sphere edge impulses
  sphereImpulses.forEach(impulse => {
    impulse.progress += impulse.speed * (impulse.reverse ? -1 : 1)

    if (impulse.progress > 1 || impulse.progress < 0) {
      // Pick a new random edge
      const edge = sphereEdges[Math.floor(Math.random() * sphereEdges.length)]
      impulse.startPos = edge.start.clone()
      impulse.endPos = edge.end.clone()
      impulse.progress = impulse.reverse ? 1 : 0
      impulse.reverse = Math.random() > 0.5
    }

    impulse.mesh.position.lerpVectors(impulse.startPos, impulse.endPos, impulse.progress)
  })

  // Update connection impulses
  connectionImpulses.forEach(impulse => {
    impulse.progress += impulse.speed * (impulse.reverse ? -1 : 1)

    if (impulse.progress > 1) {
      impulse.progress = 0
      impulse.reverse = false
    } else if (impulse.progress < 0) {
      impulse.progress = 1
      impulse.reverse = true
    }

    // Bezier curve interpolation
    const mid = new THREE.Vector3(
      (impulse.startPos.x + impulse.endPos.x) / 2,
      (impulse.startPos.y + impulse.endPos.y) / 2 + 1,
      (impulse.startPos.z + impulse.endPos.z) / 2
    )

    const t = impulse.progress
    const oneMinusT = 1 - t
    impulse.mesh.position.set(
      oneMinusT * oneMinusT * impulse.startPos.x + 2 * oneMinusT * t * mid.x + t * t * impulse.endPos.x,
      oneMinusT * oneMinusT * impulse.startPos.y + 2 * oneMinusT * t * mid.y + t * t * impulse.endPos.y,
      oneMinusT * oneMinusT * impulse.startPos.z + 2 * oneMinusT * t * mid.z + t * t * impulse.endPos.z
    )
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
    const meshes = Array.from(containerNodes.values()).map(n => n.mesh)
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
    camera.position.z = Math.max(12, Math.min(40, camera.position.z))
  })
}

function showDetail(container: NetworkContainer) {
  selectedContainer.value = container
  isDetailOpen.value = true
}

function hideDetail() {
  isDetailOpen.value = false
  selectedContainer.value = null
}

function animate() {
  animationFrameId = requestAnimationFrame(animate)

  // Slow rotation
  scene.rotation.y += 0.001

  // Pulse glow on container nodes
  const time = Date.now() * 0.001
  containerNodes.forEach((node) => {
    const scale = 1 + Math.sin(time * 2) * 0.1
    node.glow.scale.set(scale, scale, scale)
  })

  // Update impulses
  updateImpulses()

  // Update labels
  updateLabels()

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
  initThreeJS()

  const interval = setInterval(async () => {
    const oldData = JSON.stringify(networkData.value)
    await loadNetworkTopology()
    // Only rebuild if data changed
    if (JSON.stringify(networkData.value) !== oldData) {
      // Clear and rebuild
      containerNodes.forEach(node => {
        scene.remove(node.mesh)
        scene.remove(node.glow)
        node.labelDiv.remove()
      })
      containerNodes.clear()
      connectionImpulses.forEach(imp => scene.remove(imp.mesh))
      connectionImpulses.length = 0

      placeContainerNodes()
      createConnectionLines()
    }
  }, 5000)
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
    <!-- Three.js container -->
    <div id="three-container"></div>

    <!-- Overlay states -->
    <div v-if="loading && !networkData" class="overlay-state">
      <div class="loading-spinner"></div>
      <p>Loading network topology...</p>
    </div>

    <div v-else-if="error" class="overlay-state error">
      <p>{{ error }}</p>
      <p class="text-dim">Make sure Docker is running</p>
    </div>

    <div v-else-if="networkData && networkData.containers.length === 0" class="overlay-state">
      <p>No containers running</p>
      <p class="text-dim">Start some Docker containers to visualize them</p>
    </div>

    <!-- Detail Panel -->
    <div v-if="isDetailOpen && selectedContainer" class="detail-backdrop" @click="hideDetail">
      <div class="detail-panel" @click.stop>
        <div class="panel-header">CONTAINER DETAILS</div>
        <div class="panel-body">
          <div class="detail-title-bar">
            <div>
              <div class="detail-title">{{ selectedContainer.name }}</div>
              <div class="detail-subtitle">
                {{ selectedContainer.type.toUpperCase() }} |
                {{ selectedContainer.networks[0]?.toUpperCase() || 'NO NETWORK' }}
              </div>
            </div>
            <div class="detail-status-badge" :class="selectedContainer.health">
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
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.network-visualization {
  width: 100%;
  height: 100%;
  position: relative;
  background: #050810;
}

#three-container {
  width: 100%;
  height: 100%;
  position: absolute;
  top: 0;
  left: 0;
}

.overlay-state {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #7a8a9e;
  text-align: center;
  z-index: 10;
  background: rgba(5, 8, 16, 0.8);
  padding: 2rem;
  border-radius: 8px;
  border: 1px solid #1f2937;
}

.overlay-state.error {
  border-color: #ff3344;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #1f2937;
  border-top-color: #00d9ff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.text-dim {
  color: #7a8a9e;
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
  border: 1px solid #00d9ff;
  border-radius: 8px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  overflow: auto;
  box-shadow: 0 0 40px rgba(0, 217, 255, 0.2);
}

.panel-header {
  background: linear-gradient(90deg, #00d9ff, transparent);
  padding: 12px 20px;
  font-size: 11px;
  font-weight: bold;
  letter-spacing: 2px;
  color: #0a0e15;
}

.panel-body {
  padding: 20px;
}

.detail-title-bar {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
}

.detail-title {
  font-size: 20px;
  font-weight: bold;
  color: #e0e7f1;
}

.detail-subtitle {
  font-size: 10px;
  color: #7a8a9e;
  letter-spacing: 1px;
  margin-top: 4px;
}

.detail-status-badge {
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: bold;
  letter-spacing: 1px;
}

.detail-status-badge.healthy {
  background: rgba(0, 255, 136, 0.2);
  color: #00ff88;
  border: 1px solid #00ff88;
}

.detail-status-badge.unhealthy {
  background: rgba(255, 51, 68, 0.2);
  color: #ff3344;
  border: 1px solid #ff3344;
}

.detail-status-badge.starting {
  background: rgba(0, 217, 255, 0.2);
  color: #00d9ff;
  border: 1px solid #00d9ff;
}

.detail-close {
  margin-left: auto;
  background: transparent;
  border: 1px solid #7a8a9e;
  color: #7a8a9e;
  width: 28px;
  height: 28px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.detail-close:hover {
  border-color: #00d9ff;
  color: #00d9ff;
}

.detail-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.detail-section {
  background: rgba(5, 8, 16, 0.5);
  border: 1px solid #1f2937;
  border-radius: 6px;
  padding: 16px;
}

.detail-section-title {
  font-size: 10px;
  color: #00d9ff;
  letter-spacing: 1px;
  margin-bottom: 12px;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.stat-item {
  text-align: center;
}

.stat-label {
  font-size: 9px;
  color: #7a8a9e;
  letter-spacing: 1px;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 20px;
  color: #00d9ff;
  font-weight: bold;
}

.info-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #1f2937;
}

.info-label {
  font-size: 9px;
  color: #7a8a9e;
  letter-spacing: 1px;
}

.info-value {
  font-size: 11px;
  color: #e0e7f1;
  font-family: monospace;
}
</style>
