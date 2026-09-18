<script setup lang="ts">
/**
 * Importar contactos desde un CSV o un Excel (entrega 3.3).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * CUATRO PASOS, Y EL TERCERO ES EL QUE IMPORTA
 *
 *   1. Elegir el archivo
 *   2. Revisar qué es cada columna —ARLES lo propone, el usuario lo corrige—
 *   3. **Ver qué va a pasar**, y declarar de dónde salió la lista
 *   4. Confirmar
 *
 * El tercero existe porque Dirección decidió que ARLES **enseñe los choques
 * antes de importar**, en vez de saltarlos o fusionar solo. Una importación de
 * diez mil filas es irreversible en la práctica.
 *
 * Hasta que se pulsa «Importar», **la base no se toca**. Se puede volver atrás,
 * cambiar el mapeo y analizar otra vez las veces que haga falta.
 *
 * ── Lo que esta pantalla NO hace ──
 *
 * No valida direcciones ni normaliza nada: eso es del núcleo. Aquí sólo se
 * pinta lo que el análisis responde. Dos validaciones serían dos reglas que
 * mantener iguales.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { resolverError } from '@/app/errores'
import {
  CAMPOS,
  ORIGENES,
  TEXTO_PROVISIONAL,
  useImportacionStore,
  type OrigenDeLaLista,
} from '@/app/stores/importacion'
import {
  AAviso,
  ABoton,
  AInsignia,
  AModulo,
  ASelector,
  EstadoVacio,
} from '@/design/componentes'

const importacion = useImportacionStore()
const router = useRouter()
const { t, te } = useI18n()

// Si se sale de la pantalla a medias, el archivo se suelta en Rust. Sin esto,
// medio millón de filas se quedarían en memoria hasta cerrar ARLES.
onBeforeUnmount(() => {
  if (importacion.paso !== 'hecho') void importacion.cancelar()
})

const opcionesDeCampo = computed(() =>
  CAMPOS.map((valor) => ({ valor, texto: t(`importacion.campo.${valor}`) })),
)

const opcionesDeOrigen = computed(() =>
  ORIGENES.map((valor) => ({
    valor,
    texto: t(`importacion.origen.${aClave(valor)}`),
  })),
)

/** `formulario_propio` → `formularioPropio`, que es como está en los textos. */
function aClave(origen: OrigenDeLaLista): string {
  return origen.replace(/_(.)/g, (_, c: string) => c.toUpperCase())
}

function elegirCampo(indice: number, valor: string): void {
  const campo = CAMPOS.find((c) => c === valor)
  if (campo) importacion.mapeo[indice] = campo
}

/** El motivo de una fila rechazada, ya traducido. */
function motivo(clave: string): string {
  return te(clave) ? t(clave) : t('importacion.rechazo.otro')
}

const errorGeneral = computed(() =>
  importacion.errorGeneral
    ? resolverError({ clave: importacion.errorGeneral, detalle: '' })
    : null,
)

/** Una celda de la muestra, o vacío si la fila trae menos columnas. */
function celda(fila: readonly string[], i: number): string {
  return fila[i] ?? ''
}

async function terminar(): Promise<void> {
  importacion.reiniciar()
  await router.push('/contactos')
}
</script>

