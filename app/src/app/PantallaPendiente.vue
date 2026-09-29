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
      {{ $t('seccionPendiente.cuerpo') }}
    </p>
    <p class="nota">
      {{ $t('seccionPendiente.mientrasTanto') }}
    </p>
    <!-- Dos salidas, y no sólo la mención de dónde están.
         La sonda del recorrido contaba **cero acciones** aquí: de esta
         pantalla sólo se salía por la barra lateral, que es la definición de
         callejón. Decir «está en Contactos › Bajas» obliga a buscarlo; un
         enlace lleva. -->
    <ul class="salidas">
      <li>
        <RouterLink :to="{ name: 'campanas' }">
          {{ $t('seccionPendiente.verCampanas') }}
        </RouterLink>
      </li>
      <li>
        <RouterLink :to="{ name: 'supresion' }">
          {{ $t('seccionPendiente.verBajas') }}
        </RouterLink>
      </li>
    </ul>
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
  margin: 0 0 var(--arles-space-3);
  max-width: 62ch;
  color: var(--arles-text-muted);
}

.salidas {
  margin: 0;
  padding-left: var(--arles-space-4);
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
}
</style>
