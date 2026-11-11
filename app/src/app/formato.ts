/**
 * Cómo se enseña un dato que ARLES guarda normalizado.
 *
 * ── POR QUÉ EXISTE ─────────────────────────────────────────────────────────
 *
 * Un número de WhatsApp se guarda en E.164 —`+528112345678`, sin un solo
 * espacio— porque así se compara, se deduplica y se suprime. Pero eso es la
 * forma de guardarlo, no la de leerlo: doce dígitos seguidos no se pueden
 * cotejar de un vistazo con el número que alguien tiene en el teléfono, y
 * cotejarlo es justo lo que se hace en Remitentes antes de mandar nada.
 *
 * Medido el 28/10/2026 sobre las dieciocho pantallas: **cinco apariciones en
 * tres pantallas** —Remitentes › WhatsApp, Plantillas y Bajas— enseñaban el
 * número crudo. Contactos no, pero no porque lo formateara: enseña
 * `valorRaw`, que es **lo que el usuario tecleó**. Así que ARLES no tenía una
 * forma de presentar un número: tenía dos orígenes distintos.
 *
 * Es D-02 de `documentacion/10-requisitos/HALLAZGOS-DE-DISENO.md`.
 */

/** Un número de México en E.164: `+52` y diez dígitos nacionales. */
const MEXICO = /^\+52(\d{10})$/

/** Cualquier E.164: `+` y de 8 a 15 dígitos. */
const E164 = /^\+(\d{8,15})$/

/**
 * Agrupa un número guardado para que se pueda leer.
 *
 * `+528112345678` → `+52 81 1234 5678`
 *
 * ── LO QUE ESTO NO ES ──────────────────────────────────────────────────────
 *
 * **No es un formateador de números internacionales.** Eso necesita la tabla
 * de planes de numeración de cada país y es una biblioteca entera; aquí lo que
 * hace falta es que el número **se pueda leer**, no que respete la convención
 * tipográfica de su país.
 *
 * Por eso hay exactamente dos reglas:
 *
 * 1. **México tiene la suya**, porque es el país del producto y su convención
 *    se conoce: `+52 81 1234 5678` (lada de dos y dos bloques de cuatro).
 * 2. **Cualquier otro** se parte en bloques de cuatro **desde la derecha**,
 *    así el último bloque siempre está completo. Es una ayuda de lectura y
 *    está dicho que lo es.
 *
 * **Lo que no entiende, lo devuelve intacto.** Un correo, un número a medio
 * escribir o una cadena vacía salen como entraron: inventarle una forma a algo
 * que no se reconoce es peor que dejarlo como está — y la lista de bajas pinta
 * con esta función tanto correos como números.
 */
export function numeroLegible(valor: string): string {
  const limpio = valor.trim()

  const mx = MEXICO.exec(limpio)
  if (mx?.[1]) {
    const n = mx[1]
    return `+52 ${n.slice(0, 2)} ${n.slice(2, 6)} ${n.slice(6)}`
  }

  const otro = E164.exec(limpio)
  if (otro?.[1]) {
    const n = otro[1]
    const bloques: string[] = []
    for (let fin = n.length; fin > 0; fin -= 4) {
      bloques.unshift(n.slice(Math.max(0, fin - 4), fin))
    }
    return `+${bloques.join(' ')}`
  }

  return limpio
}

/**
 * «hace 4 minutos», «hace 2 horas», «hace 3 días». Vacío si no se puede leer.
 *
 * ── POR QUÉ `numeric: 'always'` Y NO `'auto'` ──────────────────────────────
 *
 * Con `'auto'`, `Intl` escribe «hace un momento» y «ayer» en vez de «hace 1
 * minuto» y «hace 1 día». Eso es un adjetivo donde el §94 pide una cifra, y en
 * el sitio donde esto se usa —cuándo se revisó un buzón por última vez— la
 * cifra es lo único que permite saber si el automatismo va: «hace un momento»
 * se lee igual con un intervalo de quince minutos que con uno de seis horas.
 *
 * ── Y POR QUÉ EL SUELO ES UN MINUTO ───────────────────────────────────────
 *
 * Un intento de hace diez segundos diría «hace 10 segundos», y ese número
 * cambiaría cada vez que se recargara la pantalla sin que nada hubiera pasado.
 * Lo útil es el orden de magnitud, no el segundero.
 *
 * ── LO QUE NO HACE ────────────────────────────────────────────────────────
 *
 * **No lleva reloj.** Devuelve la cifra del instante en que se la llama, así
 * que en una pantalla abierta durante una hora el texto envejece. Donde se usa
 * hoy eso está bien —el dato se carga al abrir la pantalla— y donde no lo
 * estuviera, quien lo use tendrá que latir, como hace Actividad desde la 7.2.
 *
 * El formato lo pone `Intl` y no el catálogo de textos porque es lo que ya hace
 * el resto del proyecto con `toLocaleString('es-MX')`: una tabla propia de
 * plurales y unidades sería una segunda verdad sobre el español.
 */
export function haceCuanto(cuando: string | null, ahora: number = Date.now()): string {
  if (cuando === null || cuando.trim() === '') return ''
  const fecha = new Date(cuando)
  if (Number.isNaN(fecha.getTime())) return ''

  const segundos = Math.floor((ahora - fecha.getTime()) / 1000)
  const formato = new Intl.RelativeTimeFormat('es-MX', { numeric: 'always' })

  // De mayor a menor; se para en la primera unidad que da al menos 1. Una
  // fecha en el futuro —el reloj del equipo se movió— cae en la rama de los
  // minutos con el suelo de 1, así que dice «hace 1 minuto» en vez de
  // «dentro de 3 horas»: raro, pero no es una cifra que mande a hacer nada.
  if (segundos >= 86_400) return formato.format(-Math.floor(segundos / 86_400), 'day')
  if (segundos >= 3_600) return formato.format(-Math.floor(segundos / 3_600), 'hour')
  return formato.format(-Math.max(1, Math.floor(segundos / 60)), 'minute')
}
