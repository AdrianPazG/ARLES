<script setup lang="ts">
/**
 * Las campañas (entrega 6.6).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA LISTA NO DICE, Y NO PUEDE DECIR
 *
 * La columna de envíos dice **«aceptados»**, nunca «entregados» (§65). ARLES
 * sabe que el servidor de correo se hizo cargo del mensaje; no sabe si llegó a
 * una bandeja, ni si acabó en la carpeta de no deseados.
 *
 * La diferencia parece pequeña hasta que alguien toma una decisión con ella
 * delante —«entregamos 1 240 y no contestó nadie»— y resulta que la mitad
 * nunca se vio.
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { resolverError } from '@/app/errores'
import { useCampanasStore } from '@/app/stores/campanas'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModal,
  AModulo,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const campanas = useCampanasStore()
const router = useRouter()
const { t, te } = useI18n()

onMounted(() => {
  void campanas.cargar()
  // El estado de la parada de emergencia. Se pide al entrar y no se supone:
  // sobrevive a cerrar ARLES —vive en la base—, así que al abrir el programa
  // puede estar puesta de ayer, y la pantalla tiene que decirlo antes de que
  // alguien se pregunte por qué no sale nada.
  void campanas.estadoDelEnvio()
})

const error = computed(() =>
  campanas.errorGeneral ? resolverError({ clave: campanas.errorGeneral, detalle: '' }) : null,
)

/**
 * La fecha de la parada, legible.
 *
 * Con día y hora, no sólo la hora: una parada puesta ayer por la tarde es una
 * jornada entera sin enviar, y «desde las 18:40» a secas se lee como de hoy.
 */
function fecha(sello: string): string {
  const d = new Date(sello)
  return Number.isNaN(d.getTime())
    ? ''
    // `dateStyle`/`timeStyle` y no los campos sueltos: con
    // `{ day: '2-digit', month: 'long' }` el navegador da el formato compacto
    // —«30-septiembre, 08:44 p.m.»—, con un guion en medio que no se escribe
    // así en español. Con `dateStyle: 'long'` sale «30 de septiembre de 2026».
    // Se vio mirando la pantalla; ninguna prueba compara formatos de fecha.
    : d.toLocaleString('es-MX', { dateStyle: 'long', timeStyle: 'short' })
}

/** El rótulo del estado. Si falta el texto se dice, no se inventa. */
function rotulo(clave: string): string {
  return te(clave) ? t(clave) : t('campana.estado.desconocido')
}

/** Qué aspecto tiene la insignia de cada estado. */
function tonoDe(clave: string): 'neutro' | 'exito' | 'aviso' | 'peligro' {
  if (clave === 'campana.estado.enviando') return 'exito'
  if (clave === 'campana.estado.enPausa') return 'aviso'
  if (clave === 'campana.estado.detenida') return 'peligro'
  return 'neutro'
}

function nueva(): void {
  void router.push({ name: 'campanaNueva' })
}

/* ── EL FRENO (7.0) ──────────────────────────────────────────────────────
 *
 * Estos botones llevaban desde la Fase 4 sin existir. El núcleo, la base y sus
 * pruebas estaban; los textos, escritos desde la Fase 6. Lo único que faltaba
 * era esto, y sin ello **una campaña activada no se podía parar**.
 *
 * Qué se puede hacer con cada estado lo decide el núcleo
 * (`AccionDeCampana::aplicar`). Aquí sólo se decide qué botón se ENSEÑA, y es
 * a propósito que las dos cosas no sean la misma: enseñar un botón de más
 * termina en un error del núcleo, que es feo pero seguro; aceptar una acción
 * que el núcleo rechaza no puede pasar, porque el núcleo la rechaza.
 */
function sePuedePausar(estado: string): boolean {
  return estado === 'campana.estado.enviando' || estado === 'campana.estado.programada'
}

function sePuedeReanudar(estado: string): boolean {
  return estado === 'campana.estado.enPausa'
}

function sePuedeDetener(estado: string): boolean {
  return sePuedePausar(estado) || sePuedeReanudar(estado)
}

/** La campaña que se está deteniendo, y lo que el usuario lleva escrito. */
const deteniendo = ref<{ id: string; nombre: string; cuantos: number } | null>(null)
const nombreEscrito = ref('')

