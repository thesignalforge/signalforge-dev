<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useTldStore, type SigDomain, type DnsTestResult } from '@/stores/tld'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
  Link,
  Plus,
  Trash2,
  CheckCircle,
  XCircle,
  Globe,
  Play,
  Info
} from 'lucide-vue-next'

const tldStore = useTldStore()

const showAddDialog = ref(false)
const showInstructionsDialog = ref(false)
const installInstructions = ref('')
const testResults = ref<Map<string, DnsTestResult>>(new Map())

const newDomain = ref({
  name: '',
  ipAddress: '127.0.0.1'
})

onMounted(async () => {
  await tldStore.checkDnsmasqStatus()
  await tldStore.loadDomains()
})

async function configureTld() {
  try {
    await tldStore.configureSigTld()
  } catch (e) {
    console.error('Failed to configure TLD:', e)
  }
}

async function addDomain() {
  if (!newDomain.value.name) return

  try {
    await tldStore.addDomain(newDomain.value.name, newDomain.value.ipAddress)
    showAddDialog.value = false
    newDomain.value = { name: '', ipAddress: '127.0.0.1' }
  } catch (e) {
    console.error('Failed to add domain:', e)
  }
}

async function removeDomain(name: string) {
  if (!confirm(`Remove ${name}.sig?`)) return

  try {
    await tldStore.removeDomain(name)
    testResults.value.delete(name)
  } catch (e) {
    console.error('Failed to remove domain:', e)
  }
}

async function testDomain(domain: SigDomain) {
  try {
    const result = await tldStore.testDomainResolution(domain.full_domain)
    testResults.value.set(domain.name, result)
  } catch (e) {
    console.error('Failed to test domain:', e)
  }
}

async function showInstallInstructions() {
  installInstructions.value = await tldStore.getInstallInstructions()
  showInstructionsDialog.value = true
}

