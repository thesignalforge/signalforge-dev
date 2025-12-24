import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './style.css'

import Dashboard from './views/Dashboard.vue'
import Services from './views/Services.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'dashboard', component: Dashboard },
    { path: '/services', name: 'services', component: Services },
    { path: '/projects', name: 'projects', component: () => import('./views/Projects.vue') },
    { path: '/logs', name: 'logs', component: () => import('./views/Logs.vue') },
    { path: '/config', name: 'config', component: () => import('./views/Config.vue') },
    { path: '/system', name: 'system', component: () => import('./views/System.vue') }
  ]
})

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.mount('#app')
