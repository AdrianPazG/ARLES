#!/usr/bin/env python3
"""Genera los iconos de la aplicación desde los tokens de diseño.

    ./generar-iconos.py

Produce `crates/arles-app/icons/` con lo que Tauri necesita en Windows, macOS y
Linux. Los colores salen de `herramientas/design-tokens/tokens.json`, así que un
cambio de marca se propaga aquí sin editar nada a mano.

El §21 define el logotipo como exclusivamente tipográfico, sin isotipo. El
sistema operativo, en cambio, exige un icono cuadrado. La salida usa la inicial
del logotipo en Mont Black: es la marca, no un símbolo inventado.

Sin dependencias externas: Chromium para renderizar, stdlib para empaquetar
.ico e .icns.
"""

import base64
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile

RAIZ = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
DESTINO = os.path.join(RAIZ, "crates", "arles-app", "icons")
TOKENS = os.path.join(RAIZ, "herramientas", "design-tokens", "tokens.json")
TIPOGRAFIA = os.path.join(RAIZ, "TIPOGRAFIA")

RUTAS_CHROMIUM = [
    os.environ.get("CHROMIUM_BIN", ""),
    "/opt/pw-browsers/chromium-1194/chrome-linux/chrome",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
]

# Tamaños que Tauri espera en `icons/`.
PNGS = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
}


def chromium():
    for r in RUTAS_CHROMIUM:
        if r and os.path.exists(r):
            return r
    for n in ("chromium", "chromium-browser", "google-chrome"):
        if (r := shutil.which(n)):
            return r
    sys.exit("ERROR: no se encontró Chromium. Define CHROMIUM_BIN=/ruta/al/chrome")


def colores():
    with open(TOKENS, encoding="utf-8") as f:
        d = json.load(f)
    m = {}
    for grupo in d["grupos"].values():
        for c in grupo["colores"]:
            m[c["token"]] = c["hex"]
    return m


def fuente_b64():
    ruta = os.path.join(TIPOGRAFIA, "Mont-Black.woff2")
    if not os.path.exists(ruta):
        sys.exit(f"ERROR: falta {ruta}")
    with open(ruta, "rb") as f:
        return base64.b64encode(f.read()).decode()


def html(lado, c, fuente):
    """El icono: la inicial del logotipo sobre el fondo profundo de la marca."""
    # Proporciones relativas al lado, para que se vea igual a 32 y a 512.
    radio = lado * 0.22
    tam = lado * 0.58
    borde = max(1, round(lado * 0.012))
    return f"""<style>
@font-face{{font-family:'Mont';src:url(data:font/woff2;base64,{fuente}) format('woff2');
 font-weight:900;font-display:block}}
*{{margin:0;padding:0}}
html,body{{width:{lado}px;height:{lado}px;background:transparent;overflow:hidden}}
.icono{{
  width:{lado}px;height:{lado}px;border-radius:{radio}px;
  background:linear-gradient(145deg, {c['--arles-surface-raised']} 0%,
                                     {c['--arles-bg-deep']} 62%);
  box-shadow: inset 0 0 0 {borde}px {c['--arles-border-strong']};
  display:flex;align-items:center;justify-content:center;
}}
.letra{{
  font-family:'Mont',sans-serif;font-weight:900;
  font-size:{tam}px;line-height:1;color:{c['--arles-accent']};
  letter-spacing:-0.02em;
  /* Compensa el bearing óptico de la A en Mont Black. */
  transform:translateY({lado * 0.015}px);
}}
</style><div class="icono"><span class="letra">A</span></div>"""


def desfase_ventana(chrome):
    """Cuántos píxeles de alto se come el navegador respecto a `--window-size`.

    En headless la captura mide lo que pide `--window-size`, pero el viewport es
    más bajo: la diferencia queda como una banda vacía al pie. Si no se
    compensa, el icono sale recortado por abajo. Se mide en vez de suponerse
    porque depende de la versión de Chromium.
    """
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "vp.html")
        with open(src, "w", encoding="utf-8") as f:
            f.write(
                "<body style='margin:0'><script>addEventListener('DOMContentLoaded',"
                "()=>{document.title='vp'+innerHeight})</script></body>"
            )
        p = subprocess.run(
            [chrome, "--headless", "--no-sandbox", "--disable-gpu",
             "--window-size=400,400", "--virtual-time-budget=1500", "--dump-dom",
             f"file://{src}"],
            capture_output=True, text=True, check=False,
        )
    import re
    if m := re.search(r"vp(\d+)", p.stdout):
        return max(0, 400 - int(m.group(1)))
    return 0


