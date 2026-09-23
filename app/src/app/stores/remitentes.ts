import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/**
 * Una cuenta remitente tal como la devuelve el núcleo.
 *
 * Coincide con `arles_app::comandos::RemitenteEnPantalla`. **No tiene un campo
 * de contraseña, y no es un descuido de este tipo**: el de Rust tampoco lo
 * tiene. Lo que sale hacia aquí no incluye secretos (regla 3 de la frontera), y
 * hay una prueba en Rust que lo comprueba serializando.
 */
export interface Remitente {
  readonly id: string
  readonly servidor: string
  readonly puerto: number
  /** `starttls` o `directo`. Se traduce con `correo.cifrado.*`. */
  readonly cifrado: string
  readonly usuario: string
  readonly remitente: string
  readonly nombreVisible: string | null
  /** `connected`, `needs_reauth`, `circuit_open` o `disabled`. */
  readonly estado: string
  readonly limiteDiario: number
  readonly limitePorHora: number
}

/** Lo que el formulario envía. Coincide con `BorradorDeCuenta`. */
export interface BorradorDeCuenta {
  servidor: string
  /** Texto, no número: un «58a» tiene que marcar SU casilla, no reventar. */
  puerto: string
  cifrado: string
  usuario: string
  remitente: string
  nombreVisible: string
}

/** Lo que devuelven guardar y probar. */
export interface ResultadoDeRemitente {
  readonly diagnostico: string | null
  readonly sirve: boolean
  readonly remitentes: readonly Remitente[]
}

/**
 * Los datos que trae una cuenta de Gmail, que es la que va a usar casi todo el
 * mundo (D-1).
 *
 * Se rellenan solos al abrir el formulario porque son siempre los mismos y
 * porque equivocarse en el puerto es el error más común: alguien que escribe
 * 465 con STARTTLS se queda esperando a un servidor que no va a contestar.
 */
export const GMAIL = {
  servidor: 'smtp.gmail.com',
  puerto: '587',
  cifrado: 'starttls',
} as const

export function borradorVacio(): BorradorDeCuenta {
  return {
    servidor: GMAIL.servidor,
    puerto: GMAIL.puerto,
    cifrado: GMAIL.cifrado,
    usuario: '',
    remitente: '',
    nombreVisible: '',
  }
}

/**
 * Las cuentas remitentes: listarlas, probarlas y guardarlas.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * LA CONTRASEÑA NO SE GUARDA AQUÍ
 *
 * Ni en esta tienda, ni en `localStorage`, ni en un `ref` que sobreviva a la
 * pantalla. Vive en el campo del formulario mientras se escribe, viaja al
 * núcleo en la llamada, y se borra al cerrar. Guardarla «para no pedirla otra
 * vez» la dejaría en la memoria de la webview, que es exactamente donde no
 * tiene que estar (ADR-0011).
 */
export const useRemitentesStore = defineStore('remitentes', () => {
  const remitentes = ref<readonly Remitente[]>([])
  const cargada = ref(false)
  const cargando = ref(false)
  const guardando = ref(false)
  const probando = ref(false)

  /** Clave de i18n del error de cada campo, por nombre de campo. */
  const erroresDeCampo = ref<Record<string, string>>({})
  const errorGeneral = ref<string | null>(null)
  /** Clave del último diagnóstico (`correo.prueba.*`), o `null`. */
  const diagnostico = ref<string | null>(null)
  /** `true` si el último diagnóstico dice que la cuenta sirve. */
  const laCuentaSirve = ref(false)

  const hayRemitentes = computed(() => remitentes.value.length > 0)
  /** Cuántas cuentas hay que configurar (§94: cifras, no adjetivos). */
  const porConfigurar = computed(
    () => remitentes.value.filter((r) => r.estado !== 'connected').length,
  )

  function limpiarAvisos(): void {
    erroresDeCampo.value = {}
    errorGeneral.value = null
    diagnostico.value = null
    laCuentaSirve.value = false
  }

  async function cargar(): Promise<void> {
    if (cargada.value || cargando.value) return
    cargando.value = true
    try {
      if (hayNucleo()) {
        remitentes.value = await invocar<Remitente[]>('remitentes')
      }
      cargada.value = true
    } catch (e) {
      errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
    } finally {
      cargando.value = false
    }
  }

  /** Reparte el error entre los campos y el aviso general. */
  function repartirError(e: unknown): void {
    if (esErrorIpc(e)) {
      const campos = e.camposDeCuenta ?? []
      if (campos.length > 0) {
        erroresDeCampo.value = Object.fromEntries(
          campos.map((c) => [c.campo, c.clave]),
        )
      } else {
        errorGeneral.value = e.clave
      }
    } else {
      errorGeneral.value = 'error.db.sqlite'
    }
  }

  /**
   * Prueba la cuenta **sin guardarla**.
   *
   * @returns `true` si la cuenta sirve.
   */
  async function probar(
    borrador: BorradorDeCuenta,
    contrasena: string,
    id: string | null,
  ): Promise<boolean> {
    probando.value = true
    limpiarAvisos()
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      const r = await invocar<ResultadoDeRemitente>('probar_remitente', {
        borrador,
        contrasena,
        id,
      })
      remitentes.value = r.remitentes
      diagnostico.value = r.diagnostico
      laCuentaSirve.value = r.sirve
      return r.sirve
    } catch (e) {
      repartirError(e)
      return false
    } finally {
      probando.value = false
    }
  }

  /**
   * Guarda la cuenta. El núcleo la prueba antes y la marca según el resultado.
   *
   * @returns `true` si se guardó, **aunque la prueba no haya pasado**: la
   * cuenta queda escrita y marcada como pendiente de configurar, así que la
   * pantalla tiene que cerrarse y enseñar el diagnóstico, no quedarse con el
   * formulario abierto como si no hubiera pasado nada.
   */
  async function guardar(
    borrador: BorradorDeCuenta,
    contrasena: string,
    id: string | null,
  ): Promise<boolean> {
    guardando.value = true
    limpiarAvisos()
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      const r = await invocar<ResultadoDeRemitente>('guardar_remitente', {
        borrador,
        contrasena,
        id,
      })
      remitentes.value = r.remitentes
      diagnostico.value = r.diagnostico
      laCuentaSirve.value = r.sirve
      cargada.value = true
      return true
    } catch (e) {
      repartirError(e)
      return false
    } finally {
      guardando.value = false
    }
  }

  return {
    remitentes,
    cargada,
    cargando,
    guardando,
    probando,
    erroresDeCampo,
    errorGeneral,
    diagnostico,
    laCuentaSirve,
    hayRemitentes,
    porConfigurar,
    cargar,
    probar,
    guardar,
    limpiarAvisos,
  }
})
