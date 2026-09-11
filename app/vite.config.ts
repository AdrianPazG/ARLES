import { fileURLToPath, URL } from 'node:url'

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [vue()],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      // El CSS de tokens se genera desde herramientas/design-tokens/tokens.json.
      // Se importa por alias para que nadie lo copie dentro de app/ y acabe
      // divergiendo de su fuente (§17).
      '@tokens': fileURLToPath(
        new URL('../herramientas/design-tokens/dist', import.meta.url),
      ),
    },
  },

  // Tauri sirve la interfaz en un puerto fijo y necesita saber cuál.
  server: {
    port: 1420,
    strictPort: true,
  },

  build: {
    // WebView2 (Chromium) y WKWebView (Safari) son ambos modernos: no hace
    // falta transpilar a ES5. Ver ADR-0001.
    target: ['es2022', 'safari15'],
    // Los sourcemaps de producción revelarían la estructura interna sin aportar
    // nada al usuario final.
    sourcemap: false,
    emptyOutDir: true,
  },

  test: {
    environment: 'jsdom',
    globals: true,
    include: ['src/**/*.spec.ts'],
  },
})
