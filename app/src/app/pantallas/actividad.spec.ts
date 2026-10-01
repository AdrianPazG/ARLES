import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/app/i18n'
import {
  useActividadStore,
  type CifrasDeCampana,
  type Cupo,
} from '@/app/stores/actividad'
import { useCampanasStore } from '@/app/stores/campanas'

import PantallaActividad from './PantallaActividad.vue'

/**
 * Un router de verdad y no un `mocks: { $router }`.
 *
 * La pantalla usa `RouterLink`, que lee de la inyección. Con el simulacro, Vue
 * avisa y la prueba pasa igual — trampa 6 de la Fase 6.
 */
const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    {
      path: '/actividad',
      name: 'actividad',
      component: { template: '<div />' },
    },
    { path: '/campanas', name: 'campanas', component: { template: '<div />' } },
    {
      path: '/campanas/nueva',
      name: 'campanaNueva',
      component: { template: '<div />' },
    },
    {
      path: '/remitentes',
      name: 'remitentes',
      component: { template: '<div />' },
    },
    {
      path: '/contactos/supresion',
      name: 'supresion',
      component: { template: '<div />' },
    },
  ],
})

const entorno = {
  global: { plugins: [i18n, router], stubs: { Teleport: true } },
}

function campana(sobre: Partial<CifrasDeCampana> = {}): CifrasDeCampana {
  return {
    id: '01900000-0000-7000-8000-0000000000c1',
    nombre: 'Clientes de agosto',
    estado: 'campana.estado.enviando',
    audiencia: 1000,
    salio: 340,
    pendiente: 650,
    fallo: 10,
    dadoDeBaja: 3,
    cancelado: 0,
    respondieron: 23,
    ritmo: [{ hora: '2026-09-30T12', salieron: 60 }],
    razon: null,
    ...sobre,
  }
}

const CUPO: Cupo = {
  nombre: 'Ventas',
  direccion: 'ventas@telemetry.mx',
  gastadoHoy: 460,
  topeDiario: 500,
  quedanHoy: 40,
  gastadoEstaHora: 100,
  topePorHora: 100,
  campanas: ['Clientes de agosto', 'Promoción de octubre'],
  hayReparto: true,
}

/** Deja el store como si el comando ya hubiera contestado. */
function conDatos(campanas: CifrasDeCampana[], remitentes: Cupo[] = []) {
  const s = useActividadStore()
  s.campanas = campanas
  s.remitentes = remitentes
  s.totalSalio = campanas.reduce((t, c) => t + c.salio, 0)
  s.totalPendiente = campanas.reduce((t, c) => t + c.pendiente, 0)
  s.totalFallo = campanas.reduce((t, c) => t + c.fallo, 0)
  s.cargada = true
  s.actualizadoA = new Date()
  return s
}

