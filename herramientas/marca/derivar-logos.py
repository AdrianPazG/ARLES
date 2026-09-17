#!/usr/bin/env python3
"""Tiñe el logotipo horizontal de TELEMETRY con los colores de ARLES.

    python3 herramientas/marca/derivar-logos.py

Escribe en `app/src/app/activos/marca/`. Los originales de `/RECURSOS/marca`
**no se tocan**: se leen y nada más. Dirección los declaró material de
referencia de sólo lectura, y además conviene que sigan siendo el original
exacto que entregó la marca, no una versión que ARLES fue retocando.

─────────────────────────────────────────────────────────────────────────────
POR QUÉ HACE FALTA TEÑIRLOS

Los dos archivos que hay en `/RECURSOS/marca` son la misma pieza en dos
tintas, y ninguna de las dos es un color de la paleta de ARLES:

    entregado #EFE7DC   vs   --arles-text     #F4ECE4
    entregado #001638   vs   --arles-bg-deep  #041A25

El claro se acerca, pero no coincide: puesto al lado del texto de la interfaz
se lee como un blanco sucio. El oscuro es un azul de otra familia —matiz 216°
frente a los 200° de ARLES—, así que sobre el fondo de la aplicación tira a
violeta mientras todo lo demás tira a cian. Eso es lo que desentona.

La pieza es de un solo color sobre transparencia, así que el color no es parte
del dibujo: es una tinta aplicada a una máscara. Teñirla no la modifica, la
completa. Las tintas salen de `tokens.json`, de modo que un cambio de paleta se
propaga aquí solo (§17: ningún hex escrito a mano).
─────────────────────────────────────────────────────────────────────────────

La pieza llega **a sangre**: la tinta toca los cuatro bordes, sin margen. Quien
la coloque en una pantalla tiene que poner el aire alrededor; no viene dentro
del archivo. Está anotado en documentacion/05-diseno/MARCA_TELEMETRY.md.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

from PIL import Image, ImageChops

RAIZ = Path(__file__).resolve().parents[2]
ORIGENES = RAIZ / "RECURSOS" / "marca"
DESTINO = RAIZ / "app" / "src" / "app" / "activos" / "marca"
TOKENS = RAIZ / "herramientas" / "design-tokens" / "tokens.json"

#: Los dos archivos que entregó la marca. Se leen los dos: uno da la máscara y
#: el otro sirve para comprobar que son la misma pieza. Si algún día llega un
#: tercero que no coincide, esto lo dice en vez de tragárselo.
MASCARA = ORIGENES / "Logo_Telemetry_Horizontal_Blanco.png"
CONTRASTE = ORIGENES / "Recurso 45@4x-8.png"

#: Los dos originales son la misma pieza reexportada, así que difieren en el
#: borde por un píxel de desplazamiento y por el remuestreo. Medido: 1.25 %.
#: El tope deja margen para otra reexportación, no para otro dibujo.
MAX_DIVERGENCIA = 0.04

#: Ancho de salida. La pieza mide 2776 px de ancho, que es tamaño de imprenta:
#: en la barra lateral ocupa unos 140 px. 560 da holgura para el doble de
#: densidad y para un encabezado grande, sin cargar el binario.
ANCHO = 560

#: Qué tinta lleva cada salida. La clave es el tema al que sirve, no el color
#: que tiene: «claro» es el archivo para fondos claros, y por eso va en tinta
#: oscura. Nombrarlos por el color —como los originales— es justo lo que
#: hace que alguien ponga el blanco sobre blanco.
SALIDAS = {
    "telemetry-horizontal-tema-oscuro.png": "--arles-text",
    "telemetry-horizontal-tema-claro.png": "--arles-bg-deep",
}

#: La misma pieza como **máscara**: blanca sobre transparencia.
#:
#: Es la que usa la interfaz. Las dos teñidas de arriba siguen existiendo para
#: el papel y para cualquier sitio donde no se pueda enmascarar, pero dentro de
#: la aplicación una máscara es estrictamente mejor: el color lo pone el token
#: y el logotipo **no puede quedarse con la tinta del otro tema**.
#:
#: Ya pasó: con dos archivos y una regla de CSS para elegir, la regla del tema
#: claro se descartó al compilar y el pie se quedó con la tinta crema sobre
#: papel claro, casi invisible. Con máscara no hay regla que descartar.
MASCARA_SALIDA = "telemetry-horizontal-mascara.png"

#: Sólo el símbolo, sin «TELEMETRY INSIGHT», también como máscara.
#:
#: Con la barra lateral plegada el logotipo horizontal no entra en 64 px y
#: desaparecía entero, dejando el pie sin ninguna marca. Dirección pidió
#: conservar el símbolo solo.
#:
#: **Dónde corta no está escrito a mano.** Se busca el hueco vertical más ancho
#: de la pieza: entre el símbolo y el texto hay 118 px sin tinta, mientras que
#: los espacios entre letras rondan los 22. Un número fijo aquí sería un número
#: que caduca en cuanto la marca reexporte el archivo con otro encuadre.
ISOTIPO_SALIDA = "telemetry-isotipo-mascara.png"

#: Por debajo de esto, un hueco es espaciado entre letras y no una separación
#: de bloques. Medido: 118 px el bueno, 26 px el mayor de los otros.
MIN_HUECO_PX = 60


def tintas() -> dict[str, str]:
    datos = json.loads(TOKENS.read_text(encoding="utf-8"))
    return {
        color["token"]: color["hex"]
        for grupo in datos["grupos"].values()
        for color in grupo["colores"]
    }


def rgb(hexa: str) -> tuple[int, int, int]:
    h = hexa.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16))


def alfa(ruta: Path) -> Image.Image:
    return Image.open(ruta).convert("RGBA").getchannel("A")


def corte_del_isotipo(mascara: Image.Image) -> int:
    """La columna donde acaba el símbolo y empieza el texto.

    Es el hueco vertical más ancho de la pieza. Devuelve su columna de inicio,
    que es por donde hay que recortar.
    """
    px = mascara.load()
    ancho, alto = mascara.size
    con_tinta = []
    for x in range(ancho):
        # Muestreo cada 4 filas: basta para saber si la columna tiene tinta, y
        # recorrerlas todas multiplica por cuatro el tiempo sin cambiar nada.
        con_tinta.append(any(px[x, y] > 16 for y in range(0, alto, 4)))

    mejor, inicio = None, None
    for x, hay in enumerate(con_tinta):
        if not hay and inicio is None:
            inicio = x
        elif hay and inicio is not None:
            if x - inicio >= MIN_HUECO_PX and (mejor is None or x - inicio > mejor[1]):
                mejor = (inicio, x - inicio)
            inicio = None

    if mejor is None:
        raise SystemExit(
            f"No encuentro ningún hueco de {MIN_HUECO_PX} px o más en el "
            "logotipo horizontal: no puedo separar el símbolo del texto. "
            "¿Cambió la pieza?"
        )
    return mejor[0]


def comprobar_que_son_la_misma_pieza(a: Image.Image, b: Image.Image) -> float:
    """Devuelve la fracción de píxeles cuya opacidad no coincide."""
    diferencia = ImageChops.difference(a, b.resize(a.size, Image.LANCZOS))
    histograma = diferencia.histogram()
    return sum(histograma[8:]) / sum(histograma)


def main() -> int:
    for ruta in (MASCARA, CONTRASTE, TOKENS):
        if not ruta.exists():
            print(f"falta {ruta.relative_to(RAIZ)}", file=sys.stderr)
            return 1

    mascara = alfa(MASCARA)
    divergencia = comprobar_que_son_la_misma_pieza(mascara, alfa(CONTRASTE))
    if divergencia > MAX_DIVERGENCIA:
        print(
            f"Los dos originales de /RECURSOS/marca ya no son la misma pieza: "
            f"difieren en el {divergencia:.1%} de los píxeles (tope "
            f"{MAX_DIVERGENCIA:.0%}). Teñir uno solo daría dos logotipos "
            f"distintos. Revisa cuál es el bueno antes de seguir.",
            file=sys.stderr,
        )
        return 1

    # El corte se busca sobre el original, antes de reducir: a 560 px de ancho
    # el hueco mide 24 px y quedaría a la altura del espaciado entre letras.
    corte = corte_del_isotipo(mascara)
    isotipo = mascara.crop((0, 0, corte, mascara.height))
    caja = isotipo.getbbox()
    if caja:
        isotipo = isotipo.crop(caja)

    alto = round(mascara.height * ANCHO / mascara.width)
    mascara = mascara.resize((ANCHO, alto), Image.LANCZOS)

    paleta = tintas()
    DESTINO.mkdir(parents=True, exist_ok=True)
    print(f"Máscara: {MASCARA.name}  ·  divergencia {divergencia:.2%}")
    for nombre, token in SALIDAS.items():
        hexa = paleta[token]
        lienzo = Image.new("RGBA", mascara.size, (*rgb(hexa), 0))
        lienzo.putalpha(mascara)
        lienzo.save(DESTINO / nombre, optimize=True)
        peso = (DESTINO / nombre).stat().st_size
        print(f"  ✓ {nombre}  {ANCHO}×{alto}  {token} {hexa}  {peso / 1024:.1f} kB")

    lienzo = Image.new("RGBA", mascara.size, (255, 255, 255, 0))
    lienzo.putalpha(mascara)
    lienzo.save(DESTINO / MASCARA_SALIDA, optimize=True)
    peso = (DESTINO / MASCARA_SALIDA).stat().st_size
    print(f"  ✓ {MASCARA_SALIDA}  {ANCHO}×{alto}  máscara  {peso / 1024:.1f} kB")

    iso_alto = 160
    iso_ancho = round(isotipo.width * iso_alto / isotipo.height)
    isotipo = isotipo.resize((iso_ancho, iso_alto), Image.LANCZOS)
    lienzo = Image.new("RGBA", isotipo.size, (255, 255, 255, 0))
    lienzo.putalpha(isotipo)
    lienzo.save(DESTINO / ISOTIPO_SALIDA, optimize=True)
    peso = (DESTINO / ISOTIPO_SALIDA).stat().st_size
    print(
        f"  ✓ {ISOTIPO_SALIDA}  {iso_ancho}×{iso_alto}  símbolo solo, "
        f"cortado en x={corte}  {peso / 1024:.1f} kB"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
