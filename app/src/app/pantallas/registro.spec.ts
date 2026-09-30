import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/app/i18n'
import { es } from '@/app/locales/es'
import { MARCADORES_DE_RAZON } from '@/app/razones'
import { useCampanasStore } from '@/app/stores/campanas'
import { useRegistroStore, type Suceso } from '@/app/stores/registro'

import PantallaRegistro from './PantallaRegistro.vue'

/**
 * Un router de verdad, no un `mocks: { $router }`.
 *
 * Aquí importa más que en ninguna otra pantalla: **los filtros viajan en la
 * dirección**, así que un router simulado dejaría sin comprobar justo lo que
 * esta pantalla hace de distinto.
 */
const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    { path: '/actividad', name: 'actividad', component: { template: '<div />' } },
    { path: '/actividad/registro', name: 'registro', component: { template: '<div />' } },
  ],
})

const entorno = { global: { plugins: [i18n, router], stubs: { Teleport: true } } }

/**
 * Los montajes vivos, para desmontarlos al acabar cada prueba.
 *
 * ── POR QUÉ HACE FALTA AQUÍ Y NO EN LAS DEMÁS PANTALLAS ──
 *
 * Esta pantalla **observa la ruta**. Un componente de una prueba anterior que
 * siga montado sigue reaccionando a los `router.push` de la siguiente, y lo que
 * falla entonces no es el comportamiento sino la contaminación: la prueba pasa
 * sola y falla en grupo, que es la peor forma de fallar porque parece
 * intermitente.
 */
const montados: { unmount: () => void }[] = []

function montar() {
  const w = mount(PantallaRegistro, entorno)
  montados.push(w)
  return w
}

function suceso(sobre: Partial<Suceso> = {}): Suceso {
  return {
    id: 'r-1',
    clave: 'motor.aceptadoPorElProveedor',
    esDesenlace: true,
    campana: 'k-1',
    canal: 'email',
    direccion: 'ana@empresa.mx',
    numeroA: null,
    numeroB: null,
    cuando: '2026-09-30T17:48:00Z',
    ...sobre,
  }
}

/** Deja el store como si el comando ya hubiera contestado. */
function conDatos(sucesos: Suceso[], total = sucesos.length) {
  const s = useRegistroStore()
  s.sucesos = sucesos
  s.total = total
  s.porPagina = 500
  s.clases = [
    { clave: 'motor.aceptadoPorElProveedor', cuantos: 2 },
    { clave: 'reintento.direccionInexistente', cuantos: 1 },
  ]
  s.cargada = true
  return s
}

