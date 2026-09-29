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
/**
 * Cómo salió un correo de prueba.
 *
 * Coincide con `arles_app::comandos::ResultadoDelEnvioDePrueba`.
 *
 * **No hay un campo «entregado»** y no puede haberlo (§65): ARLES sabe que el
 * servidor se hizo cargo del mensaje, no que llegara a una bandeja.
 */
export interface ResultadoDelEnvio {
  readonly aceptado: boolean
  /** Clave del texto que explica qué pasó, si no se aceptó. */
  readonly problema?: string
  /** La dirección usada, ya normalizada por el núcleo. */
  readonly aDonde: string
}

export const useRemitentesStore = defineStore('remitentes', () => {
  const remitentes = ref<readonly Remitente[]>([])
  const cargada = ref(false)
  const cargando = ref(false)
  const guardando = ref(false)
  const probando = ref(false)
  /** Hay un correo de prueba saliendo ahora mismo. */
  const mandando = ref(false)

  /** Clave de i18n del error de cada campo, por nombre de campo. */
  const erroresDeCampo = ref<Record<string, string>>({})
  const errorGeneral = ref<string | null>(null)
  /** Clave del último diagnóstico (`correo.prueba.*`), o `null`. */
  const diagnostico = ref<string | null>(null)
  /** `true` si el último diagnóstico dice que la cuenta sirve. */
  const laCuentaSirve = ref(false)
  /**
   * Cómo salió el último correo de prueba. `null` si no se ha mandado ninguno.
   *
   * Va aparte de `diagnostico` porque son dos cosas distintas —autenticar y
   * enviar— y fundirlas haría que mandar un correo borrara el diagnóstico de la
   * conexión, o al revés.
   */
  const resultadoDelEnvio = ref<ResultadoDelEnvio | null>(null)

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
   * Manda un correo de prueba **por una cuenta ya guardada**.
   *
   * ── ESTO NO ES «PROBAR CONEXIÓN», Y SON DOS BOTONES A PROPÓSITO ──
   *
   * `probar` dice que la cuenta autentica: se conecta, cifra y entra. Esto dice
   * que **puede enviar**, que no es lo mismo — una cuenta puede autenticar
   * perfectamente y que el servidor rechace el remitente, o que la dirección no
   * esté autorizada a enviar desde ese dominio.
   *
   * Y hay una diferencia que el usuario tiene que ver: `probar` no manda nada;
   * esto **manda un correo de verdad** a la dirección que escriba.
   *
   * No anota ninguna prueba de campaña: esa marca cuelga de una etapa, y aquí
   * no hay ninguna.
   */
  async function mandarCorreoDePrueba(id: string, aDonde: string): Promise<boolean> {
    mandando.value = true
    limpiarAvisos()
    resultadoDelEnvio.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      const r = await invocar<ResultadoDelEnvio>('probar_envio_de_la_cuenta', {
        id,
        aDonde,
      })
      resultadoDelEnvio.value = r
      return r.aceptado
    } catch (e) {
      repartirError(e)
      return false
    } finally {
      mandando.value = false
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
    mandando,
    resultadoDelEnvio,
    erroresDeCampo,
    errorGeneral,
    diagnostico,
    laCuentaSirve,
    hayRemitentes,
    porConfigurar,
    cargar,
    probar,
    mandarCorreoDePrueba,
    guardar,
    limpiarAvisos,
  }
})
