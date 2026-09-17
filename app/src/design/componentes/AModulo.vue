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
  }>(),
  { principal: false },
)
</script>

<template>
  <section
    class="modulo"
    :class="{ principal }"
  >
    <header class="cabecera">
      <h2 class="titulo">
        {{ titulo }}
      </h2>
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
  background: var(--arles-surface);
}

/* El módulo principal se distingue por el borde de acento y no por un fondo
   distinto: cambiar el fondo obliga a volver a medir el contraste de todo lo
   que lleve dentro, y el borde no. */
.modulo.principal {
  border-color: var(--arles-accent-ink);
  border-left-width: var(--arles-space-1);
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

.modulo.principal .titulo {
  font-size: var(--arles-font-size-h2);
  line-height: var(--arles-line-height-h2);
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
