<script setup lang="ts">
import NetworkVisualization from '@/components/NetworkVisualization.vue'
</script>

<template>
  <div class="network-view">
    <NetworkVisualization />
  </div>
</template>

<style scoped>
.network-view {
  width: 100%;
  height: 100%;
  background: var(--bg-deep);
}
</style>
