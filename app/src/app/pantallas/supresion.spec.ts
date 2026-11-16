import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { i18n } from '@/app/i18n'

/**
 * Apuntar una baja a mano · 23/10/2026
 *
 * ───────────────────────────────────────────────────────────────────────────
 * QUÉ SE VIGILA AQUÍ, Y QUÉ NO
 *
 * NO se comprueba que la baja funcione: eso lo prueban
 * `arles-db::supresion` —que la fila se escribe con sus notas y sin contacto
 * detrás— y `arles-motor::bucle`, que comprueba que quien está en la lista no
 * recibe nada. Duplicarlo aquí sería una segunda copia de la misma regla.
 *
 * Lo que se vigila es lo que **sólo esta pantalla puede equivocar**, y son
 * tres cosas que ya se equivocaron antes en otras pantallas:
 *
 *   1. **Que el formulario esté cuando la lista está vacía.** Es el caso que
 *      importa —la primera baja se apunta cuando no hay ninguna— y es
 *      exactamente el que se pierde si el formulario vive dentro del `v-if`
 *      de los estados de la lista. Con cero bajas la pantalla enseña su
 *      estado vacío, y ahí no habría dónde escribir.
 *   2. **Que «ya estaba» no se pinte como un fallo.** Las dos respuestas son
 *      éxito, y mandar a arreglar algo que está bien gasta la confianza en
 *      todos los demás avisos (es el H-04 de la auditoría G, en otra
 *      pantalla).
 *   3. **Que se mande la dirección tal cual se escribió.** Si esta pantalla la
 *      normalizara, la baja quedaría apuntada en una forma que el motor no
 *      encuentra al comparar — peor que no apuntarla, porque parece hecha.
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

/**
 * Finge el núcleo, con una respuesta **por comando**.
 *
 * Un `mockResolvedValue` único no sirve aquí: la pantalla pide la lista al
 * montarse y el comando de la baja al pulsar, y los dos devuelven formas
 * distintas. Con una respuesta única, el segundo recibiría la página de la
 * lista y la prueba pasaría sin ejercitar nada.
 */
