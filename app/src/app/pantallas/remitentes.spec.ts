import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useActividadStore, type Cupo } from '@/app/stores/actividad'
import { useRemitentesStore, type Remitente } from '@/app/stores/remitentes'

import PantallaRemitentes from './PantallaRemitentes.vue'

const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' },
      Teleport: true,
    },
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

/**
 * Un cupo de remitente para la pantalla, con lo que se quiera cambiar encima.
 *
 * La dirección **tiene que ser la de `CUENTA`**: la pantalla cruza las dos
 * listas por dirección, porque el identificador de la cuenta no cruza la
 * frontera en la de remitentes. Con otra dirección la ficha no se pinta y la
 * prueba pasaría en verde sin mirar nada.
 */
function cupo(sobre: Partial<Cupo> = {}): Cupo {
  return {
    nombre: 'TELEMETRY',
    direccion: CUENTA.remitente,
    gastadoHoy: 0,
    topeDiario: 200,
    quedanHoy: 200,
    gastadoEstaHora: 0,
    topePorHora: 20,
    campanas: ['Clientes de agosto', 'Promoción de octubre'],
    hayReparto: true,
    riesgo: 'remitentes.riesgo.arriesgado',
    ...sobre,
  }
}

function sembrar(cuentas: Remitente[]): ReturnType<typeof useRemitentesStore> {
  const store = useRemitentesStore()
  store.remitentes = cuentas
  store.cargada = true
  return store
}

