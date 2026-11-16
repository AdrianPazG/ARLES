/**
 * Que ningún texto se monte sobre otro, ni se quede cortado · 14/10/2026
 *
 *     npm run sonda:encimado
 *
 * ─────────────────────────────────────────────────────────────────────────
 * POR QUÉ EXISTE, Y QUIÉN LA PIDIÓ
 *
 * Dirección abrió la aplicación el 13/10/2026 y escribió esto de la pantalla
 * de Remitentes: *«el texto se ve extremadamente desordenado y poco
 * profesional»*. Y al pedir las correcciones: *«evitar texto encimado o
 * desordenado»*.
 *
 * **Las dos sondas que recorren el producto estaban en verde ese día.**
 * `auditoria.mjs` mide nueve cosas y `recorrido.mjs` otras tantas, y ninguna
 * de las dieciocho pantallas daba un hallazgo. Es el aviso de este proyecto
 * en su forma literal: una pantalla puede pasar todas sus comprobaciones y
 * seguir estando mal.
 *
 * Lo que pasaba es que **nadie medía lo que Dirección estaba viendo**:
 *
 *  · `auditoria.mjs` mide desbordes con `scrollWidth > clientWidth`, que caza
 *    lo que **no cabe de ancho**. Dos párrafos montados uno sobre otro caben
 *    perfectamente: su contenedor no se ensancha ni un píxel.
 *  · `recorrido.mjs` compara el borde izquierdo y el hueco vertical **del
 *    título**. Lo que pase cien píxeles más abajo no lo mira.
 *
 * Así que esta sonda mide las cuatro cosas que quedaban fuera, y las cuatro
 * son geométricas — se contestan con rectángulos, no con criterio:
 *
 *  1. **Dos textos encimados.** Los rectángulos de dos elementos con texto
 *     que se cruzan sin que uno contenga al otro en el DOM.
 *  2. **Texto cortado.** Un elemento cuyo contenido no cabe de alto y cuyo
 *     `overflow` lo recorta: la última línea se queda a medias o desaparece.
 *  3. **Texto que se sale de su contenedor.** El rectángulo del hijo asoma
 *     por debajo o por la derecha del padre. No se ve como un desborde —el
 *     armazón recorta— se ve como texto puesto encima de lo que sigue.
 *  4. **Un rótulo recortado por los puntos suspensivos.** Cortar **un dato**
 *     está bien: la dirección entera se lee al abrir la ficha. Cortar **un
 *     rótulo** no: una cabecera de columna o una etiqueta de campo es la
 *     única pista de qué se está mirando, y lo que se pierde es el final,
 *     que es donde está lo que la distingue de la de al lado.
 *  5. **Texto estrujado**, cayendo de una palabra por línea. Cabe, no
 *     desborda y no está cortado — las otras cuatro lo dan por bueno — y es
 *     ilegible como frase. Lo encontró Dirección el 16/10/2026 con el panel
 *     del correo de prueba abierto (H-07).
 *
 * ─────────────────────────────────────────────────────────────────────────
 * LOS CUATRO FALSOS POSITIVOS QUE HAY QUE CERRAR, Y POR QUÉ
 *
 * Medir solapes a lo bruto da un torrente de hallazgos legítimos, y una sonda
 * que grita de más se desactiva: es el error que la 8.7 ya cometió al comparar
 * huecos entre maquetaciones distintas. Así que se excluyen cuatro casos, y
 * **cada exclusión es una decisión escrita**, no un umbral subido hasta que
 * callara:
 *
 *  1. **Anidados.** Un `<strong>` dentro de un `<p>` se solapa por
 *     definición. Se compara sólo lo que no se contiene en el DOM.
 *  2. **Posicionados a propósito.** `absolute`, `fixed` y `sticky` existen
 *     para ponerse encima: el indicador de sección cae **sobre** el enlace
 *     activo y eso es el diseño. Si el elemento o alguno de sus ancestros
 *     está posicionado, no se compara.
 *  3. **Transparentes y de área cero.** Un elemento con `opacity: 0` o un
 *     rectángulo vacío no se ve, así que no puede estar encima de nada.
 *  4. **Roces de subpíxel.** Dos cajas pegadas pueden compartir una fracción
 *     de píxel por el redondeo del navegador. Se exige que el cruce mida al
 *     menos `MINIMO_PX` en las dos direcciones **y** tape una parte
 *     apreciable del menor de los dos: un píxel de más no es un defecto.
 *
 * ─────────────────────────────────────────────────────────────────────────
 * SE HA VISTO FALLAR · 14/10/2026 (§2)
 *
 * Las tres medidas se rompieron a propósito, en el código de verdad y no en
 * la sonda, restaurando desde una copia hecha antes (nunca `git checkout`
 * sobre un archivo con trabajo sin confirmar — eso ya costó una entrega):
 *
 *  · encimado → `position: relative; top: -28px` en `.remitentes__proveedor`:
 *    falla en `/remitentes` nombrando los dos textos que se cruzan y el área.
 *  · cortado  → `max-height: 12px; overflow: hidden` en la entradilla de
 *    Respaldos: falla nombrando el alto que hace falta y el que hay.
 *  · se sale  → `width: 150%` en la tabla de Contactos: falla nombrando
 *    cuántos píxeles asoma por la derecha de su contenedor.
 *
 * Y una cuarta ruptura, la que importa de verdad: **quitarle las rutas** y
 * dejar una. `rutas.mjs` revienta por su cuenta si encuentra menos de su
 * mínimo, así que esta sonda no puede recorrer cero pantallas y terminar en
 * verde — que es la trampa que este proyecto ya ha tenido seis veces.
 */
