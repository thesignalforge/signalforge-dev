<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
  size?: 'default' | 'sm' | 'lg' | 'icon'
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'default',
  disabled: false
})

const variantClasses = {
  default: 'bg-cyan text-bg-deep hover:bg-cyan-bright shadow-[0_0_10px_rgba(0,217,255,0.3)] hover:shadow-[0_0_20px_rgba(0,217,255,0.5)]',
  destructive: 'bg-red-500/20 text-red-400 border border-red-500/50 hover:bg-red-500/30',
  outline: 'border border-border bg-transparent hover:bg-bg-elevated hover:border-cyan/50 text-text',
  secondary: 'bg-bg-elevated text-text hover:bg-bg-surface',
  ghost: 'hover:bg-bg-elevated text-text-dim hover:text-text',
  link: 'text-cyan underline-offset-4 hover:underline'
}

const sizeClasses = {
  default: 'h-10 px-4 py-2',
  sm: 'h-8 px-3 text-sm',
  lg: 'h-11 px-8',
  icon: 'h-10 w-10'
}

const buttonClass = computed(() =>
  cn(
    'inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium',
    'ring-offset-bg transition-all duration-200 focus-visible:outline-none focus-visible:ring-2',
    'focus-visible:ring-cyan focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
    variantClasses[props.variant],
    sizeClasses[props.size]
  )
)
</script>

<template>
  <button :class="buttonClass" :disabled="disabled">
    <slot />
  </button>
</template>
