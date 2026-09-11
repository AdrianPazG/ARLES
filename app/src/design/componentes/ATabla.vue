<script setup lang="ts" generic="T extends { id: string }">
/**
 * Tabla virtualizada — el componente crítico (DESIGN_SYSTEM.md §6).
 *
 * Es donde el usuario pasa la mayor parte del tiempo y donde el presupuesto
 * de rendimiento se gana o se pierde: T-7 pide 500 000 contactos.
 *
 * **Virtualización siempre** (TanStack Virtual, ADR-0007). Sin ella el DOM
 * crecería con los datos y el desplazamiento se degradaría con el tamaño de
 * la lista, que es justo lo que un usuario con una lista grande nota primero.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ NO ES UN <table>
 *
 * Un `<table>` real no se puede virtualizar sin romper su propio modelo de
 * cajas: las filas ausentes se llevan por delante el ancho de las columnas.
 * Se usa la rejilla con roles ARIA explícitos (`grid`/`row`/`gridcell`), que
 * es el patrón que la especificación prevé para exactamente este caso, con
 * `aria-rowcount` declarando el total real y `aria-rowindex` la posición de
 * cada fila dentro de él — de modo que el lector de pantalla anuncia «fila
 * 40 231 de 500 000» aunque en el DOM sólo existan veinte.
 * ─────────────────────────────────────────────────────────────────────────
 *
 * Teclado completo: flechas, `Inicio`/`Fin`, `Espacio` para seleccionar.
 * La selección se marca con fondo **y** con una barra izquierda de acento:
 * no sólo color (regla 7.3).
 */
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref, watch } from 'vue'

import AIcono from './AIcono.vue'

export interface ColumnaDeTabla<F> {
  id: string
  titulo: string
  /** Fracción o medida CSS para `grid-template-columns`. */
  ancho?: string
  /** Cifras: alinea a la derecha y usa figuras tabulares. */
  numerica?: boolean
  ordenable?: boolean
  valor: (fila: F) => string
}

export type Direccion = 'asc' | 'desc'

const props = withDefaults(
  defineProps<{
    filas: readonly T[]
    columnas: readonly ColumnaDeTabla<T>[]
    /** Nombre accesible de la rejilla. Obligatorio: sin él es «tabla». */
    etiqueta: string
    densidad?: 'compacta' | 'comoda'
    ordenPor?: string
    ordenDireccion?: Direccion
  }>(),
  { densidad: 'compacta', ordenDireccion: 'asc' },
)

const emit = defineEmits<{
  ordenar: [columna: string, direccion: Direccion]
  activar: [fila: T]
}>()

const seleccionada = defineModel<string | undefined>('seleccionada', {})

// La altura de fila la fija el token, no una constante de aquí: el
// virtualizador necesita el número para calcular cuántas filas caben, y si lo
// llevara aparte acabaría discrepando del CSS sin que nadie lo notara.
const TOKEN_DE_ALTURA = {
  compacta: '--arles-row-height-compact',
  comoda: '--arles-row-height-comfortable',
} as const

/** Reserva si el token aún no está aplicado (el primer render, o un test). */
const ALTURA_DE_RESERVA = { compacta: 36, comoda: 44 } as const

const contenedor = ref<HTMLElement | null>(null)

const alturaDeFila = computed(() => {
  const raiz = contenedor.value
  if (!raiz) return ALTURA_DE_RESERVA[props.densidad]
  const valor = getComputedStyle(raiz)
    .getPropertyValue(TOKEN_DE_ALTURA[props.densidad])
    .trim()
  return Number.parseInt(valor, 10) || ALTURA_DE_RESERVA[props.densidad]
})

const virtual = useVirtualizer(
  computed(() => ({
    count: props.filas.length,
    getScrollElement: () => contenedor.value,
    estimateSize: () => alturaDeFila.value,
    // Tres filas de margen: suficiente para que el desplazamiento rápido no
    // enseñe hueco, sin pagar por renderizar una pantalla entera de más.
    overscan: 3,
  })),
)

const visibles = computed(() => virtual.value.getVirtualItems())
const alto = computed(() => virtual.value.getTotalSize())

const plantilla = computed(() =>
  props.columnas.map((c) => c.ancho ?? '1fr').join(' '),
)

const indiceSeleccionado = computed(() =>
  props.filas.findIndex((f) => f.id === seleccionada.value),
)

function seleccionarPorIndice(indice: number) {
  const fila = props.filas[Math.max(0, Math.min(indice, props.filas.length - 1))]
  if (!fila) return
  seleccionada.value = fila.id
  // La fila elegida con el teclado puede estar fuera de la ventana virtual:
  // sin esto el usuario mueve una selección que no ve.
  virtual.value.scrollToIndex(props.filas.indexOf(fila))
}

function alPulsarTecla(evento: KeyboardEvent) {
  const actual = indiceSeleccionado.value
  const acciones: Record<string, () => void> = {
    ArrowDown: () => seleccionarPorIndice(actual + 1),
    ArrowUp: () => seleccionarPorIndice(actual - 1),
    Home: () => seleccionarPorIndice(0),
    End: () => seleccionarPorIndice(props.filas.length - 1),
    PageDown: () => seleccionarPorIndice(actual + 10),
    PageUp: () => seleccionarPorIndice(actual - 10),
    ' ': () => seleccionarPorIndice(actual < 0 ? 0 : actual),
    Enter: () => {
      const fila = props.filas[actual]
      if (fila) emit('activar', fila)
    },
  }
  const accion = acciones[evento.key]
  if (!accion) return
  evento.preventDefault()
  accion()
}

