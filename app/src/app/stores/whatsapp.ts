import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/**
 * Un número de WhatsApp tal como lo devuelve el núcleo.
 *
 * Coincide con `arles_app::whatsapp::NumeroIpc`. **No tiene el token ni la
 * referencia del llavero**, y no es un descuido de este tipo: el de Rust
 * tampoco los tiene, y hay una prueba allí que lo comprueba serializando.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LOS TRES `| null` SON SIGNIFICATIVOS Y NO SE PUEDEN COLAPSAR
 *
 * `coexistenciaConfirmada`, `riesgoAceptado` y `calidad` valen `null` cuando
 * **no constan**, que no es lo mismo que «no» ni que «está bien». La pantalla
 * los distingue: una marca sin poner se enseña como «sin confirmar», y una
 * calidad que no consta se enseña como «no consultada» — nunca en verde.
 */
export interface Numero {
  readonly id: string
  readonly nombreVisible: string
  readonly telefonoE164: string
  readonly wabaId: string
  /** `seguimiento` o `prospeccion_directa`. Se traduce con `whatsapp.modo.*`. */
  readonly modo: string
  /** `connected`, `needs_reauth`, `circuit_open` o `disabled`. */
  readonly estado: string
  readonly coexistenciaConfirmada: string | null
  readonly riesgoAceptado: string | null
  /** `green`, `yellow`, `red`, `unknown` — o `null` si no consta. */
  readonly calidad: string | null
  readonly limiteDeMeta: number | null
  /**
   * ¿Puede mandar una campaña hoy?
   *
   * **Viene resuelto de Rust y aquí no se recalcula.** La regla vive en el
   * núcleo; deducirla otra vez en la webview son dos reglas que se separan en
   * cuanto una cambie, y la que decide si se envía no sería la que se ve.
   */
  readonly puedeEnviar: boolean
  readonly escribeEnFrio: boolean
}

/** Lo que el formulario de alta manda. */
export interface BorradorDeNumero {
  nombreVisible: string
  telefono: string
  wabaId: string
  phoneNumberId: string
  modo: string
}

export function borradorDeNumeroVacio(): BorradorDeNumero {
  return {
    nombreVisible: '',
    telefono: '',
    wabaId: '',
    phoneNumberId: '',
    // El modo por defecto es el que **no** escribe en frío. Un desplegable que
    // empieza en prospección directa daría de alta números en frío a quien solo
    // pulsó «guardar», y la advertencia llegaría después del alta.
    modo: 'seguimiento',
  }
}

export const useWhatsappStore = defineStore('whatsapp', () => {
  const numeros = ref<readonly Numero[]>([])
  const cargando = ref(false)
  const error = ref<string | null>(null)

  /** Cuántos pueden enviar hoy. §94: cifras, no adjetivos. */
  const listos = computed(() => numeros.value.filter((n) => n.puedeEnviar).length)

  /**
   * Cuántos están dados de alta pero no pueden enviar todavía.
   *
   * Es la cifra que la pantalla necesita para explicar por qué un número que
   * se acaba de guardar no aparece disponible en el asistente.
   */
  const porConfirmar = computed(() => numeros.value.filter((n) => !n.puedeEnviar).length)

  async function cargar(): Promise<void> {
    if (!hayNucleo()) return
    cargando.value = true
    error.value = null
    try {
      numeros.value = await invocar<Numero[]>('numeros_de_whatsapp')
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
    } finally {
      cargando.value = false
    }
  }

  /**
   * Da de alta un número.
   *
   * El token viaja en la llamada y **no se guarda en ningún sitio de aquí**: ni
   * en la tienda, ni en `localStorage`. Vive en el campo mientras se escribe y
   * se borra al cerrar el formulario (ADR-0011).
   */
  async function guardar(borrador: BorradorDeNumero, token: string): Promise<boolean> {
    if (!hayNucleo()) return false
    cargando.value = true
    error.value = null
    try {
      numeros.value = await invocar<Numero[]>('guardar_numero_de_whatsapp', {
        nombreVisible: borrador.nombreVisible,
        telefono: borrador.telefono,
        wabaId: borrador.wabaId,
        phoneNumberId: borrador.phoneNumberId,
        modo: borrador.modo,
        token,
      })
      return true
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
      return false
    } finally {
      cargando.value = false
    }
  }

  async function confirmarCoexistencia(id: string): Promise<void> {
    await marcar('confirmar_coexistencia', id)
  }

  async function aceptarRiesgo(id: string): Promise<void> {
    await marcar('aceptar_riesgo_del_canal', id)
  }

  /** Las dos marcas se ponen igual; solo cambia el comando. */
  async function marcar(comando: string, id: string): Promise<void> {
    if (!hayNucleo()) return
    cargando.value = true
    error.value = null
    try {
      numeros.value = await invocar<Numero[]>(comando, { id })
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
    } finally {
      cargando.value = false
    }
  }

  return {
    numeros,
    cargando,
    error,
    listos,
    porConfirmar,
    cargar,
    guardar,
    confirmarCoexistencia,
    aceptarRiesgo,
  }
})