describe('la pantalla de Actividad', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  /**
   * **D-7A · las tres campañas, cada una con sus cifras.**
   *
   * Dirección trabaja con hasta tres a la vez, y un total no dice cuál va bien.
   * Sin esta prueba, una pantalla que enseñara sólo los totales pasaría todo lo
   * demás: las cifras estarían bien y serían inútiles.
   */
  it('enseña una tarjeta por campaña, no sólo el total', async () => {
    conDatos([
      campana({ id: 'a', nombre: 'Clientes de agosto' }),
      campana({ id: 'b', nombre: 'Promoción de octubre' }),
      campana({
        id: 'c',
        nombre: 'Reactivación',
        estado: 'campana.estado.enPausa',
        ritmo: [],
      }),
    ])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    const texto = w.text()
    expect(texto).toContain('Clientes de agosto')
    expect(texto).toContain('Promoción de octubre')
    expect(texto).toContain('Reactivación')
  })

  /**
   * **D-7A · la cifra de quién contestó, y nunca lo que dijo.**
   *
   * Es la línea que Dirección trazó el 29/09/2026: leer las respuestas es
   * trabajo del asesor. Esta prueba comprueba la cifra; que el texto no viaje
   * lo garantiza el esquema, que no tiene columna donde guardarlo.
   */
  it('dice cuántos contestaron', async () => {
    conDatos([campana({ respondieron: 23 })])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('Contestaron')
    expect(w.text()).toContain('23')
  })

  /**
   * **§65 · en ninguna parte de esta pantalla dice «entregado».**
   *
   * ARLES no puede saber si un correo llegó a un buzón: sabe que el proveedor lo
   * aceptó. Verificada rompiéndola el 30/09/2026 cambiando el texto de
   * `actividad.salio` a «Entregados»: falla nombrando la palabra.
   */
  it('nunca dice «entregado» (§65)', async () => {
    conDatos([campana()], [CUPO])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text().toLowerCase()).not.toContain('entrega')
  })

  /**
   * **La razón se enseña cuando explica algo, y con su cifra.**
   *
   * Es el dato de más valor de la pantalla: un cero sin explicación es lo que
   * hace que alguien reinicie el programa, y reiniciar no arregla «sin cupo».
   *
   * Verificada rompiéndola: quitando el bloque `.razon` de la plantilla, falla
   * porque no encuentra el texto del tope.
   */
  it('dice por qué una campaña que envía no está sacando nada', async () => {
    conDatos([
      campana({
        ritmo: [],
        razon: {
          clave: 'ritmo.topeDeLaHora',
          numeroA: 100,
          numeroB: null,
          cuando: '2026-09-30T17:52:00Z',
        },
      }),
    ])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('Por qué no sale nada ahora')
    // La cifra rellenada, no el marcador.
    expect(w.text()).toContain('100')
    expect(
      w.text(),
      'el marcador {cabian} no puede salir a la pantalla',
    ).not.toContain('{cabian}')
  })

  /**
   * **Y no se enseña cuando la campaña está avanzando.**
   *
   * Sin esto, una campaña que pasó por «sin cupo» hace tres horas y ahora envía
   * con normalidad seguiría enseñando ese aviso, compitiendo con las cifras y
   * haciendo que alguien pare algo que funciona.
   */
  it('no enseña la razón de una campaña que sí está avanzando', async () => {
    conDatos([
      campana({
        ritmo: [{ hora: '2026-09-30T17', salieron: 60 }],
        razon: {
          clave: 'ritmo.topeDeLaHora',
          numeroA: 100,
          numeroB: null,
          cuando: '2026-09-30T14:00:00Z',
        },
      }),
    ])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).not.toContain('Por qué no sale nada ahora')
  })

  /**
   * **El reparto del cupo dice con quién se reparte.**
   *
   * Es lo que explica por qué dos campañas están quietas cuando una tercera se
   * comió el cupo. «100 de 100 esta hora» sin los nombres no dice de quién.
   */
  it('nombra las campañas que comparten un cupo', async () => {
    conDatos([campana()], [CUPO])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('Promoción de octubre')
    expect(w.text()).toContain('460')
  })

  /**
   * **Con datos viejos, lo dice antes de las cifras.**
   *
   * Si el refresco deja de contestar, lo que queda en pantalla son las cifras de
   * hace un rato y siguen pareciendo las de ahora. Alguien pararía una campaña
   * mirando un número congelado.
   *
   * Verificada rompiéndola: quitando el `v-if="actividad.estaViejo"` del aviso,
   * falla porque el texto no aparece.
   */
  it('avisa cuando las cifras dejaron de actualizarse', async () => {
    const s = conDatos([campana()])
    // Como si el último dato bueno fuera de hace un minuto.
    s.actualizadoA = new Date(Date.now() - 60_000)
    s.latir()
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(s.estaViejo, 'un minuto sin refrescar es viejo').toBe(true)
    expect(w.text()).toContain('no se han podido actualizar')
  })

  /**
   * **Y sin campañas, la pantalla no es un callejón.**
   *
   * La revisión a ojo del 29/09/2026 encontró Actividad con **cero acciones**:
   * se salía sólo por la barra lateral. Aquí se comprueba que hay a dónde ir.
   */
  it('sin campañas ofrece crear una', async () => {
    const s = useActividadStore()
    s.cargada = true
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('Todavía no hay ninguna campaña')
    expect(
      w.findAll('a').length,
      'una pantalla sin salidas es un callejón',
    ).toBeGreaterThan(0)
  })

  /**
   * **Con campañas, las tres salidas están.**
   *
   * Una de ellas es el freno: D-7B lo puso en Campañas, así que esta pantalla
   * tiene que llevar hasta allí o el usuario que ve algo mal no sabe adónde ir.
   */
  it('lleva al freno, a los remitentes y a las bajas', async () => {
    conDatos([campana()], [CUPO])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    const destinos = w.findAll('nav.salidas a').map((a) => a.attributes('href'))
    expect(destinos).toContain('/campanas')
    expect(destinos).toContain('/remitentes')
    expect(destinos).toContain('/contactos/supresion')
  })

  /**
   * **Los dos relojes se paran al salir de la pantalla.**
   *
   * Sin esto siguen corriendo para siempre: dos consultas por segundo a una base
   * que nadie está mirando, y en una aplicación de escritorio que se queda
   * abierta toda la jornada eso no se recupera solo.
   *
   * Se mide contando los intervalos vivos, no leyendo el código: un
   * `onBeforeUnmount` que existiera y no limpiara pasaría cualquier revisión.
   */
  it('para los relojes al desmontarse', async () => {
    conDatos([campana()])
    const vivos = new Set<unknown>()
    const ponerOriginal = window.setInterval
    const quitarOriginal = window.clearInterval
    // @ts-expect-error se sustituye por un doble que sólo cuenta
    window.setInterval = (...args: unknown[]) => {
      // @ts-expect-error se delega en el original
      const id = ponerOriginal(...args) as number
      vivos.add(id)
      return id
    }
    // @ts-expect-error idem
    window.clearInterval = (id: number) => {
      vivos.delete(id)
      quitarOriginal(id)
    }

    try {
      const w = mount(PantallaActividad, entorno)
      await w.vm.$nextTick()
      expect(
        vivos.size,
        'la pantalla pone dos relojes: refresco y latido',
      ).toBe(2)
      w.unmount()
      expect(vivos.size, 'y los dos se paran al salir').toBe(0)
    } finally {
      window.setInterval = ponerOriginal
      window.clearInterval = quitarOriginal
    }
  })

  /**
   * **Con ritmo cero no se estima nada.**
   *
   * Dividir el pendiente por un ritmo de cero da infinito, y redondearlo da un
   * número inventado que alguien leería como una promesa. Con ritmo cero lo que
   * hay que enseñar es la razón.
   */
  it('no estima cuánto falta cuando no sale nada', async () => {
    conDatos([campana({ ritmo: [], pendiente: 650 })])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('En las últimas 6 h no salió ninguno')
    expect(w.text()).not.toContain('a este ritmo')
    expect(w.text(), 'ni «Infinity» ni «NaN» delante de nadie').not.toMatch(
      /Infinity|NaN/,
    )
  })
  /**
   * **Con la parada de emergencia puesta, no dice «2 de 5 enviando».**
   *
   * Es la razón número uno por la que no sale nada, y esta pantalla no la
   * conocía: enseñaba la cifra correcta de un estado que ya no era cierto,
   * mientras el aviso vivía en Campañas a un clic de distancia. Se vio mirando
   * la demostración con la parada puesta y las once pruebas en verde.
   *
   * Verificada rompiéndola el 30/09/2026 quitando el aviso de la plantilla:
   * falla porque no encuentra el texto del envío detenido.
   */
  it('no dice que hay campañas enviando si el envío está detenido', async () => {
    conDatos([campana()])
    const campanas = useCampanasStore()
    campanas.parada = {
      parado: true,
      desde: '2026-09-30T16:40:00Z',
      por: null,
      clave: 'campana.parada.puesta',
    }

    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('El envío está detenido')
    expect(w.text(), 'la cifra de campañas enviando contradiría al aviso').not.toMatch(
      /\d+ de \d+ campañas enviando/,
    )
    // Y lleva a donde está el botón, sin duplicarlo aquí (D-7B).
    const destinos = w.findAll('a').map((a) => a.attributes('href'))
    expect(destinos).toContain('/campanas')
  })
  /**
   * **Cada campaña ofrece guardar su resumen.**
   *
   * En la tarjeta y no en una pantalla aparte: quien quiere el resumen de una
   * campaña la está mirando.
   *
   * Verificada rompiéndola el 30/09/2026 quitando el bloque `.exportar` de la
   * plantilla: falla porque no encuentra el botón.
   */
  it('ofrece guardar el resumen de cada campaña', async () => {
    conDatos([campana({ id: 'a' }), campana({ id: 'b', nombre: 'Promoción' })])
    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    const guardar = w.findAll('button').filter((b) => b.text().includes('Guardar el resumen'))
    expect(guardar.length, 'una por campaña').toBe(2)
  })

  /**
   * **Y al guardarlo dice qué lleva el archivo y qué no.**
   *
   * Quien lo va a mandar por correo tiene que saber **antes de mandarlo** que
   * no lleva direcciones dentro. Decirlo sólo dentro del archivo llega tarde:
   * para entonces ya se adjuntó.
   */
  it('al guardar dice que el archivo no lleva direcciones', async () => {
    const s = conDatos([campana()])
    s.ultimoResumen = 'resumen-Clientes-de-agosto.txt'

    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('resumen-Clientes-de-agosto.txt')
    expect(w.text()).toContain('No lleva ninguna dirección')
  })

  /**
   * **Lo que vuelve es el nombre del archivo, nunca la carpeta.**
   *
   * Regla 4 de la frontera: dónde guarda alguien sus cosas dice dónde vive, y
   * a esta pantalla no le aporta nada (`THREAT_MODEL.md` §4.1).
   */
  it('no enseña la carpeta donde se guardó', async () => {
    const s = conDatos([campana()])
    s.ultimoResumen = 'resumen-Clientes-de-agosto.txt'

    const w = mount(PantallaActividad, entorno)
    await w.vm.$nextTick()

    const t = w.text()
    expect(t).not.toMatch(/[A-Z]:\\|\/home\/|\/Users\//)
  })
})
