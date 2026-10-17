import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { createMemoryHistory, createRouter } from 'vue-router'

import { i18n } from '@/app/i18n'
import { useCampanasStore, type Campana, type Revision } from '@/app/stores/campanas'
import { useRemitentesStore, type Remitente } from '@/app/stores/remitentes'

import AsistenteDeCampana from './AsistenteDeCampana.vue'
import PantallaCampanas from './PantallaCampanas.vue'

/**
 * Un router de verdad, no un `mock` de `$router`.
 *
 * Las pantallas usan `useRouter()`, que lee de la inyección y **no** de
 * `$router`. Con el simulacro, Vue avisaba de que la inyección no existía y las
 * pruebas pasaban igual — hasta que alguien pulsara un botón que navega, que es
 * lo único que ese simulacro pretendía cubrir.
 *
 * Encontrado el 29/09/2026 leyendo los avisos de una tanda que estaba en verde.
 */
const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    { path: '/campanas', name: 'campanas', component: { template: '<div />' } },
    { path: '/campanas/nueva', name: 'campanaNueva', component: { template: '<div />' } },
  ],
})

const entorno = {
  global: {
    plugins: [i18n, router],
    stubs: { Teleport: true },
  },
}

const CUENTA: Remitente = {
  id: '01900000-0000-7000-8000-00000000000a',
  servidor: 'smtp.gmail.com',
  puerto: 587,
  cifrado: 'starttls',
  usuario: 'ventas@telemetry.mx',
  remitente: 'ventas@telemetry.mx',
  nombreVisible: 'TELEMETRY',
  estado: 'connected',
  limiteDiario: 50,
  limitePorHora: 10,
}

const CAMPANA: Campana = {
  id: '01900000-0000-7000-8000-0000000000c1',
  nombre: 'Septiembre',
  estado: 'campana.estado.enviando',
  audiencia: 1342,
  aceptados: 87,
}

/** Un preflight con todo en orden menos lo que se le diga. */
function revision(sobreescribe: Partial<Revision> = {}): Revision {
  return {
    preflight: {
      puntos: [
        { punto: 'laCuentaAutentica', veredicto: 'pasa' },
        { punto: 'laCredencialSirve', veredicto: 'pasa' },
        { punto: 'laAudienciaNoEstaVacia', veredicto: 'pasa' },
        { punto: 'lasVariablesSeResuelven', veredicto: 'pasa' },
        { punto: 'elHorarioAlcanza', veredicto: 'pasa' },
        { punto: 'hayConexion', veredicto: 'pasa' },
        { punto: 'seHizoElEnvioDePrueba', veredicto: 'pasa' },
        { punto: 'losRegistrosDelDominio', veredicto: 'sinComprobar' },
      ],
      resultado: { resultado: 'lista', avisos: 1 },
    },
    audiencia: 1342,
    contactosConHuecos: 34,
    estimacion: {
      // La forma de verdad que manda Rust: el enum va etiquetado con
      // `desenlace`. Esta línea decía `{ resultado: 'termina' }` hasta la
      // 8.6 E, igual que el tipo del store — y compilaba, porque nadie leía el
      // campo. Al conectarlo, el tipo corregido dejó la prueba en rojo, que es
      // lo que una prueba sobre un tipo que miente tenía que hacer desde el
      // principio.
      desenlace: { desenlace: 'termina', cuando: '24 de enero' },
      audiencia: 1342,
      techoDiario: 50,
      quienManda: 'elTopeDelDia',
      diasConEnvio: 27,
    },
    diagnostico: 'correo.prueba.correcto',
    ...sobreescribe,
  }
}

describe('la lista de campañas', () => {
  beforeEach(async () => {
    setActivePinia(createPinia())
    // El router de memoria arranca sin ruta: sin esto avisa de que no encuentra
    // «», y ese aviso taparía uno de verdad el día que lo haya.
    await router.push('/campanas')
    await router.isReady()
  })

  /**
   * **La columna dice «Aceptados», y en ninguna parte dice «entregado»** (§65).
   *
   * Es la mentira más fácil de contar de todo el producto, y la más cara: ARLES
   * sabe que el servidor se hizo cargo del mensaje, no que llegara a nadie.
   *
   * Verificada rompiéndola el 29/09/2026: cambiando el texto de
   * `campanas.columna.aceptados` a «Entregados», esta prueba falla en las dos
   * comprobaciones.
   */
  it('nunca dice «entregado»', () => {
    const store = useCampanasStore()
    store.campanas = [CAMPANA]
    store.cargada = true

    const texto = mount(PantallaCampanas, entorno).text()
    expect(texto).toContain('Aceptados')
    expect(texto.toLowerCase()).not.toContain('entregad')
  })

  /** Las cifras se enseñan como cifras (§94). */
  it('enseña los números, no adjetivos', () => {
    const store = useCampanasStore()
    store.campanas = [CAMPANA]
    store.cargada = true

    const texto = mount(PantallaCampanas, entorno).text()
    expect(texto).toContain('1,342')
    expect(texto).toContain('87')
  })
})

