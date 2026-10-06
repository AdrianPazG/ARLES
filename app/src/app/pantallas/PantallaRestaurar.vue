<script setup lang="ts">
/**
 * Restaurar · la acción más destructiva de ARLES, y la única sin deshacer.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * PRIMERO SE DICE QUÉ VA A PASAR, Y LUEGO SE PULSA
 *
 * Es el patrón del recorte del registro (7.4), del derecho de cancelación
 * (3.4) y del respaldo (8.2c). Aquí es más que una costumbre: reemplazar la
 * base no se puede deshacer, y lo que el preflight de la 8.3a descubrió es
 * que **restaurar se lleva cosas que el respaldo no devuelve** — el registro
 * de actividad y el histórico de calidad, si el respaldo es «esencial».
 *
 * Nada de eso es evidente al mirar un archivo. Por eso esta pantalla no tiene
 * un botón de restaurar: tiene uno de **revisar**, y el de restaurar aparece
 * después, debajo de las cifras.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE SE PIERDE Y LO QUE ES DE ESTE EQUIPO VAN SEPARADOS
 *
 * `rate_budget` y `engine_control` también se van por cascada, y perderlos es
 * correcto: describen este equipo, no estos datos. Mezclarlos con el registro
 * de actividad en la misma lista haría leer cuatro pérdidas donde hay dos, y
 * el aviso que exagera es el que se deja de leer.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LA CONTRASEÑA SE PIDE SÓLO SI HACE FALTA
 *
 * La cabecera del archivo va en claro a propósito (8.2c), así que al revisar
 * ya se sabe si el respaldo va cifrado. Ni se pide por si acaso, ni se
 * descubre fallando a mitad.
 */
import { usarRestauracion } from '@/app/stores/restauracion'
import { AAviso, ABoton, AEntrada } from '@/design/componentes'

const r = usarRestauracion()

/** Las cifras como las escribe el resto del producto, desde la 7.2. */
function cifra(n: number): string {
  return n.toLocaleString('es-MX')
}

/** La fecha del respaldo, legible. Si no se puede leer, se enseña tal cual. */
function fecha(iso: string): string {
  const d = new Date(iso)
  return Number.isNaN(d.getTime())
    ? iso
    : d.toLocaleString('es-MX', { dateStyle: 'long', timeStyle: 'short' })
}
</script>