<template>
  <section class="importar">
    <header class="importar__cabecera">
      <h1>{{ t('importacion.titulo') }}</h1>
      <p class="importar__paso">
        {{ t(`importacion.paso.${importacion.paso}`) }}
      </p>
    </header>

    <AAviso
      v-if="errorGeneral"
      tono="peligro"
      :titulo="errorGeneral.que"
      urgente
    >
      <p>{{ errorGeneral.como }}</p>
      <p class="importar__salvo">
        {{ errorGeneral.salvo }}
      </p>
    </AAviso>

    <!-- ── 1 · Elegir el archivo ─────────────────────────────────────── -->
    <EstadoVacio
      v-if="importacion.paso === 'elegir'"
      :titulo="t('importacion.elegir.titulo')"
      :cuerpo="t('importacion.elegir.cuerpo')"
    >
      <!--
        `#accion` y no la ranura por defecto: `EstadoVacio` sólo pinta la
        ranura con ese nombre. Sin él el botón desaparece **en silencio**, sin
        error ni aviso — y la pantalla queda sin forma de avanzar.
      -->
      <template #accion>
        <ABoton
          variante="primario"
          icono="mas"
          :ocupado="importacion.leyendo"
          @click="importacion.elegirArchivo"
        >
          {{ t('importacion.elegir.accion') }}
        </ABoton>
      </template>
    </EstadoVacio>

    <!-- ── 2 · Qué es cada columna ───────────────────────────────────── -->
    <template v-else-if="importacion.paso === 'mapear' && importacion.archivo">
      <AModulo :titulo="importacion.archivo.nombre">
        <p class="importar__recuento">
          {{
            t('importacion.mapear.recuento', {
              filas: importacion.archivo.totalDeFilas,
              columnas: importacion.archivo.encabezados.length,
            })
          }}
        </p>
        <p class="importar__ayuda">
          {{ t('importacion.mapear.ayuda') }}
        </p>

        <!--
          Cada columna con su desplegable justo encima de sus datos. Poner la
          lista de columnas aparte de la muestra obligaría a ir y venir para
          saber qué contiene cada una.
        -->
        <div
          class="mapa"
          role="group"
          :aria-label="t('importacion.mapear.grupo')"
        >
          <div
            v-for="(encabezado, i) in importacion.archivo.encabezados"
            :key="i"
            class="mapa__columna"
            :class="{ 'mapa__columna--fuera': importacion.mapeo[i] === 'ignorar' }"
          >
            <ASelector
              :model-value="importacion.mapeo[i] ?? 'ignorar'"
              :etiqueta="encabezado || t('importacion.mapear.sinEncabezado')"
              :opciones="opcionesDeCampo"
              @update:model-value="(v: string) => elegirCampo(i, v)"
            />
            <ul class="mapa__muestra">
              <li
                v-for="(fila, f) in importacion.archivo.muestra.slice(0, 5)"
                :key="f"
              >
                {{ celda(fila, i) || '—' }}
              </li>
            </ul>
          </div>
        </div>
      </AModulo>

      <!--
        Sin correo ni WhatsApp no hay a quién escribir, y la importación
        saldría vacía entera. Se dice ANTES de analizar: dejarle ver un informe
        de cero filas sin explicación es peor.
      -->
      <AAviso
        v-if="!importacion.hayAlgunCanal"
        tono="aviso"
        :titulo="t('importacion.mapear.sinCanales')"
      >
        <p>{{ t('importacion.mapear.sinCanalesDetalle') }}</p>
      </AAviso>

      <div class="importar__pie">
        <ABoton
          variante="secundario"
          @click="importacion.cancelar"
        >
          {{ t('comun.cancelar') }}
        </ABoton>
        <ABoton
          variante="primario"
          :deshabilitado="!importacion.hayAlgunCanal"
          :ocupado="importacion.analizando"
          @click="importacion.analizar"
        >
          {{ t('importacion.mapear.accion') }}
        </ABoton>
      </div>
    </template>

    <!-- ── 3 · Qué va a pasar ────────────────────────────────────────── -->
    <template v-else-if="importacion.paso === 'revisar' && importacion.analisis">
      <AModulo :titulo="t('importacion.revisar.titulo')">
        <!-- Cifras, no adjetivos (§94). -->
        <p class="importar__cifras">
          {{
            t('importacion.revisar.cifras', {
              entran: importacion.cuantasEntran,
              fuera: importacion.cuantasSeQuedanFuera,
              total: importacion.archivo?.totalDeFilas ?? 0,
            })
          }}
        </p>

        <template v-if="importacion.analisis.choques.length > 0">
          <h3 class="importar__subtitulo">
            {{
              t('importacion.revisar.choques', {
                n: importacion.analisis.choques.length,
              })
            }}
          </h3>
          <p class="importar__ayuda">
            {{ t('importacion.revisar.choquesDetalle') }}
          </p>
          <ul class="filas">
            <li
              v-for="(c, i) in importacion.analisis.choques.slice(0, 50)"
              :key="i"
              class="filas__fila"
            >
              <!-- El número es el de EXCEL: vas a abrir tu archivo para corregirlo. -->
              <AInsignia tono="neutro">
                {{ t('importacion.fila', { n: c.fila }) }}
              </AInsignia>
              <span class="filas__dato">{{ c.direccion }}</span>
              <span class="filas__motivo">
                {{
                  c.dentroDelArchivo
                    ? t('importacion.revisar.repetidaEnElArchivo', { fila: c.con })
                    : t('importacion.revisar.yaEsDe', { quien: c.con || t('contactos.sinNombre') })
                }}
              </span>
            </li>
          </ul>
        </template>

        <template
          v-if="
            importacion.analisis.invalidas.length > 0 ||
              importacion.analisis.rechazadas.length > 0
          "
        >
          <h3 class="importar__subtitulo">
            {{
              t('importacion.revisar.fuera', {
                n:
                  importacion.analisis.invalidas.length +
                  importacion.analisis.rechazadas.length,
              })
            }}
          </h3>
          <ul class="filas">
            <li
              v-for="(r, i) in [
                ...importacion.analisis.invalidas,
                ...importacion.analisis.rechazadas,
              ].slice(0, 50)"
              :key="i"
              class="filas__fila"
            >
              <AInsignia tono="neutro">
                {{ t('importacion.fila', { n: r.fila }) }}
              </AInsignia>
              <span class="filas__dato">{{ r.referencia || '—' }}</span>
              <span class="filas__motivo">{{ motivo(r.motivo) }}</span>
            </li>
          </ul>
        </template>
      </AModulo>

      <!-- La declaración de origen (ADR-0013 §1). -->
      <AModulo :titulo="t('importacion.origen.titulo')">
        <p class="importar__ayuda">
          {{ t('importacion.origen.porQue') }}
        </p>

        <ASelector
          v-model="importacion.origen"
          :etiqueta="t('importacion.origen.campo')"
          :opciones="opcionesDeOrigen"
        />

        <!--
          El texto está PENDIENTE de la revisión jurídica (P-09), y se dice.
          Un aviso que cita una ley abrogada es peor que no citar ninguna:
          aparenta rigor (ADR-0013).
        -->
        <AAviso
          tono="aviso"
          :titulo="t('importacion.origen.provisional')"
        >
          <p>{{ t('importacion.origen.provisionalDetalle') }}</p>
        </AAviso>

        <label class="declaracion">
          <input
            v-model="importacion.acepta"
            type="checkbox"
          >
          <span>{{ TEXTO_PROVISIONAL }}</span>
        </label>
      </AModulo>

      <div class="importar__pie">
        <ABoton
          variante="secundario"
          @click="importacion.volverAMapear"
        >
          {{ t('importacion.revisar.volver') }}
        </ABoton>
        <ABoton
          variante="secundario"
          @click="importacion.cancelar"
        >
          {{ t('comun.cancelar') }}
        </ABoton>
        <ABoton
          variante="primario"
          :deshabilitado="!importacion.sePuedeConfirmar"
          :ocupado="importacion.importando"
          @click="importacion.confirmar"
        >
          {{
            t('importacion.revisar.accion', { n: importacion.cuantasEntran })
          }}
        </ABoton>
      </div>
    </template>

    <!-- ── 4 · Hecho ─────────────────────────────────────────────────── -->
    <template v-else-if="importacion.paso === 'hecho' && importacion.resumen">
      <AModulo :titulo="t('importacion.hecho.titulo')">
        <p class="importar__cifras">
          {{
            t('importacion.hecho.cifras', {
              importados: importacion.resumen.importados,
              total: importacion.resumen.totalDeFilas,
            })
          }}
        </p>
        <p
          v-if="importacion.resumen.choques > 0"
          class="importar__ayuda"
        >
          {{
            t('importacion.hecho.choques', { n: importacion.resumen.choques })
          }}
        </p>
        <p
          v-if="importacion.resumen.invalidas > 0"
          class="importar__ayuda"
        >
          {{
            t('importacion.hecho.fuera', { n: importacion.resumen.invalidas })
          }}
        </p>
      </AModulo>

      <div class="importar__pie">
        <ABoton
          variante="primario"
          @click="terminar"
        >
          {{ t('importacion.hecho.accion') }}
        </ABoton>
      </div>
    </template>
  </section>
