<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink, RouterView } from 'vue-router'

import { invocar } from '@/app/nucleo'
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

/**
 * Abre telemetrymx.com en el navegador del sistema.
 *
 * El comando **no recibe la URL**: el destino es una constante compilada en
 * Rust (`comandos::SITIO_DE_TELEMETRY`). Un comando que aceptara una dirección
 * dejaría a la webview eligiendo a dónde se navega, y la seguridad dependería
 * de una expresión regular bien escrita.
 *
 * Si falla, no se interrumpe nada: se deja constancia. Un logotipo que no abre
 * el sitio es una molestia; un diálogo de error por ello, una avería aparente.
 */
async function abrirSitio(): Promise<void> {
  try {
    await invocar('abrir_sitio_de_telemetry')
  } catch (e) {
    console.error('No se pudo abrir el sitio de TELEMETRY', e)
  }
}

function medir(): void {
  interfaz.anotarAncho(window.innerWidth)
}

/**
 * El tema del sistema operativo (C-2).
 *
 * `matchMedia` y el `<html>` son cosas del navegador, y el almacén no las toca:
 * ahí vive la decisión —qué tema corresponde—, aquí la aplicación de esa
 * decisión. Si el almacén escribiera el atributo, no se podría probar sin un
 * documento montado.
 *
 * `null` cuando el entorno no ofrece `matchMedia`. Entonces no se escucha nada
 * y «Automático» se queda en el tema de la casa, que es el oscuro (§18).
 */
const consultaDelSistema =
  typeof window !== 'undefined' && typeof window.matchMedia === 'function'
    ? window.matchMedia('(prefers-color-scheme: light)')
    : null

function anotarTema(e: MediaQueryList | MediaQueryListEvent): void {
  interfaz.anotarTemaDelSistema(e.matches)
}

// Escribir el atributo en <html> y no en un div: `base.css` y los tokens lo
// leen desde `:root`, y los portales —menús, diálogos— cuelgan de <body>, así
// que un contenedor intermedio los dejaría con el tema anterior.
watchEffect(() => {
  document.documentElement.setAttribute('data-tema', interfaz.temaAplicado)
})

