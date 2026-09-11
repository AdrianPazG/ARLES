<script setup lang="ts">
/**
 * Aviso en línea.
 *
 * Es el componente que sostiene el principio de honestidad (§65, §67): donde
 * ARLES no sabe algo, lo declara aquí, en el sitio donde la duda aparece, y
 * no en una nota de pie que nadie lee.
 *
 * `role="alert"` sólo cuando el aviso aparece como consecuencia de algo que
 * el usuario acaba de hacer. Un aviso permanente con `role="alert"` se
 * anuncia cada vez que la pantalla se vuelve a montar, lo que convierte al
 * lector de pantalla en ruido.
 */
import { computed } from 'vue'

import AIcono, { type NombreDeIcono } from './AIcono.vue'

type Tono = 'info' | 'aviso' | 'peligro' | 'exito'

const ICONO: Record<Tono, NombreDeIcono> = {
  info: 'info',
  aviso: 'aviso',
  peligro: 'error',
  exito: 'exito',
}

const props = withDefaults(
  defineProps<{
    tono?: Tono
    titulo?: string
    /** Anúncialo al aparecer. Sólo para avisos que responden a una acción. */
    urgente?: boolean
  }>(),
  { tono: 'info', urgente: false },
)

const icono = computed(() => ICONO[props.tono])
</script>

<template>
  <div
    class="aviso"
    :class="`t-${tono}`"
    :role="urgente ? 'alert' : 'note'"
  >
    <AIcono
      class="icono"
      :nombre="icono"
      :tamano="18"
    />

    <div class="cuerpo">
      <p
        v-if="titulo"
        class="titulo"
      >
        {{ titulo }}
      </p>
      <div class="texto">
        <slot />
      </div>
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
.aviso {
  display: flex;
  gap: var(--arles-space-3);
  padding: var(--arles-space-3) var(--arles-space-4);

  background: var(--arles-surface);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);

  /* El tono se marca con una barra lateral ADEMÁS del color del icono: dos
     señales, no una de color sola (regla 7.3). */
  border-left-width: 3px;
}

.icono {
  margin-top: 1px;
}

.t-info {
  border-left-color: var(--arles-info);
}

.t-info .icono {
  color: var(--arles-info);
}

.t-aviso {
  border-left-color: var(--arles-warning);
}

.t-aviso .icono {
  color: var(--arles-warning);
}

.t-peligro {
  border-left-color: var(--arles-danger);
}

.t-peligro .icono {
  color: var(--arles-danger);
}

.t-exito {
  border-left-color: var(--arles-success);
}

.t-exito .icono {
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

/* El cuerpo del aviso va en el texto secundario y el título en el primario:
   la jerarquía la marca el contraste, no el tamaño, para no romper la
   densidad de la interfaz. Ambos pasan AA sobre `surface`. */
.texto {
  color: var(--arles-text-muted);
}

.acciones {
  display: flex;
  gap: var(--arles-space-2);
  margin-top: var(--arles-space-2);
}
</style>
