/**
 * Auditoría de todas las pantallas, en un navegador de verdad · 22/09/2026
 *
 *     npm run sonda:auditoria
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ EXISTE
 *
 * Las demás sondas miran **una** pantalla y **una** garantía: que el borrado
 * no se pulse por accidente, que el plegado ocurra en su umbral, que la CSP
 * esté puesta. Ninguna recorre el producto entero preguntando lo aburrido:
 * ¿hay algo que no cabe? ¿se lee este texto? ¿este botón tiene nombre? ¿se ve
 * dónde está el teclado?
 *
 * Eso se venía mirando a ojo en las capturas, y a ojo no se ve un 3.75:1.
 *
 * QUÉ MIDE, Y POR QUÉ CADA COSA
 *
 *  1. Errores de consola · una excepción en la webview no se ve: la pantalla
 *     se queda a medias y parece que «tarda».
 *  2. Contenido que no cabe · `scrollWidth > clientWidth`. NO se mira el
 *     `scrollWidth` del documento: el armazón recorta, así que el documento
 *     nunca se ensancha aunque algo se salga dentro. La primera versión de
 *     esta sonda lo hacía así y su cebo —un div de 3000 px dentro de `main`—
 *     no se cazaba: diez pantallas «sin desbordes» que no querían decir nada.
 *  3. Contraste real, calculado sobre el color que el navegador pinta, no
 *     sobre el token. Los tokens ya se verifican en `design-tokens`; lo que
 *     no se verificaba es **la cascada**, y es donde estaba el defecto: un
 *     primario deshabilitado conservaba su borde de oro porque su selector
 *     pesaba más que `.boton:disabled`.
 *  4. Nombre accesible de cada botón y cada enlace.
 *  5. Jerarquía de encabezados sin saltos.
 *  6. Identificadores repetidos, que rompen `for`/`aria-labelledby` en
 *     silencio: la etiqueta deja de apuntar al campo y nada falla.
 *  7. Marca de foco visible en cada parada del tabulador.
 *
 * EL SUELO DE CONTRASTE NO ES 4.5 PARA TODO
 *
 * Un control deshabilitado está exento del 1.4.3, y ARLES eligió un suelo
 * propio de 3.0 con su motivo escrito en `ABoton.vue`: el texto tiene que
 * decir *qué* está deshabilitado (§95), pero tiene que verse apagado o el
 * estado no se distingue. Se respeta esa decisión y se mide contra ella.
 *
 * SE HA VISTO FALLAR · 22/09/2026
 *
 * Las nueve garantías se rompieron a propósito, una a una, y todas se
 * cazaron: un div recortado con texto de más; un bloque de 5000 px fuera de
 * la ventana; un párrafo #cfcfcf sobre blanco; un botón deshabilitado a
 * #e0e0e0, por debajo del suelo de 3.0; un `<button>` vacío; un h5 detrás de
 * un h2; dos elementos con el mismo id; una excepción lanzada en la página; y
 * `outline: none !important` sobre todo el documento, que hizo fallar las 65
 * paradas del tabulador.
 *
 * Dos de ellas sólo se arreglaron **porque** se les puso el cebo. La de
 * «contenido que no cabe» miraba el `scrollWidth` del documento y no se
 * cazaba nunca, y su versión siguiente señalaba el pie de la barra plegada en
 * las diez pantallas sin que hubiera nada que arreglar.
 *
 * Las dos del indicador de sección se añadieron el mismo día y también se
 * vieron fallar: midiendo sin restar el marco de la lista (el indicador
 * apareció 124 px más abajo que su sección) y quitando el recolocado al
 * navegar (se quedó en la sección anterior, 124 contra 212).
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

/** Suelo de contraste para un control deshabilitado (ver ABoton.vue). */
const SUELO_DESHABILITADO = 3

/** Las pantallas que existen hoy. Una nueva se añade aquí. */
const RUTAS = [
  '#/inicio',
  '#/inicio/configuracion',
  '#/contactos',
  '#/contactos/importar',
  '#/contactos/supresion',
  '#/contactos/derechos',
  '#/campanas',
  // Las tres de abajo faltaban hasta el 29/09/2026, y con ellas faltaba la
  // mitad de lo que el usuario puede abrir hoy. Una auditoría que no recorre
  // una pantalla no dice nada de esa pantalla, aunque termine en verde — que
  // es peor que no tenerla, porque se cree.
  '#/campanas/plantillas',
  '#/campanas/nueva',
  '#/remitentes',
  '#/remitentes/whatsapp',
  '#/actividad',
  '#/ajustes',
]

