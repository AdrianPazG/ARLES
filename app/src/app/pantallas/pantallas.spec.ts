import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useAppStore } from '@/app/stores/app'
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
    const w = montar()
    await w.vm.$nextTick()

    // El error se fija **después** de montar, y no antes, desde el 14/10/2026.
    //
    // Antes se fijaba antes y la prueba pasaba — pero ese estado ya no es
    // alcanzable: `cargar()` limpia el aviso de la carga anterior al empezar
    // (arreglo del H-01), así que el montaje lo borraba y lo que se medía era
    // un estado imposible. Lo que sí puede ocurrir es que la carga falle y el
    // aviso quede puesto, que es lo que se simula aquí.
    empresa.errorGeneral = 'error.db.sqlite'
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

  /**
   * **Con los seis pasos hechos, Inicio no dice que falte nada.**
   *
   * Es D-01 de `10-requisitos/HALLAZGOS-DE-DISENO.md`, y es la prueba que no
   * existía: con 6 de 6, la primera pantalla del producto decía «No hay nada
   * más que puedas hacer aquí todavía» y «Lo que falta por construir · 1
   * paso», con el cuerpo nombrando tres secciones —campañas, canales y
   * actividad— que tienen pantalla desde la Fase 7.
   *
   * Ninguna prueba podía cazarlo porque todas montaban la pantalla con la
   * lista de reserva, que es **cero de seis**. El estado que mentía era el
   * único que nadie montaba: el del producto terminado.
   *
   * Verificada rompiéndola el 28/10/2026: devolviendo `None` en la rama de la
   * ventana de `PasoDeOnboarding::ruta`, que es el defecto original.
   */
  it('con los seis pasos hechos no dice que falte nada por construir', async () => {
    const empresa = useEmpresaStore()
    empresa.configuracion = {
      ...empresa.configuracion,
      empresa: {
        nombreComercial: 'Telemetry',
        pais: 'MX',
        zonaHoraria: 'America/Monterrey',
        correoCorporativo: 'hola@telemetry.mx',
        sitioWeb: '',
      },
      onboarding: {
        ...empresa.onboarding,
        pasos: empresa.onboarding.pasos.map((p) => ({ ...p, completado: true })),
        completados: 6,
        siguiente: null,
      },
    }
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('6 de 6')
    expect(w.text(), 'el producto está terminado').not.toContain(
      'No hay nada más que puedas hacer',
    )
    expect(w.text(), 'las tres secciones existen desde la Fase 7').not.toContain(
      'Lo que falta por construir',
    )
    expect(w.text()).toContain('Ya puedes enviar campañas')
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

  /**
   * Un paso que aún no existe lo dice y no se puede pulsar; uno construido
   * lleva a su pantalla.
   *
   * ── POR QUÉ YA NO SE MIRA LA POSICIÓN ──
   *
   * Esta prueba decía «el primero es accionable y los otros cinco no», y eso
   * era cierto cuando sólo existía la pantalla de la empresa. El 23/09/2026
   * dejó de serlo —Contactos llevaba construida desde la 3.2 sin ruta
   * apuntada, y Remitentes se entregó— y la prueba falló, que es justo lo que
   * tenía que hacer.
   *
   * Ahora se compara **contra lo que declara cada paso**, no contra su sitio
   * en la lista: el paso que trae ruta tiene que ser un enlace, y el que no la
   * trae tiene que decir en qué entrega llega. Así la prueba sobrevive a la
   * siguiente pantalla que se entregue en vez de volverse una afirmación
   * caducada.
   */
  it('los seis pasos son enlaces a su pantalla', () => {
    const empresa = useEmpresaStore()
    const w = montar()
    const pasos = w.findAll('.paso')
    const declarados = empresa.onboarding.pasos

    expect(pasos).toHaveLength(declarados.length)

    for (const [i, paso] of pasos.entries()) {
      const declarado = declarados[i]
      expect(declarado?.ruta, `${declarado?.paso} no lleva a ninguna parte`).toBeTruthy()
      expect(paso.find('a').exists(), `${declarado?.paso}: debe llevar a su pantalla`).toBe(
        true,
      )
      expect(paso.find('a').attributes('href')).toBe(declarado?.ruta)
    }

    // Y lo que ya no puede aparecer: el número de una entrega del roadmap en
    // la pantalla de alguien que no trabaja aquí.
    expect(w.text()).not.toContain('Llega en la entrega')
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

  /**
   * Y lleva SÓLO lo que le toca: la apariencia, y nada del negocio.
   *
   * ─────────────────────────────────────────────────────────────────────────
   * ESTA PRUEBA AFIRMABA LO CONTRARIO HASTA EL 21/09/2026
   *
   * Decía «lleva la apariencia y un enlace a la configuración», y exigía que
   * el puente existiera. Dirección mandó quitarlo el mismo día, mirando la
   * pantalla: «no me gusta que la opción de ir a configuración se encuentre en
   * esta sección, elimínala, junto con el texto».
   *
   * Tenía razón por segunda vez seguida sobre lo mismo. Ese bloque existía
   * sólo porque la configuración no aparecía en ninguna parte de la barra
   * lateral — era un parche a un fallo de navegación, no una decisión. Con la
   * barra enseñando «Inicio › Configuración», el parche sobra y lo único que
   * hace es devolver a Ajustes el vocabulario del negocio.
   *
   * Se deja escrito en vez de borrar la prueba: la afirmación vieja explica
   * por qué el bloque llegó a existir.
   * ─────────────────────────────────────────────────────────────────────────
   */
  it('lleva la apariencia y NINGÚN enlace a la configuración', () => {
    const w = mount(PantallaAjustes, entorno)

    expect(w.text()).toContain('Tema de la interfaz')

    const puente = w
      .findAll('a')
      .find((a) => a.attributes('href')?.includes('/inicio/configuracion'))
    expect(puente, 'el puente a la configuración volvió a Ajustes').toBeUndefined()

    // Y tampoco el texto, que es la otra mitad de lo que Dirección señaló: un
    // párrafo explicando dónde viven los datos del negocio sigue siendo
    // Ajustes hablando del negocio, aunque ya no haya botón.
    expect(w.text()).not.toContain('Los datos de tu empresa')
    expect(w.text()).not.toContain('zona horaria')
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

/**
 * El aviso de «esta ventana no está enviando».
 *
 * Aparece cuando hay otro ARLES abierto sobre la misma base. Es la señal que
 * distingue **«la campaña va despacio»** de **«esta ventana no manda nada»**:
 * las dos se ven igual —un contador que no sube— y la segunda no se arregla
 * esperando (§95).
 */
describe('cuando otra ventana de ARLES tiene el envío', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /** Deja el arranque cargado, diciendo si esta ventana envía o no. */
  function sembrarArranque(envia: boolean): void {
    const app = useAppStore()
    app.arranque = {
      baseDeDatosLista: true,
      versionEsquema: 7,
      empresaConfigurada: true,
      elMotorEnvia: envia,
    }
    app.cargada = true
  }

  /**
   * **Se dice, y se dicen las tres cosas** (§95).
   *
   * Sin el «qué está a salvo», la pregunta que queda es «¿entonces mi campaña
   * se perdió?». Alguien cerraría las dos ventanas y volvería a empezar.
   *
   * Verificada rompiéndola el 28/09/2026: quitando el `v-if` del aviso —o
   * cualquiera de las tres líneas— esta prueba falla.
   */
  it('lo dice en pantalla, con qué pasó, qué hacer y qué está a salvo', () => {
    sembrarEmpresa()
    sembrarArranque(false)

    const texto = mount(PantallaInicio, entorno).text()
    expect(texto).toContain('Esta ventana no está enviando')
    expect(texto).toContain('sólo uno puede enviar a la vez')
    expect(texto).toContain('Ciérralo y vuelve a abrir éste')
    expect(texto).toContain('no se ha perdido nada')
  })

  /**
   * **Y no aparece cuando esta ventana sí envía.**
   *
   * Hace falta: sin ella, la de arriba pasaría igual con un aviso pintado
   * siempre — y un aviso permanente deja de leerse, con lo que el día que
   * significara algo nadie lo miraría.
   */
  it('no aparece cuando esta ventana sí está enviando', () => {
    sembrarEmpresa()
    sembrarArranque(true)

    expect(mount(PantallaInicio, entorno).text()).not.toContain(
      'Esta ventana no está enviando',
    )
  })

  /**
   * **Ni antes de que el núcleo conteste.**
   *
   * Es el instante entre abrir ARLES y recibir el estado. Sin cuidado, el aviso
   * aparecería medio segundo en cada arranque — y un aviso que se enseña
   * siempre deja de leerse.
   *
   * ── LO QUE ESTA PRUEBA CAZA, Y LO QUE NO ──
   *
   * Lo que lo impide es **el valor de reserva del almacén**, que dice que sí se
   * envía mientras no se sepa otra cosa. No una comprobación de «ya cargó».
   *
   * La primera versión llevaba un `cargada.value &&` en el almacén con un
   * comentario que se atribuía el mérito. Al romperlo a propósito **no falló
   * ninguna prueba**: era redundante, y la explicación era falsa. Se quitó, y
   * esta prueba se reescribió para apuntar a lo que de verdad protege.
   *
   * Verificada rompiéndola el 28/09/2026: poniendo `elMotorEnvia: false` en el
   * valor de reserva, esta prueba falla.
   */
  it('no parpadea antes de que el núcleo conteste', () => {
    sembrarEmpresa()
    // Sin sembrar el arranque: el almacén está como recién creado, que es
    // exactamente el instante que se quiere mirar.
    expect(mount(PantallaInicio, entorno).text()).not.toContain(
      'Esta ventana no está enviando',
    )
  })
})
