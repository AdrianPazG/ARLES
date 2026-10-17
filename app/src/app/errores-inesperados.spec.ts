/**
 * Que un error que ARLES no esperaba diga su causa · H-05, 16/10/2026
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ EXISTE ESTE ARCHIVO
 *
 * Dirección no pudo dar de alta un contacto y ARLES contestó «Ocurrió un error
 * al acceder a los datos. Cierra ARLES y vuelve a abrirlo». **No se pudo
 * diagnosticar**, y la razón no fue que el fallo fuera raro: fue que el
 * mensaje se tragó la causa.
 *
 * Nueve stores escribían, para cualquier cosa no tipada:
 *
 *     errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'
 *
 * Eso hace dos cosas malas a la vez: **afirma una causa que no conoce** —«falló
 * la base»— y **pierde la de verdad**, así que nadie puede arreglarla después.
 * Y encima manda a cerrar y reabrir ARLES, que si la causa es otra no arregla
 * nada: es el mismo defecto que el fallo de migración tenía el 13/10.
 *
 * Estas pruebas fijan las dos mitades del arreglo: que la clave sea honesta, y
 * que el detalle llegue vivo hasta el texto.
 */
import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { resolverError } from '@/app/errores'
import { claveDelError, ultimoDetalleTecnico, olvidarDetalleTecnico } from '@/app/nucleo'

describe('un error que no viene tipado del núcleo', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    olvidarDetalleTecnico()
  })

  it('NO se declara error de base de datos', () => {
    // Es el defecto que encontró H-05: afirmar una causa que no se conoce.
    expect(claveDelError(new TypeError('x is not a function'))).toBe('error.inesperado')
    expect(claveDelError(new TypeError('x'))).not.toBe('error.db.sqlite')
  })

  it('guarda el detalle técnico de un Error de JavaScript', () => {
    claveDelError(new TypeError('no se puede leer «canales» de undefined'))
    expect(ultimoDetalleTecnico()).toContain('no se puede leer')
    expect(ultimoDetalleTecnico()).toContain('TypeError')
  })

  it('guarda también lo que no es un Error', () => {
    claveDelError('se cayó y punto')
    expect(ultimoDetalleTecnico()).toBe('se cayó y punto')

    claveDelError({ raro: true })
    expect(ultimoDetalleTecnico()).toContain('raro')
  })

  it('un error tipado del núcleo conserva su clave', () => {
    const clave = claveDelError({ clave: 'error.db.direccion_en_uso', detalle: 'ana@x.mx' })
    expect(clave).toBe('error.db.direccion_en_uso')
    expect(ultimoDetalleTecnico()).toBe('ana@x.mx')
  })

  it('el detalle llega hasta el texto que lee el usuario', () => {
    // Ésta es la que de verdad cierra H-05: las pantallas llaman con
    // `detalle: ''`, así que el dato se perdía justo antes de enseñarse.
    claveDelError(new RangeError('índice 7 fuera de 0..3'))
    const m = resolverError({ clave: 'error.inesperado', detalle: '' })
    expect(m.como).toContain('índice 7 fuera de 0..3')
  })

  it('y el texto dice las tres cosas del §95 sin culpar a la base', () => {
    claveDelError(new Error('cualquiera'))
    const m = resolverError({ clave: 'error.inesperado', detalle: '' })
    expect(m.que.length).toBeGreaterThan(0)
    expect(m.como.length).toBeGreaterThan(0)
    expect(m.salvo.length).toBeGreaterThan(0)
    expect(m.que.toLowerCase()).not.toContain('base de datos')
    // Y no manda a cerrar y reabrir, que es lo que no arregla nada.
    expect(m.como.toLowerCase()).not.toContain('cierra arles')
  })
})
