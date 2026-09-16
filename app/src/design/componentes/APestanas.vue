<script setup lang="ts">
/**
 * Pestañas.
 *
 * Patrón ARIA completo con **tabindex móvil**: sólo la pestaña activa está en
 * el orden de tabulación, y dentro del grupo se navega con flechas. El error
 * habitual es dejar las seis en el orden de tabulación, lo que obliga a
 * atravesarlas todas para llegar al contenido.
 *
 * `Inicio` y `Fin` saltan a los extremos; las flechas dan la vuelta.
 */
import { computed, nextTick, ref, useId } from 'vue'

import { despachar } from './teclado'

export interface Pestana {
  id: string
  texto: string
  deshabilitada?: boolean
}

const props = defineProps<{ pestanas: readonly Pestana[] }>()
const activa = defineModel<string>({ required: true })

const grupo = useId()
const botones = ref<HTMLButtonElement[]>([])

const seleccionables = computed(() =>
  props.pestanas.filter((p) => !p.deshabilitada),
)

function seleccionar(id: string) {
  activa.value = id
}

function mover(delta: number) {
  const lista = seleccionables.value
  if (lista.length === 0) return
  const actual = lista.findIndex((p) => p.id === activa.value)
  // El módulo con desplazamiento positivo da la vuelta también hacia atrás.
  const siguiente = lista[(actual + delta + lista.length) % lista.length]
  if (siguiente) seleccionar(siguiente.id)
}

async function enfocarLaActiva() {
  await nextTick()
  const indice = props.pestanas.findIndex((p) => p.id === activa.value)
  botones.value[indice]?.focus()
}

function alPulsarTecla(evento: KeyboardEvent) {
  const movio = despachar(
    {
      ArrowRight: () => mover(1),
      ArrowLeft: () => mover(-1),
      Home: () => {
        const primera = seleccionables.value[0]
        if (primera) seleccionar(primera.id)
      },
      End: () => {
        const ultima = seleccionables.value.at(-1)
        if (ultima) seleccionar(ultima.id)
      },
    },
    evento,
  )

  // El foco sigue a la selección SÓLO cuando la movió el teclado. Si esto
  // viviera en un `watch` sobre el modelo, cualquier cambio desde fuera —una
  // ruta, un botón de otra parte de la pantalla— arrancaría el foco de donde
  // el usuario lo tuviera y lo dejaría en la tira de pestañas.
  //
  // Y tiene que seguirla: si no, la siguiente flecha se calcularía desde
  // donde quedó el foco y no desde lo que el usuario ve marcado.
  if (movio) void enfocarLaActiva()
}
</script>

<template>
  <div>
    <div
      class="tiras"
      role="tablist"
      @keydown="alPulsarTecla"
    >
      <button
        v-for="(pestana, indice) in pestanas"
        :id="`${grupo}-${pestana.id}`"
        :key="pestana.id"
        :ref="(el) => { if (el) botones[indice] = el as HTMLButtonElement }"
        class="tira"
        type="button"
        role="tab"
        :aria-selected="pestana.id === activa"
        :aria-controls="`${grupo}-${pestana.id}-panel`"
        :tabindex="pestana.id === activa ? 0 : -1"
        :disabled="pestana.deshabilitada"
        @click="seleccionar(pestana.id)"
      >
        {{ pestana.texto }}
      </button>
    </div>

    <div
      :id="`${grupo}-${activa}-panel`"
      class="panel"
      role="tabpanel"
      :aria-labelledby="`${grupo}-${activa}`"
      tabindex="0"
    >
      <slot :activa="activa" />
    </div>
  </div>
</template>

<style scoped>
.tiras {
  display: flex;
  gap: var(--arles-space-1);
  border-bottom: var(--arles-border-width) solid var(--arles-border);
}

.tira {
  padding: var(--arles-space-2) var(--arles-space-3);
  background: transparent;
  color: var(--arles-text-muted);
  border: none;
  /* El subrayado activo se dibuja aquí para que el elemento no cambie de
     alto al seleccionarse: un salto de 2 px desplaza toda la pantalla. */
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;

  font-family: inherit;
  font-size: var(--arles-font-size-body);
  line-height: var(--arles-line-height-body);
  font-weight: var(--arles-font-weight-semibold);
  cursor: pointer;
  transition: color var(--arles-duration-fast) var(--arles-ease);
}

.tira:hover:not(:disabled) {
  color: var(--arles-text);
}

/* Seleccionada: color Y subrayado. Dos señales, no sólo color (regla 7.3). */
.tira[aria-selected='true'] {
  color: var(--arles-text);
  border-bottom-color: var(--arles-accent-ink);
}

.tira:disabled {
  /* Sin `opacity`: el texto secundario ya marca el estado y sigue pasando AA.
     Ver la nota de ABoton. */
  color: var(--arles-text-disabled);
  cursor: not-allowed;
}

.panel {
  padding-top: var(--arles-space-4);
}

/* El panel es enfocable para que el teclado pueda entrar al contenido. Como
   ES una parada de tabulación, tiene que dibujar su anillo: un elemento que
   recibe el foco sin señal visible deja al usuario de teclado sin saber dónde
   está, que es justo lo que prohíbe el §100.
   
   La versión anterior ponía `outline: none` sobre `:focus` a secas y lo
   apagaba también para el teclado. Lo encontró la sonda de teclado; el
   validador lo dejó pasar porque su regla perdonaba cualquier selector con
   `:focus`, y esa laguna se cerró a la vez.
   
   `:focus:not(:focus-visible)` es lo único que se apaga: llegar por clic. */
.panel:focus:not(:focus-visible) {
  outline: none;
}
</style>
