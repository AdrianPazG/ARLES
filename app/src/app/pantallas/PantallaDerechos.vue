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

type Canal = 'email' | 'whatsapp'

const canal = ref<Canal>('email')
const direccion = ref('')
const trabajando = ref(false)

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
</script>

<template>
  <div class="derechos">
    <AModulo
      :titulo="t('derechos.titulo')"
      principal
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
          >
            {{ t('derechos.entregar') }}
          </ABoton>
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
        tono="exito"
        urgente
        :titulo="resultado.titulo"
      >
        {{ resultado.detalle }}
      </AAviso>

      <AAviso
        v-else-if="cancelado"
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
        <h3
          id="titulo-pendiente"
          class="derechos__subtitulo"
        >
          {{ t('derechos.otrosTitulo') }}
        </h3>
        <dl class="derechos__lista">
          <div>
            <dt>{{ t('derechos.rectificacion') }}</dt>
            <dd>{{ t('derechos.rectificacionDetalle') }}</dd>
          </div>
          <div>
            <dt>{{ t('derechos.oposicion') }}</dt>
            <dd>{{ t('derechos.oposicionDetalle') }}</dd>
          </div>
          <div>
            <dt>{{ t('derechos.cancelacion') }}</dt>
            <dd>{{ t('derechos.cancelacionDetalle') }}</dd>
          </div>
        </dl>
      </section>
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
</style>
