import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { hayNucleo, invocar } from '@/app/nucleo'

/**
 * Ancho de viewport por debajo del cual la barra lateral se pliega sola.
 *
 * **Medido, no elegido a ojo**, con `npm --prefix app run sonda:plegado`. Es el
 * ancho por debajo del cual la pantalla más exigente —la lista de alta, 78 ch—
 * deja de caber a su medida de diseño con la barra desplegada: 240 px de barra
 * + 64 px de márgenes + 679 px de medida = 983, y el barrido lo confirmó en
 * 988. Plegar devuelve 176 px (240 → 64), justo lo que hace falta.
 *
 * El número se remidió al rediseñar Inicio como panel: la pantalla dejó de
 * declarar un ancho de lectura y pasó a declarar el ancho por debajo del cual
 * la rejilla áurea se apila —684 px—, cuatro más de lo que pedía la lista de
 * alta anterior. Sigue saliendo de `sonda:plegado`, no de una estimación.
 *
 * **No es un umbral de desbordamiento**, y conviene saberlo: la sonda buscó
 * primero dónde desborda y no encontró nada hasta 600 px, porque estas
 * pantallas son fluidas. Ese resultado se documentó en vez de disfrazarse de
 * número redondo. Ver `documentacion/06-calidad/UMBRAL_DE_PLEGADO.md`.
 *
 * El número **cambia cuando cambie la pantalla más ancha**, y la 3.2 trae la
 * tabla de contactos. La sonda lo detectará: falla igual si el umbral se queda
 * corto que si se infla.
 */
export const UMBRAL_MEDIDO_PX = 988

/**
 * El umbral en uso.
 *
 * `VITE_ARLES_UMBRAL_PLEGADO` **sólo lo usa la sonda que mide**: poniéndolo a 1
 * la barra no se pliega nunca sola, y entonces se puede recorrer el eje de
 * anchos con la barra desplegada para encontrar dónde deja de caber la medida de
 * diseño. Sin ese interruptor no hay forma de medir el umbral, porque el propio
 * umbral impide llegar a los anchos donde se mediría.
 *
 * En una compilación normal la variable no existe, Vite sustituye la expresión
 * por `undefined`, `Number(undefined)` es `NaN` y queda el valor medido. Con
 * notación de punto, que es la única que Vite sustituye (ver `router.ts`).
 */
export const UMBRAL_DE_PLEGADO_PX =
  Number(import.meta.env.VITE_ARLES_UMBRAL_PLEGADO) || UMBRAL_MEDIDO_PX

/**
 * Estado de la interfaz que sobrevive al cierre de la aplicación.
 *
 * P-11, decidido por Dirección el 15 de septiembre de 2026:
 *
 * | Pregunta | Decisión |
 * |---|---|
 * | ¿Cómo se pliega? | A mano **y** sola |
 * | ¿Qué queda plegada? | Iconos sin texto |
 * | ¿Se recuerda al reabrir? | Sí |
 *
 * Las dos formas de plegar no compiten: `plegada` es la unión. Lo que el
 * usuario eligió **no se pisa** cuando la ventana se estrecha — si lo
 * hiciéramos, al volver a agrandarla habría perdido su preferencia sin haber
 * tocado nada.
 */
export const useInterfazStore = defineStore('interfaz', () => {
  /** Lo que el usuario eligió la última vez. Es lo que se recuerda. */
  const preferenciaPlegada = ref(false)
  const anchoVentana = ref(typeof window === 'undefined' ? 0 : window.innerWidth)
  const cargada = ref(false)

  /** La ventana no da para la barra desplegada sin apretar el contenido. */
  const estrecha = computed(() => anchoVentana.value < UMBRAL_DE_PLEGADO_PX)

  /**
   * ¿Está plegada ahora mismo?
   *
   * A mano **o** sola. Con la ventana estrecha se pliega aunque el usuario la
   * quisiera abierta: ahí la barra se está comiendo 240 px que el contenido
   * necesita para leerse a su medida.
   */
  const plegada = computed(() => estrecha.value || preferenciaPlegada.value)

  /**
   * ¿Puede el usuario cambiarla ahora?
   *
   * Con la ventana estrecha, no: el botón queda deshabilitado y explica por
   * qué. La alternativa —dejarlo pulsable sin efecto visible— es peor: un
   * control que no hace nada al pulsarlo se lee como una avería.
   */
  const alternableAhora = computed(() => !estrecha.value)

  async function cargar(): Promise<void> {
    if (cargada.value) return
    cargada.value = true
    if (!hayNucleo()) return
    try {
      const p = await invocar<{ barraLateralPlegada: boolean }>(
        'preferencias_de_interfaz',
      )
      preferenciaPlegada.value = p.barraLateralPlegada
    } catch {
      // Una preferencia de interfaz que no se puede leer **no** impide usar la
      // aplicación: se queda desplegada, que es el valor por defecto.
    }
  }

  async function alternar(): Promise<void> {
    if (!alternableAhora.value) return
    preferenciaPlegada.value = !preferenciaPlegada.value
    if (!hayNucleo()) return
    try {
      await invocar('guardar_barra_plegada', { plegada: preferenciaPlegada.value })
    } catch {
      // Igual que arriba: la barra ya se plegó en pantalla. Que no se recuerde
      // al reabrir es peor que nada, pero mucho menos que un error modal por
      // una casilla plegada.
    }
  }

  /** La ventana cambió de tamaño. Lo llama el armazón. */
  function anotarAncho(ancho: number): void {
    anchoVentana.value = ancho
  }

  return {
    preferenciaPlegada,
    anchoVentana,
    cargada,
    estrecha,
    plegada,
    alternableAhora,
    cargar,
    alternar,
    anotarAncho,
  }
})
