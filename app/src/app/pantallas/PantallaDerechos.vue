<script setup lang="ts">
/**
 * Derechos de las personas · el de **Acceso** (entrega 3.4).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ ES ESTO, EN CASTELLANO
 *
 * Alguien escribe al cliente diciendo «¿qué datos tienen de mí?». La ley
 * mexicana (LFPDPPP) le da derecho a saberlo, y obliga a responder con TODO
 * lo que consta. Esta pantalla produce ese informe en un archivo de texto.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * TRES COSAS QUE PARECEN DETALLES Y SON EL FONDO DEL ASUNTO
 *
 * 1 · SE BUSCA POR DIRECCIÓN, NO POR CONTACTO.
 *
 *     Quien pide sus datos puede haber pedido antes que lo borraran. Si la
 *     búsqueda fuera «dame el contacto con este id», esa persona recibiría
 *     «no tenemos nada» — y sería mentira, porque queda su baja, su rastro en
 *     los envíos y su paso por una importación. El expediente se arma desde
 *     la dirección justamente para que el informe sea cierto también cuando
 *     el contacto ya no existe.
 *
 * 2 · «NO HAY NADA» NO ES UN ERROR, ES UNA RESPUESTA.
 *
 *     Y de las más valiosas: puede ser exactamente lo que la persona quería
 *     oír. El informe se entrega igual, diciéndolo. Esta pantalla NO pinta un
 *     aviso rojo por eso.
 *
 * 3 · NINGUNA RUTA CRUZA LA FRONTERA (regla 4 de ARQUITECTURA.md §5).
 *
 *     El diálogo de guardar lo abre Rust. La pantalla manda una dirección y
 *     recibe **el nombre del archivo**, nunca la carpeta. Por eso aquí no hay
 *     ni un `open`, ni un `save`, ni un `path`: la webview no tiene esos
 *     permisos y no le hacen falta.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * DE QUIÉN ES LA OBLIGACIÓN
 *
 * Del **cliente**, no de ARLES (PRIVACIDAD_LFPDPPP.md §2: el responsable es
 * quien trata los datos). ARLES es la herramienta que le permite cumplir sin
 * llamar a un programador. Se dice en pantalla porque quien atiende la
 * solicitud necesita saber que la respuesta la firma su empresa.
 */
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import { esErrorIpc, hayNucleo, invocar } from '@/app/nucleo'
import { AAviso, ABoton, AEntrada, AModulo, ASelector } from '@/design/componentes'

const { t } = useI18n()

/**
 * Lo que devuelve el comando cuando el informe se escribió.
 *
 * `null` significa que quien atiende **cerró el diálogo sin guardar**. Cancelar
 * es una decisión, no un fallo, y se trata como tal.
 */
interface InformeEntregado {
  readonly nombreDelArchivo: string
  readonly habiaDatos: boolean
  readonly sigueSiendoContacto: boolean
}

/** Lo que se va a destruir, antes de destruirlo. Lo calcula el núcleo. */
interface VistaPrevia {
  readonly direccion: string
  readonly nombre: string
  readonly sigueSiendoContacto: boolean
  readonly direcciones: number
  readonly campos: number
  readonly pertenencias: number
  readonly envios: number
  readonly tieneBaja: boolean
}

/** Lo que pasó al borrar. */
interface Cancelacion {
  readonly contactosBorrados: number
  readonly canalesBorrados: number
  readonly camposBorrados: number
  readonly pertenenciasBorradas: number
  readonly enviosAnonimizados: number
  readonly audienciasAnonimizadas: number
  readonly laBajaYaExistia: boolean
}

type Canal = 'email' | 'whatsapp'

const canal = ref<Canal>('email')
const direccion = ref('')
const trabajando = ref(false)

// ── El borrado, que es otra cosa ─────────────────────────────────────────
//
// Vive en esta pantalla y no en otra porque es uno de los cuatro derechos, y
// quien atiende una solicitud no sabe de antemano cuál le pidieron. Pero NO
// comparte estado con el informe: mezclarlos dejaría el resultado de uno
// visible mientras se prepara el otro.
const borrarDireccion = ref('')
const borrarCanal = ref<Canal>('email')
const vistaPrevia = ref<VistaPrevia | null>(null)
const confirmacion = ref('')
const borrando = ref(false)
const borrado = ref<Cancelacion | null>(null)
const errorDeBorrado = ref<{ que: string; como: string; salvo: string } | null>(null)

