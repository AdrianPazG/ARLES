<script setup lang="ts">
import { numeroLegible } from '../formato'
/**
 * El número de WhatsApp desde el que ARLES escribe (entrega W.6).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LAS DOS CONFIRMACIONES NO ESTÁN EN EL FORMULARIO, Y ÉSE ES EL DISEÑO
 *
 * Coexistencia y la aceptación del riesgo se piden **después** de guardar, cada
 * una en su tarjeta y con su propio botón. Puestas como dos casillas del alta
 * se marcarían con el resto de los campos sin leerlas — que es exactamente lo
 * que estas dos marcas existen para impedir.
 *
 * Son la constancia de que a alguien se le avisó: una, que el número sale de la
 * aplicación de WhatsApp Business (L-11); otra, que escribir a quien no lo pidió
 * puede costar el número entero. Que cuesten un gesto más es deliberado.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA NO DECIDE
 *
 * **No decide si un número puede enviar.** `puedeEnviar` llega resuelto de Rust.
 * Recalcularlo aquí serían dos reglas que se separan en cuanto una cambie, y la
 * que decide si se envía no sería la que se ve.
 *
 * **No valida el teléfono.** Lo hace el núcleo, con el país de la empresa. Aquí
 * solo se pinta lo que devolvió.
 *
 * **No guarda el token.** Vive en el campo mientras se escribe, viaja en la
 * llamada y se borra al cerrar el formulario. No pasa por la tienda ni por
 * `localStorage` (ADR-0011).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LA CALIDAD SIN CONSULTAR NO SE PINTA EN VERDE
 *
 * `calidad === null` significa «no se sabe», no «está bien». Se enseña con tono
 * neutro y su explicación. Un semáforo que se pone en verde cuando no ha podido
 * mirar es peor que no tener semáforo — la misma regla del «sin comprobar» del
 * preflight de correo.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import { claveDelError, invocar } from '@/app/nucleo'
import {
  borradorDeNumeroVacio,
  revisarDatoDeMeta,
  useWhatsappStore,
  type BorradorDeNumero,
  type Numero,
  type NumeroDescubierto,
} from '@/app/stores/whatsapp'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModulo,
  ASelector,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const store = useWhatsappStore()
const { t } = useI18n()

onMounted(() => {
  void store.cargar()
})

/**
 * El error, ya partido en las tres cosas que el §95 exige que diga: qué pasó,
 * cómo se arregla y qué está a salvo. `resolverError` recibe el objeto de la
 * frontera, no la clave suelta.
 */
const problema = computed(() =>
  store.error ? resolverError({ clave: store.error, detalle: '' }) : null,
)

const formularioAbierto = ref(false)
const borrador = ref<BorradorDeNumero>(borradorDeNumeroVacio())

/**
 * El token, fuera del borrador a propósito.
 *
 * El borrador se puede pasar, copiar y —el día que a alguien le convenga—
 * guardar como último formulario usado. El token no debe poder colarse en nada
 * de eso, así que vive en su propia referencia y se borra a mano al cerrar.
 */
const token = ref('')

// ── EL DESCUBRIMIENTO ──
const buscando = ref(false)
const errorAlBuscar = ref('')
const descubiertos = ref<NumeroDescubierto[]>([])
const elegido = ref('')

/**
 * Le pregunta a Meta qué números tiene este token.
 *
 * El token **no se guarda aquí**: viaja al comando, se usa para preguntar y se
 * tira. Quien lo guarda es el llavero, y sólo al dar de alta el número.
 */
async function buscarMisNumeros(): Promise<void> {
  buscando.value = true
  // El error de la búsqueda anterior se borra ANTES de reintentar, no al
  // terminar bien: si no, el aviso de un intento fallido sobrevive al intento
  // que lo resuelve. Es el H-01 del 14/10/2026, en otra pantalla.
  errorAlBuscar.value = ''
  descubiertos.value = []
  elegido.value = ''
  try {
    descubiertos.value = await invocar<NumeroDescubierto[]>(
      'descubrir_numeros_de_whatsapp',
      { token: token.value },
    )
  } catch (e) {
    errorAlBuscar.value = claveDelError(e)
  } finally {
    buscando.value = false
  }
}

