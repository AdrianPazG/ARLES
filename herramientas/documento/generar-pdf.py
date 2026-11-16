#!/usr/bin/env python3
"""Genera un PDF corporativo de ARLES a partir de su Markdown.

    python3 herramientas/documento/generar-pdf.py \
        documentacion/01-producto/WHATSAPP-PARA-DIRECCION.md

El PDF **no se edita a mano**: se edita el `.md` y se regenera. Igual que la
lámina de la paleta se regenera desde `tokens.json`.

─────────────────────────────────────────────────────────────────────────────
DOS DECISIONES QUE CONVIENE CONOCER ANTES DE TOCAR ESTE ARCHIVO

1 · **La tipografía no es Mont, y no puede serlo.** La licencia de Mont (P-01)
    sigue sin resolverse y no cubre incrustar la fuente en artefactos que salen
    de la empresa — un PDF lo es. Se usa Helvetica, que reportlab incrusta por
    referencia y está en todos los visores. Cuando P-01 se cierre en positivo,
    cambiar de fuente es cambiar `FUENTE` y `FUENTE_N`.

2 · **Los símbolos van dibujados, no escritos.** Helvetica no tiene emoji ni
    símbolos de señalamiento, y reportlab pinta un cuadrado negro por cada
    carácter que le falta a la fuente. Ya pasó en los informes de fase. Aquí
    cada símbolo es geometría —un triángulo, un círculo, una palomita— dibujada
    con primitivas, así que no depende de ninguna fuente y se ve igual en
    cualquier visor.
─────────────────────────────────────────────────────────────────────────────

Convenciones del Markdown de entrada:

    # Título                     portada
    > Subtítulo                   (el primero) portada
    === A · Sección               sección nueva, empieza página
    ### A-1 · Prueba              bloque de prueba
    => Debe pasar: ...            caja de resultado esperado
    !! texto                      aviso        (triángulo)
    !i texto                      información  (círculo con i)
    !x texto                      atención     (círculo con aspa)
    [[casilla]] Etiqueta          fila de resultado: Bien / Mal / notas
    %% 3 | cifras | en | fila     tira de cifras destacadas (pares valor|texto)
    @@ Etiqueta | Valor           dato de portada (varios seguidos)
    ![pie](ruta/imagen.png)       imagen con pie
    1. / -                        listas
    | tabla |                     tablas
"""

from __future__ import annotations

import argparse
import hashlib
import html
import re
import sys
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import mm
from reportlab.pdfbase.pdfmetrics import stringWidth
from reportlab.platypus import (
    BaseDocTemplate,
    CondPageBreak,
    Flowable,
    Frame,
    Image,
    KeepTogether,
    NextPageTemplate,
    PageBreak,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)

# ─────────────────────────── Paleta ───────────────────────────
#
# Los de marca salen de herramientas/design-tokens/tokens.json. ARLES es
# oscuro; el papel es claro, así que la rampa se invierte: la tinta es el azul
# profundo y el fondo es blanco. Los dos colores que cambian de valor lo hacen
# por contraste medido, no por gusto:
#
#   · el oro #FCCC0C sobre blanco da 1.5:1 — ilegible. Se oscurece a #8A6800,
#     que da 5.4:1 y sigue leyéndose como el acento de ARLES.
#   · el cian #2CA4D4 sobre blanco da 2.6:1. Para texto se usa #0A5E80 (6.2:1);
#     el original se queda para filetes, donde no carga texto.

PROFUNDO = colors.HexColor("#041A25")  # tinta de titulares y bandas
SUPERFICIE = colors.HexColor("#053048")
TINTA = colors.HexColor("#17242B")  # texto corrido
TENUE = colors.HexColor("#5A6A74")  # texto secundario
BORDE = colors.HexColor("#D2DCE2")
CREMA = colors.HexColor("#F4ECE4")  # fondo de marca, de la paleta
PAPEL_SUAVE = colors.HexColor("#F7F9FA")

ORO = colors.HexColor("#FCCC0C")  # sólo filetes y rellenos, nunca texto
ORO_TEXTO = colors.HexColor("#8A6800")  # 5.4:1 sobre blanco
CIAN = colors.HexColor("#2CA4D4")
CIAN_TEXTO = colors.HexColor("#0A5E80")  # 6.2:1 sobre blanco
EXITO = colors.HexColor("#1B7A5A")
AVISO = colors.HexColor("#8A5200")
PELIGRO = colors.HexColor("#B03124")

FUENTE = "Helvetica"
FUENTE_N = "Helvetica-Bold"
FUENTE_C = "Helvetica-Oblique"

MARGEN = 17 * mm
ANCHO_UTIL = A4[0] - 2 * MARGEN


# ─────────────────────── Símbolos dibujados ───────────────────────


