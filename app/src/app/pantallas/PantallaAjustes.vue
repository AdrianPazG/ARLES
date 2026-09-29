<script setup lang="ts">
/**
 * Ajustes · **de la aplicación**, no del negocio.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE QUITÓ DE AQUÍ, Y POR QUÉ · 21/09/2026
 *
 * Esta pantalla llevaba tres cosas: el formulario de la empresa, la apariencia
 * y la lista de los seis pasos del alta. Dirección lo señaló mirándola:
 * «Ajustes es sólo para los ajustes del sistema o de la aplicación».
 *
 * Tenía razón, y el error era mío. El 17/09 moví la lista aquí razonando que
 * «Ajustes es donde se configura». Esa frase junta dos cosas distintas:
 *
 *   - **Configurar la aplicación** —tema, densidad, versión, respaldos—: una
 *     preferencia de la herramienta, que es lo que queda aquí.
 *   - **Configurar el negocio** —nombre, país, zona horaria—: datos que
 *     deciden a qué hora sale cada correo y qué marco legal aplica. No son
 *     una preferencia.
 *
 * Y los seis pasos no son ninguna de las dos: son una guía **con final**, que
 * desaparece al completarse. Un ajuste no desaparece nunca.
 *
 * Las dos cosas se fueron a `/inicio/configuracion`.
 *
 * AQUÍ TAMPOCO QUEDA UN ENLACE, y eso también lo decidió Dirección · 21/09/2026
 *
 * El primer intento dejó un bloque «Los datos de tu negocio» con un botón que
 * llevaba a la configuración. Era un puente, y existía sólo porque no había
 * otro camino: la subpágina no salía en ninguna parte de la barra lateral.
 *
 * En cuanto la barra enseña «Inicio › Configuración», ese puente deja de ser
 * un camino y pasa a ser lo que Dirección señaló: Ajustes volviendo a hablar
 * del negocio. Se quitó entero, botón y texto.
 *
 * Es decir: la forma correcta de que una pantalla no necesite un atajo no es
 * escribirle el atajo, es que el sitio al que lleva se pueda encontrar solo.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { TEMAS, esTema, useInterfazStore } from '@/app/stores/interfaz'
import { AAviso, ASelector } from '@/design/componentes'

const interfaz = useInterfazStore()
const { t } = useI18n()

/**
 * Las opciones del tema salen de la lista del almacén, no de tres líneas
 * escritas aquí. Esa lista es la que el núcleo valida, así que una opción que
 * el desplegable ofreciera de más sería una opción que el núcleo rechaza.
 */
const opcionesDeTema = computed(() =>
  TEMAS.map((valor) => ({ valor, texto: t(`apariencia.${valor}`) })),
)

/**
 * `ASelector` emite una cadena, y `elegirTema` quiere una de las tres palabras.
 * Comprobarlo aquí es lo que permite que el almacén siga tipado sin un `as`.
 */
function elegirTema(valor: string): void {
  if (esTema(valor)) void interfaz.elegirTema(valor)
}
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ $t('nav.ajustes') }}
      </h1>
      <p class="entradilla">
        {{ $t('ajustes.entradilla') }}
      </p>
    </header>

    <!-- ── Apariencia (C-1 y C-2) ──────────────────────────────────────────
         No tiene botón de guardar: se aplica al elegir, porque el resultado se
         ve entero en la misma pantalla. Un «Guardar» aquí obligaría a
         confirmar algo que ya está a la vista. -->
    <section
      class="bloque apariencia"
      aria-labelledby="titulo-apariencia"
    >
      <h2
        id="titulo-apariencia"
        class="subtitulo"
      >
        {{ $t('apariencia.titulo') }}
      </h2>

      <ASelector
        :model-value="interfaz.tema"
        :etiqueta="$t('apariencia.campo')"
        :ayuda="$t('apariencia.ayuda')"
        :opciones="opcionesDeTema"
        @update:model-value="elegirTema"
      />

      <AAviso
        tono="info"
        :titulo="$t('apariencia.porQue')"
      >
        {{ $t('apariencia.porQueDetalle') }}
      </AAviso>
    </section>
  </section>
</template>

<style scoped>
/* ── EL ANCHO SE LE PONE AL BLOQUE, NO A LA PANTALLA ──────────────────────
   Esto era `max-width: 62ch` sobre `.pantalla`, y `.pantalla` lleva
   `margin-inline: auto` del armazón (base.css). Las dos reglas juntas dejaban
   Ajustes **centrada en mitad del área de contenido**: 540 px de columna con
   298 px de aire a cada lado, mientras las otras doce pantallas arrancan
   pegadas al mismo borde izquierdo. Al pasar de Actividad a Ajustes el título
   daba un salto de 300 px.
   Se vio el 29/09/2026 mirando las pantallas una tras otra en Chromium; cada
   una por separado se ve bien, que es por lo que no había salido antes.
   La medida de lectura sigue existiendo: va sobre el bloque, que es lo que
   tiene texto. */
.pantalla {
  /* La medida se DECLARA aquí y se APLICA abajo.
     Es lo que lee la sonda de plegado para saber por debajo de qué ancho esta
     pantalla deja de funcionar, y sólo la encuentra en `.pantalla`. Al llevarla
     al bloque junto con el `max-width` —el 29/09/2026, arreglando lo de
     arriba— la sonda se quedó sin contrato que medir y lo dijo. */
  --arles-medida: 540px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

header,
.bloque {
  max-width: 62ch;
}

.bloque {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
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
</style>
