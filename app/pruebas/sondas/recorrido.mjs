/**
 * El recorrido del usuario: qué se ve y qué se puede hacer en cada pantalla.
 *
 *     node app/pruebas/sondas/recorrido.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ MIDE ESTA SONDA QUE NINGUNA OTRA MIRABA
 *
 * `auditoria.mjs` comprueba que las pantallas son **accesibles y legibles**:
 * contraste, foco, encabezados, desbordes. Todas pasan, y ninguna de esas
 * medidas dice si la pantalla **sirve para algo**.
 *
 * Esta mira lo otro: qué encuentra una persona al abrir cada sitio.
 *
 *  1. **Ninguna pantalla se queda muda.** Sin datos hay que decir qué falta y
 *     cómo conseguirlo, no dejar el hueco en blanco. Un vacío sin explicación
 *     es la forma más cara de perder a alguien: no sabe si está roto, si le
 *     falta configurar algo, o si es que todavía no hizo nada.
 *
 *  2. **Toda pantalla tiene una salida hacia delante.** Si lo único que se
 *     puede hacer es volver atrás, es un callejón. Se cuenta cuántas acciones
 *     reales ofrece.
 *
 *  3. **Ningún texto sale como clave de i18n.** `plantilla.variable.nombre` en
 *     pantalla es un texto que nadie escribió, y ha pasado dos veces en este
 *     proyecto: el `rotulo()` del asistente y los modos del canal. No falla en
 *     ningún sitio; simplemente se lee mal.
 *
 *  4. **Ninguna cifra aparece como adjetivo** (§94). Se buscan las palabras
 *     que el proyecto prohibió porque no se pueden verificar.
 *
 *  5. **Nada dice «entregado»** (§65). ARLES no puede saberlo, y decirlo es la
 *     afirmación que el producto entero está construido para no hacer.
 *
 * SE HA VISTO FALLAR · 29/09/2026
 *
 * Las cinco se rompieron a propósito: vaciando el estado vacío de contactos;
 * escondiendo los botones de Remitentes; metiendo una clave cruda en un
 * título; escribiendo «casi listo» en una nota; y poniendo «entregado» en la
 * lista de campañas. Las cinco se cazaron con el mensaje correcto.
 *
 * LO QUE NO PRUEBA
 *
 * Que los datos sean correctos: en el navegador no hay núcleo, así que las
 * pantallas salen con lo que el modo demostración les da. Lo que se comprueba
 * es **la forma del recorrido**, no la aritmética — ésa está en Rust.
 */
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { python } from './ordenes.mjs'

const RAIZ = fileURLToPath(new URL('../../..', import.meta.url))
const ARCHIVO = path.join(RAIZ, 'vista-previa/ARLES-vista-previa.html')

/**
 * Cada sitio al que el usuario puede llegar, y qué se espera encontrar.
 *
 * `minimoAcciones` es cuántas cosas se pueden hacer sin contar «volver». Una
 * pantalla de solo lectura puede tener 0 y estar bien —la lista de bajas, por
 * ejemplo—, pero entonces tiene que decirlo por escrito.
 */
const RECORRIDO = [
  { ruta: '#/inicio', que: 'Inicio', minimoAcciones: 1 },
  { ruta: '#/inicio/configuracion', que: 'Configuración de la empresa', minimoAcciones: 1 },
  { ruta: '#/contactos', que: 'Contactos', minimoAcciones: 1 },
  { ruta: '#/contactos/importar', que: 'Importar', minimoAcciones: 1 },
  { ruta: '#/contactos/supresion', que: 'Bajas', minimoAcciones: 0 },
  { ruta: '#/contactos/derechos', que: 'Derechos ARCO', minimoAcciones: 1 },
  { ruta: '#/campanas', que: 'Campañas', minimoAcciones: 1 },
  { ruta: '#/campanas/plantillas', que: 'Plantillas y firmas', minimoAcciones: 2 },
  { ruta: '#/campanas/nueva', que: 'Asistente de campaña', minimoAcciones: 1 },
  { ruta: '#/remitentes', que: 'Remitentes', minimoAcciones: 1 },
  { ruta: '#/remitentes/whatsapp', que: 'Número de WhatsApp', minimoAcciones: 1 },
  { ruta: '#/actividad', que: 'Actividad', minimoAcciones: 0 },
  { ruta: '#/ajustes', que: 'Ajustes', minimoAcciones: 1 },
]