class Simbolo(Flowable):
    """Un señalamiento dibujado con primitivas.

    Existe porque Helvetica no tiene estos glifos y reportlab pinta un cuadrado
    negro por cada carácter que le falta a la fuente. Dibujarlos los hace
    independientes de la tipografía.
    """

    def __init__(self, clase: str, lado: float = 11):
        super().__init__()
        self.clase = clase
        self.lado = lado
        self.width = lado
        self.height = lado

    def wrap(self, *_):
        return self.lado, self.lado

    def draw(self):
        c = self.canv
        l = self.lado
        r = l / 2
        color = {
            "aviso": AVISO,
            "info": CIAN_TEXTO,
            "peligro": PELIGRO,
            "exito": EXITO,
        }[self.clase]

        c.setLineWidth(1.1)
        c.setStrokeColor(color)
        c.setFillColor(color)

        if self.clase == "aviso":
            # Triángulo con la admiración dentro.
            p = c.beginPath()
            p.moveTo(r, l)
            p.lineTo(0, 0)
            p.lineTo(l, 0)
            p.close()
            c.setLineJoin(1)
            c.drawPath(p, stroke=1, fill=0)
            c.setLineWidth(1.3)
            c.line(r, l * 0.30, r, l * 0.62)
            c.circle(r, l * 0.19, 0.7, stroke=0, fill=1)
        elif self.clase == "exito":
            # Palomita.
            c.setLineWidth(1.6)
            c.setLineCap(1)
            c.line(l * 0.12, l * 0.52, l * 0.40, l * 0.22)
            c.line(l * 0.40, l * 0.22, l * 0.90, l * 0.80)
        else:
            c.circle(r, r, r - 0.6, stroke=1, fill=0)
            if self.clase == "info":
                c.setLineWidth(1.3)
                c.line(r, l * 0.28, r, l * 0.58)
                c.circle(r, l * 0.71, 0.75, stroke=0, fill=1)
            else:  # peligro: aspa
                c.setLineWidth(1.4)
                c.setLineCap(1)
                d = l * 0.27
                c.line(r - d, r - d, r + d, r + d)
                c.line(r - d, r + d, r + d, r - d)


class Casilla(Flowable):
    """Una casilla vacía para marcar a mano."""

    def __init__(self, lado: float = 10):
        super().__init__()
        self.lado = lado
        self.width = lado
        self.height = lado

    def wrap(self, *_):
        return self.lado, self.lado

    def draw(self):
        c = self.canv
        c.setStrokeColor(TENUE)
        c.setLineWidth(0.9)
        c.roundRect(0, 0, self.lado, self.lado, 1.5, stroke=1, fill=0)


class Filete(Flowable):
    """Filete de acento. Separa sin pesar como una línea de tabla."""

    def __init__(self, ancho: float, grosor: float = 2.2, color=ORO):
        super().__init__()
        self.ancho = ancho
        self.grosor = grosor
        self.color = color
        self.width = ancho
        self.height = grosor

    def wrap(self, *_):
        return self.ancho, self.grosor

    def draw(self):
        self.canv.setFillColor(self.color)
        self.canv.rect(0, 0, self.ancho, self.grosor, stroke=0, fill=1)


# ─────────────────────────── Texto ───────────────────────────

# Cualquier línea que empiece por una de estas marcas NO es texto corrido.
# Estaba repetida en tres sitios; una sola definición evita que se separen.
MARCAS = r"^(=>|!!|!i|!x|>|###|##|===|%%|@@|\[\[|!\[|\||```|- |\d+\. |---)"

RE_EMOJI = re.compile(
    "[\U0001f000-\U0001faff☀-➿️←-⇿⬀-⯿]"
)


def sin_emoji(texto: str) -> str:
    """Helvetica no tiene emoji: reportlab pintaría un cuadrado negro."""
    return RE_EMOJI.sub("", texto)


def inline(texto: str) -> str:
    """Markdown en línea → etiquetas de reportlab."""
    texto = sin_emoji(texto)
    texto = html.escape(texto, quote=False)
    # El código dentro de negritas primero, o quedan asteriscos sueltos.
    texto = re.sub(r"\*\*(`[^`]+`)\*\*", r"\1", texto)
    texto = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", texto)
    texto = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<i>\1</i>", texto)
    texto = re.sub(
        r"`([^`]+)`",
        rf'<font face="Courier" size="8.5" color="#{ORO_TEXTO.hexval()[2:]}">\1</font>',
        texto,
    )
    texto = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"<b>\1</b>", texto)
    return texto.replace("&amp;nbsp;", "&nbsp;")


def estilos() -> dict[str, ParagraphStyle]:
    def e(nombre, **kw):
        base = dict(
            name=nombre,
            fontName=FUENTE,
            fontSize=9.5,
            leading=13.5,
            textColor=TINTA,
            spaceAfter=0,
        )
        base.update(kw)
        return ParagraphStyle(**base)

    return {
        "portada_titulo": e(
            "pt", fontName=FUENTE_N, fontSize=30, leading=34, textColor=CREMA
        ),
        "portada_sub": e("ps", fontSize=12.5, leading=17, textColor=colors.HexColor("#BFD3DE")),
        "portada_eti": e(
            "pe", fontName=FUENTE_N, fontSize=8.5, leading=11, textColor=ORO
        ),
        "portada_dato": e("pd", fontSize=10, leading=14, textColor=CREMA),
        "seccion_letra": e(
            "sl", fontName=FUENTE_N, fontSize=22, leading=24, textColor=ORO
        ),
        "seccion": e(
            "sc", fontName=FUENTE_N, fontSize=15, leading=19, textColor=PROFUNDO
        ),
        "prueba_id": e(
            "pid", fontName=FUENTE_N, fontSize=9, leading=11, textColor=colors.white
        ),
        "prueba": e(
            "pr", fontName=FUENTE_N, fontSize=11.5, leading=15, textColor=PROFUNDO
        ),
        "cuerpo": e("cu", spaceAfter=5),
        "lista": e("li", leftIndent=13, spaceAfter=2.5),
        "espera_eti": e(
            "ee", fontName=FUENTE_N, fontSize=7.5, leading=10, textColor=EXITO
        ),
        "espera": e("es", fontSize=9.5, leading=13.5),
        "callout": e("ca", fontSize=9, leading=12.5),
        "callout_eti": e("ci", fontName=FUENTE_N, fontSize=7.5, leading=10),
        "cita": e(
            "ci2", fontSize=11.5, leading=16.5, textColor=PROFUNDO
        ),
        "cifra": e(
            "cf", fontName=FUENTE_N, fontSize=19, leading=21, textColor=ORO_TEXTO
        ),
        "cifra_eti": e("ce", fontSize=8, leading=10.5, textColor=TINTA),
        "pie_figura": e(
            "pf", fontSize=8, leading=11, textColor=TENUE, alignment=TA_CENTER
        ),
        "casilla": e("cs", fontName=FUENTE_N, fontSize=9, leading=12, textColor=TINTA),
        "notas": e("no", fontSize=8, leading=11, textColor=TENUE),
        "tabla": e("ta", fontSize=9, leading=12),
        "tabla_cab": e("tc", fontName=FUENTE_N, fontSize=9, leading=12, textColor=colors.white),
        "codigo": e(
            "co",
            fontName="Courier",
            fontSize=8.5,
            leading=11.5,
            textColor=PROFUNDO,
            leftIndent=8,
        ),
    }


