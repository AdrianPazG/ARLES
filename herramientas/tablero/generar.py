#!/usr/bin/env python3
"""Genera el tablero visual de ARLES: avance, alarmas y pendientes.

    python3 herramientas/tablero/generar.py            # usa la última medición
    python3 herramientas/tablero/generar.py --medir     # corre las diez fases

Deja `tablero/ARLES-tablero.html`, que se abre con **doble clic**: un archivo
suelto, sin servidor y sin nada que instalar, igual que la vista previa.

─────────────────────────────────────────────────────────────────────────────
POR QUÉ ESTO NO ESCRIBE UN SOLO NÚMERO A MANO

Dirección pidió el 14/10/2026 «una carpeta donde podamos graficar los avances,
ver el estatus de las tareas visualmente e indicadores visuales de alarmas,
pendientes y actividades por resolver». La forma fácil de darle eso es un HTML
con las cifras escritas dentro.

**Es la forma que este proyecto ya sabe que falla**, y no por teoría: la
sección §9 del `CLAUDE.md` estuvo diciendo «entrega 3.3» con la Fase 4 cerrada
y la 5 a medias, y el propio archivo lo cuenta así: *«un archivo que se lee al
abrir el proyecto y que miente sobre el estado es peor que no tenerlo, porque
se cree»*. Un tablero es exactamente eso, pero con colores, que dan más
confianza todavía.

Así que aquí **no hay ni una cifra propia**. Todas salen de donde ya viven:

  · el porcentaje y las fases → `documentacion/07-entrega/avance.json`, que es
    la fuente única desde el 17/09/2026 y la que el validador compara con el
    README;
  · los bloqueos externos → el mismo archivo, campo `bloqueos`;
  · los hallazgos de las pruebas → la tabla de `BITACORA-DE-PRUEBAS.md`, con su
    gravedad y su estado tal como están escritos ahí;
  · los tropiezos de instalación → los títulos de `REGISTRO-DE-TROPIEZOS.md`;
  · las comprobaciones en verde → **una corrida de verdad** del validador,
    guardada en `tablero/corrida.json` con su fecha y su hora.

Si una fuente falta, el tablero **lo dice en su sitio** en vez de dejar el
hueco en blanco o poner un cero: un cero se lee como «no hay ninguno» y un
hueco se lee como «todo bien».

LA MEDICIÓN LLEVA FECHA, Y SE VE CUANDO ENVEJECE

`--medir` corre las diez fases y guarda el resultado. Sin `--medir` se usa la
última, **y el tablero enseña de cuándo es**. Pasada una semana lo dice en
amarillo, porque una cifra sin fecha es un adjetivo (§94) y una cifra con
fecha vieja presentada como actual es peor que no tenerla.
"""

from __future__ import annotations

import argparse
import datetime
import html
import json
import re
import subprocess
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
AVANCE = RAIZ / "documentacion" / "07-entrega" / "avance.json"
BITACORA = RAIZ / "documentacion" / "06-calidad" / "BITACORA-DE-PRUEBAS.md"
TROPIEZOS = RAIZ / "documentacion" / "07-entrega" / "REGISTRO-DE-TROPIEZOS.md"
VALIDAR = RAIZ / "herramientas" / "validar" / "validar.py"
SALIDA = RAIZ / "tablero" / "ARLES-tablero.html"
CORRIDA = RAIZ / "tablero" / "corrida.json"

#: Las diez fases que hay que correr antes de empujar (CLAUDE.md §6).
FASES = ["0", "1", "2", "3", "4", "5", "6", "W", "7", "8"]

#: A partir de cuántos días una medición se enseña como vieja.
DIAS_PARA_ENVEJECER = 7


# ───────────────────────────────────────────────────────────────────────────
# LEER LAS FUENTES
# ───────────────────────────────────────────────────────────────────────────

def leer_avance() -> dict:
    """El porcentaje, las fases y los bloqueos, de la fuente única."""
    if not AVANCE.exists():
        return {"falta": f"no existe {AVANCE.relative_to(RAIZ)}"}
    d = json.loads(AVANCE.read_text(encoding="utf-8"))
    fases = [f for f in d.get("fases", [])]
    total = sum(f["peso"] * f["hecho"] for f in fases)
    return {
        "version": d.get("version", "?"),
        "actualizado": d.get("actualizado", "?"),
        "porcentaje": round(total),
        "fases": fases,
        "bloqueos": d.get("bloqueos", []),
    }


