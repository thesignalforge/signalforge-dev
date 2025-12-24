<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useNginxStore, type NginxVhost } from '@/stores/nginx'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter
} from '@/components/ui/dialog'
import {
  Globe,
  Plus,
  RefreshCw,
  CheckCircle,
  XCircle,
  Trash2,
  Edit,
  FileCode
} from 'lucide-vue-next'

const nginxStore = useNginxStore()

const showCreateDialog = ref(false)
const showEditDialog = ref(false)
const showConfigDialog = ref(false)
const selectedVhost = ref<NginxVhost | null>(null)
const configContent = ref('')

const newVhost = ref({
  serverName: '',
  documentRoot: '/var/www/html/public',
  phpEnabled: true,
  sslEnabled: false
})

onMounted(async () => {
  await nginxStore.loadVhosts()
})

async function createVhost() {
  if (!newVhost.value.serverName) return

  try {
    await nginxStore.createVhost(
      newVhost.value.serverName,
      newVhost.value.documentRoot,
      newVhost.value.phpEnabled,
      newVhost.value.sslEnabled
    )
    showCreateDialog.value = false
    newVhost.value = {
      serverName: '',
      documentRoot: '/var/www/html/public',
      phpEnabled: true,
      sslEnabled: false
    }
  } catch (e) {
    console.error('Failed to create vhost:', e)
  }
}

function editVhost(vhost: NginxVhost) {
  selectedVhost.value = { ...vhost }
  showEditDialog.value = true
}

async function updateVhost() {
  if (!selectedVhost.value) return

  try {
    await nginxStore.updateVhost(selectedVhost.value)
    showEditDialog.value = false
    selectedVhost.value = null
  } catch (e) {
    console.error('Failed to update vhost:', e)
  }
}

async function deleteVhost(id: string) {
  if (!confirm('Are you sure you want to delete this virtual host?')) return

  try {
    await nginxStore.deleteVhost(id)
  } catch (e) {
    console.error('Failed to delete vhost:', e)
  }
}

async function viewConfig(vhost: NginxVhost) {
  selectedVhost.value = vhost
  configContent.value = await nginxStore.getVhostConfig(vhost.id)
  showConfigDialog.value = true
}

async function saveConfig() {
  if (!selectedVhost.value) return

  try {
    await nginxStore.saveVhostConfig(selectedVhost.value.id, configContent.value)
    showConfigDialog.value = false
  } catch (e) {
    console.error('Failed to save config:', e)
  }
}

async function testConfig() {
  try {
    await nginxStore.testConfig()
  } catch (e) {
    console.error('Failed to test config:', e)
  }
}

