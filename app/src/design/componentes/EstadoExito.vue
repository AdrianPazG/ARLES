<script setup lang="ts">
/**
 * Estado de éxito (§94, §97, DESIGN_SYSTEM.md §8).
 *
 * **Breve, concreto, con el siguiente paso si lo hay. Sin celebraciones.**
 * Nada de «¡Listo! 🎉»: ARLES es una herramienta de trabajo y el usuario
 * acaba de hacer algo rutinario que va a repetir cien veces.
 *
 * `detalle` es para la cifra: «1 240 contactos importados», no «importación
 * completada». Un número es verificable; un adjetivo no.
 *
 * Nota de honestidad (§65): si lo que terminó fue un envío, el texto dice
 * *aceptado*, nunca *entregado*. Eso lo decide quien usa el componente, pero
 * hay un test que revisa todos los textos de `locales/es.ts` por si acaso.
 */
import AIcono from './AIcono.vue'

defineProps<{
  titulo: string
  /** La cifra concreta. */
  detalle?: string
}>()
</script>

<template>
  <div
    class="exito"
    role="status"
  >
    <AIcono
      class="icono"
      nombre="exito"
      :tamano="18"
    />

    <div class="cuerpo">
      <p class="titulo">
        {{ titulo }}
      </p>
      <p
        v-if="detalle"
        class="detalle"
      >
        {{ detalle }}
      </p>
      <div
        v-if="$slots.siguiente"
        class="siguiente"
      >
        <slot name="siguiente" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.exito {
  display: flex;
  align-items: flex-start;
  gap: var(--arles-space-3);
  max-width: 64ch;
  padding: var(--arles-space-3) var(--arles-space-4);

  background: var(--arles-surface);
  border: var(--arles-border-width) solid var(--arles-border);
  border-left: 3px solid var(--arles-success);
  border-radius: var(--arles-radius-md);
}

.icono {
  margin-top: 2px;
  color: var(--arles-success);
}

.cuerpo {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  min-width: 0;
}

.titulo {
  margin: 0;
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.detalle {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  font-variant-numeric: tabular-nums;
}

.siguiente {
  margin-top: var(--arles-space-2);
}
</style>
