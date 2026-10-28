import { describe, expect, it } from 'vitest'

import { numeroLegible } from './formato'

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