import { fileURLToPath } from 'node:url'
import path from 'node:path'

import { chromium } from 'playwright-core'

import { exigirChromium } from './navegador.mjs'
import { leerRutasConAlmohadilla } from './rutas.mjs'
import { python } from './ordenes.mjs'

const AQUI = path.dirname(fileURLToPath(import.meta.url))
const RAIZ = path.resolve(AQUI, '../../..')
const ARCHIVO = path.join(RAIZ, 'vista-previa/ARLES-vista-previa.html')

const RUTAS = leerRutasConAlmohadilla()

/**
 * Las cuatro ventanas de `auditoria.mjs`, por el mismo motivo.
 *
 * La de 960×540 es la que de verdad tiene quien trabaja al 200 %: el escalado
 * del sistema no encoge los píxeles CSS, encoge la ventana. Y es justo donde
 * un texto se monta sobre otro, porque es donde menos sitio hay.
 */
const VENTANAS = [
  [1280, 900],
  [1024, 900],
  [900, 900],
  [960, 540],
]

/** Cuánto tiene que medir un cruce para contar. Menos es redondeo. */
const MINIMO_PX = 3

/** Qué parte del elemento menor tiene que quedar tapada para contar. */
const PARTE_TAPADA = 0.2

/** Cuánto puede asomar un hijo de su padre antes de ser un defecto. */
const MARGEN_DE_SALIDA = 2

let fallos = 0
function mal(linea) {
  console.error(`✗ ${linea}`)
  fallos += 1
}

/**
 * Lo que se mide dentro del navegador.
 *
 * Devuelve tres listas, una por medida. Se hace todo en una pasada porque
 * recorrer el DOM cuatro veces por pantalla y dieciocho pantallas por ventana
 * son 288 recorridos, y la sonda ya tarda.
 */
