<script setup lang="ts">
/**
 * Actividad · lo que está pasando ahora mismo (entrega 7.3).
 *
 * ═════════════════════════════════════════════════════════════════════════
 * LO QUE DIRECCIÓN PIDIÓ, CON SUS PALABRAS · D-7A, 29/09/2026
 *
 * «No me gustaría que mostrara las respuestas, para mí ese sería trabajo del
 * asesor de ventas encargado de contestar los mensajes, solo me gustaría tener
 * la estadística de cuántas personas contestaron a la campaña; esto de
 * actividad me gustaría tenerlo todo perfectamente ordenado ya que es posible
 * tirar 3 campañas, entonces tendríamos que tener las estadísticas de las 3
 * campañas y no solamente generales.»
 *
 * De ahí salen las dos decisiones de forma de esta pantalla:
 *
 *   1. **Una tarjeta por campaña, y los totales aparte.** Con tres campañas a
 *      la vez, un total es un número que no se puede accionar: no dice cuál va
 *      bien. Los totales están arriba y **son la suma de las tarjetas**, no
 *      otra consulta — si no cuadraran, alguien dejaría de creerse las dos.
 *   2. **«Contestaron: 14» y nada más.** Ni un fragmento de lo que dijeron.
 *      Eso vive en el WhatsApp del asesor y no entra aquí ni en la base.
 *
 * ═════════════════════════════════════════════════════════════════════════
 * EL DATO QUE ESTA PANTALLA EXISTE PARA DAR
 *
 * **Por qué no sale nada.** Un cero sin explicación es lo que hace que alguien
 * reinicie ARLES, y reiniciar no arregla «fuera de ventana» ni «sin cupo». El
 * motor conocía las seis razones del silencio desde la Fase 4 —las tiene
 * escritas con su texto en español— y `una_vuelta()` las convertía en un
 * sí/no antes de tirarlas.
 *
 * Las seis están en `06-calidad/CATALOGO-DE-FALLOS.md` §1.
 *
 * ═════════════════════════════════════════════════════════════════════════
 * TRES COSAS QUE ESTA PANTALLA NO HACE, Y NO ES DESCUIDO
 *
 *   - **No para nada.** El freno vive en Campañas (D-7B): es donde está la
 *     lista, y duplicar Pausar/Detener en dos pantallas es como se acaba con
 *     dos sitios que dicen cosas distintas del mismo envío. De aquí se sale
 *     hacia allá con un enlace.
 *   - **No decide si una campaña está «atascada».** Enseña el último suceso del
 *     bucle y el ritmo, que es lo que hay. Un juicio calculado aquí sería una
 *     tercera versión de la verdad, después del motor y de la base.
 *   - **No dice «entregado»** (§65). Dice «aceptados»: es lo que ARLES sabe.
 */
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import { cifrasDeRazon } from '@/app/razones'
import {
  CADA_CUANTO_MS,
  useActividadStore,
  type CifrasDeCampana,
  type Cupo,
} from '@/app/stores/actividad'
import { useCampanasStore } from '@/app/stores/campanas'
import {
  AAviso,
  AInsignia,
  AModulo,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const actividad = useActividadStore()

/**
 * La parada de emergencia, leída del almacén de campañas.
 *
 * ── POR QUÉ ESTA PANTALLA TIENE QUE SABERLO ──
 *
 * Porque **es la razón número uno por la que no sale nada**, y sin ella esta
 * pantalla decía «2 de 5 campañas enviando» con el envío entero detenido: la
 * cifra correcta de un estado que ya no es cierto. El aviso vivía en Campañas,
 * a un clic de distancia, y aquí no llegaba.
 *
 * Se vio mirando la demostración con la parada puesta, con todo en verde.
 *
 * No se duplica el botón: parar y levantar viven en Campañas (D-7B). De aquí
 * se sale hacia allá por el enlace de abajo, que ya estaba.
 */
const campanas = useCampanasStore()
const { t, te } = useI18n()

/**
 * Los dos relojes, y son dos a propósito.
 *
 * El de refresco pide datos; el del latido sólo recalcula «hace n segundos».
 * Si fueran uno, el aviso de datos viejos no aparecería nunca: lo dispararía
 * justo lo que ha dejado de ocurrir.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * `let` Y NO `ref()`, Y ESO NO ES ESTILO
 *
 * La primera versión los guardaba en `ref<number | null>`. En el navegador
 * —donde ARLES corre— `setInterval` devuelve un número, un `ref` lo guarda tal
 * cual, y funciona. En Node devuelve un **objeto `Timeout`**, y un `ref`
 * envuelve los objetos en un Proxy reactivo: `clearInterval` recibía entonces
 * el Proxy y no el objeto que Node tiene registrado, así que **no cancelaba
 * nada**. Los dos relojes sobrevivían al desmontaje.
 *
 * Dos razones para arreglarlo y no declararlo «cosa de las pruebas»:
 *
 *   1. Un identificador de temporizador no es estado de la interfaz. Nada lo
 *      pinta, así que hacerlo reactivo sólo añade un Proxy por medio.
 *   2. La única prueba que puede comprobar que los relojes se paran corre en
 *      Node, y con el `ref` **no podía pasar nunca**. Una comprobación que no
 *      puede pasar no vigila nada (§2).
 *
 * Encontrado el 30/09/2026 al estrenar esa prueba.
 */
// `number` porque es lo que devuelve `window.setInterval` en un navegador, que
// es donde ARLES corre. `ReturnType<typeof setInterval>` resolvería al
// `Timeout` de Node, que aquí no existe.
let refresco: number | null = null
let latido: number | null = null

onMounted(() => {
  void actividad.cargar()
  void campanas.estadoDelEnvio()
  refresco = window.setInterval(
    () => void actividad.refrescar(),
    CADA_CUANTO_MS,
  )
  latido = window.setInterval(() => actividad.latir(), 1_000)
})

// Sin esto los dos relojes siguen corriendo tras salir de la pantalla: dos
// consultas por segundo a una base que nadie está mirando, y para siempre. En
// una aplicación de escritorio que se queda abierta toda la jornada, eso no se
// recupera solo.
onBeforeUnmount(() => {
  if (refresco !== null) window.clearInterval(refresco)
  if (latido !== null) window.clearInterval(latido)
})

/** El tono de la insignia de estado. Sin colorear lo que no lo necesita. */
function tonoDeEstado(
  estado: string,
): 'exito' | 'aviso' | 'peligro' | 'neutro' {
  if (estado === 'campana.estado.enviando') return 'exito'
  if (estado === 'campana.estado.pausada') return 'aviso'
  if (estado === 'campana.estado.detenida') return 'peligro'
  return 'neutro'
}

/**
 * El texto de una razón, ya con sus cifras.
 *
 * Si falta el texto de la clave no se enseña la clave cruda delante de nadie
 * (§139): se dice que la campaña no ha dado ninguna vuelta, que es lo único
 * honesto que se puede decir sin el texto.
 */
function textoDeRazon(c: CifrasDeCampana): string {
  const r = c.razon
  if (r === null) return t('actividad.sinRazon')
  return te(r.clave) ? t(r.clave, cifrasDeRazon(r)) : t('actividad.sinRazon')
}

/** La hora de un sello, en la zona del equipo. Sin el día: es «lo de ahora». */
function hora(sello: string): string {
  const d = new Date(sello)
  return Number.isNaN(d.getTime())
    ? ''
    : d.toLocaleTimeString('es-MX', { hour: '2-digit', minute: '2-digit' })
}

/** Cuántos salieron por hora en las últimas seis, redondeado a un decimal. */
function porHora(c: CifrasDeCampana): number {
  const total = c.ritmo.reduce((s, r) => s + r.salieron, 0)
  return Math.round((total / 6) * 10) / 10
}

/**
 * Cuántas horas quedan al ritmo de las últimas seis.
 *
 * `null` cuando el ritmo es cero: dividir por cero daría «infinitas horas», y
 * redondearlo daría un número inventado. Con ritmo cero lo que hay que enseñar
 * es la razón, no una estimación.
 */
function horasQueFaltan(c: CifrasDeCampana): number | null {
  const ph = porHora(c)
  if (ph <= 0 || c.pendiente === 0) return null
  return Math.max(1, Math.round(c.pendiente / ph))
}

/** `true` si esta campaña está enviando y aun así no sale nada. */
function estaQuieta(c: CifrasDeCampana): boolean {
  return (
    c.estado === 'campana.estado.enviando' && porHora(c) <= 0 && c.pendiente > 0
  )
}

/** El cupo del día de una cuenta, con su tope si lo tiene. */
function cupoDelDia(c: Cupo): string {
  return c.topeDiario === null
    ? t('actividad.cupoDiaSinTope', { gastado: c.gastadoHoy })
    : t('actividad.cupoDia', { gastado: c.gastadoHoy, tope: c.topeDiario })
}

function cupoDeLaHora(c: Cupo): string {
  return c.topePorHora === null
    ? t('actividad.cupoHoraSinTope', { gastado: c.gastadoEstaHora })
    : t('actividad.cupoHora', {
        gastado: c.gastadoEstaHora,
        tope: c.topePorHora,
      })
}

const frescura = computed(() =>
  actividad.antiguedadS <= 1
    ? t('actividad.actualizadoAhora')
    : t('actividad.actualizado', { segundos: actividad.antiguedadS }),
)

const error = computed(() =>
  actividad.errorGeneral === null
    ? null
    : resolverError({ clave: actividad.errorGeneral, detalle: '' }),
)
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ $t('actividad.titulo') }}
      </h1>
      <p class="intro">
        {{ $t('actividad.intro') }}
      </p>
      <!-- La aclaración de D-7A va aquí y **una sola vez**. Estuvo dentro de la
           celda de «Contestaron», en cada campaña: repetida cinco veces, y
           además rompía la rejilla —esa celda crecía a cuatro líneas y su cifra
           quedaba tres por debajo de las demás—. Se vio abriendo la pantalla,
           con las once pruebas en verde. -->
      <p class="intro">
        {{ $t('actividad.respondieronNota') }}
      </p>
    </header>

    <!-- La parada, antes que nada: mientras está puesta, todas las cifras de
         abajo son de algo que no está saliendo. -->
    <AAviso
      v-if="campanas.parada?.parado"
      tono="peligro"
      :titulo="$t('campana.parada.avisoQue')"
    >
      <p>{{ $t('actividad.paradaComo') }}</p>
      <p>{{ $t('campana.parada.avisoSalvo') }}</p>
      <template #acciones>
        <RouterLink :to="{ name: 'campanas' }">
          {{ $t('actividad.parar') }}
        </RouterLink>
      </template>
    </AAviso>

    <!-- El aviso de datos viejos va ARRIBA de las cifras, no debajo: debajo se
         lee después de haberse creído los números. -->
    <AAviso
      v-if="actividad.estaViejo"
      tono="aviso"
      :titulo="$t('actividad.viejo.que', { segundos: actividad.antiguedadS })"
    >
      <p>{{ $t('actividad.viejo.como') }}</p>
      <p>{{ $t('actividad.viejo.salvo') }}</p>
    </AAviso>

    <AAviso
      v-else-if="error"
      tono="peligro"
      :titulo="error.que"
    >
      <p>{{ error.como }}</p>
      <p>{{ error.salvo }}</p>
    </AAviso>

    <EstadoCargando v-if="actividad.cargando && !actividad.cargada" />

    <EstadoVacio
      v-else-if="!actividad.hayCampanas"
      :titulo="$t('actividad.sinCampanas.titulo')"
      :cuerpo="$t('actividad.sinCampanas.cuerpo')"
    >
      <!-- `#accion` en singular: es el nombre que `EstadoVacio` declara. Se
           escribió `#acciones` por analogía con `AModulo` y **no pintaba
           nada**: la pantalla vacía se quedaba sin su única salida, que es el
           defecto exacto que arrastró seis fases el diálogo de bajas. Lo cazó
           `ningun_slot_se_escribe_a_ciegas`, escrita en la 7.0 por eso. -->
      <template #accion>
        <RouterLink :to="{ name: 'campanaNueva' }">
          {{ $t('actividad.sinCampanas.accion') }}
        </RouterLink>
      </template>
    </EstadoVacio>

    <template v-else>
      <AModulo
        :titulo="$t('actividad.totales')"
        :nota="
          campanas.parada?.parado
            ? $t('actividad.enMarchaDetenido')
            : $t('actividad.enMarcha', {
              n: actividad.enMarcha,
              total: actividad.campanas.length,
            })
        "
      >
        <dl class="cifras cifras--grandes">
          <div>
            <dt>{{ $t('actividad.salio') }}</dt>
            <dd>{{ actividad.totalSalio.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.pendiente') }}</dt>
            <dd>{{ actividad.totalPendiente.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.fallo') }}</dt>
            <dd>{{ actividad.totalFallo.toLocaleString('es-MX') }}</dd>
          </div>
        </dl>
        <p class="frescura">
          {{ frescura }}
        </p>
      </AModulo>

      <!-- Una por campaña (D-7A). `ordenadas` pone las que están enviando
           arriba: lo que hace falta primero es lo que está pasando ahora. -->
      <AModulo
        v-for="c in actividad.ordenadas"
        :key="c.id"
        :titulo="c.nombre"
      >
        <!-- El estado ARRIBA, antes de las cifras. Estuvo en `#acciones`, que
             `AModulo` sirve en el PIE: «Enviando» aparecía debajo de todo, y es
             el dato que dice si hay que mirar la tarjeta o no. Los slots hacen
             lo que hacen, no lo que sugiere su nombre. -->
        <p class="estado">
          <AInsignia :tono="tonoDeEstado(c.estado)">
            {{ $t(c.estado) }}
          </AInsignia>
        </p>

        <dl class="cifras">
          <div>
            <dt>{{ $t('actividad.audiencia') }}</dt>
            <dd>{{ c.audiencia.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.salio') }}</dt>
            <dd>{{ c.salio.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.pendiente') }}</dt>
            <dd>{{ c.pendiente.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.fallo') }}</dt>
            <dd>{{ c.fallo.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.dadoDeBaja') }}</dt>
            <dd>{{ c.dadoDeBaja.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.cancelado') }}</dt>
            <dd>{{ c.cancelado.toLocaleString('es-MX') }}</dd>
          </div>
          <div>
            <dt>{{ $t('actividad.respondieron') }}</dt>
            <dd>{{ c.respondieron.toLocaleString('es-MX') }}</dd>
          </div>
        </dl>

        <p
          v-if="porHora(c) > 0"
          class="ritmo"
        >
          {{ $t('actividad.ritmo', { porHora: porHora(c) }) }}
          <template v-if="horasQueFaltan(c) !== null">
            ·
            {{
              $t('actividad.faltan', {
                n: c.pendiente,
                horas: horasQueFaltan(c),
              })
            }}
          </template>
        </p>
        <p
          v-else-if="c.estado === 'campana.estado.enviando'"
          class="ritmo"
        >
          {{ $t('actividad.ritmoCero') }}
        </p>

        <!-- La razón, sólo cuando explica algo: con la campaña avanzando, el
             último suceso del bucle es ruido que compite con las cifras. -->
        <div
          v-if="estaQuieta(c)"
          class="razon"
        >
          <h3>{{ $t('actividad.razon') }}</h3>
          <p>{{ textoDeRazon(c) }}</p>
          <p
            v-if="c.razon && hora(c.razon.cuando)"
            class="desde"
          >
            {{ $t('actividad.razonDesde', { hora: hora(c.razon.cuando) }) }}
          </p>
        </div>
      </AModulo>

      <AModulo :titulo="$t('actividad.remitentes')">
        <p
          v-if="actividad.remitentes.length === 0"
          class="intro"
        >
          {{ $t('actividad.sinRemitentes') }}
        </p>
        <ul
          v-else
          class="remitentes"
        >
          <li
            v-for="r in actividad.remitentes"
            :key="r.direccion"
          >
            <span class="remitente-nombre">{{ r.nombre }}</span>
            <span class="remitente-cupo">{{ cupoDelDia(r) }} · {{ cupoDeLaHora(r) }}</span>
            <!-- Lo que explica por qué dos campañas están quietas cuando una
                 tercera se comió el cupo. -->
            <span
              v-if="r.hayReparto"
              class="remitente-reparto"
            >
              {{ $t('actividad.reparto', { campanas: r.campanas.join(', ') }) }}
            </span>
          </li>
        </ul>
      </AModulo>

      <!-- Salidas. La revisión a ojo del 29/09/2026 encontró esta pantalla con
           CERO acciones: se salía sólo por la barra lateral, que es la
           definición de callejón. -->
      <nav
        class="salidas"
        :aria-label="$t('actividad.titulo')"
      >
        <RouterLink :to="{ name: 'campanas' }">
          {{ $t('actividad.parar') }}
        </RouterLink>
        <RouterLink :to="{ name: 'remitentes' }">
          {{ $t('actividad.verRemitentes') }}
        </RouterLink>
        <RouterLink :to="{ name: 'supresion' }">
          {{ $t('actividad.verBajas') }}
        </RouterLink>
      </nav>
    </template>
  </section>
</template>

<style scoped>
/* §17: ni un hex, ni un píxel suelto, ni una duración. Todo de los tokens.
 *
 * Los nombres se copian de `app/src/design/base.css` y no se escriben de
 * memoria: un `var(--espacio-3)` inventado **no es un error** para el
 * navegador —la declaración se descarta en silencio— y así se quedó una hoja
 * entera muerta en la 3.4. Lo vigila el validador. */
.pantalla {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.titulo {
  margin: 0 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.intro {
  margin: 0;
  max-width: 62ch;
  color: var(--arles-text-muted);
}

/* Las cifras, en rejilla que se adapta sin saltos: con tres campañas abiertas
 * el ancho de la ventana manda más que el número de columnas. */
.cifras {
  margin: 0;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
  gap: var(--arles-space-4);
}

.cifras div {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
}

.cifras dt {
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
}

.cifras dd {
  margin: 0;
  font-size: var(--arles-font-size-h3);
  font-weight: var(--arles-font-weight-bold);
  /* Las cifras alineadas por columna: con tabulares, 1 000 y 999 no bailan. */
  font-variant-numeric: tabular-nums;
}

.cifras--grandes dd {
  font-size: var(--arles-font-size-h2);
}

.estado {
  margin: 0 0 var(--arles-space-4);
}

.frescura {
  margin: var(--arles-space-4) 0 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}

.ritmo {
  margin: var(--arles-space-4) 0 0;
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}

/* La razón se separa con un filete y no con color de fondo: es información, no
 * una alarma. Una campaña fuera de horario a las diez de la noche está
 * haciendo exactamente lo que se le pidió. */
.razon {
  margin-top: var(--arles-space-4);
  padding-top: var(--arles-space-4);
  border-top: var(--arles-border-width) solid var(--arles-border);
}

.razon h3 {
  margin: 0 0 var(--arles-space-1);
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
  font-weight: var(--arles-font-weight-bold);
}

.razon p {
  margin: 0;
}

.desde {
  margin-top: var(--arles-space-1);
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}

.remitentes {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
}

.remitentes li {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
}

.remitente-nombre {
  font-weight: var(--arles-font-weight-bold);
}

.remitente-cupo,
.remitente-reparto {
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}

.salidas {
  display: flex;
  flex-wrap: wrap;
  gap: var(--arles-space-4);
}
</style>
