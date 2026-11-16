#!/usr/bin/env python3
"""Genera el PDF de un informe de fase para Dirección a partir de su Markdown.

Versionado a propósito: el PDF no se edita a mano. Se edita el .md y se
regenera, igual que la lámina de la paleta se regenera desde tokens.json.

    python3 herramientas/informe-direccion/generar-pdf.py \
        documentacion/09-fases/FASE-01-PARA-DIRECCION.md

Tipografía: Helvetica, no Mont. La licencia de Mont (P-01) no cubre incrustar
la fuente en artefactos distribuidos, y un PDF que sale de la empresa lo es.
"""

from __future__ import annotations

import argparse
import hashlib
import html
import re
import sys
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    HRFlowable,
    Image,
    KeepTogether,
    ListFlowable,
    ListItem,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)

# Paleta de marca (herramientas/design-tokens/tokens.json), adaptada a papel:
# fondo claro, tinta profunda. El oro sólo como filete de acento.
PROFUNDO = colors.HexColor("#041A25")
SUPERFICIE = colors.HexColor("#053048")
BORDE = colors.HexColor("#CBD8DF")
ACENTO = colors.HexColor("#B88A00")  # oro oscurecido: sobre papel blanco el
# #FCCC0C original no alcanza contraste AA
TINTA = colors.HexColor("#1A2A33")
TENUE = colors.HexColor("#4A5B66")
CEBRA = colors.HexColor("#F2F5F7")

MARGEN = 20 * mm


# ─────────────────────────────── inline ────────────────────────────────


# Helvetica no tiene emoji, y reportlab pinta un cuadrado negro por cada uno.
# En pantalla ordenan; en papel son ruido y encima parecen un defecto de
# impresión. Se quitan, y con ellos el espacio que arrastran.
RE_EMOJI = re.compile(
    "[\U0001F000-\U0001FAFF\U00002190-\U000021FF\U00002300-\U000027BF"
    "\U00002B00-\U00002BFF\U0000FE00-\U0000FE0F\U0001F1E6-\U0001F1FF]+"
)


def sin_emoji(texto: str) -> str:
    return re.sub(r"\s{2,}", " ", RE_EMOJI.sub("", texto)).strip()


def inline(texto: str) -> str:
    """Convierte el Markdown de línea al mini-HTML que entiende reportlab."""
    texto = sin_emoji(texto)
    # El negrita se resuelve DENTRO de cada trozo de texto, así que un
    # `**`código`**` dejaba los asteriscos a la vista: los marcadores caen a un
    # lado y otro del trozo entre acentos graves, y la expresión no cruza de
    # uno a otro. El negrita ahí sobra —el código ya va en otra fuente y otro
    # color—, así que se quita el marcador en vez de intentar respetarlo.
    texto = re.sub(r"\*\*(`[^`]+`)\*\*", r"\1", texto)
    partes: list[str] = []
    for i, trozo in enumerate(re.split(r"(`[^`]+`)", texto)):
        if i % 2:  # dentro de acentos graves: literal, sin más marcado
            partes.append(
                f'<font face="Courier" size="9" color="#053048">'
                f"{html.escape(trozo[1:-1])}</font>"
            )
            continue
        t = html.escape(trozo)
        t = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", t)  # enlaces: sólo el texto
        t = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", t)
        t = re.sub(r"(?<!\*)\*([^*]+?)\*(?!\*)", r"<i>\1</i>", t)
        partes.append(t)
    return "".join(partes)


# ─────────────────────────────── estilos ───────────────────────────────


