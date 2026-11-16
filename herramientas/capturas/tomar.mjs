/**
 * Capturas anotadas de las pantallas de ARLES, para las guías.
 *
 * ═══════════════════════════════════════════════════════════════════════════
 * POR QUÉ ESTO EXISTE
 *
 * Dirección lo pidió el 12/11/2026: «me gustaría que el manual sea más
 * gráfico, agregar imágenes de las pantallas donde indiquen a dónde te debes
 * dirigir y lo que debes capturar y colocar».
 *
 * Una guía de primer uso con capturas no es un adorno: alguien que no conoce
 * la pantalla busca un botón por su forma y su sitio, no por su nombre.
 *
 * ═══════════════════════════════════════════════════════════════════════════
 * LA DECISIÓN QUE HACE QUE ESTO NO MIENTA
 *
 * **Las anotaciones se dibujan sobre el elemento de verdad**, buscándolo en la
 * página por su texto, y **si no lo encuentra, esto FALLA** en vez de pintar
 * un recuadro donde se supone que está.
 *
 * Es la diferencia entre una captura anotada y una ilustración: si alguien
 * renombra un botón, la captura deja de generarse y hay que mirarlo. Una
 * flecha dibujada a mano sobre una imagen fija se queda apuntando al sitio
 * equivocado y **nadie se entera** — y este proyecto ya tiene dos guías que
 * mandaban pulsar botones que no existen (H-26, 12/11/2026).
 *
 * ═══════════════════════════════════════════════════════════════════════════
 * LO QUE ESTAS CAPTURAS SON, Y LO QUE NO
 *
 * Salen de la **vista previa**, que es la interfaz de ARLES compilada y
 * corriendo en un navegador con datos de ejemplo. Son las pantallas de verdad
 * con sus estilos de verdad.
 *
 * Lo que NO son es la aplicación empaquetada: ahí la ventana la pinta WebView2
 * en Windows y WKWebView en macOS. Las diferencias conocidas son el marco de
 * la ventana y la tipografía cuando se compila sin Mont. Está dicho en el pie
 * de las imágenes en vez de darse por equivalente.
 *
 *     node herramientas/capturas/tomar.mjs
 */

import { execFileSync } from 'node:child_process'
import { createRequire } from 'node:module'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const AQUI = path.dirname(fileURLToPath(import.meta.url))
const RAIZ = path.resolve(AQUI, '../..')

// `playwright-core` vive en `app/node_modules`, que es donde el proyecto
// declara sus dependencias de frontend. Esta herramienta no está en `app/`
// —es de documentación, no de la aplicación—, así que se resuelve desde allí
// en vez de duplicar la dependencia en otro `package.json`.
const requerir = createRequire(path.join(RAIZ, 'app/package.json'))
const { chromium } = requerir('playwright-core')
const ARCHIVO = path.join(RAIZ, 'vista-previa/ARLES-vista-previa.html')
const DESTINO = path.join(RAIZ, 'documentacion/07-entrega/capturas')

/** Dónde está el Chromium que este contenedor trae preinstalado. */
function exigirChromium() {
  const candidatos = [
    process.env.CHROMIUM_PATH,
    '/opt/pw-browsers/chromium',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
  ].filter(Boolean)
  for (const c of candidatos) if (fs.existsSync(c)) return c
  console.error('\n✗ No hay Chromium. Sin navegador no se pueden tomar capturas.\n')
  process.exit(1)
}

/**
 * Las capturas que hacen falta, en el orden de la guía.
 *
 * `marcas` son los textos que hay que señalar **tal como se leen en la
 * pantalla**. Si uno no aparece, el script falla nombrándolo: es lo que
 * convierte a estas imágenes en algo que se puede verificar.
 */
