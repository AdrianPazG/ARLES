/**
 * De una razón del motor al texto que lee el usuario.
 *
 * ═══════════════════════════════════════════════════════════════════════════
 * POR QUÉ ESTO ES UN MÓDULO Y NO TRES LÍNEAS DENTRO DE LA PANTALLA
 *
 * Porque es el sitio exacto donde se rompió el pegamento, y una función suelta
 * dentro de un `.vue` no se puede probar sin montar el componente — y montar el
 * componente arrastra el router, que es la trampa 6 de la Fase 6.
 *
 * El suceso llega con una clave de i18n (§139) y hasta dos cifras sin nombre:
 * `numeroA` y `numeroB`. El texto del catálogo tiene marcadores CON nombre:
 * «Se alcanzó el tope de {cabian} envíos por hora». Alguien tiene que decir
 * que `numeroA` es `cabian`, y ese alguien es esta tabla.
 *
 * Lo que puede salir mal, y ya salió:
 *
 *   1. El motor no guarda la cifra que el texto pide → la pantalla enseña
 *      `{cabian}` tal cual. Lo caza `toda_razon_trae_las_cifras_de_su_texto`
 *      en Rust, que encontró dos claves así antes de que existiera la pantalla.
 *   2. Esta tabla nombra un marcador que el texto no tiene → el texto sale
 *      correcto pero la cifra se pierde en silencio, que es peor porque no se
 *      ve. Lo caza `cada_razon_declara_los_marcadores_de_su_texto` en
 *      `razones.spec.ts`.
 *   3. La cifra viaja en otra unidad que la del texto: el motor guarda
 *      **segundos** y el texto dice «{minutos} minutos». Por eso la conversión
 *      está declarada aquí y no escondida en una plantilla.
 */

import type { Razon } from '@/app/stores/actividad'

/**
 * Los marcadores del texto de cada razón, en el orden de `numeroA`, `numeroB`.
 *
 * Una razón es un **suceso de bucle**: por qué el motor no está sacando nada.
 * Los desenlaces de un envío —aceptado, reintento, baja— no salen por aquí:
 * `por_que_esta_quieta_cada_campana` los excluye, porque «se aceptó un correo»
 * no explica por qué ahora no sale ninguno.
 *
 * Una clave que no esté aquí se pinta sin cifras. Eso es correcto para las que
 * no tienen ninguna, y lo comprueba la prueba: si un texto tiene marcadores y
 * la clave no está en esta tabla, falla.
 */
export const MARCADORES_DE_RAZON: Readonly<Record<string, readonly string[]>> =
  {
    'motor.parado': [],
    'motor.fueraDeVentana': [],
    'motor.noHayTrabajo': [],
    'motor.laCuentaEstaEnPausa': ['minutos'],
    'ritmo.fueraDeVentana': [],
    'ritmo.topeDeLaHora': ['cabian'],
    'ritmo.topeDelDia': ['cabian'],
  }

/**
 * Las claves cuya cifra llega en segundos y su texto habla de minutos.
 *
 * Está declarado y no aplicado a ojo porque una conversión metida en la
 * plantilla es invisible: el día que el texto pase a decir «{segundos}», nadie
 * encontraría el `/ 60`. Aquí se ve, y la prueba de la unidad lo mira.
 */
export const RAZONES_EN_SEGUNDOS: readonly string[] = [
  'motor.laCuentaEstaEnPausa',
]

/**
 * Las cifras de una razón, con el nombre que espera su texto.
 *
 * Se redondea **hacia arriba** al convertir a minutos: con 30 segundos
 * restantes, «vuelve en 0 minutos» es falso —todavía no vuelve— y «en 1 minuto»
 * es verdad como cota. Un cero aquí se leería como «ya debería estar
 * enviando», que es justo la confusión que esta pantalla existe para evitar.
 */
export function cifrasDeRazon(razon: Razon): Record<string, number> {
  const nombres = MARCADORES_DE_RAZON[razon.clave] ?? []
  const valores = [razon.numeroA, razon.numeroB]
  const enSegundos = RAZONES_EN_SEGUNDOS.includes(razon.clave)

  const cifras: Record<string, number> = {}
  nombres.forEach((nombre, i) => {
    const v = valores[i]
    if (v === null || v === undefined) return
    cifras[nombre] = enSegundos ? Math.ceil(v / 60) : v
  })
  return cifras
}
