<script setup lang="ts">
/**
 * Escribir plantillas y firmas de correo (entrega 6.8a).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LA PANTALLA QUE FALTABA, Y LO QUE SU AUSENCIA SIGNIFICABA
 *
 * `guardar_plantilla` y `guardar_firma` existen desde la 6.3, probados. Lo que
 * no había era **nadie que los llamara**: desde la aplicación no se podía crear
 * una plantilla, y el asistente de campaña exige una para pasar del paso 4 de 9.
 *
 * Así que se podía crear una campaña «de principio a fin» solo si alguien había
 * escrito plantillas en la base por fuera. Con la aplicación sola, no.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LAS VARIABLES SE INSERTAN CON UN BOTÓN, NO SE ESCRIBEN A MANO
 *
 * Un `{{nombe}}` mal escrito no da error: el motor de sustitución no lo
 * reconoce como variable, así que viaja **tal cual** hasta el correo de la
 * persona. Y el preflight tampoco lo caza, porque para él es texto.
 *
 * Con botones, la llave y el nombre los pone el programa. Es la misma razón por
 * la que el puerto de Gmail se rellena solo.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA NO HACE
 *
 * **No sanea el HTML.** Lo hace Rust con `ammonia` (§6.2), y lo hace dos veces.
 * Limpiar aquí también daría la impresión de que es esta capa la que protege —
 * y el día que alguien llamara al comando desde otro sitio, no protegería nada.
 *
 * **No edita una versión existente.** Una versión de plantilla no se toca
 * (6.3): guardar crea una nueva. Enseñar un botón de «editar» sería prometer
 * algo que el modelo de datos impide a propósito, para que una campaña activa
 * siga mandando el texto que se aprobó.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import { useCampanasStore } from '@/app/stores/campanas'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModulo,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const store = useCampanasStore()
const { t } = useI18n()

onMounted(() => {
  void store.cargar()
})

/** Las cuatro variables del núcleo (`arles_core::plantilla::Variable`). */
const VARIABLES = ['nombre', 'apellido', 'empresa', 'mi_empresa'] as const

type Cual = 'plantilla' | 'firma' | null
const editando = ref<Cual>(null)

const nombre = ref('')
const asunto = ref('')
const cuerpo = ref('')
const porDefecto = ref(false)

/** El área de texto, para poder insertar una variable donde está el cursor. */
const areaDelCuerpo = ref<HTMLTextAreaElement | null>(null)

const problema = computed(() =>
  store.errorGeneral ? resolverError({ clave: store.errorGeneral, detalle: '' }) : null,
)

/**
 * Qué variables usa lo que se está escribiendo.
 *
 * Se recalcula mientras se escribe para que se vea **antes de guardar** qué
 * datos va a necesitar cada contacto. El mismo patrón lo reconoce el núcleo;
 * aquí solo se enseña, no se decide nada con ello.
 */
const variablesUsadas = computed(() => {
  const encontradas = new Set<string>()
  for (const m of `${asunto.value} ${cuerpo.value}`.matchAll(/\{\{\s*(\w+)\s*\}\}/g)) {
    if (m[1] !== undefined) encontradas.add(m[1])
  }
  return [...encontradas]
})

/**
 * Las que parecen una variable y no lo son.
 *
 * **Es la comprobación que salva un envío.** `{{nombe}}` no falla en ningún
 * sitio: no es una variable, así que el motor lo deja tal cual y sale impreso
 * en el correo. Aquí se ve en rojo antes de guardar.
 */
const variablesQueNoExisten = computed(() =>
  variablesUsadas.value.filter((v) => !VARIABLES.includes(v as (typeof VARIABLES)[number])),
)

const esPlantilla = computed(() => editando.value === 'plantilla')

const puedeGuardar = computed(
  () =>
    nombre.value.trim().length > 0 &&
    cuerpo.value.trim().length > 0 &&
    (!esPlantilla.value || asunto.value.trim().length > 0) &&
    variablesQueNoExisten.value.length === 0,
)

function abrir(cual: Exclude<Cual, null>): void {
  editando.value = cual
  nombre.value = ''
  asunto.value = ''
  cuerpo.value = ''
  porDefecto.value = false
}

