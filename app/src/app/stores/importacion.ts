import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { claveDelError, hayNucleo, invocar } from '@/app/nucleo'

/** Coincide con `arles_core::importacion::CampoImportable`. */
export type CampoImportable =
  | 'nombre'
  | 'apellido'
  | 'nombreCompleto'
  | 'empresa'
  | 'correo'
  | 'whatsapp'
  | 'ignorar'

/**
 * Los campos, en el orden en que se ofrecen.
 *
 * Duplicados desde `CampoImportable::TODOS` del núcleo. El validador de fase
 * comprueba que las dos listas coincidan: si divergieran, el desplegable
 * ofrecería un campo que el núcleo no sabe leer, y la importación se caería
 * después de que el usuario haya revisado todo.
 */
export const CAMPOS: readonly CampoImportable[] = [
  'nombre',
  'apellido',
  'nombreCompleto',
  'empresa',
  'correo',
  'whatsapp',
  'ignorar',
]

/** Coincide con `arles_core::importacion::OrigenDeLaLista`. */
export type OrigenDeLaLista =
  | 'formulario_propio'
  | 'clientes_existentes'
  | 'evento_o_feria'
  | 'directorio_publico'
  | 'otro'

/**
 * Los cinco orígenes, confirmados por Dirección el 18/09/2026.
 *
 * «Otro» va el último a propósito: es la salida, no la primera opción. Sin él,
 * quien no encuentre su caso elegiría el que más se le parezca, y un «clientes
 * existentes» falso es peor prueba que un «otro» honesto.
 */
export const ORIGENES: readonly OrigenDeLaLista[] = [
  'formulario_propio',
  'clientes_existentes',
  'evento_o_feria',
  'directorio_publico',
  'otro',
]

/**
 * El texto que el usuario acepta al importar.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * PROVISIONAL, Y SE DICE EN PANTALLA
 *
 * El texto definitivo está **bloqueado por P-09**, la revisión jurídica.
 * ADR-0013 §1 decide construir el mecanismo igual, con el texto marcado como
 * provisional y **visible como tal**, en vez de esperar sentados.
 *
 * Un aviso que cita una ley abrogada es peor que no citar ninguna: aparenta
 * rigor. Por eso el texto no intenta sonar legal — dice lo que el usuario está
 * afirmando, en sus palabras, y la pantalla avisa de que falta la revisión.
 *
 * Se guarda **íntegro y con su huella**, así que sustituirlo cuando llegue la
 * respuesta no pierde nada de lo ya firmado.
 * ─────────────────────────────────────────────────────────────────────────
 */
export const TEXTO_PROVISIONAL =
  'Declaro que obtuve estos contactos de forma lícita y que puedo ' +
  'escribirles con fines comerciales. [TEXTO PENDIENTE DE REVISIÓN ' +
  'JURÍDICA · P-09]'

/** Lo que devuelve `elegir_archivo_para_importar`. */
export interface ArchivoLeido {
  readonly nombre: string
  readonly encabezados: readonly string[]
  readonly muestra: readonly (readonly string[])[]
  readonly mapeo: readonly CampoImportable[]
  readonly totalDeFilas: number
  /**
   * Las columnas que van a WhatsApp **sin que su encabezado diga que es un
   * móvil**, por su posición.
   *
   * Desde el 22/10/2026 una columna «Teléfono» se reconoce — antes se ignoraba,
   * y la tabla entraba sin un solo número sin que nada lo dijera. Pero
   * «Teléfono» no dice si ese número recibe WhatsApp, y un fijo al que se le
   * manda un mensaje se paga en calidad del número ante Meta. Así que se
   * acierta y se avisa.
   */
  readonly telefonosSinAclarar: readonly number[]
}

/** Una fila que no se va a importar. Coincide con `FilaRechazada`. */
export interface FilaRechazada {
  /** El número **de Excel**: la primera fila de datos es la 2. */
  readonly fila: number
  readonly motivo: string
  readonly referencia: string
}

/** Una dirección del archivo que ya pertenece a alguien. */
export interface Choque {
  readonly fila: number
  readonly direccion: string
  readonly con: string
  readonly dentroDelArchivo: boolean
}

/** Una fila que entra, pero cuya dirección pidió no recibir. */
export interface FilaSuprimida {
  readonly fila: number
  readonly direccion: string
  /** Clave de i18n del motivo. */
  readonly motivo: string
  readonly referencia: string
  /** `true` si TODAS las direcciones de esa fila están suprimidas. */
  readonly sinNingunCanalLibre: boolean
}

/**
 * Un nombre que va a quedar repetido en la lista de contactos (23/10/2026).
 *
 * No es un duplicado: lo que hace único a un contacto es su dirección, nunca
 * su nombre. Lo que avisa es que en la lista van a verse dos filas con el
 * mismo nombre, y que el saludo del correo será el mismo para las dos.
 */
export interface NombreRepetido {
  readonly nombre: string
  /** Cuántos se llamarán así al terminar: los del archivo y los que ya había. */
  readonly cuantos: number
  /** Cuántos de ésos ya estaban en ARLES antes de este archivo. */
  readonly yaEstaban: number
}