onMounted(() => {
  void app.cargar()
  void interfaz.cargar()
  medir()
  window.addEventListener('resize', medir)
  if (consultaDelSistema) {
    anotarTema(consultaDelSistema)
    consultaDelSistema.addEventListener('change', anotarTema)
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', medir)
  consultaDelSistema?.removeEventListener('change', anotarTema)
})
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
      <!-- ── Cabecera ──────────────────────────────────────────────────────
           Dos filas de altura fija, iguales en los dos estados. Antes la marca
           desaparecía al plegar y **todo lo de abajo subía**: los iconos de
           sección saltaban de y=131 a y=80 al pulsar el botón. Reservando la
           altura, plegar sólo cambia lo que hay dentro de la cabecera, nunca
           dónde empieza la navegación.

           El logotipo vive en un componente: el §21 prohíbe estilizarlo, y
           repetir su marcado en cada sitio es justo como se acaba estilizando
           en uno de ellos. Plegada, «ARLES RELAY» en Mont Black no entra en
           64 px, así que en su lugar va el **isotipo** —la misma «A» que el
           sistema operativo enseña en la barra de tareas—, no el logotipo
           recortado: el §21 prohíbe condensarlo. -->
      <div class="cabecera">
        <div class="marca">
          <ALogotipo
            v-if="!interfaz.plegada"
            numeral
          />
          <ALogotipo
            v-else
            tamano="icono"
          />
        </div>

        <div class="fila-plegador">
          <button
            type="button"
            class="plegador"
            :disabled="!interfaz.alternableAhora"
            :aria-expanded="!interfaz.plegada"
            aria-controls="navegacion-principal"
            :aria-label="etiquetaDelBoton"
            :title="explicacionDelBoton"
            @click="interfaz.alternar()"
          >
            <AIcono :nombre="interfaz.plegada ? 'expandir' : 'plegar'" />
          </button>
        </div>
      </div>

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
            <AIcono
              :nombre="ICONO_DE_SECCION[seccion]"
              :tamano="20"
            />
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
        v-if="CATALOGO_VISIBLE"
        class="nav-enlace enlace-de-revision"
        to="/catalogo"
        :title="interfaz.plegada ? 'Catálogo del sistema' : undefined"
      >
        <AIcono
          nombre="catalogo"
          :tamano="20"
        />
        <!-- Plegado sigue estando, con su icono. La primera versión lo
             escondía, y eso lo hacía desaparecer justo a partir del 200 % de
             escala —donde la barra se pliega sola—, que es exactamente la
             condición en la que hay que abrir el catálogo para revisarlo. -->
        <span :class="interfaz.plegada ? 'solo-lectores' : 'nav-texto'">
          Catálogo del sistema
        </span>
      </RouterLink>

      <!-- §121: la atribución vive aquí, discreta. NUNCA en los correos
           que el cliente envía.

           El logotipo abre telemetrymx.com **en el navegador del sistema**, no
           dentro de la ventana: una WebView que navega a internet deja de ser
           una aplicación y pasa a ser un navegador sin barra de direcciones,
           donde el usuario no puede saber dónde está. Ver `abrirSitio`. -->
      <footer class="pie">
        <button
          type="button"
          class="sello"
          :aria-label="$t('producto.irAlSitio')"
          :title="$t('producto.irAlSitio')"
          @click="abrirSitio"
        >
          <!-- Plegada cabe el símbolo pero no «TELEMETRY INSIGHT»: el pie
               desaparecía entero y la barra se quedaba sin ninguna marca.
               Son dos piezas distintas, no una recortada con `overflow`:
               recortar dejaría la «T» partida asomando por el borde. -->
          <span
            class="logo-telemetry"
            :class="interfaz.plegada ? 'solo-simbolo' : 'horizontal'"
          />
        </button>
        <p
          v-if="!interfaz.plegada"
          class="version"
        >
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

/* ── Cabecera de altura constante ───────────────────────────────────────────
   La marca y el botón ocupan lo mismo plegada que desplegada. Es lo que impide
   que los iconos de sección salten al plegar: medido, saltaban de y=131 a
   y=80 porque la marca desaparecía y arrastraba todo hacia arriba. */
.cabecera {
  width: 100%;
  flex: none;
}

.marca {
  display: flex;
  align-items: center;
  height: var(--arles-isotipo);
  padding-inline: var(--arles-space-2);
}

.plegado .marca {
  justify-content: center;
  padding-inline: 0;
}

/* El botón vive en su propia fila y **siempre en el mismo sitio**: pegado al
   borde donde está la barra. Antes cambiaba de alineación al plegar —de
   `flex-end` a `center`— y el usuario tenía que buscarlo dos veces. */
.fila-plegador {
  display: flex;
  justify-content: flex-end;
  height: var(--arles-control-height-compact);
  margin: var(--arles-space-3) 0 var(--arles-space-4);
}

.plegado .fila-plegador {
  justify-content: center;
}

.plegador {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--arles-control-height-compact);
  height: var(--arles-control-height-compact);
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
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--arles-space-3);
  /* Alto explícito: con `padding` y un icono de 20 px, la fila medía 36 px
     desplegada y 34 plegada, y la lista entera se descuadraba al plegar. */
  min-height: var(--arles-control-height);
  padding: 0 var(--arles-space-3);
  border-radius: var(--arles-radius-md);
  color: var(--arles-text-muted);
  text-decoration: none;
  font-weight: var(--arles-font-weight-semibold);
  transition:
    background var(--arles-duration-normal) var(--arles-ease),
    color var(--arles-duration-normal) var(--arles-ease);
}

/* ── La barra de acento de la sección activa ────────────────────────────────
   Es un pseudoelemento y no una sombra interior porque una sombra no se puede
   animar por altura. Aquí crece desde el centro al entrar en la sección y se
   recoge al salir — los dos a la vez, porque la clase se quita de un enlace y
   se pone en otro en el mismo instante.

   El movimiento tiene función (§98): dice **a dónde se fue** el estado. Una
   animación decorativa en una barra de navegación se vuelve ruido a la tercera
   vez que navegas; ésta dura 200 ms y sólo ocurre al cambiar de sección.

   `prefers-reduced-motion` la anula, como todo lo demás: la regla global de
   `base.css` reduce la transición a 0.01 ms, así que el estado final es el
   mismo y sólo desaparece el recorrido. */
