<script setup lang="ts">
import { computed } from 'vue'
import { useDockerStore } from '@/stores/docker'
import { useRouter } from 'vue-router'
import StatsCard from '@/components/StatsCard.vue'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatBytes, truncateId } from '@/lib/utils'
import {
  Container,
  PlayCircle,
  StopCircle,
  Cpu,
  HardDrive,
  Activity,
  ArrowRight
} from 'lucide-vue-next'

const router = useRouter()
const dockerStore = useDockerStore()

const recentContainers = computed(() =>
  dockerStore.containers.slice(0, 5)
)

const systemStats = computed(() => {
  if (!dockerStore.dockerInfo) return null
  return dockerStore.dockerInfo
})

function getContainerStats(id: string) {
  return dockerStore.containerStats.get(id)
}

function goToServices() {
  router.push('/services')
}
</script>

<template>
  <div class="space-y-6">
    <!-- Not Connected Warning -->
    <Card v-if="!dockerStore.isConnected" class="border-yellow-500/50 bg-yellow-500/10">
      <CardContent class="flex items-center gap-4 p-6">
        <div class="flex h-12 w-12 items-center justify-center rounded-full bg-yellow-500/20">
          <Activity class="h-6 w-6 text-yellow-400" />
        </div>
        <div class="flex-1">
          <h3 class="font-semibold text-yellow-400">Docker Not Connected</h3>
          <p class="text-sm text-yellow-400/80">
            Unable to connect to Docker daemon. Make sure Docker is running.
          </p>
        </div>
        <Button variant="outline" @click="dockerStore.connect">
          Retry Connection
        </Button>
      </CardContent>
    </Card>

    <!-- Stats Grid -->
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
      <StatsCard
        title="Running"
        :value="dockerStore.runningContainers.length"
        subtitle="Active containers"
        :icon="PlayCircle"
        variant="success"
      />
      <StatsCard
        title="Stopped"
        :value="dockerStore.stoppedContainers.length"
        subtitle="Inactive containers"
        :icon="StopCircle"
        variant="default"
      />
      <StatsCard
        title="CPU Usage"
        :value="`${dockerStore.totalCpu.toFixed(1)}%`"
        subtitle="Total across containers"
        :icon="Cpu"
        variant="cyan"
      />
      <StatsCard
        title="Memory"
        :value="formatBytes(dockerStore.totalMemory)"
        subtitle="Total usage"
        :icon="HardDrive"
        variant="cyan"
      />
    </div>

    <!-- Recent Containers -->
    <Card>
      <CardHeader class="flex flex-row items-center justify-between">
        <CardTitle class="flex items-center gap-2">
          <Container class="h-5 w-5 text-cyan" />
          Recent Containers
        </CardTitle>
        <Button variant="ghost" size="sm" @click="goToServices">
          View All
          <ArrowRight class="ml-2 h-4 w-4" />
        </Button>
      </CardHeader>
      <CardContent>
        <div v-if="recentContainers.length === 0" class="py-8 text-center text-text-dim">
          No containers found
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="container in recentContainers"
            :key="container.id"
            class="flex items-center justify-between rounded-lg border border-border bg-bg-elevated p-4 transition-all hover:border-cyan/30"
          >
            <div class="flex items-center gap-4">
              <div
                :class="[
                  'h-3 w-3 rounded-full',
                  container.state === 'running'
                    ? 'bg-green-400 shadow-[0_0_8px_rgba(74,222,128,0.5)]'
                    : 'bg-red-400'
                ]"
              />
              <div>
                <p class="font-medium text-text">{{ container.name }}</p>
                <p class="font-mono text-xs text-text-dim">
                  {{ truncateId(container.id) }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-4">
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
                    {{ formatBytes(getContainerStats(container.id)?.memory_usage || 0) }}
                  </p>
                </div>
              </div>

              <Badge
                :variant="container.state === 'running' ? 'success' : 'secondary'"
              >
                {{ container.state }}
              </Badge>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- System Info -->
    <div v-if="systemStats" class="grid grid-cols-1 gap-4 md:grid-cols-3">
      <Card>
        <CardContent class="p-6">
          <p class="text-sm text-text-dim">Docker Version</p>
          <p class="font-mono text-xl text-text">{{ systemStats.docker_version }}</p>
        </CardContent>
      </Card>
      <Card>
        <CardContent class="p-6">
          <p class="text-sm text-text-dim">System</p>
          <p class="font-mono text-xl text-text">
            {{ systemStats.os_type }} / {{ systemStats.architecture }}
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardContent class="p-6">
          <p class="text-sm text-text-dim">Resources</p>
          <p class="font-mono text-xl text-text">
            {{ systemStats.cpus }} CPUs / {{ formatBytes(systemStats.memory_total) }}
          </p>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