/**
 * El informe de lo que pasaría al importar. Coincide con `Analisis`.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * `suprimidas` FALTABA AQUÍ, Y LA PANTALLA NO LO ENSEÑABA
 *
 * El núcleo lo calcula desde la Fase 3 y lo serializa; este tipo no lo
 * declaraba y ninguna pantalla lo leía. El patrón del §9 en su forma de
 * «campo de una respuesta que nadie usa» — el mismo que la 8.6 E encontró con
 * el desenlace de la simulación, y que la comprobación de comandos no caza
 * porque un campo ignorado no es un comando sin llamar.
 *
 * Lo que se perdía: alguien importa un archivo donde diez direcciones ya
 * pidieron no recibir, ARLES las mete —es correcto, el contacto existe— y
 * **no les va a escribir**. Sin el aviso, esas diez cuentan como alcanzables
 * en la cabeza de quien importó.
 */
export interface Analisis {
  readonly filasDeLasListas: readonly number[]
  readonly choques: readonly Choque[]
  readonly rechazadas: readonly FilaRechazada[]
  readonly invalidas: readonly FilaRechazada[]
  readonly suprimidas: readonly FilaSuprimida[]
  readonly nombresRepetidos: readonly NombreRepetido[]
}

export interface ResumenDeImportacion {
  readonly importados: number
  readonly choques: number
  readonly invalidas: number
  readonly totalDeFilas: number
}

/** En qué paso del asistente estamos. */
export type PasoDeImportacion = 'elegir' | 'mapear' | 'revisar' | 'hecho'

/**
 * El asistente de importación: cuatro pasos y un archivo en curso.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * EL ARCHIVO NO VIVE AQUÍ
 *
 * Esta tienda guarda **lo que se enseña**: los encabezados, veinte filas de
 * muestra y el informe. Las filas de verdad —que pueden ser medio millón— se
 * quedan en Rust, del otro lado de la frontera.
 *
 * Mandarlas a la interfaz para «tenerlas a mano» sería pasar medio millón de
 * filas por la IPC y duplicarlas en memoria, para acabar enviándolas de vuelta
 * al confirmar. La pantalla no las necesita: necesita saber cuántas hay y qué
 * va a pasar con ellas.
 * ─────────────────────────────────────────────────────────────────────────
 */
