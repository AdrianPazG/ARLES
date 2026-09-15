import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

/** Coincide con `arles_core::empresa::DatosDeEmpresa`. */
export interface DatosDeEmpresa {
  readonly nombreComercial: string
  readonly pais: string
  readonly zonaHoraria: string
  readonly correoCorporativo: string
  readonly sitioWeb: string | null
}

/** Lo que el formulario envía. Coincide con `BorradorDeEmpresa`. */
export interface BorradorDeEmpresa {
  nombreComercial: string
  pais: string
  zonaHoraria: string
  correoCorporativo: string
  sitioWeb: string
}

/** Coincide con `arles_core::onboarding::EstadoDePaso`. */
export interface PasoDeAlta {
  readonly paso: string
  readonly clave: string
  readonly completado: boolean
  readonly disponible: boolean
  readonly ruta: string | null
  readonly entrega: string
}

export interface ListaDeAlta {
  readonly pasos: readonly PasoDeAlta[]
  readonly completados: number
  readonly total: number
  readonly siguiente: string | null
}

export interface ConfiguracionDeEmpresa {
  readonly empresa: DatosDeEmpresa | null
  readonly zonas: readonly string[]
  readonly paises: readonly string[]
  readonly onboarding: ListaDeAlta
}

/**
 * Lo que se muestra cuando no hay núcleo: `vite dev` y las sondas de navegador.
 *
 * **Las listas están duplicadas aquí, y eso es una costura peligrosa.** En
 * cuanto alguien añada una zona en `arles-core` y no aquí, las sondas y las
 * capturas de revisión enseñarían un desplegable que no es el del producto.
 *
 * Por eso el validador de fase compara las dos listas y falla si divergen. No
 * se confía en que alguien se acuerde: se comprueba.
 */
const RESERVA_SIN_NUCLEO: ConfiguracionDeEmpresa = {
  empresa: null,
  zonas: [
    'America/Mexico_City',
    'America/Cancun',
    'America/Merida',
    'America/Monterrey',
    'America/Matamoros',
    'America/Chihuahua',
    'America/Ojinaga',
    'America/Mazatlan',
    'America/Bahia_Banderas',
    'America/Hermosillo',
    'America/Tijuana',
    'UTC',
  ],
  paises: ['MX'],
  onboarding: {
    pasos: [
      { paso: 'empresa', clave: 'empresa', completado: false, disponible: true, ruta: '/ajustes', entrega: '3.1' },
      { paso: 'remitente', clave: 'remitente', completado: false, disponible: false, ruta: null, entrega: '5' },
      { paso: 'contactos', clave: 'contactos', completado: false, disponible: false, ruta: null, entrega: '3.2' },
      { paso: 'plantilla', clave: 'plantilla', completado: false, disponible: false, ruta: null, entrega: '6' },
      { paso: 'ventanaDeEjecucion', clave: 'ventana', completado: false, disponible: false, ruta: null, entrega: '6' },
      { paso: 'primeraCampana', clave: 'campana', completado: false, disponible: false, ruta: null, entrega: '6' },
    ],
    completados: 0,
    total: 6,
    siguiente: 'empresa',
  },
}

/**
 * Configuración de empresa y lista de alta.
 *
 * **La validación no vive aquí.** El formulario podría comprobar lo mismo que
 * el núcleo y avisar antes, pero entonces habría dos reglas que mantener
 * sincronizadas, y el día que divergieran el usuario vería un campo en verde
 * que el núcleo rechaza. Quien decide es el núcleo; esta tienda pinta lo que
 * responde.
 */
export const useEmpresaStore = defineStore('empresa', () => {
  const configuracion = ref<ConfiguracionDeEmpresa>(RESERVA_SIN_NUCLEO)
  const cargada = ref(false)
  const cargando = ref(false)
  const guardando = ref(false)
  /** Clave de i18n del error de cada campo, por nombre de campo. */
  const erroresDeCampo = ref<Record<string, string>>({})
  /** Clave de i18n de un error que no es de un campo concreto. */
  const errorGeneral = ref<string | null>(null)
  const guardadoConExito = ref(false)

  const empresa = computed(() => configuracion.value.empresa)
  const onboarding = computed(() => configuracion.value.onboarding)
  const configurada = computed(() => configuracion.value.empresa !== null)

  async function cargar(): Promise<void> {
    if (cargada.value || cargando.value) return
    cargando.value = true
    try {
      if (hayNucleo()) {
        configuracion.value = await invocar<ConfiguracionDeEmpresa>(
          'configuracion_de_empresa',
        )
      }
      cargada.value = true
    } catch (e) {
      errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
    } finally {
      cargando.value = false
    }
  }

  /**
   * Guarda el formulario.
   *
   * @returns `true` si se guardó. `false` deja los errores puestos en
   * `erroresDeCampo` o en `errorGeneral`, que es lo que la pantalla pinta.
   */
  async function guardar(borrador: BorradorDeEmpresa): Promise<boolean> {
    guardando.value = true
    erroresDeCampo.value = {}
    errorGeneral.value = null
    guardadoConExito.value = false
    try {
      if (!hayNucleo()) {
        // Sin núcleo no se guarda nada, y se dice. Fingir un guardado en
        // `vite dev` sería enseñar una pantalla que miente.
        errorGeneral.value = 'error.app.sin_nucleo'
        return false
      }
      configuracion.value = await invocar<ConfiguracionDeEmpresa>('guardar_empresa', {
        borrador,
      })
      cargada.value = true
      guardadoConExito.value = true
      return true
    } catch (e) {
      if (esErrorIpc(e)) {
        const campos = e.campos ?? []
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
      return false
    } finally {
      guardando.value = false
    }
  }

  /**
   * Descarta el aviso de guardado.
   *
   * Lo llama la pantalla en cuanto el usuario toca un campo: «Configuración
   * guardada» junto a un formulario que ya se está editando afirma algo que ha
   * dejado de ser cierto.
   */
  function descartarAviso(): void {
    guardadoConExito.value = false
  }

  return {
    descartarAviso,
    configuracion,
    empresa,
    onboarding,
    configurada,
    cargada,
    cargando,
    guardando,
    erroresDeCampo,
    errorGeneral,
    guardadoConExito,
    cargar,
    guardar,
  }
})