def filas_de_tabla(texto: str, marca: str) -> list[list[str]]:
    """Las filas de la primera tabla Markdown que sigue a `marca`.

    Se corta en la primera línea que no empieza por `|`: así una tabla que
    crezca entra sola, y otra que venga después no se mezcla.
    """
    i = texto.find(marca)
    if i < 0:
        return []
    filas = []
    for linea in texto[i:].splitlines()[1:]:
        s = linea.strip()
        if not s.startswith("|"):
            if filas:
                break
            continue
        celdas = [c.strip() for c in s.strip("|").split("|")]
        # La fila de guiones que separa la cabecera no es un dato.
        if all(set(c) <= set("-: ") for c in celdas):
            continue
        filas.append(celdas)
    return filas


def leer_hallazgos() -> dict:
    """Los hallazgos de las pruebas, leídos de la tabla de la bitácora.

    ── POR QUÉ SE LEE EL MARKDOWN Y NO UN JSON PROPIO ──
    Un JSON propio sería más cómodo de leer y **sería una segunda copia**. La
    tabla de la bitácora es la que Dirección y yo editamos al reportar y al
    arreglar; un tablero con su propia lista se queda atrás a la segunda
    semana, y entonces enseña «0 abiertos» con uno abierto. Es el defecto que
    este proyecto ha tenido once veces de once formas distintas.
    """
    if not BITACORA.exists():
        return {"falta": f"no existe {BITACORA.relative_to(RAIZ)}"}
    texto = BITACORA.read_text(encoding="utf-8")
    filas = filas_de_tabla(texto, "| # | Gravedad | Pantalla |")
    hallazgos = []
    for celdas in filas:
        if len(celdas) < 5:
            continue
        limpio = lambda s: re.sub(r"\*\*|`", "", s).strip()  # noqa: E731
        hallazgos.append({
            "id": limpio(celdas[0]),
            "gravedad": limpio(celdas[1]),
            "pantalla": limpio(celdas[2]),
            "que": limpio(celdas[3]),
            "estado": limpio(celdas[4]),
        })
    if not hallazgos:
        return {"falta": "la tabla de hallazgos de la bitácora no se pudo leer"}
    return {"hallazgos": hallazgos}


def leer_tropiezos() -> list[str]:
    """Los títulos de los tropiezos de instalación ya documentados."""
    if not TROPIEZOS.exists():
        return []
    texto = TROPIEZOS.read_text(encoding="utf-8")
    return re.findall(r"(?m)^## \d+ · (.+)$", texto)


def medir_las_fases() -> dict:
    """Corre las diez fases del validador y cuenta lo que dice cada una.

    En modo `--rapido`, que omite las sondas: el tablero cuenta
    comprobaciones, y las sondas tardan quince minutos. Lo que se omite **se
    enseña como omitido**, nunca sumado a lo que pasa — es la regla que el
    propio validador sigue desde la Fase 0.
    """
    por_fase = {}
    for f in FASES:
        r = subprocess.run(
            [sys.executable, str(VALIDAR), "--fase", f, "--rapido"],
            capture_output=True, text=True, cwd=RAIZ,
        )
        salida = r.stdout + r.stderr
        m = re.search(
            r"(\d+) pasan\s+(\d+) fallan\s+(\d+) omitidos\s+de (\d+)", salida)
        if not m:
            por_fase[f] = {"error": "no se pudo leer el resultado"}
            continue
        por_fase[f] = {
            "pasan": int(m.group(1)),
            "fallan": int(m.group(2)),
            "omitidos": int(m.group(3)),
            "total": int(m.group(4)),
        }
    return {
        "cuando": datetime.datetime.now().isoformat(timespec="minutes"),
        "modo": "--rapido (las sondas se omiten)",
        "fases": por_fase,
    }


def leer_corrida() -> dict | None:
    if not CORRIDA.exists():
        return None
    return json.loads(CORRIDA.read_text(encoding="utf-8"))


# ───────────────────────────────────────────────────────────────────────────
# PINTAR
# ───────────────────────────────────────────────────────────────────────────