const CAPTURAS = [
  {
    nombre: 'remitentes-anadir',
    ruta: '#/remitentes',
    pie: 'Remitentes · el botón con el que empieza el alta de un correo',
    marcas: [{ boton: 'Añadir remitente', etiqueta: '1' }],
  },
  {
    nombre: 'remitentes-formulario',
    ruta: '#/remitentes',
    pie: 'El formulario del alta: dos campos a la vista, y el resto se rellena solo',
    abrir: 'Añadir remitente',
    marcas: [
      { campo: 'Dirección desde la que se envía', etiqueta: '1' },
      { campo: 'Contraseña', etiqueta: '2' },
    ],
  },
  {
    nombre: 'whatsapp-anadir',
    ruta: '#/remitentes/whatsapp',
    pie: 'Remitentes › WhatsApp · aquí se conecta el número',
    marcas: [{ boton: 'Añadir número', etiqueta: '1' }],
  },
  {
    nombre: 'whatsapp-formulario',
    ruta: '#/remitentes/whatsapp',
    pie: 'El alta del número: se pega el token y ARLES le pregunta a Meta cuáles son tus números',
    abrir: 'Añadir número',
    marcas: [
      { campo: 'Token de acceso', etiqueta: '1' },
      { boton: 'Buscar mis números', etiqueta: '2' },
    ],
  },
]

/**
 * Dibuja los recuadros y sus números, con los rectángulos **ya medidos por
 * Playwright** sobre el elemento que su rol identifica.
 *
 * La primera versión recorría el DOM por su cuenta buscando el elemento más
 * pequeño que contuviera el texto. Mirar el resultado encontró tres defectos
 * que ninguna comprobación habría visto:
 *
 *   · marcaba la **etiqueta** del campo y no el campo,
 *   · el número **tapaba la primera letra** del rótulo («1Dirección»),
 *   · y dejaba un recuadro sobrante sobre un hueco vacío.
 *
 * Por eso ahora el elemento lo localiza Playwright por su rol —botón o campo
 * por su etiqueta—, que es la misma forma en que lo encontraría una persona
 * usando un lector de pantalla.
 */
const DIBUJAR = `(marcas) => {
  const ESTILO = document.createElement('style')
  ESTILO.textContent = \`
    .arles-marca {
      position: absolute; z-index: 99999; pointer-events: none;
      border: 3px solid #E8B64C; border-radius: 6px;
      box-shadow: 0 0 0 4px rgba(232,182,76,.25);
    }
    .arles-globo {
      position: absolute; z-index: 100000; pointer-events: none;
      background: #E8B64C; color: #10243E;
      font: 700 15px/28px system-ui, sans-serif;
      width: 28px; height: 28px; border-radius: 50%; text-align: center;
      box-shadow: 0 2px 6px rgba(0,0,0,.35);
    }\`
  document.head.appendChild(ESTILO)

  for (const { x, y, width, height, etiqueta } of marcas) {
    const caja = document.createElement('div')
    caja.className = 'arles-marca'
    Object.assign(caja.style, {
      left: (x + scrollX - 5) + 'px', top: (y + scrollY - 5) + 'px',
      width: (width + 10) + 'px', height: (height + 10) + 'px',
    })
    document.body.appendChild(caja)

    // A la IZQUIERDA del recuadro y centrado: dentro tapaba la primera letra.
    const globo = document.createElement('div')
    globo.className = 'arles-globo'
    globo.textContent = etiqueta
    Object.assign(globo.style, {
      left: (x + scrollX - 42) + 'px',
      top: (y + scrollY + height / 2 - 14) + 'px',
    })
    document.body.appendChild(globo)
  }
}`

// ── Se regenera la vista previa primero ────────────────────────────────────
//
// Si se compila mal, `generar.py` aborta PERO DEJA EL HTML ANTERIOR en disco
// (trampa 2 de las herramientas, CLAUDE.md). Una captura de otra versión del
// código es peor que ninguna captura, así que se comprueba el código de salida.
console.log('Regenerando la vista previa…')
try {
  execFileSync('python3', [path.join(RAIZ, 'herramientas/vista-previa/generar.py')], {
    cwd: RAIZ, stdio: 'pipe',
  })
} catch (e) {
  console.error('\n✗ La vista previa no se pudo generar, así que no hay nada que capturar.\n')
  console.error(String(e.stderr ?? e).slice(-2000))
  process.exit(1)
}

