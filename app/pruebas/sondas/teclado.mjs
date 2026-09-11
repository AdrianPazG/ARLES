/**
 * Sonda de teclado y foco.
 *
 * Lo que jsdom **no** puede decidir: allí no hay orden de tabulación real, el
 * `inert` del fondo de un `<dialog>` no existe y `Tab` no mueve nada. Un test
 * que afirmara «el foco queda atrapado en el modal» pasaría sin comprobarlo.
 *
 * Aquí se usa el teclado de verdad contra la aplicación construida:
 *
 * - El modal atrapa el foco: `Tab` da la vuelta dentro y no llega al fondo.
 * - Al cerrarlo, el foco vuelve al botón que lo abrió.
 * - Un modal destructivo no se cierra con `Esc` ni pulsando fuera.
 * - Toda la interfaz es alcanzable con `Tab`, y **cada parada dibuja su
 *   anillo de foco**: es la regla del §100, y la única forma de comprobarla es
 *   mirar el estilo calculado en cada salto.
 * - El menú devuelve el foco al disparador al cerrarse con `Esc`.
 *
 *     npm --prefix app run sonda:teclado
 *
 * Necesita el servidor de desarrollo en marcha.
 */
import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'

const URL_BASE = process.argv[2] ?? 'http://localhost:1420'
// La ruta se localiza, no se escribe: ver navegador.mjs.
const EJECUTABLE = exigirChromium()

const problemas = []
const anotar = (q) => {
  problemas.push(q)
  console.log(`  ✗ ${q}`)
}
const bien = (q) => console.log(`  ✓ ${q}`)

const navegador = await chromium.launch({ executablePath: EJECUTABLE })
const pagina = await navegador.newPage({ viewport: { width: 1440, height: 900 } })
pagina.on('pageerror', (e) => anotar(`excepción: ${e.message}`))

await pagina.goto(`${URL_BASE}/#/catalogo`, { waitUntil: 'networkidle' })
await pagina.waitForTimeout(400)

const activo = () =>
  pagina.evaluate(() => {
    const el = document.activeElement
    if (!el) return null
    return {
      etiqueta: el.tagName,
      texto: (el.textContent ?? '').trim().slice(0, 28),
      esCuerpo: el === document.body,
      dentroDelModal: !!el.closest('dialog'),
      // `outline-style: none` sin sustituto es lo que rompe la navegación
      // por teclado. Se acepta cualquier señal visible: contorno, sombra o
      // borde marcado.
      anillo:
        getComputedStyle(el).outlineStyle !== 'none' ||
        getComputedStyle(el).boxShadow !== 'none',
    }
  })

// ── 1 · El modal atrapa el foco ───────────────────────────────────────────
await pagina.getByRole('button', { name: 'Modal normal' }).click()
await pagina.waitForTimeout(300)

// Ojo con lo que se afirma aquí. Al pasar del último elemento del diálogo,
// el navegador devuelve el foco al documento antes de volver a entrar, y
// `document.activeElement` cae en `<body>`. **Eso no es una fuga**: ningún
// control del fondo lo recibió, y en una ventana de Tauri no hay barra de
// direcciones a la que ir. Una sonda que contara esos relevos como fallo
// estaría midiendo el navegador, no la aplicación.
//
// Lo que sí es la garantía: ningún elemento DE FUERA del diálogo llega a
// tener el foco.
const fugas = []
for (let i = 0; i < 20; i += 1) {
  await pagina.keyboard.press('Tab')
  const a = await activo()
  if (a && !a.dentroDelModal && !a.esCuerpo) fugas.push(`${a.etiqueta} «${a.texto}»`)
}
if (fugas.length > 0) anotar(`el foco alcanzó el fondo: ${fugas.slice(0, 3).join(', ')}`)
else bien('el modal atrapa el foco: 20 tabulaciones sin alcanzar un control del fondo')

// ── 2 · Al cerrar, el foco vuelve al disparador ───────────────────────────
await pagina.keyboard.press('Escape')
await pagina.waitForTimeout(300)
const traCerrar = await activo()
if (!traCerrar || !traCerrar.texto.includes('Modal normal')) {
  anotar(`tras cerrar el modal el foco quedó en «${traCerrar?.texto}», no en el botón que lo abrió`)
} else bien('al cerrar el modal el foco vuelve al botón que lo abrió')

// ── 3 · El destructivo no cede ────────────────────────────────────────────
await pagina.getByRole('button', { name: 'Modal destructivo' }).click()
await pagina.waitForTimeout(300)
await pagina.keyboard.press('Escape')
await pagina.waitForTimeout(300)
if (!(await pagina.locator('dialog[open]').isVisible())) {
  anotar('«Esc» cerró un modal destructivo')
} else bien('«Esc» no cierra un modal destructivo')

// Pulsar en el fondo oscurecido tampoco.
await pagina.mouse.click(30, 30)
await pagina.waitForTimeout(300)
if (!(await pagina.locator('dialog[open]').isVisible())) {
  anotar('pulsar fuera cerró un modal destructivo')
} else bien('pulsar fuera no cierra un modal destructivo')

await pagina.getByRole('button', { name: 'Seguir enviando' }).click()
await pagina.waitForTimeout(300)

// ── 4 · Toda parada de tabulación dibuja su anillo ────────────────────────
await pagina.evaluate(() => document.body.focus())
await pagina.keyboard.press('Tab')
const sinAnillo = []
const vistos = new Set()
for (let i = 0; i < 40; i += 1) {
  const a = await activo()
  if (!a) break
  // `<body>` aparece como relevo entre vueltas del navegador; no es una
  // parada de tabulación de la aplicación y no tiene que dibujar nada.
  if (a.esCuerpo) { await pagina.keyboard.press('Tab'); continue }
  const clave = `${a.etiqueta}:${a.texto}`
  if (vistos.has(clave)) break // dio la vuelta entera
  vistos.add(clave)
  if (!a.anillo) sinAnillo.push(clave)
  await pagina.keyboard.press('Tab')
}
if (vistos.size < 10) anotar(`sólo ${vistos.size} paradas de tabulación: la interfaz no es navegable`)
else if (sinAnillo.length) anotar(`sin anillo de foco: ${sinAnillo.slice(0, 5).join(', ')}`)
else bien(`las ${vistos.size} paradas de tabulación dibujan su anillo de foco`)

// ── 5 · El menú devuelve el foco ──────────────────────────────────────────
const disparador = pagina.getByRole('button', { name: /Acciones/ })
await disparador.click()
await pagina.waitForTimeout(300)
if (!(await pagina.locator('[role="menu"]').isVisible())) anotar('el menú no se abrió')
await pagina.keyboard.press('Escape')
await pagina.waitForTimeout(300)
const traMenu = await activo()
if (!traMenu || !traMenu.texto.includes('Acciones')) {
  anotar(`tras cerrar el menú el foco quedó en «${traMenu?.texto}», no en el disparador`)
} else bien('al cerrar el menú con «Esc» el foco vuelve al disparador')

// ── 6 · Las pestañas usan tabulación móvil ────────────────────────────────
const enTabulacion = await pagina
  .locator('[role="tab"][tabindex="0"]')
  .count()
if (enTabulacion !== 1) {
  anotar(`${enTabulacion} pestañas en el orden de tabulación; debe haber exactamente 1`)
} else bien('sólo la pestaña activa está en el orden de tabulación')

await navegador.close()

if (problemas.length) {
  console.error(`\n${problemas.length} problema(s) de teclado o foco.`)
  process.exit(1)
}
console.log('\nEl teclado llega a todo y el foco no se pierde.')