/**
 * Al cambiar el token, lo descubierto deja de valer.
 *
 * Sin esto, alguien pega otro token, no vuelve a buscar, y guarda el número
 * del token anterior con la credencial del nuevo — una cuenta que no puede
 * enviar y que nadie sabe por qué.
 */
function alCambiarElToken(): void {
  descubiertos.value = []
  elegido.value = ''
  errorAlBuscar.value = ''
}

/** Rellena el borrador con lo que dijo Meta. Nada de esto se teclea. */
function elegirNumero(n: NumeroDescubierto): void {
  borrador.value.wabaId = n.wabaId
  borrador.value.phoneNumberId = n.phoneNumberId
  borrador.value.telefono = n.telefono
  borrador.value.nombreVisible = n.nombre
}

/**
 * El tono de la calidad que Meta declara.
 *
 * `UNKNOWN` va en neutro y no en verde: un semáforo que se pone en verde
 * cuando no ha podido mirar es peor que no tener semáforo.
 */
function tonoDeLaCalidadDeMeta(
  calidad: string,
): 'exito' | 'aviso' | 'peligro' | 'neutro' {
  if (calidad === 'GREEN') return 'exito'
  if (calidad === 'YELLOW') return 'aviso'
  if (calidad === 'RED') return 'peligro'
  return 'neutro'
}

const MODOS = computed(() => [
  { valor: 'seguimiento', texto: t('whatsapp.modo.seguimiento') },
  { valor: 'prospeccion_directa', texto: t('whatsapp.modo.prospeccionDirecta') },
])

/**
 * El aviso del token, que es **el único dato que se pega a mano**.
 *
 * ── R-02 · de seis campos a uno, y por qué antes no se pudo ────────────────
 *
 * Dirección pidió que esto fuera tan sencillo como el alta de correo. La
 * respuesta de entonces fue que no se podía: el correo se deduce del dominio,
 * pero el WABA ID y el Phone Number ID **sólo existen en el panel de Meta** y
 * no hay nada de lo que inferirlos.
 *
 * Era cierto y estaba incompleto. No se pueden **deducir**, pero sí se pueden
 * **preguntar**: el token ya autoriza a Meta a decir qué números cuelgan de
 * él. Eso es D-WA (26/10/2026), y deja el alta en un dato y una elección.
 *
 * El aviso de pegado se queda para el token: empieza por `EAA`, y pegar ahí un
 * identificador numérico da un error de Meta que no dice cuál es el problema.
 */
const avisoToken = computed(() =>
  claveAviso(revisarDatoDeMeta('token', token.value)),
)

function claveAviso(clave: string): string {
  return clave ? t(clave) : ''
}

/** Avisa antes de guardar, no después: el modo en frío se elige sabiéndolo. */
const elegidoEnFrio = computed(() => borrador.value.modo === 'prospeccion_directa')

/**
 * Se guarda cuando hay token Y un número elegido de la lista.
 *
 * Se mira `elegido` y no los campos del borrador: los campos se rellenan al
 * elegir, así que mirarlos diría lo mismo — hasta que alguien cambie el token
 * sin volver a buscar. Entonces el borrador sigue lleno y lo elegido ya no
 * vale, que es justo lo que `alCambiarElToken` limpia.
 */
const puedeGuardar = computed(
  () => token.value.trim().length > 0 && elegido.value.length > 0,
)

function abrir(): void {
  borrador.value = borradorDeNumeroVacio()
  token.value = ''
  alCambiarElToken()
  formularioAbierto.value = true
}

function cerrar(): void {
  formularioAbierto.value = false
  borrador.value = borradorDeNumeroVacio()
  // El token se borra aquí y no solo al abrir: si alguien cancela y no vuelve
  // a entrar, no tiene por qué seguir en memoria.
  token.value = ''
}

async function guardar(): Promise<void> {
  if (await store.guardar(borrador.value, token.value)) cerrar()
}

/** La calidad, o el texto de «no consultada» — nunca un hueco. */
function textoDeCalidad(n: Numero): string {
  return n.calidad === null
    ? t('numeroDeWhatsapp.calidad.sinConsultar')
    : t(`numeroDeWhatsapp.calidad.${n.calidad}`)
}

