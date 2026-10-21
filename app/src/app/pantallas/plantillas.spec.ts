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

/**
 * R-04 · una firma no puede llevar datos de quien recibe · 16/10/2026
 *
 * ─────────────────────────────────────────────────────────────────────────
 * DIRECCIÓN LO LLAMÓ «UNA TRAGEDIA», Y LO ES
 *
 * *«Sería una tragedia que apareciera en la firma el nombre del contacto a
 * quien se le está haciendo llegar el correo y su empresa, en lugar de los
 * datos del remitente.»*
 *
 * Este formulario sirve para plantilla **y** firma, y ofrecía las cuatro
 * variables en los dos casos. Tres son del destinatario: una firma con
 * `{{nombre}}` habría puesto el de quien recibe **debajo del mensaje, en cada
 * correo de la campaña**.
 *
 * Son dos mitades y hacen falta las dos: la pantalla **ayuda** —no ofrece lo
 * que no toca y avisa de lo escrito a mano— y el núcleo **impide**
 * (`revisar_firma`), para lo que llegue por otro camino.
 */
describe('el editor de firmas', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  async function abrirFirma(): Promise<ReturnType<typeof mount>> {
    const pantalla = mount(PantallaPlantillas, entorno)
    const nueva = pantalla.findAll('button').find((b) => b.text().includes('Nueva firma'))
    await nueva?.trigger('click')
    return pantalla
  }

  it('NO ofrece las variables de quien recibe', async () => {
    const p = await abrirFirma()
    const botones = p.findAll('button').map((b) => b.text())
    expect(botones.some((t) => t.includes('Mi empresa'))).toBe(true)
    for (const ajena of ['Nombre', 'Apellido']) {
      expect(
        botones.filter((t) => t === ajena),
        `una firma no puede ofrecer «${ajena}»`,
      ).toHaveLength(0)
    }
  })

  it('en una plantilla sí se ofrecen las cuatro', async () => {
    const p = await abrirEditor()
    const botones = p.findAll('button').map((b) => b.text())
    // En una plantilla son correctas: el mensaje SÍ habla de quien lo recibe.
    expect(botones.some((t) => t === 'Nombre')).toBe(true)
    expect(botones.some((t) => t.includes('Mi empresa'))).toBe(true)
  })

  it('avisa si se escriben a mano, y no deja guardar', async () => {
    const p = await abrirFirma()
    await p.find('input').setValue('Mi firma')
    await p.find('textarea').setValue('Ana Ruiz\n{{nombre}} · {{empresa}}')
    await p.vm.$nextTick()

    expect(p.text()).toContain('Esto no va en una firma')
    const guardar = p.findAll('button').find((b) => b.text() === 'Guardar')
    expect(guardar?.attributes('disabled')).toBeDefined()
  })

  it('con sólo «mi empresa» sí deja guardar', async () => {
    const p = await abrirFirma()
    await p.find('input').setValue('Mi firma')
    await p.find('textarea').setValue('Ana Ruiz\n{{mi_empresa}}')
    await p.vm.$nextTick()

    expect(p.text()).not.toContain('Esto no va en una firma')
    const guardar = p.findAll('button').find((b) => b.text() === 'Guardar')
    expect(guardar?.attributes('disabled')).toBeUndefined()
  })
})

/**
 * R-06 · corregir una plantilla o una firma ya creadas · 17/10/2026
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE LA CABECERA DE LA PANTALLA AFIRMABA, Y POR QUÉ ERA FALSO
 *
 * Decía que un botón de editar «sería prometer algo que el modelo de datos
 * impide a propósito». Lo que el modelo impide es **reescribir lo ya
 * enviado**; corregir una plantilla no es eso: es hacer nacer la versión
 * siguiente, y `nueva_version_de_plantilla` llevaba desde la 6.3 en
 * `arles-db`, probada y sin que nadie la llamara.
 *
 * Las cuatro pruebas de aquí son lo que corregir puede hacer mal:
 *
 * 1. Abrir el editor **vacío**, que convierte «corregir una errata» en
 *    «volver a redactar el correo entero».
 * 2. Mandar al comando de crear en vez del de corregir, que deja dos
 *    plantillas con el mismo nombre.
 * 3. No decir que lo que está en cola **no cambia**. Es el error caro, y es
 *    el que la forma de los datos invita a cometer.
 * 4. Arrastrar el identificador de lo corregido a la creación siguiente:
 *    quien corrige una plantilla y después crea otra habría creado una
 *    versión de la primera.
 *
 * **Vistas en rojo el 17/10/2026**, una ruptura por garantía: el editor sin
 * precargar, el aviso de la versión fuera, y el guardado mandando siempre al
 * comando de crear —en la plantilla y en la firma—.
 *
 * Y la quinta enseñó algo: `idEditado` se limpia en **dos** sitios, `abrir()`
 * y `cerrar()`, así que quitar uno no deja ningún síntoma —cada uno tapa al
 * otro— y la prueba seguía en verde. Hubo que quitar los dos para verla caer.
 * Está bien que el código lo haga dos veces; lo que no vale es dar la
 * verificación por hecha con una ruptura que la otra mitad tapa (la lección de
 * la pasada D: una ruptura que deja el síntoma en pie no prueba nada).
 */
