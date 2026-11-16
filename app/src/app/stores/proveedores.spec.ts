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

import { PROVEEDORES, contrasenaParaEnviar, proveedorDe } from '@/app/stores/remitentes'

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

/**
 * ── LA CONTRASEÑA QUE GOOGLE ENSEÑA NO ES LA QUE EL SERVIDOR QUIERE ────────
 *
 * Google presenta la contraseña de aplicación en cuatro grupos de cuatro
 * —`abcd efgh ijkl mnop`— sólo para que se pueda leer. Quien la copia, la
 * copia entera, y hasta el 15/10/2026 ARLES la mandaba con los espacios
 * dentro.
 *
 * **Y nada avisaba**, que es lo que lo hacía grave: el aviso del largo ya
 * descontaba los espacios para contar, así que veía dieciséis y se callaba.
 * La cuenta fallaba después con un «no se pudo autenticar» que parece culpa
 * de quien la escribió.
 *
 * Lo encontró una pregunta de Dirección —«¿de dónde consigo la contraseña de
 * 16 caracteres?»— y no una prueba, que es la razón de que estas cinco
 * existan ahora.
 */
describe('la contraseña que sale hacia el servidor', () => {
  const gmail = proveedorDe('alguien@gmail.com')
  const outlook = proveedorDe('alguien@outlook.com')

  it('le quita a Gmail los espacios con que Google la enseña', () => {
    expect(contrasenaParaEnviar(gmail, 'abcd efgh ijkl mnop')).toBe('abcdefghijklmnop')
  })

  it('deja intacta una de Gmail ya escrita sin espacios', () => {
    expect(contrasenaParaEnviar(gmail, 'abcdefghijklmnop')).toBe('abcdefghijklmnop')
  })

  it('NO toca una contraseña que no da el largo al quitar los espacios', () => {
    // `mi clave secreta` son 14 letras sin espacios, no 16: no es una
    // contraseña de aplicación mal pegada, es una contraseña con espacios.
    // Limpiarla sería mandar otra distinta y hacer fallar un alta correcta.
    expect(contrasenaParaEnviar(gmail, 'mi clave secreta')).toBe('mi clave secreta')
  })

  it('NO toca nada de un proveedor sin largo declarado', () => {
    // Outlook, Yahoo, Zoho e iCloud no declaran largo. Ahí una contraseña con
    // espacios es exactamente eso, y se manda tal cual.
    expect(contrasenaParaEnviar(outlook, 'con espacios dentro')).toBe('con espacios dentro')
  })

  it('NO toca nada cuando el dominio no se reconoce', () => {
    expect(contrasenaParaEnviar(null, 'a b c d')).toBe('a b c d')
  })
})

/**
 * ── DÓNDE SE CREA LA CONTRASEÑA ES UN DATO DEL CATÁLOGO ───────────────────
 *
 * Dirección preguntó «¿de dónde consigo la contraseña de 16 caracteres?» con
 * la guía de Gmail **ya en pantalla**. Ése es el síntoma que importa: la guía
 * estaba y no contestaba, porque su paso 2 decía «busca Contraseñas de
 * aplicaciones» — un sitio donde buscar, no una instrucción que se pueda
 * seguir.
 *
 * La dirección vive en `PROVEEDORES` y no en un `if` sobre Gmail dentro de la
 * pantalla: el día que se sepa la de Outlook se escribe aquí y la pantalla no
 * se toca. Un `if` allí sería el mismo dato en dos sitios, que es como
 * empiezan todos los que se separan.
 */
describe('dónde se crea la contraseña de cada proveedor', () => {
  it('Gmail dice exactamente dónde, y es una dirección completa', () => {
    const gmail = proveedorDe('alguien@gmail.com')
    expect(gmail?.dondeSeCreaLaContrasena).toBe('https://myaccount.google.com/apppasswords')
  })

  it('ningún proveedor apunta a una dirección a medias', () => {
    // Una dirección sin esquema no se puede pegar en un navegador, y una que
    // no es de su propio proveedor manda al sitio de otro.
    for (const [, p] of PROVEEDORES) {
      if (p.dondeSeCreaLaContrasena === null) continue
      expect(p.dondeSeCreaLaContrasena, `${p.nombre}`).toMatch(/^https:\/\/[a-z0-9.-]+\//)
    }
  })

  it('los que no se saben valen null, y NO una dirección inventada', () => {
    // Es la decisión que sostiene lo anterior: una dirección que lleva a
    // ninguna parte es peor que no dar ninguna, porque se sigue. Hoy sólo
    // está comprobada la de Google.
    const sinDireccion = PROVEEDORES
      .filter(([, p]) => p.dondeSeCreaLaContrasena === null)
      .map(([, p]) => p.nombre)
    expect(sinDireccion).toEqual(['Outlook', 'Yahoo', 'Zoho', 'iCloud'])
  })

  it('quien declara un largo de contraseña dice también dónde se crea', () => {
    // Si ARLES sabe lo bastante de un proveedor como para avisar del largo de
    // su contraseña de aplicación, tiene que saber decir dónde se crea. Avisar
    // de que algo está mal sin decir cómo arreglarlo es media instrucción
    // (§95).
    for (const [, p] of PROVEEDORES) {
      if (p.largoDeContrasena === null) continue
      expect(p.dondeSeCreaLaContrasena, `${p.nombre} avisa del largo`).not.toBeNull()
    }
  })
})