describe('la pantalla de Remitentes', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * **Sin cuentas, se dice que no hay ninguna y por qué importa.**
   *
   * Una lista vacía sin explicación deja a alguien mirando una pantalla en
   * blanco sin saber que es el paso que le falta para poder enviar.
   */
  it('sin cuentas configuradas lo dice y explica qué hacer', () => {
    sembrar([])
    const p = mount(PantallaRemitentes, entorno)
    const texto = p.text()
    expect(texto).toContain('Todavía no hay ninguna cuenta')
    expect(texto).toContain('contraseña de aplicación')
  })

  /**
   * **La cuenta se enseña con sus cifras, no con adjetivos** (§94).
   *
   * El servidor, el puerto y los topes de envío están en pantalla: son lo que
   * alguien compara cuando algo no funciona.
   */
  it('enseña el servidor, el puerto y los límites de cada cuenta', () => {
    sembrar([CUENTA])
    const texto = mount(PantallaRemitentes, entorno).text()
    expect(texto).toContain('ventas@telemetry.mx')
    expect(texto).toContain('smtp.gmail.com:587')
    expect(texto).toContain('50')
    expect(texto).toContain('10')
    expect(texto).toContain('Lista para enviar')
  })

  /**
   * **Una cuenta que hay que configurar no se enseña como lista.**
   *
   * Es lo que deja la migración V7 en las cuentas viejas, y lo que deja un
   * guardado cuya prueba no pasó. Si la pantalla la pintara igual que una que
   * funciona, el fallo aparecería al arrancar una campaña.
   *
   * Verificada rompiéndola el 23/09/2026: pintando siempre el estado
   * `connected`, la prueba falla.
   */
  it('una cuenta por configurar se distingue de una que funciona', () => {
    sembrar([{ ...CUENTA, estado: 'needs_reauth' }])
    const texto = mount(PantallaRemitentes, entorno).text()
    expect(texto).toContain('Hay que configurarla')
    expect(texto).not.toContain('Lista para enviar')
  })

  /**
   * **El diagnóstico de la prueba se enseña entero** (§95).
   *
   * Las tres partes: qué pasó, cómo se arregla y qué está a salvo. Un «no se
   * pudo conectar» a secas deja a alguien probando puertos media hora.
   */
  it('enseña las tres partes del diagnóstico cuando la prueba falla', () => {
    const store = sembrar([CUENTA])
    store.diagnostico = 'correo.prueba.contrasena_rechazada'
    store.laCuentaSirve = false

    const texto = mount(PantallaRemitentes, entorno).text()
    expect(texto).toContain('rechazó la contraseña')
    // El «cómo»: el camino exacto de la contraseña de aplicación.
    expect(texto).toContain('contraseña de aplicación')
    // El «qué está a salvo».
    expect(texto).toContain('No se envió ningún correo')
  })

  /**
   * **Y cuando la conexión no se puede cifrar, dice que no hay forma de
   * desactivarlo.**
   *
   * Es el punto donde alguien pide la casilla prohibida (§29). Dejar el texto
   * ambiguo es invitar a que la pidan otra vez, y a que alguien la escriba.
   */
  it('ante un fallo de cifrado dice que no se puede desactivar', () => {
    const store = sembrar([CUENTA])
    store.diagnostico = 'correo.prueba.sin_cifrado'
    store.laCuentaSirve = false

    const texto = mount(PantallaRemitentes, entorno).text()
    expect(texto).toContain('no hay forma de desactivarlo')
  })

  /**
   * **La pantalla dice dónde acaba la contraseña.**
   *
   * Es la pregunta que se hace cualquiera al escribir la contraseña de su
   * correo en un programa recién instalado, y la respuesta va en la pantalla,
   * no en un documento que nadie abre.
   */
  it('dice dónde se guarda la contraseña, junto al campo', async () => {
    sembrar([])
    const p = mount(PantallaRemitentes, entorno)
    await p.findAll('button').find((b) => b.text().includes('Añadir'))?.trigger('click')

    const texto = p.text()
    expect(texto).toContain('llavero de tu sistema')
    expect(texto).toContain('no en los archivos de ARLES')
  })

  /**
   * **La contraseña no sobrevive a cerrar el formulario.**
   *
   * Es la garantía que justifica que el valor viva en la pantalla y no en la
   * tienda: la tienda dura lo que dura la aplicación, y este valor tiene que
   * morir al cancelar. Guardarla «para no pedirla otra vez» la dejaría en la
   * memoria de la webview, que es donde no tiene que estar (ADR-0011).
   *
   * ── LO QUE ESTA PRUEBA CAZA, Y LO QUE NO ──
   *
   * Verificada rompiéndola el 23/09/2026, y con una corrección: quitar la línea
   * que vacía la contraseña en `cerrar()` **no la hace fallar**, porque abrir el
   * formulario también la vacía. Hay que tumbar los dos sitios para que falle,
   * y entonces sí.
   *
   * O sea que esto comprueba lo que el usuario puede ver —que el campo llega
   * vacío—, no cuál de los dos sitios lo consigue. El vaciado en `cerrar()` es
   * defensa en profundidad y no es observable desde fuera: la contraseña deja
   * de estar en memoria antes, aunque nadie vuelva a abrir el formulario. Se
   * deja escrito para que nadie lo quite creyendo que hay una prueba detrás.
   */
  it('la contraseña escrita no sobrevive a cancelar el formulario', async () => {
    sembrar([])
    const p = mount(PantallaRemitentes, entorno)

    const abrir = async (): Promise<void> => {
      await p.findAll('button').find((b) => b.text().includes('Añadir'))?.trigger('click')
    }

    await abrir()
    const campo = p.find('input[type="password"]')
    expect(campo.exists()).toBe(true)
    await campo.setValue('la-contrasena-de-aplicacion')

    // Cancelar, y volver a abrir.
    await p.findAll('button').find((b) => b.text().includes('Cancelar'))?.trigger('click')
    await abrir()

    const otraVez = p.find('input[type="password"]')
    expect(otraVez.exists()).toBe(true)
    expect((otraVez.element as HTMLInputElement).value).toBe('')
    // Y tampoco se quedó en el HTML por ninguna otra vía.
    expect(p.html()).not.toContain('la-contrasena-de-aplicacion')
  })

  /**
   * **El formulario de una cuenta nueva llega con los datos de Gmail puestos.**
   *
   * Es la que va a usar casi todo el mundo (D-1), y equivocarse en el puerto es
   * el error más común: 465 con STARTTLS deja esperando a un servidor que no va
   * a contestar.
   */
  it('una cuenta nueva llega con los datos de Gmail rellenos', async () => {
    sembrar([])
    const p = mount(PantallaRemitentes, entorno)
    await p.findAll('button').find((b) => b.text().includes('Añadir'))?.trigger('click')

    const valores = p.findAll('input').map((i) => (i.element as HTMLInputElement).value)
    expect(valores).toContain('smtp.gmail.com')
    expect(valores).toContain('587')
  })
})

/**
 * Mandar un correo de prueba.
 *
 * Es el botón que cierra R-24 desde la máquina del usuario: «Probar conexión»
 * dice que la cuenta autentica, y esto dice que **puede enviar**, que no es lo
 * mismo — una cuenta puede autenticar y que el servidor rechace el remitente.
 */
