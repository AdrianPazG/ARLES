<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  watchEffect,
} from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink, RouterView, useRoute } from 'vue-router'

import { invocar } from '@/app/nucleo'
import {
  CATALOGO_VISIBLE,
  ICONO_DE_SECCION,
  SECCIONES,
  esSeccion,
  subseccionesDe,
} from '@/app/router'
import { useAppStore } from '@/app/stores/app'
import { useInterfazStore } from '@/app/stores/interfaz'
import { AIcono, ALogotipo } from '@/design/componentes'

const app = useAppStore()
const interfaz = useInterfazStore()
const ruta = useRoute()
const { t } = useI18n()

/**
 * En qué sección estamos, según lo que declara la ruta.
 *
 * NO se usa `router-link-active` para esto: ver la nota larga de `router.ts`.
 * En resumen, esa clase no llega a `/inicio` estando en
 * `/inicio/configuracion`, porque son rutas hermanas y no madre e hija — y por
 * eso la barra se quedaba apagada entera en tres pantallas.
 */
const seccionActual = computed(() => {
  const s = ruta.meta['seccion']
  return esSeccion(s) ? s : undefined
})

/** La subsección abierta, si la hay. `undefined` en las pantallas raíz. */
const subseccionActual = computed(() => {
  const s = ruta.meta['subseccion']
  return typeof s === 'string' ? s : undefined
})

/**
 * Las subsecciones que se despliegan en la barra: **sólo las de la sección en
 * la que estamos**, nunca las seis a la vez.
 *
 * Una barra que enseña todas las hijas de todas las secciones deja de ser una
 * brújula y pasa a ser un índice. Se despliegan al entrar y se recogen al
 * salir, que es cuando dejan de significar algo.
 */
const subseccionesVisibles = computed(() =>
  seccionActual.value ? subseccionesDe(seccionActual.value) : [],
)

/**
 * Las migas: «Inicio › Configuración».
 *
 * Existen **además** de la marca en la barra y no en su lugar. Motivo medido:
 * plegada, la barra sólo enseña iconos (P-11), así que las subentradas no se
 * dibujan y el único indicativo de subsección que queda es éste. Si las migas
 * fueran el único indicador, al plegar no se perdería nada; si la barra fuera
 * el único, al plegar se perdería todo.
 */
const migas = computed(() => {
  if (!seccionActual.value || !subseccionActual.value) return null
  return {
    seccion: seccionActual.value,
    rutaDeSeccion: `/${seccionActual.value}`,
    subseccion: subseccionActual.value,
  }
})

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

/* ── El indicador de sección que viaja ──────────────────────────────────────
 *
 * Una sola pieza de oro que se DESLIZA de una sección a otra, en vez de seis
 * barritas que se encienden y se apagan. Es lo que Dirección pidió el
 * 22/09/2026 con «dinamismo entre pestañas», y es la única pieza de la
 * interfaz a la que se le permite un movimiento con inercia.
 *
 * Por qué una sola pieza y no seis: seis marcas que parpadean dicen «este
 * estado se apagó y aquel otro se encendió». Una que viaja dice «eres tú quien
 * se ha movido, y de aquí a allí». En un producto cuyo trabajo es que sepas
 * exactamente dónde estás antes de mandar correos a gente real, esa diferencia
 * no es decorativa.
 *
 * ────────────────────────────────────────────────────────────────────────
 * Y AHORA LO IMPORTANTE: ESTO NO PUEDE ROMPER NADA
 *
 * Es la única parte de la navegación que depende de MEDIR el documento, y
 * medir falla de maneras que no avisan: la fuente todavía no cargó, la barra
 * está a mitad de plegarse, la ventana cambió de tamaño, el enlace activo aún
 * no existe porque la ruta acaba de cambiar.
 *
 * Así que el indicador es **un adorno encima de un estado que ya está dicho de
 * otras dos formas**: el relleno de la sección activa y `aria-current`. Si la
 * medida falla, `visible` se queda en `false`, el indicador no se dibuja, y la
 * barra sigue diciendo exactamente lo mismo que decía antes de que esto
 * existiera. Nunca al revés.
 *
 * Por eso tampoco se quitó la barrita de cada enlace: se le quitó el ORO (pasa
 * a ser el viajero quien lo lleva) pero sigue ahí como respaldo geométrico.
 * ──────────────────────────────────────────────────────────────────────── */
