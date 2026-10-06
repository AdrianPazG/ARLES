import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/** Una tabla, con lo que hay ahora y lo que trae el respaldo. */
export interface TablaQueVuelve {
  readonly tabla: string
  readonly ahora: number
  readonly enElRespaldo: number
}

/** Algo que se pierde al restaurar y **no vuelve**. */
export interface LoQueNoVuelve {
  readonly tabla: string
  readonly filas: number
  /** `true` si describe este equipo y perderlo es correcto. */
  readonly esDeEsteEquipo: boolean
}

/** Lo que `revisar_respaldo` cuenta antes de tocar nada. */
export interface LoQueTraeElRespaldo {
  readonly nombreDelArchivo: string
  readonly creadoEn: string
  readonly alcance: string
  readonly esquemaDelRespaldo: string
  readonly esquemaDeLaBase: string
  readonly llevaContrasena: boolean
  readonly sePuedeRestaurar: boolean
  readonly tablas: readonly TablaQueVuelve[]
  readonly totalDeFilas: number
  readonly noVuelven: readonly LoQueNoVuelve[]
  readonly filasQueNoVuelven: number
  readonly bajasAhora: number
  readonly bajasEnElRespaldo: number
  readonly bajasAlMenos: number
  readonly cuentasAReconectar: number
}

/** Lo que pasó al restaurar. */
export interface RespaldoRestaurado {
  readonly filas: number
  readonly bajasAntes: number
  readonly bajasDespues: number
  readonly bajasQueSeSalvaron: number
  readonly cuentasAReconectar: number
}

/**
 * Restaurar, en dos pasos y en este orden.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ NO HAY UN SOLO BOTÓN
 *
 * Reemplazar la base entera es la acción más destructiva de ARLES, y la única
 * que no tiene deshacer. Entre elegir el archivo y que pase algo hay una
 * pantalla con cifras: qué trae, qué hay ahora, **qué se pierde sin volver** y
 * cuántas cuentas habrá que reconectar.
 *
 * El núcleo lo hace imposible de saltar: `restaurar_respaldo` no recibe ninguna
 * ruta —la regla 4 de la frontera— sino que usa la que `revisar_respaldo`
 * guardó. Sin revisar no hay nada que restaurar, y el error lo dice.
 */
export const usarRestauracion = defineStore('restauracion', () => {
  /** Lo que trae el respaldo elegido, o `null` si no hay ninguno en revisión. */
  const revisado = ref<LoQueTraeElRespaldo | null>(null)
  const contrasena = ref('')
  const revisando = ref(false)
  const restaurando = ref(false)
  const hecho = ref<RespaldoRestaurado | null>(null)
  const error = ref<string | null>(null)

  /**
   * Hace falta contraseña y todavía no se ha escrito ninguna.
   *
   * La cabecera va en claro a propósito, así que esto se sabe **antes** de
   * pedirla: ni se pide por si acaso, ni se descubre fallando.
   */
  const faltaLaContrasena = computed(
    () => revisado.value?.llevaContrasena === true && contrasena.value === '',
  )

  const sePuedeRestaurar = computed(
    () =>
      revisado.value !== null &&
      revisado.value.sePuedeRestaurar &&
      !faltaLaContrasena.value &&
      !restaurando.value,
  )

  /** Lo que se pierde y son datos, separado de lo que es estado del equipo. */
  const perdidasDeDatos = computed(() =>
    (revisado.value?.noVuelven ?? []).filter((p) => !p.esDeEsteEquipo),
  )

  /** Elige un archivo y dice qué pasaría. No toca la base. */
  async function revisar(): Promise<void> {
    if (!hayNucleo()) return
    revisando.value = true
    error.value = null
    hecho.value = null
    try {
      const r = await invocar<LoQueTraeElRespaldo | null>('revisar_respaldo')
      // `null` es cerrar el diálogo sin elegir: una decisión, no un fallo. Se
      // deja lo que hubiera revisado antes, para no borrar la pantalla por
      // haber abierto el diálogo y arrepentirse.
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

  /** Reemplaza la base. Entero o nada. */
  async function restaurar(): Promise<void> {
    if (!hayNucleo() || !sePuedeRestaurar.value) return
    restaurando.value = true
    error.value = null
    try {
      hecho.value = await invocar<RespaldoRestaurado>('restaurar_respaldo', {
        contrasena: contrasena.value === '' ? null : contrasena.value,
      })
      // El respaldo revisado se consume con la restauración, igual que en el
      // núcleo: lo que la pantalla enseñaba ya no describe esta base.
      revisado.value = null
    } catch (e) {
      error.value = esErrorIpc(e) ? e.clave : 'error.desconocido'
      // Y también aquí: el núcleo soltó la ruta pase lo que pase, así que
      // dejar la revisión puesta ofrecería un botón que ya no puede funcionar.
      revisado.value = null
    } finally {
      restaurando.value = false
      contrasena.value = ''
    }
  }

  /** Olvida lo revisado, sin tocar nada. */
  function descartar(): void {
    revisado.value = null
    contrasena.value = ''
    error.value = null
  }

  return {
    revisado,
    contrasena,
    revisando,
    restaurando,
    hecho,
    error,
    faltaLaContrasena,
    sePuedeRestaurar,
    perdidasDeDatos,
    revisar,
    restaurar,
    descartar,
  }
})
