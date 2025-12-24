<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useDockerStore } from '@/stores/docker'
import ServiceCard from '@/components/ServiceCard.vue'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { RefreshCw, Filter, Grid, List } from 'lucide-vue-next'

const router = useRouter()
const dockerStore = useDockerStore()

type FilterType = 'all' | 'running' | 'stopped'
type ViewMode = 'grid' | 'list'

const filter = ref<FilterType>('all')
const viewMode = ref<ViewMode>('grid')
const refreshing = ref(false)

const filteredContainers = computed(() => {
  switch (filter.value) {
    case 'running':
      return dockerStore.runningContainers
    case 'stopped':
      return dockerStore.stoppedContainers
    default:
      return dockerStore.containers
  }
})

function getContainerStats(id: string) {
  return dockerStore.containerStats.get(id)
}

async function handleStart(id: string) {
  await dockerStore.startContainer(id)
}

async function handleStop(id: string) {
  await dockerStore.stopContainer(id)
}

async function handleRestart(id: string) {
  await dockerStore.restartContainer(id)
}

function handleLogs(id: string) {
  router.push({ path: '/logs', query: { container: id } })
}

async function refresh() {
  refreshing.value = true
  await dockerStore.refresh()
  setTimeout(() => {
    refreshing.value = false
  }, 500)
}
</script>

<template>
  <div class="space-y-6">
    <!-- Toolbar -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <!-- Filter buttons -->
        <div class="flex rounded-lg border border-border bg-bg-surface p-1">
          <button
            v-for="f in (['all', 'running', 'stopped'] as FilterType[])"
            :key="f"
            @click="filter = f"
            :class="[
              'rounded-md px-3 py-1.5 text-sm transition-all',
              filter === f
                ? 'bg-cyan/20 text-cyan'
                : 'text-text-dim hover:text-text'
            ]"
          >
            {{ f.charAt(0).toUpperCase() + f.slice(1) }}
            <Badge
              v-if="f === 'running'"
              variant="default"
              class="ml-1.5 h-5 px-1.5"
            >
              {{ dockerStore.runningContainers.length }}
            </Badge>
            <Badge
              v-else-if="f === 'stopped'"
              variant="secondary"
              class="ml-1.5 h-5 px-1.5"
            >
              {{ dockerStore.stoppedContainers.length }}
            </Badge>
            <Badge
              v-else
              variant="secondary"
              class="ml-1.5 h-5 px-1.5"
            >
              {{ dockerStore.containers.length }}
            </Badge>
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- View mode toggle -->
        <div class="flex rounded-lg border border-border bg-bg-surface p-1">
          <button
            @click="viewMode = 'grid'"
            :class="[
              'rounded-md p-1.5 transition-all',
              viewMode === 'grid'
                ? 'bg-cyan/20 text-cyan'
                : 'text-text-dim hover:text-text'
            ]"
          >
            <Grid class="h-4 w-4" />
          </button>
          <button
            @click="viewMode = 'list'"
            :class="[
              'rounded-md p-1.5 transition-all',
              viewMode === 'list'
                ? 'bg-cyan/20 text-cyan'
                : 'text-text-dim hover:text-text'
            ]"
          >
            <List class="h-4 w-4" />
          </button>
        </div>

        <!-- Refresh button -->
        <Button variant="outline" size="sm" @click="refresh" :disabled="refreshing">
          <RefreshCw
            :class="['h-4 w-4', refreshing && 'animate-spin']"
          />
          <span class="ml-2">Refresh</span>
        </Button>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-if="filteredContainers.length === 0"
      class="flex flex-col items-center justify-center rounded-lg border border-border bg-bg-surface py-16"
    >
      <div class="flex h-16 w-16 items-center justify-center rounded-full bg-bg-elevated">
        <Filter class="h-8 w-8 text-text-dim" />
      </div>
      <h3 class="mt-4 text-lg font-medium text-text">No containers found</h3>
      <p class="mt-1 text-sm text-text-dim">
        <template v-if="filter !== 'all'">
          No {{ filter }} containers. Try changing the filter.
        </template>
        <template v-else>
          No containers are available. Start a Docker container to see it here.
        </template>
      </p>
    </div>

    <!-- Grid view -->
    <div
      v-else-if="viewMode === 'grid'"
      class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3"
    >
      <ServiceCard
        v-for="container in filteredContainers"
        :key="container.id"
        :container="container"
        :stats="getContainerStats(container.id)"
        @start="handleStart"
        @stop="handleStop"
        @restart="handleRestart"
        @logs="handleLogs"
      />
    </div>

    <!-- List view -->
    <div v-else class="space-y-2">
      <div
        v-for="container in filteredContainers"
        :key="container.id"
        class="flex items-center justify-between rounded-lg border border-border bg-bg-surface p-4 transition-all hover:border-cyan/30"
      >
        <div class="flex items-center gap-4">
          <div
            :class="[
              'h-3 w-3 rounded-full',
              container.state === 'running'
                ? 'bg-green-400 shadow-[0_0_8px_rgba(74,222,128,0.5)]'
                : 'bg-gray-500'
            ]"
          />
          <div>
            <p class="font-medium text-text">{{ container.name }}</p>
            <p class="text-sm text-text-dim">{{ container.image }}</p>
          </div>
        </div>

        <div class="flex items-center gap-6">
          <div v-if="container.state === 'running' && getContainerStats(container.id)" class="flex gap-6 text-sm">
            <div class="text-right">
              <p class="text-text-dim">CPU</p>
              <p class="font-mono text-cyan">
                {{ getContainerStats(container.id)?.cpu_percent.toFixed(1) }}%
              </p>
            </div>
            <div class="text-right">
              <p class="text-text-dim">Memory</p>
              <p class="font-mono text-cyan">
                {{ getContainerStats(container.id)?.memory_usage }}
              </p>
            </div>
          </div>

          <Badge
            :variant="container.state === 'running' ? 'success' : 'secondary'"
          >
            {{ container.state }}
          </Badge>

          <div class="flex gap-1">
            <Button
              v-if="container.state !== 'running'"
              size="sm"
              @click="handleStart(container.id)"
            >
              Start
            </Button>
            <Button
              v-else
              size="sm"
              variant="outline"
              @click="handleStop(container.id)"
            >
              Stop
            </Button>
            <Button
              size="sm"
              variant="ghost"
              @click="handleLogs(container.id)"
            >
              Logs
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
