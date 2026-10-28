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
import { useListasStore } from '@/app/stores/listas'
import { useEmpresaStore } from '@/app/stores/empresa'
import { useRemitentesStore } from '@/app/stores/remitentes'
import { useWhatsappStore } from '@/app/stores/whatsapp'
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
const whatsapp = useWhatsappStore()
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
const listasStore = useListasStore()
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

/**
 * Por dónde sale esta campaña (D-WA, 27/10/2026).
 *
 * ── POR QUÉ SE PREGUNTA EN EL PASO 1 Y NO MÁS ADELANTE ───────────────────
 *
 * Porque **decide tres pasos**: el remitente son cuentas de correo o números de
 * WhatsApp, el mensaje son plantillas nuestras o plantillas aprobadas por Meta,
 * y el envío de prueba va a un correo o a un teléfono. Preguntarlo después
 * obligaría a volver atrás a rehacer lo ya elegido.
 *
 * Y empieza en correo porque es lo que el producto hace desde la Fase 5 y lo
 * que más se usa — pero se pregunta, no se asume: una campaña de WhatsApp
 * mandada por correo sin que nadie lo eligiera es el error que no se puede
 * deshacer.
 */
const canal = ref<'email' | 'whatsapp'>('email')
const esWhatsapp = computed(() => canal.value === 'whatsapp')

const cuenta = ref('')
const versionDePlantilla = ref('')
/** El número de WhatsApp remitente. Vacío hasta que se elige. */
const numero = ref('')
/** La plantilla aprobada de Meta. Vacía hasta que se elige. */
const plantillaDeMeta = ref('')
const ventana = ref('')
// A quién va dirigida (D-15). Vacío es «toda la empresa», que es lo que
// hacían todas las campañas hasta el 23/10/2026.
const lista = ref('')
// La firma del pie. Vacío es «la de por defecto de la empresa», que es lo que
// promete su casilla en Plantillas (H-12).
const firma = ref('')
const limiteDiario = ref('')
// ── EL POR HORA YA NO ES UN CAMPO: ES UNA CONSECUENCIA (D-17, 22/10/2026) ──
//
// Decisión de Dirección, con sus palabras: *«que calcule automáticamente ARLES
// el número de correos por hora, yo sólo le asigno el horario con el que quiero
// enviar la campaña… pero el valor de entrada que se respete es el de correos
// por día»*.
//
// Pedir los dos números era la causa del defecto H-2 de la auditoría del cupo:
// con el por hora de la cuenta —diez— y nueve horas de ventana, quien escribía
// 500 al día recibía 90. El arreglo de esa auditoría fue calcularlo y avisar si
// alguien lo bajaba; esto va un paso más allá y **quita la posibilidad de
// bajarlo**, así que el defecto no puede volver por ninguna puerta.
//
// Y no se pierde nada: el cálculo REPARTE por definición. 500 en nueve horas
// son 56 por hora, que es exactamente «repartido a lo largo del día» — lo que
// el campo servía para conseguir, cuando alguien sabía hacer la división.
//
// Sigue siendo un `ref` y no un `computed` porque hay un caso en que no se
// puede calcular: sin horario elegido no hay horas entre las que repartir. Ahí
// vale lo que trae la cuenta, hasta que se elija uno.
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
  // Y las listas, para el paso 2 (D-15).
  void listasStore.cargar()
  // Y los números de WhatsApp, para el paso 3 cuando el canal sea ése (D-WA).
  void whatsapp.cargar()
})

const error = computed(() =>
  campanas.errorGeneral ? resolverError({ clave: campanas.errorGeneral, detalle: '' }) : null,
)

// ── Las opciones de los desplegables ──

/**
 * A quién va dirigida, con «toda la empresa» la primera (D-15).
 *
 * ── POR QUÉ SÓLO LAS QUE TIENEN GENTE ───────────────────────────────────
 *
 * Una lista vacía activa una campaña que para con `AudienciaVacia` nueve pasos
 * después. Ofrecerla es construir un camino que termina en un rechazo, que es
 * el defecto que este asistente ya tuvo con las cuentas rotas en el paso 3 y
 * con la plantilla en el paso 4.
 *
 * Y el nombre **nunca va solo**: lleva su cifra (§94). «Videovigilancia» puede
 * tener trescientos o doce, y elegirla sin saberlo es elegir a ciegas justo en
 * la decisión de a cuánta gente se le escribe.
 */
const listasConGente = computed(() => listasStore.conGente)

const opcionesDeLista = computed(() => [
  { valor: '', texto: t('asistente.paso.audiencia.todaLaEmpresa') },
  ...listasConGente.value.map((l) => ({
    valor: l.id,
    texto: t('asistente.paso.audiencia.conGente', {
      nombre: l.nombre,
      n: l.cuantos.toLocaleString('es-MX'),
    }),
  })),
])

/**
 * Los dos canales.
 *
 * Se ofrecen los dos siempre, aunque no haya remitente de uno: el paso 3 avisa
 * y dice dónde se conecta. Esconder el canal de WhatsApp cuando no hay números
 * dejaría a quien quiere usarlo sin saber que existe.
 */
