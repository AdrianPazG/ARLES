import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { useEmpresaStore } from '@/app/stores/empresa'

const BORRADOR = {
  nombreComercial: 'TELEMETRY INSIGHT',
  pais: 'MX',
  zonaHoraria: 'America/Mexico_City',
  correoCorporativo: 'hola@telemetry.mx',
  sitioWeb: '',
}

describe('configuración de empresa sin núcleo', () => {
  beforeEach(() => setActivePinia(createPinia()))

  /**
   * En `vite dev` y en las sondas no hay comandos de Tauri. Fingir un guardado
   * ahí enseñaría una pantalla que miente sobre lo que acaba de pasar, y esa
   * mentira se descubre en producción.
   */
  it('guardar dice que no hay núcleo en vez de fingir que guardó', async () => {
    const e = useEmpresaStore()
    const guardado = await e.guardar(BORRADOR)
    expect(guardado).toBe(false)
    expect(e.errorGeneral).toBe('error.app.sin_nucleo')
    expect(e.guardadoConExito).toBe(false)
  })

  it('sin núcleo se enseña la reserva y no hay empresa configurada', async () => {
    const e = useEmpresaStore()
    await e.cargar()
    expect(e.configurada).toBe(false)
    expect(e.onboarding.total).toBe(6)
    expect(e.onboarding.completados).toBe(0)
  })

  /**
   * La lista enseña los seis pasos desde el primer día, y **los seis llevan a
   * algún sitio**.
   *
   * ── LO QUE COMPROBABA ANTES, Y POR QUÉ NO SERVÍA ─────────────────────────
   *
   * Comprobaba dos cosas que se fueron el 28/10/2026 con el arreglo de D-01:
   * que cada paso declarase su `entrega` del roadmap, y que `disponible` y
   * `ruta` no divergieran. La segunda era la misma afirmación dicha dos veces
   * —`disponible` se calculaba como `ruta !== null`—, así que no podía fallar
   * nunca; y mientras la vigilaba, **tres de las seis filas de la reserva
   * decían `ruta: null`** con dos de esas pantallas construidas desde el 29/09.
   *
   * La comprobación cubría la coherencia entre dos campos y no la única
   * pregunta que importa: ¿lleva este paso a alguna parte? Ahora pregunta eso.
   */
  it('los seis pasos de la reserva llevan a una pantalla', async () => {
    const e = useEmpresaStore()
    await e.cargar()
    expect(e.onboarding.pasos).toHaveLength(6)
    for (const paso of e.onboarding.pasos) {
      expect(paso.ruta, `${paso.clave} no lleva a ninguna parte`).not.toBeNull()
      expect(paso.ruta?.startsWith('/'), `${paso.clave}: «${paso.ruta}» no es una ruta`).toBe(
        true,
      )
    }
  })

  it('el primer paso es la empresa y lleva a la configuración', async () => {
    const e = useEmpresaStore()
    await e.cargar()
    const primero = e.onboarding.pasos[0]
    expect(primero?.clave).toBe('empresa')
    expect(primero?.ruta).toBe('/inicio/configuracion')
  })
})
