#!/usr/bin/env python3
"""Empaqueta ARLES en **un solo archivo HTML** que se abre con doble clic.

    python3 herramientas/vista-previa/generar.py

Deja `vista-previa/ARLES-vista-previa.html`, sin nada al lado: la hoja de
estilos, el JavaScript, las cuatro tipografías y las imágenes viajan dentro.

─────────────────────────────────────────────────────────────────────────────
PARA QUÉ ES, Y QUÉ NO ES

Dirección pidió «solo preocuparme de descargar y probar». El instalador de
verdad no puede salir de aquí —los bundles son de Windows y de macOS, y esto es
Linux— y construirlo en GitHub exige etiquetas, permisos y veinte minutos.

Esto es otra cosa y llega antes: **la interfaz real**, con sus componentes, sus
tokens, sus dos temas y sus textos, corriendo en el navegador que ya está
instalado. Sirve para recorrer los bloques A a F de la checklist visual, que es
lo que había que revisar.

**Lo que NO es:**

- **No es la aplicación de escritorio.** No hay ventana nativa, ni base de datos
  cifrada, ni llavero del sistema. Lo que hay detrás es el **mismo núcleo
  simulado que usan las sondas**, guardando en el almacenamiento de la pestaña.
- Por tanto **nada de lo que escribas se guarda de verdad**. Sobrevive a
  recargar la página —que es lo que hace falta para probar «cerrar y volver a
  abrir»— y desaparece al cerrar la pestaña.
- **No sustituye a la revisión en la aplicación real**, y muy concretamente no
  sustituye a la de macOS (R-07): ahí el motor es WKWebView y aquí es el del
  navegador de quien lo abra.

── Sobre la tipografía ──

Esta vista previa **sí lleva Mont**, al contrario que el instalador. No es una
contradicción: el instalador se **distribuye**, y eso es lo que la licencia de
Fontfabric no cubre mientras P-01 siga sin confirmarse. Esto es una vista previa
interna para quien tiene esa licencia, equivalente a mirar el `vite dev` del
equipo. Si Dirección prefiere que tampoco la lleve, se genera con
`--sin-mont`.
─────────────────────────────────────────────────────────────────────────────
"""

from __future__ import annotations

import argparse
import base64
import mimetypes
import re
import shutil
import subprocess
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
APP = RAIZ / "app"
DIST = APP / "dist-vista-previa"
SALIDA = RAIZ / "vista-previa" / "ARLES-vista-previa.html"

#: El núcleo simulado. Es el mismo contrato que `arles-app`, con dos
#: diferencias deliberadas:
#:
#:   · Guarda en `sessionStorage`, no en una base cifrada. Así «cerrar y volver
#:     a abrir» —recargar la página— conserva el tema y la barra plegada, que es
#:     lo que piden los renglones B-5 y E-3 de la checklist.
#:   · Empieza **sin empresa configurada**, para que lo primero que se vea sea
#:     la portada de primera vez (bloque A).
# ─────────────────────────────────────────────────────────────────────────
# El núcleo simulado vive en `simulador.js`, al lado de este archivo.
#
# Estuvo aquí dentro, como una cadena de diecinueve mil caracteres de
# JavaScript. Se sacó el 29/09/2026: sin resaltado ni lint, y con las barras
# escapadas dos veces, añadir un comando exigía contar comillas — y faltaban
# once, los de las Fases 5 y 6 y del canal de WhatsApp.
NUCLEO_SIMULADO = "<script>\n" + (
    Path(__file__).parent / "simulador.js"
).read_text(encoding="utf-8") + "\n</script>"


def corre(cmd: list[str], cwd: Path) -> None:
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    if r.returncode != 0:
        print(r.stdout[-3000:], file=sys.stderr)
        print(r.stderr[-3000:], file=sys.stderr)
        raise SystemExit(f"falló: {' '.join(cmd)}")


def incrustar_en_css(css: str, base: Path) -> str:
    """Mete fuentes e imágenes dentro del CSS como `data:`."""

    def uno(m: re.Match[str]) -> str:
        ref = m.group(1).strip("'\"")
        if ref.startswith(("data:", "http:", "https:")):
            return m.group(0)
        archivo = base / Path(ref).name
        if not archivo.exists():
            return m.group(0)
        tipo = mimetypes.guess_type(archivo.name)[0] or "application/octet-stream"
        datos = base64.b64encode(archivo.read_bytes()).decode("ascii")
        return f"url(data:{tipo};base64,{datos})"

    return re.sub(r"url\(([^)]+)\)", uno, css)


def main() -> int:
    p = argparse.ArgumentParser(description="ARLES en un solo archivo HTML")
    p.add_argument("--sin-mont", action="store_true", help="sin la tipografía de la marca")
    args = p.parse_args()

    # ─────────────────────────────────────────────────────────────────────
    # `--sin-mont` TOCA EL PROYECTO, ASÍ QUE SE DEVUELVE COMO ESTABA
    #
    # `sin-mont.py --aplicar` reescribe `app/src/design/tipografia.css`, y hasta
    # el 29/09/2026 lo dejaba reescrito para siempre. Generar una vez la vista
    # previa sin la tipografía dejaba el repositorio sin ella **y sin avisar**:
    # la siguiente generación normal salía también sin Mont, 232 kB más ligera,
    # y nadie sabía por qué.
    #
    # Ahora se guarda el archivo antes y se restaura al terminar, pase lo que
    # pase — incluso si la compilación falla a mitad, que es justo cuando más
    # duele quedarse con el proyecto a medio modificar.
    tipografia = RAIZ / "app/src/design/tipografia.css"
    original = tipografia.read_text(encoding="utf-8") if args.sin_mont else None

    try:
        if args.sin_mont:
            corre(["python3", "herramientas/marca/sin-mont.py", "--aplicar"], RAIZ)
        return generar()
    finally:
        if original is not None:
            tipografia.write_text(original, encoding="utf-8")


