/**
 * Tests de las primitivas.
 *
 * No comprueban que «se vean bien» —eso lo decide una persona mirando el
 * catálogo—, sino que **las reglas del design system que se pueden romper en
 * silencio siguen en pie**: las que no dan error de compilación y que un
 * cambio bienintencionado se lleva por delante sin que nadie lo note.
 *
 * Cada `describe` nombra la regla que defiende y de dónde sale.
 */
import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'

import ABoton from './ABoton.vue'
import AEntrada from './AEntrada.vue'
import AInsignia from './AInsignia.vue'
import ALogotipo from './ALogotipo.vue'
import AMenu from './AMenu.vue'
import AModal from './AModal.vue'
import APestanas from './APestanas.vue'

describe('regla 7.3 · ningún estado se comunica sólo con color', () => {
  it('toda insignia lleva un icono, en los seis tonos', () => {
    for (const tono of ['neutro', 'exito', 'aviso', 'peligro', 'info', 'incierto'] as const) {
      const w = mount(AInsignia, { props: { tono }, slots: { default: 'x' } })
      expect(w.find('svg path').attributes('d'), tono).toBeTruthy()
    }
  })

  it('el envío ambiguo no comparte icono con el éxito ni con el fallo', () => {
    // ADR-0004: `presumed_sent` no es ninguno de los dos. Si compartiera
    // icono, la interfaz estaría afirmando algo que el producto no sabe.
    const trazo = (tono: 'incierto' | 'exito' | 'peligro') =>
      mount(AInsignia, { props: { tono }, slots: { default: 'x' } })
        .find('svg path')
        .attributes('d')

    expect(trazo('incierto')).not.toBe(trazo('exito'))
    expect(trazo('incierto')).not.toBe(trazo('peligro'))
  })

  it('el error de un campo va en texto, no sólo en el borde', () => {
    const w = mount(AEntrada, {
      props: { etiqueta: 'Correo', error: 'La dirección no es válida.' },
    })
    const mensaje = w.find('[role="alert"]')
    expect(mensaje.exists()).toBe(true)
    expect(mensaje.text()).toContain('La dirección no es válida.')
    // Y el campo lo declara, para que el lector de pantalla lo anuncie al
    // entrar en vez de dejarlo descubrir al enviar.
    const entrada = w.find('input')
    expect(entrada.attributes('aria-invalid')).toBe('true')
    expect(entrada.attributes('aria-describedby')).toBe(mensaje.attributes('id'))
  })

  it('el campo sólo se describe por lo que está visible', () => {
    // Con error, la ayuda no se renderiza: apuntar a un nodo ausente deja al
    // lector de pantalla con una referencia rota.
    const w = mount(AEntrada, {
      props: { etiqueta: 'Correo', ayuda: 'Ayuda', error: 'Error' },
    })
    const descrito = w.find('input').attributes('aria-describedby')
    expect(w.find(`#${descrito}`).exists()).toBe(true)
    expect(w.text()).not.toContain('Ayuda')
  })
})

describe('§21 · el logotipo es tipográfico y no lleva el numeral', () => {
  it('dice ARLES RELAY, sin «I»', () => {
    const w = mount(ALogotipo)
    expect(w.text()).toBe('ARLESRELAY')
    // ADR-0010: el «I» pertenece al nombre comercial, no al logotipo.
    expect(w.text()).not.toContain('I ')
    expect(w.get('.logotipo').attributes('aria-label')).toBe('ARLES RELAY')
  })

  it('se anuncia como una sola cosa', () => {
    // Sin el aria-label, un lector de pantalla leería «ARLES» y «RELAY» como
    // dos elementos separados.
    const w = mount(ALogotipo)
    expect(w.findAll('[aria-hidden="true"]')).toHaveLength(2)
  })
})

describe('ABoton', () => {
  it('«ocupado» deshabilita el clic y lo anuncia', () => {
    const w = mount(ABoton, { props: { ocupado: true } })
    expect(w.attributes('aria-busy')).toBe('true')
    expect(w.attributes('disabled')).toBeDefined()
  })

  it('un botón que no está ocupado no lleva aria-busy', () => {
    // `aria-busy="false"` permanente hace ruido en el lector de pantalla.
    expect(mount(ABoton).attributes('aria-busy')).toBeUndefined()
  })
})

