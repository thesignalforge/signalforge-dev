import { ref } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface DnsmasqStatus {
  installed: boolean
  running: boolean
  config_path: string | null
  sig_configured: boolean
}

export interface SigDomain {
  name: string
  full_domain: string
  ip_address: string
  in_hosts: boolean
  in_dnsmasq: boolean
}

export interface DnsTestResult {
  domain: string
  resolves: boolean
  ip_address: string | null
  method: string
}

export const useTldStore = defineStore('tld', () => {
  const dnsmasqStatus = ref<DnsmasqStatus | null>(null)
  const domains = ref<SigDomain[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function checkDnsmasqStatus() {
    try {
      loading.value = true
      error.value = null
      dnsmasqStatus.value = await invoke<DnsmasqStatus>('get_dnsmasq_status')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function configureSigTld() {
    try {
      loading.value = true
      error.value = null
      const result = await invoke<string>('configure_sig_tld')
      await checkDnsmasqStatus()
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function loadDomains() {
    try {
      loading.value = true
      error.value = null
      domains.value = await invoke<SigDomain[]>('list_sig_domains')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addDomain(name: string, ipAddress?: string) {
    try {
      loading.value = true
      error.value = null
      const domain = await invoke<SigDomain>('add_sig_domain', {
        name,
        ipAddress: ipAddress || null
      })
      domains.value.push(domain)
      return domain
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function removeDomain(name: string) {
    try {
      loading.value = true
      error.value = null
      await invoke('remove_sig_domain', { name })
      domains.value = domains.value.filter(d => d.name !== name && d.full_domain !== name)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function testDomainResolution(domain: string): Promise<DnsTestResult> {
    try {
      return await invoke<DnsTestResult>('test_domain_resolution', { domain })
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function getInstallInstructions(): Promise<string> {
    try {
      return await invoke<string>('get_dnsmasq_install_instructions')
    } catch (e) {
      error.value = String(e)
      return ''
    }
  }

  return {
    dnsmasqStatus,
    domains,
    loading,
    error,
    checkDnsmasqStatus,
    configureSigTld,
    loadDomains,
    addDomain,
    removeDomain,
    testDomainResolution,
    getInstallInstructions
  }
})
