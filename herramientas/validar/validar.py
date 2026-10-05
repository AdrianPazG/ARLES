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

    # ── La documentación dice la verdad ──────────────────────────────────
    #
    # Se añade el 22/09/2026 después de que Dirección pidiera revisar la
    # documentación entera. Son 66 archivos: releerlos no escala, y la
    # documentación no falla ruidosamente sino quedándose atrás.
    #
    # En su primer recorrido encontró cuatro cosas que ya no eran ciertas —tres
    # documentos diciendo «ocho preguntas» cuando son dieciocho, y un rango
    # «C-9 a C-18» etiquetado como nueve cuando son diez— y diez documentos que
    # el índice no nombraba, entre ellos la consulta jurídica y el informe de
    # fase para Dirección.
    #
    # Va en la Fase 0 porque la documentación no es de ninguna fase: es de
    # todas, y se queda atrás justamente cuando avanzan las otras.
    check_cmd(0, "la documentación dice la verdad",
              [sys.executable, os.path.join(RAIZ, "herramientas/validar/documentos.py")]
              + (["--rapido"] if rapido else []),
              "Un enlace que se rompió, una cifra que caducó o un anexo prometido y "
              "nunca escrito no rompen nada: hacen que Dirección lea algo que no es "
              "cierto. Es el mismo daño que hizo la vista previa desfasada — el "
              "problema no fue el defecto, fue revisar sobre una copia que mentía.")


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
# ── Tokens ─────────────────────────────────────────────────────────────────
#
# CUIDADO CON ESTAS DOS EXPRESIONES · corregidas el 22/09/2026
#
# Decían `--arles-[a-z0-9-]+`, las dos. Es decir: la comprobación de «todo
# token que se usa está definido» **sólo miraba los que ya empiezan por
# `--arles-`**. Cazaba una errata dentro de la familia —`--arles-spacee-3`— y
# era ciega a un nombre de otra familia entera.
#
# Que es exactamente lo que pasó: `PantallaSupresion.vue` se escribió con
# catorce tokens inventados —`--espacio-3`, `--color-texto-suave`,
# `--texto-sm`…— y la hoja de estilos entera quedó muerta sin que nada lo
# dijera. La comprobación pasaba en verde porque **no podía fallar**.
#
# Es el caso de manual de la regla de trabajo: una comprobación que nunca se
# ha visto fallar no está verificada. Ésta llevaba desde la Fase 2 sin poder.
RE_USO_DE_TOKEN = re.compile(r"var\(\s*(--[a-zA-Z0-9_-]+)")
RE_DEF_DE_TOKEN = re.compile(r"(--[a-zA-Z0-9_-]+)\s*:")


def sin_comentarios(texto):
    return RE_COMENTARIO_JS.sub("", RE_COMENTARIO_CSS.sub("", texto))


RE_COMENTARIO_HTML = re.compile(r"<!--.*?-->", re.S)


def sin_comentarios_ni_html(texto):
    """Como `sin_comentarios`, y además quita los `<!-- … -->` de la plantilla.

    Hace falta en cuanto una comprobación busca una cadena que el archivo
    **menciona para explicar por qué no la usa**. Pasó el 21/09/2026: dos
    comprobaciones nuevas fallaron contra los comentarios que documentaban la
    regla que ellas mismas vigilan.

    No es un detalle de fontanería: una comprobación que se dispara con un
    comentario enseña a no comentar, y en este proyecto el comentario es donde
    vive el porqué.
    """
    return RE_COMENTARIO_HTML.sub("", sin_comentarios(texto))


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

    # Un archivo puede declarar variables **suyas** —`--arles-medida: 540px` en
    # PantallaInicio.vue— y usarlas. Eso es legítimo y no es un huérfano, así
    # que lo que cada archivo define localmente se suma a lo global sólo para
    # ese archivo. Sin esta parte, ampliar la expresión de arriba habría
    # producido una lluvia de falsos positivos y la comprobación se habría
    # acabado relajando otra vez.
    huerfanos = set()
    for base, _, archivos in os.walk(os.path.join(RAIZ, "app/src")):
        for n in archivos:
            if not n.endswith((".vue", ".css", ".ts")):
                continue
            with open(os.path.join(base, n), encoding="utf-8") as f:
                # Sin comentarios, por lo mismo de siempre: `PantallaSupresion`
                # EXPLICA en su cabecera que decía `var(--borde-1)` y por qué
                # eso la dejó muerta. Leer el comentario haría que la
                # comprobación se disparara contra su propia documentación.
                cuerpo = sin_comentarios_ni_html(f.read())
            locales = set(RE_DEF_DE_TOKEN.findall(cuerpo))
            for token in RE_USO_DE_TOKEN.findall(cuerpo):
                if token not in definidos and token not in locales:
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
    # La parada de emergencia, guardada para que sobreviva a cerrar el
    # programa. Entrega 4.4, 23/09/2026.
    "V6__parada_de_emergencia.sql":
        "e70450e110aaddc263edcb1be00be9665b9cfbca0dfdbd9ab0fb695799b77218",
    # Las cuatro columnas de SMTP que faltaban en `email_account` desde la V1.
    # Entrega 5.3, 23/09/2026.
    "V7__cuenta_smtp.sql":
        "32684912c4b913f1ff64ae158c92dc265efd82c3d1bf6fef408477ae62f267fd",
    # Dónde consta el envío de prueba de una etapa, y con qué cuenta se hizo.
    # Entrega 5.5, 28/09/2026.
    "V8__envio_de_prueba.sql":
        "6109f2d428222cdb69711312fc4f387bf38e038da8cd38276364d57c7e70c206",
    # Las plantillas de Meta y el recuento de respuestas.
    # Entrega W.1, 29/09/2026.
    "V9__whatsapp.sql":
        "3d9078e63677ee3e49a39cb83a05b94d819fe577b0fdc3efec816aa9f008f4e4",
    # El registro de actividad: qué fue pasando y cuándo.
    # Entrega 7.1, 30/09/2026.
    "V10__registro_de_actividad.sql":
        "e8a4a7338c20ca88a766c6713041abf7587b0c3bb6e86943b35d9b9c548f4cd9",
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


def _esqueleto_del_catalogo(fuente):
    """El catálogo sin comentarios ni textos, para poder contar sus llaves.

    Las dos cosas hay que quitarlas **antes** de contar llaves, y las dos las
    aprendí rompiéndolo:

    - Un texto como `'… {fallan} por arreglar'` lleva una `}` dentro. Contándola,
      el nivel de anidación baja uno por cada marcador, y a mitad del archivo el
      catálogo parece tener la mitad de las rutas que tiene.
    - Un comentario de bloque puede llevar las llaves que quiera.

    Con los dos defectos puestos, `preflight.pasa` —que existe desde la 4.5— se
    leía como inexistente.
    """
    s = re.sub(r"/\*.*?\*/", "", fuente, flags=re.S)
    s = re.sub(r"//[^\n]*", "", s)
    s = re.sub(r"'(?:\\.|[^'\\\n])*'", "·", s)
    s = re.sub(r'"(?:\\.|[^"\\\n])*"', "·", s)
    return re.sub(r"`(?:\\.|[^`\\])*`", "·", s, flags=re.S)


def rutas_del_catalogo(fuente):
    """Las rutas punteadas que el catálogo define, **nodos incluidos**.

    Un nodo cuenta porque hay claves que apuntan a uno: `resolverError` pide
    `error.db.clave_incorrecta`, que no es una cadena sino `{que, como, salvo}`.
    """
    pila, rutas = [], set()
    for linea in _esqueleto_del_catalogo(fuente).splitlines():
        cuerpo = linea.strip()
        if not cuerpo:
            continue
        m = re.match(r"^'?([A-Za-z_]\w*)'?\s*:\s*\{", cuerpo)
        if m:
            pila.append(m.group(1))
            rutas.add(".".join(pila))
            continue
        m = re.match(r"^'?([A-Za-z_]\w*)'?\s*:", cuerpo)
        if m:
            # Vale tanto `clave: 'texto'` como `clave:` con el valor debajo: el
            # formateador parte los textos largos, y buscar algo tras los dos
            # puntos dejaba fuera justo los más largos.
            rutas.add(".".join(pila + [m.group(1)]))
        for ch in cuerpo:
            if ch == "}" and pila:
                pila.pop()
    return rutas


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
    # `arles-email` entró en la lista el 23/09/2026, al romper esta
    # comprobación a propósito: se borró el texto de un error nuevo del
    # adaptador de correo y **la comprobación siguió pasando**, porque el crate
    # no estaba en esta lista. Es el mismo fallo que ya cazó dos veces en la
    # Fase 4: una comprobación que dice mirar el §139 entero y mira una parte.
    for crate in ("arles-core", "arles-db", "arles-app", "arles-email"):
        d = os.path.join(RAIZ, "crates", crate, "src")
        for archivo in os.listdir(d):
            if archivo.endswith(".rs"):
                texto = leer(f"crates/{crate}/src", archivo) or ""
                claves |= set(re.findall(r'=> "(error\.[a-z_.]+)"', texto))

    es = leer("app/src/app/locales", "es.ts") or ""
    spec = leer("app/src/app", "errores.spec.ts") or ""

    # En el catálogo cada error es un objeto anidado, así que se busca la última
    # parte de la clave seguida de `: {`.
    #
    # ── LO QUE ESTA COMPROBACIÓN NO VE ──
    #
    # Sólo mira la HOJA, no la ruta. Un texto escrito en la rama equivocada
    # —`error.cuenta_de_correo_invalida` cuando la clave es
    # `error.app.cuenta_de_correo_invalida`— la pasa sin protestar, y en la
    # pantalla la clave no resuelve igualmente.
    #
    # Pasó el 23/09/2026, con esa clave exacta: el validador dijo que todo
    # estaba bien y quien lo cazó fue `errores.spec.ts`, que sí resuelve la ruta
    # entera contra el catálogo compilado. Se deja así a propósito —escribir
    # aquí un segundo intérprete de TypeScript sería peor que el problema—, y se
    # anota para que nadie confíe en esta comprobación más de lo que aguanta:
    # la que decide es la del frontend, y por eso la otra comprobación de aquí
    # exige que toda clave esté en su lista.
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

    # ── Y las claves que NO empiezan por «error.» ────────────────────────────
    #
    # La comprobación de arriba sólo miraba `error.*`, y el §139 no dice eso:
    # dice que TODA clave lleva texto. Al llegar la Fase 4 aparecieron cuarenta
    # y una claves de estado —`motor.*`, `campana.*`, `reintento.*`,
    # `ritmo.*`, `ventana.*`, `instancia.*`— y ninguna tenía texto, porque nada
    # las miraba. Las cuarenta y una habrían salido en pantalla como su propia
    # clave: «motor.fueraDeVentana».
    #
    # Se descubrió el 23/09/2026 al añadir las de pausar y detener, y es
    # exactamente el fallo del §2: una comprobación que parecía cubrir el §139
    # cubría un tercio de él.
    #
    # ─────────────────────────────────────────────────────────────────────────
    # Y VOLVIÓ A CUBRIR UNA PARTE. REESCRITA EL 01/10/2026, EN LA 7.7
    #
    # Tenía los dos defectos que ya había tenido una vez:
    #
    # 1 · **Una lista de prefijos escrita a mano** —`campana|motor|reintento|
    #     ritmo|ventana|instancia|etapa|duracion|preflight|disyuntor|correo`—.
    #     Faltaban `whatsapp` y `plantilla`, así que once claves no las miraba
    #     nadie: las seis de `whatsapp.prueba.*` que devuelve el envío de prueba
    #     y las cinco de `whatsapp.plantilla.*` de los estados de Meta.
    # 2 · **Comparaba la hoja, no la ruta.** Estaba anotado arriba como límite
    #     conocido, con el argumento de que `errores.spec.ts` resuelve la ruta
    #     de verdad — pero ese test sólo cubre `error.*`. Para todo lo demás
    #     nadie resolvía la ruta, y así `plantilla.variable.miEmpresa` pasaba en
    #     verde teniendo el catálogo `mi_empresa`: la hoja `mi_empresa` existía,
    #     la ruta `…miEmpresa` no, y el asistente habría enseñado la clave cruda.
    #
    # Ahora los prefijos **son los bloques del propio catálogo** y se resuelve
    # la ruta entera. La regla es: si el primer tramo nombra un bloque de la
    # interfaz, la ruta completa tiene que resolver dentro de él. Eso deja fuera
    # lo que sólo se parece a una clave —`smtp.gmail.com`, `lista.csv`,
    # `wamid.ABC`, `v21.0`— sin ninguna lista que mantener.
    rutas = rutas_del_catalogo(es)
    bloques = {r for r in rutas if "." not in r}
    assert len(bloques) > 20, f"se leyeron sólo {len(bloques)} bloques del catálogo"

    # ── Las acciones de la bitácora NO son claves de i18n ───────────────────
    #
    # `audit_log` guarda acciones con la misma forma punteada, y dos de ellas
    # caen bajo un bloque que sí existe en la interfaz. No se enseñan en ninguna
    # pantalla: el registro de Actividad (7.4) lee `activity_event`, que es otra
    # tabla. La excepción se declara aquí, y **se comprueba**: si una de estas
    # dejara de ser una acción de bitácora, esta comprobación falla en vez de
    # tragarse una clave de verdad.
    ACCIONES_DE_BITACORA = ("empresa.actualizada", "empresa.configurada")
    cuerpo_empresa = leer("crates/arles-db/src", "empresa.rs") or ""
    assert "auditar(" in cuerpo_empresa, (
        "las excepciones de bitácora se justifican por `auditar(`, y ya no está"
    )

    otras = set()
    for crate in ("arles-core", "arles-db", "arles-motor", "arles-app", "arles-email",
                  "arles-whatsapp", "arles-import"):
        d = os.path.join(RAIZ, "crates", crate, "src")
        if not os.path.isdir(d):
            continue
        for archivo in sorted(os.listdir(d)):
            if not archivo.endswith(".rs"):
                continue
            texto = leer(f"crates/{crate}/src", archivo) or ""
            # Se quitan los tests: sus cadenas no van a ninguna pantalla.
            cuerpo = texto.split("\n#[cfg(test)]")[0]
            cuerpo = re.sub(r"//[^\n]*", "", cuerpo)
            for m in re.finditer(r'"([a-z][a-zA-Z0-9]*(?:\.[a-zA-Z0-9_]+)+)"', cuerpo):
                clave = m.group(1)
                if clave.split(".")[0] in bloques and clave not in ACCIONES_DE_BITACORA:
                    otras.add(clave)

    otras_sin_texto = sorted(c for c in otras if c not in rutas)
    check(3, "toda clave de estado de Rust tiene texto en el catálogo",
          bool(otras) and not otras_sin_texto,
          "El §139 dice TODA clave, no sólo las de error. Sin texto, la "
          "pantalla enseña la clave cruda: «motor.fueraDeVentana». Se compara "
          "la RUTA entera, no la hoja: un texto en la rama equivocada no "
          "resuelve, y la hoja sola pasaba en verde.",
          f"claves: {sorted(otras)}\nsin texto: {otras_sin_texto}")


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
                # Se quitan los comentarios antes de mirar.
                #
                # Hacía falta: el 24/09/2026 esta comprobación marcó como
                # infractores los dos archivos de Remitentes, que **explican en
                # sus comentarios** que la contraseña NO va a `localStorage`.
                # Se encontraba a sí misma, igual que le pasó a la invariante
                # del TLS en la 5.2 y a la del secreto en la webview en la 5.3.
                # Es un patrón que en este proyecto ya lleva tres apariciones:
                # una comprobación que busca texto encuentra el texto que la
                # documenta.
                codigo = "\n".join(
                    l for l in f.read().splitlines()
                    if not l.strip().startswith(("*", "//", "/*", "<!--", "#"))
                )
                if "localStorage" in codigo:
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


def una_supresion_pedida_no_se_levanta_con_un_boton():
    """Quien pidió la baja no sale de la lista desde la aplicación.

    Es la regla de la entrega 3.4. Si `se_puede_levantar` dejara de depender de
    `lo_pidio_la_persona`, una baja que alguien pidió pasaría a poder deshacerse
    con un clic — y nada fallaría: la pantalla enseñaría el botón, el botón
    funcionaría, y esa persona volvería a recibir correos.

    Se comprueba sobre la fuente y no sólo con los tests porque el día que
    alguien «simplifique» esa función, los tests de ese módulo se tocan en el
    mismo commit.
    """
    import re as _re
    fuente = leer("crates", "arles-core", "src", "supresion.rs")
    m = _re.search(
        r"pub const fn se_puede_levantar\(self\) -> bool \{(.*?)\n    \}",
        fuente,
        _re.S,
    )
    cuerpo = m.group(1).strip() if m else ""
    check(
        3,
        "una supresión que pidió la persona no se levanta con un botón",
        cuerpo == "!self.lo_pidio_la_persona()",
        una_supresion_pedida_no_se_levanta_con_un_boton.__doc__,
        detalle=f"cuerpo de se_puede_levantar: {cuerpo!r}",
    )

    # Y que las dos que la persona pide sigan siendo las dos que son.
    m = _re.search(
        r"pub const fn lo_pidio_la_persona\(self\) -> bool \{(.*?)\n    \}",
        fuente,
        _re.S,
    )
    cuerpo = m.group(1).strip() if m else ""
    check(
        3,
        "la baja y la queja son lo que la persona pide",
        "Self::Baja" in cuerpo and "Self::Queja" in cuerpo,
        una_supresion_pedida_no_se_levanta_con_un_boton.__doc__,
        detalle=f"cuerpo de lo_pidio_la_persona: {cuerpo!r}",
    )


