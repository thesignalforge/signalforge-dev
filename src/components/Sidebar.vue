<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useDockerStore } from '@/stores/docker'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import {
  LayoutDashboard,
  Container,
  FolderOpen,
  ScrollText,
  Settings,
  Server,
  Cpu,
  Globe,
  ShieldCheck,
  Link
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const dockerStore = useDockerStore()

interface NavItem {
  name: string
  path: string
  icon: any
  badge?: () => number | string | null
}

const navItems: NavItem[] = [
  { name: 'Dashboard', path: '/', icon: LayoutDashboard },
  {
    name: 'Services',
    path: '/services',
    icon: Container,
    badge: () => dockerStore.runningContainers.length || null
  },
  { name: 'Projects', path: '/projects', icon: FolderOpen },
  { name: 'Nginx', path: '/nginx', icon: Globe },
  { name: 'SSL Certs', path: '/ssl', icon: ShieldCheck },
  { name: 'TLD (.sig)', path: '/tld', icon: Link },
  { name: 'Logs', path: '/logs', icon: ScrollText },
  { name: 'Config', path: '/config', icon: Settings },
  { name: 'System', path: '/system', icon: Server }
]

function isActive(path: string) {
  return route.path === path
}

function navigate(path: string) {
  router.push(path)
}
</script>

<template>
  <aside class="flex h-screen w-64 flex-col border-r border-border bg-bg-deep">
    <!-- Logo -->
    <div class="flex items-center gap-3 px-6 py-5">
      <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-cyan/10">
        <Cpu class="h-5 w-5 text-cyan" />
      </div>
      <div>
        <h1 class="text-lg font-semibold text-text">Signalforge</h1>
        <p class="text-xs text-text-dim">Dev Environment</p>
      </div>
    </div>

    <Separator />

    <!-- Connection Status -->
    <div class="px-4 py-3">
      <div class="flex items-center gap-2 rounded-md bg-bg-surface px-3 py-2">
        <div
          :class="[
            'h-2 w-2 rounded-full',
            dockerStore.isConnected ? 'bg-green-400 shadow-[0_0_6px_rgba(74,222,128,0.5)]' : 'bg-red-400'
          ]"
        />
        <span class="text-sm text-text-dim">
          Docker {{ dockerStore.isConnected ? 'Connected' : 'Disconnected' }}
        </span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 space-y-1 px-3 py-2">
      <button
        v-for="item in navItems"
        :key="item.path"
        @click="navigate(item.path)"
        :class="[
          'flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-all duration-200',
          isActive(item.path)
            ? 'bg-cyan/10 text-cyan shadow-[0_0_10px_rgba(0,217,255,0.15)]'
            : 'text-text-dim hover:bg-bg-surface hover:text-text'
        ]"
      >
        <component
          :is="item.icon"
          :class="['h-5 w-5', isActive(item.path) ? 'text-cyan' : '']"
        />
        <span class="flex-1 text-left">{{ item.name }}</span>
        <Badge
          v-if="item.badge && item.badge()"
          variant="default"
          class="h-5 min-w-5 justify-center px-1.5 text-xs"
        >
          {{ item.badge() }}
        </Badge>
      </button>
    </nav>

    <Separator />

    <!-- Footer Stats -->
    <div class="p-4">
      <div class="grid grid-cols-2 gap-2 text-xs">
        <div class="rounded-md bg-bg-surface p-2">
          <p class="text-text-dim">Running</p>
          <p class="font-mono text-lg text-cyan">
            {{ dockerStore.runningContainers.length }}
          </p>
        </div>
        <div class="rounded-md bg-bg-surface p-2">
          <p class="text-text-dim">Stopped</p>
          <p class="font-mono text-lg text-text">
            {{ dockerStore.stoppedContainers.length }}
          </p>
        </div>
      </div>
    </div>
  </aside>
</template>