/** §94: cifras, no adjetivos. Estas palabras no se pueden verificar. */
const ADJETIVOS = ['casi listo', 'casi todo', 'muchos', 'pocos', 'bastantes', 'la mayoría']

/** §65: ARLES no puede saber si un correo llegó. */
const PROHIBIDAS = ['entregado', 'entregados', 'entregada', 'entregadas']

const hallazgos = []
const mal = (t) => hallazgos.push(t)

/** Lo que se mide dentro de la página. */
const MEDIR = () => {
  const main = document.querySelector('main') ?? document.body
  const texto = (main.innerText ?? '').trim()

  // Las acciones reales de la pantalla.
  //
  // Se busca en TODO el documento menos la navegación, y no sólo dentro de
  // `<main>`: el pie de un asistente vive fuera, y la primera versión de esta
  // sonda daba cero acciones en el asistente de nueve pasos por eso.
  //
  // Y se cuentan los controles de formulario además de los botones: Ajustes
  // ofrece un desplegable de tema y ninguna otra cosa, así que contando sólo
  // `button` salía como un callejón sin serlo.
  const acciones = [
    ...document.querySelectorAll('button, a[href], select, input, textarea, summary'),
  ]
    .filter((el) => {
      if (el.closest('nav')) return false
      // Un control sin texto cuenta igual: lo que importa es que se pueda
      // hacer algo, y un `select` lleva su etiqueta fuera.
      if (el.matches('select, input, textarea')) return true
      const t = (el.innerText ?? '').trim().toLowerCase()
      return t.length > 0 && t !== 'volver' && t !== 'atrás' && t !== 'cancelar'
    })
    .map((el) => (el.innerText ?? '').trim() || el.tagName.toLowerCase())

  // Una clave de i18n sin traducir: palabras separadas por puntos, sin
  // espacios, que es la forma que tiene `seccion.subseccion.cosa`.
  //
  // Un dominio tiene exactamente la misma forma, y `smtp.gmail.com` salía
  // marcado. Se descartan los que acaban en un dominio de primer nivel
  // conocido, que es lo que distingue una dirección de una clave.
  const DOMINIOS = /\.(com|mx|org|net|io|dev|es|co)$/
  const claves = [...(texto.match(/\b[a-z][a-zA-Z0-9]*(?:\.[a-z][a-zA-Z0-9_]*){2,}\b/g) ?? [])]
    .filter((c) => !DOMINIOS.test(c))

  return { texto, acciones, claves, largo: texto.length }
}

const ejecutable = await exigirChromium()

console.log('Generando la vista previa…')

// ─────────────────────────────────────────────────────────────────────────
// SI LA VISTA PREVIA NO SE PUDO REGENERAR, ESTA SONDA NO SIRVE
//
// `generar.py` compila con Vite y aborta si la compilación falla, pero deja
// en disco el HTML de la vez anterior. Sin esta comprobación, la sonda lo
// medía tan contenta y daba un resultado **de otra versión del código**.
//
// Pasó el 29/09/2026 verificando la sonda: una pantalla rota a propósito no
// compiló, y el hallazgo que salió era el de la rotura anterior. Una sonda que
// mide un artefacto viejo es peor que no tenerla, porque se cree.
const generacion = python([path.join(RAIZ, 'herramientas/vista-previa/generar.py')])
if (generacion.status !== 0) {
  console.error('\n✗ No se pudo generar la vista previa, así que no hay nada que medir.\n')
  console.error(String(generacion.stderr ?? '').slice(-3000))
  process.exit(1)
}

const navegador = await chromium.launch({ executablePath: ejecutable })
const pagina = await navegador.newPage({ viewport: { width: 1280, height: 900 } })

const resumen = []

