/**
 * Rellenos de jsdom para los tests de componentes.
 *
 * jsdom implementa el elemento `<dialog>` pero **no sus métodos**:
 * `showModal()` y `close()` no existen. Sin esto, cualquier test que monte
 * AModal falla con «el.showModal is not a function» — y el reflejo fácil sería
 * dejar de usar `<dialog>` nativo, que es precisamente lo que aporta el
 * atrapado de foco y la capa superior sin reimplementar nada.
 *
 * Lo que se rellena es el mínimo: abrir, cerrar y el atributo `open`. La
 * modalidad real —el fondo inerte, el atrapado de foco— **no se simula**, y
 * por eso ningún test de este repositorio afirma comprobarla: eso se verifica
 * con el catálogo abierto en un navegador de verdad.
 */
const dialogo = globalThis.HTMLDialogElement?.prototype

if (dialogo && !dialogo.showModal) {
  dialogo.showModal = function abrirModal(this: HTMLDialogElement) {
    this.open = true
  }
  dialogo.show = function abrir(this: HTMLDialogElement) {
    this.open = true
  }
  dialogo.close = function cerrar(this: HTMLDialogElement) {
    this.open = false
    this.dispatchEvent(new Event('close'))
  }
}