def la_vista_previa_no_miente_sobre_las_rutas():
    """El simulador de la vista previa lleva a donde lleva el núcleo.

    Esto se escribe el 22/09/2026 porque la auditoría encontró que la vista
    previa —que es **donde Dirección revisa**— seguía mandando «Configurar tu
    empresa» a `/ajustes` cuatro días después de que el núcleo lo cambiara a
    `/inicio/configuracion`. La aplicación real estaba bien. El simulador no.

    El coste fue concreto: Dirección reportó el mismo defecto dos veces, y la
    segunda vez el problema ya no existía en el producto. Una vista previa que
    no corresponde al código convierte cada revisión visual en ruido.

    Ninguna prueba podía cazarlo: las de Rust miran el núcleo, las del frontend
    miran los componentes, y el simulador es un literal de JavaScript dentro de
    un generador de Python que nadie compila.
    """
    import re as _re
    nucleo = leer("crates", "arles-core", "src", "onboarding.rs")
    # El simulador salió de `generar.py` a su propio archivo el 29/09/2026: eran
    # diecinueve mil caracteres de JavaScript dentro de una cadena de Python.
    # Esta comprobación seguía leyendo el generador y dejó de encontrar nada —
    # y un `del_simulador` vacío la hacía fallar, que es lo correcto: prefiere
    # gritar antes que dar por buena una comparación contra la nada.
    generador = leer("herramientas", "vista-previa", "simulador.js")

    # Las rutas del núcleo: `Self::Empresa => Some("/inicio/configuracion"),`
    cuerpo = _re.search(
        r"pub fn ruta\(self\) -> Option<&'static str> \{(.*?)\n    \}",
        nucleo,
        _re.S,
    )
    del_nucleo = dict(
        _re.findall(r'Self::(\w+)\s*=>\s*Some\("([^"]+)"\)', cuerpo.group(1) if cuerpo else "")
    )

    # ── LA CLAVE SE LEE, NO SE ADIVINA ───────────────────────────────────
    #
    # Esto comparaba contra el nombre de la variante en minúsculas, dando por
    # hecho que `PrimeraCampana` viaja como «primeracampana». Viaja como
    # «campana», que es lo que devuelve `clave()`. Resultado: la comprobación
    # denunciaba una divergencia que no existía, y lo hizo el 29/09/2026 —una
    # hora de buscar un defecto en el simulador, que estaba bien—.
    #
    # Una comprobación que se inventa la mitad del contrato no verifica el
    # contrato: verifica su propia suposición.
    cuerpo_clave = _re.search(
        r"pub fn clave\(self\) -> &'static str \{(.*?)\n    \}", nucleo, _re.S
    )
    clave_de = dict(
        _re.findall(
            r'Self::(\w+)\s*=>\s*"([^"]+)"', cuerpo_clave.group(1) if cuerpo_clave else ""
        )
    )

    # Las del simulador: `['empresa', hay, true, '3.1', '/inicio/configuracion'],`
    #
    # Se lee hasta el final de la LÍNEA y no hasta el primer `]`. Con el corchete
    # la fila de contactos —`(e.contactos || []).length > 0`— era invisible: el
    # `[]` de la lista vacía cerraba el patrón antes de tiempo. La comprobación
    # llevaba desde entonces sin mirar ese paso y no lo decía.
    del_simulador = {
        clave: ruta
        for clave, ruta in _re.findall(
            r"\[\s*'(\w+)',[^\n]*?'[\d.]+',\s*'([^']+)'\s*\]", generador
        )
    }

    # Se comparan por la clave con la que el núcleo las nombra para el frontend.
    esperado = {clave_de.get(k, k.lower()): v for k, v in del_nucleo.items()}
    divergen = [
        f"{k}: el núcleo dice {esperado.get(k, 'sin ruta')!r}, la vista previa {v!r}"
        for k, v in del_simulador.items()
        if esperado.get(k) != v
    ]
    # Y al revés: un paso que el núcleo ya sabe llevar y el simulador no.
    divergen += [
        f"{k}: el núcleo lleva a {v!r} y la vista previa no lleva a ningún sitio"
        for k, v in esperado.items()
        if k not in del_simulador
    ]

    check(
        3,
        "la vista previa lleva a donde lleva el núcleo",
        del_simulador != {} and divergen == [],
        la_vista_previa_no_miente_sobre_las_rutas.__doc__,
        detalle=(
            "no se pudo leer ninguna ruta del simulador"
            if del_simulador == {}
            else "\n".join(divergen)
        ),
    )


def el_informe_de_importacion_consulta_las_supresiones():
    """Analizar y confirmar preguntan los dos por la lista de supresión.

    Dirección decidió el 21/09/2026 que un archivo con gente que ya pidió la
    baja se enseñe antes de importar. Esa consulta tiene que estar en los DOS
    comandos, y el de confirmar es el que se olvida: analizar enseña el informe
    y confirmar escribe, así que quitarla de `confirmar_importacion` no rompe
    ninguna pantalla — sólo hace que la cifra guardada sea la de antes de la
    última baja.
    """
    fuente = leer("crates", "arles-app", "src", "comandos.rs")
    import re as _re
    faltan = []
    for comando in ("analizar_importacion", "confirmar_importacion"):
        m = _re.search(
            rf"pub fn {comando}\(.*?\n\}}",
            fuente,
            _re.S,
        )
        cuerpo = m.group(0) if m else ""
        if "motivos_de_supresion" not in cuerpo:
            faltan.append(comando)
    check(
        3,
        "el informe de importación consulta la lista de supresión",
        not faltan,
        el_informe_de_importacion_consulta_las_supresiones.__doc__,
        detalle=f"sin consultar las supresiones: {faltan}",
    )


def el_informe_de_acceso_no_deja_cruzar_una_ruta():
    """Ninguna ruta del sistema de archivos cruza hacia la webview.

    Cuarta regla de frontera (ARQUITECTURA.md §5). El informe de acceso se
    guarda con el diálogo del sistema, que se abre DESDE RUST: la webview manda
    una dirección y recibe el nombre del archivo, nunca la carpeta.

    Se comprueba sobre el tipo que cruza y no sólo con un test porque el error
    es de una palabra: cambiar `file_name()` por `to_string_lossy()` al armar
    la respuesta manda la ruta entera, y nada falla — la pantalla enseña un
    nombre más largo y ya. La carpeta donde alguien guarda sus cosas dice dónde
    vive (THREAT_MODEL.md §4.1).
    """
    import re as _re
    fuente = leer("crates", "arles-app", "src", "comandos.rs")

    m = _re.search(r"pub struct InformeEntregado \{(.*?)\n\}", fuente, _re.S)
    campos = m.group(1) if m else ""
    sospechosos = [
        c for c in ("ruta", "path", "carpeta", "directorio")
        if _re.search(rf"\n\s+pub \w*{c}\w*:", campos)
    ]
    check(
        3,
        "el informe de acceso no devuelve ninguna ruta",
        bool(campos) and not sospechosos,
        el_informe_de_acceso_no_deja_cruzar_una_ruta.__doc__,
        detalle=f"campos sospechosos: {sospechosos}",
    )

    # Y que el nombre se saque con `file_name`, que es lo que lo garantiza.
    m = _re.search(
        r"pub fn entregar_informe_de_acceso\(.*?\n\}", fuente, _re.S
    )
    cuerpo = m.group(0) if m else ""
    check(
        3,
        "el nombre del archivo entregado se saca sin la carpeta",
        "file_name()" in cuerpo and "to_string_lossy" not in cuerpo,
        el_informe_de_acceso_no_deja_cruzar_una_ruta.__doc__,
        detalle="se esperaba `file_name()` y ningún `to_string_lossy`",
    )


def ajustes_es_de_la_aplicacion_y_no_del_negocio():
    """Ajustes no lleva el formulario de la empresa ni la lista de pasos.

    Observación de Dirección del 21/09/2026. Ajustes es para la aplicación
    —tema, densidad, versión, respaldos—; el nombre comercial, el país y la
    zona horaria son datos del negocio que deciden a qué hora sale cada correo.

    Se vigila aquí porque el fallo no rompe nada visible: duplicar el
    formulario en Ajustes «funcionaría», y dos formularios sobre la misma
    tabla divergen en cuanto alguien toca uno.

    Y porque el botón llevaba a Ajustes **desde la 3.1** sin que nada lo
    detectara: el stub de RouterLink de las pruebas no pintaba `href`, así que
    ninguna podía comprobar a dónde lleva un enlace.
    """
    ajustes = leer("app", "src", "app", "pantallas", "PantallaAjustes.vue")
    sobrantes = [
        c for c in ("nombreComercial", "correoCorporativo", "zonaHoraria", "onboarding")
        if c in ajustes
    ]
    check(
        3,
        "Ajustes no lleva los datos del negocio",
        not sobrantes,
        ajustes_es_de_la_aplicacion_y_no_del_negocio.__doc__,
        detalle=f"lo que se coló en Ajustes: {sobrantes}",
    )

    # Y el primer paso del alta lleva a la configuración, no a Ajustes. La
    # ruta la decide el NÚCLEO, así que se comprueba ahí: la pantalla sólo la
    # pinta, y cambiarla en la pantalla sin tocar el núcleo dejaría las dos
    # diciendo cosas distintas.
    nucleo = leer("crates", "arles-core", "src", "onboarding.rs")
    check(
        3,
        "el primer paso del alta no manda a Ajustes",
        'Self::Empresa => Some("/inicio/configuracion")' in nucleo,
        ajustes_es_de_la_aplicacion_y_no_del_negocio.__doc__,
        detalle="se esperaba que PasoDeOnboarding::Empresa apunte a /inicio/configuracion",
    )

    # Y tampoco queda el PUENTE. Dirección lo mandó quitar el 21/09/2026:
    # «no me gusta que la opción de ir a configuración se encuentre en esta
    # sección, elimínala, junto con el texto».
    #
    # Ese bloque existía sólo porque la configuración no salía en la barra
    # lateral — era un parche a un fallo de navegación. Con la barra
    # enseñando «Inicio › Configuración» sobra, y lo único que hacía era
    # devolverle a Ajustes el vocabulario del negocio.
    # Se mira el CÓDIGO, no los comentarios: esta pantalla EXPLICA en su
    # cabecera por qué el puente ya no está, y la primera versión de esta
    # comprobación falló por leer su propia documentación. Una comprobación
    # que se dispara con un comentario acaba empujando a no comentar.
    check(
        3,
        "Ajustes no enlaza a la configuración",
        "/inicio/configuracion" not in sin_comentarios_ni_html(ajustes),
        ajustes_es_de_la_aplicacion_y_no_del_negocio.__doc__,
        detalle="volvió el puente a la configuración dentro de PantallaAjustes.vue",
    )


def la_barra_dice_en_que_seccion_estas():
    """La sección activa se marca desde `meta.seccion`, no con `router-link-active`.

    Observación de Dirección del 21/09/2026, la segunda sobre lo mismo: entró
    en la configuración y **la barra lateral se quedó entera apagada**.

    La causa, medida: `router-link-active` no compara rutas por texto, compara
    si el registro de la ruta actual DESCIENDE del registro del enlace. Y
    `/inicio/configuracion` está declarada como hermana de `/inicio`, no como
    hija. Lo mismo les pasaba a `/contactos/importar` y `/contactos/supresion`
    **desde la entrega 3.2**, y nadie lo vio porque las pruebas de pantalla
    sustituyen `RouterLink` por un doble que no calcula estado activo.

        /inicio/configuracion  → casa con  ['/inicio/configuracion']
        /contactos/importar    → casa con  ['/contactos/importar']

    Se vigila aquí y no sólo en las pruebas porque volver al marcado
    automático es la clase de cambio que parece una simplificación: «para qué
    calcular esto si la librería ya lo hace».
    """
    # Igual que arriba: `App.vue` documenta en su cabecera por qué no se usa
    # `router-link-active`, y leer el archivo crudo daría un falso positivo
    # contra el propio comentario que explica la regla.
    app_vue = sin_comentarios_ni_html(leer("app", "src", "app", "App.vue"))

    check(
        3,
        "la barra no se marca con el activo automático de vue-router",
        "router-link-active" not in app_vue,
        la_barra_dice_en_que_seccion_estas.__doc__,
        detalle="`router-link-active` no llega a la sección madre en una subpantalla",
    )

    check(
        3,
        "la sección activa se calcula desde meta.seccion",
        "seccion === seccionActual" in app_vue,
        la_barra_dice_en_que_seccion_estas.__doc__,
        detalle="se esperaba que el enlace de sección compare con `seccionActual`",
    )

    # Toda ruta que se declare subsección tiene su texto. Sin esto, la barra
    # enseñaría la clave cruda —«nav.sub.plantillas»— el día que alguien añada
    # una y se olvide del catálogo (§139).
    router_ts = leer("app", "src", "app", "router.ts")
    es_ts = leer("app", "src", "app", "locales", "es.ts")
    declaradas = set(re.findall(r"subseccion:\s*'([a-z]+)'", router_ts))
    check(
        3,
        "hay al menos una subsección declarada",
        bool(declaradas),
        la_barra_dice_en_que_seccion_estas.__doc__,
        detalle="sin subsecciones, la comprobación de textos no comprueba nada",
    )

    bloque = re.search(r"\n    sub: \{(.*?)\n    \},", es_ts, re.S)
    con_texto = set(re.findall(r"(\w+):", bloque.group(1))) if bloque else set()
    faltan = sorted(declaradas - con_texto)
    check(
        3,
        "toda subsección de la barra tiene nombre en español",
        not faltan,
        la_barra_dice_en_que_seccion_estas.__doc__,
        detalle=f"subsecciones sin texto en nav.sub: {faltan}",
    )


def la_pantalla_de_acceso_no_pide_rutas_ni_dialogos():
    """La pantalla de Derechos no toca el disco ni ve una ruta.

    El informe de Acceso se escribe en un archivo, y la tentación evidente al
    construir la pantalla es abrir el diálogo de guardado desde la webview con
    el plugin de Tauri. Sería más corto y estaría mal por dos motivos:

      - **Regla de frontera 4** (ARQUITECTURA.md §5): ninguna ruta del sistema
        de archivos cruza hacia la webview. La carpeta donde alguien guarda
        sus cosas dice dónde vive (THREAT_MODEL.md §4.1).
      - Obligaría a conceder `dialog:allow-save` y `fs:allow-write` a la
        webview. Un permiso que no se concede no se puede abusar, y esta
        aplicación deniega por defecto.

    El diálogo lo abre **Rust**, y la pantalla recibe sólo el NOMBRE del
    archivo. Esta comprobación existe porque el atajo no rompe nada visible:
    funcionaría, y la fuga sería silenciosa.

    La sonda mide lo mismo por el otro lado —que en pantalla no aparezca una
    ruta—, pero sólo puede ver lo que se pinta. Ésta mira el código.
    """
    pantalla = os.path.join(
        RAIZ, "app/src/app/pantallas/PantallaDerechos.vue"
    )
    check(
        3,
        "existe la pantalla del derecho de Acceso",
        os.path.isfile(pantalla),
        la_pantalla_de_acceso_no_pide_rutas_ni_dialogos.__doc__,
        detalle="falta app/src/app/pantallas/PantallaDerechos.vue",
    )
    if not os.path.isfile(pantalla):
        return

    cuerpo = sin_comentarios_ni_html(leer("app", "src", "app", "pantallas", "PantallaDerechos.vue"))
    prohibidos = [
        p
        for p in ("plugin-dialog", "plugin-fs", "save(", "writeTextFile", "@tauri-apps/api/path")
        if p in cuerpo
    ]
    check(
        3,
        "la pantalla de Acceso no abre diálogos ni escribe archivos",
        not prohibidos,
        la_pantalla_de_acceso_no_pide_rutas_ni_dialogos.__doc__,
        detalle=f"lo que se coló en la webview: {prohibidos}",
    )

    # Y la ruta cuelga de CONTACTOS, no es una séptima sección. Las decisiones
    # sobre personas viven donde están las personas (UX_NAVEGACION.md §2), y
    # atender una solicitud es algo esporádico: una entrada permanente en la
    # barra desplazaría a las seis que se usan a diario.
    router = sin_comentarios_ni_html(leer("app", "src", "app", "router.ts"))
    check(
        3,
        "los derechos cuelgan de CONTACTOS",
        "'/contactos/derechos'" in router and "subseccion: 'derechos'" in router,
        la_pantalla_de_acceso_no_pide_rutas_ni_dialogos.__doc__,
        detalle="se esperaba la ruta /contactos/derechos declarada como subsección",
    )


