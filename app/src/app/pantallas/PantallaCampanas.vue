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
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { resolverError } from '@/app/errores'
import { useCampanasStore } from '@/app/stores/campanas'
import {
  AAviso,
  ABoton,
  AInsignia,
  AModulo,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const campanas = useCampanasStore()
const router = useRouter()
const { t, te } = useI18n()

onMounted(() => {
  void campanas.cargar()
})

const error = computed(() =>
  campanas.errorGeneral ? resolverError({ clave: campanas.errorGeneral, detalle: '' }) : null,
)

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
      :titulo="t('campanas.titulo')"
      :nota="t('campanas.enMarcha', { n: campanas.enMarcha })"
    >
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
          </tr>
        </tbody>
      </table>
      <p class="campanas__nota">
        {{ t('campanas.aceptadoNoEsEntregado') }}
      </p>
    </AModulo>
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
