<script setup lang="ts">
/**
 * Alta y edición de un contacto, con sus canales (L-13).
 *
 * Es el mismo formulario para las dos cosas. Separarlos daría dos sitios donde
 * mantener las mismas reglas de canales, y el día que divergieran «editar»
 * aceptaría algo que «agregar» rechaza.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * ESTA PANTALLA NO VALIDA
 *
 * Igual que la de empresa. Envía y pinta lo que el núcleo responda. Podría
 * comprobar el correo aquí y avisar antes, pero entonces habría dos reglas que
 * mantener iguales —y la del núcleo normaliza el móvil mexicano quitando el
 * «1», algo que un formulario no puede replicar sin repetir el código—.
 *
 * Lo único que hace este componente por su cuenta es **geometría de la lista**:
 * cuántas filas hay, cuál es la principal de su tipo, y no pasar del tope. Eso
 * no es validación de datos, es el estado del propio formulario.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import {
  MAX_CANALES,
  contactoEnBlanco,
  type BorradorDeContacto,
  type Canal,
} from '@/app/stores/contactos'
import { AAviso, ABoton, AEntrada, ASelector } from '@/design/componentes'

const props = defineProps<{
  /** El contacto que se edita, o `null` para dar de alta uno nuevo. */
  inicial: BorradorDeContacto | null
  guardando: boolean
  /** Clave de i18n del error de cada canal, por índice. */
  erroresDeCanal: Record<number, string>
  erroresDeCampo: Record<string, string>
  errorGeneral: string | null
  direccionEnConflicto: string | null
}>()

const emit = defineEmits<{
  guardar: [borrador: BorradorDeContacto]
  cancelar: []
  /** El usuario tocó algo: los errores de la vez anterior ya no aplican. */
  tocar: []
}>()

const { t, te } = useI18n()

const borrador = reactive<BorradorDeContacto>(
  props.inicial ? estructurado(props.inicial) : contactoEnBlanco(),
)

/**
 * Copia profunda del borrador que llega.
 *
 * Sin ella, escribir en el formulario cambiaría el contacto de la lista **antes
 * de guardar**, y cancelar dejaría la tabla enseñando algo que no está en la
 * base.
 */
function estructurado(b: BorradorDeContacto): BorradorDeContacto {
  return {
    nombre: b.nombre,
    apellido: b.apellido,
    empresa: b.empresa,
    canales: b.canales.map((c) => ({ ...c })),
  }
}

// Si cambia el contacto que se edita —se cierra uno y se abre otro sin
// desmontar el formulario— hay que rehacer el borrador. Sin esto, el segundo
// contacto abriría con los datos del primero.
watch(
  () => props.inicial,
  (nuevo) => {
    const base = nuevo ? estructurado(nuevo) : contactoEnBlanco()
    borrador.nombre = base.nombre
    borrador.apellido = base.apellido
    borrador.empresa = base.empresa
    borrador.canales = base.canales
  },
)

const esAlta = computed(() => props.inicial === null)

const OPCIONES_DE_CANAL: readonly { valor: Canal; clave: string }[] = [
  { valor: 'email', clave: 'contactos.canal.email' },
  { valor: 'whatsapp', clave: 'contactos.canal.whatsapp' },
]

const opcionesDeCanal = computed(() =>
  OPCIONES_DE_CANAL.map((o) => ({ valor: o.valor, texto: t(o.clave) })),
)

const puedeAnadir = computed(() => borrador.canales.length < MAX_CANALES)

/** Cuántos canales hay de cada tipo. Decide si el botón de principal aparece. */
function cuantosDe(canal: Canal): number {
  return borrador.canales.filter((c) => c.canal === canal).length
}

function anadirCanal(): void {
  if (!puedeAnadir.value) return
  emit('tocar')
  // El tipo del nuevo es el del último que hay: quien está metiendo tres
  // móviles seguidos no quiere elegir «WhatsApp» tres veces.
  const ultimo = borrador.canales.at(-1)
  const canal: Canal = ultimo ? ultimo.canal : 'email'
  borrador.canales.push({
    canal,
    valor: '',
    // Principal sólo si es el primero de su tipo. Si no, marcarlo movería el
    // principal del usuario sin que lo pidiera — y con él, a qué dirección se
    // le escribe.
    principal: cuantosDe(canal) === 0,
    })
}

/**
 * Quita un canal.
 *
 * Si el que se va era el principal de su tipo, el primero que quede de ese
 * tipo lo hereda. Dejar un tipo sin principal haría que la tabla no supiera
 * cuál enseñar, y que el envío no supiera a cuál escribir.
 */
