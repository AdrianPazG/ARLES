<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink, RouterView } from 'vue-router'

import { CATALOGO_VISIBLE, ICONO_DE_SECCION, SECCIONES } from '@/app/router'
import { useAppStore } from '@/app/stores/app'
import { useInterfazStore } from '@/app/stores/interfaz'
import { AIcono, ALogotipo } from '@/design/componentes'

const app = useAppStore()
const interfaz = useInterfazStore()
const { t } = useI18n()

const etiquetaDelBoton = computed(() =>
  interfaz.plegada ? t('nav.expandir') : t('nav.plegar'),
)

// Cuando la barra se pliega sola, el botón queda deshabilitado y **dice por
// qué**. Un control deshabilitado sin explicación se lee como una avería.
const explicacionDelBoton = computed(() =>
  interfaz.alternableAhora ? etiquetaDelBoton.value : t('nav.plegadaPorAncho'),
)

function medir(): void {
  interfaz.anotarAncho(window.innerWidth)
}

onMounted(() => {
  void app.cargar()
  void interfaz.cargar()
  medir()
  window.addEventListener('resize', medir)
})

onBeforeUnmount(() => window.removeEventListener('resize', medir))
</script>

<template>
  <div
    class="marco"
    :class="{ plegado: interfaz.plegada }"
  >
    <nav
      id="navegacion-principal"
      class="nav"
      :aria-label="$t('nav.principal')"
    >
      <!-- El logotipo vive en un componente: el §21 prohíbe estilizarlo, y
           repetir su marcado en cada sitio es justo como se acaba estilizando
           en uno de ellos.

           Plegada, el logotipo no cabe —«ARLES RELAY» en Mont Black no entra
           en 64 px— y se oculta entero en vez de recortarse. El §21 prohíbe
           condensarlo, y un logotipo cortado por la mitad es peor que ninguno.
           La ventana sigue diciendo el nombre en su barra de título. -->
      <div
        v-if="!interfaz.plegada"
        class="marca"
      >
        <ALogotipo />
      </div>

      <button
        type="button"
        class="plegador"
        :class="{ 'plegador-solo-icono': interfaz.plegada }"
        :disabled="!interfaz.alternableAhora"
        :aria-expanded="!interfaz.plegada"
        aria-controls="navegacion-principal"
        :aria-label="etiquetaDelBoton"
        :title="explicacionDelBoton"
        @click="interfaz.alternar()"
      >
        <AIcono :nombre="interfaz.plegada ? 'expandir' : 'plegar'" />
      </button>

      <ul class="nav-lista">
        <li
          v-for="seccion in SECCIONES"
          :key="seccion"
        >
          <RouterLink
            class="nav-enlace"
            :to="`/${seccion}`"
            :title="interfaz.plegada ? $t(`nav.${seccion}`) : undefined"
          >
            <AIcono :nombre="ICONO_DE_SECCION[seccion]" />
            <!-- Plegada se quita el TEXTO, no el nombre accesible: el enlace
                 sigue anunciándose «Campañas» porque el texto sigue en el
                 árbol, sólo que oculto a la vista. Con `display: none` el
                 lector de pantalla leería seis enlaces sin nombre. -->
            <span :class="interfaz.plegada ? 'solo-lectores' : 'nav-texto'">
              {{ $t(`nav.${seccion}`) }}
            </span>
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
        v-if="CATALOGO_VISIBLE && !interfaz.plegada"
        class="nav-enlace enlace-de-revision"
        to="/catalogo"
      >
        Catálogo del sistema
      </RouterLink>

      <!-- §121: la atribución vive aquí, discreta. NUNCA en los correos
           que el cliente envía. -->
      <footer
        v-if="!interfaz.plegada"
        class="pie"
      >
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
  /* Aquí había `min-width: var(--arles-window-min-width)` —1120 px—, con el
     razonamiento de que la ventana no puede ser más estrecha, así que el
     contenido tampoco tendría que serlo.

     Es falso en cuanto la escala de Windows pasa del 100 %. El mínimo de
     `tauri.conf.json` está en píxeles LÓGICOS: a 200 %, pedir 1120 lógicos es
     pedir 2240 físicos, más que una pantalla de 1920 entera. La ventana no
     puede cumplirlo, Windows la deja en 960 lógicos, y el suelo de CSS —que
     no cedía— provocaba desplazamiento horizontal y contenido cortado.

     Confundimos el mínimo de la VENTANA con el mínimo del DISEÑO. Lo
     encontró Dirección revisando a 200 %; lo vigila `sonda:ancho`. */

  /* `height`, no `min-height`: es lo que hace que la barra lateral se quede
     fija (P-11 a). Con `min-height` el armazón crece con el contenido y la
     página entera se desplaza, llevándose la navegación hacia arriba. Con la
     altura fijada, lo único que se desplaza es `.contenido`. */
  height: 100vh;
  overflow: hidden;
}

.marco.plegado {
  grid-template-columns: var(--arles-nav-width-plegada) 1fr;
}

.nav {
  display: flex;
  flex-direction: column;
  background: var(--arles-surface);
  border-right: var(--arles-border-width) solid var(--arles-border);
  padding: var(--arles-space-5) var(--arles-space-4);
  /* Una navegación con más secciones de las que caben tiene que poder
     recorrerse. Hoy son seis y sobran; el día que no, se desplaza sola. */
  overflow-y: auto;
  transition: padding var(--arles-duration-normal) var(--arles-ease);
}

.plegado .nav {
  padding-inline: var(--arles-space-2);
  align-items: center;
}

.marca {
  margin-bottom: var(--arles-space-5);
  padding-inline: var(--arles-space-2);
}

.plegador {
  align-self: flex-end;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--arles-control-height-compact);
  height: var(--arles-control-height-compact);
  margin-bottom: var(--arles-space-4);
  padding: 0;
  border: var(--arles-border-width) solid transparent;
  border-radius: var(--arles-radius-md);
  background: transparent;
  color: var(--arles-text-muted);
  cursor: pointer;
  transition:
    background var(--arles-duration-fast) var(--arles-ease),
    color var(--arles-duration-fast) var(--arles-ease);
}

.plegador:hover:not(:disabled) {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
}

.plegador:disabled {
  color: var(--arles-text-disabled);
  cursor: not-allowed;
}

.plegador-solo-icono {
  align-self: center;
}

.nav-lista {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  flex: 1;
  width: 100%;
}

.nav-enlace {
  display: flex;
  align-items: center;
  gap: var(--arles-space-3);
  padding: var(--arles-space-2) var(--arles-space-3);
  border-radius: var(--arles-radius-md);
  color: var(--arles-text-muted);
  text-decoration: none;
  font-weight: var(--arles-font-weight-semibold);
  transition: background var(--arles-duration-fast) var(--arles-ease);
}

.plegado .nav-enlace {
  justify-content: center;
  padding-inline: 0;
}

.nav-texto {
  /* El texto no puede empujar el ancho de la barra mientras se pliega: sin
     esto, la animación da un tirón cuando «Remitentes» deja de caber. */
  overflow: hidden;
  white-space: nowrap;
}

/* Visible para el lector de pantalla, invisible para la vista.
   `display: none` lo sacaría del árbol de accesibilidad y dejaría seis
   enlaces sin nombre — exactamente lo que `sonda:lector` prohíbe. */
.solo-lectores {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px;
  padding: 0;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
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
