<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  class?: string
}

const props = defineProps<Props>()

const containerClass = computed(() =>
  cn(
    'relative overflow-hidden',
    props.class
  )
)
</script>

<template>
  <div :class="containerClass">
    <div class="h-full w-full overflow-auto scrollbar-thin scrollbar-track-bg-deep scrollbar-thumb-border hover:scrollbar-thumb-text-dim">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.scrollbar-thin::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  @apply bg-bg-deep rounded-full;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  @apply bg-border rounded-full;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  @apply bg-text-dim;
}
</style>
