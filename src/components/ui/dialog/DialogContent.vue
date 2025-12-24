<script setup lang="ts">
import { computed, inject } from 'vue'
import { cn } from '@/lib/utils'
import { X } from 'lucide-vue-next'

interface Props {
  class?: string
  showClose?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showClose: true
})

const close = inject<() => void>('dialogClose')

const contentClass = computed(() =>
  cn(
    'relative z-50 grid w-full max-w-lg gap-4 border border-border bg-bg-elevated p-6 shadow-lg',
    'rounded-lg animate-in fade-in-0 zoom-in-95',
    props.class
  )
)
</script>

<template>
  <div :class="contentClass">
    <slot />

    <button
      v-if="showClose"
      class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-bg transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-cyan focus:ring-offset-2"
      @click="close"
    >
      <X class="h-4 w-4 text-text-dim" />
      <span class="sr-only">Close</span>
    </button>
  </div>
</template>
