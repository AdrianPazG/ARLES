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
