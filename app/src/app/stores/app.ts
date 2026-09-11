import { defineStore } from 'pinia'
import { ref } from 'vue'

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
  const cargada = ref(false)

  async function cargar(): Promise<void> {
    if (cargada.value) return
    if (!('__TAURI_INTERNALS__' in (window as unknown as ConTauri))) {
      cargada.value = true
      return
    }
    const { invoke } = await import('@tauri-apps/api/core')
    info.value = await invoke<InfoApp>('info_app')
    cargada.value = true
  }

  return { info, cargada, cargar }
})
