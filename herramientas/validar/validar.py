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
import hashlib
import json
import os
import re
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
    # El umbral sube cuando se añade un ADR: bajar de aquí significa que alguien
    # borró un registro de decisión, y ésa es la forma en que una decisión se
    # vuelve a discutir desde cero seis meses después.
    check(0, f"Los ADR están presentes ({len(adrs)})", len(adrs) >= 14,
          "Cada decisión con coste debe tener su registro, o se vuelve a discutir cada seis meses.",
          f"se esperaban al menos 14, hay {len(adrs)}")

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
    csp = conf.get("app", {}).get("security", {}).get("csp")
    # Tauri admite la CSP como objeto o como una sola cadena. Suponer el objeto
    # hacía que el validador muriera con AttributeError en vez de informar.
    if isinstance(csp, str):
        script_src = csp
        csp_txt = csp
    elif isinstance(csp, dict):
        script_src = str(csp.get("script-src", ""))
        csp_txt = json.dumps(csp)
    else:
        script_src = csp_txt = ""
    check(1, "La CSP no permite unsafe-inline ni unsafe-eval en scripts",
          bool(csp_txt) and "'unsafe-eval'" not in csp_txt
          and "'unsafe-inline'" not in script_src,
          "Una CSP laxa devuelve a la webview la capacidad de ejecutar lo que le "
          "inyecten. `style-src` sí lleva unsafe-inline —los estilos scoped de Vue "
          "lo exigen— y está registrado como riesgo aceptado en THREAT_MODEL.md §7.")

    # Se comprueba **ejecutando eslint** contra código que viola la regla, no
    # buscando texto en la configuración. Una búsqueda de texto pasa mientras la
    # palabra aparezca en cualquier sitio —un comentario, otro mensaje— y por
    # eso no detecta que alguien desactivó la regla.
    # Hallazgo F1 de la revisión de la Fase 1: un CASCADE desde `contact` borraba
    # el registro de envío, así que reimportar un contacto abría la puerta a
    # reenviarle. Los tests de `arles-db` lo cubren; esto lo detecta en la
    # migración siguiente, antes de que haya que razonarlo otra vez.
    esquema = leer("crates/arles-db/migrations", "V1__esquema_inicial.sql") or ""
    check(1, "Borrar un contacto no borra el registro de envío",
          "contact_email" in esquema
          and "ON DELETE SET NULL" in esquema
          and "idx_attempt_unique\n    ON message_attempt(campaign_id, contact_email)"
          in esquema,
          "Si el intento cascadea con el contacto, se pierden a la vez la "
          "auditoría y la protección contra duplicados: reimportar al contacto "
          "permitiría enviarle otra vez (§55, ADR-0004).")

    app_dir = os.path.join(RAIZ, "app")
    if not os.path.isdir(os.path.join(app_dir, "node_modules")):
        omitir(1, "El frontend tiene prohibido fetch y localStorage",
               "Falta node_modules: ejecuta `npm ci` en app/")
    else:
        cebo = os.path.join(app_dir, "src", "__validacion_frontera.ts")
        try:
            with open(cebo, "w", encoding="utf-8") as f:
                f.write(
                    "// Archivo temporal del validador. Debe ser rechazado por eslint.\n"
                    "export async function viola() {\n"
                    "  const r = await fetch('https://ejemplo.com')\n"
                    "  localStorage.setItem('token', 'secreto')\n"
                    "  return r\n"
                    "}\n"
                )
            codigo, salida, seg = corre(
                ["npx", "eslint", "src/__validacion_frontera.ts"], cwd=app_dir, timeout=300
            )
            # Se cuentan los errores en vez de buscar palabras en los mensajes:
            # el texto está en español y se puede reescribir, pero las dos
            # violaciones del cebo tienen que seguir produciendo dos errores.
            errores = sum(1 for l in salida.splitlines() if " error " in l)
            R.añadir(
                1, "El frontend tiene prohibido fetch y localStorage",
                "ok" if codigo != 0 and errores >= 2 else "falla",
                salida or "eslint aceptó código que usa fetch y localStorage",
                "Regla de frontera 3.1 y §30: el I/O y los secretos viven en Rust, "
                "no en la webview. Si eslint no lo rechaza, la frontera es decorativa.",
                seg,
            )
        finally:
            if os.path.exists(cebo):
                os.remove(cebo)

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

    # ── Documentación ──
    print(f"{GRIS}  documentación{FIN}")
    informe_para_direccion(1, "FASE-01-PARA-DIRECCION")