function getTestResult(name: string): DnsTestResult | undefined {
  return testResults.value.get(name)
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-text">.sig TLD Manager</h1>
        <p class="text-sm text-text-dim">Configure local .sig domains for development</p>
      </div>
      <Button @click="showAddDialog = true">
        <Plus class="mr-2 h-4 w-4" />
        Add Domain
      </Button>
    </div>

    <!-- dnsmasq Status -->
    <Card :class="tldStore.dnsmasqStatus?.sig_configured ? 'border-green-500/30' : 'border-yellow-500/30'">
      <CardContent class="flex items-center justify-between py-4">
        <div class="flex items-center gap-3">
          <div
            :class="[
              'flex h-10 w-10 items-center justify-center rounded-lg',
              tldStore.dnsmasqStatus?.installed ? 'bg-green-500/10' : 'bg-yellow-500/10'
            ]"
          >
            <component
              :is="tldStore.dnsmasqStatus?.installed ? CheckCircle : XCircle"
              :class="tldStore.dnsmasqStatus?.installed ? 'text-green-400' : 'text-yellow-400'"
              class="h-5 w-5"
            />
          </div>
          <div>
            <p class="font-medium text-text">
              dnsmasq {{ tldStore.dnsmasqStatus?.installed ? 'Installed' : 'Not Installed' }}
            </p>
            <p class="text-xs text-text-dim">
              {{ tldStore.dnsmasqStatus?.running ? 'Running' : 'Not running' }}
              {{ tldStore.dnsmasqStatus?.sig_configured ? '• .sig configured' : '' }}
            </p>
          </div>
        </div>

        <div class="flex gap-2">
          <Button
            v-if="!tldStore.dnsmasqStatus?.installed"
            variant="outline"
            @click="showInstallInstructions"
          >
            <Info class="mr-2 h-4 w-4" />
            Install Instructions
          </Button>
          <Button
            v-else-if="!tldStore.dnsmasqStatus?.sig_configured"
            @click="configureTld"
          >
            Configure .sig TLD
          </Button>
          <Badge
            v-else
            variant="success"
            class="flex items-center gap-1"
          >
            <CheckCircle class="h-3 w-3" />
            Configured
          </Badge>
        </div>
      </CardContent>
    </Card>

    <!-- Info Card -->
    <Card class="border-cyan/30 bg-cyan/5">
      <CardContent class="flex items-start gap-3 py-4">
        <Globe class="h-5 w-5 text-cyan mt-0.5" />
        <div class="text-sm">
          <p class="text-text">
            With dnsmasq configured, all <code class="rounded bg-bg-deep px-1 text-cyan">*.sig</code> domains
            automatically resolve to <code class="rounded bg-bg-deep px-1">127.0.0.1</code>.
          </p>
          <p class="mt-1 text-text-dim">
            Add individual domains below for tracking, or they'll work automatically.
          </p>
        </div>
      </CardContent>
    </Card>

    <!-- Domains List -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Link class="h-5 w-5 text-cyan" />
          Registered .sig Domains
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="tldStore.domains.length > 0" class="space-y-3">
          <div
            v-for="domain in tldStore.domains"
            :key="domain.name"
            class="flex items-center justify-between rounded-lg border border-border bg-bg-deep p-4"
          >
            <div class="flex items-center gap-3">
              <Globe class="h-5 w-5 text-cyan" />
              <div>
                <div class="flex items-center gap-2">
                  <span class="font-mono text-text">{{ domain.full_domain }}</span>
                  <Badge v-if="domain.in_hosts" variant="secondary" class="text-xs">
                    /etc/hosts
                  </Badge>
                </div>
                <p class="text-xs text-text-dim">{{ domain.ip_address }}</p>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <!-- Test Result -->
              <div v-if="getTestResult(domain.name)" class="flex items-center gap-1 text-xs">
                <component
                  :is="getTestResult(domain.name)?.resolves ? CheckCircle : XCircle"
                  :class="getTestResult(domain.name)?.resolves ? 'text-green-400' : 'text-red-400'"
                  class="h-4 w-4"
                />
                <span :class="getTestResult(domain.name)?.resolves ? 'text-green-400' : 'text-red-400'">
                  {{ getTestResult(domain.name)?.resolves ? 'Resolves' : 'Failed' }}
                </span>
              </div>

              <Button size="sm" variant="ghost" @click="testDomain(domain)">
                <Play class="h-4 w-4" />
              </Button>
              <Button size="sm" variant="ghost" @click="removeDomain(domain.name)">
                <Trash2 class="h-4 w-4 text-red-400" />
              </Button>
            </div>
          </div>
        </div>

        <div v-else class="py-8 text-center text-text-dim">
          <Link class="mx-auto mb-2 h-8 w-8 opacity-50" />
          <p>No .sig domains registered</p>
          <p class="text-xs mt-1">Domains will still work via dnsmasq wildcard</p>
        </div>
      </CardContent>
    </Card>

    <!-- Add Domain Dialog -->
    <Dialog v-model:open="showAddDialog">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add .sig Domain</DialogTitle>
          <DialogDescription>
            Register a new .sig domain for local development.
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>Domain Name</Label>
            <div class="flex items-center gap-2">
              <Input
                v-model="newDomain.name"
                placeholder="myproject"
                class="flex-1"
              />
              <span class="font-mono text-cyan">.sig</span>
            </div>
          </div>

          <div class="space-y-2">
            <Label>IP Address</Label>
            <Input
              v-model="newDomain.ipAddress"
              placeholder="127.0.0.1"
            />
            <p class="text-xs text-text-dim">
              Usually 127.0.0.1 for local development
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showAddDialog = false">
            Cancel
          </Button>
          <Button @click="addDomain" :disabled="!newDomain.name">
            Add Domain
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Install Instructions Dialog -->
    <Dialog v-model:open="showInstructionsDialog">
      <DialogContent class="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Install dnsmasq</DialogTitle>
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