/** El resultado de la última entrega. `null` mientras no haya ninguna. */
const entregado = ref<InformeEntregado | null>(null)
/** Se canceló el diálogo de guardar. No es un error y no se pinta como tal. */
const cancelado = ref(false)
const error = ref<{ que: string; como: string; salvo: string } | null>(null)

const opcionesDeCanal = computed(() => [
  { valor: 'email', texto: t('derechos.canal.correo') },
  { valor: 'whatsapp', texto: t('derechos.canal.whatsapp') },
])

/**
 * El marcador cambia con el canal.
 *
 * Un marcador de correo en un campo de WhatsApp enseña la forma equivocada
 * justo en el momento en que alguien duda de qué escribir, y en este campo
 * escribir lo que no es significa entregar un informe vacío a quien sí tiene
 * datos — con la otra forma.
 */
const marcador = computed(() =>
  canal.value === 'email'
    ? t('derechos.marcadorCorreo')
    : t('derechos.marcadorWhatsApp'),
)

const sePuedePedir = computed(() => direccion.value.trim() !== '' && !trabajando.value)

/** Lo que se enseña tras entregar. Se calcula aquí para no repetir `v-if`. */
const resultado = computed(() => {
  const i = entregado.value
  if (!i) return null
  // Tres desenlaces, y los tres son un éxito. Lo que cambia es QUÉ se
  // encontró, y eso se dice: entregar un informe vacío sin avisar dejaría a
  // quien atiende creyendo que la búsqueda falló.
  if (!i.habiaDatos) {
    return {
      titulo: t('derechos.listoSinDatos', { archivo: i.nombreDelArchivo }),
      detalle: t('derechos.listoSinDatosDetalle'),
    }
  }
  if (!i.sigueSiendoContacto) {
    return {
      titulo: t('derechos.listoSoloRastro', { archivo: i.nombreDelArchivo }),
      detalle: t('derechos.listoSoloRastroDetalle'),
    }
  }
  return {
    titulo: t('derechos.listo', { archivo: i.nombreDelArchivo }),
    detalle: t('derechos.listoDetalle'),
  }
})

async function pedirElInforme(): Promise<void> {
  if (!sePuedePedir.value) return

  // Cada intento empieza en limpio. Sin esto, un error viejo seguiría en
  // pantalla junto al resultado nuevo y no se sabría cuál es de cuál.
  entregado.value = null
  cancelado.value = false
  error.value = null

  if (!hayNucleo()) {
    error.value = {
      que: t('derechos.sinNucleoQue'),
      como: t('derechos.sinNucleoComo'),
      salvo: t('derechos.sinNucleoSalvo'),
    }
    return
  }

  trabajando.value = true
  try {
    const informe = await invocar<InformeEntregado | null>('entregar_informe_de_acceso', {
      canal: canal.value,
      direccion: direccion.value.trim(),
    })
    if (informe === null) cancelado.value = true
    else entregado.value = informe
  } catch (e) {
    // El error llega con su clave de i18n y se traduce a las tres frases del
    // §95: qué pasó, cómo se arregla y qué está a salvo. Nunca se enseña el
    // texto técnico.
    error.value = esErrorIpc(e)
      ? resolverError(e)
      : {
          que: t('derechos.falloQue'),
          como: t('derechos.falloComo'),
          salvo: t('derechos.falloSalvo'),
        }
  } finally {
    trabajando.value = false
  }
}

// ─── El derecho de Cancelación ────────────────────────────────────────────

const sePuedeMirar = computed(
  () => borrarDireccion.value.trim() !== '' && !borrando.value,
)

/**
 * ¿Hay algo de esa persona?
 *
 * Se recalcula aquí y **no viaja desde Rust**: allí es un método, no un campo,
 * y serializar el resultado de un método sería guardar una conclusión que
 * puede quedarse vieja. La regla es la misma que la del núcleo —sigue siendo
 * contacto, o hay envíos, o hay baja— y hay una prueba que compara las dos.
 */
const hayAlgoQueBorrar = computed(() => {
  const v = vistaPrevia.value
  if (!v) return false
  return v.sigueSiendoContacto || v.envios > 0 || v.tieneBaja
})

/** Lo que desaparece del todo, en una sola cifra. */
const totalABorrar = computed(() => {
  const v = vistaPrevia.value
  return v ? v.direcciones + v.campos + v.pertenencias : 0
})