/**
 * El envío de prueba, por número (el hueco que cerró la 7.7).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * EL ESTADO VA POR NÚMERO Y NO EN UNA SOLA VARIABLE
 *
 * Hay tantas tarjetas como números dados de alta. Con un único `destino` y un
 * único `resultado`, escribir en una tarjeta escribiría en todas y el resultado
 * de una prueba aparecería bajo el número equivocado — que en esta pantalla
 * significa creer que un número funciona cuando el que funciona es otro.
 */
/** Lo que se escribe para probar un número. */
interface FormularioDePrueba {
  para: string
  plantilla: string
  idioma: string
}

const resultado = ref<Record<string, { aceptado: boolean; clave: string }>>({})
const probando = ref<string | null>(null)

/**
 * La plantilla con la que Meta deja escribir primero sin aprobar nada.
 *
 * Viene aprobada con el número de pruebas que Meta regala, y por eso el envío
 * de prueba **no depende** del registro de plantillas aprobadas, que todavía no
 * tiene pantalla.
 */
const PLANTILLA_DE_META = 'hello_world'
const IDIOMA_DE_META = 'en_US'

/**
 * Cada número con su formulario de prueba al lado.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * POR QUÉ UNA LISTA DE PARES Y NO TRES MAPAS POR IDENTIFICADOR
 *
 * Tres razones, y las tres salieron de intentarlo de la otra forma:
 *
 * 1. **El teléfono escrito no se pierde.** Confirmar coexistencia recarga la
 *    lista de números; con el formulario fuera, lo escrito sobreviviría sólo por
 *    casualidad. Aquí se conserva por identificador a propósito.
 * 2. **El tipo sale bien.** Con `exactOptionalPropertyTypes` un acceso por
 *    índice es `string | undefined`, que `AEntrada` no acepta. Un par lleva su
 *    objeto concreto, y `f.para` es una cadena.
 * 3. **Es por número.** Con un único `destino`, escribir en una tarjeta
 *    escribiría en todas, y el resultado de una prueba aparecería bajo el número
 *    equivocado — que aquí significa creer que funciona un número que no es.
 *
 * Se rehace con un `watch` y no en `onMounted` porque los números también
 * aparecen al guardar uno nuevo, sin volver a montar nada.
 */
const tarjetas = ref<Array<{ n: Numero; f: FormularioDePrueba }>>([])

watch(
  () => store.numeros,
  (ns) => {
    tarjetas.value = ns.map((n) => ({
      n,
      f: tarjetas.value.find((t) => t.n.id === n.id)?.f ?? {
        para: '',
        plantilla: PLANTILLA_DE_META,
        idioma: IDIOMA_DE_META,
      },
    }))
  },
  { immediate: true, deep: true },
)

async function probar(n: Numero, f: FormularioDePrueba): Promise<void> {
  probando.value = n.id
  // El resultado anterior se borra antes de pedir el nuevo: dejarlo en pantalla
  // mientras se manda haría leer el desenlace de la prueba de antes como si
  // fuera el de ésta.
  delete resultado.value[n.id]
  const r = await store.enviarPrueba(n.id, f.para, f.plantilla, f.idioma)
  if (r) resultado.value[n.id] = r
  probando.value = null
}

/** Qué número está preguntando a Meta ahora mismo, para su botón ocupado. */
const refrescando = ref<string | null>(null)

/**
 * Vuelve a preguntarle a Meta la calidad de este número.
 *
 * **La calidad cambia sola**: baja cuando la gente bloquea o reporta, y es la
 * única señal de que Meta está a punto de cerrar el número — que no se
 * recupera. Anotarla sólo al dar de alta daría un semáforo congelado en el día
 * que se conectó, que es la peor clase de cifra: correcta sobre otra pregunta.
 */
async function refrescarCalidad(n: Numero): Promise<void> {
  refrescando.value = n.id
  await store.refrescarCalidad(n.id)
  refrescando.value = null
}

function tonoDeCalidad(n: Numero): 'exito' | 'aviso' | 'peligro' | 'neutro' {
  switch (n.calidad) {
    case 'green':
      return 'exito'
    case 'yellow':
      return 'aviso'
    case 'red':
      return 'peligro'
    // `unknown` de Meta y `null` de «no preguntamos» caen los dos aquí: ninguno
    // de los dos es una calidad buena, y ninguno se pinta como si lo fuera.
    default:
      return 'neutro'
  }
}
</script>