async function reloadNginx() {
  try {
    await nginxStore.reloadNginx()
  } catch (e) {
    console.error('Failed to reload nginx:', e)
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-text">Nginx Configuration</h1>
        <p class="text-sm text-text-dim">Manage virtual hosts and server configuration</p>
      </div>
      <div class="flex gap-2">
        <Button variant="outline" @click="testConfig">
          <CheckCircle class="mr-2 h-4 w-4" />
          Test Config
        </Button>
        <Button variant="outline" @click="reloadNginx">
          <RefreshCw class="mr-2 h-4 w-4" />
          Reload
        </Button>
        <Button @click="showCreateDialog = true">
          <Plus class="mr-2 h-4 w-4" />
          Add Vhost
        </Button>
      </div>
    </div>

    <!-- Test Result -->
    <Card v-if="nginxStore.testResult" :class="nginxStore.testResult.success ? 'border-green-500/50' : 'border-red-500/50'">
      <CardContent class="flex items-center gap-3 py-4">
        <component
          :is="nginxStore.testResult.success ? CheckCircle : XCircle"
          :class="nginxStore.testResult.success ? 'text-green-400' : 'text-red-400'"
          class="h-5 w-5"
        />
        <span class="text-sm">
          {{ nginxStore.testResult.success ? 'Configuration test passed' : 'Configuration test failed' }}
        </span>
        <span v-if="!nginxStore.testResult.success" class="ml-auto text-xs text-red-400">
          {{ nginxStore.testResult.errors[0] }}
        </span>
      </CardContent>
    </Card>

    <!-- Virtual Hosts List -->
    <div v-if="nginxStore.vhosts.length > 0" class="space-y-4">
      <Card
        v-for="vhost in nginxStore.vhosts"
        :key="vhost.id"
        class="transition-all duration-300 hover:border-cyan/50"
      >
        <CardContent class="flex items-center gap-4 py-4">
          <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-cyan/10">
            <Globe class="h-5 w-5 text-cyan" />
          </div>

          <div class="flex-1">
            <div class="flex items-center gap-2">
              <span class="font-mono text-text">{{ vhost.server_name }}</span>
              <Badge v-if="vhost.ssl_enabled" variant="success" class="text-xs">SSL</Badge>
              <Badge v-if="vhost.php_enabled" variant="secondary" class="text-xs">PHP</Badge>
            </div>
            <p class="text-xs text-text-dim font-mono">{{ vhost.document_root }}</p>
          </div>

          <div class="flex gap-2">
            <Button size="sm" variant="ghost" @click="viewConfig(vhost)">
              <FileCode class="h-4 w-4" />
            </Button>
            <Button size="sm" variant="ghost" @click="editVhost(vhost)">
              <Edit class="h-4 w-4" />
            </Button>
            <Button size="sm" variant="ghost" @click="deleteVhost(vhost.id)">
              <Trash2 class="h-4 w-4 text-red-400" />
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Empty State -->
    <div
      v-else
      class="flex flex-col items-center justify-center rounded-lg border border-dashed border-border py-16"
    >
      <Globe class="mb-4 h-12 w-12 text-text-dim" />
      <h3 class="text-lg font-medium text-text">No virtual hosts</h3>
      <p class="mt-1 text-sm text-text-dim">Create your first virtual host configuration</p>
      <Button class="mt-4" @click="showCreateDialog = true">
        <Plus class="mr-2 h-4 w-4" />
        Add Virtual Host
      </Button>
    </div>

    <!-- Create Vhost Dialog -->
    <Dialog v-model:open="showCreateDialog">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Virtual Host</DialogTitle>
          <DialogDescription>
            Configure a new Nginx server block for your domain.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>Server Name</Label>
            <Input
              v-model="newVhost.serverName"
              placeholder="myproject.sig"
            />
          </div>

          <div class="space-y-2">
            <Label>Document Root</Label>
            <Input
              v-model="newVhost.documentRoot"
              placeholder="/var/www/html/public"
            />
          </div>

          <div class="flex items-center justify-between">
            <Label>Enable PHP</Label>
            <Switch v-model="newVhost.phpEnabled" />
          </div>

          <div class="flex items-center justify-between">
            <Label>Enable SSL</Label>
            <Switch v-model="newVhost.sslEnabled" />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showCreateDialog = false">
            Cancel
          </Button>
          <Button @click="createVhost" :disabled="!newVhost.serverName">
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Edit Vhost Dialog -->
    <Dialog v-model:open="showEditDialog">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Virtual Host</DialogTitle>
        </DialogHeader>

        <div v-if="selectedVhost" class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>Server Name</Label>
            <Input v-model="selectedVhost.server_name" />
          </div>

          <div class="space-y-2">
            <Label>Document Root</Label>
            <Input v-model="selectedVhost.document_root" />
          </div>

          <div class="flex items-center justify-between">
            <Label>Enable PHP</Label>
            <Switch v-model="selectedVhost.php_enabled" />
          </div>

          <div class="flex items-center justify-between">
            <Label>Enable SSL</Label>
            <Switch v-model="selectedVhost.ssl_enabled" />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showEditDialog = false">
            Cancel
          </Button>
          <Button @click="updateVhost">
            Save Changes
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Config Editor Dialog -->
    <Dialog v-model:open="showConfigDialog">
      <DialogContent class="max-w-3xl">
        <DialogHeader>
          <DialogTitle>{{ selectedVhost?.server_name }} - Configuration</DialogTitle>
        </DialogHeader>

        <Textarea
          v-model="configContent"
          class="h-96 font-mono text-sm"
          :rows="20"
        />

        <DialogFooter>
          <Button variant="outline" @click="showConfigDialog = false">
            Cancel
          </Button>
          <Button @click="saveConfig">
            Save Configuration
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
