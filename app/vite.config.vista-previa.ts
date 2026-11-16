/**
 * Configuración de la **vista previa en un solo archivo**.
 *
 * Hereda todo de `vite.config.ts` y sólo cambia lo que impide que el resultado
 * se abra con doble clic desde el disco:
 *
 *  · `inlineDynamicImports`: el enrutador carga cada pantalla con un `import()`
 *    dinámico, que en un build normal produce un archivo por pantalla. Desde
 *    `file://` esos archivos no se pueden cargar —el navegador lo prohíbe— y la
 *    aplicación se queda en blanco sin decir nada. Comprobado: la primera
 *    versión de la vista previa hacía exactamente eso.
 *  · `assetsInlineLimit` enorme: las cuatro tipografías y las dos máscaras del
 *    logotipo entran como `data:` en vez de quedarse al lado.
 *  · `cssCodeSplit: false`: una sola hoja de estilos.
 *  · `base: './'`: ninguna ruta absoluta, que desde el disco apuntaría a la
 *    raíz del sistema de archivos.
 *
 * No se usa para la aplicación de verdad: ahí el troceado es lo correcto.
 */
import { mergeConfig } from 'vite'

import base from './vite.config'

export default mergeConfig(base, {
  base: './',
  build: {
    cssCodeSplit: false,
    assetsInlineLimit: 100_000_000,
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
    },
  },
})
