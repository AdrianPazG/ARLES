<script setup lang="ts">
/**
 * Remitentes: las cuentas desde las que ARLES envía (entrega 5.3).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ «PROBAR CONEXIÓN» ESTÁ, Y POR QUÉ NO ES OBLIGATORIO
 *
 * El botón existe porque una cuenta mal configurada, sin él, se descubre en la
 * primera campaña — con los contactos ya en cola. Probar tarda dos segundos y
 * responde por hoy.
 *
 * Pero no bloquea el guardado, y eso es deliberado: **guardar prueba también**,
 * y si la prueba falla la cuenta se guarda igualmente, marcada como «hay que
 * configurarla». Lo contrario obligaría a volver a teclear seis campos y una
 * contraseña de dieciséis caracteres porque el servidor estuvo caído dos
 * minutos.
 *
 * Así que el botón de probar es para diagnosticar mientras se configura, no un
 * peaje antes de guardar.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LO QUE ESTA PANTALLA NO DECIDE
 *
 * **No valida.** Los campos los revisa el núcleo, y lo que se pinta en rojo es
 * lo que el núcleo devolvió. Si la validación viviera aquí habría dos reglas
 * que mantener iguales, y el día que divergieran esta pantalla daría por bueno
 * un servidor que el núcleo rechaza.
 *
 * **No guarda la contraseña.** Vive en el campo mientras se escribe, viaja en
 * la llamada y se borra al cerrar el formulario. No pasa por la tienda ni por
 * `localStorage` (ADR-0011).
 * ─────────────────────────────────────────────────────────────────────────
 */
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { resolverError } from '@/app/errores'
import {
  borradorVacio,
  contrasenaParaEnviar,
  proveedorDe,
  useRemitentesStore,
  type BorradorDeCuenta,
  type Remitente,
} from '@/app/stores/remitentes'
import { useActividadStore, type Cupo } from '@/app/stores/actividad'
import {
  AAviso,
  ABoton,
  AEntrada,
  AInsignia,
  AModulo,
  ASelector,
  EstadoCargando,
  EstadoVacio,
} from '@/design/componentes'

const store = useRemitentesStore()
// ── EL CUPO SALE DEL COMANDO DE ACTIVIDAD, Y NO DE UNO NUEVO ──────────────
//
// `actividad` ya devuelve el cupo de cada remitente con las campañas que se lo
// reparten — lo construyó la 7.3. Añadir un comando sólo para esta pantalla
// sería un cruce más de la frontera y, sobre todo, **superficie de ataque**:
// cada comando registrado es una puerta, y una que casi nadie usa es una que
// nadie vigila. La regla 1 de la frontera pide comandos gruesos.
const cifras = useActividadStore()
const { t, te } = useI18n()

onMounted(() => {
  void store.cargar()
  void cifras.cargar()
})

/**
 * El cupo real de un remitente, o `undefined` mientras no haya cargado.
 *
 * Se busca por dirección porque es lo que las dos listas comparten: la de
 * remitentes trae `remitente` y la de cupos trae `direccion`, y el
 * identificador de la cuenta **no cruza la frontera** en la primera.
 */
function cupoDe(direccion: string): Cupo | undefined {
  return cifras.remitentes.find((c) => c.direccion === direccion)
}

/**
 * Lo que el buzón manda al día, ya sumado.
 *
 * `topeDiario` es `null` cuando ninguna campaña en marcha usa esta cuenta: no
 * manda nada, que no es lo mismo que mandar cero por decisión (D-13).
 */
function alDia(c: Cupo): string {
  return c.topeDiario === null
    ? t('remitentes.cupo.quieto')
    : t('remitentes.cupo.alDia', {
        total: c.topeDiario.toLocaleString('es-MX'),
      })
}

/** De dónde sale esa cifra: las campañas que se reparten el buzón. */
function deQuienEs(c: Cupo): string {
  if (c.campanas.length === 0) return ''
  return c.hayReparto
    ? t('remitentes.cupo.reparto', { campanas: c.campanas.join(', ') })
    : t('remitentes.cupo.unaSola', { campana: c.campanas[0] })
}

/**
 * El tono del semáforo. **El juicio lo trae el núcleo**, aquí sólo se traduce
 * a un tono del sistema de diseño.
 *
 * Los umbrales —60 y 150— los puso Dirección y viven en
 * `arles_core::ritmo::RiesgoDelBuzon`. Escribirlos aquí habría creado la
 * segunda copia de una regla de producto, que es el defecto que la auditoría A
 * del §140 encontró con `AVISO_DIARIO`.
 */
function tonoDelRiesgo(clave: string): 'exito' | 'aviso' | 'peligro' {
  if (clave.endsWith('arriesgado')) return 'peligro'
  if (clave.endsWith('vigilar')) return 'aviso'
  return 'exito'
}

/** El formulario abierto: `null` cuando no hay ninguno. */
const borrador = ref<BorradorDeCuenta | null>(null)
/** El id de la cuenta que se está editando, o `null` si es nueva. */
const editando = ref<string | null>(null)
/**
 * La contraseña, sólo mientras el formulario está abierto.
 *
 * Es un `ref` de esta pantalla y no de la tienda a propósito: la tienda vive
 * mientras vive la aplicación, y este valor tiene que morir al cerrar.
 */
const contrasena = ref('')
/** La guía de Gmail, plegada por defecto. */
const ayudaAbierta = ref(false)

