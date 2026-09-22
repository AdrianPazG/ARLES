/**
 * El derecho de Acceso hace lo que dice · entrega 3.4
 *
 *     npm run sonda:derechos
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ ESTO SE MIDE EN UN NAVEGADOR DE VERDAD
 *
 * Las pruebas de `derechos.spec.ts` montan la pantalla con el núcleo fingido:
 * comprueban que cada desenlace se **cuenta** distinto. Lo que no pueden ver
 * es si la pantalla **se encuentra** y si el recorrido entero funciona —
 * llegar por la barra, escribir, pulsar, leer la respuesta.
 *
 * Es exactamente la clase de fallo que se nos escapó dos veces esta semana:
 * la configuración llevaba a Ajustes y la barra se quedaba apagada, y ninguna
 * prueba de componente podía verlo porque todas usan dobles.
 * ─────────────────────────────────────────────────────────────────────────
 */
import { fileURLToPath } from 'node:url'
import path from 'node:path'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { python } from './ordenes.mjs'

const AQUI = path.dirname(fileURLToPath(import.meta.url))
const RAIZ = path.resolve(AQUI, '../../..')
const ARCHIVO = path.join(RAIZ, 'vista-previa/ARLES-vista-previa.html')

let fallos = 0
function mal(mensaje) {
  console.error(`✗ ${mensaje}`)
  fallos += 1
}

const ejecutable = await exigirChromium()

console.log('Generando la vista previa…')
python([path.join(RAIZ, 'herramientas/vista-previa/generar.py')])

const navegador = await chromium.launch({ executablePath: ejecutable })
const pagina = await navegador.newPage({ viewport: { width: 1280, height: 900 } })

const erroresDeConsola = []
pagina.on('console', (m) => {
  if (m.type() === 'error') erroresDeConsola.push(m.text())
})

