/**
 * Las pantallas de ARLES en los dos temas, una al lado de la otra.
 *
 *     node app/pruebas/sondas/temas.mjs
 *
 * No es una sonda: no comprueba nada y no falla. Produce las imágenes con las
 * que Dirección aprueba —o rechaza— el tema claro.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ NO BASTA LA LÁMINA DE COLOR
 *
 * La lámina dice que cada pareja de colores cumple su ratio. Eso es necesario
 * y no es suficiente: una paleta puede pasar todos los contratos de contraste
 * y aun así verse plana, sucia o ajena a la marca. Los contratos miden
 * legibilidad; nadie ha inventado todavía la forma de medir si una pantalla
 * se ve bien.
 *
 * Por eso esto renderiza **la aplicación de verdad**, con sus componentes
 * reales, en los dos temas. Lo que se aprueba es esto, no una cuadrícula de
 * muestras.
 * ─────────────────────────────────────────────────────────────────────────
 *
 * El tema se aplica escribiendo `data-tema` en <html>, que es exactamente lo
 * que hará la aplicación cuando el usuario lo elija en Ajustes. Si esta sonda
 * dejara de pintar el claro, sería porque el mecanismo real está roto.
 */
import { spawnSync } from 'node:child_process'
import { mkdir, readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-temas')
const IMAGENES = join(APP, '../documentacion/05-diseno/imagenes')
const PUERTO = 4177

const VENTANA = { width: 1120, height: 760 }
const EJECUTABLE = exigirChromium()

/** El mismo doble del núcleo que usa la sonda de referencias de la 3.1: lo
 *  dibujado es la interfaz de verdad, lo simulado son los datos. */
function nucleoSimulado() {
  const ZONAS = [
    'America/Mexico_City', 'America/Cancun', 'America/Merida', 'America/Monterrey',
    'America/Matamoros', 'America/Chihuahua', 'America/Ojinaga', 'America/Mazatlan',
    'America/Bahia_Banderas', 'America/Hermosillo', 'America/Tijuana', 'UTC',
  ]
  const estado = {
    empresa: {
      nombreComercial: 'TELEMETRY INSIGHT',
      pais: 'MX',
      zonaHoraria: 'America/Mexico_City',
      correoCorporativo: 'hola@telemetrymx.com',
      sitioWeb: 'https://telemetrymx.com',
    },
    plegada: false,
  }
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
      if (cmd === 'guardar_empresa') return Promise.resolve(configuracion())
      return Promise.reject({ clave: 'error.db.sqlite', detalle: `desconocido: ${cmd}` })
    },
  }
}

let servidor
let navegador

try {
  await mkdir(IMAGENES, { recursive: true })

  const build = spawnSync(
    'npx',
    ['vite', 'build', '--outDir', 'dist-temas', '--emptyOutDir'],
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

  const PANTALLAS = [
    ['inicio', '/inicio'],
    ['ajustes', '/ajustes'],
    ['catalogo', '/catalogo'],
  ]
  const TEMAS = ['oscuro', 'claro']

  for (const [nombre, ruta] of PANTALLAS) {
    for (const tema of TEMAS) {
      await pagina.goto(`http://localhost:${PUERTO}/#${ruta}`, { waitUntil: 'networkidle' })
      await pagina.evaluate((t) => {
        document.documentElement.setAttribute('data-tema', t)
      }, tema)
      await pagina.waitForTimeout(450)
      const archivo = `tema-${nombre}-${tema}.png`
      await pagina.screenshot({
        path: join(IMAGENES, archivo),
        fullPage: nombre === 'catalogo',
      })
      console.log(`  ✓ ${archivo}`)
    }
  }
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

console.log('\nLos dos temas en documentacion/05-diseno/imagenes/')
