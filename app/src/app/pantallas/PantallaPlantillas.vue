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
 * **No reescribe una versión existente.** Una versión de plantilla no se toca
 * (6.3). Lo que «Corregir» hace es **dejar nacer la siguiente**, y la pantalla
 * lo dice con esas palabras: ver abajo.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * R-06 · «NO SE PUEDEN EDITAR LAS PLANTILLAS NI LAS FIRMAS» (16/10/2026)
 *
 * La cabecera de esta pantalla decía que enseñar un botón de editar «sería
 * prometer algo que el modelo de datos impide a propósito». **Era falso en los
 * dos casos**, y lo descubrió Dirección al necesitar arreglar una errata:
 *
 * - `nueva_version_de_plantilla` estaba en `arles-db` desde la 6.3, probada, y
 *   sin nadie que la llamara ni registrada en la frontera. El modelo de datos
 *   no impedía corregir una plantilla: **impedía reescribir lo ya enviado**,
 *   que es otra cosa.
 * - Una firma no tiene versiones, así que ahí no había ni siquiera eso: nadie
 *   había escrito el `UPDATE`.
 *
 * Y la consecuencia era peor de lo que parece: una firma con una errata —o con
 * un `{{nombre}}`, que es R-04— **no se podía arreglar nunca**.
 *
 * Los dos botones dicen cosas distintas a propósito. En una plantilla,
 * «Corregir» nace una versión nueva y **lo que está en cola sigue saliendo con
 * el texto anterior**: creerlo al revés es el error caro, así que se avisa
 * antes de guardar. En una firma se reescribe encima, y no hay nada congelado.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import type { Firma, Plantilla } from '@/app/stores/campanas'
import { useCampanasStore } from '@/app/stores/campanas'
import { useWhatsappStore } from '@/app/stores/whatsapp'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModal,
  AModulo,
  ASelector,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const store = useCampanasStore()
const whatsapp = useWhatsappStore()
const { t } = useI18n()

onMounted(() => {
  void store.cargar()
  // Los números, para la sección de plantillas de Meta: una plantilla es
  // **del número** —la clave de Meta es (número, nombre, idioma)— así que sin
  // saber de cuál no hay nada que listar.
  void whatsapp.cargar()
})

// ── LAS PLANTILLAS DE META (D-WA, 27/10/2026) ───────────────────────────────
//
// `Db::anotar_plantilla_de_meta` y `Db::plantillas_de_meta` existían desde la
// 6.8, con sus pruebas, **y no las llamaba nadie**: ni comando ni pantalla. Así
// que `CLAUDE.md` afirmaba de más al decir que la 6.8 cerró «el registro de las
// plantillas aprobadas de Meta».
//
// Lo que eso costaba se ve al llegar al asistente: el paso 4, al elegir
// WhatsApp, sería un desplegable vacío sin ninguna pista de dónde se registran
// — el H-06 del 22/10/2026, otra vez y en el mismo paso.
//
// ── POR QUÉ SE REGISTRAN A MANO Y NO SE LEEN DE META ──
//
// Leerlas de Meta exigiría un permiso más y una consulta por número cada vez
// que se abre esta pantalla, y el dato que de verdad importa —qué variable de
// ARLES va en cada hueco— **Meta no lo sabe**: sus huecos son `{{1}}` y `{{2}}`,
// sin nombre. Eso lo decide quien escribió la plantilla, y es lo que se apunta
// aquí.
const numeroElegido = ref('')

/** Los números que se pueden elegir, con su teléfono para reconocerlos. */
const opcionesDeNumero = computed(() =>
  whatsapp.numeros.map((n) => ({
    valor: n.id,
    texto: `${n.nombreVisible} · ${n.telefonoE164}`,
  })),
)

/**
 * Los cinco estados de Meta, tal como los nombra su API.
 *
 * Se ofrecen los cinco y no sólo «aprobada»: quien registra una plantilla que
 * está en revisión necesita poder apuntarla para no perderla de vista, y el
 * paso de activar ya rechaza las que no están aprobadas (tramo 3a). Ofrecer
 * sólo «aprobada» obligaría a mentir para poder guardar.
 */
const ESTADOS_DE_META = ['approved', 'pending', 'rejected', 'paused', 'disabled'] as const

