<script setup lang="ts">
/**
 * Campo de texto (DESIGN_SYSTEM.md §6).
 *
 * **El error va debajo, con icono y texto** — nunca sólo con borde rojo
 * (§19, COLOR_SYSTEM.md regla 7.3). Un borde rojo es información codificada
 * únicamente en color: quien no lo distingue ve un campo idéntico a los demás
 * y no sabe qué corregir.
 *
 * El mensaje se enlaza con `aria-describedby` y se marca `aria-invalid`, así
 * que un lector de pantalla lo anuncia al entrar al campo en vez de dejar al
 * usuario descubrirlo al enviar.
 */
import { computed, useId } from 'vue'

import AIcono from './AIcono.vue'

const props = withDefaults(
  defineProps<{
    etiqueta: string
    tipo?: 'text' | 'email' | 'password' | 'number'
    /** Texto de ayuda permanente. Se oculta cuando hay error. */
    ayuda?: string
    error?: string
    marcador?: string
    deshabilitado?: boolean
    requerido?: boolean
    /** Cifras: activa figuras tabulares para que las columnas alineen. */
    numerico?: boolean
  }>(),
  {
    tipo: 'text',
    deshabilitado: false,
    requerido: false,
    numerico: false,
  },
)

const valor = defineModel<string>({ default: '' })

const id = useId()
const idAyuda = computed(() => `${id}-ayuda`)
const idError = computed(() => `${id}-error`)

// Sólo se describe por lo que está visible: apuntar a un nodo oculto deja al
// lector de pantalla anunciando una referencia rota.
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
    >
      {{ etiqueta }}
      <span
        v-if="requerido"
        class="requerido"
        aria-hidden="true"
      >*</span>
    </label>

    <input
      :id="id"
      v-model="valor"
      class="entrada"
      :class="{ 'con-error': !!error }"
      :type="tipo"
      :placeholder="marcador"
      :disabled="deshabilitado"
      :required="requerido"
      :data-numeric="numerico || undefined"
      :aria-invalid="error ? true : undefined"
      :aria-describedby="descrito"
    >

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

.requerido {
  color: var(--arles-accent);
  margin-left: 0.15em;
}

.entrada {
  min-height: var(--arles-control-height);
  padding: 0 var(--arles-space-3);

  background: var(--arles-surface);
  color: var(--arles-text);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-sm);

  font-family: inherit;
  font-size: var(--arles-font-size-body);
  line-height: var(--arles-line-height-body);

  transition: border-color var(--arles-duration-fast) var(--arles-ease);
}

.entrada::placeholder {
  color: var(--arles-text-muted);
  opacity: 0.6;
}

.entrada:hover:not(:disabled) {
  border-color: var(--arles-border-strong);
}

.entrada:disabled {
  /* Igual que en ABoton: se pinta el estado, no se atenúa el elemento. Un
     campo al 45 % de opacidad deja su etiqueta y su valor por debajo de AA, y
     el usuario deja de poder leer QUÉ es lo que no puede editar. */
  background: var(--arles-surface-raised);
  color: var(--arles-text-disabled);
  border-color: var(--arles-border);
  cursor: not-allowed;
}

/* El borde rojo ACOMPAÑA al mensaje de abajo; no lo sustituye. */
.entrada.con-error {
  border-color: var(--arles-danger);
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
