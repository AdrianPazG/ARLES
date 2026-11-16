/**
 * Despacha una tecla contra un mapa de acciones.
 *
 * Existe por un motivo concreto: `mapa[evento.key]` sobre un objeto literal
 * también encuentra lo que hereda de `Object.prototype`. Una tecla llamada
 * «constructor», «toString» o «valueOf» devuelve una función perfectamente
 * invocable, así que el despachador la da por buena, llama a
 * `preventDefault()` y **se traga la tecla** sin hacer nada.
 *
 * Ningún teclado produce hoy esos valores de `KeyboardEvent.key`, así que no
 * es explotable — pero es una búsqueda que devuelve un resultado que nadie
 * puso ahí, y ese patrón no se deja escrito en un componente que procesa
 * entrada del usuario.
 *
 * `Object.hasOwn` mira sólo las claves propias. `preventDefault()` se llama
 * únicamente cuando hay una acción de verdad.
 */
export function despachar(
  mapa: Record<string, () => void>,
  evento: KeyboardEvent,
): boolean {
  if (!Object.hasOwn(mapa, evento.key)) return false
  evento.preventDefault()
  mapa[evento.key]!()
  return true
}
