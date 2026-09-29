<script setup lang="ts">
/**
 * El asistente de campaña · nueve pasos (entrega 6.6).
 *
 * `UX_NAVEGACION.md` §5, §40 del brief.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ NUEVE PASOS Y NO UN FORMULARIO LARGO
 *
 * Porque tres de ellos **no son campos que rellenar sino cosas que leer**: la
 * simulación de duración, el preflight y el resumen de activación. En un
 * formulario largo se bajan de un scroll sin verlos, y son justo los que hacen
 * que alguien reconsidere antes de mandar mil correos.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * DÓNDE EMPIEZA LO IRREVERSIBLE, Y DÓNDE NO
 *
 * Los pasos 1 a 5 no escriben nada: se van guardando en memoria. **La campaña
 * se crea al salir del paso 5**, en borrador, porque los pasos 6, 7 y 8 la
 * necesitan existiendo —el simulador y el preflight leen de la base, y el
 * envío de prueba cuelga de la etapa—.
 *
 * Crear no manda nada. Lo que manda correos es **activar**, en el paso 9, y es
 * lo único que no se deshace: congela la audiencia y llena la cola.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * TRES REGLAS QUE ESTA PANTALLA NO DECIDE
 *
 * 1. **Si se puede activar** lo dice el preflight, que viene de Rust. Aquí sólo
 *    se apaga el botón. Si la regla viviera aquí habría dos copias, y el día
 *    que divergieran esta pantalla dejaría activar algo que el núcleo bloqueó.
 * 2. **El aviso de más de 50 diarios** lo exige `crear_campana`. La casilla de
 *    aquí es para poder leerlo antes de pulsar; quien lo impone es el núcleo.
 * 3. **La prueba del paso 8 es obligatoria** (§45), y no porque esta pantalla
 *    lo diga: es uno de los ocho puntos del preflight, y sin ella el punto
 *    falla.
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { resolverError } from '@/app/errores'
import { AVISO_DIARIO, useCampanasStore } from '@/app/stores/campanas'
import { useRemitentesStore } from '@/app/stores/remitentes'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModulo,
  ASelector,
  EstadoCargando,
} from '@/design/componentes'

const campanas = useCampanasStore()
const remitentes = useRemitentesStore()
const router = useRouter()
const { t, te } = useI18n()

/** Los nueve pasos, en orden. El índice es el paso menos uno. */
const PASOS = [
  'informacion',
  'audiencia',
  'remitente',
  'mensaje',
  'limites',
  'simulacion',
  'preflight',
  'prueba',
  'activacion',
] as const

const paso = ref(1)

// ── Lo que se va escribiendo ──
const nombre = ref('')
const cuenta = ref('')
const versionDePlantilla = ref('')
const ventana = ref('')
const limiteDiario = ref('')
const limitePorHora = ref('')
const aceptoElRiesgo = ref(false)
const direccionDePrueba = ref('')

/** La campaña, una vez creada. `null` hasta salir del paso 5. */
const creada = ref<{ id: string; etapa: string } | null>(null)
const resultadoDePrueba = ref<{ aceptado: boolean; problema?: string } | null>(null)
const activacion = ref<{ enLaAudiencia: number; fueraPorBaja: number } | null>(null)

onMounted(() => {
  void campanas.cargar()
  void remitentes.cargar()
})

const error = computed(() =>
  campanas.errorGeneral ? resolverError({ clave: campanas.errorGeneral, detalle: '' }) : null,
)

// ── Las opciones de los desplegables ──

const opcionesDeCuenta = computed(() =>
  remitentes.remitentes
    // Sólo las que pueden enviar. Ofrecer una cuenta rota es construir una
    // campaña que el preflight va a bloquear tres pasos después.
    .filter((r) => r.estado === 'connected')
    .map((r) => ({ valor: r.id, texto: r.remitente })),
)

