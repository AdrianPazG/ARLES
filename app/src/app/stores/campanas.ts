import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { claveDelError, escuchar, hayNucleo, invocar } from '@/app/nucleo'

/**
 * El umbral del aviso de volumen (§48).
 *
 * ── POR QUÉ ESTÁ AQUÍ SI YA ESTÁ EN RUST ──
 *
 * Porque la pantalla tiene que enseñar la casilla **antes** de llamar a nadie:
 * un aviso que aparece después de pulsar «crear» llega tarde. Así que hay dos
 * copias, y eso es una divergencia esperando a pasar.
 *
 * Lo que la sostiene: quien **exige** la aceptación es `crear_campana`, en
 * Rust. Si estas dos cifras dejaran de coincidir, el peor caso es una casilla
 * de más o un error del núcleo que la pantalla no vio venir — nunca una campaña
 * de alto volumen creada sin que conste el aviso. La copia puede mentir; la
 * regla, no.
 */
export const AVISO_DIARIO = 50

/** Una plantilla con su versión vigente. Coincide con `PlantillaIpc`. */
export interface Plantilla {
  readonly id: string
  readonly nombre: string
  readonly version: {
    readonly id: string
    readonly numero: number
    readonly asunto: string
    readonly cuerpoHtml: string
    /** Claves de variable: `nombre`, `apellido`, `empresa`, `mi_empresa`. */
    readonly variables: readonly string[]
  } | null
}

/** Una firma guardada. Coincide con `FirmaIpc`. */
export interface Firma {
  readonly id: string
  readonly nombre: string
  readonly cuerpoHtml: string
  readonly porDefecto: boolean
}

/** Un horario de envío. Coincide con `VentanaIpc`. */
export interface Ventana {
  readonly id: string
  readonly nombre: string
  /** Máscara de bits: bit 0 = lunes … bit 6 = domingo. */
  readonly dias: number
  readonly desde: number
  readonly hasta: number
  readonly zona: string
}

/** Una campaña en la lista. Coincide con `CampanaIpc`. */
export interface Campana {
  readonly id: string
  readonly nombre: string
  /** Clave del rótulo de estado (`campana.estado.*`). */
  readonly estado: string
  readonly audiencia: number
  /** Cuántos **aceptó el proveedor**. Nunca «entregados» (§65). */
  readonly aceptados: number
}

/** Un punto del preflight con su veredicto. */
export interface PuntoRevisado {
  /** `laCuentaAutentica`, `hayConexion`, … */
  readonly punto: string
  /** `pasa`, `falla` o `sinComprobar`. */
  readonly veredicto: string
}

/**
 * Cuándo termina una campaña, según el simulador.
 *
 * ── ESTE TIPO MENTÍA, Y COMPILABA ──────────────────────────────────────
 *
 * Hasta la 8.6 E decía `desenlace: { resultado, cuando }`. Lo que Rust manda
 * es `{ desenlace, cuando?, faltan? }` — el enum va etiquetado con `desenlace`
 * y `resultado` no existe en el JSON.
 *
 * `vue-tsc` no lo cazó y no podía: un tipo sólo se contradice al **leerlo**, y
 * nadie leía este campo. Es la cola del patrón del §9 que esta entrega vino a
 * cerrar — cuando nadie usa una pieza, el tipo que la describe se desalinea sin
 * que nada falle.
 */
export interface Estimacion {
  readonly desenlace: {
    /** `termina`, `noAvanza` o `noCabeEnElHorizonte`. */
    readonly desenlace: string
    readonly cuando?: string
    /** Cuántos se quedan fuera. Sólo en `noCabeEnElHorizonte`. */
    readonly faltan?: number
  }
  readonly audiencia: number
  readonly techoDiario: number
  readonly quienManda: string
  readonly diasConEnvio: number
}

