/**
 * El token de Meta, revisado antes de guardar.
 *
 * ── POR QUÉ QUEDA UN SOLO CAMPO QUE REVISAR (R-02 → D-WA) ───────────────────
 *
 * El 14/10/2026 Dirección pidió que dar de alta un número de WhatsApp fuera tan
 * sencillo como el alta de correo. La respuesta de entonces fue que **no se
 * podía**, porque el WABA ID y el Phone Number ID sólo existen en el panel de
 * Meta y no hay nada de lo que inferirlos. Era verdad de *inferirlos* y falsa
 * de *preguntarlos*: desde D-WA (26/10/2026) ARLES se los pregunta a Meta con
 * el token, y el alta son **un dato y una elección**.
 *
 * Así que las pruebas de aquellos dos campos **se fueron con los campos**. Una
 * prueba de una rama que la pantalla ya no recorre no vigila nada: da verde
 * sobre código que nadie ejecuta, y eso es peor que no tenerla, porque se cree.
 *
 * Lo que queda es cazar el error de pegado del token **antes** de guardar, que
 * es donde se pierde la gente.
 *
 * **Avisa, no bloquea**: Meta podría cambiar el formato, y un formulario que se
 * niega a guardar algo correcto es peor que uno que señala algo dudoso.
 */
import { describe, expect, it } from 'vitest'

import { revisarDatoDeMeta } from '@/app/stores/whatsapp'

describe('el token de Meta se revisa antes de guardar', () => {
  it('un token correcto no se avisa', () => {
    expect(revisarDatoDeMeta('token', 'EAAJZCqZA' + 'x'.repeat(150))).toBe('')
  })

  it('un campo vacío no se avisa: todavía no se ha escrito nada', () => {
    expect(revisarDatoDeMeta('token', '')).toBe('')
    expect(revisarDatoDeMeta('token', '   ')).toBe('')
  })

  /** El error más común ahora que sólo hay un campo: pegar un identificador. */
  it('un identificador pegado en el token se caza', () => {
    expect(revisarDatoDeMeta('token', '102290129340398')).toBe(
      'numeroDeWhatsapp.avisos.tokenEsUnNumero',
    )
  })

  it('un token que no empieza por EAA se caza', () => {
    expect(revisarDatoDeMeta('token', 'abc123def456')).toBe(
      'numeroDeWhatsapp.avisos.tokenNoEmpiezaBien',
    )
  })

  /** Pegar con espacios alrededor es lo normal, y no debe avisar por eso. */
  it('los espacios de alrededor no cuentan', () => {
    expect(revisarDatoDeMeta('token', `  EAAJZCqZA${'x'.repeat(150)}  `)).toBe('')
  })
})
