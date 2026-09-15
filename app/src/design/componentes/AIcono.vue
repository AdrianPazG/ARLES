<script setup lang="ts">
/**
 * Iconos de ARLES.
 *
 * Un conjunto cerrado, dibujado a mano en trazo de 1.5 sobre rejilla de 24.
 * No hay biblioteca de iconos: una dependencia de iconos trae cientos de
 * glifos que no se usan, y en Tauri el bundle lo paga la máquina del cliente.
 *
 * Por qué un conjunto CERRADO y no un `src` libre: la regla 7.3 del
 * COLOR_SYSTEM exige que ningún estado se comunique sólo con color, así que
 * cada estado semántico tiene que tener icono asignado. Con un tipo unión el
 * compilador rechaza un nombre que no existe, en vez de renderizar nada.
 *
 * `aria-hidden` por defecto: el icono acompaña a un texto que ya dice lo
 * mismo. Sólo cuando el icono va solo se le pasa `etiqueta`.
 */
import { computed } from 'vue'

const TRAZOS = {
  // Semánticos — uno por estado, regla 7.3
  exito: 'M20 6 9 17l-5-5',
  aviso: 'M12 9v4m0 4h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z',
  error: 'M12 8v4m0 4h.01M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',
  info: 'M12 16v-4m0-4h.01M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',

  // Estados de envío
  cola: 'M12 7v5l3 2m-3 7a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',
  enviando: 'M22 2 11 13M22 2l-7 20-4-9-9-4 20-7Z',
  suprimido: 'M4.9 4.9l14.2 14.2M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',
  // El envío ambiguo (ADR-0004) tiene icono propio: no es un éxito ni un
  // fallo, y mostrarlo con cualquiera de los dos sería afirmar algo que no
  // se sabe.
  incierto: 'M12 17h.01M9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',

  // Secciones de la navegación (UX_NAVEGACION.md §2).
  //
  // Existen porque P-11 decidió que la barra lateral plegada se queda en
  // «iconos sin texto»: sin un icono por sección, plegada no se puede navegar.
  // Uno por sección y ninguno de más — el conjunto sigue cerrado.
  inicio: 'M3 10.6 12 3.5l9 7.1M5.5 9.4V20.5h13V9.4',
  campanas: 'M3 10v4h3l6 4V6L6 10H3ZM17.5 9.2a4 4 0 0 1 0 5.6',
  contactos:
    'M15.5 20v-1.5a4 4 0 0 0-4-4h-5a4 4 0 0 0-4 4V20M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7M21.5 20v-1.5a4 4 0 0 0-3-3.87',
  remitentes: 'M3.5 6.5h17v11h-17zM3.5 7.5l8.5 5.5 8.5-5.5',
  actividad: 'M3 12h3.5l2.5 6 4-12 2.5 6H21',
  ajustes: 'M4 7h9M17 7h3M4 17h3M11 17h9M15 5v4M9 15v4',

  // Interfaz
  cerrar: 'M18 6 6 18M6 6l12 12',
  // Plegar y expandir la barra lateral. Dobles galones, no simples: un galón
  // simple es el de «desplegar» de un menú, y en la misma pantalla los dos
  // significarían cosas distintas con el mismo dibujo.
  plegar: 'm13 6-6 6 6 6M19 6l-6 6 6 6',
  expandir: 'm11 6 6 6-6 6M5 6l6 6-6 6',
  buscar: 'M21 21l-4.3-4.3M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z',
  desplegar: 'm6 9 6 6 6-6',
  ordenar: 'm7 15 5 5 5-5M7 9l5-5 5 5',
  mas: 'M12 5v14M5 12h14',
} as const

export type NombreDeIcono = keyof typeof TRAZOS

const props = defineProps<{
  nombre: NombreDeIcono
  /** Sólo cuando el icono va sin texto al lado. */
  etiqueta?: string
  /** Hereda el tamaño de fuente del contexto por defecto. Píxeles. */
  tamano?: number
}>()

const trazo = computed(() => TRAZOS[props.nombre])
const lado = computed(() => (props.tamano ? `${props.tamano}px` : '1em'))
</script>

<template>
  <svg
    class="icono"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-linecap="round"
    stroke-linejoin="round"
    :width="lado"
    :height="lado"
    :role="etiqueta ? 'img' : undefined"
    :aria-label="etiqueta"
    :aria-hidden="etiqueta ? undefined : true"
    focusable="false"
  >
    <path :d="trazo" />
  </svg>
</template>

<style scoped>
.icono {
  display: inline-block;
  flex: none;
  vertical-align: -0.125em;
}
</style>