const nuevaDeMeta = ref({
  nombre: '',
  idioma: 'es_MX',
  estadoEnMeta: 'approved',
  cuerpo: '',
  huecos: [] as string[],
})

const opcionesDeEstadoDeMeta = computed(() =>
  ESTADOS_DE_META.map((e) => ({ valor: e, texto: t(`plantillas.meta.estado.${e}`) })),
)

/** Cuántos huecos `{{n}}` tiene el texto que se pegó. */
const huecosDelCuerpo = computed(
  () => (nuevaDeMeta.value.cuerpo.match(/\{\{\s*\d+\s*\}\}/g) ?? []).length,
)

/**
 * Si se puede guardar.
 *
 * **Y los huecos tienen que cuadrar.** Meta manda los valores por posición, así
 * que una plantilla con dos `{{n}}` y una sola variable apuntada mandaría el
 * segundo hueco vacío en cada mensaje de la campaña — sin que nada fallara.
 */
const puedeGuardarDeMeta = computed(
  () =>
    numeroElegido.value.length > 0 &&
    nuevaDeMeta.value.nombre.trim().length > 0 &&
    nuevaDeMeta.value.idioma.trim().length > 0 &&
    nuevaDeMeta.value.cuerpo.trim().length > 0 &&
    nuevaDeMeta.value.huecos.length === huecosDelCuerpo.value,
)

watch(numeroElegido, (id) => {
  if (id.length > 0) void whatsapp.cargarPlantillasDeMeta(id)
})

function ponerHueco(clave: string): void {
  nuevaDeMeta.value.huecos = [...nuevaDeMeta.value.huecos, clave]
}

function quitarUltimoHueco(): void {
  nuevaDeMeta.value.huecos = nuevaDeMeta.value.huecos.slice(0, -1)
}

async function guardarDeMeta(): Promise<void> {
  if (!puedeGuardarDeMeta.value) return
  const ok = await whatsapp.anotarPlantillaDeMeta({
    numero: numeroElegido.value,
    nombre: nuevaDeMeta.value.nombre.trim(),
    idioma: nuevaDeMeta.value.idioma.trim(),
    estadoEnMeta: nuevaDeMeta.value.estadoEnMeta,
    cuerpo: nuevaDeMeta.value.cuerpo,
    huecos: nuevaDeMeta.value.huecos,
  })
  if (ok) {
    nuevaDeMeta.value = {
      nombre: '',
      idioma: 'es_MX',
      estadoEnMeta: 'approved',
      cuerpo: '',
      huecos: [],
    }
  }
}

/** El tono de la insignia del estado de Meta. Sólo «aprobada» es buena. */
function tonoDelEstadoDeMeta(estado: string): 'exito' | 'aviso' | 'peligro' | 'neutro' {
  if (estado === 'approved') return 'exito'
  if (estado === 'pending') return 'aviso'
  if (estado === 'rejected' || estado === 'disabled') return 'peligro'
  return 'neutro'
}

/**
 * Las cinco variables del núcleo (`arles_core::plantilla::Variable`).
 *
 * `baja` entró el 22/10/2026 con el pie de baja obligatorio (R-7): es la única
 * forma de que el correo le dé a quien lo recibe una salida con **la dirección
 * del remitente de esa campaña**, y `activar_campana` rechaza una campaña cuyo
 * correo no la tenga ni en la plantilla ni en la firma.
 *
 * Esta lista está escrita a mano y es el duplicado que el validador vigila: la
 * comprobación del §139 ata `clave()` con `clave_i18n()` en Rust, y el catálogo
 * tiene que tener texto para las cinco — lo comprueba `catalogo.spec.ts`
 * leyendo las claves **del código Rust**, así que una variante nueva sin texto
 * sale en rojo.
 */
const VARIABLES = ['nombre', 'apellido', 'empresa', 'mi_empresa', 'baja'] as const

