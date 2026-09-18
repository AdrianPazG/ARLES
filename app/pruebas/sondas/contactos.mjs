/**
 * La pantalla de contactos hace lo que dice.
 *
 *     node app/pruebas/sondas/contactos.mjs
 *
 * ─────────────────────────────────────────────────────────────────────────
 * QUÉ SE MIDE Y POR QUÉ AQUÍ
 *
 * Cuatro afirmaciones de la entrega 3.2 que **ningún test de unidad puede
 * comprobar**, porque son sobre lo que acaba en la pantalla:
 *
 *   1. Se da de alta un contacto y aparece en la tabla.
 *   2. La ficha enseña **todos** sus canales, agrupados y con el principal
 *      arriba (L-14) — y ese orden lo decide el núcleo, no la pantalla.
 *   3. Una dirección repetida **nombra la dirección** en el error, y el
 *      diálogo **no se cierra**: cerrarlo se llevaría por delante los campos
 *      marcados y el usuario no sabría qué falló.
 *   4. El móvil mexicano escrito con el «1» (+52 1 …) se guarda **sin** él.
 *      Es la trampa que rompe la deduplicación, y es invisible salvo que
 *      alguien mire la ficha después de guardar.
 *
 * Se recorre sobre la vista previa, desde `file://`, que es como la abre
 * Dirección. El núcleo es el simulado, así que esto NO demuestra que el
 * repositorio de Rust funcione —eso lo demuestran sus 49 pruebas—: demuestra
 * que la pantalla pide lo que debe y pinta lo que recibe.
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

/** Rellena el formulario de contacto: identidad y una lista de canales. */
async function rellenar(pagina, { nombre, canales }) {
  const dialogo = pagina.locator('dialog[open]')
  await dialogo.locator('input').first().fill(nombre)

  for (let i = 0; i < canales.length; i += 1) {
    const canal = canales[i]
    if (i > 0) await dialogo.getByRole('button', { name: /Añadir otra/ }).click()
    const filas = dialogo.locator('.canal')
    await filas.nth(i).locator('select').selectOption(canal.tipo)
    await filas.nth(i).locator('input').fill(canal.valor)
  }
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

  await pagina.goto(`file://${ARCHIVO}#/contactos`, { waitUntil: 'load' })
  await pagina.waitForTimeout(1200)

  // Estado vacío. Es lo primero que se ve, y tiene que decir qué es un
  // contacto y que lo de cargar una tabla todavía no está.
  const vacio = await pagina.locator('text=Todavía no hay contactos').count()
  if (vacio !== 1) {
    mal('La pantalla de contactos no arranca en su estado vacío.')
  } else {
    console.log('✓ Arranca en el estado vacío, y explica qué es un contacto.')
  }

  // 1 · Alta con dos canales de cada tipo.
  await pagina.getByRole('button', { name: 'Agregar contacto' }).first().click()
  await pagina.waitForTimeout(400)
  await rellenar(pagina, {
    nombre: 'Ana',
    canales: [
      { tipo: 'email', valor: 'ana@empresa.mx' },
      { tipo: 'email', valor: 'ana.ruiz@empresa.mx' },
      // Con el «1» a propósito: es la trampa del móvil mexicano.
      { tipo: 'whatsapp', valor: '+52 1 81 1234 5678' },
    ],
  })
  await pagina.getByRole('button', { name: 'Dar de alta' }).click()
  await pagina.waitForTimeout(700)

  if ((await pagina.locator('dialog[open]').count()) !== 0) {
    mal('Tras dar de alta un contacto válido, el diálogo sigue abierto.')
  }
  if ((await pagina.locator('text=ana@empresa.mx').count()) === 0) {
    mal('El contacto dado de alta no aparece en la tabla.')
  } else {
    console.log('✓ 1 · se da de alta un contacto y aparece en la tabla.')
  }

  // 2 · La ficha, con todos los canales y en el orden de L-14.
  await pagina.locator('[role="row"]').nth(1).dblclick()
  await pagina.waitForTimeout(400)
  const enFicha = await pagina.locator('.ficha__canal .ficha__valor').allInnerTexts()
  if (enFicha.length !== 3) {
    mal(`La ficha enseña ${enFicha.length} canales de 3: no los enseña todos (L-14).`)
  } else if (!enFicha[0].includes('@') || !enFicha[1].includes('@')) {
    mal(`La ficha no agrupa por tipo: salió ${JSON.stringify(enFicha)}.`)
  } else {
    console.log(`✓ 2 · la ficha enseña los tres canales agrupados: ${enFicha.join(' · ')}`)
  }

  const principales = await pagina.locator('.ficha__canal--principal').count()
  if (principales !== 2) {
    mal(`Hay ${principales} canales marcados como principales; deberían ser 2, uno por tipo.`)
  } else {
    console.log('✓ 2 · un principal por tipo, marcado con la barra de acento.')
  }

  // 4 · El «1» del móvil mexicano no se guarda.
  //
  // Se mide la forma NORMALIZADA, no lo que se escribió. La primera versión de
  // esta comprobación leía el texto completo de la ficha y buscaba «+521»: como
  // ahí se enseña lo escrito —«+52 1 81 1234 5678», con espacios—, el patrón no
  // podía coincidir nunca y la comprobación pasaba siempre. No medía nada.
  const normalizado = (
    await pagina.locator('.ficha__normalizado').allInnerTexts()
  ).join(' ')
  if (normalizado === '') {
    mal(
      'La ficha no enseña la forma normalizada del móvil.\n' +
        '  Quien escribe «+52 1 81…» no puede ver que se guardó sin el «1», y\n' +
        '  esta sonda no tiene dónde comprobarlo.',
    )
  } else if (/\+?521\d{10}/.test(normalizado.replace(/\s/g, ''))) {
    mal(
      `El móvil se guardó con el «1»: ${normalizado.trim()}.\n` +
        '  +52 1 81… y +52 81… son el mismo número. Guardar los dos rompe la\n' +
        '  deduplicación, la supresión y la idempotencia del envío.',
    )
  } else {
    console.log(`✓ 4 · el móvil se normaliza sin el «1», y la ficha lo dice: ${normalizado.trim()}`)
  }

  // 3 · Dirección repetida: dice cuál es y NO cierra el diálogo.
  await pagina.getByRole('button', { name: 'Agregar contacto' }).first().click()
  await pagina.waitForTimeout(400)
  await rellenar(pagina, {
    nombre: 'Otra persona',
    canales: [{ tipo: 'email', valor: 'ana@empresa.mx' }],
  })
  await pagina.getByRole('button', { name: 'Dar de alta' }).click()
  await pagina.waitForTimeout(700)

  if ((await pagina.locator('dialog[open]').count()) !== 1) {
    mal(
      'Con una dirección repetida el diálogo se cerró.\n' +
        '  Cerrarlo se lleva por delante los campos marcados, y el usuario no\n' +
        '  puede saber qué falló ni recuperar lo que había escrito.',
    )
  } else {
    const aviso = await pagina.locator('dialog[open] [role="alert"]').innerText()
    if (!aviso.includes('ana@empresa.mx')) {
      mal(
        'El error de dirección repetida no dice CUÁL es la dirección.\n' +
          `  Dijo: ${aviso.replace(/\s+/g, ' ').slice(0, 160)}`,
      )
    } else {
      console.log('✓ 3 · la dirección repetida se nombra y el diálogo no se cierra.')
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
