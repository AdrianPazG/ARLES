import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useCampanasStore } from '@/app/stores/campanas'

import PantallaPlantillas from './PantallaPlantillas.vue'

const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' },
      Teleport: true,
    },
  },
}

function store(): ReturnType<typeof useCampanasStore> {
  const s = useCampanasStore()
  s.cargando = false
  return s
}

/** Abre el editor de plantilla y devuelve el montaje. */
async function abrirEditor(): Promise<ReturnType<typeof mount>> {
  const pantalla = mount(PantallaPlantillas, entorno)
  const nueva = pantalla.findAll('button').find((b) => b.text().includes('Nueva plantilla'))
  await nueva?.trigger('click')
  return pantalla
}

describe('la pantalla de plantillas y firmas', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * **Sin plantillas, se dice por qué hacen falta.**
   *
   * Un vacío que solo dice «no hay nada» deja a quien llega sin saber que esto
   * es lo que le impide crear una campaña — que es el motivo real de estar
   * aquí.
   */
  it('sin plantillas, dice que el asistente pide una', () => {
    store()
    const texto = mount(PantallaPlantillas, entorno).text()

    expect(texto).toContain('Todavía no hay ninguna plantilla')
    expect(texto).toContain('paso 4')
  })

  /**
   * **Una variable mal escrita se caza antes de guardar.**
   *
   * Es la garantía que salva un envío. `{{nombe}}` no falla en ningún sitio: el
   * motor no lo reconoce como variable, así que lo deja tal cual y sale impreso
   * con las llaves en el correo de cada persona. El preflight tampoco lo ve,
   * porque para él es texto.
   *
   * Verificada rompiéndola el 29/09/2026: quitando `variablesQueNoExisten` de
   * `puedeGuardar`, esta prueba falla y el botón queda activo.
   */
  it('una variable mal escrita bloquea el guardado y se dice cuál', async () => {
    store()
    const pantalla = await abrirEditor()

    const campos = pantalla.findAll('input')
    await campos[0]?.setValue('Bienvenida')
    await campos[1]?.setValue('Hola')
    await pantalla.find('textarea').setValue('Hola {{nombe}}, qué tal')

    expect(pantalla.text()).toContain('no reconoce')
    expect(pantalla.text()).toContain('{{nombe}}')

    const guardar = pantalla.findAll('button').find((b) => b.text() === 'Guardar')
    expect(guardar?.attributes('disabled')).toBeDefined()
  })

  /**
   * **Una variable bien escrita no estorba, y se cuenta.**
   *
   * El contrapeso de la prueba anterior: si el aviso saltara también con las
   * buenas, el editor sería inusable y nadie leería el aviso cuando importa.
   */
  it('una variable correcta deja guardar, y se cuenta cuántas usa', async () => {
    store()
    const pantalla = await abrirEditor()

    const campos = pantalla.findAll('input')
    await campos[0]?.setValue('Bienvenida')
    await campos[1]?.setValue('Hola {{nombre}}')
    await pantalla.find('textarea').setValue('Qué tal, {{nombre}} de {{empresa}}')

    expect(pantalla.text()).not.toContain('no reconoce')
    expect(pantalla.text()).toContain('2')

    const guardar = pantalla.findAll('button').find((b) => b.text() === 'Guardar')
    expect(guardar?.attributes('disabled')).toBeUndefined()
  })

  /**
   * **Los botones insertan la variable ya escrita.**
   *
   * Es el motivo de que existan: escribir `{{nombre}}` a mano es de donde salen
   * los `{{nombe}}`. Se comprueba que lo insertado es exactamente lo que el
   * núcleo reconoce, con sus dos llaves.
   */
  it('el botón de una variable la inserta con sus llaves', async () => {
    store()
    const pantalla = await abrirEditor()

    const botonNombre = pantalla.findAll('button').find((b) => b.text() === 'Nombre')
    await botonNombre?.trigger('click')

    const area = pantalla.find('textarea').element as HTMLTextAreaElement
    expect(area.value).toBe('{{nombre}}')
  })

  /**
   * **Las cuatro variables del núcleo tienen texto en español.**
   *
   * Hasta el 29/09/2026 ninguna lo tenía: el asistente las pedía desde la 6.6 y
   * `rotulo()` devolvía la clave en crudo, así que su paso 4 habría enseñado
   * «plantilla.variable.nombre». Nadie lo vio porque sin editor no se llegaba
   * allí con una plantilla elegida.
   */
  it('las cuatro variables se enseñan en español, no como clave', async () => {
    store()
    const texto = (await abrirEditor()).text()

    for (const esperado of ['Nombre', 'Apellido', 'Su empresa', 'Mi empresa']) {
      expect(texto).toContain(esperado)
    }
    expect(texto).not.toContain('plantilla.variable.')
  })

  /**
   * **Una firma no pide asunto.** Un correo tiene un asunto; una firma es el
   * cierre del mensaje. Pedirlo obligaría a rellenar un campo que no se usa.
   */
  it('el editor de firma no pide asunto, y ofrece la casilla de por defecto', async () => {
    store()
    const pantalla = mount(PantallaPlantillas, entorno)
    const nueva = pantalla.findAll('button').find((b) => b.text().includes('Nueva firma'))
    await nueva?.trigger('click')

    expect(pantalla.text()).not.toContain('Asunto del correo')
    expect(pantalla.find('input[type="checkbox"]').exists()).toBe(true)
  })
})