def e(s) -> str:
    """Escapa para HTML. Todo lo que viene de un archivo pasa por aquí."""
    return html.escape(str(s), quote=True)


#: Los tonos salen de `COLOR_SYSTEM.md`. Aquí sí van como literales porque
#: este archivo NO es un componente de la aplicación (§17 habla de `app/`):
#: es una herramienta nuestra que no se distribuye, y no puede importar los
#: tokens de Vue sin arrastrar el build entero.
CSS = """
:root {
  --fondo: #0f1115;
  --panel: #171a21;
  --panel-alto: #1e222b;
  --borde: #2a2f3a;
  --texto: #e8eaed;
  --apagado: #9aa1ad;
  --acento: #c9a227;
  --verde: #3fa96a;
  --amarillo: #d9a521;
  --rojo: #d9534f;
  --azul: #4a8fd4;
  --radio: 10px;
}
@media (prefers-color-scheme: light) {
  :root:not([data-tema="oscuro"]) {
    --fondo: #f7f8fa; --panel: #ffffff; --panel-alto: #f0f2f5;
    --borde: #d8dce3; --texto: #1a1d23; --apagado: #5c636e;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0; padding: 24px 16px 64px;
  background: var(--fondo); color: var(--texto);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  line-height: 1.5;
}
.lienzo { max-width: 1120px; margin: 0 auto; }
h1 { font-size: 28px; margin: 0 0 4px; letter-spacing: .5px; }
h2 { font-size: 18px; margin: 36px 0 12px; }
.sub { color: var(--apagado); margin: 0 0 8px; font-size: 14px; }
.panel {
  background: var(--panel); border: 1px solid var(--borde);
  border-radius: var(--radio); padding: 18px;
}
.rejilla { display: grid; gap: 14px; }
.r4 { grid-template-columns: repeat(4, 1fr); }
.r3 { grid-template-columns: repeat(3, 1fr); }
.r2 { grid-template-columns: repeat(2, 1fr); }
@media (max-width: 860px) { .r4, .r3, .r2 { grid-template-columns: 1fr 1fr; } }
@media (max-width: 560px) { .r4, .r3, .r2 { grid-template-columns: 1fr; } }

/* ── El número grande de cada contador ─────────────────────────────── */
.cifra { font-size: 34px; font-weight: 700; line-height: 1.1; }
.rotulo { color: var(--apagado); font-size: 13px; margin-top: 2px; }
.cifra.verde { color: var(--verde); }
.cifra.rojo { color: var(--rojo); }
.cifra.amarillo { color: var(--amarillo); }
.cifra.azul { color: var(--azul); }

/* ── El anillo del porcentaje ──────────────────────────────────────── */
.anillo { display: flex; align-items: center; gap: 22px; flex-wrap: wrap; }
.anillo svg { flex: 0 0 auto; }
.anillo .dentro { font-size: 13px; color: var(--apagado); }

/* ── Las barras de fase ────────────────────────────────────────────── */
.fase { display: grid; grid-template-columns: 56px 1fr 62px; gap: 10px;
        align-items: center; padding: 5px 0; }
.fase .id { font-weight: 700; font-size: 13px; }
.canal { background: var(--panel-alto); border-radius: 999px; height: 20px;
         overflow: hidden; position: relative; }
.relleno { height: 100%; border-radius: 999px; transition: width .4s; }
.pct { text-align: right; font-variant-numeric: tabular-nums; font-size: 13px; }
.fase .nombre { font-size: 12px; color: var(--apagado); }

/* ── Semáforos y etiquetas ─────────────────────────────────────────── */
.sem { display: inline-flex; align-items: center; gap: 6px;
       font-size: 12px; padding: 3px 9px; border-radius: 999px;
       border: 1px solid var(--borde); background: var(--panel-alto); }
.punto { width: 9px; height: 9px; border-radius: 50%; flex: 0 0 auto; }
.p-verde { background: var(--verde); }
.p-amarillo { background: var(--amarillo); }
.p-rojo { background: var(--rojo); }
.p-azul { background: var(--azul); }
.p-gris { background: var(--apagado); }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 9px 10px; border-bottom: 1px solid var(--borde); vertical-align: top; }
th { color: var(--apagado); font-size: 12px; font-weight: 600; text-transform: uppercase;
     letter-spacing: .4px; }
tr:last-child td { border-bottom: 0; }
td.angosta { white-space: nowrap; }

.alarma { border-left: 4px solid var(--rojo); }
.alarma.espera { border-left-color: var(--amarillo); }
.alarma.info { border-left-color: var(--azul); }
.alarma h3 { margin: 0 0 4px; font-size: 15px; }
.alarma p { margin: 0; font-size: 13px; color: var(--apagado); }
.quien { font-size: 12px; color: var(--acento); font-weight: 600; }

.falta {
  border: 1px dashed var(--rojo); border-radius: var(--radio);
  padding: 12px; color: var(--rojo); font-size: 13px;
}
/* Sin medir NO es un fallo: es que no se sabe. Van en colores distintos
   por la misma razón por la que el validador no suma las omitidas a las
   que pasan. */
.falta.sin-medir { border-color: var(--amarillo); color: var(--amarillo); }

/* Una fase al 0 % deja el canal vacío, y un canal vacío se lee igual que
   «no hay dato». Se marca para que diga que la cifra es cero. */
.canal.vacio { border: 1px dashed var(--borde); }
.pie { margin-top: 44px; color: var(--apagado); font-size: 12px;
       border-top: 1px solid var(--borde); padding-top: 14px; }
code { background: var(--panel-alto); padding: 1px 5px; border-radius: 4px;
       font-size: 12px; }
.marca { color: var(--acento); font-weight: 700; letter-spacing: 1px; }
@media (prefers-reduced-motion: reduce) {
  .relleno { transition: none; }
}
"""


