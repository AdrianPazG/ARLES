#!/usr/bin/env python3
"""Genera los artefactos del sistema de color de ARLES RELAY desde tokens.json.

    ./generar.py --verificar   comprueba contratos y advertencias WCAG (para CI)
    ./generar.py --css         escribe dist/arles-tokens.css
    ./generar.py --lamina      renderiza la lámina PNG con Chromium
    ./generar.py --todo        las tres cosas

Sin dependencias externas: solo stdlib. Chromium únicamente para --lamina.
"""

import argparse
import base64
import json
import os
import shutil
import subprocess
import sys
import tempfile

RAIZ = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(RAIZ, "..", ".."))
TOKENS = os.path.join(RAIZ, "tokens.json")
DIST = os.path.join(RAIZ, "dist")
LAMINA = os.path.join(REPO, "REFERENCIA_DE_COLOR", "ARLES_RELAY-paleta-v1.2.0.png")
TIPOGRAFIA = os.path.join(REPO, "TIPOGRAFIA")

RUTAS_CHROMIUM = [
    os.environ.get("CHROMIUM_BIN", ""),
    "/opt/pw-browsers/chromium-1194/chrome-linux/chrome",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
]


# ─── color ────────────────────────────────────────────────────────────────────

def rgb(hexstr):
    h = hexstr.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def luminancia(c):
    def canal(v):
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    return 0.2126 * canal(c[0]) + 0.7152 * canal(c[1]) + 0.0722 * canal(c[2])


def contraste(a, b):
    x, y = luminancia(a), luminancia(b)
    x, y = max(x, y), min(x, y)
    return (x + 0.05) / (y + 0.05)


def hsl(c):
    r, g, b = [v / 255 for v in c]
    mx, mn = max(r, g, b), min(r, g, b)
    l = (mx + mn) / 2
    if mx == mn:
        return (0, 0, round(l * 100))
    d = mx - mn
    s = d / (2 - mx - mn) if l > 0.5 else d / (mx + mn)
    if mx == r:
        h = ((g - b) / d) % 6
    elif mx == g:
        h = (b - r) / d + 2
    else:
        h = (r - g) / d + 4
    return (round(h * 60), round(s * 100), round(l * 100))


# ─── modelo ───────────────────────────────────────────────────────────────────

#: Los dos temas, y de qué campo sale el color de cada uno. `hex` es el oscuro
#: porque ARLES nació oscuro: el claro es su traducción, no un sistema aparte.
TEMAS = (("oscuro", "hex"), ("claro", "claro"))


def cargar():
    with open(TOKENS, encoding="utf-8") as f:
        d = json.load(f)

    d["_mapas"] = {}
    for tema, campo in TEMAS:
        mapa = {}
        for grupo in d["grupos"].values():
            for c in grupo["colores"]:
                if campo not in c:
                    sys.exit(f"ERROR: {c['token']} no tiene valor para el tema {tema}")
                mapa[c["token"]] = c[campo]
        d["_mapas"][tema] = mapa
    #: El oscuro sigue siendo el mapa por defecto de quien no pregunte por tema.
    d["_mapa"] = d["_mapas"]["oscuro"]
    return d


def resolver(mapa, ref):
    """Un token (--arles-x) o un hex literal (#FFFFFF)."""
    if ref.startswith("#"):
        return ref
    if ref not in mapa:
        sys.exit(f"ERROR: token desconocido en tokens.json: {ref}")
    return mapa[ref]


# ─── verificación ─────────────────────────────────────────────────────────────

