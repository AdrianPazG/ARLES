<script setup lang="ts">
/**
 * Respaldos · la única salida si se pierde todo.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ CUELGA DE AJUSTES Y NO ES UNA SÉPTIMA SECCIÓN
 *
 * El producto son seis secciones (P-13), y la cabecera de `PantallaAjustes`
 * ya decía qué pertenece aquí: «tema, densidad, versión, **respaldos**». Un
 * respaldo es una propiedad de la herramienta, no del negocio: no decide a
 * quién se le escribe ni a qué hora.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * PRIMERO SE DICE CON CIFRAS QUÉ SE VA A LLEVAR, Y DESPUÉS SE PULSA
 *
 * Es el patrón del recorte del registro (7.4) y del derecho de cancelación
 * (3.4). Aquí importa por un motivo propio: los dos alcances de D-8B se
 * eligen a ciegas si no se ve la diferencia. «Lo esencial» y «Todo» son dos
 * palabras; «12 483 filas» y «48 207 filas» son una decisión (§94).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LA CONTRASEÑA SE PIDE DOS VECES, Y EL AVISO NO ES DECORATIVO
 *
 * No se guarda en ningún sitio: ni en el llavero, ni en la base, ni vuelve por
 * la frontera (D-8A). Si se olvida, **el respaldo no se abre nunca** — y el
 * respaldo es justo lo que se usa cuando ya se perdió algo. De ahí las dos
 * casillas y el aviso con todas las letras.
 *
 * Y se puede dejar en blanco a propósito, que es la mitad de D-8A que Dirección
 * pidió conservar. Cuando está en blanco, el aviso cambia y dice qué significa:
 * un archivo con todos los contactos que cualquiera puede abrir.
 */
import { computed, onMounted } from 'vue'

import { ALCANCES, type Alcance, useRespaldoStore } from '@/app/stores/respaldo'
import { AAviso, ABoton, AEntrada, ASelector } from '@/design/componentes'

const respaldo = useRespaldoStore()

onMounted(() => {
  void respaldo.verQueLlevaria()
})

/**
 * Las opciones salen de la lista del almacén, no de dos líneas escritas aquí:
 * una opción que el desplegable ofreciera de más es una que el núcleo rechaza
 * con `error.app.alcance_de_respaldo_desconocido`.
 */
const opcionesDeAlcance = computed(() =>
  ALCANCES.map((valor) => ({ valor, texto: `respaldo.alcance.${valor}` })),
)

function elegir(valor: string): void {
  if ((ALCANCES as readonly string[]).includes(valor)) {
    void respaldo.elegirAlcance(valor as Alcance)
  }
}

/**
 * Las tablas con al menos una fila, de más a menos.
 *
 * Las vacías se dejan fuera: veintitrés renglones con un cero al lado no
 * informan de nada y esconden los tres que importan.
 */
/**
 * Las cifras como las escribe el resto del producto.
 *
 * ── «12536 FILAS» ES UN DEFECTO, Y SÓLO SE VE MIRANDO ──
 *
 * Ninguna comprobación lo caza: es un número correcto. Pero Actividad, el
 * registro y el catálogo escriben `toLocaleString('es-MX')` desde la 7.2, así
 * que esta pantalla decía las cifras de otra forma que las demás — y un número
 * de cinco dígitos sin separador se lee dos veces. §94 pide cifras; que se
 * puedan leer es parte de pedirlas.
 */
function cifra(n: number): string {
  return n.toLocaleString('es-MX')
}

const conFilas = computed(() =>
  [...(respaldo.previo?.tablas ?? [])].filter((t) => t.filas > 0).sort((a, b) => b.filas - a.filas),
)
</script>

