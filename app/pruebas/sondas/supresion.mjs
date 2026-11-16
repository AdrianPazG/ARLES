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

  // SE LLEGA POR LA BARRA LATERAL, no por un enlace dentro de la pantalla.
  //
  // Hasta el 22/09/2026 esta sonda pulsaba un enlace «Quién pidió no recibir»
  // que vivía arriba a la derecha de CONTACTOS. Dirección lo mandó quitar:
  // desde O-3 la barra ya enseña «Bajas» colgando de Contactos, y dos puertas
  // al mismo cuarto acaban divergiendo el día que alguien renombra una.
  //
  // Lo que se comprueba sigue siendo lo mismo —que la lista SE ENCUENTRA— y
  // ahora se comprueba por el único camino que queda. Si ese camino se
  // rompiera, la pantalla sería inalcanzable y esta sonda lo diría.
  const enlace = pagina.locator('nav a.sub-enlace[href$="#/contactos/supresion"]')
  if ((await enlace.count()) === 0) {
    mal('La barra no enseña «Bajas» bajo CONTACTOS: la lista es inalcanzable.')
    await pagina.goto(`file://${ARCHIVO}#/contactos/supresion`, { waitUntil: 'load' })
  } else {
    if ((await enlace.first().innerText()).trim() !== 'Bajas') {
      mal(`La entrada de la barra no se llama «Bajas»: ${await enlace.first().innerText()}`)
    }
    await enlace.first().click()
    console.log('✓ 1 · se llega desde CONTACTOS, por la barra lateral.')
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
      // DESDE EL 22/09/2026 LA EXPLICACIÓN ES UN CANDADO, no la frase entera.
      //
      // Dirección la vio repetida en tres de cinco filas, en dos renglones
      // cada vez. Ahora la fila lleva el icono, su texto para lectores de
      // pantalla y el `title`; la frase completa va una sola vez encima.
      //
      // Se exige el icono Y su texto accesible. Sólo el icono sería
      // información codificada en la forma, que es el mismo error que
      // codificarla en el color (§19) — y esta sonda tiene que poder
      // distinguir «hay candado» de «hay candado y se entiende».
      const candado = fila.locator('.fila__candado')
      const tieneIcono = (await candado.locator('svg').count()) > 0
      const tieneTexto = texto.includes('Lo pidió ella misma')
      if (tieneIcono && tieneTexto) candadosBien += 1
      else if (!tieneIcono) mal(`Una fila sin botón no lleva candado: ${texto.slice(0, 80)}`)
      else mal(`El candado no dice nada a un lector de pantalla: ${texto.slice(0, 80)}`)
    }
  }

  // Y la frase completa aparece UNA VEZ, arriba, explicando el símbolo.
  const leyenda = pagina.locator('.supresion__leyenda')
  if ((await leyenda.count()) !== 1) {
    mal(
      `La leyenda del candado aparece ${await leyenda.count()} veces. ` +
        'Sin ella, el icono es un símbolo que nadie ha presentado.',
    )
  } else if (!(await leyenda.innerText()).includes('candado')) {
    mal(`La leyenda no nombra el símbolo: ${await leyenda.innerText()}`)
  } else {
    console.log('✓ 4b · el candado se explica una vez, encima de la tabla.')
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

  // ── 7 · Se puede apuntar una baja a mano ────────────────────────────
  //
  // Hasta el 23/10/2026 no se podía: la única forma era la ficha de un
  // contacto, y `suprimir_contacto` exige un `ContactId`. Así que una
  // dirección a la que le reenviaron el correo **no se podía dar de baja de
  // ninguna forma**.
  //
  // Se mide aquí, en la página de verdad, porque es un cable de cuatro
  // tramos —formulario, store, comando, lista— y la prueba de componente sólo
  // alcanza los dos primeros.
  await pagina.getByLabel('Buscar una dirección o un nombre').fill('')
  await pagina.waitForTimeout(200)

  const campo = pagina.getByLabel('Dirección o número')
  if ((await campo.count()) === 0) {
    mal(
      'No hay dónde apuntar una baja a mano.\n' +
        '  Una dirección que no sea un contacto no se puede dar de baja.',
    )
  } else {
    await campo.fill('alguien@reenviado.mx')
    await pagina.getByRole('button', { name: 'Apuntar la baja' }).click()
    await pagina.waitForTimeout(500)

    const texto = (await pagina.locator('.supresion').innerText()).replace(/\s+/g, ' ')
    if (!texto.includes('Apuntada')) {
      mal('Apuntar una baja no acusa recibo: no se sabe si se guardó.')
    } else if (!texto.includes('alguien@reenviado.mx')) {
      // La lista se recarga al apuntar. Si la baja nueva no sale, el acuse
      // está prometiendo algo que no se puede comprobar en la propia pantalla.
      mal('La baja recién apuntada no aparece en la lista.')
    } else {
      console.log('✓ 7 · se apunta una baja de una dirección que no es contacto.')
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