# ─── Fase 2 · Design System ──────────────────────────────────────────────────

# Sin acentos graves ni comillas: el nombre de un componente dentro de un
# comentario no debe contar como uso de un token.
RE_COMENTARIO_CSS = re.compile(r"/\*.*?\*/", re.S)
RE_COMENTARIO_JS = re.compile(r"^\s*//.*$", re.M)
RE_HEX = re.compile(r"#[0-9a-fA-F]{3,8}\b")
RE_MS = re.compile(r"\b\d+ms\b")
# Píxeles sueltos de 4 en adelante. Los de 1 a 3 son filetes y desplazamientos
# de borde, no decisiones de diseño: ver §2.1 del DESIGN_SYSTEM.
RE_PX = re.compile(r"[^-0-9a-z(]([4-9]|[1-9][0-9]+)px")
RE_USO_DE_TOKEN = re.compile(r"var\(\s*(--arles-[a-z0-9-]+)")
RE_DEF_DE_TOKEN = re.compile(r"^\s*(--arles-[a-z0-9-]+)\s*:", re.M)


def sin_comentarios(texto):
    return RE_COMENTARIO_JS.sub("", RE_COMENTARIO_CSS.sub("", texto))


def primitivas():
    carpeta = os.path.join(RAIZ, "app/src/design/componentes")
    if not os.path.isdir(carpeta):
        return []
    return sorted(
        os.path.join(carpeta, n)
        for n in os.listdir(carpeta)
        if n.endswith(".vue")
    )


def informe_para_direccion(fase, base):
    """Comprueba el informe sin tecnicismos de una fase, y que su PDF esté al día.

    Sirve a cualquier fase: el nombre del archivo es el único parámetro. Antes
    vivía dentro de `fase_1` con las rutas escritas dentro, así que la Fase 2
    habría necesitado una copia — y una copia de una comprobación es una
    comprobación que se queda atrás en una de las dos.
    """
    md = os.path.join(RAIZ, f"documentacion/09-fases/{base}.md")
    pdf = os.path.join(RAIZ, f"documentacion/09-fases/{base}.pdf")
    huella = pdf + ".sha256"

    check(
        fase, "existe el informe de fase para Dirección (md y pdf)",
        os.path.exists(md) and os.path.exists(pdf),
        "Una fase que sólo se puede entender leyendo código no está entregada: "
        "Dirección aprueba lo que puede leer.",
        f"falta {os.path.relpath(md if not os.path.exists(md) else pdf, RAIZ)}",
    )

    if not (os.path.exists(md) and os.path.exists(huella)):
        check(
            fase, "el PDF para Dirección corresponde a su Markdown", False,
            "Falta la huella del Markdown de origen; sin ella no hay forma de "
            "saber si el PDF está al día.",
            f"no existe {os.path.relpath(huella, RAIZ)}",
        )
        return

    actual = hashlib.sha256(open(md, "rb").read()).hexdigest()
    registrada = open(huella, encoding="utf-8").read().split()[0]
    check(
        fase, "el PDF para Dirección corresponde a su Markdown",
        actual == registrada,
        "Un PDF es binario: si se queda atrás respecto al texto que lo origina, "
        "nadie lo nota en una revisión y Dirección lee una versión que ya no es "
        "cierta. Regenerar con herramientas/informe-direccion/generar-pdf.py.",
        f"md={actual[:12]}… registrada={registrada[:12]}…",
    )


