<script setup lang="ts">
/**
 * Selector (DESIGN_SYSTEM.md §6).
 *
 * `<select>` nativo, no una lista desplegable propia. Motivo: el desplegable
 * nativo lo dibuja el sistema operativo, así que hereda gratis la navegación
 * por teclado, la búsqueda por escritura, el comportamiento en pantallas
 * pequeñas y la integración con lectores de pantalla. Una reimplementación
 * tiene que volver a ganar todo eso y normalmente pierde alguna parte.
 *
 * Lo que sí se estiliza es el campo cerrado, que es lo que se ve el 99 % del
 * tiempo. La lista abierta la pinta el sistema y **se verá distinta en Windows
 * y en macOS**: es una divergencia aceptada (R-07), no un defecto.
 *
 * Igual que en AEntrada, el error va debajo con icono y texto (regla 7.3).
 */
import { computed, useId } from 'vue'

import AIcono from './AIcono.vue'

export interface OpcionDeSelector {
  valor: string
  texto: string
  deshabilitada?: boolean
}

const props = withDefaults(
  defineProps<{
    etiqueta: string
    opciones: readonly OpcionDeSelector[]
    ayuda?: string
    error?: string
    deshabilitado?: boolean
  }>(),
  { deshabilitado: false },
)

const valor = defineModel<string>({ default: '' })

const id = useId()
const idAyuda = computed(() => `${id}-ayuda`)
const idError = computed(() => `${id}-error`)
const descrito = computed(() => {
  if (props.error) return idError.value
  if (props.ayuda) return idAyuda.value
  return undefined
})
</script>

<template>
  <div class="campo">
    <label
      class="etiqueta"
      :for="id"
    >{{ etiqueta }}</label>

    <div class="envoltorio">
      <select
        :id="id"
        v-model="valor"
        class="selector"
        :class="{ 'con-error': !!error }"
        :disabled="deshabilitado"
        :aria-invalid="error ? true : undefined"
        :aria-describedby="descrito"
      >
        <option
          v-for="opcion in opciones"
          :key="opcion.valor"
          :value="opcion.valor"
          :disabled="opcion.deshabilitada"
        >
          {{ opcion.texto }}
        </option>
      </select>

      <AIcono
        class="flecha"
        nombre="desplegar"
      />
    </div>

    <p
      v-if="error"
      :id="idError"
      class="error"
      role="alert"
    >
      <AIcono nombre="error" />
      <span>{{ error }}</span>
    </p>

    <p
      v-else-if="ayuda"
      :id="idAyuda"
      class="ayuda"
    >
      {{ ayuda }}
    </p>
  </div>
</template>

<style scoped>
.campo {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
}

.etiqueta {
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.envoltorio {
  position: relative;
  display: flex;
}

.selector {
  flex: 1;
  min-height: var(--arles-control-height);
  /* Espacio a la derecha para la flecha dibujada encima. */
  padding: 0 var(--arles-space-7) 0 var(--arles-space-3);

  background: var(--arles-surface);
  color: var(--arles-text);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-sm);

  font-family: inherit;
  font-size: var(--arles-font-size-body);
  line-height: var(--arles-line-height-body);
  cursor: pointer;

  /* Quita la flecha del sistema para poner la de ARLES, que sí sigue el
     color del tema. La lista desplegada sigue siendo la nativa. */
  appearance: none;
  transition: border-color var(--arles-duration-fast) var(--arles-ease);
}

.selector:hover:not(:disabled) {
  border-color: var(--arles-border-strong);
}

.selector:disabled {
  /* Igual que en ABoton: se pinta el estado, no se atenúa el elemento. Un
     campo al 45 % de opacidad deja su etiqueta y su valor por debajo de AA, y
     el usuario deja de poder leer QUÉ es lo que no puede editar. */
  background: var(--arles-surface-raised);
  color: var(--arles-text-disabled);
  border-color: var(--arles-border);
  cursor: not-allowed;
}

.selector.con-error {
  border-color: var(--arles-danger);
}

.flecha {
  position: absolute;
  right: var(--arles-space-3);
  top: 50%;
  transform: translateY(-50%);
  color: var(--arles-text-muted);
  /* La flecha es decoración sobre el control: no debe robarle el clic. */
  pointer-events: none;
}

.ayuda,
.error {
  margin: 0;
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
}

.ayuda {
  color: var(--arles-text-muted);
}

.error {
  display: flex;
  align-items: flex-start;
  gap: var(--arles-space-1);
  color: var(--arles-danger);
}
</style>
