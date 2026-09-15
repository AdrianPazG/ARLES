/**
 * Capturas de referencia de la entrega 3.1, para la checklist de revisión.
 *
 *     node app/pruebas/sondas/referencias-31.mjs
 *
 * No es una sonda: no comprueba nada y no falla. Produce las imágenes contra
 * las que alguien compara lo que ve en su pantalla.
 *
 * Salen del mismo motor que hay debajo de WebView2 —Chromium—, así que lo que
 * se vea distinto en Windows es una diferencia real y no una diferencia de
 * descripción. En macOS el motor es WKWebView y la divergencia es justo lo que
 * la revisión busca (R-07): allí la referencia es el punto de partida de la
 * comparación, no el resultado esperado.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * EL NÚCLEO VA SIMULADO, Y CONVIENE SABERLO
 *
 * Tres de estas capturas —el formulario con errores, el guardado correcto y la
 * lista de alta avanzada— sólo existen cuando hay datos al otro lado. En un
 * navegador no hay núcleo de Tauri, así que se sustituye por un doble que
 * responde **lo mismo que responde el Rust real**: las mismas claves de error
 * para los mismos campos, la misma forma de la configuración.
 *
 * Lo que se ve dibujado es la interfaz de verdad. Lo simulado son los datos.
 * Si el doble y el núcleo divergieran, estas capturas mentirían — por eso el
 * validador compara las listas cerradas de los dos lados, y las claves de
 * error que aquí se usan son las que devuelve `arles_core::empresa`.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { spawnSync } from 'node:child_process'
import { mkdir, readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-referencias-31')
const IMAGENES = join(APP, '../documentacion/06-calidad/imagenes')
const PUERTO = 4175

/** La ventana mínima del producto. Capturar más grande daría una referencia
 *  que nadie puede reproducir en su portátil. */
const VENTANA = { width: 1120, height: 720 }

/** Lo que deja Windows al 200 % de escala en una pantalla de 1920. */
const ESCALA_200 = { width: 960, height: 640 }

const EJECUTABLE = exigirChromium()

/** El doble del núcleo. Ver la nota de cabecera. */
function nucleoSimulado() {
  const ZONAS = [
    'America/Mexico_City', 'America/Cancun', 'America/Merida', 'America/Monterrey',
    'America/Matamoros', 'America/Chihuahua', 'America/Ojinaga', 'America/Mazatlan',
    'America/Bahia_Banderas', 'America/Hermosillo', 'America/Tijuana', 'UTC',
  ]
  const estado = { empresa: null, plegada: false }

  const pasos = () => [
    { paso: 'empresa', clave: 'empresa', completado: estado.empresa !== null, disponible: true, ruta: '/ajustes', entrega: '3.1' },
    { paso: 'remitente', clave: 'remitente', completado: false, disponible: false, ruta: null, entrega: '5' },
    { paso: 'contactos', clave: 'contactos', completado: false, disponible: false, ruta: null, entrega: '3.2' },
    { paso: 'plantilla', clave: 'plantilla', completado: false, disponible: false, ruta: null, entrega: '6' },
    { paso: 'ventanaDeEjecucion', clave: 'ventana', completado: false, disponible: false, ruta: null, entrega: '6' },
    { paso: 'primeraCampana', clave: 'campana', completado: false, disponible: false, ruta: null, entrega: '6' },
  ]
  const configuracion = () => ({
    empresa: estado.empresa,
    zonas: ZONAS,
    paises: ['MX'],
    onboarding: {
      pasos: pasos(),
      completados: pasos().filter((p) => p.completado).length,
      total: 6,
      siguiente: estado.empresa ? null : 'empresa',
    },
  })

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
      if (cmd === 'configuracion_de_empresa') return Promise.resolve(configuracion())
      if (cmd === 'preferencias_de_interfaz') {
        return Promise.resolve({ barraLateralPlegada: estado.plegada })
      }
      if (cmd === 'guardar_barra_plegada') {
        estado.plegada = args.plegada
        return Promise.resolve()
      }
      if (cmd === 'guardar_empresa') {
        const b = args.borrador
        const campos = []
        if (!b.nombreComercial.trim()) {
          campos.push({ campo: 'nombreComercial', clave: 'empresa.error.nombreVacio' })
        }
        const c = b.correoCorporativo.trim()
        if (!c.includes('@') || !c.split('@')[1]?.includes('.')) {
          campos.push({ campo: 'correoCorporativo', clave: 'empresa.error.correoInvalido' })
        }
        if (campos.length) {
          return Promise.reject({ clave: 'error.app.empresa_invalida', detalle: '', campos })
        }
        estado.empresa = {
          nombreComercial: b.nombreComercial.trim(),
          pais: b.pais.toUpperCase(),
          zonaHoraria: b.zonaHoraria,
          correoCorporativo: b.correoCorporativo.trim(),
          sitioWeb: b.sitioWeb.trim() || null,
        }
        return Promise.resolve(configuracion())
      }
      return Promise.reject({ clave: 'error.db.sqlite', detalle: `desconocido: ${cmd}` })
    },
  }
}

