/**
 * Las pantallas que existen, leídas del router · 07/10/2026
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTE ARCHIVO EXISTE, Y QUÉ ENCONTRÓ EL DÍA QUE SE ESCRIBIÓ
 *
 * `auditoria.mjs` y `recorrido.mjs` llevaban **cada una su lista de rutas
 * escrita a mano**, y las dos listas se habían quedado en las trece pantallas
 * que existían el 29/09/2026. El producto tiene dieciocho.
 *
 * Lo que eso significa, dicho sin rodeos: **cinco pantallas nunca se habían
 * medido.** Ni su contraste, ni su foco, ni sus desbordes, ni la jerarquía de
 * sus encabezados, ni sus identificadores repetidos. Y las dos sondas
 * terminaban en verde diciendo «18 pantallas medidas» — no, diciendo «13», que
 * era peor, porque el número estaba ahí y nadie lo comparó con nada.
 *
 * Y el aviso estaba escrito. La propia `auditoria.mjs` lleva desde el
 * 29/09/2026 este comentario, puesto el día que se le añadieron tres rutas que
 * faltaban:
 *
 *   «Una auditoría que no recorre una pantalla no dice nada de esa pantalla,
 *    aunque termine en verde — que es peor que no tenerla, porque se cree.»
 *
 * Se escribió la lección y **volvió a pasar cinco veces**, porque la lección
 * iba en un comentario y no en el código. Una lista a mano no se mantiene por
 * saber que hay que mantenerla.
 *
 * Es el mismo defecto que la auditoría A encontró en el grafo de crates y la C
 * en los casos frontera: la comprobación decía cubrir una regla y cubría una
 * lista. **La única defensa que ha funcionado las tres veces es leer del
 * código.**
 *
 * ─────────────────────────────────────────────────────────────────────────
 * SE LEE CON UNA EXPRESIÓN, Y ESO TIENE UN RIESGO QUE HAY QUE CERRAR
 *
 * Un patrón que deja de casar devuelve **cero rutas**, y una sonda que recorre
 * cero pantallas termina en verde en un segundo. Es la trampa del §2 en su
 * forma más barata, y este proyecto ya la ha tenido cinco veces.
 *
 * Por eso `leerRutas()` **revienta** si encuentra menos de `MINIMO`, en vez de
 * devolver lo que haya. Un fallo ruidoso en una herramienta es un rato
 * perdido; uno silencioso son meses de auditorías que no miraban nada.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const AQUI = path.dirname(fileURLToPath(import.meta.url))
const ROUTER = path.resolve(AQUI, '../../src/app/router.ts')

/**
 * Las rutas que NO son pantallas del producto, con su motivo.
 *
 * Cada una tiene que estar justificada aquí: es la única forma de que quitar
 * una pantalla de la auditoría sea una decisión visible y no un olvido.
 */
const NO_SON_PANTALLAS = {
  '/': 'es una redirección a /inicio, no una pantalla',
  '/:pathMatch(.*)*': 'es el 404',
  '/catalogo': 'es el catálogo del design system, una herramienta nuestra',
}

/**
 * Cuántas pantallas tiene que haber como mínimo.
 *
 * Son 18 el 07/10/2026. El mínimo va por debajo a propósito —quitar una
 * pantalla es legítimo— pero lo bastante alto como para que un patrón roto no
 * pase: con 13 estaríamos otra vez donde estábamos.
 */
const MINIMO = 16

/**
 * Las rutas de pantalla del router, en su orden de declaración.
 *
 * @returns {string[]} rutas como `/actividad/registro`.
 */
export function leerRutas() {
  const fuente = fs.readFileSync(ROUTER, 'utf8')
  const todas = [...fuente.matchAll(/^\s*path:\s*'([^']+)'/gm)].map((m) => m[1])

  if (todas.length === 0) {
    throw new Error(
      `rutas.mjs: no se encontró ninguna ruta en ${ROUTER}. El patrón dejó de ` +
        'casar; una sonda que recorre cero pantallas pasa en verde sin medir nada.',
    )
  }

  const pantallas = todas.filter((r) => !(r in NO_SON_PANTALLAS))
  if (pantallas.length < MINIMO) {
    throw new Error(
      `rutas.mjs: salieron ${pantallas.length} pantallas y el mínimo es ${MINIMO}. ` +
        'O el patrón se rompió, o se quitaron pantallas sin bajar el mínimo. ' +
        `Encontradas: ${pantallas.join(', ')}`,
    )
  }
  return pantallas
}

/** Las mismas, con el `#` que usa la vista previa. */
export function leerRutasConAlmohadilla() {
  return leerRutas().map((r) => `#${r}`)
}
