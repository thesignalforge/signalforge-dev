<script setup lang="ts">
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { FolderOpen, Plus } from 'lucide-vue-next'
</script>

<template>
  <div class="space-y-6">
    <!-- Coming Soon -->
    <Card class="border-dashed">
      <CardContent class="flex flex-col items-center justify-center py-16">
        <div class="flex h-16 w-16 items-center justify-center rounded-full bg-cyan/10">
          <FolderOpen class="h-8 w-8 text-cyan" />
        </div>
        <h3 class="mt-4 text-lg font-medium text-text">Project Management</h3>
        <p class="mt-2 max-w-sm text-center text-sm text-text-dim">
          Manage your Docker Compose projects, create new stacks, and organize your development environments.
        </p>
        <Button class="mt-6" disabled>
          <Plus class="mr-2 h-4 w-4" />
          Coming Soon
        </Button>
      </CardContent>
    </Card>
  </div>
</template>
