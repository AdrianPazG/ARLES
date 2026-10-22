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
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { resolverError } from '@/app/errores'
import { AVISO_DIARIO, useCampanasStore } from '@/app/stores/campanas'
import { useEmpresaStore } from '@/app/stores/empresa'
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
// La zona horaria del horario es la de la EMPRESA, nunca la del equipo: es la
// decisión que la pantalla de Configuración explica en su aviso.
const empresa = useEmpresaStore()

/**
 * Por encima de cuántos destinatarios se avisa de la espera.
 *
 * ── DE DÓNDE SALE ESTE NÚMERO ─────────────────────────────────────────────
 *
 * De medir, no de elegirlo. `activar_campana` escribe dos filas por
 * destinatario con sus índices sobre SQLCipher, y la auditoría C del §140 lo
 * midió a cuatro escalas el 07/10/2026:
 *
 *     18 121 → 1.2 s     45 253 → 3.8 s
 *    135 588 → 16.7 s   451 196 → 73.5 s
 *
 * El presupuesto del §1 son 2 s, y se rompe **alrededor de los 30 000**. Por
 * debajo de eso el aviso estorbaría; por encima, su ausencia convierte una
 * espera normal en una avería aparente (§95).
 *
 * **Esto no es un tope.** Una campaña de 136 000 personas que termina en nueve
 * meses es perfectamente razonable, y prohibirla por tardar diecisiete
 * segundos en prepararse sería cambiar un problema de rendimiento por una
 * limitación de producto. El arreglo de verdad —sacarlo a segundo plano con
 * avance— es la Fase 9.
 */
const AUDIENCIA_QUE_TARDA = 30_000

/**
 * Cuántos segundos va a tardar en prepararse, redondeado hacia arriba.
 *
 * La curva medida no es lineal —163 µs por fila a 451 000 contra 66 a 18 000,
 * porque los índices crecen— así que una regla de tres con el caso pequeño
 * prometería la mitad del tiempo real. Se usa el peor caso medido, 163 µs por
 * destinatario, y se redondea a múltiplos de cinco: dar «17 segundos» finge
 * una precisión que esta estimación no tiene (§94), y quedarse corto es el
 * único error que importa — quien espera diez y tarda veinte cree que se
 * colgó.
 */
function segundosDePreparacion(audiencia: number): number {
  const estimados = (audiencia * 163) / 1_000_000
  return Math.max(5, Math.ceil(estimados / 5) * 5)
}
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
// La firma del pie. Vacío es «la de por defecto de la empresa», que es lo que
// promete su casilla en Plantillas (H-12).
const firma = ref('')
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
  // Hace falta su zona horaria para crear un horario (H-06).
  void empresa.cargar()
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

/**
 * Las firmas, con «la de por defecto» la primera.
 *
 * Se ofrece la opción vacía y no se obliga a elegir: `campaign_stage` admite
 * `NULL` y eso significa exactamente lo que la casilla de Plantillas promete.
 * Obligar a elegir convertiría «por defecto» en una palabra sin uso.
 */
const opcionesDeFirma = computed(() => [
  { valor: '', texto: t('asistente.paso.mensaje.firmaPorDefecto') },
  ...campanas.firmas.map((f) => ({ valor: f.id, texto: f.nombre })),
])

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

// ── EL HORARIO DE ENVÍO, QUE DEJABA LA CAMPAÑA SIN SALIDA (H-06) ─────────
//
// Dirección llegó al paso 5 el 16/10/2026 y el desplegable estaba VACÍO, así
// que «Continuar» no dejaba pasar y la campaña no se podía crear.
//
// La causa es el patrón del §9 por decimoctava vez: `guardar_ventana` existe,
// el store lo envuelve, y **ninguna pantalla lo llamaba**. La lista de Inicio
// lo decía —«Definir una ventana de envío · Llega en la entrega 6»— mientras
// el asistente la exigía para avanzar.
//
// Se resuelve AQUÍ y no en otra pantalla porque mandar a definir un horario a
// otra sección, a mitad de un asistente de nueve pasos, es perder lo tecleado
// o volver con el hilo roto. Y lo que hace falta casi siempre es un horario de
// oficina, que no hay por qué construir campo a campo.
const noHayHorarios = computed(() => campanas.ventanas.length === 0)