try {
  // ── 1 · Se encuentra desde CONTACTOS, por la barra ────────────────────
  //
  // No se navega por URL a propósito: que la pantalla EXISTA no dice nada si
  // no hay forma de llegar a ella. Es la misma lección de las dos
  // observaciones de Dirección.
  await pagina.goto(`file://${ARCHIVO}#/contactos`, { waitUntil: 'load' })
  await pagina.waitForTimeout(1200)

  const enlace = pagina.locator('nav a.sub-enlace[href$="#/contactos/derechos"]')
  if ((await enlace.count()) === 0) {
    mal('La barra no enseña «Derechos» bajo CONTACTOS: la pantalla es inalcanzable.')
    await pagina.goto(`file://${ARCHIVO}#/contactos/derechos`, { waitUntil: 'load' })
  } else {
    await enlace.first().click()
    console.log('✓ 1 · se llega desde CONTACTOS, por la barra lateral.')
  }
  await pagina.waitForTimeout(900)

  // ── 2 · La sigla ARCO no se le enseña a nadie ─────────────────────────
  const texto = await pagina.locator('main').innerText()
  if (texto.includes('ARCO')) {
    mal('La pantalla enseña la sigla «ARCO», que no se la sabe nadie fuera de protección de datos.')
  } else {
    console.log('✓ 2 · no se enseña la sigla ARCO.')
  }

  // ── 3 · Se dice de quién es la obligación ─────────────────────────────
  //
  // Quien manda el archivo lo firma en nombre de su empresa, no de ARLES
  // (PRIVACIDAD_LFPDPPP.md §2: el responsable es el cliente). Callarlo dejaría
  // a alguien creyendo que ARLES responde por él.
  if (!texto.includes('Respondes tú')) {
    mal('No se dice que la obligación de responder es del cliente, no de ARLES.')
  } else {
    console.log('✓ 3 · se dice quién responde.')
  }

  /** Escribe una dirección y pide el informe. */
  async function pedir(direccion) {
    await pagina.fill('form input', '')
    await pagina.fill('form input', direccion)
    await pagina.click('form button[type="submit"]')
    await pagina.waitForTimeout(700)
    return (await pagina.locator('main').innerText()).replace(/\s+/g, ' ')
  }

  // ── 4 · Con datos ─────────────────────────────────────────────────────
  const conDatos = await pedir('ana@empresa.mx')
  if (!conDatos.includes('datos-de-ana')) {
    mal(`No se dice qué archivo se guardó: ${conDatos.slice(0, 160)}`)
  } else if (!conDatos.includes('todo lo que ARLES tiene')) {
    mal('No se cuenta qué lleva el archivo.')
  } else {
    console.log('✓ 4 · con datos, dice el archivo y qué lleva.')
  }

  // Y NO se enseña una ruta del sistema de archivos. Regla de frontera 4:
  // ninguna ruta cruza a la webview, y la carpeta donde alguien guarda sus
  // cosas dice dónde vive (THREAT_MODEL.md §4.1).
  if (/[/\\](home|Users|C:)/.test(conDatos)) {
    mal(`Se está enseñando una ruta del sistema: ${conDatos.slice(0, 160)}`)
  } else {
    console.log('✓ 4b · no se enseña ninguna ruta, sólo el nombre del archivo.')
  }

  // ── 5 · EL CASO QUE MÁS IMPORTA · sin datos ───────────────────────────
  //
  // «No consta nada» no es un error: puede ser justo lo que la persona quería
  // oír. Pero tiene que decirse, porque quien atiende está a punto de mandar
  // un archivo vacío y necesita saber que va vacío A PROPÓSITO.
  const sinDatos = await pedir('nadie@empresa.mx')
  if (!sinDatos.includes('va vacío')) {
    mal(`Un informe sin datos no avisa de que va vacío: ${sinDatos.slice(0, 160)}`)
  } else {
    console.log('✓ 5 · sin datos, lo dice en vez de entregar un archivo vacío en silencio.')
  }

  // Y no se pinta como una avería.
  const pintadoComoError = await pagina.evaluate(() => {
    const avisos = [...document.querySelectorAll('main [class*="aviso"], main [role="alert"]')]
    return avisos.some((a) => /peligro|danger|error/.test(a.className))
  })
  if (pintadoComoError) {
    mal('Un informe vacío se pinta como un fallo. Es una respuesta legítima, no una avería.')
  } else {
    console.log('✓ 5b · no se pinta como un fallo.')
  }

  // ── 6 · Ya no es contacto, pero queda rastro ──────────────────────────
  //
  // `beto@otra.mx` está en la semilla de bajas del núcleo simulado: no es
  // contacto, y sí tiene rastro. Si la pantalla dijera «todo lo que tenemos»
  // a secas, quien atiende no entendería por qué aparece gente que creía
  // borrada.
  const soloRastro = await pedir('beto@otra.mx')
  if (!soloRastro.includes('ya no está dada de alta como contacto')) {
    mal(`No se distingue a quien ya no es contacto: ${soloRastro.slice(0, 160)}`)
  } else {
    console.log('✓ 6 · distingue a quien ya no es contacto pero deja rastro.')
  }

  // ── 7 · Cancelar no es un error ───────────────────────────────────────
  const cancelado = await pedir('cancela@empresa.mx')
  if (!cancelado.includes('sin guardar')) {
    mal(`Cancelar el guardado no se explica: ${cancelado.slice(0, 160)}`)
  } else {
    console.log('✓ 7 · cancelar se cuenta como una decisión, no como un fallo.')
  }

  // ── 8 · Se dice lo que todavía no existe ──────────────────────────────
  if (!cancelado.includes('Todavía no está construido')) {
    mal('No se avisa de que el borrado de datos todavía no existe.')
  } else {
    console.log('✓ 8 · avisa de que el borrado todavía no está construido.')
  }

  if (erroresDeConsola.length > 0) {
    mal(`Errores de consola:\n    ${erroresDeConsola.join('\n    ')}`)
  } else {
    console.log('✓ Ni un error de consola.')
  }
} finally {
  await navegador.close()
}

if (fallos > 0) {
  console.error(`\n${fallos} comprobación(es) fallaron.`)
  process.exit(1)
}
