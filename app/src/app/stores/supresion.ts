import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { claveDelError, hayNucleo, invocar } from '@/app/nucleo'

/** Coincide con `arles_core::MotivoDeSupresion`. */
export type MotivoDeSupresion =
  | 'hard_bounce'
  | 'unsubscribe'
  | 'manual'
  | 'complaint'
  | 'invalid_number'
  | 'channel_failure'

/**
 * Los motivos que una persona de la empresa puede elegir a mano.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * NO ESTÁN LOS SEIS, Y ES DELIBERADO
 *
 * `hard_bounce`, `invalid_number` y `channel_failure` los **deduce el sistema**
 * al enviar: son el resultado de una conversación con un servidor, no una
 * decisión de nadie. Ofrecerlos en un desplegable dejaría a alguien marcar
 * «rebote permanente» sin que haya habido ningún rebote, y el registro dejaría
 * de servir para lo único que sirve: explicar, dentro de un año, por qué esa
 * persona no recibe.
 *
 * Quedan los tres que **sí** decide una persona. Y de esos, dos —`unsubscribe`
 * y `complaint`— no se pueden deshacer después con un botón: el núcleo lo
 * decide, no esta lista.
 * ─────────────────────────────────────────────────────────────────────────
 */
export const MOTIVOS_A_MANO: readonly MotivoDeSupresion[] = [
  'unsubscribe',
  'complaint',
  'manual',
]

/** Coincide con `arles_core::Canal`. */
export type Canal = 'email' | 'whatsapp'

/** Una fila de la lista, tal y como llega de Rust. */
export interface SupresionParaLaPantalla {
  readonly id: string
  readonly canal: Canal
  readonly direccion: string
  /** Clave de i18n del motivo. */
  readonly motivo: string
  /**
   * **Ya decidido por el núcleo.** La pantalla no vuelve a razonarlo: si
   * tuviera su propia copia de la regla, el día que divergieran enseñaría un
   * botón que Rust rechaza, o escondería uno que sí valía.
   */
  readonly sePuedeLevantar: boolean
  /** Nombre del contacto vivo con esa dirección, o cadena vacía. */
  readonly deQuien: string
  readonly notas: string
  readonly creada: string
  /** `true` si la baja fue para todos los canales de esa persona. */
  readonly global: boolean
}

export interface PaginaDeSupresiones {
  readonly supresiones: readonly SupresionParaLaPantalla[]
  readonly total: number
}

export interface ResumenDeSupresion {
  readonly nuevas: number
  readonly yaEstaban: number
}

/** Tiene que coincidir con `SUPRESIONES_POR_PAGINA` de los comandos. */
export const POR_PAGINA = 50

/**
 * La lista de quién pidió no recibir.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTA PANTALLA EXISTE
 *
 * Dirección la pidió el 21/09/2026 con estas palabras: «tenemos que tener una
 * lista de los prospectos que seleccionaron la cancelación de correos o
 * WhatsApp».
 *
 * Y hace falta por dos razones distintas. Una es operativa: saber a cuánta
 * gente de una lista ya no se le puede escribir. La otra es que **es la prueba**
 * de que esas bajas se respetaron, con su fecha y su motivo, el día que alguien
 * pregunte.
 * ─────────────────────────────────────────────────────────────────────────
 */
export const useSupresionStore = defineStore('supresion', () => {
  const supresiones = ref<readonly SupresionParaLaPantalla[]>([])
  const total = ref(0)
  const desde = ref(0)
  const cargando = ref(false)
  const errorGeneral = ref<string | null>(null)

  /** El texto que se escribe en la caja de buscar. */
  const filtro = ref('')

  /**
   * Las filas que se enseñan, filtradas **en memoria**.
   *
   * Sobre la página cargada, no sobre la lista entera: filtrar en la base
   * sería otro comando y otra decisión sobre qué significa «buscar» en una
   * lista de direcciones normalizadas. Mientras la lista quepa en unas pocas
   * páginas esto basta, y **la pantalla dice sobre qué está filtrando** para
   * que nadie crea que ha buscado en las mil doscientas.
   */
  const visibles = computed(() => {
    const t = filtro.value.trim().toLowerCase()
    if (t === '') return supresiones.value
    return supresiones.value.filter(
      (s) =>
        s.direccion.toLowerCase().includes(t) ||
        s.deQuien.toLowerCase().includes(t),
    )
  })

  /** Cuántas de las de esta página son de correo. Cifras, no adjetivos (§94). */
  const cuantasDeCorreo = computed(
    () => supresiones.value.filter((s) => s.canal === 'email').length,
  )

  const cuantasDeWhatsApp = computed(
    () => supresiones.value.filter((s) => s.canal === 'whatsapp').length,
  )

  const hayMas = computed(() => desde.value + supresiones.value.length < total.value)

  function fallo(e: unknown): void {
    errorGeneral.value = claveDelError(e)
  }

  /** Carga una página. `desde` cuenta filas, no páginas. */
  async function cargar(inicio = 0): Promise<void> {
    cargando.value = true
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return
      }
      const pagina = await invocar<PaginaDeSupresiones>('listar_supresiones', {
        desde: inicio,
      })
      supresiones.value = pagina.supresiones
      total.value = pagina.total
      desde.value = inicio
    } catch (e) {
      fallo(e)
    } finally {
      cargando.value = false
    }
  }

  /**
   * Da de baja a un contacto entero.
   *
   * **No se le mandan las direcciones**: sólo su identificador. Las lee Rust de
   * la base, porque entre que esta pantalla cargó la ficha y alguien pulsó el
   * botón pudo añadirse un canal — y darlo de baja «todo» con una lista vieja
   * dejaría ese canal abierto.
   */
  async function suprimirContacto(
    id: string,
    motivo: MotivoDeSupresion,
    notas: string,
  ): Promise<ResumenDeSupresion | null> {
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return null
      }
      const resumen = await invocar<ResumenDeSupresion>('suprimir_contacto', {
        id,
        motivo,
        notas,
      })
      await cargar(desde.value)
      return resumen
    } catch (e) {
      fallo(e)
      return null
    }
  }

  /**
   * Vuelve a permitir una dirección.
   *
   * Se pide sin comprobar antes si se puede: **lo decide Rust**, leyendo el
   * motivo guardado. Comprobarlo también aquí sería una segunda copia de la
   * regla, y dos copias divergen. El botón se esconde cuando
   * `sePuedeLevantar` es `false`, pero eso es cortesía, no la defensa.
   */
  async function levantar(id: string): Promise<boolean> {
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      await invocar<void>('levantar_supresion', { id })
      await cargar(desde.value)
      return true
    } catch (e) {
      fallo(e)
      return false
    }
  }

  return {
    supresiones,
    total,
    desde,
    cargando,
    errorGeneral,
    filtro,
    visibles,
    cuantasDeCorreo,
    cuantasDeWhatsApp,
    hayMas,
    cargar,
    suprimirContacto,
    levantar,
  }
})