describe('el registro de actividad', () => {
  beforeEach(async () => {
    setActivePinia(createPinia())
    await router.push({ name: 'registro', query: {} })
  })

  afterEach(() => {
    while (montados.length > 0) montados.pop()?.unmount()
  })

  /**
   * **El tope nunca esconde el tamaño: «500 de 3 412».**
   *
   * ── LA MITAD QUE HACE QUE EL TOPE NO MIENTA ──
   *
   * Quinientas filas sin la segunda cifra se leen como «esto es todo», y quien
   * busca algo de la semana pasada y no lo ve concluye que no pasó. D-7C lo
   * pide explícitamente.
   *
   * Verificada rompiéndola el 30/09/2026 haciendo que `cuantos` use siempre
   * `registro.todos`: falla porque no aparece el total.
   */
  it('con más de los que caben, dice cuántos hay en total', async () => {
    conDatos([suceso()], 3412)
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('3412')
    expect(w.text()).toContain('Usa los filtros')
  })

  /**
   * **Y cuando caben todos, no dice «1 de 1».**
   *
   * Sin esta segunda prueba, una pantalla que enseñara siempre las dos cifras
   * pasaría la de arriba — y «17 de 17 registros» hace pensar que falta algo.
   */
  it('cuando caben todos, enseña una sola cifra', async () => {
    conDatos([suceso(), suceso({ id: 'r-2' })])
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('2 registros')
    expect(w.text()).not.toContain('Usa los filtros')
  })

  /**
   * **Los filtros viajan en la dirección.**
   *
   * `#/actividad/registro?texto=ana` se puede volver a abrir, guardar o pasar a
   * alguien. Sin eso, «mira lo que pasó con Ana» obliga a explicar qué teclear.
   *
   * Verificada rompiéndola: haciendo que `cambiar` llame a `registro.filtrar`
   * en vez de escribir la ruta, la dirección se queda vacía y falla.
   */
  it('escribe los filtros en la dirección', async () => {
    conDatos([suceso()])
    const w = montar()
    await w.vm.$nextTick()

    const entradas = w.findAll('input')
    const texto = entradas[entradas.length - 1]
    expect(texto, 'el cuadro de búsqueda por dirección').toBeDefined()
    await texto!.setValue('ana')
    // `cambiar` navega, y navegar es asíncrono: sin esto se comprueba la ruta
    // antes de que el `router.push` haya terminado, y la prueba falla por el
    // tiempo y no por el comportamiento.
    await flushPromises()

    expect(router.currentRoute.value.query['texto']).toBe('ana')
  })

  /**
   * **Y al abrir con filtros en la dirección, se aplican antes de consultar.**
   *
   * Entrar por un enlace con filtros y ver primero la lista entera es un
   * parpadeo que hace dudar de si el enlace funcionó.
   */
  it('lee los filtros de la dirección al entrar', async () => {
    conDatos([suceso()])
    await router.push({ name: 'registro', query: { texto: 'ana', clase: 'motor.parado' } })

    const w = montar()
    await w.vm.$nextTick()

    const s = useRegistroStore()
    expect(s.filtro.texto).toBe('ana')
    expect(s.filtro.clave).toBe('motor.parado')
  })

  /**
   * **Un suceso del motor no inventa un destinatario.**
   *
   * «Fuera de horario» no es de nadie. Una celda vacía se lee como un dato que
   * falta; decirlo es la diferencia entre un hueco y una explicación.
   */
  it('dice que un suceso del motor no es de nadie', async () => {
    conDatos([
      suceso({ clave: 'motor.fueraDeVentana', esDesenlace: false, direccion: null }),
    ])
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('Del motor, no de nadie')
  })

  /**
   * **§65 · la aclaración de «aceptado» está a la vista.**
   *
   * El plan lo pide así: «con su aclaración a la vista, no en un pie de
   * página». Quien lee «aceptado» y entiende «entregado» toma una decisión
   * sobre datos que ARLES no tiene.
   */
  it('explica qué quiere decir «aceptado» (§65)', async () => {
    conDatos([suceso()])
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('el servidor se hizo cargo')
    expect(w.text().toLowerCase()).not.toContain('entregado')
  })

  /**
   * **El recorte dice qué NO se toca antes de ofrecer el botón.**
   *
   * Es la tercera cosa del §95 y la que hace que alguien se atreva a pulsar.
   * Sin ella, «quitar 3 412 registros» se lee como «borrar el historial».
   *
   * Verificada rompiéndola quitando la línea del «salvo»: falla porque no
   * aparece la cifra de los que se quedan.
   */
  it('antes de recortar dice qué se va y qué se queda', async () => {
    const s = conDatos([suceso()])
    s.recorte = { seIrian: 3412, seQuedan: 18900, antesDe: '2026-07-31T00:00:00Z' }

    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('3412')
    expect(w.text()).toContain('18900')
    // La garantía que hace que alguien se atreva a pulsar: los envíos siguen.
    expect(w.text()).toContain('se quedan')
    expect(w.text()).toContain('todos los envíos')
    // Y sin asteriscos: `vue-i18n` no interpreta Markdown y saldrían literales.
    expect(w.text(), 'los asteriscos de Markdown salen tal cual').not.toContain('**')
  })

  /**
   * **Sin nada que recortar, no se ofrece el botón.**
   *
   * Un botón destructivo que no hace nada enseña a pulsarlo sin leer, que es
   * como se acaba pulsando el día que sí hace algo.
   */
  it('sin nada viejo no ofrece recortar', async () => {
    const s = conDatos([suceso()])
    s.recorte = { seIrian: 0, seQuedan: 120, antesDe: '2026-07-31T00:00:00Z' }

    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('No hay nada que recortar')
    const botones = w.findAll('button').map((b) => b.text())
    expect(botones.join(' | ')).not.toContain('Quitar los')
  })

  /**
   * **El desplegable de «qué pasó» lleva la cifra de cada opción.**
   *
   * Una opción sin cifra puede estar vacía, y quien la elige y no ve nada no
   * sabe si es que no pasó o si el filtro está roto.
   */
  it('cada clase del desplegable dice cuántas hay', async () => {
    conDatos([suceso()])
    const w = montar()
    await w.vm.$nextTick()

    const opciones = w.findAll('option').map((o) => o.text())
    expect(opciones.some((o) => o.includes('(2)'))).toBe(true)
  })

  /**
   * **Sin resultados, se ofrece quitar los filtros.**
   *
   * Es el callejón sin salida más fácil de crear: una lista vacía con seis
   * filtros puestos y nada que los quite.
   */
  it('sin resultados ofrece quitar los filtros', async () => {
    conDatos([], 0)
    // El filtro se pone **en la ruta** y no en el store: la pantalla lo relee
    // de la dirección al montarse, así que un filtro puesto sólo en el store se
    // pierde. Escribirlo aquí en el store hacía pasar la prueba por el camino
    // equivocado.
    await router.push({ name: 'registro', query: { texto: 'nadie' } })

    const w = montar()
    await flushPromises()

    expect(w.text()).toContain('Ningún registro coincide')
    const botones = w.findAll('button').map((b) => b.text())
    expect(botones.join(' | ')).toContain('Quitar los filtros')
  })

  /**
   * **Y el vacío de verdad no habla de filtros.**
   *
   * Sin nada registrado todavía, «prueba a quitar algún filtro» es un consejo
   * que no lleva a ninguna parte.
   */
  it('el registro vacío no culpa a los filtros', async () => {
    conDatos([], 0)
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('Todavía no hay nada registrado')
    expect(w.text()).not.toContain('Ningún registro coincide')
  })

  /**
   * **La campaña se enseña por su nombre, no por su identificador.**
   *
   * `01900000-0000-7000-8000-0000000000c3` no le dice nada a nadie.
   */
  it('enseña el nombre de la campaña', async () => {
    conDatos([suceso({ campana: 'k-9' })])
    const campanas = useCampanasStore()
    campanas.campanas = [
      {
        id: 'k-9',
        nombre: 'Clientes de agosto',
        estado: 'campana.estado.enviando',
        audiencia: 10,
        aceptados: 3,
      },
    ]

    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('Clientes de agosto')
    expect(w.text()).not.toContain('k-9')
  })
  /**
   * **«Reintento 2 de 3», con las dos cifras y cada una en su sitio.**
   *
   * ── EL DEFECTO QUE ESTO CAZA ──
   *
   * La primera versión pasaba `numeroA` a los dos marcadores, y la tabla decía
   * «Reintento 2 de **2**»: un número plausible, coherente consigo mismo y
   * falso. Ninguna comprobación de tipos lo ve —los dos son `number | null`— ni
   * ninguna de texto, porque la frase se forma igual de bien. Se vio leyendo la
   * demostración.
   *
   * Verificada rompiéndola el 30/09/2026 devolviendo a `de: s.numeroA`: la
   * tabla dice «Reintento 2 de 2» y falla.
   */
  it('las dos cifras de un reintento van cada una en su sitio', async () => {
    conDatos([
      suceso({ clave: 'reintento.seVolveraAIntentar', numeroA: 2, numeroB: 3 }),
    ])
    const w = montar()
    await w.vm.$nextTick()

    expect(w.text()).toContain('Reintento 2 de 3')
    expect(w.text(), 'el total de intentos sale de numeroB').not.toContain('Reintento 2 de 2')
  })

  /**
   * **Toda clase que el motor puede anotar tiene nombre corto en el filtro.**
   *
   * Sin él, el desplegable enseña la frase con sus marcadores sin rellenar:
   * «Reintento de», «Se alcanzó el tope de envíos por hora». Se entiende a
   * medias y se lee como un texto roto.
   *
   * Las claves se leen de `MARCADORES_DE_RAZON` y de los desenlaces que el
   * motor anota, no de una lista escrita aquí: una lista a mano se queda vieja
   * el día que hace falta que esto salte.
   */
  it('ninguna clase se queda sin nombre corto en el filtro', () => {
    const catalogo = es.registro.clase as Record<string, string>
    const delMotor = [
      ...Object.keys(MARCADORES_DE_RAZON),
      'motor.aceptadoPorElProveedor',
      'motor.cancelado',
      'reintento.seVolveraAIntentar',
      'reintento.seAgotaronLosIntentos',
      'reintento.direccionInexistente',
      'reintento.laCuentaNoAutentica',
      'reintento.pidioLaBaja',
    ]

    for (const clave of delMotor) {
      const corta = clave.split('.').pop() ?? ''
      expect(
        Object.prototype.hasOwnProperty.call(catalogo, corta),
        `${clave}: el desplegable la ofrecería con su frase larga y sus marcadores`,
      ).toBe(true)
    }
  })
})
