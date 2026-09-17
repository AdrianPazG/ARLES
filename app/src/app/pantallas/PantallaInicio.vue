<script setup lang="ts">
/**
 * Inicio · panel modular.
 *
 * ──────────────────────────────────────────────────────────────────────────
 * DOS PANTALLAS DISTINTAS, NO UNA CON VARIANTES
 *
 * Dirección pidió dos cosas que parecen una: que Inicio fuera modular con
 * acciones a mano, y que la primera vez guiara a configurar la empresa. Son
 * necesidades opuestas.
 *
 * Quien abre ARLES por primera vez no necesita un panel: no tiene nada que
 * ver en él. Necesita **una sola cosa que hacer**, grande y sin competencia.
 * Un panel de seis módulos vacíos en el primer arranque es la forma más rápida
 * de que alguien cierre la aplicación sin configurar nada.
 *
 * Quien ya la tiene configurada necesita lo contrario: el estado de un vistazo
 * y las acciones a un clic.
 *
 * Por eso aquí hay dos composiciones, no una con un `v-if` en medio. La
 * frontera es `empresa.configurada`, que el núcleo **deriva de los datos** y no
 * de una marca que alguien puso.
 * ──────────────────────────────────────────────────────────────────────────
 *
 * LO QUE NO SE DIBUJA
 *
 * El panel de la logística tendrá módulos de campañas en marcha, salud de los
 * canales y actividad reciente. Hoy nada de eso existe. **No se maquetan
 * vacíos**: un módulo que no puede decir nada cierto no se monta (ver
 * `AModulo`). Enseñar cajas en espera haría que la pantalla pareciera rota y,
 * peor, anunciaría capacidades que no están.
 *
 * Lo que sí se conserva del diseño anterior es la honestidad sobre lo que
 * falta: el avance dice «1 de 6» y los seis pasos siguen siendo consultables.
 * Lo que cambia es que su detalle se va a Ajustes y aquí queda la cifra y la
 * acción siguiente, que es lo que Dirección pidió.
 */
import { computed, onMounted } from 'vue'

import { resolverError } from '@/app/errores'
import { useAppStore } from '@/app/stores/app'
import { useEmpresaStore } from '@/app/stores/empresa'
import { ABoton, AModulo, EstadoError } from '@/design/componentes'

const empresa = useEmpresaStore()
const app = useAppStore()

const error = computed(() =>
  empresa.errorGeneral
    ? resolverError({ clave: empresa.errorGeneral, detalle: '' })
    : null,
)

onMounted(() => void empresa.cargar())

const lista = computed(() => empresa.onboarding)
const avance = computed(() => `${lista.value.completados} / ${lista.value.total}`)

/**
 * El siguiente paso que **se puede hacer hoy**, no el siguiente de la lista.
 *
 * La diferencia importa: los pasos 2 a 6 llegan en entregas futuras, así que
 * apuntar al «siguiente» sin más produciría un botón que lleva a una pantalla
 * pendiente. Cuando no hay ninguno accionable, el módulo lo dice en vez de
 * ofrecer un botón que no lleva a nada.
 */
const siguienteAccionable = computed(
  () => lista.value.pasos.find((p) => !p.completado && p.disponible) ?? null,
)

/** Los pasos que existirán pero todavía no, para poder decir cuántos son. */
const porConstruir = computed(
  () => lista.value.pasos.filter((p) => !p.disponible).length,
)

/**
 * Accesos rápidos: los «botones ágiles» que pidió Dirección.
 *
 * Cada uno declara si **ya lleva a algo**. Los que no, se dibujan deshabilitados
 * con su motivo en vez de desaparecer: quien busca dónde se cargan los
 * contactos tiene que encontrar la respuesta «aún no», no el vacío.
 */
const accesos = computed(() => [
  { clave: 'contactos', ruta: '/contactos', listo: false },
  { clave: 'remitentes', ruta: '/remitentes', listo: false },
  { clave: 'campana', ruta: '/campanas', listo: false },
  { clave: 'ajustes', ruta: '/ajustes', listo: true },
])
</script>

