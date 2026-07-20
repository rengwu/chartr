import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// The star-map island's seam is the one frontend test point (spec, Testing
// Decisions). A jsdom environment gives the island a DOM to mount into; it runs
// headless (no 2D context), which is exactly what the seam tests need — layout
// determinism, zero star movement, and selection emission, none of which touch
// the canvas.
export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    setupFiles: ['src/lib/starmap/test-setup.ts'],
  },
})
