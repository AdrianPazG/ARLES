<script setup lang="ts">
/**
 * Archivar el historial · sacar los envíos viejos a un disco y quitarlos de
 * aquí (D-8D, entrega 8.3 bis b).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * PRIMERO SE DICE QUÉ VA A PASAR, Y LUEGO SE PULSA
 *
 * El patrón del recorte del registro (7.4), del respaldo (8.2c) y de restaurar
 * (8.3c). Aquí hace falta por dos motivos que no son evidentes mirando un
 * botón:
 *
 * 1. **El periodo que se archiva no es el que se pide.** Alguien que elige
 *    «dejar 12 meses» no sabe si hay algo antes de eso. La pantalla enseña el
 *    periodo **real** de lo que se va — de cuándo a cuándo — y no la fecha de
 *    corte, que no dice nada sola.
 * 2. **Borrar envíos se lleva sucesos del registro de actividad**, porque
 *    cuelgan de ellos en la base, y el archivo **no** los guarda. Eso es una
 *    pérdida de verdad y va en un aviso de peligro, con su cifra.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA TIENE QUE DECIR Y NO SE PUEDE SUAVIZAR
 *
 * Que el archivo pasa a ser **responsabilidad de quien lo guarda**. Hoy ese
 * riesgo no existe porque el historial nunca sale de la base; después de
 * archivar, contestar una solicitud de datos de ese periodo necesita ese
 * archivo. Se cambia espacio en disco por una obligación de custodia, y la ley
 * no acepta «lo tenía en un disco que se rompió».
 *
 * Y que el archivo va **en claro**: se abre años después, puede que en otro
 * equipo, y una contraseña que nadie recuerda lo haría igual a no tenerlo.
 */
import { MESES_QUE_SE_OFRECEN, usarArchivado } from '@/app/stores/archivado'
import { AAviso, ABoton, ASelector } from '@/design/componentes'

const a = usarArchivado()

/** Las cifras como las escribe el resto del producto, desde la 7.2. */
function cifra(n: number): string {
  return n.toLocaleString('es-MX')
}

/** Una fecha legible. Si no se puede leer, se enseña tal cual. */
function fecha(iso: string): string {
  const d = new Date(iso)
  return Number.isNaN(d.getTime())
    ? iso
    : d.toLocaleDateString('es-MX', { dateStyle: 'long' })
}

/** Los megabytes de un archivo, con un decimal. */
function enMegas(bytes: number): string {
  return (bytes / 1024 / 1024).toLocaleString('es-MX', {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  })
}
</script>