function alternarOrden(columna: ColumnaDeTabla<T>) {
  if (!columna.ordenable) return
  const mismaColumna = props.ordenPor === columna.id
  const direccion: Direccion =
    mismaColumna && props.ordenDireccion === 'asc' ? 'desc' : 'asc'
  emit('ordenar', columna.id, direccion)
}

function ordenAria(columna: ColumnaDeTabla<T>) {
  if (!columna.ordenable) return undefined
  if (props.ordenPor !== columna.id) return 'none'
  return props.ordenDireccion === 'asc' ? 'ascending' : 'descending'
}

// Si la lista se acorta —un filtro, un borrado— la selección puede apuntar a
// una fila que ya no existe. Dejarla colgada da una rejilla sin nada marcado
// y un `aria-activedescendant` roto.
watch(
  () => props.filas,
  (filas) => {
    if (seleccionada.value && !filas.some((f) => f.id === seleccionada.value)) {
      seleccionada.value = undefined
    }
  },
)
</script>

<template>
  <div
    class="tabla"
    role="grid"
    :aria-label="etiqueta"
    :aria-rowcount="filas.length + 1"
    tabindex="0"
    @keydown="alPulsarTecla"
  >
    <div
      class="cabecera"
      role="row"
      aria-rowindex="1"
      :style="{ gridTemplateColumns: plantilla }"
    >
      <div
        v-for="columna in columnas"
        :key="columna.id"
        class="celda titulo"
        :class="{ numerica: columna.numerica }"
        role="columnheader"
        :aria-sort="ordenAria(columna)"
      >
        <button
          v-if="columna.ordenable"
          class="orden"
          type="button"
          @click="alternarOrden(columna)"
        >
          <span>{{ columna.titulo }}</span>
          <!-- La dirección se ve en el icono, además de estar en
               `aria-sort`: el orden es información y no puede vivir sólo en
               el atributo que el lector de pantalla lee. -->
          <AIcono
            v-if="ordenPor === columna.id"
            class="marca-de-orden"
            :class="{ descendente: ordenDireccion === 'desc' }"
            nombre="desplegar"
            :tamano="14"
          />
        </button>
        <span v-else>{{ columna.titulo }}</span>
      </div>
    </div>

    <div
      ref="contenedor"
      class="cuerpo"
    >
      <div
        class="lienzo"
        :style="{ height: `${alto}px` }"
      >
        <div
          v-for="item in visibles"
          :key="filas[item.index]!.id"
          class="fila"
          :class="{
            seleccionada: filas[item.index]!.id === seleccionada,
            alterna: item.index % 2 === 1,
          }"
          role="row"
          :aria-rowindex="item.index + 2"
          :aria-selected="filas[item.index]!.id === seleccionada"
          :style="{
            gridTemplateColumns: plantilla,
            height: `${item.size}px`,
            transform: `translateY(${item.start}px)`,
          }"
          @click="seleccionada = filas[item.index]!.id"
          @dblclick="emit('activar', filas[item.index]!)"
        >
          <div
            v-for="columna in columnas"
            :key="columna.id"
            class="celda"
            :class="{ numerica: columna.numerica }"
            role="gridcell"
          >
            {{ columna.valor(filas[item.index]!) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tabla {
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  overflow: hidden;
}

/* Pegajosa por posición en el flex, no por `position: sticky`: el cuerpo es
   el que se desplaza, así que la cabecera se queda quieta sin coste. */
.cabecera {
  display: grid;
  background: var(--arles-surface);
  border-bottom: var(--arles-border-width) solid var(--arles-border);
}

.cuerpo {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.lienzo {
  position: relative;
  width: 100%;
}

.fila {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  display: grid;
  align-items: center;
  cursor: default;
  /* La barra de selección se reserva siempre para que la fila no se desplace
     3 px al seleccionarla. */
  box-shadow: inset 3px 0 0 transparent;
}

/* Filas alternas: guían el ojo sin llegar a rayado (DESIGN_SYSTEM §6).
   La alternancia va por el ÍNDICE VIRTUAL, no por `nth-child`: en el DOM sólo
   existen las filas visibles, así que un selector posicional cambiaría de
   pie al desplazar y el rayado parpadearía. */
.fila.alterna {
  background: var(--arles-surface);
}

.fila:hover {
  background: var(--arles-surface-hover);
}

/* Fondo Y barra de acento: dos señales (regla 7.3). */
.fila.seleccionada {
  background: var(--arles-surface-raised);
  box-shadow: inset 3px 0 0 var(--arles-accent);
}

.celda {
  padding: 0 var(--arles-space-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  color: var(--arles-text);
}

.titulo {
  display: flex;
  align-items: center;
  min-height: var(--arles-row-height-compact);
  font-size: var(--arles-font-size-caption);
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

/* Las cifras van a la derecha y con figuras tabulares: sin esto las columnas
   de números no alinean y bailan al actualizarse en vivo. */
.numerica {
  justify-content: flex-end;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.orden {
  display: inline-flex;
  align-items: center;
  gap: var(--arles-space-1);
  padding: 0;
  background: transparent;
  border: none;
  color: inherit;
  font: inherit;
  letter-spacing: inherit;
  text-transform: inherit;
  cursor: pointer;
}

.orden:hover {
  color: var(--arles-text);
}

.marca-de-orden {
  color: var(--arles-accent);
  transform: rotate(180deg);
  transition: transform var(--arles-duration-fast) var(--arles-ease);
}

.marca-de-orden.descendente {
  transform: rotate(0deg);
}
</style>
