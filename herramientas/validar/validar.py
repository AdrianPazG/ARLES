#!/usr/bin/env python3
"""Valida que una fase de ARLES RELAY está bien estructurada, compilada y probada.

    ./validar.py                  valida todas las fases cerradas
    ./validar.py --fase 1         valida solo la Fase 1
    ./validar.py --rapido         omite lo que tarda (compilación de Tauri, build de producción)
    ./validar.py --json informe.json   además escribe el resultado en JSON

Cada comprobación dice **qué verifica y por qué importa**, para que un fallo se
entienda sin abrir el código. Código de salida 0 si todo pasa, 1 si algo falla.

Sin dependencias externas: solo stdlib de Python 3.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import time

RAIZ = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

VERDE, ROJO, AMBAR, GRIS, NEGRITA, FIN = (
    "\033[32m",
    "\033[31m",
    "\033[33m",
    "\033[90m",
    "\033[1m",
    "\033[0m",
)
if not sys.stdout.isatty() or os.environ.get("NO_COLOR"):
    VERDE = ROJO = AMBAR = GRIS = NEGRITA = FIN = ""


class Resultado:
    def __init__(self):
        self.checks = []

    def añadir(self, fase, nombre, estado, detalle="", porque="", segundos=0.0):
        self.checks.append(
            {
                "fase": fase,
                "nombre": nombre,
                "estado": estado,  # ok | falla | omitido
                "detalle": detalle,
                "porque": porque,
                "segundos": round(segundos, 1),
            }
        )
        icono = {"ok": f"{VERDE}✓{FIN}", "falla": f"{ROJO}✗{FIN}", "omitido": f"{AMBAR}–{FIN}"}[estado]
        t = f"{GRIS}{segundos:5.1f}s{FIN}" if segundos >= 0.1 else f"{GRIS}      {FIN}"
        print(f"  {icono} {t}  {nombre}")
        if estado == "falla":
            if porque:
                print(f"      {AMBAR}por qué importa:{FIN} {porque}")
            for linea in detalle.strip().splitlines()[-15:]:
                print(f"      {GRIS}{linea}{FIN}")

    @property
    def fallos(self):
        return [c for c in self.checks if c["estado"] == "falla"]

    @property
    def omitidos(self):
        return [c for c in self.checks if c["estado"] == "omitido"]


R = Resultado()


def corre(cmd, cwd=RAIZ, timeout=1800):
    t0 = time.time()
    try:
        p = subprocess.run(
            cmd, cwd=cwd, shell=isinstance(cmd, str), capture_output=True,
            text=True, timeout=timeout,
        )
        return p.returncode, (p.stdout + p.stderr), time.time() - t0
    except subprocess.TimeoutExpired:
        return 124, f"agotó el tiempo tras {timeout}s", time.time() - t0
    except FileNotFoundError as e:
        return 127, str(e), time.time() - t0


def check_cmd(fase, nombre, cmd, porque, cwd=RAIZ, timeout=1800):
    codigo, salida, seg = corre(cmd, cwd, timeout)
    R.añadir(fase, nombre, "ok" if codigo == 0 else "falla", salida, porque, seg)
    return codigo == 0


def check(fase, nombre, condicion, porque, detalle=""):
    R.añadir(fase, nombre, "ok" if condicion else "falla", detalle, porque)
    return condicion


def omitir(fase, nombre, motivo):
    R.añadir(fase, nombre, "omitido", "", motivo)


def titulo(t):
    print(f"\n{NEGRITA}{t}{FIN}")
    print(GRIS + "─" * 70 + FIN)


def leer(*partes):
    ruta = os.path.join(RAIZ, *partes)
    if not os.path.exists(ruta):
        return None
    with open(ruta, encoding="utf-8") as f:
        return f.read()


# ─── Fase 0 · Documentación ──────────────────────────────────────────────────

def fase_0(rapido):
    titulo("FASE 0 · Discovery, auditoría y arquitectura")

    esperados = [
        "documentacion/00-INDICE.md",
        "documentacion/01-producto/VISION_Y_ALCANCE.md",
        "documentacion/01-producto/DECISIONES_DE_DIRECCION.md",
        "documentacion/02-auditoria/AUDITORIA_DISCOVERY.md",
        "documentacion/02-auditoria/MATRIZ_DE_RIESGOS.md",
        "documentacion/03-arquitectura/ARQUITECTURA.md",
        "documentacion/03-arquitectura/MODELO_DE_DATOS.md",
        "documentacion/03-arquitectura/MOTOR_DE_EJECUCION.md",
        "documentacion/04-seguridad/THREAT_MODEL.md",
        "documentacion/05-diseno/COLOR_SYSTEM.md",
        "documentacion/06-calidad/ESTRATEGIA_QA.md",
        "documentacion/07-entrega/ROADMAP.md",
        "documentacion/08-legal/LICENCIAS_DE_TERCEROS.md",
        "CHANGELOG.md",
    ]
    faltan = [p for p in esperados if not os.path.exists(os.path.join(RAIZ, p))]
    check(0, "El cuerpo documental está completo", not faltan,
          "Sin estos documentos no hay trazabilidad de por qué se tomó cada decisión.",
          "Faltan:\n" + "\n".join(faltan))

    adrs = sorted(
        f for f in os.listdir(os.path.join(RAIZ, "documentacion/03-arquitectura/adr"))
        if f.endswith(".md")
    )
    check(0, f"Los 12 ADRs están presentes ({len(adrs)})", len(adrs) >= 12,
          "Cada decisión con coste debe tener su registro, o se vuelve a discutir cada seis meses.")

    # Enlaces internos
    import re
    rotos = []
    total = 0
    for raiz, dirs, ficheros in os.walk(RAIZ):
        if ".git" in raiz or "node_modules" in raiz or "/target" in raiz:
            continue
        for f in ficheros:
            if not f.endswith(".md"):
                continue
            ruta = os.path.join(raiz, f)
            with open(ruta, encoding="utf-8") as fh:
                for m in re.finditer(r"\[([^\]]+)\]\(([^)]+)\)", fh.read()):
                    enlace = m.group(2)
                    if enlace.startswith(("http", "#", "mailto")):
                        continue
                    total += 1
                    destino = os.path.normpath(
                        os.path.join(raiz, enlace.split("#")[0])
                    )
                    if not os.path.exists(destino):
                        rotos.append(f"{os.path.relpath(ruta, RAIZ)} -> {enlace}")
    check(0, f"Los enlaces internos resuelven ({total} comprobados)", not rotos,
          "Un enlace roto hace que nadie siga el rastro de una decisión.",
          "\n".join(rotos))

    # Contradicciones cerradas
    indice = leer("documentacion", "00-INDICE.md") or ""
    check(0, "Las contradicciones del brief están cerradas por escrito",
          "§37" in indice and "§69" in indice and "T-4" in indice,
          "El brief se contradecía en tres puntos; si no consta la resolución, alguien los reabrirá.")


# ─── Fase 1 · Cimientos ──────────────────────────────────────────────────────

def fase_1(rapido):
    titulo("FASE 1 · Cimientos")

    # ── Estructura ──
    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("Cargo.toml", "Sin workspace, los crates no comparten versión ni lints."),
        ("crates/arles-core/src/lib.rs", "El dominio sin I/O es lo que permite probar sin infraestructura."),
        ("crates/arles-db/migrations/V1__esquema_inicial.sql", "El esquema es donde viven las garantías del producto."),
        ("crates/arles-app/tauri.conf.json", "Sin configuración de Tauri no hay aplicación de escritorio."),
        ("crates/arles-app/capabilities/principal.json", "Las capabilities son el control de seguridad central de ADR-0001."),
        ("app/package.json", "El frontend es la superficie de presentación."),
        ("herramientas/design-tokens/tokens.json", "Fuente única del color (§17)."),
        (".github/workflows/ci.yml", "Sin CI, las verificaciones son opcionales en la práctica."),
        ("deny.toml", "La política de licencias debe hacerse cumplir, no documentarse."),
    ]:
        check(1, f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    # ── Fronteras de arquitectura ──
    print(f"{GRIS}  fronteras de arquitectura{FIN}")

    core = ""
    core_dir = os.path.join(RAIZ, "crates/arles-core/src")
    for f in os.listdir(core_dir):
        core += leer("crates/arles-core/src", f) or ""
    check(1, "arles-core no hace I/O",
          not any(p in core for p in ["std::fs", "std::net", "rusqlite", "reqwest", "tokio::net"]),
          "Regla de frontera 3.2: si el dominio toca I/O, deja de poder probarse sin infraestructura.")

    core_toml = leer("crates/arles-core", "Cargo.toml") or ""
    check(1, "arles-core no depende de arles-db",
          "arles-db" not in core_toml,
          "Las dependencias solo apuntan hacia dentro; al revés, el monolito modular deja de ser modular.")

    cap = json.loads(leer("crates/arles-app", "capabilities/principal.json") or "{}")
    permisos = cap.get("permissions", [])
    prohibidos = [p for p in permisos if any(
        x in str(p) for x in ["shell", "fs:", "http:", "process"]
    )]
    check(1, "Las capabilities de Tauri no exponen shell, fs ni http", not prohibidos,
          "Es el argumento que decidió Tauri sobre Electron: la capacidad ausente es más fuerte que la bien configurada.",
          f"Permisos problemáticos: {prohibidos}")

    conf = json.loads(leer("crates/arles-app", "tauri.conf.json") or "{}")
    csp = conf.get("app", {}).get("security", {}).get("csp", {})
    csp_txt = json.dumps(csp)
    check(1, "La CSP no permite unsafe-inline ni unsafe-eval en scripts",
          "'unsafe-eval'" not in csp_txt and "'unsafe-inline'" not in csp.get("script-src", ""),
          "Una CSP laxa devuelve a la webview la capacidad de ejecutar lo que le inyecten.")

    eslint = leer("app", "eslint.config.js") or ""
    check(1, "El frontend tiene prohibido fetch y localStorage",
          "fetch" in eslint and "localStorage" in eslint,
          "Regla de frontera 3.1 y §30: el I/O y los secretos viven en Rust, no en la webview.")

    # ── Rust ──
    print(f"{GRIS}  rust{FIN}")
    check_cmd(1, "cargo fmt", ["cargo", "fmt", "--all", "--check"],
              "Un formato inconsistente convierte cada diff en ruido.")
    check_cmd(1, "cargo clippy -D warnings",
              ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
              "§138: sin unwrap ni panic indiscriminados en producción.")
    check_cmd(1, "cargo test --workspace",
              ["cargo", "test", "--workspace", "--all-targets"],
              "Los tests codifican las decisiones documentadas; si fallan, una garantía del producto se rompió.")
    check_cmd(1, "doctests", ["cargo", "test", "--workspace", "--doc"],
              "Los ejemplos de la documentación deben compilar y funcionar.")

    # ── Frontend ──
    print(f"{GRIS}  frontend{FIN}")
    app = os.path.join(RAIZ, "app")
    if not os.path.isdir(os.path.join(app, "node_modules")):
        omitir(1, "frontend", "Falta node_modules: ejecuta `npm ci` en app/")
    else:
        check_cmd(1, "eslint", ["npm", "run", "--silent", "lint"],
                  "Las reglas de frontera se hacen cumplir aquí mecánicamente.", cwd=app)
        check_cmd(1, "comprobación de tipos", ["npm", "run", "--silent", "typecheck"],
                  "TypeScript estricto: `any` desactiva las comprobaciones justo en la superficie más expuesta.", cwd=app)
        check_cmd(1, "vitest", ["npm", "run", "--silent", "test"],
                  "Verifican que ningún estado diga «entregado» y que todo error tenga sus tres partes.", cwd=app)
        if rapido:
            omitir(1, "build de producción", "--rapido")
        else:
            check_cmd(1, "build de producción", ["npm", "run", "--silent", "build"],
                      "Si no compila, no hay nada que empaquetar en la aplicación.", cwd=app)
        check_cmd(1, "npm audit (lo que se distribuye)",
                  ["npm", "audit", "--omit=dev", "--audit-level=moderate"],
                  "Las dependencias que se empaquetan llegan a la máquina del cliente.", cwd=app)

    # ── Tokens ──
    print(f"{GRIS}  sistema de diseño{FIN}")
    check_cmd(1, "contraste WCAG de los tokens",
              [sys.executable, "herramientas/design-tokens/generar.py", "--verificar"],
              "La accesibilidad AA es requisito (§19), y una regresión de color es invisible a ojo.")

    codigo, _, _ = corre([sys.executable, "herramientas/design-tokens/generar.py", "--css"])
    codigo_diff, salida_diff, _ = corre(["git", "diff", "--exit-code",
                                         "herramientas/design-tokens/dist/"])
    check(1, "El CSS generado coincide con tokens.json", codigo == 0 and codigo_diff == 0,
          "§17: si el CSS se edita a mano, deja de haber una fuente única de color.",
          salida_diff)

    # ── Iconos ──
    faltan_iconos = [
        n for n in ("32x32.png", "128x128.png", "128x128@2x.png",
                    "icon.png", "icon.ico", "icon.icns")
        if not os.path.exists(os.path.join(RAIZ, "crates/arles-app/icons", n))
    ]
    check(1, "Los iconos de la aplicación están generados", not faltan_iconos,
          "Sin ellos el empaquetado falla, y el .ico y el .icns son requisitos de Windows y macOS.",
          f"Faltan: {faltan_iconos}")

    # ── Tauri ──
    print(f"{GRIS}  shell de escritorio{FIN}")
    tiene_webkit = corre(["pkg-config", "--exists", "webkit2gtk-4.1"])[0] == 0
    es_linux = sys.platform.startswith("linux")

    if rapido:
        omitir(1, "compilación del shell de Tauri", "--rapido")
    elif es_linux and not tiene_webkit:
        omitir(1, "compilación del shell de Tauri",
               "Falta webkit2gtk-4.1. Instala: apt-get install -y libwebkit2gtk-4.1-dev")
    else:
        check_cmd(1, "cargo build -p arles-app", ["cargo", "build", "-p", "arles-app"],
                  "Es la prueba de que el shell, el dominio y el llavero encajan de verdad.")

    # El arranque tiene dos ramas y ambas importan. La de rechazo se prueba
    # siempre —en un contenedor sin llavero es la que ocurre de forma natural—
    # pero un «todo verde» que solo ejercita el camino de error no es una
    # validación, es una coincidencia.
    con_llavero = os.path.join(RAIZ, "herramientas/validar/con-llavero.sh")
    tiene_receta = (
        not es_linux
        or (shutil.which("dbus-run-session") and shutil.which("gnome-keyring-daemon"))
    )
    if rapido:
        omitir(1, "arranque con llavero disponible", "--rapido")
    elif not tiene_receta:
        omitir(1, "arranque con llavero disponible",
               "Faltan dbus-run-session y gnome-keyring-daemon. "
               "Instala: apt-get install -y gnome-keyring dbus-x11")
    else:
        codigo, salida, seg = corre(
            [con_llavero, "cargo", "test", "-p", "arles-app",
             "--test", "arranque", "--", "--nocapture"]
        )
        rama_feliz = "RAMA: llavero disponible" in salida
        R.añadir(
            1, "arranque con llavero disponible (ruta feliz)",
            "ok" if codigo == 0 and rama_feliz else "falla",
            salida,
            "Si no se ejercita esta rama, nunca se prueba que la aplicación "
            "arranque de verdad: solo que sabe rechazar.",
            seg,
        )

    codigo, salida, seg = corre(
        ["cargo", "test", "-p", "arles-app", "--test", "arranque", "--", "--nocapture"],
        timeout=600,
    )
    R.añadir(
        1, "arranque sin llavero: se rechaza y no queda base en disco",
        "ok" if codigo == 0 else "falla", salida,
        "ADR-0011: sin degradación a texto plano. Es la decisión de seguridad "
        "central del producto y la única forma de comprobarla es ejecutarla.",
        seg,
    )


FASES = {0: fase_0, 1: fase_1}


def main():
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    p.add_argument("--fase", type=int, choices=sorted(FASES), help="valida solo esa fase")
    p.add_argument("--rapido", action="store_true", help="omite lo que tarda")
    p.add_argument("--json", metavar="RUTA", help="escribe el informe en JSON")
    a = p.parse_args()

    print(f"{NEGRITA}Validación de ARLES RELAY I · v1.2.0{FIN}")
    print(f"{GRIS}{RAIZ}{FIN}")

    t0 = time.time()
    for n in ([a.fase] if a.fase is not None else sorted(FASES)):
        FASES[n](a.rapido)

    total = len(R.checks)
    ok = total - len(R.fallos) - len(R.omitidos)
    print(f"\n{NEGRITA}Resumen{FIN}")
    print(GRIS + "─" * 70 + FIN)
    print(f"  {VERDE}{ok} pasan{FIN}   {ROJO}{len(R.fallos)} fallan{FIN}"
          f"   {AMBAR}{len(R.omitidos)} omitidos{FIN}   "
          f"{GRIS}de {total} en {time.time() - t0:.0f}s{FIN}")

    if R.omitidos:
        print(f"\n{AMBAR}Omitidos{FIN} (no son fallos, pero tampoco están verificados):")
        for c in R.omitidos:
            print(f"  – {c['nombre']}: {c['porque']}")

    if R.fallos:
        print(f"\n{ROJO}Fallos{FIN}:")
        for c in R.fallos:
            print(f"  ✗ Fase {c['fase']} · {c['nombre']}")

    if a.json:
        with open(a.json, "w", encoding="utf-8") as f:
            json.dump(
                {"ok": ok, "fallos": len(R.fallos), "omitidos": len(R.omitidos),
                 "checks": R.checks},
                f, ensure_ascii=False, indent=2,
            )
        print(f"\n{GRIS}Informe en {a.json}{FIN}")

    if R.fallos:
        return 1
    print(f"\n{VERDE}Todo lo verificable pasa.{FIN}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