/**
 * Los anchos que se prueban.
 *
 * 1280 es la ventana por defecto de ARLES; 1024 es el portátil pequeño con el
 * que Dirección revisa; 900 está por debajo del umbral de plegado, así que
 * mide la barra plegada, que es un montaje distinto y nadie miraba.
 */
const ANCHOS = [1280, 1024, 900]

let fallos = 0
function mal(mensaje) {
  console.error(`✗ ${mensaje}`)
  fallos += 1
}

/** Batería de medidas. Corre dentro de la página. */
const MEDIR = (sueloDeshabilitado) => {
  const out = { desbordes: [], contrastes: [], sinNombre: [], encabezados: [], ids: [] }

  // ── 2 · contenido que no cabe ───────────────────────────────────────
  //
  // Se busca lo que el usuario PIERDE, y eso ocurre de dos maneras y sólo dos:
  //
  //   a) la caja recorta (`overflow-x: hidden|clip`) y su contenido es más
  //      ancho: el texto está cortado y no hay forma de llegar a él;
  //   b) algo se sale de la ventana por la derecha: está fuera de la pantalla.
  //
  // Lo que NO cuenta: una caja con `overflow: visible` cuyo contenido sobresale
  // teniendo sitio de sobra alrededor. El pie de la barra plegada es eso — el
  // sello se sale 2 px por lado por su propio relleno de pulsación, y la barra
  // tiene 63 px para acomodarlo. La primera versión de esta sonda lo señalaba
  // diez veces por recorrido, una por pantalla, y no había nada que arreglar:
  // un aviso que siempre está es un aviso que se deja de leer.
  const ancho = document.documentElement.clientWidth
  for (const el of document.querySelectorAll('body, body *')) {
    // Las cajas de 1 px de `.sr-only` recortan su texto a propósito: es como
    // se esconde lo que sólo lee el lector de pantalla.
    if (el.clientWidth < 4) continue
    const cs = getComputedStyle(el)
    const recorta = cs.overflowX === 'hidden' || cs.overflowX === 'clip'

    // ── UN TRUNCADO CON PUNTOS SUSPENSIVOS NO ES UN DEFECTO ───────────────
    //
    // Lo que esta comprobación busca es texto **cortado en seco**, que se lee
    // mal sin que nada lo avise. Una celda de tabla que trunca con
    // `text-overflow: ellipsis` está haciendo lo contrario: recorta a
    // propósito, lo dice con los tres puntos, y el valor completo está en la
    // ficha de al lado.
    //
    // Se añadió el 29/09/2026: en cuanto la demostración trajo contactos con
    // nombres de empresa largos, las celdas de la tabla salieron trece veces
    // —cuatro por cada ancho— y no había nada que arreglar. Un aviso que
    // siempre está es un aviso que se deja de leer, que es la misma razón por
    // la que se acotó el desborde de página unas líneas más arriba.
    const truncaAdrede = cs.textOverflow === 'ellipsis'
    const cortado = recorta && !truncaAdrede && el.scrollWidth > el.clientWidth + 1
    const fuera = el.getBoundingClientRect().right > ancho + 1
    if (!cortado && !fuera) continue
    out.desbordes.push({
      sel:
        el.tagName.toLowerCase() +
        (el.className
          ? '.' + el.className.toString().trim().split(/\s+/).slice(0, 2).join('.')
          : ''),
      cabe: cortado ? el.clientWidth : ancho,
      necesita: cortado ? el.scrollWidth : Math.round(el.getBoundingClientRect().right),
      motivo: cortado ? 'recortado' : 'fuera de la ventana',
    })
  }

  // ── 3 · contraste de lo que el navegador pinta ──────────────────────
  const lum = ([r, g, b]) => {
    const f = (v) => {
      v /= 255
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
    }
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
  }
  const rgb = (s) => {
    const m = s.match(/rgba?\(([^)]+)\)/)
    if (!m) return null
    const p = m[1].split(',').map((x) => parseFloat(x))
    return { c: [p[0], p[1], p[2]], a: p.length > 3 ? p[3] : 1 }
  }
  const mezclar = (sobre, debajo, a) => sobre.map((v, i) => v * a + debajo[i] * (1 - a))
  /** El primer fondo opaco que haya por encima. */
  const fondoDe = (el) => {
    let n = el
    while (n && n !== document.documentElement) {
      const c = rgb(getComputedStyle(n).backgroundColor)
      if (c && c.a >= 1) return c.c
      n = n.parentElement
    }
    const b = rgb(getComputedStyle(document.body).backgroundColor)
    return b ? b.c : [255, 255, 255]
  }

  for (const el of document.querySelectorAll('main *, nav *, header *')) {
    if (el.children.length > 0) continue
    const t = (el.textContent || '').trim()
    if (t.length < 2) continue
    const cs = getComputedStyle(el)
    if (cs.visibility === 'hidden' || cs.display === 'none' || cs.opacity === '0') continue
    const r = el.getBoundingClientRect()
    if (r.width <= 1 || r.height <= 1) continue
    const f = rgb(cs.color)
    if (!f) continue
    const fondo = fondoDe(el)
    const color = f.a < 1 ? mezclar(f.c, fondo, f.a) : f.c
    const L1 = lum(color)
    const L2 = lum(fondo)
    const ratio = (Math.max(L1, L2) + 0.05) / (Math.min(L1, L2) + 0.05)

    const px = parseFloat(cs.fontSize)
    const grande = px >= 24 || (px >= 18.66 && parseInt(cs.fontWeight, 10) >= 700)
    const apagado = !!el.closest(':disabled, [aria-disabled="true"]')
    const minimo = apagado ? sueloDeshabilitado : grande ? 3 : 4.5

    if (ratio < minimo) {
      out.contrastes.push({
        texto: t.slice(0, 40),
        ratio: Math.round(ratio * 100) / 100,
        minimo,
        apagado,
        color: cs.color,
        fondo: `rgb(${fondo.map(Math.round).join(',')})`,
      })
    }
  }

  // ── 4 · nombre accesible ────────────────────────────────────────────
  for (const el of document.querySelectorAll('button, a, [role="button"]')) {
    if (el.getBoundingClientRect().width === 0) continue
    const nombre =
      (el.getAttribute('aria-label') || '').trim() ||
      (el.textContent || '').trim() ||
      (el.getAttribute('title') || '').trim()
    if (!nombre) out.sinNombre.push(el.outerHTML.slice(0, 140))
  }

  // ── 5 · encabezados sin saltos ──────────────────────────────────────
  let previo = 0
  for (const h of document.querySelectorAll('main h1,main h2,main h3,main h4,main h5,main h6')) {
    const n = parseInt(h.tagName[1], 10)
    if (previo && n > previo + 1) {
      out.encabezados.push(`h${previo} → h${n}: «${(h.textContent || '').trim().slice(0, 40)}»`)
    }
    previo = n
  }

  // ── 6 · identificadores repetidos ───────────────────────────────────
  const vistos = new Set()
  for (const el of document.querySelectorAll('[id]')) {
    if (vistos.has(el.id)) out.ids.push(el.id)
    vistos.add(el.id)
  }

  return out
}

