/**
 * Primitivas del design system de ARLES (Fase 2).
 *
 * Punto de entrada único: una pantalla importa desde aquí y nunca desde el
 * archivo de un componente. Así hay un solo sitio donde ver qué existe, y
 * añadir una primitiva obliga a pasar por esta lista —que es lo que impide
 * que aparezca un décimo botón local en una pantalla cualquiera.
 *
 * Regla que sostiene todo esto: **ningún componente contiene un hex, un píxel
 * suelto ni una duración literal** (§17). Todo sale de los tokens. El
 * validador de fase lo comprueba.
 */
export { default as AAviso } from './AAviso.vue'
export { default as ABoton } from './ABoton.vue'
export { default as AEntrada } from './AEntrada.vue'
export { default as AIcono } from './AIcono.vue'
export { default as AInsignia } from './AInsignia.vue'
export { default as ALogotipo } from './ALogotipo.vue'
export { default as AMenu } from './AMenu.vue'
export { default as AModal } from './AModal.vue'
export { default as AModulo } from './AModulo.vue'
export { default as APestanas } from './APestanas.vue'
export { default as ASelector } from './ASelector.vue'
export { default as ATabla } from './ATabla.vue'

export { default as EstadoCargando } from './EstadoCargando.vue'
export { default as EstadoError } from './EstadoError.vue'
export { default as EstadoExito } from './EstadoExito.vue'
export { default as EstadoVacio } from './EstadoVacio.vue'

export type { NombreDeIcono } from './AIcono.vue'
export type { AccionDeMenu } from './AMenu.vue'
export type { Pestana } from './APestanas.vue'
export type { OpcionDeSelector } from './ASelector.vue'
export type { ColumnaDeTabla, Direccion } from './ATabla.vue'