/**
 * La confirmación: hay que **escribir la dirección completa**.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESCRIBIRLA Y NO MARCAR UNA CASILLA
 *
 * Una casilla de «entiendo que es irreversible» se marca sin leerla. Es un
 * trámite, no una decisión, y la prueba es que todo el mundo las marca.
 *
 * Escribir la dirección obliga a **mirarla**. Y si quien atiende se equivocó
 * de persona al teclear arriba, aquí vuelve a leerla y lo ve — que es el
 * accidente concreto que esto existe para evitar: borrar a la persona
 * equivocada, sin deshacer.
 *
 * Se compara contra la dirección que devolvió el NÚCLEO ya normalizada, no
 * contra lo que se escribió arriba: así «Ana@Empresa.MX» también vale, y la
 * comparación es la misma que usará el borrado.
 */
const confirmacionCorrecta = computed(() => {
  const esperada = vistaPrevia.value?.direccion ?? ''
  return (
    esperada !== '' &&
    confirmacion.value.trim().toLowerCase() === esperada.toLowerCase()
  )
})

/** Limpia el paso de confirmación. Se llama al cambiar lo que se va a borrar. */
function olvidarLaVistaPrevia(): void {
  vistaPrevia.value = null
  confirmacion.value = ''
  borrado.value = null
  errorDeBorrado.value = null
}

async function mirarQueSeBorraria(): Promise<void> {
  if (!sePuedeMirar.value) return
  olvidarLaVistaPrevia()

  if (!hayNucleo()) {
    errorDeBorrado.value = {
      que: t('derechos.sinNucleoQue'),
      como: t('derechos.sinNucleoComo'),
      salvo: t('derechos.sinNucleoSalvo'),
    }
    return
  }

  borrando.value = true
  try {
    vistaPrevia.value = await invocar<VistaPrevia>('que_se_borraria', {
      canal: borrarCanal.value,
      direccion: borrarDireccion.value.trim(),
    })
  } catch (e) {
    errorDeBorrado.value = esErrorIpc(e)
      ? resolverError(e)
      : {
          que: t('derechos.falloQue'),
          como: t('derechos.falloComo'),
          salvo: t('derechos.falloSalvo'),
        }
  } finally {
    borrando.value = false
  }
}

async function borrarDeVerdad(): Promise<void> {
  // Dos condiciones, y las dos se comprueban aquí aunque el botón esté
  // deshabilitado: un botón deshabilitado es una capa de pintura, y esta
  // función no se puede ejecutar por accidente.
  if (!confirmacionCorrecta.value || borrando.value) return

  borrando.value = true
  errorDeBorrado.value = null
  try {
    borrado.value = await invocar<Cancelacion>('cancelar_datos_de', {
      canal: borrarCanal.value,
      direccion: borrarDireccion.value.trim(),
    })
    // La vista previa se retira: ya no describe nada que exista. Dejarla
    // puesta junto al resultado haría dudar de si se borró o no.
    vistaPrevia.value = null
    confirmacion.value = ''
  } catch (e) {
    errorDeBorrado.value = esErrorIpc(e)
      ? resolverError(e)
      : {
          que: t('derechos.borrarFalloQue'),
          como: t('derechos.borrarFalloComo'),
          salvo: t('derechos.borrarFalloSalvo'),
        }
  } finally {
    borrando.value = false
  }
}
</script>

