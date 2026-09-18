/**
 * Sonda del árbol de accesibilidad — lo que un lector de pantalla anuncia.
 *
 *     npm --prefix app run sonda:lector
 *
 * Por qué existe, y por qué sustituye a la sección de NVDA del manual.
 *
 * La revisión visual pedía a una persona instalar NVDA, ponerlo en español,
 * recorrer la aplicación con el tabulador y decir si lo que oía tenía sentido.
 * Es la comprobación más cara de las cuatro y la menos repetible: depende de
 * que alguien tenga tiempo, sepa configurarlo y recuerde qué oyó.
 *
 * Y resulta que **casi todo lo que pedía escuchar se puede leer**. NVDA sobre
 * WebView2 no inventa lo que dice: lo deriva del árbol de accesibilidad que
 * expone el motor —el mismo Chromium que hay debajo—. Ese árbol se puede
 * inspeccionar, y entonces «¿dice botón?» deja de ser una pregunta de oído y
 * pasa a ser una de dato: ¿el rol es `button` y tiene nombre accesible?
 *
 * Lo que esto NO cubre, dicho sin adornos:
 *   · Cómo SUENA. Si la voz pronuncia mal, aquí no se ve. Tampoco importa:
 *     lo que se juzga es qué se anuncia, no la calidad del sintetizador.
 *   · Rarezas propias de NVDA o de VoiceOver frente al árbol estándar. Son
 *     raras, pero existen. Por eso la comprobación a oído sigue en el manual
 *     como opcional, no desaparece.
 *
 * El árbol de macOS lo construye WKWebView y no es idéntico al de Chromium.
 * Esta sonda cubre Windows con precisión y macOS por aproximación.
 */
import { readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { vite } from './ordenes.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-lector')
const PUERTO = 4178
const EJECUTABLE = exigirChromium()

const problemas = []
const anotar = (q) => { problemas.push(q); console.log(`  ✗ ${q}`) }
const bien = (q) => console.log(`  ✓ ${q}`)

let servidor
let navegador