describe('los estados de campaña', () => {
  /**
   * **Las siete claves que emite Rust tienen su rótulo, y el respaldo existe.**
   *
   * La lista viva está en `EstadoDeCampana::clave_i18n`. Una clave sin texto no
   * da ningún error: la tabla enseñaría `campana.estado.enPausa` en crudo, y
   * sólo se descubre con una campaña en ese estado concreto delante.
   *
   * El respaldo se comprueba aparte porque la pantalla cae en él cuando falta
   * un texto — y si el respaldo tampoco existiera, la red de seguridad sería la
   * que enseñara la clave.
   */
  it('las siete tienen rótulo, y hay respaldo', () => {
    const ESTADOS = [
      'borrador',
      'lista',
      'programada',
      'enviando',
      'enPausa',
      'detenida',
      'terminada',
      'desconocido',
    ]
    for (const estado of ESTADOS) {
      const clave = `campana.estado.${estado}`
      expect(String(i18n.global.t(clave)), clave).not.toBe(clave)
    }
  })
})

describe('el asistente de campaña', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * **Los nueve pasos tienen título, y ninguno enseña su clave en crudo.**
   *
   * Sin esto, un paso sin texto sale como `asistente.paso.preflight.titulo` en
   * la cabecera, y sólo se ve llegando a ese paso concreto — que es el séptimo.
   *
   * Ya pasó: al escribir estas pruebas faltaba justo el del preflight, y las
   * demás pasaban en verde con el aviso de `vue-i18n` en la salida de error,
   * donde nadie lo mira.
   */
  it('los nueve pasos tienen su título', () => {
    const PASOS = [
      'informacion',
      'audiencia',
      'remitente',
      'mensaje',
      'limites',
      'simulacion',
      'preflight',
      'prueba',
      'activacion',
    ]
    for (const paso of PASOS) {
      const clave = `asistente.paso.${paso}.titulo`
      expect(String(i18n.global.t(clave)), clave).not.toBe(clave)
    }
  })

  function montar(): ReturnType<typeof mount> {
    const remitentes = useRemitentesStore()
    remitentes.remitentes = [CUENTA]
    remitentes.cargada = true

    const campanas = useCampanasStore()
    campanas.cargada = true
    return mount(AsistenteDeCampana, entorno)
  }

  /** §94: dónde estás, con números, no «ya casi». */
  it('dice en qué paso vas de cuántos', () => {
    expect(montar().text()).toContain('Paso 1 de 9')
  })

  /**
   * **Sólo se ofrecen cuentas que ya conectan.**
   *
   * Ofrecer una cuenta rota construye una campaña entera que el preflight va a
   * bloquear cuatro pasos después, y para entonces ya se escribió el mensaje y
   * se eligió el horario.
   */
  it('no ofrece cuentas que no pueden enviar', async () => {
    const remitentes = useRemitentesStore()
    remitentes.remitentes = [{ ...CUENTA, estado: 'needs_reauth' }]
    remitentes.cargada = true
    useCampanasStore().cargada = true

    const p = mount(AsistenteDeCampana, entorno)
    // @ts-expect-error acceso al estado interno para situarse en el paso 3
    p.vm.paso = 3
    await p.vm.$nextTick()

    expect(p.text()).toContain('No hay ninguna cuenta lista para enviar')
    expect(p.text()).not.toContain('ventas@telemetry.mx')
  })

  /**
   * **El aviso del §48 aparece a partir de 51, no de 50.**
   *
   * El umbral es «más de 50». Un `>=` obligaría a aceptar un riesgo en el caso
   * que el propio §48 pone como seguro, y nadie lo notaría salvo por una
   * casilla de más — que es justo lo que enseña a ignorar los avisos.
   *
   * Verificada rompiéndola el 29/09/2026: cambiando el `>` por `>=` en
   * `avisaDelVolumen`, la primera comprobación falla.
   */
  it('avisa del volumen a partir de 51 diarios, y no antes', async () => {
    const p = montar()
    // @ts-expect-error acceso al estado interno para situarse en el paso 5
    p.vm.paso = 5
    // @ts-expect-error idem
    p.vm.limiteDiario = '50'
    await p.vm.$nextTick()
    expect(p.text()).not.toContain('tiene riesgo')

    // @ts-expect-error idem
    p.vm.limiteDiario = '51'
    await p.vm.$nextTick()
    expect(p.text()).toContain('tiene riesgo')
    // §154: se informa, no se prohíbe. Y se dice que no se prohíbe.
    expect(p.text()).toContain('la decisión es tuya')
  })

  /**
   * **«Sin comprobar» no se lee como un aprobado.**
   *
   * Es la regla que gobierna el preflight entero: un punto sin comprobar
   * bloquea igual que uno que falla. Si la pantalla lo pintara como un aviso
   * menor, el botón apagado no tendría explicación visible.
   */
  it('en el preflight, lo que no se pudo mirar se dice', async () => {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.revision = revision()

    const p = montar()
    // @ts-expect-error acceso al estado interno para situarse en el paso 7
    p.vm.paso = 7
    await p.vm.$nextTick()

    const texto = p.text()
    expect(texto).toContain('Sin comprobar')
    expect(texto).toContain('no lo da por bueno')
    expect(texto).toContain('impide activar')
  })

  /**
   * **El botón de activar lo apaga el preflight, no la pantalla.**
   *
   * Con el veredicto en «bloqueada», activar no se puede pulsar — y se dice a
   * dónde volver. Un botón apagado sin explicación se lee como una avería.
   *
   * Verificada rompiéndola el 29/09/2026: quitando el `:deshabilitado` del
   * botón de activar, esta prueba falla.
   */
  it('con el preflight bloqueado no se puede activar', async () => {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.revision = revision({
      preflight: {
        puntos: [{ punto: 'hayConexion', veredicto: 'falla' }],
        resultado: { resultado: 'bloqueada', fallan: 1, sinComprobar: 0 },
      },
    })

    const p = montar()
    // @ts-expect-error acceso al estado interno para situarse en el paso 9
    p.vm.paso = 9
    await p.vm.$nextTick()

    const boton = p.findAll('button').find((b) => b.text().includes('Activar campaña'))
    expect(boton?.attributes('disabled')).toBeDefined()
    expect(p.text()).toContain('Vuelve al paso 7')
  })

  /**
   * **Antes de activar se dice que no se deshace, y a cuánta gente llega.**
   *
   * Una confirmación sin cifra no es una confirmación informada, y esto congela
   * la audiencia y empieza a mandar correos que no vuelven.
   */
  it('avisa de lo que cuesta activar, con la cifra delante', async () => {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.revision = revision()

    const p = montar()
    // @ts-expect-error acceso al estado interno para situarse en el paso 9
    p.vm.paso = 9
    await p.vm.$nextTick()

    const texto = p.text()
    expect(texto).toContain('Activar no se puede deshacer')
    expect(texto).toContain('1342')
  })

  /**
   * **La simulación habla en castellano llano, con las dos cifras.**
   *
   * Es, según `UX_NAVEGACION.md` §5, la información más valiosa de ARLES: lo
   * que hace que alguien reconsidere **antes** de lanzar. Y la cifra de a
   * cuántos les falta un dato se enseña como cifra: con 34 de 1 342 se sigue,
   * con 900 se para a revisar la lista.
   */
  it('la simulación dice cuántos, a qué ritmo y a cuántos les falta un dato', async () => {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.revision = revision()

    const p = montar()
    // @ts-expect-error acceso al estado interno para situarse en el paso 6
    p.vm.paso = 6
    await p.vm.$nextTick()

    const texto = p.text()
    expect(texto).toContain('1,342 correos')
    expect(texto).toContain('50 al día')
    expect(texto).toContain('27 días')
    expect(texto).toContain('34 de 1342')
  })

  // ═══════════════════════════════════════════════════════════════════════
  // EL DESENLACE · conectado en la 8.6 E
  //
  // El núcleo calcula los tres desde la Fase 4 y los tres textos estaban
  // escritos desde entonces. La pantalla pintaba el techo y los días, y nada
  // más: con 451 196 personas a 50 diarios decía «1 303 días» —una cifra
  // correcta— y NUNCA decía que 386 046 personas no iban a recibir nada.
  //
  // Las tres pruebas se vieron fallar quitando los `AAviso` del paso 6.
  // ═══════════════════════════════════════════════════════════════════════

  /** Pone el asistente en el paso de la simulación con la revisión que se diga. */
  async function enLaSimulacion(rev: Revision): Promise<ReturnType<typeof mount>> {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.revision = rev
    const p = montar()
    // @ts-expect-error acceso al estado interno para situarse en el paso 6
    p.vm.paso = 6
    await p.vm.$nextTick()
    return p
  }

  /**
   * **Una campaña que no termina lo dice, y dice a cuánta gente deja fuera.**
   *
   * Es la mitad que faltaba: sin la cifra, «no termina» no permite decidir
   * entre subir el tope y partir la campaña (§94).
   */
  it('avisa cuando la campaña no termina en cinco años, con la cifra', async () => {
    const p = await enLaSimulacion(
      revision({
        audiencia: 451_196,
        estimacion: {
          desenlace: { desenlace: 'noCabeEnElHorizonte', faltan: 386_046 },
          audiencia: 451_196,
          techoDiario: 50,
          quienManda: 'elTopeDelDia',
          diasConEnvio: 1303,
        },
      }),
    )

    const texto = p.text()
    expect(texto).toContain('no termina en cinco años')
    expect(texto, 'la cifra con separador de miles (§94, es-MX)').toContain('386,046')
  })

  /** **Y con el tope a cero también, que es el otro caso que el núcleo caza.** */
  it('avisa cuando con esos topes no sale ni un mensaje', async () => {
    const p = await enLaSimulacion(
      revision({
        estimacion: {
          desenlace: { desenlace: 'noAvanza' },
          audiencia: 1342,
          techoDiario: 0,
          quienManda: 'elTopeDelDia',
          diasConEnvio: 0,
        },
      }),
    )
    expect(p.text()).toContain('no saldría ni un mensaje')
  })

  /**
   * **El aviso de espera, por encima de 30 000 destinatarios.**
   *
   * Medido: 4 s con 45 000 y 73 s con 451 000. No es un tope —una campaña de
   * 136 000 que termina en nueve meses es razonable— es convertir un cuelgue
   * en una espera explicada (§95).
   */
  it('avisa de cuánto va a tardar en prepararse una campaña grande', async () => {
    const p = await enLaSimulacion(
      revision({
        audiencia: 451_196,
        estimacion: {
          desenlace: { desenlace: 'termina', cuando: '3 de marzo' },
          audiencia: 451_196,
          techoDiario: 500,
          quienManda: 'elTopeDelDia',
          diasConEnvio: 903,
        },
      }),
    )
    const texto = p.text()
    expect(texto).toContain('segundos en preparar')
    expect(texto, 'no cerrar ARLES es la parte accionable (§95)').toContain('no cierres ARLES')
  })

  /** **Y una campaña normal no lo enseña**, o el aviso se deja de leer. */
  it('no avisa de la espera con una audiencia normal', async () => {
    const p = await enLaSimulacion(revision())
    expect(p.text()).not.toContain('segundos en preparar')
  })
  // ═══════════════════════════════════════════════════════════════════════
  // LA PARADA DE EMERGENCIA · enchufada en la 7.3
  //
  // La Fase 4 la construyó, la 7.0 le puso sus tres comandos con sus pruebas, y
  // **ninguna pantalla los llamaba**. La comprobación que la 7.0 escribió para
  // este patrón sólo miraba pausar, reanudar y detener: verificaba tres de seis
  // y pasaba en verde. Estas dos pruebas miran los otros tres.
  // ═══════════════════════════════════════════════════════════════════════

  /**
   * **Con la parada puesta, la pantalla lo dice y ofrece levantarla.**
   *
   * Sin el aviso, cada cifra de la tabla es de algo que no está saliendo y no
   * hay nada que lo diga: alguien mira «enviando, 340 de 1 342» y espera a que
   * avance para siempre.
   *
   * Verificada rompiéndola el 30/09/2026 quitando el bloque del aviso: falla
   * porque no encuentra el texto de la parada ni el botón de reanudar.
   */
  it('avisa de la parada de emergencia y ofrece reanudar', async () => {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.campanas = [CAMPANA]
    campanas.parada = {
      parado: true,
      desde: '2026-09-30T16:40:00Z',
      por: 'Adrián',
      clave: 'campana.parada.puesta',
    }

    const w = mount(PantallaCampanas, entorno)
    await w.vm.$nextTick()

    expect(w.text()).toContain('no sale nada de ninguna campaña')
    // §95: las tres partes, no sólo el qué.
    expect(w.text()).toContain('sigue por enviar')
    const botones = w.findAll('button').map((b) => b.text())
    expect(botones.join(' | ')).toContain('Reanudar los envíos')
    // Y no ofrece parar otra vez lo que ya está parado.
    expect(botones.join(' | ')).not.toContain('Detener todos los envíos')
  })

  /**
   * **Y sin la parada puesta, el botón de pararlo todo está.**
   *
   * Es la mitad que faltaba: un aviso que sólo aparece cuando ya está parado no
   * sirve de nada si no hay forma de pararlo. Ésta es la prueba que habría
   * fallado desde la 7.0.
   */
  it('con el envío activo ofrece la parada de emergencia', async () => {
    const campanas = useCampanasStore()
    campanas.cargada = true
    campanas.campanas = [CAMPANA]
    campanas.parada = { parado: false, desde: null, por: null, clave: 'campana.parada.levantada' }

    const w = mount(PantallaCampanas, entorno)
    await w.vm.$nextTick()

    const botones = w.findAll('button').map((b) => b.text())
    expect(botones.join(' | ')).toContain('Detener todos los envíos')
    expect(w.text()).not.toContain('no sale nada de ninguna campaña')
  })
})

