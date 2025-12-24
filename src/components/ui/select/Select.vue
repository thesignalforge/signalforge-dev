<script setup lang="ts">
import { ref, provide, computed } from 'vue'
import { cn } from '@/lib/utils'
import { ChevronDown } from 'lucide-vue-next'

interface Props {
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  placeholder: 'Select an option',
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const isOpen = ref(false)
const selectedLabel = ref('')

provide('selectValue', computed(() => props.modelValue))
provide('selectOpen', isOpen)
provide('selectDisabled', computed(() => props.disabled))
provide('onSelect', (value: string, label: string) => {
  emit('update:modelValue', value)
  selectedLabel.value = label
  isOpen.value = false
})

function toggleOpen() {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
  }
}

function close() {
  isOpen.value = false
}

const triggerClass = computed(() =>
  cn(
    'flex h-10 w-full items-center justify-between rounded-md border border-border',
    'bg-bg-deep px-3 py-2 text-sm text-text ring-offset-bg',
    'placeholder:text-text-dim focus:outline-none focus:ring-2 focus:ring-cyan',
    'focus:ring-offset-2 focus:border-cyan disabled:cursor-not-allowed disabled:opacity-50',
    'transition-all duration-200',
    isOpen.value && 'ring-2 ring-cyan border-cyan',
    props.class
  )
)
</script>

<template>
  <div class="relative" v-click-outside="close">
    <button
      type="button"
      :class="triggerClass"
      :disabled="disabled"
      @click="toggleOpen"
    >
      <span :class="{ 'text-text-dim': !modelValue }">
        {{ selectedLabel || placeholder }}
      </span>
      <ChevronDown
        class="h-4 w-4 text-text-dim transition-transform duration-200"
        :class="{ 'rotate-180': isOpen }"
      />
    </button>

    <Transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="isOpen"
        class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border bg-bg-elevated py-1 shadow-lg"
      >
        <slot />
      </div>
    </Transition>
  </div>
</template>

<script lang="ts">
// Directive for clicking outside
interface ClickOutsideElement extends HTMLElement {
  _clickOutside?: (event: MouseEvent) => void
}

export default {
  directives: {
    'click-outside': {
      mounted(el: ClickOutsideElement, binding: { value: () => void }) {
        el._clickOutside = (event: MouseEvent) => {
          if (!(el === event.target || el.contains(event.target as Node))) {
            binding.value()
          }
        }
        document.addEventListener('click', el._clickOutside)
      },
      unmounted(el: ClickOutsideElement) {
        if (el._clickOutside) {
          document.removeEventListener('click', el._clickOutside)
        }
      }
    }
  }
}
</script>
