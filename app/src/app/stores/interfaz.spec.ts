import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { UMBRAL_DE_PLEGADO_PX, useInterfazStore } from '@/app/stores/interfaz'

/**
 * P-11 escrito como pruebas.
 *
 * Las tres decisiones de Dirección son reglas de comportamiento, y una regla de
 * comportamiento que sólo vive en un comentario se pierde en la primera
 * refactorización de la barra lateral.
 */
describe('la barra lateral se pliega a mano y sola (P-11)', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('con la ventana ancha, empieza desplegada', () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX + 200)
    expect(i.plegada).toBe(false)
    expect(i.alternableAhora).toBe(true)
  })

  it('a mano: alternar la pliega y la vuelve a abrir', async () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX + 200)
    await i.alternar()
    expect(i.plegada).toBe(true)
    await i.alternar()
    expect(i.plegada).toBe(false)
  })

  it('sola: por debajo del umbral se pliega sin que nadie lo pida', () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX - 1)
    expect(i.plegada).toBe(true)
    expect(i.preferenciaPlegada).toBe(false)
  })

  it('justo en el umbral todavía no se pliega', () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX)
    expect(i.plegada).toBe(false)
  })

  /**
   * La regla que más fácil sería romper al «simplificar»: si el plegado
   * automático escribiera la preferencia, agrandar la ventana devolvería la
   * barra plegada — el usuario habría perdido su elección sin tocar nada.
   */
  it('el plegado automático no pisa lo que eligió el usuario', () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX + 200)
    expect(i.plegada).toBe(false)

    i.anotarAncho(UMBRAL_DE_PLEGADO_PX - 100)
    expect(i.plegada).toBe(true)
    expect(i.preferenciaPlegada).toBe(false)

    i.anotarAncho(UMBRAL_DE_PLEGADO_PX + 200)
    expect(i.plegada, 'al ensanchar debe volver a como estaba').toBe(false)
  })

  it('una preferencia de plegada sobrevive a ensanchar la ventana', async () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX + 200)
    await i.alternar()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX - 100)
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX + 400)
    expect(i.plegada, 'la eligió plegada; sigue plegada').toBe(true)
  })

  /**
   * Con la ventana estrecha el botón queda deshabilitado. Si además alternara,
   * el usuario cambiaría una preferencia invisible cuyo efecto aparecería al
   * ensanchar, sin relación aparente con lo que hizo.
   */
  it('con la ventana estrecha, alternar no cambia nada', async () => {
    const i = useInterfazStore()
    i.anotarAncho(UMBRAL_DE_PLEGADO_PX - 100)
    expect(i.alternableAhora).toBe(false)
    await i.alternar()
    expect(i.preferenciaPlegada).toBe(false)
    expect(i.plegada).toBe(true)
  })

  /** Sin núcleo —`vite dev`, las sondas— no revienta: se queda desplegada. */
  it('sin núcleo, cargar no falla', async () => {
    const i = useInterfazStore()
    await expect(i.cargar()).resolves.toBeUndefined()
    expect(i.plegada).toBe(false)
  })
})

/**
 * C-1 y C-2 escritos como pruebas.
 *
 * Lo que se comprueba aquí es la **resolución**: qué tema corresponde dada una
 * elección y un sistema operativo. Que el atributo llegue al `<html>` y que la
 * elección sobreviva al cierre son afirmaciones sobre el navegador y sobre la
 * base, y se miden en `sonda:tema`.
 */
describe('el tema se elige, o lo pone el sistema (C-1, C-2)', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('sin elegir nada, sigue al sistema', async () => {
    const i = useInterfazStore()
    expect(i.tema).toBe('auto')

    i.anotarTemaDelSistema(true)
    expect(i.temaAplicado).toBe('claro')
    i.anotarTemaDelSistema(false)
    expect(i.temaAplicado).toBe('oscuro')
  })

  it('elegido a mano, el sistema deja de mandar', async () => {
    const i = useInterfazStore()
    await i.elegirTema('claro')
    expect(i.temaAplicado).toBe('claro')

    // El sistema se va a oscuro y la pantalla NO le hace caso: el usuario
    // eligió. Si esto se rompiera, su elección duraría hasta el anochecer.
    i.anotarTemaDelSistema(false)
    expect(i.temaAplicado).toBe('claro')
  })

  it('volver a «Automático» devuelve el mando al sistema', async () => {
    const i = useInterfazStore()
    await i.elegirTema('oscuro')
    i.anotarTemaDelSistema(true)
    expect(i.temaAplicado).toBe('oscuro')

    await i.elegirTema('auto')
    expect(i.temaAplicado).toBe('claro')
  })

  /**
   * `auto` nunca se escribe en `data-tema`: no es un tema, es una instrucción.
   * Si se escapara, los tokens no encontrarían ninguna regla que aplicar.
   */
  it('lo que se aplica siempre es oscuro o claro, nunca «auto»', async () => {
    const i = useInterfazStore()
    for (const elegido of ['auto', 'oscuro', 'claro'] as const) {
      await i.elegirTema(elegido)
      expect(['oscuro', 'claro']).toContain(i.temaAplicado)
    }
  })

  it('un tema que no existe se ignora', async () => {
    const i = useInterfazStore()
    await i.elegirTema('rosa' as never)
    expect(i.tema).toBe('auto')
  })

  /** Sin `matchMedia` —o sin haber preguntado aún— manda el dark-first (§18). */
  it('sin saber nada del sistema, el tema de la casa es el oscuro', () => {
    const i = useInterfazStore()
    expect(i.temaDelSistema).toBe('oscuro')
    expect(i.temaAplicado).toBe('oscuro')
  })
})
