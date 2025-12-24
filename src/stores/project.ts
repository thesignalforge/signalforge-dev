import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface PortMapping {
  host: number
  container: number
}

export interface ServiceConfig {
  name: string
  image: string
  enabled: boolean
  ports: PortMapping[]
  environment: Record<string, string>
}

export interface VolumeMapping {
  host_path: string
  container_path: string
  read_only: boolean
}

export interface Project {
  id: string
  name: string
  root_path: string
  compose_path: string
  services: ServiceConfig[]
  volumes: VolumeMapping[]
  environment: Record<string, string>
  created_at: number
  updated_at: number
}

export interface DirectoryEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  modified: number
}

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const projectCount = computed(() => projects.value.length)

  async function loadProjects() {
    try {
      loading.value = true
      error.value = null
      projects.value = await invoke<Project[]>('list_projects')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function getProject(id: string) {
    try {
      loading.value = true
      error.value = null
      currentProject.value = await invoke<Project>('get_project', { id })
      return currentProject.value
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function createProject(name: string, rootPath: string) {
    try {
      loading.value = true
      error.value = null
      const project = await invoke<Project>('create_project', { name, rootPath })
      projects.value.push(project)
      return project
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateProject(project: Project) {
    try {
      loading.value = true
      error.value = null
      const updated = await invoke<Project>('update_project', { project })
      const idx = projects.value.findIndex(p => p.id === updated.id)
      if (idx !== -1) {
        projects.value[idx] = updated
      }
      return updated
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteProject(id: string) {
    try {
      loading.value = true
      error.value = null
      await invoke('delete_project', { id })
      projects.value = projects.value.filter(p => p.id !== id)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function composeUp(projectId: string) {
    try {
      loading.value = true
      error.value = null
      return await invoke<string>('compose_up', { projectId })
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function composeDown(projectId: string) {
    try {
      loading.value = true
      error.value = null
      return await invoke<string>('compose_down', { projectId })
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function composeRestart(projectId: string) {
    try {
      loading.value = true
      error.value = null
      return await invoke<string>('compose_restart', { projectId })
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function listDirectory(path: string): Promise<DirectoryEntry[]> {
    try {
      return await invoke<DirectoryEntry[]>('list_directory', { path })
    } catch (e) {
      error.value = String(e)
      return []
    }
  }

  async function getHomeDir(): Promise<string> {
    try {
      return await invoke<string>('get_home_dir')
    } catch (e) {
      return '/home'
    }
  }

  return {
    projects,
    currentProject,
    loading,
    error,
    projectCount,
    loadProjects,
    getProject,
    createProject,
    updateProject,
    deleteProject,
    composeUp,
    composeDown,
    composeRestart,
    listDirectory,
    getHomeDir
  }
})
