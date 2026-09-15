/**
 * Sonda de desbordamiento horizontal.
 *
 *     npm --prefix app run sonda:ancho
 *
 * Por qué existe. La revisión visual del 14 de septiembre encontró que por
 * encima del **150 % de escala de Windows** la aplicación desborda a lo ancho:
 * aparece una barra de desplazamiento horizontal y el contenido se corta por
 * el borde derecho.
 *
 * La causa fue un razonamiento que parecía sólido: «la ventana no puede ser
 * más estrecha de 1120 px, así que el contenido nunca tiene que bajar de ahí»,
 * y de ahí el `min-width: 1120px` del armazón. Es falso en cuanto la escala
 * pasa del 100 %, porque el mínimo de la ventana está en píxeles **lógicos**:
 * a 200 %, en una pantalla de 1920, el viewport CSS es de 960 y no hay forma
 * de que llegue a 1120.
 *
 * Confundimos el mínimo de la VENTANA con el mínimo del DISEÑO. A escala alta
 * dejan de ser el mismo número.
 *
 * Por qué no lo vio ninguna comprobación anterior: las otras sondas corren a
 * 1440×900 y a 1120×720, las dos por encima del umbral. Nadie probó por
 * debajo, porque el razonamiento decía que era imposible. Era imposible en
 * píxeles físicos, no en lógicos — y el navegador trabaja en lógicos.
 *
 * Esta sonda recorre los anchos que produce cada escala de Windows en una
 * pantalla de 1920 y falla si alguno desborda.
 */
import { spawnSync } from 'node:child_process'
import { readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const SALIDA = join(APP, 'dist-ancho')
const PUERTO = 4177
const EJECUTABLE = exigirChromium()

// El ancho de viewport CSS que deja cada escala de Windows en una pantalla de
// 1920 px, que es la más común. La de 100 % está para que se vea que la sonda
// también pasa por el caso bueno.
const ESCALAS = [
  [100, 1920],
  [125, 1536],
  [150, 1280],
  [175, 1097],
  [200, 960],
  [250, 768],
]

const problemas = []
const anotar = (q) => { problemas.push(q); console.log(`  ✗ ${q}`) }
const bien = (q) => console.log(`  ✓ ${q}`)

let servidor
let navegador

try {
  const build = spawnSync(
    'npx',
    ['vite', 'build', '--outDir', 'dist-ancho', '--emptyOutDir'],
    { cwd: APP, encoding: 'utf8', env: { ...process.env, VITE_ARLES_CATALOGO: '1' } },
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
    // El tipo se resuelve sobre la RUTA DEL ARCHIVO, no sobre la URL:
    // extname('/') es la cadena vacía, y el index.html se serviría como
    // octet-stream — el navegador lo descargaría en vez de pintarlo.
    const archivo = join(SALIDA, ruta === '/' ? 'index.html' : ruta)
    try {
      const cuerpo = await readFile(archivo)
      res.writeHead(200, { 'Content-Type': TIPOS[extname(archivo)] ?? 'application/octet-stream' })
      res.end(cuerpo)
    } catch { res.writeHead(404); res.end() }
  })
  await new Promise((r) => servidor.listen(PUERTO, r))

  navegador = await chromium.launch({ executablePath: EJECUTABLE })

  for (const [escala, ancho] of ESCALAS) {
    const pagina = await navegador.newPage({ viewport: { width: ancho, height: 700 } })
    await pagina.goto(`http://localhost:${PUERTO}/#/catalogo`, { waitUntil: 'networkidle' })
    await pagina.waitForTimeout(350)

    const medida = await pagina.evaluate(() => ({
      pedido: document.documentElement.scrollWidth,
      disponible: document.documentElement.clientWidth,
      // Quién se sale. Sin esto el fallo dice «desborda» y no dónde mirar.
      culpables: [...document.querySelectorAll('body *')]
        .filter((el) => el.getBoundingClientRect().right > document.documentElement.clientWidth + 1)
        .slice(0, 4)
        .map((el) => el.className || el.tagName),
    }))
    await pagina.close()

    const etiqueta = `escala ${escala} % (viewport ${ancho} px)`
    if (medida.pedido > medida.disponible) {
      anotar(
        `${etiqueta}: la página pide ${medida.pedido} px y sólo hay ${medida.disponible}. ` +
        `Se salen: ${medida.culpables.join(', ') || '(no identificado)'}`,
      )
    } else {
      bien(`${etiqueta}: sin desbordamiento`)
    }
  }
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SALIDA, { recursive: true, force: true })
}

if (problemas.length) {
  console.error(
    `\n${problemas.length} escala(s) desbordan a lo ancho.\n` +
    'Recuerda: el mínimo de la ventana está en píxeles lógicos. A 200 % de\n' +
    'escala, 1120 lógicos son 2240 físicos — más que la pantalla entera.',
  )
  process.exit(1)
}
console.log('\nNinguna escala de Windows provoca desplazamiento horizontal.')
