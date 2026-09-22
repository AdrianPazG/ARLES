import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { i18n } from '@/app/i18n'

import PantallaDerechos from './PantallaDerechos.vue'

/**
 * La pantalla del derecho de Acceso · entrega 3.4
 *
 * ───────────────────────────────────────────────────────────────────────────
 * QUÉ SE VIGILA AQUÍ, Y QUÉ NO
 *
 * NO se comprueba que el informe lleve los datos correctos: eso lo arma Rust
 * y lo prueban `arles-core::arco` y `arles-db::arco`. Duplicarlo aquí sería
 * una segunda copia de la misma regla, y las copias divergen.
 *
 * Lo que se vigila es lo que **sólo esta pantalla puede equivocar**: que los
 * tres desenlaces distintos se cuenten distinto. Los tres son un éxito, y por
 * eso es fácil pintarlos igual — y entonces quien atiende una solicitud
 * enviaría un archivo vacío creyendo que va lleno.
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
 * Finge el núcleo y le hace devolver `respuesta`.
 *
 * Se falsea el módulo entero y no sólo `invocar`, porque la pantalla también
 * pregunta `hayNucleo()`: sin eso, cae por la rama de «esto es la vista
 * previa» y nunca llega a llamar al comando.
 */
function conNucleo(respuesta: unknown) {
  const invocar = vi.fn().mockResolvedValue(respuesta)
  vi.doMock('@/app/nucleo', async () => {
    const real = await vi.importActual<typeof import('@/app/nucleo')>('@/app/nucleo')
    return { ...real, hayNucleo: () => true, invocar }
  })
  return invocar
}

/** Monta la pantalla, escribe una dirección y pulsa el botón. */
async function pedir(w: ReturnType<typeof mount>, direccion = 'ana@empresa.mx') {
  await w.find('input').setValue(direccion)
  await w.find('form').trigger('submit')
  await flushPromises()
}

