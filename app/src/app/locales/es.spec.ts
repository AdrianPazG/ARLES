import { describe, expect, it } from 'vitest'

import { i18n } from '@/app/i18n'
import { es } from '@/app/locales/es'

/**
 * Los principios del producto, escritos como tests.
 *
 * Un principio que solo vive en un documento se erosiona: alguien cambia un
 * texto un martes por la tarde con buena intención. Aquí falla la compilación.
 */

/** Estados de `arles_core::AttemptState`, en su forma persistida. */
const ESTADOS = [
  'queued',
  'claimed',
  'sending',
  'sent',
  'failed',
  'permanently_failed',
  'suppressed',
  'cancelled',
  'presumed_sent',
] as const

describe('honestidad de las métricas (§65)', () => {
  it('el estado «sent» se muestra como Aceptado, nunca como Entregado', () => {
    expect(es.intento.sent).toBe('Aceptado')
  })

  it('ningún texto de estado afirma una entrega', () => {
    const prohibidas = ['entregado', 'entregada', 'recibido', 'recibida']
    for (const [estado, etiqueta] of Object.entries(es.intento)) {
      const texto = etiqueta.toLowerCase()
      for (const palabra of prohibidas) {
        expect(
          texto.includes(palabra),
          `el estado «${estado}» dice «${etiqueta}»: ARLES no sabe si el ` +
            `mensaje se entregó y no debe afirmarlo`,
        ).toBe(false)
      }
    }
  })

  it('cada estado del motor tiene su texto', () => {
    for (const estado of ESTADOS) {
      expect(es.intento, `falta el texto del estado «${estado}»`).toHaveProperty(
        estado,
      )
    }
  })

  it('el estado ambiguo no se presenta como un envío confirmado', () => {
    // ADR-0004: presumed_sent significa «no sabemos». Decir «Enviado» aquí
    // sería exactamente la deshonestidad que el §65 prohíbe.
    expect(es.intento.presumed_sent).toBe('Envío no confirmado')
  })
})

describe('estructura de los errores (§95)', () => {
  type Error3 = { que: string; como: string; salvo: string }

  const esError = (v: unknown): v is Error3 =>
    typeof v === 'object' &&
    v !== null &&
    'que' in v &&
    'como' in v &&
    'salvo' in v

  const recorrer = (nodo: object, ruta: string): [string, Error3][] => {
    const encontrados: [string, Error3][] = []
    for (const [clave, valor] of Object.entries(nodo)) {
      const actual = ruta ? `${ruta}.${clave}` : clave
      if (esError(valor)) {
        encontrados.push([actual, valor])
      } else if (typeof valor === 'object' && valor !== null) {
        encontrados.push(...recorrer(valor, actual))
      }
    }
    return encontrados
  }

  const errores = recorrer(es.error, '')

  it('hay errores declarados', () => {
    expect(errores.length).toBeGreaterThan(0)
  })

  it('todos dicen qué pasó, cómo arreglarlo y qué está a salvo', () => {
    for (const [ruta, e] of errores) {
      expect(e.que.length, `${ruta}.que está vacío`).toBeGreaterThan(0)
      expect(e.como.length, `${ruta}.como está vacío`).toBeGreaterThan(0)
      // La tercera parte es la que falta en casi todo el software, y la que
      // importa cuando alguien acaba de ver fallar una campaña.
      expect(e.salvo.length, `${ruta}.salvo está vacío`).toBeGreaterThan(0)
    }
  })

  it('ninguno usa un tono infantil (§94)', () => {
    const prohibidas = ['ups', 'oops', 'vaya', 'woohoo', 'genial', '¡uy']
    for (const [ruta, e] of errores) {
      const texto = `${e.que} ${e.como} ${e.salvo}`.toLowerCase()
      for (const palabra of prohibidas) {
        expect(
          texto.includes(palabra),
          `${ruta} usa «${palabra}»: los errores de negocio se comunican en serio`,
        ).toBe(false)
      }
    }
  })
})

describe('nomenclatura del producto (ADR-0010)', () => {
  it('el nombre visible no lleva el numeral', () => {
    expect(es.producto.nombre).toBe('ARLES RELAY')
  })

  it('ningún texto pone el numeral junto al número de versión', () => {
    const todos = JSON.stringify(es)
    expect(todos).not.toMatch(/ARLES RELAY I\s*[/·-]?\s*v?1\.2\.0/)
  })
})

describe('declaración de lo que el producto no sabe', () => {
  it('advierte de que la detección de rebotes es parcial (ADR-0009)', () => {
    expect(es.honestidad.rebotesParciales).toContain('parcial')
    expect(es.honestidad.rebotesParcialesQueHacer.length).toBeGreaterThan(0)
  })

  it('advierte de que los respaldos no están cifrados (T-6)', () => {
    expect(es.honestidad.respaldosSinCifrar).toContain('no están cifrados')
  })
})

describe('todos los textos se pueden compilar', () => {
  /**
   * Existe por un fallo real, encontrado auditando la 3.1 con el navegador.
   *
   * `vue-i18n` **compila** cada mensaje, y en su gramática `@` empieza un
   * enlace a otra clave (`@:otra.clave`). El texto «Revisa que el correo tenga
   * la forma nombre@dominio.com» no compila: lanza `SyntaxError`, la función
   * de render revienta y **la pantalla entera deja de pintarse**.
   *
   * Lo peor era cómo fallaba: en silencio. El formulario se quedaba con lo
   * último que había pintado, sin marcar ningún campo, sin mensaje y sin nada
   * en la interfaz que dijera que algo había ido mal. El error sólo existía en
   * la consola del navegador, que en una ventana de Tauri no ve nadie.
   *
   * Ningún test lo veía porque los textos se leían como datos —comprobando que
   * estuvieran, no que se pudieran usar—. Este los pasa por `t()`, que es lo
   * que hace la interfaz.
   */
  const rutas: string[] = []
  const recorrer = (nodo: object, base: string): void => {
    for (const [clave, valor] of Object.entries(nodo)) {
      const ruta = base ? `${base}.${clave}` : clave
      if (typeof valor === 'string') rutas.push(ruta)
      else if (typeof valor === 'object' && valor !== null) recorrer(valor, ruta)
    }
  }
  recorrer(es, '')

  it('hay textos que comprobar', () => {
    expect(rutas.length).toBeGreaterThan(50)
  })

  it('ninguno lanza al compilarse', () => {
    const rotos: string[] = []
    for (const ruta of rutas) {
      try {
        // Los parámetros van con valores de relleno: una clave con `{producto}`
        // sin valor avisa por consola, pero no es lo que se comprueba aquí.
        i18n.global.t(ruta, {
          producto: 'x', hechos: 0, total: 0, entrega: 'x',
        })
      } catch (e) {
        rotos.push(`${ruta}: ${String(e).slice(0, 80)}`)
      }
    }
    expect(rotos, 'estos textos revientan la pantalla que los use').toEqual([])
  })
})