/**
 * El acuse de «Copiada», y el aviso si no se pudo.
 *
 * ── POR QUÉ UN BOTÓN DE COPIAR Y NO UN ENLACE QUE ABRA EL NAVEGADOR ───────
 *
 * Sería más cómodo un enlace, y **cuesta un permiso nuevo**. ARLES declara
 * hoy `core:default` y nada más, y la auditoría B del §140 lo recogió como
 * uno de los puntos que salieron limpios: cada permiso que se añade es
 * superficie de ataque que alguien tiene que vigilar.
 *
 * Copiar y pegar en el navegador son dos gestos en vez de uno. Abrir el
 * navegador desde la aplicación es un plugin, una dependencia nueva que pasar
 * por `cargo deny`, y un permiso que habría que acotar a una lista de
 * direcciones. **Se queda como está hasta que Dirección decida que el gesto
 * de más lo merece**, y queda escrito aquí para que sea una decisión y no un
 * olvido.
 */
const copiada = ref(false)
const fallóAlCopiar = ref(false)

/**
 * Dónde se crea la contraseña de este proveedor, o cadena vacía.
 *
 * Sale del catálogo y no de un `if` sobre Gmail: el día que se sepa la de
 * Outlook, se escribe en `PROVEEDORES` y esta pantalla no se toca. Un `if`
 * aquí sería el mismo dato en dos sitios.
 */
const donde = computed(() => proveedor.value?.dondeSeCreaLaContrasena ?? '')

async function copiarLaDireccion(donde: string): Promise<void> {
  fallóAlCopiar.value = false
  try {
    await navigator.clipboard.writeText(donde)
    copiada.value = true
    // El acuse se borra solo: un «Copiada» que se queda para siempre deja de
    // decir que acaba de pasar algo.
    window.setTimeout(() => {
      copiada.value = false
    }, 2000)
  } catch {
    // El portapapeles puede estar negado o no existir. La dirección sigue en
    // pantalla y se puede seleccionar a mano, así que esto no bloquea nada —
    // pero se dice, porque un botón que no hace nada se lee como una avería.
    fallóAlCopiar.value = true
  }
}
/** La configuración avanzada —servidor, puerto, cifrado—, plegada por defecto. */
const avanzadaAbierta = ref(false)

/**
 * El proveedor que corresponde a la dirección escrita, o `null`.
 *
 * ── POR QUÉ ESTO EXISTE (R-01, 14/10/2026) ──────────────────────────────────
 *
 * El formulario pedía siete campos y Dirección no pudo dar de alta su cuenta
 * (H-03): escribió un usuario que no era la dirección y una contraseña que no
 * era la de aplicación, y **nada se lo dijo** hasta que la conexión falló con
 * un error de cifrado que no apuntaba a ninguna de las dos cosas.
 *
 * Ahora servidor, puerto y cifrado se deducen del dominio en cuanto se escribe
 * la dirección. Lo único que se teclea es **correo y contraseña**.
 */
const proveedor = computed(() => (borrador.value ? proveedorDe(borrador.value.remitente) : null))

/**
 * Aviso si la contraseña no tiene el largo que el proveedor exige.
 *
 * Google da **dieciséis letras** para una contraseña de aplicación, y meter la
 * contraseña normal de la cuenta es el error más común: hoy sólo se descubre
 * cuando el servidor la rechaza, con un mensaje que habla de autenticación y no
 * de esto. Se avisa **antes** de intentar conectar, y es un aviso y no un
 * bloqueo: el largo podría cambiar y nadie quiere un formulario que se niega a
 * guardar algo correcto.
 */
const avisoDeContrasena = computed(() => {
  const largo = proveedor.value?.largoDeContrasena
  if (!largo || contrasena.value.length === 0) return ''
  const sinEspacios = contrasena.value.replace(/\s/g, '')
  if (sinEspacios.length === largo) return ''
  return t('remitentes.avisoLargoContrasena', {
    proveedor: proveedor.value?.nombre ?? '',
    esperado: largo,
    escrito: sinEspacios.length,
  })
})

/**
 * Al escribir la dirección: rellena lo que se puede deducir.
 *
 * El **usuario** se iguala a la dirección porque es lo correcto en Gmail,
 * Outlook y casi todo lo demás — y porque dejarlo vacío invita a inventarse un
 * nombre de usuario, que es lo que pasó. Sigue siendo editable: hay servidores
 * donde el usuario es otra cosa.
 *
 * Si el dominio **no** se reconoce, no se toca nada y se abre la configuración
 * avanzada: ARLES no se inventa un servidor que no sabe.
 */
function alEscribirLaDireccion(): void {
  const b = borrador.value
  if (!b) return

  const p = proveedorDe(b.remitente)
  if (p) {
    b.servidor = p.servidor
    b.puerto = p.puerto
    b.cifrado = p.cifrado
    avanzadaAbierta.value = false
  } else if (b.remitente.includes('@') && b.remitente.split('@')[1]) {
    avanzadaAbierta.value = true
  }

  // El usuario sigue a la dirección mientras nadie lo haya cambiado a mano.
  if (!usuarioTocado.value) b.usuario = b.remitente
}

/** Si alguien editó el usuario, deja de seguir a la dirección. */
const usuarioTocado = ref(false)

/**
 * Qué cuenta tiene abierto el formulario de «mandar un correo de prueba».
 *
 * Una a la vez, y no una bandera por fila: con dos abiertos a la vez sería
 * posible escribir una dirección en uno y pulsar en el otro.
 */