<template>
  <div class="pantalla">
    <!-- ── EL TÍTULO Y LA ENTRADILLA VAN JUNTOS, Y NO ES DECORATIVO ──────
         `.pantalla` es un flex con `gap: space-6` (48 px). Sueltos, el título
         y su entradilla son DOS hijos del flex, así que ese hueco se mete
         entre ellos: medido el 07/10/2026, 54 px aquí contra los 8 de
         Respaldos, que sí los envuelve. Cada pantalla, sola, se ve bien; el
         defecto sólo existe al pasar de una a otra — es el mismo caso que
         «Ajustes empezaba 300 px más a la derecha» de la Fase 3. -->
    <header>
      <h1 class="titulo">
        {{ $t('archivar.titulo') }}
      </h1>
      <p class="entradilla">
        {{ $t('archivar.entradilla') }}
      </p>
    </header>

    <!-- ── 1 · Cuánto se deja ────────────────────────────────────────────── -->
    <section
      class="bloque"
      aria-labelledby="titulo-periodo"
    >
      <h2
        id="titulo-periodo"
        class="subtitulo"
      >
        {{ $t('archivar.periodo') }}
      </h2>

      <ASelector
        :etiqueta="$t('archivar.dejarEnLaBase')"
        :ayuda="$t('archivar.dejarEnLaBaseAyuda')"
        :opciones="
          MESES_QUE_SE_OFRECEN.map((m) => ({
            valor: String(m),
            texto: $t('archivar.meses', { meses: m }, m),
          }))
        "
        :model-value="String(a.meses)"
        :deshabilitado="a.revisando || a.archivando"
        @update:model-value="a.elegirMeses(Number($event))"
      />

      <ABoton
        class="boton-suelto"
        variante="secundario"
        :cargando="a.revisando"
        :deshabilitado="a.archivando"
        @click="a.revisar()"
      >
        {{ $t('archivar.verQuePasaria') }}
      </ABoton>
    </section>

    <!-- ── 2 · Qué va a pasar ────────────────────────────────────────────── -->
    <section
      v-if="a.revisado"
      class="bloque"
      aria-labelledby="titulo-preflight"
    >
      <h2
        id="titulo-preflight"
        class="subtitulo"
      >
        {{ $t('archivar.queVaAPasar') }}
      </h2>

      <!-- Nada que archivar: se dice y no se ofrece un botón que escribiría
           un archivo vacío. -->
      <AAviso
        v-if="!a.hayAlgoQueArchivar"
        tono="info"
        :titulo="$t('archivar.nada.titulo')"
      >
        {{ $t('archivar.nada.cuerpo', { envios: cifra(a.revisado.enviosQueSeQuedan) }) }}
      </AAviso>

      <template v-else>
        <dl class="ficha">
          <dt>{{ $t('archivar.seVan') }}</dt>
          <dd>{{ $t('archivar.seVanEnvios', { envios: cifra(a.revisado.enviosQueSeArchivan) }) }}</dd>
          <dt>{{ $t('archivar.deCuando') }}</dt>
          <dd>
            {{
              a.revisado.primero && a.revisado.ultimo
                ? $t('archivar.deAA', {
                  desde: fecha(a.revisado.primero),
                  hasta: fecha(a.revisado.ultimo),
                })
                : '—'
            }}
          </dd>
          <dt>{{ $t('archivar.seQuedan') }}</dt>
          <dd>{{ $t('archivar.seQuedanEnvios', { envios: cifra(a.revisado.enviosQueSeQuedan) }) }}</dd>
          <dt>{{ $t('archivar.campanas') }}</dt>
          <dd>
            {{ $t('archivar.campanasQueViajan', { campanas: a.revisado.campanas }, a.revisado.campanas) }}
          </dd>
        </dl>

        <!-- Lo que la cascada se lleva y el archivo NO guarda. Va antes que
             nada más: es lo único de esta pantalla que no se recupera. -->
        <AAviso
          v-if="a.revisado.totalArrastrado > 0"
          tono="peligro"
          :titulo="$t('archivar.arrastra.titulo')"
        >
          <p>
            {{ $t('archivar.arrastra.cuerpo', { filas: cifra(a.revisado.totalArrastrado) }) }}
          </p>
          <ul class="perdidas">
            <li
              v-for="p in a.revisado.arrastradas"
              :key="p.tabla"
            >
              {{ $t(`restaurar.tabla.${p.tabla}`) }} — {{ cifra(p.filas) }}
            </li>
          </ul>
        </AAviso>

        <!-- La custodia. Es la frase que D-8D pide con esas palabras. -->
        <AAviso
          tono="aviso"
          :titulo="$t('archivar.custodia.titulo')"
        >
          <p>{{ $t('archivar.custodia.cuerpo') }}</p>
          <p>{{ $t('archivar.custodia.enClaro') }}</p>
        </AAviso>

        <AAviso
          tono="info"
          :titulo="$t('archivar.comoLoHace.titulo')"
        >
          {{ $t('archivar.comoLoHace.cuerpo') }}
        </AAviso>

        <div class="acciones">
          <ABoton
            variante="peligro"
            :deshabilitado="!a.sePuedeArchivar"
            :cargando="a.archivando"
            @click="a.archivar()"
          >
            {{ $t('archivar.hazlo') }}
          </ABoton>
          <ABoton
            variante="sutil"
            :deshabilitado="a.archivando"
            @click="a.descartar()"
          >
            {{ $t('archivar.mejorNo') }}
          </ABoton>
        </div>
      </template>
    </section>

    <!-- ── 3 · Lo que pasó ───────────────────────────────────────────────── -->
    <AAviso
      v-if="a.hecho"
      tono="exito"
      :titulo="$t('archivar.hecho.titulo')"
    >
      <p>
        {{
          $t('archivar.hecho.cuerpo', {
            archivo: a.hecho.nombreDelArchivo,
            megas: enMegas(a.hecho.bytes),
            archivados: cifra(a.hecho.enviosArchivados),
            borrados: cifra(a.hecho.enviosBorrados),
          })
        }}
      </p>
      <p class="guarda">
        {{ $t('archivar.hecho.guardalo') }}
      </p>
    </AAviso>

    <AAviso
      v-if="a.error"
      tono="peligro"
      :titulo="$t(`${a.error}.que`)"
    >
      <p>{{ $t(`${a.error}.como`) }}</p>
      <p class="salvo">
        {{ $t(`${a.error}.salvo`) }}
      </p>
    </AAviso>
  </div>
</template>

<style scoped>
.pantalla {
  /* La medida va aquí y el `max-width` en los bloques, no las dos sobre
     `.pantalla`: eso dejó Ajustes centrada y 300 px a la derecha de las demás
     el 29/09/2026. */
  --arles-medida: 540px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

header {
  max-width: 62ch;
}

.titulo {
  /* `margin: 0` arriba y el hueco justo abajo. Sin esto el navegador mete su
     propio margen en el `h1` y en el `<p>`, y el título arranca 19 px más
     abajo que en Respaldos con 14 px de separación en vez de 8. */
  margin: 0 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
}

.entradilla {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-body);
  max-width: 62ch;
}

.bloque {
  max-width: 62ch;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.subtitulo {
  font-size: var(--arles-font-size-h3);
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.ficha {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: var(--arles-space-2) var(--arles-space-4);
  font-size: var(--arles-font-size-small);
}

.ficha dt {
  color: var(--arles-text-muted);
}

.ficha dd {
  color: var(--arles-text);
  font-weight: var(--arles-font-weight-medium);
}

.perdidas {
  margin: var(--arles-space-2) 0 0;
  padding-left: var(--arles-space-5);
}

.boton-suelto {
  align-self: flex-start;
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
}

.guarda,
.salvo {
  margin-top: var(--arles-space-2);
  font-weight: var(--arles-font-weight-medium);
}
</style>