<template>
  <div class="derechos">
    <AModulo
      :titulo="t('derechos.titulo')"
      principal
      nivel="h1"
    >
      <p class="derechos__explicacion">
        {{ t('derechos.explicacion') }}
      </p>

      <!-- De quién es la obligación. No es un tecnicismo legal de adorno:
           quien atiende la solicitud firma la respuesta en nombre de su
           empresa, y conviene que lo sepa antes de mandarla. -->
      <AAviso
        tono="info"
        :titulo="t('derechos.deQuienEsTitulo')"
      >
        {{ t('derechos.deQuienEs') }}
      </AAviso>

      <form
        class="derechos__formulario"
        @submit.prevent="pedirElInforme"
      >
        <ASelector
          v-model="canal"
          :etiqueta="t('derechos.canalEtiqueta')"
          :ayuda="t('derechos.canalAyuda')"
          :opciones="opcionesDeCanal"
        />

        <AEntrada
          v-model="direccion"
          :etiqueta="t('derechos.direccionEtiqueta')"
          :ayuda="t('derechos.direccionAyuda')"
          :marcador="marcador"
          :tipo="canal === 'email' ? 'email' : 'text'"
          requerido
        />

        <div class="derechos__acciones">
          <ABoton
            variante="primario"
            tipo="submit"
            :deshabilitado="!sePuedePedir"
            :ocupado="trabajando"
            :aria-describedby="!sePuedePedir && !trabajando ? 'derechos-falta-acceso' : undefined"
          >
            {{ t('derechos.entregar') }}
          </ABoton>
          <!-- Por qué está apagado (§95, D-04). -->
          <p
            v-if="!sePuedePedir && !trabajando"
            id="derechos-falta-acceso"
            class="derechos__falta"
          >
            {{ t('derechos.faltaLaDireccion') }}
          </p>
        </div>
      </form>

      <!-- ── Los desenlaces ─────────────────────────────────────────────────
           Cuatro, y sólo uno es un fallo. El orden importa: el error va
           primero porque es lo que exige atención; cancelar va al final
           porque no pide nada. -->
      <AAviso
        v-if="error"
        tono="peligro"
        urgente
      >
        <strong>{{ error.que }}</strong>
        <span>{{ error.como }}</span>
        <span>{{ error.salvo }}</span>
      </AAviso>

      <AAviso
        v-else-if="resultado"
        class="derechos__resultado"
        tono="exito"
        urgente
        :titulo="resultado.titulo"
      >
        {{ resultado.detalle }}
      </AAviso>

      <AAviso
        v-else-if="cancelado"
        class="derechos__resultado"
        tono="info"
      >
        {{ t('derechos.cancelado') }}
      </AAviso>

      <!-- Lo que ARLES NO hace, dicho aquí y no escondido en la documentación
           (§95 y el principio de honestidad del producto). Quien atiende una
           solicitud de borrado tiene que saber que todavía no está construido
           antes de prometérselo a nadie. -->
      <section
        class="derechos__pendiente"
        aria-labelledby="titulo-pendiente"
      >
        <!-- `h2` y no `h3`: es la subsección directa del módulo que hace de
             título de la pantalla (7.7). El tamaño lo da `__subtitulo`, no la
             etiqueta, así que no cambia nada de lo que se ve. -->
        <h2
          id="titulo-pendiente"
          class="derechos__subtitulo"
        >
          {{ t('derechos.otrosTitulo') }}
        </h2>
        <dl class="derechos__lista">
          <div>
            <dt>{{ t('derechos.rectificacion') }}</dt>
            <dd>{{ t('derechos.rectificacionDetalle') }}</dd>
          </div>
          <div>
            <dt>{{ t('derechos.oposicion') }}</dt>
            <dd>{{ t('derechos.oposicionDetalle') }}</dd>
          </div>
        </dl>
      </section>
    </AModulo>

    <!-- ══ EL BORRADO ══════════════════════════════════════════════════════
         Módulo aparte y no una sección más del de arriba. Es la única acción
         irreversible de toda la aplicación, y mezclarla con «preparar un
         informe» —que no rompe nada— invita a pulsarla con la misma ligereza.

         Va SEGUNDO a propósito: quien llega buscando el informe lo encuentra
         primero, y quien llega a borrar pasa por delante de lo que no hace
         falta borrar. -->
    <AModulo :titulo="t('derechos.borrarTitulo')">
      <p class="derechos__explicacion">
        {{ t('derechos.borrarExplicacion') }}
      </p>

      <AAviso
        tono="peligro"
        :titulo="t('derechos.borrarAvisoTitulo')"
      >
        {{ t('derechos.borrarAviso') }}
      </AAviso>

      <!-- ── Paso 1 · quién ─────────────────────────────────────────────── -->
      <form
        class="derechos__formulario"
        @submit.prevent="mirarQueSeBorraria"
      >
        <ASelector
          v-model="borrarCanal"
          :etiqueta="t('derechos.canalEtiqueta')"
          :opciones="opcionesDeCanal"
          @update:model-value="olvidarLaVistaPrevia"
        />

        <AEntrada
          v-model="borrarDireccion"
          :etiqueta="t('derechos.borrarDireccionEtiqueta')"
          :ayuda="t('derechos.borrarDireccionAyuda')"
          :marcador="borrarCanal === 'email' ? t('derechos.marcadorCorreo') : t('derechos.marcadorWhatsApp')"
          :tipo="borrarCanal === 'email' ? 'email' : 'text'"
          requerido
          @update:model-value="olvidarLaVistaPrevia"
        />

        <div class="derechos__acciones">
          <ABoton
            variante="secundario"
            tipo="submit"
            :deshabilitado="!sePuedeMirar"
            :ocupado="borrando && !vistaPrevia"
            :aria-describedby="!sePuedeMirar && !borrando ? 'derechos-falta-borrado' : undefined"
          >
            {{ t('derechos.verQueSeBorraria') }}
          </ABoton>
          <p
            v-if="!sePuedeMirar && !borrando"
            id="derechos-falta-borrado"
            class="derechos__falta"
          >
            {{ t('derechos.faltaLaDireccion') }}
          </p>
        </div>
      </form>

      <!-- ── Paso 2 · qué se destruye ────────────────────────────────────
           Las cifras van ANTES del botón, no después. Un «¿seguro?» sobre
           algo que no se ha visto no pregunta nada (§94: cifras, no
           adjetivos). -->
      <section
        v-if="vistaPrevia"
        class="borrar"
        aria-labelledby="titulo-que-se-borra"
      >
        <h3
          id="titulo-que-se-borra"
          class="derechos__subtitulo"
        >
          {{ t('derechos.esteEs') }}
        </h3>

        <!-- El nombre, para RECONOCER a la persona antes de borrarla. Fiarse
             de haber tecleado bien es exactamente como se borra a quien no
             era. -->
        <p class="borrar__quien">
          <strong>{{ vistaPrevia.direccion }}</strong>
          <span v-if="vistaPrevia.nombre"> · {{ vistaPrevia.nombre }}</span>
        </p>

        <template v-if="hayAlgoQueBorrar">
          <dl class="borrar__cifras">
            <div class="borrar__cifra borrar__cifra--va">
              <dt>{{ t('derechos.seBorra') }}</dt>
              <dd>
                <span class="borrar__numero">{{ totalABorrar }}</span>
                {{ t('derechos.seBorraDetalle') }}
              </dd>
            </div>
            <div class="borrar__cifra borrar__cifra--queda">
              <dt>{{ t('derechos.seConserva') }}</dt>
              <dd>
                <span class="borrar__numero">{{ vistaPrevia.envios }}</span>
                {{ t('derechos.seConservaDetalle') }}
              </dd>
            </div>
          </dl>

          <p class="borrar__baja">
            {{ vistaPrevia.tieneBaja ? t('derechos.bajaYaEstaba') : t('derechos.bajaSeCrea') }}
          </p>
        </template>

        <!-- «No hay nada» no es un error: es una respuesta. Y la baja se crea
             igual, para que siga sin haberlo. -->
        <AAviso
          v-else
          tono="info"
        >
          {{ t('derechos.nadaQueBorrar') }}
        </AAviso>

        <!-- ── Paso 3 · confirmar escribiendo ─────────────────────────────
             Escribir la dirección y no marcar una casilla: una casilla se
             marca sin leerla. Ver la nota de `confirmacionCorrecta`. -->
        <form
          class="borrar__confirmar"
          @submit.prevent="borrarDeVerdad"
        >
          <AEntrada
            v-model="confirmacion"
            :etiqueta="t('derechos.confirmarEtiqueta', { direccion: vistaPrevia.direccion })"
            :ayuda="t('derechos.confirmarAyuda')"
            :marcador="vistaPrevia.direccion"
            autocompletado="off"
          />
          <div class="derechos__acciones">
            <ABoton
              variante="peligro"
              tipo="submit"
              :deshabilitado="!confirmacionCorrecta"
              :ocupado="borrando"
            >
              {{ t('derechos.borrarAhora') }}
            </ABoton>
            <ABoton
              variante="sutil"
              @click="olvidarLaVistaPrevia"
            >
              {{ t('derechos.mejorNo') }}
            </ABoton>
          </div>
        </form>
      </section>

      <AAviso
        v-if="errorDeBorrado"
        tono="peligro"
        urgente
      >
        <strong>{{ errorDeBorrado.que }}</strong>
        <span>{{ errorDeBorrado.como }}</span>
        <span>{{ errorDeBorrado.salvo }}</span>
      </AAviso>

      <!-- ── Después ─────────────────────────────────────────────────────
           Se dice QUÉ se borró y QUÉ se conservó, con cifras. Quien atiende
           la solicitud tiene que poder responderle eso mismo a una persona,
           y «listo» no le sirve para eso. -->
      <AAviso
        v-else-if="borrado"
        tono="exito"
        urgente
        :titulo="t('derechos.borradoTitulo')"
      >
        <span>{{
          t('derechos.borradoDetalle', {
            borradas: borrado.contactosBorrados + borrado.canalesBorrados
              + borrado.camposBorrados + borrado.pertenenciasBorradas,
            conservados: borrado.enviosAnonimizados + borrado.audienciasAnonimizadas,
          })
        }}</span>
        <span>{{
          borrado.laBajaYaExistia ? t('derechos.borradoBajaYaEstaba') : t('derechos.borradoBajaCreada')
        }}</span>
      </AAviso>
    </AModulo>
  </div>