const probandoEnvioDe = ref<string | null>(null)
/** Qué cuenta tiene abierto el bloque de los registros del dominio (7.5). */
const mirandoDominioDe = ref<string | null>(null)
/** La dirección a la que mandar la prueba. Vive aquí y se vacía al cerrar. */
const aDonde = ref('')

const cifrados = computed(() => [
  { valor: 'starttls', texto: t('correo.cifrado.starttls') },
  { valor: 'directo', texto: t('correo.cifrado.directo') },
])

const nota = computed(() => {
  if (!store.cargada || store.remitentes.length === 0) return ''
  return store.porConfigurar > 0
    ? t(
        'remitentes.notaConPendientes',
        { cuantas: store.remitentes.length, pendientes: store.porConfigurar },
        store.remitentes.length,
      )
    : t('remitentes.nota', { cuantas: store.remitentes.length }, store.remitentes.length)
})

const errorGeneral = computed(() =>
  store.errorGeneral
    ? resolverError({ clave: store.errorGeneral, detalle: '' })
    : null,
)

/**
 * El diagnóstico de la última prueba, con sus tres partes.
 *
 * Se resuelve con el mismo mecanismo que un error aunque «correcto» no lo sea:
 * las tres partes —qué pasó, qué hacer, qué está a salvo— valen igual para una
 * buena noticia, y tener dos formatos obligaría a la pantalla a decidir cuál
 * usar antes de saber qué pasó.
 */
const diagnostico = computed(() =>
  store.diagnostico
    ? resolverError({ clave: store.diagnostico, detalle: '' })
    : null,
)

/** El texto del estado de una cuenta. Si falta, se dice el estado crudo. */
function estado(clave: string): string {
  const ruta = `remitentes.estado.${clave}`
  return te(ruta) ? t(ruta) : clave
}

/**
 * El texto del error de un campo, o cadena vacía si no lo hay.
 *
 * Cadena vacía y no `undefined`: con `exactOptionalPropertyTypes` activo,
 * pasar `undefined` a una propiedad opcional es un error de tipo, y es la
 * convención que ya siguen las demás pantallas.
 */
function errorDe(campo: string): string {
  const clave = store.erroresDeCampo[campo]
  if (!clave) return ''
  return te(clave) ? t(clave) : t('correo.error.generico')
}

function abrirNuevo(): void {
  store.limpiarAvisos()
  borrador.value = borradorVacio()
  editando.value = null
  contrasena.value = ''
  ayudaAbierta.value = false
}

function abrirEdicion(r: Remitente): void {
  store.limpiarAvisos()
  borrador.value = {
    servidor: r.servidor,
    puerto: String(r.puerto),
    cifrado: r.cifrado,
    usuario: r.usuario,
    remitente: r.remitente,
    nombreVisible: r.nombreVisible ?? '',
  }
  editando.value = r.id
  // Vacía: la guardada sigue sirviendo. Se escribe sólo para cambiarla.
  contrasena.value = ''
  ayudaAbierta.value = false
}

function cerrar(): void {
  borrador.value = null
  editando.value = null
  // Lo primero que se borra, y no al final: si algo fallara más abajo, la
  // contraseña ya no está en memoria.
  contrasena.value = ''
  store.limpiarAvisos()
}

// Las dos salidas de la contraseña pasan por `contrasenaParaEnviar`, y las
// dos tienen que hacerlo: probar con una y guardar otra daría una prueba en
// verde y una cuenta que no envía.
async function probar(): Promise<void> {
  if (!borrador.value) return
  await store.probar(
    borrador.value,
    contrasenaParaEnviar(proveedor.value, contrasena.value),
    editando.value,
  )
}

async function guardar(): Promise<void> {
  if (!borrador.value) return
  const guardado = await store.guardar(
    borrador.value,
    contrasenaParaEnviar(proveedor.value, contrasena.value),
    editando.value,
  )
  // Se cierra aunque la prueba no haya pasado: la cuenta quedó escrita, y
  // dejar el formulario abierto haría creer que no se guardó nada. El
  // diagnóstico se queda en pantalla, debajo de la lista.
  if (guardado) {
    borrador.value = null
    editando.value = null
    contrasena.value = ''
  }
}

/** Probar una cuenta ya guardada, sin abrir el formulario. */
/**
 * Abre —o cierra— el formulario de envío de prueba de una cuenta.
 *
 * Al abrirlo se propone la propia dirección de la cuenta. Es la que casi
 * siempre se quiere: mandarse la prueba a uno mismo es la única forma de ver
 * si llegó, y escribirla a mano es un paso que no aporta nada. Se puede
 * cambiar.
 */
/**
 * Abre o cierra los registros del dominio de una cuenta.
 *
 * Sólo pregunta al DNS la **primera** vez: el store cachea una hora, y volver
 * a abrir el bloque no debe costar tres consultas de red. Quien acabe de
 * cambiar su DNS tiene el botón de «volver a comprobar» dentro.
 */
function alternarDominio(r: Remitente): void {
  if (mirandoDominioDe.value === r.id) {
    mirandoDominioDe.value = null
    return
  }
  mirandoDominioDe.value = r.id
  if (!store.registros[r.id]) void store.mirarElDominio(r.id)
}

/**
 * El tono de la insignia de un hallazgo.
 *
 * `enObservacion` va en AVISO y no en éxito, y es la decisión que todo este
 * trabajo defiende: un DMARC en `p=none` está bien puesto y no detiene a
 * nadie. Pintarlo verde diría que el dominio está protegido.
 */
function tonoDelHallazgo(hallazgo: string): 'exito' | 'aviso' | 'peligro' {
  if (hallazgo === 'enOrden') return 'exito'
  if (hallazgo === 'enObservacion') return 'aviso'
  return 'peligro'
}