/** Lunes a viernes, que es `Dias::LABORABLES` en el núcleo. */
const DIAS_LABORABLES = 0b001_1111
const HORA_DE_ENTRADA = 9
const HORA_DE_SALIDA = 18

const creandoHorario = ref(false)
const horarioPropio = ref({ nombre: '', desde: '9', hasta: '18' })

async function usarElDeOficina(): Promise<void> {
  await crearHorario(
    t('asistente.paso.limites.horarioDeOficina'),
    HORA_DE_ENTRADA,
    HORA_DE_SALIDA,
  )
}

async function crearElPropio(): Promise<void> {
  const desde = Number.parseInt(horarioPropio.value.desde, 10)
  const hasta = Number.parseInt(horarioPropio.value.hasta, 10)
  await crearHorario(
    horarioPropio.value.nombre.trim() || t('asistente.paso.limites.horarioSinNombre'),
    Number.isNaN(desde) ? HORA_DE_ENTRADA : desde,
    Number.isNaN(hasta) ? HORA_DE_SALIDA : hasta,
  )
}

/**
 * Crea el horario y **lo deja elegido**.
 *
 * Dejarlo creado pero sin elegir obligaría a desplegar y seleccionar lo que se
 * acaba de crear, que además es el único horario que hay.
 */
async function crearHorario(nombre: string, desde: number, hasta: number): Promise<void> {
  creandoHorario.value = true
  try {
    const zona = empresa.empresa?.zonaHoraria ?? 'America/Mexico_City'
    if (await campanas.guardarVentana(nombre, DIAS_LABORABLES, desde, hasta, zona)) {
      const recien = campanas.ventanas[campanas.ventanas.length - 1]
      if (recien) ventana.value = recien.id
    }
  } finally {
    creandoHorario.value = false
  }
}

// ── EL MÁXIMO DIARIO, QUE NO DECÍA CUÁL ERA (H-10) ──────────────────────
//
// «Déjalo en blanco para usar el de la cuenta» no decía CUÁL es, así que se
// aceptaba un número sin saberlo — y es un número que importa: por encima de
// 50 ARLES exige aceptar un aviso (§48).
//
// «El de la cuenta» es un adjetivo; «50» es una cifra (§94).
const cuentaElegida = computed(() => remitentes.remitentes.find((r) => r.id === cuenta.value))
const topeDeLaCuenta = computed(() => cuentaElegida.value?.limiteDiario ?? null)
const topePorHoraDeLaCuenta = computed(() => cuentaElegida.value?.limitePorHora ?? null)

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

/**
 * Al elegir cuenta, los dos campos se rellenan con lo que esa cuenta trae.
 *
 * ── POR QUÉ SE RELLENAN Y NO SE DEJAN EN BLANCO (22/10/2026) ──────────────
 *
 * Dirección pidió fijar el límite **al crear la campaña**, y un campo vacío
 * con un texto que dice «déjalo en blanco para usar el de la cuenta» es lo
 * contrario de eso: esconde la cifra que de verdad se va a aplicar detrás de
 * una frase. Con el número delante se ve, se compara y se cambia.
 *
 * Sólo se rellena lo que esté vacío: si alguien escribió 300 y después vuelve
 * al paso 3 a cambiar de remitente, pisarle el 300 le quitaría su decisión sin
 * avisar.
 */
watch(cuentaElegida, (r) => {
  if (!r) return
  if (limiteDiario.value.trim() === '') limiteDiario.value = String(r.limiteDiario)
  if (limitePorHora.value.trim() === '') limitePorHora.value = String(r.limitePorHora)
})