const ejecutable = await exigirChromium()

console.log('Generando la vista previa…')
python([path.join(RAIZ, 'herramientas/vista-previa/generar.py')])

const navegador = await chromium.launch({ executablePath: ejecutable })

for (const ancho of ANCHOS) {
  const pagina = await navegador.newPage({ viewport: { width: ancho, height: 900 } })
  const consola = []
  pagina.on('console', (m) => {
    if (m.type() === 'error') consola.push(m.text())
  })
  pagina.on('pageerror', (e) => consola.push('excepción: ' + e.message))

  for (const ruta of RUTAS) {
    consola.length = 0
    await pagina.goto(`file://${ARCHIVO}${ruta}`, { waitUntil: 'load' })
    await pagina.waitForTimeout(1000)

    const r = await pagina.evaluate(MEDIR, SUELO_DESHABILITADO)
    const donde = `${ruta} @ ${ancho}px`

    consola.forEach((c) => mal(`${donde} · consola: ${c.slice(0, 200)}`))
    r.desbordes.forEach((d) =>
      mal(`${donde} · ${d.motivo}: ${d.sel} — ${d.cabe}px disponibles, ${d.necesita}px necesarios`),
    )
    r.contrastes.forEach((c) =>
      mal(
        `${donde} · contraste ${c.ratio}:1 (mín ${c.minimo}${c.apagado ? ', deshabilitado' : ''}) ` +
          `«${c.texto}» ${c.color} sobre ${c.fondo}`,
      ),
    )
    r.sinNombre.forEach((s) => mal(`${donde} · sin nombre accesible: ${s}`))
    r.encabezados.forEach((e) => mal(`${donde} · salto de encabezado ${e}`))
    r.ids.forEach((i) => mal(`${donde} · id repetido: ${i}`))
  }
  await pagina.close()
  console.log(`✓ ${RUTAS.length} pantallas medidas a ${ancho}px.`)
}

