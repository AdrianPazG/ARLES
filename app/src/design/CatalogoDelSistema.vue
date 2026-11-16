<script setup lang="ts">
/**
 * Catálogo del design system.
 *
 * Existe para que **una persona vea las primitivas** antes de que aparezcan en
 * una pantalla de producto. La Fase 1 cerró con «la aplicación abriéndose en
 * una ventana real» sin verificar, y este catálogo es lo que se abre en
 * Windows y en macOS para cerrar ese punto y el escalado del §22.
 *
 * Muestra **cada primitiva en sus cuatro estados** —normal, hover, foco y
 * deshabilitado—, porque el estado que nadie mira es el que llega roto al
 * primer usuario.
 *
 * Es una pantalla de desarrollo: el router sólo la registra fuera de
 * producción. No es una excusa para saltarse ninguna regla —usa los mismos
 * tokens y pasa las mismas comprobaciones—, pero tampoco viaja en el bundle
 * que se instala en la máquina del cliente.
 */
import { computed, ref } from 'vue'

import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  ALogotipo,
  AMenu,
  AModal,
  APestanas,
  ASelector,
  ATabla,
  EstadoCargando,
  EstadoError,
  EstadoExito,
  EstadoVacio,
  type ColumnaDeTabla,
  type Direccion,
} from '@/design/componentes'

const seccion = ref('primitivas')

// ── Estado de los ejemplos ────────────────────────────────────────────────
const texto = ref('ventas@telemetryinsight.mx')
const conError = ref('ventas@@telemetry')
const proveedor = ref('smtp')
const modalAbierto = ref(false)
const modalDestructivo = ref(false)
const seleccionada = ref<string | undefined>(undefined)
const ordenPor = ref('nombre')
const ordenDireccion = ref<Direccion>('asc')

// ── Datos de la tabla ─────────────────────────────────────────────────────
interface FilaDeMuestra {
  id: string
  nombre: string
  correo: string
  enviados: string
  estado: string
}

// 5 000 filas: suficiente para que la virtualización se note al desplazar y
// para que un `<table>` sin virtualizar se viera lento en comparación.
const TOTAL = 5000

const filtro = ref('')

const filas = computed<FilaDeMuestra[]>(() => {
  const estados = ['En cola', 'Aceptado', 'Reintentando', 'Envío no confirmado']
  const lista = Array.from({ length: TOTAL }, (_, i) => ({
    id: `c-${i}`,
    nombre: `Contacto de prueba ${i + 1}`,
    correo: `contacto.${i + 1}@ejemplo.mx`,
    enviados: String((i * 37) % 1200),
    estado: estados[i % estados.length]!,
  }))

  // Un filtro es como un usuario encoge una tabla de verdad, y el encogido
  // brusco es donde un virtualizador mal atado se queda con índices que ya no
  // existen. Aquí está para poder ejercitarlo, no de adorno.
  const aguja = filtro.value.trim().toLowerCase()
  const vistas = aguja
    ? lista.filter(
        (f) =>
          f.nombre.toLowerCase().includes(aguja) ||
          f.correo.toLowerCase().includes(aguja),
      )
    : lista

  const signo = ordenDireccion.value === 'asc' ? 1 : -1
  const clave = ordenPor.value as keyof FilaDeMuestra
  // Las cifras se ordenan como cifras: comparar «1200» y «37» como texto da
  // el orden equivocado, que es el defecto clásico de una tabla de datos.
  return vistas.sort((a, b) =>
    clave === 'enviados'
      ? signo * (Number(a.enviados) - Number(b.enviados))
      : signo * a[clave].localeCompare(b[clave], 'es'),
  )
})

const columnas: ColumnaDeTabla<FilaDeMuestra>[] = [
  { id: 'nombre', titulo: 'Nombre', ancho: '1.4fr', ordenable: true, valor: (f) => f.nombre },
  { id: 'correo', titulo: 'Correo', ancho: '1.6fr', ordenable: true, valor: (f) => f.correo },
  { id: 'enviados', titulo: 'Enviados', ancho: '120px', numerica: true, ordenable: true, valor: (f) => f.enviados },
  { id: 'estado', titulo: 'Estado', ancho: '200px', valor: (f) => f.estado },
]

function ordenar(columna: string, direccion: Direccion) {
  ordenPor.value = columna
  ordenDireccion.value = direccion
}

