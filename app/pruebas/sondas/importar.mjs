/**
 * El asistente de importación hace lo que dice.
 *
 *     node app/pruebas/sondas/importar.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE MIDE
 *
 * Cinco afirmaciones de la entrega 3.3 que ningún test de unidad alcanza,
 * porque son sobre el recorrido completo en pantalla:
 *
 *   1. El mapeo de columnas **se propone solo** y se puede corregir.
 *   2. **Analizar no escribe nada**: se puede volver atrás y repetir.
 *   3. El informe separa lo que entra de lo que no, **con cifras**.
 *   4. **No se puede confirmar sin aceptar la declaración de origen**, y el
 *      texto provisional se ve marcado como tal (ADR-0013, P-09).
 *   5. Lo importado aparece en CONTACTOS, y el móvil con el «1» mexicano
 *      llega sin él.
 *
 * El núcleo es el simulado, así que esto NO demuestra que la importación de
 * Rust funcione —eso lo demuestran sus 305 pruebas—: demuestra que la pantalla
 * pide lo que debe, en el orden que debe, y pinta lo que recibe.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { python } from './ordenes.mjs'

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
  const gen = python(['herramientas/vista-previa/generar.py'], {
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

  await pagina.goto(`file://${ARCHIVO}#/contactos/importar`, { waitUntil: 'load' })
  await pagina.waitForTimeout(1200)

  // ── 1 · Elegir el archivo y ver el mapeo propuesto ──────────────────
  await pagina.getByRole('button', { name: 'Elegir archivo' }).click()
  await pagina.waitForTimeout(600)

  const selectores = pagina.locator('.mapa__columna select')
  const columnas = await selectores.count()
  if (columnas !== 5) {
    mal(`Se leyeron ${columnas} columnas de 5: el archivo no se cargó bien.`)
  } else {
    const propuesto = await selectores.evaluateAll((s) =>
      s.map((x) => /** @type {HTMLSelectElement} */ (x).value),
    )
    const esperado = ['nombre', 'apellido', 'empresa', 'correo', 'whatsapp']
    if (propuesto.join() !== esperado.join()) {
      mal(
        `El mapeo automático no reconoció los encabezados en español.\n` +
          `  salió:    ${propuesto.join(' · ')}\n` +
          `  esperado: ${esperado.join(' · ')}`,
      )
    } else {
      console.log(`✓ 1 · el mapeo se propone solo: ${propuesto.join(' · ')}`)
    }
  }

  // Y se puede corregir: se cambia una columna y se vuelve a dejar.
  await selectores.nth(2).selectOption('ignorar')
  await pagina.waitForTimeout(200)
  if ((await selectores.nth(2).inputValue()) !== 'ignorar') {
    mal('El mapeo propuesto no se puede corregir.')
  } else {
    await selectores.nth(2).selectOption('empresa')
    console.log('✓ 1 · y se puede corregir a mano.')
  }

  // ── 2 y 3 · Analizar, sin escribir nada ─────────────────────────────
  await pagina.getByRole('button', { name: 'Ver qué va a pasar' }).click()
  await pagina.waitForTimeout(700)

  const cifras = await pagina.locator('.importar__cifras').innerText()
  // Del archivo de muestra: entran Ana y Luis; se quedan fuera la fila sin
  // contacto, la del correo mal escrito y la repetida.
  if (!cifras.includes('2') || !cifras.includes('3') || !cifras.includes('5')) {
    mal(`Las cifras del informe no cuadran con el archivo de muestra: «${cifras.trim()}»`)
  } else {
    console.log(`✓ 3 · el informe da cifras, no adjetivos: ${cifras.trim()}`)
  }

  // El choque nombra a quién pertenece o en qué fila estaba.
  const choques = await pagina.locator('.filas__motivo').allInnerTexts()
  if (!choques.some((c) => c.includes('fila'))) {
    mal(
      'El informe no dice dónde estaba la dirección repetida.\n' +
        `  Dijo: ${choques.slice(0, 3).join(' | ')}`,
    )
  } else {
    console.log('✓ 3 · la repetida dice en qué fila estaba.')
  }

  // Los números de fila son los de Excel: la primera de datos es la 2.
  const filas = (await pagina.locator('.filas__fila').allInnerTexts()).join(' ')
  if (!/Fila\s*[2-9]/.test(filas)) {
    mal(`Los números de fila no son los de Excel: ${filas.slice(0, 120)}`)
  } else {
    console.log('✓ 3 · las filas se numeran como en Excel.')
  }

  // ── 4 · No se puede confirmar sin aceptar ───────────────────────────
  const confirmar = pagina.getByRole('button', { name: /^Importar \d+ contactos$/ })
  if (await confirmar.isEnabled()) {
    mal(
      'Se puede importar SIN aceptar la declaración de origen.\n' +
        '  Esa declaración es la prueba de que la lista es lícita (ADR-0013).',
    )
  } else {
    console.log('✓ 4 · no se puede importar sin aceptar la declaración.')
  }

  // Y el texto provisional se ve marcado como tal: P-09 sigue abierta, y un
  // texto legal sin revisar que no lo diga aparenta un rigor que no tiene.
  const declaracion = await pagina.locator('.declaracion').innerText()
  if (!/PENDIENTE DE REVISIÓN JUR/i.test(declaracion)) {
    mal(
      'El texto de la declaración no avisa de que está pendiente de revisión ' +
        'jurídica.\n' +
        `  Dice: ${declaracion.replace(/\s+/g, ' ').slice(0, 140)}`,
    )
  } else {
    console.log('✓ 4 · el texto provisional se ve marcado como provisional.')
  }

  // ── 2 · Volver atrás no rompe nada ──────────────────────────────────
  await pagina.getByRole('button', { name: 'Volver a las columnas' }).click()
  await pagina.waitForTimeout(400)
  if ((await pagina.locator('.mapa__columna select').count()) !== 5) {
    mal('Volver a las columnas perdió el archivo: hay que empezar de cero.')
  } else {
    console.log('✓ 2 · se puede volver atrás y el archivo sigue cargado.')
  }
  await pagina.getByRole('button', { name: 'Ver qué va a pasar' }).click()
  await pagina.waitForTimeout(600)

  // ── 5 · Aceptar, importar, y comprobarlo en Contactos ───────────────
  await pagina.locator('.declaracion input[type="checkbox"]').check()
  await pagina.waitForTimeout(200)
  await pagina.getByRole('button', { name: /^Importar \d+ contactos$/ }).click()
  await pagina.waitForTimeout(800)

  const hecho = await pagina.locator('.importar__cifras').innerText()
  if (!hecho.includes('2')) {
    mal(`El resumen final no dice cuántos entraron: «${hecho.trim()}»`)
  } else {
    console.log(`✓ 5 · el resumen final da la cifra: ${hecho.trim()}`)
  }

  await pagina.getByRole('button', { name: 'Ver mis contactos' }).click()
  await pagina.waitForTimeout(800)

  const enLaTabla = await pagina.locator('[role="row"]').count()
  // Una fila de encabezado más dos contactos.
  if (enLaTabla < 3) {
    mal(`Tras importar, la tabla de contactos tiene ${enLaTabla} filas: no llegaron.`)
  } else {
    console.log('✓ 5 · los contactos importados aparecen en la tabla.')
  }

  // El «1» del móvil mexicano: se mira la forma normalizada de la ficha, no
  // lo escrito. Es la misma trampa que ya se coló una vez en `sonda:contactos`.
  await pagina.locator('[role="row"]').nth(1).dblclick()
  await pagina.waitForTimeout(400)
  const normalizado = (
    await pagina.locator('.ficha__normalizado').allInnerTexts()
  ).join(' ')
  if (normalizado === '') {
    mal('Ningún contacto importado enseña su forma normalizada: no se puede comprobar el móvil.')
  } else if (/\+?521\d{10}/.test(normalizado.replace(/\s/g, ''))) {
    mal(`El móvil se importó con el «1»: ${normalizado.trim()}`)
  } else {
    console.log(`✓ 5 · el móvil importado llega sin el «1»: ${normalizado.trim()}`)
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
