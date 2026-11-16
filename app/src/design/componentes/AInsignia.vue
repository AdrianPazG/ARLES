<script setup lang="ts">
/**
 * Insignia de estado (DESIGN_SYSTEM.md §6).
 *
 * **Relleno sólido con texto oscuro**, no color de texto sobre transparente.
 * El motivo está medido: sobre `surface-raised` los colores semánticos dan
 * entre 2.58 y 2.92 contra el fondo — todos por debajo de AA. Con relleno
 * sólido, el contraste que cuenta es el del relleno contra
 * `--arles-text-on-accent`, que pasa siempre.
 *
 * **Cada insignia de color lleva icono además del color** (regla 7.3). No hay
 * forma de construir una sin él: el icono se deriva del tono, no se pasa por
 * fuera.
 *
 * ── EL NEUTRO ES LA EXCEPCIÓN, Y TIENE QUE SERLO ─────────────────────────
 *
 * El neutro llevaba el icono de reloj de «en cola». La regla 7.3 lo pedía, y
 * la regla 7.3 no le aplica: existe para que un estado no se distinga **sólo
 * por el color**, y el neutro no tiene color semántico —es el fondo elevado
 * con un borde—, así que no hay nada que redundar.
 *
 * Lo que sí hacía el reloj era afirmar una espera que en ninguna de sus ocho
 * apariciones existe: «3 por correo» es un contador, «Todos sus canales» una
 * marca, «Calidad desconocida» es que no lo hemos preguntado, y una campaña en
 * borrador no está esperando a nada. Un icono equivocado dice más que ninguno.
 *
 * Se vio el 29/09/2026 mirando la pantalla de Bajas en un navegador: la sonda
 * mide contrastes y desbordes, no si un dibujo miente.
 */
import { computed } from 'vue'

import AIcono, { type NombreDeIcono } from './AIcono.vue'

type Tono = 'neutro' | 'exito' | 'aviso' | 'peligro' | 'info' | 'incierto'
type TonoDeColor = Exclude<Tono, 'neutro'>

const ICONO: Record<TonoDeColor, NombreDeIcono> = {
  exito: 'exito',
  aviso: 'aviso',
  peligro: 'error',
  info: 'info',
  // ADR-0004: el envío ambiguo no es un éxito ni un fallo. Mostrarlo con el
  // icono de cualquiera de los dos afirmaría algo que no se sabe.
  incierto: 'incierto',
}

const props = withDefaults(defineProps<{ tono?: Tono }>(), { tono: 'neutro' })

const icono = computed(() => (props.tono === 'neutro' ? null : ICONO[props.tono]))
</script>

<template>
  <span
    class="insignia"
    :class="`t-${tono}`"
  >
    <AIcono
      v-if="icono"
      :nombre="icono"
    />
    <span><slot /></span>
  </span>
</template>

<style scoped>
.insignia {
  display: inline-flex;
  align-items: center;
  gap: var(--arles-space-1);

  padding: 2px var(--arles-space-2);
  border-radius: var(--arles-radius-sm);

  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  font-weight: var(--arles-font-weight-semibold);
  white-space: nowrap;

  /* ── CADA TONO LLEVA SU PROPIA TINTA, Y NO UNA COMÚN ──────────────────
     Aquí había un solo `--arles-text-on-accent` para todos los rellenos, con
     el motivo escrito de que «el contraste sea el del relleno y no dependa del
     tono». La idea era buena y el dato no la sostenía: **en el tema claro, los
     colores de estado son oscuros** —el verde es #0F6B52 y el ámbar #8A5200—,
     así que ponerles encima una tinta casi negra daba 2.47:1 sobre un mínimo
     de 4.5.
     Lo midió la auditoría el 29/09/2026, en cuanto las pantallas nuevas
     empezaron a enseñar insignias de verdad. Antes no había datos que las
     pintaran, así que el defecto existía y nadie lo veía.
     Cada tono usa ahora su `--arles-text-on-*`, que ya era el patrón de
     peligro y de info: blanco en el tema claro, tinta oscura en el oscuro. */
  color: var(--arles-text-on-accent);
}

.t-exito {
  background: var(--arles-success);
  color: var(--arles-text-on-success);
}

.t-aviso {
  background: var(--arles-warning);
  color: var(--arles-text-on-warning);
}

.t-peligro {
  background: var(--arles-danger);
  color: var(--arles-text-on-danger);
}

.t-info {
  background: var(--arles-info);
  color: var(--arles-text-on-info);
}

.t-incierto {
  background: var(--arles-accent);
}

/* El neutro es la excepción y por eso va al revés: no hay un relleno claro
   que represente «nada todavía» sin gritar. Lleva borde para no confundirse
   con texto suelto, y su texto sí es el claro de la aplicación, que pasa AA
   sobre `surface-raised` (COLOR_SYSTEM). */
.t-neutro {
  background: var(--arles-surface-raised);
  color: var(--arles-text);
  border: var(--arles-border-width) solid var(--arles-border-strong);
  /* Compensa el borde para que todas las insignias midan igual de alto. */
  padding: 1px calc(var(--arles-space-2) - 1px);
}
</style>
