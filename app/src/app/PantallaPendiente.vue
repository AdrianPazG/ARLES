<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

import type { Seccion } from '@/app/router'

const ruta = useRoute()
const seccion = computed(() => ruta.meta['seccion'] as Seccion | undefined)
</script>

<template>
  <section>
    <h1 class="titulo">
      {{ seccion ? $t(`nav.${seccion}`) : '' }}
    </h1>
    <p class="nota">
      Esta sección se construye en su fase del roadmap. El andamiaje ya aplica
      los tokens de diseño, la navegación y los textos localizados.
    </p>
  </section>
</template>

<style scoped>
.titulo {
  margin: 0 0 var(--arles-space-3);
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.nota {
  margin: 0;
  max-width: 62ch;
  color: var(--arles-text-muted);
}
</style>