def generar() -> int:
    print("Compilando el frontend…")
    corre(
        [
            "npx", "vite", "build",
            "--config", "vite.config.vista-previa.ts",
            "--outDir", "dist-vista-previa", "--emptyOutDir",
        ],
        APP,
    )

    html = (DIST / "index.html").read_text(encoding="utf-8")
    activos = DIST / "assets"

    # El CSS, con las fuentes y las máscaras dentro.
    for m in re.finditer(r'<link[^>]+href="([^"]+\.css)"[^>]*>', html):
        archivo = activos / Path(m.group(1)).name
        css = incrustar_en_css(archivo.read_text(encoding="utf-8"), activos)
        html = html.replace(m.group(0), f"<style>{css}</style>")

    # El JavaScript.
    for m in re.finditer(r'<script[^>]+src="([^"]+\.js)"[^>]*></script>', html):
        archivo = activos / Path(m.group(1)).name
        js = archivo.read_text(encoding="utf-8")
        html = html.replace(m.group(0), f"<script type=\"module\">{js}</script>")

    # Cualquier imagen que haya quedado suelta en el HTML.
    def imagen(m: re.Match[str]) -> str:
        ref = m.group(2)
        archivo = activos / Path(ref).name
        if not archivo.exists():
            return m.group(0)
        tipo = mimetypes.guess_type(archivo.name)[0] or "application/octet-stream"
        datos = base64.b64encode(archivo.read_bytes()).decode("ascii")
        return f'{m.group(1)}="data:{tipo};base64,{datos}"'

    html = re.sub(r'(src|href)="(/assets/[^"]+)"', imagen, html)

    # ── La CSP ──
    #
    # El producto declara `default-src 'self'`, que **prohíbe el script en
    # línea**. Es un control real y hace exactamente su trabajo: la primera
    # versión de esta vista previa salía en blanco porque el navegador se negó a
    # ejecutar nada, y con razón.
    #
    # Aquí todo va en línea —es el punto de tener un solo archivo—, así que la
    # CSP se relaja **en esta copia y sólo aquí**. Se sustituye por una
    # explícita en vez de borrarla: sin ninguna, el archivo podría cargar
    # cualquier cosa de internet, y esto se abre desde el disco.
    #
    # Es, además, la razón más clara de por qué este archivo NO es el producto.
    html = re.sub(
        r'<meta\s+http-equiv="Content-Security-Policy"[^>]*>',
        '<meta http-equiv="Content-Security-Policy" content="'
        "default-src 'none'; "
        "script-src 'unsafe-inline'; "
        "style-src 'unsafe-inline'; "
        "img-src data:; "
        "font-src data:"
        '">',
        html,
        flags=re.S,
    )

    # El icono de la pestaña queda como una referencia a un archivo que no
    # viaja. No se incrusta: un favicon ausente es un detalle del navegador, y
    # dejar el enlace roto produce un error en la consola que confunde a quien
    # abra las herramientas de desarrollo buscando otra cosa.
    html = re.sub(r'<link[^>]+rel="icon"[^>]*>', "", html)

    # El núcleo simulado va **antes** del módulo de la aplicación: `hayNucleo()`
    # se consulta en el primer render, y si llegara después la pantalla ya
    # habría decidido que no hay núcleo.
    html = html.replace("</head>", NUCLEO_SIMULADO + "</head>", 1)

    # Un aviso dentro del propio archivo. Sin él, alguien que lo reciba
    # reenviado no tiene forma de saber que no es la aplicación.
    html = html.replace(
        "<title>",
        "<!-- VISTA PREVIA de ARLES RELAY I. No es la aplicación de escritorio:\n"
        "     no hay base de datos cifrada ni llavero del sistema, y nada de lo\n"
        "     que se escriba aquí se guarda de verdad. Generado con\n"
        "     herramientas/vista-previa/generar.py -->\n<title>",
        1,
    )

    SALIDA.parent.mkdir(parents=True, exist_ok=True)
    SALIDA.write_text(html, encoding="utf-8")
    shutil.rmtree(DIST, ignore_errors=True)

    sueltos = len(re.findall(r'(?:src|href)="(?:\./)?assets/', html))
    if sueltos:
        raise SystemExit(
            f"quedaron {sueltos} referencias a archivos de al lado: el HTML no "
            "es autocontenido y al abrirlo faltarían piezas."
        )

    kb = SALIDA.stat().st_size / 1024
    print(f"\n✓ {SALIDA.relative_to(RAIZ)}  ·  {kb:.0f} kB  ·  un solo archivo")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
