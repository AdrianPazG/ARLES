/**
 * La vista previa de un solo archivo se abre y funciona desde el disco.
 *
 *     node app/pruebas/sondas/vista-previa.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE MIDE Y POR QUÉ
 *
 * `herramientas/vista-previa/generar.py` empaqueta ARLES en un único HTML que
 * Dirección abre con doble clic. Que ese archivo **exista** no dice nada: las
 * dos primeras versiones se generaron sin un error y salieron **en blanco**.
 *
 *   1. El enrutador carga cada pantalla con un `import()` dinámico, que en un
 *      build normal produce un archivo por pantalla. Desde `file://` el
 *      navegador se niega a cargarlos y no queda rastro visible.
 *   2. El producto declara `default-src 'self'`, que **prohíbe el script en
 *      línea** — y en la vista previa todo va en línea. El navegador se negó a
 *      ejecutar nada, que es exactamente lo que esa política existe para hacer.
 *
 * Las dos veces el síntoma fue el mismo: un archivo de medio megabyte que se
 * abre y no enseña nada. Por eso esto se mide **abriéndolo desde `file://`**,
 * como lo abrirá quien lo reciba, y no sirviéndolo por HTTP — que es donde los
 * dos fallos desaparecen.
 * ─────────────────────────────────────────────────────────────────────────
 *
 * Recorre además los renglones de la checklist que se pueden automatizar: la
 * portada de primera vez, guardar la empresa, el tema y que sobreviva a
 * recargar. Lo que no se puede automatizar —si se entiende— sigue siendo tarea
 * de una persona.
 */
import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'

const RAIZ = fileURLToPath(new URL('../../..', import.meta.url))
const ARCHIVO = `${RAIZ}vista-previa/ARLES-vista-previa.html`

let navegador
let codigo = 0

function mal(texto) {
  console.error(`\n✗ ${texto}`)
  codigo = 1
}

try {
  console.log('Generando la vista previa…')
  const gen = spawnSync('python3', ['herramientas/vista-previa/generar.py'], {
    cwd: RAIZ,
    encoding: 'utf8',
  })
  if (gen.status !== 0) {
    console.error(gen.stderr || gen.stdout)
    process.exit(1)
  }
  if (!existsSync(ARCHIVO)) {
    console.error('el generador terminó bien pero no dejó el archivo')
    process.exit(1)
  }

  navegador = await chromium.launch({ executablePath: exigirChromium() })
  // `colorScheme: 'dark'` no es decoración: con «Automático», el tema sale del
  // sistema. Si el navegador arrancara en claro —que es lo que hace por
  // defecto—, elegir «Claro» no cambiaría nada y la comprobación de abajo
  // acusaría al producto de un fallo suyo. Pasó, y por eso está escrito.
  const pagina = await navegador.newPage({
    viewport: { width: 1360, height: 900 },
    colorScheme: 'dark',
  })

  // Todo lo que el navegador se queje, se recoge. Un archivo que «funciona»
  // con errores en la consola es un archivo que funcionará a medias en otro
  // navegador.
  const quejas = []
  pagina.on('pageerror', (e) => quejas.push(`error: ${e.message}`))
  pagina.on('console', (m) => {
    if (m.type() === 'error') quejas.push(`consola: ${m.text().slice(0, 200)}`)
  })
  pagina.on('requestfailed', (r) => quejas.push(`petición: ${r.url().slice(0, 120)}`))

  // `file://`, no `http://`. Es la diferencia entre medir lo que va a pasar y
  // medir un caso que no se va a dar.
  await pagina.goto(`file://${ARCHIVO}`, { waitUntil: 'load' })
  await pagina.waitForTimeout(1200)

  const pintado = (await pagina.locator('#app').innerHTML()).length
  if (pintado < 500) {
    mal(
      'La vista previa se abre en blanco.\n' +
        '  Ha pasado dos veces: por los trozos del enrutador y por la CSP.\n' +
        '  Revisa los mensajes de abajo antes que nada.',
    )
  } else {
    console.log(`✓ Se abre desde el disco y pinta la interfaz (${pintado} caracteres).`)
  }

  // A · lo primero que se ve es la portada de primera vez.
  if ((await pagina.locator('.portada').count()) !== 1) {
    mal('No sale la portada de primera vez: el bloque A de la checklist no se puede recorrer.')
  } else {
    console.log('✓ A · arranca en la portada de primera vez.')
  }

  // A-3 y A-4 · configurar la empresa.
  await pagina.click('text=Configurar mi empresa')
  await pagina.waitForTimeout(500)
  if (!pagina.url().includes('ajustes')) {
    mal('El botón de la portada no lleva a Ajustes.')
  }
  await pagina.fill('input[autocomplete="organization"]', 'TELEMETRY INSIGHT')
  await pagina.fill('input[type="email"]', 'hola@telemetrymx.com')
  await pagina.click('button[type="submit"]')
  await pagina.waitForTimeout(600)
  if ((await pagina.locator('text=Configuración guardada').count()) !== 1) {
    mal('Guardar la empresa no confirma nada: el núcleo simulado no responde como el real.')
  } else {
    console.log('✓ A-3 y A-4 · se configura la empresa y se confirma.')
  }

  // E-1 · el tema cambia el píxel, no sólo el atributo.
  const fondoAntes = await pagina.evaluate(() => getComputedStyle(document.body).backgroundColor)
  await pagina.selectOption('.apariencia select', 'claro')
  await pagina.waitForTimeout(300)
  const fondoDespues = await pagina.evaluate(() => getComputedStyle(document.body).backgroundColor)
  if (fondoAntes === fondoDespues) {
    mal(`El tema claro no cambia el color: sigue en ${fondoDespues}.`)
  } else {
    console.log(`✓ E-1 · el tema claro cambia el fondo (${fondoAntes} → ${fondoDespues}).`)
  }

  // E-3 y A-5 · recargar es el «cerrar y volver a abrir» de la checklist.
  await pagina.reload({ waitUntil: 'load' })
  await pagina.waitForTimeout(1000)
  const tema = await pagina.evaluate(() =>
    document.documentElement.getAttribute('data-tema'),
  )
  if (tema !== 'claro') {
    mal(`Tras recargar, el tema volvió a «${tema}». E-3 no se puede recorrer.`)
  } else if ((await pagina.locator('.portada').count()) !== 0) {
    mal('Tras recargar vuelve a salir la portada de primera vez: la empresa no se recuerda.')
  } else {
    console.log('✓ E-3 y A-5 · el tema y la empresa sobreviven a recargar.')
  }

  // La tipografía de la marca, que es lo que distingue esta copia del
  // instalador. Si se generó con `--sin-mont`, esto lo dice en vez de fallar.
  const fuente = await pagina.evaluate(() =>
    getComputedStyle(document.body).fontFamily.split(',')[0].replace(/"/g, ''),
  )
  console.log(`  tipografía en uso: ${fuente}`)

  if (quejas.length) {
    mal(`El navegador se quejó ${quejas.length} veces:`)
    for (const q of quejas.slice(0, 6)) console.error(`    ${q}`)
  } else {
    console.log('✓ Ni un error de consola ni una petición fallida.')
  }
} finally {
  await navegador?.close()
}

process.exit(codigo)