def color_de(frac: float) -> str:
    """El color de una barra de avance. Tres tramos, no un degradado."""
    if frac >= 0.999:
        return "var(--verde)"
    if frac >= 0.5:
        return "var(--amarillo)"
    return "var(--rojo)"


def anillo(pct: int) -> str:
    """El anillo del porcentaje, en SVG puro: ni una dependencia."""
    r, grosor = 58, 13
    largo = 2 * 3.14159265 * r
    hecho = largo * pct / 100
    return f"""
<svg width="140" height="140" viewBox="0 0 140 140" role="img"
     aria-label="Avance total: {pct} por ciento">
  <circle cx="70" cy="70" r="{r}" fill="none"
          stroke="var(--panel-alto)" stroke-width="{grosor}"/>
  <circle cx="70" cy="70" r="{r}" fill="none"
          stroke="var(--acento)" stroke-width="{grosor}" stroke-linecap="round"
          stroke-dasharray="{hecho:.1f} {largo:.1f}"
          transform="rotate(-90 70 70)"/>
  <text x="70" y="70" text-anchor="middle" dominant-baseline="central"
        fill="var(--texto)" font-size="30" font-weight="700">{pct}%</text>
</svg>"""


def plural(n: int, singular: str, plural_: str) -> str:
    """«1 hallazgo abierto», no «1 hallazgos abiertos».

    El paréntesis —«1 bloqueante(s)»— es el mismo defecto que la 8.3c corrigió
    en «2 cuenta(s)» y que Remitentes ya pluraliza. Un tablero que Dirección
    enseña a alguien no puede escribir así.
    """
    return f"{n} {singular if n == 1 else plural_}"


def tono_de_gravedad(g: str) -> str:
    """El punto de color de un hallazgo, por su gravedad escrita."""
    if "Bloqueante" in g:
        return "p-rojo"
    if "Grave" in g:
        return "p-amarillo"
    if "Molesto" in g:
        return "p-amarillo"
    return "p-azul"


def esta_abierto(estado: str) -> bool:
    """¿Sigue pendiente este hallazgo?

    Se decide por lo que NO es: arreglado, hecho, declarado o descartado. Al
    revés —enumerar lo que sí está abierto— un estado nuevo entraría como
    cerrado, y el tablero enseñaría menos pendientes de los que hay. Es la
    misma razón por la que la comprobación de la cuenta de correo exige
    `connected` en vez de enumerar los estados que rechaza (Fase 5).
    """
    cerrado = ("Arreglado", "Hecho", "Declarado", "No era un defecto", "Cerrado")
    return not any(c.lower() in estado.lower() for c in cerrado)


