/**
 * Sonda del umbral de plegado automático de la barra lateral.
 *
 *     npm --prefix app run sonda:plegado
 *
 * P-11 decidió que la barra se pliega «a mano **y** sola». «Sola» necesita un
 * número, y el número no se elige a ojo: el defecto R-01 nació justamente de
 * razonar un ancho en vez de medirlo.
 *
 * Qué se mide, y por qué **no** es el desbordamiento.
 *
 * La primera versión de esta sonda buscaba el ancho al que la aplicación
 * empieza a desbordar con la barra desplegada, como hace `sonda:ancho`. No
 * encontró ninguno: hasta 600 px no desborda nada, porque las pantallas de la
 * 3.1 son fluidas y todo se estrecha en vez de cortarse. **Ese resultado dejó
 * el umbral sin nada que lo respaldara**, y así se dijo en vez de inventar un
 * número.
 *
 * Lo que sí se rompe antes es la **medida de diseño**: cada pantalla declara en
 * `--arles-medida` el ancho por debajo del cual deja de funcionar, y por
 * debajo de cierto viewport deja de alcanzarlo. Plegar devuelve 176 px (240 →
 * 64), que es justo lo que hace falta para recuperarla.
 *
 * Así que el umbral es: **el ancho por debajo del cual la pantalla más exigente
 * ya no cabe a su medida de diseño con la barra desplegada.** Se mide, no se
 * elige.
 *
 * Tres comprobaciones, y hacen falta las tres:
 *
 *   1 · Dónde está ese suelo, con el plegado automático desactivado.
 *   2 · Que el umbral declarado no está por debajo (se cortaría la medida) ni
 *       muy por encima (se plegaría sola donde no hace falta). Sin esta
 *       segunda mitad, el umbral podría subirse a 4000 px y todo seguiría en
 *       verde con la barra plegada siempre — que es lo contrario de lo que
 *       P-11 decidió.
 *   3 · Que plegada cabe en los anchos de Windows a escala alta —960 px al
 *       200 %, 768 px al 250 %— y que ahí los enlaces **siguen teniendo nombre
 *       accesible**, que es lo que se pierde al quitarles el texto.
 */
import { readFile, rm } from 'node:fs/promises'
import http from 'node:http'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { vite } from './ordenes.mjs'

const APP = fileURLToPath(new URL('../..', import.meta.url))
const PUERTO = 4179
const EJECUTABLE = exigirChromium()

/**
 * Holgura admitida entre el suelo medido y el umbral declarado.
 *
 * 40 px: menos de lo que ahorra plegar (240 − 64 = 176), así que un umbral
 * inflado no puede esconderse dentro de ella, y suficiente para que un cambio
 * tipográfico de unos pocos píxeles no obligue a retocar el número cada
 * semana.
 */
const HOLGURA_PX = 40

/** Paso del barrido. Más fino no aporta: la holgura admitida es mayor. */
const PASO_PX = 4

/** Los anchos que deja Windows a escala alta en una pantalla de 1920. */
const ANCHOS_DE_ESCALA_ALTA = [
  [200, 960],
  [250, 768],
]

const problemas = []
const anotar = (q) => { problemas.push(q); console.log(`  ✗ ${q}`) }
const bien = (q) => console.log(`  ✓ ${q}`)

/** El número declarado en el código, leído del fuente para no duplicarlo. */
async function umbralDeclarado() {
  const fuente = await readFile(join(APP, 'src/app/stores/interfaz.ts'), 'utf8')
  const m = /UMBRAL_MEDIDO_PX = (\d+)/.exec(fuente)
  if (!m) throw new Error('no se encontró UMBRAL_MEDIDO_PX en stores/interfaz.ts')
  return Number(m[1])
}

function compilar(salida, entorno) {
  const r = vite(['build', '--outDir', salida, '--emptyOutDir'], {
    cwd: APP,
    encoding: 'utf8',
    env: { ...process.env, ...entorno },
  })
  if (r.status !== 0) {
    throw new Error(`no compiló: ${(r.stderr || r.stdout).slice(-400)}`)
  }
}

const TIPOS = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.woff2': 'font/woff2',
  '.png': 'image/png',
}

