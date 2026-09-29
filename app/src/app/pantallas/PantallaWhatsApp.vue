<script setup lang="ts">
/**
 * El número de WhatsApp desde el que ARLES escribe (entrega W.6).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LAS DOS CONFIRMACIONES NO ESTÁN EN EL FORMULARIO, Y ÉSE ES EL DISEÑO
 *
 * Coexistencia y la aceptación del riesgo se piden **después** de guardar, cada
 * una en su tarjeta y con su propio botón. Puestas como dos casillas del alta
 * se marcarían con el resto de los campos sin leerlas — que es exactamente lo
 * que estas dos marcas existen para impedir.
 *
 * Son la constancia de que a alguien se le avisó: una, que el número sale de la
 * aplicación de WhatsApp Business (L-11); otra, que escribir a quien no lo pidió
 * puede costar el número entero. Que cuesten un gesto más es deliberado.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA NO DECIDE
 *
 * **No decide si un número puede enviar.** `puedeEnviar` llega resuelto de Rust.
 * Recalcularlo aquí serían dos reglas que se separan en cuanto una cambie, y la
 * que decide si se envía no sería la que se ve.
 *
 * **No valida el teléfono.** Lo hace el núcleo, con el país de la empresa. Aquí
 * solo se pinta lo que devolvió.
 *
 * **No guarda el token.** Vive en el campo mientras se escribe, viaja en la
 * llamada y se borra al cerrar el formulario. No pasa por la tienda ni por
 * `localStorage` (ADR-0011).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LA CALIDAD SIN CONSULTAR NO SE PINTA EN VERDE
 *
 * `calidad === null` significa «no se sabe», no «está bien». Se enseña con tono
 * neutro y su explicación. Un semáforo que se pone en verde cuando no ha podido
 * mirar es peor que no tener semáforo — la misma regla del «sin comprobar» del
 * preflight de correo.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import {
  borradorDeNumeroVacio,
  useWhatsappStore,
  type BorradorDeNumero,
  type Numero,
} from '@/app/stores/whatsapp'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModulo,
  ASelector,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const store = useWhatsappStore()
const { t } = useI18n()

onMounted(() => {
  void store.cargar()
})

/**
 * El error, ya partido en las tres cosas que el §95 exige que diga: qué pasó,
 * cómo se arregla y qué está a salvo. `resolverError` recibe el objeto de la
 * frontera, no la clave suelta.
 */
const problema = computed(() =>
  store.error ? resolverError({ clave: store.error, detalle: '' }) : null,
)

const formularioAbierto = ref(false)
const borrador = ref<BorradorDeNumero>(borradorDeNumeroVacio())

/**
 * El token, fuera del borrador a propósito.
 *
 * El borrador se puede pasar, copiar y —el día que a alguien le convenga—
 * guardar como último formulario usado. El token no debe poder colarse en nada
 * de eso, así que vive en su propia referencia y se borra a mano al cerrar.
 */
const token = ref('')

const MODOS = computed(() => [
  { valor: 'seguimiento', texto: t('whatsapp.modo.seguimiento') },
  { valor: 'prospeccion_directa', texto: t('whatsapp.modo.prospeccionDirecta') },
])

/** Avisa antes de guardar, no después: el modo en frío se elige sabiéndolo. */
const elegidoEnFrio = computed(() => borrador.value.modo === 'prospeccion_directa')

const puedeGuardar = computed(
  () =>
    borrador.value.nombreVisible.trim().length > 0 &&
    borrador.value.telefono.trim().length > 0 &&
    borrador.value.wabaId.trim().length > 0 &&
    borrador.value.phoneNumberId.trim().length > 0 &&
    token.value.trim().length > 0,
)

function abrir(): void {
  borrador.value = borradorDeNumeroVacio()
  token.value = ''
  formularioAbierto.value = true
}

function cerrar(): void {
  formularioAbierto.value = false
  borrador.value = borradorDeNumeroVacio()
  // El token se borra aquí y no solo al abrir: si alguien cancela y no vuelve
  // a entrar, no tiene por qué seguir en memoria.
  token.value = ''
}

async function guardar(): Promise<void> {
  if (await store.guardar(borrador.value, token.value)) cerrar()
}

/** La calidad, o el texto de «no consultada» — nunca un hueco. */
function textoDeCalidad(n: Numero): string {
  return n.calidad === null
    ? t('numeroDeWhatsapp.calidad.sinConsultar')
    : t(`numeroDeWhatsapp.calidad.${n.calidad}`)
}

