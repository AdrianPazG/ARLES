import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import App from '@/app/App.vue'
import { i18n } from '@/app/i18n'
import { router, subseccionesDe } from '@/app/router'

/**
 * La barra lateral dice dónde estás · O-3, 21/09/2026
 *
 * ───────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTAS PRUEBAS NO ESTÁN EN `pantallas.spec.ts`
 *
 * Porque allí `RouterLink` es un **doble**, y un doble no calcula ningún
 * estado activo. Ése es exactamente el motivo de que el fallo que Dirección
 * encontró el 21/09 sobreviviera desde la entrega 3.2: estando en
 * `/contactos/importar`, `/contactos/supresion` o `/inicio/configuracion`, la
 * barra se quedaba **entera apagada**, y ninguna prueba podía verlo.
 *
 * Aquí se monta `App.vue` con el enrutador REAL. Es más lento y monta más
 * cosas; es el precio de poder afirmar algo sobre navegación.
 *
 * VERIFICADO ROMPIÉNDOLO (regla de trabajo de CLAUDE.md §2). Se devolvió la
 * barra al marcado automático de vue-router —`.router-link-active` en vez de
 * la clase `activo` calculada desde `meta.seccion`— y los tres casos de
 * «marca la sección madre» fallaron, cada uno nombrando su ruta. Restaurado
 * después.
 * ───────────────────────────────────────────────────────────────────────────
 */

const entorno = {
  global: {
    plugins: [i18n, router],
    stubs: { Teleport: true },
  },
}

/** Monta la aplicación ya situada en `ruta`. */
async function en(ruta: string) {
  await router.push(ruta)
  await router.isReady()
  const w = mount(App, entorno)
  await w.vm.$nextTick()
  return w
}

/** El enlace de la barra que lleva a `destino`, con su clase. */
function enlaceDeBarra(w: ReturnType<typeof mount>, destino: string) {
  return w
    .findAll('a.nav-enlace')
    .find((a) => a.attributes('href')?.endsWith(destino))
}

describe('la barra marca la sección, también en las subpantallas', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * Los tres casos que estaban rotos. Se escriben los tres y no uno:
   * `/inicio/configuracion` es el que Dirección vio, pero los otros dos
   * llevaban el mismo fallo y se habrían quedado rotos si sólo se arreglara
   * el que se veía.
   */
  const casos = [
    { ruta: '/inicio/configuracion', seccion: 'inicio' },
    { ruta: '/contactos/importar', seccion: 'contactos' },
    { ruta: '/contactos/supresion', seccion: 'contactos' },
  ] as const

  for (const { ruta, seccion } of casos) {
    it(`en ${ruta} la sección ${seccion} queda marcada`, async () => {
      const w = await en(ruta)

      const madre = enlaceDeBarra(w, `/${seccion}`)
      expect(madre, `no hay enlace a /${seccion} en la barra`).toBeTruthy()
      expect(
        madre?.classes(),
        `estando en ${ruta}, la barra se queda apagada: nadie sabe en qué sección está`,
      ).toContain('activo')
    })
  }

  /** Y en una pantalla raíz sigue marcándose, que es lo que ya funcionaba. */
  it('en /contactos la sección contactos queda marcada', async () => {
    const w = await en('/contactos')
    expect(enlaceDeBarra(w, '/contactos')?.classes()).toContain('activo')
  })

  /**
   * La otra mitad: marcar la madre no vale si se marcan TODAS. La primera
   * versión de esto podría haber puesto `activo` en las seis y las pruebas de
   * arriba habrían pasado igual.
   */
  it('sólo una sección queda marcada a la vez', async () => {
    const w = await en('/inicio/configuracion')
    const marcados = w
      .findAll('a.nav-enlace')
      .filter((a) => a.classes().includes('activo'))
    expect(marcados).toHaveLength(1)
  })
})

