<script setup lang="ts">
/**
 * Contactos (entrega 3.2).
 *
 * Tres cosas conviven aquí, y el orden importa:
 *
 * 1. **La tabla** enseña el canal *principal* de cada tipo. No todos: con diez
 *    canales por contacto, una tabla que los enseñara todos dejaría de ser una
 *    tabla (L-14).
 * 2. **La ficha** sí los enseña todos, agrupados y con el principal arriba. Ese
 *    orden **lo decide la base**, no esta pantalla: si lo reordenara aquí, la
 *    lista y la ficha podrían acabar diciendo cosas distintas.
 * 3. **El aviso de supresión** marca a quien está en la lista de no escribir.
 *    Es un aviso, **no un bloqueo** (L-13): el contacto se guarda y se edita
 *    igual. Quien decide si se le escribe es la campaña, en su comprobación
 *    previa.
 *
 * Lo que NO hay todavía, y se dice en pantalla en vez de callarlo: importar un
 * CSV o un XLSX (entrega 3.3) y el borrado definitivo con sus derechos ARCO
 * (entrega 3.4). El botón de baja de aquí es **lógico**: el contacto deja de
 * salir, y su rastro de envíos se conserva.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * DAR DE BAJA Y NO VOLVER A ESCRIBIR SON DOS COSAS (17/10/2026)
 *
 * Y hasta hoy sólo se podía hacer la primera. `suprimir_contacto` estaba
 * registrado en la frontera, el almacén lo envolvía, y **ninguna pantalla lo
 * llamaba**: el patrón del §9 por vigésima vez, encontrado al arreglar la
 * comprobación que existe para cazarlo —que vigilaba cero funciones—.
 *
 * La diferencia importa y por eso van juntas en el mismo diálogo pero
 * separadas:
 *
 * - **Dar de baja** quita el contacto de la lista. Si mañana se vuelve a
 *   importar el mismo Excel, **vuelve a entrar**, y se le escribe.
 * - **No volver a escribirle** apunta sus direcciones en la lista de
 *   supresión, que no depende del contacto: sobrevive a la reimportación y es
 *   lo único que cumple de verdad un «no me escriban más».
 *
 * Quien pide por teléfono que no le escriban necesita la segunda, y desde
 * ARLES no había dónde apuntarlo.
 *
 * **El motivo se pregunta y no se supone.** Marcar «lo pidió la persona»
 * cuando lo decidimos nosotros falsea el registro que existe justo para
 * contestar una reclamación.
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import {
  aBorrador,
  nombreVisible,
  principalDe,
  useContactosStore,
  type BorradorDeContacto,
  type Canal,
  type Contacto,
} from '@/app/stores/contactos'
import {
  CLAVE_DEL_MOTIVO,
  MOTIVOS_A_MANO,
  useSupresionStore,
  type MotivoDeSupresion,
} from '@/app/stores/supresion'
import {
  AAviso,
  ABoton,
  AEntrada,
  AIcono,
  AInsignia,
  AModal,
  AModulo,
  ASelector,
  ATabla,
  EstadoCargando,
  EstadoError,
  EstadoVacio,
  type ColumnaDeTabla,
} from '@/design/componentes'

import FormularioDeContacto from './FormularioDeContacto.vue'

const contactos = useContactosStore()
const supresion = useSupresionStore()
const { t } = useI18n()

/** ¿Además de la baja, apuntar que no se le vuelva a escribir? */
const noEscribirNuncaMas = ref(false)
const motivoDeLaBaja = ref<MotivoDeSupresion>('unsubscribe')
const notasDeLaBaja = ref('')

const opcionesDeMotivo = computed(() =>
  MOTIVOS_A_MANO.map((m) => ({ valor: m, texto: t(CLAVE_DEL_MOTIVO[m]) })),
)

/** Qué diálogo está abierto. `null` es «ninguno». */
const dialogo = ref<'alta' | 'edicion' | 'baja' | null>(null)