/**
 * Un límite escrito que no es un número de envíos.
 *
 * Cero, negativo o con letras. **No es cosmética**: hasta hoy un «0» se
 * guardaba tal cual y la campaña se activaba sin mandar nunca nada, y un «-5»
 * se descartaba en Rust y se usaba el de la cuenta — las dos cosas en
 * silencio, que es lo que esta pantalla tiene que dejar de hacer.
 */
function esUnLimiteImposible(texto: string): boolean {
  if (texto.trim() === '') return false
  const n = Number.parseInt(texto, 10)
  return !Number.isFinite(n) || n < 1
}

const limiteImposible = computed(
  () => esUnLimiteImposible(limiteDiario.value) || esUnLimiteImposible(limitePorHora.value),
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
        ventana.value.length > 0 &&
        !limiteImposible.value &&
        (!avisaDelVolumen.value || aceptoElRiesgo.value)
      )
    case 6: {
      // ── NO SE DEJA AVANZAR SI LA CAMPAÑA NO VA A PODER ACTIVARSE ──
      //
      // `activar_campana` la rechaza desde la 8.6 E, y eso pasa en el paso 9.
      // Dejar seguir aquí sería hacer rellenar tres pasos más —el preflight y
      // el envío de prueba, que habla con el servidor— para fallar al final:
      // es exactamente el defecto que el aviso de volumen evita pidiéndolo
      // antes de escribir nada, y que CLAUDE.md nombra como lo que no hay que
      // hacer («rechaza al final nueve pasos de formulario»).
      //
      // Lo que se puede hacer es volver al paso 5 y subir los topes, que es lo
      // que el aviso dice. Por eso se bloquea «Continuar» y no «Atrás».
      const d = campanas.revision?.estimacion.desenlace.desenlace
      return d !== 'noAvanza' && d !== 'noCabeEnElHorizonte'
    }
    case 8:
      // La prueba es obligatoria (§45). Y vale la que el servidor aceptó: una
      // rechazada demuestra lo contrario de lo que se quería demostrar.
      return resultadoDePrueba.value?.aceptado === true
    default:
      return true
  }
})