# ─────────────────────────── Bloques ───────────────────────────


def callout(clase: str, texto: str, est, ancho: float) -> Table:
    etiqueta, color, fondo = {
        "aviso": ("ATENCIÓN", AVISO, colors.HexColor("#FDF5E6")),
        "info": ("NOTA", CIAN_TEXTO, colors.HexColor("#EEF6FA")),
        "peligro": ("OJO CON ESTO", PELIGRO, colors.HexColor("#FDF0EE")),
    }[clase]

    eti = ParagraphStyle("x", parent=est["callout_eti"], textColor=color)
    izquierda = Table(
        [[Simbolo(clase, 11)], [Spacer(1, 2)]],
        colWidths=[14],
        style=TableStyle([("VALIGN", (0, 0), (-1, -1), "TOP"), ("LEFTPADDING", (0, 0), (-1, -1), 0),
                          ("RIGHTPADDING", (0, 0), (-1, -1), 0), ("TOPPADDING", (0, 0), (-1, -1), 1),
                          ("BOTTOMPADDING", (0, 0), (-1, -1), 0)]),
    )
    cuerpo = [Paragraph(etiqueta, eti), Spacer(1, 2), Paragraph(inline(texto), est["callout"])]

    t = Table([[izquierda, cuerpo]], colWidths=[22, ancho - 22])
    t.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("BACKGROUND", (0, 0), (-1, -1), fondo),
                ("LINEBEFORE", (0, 0), (0, -1), 2.5, color),
                ("LEFTPADDING", (0, 0), (0, -1), 7),
                ("LEFTPADDING", (1, 0), (1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 9),
                ("TOPPADDING", (0, 0), (-1, -1), 7),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 8),
            ]
        )
    )
    return t


def caja_esperado(texto: str, est, ancho: float, etiqueta: str = "QUÉ DEBE PASAR") -> Table:
    contenido = [
        Paragraph(etiqueta, est["espera_eti"]),
        Spacer(1, 2.5),
        Paragraph(inline(texto), est["espera"]),
    ]
    t = Table([[Simbolo("exito", 11), contenido]], colWidths=[22, ancho - 22])
    t.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("BACKGROUND", (0, 0), (-1, -1), colors.HexColor("#EFF7F3")),
                ("LINEBEFORE", (0, 0), (0, -1), 2.5, EXITO),
                ("LEFTPADDING", (0, 0), (0, -1), 7),
                ("LEFTPADDING", (1, 0), (1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 9),
                ("TOPPADDING", (0, 0), (-1, -1), 7),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 8),
            ]
        )
    )
    return t


def fila_de_resultado(etiqueta: str, est, ancho: float) -> Table:
    """Bien / Mal / y sitio para escribir qué pasó."""
    marca = Table(
        [[Casilla(10), Paragraph("Bien", est["casilla"]), Spacer(10, 1),
          Casilla(10), Paragraph("Mal", est["casilla"])]],
        colWidths=[14, 30, 12, 14, 26],
        style=TableStyle([("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                          ("LEFTPADDING", (0, 0), (-1, -1), 0),
                          ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                          ("TOPPADDING", (0, 0), (-1, -1), 0),
                          ("BOTTOMPADDING", (0, 0), (-1, -1), 0)]),
    )
    t = Table(
        [
            [Paragraph(inline(etiqueta), est["casilla"]), marca],
            [Paragraph("¿Qué pasó?", est["notas"]), ""],
        ],
        colWidths=[ancho - 100, 100],
    )
    t.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("SPAN", (0, 1), (1, 1)),
                ("BACKGROUND", (0, 0), (-1, -1), PAPEL_SUAVE),
                ("BOX", (0, 0), (-1, -1), 0.7, BORDE),
                ("LINEBELOW", (0, 0), (-1, 0), 0.7, BORDE),
                # La línea para escribir: una fila con altura y el borde abajo.
                ("LINEBELOW", (0, 1), (-1, 1), 0.7, BORDE),
                ("TOPPADDING", (0, 0), (-1, 0), 7),
                ("BOTTOMPADDING", (0, 0), (-1, 0), 7),
                ("TOPPADDING", (0, 1), (-1, 1), 6),
                ("BOTTOMPADDING", (0, 1), (-1, 1), 20),
                ("LEFTPADDING", (0, 0), (-1, -1), 9),
                ("RIGHTPADDING", (0, 0), (-1, -1), 9),
            ]
        )
    )
    return t


