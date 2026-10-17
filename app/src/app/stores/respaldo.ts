import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { escuchar, claveDelError, hayNucleo, invocar } from '@/app/nucleo'

/** Cuántas filas trae una tabla. Coincide con `FilasDeTabla`. */
export interface FilasDeTabla {
  readonly tabla: string
  readonly filas: number
}

/** Lo que llevaría el respaldo, **antes** de escribir nada. */
export interface LoQueLlevaria {
  readonly esquema: string
  readonly tablas: readonly FilasDeTabla[]
  readonly total: number
  readonly mesesEnElEsencial: number
}

/** Lo que se escribió. **Nunca la ruta** (regla 4 de la frontera). */
export interface RespaldoEscrito {
  readonly nombreDelArchivo: string
  readonly filas: number
  readonly bytes: number
  readonly cifrado: boolean
  readonly alcance: string
}

/**
 * Los dos alcances de D-8B.
 *
 * `'esencial'` viene elegido, que es la decisión de Dirección: un respaldo que
 * tarda veinte minutos no se hace nunca.
 */
export const ALCANCES = ['esencial', 'completo'] as const
export type Alcance = (typeof ALCANCES)[number]

/** El nombre del evento con el que el núcleo avisa del avance. */
const EVENTO_DE_AVANCE = 'respaldo:avance'

interface Avance {
  readonly filas: number
  readonly total: number
}

export const useRespaldoStore = defineStore('respaldo', () => {
  const alcance = ref<Alcance>('esencial')
  /** `''` significa **sin cifrar**, que D-8A permite sólo a propósito. */
  const contrasena = ref('')
  const repetida = ref('')

  const previo = ref<LoQueLlevaria | null>(null)
  const escrito = ref<RespaldoEscrito | null>(null)

  const cargando = ref(false)
  const escribiendo = ref(false)
  const errorGeneral = ref<string | null>(null)

  /** Cuántas filas van escritas, de cuántas. Cero mientras no empieza. */
  const filasHechas = ref(0)

  /**
   * ¿Se puede pulsar «Crear el respaldo»?
   *
   * Dos condiciones, y la segunda es la que importa: **si hay contraseña, las
   * dos tienen que coincidir**. Se escribe una vez y no se puede recuperar, así
   * que un dedo torcido convierte el respaldo en un archivo que nadie abre.
   */
  const sePuedeEscribir = computed(
    () => !escribiendo.value && (contrasena.value === '' || contrasena.value === repetida.value),
  )

  /** `true` si hay contraseña escrita y la repetición todavía no cuadra. */
  const noCuadran = computed(
    () => contrasena.value !== '' && repetida.value !== '' && contrasena.value !== repetida.value,
  )

  const cifraria = computed(() => contrasena.value !== '')

  function recordarError(e: unknown): void {
    errorGeneral.value = claveDelError(e)
  }

  /**
   * Pregunta qué llevaría el respaldo. **No escribe nada.**
   *
   * Es el patrón del recorte del registro (7.4) y del derecho de cancelación
   * (3.4): primero se dice con cifras qué va a pasar, y después se pulsa.
   */
  async function verQueLlevaria(): Promise<void> {
    if (!hayNucleo()) return
    cargando.value = true
    errorGeneral.value = null
    try {
      previo.value = await invocar<LoQueLlevaria>('que_llevaria_el_respaldo', {
        alcance: alcance.value,
      })
    } catch (e) {
      recordarError(e)
    } finally {
      cargando.value = false
    }
  }

  /** Cambia el alcance y vuelve a preguntar: las cifras son otras. */
  async function elegirAlcance(nuevo: Alcance): Promise<void> {
    alcance.value = nuevo
    escrito.value = null
    await verQueLlevaria()
  }

  /**
   * Escribe el respaldo.
   *
   * Devuelve `false` si falló **y también si se cerró el diálogo sin elegir
   * sitio**, que no es un error: cancelar no puede pintar un aviso rojo.
   * `escrito` distingue los dos casos.
   *
   * **La contraseña se borra al terminar, pase lo que pase.** Vive en el campo
   * mientras se escribe y en ningún otro sitio: ni en el llavero, ni en la base,
   * ni de vuelta por la frontera (D-8A).
   */
  async function escribir(): Promise<boolean> {
    if (!hayNucleo() || !sePuedeEscribir.value) return false
    escribiendo.value = true
    errorGeneral.value = null
    escrito.value = null
    filasHechas.value = 0

    const dejarDeEscuchar = await escuchar<Avance>(EVENTO_DE_AVANCE, (a) => {
      filasHechas.value = a.filas
    })

    try {
      const r = await invocar<RespaldoEscrito | null>('exportar_respaldo', {
        alcance: alcance.value,
        contrasena: contrasena.value === '' ? null : contrasena.value,
      })
      escrito.value = r
      return r !== null
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      dejarDeEscuchar()
      contrasena.value = ''
      repetida.value = ''
      escribiendo.value = false
    }
  }

  return {
    alcance,
    contrasena,
    repetida,
    previo,
    escrito,
    cargando,
    escribiendo,
    errorGeneral,
    filasHechas,
    sePuedeEscribir,
    noCuadran,
    cifraria,
    verQueLlevaria,
    elegirAlcance,
    escribir,
  }
})
