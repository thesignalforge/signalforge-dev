<script setup lang="ts">
import { Card, CardContent } from '@/components/ui/card'

interface Props {
  title: string
  value: string | number
  subtitle?: string
  icon?: any
  trend?: 'up' | 'down' | 'neutral'
  variant?: 'default' | 'cyan' | 'success' | 'warning' | 'danger'
}

withDefaults(defineProps<Props>(), {
  variant: 'default',
  trend: 'neutral'
})

const variantStyles = {
  default: 'text-text',
  cyan: 'text-cyan',
  success: 'text-green-400',
  warning: 'text-yellow-400',
  danger: 'text-red-400'
}

const iconBgStyles = {
  default: 'bg-bg-elevated',
  cyan: 'bg-cyan/10',
  success: 'bg-green-500/10',
  warning: 'bg-yellow-500/10',
  danger: 'bg-red-500/10'
}
</script>

<template>
  <Card class="glow-hover">
    <CardContent class="p-6">
      <div class="flex items-start justify-between">
        <div class="space-y-2">
          <p class="text-sm text-text-dim">{{ title }}</p>
          <p
            :class="[
              'font-mono text-3xl font-semibold',
              variantStyles[variant]
            ]"
          >
            {{ value }}
          </p>
          <p v-if="subtitle" class="text-xs text-text-dim">
            {{ subtitle }}
          </p>
        </div>

        <div
          v-if="icon"
          :class="[
            'flex h-12 w-12 items-center justify-center rounded-lg',
            iconBgStyles[variant]
          ]"
        >
          <component
            :is="icon"
            :class="['h-6 w-6', variantStyles[variant]]"
          />
        </div>
      </div>
    </CardContent>
  </Card>
</template>