/**
 * Cuántas veces se ha abierto un diálogo. Es la clave del formulario.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ HACE FALTA UN CONTADOR
 *
 * `AModal` **no desmonta** lo que lleva dentro al cerrarse: sólo cierra el
 * `<dialog>`. Así que el formulario sobrevive entre aperturas, con lo que
 * hubiera escrito la vez anterior.
 *
 * El `watch` sobre `inicial` no lo arregla: al dar de alta dos contactos
 * seguidos, `inicial` pasa de `null` a `null` y no se dispara. El segundo
 * formulario abría con las formas de contacto del primero, y al guardar
 * chocaba con una dirección que el usuario **no había escrito**.
 *
 * Lo cazó `sonda:contactos`: el error de dirección repetida nombraba un móvil
 * que en ese formulario no existía. Un `key` que cambia en cada apertura
 * fuerza el remontaje y el formulario nace limpio.
 * ─────────────────────────────────────────────────────────────────────────
 */
const aperturas = ref(0)
/** El contacto sobre el que actúan «editar», «baja» y la ficha. */
const elegido = ref<Contacto | null>(null)
const seleccionada = ref<string | undefined>(undefined)

onMounted(() => {
  void contactos.cargar()
})

const filas = computed(() => contactos.contactos)

/**
 * El contacto de la ficha.
 *
 * Se recalcula desde la lista en vez de guardarse una copia: tras editar, la
 * lista se recarga y una copia dejaría la ficha enseñando los datos viejos.
 */
const enFicha = computed(() =>
  elegido.value
    ? (filas.value.find((c) => c.id === elegido.value?.id) ?? null)
    : null,
)

/** Lo que enseña la tabla de un tipo de canal: el principal, o nada. */
function principalVisible(c: Contacto, canal: Canal): string {
  return principalDe(c, canal)?.valorRaw ?? ''
}

/** Cuántos canales de ese tipo tiene, para el «+2» de la tabla. */
function cuantosDe(c: Contacto, canal: Canal): number {
  return c.canales.filter((k) => k.canal === canal).length
}

/**
 * El texto de una celda de canal: el principal y, si hay más, cuántos.
 *
 * «ana@empresa.mx +2» dice que hay otros dos sin obligar a abrir la ficha, y
 * sin ensanchar la columna hasta lo ilegible.
 */
function celdaDeCanal(c: Contacto, canal: Canal): string {
  const principal = principalVisible(c, canal)
  if (principal === '') return '—'
  const otros = cuantosDe(c, canal) - 1
  return otros > 0 ? `${principal}  +${otros}` : principal
}

const columnas = computed<readonly ColumnaDeTabla<Contacto>[]>(() => [
  {
    id: 'nombre',
    titulo: t('contactos.columna.nombre'),
    ancho: '1.4fr',
    valor: (c) => nombreVisible(c) || t('contactos.sinNombre'),
  },
  {
    id: 'empresa',
    titulo: t('contactos.columna.empresa'),
    ancho: '1fr',
    valor: (c) => c.empresa || '—',
  },
  {
    id: 'email',
    titulo: t('contactos.columna.email'),
    ancho: '1.6fr',
    valor: (c) => celdaDeCanal(c, 'email'),
  },
  {
    id: 'whatsapp',
    titulo: t('contactos.columna.whatsapp'),
    ancho: '1.2fr',
    valor: (c) => celdaDeCanal(c, 'whatsapp'),
  },
])

/** Cuántos de la página tienen alguna dirección suprimida. */
const cuantosSuprimidos = computed(
  () => filas.value.filter((c) => contactos.estaSuprimido(c)).length,
)

const errorGeneral = computed(() =>
  contactos.errorGeneral && dialogo.value === null
    ? resolverError({ clave: contactos.errorGeneral, detalle: '' })
    : null,
)

/** El borrador que recibe el formulario: `null` en alta, el contacto en edición. */
const borradorInicial = computed<BorradorDeContacto | null>(() =>
  dialogo.value === 'edicion' && enFicha.value ? aBorrador(enFicha.value) : null,
)

function abrirAlta(): void {
  contactos.limpiarErrores()
  aperturas.value += 1
  elegido.value = null
  dialogo.value = 'alta'
}

function abrirEdicion(c: Contacto): void {
  contactos.limpiarErrores()
  aperturas.value += 1
  elegido.value = c
  dialogo.value = 'edicion'
}

