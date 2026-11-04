import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { i18n } from '@/app/i18n'

/**
 * Restaurar un respaldo de un esquema anterior · 04/11/2026
 *
 * ───────────────────────────────────────────────────────────────────────────
 * QUÉ SE VIGILA AQUÍ, Y QUÉ NO
 *
 * NO se comprueba que la puesta al día funcione: eso lo prueban
 * `arles-db::puesta_al_dia` —once pruebas, incluida la que lleva un respaldo
 * V2 hasta hoy pasando por la V3 y la V4— y la de ida y vuelta de
 * `arles-app::restauracion`. Repetirlo aquí sería una segunda copia de la
 * misma regla.
 *
 * Lo que se vigila es lo que **sólo esta pantalla puede equivocar**:
 *
 *   1. **Que un respaldo de una versión anterior no apague el botón.** Es el
 *      defecto exacto que esta entrega viene a cerrar: hasta hoy la pantalla
 *      decía «no se puede restaurar aquí» y no dejaba pulsar. Si el `v-if` se
 *      queda como estaba, el núcleo sabe ponerlo al día y nadie se lo pide.
 *   2. **Que lo diga antes de pulsar.** Tarda más y lo que entra son los datos
 *      del archivo migrados, no el archivo tal cual: callarlo deja a alguien
 *      mirando una pantalla quieta sin saber por qué.
 *   3. **Que un respaldo de una versión MÁS NUEVA sí apague el botón, y que el
 *      texto mande a actualizar ARLES** en vez de a buscar otra copia. Es el
 *      único caso sin salida, y mandar a buscar otra copia sería mandar a
 *      repetir el intento — la lección del error del archivo de historial.
 * ───────────────────────────────────────────────────────────────────────────
 */

const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' },
      Teleport: true,
    },
  },
}

function conNucleo(respuestas: Record<string, unknown>) {
  const invocar = vi.fn((comando: string, args?: Record<string, unknown>) => {
    if (!(comando in respuestas)) {
      return Promise.reject(
        new Error(`comando sin respuesta en la prueba: ${comando} ${JSON.stringify(args)}`),
      )
    }
    return Promise.resolve(respuestas[comando])
  })
  vi.doMock('@/app/nucleo', async () => {
    const real = await vi.importActual<typeof import('@/app/nucleo')>('@/app/nucleo')
    return { ...real, hayNucleo: () => true, invocar }
  })
  return invocar
}

/** Lo que `revisar_respaldo` contesta, con lo justo para pintar la pantalla. */
function revisado(extra: Record<string, unknown>) {
  return {
    nombreDelArchivo: 'arles-20260921.arles',
    creadoEn: '2026-09-21T09:30:00Z',
    alcance: 'completo',
    esquemaDelRespaldo: 'V11',
    esquemaDeLaBase: 'V16',
    llevaContrasena: false,
    sePuedeRestaurar: true,
    hayQuePonerloAlDia: false,
    migracionesQueFaltan: 0,
    esDeUnArlesPosterior: false,
    tablas: [{ tabla: 'contact', ahora: 10, enElRespaldo: 12 }],
    totalDeFilas: 12,
    noVuelven: [],
    filasQueNoVuelven: 0,
    bajasAhora: 3,
    bajasEnElRespaldo: 2,
    bajasAlMenos: 3,
    cuentasAReconectar: 0,
    ...extra,
  }
}

/** Monta la pantalla y pulsa «elegir», que es lo que trae el preflight. */
async function pantallaCon(respuesta: Record<string, unknown>) {
  conNucleo({ revisar_respaldo: respuesta })
  const { default: Pantalla } = await import('@/app/pantallas/PantallaRestaurar.vue')
  const w = mount(Pantalla, entorno)
  await flushPromises()
  // El primer botón de la pantalla es el de elegir archivo.
  const botones = w.findAll('button')
  const elegir = botones[0]
  expect(elegir, 'la pantalla no tiene el botón de elegir archivo').toBeDefined()
  await elegir?.trigger('click')
  await flushPromises()
  return w
}

describe('restaurar un respaldo de otro esquema', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
    vi.clearAllMocks()
  })

  it('uno de una versión anterior se puede restaurar, y la pantalla lo dice', async () => {
    const w = await pantallaCon(
      revisado({ hayQuePonerloAlDia: true, migracionesQueFaltan: 5 }),
    )
    const texto = w.text()

    expect(
      texto,
      'la pantalla no avisa de que el respaldo se va a poner al día',
    ).toContain('es de una versión anterior')
    // La cifra, no un adjetivo (§94).
    expect(texto, 'no dice cuántos cambios de estructura se le van a aplicar').toContain('5')

    // Y el botón de restaurar NO está apagado: es el defecto que esta entrega
    // cierra. Se busca por su texto porque el orden de los botones cambia con
    // los avisos que haya arriba.
    const restaurar = w.findAll('button').find((b) => b.text().includes('Restaurar y reemplazar'))
    expect(restaurar, 'no está el botón de restaurar').toBeDefined()
    expect(
      restaurar?.attributes('disabled'),
      'un respaldo de una versión anterior dejó el botón apagado: ARLES sabe ponerlo al día',
    ).toBeUndefined()
  })

  it('uno de una versión más nueva no se puede, y manda a actualizar ARLES', async () => {
    const w = await pantallaCon(
      revisado({
        esquemaDelRespaldo: 'V20',
        sePuedeRestaurar: false,
        esDeUnArlesPosterior: true,
      }),
    )
    const texto = w.text()

    expect(texto, 'no dice que el respaldo es de una versión más nueva').toContain(
      'una versión más nueva',
    )
    expect(texto, 'no manda a actualizar ARLES, que es la única salida').toContain('Actualiza ARLES')
    // Y NO manda a buscar otra copia, que sería mandar a repetir el intento.
    expect(texto, 'manda a buscar otra copia, que no arreglaría nada').not.toContain(
      'Prueba con otra copia',
    )
    expect(
      w.findAll('button').some((b) => b.text().includes('Restaurar y reemplazar')),
      'con un respaldo del futuro no debería haber botón de restaurar',
    ).toBe(false)
  })

  it('uno del esquema de hoy no habla de ponerse al día', async () => {
    const w = await pantallaCon(revisado({ esquemaDelRespaldo: 'V16' }))
    expect(
      w.text(),
      'habla de poner al día un respaldo que ya está al día',
    ).not.toContain('es de una versión anterior')
  })
})