describe('mandar un correo de prueba desde Remitentes', () => {
  beforeEach(() => setActivePinia(createPinia()))

  async function abrirEnvio(p: ReturnType<typeof mount>): Promise<void> {
    await p
      .findAll('button')
      .find((b) => b.text().includes('Mandar un correo de prueba'))
      ?.trigger('click')
  }

  /**
   * **El formulario avisa de que se manda un correo de verdad.**
   *
   * Es lo que distingue este botón de «Probar conexión», que no manda nada.
   * Sin la advertencia, alguien lo pulsa creyendo que sólo comprueba, y sale un
   * correo.
   */
  it('dice que el correo se envía de verdad', async () => {
    sembrar([CUENTA])
    const p = mount(PantallaRemitentes, entorno)
    await abrirEnvio(p)

    expect(p.text()).toContain('Se envía un correo real')
  })

  /**
   * **Propone la propia dirección de la cuenta.**
   *
   * Mandarse la prueba a uno mismo es la única forma de ver si llegó, y
   * escribirla a mano es un paso que no aporta nada.
   */
  it('propone la dirección de la propia cuenta', async () => {
    sembrar([CUENTA])
    const p = mount(PantallaRemitentes, entorno)
    await abrirEnvio(p)

    const valores = p.findAll('input').map((i) => (i.element as HTMLInputElement).value)
    expect(valores).toContain(CUENTA.remitente)
  })

  /**
   * **Cuando sale, dice «aceptó» y NUNCA «entregado»** (§65).
   *
   * ARLES no puede saber si el correo llegó a una bandeja: sabe que el servidor
   * se hizo cargo. Éste es el punto de toda la aplicación donde más tienta
   * decir lo contrario, porque el usuario acaba de pulsar un botón y quiere un
   * sí.
   *
   * Verificada rompiéndola el 29/09/2026: cambiando el texto a «Correo
   * entregado», esta prueba falla.
   */
  it('al salir dice que el servidor lo aceptó, no que llegó', async () => {
    const store = sembrar([CUENTA])
    const p = mount(PantallaRemitentes, entorno)
    await abrirEnvio(p)

    store.resultadoDelEnvio = {
      aceptado: true,
      aDonde: 'ventas@telemetry.mx',
    }
    await p.vm.$nextTick()

    const texto = p.text()
    expect(texto).toContain('El servidor aceptó el correo')
    expect(texto).toContain('no puede saber si llegó')
    expect(texto).not.toContain('entregado')
    expect(texto).not.toContain('Entregado')
  })

  /**
   * **Y cuando no sale, dice las tres cosas, incluida la que más importa.**
   *
   * Ante «esa dirección no existe», quien acaba de mandarse una prueba a sí
   * mismo tiene que poder ver que **no se ha dado de baja**: en una campaña ese
   * mismo rechazo del servidor sí suprime.
   */
  it('ante un rechazo del destinatario aclara que no se dio de baja a nadie', async () => {
    const store = sembrar([CUENTA])
    const p = mount(PantallaRemitentes, entorno)
    await abrirEnvio(p)

    store.resultadoDelEnvio = {
      aceptado: false,
      problema: 'correo.prueba.envio.esaDireccionNoExiste',
      aDonde: 'noexiste@telemetry.mx',
    }
    await p.vm.$nextTick()

    const texto = p.text()
    expect(texto).toContain('esa dirección no existe')
    expect(texto).toContain('No se dio de baja a nadie')
  })

  /**
   * Sin haber mandado nada, no se anuncia ningún resultado.
   *
   * Sin esta prueba, las dos de arriba pasarían igual con un aviso pintado
   * siempre.
   */
  it('no anuncia ningún resultado antes de mandar', async () => {
    sembrar([CUENTA])
    const p = mount(PantallaRemitentes, entorno)
    await abrirEnvio(p)

    const texto = p.text()
    expect(texto).not.toContain('El servidor aceptó el correo')
    expect(texto).not.toContain('No se dio de baja a nadie')
  })
  // ──────────────────────────────────────────────────────────────────────
  // EL CUPO REAL DEL BUZÓN Y SU SEMÁFORO (D-13 y D-14, 22/10/2026)
  // ──────────────────────────────────────────────────────────────────────

  /**
   * **La cifra que Dirección pidió ver: lo que el buzón manda al día, sumado.**
   *
   * Lo pidió así: «que en la pestaña de remitentes se viera en realidad cuántos
   * correos está enviando por día, sumando los correos que se envían en ambas
   * campañas». Desde la D-13 esa suma es la única cifra sin techo, así que es
   * la que tiene que estar delante antes de activar nada.
   *
   * Y con separador de miles, que es el defecto que este proyecto ha tenido
   * siete veces —«12536 filas», «1303 días»— sobre números perfectamente
   * correctos.
   */
  it('enseña lo que el buzón manda al día, con las campañas que se lo reparten', async () => {
    sembrar([CUENTA])
    const cifras = useActividadStore()
    cifras.remitentes = [cupo({ topeDiario: 1200, gastadoHoy: 340 })]

    const p = mount(PantallaRemitentes, entorno)
    await p.vm.$nextTick()
    const texto = p.text()

    expect(texto).toContain('1,200 correos al día')
    expect(texto).toContain('340 enviados hoy')
    expect(texto).toContain('Clientes de agosto')
    expect(texto).toContain('Promoción de octubre')
  })

  /**
   * **El semáforo lleva TEXTO, no sólo color.**
   *
   * Es la regla que dejó escrita la auditoría D del §140: *ningún estado se
   * distingue sólo por color*. Un punto rojo no lo lee quien no distingue el
   * rojo del verde, y es justo la persona a la que hay que avisar.
   *
   * Y dice **qué hacer** (§95), no sólo cómo está: un semáforo en rojo sin
   * salida es un susto, no un aviso. La salida que da es la de ADR-0008 —un
   * remitente por campaña—, que es la que de verdad resuelve.
   */
  it('el semáforo dice en palabras que se está arriesgando el dominio', async () => {
    sembrar([CUENTA])
    const cifras = useActividadStore()
    cifras.remitentes = [
      cupo({ topeDiario: 400, riesgo: 'remitentes.riesgo.arriesgado' }),
    ]

    const p = mount(PantallaRemitentes, entorno)
    await p.vm.$nextTick()
    const texto = p.text()

    expect(texto).toContain('Estás arriesgando el dominio')
    expect(texto).toContain('remitentes distintos')
  })

  /**
   * **Y en verde no asusta.** Un indicador que siempre está en rojo se deja de
   * mirar a los dos días, así que hay que poder ver que el verde existe.
   */
  it('con poco volumen el semáforo está tranquilo', async () => {
    sembrar([CUENTA])
    const cifras = useActividadStore()
    cifras.remitentes = [
      cupo({ topeDiario: 50, riesgo: 'remitentes.riesgo.tranquilo' }),
    ]

    const p = mount(PantallaRemitentes, entorno)
    await p.vm.$nextTick()
    const texto = p.text()

    expect(texto).toContain('Volumen tranquilo')
    expect(texto).not.toContain('Estás arriesgando el dominio')
  })

  /**
   * **Un buzón sin ninguna campaña en marcha no manda nada, y eso no es cero.**
   *
   * `topeDiario` llega en `null` y la pantalla tiene que decirlo con palabras.
   * «0 correos al día» se leería como un tope puesto a cero —una campaña
   * parada a propósito—, que es otra cosa y llevaría a buscar un problema que
   * no existe.
   */
  it('un buzón que ninguna campaña usa lo dice, en vez de enseñar un cero', async () => {
    sembrar([CUENTA])
    const cifras = useActividadStore()
    cifras.remitentes = [cupo({ topeDiario: null, campanas: [] })]

    const p = mount(PantallaRemitentes, entorno)
    await p.vm.$nextTick()
    const texto = p.text()

    expect(texto).toContain('Ninguna campaña en marcha lo usa')
    expect(texto).not.toContain('0 correos al día')
  })

  /**
   * **Y «Propone 50 al día» dejó de decir «Hasta».**
   *
   * El límite de la cuenta ya no es un techo desde la D-13: es lo que el
   * asistente rellena al elegirla, y cada campaña puede subirlo. Llamarlo
   * «hasta» prometía un límite que no existe — y justo al lado está la cifra
   * que sí manda.
   */
  it('el límite de la cuenta se presenta como una propuesta y no como un techo', () => {
    sembrar([CUENTA])
    const p = mount(PantallaRemitentes, entorno)
    const texto = p.text()

    expect(texto).toContain('Propone 50 al día')
    expect(texto).not.toContain('Hasta 50 al día')
  })
})
