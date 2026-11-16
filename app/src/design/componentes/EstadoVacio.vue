<script setup lang="ts">
/**
 * Estado vacío (§97, DESIGN_SYSTEM.md §8).
 *
 * **Explica qué es esto y da la acción para empezar.** No un dibujo con
 * «Nada por aquí»: el estado vacío es la primera pantalla que ve un usuario
 * nuevo y es la mejor oportunidad de explicar el producto que hay en toda la
 * aplicación. Desperdiciarla en un chiste es caro.
 *
 * El cuerpo no es opcional a propósito: un vacío sin explicación es
 * exactamente lo que este componente existe para impedir.
 */
defineProps<{
  titulo: string
  cuerpo: string
}>()
</script>

<template>
  <div class="vacio">
    <h2 class="titulo">
      {{ titulo }}
    </h2>
    <p class="cuerpo">
      {{ cuerpo }}
    </p>
    <div
      v-if="$slots.accion"
      class="accion"
    >
      <slot name="accion" />
    </div>
  </div>
</template>

<style scoped>
.vacio {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--arles-space-2);
  max-width: 56ch;
  padding: var(--arles-space-7) var(--arles-space-6);
}

.titulo {
  margin: 0;
  font-size: var(--arles-font-size-h2);
  line-height: var(--arles-line-height-h2);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
}

.cuerpo {
  margin: 0;
  color: var(--arles-text-muted);
}

.accion {
  margin-top: var(--arles-space-3);
}
</style>