function tonoDeCalidad(n: Numero): 'exito' | 'aviso' | 'peligro' | 'neutro' {
  switch (n.calidad) {
    case 'green':
      return 'exito'
    case 'yellow':
      return 'aviso'
    case 'red':
      return 'peligro'
    // `unknown` de Meta y `null` de «no preguntamos» caen los dos aquí: ninguno
    // de los dos es una calidad buena, y ninguno se pinta como si lo fuera.
    default:
      return 'neutro'
  }
}
</script>

<template>
  <section class="pantalla">
    <header class="cabecera">
      <div>
        <h1>{{ t('numeroDeWhatsapp.titulo') }}</h1>
        <p class="explicacion">
          {{ t('numeroDeWhatsapp.explicacion') }}
        </p>
      </div>
      <ABoton
        v-if="!formularioAbierto"
        variante="primario"
        @click="abrir"
      >
        {{ t('numeroDeWhatsapp.anadir') }}
      </ABoton>
    </header>

    <AAviso
      v-if="problema"
      tono="peligro"
    >
      <strong>{{ problema.que }}</strong>
      <p>{{ problema.como }}</p>
      <p>{{ problema.salvo }}</p>
    </AAviso>

    <EstadoCargando v-if="store.cargando && store.numeros.length === 0" />

    <!-- ── El formulario de alta ── -->
    <AModulo
      v-if="formularioAbierto"
      :titulo="t('numeroDeWhatsapp.anadir')"
    >
      <div class="campos">
        <AEntrada
          v-model="borrador.nombreVisible"
          :etiqueta="t('numeroDeWhatsapp.nombreVisible')"
          :ayuda="t('numeroDeWhatsapp.nombreVisibleAyuda')"
        />
        <AEntrada
          v-model="borrador.telefono"
          :etiqueta="t('numeroDeWhatsapp.telefono')"
          :ayuda="t('numeroDeWhatsapp.telefonoAyuda')"
        />
        <AEntrada
          v-model="borrador.wabaId"
          :etiqueta="t('numeroDeWhatsapp.wabaId')"
          :ayuda="t('numeroDeWhatsapp.wabaIdAyuda')"
        />
        <AEntrada
          v-model="borrador.phoneNumberId"
          :etiqueta="t('numeroDeWhatsapp.phoneNumberId')"
          :ayuda="t('numeroDeWhatsapp.phoneNumberIdAyuda')"
        />
        <AEntrada
          v-model="token"
          tipo="password"
          :etiqueta="t('numeroDeWhatsapp.token')"
          :ayuda="t('numeroDeWhatsapp.tokenAyuda')"
        />
        <ASelector
          v-model="borrador.modo"
          :etiqueta="t('numeroDeWhatsapp.modo')"
          :opciones="MODOS"
          :ayuda="t('numeroDeWhatsapp.modoAyuda')"
        />
      </div>

      <!-- Se avisa ANTES de guardar. Un aviso que llega después del alta no
           cambia ninguna decisión: solo informa de una que ya se tomó. -->
      <AAviso
        v-if="elegidoEnFrio"
        tono="aviso"
      >
        {{ t('numeroDeWhatsapp.riesgo.cuerpo') }}
      </AAviso>

      <div class="acciones">
        <ABoton
          variante="primario"
          :deshabilitado="!puedeGuardar"
          @click="guardar"
        >
          {{ t('numeroDeWhatsapp.guardar') }}
        </ABoton>
        <ABoton
          variante="secundario"
          @click="cerrar"
        >
          {{ t('numeroDeWhatsapp.cancelar') }}
        </ABoton>
      </div>
    </AModulo>

    <EstadoVacio
      v-else-if="store.numeros.length === 0 && !store.cargando"
      :titulo="t('numeroDeWhatsapp.vacioTitulo')"
      :cuerpo="t('numeroDeWhatsapp.vacioCuerpo')"
    />

    <!-- ── Los números dados de alta ── -->
    <p
      v-if="store.numeros.length > 0"
      class="nota"
    >
      {{ t('numeroDeWhatsapp.nota', { listos: store.listos, total: store.numeros.length }) }}
      <template v-if="store.porConfirmar > 0">
        ·
        {{ t('numeroDeWhatsapp.notaPorConfirmar', { porConfirmar: store.porConfirmar }) }}
      </template>
    </p>

    <AModulo
      v-for="n in store.numeros"
      :key="n.id"
      :titulo="n.nombreVisible"
    >
      <div class="fila">
        <span class="telefono">{{ n.telefonoE164 }}</span>
        <AInsignia :tono="n.puedeEnviar ? 'exito' : 'aviso'">
          {{
            n.puedeEnviar
              ? t('numeroDeWhatsapp.puedeEnviar')
              : t('numeroDeWhatsapp.noPuedeEnviar')
          }}
        </AInsignia>
        <AInsignia tono="neutro">
          {{ t(`numeroDeWhatsapp.estado.${n.estado}`) }}
        </AInsignia>
        <AInsignia tono="neutro">
          {{ t(`whatsapp.modo.${n.modo === 'seguimiento' ? 'seguimiento' : 'prospeccionDirecta'}`) }}
        </AInsignia>
      </div>

      <!-- La calidad, y lo que significa que no conste. -->
      <div class="fila">
        <span class="etiqueta">{{ t('numeroDeWhatsapp.calidad.etiqueta') }}</span>
        <AInsignia :tono="tonoDeCalidad(n)">
          {{ textoDeCalidad(n) }}
        </AInsignia>
      </div>
      <p
        v-if="n.calidad === null"
        class="ayuda"
      >
        {{ t('numeroDeWhatsapp.calidad.ayudaSinConsultar') }}
      </p>

      <p class="ayuda">
        {{
          n.limiteDeMeta === null
            ? t('numeroDeWhatsapp.limiteSinSaber')
            : t('numeroDeWhatsapp.limiteDeMeta', { cuantos: n.limiteDeMeta })
        }}
      </p>

      <!-- ── Coexistencia ── -->
      <AAviso
        v-if="n.coexistenciaConfirmada === null"
        tono="aviso"
      >
        <strong>{{ t('numeroDeWhatsapp.coexistencia.titulo') }}</strong>
        <p>{{ t('numeroDeWhatsapp.coexistencia.cuerpo') }}</p>
        <p>{{ t('numeroDeWhatsapp.coexistencia.porQue') }}</p>
        <ABoton
          variante="secundario"
          @click="store.confirmarCoexistencia(n.id)"
        >
          {{ t('numeroDeWhatsapp.coexistencia.boton') }}
        </ABoton>
      </AAviso>
      <p
        v-else
        class="confirmado"
      >
        {{ t('numeroDeWhatsapp.coexistencia.confirmada', { cuando: n.coexistenciaConfirmada }) }}
      </p>

      <!-- ── El riesgo del modo en frío. Solo aparece si el modo lo pide. ── -->
      <template v-if="n.escribeEnFrio">
        <AAviso
          v-if="n.riesgoAceptado === null"
          tono="peligro"
        >
          <strong>{{ t('numeroDeWhatsapp.riesgo.titulo') }}</strong>
          <p>{{ t('numeroDeWhatsapp.riesgo.cuerpo') }}</p>
          <p>{{ t('numeroDeWhatsapp.riesgo.porQue') }}</p>
          <ABoton
            variante="secundario"
            @click="store.aceptarRiesgo(n.id)"
          >
            {{ t('numeroDeWhatsapp.riesgo.boton') }}
          </ABoton>
        </AAviso>
        <p
          v-else
          class="confirmado"
        >
          {{ t('numeroDeWhatsapp.riesgo.aceptado', { cuando: n.riesgoAceptado }) }}
        </p>
      </template>
    </AModulo>
  </section>
</template>

<style scoped>
/* §17: ni un hex, ni un píxel suelto. Todo sale de los tokens. */
.pantalla {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.cabecera {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--arles-space-4);
}

.explicacion {
  color: var(--arles-text-muted);
  margin-top: var(--arles-space-2);
}

.campos {
  display: grid;
  gap: var(--arles-space-4);
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-4);
}

.fila {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--arles-space-3);
  margin-bottom: var(--arles-space-3);
}

.telefono {
  font-variant-numeric: tabular-nums;
  font-weight: var(--arles-font-weight-semibold);
}

.etiqueta,
.ayuda {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.confirmado {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  margin-top: var(--arles-space-3);
}

.nota {
  color: var(--arles-text-muted);
  font-variant-numeric: tabular-nums;
}
</style>