const SECCIONES = [
  { id: 'primitivas', texto: 'Primitivas' },
  { id: 'estados', texto: 'Los cuatro estados' },
  { id: 'tabla', texto: 'Tabla virtualizada' },
  { id: 'tipografia', texto: 'Tipografía' },
] as const

// Escogidas a propósito: el «1» de Mont mide 331 unidades y el «0» 622, así
// que una columna que mezcla unos y ceros es donde más se nota la diferencia.
const CIFRAS = ['1 240', '37', '918', '1 102', '8 888'] as const

const ESCALA = [
  { clase: 'e-display', nombre: 'Display', detalle: '32 / 40 · Black 900 · sólo bienvenida' },
  { clase: 'e-h1', nombre: 'H1', detalle: '24 / 32 · Bold 700 · título de pantalla' },
  { clase: 'e-h2', nombre: 'H2', detalle: '20 / 28 · Bold 700 · sección' },
  { clase: 'e-h3', nombre: 'H3', detalle: '16 / 24 · SemiBold 600 · subsección' },
  { clase: 'e-body', nombre: 'Body', detalle: '14 / 20 · Regular 400 · por defecto' },
  { clase: 'e-small', nombre: 'Body small', detalle: '13 / 18 · Regular 400 · celdas densas' },
  { clase: 'e-caption', nombre: 'Caption', detalle: '12 / 16 · Regular 400 · ayudas' },
] as const
</script>