describe('el informe de Acceso cuenta qué se encontró', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
    vi.clearAllMocks()
  })

  it('con datos, dice que el archivo los lleva', async () => {
    conNucleo({
      nombreDelArchivo: 'datos-de-ana-empresa.mx.txt',
      habiaDatos: true,
      sigueSiendoContacto: true,
    })
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)
    await pedir(w)

    expect(w.text()).toContain('datos-de-ana-empresa.mx.txt')
    expect(w.text()).toContain('todo lo que ARLES tiene')
  })

  /**
   * EL CASO QUE MÁS IMPORTA.
   *
   * «No consta nada» NO es un error: puede ser justo lo que la persona quería
   * oír, y el informe se entrega igual. Pero tiene que **decirse**, porque
   * quien atiende está a punto de mandar un archivo vacío y necesita saber que
   * va vacío a propósito y no por un fallo de búsqueda.
   *
   * Se comprueba además que NO se pinta como error: un aviso rojo convertiría
   * una respuesta legítima en una avería aparente.
   */
  it('sin datos, lo dice y no lo pinta como un fallo', async () => {
    conNucleo({
      nombreDelArchivo: 'datos-de-nadie-empresa.mx.txt',
      habiaDatos: false,
      sigueSiendoContacto: false,
    })
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)
    await pedir(w, 'nadie@empresa.mx')

    expect(w.text()).toContain('va vacío')
    expect(w.text()).toContain('es una respuesta válida')
    // Se mira EL RESULTADO, no la página entera.
    //
    // La primera versión buscaba «peligro» en todo el HTML, y eso dejó de
    // valer en cuanto la pantalla ganó el bloque de borrado —que lleva un
    // aviso rojo legítimo—. Una aserción sobre toda la página no distingue
    // «el resultado está mal pintado» de «hay algo rojo en otra parte».
    const aviso = w.find('.derechos__resultado')
    expect(aviso.exists(), 'no se pintó ningún resultado').toBe(true)
    expect(
      aviso.html(),
      'un informe vacío se está pintando como una avería',
    ).not.toContain('peligro')
  })

  /**
   * Alguien que pidió que lo borraran sigue teniendo rastro —su baja, sus
   * envíos—, y el informe lo incluye. Si esta pantalla dijera «todo lo que
   * tenemos de esa dirección» sin más, quien atiende no entendería por qué
   * aparece gente que creía borrada.
   */
  it('si ya no es contacto pero queda rastro, lo distingue', async () => {
    conNucleo({
      nombreDelArchivo: 'datos-de-beto-otra.mx.txt',
      habiaDatos: true,
      sigueSiendoContacto: false,
    })
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)
    await pedir(w, 'beto@otra.mx')

    expect(w.text()).toContain('ya no está dada de alta como contacto')
    expect(
      w.text(),
      'se está contando como si siguiera siendo un contacto normal',
    ).not.toContain('todo lo que ARLES tiene')
  })

  /**
   * Cancelar el diálogo de guardado devuelve `null`. Es una decisión, no un
   * fallo: quien atiende cambió de opinión. Tratarlo como error le haría
   * pensar que algo se rompió, y revisar una base de datos que está intacta.
   */
  it('cancelar no es un error', async () => {
    conNucleo(null)
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)
    await pedir(w)

    expect(w.text()).toContain('sin guardar')
    const aviso = w.find('.derechos__resultado')
    expect(aviso.exists()).toBe(true)
    expect(aviso.html(), 'cancelar se está pintando como un fallo').not.toContain('peligro')
  })

  /**
   * La dirección se manda **recortada**. Quien copia de un correo se lleva
   * espacios al final sin verlos, y con ellos el validador del núcleo
   * rechazaría una dirección perfectamente buena.
   *
   * ─────────────────────────────────────────────────────────────────────────
   * SE PRUEBA SOBRE WHATSAPP, Y NO SOBRE CORREO, A PROPÓSITO
   *
   * La primera versión de esta prueba escribía «  ana{'@'}empresa.mx  » en el
   * campo de correo y **pasaba con el `.trim()` quitado**: era decorativa. El
   * motivo es del HTML, no del código — un `input[type="email"]` tiene
   * saneado de valor por norma y se come los espacios de los extremos él
   * solo, así que la prueba medía el navegador, no la pantalla.
   *
   * Con el canal de WhatsApp el campo es `type="text"`, que no sanea nada, y
   * el recorte vuelve a ser responsabilidad de la pantalla. Comprobado
   * rompiéndolo: sin `.trim()` esta versión falla, y la de correo no.
   * ─────────────────────────────────────────────────────────────────────────
   */
  it('recorta los espacios que vienen de copiar y pegar', async () => {
    const invocar = conNucleo({
      nombreDelArchivo: 'x.txt',
      habiaDatos: true,
      sigueSiendoContacto: true,
    })
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)

    await w.find('select').setValue('whatsapp')
    await w.find('input').setValue('  +528112345678  ')
    await w.find('form').trigger('submit')
    await flushPromises()

    expect(invocar).toHaveBeenCalledWith('entregar_informe_de_acceso', {
      canal: 'whatsapp',
      direccion: '+528112345678',
    })
  })

  /**
   * Sin dirección no se llama al núcleo. Es una barandilla barata: el comando
   * la rechazaría igual, pero con un error, y un error por un campo que el
   * usuario aún no ha rellenado se lee como una regañina.
   */
  it('sin dirección no se llama al núcleo', async () => {
    const invocar = conNucleo(null)
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)
    await w.find('form').trigger('submit')
    await flushPromises()

    expect(invocar).not.toHaveBeenCalled()
  })
})

