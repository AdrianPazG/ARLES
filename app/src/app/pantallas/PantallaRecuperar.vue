<script setup lang="ts">
/**
 * Recuperar · la pantalla que no existía, y el día peor para que faltara.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ NO TIENE BARRA LATERAL
 *
 * Porque no hay a dónde ir. En este modo no hay base abierta: ni contactos, ni
 * campañas, ni ajustes. Enseñar las seis secciones sería ofrecer seis puertas
 * pintadas a alguien que acaba de perder el acceso a sus datos.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LAS TRES PARTES DEL §95, Y LA QUE IMPORTA ES LA TERCERA
 *
 * Qué pasó: la clave que cifraba tu base ya no está en este equipo.
 * Cómo se arregla: restaurando un respaldo.
 * **Qué está a salvo**: el archivo no se ha borrado. Sigue ahí, cifrado, y se
 * dice cuánto ocupa — un número que se puede comprobar, no una promesa.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * Y SI NO HAY RESPALDO, SE DICE
 *
 * Sin respaldo no hay salida, con esas palabras. Un botón que no puede
 * funcionar es peor que ninguno: hace perder media hora a quien ya está
 * teniendo un mal día.
 */
import { onMounted } from 'vue'

import { usarRecuperacion } from '@/app/stores/recuperacion'
import { AAviso, ABoton, AEntrada, ALogotipo } from '@/design/componentes'

const r = usarRecuperacion()

/**
 * ── POR QUÉ ESTA LÍNEA NO ES OPCIONAL ──
 *
 * Sin ella, `quePaso` se queda en `null` y la pantalla cae en el texto del
 * caso «no hay base»: le dice a alguien que **no ha perdido nada** justo
 * cuando acaba de perder el acceso a sus datos. Medido el 06/10/2026 en la
 * demostración, con 48 MB de base en disco y ese mensaje en pantalla.
 *
 * Es el patrón del §9 una vez más —el comando está, el almacén lo llama, y
 * nadie llamaba al almacén— y aquí la consecuencia no es una función que
 * falta: es un usuario que borra el archivo porque le dijimos que estaba
 * vacío.
 */
onMounted(() => {
  void r.cargar()
})

function cifra(n: number): string {
  return n.toLocaleString('es-MX')
}

/** Los bytes de la base, en una unidad que se lea. */
function tamano(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  if (bytes >= 1024) return `${Math.round(bytes / 1024)} KB`
  return `${bytes} bytes`
}

function fecha(iso: string): string {
  const d = new Date(iso)
  return Number.isNaN(d.getTime())
    ? iso
    : d.toLocaleString('es-MX', { dateStyle: 'long', timeStyle: 'short' })
}
</script>

