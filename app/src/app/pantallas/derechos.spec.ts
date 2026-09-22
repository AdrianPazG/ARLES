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
    expect(
      w.html(),
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
    expect(w.html()).not.toContain('peligro')
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
   * Honestidad del producto. Quien atiende una solicitud de borrado tiene que
   * enterarse **aquí**, antes de prometérselo a nadie, de que todavía no está
   * construido. Enterrarlo en la documentación equivale a ocultarlo.
   */
  it('avisa de que el borrado todavía no existe', () => {
    const w = mount(PantallaDerechos, entorno)
    expect(w.text()).toContain('Que borres sus datos')
    expect(w.text()).toContain('Todavía no está construido')
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
