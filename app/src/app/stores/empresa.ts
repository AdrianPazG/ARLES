import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { claveDelError, esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'

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

/**
 * Coincide con `arles_core::onboarding::EstadoDePaso`.
 *
 * **Llevaba dos campos más hasta el 28/10/2026**, y los dos se fueron con el
 * arreglo de D-01: `disponible`, que era `ruta !== null` dicho otra vez, y
 * `entrega`, que sacaba el número de entrega del roadmap a la pantalla de
 * alguien que no trabaja aquí —«Llega en la entrega 6»—.
 */
export interface PasoDeAlta {
  readonly paso: string
  readonly clave: string
  readonly completado: boolean
  readonly ruta: string | null
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
      // Las rutas son las mismas que las de `PasoDeOnboarding::ruta` en el
      // núcleo, y lo comprueba `las_rutas_del_alta_coinciden_con_el_nucleo`
      // en la fase 3. Hasta el 28/10/2026 no lo comprobaba nada: las tres
      // últimas decían aquí `ruta: null` mientras el núcleo daba pantalla a
      // dos de ellas desde el 29/09, así que `vite dev` y las sondas pintaban
      // una lista de alta que no era la del producto.
      { paso: 'empresa', clave: 'empresa', completado: false, ruta: '/inicio/configuracion' },
      { paso: 'remitente', clave: 'remitente', completado: false, ruta: '/remitentes' },
      { paso: 'contactos', clave: 'contactos', completado: false, ruta: '/contactos' },
      { paso: 'plantilla', clave: 'plantilla', completado: false, ruta: '/campanas/plantillas' },
      { paso: 'ventanaDeEjecucion', clave: 'ventana', completado: false, ruta: '/campanas/nueva' },
      { paso: 'primeraCampana', clave: 'campana', completado: false, ruta: '/campanas' },
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

  /**
   * Pide la configuración y **la lista de los seis pasos**.
   *
   * ── POR QUÉ HAY UN `refrescar` Y ESTO NO BASTA (H-08, 16/10/2026) ───────
   *
   * Dirección dio de alta su cuenta de correo, volvió a Inicio y la lista
   * seguía diciendo **«1 de 6»**, con «Conectar una cuenta remitente» sin su
   * palomita — mientras la pantalla de Remitentes enseñaba esa misma cuenta
   * como «Lista para enviar».
   *
   * El recuento es correcto en la base: `recuento_de_alta` cuenta
   * `email_account` y habría devuelto 1. Lo que pasa es que **este `cargar`
   * no vuelve a preguntar**: el `return` de abajo lo salta en cuanto hay
   * datos, así que la lista se queda con la foto del arranque.
   *
   * Es la hermana del H-01: allí un aviso sobrevivía a lo que lo resolvía;
   * aquí es un contador que no sube cuando acabas de hacer el paso, que hace
   * dudar de si de verdad quedó hecho.
   */
  async function cargar(): Promise<void> {
    if (cargada.value || cargando.value) return
    cargando.value = true
    // El aviso de la carga anterior se borra **antes** de reintentar. Ver el
    // porqué entero en `remitentes.ts` y en `avisos-al-recargar.spec.ts`: un
    // error que ya no existe seguía dando instrucciones (H-01, 14/10/2026).
    errorGeneral.value = null
    try {
      if (hayNucleo()) {
        configuracion.value = await invocar<ConfiguracionDeEmpresa>(
          'configuracion_de_empresa',
        )
      }
      cargada.value = true
    } catch (e) {
      errorGeneral.value = claveDelError(e)
    } finally {
      cargando.value = false
    }
  }
  /**
   * Vuelve a pedirla aunque ya esté cargada.
   *
   * Es lo que hace falta tras dar de alta un remitente, un contacto o una
   * plantilla: el paso correspondiente acaba de completarse y la lista tiene
   * que enterarse. `cargar` no sirve porque se salta a sí mismo (H-08).
   */
  async function refrescar(): Promise<void> {
    cargada.value = false
    await cargar()
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
        errorGeneral.value = claveDelError(e)
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
    refrescar,
    guardar,
  }
})
