import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
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
