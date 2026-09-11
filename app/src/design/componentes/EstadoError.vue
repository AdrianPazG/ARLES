<script setup lang="ts">
/**
 * Estado de error (§95, §97, DESIGN_SYSTEM.md §8).
 *
 * **Qué pasó, cómo arreglarlo, y qué está a salvo.** Las tres partes son
 * propiedades obligatorias: el compilador rechaza construir este componente
 * sin las tres. La tercera es la que falta en casi todo el software y la que
 * importa cuando alguien acaba de ver fallar una campaña de 40 000 correos.
 *
 * Es el mismo contrato que `ErrorIpc` en Rust y que `resolverError()` en
 * `app/src/app/errores.ts`: un error que cruza la frontera IPC llega ya con
 * sus tres partes, y este componente es donde se enseñan.
 */
import AIcono from './AIcono.vue'

defineProps<{
  que: string
  como: string
  salvo: string
}>()
</script>

<template>
  <!-- `role="alert"`: el error aparece como consecuencia de algo que el
       usuario acaba de intentar, así que sí debe anunciarse al momento. -->
  <div
    class="error"
    role="alert"
  >
    <AIcono
      class="icono"
      nombre="error"
      :tamano="20"
    />

    <div class="cuerpo">
      <p class="que">
        {{ que }}
      </p>
      <p class="como">
        {{ como }}
      </p>
      <!-- Destacado, no como pie de página: es la parte que baja la tensión
           de quien acaba de ver fallar algo. -->
      <p class="salvo">
        {{ salvo }}
      </p>

      <div
        v-if="$slots.acciones"
        class="acciones"
      >
        <slot name="acciones" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.error {
  display: flex;
  gap: var(--arles-space-3);
  max-width: 64ch;
  padding: var(--arles-space-4);

  background: var(--arles-surface);
  border: var(--arles-border-width) solid var(--arles-border);
  border-left: 3px solid var(--arles-danger);
  border-radius: var(--arles-radius-md);
}

.icono {
  margin-top: 2px;
  color: var(--arles-danger);
}

.cuerpo {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
  min-width: 0;
}

.que,
.como,
.salvo {
  margin: 0;
}

.que {
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.como {
  color: var(--arles-text-muted);
}

/* «Qué está a salvo» va sobre fondo propio para que se lea aunque el usuario
   sólo mire por encima. El acento de la barra es el de éxito a propósito:
   dentro de un error, esta frase es la buena noticia. */
.salvo {
  padding: var(--arles-space-2) var(--arles-space-3);
  background: var(--arles-surface-raised);
  border-left: 2px solid var(--arles-success);
  border-radius: var(--arles-radius-sm);
  color: var(--arles-text);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

.acciones {
  display: flex;
  gap: var(--arles-space-2);
  margin-top: var(--arles-space-1);
}
</style>