def estilos() -> dict[str, ParagraphStyle]:
    base = getSampleStyleSheet()["Normal"]
    comun = dict(fontName="Helvetica", fontSize=9.8, leading=14.6, textColor=TINTA)
    return {
        "titulo": ParagraphStyle(
            "titulo", base, fontName="Helvetica-Bold", fontSize=23, leading=28,
            textColor=PROFUNDO, spaceAfter=4,
        ),
        "subtitulo": ParagraphStyle(
            "subtitulo", base, fontName="Helvetica", fontSize=10, leading=15,
            textColor=TENUE, spaceAfter=16,
        ),
        "h2": ParagraphStyle(
            "h2", base, fontName="Helvetica-Bold", fontSize=14, leading=18,
            textColor=PROFUNDO, spaceBefore=20, spaceAfter=7,
        ),
        "h3": ParagraphStyle(
            "h3", base, fontName="Helvetica-Bold", fontSize=11, leading=15,
            textColor=SUPERFICIE, spaceBefore=13, spaceAfter=4,
        ),
        "p": ParagraphStyle("p", base, **comun, alignment=TA_JUSTIFY, spaceAfter=8),
        "cita": ParagraphStyle(
            "cita", base, fontName="Helvetica-Oblique", fontSize=9.2, leading=13.6,
            textColor=TENUE, leftIndent=9, borderPadding=0, spaceAfter=12,
        ),
        "codigo": ParagraphStyle(
            "codigo", base, fontName="Courier", fontSize=9, leading=13,
            textColor=PROFUNDO, leftIndent=8, spaceBefore=4, spaceAfter=10,
        ),
        "li": ParagraphStyle("li", base, **comun, spaceAfter=4),
        "celda": ParagraphStyle(
            "celda", base, fontName="Helvetica", fontSize=8.6, leading=12.2,
            textColor=TINTA,
        ),
        "celda_cab": ParagraphStyle(
            "celda_cab", base, fontName="Helvetica-Bold", fontSize=8.6, leading=12.2,
            textColor=colors.white,
        ),
        "pie": ParagraphStyle(
            "pie", base, fontName="Helvetica", fontSize=7.6, textColor=TENUE,
        ),
        # El pie de una captura. Centrado y en cursiva para que no se confunda
        # con el párrafo que viene después.
        "pie_figura": ParagraphStyle(
            "pie_figura", base, fontName="Helvetica-Oblique", fontSize=8.2,
            leading=11.5, textColor=TENUE, alignment=TA_CENTER,
        ),
    }


# ──────────────────────────────── tablas ───────────────────────────────