<template>
  <section class="pantalla">
    <header>
      <h1 class="titulo">
        {{ $t('nav.sub.respaldos') }}
      </h1>
      <p class="entradilla">
        {{ $t('respaldo.entradilla') }}
      </p>
    </header>

    <!-- ── Qué se lleva (D-8B) ──────────────────────────────────────────── -->
    <section
      class="bloque"
      aria-labelledby="titulo-alcance"
    >
      <h2
        id="titulo-alcance"
        class="subtitulo"
      >
        {{ $t('respaldo.queSeLleva') }}
      </h2>

      <ASelector
        :model-value="respaldo.alcance"
        :etiqueta="$t('respaldo.alcance.campo')"
        :ayuda="$t('respaldo.alcance.ayuda', { meses: respaldo.previo?.mesesEnElEsencial ?? 12 })"
        :opciones="opcionesDeAlcance.map((o) => ({ valor: o.valor, texto: $t(o.texto) }))"
        :deshabilitado="respaldo.escribiendo"
        @update:model-value="elegir"
      />

      <!-- Las cifras. Sin núcleo detrás —la vista previa— no hay nada que
           enseñar, y se dice en vez de pintar un cero que parecería un dato. -->
      <p
        v-if="respaldo.previo"
        class="cifra"
      >
        {{ $t('respaldo.seLlevaria', { filas: cifra(respaldo.previo.total), tablas: conFilas.length }) }}
      </p>
      <p
        v-else-if="respaldo.cargando"
        class="entradilla"
      >
        {{ $t('respaldo.contando') }}
      </p>
      <p
        v-else
        class="entradilla"
      >
        {{ $t('respaldo.sinNucleo') }}
      </p>

      <details v-if="conFilas.length > 0">
        <summary>{{ $t('respaldo.verElDetalle') }}</summary>
        <ul class="detalle">
          <li
            v-for="t in conFilas"
            :key="t.tabla"
          >
            <span class="tabla">{{ t.tabla }}</span>
            <span class="filas">{{ cifra(t.filas) }}</span>
          </li>
        </ul>
      </details>
    </section>

    <!-- ── La contraseña (D-8A) ─────────────────────────────────────────── -->
    <section
      class="bloque"
      aria-labelledby="titulo-contrasena"
    >
      <h2
        id="titulo-contrasena"
        class="subtitulo"
      >
        {{ $t('respaldo.proteccion') }}
      </h2>

      <AEntrada
        v-model="respaldo.contrasena"
        tipo="password"
        :etiqueta="$t('respaldo.contrasena')"
        :ayuda="$t('respaldo.contrasenaAyuda')"
        :deshabilitado="respaldo.escribiendo"
      />
      <!-- `exactOptionalPropertyTypes` no admite pasar `undefined` a una prop
           opcional, así que se pasa la cadena vacía: `AEntrada` ya trata «sin
           error» como ausencia. Un `as string` aquí sería una afirmación que
           nadie ha comprobado. -->
      <AEntrada
        v-model="respaldo.repetida"
        tipo="password"
        :etiqueta="$t('respaldo.repetida')"
        :ayuda="respaldo.contrasena === '' ? $t('respaldo.repetidaBloqueada') : ''"
        :error="respaldo.noCuadran ? $t('respaldo.noCuadran') : ''"
        :deshabilitado="respaldo.escribiendo || respaldo.contrasena === ''"
      />

      <AAviso
        :tono="respaldo.cifraria ? 'info' : 'aviso'"
        :titulo="
          respaldo.cifraria ? $t('respaldo.avisoCifrado') : $t('respaldo.avisoSinCifrar')
        "
      >
        {{
          respaldo.cifraria
            ? $t('respaldo.avisoCifradoDetalle')
            : $t('respaldo.avisoSinCifrarDetalle')
        }}
      </AAviso>
    </section>

    <!-- ── Escribir ─────────────────────────────────────────────────────── -->
    <section class="bloque acciones">
      <ABoton
        variante="primario"
        :deshabilitado="!respaldo.sePuedeEscribir"
        @click="respaldo.escribir()"
      >
        {{ respaldo.escribiendo ? $t('respaldo.escribiendo') : $t('respaldo.crear') }}
      </ABoton>

      <!-- Cifras, no una rueda (§94). Mientras el núcleo avisa, se enseña por
           dónde va; si no llega ningún aviso —la vista previa no los emite—,
           se enseña cuántas filas va a escribir, que es un dato real. -->
      <p
        v-if="respaldo.escribiendo"
        class="cifra"
        role="status"
      >
        {{
          respaldo.filasHechas > 0
            ? $t('respaldo.avance', {
              filas: respaldo.filasHechas,
              total: respaldo.previo?.total ?? respaldo.filasHechas,
            })
            : $t('respaldo.arrancando', { filas: respaldo.previo?.total ?? 0 })
        }}
      </p>

      <AAviso
        v-if="respaldo.escrito"
        tono="exito"
        :titulo="$t('respaldo.hecho')"
      >
        {{
          $t('respaldo.hechoDetalle', {
            archivo: respaldo.escrito.nombreDelArchivo,
            filas: cifra(respaldo.escrito.filas),
          })
        }}
      </AAviso>

      <AAviso
        v-if="respaldo.errorGeneral"
        tono="peligro"
        :titulo="$t(`${respaldo.errorGeneral}.que`)"
      >
        {{ $t(`${respaldo.errorGeneral}.como`) }}
      </AAviso>
    </section>
  </section>
</template>

<style scoped>
.pantalla {
  /* La medida va aquí porque es lo que lee la sonda de plegado; el `max-width`
     va en los bloques. Las dos juntas sobre `.pantalla` dejaban Ajustes
     centrada y 300 px a la derecha de las demás — el defecto del 29/09/2026. */
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

.cifra {
  margin: 0;
  font-weight: var(--arles-font-weight-semibold);
}

summary {
  cursor: pointer;
  color: var(--arles-text-muted);
}

.detalle {
  margin: var(--arles-space-3) 0 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
}

.detalle li {
  display: flex;
  justify-content: space-between;
  gap: var(--arles-space-4);
  color: var(--arles-text-muted);
}

/* ── AQUÍ HABÍA UN `--arles-font-family-mono` QUE NO EXISTE ──────────────
   Los nombres de tabla son identificadores técnicos y pedían monoespaciada.
   El token no está en el sistema, y el navegador resuelve un `var()`
   inexistente **a nada, sin error**: la lista se habría visto bien y el
   defecto habría pasado. Lo cazó `todo token que se usa está definido` en la
   fase 2 — el mismo caso que los seis tokens inventados de la 5.2.
   Añadir un token de fuente es una decisión del sistema de diseño, no algo
   que se cuela en una entrega de respaldos. Los nombres van en la fuente de
   siempre. */

.filas {
  font-variant-numeric: tabular-nums;
}

.acciones {
  align-items: flex-start;
}
</style>