describe('corregir lo ya escrito', () => {
  beforeEach(() => setActivePinia(createPinia()))

  const UNA_PLANTILLA = {
    id: 'p-1',
    nombre: 'Bienvenida',
    version: {
      id: 'v-1',
      numero: 2,
      asunto: 'Hola',
      cuerpoHtml: '<p>Hola {{nombre}}</p>',
      variables: ['nombre'],
    },
  }

  const UNA_FIRMA = {
    id: 'f-1',
    nombre: 'Ventas',
    cuerpoHtml: '<p>Ana Ruiz</p>',
    porDefecto: true,
  }

  /** Abre el editor pulsando «Corregir» en la enésima tarjeta que lo tenga. */
  async function corregir(cual: number): Promise<ReturnType<typeof mount>> {
    const pantalla = mount(PantallaPlantillas, entorno)
    const botones = pantalla.findAll('button').filter((b) => b.text() === 'Corregir')
    await botones[cual]?.trigger('click')
    return pantalla
  }

  it('el editor se abre con el texto que la plantilla dice hoy', async () => {
    const s = store()
    s.plantillas = [UNA_PLANTILLA]

    const p = await corregir(0)
    const campos = p.findAll('input')
    expect(campos[0]?.element.value).toBe('Bienvenida')
    expect(campos[1]?.element.value).toBe('Hola')
    expect(p.find('textarea').element.value).toBe('<p>Hola {{nombre}}</p>')
  })

  it('dice qué versión nace y que lo que está en cola no cambia', async () => {
    const s = store()
    s.plantillas = [UNA_PLANTILLA]

    const texto = (await corregir(0)).text()
    // §94: la cifra, no «una versión nueva». La vigente es la 2.
    expect(texto).toContain('crear la versión 3')
    expect(texto).toContain('lo que está en cola no cambia')
  })

  it('guardar una corrección llama al comando de versionar, no al de crear', async () => {
    const s = store()
    s.plantillas = [UNA_PLANTILLA]
    const pedidos: string[] = []
    s.nuevaVersionDePlantilla = async (id: string) => {
      pedidos.push(`version:${id}`)
      return true
    }
    s.guardarPlantilla = async () => {
      pedidos.push('crear')
      return true
    }

    const p = await corregir(0)
    await p.find('textarea').setValue('<p>Hola otra vez</p>')
    const guardar = p.findAll('button').find((b) => b.text() === 'Guardar')
    await guardar?.trigger('click')

    expect(pedidos).toEqual(['version:p-1'])
  })

  it('corregir una firma la reescribe, y no crea otra', async () => {
    const s = store()
    s.firmas = [UNA_FIRMA]
    const pedidos: string[] = []
    s.editarFirma = async (id: string) => {
      pedidos.push(`editar:${id}`)
      return true
    }
    s.guardarFirma = async () => {
      pedidos.push('crear')
      return true
    }

    const p = await corregir(0)
    expect(p.find('textarea').element.value).toBe('<p>Ana Ruiz</p>')
    const casilla = p.find('input[type="checkbox"]').element as HTMLInputElement
    expect(casilla.checked, 'y la casilla llega marcada como está guardada').toBe(true)

    const guardar = p.findAll('button').find((b) => b.text() === 'Guardar')
    await guardar?.trigger('click')
    expect(pedidos).toEqual(['editar:f-1'])
  })

  /**
   * **Y lo de después de corregir.** `idEditado` se queda puesto si `abrir()`
   * no lo limpia, y entonces «Nueva plantilla» crearía una versión de la que
   * se acababa de corregir: una plantilla nueva que no aparece y una vieja que
   * cambió sola.
   */
  it('después de corregir, crear una nueva vuelve a crear', async () => {
    const s = store()
    s.plantillas = [UNA_PLANTILLA]
    const pedidos: string[] = []
    s.nuevaVersionDePlantilla = async () => {
      pedidos.push('version')
      return true
    }
    s.guardarPlantilla = async () => {
      pedidos.push('crear')
      return true
    }

    const p = await corregir(0)
    const cancelar = p.findAll('button').find((b) => b.text() === 'Cancelar')
    await cancelar?.trigger('click')

    const nueva = p.findAll('button').find((b) => b.text().includes('Nueva plantilla'))
    await nueva?.trigger('click')
    const campos = p.findAll('input')
    await campos[0]?.setValue('Otra')
    await campos[1]?.setValue('Asunto')
    await p.find('textarea').setValue('<p>Texto</p>')
    const guardar = p.findAll('button').find((b) => b.text() === 'Guardar')
    await guardar?.trigger('click')

    expect(pedidos).toEqual(['crear'])
  })
})