export const useImportacionStore = defineStore('importacion', () => {
  const paso = ref<PasoDeImportacion>('elegir')
  const archivo = ref<ArchivoLeido | null>(null)
  /** El mapeo que el usuario puede corregir. Arranca con el que propone el núcleo. */
  const mapeo = ref<CampoImportable[]>([])
  const origen = ref<OrigenDeLaLista>('formulario_propio')

  /**
   * A qué lista van estos contactos (D-15).
   *
   * ───────────────────────────────────────────────────────────────────────
   * POR QUÉ UN NOMBRE Y NO UN IDENTIFICADOR
   *
   * Porque el camino real tiene dos formas y las dos pasan por aquí: elegir
   * una lista que ya existe —la tabla de marzo del mismo público— o escribir
   * una nueva. Con un identificador haría falta crear la lista en otra
   * pantalla antes de poder importar a ella, y nadie hace eso: carga el
   * archivo y le pone nombre.
   *
   * Rust resuelve las dos con `crear_lista`, que devuelve la existente si ya
   * se llamaba así. Vacío es «sin lista»: los contactos entran a la empresa y
   * ya, que es lo que hacían todas las importaciones hasta el 23/10/2026.
   */
  const lista = ref('')
  const acepta = ref(false)
  const analisis = ref<Analisis | null>(null)
  const resumen = ref<ResumenDeImportacion | null>(null)

  const leyendo = ref(false)
  const descargandoPlantilla = ref(false)
  /** El nombre del archivo guardado, para acusar recibo (§97). */
  const plantillaGuardada = ref<string | null>(null)
  const analizando = ref(false)
  const importando = ref(false)
  const errorGeneral = ref<string | null>(null)

  /** Cuántas filas entrarían. Es una cifra, no un adjetivo (§94). */
  const cuantasEntran = computed(() => analisis.value?.filasDeLasListas.length ?? 0)

  /** Cuántas se quedan fuera, y por qué se cuentan juntas: no van a entrar. */
  const cuantasSeQuedanFuera = computed(() => {
    const a = analisis.value
    if (!a) return 0
    return a.choques.length + a.rechazadas.length + a.invalidas.length
  })

  /**
   * Hace falta al menos un correo o un WhatsApp en el mapeo.
   *
   * Sin ninguno, **ninguna fila tiene a quién escribir** y la importación
   * entera saldría vacía. Se comprueba aquí para poder decírselo antes de
   * analizar, en vez de dejar que vea un informe con cero filas y no entienda
   * por qué.
   */
  const hayAlgunCanal = computed(() =>
    mapeo.value.some((c) => c === 'correo' || c === 'whatsapp'),
  )

  /**
   * Los encabezados de las columnas que **hoy** van a WhatsApp sin decir que su
   * número sea un móvil.
   *
   * Cruza dos cosas y no una: lo que el archivo dijo —que no cambia— y el mapeo
   * **actual**, que el usuario puede corregir. Si cambia esa columna a
   * «Ignorar», el aviso desaparece: un aviso que sigue ahí después de hacer lo
   * que pedía enseña a no leerlos.
   */
  const columnasPorAclarar = computed(() => {
    const a = archivo.value
    if (!a) return []
    return a.telefonosSinAclarar.filter((i) => mapeo.value[i] === 'whatsapp').map(
      (i) => a.encabezados[i] ?? '',
    )
  })

  /**
   * ¿Se puede confirmar?
   *
   * Tres condiciones, y las tres tienen que decirse en pantalla cuando faltan:
   * que haya algo que importar, que se haya aceptado la declaración, y que no
   * se esté importando ya.
   */
  const sePuedeConfirmar = computed(
    () => cuantasEntran.value > 0 && acepta.value && !importando.value,
  )

  function reiniciar(): void {
    paso.value = 'elegir'
    archivo.value = null
    mapeo.value = []
    origen.value = 'formulario_propio'
    acepta.value = false
    analisis.value = null
    resumen.value = null
    errorGeneral.value = null
  }

  function fallo(e: unknown): void {
    errorGeneral.value = claveDelError(e)
  }

  /**
   * Guarda la plantilla vacía donde diga el usuario.
   *
   * Pedida por Dirección el 22/10/2026: la tabla la trae el cliente con sus
   * columnas y en su orden, y esto le da dónde pegar las tres que ARLES usa.
   *
   * No toca el paso del asistente: descargar la plantilla no empieza ninguna
   * importación, y mover al usuario de paso le quitaría de donde estaba.
   */
  async function descargarPlantilla(): Promise<void> {
    descargandoPlantilla.value = true
    errorGeneral.value = null
    plantillaGuardada.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return
      }
      const nombre = await invocar<string | null>(
        'descargar_plantilla_de_contactos',
      )
      // `null` es que cerró el diálogo sin elegir sitio. Cancelar es una
      // decisión, y no se acusa nada.
      if (nombre === null) return
      plantillaGuardada.value = nombre
    } catch (e) {
      fallo(e)
    } finally {
      descargandoPlantilla.value = false
    }
  }

  /** Abre el diálogo del sistema. La ruta no llega hasta aquí: la elige Rust. */
  async function elegirArchivo(): Promise<void> {
    leyendo.value = true
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return
      }
      const leido = await invocar<ArchivoLeido | null>(
        'elegir_archivo_para_importar',
      )
      // `null` es que cerró el diálogo sin elegir. No es un error: cancelar es
      // una decisión, y un aviso rojo por cambiar de opinión sobra.
      if (leido === null) return

      archivo.value = leido
      mapeo.value = [...leido.mapeo]
      analisis.value = null
      paso.value = 'mapear'
    } catch (e) {
      fallo(e)
    } finally {
      leyendo.value = false
    }
  }

  /** Analiza con el mapeo actual. No escribe nada, y se puede repetir. */
  async function analizar(): Promise<void> {
    analizando.value = true
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return
      }
      analisis.value = await invocar<Analisis>('analizar_importacion', {
        mapeo: mapeo.value,
      })
      paso.value = 'revisar'
    } catch (e) {
      fallo(e)
    } finally {
      analizando.value = false
    }
  }

  /** Escribe lo que el análisis marcó como listo. */
  async function confirmar(): Promise<boolean> {
    importando.value = true
    errorGeneral.value = null
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      resumen.value = await invocar<ResumenDeImportacion>(
        'confirmar_importacion',
        {
          mapeo: mapeo.value,
          origen: origen.value,
          textoDelConsentimiento: TEXTO_PROVISIONAL,
          lista: lista.value.trim() || null,
        },
      )
      paso.value = 'hecho'
      return true
    } catch (e) {
      fallo(e)
      return false
    } finally {
      importando.value = false
    }
  }

  /** Suelta el archivo en Rust y vuelve al principio. */
  async function cancelar(): Promise<void> {
    try {
      if (hayNucleo()) await invocar<void>('cancelar_importacion')
    } catch {
      // Cancelar no puede fallar de cara al usuario: ya se está yendo. Si el
      // comando falla, el archivo se suelta igual al cerrar ARLES.
    }
    reiniciar()
  }

  /** Vuelve al paso del mapeo sin soltar el archivo. */
  function volverAMapear(): void {
    paso.value = 'mapear'
    analisis.value = null
  }

  return {
    paso,
    archivo,
    mapeo,
    origen,
    lista,
    acepta,
    analisis,
    resumen,
    leyendo,
    descargandoPlantilla,
    plantillaGuardada,
    analizando,
    importando,
    errorGeneral,
    cuantasEntran,
    cuantasSeQuedanFuera,
    hayAlgunCanal,
    sePuedeConfirmar,
    columnasPorAclarar,
    descargarPlantilla,
    elegirArchivo,
    analizar,
    confirmar,
    cancelar,
    volverAMapear,
    reiniciar,
  }
})
