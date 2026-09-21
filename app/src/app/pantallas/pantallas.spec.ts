import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useEmpresaStore } from '@/app/stores/empresa'

import PantallaAjustes from './PantallaAjustes.vue'
import PantallaConfiguracion from './PantallaConfiguracion.vue'
import PantallaInicio from './PantallaInicio.vue'

/**
 * Lo que las pantallas **no** deben afirmar.
 *
 * Salió de la auditoría de la 3.1: las aserciones de las capas decían que todo
 * estaba bien, y lo que fallaba era lo que la pantalla comunicaba.
 *
 * La lista de los seis pasos **se ha mudado dos veces**: Inicio → Ajustes
 * (17/09) → Configuración (21/09). Lo que se comprueba aquí **no es dónde
 * está**, que es composición y puede volver a cambiar: es que las afirmaciones
 * que sostienen la honestidad de la pantalla sigan siendo ciertas, esté donde
 * esté. Por eso los casos se reparten entre pantallas en vez de borrarse.
 */
const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      // Con `href`: el stub de antes era `<a><slot /></a>` y **ninguna prueba
      // podía comprobar a dónde lleva un enlace**. Es justo lo que falló el
      // 21/09/2026 — el botón de configurar llevaba a Ajustes y nada lo vio.
      RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' },
      Teleport: true,
    },
  },
}


/**
 * Deja una empresa guardada en el almacén.
 *
 * Se escribe en `configuracion`, que es el `ref`: `empresa` es un `computed`
 * derivado de él, y asignarle no hace nada —la primera versión de esto lo
 * intentaba y las pruebas pasaban por el camino equivocado—.
 */
function sembrarEmpresa(): void {
  const almacen = useEmpresaStore()
  almacen.configuracion = {
    ...almacen.configuracion,
    empresa: {
      nombreComercial: 'TELEMETRY INSIGHT',
      pais: 'MX',
      zonaHoraria: 'America/Mexico_City',
      correoCorporativo: 'hola@telemetrymx.com',
      sitioWeb: null,
    },
  }
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

describe('La configuración sigue enseñando los seis pasos', () => {
  beforeEach(() => setActivePinia(createPinia()))

  const montar = () => mount(PantallaConfiguracion, entorno)

  /**
   * La lista enseña los seis desde el primer día. Con sólo los construidos,
   * alguien la vería completa al terminar el primero y concluiría que ya puede
   * enviar.
   *
   * **Se ha mudado dos veces** —Inicio → Ajustes → Configuración— y esta
   * prueba ha sobrevivido a las dos, porque lo que comprueba no es dónde está
   * la lista, que es composición, sino que la afirmación siga siendo cierta.
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

describe('Ajustes es de la aplicación, no del negocio', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * Observación de Dirección del 21/09/2026. Ajustes llevaba el formulario de
   * la empresa, la apariencia y los seis pasos; se quedó sólo con lo de la
   * aplicación.
   *
   * Lo que se vigila es que **no vuelva a colarse el formulario**: duplicarlo
   * aquí no rompería nada visible, y dos formularios sobre la misma tabla
   * divergen en cuanto alguien toca uno.
   */
  it('no lleva el formulario de la empresa ni la lista de pasos', () => {
    const w = mount(PantallaAjustes, entorno)

    expect(w.findAll('.paso'), 'la lista de pasos volvió a Ajustes').toHaveLength(0)
    expect(w.find('form').exists(), 'el formulario volvió a Ajustes').toBe(false)
    expect(w.text()).not.toContain('Nombre comercial')
    expect(w.text()).not.toContain('Correo corporativo')
  })

  /** Y sí lleva lo que le toca: la apariencia, y el puente a donde están. */
  it('lleva la apariencia y un enlace a la configuración', () => {
    const w = mount(PantallaAjustes, entorno)

    expect(w.text()).toContain('Tema de la interfaz')
    const enlace = w
      .findAll('a')
      .find((a) => a.attributes('href')?.includes('/inicio/configuracion'))
    expect(enlace, 'sin puente, los datos de la empresa no se encuentran').toBeTruthy()
  })
})

describe('Lo guardado no se toca hasta pulsar «Editar»', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * Decisión de Dirección del 21/09/2026: al guardar, los campos quedan en
   * sólo lectura para que no se cambien por accidente.
   *
   * La zona horaria decide a qué hora sale cada campaña. Cambiarla con un roce
   * del ratón es la clase de accidente que se descubre tres semanas después
   * mirando por qué un envío salió de madrugada.
   */
  it('con algo guardado, los campos nacen bloqueados y hay botón de editar', async () => {
    sembrarEmpresa()

    const w = mount(PantallaConfiguracion, entorno)
    await w.vm.$nextTick()

    const editables = w
      .findAll('input')
      .filter((i) => i.attributes('readonly') === undefined)
    expect(editables, 'hay campos que se pueden escribir sin haber pulsado «Editar»')
      .toHaveLength(0)

    // Y no hay desplegables abiertos: el país y la zona salen como texto.
    expect(w.findAll('select'), 'un desplegable sigue pudiendo cambiarse').toHaveLength(0)

    expect(w.text()).toContain('Editar')
    expect(w.find('button[type="submit"]').exists(), 'un «Guardar» sin nada que guardar')
      .toBe(false)
  })

  /** Y al pulsar «Editar», se abren. Sin esto el bloqueo sería una jaula. */
  it('al pulsar «Editar» los campos se abren', async () => {
    sembrarEmpresa()

    const w = mount(PantallaConfiguracion, entorno)
    await w.vm.$nextTick()

    const boton = w.findAll('button').find((b) => b.text().includes('Editar'))
    expect(boton, 'no hay forma de editar lo guardado').toBeTruthy()
    await boton?.trigger('click')

    const bloqueados = w
      .findAll('input')
      .filter((i) => i.attributes('readonly') !== undefined)
    expect(bloqueados, 'siguen bloqueados después de pulsar «Editar»').toHaveLength(0)
    expect(w.findAll('select').length, 'los desplegables no volvieron').toBeGreaterThan(0)
  })

  /**
   * **La primera vez todo está abierto.** Sin esto, alguien que abre ARLES por
   * primera vez se encuentra un formulario vacío y bloqueado, y tiene que
   * adivinar que hay que pulsar «Editar» para empezar.
   */
  it('sin nada guardado, los campos nacen abiertos', () => {
    const w = mount(PantallaConfiguracion, entorno)

    const bloqueados = w
      .findAll('input')
      .filter((i) => i.attributes('readonly') !== undefined)
    expect(bloqueados, 'la primera vez el formulario salió bloqueado').toHaveLength(0)
    expect(w.find('button[type="submit"]').exists()).toBe(true)
  })
})

describe('Configurar la empresa no saca de INICIO', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * Observación de Dirección del 21/09/2026: pulsar «Configurar mi empresa»
   * saltaba a **Ajustes**, que es de la aplicación y no del negocio.
   *
   * Esto no se veía porque el stub de `RouterLink` no pintaba `href`: ninguna
   * prueba podía comprobar a dónde llevaba un enlace. Ahora sí.
   */
  it('la acción de Inicio lleva a la configuración, no a Ajustes', () => {
    const w = mount(PantallaInicio, entorno)
    const destinos = w.findAll('a').map((a) => a.attributes('href') ?? '')

    expect(
      destinos.some((d) => d.startsWith('/inicio/')),
      `ninguna acción lleva a la configuración: ${destinos.join(' · ')}`,
    ).toBe(true)
    expect(
      destinos.some((d) => d === '/ajustes'),
      'configurar la empresa vuelve a mandar a Ajustes',
    ).toBe(false)
  })
})
