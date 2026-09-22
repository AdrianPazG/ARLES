/**
 * La navegación no se mueve al plegar la barra.
 *
 *     node app/pruebas/sondas/cabecera.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE MIDE Y POR QUÉ
 *
 * Dirección señaló que «los iconos saltan» al pulsar el botón de plegar.
 * Medido en la entrega 3.1: el primer enlace de sección estaba en y=131 con la
 * barra abierta y en y=80 con la barra plegada. El motivo era que el logotipo
 * desaparecía al plegar y **arrastraba hacia arriba todo lo que venía
 * debajo**.
 *
 * La corrección es reservar la altura de la cabecera en los dos estados: la
 * marca deja sitio al isotipo, y el botón vive en su propia fila. Eso es una
 * afirmación geométrica, así que se mide en vez de mirarse.
 *
 * Esta sonda **falla** si la navegación se desplaza más de un píxel. Un píxel
 * de tolerancia porque el redondeo de subpíxel de Chromium puede dar 131.0
 * frente a 130.5 sin que nadie perciba nada.
 * ─────────────────────────────────────────────────────────────────────────
 *
 * Comprueba además que **la barra de la sección activa se anima**, y que deja
 * de animarse con `prefers-reduced-motion`. Una animación se escribe en una
 * línea de CSS y se rompe con la misma facilidad —basta una regla que gane en
 * especificidad— sin que nada falle: simplemente el estado salta. Por eso se
 * mide el recorrido y no sólo el estado final.
 *
 * Lo que NO prueba: que en macOS sea igual. Ahí el motor es WKWebView (R-07).
 */
import { readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { vite } from './ordenes.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-cabecera')
const PUERTO = 4178

/** Un píxel: el redondeo de subpíxel, no un desplazamiento que se vea. */
const TOLERANCIA_PX = 1

const EJECUTABLE = exigirChromium()

function nucleoSimulado() {
  const estado = { plegada: false }
  window.__TAURI_INTERNALS__ = {
    invoke(cmd, args) {
      if (cmd === 'info_app') {
        return Promise.resolve({
          nombre: 'ARLES RELAY',
          nombreComercial: 'ARLES RELAY I',
          version: '1.2.0',
          atribucion: 'Software desarrollado por TELEMETRY INSIGHT',
        })
      }
      if (cmd === 'configuracion_de_empresa') {
        return Promise.resolve({
          empresa: null,
          zonas: ['America/Mexico_City'],
          paises: ['MX'],
          onboarding: { pasos: [], completados: 0, total: 6, siguiente: 'empresa' },
        })
      }
      if (cmd === 'preferencias_de_interfaz') {
        return Promise.resolve({ barraLateralPlegada: estado.plegada })
      }
      if (cmd === 'guardar_barra_plegada') {
        estado.plegada = args.plegada
        return Promise.resolve()
      }
      return Promise.reject({ clave: 'error.db.sqlite', detalle: `desconocido: ${cmd}` })
    },
  }
}

let servidor
let navegador
let codigo = 0

