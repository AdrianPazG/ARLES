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
import { onMounted, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useEmpresaStore, type BorradorDeEmpresa } from '@/app/stores/empresa'
import { AAviso, ABoton, AEntrada, ASelector, EstadoExito } from '@/design/componentes'

const empresa = useEmpresaStore()
const { t, te } = useI18n()

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

async function enviar(): Promise<void> {
  await empresa.guardar({ ...formulario })
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

    <form
      class="formulario"
      novalidate
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
        :opciones="empresa.configuracion.paises.map((p) => ({ valor: p, texto: p }))"
      />

      <ASelector
        v-model="formulario.zonaHoraria"
        :etiqueta="$t('empresa.campo.zonaHoraria')"
        :ayuda="$t('empresa.ayuda.zonaHoraria')"
        :error="errorDe('zonaHoraria')"
        :opciones="empresa.configuracion.zonas.map((z) => ({ valor: z, texto: z }))"
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

      <AAviso
        v-if="empresa.errorGeneral"
        tono="peligro"
        urgente
        :titulo="$t(`${empresa.errorGeneral}.que`)"
      >
        {{ $t(`${empresa.errorGeneral}.como`) }}
        {{ $t(`${empresa.errorGeneral}.salvo`) }}
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
  </section>
</template>

<style scoped>
.pantalla {
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
