import { defineConfig } from 'vite'
import { fileURLToPath, URL } from 'node:url'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

// In production the Go binary serves the embedded build; in development Vite
// serves the SPA with HMR and proxies API + websocket traffic to the Go backend
// (ADR 0010), so the browser only ever speaks to one origin. Point at a
// different backend with CHARTR_BACKEND when the chartr runs on another port.
const backend = process.env.CHARTR_BACKEND ?? 'http://127.0.0.1:8787'

export default defineConfig({
  // Tailwind v4 is CSS-first: the plugin reads @theme/@import from app.css, no
  // tailwind.config.js. The $lib alias is what shadcn-svelte's imports assume;
  // this app is not SvelteKit, so it is declared here rather than inferred.
  plugins: [tailwindcss(), svelte()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
    },
  },
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
