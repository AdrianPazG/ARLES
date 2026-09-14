/**
 * Convierte una plantilla de entrega (.md) en un .docx que se rellena escribiendo.
 *
 *     npm install docx          (una vez, en esta carpeta)
 *     node herramientas/plantilla-docx/generar-docx.js \
 *          documentacion/06-calidad/PLANTILLA-WINDOWS.md
 *
 * Por qué parte del Markdown y no de una estructura escrita aquí: los nombres
 * de las quince capturas ya viven en dos sitios —el manual y la plantilla— y
 * una comprobación los compara. Meter un tercero en este script los pondría a
 * divergir sin que nadie lo notara. El .md manda; el .docx se regenera.
 *
 * Lo que cambia respecto al .md, y por qué:
 *   `[ ]`   → ☐   una casilla de verdad. Para marcarla se escribe una x al
 *                 lado o se sustituye por ☒: en Word no hace falta nada más.
 *   `_____` → una línea con borde inferior, del ancho de la caja, para
 *                 escribir encima. Cinco guiones bajos en Word se quedan en
 *                 cinco guiones bajos, que no invitan a escribir.
 *   <!-- -->→ nota gris en cursiva. Son instrucciones para quien rellena, no
 *                 comentarios de código: en el .md van ocultas por el formato,
 *                 pero en Word tienen que verse.
 */
const fs = require('node:fs')
const path = require('node:path')
const crypto = require('node:crypto')
const {
  AlignmentType,
  BorderStyle,
  Document,
  HeadingLevel,
  Packer,
  Paragraph,
  ShadingType,
  Table,
  TableCell,
  TableRow,
  TextRun,
  WidthType,
} = require('docx')

// A4 menos 2 cm por lado, en DXA (1440 = una pulgada).
const ANCHO = 9638
const TINTA = '1A2A33'
const PROFUNDO = '041A25'
const TENUE = '4A5B66'
const ACENTO = 'B88A00'
const CEBRA = 'F2F5F7'
const CABECERA = '053048'

// Helvetica no está en todos los Windows; Calibri sí, y es la de Word.
const FUENTE = 'Calibri'