def hay_navegador(app):
    """¿Se puede correr una sonda? Devuelve el motivo si no.

    La búsqueda del navegador vive en `app/pruebas/sondas/navegador.mjs` y aquí
    se le pregunta, en vez de repetir la lógica: dos copias de un localizador
    de rutas divergen, y la que divergiría es la que decide si una sonda se
    omite en silencio.
    """
    if not os.path.isdir(os.path.join(app, "node_modules", "playwright-core")):
        return "falta playwright-core: npm --prefix app ci"

    codigo, salida, _ = corre(
        [
            "node",
            "-e",
            "import('./pruebas/sondas/navegador.mjs').then(m => {"
            "  const r = m.buscarChromium();"
            "  if (!r.ruta) { process.stderr.write(r.motivo); process.exit(1) }"
            "})",
        ],
        cwd=app,
        timeout=60,
    )
    return None if codigo == 0 else (salida.strip() or "no se encontró Chromium")


def sondas(rapido, app):
    """Las tres sondas de navegador de la Fase 2.

    Se omiten **con motivo** si falta el navegador, nunca se dan por buenas:
    un omitido no es un fallo, pero tampoco es una validación.
    """
    definidas = [
        (
            "sonda: la tabla virtualiza de verdad",
            "sonda:tabla",
            True,
            "Sin altura que desborde, el virtualizador concluye que caben todas "
            "las filas y renderiza la lista entera. Se veía perfecto en una "
            "captura y habría tirado la ventana con 500 000 contactos (T-7).",
        ),
        (
            "sonda: teclado y foco",
            "sonda:teclado",
            True,
            "§100: el atrapado de foco del modal y el anillo en cada parada de "
            "tabulación no se pueden comprobar sin un orden de tabulación real.",
        ),
        (
            "sonda: la CSP del producto no necesita estilo en línea",
            "sonda:csp",
            False,
            "La CSP que se instala en la máquina del cliente. Si alguien vuelve "
            "a meter `unsafe-inline`, aquí salta.",
        ),
    ]

    motivo = hay_navegador(app)
    servidor = None
    try:
        for nombre, script, necesita_servidor, porque in definidas:
            if motivo:
                omitir(2, nombre, motivo)
                continue
            if rapido and not necesita_servidor:
                omitir(2, nombre, "--rapido")
                continue
            if necesita_servidor and servidor is None:
                servidor = subprocess.Popen(
                    ["npm", "run", "dev", "--silent"],
                    cwd=app, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                )
                time.sleep(8)
            check_cmd(2, nombre, ["npm", "run", script, "--silent"],
                      porque, cwd=app, timeout=900)
    finally:
        if servidor is not None:
            servidor.terminate()
            try:
                servidor.wait(timeout=10)
            except subprocess.TimeoutExpired:
                servidor.kill()