describe('AModal · una acción destructiva exige decisión explícita', () => {
  it('Esc no cierra un modal destructivo', async () => {
    const w = mount(AModal, {
      props: { abierto: true, titulo: 'Detener la campaña', destructiva: true },
      attachTo: document.body,
    })
    await nextTick()
    await w.find('dialog').trigger('keydown', { key: 'Escape' })
    expect(w.emitted('cerrar')).toBeUndefined()
    w.unmount()
  })

  it('Esc sí cierra un modal normal', async () => {
    const w = mount(AModal, {
      props: { abierto: true, titulo: 'Duplicar campaña' },
      attachTo: document.body,
    })
    await nextTick()
    await w.find('dialog').trigger('keydown', { key: 'Escape' })
    expect(w.emitted('cerrar')).toHaveLength(1)
    w.unmount()
  })

  it('el modal destructivo no ofrece la X de cerrar', () => {
    // Si estuviera, cerrar por accidente y creer que se canceló sería el
    // mismo error que confirmar sin querer, pero silencioso.
    const w = mount(AModal, {
      props: { abierto: true, titulo: 'Detener', destructiva: true },
      attachTo: document.body,
    })
    expect(w.find('.cerrar').exists()).toBe(false)
    w.unmount()
  })
})

describe('APestanas · tabindex móvil', () => {
  const pestanas = [
    { id: 'a', texto: 'Uno' },
    { id: 'b', texto: 'Dos' },
    { id: 'c', texto: 'Tres' },
  ]

  it('sólo la pestaña activa está en el orden de tabulación', () => {
    const w = mount(APestanas, { props: { pestanas, modelValue: 'b' } })
    const indices = w.findAll('[role="tab"]').map((t) => t.attributes('tabindex'))
    expect(indices).toEqual(['-1', '0', '-1'])
  })

  it('las flechas dan la vuelta en los dos extremos', async () => {
    const w = mount(APestanas, { props: { pestanas, modelValue: 'c' } })
    await w.find('[role="tablist"]').trigger('keydown', { key: 'ArrowRight' })
    expect(w.emitted('update:modelValue')?.at(-1)).toEqual(['a'])

    await w.setProps({ modelValue: 'a' })
    await w.find('[role="tablist"]').trigger('keydown', { key: 'ArrowLeft' })
    expect(w.emitted('update:modelValue')?.at(-1)).toEqual(['c'])
  })

  it('las flechas saltan las pestañas deshabilitadas', async () => {
    const conHueco = [
      { id: 'a', texto: 'Uno' },
      { id: 'b', texto: 'Dos', deshabilitada: true },
      { id: 'c', texto: 'Tres' },
    ]
    const w = mount(APestanas, { props: { pestanas: conHueco, modelValue: 'a' } })
    await w.find('[role="tablist"]').trigger('keydown', { key: 'ArrowRight' })
    expect(w.emitted('update:modelValue')?.at(-1)).toEqual(['c'])
  })
})

describe('AMenu · el foco vuelve al disparador', () => {
  const acciones = [
    { id: 'uno', texto: 'Uno' },
    { id: 'dos', texto: 'Dos' },
  ]

  it('Esc cierra y devuelve el foco', async () => {
    const w = mount(AMenu, {
      props: { etiqueta: 'Acciones', acciones },
      attachTo: document.body,
    })
    const disparador = w.find('.disparador').element as HTMLButtonElement
    const volver = vi.spyOn(disparador, 'focus')

    await w.find('.disparador').trigger('click')
    await nextTick()
    await w.find('[role="menu"]').trigger('keydown', { key: 'Escape' })

    expect(w.find('[role="menu"]').exists()).toBe(false)
    // Sin esto el usuario de teclado acaba al principio de la pantalla.
    expect(volver).toHaveBeenCalled()
    w.unmount()
  })

  it('Tab cierra el menú pero deja que el foco siga su camino', async () => {
    const w = mount(AMenu, {
      props: { etiqueta: 'Acciones', acciones },
      attachTo: document.body,
    })
    const disparador = w.find('.disparador').element as HTMLButtonElement
    const volver = vi.spyOn(disparador, 'focus')

    await w.find('.disparador').trigger('click')
    await nextTick()
    await w.find('[role="menu"]').trigger('keydown', { key: 'Tab' })

    expect(w.find('[role="menu"]').exists()).toBe(false)
    expect(volver).not.toHaveBeenCalled()
    w.unmount()
  })

  it('una acción deshabilitada no emite nada', async () => {
    const w = mount(AMenu, {
      props: {
        etiqueta: 'Acciones',
        acciones: [{ id: 'uno', texto: 'Uno', deshabilitada: true }],
      },
      attachTo: document.body,
    })
    await w.find('.disparador').trigger('click')
    await nextTick()
    await w.find('.opcion').trigger('click')
    expect(w.emitted('elegir')).toBeUndefined()
    w.unmount()
  })
})