function alternarEnvio(r: Remitente): void {
  if (probandoEnvioDe.value === r.id) {
    probandoEnvioDe.value = null
    aDonde.value = ''
    return
  }
  probandoEnvioDe.value = r.id
  aDonde.value = r.remitente
  store.limpiarAvisos()
}

async function mandarPrueba(r: Remitente): Promise<void> {
  await store.mandarCorreoDePrueba(r.id, aDonde.value)
}

/** El texto de por qué no salió el correo, si no salió. */
const problemaDelEnvio = computed(() => {
  const p = store.resultadoDelEnvio
  return p && !p.aceptado && p.problema
    ? resolverError({ clave: p.problema, detalle: '' })
    : null
})

async function probarGuardada(r: Remitente): Promise<void> {
  await store.probar(
    {
      servidor: r.servidor,
      puerto: String(r.puerto),
      cifrado: r.cifrado,
      usuario: r.usuario,
      remitente: r.remitente,
      nombreVisible: r.nombreVisible ?? '',
    },
    // Vacía: el núcleo usa la que está en el llavero.
    '',
    r.id,
  )
}
</script>

<template>
  <div class="remitentes">
    <AModulo
      :titulo="t('remitentes.titulo')"
      :nota="nota"
      principal
      nivel="h1"
    >
      <p class="remitentes__explicacion">
        {{ t('remitentes.explicacion') }}
      </p>

      <AAviso
        v-if="errorGeneral"
        tono="peligro"
      >
        <strong>{{ errorGeneral.que }}</strong>
        <span>{{ errorGeneral.como }}</span>
        <span>{{ errorGeneral.salvo }}</span>
      </AAviso>

      <!-- El diagnóstico de la última prueba. Tono según lo que se averiguó:
           una cuenta que funciona no se anuncia con el mismo color que una que
           no, y el color es lo primero que se ve. -->
      <AAviso
        v-if="diagnostico"
        :tono="store.laCuentaSirve ? 'exito' : 'peligro'"
      >
        <strong>{{ diagnostico.que }}</strong>
        <span>{{ diagnostico.como }}</span>
        <span>{{ diagnostico.salvo }}</span>
      </AAviso>

      <EstadoCargando
        v-if="store.cargando"
        :mensaje="t('remitentes.cargando')"
      />

      <template v-else>
        <!-- ── La lista ───────────────────────────────────────────────── -->
        <EstadoVacio
          v-if="!store.hayRemitentes && !borrador"
          :titulo="t('remitentes.vacioTitulo')"
          :cuerpo="t('remitentes.vacioCuerpo')"
        />

        <ul
          v-else-if="store.hayRemitentes"
          class="remitentes__lista"
        >
          <li
            v-for="r in store.remitentes"
            :key="r.id"
            class="remitentes__fila"
          >
            <div class="remitentes__quien">
              <span class="remitentes__direccion">{{ r.remitente }}</span>
              <span
                v-if="r.nombreVisible"
                class="remitentes__nombre"
              >{{ r.nombreVisible }}</span>
              <span class="remitentes__servidor">
                {{ r.servidor }}:{{ r.puerto }} · {{ t(`correo.cifrado.${r.cifrado}`) }}
              </span>
              <span class="remitentes__limites">
                {{ t('remitentes.limites', { diario: r.limiteDiario, porHora: r.limitePorHora }) }}
              </span>
            </div>

            <!--
              EL CUPO REAL DEL BUZÓN (D-14). Es la cifra que Dirección pidió
              ver, y desde la D-13 es la única que ya no tiene techo: los
              límites de las campañas que comparten un remitente se suman.

              Va en su propia columna y no junto a «Propone …» a propósito: son
              dos números parecidos que dicen cosas distintas —lo que el
              asistente propone y lo que de verdad sale—, y uno al lado del
              otro se leen como una repetición.
            -->
            <div
              v-if="cupoDe(r.remitente)"
              class="remitentes__cupo"
            >
              <span class="remitentes__cupo-rotulo">
                {{ t('remitentes.cupo.titulo') }}
              </span>
              <span class="remitentes__cupo-cifra">
                {{ alDia(cupoDe(r.remitente)!) }}
              </span>
              <AInsignia :tono="tonoDelRiesgo(cupoDe(r.remitente)!.riesgo)">
                {{ t(cupoDe(r.remitente)!.riesgo) }}
              </AInsignia>
              <span class="remitentes__cupo-detalle">
                {{ t(`${cupoDe(r.remitente)!.riesgo}Detalle`) }}
              </span>
              <span
                v-if="deQuienEs(cupoDe(r.remitente)!)"
                class="remitentes__cupo-campanas"
              >
                {{ deQuienEs(cupoDe(r.remitente)!) }}
              </span>
              <span class="remitentes__cupo-hoy">
                {{ t('remitentes.cupo.gastadoHoy', {
                  gastado: cupoDe(r.remitente)!.gastadoHoy.toLocaleString('es-MX'),
                }) }}
              </span>
            </div>

            <div class="remitentes__estado">
              <AInsignia :tono="r.estado === 'connected' ? 'exito' : 'aviso'">
                {{ estado(r.estado) }}
              </AInsignia>
            </div>

            <div class="remitentes__acciones">
              <ABoton
                variante="sutil"
                tamano="compacto"
                :ocupado="store.probando"
                @click="probarGuardada(r)"
              >
                {{ t('remitentes.probarDeNuevo') }}
              </ABoton>
              <ABoton
                variante="sutil"
                tamano="compacto"
                @click="alternarEnvio(r)"
              >
                {{ t('remitentes.envioDePrueba.titulo') }}
              </ABoton>
              <ABoton
                variante="sutil"
                tamano="compacto"
                :ocupado="store.mirandoElDominio === r.id"
                @click="alternarDominio(r)"
              >
                {{ t('entregabilidad.titulo') }}
              </ABoton>
              <ABoton
                variante="secundario"
                tamano="compacto"
                @click="abrirEdicion(r)"
              >
                {{ t('remitentes.editar') }}
              </ABoton>
            </div>

            <!-- ── Los registros del dominio (7.5) ───────────────────────
                 Va aquí y no en una pantalla aparte porque es una propiedad
                 DE ESTA CUENTA: el dominio que hay que autorizar es el de su
                 dirección, y dos remitentes de dominios distintos tienen
                 veredictos distintos.

                 Y no bloquea nada (§154): se enseña, se explica qué poner, y
                 lo arregla quien administre el dominio. -->
            <div
              v-if="mirandoDominioDe === r.id"
              class="remitentes__dominio"
            >
              <p class="remitentes__ayuda">
                {{ t('entregabilidad.explicacion') }}
              </p>

              <p
                v-if="store.mirandoElDominio === r.id"
                class="remitentes__ayuda"
              >
                {{ t('comun.cargando') }}
              </p>

              <template v-else-if="store.registros[r.id]">
                <ul class="remitentes__registros">
                  <li
                    v-for="reg in store.registros[r.id]!.revisiones"
                    :key="reg.registro"
                    class="remitentes__registro"
                  >
                    <span class="remitentes__registro-nombre">
                      {{ t(`entregabilidad.${reg.registro}`) }}
                    </span>
                    <AInsignia :tono="tonoDelHallazgo(reg.hallazgo)">
                      {{ t(`entregabilidad.${reg.hallazgo}`, { cuantos: reg.cuantos ?? 0 }) }}
                    </AInsignia>
                    <span class="remitentes__registro-detalle">
                      {{ t(`entregabilidad.${reg.hallazgo}Detalle`, { cuantos: reg.cuantos ?? 0 }) }}
                    </span>

                    <!-- Qué poner, sólo cuando hay algo que arreglar. -->
                    <div
                      v-if="reg.quePoner"
                      class="remitentes__que-poner"
                    >
                      <span class="remitentes__que-poner-rotulo">
                        {{ t('entregabilidad.quePoner') }}
                      </span>
                      <span class="remitentes__que-poner-dato">
                        {{ t('entregabilidad.quePonerNombre') }}: {{ reg.quePoner.nombre }}
                      </span>
                      <span class="remitentes__que-poner-dato">
                        {{ t('entregabilidad.quePonerTipo') }}: {{ reg.quePoner.tipo }}
                      </span>
                      <span
                        v-if="reg.quePoner.valor"
                        class="remitentes__que-poner-valor"
                      >{{ reg.quePoner.valor }}</span>
                      <!-- Y cuando ARLES no puede saberlo, lo dice en vez de
                           inventarse un valor que alguien copiaría. -->
                      <span
                        v-else
                        class="remitentes__que-poner-dato"
                      >{{ t('entregabilidad.quePonerValorNoSeSabe') }}</span>
                    </div>
                  </li>

                  <!-- Los que no se pudieron mirar. Omitido no es aprobado. -->
                  <li
                    v-for="clave in store.registros[r.id]!.sinComprobar"
                    :key="clave"
                    class="remitentes__registro"
                  >
                    <span class="remitentes__registro-nombre">{{ t(clave) }}</span>
                    <AInsignia tono="neutro">
                      {{ t('entregabilidad.sinComprobar') }}
                    </AInsignia>
                    <span class="remitentes__registro-detalle">
                      {{ t('entregabilidad.sinComprobarDetalle') }}
                    </span>
                  </li>
                </ul>

                <ABoton
                  variante="sutil"
                  tamano="compacto"
                  @click="store.mirarElDominio(r.id, true)"
                >
                  {{ t('entregabilidad.volverAComprobar') }}
                </ABoton>
              </template>
            </div>

            <!-- ── Mandar un correo de prueba ────────────────────────────
                 Va plegado y dentro de la fila de su cuenta: es una acción
                 de ESA cuenta, y sacarlo a un formulario aparte obligaría a
                 volver a elegir cuál.

                 Y lleva su propia advertencia porque hace algo que ningún
                 otro botón de esta pantalla hace: **manda un correo de
                 verdad**. «Probar conexión», al lado, no manda nada. -->
            <div
              v-if="probandoEnvioDe === r.id"
              class="remitentes__envio"
            >
              <p class="remitentes__ayuda">
                {{ t('remitentes.envioDePrueba.explicacion') }}
              </p>

              <AEntrada
                v-model="aDonde"
                :etiqueta="t('remitentes.envioDePrueba.aDonde')"
                :ayuda="t('remitentes.envioDePrueba.aDondeAyuda')"
                tipo="email"
              />

              <ABoton
                variante="primario"
                tamano="compacto"
                :ocupado="store.mandando"
                @click="mandarPrueba(r)"
              >
                {{ store.mandando
                  ? t('remitentes.envioDePrueba.mandando')
                  : t('remitentes.envioDePrueba.mandar') }}
              </ABoton>

              <!-- §65: «aceptó», nunca «entregado». ARLES no puede saber si
                   llegó a una bandeja, y aquí es donde más tienta decirlo. -->
              <AAviso
                v-if="store.resultadoDelEnvio?.aceptado"
                tono="exito"
                urgente
                :titulo="t('remitentes.envioDePrueba.aceptado')"
              >
                {{ t('remitentes.envioDePrueba.aceptadoDetalle', {
                  aDonde: store.resultadoDelEnvio.aDonde,
                }) }}
              </AAviso>

              <AAviso
                v-if="problemaDelEnvio"
                tono="peligro"
                urgente
                :titulo="problemaDelEnvio.que"
              >
                {{ problemaDelEnvio.como }} {{ problemaDelEnvio.salvo }}
              </AAviso>
            </div>
          </li>
        </ul>

        <div
          v-if="!borrador"
          class="remitentes__anadir"
        >
          <ABoton
            variante="primario"
            icono="mas"
            @click="abrirNuevo"
          >
            {{ t('remitentes.anadir') }}
          </ABoton>
        </div>
      </template>
    </AModulo>

    <!-- ── El formulario ──────────────────────────────────────────────── -->
    <AModulo
      v-if="borrador"
      :titulo="editando ? t('remitentes.formularioEditar') : t('remitentes.formularioNuevo')"
    >
      <form
        class="remitentes__formulario"
        @submit.prevent="guardar"
      >
        <!-- ── Los dos campos que de verdad se escriben ──
             R-01: «que sea correo y contraseña nomás». Todo lo demás se deduce
             del dominio o se esconde en la configuración avanzada. -->
        <AEntrada
          v-model="borrador.remitente"
          tipo="email"
          :etiqueta="t('remitentes.remitente')"
          :ayuda="t('remitentes.remitenteAyuda')"
          :error="errorDe('remitente')"
          @update:model-value="alEscribirLaDireccion"
        />

        <!-- Qué proveedor se reconoció. Se dice en voz alta para que nadie tenga
             que abrir la configuración avanzada sólo para comprobar que los
             valores automáticos son los correctos. -->
        <p
          v-if="proveedor"
          class="remitentes__proveedor"
        >
          {{ t('remitentes.proveedorReconocido', { nombre: proveedor.nombre }) }}
        </p>

        <AEntrada
          v-model="contrasena"
          tipo="password"
          :etiqueta="t('remitentes.contrasena')"
          :ayuda="editando ? t('remitentes.contrasenaGuardada') : ''"
        />

        <!-- El aviso del largo. No bloquea: avisa. -->
        <p
          v-if="avisoDeContrasena"
          class="remitentes__aviso-largo"
        >
          {{ avisoDeContrasena }}
        </p>

        <AEntrada
          v-model="borrador.nombreVisible"
          :etiqueta="t('remitentes.nombreVisible')"
          :ayuda="t('remitentes.nombreVisibleAyuda')"
          :error="errorDe('nombreVisible')"
        />

        <p class="remitentes__secreto">
          {{ t('remitentes.dondeVaLaContrasena') }}
        </p>

        <!-- ── La guía de Gmail ──
             Plegada: quien ya tiene su contraseña de aplicación no necesita
             leerla, y quien no, se atasca justo aquí. Es un `<details>` nativo
             y no un desplegable propio porque el navegador ya lo sabe abrir
             con el teclado y anunciarlo a un lector de pantalla. -->
        <!-- Sólo se enseña cuando sabemos dónde se crea la contraseña de ese
             proveedor. Enseñar la guía de Gmail a quien escribe un correo de
             Zoho sería mandarlo a una página que no es la suya. -->
        <details
          v-if="donde"
          class="remitentes__ayuda"
          :open="ayudaAbierta"
        >
          <summary>{{ t('remitentes.ayudaGmailTitulo') }}</summary>
          <p>{{ t('remitentes.ayudaGmailCuerpo') }}</p>
          <ol>
            <li>{{ t('remitentes.ayudaGmailPaso1') }}</li>
            <li>
              {{ t('remitentes.ayudaGmailPaso2') }}
              <!-- La dirección exacta, que es lo que faltaba. Va en un bloque
                   seleccionable aunque el botón falle: lo que no puede pasar
                   es que la única forma de tenerla sea un botón. -->
              <span class="remitentes__donde">
                <code>{{ donde }}</code>
                <!-- `secundario` y no `sutil`: con sutil no parecía un
                     botón, sino una etiqueta puesta al lado de la dirección.
                     Es la acción de este paso, y lo tiene que parecer. Lo
                     encontró mirar la pantalla, no una medida: mide 170×40 y
                     cumple el mínimo del §22 igual con las dos variantes. -->
                <ABoton
                  variante="secundario"
                  tamano="compacto"
                  @click="copiarLaDireccion(donde)"
                >
                  {{ copiada ? t('remitentes.direccionCopiada') : t('remitentes.copiarDireccion') }}
                </ABoton>
              </span>
              <span
                v-if="fallóAlCopiar"
                class="remitentes__aviso-largo"
              >{{ t('remitentes.noSePudoCopiar') }}</span>
            </li>
            <li>{{ t('remitentes.ayudaGmailPaso3') }}</li>
          </ol>
          <p class="remitentes__aviso-largo">
            {{ t('remitentes.ayudaGmailSoloUnaVez') }}
          </p>

          <!-- Los cuatro casos. El segundo y el cuarto no los arregla quien
               está delante de la pantalla, y decirlo ahorra la hora que se
               pasaría buscando una opción que su empresa le ha quitado. -->
          <p class="remitentes__no-aparece-titulo">
            {{ t('remitentes.noApareceTitulo') }}
          </p>
          <ul>
            <li>{{ t('remitentes.noAparece1') }}</li>
            <li>{{ t('remitentes.noAparece2') }}</li>
            <li>{{ t('remitentes.noAparece3') }}</li>
            <li>{{ t('remitentes.noAparece4') }}</li>
          </ul>

          <p>{{ t('remitentes.ayudaGmailNota') }}</p>
        </details>

        <!-- ── Configuración avanzada ──
             Cuatro campos que la inmensa mayoría no debería tocar nunca:
             servidor, puerto, cifrado y el nombre de usuario. Ocupaban la mitad
             del formulario y eran donde se atascaba la gente (R-01).

             Se abre sola cuando el dominio **no** se reconoce, porque entonces
             sí hay que escribirlos: ARLES no se inventa un servidor que no
             sabe. -->
        <details
          class="remitentes__avanzada"
          :open="avanzadaAbierta"
        >
          <summary>{{ t('remitentes.avanzadaTitulo') }}</summary>
          <p class="remitentes__avanzada-nota">
            {{ t('remitentes.avanzadaNota') }}
          </p>

          <AEntrada
            v-model="borrador.usuario"
            :etiqueta="t('remitentes.usuario')"
            :ayuda="t('remitentes.usuarioAyuda')"
            :error="errorDe('usuario')"
            @update:model-value="usuarioTocado = true"
          />
          <AEntrada
            v-model="borrador.servidor"
            :etiqueta="t('remitentes.servidor')"
            :ayuda="t('remitentes.servidorAyuda')"
            :error="errorDe('servidor')"
          />
          <AEntrada
            v-model="borrador.puerto"
            :etiqueta="t('remitentes.puerto')"
            :error="errorDe('puerto')"
          />
          <ASelector
            v-model="borrador.cifrado"
            :etiqueta="t('remitentes.cifrado')"
            :opciones="cifrados"
            :error="errorDe('cifrado')"
          />
        </details>

        <div class="remitentes__botones">
          <ABoton
            variante="secundario"
            :ocupado="store.probando"
            @click="probar"
          >
            {{ store.probando ? t('remitentes.probando') : t('remitentes.probar') }}
          </ABoton>
          <ABoton
            tipo="submit"
            variante="primario"
            :ocupado="store.guardando"
          >
            {{ store.guardando ? t('remitentes.guardando') : t('remitentes.guardar') }}
          </ABoton>
          <ABoton
            variante="sutil"
            @click="cerrar"
          >
            {{ t('remitentes.cancelar') }}
          </ABoton>
        </div>

        <p
          v-if="store.probando || store.guardando"
          class="remitentes__espera"
        >
          {{ t('remitentes.seEstaProbando') }}
        </p>
      </form>
    </AModulo>
  </div>
