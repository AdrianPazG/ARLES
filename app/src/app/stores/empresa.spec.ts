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
   * La lista enseña los seis pasos desde el primer día, y los que aún no
   * existen lo dicen con su entrega. Si sólo se enseñaran los construidos,
   * alguien vería la lista completa al terminar el primero y concluiría que ya
   * puede enviar.
   */
  it('los pasos que aún no existen lo declaran y no llevan a ninguna parte', async () => {
    const e = useEmpresaStore()
    await e.cargar()
    for (const paso of e.onboarding.pasos) {
      expect(paso.entrega.length, `${paso.clave} no dice su entrega`).toBeGreaterThan(0)
      expect(
        paso.disponible,
        `${paso.clave}: «disponible» y «ruta» no pueden divergir`,
      ).toBe(paso.ruta !== null)
    }
  })

  it('el primer paso es la empresa y lleva a Ajustes', async () => {
    const e = useEmpresaStore()
    await e.cargar()
    const primero = e.onboarding.pasos[0]
    expect(primero?.clave).toBe('empresa')
    expect(primero?.ruta).toBe('/ajustes')
  })
})
