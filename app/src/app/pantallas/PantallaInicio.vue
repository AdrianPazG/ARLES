<script setup lang="ts">
/**
 * Inicio · la lista de alta (entrega 3.1).
 *
 * Qué falta para poder enviar una campaña, y qué de eso **todavía no está
 * construido**.
 *
 * Lo segundo es la decisión que sostiene esta pantalla. Lo cómodo sería
 * enseñar sólo los pasos que ya existen y que la lista fuera creciendo entrega
 * a entrega; entonces alguien la vería completa al terminar el primer paso y
 * concluiría que ya puede enviar. La lista enseña los seis desde el principio
 * y dice en qué entrega llega cada uno.
 *
 * El estado de cada paso lo **deriva el núcleo de los datos reales**, no de un
 * registro de lo que el usuario fue marcando: ver `arles_core::onboarding`.
 */
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'

import { resolverError } from '@/app/errores'
import { useAppStore } from '@/app/stores/app'
import { useEmpresaStore } from '@/app/stores/empresa'
import { AIcono, EstadoError } from '@/design/componentes'

const empresa = useEmpresaStore()
const app = useAppStore()

/**
 * Si los datos no se pudieron leer, **no se enseña la lista**.
 *
 * Sin esto, un fallo al leer dejaba la pantalla enseñando la lista de reserva
 * —«0 de 6», todo pendiente— como si fuera el estado real. Una lista de alta
 * que miente sobre lo que falta es peor que una pantalla que dice que no pudo
 * leerlo: la primera hace que alguien vuelva a configurar lo que ya tenía.
 */
const error = computed(() =>
  empresa.errorGeneral
    ? resolverError({ clave: empresa.errorGeneral, detalle: '' })
    : null,
)

onMounted(() => void empresa.cargar())

const lista = computed(() => empresa.onboarding)
const pendientesDisponibles = computed(() =>
  lista.value.pasos.filter((p) => !p.completado && p.disponible).length,
)
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ $t('inicio.titulo', { producto: app.info.nombre }) }}
      </h1>
      <p class="entradilla">
        {{ $t('inicio.entradilla') }}
      </p>
    </header>

    <EstadoError
      v-if="error"
      :que="error.que"
      :como="error.como"
      :salvo="error.salvo"
    />

    <section
      v-else
      aria-labelledby="titulo-alta"
    >
      <div class="cabecera-alta">
        <h2
          id="titulo-alta"
          class="subtitulo"
        >
          {{ $t('inicio.alta') }}
        </h2>
        <!-- La cifra, no un adjetivo: «2 de 6» es verificable, «casi listo»
             no (§94). -->
        <p class="avance">
          {{ $t('inicio.avance', { hechos: lista.completados, total: lista.total }) }}
        </p>
      </div>

      <ol class="lista">
        <li
          v-for="paso in lista.pasos"
          :key="paso.clave"
          class="paso"
          :class="{ hecho: paso.completado }"
        >
          <!-- Estado con icono Y texto, nunca sólo con color (regla 7.3):
               quien no distingue el verde ve exactamente lo mismo. -->
          <AIcono
            class="marca"
            :nombre="paso.completado ? 'exito' : 'cola'"
            :etiqueta="paso.completado ? $t('inicio.hecho') : $t('inicio.pendiente')"
          />

          <div class="cuerpo">
            <p class="nombre">
              <component
                :is="paso.disponible && paso.ruta ? RouterLink : 'span'"
                :to="paso.ruta ?? undefined"
              >
                {{ $t(`inicio.paso.${paso.clave}.titulo`) }}
              </component>
            </p>
            <p class="detalle">
              {{ $t(`inicio.paso.${paso.clave}.detalle`) }}
            </p>
          </div>

          <!-- Un paso que todavía no existe lo dice, con su entrega. Es la
               diferencia entre «no encuentro dónde hacerlo» y «aún no está».

               Texto apagado y no `AInsignia`: con insignia —relleno sólido—
               los cinco pasos que NO se pueden hacer pesaban más en la pantalla
               que el único que sí, que es justo al revés de lo que la lista
               tiene que decir. La insignia está para estados que hay que
               atender; esto es una nota al margen. -->
          <p
            v-if="!paso.disponible"
            class="cuando"
          >
            {{ $t('inicio.llegaEn', { entrega: paso.entrega }) }}
          </p>
        </li>
      </ol>

      <p
        v-if="pendientesDisponibles === 0 && !empresa.configurada"
        class="detalle"
      >
        {{ $t('inicio.sinPasosDisponibles') }}
      </p>
    </section>
  </section>
</template>

<style scoped>
.pantalla {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
  max-width: 78ch;
}

.titulo {
  margin: 0 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.subtitulo {
  margin: 0;
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
  font-weight: var(--arles-font-weight-semibold);
}

.entradilla,
.detalle {
  margin: 0;
  color: var(--arles-text-muted);
}

.cabecera-alta {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--arles-space-4);
  margin-bottom: var(--arles-space-4);
}

.avance {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  font-variant-numeric: tabular-nums;
}

.lista {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
}

.paso {
  display: flex;
  align-items: flex-start;
  gap: var(--arles-space-3);
  padding: var(--arles-space-3) var(--arles-space-4);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  background: var(--arles-surface);
}

.paso .marca {
  margin-top: var(--arles-space-1);
  color: var(--arles-text-disabled);
}

.paso.hecho .marca {
  color: var(--arles-success);
}

.cuerpo {
  flex: 1;
  min-width: 0;
}

.nombre {
  margin: 0 0 var(--arles-space-1);
  font-weight: var(--arles-font-weight-semibold);
}

.nombre a {
  color: var(--arles-text);
}

.detalle {
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

.cuando {
  margin: 0;
  flex: none;
  align-self: center;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  white-space: nowrap;
}
</style>
