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
  height: calc(100vh - 8rem);
  min-height: 500px;
  background: #050810;
  border-radius: 8px;
  overflow: hidden;
}
</style>