function abrirBaja(c: Contacto): void {
  contactos.limpiarErrores()
  supresion.errorGeneral = null
  elegido.value = c
  dialogo.value = 'baja'
  // En blanco cada vez. Dejar la casilla marcada de la baja anterior haría
  // suprimir a quien sólo se quería quitar de la lista, y eso no se deshace
  // con un botón cuando el motivo es «lo pidió la persona».
  noEscribirNuncaMas.value = false
  motivoDeLaBaja.value = 'unsubscribe'
  notasDeLaBaja.value = ''
}

function cerrar(): void {
  dialogo.value = null
  contactos.limpiarErrores()
  supresion.errorGeneral = null
}

/** Abre la ficha al activar una fila (doble clic o Intro). */
function verFicha(c: Contacto): void {
  elegido.value = c
  seleccionada.value = c.id
}

/**
 * Y también **al elegirla con un solo clic**.
 *
 * ── POR QUÉ NO BASTABA CON DECIRLO (21/10/2026) ───────────────────────────
 *
 * El estado vacío de la columna lleva escrito desde la 7.7 *«haz doble clic
 * en una fila para ver todos sus correos y WhatsApp, y para editarlo o darlo
 * de baja»*. Dirección probó la aplicación y escribió: **«tampoco el
 * modificar la información de contacto»**.
 *
 * El botón de editar existía, la instrucción estaba escrita, y aun así no se
 * llegó a él. Un doble clic no es un gesto que nadie pruebe en una tabla, y
 * **una instrucción que nadie sigue no es una instrucción: es una
 * coartada**.
 *
 * Elegir una fila y ver su ficha es la misma intención, así que ahora basta
 * un clic. El doble clic sigue valiendo —quien ya lo aprendió no tiene que
 * desaprenderlo— y el teclado también, que es lo que `@activar` cubre.
 */
function alElegir(id: string | undefined): void {
  seleccionada.value = id
  elegido.value = contactos.contactos.find((c) => c.id === id) ?? null
}

async function guardar(borrador: BorradorDeContacto): Promise<void> {
  const id = enFicha.value?.id
  const bien =
    dialogo.value === 'edicion' && id !== undefined
      ? await contactos.editar(id, borrador)
      : await contactos.crear(borrador)
  // Sólo se cierra si se guardó. Cerrar con un error puesto haría desaparecer
  // los campos marcados junto con el diálogo, y el usuario no sabría qué falló.
  if (bien) cerrar()
}

/**
 * Da de baja, y —si se marcó— apunta además que no se le vuelva a escribir.
 *
 * **El orden no es indiferente: la supresión va primero.** Si fallara la baja
 * después, queda apuntado que esa persona no quiere recibir, que es lo
 * irreversible de verdad. Al revés, un fallo al suprimir dejaría el contacto
 * fuera de la lista y su dirección **sin proteger**: la siguiente importación
 * del mismo Excel lo devuelve y se le escribe.
 *
 * Y si la supresión falla, no se da de baja: con el error en pantalla, quien
 * mire ve el contacto todavía ahí y sabe que no se hizo nada.
 */
async function confirmarBaja(): Promise<void> {
  const id = elegido.value?.id
  if (id === undefined) return

  if (noEscribirNuncaMas.value) {
    const resumen = await supresion.suprimirContacto(
      id,
      motivoDeLaBaja.value,
      notasDeLaBaja.value,
    )
    if (resumen === null) return
  }

  if (await contactos.borrar(id)) {
    elegido.value = null
    seleccionada.value = undefined
    cerrar()
  }
}

/**
 * El error del diálogo de baja, de cualquiera de los dos almacenes.
 *
 * La supresión va primero y puede fallar sola. Mirando sólo
 * `contactos.errorGeneral`, ese fallo no diría nada: el diálogo se quedaría
 * quieto al pulsar y se leería como una avería (§95).
 */
const errorDelDialogo = computed(() => {
  if (dialogo.value !== 'baja') return null
  const clave = supresion.errorGeneral ?? contactos.errorGeneral
  return clave ? resolverError({ clave, detalle: '' }) : null
})
</script>

