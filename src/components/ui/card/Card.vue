<script setup lang="ts">
import { cn } from '@/lib/utils'

interface Props {
  class?: string
}

const props = defineProps<Props>()
</script>

<template>
  <div
    :class="cn(
      'rounded-lg border border-border bg-bg-surface text-text shadow-sm',
      'transition-all duration-300 hover:border-cyan/30',
      props.class
    )"
  >
    <slot />
  </div>
</template>