describe('la barra marca también la subsección', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('bajo Inicio aparece Configuración, y marcada', async () => {
    const w = await en('/inicio/configuracion')

    const sub = w
      .findAll('a.sub-enlace')
      .find((a) => a.attributes('href')?.endsWith('/inicio/configuracion'))
    expect(sub, 'la subsección no se enseña en la barra').toBeTruthy()
    expect(sub?.classes(), 'la subsección no se marca').toContain('activo')
    expect(sub?.attributes('aria-current')).toBe('page')
  })

  /*
   * AQUÍ NO VA LA BARRA DE ACENTO DE LA SUBSECCIÓN, Y ES A PROPÓSITO
   *
   * Se dibuja con `.sub-enlace::before`, y jsdom no calcula pseudoelementos:
   * cualquier prueba que escribiera aquí acabaría afirmando otra vez que la
   * clase `activo` está puesta —cosa que ya comprueba la prueba de arriba— y
   * pasaría igual con la regla de CSS borrada. Una prueba decorativa es peor
   * que ninguna: ocupa el sitio de la que sí mide.
   *
   * La mide `sonda:vista-previa` en Chromium, leyendo la altura real del
   * pseudoelemento. Ver la nota de §19 en la hoja de estilos de App.vue.
   */

  /**
   * Las subsecciones de las OTRAS secciones no se despliegan. Sin esto, la
   * barra pasa de seis entradas a nueve permanentes y deja de ser una brújula.
   */
  it('no se despliegan las subsecciones de las demás secciones', async () => {
    const w = await en('/inicio/configuracion')

    const rutas = w.findAll('a.sub-enlace').map((a) => a.attributes('href'))
    expect(rutas.some((r) => r?.includes('/contactos/'))).toBe(false)
  })

  /**
   * En una pantalla raíz la sección abierta SÍ enseña sus hijas: es como se
   * descubre que existen. Ésta es la que hace innecesario el puente que
   * Dirección mandó quitar de Ajustes — si fallara, la configuración volvería
   * a no encontrarse desde ningún sitio.
   */
  it('desde /inicio ya se ve Configuración sin tener que entrar', async () => {
    const w = await en('/inicio')

    const sub = w
      .findAll('a.sub-enlace')
      .find((a) => a.attributes('href')?.endsWith('/inicio/configuracion'))
    expect(
      sub,
      'sin esto la configuración no se encuentra desde ninguna parte: Ajustes ya no tiene el enlace',
    ).toBeTruthy()
    // Se ve, pero no está marcada: no estamos dentro.
    expect(sub?.classes()).not.toContain('activo')
  })
})

describe('las migas dicen dónde estás', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('en una subsección se enseña «Inicio › Configuración»', async () => {
    const w = await en('/inicio/configuracion')

    const migas = w.find('.migas')
    expect(migas.exists(), 'no hay migas en una subsección').toBe(true)
    expect(migas.text()).toContain('Inicio')
    expect(migas.text()).toContain('Configuración')

    // La primera miga vuelve a la sección; la última es la página actual y no
    // es un enlace —un enlace a donde ya estás no lleva a ninguna parte—.
    expect(migas.find('a').attributes('href')).toContain('/inicio')
    expect(migas.find('[aria-current="page"]').exists()).toBe(true)
  })

  /**
   * En una pantalla raíz no hay migas. Una miga sola —«Inicio»— ocupa una
   * línea para no decir ni de dónde vienes ni a dónde volver.
   */
  it('en una pantalla raíz no hay migas', async () => {
    const w = await en('/inicio')
    expect(w.find('.migas').exists()).toBe(false)
  })
})

describe('las subsecciones salen de las rutas, no de una lista aparte', () => {
  /**
   * `subseccionesDe` deriva de `router.getRoutes()`. Si alguien añadiera una
   * segunda lista escrita a mano, esta prueba no la vería — así que lo que se
   * comprueba es que la derivación hace lo que dice: sólo las de esa sección,
   * y en el orden declarado.
   */
  it('devuelve las de la sección, ordenadas', () => {
    expect(subseccionesDe('inicio').map((s) => s.ruta)).toEqual([
      '/inicio/configuracion',
    ])
    expect(subseccionesDe('contactos').map((s) => s.ruta)).toEqual([
      '/contactos/importar',
      '/contactos/supresion',
    ])
  })

  /** Una sección sin hijas devuelve una lista vacía, no `undefined`. */
  it('una sección sin subsecciones devuelve una lista vacía', () => {
    expect(subseccionesDe('ajustes')).toEqual([])
    expect(subseccionesDe('remitentes')).toEqual([])
  })

  /** Toda subsección tiene texto: §139, claves de i18n con traducción. */
  it('toda subsección tiene nombre en español', () => {
    for (const seccion of ['inicio', 'contactos'] as const) {
      for (const sub of subseccionesDe(seccion)) {
        const texto = i18n.global.t(sub.clave)
        expect(texto, `${sub.clave} no tiene texto`).not.toBe(sub.clave)
        expect(texto.length).toBeGreaterThan(0)
      }
    }
  })
})