/** Dibuja un anillo de acento alrededor de un elemento y oscurece el resto. */
function senalar(selector) {
  const el = document.querySelector(selector)
  if (!el) return
  const r = el.getBoundingClientRect()
  const marca = document.createElement('div')
  Object.assign(marca.style, {
    position: 'fixed',
    left: `${r.left - 8}px`,
    top: `${r.top - 8}px`,
    width: `${r.width + 16}px`,
    height: `${r.height + 16}px`,
    border: '3px solid #FCCC0C',
    borderRadius: '10px',
    boxShadow: '0 0 0 9999px rgba(0,0,0,.5)',
    pointerEvents: 'none',
    zIndex: '9999',
  })
  marca.id = 'marca-de-referencia'
  document.body.appendChild(marca)
}

let servidor
let navegador

try {
  await mkdir(IMAGENES, { recursive: true })

  const build = spawnSync(
    'npx',
    ['vite', 'build', '--outDir', 'dist-referencias-31', '--emptyOutDir'],
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
  const pagina = await navegador.newPage({ viewport: VENTANA, deviceScaleFactor: 2 })
  await pagina.addInitScript(nucleoSimulado)

  const guardar = async (nombre) => {
    await pagina.screenshot({ path: join(IMAGENES, `${nombre}.png`) })
    console.log(`  ✓ ${nombre}.png`)
  }
  const ir = async (ruta) => {
    await pagina.goto(`http://localhost:${PUERTO}/#${ruta}`, { waitUntil: 'networkidle' })
    await pagina.waitForTimeout(450)
  }

  // ── 1 · Inicio, recién abierto ──
  await ir('/inicio')
  await guardar('chk-01-inicio')

  // ── 2 · El botón de plegar, señalado ──
  await pagina.evaluate(senalar, '.plegador')
  await pagina.waitForTimeout(150)
  await guardar('chk-02-plegador')
  await pagina.evaluate(() => document.getElementById('marca-de-referencia')?.remove())

  // ── 3 · La barra plegada ──
  await pagina.click('.plegador')
  await pagina.waitForTimeout(400)
  await guardar('chk-03-plegada')
  await pagina.click('.plegador')
  await pagina.waitForTimeout(400)

  // ── 4 · Ajustes, vacío ──
  await ir('/ajustes')
  await guardar('chk-04-ajustes')

  // ── 5 · Ajustes tras enviar vacío: campos marcados y aviso ──
  await pagina.click('button[type=submit]')
  await pagina.waitForTimeout(400)
  await guardar('chk-05-ajustes-errores')

  // ── 6 · Ajustes tras guardar bien ──
  await pagina.fill('input[type=email]', 'hola@telemetry.mx')
  const nombre = pagina.locator('form input').first()
  await nombre.fill('TELEMETRY INSIGHT')
  await pagina.click('button[type=submit]')
  await pagina.waitForTimeout(500)
  await guardar('chk-06-ajustes-guardado')

  // ── 7 · Inicio con el primer paso hecho ──
  await ir('/inicio')
  await guardar('chk-07-inicio-avanzado')

  // ── 8 · Al 200 % de escala: se pliega sola y el botón queda apagado ──
  await pagina.setViewportSize(ESCALA_200)
  await pagina.waitForTimeout(400)
  await guardar('chk-08-escala-200')
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

console.log('\nReferencias de la 3.1 en documentacion/06-calidad/imagenes/')
