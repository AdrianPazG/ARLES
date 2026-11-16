<script setup lang="ts">
/**
 * Configurar ARLES · los datos de la empresa y los seis pasos.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTA PANTALLA EXISTE, Y POR QUÉ CUELGA DE INICIO
 *
 * Dirección lo señaló el 21/09/2026: pulsar «Configurar mi empresa» en Inicio
 * saltaba a **Ajustes**, y Ajustes es para la aplicación —tema, densidad,
 * versión, respaldos—, no para el negocio.
 *
 * El error de fondo venía de una decisión mía del 17/09 que decía «la lista se
 * muda a Ajustes, que es donde se configura». Esa frase mezcla dos cosas que
 * no son la misma:
 *
 *   - **Configurar la aplicación:** una preferencia de la herramienta.
 *   - **Configurar el negocio:** el nombre, el país y la zona horaria, que
 *     deciden a qué hora sale cada correo y qué marco legal aplica.
 *
 * Y la lista de los seis pasos no es ninguna de las dos: es una **guía con
 * final**, que debe desaparecer cuando se complete. Un ajuste no desaparece
 * nunca.
 *
 * Cuelga de INICIO y no es una séptima sección porque Dirección cerró esa
 * puerta el 17/09 al rechazar CONVERSACIONES —el producto son seis—, y porque
 * una sección de «configuración inicial» es una entrada en la barra que,
 * pasada la primera semana, no se vuelve a usar y sigue ocupando sitio.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO GUARDADO NO SE TOCA HASTA PULSAR «EDITAR»
 *
 * Segunda decisión del 21/09. La primera vez, todos los campos abiertos. Al
 * guardar, quedan en **sólo lectura** —no deshabilitados: contraste normal, y
 * el valor se puede seleccionar y copiar— hasta que alguien pulse «Editar».
 *
 * La zona horaria decide a qué hora sale cada campaña. Cambiarla con un roce
 * del ratón, sin querer y sin enterarse, es la clase de accidente que se
 * descubre tres semanas después mirando por qué un envío salió de madrugada.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

import { resolverError } from '@/app/errores'
import { useEmpresaStore, type BorradorDeEmpresa } from '@/app/stores/empresa'
import { AAviso, ABoton, AEntrada, AIcono, ASelector, EstadoExito } from '@/design/componentes'

const empresa = useEmpresaStore()
const { t, te } = useI18n()

const formulario = reactive<BorradorDeEmpresa>({
  nombreComercial: '',
  pais: 'MX',
  zonaHoraria: 'America/Mexico_City',
  correoCorporativo: '',
  sitioWeb: '',
})

/**
 * ¿Se están editando los campos?
 *
 * Arranca abierto y **sólo se cierra cuando hay algo guardado**. La primera
 * vez nadie ha guardado nada, así que cerrarlo dejaría al usuario delante de
 * un formulario vacío y bloqueado, teniendo que descubrir que hay que pulsar
 * «Editar» para empezar.
 */
const editando = ref(true)

/** Rellena el formulario con lo guardado, cuando llega, y lo cierra. */
watch(
  () => empresa.empresa,
  (e) => {
    if (!e) return
    formulario.nombreComercial = e.nombreComercial
    formulario.pais = e.pais
    formulario.zonaHoraria = e.zonaHoraria
    formulario.correoCorporativo = e.correoCorporativo
    formulario.sitioWeb = e.sitioWeb ?? ''
    editando.value = false
  },
  { immediate: true },
)

onMounted(() => void empresa.cargar())

/** Se abre para editar. El aviso de «guardado» sobra en cuanto se toca algo. */
function editar(): void {
  empresa.descartarAviso()
  editando.value = true
}

/**
 * Se cancela la edición y **se restaura lo guardado**.
 *
 * Sin restaurar, cancelar dejaría en pantalla los valores a medio escribir,
 * bloqueados y distintos de los que están en la base — que es exactamente la
 * confusión que esta pantalla existe para evitar.
 */
function cancelar(): void {
  const e = empresa.empresa
  if (e) {
    formulario.nombreComercial = e.nombreComercial
    formulario.pais = e.pais
    formulario.zonaHoraria = e.zonaHoraria
    formulario.correoCorporativo = e.correoCorporativo
    formulario.sitioWeb = e.sitioWeb ?? ''
  }
  empresa.descartarAviso()
  editando.value = false
}

const errorGeneral = computed(() =>
  empresa.errorGeneral ? resolverError({ clave: empresa.errorGeneral, detalle: '' }) : null,
)

const hayErroresDeCampo = computed(() => Object.keys(empresa.erroresDeCampo).length > 0)
const resumenDeCampos = computed(() =>
  resolverError({ clave: 'error.app.empresa_invalida', detalle: '' }),
)

