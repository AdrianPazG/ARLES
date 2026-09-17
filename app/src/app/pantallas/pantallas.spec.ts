import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useEmpresaStore } from '@/app/stores/empresa'

import PantallaAjustes from './PantallaAjustes.vue'
import PantallaInicio from './PantallaInicio.vue'

/**
 * Lo que las pantallas **no** deben afirmar.
 *
 * Salió de la auditoría de la 3.1: las aserciones de las capas decían que todo
 * estaba bien, y lo que fallaba era lo que la pantalla comunicaba.
 *
 * Al rediseñar Inicio como panel modular, la lista de los seis pasos se mudó a
 * Ajustes. Lo que se comprueba aquí **no es dónde está la lista**, que es una
 * decisión de composición y puede volver a cambiar: es que las dos afirmaciones
 * que sostienen la honestidad de la pantalla sigan siendo ciertas, esté donde
 * esté. Por eso los casos se reparten entre las dos pantallas en vez de
 * borrarse.
 */
const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      RouterLink: { template: '<a><slot /></a>' },
      Teleport: true,
    },
  },
}

describe('Inicio no afirma lo que no sabe', () => {
  beforeEach(() => setActivePinia(createPinia()))

  const montar = () => mount(PantallaInicio, entorno)

  /**
   * Sin empresa configurada, Inicio es la pantalla de primera vez: **una sola
   * cosa que hacer**. Lo que no puede perderse es la cifra, que es lo que
   * impide concluir que con ese paso ya se puede enviar.
   */
  it('la primera vez ofrece una sola acción, y dice cuántos pasos faltan', () => {
    const w = montar()
    expect(w.text()).toContain('Empieza por aquí')
    expect(w.text()).toContain('0 de 6')

    // Navegar es un enlace, no un botón con `push` dentro: ver `ABoton`.
    const acciones = w.findAll('.boton')
    expect(acciones, 'una sola acción, sin nada que compita').toHaveLength(1)
    expect(acciones[0]?.text()).toContain('Configurar mi empresa')
    expect(acciones[0]?.element.tagName, 'tiene que ser un enlace').toBe('A')
  })

  /**
   * Si los datos no se pudieron leer, la lista de reserva —«0 de 6», todo
   * pendiente— **no es el estado real**. Enseñarla haría que alguien volviera a
   * configurar lo que ya tenía configurado.
   */
  it('si no pudo leer los datos, no enseña nada: enseña el error', async () => {
    const empresa = useEmpresaStore()
    empresa.errorGeneral = 'error.db.sqlite'
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text(), 'no debe pintar un avance que no sabe si es cierto').not.toContain('0 de 6')
    expect(w.text()).not.toContain('Empieza por aquí')

    const alerta = w.find('[role="alert"]')
    expect(alerta.exists(), 'el error tiene que anunciarse').toBe(true)
    // Las tres partes del §95, no sólo la primera.
    expect(alerta.text()).toContain('Ocurrió un error al acceder a los datos')
    expect(alerta.text()).toContain('Cierra ARLES')
    expect(alerta.text()).toContain('se revirtieron')
  })

  /**
   * El panel modular no maqueta lo que no existe. Si algún día aparece un
   * módulo de campañas antes de que haya campañas, esta prueba lo dice.
   */
  it('no dibuja módulos vacíos esperando datos', () => {
    const empresa = useEmpresaStore()
    empresa.errorGeneral = 'error.db.sqlite'
    const w = montar()
    expect(w.findAll('.modulo')).toHaveLength(0)
  })
})

describe('Ajustes sigue enseñando los seis pasos', () => {
  beforeEach(() => setActivePinia(createPinia()))

  const montar = () => mount(PantallaAjustes, entorno)

  /**
   * La lista enseña los seis desde el primer día. Con sólo los construidos,
   * alguien la vería completa al terminar el primero y concluiría que ya puede
   * enviar. Se mudó de pantalla; la razón no se mudó.
   */
  it('enseña los seis, no sólo los construidos', () => {
    const w = montar()
    expect(w.findAll('.paso')).toHaveLength(6)
    expect(w.text()).toContain('0 de 6')
  })

  /** Un paso que aún no existe lo dice y no se puede pulsar. */
  it('los pasos no construidos no son enlaces y declaran su entrega', () => {
    const w = montar()
    const pasos = w.findAll('.paso')
    expect(pasos[0]?.find('a').exists(), 'el primero sí es accionable').toBe(true)
    for (const paso of pasos.slice(1)) {
      expect(paso.find('a').exists(), 'no debe ser un enlace').toBe(false)
      expect(paso.text()).toContain('Llega en la entrega')
    }
  })
})
