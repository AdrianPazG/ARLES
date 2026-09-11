<script setup lang="ts">
/**
 * Logotipo de ARLES (§21 · TIPOGRAFIA.md §5).
 *
 * **Exclusivamente tipográfico: «ARLES RELAY» en Mont Black. Sin isotipo.**
 *
 * Prohibido por el §21: estilizar letras, estirar, condensar, degradados,
 * efectos 3D, rotación y sombras. Este componente no expone ninguna propiedad
 * que permita hacer nada de eso; el único grado de libertad es el tamaño.
 *
 * **El numeral «I» no aparece aquí.** El logotipo dice ARLES RELAY. El «I»
 * pertenece al nombre comercial, vive en el empaque y en «Acerca de», y nunca
 * va adyacente al número de versión (ADR-0010).
 *
 * Por qué texto y no un SVG con contornos: mientras P-01 siga abierta la
 * fuente no sale del equipo de desarrollo, y aquí el texto es además
 * seleccionable y accesible. Si P-01 se cierra en negativo, este componente
 * es el único punto donde hay que sustituir el texto por el trazado —que sí
 * es legítimo bajo licencia de escritorio, porque el archivo resultante es
 * geometría, no la fuente (TIPOGRAFIA.md §3).
 */
withDefaults(
  defineProps<{
    /** Altura tipográfica. `nav` es el uso habitual; `display` la bienvenida. */
    tamano?: 'nav' | 'display'
  }>(),
  { tamano: 'nav' },
)
</script>

<template>
  <!-- Un solo nodo de texto accesible: el lector de pantalla no debe leer
       «ARLES» y «RELAY» como dos cosas distintas. -->
  <span
    class="logotipo"
    :class="`t-${tamano}`"
    role="img"
    aria-label="ARLES RELAY"
  >
    <span aria-hidden="true">ARLES</span>
    <span
      class="relay"
      aria-hidden="true"
    >RELAY</span>
  </span>
</template>

<style scoped>
.logotipo {
  display: inline-block;
  font-weight: var(--arles-font-weight-black);
  letter-spacing: 0.02em;
  line-height: 1.2;
  white-space: nowrap;

  /* Área de respeto: la altura de la «A» por cada lado (TIPOGRAFIA.md §5).
     En Mont Black la mayúscula mide ~0.72em. */
  padding: 0.72em;
  margin: -0.72em;
}

.t-nav {
  font-size: var(--arles-font-size-h2);
}

.t-display {
  font-size: var(--arles-font-size-display);
}

.relay {
  color: var(--arles-accent);
  margin-left: 0.35ch;
}
</style>
