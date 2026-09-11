/**
 * Sonda de CSP.
 *
 * Sirve el build de producción **bajo la política exacta de
 * `crates/arles-app/tauri.conf.json`** —leída del archivo, no copiada aquí— y
 * ejercita las primitivas que tocan estilo en línea: la tabla virtualizada
 * (transform y alto por fila), el modal (sombra y fondo oscurecido) y el menú
 * (posicionamiento). Cualquier violación de CSP o error de página falla.
 *
 * Por qué hace falta. La Fase 1 dejó registrado (F14) que `style-src` llevaba
 * `'unsafe-inline'` como riesgo aceptado. Al medirlo resultó que **la
 * aplicación empaquetada no lo necesita**: Vite extrae las hojas de los
 * componentes a un `.css`, y los `:style` de Vue se aplican con
 * `element.style.setProperty`, que la CSP no gobierna. Se retiró.
 *
 * Esta sonda existe para que no vuelva a entrar sin que nadie lo note: si
 * alguien añade un `<style>` inyectado o un atributo `style=` en la plantilla,
 * aquí salta.
 *
 *     npm --prefix app run sonda:csp
 *
 * Compila con el catálogo forzado dentro —normalmente es sólo de desarrollo—
 * porque es la única pantalla que usa todas las primitivas. El artefacto es
 * temporal: se construye en `dist-csp/` y se borra al terminar.
 */
import { spawnSync } from 'node:child_process'
import { readFile, rm, writeFile } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-csp')
const RUTA_ROUTER = join(APP, 'src/app/router.ts')
const CONF = join(APP, '../crates/arles-app/tauri.conf.json')
const EJECUTABLE =
  process.env['ARLES_CHROMIUM'] ?? '/opt/pw-browsers/chromium-1194/chrome-linux/chrome'
const PUERTO = 4173

const problemas = []
const anotar = (q) => {
  problemas.push(q)
  console.log(`  ✗ ${q}`)
}
const bien = (q) => console.log(`  ✓ ${q}`)

// ── La política se lee del archivo que gobierna el producto ───────────────
const csp = JSON.parse(await readFile(CONF, 'utf8')).app.security.csp
const politica = Object.entries(csp)
  .map(([k, v]) => `${k} ${v}`)
  .join('; ')

if (/unsafe-inline|unsafe-eval/.test(politica)) {
  anotar(`la CSP del producto admite ${/unsafe-eval/.test(politica) ? 'unsafe-eval' : 'unsafe-inline'}`)
} else {
  bien('la CSP del producto no admite unsafe-inline ni unsafe-eval')
}

// ── Build de verificación con el catálogo dentro ──────────────────────────
const routerOriginal = await readFile(RUTA_ROUTER, 'utf8')
let servidor
let navegador

try {
  await writeFile(
    RUTA_ROUTER,
    routerOriginal.replace(
      'if (import.meta.env.DEV) {',
      'if (true) { // forzado por la sonda de CSP',
    ),
  )

  const build = spawnSync(
    'npx',
    ['vite', 'build', '--outDir', 'dist-csp', '--emptyOutDir'],
    { cwd: APP, encoding: 'utf8' },
  )
  if (build.status !== 0) {
    anotar(`no compiló: ${(build.stderr || build.stdout).slice(-400)}`)
    throw new Error('build')
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
    // extname('/') es la cadena vacía: sin resolver primero a index.html, el
    // HTML se serviría como octet-stream y el navegador lo descargaría en vez
    // de renderizarlo.
    const archivo = join(SALIDA, ruta === '/' ? 'index.html' : ruta)
    try {
      const cuerpo = await readFile(archivo)
      res.writeHead(200, {
        'Content-Type': TIPOS[extname(archivo)] ?? 'application/octet-stream',
        // Se sirve la política del producto como cabecera, igual que hace
        // Tauri. El <meta> de index.html es más laxo por el modo desarrollo;
        // cuando coinciden, gana la intersección — que es lo que se prueba.
        'Content-Security-Policy': politica,
      })
      res.end(cuerpo)
    } catch {
      res.writeHead(404)
      res.end()
    }
  })
  await new Promise((r) => servidor.listen(PUERTO, r))

  navegador = await chromium.launch({ executablePath: EJECUTABLE })
  const pagina = await navegador.newPage({ viewport: { width: 1440, height: 900 } })

  const violaciones = []
  pagina.on('console', (m) => {
    if (/Content Security Policy|Refused to/i.test(m.text())) violaciones.push(m.text())
  })
  pagina.on('pageerror', (e) => violaciones.push(`excepción: ${e.message}`))

  await pagina.goto(`http://localhost:${PUERTO}/#/catalogo`, { waitUntil: 'networkidle' })
  await pagina.waitForTimeout(600)

  // Tabla: estilo en línea por fila (transform, height, grid-template-columns)
  await pagina.getByRole('tab', { name: 'Tabla virtualizada', exact: true }).click()
  await pagina.waitForTimeout(500)
  await pagina.locator('[role="grid"] .cuerpo').evaluate((el) => el.scrollTo(0, 9000))
  await pagina.waitForTimeout(300)
  const filas = await pagina.locator('[role="row"]').count()
  if (filas < 2) anotar('la tabla no renderizó filas bajo la CSP del producto')
  else bien(`la tabla virtualiza bajo la CSP del producto (${filas} nodos)`)

  // Modal: sombra y fondo oscurecido
  await pagina.getByRole('tab', { name: 'Primitivas', exact: true }).click()
  await pagina.waitForTimeout(300)
  await pagina.getByRole('button', { name: 'Modal destructivo' }).click()
  await pagina.waitForTimeout(400)
  const fondo = await pagina
    .locator('dialog[open]')
    .evaluate((el) => getComputedStyle(el).backgroundColor)
  if (fondo === 'rgba(0, 0, 0, 0)') anotar('el modal se abrió sin fondo: la hoja no se aplicó')
  else bien('el modal se pinta bajo la CSP del producto')
  await pagina.getByRole('button', { name: 'Seguir enviando' }).click()
  await pagina.waitForTimeout(200)

  // Menú: posicionamiento absoluto y sombra
  await pagina.getByRole('button', { name: /Acciones/ }).click()
  await pagina.waitForTimeout(300)
  if (!(await pagina.locator('[role="menu"]').isVisible())) {
    anotar('el menú no se abrió bajo la CSP del producto')
  } else bien('el menú se abre bajo la CSP del producto')

  // La tipografía también pasa por la CSP: font-src.
  const mont = await pagina.evaluate(async () => {
    await document.fonts.ready
    return [...document.fonts].filter((f) => f.status === 'loaded').length
  })
  if (mont !== 4) anotar(`bajo la CSP del producto cargaron ${mont}/4 cortes de Mont`)
  else bien('los cuatro cortes de Mont cargan bajo font-src del producto')

  for (const v of violaciones) anotar(`violación: ${v}`)
  if (violaciones.length === 0) bien('ninguna violación de CSP en todo el recorrido')
} finally {
  await writeFile(RUTA_ROUTER, routerOriginal)
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

if (problemas.length) {
  console.error(`\n${problemas.length} problema(s) de CSP.`)
  process.exit(1)
}
console.log('\nLa aplicación empaquetada funciona sin estilo en línea.')
