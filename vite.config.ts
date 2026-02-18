import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  base: '/natal-chart-solver/',
  build: {
    outDir: 'build',
  },
  resolve: {
    alias: {
      '@wasm': path.resolve(__dirname, 'wasm/pkg')
    }
  },
  worker: {
    format: 'es'
  },
  optimizeDeps: {
    exclude: ['natal-chart-solver']
  }
})