def cita(texto: str, est, ancho: float) -> Table:
    """Cita destacada.

    Es el recurso para la frase que hay que recordar. Antes no existía y las
    líneas que empezaban por `>` salían con el símbolo a la vista, metidas
    dentro del párrafo anterior.
    """
    t = Table([[Paragraph(inline(texto), est["cita"])]], colWidths=[ancho])
    t.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), CREMA),
                ("LINEBEFORE", (0, 0), (0, -1), 3.5, ORO),
                ("LEFTPADDING", (0, 0), (-1, -1), 16),
                ("RIGHTPADDING", (0, 0), (-1, -1), 16),
                ("TOPPADDING", (0, 0), (-1, -1), 12),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 13),
            ]
        )
    )
    return t


def tira_de_cifras(pares: list[tuple[str, str]], est, ancho: float) -> Table:
    """Cifras grandes con su etiqueta debajo, en fila.

    Dan densidad donde un párrafo dejaría aire, y hacen que lo que importa se
    lea antes de leer. La cifra va en oro oscurecido: sobre papel blanco el
    `#FCCC0C` de marca da 1,5:1 y sería ilegible.
    """
    n = max(1, len(pares))
    celdas_fila = []
    for valor, etiqueta in pares:
        celdas_fila.append(
            [
                Paragraph(inline(valor), est["cifra"]),
                Spacer(1, 1),
                Paragraph(inline(etiqueta), est["cifra_eti"]),
            ]
        )
    t = Table([celdas_fila], colWidths=[ancho / n] * n)
    estilo = [
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("BACKGROUND", (0, 0), (-1, -1), CREMA),
        ("TOPPADDING", (0, 0), (-1, -1), 9),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 9),
        ("LEFTPADDING", (0, 0), (-1, -1), 11),
        ("RIGHTPADDING", (0, 0), (-1, -1), 11),
    ]
    for i in range(1, n):
        estilo.append(("LINEBEFORE", (i, 0), (i, 0), 0.7, colors.HexColor("#DED4C6")))
    t.setStyle(TableStyle(estilo))
    return t


def cabecera_de_prueba(ident: str, titulo: str, est, ancho: float) -> Table:
    """El identificador en un cuadro oscuro, y el título al lado."""
    ancho_id = max(30, stringWidth(ident, FUENTE_N, 9) + 14)
    caja = Table(
        [[Paragraph(ident, est["prueba_id"])]],
        colWidths=[ancho_id],
        style=TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), PROFUNDO),
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("ALIGN", (0, 0), (-1, -1), "CENTER"),
                ("TOPPADDING", (0, 0), (-1, -1), 4),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
                ("LEFTPADDING", (0, 0), (-1, -1), 2),
                ("RIGHTPADDING", (0, 0), (-1, -1), 2),
            ]
        ),
    )
    t = Table(
        [[caja, Paragraph(inline(titulo), est["prueba"])]],
        colWidths=[ancho_id + 8, ancho - ancho_id - 8],
    )
    t.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 0),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
            ]
        )
    )
    return t


def banda_de_seccion(letra: str, titulo: str, est, ancho: float) -> Table:
    t = Table(
        [[Paragraph(letra, est["seccion_letra"]), Paragraph(inline(titulo), est["seccion"])]],
        colWidths=[26, ancho - 26],
    )
    t.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 0),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
            ]
        )
    )
    return t


def imagen(pie: str, ruta: Path, ancho: float, est) -> list:
    if not ruta.exists():
        return [Paragraph(f"[falta la imagen {ruta.name}]", est["pie_figura"])]
    try:
        from reportlab.lib.utils import ImageReader

        iw, ih = ImageReader(str(ruta)).getSize()
    except Exception:  # noqa: BLE001 — sin la medida no se puede escalar
        return [Paragraph(f"[no se pudo leer {ruta.name}]", est["pie_figura"])]

    util = ancho * 0.88
    w = min(util, iw)
    h = w * ih / iw
    # Tope de alto: una captura vertical no puede comerse la página entera.
    alto_max = 118 * mm
    if h > alto_max:
        h = alto_max
        w = h * iw / ih

    img = Image(str(ruta), width=w, height=h)
    img.hAlign = "CENTER"
    marco = Table([[img]], colWidths=[w])
    marco.setStyle(
        TableStyle(
            [
                ("BOX", (0, 0), (-1, -1), 0.8, BORDE),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 0),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
            ]
        )
    )
    marco.hAlign = "CENTER"
    return [
        Spacer(1, 3),
        marco,
        Spacer(1, 3),
        Paragraph(inline(pie), est["pie_figura"]),
        Spacer(1, 7),
    ]