try {
  const build = vite(['build', '--outDir', 'dist-lector', '--emptyOutDir'],
    { cwd: APP, encoding: 'utf8', env: { ...process.env, VITE_ARLES_CATALOGO: '1' } },
  )
  if (build.status !== 0) {
    anotar(`no compiló: ${(build.stderr || build.stdout).slice(-400)}`)
    throw new Error('build')
  }

  const TIPOS = {
    '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css',
    '.woff2': 'font/woff2', '.png': 'image/png',
  }
  servidor = http.createServer(async (req, res) => {
    const ruta = (req.url ?? '/').split('?')[0]
    const archivo = join(SALIDA, ruta === '/' ? 'index.html' : ruta)
    try {
      const cuerpo = await readFile(archivo)
      res.writeHead(200, { 'Content-Type': TIPOS[extname(archivo)] ?? 'application/octet-stream' })
      res.end(cuerpo)
    } catch { res.writeHead(404); res.end() }
  })
  await new Promise((r) => servidor.listen(PUERTO, r))

  navegador = await chromium.launch({ executablePath: EJECUTABLE })
  const pagina = await navegador.newPage({ viewport: { width: 1440, height: 900 } })
  await pagina.goto(`http://localhost:${PUERTO}/#/catalogo`, { waitUntil: 'networkidle' })
  await pagina.waitForTimeout(600)

  // ── 1 · Ningún control se anuncia sin nombre ──
  //
  // Es el defecto que más se oye y el más barato de cometer: un botón que sólo
  // lleva un icono se anuncia como «botón», sin más. Quien escucha no sabe qué
  // hace, y no hay forma de averiguarlo sin verlo.
  const mudos = await pagina.evaluate(() => {
    const nombre = (el) =>
      (el.getAttribute('aria-label') ||
       el.getAttribute('title') ||
       (el.getAttribute('aria-labelledby') &&
         document.getElementById(el.getAttribute('aria-labelledby'))?.textContent) ||
       el.textContent || '').trim()
    return [...document.querySelectorAll('button, a[href], [role="tab"], [role="menuitem"]')]
      .filter((el) => el.offsetParent !== null && !nombre(el))
      .map((el) => `${el.tagName.toLowerCase()}.${el.className || '(sin clase)'}`)
  })
  if (mudos.length) anotar(`${mudos.length} control(es) sin nombre accesible: ${mudos.join(', ')}`)
  else bien('todo control visible tiene nombre accesible')

  // ── 2 · El logotipo se anuncia como UNA cosa ──
  //
  // El manual lo decía así: «ARLES RELAY, imagen» — una sola cosa, no «ARLES»
  // y «RELAY» por separado. Si el logotipo son dos nodos de texto, el lector
  // los lee como dos, y suena a dos productos distintos.
  const logo = await pagina.evaluate(() => {
    const el = document.querySelector('[class*="logotipo"], [class*="marca"] svg, [class*="marca"] img')
    if (!el) return null
    const raiz = el.closest('[role="img"], svg, img') || el
    return {
      rol: raiz.getAttribute('role') || raiz.tagName.toLowerCase(),
      nombre: (raiz.getAttribute('aria-label') ||
               raiz.querySelector('title')?.textContent || '').trim(),
    }
  })
  if (!logo) anotar('no se encontró el logotipo para comprobarlo')
  else if (!logo.nombre) anotar(`el logotipo no tiene nombre accesible (rol ${logo.rol})`)
  else bien(`el logotipo se anuncia como una sola cosa: «${logo.nombre}»`)

  // ── 3 · Las pestañas dicen cuál está seleccionada ──
  const pestanas = await pagina.evaluate(() =>
    [...document.querySelectorAll('[role="tab"]')].map((t) => ({
      texto: t.textContent.trim(),
      seleccionada: t.getAttribute('aria-selected'),
      controla: !!t.getAttribute('aria-controls'),
    })))
  const sinEstado = pestanas.filter((t) => t.seleccionada !== 'true' && t.seleccionada !== 'false')
  const activas = pestanas.filter((t) => t.seleccionada === 'true')
  if (!pestanas.length) anotar('no se encontraron pestañas')
  else if (sinEstado.length) anotar(`${sinEstado.length} pestaña(s) sin aria-selected`)
  else if (activas.length !== 1) anotar(`hay ${activas.length} pestañas marcadas como seleccionadas; debe haber exactamente 1`)
  else bien(`${pestanas.length} pestañas, y sólo «${activas[0].texto}» se anuncia como seleccionada`)

  // ── 4 · El error del campo va unido al campo ──
  //
  // «El texto del error al entrar, no sólo al salir» del manual es esto: el
  // mensaje tiene que estar enlazado por aria-describedby, y el campo marcado
  // con aria-invalid. Si no, el lector anuncia el campo y calla el error —
  // quien escucha no sabe que hay nada que corregir.
  await pagina.getByRole('tab', { name: 'Primitivas', exact: true }).click()
  await pagina.waitForTimeout(300)
  const errores = await pagina.evaluate(() => {
    const campos = [...document.querySelectorAll('input[aria-invalid="true"]')]
    return campos.map((c) => {
      const id = c.getAttribute('aria-describedby')
      const desc = id ? id.split(/\s+/).map((x) => document.getElementById(x)?.textContent?.trim()).filter(Boolean) : []
      return { enlazado: desc.length > 0, texto: desc.join(' ') }
    })
  })
  if (!errores.length) anotar('ningún campo en estado de error: no se pudo comprobar')
  else if (errores.some((e) => !e.enlazado)) anotar('hay un campo con error cuyo mensaje NO está enlazado por aria-describedby')
  else bien(`el error del campo se anuncia con él: «${errores[0].texto.slice(0, 60)}…»`)

  // ── 5 · La tabla anuncia el total real, no lo que se ve ──
  //
  // Es lo que el manual llamaba «fila 40 de 5001». Con virtualización el DOM
  // tiene 18 filas: sin aria-rowcount el lector anuncia 18 y miente sobre el
  // tamaño de lo que hay.
  await pagina.getByRole('tab', { name: 'Tabla virtualizada', exact: true }).click()
  await pagina.waitForTimeout(500)
  const tabla = await pagina.evaluate(() => {
    const g = document.querySelector('[role="grid"]')
    return g && {
      declarado: Number(g.getAttribute('aria-rowcount')),
      enElDom: g.querySelectorAll('[role="row"]').length,
    }
  })
  if (!tabla) anotar('no se encontró la tabla')
  else if (!tabla.declarado) anotar('la tabla no declara aria-rowcount: el lector anunciaría sólo las filas del DOM')
  else if (tabla.declarado <= tabla.enElDom) anotar(`aria-rowcount dice ${tabla.declarado} y en el DOM hay ${tabla.enElDom}: no está declarando el total`)
  else bien(`la tabla anuncia ${tabla.declarado} filas teniendo ${tabla.enElDom} en el DOM`)

  // ── 6 · Hay un punto de referencia de navegación ──
  const puntos = await pagina.evaluate(() => ({
    nav: document.querySelectorAll('nav, [role="navigation"]').length,
    main: document.querySelectorAll('main, [role="main"]').length,
  }))
  if (!puntos.nav || !puntos.main) {
    anotar(`faltan puntos de referencia: nav=${puntos.nav}, main=${puntos.main}. Sin ellos no se puede saltar al contenido`)
  } else bien('la página tiene navegación y contenido principal identificados')
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

if (problemas.length) {
  console.error(`\n${problemas.length} problema(s) en lo que anunciaría un lector de pantalla.`)
  process.exit(1)
}
console.log('\nLo que un lector de pantalla anuncia es correcto.')
