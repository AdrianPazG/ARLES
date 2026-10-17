<script setup lang="ts">
/**
 * Módulo: el bloque con el que se arma un panel.
 *
 * Dirección pidió que Inicio dejara de ser una lista larga y pasara a ser
 * modular, con acciones a mano. Esta es la pieza.
 *
 * ──────────────────────────────────────────────────────────────────────────
 * LA REGLA QUE HACE QUE UN PANEL MODULAR NO SE VEA ROTO
 *
 * Un módulo **sólo se dibuja cuando puede decir algo cierto**. La tentación
 * del panel modular es maquetar hoy las ocho cajas que habrá algún día y
 * dejarlas vacías esperando datos; el resultado es una pantalla que parece
 * estropeada y que además enseña capacidades que no existen.
 *
 * Aquí un módulo que no tiene nada que decir **no se monta**. La decisión de
 * si hay algo que decir es de la pantalla, no del componente: el componente
 * sólo garantiza que, si se monta, tiene título y cuerpo.
 * ──────────────────────────────────────────────────────────────────────────
 *
 * `nota` es para una cifra verificable —«1 de 6», «35 al día»—, nunca para un
 * adjetivo (§94). Va arriba a la derecha, que es donde el ojo la busca después
 * de leer el título.
 *
 * `principal` lo reserva la pantalla para **un solo módulo**: el que contiene
 * la acción que se espera que el usuario haga ahora. Si dos módulos compiten
 * por ser el principal, ninguno lo es.
 */
withDefaults(
  defineProps<{
    titulo: string
    /** Cifra verificable, arriba a la derecha. Nunca un adjetivo. */
    nota?: string
    /** Uno por pantalla: el que lleva la acción esperada. */
    principal?: boolean
    /**
     * El nivel del encabezado. `h2` salvo que este módulo **sea** el título de
     * su pantalla.
     *
     * ── POR QUÉ HIZO FALTA ────────────────────────────────────────────────
     *
     * Tres pantallas de catorce —Remitentes, Bajas y Derechos— no tienen
     * cabecera propia: son un módulo y ya. Como el módulo siempre pintaba un
     * `h2`, esas tres **no tenían ningún `h1`**. Para quien usa un lector de
     * pantalla, «ir al encabezado 1» no llevaba a ninguna parte.
     *
     * La sonda decía vigilar la jerarquía de encabezados y no lo veía: medía
     * los saltos *entre* encabezados y nunca comprobaba por cuál empieza, así
     * que un `h2` inicial sin `h1` pasaba en verde. Encontrado mirando, el
     * 01/10/2026 en la 7.7.
     */
    nivel?: 'h1' | 'h2'
  }>(),
  { principal: false, nivel: 'h2' },
)
</script>

<template>
  <section
    class="modulo"
    :class="{ principal }"
  >
    <header class="cabecera">
      <component
        :is="nivel"
        class="titulo"
        :class="{ 'titulo--pagina': nivel === 'h1' }"
      >
        {{ titulo }}
      </component>
      <p
        v-if="nota"
        class="nota"
      >
        {{ nota }}
      </p>
    </header>

    <div class="cuerpo">
      <slot />
    </div>

    <!-- Las acciones van al final y no flotando en el cuerpo: un módulo se lee
         de arriba abajo y termina en lo que se puede hacer con él. -->
    <footer
      v-if="$slots.acciones"
      class="acciones"
    >
      <slot name="acciones" />
    </footer>
  </section>
</template>

<style scoped>
.modulo {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  padding: var(--arles-space-5);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-lg);
  /* Veladura sobre el color sólido: la tarjeta deja de ser un rectángulo
     plano y pasa a tener un canto de luz arriba y peso abajo. El
     `background-color` es el que sigue midiendo el contraste de todo lo que
     lleve dentro — ver la nota de las veladuras en `base.css`.

     Es deliberadamente la más floja de las tres. Una pantalla puede tener seis
     módulos, y seis degradados marcados convierten una página en un muestrario
     de tarjetas. Aquí el degradado no se ve: se nota. */
  background-color: var(--arles-surface);
  background-image: var(--arles-veladura-tarjeta);
}