def verificar(d):
    """Los mismos contratos, en los dos temas.

    Un contrato que sólo se comprobara en el oscuro no serviría de nada: el
    tema claro es donde el oro se vuelve ilegible y donde los semánticos se
    invierten. Por eso la lista es una sola y se mide dos veces.
    """
    fallos = []
    contados = 0

    for tema, _ in TEMAS:
        mapa = d["_mapas"][tema]
        print(f"Contratos de contraste · tema {tema}")
        print("─" * 74)
        for c in d["contratos"]:
            fg, bg = resolver(mapa, c["frente"]), resolver(mapa, c["fondo"])
            v = contraste(rgb(bg), rgb(fg))
            ok = v >= c["min"]
            contados += 1
            print(f"  {'OK ' if ok else 'FALLA'}  {v:6.2f}:1  (min {c['min']})  "
                  f"{c['frente']} sobre {c['fondo']}")
            if not ok:
                fallos.append(f"[{tema}] {c['frente']} sobre {c['fondo']}: "
                              f"{v:.2f}:1, exigido {c['min']}:1 — {c['por_que']}")
        print()

    print("Advertencias documentadas (deben seguir fallando)")
    print("─" * 74)
    avisos = 0
    for a in d["advertencias"]:
        #: Una advertencia puede ser de un solo tema. La del cian vivo lo es:
        #: en claro ese token guarda un cian entintado que sí carga texto.
        for tema, _ in TEMAS:
            if a.get("tema", "ambos") not in ("ambos", tema):
                continue
            mapa = d["_mapas"][tema]
            fg, bg = resolver(mapa, a["frente"]), resolver(mapa, a["fondo"])
            v = contraste(rgb(bg), rgb(fg))
            ok = v < a["max"]
            avisos += 1
            print(f"  {'OK ' if ok else 'STALE'}  {v:6.2f}:1  (< {a['max']})  "
                  f"[{tema}] {a['frente']} sobre {a['fondo']}")
            if not ok:
                fallos.append(
                    f"DOCUMENTACION OBSOLETA [{tema}]: {a['frente']} sobre "
                    f"{a['fondo']} ahora da {v:.2f}:1 y ya pasaría AA. La regla "
                    f"de {a['regla']} dejó de ser cierta: actualízala o "
                    f"revierte el color.")

    print()
    if fallos:
        print(f"✗ {len(fallos)} problema(s):\n")
        for f in fallos:
            print(f"  · {f}")
        return 1
    print(f"✓ {contados} comprobaciones de contrato y {avisos} advertencias, "
          f"en los {len(TEMAS)} temas.")
    return 0


# ─── CSS ──────────────────────────────────────────────────────────────────────

def generar_css(d):
    os.makedirs(DIST, exist_ok=True)
    m = d["meta"]
    out = [
        "/* Generado por herramientas/design-tokens/generar.py — NO EDITAR A MANO.",
        f" * Fuente: herramientas/design-tokens/tokens.json",
        f" * {m['producto']} v{m['version']} · {m['objetivo_accesibilidad']} · {m['decision']}",
        " */",
        "",
        ":root {",
    ]
    for grupo in d["grupos"].values():
        out.append(f"  /* {grupo['titulo']} */")
        ancho = max(len(c["token"]) for c in grupo["colores"])
        for c in grupo["colores"]:
            out.append(f"  {c['token']:<{ancho}}: {c['hex']};  /* {c['uso']} */")
        out.append("")
    out += ["}", ""]

    # El claro se aplica por atributo y NO por `prefers-color-scheme` a secas:
    # Dirección pidió que el sistema mande de entrada pero que un desplegable
    # pueda fijarlo. Eso lo decide la aplicación escribiendo `data-tema`, que es
    # lo único que puede saber si el usuario eligió o se dejó llevar.
    todos = [c for g in d["grupos"].values() for c in g["colores"]]
    ancho = max(len(c["token"]) for c in todos)
    claro = [f"  {c['token']:<{ancho}}: {c['claro']};" for c in todos]
    out += [
        "/* Tema claro. Se activa con [data-tema=\"claro\"] en <html>.",
        " * No se cuelga de @media (prefers-color-scheme) porque el sistema es",
        " * sólo el valor de partida: el usuario puede fijarlo desde Ajustes, y",
        " * esa elección tiene que ganarle a la del sistema operativo.",
        " */",
        ':root[data-tema="claro"] {',
        *claro,
        "}",
        "",
    ]

    destino = os.path.join(DIST, "arles-tokens.css")
    with open(destino, "w", encoding="utf-8") as f:
        f.write("\n".join(out))
    print(f"✓ {os.path.relpath(destino, REPO)}")


# ─── lámina ───────────────────────────────────────────────────────────────────

def fuente_b64(nombre):
    ruta = os.path.join(TIPOGRAFIA, f"{nombre}.woff2")
    if not os.path.exists(ruta):
        sys.exit(f"ERROR: falta la tipografía {ruta}")
    with open(ruta, "rb") as f:
        return base64.b64encode(f.read()).decode()


def buscar_chromium():
    for r in RUTAS_CHROMIUM:
        if r and os.path.exists(r):
            return r
    for n in ("chromium", "chromium-browser", "google-chrome"):
        r = shutil.which(n)
        if r:
            return r
    sys.exit("ERROR: no se encontró Chromium. Define CHROMIUM_BIN=/ruta/al/chrome")