/**
 * Las que habla de **quien recibe** el correo.
 *
 * ── R-04, 16/10/2026 ───────────────────────────────────────────────────────
 *
 * Dirección: *«sería una tragedia que apareciera en la firma el nombre del
 * contacto a quien se le está haciendo llegar el correo y su empresa, en lugar
 * de los datos del remitente»*.
 *
 * Y este formulario sirve para las dos cosas —plantilla y firma— y ofrecía las
 * cuatro en los dos casos. Una firma con `{{nombre}}` pone el de quien recibe
 * **debajo del mensaje, en cada correo de la campaña**.
 *
 * Esto **ayuda**: quita los botones que no tocan. Lo que **impide** es
 * `revisar_firma` en el núcleo, que para el guardado aunque se escriban a
 * mano. Las dos mitades hacen falta: sin la de aquí se tropieza, sin la de
 * allí se cuela.
 */
const DE_QUIEN_RECIBE = ['nombre', 'apellido', 'empresa'] as const

/** Las que se ofrecen según lo que se esté escribiendo. */
const variablesQueSeOfrecen = computed(() =>
  editando.value === 'firma'
    ? VARIABLES.filter((v) => !DE_QUIEN_RECIBE.includes(v as (typeof DE_QUIEN_RECIBE)[number]))
    : VARIABLES,
)

/**
 * `true` si lo que se está escribiendo **no** da una salida de baja.
 *
 * ── AVISA Y NO BLOQUEA, Y LA RAZÓN ESTÁ EN DÓNDE SE DECIDE ────────────────
 *
 * Quien **impide** es `activar_campana`, que es el único sitio donde se saben
 * las dos cosas: la plantilla y la firma que esa campaña va a usar. La salida
 * puede venir de cualquiera de las dos, así que exigirla aquí obligaría a
 * repetirla en todas las plantillas aunque la firma ya la traiga — y entonces
 * cada correo diría dos veces lo mismo.
 *
 * Esto es la mitad de arriba: que no se descubra al activar, después de nueve
 * pasos. Es el mismo reparto que R-04 (`ajenasEnLaFirma` ayuda,
 * `revisar_firma` impide) y la misma razón: sin la de aquí se tropieza, sin la
 * de allí se cuela.
 */
const sinSalidaDeBaja = computed(() => !variablesUsadas.value.includes('baja'))

/** Las de quien recibe que se hayan escrito a mano en una firma. */
const ajenasEnLaFirma = computed(() =>
  editando.value === 'firma'
    ? variablesUsadas.value.filter((v) =>
        DE_QUIEN_RECIBE.includes(v as (typeof DE_QUIEN_RECIBE)[number]),
      )
    : [],
)

type Cual = 'plantilla' | 'firma' | null
const editando = ref<Cual>(null)

/**
 * El identificador de lo que se está corrigiendo, o `null` si es nuevo.
 *
 * Es lo único que distingue los cuatro caminos de guardado, y por eso se
 * limpia en `abrir()`: dejarlo puesto crearía una versión de la plantilla que
 * se corrigió la vez anterior.
 */
const idEditado = ref<string | null>(null)

/** La versión vigente de la plantilla que se corrige, para decir cuál nace. */
const versionVigente = ref<number | null>(null)

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
    variablesQueNoExisten.value.length === 0 &&
    // El núcleo las rechaza igual (R-04), pero dejar pulsar «Guardar» para
    // que el guardado falle es hacer escribir la firma entera para enterarse.
    // El aviso de abajo queda justo encima del botón, así que el botón
    // apagado dice por qué (§95).
    ajenasEnLaFirma.value.length === 0,
)

function abrir(cual: Exclude<Cual, null>): void {
  editando.value = cual
  idEditado.value = null
  versionVigente.value = null
  nombre.value = ''
  asunto.value = ''
  cuerpo.value = ''
  porDefecto.value = false
}

/**
 * Abre el editor con lo que la plantilla dice hoy.
 *
 * Precargarlo no es comodidad: corregir una errata de una frase sin tener el
 * texto delante significa reescribirlo entero, y entonces «corregir» es
 * «volver a redactar», que es lo que Dirección no podía hacer.
 */
function corregirPlantilla(p: Plantilla): void {
  editando.value = 'plantilla'
  idEditado.value = p.id
  versionVigente.value = p.version?.numero ?? null
  nombre.value = p.nombre
  asunto.value = p.version?.asunto ?? ''
  cuerpo.value = p.version?.cuerpoHtml ?? ''
  porDefecto.value = false
}

