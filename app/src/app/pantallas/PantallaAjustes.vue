<script setup lang="ts">
/**
 * Ajustes · Configuración de empresa (entrega 3.1).
 *
 * Es la primera pantalla real del producto y el primer paso del alta (§25).
 *
 * Dos decisiones que se ven aquí:
 *
 * 1. **Los desplegables se llenan con lo que dice el núcleo**, no con una
 *    lista escrita en esta pantalla. Si divergieran, el usuario elegiría una
 *    zona de la lista y el núcleo se la rechazaría — un error imposible de
 *    entender desde fuera.
 * 2. **Esta pantalla no valida.** Envía y pinta lo que el núcleo responda. Dos
 *    validaciones son dos reglas que mantener iguales, y el día que divergen el
 *    formulario aprueba lo que el núcleo rechaza.
 */
import { computed, nextTick, onMounted, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

import { resolverError } from '@/app/errores'
import { useEmpresaStore, type BorradorDeEmpresa } from '@/app/stores/empresa'
import {
  AAviso,
  ABoton,
  AEntrada,
  AIcono,
  ASelector,
  EstadoExito,
} from '@/design/componentes'

const empresa = useEmpresaStore()
const { t, te } = useI18n()

/**
 * El error general, ya resuelto a sus tres partes.
 *
 * Pasa por `resolverError` y no por `$t('clave.que')` porque esa función es la
 * que sabe qué hacer cuando **falta** el texto: deja constancia y cae en un
 * genérico legible. Con `$t` a pelo, una clave sin texto se pinta cruda —el
 * usuario leería `error.db.dato_invalido.que` como título del aviso.
 */
const errorGeneral = computed(() =>
  empresa.errorGeneral
    ? resolverError({ clave: empresa.errorGeneral, detalle: '' })
    : null,
)

/** Hay campos marcados: se anuncia, además de marcarlos. */
const hayErroresDeCampo = computed(
  () => Object.keys(empresa.erroresDeCampo).length > 0,
)
const resumenDeCampos = computed(() =>
  resolverError({ clave: 'error.app.empresa_invalida', detalle: '' }),
)

const formulario = reactive<BorradorDeEmpresa>({
  nombreComercial: '',
  pais: 'MX',
  zonaHoraria: 'America/Mexico_City',
  correoCorporativo: '',
  sitioWeb: '',
})

/** Rellena el formulario con lo guardado, cuando llega. */
watch(
  () => empresa.empresa,
  (e) => {
    if (!e) return
    formulario.nombreComercial = e.nombreComercial
    formulario.pais = e.pais
    formulario.zonaHoraria = e.zonaHoraria
    formulario.correoCorporativo = e.correoCorporativo
    formulario.sitioWeb = e.sitioWeb ?? ''
  },
  { immediate: true },
)

onMounted(() => void empresa.cargar())

/**
 * El texto del error de un campo.
 *
 * Si la clave no existe en el catálogo se devuelve un texto genérico en vez de
 * pintar la clave cruda: enseñar `empresa.error.zonaNoSoportada` a un usuario
 * es peor que no decir nada. La clave que falta se detecta en las pruebas, que
 * es donde toca.
 */
function errorDe(campo: keyof BorradorDeEmpresa): string {
  const clave = empresa.erroresDeCampo[campo]
  // Cadena vacía y no `undefined`: con `exactOptionalPropertyTypes` activado,
  // pasar `undefined` a una propiedad opcional es un error de tipos, y las
  // primitivas ya tratan la cadena vacía como «sin error».
  if (!clave) return ''
  return te(clave) ? t(clave) : t('empresa.error.generico')
}

/**
 * El nombre legible de una opción del desplegable.
 *
 * El valor que viaja al núcleo no cambia —sigue siendo `MX` o
 * `America/Mexico_City`—: esto es sólo lo que se lee. Si falta el nombre se
 * enseña el valor crudo, que es feo pero no engaña; el validador comprueba que
 * no falte ninguno.
 */
function nombreDe(grupo: 'pais' | 'zona', valor: string): string {
  const clave = `empresa.${grupo}.${valor}`
  return te(clave) ? t(clave) : valor
}

const opcionesDePais = computed(() =>
  empresa.configuracion.paises.map((p) => ({ valor: p, texto: nombreDe('pais', p) })),
)
const opcionesDeZona = computed(() =>
  empresa.configuracion.zonas.map((z) => ({ valor: z, texto: nombreDe('zona', z) })),
)

async function enviar(): Promise<void> {
  const guardado = await empresa.guardar({ ...formulario })
  if (guardado) return

  // Marcar el campo en rojo no le sirve de nada a quien no ve la pantalla, y
  // tampoco a quien acaba de pulsar con el teclado y sigue con el foco en el
  // botón. Se lleva el foco al primer campo malo: el lector de pantalla
  // anuncia ahí su etiqueta y su mensaje, enlazado por `aria-describedby`.
  await nextTick()
  const primero = document.querySelector<HTMLElement>('[aria-invalid="true"]')
  primero?.focus()
}
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ $t('nav.ajustes') }}
      </h1>
      <p class="entradilla">
        {{ $t('empresa.entradilla') }}
      </p>
    </header>

    <!-- `@input` en el formulario y no un `watch` sobre el objeto: «Configuración
         guardada» junto a un formulario que ya se está editando afirma algo que
         ha dejado de ser cierto, pero vigilar el objeto descartaba el aviso en
         el mismo instante en que aparecía —al guardar, la respuesta del núcleo
         rellena el formulario con los valores normalizados, y eso es una
         escritura—. El evento nativo sólo lo dispara una persona escribiendo. -->
    <form
      class="formulario"
      novalidate
      @input="empresa.descartarAviso()"
      @submit.prevent="enviar"
    >
      <h2 class="subtitulo">
        {{ $t('empresa.titulo') }}
      </h2>

      <AEntrada
        v-model="formulario.nombreComercial"
        :etiqueta="$t('empresa.campo.nombreComercial')"
        :ayuda="$t('empresa.ayuda.nombreComercial')"
        :error="errorDe('nombreComercial')"
        requerido
        autocompletado="organization"
      />

      <ASelector
        v-model="formulario.pais"
        :etiqueta="$t('empresa.campo.pais')"
        :ayuda="$t('empresa.ayuda.pais')"
        :error="errorDe('pais')"
        :opciones="opcionesDePais"
      />

      <ASelector
        v-model="formulario.zonaHoraria"
        :etiqueta="$t('empresa.campo.zonaHoraria')"
        :ayuda="$t('empresa.ayuda.zonaHoraria')"
        :error="errorDe('zonaHoraria')"
        :opciones="opcionesDeZona"
      />

      <AEntrada
        v-model="formulario.correoCorporativo"
        tipo="email"
        :etiqueta="$t('empresa.campo.correoCorporativo')"
        :ayuda="$t('empresa.ayuda.correoCorporativo')"
        :error="errorDe('correoCorporativo')"
        requerido
      />

      <AEntrada
        v-model="formulario.sitioWeb"
        :etiqueta="$t('empresa.campo.sitioWeb')"
        :ayuda="$t('empresa.ayuda.sitioWeb')"
        :error="errorDe('sitioWeb')"
        marcador="https://"
      />

      <!-- Dos avisos distintos a propósito: uno dice que el formulario tiene
           campos malos —y los campos están marcados—, y el otro que el fallo
           no es del formulario. Mezclarlos haría buscar un campo rojo que no
           existe. -->
      <AAviso
        v-if="hayErroresDeCampo"
        tono="peligro"
        urgente
        :titulo="resumenDeCampos.que"
      >
        {{ resumenDeCampos.como }} {{ resumenDeCampos.salvo }}
      </AAviso>

      <AAviso
        v-if="errorGeneral"
        tono="peligro"
        urgente
        :titulo="errorGeneral.que"
      >
        {{ errorGeneral.como }} {{ errorGeneral.salvo }}
      </AAviso>

      <EstadoExito
        v-if="empresa.guardadoConExito"
        :titulo="$t('empresa.guardada')"
        :detalle="$t('empresa.guardadaDetalle')"
      />

      <div class="acciones">
        <ABoton
          variante="primario"
          tipo="submit"
          :ocupado="empresa.guardando"
        >
          {{ $t('empresa.guardar') }}
        </ABoton>
      </div>
    </form>

    <!-- §67: la zona horaria no es un dato decorativo. Decide a qué hora sale
         cada correo, y decirlo aquí evita la conversación de por qué una
         campaña salió a las tres de la mañana. -->
    <AAviso
      tono="info"
      :titulo="$t('empresa.porQueLaZona')"
    >
      {{ $t('empresa.porQueLaZonaDetalle') }}
    </AAviso>

    <!-- ── Los seis pasos del alta ──────────────────────────────────────────
         Vivían en Inicio. Dirección pidió que Inicio dejara de ser una lista
         de configuración, así que el detalle se muda aquí, que es donde se
         configura, y en Inicio queda la cifra y la acción siguiente.

         Lo que NO se muda es la razón por la que la lista enseña los seis
         desde el primer día: con sólo los pasos construidos, alguien vería la
         lista completa al terminar el primero y concluiría que ya puede
         enviar. Por eso los que aún no existen siguen apareciendo, con su
         entrega, y sin ser enlaces. -->
    <section
      class="alta"
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
          {{
            $t('inicio.avance', {
              hechos: empresa.onboarding.completados,
              total: empresa.onboarding.total,
            })
          }}
        </p>
      </div>

      <ol class="lista">
        <li
          v-for="paso in empresa.onboarding.pasos"
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

          <div class="cuerpo-paso">
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

          <!-- Texto apagado y no `AInsignia`: con relleno sólido, los cinco
               pasos que NO se pueden hacer pesaban más en la pantalla que el
               único que sí. -->
          <p
            v-if="!paso.disponible"
            class="cuando"
          >
            {{ $t('inicio.llegaEn', { entrega: paso.entrega }) }}
          </p>
        </li>
      </ol>
    </section>
  </section>
</template>

<style scoped>
.alta {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.cabecera-alta {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--arles-space-4);
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

.cuerpo-paso {
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

.cuando {
  margin: 0;
  flex: none;
  align-self: center;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  white-space: nowrap;
}

.pantalla {
  /* Un formulario sí es pantalla de lectura: su medida y su tope coinciden.
     Se declara igual que en el panel para que la sonda del umbral de plegado
     mida lo mismo en las dos. */
  --arles-medida: 540px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
  max-width: 62ch;
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

.entradilla {
  margin: 0;
  color: var(--arles-text-muted);
}

.formulario {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
}
</style>
