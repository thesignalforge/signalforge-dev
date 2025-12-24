<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useDockerStore } from '@/stores/docker'
import Sidebar from '@/components/Sidebar.vue'

const dockerStore = useDockerStore()

let refreshInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  // Check Docker connection on mount
  await dockerStore.checkConnection()

  if (dockerStore.isConnected) {
    // Initial load
    await dockerStore.refresh()

    // Auto-refresh every 5 seconds
    refreshInterval = setInterval(() => {
      dockerStore.refresh()
    }, 5000)
  }
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>

<template>
  <div class="flex h-screen bg-bg">
    <Sidebar />

    <main class="flex-1 overflow-auto">
      <!-- Header -->
      <header class="sticky top-0 z-10 border-b border-border bg-bg/80 backdrop-blur-sm">
        <div class="flex h-14 items-center justify-between px-6">
          <div>
            <h2 class="text-lg font-semibold text-text">
              <router-view v-slot="{ route }">
                {{ route.name ? route.name.toString().charAt(0).toUpperCase() + route.name.toString().slice(1) : 'Dashboard' }}
              </router-view>
            </h2>
          </div>

          <div class="flex items-center gap-4">
            <!-- Loading indicator -->
            <div
              v-if="dockerStore.loading"
              class="h-2 w-2 animate-pulse rounded-full bg-cyan"
            />

            <!-- Error indicator -->
            <div
              v-if="dockerStore.error"
              class="text-sm text-red-400"
              :title="dockerStore.error"
            >
              Error
            </div>
          </div>
        </div>
      </header>

      <!-- Content -->
      <div class="p-6">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
