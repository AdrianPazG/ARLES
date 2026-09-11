import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import ATabla from '@/design/componentes/ATabla.vue'
import APestanas from '@/design/componentes/APestanas.vue'
import AMenu from '@/design/componentes/AMenu.vue'

const cols = [{ id: 'n', titulo: 'N', valor: (f: { id: string }) => f.id }]
const hacer = (n: number) => Array.from({ length: n }, (_, i) => ({ id: `f${i}` }))

describe('sondas adversarias', () => {
  // ⚠ En jsdom el contenedor de desplazamiento mide 0 px de alto, así que el
  // virtualizador no renderiza NINGUNA fila. Un test aquí de «la lista se
  // acorta y no revienta» pasaría sin haber ejercitado nada — que es
  // exactamente la comprobación-teatro que la Fase 1 dejó como lección.
  //
  // Lo que sí se puede afirmar desde jsdom es el contrato de ARIA y la
  // limpieza de la selección. El encogido con filas de verdad se prueba en un
  // navegador: herramientas/validar/sondas/tabla.mjs.
  it('H1 · al encoger la lista, la selección que ya no existe se suelta', async () => {
    const w = mount(ATabla, {
      props: { filas: hacer(500), columnas: cols, etiqueta: 'T', seleccionada: 'f400' },
      attachTo: document.body,
    })
    await nextTick()
    await w.setProps({ filas: hacer(3) })
    await nextTick()
    // Sin esto quedaría una rejilla con una selección que apunta a nada.
    expect(w.emitted('update:seleccionada')?.at(-1)).toEqual([undefined])
    w.unmount()
  })

  it('H1b · la cuenta de filas que anuncia ARIA sigue al total real', async () => {
    const w = mount(ATabla, {
      props: { filas: hacer(50), columnas: cols, etiqueta: 'T' },
      attachTo: document.body,
    })
    // +1 por la cabecera: es lo que el lector de pantalla lee como total.
    expect(w.find('[role="grid"]').attributes('aria-rowcount')).toBe('51')
    await w.setProps({ filas: [] })
    expect(w.find('[role="grid"]').attributes('aria-rowcount')).toBe('1')
    w.unmount()
  })

  it('H2 · una tecla llamada «constructor» no ejecuta nada', async () => {
    const w = mount(APestanas, {
      props: { pestanas: [{ id: 'a', texto: 'A' }, { id: 'b', texto: 'B' }], modelValue: 'a' },
    })
    // Estricto: no basta con que no emita. `acciones['constructor']` devuelve
    // el constructor de Object, que es invocable y no hace nada visible — pero
    // el camino SÍ llega a preventDefault(), y entonces la tecla queda
    // tragada. Eso es lo que hay que comprobar.
    const prevenir = vi.fn()
    await w.find('[role="tablist"]').trigger('keydown', {
      key: 'constructor',
      preventDefault: prevenir,
    })
    expect(w.emitted('update:modelValue')).toBeUndefined()
    expect(prevenir).not.toHaveBeenCalled()
  })

  it('H2b · «constructor» tampoco en la tabla', async () => {
    const w = mount(ATabla, {
      props: { filas: hacer(10), columnas: cols, etiqueta: 'T' },
      attachTo: document.body,
    })
    const prevenir = vi.fn()
    await w.find('[role="grid"]').trigger('keydown', {
      key: 'toString',
      preventDefault: prevenir,
    })
    expect(w.emitted('update:seleccionada')).toBeUndefined()
    expect(prevenir).not.toHaveBeenCalled()
    w.unmount()
  })

  it('H2c · «constructor» tampoco en el menú', async () => {
    const w = mount(AMenu, {
      props: { etiqueta: 'M', acciones: [{ id: 'a', texto: 'A' }] },
      attachTo: document.body,
    })
    await w.find('.disparador').trigger('click')
    await nextTick()
    const prevenir = vi.fn()
    await w.find('[role="menu"]').trigger('keydown', {
      key: 'valueOf',
      preventDefault: prevenir,
    })
    expect(w.find('[role="menu"]').exists()).toBe(true) // no se cerró
    expect(prevenir).not.toHaveBeenCalled()
    w.unmount()
  })

  it('H4 · cambiar la pestaña desde fuera no roba el foco', async () => {
    const w = mount(APestanas, {
      props: { pestanas: [{ id: 'a', texto: 'A' }, { id: 'b', texto: 'B' }], modelValue: 'a' },
      attachTo: document.body,
    })
    const otro = document.createElement('input')
    document.body.append(otro)
    otro.focus()
    expect(document.activeElement).toBe(otro)

    await w.setProps({ modelValue: 'b' })
    await nextTick()
    expect(document.activeElement).toBe(otro)

    otro.remove()
    w.unmount()
  })
})
