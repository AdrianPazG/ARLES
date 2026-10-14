/**
 * El dominio del correo decide servidor, puerto y cifrado.
 *
 * ── POR QUÉ (R-01 y H-03, 14/10/2026) ───────────────────────────────────────
 *
 * Dirección no pudo dar de alta su cuenta. El formulario pedía **siete campos**
 * —tres técnicos— y aceptó valores que no podían funcionar sin decir nada. Su
 * requisito fue: «que sea correo y contraseña nomás. Comprueba que el servidor,
 * puerto y cifrado sean siempre correctos».
 *
 * Esto comprueba lo segundo: que los valores que ARLES pone solo son los que
 * cada proveedor publica, y sobre todo **que el par puerto+cifrado nunca se
 * cruza**. 587 con TLS directo, o 465 con STARTTLS, deja la conexión esperando
 * a un servidor que no va a contestar — y ése es exactamente el error que vio.
 */
import { describe, expect, it } from 'vitest'

import { PROVEEDORES, proveedorDe } from '@/app/stores/remitentes'

describe('el proveedor se deduce del dominio', () => {
  it('Gmail, con sus dos dominios', () => {
    for (const d of ['pruebas.telemetry@gmail.com', 'alguien@googlemail.com']) {
      const p = proveedorDe(d)
      expect(p?.nombre, d).toBe('Gmail')
      expect(p?.servidor).toBe('smtp.gmail.com')
      expect(p?.puerto).toBe('587')
      expect(p?.cifrado).toBe('starttls')
    }
  })

  it('Outlook, en sus cuatro dominios', () => {
    for (const d of ['a@outlook.com', 'a@hotmail.com', 'a@live.com', 'a@msn.com']) {
      expect(proveedorDe(d)?.nombre, d).toBe('Outlook')
    }
  })

  it('no distingue mayúsculas: nadie escribe su correo siempre igual', () => {
    expect(proveedorDe('Alguien@GMAIL.com')?.nombre).toBe('Gmail')
  })

  it('tolera espacios al pegar desde otro sitio', () => {
    expect(proveedorDe('  alguien@gmail.com  ')?.nombre).toBe('Gmail')
  })

  /**
   * Un dominio propio devuelve `null` **a propósito**: ARLES no se inventa un
   * servidor que no sabe. En la pantalla eso abre la configuración avanzada,
   * que es lo correcto.
   */
  it('un dominio propio no se adivina', () => {
    expect(proveedorDe('contacto@telemetrymx.com')).toBeNull()
    expect(proveedorDe('sin-arroba')).toBeNull()
    expect(proveedorDe('')).toBeNull()
  })

  /**
   * ── LA GARANTÍA QUE PIDIÓ DIRECCIÓN ──
   *
   * «Comprueba que el servidor, puerto y cifrado sean siempre correctos.»
   *
   * El par puerto+cifrado sólo tiene dos combinaciones válidas en SMTP, y
   * cruzarlas es el error que no da un mensaje útil: la conexión se queda
   * esperando. Esto lo fija para todos los proveedores, los de hoy y los que
   * se añadan.
   */
  it('el puerto y el cifrado nunca se cruzan', () => {
    for (const [, p] of PROVEEDORES) {
      if (p.puerto === '587') {
        expect(p.cifrado, `${p.nombre}: 587 va con STARTTLS`).toBe('starttls')
      } else if (p.puerto === '465') {
        expect(p.cifrado, `${p.nombre}: 465 va con TLS directo`).toBe('directo')
      } else {
        throw new Error(`${p.nombre} usa el puerto ${p.puerto}, que no es 587 ni 465`)
      }
    }
  })

  it('ningún servidor se deja vacío ni sin el punto que el núcleo exige', () => {
    for (const [, p] of PROVEEDORES) {
      expect(p.servidor, p.nombre).toMatch(/^[a-z0-9.-]+\.[a-z]{2,}$/i)
    }
  })
})