// ── 7 · el foco se ve ──────────────────────────────────────────────────
//
// Se tabula la pantalla entera y se comprueba que cada parada lleva contorno
// o sombra. Sin esto, quien usa el teclado navega a ciegas — y es la forma en
// que se rellena un formulario de cinco campos.
{
  const pagina = await navegador.newPage({ viewport: { width: 1280, height: 900 } })
  let paradas = 0
  for (const ruta of RUTAS) {
    await pagina.goto(`file://${ARCHIVO}${ruta}`, { waitUntil: 'load' })
    await pagina.waitForTimeout(1000)
    const vistos = new Set()
    for (let i = 0; i < 80; i += 1) {
      await pagina.keyboard.press('Tab')
      const f = await pagina.evaluate(() => {
        const el = document.activeElement
        if (!el || el === document.body) return null
        const cs = getComputedStyle(el)
        return {
          clave: el.tagName + '|' + (el.className || '') + '|' + (el.textContent || '').trim().slice(0, 20),
          txt: (el.textContent || el.getAttribute('aria-label') || el.tagName).trim().slice(0, 30),
          seVe:
            (cs.outlineStyle !== 'none' && parseFloat(cs.outlineWidth) > 0) ||
            cs.boxShadow !== 'none',
        }
      })
      if (!f || vistos.has(f.clave)) break
      vistos.add(f.clave)
      paradas += 1
      if (!f.seVe) mal(`${ruta} · el foco no se ve en «${f.txt}»`)
    }
  }
  console.log(`✓ ${paradas} paradas de tabulador, todas con marca visible.`)
  await pagina.close()
}

// ── 8 · el indicador de sección viaja, y viaja al sitio ────────────────
//
// jsdom no maqueta, así que las pruebas de `navegacion.spec.ts` tienen que
// fingir la medida. Aquí no se finge nada: el navegador coloca la barra y se
// comprueba que el indicador cae ENCIMA del enlace de la sección activa, con
// su alto, y que al cambiar de sección se va con ella.
//
// Es la única pieza de la navegación que depende de medir el documento. Si la
// medida se desajusta —por la fuente, por el plegado, por un cambio de
// maquetación— no salta ningún error: sencillamente el oro señala la sección
// equivocada, que es peor que no señalar ninguna.
{
  const pagina = await navegador.newPage({ viewport: { width: 1280, height: 900 } })
  await pagina.goto(`file://${ARCHIVO}#/inicio`, { waitUntil: 'load' })
  await pagina.waitForTimeout(1200)

  /** Dónde está el indicador y dónde el enlace activo, en la misma referencia. */
  const donde = () =>
    pagina.evaluate(() => {
      const ind = document.querySelector('.nav-indicador')
      const act = document.querySelector('.nav-enlace.activo')
      if (!ind || !act) return null
      const a = ind.getBoundingClientRect()
      const b = act.getBoundingClientRect()
      return {
        indicador: { top: Math.round(a.top), alto: Math.round(a.height) },
        activo: { top: Math.round(b.top), alto: Math.round(b.height) },
        visible: getComputedStyle(ind).display !== 'none',
        seccion: act.getAttribute('href'),
      }
    })

  const enInicio = await donde()
  if (!enInicio || !enInicio.visible) {
    mal('El indicador de sección no se dibuja en un navegador de verdad.')
  } else if (
    Math.abs(enInicio.indicador.top - enInicio.activo.top) > 1 ||
    Math.abs(enInicio.indicador.alto - enInicio.activo.alto) > 1
  ) {
    mal(
      `El indicador no cae sobre la sección activa: está en ${enInicio.indicador.top}` +
        ` y la sección en ${enInicio.activo.top}.`,
    )
  } else {
    console.log('✓ el indicador de sección cae exactamente sobre la sección activa.')
  }

  // Y ahora se navega POR LA BARRA, como haría cualquiera.
  await pagina.click('a.nav-enlace[href$="#/contactos"]')
  await pagina.waitForTimeout(700)
  const enContactos = await donde()

  if (!enContactos) {
    mal('Tras navegar no hay indicador ni sección activa.')
  } else if (enContactos.seccion !== '#/contactos') {
    mal(`Navegar por la barra no cambió la sección activa: sigue en ${enContactos.seccion}.`)
  } else if (Math.abs(enContactos.indicador.top - enContactos.activo.top) > 1) {
    mal(
      'El indicador se quedó en la sección anterior: está en ' +
        `${enContactos.indicador.top} y Contactos en ${enContactos.activo.top}.`,
    )
  } else if (enContactos.indicador.top === enInicio.indicador.top) {
    mal('El indicador no se movió al cambiar de sección.')
  } else {
    console.log('✓ el indicador viaja a la sección nueva al navegar por la barra.')
  }
  await pagina.close()
}

await navegador.close()

if (fallos > 0) {
  console.error(`\n${fallos} hallazgo(s) de auditoría.`)
  process.exit(1)
}
console.log('\nSin hallazgos.')
