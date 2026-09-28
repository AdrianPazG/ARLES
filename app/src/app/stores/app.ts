import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

/**
 * Identidad de la aplicación, tal como la reporta el núcleo.
 *
 * Coincide con `arles_app::comandos::InfoApp`.
 */
export interface InfoApp {
  /** Nombre visible, sin numeral: «ARLES RELAY». */
  readonly nombre: string
  /** Nombre comercial, con numeral: «ARLES RELAY I». */
  readonly nombreComercial: string
  readonly version: string
  readonly atribucion: string
}

/** Valor de reserva para `vite dev`, donde no hay comandos de Tauri. */
const SIN_NUCLEO: InfoApp = {
  nombre: 'ARLES RELAY',
  nombreComercial: 'ARLES RELAY I',
  version: '—',
  atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
}

/**
 * Estado de salud del arranque.
 *
 * Coincide con `arles_app::comandos::EstadoArranque`.
 */
export interface EstadoArranque {
  readonly baseDeDatosLista: boolean
  readonly versionEsquema: number | null
  readonly empresaConfigurada: boolean
  /**
   * El motor está enviando.
   *
   * `false` significa que hay otro ARLES abierto sobre la misma base: esta
   * ventana deja mirar y configurar, y **no manda ningún correo**.
   */
  readonly elMotorEnvia: boolean
}

/**
 * Valor de reserva fuera de Tauri.
 *
 * `elMotorEnvia` en `true` **a propósito**, y es la decisión menos obvia de
 * este archivo: en `vite dev` y en las pruebas no hay núcleo, y un `false` de
 * reserva pintaría el aviso de «esta ventana no envía» en una pantalla donde no
 * significa nada. Un aviso que aparece siempre deja de leerse, y el día que
 * apareciera de verdad nadie lo miraría.
 */
const ARRANQUE_SIN_NUCLEO: EstadoArranque = {
  baseDeDatosLista: false,
  versionEsquema: null,
  empresaConfigurada: false,
  elMotorEnvia: true,
}

interface ConTauri {
  __TAURI_INTERNALS__?: unknown
}

/**
 * La versión la dice el núcleo, no la interfaz.
 *
 * Estaba escrita a mano en el pie. Habría quedado obsoleta en la siguiente
 * subida de versión sin que nada lo detectara, y el número de versión es
 * justamente lo que alguien mira cuando reporta un problema.
 *
 * El guion como reserva es deliberado: es visible. Un `1.2.0` de reserva se
 * confundiría con el valor real y volvería a esconder el problema.
 */
export const useAppStore = defineStore('app', () => {
  const info = ref<InfoApp>(SIN_NUCLEO)
  const arranque = ref<EstadoArranque>(ARRANQUE_SIN_NUCLEO)
  const cargada = ref(false)

  async function cargar(): Promise<void> {
    if (cargada.value) return
    if (!('__TAURI_INTERNALS__' in (window as unknown as ConTauri))) {
      cargada.value = true
      return
    }
    const { invoke } = await import('@tauri-apps/api/core')
    info.value = await invoke<InfoApp>('info_app')
    arranque.value = await invoke<EstadoArranque>('estado_arranque')
    cargada.value = true
  }

  /**
   * Esta ventana no está enviando: hay otro ARLES abierto sobre la misma base.
   *
   * Se expone con nombre propio en vez de dejar que la pantalla escriba
   * `!arranque.elMotorEnvia`: una negación suelta en una plantilla se lee al
   * revés con facilidad, y equivocarse aquí pinta un aviso permanente.
   *
   * **Lo que impide que parpadee al abrir es el valor de reserva**, no esto.
   * La primera versión llevaba además un `cargada.value &&` delante, con un
   * comentario que decía que era lo que evitaba el parpadeo. Al romperlo a
   * propósito no falló ninguna prueba: con `elMotorEnvia` en `true` mientras no
   * hay respuesta, no hay instante en que la negación sea cierta. Era código
   * redundante sostenido por una explicación equivocada, y se quitó.
   */
  const estaVentanaNoEnvia = computed(() => !arranque.value.elMotorEnvia)

  return { info, arranque, cargada, estaVentanaNoEnvia, cargar }
})
