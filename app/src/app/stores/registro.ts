import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/** Una fila del registro. Coincide con `SucesoIpc`. */
export interface Suceso {
  readonly id: string
  /** Clave de i18n (§139). La pantalla la traduce; aquí no viaja texto. */
  readonly clave: string
  /** `true` si es el desenlace de un envío. **No se borra nunca** (D-7C). */
  readonly esDesenlace: boolean
  readonly campana: string | null
  readonly canal: string | null
  /** A quién se le escribió. `null` en los sucesos de bucle. */
  readonly direccion: string | null
  readonly numeroA: number | null
  readonly numeroB: number | null
  readonly cuando: string
}

/** Una clase de suceso con su cifra, para el desplegable de «qué pasó». */
export interface Clase {
  readonly clave: string
  readonly cuantos: number
}

/** Lo que devuelve el comando `registro`. Coincide con `RegistroIpc`. */
export interface Registro {
  readonly sucesos: readonly Suceso[]
  readonly total: number
  readonly porPagina: number
  readonly clases: readonly Clase[]
}

/** Lo que pasaría al recortar. Coincide con `RecorteIpc`. */
export interface Recorte {
  readonly seIrian: number
  readonly antesDe: string
  readonly seQuedan: number
}

/**
 * Los cuatro criterios de D-7C.
 *
 * Cadenas y no tipos ricos porque **viajan en la dirección** (`?campana=…`):
 * el plan pide que el filtro se refleje ahí para poder volver a un estado
 * concreto, y lo que vuelve de una URL siempre es texto.
 */
export interface Filtro {
  campana: string
  clave: string
  desde: string
  hasta: string
  texto: string
  /** `''` es «todo»; `'si'` sólo envíos; `'no'` sólo sucesos del motor. */
  soloDesenlaces: string
}

export function filtroVacio(): Filtro {
  return { campana: '', clave: '', desde: '', hasta: '', texto: '', soloDesenlaces: '' }
}

export const useRegistroStore = defineStore('registro', () => {
  const sucesos = ref<readonly Suceso[]>([])
  const clases = ref<readonly Clase[]>([])
  const total = ref(0)
  const porPagina = ref(500)

  const cargada = ref(false)
  const cargando = ref(false)
  const trabajando = ref(false)
  const errorGeneral = ref<string | null>(null)

  const filtro = ref<Filtro>(filtroVacio())
  const recorte = ref<Recorte | null>(null)

  const hayFiltro = computed(() => Object.values(filtro.value).some((v) => v !== ''))

  /**
   * `true` si hay más de los que caben en pantalla.
   *
   * Es lo que decide si se enseña «500 de 3 412» o sólo «17 registros». Decir
   * «500 de 500» cuando hay justo 500 haría pensar que falta algo.
   */
  const hayMasDeLosQueCaben = computed(() => total.value > sucesos.value.length)

  function recordarError(e: unknown): void {
    errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
  }

  /** Lo que se manda al núcleo: las cadenas vacías se quedan aquí. */
  function paraElNucleo(): Record<string, unknown> {
    const f = filtro.value
    return {
      campana: f.campana || null,
      clave: f.clave || null,
      desde: f.desde || null,
      hasta: f.hasta || null,
      texto: f.texto || null,
      // Tres estados y no un booleano: «todo» no es ni sí ni no, y con un
      // booleano habría que elegir cuál de los dos significa «no filtrar».
      soloDesenlaces: f.soloDesenlaces === '' ? null : f.soloDesenlaces === 'si',
    }
  }

  async function cargar(): Promise<void> {
    if (cargando.value) return
    cargando.value = true
    errorGeneral.value = null
    try {
      if (hayNucleo()) {
        const r = await invocar<Registro>('registro', { filtro: paraElNucleo() })
        sucesos.value = r.sucesos
        clases.value = r.clases
        total.value = r.total
        porPagina.value = r.porPagina
      }
      cargada.value = true
    } catch (e) {
      recordarError(e)
    } finally {
      cargando.value = false
    }
  }

  /** Cambia el filtro y vuelve a pedir. */
  async function filtrar(nuevo: Partial<Filtro>): Promise<void> {
    filtro.value = { ...filtro.value, ...nuevo }
    await cargar()
  }

  async function limpiar(): Promise<void> {
    filtro.value = filtroVacio()
    await cargar()
  }

  /** Qué se recortaría. **No borra nada**: es la pregunta, no la acción. */
  async function verElRecorte(): Promise<void> {
    if (!hayNucleo()) return
    errorGeneral.value = null
    try {
      recorte.value = await invocar<Recorte>('que_se_recortaria')
    } catch (e) {
      recordarError(e)
    }
  }

  /**
   * Recorta de verdad, y refresca la lista.
   *
   * Los desenlaces de envío no se tocan: lo garantiza `arles-db`, no esta
   * función. Aquí sólo se pide.
   */
  async function recortar(): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      recorte.value = await invocar<Recorte>('recortar_registro')
      await cargar()
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  return {
    sucesos,
    clases,
    total,
    porPagina,
    cargada,
    cargando,
    trabajando,
    errorGeneral,
    filtro,
    recorte,
    hayFiltro,
    hayMasDeLosQueCaben,
    cargar,
    filtrar,
    limpiar,
    verElRecorte,
    recortar,
  }
})
