<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSslStore } from '@/stores/ssl'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter
} from '@/components/ui/dialog'
import {
  ShieldCheck,
  Plus,
  Trash2,
  CheckCircle,
  XCircle,
  Lock,
  Info
} from 'lucide-vue-next'

const sslStore = useSslStore()

const showGenerateDialog = ref(false)
const showInstructionsDialog = ref(false)
const installInstructions = ref('')

const newCert = ref({
  domain: '',
  wildcard: false
})

onMounted(async () => {
  await sslStore.checkMkcertStatus()
  await sslStore.loadCertificates()
})

async function installCA() {
  try {
    await sslStore.installCA()
  } catch (e) {
    console.error('Failed to install CA:', e)
  }
}

async function generateCertificate() {
  if (!newCert.value.domain) return

  try {
    await sslStore.generateCertificate(newCert.value.domain, newCert.value.wildcard)
    showGenerateDialog.value = false
    newCert.value = { domain: '', wildcard: false }
  } catch (e) {
    console.error('Failed to generate certificate:', e)
  }
}

async function deleteCertificate(domain: string) {
  if (!confirm(`Delete certificate for ${domain}?`)) return

  try {
    await sslStore.deleteCertificate(domain)
  } catch (e) {
    console.error('Failed to delete certificate:', e)
  }
}

async function showInstallInstructions() {
  installInstructions.value = await sslStore.getInstallInstructions()
  showInstructionsDialog.value = true
}

function formatDate(timestamp: number) {
  return new Date(timestamp * 1000).toLocaleDateString()
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-text">SSL Certificates</h1>
        <p class="text-sm text-text-dim">Manage SSL certificates with mkcert</p>
      </div>
      <Button @click="showGenerateDialog = true" :disabled="!sslStore.mkcertStatus?.installed">
        <Plus class="mr-2 h-4 w-4" />
        Generate Certificate
      </Button>
    </div>

    <!-- mkcert Status -->
    <Card :class="sslStore.mkcertStatus?.installed ? 'border-green-500/30' : 'border-yellow-500/30'">
      <CardContent class="flex items-center justify-between py-4">
        <div class="flex items-center gap-3">
          <div
            :class="[
              'flex h-10 w-10 items-center justify-center rounded-lg',
              sslStore.mkcertStatus?.installed ? 'bg-green-500/10' : 'bg-yellow-500/10'
            ]"
          >
            <component
              :is="sslStore.mkcertStatus?.installed ? CheckCircle : XCircle"
              :class="sslStore.mkcertStatus?.installed ? 'text-green-400' : 'text-yellow-400'"
              class="h-5 w-5"
            />
          </div>
          <div>
            <p class="font-medium text-text">
              mkcert {{ sslStore.mkcertStatus?.installed ? 'Installed' : 'Not Installed' }}
            </p>
            <p v-if="sslStore.mkcertStatus?.version" class="text-xs text-text-dim">
              Version: {{ sslStore.mkcertStatus.version }}
            </p>
          </div>
        </div>

        <div class="flex gap-2">
          <Button
            v-if="!sslStore.mkcertStatus?.installed"
            variant="outline"
            @click="showInstallInstructions"
          >
            <Info class="mr-2 h-4 w-4" />
            Install Instructions
          </Button>
          <Button
            v-else-if="!sslStore.mkcertStatus?.ca_installed"
            @click="installCA"
          >
            Install CA
          </Button>
          <Badge
            v-else
            variant="success"
            class="flex items-center gap-1"
          >
            <CheckCircle class="h-3 w-3" />
            CA Installed
          </Badge>
        </div>
      </CardContent>
    </Card>

    <!-- Certificates List -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Lock class="h-5 w-5 text-cyan" />
          SSL Certificates
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="sslStore.certificates.length > 0" class="space-y-3">
          <div
            v-for="cert in sslStore.certificates"
            :key="cert.domain"
            class="flex items-center justify-between rounded-lg border border-border bg-bg-deep p-4"
          >
            <div class="flex items-center gap-3">
              <ShieldCheck class="h-5 w-5 text-cyan" />
              <div>
                <div class="flex items-center gap-2">
                  <span class="font-mono text-text">{{ cert.domain }}</span>
                  <Badge v-if="cert.is_wildcard" variant="secondary" class="text-xs">
                    Wildcard
                  </Badge>
                </div>
                <p class="text-xs text-text-dim">Created: {{ formatDate(cert.created_at) }}</p>
              </div>
            </div>

            <div class="flex items-center gap-4">
              <div class="text-right text-xs text-text-dim">
                <p class="font-mono">{{ cert.cert_path.split('/').pop() }}</p>
                <p class="font-mono">{{ cert.key_path.split('/').pop() }}</p>
              </div>
              <Button size="sm" variant="ghost" @click="deleteCertificate(cert.domain)">
                <Trash2 class="h-4 w-4 text-red-400" />
              </Button>
            </div>
          </div>
        </div>

        <div v-else class="py-8 text-center text-text-dim">
          <ShieldCheck class="mx-auto mb-2 h-8 w-8 opacity-50" />
          <p>No certificates generated yet</p>
        </div>
      </CardContent>
    </Card>

    <!-- Generate Certificate Dialog -->
    <Dialog v-model:open="showGenerateDialog">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Generate SSL Certificate</DialogTitle>
          <DialogDescription>
            Create a new SSL certificate for local development.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>Domain Name</Label>
            <Input
              v-model="newCert.domain"
              placeholder="myproject.sig"
            />
            <p class="text-xs text-text-dim">
              Use .sig extension for Signalforge domains
            </p>
          </div>

          <div class="flex items-center justify-between">
            <div>
              <Label>Wildcard Certificate</Label>
              <p class="text-xs text-text-dim">Include *.{{ newCert.domain || 'domain' }}</p>
            </div>
            <Switch v-model="newCert.wildcard" />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showGenerateDialog = false">
            Cancel
          </Button>
          <Button @click="generateCertificate" :disabled="!newCert.domain">
            Generate
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Install Instructions Dialog -->
    <Dialog v-model:open="showInstructionsDialog">
      <DialogContent class="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Install mkcert</DialogTitle>
        </DialogHeader>

        <ScrollArea class="h-96">
          <div class="prose prose-invert prose-sm p-4" v-html="installInstructions.replace(/```(\w*)\n([\s\S]*?)```/g, '<pre class=\'bg-bg-deep p-3 rounded text-xs overflow-x-auto\'><code>$2</code></pre>').replace(/\n/g, '<br>')"></div>
        </ScrollArea>

        <DialogFooter>
          <Button @click="showInstructionsDialog = false">
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