/**
 * R-03 · negrita y cursiva · 17/10/2026
 *
 * Dirección: *«me gustaría tener la posibilidad de poder colocar texto en
 * negrita o cursiva»*. El saneado del núcleo acepta `strong`, `em`, `b`, `i`,
 * `u`, listas y enlaces **desde la 6.2**: lo que no había era forma de
 * ponerlos sin escribir las etiquetas a mano.
 *
 * Las dos pruebas son los dos casos que un botón de formato puede equivocar, y
 * el segundo es el que se olvida. **Vistas en rojo el 17/10/2026**: pegando
 * las etiquetas al final en vez de envolver, y saliendo sin hacer nada cuando
 * no hay nada seleccionado.
 */
describe('negrita y cursiva', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('envuelve lo que está seleccionado', async () => {
    store()
    const p = await abrirEditor()
    const area = p.find('textarea')
    await area.setValue('Hola Ana, qué tal')

    const el = area.element as HTMLTextAreaElement
    el.setSelectionRange(5, 8) // «Ana»
    const negrita = p.findAll('button').find((b) => b.text() === 'Negrita')
    await negrita?.trigger('click')

    expect(el.value).toBe('Hola <strong>Ana</strong>, qué tal')
  })

  /**
   * **Sin selección, inserta el par y no se queda quieto.**
   *
   * Es el caso que se olvida, y el que se lee como una avería: quien quiere
   * *empezar* a escribir en cursiva pulsa el botón antes de escribir, y un
   * botón que no hace nada parece roto (§95).
   */
  it('sin selección, deja el par de etiquetas para escribir dentro', async () => {
    store()
    const p = await abrirEditor()
    const area = p.find('textarea')
    await area.setValue('Hola')

    const el = area.element as HTMLTextAreaElement
    el.setSelectionRange(4, 4)
    const cursiva = p.findAll('button').find((b) => b.text() === 'Cursiva')
    await cursiva?.trigger('click')

    expect(el.value).toBe('Hola<em></em>')
  })
})

/**
 * R-08 · eliminar una plantilla o una firma · 21/10/2026
 *
 * Dirección: *«tampoco me permite eliminar una plantilla ni una firma, donde
 * también debería tener esa opción»*. Se creaban y se quedaban para siempre,
 * así que una plantilla de prueba estorbaba en el paso 4 del asistente para
 * siempre.
 *
 * Las tres pruebas son lo que un borrado puede hacer mal, y las tres duelen:
 * borrar sin preguntar, borrar lo que no era, y no avisar de que la firma que
 * se va es la que usan las campañas.
 */
describe('eliminar lo ya escrito', () => {
  beforeEach(() => setActivePinia(createPinia()))

  const UNA_PLANTILLA = {
    id: 'p-1',
    nombre: 'Bienvenida',
    version: { id: 'v-1', numero: 1, asunto: 'Hola', cuerpoHtml: '<p>Hola</p>', variables: [] },
  }

  /** Pulsa el botón «Eliminar» de la enésima tarjeta que lo tenga. */
  async function abrirBorrado(cual: number): Promise<ReturnType<typeof mount>> {
    const pantalla = mount(PantallaPlantillas, entorno)
    const botones = pantalla.findAll('button').filter((b) => b.text() === 'Eliminar')
    await botones[cual]?.trigger('click')
    return pantalla
  }

  it('pregunta antes de eliminar, y no elimina al abrir la pregunta', async () => {
    const s = store()
    s.plantillas = [UNA_PLANTILLA]
    const pedidos: string[] = []
    s.borrarPlantilla = async (id: string) => {
      pedidos.push(id)
      return true
    }

    const p = await abrirBorrado(0)
    expect(p.text()).toContain('¿Eliminar «Bienvenida»?')
    expect(pedidos, 'abrir la pregunta no borra nada').toEqual([])
  })

  it('al confirmar, elimina esa plantilla y no otra', async () => {
    const s = store()
    s.plantillas = [UNA_PLANTILLA, { ...UNA_PLANTILLA, id: 'p-2', nombre: 'Otra' }]
    const pedidos: string[] = []
    s.borrarPlantilla = async (id: string) => {
      pedidos.push(id)
      return true
    }

    const p = await abrirBorrado(1)
    const botones = p.findAll('button').filter((b) => b.text() === 'Eliminar')
    await botones[botones.length - 1]?.trigger('click')

    expect(pedidos).toEqual(['p-2'])
  })

  /**
   * **Y avisa si la firma que se va es la de por defecto.**
   *
   * Es la que usan las campañas cuando no se elige otra, así que quitarla
   * cambia lo que van a mandar — en silencio, y sin que falle nada.
   */
  it('avisa cuando la firma que se elimina es la de por defecto', async () => {
    const s = store()
    s.firmas = [
      { id: 'f-1', nombre: 'Ventas', cuerpoHtml: '<p>V</p>', porDefecto: true },
      { id: 'f-2', nombre: 'Soporte', cuerpoHtml: '<p>S</p>', porDefecto: false },
    ]

    const conDefecto = await abrirBorrado(0)
    expect(conDefecto.text()).toContain('Es la firma que se usa por defecto')

    const sinDefecto = await abrirBorrado(1)
    expect(sinDefecto.text()).not.toContain('Es la firma que se usa por defecto')
  })
})