fs.mkdirSync(DESTINO, { recursive: true })

const navegador = await chromium.launch({ executablePath: exigirChromium() })

const problemas = []
for (const c of CAPTURAS) {
  // ── UNA PÁGINA NUEVA POR CAPTURA, Y NO ES UNA PRECAUCIÓN ──
  //
  // La vista previa es una SPA con rutas en el hash, así que un `goto` a la
  // MISMA ruta con distinto hash **no recarga nada**: el DOM sigue siendo el
  // de antes. Dos capturas de `#/remitentes` seguidas salían con los
  // recuadros de la primera pegados encima de la segunda — un rectángulo
  // dorado sobre un hueco vacío, señalando un botón que ya no estaba ahí.
  //
  // Lo encontró mirar la imagen, no una comprobación: una captura con una
  // anotación de más se genera igual de bien y pesa lo mismo.
  //
  // deviceScaleFactor 2 porque estas imágenes las mira una persona en un PDF.
  const pagina = await navegador.newPage({
    viewport: { width: 1180, height: 820 },
    deviceScaleFactor: 2,
  })
  await pagina.goto(`file://${ARCHIVO}${c.ruta}`, { waitUntil: 'load' })
  // La misma espera fija que usan las sondas, y por el mismo motivo: 700 ms
  // medían pantallas a medio pintar.
  await pagina.waitForTimeout(1500)

  if (c.abrir) {
    const boton = pagina.getByRole('button', { name: c.abrir }).first()
    if ((await boton.count()) === 0) {
      problemas.push(`${c.nombre}: no existe el botón «${c.abrir}»`)
      await pagina.close()
      continue
    }
    await boton.click()
    await pagina.waitForTimeout(900)
  }

  // Cada marca se localiza por su ROL, no recorriendo el DOM: un botón como
  // botón y un campo por su etiqueta. Si no está, esto falla nombrándolo —
  // que es lo que convierte la captura en algo verificable.
  const medidas = []
  let falta = null
  for (const m of c.marcas) {
    const texto = m.boton ?? m.campo
    const loc = m.boton
      ? pagina.getByRole('button', { name: m.boton }).first()
      : pagina.getByLabel(m.campo).first()
    if ((await loc.count()) === 0) { falta = texto; break }
    const caja = await loc.boundingBox()
    if (!caja) { falta = `${texto} (existe pero no se ve)`; break }
    medidas.push({ ...caja, etiqueta: m.etiqueta })
  }
  if (falta) {
    problemas.push(`${c.nombre}: no se encontró en la pantalla «${falta}»`)
    await pagina.close()
    continue
  }

  await pagina.evaluate(`(${DIBUJAR})(${JSON.stringify(medidas)})`)

  const destino = path.join(DESTINO, `${c.nombre}.png`)
  await pagina.screenshot({ path: destino })
  await pagina.close()
  const kb = Math.round(fs.statSync(destino).size / 1024)
  console.log(`  ✓ ${c.nombre}.png  ·  ${kb} kB  ·  ${c.pie}`)
}

await navegador.close()

if (problemas.length) {
  console.error('\n✗ Hay capturas que no se pudieron anotar:\n')
  for (const p of problemas) console.error(`    ${p}`)
  console.error(
    '\n  Esto NO es un fallo de la herramienta: quiere decir que la pantalla\n' +
      '  cambió y la guía que usa esa captura ya no describe lo que hay.\n',
  )
  process.exit(1)
}

console.log(`\n✓ ${CAPTURAS.length} capturas en documentacion/07-entrega/capturas/\n`)
