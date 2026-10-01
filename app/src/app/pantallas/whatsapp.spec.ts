import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { useWhatsappStore, type Numero } from '@/app/stores/whatsapp'

import PantallaWhatsApp from './PantallaWhatsApp.vue'

const entorno = {
  global: {
    plugins: [i18n],
    stubs: {
      RouterLink: { props: ['to'], template: '<a :href="to"><slot /></a>' },
      Teleport: true,
    },
  },
}

/** Un número en frío, sin ninguna de las dos marcas y sin calidad conocida. */
const EN_FRIO: Numero = {
  id: '01900000-0000-7000-8000-0000000000a1',
  nombreVisible: 'Ventas',
  telefonoE164: '+528112345678',
  wabaId: 'waba-1',
  modo: 'prospeccion_directa',
  estado: 'connected',
  coexistenciaConfirmada: null,
  riesgoAceptado: null,
  calidad: null,
  limiteDeMeta: null,
  puedeEnviar: false,
  escribeEnFrio: true,
}

function sembrar(numeros: Numero[]): ReturnType<typeof useWhatsappStore> {
  const store = useWhatsappStore()
  store.numeros = numeros
  return store
}

describe('la pantalla del número de WhatsApp', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * **Sin número, se dice qué hace falta para dar uno de alta.**
   *
   * Un vacío que solo dice «no hay nada» deja a quien llega sin saber si le
   * falta configurar algo en ARLES o algo en Meta.
   */
  it('sin ningún número, explica qué hace falta', () => {
    sembrar([])
    const texto = mount(PantallaWhatsApp, entorno).text()

    expect(texto).toContain('Todavía no hay ningún número')
    expect(texto).toContain('Meta')
  })

  /**
   * **Las dos confirmaciones que faltan se piden, cada una con su motivo.**
   *
   * Es la garantía central de la pantalla: un número en frío recién dado de
   * alta tiene que enseñar los dos avisos, no uno.
   *
   * Verificada rompiéndola el 29/09/2026: envolviendo el aviso del riesgo en
   * `v-if="false"`, esta prueba falla.
   */
  it('pide las dos confirmaciones de un número en frío sin marcar', () => {
    sembrar([EN_FRIO])
    const texto = mount(PantallaWhatsApp, entorno).text()

    expect(texto).toContain('Coexistencia sin confirmar')
    expect(texto).toContain('escribir a quien no lo pidió')
    expect(texto).toContain('Todavía no puede enviar')
  })

  /**
   * **Un número de seguimiento no enseña el aviso del riesgo.**
   *
   * El aviso del modo en frío solo aplica a quien escribe en frío. Enseñarlo
   * siempre lo convertiría en decorado, y quien lo vea en el número donde sí
   * importa ya no lo leería.
   */
  it('no avisa del riesgo en un número que no escribe en frío', () => {
    sembrar([
      {
        ...EN_FRIO,
        modo: 'seguimiento',
        escribeEnFrio: false,
        coexistenciaConfirmada: '2026-09-29T00:00:00Z',
        puedeEnviar: true,
      },
    ])
    const texto = mount(PantallaWhatsApp, entorno).text()

    expect(texto).toContain('Puede enviar')
    expect(texto).not.toContain('escribir a quien no lo pidió')
  })

  /**
   * **La calidad que no se ha podido consultar NO se enseña como buena.**
   *
   * `calidad: null` significa «no se sabe». La pantalla tiene que decirlo con
   * esas palabras y no dejar un hueco ni pintar verde: un semáforo que se pone
   * en verde cuando no ha podido mirar es peor que no tener semáforo.
   *
   * Verificada rompiéndola el 29/09/2026: haciendo que `textoDeCalidad`
   * devuelva el texto de `green` cuando la calidad es nula, esta prueba falla.
   */
  it('la calidad sin consultar se dice, y no se pinta como alta', () => {
    sembrar([EN_FRIO])
    const texto = mount(PantallaWhatsApp, entorno).text()

    expect(texto).toContain('No consultada')
    expect(texto).toContain('no se sabe')
    expect(texto).not.toContain('Calidad del número, según Meta: Alta')
  })

  /**
   * **Las cifras son cifras** (§94).
   *
   * «2 de 3 pueden enviar hoy», no «casi todos listos».
   */
  it('cuenta cuántos pueden enviar, con cifras', () => {
    sembrar([
      EN_FRIO,
      { ...EN_FRIO, id: 'b', puedeEnviar: true },
      { ...EN_FRIO, id: 'c', puedeEnviar: true },
    ])
    const texto = mount(PantallaWhatsApp, entorno).text()

    expect(texto).toContain('2 de 3')
    expect(texto).toContain('1')
  })

  /**
   * **El token no queda en el DOM después de cerrar el formulario.**
   *
   * El campo es de tipo `password`, así que no se ve — pero el valor sigue en
   * el elemento hasta que alguien lo borra, y de ahí lo saca cualquier cosa
   * que recorra el formulario. Se comprueba que al cancelar queda vacío.
   */
  it('el token se borra del formulario al cancelar', async () => {
    sembrar([])
    const pantalla = mount(PantallaWhatsApp, entorno)

    await pantalla.findAll('button')[0]?.trigger('click')
    const campo = pantalla.find('input[type="password"]')
    await campo.setValue('EAAG-un-token-de-meta')
    expect((campo.element as HTMLInputElement).value).toBe('EAAG-un-token-de-meta')

    const botones = pantalla.findAll('button')
    const cancelar = botones.find((b) => b.text().includes('Cancelar'))
    await cancelar?.trigger('click')

    expect(pantalla.find('input[type="password"]').exists()).toBe(false)
    expect(pantalla.html()).not.toContain('EAAG-un-token-de-meta')
  })

  // ── El envío de prueba, que es lo que la 7.7 añadió ─────────────────────
  //
  // El comando existía desde la W.8 y ninguna pantalla lo llamaba, así que
  // desde ARLES no se podía mandar un WhatsApp. Estas cuatro pruebas miran lo
  // que ninguna prueba de Rust podía ver: que hay un botón, que llama al
  // comando, y que los dos desenlaces se enseñan por lo que son.

  /**
   * **Un número que todavía no puede enviar no tiene botón: tiene una
   * explicación.**
   *
   * El comando devolvería un error si el número le falta una confirmación. Un
   * botón cuyo único resultado posible es un error no es un botón, es una
   * trampa — y el §95 pide explicar, no fallar.
   */
  it('si el número no puede enviar todavía, no ofrece el botón', () => {
    sembrar([EN_FRIO])
    const pantalla = mount(PantallaWhatsApp, entorno)

    expect(pantalla.text()).toContain('Este número no puede enviar todavía')
    const mandar = pantalla.findAll('button').find((b) => b.text().includes('Mandar la prueba'))
    expect(mandar).toBeUndefined()
  })

  /**
   * **Cuando sí puede enviar, el botón está y llama al comando.**
   *
   * Esto es exactamente lo que faltaba: no que el comando funcione —eso ya
   * estaba probado en Rust— sino que alguien lo llame.
   */
  it('manda la prueba con el número escrito, la plantilla y el idioma', async () => {
    const store = sembrar([{ ...EN_FRIO, puedeEnviar: true }])
    const pedidas: unknown[][] = []
    store.enviarPrueba = (...args: unknown[]) => {
      pedidas.push(args)
      return Promise.resolve({ aceptado: true, clave: 'whatsapp.prueba.aceptado' })
    }

    const pantalla = mount(PantallaWhatsApp, entorno)
    const campos = pantalla.findAll('input')
    const destino = campos[campos.length - 3]
    await destino?.setValue('+528112345678')

    const mandar = pantalla.findAll('button').find((b) => b.text().includes('Mandar la prueba'))
    await mandar?.trigger('click')
    await pantalla.vm.$nextTick()

    expect(pedidas).toHaveLength(1)
    // Los cuatro argumentos, en orden: el número de ARLES, el destino, y los
    // dos valores que Meta exige y que vienen rellenos por defecto.
    expect(pedidas[0]).toEqual([EN_FRIO.id, '+528112345678', 'hello_world', 'en_US'])
  })

  /**
   * **Aceptado no es entregado** (§65).
   *
   * Es el mismo texto que el del correo y por la misma razón: ARLES no lee
   * teléfonos. Si esta prueba se pudiera cambiar a «entregado» sin que fallara,
   * la distinción no estaría sostenida por nada.
   */
  it('cuando Meta acepta, no dice «entregado»', async () => {
    const store = sembrar([{ ...EN_FRIO, puedeEnviar: true }])
    store.enviarPrueba = () =>
      Promise.resolve({ aceptado: true, clave: 'whatsapp.prueba.aceptado' })

    const pantalla = mount(PantallaWhatsApp, entorno)
    const campos = pantalla.findAll('input')
    await campos[campos.length - 3]?.setValue('+528112345678')
    const mandar = pantalla.findAll('button').find((b) => b.text().includes('Mandar la prueba'))
    await mandar?.trigger('click')
    await pantalla.vm.$nextTick()

    const texto = pantalla.text()
    expect(texto).toContain('Meta aceptó el mensaje')
    expect(texto).toContain('Mira tu teléfono')
    expect(texto.toLowerCase()).not.toContain('entregado')
  })

  /**
   * **Un rechazo de Meta se enseña con su texto, no como clave ni como error.**
   *
   * Las seis claves de `whatsapp.prueba.*` no tenían texto en el catálogo —el
   * comando llevaba tres entregas devolviéndolas sin que nadie las pintara—, así
   * que esta prueba comprueba lo que se habría visto: la clave cruda.
   */
  it('un rechazo de Meta sale con su explicación en español', async () => {
    const store = sembrar([{ ...EN_FRIO, puedeEnviar: true }])
    store.enviarPrueba = () =>
      Promise.resolve({ aceptado: false, clave: 'whatsapp.prueba.tokenNoVale' })

    const pantalla = mount(PantallaWhatsApp, entorno)
    const campos = pantalla.findAll('input')
    await campos[campos.length - 3]?.setValue('+528112345678')
    const mandar = pantalla.findAll('button').find((b) => b.text().includes('Mandar la prueba'))
    await mandar?.trigger('click')
    await pantalla.vm.$nextTick()

    const texto = pantalla.text()
    expect(texto).toContain('La prueba no salió')
    expect(texto).toContain('Meta rechazó el token')
    expect(texto).not.toContain('whatsapp.prueba')
  })
})