</template>

<style scoped>
.remitentes {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-6);
}

.remitentes__explicacion {
  margin: 0 0 var(--arles-space-4);
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  line-height: var(--arles-line-height-body);
}

.remitentes__lista {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-2);
}

.remitentes__fila {
  display: flex;
  align-items: center;
  /* ── LA FICHA SE PARTÍA UNA PALABRA POR LÍNEA (H-07, 16/10/2026) ───────
     Con el panel de «Mandar un correo de prueba» abierto al lado, la fila se
     estrechaba y la línea de configuración se leía así:

         Cifrado / (STARTTLS, / puerto / 587 / Hasta / 50 / al / día

     Cabía —no desbordaba nada— pero era ilegible. Cuando no hay sitio, lo
     correcto es que las acciones BAJEN a la línea siguiente, no que el texto
     se estruje hasta una palabra por renglón. */
  flex-wrap: wrap;
  gap: var(--arles-space-4);
  padding: var(--arles-space-3) var(--arles-space-4);
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  background: var(--arles-surface);
}

.remitentes__quien {
  /* La base de 18rem es lo que dispara el envoltorio de arriba: por debajo de
     ese ancho los botones se van abajo en vez de robarle sitio al texto.
     `min-width: 0` sigue haciendo falta para que un correo largo se pueda
     recortar dentro de un flex, pero ya no deja que el bloque se encoja hasta
     lo ilegible, porque `flex-basis` manda antes. */
  flex: 1 1 18rem;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  min-width: 0;
}

