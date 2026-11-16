import { describe, expect, it } from 'vitest'

import { haceCuanto, numeroLegible } from './formato'

describe('un número guardado se enseña para poder leerlo', () => {
  it('un número de México se parte como se escribe en México', () => {
    expect(numeroLegible('+528112345678')).toBe('+52 81 1234 5678')
    expect(numeroLegible('+525500000000')).toBe('+52 55 0000 0000')
  })

  /**
   * La regla genérica agrupa **desde la derecha**, así que el último bloque
   * siempre está completo. Agrupando desde la izquierda, un número de once
   * dígitos acabaría en un bloque de uno — `+1 2345 6789 0`—, que se lee peor
   * que no agrupar.
   */
  it('un número de otro país se agrupa en bloques de cuatro desde la derecha', () => {
    // Las dos cifras están calculadas a mano, no pedidas a la función.
    //
    // La primera versión de esta prueba comparaba `numeroLegible(x)` con
    // `numeroLegible(x)` envuelto en un `replace`, así que **no podía fallar**:
    // afirmaba que la función es igual a sí misma. Es el defecto que este
    // proyecto lleva persiguiendo desde la Fase 0, cometido al escribir la
    // prueba de un arreglo. Cazado antes de correrla.
    expect(numeroLegible('+14155552671')).toBe('+141 5555 2671')
    expect(numeroLegible('+34911223344')).toBe('+349 1122 3344')
  })

  /**
   * **Lo que no se reconoce se devuelve intacto**, y esto no es cortesía: la
   * lista de bajas pinta con esta misma función correos y números, porque una
   * baja puede ser de cualquiera de los dos canales. Inventarle bloques a un
   * correo lo dejaría ilegible.
   */
  it('lo que no es un número guardado sale como entró', () => {
    for (const v of ['ana@ejemplo.mx', '', '81 1234 5678', '+52', 'sin número']) {
      expect(numeroLegible(v)).toBe(v.trim())
    }
  })

  it('no toca un número que ya viene con espacios', () => {
    expect(numeroLegible('+52 81 1111 1111')).toBe('+52 81 1111 1111')
  })
})

describe('haceCuanto', () => {
  /** Un instante fijo, para que las cifras no dependan de cuándo se corra. */
  const AHORA = Date.parse('2026-11-11T12:00:00Z')
  const haceSegundos = (s: number) => new Date(AHORA - s * 1000).toISOString()

  /**
   * **La cifra siempre está**, que es el §94 y la razón de `numeric: 'always'`.
   *
   * Con `numeric: 'auto'`, `Intl` devolvería «hace un momento» y «ayer» en vez
   * de «hace 1 minuto» y «hace 1 día». Eso se lee igual con un intervalo de
   * quince minutos que con uno de seis horas, y el sitio donde esto se usa
   * —cuándo se revisó un buzón— existe precisamente para saber si el
   * automatismo va.
   *
   * Verificada rompiéndola: con `numeric: 'auto'`, los casos de 1 minuto y 1
   * día fallan y los demás siguen pasando — que es el par que distingue las
   * dos opciones.
   */
  it('da una cifra, nunca un adjetivo', () => {
    expect(haceCuanto(haceSegundos(60), AHORA)).toBe('hace 1 minuto')
    expect(haceCuanto(haceSegundos(4 * 60), AHORA)).toBe('hace 4 minutos')
    expect(haceCuanto(haceSegundos(3600), AHORA)).toBe('hace 1 hora')
    expect(haceCuanto(haceSegundos(2 * 3600), AHORA)).toBe('hace 2 horas')
    expect(haceCuanto(haceSegundos(86_400), AHORA)).toBe('hace 1 día')
    expect(haceCuanto(haceSegundos(3 * 86_400), AHORA)).toBe('hace 3 días')
  })

  /**
   * **El suelo es un minuto.** Sin él, un intento de hace diez segundos diría
   * «hace 10 segundos» y el número cambiaría al recargar la pantalla sin que
   * nada hubiera pasado. Lo útil es el orden de magnitud.
   */
  it('nada baja de un minuto', () => {
    expect(haceCuanto(haceSegundos(0), AHORA)).toBe('hace 1 minuto')
    expect(haceCuanto(haceSegundos(10), AHORA)).toBe('hace 1 minuto')
    expect(haceCuanto(haceSegundos(59), AHORA)).toBe('hace 1 minuto')
  })

  /**
   * **Lo que no se puede leer sale vacío**, y no con un texto de relleno.
   *
   * La pantalla pone esto dentro de una frase, así que una cadena vacía deja
   * «Última vez, .» —feo— y un «desconocido» inventado sería un dato falso.
   * Que la fecha no se entienda sólo puede pasar con una base tocada a mano, y
   * ahí lo correcto es no afirmar nada.
   */
  it('una fecha que no se entiende no inventa una cifra', () => {
    expect(haceCuanto(null, AHORA)).toBe('')
    expect(haceCuanto('', AHORA)).toBe('')
    expect(haceCuanto('el martes por la tarde', AHORA)).toBe('')
  })

  /**
   * Una fecha en el futuro —el reloj del equipo se movió— no dice «dentro de».
   *
   * Es raro, y lo que importa es que no mande a hacer nada: el núcleo ya trata
   * ese caso como «recién revisado» y no dispara ninguna revisión
   * (`toca_revisar`).
   */
  it('un reloj movido no dice «dentro de»', () => {
    expect(haceCuanto(haceSegundos(-3 * 3600), AHORA)).toBe('hace 1 minuto')
  })
})