const opcionesDeCanal = computed(() => [
  { valor: 'email', texto: t('asistente.paso.informacion.canalCorreo') },
  { valor: 'whatsapp', texto: t('asistente.paso.informacion.canalWhatsapp') },
])

const opcionesDeCuenta = computed(() =>
  remitentes.remitentes
    // Sólo las que pueden enviar. Ofrecer una cuenta rota es construir una
    // campaña que el preflight va a bloquear tres pasos después.
    .filter((r) => r.estado === 'connected')
    .map((r) => ({ valor: r.id, texto: r.remitente })),
)

/**
 * Los números que pueden mandar una campaña **hoy**.
 *
 * `puedeEnviar` **viene resuelto de Rust**: la puerta del canal son tres
 * condiciones —conectado, Coexistencia confirmada, y el riesgo aceptado si
 * escribe en frío— y vive en `NumeroGuardado::puede_enviar`. Recalcularla aquí
 * sería una segunda verdad que puede separarse de la que decide.
 *
 * Y el rótulo lleva **las dos cifras que deciden si un número puede trabajar**
 * (§94): su calidad y cuántos permite Meta al día. Elegir sin verlas es elegir
 * a ciegas justo donde se juega el número.
 */
const opcionesDeNumero = computed(() =>
  whatsapp.numeros
    .filter((n) => n.puedeEnviar)
    .map((n) => ({
      valor: n.id,
      texto: t('asistente.paso.remitente.numeroCon', {
        nombre: n.nombreVisible,
        telefono: n.telefonoE164,
        calidad:
          n.calidad === null
            ? t('numeroDeWhatsapp.calidad.sinConsultar')
            : t(`numeroDeWhatsapp.calidad.${n.calidad}`),
        tope:
          n.limiteDeMeta === null
            ? t('asistente.paso.remitente.topeSinSaber')
            : n.limiteDeMeta.toLocaleString('es-MX'),
      }),
    })),
)

/**
 * Las plantillas de Meta **que se pueden usar**.
 *
 * Sólo las aprobadas: `activar_campana` rechaza las demás (tramo 3a), así que
 * ofrecerlas sería construir un camino que termina en un rechazo cuatro pasos
 * después — el defecto que este asistente ya tuvo con las cuentas rotas y con
 * la plantilla de correo.
 *
 * Las que están en revisión **sí se ven en Plantillas**, que es donde hay que
 * poder seguirles la pista.
 */
const opcionesDePlantillaDeMeta = computed(() =>
  whatsapp.plantillasDeMeta
    .filter((p) => p.sePuedeUsar)
    .map((p) => ({ valor: p.id, texto: `${p.nombre} · ${p.idioma}` })),
)

/** La plantilla de Meta elegida, para poder enseñar su texto y sus huecos. */
const plantillaDeMetaElegida = computed(
  () => whatsapp.plantillasDeMeta.find((p) => p.id === plantillaDeMeta.value) ?? null,
)

// Al cambiar de número hay que releer sus plantillas: la clave de Meta es
// (número, nombre, idioma), así que las de un número no son las de otro.
// Y se olvida la elegida: dejarla haría crear una campaña cuya plantilla es de
// otro número, y `activar_campana` la rechazaría al final.
watch(numero, (id) => {
  plantillaDeMeta.value = ''
  if (id.length > 0) void whatsapp.cargarPlantillasDeMeta(id)
})

// Y al cambiar de canal se olvida lo elegido del otro. Sin esto, quien empiece
// en correo, elija cuenta y plantilla, y cambie a WhatsApp, mandaría una
// campaña con los dos juegos de datos dentro — y la variante etiquetada de la
// frontera la rechazaría sin decir cuál sobra.
watch(canal, () => {
  cuenta.value = ''
  versionDePlantilla.value = ''
  firma.value = ''
  numero.value = ''
  plantillaDeMeta.value = ''
})

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
  // El por hora de la cuenta sólo se usa mientras no haya horario elegido: en
  // cuanto lo haya, el `watch` de abajo lo calcula del diario.
  if (limitePorHora.value.trim() === '') limitePorHora.value = String(r.limitePorHora)
})

// ──────────────────────────────────────────────────────────────────────────
// EL TOPE DE LA HORA NO PUEDE IMPEDIR QUE EL DEL DÍA SE CUMPLA
//
// Medido el 22/10/2026, y es el defecto que más caro salía de los cinco de esa
// auditoría: con lo que esta misma pantalla rellena sola —diez por hora, el de
// la cuenta— y la ventana de nueve a dieciocho,
//
//     se pide 500 al día  →  salen 90
//     se pide 100 al día  →  salen 90
//     se pide  50 al día  →  salen 50
//
// o sea que CUALQUIER TOPE POR ENCIMA DE NOVENTA no se cumplía. Y no se veía
// porque con los 50 del §48 todo cuadra: el defecto empieza justo donde
// Dirección quiere trabajar.
//
// Se cierra por los dos lados, porque uno solo no basta:
//
//   · el por hora **se calcula** del diario y de la ventana, para que no haya
//     que saberse esta aritmética para usar el producto;
//   · y si alguien lo deja por debajo a mano, **se avisa con las dos cifras**
//     (§154: informar, no dictar — bajarlo puede ser lo que se quiere, mandar
//     menos de lo que se pidió sin saberlo nunca lo es).