const listaDeNav = ref<HTMLElement | null>(null)
const indicador = ref({ y: 0, alto: 0, visible: false })

function recolocarIndicador(): void {
  const lista = listaDeNav.value
  if (!lista) {
    indicador.value = { ...indicador.value, visible: false }
    return
  }
  // El enlace de SECCIÓN, no el de subsección: el viajero marca en qué sección
  // estás. La subsección la marcan la sangría y su propio filete.
  const activo = lista.querySelector('.nav-enlace.activo')
  if (!activo) {
    indicador.value = { ...indicador.value, visible: false }
    return
  }
  const caja = activo.getBoundingClientRect()
  const marco = lista.getBoundingClientRect()
  // Una caja de altura cero significa que el navegador todavía no ha maquetado
  // —pasa entre la carga de la fuente y el primer pintado—. Medir ahí daría un
  // indicador pegado al borde superior; se prefiere no dibujarlo.
  if (caja.height === 0) {
    indicador.value = { ...indicador.value, visible: false }
    return
  }
  indicador.value = {
    y: caja.top - marco.top,
    alto: caja.height,
    visible: true,
  }
}

/** Recoloca en cuanto Vue haya pintado el cambio, no antes. */
function recolocarTrasPintar(): void {
  void nextTick(recolocarIndicador)
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

/* Todo lo que puede mover el enlace activo de sitio, y por qué cada uno:
 *
 *  · la ruta            — es el caso normal: cambias de sección.
 *  · el plegado         — la barra cambia de ancho y los enlaces de alto.
 *  · las subsecciones   — al abrirse empujan hacia abajo las secciones de
 *                         debajo, así que el viajero tiene que recolocarse
 *                         aunque la sección activa no haya cambiado.
 *  · el ancho de ventana — `medir()` puede plegar la barra sola (P-11).
 *
 * Los cuatro pasan por `nextTick`: medir antes de que Vue haya pintado el
 * cambio devuelve la posición ANTERIOR, y el indicador se queda una sección
 * por detrás. Es el defecto clásico de este patrón. */
watch(
  () => [seccionActual.value, interfaz.plegada, subseccionesVisibles.value.length],
  recolocarTrasPintar,
)

onMounted(() => {
  void app.cargar()
  void interfaz.cargar()
  medir()
  window.addEventListener('resize', medir)
  window.addEventListener('resize', recolocarIndicador)
  recolocarTrasPintar()
  /* Mont llega como woff2 incrustado y cambia la altura de los enlaces al
     sustituir a la fuente de reserva. Sin esto, el indicador nace medido
     contra la métrica equivocada y se queda descuadrado hasta que navegas.
     `document.fonts` no existe en jsdom, de ahí la guarda. */
  if (typeof document !== 'undefined' && document.fonts) {
    void document.fonts.ready.then(recolocarIndicador)
  }
  if (consultaDelSistema) {
    anotarTema(consultaDelSistema)
    consultaDelSistema.addEventListener('change', anotarTema)
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', medir)
  window.removeEventListener('resize', recolocarIndicador)
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

      <!-- ── El marco del viajero ────────────────────────────────────────────
           Un `<div>` alrededor de la lista, y el indicador DENTRO del div,
           nunca dentro del `<ul>`.

           La primera versión lo metía como primer hijo del `<ul>`, y eso hizo
           dos daños a la vez. Uno: un `<ul>` sólo admite `<li>`; un `<span>`
           suelto ahí es marcado inválido que los navegadores toleran. Dos, y
           es el que dolió: `sonda:cabecera` busca
           `.nav-lista li:first-child .nav-enlace` para medir el recorrido del
           teclado, y con el span delante **ningún `li` era ya el primer
           hijo**. La sonda se quedó esperando treinta segundos a un elemento
           que existía y que ella ya no podía nombrar.

           Nada había fallado en la pantalla. Lo que se rompió fue la capacidad
           de comprobarla, que es peor, porque no se nota mirando. -->
      <div
        ref="listaDeNav"
        class="nav-marco"
        :class="{ 'sin-viajero': !indicador.visible }"
      >
        <!-- El viajero. `aria-hidden` porque no dice nada que no digan ya el
             relleno del enlace activo y su `aria-current`: para un lector de
             pantalla sería una tercera voz repitiendo lo mismo. -->
        <span
          v-show="indicador.visible"
          class="nav-indicador"
          aria-hidden="true"
          :style="{
            transform: `translateY(${indicador.y}px)`,
            height: `${indicador.alto}px`,
          }"
        />
        <ul class="nav-lista">
          <li
            v-for="seccion in SECCIONES"
            :key="seccion"
          >
            <RouterLink
              class="nav-enlace"
              :class="{ activo: seccion === seccionActual }"
              :to="`/${seccion}`"
              :title="interfaz.plegada ? $t(`nav.${seccion}`) : undefined"
              :aria-current="
                seccion === seccionActual && !subseccionActual ? 'page' : undefined
              "
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

            <!-- ── Las subsecciones de la sección abierta ────────────────────
               Sólo cuelgan de la sección en la que estamos, y **no se dibujan
               plegada**: ahí la barra es de iconos (P-11), y un texto indentado
               sin icono no tiene dónde ponerse. El indicativo no se pierde,
               se lo quedan las migas de arriba. -->
            <ul
              v-if="seccion === seccionActual && !interfaz.plegada && subseccionesVisibles.length"
              class="sub-lista"
            >
              <li
                v-for="sub in subseccionesVisibles"
                :key="sub.ruta"
              >
                <RouterLink
                  class="sub-enlace"
                  :class="{ activo: sub.clave === `nav.sub.${subseccionActual}` }"
                  :to="sub.ruta"
                  :aria-current="
                    sub.clave === `nav.sub.${subseccionActual}` ? 'page' : undefined
                  "
                >
                  {{ $t(sub.clave) }}
                </RouterLink>
              </li>
            </ul>
          </li>
        </ul>
      </div>

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
        :class="{ activo: ruta.path === '/catalogo' }"
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
      <!-- ── Migas ──────────────────────────────────────────────────────────
           Sólo aparecen dentro de una subsección. En una pantalla raíz serían
           una miga sola —«Inicio»— que no dice a dónde volver ni de dónde se
           viene: ocuparía una línea para no informar de nada.

           `<nav>` con su propio nombre y `aria-current="page"` en la hoja: un
           lector de pantalla anuncia «Inicio, enlace · Configuración, página
           actual» en vez de leer dos palabras sueltas sobre el título. -->
      <nav
        v-if="migas"
        class="migas"
        :aria-label="$t('nav.migas')"
      >
        <ol>
          <li>
            <RouterLink :to="migas.rutaDeSeccion">
              {{ $t(`nav.${migas.seccion}`) }}
            </RouterLink>
          </li>
          <li aria-current="page">
            {{ $t(`nav.sub.${migas.subseccion}`) }}
          </li>
        </ol>
      </nav>

      <!-- ── La entrada de pantalla ──────────────────────────────────────────
           Cada pantalla llega subiendo seis píxeles y apareciendo, en 200 ms.
           Es lo que convierte «la ventana ha cambiado de contenido» en «he ido
           a otro sitio»: sin ella, navegar es un parpadeo y el ojo tiene que
           releer la pantalla entera para saber si pasó algo.

           `mode="out-in"` porque las dos pantallas no pueden cruzarse: si la
           saliente y la entrante se solapan, durante 200 ms hay dos `h1` y dos
           formularios en el documento, y un lector de pantalla los anuncia los
           dos. Con `out-in` sólo hay una cada vez.

           Sólo `opacity` y `transform`: las dos las compone la tarjeta gráfica
           sin recalcular maquetación, así que esto cuesta lo mismo con una
           tabla de 500 000 contactos que con una pantalla vacía (T-7). Un
           `height` o un `margin` animados aquí habrían sido justo lo contrario.

           El recorrido es de 6 px, no de 20. Una pantalla que llega desde lejos
           se lee como una aplicación que quiere que la miren; ésta es una mesa
           de trabajo, y lo que tiene que transmitir el movimiento es que el
           cambio fue intencionado, no que el cambio es un espectáculo. -->
      <RouterView v-slot="{ Component }">
        <Transition
          name="pantalla"
          mode="out-in"
        >
          <component :is="Component" />
        </Transition>
      </RouterView>
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

/* La barra es la pieza que está en las diez pantallas, así que es la que
   decide si el producto se siente hecho a mano o montado con piezas sueltas.
   Lleva veladura: aclara arriba, donde está la marca, y ahonda hacia el pie.
   El color sólido no se toca — ver la nota de las veladuras en `base.css`. */
.nav {
  display: flex;
  flex-direction: column;
  background-color: var(--arles-surface);
  background-image: var(--arles-veladura-superficie);
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
  width: 100%;
}

/* El marco contra el que se mide el viajero, y el que ocupa el alto sobrante
   de la barra. La lista vuelve a ser una lista y nada más. */
.nav-marco {
  position: relative;
  flex: 1;
  width: 100%;
}

/* ── El viajero ─────────────────────────────────────────────────────────────
   Una sola pieza de oro que se desliza entre secciones. Ver la nota larga del
   bloque `<script>`; aquí sólo lo que es propio del dibujo.

   Va DEBAJO de los enlaces (`z-index: 0` contra el `1` de `.nav-enlace`), no
   encima: tiene que quedar detrás del texto y del icono, como el papel de una
   pestaña física queda detrás de lo que hay impreso en ella.

   Se anima `transform` y `height`, y nada más. `transform` no toca la
   maquetación ni provoca repintado del resto de la barra, así que el
   deslizamiento no puede tirar del rendimiento por muchas veces que se
   navegue. `height` sí la toca, pero sólo dentro de este elemento absoluto,
   que no tiene hermanos en flujo.

   No lleva `transition` de opacidad a propósito: aparece y desaparece de
   golpe con `v-show`. Un desvanecido lo dejaría medio dibujado en el instante
   en que la medida falla, que es justo cuando no queremos verlo. */
.nav-indicador {
  position: absolute;
  left: 0;
  top: 0;
  z-index: 0;
  width: 100%;
  border-radius: var(--arles-radius-md);
  background-color: var(--arles-nav-activo);
  background-image: var(--arles-veladura-tarjeta);
  /* El filete de oro del borde izquierdo, que es lo que de verdad se sigue con
     la vista mientras viaja. Va como sombra interior y no como pseudoelemento
     porque un pseudoelemento con su propia transición llegaría tarde: éste se
     mueve pegado a la pieza, con ella, sin un segundo cronómetro. */
  box-shadow: inset var(--arles-space-1) 0 0 0 var(--arles-accent-ink);
  transition:
    transform var(--arles-duration-normal) var(--arles-ease-resorte),
    height var(--arles-duration-normal) var(--arles-ease-resorte);
}

.nav-enlace {
  position: relative;
  /* Por encima del viajero, que va en `z-index: 0`. El texto y el icono tienen
     que quedar sobre el oro, no debajo. */
  z-index: 1;
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

.nav-enlace.activo::before {
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
.nav-enlace.activo {
  background: var(--arles-nav-activo);
  color: var(--arles-text);
}

/* ── El reparto entre el viajero y el respaldo ──────────────────────────────
   Cuando el viajero está dibujado, ÉL es el relleno y él lleva el filete de
   oro: el enlace se aparta para no taparlo. Cuando no —porque la medida falló
   o porque todavía no se ha hecho—, la lista lleva `sin-viajero` y vuelve
   exactamente el dibujo anterior: relleno plano en el enlace y su barrita de
   acento.

   Escrito así, y no al revés, por un motivo concreto: el estado por defecto
   del CSS es el que funciona sin JavaScript. Lo que puede fallar es lo que
   tiene que pedir permiso para dibujarse, no lo que tiene que pedir permiso
   para desaparecer. */
.nav-marco:not(.sin-viajero) .nav-enlace.activo {
  background: transparent;
}

.nav-marco:not(.sin-viajero) .nav-enlace.activo::before {
  height: 0;
}

/* ── Subsecciones ───────────────────────────────────────────────────────────
   Indentadas hasta alinear con el TEXTO del padre, no con su icono: la sangría
   es lo que dice «esto cuelga de aquello», y alinearlas con el icono las
   dejaría pareciendo secciones más.

   Sin icono propio a propósito. Las seis secciones se reconocen por su icono
   incluso plegadas (P-11); darle uno a una subsección la pondría al mismo
   nivel visual que su madre. */
.sub-lista {
  list-style: none;
  margin: var(--arles-space-1) 0 var(--arles-space-2);
  padding: 0 0 0 calc(var(--arles-space-3) + 20px + var(--arles-space-3));
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  /* La línea vertical dibuja la JERARQUÍA —estas cuelgan de aquélla—, no el
     estado. Al escribirla creí que servía para las dos cosas; medirla lo
     deshizo: es un filete sobre `--arles-border` y no distingue cuál de las
     hijas está abierta. El estado lo lleva la barra de acento de abajo. */
  position: relative;
}

.sub-lista::before {
  content: '';
  position: absolute;
  left: calc(var(--arles-space-3) + 10px);
  top: 0;
  bottom: 0;
  width: var(--arles-border-width);
  background: var(--arles-border);
}

.sub-enlace {
  position: relative;
  display: flex;
  align-items: center;
  min-height: var(--arles-control-height-compact);
  padding: 0 var(--arles-space-3);
  border-radius: var(--arles-radius-sm);
  color: var(--arles-text-muted);
  text-decoration: none;
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  font-weight: var(--arles-font-weight-semibold);
  transition:
    background var(--arles-duration-normal) var(--arles-ease),
    color var(--arles-duration-normal) var(--arles-ease);
}

.sub-enlace:hover {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
}

/* ── El estado activo de una subsección NO puede vivir sólo en el color ─────
   Medido el 22/09/2026, auditando: el relleno `--arles-nav-activo` sobre la
   barra da **1.76:1** en oscuro y **1.80:1** en claro. Por debajo de 3:1, que
   es el mínimo de un elemento no textual.

   La sección madre aguanta ese relleno flojo porque **además** lleva la barra
   de acento: el relleno acompaña, no sostiene. La subsección no llevaba nada,
   así que su estado era color y nada más — justo lo que el §19 prohíbe.

   Y la línea vertical que las ata a la madre no lo resolvía: es de un filete
   sobre `--arles-border`, sirve para dibujar la jerarquía y es invisible como
   indicador de estado. Escribirla como «ancla sin color» en el comentario fue
   optimismo mío; medirla lo deshizo.

   Ahora lleva su propia barra, más corta que la de la madre para que se lea
   como lo que es: una hija dentro de la sección abierta, no otra sección. */
.sub-enlace::before {
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

.sub-enlace.activo::before {
  height: 45%;
}

/* Relleno Y barra, nunca sólo color (§19). */
.sub-enlace.activo {
  background: var(--arles-nav-activo);
  color: var(--arles-text);
}

/* ── Migas ──────────────────────────────────────────────────────────────────
   El separador es un pseudoelemento y no un carácter en la plantilla: escrito
   en el marcado, el lector de pantalla leería «Inicio, mayor que,
   Configuración». Aquí es decoración y no se anuncia. */
.migas {
  margin-bottom: var(--arles-space-4);
}

.migas ol {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--arles-space-2);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

.migas li {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  color: var(--arles-text-muted);
}

.migas li + li::before {
  content: '›';
  color: var(--arles-text-disabled);
}

.migas a {
  color: var(--arles-text-muted);
  text-decoration: none;
  border-radius: var(--arles-radius-sm);
}

.migas a:hover {
  color: var(--arles-text);
  text-decoration: underline;
}

/* La hoja es la página actual: se lee con el color normal para que se
   distinga del enlace de vuelta sin apoyarse sólo en el subrayado. */
.migas [aria-current='page'] {
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

/* ── La entrada de pantalla ─────────────────────────────────────────────────
   Ver la nota del `<Transition>` en la plantilla. Las dos mitades no duran lo
   mismo a propósito: la saliente se va en `fast` y la entrante llega en
   `normal`, con la curva de entrada, que frena mucho al final.

   Si las dos duraran igual, el cambio se sentiría como una sustitución. Así se
   siente como que una pantalla deja sitio y la otra se acomoda — que es lo que
   de verdad está pasando. */
.pantalla-enter-active {
  transition:
    opacity var(--arles-duration-normal) var(--arles-ease-entrada),
    transform var(--arles-duration-normal) var(--arles-ease-entrada);
}

.pantalla-leave-active {
  transition:
    opacity var(--arles-duration-fast) var(--arles-ease),
    transform var(--arles-duration-fast) var(--arles-ease);
}

.pantalla-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

/* La saliente NO se va hacia abajo: se queda donde está y sólo se apaga. Un
   movimiento de salida hacia el mismo lado del que llega la entrante haría que
   las dos parecieran ir en fila, y no van en fila: la una sustituye a la otra.
   Se probó con `translateY(-6px)` y lo que se lee es un carrusel. */
.pantalla-leave-to {
  opacity: 0;
}
</style>
