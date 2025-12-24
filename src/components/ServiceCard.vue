<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ContainerInfo, ContainerStats } from '@/stores/docker'
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatBytes, formatUptime, truncateId } from '@/lib/utils'
import { Play, Square, RotateCw, Terminal } from 'lucide-vue-next'

interface Props {
  container: ContainerInfo
  stats?: ContainerStats
}

const props = defineProps<Props>()

const emit = defineEmits<{
  start: [id: string]
  stop: [id: string]
  restart: [id: string]
  logs: [id: string]
}>()

const loading = ref(false)

const isRunning = computed(() => props.container.state === 'running')

const statusBadgeVariant = computed(() => {
  switch (props.container.state) {
    case 'running':
      return 'success'
    case 'paused':
      return 'warning'
    default:
      return 'secondary'
  }
})

const portMappings = computed(() => {
  return props.container.ports
    .filter((p) => p.public_port)
    .map((p) => `${p.public_port}:${p.private_port}`)
})

function handleStart() {
  loading.value = true
  emit('start', props.container.id)
  setTimeout(() => { loading.value = false }, 1000)
}

function handleStop() {
  loading.value = true
  emit('stop', props.container.id)
  setTimeout(() => { loading.value = false }, 1000)
}

function handleRestart() {
  loading.value = true
  emit('restart', props.container.id)
  setTimeout(() => { loading.value = false }, 1000)
}
</script>

<template>
  <Card class="group transition-all duration-300 hover:border-cyan/50 hover:shadow-[0_0_20px_rgba(0,217,255,0.1)]">
    <CardHeader class="pb-3">
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-3">
          <div
            :class="[
              'h-3 w-3 rounded-full transition-all',
              isRunning
                ? 'bg-green-400 shadow-[0_0_8px_rgba(74,222,128,0.6)]'
                : 'bg-gray-500'
            ]"
          />
          <div>
            <CardTitle class="text-base">{{ container.name }}</CardTitle>
            <p class="font-mono text-xs text-text-dim">
              {{ truncateId(container.id) }}
            </p>
          </div>
        </div>
        <Badge :variant="statusBadgeVariant">
          {{ container.state }}
        </Badge>
      </div>
    </CardHeader>

    <CardContent class="space-y-4">
      <!-- Image -->
      <div>
        <p class="text-xs text-text-dim">Image</p>
        <p class="truncate font-mono text-sm text-text">{{ container.image }}</p>
      </div>

      <!-- Ports -->
      <div v-if="portMappings.length">
        <p class="text-xs text-text-dim">Ports</p>
        <div class="flex flex-wrap gap-1">
          <Badge
            v-for="port in portMappings"
            :key="port"
            variant="outline"
            class="font-mono text-xs"
          >
            {{ port }}
          </Badge>
        </div>
      </div>

      <!-- Stats (only for running containers) -->
      <div v-if="isRunning && stats" class="grid grid-cols-3 gap-4">
        <div>
          <p class="text-xs text-text-dim">CPU</p>
          <p class="font-mono text-lg text-cyan">
            {{ stats.cpu_percent.toFixed(1) }}%
          </p>
        </div>
        <div>
          <p class="text-xs text-text-dim">Memory</p>
          <p class="font-mono text-lg text-cyan">
            {{ formatBytes(stats.memory_usage) }}
          </p>
        </div>
        <div>
          <p class="text-xs text-text-dim">Uptime</p>
          <p class="font-mono text-lg text-text">
            {{ formatUptime(container.created) }}
          </p>
        </div>
      </div>

      <!-- Status message for stopped containers -->
      <div v-else-if="!isRunning" class="rounded-md bg-bg-elevated p-3 text-center">
        <p class="text-sm text-text-dim">{{ container.status }}</p>
      </div>
    </CardContent>

    <CardFooter class="gap-2 border-t border-border pt-4">
      <Button
        v-if="!isRunning"
        size="sm"
        :disabled="loading"
        @click="handleStart"
      >
        <Play class="mr-1.5 h-4 w-4" />
        Start
      </Button>

      <Button
        v-if="isRunning"
        size="sm"
        variant="outline"
        :disabled="loading"
        @click="handleStop"
      >
        <Square class="mr-1.5 h-4 w-4" />
        Stop
      </Button>

      <Button
        v-if="isRunning"
        size="sm"
        variant="ghost"
        :disabled="loading"
        @click="handleRestart"
      >
        <RotateCw class="mr-1.5 h-4 w-4" />
        Restart
      </Button>

      <Button
        size="sm"
        variant="ghost"
        @click="emit('logs', container.id)"
      >
        <Terminal class="mr-1.5 h-4 w-4" />
        Logs
      </Button>
    </CardFooter>
  </Card>
</template>
