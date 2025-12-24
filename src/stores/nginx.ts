import { ref } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface NginxVhost {
  id: string
  server_name: string
  document_root: string
  php_enabled: boolean
  ssl_enabled: boolean
  ssl_cert_path: string | null
  ssl_key_path: string | null
  config_path: string
}

export interface NginxTestResult {
  success: boolean
  output: string
  errors: string[]
}

export const useNginxStore = defineStore('nginx', () => {
  const vhosts = ref<NginxVhost[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const testResult = ref<NginxTestResult | null>(null)

  async function loadVhosts() {
    try {
      loading.value = true
      error.value = null
      vhosts.value = await invoke<NginxVhost[]>('list_vhosts')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function createVhost(
    serverName: string,
    documentRoot: string,
    phpEnabled: boolean,
    sslEnabled: boolean,
    sslCertPath?: string,
    sslKeyPath?: string
  ) {
    try {
      loading.value = true
      error.value = null
      const vhost = await invoke<NginxVhost>('create_vhost', {
        serverName,
        documentRoot,
        phpEnabled,
        sslEnabled,
        sslCertPath: sslCertPath || null,
        sslKeyPath: sslKeyPath || null
      })
      vhosts.value.push(vhost)
      return vhost
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function updateVhost(vhost: NginxVhost) {
    try {
      loading.value = true
      error.value = null
      const updated = await invoke<NginxVhost>('update_vhost', { vhost })
      const idx = vhosts.value.findIndex(v => v.id === updated.id)
      if (idx !== -1) {
        vhosts.value[idx] = updated
      }
      return updated
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function deleteVhost(id: string) {
    try {
      loading.value = true
      error.value = null
      await invoke('delete_vhost', { id })
      vhosts.value = vhosts.value.filter(v => v.id !== id)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function getVhostConfig(id: string): Promise<string> {
    try {
      return await invoke<string>('get_vhost_config', { id })
    } catch (e) {
      error.value = String(e)
      return ''
    }
  }

  async function saveVhostConfig(id: string, content: string) {
    try {
      loading.value = true
      error.value = null
      await invoke('save_vhost_config', { id, content })
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function testConfig() {
    try {
      loading.value = true
      error.value = null
      testResult.value = await invoke<NginxTestResult>('test_nginx_config')
      return testResult.value
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function reloadNginx() {
    try {
      loading.value = true
      error.value = null
      return await invoke<string>('reload_nginx')
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function getDefaultConfig(): Promise<string> {
    try {
      return await invoke<string>('generate_default_nginx_config')
    } catch (e) {
      error.value = String(e)
      return ''
    }
  }

  return {
    vhosts,
    loading,
    error,
    testResult,
    loadVhosts,
    createVhost,
    updateVhost,
    deleteVhost,
    getVhostConfig,
    saveVhostConfig,
    testConfig,
    reloadNginx,
    getDefaultConfig
  }
})