const opcionesDePlantilla = computed(() =>
  campanas.plantillas
    .filter((p) => p.version !== null)
    .map((p) => ({
      valor: p.version?.id ?? '',
      texto: `${p.nombre} · v${p.version?.numero ?? 0}`,
    })),
)

const opcionesDeVentana = computed(() =>
  campanas.ventanas.map((v) => ({
    valor: v.id,
    texto: `${v.nombre} · ${v.desde}:00–${v.hasta}:00 · ${v.zona}`,
  })),
)

/** La plantilla elegida, para poder enseñar sus variables en el paso 4. */
const plantillaElegida = computed(() =>
  campanas.plantillas.find((p) => p.version?.id === versionDePlantilla.value),
)

// ── El aviso del §48 ──

const diarioComoNumero = computed(() => {
  const n = Number.parseInt(limiteDiario.value, 10)
  return Number.isFinite(n) ? n : null
})

/**
 * ¿Hay que enseñar el aviso de volumen?
 *
 * **Más de 50**, no 50 o más: el umbral del §48 es «más de 50», y está escrito
 * igual en `Limites::AVISO_DIARIO` y en `crear_campana`.
 */
const avisaDelVolumen = computed(
  () => diarioComoNumero.value !== null && diarioComoNumero.value > AVISO_DIARIO,
)

// ── Qué hace falta para poder seguir ──

/**
 * Si el paso actual está completo.
 *
 * Deja avanzar, y **no valida a fondo**: quien valida de verdad es Rust. Esto
 * es para no llevar a alguien al paso 9 con el nombre en blanco, no para
 * decidir si la campaña vale.
 */
const sePuedeSeguir = computed(() => {
  switch (paso.value) {
    case 1:
      return nombre.value.trim().length > 0
    case 3:
      return cuenta.value.length > 0
    case 4:
      return versionDePlantilla.value.length > 0
    case 5:
      return (
        ventana.value.length > 0 && (!avisaDelVolumen.value || aceptoElRiesgo.value)
      )
    case 8:
      // La prueba es obligatoria (§45). Y vale la que el servidor aceptó: una
      // rechazada demuestra lo contrario de lo que se quería demostrar.
      return resultadoDePrueba.value?.aceptado === true
    default:
      return true
  }
})

/** Lo que dice el preflight. `null` mientras no se haya pedido. */
const sePuedeActivar = computed(
  () => campanas.revision?.preflight.resultado.resultado === 'lista',
)

// ── Navegación ──

async function siguiente(): Promise<void> {
  if (!sePuedeSeguir.value) return

  // Al salir del 5 se crea la campaña: del 6 en adelante todo lee de la base.
  if (paso.value === 5 && creada.value === null) {
    const r = await campanas.crearCampana({
      nombre: nombre.value.trim(),
      cuenta: cuenta.value,
      versionDePlantilla: versionDePlantilla.value,
      ventana: ventana.value,
      limitePorHora: Number.parseInt(limitePorHora.value, 10) || null,
      limiteDiario: diarioComoNumero.value,
      aceptoElRiesgoDelVolumen: aceptoElRiesgo.value,
    })
    if (r === null) return
    creada.value = r
  }

  // El 6 y el 7 salen de **una sola** llamada, la de aquí.
  //
  // ── AUDITORÍA DE LA FASE 6 · 29/09/2026 ──
  //
  // Esto pedía la revisión otra vez al salir del 6, y entre el 6 y el 7 no
  // cambia nada: era una sonda SMTP de más, con su espera de hasta 60 segundos
  // (`ESPERA_MAXIMA`), para volver a calcular exactamente lo mismo. Y en macOS,
  // una lectura del llavero de más — o sea, otro diálogo del sistema.
  //
  // Pedirlos juntos además es lo correcto: son dos vistas del mismo instante de
  // la campaña, y dos llamadas darían dos fotos distintas de la misma cosa.
  if (paso.value === 5 && creada.value) {
    await campanas.revisar(creada.value.id)
  }

  paso.value = Math.min(paso.value + 1, PASOS.length)
}