def fase_2(rapido):
    titulo("FASE 2 · Design System")

    # ── Estructura ──
    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("app/src/design/tipografia.css", "Sin las @font-face la escala cae en la fuente de reserva sin avisar."),
        ("app/src/design/componentes/index.ts", "El punto de entrada único es lo que impide que aparezca un décimo botón local."),
        ("app/src/design/CatalogoDelSistema.vue", "Sin catálogo, las primitivas no se pueden revisar con los ojos."),
    ]:
        check(2, f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    cortes = ["Regular", "SemiBold", "Bold", "Black"]
    faltan = [
        c for c in cortes
        if not os.path.exists(os.path.join(RAIZ, f"TIPOGRAFIA/Mont-{c}.woff2"))
    ]
    referencias = sum(
        f"@fuentes/Mont-{c}.woff2" in (leer("app/src/design/tipografia.css") or "")
        for c in cortes
    )
    check(
        2, "los cuatro cortes de Mont están incrustados",
        not faltan and referencias == len(cortes),
        "La escala usa Regular, SemiBold, Bold y Black. Si falta uno, el motor "
        "lo sintetiza engordando el trazo y la tipografía deja de ser Mont sin "
        "que nada falle.",
        f"faltan: {', '.join(faltan)}" if faltan else "",
    )

    # Sin comentarios: la cabecera de tipografia.css menciona «@font-face» al
    # explicar la regla, y contarla daría una cara de más.
    css_tipografia = sin_comentarios(leer("app/src/design/tipografia.css") or "")
    caras = css_tipografia.count("@font-face")
    pesos = len(re.findall(r"font-weight:\s*\d+", css_tipografia))
    check(
        2, "cada @font-face declara su font-weight", caras > 0 and caras == pesos,
        "El usWeightClass de este kit está desplazado (Mont-Regular declara "
        "600). Si un @font-face no fija su peso, el motor usa el del archivo y "
        "asigna el corte equivocado (TIPOGRAFIA.md §2).",
        f"{caras} caras, {pesos} pesos declarados",
    )

    # ── §17 · ningún componente contiene un valor de diseño literal ──
    print(f"{GRIS}  tokens{FIN}")
    hex_sueltos, ms_sueltos, px_sueltos = [], [], []
    for ruta in primitivas():
        with open(ruta, encoding="utf-8") as f:
            cuerpo = sin_comentarios(f.read())
        corto = os.path.relpath(ruta, RAIZ)
        hex_sueltos += [f"{corto}: {m}" for m in RE_HEX.findall(cuerpo)]
        ms_sueltos += [f"{corto}: {m}" for m in RE_MS.findall(cuerpo)]
        px_sueltos += [f"{corto}: {m}px" for m in RE_PX.findall(cuerpo)]

    check(
        2, "ninguna primitiva contiene un color literal", not hex_sueltos,
        "§17: el color tiene una sola fuente, tokens.json, porque es la única "
        "que pasa por la verificación de contraste de CI. Un hex escrito en un "
        "componente se salta esa verificación entera.",
        " · ".join(hex_sueltos[:6]),
    )
    check(
        2, "ninguna primitiva contiene una duración literal", not ms_sueltos,
        "§98: el movimiento tiene tres duraciones y una curva. Una duración "
        "suelta es una cuarta que nadie decidió.",
        " · ".join(ms_sueltos[:6]),
    )
    check(
        2, "ninguna primitiva contiene una medida suelta", not px_sueltos,
        "§17: alturas, anchos y sombras salen de tokens. Los filetes de 1 a 3 "
        "px sí se escriben (DESIGN_SYSTEM §2.1): son detalle de borde, no una "
        "decisión de escala.",
        " · ".join(px_sueltos[:6]),
    )

    # ── Todo token usado existe ──
    definidos = set()
    for ruta in ("app/src/design/base.css", "herramientas/design-tokens/dist/arles-tokens.css"):
        definidos |= set(RE_DEF_DE_TOKEN.findall(leer(ruta) or ""))

    huerfanos = set()
    for base, _, archivos in os.walk(os.path.join(RAIZ, "app/src")):
        for n in archivos:
            if not n.endswith((".vue", ".css", ".ts")):
                continue
            with open(os.path.join(base, n), encoding="utf-8") as f:
                for token in RE_USO_DE_TOKEN.findall(f.read()):
                    if token not in definidos:
                        huerfanos.add(f"{n}: {token}")

    # ── El catálogo no puede llegar a la máquina del cliente ──
    #
    # Dos razones para comprobarlo, y la segunda es la que importa: la sonda de
    # CSP **edita este archivo** para forzar el catálogo dentro, compila, y lo
    # restaura al terminar. Si algo la interrumpe entre medias, el `if (true)`
    # se queda escrito. Esta comprobación es lo que impide que eso se suba.
    router = leer("app/src/app/router.ts") or ""
    check(
        2, "el catálogo está limitado al modo de desarrollo",
        "if (import.meta.env.DEV) {" in router
        and "CatalogoDelSistema" in router
        and "if (true)" not in router,
        "El catálogo es una pantalla de desarrollo. En el bundle del cliente "
        "sería superficie de ataque sin contrapartida, y la sonda de CSP fuerza "
        "temporalmente su inclusión: si se interrumpe, deja el interruptor "
        "abierto.",
        "el interruptor no es import.meta.env.DEV",
    )

    check(
        2, "todo token que se usa está definido", not huerfanos,
        "Un `var(--arles-bg-deeep)` mal escrito no da error: el navegador lo "
        "resuelve a nada y el elemento se queda transparente. Es el fallo de "
        "CSS que más lejos llega sin que nadie lo vea.",
        " · ".join(sorted(huerfanos)[:6]),
    )

    # ── Accesibilidad estructural ──
    print(f"{GRIS}  accesibilidad{FIN}")
    con_outline_none = []
    for ruta in primitivas() + [os.path.join(RAIZ, "app/src/design/base.css")]:
        with open(ruta, encoding="utf-8") as f:
            cuerpo = f.read()
        for bloque in re.finditer(r"([^{}]*)\{([^{}]*outline:\s*none[^{}]*)\}", cuerpo):
            selector = bloque.group(1).strip().splitlines()[-1].strip()
            # La ÚNICA excepción es `:focus:not(:focus-visible)`: apaga el
            # anillo al llegar por ratón y lo deja intacto para el teclado.
            #
            # La versión anterior perdonaba cualquier selector que contuviera
            # `:focus`, y así dejó pasar un `.panel:focus { outline: none }`
            # que apagaba el anillo también para el teclado — lo encontró la
            # sonda de teclado, no esta comprobación. Ahora la excepción es
            # literal: si alguien quiere otra, tiene que discutirla.
            if re.search(r":focus:not\(\s*:focus-visible\s*\)\s*$", selector):
                continue
            con_outline_none.append(f"{os.path.relpath(ruta, RAIZ)}: {selector}")

    check(
        2, "nadie quita el anillo de foco sin sustituto", not con_outline_none,
        "DESIGN_SYSTEM §5: es la regla que más a menudo se rompe en una "
        "revisión de diseño y la que más rompe la navegación por teclado "
        "(§100).",
        " · ".join(con_outline_none[:6]),
    )

    # La regla completa, no la subcadena: una búsqueda de texto pasaría con
    # `prefers-reduced-motionXX` escrito por error. Es la misma trampa que la
    # Fase 1 encontró en la comprobación de eslint.
    base = leer("app/src/design/base.css") or ""
    regla = re.search(
        r"@media[^{]*\(\s*prefers-reduced-motion\s*:\s*reduce\s*\)\s*\{(.*?)\n\}",
        base, re.S,
    )
    cuerpo = regla.group(1) if regla else ""
    check(
        2, "se respeta prefers-reduced-motion",
        bool(regla)
        and "animation-duration" in cuerpo
        and "transition-duration" in cuerpo,
        "§98: obligatorio, no opcional. Y tiene que apagar las dos cosas: una "
        "regla que sólo neutraliza `transition` deja corriendo cualquier "
        "`animation`, como la del esqueleto de carga.",
        "no se encontró la regla" if not regla else "la regla no apaga ambas",
    )

    # ── Se construye y pasa sus tests ──
    print(f"{GRIS}  frontend{FIN}")
    app = os.path.join(RAIZ, "app")
    if rapido:
        omitir(2, "build con las primitivas", "--rapido")
    else:
        check_cmd(
            2, "build con las primitivas",
            ["npm", "run", "build", "--silent"],
            "Un componente que no entra en el bundle no existe para el usuario.",
            cwd=app, timeout=600,
        )

    # ── Sondas de navegador ──
    #
    # Comprueban lo que jsdom NO puede: allí el contenedor de la tabla mide 0px
    # de alto (así que el virtualizador no renderiza nada y un test de
    # virtualización pasaría en vacío), no hay orden de tabulación real, y no
    # hay CSP. Las tres encontraron defectos que los tests daban por buenos.
    print(f"{GRIS}  sondas de navegador{FIN}")
    sondas(rapido, app)

    # ── Documentación ──
    print(f"{GRIS}  documentación{FIN}")
    informe_para_direccion(2, "FASE-02-PARA-DIRECCION")


FASES = {0: fase_0, 1: fase_1, 2: fase_2}


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