<template>
  <section class="catalogo">
    <header class="encabezado">
      <ALogotipo tamano="display" />
      <h1 class="titulo">
        Catálogo del design system
      </h1>
      <p class="nota">
        Cada primitiva en sus cuatro estados. Esta pantalla existe para
        revisarla con los ojos en Windows y en macOS, y a escalado 100 %,
        125 %, 150 % y 200 %.
      </p>
    </header>

    <APestanas
      v-model="seccion"
      :pestanas="SECCIONES"
    >
      <!-- ── Primitivas ──────────────────────────────────────────────── -->
      <template v-if="seccion === 'primitivas'">
        <div class="rejilla">
          <article class="bloque">
            <h2 class="bloque-titulo">
              Botones
            </h2>
            <p class="bloque-nota">
              El primario es <strong>uno por pantalla</strong>. Lleva texto
              oscuro: 11.68:1. Con texto blanco sería 1.52:1.
            </p>
            <div class="fila-de-muestras">
              <ABoton variante="primario">
                Crear campaña
              </ABoton>
              <ABoton variante="secundario">
                Duplicar
              </ABoton>
              <ABoton variante="sutil">
                Cancelar
              </ABoton>
              <ABoton variante="peligro">
                Detener
              </ABoton>
            </div>
            <div class="fila-de-muestras">
              <ABoton
                variante="primario"
                deshabilitado
              >
                Deshabilitado
              </ABoton>
              <ABoton
                variante="secundario"
                ocupado
              >
                Enviando…
              </ABoton>
              <ABoton
                variante="secundario"
                tamano="compacto"
                icono="mas"
              >
                Compacto
              </ABoton>
              <ABoton
                variante="sutil"
                icono="buscar"
                etiqueta="Buscar"
              />
            </div>
          </article>

          <article class="bloque">
            <h2 class="bloque-titulo">
              Campos
            </h2>
            <p class="bloque-nota">
              El error va <strong>debajo, con icono y texto</strong>. Nunca
              sólo con borde rojo.
            </p>
            <div class="columna-de-muestras">
              <AEntrada
                v-model="texto"
                etiqueta="Correo del remitente"
                tipo="email"
                ayuda="Debe ser una cuenta que ya puedas usar para enviar."
              />
              <AEntrada
                v-model="conError"
                etiqueta="Correo del remitente"
                tipo="email"
                error="La dirección de correo no es válida. Revisa que tenga la forma nombre@dominio.com."
              />
              <AEntrada
                model-value="No editable"
                etiqueta="Campo deshabilitado"
                deshabilitado
              />
              <ASelector
                v-model="proveedor"
                etiqueta="Proveedor"
                ayuda="SMTP está disponible hoy; Gmail espera la verificación de Google."
                :opciones="[
                  { valor: 'smtp', texto: 'SMTP' },
                  { valor: 'gmail', texto: 'Gmail (pendiente de verificación)', deshabilitada: true },
                ]"
              />
            </div>
          </article>

          <article class="bloque">
            <h2 class="bloque-titulo">
              Insignias de estado
            </h2>
            <p class="bloque-nota">
              Relleno sólido con texto oscuro, y <strong>siempre con
                icono</strong>: el color solo no comunica.
            </p>
            <div class="fila-de-muestras">
              <AInsignia tono="neutro">
                En cola
              </AInsignia>
              <AInsignia tono="exito">
                Aceptado
              </AInsignia>
              <AInsignia tono="aviso">
                Reintentando
              </AInsignia>
              <AInsignia tono="peligro">
                Falló
              </AInsignia>
              <AInsignia tono="info">
                Enviando
              </AInsignia>
              <AInsignia tono="incierto">
                Envío no confirmado
              </AInsignia>
            </div>
            <p class="bloque-nota">
              «Aceptado», nunca «Entregado» (§65). Y el envío ambiguo de
              ADR-0004 tiene icono propio: no es un éxito ni un fallo.
            </p>
          </article>

          <article class="bloque">
            <h2 class="bloque-titulo">
              Menú y modales
            </h2>
            <div class="fila-de-muestras">
              <AMenu
                etiqueta="Acciones"
                :acciones="[
                  { id: 'duplicar', texto: 'Duplicar campaña', icono: 'mas' },
                  { id: 'exportar', texto: 'Exportar resultados' },
                  { id: 'detener', texto: 'Detener campaña', icono: 'suprimido', destructiva: true },
                ]"
              />
              <ABoton @click="modalAbierto = true">
                Modal normal
              </ABoton>
              <ABoton
                variante="peligro"
                @click="modalDestructivo = true"
              >
                Modal destructivo
              </ABoton>
            </div>
            <p class="bloque-nota">
              En el destructivo, <code>Esc</code> y el clic fuera
              <strong>no cierran</strong>: la decisión tiene que ser explícita
              en las dos direcciones.
            </p>
          </article>

          <article class="bloque ancho">
            <h2 class="bloque-titulo">
              Avisos
            </h2>
            <div class="columna-de-muestras">
              <AAviso
                tono="info"
                titulo="¿Por qué «aceptado» y no «entregado»?"
              >
                Tu proveedor de correo confirma que recibió el mensaje y que
                intentará entregarlo, pero no informa si llegó al buzón del
                destinatario. ARLES sólo muestra lo que puede verificar.
              </AAviso>
              <AAviso
                tono="aviso"
                titulo="La detección de rebotes es parcial en esta versión"
              >
                ARLES detecta los rechazos que el servidor de destino comunica
                durante el envío, pero no los que llegan después como correo a
                tu bandeja.
              </AAviso>
            </div>
          </article>
        </div>
      </template>

      <!-- ── Los cuatro estados ──────────────────────────────────────── -->
      <template v-else-if="seccion === 'estados'">
        <div class="rejilla">
          <article class="bloque">
            <h2 class="bloque-titulo">
              Vacío
            </h2>
            <EstadoVacio
              :titulo="$t('vacio.campanas.titulo')"
              :cuerpo="$t('vacio.campanas.cuerpo')"
            >
              <template #accion>
                <ABoton variante="primario">
                  {{ $t('vacio.campanas.accion') }}
                </ABoton>
              </template>
            </EstadoVacio>
          </article>

          <article class="bloque">
            <h2 class="bloque-titulo">
              Cargando
            </h2>
            <p class="bloque-nota">
              Esqueleto con la forma del contenido, no un girador. A partir de
              los 2 s aparece la explicación.
            </p>
            <EstadoCargando
              :lineas="4"
              explicacion="Leyendo 48 200 contactos del archivo. Esto puede tardar un momento."
            />
          </article>

          <article class="bloque">
            <h2 class="bloque-titulo">
              Error
            </h2>
            <EstadoError
              que="No se pudo conectar con smtp.gmail.com."
              como="Revisa tu conexión a internet y que el puerto 587 no esté bloqueado."
              salvo="Tu campaña está en pausa, no cancelada. Los 1 240 envíos pendientes se reanudarán al restablecerse la conexión."
            >
              <template #acciones>
                <ABoton variante="secundario">
                  Reintentar ahora
                </ABoton>
              </template>
            </EstadoError>
          </article>

          <article class="bloque">
            <h2 class="bloque-titulo">
              Éxito
            </h2>
            <p class="bloque-nota">
              Breve y con una cifra. Sin celebraciones (§94).
            </p>
            <EstadoExito
              titulo="Importación terminada."
              detalle="1 240 contactos añadidos · 38 duplicados omitidos · 4 direcciones rechazadas"
            >
              <template #siguiente>
                <ABoton
                  variante="secundario"
                  tamano="compacto"
                >
                  Ver los rechazados
                </ABoton>
              </template>
            </EstadoExito>
          </article>
        </div>
      </template>

      <!-- ── Tabla ───────────────────────────────────────────────────── -->
      <template v-else-if="seccion === 'tabla'">
        <div class="barra-de-tabla">
          <AEntrada
            v-model="filtro"
            etiqueta="Filtrar"
            :ayuda="`${filas.length.toLocaleString('es-MX')} de ${TOTAL.toLocaleString('es-MX')} filas`"
            marcador="nombre o correo"
          />
        </div>
        <p class="bloque-nota">
          {{ TOTAL.toLocaleString('es-MX') }} filas, virtualizadas. Flechas,
          <code>Inicio</code>, <code>Fin</code>, <code>Re Pág</code>,
          <code>Av Pág</code> y <code>Enter</code>. La selección se marca con
          fondo <strong>y</strong> barra de acento.
        </p>
        <div
          v-if="filas.length === 0"
          class="caja-de-tabla"
        >
          <EstadoVacio
            titulo="Ningún contacto coincide con el filtro."
            cuerpo="Prueba con menos letras, o borra el filtro para volver a ver la lista completa."
          >
            <template #accion>
              <ABoton @click="filtro = ''">
                Borrar el filtro
              </ABoton>
            </template>
          </EstadoVacio>
        </div>

        <div
          v-else
          class="caja-de-tabla"
        >
          <ATabla
            v-model:seleccionada="seleccionada"
            etiqueta="Contactos de muestra"
            :filas="filas"
            :columnas="columnas"
            :orden-por="ordenPor"
            :orden-direccion="ordenDireccion"
            @ordenar="ordenar"
          />
        </div>
      </template>

      <!-- ── Tipografía ──────────────────────────────────────────────── -->
      <template v-else>
        <div class="escala">
          <div
            v-for="paso in ESCALA"
            :key="paso.clase"
            class="paso"
          >
            <p :class="['muestra', paso.clase]">
              ARLES RELAY · 1 240 enviados
            </p>
            <p class="paso-detalle">
              <strong>{{ paso.nombre }}</strong> · {{ paso.detalle }}
            </p>
          </div>
        </div>

        <AAviso
          tono="aviso"
          titulo="Sin corte Medium 500"
        >
          El kit disponible salta de Regular (asta de 87 por mil) a SemiBold
          (115). Las cifras usan Regular 400 con figuras tabulares. Se verificó
          que los cuatro cortes declaran <code>tnum</code>.
        </AAviso>

        <div class="comparativa">
          <div>
            <p class="etiqueta-de-comparativa">
              Con figuras tabulares — cada dígito ocupa lo mismo
            </p>
            <ul class="cifras tabulares">
              <li
                v-for="n in CIFRAS"
                :key="n"
              >
                {{ n }}
              </li>
            </ul>
          </div>

          <div>
            <p class="etiqueta-de-comparativa">
              Sin figuras tabulares — el «1» ocupa la mitad que el «0»
            </p>
            <ul class="cifras">
              <li
                v-for="n in CIFRAS"
                :key="n"
              >
                {{ n }}
              </li>
            </ul>
          </div>
        </div>

        <p class="bloque-nota medido">
          Medido en Mont Regular a 16 px: «1240» mide 36.61 px con figuras
          tabulares y 32.27 px sin ellas; «1102», 36.61 frente a 28.95. Siete
          píxeles y pico de diferencia entre dos cifras de cuatro dígitos —
          suficiente para que una columna de miles no alinee y para que el
          número baile cada vez que se actualiza en vivo.
        </p>
      </template>
    </APestanas>

    <AModal
      :abierto="modalAbierto"
      titulo="Duplicar campaña"
      @cerrar="modalAbierto = false"
    >
      Se creará una copia en borrador con la misma plantilla y los mismos
      límites. La audiencia no se copia.

      <template #acciones>
        <ABoton
          variante="sutil"
          @click="modalAbierto = false"
        >
          {{ $t('comun.cancelar') }}
        </ABoton>
        <ABoton
          variante="primario"
          @click="modalAbierto = false"
        >
          Duplicar
        </ABoton>
      </template>
    </AModal>

    <AModal
      :abierto="modalDestructivo"
      titulo="Detener la campaña"
      destructiva
      @cerrar="modalDestructivo = false"
    >
      Detener una campaña no se puede deshacer. Los mensajes ya aceptados por
      el proveedor seguirán su curso.

      <!-- Un número concreto, no «¿estás seguro?» (DESIGN_SYSTEM §6). -->
      <template #detalle>
        <strong>1 240 mensajes</strong> quedarán sin enviar.
      </template>

      <template #acciones>
        <ABoton
          variante="sutil"
          @click="modalDestructivo = false"
        >
          Seguir enviando
        </ABoton>
        <ABoton
          variante="peligro"
          @click="modalDestructivo = false"
        >
          Detener la campaña
        </ABoton>
      </template>
    </AModal>
  </section>