// ── Markdown de línea → runs ────────────────────────────────────────────
//
// Se resuelven negrita, cursiva y código. Los enlaces se quedan con su texto
// y sueltan la URL: en un papel que se rellena a mano, una URL entre
// paréntesis es ruido.
function runs(texto, base = {}) {
  const limpio = texto
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/\[ \]/g, '☐')
  const salida = []
  // Se parte por los tres marcadores a la vez para no anidarlos mal.
  const trozos = limpio.split(/(`[^`]+`|\*\*[^*]+\*\*|(?<!\*)\*[^*]+\*(?!\*))/g)
  for (const trozo of trozos) {
    if (!trozo) continue
    if (trozo.startsWith('`') && trozo.endsWith('`')) {
      salida.push(new TextRun({ ...base, text: trozo.slice(1, -1), font: 'Consolas' }))
    } else if (trozo.startsWith('**') && trozo.endsWith('**')) {
      salida.push(new TextRun({ ...base, text: trozo.slice(2, -2), bold: true }))
    } else if (trozo.startsWith('*') && trozo.endsWith('*') && trozo.length > 2) {
      salida.push(new TextRun({ ...base, text: trozo.slice(1, -1), italics: true }))
    } else {
      salida.push(new TextRun({ ...base, text: trozo }))
    }
  }
  return salida.length ? salida : [new TextRun({ ...base, text: '' })]
}

const parrafo = (texto, extra = {}) =>
  new Paragraph({
    children: runs(texto, { font: FUENTE, size: 21, color: TINTA }),
    spacing: { after: 120 },
    ...extra,
  })

// La línea sobre la que se escribe. Un párrafo vacío con borde inferior deja
// un renglón real, no cinco guiones bajos que nadie sabe si puede tocar.
const renglon = () =>
  new Paragraph({
    children: [new TextRun({ text: ' ', font: FUENTE, size: 21 })],
    spacing: { before: 60, after: 220 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 6, color: 'A9B6BF' } },
  })

const regla = () =>
  new Paragraph({
    children: [new TextRun({ text: '' })],
    spacing: { before: 120, after: 200 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 6, color: 'CBD8DF' } },
  })

const nota = (lineas) =>
  new Paragraph({
    children: lineas.flatMap((l, i) =>
      runs(l, { font: FUENTE, size: 18, color: TENUE, italics: true }).concat(
        i < lineas.length - 1 ? [new TextRun({ break: 1 })] : [],
      ),
    ),
    spacing: { before: 60, after: 200 },
    shading: { type: ShadingType.CLEAR, fill: CEBRA },
    border: { left: { style: BorderStyle.SINGLE, size: 12, color: ACENTO } },
    indent: { left: 140 },
  })

// ── Tablas ──────────────────────────────────────────────────────────────

function celdas(linea) {
  return linea.trim().replace(/^\||\|$/g, '').split('|').map((c) => c.trim())
}

const esSeparador = (fila) => {
  const conTexto = fila.filter((c) => c)
  return conTexto.length > 0 && conTexto.every((c) => /^:?-{2,}:?$/.test(c))
}

function tabla(filas) {
  const conCabecera = filas[0].some((c) => c)
  const cuerpo = conCabecera ? filas.slice(1) : filas.slice(1)
  const n = filas[0].length

  // Una columna de casillas no necesita más que el ancho de la casilla.
  const soloCasilla = (j) =>
    cuerpo.length > 0 && cuerpo.every((f) => (f[j] || '').replace(/☐|\[ \]/g, '').trim() === '')
  const pesos = []
  for (let j = 0; j < n; j++) pesos.push(soloCasilla(j) ? 0.12 : 1)
  const suma = pesos.reduce((a, b) => a + b, 0)
  const anchos = pesos.map((p) => Math.round((ANCHO * p) / suma))
  anchos[0] += ANCHO - anchos.reduce((a, b) => a + b, 0) // cuadrar el redondeo

  const fila = (valores, opciones) =>
    new TableRow({
      tableHeader: opciones.cabecera,
      children: valores.map((v, j) => {
        const casilla = v.replace(/☐|\[ \]/g, '').trim() === '' && /\[ \]|☐/.test(v)
        return new TableCell({
          width: { size: anchos[j], type: WidthType.DXA },
          shading: {
            type: ShadingType.CLEAR,
            fill: opciones.cabecera ? CABECERA : opciones.cebra ? CEBRA : 'FFFFFF',
          },
          margins: { top: 70, bottom: 70, left: 110, right: 110 },
          children: [
            new Paragraph({
              alignment: casilla ? AlignmentType.CENTER : AlignmentType.LEFT,
              children: runs(v, {
                font: FUENTE,
                size: casilla ? 26 : 19,
                color: opciones.cabecera ? 'FFFFFF' : TINTA,
                bold: opciones.cabecera,
              }),
            }),
          ],
        })
      }),
    })

  const filasDoc = []
  if (conCabecera) filasDoc.push(fila(filas[0], { cabecera: true }))
  cuerpo.forEach((f, i) => {
    const completa = [...f, ...Array(Math.max(0, n - f.length)).fill('')]
    filasDoc.push(fila(completa, { cebra: i % 2 === 1 }))
  })

  return new Table({
    columnWidths: anchos,
    width: { size: ANCHO, type: WidthType.DXA },
    rows: filasDoc,
  })
}

// ── Conversión ──────────────────────────────────────────────────────────

function convertir(md) {
  const lineas = md.split('\n')
  const hijos = []
  let i = 0
  let primerH1 = true

  while (i < lineas.length) {
    const cruda = lineas[i]
    const linea = cruda.trim()

    if (!linea) { i++; continue }

    // Comentario: instrucciones para quien rellena.
    if (linea.startsWith('<!--')) {
      const cuerpo = []
      while (i < lineas.length && !lineas[i].includes('-->')) {
        const t = lineas[i].replace('<!--', '').trim()
        if (t) cuerpo.push(t)
        i++
      }
      const ultima = (lineas[i] || '').replace('-->', '').replace('<!--', '').trim()
      if (ultima) cuerpo.push(ultima)
      i++
      if (cuerpo.length) hijos.push(nota(cuerpo))
      continue
    }

    if (/^-{3,}$/.test(linea)) { hijos.push(regla()); i++; continue }

    // Bloque de código: la lista de capturas que se marcan.
    if (linea.startsWith('```')) {
      i++
      while (i < lineas.length && !lineas[i].trim().startsWith('```')) {
        const t = lineas[i].replace(/\[ \]/g, '☐')
        hijos.push(new Paragraph({
          children: [new TextRun({ text: t || ' ', font: 'Consolas', size: 19, color: TINTA })],
          spacing: { after: 20 },
        }))
        i++
      }
      i++
      hijos.push(new Paragraph({ children: [new TextRun({ text: '' })], spacing: { after: 160 } }))
      continue
    }

    // Tabla
    if (linea.startsWith('|')) {
      const filas = []
      while (i < lineas.length && lineas[i].trim().startsWith('|')) {
        const f = celdas(lineas[i])
        if (!esSeparador(f)) filas.push(f)
        i++
      }
      if (filas.length) {
        hijos.push(tabla(filas))
        hijos.push(new Paragraph({ children: [new TextRun({ text: '' })], spacing: { after: 200 } }))
      }
      continue
    }

    // El hueco para escribir
    if (/^_{3,}$/.test(linea)) { hijos.push(renglon()); i++; continue }

    // Encabezados
    const m = linea.match(/^(#{1,4})\s+(.*)$/)
    if (m) {
      const nivel = m[1].length
      const texto = m[2]
      if (nivel === 1 && primerH1) {
        primerH1 = false
        hijos.push(new Paragraph({
          children: runs(texto, { font: FUENTE, size: 40, bold: true, color: PROFUNDO }),
          spacing: { after: 60 },
          border: { bottom: { style: BorderStyle.SINGLE, size: 18, color: ACENTO } },
        }))
      } else {
        hijos.push(new Paragraph({
          children: runs(texto, {
            font: FUENTE,
            size: nivel <= 2 ? 28 : 23,
            bold: true,
            color: nivel <= 2 ? PROFUNDO : CABECERA,
          }),
          heading: nivel <= 2 ? HeadingLevel.HEADING_1 : HeadingLevel.HEADING_2,
          spacing: { before: nivel <= 2 ? 320 : 220, after: 120 },
        }))
      }
      i++
      continue
    }

    // Lista
    if (/^[-*] /.test(linea) || /^\d+\. /.test(linea)) {
      const ordenada = /^\d+\. /.test(linea)
      let n = 1
      while (i < lineas.length) {
        const actual = lineas[i].trim()
        if (!/^[-*] /.test(actual) && !/^\d+\. /.test(actual)) break
        const texto = actual.replace(/^(?:[-*]|\d+\.)\s+/, '')
        // Una entrada que es sólo un hueco se convierte en renglón numerado.
        if (/^_{3,}$/.test(texto)) {
          hijos.push(new Paragraph({
            children: [new TextRun({ text: `${n}.  `, font: FUENTE, size: 21, color: TENUE })],
            spacing: { before: 60, after: 220 },
            indent: { left: 280 },
            border: { bottom: { style: BorderStyle.SINGLE, size: 6, color: 'A9B6BF' } },
          }))
        } else {
          hijos.push(new Paragraph({
            children: runs(`${ordenada ? n + '.  ' : '·  '}${texto}`,
              { font: FUENTE, size: 21, color: TINTA }),
            spacing: { after: 90 },
            indent: { left: 280 },
          }))
        }
        n++
        i++
      }
      hijos.push(new Paragraph({ children: [new TextRun({ text: '' })], spacing: { after: 80 } }))
      continue
    }

    // Párrafo: se junta con sus continuaciones, y si acaba en un hueco,
    // el hueco se convierte en renglón.
    const cuerpo = []
    while (i < lineas.length) {
      const t = lineas[i].trim()
      if (!t || /^(#{1,4}\s|\||>|```|-{3,}$|[-*] |\d+\. |_{3,}$|<!--)/.test(t)) break
      cuerpo.push(t)
      i++
    }
    if (cuerpo.length) hijos.push(parrafo(cuerpo.join(' ')))
  }

  return hijos
}

// ── Entrada ─────────────────────────────────────────────────────────────

async function main() {
  const origen = process.argv[2]
  if (!origen) {
    console.error('uso: generar-docx.js <plantilla.md> [-o salida.docx]')
    process.exit(2)
  }
  const salidaIdx = process.argv.indexOf('-o')
  const destino = salidaIdx > -1
    ? process.argv[salidaIdx + 1]
    : origen.replace(/\.md$/, '.docx')

  const md = fs.readFileSync(origen, 'utf8')
  const titulo = (md.match(/^#\s+(.*)$/m) || [, path.basename(origen)])[1]
    .replace(/\*\*/g, '')

  const doc = new Document({
    creator: 'TELEMETRY INSIGHT',
    title: titulo,
    description: 'Plantilla de entrega de la revisión visual de ARLES RELAY',
    sections: [{
      properties: {
        page: { margin: { top: 1134, right: 1134, bottom: 1134, left: 1134 } },
      },
      children: convertir(md),
    }],
  })

  fs.writeFileSync(destino, await Packer.toBuffer(doc))
  const kb = Math.round(fs.statSync(destino).size / 1024)
  console.log(`✓ ${destino} (${kb} KB)`)

  // La huella del ORIGEN, no del .docx: es lo que delata un .docx que se quedó
  // atrás de su Markdown. El .docx no es reproducible byte a byte —lleva
  // identificadores y fechas—, así que su propia huella no serviría de nada.
  const huella = crypto.createHash('sha256').update(fs.readFileSync(origen)).digest('hex')
  const marca = destino.replace(/\.docx$/, '.docx.sha256')
  fs.writeFileSync(marca, `${huella}  ${path.basename(origen)}\n`)
  console.log(`✓ ${path.basename(marca)}`)
}

main().catch((e) => { console.error(e); process.exit(1) })