/* El módulo principal se distingue por el borde de acento y no por un fondo
   distinto: cambiar el fondo obliga a volver a medir el contraste de todo lo
   que lleve dentro, y el borde no. */
.modulo.principal {
  border-color: var(--arles-accent-ink);
  border-left-width: var(--arles-space-1);
}

/* ── DOS CAMPOS SEGUIDOS NECESITAN MÁS AIRE QUE EL RESTO (H-09) ──────────
   Dirección leyó el paso 5 del asistente el 16/10/2026 y dijo que el hueco
   entre «Déjalo en blanco para usar el de la cuenta» y «Máximo por hora» era
   «muy reducido»: las dos líneas se leían como una sola.

   La causa es que la ayuda de un campo y la etiqueta del siguiente son **las
   dos texto pequeño**, y entre ellas había los 16 px del módulo mientras
   dentro del campo hay 4. Cuatro veces más no basta cuando lo que separa y lo
   que junta se parecen tanto.

   Se añade sólo ENTRE CAMPOS, no al `gap` del módulo: subir ése alejaría
   también el título de su primer campo, y lo que Dirección pidió fue
   «equilibrado, ni amontonado ni con demasiado espacio». 16 + 8 = 24 px entre
   campos, seis veces lo de dentro, y el título donde estaba. */
.modulo :deep(.campo + .campo) {
  margin-top: var(--arles-space-2);
}

.cabecera {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--arles-space-4);
}

.titulo {
  margin: 0;
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
  font-weight: var(--arles-font-weight-semibold);
}

/* Cuando el módulo ES el título de su pantalla, se lee como el de las otras
   once: mismo tamaño y mismo peso. Cambiar sólo la etiqueta y dejarlo pequeño
   arreglaría la sonda sin arreglar lo que se ve, que es peor que no tocarlo.

   ── EL SELECTOR LLEVA `.modulo` A PROPÓSITO ──

   Escrito como `.titulo--pagina` a secas **no se aplicaba**: `.modulo.principal
   .titulo` pesa (0,2,0) contra (0,1,0), y los tres módulos que llevan el título
   de pantalla son justo los `principal`. La etiqueta cambiaba a `h1`, la sonda
   se ponía verde y en pantalla se seguía viendo igual de pequeño — una
   comprobación satisfecha sin arreglar el defecto. Se vio **mirando la captura**
   después de que la sonda diera verde, que es exactamente la lección del A3. */
.modulo.principal .titulo {
  font-size: var(--arles-font-size-h2);
  line-height: var(--arles-line-height-h2);
}

/* ── TRES INTENTOS HASTA QUE ESTE SELECTOR GANÓ, Y MERECE ESTAR ESCRITO ──

   Hay que vencer a `.modulo.principal .titulo`, que es justo la que llevan los
   tres módulos que hacen de título de pantalla.

   1. `.titulo--pagina` · pierde por peso (0,1,0 contra 0,2,0).
   2. `.modulo .titulo--pagina` puesto ANTES · pierde por orden… o eso parecía.
   3. `.modulo .titulo--pagina` puesto DESPUÉS · **también perdía**, y aquí
      estaba lo que no había mirado: `<style scoped>` añade el atributo
      `[data-v-…]` **sólo al último selector**. Así que el de `principal` pesa
      `.modulo.principal` + `.titulo[attr]` = cuatro, y el mío sólo tres. No era
      cuestión de orden en ningún momento.

   La forma de abajo lleva las dos clases juntas en el último selector: cuatro
   también, y como va después, gana — sin atarse a `principal`, para que sirva
   igual en un módulo que no lo sea.

   Las tres veces la sonda dio verde con el `h1` ya puesto, y las tres veces la
   captura seguía enseñando el título pequeño. Es el A3 otra vez: **medir no es
   mirar**. */
.modulo .titulo.titulo--pagina {
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.nota {
  margin: 0;
  flex: none;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  font-variant-numeric: tabular-nums;
}

.cuerpo {
  flex: 1;
  min-width: 0;
}

.acciones {
  display: flex;
  flex-wrap: wrap;
  gap: var(--arles-space-3);
  align-items: center;
}
</style>
