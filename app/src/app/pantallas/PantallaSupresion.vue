<script setup lang="ts">
/**
 * Quién pidió no recibir (entrega 3.4).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE DIRECCIÓN PIDIÓ, CON SUS PALABRAS
 *
 * «Tenemos que tener una lista de los prospectos que seleccionaron la
 * cancelación de correos o WhatsApp» — 21 de septiembre de 2026.
 *
 * Esta pantalla es esa lista. Sirve para dos cosas distintas, y las dos
 * importan:
 *
 *   - **Operar.** Saber a cuánta gente ya no se le puede escribir, y por qué.
 *   - **Probar.** Es el registro de que esas bajas se respetaron, con su fecha
 *     y su motivo, el día que alguien lo pregunte.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA NO DECIDE
 *
 * **Si una baja se puede deshacer no lo decide aquí.** Viene resuelto de Rust
 * en `sePuedeLevantar`. Esta pantalla esconde el botón cuando es `false`, pero
 * eso es cortesía: la defensa está en el núcleo, y si alguien llamara al
 * comando igualmente, el núcleo diría que no.
 *
 * Por qué importa: si la regla viviera aquí, habría dos copias —una en Rust y
 * otra en TypeScript— y el día que divergieran esta pantalla enseñaría un botón
 * que el núcleo rechaza, o escondería uno que sí valía.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import { POR_PAGINA, useSupresionStore } from '@/app/stores/supresion'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModulo,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const supresion = useSupresionStore()
const { t, te } = useI18n()

onMounted(() => {
  void supresion.cargar(0)
})

/** El motivo, ya traducido. Si falta el texto, se dice — no se inventa. */
function motivo(clave: string): string {
  return te(clave) ? t(clave) : t('supresion.motivo.desconocido')
}

/**
 * La fecha, en el formato que se lee de un vistazo.
 *
 * Se guarda en ISO 8601 con zona, que es lo correcto para comparar y ordenar,
 * y es ilegible en una tabla. Si la cadena no se puede interpretar se enseña
 * tal cual: mejor un texto raro que una fecha inventada.
 */