def construir_tabla(filas: list[list[str]], est, ancho: float) -> Table:
    cab, cuerpo = filas[0], filas[1:]
    # Una cabecera vacía —`| |`— no es cabecera: sin esto, la primera fila de
    # datos se pintaba con el fondo oscuro del encabezado.
    hay_cabecera = any(c.strip() for c in cab)
    datos = []
    if hay_cabecera:
        datos.append([Paragraph(inline(c), est["tabla_cab"]) for c in cab])
    else:
        cuerpo = filas[1:] if len(filas) > 1 else filas
    datos += [[Paragraph(inline(c), est["tabla"]) for c in fila] for fila in cuerpo]

    n = max(len(f) for f in datos)
    datos = [f + [""] * (n - len(f)) for f in datos]

    # Columnas proporcionales a lo que llevan dentro, no a partes iguales.
    #
    # Con reparto uniforme, una columna de números —«1», «2», «3»— se quedaba
    # con un tercio de la página en blanco mientras la de al lado se partía en
    # cuatro renglones. Se mide el texto de cada columna y se reparte, con topes
    # para que ninguna quede ni ridícula ni acaparadora.
    crudas = [
        [re.sub(r"<[^>]+>", "", c) if isinstance(c, str) else "" for c in fila]
        for fila in ([cab] if hay_cabecera else []) + cuerpo
    ]
    pesos = []
    for col in range(n):
        largo = max(
            (len(fila[col]) for fila in crudas if col < len(fila)), default=1
        )
        pesos.append(max(6, min(largo, 70)))
    total = sum(pesos)
    anchos = [ancho * peso / total for peso in pesos]
    # El suelo de cada columna es **la palabra más larga que lleva dentro**, no
    # un porcentaje. Con un porcentaje, «Operaciones» se partía en
    # «Operacione / s»: el reparto proporcional le daba poco ancho porque el
    # resto de su columna era corto. Una palabra no se puede partir, así que es
    # ella la que manda.
    minimos = []
    for col in range(n):
        palabras = [
            w
            for fila in crudas
            if col < len(fila)
            for w in fila[col].split()
        ]
        ancha = max(
            (stringWidth(w, FUENTE_N, 9) for w in palabras), default=0
        )
        minimos.append(min(ancho * 0.45, ancha + 16))

    if sum(minimos) <= ancho:
        # Se reparte lo que sobra por encima de los mínimos, en proporción.
        sobrante = ancho - sum(minimos)
        peso_total = sum(pesos) or 1
        anchos = [
            minimos[c] + sobrante * pesos[c] / peso_total for c in range(n)
        ]
    else:
        anchos = [ancho * m / sum(minimos) for m in minimos]

    t = Table(datos, colWidths=anchos, repeatRows=1 if hay_cabecera else 0)
    estilo = [
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("GRID", (0, 0), (-1, -1), 0.5, BORDE),
        ("TOPPADDING", (0, 0), (-1, -1), 5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
        ("LEFTPADDING", (0, 0), (-1, -1), 7),
        ("RIGHTPADDING", (0, 0), (-1, -1), 7),
    ]
    if hay_cabecera:
        estilo.append(("BACKGROUND", (0, 0), (-1, 0), SUPERFICIE))
        for i in range(1, len(datos)):
            if i % 2 == 0:
                estilo.append(("BACKGROUND", (0, i), (-1, i), PAPEL_SUAVE))
    else:
        for i in range(len(datos)):
            if i % 2 == 1:
                estilo.append(("BACKGROUND", (0, i), (-1, i), PAPEL_SUAVE))
    t.setStyle(TableStyle(estilo))
    return t


def es_separador(fila: list[str]) -> bool:
    celdas_con_texto = [c.strip() for c in fila if c.strip()]
    return bool(celdas_con_texto) and all(
        re.fullmatch(r":?-{2,}:?", c) for c in celdas_con_texto
    )


def celdas(linea: str) -> list[str]:
    return [c.strip() for c in linea.strip().strip("|").split("|")]


# ─────────────────────────── Conversión ───────────────────────────


def convertir(md: str, est, ancho: float, base: Path) -> tuple[list, dict]:
    flujo: list = []
    meta: dict = {"titulo": "", "subtitulo": "", "secciones": [], "pruebas": 0}
    lineas = md.splitlines()
    i = 0
    tabla: list[list[str]] = []
    lista: list = []
    codigo: list[str] = []
    en_codigo = False
    primera_seccion = True

    def cerrar_tabla():
        nonlocal tabla
        if tabla:
            flujo.append(Spacer(1, 3))
            flujo.append(construir_tabla(tabla, est, ancho))
            flujo.append(Spacer(1, 7))
            tabla = []

    def cerrar_lista():
        nonlocal lista
        if lista:
            flujo.extend(lista)
            flujo.append(Spacer(1, 4))
            lista = []

    def cerrar():
        cerrar_tabla()
        cerrar_lista()

    while i < len(lineas):
        linea = lineas[i]
        bruto = linea.rstrip()
        s = bruto.strip()

        if s.startswith("```"):
            if en_codigo:
                flujo.append(
                    Table(
                        [[Paragraph("<br/>".join(html.escape(c) for c in codigo), est["codigo"])]],
                        colWidths=[ancho],
                        style=TableStyle(
                            [
                                ("BACKGROUND", (0, 0), (-1, -1), CREMA),
                                ("LEFTPADDING", (0, 0), (-1, -1), 8),
                                ("RIGHTPADDING", (0, 0), (-1, -1), 8),
                                ("TOPPADDING", (0, 0), (-1, -1), 7),
                                ("BOTTOMPADDING", (0, 0), (-1, -1), 7),
                            ]
                        ),
                    )
                )
                flujo.append(Spacer(1, 7))
                codigo, en_codigo = [], False
            else:
                cerrar()
                en_codigo = True
            i += 1
            continue
        if en_codigo:
            codigo.append(bruto)
            i += 1
            continue

        if not s:
            cerrar()
            i += 1
            continue

        # ── Portada ──
        if s.startswith("# "):
            meta["titulo"] = sin_emoji(s[2:].strip())
            i += 1
            continue
        if s.startswith("> ") and not meta["subtitulo"]:
            meta["subtitulo"] = sin_emoji(s[2:].strip())
            i += 1
            continue

        # ── Cita destacada ──
        if s.startswith(">"):
            cerrar()
            texto = s.lstrip("> ").strip()
            while i + 1 < len(lineas) and lineas[i + 1].strip().startswith(">"):
                i += 1
                texto += " " + lineas[i].strip().lstrip("> ").strip()
            flujo.append(cita(texto, est, ancho))
            flujo.append(Spacer(1, 10))
            i += 1
            continue

        # ── Sección ──
        if s.startswith("=== "):
            cerrar()
            titulo = s[4:].strip()
            letra, _, resto = titulo.partition("·")
            letra, resto = letra.strip(), resto.strip()
            meta["secciones"].append((letra, resto))
            # `CondPageBreak` y no `PageBreak`: forzar página por sección dejaba
            # media página en blanco cada vez. Sólo salta si lo que viene no
            # cabe, que es cuando el salto sirve para algo.
            if not primera_seccion:
                flujo.append(Spacer(1, 14))
                flujo.append(CondPageBreak(42 * mm))
            primera_seccion = False
            flujo.append(banda_de_seccion(letra, resto, est, ancho))
            flujo.append(Spacer(1, 4))
            flujo.append(Filete(ancho, 2.2, ORO))
            flujo.append(Spacer(1, 9))
            i += 1
            continue

        # ── Bloque de prueba ──
        if s.startswith("### "):
            cerrar()
            titulo = s[4:].strip()
            ident, _, resto = titulo.partition("·")
            ident, resto = ident.strip(), resto.strip()
            if not resto:  # un ### sin identificador
                ident, resto = "", titulo
            meta["pruebas"] += 1
            # 38 mm y no 60: reservar más deja huecos grandes al final de las
            # páginas, que era la queja concreta de Dirección. Con 38 entra el
            # encabezado y sus primeras líneas, que es lo que hay que evitar
            # separar.
            flujo.append(CondPageBreak(38 * mm))
            flujo.append(Spacer(1, 4))
            flujo.append(cabecera_de_prueba(ident, resto, est, ancho) if ident
                         else Paragraph(inline(resto), est["prueba"]))
            flujo.append(Spacer(1, 6))
            i += 1
            continue

        if s.startswith("## "):
            cerrar()
            flujo.append(Spacer(1, 6))
            flujo.append(Paragraph(inline(s[3:].strip()), est["seccion"]))
            flujo.append(Spacer(1, 3))
            flujo.append(Filete(ancho, 1.6, CIAN))
            flujo.append(Spacer(1, 8))
            i += 1
            continue

        # ── Qué debe pasar ──
        if s.startswith("=> "):
            cerrar()
            texto = s[3:].strip()
            # Puede continuar en las líneas siguientes.
            while i + 1 < len(lineas) and lineas[i + 1].strip() and not re.match(
                MARCAS, lineas[i + 1].strip()
            ):
                i += 1
                texto += " " + lineas[i].strip()
            texto = re.sub(r"^Debe pasar:\s*", "", texto)
            # `=> ETIQUETA :: texto` permite renombrar la caja. Sin `::`, la
            # etiqueta por defecto sigue siendo la de las checklists.
            etiqueta = "QUÉ DEBE PASAR"
            if "::" in texto:
                posible, _, resto = texto.partition("::")
                if len(posible) < 40:
                    etiqueta, texto = posible.strip().upper(), resto.strip()
            flujo.append(caja_esperado(texto, est, ancho, etiqueta))
            flujo.append(Spacer(1, 7))
            i += 1
            continue

        # ── Callouts ──
        m = re.match(r"^!([!ix])\s+(.*)$", s)
        if m:
            cerrar()
            clase = {"!": "aviso", "i": "info", "x": "peligro"}[m.group(1)]
            texto = m.group(2)
            while i + 1 < len(lineas) and lineas[i + 1].strip() and not re.match(
                MARCAS, lineas[i + 1].strip()
            ):
                i += 1
                texto += " " + lineas[i].strip()
            flujo.append(callout(clase, texto, est, ancho))
            flujo.append(Spacer(1, 8))
            i += 1
            continue

        # ── Tira de cifras ──
        if s.startswith("%% "):
            cerrar()
            partes = [p.strip() for p in s[3:].split("|")]
            pares = [
                (partes[i], partes[i + 1] if i + 1 < len(partes) else "")
                for i in range(0, len(partes), 2)
            ]
            flujo.append(tira_de_cifras(pares, est, ancho))
            flujo.append(Spacer(1, 9))
            i += 1
            continue

        # ── Dato de portada ──
        if s.startswith("@@ "):
            etiqueta, _, valor = s[3:].partition("|")
            meta.setdefault("datos", []).append((etiqueta.strip(), valor.strip()))
            i += 1
            continue

        # ── Fila de resultado ──
        if s.startswith("[[casilla]]"):
            cerrar()
            flujo.append(fila_de_resultado(s[len("[[casilla]]"):].strip(), est, ancho))
            flujo.append(Spacer(1, 12))
            i += 1
            continue

        # ── Imagen ──
        m = re.match(r"^!\[(.*?)\]\((.*?)\)\s*$", s)
        if m:
            cerrar()
            flujo.extend(imagen(m.group(1), (base / m.group(2)).resolve(), ancho, est))
            i += 1
            continue

        # ── Tabla ──
        if s.startswith("|"):
            cerrar_lista()
            fila = celdas(s)
            if not es_separador(fila):
                tabla.append(fila)
            i += 1
            continue
        cerrar_tabla()

        # ── Listas ──
        m = re.match(r"^(\d+)\.\s+(.*)$", s)
        if m:
            numero, texto = m.group(1), m.group(2)
            # Continuaciones indentadas.
            while i + 1 < len(lineas) and lineas[i + 1].startswith("   ") and lineas[i + 1].strip():
                i += 1
                texto += " " + lineas[i].strip()
            lista.append(
                Table(
                    [[Paragraph(f"<b>{numero}.</b>", est["cuerpo"]),
                      Paragraph(inline(texto), est["cuerpo"])]],
                    colWidths=[16, ancho - 16],
                    style=TableStyle([("VALIGN", (0, 0), (-1, -1), "TOP"),
                                      ("LEFTPADDING", (0, 0), (-1, -1), 0),
                                      ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                                      ("TOPPADDING", (0, 0), (-1, -1), 1),
                                      ("BOTTOMPADDING", (0, 0), (-1, -1), 3)]),
                )
            )
            i += 1
            continue

        if s.startswith("- "):
            texto = s[2:]
            while i + 1 < len(lineas) and lineas[i + 1].startswith("  ") and lineas[i + 1].strip():
                i += 1
                texto += " " + lineas[i].strip()
            lista.append(
                Table(
                    [[Paragraph("&bull;", est["cuerpo"]), Paragraph(inline(texto), est["cuerpo"])]],
                    colWidths=[14, ancho - 14],
                    style=TableStyle([("VALIGN", (0, 0), (-1, -1), "TOP"),
                                      ("LEFTPADDING", (0, 0), (-1, -1), 0),
                                      ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                                      ("TOPPADDING", (0, 0), (-1, -1), 1),
                                      ("BOTTOMPADDING", (0, 0), (-1, -1), 3)]),
                )
            )
            i += 1
            continue

        if s.startswith("---"):
            i += 1
            continue

        # Texto corrido: se juntan las líneas hasta el siguiente hueco o marca.
        #
        # Sin esto cada línea del fuente era un párrafo suyo, con su propio
        # interlineado, y una negrita que cruzara de línea salía con los
        # asteriscos a la vista porque el patrón no cierra dentro de la línea.
        cerrar_lista()
        texto = s
        while i + 1 < len(lineas):
            siguiente = lineas[i + 1].strip()
            if not siguiente or re.match(MARCAS, siguiente):
                break
            i += 1
            texto += " " + siguiente
        flujo.append(Paragraph(inline(texto), est["cuerpo"]))
        i += 1

    cerrar()
    return flujo, meta


# ─────────────────────────── Página ───────────────────────────


def portada(canvas, doc, meta: dict) -> None:
    """Portada a sangre completa.

    Oscura de arriba abajo, y no a medias: la primera versión pintaba la banda
    hasta el 42 % de la altura y la tabla de datos caía debajo, con texto crema
    sobre papel blanco. Ilegible. Si el fondo es de una pieza, nada de lo que se
    coloque encima puede quedarse sin contraste.
    """
    ancho, alto = A4
    canvas.saveState()

    canvas.setFillColor(PROFUNDO)
    canvas.rect(0, 0, ancho, alto, stroke=0, fill=1)

    # Filete de acento arriba, a sangre: da presencia sin ocupar espacio.
    canvas.setFillColor(ORO)
    canvas.rect(0, alto - 6, ancho, 6, stroke=0, fill=1)

    # Marca. Es texto, no el logotipo: el logotipo es Mont Black y esta fuente
    # no lo es (P-01). Se escribe el nombre, no se imita la marca.
    canvas.setFont(FUENTE_N, 12.5)
    canvas.setFillColor(CREMA)
    canvas.drawString(MARGEN, alto - 24 * mm, "ARLES")
    w = stringWidth("ARLES ", FUENTE_N, 12.5)
    canvas.setFillColor(ORO)
    canvas.drawString(MARGEN + w, alto - 24 * mm, "RELAY I")
    canvas.setFont(FUENTE, 8.5)
    canvas.setFillColor(colors.HexColor("#8FA9B8"))
    canvas.drawRightString(ancho - MARGEN, alto - 24 * mm, "v1.2.0")

    # Pie de portada.
    canvas.setStrokeColor(colors.HexColor("#123448"))
    canvas.setLineWidth(0.6)
    canvas.line(MARGEN, 24 * mm, ancho - MARGEN, 24 * mm)
    canvas.setFont(FUENTE_N, 8.5)
    canvas.setFillColor(CREMA)
    canvas.drawString(MARGEN, 18 * mm, "TELEMETRY INSIGHT")
    canvas.setFont(FUENTE, 8.5)
    canvas.setFillColor(colors.HexColor("#8FA9B8"))
    canvas.drawRightString(ancho - MARGEN, 18 * mm, "telemetrymx.com")

    canvas.restoreState()


def cuerpo_pagina(canvas, doc, meta: dict) -> None:
    ancho, alto = A4
    canvas.saveState()

    canvas.setFont(FUENTE, 7.5)
    canvas.setFillColor(TENUE)
    canvas.drawString(MARGEN, alto - 12 * mm, meta["titulo"])
    canvas.setStrokeColor(BORDE)
    canvas.setLineWidth(0.5)
    canvas.line(MARGEN, alto - 14 * mm, ancho - MARGEN, alto - 14 * mm)

    canvas.line(MARGEN, 14 * mm, ancho - MARGEN, 14 * mm)
    canvas.setFont(FUENTE, 7.5)
    canvas.setFillColor(TENUE)
    canvas.drawString(MARGEN, 10 * mm, "ARLES RELAY I · TELEMETRY INSIGHT")
    canvas.setFillColor(PROFUNDO)
    canvas.setFont(FUENTE_N, 7.5)
    canvas.drawRightString(
        ancho - MARGEN, 10 * mm, f"{canvas.getPageNumber()} / {meta.get('paginas', '?')}"
    )
    canvas.restoreState()


def bloque_de_portada(meta: dict, est, ancho: float) -> list:
    alto = A4[1]
    flujo = [Spacer(1, alto * 0.26)]
    flujo.append(Paragraph(meta["titulo"], est["portada_titulo"]))
    flujo.append(Spacer(1, 8))
    flujo.append(Filete(58, 3, ORO))
    flujo.append(Spacer(1, 10))
    flujo.append(Paragraph(meta["subtitulo"], est["portada_sub"]))
    flujo.append(Spacer(1, alto * 0.17))

    datos = meta.get("datos") or [
        ("PARA", "Dirección · Adrián Paz"),
        ("QUÉ SE REVISA", f"{meta['pruebas']} pruebas en {len(meta['secciones'])} bloques"),
        ("CUÁNTO TARDA", "Unos 35 minutos"),
    ]
    filas = []
    for eti, val in datos:
        filas.append(
            [Paragraph(eti, est["portada_eti"]), Paragraph(val, est["portada_dato"])]
        )
    t = Table(filas, colWidths=[52 * mm, ancho - 52 * mm])
    t.setStyle(
        TableStyle(
            [
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 5),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
                ("LINEBELOW", (0, 0), (-1, -1), 0.4, colors.HexColor("#123448")),
            ]
        )
    )
    flujo.append(t)
    return flujo


