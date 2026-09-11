<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterLink, RouterView } from 'vue-router'

import { SECCIONES } from '@/app/router'
import { useAppStore } from '@/app/stores/app'

const app = useAppStore()
onMounted(() => void app.cargar())
</script>

<template>
  <div class="marco">
    <nav
      class="nav"
      aria-label="Navegación principal"
    >
      <div class="marca">
        <span class="marca-arles">ARLES</span>
        <span class="marca-relay">RELAY</span>
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

/* El logotipo es exclusivamente tipográfico (§21). Sin isotipo. */
.marca {
  font-weight: var(--arles-font-weight-black);
  font-size: var(--arles-font-size-h2);
  letter-spacing: 0.02em;
  margin-bottom: var(--arles-space-7);
  padding-inline: var(--arles-space-2);
}

.marca-relay {
  color: var(--arles-accent);
  margin-left: 0.35ch;
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