.remitentes__direccion {
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.remitentes__nombre,
.remitentes__servidor,
.remitentes__limites {
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

/* ── LA FICHA DEL CUPO (D-14) ─────────────────────────────────────────────
   Misma base de 18rem que `__quien`, para que las dos columnas bajen a la
   línea siguiente en el mismo punto en vez de quedar una estrujada: es la
   lección del H-07, que a 1024 px dejaba una palabra por renglón. */
.remitentes__cupo {
  flex: 1 1 18rem;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  min-width: 0;
  padding-left: var(--arles-space-4);
  border-left: var(--arles-border-width) solid var(--arles-border);
}

.remitentes__cupo-rotulo {
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
  /* Sólo mayúsculas, sin `letter-spacing`: la primera versión de esto usaba un
     `var(--arles-letter-spacing-wide)` **que no existe** —séptima vez de este
     defecto en el proyecto, y lo cazó el validador—. No se arregla
     inventándole un token al sistema de diseño por un rótulo, ni copiando el
     `0.08em` literal que tiene otra pantalla, que es una infracción del §17
     que ya está ahí. Un rótulo corto en mayúsculas se lee igual de bien. */
  text-transform: uppercase;
}

/* La cifra es lo que se viene a leer, así que pesa más que lo que la rodea. */
.remitentes__cupo-cifra {
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.remitentes__cupo-detalle,
.remitentes__cupo-campanas,
.remitentes__cupo-hoy {
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}

/* El qué-hacer del semáforo se lee entero: es lo que lo separa de un susto
   sin salida (§95). Con el ancho de lectura acotado no se estira a toda la
   fila en una pantalla grande. */
.remitentes__cupo-detalle {
  max-width: var(--arles-ancho-lectura);
}

.remitentes__envio {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-3);
  padding-top: var(--arles-space-3);
  border-top: var(--arles-border-width) solid var(--arles-border);
}

/* ── Los registros del dominio (7.5) ──
   Mismo armazón que el bloque del envío de prueba: ocupa la fila entera y se
   separa con una línea. Son dos cosas plegadas dentro de la misma ficha y
   tienen que leerse igual. */
.remitentes__dominio {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-3);
  margin-top: var(--arles-space-3);
  padding-top: var(--arles-space-3);
  border-top: var(--arles-border-width) solid var(--arles-border);
  align-items: flex-start;
}

.remitentes__registros {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
  width: 100%;
}

.remitentes__registro {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  align-items: flex-start;
}

.remitentes__registro-nombre {
  font-weight: var(--arles-font-weight-semibold);
}

.remitentes__registro-detalle {
  color: var(--arles-text-muted);
  font-size: var(--arles-font-size-small);
  max-width: 68ch;
}

/* Lo que hay que escribir en el DNS. Se separa del diagnóstico porque es lo
   único de este bloque que alguien va a copiar. */
.remitentes__que-poner {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-1);
  margin-top: var(--arles-space-2);
  padding: var(--arles-space-3);
  background: var(--arles-surface-hover);
  border-radius: var(--arles-radius-md);
  max-width: 68ch;
}

