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
              <td class="fila__direccion">
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
              </td>
              <td>{{ fila.deQuien || t('supresion.sinContacto') }}</td>
              <td class="fila__motivo">
                {{ motivo(fila.motivo) }}
                <span
                  v-if="!fila.sePuedeLevantar"
                  class="fila__candado"
                >
                  {{ t('supresion.loPidioLaPersona') }}
                </span>
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
/* §17: ni un hex, ni un píxel suelto. Todo sale de los tokens. */
.supresion {
  display: flex;
  flex-direction: column;
  gap: var(--espacio-6);
}

.supresion__explicacion {
  margin: 0 0 var(--espacio-4);
  max-width: var(--medida-texto);
  color: var(--color-texto-suave);
}

.supresion__cifras {
  display: flex;
  gap: var(--espacio-3);
  margin-bottom: var(--espacio-4);
}

.supresion__buscar {
  margin-bottom: var(--espacio-4);
  max-width: var(--medida-texto);
}

.supresion__filtro-parcial {
  margin: var(--espacio-2) 0 0;
  font-size: var(--texto-xs);
  color: var(--color-texto-suave);
}

.tabla {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--texto-sm);
}

.tabla th {
  text-align: start;
  padding: var(--espacio-2) var(--espacio-3);
  border-bottom: var(--borde-1) solid var(--color-borde);
  color: var(--color-texto-suave);
  font-weight: var(--peso-medio);
}

.fila td {
  padding: var(--espacio-3);
  border-bottom: var(--borde-1) solid var(--color-borde-sutil);
  vertical-align: top;
}

.fila__direccion {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--espacio-2);
}

.fila__canal {
  color: var(--color-texto-suave);
  font-size: var(--texto-xs);
  text-transform: uppercase;
  letter-spacing: var(--espaciado-ancho);
}

.fila__valor {
  font-variant-numeric: tabular-nums;
}

.fila__motivo {
  display: flex;
  flex-direction: column;
  gap: var(--espacio-1);
}

/* Por qué no hay botón en esa fila. Un hueco sin explicación se lee como un
   fallo de la aplicación, no como una decisión. */
.fila__candado {
  font-size: var(--texto-xs);
  color: var(--color-texto-suave);
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
  margin-top: var(--espacio-4);
}

.sr-only {
  position: absolute;
  width: var(--borde-1);
  height: var(--borde-1);
  padding: 0;
  margin: calc(var(--borde-1) * -1);
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}
</style>
