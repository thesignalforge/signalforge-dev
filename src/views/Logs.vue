<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useDockerStore } from '@/stores/docker'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ScrollText, Download, Trash2 } from 'lucide-vue-next'

const route = useRoute()
const dockerStore = useDockerStore()

const selectedContainer = ref<string | null>(null)
const logs = ref<string[]>([])

const containers = computed(() => dockerStore.containers)

onMounted(() => {
  // Check if a container was passed via query param
  const containerId = route.query.container as string
  if (containerId) {
    selectedContainer.value = containerId
  }
})

function selectContainer(id: string) {
  selectedContainer.value = id
  logs.value = ['Log streaming will be implemented in the next iteration...']
}

function clearLogs() {
  logs.value = []
}
</script>

<template>
  <div class="flex h-[calc(100vh-10rem)] gap-4">
    <!-- Container List -->
    <Card class="w-64 shrink-0">
      <CardHeader>
        <CardTitle class="text-sm">Containers</CardTitle>
      </CardHeader>
      <CardContent class="space-y-1 p-2">
        <button
          v-for="container in containers"
          :key="container.id"
          @click="selectContainer(container.id)"
          :class="[
            'flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-all',
            selectedContainer === container.id
              ? 'bg-cyan/20 text-cyan'
              : 'text-text-dim hover:bg-bg-elevated hover:text-text'
          ]"
        >
          <div
            :class="[
              'h-2 w-2 rounded-full',
              container.state === 'running' ? 'bg-green-400' : 'bg-gray-500'
            ]"
          />
          <span class="truncate">{{ container.name }}</span>
        </button>

        <div
          v-if="containers.length === 0"
          class="py-4 text-center text-sm text-text-dim"
        >
          No containers
        </div>
      </CardContent>
    </Card>

    <!-- Log Viewer -->
    <Card class="flex-1">
      <CardHeader class="flex flex-row items-center justify-between">
        <CardTitle class="flex items-center gap-2 text-sm">
          <ScrollText class="h-4 w-4 text-cyan" />
          {{ selectedContainer ? containers.find(c => c.id === selectedContainer)?.name : 'Select a container' }}
        </CardTitle>
        <div v-if="selectedContainer" class="flex gap-2">
          <Button variant="ghost" size="sm" @click="clearLogs">
            <Trash2 class="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="sm">
            <Download class="h-4 w-4" />
          </Button>
        </div>
      </CardHeader>
      <CardContent class="h-full">
        <div
          v-if="!selectedContainer"
          class="flex h-full items-center justify-center text-text-dim"
        >
          Select a container to view logs
        </div>
        <div
          v-else
          class="h-[calc(100%-2rem)] overflow-auto rounded-lg bg-bg-deep p-4 font-mono text-sm"
        >
          <div v-if="logs.length === 0" class="text-text-dim">
            No logs available
          </div>
          <div v-for="(line, i) in logs" :key="i" class="text-text-dim">
            {{ line }}
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
