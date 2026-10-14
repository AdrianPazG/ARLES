/**
 * Los tres datos que se copian del panel de Meta, revisados antes de guardar.
 *
 * ── POR QUÉ ESTO Y NO «DOS CAMPOS COMO EL CORREO» (R-02, 14/10/2026) ────────
 *
 * Dirección pidió que dar de alta un número de WhatsApp fuera tan sencillo como
 * el alta de correo. **No se puede, y conviene que quede escrito por qué**: el
 * correo se deduce del dominio —`@gmail.com` implica `smtp.gmail.com:587`— pero
 * el WABA ID, el Phone Number ID y el token **sólo existen en el panel de
 * Meta**. No hay nada de lo que inferirlos.
 *
 * Lo que sí se puede es cazar el error de pegado **antes** de guardar, que es
 * donde se pierde la gente: copiar un identificador en el campo del otro, o
 * pegar sólo un trozo, da un error de Meta que no dice cuál de los tres falla.
 *
 * **Avisa, no bloquea**: Meta podría cambiar el formato, y un formulario que se
 * niega a guardar algo correcto es peor que uno que señala algo dudoso.
 */
import { describe, expect, it } from 'vitest'

import { revisarDatoDeMeta } from '@/app/stores/whatsapp'

describe('los datos copiados de Meta se revisan antes de guardar', () => {
  it('un identificador correcto no se avisa', () => {
    expect(revisarDatoDeMeta('wabaId', '102290129340398')).toBe('')
    expect(revisarDatoDeMeta('phoneNumberId', '106540352242922')).toBe('')
  })

  it('un token correcto no se avisa', () => {
    expect(revisarDatoDeMeta('token', 'EAAJZCqZA' + 'x'.repeat(150))).toBe('')
  })

  it('un campo vacío no se avisa: todavía no se ha escrito nada', () => {
    expect(revisarDatoDeMeta('wabaId', '')).toBe('')
    expect(revisarDatoDeMeta('token', '   ')).toBe('')
  })

  /** El error más común: pegar el token donde va el identificador. */
  it('el token pegado en un identificador se caza', () => {
    expect(revisarDatoDeMeta('wabaId', 'EAAJZCqZAx123')).toBe(
      'numeroDeWhatsapp.avisos.esElToken',
    )
  })

  /** Y el contrario. */
  it('un identificador pegado en el token se caza', () => {
    expect(revisarDatoDeMeta('token', '102290129340398')).toBe(
      'numeroDeWhatsapp.avisos.tokenEsUnNumero',
    )
  })

  it('un identificador con espacios o guiones se caza', () => {
    expect(revisarDatoDeMeta('wabaId', '102 290 129')).toBe(
      'numeroDeWhatsapp.avisos.soloDigitos',
    )
    expect(revisarDatoDeMeta('phoneNumberId', '106-540-352')).toBe(
      'numeroDeWhatsapp.avisos.soloDigitos',
    )
  })

  it('un identificador al que le falta un trozo se caza', () => {
    expect(revisarDatoDeMeta('wabaId', '10229')).toBe(
      'numeroDeWhatsapp.avisos.demasiadoCorto',
    )
  })

  it('un token que no empieza por EAA se caza', () => {
    expect(revisarDatoDeMeta('token', 'abc123def456')).toBe(
      'numeroDeWhatsapp.avisos.tokenNoEmpiezaBien',
    )
  })

  /** Pegar con espacios alrededor es lo normal, y no debe avisar por eso. */
  it('los espacios de alrededor no cuentan', () => {
    expect(revisarDatoDeMeta('wabaId', '  102290129340398  ')).toBe('')
  })
})
