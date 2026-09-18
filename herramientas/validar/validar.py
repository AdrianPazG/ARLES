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
import datetime
import hashlib
import json
import os
import re
import shutil
import struct
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


def resuelve(cmd):
    """Pone la ruta completa del programa en una orden dada como lista.

    En Windows `npm` es `npm.cmd`, y `subprocess` sin `shell` no aplica
    PATHEXT: pedir «npm» daba WinError 2 y las seis sondas fallaban en 0 s sin
    llegar a abrir el navegador. En Linux, donde corre CI, no pasaba.
    """
    if isinstance(cmd, str):
        return cmd
    return [shutil.which(cmd[0]) or cmd[0], *cmd[1:]]


def corre(cmd, cwd=RAIZ, timeout=1800):
    t0 = time.time()
    try:
        p = subprocess.run(
            resuelve(cmd), cwd=cwd, shell=isinstance(cmd, str), capture_output=True,
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
    # Se lee la V3 y no la V1: la V1 está congelada y su texto ya no describe el
    # esquema que corre. Comprobar la migración vieja daría un ✓ sobre algo que
    # otra migración pudo deshacer después.
    esquema = leer("crates/arles-db/migrations", "V3__canales_de_contacto.sql") or ""
    ultima = leer("crates/arles-db/migrations", "V4__etapas_de_campana.sql") or ""
    check(1, "Borrar un contacto no borra el registro de envío",
          "contact_address" in ultima
          and "contact_id          TEXT REFERENCES contact(id) ON DELETE SET NULL"
          in ultima
          and "idx_attempt_unique\n    ON message_attempt(campaign_id, channel, contact_address)"
          in ultima,
          "Si el intento cascadea con el contacto, se pierden a la vez la "
          "auditoría y la protección contra duplicados: reimportar al contacto "
          "permitiría enviarle otra vez (§55, ADR-0004).")

    # L-2: los dos nombres de canal están escritos en dos sitios —el enum de
    # Rust y las CHECK del esquema— y no hay forma de importar uno del otro. Si
    # divergen, la base rechaza las filas que el código cree válidas, y el fallo
    # aparece lejísimos de la causa.
    canal_rs = leer("crates/arles-core/src", "canal.rs") or ""
    en_rust = re.findall(r'Self::\w+ => "(\w+)"', canal_rs)
    en_sql = "CHECK (channel IN ('email', 'whatsapp'))"
    check(1, "Los nombres de canal de Rust y del esquema coinciden",
          sorted(set(en_rust)) == ["email", "whatsapp"] and en_sql in esquema,
          "`Canal::como_texto` produce las cadenas que las CHECK de la migración "
          "aceptan. Una divergencia no se ve al compilar: se ve como filas "
          "rechazadas en tiempo de ejecución.",
          detalle=f"en Rust: {sorted(set(en_rust))}")

    # La primera versión de la V4 reconstruía `campaign` con DROP + RENAME. Con
    # las claves foráneas activas, soltar una tabla PADRE ejecuta un borrado
    # implícito que cascadea a `campaign_audience` y `message_attempt`: la
    # migración terminaba sin un solo error y con la audiencia congelada y el
    # registro de envíos en cero filas. Lo cubre un test sobre base poblada;
    # esto lo detecta en la migración SIGUIENTE, antes de razonarlo otra vez.
    check(1, "Ninguna migración suelta una tabla de la que cuelgan otras",
          not re.search(r"^DROP TABLE (campaign|contact|company|email_account)\s*;",
                        ultima, re.MULTILINE),
          "Soltar una tabla padre con las claves foráneas activas borra sus "
          "hijas en cascada. Las columnas se quitan en su sitio con ALTER TABLE "
          "… DROP COLUMN; sólo se reconstruyen tablas de las que no cuelga nada.",
          detalle="revisado sobre la última migración")

    check(1, "El consentimiento es append-only y no bloquea el borrado",
          "consent_entry_sin_update" in esquema
          and "consent_entry_sin_delete" in esquema
          # Una DECLARACIÓN de columna, no la palabra suelta: dentro de esa
          # tabla hay un comentario largo que explica por qué la columna no
          # está, y buscar el texto a secas hacía fallar la comprobación por la
          # explicación de sí misma.
          and not re.search(
              r"^\s+contact_id\s+TEXT",
              esquema.split("CREATE TABLE consent_entry")[-1].split(") STRICT;")[0],
              re.MULTILINE),
          "Si `consent_entry` llevara un contact_id con ON DELETE SET NULL, ese "
          "UPDATE chocaría con el disparador de append-only y un contacto con "
          "consentimiento registrado dejaría de poder borrarse: la prueba de "
          "que se le podía escribir impediría su derecho de cancelación.")

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

    # ── Invariantes que solo se rompen en otra plataforma o con el tiempo ──
    #
    # Las tres nacen del mismo incidente: CI llevaba trece ejecuciones en rojo
    # mientras el validador daba 54/54. Ninguna de ellas se puede comprobar
    # ejecutando el código aquí, y por eso se comprueban leyendo.
    print(f"{GRIS}  invariantes de plataforma y de política{FIN}")
    invariantes_de_unsafe()
    caducidad_de_deny()
    dependencias_de_linux_en_ci()
    rutas_de_tauri_conf()

    # ── Documentación ──
    print(f"{GRIS}  documentación{FIN}")
    informe_para_direccion(1, "FASE-01-PARA-DIRECCION")


def invariantes_de_unsafe():
    """El workspace deniega `unsafe`; la prohibición dura vive en las raíces.

    Por qué no basta con `forbid` en el workspace: `forbid` NO se puede levantar
    con un `#[allow]` local, y `arles-app` necesita exactamente uno —el
    MessageBoxW que impide que la aplicación muera en silencio en Windows—. Con
    `forbid` el crate no compilaba, pero el error solo aparecía al compilar PARA
    Windows: en Linux el bloque está detrás de `#[cfg(windows)]` y ni se mira.

    Así que la política pasó a `deny`, y estas comprobaciones son lo que impide
    que ese relajamiento se convierta en barra libre.
    """
    for crate in ("arles-core", "arles-db"):
        fuente = leer("crates", crate, "src", "lib.rs") or ""
        check(1, f"{crate} prohíbe unsafe en su raíz",
              "#![forbid(unsafe_code)]" in fuente,
              "Un crate de dominio que acepta `unsafe` ha dejado de ser de dominio.")

    app = leer("crates", "arles-app", "src", "lib.rs") or ""
    permisos = app.count("#[allow(unsafe_code)]")
    bloques = len(re.findall(r"\bunsafe\s*\{", app))
    check(1, "arles-app tiene exactamente una excepción de unsafe",
          permisos == 1 and bloques == 1,
          "La excepción es UNA y está auditada. Dos ya es una política, y esta no lo es.",
          f"{permisos} allow, {bloques} bloques unsafe")

    politica = leer("Cargo.toml") or ""
    check(1, "el workspace sigue denegando unsafe",
          'unsafe_code = "deny"' in politica,
          "Si alguien lo pone en `allow`, las dos raíces siguen protegidas pero arles-app deja de estarlo.")


def rutas_de_tauri_conf():
    """Las rutas de tauri.conf.json se resuelven contra DOS bases distintas.

    `frontendDist` es relativa al **archivo de configuración**
    (crates/arles-app/), pero `beforeBuildCommand` y `beforeDevCommand` se
    ejecutan desde el **directorio padre** de esa carpeta (crates/), que es lo
    que Tauri toma por raíz de la aplicación cuando la carpeta no se llama
    `src-tauri`.

    Las tres llevaban `../../app`, o sea la misma ruta para dos bases: una era
    correcta y las otras dos apuntaban FUERA del repositorio. Nada lo delataba
    —`cargo build` no ejecuta el beforeBuildCommand— hasta que el flujo de
    revisión intentó generar el instalador y murió en los dos sistemas a la vez.

    Medido, no deducido: con el repositorio en /home/user/ARLES, npm buscaba el
    package.json en /home/user/app.
    """
    tauri = os.path.join(RAIZ, "crates", "arles-app")
    crudo = leer("crates", "arles-app", "tauri.conf.json")
    if crudo is None:
        check(1, "existe tauri.conf.json", False, "Sin él no hay aplicación de escritorio.")
        return
    conf = json.loads(crudo).get("build", {})

    destino = conf.get("frontendDist", "")
    check(1, "frontendDist apunta a la carpeta real (base: el archivo de config)",
          os.path.isdir(os.path.normpath(os.path.join(tauri, destino)))
          or os.path.isdir(os.path.dirname(os.path.normpath(os.path.join(tauri, destino)))),
          "Si apunta fuera del repositorio, el instalador sale sin interfaz.",
          destino)

    # ── Iconos ──
    #
    # Tauri 2 NO busca en icons/ por su cuenta: si `bundle.icon` no está
    # declarado, la lista queda vacía. El empaquetado de Windows murió con
    # «Couldn't find a .ico icon» después de quince minutos compilando, con los
    # archivos ahí mismo y correctos.
    bundle = json.loads(crudo).get("bundle", {})
    iconos = bundle.get("icon", [])
    check(1, "bundle.icon está declarado", bool(iconos),
          "Sin la lista, Tauri no encuentra los iconos aunque estén en su sitio.")
    faltan = [i for i in iconos if not os.path.isfile(os.path.join(tauri, i))]
    check(1, "todos los iconos declarados existen", not faltan,
          "Un icono declarado que no está rompe el empaquetado, no la compilación.",
          ", ".join(faltan))

    # Windows pinta el icono a 16 y 32 px en la barra de tareas. Un .ico con
    # una sola imagen de 256 px se reescala y se ve blando: pasa el
    # empaquetado y falla a la vista, que es peor.
    ico = os.path.join(tauri, "icons", "icon.ico")
    if os.path.isfile(ico):
        with open(ico, "rb") as f:
            cabecera = f.read(6)
            _, tipo, cuantas = struct.unpack("<HHH", cabecera)
            medidas = set()
            for _ in range(cuantas):
                ancho = struct.unpack("<B", f.read(1))[0] or 256
                f.read(15)
                medidas.add(ancho)
        check(1, "el .ico trae las medidas pequeñas de Windows",
              tipo == 1 and {16, 32}.issubset(medidas),
              "A 16 y 32 px Windows reescala si no están, y el icono se ve blando en la barra de tareas.",
              f"medidas: {sorted(medidas)}")

    # `--prefix X` le dice a npm dónde está el package.json.
    raiz_app = os.path.dirname(tauri)  # crates/
    for clave in ("beforeBuildCommand", "beforeDevCommand"):
        orden = conf.get(clave, "")
        m = re.search(r"--prefix\s+(\S+)", orden)
        if not m:
            check(1, f"{clave} lleva un --prefix legible", False,
                  "Sin poder leer la ruta no se puede comprobar a dónde apunta.", orden)
            continue
        resuelta = os.path.normpath(os.path.join(raiz_app, m.group(1)))
        check(1, f"{clave} encuentra el package.json (base: la carpeta padre)",
              os.path.isfile(os.path.join(resuelta, "package.json")),
              "Tauri lo ejecuta desde crates/, no desde crates/arles-app/. "
              "Con la base equivocada apunta fuera del repositorio y el instalador no se genera.",
              f"{m.group(1)} → {resuelta}")


RE_REVISAR = re.compile(r"REVISAR (\d{4}-\d{2}-\d{2})")


def caducidad_de_deny():
    """Cada aviso de seguridad ignorado lleva fecha, y la fecha se hace cumplir.

    cargo-deny 0.20 solo admite `id` y `reason` en un `ignore`: no hay `expires`.
    Sin esto, la fecha escrita en el texto sería un comentario que nadie vuelve
    a leer, y un aviso ignorado «temporalmente» se queda para siempre.
    """
    texto = leer("deny.toml")
    if texto is None:
        check(1, "deny.toml existe", False, "La política de licencias debe existir.")
        return

    entradas = re.findall(r"\{\s*id\s*=\s*\"([^\"]+)\"\s*,\s*reason\s*=\s*\"([^\"]*)\"", texto)
    sin_fecha = [i for i, razon in entradas if not RE_REVISAR.search(razon)]
    check(1, "todo aviso ignorado lleva fecha de revisión",
          not sin_fecha,
          "Un `ignore` sin fecha es una alfombra. Con fecha es una deuda con vencimiento.",
          ", ".join(sin_fecha))

    hoy = datetime.date.today()
    vencidos = []
    for ident, razon in entradas:
        m = RE_REVISAR.search(razon)
        if m and datetime.date.fromisoformat(m.group(1)) < hoy:
            vencidos.append(f"{ident} venció el {m.group(1)}")
    check(1, "ningún aviso ignorado ha caducado",
          not vencidos,
          "Pasada la fecha hay que volver a mirar si ya existe versión a la que subir.",
          "; ".join(vencidos))


def dependencias_de_linux_en_ci():
    """Todo job de Linux que compile el workspace instala WebKitGTK.

    Esta es la comprobación que importa de las tres, porque ataca la clase y no
    el caso: el job `formato` corría `cargo clippy --workspace`, que arrastra
    Tauri, sin instalar `libwebkit2gtk-4.1-dev`. Moría en el build script de
    glib-sys —«Package glib-2.0 was not found»— y llevaba así desde el primer
    push. El job `tests` sí las instalaba, así que el fallo no era del código
    sino de un job que se quedó atrás del otro.
    """
    ci = leer(".github", "workflows", "ci.yml")
    if ci is None:
        check(1, "existe el flujo de CI", False, "Sin CI, nada de esto se hace cumplir.")
        return

    # Los jobs son las claves de dos espacios bajo `jobs:`.
    bloques = re.split(r"\n  (?=[a-z][a-z0-9_-]*:\n)", ci.split("\njobs:\n", 1)[-1])
    culpables = []
    for bloque in bloques:
        nombre = bloque.split(":", 1)[0].strip()
        corre_workspace = re.search(r"cargo (clippy|build|test|check).*--workspace", bloque)
        # Un job de matriz instala las dependencias bajo `if: runner.os == 'Linux'`;
        # uno que solo corre en ubuntu las instala sin condición. Ambos valen.
        instala = "libwebkit2gtk-4.1-dev" in bloque
        toca_linux = "ubuntu-latest" in bloque or "matrix.os" in bloque
        if corre_workspace and toca_linux and not instala:
            culpables.append(nombre)

    check(1, "todo job de Linux que compila el workspace instala WebKitGTK",
          not culpables,
          "arles-app arrastra Tauri, y Tauri en Linux enlaza WebKitGTK. Sin ella el job muere en glib-sys.",
          ", ".join(culpables))


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
        (
            "sonda: lo que anuncia un lector de pantalla",
            "sonda:lector",
            False,
            "Sustituye a la sección de NVDA del manual, que era la comprobación "
            "más cara y menos repetible de las cuatro. NVDA no inventa lo que "
            "dice: lo deriva del árbol de accesibilidad, y ese árbol se lee. "
            "«¿Dice botón?» deja de ser cuestión de oído y pasa a ser de dato.",
        ),
        (
            "sonda: ninguna escala de Windows desborda a lo ancho",
            "sonda:ancho",
            False,
            "El mínimo de la ventana está en píxeles lógicos: a 200 % de escala, "
            "1120 lógicos son 2240 físicos, más que la pantalla. El armazón "
            "fijaba min-width 1120 y desbordaba. Lo encontró Dirección revisando; "
            "ninguna comprobación lo veía porque todas corrían por encima de 1120.",
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
                    resuelve(["npm", "run", "dev", "--silent"]),
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

    app = os.path.join(RAIZ, "app")

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
    # Esta comprobación mira **el bundle compilado**, no el texto del router.
    # La versión anterior buscaba `if (import.meta.env.DEV)` en el archivo, y
    # eso es una búsqueda de texto: habría pasado con el interruptor correcto y
    # un segundo `rutas.push` del catálogo cinco líneas más abajo. Y se rompió
    # sola en cuanto el interruptor cambió de forma, que es la otra cara del
    # mismo defecto.
    #
    # Lo que importa no es cómo está escrito el interruptor: es si el catálogo
    # acaba dentro de lo que se instala en la máquina del cliente.
    dist = os.path.join(app, "dist")
    if rapido and not os.path.isdir(dist):
        omitir(2, "el catálogo no entra en el bundle de producción",
               "--rapido y no hay dist/ previo que inspeccionar")
    else:
        if not os.path.isdir(dist):
            corre(["npm", "run", "build", "--silent"], cwd=app, timeout=600)
        rastro = []
        for base, _, archivos in os.walk(dist):
            for n in archivos:
                if not n.endswith((".js", ".css", ".html")):
                    continue
                ruta = os.path.join(base, n)
                with open(ruta, encoding="utf-8", errors="ignore") as f:
                    if "Catálogo del design system" in f.read():
                        rastro.append(os.path.relpath(ruta, app))
        check(
            2, "el catálogo no entra en el bundle de producción", not rastro,
            "El catálogo es una pantalla de desarrollo: en el bundle del "
            "cliente sería superficie de ataque sin contrapartida. Sólo entra "
            "con VITE_ARLES_CATALOGO=1, que usan la sonda de CSP y la "
            "compilación de revisión visual.",
            " · ".join(rastro[:4]),
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
    manual_de_entrega()


RE_IMAGEN_MD = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")
RE_CAPTURA = re.compile(r"\b([WM]-\d{2}-[a-z0-9-]+\.png)\b")


def manual_de_entrega():
    """Cada manual y su plantilla nombran exactamente las mismas capturas.

    Hay un manual por sistema porque las instrucciones se separan: SmartScreen
    no es Gatekeeper y el escalado sólo existe en Windows. Eso multiplica por
    dos la superficie donde algo puede quedarse atrás.

    Los nombres de archivo viven en dos sitios por pareja —la tabla del manual
    y la lista de la plantilla—. Renombrar una captura en uno y no en el otro
    deja a quien revisa siguiendo instrucciones que no concuerdan, y el fallo
    no se ve leyendo ninguno de los dos por separado: eso es lo que lo hace
    difícil de pillar a ojo.

    Y se comprueban las dos direcciones de la separación:
      · que ningún manual nombre capturas del otro sistema (una `M-` en el
        manual de Windows significa que se copió un bloque sin adaptarlo),
      · que el índice no se quede sin enlazar a uno de los dos.
    """
    base = os.path.join(RAIZ, "documentacion", "06-calidad")

    parejas = [
        ("Windows", "MANUAL_WINDOWS.md", "PLANTILLA-WINDOWS.md", "W"),
        ("Mac", "MANUAL_MAC.md", "PLANTILLA-MAC.md", "M"),
    ]

    for sistema, nombre_manual, nombre_plantilla, prefijo in parejas:
        manual = leer("documentacion", "06-calidad", nombre_manual)
        plantilla = leer("documentacion", "06-calidad", nombre_plantilla)

        if not check(2, f"existen el manual y la plantilla de {sistema}",
                     manual is not None and plantilla is not None,
                     "Sin ellos, la revisión de ese sistema depende de que yo "
                     "explique lo mismo otra vez en el chat."):
            continue

        rotas = [
            ruta for ruta in RE_IMAGEN_MD.findall(manual)
            if not ruta.startswith("http")
            and not os.path.isfile(os.path.normpath(os.path.join(base, ruta)))
        ]
        check(2, f"todas las imágenes del manual de {sistema} existen", not rotas,
              "Una imagen rota deja el manual sin lo que lo hacía claro.",
              ", ".join(rotas))

        en_manual = set(RE_CAPTURA.findall(manual))
        en_plantilla = set(RE_CAPTURA.findall(plantilla))
        check(2, f"el manual y la plantilla de {sistema} nombran las mismas capturas",
              bool(en_manual) and en_manual == en_plantilla,
              "Si divergen, quien revisa sigue dos instrucciones distintas y no lo nota.",
              f"sólo en el manual: {sorted(en_manual - en_plantilla)} · "
              f"sólo en la plantilla: {sorted(en_plantilla - en_manual)}")

        # Una captura del otro sistema es el rastro de un bloque copiado y no
        # adaptado. En Windows son nueve y en Mac seis: si aparece una `M-` en
        # el de Windows, alguien va a hacer una captura de más o de menos.
        intrusas = sorted(c for c in en_manual | en_plantilla if not c.startswith(prefijo + "-"))
        check(2, f"el material de {sistema} no nombra capturas del otro sistema",
              not intrusas,
              "Separar los manuales sólo sirve si cada uno pide lo suyo.",
              ", ".join(intrusas))

    # ── Los formatos distribuibles van con su origen ──
    #
    # El PDF y el .docx se generan desde el Markdown, no se editan a mano.
    # La huella del ORIGEN guardada al lado delata el caso que importa: que
    # alguien corrija el .md y reparta un PDF que dice otra cosa. Es el mismo
    # mecanismo que ya protege los informes para Dirección.
    for nombre, extension in [
        ("MANUAL_WINDOWS", "pdf"), ("MANUAL_MAC", "pdf"),
        ("PLANTILLA-WINDOWS", "docx"), ("PLANTILLA-MAC", "docx"),
    ]:
        md = os.path.join(base, f"{nombre}.md")
        binario = os.path.join(base, f"{nombre}.{extension}")
        marca = f"{binario}.sha256"

        if not check(2, f"existe {nombre}.{extension}", os.path.isfile(binario),
                     f"El .{extension} es lo que se manda a quien revisa; sin él sólo queda el Markdown."):
            continue

        if not os.path.isfile(marca):
            check(2, f"{nombre}.{extension} lleva la huella de su Markdown", False,
                  "Sin huella no hay forma de saber si se quedó atrás.")
            continue

        with open(md, "rb") as f:
            esperada = hashlib.sha256(f.read()).hexdigest()
        with open(marca, encoding="utf-8") as f:
            anotada = f.read().split()[0]
        check(2, f"{nombre}.{extension} corresponde a su Markdown",
              esperada == anotada,
              f"Regenera con la herramienta; un .{extension} viejo reparte instrucciones que ya no son.",
              f"esperada {esperada[:12]}… · anotada {anotada[:12]}…")

    # El índice reparte hacia los dos. Si se queda sin uno, ese manual existe
    # pero nadie llega a él.
    indice = leer("documentacion", "06-calidad", "MANUAL_DE_ENTREGA.md") or ""
    check(2, "el índice enlaza a los dos manuales",
          "MANUAL_WINDOWS.md" in indice and "MANUAL_MAC.md" in indice,
          "Un manual al que no se llega es un manual que no existe.")



# ─────────────────────────────────────────────────────────────────────────────
# FASE 3 · Empresa y contactos
# ─────────────────────────────────────────────────────────────────────────────

# sha256 de las migraciones ya publicadas.
#
# La regla del módulo `migraciones.rs` dice «nunca se edita una migración ya
# publicada»: hay bases con ella aplicada, y cambiarla produce divergencias
# silenciosas —dos instalaciones con el mismo número de versión y esquemas
# distintos—. Una regla escrita se erosiona; un hash no.
#
# Para añadir una migración: se añade su hash aquí. Para *cambiar* una ya
# publicada: no se hace.
MIGRACIONES_PUBLICADAS = {
    "V1__esquema_inicial.sql":
        "1a9e4d2bb22e02d89775d3a9543ae4bcb58369632a83a275972e548079b2b1ef",
    "V2__preferencias_de_interfaz.sql":
        "5ba0d6730f9d926d7137886bbbf34e4cc30d8f777c70be3b557ada37364c0b02",
    # L-1 y L-11, 17/09/2026.
    "V4__etapas_de_campana.sql":
        "2dd0cc3b5174e31d35706607df4811784edd48d95a84c38ffdfb1d546f7483de",
    # L-2 / D-6, 17/09/2026. A partir de aquí queda congelada como las otras dos.
    "V3__canales_de_contacto.sql":
        "bb057af229cf4b47ccb11063c8145631c61526b6fb8d00c3b782bf4a02c4dbdb",
    # ADR-0013 §1, origen declarado de la importación. 18/09/2026.
    "V5__origen_de_la_importacion.sql":
        "d3a7533331ec838fa2df6799cc2b65ee1691dc12d9fc52f21541a9945f25720f",
}


def listas_cerradas_del_nucleo():
    """Lo que ofrece el desplegable y lo que acepta el núcleo son el mismo dato.

    El frontend lleva una copia de las zonas y los países para cuando corre sin
    núcleo —`vite dev` y las sondas—. Es una costura real: en cuanto alguien
    añada una zona en Rust y no en TypeScript, las capturas de revisión
    enseñarían un desplegable que no es el del producto, y nadie lo notaría
    porque las dos pantallas se ven bien.
    """
    rust = leer("crates/arles-core/src", "empresa.rs") or ""
    ts = leer("app/src/app/stores", "empresa.ts") or ""

    def lista_rust(nombre):
        m = re.search(rf"{nombre}: &\[&str\] = &\[(.*?)\];", rust, re.S)
        return re.findall(r'"([^"]+)"', m.group(1)) if m else []

    def lista_ts(nombre):
        m = re.search(rf"{nombre}: \[(.*?)\],", ts, re.S)
        return re.findall(r"'([^']+)'", m.group(1)) if m else []

    for nombre_rust, nombre_ts, etiqueta in [
        ("ZONAS_SOPORTADAS", "zonas", "las zonas horarias"),
        ("PAISES_SOPORTADOS", "paises", "los países"),
    ]:
        a, b = lista_rust(nombre_rust), lista_ts(nombre_ts)
        check(3, f"{etiqueta} del núcleo y del frontend coinciden",
              bool(a) and a == b,
              "Si divergen, el usuario elige una opción de la lista y el núcleo "
              "se la rechaza: un error imposible de entender desde fuera.",
              f"núcleo: {a}\nfrontend: {b}")


def claves_de_texto_del_nucleo():
    """Toda clave de i18n que devuelve el núcleo existe en el catálogo.

    El núcleo no manda mensajes: manda claves (§139). Una clave sin texto no
    revienta nada —la pantalla enseña el texto genérico— y por eso se queda ahí
    para siempre, diciéndole a alguien «este dato no es válido» sin decirle cuál
    ni por qué.
    """
    rust = (leer("crates/arles-core/src", "empresa.rs") or "") + \
           (leer("crates/arles-core/src", "onboarding.rs") or "")
    es = leer("app/src/app/locales", "es.ts") or ""

    claves = sorted(set(re.findall(r'clave: "(empresa\.error\.[A-Za-z]+)"', rust)))
    faltan = [c for c in claves if c.rsplit(".", 1)[-1] not in es]
    check(3, "toda clave de error del núcleo tiene texto en español",
          bool(claves) and not faltan,
          "Una clave sin texto no revienta nada: enseña un mensaje genérico y "
          "se queda así para siempre.",
          f"claves encontradas: {claves}\nsin texto: {faltan}")

    # Y los pasos del alta: cada uno necesita título y detalle. Se leen sólo de
    # `fn clave`, no de todo el archivo: `fn entrega` devuelve también cadenas
    # con esa forma —«3.1», «5»— y la primera versión de esta comprobación las
    # tomó por pasos y falló pidiendo un texto para el paso «5».
    onboarding = leer("crates/arles-core/src", "onboarding.rs") or ""
    bloque = onboarding.split("pub fn clave(self)", 1)[-1].split("}", 1)[0]
    pasos = re.findall(r'=> "(\w+)"', bloque)
    faltan_pasos = [p for p in set(pasos) if f"{p}: {{" not in es]
    check(3, "todo paso del alta tiene título y detalle",
          bool(pasos) and not faltan_pasos,
          "Un paso sin texto sale en la lista como una clave cruda.",
          f"pasos sin texto: {faltan_pasos}")


def nombres_de_las_opciones():
    """Ninguna opción del desplegable se queda sin nombre legible.

    El valor que viaja al núcleo es `MX` o `America/Mexico_City`; lo que se lee
    es «México» o «Ciudad de México». Si falta el nombre se enseña el valor
    crudo —feo, pero no engaña— y nadie se entera. Añadir una zona en Rust y
    olvidar su nombre aquí es exactamente eso.
    """
    rust = leer("crates/arles-core/src", "empresa.rs") or ""
    es = leer("app/src/app/locales", "es.ts") or ""

    def lista(nombre):
        m = re.search(rf"{nombre}: &\[&str\] = &\[(.*?)\];", rust, re.S)
        return re.findall(r'"([^"]+)"', m.group(1)) if m else []

    for constante, etiqueta in [
        ("ZONAS_SOPORTADAS", "zona horaria"),
        ("PAISES_SOPORTADOS", "país"),
    ]:
        valores = lista(constante)
        # La clave se escribe entrecomillada si lleva barras, y suelta si no.
        faltan = [v for v in valores if f"'{v}':" not in es and f"\n      {v}:" not in es]
        check(3, f"toda opción de {etiqueta} tiene nombre legible",
              bool(valores) and not faltan,
              "Sin nombre se enseña el identificador crudo, y elegir la zona "
              "obliga a saber qué es un identificador IANA.",
              f"sin nombre: {faltan}")


def claves_de_error_sin_texto():
    """Toda clave que emite Rust está declarada y tiene texto.

    Hay tres sitios que deben coincidir: los `clave_i18n()` de Rust, el catálogo
    de textos y la lista de `errores.spec.ts`. Esa lista dice en su comentario
    que un error nuevo sin texto la hace fallar — y no era verdad: la lista está
    escrita a mano, así que una variante nueva simplemente no aparecía en ella.
    Ocurrió en la 3.1, con dos variantes.

    El patrón es el mismo que dejó `Some(1)` obsoleto en el test de arranque: un
    dato duplicado a mano que nadie compara. Aquí se compara.
    """
    claves = set()
    for crate in ("arles-core", "arles-db", "arles-app"):
        d = os.path.join(RAIZ, "crates", crate, "src")
        for archivo in os.listdir(d):
            if archivo.endswith(".rs"):
                texto = leer(f"crates/{crate}/src", archivo) or ""
                claves |= set(re.findall(r'=> "(error\.[a-z_.]+)"', texto))

    es = leer("app/src/app/locales", "es.ts") or ""
    spec = leer("app/src/app", "errores.spec.ts") or ""

    # En el catálogo cada error es un objeto anidado, así que se busca la última
    # parte de la clave seguida de `: {`.
    sin_texto = sorted(c for c in claves if f"{c.rsplit('.', 1)[-1]}: {{" not in es)
    check(3, "toda clave de error de Rust tiene texto en el catálogo",
          bool(claves) and not sin_texto,
          "Sin texto, el usuario ve el mensaje genérico y nadie se entera.",
          f"claves: {sorted(claves)}\nsin texto: {sin_texto}")

    sin_vigilar = sorted(c for c in claves if f"'{c}'" not in spec)
    check(3, "la lista de claves de errores.spec.ts está completa",
          bool(claves) and not sin_vigilar,
          "Esa lista afirma vigilar que ninguna clave se quede sin texto, y "
          "está escrita a mano: si no se actualiza, la afirmación es falsa y "
          "el test pasa sin comprobar nada de lo nuevo.",
          f"claves sin vigilar: {sin_vigilar}")


def iconos_de_seccion():
    """Las seis secciones tienen icono, y el mapa los usa.

    P-11 decidió «iconos sin texto» al plegar. Una sección sin icono queda
    plegada como un hueco en blanco que no se puede pulsar con criterio.
    """
    icono = leer("app/src/design/componentes", "AIcono.vue") or ""
    router = leer("app/src/app", "router.ts") or ""

    secciones = re.findall(r"'(\w+)',", router.split("] as const")[0])
    faltan = [s for s in secciones if f"\n  {s}:" not in icono]
    check(3, "cada sección de la navegación tiene su icono",
          len(secciones) == 6 and not faltan,
          "Plegada, la barra sólo enseña iconos: una sección sin el suyo queda "
          "en blanco.",
          f"secciones: {secciones}\nsin icono: {faltan}")

    check(3, "el mapa de iconos está tipado por sección",
          "Record<Seccion, NombreDeIcono>" in router,
          "Con el tipo, añadir una séptima sección sin icono no compila. Sin "
          "él, compila y se descubre mirando la barra plegada.")

    check(3, "la barra plegada conserva el nombre accesible",
          "solo-lectores" in (leer("app/src/app", "App.vue") or ""),
          "Con `display: none` el texto sale del árbol de accesibilidad y un "
          "lector de pantalla anuncia seis enlaces sin nombre.")


def preferencias_fuera_del_navegador():
    """Ninguna preferencia que deba recordarse vive en `localStorage`.

    P-11 dice «se recuerda». `localStorage` vive en el perfil de la WebView: se
    borra con la caché del sistema, no entra en el respaldo `.arles` y en
    Windows depende del directorio de WebView2. Una preferencia que se pierde
    al limpiar la caché no se recuerda.
    """
    # Sólo código. `app/src` también guarda activos binarios —el logotipo de
    # TELEMETRY teñido— y leerlos como texto rompía el validador con un
    # UnicodeDecodeError en vez de decir qué pasaba.
    FUENTES = (".ts", ".js", ".vue", ".mts", ".cts", ".json", ".css", ".html")
    usos = []
    base = os.path.join(RAIZ, "app/src")
    for dir_actual, _, archivos in os.walk(base):
        for a in archivos:
            if not a.endswith(FUENTES):
                continue
            ruta = os.path.join(dir_actual, a)
            with open(ruta, encoding="utf-8") as f:
                if "localStorage" in f.read():
                    usos.append(os.path.relpath(ruta, RAIZ))
    check(3, "ninguna preferencia se guarda en localStorage", not usos,
          "Se borra con la caché, no entra en el respaldo y no viaja con los "
          "datos del usuario.",
          f"archivos: {usos}")


def avance_no_diverge():
    """El porcentaje es el mismo en los tres sitios.

    Dirección lo pidió en GitHub y en la pantalla de Inicio. Son dos sitios, y
    dos sitios con el mismo número escrito a mano son dos números que divergen
    en la primera prisa: se actualiza el del README al cerrar una fase y el de
    la pantalla se queda con el del mes pasado.

    Aquí se recalcula desde `avance.json` y se compara con lo que hay escrito
    en los dos destinos generados. Si no coinciden, es que alguien editó un
    destino a mano o se olvidó de correr el generador.
    """
    fuente = os.path.join(RAIZ, "documentacion/07-entrega/avance.json")
    if not os.path.exists(fuente):
        check(3, "el avance tiene una fuente única", False,
              "Sin `avance.json` el porcentaje vuelve a ser un número suelto.")
        return

    with open(fuente, encoding="utf-8") as f:
        datos = json.load(f)

    pesos = sum(x["peso"] for x in datos["fases"])
    check(3, "los pesos del avance suman 100", pesos == 100,
          "Si no suman 100, el porcentaje no es un porcentaje de nada.",
          f"suman {pesos}")

    sin_porque = [x["id"] for x in datos["fases"] if not x.get("porque", "").strip()]
    check(3, "cada fracción de avance dice en qué se apoya", not sin_porque,
          "Una fracción sin nada que la sostenga es una opinión disfrazada de "
          "medición (§94).",
          f"sin «porque»: {sin_porque}")

    esperado = round(sum(x["peso"] * x["hecho"] for x in datos["fases"]))

    readme = leer(".", "README.md") or ""
    m = re.search(r"avance_v1\.2\.0-(\d+)%25", readme)
    en_readme = int(m.group(1)) if m else None
    check(3, "el avance del README coincide con la fuente",
          en_readme == esperado,
          "La portada de GitHub es donde Dirección lo mira. Un número viejo ahí "
          "es peor que ninguno.",
          f"README {en_readme} vs calculado {esperado}")

    # Aquí había una tercera comprobación, sobre el porcentaje dentro de la
    # aplicación. Dirección decidió el 17/09/2026 que el avance va **sólo** en
    # la portada de GitHub, así que se retiró de la pantalla y con ella su
    # comprobación: vigilar un destino que ya no existe sería un ✓ sobre nada.


def migraciones_no_se_editan():
    """Una migración publicada no cambia.

    Hay bases de clientes con ella aplicada. Editarla deja dos instalaciones
    con el mismo número de versión y esquemas distintos, y el fallo aparece
    mucho después, en una consulta que funciona en un equipo y no en otro.
    """
    dir_m = os.path.join(RAIZ, "crates/arles-db/migrations")
    en_disco = sorted(os.listdir(dir_m))
    for nombre in en_disco:
        with open(os.path.join(dir_m, nombre), "rb") as f:
            real = hashlib.sha256(f.read()).hexdigest()
        esperado = MIGRACIONES_PUBLICADAS.get(nombre)
        check(3, f"la migración {nombre} no ha cambiado",
              esperado == real,
              "Editar una migración publicada produce esquemas divergentes con "
              "el mismo número de versión.",
              f"esperado {esperado}\nreal     {real}")


def contactos_no_entran_sin_validar():
    """Un contacto validado no puede construirse deserializando.

    `DatosDeContacto` y `CanalValidado` sólo salen del validador del núcleo. Si
    alguno ganara `Deserialize`, la webview podría mandar un contacto con los
    canales ya «normalizados» a su gusto, y la deduplicación y la supresión
    —que comparan exactamente esa forma— dejarían de funcionar sin que nada
    fallara ni en los tests ni en pantalla.

    La puerta de entrada buena es `BorradorDeContacto`, que sí deserializa
    porque es lo que hay que validar.
    """
    txt = leer("crates", "arles-core", "src", "contacto.rs")
    for tipo in ("DatosDeContacto", "CanalValidado"):
        # TODOS los atributos de encima, no sólo el último renglón. La primera
        # versión miraba una sola línea y ahí está `#[serde(rename_all …)]`, no
        # el `#[derive(…)]`: la comprobación pasaba siempre, dijera lo que
        # dijera el derive. Se vio al intentar romperla y no conseguirlo.
        i = txt.find(f"pub struct {tipo} {{")
        atributos = []
        if i > 0:
            for linea in reversed(txt[:i].rstrip().split("\n")):
                if not linea.lstrip().startswith("#["):
                    break
                atributos.append(linea)
        cabecera = "\n".join(atributos)
        check(
            3,
            f"{tipo} no se puede deserializar",
            i > 0 and bool(atributos) and "Deserialize" not in cabecera,
            contactos_no_entran_sin_validar.__doc__,
            detalle=cabecera.strip(),
        )

    check(
        3,
        "BorradorDeContacto sí se deserializa",
        "pub struct BorradorDeContacto" in txt,
        "Es lo que entra por la IPC; sin él no habría nada que validar.",
    )


def la_capa_de_datos_no_conoce_la_ipc():
    """`arles-db` no depende de `serde`.

    El crate de datos no sabe en qué formato viajan las cosas por la IPC, y no
    debe: si `ContactoGuardado` se serializara directamente, renombrar una
    columna aquí cambiaría el JSON que recibe la pantalla. La conversión vive
    en `arles-app`, que es la frontera.
    """
    cargo = leer("crates", "arles-db", "Cargo.toml")
    deps = cargo.split("[dependencies]", 1)[-1].split("[features]", 1)[0]
    check(
        3,
        "arles-db no depende de serde",
        "serde" not in deps,
        la_capa_de_datos_no_conoce_la_ipc.__doc__,
    )


def los_comandos_de_contactos_no_reciben_la_empresa():
    """Ningún comando de contactos acepta un `CompanyId` de la webview.

    En v1.2.0 hay una sola empresa (D-4), así que el identificador se lee de la
    base. Si viajara como parámetro, el frontend podría pedir los contactos de
    cualquier empresa pasando otro UUID — y que hoy sólo haya una no es una
    defensa, es una coincidencia que dejará de serlo en v1.3.
    """
    txt = leer("crates", "arles-app", "src", "comandos.rs")
    malas = []
    for bloque in txt.split("#[tauri::command]")[1:]:
        firma = bloque.split("{", 1)[0]
        nombre = firma.split("pub fn ", 1)[-1].split("(", 1)[0].strip()
        if "contacto" in nombre and "CompanyId" in firma:
            malas.append(nombre)
    check(
        3,
        "ningún comando de contactos recibe la empresa desde la webview",
        not malas,
        los_comandos_de_contactos_no_reciben_la_empresa.__doc__,
        detalle=", ".join(malas),
    )


def la_pantalla_de_contactos_no_reordena():
    """La ficha no reordena los canales: los pinta como llegan.

    El orden —correos primero, móviles después, el principal arriba— lo decide
    la consulta de `arles-db`. Si la pantalla volviera a ordenarlos, habría dos
    criterios, y el día que divergieran la lista y la ficha enseñarían los
    mismos canales en distinto orden sin que nada fallara.
    """
    txt = leer("app", "src", "app", "pantallas", "PantallaContactos.vue")
    sospechosos = [t for t in (".sort(", ".reverse(") if t in txt]
    check(
        3,
        "la pantalla de contactos no reordena los canales",
        not sospechosos,
        la_pantalla_de_contactos_no_reordena.__doc__,
        detalle=", ".join(sospechosos),
    )


def el_tope_de_canales_no_se_duplica():
    """`MAX_CANALES` sale del núcleo, no de un número escrito en el formulario.

    El formulario necesita el tope para deshabilitar «Añadir otra». Si lo
    llevara suelto, el día que el núcleo suba a quince el formulario seguiría
    cortando en diez — y nadie vería un error, sólo un botón apagado.
    """
    rust = leer("crates", "arles-core", "src", "contacto.rs")
    ts = leer("app", "src", "app", "stores", "contactos.ts")
    import re as _re
    m = _re.search(r"MAX_CANALES: usize = (\d+)", rust)
    m2 = _re.search(r"MAX_CANALES = (\d+)", ts)
    check(
        3,
        "el tope de canales coincide con el del núcleo",
        bool(m) and bool(m2) and m.group(1) == m2.group(1),
        el_tope_de_canales_no_se_duplica.__doc__,
        detalle=f"núcleo: {m.group(1) if m else '?'} · interfaz: {m2.group(1) if m2 else '?'}",
    )


def _bloque_de_lista(texto, nombre):
    """El contenido entre `= [` y su `]` de cierre, para una constante de TS.

    Se busca el `= [` y **no el primer corchete**: la declaración lleva el tipo
    delante —`readonly CampoImportable[]`— y el primer `]` está ahí. La primera
    versión de esto cortaba en ese corchete y devolvía una lista vacía, con lo
    que la comprobación pasaba sin mirar nada.
    """
    import re as _re
    m = _re.search(nombre + r"[^=]*=\s*\[(.*?)\]", texto, _re.S)
    return set(_re.findall(r"'([\w]+)'", m.group(1))) if m else set()


def las_listas_de_la_importacion_no_divergen():
    """Los campos y los orígenes de la pantalla son los del núcleo.

    Están escritos en los dos sitios: el núcleo decide qué acepta, y la interfaz
    tiene que ofrecer eso y nada más. Si divergieran, el desplegable ofrecería
    algo que el núcleo rechaza — y la importación se caería DESPUÉS de que el
    usuario haya revisado todos los choques y aceptado la declaración.
    """
    import re as _re
    rust = leer("crates", "arles-core", "src", "importacion.rs")
    ts = leer("app", "src", "app", "stores", "importacion.ts")

    # Los orígenes, por su texto guardado: es lo que viaja por la IPC.
    en_rust = set(_re.findall(r'Self::\w+ => "([a-z]+(?:_[a-z]+)*)",', rust))
    en_rust = {o for o in en_rust if "_" in o or o == "otro"}
    en_ts = _bloque_de_lista(ts, "export const ORIGENES")
    check(
        3,
        "los orígenes de la importación coinciden con los del núcleo",
        bool(en_rust) and en_rust == en_ts,
        las_listas_de_la_importacion_no_divergen.__doc__,
        detalle=f"núcleo: {sorted(en_rust)}\ninterfaz: {sorted(en_ts)}",
    )

    # Y los campos importables.
    #
    # Se leen de la prueba `cada_campo_viaja_con_su_nombre_exacto`, que fija el
    # JSON de cada uno y lo comprueba contra serde de verdad. NO se derivan del
    # nombre del enum: la primera versión de esto lo hacía —pasar `WhatsApp` a
    # camelCase— y daba `whatsApp`, que es lo que serde produce **sin** el
    # `rename`. Es decir, la comprobación reproducía el fallo en vez de cazarlo.
    #
    # Así hay una sola cadena de custodia: la prueba de Rust ata el enum al
    # JSON, y esto ata el JSON a la interfaz.
    bloque = _re.search(
        r"fn cada_campo_viaja_con_su_nombre_exacto.*?\n    \}", rust, _re.S
    )
    del_nucleo = (
        set(_re.findall(r'\\"(\w+)\\"', bloque.group(0))) if bloque else set()
    )
    en_pantalla = _bloque_de_lista(ts, "export const CAMPOS")
    check(
        3,
        "los campos importables coinciden con los del núcleo",
        bool(del_nucleo) and del_nucleo == en_pantalla,
        las_listas_de_la_importacion_no_divergen.__doc__,
        detalle=f"núcleo: {sorted(del_nucleo)}\ninterfaz: {sorted(en_pantalla)}",
    )


def la_bomba_se_mide_antes_de_abrir_la_hoja():
    """El tope de descompresión va ANTES de abrir el XLSX, o no sirve de nada.

    Este es el orden que arregló el fallo del 18/09/2026. `calamine` construye
    la hoja entera en memoria al abrirla, así que medir después es medir cuando
    la memoria ya se gastó: un archivo de 47 MB hacía subir el proceso 1 586 MB
    y **devolvía el error correcto igualmente**.

    Ésa es la razón de que esto se compruebe aquí y no sólo con una prueba de
    unidad: invertir las dos líneas no rompe ningún test que mire el error.
    Sólo `tests/bomba.rs`, que mide la memoria, lo caza — y sólo si alguien se
    acuerda de correrlo. Esto no se olvida.
    """
    fuente = leer("crates", "arles-import", "src", "lib.rs")
    medir = fuente.find("medir_la_descompresion(ruta)?")
    abrir = fuente.find("calamine::open_workbook_auto")
    check(
        3,
        "la descompresión se mide antes de abrir la hoja",
        medir != -1 and abrir != -1 and medir < abrir,
        la_bomba_se_mide_antes_de_abrir_la_hoja.__doc__,
        detalle=(
            f"llamada a medir_la_descompresion: {medir}\n"
            f"llamada a open_workbook_auto:    {abrir}"
        ),
    )


def el_texto_legal_se_ve_provisional():
    """Mientras P-09 siga abierta, el texto de la declaración lo dice.

    ADR-0013 §1 permite construir el mecanismo con texto provisional, con una
    condición: que **se vea como tal**. Un aviso que cita una ley abrogada es
    peor que no citar ninguna, porque aparenta rigor.

    Cuando llegue la respuesta del abogado, esta comprobación hay que quitarla
    a mano — y que haya que quitarla es justo lo que impide olvidarse.
    """
    import re as _re
    ts = leer("app", "src", "app", "stores", "importacion.ts")
    # Hasta la línea en blanco: este proyecto no usa punto y coma, así que
    # cortar en `;` no encontraba nada y la comprobación pasaba con el texto
    # vacío. Se vio al escribirla.
    m = _re.search(r"export const TEXTO_PROVISIONAL =(.*?)\n\n", ts, _re.S)
    texto = m.group(1) if m else ""
    check(
        3,
        "el texto de la declaración se ve marcado como provisional",
        "P-09" in texto and "PENDIENTE" in texto.upper(),
        el_texto_legal_se_ve_provisional.__doc__,
        detalle=texto.strip()[:160] or "(no se encontró la constante)",
    )


def fase_3(rapido):
    titulo("FASE 3 · Empresa y contactos · entrega 3.1")

    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("crates/arles-core/src/empresa.rs", "La validación de la empresa es del dominio, no del formulario."),
        ("crates/arles-core/src/onboarding.rs", "La lista de alta se deriva de los datos; sin este módulo volvería a ser un booleano por paso."),
        ("crates/arles-db/migrations/V2__preferencias_de_interfaz.sql", "P-11 dice «se recuerda», y eso necesita una tabla."),
        ("crates/arles-db/src/empresa.rs", "Sin repositorio, el SQL se derramaría al shell."),
        ("crates/arles-core/src/contacto.rs", "Un contacto es una persona con canales, y quien decide si es válido es el dominio (L-13)."),
        ("crates/arles-db/src/contactos.rs", "Alta, edición, ficha y baja de contactos con sus canales."),
        ("app/src/app/stores/contactos.ts", "El estado de la lista y el reparto de errores por canal."),
        ("app/src/app/pantallas/PantallaContactos.vue", "La tabla, la ficha y las acciones (L-13, L-14)."),
        ("app/src/app/pantallas/FormularioDeContacto.vue", "Alta y edición a mano, con sus canales (L-13)."),
        ("crates/arles-import/src/lib.rs", "Lectura defensiva de CSV y XLSX: el archivo que llega no es de fiar."),
        ("crates/arles-core/src/analisis.rs", "Qué pasaría al importar, sin escribir nada."),
        ("app/src/app/pantallas/PantallaImportar.vue", "El asistente de importación de cuatro pasos (3.3)."),
        ("app/src/app/pantallas/PantallaAjustes.vue", "Es el primer paso del alta (§25)."),
        ("app/src/app/pantallas/PantallaInicio.vue", "La lista de alta vive en Inicio."),
        ("app/src/app/stores/interfaz.ts", "El estado de la barra lateral (P-11)."),
        ("documentacion/06-calidad/UMBRAL_DE_PLEGADO.md", "El umbral está medido: sin el documento, el número vuelve a ser una opinión."),
    ]:
        check(3, f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    print(f"{GRIS}  invariantes{FIN}")
    migraciones_no_se_editan()
    avance_no_diverge()
    listas_cerradas_del_nucleo()
    claves_de_texto_del_nucleo()
    claves_de_error_sin_texto()
    nombres_de_las_opciones()
    iconos_de_seccion()
    preferencias_fuera_del_navegador()
    contactos_no_entran_sin_validar()
    la_capa_de_datos_no_conoce_la_ipc()
    los_comandos_de_contactos_no_reciben_la_empresa()
    la_pantalla_de_contactos_no_reordena()
    el_tope_de_canales_no_se_duplica()
    las_listas_de_la_importacion_no_divergen()
    la_bomba_se_mide_antes_de_abrir_la_hoja()
    el_texto_legal_se_ve_provisional()

    print(f"{GRIS}  sondas{FIN}")
    app = os.path.join(RAIZ, "app")
    motivo = hay_navegador(app)
    nombre = "sonda: el umbral de plegado está medido y se aplica"
    porque = (
        "«A mano y sola» necesita un número, y un número elegido a ojo es lo que "
        "produjo R-01. La sonda falla por los dos lados: si el umbral se queda "
        "corto la pantalla no alcanza su medida, y si se infla la barra se pliega "
        "sola donde cabe holgada."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:plegado", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: la navegación no salta al plegar la barra"
    porque = (
        "Dirección vio los iconos saltar, y medido saltaban 40 px: la marca "
        "desaparecía al plegar y arrastraba hacia arriba todo lo de abajo. Es "
        "una afirmación geométrica, así que se mide en vez de mirarse."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:cabecera", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: el desplegable de tema hace lo que dice"
    porque = (
        "La paleta clara estaba medida desde el paso 2 pero no había forma de "
        "elegirla. Las cuatro afirmaciones del desplegable —cambia el color, "
        "se recuerda, «Automático» sigue al sistema en caliente, y las "
        "opciones son las que el núcleo acepta— son sobre el navegador y sobre "
        "la persistencia, y ninguna se puede comprobar con un test de unidad."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:tema", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: el asistente de importación hace lo que dice"
    porque = (
        "Cinco afirmaciones de la 3.3 que ningún test de unidad alcanza: que el "
        "mapeo se propone solo y se corrige, que analizar no escribe nada, que "
        "el informe da cifras, que NO se puede importar sin aceptar la "
        "declaración de origen, y que el móvil con el «1» mexicano llega sin "
        "él. Esta sonda ya encontró un fallo real: los botones dentro de "
        "EstadoVacio no se pintaban, porque la ranura tiene nombre."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:importar", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: la pantalla de contactos hace lo que dice"
    porque = (
        "Cuatro afirmaciones de la 3.2 que ningún test de unidad alcanza: que "
        "el alta aparece en la tabla, que la ficha enseña TODOS los canales "
        "agrupados (L-14), que una dirección repetida se nombra sin cerrar el "
        "diálogo, y que el móvil mexicano se guarda sin el «1». Esta sonda ya "
        "encontró un fallo real: el diálogo conservaba el contacto anterior, "
        "así que el segundo alta chocaba con una dirección que el usuario no "
        "había escrito."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:contactos", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: la vista previa de un archivo se abre desde el disco"
    porque = (
        "Es lo que Dirección abre con doble clic para revisar. Que el archivo "
        "exista no dice nada: dos versiones se generaron sin error y salieron "
        "EN BLANCO —por los trozos del enrutador y por la CSP del producto, que "
        "prohíbe el script en línea—. Se mide desde `file://`, que es como se "
        "va a abrir; servido por HTTP los dos fallos desaparecen."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:vista-previa", "--silent"],
                  porque, cwd=app, timeout=900)


FASES = {0: fase_0, 1: fase_1, 2: fase_2, 3: fase_3}


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