<template>
  <section class="pantalla">
    <EstadoError
      v-if="error"
      :que="error.que"
      :como="error.como"
      :salvo="error.salvo"
    />

    <!-- ── Primera vez: una sola cosa que hacer ─────────────────────────── -->
    <template v-else-if="!empresa.configurada">
      <header class="portada">
        <p class="marca">
          {{ app.info.nombreComercial }}
        </p>
        <h1 class="titulo-grande">
          {{ $t('inicio.primera.titulo') }}
        </h1>
        <p class="entradilla">
          {{ $t('inicio.primera.entradilla') }}
        </p>
      </header>

      <AModulo
        principal
        :titulo="$t('inicio.paso.empresa.titulo')"
        :nota="$t('inicio.avance', { hechos: lista.completados, total: lista.total })"
      >
        <p class="detalle">
          {{ $t('inicio.primera.porQue') }}
        </p>
        <template #acciones>
          <ABoton
            variante="primario"
            a="/ajustes"
          >
            {{ $t('inicio.primera.accion') }}
          </ABoton>
          <span class="detalle">{{ $t('inicio.primera.despues') }}</span>
        </template>
      </AModulo>
    </template>

    <!-- ── Ya configurada: panel modular ────────────────────────────────── -->
    <template v-else>
      <header>
        <h1 class="titulo">
          {{ $t('inicio.titulo', { producto: app.info.nombre }) }}
        </h1>
        <p class="entradilla">
          {{ $t('inicio.entradilla') }}
        </p>
      </header>

      <!-- Rejilla áurea: 1.618 a 1. Partirla por la mitad haría que el módulo
           principal y el secundario pesaran lo mismo, que es lo contrario de
           lo que un panel tiene que decir. -->
      <div class="rejilla">
        <AModulo
          principal
          :titulo="$t('inicio.alta')"
          :nota="avance"
        >
          <p
            v-if="siguienteAccionable"
            class="detalle"
          >
            {{ $t(`inicio.paso.${siguienteAccionable.clave}.detalle`) }}
          </p>
          <p
            v-else
            class="detalle"
          >
            {{ $t('inicio.sinPasosDisponibles') }}
          </p>

          <template #acciones>
            <ABoton
              v-if="siguienteAccionable?.ruta"
              variante="primario"
              :a="siguienteAccionable.ruta"
            >
              {{ $t(`inicio.paso.${siguienteAccionable.clave}.titulo`) }}
            </ABoton>
            <RouterLink
              class="enlace"
              to="/ajustes"
            >
              {{ $t('inicio.verPasos', { n: lista.total }) }}
            </RouterLink>
          </template>
        </AModulo>

        <AModulo :titulo="$t('inicio.accesos.titulo')">
          <ul class="accesos">
            <li
              v-for="a in accesos"
              :key="a.clave"
            >
              <RouterLink
                v-if="a.listo"
                class="enlace"
                :to="a.ruta"
              >
                {{ $t(`nav.${a.clave === 'campana' ? 'campanas' : a.clave}`) }}
              </RouterLink>
              <span
                v-else
                class="detalle"
              >
                {{ $t(`nav.${a.clave === 'campana' ? 'campanas' : a.clave}`) }}
                · {{ $t('inicio.accesos.aun') }}
              </span>
            </li>
          </ul>
        </AModulo>
      </div>

      <!-- Un módulo a todo el ancho, y no una nota al pie, porque es la
           respuesta a «¿por qué mi panel está tan vacío?». -->
      <AModulo
        v-if="porConstruir > 0"
        :titulo="$t('inicio.enConstruccion.titulo')"
        :nota="$t('inicio.enConstruccion.nota', { n: porConstruir })"
      >
        <p class="detalle">
          {{ $t('inicio.enConstruccion.cuerpo') }}
        </p>
      </AModulo>
    </template>
  </section>
</template>

<style scoped>
.pantalla {
  /* El ancho por debajo del cual el panel deja de funcionar: la rejilla áurea
     con la columna principal todavía legible (420 px) y la secundaria capaz de
     poner «Remitentes · aún no» en una línea (240 px), más el hueco.

     No es un `max-width`. Un panel modular quiere usar el ancho que haya —de
     eso va el espacio muerto lateral—, y lo que hay que proteger es el suelo,
     no el techo. Es lo que respalda el umbral de plegado automático. */
  --arles-medida: 684px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

/* La portada de la primera vez no lleva la medida de 78 ch de las pantallas de
   lectura: es un bloque corto y centrado en su columna, no un texto largo. */
.portada {
  max-width: 62ch;
}

.marca {
  margin: 0 0 var(--arles-space-2);
  color: var(--arles-accent-ink);
  font-size: var(--arles-font-size-small);
  font-weight: var(--arles-font-weight-semibold);
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.titulo-grande {
  margin: 0 0 var(--arles-space-3);
  font-size: var(--arles-font-size-display);
  line-height: var(--arles-line-height-display);
  font-weight: var(--arles-font-weight-bold);
}

.titulo {
  margin: 0 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.entradilla,
.detalle {
  margin: 0;
  color: var(--arles-text-muted);
}

.detalle {
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

.rejilla {
  display: grid;
  grid-template-columns: var(--arles-aureo-fr) 1fr;
  gap: var(--arles-space-5);
  align-items: start;
}

/* Por debajo del umbral de plegado medido (988 px) la rejilla áurea deja la
   columna estrecha por debajo de su medida legible, así que se apila. Es el
   mismo número que `--arles-medida` produce al medirlo con la barra abierta:
   684 + 240 de barra + 64 de márgenes. */
@media (width <= 988px) {
  .rejilla {
    grid-template-columns: 1fr;
  }
}

.accesos {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
}

.enlace {
  color: var(--arles-text);
  font-size: var(--arles-font-size-small);
}
</style>
