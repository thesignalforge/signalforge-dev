<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useProjectStore, type Project } from '@/stores/project'
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter
} from '@/components/ui/dialog'
import {
  FolderOpen,
  Plus,
  Play,
  Square,
  RefreshCw,
  Trash2,
  Folder,
  ChevronRight,
  ArrowLeft
} from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'

const projectStore = useProjectStore()

const showCreateDialog = ref(false)
const showDeleteDialog = ref(false)
const projectToDelete = ref<Project | null>(null)

const newProject = ref({
  name: '',
  rootPath: ''
})

const currentPath = ref('')
const directoryEntries = ref<any[]>([])
const pathHistory = ref<string[]>([])

onMounted(async () => {
  await projectStore.loadProjects()
  currentPath.value = await projectStore.getHomeDir()
  await loadDirectory(currentPath.value)
})

async function loadDirectory(path: string) {
  directoryEntries.value = await projectStore.listDirectory(path)
  currentPath.value = path
}

async function navigateToDirectory(path: string) {
  pathHistory.value.push(currentPath.value)
  await loadDirectory(path)
}

async function navigateBack() {
  if (pathHistory.value.length > 0) {
    const previousPath = pathHistory.value.pop()!
    await loadDirectory(previousPath)
  }
}

function selectDirectory(path: string) {
  newProject.value.rootPath = path
}

async function browseDirectory() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Project Root Directory'
  })

  if (selected && typeof selected === 'string') {
    newProject.value.rootPath = selected
  }
}

async function createProject() {
  if (!newProject.value.name || !newProject.value.rootPath) return

  try {
    await projectStore.createProject(newProject.value.name, newProject.value.rootPath)
    showCreateDialog.value = false
    newProject.value = { name: '', rootPath: '' }
  } catch (e) {
    console.error('Failed to create project:', e)
  }
}

async function startProject(project: Project) {
  try {
    await projectStore.composeUp(project.id)
  } catch (e) {
    console.error('Failed to start project:', e)
  }
}

async function stopProject(project: Project) {
  try {
    await projectStore.composeDown(project.id)
  } catch (e) {
    console.error('Failed to stop project:', e)
  }
}

async function restartProject(project: Project) {
  try {
    await projectStore.composeRestart(project.id)
  } catch (e) {
    console.error('Failed to restart project:', e)
  }
}

function confirmDelete(project: Project) {
  projectToDelete.value = project
  showDeleteDialog.value = true
}

async function deleteProject() {
  if (!projectToDelete.value) return

  try {
    await projectStore.deleteProject(projectToDelete.value.id)
    showDeleteDialog.value = false
    projectToDelete.value = null
  } catch (e) {
    console.error('Failed to delete project:', e)
  }
}

function formatDate(timestamp: number) {
  return new Date(timestamp * 1000).toLocaleDateString()
}

