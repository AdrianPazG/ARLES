<script setup lang="ts">
/**
 * El registro de actividad, con sus filtros · entrega 7.4 (D-7C).
 *
 * ═════════════════════════════════════════════════════════════════════════
 * LO QUE DIRECCIÓN PIDIÓ, CON SUS PALABRAS · 29/09/2026
 *
 * «Se recorta, pero no se borra, me gustaría que tuviera los registros
 * bimestrales o en caso que sean demasiados, más de 20 000 correos y mensajes
 * juntos, pues que solo sea posible visualizar los últimos 500 registros, así
 * que debería tener un filtro del registro, para poder buscar entre todos los
 * registros, todo perfectamente estructurado y ordenado.»
 *
 * ═════════════════════════════════════════════════════════════════════════
 * LAS TRES DECISIONES DE FORMA, Y DE DÓNDE SALEN
 *
 * 1. **El tope siempre lleva el total al lado**: «500 de 3 412». Quinientas
 *    filas sin esa segunda cifra se leen como «esto es todo», y quien busca
 *    algo de la semana pasada y no lo ve concluye que no pasó. Es la mitad del
 *    tope que hace que el tope no mienta.
 *
 * 2. **Los filtros viajan en la dirección.** `#/actividad/registro?texto=ana`
 *    se puede volver a abrir, guardar o pasar a alguien. Sin eso, «mira lo que
 *    pasó con Ana» obliga a explicar qué hay que teclear.
 *
 * 3. **Recortar no es borrar, y se dice antes.** El aviso da las tres cosas del
 *    §95 —qué se va, qué son y qué **no** se toca— con sus cifras. Sin la
 *    tercera nadie pulsa el botón, y con razón.
 *
 * ═════════════════════════════════════════════════════════════════════════
 * LO QUE ESTA PANTALLA NO HACE
 *
 * **No enseña lo que la gente contestó.** D-7A: eso es trabajo del asesor de
 * ventas en su WhatsApp. Aquí sólo consta que se escribió, a quién y con qué
 * desenlace.
 */
import { computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import { useCampanasStore } from '@/app/stores/campanas'
import { filtroVacio, useRegistroStore, type Filtro, type Suceso } from '@/app/stores/registro'
import {
  AAviso,
  ABoton,
  AEntrada,
  AModulo,
  ASelector,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const registro = useRegistroStore()
const campanas = useCampanasStore()
const route = useRoute()
const router = useRouter()
const { t, te } = useI18n()

onMounted(async () => {
  // El filtro se lee de la dirección **antes** de la primera consulta: entrar
  // por un enlace con filtros y ver primero la lista entera es un parpadeo que
  // hace dudar de si el enlace funcionó.
  registro.filtro = deLaRuta()
  await registro.cargar()
  // La lista de campañas, para el desplegable. Sin ella el filtro de campaña
  // tendría que enseñar identificadores.
  void campanas.cargar()
})

/** El filtro que dice la dirección. Lo que no esté, vacío. */
function deLaRuta(): Filtro {
  const q = route.query
  const leer = (k: string): string => (typeof q[k] === 'string' ? q[k] : '')
  return {
    campana: leer('campana'),
    clave: leer('clase'),
    desde: leer('desde'),
    hasta: leer('hasta'),
    texto: leer('texto'),
    soloDesenlaces: leer('tipo'),
  }
}

/**
 * Cambia un criterio: escribe la dirección y deja que el `watch` recargue.
 *
 * ── UN SOLO CAMINO, Y NO DOS ──
 *
 * La versión obvia —filtrar y además escribir la URL— deja dos caminos hacia el
 * mismo estado, y basta que uno se olvide de algo para que el botón de atrás
 * del navegador enseñe una lista que no corresponde a sus filtros. Aquí la
 * dirección es la única fuente: se escribe, y el `watch` hace el resto.
 */
async function cambiar(parte: Partial<Filtro>): Promise<void> {
  const f = { ...registro.filtro, ...parte }
  const query: Record<string, string> = {}
  if (f.campana) query['campana'] = f.campana
  if (f.clave) query['clase'] = f.clave
  if (f.desde) query['desde'] = f.desde
  if (f.hasta) query['hasta'] = f.hasta
  if (f.texto) query['texto'] = f.texto
  if (f.soloDesenlaces) query['tipo'] = f.soloDesenlaces
  await router.push({ name: 'registro', query })
}

watch(
  () => route.query,
  () => {
    registro.filtro = deLaRuta()
    void registro.cargar()
  },
)

async function limpiar(): Promise<void> {
  await router.push({ name: 'registro', query: {} })
  registro.filtro = filtroVacio()
}

/** El texto de un suceso, con sus cifras. Si falta, se dice cuál falta. */
function quePaso(s: Suceso): string {
  if (!te(s.clave)) return t('registro.desconocido', { clave: s.clave })
  // Las cifras viajan sin nombre, así que se ofrecen todas las que el catálogo
  // pueda pedir. `vue-i18n` ignora las que no use el texto.
  // ── `de` SALE DE `numeroB`, Y ESTUVO MAL ─────────────────────────────
  //
  // La primera versión pasaba `numeroA` a los dos, así que «Reintento 2 de 3»
  // salía «Reintento 2 de **2**»: un número plausible, coherente consigo mismo
  // y falso. No lo caza ninguna prueba de tipos —los dos son `number | null`—
  // ni ninguna de texto, porque la frase se forma igual. Se vio leyendo la
  // tabla en la demostración.
  return t(s.clave, {
    cual: s.numeroA ?? '',
    de: s.numeroB ?? '',
    cabian: s.numeroA ?? '',
    minutos: s.numeroA === null ? '' : Math.ceil(s.numeroA / 60),
  })
}

/**
 * El nombre corto de una clase, para el desplegable.
 *
 * `registro.clase.<ultimaParte>`: `reintento.seVolveraAIntentar` busca
 * `registro.clase.seVolveraAIntentar`. Si no está, se usa la frase larga — se
 * ve mal, pero se entiende, y la prueba lo caza antes de que llegue a nadie.
 */
function nombreCorto(clave: string): string {
  const corta = `registro.clase.${clave.split('.').pop() ?? ''}`
  if (te(corta)) return t(corta)
  return te(clave) ? t(clave) : clave
}

/** La fecha de un suceso, con día y hora: el registro se lee por días. */
function cuando(sello: string): string {
  const d = new Date(sello)
  return Number.isNaN(d.getTime())
    ? ''
    : d.toLocaleString('es-MX', { dateStyle: 'medium', timeStyle: 'short' })
}

/** El nombre de una campaña, o su identificador si ya no está en la lista. */
function nombreDeCampana(id: string | null): string {
  if (id === null) return ''
  return campanas.campanas.find((c) => c.id === id)?.nombre ?? id
}

const opcionesDeCampana = computed(() => [
  { valor: '', texto: t('registro.campanaTodas') },
  ...campanas.campanas.map((c) => ({ valor: c.id, texto: c.nombre })),
])

const opcionesDeClase = computed(() => [
  { valor: '', texto: t('registro.quePasoTodo') },
  // ── NOMBRE CORTO, NO LA FRASE ENTERA ────────────────────────────────
  //
  // El texto de un suceso es una frase con cifras —«Reintento {cual} de
  // {de}»—, y en un desplegable, sin cifras que poner, sale «Reintento de».
  // Un filtro pide categorías, no frases: «Reintentos», «Rebotes», «Bajas».
  // Si a una clave le falta su nombre corto se cae a la frase, que es feo pero
  // legible; lo vigila una prueba para que no se quede así.
  ...registro.clases.map((c) => ({
    valor: c.clave,
    texto: t('registro.claseConCifra', {
      que: nombreCorto(c.clave),
      cuantos: c.cuantos,
    }),
  })),
])

const opcionesDeTipo = computed(() => [
  { valor: '', texto: t('registro.tipoTodo') },
  { valor: 'si', texto: t('registro.tipoEnvios') },
  { valor: 'no', texto: t('registro.tipoMotor') },
])

const cuantos = computed(() =>
  registro.hayMasDeLosQueCaben
    ? t('registro.deTantos', {
        cuantos: registro.sucesos.length,
        total: registro.total,
      })
    : t('registro.todos', { total: registro.total }),
)

const error = computed(() =>
  registro.errorGeneral === null
    ? null
    : resolverError({ clave: registro.errorGeneral, detalle: '' }),
)
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ t('registro.titulo') }}
      </h1>
      <p class="intro">
        {{ t('registro.intro') }}
      </p>
      <!-- §65 a la vista, no en un pie de página que nadie lee. -->
      <p class="intro">
        {{ t('registro.aceptadoAclaracion') }}
      </p>
    </header>

    <AAviso
      v-if="error"
      tono="peligro"
      :titulo="error.que"
    >
      <p>{{ error.como }}</p>
      <p>{{ error.salvo }}</p>
    </AAviso>

    <!-- Sin `nota` aquí: la cifra vive en el módulo de la tabla, que es lo que
         cuenta. Estuvo en los dos y se leía «9 registros» dos veces en la misma
         pantalla, a diez centímetros una de otra. -->
    <AModulo :titulo="t('registro.filtros')">
      <div class="filtros">
        <ASelector
          :etiqueta="t('registro.campana')"
          :model-value="registro.filtro.campana"
          :opciones="opcionesDeCampana"
          @update:model-value="cambiar({ campana: $event })"
        />
        <ASelector
          :etiqueta="t('registro.quePaso')"
          :model-value="registro.filtro.clave"
          :opciones="opcionesDeClase"
          @update:model-value="cambiar({ clave: $event })"
        />
        <ASelector
          :etiqueta="t('registro.tipo')"
          :model-value="registro.filtro.soloDesenlaces"
          :opciones="opcionesDeTipo"
          @update:model-value="cambiar({ soloDesenlaces: $event })"
        />
        <!-- ── EL FORMATO DE LA FECHA LO PONE EL SISTEMA, NO ARLES ───────
             Un `input[type=date]` se dibuja con el idioma **del navegador**, no
             con el `lang` del documento: en esta demostración, abierta en un
             Chromium sin idioma, sale «mm/dd/yyyy»; en Windows en español de
             México sale «dd/mm/aaaa». No es algo que se pueda forzar desde
             aquí, y hacerlo a mano con tres cuadros de texto sería peor: habría
             que validar lo que alguien teclee y adivinar el orden.
             R-07 y la revisión en Windows lo tienen que confirmar. -->
        <AEntrada
          :etiqueta="t('registro.desde')"
          tipo="date"
          :model-value="registro.filtro.desde"
          @update:model-value="cambiar({ desde: $event })"
        />
        <AEntrada
          :etiqueta="t('registro.hasta')"
          tipo="date"
          :model-value="registro.filtro.hasta"
          @update:model-value="cambiar({ hasta: $event })"
        />
        <AEntrada
          :etiqueta="t('registro.texto')"
          :ayuda="t('registro.textoAyuda')"
          :model-value="registro.filtro.texto"
          @update:model-value="cambiar({ texto: $event })"
        />
      </div>

      <template #acciones>
        <ABoton
          v-if="registro.hayFiltro"
          variante="secundario"
          @click="limpiar"
        >
          {{ t('registro.limpiar') }}
        </ABoton>
      </template>
    </AModulo>

    <EstadoCargando
      v-if="registro.cargando && !registro.cargada"
      :lineas="6"
    />

    <EstadoVacio
      v-else-if="registro.sucesos.length === 0 && registro.hayFiltro"
      :titulo="t('registro.sinResultados.titulo')"
      :cuerpo="t('registro.sinResultados.cuerpo')"
    >
      <template #accion>
        <ABoton
          variante="secundario"
          @click="limpiar"
        >
          {{ t('registro.limpiar') }}
        </ABoton>
      </template>
    </EstadoVacio>

    <EstadoVacio
      v-else-if="registro.sucesos.length === 0"
      :titulo="t('registro.vacio.titulo')"
      :cuerpo="t('registro.vacio.cuerpo')"
    >
      <template #accion>
        <RouterLink :to="{ name: 'actividad' }">
          {{ t('registro.volver') }}
        </RouterLink>
      </template>
    </EstadoVacio>

    <AModulo
      v-else
      :titulo="t('registro.titulo')"
      :nota="cuantos"
    >
      <p
        v-if="registro.hayMasDeLosQueCaben"
        class="aviso-tope"
      >
        {{ t('registro.usaElFiltro') }}
      </p>

      <table class="tabla">
        <thead>
          <tr>
            <th scope="col">
              {{ t('registro.columna.cuando') }}
            </th>
            <th scope="col">
              {{ t('registro.columna.quePaso') }}
            </th>
            <th scope="col">
              {{ t('registro.columna.quien') }}
            </th>
            <th scope="col">
              {{ t('registro.columna.campana') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="s in registro.sucesos"
            :key="s.id"
          >
            <td class="cuando">
              {{ cuando(s.cuando) }}
            </td>
            <td>{{ quePaso(s) }}</td>
            <!-- Un suceso del motor no es de nadie: no se inventa un
                 destinatario, se dice que no lo tiene. -->
            <td class="quien">
              <span v-if="s.direccion">{{ s.direccion }}</span>
              <span
                v-else
                class="sin-dueno"
              >{{ t('registro.sinDestinatario') }}</span>
            </td>
            <td>{{ nombreDeCampana(s.campana) }}</td>
          </tr>
        </tbody>
      </table>
    </AModulo>

    <!-- ── EL RECORTE BIMESTRAL (D-7C) ───────────────────────────────────
         Abajo y no arriba: es mantenimiento, no lo que se viene a hacer. -->
    <AModulo :titulo="t('registro.recorte.titulo')">
      <template v-if="registro.recorte === null">
        <ABoton
          variante="secundario"
          @click="registro.verElRecorte()"
        >
          {{ t('registro.recorte.revisar') }}
        </ABoton>
      </template>

      <p
        v-else-if="registro.recorte.seIrian === 0"
        class="intro"
      >
        {{ t('registro.recorte.nada') }}
      </p>

      <AAviso
        v-else
        tono="aviso"
        :titulo="t('registro.recorte.que', { seIrian: registro.recorte.seIrian })"
      >
        <p>{{ t('registro.recorte.como') }}</p>
        <!-- La tercera cosa del §95, y la que hace que alguien se atreva a
             pulsar: lo que **no** se toca. -->
        <p>{{ t('registro.recorte.salvo', { seQuedan: registro.recorte.seQuedan }) }}</p>
        <template #acciones>
          <ABoton
            variante="peligro"
            :cargando="registro.trabajando"
            @click="registro.recortar()"
          >
            {{ t('registro.recorte.accion', { seIrian: registro.recorte.seIrian }) }}
          </ABoton>
        </template>
      </AAviso>
    </AModulo>

    <nav
      class="salidas"
      :aria-label="t('registro.titulo')"
    >
      <RouterLink :to="{ name: 'actividad' }">
        {{ t('registro.volver') }}
      </RouterLink>
    </nav>
  </section>
</template>

<style scoped>
/* §17: todo de los tokens. Los nombres se copian de `base.css` y no se
 * escriben de memoria: un `var(--espacio-3)` inventado no es un error para el
 * navegador, y así se quedó una hoja entera muerta en la 3.4. */
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
  margin: 0 0 var(--arles-space-2);
  max-width: 62ch;
  color: var(--arles-text-muted);
}

/* Los seis filtros en rejilla: a ancho de portátil caben en dos filas, y al
 * estrecharse pasan a una columna sin que ninguno se parta. */
.filtros {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: var(--arles-space-4);
}

.aviso-tope {
  margin: 0 0 var(--arles-space-4);
  color: var(--arles-text-muted);
}

.tabla {
  width: 100%;
  border-collapse: collapse;
}

.tabla th {
  text-align: left;
  padding-bottom: var(--arles-space-2);
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
  font-weight: var(--arles-font-weight-bold);
  border-bottom: var(--arles-border-width) solid var(--arles-border);
}

.tabla td {
  padding: var(--arles-space-2) var(--arles-space-3) var(--arles-space-2) 0;
  border-bottom: var(--arles-border-width) solid var(--arles-border);
  vertical-align: top;
}

/* La fecha con cifras tabulares y sin partirse: una columna de fechas que
 * baila es ilegible de un vistazo, y el registro se lee de un vistazo. */
.cuando {
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
  color: var(--arles-text-muted);
}

.quien {
  word-break: break-all;
}

.sin-dueno {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.salidas {
  display: flex;
  flex-wrap: wrap;
  gap: var(--arles-space-4);
}

/* Las salidas se pulsan, así que tienen que medir lo que el §22 exige. Sin
   esto medían 20 px —el alto de su línea—, y lo encontró la auditoría D del
   §140 el 07/10/2026 midiendo, porque a ojo un enlace corto no se ve
   pequeño. */
.salidas a {
  display: inline-flex;
  align-items: center;
  min-height: var(--arles-objetivo-minimo);
}
</style>
