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

/** Lo que hay que saber de un proveedor para conectarse a él. */
export interface Proveedor {
  /** Cómo se llama para el usuario. */
  readonly nombre: string
  readonly servidor: string
  readonly puerto: string
  readonly cifrado: string
  /**
   * Cuántos caracteres tiene su contraseña de aplicación, si usa una.
   *
   * `null` cuando el proveedor no la impone. Sirve para avisar **antes** de
   * intentar conectar: con Google, meter la contraseña normal de la cuenta es
   * el error más común y hoy sólo se descubre cuando el servidor la rechaza.
   */
  readonly largoDeContrasena: number | null
  /**
   * Dónde se crea esa contraseña, exactamente.
   *
   * ── POR QUÉ UNA DIRECCIÓN Y NO «BUSCA EN SEGURIDAD» ──
   *
   * Hasta el 15/10/2026 tanto la pantalla como el error decían «se genera en
   * la configuración de seguridad de Google». Eso es una instrucción que
   * **no se puede seguir sin buscar**, y buscar es donde la gente se rinde:
   * la opción vive al final de una página a la que se llega por tres menús, y
   * sólo aparece si la verificación en dos pasos está activada.
   *
   * `null` cuando no la sabemos. **No se inventa una**: una dirección que
   * lleva a ninguna parte es peor que no dar ninguna, porque se sigue. Hoy
   * sólo está la de Google, que es la única comprobada.
   */
  readonly dondeSeCreaLaContrasena: string | null
}

/**
 * Los proveedores que ARLES reconoce por el dominio del correo.
 *
 * ── POR QUÉ ESTA TABLA ──────────────────────────────────────────────────────
 *
 * Dirección intentó dar de alta una cuenta el 14/10/2026 y no pudo (H-03). El
 * formulario pedía **siete campos** —tres de ellos técnicos— y aceptó valores
 * que no podían funcionar sin decir nada: un usuario que no era la dirección y
 * una contraseña que no era la de aplicación.
 *
 * El requisito que salió de ahí (R-01) es: **correo y contraseña, nada más.**
 * Todo lo demás se deduce del dominio, y los tres campos técnicos se esconden
 * para quien tenga un servidor propio.
 *
 * **Los valores no se inventan**: son los que publica cada proveedor, y el par
 * puerto+cifrado va junto a propósito. Cruzarlos —587 con TLS directo, o 465
 * con STARTTLS— deja la conexión esperando a un servidor que no va a contestar,
 * y ése es exactamente el error que Dirección vio.
 */
export const PROVEEDORES: ReadonlyArray<readonly [RegExp, Proveedor]> = [
  [
    /@(gmail\.com|googlemail\.com)$/i,
    {
      nombre: 'Gmail',
      servidor: 'smtp.gmail.com',
      puerto: '587',
      cifrado: 'starttls',
      largoDeContrasena: 16,
      dondeSeCreaLaContrasena: 'https://myaccount.google.com/apppasswords',
    },
  ],
  [
    /@(outlook\.com|hotmail\.com|live\.com|msn\.com)$/i,
    { nombre: 'Outlook', servidor: 'smtp-mail.outlook.com', puerto: '587', cifrado: 'starttls', largoDeContrasena: null, dondeSeCreaLaContrasena: null },
  ],
  [
    /@(yahoo\.(com|com\.mx|es))$/i,
    { nombre: 'Yahoo', servidor: 'smtp.mail.yahoo.com', puerto: '465', cifrado: 'directo', largoDeContrasena: null, dondeSeCreaLaContrasena: null },
  ],
  [
    /@(zoho\.com|zohomail\.com)$/i,
    { nombre: 'Zoho', servidor: 'smtp.zoho.com', puerto: '465', cifrado: 'directo', largoDeContrasena: null, dondeSeCreaLaContrasena: null },
  ],
  [
    /@icloud\.com$/i,
    { nombre: 'iCloud', servidor: 'smtp.mail.me.com', puerto: '587', cifrado: 'starttls', largoDeContrasena: null, dondeSeCreaLaContrasena: null },
  ],
]

/**
 * El proveedor que corresponde a una dirección, o `null` si no se reconoce.
 *
 * Un dominio propio —`@telemetrymx.com`— devuelve `null` **a propósito**:
 * ARLES no se inventa un servidor que no sabe. En ese caso la configuración
 * avanzada se abre sola y se escriben a mano, que es lo correcto.
 */
export function proveedorDe(direccion: string): Proveedor | null {
  const limpia = direccion.trim()
  for (const [patron, proveedor] of PROVEEDORES) {
    if (patron.test(limpia)) return proveedor
  }
  return null
}

/**
 * La contraseña tal como hay que mandarla al servidor.
 *
 * ── GOOGLE LA ENSEÑA CON ESPACIOS, Y EL SERVIDOR NO LOS QUIERE (15/10/2026)
 *
 * Google presenta la contraseña de aplicación en cuatro grupos de cuatro
 * —`abcd efgh ijkl mnop`— **sólo para que se pueda leer**. Lo que vale son las
 * dieciséis letras; los espacios son del dibujo.
 *
 * Quien la copia, la copia entera. Y hasta hoy ARLES la mandaba tal cual, con
 * los tres espacios dentro, así que Gmail la rechazaba. Lo peor no es eso: es
 * que **nada avisaba**. El aviso del largo ya descontaba los espacios para
 * contar, así que con `abcd efgh ijkl mnop` veía dieciséis, se callaba, y la
 * cuenta fallaba después con un «no se pudo autenticar» que parece culpa de
 * quien lo escribió. Lo encontró una pregunta de Dirección, no una prueba.
 *
 * ── POR QUÉ NO SE QUITAN LOS ESPACIOS SIEMPRE ──
 *
 * Porque una contraseña puede llevarlos de verdad. Outlook, Yahoo, Zoho e
 * iCloud no declaran ningún largo —su `largoDeContrasena` es `null`— y ahí una
 * contraseña con espacios es una contraseña con espacios: quitárselos sería
 * mandar otra distinta y hacer fallar un alta correcta.
 *
 * Así que sólo se quitan cuando hay un largo declarado **y quitarlos da
 * exactamente ese largo**. Con Gmail, `abcd efgh ijkl mnop` da 16 y se limpia;
 * `mi clave secreta` da 15 y se respeta tal cual.
 */
export function contrasenaParaEnviar(
  proveedor: Proveedor | null,
  escrita: string,
): string {
  const largo = proveedor?.largoDeContrasena
  if (!largo) return escrita
  const sinEspacios = escrita.replace(/\s/g, '')
  return sinEspacios.length === largo ? sinEspacios : escrita
}

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
    // El aviso de la carga anterior se borra **antes** de reintentar, no al
    // terminar bien. Sin esto, el error de un intento fallido sobrevive al
    // intento que lo resuelve: Dirección entró aquí sin empresa configurada
    // —el aviso era correcto—, fue a configurarla, volvió, la carga funcionó,
    // y la pantalla seguía diciendo «todavía no has configurado los datos de
    // tu empresa» (H-01, 14/10/2026). Un error que ya no existe dando
    // instrucciones es el §95 al revés.
    errorGeneral.value = null
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
