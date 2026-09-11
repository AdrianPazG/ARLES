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
      // Los .woff2 de Mont viven en /TIPOGRAFIA, que el §11 declara material
      // de referencia de sólo lectura. Se leen desde ahí en vez de copiarlos:
      // una copia dentro de app/ sería un segundo sitio del que retirarlos si
      // P-01 se cierra en negativo, y ADR-0012 existe precisamente por eso.
      // Sólo entran en el bundle los cuatro que tipografia.css referencia.
      '@fuentes': fileURLToPath(new URL('../TIPOGRAFIA', import.meta.url)),
    },
  },

  // Tauri sirve la interfaz en un puerto fijo y necesita saber cuál.
  server: {
    port: 1420,
    strictPort: true,

    // El servidor de desarrollo sólo sirve archivos bajo la raíz del proyecto;
    // cualquier otro devuelve 403. Los .woff2 de Mont viven en /TIPOGRAFIA, un
    // nivel por encima, así que sin esto **la fuente no cargaba en desarrollo
    // y la interfaz caía en la reserva sin decir nada**: los 403 sólo se veían
    // abriendo la consola. En el build de producción no pasaba —Vite copia el
    // archivo al bundle—, así que el fallo sólo existía donde se trabaja.
    //
    // La lista es explícita y de sólo dos entradas: abrir la raíz entera
    // dejaría el servidor sirviendo cualquier archivo del repositorio, incluido
    // lo que no debe salir de él.
    fs: {
      allow: [
        fileURLToPath(new URL('.', import.meta.url)),
        fileURLToPath(new URL('../TIPOGRAFIA', import.meta.url)),
        fileURLToPath(new URL('../herramientas/design-tokens/dist', import.meta.url)),
      ],
    },
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
    // jsdom trae el elemento <dialog> pero no sus métodos. Ver el archivo.
    setupFiles: ['src/pruebas/entorno.ts'],
  },
})
