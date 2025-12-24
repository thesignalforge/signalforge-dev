<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  class?: string
}

const props = defineProps<Props>()

const titleClass = computed(() =>
  cn(
    'text-lg font-semibold leading-none tracking-tight text-text',
    props.class
  )
)
</script>

<template>
  <h2 :class="titleClass">
    <slot />
  </h2>
</template>