function servir(directorio) {
  const s = http.createServer(async (req, res) => {
    const ruta = (req.url ?? '/').split('?')[0]
    // El tipo se resuelve sobre la RUTA DEL ARCHIVO, no sobre la URL:
    // extname('/') es la cadena vacía y el index.html se serviría como
    // octet-stream. Ya mordió dos veces; aquí va bien desde el principio.
    const archivo = join(directorio, ruta === '/' ? 'index.html' : ruta)
    try {
      const cuerpo = await readFile(archivo)
      res.writeHead(200, { 'Content-Type': TIPOS[extname(archivo)] ?? 'application/octet-stream' })
      res.end(cuerpo)
    } catch { res.writeHead(404); res.end() }
  })
  return new Promise((r) => s.listen(PUERTO, () => r(s)))
}

/** Estado de una pantalla a un ancho dado. */
async function medir(pagina, ancho, ruta) {
  await pagina.setViewportSize({ width: ancho, height: 700 })
  if (ruta) {
    await pagina.goto(`http://localhost:${PUERTO}/#${ruta}`, { waitUntil: 'networkidle' })
  }
  await pagina.waitForTimeout(120)
  const r = await pagina.evaluate(() => {
    const raiz = document.documentElement
    const pantalla = document.querySelector('.pantalla')
    const estilo = pantalla && getComputedStyle(pantalla)
    return {
      pedido: raiz.scrollWidth,
      disponible: raiz.clientWidth,
      plegada: !!document.querySelector('.marco.plegado'),
      // La medida de diseño: el ancho POR DEBAJO DEL CUAL la pantalla deja
      // de funcionar, declarado por ella misma en `--arles-medida`.
      //
      // Antes se leía `max-width`, y eso sólo vale para una pantalla de
      // lectura: un formulario que se capa a 540 px pide exactamente 540. Un
      // panel modular no: su `max-width` es un tope estético —para que las
      // líneas no crucen la pantalla— y su necesidad real es el ancho por
      // debajo del cual la rejilla se apila. Con `max-width` como medida, un
      // panel generoso habría empujado el umbral de plegado hasta casi el
      // ancho de la ventana.
      //
      // Se cambió el criterio porque cambió lo medido, no para que pasara:
      // la sonda sigue fallando por los dos lados y se probó rompiéndola.
      medidaDeDiseno: estilo
        ? parseFloat(estilo.getPropertyValue('--arles-medida'))
        : null,
      anchoReal: pantalla ? pantalla.getBoundingClientRect().width : null,
      // Nombre accesible de cada enlace de navegación. Plegada se les quita el
      // texto a la vista, y aquí se comprueba que no se les quita el nombre.
      enlacesSinNombre: [...document.querySelectorAll('.nav-enlace')].filter(
        (a) => !(a.getAttribute('aria-label') || a.textContent || '').trim(),
      ).length,
      enlaces: document.querySelectorAll('.nav-enlace').length,
      culpables: [...document.querySelectorAll('body *')]
        .filter((el) => el.getBoundingClientRect().right > raiz.clientWidth + 1)
        .slice(0, 3)
        .map((el) => el.className || el.tagName),
    }
  })
  return {
    ...r,
    desborda: r.pedido > r.disponible,
    // Medio píxel de margen: el ancho real se redondea al subpíxel.
    aMedida:
      r.medidaDeDiseno === null || r.anchoReal >= r.medidaDeDiseno - 0.5,
  }
}

// Las dos pantallas reales de la 3.1. El catálogo no cuenta: no viaja al
// cliente, y medir el umbral del producto con una pantalla de revisión daría
// un número que no es el del producto.
const PANTALLAS = ['/inicio', '/ajustes']

const SIN_PLEGADO = join(APP, 'dist-plegado-libre')
const NORMAL = join(APP, 'dist-plegado-normal')

let servidor
let navegador