<template>
  <section class="pantalla">
    <header class="cabecera">
      <div>
        <h1>{{ t('numeroDeWhatsapp.titulo') }}</h1>
        <p class="explicacion">
          {{ t('numeroDeWhatsapp.explicacion') }}
        </p>
      </div>
      <ABoton
        v-if="!formularioAbierto"
        variante="primario"
        @click="abrir"
      >
        {{ t('numeroDeWhatsapp.anadir') }}
      </ABoton>
    </header>

    <AAviso
      v-if="problema"
      tono="peligro"
    >
      <strong>{{ problema.que }}</strong>
      <p>{{ problema.como }}</p>
      <p>{{ problema.salvo }}</p>
    </AAviso>

    <EstadoCargando v-if="store.cargando && store.numeros.length === 0" />

    <!-- ── El formulario de alta ── -->
    <AModulo
      v-if="formularioAbierto"
      :titulo="t('numeroDeWhatsapp.anadir')"
    >
      <!-- ── R-02 → D-WA · de seis campos a un dato y una elección ──
           El 14/10/2026 esto eran seis campos seguidos, y la respuesta a
           Dirección fue que **no se podía** reducir como el correo: el WABA ID
           y el Phone Number ID sólo existen en el panel de Meta y no hay nada
           de lo que inferirlos.

           Era verdad de *inferirlos* y falsa de *preguntarlos*. Con el token,
           Meta dice cuáles son sus números: el 26/10/2026 los seis campos
           quedaron en **pegar el token y elegir de una lista**, y lo que se
           guarda viene verificado por la fuente en vez de por quien copia. -->
      <div class="campos">
        <!-- ══ EL ALTA, EN UN DATO Y UNA ELECCIÓN (D-WA, 26/10/2026) ══
             Pedía SEIS campos, cuatro de ellos identificadores que sólo
             existen en el panel de Meta y hay que copiar sin equivocarse. Con
             el token, Meta dice el resto.

             Los campos de antes no se esconden: dejan de existir. Un
             formulario con «configuración avanzada» por si acaso es el
             formulario de antes con un pliegue. -->
        <p class="grupo">
          {{ t('numeroDeWhatsapp.grupoElToken') }}
        </p>
        <p class="grupo-nota">
          {{ t('numeroDeWhatsapp.grupoElTokenNota') }}
        </p>

        <AEntrada
          v-model="token"
          tipo="password"
          :etiqueta="t('numeroDeWhatsapp.token')"
          :ayuda="t('numeroDeWhatsapp.tokenAyuda')"
          @update:model-value="alCambiarElToken"
        />
        <p
          v-if="avisoToken"
          class="aviso-de-pegado"
        >
          {{ avisoToken }}
        </p>

        <ABoton
          variante="secundario"
          :ocupado="buscando"
          :deshabilitado="token.trim().length === 0"
          @click="buscarMisNumeros"
        >
          {{ t('numeroDeWhatsapp.buscar') }}
        </ABoton>

        <!-- El motivo por el que no se pudo preguntar. Son tres y mandan a
             hacer tres cosas distintas: revisar el token, esperar, o ir al
             panel a dar de alta un número. -->
        <AAviso
          v-if="errorAlBuscar"
          tono="peligro"
        >
          {{ t(errorAlBuscar) }}
        </AAviso>

        <!-- Los números que Meta dice que existen. Con su calidad y su tope:
             son las dos cifras que deciden si ese número puede trabajar, y
             enseñarlas aquí evita elegir a ciegas. -->
        <template v-if="descubiertos.length > 0">
          <p class="grupo">
            {{ t('numeroDeWhatsapp.grupoElige') }}
          </p>
          <ul class="descubiertos">
            <li
              v-for="n in descubiertos"
              :key="n.phoneNumberId"
            >
              <label class="descubierto">
                <input
                  v-model="elegido"
                  type="radio"
                  name="numero-descubierto"
                  :value="n.phoneNumberId"
                  @change="elegirNumero(n)"
                >
                <span class="descubierto__nombre">{{ n.nombre }}</span>
                <span class="descubierto__telefono">{{ numeroLegible(n.telefono) }}</span>
                <AInsignia :tono="tonoDeLaCalidadDeMeta(n.calidad)">
                  {{ t(`numeroDeWhatsapp.calidad.${n.calidad.toLowerCase()}`) }}
                </AInsignia>
                <span
                  v-if="n.topeDiario !== null"
                  class="descubierto__tope"
                >
                  {{ t('numeroDeWhatsapp.topeDeMeta', {
                    tope: n.topeDiario.toLocaleString('es-MX'),
                  }) }}
                </span>
              </label>
            </li>
          </ul>
        </template>

        <p class="grupo">
          {{ t('numeroDeWhatsapp.grupoComoSeUsa') }}
        </p>

        <ASelector
          v-model="borrador.modo"
          :etiqueta="t('numeroDeWhatsapp.modo')"
          :opciones="MODOS"
          :ayuda="t('numeroDeWhatsapp.modoAyuda')"
        />

        <!-- La única cosa del canal que le toca hacer a mano a quien lo usa, y
             el sitio donde se decide usarlo es éste. Va siempre, no sólo en
             frío: una baja se puede pedir desde cualquier modo. -->
        <p class="grupo-nota">
          {{ t('numeroDeWhatsapp.bajaAMano') }}
        </p>
      </div>

      <!-- Se avisa ANTES de guardar. Un aviso que llega después del alta no
           cambia ninguna decisión: solo informa de una que ya se tomó. -->
      <AAviso
        v-if="elegidoEnFrio"
        tono="aviso"
      >
        {{ t('numeroDeWhatsapp.riesgo.cuerpo') }}
      </AAviso>

      <div class="acciones">
        <ABoton
          variante="primario"
          :deshabilitado="!puedeGuardar"
          @click="guardar"
        >
          {{ t('numeroDeWhatsapp.guardar') }}
        </ABoton>
        <ABoton
          variante="secundario"
          @click="cerrar"
        >
          {{ t('numeroDeWhatsapp.cancelar') }}
        </ABoton>
      </div>
    </AModulo>

    <EstadoVacio
      v-else-if="store.numeros.length === 0 && !store.cargando"
      :titulo="t('numeroDeWhatsapp.vacioTitulo')"
      :cuerpo="t('numeroDeWhatsapp.vacioCuerpo')"
    />

    <!-- ── Los números dados de alta ── -->
    <p
      v-if="store.numeros.length > 0"
      class="nota"
    >
      {{ t('numeroDeWhatsapp.nota', { listos: store.listos, total: store.numeros.length }) }}
      <template v-if="store.porConfirmar > 0">
        ·
        {{ t('numeroDeWhatsapp.notaPorConfirmar', { porConfirmar: store.porConfirmar }) }}
      </template>
    </p>

    <AModulo
      v-for="{ n, f } in tarjetas"
      :key="n.id"
      :titulo="n.nombreVisible"
    >
      <div class="fila">
        <span class="telefono">{{ numeroLegible(n.telefonoE164) }}</span>
        <AInsignia :tono="n.puedeEnviar ? 'exito' : 'aviso'">
          {{
            n.puedeEnviar
              ? t('numeroDeWhatsapp.puedeEnviar')
              : t('numeroDeWhatsapp.noPuedeEnviar')
          }}
        </AInsignia>
        <AInsignia tono="neutro">
          {{ t(`numeroDeWhatsapp.estado.${n.estado}`) }}
        </AInsignia>
        <AInsignia tono="neutro">
          {{ t(`whatsapp.modo.${n.modo === 'seguimiento' ? 'seguimiento' : 'prospeccionDirecta'}`) }}
        </AInsignia>
      </div>

      <!-- La calidad, y lo que significa que no conste.
           El botón de refrescar va AQUÍ y no en la barra de acciones de abajo:
           lo que se refresca es esta cifra, y un botón lejos del dato que
           cambia hace dudar de qué se ha actualizado. -->
      <div class="fila">
        <span class="etiqueta">{{ t('numeroDeWhatsapp.calidad.etiqueta') }}</span>
        <AInsignia :tono="tonoDeCalidad(n)">
          {{ textoDeCalidad(n) }}
        </AInsignia>
        <ABoton
          variante="secundario"
          :ocupado="refrescando === n.id"
          @click="refrescarCalidad(n)"
        >
          {{ t('numeroDeWhatsapp.calidad.refrescar') }}
        </ABoton>
      </div>
      <p
        v-if="n.calidadActualizada !== null"
        class="ayuda"
      >
        {{ t('numeroDeWhatsapp.calidad.preguntadaEl', { cuando: n.calidadActualizada }) }}
      </p>
      <p
        v-if="n.calidad === null"
        class="ayuda"
      >
        {{ t('numeroDeWhatsapp.calidad.ayudaSinConsultar') }}
      </p>

      <p class="ayuda">
        {{
          n.limiteDeMeta === null
            ? t('numeroDeWhatsapp.limiteSinSaber')
            : t('numeroDeWhatsapp.limiteDeMeta', { cuantos: n.limiteDeMeta })
        }}
      </p>

      <!-- ── Coexistencia ── -->
      <AAviso
        v-if="n.coexistenciaConfirmada === null"
        tono="aviso"
      >
        <strong>{{ t('numeroDeWhatsapp.coexistencia.titulo') }}</strong>
        <p>{{ t('numeroDeWhatsapp.coexistencia.cuerpo') }}</p>
        <p>{{ t('numeroDeWhatsapp.coexistencia.porQue') }}</p>

        <!-- En el slot `acciones` y no en el cuerpo: el cuerpo es una columna
             flex, que estira a sus hijos, así que este botón medía 1020 px
             contra los 139 a 169 del resto del producto. `acciones` es una
             fila y le da su ancho natural. D-06, 28/10/2026. -->
        <template #acciones>
          <ABoton
            variante="secundario"
            @click="store.confirmarCoexistencia(n.id)"
          >
            {{ t('numeroDeWhatsapp.coexistencia.boton') }}
          </ABoton>
        </template>
      </AAviso>
      <p
        v-else
        class="confirmado"
      >
        {{ t('numeroDeWhatsapp.coexistencia.confirmada', { cuando: n.coexistenciaConfirmada }) }}
      </p>

      <!-- ── El riesgo del modo en frío. Solo aparece si el modo lo pide. ── -->
      <template v-if="n.escribeEnFrio">
        <AAviso
          v-if="n.riesgoAceptado === null"
          tono="peligro"
        >
          <strong>{{ t('numeroDeWhatsapp.riesgo.titulo') }}</strong>
          <p>{{ t('numeroDeWhatsapp.riesgo.cuerpo') }}</p>
          <p>{{ t('numeroDeWhatsapp.riesgo.porQue') }}</p>

          <template #acciones>
            <ABoton
              variante="secundario"
              @click="store.aceptarRiesgo(n.id)"
            >
              {{ t('numeroDeWhatsapp.riesgo.boton') }}
            </ABoton>
          </template>
        </AAviso>
        <p
          v-else
          class="confirmado"
        >
          {{ t('numeroDeWhatsapp.riesgo.aceptado', { cuando: n.riesgoAceptado }) }}
        </p>
      </template>

      <!-- ── Mandar un WhatsApp de prueba ──────────────────────────────────
           Sólo cuando el número ya puede enviar. Un botón que únicamente
           sirve para dar un error no es un botón: es una trampa. Cuando no
           puede, se explica (§95) en vez de ofrecerlo. -->
      <section class="prueba">
        <h2>{{ t('numeroDeWhatsapp.prueba.titulo') }}</h2>

        <template v-if="n.puedeEnviar">
          <p class="ayuda">
            {{ t('numeroDeWhatsapp.prueba.porQue') }}
          </p>

          <div class="campos">
            <AEntrada
              v-model="f.para"
              :etiqueta="t('numeroDeWhatsapp.prueba.para')"
              :ayuda="t('numeroDeWhatsapp.prueba.paraAyuda')"
            />
            <AEntrada
              v-model="f.plantilla"
              :etiqueta="t('numeroDeWhatsapp.prueba.plantilla')"
              :ayuda="t('numeroDeWhatsapp.prueba.plantillaAyuda')"
            />
            <AEntrada
              v-model="f.idioma"
              :etiqueta="t('numeroDeWhatsapp.prueba.idioma')"
              :ayuda="t('numeroDeWhatsapp.prueba.idiomaAyuda')"
            />
          </div>

          <ABoton
            variante="secundario"
            :deshabilitado="probando === n.id || f.para.trim().length === 0"
            @click="probar(n, f)"
          >
            {{
              probando === n.id
                ? t('numeroDeWhatsapp.prueba.mandando')
                : t('numeroDeWhatsapp.prueba.mandar')
            }}
          </ABoton>

          <!-- Que Meta lo rechace no es un error del programa: es el resultado
               que la prueba venía a averiguar, y se enseña como tal. -->
          <AAviso
            v-if="resultado[n.id]"
            :tono="resultado[n.id]?.aceptado ? 'exito' : 'aviso'"
          >
            <strong>
              {{
                resultado[n.id]?.aceptado
                  ? t('numeroDeWhatsapp.prueba.aceptado')
                  : t('numeroDeWhatsapp.prueba.noSalio')
              }}
            </strong>
            <!-- El texto de la clave sólo cuando NO se aceptó. Al aceptar,
                 `whatsapp.prueba.aceptado` dice lo mismo que el titular y se
                 leía dos veces seguidas: «Meta aceptó el mensaje» / «Meta
                 aceptó el mensaje.». Se vio en la captura, no en las pruebas:
                 las dos frases contienen lo que se buscaba. -->
            <p v-if="!resultado[n.id]?.aceptado">
              {{ t(resultado[n.id]?.clave ?? '') }}
            </p>
            <p v-else>
              {{ t('numeroDeWhatsapp.prueba.aceptadoDetalle') }}
            </p>
          </AAviso>
        </template>

        <!-- `info` y no `aviso`: que falte una confirmación no es un problema,
             es un paso pendiente. Pintarlo en ámbar haría buscar una avería. -->
        <AAviso
          v-else
          tono="info"
        >
          <strong>{{ t('numeroDeWhatsapp.prueba.todaviaNo') }}</strong>
          <p>{{ t('numeroDeWhatsapp.prueba.todaviaNoDetalle') }}</p>
        </AAviso>
      </section>
    </AModulo>
  </section>