/**
 * El paso 5, que dejaba la campaña sin salida · H-06 y H-10, 16/10/2026
 *
 * ─────────────────────────────────────────────────────────────────────────
 * DIRECCIÓN NO PUDO CREAR NINGUNA CAMPAÑA
 *
 * El desplegable de **Horario de envío** salía vacío, y sin elegir uno
 * «Continuar» no deja pasar. Las opciones son las ventanas guardadas, y
 * **ninguna pantalla podía crear una**: `guardar_ventana` existía, el store lo
 * envolvía, y nadie lo llamaba. Es el patrón del §9 por decimoctava vez.
 *
 * La propia lista de Inicio lo decía —«Definir una ventana de envío · Llega en
 * la entrega 6»— mientras el asistente la exigía para avanzar.
 *
 * **Y la vista previa no podía enseñarlo**: su demo trae una ventana ya
 * guardada, así que ahí el desplegable nunca estuvo vacío. Un decorado con más
 * datos que la realidad esconde justo los caminos de quien empieza.
 */
describe('el paso 5 sin ningún horario guardado', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  function enElPaso5(ventanas: unknown[] = []) {
    const remitentes = useRemitentesStore()
    remitentes.remitentes = [CUENTA]
    remitentes.cargada = true
    const campanas = useCampanasStore()
    campanas.cargada = true
    // @ts-expect-error se fija el estado del store para el caso bajo prueba
    campanas.ventanas = ventanas
    const p = mount(AsistenteDeCampana, entorno)
    // @ts-expect-error acceso al estado interno para situarse en el paso 5
    p.vm.paso = 5
    return p
  }

  it('NO deja a nadie sin salida: ofrece crear uno', async () => {
    const p = enElPaso5([])
    await p.vm.$nextTick()
    expect(p.text()).toContain('Todavía no has definido ningún horario')
    expect(p.text()).toContain('Usar lunes a viernes')
  })

  it('con horarios guardados no estorba con el aviso', async () => {
    const p = enElPaso5([
      { id: 'v1', nombre: 'Días laborables', dias: 31, desde: 9, hasta: 18, zona: 'America/Monterrey' },
    ])
    await p.vm.$nextTick()
    expect(p.text()).not.toContain('Todavía no has definido ningún horario')
    expect(p.text()).toContain('Horario de envío')
  })

  /**
   * ── «EL DE LA CUENTA» ES UN ADJETIVO; «50» ES UNA CIFRA (H-10) ──────────
   *
   * La ayuda decía «Déjalo en blanco para usar el de la cuenta» **sin decir
   * cuál**, así que se aceptaba un número sin saberlo. Y es un número que
   * importa: por encima de 50 ARLES exige aceptar un aviso (§48).
   */
  it('dice CUÁL es el máximo diario que se va a usar', async () => {
    const p = enElPaso5([])
    // @ts-expect-error se elige la cuenta, que es de donde sale la cifra
    p.vm.cuenta = CUENTA.id
    await p.vm.$nextTick()
    expect(p.text()).toContain(`se usan ${CUENTA.limiteDiario} al día`)
    expect(p.text()).toContain(`se usan ${CUENTA.limitePorHora} por hora`)
  })

  it('y sin cuenta elegida no se inventa una cifra', async () => {
    const p = enElPaso5([])
    await p.vm.$nextTick()
    // Sin cuenta no hay tope que nombrar, y decir uno sería inventarlo.
    expect(p.text()).toContain('Déjalo en blanco para usar el de la cuenta')
    expect(p.text()).not.toMatch(/se usan \d+ al día/)
  })
})
