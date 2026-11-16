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
import { AAviso, ABoton, AModulo, EstadoError } from '@/design/componentes'

const empresa = useEmpresaStore()
const app = useAppStore()

const error = computed(() =>
  empresa.errorGeneral
    ? resolverError({ clave: empresa.errorGeneral, detalle: '' })
    : null,
)

onMounted(() => {
  // ── `refrescar`, no `cargar` (H-08, 16/10/2026) ─────────────────────────
  //
  // Esta pantalla es la lista de los seis pasos, y se vuelve a ella JUSTO
  // DESPUÉS de completar uno. `cargar` se salta a sí mismo en cuanto hay
  // datos, así que enseñaba la foto del arranque: Dirección dio de alta su
  // cuenta de correo y la lista seguía en «1 de 6», con el paso del remitente
  // sin marcar, mientras Remitentes enseñaba esa cuenta como lista para
  // enviar.
  //
  // Un contador que no sube cuando acabas de hacer el paso hace dudar de si
  // de verdad quedó hecho.
  void empresa.refrescar()
  void app.cargar()
})

const lista = computed(() => empresa.onboarding)
// El avance sale del catálogo, no de una plantilla escrita aquí.
//
// Esto decía `${completados} / ${total}` y daba «6 / 6», mientras el bloque de
// primera vez de esta MISMA pantalla usa `$t('inicio.avance')` y da «0 de 6».
// La misma cifra con dos formatos en el mismo archivo, y uno de los dos fuera
// del catálogo de textos. Lo encontró escribir la prueba de los seis pasos
// hechos, el 28/10/2026.

/**
 * El siguiente paso pendiente que tiene a dónde llevar.
 *
 * ── LO QUE DECÍA ESTE COMENTARIO, Y POR QUÉ ERA EL DEFECTO ─────────────────
 *
 * Decía: «los pasos 2 a 6 llegan en entregas futuras, así que apuntar al
 * siguiente sin más produciría un botón que lleva a una pantalla pendiente».
 * Eso dejó de ser verdad hace semanas y **el código siguió comportándose como
 * si lo fuera**, porque filtraba por un campo —`disponible`— que el núcleo
 * calculaba como «¿tiene pantalla propia?».
 *
 * La ventana de envío no tiene pantalla propia pero **sí se puede hacer**: es
 * el paso 5 del asistente. Así que nunca era «accionable», esto devolvía
 * `null` con los seis pasos hechos, y el módulo decía «No hay nada más que
 * puedas hacer aquí todavía» en la primera pantalla de un producto terminado.
 *
 * Hoy `disponible` no existe: hay una ruta o no la hay. Ver D-01 en
 * `10-requisitos/HALLAZGOS-DE-DISENO.md`.
 */
const siguienteAccionable = computed(
  () => lista.value.pasos.find((p) => !p.completado && p.ruta) ?? null,
)

/** Los seis hechos. Entonces el módulo felicita en vez de pedir otro paso. */
const terminada = computed(
  () => lista.value.completados === lista.value.total && lista.value.total > 0,
)

/**
 * Accesos rápidos: los «botones ágiles» que pidió Dirección.
 *
 * Cada uno declara si **ya lleva a algo**. Los que no, se dibujan deshabilitados
 * con su motivo en vez de desaparecer: quien busca dónde se cargan los
 * contactos tiene que encontrar la respuesta «aún no», no el vacío.
 */
const accesos = computed(() => [
  // Los tres primeros decían `listo: false` desde la 3.1, cuando ninguna de
  // esas pantallas existía. Siguieron diciéndolo con las tres construidas y
  // funcionando: Inicio anunciaba «Contactos · aún no» con la pantalla de
  // contactos a un clic, en la barra de al lado.
  //
  // Encontrado el 29/09/2026 mirando la demostración, no midiéndola: «aún no»
  // es un texto legítimo y ninguna sonda podía saber que era falso.
  { clave: 'contactos', ruta: '/contactos', listo: true },
  { clave: 'remitentes', ruta: '/remitentes', listo: true },
  { clave: 'campana', ruta: '/campanas', listo: true },
  { clave: 'ajustes', ruta: '/ajustes', listo: true },
])
</script>

<template>
  <section class="pantalla">
    <!-- ── Esta ventana no envía ────────────────────────────────────────────
         Va ARRIBA DEL TODO y fuera de las dos composiciones, no dentro de un
         módulo del panel. Es una condición de la ventana entera: mientras dure,
         ninguna campaña avanza desde aquí. Meterlo en un módulo lo dejaría
         compitiendo con el resto de la pantalla, y el usuario sólo lo miraría
         después de preguntarse por qué su campaña no sube.

         No es `urgente`: no responde a nada que el usuario acabe de hacer, y un
         aviso permanente que se anuncia solo convierte al lector de pantalla en
         ruido. -->
    <AAviso
      v-if="app.estaVentanaNoEnvia"
      tono="aviso"
      :titulo="$t('inicio.noEnvia.titulo')"
    >
      {{ $t('inicio.noEnvia.que') }}
      {{ $t('inicio.noEnvia.como') }}
      {{ $t('inicio.noEnvia.salvo') }}
    </AAviso>

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
            a="/inicio/configuracion"
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
          :titulo="terminada ? $t('inicio.altaTerminada') : $t('inicio.alta')"
          :nota="$t('inicio.avance', { hechos: lista.completados, total: lista.total })"
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
            {{ $t('inicio.todoHecho') }}
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
              to="/inicio/configuracion"
            >
              {{ $t('inicio.verPasos', { n: lista.total }, lista.total) }}
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
  /* No es texto corrido: es la salida de la tarjeta, y se pulsa. El alto
     mínimo es el del §22 y lo mide `sonda:auditoria` (auditoría D, §140). */
  display: inline-flex;
  align-items: center;
  min-height: var(--arles-objetivo-minimo);
  color: var(--arles-text);
  font-size: var(--arles-font-size-small);
}
</style>
