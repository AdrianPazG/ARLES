/**
 * Sonda de navegador para la tabla virtualizada.
 *
 * Lo que jsdom NO puede comprobar: allí el contenedor de desplazamiento mide
 * 0 px de alto, así que el virtualizador no renderiza ni una fila y cualquier
 * test de «se encoge la lista y no revienta» pasaría sin ejercitar nada.
 *
 * Aquí hay layout de verdad. Se ataca la tabla como lo haría alguien que
 * quiere romperla: encogerla de 5 000 filas a ninguna de golpe, desplazarse al
 * fondo y encoger desde ahí, y recorrerla entera con el teclado. Cualquier
 * error de consola o excepción de página hace fallar la sonda.
 *
 *     npm --prefix app run sonda:tabla
 *
 * Vive dentro de `app/` porque depende de sus devDependencies: Node resuelve
 * los módulos desde la carpeta del archivo, no desde el directorio de trabajo.
 *
 * Necesita `playwright-core` y un Chromium. El validador la **omite con
 * motivo** si no están, en vez de darla por buena: un omitido no es un fallo,
 * pero tampoco es una validación.
 */
import { chromium } from 'playwright-core'

const URL_BASE = process.argv[2] ?? 'http://localhost:1420'
const EJECUTABLE =
  process.env['ARLES_CHROMIUM'] ?? '/opt/pw-browsers/chromium-1194/chrome-linux/chrome'

const problemas = []
const anotar = (que) => {
  problemas.push(que)
  console.log(`  ✗ ${que}`)
}
const bien = (que) => console.log(`  ✓ ${que}`)

const navegador = await chromium.launch({ executablePath: EJECUTABLE })
const pagina = await navegador.newPage({ viewport: { width: 1440, height: 900 } })

pagina.on('console', (m) => {
  if (m.type() === 'error') anotar(`consola: ${m.text()}`)
})
pagina.on('pageerror', (e) => anotar(`excepción: ${e.message}`))

await pagina.goto(`${URL_BASE}/#/catalogo`, { waitUntil: 'networkidle' })
await pagina.getByRole('tab', { name: 'Tabla virtualizada', exact: true }).click()
await pagina.waitForSelector('[role="grid"]')

const filasVisibles = () => pagina.locator('[role="row"]').count()

// ── 0 · Mont cargó de verdad ──────────────────────────────────────────────
// La pila de reserva existe para que un fallo no rompa la interfaz, y por eso
// mismo lo esconde: sin esta comprobación, «se ve bien» no distingue entre
// Mont y la fuente del sistema.
const fuentes = await pagina.evaluate(async () => {
  await document.fonts.ready
  return {
    caras: [...document.fonts].map((f) => `${f.family} ${f.weight} ${f.status}`),
    disponible: document.fonts.check('400 14px Mont'),
  }
})
const cargadas = fuentes.caras.filter((c) => c.endsWith('loaded')).length
if (!fuentes.disponible || cargadas !== 4) {
  anotar(`Mont no cargó: ${cargadas}/4 caras — ${fuentes.caras.join(', ') || 'ninguna'}`)
} else bien('los cuatro cortes de Mont cargaron')

// ── 1 · La virtualización renderiza una ventana, no la lista entera ───────
const alPrincipio = await filasVisibles()
if (alPrincipio < 2) anotar('no se renderizó ninguna fila: la sonda no probaría nada')
else if (alPrincipio > 120) anotar(`se renderizaron ${alPrincipio} filas: no está virtualizando`)
else bien(`virtualiza: ${alPrincipio} nodos de fila para 5 000 filas`)

// ── 2 · Encoger de golpe desde el principio ───────────────────────────────
const filtro = pagina.getByLabel('Filtrar')
await filtro.fill('contacto de prueba 4242')
await pagina.waitForTimeout(300)
const traFiltrar = await filasVisibles()
if (traFiltrar < 2) anotar('el filtro dejó la rejilla sin filas cuando sí había coincidencia')
else bien(`el filtro estrecho deja ${traFiltrar - 1} fila(s)`)

// ── 3 · Encoger hasta cero ────────────────────────────────────────────────
await filtro.fill('zzz-no-existe-nada')
await pagina.waitForTimeout(300)
if (!(await pagina.getByText('Ningún contacto coincide').isVisible())) {
  anotar('sin coincidencias no se muestra el estado vacío')
} else bien('sin coincidencias aparece el estado vacío')

// ── 4 · Desplazarse al fondo y encoger desde ahí ──────────────────────────
// Es el caso que rompe un virtualizador mal atado: el desplazamiento queda
// muy por debajo del nuevo alto total y los índices apuntan fuera de la lista.
await filtro.fill('')
await pagina.waitForTimeout(300)
const cuerpo = pagina.locator('[role="grid"] .cuerpo')
await cuerpo.evaluate((el) => el.scrollTo(0, el.scrollHeight))
await pagina.waitForTimeout(300)
await filtro.fill('contacto de prueba 7')
await pagina.waitForTimeout(400)
const traEncoger = await filasVisibles()
if (traEncoger < 2) anotar('tras desplazar al fondo y filtrar, la rejilla se quedó vacía')
else bien(`encoger desde el fondo deja ${traEncoger - 1} filas visibles`)

// ── 5 · Recorrido completo por teclado ────────────────────────────────────
await filtro.fill('')
await pagina.waitForTimeout(300)
await pagina.locator('[role="grid"]').focus()
for (const tecla of ['End', 'Home', 'PageDown', 'PageDown', 'ArrowDown', 'ArrowUp', ' ']) {
  await pagina.keyboard.press(tecla)
  await pagina.waitForTimeout(60)
}
const seleccionadas = await pagina.locator('[aria-selected="true"][role="row"]').count()
if (seleccionadas !== 1) anotar(`tras navegar con el teclado hay ${seleccionadas} filas seleccionadas, no 1`)
else bien('el teclado deja exactamente una fila seleccionada')

// `End` lleva a la última: el lector de pantalla tiene que poder decir cuál.
await pagina.keyboard.press('End')
await pagina.waitForTimeout(200)
const indice = await pagina
  .locator('[aria-selected="true"][role="row"]')
  .getAttribute('aria-rowindex')
if (indice !== '5001') anotar(`«Fin» dejó aria-rowindex=${indice}, se esperaba 5001`)
else bien('«Fin» selecciona la última fila y lo declara en aria-rowindex')

// ── 6 · La cuenta que anuncia ARIA es la real, no la renderizada ──────────
const total = await pagina.locator('[role="grid"]').getAttribute('aria-rowcount')
if (total !== '5001') anotar(`aria-rowcount=${total}, se esperaba 5001`)
else bien('aria-rowcount declara el total real, no lo que hay en el DOM')

await navegador.close()

if (problemas.length) {
  console.error(`\n${problemas.length} problema(s) en la tabla.`)
  process.exit(1)
}
console.log('\nLa tabla aguanta las seis sondas.')
