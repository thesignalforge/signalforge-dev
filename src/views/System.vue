<script setup lang="ts">
import { computed } from 'vue'
import { useDockerStore } from '@/stores/docker'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { formatBytes } from '@/lib/utils'
import { Server, Cpu, HardDrive, Container, Image } from 'lucide-vue-next'

const dockerStore = useDockerStore()

const info = computed(() => dockerStore.dockerInfo)
</script>

<template>
  <div class="space-y-6">
    <!-- Connection Status -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Server class="h-5 w-5 text-cyan" />
          Docker Engine
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div class="flex items-center gap-4">
          <div
            :class="[
              'h-4 w-4 rounded-full',
              dockerStore.isConnected
                ? 'bg-green-400 shadow-[0_0_10px_rgba(74,222,128,0.5)]'
                : 'bg-red-400'
            ]"
          />
          <div>
            <p class="font-medium text-text">
              {{ dockerStore.isConnected ? 'Connected' : 'Disconnected' }}
            </p>
            <p class="text-sm text-text-dim">
              {{ dockerStore.isConnected ? 'Docker daemon is running' : 'Unable to connect to Docker' }}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- System Info -->
    <div v-if="info" class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
      <Card>
        <CardContent class="p-6">
          <div class="flex items-center gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-cyan/10">
              <Server class="h-6 w-6 text-cyan" />
            </div>
            <div>
              <p class="text-sm text-text-dim">Docker Version</p>
              <p class="font-mono text-xl text-text">{{ info.docker_version }}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent class="p-6">
          <div class="flex items-center gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-cyan/10">
              <Cpu class="h-6 w-6 text-cyan" />
            </div>
            <div>
              <p class="text-sm text-text-dim">CPUs</p>
              <p class="font-mono text-xl text-text">{{ info.cpus }} cores</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent class="p-6">
          <div class="flex items-center gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-cyan/10">
              <HardDrive class="h-6 w-6 text-cyan" />
            </div>
            <div>
              <p class="text-sm text-text-dim">Total Memory</p>
              <p class="font-mono text-xl text-text">{{ formatBytes(info.memory_total) }}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent class="p-6">
          <div class="flex items-center gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-green-500/10">
              <Container class="h-6 w-6 text-green-400" />
            </div>
            <div>
              <p class="text-sm text-text-dim">Running Containers</p>
              <p class="font-mono text-xl text-green-400">{{ info.containers_running }}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent class="p-6">
          <div class="flex items-center gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-bg-elevated">
              <Container class="h-6 w-6 text-text-dim" />
            </div>
            <div>
              <p class="text-sm text-text-dim">Stopped Containers</p>
              <p class="font-mono text-xl text-text">{{ info.containers_stopped }}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent class="p-6">
          <div class="flex items-center gap-4">
            <div class="flex h-12 w-12 items-center justify-center rounded-lg bg-cyan/10">
              <Image class="h-6 w-6 text-cyan" />
            </div>
            <div>
              <p class="text-sm text-text-dim">Images</p>
              <p class="font-mono text-xl text-cyan">{{ info.images }}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Platform Info -->
    <Card v-if="info">
      <CardHeader>
        <CardTitle>Platform</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <p class="text-sm text-text-dim">Operating System</p>
            <p class="font-mono text-text">{{ info.os_type }}</p>
          </div>
          <div>
            <p class="text-sm text-text-dim">Architecture</p>
            <p class="font-mono text-text">{{ info.architecture }}</p>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- No info available -->
    <Card v-else class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-16">
        <Server class="h-12 w-12 text-text-dim" />
        <p class="mt-4 text-text-dim">
          Connect to Docker to view system information
        </p>
      </CardContent>
    </Card>
  </div>
</template>
