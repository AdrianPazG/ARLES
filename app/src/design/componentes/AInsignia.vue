<script setup lang="ts">
/**
 * Insignia de estado (DESIGN_SYSTEM.md §6).
 *
 * **Relleno sólido con texto oscuro**, no color de texto sobre transparente.
 * El motivo está medido: sobre `surface-raised` los colores semánticos dan
 * entre 2.58 y 2.92 contra el fondo — todos por debajo de AA. Con relleno
 * sólido, el contraste que cuenta es el del relleno contra
 * `--arles-text-on-accent`, que pasa siempre.
 *
 * **Cada insignia lleva icono además del color** (regla 7.3). No hay forma de
 * construir una sin él: el icono se deriva del tono, no se pasa por fuera.
 */
import { computed } from 'vue'

import AIcono, { type NombreDeIcono } from './AIcono.vue'

type Tono = 'neutro' | 'exito' | 'aviso' | 'peligro' | 'info' | 'incierto'

const ICONO: Record<Tono, NombreDeIcono> = {
  neutro: 'cola',
  exito: 'exito',
  aviso: 'aviso',
  peligro: 'error',
  info: 'info',
  // ADR-0004: el envío ambiguo no es un éxito ni un fallo. Mostrarlo con el
  // icono de cualquiera de los dos afirmaría algo que no se sabe.
  incierto: 'incierto',
}

const props = withDefaults(defineProps<{ tono?: Tono }>(), { tono: 'neutro' })

const icono = computed(() => ICONO[props.tono])
</script>

<template>
  <span
    class="insignia"
    :class="`t-${tono}`"
  >
    <AIcono :nombre="icono" />
    <span><slot /></span>
  </span>
</template>

<style scoped>
.insignia {
  display: inline-flex;
  align-items: center;
  gap: var(--arles-space-1);

  padding: 2px var(--arles-space-2);
  border-radius: var(--arles-radius-sm);

  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  font-weight: var(--arles-font-weight-semibold);
  white-space: nowrap;

  /* Todos los rellenos claros llevan el mismo texto oscuro: es lo que hace
     que el contraste sea el del relleno y no dependa del tono. */
  color: var(--arles-text-on-accent);
}

.t-exito {
  background: var(--arles-success);
}

.t-aviso {
  background: var(--arles-warning);
}

.t-peligro {
  background: var(--arles-danger);
}

.t-info {
  background: var(--arles-info);
}

.t-incierto {
  background: var(--arles-accent);
}

/* El neutro es la excepción y por eso va al revés: no hay un relleno claro
   que represente «nada todavía» sin gritar. Lleva borde para no confundirse
   con texto suelto, y su texto sí es el claro de la aplicación, que pasa AA
   sobre `surface-raised` (COLOR_SYSTEM). */
.t-neutro {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
  border: var(--arles-border-width) solid var(--arles-border-strong);
  /* Compensa el borde para que todas las insignias midan igual de alto. */
  padding: 1px calc(var(--arles-space-2) - 1px);
}
</style>