def html_lamina(d):
    mapa = d["_mapa"]
    pesos = [("Regular", 400), ("SemiBold", 600), ("Bold", 700), ("Black", 900)]
    faces = "".join(
        f"@font-face{{font-family:'Mont';src:url(data:font/woff2;base64,{fuente_b64(f'Mont-{n}')}) "
        f"format('woff2');font-weight:{w};font-style:normal;font-display:block}}"
        for n, w in pesos)

    def ficha(c):
        """Cada token con sus dos tintas, la oscura y la clara, una sobre otra.

        Van juntas a propósito: la pregunta que hay que poder contestar mirando
        la lámina es «¿esto es el mismo color traducido, o me inventé otro?», y
        eso no se ve con las dos paletas en hojas distintas.
        """
        col = rgb(c["hex"])
        h = hsl(col)
        cl = rgb(c["claro"])
        return (f'<div class="sw">'
                f'<div class="par">'
                f'<div class="chip" style="background:{c["hex"]}"></div>'
                f'<div class="chip cl" style="background:{c["claro"]}"></div>'
                f'</div>'
                f'<div class="meta"><div class="hex">{c["hex"]}'
                f'<em>{c["claro"]}</em></div>'
                f'<div class="tok">{c["token"]}</div>'
                f'<div class="num">osc HSL {h[0]}° {h[1]}% {h[2]}% &nbsp;·&nbsp; '
                f'cla HSL {hsl(cl)[0]}° {hsl(cl)[1]}% {hsl(cl)[2]}%</div>'
                f'<div class="use">{c["uso"]}</div></div></div>')

    todos = [c for g in d["grupos"].values() for c in g["colores"]]
    banda = "".join(f'<i style="background:{c["hex"]}"></i>' for c in todos)

    secciones = ""
    for clave, grupo in d["grupos"].items():
        n = len(grupo["colores"])
        cols = 4 if n % 4 == 0 else 3
        secciones += (f'<h2>{grupo["titulo"]}</h2><div class="grid g{cols}">'
                      + "".join(ficha(c) for c in grupo["colores"]) + "</div>")

    cabecera = "".join(f"<th>{f['etiqueta']}</th>" for f in d["matriz"]["frentes"])

    def matriz(tema):
        """La misma rejilla, medida en el tema que se le pida."""
        m = d["_mapas"][tema]
        filas = ""
        for bt in d["matriz"]["fondos"]:
            bg = resolver(m, bt)
            celdas = ""
            for fr in d["matriz"]["frentes"]:
                v = contraste(rgb(bg), rgb(resolver(m, fr["token"])))
                lvl = "aa" if v >= 4.5 else ("ui" if v >= 3.0 else "no")
                etq = {"aa": "AA", "ui": "UI", "no": "—"}[lvl]
                celdas += f'<td class="{lvl}"><b>{v:.2f}</b><span>{etq}</span></td>'
            filas += (f'<tr><th class="rh"><i style="background:{bg}"></i>{bg}</th>'
                      f"{celdas}</tr>")
        return filas

    filas = matriz("oscuro")
    filas_claro = matriz("claro")

    reglas = "".join(f'<div class="rule"><h3>{r["titulo"]}</h3><p>{r["cuerpo"]}</p></div>'
                     for r in d["reglas"])
    m = d["meta"]

    # Alto = suma de las alturas reales de cada bloque. Se recalcula solo al
    # añadir colores, así que la lámina nunca queda cortada ni con hueco muerto.
    ENCABEZADO, BANDA = 180, 132
    TITULO_SECCION, FICHA, HUECO = 79, 214, 18
    alto = ENCABEZADO + BANDA
    for grupo in d["grupos"].values():
        n = len(grupo["colores"])
        cols = 4 if n % 4 == 0 else 3
        n_filas = -(-n // cols)
        alto += TITULO_SECCION + n_filas * FICHA + (n_filas - 1) * HUECO
    #: Dos matrices, una por tema. Ver la nota del encabezado de `ficha`.
    alto += 2 * (TITULO_SECCION + 40 + 52 * len(d["matriz"]["fondos"]))
    #: Las reglas van en rejilla de 3. Al pasar de tres, aparece una segunda
    #: fila y el pie se salía de la lámina: el alto tiene que contarlas.
    FILA_REGLAS = 270
    alto += TITULO_SECCION + FILA_REGLAS * -(-len(d["reglas"]) // 3)
    alto += 158 + 56                                                # pie + padding
    # El pie lleva holgura deliberada: prefiero unos píxeles de fondo de más a
    # una lámina con el pie cortado. Si alguna vez sobra mucho espacio, ajusta
    # aquí en vez de recortar la imagen a mano.

    return alto, f"""<style>
{faces}
*{{margin:0;padding:0;box-sizing:border-box}}
body{{width:1400px;height:{alto}px;background:{mapa['--arles-bg-deep']};
 font-family:'Mont',sans-serif;color:{mapa['--arles-text']};
 -webkit-font-smoothing:antialiased;overflow:hidden}}
.page{{padding:64px 72px 56px}}
header{{display:flex;justify-content:space-between;align-items:flex-end;
 padding-bottom:28px;border-bottom:1px solid {mapa['--arles-border']}}}
.wm{{font-weight:900;font-size:44px;letter-spacing:.02em;line-height:1}}
.wm span{{color:{mapa['--arles-accent']}}}
.sub{{margin-top:10px;font-size:14px;color:{mapa['--arles-text-muted']};
 letter-spacing:.16em;text-transform:uppercase;font-weight:600}}
.rt{{text-align:right;font-size:13px;color:{mapa['--arles-text-muted']};line-height:1.9}}
.rt b{{color:{mapa['--arles-text']};font-weight:700}}
.band{{display:flex;height:88px;margin:36px 0 8px;border-radius:6px;overflow:hidden}}
.band i{{flex:1}}
h2{{font-size:12px;letter-spacing:.22em;text-transform:uppercase;color:{mapa['--arles-info']};
 font-weight:700;margin:44px 0 20px;display:flex;align-items:center;gap:16px}}
h2::after{{content:'';flex:1;height:1px;background:{mapa['--arles-border']}}}
.grid{{display:grid;gap:18px}}
.g3{{grid-template-columns:repeat(3,1fr)}}
.g4{{grid-template-columns:repeat(4,1fr)}}
.sw{{background:{mapa['--arles-surface']};border:1px solid {mapa['--arles-border']};
 border-radius:8px;overflow:hidden}}
.par{{display:grid;grid-template-columns:1fr 1fr}}
.chip{{height:104px;position:relative}}
.chip::after{{content:'OSCURO';position:absolute;left:10px;bottom:8px;font-size:9px;
 letter-spacing:.16em;font-weight:700;color:{mapa['--arles-text']};opacity:.55;
 mix-blend-mode:difference}}
.chip.cl::after{{content:'CLARO'}}
.meta{{padding:16px 18px 18px}}
.hex{{font-size:21px;font-weight:700;letter-spacing:.06em;display:flex;
 justify-content:space-between;align-items:baseline;gap:10px}}
.hex em{{font-style:normal;font-size:15px;font-weight:600;opacity:.62}}
.tok{{font-size:12px;font-weight:600;color:{mapa['--arles-info']};margin-top:5px}}
.num{{font-size:10.5px;color:{mapa['--arles-text-muted']};opacity:.72;margin-top:10px}}
.use{{font-size:12px;color:{mapa['--arles-text-muted']};margin-top:9px;line-height:1.45}}
table{{width:100%;border-collapse:collapse;font-size:13px}}
th,td{{padding:13px 8px;text-align:center}}
thead th{{font-size:10px;letter-spacing:.14em;text-transform:uppercase;
 color:{mapa['--arles-text-muted']};font-weight:700;
 border-bottom:1px solid {mapa['--arles-border']};opacity:.8}}
.rh{{text-align:left;font-size:12px;font-weight:600;letter-spacing:.05em;
 white-space:nowrap;padding-left:0}}
.rh i{{display:inline-block;width:26px;height:26px;border-radius:4px;vertical-align:middle;
 margin-right:11px;border:1px solid {mapa['--arles-border-strong']}}}
tbody tr{{border-bottom:1px solid {mapa['--arles-border']}80}}
td b{{display:block;font-size:15px;font-weight:700;line-height:1.15}}
td span{{display:block;font-size:9px;letter-spacing:.12em;margin-top:3px;font-weight:700}}
td.aa b{{color:{mapa['--arles-text']}}} td.aa span{{color:{mapa['--arles-success']}}}
td.ui b{{color:{mapa['--arles-text-muted']};opacity:.85}} td.ui span{{color:{mapa['--arles-warning']}}}
td.no b{{color:{mapa['--arles-text-muted']};opacity:.3}}
td.no span{{color:{mapa['--arles-danger']};opacity:.75}}
.rules{{display:grid;grid-template-columns:repeat(3,1fr);gap:22px}}
.rule{{background:{mapa['--arles-surface']};border:1px solid {mapa['--arles-border']};
 border-left:3px solid {mapa['--arles-accent']};border-radius:6px;padding:20px 22px}}
.rule h3{{font-size:13.5px;font-weight:700;margin-bottom:9px;line-height:1.35}}
.rule p{{font-size:12px;color:{mapa['--arles-text-muted']};line-height:1.6}}
.rule code{{font-family:'Mont';font-weight:700;color:{mapa['--arles-accent']};letter-spacing:.04em}}
footer{{margin-top:38px;padding-top:20px;border-top:1px solid {mapa['--arles-border']};
 display:flex;justify-content:space-between;font-size:11px;
 color:{mapa['--arles-text-muted']};opacity:.7;line-height:1.7}}
footer b{{font-weight:700;opacity:1}}
</style>
<div class="page">
<header>
  <div><div class="wm">ARLES <span>RELAY</span></div>
  <div class="sub">Sistema de color</div></div>
  <div class="rt"><b>v{m['version']}</b> · Dark-first<br>
  Objetivo <b>{m['objetivo_accesibilidad']}</b><br>
  Ratios calculados, no estimados</div>
</header>
<div class="band">{banda}</div>
{secciones}
<h2>Matriz de contraste · tema oscuro</h2>
<table><thead><tr><th class="rh">Superficie</th>{cabecera}</tr></thead>
<tbody>{filas}</tbody></table>
<h2>Matriz de contraste · tema claro</h2>
<table><thead><tr><th class="rh">Superficie</th>{cabecera}</tr></thead>
<tbody>{filas_claro}</tbody></table>
<h2>Las reglas que salen de los datos</h2>
<div class="rules">{reglas}</div>
<footer>
  <div><b>Origen.</b> Cian, oro y cremas extraídos de <b>/REFERENCIA_DE_COLOR</b>
  (PNG 2320×3080, 1.027 líneas muestreadas).<br>
  Los azules profundos se construyen por rampa: la referencia solo contiene
  0.73&#37; de azul profundo. Ver {m['decision']}.</div>
  <div style="text-align:right"><b>AA</b> texto normal ≥ 4.5:1 &nbsp;·&nbsp;
  <b>UI</b> texto grande e interfaz ≥ 3.0:1<br>
  Generado desde <b>tokens.json</b> · TELEMETRY INSIGHT</div>
</footer></div>"""


def generar_lamina(d):
    alto, html = html_lamina(d)
    chrome = buscar_chromium()
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(tmp, "lamina.html")
        with open(src, "w", encoding="utf-8") as f:
            f.write(html)
        r = subprocess.run(
            [chrome, "--headless", "--no-sandbox", "--disable-gpu", "--hide-scrollbars",
             "--force-device-scale-factor=2", f"--window-size=1400,{alto}",
             f"--screenshot={LAMINA}", f"file://{src}"],
            capture_output=True, text=True)
        if not os.path.exists(LAMINA):
            sys.exit(f"ERROR: Chromium no generó la lámina.\n{r.stderr[-800:]}")
    kb = os.path.getsize(LAMINA) // 1024
    print(f"✓ {os.path.relpath(LAMINA, REPO)}  (2800×{alto * 2}, {kb} KB)")


# ─── cli ──────────────────────────────────────────────────────────────────────

def main():
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--verificar", action="store_true", help="comprueba WCAG (CI)")
    p.add_argument("--css", action="store_true", help="escribe dist/arles-tokens.css")
    p.add_argument("--lamina", action="store_true", help="renderiza la lámina PNG")
    p.add_argument("--todo", action="store_true", help="las tres cosas")
    a = p.parse_args()
    if not any([a.verificar, a.css, a.lamina, a.todo]):
        p.print_help()
        return 0

    d = cargar()
    codigo = 0
    if a.verificar or a.todo:
        codigo = verificar(d)
    if (a.css or a.todo) and codigo == 0:
        generar_css(d)
    if (a.lamina or a.todo) and codigo == 0:
        generar_lamina(d)
    return codigo


if __name__ == "__main__":
    sys.exit(main())
