<script setup lang="ts">
/**
 * Menú de acciones.
 *
 * Patrón ARIA de menú de botón: `Flecha abajo` abre y entra, las flechas
 * recorren, `Esc` cierra **y devuelve el foco al disparador** — sin eso el
 * usuario de teclado queda al principio de la pantalla después de cerrar.
 *
 * El menú se cierra al elegir, al pulsar fuera y al salir el foco del grupo.
 * Los tres, no uno: cada camino de salida que falta deja un menú abierto
 * sobre una pantalla que el usuario ya no está mirando.
 */
import { nextTick, onBeforeUnmount, onMounted, ref, useId } from 'vue'

import AIcono, { type NombreDeIcono } from './AIcono.vue'

export interface AccionDeMenu {
  id: string
  texto: string
  icono?: NombreDeIcono
  /** Se pinta en rojo y se separa del resto: borrar, detener. */
  destructiva?: boolean
  deshabilitada?: boolean
}

const props = defineProps<{
  etiqueta: string
  acciones: readonly AccionDeMenu[]
}>()

const emit = defineEmits<{ elegir: [id: string] }>()

const abierto = ref(false)
const raiz = ref<HTMLElement | null>(null)
const disparador = ref<HTMLButtonElement | null>(null)
const opciones = ref<HTMLButtonElement[]>([])
const id = useId()

function utilizables() {
  return props.acciones
    .map((a, i) => ({ a, i }))
    .filter(({ a }) => !a.deshabilitada)
}

async function abrir(desdeElFinal = false) {
  abierto.value = true
  await nextTick()
  const lista = utilizables()
  const destino = desdeElFinal ? lista.at(-1) : lista[0]
  if (destino) opciones.value[destino.i]?.focus()
}

function cerrar(devolverFoco = true) {
  if (!abierto.value) return
  abierto.value = false
  if (devolverFoco) disparador.value?.focus()
}

function elegir(accion: AccionDeMenu) {
  if (accion.deshabilitada) return
  emit('elegir', accion.id)
  cerrar()
}

function moverFoco(delta: number) {
  const lista = utilizables()
  if (lista.length === 0) return
  const activo = document.activeElement
  const actual = lista.findIndex(({ i }) => opciones.value[i] === activo)
  const siguiente = lista[(actual + delta + lista.length) % lista.length]
  if (siguiente) opciones.value[siguiente.i]?.focus()
}

function teclaDelDisparador(evento: KeyboardEvent) {
  if (evento.key === 'ArrowDown' || evento.key === 'Enter' || evento.key === ' ') {
    evento.preventDefault()
    void abrir()
  } else if (evento.key === 'ArrowUp') {
    evento.preventDefault()
    void abrir(true)
  }
}

function enfocarExtremo(final: boolean) {
  const destino = final ? utilizables().at(-1) : utilizables()[0]
  if (destino) opciones.value[destino.i]?.focus()
}

function teclaDelMenu(evento: KeyboardEvent) {
  const acciones: Record<string, () => void> = {
    ArrowDown: () => moverFoco(1),
    ArrowUp: () => moverFoco(-1),
    Home: () => enfocarExtremo(false),
    End: () => enfocarExtremo(true),
    Escape: () => cerrar(),
    Tab: () => cerrar(false),
  }
  const accion = acciones[evento.key]
  if (!accion) return
  // Tab sí debe seguir moviendo el foco fuera: sólo se cierra el menú.
  if (evento.key !== 'Tab') evento.preventDefault()
  accion()
}

function alPulsarEnElDocumento(evento: MouseEvent) {
  if (!abierto.value) return
  const destino = evento.target
  if (destino instanceof Node && raiz.value?.contains(destino)) return
  // Sin devolver el foco: el usuario ya lo llevó a otro sitio con el ratón.
  cerrar(false)
}

onMounted(() => document.addEventListener('pointerdown', alPulsarEnElDocumento))
onBeforeUnmount(() =>
  document.removeEventListener('pointerdown', alPulsarEnElDocumento),
)
</script>

<template>
  <div
    ref="raiz"
    class="menu"
  >
    <button
      ref="disparador"
      class="disparador"
      type="button"
      :aria-haspopup="true"
      :aria-expanded="abierto"
      :aria-controls="abierto ? id : undefined"
      @click="abierto ? cerrar(false) : abrir()"
      @keydown="teclaDelDisparador"
    >
      <span>{{ etiqueta }}</span>
      <AIcono nombre="desplegar" />
    </button>

    <div
      v-if="abierto"
      :id="id"
      class="lista"
      role="menu"
      :aria-label="etiqueta"
      @keydown="teclaDelMenu"
    >
      <button
        v-for="(accion, indice) in acciones"
        :key="accion.id"
        :ref="(el) => { if (el) opciones[indice] = el as HTMLButtonElement }"
        class="opcion"
        :class="{ destructiva: accion.destructiva }"
        type="button"
        role="menuitem"
        tabindex="-1"
        :disabled="accion.deshabilitada"
        @click="elegir(accion)"
      >
        <AIcono
          v-if="accion.icono"
          :nombre="accion.icono"
        />
        <span>{{ accion.texto }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.menu {
  position: relative;
  display: inline-block;
}

.disparador {
  display: inline-flex;
  align-items: center;
  gap: var(--arles-space-2);
  min-height: var(--arles-control-height-compact);
  padding: 0 var(--arles-space-3);

  background: transparent;
  color: var(--arles-text);
  border: var(--arles-border-width) solid var(--arles-border-strong);
  border-radius: var(--arles-radius-md);

  font-family: inherit;
  font-size: var(--arles-font-size-body);
  font-weight: var(--arles-font-weight-semibold);
  cursor: pointer;
}

.disparador:hover {
  background: var(--arles-surface-hover);
}

.lista {
  position: absolute;
  z-index: 20;
  top: calc(100% + var(--arles-space-1));
  left: 0;
  min-width: var(--arles-menu-min-width);

  display: flex;
  flex-direction: column;
  padding: var(--arles-space-1);

  background: var(--arles-surface-raised);
  border: var(--arles-border-width) solid var(--arles-border-strong);
  border-radius: var(--arles-radius-md);
  box-shadow: var(--arles-shadow-menu);
}

.opcion {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  min-height: var(--arles-control-height-compact);
  padding: 0 var(--arles-space-3);

  background: transparent;
  color: var(--arles-text);
  border: none;
  border-radius: var(--arles-radius-sm);

  font-family: inherit;
  font-size: var(--arles-font-size-body);
  text-align: left;
  cursor: pointer;
}

.opcion:hover:not(:disabled) {
  background: var(--arles-surface-hover);
}

.opcion:disabled {
  /* Sin `opacity`: el texto secundario ya marca el estado y sigue pasando AA.
     Ver la nota de ABoton. */
  color: var(--arles-text-disabled);
  cursor: not-allowed;
}

/* La acción destructiva se marca con color Y con una separación: el color
   solo no la distingue para quien no lo percibe (regla 7.3). */
.destructiva {
  color: var(--arles-danger);
  margin-top: var(--arles-space-1);
  border-top: var(--arles-border-width) solid var(--arles-border);
  border-radius: 0 0 var(--arles-radius-sm) var(--arles-radius-sm);
}
</style>