</template>

<style scoped>
.importar {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  max-width: var(--arles-ancho-pagina);
}

.importar__cabecera h1 {
  margin: 0;
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
}

.importar__paso,
.importar__ayuda,
.importar__recuento,
.importar__salvo {
  margin: var(--arles-space-1) 0 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.importar__cifras {
  margin: 0 0 var(--arles-space-3);
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
}

.importar__subtitulo {
  margin: var(--arles-space-5) 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
}

/* Las columnas del archivo, en horizontal y con desplazamiento: una tabla de
   veinte columnas no cabe, y apilarlas verticalmente pierde la noción de que
   son columnas de lo mismo. */
.mapa {
  display: flex;
  gap: var(--arles-space-3);
  overflow-x: auto;
  padding-bottom: var(--arles-space-2);
}

.mapa__columna {
  flex: none;
  min-width: var(--arles-menu-min-width);
}

/* Una columna que no se importa se atenúa, pero NO desaparece: tiene que
   poder recuperarse, y para eso hay que verla. */
.mapa__columna--fuera {
  opacity: 0.55;
}

.mapa__muestra {
  margin: var(--arles-space-2) 0 0;
  padding: var(--arles-space-2);
  border-radius: var(--arles-radius-sm);
  background: var(--arles-surface);
  list-style: none;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.mapa__muestra li {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.filas {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  margin: 0;
  padding: 0;
  list-style: none;
}

.filas__fila {
  display: grid;
  grid-template-columns: auto 1fr 1.2fr;
  gap: var(--arles-space-3);
  align-items: center;
  padding: var(--arles-space-2) var(--arles-space-3);
  border-radius: var(--arles-radius-sm);
  background: var(--arles-surface);
}

.filas__dato {
  overflow-wrap: anywhere;
}

.filas__motivo {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.declaracion {
  display: flex;
  gap: var(--arles-space-3);
  align-items: flex-start;
  margin-top: var(--arles-space-4);
  padding: var(--arles-space-3);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  cursor: pointer;
}

.importar__pie {
  display: flex;
  justify-content: flex-end;
  gap: var(--arles-space-3);
}

@media (max-width: 40rem) {
  .filas__fila {
    grid-template-columns: 1fr;
  }
}
</style>
