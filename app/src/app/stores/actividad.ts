import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/**
 * Lo que salió en una hora. Coincide con `RitmoDeHoraIpc`.
 */
export interface RitmoDeHora {
  /** La hora, como `2026-09-30T10`. */
  readonly hora: string
  readonly salieron: number
}

/**
 * Por qué el bucle de una campaña no está sacando nada.
 *
 * Coincide con `RazonIpc`. Es **el último suceso de bucle**, no un diagnóstico
 * calculado aquí: el núcleo ya sabía las seis razones del silencio desde la
 * Fase 4 y las tiraba todas.
 */
export interface Razon {
  /** Clave de i18n (§139). El texto vive en el catálogo. */
  readonly clave: string
  readonly numeroA: number | null
  readonly numeroB: number | null
  readonly cuando: string
}

/** Cómo va una campaña. Coincide con `CifrasDeCampanaIpc`. */
export interface CifrasDeCampana {
  readonly id: string
  readonly nombre: string
  /** Clave del rótulo de estado (`campana.estado.*`). */
  readonly estado: string
  readonly audiencia: number
  /** Cuántos **aceptó el proveedor**. Nunca «entregados» (§65). */
  readonly salio: number
  readonly pendiente: number
  readonly fallo: number
  readonly dadoDeBaja: number
  readonly cancelado: number
  /** Cuánta gente contestó (D-7A). **Nunca lo que dijeron.** */
  readonly respondieron: number
  readonly ritmo: readonly RitmoDeHora[]
  readonly razon: Razon | null
}

/** El cupo de un remitente. Coincide con `CupoIpc`. */
export interface Cupo {
  readonly nombre: string
  readonly direccion: string
  readonly gastadoHoy: number
  readonly topeDiario: number | null
  readonly quedanHoy: number | null
  readonly gastadoEstaHora: number
  readonly topePorHora: number | null
  /** Las campañas en marcha que tiran de esta cuenta, por nombre. */
  readonly campanas: readonly string[]
  readonly hayReparto: boolean
}

/** Lo que devuelve el comando `actividad`. Coincide con `ActividadIpc`. */
export interface Actividad {
  readonly campanas: readonly CifrasDeCampana[]
  readonly totalSalio: number
  readonly totalPendiente: number
  readonly totalFallo: number
  readonly remitentes: readonly Cupo[]
}

/**
 * Cada cuántos milisegundos se vuelve a preguntar.
 *
 * ── POR QUÉ CINCO SEGUNDOS Y NO UNO ──
 *
 * Porque es el ritmo del motor: `Trabajador` da una vuelta cada cinco segundos,
 * así que preguntar más a menudo devuelve **las mismas cifras** y sólo añade
 * consultas. Y porque una cifra que cambia cada segundo delante de alguien que
 * la está leyendo no se puede leer.
 */
export const CADA_CUANTO_MS = 5_000

/**
 * A partir de cuántos segundos sin refrescar los datos dejan de ser «de ahora».
 *
 * Tres vueltas perdidas. Con una sola, cualquier consulta lenta marcaría la
 * pantalla como vieja y el aviso saldría y se iría; con tres, cuando aparece es
 * porque de verdad hace rato que nadie contesta.
 */
export const VIEJO_TRAS_S = 15