/** Lo que devuelve `revisar_campana`. Coincide con `RevisionDeCampana`. */
export interface Revision {
  readonly preflight: {
    readonly puntos: readonly PuntoRevisado[]
    readonly resultado: {
      readonly resultado: string
      readonly avisos?: number
      readonly fallan?: number
      readonly sinComprobar?: number
    }
  }
  readonly audiencia: number
  readonly contactosConHuecos: number
  readonly estimacion: Estimacion
  /** Clave del diagnóstico de la sonda, o `null` si no se sondeó. */
  readonly diagnostico: string | null
  /**
   * Las campañas que **ya están enviando** con este mismo remitente.
   *
   * ── LO QUE ESTO SIGNIFICA CAMBIÓ EL 22/10/2026 POR LA TARDE (D-13) ──────
   *
   * Hasta esa tarde el cupo colgaba de la cuenta: dos campañas de 50 sacaban
   * 25 y 25 **por orden de llegada**, y lo que se rompía era el plazo que el
   * paso 6 promete. Por eso existe este campo.
   *
   * Desde la D-13 el cupo cuelga de la campaña y **los topes se suman**: las
   * dos sacan 50, y el plazo se cumple. Así que el aviso ya no desmiente la
   * estimación — avisa de lo otro: que de ese buzón van a salir 100 al día, y
   * el que arriesga el dominio es el total, no cada campaña.
   */
  readonly compartenRemitente: readonly {
    readonly nombre: string
    readonly limiteDiario: number
  }[]
}

/**
 * El estado de la parada de emergencia. Coincide con `EstadoDelEnvioIpc`.
 *
 * `clave` es el texto para la pantalla (§139), no un código propio: lo decide
 * el núcleo, así que no hay dos formas de nombrar el mismo estado.
 */
export interface Parada {
  readonly parado: boolean
  /** Desde cuándo, si está puesta. */
  readonly desde: string | null
  /** Quién la puso, si consta. */
  readonly por: string | null
  readonly clave: string
}

/** Lo que queda al crear una campaña. Coincide con `CampanaCreadaIpc`. */
export interface CampanaCreada {
  readonly id: string
  /**
   * El identificador de la etapa de correo.
   *
   * Viaja desde Rust porque el envío de prueba (§45) cuelga de la etapa, no de
   * la campaña. Construirlo aquí a partir del id de la campaña sería copiar un
   * detalle de `arles-db` que dejaría de ser cierto sin avisar.
   */
  readonly etapa: string
}

/** Lo que pasó al activar. Coincide con `ActivacionIpc`. */
export interface Activacion {
  readonly enLaAudiencia: number
  readonly fueraPorBaja: number
}

/**
 * Cómo va una activación, tal y como llega del núcleo.
 *
 * `van` y `de` **sólo vienen en la fase `cola`**, y la diferencia entre
 * ausentes y cero importa: con `0` la pantalla pintaría una barra al 0 %
 * durante el 38 % de la espera, sin forma de saber que ahí no hay cifra que
 * pintar. Medido el 28/10/2026 — ver `FaseDeActivacion` en el núcleo.
 */
export interface ComoVa {
  /** `congelando`, `cola` o `cerrando`. Clave de i18n, no texto (§139). */
  readonly fase: 'congelando' | 'cola' | 'cerrando'
  readonly van?: number
  readonly de?: number
  /**
   * ¿Sirve de algo cancelar todavía?
   *
   * Viene del núcleo y **no se deduce aquí de la fase**: quien sabe la
   * respuesta es quien tiene el dato, igual que con los tres puntos de red del
   * preflight. En `cerrando` es `false`, porque a esa altura sólo quedan dos
   * `UPDATE` de una fila y el `COMMIT`.
   */
  readonly sePuedeCancelar: boolean
}

/**
 * Campañas, plantillas, firmas y horarios.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * AQUÍ NO SE DECIDE SI UNA CAMPAÑA SE PUEDE ACTIVAR
 *
 * Eso lo dice `revision.preflight.resultado`, que viene de Rust. Esta tienda lo
 * guarda y la pantalla lo pinta; ninguna de las dos lo recalcula.
 *
 * Si la regla viviera aquí habría dos copias, y el día que divergieran la
 * pantalla encendería un botón que el núcleo rechaza — o, peor, dejaría activar
 * una campaña que el preflight había bloqueado, que es irreversible.
 */
/**
 * Por dónde sale una campaña, con lo que ese canal necesita y nada más (D-WA).
 *
 * ── UNA UNIÓN ETIQUETADA, Y NO SIETE CAMPOS OPCIONALES ──
 *
 * Es la forma que `PorDondeSaleIpc` deserializa en Rust, y la etiqueta la
 * rechaza si llega mezclada: un `{ canal: 'whatsapp', cuenta: '…' }` no
 * deserializa y el comando no llega a correr.
 *
 * Escrita así aquí, **TypeScript exige lo mismo antes de salir**: no se puede
 * construir un objeto con `canal: 'whatsapp'` y `cuenta` dentro. Con siete
 * campos opcionales se podía, y el error habría aparecido al otro lado de la
 * frontera como un fallo de deserialización sin decir qué campo sobra.
 */
