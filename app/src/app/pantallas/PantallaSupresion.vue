<script setup lang="ts">
/**
 * Quién pidió no recibir (entrega 3.4).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE DIRECCIÓN PIDIÓ, CON SUS PALABRAS
 *
 * «Tenemos que tener una lista de los prospectos que seleccionaron la
 * cancelación de correos o WhatsApp» — 21 de septiembre de 2026.
 *
 * Esta pantalla es esa lista. Sirve para dos cosas distintas, y las dos
 * importan:
 *
 *   - **Operar.** Saber a cuánta gente ya no se le puede escribir, y por qué.
 *   - **Probar.** Es el registro de que esas bajas se respetaron, con su fecha
 *     y su motivo, el día que alguien lo pregunte.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA NO DECIDE
 *
 * **Si una baja se puede deshacer no lo decide aquí.** Viene resuelto de Rust
 * en `sePuedeLevantar`. Esta pantalla esconde el botón cuando es `false`, pero
 * eso es cortesía: la defensa está en el núcleo, y si alguien llamara al
 * comando igualmente, el núcleo diría que no.
 *
 * Por qué importa: si la regla viviera aquí, habría dos copias —una en Rust y
 * otra en TypeScript— y el día que divergieran esta pantalla enseñaría un botón
 * que el núcleo rechaza, o escondería uno que sí valía.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import {
  CLAVE_DEL_MOTIVO,
  MOTIVOS_A_MANO,
  POR_PAGINA,
  useSupresionStore,
  type Canal,
  type MotivoDeSupresion,
} from '@/app/stores/supresion'
import {
  AAviso,
  ABoton,
  AEntrada,
  AIcono,
  AInsignia,
  AModulo,
  ASelector,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const supresion = useSupresionStore()
const { t, te } = useI18n()

onMounted(() => {
  void supresion.cargar(0)
})

/** El motivo, ya traducido. Si falta el texto, se dice — no se inventa. */
function motivo(clave: string): string {
  return te(clave) ? t(clave) : t('supresion.motivo.desconocido')
}

/**
 * La fecha, en el formato que se lee de un vistazo.
 *
 * Se guarda en ISO 8601 con zona, que es lo correcto para comparar y ordenar,
 * y es ilegible en una tabla. Si la cadena no se puede interpretar se enseña
 * tal cual: mejor un texto raro que una fecha inventada.
 */
