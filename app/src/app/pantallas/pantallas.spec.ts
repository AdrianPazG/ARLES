import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useEmpresaStore } from '@/app/stores/empresa'

import PantallaInicio from './PantallaInicio.vue'

/**
 * Lo que las pantallas **no** deben afirmar.
 *
 * Salió de la auditoría de la 3.1: las aserciones de las capas decían que todo
 * estaba bien, y lo que fallaba era lo que la pantalla comunicaba.
 */
describe('Inicio no afirma lo que no sabe', () => {
  beforeEach(() => setActivePinia(createPinia()))

  const montar = () =>
    mount(PantallaInicio, {
      global: {
        plugins: [i18n],
        stubs: { RouterLink: { template: '<a><slot /></a>' } },
      },
    })

  it('sin error, enseña los seis pasos', () => {
    const w = montar()
    expect(w.findAll('.paso')).toHaveLength(6)
    expect(w.text()).toContain('0 de 6')
  })

  /**
   * Si los datos no se pudieron leer, la lista de reserva —«0 de 6», todo
   * pendiente— **no es el estado real**. Enseñarla haría que alguien volviera a
   * configurar lo que ya tenía configurado.
   */
  it('si no pudo leer los datos, no enseña la lista: enseña el error', async () => {
    const empresa = useEmpresaStore()
    empresa.errorGeneral = 'error.db.sqlite'
    const w = montar()
    await w.vm.$nextTick()

    expect(w.findAll('.paso'), 'no debe pintar una lista que no sabe si es cierta').toHaveLength(0)
    expect(w.text()).not.toContain('0 de 6')

    const alerta = w.find('[role="alert"]')
    expect(alerta.exists(), 'el error tiene que anunciarse').toBe(true)
    // Las tres partes del §95, no sólo la primera.
    expect(alerta.text()).toContain('Ocurrió un error al acceder a los datos')
    expect(alerta.text()).toContain('Cierra ARLES')
    expect(alerta.text()).toContain('se revirtieron')
  })

  /** Un paso que aún no existe lo dice y no se puede pulsar. */
  it('los pasos no construidos no son enlaces y declaran su entrega', () => {
    const w = montar()
    const pasos = w.findAll('.paso')
    expect(pasos[0]?.find('a').exists(), 'el primero sí lleva a Ajustes').toBe(true)
    for (const paso of pasos.slice(1)) {
      expect(paso.find('a').exists(), 'no debe ser un enlace').toBe(false)
      expect(paso.text()).toContain('Llega en la entrega')
    }
  })
})