function fecha(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleDateString('es-MX', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

const errorGeneral = computed(() =>
  supresion.errorGeneral
    ? resolverError({ clave: supresion.errorGeneral, detalle: '' })
    : null,
)

/**
 * La cuenta de esta página, no de la lista entera.
 *
 * Y se dice así en pantalla: «50 de 1 284». Enseñar «50 de correo» a secas
 * cuando hay mil doscientas haría creer que ésas son todas (§94).
 */
const nota = computed(() =>
  t('supresion.nota', {
    cuantas: supresion.supresiones.length,
    total: supresion.total,
  }),
)

/** Sobre cuántas filas está buscando de verdad. Se dice para no engañar. */
const avisoDelFiltro = computed(() => {
  if (supresion.filtro.trim() === '') return ''
  return t('supresion.filtroParcial', {
    encontradas: supresion.visibles.length,
    de: supresion.supresiones.length,
  })
})

async function levantar(id: string, direccion: string): Promise<void> {
  const seguro = window.confirm(t('supresion.confirmarLevantar', { direccion }))
  if (!seguro) return
  await supresion.levantar(id)
}

function pagina(delta: number): void {
  const siguiente = Math.max(0, supresion.desde + delta * POR_PAGINA)
  void supresion.cargar(siguiente)
}
</script>

<template>
  <div class="supresion">
    <AModulo
      :titulo="t('supresion.titulo')"
      :nota="nota"
      principal
    >
      <p class="supresion__explicacion">
        {{ t('supresion.explicacion') }}
      </p>

      <AAviso
        v-if="errorGeneral"
        tono="peligro"
      >
        <strong>{{ errorGeneral.que }}</strong>
        <span>{{ errorGeneral.como }}</span>
        <span>{{ errorGeneral.salvo }}</span>
      </AAviso>

      <EstadoCargando
        v-else-if="supresion.cargando"
        :mensaje="t('supresion.cargando')"
      />

      <EstadoVacio
        v-else-if="supresion.total === 0"
        :titulo="t('supresion.vacioTitulo')"
        :cuerpo="t('supresion.vacioCuerpo')"
      />

      <template v-else>
        <!-- Cuántas por canal. Es lo primero que Dirección quiere ver. -->
        <div class="supresion__cifras">
          <AInsignia tono="neutro">
            {{ t('supresion.porCorreo', { n: supresion.cuantasDeCorreo }) }}
          </AInsignia>
          <AInsignia tono="neutro">
            {{ t('supresion.porWhatsApp', { n: supresion.cuantasDeWhatsApp }) }}
          </AInsignia>
        </div>

        <div class="supresion__buscar">
          <AEntrada
            v-model="supresion.filtro"
            :etiqueta="t('supresion.buscar')"
            :marcador="t('supresion.buscarMarcador')"
          />
          <p
            v-if="avisoDelFiltro"
            class="supresion__filtro-parcial"
          >
            {{ avisoDelFiltro }}
          </p>
        </div>

        <table class="tabla">
          <thead>
            <tr>
              <th scope="col">
                {{ t('supresion.columna.direccion') }}
              </th>
              <th scope="col">
                {{ t('supresion.columna.deQuien') }}
              </th>
              <th scope="col">
                {{ t('supresion.columna.motivo') }}
              </th>
              <th scope="col">
                {{ t('supresion.columna.fecha') }}
              </th>
              <th scope="col">
                <span class="sr-only">{{ t('supresion.columna.acciones') }}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="fila in supresion.visibles"
              :key="fila.id"
              class="fila"
            >
              <!-- La caja interior existe por una razón concreta: `display:
                   flex` puesto sobre el `<td>` saca la celda del algoritmo de
                   tabla, y entonces su línea inferior deja de alinear con la
                   del resto de la fila. Se veía en la vista previa: los
                   separadores de la primera columna caían a una altura
                   distinta de los de «De quién». La celda se queda celda; lo
                   que se coloca es su contenido. -->
              <td>
                <div class="fila__direccion">
                  <span class="fila__canal">
                    {{
                      fila.canal === 'email'
                        ? t('supresion.canal.correo')
                        : t('supresion.canal.whatsapp')
                    }}
                  </span>
                  <span class="fila__valor">{{ fila.direccion }}</span>
                  <AInsignia
                    v-if="fila.global"
                    tono="neutro"
                  >
                    {{ t('supresion.global') }}
                  </AInsignia>
                </div>
              </td>
              <td>{{ fila.deQuien || t('supresion.sinContacto') }}</td>
              <td>
                <div class="fila__motivo">
                  <span>{{ motivo(fila.motivo) }}</span>
                  <span
                    v-if="!fila.sePuedeLevantar"
                    class="fila__candado"
                  >
                    {{ t('supresion.loPidioLaPersona') }}
                  </span>
                </div>
              </td>
              <td class="fila__fecha">
                {{ fecha(fila.creada) }}
              </td>
              <td class="fila__acciones">
                <ABoton
                  v-if="fila.sePuedeLevantar"
                  variante="sutil"
                  tamano="compacto"
                  @click="levantar(fila.id, fila.direccion)"
                >
                  {{ t('supresion.volverAPermitir') }}
                </ABoton>
              </td>
            </tr>
          </tbody>
        </table>

        <div
          v-if="supresion.total > POR_PAGINA"
          class="supresion__paginas"
        >
          <ABoton
            variante="sutil"
            :deshabilitado="supresion.desde === 0"
            @click="pagina(-1)"
          >
            {{ t('supresion.anteriores') }}
          </ABoton>
          <ABoton
            variante="sutil"
            :deshabilitado="!supresion.hayMas"
            @click="pagina(1)"
          >
            {{ t('supresion.siguientes') }}
          </ABoton>
        </div>
      </template>
    </AModulo>
  </div>
</template>

<style scoped>
/* §17: ni un hex, ni un píxel suelto. Todo sale de los tokens.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * ESTA HOJA ENTERA ESTUVO MUERTA · encontrado el 22/09/2026
 *
 * Se escribió con catorce nombres de token **inventados** —`--espacio-3`,
 * `--color-texto-suave`, `--texto-sm`…— cuando los del proyecto son
 * `--arles-space-3`, `--arles-text-muted`, `--arles-font-size-small`.
 * Ninguno de los catorce existía. Ni uno.
 *
 * Consecuencia: la tabla salía **sin relleno, sin separadores y sin
 * jerarquía**; la etiqueta del canal se pegaba a la dirección
 * —«CORREOana@empresa.mx»—, la columna «Por qué» se desbordaba sobre la fila
 * siguiente, y el texto reservado a los lectores de pantalla se veía.
 *
 * POR QUÉ NO LO CAZÓ NADA, que es lo que importa:
 *
 *   - CSS con una variable inexistente **no es un error**. No avisa el
 *     compilador, ni el linter, ni el navegador: la declaración se descarta en
 *     silencio y la propiedad se queda en su valor inicial.
 *   - La comprobación de §17 busca literales —hex, píxeles sueltos—. Estos
 *     eran `var(…)` perfectamente formados, así que pasaba.
 *   - `sonda:supresion` mide **comportamiento**: que la lista cargue, que
 *     busque, que el botón de levantar aparezca sólo donde toca. Todo eso
 *     funcionaba. Una pantalla puede hacer lo correcto y ser ilegible.
 *
 * Desde hoy lo vigila el validador: «ningún estilo usa un token que no
 * existe». Se comprobó rompiéndolo.
 * ───────────────────────────────────────────────────────────────────────────
 */
.supresion {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.supresion__explicacion {
  margin: 0 0 var(--arles-space-4);
  max-width: 62ch;
  color: var(--arles-text-muted);
}

.supresion__cifras {
  display: flex;
  gap: var(--arles-space-3);
  margin-bottom: var(--arles-space-4);
}

.supresion__buscar {
  margin-bottom: var(--arles-space-4);
  max-width: 62ch;
}

.supresion__filtro-parcial {
  margin: var(--arles-space-2) 0 0;
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

.tabla {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--arles-font-size-small);
}

.tabla th {
  text-align: start;
  padding: var(--arles-space-2) var(--arles-space-3);
  border-bottom: var(--arles-border-width) solid var(--arles-border-strong);
  color: var(--arles-text-muted);
  font-weight: var(--arles-font-weight-semibold);
}

.fila td {
  padding: var(--arles-space-3);
  border-bottom: var(--arles-border-width) solid var(--arles-border);
  vertical-align: top;
}

.fila__direccion {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--arles-space-2);
}

.fila__canal {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.fila__valor {
  font-variant-numeric: tabular-nums;
}

.fila__motivo {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
}

/* Por qué no hay botón en esa fila. Un hueco sin explicación se lee como un
   fallo de la aplicación, no como una decisión. */
.fila__candado {
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

.fila__fecha {
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.fila__acciones {
  text-align: end;
}

.supresion__paginas {
  display: flex;
  justify-content: space-between;
  margin-top: var(--arles-space-4);
}

/* Visible para el lector de pantalla, invisible para la vista. Igual que
   `.solo-lectores` de App.vue, y con el mismo `1px` literal: no es una medida
   de diseño que deba salir de un token, es el mínimo que mantiene el nodo en
   el árbol de accesibilidad. Antes decía `var(--borde-1)`, un token que **no
   existe**, así que la declaración era inválida, el ancho caía en `auto` y el
   texto para lectores de pantalla **se veía**. */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}
</style>