export type PorDondeSale =
  | {
      canal: 'email'
      cuenta: string
      versionDePlantilla: string
      /**
       * La firma del pie, o `null` para la de por defecto de la empresa (H-12).
       *
       * `campaign_stage.signature_id` existía desde la V4 y nada la escribía,
       * así que todas las campañas salían con la misma firma. Ahora se elige en
       * el paso 4 del asistente.
       */
      firma: string | null
    }
  | {
      canal: 'whatsapp'
      numero: string
      /**
       * La plantilla aprobada de Meta, o `null` si todavía no se eligió.
       *
       * **No lleva firma**: una firma es cosa del correo y el esquema lo exige
       * con un `CHECK` desde la V4.
       */
      plantillaDeMeta: string | null
    }

/**
 * El preflight del canal de WhatsApp (W.5, conectado en D-WA 3c-2b).
 *
 * **No es el mismo que el del correo y no se mezclan.** El de correo sondea el
 * servidor SMTP y los registros del dominio; éste mira la puerta del canal: las
 * dos cifras de permiso, Coexistencia, el riesgo aceptado del modo en frío, y
 * cuántas plantillas hay aprobadas.
 *
 * `sePuedeActivar` **viene resuelto de Rust**. Recalcularlo aquí sería una
 * segunda verdad sobre si se puede enviar, y la que decide es la otra.
 */
export interface RevisionDeWhatsapp {
  readonly audiencia: number
  /** De ésos, a cuántos consta permiso vigente. */
  readonly conPermiso: number
  /** Y a cuántos no. **Es la cifra que cambia el botón de activar.** */
  readonly sinPermiso: number
  readonly coexistencia: boolean
  readonly riesgoAceptado: boolean
  readonly escribeEnFrio: boolean
  readonly plantillasAprobadas: number
  readonly sePuedeActivar: boolean
  readonly pidePalabraDeConfirmacion: boolean
  /**
   * La calidad que Meta asigna al número, o `null` si no se pudo preguntar.
   *
   * **`null` es «no se sabe», no «está bien»** — la misma regla que el «sin
   * comprobar» del preflight de correo.
   */
  readonly calidad: string | null
}