function corregirFirma(f: Firma): void {
  editando.value = 'firma'
  idEditado.value = f.id
  versionVigente.value = null
  nombre.value = f.nombre
  asunto.value = ''
  cuerpo.value = f.cuerpoHtml
  porDefecto.value = f.porDefecto
}

function cerrar(): void {
  editando.value = null
  idEditado.value = null
  versionVigente.value = null
}

/**
 * Lo que se va a retirar, o `null`. · R-08 (21/10/2026)
 *
 * **Se retira, no se borra.** Deja de ofrecerse al crear una campaña, y el
 * texto se queda en la base: lo que una campaña envió tiene que poder
 * consultarse dentro de un año.
 *
 * Se pregunta igualmente, porque una firma retirada deja de ser la de por
 * defecto y eso cambia lo que van a mandar las campañas siguientes.
 */
const borrando = ref<{ cual: 'plantilla' | 'firma'; id: string; nombre: string } | null>(null)

/** `true` si lo que se va a borrar es la firma marcada por defecto. */
const esLaPorDefecto = computed(
  () =>
    borrando.value?.cual === 'firma' &&
    store.firmas.some((f) => f.id === borrando.value?.id && f.porDefecto),
)

async function confirmarBorrado(): Promise<void> {
  const q = borrando.value
  if (q === null) return
  const hecho =
    q.cual === 'plantilla' ? await store.retirarPlantilla(q.id) : await store.retirarFirma(q.id)
  // Sólo se cierra si se borró: con un error puesto, cerrar el diálogo haría
  // desaparecer el aviso junto con la pregunta (§95).
  if (hecho) borrando.value = null
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
 * Pone la selección en negrita o en cursiva · R-03 (17/10/2026)
 *
 * ── POR QUÉ UN BOTÓN Y NO UN EDITOR VISUAL ────────────────────────────────
 *
 * Dirección pidió «poder colocar texto en negrita o cursiva». El cuerpo es
 * HTML y el saneado del núcleo ya acepta `strong`, `b`, `em`, `i`, `u`, listas
 * y enlaces desde la 6.2 — lo que no había era **forma de ponerlos** sin
 * escribir las etiquetas a mano, que es pedirle HTML a quien no lo escribe.
 *
 * Un editor visual (`contenteditable`) sería el camino largo: produce el HTML
 * que le dé la gana el navegador —`<span style="font-weight:700">` en algunos—
 * y eso el saneado lo tira, así que el texto se vería bien al escribirlo y
 * saldría plano al guardarlo. El botón es la misma solución que las variables,
 * por el mismo motivo: lo que acaba en el texto lo pone el programa.
 *
 * **Sin selección no hace nada en silencio**: inserta el par de etiquetas y
 * deja el cursor dentro, que es lo que hace falta para empezar a escribir en
 * negrita. Un botón que no responde se lee como una avería (§95).
 */
function envolver(etiqueta: 'strong' | 'em'): void {
  const abre = `<${etiqueta}>`
  const cierra = `</${etiqueta}>`
  const area = areaDelCuerpo.value
  if (!area) {
    cuerpo.value += `${abre}${cierra}`
    return
  }
  const desde = area.selectionStart
  const hasta = area.selectionEnd
  const dentro = cuerpo.value.slice(desde, hasta)
  cuerpo.value =
    cuerpo.value.slice(0, desde) + abre + dentro + cierra + cuerpo.value.slice(hasta)

  // El cursor detrás de lo marcado, o en medio del par si no había nada
  // seleccionado: en los dos casos se sigue escribiendo donde se espera.
  const nuevo = desde + abre.length + dentro.length
  void Promise.resolve().then(() => {
    area.focus()
    area.setSelectionRange(nuevo, nuevo)
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

/** Cuatro caminos: crear o corregir, plantilla o firma. */
async function guardar(): Promise<void> {
  const id = idEditado.value
  let hecho: boolean
  if (esPlantilla.value) {
    hecho =
      id === null
        ? await store.guardarPlantilla(nombre.value, asunto.value, cuerpo.value)
        : await store.nuevaVersionDePlantilla(id, asunto.value, cuerpo.value)
  } else {
    hecho =
      id === null
        ? await store.guardarFirma(nombre.value, cuerpo.value, porDefecto.value)
        : await store.editarFirma(id, nombre.value, cuerpo.value, porDefecto.value)
  }
  if (hecho) cerrar()
}

/** El rótulo del editor, que no es el mismo en los cuatro casos. */
const tituloDelEditor = computed(() => {
  if (esPlantilla.value) {
    return idEditado.value === null
      ? t('plantillas.nuevaPlantilla')
      : t('plantillas.corregirPlantilla')
  }
  return idEditado.value === null ? t('plantillas.nuevaFirma') : t('plantillas.corregirFirma')
})
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
      :titulo="tituloDelEditor"
    >
      <!-- ── R-06 · lo que corregir una plantilla hace y lo que NO hace ──
           Creer que esto arregla la campaña que está saliendo es el error
           caro, y es el que la forma de los datos invita a cometer. Así que
           se dice antes de guardar, con la cifra de la versión que nace. -->
      <AAviso
        v-if="esPlantilla && idEditado !== null"
        tono="info"
      >
        <strong>{{
          t('plantillas.naceUnaVersion', { numero: (versionVigente ?? 0) + 1 })
        }}</strong>
        <p>{{ t('plantillas.naceUnaVersionAyuda') }}</p>
      </AAviso>

      <div class="campos">
        <AEntrada
          v-model="nombre"
          :etiqueta="t('plantillas.nombre')"
          :ayuda="
            esPlantilla && idEditado !== null
              ? t('plantillas.nombreNoCambia')
              : t('plantillas.nombreAyuda')
          "
          :solo-lectura="esPlantilla && idEditado !== null"
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

          <!-- R-03 · negrita y cursiva. El saneado las acepta desde la 6.2;
               lo que faltaba era cómo ponerlas sin escribir las etiquetas. -->
          <div class="variables">
            <span class="ayuda">{{ t('plantillas.formato') }}</span>
            <ABoton
              variante="sutil"
              @click="envolver('strong')"
            >
              {{ t('plantillas.negrita') }}
            </ABoton>
            <ABoton
              variante="sutil"
              @click="envolver('em')"
            >
              {{ t('plantillas.cursiva') }}
            </ABoton>
          </div>

          <!-- Las variables, con botón. Escribirlas a mano es de donde salen
               los `{{nombe}}` que llegan impresos al correo de la gente. -->
          <div class="variables">
            <span class="ayuda">{{ t('plantillas.insertar') }}</span>
            <ABoton
              v-for="v in variablesQueSeOfrecen"
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
      <!-- ── R-04 · datos de quien RECIBE en una firma ──────────────────
           Los botones ya no las ofrecen, pero se pueden escribir a mano o
           venir de una firma vieja. Y el daño es grande: el nombre de quien
           recibe, debajo del mensaje, en cada correo de la campaña. -->
      <AAviso
        v-if="ajenasEnLaFirma.length > 0"
        tono="peligro"
      >
        <strong>{{ t('plantillas.firmaConDatosAjenos') }}</strong>
        <p>{{ comoSeVen(ajenasEnLaFirma) }}</p>
        <p>{{ t('plantillas.firmaConDatosAjenosAyuda') }}</p>
      </AAviso>

      <!-- ── R-7 · la salida de quien recibe ────────────────────────────
           Avisa y no bloquea: la salida puede venir de la firma, y aquí no se
           sabe cuál se va a usar. Quien impide es `activar_campana`, que sí lo
           sabe. Esto es para que no se descubra al final de nueve pasos.

           Tono de aviso y no de peligro: lo que hay delante no está roto —
           puede que la firma lo traiga—. El peligro es el error del activado,
           que sí lo es. -->
      <AAviso
        v-if="sinSalidaDeBaja"
        tono="aviso"
      >
        <strong>{{ t('plantillas.sinSalidaDeBaja') }}</strong>
        <p>{{ t('plantillas.sinSalidaDeBajaAyuda') }}</p>
      </AAviso>

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
        <div class="acciones">
          <ABoton
            variante="secundario"
            :deshabilitado="p.version === null"
            @click="corregirPlantilla(p)"
          >
            {{ t('plantillas.corregir') }}
          </ABoton>
          <ABoton
            variante="peligro"
            @click="borrando = { cual: 'plantilla', id: p.id, nombre: p.nombre }"
          >
            {{ t('plantillas.borrar') }}
          </ABoton>
        </div>
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
        <div class="acciones">
          <ABoton
            variante="secundario"
            @click="corregirFirma(f)"
          >
            {{ t('plantillas.corregir') }}
          </ABoton>
          <ABoton
            variante="peligro"
            @click="borrando = { cual: 'firma', id: f.id, nombre: f.nombre }"
          >
            {{ t('plantillas.borrar') }}
          </ABoton>
        </div>
      </AModulo>

      <!-- ── Las plantillas de Meta (D-WA) ────────────────────────────────
           ARLES **no las crea**: el texto lo aprueba Meta palabra por palabra
           en su panel. Esto es el apunte de que existe, para poder ofrecerla
           al crear una campaña y para poder leer lo que va a salir.

           Y el dato que de verdad importa —qué variable de ARLES va en cada
           hueco— **Meta no lo sabe**: sus huecos son `{{1}}` y `{{2}}`, sin
           nombre. Eso lo decide quien escribió la plantilla. -->
      <h2>{{ t('plantillas.meta.titulo') }}</h2>
      <p class="ayuda">
        {{ t('plantillas.meta.explicacion') }}
      </p>

      <AAviso
        v-if="opcionesDeNumero.length === 0"
        tono="info"
        :titulo="t('plantillas.meta.sinNumeros')"
      >
        <p>{{ t('plantillas.meta.sinNumerosComo') }}</p>
        <ABoton
          variante="secundario"
          @click="$router.push({ name: 'whatsapp' })"
        >
          {{ t('plantillas.meta.irAWhatsApp') }}
        </ABoton>
      </AAviso>

      <template v-else>
        <ASelector
          v-model="numeroElegido"
          :etiqueta="t('plantillas.meta.numero')"
          :opciones="opcionesDeNumero"
          :ayuda="t('plantillas.meta.numeroAyuda')"
        />

        <template v-if="numeroElegido">
          <EstadoVacio
            v-if="whatsapp.plantillasDeMeta.length === 0"
            :titulo="t('plantillas.meta.vacioTitulo')"
            :cuerpo="t('plantillas.meta.vacioCuerpo')"
          />
          <AModulo
            v-for="p in whatsapp.plantillasDeMeta"
            :key="p.id"
            :titulo="p.nombre"
          >
            <div class="fila">
              <AInsignia :tono="tonoDelEstadoDeMeta(p.estado)">
                {{ t(`plantillas.meta.estado.${p.estado}`) }}
              </AInsignia>
              <AInsignia tono="neutro">
                {{ p.idioma }}
              </AInsignia>
              <AInsignia
                v-for="(h, i) in p.huecos"
                :key="`${p.id}-${i}`"
                tono="neutro"
              >
                {{ i + 1 }} · {{ t(`plantilla.variable.${h}`) }}
              </AInsignia>
            </div>
            <p class="asunto">
              {{ p.cuerpo }}
            </p>
          </AModulo>

          <!-- El formulario de apunte. Va debajo de la lista para que lo
               primero que se vea sea lo que ya hay: quien vuelve aquí viene
               casi siempre a comprobar un estado, no a añadir. -->
          <AModulo :titulo="t('plantillas.meta.anotar')">
            <AEntrada
              v-model="nuevaDeMeta.nombre"
              :etiqueta="t('plantillas.meta.nombre')"
              :ayuda="t('plantillas.meta.nombreAyuda')"
            />
            <AEntrada
              v-model="nuevaDeMeta.idioma"
              :etiqueta="t('plantillas.meta.idioma')"
              :ayuda="t('plantillas.meta.idiomaAyuda')"
            />
            <ASelector
              v-model="nuevaDeMeta.estadoEnMeta"
              :etiqueta="t('plantillas.meta.estadoEtiqueta')"
              :opciones="opcionesDeEstadoDeMeta"
              :ayuda="t('plantillas.meta.estadoAyuda')"
            />

            <div class="campo">
              <label for="cuerpo-de-meta">{{ t('plantillas.meta.cuerpo') }}</label>
              <textarea
                id="cuerpo-de-meta"
                v-model="nuevaDeMeta.cuerpo"
                class="area"
                rows="6"
                :placeholder="t('plantillas.meta.cuerpoMarcador')"
              />
              <p class="ayuda">
                {{ t('plantillas.meta.cuerpoAyuda') }}
              </p>
            </div>

            <!-- Los huecos, EN ORDEN. Meta manda los valores por posición, así
                 que el orden no es cosmético: es a quién se le escribe. -->
            <div class="campo">
              <label>{{ t('plantillas.meta.huecos', { n: huecosDelCuerpo }) }}</label>
              <p class="ayuda">
                {{ t('plantillas.meta.huecosAyuda') }}
              </p>
              <div class="fila">
                <AInsignia
                  v-for="(h, i) in nuevaDeMeta.huecos"
                  :key="`puesto-${i}`"
                  tono="info"
                >
                  {{ i + 1 }} · {{ t(`plantilla.variable.${h}`) }}
                </AInsignia>
              </div>
              <div class="fila">
                <ABoton
                  v-for="v in VARIABLES"
                  :key="v"
                  variante="sutil"
                  :deshabilitado="nuevaDeMeta.huecos.length >= huecosDelCuerpo"
                  @click="ponerHueco(v)"
                >
                  {{ t(`plantilla.variable.${v}`) }}
                </ABoton>
                <ABoton
                  variante="sutil"
                  :deshabilitado="nuevaDeMeta.huecos.length === 0"
                  @click="quitarUltimoHueco"
                >
                  {{ t('plantillas.meta.quitarHueco') }}
                </ABoton>
              </div>
            </div>

            <ABoton
              variante="primario"
              :deshabilitado="!puedeGuardarDeMeta"
              :ocupado="whatsapp.cargando"
              @click="guardarDeMeta"
            >
              {{ t('plantillas.meta.guardar') }}
            </ABoton>
            <!-- §95: un botón apagado dice por qué. Es el defecto que la 8.2c
                 y la 8.3c tuvieron con «Repite la contraseña». -->
            <p
              v-if="!puedeGuardarDeMeta"
              class="ayuda"
            >
              {{
                nuevaDeMeta.huecos.length === huecosDelCuerpo
                  ? t('plantillas.meta.faltanDatos')
                  : t('plantillas.meta.huecosNoCuadran', {
                    puestos: nuevaDeMeta.huecos.length,
                    n: huecosDelCuerpo,
                  })
              }}
            </p>
          </AModulo>
        </template>
      </template>
    </template>

    <!-- ── R-08 · borrar, con su pregunta ──────────────────────────────── -->
    <AModal
      :abierto="borrando !== null"
      :titulo="
        borrando?.cual === 'plantilla'
          ? t('plantillas.borrarPlantillaTitulo')
          : t('plantillas.borrarFirmaTitulo')
      "
      destructiva
      @cerrar="borrando = null"
    >
      <p>{{ t('plantillas.borrarPregunta', { nombre: borrando?.nombre ?? '' }) }}</p>
      <p class="ayuda">
        {{
          borrando?.cual === 'plantilla'
            ? t('plantillas.borrarPlantillaAviso')
            : t('plantillas.borrarFirmaAviso')
        }}
      </p>

      <!-- La firma por defecto es la que usan las campañas cuando no se elige
           otra, así que quitarla cambia lo que van a mandar. -->
      <AAviso
        v-if="esLaPorDefecto"
        tono="aviso"
      >
        <strong>{{ t('plantillas.borrarLaPorDefecto') }}</strong>
        <p>{{ t('plantillas.borrarLaPorDefectoAyuda') }}</p>
      </AAviso>

      <AAviso
        v-if="problema"
        tono="peligro"
        urgente
      >
        <strong>{{ problema.que }}</strong>
        <p>{{ problema.como }}</p>
        <p>{{ problema.salvo }}</p>
      </AAviso>

      <template #acciones>
        <ABoton
          variante="secundario"
          @click="borrando = null"
        >
          {{ t('plantillas.cancelar') }}
        </ABoton>
        <ABoton
          variante="peligro"
          :ocupado="store.trabajando"
          @click="confirmarBorrado"
        >
          {{ t('plantillas.borrar') }}
        </ABoton>
      </template>
    </AModal>
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
