import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import { setupDesktopShell } from './services/desktop'

createApp(App).mount('#app')

setupDesktopShell().catch((error) => {
  console.warn('Failed to initialize Tauri desktop shell:', error)
})