/** ¿El simulador dice que esta campaña llega al final? */
const termina = computed(
  () => campanas.revision?.estimacion.desenlace.desenlace === 'termina',
)

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
      // Vacío es «la de por defecto»: `null`, no una cadena vacía, que la
      // clave ajena rechazaría.
      firma: firma.value === '' ? null : firma.value,
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
        <!--
          ── SIN PLANTILLAS, EL PASO 4 DEJABA SIN SALIDA (22/10/2026) ──────
          El paso 3 avisa cuando no hay cuentas y el 5 cuando no hay horarios;
          **éste no avisaba de nada**: un desplegable vacío, «Continuar»
          apagado, y ninguna pista de que hay que crear la plantilla en otra
          pantalla. Es el H-06 en el paso de al lado, y se encontró mirando la
          demostración con las diez fases en verde.
        -->
        <AAviso
          v-if="opcionesDePlantilla.length === 0"
          tono="aviso"
          :titulo="t('asistente.paso.mensaje.sinPlantillas')"
        >
          <p>{{ t('asistente.paso.mensaje.sinPlantillasComo') }}</p>
          <ABoton
            variante="secundario"
            @click="router.push({ name: 'plantillas' })"
          >
            {{ t('asistente.paso.mensaje.sinPlantillasIr') }}
          </ABoton>
        </AAviso>
        <!--
          La firma, que hasta el 22/10/2026 no se podía elegir: la columna
          existía desde la V4, la consulta del envío ya la miraba, y nadie la
          escribía — así que todas las campañas salían con la de por defecto.
        -->
        <ASelector
          v-model="firma"
          :etiqueta="t('asistente.paso.mensaje.firma')"
          :opciones="opcionesDeFirma"
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
        <!-- ── SIN NINGÚN HORARIO ESTO ERA UN CALLEJÓN SIN SALIDA (H-06) ──
             El desplegable salía vacío y «Continuar» no dejaba pasar, así que
             la campaña no se podía crear. Ahora se ofrece el de oficina a un
             botón, que es lo que hace falta casi siempre. -->
        <AAviso
          v-if="noHayHorarios"
          tono="info"
          :titulo="t('asistente.paso.limites.sinHorarios.titulo')"
        >
          <p>{{ t('asistente.paso.limites.sinHorarios.cuerpo') }}</p>
          <div class="asistente__horario-acciones">
            <ABoton
              variante="primario"
              :ocupado="creandoHorario"
              @click="usarElDeOficina()"
            >
              {{ t('asistente.paso.limites.sinHorarios.usarOficina') }}
            </ABoton>
          </div>

          <details class="asistente__horario-propio">
            <summary>{{ t('asistente.paso.limites.sinHorarios.oDefineElTuyo') }}</summary>
            <AEntrada
              v-model="horarioPropio.nombre"
              :etiqueta="t('asistente.paso.limites.sinHorarios.nombre')"
              :ayuda="t('asistente.paso.limites.sinHorarios.nombreAyuda')"
            />
            <div class="asistente__horas">
              <AEntrada
                v-model="horarioPropio.desde"
                :etiqueta="t('asistente.paso.limites.sinHorarios.desde')"
              />
              <AEntrada
                v-model="horarioPropio.hasta"
                :etiqueta="t('asistente.paso.limites.sinHorarios.hasta')"
              />
            </div>
            <ABoton
              variante="secundario"
              :ocupado="creandoHorario"
              @click="crearElPropio()"
            >
              {{ t('asistente.paso.limites.sinHorarios.crear') }}
            </ABoton>
          </details>
        </AAviso>

        <ASelector
          v-else
          v-model="ventana"
          :etiqueta="t('asistente.paso.limites.horario')"
          :opciones="opcionesDeVentana"
        />

        <AEntrada
          v-model="limiteDiario"
          :etiqueta="t('asistente.paso.limites.diario')"
          :ayuda="
            topeDeLaCuenta === null
              ? t('asistente.paso.limites.diarioAyuda')
              : t('asistente.paso.limites.diarioAyudaConCifra', { n: topeDeLaCuenta })
          "
        />
        <AEntrada
          v-model="limitePorHora"
          :etiqueta="t('asistente.paso.limites.porHora')"
          :ayuda="
            topePorHoraDeLaCuenta === null
              ? ''
              : t('asistente.paso.limites.porHoraAyuda', { n: topePorHoraDeLaCuenta })
          "
        />

        <!--
          Un 0, un negativo o letras. Antes se guardaba tal cual: con «0» la
          campaña se activaba y no mandaba nunca nada, y con «-5» Rust lo
          descartaba y usaba el de la cuenta. Las dos, en silencio.
        -->
        <AAviso
          v-if="limiteImposible"
          tono="aviso"
          :titulo="t('asistente.paso.limites.imposible.titulo')"
        >
          <p>{{ t('asistente.paso.limites.imposible.cuerpo') }}</p>
        </AAviso>

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
          <!-- ── LA FRASE DEL RITMO NO PROMETE LO QUE NO VA A PASAR ──
               Mirando la demostración se vio «saldrán a lo largo de 1303 días
               de envío» y justo debajo «no termina en cinco años». La primera
               no es falsa —el reloj se para en el horizonte— pero dice
               «saldrán» de unos mensajes que no van a salir.

               Con un desenlace que impide activar se enseña sólo el techo, que
               es el dato con el que se arregla; los días los dice el aviso. -->
          <p class="asistente__frase">
            {{
              termina
                ? t('asistente.paso.simulacion.ritmo', {
                  techo: campanas.revision.estimacion.techoDiario,
                  dias: campanas.revision.estimacion.diasConEnvio.toLocaleString('es-MX'),
                })
                : t('asistente.paso.simulacion.ritmoSinFinal', {
                  techo: campanas.revision.estimacion.techoDiario,
                })
            }}
          </p>

          <!--
            ── EL PLAZO DE ARRIBA NO VALE SI EL REMITENTE SE COMPARTE ──────
            Lo vio Dirección el 22/10/2026: el cupo de envío cuelga de la
            CUENTA, así que dos campañas de 50 al día sobre un remitente sacan
            25 y 25 — y por orden de llegada, no por mitades. La estimación
            calcula con el límite de esta campaña como si el remitente fuera
            sólo suyo, así que promete un plazo que no se cumple.

            Va JUSTO DEBAJO de la frase del ritmo, no al final de la pantalla:
            un aviso que desmiente una cifra tiene que verse con la cifra
            delante, o se lee la cifra y se ignora el aviso.
          -->
          <AAviso
            v-if="campanas.revision.compartenRemitente.length > 0"
            tono="aviso"
            :titulo="
              t('asistente.paso.simulacion.remitenteCompartido', {
                campanas: campanas.revision.compartenRemitente
                  .map((c) => c.nombre)
                  .join(', '),
                n: campanas.revision.compartenRemitente.length,
              })
            "
          >
            <p>{{ t('asistente.paso.simulacion.remitenteCompartidoDetalle') }}</p>
            <p>{{ t('asistente.paso.simulacion.remitenteCompartidoQueHacer') }}</p>
          </AAviso>

          <!-- ── EL DESENLACE, QUE EL NÚCLEO CALCULABA Y NADIE ENSEÑABA ──
               `estimar` devuelve tres desenlaces desde la Fase 4 y los tres
               textos estaban escritos desde entonces. La pantalla pintaba el
               techo y los días, y nada más: con 451 196 personas a 50 diarios
               decía «1 303 días» —correcto— y **nunca decía que 386 046
               personas no iban a recibir un mensaje nunca**.

               Va en un `AAviso` y no en un párrafo porque los dos casos malos
               impiden activar: `activar_campana` los rechaza desde la 8.6 E, y
               una frase suelta no se lee como un impedimento. -->
          <AAviso
            v-if="campanas.revision.estimacion.desenlace.desenlace === 'noAvanza'"
            tono="peligro"
            :titulo="t('duracion.noAvanza')"
          />
          <AAviso
            v-else-if="
              campanas.revision.estimacion.desenlace.desenlace === 'noCabeEnElHorizonte'
            "
            tono="peligro"
            :titulo="
              t('duracion.noCabeEnElHorizonte', {
                faltan: (campanas.revision.estimacion.desenlace.faltan ?? 0).toLocaleString(
                  'es-MX',
                ),
              })
            "
          />
          <p
            v-else
            class="asistente__frase"
          >
            {{
              t('duracion.termina', {
                cuantos: campanas.revision.audiencia.toLocaleString('es-MX'),
                cuando: campanas.revision.estimacion.desenlace.cuando ?? '—',
              })
            }}
          </p>

          <!-- ── Y LO QUE VA A TARDAR EN PREPARARSE (8.6 E) ──
               Medido: 4 s con 45 000 destinatarios, 73 s con 451 000. El
               arreglo de verdad es sacarlo a segundo plano (Fase 9); hasta
               entonces lo que convierte un cuelgue en una espera es avisar
               antes, con la cifra (§95). -->
          <AAviso
            v-if="campanas.revision.audiencia > AUDIENCIA_QUE_TARDA"
            tono="aviso"
            :titulo="
              t('asistente.paso.simulacion.tardaraEnPrepararse', {
                segundos: segundosDePreparacion(campanas.revision.audiencia),
              })
            "
          />
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

/* ── El horario que se crea desde aquí (H-06) ───────────────────────────── */
.asistente__horario-acciones {
  display: flex;
  flex-wrap: wrap;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-3);
}

.asistente__horario-propio {
  margin-top: var(--arles-space-4);
}

.asistente__horario-propio[open] > *:not(summary) {
  margin-top: var(--arles-space-3);
}

.asistente__horas {
  display: flex;
  gap: var(--arles-space-3);
  flex-wrap: wrap;
}

.asistente__horas > * {
  flex: 1 1 8rem;
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