try {
  const build = vite(['build', '--outDir', 'dist-cabecera', '--emptyOutDir'],
    { cwd: APP, encoding: 'utf8' },
  )
  if (build.status !== 0) {
    console.error(build.stderr || build.stdout)
    process.exit(1)
  }

  const TIPOS = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.woff2': 'font/woff2',
    '.png': 'image/png',
  }
  servidor = http.createServer(async (req, res) => {
    const ruta = (req.url ?? '/').split('?')[0]
    const archivo = join(SALIDA, ruta === '/' ? 'index.html' : ruta)
    try {
      const cuerpo = await readFile(archivo)
      res.writeHead(200, { 'Content-Type': TIPOS[extname(archivo)] ?? 'application/octet-stream' })
      res.end(cuerpo)
    } catch {
      res.writeHead(404)
      res.end()
    }
  })
  await new Promise((r) => servidor.listen(PUERTO, r))

  navegador = await chromium.launch({ executablePath: EJECUTABLE })
  const pagina = await navegador.newPage({
    viewport: { width: 1120, height: 760 },
  })
  await pagina.addInitScript(nucleoSimulado)
  await pagina.goto(`http://localhost:${PUERTO}/#/inicio`, { waitUntil: 'networkidle' })
  await pagina.waitForTimeout(400)

  /** La y del primer enlace de sección, que es lo que Dirección vio saltar. */
  const yDelPrimerEnlace = async () => {
    const caja = await pagina.locator('.nav-lista .nav-enlace').first().boundingBox()
    if (!caja) throw new Error('no encuentro el primer enlace de la navegación')
    return caja.y
  }

  const abierta = await yDelPrimerEnlace()

  await pagina.click('.plegador')
  // Más que la transición de la barra: medir a mitad de animación daría un
  // número que no es ni el de antes ni el de después.
  await pagina.waitForTimeout(600)
  const plegada = await yDelPrimerEnlace()

  const salto = Math.abs(plegada - abierta)
  console.log(`  abierta  y = ${abierta.toFixed(1)}`)
  console.log(`  plegada  y = ${plegada.toFixed(1)}`)
  console.log(`  salto      = ${salto.toFixed(1)} px  (tope ${TOLERANCIA_PX})`)

  if (salto > TOLERANCIA_PX) {
    console.error(
      `\n✗ La navegación se desplaza ${salto.toFixed(1)} px al plegar la barra.\n` +
        '  La cabecera tiene que ocupar lo mismo en los dos estados: si la marca\n' +
        '  deja de reservar su altura, todo lo de abajo sube y los iconos saltan.',
    )
    codigo = 1
  } else {
    console.log('\n✓ La navegación no se mueve al plegar.')
  }

  // El isotipo es lo que ocupa el sitio de la marca al plegar. Si dejara de
  // pintarse, el salto volvería — pero con la altura reservada el salto no lo
  // detectaría, así que se comprueba aparte.
  // ── El acento de la sección activa se MUEVE, no salta ──
  //
  // La garantía no ha cambiado desde que se escribió: el §98 pide que el
  // movimiento diga a dónde se fue el estado, y un acento que aparece de golpe
  // en otro sitio no lo dice. Lo que cambió el 22/09/2026 es **qué pieza lo
  // lleva**: antes era la barrita `::before` de cada enlace, que crecía de 0 a
  // su altura; ahora es una sola pieza que se desliza entre secciones.
  //
  // Así que se mide la pieza que de verdad esté puesta. Si el indicador que
  // viaja está dibujado, se mide SU POSICIÓN; si la medida falló y la barra
  // volvió al dibujo de respaldo, se mide la altura del `::before`, como
  // antes. Las dos son la misma pregunta: ¿a mitad de camino ya está donde va
  // a acabar?
  //
  // Y se lee `.nav-enlace.activo`, la clase que calcula el producto, no
  // `.router-link-active`: vue-router no marca la sección madre desde una
  // subpantalla, que es el defecto que Dirección encontró el 21/09 y el motivo
  // de que exista la clase propia.
  const posicionDelAcento = () =>
    pagina.evaluate(() => {
      const viajero = document.querySelector('.nav-indicador')
      const marco = document.querySelector('.nav-marco')
      if (viajero && marco && getComputedStyle(viajero).display !== 'none') {
        return {
          modo: 'viajero',
          valor: viajero.getBoundingClientRect().top - marco.getBoundingClientRect().top,
        }
      }
      const activo = document.querySelector('.nav-enlace.activo')
      if (!activo) return null
      return {
        modo: 'respaldo',
        valor: parseFloat(getComputedStyle(activo, '::before').height),
      }
    })

  const medirRecorrido = async () => {
    // Se parte de OTRA sección. La primera versión iba a Inicio y después
    // pulsaba Inicio: sin cambio de sección no hay nada que animar, y la
    // sonda acusaba al producto de un fallo suyo.
    await pagina.goto(`http://localhost:${PUERTO}/#/ajustes`, { waitUntil: 'networkidle' })
    await pagina.waitForTimeout(500)
    const alSalir = await posicionDelAcento()
    await pagina.click('.nav-lista li:first-child .nav-enlace')
    // Un cuarto de la duración declarada (200 ms): lo bastante pronto para
    // pillarla a medias y lo bastante tarde para que haya empezado.
    await pagina.waitForTimeout(50)
    const aMedias = await posicionDelAcento()
    await pagina.waitForTimeout(500)
    return { alSalir, aMedias, alFinal: await posicionDelAcento() }
  }

  const { alSalir, aMedias, alFinal } = await medirRecorrido()
  console.log(
    `\n  acento (${alFinal?.modo}): al salir = ${alSalir?.valor.toFixed(1)},` +
      ` a los 50 ms = ${aMedias?.valor.toFixed(1)}, al final = ${alFinal?.valor.toFixed(1)}`,
  )

  /**
   * ¿Está a mitad de camino, o ya llegó?
   *
   * Se compara contra el punto de PARTIDA y no contra cero, porque el viajero
   * mide una posición y la posición final de la primera sección **es** cero.
   * Medir «mayor que cero» daría por no dibujado justo el caso más común.
   */
  const aMitadDeCamino = (a, b, fin) =>
    a !== null && b !== null && fin !== null &&
    Math.abs(b.valor - fin.valor) > 0.5 &&
    Math.abs(b.valor - a.valor) > 0.5

  if (!alFinal || !alSalir || Math.abs(alSalir.valor - alFinal.valor) < 0.5) {
    console.error(
      '\n✗ La sección activa no marca su acento en ningún sitio: el indicador\n' +
        '  no se dibuja y el respaldo tampoco. El estado se queda sólo en el\n' +
        '  color de relleno, que es justo lo que prohíbe el §19.',
    )
    codigo = 1
  } else if (!aMitadDeCamino(alSalir, aMedias, alFinal)) {
    console.error(
      '\n✗ El acento de la sección activa NO se mueve: a los 50 ms ya está\n' +
        '  donde acaba. El movimiento dice a dónde se fue el estado; sin él,\n' +
        '  el acento aparece de golpe en otro sitio.',
    )
    codigo = 1
  } else {
    console.log('✓ El acento de la sección activa se mueve al cambiar de sección.')
  }

  // ── Y deja de moverse si el sistema lo pide ──
  await pagina.emulateMedia({ reducedMotion: 'reduce' })
  const reducido = await medirRecorrido()
  console.log(`  con movimiento reducido: a los 50 ms = ${reducido.aMedias?.valor.toFixed(1)}`)
  if (aMitadDeCamino(reducido.alSalir, reducido.aMedias, reducido.alFinal)) {
    console.error(
      '\n✗ Con `prefers-reduced-motion: reduce` el acento sigue moviéndose.\n' +
        '  El §98 lo hace obligatorio, no opcional.',
    )
    codigo = 1
  } else {
    console.log('✓ Con movimiento reducido, el acento llega directo a su sitio.')
  }
  await pagina.emulateMedia({ reducedMotion: 'no-preference' })

  const isotipo = await pagina.locator('.marca .t-icono').count()
  if (isotipo !== 1) {
    console.error(
      `\n✗ Plegada, la marca no enseña el isotipo (encontrados: ${isotipo}).\n` +
        '  Es la «A» que el sistema operativo enseña en la barra de tareas, y\n' +
        '  es lo único que identifica la aplicación cuando no cabe el logotipo.',
    )
    codigo = 1
  } else {
    console.log('✓ Plegada, la marca enseña el isotipo.')
  }
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

process.exit(codigo)