describe('la pantalla dice lo que ARLES todavía no hace', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * ESTA PRUEBA DECÍA LO CONTRARIO HASTA EL 22/09/2026.
   *
   * Decía «avisa de que el borrado todavía no existe», y era cierto: la
   * pantalla listaba «Que borres sus datos → Todavía no está construido».
   * Ahora está construido, así que lo que hay que vigilar es lo contrario —
   * que se pueda llegar a él — y que no haya quedado el aviso viejo diciendo
   * que no existe algo que sí existe.
   *
   * Se deja escrita la afirmación vieja en vez de borrar la prueba: explica
   * por qué ese texto llegó a estar ahí.
   */
  it('el borrado existe, y ya no se dice que falta', () => {
    const w = mount(PantallaDerechos, entorno)
    expect(w.text()).toContain('Borrar los datos de alguien')
    expect(
      w.text(),
      'quedó el aviso viejo: la pantalla dice que falta algo que ya está',
    ).not.toContain('Todavía no está construido')
  })

  /** Y encamina los dos derechos que sí están hechos, en vez de callarlos. */
  it('encamina la corrección y la baja, que sí existen', () => {
    const w = mount(PantallaDerechos, entorno)
    expect(w.text()).toContain('Búscalo en Contactos y edítalo')
    expect(w.text()).toContain('desde Bajas')
  })

  /**
   * «ARCO» no se escribe en pantalla. Es la sigla de Acceso, Rectificación,
   * Cancelación y Oposición, y no se la sabe nadie fuera de protección de
   * datos. Quien atiende necesita entender el botón, no la sigla.
   */
  it('no usa la sigla ARCO en la interfaz', () => {
    const w = mount(PantallaDerechos, entorno)
    expect(w.text()).not.toContain('ARCO')
  })
})

/**
 * El borrado · la única acción irreversible de ARLES
 *
 * ───────────────────────────────────────────────────────────────────────────
 * LO QUE SE VIGILA AQUÍ ES QUE NO SE PUEDA BORRAR POR ACCIDENTE
 *
 * Que el borrado funcione lo prueban `arles-core` y `arles-db` sobre una base
 * real. Lo que sólo esta pantalla puede equivocar es **dejar pulsar el botón
 * antes de tiempo** — y ése es el fallo que no tiene arreglo: no hay papelera.
 * ───────────────────────────────────────────────────────────────────────────
 */