def generar(origen: Path, destino: Path) -> None:
    md = origen.read_text(encoding="utf-8")
    est = estilos()
    flujo, meta = convertir(md, est, ANCHO_UTIL, origen.parent)

    if not meta["titulo"]:
        meta["titulo"] = origen.stem

    marco_portada = Frame(
        MARGEN, MARGEN, ANCHO_UTIL, A4[1] - 2 * MARGEN, id="portada",
        leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0,
    )
    marco_cuerpo = Frame(
        MARGEN, 18 * mm, ANCHO_UTIL, A4[1] - 18 * mm - 20 * mm, id="cuerpo",
        leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0,
    )

    doc = BaseDocTemplate(
        str(destino),
        pagesize=A4,
        title=meta["titulo"],
        author="TELEMETRY INSIGHT",
        subject="Checklist de revisión de ARLES RELAY I",
    )
    doc.addPageTemplates(
        [
            PageTemplate(id="portada", frames=[marco_portada],
                         onPage=lambda c, d: portada(c, d, meta)),
            PageTemplate(id="cuerpo", frames=[marco_cuerpo],
                         onPage=lambda c, d: cuerpo_pagina(c, d, meta)),
        ]
    )

    completo = (
        bloque_de_portada(meta, est, ANCHO_UTIL)
        + [NextPageTemplate("cuerpo"), PageBreak()]
        + flujo
    )

    # Dos pasadas: la primera cuenta páginas, la segunda las imprime en el pie.
    # Sin esto, «n / total» no puede saber el total mientras lo está generando.
    doc.build(list(completo))
    meta["paginas"] = doc.page
    doc2 = BaseDocTemplate(
        str(destino),
        pagesize=A4,
        title=meta["titulo"],
        author="TELEMETRY INSIGHT",
        subject="Checklist de revisión de ARLES RELAY I",
    )
    doc2.addPageTemplates(
        [
            PageTemplate(id="portada", frames=[marco_portada],
                         onPage=lambda c, d: portada(c, d, meta)),
            PageTemplate(id="cuerpo", frames=[marco_cuerpo],
                         onPage=lambda c, d: cuerpo_pagina(c, d, meta)),
        ]
    )
    flujo2, _ = convertir(md, est, ANCHO_UTIL, origen.parent)
    doc2.build(
        bloque_de_portada(meta, est, ANCHO_UTIL)
        + [NextPageTemplate("cuerpo"), PageBreak()]
        + flujo2
    )

    sha = hashlib.sha256(destino.read_bytes()).hexdigest()
    destino.with_suffix(destino.suffix + ".sha256").write_text(
        f"{sha}  {destino.name}\n", encoding="utf-8"
    )
    print(f"  ✓ {destino}  ({meta['paginas']} páginas, {meta['pruebas']} pruebas)")
    print(f"  ✓ {destino.name}.sha256")


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("origen", type=Path)
    p.add_argument("--destino", type=Path)
    a = p.parse_args()

    if not a.origen.exists():
        print(f"no existe: {a.origen}", file=sys.stderr)
        return 1

    destino = a.destino or a.origen.with_suffix(".pdf")
    generar(a.origen, destino)
    return 0


if __name__ == "__main__":
    sys.exit(main())
