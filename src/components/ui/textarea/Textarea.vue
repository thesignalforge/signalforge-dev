<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  rows?: number
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  disabled: false,
  readonly: false,
  rows: 4
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const textareaClass = computed(() =>
  cn(
    'flex min-h-[80px] w-full rounded-md border border-border bg-bg-deep px-3 py-2',
    'text-sm text-text font-mono ring-offset-bg placeholder:text-text-dim',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan',
    'focus-visible:ring-offset-2 focus-visible:border-cyan',
    'disabled:cursor-not-allowed disabled:opacity-50',
    'resize-none transition-all duration-200',
    props.class
  )
)

function handleInput(event: Event) {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <textarea
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    :rows="rows"
    :class="textareaClass"
    @input="handleInput"
  />
</template>