def la_cancelacion_borra_de_verdad():
    """Cancelar es un `DELETE`, no un `deleted_at`, y deja siempre la baja.

    La LFPDPPP da derecho a un borrado **físico**. El atajo evidente es
    reutilizar el borrado lógico que ya existe para la papelera de contactos:
    la interfaz quedaría idéntica, nadie lo notaría, y los datos seguirían
    íntegros en el disco de quien pidió que los borraran.

    Y la segunda mitad, que importa igual: cancelar **deja siempre una baja**.
    Sin ella, esa dirección en un CSV la semana que viene volvería a recibir
    correos — de alguien que pidió desaparecer. Sería el peor fallo posible del
    producto, y no lo detectaría nada: el envío «funcionaría».

    Las pruebas de `arles-db` lo comprueban sobre una base real. Esto vigila el
    código, que es donde se introduciría el atajo.
    """
    nucleo = leer("crates", "arles-core", "src", "cancelacion.rs")
    datos = leer("crates", "arles-db", "src", "cancelacion.rs")

    check(
        3,
        "la cancelación borra la ficha de verdad",
        "DELETE FROM contact" in datos,
        la_cancelacion_borra_de_verdad.__doc__,
        detalle="no aparece un DELETE sobre `contact`: ¿se volvió a un borrado lógico?",
    )
    # AQUÍ NO VA «cancelar deja siempre la baja», Y ES A PROPÓSITO.
    #
    # Se escribió, y el cebo la dejó en evidencia: buscaba el texto
    # `INSERT INTO suppression_entry`, así que comentar esa sentencia —o
    # dejarla dentro de un `if false`— la habría dejado pasando en verde. Una
    # comprobación de texto no distingue «está» de «se ejecuta».
    #
    # Lo comprueba de verdad `cancelar_crea_la_baja_si_no_la_habia` en
    # `arles-db`, sobre una base real, y se verificó rompiéndola: sin el
    # INSERT falla diciendo «cancelar no dejó baja: se le puede volver a
    # escribir». Duplicarla aquí en una versión que no puede fallar sólo
    # añadiría una luz verde que no significa nada.

    # La marca que sustituye a la dirección NO puede derivarse de ella. Una
    # huella de un correo se revierte con un diccionario, y entonces se habría
    # conservado a la persona con otro nombre llamándolo borrado.
    check(
        3,
        "la marca de borrado no se deriva de la dirección",
        "Uuid::new_v4()" in nucleo and "huella(" not in sin_comentarios(nucleo),
        la_cancelacion_borra_de_verdad.__doc__,
        detalle="la marca parece derivarse de la dirección: sería reversible",
    )

    # Y `Cancelacion` no se deserializa: un tipo que afirma que un borrado
    # ocurrió sólo puede salir de quien lo hizo (quinta regla de frontera).
    derives = re.search(r"#\[derive\(([^)]*)\)\]\s*(?:#\[serde[^\]]*\]\s*)*pub struct Cancelacion",
                        nucleo)
    check(
        3,
        "Cancelacion no se puede deserializar",
        bool(derives) and "Deserialize" not in derives.group(1),
        la_cancelacion_borra_de_verdad.__doc__,
        detalle="la webview podría fabricar un resumen de borrado que no ocurrió",
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
    una_supresion_pedida_no_se_levanta_con_un_boton()
    el_informe_de_importacion_consulta_las_supresiones()
    la_vista_previa_no_miente_sobre_las_rutas()
    el_informe_de_acceso_no_deja_cruzar_una_ruta()
    ajustes_es_de_la_aplicacion_y_no_del_negocio()
    la_barra_dice_en_que_seccion_estas()
    la_pantalla_de_acceso_no_pide_rutas_ni_dialogos()
    la_cancelacion_borra_de_verdad()
    el_texto_legal_se_ve_provisional()
    # Entrega 3.5 · el cierre. Una fase sin su informe para Dirección está
    # construida pero no entregada: lo que no se puede explicar sin abrir el
    # código no se ha entregado a nadie.
    informe_para_direccion(3, "FASE-03-PARA-DIRECCION")

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

    nombre = "sonda: la lista de bajas hace lo que dice"
    porque = (
        "Seis afirmaciones de la 3.4 sobre lo que una persona ve y puede "
        "pulsar. La que importa: una baja que pidió la propia persona NO "
        "tiene botón de deshacer, y la pantalla dice por qué en vez de dejar "
        "un hueco. Es la clase de regla que se rompe sin que nada falle — el "
        "botón aparecería, funcionaría, y una baja que alguien pidió se "
        "habría deshecho con un clic. Verificada poniendo el botón en todas "
        "las filas: la sonda nombra las tres que no deberían tenerlo."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:supresion", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: el informe de Acceso hace lo que dice"
    porque = (
        "Diez afirmaciones de la 3.4 sobre el derecho de Acceso, recorriendo "
        "la pantalla en un navegador de verdad. La que más importa: cuando no "
        "consta NADA de una dirección, el informe se entrega igual y la "
        "pantalla **avisa de que va vacío**. Si se callara, quien atiende una "
        "solicitud mandaría un archivo en blanco creyendo que va lleno, y "
        "nada habría fallado por el camino. Se vigilan además dos cosas que "
        "sólo se ven aquí: que ninguna ruta del sistema llegue a la webview "
        "(regla de frontera 4), y que la pantalla SE ENCUENTRE desde la barra "
        "— una pantalla que existe y no se alcanza no está entregada, que es "
        "la lección de las dos observaciones de Dirección del 21 y 22/09."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:derechos", "--silent"],
                  porque, cwd=app, timeout=900)

    nombre = "sonda: auditoría de las diez pantallas a tres anchos"
    porque = (
        "Las demás sondas miran una pantalla y una garantía. Ésta recorre el "
        "producto entero a 1280, 1024 y 900 px —el último pliega la barra, que "
        "es un montaje distinto que nadie medía— y pregunta lo aburrido: si "
        "algo queda recortado o fuera de la ventana, si cada texto se lee, si "
        "cada botón tiene nombre, si hay saltos de encabezado o identificadores "
        "repetidos, si la consola está limpia y si se ve dónde está el teclado "
        "en cada parada del tabulador.\n\n"
        "Nació de la auditoría del 22/09/2026, y encontró en su primer "
        "recorrido algo que nadie veía: un primario deshabilitado conservaba su "
        "contorno de oro porque su selector pesaba más que `.boton:disabled`. "
        "Parecía pulsable y no lo era. Los tokens se verificaban desde la "
        "Fase 2; lo que no se verificaba era LA CASCADA."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:auditoria", "--silent"],
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


# ═══════════════════════════════════════════════════════════════════════════
# FASE 4 · El motor de ejecución
# ═══════════════════════════════════════════════════════════════════════════


def el_motor_no_habla_sql():
    """`arles-motor` no escribe SQL. Es la regla que mantiene `rusqlite` dentro.

    El motor pide a `arles-db` con métodos tipados. En cuanto una consulta se
    escriba aquí, habrá dos sitios donde auditar el SQL del producto — y el
    segundo no lo mirará nadie, porque nadie espera encontrarlo ahí (ADR-0002).

    Se mira el código de produccion, no las pruebas: las pruebas siembran la
    cola con SQL crudo a propósito, por `test-util`.
    """
    sospechosos = []
    d = os.path.join(RAIZ, "crates/arles-motor/src")
    for archivo in sorted(os.listdir(d)):
        if not archivo.endswith(".rs"):
            continue
        texto = leer("crates/arles-motor/src", archivo) or ""
        produccion = texto.split("\n#[cfg(test)]")[0]
        if re.search(r"\b(SELECT|INSERT|UPDATE|DELETE)\s", produccion):
            sospechosos.append(archivo)
    check(4, "el motor no escribe SQL", not sospechosos,
          "ADR-0002: el SQL vive en arles-db, donde se puede auditar entero.",
          f"archivos con SQL: {sospechosos}")


def la_suspension_no_toca_el_presupuesto():
    """La detección de suspensión informa; no reinicia contadores.

    ROTO A PROPÓSITO, y encontrado así: `Despertar::presupuesto` devolvía el
    presupuesto a cero al detectar una suspensión. Con una siesta dentro del
    mismo día eso devolvía el cupo diario entero — cien envíos con un tope de
    cincuenta, la ráfaga que el §53 prohíbe, escrita dentro de su propia
    solución. Se quitó el 23/09/2026 al auditar la fase.

    Esta comprobación impide que vuelva.
    """
    ritmo = leer("crates/arles-core/src", "ritmo.rs") or ""
    check(4, "la suspensión no reinicia el presupuesto",
          "fn presupuesto" not in ritmo,
          "Reiniciar el presupuesto al despertar devuelve el cupo diario "
          "entero si la siesta cae dentro del mismo día: es la ráfaga del §53.")


def el_reloj_atrasado_no_regala_permisos():
    """Una etiqueta de ventana más vieja no reinicia el contador.

    Sin esto basta con atrasar el reloj para que la cuenta mande el cupo diario
    otra vez. Es el caso «reloj del sistema ajustado hacia atrás» de
    ESTRATEGIA_QA §3, que estuvo enunciado y sin cubrir hasta la auditoría.
    """
    p = leer("crates/arles-db/src", "presupuesto.rs") or ""
    check(4, "atrasar el reloj no reinicia el presupuesto",
          "excluded.window_start > window_start" in p
          and "MAX(window_start, excluded.window_start)" in p,
          "Mover el reloj hacia atrás no puede ser una forma de saltarse el "
          "límite diario del proveedor.")


def el_disyuntor_distingue_de_quien_es_la_culpa():
    """Un rebote no abre el disyuntor; un fallo de red sí.

    Es la mitad que rompería el producto: una lista importada con direcciones
    viejas tiene rachas de rebotes, y si contaran, la campaña se pararía sola
    cada pocos minutos sin que el usuario entendiera por qué.
    """
    d = leer("crates/arles-core/src", "disyuntor.rs") or ""
    check(4, "el disyuntor distingue la cuenta del destinatario",
          "habla_de_la_cuenta" in d
          and "Fallo::LaDireccionNoExiste | Fallo::EsaPersonaPidioLaBaja => false" in d,
          "Que una dirección no exista no dice nada malo del remitente.")


def cada_suceso_tiene_un_solo_texto():
    """Un paso del bucle no inventa una copia del texto de otro módulo.

    Antes de la auditoría había una clave `motor.*` por cada paso, y eso creaba
    dos textos para el mismo hecho: Actividad decía «Se volverá a intentar» y el
    detalle del envío, a un clic, «Reintento 2 de 3». El mismo suceso contado de
    dos maneras, y una sin la cifra que exige el §94.
    """
    b = leer("crates/arles-motor/src", "bucle.rs") or ""
    duplicadas = [c for c in
                  ("motor.seVolveraAIntentar", "motor.seAgotaronLosIntentos",
                   "motor.direccionInexistente", "motor.laCuentaNoAutentica",
                   "motor.esaPersonaPidioLaBaja", "motor.sinPresupuesto")
                  if c in b]
    check(4, "cada suceso del motor tiene un solo texto", not duplicadas,
          "Dos textos para el mismo hecho hacen que la misma campaña se lea "
          "distinta en dos pantallas.",
          f"claves duplicadas: {duplicadas}")


def la_parada_de_emergencia_sobrevive_al_cierre():
    """La parada está en la base, no en memoria.

    En memoria, cerrar ARLES la levanta — y quien la pulsó no está delante para
    verlo. El caso: alguien ve que la campaña recién activada lleva la plantilla
    equivocada, pulsa parar y cierra el programa.
    """
    hay_tabla = os.path.exists(os.path.join(
        RAIZ, "crates/arles-db/migrations/V6__parada_de_emergencia.sql"))
    c = leer("crates/arles-db/src", "campanas.rs") or ""
    check(4, "la parada de emergencia se guarda en la base",
          hay_tabla and "engine_control" in c,
          "Una bandera en memoria se levanta sola al arrancar.")


def fase_4(rapido):
    titulo("FASE 4 · El motor de ejecución")

    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("crates/arles-core/src/attempt.rs", "La máquina de estados del envío: qué transición existe y cuál no."),
        ("crates/arles-core/src/ritmo.rs", "El cupo por hora y por día, y la detección de suspensión."),
        ("crates/arles-core/src/ventana.rs", "El horario de la campaña con su zona IANA (D-10)."),
        ("crates/arles-core/src/reintento.rs", "La política de reintentos de D-12: 5, 15 y 30 minutos."),
        ("crates/arles-core/src/disyuntor.rs", "El disyuntor por cuenta (§7): cuándo dejar de insistir."),
        ("crates/arles-core/src/campana.rs", "Pausar, reanudar y detener, y por qué detener no se deshace."),
        ("crates/arles-core/src/duracion.rs", "Cuándo terminará la campaña (§11), antes de activarla."),
        ("crates/arles-core/src/preflight.rs", "Las ocho comprobaciones previas (§12), con «sin comprobar»."),
        ("crates/arles-db/src/cola.rs", "La cola que no duplica: compare-and-swap y recuperación al arrancar."),
        ("crates/arles-db/src/presupuesto.rs", "El cupo persistido, que sobrevive a cerrar el programa."),
        ("crates/arles-db/src/cuentas.rs", "El disyuntor escrito: las columnas de la V1 que nadie tocaba."),
        ("crates/arles-db/src/campanas.rs", "Pausar/detener en una transacción, y la parada de emergencia."),
        ("crates/arles-db/migrations/V6__parada_de_emergencia.sql", "La parada tiene que sobrevivir a cerrar el programa."),
        ("crates/arles-motor/src/proveedor.rs", "La frontera con el mundo: quién entrega un mensaje."),
        ("crates/arles-motor/src/simulador.rs", "Un proveedor al que se le puede ordenar que falle."),
        ("crates/arles-motor/src/bucle.rs", "El bucle del §13, donde todo funciona junto."),
        ("crates/arles-motor/src/instancia.rs", "Un solo ARLES sobre una sola base."),
        ("crates/arles-motor/src/tiempo.rs", "El sello de tiempo, que la base compara como texto."),
    ]:
        check(4, f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    print(f"{GRIS}  invariantes{FIN}")
    migraciones_no_se_editan()
    el_motor_no_habla_sql()
    la_suspension_no_toca_el_presupuesto()
    el_reloj_atrasado_no_regala_permisos()
    el_disyuntor_distingue_de_quien_es_la_culpa()
    cada_suceso_tiene_un_solo_texto()
    la_parada_de_emergencia_sobrevive_al_cierre()
    claves_de_error_sin_texto()

    print(f"{GRIS}  documentación{FIN}")
    informe_para_direccion(4, "FASE-04-PARA-DIRECCION")
    check(4, "existe el documento de la Fase 4",
          os.path.exists(os.path.join(
              RAIZ, "documentacion/09-fases/FASE-04-MOTOR-DE-EJECUCION.md")),
          "Sin documento de fase, lo que se decidió se pierde con la sesión.")


# ── Fase 5 · El correo de verdad ────────────────────────────────────────────


def no_hay_forma_de_no_cifrar():
    """Nada en el código de producción desactiva la verificación de TLS.

    `lettre` trae dos llamadas que la apagan —una para el certificado y otra
    para el nombre del servidor—. Cada una es UNA línea y resuelve al instante
    cualquier problema de certificados, y por eso la regla no es «no las
    usamos»: es que no pueden estar escritas en ninguna parte, ni siquiera tras
    un `cfg(test)`. Una vez que la llamada existe, alguien la encuentra y la
    mueve (§29).

    Se comprueba sobre el árbol entero de Rust, pruebas incluidas, porque el
    sitio donde de verdad tienta escribirla es una prueba.
    """
    # El texto se arma por trozos para que esta comprobación no se encuentre a
    # sí misma al leer su propio archivo, que es exactamente lo que le pasó a
    # la prueba equivalente de `tls.rs` el 23/09/2026.
    prohibidas = [".dangerous_accept_invalid_" + q + "(" for q in ("certs", "hostnames")]
    prohibidas.append("builder_" + "dangerous(")
    sospechosos = []
    for base, _, archivos in os.walk(os.path.join(RAIZ, "crates")):
        if "target" in base:
            continue
        for archivo in archivos:
            if not archivo.endswith(".rs"):
                continue
            ruta = os.path.join(base, archivo)
            with open(ruta, encoding="utf-8") as f:
                texto = f.read()
            if any(p in texto for p in prohibidas):
                sospechosos.append(os.path.relpath(ruta, RAIZ))
    check(5, "nada desactiva la verificación de TLS", not sospechosos,
          "§29: sin verificación, cifrar no sirve de nada — la conexión va "
          "cifrada con quien sea que esté al otro lado.",
          f"archivos: {sorted(sospechosos)}")


def confiar_en_un_certificado_es_solo_de_pruebas():
    """`confiando_ademas_en` no se llama desde el código de producción.

    Es la puerta que permite probar contra el servidor de mentira sin escribir
    una llamada que acepte certificados inválidos. Añade una raíz de confianza,
    no quita comprobaciones — pero una raíz de confianza metida desde la
    interfaz sería una forma de que alguien confiara en un certificado
    cualquiera, que es lo mismo por otro camino.

    ── DOS SITIOS DONDE SÍ VALE, Y POR QUÉ SON DOS ──

    Uno es un módulo `#[cfg(test)]` dentro de un archivo: se corta por ahí.

    El otro es un archivo bajo `tests/`, que **Cargo compila sólo al probar** y
    nunca enlaza en el binario. Hizo falta en la entrega 5.5: la prueba de punta
    a punta del envío vive en `arles-app/tests/`, y esta comprobación la cazó al
    correrla —correctamente, porque un archivo de `tests/` no tiene un
    `#[cfg(test)]` que cortar—.

    Esto **no afloja la comprobación**: la hace más precisa. Un archivo bajo
    `tests/` no puede ser código de producción por construcción, y el resto —
    `src/`— se sigue mirando entero.
    """
    sospechosos = []
    for base, _, archivos in os.walk(os.path.join(RAIZ, "crates")):
        if "target" in base:
            continue
        # Las pruebas de integración de Cargo. No entran en el binario.
        if os.sep + "tests" in base + os.sep:
            continue
        for archivo in archivos:
            if not archivo.endswith(".rs"):
                continue
            ruta = os.path.join(base, archivo)
            with open(ruta, encoding="utf-8") as f:
                texto = f.read()
            # Se corta por el primer módulo de pruebas: ahí sí está permitido.
            produccion = texto.split("\n#[cfg(test)]")[0]
            if ".confiando_ademas_en(" in produccion:
                sospechosos.append(os.path.relpath(ruta, RAIZ))
    check(5, "confiar en un certificado concreto es sólo de pruebas",
          not sospechosos,
          "Una raíz de confianza elegida desde la interfaz sería, por otro "
          "camino, la casilla de «aceptar cualquier certificado».",
          f"archivos: {sorted(sospechosos)}")


def la_contrasena_no_viaja_en_claro():
    """El adaptador comprueba por su cuenta que el canal está cifrado.

    Configurar `lettre` para exigir TLS es necesario y no basta: es confiar en
    que una biblioteca de terceros haga lo que promete, justo donde equivocarse
    cuesta la contraseña de correo de la empresa. Antes de mandar `AUTH`, ARLES
    le pregunta al socket.

    Comprobado rompiéndolo el 23/09/2026: quitando esta línea Y poniendo el TLS
    en «oportunista», la contraseña aparece en lo que el servidor de mentira
    recibió sin cifrar. Con una sola de las dos roturas, no.
    """
    smtp = leer("crates/arles-email/src", "smtp.rs") or ""
    produccion = smtp.split("\n#[cfg(test)]")[0]
    check(5, "el adaptador comprueba el cifrado antes de autenticar",
          "is_encrypted()" in produccion,
          "Es la comprobación que sigue valiendo si mañana cambia un valor por "
          "defecto de `lettre` o alguien toca la configuración.")


def la_base_no_admite_una_conexion_sin_cifrar():
    """El `CHECK` de `smtp_security` sólo deja pasar los dos valores que cifran.

    Es la tercera vez que se escribe la misma regla —está en
    `CifradoDeCorreo`, en `arles_email::tls` y aquí—, y es la única que sigue
    valiendo si un día el código deja pasar el valor: aunque lo dejara, no
    entra en la tabla (§29).
    """
    v7 = leer("crates/arles-db/migrations", "V7__cuenta_smtp.sql") or ""
    # Se mira la línea del CHECK, no el archivo entero: los comentarios de la
    # migración nombran 'none' y 'plain' al explicar por qué NO están.
    linea = ""
    for l in v7.splitlines():
        if "smtp_security IN" in l:
            linea = l
            break
    check(5, "la base no admite una conexión sin cifrar",
          bool(linea) and "'starttls'" in linea and "'directo'" in linea
          and not any(x in linea for x in ("'none'", "'plain'", "'opportunistic'", "'ninguno'")),
          "§29: si el valor cabe en la columna, algún día habrá una fila con él.",
          f"la línea del CHECK: {linea.strip() or 'no se encontró'}")


def solo_el_llavero_toca_contrasenas():
    """Las contraseñas se leen y se escriben **en un solo archivo**.

    `llavero.rs` es el único sitio que llama a `set_password` y `get_password`.
    Con una sola puerta, auditar dónde para una contraseña de correo es abrir un
    archivo; repartida, hay que confiar en que nadie la haya guardado «un
    momento» en otro sitio — y ese momento es el que acaba en una tabla, en una
    preferencia o en una bitácora (ADR-0011).

    Se mira el árbol entero, pruebas incluidas: una prueba que escribe una
    contraseña por su cuenta es exactamente igual de mala, porque el código que
    la rodea se copia.
    """
    permitido = os.path.join("crates", "arles-app", "src", "llavero.rs")
    sospechosos = []
    for base, _, archivos in os.walk(os.path.join(RAIZ, "crates")):
        if "target" in base:
            continue
        for archivo in archivos:
            if not archivo.endswith(".rs"):
                continue
            ruta = os.path.join(base, archivo)
            relativa = os.path.relpath(ruta, RAIZ)
            if relativa == permitido:
                continue
            with open(ruta, encoding="utf-8") as f:
                texto = f.read()
            if ".set_password(" in texto or ".get_password(" in texto:
                sospechosos.append(relativa)
    check(5, "sólo el llavero toca contraseñas", not sospechosos,
          "ADR-0011: con una sola puerta, auditar dónde para una contraseña es "
          "abrir un archivo.",
          f"archivos: {sorted(sospechosos)}")


def produccion_de(relativa):
    """El código de producción de un archivo Rust: sin pruebas y sin comentarios.

    Las dos cosas por el mismo motivo —que una comprobación no se dispare con
    algo que no es el código que vigila—, y las dos ya han dado un falso
    positivo en este proyecto.

    Devuelve `None` si el archivo no existe, para que quien llama decida si eso
    es un fallo o no.
    """
    ruta = os.path.join(RAIZ, relativa)
    if not os.path.exists(ruta):
        return None
    with open(ruta, encoding="utf-8") as f:
        texto = f.read()
    corte = texto.find("#[cfg(test)]")
    if corte != -1:
        texto = texto[:corte]
    return sin_comentarios(texto)


def la_fabrica_no_envia_con_una_contrasena_vacia():
    """La fábrica del proveedor **no continúa** si el llavero no tiene nada.

    El atajo es de una línea —`unwrap_or_default()`— y es el peor error posible
    de ese módulo: el adaptador se construiría, mandaría `AUTH` con la
    contraseña en blanco, el servidor contestaría «credenciales incorrectas» y
    ARLES le diría al usuario que su contraseña de aplicación no sirve. El
    problema sería que ARLES nunca la buscó bien, y varios intentos fallidos
    seguidos marcan la cuenta en Gmail.

    Se comprueba por ausencia y no por presencia a propósito: la prueba
    `sin_contrasena_guardada_no_se_arma_nada` ya cubre el comportamiento
    correcto. Esto caza el atajo el día que alguien lo escriba «para salir del
    paso», que es cuando no se corre la prueba pensando en ella.

    Se mira **sólo el código de producción** —el archivo se corta en
    `#[cfg(test)]`— y sin comentarios. Lo segundo porque esta comprobación se
    caza a sí misma en su propia explicación si no se hace, y ya ha pasado tres
    veces en este proyecto. Lo primero porque las pruebas usan `unwrap_or_else`
    legítimamente para dar un mensaje de fallo.
    """
    codigo = produccion_de("crates/arles-app/src/fabrica.rs")
    if codigo is None:
        check(5, "la fábrica no envía con una contraseña vacía", False,
              "sin el módulo no hay nada que comprobar.", "falta fabrica.rs")
        return

    atajos = [a for a in ("unwrap_or_default", 'unwrap_or("', "unwrap_or_else")
              if a in codigo]
    check(5, "la fábrica no envía con una contraseña vacía", not atajos,
          "una contraseña en blanco gasta un intento de autenticación fallido y "
          "hace que ARLES culpe al usuario de un fallo suyo.",
          f"atajos encontrados: {atajos}")


def solo_una_cuenta_conectada_puede_enviar():
    """La puerta se abre con `connected`; no se cierra ante una lista.

    Escrita al revés —rechazando los estados conocidos— el día que una
    migración añada un quinto estado, una cuenta que el sistema ya había
    marcado como problemática **enviaría correo** hasta que alguien se acordara
    de añadirla a la lista. Y nadie se acuerda.

    Se exige que la comparación con la constante exista tal cual. Es una
    comprobación de forma y no de comportamiento: lo que prueba que funciona es
    `un_estado_que_no_existe_no_envia`, que además está verificada rompiéndola.
    Esto es lo que impide que alguien la reescriba sin correr aquella.
    """
    codigo = produccion_de("crates/arles-app/src/fabrica.rs")
    if codigo is None:
        check(5, "sólo una cuenta conectada puede enviar", False,
              "sin el módulo no hay nada que comprobar.", "falta fabrica.rs")
        return

    bien = 'const PUEDE_ENVIAR: &str = "connected"' in codigo \
        and "cuenta.estado != PUEDE_ENVIAR" in codigo
    check(5, "sólo una cuenta conectada puede enviar", bien,
          "escrita como lista de rechazados, un estado nuevo enviaría correo "
          "hasta que alguien se acordara de añadirlo.",
          "no se encontró la comparación contra PUEDE_ENVIAR")


def el_motor_arranca_con_la_aplicacion():
    """`arles-app` depende del motor **y lo arranca**.

    Hasta la 5.4b el motor estaba construido y probado entero y nadie lo
    arrancaba: `arles-app` ni siquiera dependía de `arles-motor`. Las dos
    mitades existían y no estaban presentadas, y eso no lo cazaba ninguna
    prueba — porque cada mitad pasaba la suya.

    Se miran las dos cosas, porque una sin la otra no dice nada: la dependencia
    puede estar y no usarse, y el arranque no puede estar sin la dependencia.
    """
    manifiesto = os.path.join(RAIZ, "crates", "arles-app", "Cargo.toml")
    depende = False
    if os.path.exists(manifiesto):
        with open(manifiesto, encoding="utf-8") as f:
            depende = "arles-motor" in f.read()

    estado = produccion_de("crates/arles-app/src/estado.rs") or ""
    arranca = "MotorEnMarcha::arrancar" in estado

    check(5, "el motor arranca con la aplicación", depende and arranca,
          "un motor construido que nadie arranca deja ARLES abierto sin enviar, "
          "y ninguna prueba de las dos mitades lo caza.",
          f"depende={depende} arranca={arranca}")


def el_motor_no_corre_en_el_hilo_de_la_ventana():
    """El bucle de envío vive en su propio hilo (§51).

    Cerrar la ventana no detiene una campaña, y un envío que espera sesenta
    segundos a un servidor que no contesta no puede congelar el programa.
    """
    codigo = produccion_de("crates/arles-app/src/motor.rs")
    if codigo is None:
        check(5, "el motor no corre en el hilo de la ventana", False,
              "sin el módulo no hay nada que comprobar.", "falta motor.rs")
        return

    check(5, "el motor no corre en el hilo de la ventana",
          "thread::Builder" in codigo and ".spawn(" in codigo,
          "§51: cerrar la ventana no detiene una campaña, y un envío lento no "
          "puede congelar la interfaz.",
          "no se encontró el hilo propio")


def un_fallo_no_saca_al_motor_del_bucle():
    """El bucle del motor **no propaga errores**: los reintenta.

    Va al revés que el resto del programa, y a propósito. Si este hilo termina,
    ARLES deja de enviar y la ventana sigue abierta como si nada: las campañas
    se quedan en «en marcha», la pantalla no cambia, y el usuario se entera
    horas después.

    Se comprueba por la firma: `correr` no devuelve `Result`. Una firma que lo
    devolviera invitaría a un `?` en el bucle, y un `?` ahí es ARLES mudo hasta
    el siguiente arranque.
    """
    codigo = produccion_de("crates/arles-app/src/motor.rs")
    if codigo is None:
        check(5, "un fallo no saca al motor del bucle", False,
              "sin el módulo no hay nada que comprobar.", "falta motor.rs")
        return

    firma = [l for l in codigo.splitlines() if l.startswith("fn correr(")]
    check(5, "un fallo no saca al motor del bucle",
          bool(firma) and "->" not in firma[0],
          "un `?` en ese bucle deja ARLES sin enviar y sin decirlo, hasta el "
          "siguiente arranque.",
          f"firma: {firma}")


def un_envio_de_prueba_no_da_de_baja_a_nadie():
    """El envío de prueba **no toca la lista de bajas**. Regla 3 de la fase.

    Un `550` en una campaña señala a un contacto y es la puerta a una
    supresión. En un envío de prueba señala a la dirección que el usuario acaba
    de escribir, casi siempre la suya: suprimir ahí **lo daría de baja a él**, y
    una baja no se deshace.

    Se comprueba por ausencia sobre el código de producción de `comandos.rs`:
    la función del envío de prueba no puede nombrar ninguna supresión. Lo que
    prueba el comportamiento es `un_envio_de_prueba_rechazado_no_da_de_baja_a_nadie`,
    que además está verificada rompiéndola; esto caza el día que alguien
    «aproveche» el resultado del servidor para alimentar la lista.
    """
    codigo = produccion_de("crates/arles-app/src/comandos.rs")
    if codigo is None:
        check(5, "un envío de prueba no da de baja a nadie", False,
              "sin el archivo no hay nada que comprobar.", "falta comandos.rs")
        return

    # Sólo el trozo del envío de prueba: el resto del archivo tiene comandos de
    # supresión legítimos, que son otra pantalla y otra cosa.
    inicio = codigo.find("fn enviar_la_prueba(")
    trozo = codigo[inicio:] if inicio != -1 else ""
    sospechas = [t for t in ("suprimir", "supresion", "suppression") if t in trozo]

    check(5, "un envío de prueba no da de baja a nadie",
          inicio != -1 and not sospechas,
          "probar una cuenta mal configurada contra la dirección propia no "
          "puede dar de baja a quien la escribió, y una baja no se deshace.",
          f"encontrado={sospechas} trozo={inicio != -1}")


def la_pantalla_sabe_si_esta_ventana_envia():
    """El estado del motor cruza la frontera **y se enseña**.

    Sin este dato, la pantalla no puede distinguir «la campaña va despacio» de
    «esta ventana no está enviando nada». Las dos se ven igual —un contador que
    no sube— y la segunda no se arregla esperando (§95).

    Se miran los dos extremos, porque uno sin el otro no sirve de nada: el campo
    puede viajar y no pintarse, y la pantalla no puede pintar lo que no recibe.
    Es el mismo fallo que la 5.4 vino a arreglar un piso más abajo —dos mitades
    construidas y sin presentar—, y por eso se vigila igual.
    """
    comandos = produccion_de("crates/arles-app/src/comandos.rs") or ""
    viaja = "el_motor_envia" in comandos

    ruta = os.path.join(RAIZ, "app", "src", "app", "pantallas", "PantallaInicio.vue")
    se_pinta = False
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            se_pinta = "estaVentanaNoEnvia" in sin_comentarios_ni_html(f.read())

    check(5, "la pantalla sabe si esta ventana envía", viaja and se_pinta,
          "una campaña que no avanza sin explicación se lee como una avería.",
          f"viaja={viaja} se_pinta={se_pinta}")


def nada_que_cruce_la_frontera_lleva_contrasena():
    """Ninguna estructura que se serializa hacia la webview tiene contraseña.

    Es la regla 3 de la frontera (`ARQUITECTURA.md` §5) comprobada por encima
    de la disciplina: la contraseña de correo entra —la escribe el usuario— y
    **no sale**. Un campo con ese nombre en algo que se serializa la pondría en
    el JSON que recibe la webview, y de ahí a las herramientas de desarrollo, a
    una captura de pantalla o a un informe de error hay un paso.

    Se miran los `struct` marcados con `Serialize` en la capa de la frontera.

    **Esto busca el NOMBRE del campo, y con eso no basta.** Al romperla a
    propósito se vio que un campo llamado `clave_de_acceso` se le escapa. La que
    caza ése es la prueba `lo_que_cruza_la_frontera_no_lleva_la_contrasena`, que
    serializa una cuenta y busca el valor. Las dos hacen falta: ésta ve al que
    declara el campo aunque no lo rellene todavía; la otra, al que lo llama de
    otra manera.
    """
    texto = leer("crates/arles-app/src", "comandos.rs") or ""
    # Cada bloque «#[derive(... Serialize ...)] ... pub struct X { ... }».
    sospechosos = []
    for bloque in re.finditer(
        r"#\[derive\(([^)]*)\)\][^{]*?pub struct (\w+)\s*\{([^}]*)\}",
        texto,
        re.S,
    ):
        derivados, nombre, cuerpo = bloque.group(1), bloque.group(2), bloque.group(3)
        if "Serialize" not in derivados:
            continue
        if re.search(r"^\s*(pub )?(contrasena|password|secreto)\s*:", cuerpo, re.M):
            sospechosos.append(nombre)
    check(5, "nada que cruce la frontera lleva contraseña", not sospechosos,
          "Regla 3 de la frontera: la contraseña entra y no sale.",
          f"estructuras: {sorted(sospechosos)}")


def la_contrasena_no_se_guarda_en_la_webview():
    """Nada del frontend deja la contraseña en un sitio que sobreviva.

    Ni `localStorage`, ni `sessionStorage`, ni un campo de la tienda: la tienda
    vive lo que vive la aplicación, y un secreto ahí se queda en la memoria de
    la webview mucho después de haberse usado (ADR-0011).

    Vive en un `ref` de la pantalla, que muere al cerrar el formulario.
    """
    def sin_comentarios(texto: str) -> str:
        """Quita las líneas de comentario.

        Hace falta: la primera versión de esta comprobación **se encontraba a sí
        misma**, porque los dos archivos explican en sus comentarios que la
        contraseña NO va a `localStorage`. Es el mismo fallo que ya salió en la
        5.2 con la invariante del TLS, y por eso ahora se mira sólo el código.
        """
        limpias = []
        for l in texto.splitlines():
            despojada = l.strip()
            if despojada.startswith(("*", "//", "/*", "<!--")):
                continue
            limpias.append(l)
        return "\n".join(limpias)

    tienda = sin_comentarios(leer("app/src/app/stores", "remitentes.ts") or "")
    pantalla = sin_comentarios(
        leer("app/src/app/pantallas", "PantallaRemitentes.vue") or ""
    )

    # En la tienda no puede haber NINGÚN sitio donde se guarde.
    en_la_tienda = re.search(r"^\s*(const|let)\s+contrasena\s*=\s*ref", tienda, re.M)
    # Y en ninguno de los dos, almacenamiento del navegador.
    almacenado = any(
        x in tienda or x in pantalla for x in ("localStorage", "sessionStorage")
    )
    check(5, "la contraseña no se queda en la webview",
          not en_la_tienda and not almacenado,
          "ADR-0011: un secreto en la tienda o en el almacenamiento del "
          "navegador sobrevive al formulario que lo pidió.",
          f"en la tienda: {bool(en_la_tienda)} · almacenamiento: {almacenado}")


def fase_5(rapido):
    titulo("FASE 5 · El correo de verdad")

    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("crates/arles-email/src/codigos.rs", "Traducir lo que dice un servidor de correo a los cinco fallos del motor."),
        ("crates/arles-email/src/servidor.rs", "Un servidor SMTP de mentira al que se le puede ordenar que falle."),
        ("crates/arles-email/src/tls.rs", "El cifrado obligatorio, sin ninguna forma de desactivarlo (§29)."),
        ("crates/arles-email/src/smtp.rs", "El adaptador que habla con Gmail."),
        ("crates/arles-email/src/error.rs", "Los problemas de configuración, que no son fallos de envío."),
        ("crates/arles-email/src/prueba.rs", "«Probar conexión»: seis respuestas, cada una con una acción."),
        ("crates/arles-core/src/correo.rs", "La cuenta de correo validada. Y el cifrado, que no tiene un «ninguno»."),
        ("crates/arles-db/migrations/V7__cuenta_smtp.sql", "Las columnas de SMTP que faltaban en `email_account` desde la V1."),
        ("crates/arles-db/migrations/V8__envio_de_prueba.sql", "Dónde consta el envío de prueba de una etapa, y con qué cuenta se hizo."),
        ("crates/arles-db/src/prueba.rs", "Registrar el envío de prueba, y saber si todavía vale."),
        ("crates/arles-app/tests/envio_de_prueba.rs", "ARLES mandando un correo entero, contra un servidor que habla cifrado."),
        ("crates/arles-db/src/correo.rs", "Guardar y leer cuentas remitentes. Sin ninguna contraseña."),
        ("crates/arles-app/src/fabrica.rs", "De una cuenta guardada al adaptador: el único sitio donde una contraseña sale del llavero."),
        ("crates/arles-db/src/trabajo.rs", "Qué etapas están enviando, con su ventana y el más bajo de sus topes."),
        ("crates/arles-app/src/motor.rs", "El hilo que envía, aparte de la ventana (§51)."),
        ("app/src/app/stores/remitentes.ts", "La tienda de remitentes. La contraseña no vive aquí."),
        ("app/src/app/pantallas/PantallaRemitentes.vue", "Donde se da de alta una cuenta y se prueba."),
        ("app/src/app/pantallas/remitentes.spec.ts", "Lo que la pantalla de Remitentes no puede dejar de decir."),
    ]:
        check(5, f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    print(f"{GRIS}  invariantes{FIN}")
    no_hay_forma_de_no_cifrar()
    la_base_no_admite_una_conexion_sin_cifrar()
    confiar_en_un_certificado_es_solo_de_pruebas()
    la_contrasena_no_viaja_en_claro()
    solo_el_llavero_toca_contrasenas()
    la_fabrica_no_envia_con_una_contrasena_vacia()
    solo_una_cuenta_conectada_puede_enviar()
    el_motor_arranca_con_la_aplicacion()
    el_motor_no_corre_en_el_hilo_de_la_ventana()
    un_fallo_no_saca_al_motor_del_bucle()
    la_pantalla_sabe_si_esta_ventana_envia()
    un_envio_de_prueba_no_da_de_baja_a_nadie()
    nada_que_cruce_la_frontera_lleva_contrasena()
    la_contrasena_no_se_guarda_en_la_webview()
    claves_de_error_sin_texto()


# ── Fase 6 · Campañas y mensajes ────────────────────────────────────────────


def el_motor_de_plantillas_no_evalua_nada():
    """El núcleo no depende de ninguna biblioteca de plantillas (ADR-0006).

    Handlebars, Tera, MiniJinja y compañía **evalúan expresiones**, y aquí las
    plantillas llevan datos de contacto importados de un XLSX que no escribimos
    nosotros. Un motor con lógica sobre datos ajenos es una vía de ejecución.

    Se comprueba por ausencia en el `Cargo.toml` porque el día que alguien
    añada una de estas «para poder poner un `{{#if}}`», el código seguirá
    compilando y todas las pruebas seguirán pasando: lo que cambia no falla,
    se abre.
    """
    ruta = os.path.join(RAIZ, "crates", "arles-core", "Cargo.toml")
    texto = ""
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            texto = sin_comentarios(f.read())

    motores = [m for m in ("handlebars", "tera", "minijinja", "liquid", "askama")
               if re.search(rf"^\s*{m}\s*=", texto, re.M)]
    check(6, "el motor de plantillas no evalúa nada", not motores,
          "ADR-0006: sustituir es seguro; evaluar sobre datos importados, no.",
          f"encontrado: {motores}")


def el_html_se_sanea_en_rust():
    """El saneado vive en Rust, con lista blanca, y no en la webview.

    Un saneado en el frontend se salta escribiendo en la base por cualquier
    otro camino, y además **no protege al destinatario**: el correo sale del
    proceso de Rust, no del navegador. Por eso `sanear.rs` existe y por eso se
    aplica dos veces —al guardar y al enviar—.

    Se miran las dos mitades: que el módulo esté y use `ammonia`, y que el
    frontend no traiga su propio saneador, que es como se empieza a confiar en
    el del navegador.
    """
    codigo = produccion_de("crates/arles-core/src/sanear.rs") or ""
    con_lista_blanca = "ammonia" in codigo and "ETIQUETAS" in codigo

    del_navegador = []
    for base, _, archivos in os.walk(os.path.join(RAIZ, "app", "src")):
        for nombre in archivos:
            if not nombre.endswith((".ts", ".vue")):
                continue
            with open(os.path.join(base, nombre), encoding="utf-8") as f:
                if "DOMPurify" in f.read():
                    del_navegador.append(nombre)

    check(6, "el HTML se sanea en Rust y con lista blanca",
          con_lista_blanca and not del_navegador,
          "el correo sale de Rust: un saneado en la webview no protege a quien "
          "lo recibe.",
          f"lista blanca={con_lista_blanca} · en la webview: {del_navegador}")


def una_version_de_plantilla_no_se_edita():
    """Ninguna consulta actualiza `template_version`.

    Una versión es lo que se envió. Editarla cambia, a posteriori, el texto de
    los correos que ya salieron: el registro diría una cosa y los destinatarios
    habrían leído otra, **y nada fallaría**. Por eso se crean versiones nuevas
    y no se toca ninguna.

    Se busca en todo `arles-db` y no sólo en `plantillas.rs`: el atajo lo
    escribe quien añade una pantalla de edición, y no necesariamente ahí.
    """
    culpables = []
    for base, _, archivos in os.walk(os.path.join(RAIZ, "crates", "arles-db", "src")):
        for nombre in archivos:
            if not nombre.endswith(".rs"):
                continue
            relativa = os.path.relpath(os.path.join(base, nombre), RAIZ)
            codigo = produccion_de(relativa) or ""
            if re.search(r"UPDATE\s+template_version", codigo, re.I):
                culpables.append(os.path.basename(relativa))

    check(6, "una versión de plantilla no se edita", not culpables,
          "una versión es lo que se envió: cambiarla reescribe el pasado.",
          f"archivos: {sorted(culpables)}")


def el_preflight_no_se_rellena_a_mano():
    """Los tres puntos que exigen salir a la red no se afirman a mano.

    `cuenta_autentica`, `hay_conexion` y `registros_del_dominio` sólo se pueden
    contestar hablando con alguien de fuera. Ponerles `Some(true)` para que el
    preflight pase es el atajo que deja al usuario activando una campaña sobre
    una comprobación que nadie hizo — y no falla nada: el botón se enciende.

    Se mira el código de producción de la frontera, que es donde se construyen
    las observaciones.
    """
    codigo = produccion_de("crates/arles-app/src/comandos.rs") or ""
    inventados = [
        campo
        for campo in ("cuenta_autentica", "hay_conexion", "registros_del_dominio")
        if re.search(rf"{campo}\s*:\s*Some\(true\)", codigo)
    ]
    check(6, "el preflight no se rellena a mano", not inventados,
          "§12: lo que no se ha mirado se dice, no se da por bueno.",
          f"afirmados sin mirar: {inventados}")


def la_revision_descuenta_las_bajas():
    """La audiencia del preflight aplica la lista de supresión.

    Es el punto entero: una campaña puede tener tres mil contactos y cero
    destinatarios. Contar el total diría «queda alguien a quien escribir» y la
    activación —que congela la audiencia y no se deshace— seguiría adelante.

    Se comprueba que la consulta nombra `suppression_entry`, porque el error
    aquí no es escribir algo malo sino **quitar** una línea: un `NOT EXISTS`
    que desaparece no rompe la consulta, sólo cambia la cifra.
    """
    codigo = produccion_de("crates/arles-db/src/revision.rs") or ""
    check(6, "la revisión descuenta las bajas",
          "suppression_entry" in codigo and "NOT EXISTS" in codigo,
          "§63: la audiencia es la que queda después de aplicar las bajas.",
          "la consulta de la audiencia no mira la lista de supresión")


def el_envio_de_prueba_llega_al_preflight():
    """La marca que la 5.5 anota la lee el preflight, y con la cuenta correcta.

    Estuvo tres entregas anotándose sin que nadie la preguntara. Un preflight
    que no la lee diría «no se hizo el envío de prueba» con el correo ya
    mandado y recibido, y quien lo lea volverá a mandarlo pensando que falló.

    Se miran los dos extremos, como en la 5.4: que la base la resuelva
    comparando **la cuenta**, y que la frontera la pase a las observaciones.
    """
    revision = produccion_de("crates/arles-db/src/revision.rs") or ""
    comandos = produccion_de("crates/arles-app/src/comandos.rs") or ""

    se_resuelve = "se_hizo_el_envio_de_prueba" in revision and "test_sent_with_account_id" in revision
    se_pasa = re.search(
        r"se_hizo_el_envio_de_prueba:\s*Some\(sabido\.se_hizo_el_envio_de_prueba\)",
        comandos,
    )

    check(6, "el envío de prueba llega al preflight",
          se_resuelve and bool(se_pasa),
          "§45: una comprobación que se hace y nadie lee es una que no se hizo.",
          f"se_resuelve={se_resuelve} se_pasa={bool(se_pasa)}")


def el_aviso_de_volumen_lo_exige_el_nucleo():
    """La casilla del §48 la enseña la pantalla, pero la exige Rust.

    ─────────────────────────────────────────────────────────────────────────
    POR QUÉ NO BASTA CON QUE LA PANTALLA LA PIDA

    Una casilla obligatoria en el asistente se salta por cualquier otro camino
    que llegue a `crear_campana` —otra pantalla, una prueba, el día que haya
    importación de campañas—, y la campaña de alto volumen se crea **sin que
    nada falle**: los correos salen, y el día que el proveedor marque la cuenta
    como spam no hay forma de saber si alguien avisó.

    Así que se comprueban los dos extremos: que el núcleo rechace sin aceptación
    y que la pantalla enseñe el aviso antes de pedirla. El segundo sin el
    primero es cortesía; el primero sin el segundo es un error sin explicación.
    """
    db = produccion_de("crates/arles-db/src/alta_de_campana.rs") or ""
    lo_exige = "FaltaAceptarElVolumen" in db and "AVISO_DIARIO" in db
    # Y deja rastro: sin la línea de bitácora no hay constancia de que se avisó.
    deja_rastro = "aceptar_riesgo_de_volumen" in db

    ruta = os.path.join(RAIZ, "app", "src", "app", "pantallas", "AsistenteDeCampana.vue")
    se_enseña = False
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            se_enseña = "avisaDelVolumen" in sin_comentarios_ni_html(f.read())

    check(6, "el aviso de volumen lo exige el núcleo",
          lo_exige and deja_rastro and se_enseña,
          "§48: ARLES no bloquea el volumen, pero no deja crear la campaña sin "
          "que conste que se avisó.",
          f"lo_exige={lo_exige} deja_rastro={deja_rastro} se_enseña={se_enseña}")


def la_pantalla_no_dice_entregado():
    """Ningún texto de campañas usa la palabra «entregado» (§65).

    ARLES sabe que el proveedor **aceptó** el mensaje. No sabe si llegó a una
    bandeja, ni si acabó en no deseados. La diferencia parece pequeña hasta que
    alguien decide algo con ella delante.

    ─────────────────────────────────────────────────────────────────────────
    SE MIRAN LOS TEXTOS, NO LOS ARCHIVOS DE PANTALLA

    La primera versión de esta comprobación leía los `.vue`, y falló **en su
    primera ejecución** por una razón que conviene dejar escrita: cazó la clave
    `campanas.aceptadoNoEsEntregado`, que existe justamente para explicar que no
    se dice «entregado». La comprobación estaba cazando la solución.

    Mirar los `.vue` además no serviría: los componentes no llevan literales
    (§17), así que la palabra nunca estaría ahí — la comprobación pasaría
    siempre y no verificaría nada.

    Lo que se mira son los **valores de texto** bajo `campanas:` y `asistente:`
    en el archivo de idioma, que es donde de verdad puede aparecer.
    """
    ruta = os.path.join(RAIZ, "app", "src", "app", "locales", "es.ts")
    if not os.path.exists(ruta):
        check(6, "la pantalla no dice «entregado»", False,
              "§65: aceptado por el proveedor no es entregado a una persona.",
              "falta el archivo de idioma")
        return

    with open(ruta, encoding="utf-8") as f:
        texto = sin_comentarios(f.read())

    # Las dos ramas de esta fase, desde su nombre hasta el cierre de su bloque.
    culpables = []
    for rama in ("campanas", "asistente"):
        m = re.search(rf"^  {rama}: {{$", texto, re.M)
        if not m:
            continue
        fin = texto.find("\n  },", m.end())
        bloque = texto[m.end():fin if fin != -1 else len(texto)]
        # Sólo el contenido de las cadenas: los nombres de clave no se leen.
        for cadena in re.findall(r"'([^']*)'", bloque):
            if "entregad" in cadena.lower():
                culpables.append(f"{rama}: …{cadena[:40]}…")

    check(6, "la pantalla no dice «entregado»", not culpables,
          "§65: aceptado por el proveedor no es entregado a una persona.",
          f"textos: {culpables}")


def el_boton_de_activar_lo_apaga_el_preflight():
    """Activar está atado al veredicto del núcleo, no a una regla de pantalla.

    Activar congela la audiencia y empieza a mandar correos: no se deshace. Si
    la pantalla decidiera por su cuenta cuándo se puede pulsar, habría dos
    copias de la regla, y el día que divergieran dejaría activar algo que el
    preflight había bloqueado.

    Se comprueba que el botón mira `sePuedeActivar`, y que ese valor sale del
    `resultado` que viene de Rust y no de una cuenta de puntos hecha aquí.
    """
    ruta = os.path.join(RAIZ, "app", "src", "app", "pantallas", "AsistenteDeCampana.vue")
    codigo = ""
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            codigo = sin_comentarios_ni_html(f.read())

    atado = ':deshabilitado="!sePuedeActivar"' in codigo
    del_nucleo = "preflight.resultado.resultado === 'lista'" in codigo

    check(6, "el botón de activar lo apaga el preflight", atado and del_nucleo,
          "§63: quien decide si se puede activar es el núcleo, y esto no se "
          "deshace.",
          f"atado={atado} del_nucleo={del_nucleo}")


def fase_6(rapido):
    titulo("FASE 6 · Campañas y mensajes")

    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("crates/arles-core/src/plantilla.rs", "Sustituir `{{variable}}` y nada más (ADR-0006)."),
        ("crates/arles-core/src/sanear.rs", "El HTML con lista blanca, en Rust y dos veces."),
        ("crates/arles-db/src/plantillas.rs", "Plantillas con versiones que no se editan, y firmas."),
        ("crates/arles-db/src/alta_de_campana.rs", "Crear campañas, y congelar la audiencia al activar."),
        ("crates/arles-db/src/revision.rs", "Lo que la base sabe de una campaña, para el preflight."),
        ("crates/arles-app/tests/preflight.rs", "Los ocho puntos, rellenados desde una base de verdad."),
        ("crates/arles-db/src/ventanas.rs", "Los horarios de envío, validados al guardarlos."),
        ("crates/arles-app/src/campanas.rs", "Los comandos de campañas, plantillas, firmas y horarios."),
        ("app/src/app/pantallas/PantallaCampanas.vue", "La lista de campañas. Aceptados, nunca entregados (§65)."),
        ("app/src/app/pantallas/AsistenteDeCampana.vue", "El asistente de nueve pasos."),
        ("app/src/app/pantallas/campanas.spec.ts", "Lo que estas dos pantallas no pueden dejar de decir."),
    ]:
        check(6, f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    print(f"{GRIS}  invariantes{FIN}")
    el_motor_de_plantillas_no_evalua_nada()
    el_html_se_sanea_en_rust()
    una_version_de_plantilla_no_se_edita()
    el_preflight_no_se_rellena_a_mano()
    la_revision_descuenta_las_bajas()
    el_envio_de_prueba_llega_al_preflight()
    el_aviso_de_volumen_lo_exige_el_nucleo()
    la_pantalla_no_dice_entregado()
    el_boton_de_activar_lo_apaga_el_preflight()


# ─── Fase W · El canal de WhatsApp ───────────────────────────────────────────


def lo_que_se_construye_se_usa():
    """**La comprobación que faltaba, y que habría cazado dos huecos.**

    Dos veces en dos fases se construyó una pieza correcta, probada, y **nadie
    la llamó**:

    - Los comandos `guardar_plantilla` y `guardar_firma` de la Fase 6: existen,
      tienen pruebas, y ninguna pantalla los invoca — así que desde la
      aplicación no se puede crear una plantilla.
    - `ProveedorDeWhatsApp` de la W.2: habla con Cloud API, tiene su prueba de
      punta a punta contra el servidor de mentira, y **nadie lo construye** —
      así que ARLES no puede mandar un WhatsApp.

    Ninguna prueba podía verlo. Una prueba comprueba que la pieza funciona; no
    que otro módulo la use. El hueco vive justo entre dos módulos que cada uno
    pasa sus pruebas.

    Aquí se comprueba lo que ninguna de las dos capas puede comprobar sola: que
    cada función del store de Vue que invoca un comando **se llama desde alguna
    pantalla**, y que cada proveedor **se construye fuera de las pruebas**.
    """
    sueltas = []

    # 1 · Las funciones de los stores que nadie usa.
    dir_stores = os.path.join(RAIZ, "app/src/app/stores")
    pantallas = ""
    for base, _, ficheros in os.walk(os.path.join(RAIZ, "app/src")):
        for f in ficheros:
            if f.endswith(".vue"):
                with open(os.path.join(base, f), encoding="utf-8") as fh:
                    pantallas += fh.read()

    if os.path.isdir(dir_stores):
        for f in sorted(os.listdir(dir_stores)):
            if not f.endswith(".ts") or f.endswith(".spec.ts"):
                continue
            texto = leer("app/src/app/stores", f) or ""
            # Las funciones que cruzan la frontera: las que llaman a `invocar`.
            for m in re.finditer(
                r"async function (\w+)\([^)]*\)[^{]*\{(?:[^{}]|\{[^{}]*\})*?invocar",
                texto,
                re.S,
            ):
                nombre = m.group(1)
                # Una función que solo usan otras del mismo store no es un hueco.
                usos_internos = len(re.findall(rf"\b{nombre}\b", texto))
                if nombre not in pantallas and usos_internos <= 2:
                    sueltas.append(f"{f}: {nombre}() no se llama desde ninguna pantalla")

    # 2 · Los proveedores que nadie construye fuera de las pruebas.
    for crate, tipo in [("arles-email", "ProveedorSmtp"), ("arles-whatsapp", "ProveedorDeWhatsApp")]:
        if not os.path.isdir(os.path.join(RAIZ, "crates", crate)):
            continue
        usado = False
        for base, _, ficheros in os.walk(os.path.join(RAIZ, "crates/arles-app/src")):
            for f in ficheros:
                if not f.endswith(".rs"):
                    continue
                with open(os.path.join(base, f), encoding="utf-8") as fh:
                    cuerpo = fh.read()
                # Solo cuenta fuera de `mod tests`: construirlo en una prueba no
                # es usarlo, y es exactamente el espejismo que se busca romper.
                cuerpo = cuerpo.split("#[cfg(test)]")[0]
                if f"{tipo}::nuevo" in cuerpo:
                    usado = True
        if not usado:
            sueltas.append(f"{tipo} no se construye en arles-app fuera de las pruebas")

    check(
        "W",
        "todo lo que se construye lo usa alguien",
        not sueltas,
        "Una pieza probada que nadie llama no hace nada, y ninguna prueba de esa "
        "pieza puede verlo. Pasó dos veces: el editor de plantillas de la Fase 6 "
        "y el adaptador de WhatsApp de la W.2.",
        " · ".join(sueltas),
    )


def el_token_de_meta_no_cruza_la_frontera():
    """Regla 3 de la frontera IPC, aplicada al canal nuevo."""
    texto = leer("crates/arles-app/src/whatsapp.rs") or ""
    # Se mira DENTRO del struct que cruza, no en todo el archivo: mencionar el
    # campo en un comentario que explica por qué no se copia es lo contrario de
    # un fallo, y la primera versión de esta comprobación lo marcaba como tal.
    m = re.search(r"pub struct NumeroIpc \{(.*?)\n\}", texto, re.S)
    campos = m.group(1) if m else ""
    check(
        "W",
        "lo que sale hacia la webview no lleva el token",
        bool(m) and "referencia_de_credencial" not in campos and "token" not in campos,
        "El token de Meta es con lo que se envía en nombre de la empresa. En la "
        "base va una referencia opaca; en la pantalla, ni eso.",
    )


def lo_que_dice_la_gente_no_se_guarda():
    """La decisión de Dirección hecha tipo: se cuenta quién, no qué dijo."""
    texto = leer("crates/arles-whatsapp/src/entrante.rs") or ""
    cuerpo = texto.split("#[cfg(test)]")[0]
    tiene_respuesta = "pub struct Respuesta" in cuerpo
    sin_texto = not re.search(r"pub (texto|mensaje|cuerpo|body)\s*:", cuerpo)
    check(
        "W",
        "una respuesta no lleva lo que dijo la persona",
        tiene_respuesta and sin_texto,
        "El texto de un mensaje entrante muere en la función que lo lee. Si "
        "viajara hasta la base y allí se ignorara, habría pasado por una capa "
        "más — y por cualquier traza de error que haya en medio.",
    )


def la_firma_se_comprueba_antes_de_analizar():
    """El orden no es estético: analizar primero es procesar lo que mande
    cualquiera que descubra la dirección, y preguntarse después."""
    texto = leer("crates/arles-whatsapp/src/entrante.rs") or ""
    m = re.search(r"pub fn leer\((?:[^)]*)\)[^{]*\{(.*?)\n\}", texto, re.S)
    cuerpo = m.group(1) if m else ""
    pos_firma = cuerpo.find("la_firma_cuadra")
    pos_analisis = cuerpo.find("serde_json")
    check(
        "W",
        "la firma se comprueba antes de mirar el cuerpo",
        pos_firma >= 0 and (pos_analisis < 0 or pos_firma < pos_analisis),
        "Un aviso falso podría decir que un número contestó —lo que falsea la "
        "métrica— o que la calidad se hundió, lo que pararía el canal.",
    )


def una_calidad_sin_consultar_no_es_buena():
    """Un semáforo que se pone en verde cuando no ha podido mirar es peor que
    no tener semáforo."""
    texto = leer("app/src/app/pantallas/PantallaWhatsApp.vue") or ""
    check(
        "W",
        "una calidad sin consultar no se pinta como alta",
        "sinConsultar" in texto and "calidad === null" in texto,
        "`null` significa «no se sabe», no «está bien». Es la misma regla del "
        "«sin comprobar» del preflight de correo.",
    )


def las_dos_marcas_no_estan_en_el_alta():
    """Como casillas del formulario se marcarían con el resto de los campos sin
    leerlas — que es justo lo que existen para impedir."""
    texto = leer("crates/arles-app/src/whatsapp.rs") or ""
    cuerpo = texto.split("#[cfg(test)]")[0]
    m = re.search(r"pub fn guardar_numero_de_whatsapp\(.*?\n\}", cuerpo, re.S)
    alta = m.group(0) if m else ""
    check(
        "W",
        "el alta no pone las dos confirmaciones",
        "confirmar_coexistencia" not in alta and "aceptar_el_riesgo" not in alta,
        "Son la constancia de que a alguien se le avisó. Puestas en el alta se "
        "marcarían sin leerlas, y no constaría nada.",
    )


def ninguna_tabla_se_queda_sin_escritor():
    """**Una tabla que nadie rellena es una función que nadie llama, en datos.**

    `whatsapp_template` estuvo en el esquema desde la V9 sin que ningún
    repositorio escribiera en ella. Sólo se leía, para contar cuántas plantillas
    aprobadas hay — y como la cuenta daba siempre cero, y la puerta del canal
    exige al menos una, **ninguna campaña de WhatsApp se podía activar nunca**.

    Ninguna prueba podía verlo: las de la migración comprueban que la tabla
    existe con sus restricciones, y pasan; las de la revisión siembran filas a
    mano y pasan. El hueco vive entre el esquema y el repositorio.

    Es el cuarto caso del mismo patrón —algo correcto que nadie usa— y el
    primero en forma de tabla. `lo_que_se_construye_se_usa()` no lo caza porque
    no es código sin llamar.

    Se mira el `INSERT` fuera de las pruebas: sembrar en un `mod tests` es
    justamente el espejismo que hizo que esto pasara desapercibido.
    """
    import glob

    huerfanas = []

    tablas = set()
    for sql in glob.glob(os.path.join(RAIZ, "crates/arles-db/migrations", "*.sql")):
        with open(sql, encoding="utf-8") as f:
            for m in re.finditer(r"CREATE TABLE (?:IF NOT EXISTS )?(\w+)", f.read()):
                tablas.add(m.group(1))

    # Las que nadie rellena porque nadie debe rellenarlas a mano.
    DE_SISTEMA = {"refinery_schema_history", "schema_version", "sqlite_sequence"}

    # ── Las que SÍ están sin escritor, y por qué ──
    #
    # Una excepción con su motivo escrito no es un silencio: si mañana aparece
    # una tabla nueva sin quien la rellene, esta comprobación falla igual. Lo
    # que no puede es fallar para siempre por trabajo que se decidió no hacer,
    # porque entonces se apaga y deja de mirar.
    #
    # Cada una se quita de aquí el día que se construya lo suyo.
    PENDIENTES_CONOCIDOS = {
        # Etiquetas, listas y campos propios: §40, declarado fuera del alcance
        # de la Fase 6 en su documento de cierre. Enseñar un filtro que no
        # filtra sería peor que no tenerlo.
        "tag",
        "contact_tag",
        "contact_list",
        "contact_list_member",
        "contact_field",
        # El registro de consentimiento (P-17): la tabla existe y se LEE para
        # decidir si alguien dio permiso, pero todavía no hay pantalla que
        # registre uno. Hoy el permiso sólo entra por importación.
        "consent_entry",
        # Los eventos de calidad de Meta llegan por webhook, y el «relay ciego»
        # que Dirección decidió el 29/09/2026 no existe todavía.
        "whatsapp_quality_event",
    }

    cuerpo = ""
    for base, _, ficheros in os.walk(os.path.join(RAIZ, "crates/arles-db/src")):
        for f in ficheros:
            if f.endswith(".rs"):
                with open(os.path.join(base, f), encoding="utf-8") as fh:
                    # Sólo lo que hay ANTES de `mod tests`: una siembra de
                    # prueba no es un escritor.
                    cuerpo += fh.read().split("#[cfg(test)]")[0]

    for tabla in sorted(tablas - DE_SISTEMA - PENDIENTES_CONOCIDOS):
        # Las `*_nueva` son el paso intermedio de una migración que reconstruye
        # una tabla: se crean, se copian y se renombran dentro del mismo `.sql`.
        # No existen cuando el programa arranca.
        if tabla.endswith("_nueva"):
            continue
        if not re.search(rf"INSERT (?:OR \w+ )?INTO {tabla}\b", cuerpo):
            huerfanas.append(tabla)

    check(
        "W",
        "ninguna tabla del esquema se queda sin quien la rellene",
        not huerfanas,
        "Una tabla que nadie rellena hace que todo lo que dependa de ella "
        "quede en cero para siempre. `whatsapp_template` tuvo a la puerta del "
        "canal cerrada con llave desde dentro, y ninguna prueba podía verlo.",
        " · ".join(huerfanas),
    )


def toda_pantalla_se_puede_abrir():
    """**Una pantalla sin ruta es una pantalla que no existe.**

    Pasó con `PantallaPlantillas.vue` el 29/09/2026: trece kilobytes de
    pantalla, con sus seis pruebas en verde, y **ninguna ruta la abría**. El
    usuario no tenía forma de llegar a ella.

    Las pruebas no podían verlo porque montan el componente directamente —que
    es lo correcto para probar lo que enseña— y nunca navegan. El hueco vive
    entre el componente y el enrutador.

    Es la misma familia que `lo_que_se_construye_se_usa`, en la capa de arriba:
    allí una función que nadie llama, aquí una pantalla a la que nadie va.
    """
    sueltas = []
    rutas = leer("app/src/app/router.ts") or ""

    dir_pantallas = os.path.join(RAIZ, "app/src/app/pantallas")
    if os.path.isdir(dir_pantallas):
        for f in sorted(os.listdir(dir_pantallas)):
            if not f.endswith(".vue"):
                continue
            # Un formulario o un diálogo no tiene ruta propia: se abre desde
            # otra pantalla. Se distinguen porque alguien los importa.
            usado_por_otra = False
            for base, _, ficheros in os.walk(os.path.join(RAIZ, "app/src")):
                for otro in ficheros:
                    if not otro.endswith(".vue") or otro == f:
                        continue
                    with open(os.path.join(base, otro), encoding="utf-8") as fh:
                        if f[:-4] in fh.read():
                            usado_por_otra = True
            if f not in rutas and not usado_por_otra:
                sueltas.append(f)

    check(
        "W",
        "toda pantalla escrita se puede abrir",
        not sueltas,
        "Una pantalla sin ruta no la ve nadie, y sus pruebas pasan igual "
        "porque montan el componente sin navegar. Pasó con la de plantillas.",
        " · ".join(sueltas),
    )


def ningun_comando_se_queda_sin_quien_lo_llame():
    """**El patrón del §9, por fin visto desde el único sitio que lo ve entero.**

    Había tres comprobaciones persiguiendo este patrón, y cada una miraba un
    tramo del cable:

    - `lo_que_se_construye_se_usa` · función de store → pantalla.
    - `toda_capacidad_del_nucleo_tiene_puerta` · núcleo → comando.
    - `el_freno_llega_hasta_la_pantalla` · los comandos **de `control.rs`** →
      pantalla.

    Falta el tramo de en medio, y es el único que el usuario pisa:
    **comando registrado → alguien que lo invoque.** La tercera lo cubre para
    seis comandos de un archivo; los otros cuarenta y nueve no los miraba nadie.

    ───────────────────────────────────────────────────────────────────────────
    LO QUE ENCONTRÓ AL ESCRIBIRLA, EL 01/10/2026, CON LAS NUEVE FASES EN VERDE

    Tres comandos registrados en `generate_handler!` que **ninguna pantalla y
    ningún almacén invocan**:

    1. `enviar_whatsapp_de_prueba` — el envío de prueba de la W.8. Construido,
       probado de punta a punta contra el servidor de mentira, y **sin botón**.
       El `CLAUDE.md` decía «el envío de prueba que manda un WhatsApp real»:
       cierto del adaptador, falso del producto. Desde ARLES no se podía mandar.
    2. `revisar_campana_de_whatsapp` — el preflight del canal de la W.5, con sus
       dos cifras de permiso. Mismo caso.
    3. `contacto` — la ficha completa de un contacto. Éste no es un botón que
       falte: **es un comando que no hace falta**, porque la lista ya trae todos
       los canales y la ficha se recalcula de ella. Se quitó.

    Los tres sobrevivieron al cierre de la W.7, al de la 6.8 y a la auditoría de
    recorrido A1–A3, que inventarió **pantallas y acciones** — y un comando sin
    botón no aparece en un inventario de botones.

    Y hay una razón que no es de pulcritud: **cada comando registrado es
    superficie de ataque**. Uno que nadie llama es una puerta que nadie vigila,
    y la regla 3 de la frontera —ningún comando devuelve un secreto— hay que
    sostenerla en todas, no sólo en las que se usan.

    La lista se lee de `generate_handler!`, no a mano, por la misma razón que la
    del freno: la lista a mano fue el defecto.
    """
    import re as _re

    lib = leer("crates", "arles-app", "src", "lib.rs") or ""
    bloque = _re.search(r"generate_handler!\[(.*?)\]", lib, _re.S)
    assert bloque, "no se encontró generate_handler![…]: el patrón dejó de casar"
    # Sin comentarios: el bloque los lleva, y un comando nombrado en uno no
    # está registrado.
    crudo = _re.sub(r"//[^\n]*", "", bloque.group(1))
    registrados = sorted(set(_re.findall(r"(?:\w+::)?(\w+)\s*,", crudo)))
    assert len(registrados) > 40, f"se leyeron sólo {len(registrados)} comandos"

    # ── TODO `app/src`, Y NO UNA LISTA DE CARPETAS ─────────────────────────
    #
    # La primera versión leía `pantallas/` y `stores/`, más `nucleo.ts` y
    # `app.ts` a mano. Dio **un falso positivo inmediato**:
    # `abrir_sitio_de_telemetry` se invoca desde `App.vue`, que no está en
    # ninguna de las dos carpetas ni en la lista de sueltos.
    #
    # Es el mismo defecto que esta comprobación persigue, cometido al
    # escribirla: una lista a mano de dónde mirar. Ahora se recorre el árbol.
    fuentes = []
    for base, _, ficheros in os.walk(os.path.join(RAIZ, "app", "src")):
        for archivo in sorted(ficheros):
            if not archivo.endswith((".vue", ".ts")) or archivo.endswith(".spec.ts"):
                continue
            with open(os.path.join(base, archivo), encoding="utf-8") as fh:
                fuentes.append(fh.read())

    # Sin comentarios: hablar de un comando no es llamarlo. Es la lección de
    # `el_freno_llega_hasta_la_pantalla`, que llegó a satisfacerse con el
    # comentario que explicaba el defecto que buscaba.
    unido = _re.sub(r"/\*.*?\*/|//[^\n]*|<!--.*?-->", "", "".join(fuentes), flags=_re.S)

    huerfanos = [c for c in registrados if f"'{c}'" not in unido and f'"{c}"' not in unido]
    check(
        "W",
        "ningún comando se queda sin quien lo llame",
        not huerfanos,
        ningun_comando_se_queda_sin_quien_lo_llame.__doc__,
        detalle=f"registrados y sin invocar: {huerfanos}",
    )


def fase_w(rapido):
    titulo("CANAL W · WhatsApp")

    print(f"{GRIS}  estructura{FIN}")
    for ruta, porque in [
        ("crates/arles-core/src/whatsapp.rs", "Los dos modos del canal (L-9) y la plantilla aprobada."),
        ("crates/arles-whatsapp/src/cloud.rs", "El adaptador que habla con Cloud API."),
        ("crates/arles-whatsapp/src/servidor.rs", "El servidor de mentira que imita a Meta."),
        ("crates/arles-whatsapp/src/entrante.rs", "Los avisos: firma verificada, texto descartado."),
        ("crates/arles-db/src/numero.rs", "El número guardado, con sus dos marcas."),
        ("crates/arles-db/src/revision_whatsapp.rs", "La puerta del canal y las dos cifras de permiso."),
        ("crates/arles-app/src/whatsapp.rs", "Los comandos del número."),
        ("app/src/app/pantallas/PantallaWhatsApp.vue", "La pantalla del número."),
        ("app/src/app/pantallas/whatsapp.spec.ts", "Lo que esa pantalla no puede dejar de decir."),
        ("documentacion/09-fases/FASE-W-CANAL-WHATSAPP.md", "Qué se construyó, y qué no."),
        ("documentacion/07-entrega/GUIA-DE-CONEXION-REAL.md", "Cómo conectar a Meta y a Gmail de verdad."),
    ]:
        check("W", f"existe {ruta}", os.path.exists(os.path.join(RAIZ, ruta)), porque)

    print(f"{GRIS}  invariantes{FIN}")
    el_token_de_meta_no_cruza_la_frontera()
    lo_que_dice_la_gente_no_se_guarda()
    la_firma_se_comprueba_antes_de_analizar()
    una_calidad_sin_consultar_no_es_buena()
    las_dos_marcas_no_estan_en_el_alta()

    print(f"{GRIS}  el recorrido del usuario{FIN}")
    if rapido:
        omitir("W", "sonda: el recorrido del usuario", "--rapido")
    else:
        check_cmd(
            "W",
            "sonda: el recorrido del usuario",
            ["npm", "run", "--silent", "sonda:recorrido"],
            "Recorre las trece pantallas en un navegador y mira qué encuentra "
            "quien llega: que ninguna esté muda, que todas tengan algo que "
            "hacer, que ningún texto salga como clave, que no haya adjetivos "
            "donde el §94 pide cifras, que nada diga «entregado» (§65), que "
            "nadie reciba un error de bienvenida y que ningún dato aparezca "
            "como «no reconocido». Encontró cinco pantallas que recibían con "
            "«Ocurrió un error» y el estado de campaña mal traducido.",
            cwd=os.path.join(RAIZ, "app"),
        )

    print(f"{GRIS}  el cable entre las capas{FIN}")
    lo_que_se_construye_se_usa()
    ninguna_tabla_se_queda_sin_escritor()
    toda_pantalla_se_puede_abrir()
    ningun_comando_se_queda_sin_quien_lo_llame()


def toda_capacidad_del_nucleo_tiene_puerta():
    """**Lo que el núcleo sabe hacer, alguien lo tiene que poder pedir.**

    El séptimo caso del patrón, y el que lo cierra de raíz.

    Las tres comprobaciones que ya existen miran **de dentro hacia fuera**:
    `lo_que_se_construye_se_usa` exige que `arles-app` dependa de cada crate,
    `ninguna_tabla_se_queda_sin_escritor` que cada tabla tenga quien escriba, y
    `toda_pantalla_se_puede_abrir` que cada `.vue` tenga su ruta. **Ninguna
    pregunta lo contrario**: qué sabe hacer el núcleo que nadie puede pedir.

    Por eso pasó lo siguiente, y pasó durante seis entregas sin que nada
    fallara: la Fase 4 construyó Pausar, Reanudar, Detener y la parada de
    emergencia; `arles-db` los expone; están probados y sobreviven a cerrar el
    programa; los textos en español llevan escritos en `es.ts` desde la Fase 6
    — y **ningún comando de Tauri los llamaba**, así que una campaña activada
    no se podía parar desde la aplicación.

    Los seis anteriores dejaban una función sin usar. Éste dejaba al usuario
    **sin freno** sobre un proceso que escribe a personas reales.

    ─────────────────────────────────────────────────────────────────────────
    CÓMO SE COMPRUEBA, Y POR QUÉ LA LISTA ES A MANO

    Se podría intentar deducir «qué métodos cambian estado» leyendo todo
    `arles-db`, y sería una comprobación que casi funciona: la clase de
    comprobación que da falsos negativos en silencio y que este proyecto ya ha
    pagado cuatro veces.

    Así que la lista es explícita. Cada entrada dice el método de `arles-db` y
    el comando que lo tiene que alcanzar. **Añadir una capacidad sin añadirla
    aquí no falla**, y eso es una limitación real y declarada; lo que esta
    comprobación garantiza es que **lo que está en la lista no se rompe**, y
    que quien añada una capacidad de control encuentre el sitio donde apuntarla.
    """
    # Se leen todos los `.rs` de arles-db por el mismo motivo que los de
    # arles-app: una lista de archivos a mano envejece, y el defecto que esto
    # caza es precisamente el de una suposición que dejó de ser cierta.
    carpeta_db = os.path.join(RAIZ, "crates", "arles-db", "src")
    db = "".join(
        open(os.path.join(carpeta_db, f), encoding="utf-8").read()
        for f in sorted(os.listdir(carpeta_db))
        if f.endswith(".rs")
    )
    # Se leen TODOS los `.rs` del crate, no una lista escrita a mano.
    #
    # La primera versión llevaba la lista a mano —`comandos.rs`, `campanas.rs`,
    # `whatsapp.rs`— y falló en su estreno: los comandos nuevos se escribieron
    # en `control.rs`, que no estaba en ella, así que la comprobación siguió
    # diciendo «falta el comando» con el comando delante.
    #
    # Es el mismo defecto que esta comprobación existe para cazar, una capa más
    # arriba: algo construido que el que mira no ve. Una lista a mano de dónde
    # buscar envejece; el directorio, no.
    carpeta = os.path.join(RAIZ, "crates", "arles-app", "src")
    app = "".join(
        open(os.path.join(carpeta, f), encoding="utf-8").read()
        for f in sorted(os.listdir(carpeta))
        if f.endswith(".rs")
    )
    registro = leer("crates", "arles-app", "src", "lib.rs")

    # (método de arles-db, comando que lo alcanza, para qué sirve)
    #
    # ── TAMBIÉN LAS DE LECTURA, Y NO ES UN AÑADIDO MENOR ─────────────────
    #
    # La primera versión sólo listaba las capacidades de CONTROL, y por eso no
    # cazó el octavo caso del patrón — que ocurrió en la 7.2, en la misma
    # entrega que documenta el patrón: `ritmo_por_hora` se construyó con su
    # prueba y **ningún comando lo llamaba**, así que la pantalla no podía
    # decir «al ritmo de las últimas 6 h, termina el jueves».
    #
    # Una consulta sin puerta no rompe nada y no se ve: es trabajo hecho que
    # no llega a nadie. Van en la misma lista.
    PUERTAS = [
        # Control (7.0)
        ("aplicar_a_la_campana", "pausar_campana", "pausar una campaña"),
        ("aplicar_a_la_campana", "reanudar_campana", "reanudar una campaña"),
        ("aplicar_a_la_campana", "detener_campana", "detener una campaña"),
        ("poner_parada_de_emergencia", "parar_todo", "parar todos los envíos"),
        ("levantar_parada_de_emergencia", "levantar_parada", "volver a enviar"),
        ("parada_de_emergencia", "estado_del_envio", "saber si hay algo parado"),
        # Cifras (7.2)
        ("resumen_de_la_empresa", "actividad", "ver cómo va cada campaña"),
        ("cupo_de_los_remitentes", "actividad", "ver el cupo de cada remitente"),
        ("ritmo_por_hora", "actividad", "saber a qué ritmo va una campaña"),
    ]

    sin_puerta = []
    for metodo, comando, para_que in PUERTAS:
        if f"pub fn {metodo}" not in db:
            # El método ya no existe: no es este fallo, y decirlo como si lo
            # fuera mandaría a quien lo lea al sitio equivocado.
            sin_puerta.append(f"{metodo}: ya no está en arles-db (¿se renombró?)")
            continue
        if f"fn {comando}" not in app:
            sin_puerta.append(f"{para_que}: falta el comando `{comando}`")
        elif comando not in registro:
            sin_puerta.append(f"{para_que}: `{comando}` existe pero no está registrado en lib.rs")
        elif f".{metodo}(" not in app:
            # ── LA COMPROBACIÓN QUE DE VERDAD CIERRA EL PATRÓN ─────────────
            #
            # Que exista un comando con el nombre correcto **no significa que
            # llame a la capacidad**. La primera versión se quedaba en las dos
            # líneas de arriba, y por eso no cazó el octavo caso: `actividad`
            # existía y estaba registrado, y `ritmo_por_hora` seguía sin que
            # nadie lo invocara.
            #
            # Verificado rompiéndolo: renombrando la llamada en `cifras.rs` de
            # `arles-app`, esta comprobación lo dice.
            sin_puerta.append(
                f"{para_que}: `{comando}` existe pero **no llama a "
                f"`{metodo}`**. Una capacidad construida que nadie invoca es "
                f"trabajo hecho que no llega a nadie"
            )

    check(
        7,
        "toda capacidad de control tiene su comando",
        not sin_puerta,
        toda_capacidad_del_nucleo_tiene_puerta.__doc__,
        detalle="\n      ".join(sin_puerta),
    )


def el_freno_llega_hasta_la_pantalla():
    """No basta con el comando: tiene que haber un botón.

    La otra mitad del §1. Un comando que ninguna pantalla invoca es el mismo
    hueco una capa más arriba — es exactamente lo que pasó con
    `guardar_plantilla` en la Fase 6, que tenía comando y no tenía editor.

    ───────────────────────────────────────────────────────────────────────────
    Y ESTA COMPROBACIÓN TUVO EL MISMO DEFECTO QUE PERSEGUÍA

    Miraba **dos de los seis** comandos de control: `pausar_campana` y
    `detener_campana`. Los tres de la parada de emergencia —`parar_todo`,
    `levantar_parada`, `estado_del_envio`— no estaban en la lista, así que
    llegaron al cierre de la 7.0 **sin ninguna pantalla que los llamara** y
    esta comprobación pasó en verde las dos veces que se corrió.

    Lo encontró otra, por la dirección contraria: tres `case` del simulador de
    la vista previa que nadie pedía. Un comando sin nadie que lo llame y un
    decorado sin nadie que lo use son el mismo hueco visto desde los dos lados.

    Ahora la lista se lee **de los propios comandos de `control.rs`**, no a
    mano: el día que se añada un séptimo, entra aquí solo. Una lista escrita a
    mano es lo que dejó fuera a los tres de arriba.
    """
    import re as _re

    fuentes = []
    # Se leen todas las pantallas y todos los almacenes, y no cuatro archivos
    # elegidos: el freno podría vivir en otra pantalla —y de hecho la parada de
    # emergencia acabó en Campañas, que sí estaba, mientras los comandos no
    # estaban en la lista de comandos—.
    for carpeta in ("pantallas", "stores"):
        raiz = os.path.join(RAIZ, "app", "src", "app", carpeta)
        for archivo in sorted(os.listdir(raiz)):
            if archivo.endswith((".vue", ".ts")) and not archivo.endswith(".spec.ts"):
                fuentes.append(open(os.path.join(raiz, archivo), encoding="utf-8").read())

    # ── SIN COMENTARIOS, Y LA PRIMERA VERSIÓN CON ELLOS ────────────────────
    #
    # Buscaba el nombre del comando en el archivo entero. Y el almacén los
    # nombra en un comentario —el que cuenta por qué la parada llegó tarde—, así
    # que **la comprobación se satisfacía con la explicación del defecto**.
    # Comprobado el 30/09/2026: renombrando las dos llamadas a `xx_parar_todo`,
    # seguía pasando en verde.
    #
    # Ahora se quitan los comentarios de línea y de bloque antes de buscar, y lo
    # que se busca es el nombre **entre comillas**, que es como viaja en una
    # llamada. Hablar de un comando no es llamarlo.
    sin_comentarios = _re.sub(r"/\*.*?\*/|//[^\n]*|<!--.*?-->", "", "".join(fuentes), flags=_re.S)
    unido = sin_comentarios

    control = leer("crates", "arles-app", "src", "control.rs")
    del_nucleo = _re.findall(r"#\[tauri::command\]\s*\npub fn (\w+)", control)
    assert del_nucleo, "no se leyó ningún comando de control.rs: el patrón dejó de casar"

    faltan = [
        c
        for c in sorted(del_nucleo)
        if f"'{c}'" not in unido and f'"{c}"' not in unido
    ]
    check(
        7,
        "el freno llega hasta la pantalla",
        not faltan,
        el_freno_llega_hasta_la_pantalla.__doc__,
        detalle=f"comandos que ninguna pantalla ni almacén invoca: {faltan}",
    )


def ningun_slot_se_escribe_a_ciegas():
    """Un `<template #loquesea>` que el componente no declara **no existe**.

    Vue no avisa: no compila mal, no da error en consola, no rompe ninguna
    prueba. Sencillamente **no pinta nada**.

    Lo encontró la revisión a ojo de la 7.0, y lo que salió fue peor que el
    defecto que se buscaba: `AModal` sirve su pie por un slot llamado
    `acciones`, y el diálogo de baja de contactos llevaba desde la Fase 3
    escribiendo `#pie`. Resultado: **un diálogo destructivo sin ningún botón**,
    y el propio componente documenta que en una acción destructiva sólo cierran
    los botones del pie. Un callejón sin salida, en producción, seis fases.

    Se comprueba contra los slots que cada componente declara de verdad, leídos
    de su `<slot name="…">`, y no contra una lista escrita a mano: una lista a
    mano envejece, y el defecto que esto caza es precisamente el de una
    suposición que dejó de ser cierta.
    """
    import re as _re

    comp = os.path.join(RAIZ, "app", "src", "design", "componentes")
    declarados = {}
    for archivo in sorted(os.listdir(comp)):
        if not archivo.endswith(".vue"):
            continue
        fuente = open(os.path.join(comp, archivo), encoding="utf-8").read()
        declarados[archivo[:-4]] = set(_re.findall(r'<slot\s+name="([\w-]+)"', fuente))

    malos = []
    for carpeta, _, archivos in os.walk(os.path.join(RAIZ, "app", "src", "app")):
        for archivo in sorted(archivos):
            if not archivo.endswith(".vue"):
                continue
            ruta = os.path.join(carpeta, archivo)
            fuente = open(ruta, encoding="utf-8").read()
            # Qué componente envuelve cada `#slot`: se busca hacia atrás la
            # última etiqueta de componente abierta antes del template.
            for m in _re.finditer(r"<template\s+#([\w-]+)", fuente):
                # Qué componente ENVUELVE este template, con una pila.
                #
                # La primera versión tomaba «la última etiqueta que aparece
                # antes», y se equivocaba en cuanto había un componente
                # autocerrado —`<AEntrada … />`— entre la apertura y el
                # template: lo atribuía a ése. Dio un falso positivo en su
                # estreno, que es la forma más barata de descubrir que una
                # comprobación mide mal.
                pila = []
                # Se reconocen **los componentes que existen**, no los que
                # empiezan por A: `EstadoVacio` y `EstadoCargando` también
                # sirven slots, y un patrón por la inicial los dejaba fuera —
                # es decir, ciego justo donde vive uno de los defectos.
                nombres = "|".join(_re.escape(n) for n in sorted(declarados))
                for t in _re.finditer(
                    rf"<({nombres})(?P<cuerpo>(?:[^<>\"']|\"[^\"]*\"|'[^']*')*?)>"
                    rf"|</({nombres})>",
                    fuente[: m.start()],
                ):
                    if t.group(1):
                        if not t.group("cuerpo").rstrip().endswith("/"):
                            pila.append(t.group(1))
                    elif pila and pila[-1] == t.group(3):
                        pila.pop()
                if not pila:
                    continue
                componente = pila[-1]
                if componente not in declarados:
                    continue
                if m.group(1) not in declarados[componente]:
                    malos.append(
                        f"{archivo}: `#{m.group(1)}` sobre <{componente}>, que sirve "
                        f"{sorted(declarados[componente]) or 'ningún slot con nombre'}"
                    )

    check(
        7,
        "ningún slot se escribe a ciegas",
        not malos,
        ningun_slot_se_escribe_a_ciegas.__doc__,
        detalle="\n      ".join(malos),
    )


def el_motor_no_tira_lo_que_sabe():
    """Cada paso del motor acaba escrito, no convertido en un sí/no.

    `Trabajador::un_paso` devuelve once desenlaces distintos, **cada uno con su
    clave de texto en español** escrita desde la Fase 4 y pensada para la
    pantalla de Actividad. Y `una_vuelta()` en `arles-app` hacía:

        .map(|ultimo| ultimo.hay_mas_que_hacer())

    Lo convertía en un booleano. De que a las 10:04 la cuenta de ventas se paró
    no quedaba nada: sólo el estado final de cada intento, sin cuándo ni por
    qué. Seis entregas sin que nada fallara, porque un dato que no se guarda no
    rompe ninguna prueba.

    Se comprueba que el pegamento use `correr_contando` y que lo que anota sea
    `clave_i18n()` y no un texto: un texto en español dentro de la tabla haría
    imposible corregir el mensaje sin migrar millones de filas.
    """
    pegamento = leer("crates", "arles-app", "src", "motor.rs")

    faltas = []
    if "correr_contando" not in pegamento:
        faltas.append(
            "`una_vuelta` no usa `correr_contando`: los pasos del motor se "
            "vuelven a tirar"
        )
    if "clave_i18n()" not in pegamento:
        faltas.append(
            "lo que se anota no es `Paso::clave_i18n()`: un texto en la tabla "
            "no se puede traducir ni corregir después (§139)"
        )
    # Y que el recorte por repetición siga en pie: sin él, el motor escribe
    # 17 280 filas al día por etapa fuera de horario.
    if "anotar_si_cambia" not in pegamento:
        faltas.append(
            "no se usa `anotar_si_cambia` para los sucesos de bucle: la tabla "
            "se llena de «fuera de ventana» repetido"
        )

    check(
        7,
        "el motor no tira lo que sabe",
        not faltas,
        el_motor_no_tira_lo_que_sabe.__doc__,
        detalle="\n      ".join(faltas),
    )


def las_cifras_salen_de_un_solo_sitio():
    """El total es la suma de las campañas, y la agrupación vive en el núcleo.

    Dos garantías de la 7.2, y las dos existen por el mismo motivo: **dos
    formas de calcular lo mismo acaban discrepando**, y cuando lo hacen la
    pantalla enseña dos números que no cuadran y nadie sabe cuál creer.

    1. `resumen_de_la_empresa` suma los resúmenes de campaña. No puede tener un
       `SELECT` propio sobre `message_attempt`: eso sería la segunda forma.
    2. Los nueve estados se agrupan con `GrupoDeDesenlace`, del núcleo. La
       necesitan la tarjeta de campaña, el filtro del registro y el resumen
       exportable; con una copia en cada sitio, la que se olvide de cambiar
       enseñaría otra cifra sin que nada falle.

    Y una tercera, de honestidad: las cifras salen de `message_attempt`, que es
    la prueba de que se envió, **no de `activity_event`**, que es la narración
    y puede perder una línea (ver la cabecera de la V10).
    """
    cifras = leer("crates", "arles-db", "src", "cifras.rs")
    import re as _re

    faltas = []

    cuerpo = _re.search(
        r"pub fn resumen_de_la_empresa\(.*?\n    \}", cifras, _re.S
    )
    cuerpo = cuerpo.group(0) if cuerpo else ""
    if "message_attempt" in cuerpo:
        faltas.append(
            "`resumen_de_la_empresa` consulta `message_attempt` por su cuenta: "
            "el total tiene que ser la suma de las campañas, no otra consulta"
        )
    if "total.sumar" not in cuerpo:
        faltas.append("`resumen_de_la_empresa` no suma los resúmenes de campaña")

    if "GrupoDeDesenlace" not in cifras:
        faltas.append(
            "la agrupación de estados no usa `GrupoDeDesenlace` del núcleo: "
            "es una copia que puede divergir"
        )
    # Se busca la CONSULTA, no la palabra: la cabecera de este módulo explica
    # justamente que las cifras **no** salen de `activity_event`, y una
    # comprobación que mire el texto suelto marca en rojo el comentario que
    # documenta la regla. Falso positivo en su estreno, corregido el 30/09/2026.
    if _re.search(r"(?i)FROM\s+activity_event", cifras):
        faltas.append(
            "las cifras consultan `activity_event`, que es la narración y puede "
            "perder líneas. Tienen que salir de `message_attempt`"
        )

    check(
        7,
        "las cifras salen de un solo sitio",
        not faltas,
        las_cifras_salen_de_un_solo_sitio.__doc__,
        detalle="\n      ".join(faltas),
    )


def todo_comando_que_piden_las_pantallas_lo_contesta_el_simulador():
    """Un comando que el frontend invoca y la vista previa no contesta **es un error en pantalla**.

    ───────────────────────────────────────────────────────────────────────────
    EL QUINTO CASO DEL PATRÓN, Y EL QUE MÁS CARO SALIÓ

    La vista previa es **donde Dirección revisa**. Su simulador responde los
    comandos de Tauri con datos de mentira, y lo que no responde acaba en un
    `Promise.reject` que la pantalla enseña como un error — o peor, en una
    pantalla vacía sin explicación.

    Pasó con **once comandos a la vez**, en cinco pantallas, y se descubrió
    recorriendo la demostración a mano. Nada podía cazarlo: las pruebas de Rust
    miran el núcleo, las de Vue montan los componentes con el store ya relleno,
    y el simulador es un literal de JavaScript que nadie compila.

    Se comprueba en las dos direcciones a propósito:

      - Un comando que el frontend pide y el simulador no contesta → la
        demostración enseña un error donde el producto funciona.
      - Un `case` del simulador que ningún comando pide → código muerto que
        alguien mantiene creyendo que sirve, y que esconde el renombrado de un
        comando: el simulador sigue contestando al nombre viejo y la pantalla
        pide el nuevo.
    """
    import re as _re

    simulador = leer("herramientas", "vista-previa", "simulador.js")
    contesta = set(_re.findall(r"case \"([a-z_]+)\":|case '([a-z_]+)':", simulador))
    contesta = {a or b for a, b in contesta}
    assert contesta, "no se leyó ningún `case` del simulador: la comparación sería contra la nada"

    # ── DOS LECTURAS, PORQUE LA PRIMERA VERSIÓN SE EQUIVOCÓ EN UNA ──────────
    #
    # `piden` son las llamadas con el nombre escrito en la propia llamada:
    # `invocar<Algo>('nombre')`. Es lo que hace falta para saber que un comando
    # **se usa**, y es fiable.
    #
    # `nombrados` es cualquier aparición del literal en el frontend, y hace
    # falta para la dirección contraria: el freno invoca a través de una
    # variable —`aplicar('detener_campana', …)`, con el comando en una unión de
    # tipos—, así que buscando sólo `invocar('…')` esta comprobación denunciaba
    # como muerto un `case` que se usa todos los días. Ocho falsos positivos en
    # su estreno, que es la forma más barata de descubrir que mide mal.
    piden = set()
    nombrados = set()
    for carpeta, _, archivos in os.walk(os.path.join(RAIZ, "app", "src")):
        for archivo in sorted(archivos):
            if not archivo.endswith((".ts", ".vue")) or archivo.endswith(".spec.ts"):
                continue
            fuente = open(os.path.join(carpeta, archivo), encoding="utf-8").read()
            piden |= set(
                _re.findall(r"invocar(?:<[^>]*>)?\(\s*[\"']([a-z_]+)[\"']", fuente)
            )
            nombrados |= set(_re.findall(r"[\"']([a-z_]+_[a-z_]+)[\"']", fuente))
    assert piden, "no se leyó ninguna llamada a `invocar`: el patrón dejó de casar"

    # El arranque de la aplicación pide estos tres antes de que haya pantalla;
    # los invoca `app.ts` con el nombre escrito, así que salen por `piden`.
    malos = []
    for cmd in sorted(piden - contesta):
        malos.append(f"la pantalla pide «{cmd}» y el simulador no lo contesta")
    for cmd in sorted(contesta - piden - nombrados):
        malos.append(f"el simulador contesta «{cmd}» y nadie en el frontend lo nombra")

    check(
        7,
        "la vista previa contesta lo que las pantallas piden",
        not malos,
        "\n      ".join(malos),
        todo_comando_que_piden_las_pantallas_lo_contesta_el_simulador.__doc__,
    )


def ningun_texto_lleva_una_arroba_suelta():
    """Una `@` en un texto **tumba la pantalla entera**, no sólo ese texto.

    ───────────────────────────────────────────────────────────────────────────
    CÓMO SE DESCUBRE, Y POR QUÉ NO LO CAZA NADA MÁS

    `vue-i18n` usa `@` para su sintaxis de enlaces —`@:otra.clave`— así que un
    texto con un correo de ejemplo no compila: «Message compilation error:
    Invalid linked format». Y no falla el texto: falla **el componente que lo
    pide**, con la pantalla en blanco.

    Pasó el 30/09/2026 al escribir la ayuda del buscador del registro —«escribe
    ana@empresa.mx»—, que es lo más natural del mundo poner en un campo que
    busca direcciones de correo. Las doce pruebas de esa pantalla fallaron a la
    vez con un mensaje que no nombra ni la clave ni el archivo.

    Ni `vue-tsc` ni ESLint lo ven: es una cadena válida de TypeScript. Sólo se
    nota al renderizar, y una pantalla sin prueba de montaje no lo notaría hasta
    que alguien la abriera.

    Lo que sí se permite: `@:` a propósito —el enlace de verdad— y una arroba
    escapada con `{'@'}`, que es como `vue-i18n` la admite literal.
    """
    import re as _re

    catalogo = leer("app", "src", "app", "locales", "es.ts")
    malos = []
    for n, linea in enumerate(catalogo.split("\n"), start=1):
        cruda = linea.split("//")[0]
        for m in _re.finditer(r"@", cruda):
            trozo = cruda[m.start() : m.start() + 2]
            # `@:clave` es un enlace de verdad, y se deja pasar.
            if trozo == "@:":
                continue
            # `{'@'}` es la forma escapada que `vue-i18n` admite. Dentro de una
            # cadena de comillas simples aparece como `{\'@\'}`, con las
            # comillas a su vez escapadas — y la primera versión de esto sólo
            # reconocía la forma sin escapar, así que denunció un texto que
            # estaba bien. Se ve al estrenarla, que es lo barato.
            antes = cruda[max(0, m.start() - 3) : m.start()]
            despues = cruda[m.start() + 1 : m.start() + 4]
            if antes.endswith(("{'", "{\\'")) and despues.startswith(("'}", "\\'}")):
                continue
            malos.append(f"es.ts:{n}: {cruda.strip()[:70]}")
            break

    check(
        7,
        "ningún texto lleva una arroba suelta",
        not malos,
        "\n      ".join(malos),
        ningun_texto_lleva_una_arroba_suelta.__doc__,
    )


def ningun_texto_lleva_markdown():
    """Los `**asteriscos**` de un texto salen **literales** en la pantalla.

    `vue-i18n` no interpreta Markdown: lo que se escribe es lo que se ve. Un
    `**no se tocan**` en un aviso se lee tal cual, con los cuatro asteriscos, y
    encima en el sitio donde más se mira — la frase que alguien resaltó porque
    era la importante.

    Pasó el 30/09/2026 en el aviso del recorte, y no lo ve nada: es una cadena
    de TypeScript perfectamente válida, la prueba del componente busca «no se
    tocan» y lo encuentra —los asteriscos no estorban a un `toContain`—, y sólo
    se nota abriendo la pantalla.

    Se miran los valores, no los comentarios: los comentarios de este archivo
    usan Markdown a propósito y se leen en el editor, no en la aplicación.
    """
    import re as _re

    malos = []

    # ── 1 · El catálogo de la pantalla ──────────────────────────────────────
    catalogo = leer("app", "src", "app", "locales", "es.ts")
    for n, linea in enumerate(catalogo.split("\n"), start=1):
        cruda = linea.split("//")[0]
        # Sólo dentro de un valor: `clave: '…'` o la continuación de una cadena.
        if "'" not in cruda:
            continue
        for trozo in _re.findall(r"'([^']*)'", cruda):
            if "**" in trozo or _re.search(r"(?<![\w`])`[^`]+`(?![\w])", trozo):
                malos.append(f"es.ts:{n}: {trozo[:60]}")
                break

    # ── 2 · Y EL ARCHIVO QUE SE EXPORTA ────────────────────────────────────
    #
    # La primera versión sólo miraba `es.ts`, y el mismo día el defecto
    # apareció **en el núcleo**: el resumen exportable llevaba `**Esto no
    # quiere decir…**` en su frase más importante, y un `.txt` tampoco
    # interpreta Markdown. Una comprobación que mira un sitio y no el otro deja
    # pasar el mismo error por la puerta de al lado.
    #
    # Se miran los `push_str` de `resumen.rs`, que es lo que acaba en el
    # archivo; sus comentarios usan Markdown a propósito y se quedan fuera.
    resumen = leer("crates", "arles-core", "src", "resumen.rs")
    for m in _re.finditer(r'push_str\(\s*\n?\s*"((?:[^"\\]|\\.)*)"', resumen, _re.S):
        if "**" in m.group(1):
            linea = resumen[: m.start()].count("\n") + 1
            malos.append(f"resumen.rs:{linea}: {m.group(1)[:60]}")

    check(
        7,
        "ningún texto lleva Markdown",
        not malos,
        "\n      ".join(malos),
        ningun_texto_lleva_markdown.__doc__,
    )


def el_resumen_no_puede_llevar_direcciones():
    """El archivo que se exporta **no tiene dónde meter una dirección**.

    ───────────────────────────────────────────────────────────────────────────
    POR QUÉ ES UNA COMPROBACIÓN Y NO SÓLO UNA PRUEBA

    El resumen de una campaña se manda por correo: a Dirección, a un cliente.
    Si llevara la lista de destinatarios dentro, una campaña se convertiría en
    una filtración — y el descuido sería de una línea, en un `format!` que
    alguien añade para «que se vea mejor».

    La prueba de `arles-core` comprueba que el texto no lleva ninguna; ésta
    comprueba algo más fuerte y más difícil de romper sin querer: que
    `ResumenDeCampana`, la estructura que se redacta, **no tiene ningún campo
    donde pudiera entrar una**. Sin campo no hay descuido posible.

    Los nombres que se buscan son los que usan las tablas y los tipos del
    proyecto para una dirección o un teléfono.
    """
    import re as _re

    fuente = leer("crates", "arles-core", "src", "resumen.rs")
    cuerpo = _re.search(
        r"pub struct ResumenDeCampana \{(.*?)\n\}", fuente, _re.S
    )
    assert cuerpo, "no se encontró la estructura del resumen: el patrón dejó de casar"

    prohibidos = (
        "direccion",
        "correo",
        "email",
        "telefono",
        "contacto",
        "destinatario",
    )
    malos = [
        linea.strip()
        for linea in cuerpo.group(1).split("\n")
        # Sólo los campos, no los comentarios que los explican.
        if linea.strip().startswith("pub ")
        and any(p in linea.lower() for p in prohibidos)
    ]

    check(
        7,
        "el resumen exportable no puede llevar direcciones",
        not malos,
        "\n      ".join(malos),
        el_resumen_no_puede_llevar_direcciones.__doc__,
    )


def tablas_del_esquema_final():
    """Las tablas que existen **después** de correr todas las migraciones.

    No vale quedarse con todos los `CREATE TABLE`: la V3, la V4 y la V9 usan el
    rodeo que SQLite obliga a hacer para cambiar una restricción —crear
    `x_nueva`, copiar, `DROP TABLE x`, renombrar—, así que hay cuatro nombres
    que se crean y **dejan de existir en la misma migración**.

    Contándolos, el manifiesto del respaldo tendría que clasificar cuatro tablas
    que no existen, y quien lo leyera buscaría en vano `message_attempt_nueva`
    en el esquema.
    """
    import glob

    creadas, renombradas = set(), set()
    for sql in sorted(glob.glob(os.path.join(RAIZ, "crates/arles-db/migrations", "*.sql"))):
        with open(sql, encoding="utf-8") as f:
            texto = f.read()
        creadas |= set(re.findall(r"CREATE TABLE (?:IF NOT EXISTS )?(\w+)", texto))
        renombradas |= set(re.findall(r"ALTER TABLE (\w+) RENAME TO", texto))
    return creadas - renombradas


def ninguna_tabla_se_queda_fuera_del_respaldo():
    """**La forma más cara del patrón del §9: datos que desaparecen sin error.**

    Un respaldo se escribe una vez y se mantiene durante años. El día que
    alguien añada una tabla en la V11 y no la meta en el exportador, **nada
    falla**: se exporta, se restaura, y los datos de esa tabla no están. No hay
    excepción, ni aviso, ni una prueba en rojo. Lo descubre el usuario meses
    después, cuando ya no hay de dónde sacarlos.

    Es el mismo hueco que `ninguna_tabla_se_queda_sin_escritor` caza en la otra
    dirección —una tabla que nadie rellena— y se escribe igual: leyendo **el
    esquema**, no una lista a mano. Una lista a mano envejece, y envejecer en
    silencio es exactamente el problema.

    Clasificar una tabla como `Nunca` es una respuesta válida —`ui_preference`
    es de este equipo, no de estos datos— pero hay que **decirlo y por qué**. Lo
    que no se puede es omitirla: el olvido no tiene dónde esconderse.

    Esta comprobación se escribió **antes** que el manifiesto, y se vio fallar
    nombrando las 29 tablas sin clasificar (§2).
    """
    manifiesto = leer("crates/arles-core/src/respaldo.rs") or ""

    # Se leen los nombres tal y como el manifiesto los declara. Si el patrón
    # deja de casar, la aserción lo dice en vez de pasar en verde con cero.
    declaradas = set(re.findall(r'tabla:\s*"(\w+)"', manifiesto))
    assert manifiesto, "no existe crates/arles-core/src/respaldo.rs"
    assert declaradas, "el manifiesto no declara ninguna tabla: el patrón dejó de casar"

    # Las del propio motor de migraciones no son datos de nadie.
    DE_SISTEMA = {"refinery_schema_history", "schema_version", "sqlite_sequence"}
    reales = tablas_del_esquema_final() - DE_SISTEMA

    sin_clasificar = sorted(reales - declaradas)
    inventadas = sorted(declaradas - reales)

    check(
        8,
        "ninguna tabla se queda fuera del respaldo",
        not sin_clasificar and not inventadas,
        ninguna_tabla_se_queda_fuera_del_respaldo.__doc__,
        detalle=(
            f"en el esquema y sin clasificar: {sin_clasificar}\n"
            f"      clasificadas y sin existir en el esquema: {inventadas}"
        ),
    )


def toda_tabla_que_no_entra_dice_por_que():
    """Una exclusión sin motivo es un olvido con mejor presentación.

    El campo existe en el tipo, así que el compilador obliga a rellenarlo — pero
    nada impide escribir `""` o `"pendiente"`. Aquí se exige que el motivo sea
    una frase, no un hueco, y que las excluidas lo lleven de verdad.

    Verificada el 05/10/2026 vaciando el motivo de `ui_preference`: falla
    nombrándola.
    """
    manifiesto = leer("crates/arles-core/src/respaldo.rs") or ""
    entradas = re.findall(
        r'tabla:\s*"(\w+)",\s*alcance:\s*Alcance::(\w+),\s*porque:\s*(.+?),?\n',
        manifiesto,
    )
    assert entradas, "no se leyó ninguna entrada del manifiesto"

    flojas = [t for t, _, p in entradas if len(p.strip().strip('"')) < 25]
    check(
        8,
        "toda tabla excluida dice por qué",
        not flojas,
        toda_tabla_que_no_entra_dice_por_que.__doc__,
        detalle=f"sin motivo escrito: {sorted(flojas)}",
    )


def las_bajas_nunca_se_recortan():
    """**D-8B, hecha comprobación.**

    La lista de supresión es el control anti-abuso del producto: perder una
    baja significa volver a escribirle a quien pidió que no. Por eso entra
    **entera en los dos alcances** y no se recorta por fecha ni se queda fuera
    del esencial.

    Es una regla de negocio, no de formato, así que se vigila aparte: una tabla
    mal clasificada pasaría las otras dos comprobaciones sin protestar.

    Verificada el 05/10/2026 bajando `suppression_entry` a `SoloCompleto`:
    falla con su nombre.
    """
    manifiesto = leer("crates/arles-core/src/respaldo.rs") or ""
    m = re.search(
        r'tabla:\s*"suppression_entry",\s*alcance:\s*Alcance::(\w+)', manifiesto
    )
    alcance = m.group(1) if m else "(no declarada)"
    check(
        8,
        "las bajas entran enteras en los dos respaldos",
        alcance == "SiempreEntera",
        las_bajas_nunca_se_recortan.__doc__,
        detalle=f"suppression_entry está como: {alcance}",
    )


def fase_8(rapido=False):
    print(f"{NEGRITA}Fase 8 · Respaldos y endurecimiento{FIN}")

    print(f"{GRIS}  el manifiesto del respaldo (8.0){FIN}")
    ninguna_tabla_se_queda_fuera_del_respaldo()
    toda_tabla_que_no_entra_dice_por_que()
    las_bajas_nunca_se_recortan()


def fase_7(rapido=False):
    print(f"{NEGRITA}Fase 7 · Actividad y entregabilidad{FIN}")

    print(f"{GRIS}  el freno (7.0){FIN}")
    toda_capacidad_del_nucleo_tiene_puerta()
    el_freno_llega_hasta_la_pantalla()
    ningun_slot_se_escribe_a_ciegas()

    print(f"{GRIS}  el registro de actividad (7.1){FIN}")
    el_motor_no_tira_lo_que_sabe()

    print(f"{GRIS}  las cifras (7.2){FIN}")
    las_cifras_salen_de_un_solo_sitio()

    print(f"{GRIS}  la pantalla de lo de ahora (7.3){FIN}")
    todo_comando_que_piden_las_pantallas_lo_contesta_el_simulador()

    print(f"{GRIS}  el registro con sus filtros (7.4){FIN}")
    ningun_texto_lleva_una_arroba_suelta()
    ningun_texto_lleva_markdown()

    print(f"{GRIS}  el resumen exportable (7.6){FIN}")
    el_resumen_no_puede_llevar_direcciones()


# El canal de WhatsApp no es una fase numerada: cruza la 5 y la 6 y se entregó
# aparte. Va con clave "W" para que `--fase W` funcione y para que el recorrido
# completo la incluya sin renumerar nada.
FASES = {0: fase_0, 1: fase_1, 2: fase_2, 3: fase_3, 4: fase_4, 5: fase_5,
         6: fase_6, "W": fase_w, 7: fase_7, 8: fase_8}


def main():
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    p.add_argument(
        "--fase",
        type=lambda v: v.upper() if v.upper() in ("W",) else int(v),
        choices=list(FASES),
        metavar="{0,1,2,3,4,5,6,W,7,8}",
        help="valida solo esa fase",
    )
    p.add_argument("--rapido", action="store_true", help="omite lo que tarda")
    p.add_argument("--json", metavar="RUTA", help="escribe el informe en JSON")
    a = p.parse_args()

    print(f"{NEGRITA}Validación de ARLES RELAY I · v1.2.0{FIN}")
    print(f"{GRIS}{RAIZ}{FIN}")

    t0 = time.time()
    # `sorted` no puede comparar int con str, así que el orden se escribe: las
    # numeradas y al final el canal.
    todas = [k for k in FASES if isinstance(k, int)] + [k for k in FASES if not isinstance(k, int)]
    for n in ([a.fase] if a.fase is not None else sorted(
        [k for k in todas if isinstance(k, int)]) + [k for k in todas if not isinstance(k, int)]):
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