.remitentes__que-poner-rotulo {
  font-size: var(--arles-font-size-caption);
  font-weight: var(--arles-font-weight-bold);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--arles-text-muted);
}

.remitentes__que-poner-dato {
  font-size: var(--arles-font-size-small);
}

/* El valor va en monoespaciada y se puede partir: un DMARC no cabe en una
   línea y cortarlo con puntos suspensivos lo haría incopiable. */
.remitentes__que-poner-valor {
  font-family: var(--arles-font-family-mono);
  font-size: var(--arles-font-size-small);
  overflow-wrap: anywhere;
  user-select: all;
}

.remitentes__acciones {
  display: flex;
  gap: var(--arles-space-2);
}

.remitentes__anadir {
  margin-top: var(--arles-space-4);
}

.remitentes__formulario {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.remitentes__secreto {
  margin: 0;
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
  line-height: var(--arles-line-height-body);
}

.remitentes__ayuda,
.remitentes__avanzada {
  border: var(--arles-border-width) solid var(--arles-border);
  border-radius: var(--arles-radius-md);
  padding: var(--arles-space-3) var(--arles-space-4);
}

/* Los cuatro campos de dentro, separados entre sí y del resumen. El `details`
   no maqueta su contenido: sin esto saldrían pegados, que es el defecto H-02
   con otra cara. */
.remitentes__avanzada[open] {
  display: flex;
  flex-direction: column;
  gap: var(--arles-space-4);
}

.remitentes__avanzada summary {
  cursor: pointer;
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.remitentes__avanzada-nota {
  margin: 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
}

/* El proveedor reconocido: confirma que no hay nada más que tocar. */
.remitentes__proveedor {
  margin: 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-success);
}

/* El aviso del largo de la contraseña. Avisa, no bloquea, así que va en el
   color de advertencia y no en el de error. */
.remitentes__donde {
  display: flex;
  align-items: center;
  gap: var(--arles-space-2);
  flex-wrap: wrap;
  margin-top: var(--arles-space-2);
}

.remitentes__donde code {
  /* `--arles-surface-raised`, no `--arles-surface-alt`: el segundo NO EXISTE y
     me lo inventé al escribir esto. El navegador resuelve un `var()`
     inexistente a nada, sin error, así que el bloque se habría visto sin
     fondo y nadie lo habría notado. Cuarta vez en el proyecto — las otras tres
     están contadas en la Fase 8. Se comprueba que el token existe ANTES de
     usarlo, no después. */
  background: var(--arles-surface-raised);
  border: 1px solid var(--arles-border);
  border-radius: var(--arles-radius-sm);
  padding: var(--arles-space-1) var(--arles-space-2);
  /* Seleccionable a mano: si el portapapeles falla, ésta es la salida. */
  user-select: all;
}

.remitentes__no-aparece-titulo {
  margin-top: var(--arles-space-4);
  font-weight: var(--arles-font-weight-semibold);
}

.remitentes__aviso-largo {
  margin: 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-warning);
}

.remitentes__ayuda summary {
  cursor: pointer;
  font-weight: var(--arles-font-weight-semibold);
  color: var(--arles-text);
}

.remitentes__ayuda p,
.remitentes__ayuda ol {
  margin: var(--arles-space-3) 0 0;
  font-size: var(--arles-font-size-small);
  color: var(--arles-text-muted);
  line-height: var(--arles-line-height-body);
}

.remitentes__ayuda li {
  margin-bottom: var(--arles-space-2);
}

.remitentes__botones {
  display: flex;
  gap: var(--arles-space-3);
  flex-wrap: wrap;
}

.remitentes__espera {
  margin: 0;
  font-size: var(--arles-font-size-caption);
  color: var(--arles-text-muted);
}
</style>