def recortar_png(ruta, alto):
    """Deja el PNG en sus primeras `alto` filas.

    Sin dependencias de imagen: se descomprime, se desfiltra, se quedan las
    filas de arriba y se vuelve a comprimir sin filtro.
    """
    import zlib

    with open(ruta, "rb") as f:
        bruto = f.read()

    pos, idat = 8, bytearray()
    ancho = alto_actual = tipo_color = None
    while pos < len(bruto):
        ln = struct.unpack(">I", bruto[pos:pos + 4])[0]
        tipo = bruto[pos + 4:pos + 8]
        if tipo == b"IHDR":
            ancho, alto_actual, _prof, tipo_color = struct.unpack(
                ">IIBB", bruto[pos + 8:pos + 18]
            )
        elif tipo == b"IDAT":
            idat += bruto[pos + 8:pos + 8 + ln]
        elif tipo == b"IEND":
            break
        pos += 12 + ln

    if alto_actual == alto:
        return

    canales = {0: 1, 2: 3, 4: 2, 6: 4}[tipo_color]
    paso = ancho * canales
    datos = zlib.decompress(bytes(idat))

    previa = bytearray(paso)
    salida = bytearray()
    puntero = 0
    for y in range(alto_actual):
        filtro = datos[puntero]
        puntero += 1
        linea = bytearray(datos[puntero:puntero + paso])
        puntero += paso
        if filtro == 1:
            for i in range(canales, paso):
                linea[i] = (linea[i] + linea[i - canales]) & 255
        elif filtro == 2:
            for i in range(paso):
                linea[i] = (linea[i] + previa[i]) & 255
        elif filtro == 3:
            for i in range(canales):
                linea[i] = (linea[i] + (previa[i] >> 1)) & 255
            for i in range(canales, paso):
                linea[i] = (linea[i] + ((linea[i - canales] + previa[i]) >> 1)) & 255
        elif filtro == 4:
            for i in range(canales):
                linea[i] = (linea[i] + previa[i]) & 255
            for i in range(canales, paso):
                a, b, cc = linea[i - canales], previa[i], previa[i - canales]
                pa, pb = b - cc, a - cc
                pc = pa + pb
                pa, pb, pc = abs(pa), abs(pb), abs(pc)
                pred = a if (pa <= pb and pa <= pc) else (b if pb <= pc else cc)
                linea[i] = (linea[i] + pred) & 255
        previa = linea
        if y < alto:
            salida += b"\x00" + linea

    def chunk(tipo, datos):
        return (struct.pack(">I", len(datos)) + tipo + datos
                + struct.pack(">I", zlib.crc32(tipo + datos) & 0xFFFFFFFF))

    ihdr = struct.pack(">IIBBBBB", ancho, alto, 8, tipo_color, 0, 0, 0)
    with open(ruta, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"IDAT", zlib.compress(bytes(salida), 9)))
        f.write(chunk(b"IEND", b""))


def render(chrome, lado, c, fuente, salida, desfase):
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "i.html")
        with open(src, "w", encoding="utf-8") as f:
            f.write(html(lado, c, fuente))
        subprocess.run(
            [chrome, "--headless", "--no-sandbox", "--disable-gpu",
             "--hide-scrollbars", "--default-background-color=00000000",
             f"--window-size={lado},{lado + desfase}",
             f"--screenshot={salida}", f"file://{src}"],
            capture_output=True, text=True, check=False,
        )
    if not os.path.exists(salida):
        sys.exit(f"ERROR: Chromium no generó {salida}")
    recortar_png(salida, lado)


def escribir_ico(png_256, destino):
    """ICO con un único PNG de 256×256.

    Windows admite PNG dentro de ICO desde Vista. Un lado de 256 se codifica
    como 0 en el byte de dimensión, que es lo que marca «256» en el formato.
    """
    with open(png_256, "rb") as f:
        datos = f.read()
    cabecera = struct.pack("<HHH", 0, 1, 1)          # reservado, tipo ICO, 1 imagen
    entrada = struct.pack(
        "<BBBBHHII",
        0, 0,          # ancho y alto: 0 significa 256
        0, 0,          # paleta, reservado
        1, 32,         # planos, bits por píxel
        len(datos),
        6 + 16,        # desplazamiento: cabecera + esta entrada
    )
    with open(destino, "wb") as f:
        f.write(cabecera + entrada + datos)


def escribir_icns(png_256, png_512, destino):
    """ICNS con las variantes de 256 y 512.

    El formato es un contenedor sencillo: magia `icns`, tamaño total, y luego
    entradas de {tipo, longitud, datos}. `ic08` es 256×256 y `ic09` 512×512,
    ambas con carga PNG.
    """
    entradas = []
    for tipo, ruta in ((b"ic08", png_256), (b"ic09", png_512)):
        with open(ruta, "rb") as f:
            datos = f.read()
        entradas.append(tipo + struct.pack(">I", len(datos) + 8) + datos)
    cuerpo = b"".join(entradas)
    with open(destino, "wb") as f:
        f.write(b"icns" + struct.pack(">I", len(cuerpo) + 8) + cuerpo)


def main():
    chrome = chromium()
    c = colores()
    fuente = fuente_b64()
    os.makedirs(DESTINO, exist_ok=True)

    desfase = desfase_ventana(chrome)
    print(f"  desfase de ventana medido: {desfase}px")

    for nombre, lado in PNGS.items():
        salida = os.path.join(DESTINO, nombre)
        render(chrome, lado, c, fuente, salida, desfase)
        print(f"  ✓ {nombre}  ({lado}×{lado}, {os.path.getsize(salida) // 1024 or 1} KB)")

    png256 = os.path.join(DESTINO, "128x128@2x.png")
    png512 = os.path.join(DESTINO, "icon.png")

    ico = os.path.join(DESTINO, "icon.ico")
    escribir_ico(png256, ico)
    print(f"  ✓ icon.ico  ({os.path.getsize(ico) // 1024 or 1} KB)")

    icns = os.path.join(DESTINO, "icon.icns")
    escribir_icns(png256, png512, icns)
    print(f"  ✓ icon.icns  ({os.path.getsize(icns) // 1024 or 1} KB)")

    print(f"\nIconos en {os.path.relpath(DESTINO, RAIZ)}")


if __name__ == "__main__":
    main()
