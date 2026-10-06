import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/** Filas de otra tabla que el borrado se lleva y el archivo no guarda. */
export interface FilasArrastradas {
  readonly tabla: string
  readonly filas: number
}

/** Lo que `revisar_archivado` cuenta antes de tocar nada. */
export interface LoQueSeArchivaria {
  readonly corte: string
  readonly enviosQueSeArchivan: number
  readonly enviosQueSeQuedan: number
  /** El periodo **real** de lo que se va, no el pedido. */
  readonly primero: string | null
  readonly ultimo: string | null
  readonly campanas: number
  readonly arrastradas: readonly FilasArrastradas[]
  readonly totalArrastrado: number
}

/** Lo que pasó al archivar. */
export interface HistorialArchivado {
  readonly nombreDelArchivo: string
  readonly bytes: number
  readonly enviosArchivados: number
  readonly enviosBorrados: number
  readonly arrastradas: readonly FilasArrastradas[]
  readonly corte: string
}

/** Cuánto historial se puede dejar en la base. El mínimo es el primero. */
export const MESES_QUE_SE_OFRECEN = [12, 18, 24, 36] as const

/**
 * Archivar el historial, en dos pasos y en este orden (D-8D).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ NO HAY UN SOLO BOTÓN
 *
 * Porque el segundo paso **borra**, y lo único que separa archivar de borrar es
 * haber escrito el archivo y haberlo vuelto a leer. Entre elegir el periodo y
 * que pase algo hay una pantalla con cifras: cuántos envíos se van, de cuándo a
 * cuándo, cuántos se quedan, y **qué se pierde que el archivo no guarda**.
 *
 * Lo del final es lo que no se puede omitir: borrar envíos arrastra sucesos del
 * registro de actividad, porque cuelgan de ellos en la base, y ésos **no** van
 * en el archivo.
 *
 * El recorrido no se puede saltar desde aquí: el comando que archiva vuelve a
 * hacer el preflight por su cuenta, no se fía de esta cifra, y verifica el
 * archivo antes de borrar. Esta pantalla informa; la frontera decide.
 */
export const usarArchivado = defineStore('archivado', () => {
  /** Cuánto historial se deja en la base. Doce es el mínimo y el que viene. */
  const meses = ref<number>(MESES_QUE_SE_OFRECEN[0])
  const revisado = ref<LoQueSeArchivaria | null>(null)
  const revisando = ref(false)
  const archivando = ref(false)
  const hecho = ref<HistorialArchivado | null>(null)
  const error = ref<string | null>(null)

  /** Hay algo anterior al corte, así que hay algo que archivar. */
  const hayAlgoQueArchivar = computed(
    () => (revisado.value?.enviosQueSeArchivan ?? 0) > 0,
  )

  const sePuedeArchivar = computed(
    () => hayAlgoQueArchivar.value && !archivando.value,
  )

  /** Dice qué pasaría con el periodo elegido. No escribe ni borra nada. */
  async function revisar(): Promise<void> {
    if (!hayNucleo()) return
    revisando.value = true
    error.value = null
    hecho.value = null
    try {
      revisado.value = await invocar<LoQueSeArchivaria>('revisar_archivado', {
        meses: meses.value,
      })
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
      revisado.value = null
    } finally {
      revisando.value = false
    }
  }

  /** Escribe el archivo, lo verifica y entonces borra. */
  async function archivar(): Promise<void> {
    if (!hayNucleo() || !sePuedeArchivar.value) return
    archivando.value = true
    error.value = null
    try {
      const r = await invocar<HistorialArchivado | null>('archivar_historial', {
        meses: meses.value,
      })
      // `null` es cerrar el diálogo sin elegir dónde: una decisión, no un
      // fallo, y lo revisado sigue valiendo.
      if (r !== null) {
        hecho.value = r
        // Lo revisado describe una base que ya no es ésta.
        revisado.value = null
      }
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
      // Se deja lo revisado: si falló, **no se borró nada**, así que la cifra
      // que la pantalla enseña sigue siendo cierta. Borrarla aquí daría a
      // entender que algo cambió.
    } finally {
      archivando.value = false
    }
  }

  /**
   * Olvida lo revisado, sin tocar nada.
   *
   * Faltaba, y lo encontró mirar la pantalla el 06/10/2026: el único botón
   * debajo de las cifras era el rojo que borra. Restaurar, que hace lo mismo
   * de grave, tiene su «Mejor no» al lado. Una pantalla que enseña una acción
   * sin vuelta atrás y no ofrece salirse se lee como que ya no hay marcha
   * atrás — y la hay, hasta que se pulsa.
   */
  function descartar(): void {
    revisado.value = null
    error.value = null
  }

  /** Cambia el periodo y vuelve a preguntar. */
  async function elegirMeses(nuevos: number): Promise<void> {
    meses.value = nuevos
    revisado.value = null
    await revisar()
  }

  return {
    meses,
    revisado,
    revisando,
    archivando,
    hecho,
    error,
    hayAlgoQueArchivar,
    sePuedeArchivar,
    revisar,
    archivar,
    descartar,
    elegirMeses,
  }
})
