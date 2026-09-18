/**
 * Capturas de referencia para el manual de revisión visual.
 *
 * No es una sonda: no comprueba nada, no falla. Produce las imágenes contra
 * las que alguien compara lo que ve en su pantalla real.
 *
 *     node app/pruebas/sondas/referencias.mjs
 *
 * Por qué existen. La revisión visual le pide a una persona que mire cuatro
 * cosas y diga si están bien. «Bien» comparado con qué, es la pregunta que
 * nadie hacía: la guía las describía con palabras. Estas capturas salen del
 * mismo motor que usa Windows —Chromium, igual que WebView2—, así que lo que
 * se vea distinto en la pantalla de verdad es una diferencia real, no una
 * diferencia de descripción.
 *
 * Lo que NO prueban: que en macOS se vea igual. Ahí el motor es WKWebView y la
 * divergencia es justo lo que buscamos (R-07). La referencia es el punto de
 * partida de la comparación, no el resultado esperado.
 */
import { mkdir, readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { vite } from './ordenes.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-referencias')
const IMAGENES = join(APP, '../documentacion/06-calidad/imagenes')
const PUERTO = 4174

// La ventana mínima del producto (tauri.conf.json). Capturar más grande daría
// una referencia que nadie puede reproducir en un portátil.
const VENTANA = { width: 1120, height: 720 }

const EJECUTABLE = exigirChromium()

let servidor
let navegador

try {
  await mkdir(IMAGENES, { recursive: true })

  const build = vite(['build', '--outDir', 'dist-referencias', '--emptyOutDir'],
    { cwd: APP, encoding: 'utf8', env: { ...process.env, VITE_ARLES_CATALOGO: '1' } },
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
    viewport: VENTANA,
    deviceScaleFactor: 2, // que la captura no se vea borrosa al ampliarla
  })

  const guardar = async (nombre) => {
    await pagina.screenshot({ path: join(IMAGENES, `${nombre}.png`) })
    console.log(`  ✓ ${nombre}.png`)
  }

  // ── El enlace del catálogo en la navegación ──
  await pagina.goto(`http://localhost:${PUERTO}/#/inicio`, { waitUntil: 'networkidle' })
  await pagina.waitForTimeout(500)
  await guardar('ref-00-inicio')

  const enlace = pagina.locator('.enlace-de-revision')
  await enlace.scrollIntoViewIfNeeded()
  const caja = await enlace.boundingBox()
  if (caja) {
    // Un recorte generoso alrededor: el enlace solo no se sitúa en la pantalla.
    await pagina.screenshot({
      path: join(IMAGENES, 'ref-01-enlace-catalogo.png'),
      clip: {
        x: Math.max(0, caja.x - 24),
        y: Math.max(0, caja.y - 180),
        width: Math.min(VENTANA.width, caja.width + 120),
        height: caja.height + 220,
      },
    })
    console.log('  ✓ ref-01-enlace-catalogo.png')
  }

  // Y la misma, con el enlace señalado. Es lo único que la gente no encuentra:
  // está abajo del todo y a propósito no parece una sección del producto.
  await pagina.evaluate(() => {
    const el = document.querySelector('.enlace-de-revision')
    if (!el) return
    const r = el.getBoundingClientRect()
    const marca = document.createElement('div')
    Object.assign(marca.style, {
      position: 'fixed',
      left: `${r.left - 10}px`,
      top: `${r.top - 10}px`,
      width: `${r.width + 20}px`,
      height: `${r.height + 20}px`,
      border: '4px solid #FCCC0C',
      borderRadius: '12px',
      boxShadow: '0 0 0 9999px rgba(0,0,0,.55)',
      pointerEvents: 'none',
      zIndex: '9999',
    })
    document.body.appendChild(marca)
  })
  await pagina.waitForTimeout(200)
  await guardar('ref-01b-enlace-senalado')

  // ── Las cuatro pestañas del catálogo ──
  await pagina.goto(`http://localhost:${PUERTO}/#/catalogo`, { waitUntil: 'networkidle' })
  await pagina.waitForTimeout(700)

  const pestanas = [
    ['Primitivas', 'ref-02-primitivas'],
    ['Los cuatro estados', 'ref-03-estados'],
    ['Tabla virtualizada', 'ref-04-tabla'],
    ['Tipografía', 'ref-05-tipografia'],
  ]
  for (const [etiqueta, nombre] of pestanas) {
    await pagina.getByRole('tab', { name: etiqueta, exact: true }).click()
    await pagina.waitForTimeout(600)
    await pagina.evaluate(() => window.scrollTo(0, 0))
    await guardar(nombre)
  }

  // ── El modal, que es lo que más se nota entre plataformas ──
  await pagina.getByRole('tab', { name: 'Primitivas', exact: true }).click()
  await pagina.waitForTimeout(400)
  await pagina.getByRole('button', { name: 'Modal destructivo' }).click()
  await pagina.waitForTimeout(500)
  await guardar('ref-06-modal')
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

console.log(`\nCapturas en documentacion/06-calidad/imagenes/`)