function atras(): void {
  paso.value = Math.max(paso.value - 1, 1)
}

async function mandarLaPrueba(): Promise<void> {
  if (!creada.value) return
  resultadoDePrueba.value = await campanas.enviarPrueba(
    creada.value.etapa,
    direccionDePrueba.value.trim(),
  )
  // La prueba cambia uno de los ocho puntos, así que el preflight de la
  // pantalla deja de ser cierto. Se vuelve a pedir en vez de arreglarlo a mano:
  // parchear el punto aquí sería la pantalla decidiendo el veredicto.
  if (resultadoDePrueba.value?.aceptado) await campanas.revisar(creada.value.id)
}

async function activar(): Promise<void> {
  if (!creada.value || !sePuedeActivar.value) return
  activacion.value = await campanas.activar(creada.value.id)
}

function alFinal(): void {
  void router.push({ name: 'campanas' })
}

function rotulo(clave: string): string {
  return te(clave) ? t(clave) : clave
}
</script>

<template>
  <section class="asistente">
    <header>
      <h1 class="asistente__titulo">
        {{ t('asistente.titulo') }}
      </h1>
      <!-- §94: cifras. «Paso 4 de 9», no «casi listo». -->
      <p class="asistente__progreso">
        {{ t('asistente.progreso', { paso, total: PASOS.length }) }} ·
        {{ t(`asistente.paso.${PASOS[paso - 1]}.titulo`) }}
      </p>
    </header>

    <AAviso
      v-if="error"
      tono="peligro"
      :titulo="error.que"
      urgente
    >
      <p>{{ error.como }}</p>
      <p class="asistente__salvo">
        {{ error.salvo }}
      </p>
    </AAviso>

    <AModulo :titulo="t(`asistente.paso.${PASOS[paso - 1]}.titulo`)">
      <!-- 1 · Información -->
      <template v-if="paso === 1">
        <AEntrada
          v-model="nombre"
          :etiqueta="t('asistente.paso.informacion.nombre')"
          :ayuda="t('asistente.paso.informacion.ayuda')"
        />
      </template>

      <!-- 2 · Audiencia -->
      <template v-else-if="paso === 2">
        <p>{{ t('asistente.paso.audiencia.explicacion') }}</p>
        <AAviso
          tono="info"
          :titulo="t('asistente.paso.audiencia.alcance')"
        >
          <p>{{ t('asistente.paso.audiencia.alcanceDetalle') }}</p>
        </AAviso>
      </template>

      <!-- 3 · Remitente -->
      <template v-else-if="paso === 3">
        <ASelector
          v-model="cuenta"
          :etiqueta="t('asistente.paso.remitente.cuenta')"
          :opciones="opcionesDeCuenta"
        />
        <AAviso
          v-if="opcionesDeCuenta.length === 0"
          tono="aviso"
          :titulo="t('asistente.paso.remitente.sinCuentas')"
        >
          <p>{{ t('asistente.paso.remitente.sinCuentasComo') }}</p>
        </AAviso>
        <!-- ADR-0008: ARLES no rota remitentes, y se dice donde se elige uno. -->
        <p class="asistente__nota">
          {{ t('asistente.paso.remitente.sinRotacion') }}
        </p>
      </template>

      <!-- 4 · Mensaje -->
      <template v-else-if="paso === 4">
        <ASelector
          v-model="versionDePlantilla"
          :etiqueta="t('asistente.paso.mensaje.plantilla')"
          :opciones="opcionesDePlantilla"
        />
        <div
          v-if="plantillaElegida?.version"
          class="asistente__variables"
        >
          <p>{{ t('asistente.paso.mensaje.asunto') }}: {{ plantillaElegida.version.asunto }}</p>
          <p v-if="plantillaElegida.version.variables.length > 0">
            {{ t('asistente.paso.mensaje.variables') }}:
            <AInsignia
              v-for="v in plantillaElegida.version.variables"
              :key="v"
              tono="info"
            >
              {{ rotulo(`plantilla.variable.${v}`) }}
            </AInsignia>
          </p>
        </div>
      </template>

      <!-- 5 · Límites y horario -->
      <template v-else-if="paso === 5">
        <ASelector
          v-model="ventana"
          :etiqueta="t('asistente.paso.limites.horario')"
          :opciones="opcionesDeVentana"
        />
        <AEntrada
          v-model="limiteDiario"
          :etiqueta="t('asistente.paso.limites.diario')"
          :ayuda="t('asistente.paso.limites.diarioAyuda')"
        />
        <AEntrada
          v-model="limitePorHora"
          :etiqueta="t('asistente.paso.limites.porHora')"
        />

        <!-- §48: en línea, no en un modal. Informar, no dictar (§154). -->
        <AAviso
          v-if="avisaDelVolumen"
          tono="aviso"
          :titulo="t('asistente.paso.limites.aviso.titulo')"
        >
          <p>{{ t('asistente.paso.limites.aviso.cuerpo') }}</p>
          <p>{{ t('asistente.paso.limites.aviso.noBloquea') }}</p>
          <label class="asistente__casilla">
            <input
              v-model="aceptoElRiesgo"
              type="checkbox"
            >
            {{
              t('asistente.paso.limites.aviso.acepto', { n: diarioComoNumero ?? 0 })
            }}
          </label>
        </AAviso>
      </template>

      <!-- 6 · Simulación -->
      <template v-else-if="paso === 6">
        <!-- Puede tardar: habla con el servidor de correo, que tiene hasta un
             minuto para contestar. `EstadoCargando` enseña la explicación a
             partir de los 2 s, así que una espera larga deja de parecer una
             avería y pasa a ser algo que está pasando. -->
        <EstadoCargando
          v-if="campanas.revisando"
          :lineas="3"
          :explicacion="t('asistente.paso.simulacion.calculando')"
        />
        <div
          v-else-if="campanas.revision"
          class="asistente__simulacion"
        >
          <p class="asistente__frase">
            {{
              t('asistente.paso.simulacion.cuantos', {
                n: campanas.revision.audiencia.toLocaleString('es-MX'),
              })
            }}
          </p>
          <p class="asistente__frase">
            {{
              t('asistente.paso.simulacion.ritmo', {
                techo: campanas.revision.estimacion.techoDiario,
                dias: campanas.revision.estimacion.diasConEnvio,
              })
            }}
          </p>
          <p
            v-if="campanas.revision.contactosConHuecos > 0"
            class="asistente__nota"
          >
            {{
              t('asistente.paso.simulacion.huecos', {
                n: campanas.revision.contactosConHuecos,
                total: campanas.revision.audiencia,
              })
            }}
          </p>
        </div>
      </template>

      <!-- 7 · Preflight -->
      <template v-else-if="paso === 7">
        <ul
          v-if="campanas.revision"
          class="asistente__puntos"
        >
          <li
            v-for="p in campanas.revision.preflight.puntos"
            :key="p.punto"
          >
            <AInsignia
              :tono="
                p.veredicto === 'pasa'
                  ? 'exito'
                  : p.veredicto === 'falla'
                    ? 'peligro'
                    : 'incierto'
              "
            >
              {{ t(`preflight.${p.veredicto}`) }}
            </AInsignia>
            {{ t(`preflight.${p.punto}`) }}
          </li>
        </ul>
        <!-- «Sin comprobar» no es un aprobado, y el texto lo dice. -->
        <p class="asistente__nota">
          {{ t('preflight.sinComprobarDetalle') }}
        </p>
      </template>

      <!-- 8 · Envío de prueba -->
      <template v-else-if="paso === 8">
        <p>{{ t('asistente.paso.prueba.porQue') }}</p>
        <AEntrada
          v-model="direccionDePrueba"
          :etiqueta="t('asistente.paso.prueba.aDonde')"
        />
        <ABoton
          variante="secundario"
          :ocupado="campanas.trabajando"
          @click="mandarLaPrueba"
        >
          {{ t('asistente.paso.prueba.mandar') }}
        </ABoton>

        <AAviso
          v-if="resultadoDePrueba?.aceptado"
          tono="exito"
          :titulo="t('asistente.paso.prueba.aceptado')"
          urgente
        >
          <!-- §65: el servidor se hizo cargo. No se dice «entregado». -->
          <p>{{ t('asistente.paso.prueba.aceptadoDetalle') }}</p>
        </AAviso>
        <AAviso
          v-else-if="resultadoDePrueba"
          tono="peligro"
          :titulo="t('asistente.paso.prueba.noSalio')"
          urgente
        >
          <p>{{ rotulo(`${resultadoDePrueba.problema}.que`) }}</p>
        </AAviso>
      </template>

      <!-- 9 · Activación -->
      <template v-else>
        <div v-if="activacion === null">
          <p class="asistente__frase">
            {{
              t('asistente.paso.activacion.resumen', {
                nombre,
                n: campanas.revision?.audiencia ?? 0,
              })
            }}
          </p>
          <AAviso
            tono="aviso"
            :titulo="t('asistente.paso.activacion.noSeDeshace')"
          >
            <p>{{ t('asistente.paso.activacion.noSeDeshaceDetalle') }}</p>
          </AAviso>
          <ABoton
            variante="primario"
            :deshabilitado="!sePuedeActivar"
            :ocupado="campanas.trabajando"
            @click="activar"
          >
            {{ t('asistente.paso.activacion.activar') }}
          </ABoton>
          <p
            v-if="!sePuedeActivar"
            class="asistente__nota"
          >
            {{ t('asistente.paso.activacion.bloqueada') }}
          </p>
        </div>

        <AAviso
          v-else
          tono="exito"
          :titulo="t('asistente.paso.activacion.activada')"
          urgente
        >
          <!-- §94: la cifra de los que quedaron fuera se dice siempre, aunque
               sea cero. Sin ella, quien lleve su lista en la cabeza se pasa la
               tarde buscando por qué no cuadra. -->
          <p>
            {{
              t('asistente.paso.activacion.cifras', {
                n: activacion.enLaAudiencia,
                fuera: activacion.fueraPorBaja,
              })
            }}
          </p>
          <ABoton
            variante="primario"
            @click="alFinal"
          >
            {{ t('asistente.paso.activacion.verCampanas') }}
          </ABoton>
        </AAviso>
      </template>
    </AModulo>

    <nav
      v-if="activacion === null"
      class="asistente__pies"
    >
      <ABoton
        variante="sutil"
        :deshabilitado="paso === 1"
        @click="atras"
      >
        {{ t('asistente.atras') }}
      </ABoton>
      <ABoton
        v-if="paso < PASOS.length"
        variante="primario"
        :deshabilitado="!sePuedeSeguir"
        :ocupado="campanas.trabajando || campanas.revisando"
        @click="siguiente"
      >
        {{ t('asistente.siguiente') }}
      </ABoton>
    </nav>
  </section>
</template>

<style scoped>
.asistente {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.asistente__titulo {
  margin: 0;
  font-size: var(--arles-font-size-h1);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
}

.asistente__progreso {
  margin: var(--arles-space-1) 0 0;
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}

.asistente__frase {
  margin: 0 0 var(--arles-space-3);
  font-size: var(--arles-font-size-h3);
  color: var(--arles-text);
}

.asistente__nota,
.asistente__salvo {
  margin: var(--arles-space-3) 0 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.asistente__puntos {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
  margin: 0;
  padding: 0;
  list-style: none;
}

.asistente__variables,
.asistente__simulacion {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
}

.asistente__casilla {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  margin-top: var(--arles-space-3);
}

.asistente__pies {
  display: flex;
  justify-content: space-between;
  gap: var(--arles-space-3);
}
</style>