<template>
  <div class="pantalla">
    <h1 class="titulo">
      {{ $t('restaurar.titulo') }}
    </h1>
    <p class="entradilla">
      {{ $t('restaurar.entradilla') }}
    </p>

    <!-- ── 1 · Elegir y revisar ──────────────────────────────────────────── -->
    <section
      class="bloque"
      aria-labelledby="titulo-elegir"
    >
      <h2
        id="titulo-elegir"
        class="subtitulo"
      >
        {{ $t('restaurar.elegir') }}
      </h2>
      <p class="entradilla">
        {{ $t('restaurar.elegirAyuda') }}
      </p>
      <!-- El `align-self` va aquí porque el bloque es una columna flexible y
           los hijos se estiran: medido el 06/10/2026, este botón ocupaba
           540 px y los de abajo 200 y 96. Un botón del ancho de la pantalla
           no es un defecto que cace ninguna sonda, y se ve a un metro. -->
      <ABoton
        class="boton-suelto"
        variante="secundario"
        :cargando="r.revisando"
        @click="r.revisar()"
      >
        {{ $t('restaurar.elegirArchivo') }}
      </ABoton>
    </section>

    <!-- ── 2 · Qué va a pasar ────────────────────────────────────────────── -->
    <section
      v-if="r.revisado"
      class="bloque"
      aria-labelledby="titulo-preflight"
    >
      <h2
        id="titulo-preflight"
        class="subtitulo"
      >
        {{ $t('restaurar.queVaAPasar') }}
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

      <!-- De otro esquema: no se puede, y se dice aquí en vez de dejar pulsar
           un botón que va a fallar. -->
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
        <!-- Lo que NO vuelve. Es el hallazgo de la 8.3a y va antes que nada
             más: es lo único de esta pantalla que no se puede recuperar. -->
        <AAviso
          v-if="r.perdidasDeDatos.length > 0"
          tono="peligro"
          :titulo="$t('restaurar.noVuelve.titulo')"
        >
          <p>
            {{
              $t('restaurar.noVuelve.cuerpo', { filas: cifra(r.revisado.filasQueNoVuelven) })
            }}
          </p>
          <ul class="perdidas">
            <li
              v-for="p in r.perdidasDeDatos"
              :key="p.tabla"
            >
              {{ $t(`restaurar.tabla.${p.tabla}`) }} — {{ cifra(p.filas) }}
            </li>
          </ul>
        </AAviso>

        <!-- Las bajas: la promesa que no se negocia. -->
        <AAviso
          tono="info"
          :titulo="$t('restaurar.bajas.titulo')"
        >
          {{
            $t('restaurar.bajas.cuerpo', {
              ahora: cifra(r.revisado.bajasAhora),
              enElRespaldo: cifra(r.revisado.bajasEnElRespaldo),
              alMenos: cifra(r.revisado.bajasAlMenos),
            })
          }}
        </AAviso>

        <AAviso
          v-if="r.revisado.cuentasAReconectar > 0"
          tono="aviso"
          :titulo="$t('restaurar.cuentas.titulo')"
        >
          {{
            $t(
              'restaurar.cuentas.cuerpo',
              { cuentas: r.revisado.cuentasAReconectar },
              r.revisado.cuentasAReconectar,
            )
          }}
        </AAviso>

        <details>
          <summary>{{ $t('restaurar.verElDetalle') }}</summary>
          <ul class="detalle">
            <li
              v-for="t in r.revisado.tablas.filter((x) => x.ahora > 0 || x.enElRespaldo > 0)"
              :key="t.tabla"
            >
              <span class="tabla">{{ t.tabla }}</span>
              <span class="filas">
                {{ $t('restaurar.deAA', { ahora: cifra(t.ahora), despues: cifra(t.enElRespaldo) }) }}
              </span>
            </li>
          </ul>
        </details>

        <AEntrada
          v-if="r.revisado.llevaContrasena"
          v-model="r.contrasena"
          tipo="password"
          :etiqueta="$t('restaurar.contrasena')"
          :ayuda="$t('restaurar.contrasenaAyuda')"
          :deshabilitado="r.restaurando"
        />

        <p
          v-if="r.faltaLaContrasena"
          class="por-que-no"
        >
          {{ $t('restaurar.faltaLaContrasena') }}
        </p>

        <div class="acciones">
          <ABoton
            variante="peligro"
            :deshabilitado="!r.sePuedeRestaurar"
            :cargando="r.restaurando"
            @click="r.restaurar()"
          >
            {{ $t('restaurar.hazlo') }}
          </ABoton>
          <ABoton
            variante="sutil"
            :deshabilitado="r.restaurando"
            @click="r.descartar()"
          >
            {{ $t('restaurar.mejorNo') }}
          </ABoton>
        </div>
      </template>
    </section>

    <!-- ── 3 · Lo que pasó ───────────────────────────────────────────────── -->
    <AAviso
      v-if="r.hecho"
      tono="exito"
      :titulo="$t('restaurar.hecho.titulo')"
    >
      <p>
        {{
          $t('restaurar.hecho.cuerpo', {
            filas: cifra(r.hecho.filas),
            bajas: cifra(r.hecho.bajasDespues),
            salvadas: cifra(r.hecho.bajasQueSeSalvaron),
          })
        }}
      </p>
      <p v-if="r.hecho.cuentasAReconectar > 0">
        {{
          $t(
            'restaurar.hecho.cuentas',
            { cuentas: r.hecho.cuentasAReconectar },
            r.hecho.cuentasAReconectar,
          )
        }}
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
  </div>
</template>

<style scoped>
.pantalla {
  /* La medida va aquí y el `max-width` en los bloques, no las dos sobre
     `.pantalla`: eso dejó Ajustes centrada y 300 px a la derecha de las demás
     el 29/09/2026, y es lo que lee la sonda de plegado. */
  --arles-medida: 540px;

  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.titulo {
  font-size: var(--arles-font-size-h1);
  font-weight: var(--arles-font-weight-bold);
  color: var(--arles-text);
  margin-bottom: var(--arles-space-2);
}

.entradilla {
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

.detalle {
  list-style: none;
  padding: var(--arles-space-3) 0 0;
  margin: 0;
  font-size: var(--arles-font-size-small);
}

.detalle li {
  display: flex;
  justify-content: space-between;
  padding: var(--arles-space-1) 0;
  border-bottom: 1px solid var(--arles-border);
}

.tabla {
  color: var(--arles-text-muted);
}

.filas {
  color: var(--arles-text);
  font-variant-numeric: tabular-nums;
}

.boton-suelto {
  align-self: flex-start;
}

.por-que-no {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.acciones {
  display: flex;
  gap: var(--arles-space-3);
}

.salvo {
  margin-top: var(--arles-space-2);
  font-weight: var(--arles-font-weight-medium);
}
</style>
