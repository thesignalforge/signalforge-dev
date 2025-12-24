<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  modelValue?: string | number
  type?: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  modelValue: '',
  disabled: false,
  readonly: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const inputClass = computed(() =>
  cn(
    'flex h-10 w-full rounded-md border border-border bg-bg-deep px-3 py-2 text-sm text-text',
    'ring-offset-bg file:border-0 file:bg-transparent file:text-sm file:font-medium',
    'placeholder:text-text-dim focus-visible:outline-none focus-visible:ring-2',
    'focus-visible:ring-cyan focus-visible:ring-offset-2 focus-visible:border-cyan',
    'disabled:cursor-not-allowed disabled:opacity-50',
    'transition-all duration-200',
    props.class
  )
)

function handleInput(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <input
    :type="type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    :class="inputClass"
    @input="handleInput"
  />
</template>