function quitarCanal(indice: number): void {
  emit('tocar')
  const fuera = borrador.canales[indice]
  if (!fuera) return
  borrador.canales.splice(indice, 1)
  if (!fuera.principal) return
  const heredero = borrador.canales.find((c) => c.canal === fuera.canal)
  if (heredero) heredero.principal = true
}

/** Marca uno como principal y quita la marca a los demás **de su tipo**. */
function marcarPrincipal(indice: number): void {
  emit('tocar')
  const elegido = borrador.canales[indice]
  if (!elegido) return
  for (const c of borrador.canales) {
    if (c.canal === elegido.canal) c.principal = false
  }
  elegido.principal = true
}

/**
 * Al cambiar el tipo de un canal hay que recolocar los principales de los
 * **dos** tipos: el que abandona puede quedarse sin principal, y el que recibe
 * puede acabar con dos.
 */
function cambiarTipo(indice: number, valor: string): void {
  emit('tocar')
  const fila = borrador.canales[indice]
  if (!fila) return
  const nuevo: Canal = valor === 'whatsapp' ? 'whatsapp' : 'email'
  if (nuevo === fila.canal) return
  const anterior = fila.canal
  const eraPrincipal = fila.principal

  fila.canal = nuevo
  // En su nuevo tipo: principal sólo si no había ninguno.
  fila.principal = !borrador.canales.some(
    (c) => c !== fila && c.canal === nuevo && c.principal,
  )

  if (eraPrincipal) {
    const heredero = borrador.canales.find((c) => c !== fila && c.canal === anterior)
    if (heredero) heredero.principal = true
  }
}

/** El texto de ayuda de cada tipo, que es donde se explica el formato. */
function ayudaDe(canal: Canal): string {
  return t(`contactos.ayuda.${canal}`)
}

/**
 * El texto del error de un canal, o cadena vacía si no lo hay.
 *
 * Cadena vacía y no `undefined`: con `exactOptionalPropertyTypes` pasar
 * `undefined` a una prop opcional no compila, y las primitivas ya tratan la
 * cadena vacía como «sin error» (`v-if="error"`).
 */
function errorDeCanal(indice: number): string {
  const clave = props.erroresDeCanal[indice]
  if (clave === undefined) return ''
  // `te` y no `t` a secas: una clave sin texto se pintaría cruda debajo del
  // campo, y el usuario leería «error.telefono_invalido» como explicación.
  return te(clave) ? t(clave) : t('contactos.error.canalGenerico')
}

function errorDeCampo(campo: string): string {
  const clave = props.erroresDeCampo[campo]
  if (clave === undefined) return ''
  return te(clave) ? t(clave) : t('contactos.error.campoGenerico')
}

/**
 * El error general, ya resuelto a sus tres partes.
 *
 * Pasa por `resolverError` y no por `$t(...)` porque esa función sabe qué
 * hacer cuando falta el texto: deja constancia y cae en un genérico legible.
 */
const avisoGeneral = computed(() =>
  props.errorGeneral
    ? resolverError({
        clave: props.errorGeneral,
        detalle: props.direccionEnConflicto ?? '',
      })
    : null,
)

function enviar(): void {
  emit('guardar', estructurado(borrador))
}
</script>

