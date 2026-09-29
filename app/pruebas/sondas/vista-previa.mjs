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

  // ── ESTA SONDA RECORRE «LA PRIMERA VEZ», Y HAY QUE PEDIRLA ──────────────
  //
  // El bloque A de la checklist es lo que ve alguien que abre ARLES por
  // primera vez: la portada, el botón de configurar la empresa, el alta. Eso
  // dejó de ser el arranque de la vista previa el 29/09/2026, cuando el
  // decorado pasó a traer la empresa ya configurada —para que la demostración
  // no se contara dos historias a la vez—. La sonda se quedó esperando
  // treinta segundos un botón que ya no existía, y murió con un timeout.
  //
  // Vaciar el estado guardado devuelve a la pantalla de primera vez, que es
  // exactamente lo que estas comprobaciones miden. Un `{}` basta: el simulador
  // sirve el decorado sólo cuando no hay NADA guardado.

  //
  // Se siembra **una sola vez**: `addInitScript` corre en cada carga, y la
  // comprobación E-3 recarga la página a propósito para ver si el tema elegido
  // se recuerda. Sin la marca, la siembra borraba esa preferencia en la
  // recarga y la sonda acusaba al producto de no recordar nada.
  // El §30 prohíbe el almacenamiento del navegador, y con razón: ahí no puede
  // vivir nada de la aplicación. Esto no es la aplicación — es código que la
  // sonda inyecta en la página de la DEMOSTRACIÓN para colocar su decorado,
  // y lo que escribe son contactos de mentira. El llavero no tiene nada que
  // ver aquí, así que la regla se desactiva en estas tres líneas y sólo en
  // ellas, con el motivo a la vista.
  /* eslint-disable no-restricted-syntax */
  await pagina.addInitScript((valor) => {
    try {
      if (sessionStorage.getItem('arles-sonda-sembrada')) return
      sessionStorage.setItem('arles-vista-previa', valor)
      sessionStorage.setItem('arles-sonda-sembrada', '1')
    } catch { /* bloqueado: se sigue con el decorado y la sonda lo dirá */ }
  }, '{}')
  /* eslint-enable no-restricted-syntax */

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
  //
  // **Esta comprobación cambió el 21/09/2026.** Antes exigía que el botón
  // llevara a `ajustes`, y eso era el fallo que Dirección señaló mirando la
  // pantalla: Ajustes es de la aplicación —tema, versión, respaldos—, no del
  // negocio. Ahora se exige lo contrario, y además que **no se salga de
  // INICIO**: saltar de sección al pulsar la única acción de la portada
  // desorienta justo en el primer minuto de uso.
  await pagina.click('text=Configurar mi empresa')
  await pagina.waitForTimeout(500)
  if (!pagina.url().includes('/inicio/configuracion')) {
    mal(`El botón de la portada no lleva a la configuración: ${pagina.url()}`)
  }
  if (/#\/ajustes/.test(pagina.url())) {
    mal('El botón de la portada volvió a mandar a Ajustes.')
  }

  // O-3 · Y AHORA LA BARRA TIENE QUE DECIR DÓNDE ESTÁ · 21/09/2026
  //
  // Segunda observación de Dirección sobre la misma pantalla: llegó a la
  // configuración y **la barra lateral estaba entera apagada**. No había forma
  // de saber en qué sección se encontraba.
  //
  // La causa medida: `router-link-active` no marca `/inicio` estando en
  // `/inicio/configuracion`, porque son rutas hermanas. Lo mismo les pasaba a
  // `/contactos/importar` y `/contactos/supresion` desde la entrega 3.2.
  //
  // Esto se mide AQUÍ, en un navegador de verdad, porque es donde se vio:
  // las pruebas de pantalla sustituyen los enlaces por dobles que no calculan
  // ningún estado activo, y por eso el fallo sobrevivió tres entregas.
  const inicioMarcado = await pagina.evaluate(() => {
    const a = document.querySelector('nav a.nav-enlace[href$="#/inicio"]')
    return a ? a.classList.contains('activo') : null
  })
  if (inicioMarcado !== true) {
    mal(
      `Estando en la configuración, la sección INICIO no queda marcada en la barra ` +
        `(classList.activo = ${inicioMarcado}). Es el fallo que Dirección vio el 21/09.`,
    )
  } else {
    console.log('✓ O-3 · la barra marca INICIO estando en la configuración.')
  }

  // Y la subsección: indentada bajo Inicio, marcada, y sin las de Contactos.
  const sub = await pagina.evaluate(() => {
    const enlaces = [...document.querySelectorAll('nav a.sub-enlace')]
    return {
      total: enlaces.length,
      marcada: enlaces.some(
        (a) => a.getAttribute('href').endsWith('#/inicio/configuracion') &&
          a.classList.contains('activo'),
      ),
      ajenas: enlaces.filter((a) => a.getAttribute('href').includes('/contactos/')).length,
    }
  })
  if (!sub.marcada) {
    mal(`La subsección Configuración no se enseña marcada en la barra (vistas: ${sub.total}).`)
  } else if (sub.ajenas > 0) {
    mal(`Se despliegan ${sub.ajenas} subsecciones de otras secciones: la barra deja de orientar.`)
  } else {
    console.log('✓ O-3 · la subsección Configuración se enseña marcada, y sólo la suya.')
  }

  // §19 · EL ESTADO NO PUEDE VIVIR SÓLO EN EL COLOR · 22/09/2026
  //
  // Medido auditando: el relleno de la subsección activa da 1.76:1 en oscuro y
  // 1.80:1 en claro sobre la barra — por debajo del 3:1 mínimo de un elemento
  // no textual. La sección madre aguanta ese relleno flojo porque ADEMÁS lleva
  // su barra de acento; la subsección no llevaba nada.
  //
  // Se mide AQUÍ y no en una prueba de componente porque la barra es un
  // pseudoelemento, y jsdom no los calcula: allí sólo se podría volver a
  // afirmar que la clase está puesta, cosa que pasaría igual con el CSS
  // borrado.
  const acento = await pagina.evaluate(() => {
    const activa = document.querySelector('a.sub-enlace.activo')
    const dormida = document.querySelector('nav a.sub-enlace:not(.activo)')
    const alto = (n) => (n ? parseFloat(getComputedStyle(n, '::before').height) || 0 : null)
    return { activa: alto(activa), dormida: alto(dormida) }
  })
  if (!(acento.activa > 0)) {
    mal(
      `La subsección activa no dibuja barra de acento (alto ${acento.activa}px). ` +
        `Su estado quedaría sólo en un relleno de 1.76:1, que el §19 prohíbe.`,
    )
  } else if (acento.dormida !== null && acento.dormida > 0) {
    mal(`Una subsección que no está abierta también dibuja barra (${acento.dormida}px).`)
  } else {
    console.log(`✓ §19 · la subsección activa lleva barra de acento (${acento.activa}px), no sólo relleno.`)
  }

  // Las migas, que son el indicativo que sobrevive a plegar la barra.
  const migas = await pagina.evaluate(() => {
    const n = document.querySelector('.migas')
    return n ? n.innerText.replace(/\s+/g, ' ').trim() : null
  })
  if (!migas || !migas.includes('Inicio') || !migas.includes('Configuración')) {
    mal(`Las migas no dicen dónde estás: ${JSON.stringify(migas)}`)
  } else {
    console.log(`✓ O-3 · las migas dicen «${migas}».`)
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
  //
  // Hay que ir a Ajustes: la apariencia vive ahí y los datos del negocio no
  // (21/09/2026). Antes las dos cosas estaban en la misma pantalla, y por eso
  // esta sonda no navegaba.
  await pagina.click('nav a[href*="ajustes"], a[href$="#/ajustes"]')
  await pagina.waitForTimeout(500)
  if (!/#\/ajustes/.test(pagina.url())) {
    mal(`No se llega a Ajustes desde la barra: ${pagina.url()}`)
  }
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
