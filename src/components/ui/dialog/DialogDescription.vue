<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  class?: string
}

const props = defineProps<Props>()

const descriptionClass = computed(() =>
  cn(
    'text-sm text-text-dim',
    props.class
  )
)
</script>

<template>
  <p :class="descriptionClass">
    <slot />
  </p>
</template>