<template>
  <section class="contactos">
    <header class="contactos__cabecera">
      <div>
        <h1>{{ t('contactos.titulo') }}</h1>
        <!--
          La cifra es una cifra, no un adjetivo (§94). Y dice cuántos se están
          viendo de cuántos hay, no «muchos».
        -->
        <p class="contactos__recuento">
          {{
            t('contactos.recuento', {
              viendo: filas.length,
              total: contactos.total,
            })
          }}
        </p>
      </div>
      <!-- ── Acciones, no destinos ──────────────────────────────────────────
           AQUÍ HABÍA UN ENLACE A LAS BAJAS. Se quitó el 22/09/2026 por
           decisión de Dirección, y el criterio que deja vale para lo que
           venga después:

             **La barra lateral lleva a los SITIOS. Los botones hacen COSAS.**

           «Quién pidió no recibir» era un sitio, y desde O-3 la barra ya lo
           enseña como «Bajas» colgando de Contactos. Tener las dos puertas no
           era comodidad: era el mismo cuarto con dos nombres, que es como
           acaban divergiendo el día que alguien renombra uno.

           Es la segunda vez que Dirección corrige lo mismo —la primera fue el
           puente de Ajustes— y las dos veces el atajo existía por el mismo
           motivo: lo escribí cuando la barra no enseñaba el destino. La
           respuesta a «no se encuentra» es arreglar la navegación, no añadir
           un atajo.

           «Cargar una tabla» y «Agregar contacto» SÍ se quedan, y no es
           incoherencia: importar y dar de alta son cosas que se hacen, no
           sitios a los que se va. Que importar además tenga pantalla propia no
           lo convierte en un destino; el asistente es el camino de la acción.
           Por eso son botones con icono y no enlaces discretos. -->
      <div class="contactos__acciones">
        <ABoton
          variante="secundario"
          icono="remitentes"
          a="/contactos/importar"
        >
          {{ t('contactos.importar') }}
        </ABoton>
        <ABoton
          variante="primario"
          icono="mas"
          @click="abrirAlta"
        >
          {{ t('contactos.agregar') }}
        </ABoton>
      </div>
    </header>

    <AAviso
      v-if="errorGeneral"
      tono="peligro"
      :titulo="errorGeneral.que"
      urgente
    >
      <p>{{ errorGeneral.como }}</p>
      <p class="contactos__salvo">
        {{ errorGeneral.salvo }}
      </p>
    </AAviso>

    <!--
      El aviso de supresión va arriba y no dentro de cada fila: es un dato del
      conjunto. Y dice explícitamente que NO bloquea, porque un aviso rojo sin
      esa frase se lee como «esto no se puede usar».
    -->
    <AAviso
      v-if="cuantosSuprimidos > 0"
      tono="aviso"
      :titulo="t('contactos.supresion.titulo', { n: cuantosSuprimidos })"
    >
      <p>{{ t('contactos.supresion.explicacion') }}</p>
    </AAviso>

    <EstadoCargando v-if="contactos.cargando && !contactos.cargada" />

    <EstadoError
      v-else-if="errorGeneral && filas.length === 0"
      :que="errorGeneral.que"
      :como="errorGeneral.como"
      :salvo="errorGeneral.salvo"
    />

    <EstadoVacio
      v-else-if="filas.length === 0"
      :titulo="t('contactos.vacio.titulo')"
      :cuerpo="t('contactos.vacio.cuerpo')"
    >
      <!--
        `#accion` y no la ranura por defecto. Estos botones llevaban desde la
        entrega 3.2 **sin pintarse**: `EstadoVacio` sólo pinta la ranura con ese
        nombre, y todo lo demás se descarta sin error ni aviso. `sonda:contactos`
        no lo vio porque buscaba el botón con `.first()`, y encontraba el del
        encabezado — que sí existe.
      -->
      <template #accion>
        <ABoton
          variante="primario"
          icono="remitentes"
          a="/contactos/importar"
        >
          {{ t('contactos.importar') }}
        </ABoton>
        <ABoton
          variante="secundario"
          icono="mas"
          @click="abrirAlta"
        >
          {{ t('contactos.agregar') }}
        </ABoton>
      </template>
    </EstadoVacio>

    <div
      v-else
      class="contactos__cuerpo"
    >
      <ATabla
        :seleccionada="seleccionada"
        class="contactos__tabla"
        :filas="filas"
        :columnas="columnas"
        :etiqueta="t('contactos.tablaEtiqueta')"
        @update:seleccionada="alElegir"
        @activar="verFicha"
      />

      <!-- Sin contacto elegido, la columna decía nada. Ahora dice qué hacer:
           el hueco estaba, la pista no. -->
      <EstadoVacio
        v-if="!enFicha && filas.length > 0"
        class="ficha"
        :titulo="t('contactos.fichaVaciaTitulo')"
        :cuerpo="t('contactos.fichaVaciaCuerpo')"
      />

      <!-- La ficha, al lado de la tabla y no encima: se compara con la lista. -->
      <AModulo
        v-else-if="enFicha"
        class="ficha"
        :titulo="nombreVisible(enFicha) || t('contactos.sinNombre')"
      >
        <p
          v-if="enFicha.empresa"
          class="ficha__empresa"
        >
          {{ enFicha.empresa }}
        </p>

        <AInsignia
          v-if="contactos.estaSuprimido(enFicha)"
          tono="aviso"
        >
          {{ t('contactos.supresion.insignia') }}
        </AInsignia>

        <h3 class="ficha__titulo">
          {{ t('contactos.canales.titulo') }}
        </h3>
        <!--
          TODOS los canales, en el orden que da la base (L-14). Esta pantalla
          no los reordena: si lo hiciera, la ficha y la lista podrían acabar
          ordenando distinto.
        -->
        <ul class="ficha__canales">
          <li
            v-for="canal in enFicha.canales"
            :key="canal.valorNormalizado"
            class="ficha__canal"
            :class="{ 'ficha__canal--principal': canal.principal }"
          >
            <AIcono :nombre="canal.canal === 'email' ? 'remitentes' : 'campanas'" />
            <span class="ficha__valor">
              {{ canal.valorRaw }}
              <!--
                La forma que ARLES va a usar de verdad, cuando no coincide con
                lo escrito. Pasa con el móvil mexicano: quien escribe
                «+52 1 81…» tiene que poder ver que se guardó «+5281…», sin el
                «1». Es §65 —lo que se dice es lo que se hace— y es además el
                único sitio donde esa normalización es visible.
              -->
              <small
                v-if="canal.valorNormalizado !== canal.valorRaw"
                class="ficha__normalizado"
              >
                {{ t('contactos.canales.seUsara', { valor: canal.valorNormalizado }) }}
              </small>
            </span>
            <AInsignia
              v-if="canal.principal"
              tono="exito"
            >
              {{ t('contactos.canales.principal') }}
            </AInsignia>
            <AInsignia
              v-if="contactos.suprimidas.includes(canal.valorNormalizado)"
              tono="aviso"
            >
              {{ t('contactos.supresion.insignia') }}
            </AInsignia>
          </li>
        </ul>

        <div class="ficha__acciones">
          <ABoton
            icono="ajustes"
            @click="abrirEdicion(enFicha)"
          >
            {{ t('contactos.editar') }}
          </ABoton>
          <ABoton
            variante="peligro"
            icono="suprimido"
            @click="abrirBaja(enFicha)"
          >
            {{ t('contactos.baja') }}
          </ABoton>
        </div>
      </AModulo>
    </div>

    <AModal
      :abierto="dialogo === 'alta' || dialogo === 'edicion'"
      :titulo="dialogo === 'edicion' ? t('contactos.editar') : t('contactos.agregar')"
      @cerrar="cerrar"
    >
      <FormularioDeContacto
        :key="aperturas"
        :inicial="borradorInicial"
        :guardando="contactos.guardando"
        :errores-de-canal="contactos.erroresDeCanal"
        :errores-de-campo="contactos.erroresDeCampo"
        :error-general="contactos.errorGeneral"
        :direccion-en-conflicto="contactos.direccionEnConflicto"
        @guardar="guardar"
        @cancelar="cerrar"
        @tocar="contactos.limpiarErrores"
      />
    </AModal>

    <AModal
      :abierto="dialogo === 'baja'"
      :titulo="t('contactos.bajaDialogo.titulo')"
      destructiva
      @cerrar="cerrar"
    >
      <p>
        {{
          t('contactos.bajaDialogo.pregunta', {
            nombre: nombreVisible(elegido ?? { nombre: '', apellido: '' }) || t('contactos.sinNombre'),
          })
        }}
      </p>
      <!--
        Qué se conserva, dicho antes de pulsar. Es la tercera parte del §95
        —«qué está a salvo»— y aquí es lo que distingue esta baja del borrado
        definitivo de la entrega 3.4.
      -->
      <p class="contactos__salvo">
        {{ t('contactos.bajaDialogo.queSeConserva') }}
      </p>

      <!-- ── La supresión, que es lo que cumple un «no me escriban más» ──
           Dar de baja quita el contacto de la lista; si mañana se reimporta
           el mismo Excel, vuelve a entrar. Esto apunta sus direcciones en la
           lista de supresión, que no depende del contacto. -->
      <div class="contactos__supresion">
        <label class="contactos__casilla">
          <input
            v-model="noEscribirNuncaMas"
            type="checkbox"
          >
          {{ t('contactos.bajaDialogo.noEscribirNuncaMas') }}
        </label>
        <p class="contactos__ayuda">
          {{ t('contactos.bajaDialogo.noEscribirAyuda') }}
        </p>

        <template v-if="noEscribirNuncaMas">
          <ASelector
            v-model="motivoDeLaBaja"
            :etiqueta="t('contactos.bajaDialogo.motivo')"
            :opciones="opcionesDeMotivo"
            :ayuda="t('contactos.bajaDialogo.motivoAyuda')"
          />
          <AEntrada
            v-model="notasDeLaBaja"
            :etiqueta="t('contactos.bajaDialogo.notas')"
            :ayuda="t('contactos.bajaDialogo.notasAyuda')"
          />
        </template>
      </div>

      <AAviso
        v-if="errorDelDialogo"
        tono="peligro"
        :titulo="errorDelDialogo.que"
        urgente
      >
        <p>{{ errorDelDialogo.como }}</p>
      </AAviso>

      <template #acciones>
        <ABoton
          variante="secundario"
          @click="cerrar"
        >
          {{ t('comun.cancelar') }}
        </ABoton>
        <ABoton
          variante="peligro"
          :ocupado="contactos.guardando"
          @click="confirmarBaja"
        >
          {{ t('contactos.bajaDialogo.confirmar') }}
        </ABoton>
      </template>
    </AModal>
  </section>
