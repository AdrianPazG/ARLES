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
 * **El numeral «I» no es parte del logotipo.** El logotipo dice ARLES RELAY.
 * El «I» pertenece al **nombre comercial**, y por eso está detrás de una
 * propiedad aparte en lugar de escribirse dentro de la marca: quien pone
 * `numeral` está diciendo «aquí va el nombre comercial», no «aquí va el
 * logotipo con una letra más».
 *
 * La regla de ADR-0010 que sigue en pie: **el numeral nunca va adyacente al
 * número de versión.** «ARLES RELAY I v1.2.0» hace pensar que el «I» es la
 * versión 1. En la barra lateral el numeral está arriba y la versión en el
 * pie, con toda la navegación en medio.
 *
 * `tamano="icono"` dibuja el **isotipo**: la «A» sobre su placa, que es el
 * mismo icono que el sistema operativo enseña en la barra de tareas
 * (`herramientas/iconos/generar-iconos.py`). Sus dos colores no se invierten
 * con el tema, porque el icono de la barra de tareas tampoco lo hace.
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
    /** `nav` es el uso habitual, `display` la bienvenida, `icono` el isotipo. */
    tamano?: 'nav' | 'display' | 'icono'
    /** Añade el numeral «I» del nombre comercial. Nunca junto a la versión. */
    numeral?: boolean
  }>(),
  { tamano: 'nav', numeral: false },
)
</script>

<template>
  <!-- Un solo nodo de texto accesible: el lector de pantalla no debe leer
       «ARLES» y «RELAY» como dos cosas distintas. El nombre accesible es el
       mismo en el isotipo: quien no ve la placa oye la marca completa, no la
       letra «A» suelta. -->
  <span
    class="logotipo"
    :class="`t-${tamano}`"
    role="img"
    :aria-label="numeral ? 'ARLES RELAY I' : 'ARLES RELAY'"
  >
    <template v-if="tamano === 'icono'">
      <span aria-hidden="true">A</span>
    </template>
    <template v-else>
      <span aria-hidden="true">ARLES</span>
      <span
        class="relay"
        aria-hidden="true"
      >RELAY</span>
      <span
        v-if="numeral"
        class="numeral"
        aria-hidden="true"
      >I</span>
    </template>
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

/* El isotipo: la misma placa y la misma «A» que el icono del sistema
   operativo. El área de respeto de arriba no aplica —la placa ya es su propio
   margen— y por eso se anula. */
.t-icono {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--arles-isotipo);
  height: var(--arles-isotipo);
  padding: 0;
  margin: 0;
  border-radius: var(--arles-radius-lg);
  background: var(--arles-marca-fondo);
  color: var(--arles-marca-tinta);
  font-size: var(--arles-font-size-h2);
  /* Compensa el bearing óptico de la A en Mont Black, igual que el generador
     de iconos. Sin esto la letra se ve caída hacia la izquierda. */
  text-indent: 0.04em;
}

/* El numeral pesa menos que la marca: es el nombre comercial, no parte del
   logotipo. Mismo cuerpo, tinta apagada y aire propio. */
.numeral {
  margin-left: 0.5ch;
  color: var(--arles-text-muted);
  font-weight: var(--arles-font-weight-bold);
}

.relay {
  color: var(--arles-accent-ink);
  margin-left: 0.35ch;
}
</style>