</template>

<style scoped>
/* §17: ni un hex, ni un píxel suelto. Todo sale de los tokens. */
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

.explicacion {
  color: var(--arles-text-muted);
  margin-top: var(--arles-space-2);
}

.campos {
  display: grid;
  gap: var(--arles-space-4);
}

/* Los tres títulos que parten el formulario (R-02). Separan lo que se sabe de
   memoria de lo que hay que ir a buscar al panel de Meta. */
/* ── La lista de números que Meta dice que existen (D-WA) ──
   Cada uno es una fila pulsable entera, no sólo el redondel: el objetivo de
   32 px del §22 se cumple con la fila, y elegir el número es la acción
   principal de esta pantalla. */
.descubiertos {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
}

.descubierto {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--arles-space-3);
  min-height: var(--arles-objetivo-minimo);
  padding: var(--arles-space-3);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  cursor: pointer;
}

.descubierto:hover {
  background: var(--arles-surface-hover);
}

.descubierto__nombre {
  font-weight: var(--arles-font-weight-semibold);
}

.descubierto__telefono {
  font-variant-numeric: tabular-nums;
}

.descubierto__tope {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.grupo {
  margin: var(--arles-space-2) 0 0;
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.grupo-nota {
  margin: 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
}

/* El aviso de un dato que no tiene la pinta que debería. Avisa, no bloquea:
   Meta podría cambiar el formato, y negarse a guardar algo correcto es peor
   que señalar algo dudoso. */
.aviso-de-pegado {
  margin: 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-warning);
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-4);
}

.fila {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--arles-space-3);
  margin-bottom: var(--arles-space-3);
}

.telefono {
  font-variant-numeric: tabular-nums;
  font-weight: var(--arles-font-weight-semibold);
}

.etiqueta,
.ayuda {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.confirmado {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  margin-top: var(--arles-space-3);
}

.nota {
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}

.prueba {
  margin-top: var(--arles-space-5);
  padding-top: var(--arles-space-4);
  border-top: var(--arles-border-width) solid var(--arles-border);
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
}

.prueba h2 {
  font-size: var(--arles-font-size-body);
  font-weight: var(--arles-font-weight-semibold);
}

/* El botón no se estira: `.prueba` es una columna flex, y sin esto ocupaba
   todo el ancho de la tarjeta, que lo hacía parecer una barra y no un botón.
   Se vio en la captura. */
.prueba > button {
  align-self: flex-start;
}
</style>
