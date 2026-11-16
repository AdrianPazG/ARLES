/**
 * Un aviso de error no puede sobrevivir a la recarga que lo resuelve.
 *
 * ── EL FALLO QUE ESTA PRUEBA REPRODUCE ──────────────────────────────────────
 *
 * Encontrado por Dirección el 14/10/2026, usando ARLES (hallazgo H-01):
 *
 *   1. Entra en Remitentes **antes** de configurar la empresa. `cargar()` falla
 *      con `EmpresaNoConfigurada` y pinta el aviso. Correcto.
 *   2. Va a Configuración y guarda los datos de la empresa. Correcto.
 *   3. Vuelve a Remitentes. La carga **ahora sí funciona**… y el aviso viejo
 *      **sigue pintado**: «Todavía no has configurado los datos de tu empresa».
 *
 * La pantalla afirmaba algo **falso** sobre el estado de su propia base, y le
 * hizo dudar de si el guardado había funcionado. Es el §95 al revés: un error
 * que ya no existe sigue dando instrucciones.
 *
 * La causa es de una línea: `cargar()` escribe el aviso en su `catch` y **nunca
 * lo limpia al empezar**. No era de Remitentes: lo tenían tres de los diez
 * stores —remitentes, empresa y whatsapp—.
 *
 * ── LOS DOS QUE NO ESTÁN AQUÍ, Y POR QUÉ ────────────────────────────────────
 *
 * Un detector por heurística marcó también `interfaz` y `recuperacion`. **Son
 * falsos positivos** y se descartaron leyendo el código: sus `catch` no
 * escriben ningún aviso a propósito —una preferencia de interfaz ilegible no
 * impide usar ARLES, y la pantalla de recuperación sigue siendo útil sin los
 * metadatos—. Meterlos aquí habría mandado a arreglar algo que funciona.
 *
 * ── CÓMO SE VIO EN ROJO ─────────────────────────────────────────────────────
 *
 * Con el código anterior al arreglo, los tres casos fallaban: `errorGeneral`
 * seguía con su valor después de una recarga limpia.
 */
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { useEmpresaStore } from '@/app/stores/empresa'
import { useRemitentesStore } from '@/app/stores/remitentes'
import { useWhatsappStore } from '@/app/stores/whatsapp'

/**
 * Sin núcleo —que es lo que hay en las pruebas— `cargar()` no llega a invocar
 * nada, así que la recarga es «limpia» por definición: si el aviso sigue ahí
 * después, es porque nadie lo borró.
 */
describe('un aviso de error no sobrevive a la recarga que lo resuelve', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('remitentes · el caso que encontró Dirección', async () => {
    const s = useRemitentesStore()
    s.errorGeneral = 'error.app.empresa_no_configurada'
    await s.cargar()
    expect(s.errorGeneral).toBeNull()
  })

  it('empresa', async () => {
    const s = useEmpresaStore()
    s.errorGeneral = 'error.app.empresa_no_configurada'
    await s.cargar()
    expect(s.errorGeneral).toBeNull()
  })

  it('whatsapp', async () => {
    const s = useWhatsappStore()
    s.error = 'error.app.empresa_no_configurada'
    await s.cargar()
    expect(s.error).toBeNull()
  })
})