<template>
  <form
    class="formulario"
    novalidate
    @submit.prevent="enviar"
  >
    <p class="formulario__intro">
      {{ esAlta ? t('contactos.formulario.introAlta') : t('contactos.formulario.introEdicion') }}
    </p>

    <AAviso
      v-if="avisoGeneral"
      tono="peligro"
      :titulo="avisoGeneral.que"
      urgente
    >
      <p>{{ avisoGeneral.como }}</p>
      <p class="formulario__salvo">
        {{ avisoGeneral.salvo }}
      </p>
    </AAviso>

    <div class="formulario__identidad">
      <AEntrada
        v-model="borrador.nombre"
        :etiqueta="t('contactos.campo.nombre')"
        :error="errorDeCampo('nombre')"
        @update:model-value="emit('tocar')"
      />
      <AEntrada
        v-model="borrador.apellido"
        :etiqueta="t('contactos.campo.apellido')"
        :error="errorDeCampo('apellido')"
        @update:model-value="emit('tocar')"
      />
    </div>

    <AEntrada
      v-model="borrador.empresa"
      :etiqueta="t('contactos.campo.empresa')"
      :ayuda="t('contactos.ayuda.empresa')"
      :error="errorDeCampo('empresa')"
      @update:model-value="emit('tocar')"
    />

    <fieldset class="canales">
      <legend class="canales__titulo">
        {{ t('contactos.canales.titulo') }}
      </legend>
      <p class="canales__ayuda">
        {{ t('contactos.canales.ayuda') }}
      </p>

      <div
        v-for="(canal, i) in borrador.canales"
        :key="i"
        class="canal"
        :class="{ 'canal--principal': canal.principal }"
      >
        <ASelector
          :model-value="canal.canal"
          :etiqueta="t('contactos.campo.tipo')"
          :opciones="opcionesDeCanal"
          class="canal__tipo"
          @update:model-value="(v: string) => cambiarTipo(i, v)"
        />

        <AEntrada
          v-model="canal.valor"
          :etiqueta="t('contactos.campo.valor')"
          :ayuda="ayudaDe(canal.canal)"
          :error="errorDeCanal(i)"
          :tipo="canal.canal === 'email' ? 'email' : 'text'"
          class="canal__valor"
          @update:model-value="emit('tocar')"
        />

        <div class="canal__acciones">
          <!--
            El botón de principal sólo aparece cuando hay más de uno de ese
            tipo. Con un solo correo, «marcar como principal» es una pregunta
            sin alternativa: ya lo es.
          -->
          <ABoton
            v-if="cuantosDe(canal.canal) > 1"
            variante="sutil"
            tamano="compacto"
            :icono="canal.principal ? 'exito' : 'info'"
            :aria-pressed="canal.principal"
            @click="marcarPrincipal(i)"
          >
            {{ t('contactos.canales.principal') }}
          </ABoton>

          <ABoton
            v-if="borrador.canales.length > 1"
            variante="sutil"
            tamano="compacto"
            icono="cerrar"
            :etiqueta="
              t('contactos.canales.quitarDe', {
                valor: canal.valor || t('contactos.canales.sinValor'),
              })
            "
            @click="quitarCanal(i)"
          />
        </div>
      </div>

      <div class="canales__pie">
        <ABoton
          variante="secundario"
          icono="mas"
          :deshabilitado="!puedeAnadir"
          @click="anadirCanal"
        >
          {{ t('contactos.canales.anadir') }}
        </ABoton>
        <!--
          El tope se dice SIEMPRE, no sólo al llegar a él: un botón que se
          deshabilita sin explicación se lee como una avería.
        -->
        <p class="canales__tope">
          {{ t('contactos.canales.tope', { hay: borrador.canales.length, max: MAX_CANALES }) }}
        </p>
      </div>
    </fieldset>

    <div class="formulario__pie">
      <ABoton
        variante="secundario"
        @click="emit('cancelar')"
      >
        {{ t('comun.cancelar') }}
      </ABoton>
      <ABoton
        variante="primario"
        tipo="submit"
        :ocupado="guardando"
      >
        {{ esAlta ? t('contactos.formulario.darDeAlta') : t('comun.guardar') }}
      </ABoton>
    </div>
  </form>
</template>

<style scoped>
.formulario {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.formulario__intro {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.formulario__salvo {
  margin: var(--arles-space-2) 0 0;
  color: var(--arles-text-muted);
}

.formulario__identidad {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--arles-space-3);
}

.canales {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
  margin: 0;
  padding: var(--arles-space-4);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
}

.canales__titulo {
  padding: 0 var(--arles-space-2);
  color: var(--arles-text);
  font-weight: var(--arles-font-weight-bold);
}

.canales__ayuda,
.canales__tope {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.canal {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: var(--arles-space-3);
  align-items: start;
  padding-left: var(--arles-space-3);
  /* La barra de acento marca el principal además del icono: nunca sólo
     color, y nunca sólo un icono pequeño (regla 7.3). */
  border-left: var(--arles-space-1) solid transparent;
}

.canal--principal {
  border-left-color: var(--arles-accent);
}

.canal__tipo {
  min-width: var(--arles-menu-min-width);
}

.canal__acciones {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  /* A la altura del campo, no de su etiqueta. */
  padding-top: var(--arles-space-5);
}

.canales__pie {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--arles-space-3);
}

.formulario__pie {
  display: flex;
  justify-content: flex-end;
  gap: var(--arles-space-3);
}

@media (max-width: 40rem) {
  .formulario__identidad,
  .canal {
    grid-template-columns: 1fr;
  }

  .canal__acciones {
    padding-top: 0;
  }
}
</style>