try {
  const umbral = await umbralDeclarado()
  console.log(`  · umbral declarado: ${umbral} px\n`)

  // ── 1 · Dónde está el suelo de la medida de diseño ──
  compilar('dist-plegado-libre', { VITE_ARLES_UMBRAL_PLEGADO: '1' })
  servidor = await servir(SIN_PLEGADO)
  navegador = await chromium.launch({ executablePath: EJECUTABLE })

  let suelo = 0
  let pantallaDelSuelo = null

  for (const pantalla of PANTALLAS) {
    const pagina = await navegador.newPage({ viewport: { width: 1400, height: 700 } })
    const holgada = await medir(pagina, 1400, pantalla)

    if (holgada.plegada) {
      anotar(
        'la barra se plegó sola con el plegado desactivado: la compilación de ' +
        'medición no está usando VITE_ARLES_UMBRAL_PLEGADO',
      )
      await pagina.close()
      throw new Error('medición inválida')
    }
    if (holgada.medidaDeDiseno === null || Number.isNaN(holgada.medidaDeDiseno)) {
      anotar(
        `${pantalla} no declara \`--arles-medida\`: sin el ancho por debajo ` +
        'del cual deja de funcionar no hay nada que medir, y el umbral de ' +
        'plegado se quedaría sin respaldo',
      )
      await pagina.close()
      continue
    }

    let apretada = null
    for (let ancho = 1400; ancho >= 600; ancho -= PASO_PX) {
      const m = await medir(pagina, ancho, null)
      if (!m.aMedida) { apretada = ancho; break }
    }
    await pagina.close()

    if (apretada === null) {
      anotar(`${pantalla}: mantiene su medida hasta 600 px; la medición no dice nada`)
      continue
    }

    const ultimoHolgado = apretada + PASO_PX
    bien(
      `${pantalla}: mantiene sus ${Math.round(holgada.medidaDeDiseno)} px de ` +
      `medida hasta ${ultimoHolgado} px de ventana`,
    )
    if (ultimoHolgado > suelo) { suelo = ultimoHolgado; pantallaDelSuelo = pantalla }
  }

  await navegador.close(); navegador = undefined
  servidor.close(); servidor = undefined

  if (suelo === 0) {
    anotar('ninguna pantalla dio un suelo medible: el umbral no queda respaldado')
  } else if (umbral < suelo) {
    anotar(
      `el umbral declarado (${umbral} px) está por debajo del suelo medido ` +
      `(${suelo} px, por ${pantallaDelSuelo}): entre los dos la barra sigue ` +
      'desplegada y la pantalla no alcanza su medida de diseño',
    )
  } else if (umbral > suelo + HOLGURA_PX) {
    anotar(
      `el umbral declarado (${umbral} px) excede el suelo medido (${suelo} px) ` +
      `en más de ${HOLGURA_PX} px: la barra se plegaría sola en anchos donde ` +
      'la pantalla cabe holgada',
    )
  } else {
    bien(`el umbral declarado (${umbral} px) se ajusta al suelo medido (${suelo} px)`)
  }

  // ── 2 · Con la compilación normal, plegada cabe donde tiene que caber ──
  compilar('dist-plegado-normal', {})
  servidor = await servir(NORMAL)
  navegador = await chromium.launch({ executablePath: EJECUTABLE })

  const pagina = await navegador.newPage({ viewport: { width: 1440, height: 700 } })

  for (const [escala, ancho] of ANCHOS_DE_ESCALA_ALTA) {
    for (const pantalla of PANTALLAS) {
      const m = await medir(pagina, ancho, pantalla)
      const etiqueta = `escala ${escala} % (${ancho} px) · ${pantalla}`
      if (!m.plegada) {
        anotar(`${etiqueta}: la barra NO se plegó sola por debajo del umbral`)
      } else if (m.desborda) {
        anotar(
          `${etiqueta}: plegada y aun así desborda ${m.pedido - m.disponible} px ` +
          `(${m.culpables.join(', ') || '?'})`,
        )
      } else if (m.enlaces !== 6) {
        anotar(`${etiqueta}: se ven ${m.enlaces} enlaces de navegación y deberían ser 6`)
      } else if (m.enlacesSinNombre > 0) {
        anotar(
          `${etiqueta}: ${m.enlacesSinNombre} enlace(s) plegados sin nombre ` +
          'accesible: un lector de pantalla los anunciaría como «enlace»',
        )
      } else {
        bien(`${etiqueta}: plegada, sin desbordar y con los 6 enlaces con nombre`)
      }
    }
  }

  // ── 3 · Por encima del umbral no se pliega sola ──
  const holgado = await medir(pagina, 1440, '/inicio')
  if (holgado.plegada) {
    anotar('a 1440 px la barra se pliega sola: el umbral está mal aplicado')
  } else {
    bien('a 1440 px la barra se queda desplegada')
  }
  await pagina.close()
} finally {
  await navegador?.close()
  servidor?.close()
  await rm(SIN_PLEGADO, { recursive: true, force: true })
  await rm(NORMAL, { recursive: true, force: true })
}

if (problemas.length) {
  console.error(`\n${problemas.length} problema(s) con el umbral de plegado.`)
  process.exit(1)
}
console.log('\nEl umbral de plegado está medido y se aplica.')