</template>

<style scoped>
.catalogo {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
  padding-bottom: var(--arles-space-8);
}

.encabezado {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--arles-space-3);
}

.titulo {
  margin: 0;
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.nota,
.bloque-nota {
  margin: 0;
  max-width: 68ch;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

/* A escala alta de Windows el viewport CSS se encoge: a 200 % en una pantalla
   de 1920 quedan 960 px, y descontando la navegación el contenido baja de 720.
   Dos columnas ahí no se leen. Por debajo de 900 px de ventana pasa a una. */
@media (max-width: 900px) {
  .rejilla {
    grid-template-columns: minmax(0, 1fr);
  }
}

.rejilla {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  /* `start`: sin esto los dos bloques de una fila se estiran al alto del más
     largo y el corto queda con medio panel vacío debajo. */
  align-items: start;
  gap: var(--arles-space-4);
}

.bloque {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
  padding: var(--arles-space-4);
  background: var(--arles-surface);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-lg);
}

.ancho {
  grid-column: 1 / -1;
}

.bloque-titulo {
  margin: 0;
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
  font-weight: var(--arles-font-weight-semibold);
}

.fila-de-muestras {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--arles-space-3);
}

.columna-de-muestras {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.barra-de-tabla {
  max-width: 380px;
  margin-bottom: var(--arles-space-4);
}

.caja-de-tabla {
  /* Altura fija: la tabla virtualizada necesita un contenedor con altura
     conocida, porque es la que decide cuántas filas hay que renderizar. */
  height: 520px;
}

.escala {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  margin-bottom: var(--arles-space-5);
}

.paso {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  padding-bottom: var(--arles-space-3);
  border-bottom: var(--arles-border-width) solid var(--arles-border);
}

.muestra {
  margin: 0;
  font-variant-numeric: tabular-nums;
}

.e-display {
  font-size: var(--arles-font-size-display);
  line-height: var(--arles-line-height-display);
  font-weight: var(--arles-font-weight-black);
}

.e-h1 {
  font-size: var(--arles-font-size-h1);
  line-height: var(--arles-line-height-h1);
  font-weight: var(--arles-font-weight-bold);
}

.e-h2 {
  font-size: var(--arles-font-size-h2);
  line-height: var(--arles-line-height-h2);
  font-weight: var(--arles-font-weight-bold);
}

.e-h3 {
  font-size: var(--arles-font-size-h3);
  line-height: var(--arles-line-height-h3);
  font-weight: var(--arles-font-weight-semibold);
}

.e-body {
  font-size: var(--arles-font-size-body);
  line-height: var(--arles-line-height-body);
}

.e-small {
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-small);
}

.e-caption {
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
}

.paso-detalle {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
  line-height: var(--arles-line-height-caption);
}

.comparativa {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 260px));
  gap: var(--arles-space-2) var(--arles-space-5);
  margin-top: var(--arles-space-5);
}

.medido {
  margin-top: var(--arles-space-4);
}

.etiqueta-de-comparativa {
  margin: 0;
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-caption);
}

.cifras {
  margin: 0;
  padding: var(--arles-space-3);
  list-style: none;
  background: var(--arles-surface);
  border-radius: var(--arles-radius-md);
  font-size: var(--arles-font-size-h3);
  /* Alineadas a la derecha, que es como van en una tabla: ahí la diferencia
     se ve en las columnas internas, no en el borde. */
  text-align: right;
  font-variant-numeric: proportional-nums;
}

.tabulares {
  font-variant-numeric: tabular-nums;
}

code {
  padding: 1px 4px;
  background: var(--arles-surface-raised);
  border-radius: var(--arles-radius-sm);
  font-size: 0.92em;
}
</style>