describe('borrar no se puede hacer por accidente', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
    vi.clearAllMocks()
  })

  const PREVIA = {
    direccion: 'ana@empresa.mx',
    nombre: 'Ana Ruiz',
    sigueSiendoContacto: true,
    direcciones: 2,
    campos: 3,
    pertenencias: 1,
    envios: 4,
    tieneBaja: false,
  }

  /** Monta la pantalla y llega hasta la vista previa del borrado. */
  async function hastaLaVistaPrevia(respuesta: unknown = PREVIA) {
    const invocar = vi.fn().mockResolvedValue(respuesta)
    vi.doMock('@/app/nucleo', async () => {
      const real = await vi.importActual<typeof import('@/app/nucleo')>('@/app/nucleo')
      return { ...real, hayNucleo: () => true, invocar }
    })
    const { default: Pantalla } = await import('./PantallaDerechos.vue')
    const w = mount(Pantalla, entorno)
    // El segundo formulario es el del borrado; el primero es el del informe.
    const form = w.findAll('form')[1]!
    await form.find('input').setValue('ana@empresa.mx')
    await form.trigger('submit')
    await flushPromises()
    return { w, invocar }
  }

  /**
   * LA GARANTÍA CENTRAL: sin escribir la dirección, no se borra.
   *
   * Se comprueba llamando al envío del formulario directamente, **no
   * pulsando el botón**: un botón deshabilitado es una capa de pintura, y lo
   * que hay que garantizar es que la función no haga nada. Si alguien quitara
   * el `deshabilitado` de la plantilla, esta prueba seguiría protegiendo.
   */
  it('sin escribir la dirección, no llama al núcleo', async () => {
    const { w, invocar } = await hastaLaVistaPrevia()
    invocar.mockClear()

    await w.find('.borrar__confirmar').trigger('submit')
    await flushPromises()

    expect(
      invocar,
      'se borró sin confirmar: el accidente que esto existe para impedir',
    ).not.toHaveBeenCalled()
  })

  /** Con la dirección MAL escrita tampoco. Una letra de más no vale. */
  it('con la dirección mal escrita, tampoco', async () => {
    const { w, invocar } = await hastaLaVistaPrevia()
    invocar.mockClear()

    await w.find('.borrar__confirmar input').setValue('ana@empresa.m')
    await w.find('.borrar__confirmar').trigger('submit')
    await flushPromises()

    expect(invocar).not.toHaveBeenCalled()
  })

  /**
   * Y con la dirección bien escrita sí. La comparación NO distingue
   * mayúsculas: quien copia de un correo se trae «Ana@Empresa.MX», y
   * rechazarlo sería pedir que teclee con más cuidado que el propio validador.
   */
  it('con la dirección bien escrita sí borra, y no distingue mayúsculas', async () => {
    const { w, invocar } = await hastaLaVistaPrevia()
    invocar.mockClear()
    invocar.mockResolvedValue({
      contactosBorrados: 1,
      canalesBorrados: 2,
      camposBorrados: 3,
      pertenenciasBorradas: 1,
      enviosAnonimizados: 4,
      audienciasAnonimizadas: 1,
      laBajaYaExistia: false,
    })

    await w.find('.borrar__confirmar input').setValue('Ana@Empresa.MX')
    await w.find('.borrar__confirmar').trigger('submit')
    await flushPromises()

    expect(invocar).toHaveBeenCalledWith('cancelar_datos_de', {
      canal: 'email',
      direccion: 'ana@empresa.mx',
    })
  })

  /**
   * Cambiar la dirección de arriba **olvida la confirmación**.
   *
   * Sin esto, alguien podría mirar lo que se borraría de Ana, confirmar
   * escribiendo su correo, cambiar de idea y teclear otra dirección arriba… y
   * el botón seguiría armado, apuntando ahora a otra persona.
   */
  it('cambiar la dirección de arriba desarma la confirmación', async () => {
    const { w } = await hastaLaVistaPrevia()
    await w.find('.borrar__confirmar input').setValue('ana@empresa.mx')
    expect(w.find('.borrar').exists()).toBe(true)

    const form = w.findAll('form')[1]!
    await form.find('input').setValue('otra@empresa.mx')
    await flushPromises()

    expect(
      w.find('.borrar').exists(),
      'la confirmación sobrevivió al cambio de dirección: el botón apunta a otra persona',
    ).toBe(false)
  })

  /**
   * Se enseñan las CIFRAS antes de decidir (§94). Un «¿seguro?» sobre algo
   * que no se ha visto no pregunta nada.
   */
  it('enseña qué desaparece y qué se conserva, con números', async () => {
    const { w } = await hastaLaVistaPrevia()
    const texto = w.find('.borrar').text()

    expect(texto).toContain('6') // 2 direcciones + 3 campos + 1 pertenencia
    expect(texto).toContain('4') // envíos que se conservan
    expect(texto).toContain('Desaparece')
    expect(texto).toContain('Se conserva')
  })

  /** Y a quién se está borrando, por su nombre. Teclear bien no es reconocer. */
  it('enseña de quién es esa dirección', async () => {
    const { w } = await hastaLaVistaPrevia()
    expect(w.find('.borrar').text()).toContain('Ana Ruiz')
  })

  /**
   * Cuando no hay nada, se dice — y se puede borrar igual, porque la baja se
   * crea de todos modos. Es la parte que impide que vuelva a entrar.
   */
  it('si no consta nada, lo dice y deja borrar igual', async () => {
    const { w } = await hastaLaVistaPrevia({
      direccion: 'nadie@empresa.mx',
      nombre: '',
      sigueSiendoContacto: false,
      direcciones: 0,
      campos: 0,
      pertenencias: 0,
      envios: 0,
      tieneBaja: false,
    })

    expect(w.find('.borrar').text()).toContain('No consta nada')
    expect(w.find('.borrar__confirmar').exists()).toBe(true)
  })
})
