import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

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

/** El informe de lo que pasaría al importar. Coincide con `Analisis`. */
export interface Analisis {
  readonly filasDeLasListas: readonly number[]
  readonly choques: readonly Choque[]
  readonly rechazadas: readonly FilaRechazada[]
  readonly invalidas: readonly FilaRechazada[]
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
  const acepta = ref(false)
  const analisis = ref<Analisis | null>(null)
  const resumen = ref<ResumenDeImportacion | null>(null)

  const leyendo = ref(false)
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
    errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
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
    acepta,
    analisis,
    resumen,
    leyendo,
    analizando,
    importando,
    errorGeneral,
    cuantasEntran,
    cuantasSeQuedanFuera,
    hayAlgunCanal,
    sePuedeConfirmar,
    elegirArchivo,
    analizar,
    confirmar,
    cancelar,
    volverAMapear,
    reiniciar,
  }
})
