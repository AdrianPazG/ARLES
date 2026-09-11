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
from reportlab.lib.enums import TA_JUSTIFY
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    HRFlowable,
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


def inline(texto: str) -> str:
    """Convierte el Markdown de línea al mini-HTML que entiende reportlab."""
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
    }


# ──────────────────────────────── tablas ───────────────────────────────


def construir_tabla(filas: list[list[str]], est, ancho: float) -> Table:
    cab, *cuerpo = filas
    datos = [[Paragraph(inline(c), est["celda_cab"]) for c in cab]]
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

    t = Table(datos, colWidths=anchos, repeatRows=1, hAlign="LEFT")
    estilo = [
        ("BACKGROUND", (0, 0), (-1, 0), SUPERFICIE),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("GRID", (0, 0), (-1, -1), 0.4, BORDE),
        ("LINEBELOW", (0, 0), (-1, 0), 1.1, ACENTO),
        ("TOPPADDING", (0, 0), (-1, -1), 5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
        ("LEFTPADDING", (0, 0), (-1, -1), 7),
        ("RIGHTPADDING", (0, 0), (-1, -1), 7),
    ]
    for i in range(1, len(datos)):
        if i % 2 == 0:
            estilo.append(("BACKGROUND", (0, i), (-1, i), CEBRA))
    t.setStyle(TableStyle(estilo))
    return t


# ─────────────────────────────── parseo ────────────────────────────────


def es_separador(fila: list[str]) -> bool:
    return all(re.fullmatch(r":?-{2,}:?", c.strip()) for c in fila if c.strip())


def celdas(linea: str) -> list[str]:
    return [c.strip() for c in linea.strip().strip("|").split("|")]


def convertir(md: str, est, ancho: float) -> list:
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

    flujo = convertir(md, est, doc.width)

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