def pintar(avance: dict, hallazgos: dict, tropiezos: list[str],
           corrida: dict | None) -> str:
    hoy = datetime.date.today().isoformat()
    partes: list[str] = []
    A = partes.append

    A("<!DOCTYPE html>")
    A('<html lang="es"><head><meta charset="utf-8">')
    A('<meta name="viewport" content="width=device-width, initial-scale=1">')
    A("<title>Tablero de ARLES RELAY I</title>")
    A(f"<style>{CSS}</style></head><body><div class='lienzo'>")

    # ── Cabecera ──────────────────────────────────────────────────────
    A("<h1><span class='marca'>ARLES RELAY I</span> · tablero</h1>")
    A(f"<p class='sub'>Generado el {hoy}. "
      "Todas las cifras salen de los archivos del proyecto; ninguna está "
      "escrita en este tablero.</p>")

    if "falta" in avance:
        A(f"<div class='falta'>No se pudo leer el avance: {e(avance['falta'])}</div>")
        A("</div></body></html>")
        return "\n".join(partes)

    # ── Los cuatro contadores de arriba ───────────────────────────────
    lista = hallazgos.get("hallazgos", [])
    abiertos = [h for h in lista if esta_abierto(h["estado"])]
    bloqueantes = [h for h in abiertos if "Bloqueante" in h["gravedad"]]
    bloqueos = avance["bloqueos"]

    total_pasan = total_fallan = total_omit = 0
    if corrida:
        for d in corrida["fases"].values():
            if "error" in d:
                continue
            total_pasan += d["pasan"]
            total_fallan += d["fallan"]
            total_omit += d["omitidos"]

    A("<div class='rejilla r4'>")
    A("<div class='panel'><div class='anillo'>" + anillo(avance["porcentaje"]) +
      f"<div class='dentro'>de la v{e(avance['version'])}<br>"
      f"<small>fuente: avance.json, {e(avance['actualizado'])}</small></div>"
      "</div></div>")

    if corrida:
        clase = "verde" if total_fallan == 0 else "rojo"
        A(f"<div class='panel'><div class='cifra {clase}'>{total_pasan}</div>"
          f"<div class='rotulo'>comprobaciones en verde<br>"
          f"<small>{total_fallan} en rojo · {total_omit} omitidas</small></div></div>")
    else:
        A("<div class='panel'><div class='cifra amarillo'>sin medir</div>"
          "<div class='rotulo'>comprobaciones<br><small>corre "
          "<code>generar.py --medir</code></small></div></div>")

    clase = "rojo" if bloqueantes else ("amarillo" if abiertos else "verde")
    A(f"<div class='panel'><div class='cifra {clase}'>{len(abiertos)}</div>"
      f"<div class='rotulo'>{'hallazgo abierto' if len(abiertos) == 1 else 'hallazgos abiertos'}"
      f"<br><small>{plural(len(bloqueantes), 'bloqueante', 'bloqueantes')} · "
      f"{len(lista)} en total</small></div></div>")

    A(f"<div class='panel'><div class='cifra amarillo'>{len(bloqueos)}</div>"
      f"<div class='rotulo'>{'espera' if len(bloqueos) == 1 else 'esperan'} a alguien "
      "de fuera<br><small>ninguno impide construir</small></div></div>")
    A("</div>")

    # ── La frescura de la medición ────────────────────────────────────
    if corrida:
        cuando = corrida["cuando"][:16].replace("T", " a las ")
        dias = (datetime.date.today()
                - datetime.date.fromisoformat(corrida["cuando"][:10])).days
        if dias >= DIAS_PARA_ENVEJECER:
            A(f"<p class='sub'><span class='sem'><span class='punto p-amarillo'>"
              f"</span>medido hace {dias} días</span> — las comprobaciones se "
              f"midieron el {e(cuando)} y pueden no reflejar el código de hoy. "
              "Vuelve a correr <code>generar.py --medir</code>.</p>")
        else:
            A(f"<p class='sub'><span class='sem'><span class='punto p-verde'>"
              f"</span>medido el {e(cuando)}</span> · {e(corrida['modo'])}</p>")

    # ── Avance por fase ───────────────────────────────────────────────
    A("<h2>Avance por fase</h2>")
    A("<p class='sub'>El largo de cada barra es lo construido y verificado de "
      "esa fase. El <em>peso</em> es su tamaño dentro de la versión, estimado: "
      "se dice porque presentarlo como medición exacta sería mentir.</p>")
    A("<div class='panel'>")
    for f in avance["fases"]:
        frac = float(f["hecho"])
        vacio = " vacio" if frac <= 0 else ""
        A("<div class='fase'>"
          f"<div><div class='id'>{e(f['id'])}</div>"
          f"<div class='nombre'>peso {e(f['peso'])}</div></div>"
          f"<div><div class='canal{vacio}'>"
          f"<div class='relleno' style='width:{frac*100:.0f}%;"
          f"background:{color_de(frac)}'></div></div>"
          f"<div class='nombre'>{e(f['nombre'])}</div></div>"
          f"<div class='pct'>{frac*100:.0f}%</div>"
          "</div>")
    A("</div>")

    # ── Las comprobaciones, fase a fase ──────────────────────────────
    A("<h2>Comprobaciones del validador</h2>")
    if not corrida:
        # ── SIN MEDIR VA EN AMARILLO, NO EN ROJO ──
        #
        # Rojo quiere decir «algo falla». Lo que pasa aquí es que no se sabe,
        # y son dos cosas distintas: es la misma razón por la que el validador
        # enseña las omitidas en su columna y no sumadas a las que pasan.
        A("<div class='falta sin-medir'>Nunca se han medido desde este tablero. "
          "Corre <code>python3 herramientas/tablero/generar.py --medir</code>. "
          "No se pone un cero porque un cero se lee como «ninguna falla», y "
          "lo que hay es que no se sabe.</div>")
    else:
        A("<div class='panel'><table><thead><tr><th>Fase</th><th>En verde</th>"
          "<th>En rojo</th><th>Omitidas</th><th>Estado</th></tr></thead><tbody>")
        for f in FASES:
            d = corrida["fases"].get(f, {})
            if "error" in d or not d:
                A(f"<tr><td class='angosta'><strong>{e(f)}</strong></td>"
                  "<td colspan='4'><span class='sem'><span class='punto p-gris'>"
                  "</span>no se pudo leer</span></td></tr>")
                continue
            if d["fallan"]:
                sem = ("<span class='sem'><span class='punto p-rojo'></span>"
                       f"{d['fallan']} en rojo</span>")
            elif d["omitidos"]:
                sem = ("<span class='sem'><span class='punto p-amarillo'></span>"
                       f"{d['omitidos']} sin medir aquí</span>")
            else:
                sem = "<span class='sem'><span class='punto p-verde'></span>entera</span>"
            A(f"<tr><td class='angosta'><strong>{e(f)}</strong></td>"
              f"<td>{d['pasan']}</td><td>{d['fallan']}</td>"
              f"<td>{d['omitidos']}</td><td>{sem}</td></tr>")
        A("</tbody></table></div>")
        A("<p class='sub'>Las omitidas <strong>no son pasadas</strong>, y por "
          "eso van en su columna: son las sondas que abren un navegador de "
          "verdad —se omiten en <code>--rapido</code>— y las que necesitan el "
          "llavero del sistema, que un contenedor no tiene.</p>")

    # ── Hallazgos de las pruebas ─────────────────────────────────────
    A("<h2>Hallazgos de las pruebas de Dirección</h2>")
    if "falta" in hallazgos:
        A(f"<div class='falta'>{e(hallazgos['falta'])}</div>")
    else:
        A("<div class='panel'><table><thead><tr><th></th><th>#</th>"
          "<th>Pantalla</th><th>Qué pasa</th><th>Estado</th></tr></thead><tbody>")
        for h in lista:
            A(f"<tr><td class='angosta'><span class='punto "
              f"{tono_de_gravedad(h['gravedad'])}' title='{e(h['gravedad'])}'"
              " style='display:inline-block'></span></td>"
              f"<td class='angosta'><strong>{e(h['id'])}</strong></td>"
              f"<td class='angosta'>{e(h['pantalla'])}</td>"
              f"<td>{e(h['que'])}</td>"
              f"<td class='angosta'>{e(h['estado'])}</td></tr>")
        A("</tbody></table></div>")
        A("<p class='sub'>Rojo: bloqueante. Amarillo: grave o molesto. "
          "Azul: propuesta o requisito. La tabla sale de "
          "<code>BITACORA-DE-PRUEBAS.md</code> — se edita ahí, no aquí.</p>")

    # ── Lo que espera a alguien de fuera ─────────────────────────────
    A("<h2>Esperan a alguien de fuera</h2>")
    A("<p class='sub'>Ninguno impide construir. Los cuatro primeros impiden "
      "entregar la versión.</p>")
    A("<div class='rejilla r2'>")
    for b in bloqueos:
        A(f"<div class='panel alarma espera'><h3>{e(b['id'])}</h3>"
          f"<p>{e(b['que'])}</p>"
          f"<p style='margin-top:8px'><span class='quien'>{e(b['quien'])}</span> — "
          f"{e(b['bloquea'])}</p></div>")
    A("</div>")

    # ── Tropiezos ya documentados ────────────────────────────────────
    A("<h2>Tropiezos de instalación ya documentados</h2>")
    if not tropiezos:
        A("<div class='falta'>No se pudo leer "
          "<code>REGISTRO-DE-TROPIEZOS.md</code>.</div>")
    else:
        A("<p class='sub'>Cada uno ocurrió de verdad y tiene su solución "
          "escrita. Si vuelve a pasar uno de éstos, la respuesta ya está.</p>")
        # ── NI UN PUNTO DE COLOR QUE NO SIGNIFIQUE NADA ──
        #
        # La primera versión ponía un punto verde delante de cada tropiezo.
        # Verde quiere decir «esto está bien» y lo que decía era «éste es el
        # número 1»: un indicador que no indica. Es el defecto que la revisión
        # del 29/09/2026 encontró con las ocho insignias neutras que llevaban
        # un reloj sin esperar nada.
        A("<div class='panel'><table><tbody>")
        for i, t in enumerate(tropiezos, 1):
            A(f"<tr><td class='angosta' style='color:var(--apagado);"
              f"width:28px'>{i}</td><td>{e(t)}</td></tr>")
        A("</tbody></table></div>")

    # ── Pie ──────────────────────────────────────────────────────────
    A("<div class='pie'>")
    A("<p><strong>De dónde sale cada cosa.</strong> Porcentaje, fases y "
      "bloqueos: <code>documentacion/07-entrega/avance.json</code>. "
      "Hallazgos: la tabla de <code>documentacion/06-calidad/"
      "BITACORA-DE-PRUEBAS.md</code>. Tropiezos: los títulos de "
      "<code>documentacion/07-entrega/REGISTRO-DE-TROPIEZOS.md</code>. "
      "Comprobaciones: una corrida de <code>validar.py</code> guardada en "
      "<code>tablero/corrida.json</code>.</p>")
    A("<p>Para rehacerlo: <code>python3 herramientas/tablero/generar.py "
      "--medir</code>. Sin <code>--medir</code> se reutiliza la última "
      "medición y se enseña su fecha.</p>")
    A("</div></div></body></html>")
    return "\n".join(partes)


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--medir", action="store_true",
                   help="corre las diez fases del validador antes de pintar")
    a = p.parse_args()

    if a.medir:
        print("Corriendo las diez fases (--rapido)…")
        corrida = medir_las_fases()
        CORRIDA.parent.mkdir(parents=True, exist_ok=True)
        CORRIDA.write_text(
            json.dumps(corrida, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8")
        rojas = sum(d.get("fallan", 0) for d in corrida["fases"].values())
        print(f"  medido: {sum(d.get('pasan', 0) for d in corrida['fases'].values())}"
              f" en verde, {rojas} en rojo")
    else:
        corrida = leer_corrida()
        if corrida is None:
            print("Sin medición previa. El tablero lo dirá; usa --medir para medir.")

    avance = leer_avance()
    hallazgos = leer_hallazgos()
    tropiezos = leer_tropiezos()

    SALIDA.parent.mkdir(parents=True, exist_ok=True)
    SALIDA.write_text(pintar(avance, hallazgos, tropiezos, corrida),
                      encoding="utf-8")
    print(f"✓ {SALIDA.relative_to(RAIZ)}")
    if "falta" not in avance:
        print(f"  avance {avance['porcentaje']} % · "
              f"{len(hallazgos.get('hallazgos', []))} hallazgos · "
              f"{len(avance['bloqueos'])} bloqueos · {len(tropiezos)} tropiezos")
    return 0


if __name__ == "__main__":
    sys.exit(main())
