/**
 * Variables de entorno de compilación.
 *
 * Declararlas aquí no es ceremonia de tipos: es lo que permite escribirlas
 * con **notación de punto**, y eso decide si funcionan.
 *
 * Vite sustituye `import.meta.env.LO_QUE_SEA` por su valor en tiempo de
 * compilación, y después Rollup elimina la rama muerta. Con **corchetes**
 * —`import.meta.env['LO_QUE_SEA']`— no sustituye nada: la expresión queda
 * dinámica, la rama sobrevive, y todo lo que cuelga de ella entra en el
 * bundle.
 *
 * Se descubrió así: al pasar el interruptor del catálogo de `import.meta.env.DEV`
 * a una variable escrita con corchetes, **el catálogo empezó a compilarse
 * dentro del bundle de producción** como un trozo cargado bajo demanda. Lo
 * atrapó la comprobación «el catálogo no entra en el bundle de producción»,
 * que mira el bundle compilado y no el texto del router.
 */
interface ImportMetaEnv {
  /**
   * `'1'` incluye el catálogo del design system en la compilación.
   *
   * Lo usan la sonda de CSP y la compilación de revisión visual —la que se
   * abre en Windows y en macOS—. **Nunca una compilación de producción.**
   */
  readonly VITE_ARLES_CATALOGO?: string

  /**
   * Umbral de plegado automático de la barra lateral, en píxeles.
   *
   * **Sólo la sonda que mide el umbral lo pone**, a 1, para que la barra no se
   * pliegue nunca sola y se pueda recorrer el eje de anchos con ella
   * desplegada — sin el interruptor, el propio umbral impide llegar a los
   * anchos donde se mediría. En cualquier otra compilación no existe, y vale el número
   * medido de `stores/interfaz.ts`.
   */
  readonly VITE_ARLES_UMBRAL_PLEGADO?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