function cerrar(): void {
  editando.value = null
}

/** Mete la variable donde está el cursor, no al final. */
function insertar(variable: string): void {
  const marca = `{{${variable}}}`
  const area = areaDelCuerpo.value
  if (!area) {
    cuerpo.value += marca
    return
  }
  const desde = area.selectionStart
  const hasta = area.selectionEnd
  cuerpo.value = cuerpo.value.slice(0, desde) + marca + cuerpo.value.slice(hasta)
  // El cursor detrás de lo insertado: quien escribe sigue escribiendo.
  void Promise.resolve().then(() => {
    area.focus()
    area.setSelectionRange(desde + marca.length, desde + marca.length)
  })
}

/**
 * Las variables desconocidas, escritas como se ven en el texto.
 *
 * Vive aquí y no en la plantilla del componente porque **las dos llaves de
 * apertura rompen el analizador de Vue**: dentro de una interpolación las lee
 * como el principio de otra. Es un detalle de sintaxis, no de diseño, pero
 * mueve código y conviene que esté dicho.
 */
function comoSeVen(variables: readonly string[]): string {
  const abre = '{'.repeat(2)
  const cierra = '}'.repeat(2)
  return variables.map((v) => `${abre}${v}${cierra}`).join(' · ')
}

async function guardar(): Promise<void> {
  const hecho = esPlantilla.value
    ? await store.guardarPlantilla(nombre.value, asunto.value, cuerpo.value)
    : await store.guardarFirma(nombre.value, cuerpo.value, porDefecto.value)
  if (hecho) cerrar()
}
</script>

