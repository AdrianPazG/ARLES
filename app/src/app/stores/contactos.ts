import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { claveDelError, esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/** Coincide con `arles_core::canal::Canal`. */
export type Canal = 'email' | 'whatsapp'

/**
 * Un canal ya validado, tal y como lo devuelve el núcleo.
 *
 * Llegan **las dos formas**: `valorRaw` es lo que el usuario escribió y es lo
 * que se le enseña; `valorNormalizado` es la forma canónica y es con la que se
 * compara. Enseñar la normalizada haría que alguien que escribió
 * `Ana@Empresa.MX` viera `ana@empresa.mx` y creyera que ARLES le cambió el
 * dato.
 */
export interface CanalValidado {
  readonly canal: Canal
  readonly valorRaw: string
  readonly valorNormalizado: string
  readonly principal: boolean
}

/** Un canal tal y como se escribe en el formulario. Coincide con `BorradorDeCanal`. */
export interface BorradorDeCanal {
  canal: Canal
  valor: string
  principal: boolean
}

export interface BorradorDeContacto {
  nombre: string
  apellido: string
  empresa: string
  canales: BorradorDeCanal[]
}

/** Coincide con `arles_app::comandos::ContactoParaLaPantalla`. */
export interface Contacto {
  readonly id: string
  readonly nombre: string
  readonly apellido: string
  readonly empresa: string
  readonly canales: readonly CanalValidado[]
  readonly creadoEn: string
  readonly actualizadoEn: string
}

interface PaginaDeContactos {
  readonly contactos: readonly Contacto[]
  readonly total: number
  readonly suprimidas: readonly string[]
}

/** Tope de canales por contacto. Coincide con `arles_core::contacto::MAX_CANALES`. */
export const MAX_CANALES = 10

/**
 * Un error de un canal concreto del formulario.
 *
 * Lleva **índice**: con hasta diez canales, decir «un canal es inválido» sin
 * decir cuál obliga a repasarlos todos (§95).
 */
export interface ErrorDeCanal {
  readonly campo: string
  readonly indice: number | null
  readonly clave: string
}

/** Un contacto vacío, para el formulario de alta. */
export function contactoEnBlanco(): BorradorDeContacto {
  return {
    nombre: '',
    apellido: '',
    empresa: '',
    // Arranca con un correo: es el canal que casi siempre se escribe primero,
    // y un formulario que empieza sin ninguna fila obliga a pulsar «añadir»
    // antes de poder escribir nada.
    canales: [{ canal: 'email', valor: '', principal: true }],
  }
}

/**
 * Convierte un contacto guardado en un borrador editable.
 *
 * Usa `valorRaw` y no la forma normalizada: al abrir «editar», el campo tiene
 * que decir lo que el usuario escribió, no lo que el núcleo guardó para
 * comparar.
 */
export function aBorrador(c: Contacto): BorradorDeContacto {
  return {
    nombre: c.nombre,
    apellido: c.apellido,
    empresa: c.empresa,
    canales: c.canales.map((k) => ({
      canal: k.canal,
      valor: k.valorRaw,
      principal: k.principal,
    })),
  }
}

/** El canal principal de un tipo, que es lo que enseña la tabla (L-14). */
export function principalDe(
  c: Pick<Contacto, 'canales'>,
  canal: Canal,
): CanalValidado | null {
  return c.canales.find((k) => k.canal === canal && k.principal) ?? null
}

/** Nombre completo, o el marcador de que no hay ninguno. */
export function nombreVisible(c: Pick<Contacto, 'nombre' | 'apellido'>): string {
  return [c.nombre, c.apellido].filter((t) => t.length > 0).join(' ')
}

/**
 * La lista de contactos y sus altas, ediciones y bajas.
 *
 * **La validación no vive aquí**, igual que en la tienda de empresa: quien
 * decide si un contacto es válido es el núcleo. El formulario se limita a
 * pintar lo que responde. Dos validaciones son dos reglas que mantener
 * iguales, y el día que divergen el formulario aprueba lo que el núcleo
 * rechaza.
 */
export const useContactosStore = defineStore('contactos', () => {
  const contactos = ref<readonly Contacto[]>([])
  const total = ref(0)
  /**
   * Direcciones de la página que están en la lista de supresión.
   *
   * **Es un aviso, no un bloqueo** (L-13). Quien decide si se escribe o no es
   * la campaña, en su comprobación previa. Duplicar aquí esa decisión sería
   * tener dos reglas que mantener iguales.
   */
  const suprimidas = ref<readonly string[]>([])
  const cargada = ref(false)
  const cargando = ref(false)
  const guardando = ref(false)
  const desde = ref(0)

  /**
   * El filtro de la lista (R-26). Los cuatro criterios son combinables.
   *
   * ── POR QUÉ VIVE EN LA TIENDA Y NO EN LA PANTALLA ──
   *
   * Porque la página y el total se piden **con él**, y un filtro que viviera
   * sólo en el componente se perdería al navegar a la ficha y volver: la lista
   * volvería entera sin decir que el filtro se fue, que es lo que hace dudar
   * de un total.
   *
   * ── Y POR QUÉ NO SE FILTRA EN EL NAVEGADOR ──
   *
   * Porque sólo están las cien filas de la página. Filtrar aquí daría «3 de
   * 100» sobre un total de 455 470, y los otros 455 370 no se habrían mirado:
   * un filtro que no encuentra lo que hay es peor que no tenerlo.
   */
  const filtroTexto = ref('')
  const filtroDireccion = ref('')
  const filtroCanal = ref<Canal | ''>('')
  /** `''` es «todos»; `'si'` y `'no'` son los dos cortes. */
  const filtroConAlgunaBaja = ref<'' | 'si' | 'no'>('')

  const hayFiltro = computed(
    () =>
      filtroTexto.value.trim() !== '' ||
      filtroDireccion.value.trim() !== '' ||
      filtroCanal.value !== '' ||
      filtroConAlgunaBaja.value !== '',
  )

  function limpiarFiltro(): void {
    filtroTexto.value = ''
    filtroDireccion.value = ''
    filtroCanal.value = ''
    filtroConAlgunaBaja.value = ''
  }

  /** Clave de i18n del error de cada canal, por índice. */
  const erroresDeCanal = ref<Record<number, string>>({})
  /** Clave de i18n del error de cada campo del contacto (nombre, apellido…). */
  const erroresDeCampo = ref<Record<string, string>>({})
  /** Error que no pertenece a ningún campo: la base, una dirección repetida… */
  const errorGeneral = ref<string | null>(null)
  /**
   * La dirección que provocó el conflicto, si lo hubo.
   *
   * Va aparte de la clave de i18n porque el texto la interpola: «la dirección
   * X ya está registrada». Sin ella el mensaje obliga a adivinar cuál de los
   * diez canales se repitió.
   */
  const direccionEnConflicto = ref<string | null>(null)

  const hayContactos = computed(() => total.value > 0)
  const suprimidasEnPagina = computed(() => new Set(suprimidas.value))

  /** ¿Alguno de los canales de este contacto está suprimido? */
  function estaSuprimido(c: Contacto): boolean {
    return c.canales.some((k) => suprimidasEnPagina.value.has(k.valorNormalizado))
  }

  function limpiarErrores(): void {
    erroresDeCanal.value = {}
    erroresDeCampo.value = {}
    errorGeneral.value = null
    direccionEnConflicto.value = null
  }

  async function cargar(inicio = 0): Promise<void> {
    if (cargando.value) return
    cargando.value = true
    errorGeneral.value = null
    try {
      if (hayNucleo()) {
        // Un campo vacío se manda como `null` y no como `''`: una cadena
        // vacía es un `LIKE '%%'`, que casa con todo y **recorre la tabla
        // entera** para descubrirlo — 867 ms medidos con 455 470 contactos,
        // pagados por un campo en blanco. La frontera también lo recorta, por
        // la regla de no fiarse de la webview.
        const oNada = (v: string): string | null => (v.trim() === '' ? null : v.trim())
        const pagina = await invocar<PaginaDeContactos>('listar_contactos', {
          desde: inicio,
          texto: oNada(filtroTexto.value),
          direccion: oNada(filtroDireccion.value),
          canal: filtroCanal.value === '' ? null : filtroCanal.value,
          conAlgunaBaja: filtroConAlgunaBaja.value === '' ? null : filtroConAlgunaBaja.value === 'si',
        })
        contactos.value = pagina.contactos
        total.value = pagina.total
        suprimidas.value = pagina.suprimidas
        desde.value = inicio
      }
      cargada.value = true
    } catch (e) {
      errorGeneral.value = claveDelError(e)
    } finally {
      cargando.value = false
    }
  }

  /** Vuelve a pedir la página actual. Tras guardar, el total pudo cambiar. */
  async function recargar(): Promise<void> {
    cargando.value = false
    await cargar(desde.value)
  }

  /**
   * Traduce un error del núcleo a lo que la pantalla puede pintar.
   *
   * Se reparte en tres sitios distintos a propósito: los canales se marcan por
   * índice, los campos por nombre, y lo que no es ni una cosa ni la otra va al
   * aviso general. Meterlo todo en un mensaje obligaría a revisar el
   * formulario entero (§95).
   */
  function repartirError(e: unknown): void {
    limpiarErrores()
    if (!esErrorIpc(e)) {
      errorGeneral.value = claveDelError(e)
      return
    }

    const canales = (e as { canales?: readonly ErrorDeCanal[] }).canales ?? []
    for (const c of canales) {
      if (c.indice !== null && c.indice !== undefined) {
        erroresDeCanal.value[c.indice] = c.clave
      } else {
        erroresDeCampo.value[c.campo] = c.clave
      }
    }

    // La dirección repetida no es un error de forma: el canal está bien
    // escrito, lo que pasa es que ya es de otro contacto. Por eso llega sin
    // índice y con la dirección dentro del detalle.
    if (e.clave === 'error.db.direccion_en_uso') {
      direccionEnConflicto.value = e.detalle
    }

    if (canales.length === 0) errorGeneral.value = e.clave
  }

  async function crear(borrador: BorradorDeContacto): Promise<boolean> {
    guardando.value = true
    limpiarErrores()
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      await invocar<string>('crear_contacto', { borrador })
      await recargar()
      return true
    } catch (e) {
      repartirError(e)
      return false
    } finally {
      guardando.value = false
    }
  }

  async function editar(id: string, borrador: BorradorDeContacto): Promise<boolean> {
    guardando.value = true
    limpiarErrores()
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      await invocar<void>('editar_contacto', { id, borrador })
      await recargar()
      return true
    } catch (e) {
      repartirError(e)
      return false
    } finally {
      guardando.value = false
    }
  }

  async function borrar(id: string): Promise<boolean> {
    guardando.value = true
    limpiarErrores()
    try {
      if (!hayNucleo()) {
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      await invocar<void>('borrar_contacto', { id })
      await recargar()
      return true
    } catch (e) {
      repartirError(e)
      return false
    } finally {
      guardando.value = false
    }
  }

  return {
    contactos,
    total,
    suprimidas,
    cargada,
    cargando,
    guardando,
    desde,
    filtroTexto,
    filtroDireccion,
    filtroCanal,
    filtroConAlgunaBaja,
    hayFiltro,
    limpiarFiltro,
    erroresDeCanal,
    erroresDeCampo,
    errorGeneral,
    direccionEnConflicto,
    hayContactos,
    estaSuprimido,
    limpiarErrores,
    cargar,
    recargar,
    crear,
    editar,
    borrar,
  }
})
