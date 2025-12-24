<script setup lang="ts">
import { ref, provide, watch } from 'vue'

interface Props {
  open?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  open: false
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const isOpen = ref(props.open)

watch(() => props.open, (value) => {
  isOpen.value = value
})

function close() {
  isOpen.value = false
  emit('update:open', false)
}

provide('dialogClose', close)
provide('dialogOpen', isOpen)
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center"
      >
        <!-- Backdrop -->
        <div
          class="fixed inset-0 bg-black/80 backdrop-blur-sm"
          @click="close"
        />

        <!-- Content -->
        <slot />
      </div>
    </Transition>
  </Teleport>
</template>