for (const { ruta, que, minimoAcciones } of RECORRIDO) {
  await pagina.goto(`file://${ARCHIVO}${ruta}`, { waitUntil: 'load' })

  // Una espera generosa y fija, y aquí sí es lo correcto.
  //
  // Tres intentos más listos fallaron, y conviene que quede escrito porque es
  // la trampa de cualquier sonda:
  //
  //  – 700 ms medían pantallas a medio pintar: la de plantillas salía sin sus
  //    dos listas y parecía un fallo de la pantalla cuando era de la sonda.
  //  – Esperar a que desapareciera el indicador de carga se cumplía al
  //    instante, porque en ese momento todavía no había aparecido.
  //  – Comparar el contenido con la medida anterior se envenenaba solo:
  //    **navegar cambiando el hash no recarga el documento**, así que la
  //    variable con la medida previa sobrevivía de una pantalla a la otra.
  //
  // Dos segundos cuesta medio minuto en las trece pantallas, y a cambio la
  // medida es la que se ve. Una sonda lenta y cierta vale más que una rápida
  // que mide a medias.
  await pagina.waitForTimeout(2000)

  const r = await pagina.evaluate(MEDIR)

  // 1 · Nadie se queda mudo.
  if (r.largo < 40) {
    mal(`${ruta} (${que}): la pantalla está prácticamente vacía (${r.largo} caracteres). ` +
        'Sin datos hay que decir qué falta y cómo conseguirlo.')
  }

  // 2 · Hay algo que hacer.
  if (r.acciones.length < minimoAcciones) {
    mal(`${ruta} (${que}): ${r.acciones.length} acciones, se esperaban ${minimoAcciones}. ` +
        'Una pantalla de la que solo se puede salir es un callejón.')
  }

  // 3 · Ningún texto es una clave.
  for (const c of r.claves) {
    mal(`${ruta} (${que}): sale una clave de i18n sin traducir — «${c}»`)
  }

  // 4 · Cifras, no adjetivos.
  const bajo = r.texto.toLowerCase()
  for (const a of ADJETIVOS) {
    if (bajo.includes(a)) mal(`${ruta} (${que}): «${a}» es un adjetivo, y el §94 pide una cifra`)
  }

  // 6 · Ninguna pantalla recibe a nadie con un error.
  //
  // Abrir algo por primera vez y encontrar «Ocurrió un error, cierra ARLES»
  // es lo que hacía el asistente sin núcleo. Un error de arranque que no es
  // culpa de nadie enseñado como si algo se hubiera roto cuesta la confianza
  // de la primera pantalla, que es la que más vale.
  if (/ocurrió un error|vuelve a abrirlo|error inesperado/i.test(bajo)) {
    mal(`${ruta} (${que}): recibe al usuario con un mensaje de error. ` +
        'Una pantalla recién abierta no debería empezar disculpándose.')
  }

  // 7 · Ningún dato se enseña como «no reconocido».
  //
  // Es lo que pinta una pantalla cuando recibe un valor que no sabe traducir,
  // y hace bien en decirlo en vez de inventarse un rótulo. Pero verlo en la
  // demostración significa que algo manda un valor que no cuadra con el
  // contrato: pasó con el estado de las campañas, que viaja como **clave de
  // rótulo** y no como el estado crudo de la base (§139).
  if (/no reconocid[oa]|desconocid[oa]/i.test(bajo)) {
    mal(`${ruta} (${que}): enseña un valor como «no reconocido». ` +
        'Algo está mandando un dato que no cuadra con lo que la pantalla espera.')
  }

  // 5 · Aceptado ≠ entregado.
  for (const p of PROHIBIDAS) {
    if (new RegExp(`\\b${p}\\b`).test(bajo)) {
      mal(`${ruta} (${que}): dice «${p}», y ARLES no puede saberlo (§65)`)
    }
  }

  resumen.push(`  ${String(r.acciones.length).padStart(2)} acciones · ${String(r.largo).padStart(5)} car. · ${que}`)
}

await navegador.close()

console.log(`\n✓ ${RECORRIDO.length} pantallas recorridas.\n`)
console.log(resumen.join('\n'))

if (hallazgos.length > 0) {
  console.error(`\n✗ ${hallazgos.length} hallazgo(s):\n`)
  hallazgos.forEach((h) => console.error(`  – ${h}`))
  process.exit(1)
}
console.log('\nSin hallazgos.')