function errorDe(campo: keyof BorradorDeEmpresa): string {
  const clave = empresa.erroresDeCampo[campo]
  if (!clave) return ''
  return te(clave) ? t(clave) : t('empresa.error.generico')
}

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
  if (guardado) {
    // Se cierra aquí y no sólo en el `watch`: al reguardar los mismos valores
    // el objeto puede no cambiar, el `watch` no dispara, y el formulario se
    // quedaría abierto después de un guardado correcto.
    editando.value = false
    return
  }

  await nextTick()
  const primero = document.querySelector<HTMLElement>('[aria-invalid="true"]')
  primero?.focus()
}
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ $t('configuracion.titulo') }}
      </h1>
      <p class="entradilla">
        {{ $t('configuracion.entradilla') }}
      </p>
    </header>

    <div class="dos-columnas">
      <form
        class="formulario"
        novalidate
        @input="empresa.descartarAviso()"
        @submit.prevent="enviar"
      >
        <div class="cabecera-empresa">
          <h2 class="subtitulo">
            {{ $t('empresa.titulo') }}
          </h2>
          <!-- El botón de editar sólo existe cuando hay algo que proteger. -->
          <ABoton
            v-if="!editando"
            variante="secundario"
            tamano="compacto"
            @click="editar"
          >
            {{ $t('configuracion.editar') }}
          </ABoton>
        </div>

        <AEntrada
          v-model="formulario.nombreComercial"
          :etiqueta="$t('empresa.campo.nombreComercial')"
          :ayuda="$t('empresa.ayuda.nombreComercial')"
          :error="errorDe('nombreComercial')"
          :solo-lectura="!editando"
          requerido
          autocompletado="organization"
        />

        <ASelector
          v-model="formulario.pais"
          :etiqueta="$t('empresa.campo.pais')"
          :ayuda="$t('empresa.ayuda.pais')"
          :error="errorDe('pais')"
          :opciones="opcionesDePais"
          :solo-lectura="!editando"
        />

        <ASelector
          v-model="formulario.zonaHoraria"
          :etiqueta="$t('empresa.campo.zonaHoraria')"
          :ayuda="$t('empresa.ayuda.zonaHoraria')"
          :error="errorDe('zonaHoraria')"
          :opciones="opcionesDeZona"
          :solo-lectura="!editando"
        />

        <AEntrada
          v-model="formulario.correoCorporativo"
          tipo="email"
          :etiqueta="$t('empresa.campo.correoCorporativo')"
          :ayuda="$t('empresa.ayuda.correoCorporativo')"
          :error="errorDe('correoCorporativo')"
          :solo-lectura="!editando"
          requerido
        />

        <AEntrada
          v-model="formulario.sitioWeb"
          :etiqueta="$t('empresa.campo.sitioWeb')"
          :ayuda="$t('empresa.ayuda.sitioWeb')"
          :error="errorDe('sitioWeb')"
          :solo-lectura="!editando"
          marcador="https://"
        />

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

        <!-- Sin botones cuando no se está editando: un «Guardar» junto a
             campos que no se pueden escribir no tiene nada que guardar. -->
        <div
          v-if="editando"
          class="acciones"
        >
          <ABoton
            variante="primario"
            tipo="submit"
            :ocupado="empresa.guardando"
          >
            {{ $t('empresa.guardar') }}
          </ABoton>
          <!-- Cancelar sólo aparece si hay algo a lo que volver. La primera
               vez no hay nada guardado, así que cancelar no significa nada. -->
          <ABoton
            v-if="empresa.empresa"
            variante="sutil"
            @click="cancelar"
          >
            {{ $t('configuracion.cancelar') }}
          </ABoton>
        </div>
      </form>

      <aside class="contexto">
        <!-- §67: la zona horaria no es un dato decorativo. Decide a qué hora
             sale cada correo, y decirlo aquí evita la conversación de por qué
             una campaña salió a las tres de la mañana. -->
        <AAviso
          tono="info"
          :titulo="$t('empresa.porQueLaZona')"
        >
          {{ $t('empresa.porQueLaZonaDetalle') }}
        </AAviso>

        <!-- ── Los seis pasos ──────────────────────────────────────────────
             Enseña los seis desde el primer día, también los que aún no
             existen. Con sólo los construidos, alguien vería la lista
             completa al terminar el primero y concluiría que ya puede
             enviar. -->
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
              <AIcono
                class="marca"
                :nombre="paso.completado ? 'exito' : 'cola'"
                :etiqueta="paso.completado ? $t('inicio.hecho') : $t('inicio.pendiente')"
              />

              <div class="cuerpo-paso">
                <p class="nombre">
                  <component
                    :is="paso.ruta ? RouterLink : 'span'"
                    :to="paso.ruta ?? undefined"
                  >
                    {{ $t(`inicio.paso.${paso.clave}.titulo`) }}
                  </component>
                </p>
                <p class="detalle">
                  {{ $t(`inicio.paso.${paso.clave}.detalle`) }}
                </p>
              </div>

              <!-- Aquí iba «Llega en la entrega 6», que es el número de una
                   entrega de nuestro roadmap en la pantalla de alguien que no
                   trabaja aquí. Se fue el 28/10/2026 con el arreglo de D-01:
                   los seis pasos están construidos, así que no había ninguno
                   del que decirlo — y mientras lo hubo, lo decía de dos que sí
                   estaban hechos. Es el mismo defecto que el 29/09/2026 sacó
                   «roadmap», «andamiaje» y «tokens» de Actividad. -->
            </li>
          </ol>
        </section>
      </aside>
    </div>
  </section>
</template>

<style scoped>
.pantalla {
  --arles-medida: 540px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.dos-columnas {
  display: grid;
  gap: var(--arles-space-6);
}

@media (width >= 1240px) {
  .dos-columnas {
    grid-template-columns: 62ch 1fr;
    align-items: start;
  }
}

.contexto {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
  min-width: 0;
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

.cabecera-empresa {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--arles-space-4);
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
}

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
</style>
