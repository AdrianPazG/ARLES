<script setup lang="ts">
/**
 * Modal (DESIGN_SYSTEM.md §6).
 *
 * Máximo 640 px. Foco atrapado dentro. `Esc` cierra **salvo en acciones
 * destructivas**, que exigen una decisión explícita: cerrar por accidente
 * una confirmación de borrado y creer que se canceló es el mismo error que
 * confirmarla sin querer, con el agravante de que nadie se entera.
 *
 * Se construye sobre `<dialog>` nativo, que aporta el atrapado de foco, la
 * capa superior y el fondo oscurecido sin reimplementar nada. Lo que sí hay
 * que gestionar a mano es `Esc`, porque el nativo siempre cierra.
 *
 * **Detener una campaña dice cuántos mensajes quedarán sin enviar.** Un número
 * concreto, no «¿estás seguro?». Eso vive en quien usa este componente, pero
 * la ranura `detalle` existe precisamente para que tenga dónde ponerlo.
 */
import { nextTick, onBeforeUnmount, ref, useId, watch } from 'vue'

import AIcono from './AIcono.vue'

const props = withDefaults(
  defineProps<{
    abierto: boolean
    titulo: string
    /** Destructiva: `Esc` y el clic fuera dejan de cerrar. */
    destructiva?: boolean
  }>(),
  { destructiva: false },
)

const emit = defineEmits<{ cerrar: [] }>()

const dialogo = ref<HTMLDialogElement | null>(null)
const id = useId()

function intentarCerrar() {
  // En una acción destructiva sólo cierran los botones del pie: la decisión
  // tiene que ser explícita en las dos direcciones.
  if (props.destructiva) return
  emit('cerrar')
}

function alPulsarTecla(evento: KeyboardEvent) {
  if (evento.key !== 'Escape') return
  // `<dialog>` cierra con Esc por su cuenta; hay que impedirlo para poder
  // decidir nosotros, y para que el estado del componente no se desincronice
  // del atributo `abierto` que controla quien lo usa.
  evento.preventDefault()
  intentarCerrar()
}

function alPulsarFuera(evento: MouseEvent) {
  // El ::backdrop pertenece al propio <dialog>, así que un clic en él llega
  // con el diálogo como destino. Los hijos no lo son.
  if (evento.target === dialogo.value) intentarCerrar()
}

watch(
  () => props.abierto,
  async (abierto) => {
    await nextTick()
    const el = dialogo.value
    if (!el) return
    if (abierto && !el.open) el.showModal()
    if (!abierto && el.open) el.close()
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  // Un diálogo modal desmontado sin cerrar deja el resto de la página inerte.
  if (dialogo.value?.open) dialogo.value.close()
})
</script>

<template>
  <dialog
    ref="dialogo"
    class="modal"
    :aria-labelledby="`${id}-titulo`"
    @keydown="alPulsarTecla"
    @click="alPulsarFuera"
    @cancel.prevent
  >
    <header class="cabecera">
      <h2
        :id="`${id}-titulo`"
        class="titulo"
      >
        {{ titulo }}
      </h2>

      <button
        v-if="!destructiva"
        class="cerrar"
        type="button"
        @click="emit('cerrar')"
      >
        <AIcono
          nombre="cerrar"
          etiqueta="Cerrar"
          :tamano="18"
        />
      </button>
    </header>

    <div class="cuerpo">
      <slot />
      <!-- Aquí va el número concreto: «quedarán 1 240 mensajes sin enviar». -->
      <div
        v-if="$slots.detalle"
        class="detalle"
      >
        <slot name="detalle" />
      </div>
    </div>

    <footer class="pie">
      <slot name="acciones" />
    </footer>
  </dialog>
</template>

<style scoped>
.modal {
  width: min(var(--arles-modal-max-width), calc(100vw - var(--arles-space-8)));
  max-width: var(--arles-modal-max-width);
  padding: 0;

  background: var(--arles-surface-raised);
  color: var(--arles-text);
  border: var(--arles-border-width) solid var(--arles-border-strong);
  border-radius: var(--arles-radius-lg);

  box-shadow: var(--arles-shadow-modal);
}

.modal::backdrop {
  background: var(--arles-backdrop);
}

.cabecera {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--arles-space-4);
  padding: var(--arles-space-5) var(--arles-space-5) var(--arles-space-3);
}

.titulo {
  margin: 0;
  font-size: var(--arles-font-size-h2);
  line-height: var(--arles-line-height-h2);
  font-weight: var(--arles-font-weight-bold);
}

.cerrar {
  display: flex;
  align-items: center;
  justify-content: center;
  /* 32 px: el mínimo de objetivo táctil del sistema (§22). */
  width: var(--arles-control-height-compact);
  height: var(--arles-control-height-compact);
  flex: none;

  background: transparent;
  color: var(--arles-text-muted);
  border: none;
  border-radius: var(--arles-radius-sm);
  cursor: pointer;
  transition: background var(--arles-duration-fast) var(--arles-ease);
}

.cerrar:hover {
  background: var(--arles-surface-hover);
  color: var(--arles-text);
}

.cuerpo {
  padding: 0 var(--arles-space-5);
  color: var(--arles-text-muted);
}

.detalle {
  margin-top: var(--arles-space-4);
  padding: var(--arles-space-3) var(--arles-space-4);
  background: var(--arles-surface);
  border-radius: var(--arles-radius-md);
  color: var(--arles-text);
  font-variant-numeric: tabular-nums;
}

.pie {
  display: flex;
  justify-content: flex-end;
  gap: var(--arles-space-2);
  padding: var(--arles-space-5);
}
</style>