function pedirDetener(c: { id: string; nombre: string; audiencia: number; aceptados: number }): void {
  deteniendo.value = {
    id: c.id,
    nombre: c.nombre,
    // Lo que se va a cancelar es lo que queda por salir, no la audiencia
    // entera: decir «cancela 1 342» cuando ya salieron 900 es una cifra falsa
    // en la única pantalla donde la cifra decide (§94).
    cuantos: Math.max(0, c.audiencia - c.aceptados),
  }
  nombreEscrito.value = ''
}

function cerrarDetener(): void {
  deteniendo.value = null
  nombreEscrito.value = ''
}

/** El botón de confirmar sólo se enciende con el nombre exacto. */
const puedeConfirmar = computed(
  () => deteniendo.value !== null && nombreEscrito.value.trim() === deteniendo.value.nombre.trim(),
)

async function confirmarDetener(): Promise<void> {
  if (!deteniendo.value || !puedeConfirmar.value) return
  const ok = await campanas.detener(deteniendo.value.id, nombreEscrito.value)
  if (ok) cerrarDetener()
}
</script>

<template>
  <section class="campanas">
    <header class="campanas__cabecera">
      <div>
        <h1 class="campanas__titulo">
          {{ t('campanas.titulo') }}
        </h1>
        <p class="campanas__entradilla">
          {{ t('campanas.entradilla') }}
        </p>
      </div>
      <ABoton
        variante="primario"
        icono="mas"
        @click="nueva"
      >
        {{ t('campanas.nueva') }}
      </ABoton>
    </header>

    <!-- ── LA PARADA DE EMERGENCIA · enchufada en la 7.3 ──────────────────
         Arriba de todo, y con el aviso antes de la tabla: mientras está puesta,
         cada cifra de abajo es de algo que **no está saliendo**. Verlo después
         de leer la tabla es leerla mal.

         Aquí y no en una sección aparte: D-7B puso el freno en Campañas, que es
         donde está la lista. Y separada de Detener, porque son dos cosas: detener
         cancela una campaña y no se deshace; parar todo es un interruptor. -->
    <AAviso
      v-if="campanas.parada?.parado"
      tono="peligro"
      :titulo="t('campana.parada.avisoQue')"
      urgente
    >
      <p>{{ t('campana.parada.avisoComo') }}</p>
      <p class="campanas__salvo">
        {{ t('campana.parada.avisoSalvo') }}
      </p>
      <p
        v-if="campanas.parada.desde"
        class="campanas__salvo"
      >
        {{ t('campana.parada.desdeCuando', { cuando: fecha(campanas.parada.desde) }) }}
      </p>
      <template #acciones>
        <ABoton
          variante="primario"
          :cargando="campanas.trabajando"
          @click="campanas.levantarParada()"
        >
          {{ t('campana.parada.levantar') }}
        </ABoton>
      </template>
    </AAviso>

    <AAviso
      v-if="error"
      tono="peligro"
      :titulo="error.que"
    >
      <p>{{ error.como }}</p>
      <p class="campanas__salvo">
        {{ error.salvo }}
      </p>
    </AAviso>

    <EstadoCargando
      v-if="campanas.cargando"
      :lineas="4"
      :explicacion="t('campanas.cargando')"
    />

    <EstadoVacio
      v-else-if="!campanas.hayCampanas"
      :titulo="t('campanas.vacio.titulo')"
      :cuerpo="t('campanas.vacio.cuerpo')"
    >
      <ABoton
        variante="primario"
        @click="nueva"
      >
        {{ t('campanas.nueva') }}
      </ABoton>
    </EstadoVacio>

    <AModulo
      v-else
      :titulo="t('campanas.listaTitulo')"
      :nota="
        campanas.parada?.parado
          ? t('campanas.enMarchaDetenido')
          : t('campanas.enMarcha', { n: campanas.enMarcha })
      "
    >
      <!-- No pide confirmación, a diferencia de Detener, y es deliberado: se
           pulsa cuando algo va mal y no se sabe todavía qué. Lo que se confirma
           es lo irreversible, y esto se levanta con el botón del aviso de
           arriba. Pedir que alguien escriba algo en una emergencia sólo añade
           segundos. -->
      <template
        v-if="!campanas.parada?.parado"
        #acciones
      >
        <ABoton
          variante="peligro"
          :cargando="campanas.trabajando"
          @click="campanas.pararTodo()"
        >
          {{ t('campana.parada.poner') }}
        </ABoton>
      </template>
      <table class="campanas__tabla">
        <thead>
          <tr>
            <th scope="col">
              {{ t('campanas.columna.nombre') }}
            </th>
            <th scope="col">
              {{ t('campanas.columna.estado') }}
            </th>
            <th
              scope="col"
              class="campanas__cifra"
            >
              {{ t('campanas.columna.audiencia') }}
            </th>
            <!-- §65: «aceptados», nunca «entregados». -->
            <th
              scope="col"
              class="campanas__cifra"
            >
              {{ t('campanas.columna.aceptados') }}
            </th>
            <th scope="col">
              <span class="sr-only">{{ t('campanas.columna.acciones') }}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="c in campanas.campanas"
            :key="c.id"
          >
            <td>{{ c.nombre }}</td>
            <td>
              <AInsignia :tono="tonoDe(c.estado)">
                {{ rotulo(c.estado) }}
              </AInsignia>
            </td>
            <td class="campanas__cifra">
              {{ c.audiencia.toLocaleString('es-MX') }}
            </td>
            <td class="campanas__cifra">
              {{ c.aceptados.toLocaleString('es-MX') }}
            </td>
            <td>
              <div class="campanas__acciones">
                <ABoton
                  v-if="sePuedePausar(c.estado)"
                  variante="secundario"
                  :cargando="campanas.trabajando"
                  @click="campanas.pausar(c.id)"
                >
                  {{ t('campana.accion.pausar') }}
                </ABoton>
                <ABoton
                  v-if="sePuedeReanudar(c.estado)"
                  variante="secundario"
                  :cargando="campanas.trabajando"
                  @click="campanas.reanudar(c.id)"
                >
                  {{ t('campana.accion.reanudar') }}
                </ABoton>
                <ABoton
                  v-if="sePuedeDetener(c.estado)"
                  variante="peligro"
                  @click="pedirDetener(c)"
                >
                  {{ t('campana.accion.detener') }}
                </ABoton>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      <p class="campanas__nota">
        {{ t('campanas.aceptadoNoEsEntregado') }}
      </p>
    </AModulo>

    <!--
      Detener no se deshace, así que la confirmación lleva la cifra de lo que
      se cancela y pide escribir el nombre. Con varias campañas a la vez, el
      riesgo no es detener sin querer: es detener la que no era.
    -->
    <AModal
      :abierto="deteniendo !== null"
      :titulo="t('campana.accion.detener')"
      destructiva
      @cerrar="cerrarDetener"
    >
      <!--
        La cifra va formateada como las de la tabla: «1,002», no «1002».
        Se vio mirando el diálogo — el resto de la pantalla usa
        `toLocaleString` y aquí se había quedado el número crudo, así que la
        cifra que más importa era la peor presentada.
      -->
      <p>
        {{
          t('campana.accion.confirmarDetener', {
            cuantos: (deteniendo?.cuantos ?? 0).toLocaleString('es-MX'),
          })
        }}
      </p>

      <AEntrada
        v-model="nombreEscrito"
        :etiqueta="t('campana.accion.escribeElNombre')"
        :marcador="t('campana.accion.nombreEsperado', { nombre: deteniendo?.nombre ?? '' })"
      />

      <template #acciones>
        <ABoton
          variante="secundario"
          @click="cerrarDetener"
        >
          {{ t('comun.cancelar') }}
        </ABoton>
        <ABoton
          variante="peligro"
          :deshabilitado="!puedeConfirmar"
          :cargando="campanas.trabajando"
          @click="confirmarDetener"
        >
          {{ t('campana.accion.detener') }}
        </ABoton>
      </template>
    </AModal>
  </section>
</template>

<style scoped>
.campanas {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.campanas__cabecera {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--arles-space-4);
}

.campanas__titulo {
  margin: 0;
  font-size: var(--arles-font-size-h1);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
}

.campanas__entradilla {
  margin: var(--arles-space-1) 0 0;
  color: var(--arles-text-muted);
}

.campanas__tabla {
  width: 100%;
  border-collapse: collapse;
}

.campanas__tabla th,
.campanas__tabla td {
  padding: var(--arles-space-3);
  text-align: left;
  border-bottom: var(--arles-border-width) solid var(--arles-border);
}

.campanas__tabla th {
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text-muted);
}

.campanas__acciones {
  display: flex;
  gap: var(--arles-space-2);
  justify-content: flex-end;
}

.campanas__cifra {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.campanas__nota,
.campanas__salvo {
  margin: var(--arles-space-3) 0 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}
</style>