function conNucleo(respuestas: Record<string, unknown>) {
  const invocar = vi.fn((comando: string, args?: Record<string, unknown>) => {
    if (!(comando in respuestas)) {
      return Promise.reject(
        new Error(
          `comando sin respuesta en la prueba: ${comando} ${JSON.stringify(args)}`,
        ),
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

const SIN_BAJAS = { supresiones: [], total: 0 }

/** Una fila cualquiera, para el caso de la lista con contenido. */
const UNA_BAJA = {
  supresiones: [
    {
      id: 'b1',
      canal: 'email',
      direccion: 'ana@empresa.mx',
      motivo: 'supresion.motivo.baja',
      sePuedeLevantar: false,
      deQuien: 'Ana Ruiz',
      notas: '',
      creada: '2026-10-20T10:00:00Z',
      global: false,
    },
  ],
  total: 1,
}

/** Escribe la dirección en el formulario de apuntar y pulsa el botón. */
async function apuntar(
  w: ReturnType<typeof mount>,
  direccion: string,
): Promise<void> {
  // Por el contenedor del formulario y no por el índice de `findAll('input')`:
  // con la lista llena hay antes la caja de buscar, así que un índice fijo
  // mediría una prueba u otra según el caso.
  const campos = w.find('.supresion__apuntar').findAll('input')
  const primero = campos[0]
  expect(primero, 'el formulario de apuntar no tiene ningún campo').toBeDefined()
  await primero?.setValue(direccion)
  await w.find('.supresion__apuntar-pie button').trigger('click')
  await flushPromises()
}

describe('apuntar una baja a mano', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
    vi.clearAllMocks()
  })

  /**
   * EL CASO QUE ESTA ENTREGA EXISTÍA PARA TENER.
   *
   * Con cero bajas, la pantalla enseña su estado vacío. Si el formulario
   * viviera dentro de ese `v-if`, no habría forma de apuntar la primera.
   */
  it('con la lista vacía, el formulario sigue estando', async () => {
    conNucleo({ listar_supresiones: SIN_BAJAS })
    const { default: Pantalla } = await import('./PantallaSupresion.vue')
    const w = mount(Pantalla, entorno)
    await flushPromises()

    expect(w.text(), 'no se enseñó el estado vacío').toContain(
      'Nadie ha pedido darse de baja',
    )
    expect(
      w.find('.supresion__apuntar').exists(),
      'con la lista vacía no hay dónde apuntar la primera baja',
    ).toBe(true)
  })

  it('apuntada, lo dice y manda la dirección tal cual se escribió', async () => {
    const invocar = conNucleo({
      listar_supresiones: SIN_BAJAS,
      suprimir_direccion: true,
    })
    const { default: Pantalla } = await import('./PantallaSupresion.vue')
    const w = mount(Pantalla, entorno)
    await flushPromises()
    await apuntar(w, '  Alguien@Reenviado.MX  ')

    expect(w.text()).toContain('Apuntada')

    const llamada = invocar.mock.calls.find((c) => c[0] === 'suprimir_direccion')
    expect(llamada, 'no se llamó a suprimir_direccion').toBeDefined()
    // Tal cual, con sus mayúsculas y sus espacios: la forma que se compara la
    // decide el validador del núcleo, no esta pantalla.
    expect(llamada?.[1]).toMatchObject({
      canal: 'email',
      direccion: '  Alguien@Reenviado.MX  ',
      motivo: 'unsubscribe',
    })
  })

  /**
   * «Ya estaba» es un éxito, y se cuenta distinto.
   *
   * Los dos desenlaces son correctos, y por eso es fácil pintarlos igual —
   * y entonces alguien apunta dos veces la misma baja sin saberlo. Se
   * comprueba además que **no** se pinta como avería: un aviso rojo mandaría
   * a arreglar algo que está bien.
   */
  it('si ya estaba, lo dice y no lo pinta como un fallo', async () => {
    conNucleo({
      listar_supresiones: UNA_BAJA,
      suprimir_direccion: false,
    })
    const { default: Pantalla } = await import('./PantallaSupresion.vue')
    const w = mount(Pantalla, entorno)
    await flushPromises()
    await apuntar(w, 'ana@empresa.mx')

    expect(w.text()).toContain('ya estaba en la lista')
    expect(w.text()).not.toContain('Apuntada. ARLES ya no escribe')

    // Se mira el acuse, no la página entera: la pantalla tiene otros avisos
    // legítimos, y una aserción sobre todo el HTML no distingue «este acuse
    // está mal pintado» de «hay algo rojo en otra parte».
    const acuse = w.find('.supresion__apuntar-pie + .aviso, .modulo .aviso')
    expect(acuse.exists(), 'no se pintó ningún acuse').toBe(true)
    expect(acuse.html(), 'un «ya estaba» se pinta como una avería').not.toContain(
      't-peligro',
    )
  })

  /**
   * **El botón apagado dice por qué** (§95).
   *
   * Un control que no responde y no se explica se lee como una avería. Es el
   * mismo defecto que la 8.2c tuvo con «Repite la contraseña» y la 8.3c con
   * «Restaurar y reemplazar».
   */
  it('sin dirección, el botón está apagado y dice por qué', async () => {
    conNucleo({ listar_supresiones: SIN_BAJAS })
    const { default: Pantalla } = await import('./PantallaSupresion.vue')
    const w = mount(Pantalla, entorno)
    await flushPromises()

    const boton = w.find('.supresion__apuntar-pie button')
    expect(boton.attributes('disabled')).toBeDefined()
    expect(w.find('.supresion__apuntar-pie').text()).toContain(
      'Escribe la dirección',
    )
  })
})
