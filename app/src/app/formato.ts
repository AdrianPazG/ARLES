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
