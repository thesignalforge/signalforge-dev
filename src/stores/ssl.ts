import { ref } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface MkcertStatus {
  installed: boolean
  version: string | null
  ca_installed: boolean
  ca_path: string | null
}

export interface Certificate {
  domain: string
  cert_path: string
  key_path: string
  created_at: number
  is_wildcard: boolean
}

export const useSslStore = defineStore('ssl', () => {
  const mkcertStatus = ref<MkcertStatus | null>(null)
  const certificates = ref<Certificate[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function checkMkcertStatus() {
    try {
      loading.value = true
      error.value = null
      mkcertStatus.value = await invoke<MkcertStatus>('get_mkcert_status')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function installCA() {
    try {
      loading.value = true
      error.value = null
      const result = await invoke<string>('install_mkcert_ca')
      await checkMkcertStatus()
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function loadCertificates() {
    try {
      loading.value = true
      error.value = null
      certificates.value = await invoke<Certificate[]>('list_certificates')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function generateCertificate(domain: string, wildcard: boolean = false) {
    try {
      loading.value = true
      error.value = null
      const cert = await invoke<Certificate>('generate_certificate', { domain, wildcard })
      certificates.value.push(cert)
      return cert
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteCertificate(domain: string) {
    try {
      loading.value = true
      error.value = null
      await invoke('delete_certificate', { domain })
      certificates.value = certificates.value.filter(c => c.domain !== domain)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function getInstallInstructions(): Promise<string> {
    try {
      return await invoke<string>('get_mkcert_install_instructions')
    } catch (e) {
      error.value = String(e)
      return ''
    }
  }

  return {
    mkcertStatus,
    certificates,
    loading,
    error,
    checkMkcertStatus,
    installCA,
    loadCertificates,
    generateCertificate,
    deleteCertificate,
    getInstallInstructions
  }
})