/** La ventana elegida, para saber cuántas horas de envío tiene al día. */
const ventanaElegida = computed(() =>
  campanas.ventanas.find((v) => v.id === ventana.value),
)

/**
 * Cuántos por hora hacen falta para que el tope del día se alcance.
 *
 * **Redondea hacia arriba**, igual que `por_hora_que_hace_falta` en el núcleo:
 * 500 en nueve horas son 55.5, y con 55 salen 495. La cifra viaja por la
 * frontera en `techoDiario`, así que el día que las dos cuentas no coincidan
 * lo dirá el propio paso 6 — no hay una segunda regla que mantener a mano.
 */
const porHoraQueHaceFalta = computed(() => {
  const v = ventanaElegida.value
  const dia = diarioComoNumero.value
  if (!v || dia === null || dia < 1) return null
  const horas = Math.max(0, v.hasta - v.desde)
  if (horas === 0) return dia
  return Math.ceil(dia / horas)
})

/**
 * Al cambiar el tope diario o la ventana, el por hora se recalcula.
 *
 * ── POR QUÉ PISA LO QUE HAYA, AL CONTRARIO QUE EL `watch` DE ARRIBA ──
 *
 * Aquél no pisa porque el número que hay es **una decisión**: alguien escribió
 * 300. Éste sí pisa porque lo que hay es **una consecuencia**: el por hora de
 * la cuenta, que ya no cuadra con el diario nuevo. Dejarlo quieto es lo que
 * producía los 90 de 500.
 *
 * Quien de verdad quiera un por hora más bajo lo escribe después y se queda:
 * este `watch` sólo reacciona a que cambie el diario o la ventana, no a que se
 * toque el por hora. Y entonces salta el aviso.
 */
watch([diarioComoNumero, ventanaElegida], () => {
  const hace_falta = porHoraQueHaceFalta.value
  if (hace_falta !== null) limitePorHora.value = String(hace_falta)
})

/**
 * El techo real del día, cruzando los dos topes con las horas de la ventana.
 *
 * Es la misma cuenta que `techo_diario` en el núcleo. Se hace aquí para poder
 * avisar **antes** de llamar a nadie, que es la misma razón por la que
 * `AVISO_DIARIO` está duplicado — y, como aquél, hay una comprobación del
 * validador que falla si las dos cuentas se separan.
 */
const techoRealDelDia = computed(() => {
  const v = ventanaElegida.value
  const dia = diarioComoNumero.value
  const hora = Number.parseInt(limitePorHora.value, 10)
  if (!v || dia === null || !Number.isFinite(hora) || hora < 1) return null
  const horas = Math.max(0, v.hasta - v.desde)
  return Math.min(dia, hora * horas)
})

/**
 * `true` cuando el tope de la hora deja el del día sin alcanzar.
 *
 * ── ESTO YA NO DEBERÍA PODER PASAR, Y POR ESO SE QUEDA ───────────────────
 *
 * Desde la D-17 el por hora no se escribe: lo calcula ARLES del diario y del
 * horario, así que siempre es el que hace falta. Este aviso, entonces, **no
 * tiene que aparecer nunca**.
 *
 * Se queda por lo mismo que se queda una alarma en una puerta que ya está
 * cerrada: si algún día vuelve a haber una forma de escribir el por hora —un
 * campo avanzado, un valor que llegue de la base, una etapa creada por otro
 * camino—, lo que no puede pasar es que **salgan menos correos de los pedidos y
 * nadie lo diga**. Eso ya pasó una vez, con 90 de 500.
 *
 * Y mientras no pueda pasar, su prueba lo comprueba sembrando el valor a mano:
 * una alarma que nunca se ha visto sonar no está verificada.
 */
