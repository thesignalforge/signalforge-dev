<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning'
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default'
})

const variantClasses = {
  default: 'bg-cyan/20 text-cyan border-cyan/50',
  secondary: 'bg-bg-elevated text-text-dim border-border',
  destructive: 'bg-red-500/20 text-red-400 border-red-500/50',
  outline: 'bg-transparent text-text border-border',
  success: 'bg-green-500/20 text-green-400 border-green-500/50',
  warning: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/50'
}

const badgeClass = computed(() =>
  cn(
    'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold',
    'transition-colors focus:outline-none focus:ring-2 focus:ring-cyan focus:ring-offset-2',
    variantClasses[props.variant],
    props.class
  )
)
</script>

<template>
  <div :class="badgeClass">
    <slot />
  </div>
</template>
