import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'
import type { LoQueTraeElRespaldo } from '@/app/stores/restauracion'

/** Qué pasó con la clave, con cifras que se pueden comprobar. */
export interface QuePaso {
  readonly bytesDeLaBase: number
  readonly nombreDeLaBase: string
}

/** Lo que quedó después de recuperar. */
export interface Recuperado {
  readonly filas: number
  readonly bajas: number
  readonly cuentasAReconectar: number
  readonly nombreDeLaApartada: string
}

/**
 * El modo de recuperación: cuando ARLES arranca sin poder abrir su base.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * AQUÍ NO HAY CASI NADA, Y ES A PROPÓSITO
 *
 * El núcleo registra **tres comandos** en este modo y ninguno más: no hay base
 * que consultar. Este almacén no pide contactos ni campañas porque no existen
 * todavía, y pedirlos daría errores que no significan nada.
 */
export const usarRecuperacion = defineStore('recuperacion', () => {
  const quePaso = ref<QuePaso | null>(null)
  const revisado = ref<LoQueTraeElRespaldo | null>(null)
  const contrasena = ref('')
  const revisando = ref(false)
  const recuperando = ref(false)
  const hecho = ref<Recuperado | null>(null)
  const error = ref<string | null>(null)

  const faltaLaContrasena = computed(
    () => revisado.value?.llevaContrasena === true && contrasena.value === '',
  )

  const sePuedeRecuperar = computed(
    () =>
      revisado.value !== null &&
      revisado.value.sePuedeRestaurar &&
      !faltaLaContrasena.value &&
      !recuperando.value,
  )

  /** Qué pasó, para poder decirlo. */
  async function cargar(): Promise<void> {
    if (!hayNucleo()) return
    try {
      quePaso.value = await invocar<QuePaso>('que_paso_con_la_clave')
    } catch {
      // Si ni esto se puede leer, la pantalla sigue siendo útil: dice lo que
      // pasó y ofrece la salida. Lo único que se pierde es el tamaño del
      // archivo, y quedarse sin salida por no poder leer unos metadatos sería
      // el mismo fallo que esta entrega vino a arreglar.
      quePaso.value = null
    }
  }

  async function revisar(): Promise<void> {
    if (!hayNucleo()) return
    revisando.value = true
    error.value = null
    try {
      const r = await invocar<LoQueTraeElRespaldo | null>('revisar_respaldo_para_recuperar')
      if (r !== null) {
        revisado.value = r
        contrasena.value = ''
      }
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
      revisado.value = null
    } finally {
      revisando.value = false
    }
  }

  async function recuperar(): Promise<void> {
    if (!hayNucleo() || !sePuedeRecuperar.value) return
    recuperando.value = true
    error.value = null
    try {
      hecho.value = await invocar<Recuperado>('recuperar_desde_respaldo', {
        contrasena: contrasena.value === '' ? null : contrasena.value,
      })
      revisado.value = null
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
      revisado.value = null
    } finally {
      recuperando.value = false
      contrasena.value = ''
    }
  }

  return {
    quePaso,
    revisado,
    contrasena,
    revisando,
    recuperando,
    hecho,
    error,
    faltaLaContrasena,
    sePuedeRecuperar,
    cargar,
    revisar,
    recuperar,
  }
})