const laHoraFrenaElDia = computed(
  () =>
    techoRealDelDia.value !== null &&
    diarioComoNumero.value !== null &&
    techoRealDelDia.value < diarioComoNumero.value,
)

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
      // El remitente, del canal que sea.
      return esWhatsapp.value ? numero.value.length > 0 : cuenta.value.length > 0
    case 4:
      // El mensaje. En WhatsApp **no hay firma**: una firma es cosa del correo
      // y el esquema lo exige con un `CHECK` desde la V4.
      return esWhatsapp.value
        ? plantillaDeMeta.value.length > 0
        : versionDePlantilla.value.length > 0
    case 5:
      return (
        ventana.value.length > 0 &&
        !limiteImposible.value &&
        (!avisaDelVolumen.value || aceptoElRiesgo.value)
      )
    case 6: {
      // En WhatsApp no hay estimación de duración: su revisión no la trae. El
      // desenlace imposible lo sigue rechazando `activar_campana`, que es donde
      // vive la regla — aquí lo que no se puede es enseñar una cifra que nadie
      // ha calculado.
      if (esWhatsapp.value) return true
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

/**
 * Por qué «Continuar» está apagado, para poder decirlo.
 *
 * ── POR QUÉ HIZO FALTA ────────────────────────────────────────────────────
 *
 * Medido el 28/10/2026 en el paso 1: el botón salía `disabled` **sin `title`,
 * sin `aria-describedby` y sin ningún texto cerca**. Para quien usa un lector
 * de pantalla no había absolutamente nada.
 *
 * Es la cuarta vez de este defecto en el proyecto —«Repite la contraseña» en
 * la 8.2c, «Restaurar y reemplazar» en la 8.3c, «Continuar» del paso 6 en la
 * 8.6 E— y las tres anteriores se arreglaron por la misma razón: un control
 * que no responde y no se explica se lee como una avería (§95).
 *
 * Devuelve **una clave**, no un texto (§139). `null` cuando se puede seguir.
 *
 * Los pasos 5 y 6 ya pintan su aviso encima con el detalle entero; esto les
 * añade el renglón corto pegado al botón, que es lo que contesta «¿y por qué
 * no puedo pulsar?» sin levantar la vista.
 */
const porQueNoSeSigue = computed<string | null>(() => {
  if (sePuedeSeguir.value) return null
  switch (paso.value) {
    case 1:
      return 'asistente.falta.nombre'
    case 3:
      return esWhatsapp.value ? 'asistente.falta.numero' : 'asistente.falta.cuenta'
    case 4:
      return esWhatsapp.value ? 'asistente.falta.plantillaDeMeta' : 'asistente.falta.plantilla'
    case 5:
      if (ventana.value.length === 0) return 'asistente.falta.horario'
      if (limiteImposible.value) return 'asistente.falta.limite'
      return 'asistente.falta.aviso'
    case 6:
      return 'asistente.falta.desenlace'
    case 8:
      return 'asistente.falta.prueba'
    default:
      return null
  }
})

/** ¿El simulador dice que esta campaña llega al final? */
const termina = computed(
  () => campanas.revision?.estimacion.desenlace.desenlace === 'termina',
)

/**
 * Lo que dice el preflight. `null` mientras no se haya pedido.
 *
 * **Son dos veredictos distintos y no se mezclan.** El de correo sale de los
 * ocho puntos; el de WhatsApp de su propia puerta —Coexistencia, el riesgo
 * aceptado y las plantillas aprobadas—, que es `se_puede_activar` resuelto en
 * Rust. Recalcularlo aquí sería una segunda verdad sobre si se puede enviar.
 */
const sePuedeActivar = computed(() =>
  esWhatsapp.value
    ? campanas.revisionDeWhatsapp?.sePuedeActivar === true
    : campanas.revision?.preflight.resultado.resultado === 'lista',
)

// ── Navegación ──

async function siguiente(): Promise<void> {
  if (!sePuedeSeguir.value) return

  // Al salir del 5 se crea la campaña: del 6 en adelante todo lee de la base.
  if (paso.value === 5 && creada.value === null) {
    const r = await campanas.crearCampana({
      nombre: nombre.value.trim(),
      // Por dónde sale (D-WA). **Hoy este asistente sólo ofrece correo**: la
      // elección de canal es el tramo 3c. Va ya con la forma etiquetada
      // porque es la que la frontera acepta, y porque así el día que entre
      // WhatsApp TypeScript exige decidir en vez de dejar colar una mezcla.
      // Por dónde sale (D-WA). La forma **etiquetada**: TypeScript no deja
      // construir una mezcla, y la frontera tampoco la deserializaría.
      salePor: esWhatsapp.value
        ? {
            canal: 'whatsapp',
            numero: numero.value,
            plantillaDeMeta: plantillaDeMeta.value === '' ? null : plantillaDeMeta.value,
          }
        : {
            canal: 'email',
            cuenta: cuenta.value,
            versionDePlantilla: versionDePlantilla.value,
            // Vacío es «la de por defecto»: `null`, no una cadena vacía, que la
            // clave ajena rechazaría.
            firma: firma.value === '' ? null : firma.value,
          },
      // Vacío es «toda la empresa»: `null`, no una cadena vacía, que la clave
      // ajena rechazaría. Mismo trato que la firma.
      lista: lista.value === '' ? null : lista.value,
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
    // El preflight del canal que sea. Son dos comandos distintos porque son
    // dos preguntas distintas: el de correo sondea el servidor SMTP y los
    // registros del dominio; el de WhatsApp mira **las dos cifras de permiso**,
    // Coexistencia, el riesgo aceptado y cuántas plantillas hay aprobadas.
    //
    // `revisar_campana_de_whatsapp` existía desde W.5 y estuvo **desregistrado
    // desde la 7.7** con este motivo escrito: era «la puerta de un flujo que no
    // existe». Vuelve aquí, con su llamador.
    if (esWhatsapp.value) {
      await campanas.revisarWhatsapp(creada.value.id)
    } else {
      await campanas.revisar(creada.value.id)
    }
  }

  paso.value = Math.min(paso.value + 1, PASOS.length)
}

function atras(): void {
  paso.value = Math.max(paso.value - 1, 1)
}

async function mandarLaPrueba(): Promise<void> {
  if (!creada.value) return

  // ── LA PRUEBA DE WHATSAPP CUELGA DEL NÚMERO, NO DE LA ETAPA ───────────
  //
  // Y no es un detalle de implementación: `enviar_whatsapp_de_prueba` le pide a
  // Meta que mande **esa plantilla** desde **ese número**, que es justo lo que
  // hay que comprobar antes de activar. El correo de prueba cuelga de la etapa
  // porque lo que se prueba allí es la redacción con su firma.
  if (esWhatsapp.value) {
    const p = plantillaDeMetaElegida.value
    if (p === null) return
    const r = await whatsapp.enviarPrueba(
      numero.value,
      direccionDePrueba.value.trim(),
      p.nombre,
      p.idioma,
    )
    // Las dos formas se igualan aquí: la de WhatsApp devuelve `{aceptado,
    // clave}` y la del correo `{aceptado, problema}`. Igualarlas en la pantalla
    // y no en los comandos es a propósito — son dos proveedores distintos y
    // forzarlos a un tipo común escondería en qué se diferencian.
    resultadoDePrueba.value =
      r === null ? null : { aceptado: r.aceptado, problema: r.clave }
    if (resultadoDePrueba.value?.aceptado) {
      await campanas.revisarWhatsapp(creada.value.id)
    }
    return
  }

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

/**
 * Pide que la activación en curso pare.
 *
 * No deshace nada desde aquí: levanta una bandera que el núcleo consulta entre
 * tandas, y quien deshace es su transacción. Por eso cancelar **no puede dejar
 * una campaña a medio activar** — está probado en `arles-db` comparando el
 * recuento de las cuatro tablas antes y después.
 *
 * No se mira lo que devuelve: `false` significa que ya había terminado, que es
 * la carrera normal de pulsar mientras la barra se mueve, y en ese caso el
 * resultado de la activación ya está en camino por el valor del comando.
 */
async function cancelar(): Promise<void> {
  await campanas.cancelarLaActivacion()
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

        <!-- ── EL CANAL (D-WA, 27/10/2026) ─────────────────────────────
             Va aquí y no más adelante porque **decide tres pasos**: el
             remitente, el mensaje y el envío de prueba. Preguntarlo después
             obligaría a volver atrás a rehacer lo ya elegido. -->
        <ASelector
          v-model="canal"
          :etiqueta="t('asistente.paso.informacion.canal')"
          :opciones="opcionesDeCanal"
          :ayuda="t('asistente.paso.informacion.canalAyuda')"
        />

        <!-- Lo que cambia al elegir WhatsApp, dicho antes de elegirlo y no
             después: por WhatsApp sólo se manda una plantilla que Meta aprobó,
             y eso no es un detalle — es a quién se le puede escribir. -->
        <AAviso
          v-if="esWhatsapp"
          tono="info"
          :titulo="t('asistente.paso.informacion.whatsappEsDistinto')"
        >
          <p>{{ t('asistente.paso.informacion.whatsappEsDistintoDetalle') }}</p>
        </AAviso>
      </template>

      <!-- ── 2 · A quién va dirigida (D-15) ──────────────────────────────
           Este paso existía desde la 6.6 y no se podía elegir nada: decía «en
           esta versión no hay listas ni filtros» y toda campaña iba a todo el
           padrón. Ahora se elige, y «toda la empresa» sigue estando — como una
           opción que se toma, no como lo que pasa por omisión. -->
      <template v-else-if="paso === 2">
        <p>{{ t('asistente.paso.audiencia.explicacion') }}</p>

        <ASelector
          v-model="lista"
          :etiqueta="t('asistente.paso.audiencia.campo')"
          :opciones="opcionesDeLista"
          :ayuda="t('asistente.paso.audiencia.ayuda')"
        />

        <!-- Sin ninguna lista cargada, el desplegable tiene una sola opción y
             ninguna pista de dónde salen las demás. Es el H-06 en otro paso, y
             lo que lo arregla es decir dónde se hace Y llevar allí. -->
        <AAviso
          v-if="!listasConGente.length"
          tono="info"
          :titulo="t('asistente.paso.audiencia.sinListas')"
        >
          <p>{{ t('asistente.paso.audiencia.sinListasDetalle') }}</p>
          <ABoton
            variante="sutil"
            tamano="compacto"
            @click="router.push('/contactos/importar')"
          >
            {{ t('asistente.paso.audiencia.irAImportar') }}
          </ABoton>
        </AAviso>

        <!-- Y si se elige «toda la empresa» habiendo listas, se dice lo que
             eso significa. Elegirlo tiene que ser una decisión, no un
             descuido. -->
        <AAviso
          v-else-if="lista === ''"
          tono="aviso"
          :titulo="t('asistente.paso.audiencia.alcance')"
        >
          <p>{{ t('asistente.paso.audiencia.alcanceDetalle') }}</p>
        </AAviso>
      </template>

      <!-- 3 · Remitente · del canal que sea -->
      <template v-else-if="paso === 3">
        <!-- ── WHATSAPP ────────────────────────────────────────────────
             El rótulo de cada número lleva **las dos cifras que deciden si
             puede trabajar**: su calidad y cuántos permite Meta al día (§94).
             Elegir sin verlas es elegir a ciegas justo donde se juega el
             número, que no se recupera. -->
        <template v-if="esWhatsapp">
          <ASelector
            v-model="numero"
            :etiqueta="t('asistente.paso.remitente.numero')"
            :opciones="opcionesDeNumero"
            :ayuda="t('asistente.paso.remitente.numeroAyuda')"
          />
          <!-- Sin ningún número que pueda enviar, esto sería un desplegable
               vacío sin salida: el H-06 del 22/10/2026. Se dice qué falta Y se
               lleva allí. -->
          <AAviso
            v-if="opcionesDeNumero.length === 0"
            tono="aviso"
            :titulo="t('asistente.paso.remitente.sinNumeros')"
          >
            <p>{{ t('asistente.paso.remitente.sinNumerosComo') }}</p>
            <ABoton
              variante="secundario"
              @click="router.push({ name: 'whatsapp' })"
            >
              {{ t('asistente.paso.remitente.irAWhatsApp') }}
            </ABoton>
          </AAviso>
        </template>

        <!-- ── CORREO ─────────────────────────────────────────────────── -->
        <template v-else>
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
        </template>

        <!-- ADR-0008: ARLES no rota remitentes, y se dice donde se elige uno.
             Vale para los dos canales — y en WhatsApp más, porque el que se
             quema es el número. -->
        <p class="asistente__nota">
          {{ t('asistente.paso.remitente.sinRotacion') }}
        </p>
      </template>

      <!-- 4 · Mensaje · del canal que sea -->
      <template v-else-if="paso === 4">
        <!-- ── WHATSAPP: UNA PLANTILLA QUE META APROBÓ, Y NADA MÁS ─────
             No hay firma —el esquema lo exige con un `CHECK` desde la V4— ni
             texto que escribir: el de una plantilla de Meta lo aprueba Meta
             palabra por palabra, y lo que ARLES manda es su nombre y los
             valores de sus huecos.

             Sólo se ofrecen las APROBADAS: `activar_campana` rechaza las demás
             (tramo 3a), así que ofrecerlas sería construir un camino que
             termina en un rechazo cuatro pasos después. -->
        <template v-if="esWhatsapp">
          <ASelector
            v-model="plantillaDeMeta"
            :etiqueta="t('asistente.paso.mensaje.plantillaDeMeta')"
            :opciones="opcionesDePlantillaDeMeta"
            :ayuda="t('asistente.paso.mensaje.plantillaDeMetaAyuda')"
          />

          <!-- Sin ninguna aprobada, el mismo H-06: se dice qué falta y se
               lleva allí. Y se distingue «no has apuntado ninguna» de «las que
               tienes están en revisión», que mandan a hacer cosas distintas. -->
          <AAviso
            v-if="opcionesDePlantillaDeMeta.length === 0"
            tono="aviso"
            :titulo="t('asistente.paso.mensaje.sinAprobadas')"
          >
            <p>
              {{
                whatsapp.plantillasDeMeta.length === 0
                  ? t('asistente.paso.mensaje.sinAprobadasNinguna')
                  : t('asistente.paso.mensaje.sinAprobadasEnRevision')
              }}
            </p>
            <ABoton
              variante="secundario"
              @click="router.push({ name: 'plantillas' })"
            >
              {{ t('asistente.paso.mensaje.irAPlantillasDeMeta') }}
            </ABoton>
          </AAviso>

          <!-- El texto que va a salir, y qué dato va en cada hueco. Es lo
               único que se puede enseñar antes de mandar: el cuerpo no se
               puede cambiar desde aquí. -->
          <div
            v-if="plantillaDeMetaElegida"
            class="asistente__variables"
          >
            <p>{{ plantillaDeMetaElegida.cuerpo }}</p>
            <p v-if="plantillaDeMetaElegida.huecos.length > 0">
              {{ t('asistente.paso.mensaje.variables') }}:
              <AInsignia
                v-for="(h, i) in plantillaDeMetaElegida.huecos"
                :key="`${plantillaDeMetaElegida.id}-${i}`"
                tono="info"
              >
                {{ i + 1 }} · {{ rotulo(`plantilla.variable.${h}`) }}
              </AInsignia>
            </p>
          </div>
        </template>

        <!-- ── CORREO ─────────────────────────────────────────────────── -->
        <template v-else>
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
        <!--
          EL REPARTO POR HORA, QUE YA NO SE PIDE (D-17, 22/10/2026).

          Aquí había un segundo campo —«máximo por hora»— y era la causa del
          defecto H-2: se rellenaba con el de la cuenta, diez, y con nueve
          horas de ventana hacía que un tope diario de 500 entregara 90.

          Dirección decidió que ARLES lo calcule: el valor de entrada es el de
          correos por día, y el horario. Así que esto **no es un campo, es una
          consecuencia**, y se enseña como tal — no se puede escribir, y por eso
          el defecto no puede volver por ninguna puerta.

          Se enseña y no se esconde porque es la cifra que explica el ritmo:
          quien ve «unos 56 por hora» entiende que no sale todo de golpe, que
          es justo lo que el campo servía para conseguir cuando alguien sabía
          hacer la división.
        -->
        <p
          v-if="porHoraQueHaceFalta !== null"
          class="asistente__frase"
        >
          {{ t('asistente.paso.limites.repartoPorHora', {
            n: porHoraQueHaceFalta.toLocaleString('es-MX'),
            horas: (ventanaElegida?.hasta ?? 0) - (ventanaElegida?.desde ?? 0),
          }) }}
        </p>

        <!--
          LA ALARMA DE LA PUERTA QUE YA ESTÁ CERRADA.

          Desde la D-17 el por hora no se escribe, así que esto no tiene que
          aparecer nunca. Se queda porque lo que no puede volver a pasar es que
          salgan menos correos de los pedidos y nadie lo diga — ya pasó una vez,
          con 90 de 500. Si algún día vuelve a haber una forma de fijarlo (un
          campo avanzado, un valor que llegue de la base), el aviso está puesto.

          Su prueba lo siembra a mano, porque una alarma que nunca se ha visto
          sonar no está verificada.
        -->
        <AAviso
          v-if="laHoraFrenaElDia"
          tono="aviso"
          :titulo="t('asistente.paso.limites.laHoraFrena.titulo')"
        >
          <p>
            {{ t('asistente.paso.limites.laHoraFrena.cuerpo', {
              pedido: (diarioComoNumero ?? 0).toLocaleString('es-MX'),
              salen: (techoRealDelDia ?? 0).toLocaleString('es-MX'),
              porHora: limitePorHora,
              horas: (ventanaElegida?.hasta ?? 0) - (ventanaElegida?.desde ?? 0),
            }) }}
          </p>
          <p>
            {{ t('asistente.paso.limites.laHoraFrena.comoSeArregla', {
              n: (porHoraQueHaceFalta ?? 0).toLocaleString('es-MX'),
              pedido: (diarioComoNumero ?? 0).toLocaleString('es-MX'),
            }) }}
          </p>
        </AAviso>

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
        <!-- ── EL PREFLIGHT DEL CANAL DE WHATSAPP (W.5 → D-WA 3c-2b) ───
             No son los ocho puntos del correo: es la puerta del canal. Y la
             cifra que de verdad decide es **a cuánta gente no le consta
             permiso**, porque ésa es la que se paga en calidad del número si
             bloquean o reportan — y el número no se recupera.

             `revisar_campana_de_whatsapp` existía desde W.5 y estuvo
             desregistrado desde la 7.7 por no tener quien lo llamara. Éste es
             el llamador. -->
        <template v-if="esWhatsapp">
          <template v-if="campanas.revisionDeWhatsapp">
            <ul class="asistente__puntos">
              <li>
                <AInsignia tono="neutro">
                  {{ campanas.revisionDeWhatsapp.audiencia.toLocaleString('es-MX') }}
                </AInsignia>
                {{ t('asistente.paso.preflight.wa.audiencia') }}
              </li>
              <li>
                <AInsignia
                  :tono="campanas.revisionDeWhatsapp.sinPermiso === 0 ? 'exito' : 'aviso'"
                >
                  {{ campanas.revisionDeWhatsapp.sinPermiso.toLocaleString('es-MX') }}
                </AInsignia>
                {{ t('asistente.paso.preflight.wa.sinPermiso') }}
              </li>
              <li>
                <AInsignia :tono="campanas.revisionDeWhatsapp.coexistencia ? 'exito' : 'peligro'">
                  {{ t(`asistente.paso.preflight.wa.${campanas.revisionDeWhatsapp.coexistencia ? 'si' : 'no'}`) }}
                </AInsignia>
                {{ t('asistente.paso.preflight.wa.coexistencia') }}
              </li>
              <li v-if="campanas.revisionDeWhatsapp.escribeEnFrio">
                <AInsignia :tono="campanas.revisionDeWhatsapp.riesgoAceptado ? 'exito' : 'peligro'">
                  {{ t(`asistente.paso.preflight.wa.${campanas.revisionDeWhatsapp.riesgoAceptado ? 'si' : 'no'}`) }}
                </AInsignia>
                {{ t('asistente.paso.preflight.wa.riesgo') }}
              </li>
              <li>
                <AInsignia
                  :tono="campanas.revisionDeWhatsapp.plantillasAprobadas > 0 ? 'exito' : 'peligro'"
                >
                  {{ campanas.revisionDeWhatsapp.plantillasAprobadas }}
                </AInsignia>
                {{ t('asistente.paso.preflight.wa.plantillas') }}
              </li>
              <li>
                <!-- `null` es «no se pudo preguntar», NO «está bien»: la misma
                     regla que el «sin comprobar» del preflight de correo. -->
                <AInsignia
                  :tono="campanas.revisionDeWhatsapp.calidad === null ? 'incierto' : 'neutro'"
                >
                  {{
                    campanas.revisionDeWhatsapp.calidad === null
                      ? t('numeroDeWhatsapp.calidad.sinConsultar')
                      : t(`numeroDeWhatsapp.calidad.${campanas.revisionDeWhatsapp.calidad}`)
                  }}
                </AInsignia>
                {{ t('asistente.paso.preflight.wa.calidad') }}
              </li>
            </ul>
            <!-- El aviso que más importa de esta pantalla, y va con su cifra
                 delante: escribir a quien no dio permiso es legítimo y es lo
                 que más calidad cuesta. §154 — se informa, no se prohíbe. -->
            <AAviso
              v-if="campanas.revisionDeWhatsapp.sinPermiso > 0"
              tono="aviso"
              :titulo="t('asistente.paso.preflight.wa.enFrioTitulo')"
            >
              <p>
                {{
                  t('asistente.paso.preflight.wa.enFrioDetalle', {
                    n: campanas.revisionDeWhatsapp.sinPermiso.toLocaleString('es-MX'),
                  })
                }}
              </p>
            </AAviso>
          </template>
        </template>

        <ul
          v-else-if="campanas.revision"
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
        <!-- «Sin comprobar» no es un aprobado, y el texto lo dice. Es del
             preflight de correo: el de WhatsApp no tiene puntos de red. -->
        <p
          v-if="!esWhatsapp"
          class="asistente__nota"
        >
          {{ t('preflight.sinComprobarDetalle') }}
        </p>
      </template>

      <!-- 8 · Envío de prueba · del canal que sea -->
      <template v-else-if="paso === 8">
        <p>
          {{
            esWhatsapp
              ? t('asistente.paso.prueba.porQueWhatsapp')
              : t('asistente.paso.prueba.porQue')
          }}
        </p>
        <AEntrada
          v-model="direccionDePrueba"
          :etiqueta="
            esWhatsapp
              ? t('asistente.paso.prueba.aDondeWhatsapp')
              : t('asistente.paso.prueba.aDonde')
          "
          :ayuda="esWhatsapp ? t('asistente.paso.prueba.aDondeWhatsappAyuda') : ''"
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
          <!--
            EL AVANCE, Y POR QUÉ NO ES UNA BARRA DE PORCENTAJE

            Activar tarda 73.5 s con 451 196 destinatarios y rompe su
            presupuesto de 2 s hacia los 30 000 (auditoría C). Medido por
            tramos el 28/10/2026: el bucle de la cola es el 52-57 % del tiempo
            y el ÚNICO que se puede contar fila a fila; el INSERT de la
            audiencia (33-38 %) es una sola sentencia y el COMMIT (7-9 %)
            tampoco se subdivide.

            Una barra de 0 a 100 estaría quieta el 45 % de la espera y después
            saltaría. Así que se dice EN QUÉ VA con palabras, y la cifra sólo
            aparece en la fase que la tiene. Es el §94: cifras y no adjetivos,
            y menos una cifra que no mide nada.
          -->
          <div
            v-if="campanas.comoVa"
            class="asistente__avance"
          >
            <p class="asistente__frase">
              {{ t(`asistente.paso.activacion.fase.${campanas.comoVa.fase}`) }}
            </p>
            <p
              v-if="campanas.comoVa.van !== undefined && campanas.comoVa.de !== undefined"
              class="asistente__nota"
            >
              {{
                t('asistente.paso.activacion.vanDe', {
                  van: campanas.comoVa.van.toLocaleString('es-MX'),
                  de: campanas.comoVa.de.toLocaleString('es-MX'),
                })
              }}
            </p>
            <!--
              El botón se apaga con lo que manda el núcleo y no con la clave de
              la fase: quien sabe si cancelar sirve es quien tiene el dato. En
              `cerrando` ya no sirve, y un botón encendido que no hace nada es
              el §95 al revés.
            -->
            <ABoton
              variante="secundario"
              :deshabilitado="!campanas.comoVa.sePuedeCancelar"
              :title="
                campanas.comoVa.sePuedeCancelar
                  ? undefined
                  : t('asistente.paso.activacion.yaNoSePuedeCancelar')
              "
              @click="cancelar"
            >
              {{ t('asistente.paso.activacion.cancelar') }}
            </ABoton>
          </div>

          <ABoton
            v-else
            variante="primario"
            :deshabilitado="!sePuedeActivar"
            :ocupado="campanas.trabajando"
            @click="activar"
          >
            {{ t('asistente.paso.activacion.activar') }}
          </ABoton>
          <p
            v-if="!sePuedeActivar && !campanas.comoVa"
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
      <!-- `title` y no un renglón aparte: en el paso 1 «Atrás» está apagado
           porque no hay paso anterior, y eso ya lo dice «Paso 1 de 9». Lo que
           faltaba es que lo supiera quien no ve la pantalla. -->
      <ABoton
        variante="sutil"
        :deshabilitado="paso === 1"
        :title="paso === 1 ? t('asistente.atrasEnElPrimero') : undefined"
        @click="atras"
      >
        {{ t('asistente.atras') }}
      </ABoton>
      <ABoton
        v-if="paso < PASOS.length"
        variante="primario"
        :deshabilitado="!sePuedeSeguir"
        :ocupado="campanas.trabajando || campanas.revisando"
        :aria-describedby="porQueNoSeSigue ? 'asistente-falta' : undefined"
        @click="siguiente"
      >
        {{ t('asistente.siguiente') }}
      </ABoton>

      <!-- Por qué está apagado, pegado al botón y no al final de la pantalla:
           un aviso que explica un control tiene que verse con el control
           delante. Y con `aria-describedby`, porque si sólo se pinta, quien
           usa un lector de pantalla sigue sin saberlo (D-04). -->
      <p
        v-if="porQueNoSeSigue"
        id="asistente-falta"
        class="asistente__falta"
      >
        {{ t(porQueNoSeSigue) }}
      </p>
    </nav>
  </section>
</template>

<style scoped>
.asistente {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.asistente__falta {
  flex-basis: 100%;
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  text-align: right;
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
