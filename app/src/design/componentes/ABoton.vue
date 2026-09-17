<script setup lang="ts">
/**
 * Botón (DESIGN_SYSTEM.md §6).
 *
 * Cuatro variantes. **El primario es uno por pantalla**: si hay dos, ninguno
 * de los dos dirige la atención a ningún sitio.
 *
 * El primario lleva texto oscuro sobre el acento: 11.68:1. Con texto blanco
 * sería 1.52:1 — ilegible (COLOR_SYSTEM.md §7.2). Por eso el color del texto
 * no es una propiedad: es una consecuencia de la variante.
 *
 * Altura mínima 32 px. El §22 pide escalado de Windows hasta el 200 % y un
 * objetivo más pequeño se vuelve impreciso mucho antes de llegar ahí.
 *
 * **`a` cambia el elemento, no sólo el destino.** Un control que navega tiene
 * que ser un enlace: un `<button>` con un `router.push` dentro pierde el menú
 * contextual, el foco anunciado como enlace y la posibilidad de saber a dónde
 * lleva antes de pulsarlo. El aspecto es el mismo; la semántica, no.
 */
import { RouterLink } from 'vue-router'

import AIcono, { type NombreDeIcono } from './AIcono.vue'

withDefaults(
  defineProps<{
    variante?: 'primario' | 'secundario' | 'sutil' | 'peligro'
    tamano?: 'compacto' | 'normal'
    tipo?: 'button' | 'submit'
    icono?: NombreDeIcono
    deshabilitado?: boolean
    /** Deshabilita y anuncia el trabajo en curso a los lectores de pantalla. */
    ocupado?: boolean
    /** Obligatorio si el botón sólo lleva icono. */
    etiqueta?: string
    /** Ruta a la que navega. Con esto el botón se dibuja como enlace. */
    a?: string
  }>(),
  {
    variante: 'secundario',
    tamano: 'normal',
    tipo: 'button',
    deshabilitado: false,
    ocupado: false,
  },
)
</script>

<template>
  <component
    :is="a ? RouterLink : 'button'"
    class="boton"
    :class="[`v-${variante}`, `t-${tamano}`]"
    :to="a"
    :type="a ? undefined : tipo"
    :disabled="a ? undefined : deshabilitado || ocupado"
    :aria-busy="ocupado || undefined"
    :aria-label="etiqueta"
  >
    <AIcono
      v-if="icono"
      :nombre="icono"
    />
    <span class="texto"><slot /></span>
  </component>
</template>

<style scoped>
.boton {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--arles-space-2);

  font-family: inherit;
  font-size: var(--arles-font-size-body);
  line-height: var(--arles-line-height-body);
  font-weight: var(--arles-font-weight-semibold);

  border: var(--arles-border-width) solid transparent;
  border-radius: var(--arles-radius-md);
  cursor: pointer;
  white-space: nowrap;

  /* Cuando el botón es un enlace hereda el subrayado y el color de enlace del
     documento, y dejaría de parecer un botón. */
  text-decoration: none;

  transition:
    background var(--arles-duration-fast) var(--arles-ease),
    border-color var(--arles-duration-fast) var(--arles-ease);
}

.t-normal {
  min-height: var(--arles-control-height);
  padding: 0 var(--arles-space-4);
}

.t-compacto {
  min-height: var(--arles-control-height-compact);
  padding: 0 var(--arles-space-3);
}

/* El texto vacío no debe dejar un hueco cuando el botón es sólo icono. */
.texto:empty {
  display: none;
}

.v-primario {
  background: var(--arles-accent);
  color: var(--arles-text-on-accent);
}

.v-primario:hover:not(:disabled) {
  background: var(--arles-accent-hover);
}

/* En el tema claro el oro sobre papel da 1.32:1: el botón se lee, pero como
   forma no existe. El borde es lo que le devuelve el contorno —5.47:1 contra
   la página— y lo que cumple el 1.4.11. En oscuro no hace falta: ahí el mismo
   oro ya recorta contra el fondo profundo. */
:root[data-tema='claro'] .v-primario {
  border-color: var(--arles-accent-ink);
}

.v-secundario {
  background: transparent;
  color: var(--arles-text);
  border-color: var(--arles-border-strong);
}

.v-secundario:hover:not(:disabled) {
  background: var(--arles-surface-hover);
}

.v-sutil {
  background: transparent;
  color: var(--arles-text-muted);
}

.v-sutil:hover:not(:disabled) {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
}

.v-peligro {
  background: var(--arles-danger);
  color: var(--arles-text-on-accent);
}

/* Token, no `filter: brightness()`: un filtro no se puede verificar contra
   WCAG, y el texto oscuro de encima tiene que pasar AA también en hover.
   #F4867A da 7.24:1 (herramientas/design-tokens). */
.v-peligro:hover:not(:disabled) {
  background: var(--arles-danger-hover);
}

/* ─────────────────────────────────────────────────────────────────────────
   DESHABILITADO — por qué NO es `opacity`

   Lo era, y se midió en el catálogo abierto en el navegador: un primario
   deshabilitado daba **1.89:1** y un secundario ocupado **3.56:1**. La
   etiqueta no se leía. WCAG exime a los controles deshabilitados del mínimo
   de contraste, pero un botón cuyo texto no se distingue no comunica *qué*
   está deshabilitado, y el §95 pide justo lo contrario.

   Así que el estado se pinta, no se atenúa: relleno neutro y
   `--arles-text-disabled`, iguales para las cuatro variantes —un
   deshabilitado no tiene jerarquía que conservar— más el cursor, que es la
   segunda señal (7.3). El token da 3.64:1 sobre `surface-raised`, verificado
   en CI: por encima del suelo de 3.0 y claramente más apagado que el texto
   secundario, que ahí da 6.07.
   ───────────────────────────────────────────────────────────────────────── */
.boton:disabled {
  background: var(--arles-surface-raised);
  color: var(--arles-text-disabled);
  border-color: var(--arles-border);
  cursor: not-allowed;
}

/* «Ocupado» NO se atenúa. No está deshabilitado por una regla de negocio: está
   trabajando, y el usuario está mirándolo precisamente ahora. Conserva su
   contraste completo y sólo deja de aceptar clics. */
.boton[aria-busy='true']:disabled {
  background: inherit;
  color: inherit;
  border-color: inherit;
  cursor: progress;
}

.v-primario[aria-busy='true']:disabled {
  background: var(--arles-accent);
  color: var(--arles-text-on-accent);
}

.v-peligro[aria-busy='true']:disabled {
  background: var(--arles-danger);
  color: var(--arles-text-on-accent);
}

.v-secundario[aria-busy='true']:disabled {
  background: transparent;
  color: var(--arles-text);
  border-color: var(--arles-border-strong);
}

.v-sutil[aria-busy='true']:disabled {
  background: transparent;
  color: var(--arles-text-muted);
}
</style>
