<script setup lang="ts">
import { inject, computed, ComputedRef } from 'vue'
import { cn } from '@/lib/utils'
import { Check } from 'lucide-vue-next'

interface Props {
  value: string
  disabled?: boolean
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
})

const selectValue = inject<ComputedRef<string>>('selectValue')
const onSelect = inject<(value: string, label: string) => void>('onSelect')

const isSelected = computed(() => selectValue?.value === props.value)

const itemClass = computed(() =>
  cn(
    'relative flex cursor-pointer select-none items-center rounded-sm px-3 py-2 text-sm text-text',
    'outline-none transition-colors',
    'hover:bg-bg-surface hover:text-cyan',
    'focus:bg-bg-surface focus:text-cyan',
    isSelected.value && 'bg-cyan/10 text-cyan',
    props.disabled && 'pointer-events-none opacity-50',
    props.class
  )
)

function handleSelect(event: Event) {
  if (props.disabled) return
  const target = event.currentTarget as HTMLElement
  const label = target.textContent?.trim() || props.value
  onSelect?.(props.value, label)
}
</script>

<template>
  <div
    :class="itemClass"
    @click="handleSelect"
  >
    <span class="flex-1">
      <slot />
    </span>
    <Check
      v-if="isSelected"
      class="h-4 w-4 text-cyan"
    />
  </div>
</template>