function getEnabledServices(project: Project) {
  return project.services.filter(s => s.enabled).map(s => s.name)
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-text">Projects</h1>
        <p class="text-sm text-text-dim">Manage your Docker Compose projects</p>
      </div>
      <Button @click="showCreateDialog = true">
        <Plus class="mr-2 h-4 w-4" />
        New Project
      </Button>
    </div>

    <!-- Projects Grid -->
    <div v-if="projectStore.projects.length > 0" class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card
        v-for="project in projectStore.projects"
        :key="project.id"
        class="transition-all duration-300 hover:border-cyan/50"
      >
        <CardHeader class="pb-3">
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-3">
              <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-cyan/10">
                <FolderOpen class="h-5 w-5 text-cyan" />
              </div>
              <div>
                <CardTitle class="text-base">{{ project.name }}</CardTitle>
                <CardDescription class="text-xs font-mono">
                  {{ project.root_path.split('/').slice(-2).join('/') }}
                </CardDescription>
              </div>
            </div>
          </div>
        </CardHeader>

        <CardContent class="space-y-3">
          <!-- Services -->
          <div>
            <p class="mb-2 text-xs text-text-dim">Services</p>
            <div class="flex flex-wrap gap-1">
              <Badge
                v-for="service in getEnabledServices(project)"
                :key="service"
                variant="secondary"
                class="text-xs"
              >
                {{ service }}
              </Badge>
            </div>
          </div>

          <!-- Created -->
          <div class="flex items-center justify-between text-xs text-text-dim">
            <span>Created</span>
            <span>{{ formatDate(project.created_at) }}</span>
          </div>
        </CardContent>

        <CardFooter class="gap-2 border-t border-border pt-4">
          <Button size="sm" variant="outline" @click="startProject(project)">
            <Play class="mr-1 h-3 w-3" />
            Start
          </Button>
          <Button size="sm" variant="outline" @click="stopProject(project)">
            <Square class="mr-1 h-3 w-3" />
            Stop
          </Button>
          <Button size="sm" variant="ghost" @click="restartProject(project)">
            <RefreshCw class="h-3 w-3" />
          </Button>
          <div class="flex-1" />
          <Button size="sm" variant="ghost" @click="confirmDelete(project)">
            <Trash2 class="h-3 w-3 text-red-400" />
          </Button>
        </CardFooter>
      </Card>
    </div>

    <!-- Empty State -->
    <div
      v-else
      class="flex flex-col items-center justify-center rounded-lg border border-dashed border-border py-16"
    >
      <FolderOpen class="mb-4 h-12 w-12 text-text-dim" />
      <h3 class="text-lg font-medium text-text">No projects yet</h3>
      <p class="mt-1 text-sm text-text-dim">Create your first Docker Compose project</p>
      <Button class="mt-4" @click="showCreateDialog = true">
        <Plus class="mr-2 h-4 w-4" />
        Create Project
      </Button>
    </div>

    <!-- Create Project Dialog -->
    <Dialog v-model:open="showCreateDialog">
      <DialogContent class="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Create New Project</DialogTitle>
          <DialogDescription>
            Set up a new Docker Compose project with nginx, PHP, and databases.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <!-- Project Name -->
          <div class="space-y-2">
            <Label>Project Name</Label>
            <Input
              v-model="newProject.name"
              placeholder="my-awesome-project"
            />
          </div>

          <!-- Project Root -->
          <div class="space-y-2">
            <Label>Project Root Directory</Label>
            <div class="flex gap-2">
              <Input
                v-model="newProject.rootPath"
                placeholder="/path/to/your/project"
                readonly
                class="flex-1"
              />
              <Button variant="outline" @click="browseDirectory">
                <FolderOpen class="mr-2 h-4 w-4" />
                Browse
              </Button>
            </div>
          </div>

          <!-- Directory Browser -->
          <div class="space-y-2">
            <div class="flex items-center gap-2">
              <Button
                size="sm"
                variant="ghost"
                :disabled="pathHistory.length === 0"
                @click="navigateBack"
              >
                <ArrowLeft class="h-4 w-4" />
              </Button>
              <span class="flex-1 truncate font-mono text-sm text-text-dim">
                {{ currentPath }}
              </span>
            </div>

            <ScrollArea class="h-48 rounded-md border border-border bg-bg-deep">
              <div class="p-2">
                <div
                  v-for="entry in directoryEntries.filter(e => e.is_dir)"
                  :key="entry.path"
                  class="flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 hover:bg-bg-surface"
                  :class="{ 'bg-cyan/10 text-cyan': newProject.rootPath === entry.path }"
                  @click="selectDirectory(entry.path)"
                  @dblclick="navigateToDirectory(entry.path)"
                >
                  <Folder class="h-4 w-4 text-cyan" />
                  <span class="flex-1 truncate text-sm">{{ entry.name }}</span>
                  <ChevronRight class="h-4 w-4 text-text-dim" />
                </div>
              </div>
            </ScrollArea>
            <p class="text-xs text-text-dim">
              Double-click to navigate, single-click to select
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showCreateDialog = false">
            Cancel
          </Button>
          <Button
            @click="createProject"
            :disabled="!newProject.name || !newProject.rootPath"
          >
            Create Project
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Delete Confirmation Dialog -->
    <Dialog v-model:open="showDeleteDialog">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Project</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ projectToDelete?.name }}"?
            This will remove the project configuration but not your files.
          </DialogDescription>
        </DialogHeader>

        <DialogFooter>
          <Button variant="outline" @click="showDeleteDialog = false">
            Cancel
          </Button>
          <Button variant="destructive" @click="deleteProject">
            Delete Project
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
