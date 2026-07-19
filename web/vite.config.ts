import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// In production the Go binary serves the embedded build; in development Vite
// serves the SPA with HMR and proxies API + websocket traffic to the Go backend
// (ADR 0010), so the browser only ever speaks to one origin. Point at a
// different backend with HARNESS_BACKEND when the harness runs on another port.
const backend = process.env.HARNESS_BACKEND ?? 'http://127.0.0.1:8787'

export default defineConfig({
  plugins: [svelte()],
  build: {
    // Output lands in web/dist, which the Go `web` package embeds.
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      '/api': { target: backend, changeOrigin: true },
      // ws:true is what forwards the control and terminal sockets to Go.
      '/ws': { target: backend, ws: true, changeOrigin: true },
    },
  },
})