const MEDIR = ([MINIMO_PX, PARTE_TAPADA, MARGEN_DE_SALIDA]) => {
  const main = document.querySelector('main') ?? document.body

  /** ¿Tiene texto propio, no sólo de sus hijos? */
  const tieneTextoPropio = (el) => {
    for (const n of el.childNodes) {
      if (n.nodeType === Node.TEXT_NODE && n.textContent.trim().length > 0) return true
    }
    return false
  }

  const texto = (el) => el.textContent.trim().replace(/\s+/g, ' ').slice(0, 60)

  /**
   * ¿Está este elemento —o alguno de sus ancestros— puesto encima a propósito?
   *
   * Se sube hasta `main` porque un hijo dentro de un contenedor `absolute`
   * hereda el derecho a solaparse: el que decidió ponerse encima es el padre.
   */
  const estaPosicionado = (el) => {
    for (let n = el; n && n !== document.body; n = n.parentElement) {
      const e = getComputedStyle(n)
      if (e.position !== 'static' && e.position !== 'relative') return true
      // Un `relative` con desplazamiento también es una decisión de poner algo
      // encima. Un `relative` sin desplazamiento no mueve nada y sí se mide.
      if (e.position === 'relative' && (e.top !== 'auto' || e.left !== 'auto')) {
        if (e.top !== '0px' && e.top !== 'auto') return true
        if (e.left !== '0px' && e.left !== 'auto') return true
      }
      if (parseFloat(e.opacity) < 0.95) return true
      if (e.visibility === 'hidden' || e.display === 'none') return true
    }
    return false
  }

  const candidatos = []
  for (const el of main.querySelectorAll('*')) {
    if (!tieneTextoPropio(el)) continue
    const r = el.getBoundingClientRect()
    // ── SE EXIGE ALTO, NO ANCHO, Y LA DIFERENCIA ES EL CASO PEOR ─────────
    //
    // Esto decía `r.width < 1 || r.height < 1`, para descartar lo que no se
    // ve. Y descartaba justo el síntoma más extremo del estrujado: un bloque
    // al que no le queda NADA de ancho tiene `width: 0` y un alto enorme
    // —seis líneas, una palabra cada una— porque el texto se desborda.
    //
    // Medido con la ficha de Remitentes reproducida como la vio Dirección:
    // «smtp.gmail.com:587 · Cifrado (STARTTLS, puerto 587)», 6 palabras,
    // 6 líneas, **ancho 0**. La medida 5 lo habría cazado y no llegaba a
    // verlo.
    //
    // Un elemento de alto cero no se ve; uno de ancho cero y alto 120 sí, y
    // es el defecto. Las medidas que necesitan ancho ya lo comprueban por su
    // cuenta.
    if (r.height < 1) continue
    // ── EL RECTÁNGULO NO DICE SI ALGO SE VE, Y ESO DIO NUEVE HALLAZGOS
    //    FALSOS EL DÍA QUE SE ESCRIBIÓ ESTA SONDA
    //
    // El contenido de un `<details>` **cerrado** sigue devolviendo un
    // `getBoundingClientRect()` con medidas, y todos sus hijos caen apilados
    // en la posición del `<details>`. Medido en Respaldos: el primer `<li>`
    // del detalle de tablas devolvía 540×20 en y=424 con el `<details>`
    // cerrado. Resultado: las cuatro tablas del detalle salían «encimadas»
    // sobre el título «Protección» y sobre la ayuda de la contraseña, en las
    // cuatro ventanas. Treinta y cinco hallazgos, de los que nueve por
    // ventana eran éste.
    //
    // `checkVisibility()` contesta la pregunta de verdad —¿se ve?— y para ese
    // mismo `<li>` devolvía `false`. Es lo que hay que preguntar: un solape
    // entre dos cosas que no se ven no existe.
    if (!el.checkVisibility({
      contentVisibilityAuto: true,
      opacityProperty: true,
      visibilityProperty: true,
    })) continue
    if (estaPosicionado(el)) continue
    candidatos.push({ el, r, txt: texto(el) })
  }

  // ── 1 · dos textos encimados ──────────────────────────────────────────
  const encimados = []
  for (let i = 0; i < candidatos.length; i += 1) {
    for (let j = i + 1; j < candidatos.length; j += 1) {
      const a = candidatos[i]
      const b = candidatos[j]
      if (a.el.contains(b.el) || b.el.contains(a.el)) continue

      const ancho = Math.min(a.r.right, b.r.right) - Math.max(a.r.left, b.r.left)
      const alto = Math.min(a.r.bottom, b.r.bottom) - Math.max(a.r.top, b.r.top)
      if (ancho < MINIMO_PX || alto < MINIMO_PX) continue

      const cruce = ancho * alto
      const menor = Math.min(a.r.width * a.r.height, b.r.width * b.r.height)
      if (cruce < menor * PARTE_TAPADA) continue

      encimados.push({
        a: a.txt,
        b: b.txt,
        cruce: `${Math.round(ancho)}×${Math.round(alto)} px`,
      })
    }
  }

  // ── 2 · texto cortado ─────────────────────────────────────────────────
  const cortados = []
  for (const { el, txt } of candidatos) {
    const e = getComputedStyle(el)
    const recorta = ['hidden', 'clip'].includes(e.overflowY)
    if (!recorta) continue
    // `text-overflow: ellipsis` y el recorte por líneas son decisiones: el
    // texto se acorta con sus puntos suspensivos y eso se ve a propósito.
    if (e.textOverflow === 'ellipsis') continue
    if (e.webkitLineClamp && e.webkitLineClamp !== 'none') continue
    if (el.scrollHeight > el.clientHeight + 1) {
      cortados.push({ txt, hace: el.scrollHeight, hay: el.clientHeight })
    }
  }

  // ── 3 · texto que se sale de su contenedor ────────────────────────────
  //
  // ── UN PADRE CON PUNTOS SUSPENSIVOS NO DEJA QUE NADA ASOME, Y ESO DIO
  //    TRES HALLAZGOS FALSOS
  //
  // Esta medida existe para cazar texto que **asoma por encima de lo que
  // sigue**, que es lo que se ve como desorden. Un padre con
  // `text-overflow: ellipsis` no deja asomar nada: corta y pone los puntos.
  // La cabecera de `ATabla` es así, y las tres columnas que no cabían salían
  // aquí como si se desbordaran sobre la columna siguiente. No lo hacían.
  //
  // El defecto que había detrás era real —«WhatsApp principal» se leía
  // «WhatsApp pri…»— pero es **otra** pregunta, y la contesta la medida 4.
  // Mezclarlas habría dejado la 3 gritando de más y la 4 sin existir.
  const salidos = []
  for (const { el, r, txt } of candidatos) {
    const padre = el.parentElement
    if (!padre || padre === document.body) continue
    const p = padre.getBoundingClientRect()
    if (p.width < 1 || p.height < 1) continue
    const e = getComputedStyle(padre)
    // Un padre que deja desplazar no «corta»: lo que asoma se alcanza.
    if (['auto', 'scroll'].includes(e.overflowX)) continue
    if (['auto', 'scroll'].includes(e.overflowY)) continue
    if (e.textOverflow === 'ellipsis') continue

    const derecha = r.right - p.right
    const abajo = r.bottom - p.bottom
    if (derecha > MARGEN_DE_SALIDA || abajo > MARGEN_DE_SALIDA) {
      salidos.push({
        txt,
        derecha: Math.round(Math.max(0, derecha)),
        abajo: Math.round(Math.max(0, abajo)),
        padre: padre.tagName.toLowerCase() + (padre.className ? `.${padre.className}` : ''),
      })
    }
  }

  // ── 4 · ningún rótulo se queda cortado por los puntos suspensivos ─────
  //
  // Los puntos suspensivos son la decisión correcta para **un dato**: una
  // dirección larga se corta y se lee entera al abrir la ficha. No lo son
  // para **un rótulo**: la cabecera de una columna y la etiqueta de un campo
  // son la única pista de qué se está mirando, y recortadas pierden justo el
  // final, que es donde está lo que distingue una de otra.
  //
  // «Correo princip…» y «WhatsApp pri…» era lo que enseñaba Contactos, y
  // ninguna sonda lo miraba porque de ancho **cabe**: el recorte no ensancha
  // nada. Dirección lo vio el 13/10/2026 y lo llamó «desordenado».
  /**
   * ¿Es texto puesto sólo para el lector de pantalla?
   *
   * ── EL TERCER HALLAZGO FALSO DEL DÍA, Y SE COMPROBÓ ANTES DE ANOTARLO
   *
   * La cabecera de acciones de Bajas lleva un `<span class="sr-only">` con la
   * palabra «Acciones»: la columna de botones no necesita rótulo a la vista,
   * pero una cabecera muda deja la tabla sin nombre para quien la oye. El
   * patrón es 1×1 px con `overflow: hidden` y `clip-path`, así que su
   * `scrollWidth` es 60 y su `clientWidth` 1 — exactamente la forma de un
   * rótulo recortado, y **se lee entero** por el único camino que lo usa.
   *
   * No se reconoce por el nombre de la clase, que cada proyecto escribe a su
   * manera, sino por lo que lo define: una caja de menos de cuatro píxeles, o
   * recortada con `clip-path`. Buscar `.sr-only` habría dejado pasar el mismo
   * patrón con otro nombre.
   */
  const esParaElLector = (n) => {
    const r = n.getBoundingClientRect()
    if (r.width <= 4 || r.height <= 4) return true
    const e = getComputedStyle(n)
    return e.clipPath !== 'none' || (e.clip && e.clip !== 'auto')
  }

  const rotulosCortados = []
  const ES_ROTULO = 'th, [role="columnheader"], label, legend, caption, .titulo'
  for (const el of main.querySelectorAll(ES_ROTULO)) {
    if (!el.checkVisibility({ visibilityProperty: true })) continue
    // El rótulo recorta con los puntos, o lo hace el hijo que lleva el texto.
    for (const n of [el, ...el.querySelectorAll('*')]) {
      const e = getComputedStyle(n)
      if (e.textOverflow !== 'ellipsis' && e.overflowX !== 'hidden') continue
      if (esParaElLector(n)) continue
      const sobra = n.scrollWidth - n.clientWidth
      if (sobra > 1) {
        rotulosCortados.push({ txt: texto(el), sobra: Math.round(sobra) })
        break
      }
    }
  }

  // ── 5 · texto estrujado, cayendo de una palabra por línea ─────────────
  //
  // ── POR QUÉ LAS OTRAS CUATRO NO LO VEÍAN (H-07, 16/10/2026) ──
  //
  // Dirección abrió el panel de «Mandar un correo de prueba» y la ficha de la
  // cuenta, al estrecharse, se leyó así:
  //
  //     Cifrado / (STARTTLS, / puerto / 587 / Hasta / 50 / al / día
  //
  // **Cabe perfectamente.** No desborda, no está cortado, no asoma de su
  // padre y no lo recortan unos puntos suspensivos — las cuatro medidas de
  // arriba lo dan por bueno. Está **estrujado**: legible carácter a carácter
  // e ilegible como frase.
  //
  // ── LA PRIMERA VERSIÓN DE ESTA MEDIDA ESTABA MAL PLANTEADA ──
  //
  // Preguntaba si en la caja caben «dos palabras largas juntas», medidas con
  // un `canvas`. Salió en rojo con **diecinueve hallazgos en Bajas**, y al
  // mirar la pantalla la tabla se leía bien: dos o tres palabras por línea.
  // Era el umbral acusando a un texto correcto.
  //
  // El arreglo no fue subirlo hasta que callara —eso es ajustar el
  // instrumento hasta que deje de medir, y esta sonda ya tuvo esa lección en
  // la 8.7— sino **preguntar por el síntoma de verdad**: cuántas líneas ocupa
  // frente a cuántas palabras tiene. Si son casi iguales, cada palabra está
  // cayendo en su propia línea, que es exactamente lo que se vio.
  //
  //     «Cifrado (STARTTLS, puerto 587 Hasta 50 al día»  8 palabras, 8 líneas
  //     «El correo rebotó de forma definitiva»           6 palabras, 3 líneas
  //
  // El primero es el defecto; el segundo es una tabla estrecha y se lee.
  const estrujados = []
  for (const { el, txt } of candidatos) {
    const palabras = txt.split(/\s+/).filter((p) => p.length > 0)
    // Con tres palabras o menos, una por línea puede ser el diseño: un rótulo
    // corto en una columna estrecha se lee bien así.
    if (palabras.length < 4) continue
    // ── LAS LÍNEAS SE CUENTAN CON UN RANGO, NO CON `scrollHeight` ────────
    //
    // Las dos primeras versiones de esto dividían `scrollHeight` entre el alto
    // de línea, y las dos daban cifras falsas:
    //
    //   · `scrollHeight` **incluye el relleno**, así que tres líneas con
    //     12 px arriba y abajo parecían cuatro;
    //   · y en un `<td>` **incluye el alto de la fila entera**, que lo manda
    //     la celda más alta. Descontar el relleno no arreglaba eso.
    //
    // Las dos acusaban a la tabla de Bajas, cuyo «Sin contacto en ARLES» se
    // lee en tres líneas perfectamente. Lo encontró **mirar la celda**, no el
    // número: 96 px de ancho y tres renglones.
    //
    // Un rango sobre el contenido devuelve **un rectángulo por línea
    // pintada**, que es la cifra que se busca y no depende ni del relleno ni
    // de la tabla. Se agrupan por su borde superior porque un texto con algo
    // en línea dentro da varios rectángulos en el mismo renglón.
    const rango = document.createRange()
    rango.selectNodeContents(el)
    const topes = new Set()
    for (const caja of rango.getClientRects()) {
      if (caja.width > 0 || caja.height > 0) topes.add(Math.round(caja.top))
    }
    const lineas = topes.size
    if (lineas === 0) continue
    // ── `palabras.length`, NO `palabras` ─────────────────────────────────
    //
    // Aquí decía `lineas >= palabras * 0.8`, y `palabras` es un **array**.
    // Multiplicar un array por un número da `NaN`, y `6 >= NaN` es `false`:
    // **esta medida no podía saltar nunca**. Habría vivido en verde para
    // siempre sin mirar un solo texto.
    //
    // No lo encontró leer el código: lo encontró **romper la pantalla a
    // propósito** y ver que la sonda seguía en verde. Es el §2 en su forma
    // literal — una comprobación que nunca se ha visto fallar no está
    // verificada — y la razón de que cada medida de esta sonda tenga su
    // ruptura escrita.
    //
    // El 0.8: con seis palabras en seis líneas da 1.0 y salta; con seis en
    // tres da 0.5 y no. El hueco entre los dos casos medidos es ancho, así
    // que el umbral no está pegado a ninguno.
    if (lineas >= palabras.length * 0.8) {
      estrujados.push({ txt, lineas, palabras: palabras.length })
    }
  }

  return {
    medidos: candidatos.length,
    encimados,
    cortados,
    salidos,
    rotulosCortados,
    estrujados,
  }
}