</template>

<style scoped>
.derechos {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.derechos__explicacion {
  margin: 0 0 var(--arles-space-4);
  max-width: 62ch;
  color: var(--arles-text-muted);
}

.derechos__formulario {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  /* El formulario no se estira con la ventana: son dos campos cortos, y un
     campo de 1 100 px de ancho para escribir un correo invita a creer que
     cabe algo más largo de lo que cabe. */
  max-width: 34rem;
  margin: var(--arles-space-5) 0;
}

.derechos__falta {
  margin: var(--arles-space-2) 0 0;
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

.derechos__acciones {
  display: flex;
  gap: var(--arles-space-3);
}

.derechos__pendiente {
  margin-top: var(--arles-space-6);
  padding-top: var(--arles-space-5);
  border-top: var(--arles-border-width) solid var(--arles-border);
  max-width: 62ch;
}

.derechos__subtitulo {
  margin: 0 0 var(--arles-space-3);
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
  font-weight: var(--arles-font-weight-semibold);
}

.derechos__lista {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
}

.derechos__lista dt {
  font-weight: var(--arles-font-weight-semibold);
}

.derechos__lista dd {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

/* ── El paso de «esto es lo que va a pasar» ────────────────────────────────
   Se separa del formulario con un filete y un fondo propio porque es **otro
   momento**: arriba se busca, aquí se decide. Sin esa separación, el botón
   rojo queda a un palmo del campo de texto y se pulsa por inercia. */
.borrar {
  margin-top: var(--arles-space-5);
  padding: var(--arles-space-5);
  border: var(--arles-border-width) solid var(--arles-border-strong);
  border-radius: var(--arles-radius-md);
  background: var(--arles-surface-raised);
  max-width: 34rem;
}

.borrar__quien {
  margin: 0 0 var(--arles-space-4);
  font-size: var(--arles-font-size-body);
}

/* Dos columnas: lo que se va y lo que se queda, una al lado de la otra.
   Enseñarlas en una lista vertical haría leer la segunda como una nota al
   pie, y no lo es: es la mitad del trato. */
.borrar__cifras {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--arles-space-4);
  margin: 0 0 var(--arles-space-4);
}

.borrar__cifra dt {
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
  font-weight: var(--arles-font-weight-semibold);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--arles-text-muted);
}

.borrar__cifra dd {
  margin: var(--arles-space-1) 0 0;
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  color: var(--arles-text-muted);
}

/* La cifra manda. Es lo único que alguien mira antes de decidir. */
.borrar__numero {
  display: block;
  font-size: var(--arles-font-size-h2);
  line-height: var(--arles-line-height-h2);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
  font-variant-numeric: tabular-nums;
}

/* Lo que desaparece lleva el color de peligro; lo que se queda, no. Es la
   diferencia que hay que entender de un vistazo, y el §19 exige que no
   dependa sólo del color: por eso cada columna lleva además su título en
   palabras —«Desaparece» y «Se conserva»—. */
.borrar__cifra--va .borrar__numero {
  color: var(--arles-danger);
}

.borrar__baja {
  margin: 0;
  padding: var(--arles-space-3) 0 0;
  border-top: var(--arles-border-width) solid var(--arles-border);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
  color: var(--arles-text-muted);
}

.borrar__confirmar {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  margin-top: var(--arles-space-5);
  padding-top: var(--arles-space-5);
  border-top: var(--arles-border-width) solid var(--arles-border);
}

@media (max-width: 40rem) {
  /* En estrecho, dos columnas de cifras las parten en tres líneas cada una y
     dejan de leerse como un par. Apiladas siguen siendo dos bloques. */
  .borrar__cifras {
    grid-template-columns: 1fr;
    gap: var(--arles-space-3);
  }
}
</style>
