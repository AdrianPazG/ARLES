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
    border-color var(--arles-duration-fast) var(--arles-ease),
    transform var(--arles-duration-fast) var(--arles-ease);
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

/* El oro va con veladura: un relleno plano se lee como una etiqueta de color,
   y con la veladura se lee como una pieza de metal con luz encima. Es el único
   sitio del producto donde el acento ocupa una superficie entera, así que es
   donde ese medio tono se nota.

   `background-color` aparte de `background-image` no es manía: es lo que deja
   que el contraste se siga midiendo contra un color real (`base.css`). */
.v-primario {
  background-color: var(--arles-accent);
  background-image: var(--arles-veladura-acento);
  color: var(--arles-text-on-accent);
}

.v-primario:hover:not(:disabled) {
  background-color: var(--arles-accent-hover);
}

/* En el tema claro el oro sobre papel da 1.32:1: el botón se lee, pero como
   forma no existe. El borde es lo que le devuelve el contorno —5.47:1 contra
   la página— y lo que cumple el 1.4.11. En oscuro no hace falta: ahí el mismo
   oro ya recorta contra el fondo profundo.

   El `:not(:disabled)` no es adorno. Sin él este selector pesa más que
   `.boton:disabled` —dos clases y un atributo contra dos clases— y un primario
   deshabilitado conservaba su contorno de oro. La auditoría del 22/09/2026 lo
   midió en la pantalla de Derechos: «Preparar el informe», deshabilitado hasta
   escribir una dirección, salía con borde `rgb(122, 91, 0)` y texto apagado.
   Parecía pulsable y no lo era, que es el peor de los dos errores posibles, y
   además contradecía lo que dice el bloque de DESHABILITADO aquí abajo: que el
   estado se pinta igual para las cuatro variantes. */
:root[data-tema='claro'] .v-primario:not(:disabled) {
  border-color: var(--arles-accent-ink);
}

/* ── La respuesta al pulsar ─────────────────────────────────────────────────
   El botón cede bajo el dedo: un 2 % de escala durante 120 ms. Es la
   diferencia entre pulsar una imagen de un botón y pulsar un botón.

   `:active` y no `:focus`: sólo mientras se mantiene apretado. Y no se aplica
   a un botón deshabilitado ni a uno ocupado — un control que se hunde cuando
   no va a pasar nada promete algo que no cumple, y con `aria-busy` ya está
   pasando lo que tenía que pasar.

   `scale` en vez de moverlo: un desplazamiento arrastra el texto y se lee como
   un salto. Y `transform` no provoca recálculo de maquetación, así que esto no
   puede costar fluidez por muchos botones que haya en la pantalla — es la
   restricción que puso Dirección el 22/09/2026 al pedir los efectos.

   `prefers-reduced-motion` lo deja en 0.01 ms por la regla global de
   `base.css`: el hundido sigue ocurriendo, pero sin recorrido. */
.boton:active:not(:disabled):not([aria-busy='true']) {
  transform: scale(var(--arles-pulsado));
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

/* ─────────────────────────────────────────────────────────────────────────
   LA TINTA DEL BOTÓN DE PELIGRO ES LA SUYA, NO LA DEL ACENTO

   Decía `--arles-text-on-accent`, que es la tinta oscura del dorado. En el
   tema oscuro no se notaba, porque el rojo también es claro y la tinta oscura
   encaja. En el tema CLARO el rojo es `#B0281B` —oscuro— y la tinta seguía
   siendo oscura: **2.41:1**, menos de la mitad del mínimo de AA.

   Lo llevaban «Detener» y «Detener todos los envíos», que son el botón más
   destructivo del producto y el único freno de una campaña en marcha. El
   token correcto, `--arles-text-on-danger`, existía desde la Fase 2 con su
   comentario —«en oscuro el rojo es claro; en claro es oscuro, la tinta se
   invierte»— y **no lo usaba nadie aquí**. `AInsignia` sí lo usa, con un
   comentario que cuenta este mismo defecto arreglado en la insignia y no en
   el botón.

   La sonda no lo cazaba antes porque hasta la 7.0 no hubo ningún botón de
   peligro en una pantalla: la variante existía y nada la usaba. Medido el
   01/10/2026 en la 7.7. */
.v-peligro {
  background: var(--arles-danger);
  color: var(--arles-text-on-danger);
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
  color: var(--arles-text-on-danger);
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