function fecha(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleDateString('es-MX', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

const errorGeneral = computed(() =>
  supresion.errorGeneral
    ? resolverError({ clave: supresion.errorGeneral, detalle: '' })
    : null,
)

/**
 * La cuenta de esta página, no de la lista entera.
 *
 * Y se dice así en pantalla: «50 de 1 284». Enseñar «50 de correo» a secas
 * cuando hay mil doscientas haría creer que ésas son todas (§94).
 */
const nota = computed(() =>
  t('supresion.nota', {
    cuantas: supresion.supresiones.length,
    total: supresion.total,
  }),
)

/** Sobre cuántas filas está buscando de verdad. Se dice para no engañar. */
const avisoDelFiltro = computed(() => {
  if (supresion.filtro.trim() === '') return ''
  return t('supresion.filtroParcial', {
    encontradas: supresion.visibles.length,
    de: supresion.supresiones.length,
  })
})

/**
 * ¿Hay alguna fila que no se pueda levantar?
 *
 * Decide si se enseña la leyenda del candado. Mira **las filas que se están
 * viendo**, no la lista entera: la leyenda explica un símbolo, y un símbolo
 * que no está en pantalla no necesita explicación — aunque exista en la
 * página siguiente.
 */
const hayBloqueadas = computed(() =>
  supresion.visibles.some((f) => !f.sePuedeLevantar),
)

async function levantar(id: string, direccion: string): Promise<void> {
  const seguro = window.confirm(t('supresion.confirmarLevantar', { direccion }))
  if (!seguro) return
  await supresion.levantar(id)
}

function pagina(delta: number): void {
  const siguiente = Math.max(0, supresion.desde + delta * POR_PAGINA)
  void supresion.cargar(siguiente)
}

/* ── Apuntar una baja a mano ────────────────────────────────────────────────
 *
 * EL HUECO QUE ESTO CIERRA, MEDIDO EL 23/10/2026
 *
 * Esta pantalla tenía un buscador y ningún campo para añadir. La única forma
 * de apuntar una baja era la ficha de un contacto, y eso exige que la
 * dirección **sea** un contacto. El caso que de verdad llega no lo es: alguien
 * contesta «ya no me escriban» desde una dirección a la que le reenviaron el
 * correo, o desde otra suya. Esas bajas no se podían apuntar de ninguna forma.
 *
 * Y va en su propio módulo, **fuera** del `v-if` de los estados de la lista:
 * con cero bajas la pantalla enseña su estado vacío, y el formulario tiene que
 * estar precisamente entonces — la primera baja se apunta cuando no hay
 * ninguna.
 */
const nuevaDireccion = ref('')
const nuevoCanal = ref('email')
const nuevoMotivo = ref<string>(MOTIVOS_A_MANO[0] ?? 'manual')
const nuevasNotas = ref('')
const apuntando = ref(false)

/**
 * Qué pasó al apuntar, en clave de texto (§139).
 *
 * Son **dos acuses distintos y las dos son éxito**: «apuntada» y «ya estaba».
 * Enseñar lo mismo en los dos casos haría apuntar dos veces la misma baja sin
 * saberlo; enseñar un error en el segundo mandaría a arreglar algo que está
 * bien. Suprimir es idempotente, y eso se dice (§94).
 */
const acuse = ref<string | null>(null)

const opcionesDeCanal = computed(() => [
  { valor: 'email', texto: t('supresion.canal.correo') },
  { valor: 'whatsapp', texto: t('supresion.canal.whatsapp') },
])

/**
 * Los tres motivos que decide una persona, no los seis.
 *
 * Salen de `MOTIVOS_A_MANO`, que es donde está argumentado: los otros tres los
 * **deduce el sistema** hablando con un servidor, y ofrecerlos aquí dejaría
 * marcar «rebote permanente» sin que haya habido ningún rebote — y el registro
 * dejaría de servir para lo único que sirve, explicar dentro de un año por qué
 * esa persona no recibe.
 */
const opcionesDeMotivo = computed(() =>
  MOTIVOS_A_MANO.map((m) => ({ valor: m, texto: t(CLAVE_DEL_MOTIVO[m]) })),
)

/** Sin dirección no hay nada que apuntar. Lo demás tiene valor por defecto. */
const puedeApuntar = computed(
  () => nuevaDireccion.value.trim() !== '' && !apuntando.value,
)

/**
 * El canal elegido, estrechado sin un `as`.
 *
 * `ASelector` trabaja con cadenas —un `<select>` no tiene tipos—, así que el
 * valor vuelve como `string`. Se estrecha preguntando, no afirmando: una
 * aserción de tipo dejaría pasar cualquier cadena el día que alguien añada una
 * opción y olvide el tipo.
 */
function canalElegido(): Canal {
  return nuevoCanal.value === 'whatsapp' ? 'whatsapp' : 'email'
}

/**
 * Lo mismo con el motivo, y la caída **no es la más cómoda sino la más
 * prudente**: si el valor no está en la lista, el desplegable se manipuló, y
 * ahí el lado seguro es la baja que un clic no deshace.
 */
function motivoElegido(): MotivoDeSupresion {
  return MOTIVOS_A_MANO.find((m) => m === nuevoMotivo.value) ?? 'unsubscribe'
}

async function apuntar(): Promise<void> {
  acuse.value = null
  apuntando.value = true
  try {
    const nueva = await supresion.suprimirDireccion(
      canalElegido(),
      nuevaDireccion.value,
      motivoElegido(),
      nuevasNotas.value,
    )
    // `null` es un fallo, y ya está contado en `errorGeneral`. Poner un acuse
    // encima diría que se apuntó algo que no se apuntó.
    if (nueva === null) return
    acuse.value = nueva ? 'supresion.apuntar.hecha' : 'supresion.apuntar.yaEstaba'
    nuevaDireccion.value = ''
    nuevasNotas.value = ''
  } finally {
    apuntando.value = false
  }
}
</script>

<template>
  <div class="supresion">
    <AModulo
      :titulo="t('supresion.titulo')"
      :nota="nota"
      principal
      nivel="h1"
    >
      <p class="supresion__explicacion">
        {{ t('supresion.explicacion') }}
      </p>

      <!-- ── Apuntar una baja a mano ───────────────────────────────────
           DENTRO del módulo del título y ANTES de la lista, por dos razones
           que tiran en la misma dirección:

             1. Fuera del `v-if` de los estados: con cero bajas la pantalla
                enseña su estado vacío, y la primera baja se apunta justo
                entonces. En su propio módulo debajo también se cumplía, pero
                caía **detrás de una tabla de hasta cincuenta filas** — a dos
                mil píxeles de scroll de quien viene a apuntar una.
             2. El encabezado de la pantalla tiene que seguir siendo el `h1`
                (lo mide la sonda de accesibilidad: no sólo los saltos, también
                por cuál empieza), así que esto no puede ir encima del módulo.

           La lista sigue siendo lo que da nombre a la pantalla. Lo que cambia
           es que la acción semanal se alcanza sin buscarla. -->
      <section class="supresion__apuntar-caja">
        <h2 class="supresion__apuntar-titulo">
          {{ t('supresion.apuntar.titulo') }}
        </h2>
        <p class="supresion__explicacion">
          {{ t('supresion.apuntar.explicacion') }}
        </p>

        <div class="supresion__apuntar">
          <AEntrada
            v-model="nuevaDireccion"
            :etiqueta="t('supresion.apuntar.direccion')"
            :marcador="t('supresion.apuntar.direccionMarcador')"
            :ayuda="t('supresion.apuntar.direccionAyuda')"
            autocompletado="off"
          />
          <ASelector
            v-model="nuevoCanal"
            :etiqueta="t('supresion.apuntar.canal')"
            :opciones="opcionesDeCanal"
          />
          <ASelector
            v-model="nuevoMotivo"
            :etiqueta="t('supresion.apuntar.motivo')"
            :opciones="opcionesDeMotivo"
            :ayuda="t('supresion.apuntar.motivoAyuda')"
          />
          <AEntrada
            v-model="nuevasNotas"
            :etiqueta="t('supresion.apuntar.notas')"
            :marcador="t('supresion.apuntar.notasMarcador')"
            :ayuda="t('supresion.apuntar.notasAyuda')"
            autocompletado="off"
          />
        </div>

        <!-- El botón dice POR QUÉ está apagado (§95). Un control que no
             responde y no se explica se lee como una avería — es el defecto
             que la 8.2c y la 8.3c tuvieron con «Repite la contraseña» y con
             «Restaurar y reemplazar». -->
        <div class="supresion__apuntar-pie">
          <ABoton
            :deshabilitado="!puedeApuntar"
            @click="apuntar"
          >
            {{ t('supresion.apuntar.boton') }}
          </ABoton>
          <p
            v-if="!puedeApuntar && !apuntando"
            class="supresion__falta"
          >
            {{ t('supresion.apuntar.faltaLaDireccion') }}
          </p>
        </div>

        <AAviso
          v-if="acuse"
          tono="exito"
        >
          {{ t(acuse) }}
        </AAviso>
      </section>

      <!-- ── LA LISTA NECESITA SU PROPIO ENCABEZADO ──────────────────────
           Y no es cosmética. Con un `h2` encima del formulario y ninguno
           aquí, todo lo que viene después —las cifras, el buscador y la tabla
           entera— cuelga de «Apuntar una baja a mano» para quien navega por
           encabezados: la lista quedaría leída como parte del formulario.

           La sonda de accesibilidad no lo caza, y es correcto que no lo cace:
           mide los saltos de nivel y por cuál empieza, y `h1 → h2 → h2` no
           tiene ninguno. Lo que está mal no es la jerarquía, es lo que
           agrupa. Esto se ve mirando, no midiendo. -->
      <h2 class="supresion__apuntar-titulo">
        {{ t('supresion.listaTitulo') }}
      </h2>

      <AAviso
        v-if="errorGeneral"
        tono="peligro"
      >
        <strong>{{ errorGeneral.que }}</strong>
        <span>{{ errorGeneral.como }}</span>
        <span>{{ errorGeneral.salvo }}</span>
      </AAviso>

      <EstadoCargando
        v-else-if="supresion.cargando"
        :mensaje="t('supresion.cargando')"
      />

      <EstadoVacio
        v-else-if="supresion.total === 0"
        :titulo="t('supresion.vacioTitulo')"
        :cuerpo="t('supresion.vacioCuerpo')"
      />

      <template v-else>
        <!-- Cuántas por canal. Es lo primero que Dirección quiere ver. -->
        <div class="supresion__cifras">
          <AInsignia tono="neutro">
            {{ t('supresion.porCorreo', { n: supresion.cuantasDeCorreo }) }}
          </AInsignia>
          <AInsignia tono="neutro">
            {{ t('supresion.porWhatsApp', { n: supresion.cuantasDeWhatsApp }) }}
          </AInsignia>
        </div>

        <div class="supresion__buscar">
          <AEntrada
            v-model="supresion.filtro"
            :etiqueta="t('supresion.buscar')"
            :marcador="t('supresion.buscarMarcador')"
          />
          <p
            v-if="avisoDelFiltro"
            class="supresion__filtro-parcial"
          >
            {{ avisoDelFiltro }}
          </p>
        </div>

        <!-- ── La leyenda del candado ─────────────────────────────────────
             Dice UNA VEZ lo que antes se repetía en cada fila bloqueada.

             `v-if` y no siempre: sin ninguna fila bloqueada, explicar un
             símbolo que no aparece es ruido, y además haría dudar de si falta
             algo. La leyenda existe cuando existe lo que explica.

             Va ENCIMA de la tabla y no debajo: se lee antes de encontrarse el
             primer candado, no después de haber dudado qué era. -->
        <p
          v-if="hayBloqueadas"
          class="supresion__leyenda"
        >
          <AIcono
            nombre="candado"
            :tamano="14"
          />
          <span>{{ t('supresion.leyendaCandado') }}</span>
        </p>

        <table class="tabla">
          <thead>
            <tr>
              <th scope="col">
                {{ t('supresion.columna.direccion') }}
              </th>
              <th scope="col">
                {{ t('supresion.columna.deQuien') }}
              </th>
              <th scope="col">
                {{ t('supresion.columna.motivo') }}
              </th>
              <th scope="col">
                {{ t('supresion.columna.fecha') }}
              </th>
              <th scope="col">
                <span class="sr-only">{{ t('supresion.columna.acciones') }}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="fila in supresion.visibles"
              :key="fila.id"
              class="fila"
            >
              <!-- La caja interior existe por una razón concreta: `display:
                   flex` puesto sobre el `<td>` saca la celda del algoritmo de
                   tabla, y entonces su línea inferior deja de alinear con la
                   del resto de la fila. Se veía en la vista previa: los
                   separadores de la primera columna caían a una altura
                   distinta de los de «De quién». La celda se queda celda; lo
                   que se coloca es su contenido. -->
              <td>
                <div class="fila__direccion">
                  <span class="fila__canal">
                    {{
                      fila.canal === 'email'
                        ? t('supresion.canal.correo')
                        : t('supresion.canal.whatsapp')
                    }}
                  </span>
                  <span class="fila__valor">{{ fila.direccion }}</span>
                  <AInsignia
                    v-if="fila.global"
                    tono="neutro"
                  >
                    {{ t('supresion.global') }}
                  </AInsignia>
                </div>
              </td>
              <td>{{ fila.deQuien || t('supresion.sinContacto') }}</td>
              <td>
                <div class="fila__motivo">
                  <span>{{ motivo(fila.motivo) }}</span>
                  <!-- ── El candado, y por qué no es la frase entera ────────
                       Hasta el 22/09/2026 aquí se repetía «Lo pidió ella
                       misma, así que ARLES no lo deshace», en dos renglones y
                       en cada fila bloqueada. Dirección lo vio en la vista
                       previa: con doscientas bajas es una tabla tres veces más
                       larga para decir lo mismo tres veces.

                       El candado NO se queda solo. Lleva tres cosas:
                         - `title`, para quien pase el ratón;
                         - texto en `.sr-only`, para el lector de pantalla,
                           que lee la frase completa y no «imagen»;
                         - la frase entera una vez encima de la tabla.

                       Un icono suelto es información codificada sólo en la
                       forma, que es el mismo error que codificarla sólo en el
                       color (§19). Y §95 sigue exigiendo que se diga qué pasó
                       y qué está a salvo: eso lo dice la leyenda de arriba. -->
                  <span
                    v-if="!fila.sePuedeLevantar"
                    class="fila__candado"
                    :title="t('supresion.loPidioLaPersona')"
                  >
                    <AIcono
                      nombre="candado"
                      :tamano="14"
                    />
                    <span class="sr-only">{{ t('supresion.loPidioLaPersona') }}</span>
                  </span>
                </div>
              </td>
              <td class="fila__fecha">
                {{ fecha(fila.creada) }}
              </td>
              <td class="fila__acciones">
                <ABoton
                  v-if="fila.sePuedeLevantar"
                  variante="sutil"
                  tamano="compacto"
                  @click="levantar(fila.id, fila.direccion)"
                >
                  {{ t('supresion.volverAPermitir') }}
                </ABoton>
              </td>
            </tr>
          </tbody>
        </table>

        <div
          v-if="supresion.total > POR_PAGINA"
          class="supresion__paginas"
        >
          <ABoton
            variante="sutil"
            :deshabilitado="supresion.desde === 0"
            @click="pagina(-1)"
          >
            {{ t('supresion.anteriores') }}
          </ABoton>
          <ABoton
            variante="sutil"
            :deshabilitado="!supresion.hayMas"
            @click="pagina(1)"
          >
            {{ t('supresion.siguientes') }}
          </ABoton>
        </div>
      </template>
    </AModulo>
  </div>
</template>

<style scoped>
/* §17: ni un hex, ni un píxel suelto. Todo sale de los tokens.
 *
 * ───────────────────────────────────────────────────────────────────────────
 * ESTA HOJA ENTERA ESTUVO MUERTA · encontrado el 22/09/2026
 *
 * Se escribió con catorce nombres de token **inventados** —`--espacio-3`,
 * `--color-texto-suave`, `--texto-sm`…— cuando los del proyecto son
 * `--arles-space-3`, `--arles-text-muted`, `--arles-font-size-small`.
 * Ninguno de los catorce existía. Ni uno.
 *
 * Consecuencia: la tabla salía **sin relleno, sin separadores y sin
 * jerarquía**; la etiqueta del canal se pegaba a la dirección
 * —«CORREOana@empresa.mx»—, la columna «Por qué» se desbordaba sobre la fila
 * siguiente, y el texto reservado a los lectores de pantalla se veía.
 *
 * POR QUÉ NO LO CAZÓ NADA, que es lo que importa:
 *
 *   - CSS con una variable inexistente **no es un error**. No avisa el
 *     compilador, ni el linter, ni el navegador: la declaración se descarta en
 *     silencio y la propiedad se queda en su valor inicial.
 *   - La comprobación de §17 busca literales —hex, píxeles sueltos—. Estos
 *     eran `var(…)` perfectamente formados, así que pasaba.
 *   - `sonda:supresion` mide **comportamiento**: que la lista cargue, que
 *     busque, que el botón de levantar aparezca sólo donde toca. Todo eso
 *     funcionaba. Una pantalla puede hacer lo correcto y ser ilegible.
 *
 * Desde hoy lo vigila el validador: «ningún estilo usa un token que no
 * existe». Se comprobó rompiéndolo.
 * ───────────────────────────────────────────────────────────────────────────
 */
.supresion {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.supresion__explicacion {
  margin: 0 0 var(--arles-space-4);
  max-width: 62ch;
  color: var(--arles-text-muted);
}

.supresion__cifras {
  display: flex;
  gap: var(--arles-space-3);
  margin-bottom: var(--arles-space-4);
}

.supresion__buscar {
  margin-bottom: var(--arles-space-4);
  max-width: 62ch;
}

/* Dos columnas en pantalla ancha y una en estrecha. `minmax(0, …)` y no
 * `1fr` a secas: sin el mínimo a cero, una ayuda larga ensancha su columna y
 * empuja la otra fuera — el defecto que las cinco cabeceras de Contactos
 * tuvieron a 1024 px. */
/* La caja de apuntar, separada de la lista por una línea y no sólo por aire:
 * son dos cosas distintas dentro del mismo módulo, y un hueco a secas las
 * lee como una sola. */
.supresion__apuntar-caja {
  margin-top: var(--arles-space-5);
  margin-bottom: var(--arles-space-6);
  padding-bottom: var(--arles-space-6);
  border-bottom: 1px solid var(--arles-border);
}

.supresion__apuntar-titulo {
  margin: 0 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h3);
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

/* Dos columnas en ancho y una en estrecho, con los cuatro campos en 2×2.
 *
 * Con `28ch` cabían tres y el cuarto bajaba solo a una fila con dos huecos al
 * lado. Medido antes de tocarlo: 297 px por columna en 923, y «Notas» 146 px
 * por debajo de los otros tres.
 *
 * El `min(…, 100%)` es lo que evita el desborde en estrecho: sin él una
 * columna de 38ch no cabe en una ventana de 360 px y la rejilla se sale. */
.supresion__apuntar {
  display: grid;
  gap: var(--arles-space-4);
  grid-template-columns: repeat(auto-fit, minmax(min(38ch, 100%), 1fr));
  margin-bottom: var(--arles-space-4);
}

/* ── LA REJILLA NEUTRALIZA EL AIRE ENTRE CAMPOS APILADOS ─────────────────
 *
 * `AModulo` añade 8 px con `.campo + .campo` (H-09, que pidió Dirección el
 * 16/10/2026): ahí separa la ayuda de un campo de la etiqueta del siguiente
 * —las dos texto pequeño, que se leían como una sola línea—. En una columna
 * está bien.
 *
 * Lado a lado hace otra cosa: se lo pone a todos menos al primero, así que la
 * fila entera arranca 8 px por debajo de su primera columna. **Medido antes
 * de tocar nada**: 343 px el primer campo, 351 los otros dos.
 *
 * No se toca la regla del módulo. Sirve para lo suyo, y al otro formulario en
 * rejilla del proyecto ni siquiera le llega: `FormularioDeContacto` vive en un
 * modal y no dentro de un `AModulo` — comprobado midiéndolo, los dos campos a
 * 253 px con margen 0, en vez de darlo por hecho y anotar un defecto que no
 * existe. El aire entre filas lo pone el `gap`, que es de quien es ese
 * trabajo. */
.supresion__apuntar-caja .supresion__apuntar > :deep(.campo + .campo) {
  margin-top: 0;
}

/* El botón y el por qué está apagado, en la misma banda y con el aviso
 * debajo: un aviso que explica un control tiene que verse con el control
 * delante. */
.supresion__apuntar-pie {
  margin-bottom: var(--arles-space-4);
}

.supresion__filtro-parcial,
.supresion__falta {
  margin: var(--arles-space-2) 0 0;
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

.tabla {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--arles-font-size-small);
}

.tabla th {
  text-align: start;
  padding: var(--arles-space-2) var(--arles-space-3);
  border-bottom: var(--arles-border-width) solid var(--arles-border-strong);
  color: var(--arles-text-muted);
  font-weight: var(--arles-font-weight-semibold);
}

.fila td {
  padding: var(--arles-space-3);
  border-bottom: var(--arles-border-width) solid var(--arles-border);
  vertical-align: top;
}

.fila__direccion {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--arles-space-2);
}

.fila__canal {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.fila__valor {
  font-variant-numeric: tabular-nums;
}

/* NO es una columna flex.
 *
 * Lo era mientras el motivo y su explicación eran dos bloques apilados. Ahora
 * la explicación es un candado que tiene que ir **en la misma línea** que el
 * motivo —«Pidió la baja 🔒»—, y una columna flex lo tiraba al renglón de
 * abajo, que es justo el renglón de más que Dirección mandó quitar.
 *
 * Como bloque normal, el candado fluye detrás del texto y se lleva su salto
 * de línea sólo si no cabe. */
.fila__motivo {
  display: block;
}

/* El candado de la fila. Por qué no hay botón ahí.

   `inline-flex` y no `block`: va pegado al motivo, en su misma línea, para
   que se lea «Pidió la baja 🔒» como una sola cosa. Antes era un párrafo
   debajo con la frase entera, y cada fila medía dos renglones de más. */
.fila__candado {
  display: inline-flex;
  align-items: center;
  margin-inline-start: var(--arles-space-2);
  color: var(--arles-text-muted);
  /* La línea base del icono se alinea con la del texto que acompaña. */
  vertical-align: middle;
}

/* La leyenda que explica el candado, una sola vez. Se lee como una nota, no
   como un aviso: no ha pasado nada malo, sólo se está nombrando un símbolo. */
.supresion__leyenda {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  margin: 0 0 var(--arles-space-3);
  max-width: 62ch;
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

.fila__fecha {
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.fila__acciones {
  text-align: end;
}

.supresion__paginas {
  display: flex;
  justify-content: space-between;
  margin-top: var(--arles-space-4);
}

/* `.sr-only` vive en `base.css` desde el 22/10/2026, global: `ATabla` la
   necesitó y una clase `scoped` no sale de su componente. La copia que estaba
   aquí se quitó para que no se separen — dos definiciones de «esconder sin
   sacar del árbol de accesibilidad» es el sitio donde una se queda con un
   `display: none` y nadie lo nota. */
</style>