</template>

<style scoped>
.contactos {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  height: 100%;
}

.contactos__cabecera {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--arles-space-4);
}

.contactos__cabecera h1 {
  margin: 0;
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
}

.contactos__acciones {
  display: flex;
  gap: var(--arles-space-3);
}

.contactos__recuento,
.contactos__salvo {
  margin: var(--arles-space-1) 0 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

/* La supresión, separada de lo anterior por una línea: es otra decisión, no
   un detalle más de la baja. */
.contactos__supresion {
  display: grid;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-4);
  padding-top: var(--arles-space-4);
  border-top: var(--arles-border-width) solid var(--arles-border);
}

.contactos__casilla {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
}

.contactos__ayuda {
  margin: calc(-1 * var(--arles-space-2)) 0 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.contactos__cuerpo {
  display: grid;
  /* La proporción áurea, como el resto del producto: la tabla manda y la
     ficha acompaña. */
  grid-template-columns: var(--arles-aureo-fr) 1fr;
  gap: var(--arles-space-4);
  min-height: 0;
  flex: 1;
}

.contactos__tabla {
  min-height: 0;
}

.ficha__empresa {
  margin: 0 0 var(--arles-space-3);
  color: var(--arles-text-muted);
}

.ficha__titulo {
  margin: var(--arles-space-4) 0 var(--arles-space-2);
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
}

.ficha__canales {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
  margin: 0;
  padding: 0;
  list-style: none;
}

.ficha__canal {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  padding: var(--arles-space-2) var(--arles-space-3);
  border-left: var(--arles-space-1) solid transparent;
  border-radius: var(--arles-radius-sm);
  background: var(--arles-surface);
}

/* El principal se marca con la barra de acento, no sólo con la insignia:
   nunca una sola señal (regla 7.3). */
.ficha__canal--principal {
  border-left-color: var(--arles-accent);
}

.ficha__valor {
  display: flex;
  flex: 1;
  flex-direction: column;
  overflow-wrap: anywhere;
}

.ficha__normalizado {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
}

.ficha__acciones {
  display: flex;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-5);
}

@media (max-width: 60rem) {
  .contactos__cuerpo {
    grid-template-columns: 1fr;
  }
}
</style>