const ejecutable = await exigirChromium()

console.log('Generando la vista previa…')
python([path.join(RAIZ, 'herramientas/vista-previa/generar.py')])

const navegador = await chromium.launch({ executablePath: ejecutable })

let totalMedidos = 0

for (const [ancho, alto] of VENTANAS) {
  const pagina = await navegador.newPage({ viewport: { width: ancho, height: alto } })

  for (const ruta of RUTAS) {
    await pagina.goto(`file://${ARCHIVO}${ruta}`, { waitUntil: 'load' })
    await pagina.waitForTimeout(700)

    const r = await pagina.evaluate(MEDIR, [MINIMO_PX, PARTE_TAPADA, MARGEN_DE_SALIDA])
    const donde = `${ruta} @ ${ancho}×${alto}`
    totalMedidos += r.medidos

    for (const e of r.encimados) {
      mal(`${donde} · encimado ${e.cruce}: «${e.a}» sobre «${e.b}»`)
    }
    for (const c of r.cortados) {
      mal(`${donde} · texto cortado: «${c.txt}» necesita ${c.hace} px y tiene ${c.hay}`)
    }
    for (const s of r.salidos) {
      const por = [s.derecha > 0 ? `${s.derecha} px por la derecha` : null,
        s.abajo > 0 ? `${s.abajo} px por abajo` : null].filter(Boolean).join(' y ')
      mal(`${donde} · «${s.txt}» se sale ${por} de ${s.padre}`)
    }
    for (const c of r.rotulosCortados) {
      mal(`${donde} · rótulo cortado: «${c.txt}» necesita ${c.sobra} px más`)
    }
    for (const e of r.estrujados) {
      mal(
        `${donde} · texto estrujado: «${e.txt}» cae en ${e.lineas} líneas` +
          ` teniendo ${e.palabras} palabras — una por renglón`,
      )
    }
  }

  await pagina.close()
  console.log(`✓ ${RUTAS.length} pantallas medidas a ${ancho}×${alto}.`)
}

await navegador.close()

/**
 * La sonda no puede terminar en verde sin haber medido nada.
 *
 * Si un selector deja de casar —porque el armazón cambia de `main` a otra
 * etiqueta, por ejemplo— `candidatos` queda vacío en las dieciocho pantallas
 * y la sonda sale en verde sin mirar un píxel. Es la trampa del §2 que este
 * proyecto ha tenido seis veces, y cuesta tres líneas cerrarla.
 */
const MINIMO_DE_TEXTOS = 200
if (totalMedidos < MINIMO_DE_TEXTOS) {
  console.error(
    `\n✗ sólo se midieron ${totalMedidos} textos en ${RUTAS.length * VENTANAS.length}` +
      ` pantallas, y hacen falta ${MINIMO_DE_TEXTOS}. La sonda no está mirando` +
      ' lo que cree: revisa el selector de `main` y `tieneTextoPropio`.',
  )
  process.exit(1)
}

if (fallos > 0) {
  console.error(`\n${fallos} hallazgo(s) de maquetación.`)
  process.exit(1)
}
console.log(`\nSin hallazgos. ${totalMedidos} textos medidos.`)