export const useActividadStore = defineStore('actividad', () => {
  const campanas = ref<readonly CifrasDeCampana[]>([])
  const remitentes = ref<readonly Cupo[]>([])
  const totalSalio = ref(0)
  const totalPendiente = ref(0)
  const totalFallo = ref(0)

  const cargada = ref(false)
  const cargando = ref(false)
  const errorGeneral = ref<string | null>(null)

  /**
   * Cuándo llegaron estas cifras. `null` mientras no haya llegado ninguna.
   *
   * ── POR QUÉ LA PANTALLA TIENE QUE ENSEÑARLO ──
   *
   * Porque si el refresco falla, lo que queda en la pantalla son las cifras de
   * hace un rato, **y siguen pareciendo las de ahora**. Eso es exactamente el
   * fallo que el §94 y el §95 quieren impedir: alguien decide que una campaña
   * está atascada mirando un número congelado. La marca, y el aviso de
   * `estaViejo`, son lo que impide creerse una pantalla muerta.
   */
  const actualizadoA = ref<Date | null>(null)
  /** `true` si el último intento de refresco falló. Lo anterior sigue visible. */
  const refrescoFallido = ref(false)
  /** Segundos desde el último dato bueno. Lo mueve `latir()`. */
  const antiguedadS = ref(0)

  const hayCampanas = computed(() => campanas.value.length > 0)

  /** Cuántas están enviando ahora mismo (§94, cifras y no adjetivos). */
  const enMarcha = computed(
    () =>
      campanas.value.filter((c) => c.estado === 'campana.estado.enviando')
        .length,
  )

  /**
   * Las que están enviando, primero; y dentro de cada grupo, la que más tiene
   * pendiente.
   *
   * Con tres campañas a la vez (D-7A) el orden no puede ser el de la base: lo
   * que hace falta arriba es lo que está pasando ahora, no lo que se creó antes.
   */
  const ordenadas = computed(() =>
    [...campanas.value].sort((a, b) => {
      const av = a.estado === 'campana.estado.enviando' ? 0 : 1
      const bv = b.estado === 'campana.estado.enviando' ? 0 : 1
      return av !== bv ? av - bv : b.pendiente - a.pendiente
    }),
  )

  /** `true` si hace demasiado que no llega un dato bueno. */
  const estaViejo = computed(() => antiguedadS.value >= VIEJO_TRAS_S)

  /**
   * Los remitentes cuyo cupo se reparte entre varias campañas.
   *
   * Es lo que explica por qué dos campañas están quietas cuando una tercera se
   * comió el cupo. Sin esto, «quedan 20 de 500» no dice de quién.
   */
  const conReparto = computed(() =>
    remitentes.value.filter((r) => r.hayReparto),
  )

  /** Lo que consta de un resumen guardado. Coincide con `ResumenGuardado`. */
  const ultimoResumen = ref<string | null>(null)

  /**
   * Guarda el resumen de una campaña en un archivo.
   *
   * ── LA RUTA NO CRUZA, Y ESO ES LA REGLA 4 ──
   *
   * El diálogo lo abre el núcleo y lo único que vuelve es **el nombre** del
   * archivo, nunca la carpeta: dónde guarda alguien sus cosas dice dónde vive,
   * y a esta pantalla no le aporta nada.
   *
   * `null` si se cerró el diálogo sin guardar. Cancelar no es un fallo.
   */
  async function exportar(id: string): Promise<boolean> {
    if (!hayNucleo()) return false
    errorGeneral.value = null
    try {
      const r = await invocar<{ nombreDelArchivo: string } | null>('exportar_resumen', {
        campana: id,
      })
      ultimoResumen.value = r === null ? null : r.nombreDelArchivo
      return r !== null
    } catch (e) {
      recordarError(e)
      return false
    }
  }

  function recordarError(e: unknown): void {
    errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
  }

  function guardar(a: Actividad): void {
    campanas.value = a.campanas
    remitentes.value = a.remitentes
    totalSalio.value = a.totalSalio
    totalPendiente.value = a.totalPendiente
    totalFallo.value = a.totalFallo
    actualizadoA.value = new Date()
    antiguedadS.value = 0
    refrescoFallido.value = false
  }

  /**
   * La primera carga. Enseña el estado de carga y borra lo que hubiera.
   */
  async function cargar(): Promise<void> {
    if (cargando.value) return
    cargando.value = true
    errorGeneral.value = null
    try {
      if (hayNucleo()) {
        guardar(await invocar<Actividad>('actividad'))
      }
      cargada.value = true
    } catch (e) {
      recordarError(e)
    } finally {
      cargando.value = false
    }
  }

  /**
   * Un refresco, que **no borra lo que ya hay si falla**.
   *
   * ── POR QUÉ NO ES `cargar()` OTRA VEZ ──
   *
   * Porque `cargar` enseña «cargando…» y limpia el error, y hacerlo cada cinco
   * segundos deja la pantalla parpadeando encima de alguien que está leyendo.
   * Y porque un fallo pasajero de la base no es motivo para vaciar la pantalla:
   * se marca como viejo y lo de antes sigue ahí, con su hora.
   */
  async function refrescar(): Promise<void> {
    if (!hayNucleo() || cargando.value) return
    try {
      guardar(await invocar<Actividad>('actividad'))
    } catch (e) {
      refrescoFallido.value = true
      recordarError(e)
    }
  }

  /**
   * Recalcula la antigüedad. La llama el reloj de la pantalla.
   *
   * Separado del refresco a propósito: si el refresco deja de contestar, esto
   * sigue corriendo y el número de «hace n segundos» sigue subiendo. Al
   * contrario —derivar la antigüedad del propio refresco— el aviso de datos
   * viejos no aparecería nunca, porque lo dispararía justo lo que ha dejado de
   * ocurrir.
   */
  function latir(): void {
    const t = actualizadoA.value
    antiguedadS.value =
      t === null ? 0 : Math.floor((Date.now() - t.getTime()) / 1000)
  }

  return {
    campanas,
    remitentes,
    totalSalio,
    totalPendiente,
    totalFallo,
    cargada,
    cargando,
    errorGeneral,
    actualizadoA,
    refrescoFallido,
    antiguedadS,
    hayCampanas,
    enMarcha,
    ordenadas,
    estaViejo,
    conReparto,
    cargar,
    refrescar,
    latir,
    exportar,
    ultimoResumen,
  }
})
