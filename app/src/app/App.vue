<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterLink, RouterView } from 'vue-router'

import { CATALOGO_VISIBLE, SECCIONES } from '@/app/router'
import { useAppStore } from '@/app/stores/app'
import { ALogotipo } from '@/design/componentes'

const app = useAppStore()
onMounted(() => void app.cargar())
</script>

<template>
  <div class="marco">
    <nav
      class="nav"
      aria-label="Navegación principal"
    >
      <!-- El logotipo vive en un componente: el §21 prohíbe estilizarlo, y
           repetir su marcado en cada sitio es justo como se acaba estilizando
           en uno de ellos. -->
      <div class="marca">
        <ALogotipo />
      </div>

      <ul class="nav-lista">
        <li
          v-for="seccion in SECCIONES"
          :key="seccion"
        >
          <RouterLink
            class="nav-enlace"
            :to="`/${seccion}`"
          >
            {{ $t(`nav.${seccion}`) }}
          </RouterLink>
        </li>
      </ul>

      <!-- En una ventana nativa no hay barra de direcciones: sin este enlace,
           el catálogo existe pero no hay forma de llegar a él. Sólo aparece
           cuando el catálogo está compilado dentro — en desarrollo y en la
           compilación de revisión visual.

           Precisión: en producción `CATALOGO_VISIBLE` es `false`, así que el
           enlace nunca se pinta y el componente del catálogo no entra en el
           bundle. Lo que sí queda son estas pocas letras de plantilla. No es
           superficie de ataque; es un detalle que conviene no vender como
           «no viaja nada». -->
      <RouterLink
        v-if="CATALOGO_VISIBLE"
        class="nav-enlace enlace-de-revision"
        to="/catalogo"
      >
        Catálogo del sistema
      </RouterLink>

      <!-- §121: la atribución vive aquí, discreta. NUNCA en los correos
           que el cliente envía. -->
      <footer class="pie">
        <p class="atribucion">
          {{ $t('producto.atribucion') }}
        </p>
        <p class="version">
          v{{ app.info.version }}
        </p>
      </footer>
    </nav>

    <main class="contenido">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.marco {
  display: grid;
  grid-template-columns: var(--arles-nav-width) 1fr;
  min-width: var(--arles-window-min-width);
  /* 100vh en vez de 100%: no depende de que html, body y #app mantengan la
     cadena de alturas porcentuales, que se rompe en cuanto alguien inserta un
     envoltorio. */
  min-height: 100vh;
}

.nav {
  display: flex;
  flex-direction: column;
  background: var(--arles-surface);
  border-right: var(--arles-border-width) solid var(--arles-border);
  padding: var(--arles-space-5) var(--arles-space-4);
}

.marca {
  margin-bottom: var(--arles-space-7);
  padding-inline: var(--arles-space-2);
}

.nav-lista {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  flex: 1;
}

.nav-enlace {
  display: block;
  padding: var(--arles-space-2) var(--arles-space-3);
  border-radius: var(--arles-radius-md);
  color: var(--arles-text-muted);
  text-decoration: none;
  font-weight: var(--arles-font-weight-semibold);
  transition: background var(--arles-duration-fast) var(--arles-ease);
}

.nav-enlace:hover {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
}

/* La sección activa se marca con color Y con una barra: el color solo no basta
   (§19, COLOR_SYSTEM.md §7.3). */
.nav-enlace.router-link-active {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
  box-shadow: inset 3px 0 0 var(--arles-accent);
}

/* Se distingue de la navegación real: no es una sección del producto, es una
   herramienta de revisión que no viaja al cliente. */
.enlace-de-revision {
  margin-bottom: var(--arles-space-3);
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-disabled);
  border: var(--arles-border-width) dashed var(--arles-border-strong);
}

.pie {
  padding: var(--arles-space-3) var(--arles-space-2) 0;
  border-top: var(--arles-border-width) solid var(--arles-border);
}

.atribucion,
.version {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  opacity: 0.75;
}

.version {
  margin-top: var(--arles-space-1);
}

.contenido {
  padding: var(--arles-space-6);
  overflow: auto;
}
</style>