<template>
  <!-- ── POR QUÉ ESTO ES UN `<main>` Y NO UN `<div>` ──────────────────────
       Es la única pantalla que no pasa por el armazón de `App.vue`, que es
       quien pone el `<main>` de las otras diecisiete. Con un `div` esta
       pantalla se quedaba **sin ningún landmark**: quien usa un lector de
       pantalla no tiene «ir al contenido principal» justo el día en que algo
       se ha roto y hay que leer con atención.

       Y tenía una segunda consecuencia, que es la que lo encontró: dos de las
       siete medidas de `sonda:auditoria` se acotan a `main` —el contraste y
       la jerarquía de encabezados—, así que aquí **medían cero elementos**
       mientras la sonda contaba esta pantalla como medida. Auditoría D del
       §140, 07/10/2026. -->
  <main class="recuperar">
    <!-- La única pantalla sin barra lateral, así que la marca va aquí: sin
         ella, una ventana suelta con un aviso rojo no dice ni de qué programa
         es. El logotipo va en su componente porque el §21 prohíbe
         estilizarlo. -->
    <ALogotipo class="marca" />

    <h1 class="titulo">
      {{ $t('recuperar.titulo') }}
    </h1>

    <!-- ── 1 · Qué pasó, y qué está a salvo ──────────────────────────────── -->
    <AAviso
      tono="peligro"
      :titulo="$t('recuperar.quePaso.titulo')"
    >
      <p>{{ $t('recuperar.quePaso.cuerpo') }}</p>
      <p
        v-if="r.quePaso && r.quePaso.bytesDeLaBase > 0"
        class="salvo"
      >
        {{
          $t('recuperar.quePaso.aSalvo', {
            archivo: r.quePaso.nombreDeLaBase,
            tamano: tamano(r.quePaso.bytesDeLaBase),
          })
        }}
      </p>
      <p
        v-else
        class="salvo"
      >
        {{ $t('recuperar.quePaso.sinBase') }}
      </p>
    </AAviso>

    <!-- ── 2 · La única salida ───────────────────────────────────────────── -->
    <section
      v-if="!r.hecho"
      class="bloque"
      aria-labelledby="titulo-salida"
    >
      <h2
        id="titulo-salida"
        class="subtitulo"
      >
        {{ $t('recuperar.salida.titulo') }}
      </h2>
      <p class="parrafo">
        {{ $t('recuperar.salida.cuerpo') }}
      </p>

      <!-- Sin respaldo no hay salida, y se dice con esas palabras. -->
      <AAviso
        tono="aviso"
        :titulo="$t('recuperar.sinRespaldo.titulo')"
      >
        {{ $t('recuperar.sinRespaldo.cuerpo') }}
      </AAviso>

      <ABoton
        class="boton-suelto"
        variante="secundario"
        :cargando="r.revisando"
        @click="r.revisar()"
      >
        {{ $t('recuperar.elegirArchivo') }}
      </ABoton>
    </section>

    <!-- ── 3 · Qué trae el respaldo ──────────────────────────────────────── -->
    <section
      v-if="r.revisado && !r.hecho"
      class="bloque"
      aria-labelledby="titulo-trae"
    >
      <h2
        id="titulo-trae"
        class="subtitulo"
      >
        {{ $t('recuperar.trae.titulo') }}
      </h2>

      <dl class="ficha">
        <dt>{{ $t('restaurar.archivo') }}</dt>
        <dd>{{ r.revisado.nombreDelArchivo }}</dd>
        <dt>{{ $t('restaurar.hechoEl') }}</dt>
        <dd>{{ fecha(r.revisado.creadoEn) }}</dd>
        <dt>{{ $t('restaurar.alcance') }}</dt>
        <dd>{{ $t(`respaldo.alcance.${r.revisado.alcance}`) }}</dd>
        <dt>{{ $t('restaurar.trae') }}</dt>
        <dd>{{ $t('restaurar.traeFilas', { filas: cifra(r.revisado.totalDeFilas) }) }}</dd>
      </dl>

      <AAviso
        v-if="!r.revisado.sePuedeRestaurar"
        tono="peligro"
        :titulo="$t('restaurar.otroEsquema.titulo')"
      >
        {{
          $t('restaurar.otroEsquema.cuerpo', {
            delRespaldo: r.revisado.esquemaDelRespaldo,
            deLaBase: r.revisado.esquemaDeLaBase,
          })
        }}
      </AAviso>

      <template v-else>
        <AAviso
          tono="info"
          :titulo="$t('recuperar.loQueVaAPasar.titulo')"
        >
          {{ $t('recuperar.loQueVaAPasar.cuerpo') }}
        </AAviso>

        <AEntrada
          v-if="r.revisado.llevaContrasena"
          v-model="r.contrasena"
          tipo="password"
          :etiqueta="$t('restaurar.contrasena')"
          :ayuda="$t('restaurar.contrasenaAyuda')"
          :deshabilitado="r.recuperando"
        />
        <p
          v-if="r.faltaLaContrasena"
          class="por-que-no"
        >
          {{ $t('restaurar.faltaLaContrasena') }}
        </p>

        <ABoton
          class="boton-suelto"
          variante="peligro"
          :deshabilitado="!r.sePuedeRecuperar"
          :cargando="r.recuperando"
          @click="r.recuperar()"
        >
          {{ $t('recuperar.hazlo') }}
        </ABoton>
      </template>
    </section>

    <!-- ── 4 · Lo que pasó ───────────────────────────────────────────────── -->
    <AAviso
      v-if="r.hecho"
      tono="exito"
      :titulo="$t('recuperar.hecho.titulo')"
    >
      <p>
        {{
          $t('recuperar.hecho.cuerpo', {
            filas: cifra(r.hecho.filas),
            bajas: cifra(r.hecho.bajas),
          })
        }}
      </p>
      <p v-if="r.hecho.nombreDeLaApartada !== ''">
        {{ $t('recuperar.hecho.apartada', { archivo: r.hecho.nombreDeLaApartada }) }}
      </p>
      <p class="salvo">
        {{ $t('recuperar.hecho.ahora') }}
      </p>
    </AAviso>

    <AAviso
      v-if="r.error"
      tono="peligro"
      :titulo="$t(`${r.error}.que`)"
    >
      <p>{{ $t(`${r.error}.como`) }}</p>
      <p class="salvo">
        {{ $t(`${r.error}.salvo`) }}
      </p>
    </AAviso>
  </main>
</template>

<style scoped>
.recuperar {
  --arles-medida: 540px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-5);
  padding: var(--arles-space-7) var(--arles-space-6);
  max-width: 68ch;
  margin: 0 auto;
}

.marca {
  margin-bottom: var(--arles-space-2);
}

.titulo {
  font-size: var(--arles-font-size-h1);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
}

.subtitulo {
  font-size: var(--arles-font-size-h3);
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.parrafo {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-body);
}

.bloque {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
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

.boton-suelto {
  align-self: flex-start;
}

.por-que-no {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.salvo {
  margin-top: var(--arles-space-2);
  font-weight: var(--arles-font-weight-medium);
}
</style>