.nav-enlace::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  width: var(--arles-space-1);
  height: 0;
  transform: translateY(-50%);
  border-radius: 0 var(--arles-radius-sm) var(--arles-radius-sm) 0;
  background: var(--arles-accent-ink);
  transition: height var(--arles-duration-normal) var(--arles-ease);
}

.nav-enlace.router-link-active::before {
  height: 60%;
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

/* La sección activa se marca con relleno Y con barra: el color solo no basta
   (§19, COLOR_SYSTEM.md §7.3).

   El relleno es `--arles-nav-activo` y no `--arles-surface-raised`. Con la
   superficie elevada, en tema claro el activo quedaba en blanco puro sobre una
   barra casi blanca —**1.06:1**— y el relleno no aportaba nada: el estado se
   sostenía sólo en la barra de acento. Ahora da 1.80, prácticamente lo mismo
   que el 1.76 del tema oscuro. */
.nav-enlace.router-link-active {
  background: var(--arles-nav-activo);
  color: var(--arles-text);
}

/* Se distingue de la navegación real: no es una sección del producto, es una
   herramienta de revisión que no viaja al cliente. */
.enlace-de-revision {
  margin-bottom: var(--arles-space-3);
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-disabled);
  border: var(--arles-border-width) dashed var(--arles-border-strong);
}

/* ── Pie ────────────────────────────────────────────────────────────────────
   El logotipo de TELEMETRY y la versión comparten fila y **línea de base**.
   Antes iban en dos párrafos apilados a la izquierda, y la versión colgaba
   debajo de la atribución sin alinearse con nada. */
.pie {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--arles-space-3);
  padding: var(--arles-space-4) var(--arles-space-2) 0;
  border-top: var(--arles-border-width) solid var(--arles-border);
}

.plegado .pie {
  justify-content: center;
  padding-inline: 0;
}

.sello {
  display: block;
  padding: var(--arles-space-1);
  margin: calc(-1 * var(--arles-space-1));
  border: 0;
  background: transparent;
  border-radius: var(--arles-radius-sm);
  cursor: pointer;
  opacity: 0.8;
  transition: opacity var(--arles-duration-fast) var(--arles-ease);
}

.sello:hover {
  opacity: 1;
}

/* La pieza llega a sangre —la tinta toca los cuatro bordes— así que el aire lo
   pone el botón, no el archivo (MARCA_TELEMETRY.md §4).

   **Va como máscara, no como imagen.** El logotipo es de un solo color sobre
   transparencia, así que el color puede venir del token y seguir al tema solo.
   La primera versión traía las dos piezas teñidas y elegía con una regla de
   CSS; esa regla **se descartó al compilar** y el pie se quedó con la tinta
   crema sobre papel claro, casi invisible. Con máscara no hay regla que
   descartar: sólo hay una pieza y su color es el del texto. */
.logo-telemetry {
  display: block;
  background-color: var(--arles-text-muted);
  mask-size: contain;
  mask-repeat: no-repeat;
  -webkit-mask-size: contain;
  -webkit-mask-repeat: no-repeat;
}

/* Proporción 3.108:1: se fija el ancho y el alto sale solo. */
.horizontal {
  width: 116px;
  height: 37px;
  mask-image: url('./activos/marca/telemetry-horizontal-mascara.png');
  -webkit-mask-image: url('./activos/marca/telemetry-horizontal-mascara.png');
}

/* Proporción 149:160, la del símbolo solo. */
.solo-simbolo {
  width: 30px;
  height: 32px;
  mask-image: url('./activos/marca/telemetry-isotipo-mascara.png');
  -webkit-mask-image: url('./activos/marca/telemetry-isotipo-mascara.png');
}

.version {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  opacity: 0.75;
}

.contenido {
  padding: var(--arles-space-6);
  overflow: auto;
}
</style>
