<script setup lang="ts">
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Settings, FileCode } from 'lucide-vue-next'
</script>

<template>
  <div class="space-y-6">
    <!-- Coming Soon -->
    <Card class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-16">
        <div class="flex h-16 w-16 items-center justify-center rounded-full bg-cyan/10">
          <FileCode class="h-8 w-8 text-cyan" />
        </div>
        <h3 class="mt-4 text-lg font-medium text-text">Configuration Editor</h3>
        <p class="mt-2 max-w-sm text-center text-sm text-text-dim">
          Edit docker-compose.yml files with Monaco Editor, featuring syntax highlighting and validation.
        </p>
        <Button class="mt-6" disabled>
          <Settings class="mr-2 h-4 w-4" />
          Coming Soon
        </Button>
      </CardContent>
    </Card>
  </div>
</template>
