<script setup lang="ts">
/**
 * Estado de carga (§97, DESIGN_SYSTEM.md §8).
 *
 * **Esqueleto con la forma del contenido real, no un girador centrado.** Un
 * girador no dice cuánto falta ni qué va a aparecer; un esqueleto reserva el
 * espacio, así que al llegar los datos la pantalla no salta.
 *
 * **Si supera 2 s, se explica qué está pasando.** Ese es el umbral a partir
 * del cual una espera sin explicación se interpreta como que algo se rompió.
 * El aviso aparece solo: quien usa el componente no tiene que acordarse.
 *
 * Nada parpadea (§98): el esqueleto respira con una opacidad suave, y con
 * `prefers-reduced-motion` se queda quieto — lo aplica base.css a todo.
 */
import { onBeforeUnmount, onMounted, ref } from 'vue'

withDefaults(
  defineProps<{
    /** Cuántas líneas de esqueleto. Debe parecerse al contenido que viene. */
    lineas?: number
    /** Qué se está haciendo. Se muestra a partir de los 2 s. */
    explicacion?: string
  }>(),
  { lineas: 5 },
)

const tarda = ref(false)
let temporizador: ReturnType<typeof setTimeout> | undefined

onMounted(() => {
  temporizador = setTimeout(() => (tarda.value = true), 2000)
})

onBeforeUnmount(() => clearTimeout(temporizador))
</script>

<template>
  <!-- `aria-busy` y `polite`: el lector de pantalla anuncia que se está
       cargando sin interrumpir lo que estuviera leyendo. -->
  <div
    class="cargando"
    role="status"
    aria-busy="true"
    aria-live="polite"
  >
    <span class="sr-solo">{{ $t('comun.cargando') }}</span>

    <div
      v-for="n in lineas"
      :key="n"
      class="linea"
      aria-hidden="true"
    />

    <p
      v-if="tarda && explicacion"
      class="explicacion"
    >
      {{ explicacion }}
    </p>
  </div>
</template>

<style scoped>
.cargando {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
  padding: var(--arles-space-4) 0;
}

.linea {
  height: var(--arles-font-size-body);
  border-radius: var(--arles-radius-sm);
  background: var(--arles-surface-raised);
  animation: respirar 1.6s var(--arles-ease) infinite;
}

/* Las líneas no miden todas lo mismo: un bloque de barras idénticas no se
   parece a un texto y se lee como una barra de progreso rota. */
.linea:nth-child(3n) {
  width: 72%;
}

.linea:nth-child(3n + 1) {
  width: 94%;
}

.linea:nth-child(3n + 2) {
  width: 61%;
}

/* Opacidad, no color ni posición: nada parpadea (§98) y el movimiento no
   arrastra la mirada fuera de donde el usuario está leyendo. */
@keyframes respirar {
  0%,
  100% {
    opacity: 0.45;
  }

  50% {
    opacity: 0.8;
  }
}

.explicacion {
  margin: var(--arles-space-2) 0 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

/* Visible sólo para lectores de pantalla. */
.sr-solo {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}
</style>