export const useCampanasStore = defineStore('campanas', () => {
  const campanas = ref<readonly Campana[]>([])
  const plantillas = ref<readonly Plantilla[]>([])
  const firmas = ref<readonly Firma[]>([])
  const ventanas = ref<readonly Ventana[]>([])

  const cargada = ref(false)
  const cargando = ref(false)
  const trabajando = ref(false)
  const errorGeneral = ref<string | null>(null)

  /** La última revisión pedida. `null` mientras no se haya pedido ninguna. */
  const revision = ref<Revision | null>(null)
  const revisionDeWhatsapp = ref<RevisionDeWhatsapp | null>(null)
  const revisando = ref(false)

  const hayCampanas = computed(() => campanas.value.length > 0)
  /** Cuántas están enviando ahora mismo (§94). */
  const enMarcha = computed(
    () => campanas.value.filter((c) => c.estado === 'campana.estado.enviando').length,
  )

  function recordarError(e: unknown): void {
    errorGeneral.value = claveDelError(e)
  }

  async function cargar(): Promise<void> {
    if (cargada.value || cargando.value) return
    cargando.value = true
    errorGeneral.value = null
    try {
      if (hayNucleo()) {
        // Las cuatro juntas: el asistente las necesita todas en su primer paso,
        // y cargarlas por separado deja pasos que aparecen vacíos y se rellenan
        // solos delante de quien los está leyendo.
        campanas.value = await invocar<Campana[]>('campanas')
        plantillas.value = await invocar<Plantilla[]>('plantillas')
        firmas.value = await invocar<Firma[]>('firmas')
        ventanas.value = await invocar<Ventana[]>('ventanas')
      }
      cargada.value = true
    } catch (e) {
      recordarError(e)
    } finally {
      cargando.value = false
    }
  }

  async function guardarPlantilla(
    nombre: string,
    asunto: string,
    cuerpoHtml: string,
  ): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      plantillas.value = await invocar<Plantilla[]>('guardar_plantilla', {
        nombre,
        asunto,
        cuerpoHtml,
      })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  /**
   * Hace nacer **la versión siguiente** de una plantilla que ya existe.
   *
   * No reescribe la anterior: lo que ya se activó sigue saliendo con el texto
   * que eligió (§44). Es lo que la pantalla llama «editar», y la diferencia se
   * dice ahí con palabras, porque es contraintuitiva.
   */
  async function nuevaVersionDePlantilla(
    plantilla: string,
    asunto: string,
    cuerpoHtml: string,
  ): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      plantillas.value = await invocar<Plantilla[]>('nueva_version_de_plantilla', {
        plantilla,
        asunto,
        cuerpoHtml,
      })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  async function guardarFirma(
    nombre: string,
    cuerpoHtml: string,
    porDefecto: boolean,
  ): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      firmas.value = await invocar<Firma[]>('guardar_firma', {
        nombre,
        cuerpoHtml,
        porDefecto,
      })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  /**
   * Reescribe una firma que ya existe, **encima**.
   *
   * Una firma no tiene versiones, así que aquí no hay nada congelado que
   * proteger. El porqué entero está en `actualizar_firma` de `arles-db`.
   */
  async function editarFirma(
    firma: string,
    nombre: string,
    cuerpoHtml: string,
    porDefecto: boolean,
  ): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      firmas.value = await invocar<Firma[]>('editar_firma', {
        firma,
        nombre,
        cuerpoHtml,
        porDefecto,
      })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  /**
   * Retira una plantilla: deja de ofrecerse y **no se borra**.
   *
   * Decisión de Dirección del 21/10/2026. Lo que una campaña envió tiene que
   * poder consultarse, así que el texto se queda en la base.
   */
  async function retirarPlantilla(plantilla: string): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      plantillas.value = await invocar<Plantilla[]>('retirar_plantilla', { plantilla })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  /** Retira una firma, y con ella su marca de «por defecto». */
  async function retirarFirma(firma: string): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      firmas.value = await invocar<Firma[]>('retirar_firma', { firma })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  async function guardarVentana(
    nombre: string,
    dias: number,
    desde: number,
    hasta: number,
    zona: string,
  ): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      ventanas.value = await invocar<Ventana[]>('guardar_ventana', {
        nombre,
        dias,
        desde,
        hasta,
        zona,
      })
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  /** Crea la campaña en borrador. Devuelve sus ids, o `null` si falló. */
  async function crearCampana(datos: {
    nombre: string
    salePor: PorDondeSale
    /**
     * **A quién va dirigida** (D-15), o `null` para toda la empresa.
     *
     * `null` es lo que hacían todas las campañas hasta el 23/10/2026, y sigue
     * siendo legítimo —un aviso que va a todo el padrón existe—. Lo que no
     * puede ser es lo que ocurre sin que nadie lo elija: el paso 2 del
     * asistente lo pregunta con esas palabras.
     */
    lista: string | null
    ventana: string
    limitePorHora: number | null
    limiteDiario: number | null
    aceptoElRiesgoDelVolumen: boolean
  }): Promise<CampanaCreada | null> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      const creada = await invocar<CampanaCreada>('crear_campana', datos)
      campanas.value = await invocar<Campana[]>('campanas')
      return creada
    } catch (e) {
      recordarError(e)
      return null
    } finally {
      trabajando.value = false
    }
  }

  /** Pide el preflight. Habla con el servidor, así que puede tardar. */
  async function revisar(id: string): Promise<void> {
    revisando.value = true
    errorGeneral.value = null
    try {
      revision.value = await invocar<Revision>('revisar_campana', { id })
    } catch (e) {
      // La revisión se pierde a propósito: dejar la anterior en pantalla
      // mientras se enseña un error haría creer que ese veredicto sigue siendo
      // el de ahora, y es sobre el que se pulsa «activar».
      revision.value = null
      recordarError(e)
    } finally {
      revisando.value = false
    }
  }

  /**
   * Pide el preflight del canal de WhatsApp.
   *
   * `revisar_campana_de_whatsapp` existía desde W.5 y estuvo **desregistrado
   * desde la 7.7** con su motivo escrito en `lib.rs`: era «la puerta de un
   * flujo que no existe», porque el motor no recorría WhatsApp. Lo recorre
   * desde el tramo 1, y desde el 3a se puede crear y activar una campaña del
   * canal — así que vuelve, con su llamador.
   */
  async function revisarWhatsapp(id: string): Promise<void> {
    revisando.value = true
    errorGeneral.value = null
    try {
      revisionDeWhatsapp.value = await invocar<RevisionDeWhatsapp>(
        'revisar_campana_de_whatsapp',
        { id },
      )
    } catch (e) {
      // Se pierde a propósito, igual que la del correo: dejar la anterior en
      // pantalla mientras se enseña un error haría creer que ese veredicto
      // sigue siendo el de ahora — y es sobre el que se pulsa «activar».
      revisionDeWhatsapp.value = null
      recordarError(e)
    } finally {
      revisando.value = false
    }
  }

  /** Manda el correo de prueba de una etapa (§45). */
  async function enviarPrueba(
    etapa: string,
    aDonde: string,
  ): Promise<{ aceptado: boolean; problema?: string } | null> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      return await invocar<{ aceptado: boolean; problema?: string }>(
        'enviar_correo_de_prueba',
        { etapa, aDonde },
      )
    } catch (e) {
      recordarError(e)
      return null
    } finally {
      trabajando.value = false
    }
  }

  /**
   * Cómo va la activación en curso, o `null` si no hay ninguna.
   *
   * ── POR QUÉ ESTO EXISTE (Fase 9) ─────────────────────────────────────────
   *
   * Activar tarda **73.5 s con 451 196 destinatarios** y rompe su presupuesto
   * de 2 s **hacia los 30 000**, que es una lista ordinaria (auditoría C,
   * 07/10/2026). Hasta hoy no había cifra de avance y no se podía cancelar.
   *
   * `fase` es una clave y no un texto: el §139 manda, y el texto vive en el
   * catálogo. `van` y `de` son opcionales **a propósito** — sólo la fase de la
   * cola tiene cifra, porque es el único tramo que se puede contar fila a
   * fila. Interpolar un porcentaje sobre los otros dos sería inventarse una
   * medición (§94).
   */
  const comoVa = ref<ComoVa | null>(null)

  /** Activa. **No se deshace.** */
  async function activar(id: string): Promise<Activacion | null> {
    trabajando.value = true
    errorGeneral.value = null
    comoVa.value = null

    // El oyente se pone **antes** de invocar: puesto después, los primeros
    // avisos de una campaña pequeña ya habrían pasado y la barra aparecería a
    // mitad. Y se quita siempre, pase lo que pase — un oyente que no se quita
    // sigue vivo al salir de la pantalla, y la segunda visita deja dos
    // escribiendo sobre el mismo contador.
    const dejarDeEscuchar = await escuchar<ComoVa>('campana:activando', (c) => {
      comoVa.value = c
    })
    try {
      const a = await invocar<Activacion>('activar_campana', { id })
      campanas.value = await invocar<Campana[]>('campanas')
      return a
    } catch (e) {
      recordarError(e)
      return null
    } finally {
      dejarDeEscuchar()
      comoVa.value = null
      trabajando.value = false
    }
  }

  /**
   * Pide que la activación en curso pare.
   *
   * No deshace nada por sí misma: levanta una bandera que el núcleo consulta
   * entre tandas, y quien deshace es su transacción. Por eso cancelar **no
   * puede dejar una campaña a medio activar**.
   *
   * Devuelve `false` si ya había terminado, y **no es un error**: es la carrera
   * normal de pulsar mientras la barra se mueve.
   */
  async function cancelarLaActivacion(): Promise<boolean> {
    try {
      return await invocar<boolean>('cancelar_activacion', {})
    } catch (e) {
      recordarError(e)
      return false
    }
  }

  /**
   * Pausar, reanudar y detener.
   *
   * ── LAS TRES COMPARTEN CAMINO, Y NO ES POR AHORRAR LÍNEAS ──
   *
   * Es para que **las tres refresquen la lista igual**. La versión obvia
   * —una función por acción— deja que una de ellas se olvide del refresco, y
   * entonces la pantalla enseña «enviando» sobre una campaña ya detenida.
   * Eso no es un defecto cosmético: es alguien mirando un estado que no es.
   */
  async function aplicar(
    comando: 'pausar_campana' | 'reanudar_campana' | 'detener_campana',
    argumentos: Record<string, unknown>,
  ): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      await invocar<unknown>(comando, argumentos)
      campanas.value = await invocar<Campana[]>('campanas')
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  const pausar = (id: string): Promise<boolean> => aplicar('pausar_campana', { id })
  const reanudar = (id: string): Promise<boolean> => aplicar('reanudar_campana', { id })

  /**
   * Detener **no se deshace**, y por eso pide el nombre escrito.
   *
   * Quien comprueba que coincide es el comando en Rust, no esta función: lo
   * que sólo se comprueba en la webview se lo salta cualquier otro camino.
   * Aquí se manda tal cual y se deja que el núcleo lo rechace.
   */
  const detener = (id: string, nombreEscrito: string): Promise<boolean> =>
    aplicar('detener_campana', { id, nombreEscrito })

  /**
   * La parada de emergencia: **todas las campañas a la vez**.
   *
   * ═════════════════════════════════════════════════════════════════════════
   * EL SÉPTIMO CASO DEL PATRÓN, OTRA VEZ, Y EN LA ENTREGA QUE LO ARREGLABA
   *
   * La Fase 4 construyó la parada de emergencia. La 7.0 le puso sus tres
   * comandos de Tauri —parar todo, levantarla y consultarla—, con sus pruebas,
   * y el simulador de la vista previa los contestaba.
   *
   * Y **ninguna pantalla los llamaba**. La comprobación que la 7.0 escribió
   * para este patrón, `el_freno_llega_hasta_la_pantalla`, sólo miraba pausar,
   * reanudar y detener: verificaba tres de seis y pasaba en verde.
   *
   * Lo encontró otra comprobación en la 7.3, por la dirección contraria: tres
   * `case` del simulador que nadie pedía. **Un comando sin nadie que lo llame y
   * un decorado sin nadie que lo use son el mismo hueco visto desde los dos
   * lados.**
   * ═════════════════════════════════════════════════════════════════════════
   *
   * Es distinta de detener una campaña, y las dos hacen falta:
   *
   *   - **Detener** cancela lo pendiente de una campaña, y no se deshace.
   *   - **Parar todo** es un interruptor: nada sale de ninguna campaña hasta
   *     que alguien lo levante, y lo pendiente sigue pendiente. Es lo que se
   *     pulsa cuando algo va mal y no se sabe todavía qué.
   *
   * Por eso ésta **sí se puede levantar** y no pide escribir ningún nombre: lo
   * que se pide confirmar es lo irreversible, y pedir confirmación para algo
   * reversible y urgente sólo añade segundos a una emergencia.
   */
  const parada = ref<Parada | null>(null)

  async function estadoDelEnvio(): Promise<void> {
    if (!hayNucleo()) return
    try {
      parada.value = await invocar<Parada>('estado_del_envio')
    } catch (e) {
      recordarError(e)
    }
  }

  /** Pone o levanta la parada. `poner` dice cuál de las dos. */
  async function cambiarParada(poner: boolean, quien?: string): Promise<boolean> {
    trabajando.value = true
    errorGeneral.value = null
    try {
      parada.value = poner
        ? await invocar<Parada>('parar_todo', { quien: quien ?? null })
        : await invocar<Parada>('levantar_parada')
      // La lista se refresca por lo mismo que en `aplicar`: si no, la pantalla
      // sigue enseñando «enviando» sobre campañas que ya no salen.
      campanas.value = await invocar<Campana[]>('campanas')
      return true
    } catch (e) {
      recordarError(e)
      return false
    } finally {
      trabajando.value = false
    }
  }

  const pararTodo = (quien?: string): Promise<boolean> => cambiarParada(true, quien)
  const levantarParada = (): Promise<boolean> => cambiarParada(false)

  return {
    parada,
    estadoDelEnvio,
    pararTodo,
    levantarParada,
    campanas,
    plantillas,
    firmas,
    ventanas,
    cargada,
    cargando,
    trabajando,
    revisando,
    revision,
    revisionDeWhatsapp,
    revisarWhatsapp,
    errorGeneral,
    hayCampanas,
    enMarcha,
    cargar,
    guardarPlantilla,
    nuevaVersionDePlantilla,
    retirarPlantilla,
    guardarFirma,
    editarFirma,
    retirarFirma,
    guardarVentana,
    crearCampana,
    revisar,
    enviarPrueba,
    activar,
    comoVa,
    cancelarLaActivacion,
    pausar,
    reanudar,
    detener,
  }
})