<template>
  <section class="pantalla">
    <header class="cabecera">
      <div>
        <h1>{{ t('plantillas.titulo') }}</h1>
        <p class="explicacion">
          {{ t('plantillas.explicacion') }}
        </p>
      </div>
      <div
        v-if="editando === null"
        class="acciones"
      >
        <ABoton
          variante="primario"
          @click="abrir('plantilla')"
        >
          {{ t('plantillas.nuevaPlantilla') }}
        </ABoton>
        <ABoton
          variante="secundario"
          @click="abrir('firma')"
        >
          {{ t('plantillas.nuevaFirma') }}
        </ABoton>
      </div>
    </header>

    <AAviso
      v-if="problema"
      tono="peligro"
    >
      <strong>{{ problema.que }}</strong>
      <p>{{ problema.como }}</p>
      <p>{{ problema.salvo }}</p>
    </AAviso>

    <EstadoCargando v-if="store.cargando" />

    <!-- ── El editor ── -->
    <AModulo
      v-if="editando !== null"
      :titulo="esPlantilla ? t('plantillas.nuevaPlantilla') : t('plantillas.nuevaFirma')"
    >
      <div class="campos">
        <AEntrada
          v-model="nombre"
          :etiqueta="t('plantillas.nombre')"
          :ayuda="t('plantillas.nombreAyuda')"
        />
        <AEntrada
          v-if="esPlantilla"
          v-model="asunto"
          :etiqueta="t('plantillas.asunto')"
          :ayuda="t('plantillas.asuntoAyuda')"
        />

        <div class="cuerpo">
          <label
            class="etiqueta"
            for="cuerpo-plantilla"
          >{{ t('plantillas.cuerpo') }}</label>

          <!-- Las variables, con botón. Escribirlas a mano es de donde salen
               los `{{nombe}}` que llegan impresos al correo de la gente. -->
          <div class="variables">
            <span class="ayuda">{{ t('plantillas.insertar') }}</span>
            <ABoton
              v-for="v in VARIABLES"
              :key="v"
              variante="sutil"
              @click="insertar(v)"
            >
              {{ t(`plantilla.variable.${v}`) }}
            </ABoton>
          </div>

          <textarea
            id="cuerpo-plantilla"
            ref="areaDelCuerpo"
            v-model="cuerpo"
            class="area"
            rows="12"
            :placeholder="t('plantillas.cuerpoMarcador')"
          />
          <p class="ayuda">
            {{ t('plantillas.cuerpoAyuda') }}
          </p>
        </div>

        <label
          v-if="!esPlantilla"
          class="casilla"
        >
          <input
            v-model="porDefecto"
            type="checkbox"
          >
          {{ t('plantillas.porDefecto') }}
        </label>
      </div>

      <!-- Lo que se va a necesitar de cada contacto. §94: cifras. -->
      <p
        v-if="variablesUsadas.length > 0"
        class="nota"
      >
        {{ t('plantillas.usaVariables', { cuantas: variablesUsadas.length }) }}
      </p>

      <!-- La que salva un envío: una variable mal escrita no falla en ningún
           sitio, sale impresa tal cual en el correo. -->
      <AAviso
        v-if="variablesQueNoExisten.length > 0"
        tono="peligro"
      >
        <strong>{{ t('plantillas.variableQueNoExiste') }}</strong>
        <p>{{ comoSeVen(variablesQueNoExisten) }}</p>
        <p>{{ t('plantillas.variableQueNoExisteAyuda') }}</p>
      </AAviso>

      <div class="acciones">
        <ABoton
          variante="primario"
          :deshabilitado="!puedeGuardar || store.trabajando"
          @click="guardar"
        >
          {{ t('plantillas.guardar') }}
        </ABoton>
        <ABoton
          variante="secundario"
          @click="cerrar"
        >
          {{ t('plantillas.cancelar') }}
        </ABoton>
      </div>
    </AModulo>

    <template v-else>
      <!-- ── Las plantillas ── -->
      <h2>{{ t('plantillas.tituloPlantillas') }}</h2>
      <EstadoVacio
        v-if="store.plantillas.length === 0"
        :titulo="t('plantillas.vacioPlantillasTitulo')"
        :cuerpo="t('plantillas.vacioPlantillasCuerpo')"
      />
      <AModulo
        v-for="p in store.plantillas"
        :key="p.id"
        :titulo="p.nombre"
      >
        <template v-if="p.version">
          <p class="asunto">
            {{ p.version.asunto }}
          </p>
          <div class="fila">
            <AInsignia tono="neutro">
              {{ t('plantillas.version', { numero: p.version.numero }) }}
            </AInsignia>
            <AInsignia
              v-for="v in p.version.variables"
              :key="v"
              tono="neutro"
            >
              {{ t(`plantilla.variable.${v}`) }}
            </AInsignia>
          </div>
        </template>
        <p
          v-else
          class="ayuda"
        >
          {{ t('plantillas.sinVersion') }}
        </p>
      </AModulo>

      <!-- ── Las firmas ── -->
      <h2>{{ t('plantillas.tituloFirmas') }}</h2>
      <EstadoVacio
        v-if="store.firmas.length === 0"
        :titulo="t('plantillas.vacioFirmasTitulo')"
        :cuerpo="t('plantillas.vacioFirmasCuerpo')"
      />
      <AModulo
        v-for="f in store.firmas"
        :key="f.id"
        :titulo="f.nombre"
      >
        <AInsignia
          v-if="f.porDefecto"
          tono="exito"
        >
          {{ t('plantillas.esLaPorDefecto') }}
        </AInsignia>
      </AModulo>
    </template>
  </section>
</template>

<style scoped>
/* §17: todo sale de los tokens. */
.pantalla {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.cabecera {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--arles-space-4);
}

.explicacion,
.ayuda {
  color: var(--arles-text-muted);
}

.ayuda {
  font-size: var(--arles-font-size-small);
}

.campos {
  display: grid;
  gap: var(--arles-space-4);
}

.cuerpo {
  display: grid;
  gap: var(--arles-space-2);
}

.etiqueta {
  font-weight: var(--arles-font-weight-semibold);
}

.variables {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--arles-space-2);
}

.area {
  width: 100%;
  padding: var(--arles-space-3);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  background: var(--arles-surface);
  color: var(--arles-text);
  line-height: var(--arles-line-height-body);
  font-family: inherit;
  font-size: var(--arles-font-size-small);
  resize: vertical;
}

.casilla {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-4);
}

.fila {
  display: flex;
  flex-wrap: wrap;
  gap: var(--arles-space-2);
}

.asunto {
  margin-bottom: var(--arles-space-3);
}

.nota {
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
  margin-top: var(--arles-space-3);
}
</style>
