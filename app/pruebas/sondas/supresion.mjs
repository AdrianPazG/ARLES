/**
 * La lista de quién pidió no recibir hace lo que dice.
 *
 *     node app/pruebas/sondas/supresion.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE MIDE
 *
 * Seis afirmaciones de la entrega 3.4 que ningún test de unidad alcanza,
 * porque son sobre lo que una persona ve y puede pulsar:
 *
 *   1. La lista se abre desde CONTACTOS y enseña las bajas.
 *   2. Cada fila dice **por qué** esa persona dejó de recibir.
 *   3. Las cifras separan correo de WhatsApp (§94).
 *   4. **Una baja que pidió la persona NO tiene botón de deshacer**, y la
 *      pantalla dice por qué en vez de dejar un hueco.
 *   5. Un rebote **sí** se puede deshacer, y al hacerlo desaparece.
 *   6. Buscar dice **sobre cuántas filas** está buscando, para no hacer creer
 *      que se buscó en la lista entera.
 *
 * La 4 es la que importa. Es la regla del módulo, y es justo la clase de cosa
 * que se rompe sin que nada falle: el botón aparecería, funcionaría, y una
 * baja que alguien pidió se habría deshecho con un clic.
 *
 * El núcleo es el simulado, así que esto NO demuestra que la supresión de Rust
 * funcione —eso lo demuestran sus pruebas—: demuestra que la pantalla pide lo
 * que debe y pinta lo que recibe.
 * ─────────────────────────────────────────────────────────────────────────
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
  const pagina = await navegador.newPage({
    viewport: { width: 1360, height: 900 },
    colorScheme: 'dark',
  })

  const quejas = []
  pagina.on('pageerror', (e) => quejas.push(`error: ${e.message}`))
  pagina.on('console', (m) => {
    if (m.type() === 'error') quejas.push(`consola: ${m.text().slice(0, 200)}`)
  })

  // ── 1 · Se llega desde CONTACTOS ────────────────────────────────────
  await pagina.goto(`file://${ARCHIVO}#/contactos`, { waitUntil: 'load' })
  await pagina.waitForTimeout(1200)

  const enlace = pagina.getByRole('link', { name: 'Quién pidió no recibir' })
  if ((await enlace.count()) === 0) {
    mal('Desde CONTACTOS no se puede llegar a la lista de bajas.')
    await pagina.goto(`file://${ARCHIVO}#/contactos/supresion`, { waitUntil: 'load' })
  } else {
    await enlace.first().click()
    console.log('✓ 1 · se llega desde CONTACTOS.')
  }
  await pagina.waitForTimeout(900)

  const filas = pagina.locator('.fila')
  const cuantas = await filas.count()
  if (cuantas < 5) {
    mal(`La lista enseña ${cuantas} bajas de 5: no se cargó bien.`)
  } else {
    console.log(`✓ 1 · la lista enseña ${cuantas} bajas.`)
  }

  // ── 2 · Cada fila dice por qué ──────────────────────────────────────
  const motivos = await pagina.locator('.fila__motivo').allInnerTexts()
  const sinMotivo = motivos.filter((m) => m.trim() === '').length
  if (sinMotivo > 0) {
    mal(`${sinMotivo} filas no dicen por qué esa persona dejó de recibir.`)
  } else if (!motivos.some((m) => m.includes('Pidió la baja'))) {
    mal(`Ninguna fila dice «Pidió la baja»: ${motivos.slice(0, 3).join(' | ')}`)
  } else {
    console.log('✓ 2 · cada fila dice por qué.')
  }

  // ── 3 · Las cifras separan los dos canales ──────────────────────────
  const cifras = (await pagina.locator('.supresion__cifras').innerText()).replace(/\s+/g, ' ')
  if (!/\d+ por correo/.test(cifras) || !/\d+ por WhatsApp/.test(cifras)) {
    mal(`Las cifras no separan correo de WhatsApp: «${cifras}»`)
  } else {
    console.log(`✓ 3 · las cifras separan los canales: ${cifras.trim()}`)
  }

  // ── 4 · LA REGLA · lo que pidió la persona no se deshace ────────────
  //
  // Se mide fila por fila y no en conjunto: «hay algún candado» pasaría
  // aunque el candado estuviera en la fila equivocada.
  let conBoton = []
  let candadosBien = 0
  for (let i = 0; i < cuantas; i += 1) {
    const fila = filas.nth(i)
    const texto = (await fila.innerText()).replace(/\s+/g, ' ')
    const tieneBoton =
      (await fila.getByRole('button', { name: 'Volver a permitir' }).count()) > 0
    const loPidio =
      texto.includes('Pidió la baja') || texto.includes('no deseado')

    if (loPidio && tieneBoton) {
      conBoton.push(texto.slice(0, 80))
    }
    if (loPidio && !tieneBoton) {
      if (texto.includes('Lo pidió ella misma')) candadosBien += 1
      else mal(`Una fila sin botón no explica por qué: ${texto.slice(0, 80)}`)
    }
  }

  if (conBoton.length > 0) {
    mal(
      'Hay bajas que la persona pidió y que se pueden deshacer con un botón:\n' +
        conBoton.map((f) => `    ${f}`).join('\n'),
    )
  } else if (candadosBien < 2) {
    mal(
      `Sólo ${candadosBien} filas bloqueadas y explicadas; la semilla trae al ` +
        'menos 2 (una baja y una queja).',
    )
  } else {
    console.log(`✓ 4 · las ${candadosBien} bajas que pidió la persona no se deshacen, y se dice.`)
  }

  // ── 5 · Un rebote sí se deshace ─────────────────────────────────────
  const antes = await filas.count()
  const filaDelRebote = filas.filter({ hasText: 'rebote@empresa.mx' })
  if ((await filaDelRebote.count()) === 0) {
    mal('No está la fila del rebote: no se puede probar que se deshaga.')
  } else {
    pagina.once('dialog', (d) => void d.accept())
    await filaDelRebote
      .getByRole('button', { name: 'Volver a permitir' })
      .click()
    await pagina.waitForTimeout(700)

    const despues = await filas.count()
    if (despues !== antes - 1) {
      mal(`Tras deshacer el rebote quedan ${despues} filas de ${antes - 1} esperadas.`)
    } else if ((await filas.filter({ hasText: 'rebote@empresa.mx' }).count()) !== 0) {
      mal('El rebote sigue en la lista después de deshacerlo.')
    } else {
      console.log('✓ 5 · un rebote se deshace y desaparece de la lista.')
    }
  }

  // ── 6 · Buscar no engaña sobre su alcance ───────────────────────────
  await pagina.getByLabel('Buscar una dirección o un nombre').fill('ana')
  await pagina.waitForTimeout(400)

  const aviso = pagina.locator('.supresion__filtro-parcial')
  if ((await aviso.count()) === 0) {
    mal(
      'Al buscar no se dice sobre cuántas filas se está buscando.\n' +
        '  Quien busque en una lista de mil creerá que buscó en las mil.',
    )
  } else {
    const texto = (await aviso.innerText()).replace(/\s+/g, ' ')
    if (!/\d+ de las \d+/.test(texto)) {
      mal(`El aviso de la búsqueda no da cifras: «${texto}»`)
    } else {
      console.log(`✓ 6 · la búsqueda dice su alcance: ${texto.trim()}`)
    }
  }

  if (quejas.length) {
    mal(`El navegador se quejó ${quejas.length} veces:`)
    for (const q of quejas.slice(0, 6)) console.error(`    ${q}`)
  } else {
    console.log('✓ Ni un error de consola.')
  }
} finally {
  await navegador?.close()
}

process.exit(codigo)