def construir_tabla(filas: list[list[str]], est, ancho: float) -> Table:
    # Una tabla escrita como `| | |` no tiene cabecera: es una rejilla de
    # etiqueta y valor. Pintarle una barra azul con la primera fila de datos
    # dentro convierte un dato en un título de columna, que es mentira.
    con_cabecera = any(c.strip() for c in filas[0])

    if con_cabecera:
        cab, *cuerpo = filas
        datos = [[Paragraph(inline(c), est["celda_cab"]) for c in cab]]
    else:
        cab, cuerpo = filas[0], filas[1:]
        datos = []
    datos += [[Paragraph(inline(c), est["celda"]) for c in fila] for fila in cuerpo]

    n = len(cab)
    # La primera columna suele ser la etiqueta: se le da menos aire que al resto.
    if n == 1:
        pesos = [1.0]
    elif n == 2:
        pesos = [0.34, 0.66]
    else:
        pesos = [0.30] + [0.70 / (n - 1)] * (n - 1)
    anchos = [ancho * p for p in pesos]

    t = Table(datos, colWidths=anchos, repeatRows=1 if con_cabecera else 0,
              hAlign="LEFT")
    estilo = [
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("GRID", (0, 0), (-1, -1), 0.4, BORDE),
        ("TOPPADDING", (0, 0), (-1, -1), 5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
        ("LEFTPADDING", (0, 0), (-1, -1), 7),
        ("RIGHTPADDING", (0, 0), (-1, -1), 7),
    ]
    if con_cabecera:
        estilo.append(("BACKGROUND", (0, 0), (-1, 0), SUPERFICIE))
        estilo.append(("LINEBELOW", (0, 0), (-1, 0), 1.1, ACENTO))
    # El cebreado cuenta desde la primera fila de DATOS, tenga cabecera o no.
    primera = 1 if con_cabecera else 0
    for i in range(primera, len(datos)):
        if (i - primera) % 2 == 1:
            estilo.append(("BACKGROUND", (0, i), (-1, i), CEBRA))
    t.setStyle(TableStyle(estilo))
    return t


# ─────────────────────────────── parseo ────────────────────────────────


def es_separador(fila: list[str]) -> bool:
    """La fila de guiones que separa la cabecera del cuerpo.

    El `any` no es decorativo: sin él, una fila **enteramente vacía** —`| | |`,
    que es como se escribe una tabla sin cabecera— pasaba por separador,
    porque `all()` sobre cero elementos es cierto. La consecuencia era que la
    primera fila de datos ascendía a cabecera y se pintaba en azul oscuro:
    «Windows 10 u 11» salía como si fuera un título de columna.
    """
    celdas_con_texto = [c.strip() for c in fila if c.strip()]
    return bool(celdas_con_texto) and all(
        re.fullmatch(r":?-{2,}:?", c) for c in celdas_con_texto
    )


def celdas(linea: str) -> list[str]:
    return [c.strip() for c in linea.strip().strip("|").split("|")]


RE_IMAGEN = re.compile(r"^!\[([^\]]*)\]\(([^)]+)\)$")


def imagen(alt: str, ruta: Path, ancho: float, est) -> list:
    """Una captura, a lo ancho de la caja de texto y con su pie.

    Las referencias se capturaron a 2240 px de ancho (2× de 1120). Aquí se
    reducen a la caja del PDF conservando la proporción: en papel se ven con
    el doble de densidad que el punto tipográfico, que es lo que hace que se
    puedan ampliar en pantalla sin que se deshagan.
    """
    from reportlab.lib.utils import ImageReader

    ancho_px, alto_px = ImageReader(str(ruta)).getSize()
    # 78 % de la caja, centrada. Medido: a lo ancho completo el manual sale a
    # 12 páginas y a 78 % a 11, así que el ahorro es modesto — casi siempre
    # cabe una captura por página y el resto queda en blanco, porque una de
    # 1120×720 ocupa un tercio largo de un A4 y el título de su sección va
    # pegado a ella.
    #
    # Se deja así por legibilidad, no por ahorrar papel: la captura sigue
    # entrando entera y a 2240 px en 133 mm son 428 ppp, de sobra para ampliar
    # en pantalla sin que se deshaga.
    util = ancho * 0.78
    alto = util * alto_px / ancho_px
    figura = Image(str(ruta), width=util, height=alto)
    figura.hAlign = "CENTER"
    partes = [figura]
    if alt:
        partes.append(Spacer(1, 3))
        partes.append(Paragraph(inline(alt), est["pie_figura"]))
    partes.append(Spacer(1, 11))
    # KeepTogether para que el pie no se quede huérfano en la página siguiente.
    return [KeepTogether(partes)]


def convertir(md: str, est, ancho: float, base: Path) -> list:
    lineas = md.splitlines()
    flujo: list = []
    i = 0
    primer_h1 = True

    while i < len(lineas):
        linea = lineas[i]
        desnuda = linea.strip()

        if not desnuda:
            i += 1
            continue

        # Regla horizontal
        if re.fullmatch(r"-{3,}", desnuda):
            flujo.append(Spacer(1, 4))
            flujo.append(HRFlowable(width="100%", thickness=0.6, color=BORDE))
            flujo.append(Spacer(1, 6))
            i += 1
            continue

        # Bloque de código
        if desnuda.startswith("```"):
            i += 1
            cuerpo = []
            while i < len(lineas) and not lineas[i].strip().startswith("```"):
                cuerpo.append(html.escape(lineas[i]))
                i += 1
            i += 1
            texto = "<br/>".join(c.replace(" ", "&nbsp;") for c in cuerpo)
            flujo.append(Paragraph(texto, est["codigo"]))
            continue

        # Imagen
        m_img = RE_IMAGEN.match(desnuda)
        if m_img:
            alt, ruta = m_img.group(1), m_img.group(2)
            archivo = (base / ruta).resolve()
            if archivo.is_file():
                flujo.extend(imagen(alt, archivo, ancho, est))
            else:
                # Una imagen que falta no se traga en silencio: en un manual
                # cuyo valor son las capturas, el hueco tiene que verse.
                flujo.append(Paragraph(
                    f"[falta la imagen: {html.escape(ruta)}]", est["pie_figura"]))
            i += 1
            continue

        # Tabla
        if desnuda.startswith("|"):
            filas = []
            while i < len(lineas) and lineas[i].strip().startswith("|"):
                fila = celdas(lineas[i])
                if not es_separador(fila):
                    filas.append(fila)
                i += 1
            if filas:
                ancho_max = max(len(f) for f in filas)
                filas = [f + [""] * (ancho_max - len(f)) for f in filas]
                flujo.append(Spacer(1, 3))
                flujo.append(construir_tabla(filas, est, ancho))
                flujo.append(Spacer(1, 11))
            continue

        # Cita
        if desnuda.startswith(">"):
            cuerpo = []
            while i < len(lineas) and lineas[i].strip().startswith(">"):
                cuerpo.append(lineas[i].strip().lstrip(">").strip())
                i += 1
            flujo.append(Paragraph(inline(" ".join(cuerpo)), est["cita"]))
            continue

        # Listas
        if re.match(r"[-*] ", desnuda) or re.match(r"\d+\. ", desnuda):
            ordenada = bool(re.match(r"\d+\. ", desnuda))
            puntos = []
            while i < len(lineas):
                actual = lineas[i].strip()
                if re.match(r"[-*] ", actual) or re.match(r"\d+\. ", actual):
                    texto = re.sub(r"^(?:[-*]|\d+\.)\s+", "", actual)
                    i += 1
                    # continuación indentada
                    while i < len(lineas) and lineas[i].startswith("  ") and lineas[i].strip():
                        texto += " " + lineas[i].strip()
                        i += 1
                    puntos.append(ListItem(Paragraph(inline(texto), est["li"]), leftIndent=14))
                elif not actual:
                    if i + 1 < len(lineas) and re.match(
                        r"(?:[-*]|\d+\.) ", lineas[i + 1].strip()
                    ):
                        i += 1
                        continue
                    break
                else:
                    break
            flujo.append(
                ListFlowable(
                    puntos,
                    bulletType="1" if ordenada else "bullet",
                    bulletFontSize=8,
                    bulletColor=ACENTO,
                    leftIndent=14,
                    start="1" if ordenada else None,
                )
            )
            flujo.append(Spacer(1, 7))
            continue

        # Encabezados
        m = re.match(r"(#{1,4})\s+(.*)", desnuda)
        if m:
            nivel, texto = len(m.group(1)), m.group(2)
            if nivel == 1 and primer_h1:
                primer_h1 = False
                flujo.append(Paragraph(inline(texto), est["titulo"]))
                flujo.append(HRFlowable(width=62 * mm, thickness=2.4, color=ACENTO,
                                        spaceBefore=2, spaceAfter=9))
            elif nivel <= 2:
                flujo.append(Paragraph(inline(texto), est["h2"]))
            else:
                flujo.append(Paragraph(inline(texto), est["h3"]))
            i += 1
            continue

        # Párrafo
        cuerpo = []
        while i < len(lineas) and lineas[i].strip() and not re.match(
            r"(#{1,4}\s|\||>|```|-{3,}$|[-*] |\d+\. )", lineas[i].strip()
        ):
            cuerpo.append(lineas[i].strip())
            i += 1
        flujo.append(Paragraph(inline(" ".join(cuerpo)), est["p"]))

    return flujo


# ──────────────────────────────── página ───────────────────────────────


def pie_de_pagina(canvas, doc, etiqueta: str) -> None:
    canvas.saveState()
    y = MARGEN - 7 * mm
    canvas.setStrokeColor(BORDE)
    canvas.setLineWidth(0.5)
    canvas.line(MARGEN, y + 4 * mm, A4[0] - MARGEN, y + 4 * mm)
    canvas.setFont("Helvetica", 7.6)
    canvas.setFillColor(TENUE)
    canvas.drawString(MARGEN, y, etiqueta)
    canvas.drawRightString(A4[0] - MARGEN, y, f"Página {doc.page}")
    canvas.restoreState()


def generar(origen: Path, destino: Path) -> None:
    md = origen.read_text(encoding="utf-8")
    est = estilos()

    titulo = next(
        (l[2:].strip() for l in md.splitlines() if l.startswith("# ")), origen.stem
    )
    etiqueta = "ARLES RELAY I · TELEMETRY INSIGHT · documento interno"

    doc = BaseDocTemplate(
        str(destino),
        pagesize=A4,
        leftMargin=MARGEN,
        rightMargin=MARGEN,
        topMargin=MARGEN,
        bottomMargin=MARGEN + 4 * mm,
        title=titulo,
        author="TELEMETRY INSIGHT",
        subject="Informe de fase para Dirección",
    )
    marco = Frame(
        doc.leftMargin, doc.bottomMargin, doc.width, doc.height, id="cuerpo"
    )
    doc.addPageTemplates(
        PageTemplate(
            id="normal",
            frames=[marco],
            onPage=lambda c, d: pie_de_pagina(c, d, etiqueta),
        )
    )

    flujo = convertir(md, est, doc.width, origen.parent)

    # Evitar que un encabezado quede solo al pie de una página.
    agrupado: list = []
    k = 0
    while k < len(flujo):
        f = flujo[k]
        es_h = isinstance(f, Paragraph) and f.style.name in ("h2", "h3")
        if es_h and k + 1 < len(flujo):
            agrupado.append(KeepTogether([f, flujo[k + 1]]))
            k += 2
        else:
            agrupado.append(f)
            k += 1

    doc.build(agrupado)


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("origen", type=Path, help="Markdown de entrada")
    p.add_argument("-o", "--salida", type=Path, help="PDF de salida")
    args = p.parse_args()

    if not args.origen.exists():
        print(f"No existe: {args.origen}", file=sys.stderr)
        return 1

    destino = args.salida or args.origen.with_suffix(".pdf")
    destino.parent.mkdir(parents=True, exist_ok=True)
    generar(args.origen, destino)

    # Huella del Markdown de origen. El validador la compara con el .md actual:
    # es lo que delata un PDF que quedó atrás respecto a su fuente.
    huella = hashlib.sha256(args.origen.read_bytes()).hexdigest()
    destino.with_suffix(".pdf.sha256").write_text(
        f"{huella}  {args.origen.name}\n", encoding="utf-8"
    )

    print(f"✓ {destino} ({destino.stat().st_size // 1024} KB)")
    print(f"✓ {destino.with_suffix('.pdf.sha256').name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
