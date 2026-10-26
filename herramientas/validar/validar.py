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
import difflib
import hashlib
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
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

def la_aplicacion_se_construye_como_aplicacion():
    """**El binario que se entrega no puede quedar en modo de desarrollo.**

    Tauri NO decide por el perfil de cargo si la aplicación es un programa
    terminado o un entorno de desarrollo. Lo decide por una característica del
    compilador:

        pub const fn is_dev() -> bool { !cfg!(feature = "custom-protocol") }

    Sin ella, **incluso compilando en `--release`**, la ventana abre buscando
    la interfaz en el servidor de desarrollo y enseña «no se puede acceder a
    esta página». La aplicación compila, arranca, abre su ventana — y no sirve
    para nada.

    ── POR QUÉ ESTO ESTUVO NUEVE FASES SIN VERSE ──

    Porque **nadie había construido la aplicación**. Todo lo que el proyecto
    verifica lo verifica con pruebas, que no abren ninguna ventana, y con la
    vista previa, que es HTML en un navegador sin nada de Rust detrás. El
    andamiaje estándar de Tauri declara esta característica desde el primer
    día; aquí faltaba, y lo encontró Dirección al abrir ARLES por primera vez
    el 08/10/2026.

    Es el patrón del §9 en el último tramo que quedaba: **la junta entre
    «compila» y «se puede usar»**.

    Se comprueban cuatro piezas, porque cualquiera sola se queda corta: la
    característica declarada, el alias que la usa (para que nadie tenga que
    acordarse), que la guía mande ese comando, y que **ningún bloque de la guía
    mande el que no sirve**.

    ── POR QUÉ LA CUARTA, Y POR QUÉ MIRANDO SÓLO LOS BLOQUES ──

    La tercera nació el 08/10/2026 preguntando `"cargo arles" in guia`, sobre el
    archivo entero. Pasó en verde ese mismo día **sobre una guía cuyo bloque de
    arranque decía `cargo run --release -p arles-app`**: la cadena buena estaba,
    pero en los avisos de solución de problemas del §5, no donde se copia. Y el
    equipo nuevo de Dirección arrancó con el comando viejo porque lo leyó ahí.

    Es la trampa del comentario con otra cara: **medir que una cadena aparece no
    es medir que el sitio que importa la tenga.** Por eso esta mira los bloques
    de código y no la prosa — que nombra `cargo run --release` a propósito, para
    explicar por qué no se usa.

    Verificada el 08/10/2026 rompiendo las cuatro: quitando la característica,
    el `--features` del alias, el `cargo arles` del bloque, y devolviendo el
    `cargo run --release -p arles-app` al §3. Las cuatro en rojo. Y medida en el
    binario, que es lo que de verdad importa — sin la característica,
    20 017 216 bytes y **cero** apariciones de `index.html`; con ella,
    20 377 184 y siete.

    **La cuarta nació con un hueco propio**, encontrado antes de darla por
    buena: los bloques de los avisos viven dentro de citas, y `^\\s*cargo run`
    no casa `> cargo run`. Colar el comando viejo ahí —que es justo donde se
    escriben los comandos de salida de un tropiezo— pasaba en verde. Verificada
    otra vez con esa tercera ruptura, ya en rojo.
    """
    cargo = leer(os.path.join(RAIZ, "crates/arles-app/Cargo.toml"))
    alias = leer(os.path.join(RAIZ, ".cargo/config.toml"))
    guia = leer(os.path.join(RAIZ, "documentacion/07-entrega/COMO-ABRIR-ARLES.md"))

    declarada = 'custom-protocol = ["tauri/custom-protocol"]' in sin_comentarios_toml(cargo)
    check(
        1,
        "arles-app declara la característica custom-protocol",
        declarada,
        "Sin ella la aplicación abre en modo desarrollo y busca la interfaz en localhost.",
    )

    en_el_alias = "--features custom-protocol" in sin_comentarios_toml(alias)
    check(
        1,
        "el alias `cargo arles` construye la aplicación de verdad",
        en_el_alias,
        "Un alias sin la característica deja la ventana pidiendo localhost, que es el defecto que vino a evitar.",
    )

    # Sólo los bloques de código, que son los que alguien copia y pega. La prosa
    # de la guía nombra `cargo run --release` a propósito, para explicar por qué
    # NO se usa; medir el archivo entero confunde las dos cosas.
    #
    # El `> ` de cada línea se quita porque los bloques de los avisos viven
    # dentro de citas, y ahí es justo donde se escriben los comandos de salida
    # de un tropiezo. Sin quitarlo, `^\s*cargo run` no casa `> cargo run` y la
    # comprobación mira los bloques sueltos y **ninguno de los citados**.
    bloques = "\n".join(
        re.sub(r"(?m)^> ?", "", b)
        for b in re.findall(r"```[a-z]*\n(.*?)```", guia, re.S)
    )

    check(
        1,
        "la guía de arranque manda el comando que funciona",
        "cargo arles" in bloques,
        "Una guía que manda `cargo run --release` a secas describe un arranque que no sirve.",
    )

    viejo = re.search(r"^\s*cargo run\b.*\barles-app\b", bloques, re.M)
    check(
        1,
        "ningún bloque de la guía manda el arranque que no sirve",
        viejo is None,
        "Un bloque que se puede copiar pesa más que el aviso que lo desmiente tres secciones más abajo.",
        f"Encontrado: {viejo.group(0).strip() if viejo else ''}",
    )

    # ── EL OTRO CAMINO, QUE NO VIGILABA NADIE ────────────────────────────────
    #
    # Las tres de arriba vigilan `cargo arles`, que es como se abre ARLES **a
    # mano**. El instalador que descarga Dirección no sale de ahí: lo construye
    # `entrega-de-prueba.yml` con `tauri-apps/tauri-action`, y de ese camino no
    # decía nada ninguna comprobación.
    #
    # Importa porque `tauri-action` ejecuta `tauri build`, y es el CLI de Tauri
    # quien añade `custom-protocol`. Cambiarlo por un `cargo build --release` a
    # secas —el reflejo natural de quien quiera «simplificar» el flujo— produce
    # un instalador que compila, instala, abre su ventana y enseña «localhost
    # refused to connect». Exactamente el defecto del 08/10/2026, pero ya
    # empaquetado y en manos de alguien.
    #
    # Medido el 13/10/2026 sobre el instalador real, extrayendo el NSIS: el
    # binario de dentro pesa 22 266 368 bytes y trae **siete** apariciones de
    # `index.html` y los assets del frontend (`AsistenteDeCampana-*.js`). Las
    # mismas siete que el 08/10 con la característica activada.
    #
    # Verificada rompiéndola de dos formas: cambiando `tauri-action` por un paso
    # con `cargo build --release`, y quitando el paso de compilación entero.
    entrega = leer(os.path.join(RAIZ, ".github/workflows/entrega-de-prueba.yml"))
    sin_comentarios = re.sub(r"(?m)^\s*#.*$", "", entrega)
    construye_con_tauri = "tauri-apps/tauri-action" in sin_comentarios
    check(
        1,
        "el instalador se construye con tauri-action, que activa custom-protocol",
        construye_con_tauri,
        "Un `cargo build` a secas en el flujo da un instalador que abre la ventana pidiendo localhost.",
    )


def sin_comentarios_toml(texto):
    """Quita los comentarios de un TOML.

    Octava vez que una comprobación de este proyecto se caza a sí misma en un
    comentario (5.3, 8.5, dos en la 8.3 bis, 8.6 A, 8.6 D, 8.6 E). Aquí los
    comentarios explican la característica **citándola entera**, así que buscar
    la cadena sin quitarlos pasaría en verde con la línea borrada.
    """
    return "\n".join(l.split("#", 1)[0] for l in texto.splitlines())


def sin_comentarios_de_barras(texto):
    """Quita los comentarios `//` y `/* … */` de Rust, TypeScript o Vue.

    La misma razón que `sin_comentarios_toml`, y en este proyecto es la novena
    vez: una comprobación que busca una cadena en un archivo **se caza a sí
    misma** en la explicación de por qué existe. Aquí los comentarios citan las
    operaciones enteras —`por_dia.div_ceil(horas)`, `Math.ceil(dia / horas)`—
    para explicar el defecto que vigilan, así que buscarlas sin quitarlos
    pasaría en verde con el código borrado.

    Sólo quita las líneas que **empiezan** por `//`, no el `//` a media línea:
    así una URL dentro de una cadena no parte la línea en dos. Para lo que esto
    mide es de sobra, y lo que se buscaba era no perder código por accidente.
    """
    sin_bloques = re.sub(r"/\*.*?\*/", "", texto, flags=re.S)
    return "\n".join(
        "" if l.lstrip().startswith("//") else l for l in sin_bloques.splitlines()
    )


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

    # ── Que lo que se construye sea una aplicación, no un andamio ──
    print(f"{GRIS}  la aplicación se construye como aplicación{FIN}")
    la_aplicacion_se_construye_como_aplicacion()

    # ── Fronteras de arquitectura ──
    print(f"{GRIS}  fronteras de arquitectura{FIN}")

    el_nucleo_no_toca_el_mundo()
    el_grafo_de_crates_solo_apunta_hacia_dentro()
    las_cifras_del_nucleo_son_las_mismas_en_la_pantalla()
    el_techo_del_dia_se_calcula_igual_en_los_dos_lados()

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

    # Se genera a un directorio temporal y se comparan CONTENIDOS. La versión
    # anterior escribía dentro del repositorio y preguntaba a `git diff`, y eso
    # tenía dos problemas: medir ensuciaba el árbol de trabajo, y dos corridas
    # del validador a la vez se pisaban el archivo — una escribía mientras la
    # otra comparaba, y salía en ROJO sobre unos tokens perfectamente
    # correctos. Medido el 22/10/2026: 1 falso de 40 con dos escritores.
    # Un validador no puede necesitar ser el único proceso del equipo.
    temporal = tempfile.mkdtemp(prefix="arles-tokens-")
    try:
        codigo, _, _ = corre([sys.executable, "herramientas/design-tokens/generar.py",
                              "--css", "--salida", temporal])
        recien = os.path.join(temporal, "arles-tokens.css")
        enelrepo = os.path.join(RAIZ, "herramientas/design-tokens/dist/arles-tokens.css")
        iguales, detalle = False, "No se pudo generar el CSS."
        if codigo == 0 and os.path.exists(recien) and os.path.exists(enelrepo):
            a = open(enelrepo, encoding="utf-8").read().splitlines()
            b = open(recien, encoding="utf-8").read().splitlines()
            iguales = a == b
            if not iguales:
                detalle = "\n".join(difflib.unified_diff(
                    a, b, "dist/arles-tokens.css", "generado desde tokens.json",
                    lineterm="", n=1))
        check(1, "El CSS generado coincide con tokens.json", codigo == 0 and iguales,
              "§17: si el CSS se edita a mano, deja de haber una fuente única de color.",
              detalle)
    finally:
        shutil.rmtree(temporal, ignore_errors=True)

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


def bloque_de(texto, clave, sangria):
    """El cuerpo de `clave: { … }` en un catálogo de textos, por su sangría.

    Por la RUTA y no por la hoja. `es.ts` tiene varias `cuerpo:` y varias
    `guardadaDetalle:` —una por pantalla—, así que buscar la clave suelta mide
    el primer bloque del archivo y no el que importa. La primera versión de la
    comprobación de la plantilla hizo exactamente eso y **salió en rojo sobre
    dos textos correctos**: es el mismo defecto que la 7.7 arregló en la
    comprobación del §139, repetido.

    La sangría basta porque `es.ts` lo formatea prettier y no hay dos
    anidamientos del mismo nombre al mismo nivel.

    Está a nivel de módulo desde el 22/10/2026, cuando la comprobación de los
    umbrales del semáforo la necesitó: era una función local de otra
    comprobación, y copiarla habría dejado dos sitios donde arreglar el mismo
    defecto de acotación — que es exactamente la clase de duplicado que este
    validador persigue.
    """
    m = re.search(
        rf"^{' ' * sangria}{clave}: \{{\n(.*?)^{' ' * sangria}\}},?$",
        texto,
        re.S | re.M,
    )
    return m.group(1) if m else ""


def las_cifras_del_nucleo_son_las_mismas_en_la_pantalla():
    """**Dos números de negocio viven en Rust y en TypeScript, y nada los
    mantenía iguales.**

    Encontrado por la auditoría A del §140 el 07/10/2026.

    La duplicación **es deliberada en los dos casos** y está escrita: la
    pantalla necesita enseñar el aviso *antes* de llamar a nadie, así que no
    puede preguntárselo al núcleo. Lo que faltaba no era quitar la copia, era
    que alguien comprobara que las dos dicen lo mismo.

    ── POR QUÉ IMPORTA, CON LO QUE PASARÍA ──

    - **`AVISO_DIARIO` = 50** (`arles-core/src/ritmo.rs` y
      `app/src/app/stores/campanas.ts`). Si el núcleo sube a 100 y la pantalla
      se queda en 50, el asistente **exige aceptar un aviso que el núcleo ya no
      pide**. Al revés es peor: la pantalla no enseña el aviso, el usuario
      rellena nueve pasos, y `crear_campana` lo rechaza al final con un error
      que no vio venir.
    - **Los meses del historial = 12** (`MESES_EN_EL_ESENCIAL` en el núcleo y
      el primer valor de `MESES_QUE_SE_OFRECEN` en el archivado). Si no
      coinciden, la pantalla ofrece un periodo que la frontera **sube en
      silencio**: el preflight enseña una cifra y se archiva otra.

    Las dos son del mismo tipo que los seis tokens de diseño inventados de la
    5.2: nada falla, nada cambia de color, y lo que se rompe es lo que el
    usuario ve.

    Verificada el 07/10/2026 cambiando el 50 de `ritmo.rs` a 75: falla
    nombrando las dos cifras.
    """
    def numero(texto, patron):
        m = re.search(patron, texto or "")
        return int(m.group(1)) if m else None

    ritmo = leer("crates", "arles-core", "src", "ritmo.rs")
    campanas = leer("app", "src", "app", "stores", "campanas.ts")
    respaldo = leer("crates", "arles-core", "src", "respaldo.rs")
    archivado = leer("app", "src", "app", "stores", "archivado.ts")

    # Acotado al bloque `riesgo:` y no a todo `es.ts`: «Hasta 60» puede
    # aparecer en cualquier otro texto del catálogo, y entonces esto mediría la
    # hoja equivocada. Es el defecto que la 7.7 arregló en la comprobación del
    # §139 y que volvió a aparecer el 22/10 con `cuerpo:`.
    bloque_del_semaforo = bloque_de(
        leer("app", "src", "app", "locales", "es.ts") or "", "riesgo", 4
    )

    parejas = [
        (
            "el aviso de envíos diarios",
            numero(ritmo, r"AVISO_DIARIO:\s*u32\s*=\s*(\d+)"),
            numero(campanas, r"AVISO_DIARIO\s*=\s*(\d+)"),
        ),
        (
            "los meses de historial que se quedan",
            numero(respaldo, r"MESES_EN_EL_ESENCIAL:\s*u32\s*=\s*(\d+)"),
            numero(archivado, r"MESES_QUE_SE_OFRECEN\s*=\s*\[\s*(\d+)"),
        ),
        # ── LOS UMBRALES DEL SEMÁFORO (D-14, 22/10/2026) ─────────────────
        #
        # Aquí la cifra NO está duplicada en TypeScript: el juicio lo hace
        # `RiesgoDelBuzon` en el núcleo y la pantalla recibe su clave de i18n.
        # Lo que sí puede separarse son **los textos**, que nombran los
        # umbrales con todas sus letras —«Hasta 60 correos al día», «Por encima
        # de 150»— porque un semáforo que no dice dónde están sus rayas no
        # sirve para decidir.
        #
        # Si Dirección mueve un umbral y los textos se quedan, el aviso dice un
        # número y el color obedece a otro. Nada falla, y lo que se rompe es lo
        # único que esa pantalla existe para dar: una cifra en la que confiar.
        (
            "el umbral verde del semáforo de remitentes",
            numero(ritmo, r"TRANQUILO_HASTA:\s*u32\s*=\s*(\d+)"),
            numero(bloque_del_semaforo, r"Hasta\s+(\d+)\s+correos al día"),
        ),
        (
            "el umbral ámbar del semáforo de remitentes",
            numero(ritmo, r"VIGILAR_HASTA:\s*u32\s*=\s*(\d+)"),
            numero(bloque_del_semaforo, r"Por encima de\s+(\d+)\s+al día"),
        ),
    ]

    problemas = []
    for nombre, en_rust, en_ts in parejas:
        if en_rust is None or en_ts is None:
            # Que una no se encuentre es tan grave como que no coincidan: la
            # comprobación habría pasado en verde sin mirar nada. Es la trampa
            # de `el_manifiesto_se_lee_entero`, aplicada aquí.
            problemas.append(
                f"{nombre}: no se encuentra en "
                f"{'Rust' if en_rust is None else 'TypeScript'}"
            )
        elif en_rust != en_ts:
            problemas.append(f"{nombre}: Rust dice {en_rust} y la pantalla {en_ts}")

    check(1, "las cifras de negocio dicen lo mismo en el núcleo y en la pantalla",
          not problemas,
          las_cifras_del_nucleo_son_las_mismas_en_la_pantalla.__doc__,
          detalle="\n      ".join(problemas))



def el_techo_del_dia_se_calcula_igual_en_los_dos_lados():
    """**El tope de la hora no puede dejar el del día sin cumplir, y la
    pantalla y el núcleo tienen que estar de acuerdo en cuántos caben.**

    ── EL DEFECTO QUE ESTA COMPROBACIÓN VIGILA, MEDIDO EL 22/10/2026 ──

    Los dos topes se piden por separado en el paso 5, y el de la hora puede
    frenar al del día. Con lo que la pantalla rellenaba sola —el por hora de la
    cuenta, diez— y la ventana de nueve a dieciocho:

        se pide 500 al día  →  salían  90
        se pide 100 al día  →  salían  90

    o sea que **cualquier tope por encima de noventa no se cumplía**, y nadie lo
    decía. No se veía porque con los 50 del §48 todo cuadra: el defecto empezaba
    justo donde Dirección quiere trabajar. Lo arregla `por_hora_que_hace_falta`
    en el núcleo y el cálculo gemelo del asistente, que rellena el campo.

    ── POR QUÉ HAY DOS CÁLCULOS Y NO UNO ──

    Igual que `AVISO_DIARIO`: **la pantalla tiene que avisar antes de llamar a
    nadie**, así que no puede preguntárselo al núcleo. La duplicación es
    deliberada y lo que faltaba era que alguien comprobara que las dos dicen lo
    mismo.

    Si se separan, pasa una de dos y las dos son malas:

    - la pantalla rellena **de menos** → se vuelve al defecto de arriba, y el
      aviso tampoco salta porque usa el mismo cálculo;
    - la pantalla rellena **de más** → el campo propone un por hora que el
      núcleo no necesita, y quien lo lea pensará que ARLES no sabe sumar.

    Esto no compara cifras —no hay ninguna que comparar— sino **que las dos
    operaciones existan y redondeen igual**: `div_ceil` en Rust y `Math.ceil` en
    la pantalla. Un `/` o un `Math.floor` en cualquiera de los dos lados deja el
    tope diario sin alcanzar por poco, que es el mismo defecto en pequeño y más
    difícil de ver.

    Verificada el 22/10/2026 por los dos lados: cambiando `div_ceil` por `/` en
    el núcleo y `Math.ceil` por `Math.floor` en el asistente.
    """
    nucleo = leer("crates", "arles-core", "src", "duracion.rs") or ""
    asistente = leer("app", "src", "app", "pantallas", "AsistenteDeCampana.vue") or ""

    # Sin comentarios antes de medir. Es la lección que este proyecto ha
    # aprendido ocho veces: una comprobación que busca una cadena en un archivo
    # se caza a sí misma en la explicación de por qué existe.
    nucleo_codigo = sin_comentarios_de_barras(nucleo)
    asistente_codigo = sin_comentarios_de_barras(asistente)

    problemas = []
    if "por_dia.div_ceil(horas)" not in nucleo_codigo:
        problemas.append(
            "el núcleo ya no redondea hacia arriba en `por_hora_que_hace_falta`: "
            "con una división entera, 500 en nueve horas da 55 y salen 495"
        )
    if "Math.ceil(dia / horas)" not in asistente_codigo:
        problemas.append(
            "el asistente ya no redondea hacia arriba al rellenar el por hora: "
            "el campo propondrá un número con el que el tope diario no se cumple"
        )
    if "laHoraFrenaElDia" not in asistente_codigo:
        problemas.append(
            "el asistente ya no avisa de que el tope de la hora frena al del día: "
            "quien baje el por hora a mano mandará menos de lo que pidió y nadie "
            "se lo dirá"
        )

    check(1, "el techo del día se calcula igual en el núcleo y en la pantalla",
          not problemas,
          el_techo_del_dia_se_calcula_igual_en_los_dos_lados.__doc__,
          detalle="\n      ".join(problemas))

def nada_sale_por_correo_sin_pasar_por_el_saneado():
    """**El saneado «al usar» del §8: hoy no aplica, y el día que aplique no
    habrá nada que lo recuerde.**

    Encontrado por la auditoría B del §140 el 07/10/2026.

    `THREAT_MODEL.md` §8 pide sanear el HTML **al guardar y al usar**. Al
    guardar está hecho y probado: `crear_plantilla` y `crear_firma` pasan por
    `sanear::html`. **Al usar no existe**, y por un motivo que no es un olvido:
    `Mensaje` sólo lleva el destinatario y la clave de idempotencia, y el único
    `Contenido` que llega al adaptador es un texto fijo escrito en el código
    —el del envío de prueba—. **El cuerpo de una campaña no llega al remitente
    todavía.**

    ── POR QUÉ ESTO ES UNA COMPROBACIÓN Y NO UNA NOTA ──

    El día que alguien conecte el cuerpo de la plantilla al envío, el camino
    nuevo pasará por aquí **sin que nada pida sanear otra vez**. Y el saneado al
    guardar no cubre ese caso: entre guardar y enviar está la sustitución de
    variables, que mete datos de contacto importados de un XLSX ajeno en medio
    del HTML ya saneado.

    Es el patrón del §9 aplicado a una regla de seguridad: las dos piezas
    correctas y el agujero en la junta, que ninguna prueba de una u otra puede
    cazar.

    Así que esto vigila la condición de hoy —que `Contenido` se arme de
    constantes— y falla el día que deje de ser cierta, **para que quien lo
    conecte tenga que decidir dónde se sanea**.

    Verificada el 07/10/2026 armando un `Contenido` con una variable en vez de
    una constante: falla nombrando el archivo.
    """
    # Quién construye un `Contenido` fuera de las pruebas, y con qué.
    problemas = []
    base = os.path.join(RAIZ, "crates", "arles-app", "src")
    for carpeta, _, ficheros in os.walk(base):
        for f in sorted(ficheros):
            if not f.endswith(".rs"):
                continue
            ruta = os.path.join(carpeta, f)
            with open(ruta, encoding="utf-8") as fh:
                texto = fh.read()
            corte = texto.find("#[cfg(test)]")
            if corte > 0:
                texto = texto[:corte]
            for m in re.finditer(r"Contenido\s*\{(.*?)\}", texto, re.S):
                cuerpo = m.group(1)
                campo = re.search(r"cuerpo\s*:\s*([^,\n]+)", cuerpo)
                if not campo:
                    continue
                valor = campo.group(1).strip()
                # Lo que vale hoy: una cadena literal, una vacía, o una
                # concatenación de literales. Cualquier otra cosa es un cuerpo
                # que viene de fuera.
                es_constante = (
                    valor.startswith('"')
                    or valor.startswith("String::new()")
                    or valor.startswith("String::from(\"")
                )
                if not es_constante and "sanear" not in cuerpo:
                    rel = os.path.relpath(ruta, RAIZ)
                    problemas.append(
                        f"{rel}: el cuerpo del correo sale de `{valor[:40]}` "
                        f"y no pasa por `sanear`"
                    )

    check(5, "nada sale por correo sin pasar por el saneado", not problemas,
          nada_sale_por_correo_sin_pasar_por_el_saneado.__doc__,
          detalle="\n      ".join(problemas))


def el_nucleo_no_toca_el_mundo():
    """**Cero I/O en `arles-core`, y el reloj también es I/O.**

    Es la regla que sostiene la arquitectura entera: un dominio que no toca el
    mundo se prueba sin infraestructura, da el mismo resultado dos veces y no
    necesita una base ni una red para decir si una dirección vale.

    ── LO QUE ESTA COMPROBACIÓN NO MIRABA, Y SE VIO EL 07/10/2026 ──

    La versión anterior buscaba cinco cadenas —`std::fs`, `std::net`,
    `rusqlite`, `reqwest`, `tokio::net`— en los archivos **sueltos** de
    `src/`, con un `os.listdir` que **no entra en subcarpetas**. Dos huecos:

    1. **El reloj no estaba en la lista.** Y el proyecto repite en tres sitios
       que el núcleo no tiene reloj —«la fecha la calcula `arles-app`, que es
       quien tiene reloj: ni el núcleo ni este crate lo tienen»—. Verificado
       metiendo un `OffsetDateTime::now_utc()` en `respaldo.rs`: la
       comprobación **pasó en verde**. Tampoco miraba la aleatoriedad, que es
       lo mismo: la 8.2a declara que la sal y el nonce entran como parámetros
       **porque el núcleo no pide entropía al sistema**.
    2. **Sin recursión.** Hoy `arles-core/src` no tiene subcarpetas, así que el
       hueco no se nota; el día que tenga una, todo lo que viva dentro deja de
       mirarse **sin que nada cambie de color**.

    Es el mismo patrón que la 7.7 y la 8.5: una comprobación que dice cubrir
    una regla y cubre una parte, y la parte que falta es justo la que nadie ha
    roto todavía.

    Verificada el 07/10/2026 con las dos rupturas: un reloj y un
    `getrandom::fill` dentro del núcleo. Falla nombrando el archivo y la línea.
    """
    # Lo que un dominio puro no puede tener. El reloj y la aleatoriedad están
    # aquí por derecho propio: los dos hacen que la misma entrada dé salidas
    # distintas, que es exactamente lo que impide probar sin infraestructura.
    PROHIBIDO = (
        "std::fs", "std::net", "std::io", "std::env", "std::process",
        "rusqlite", "reqwest", "tokio::net", "File::", "OpenOptions",
        "SystemTime::now", "Instant::now", "OffsetDateTime::now",
        "getrandom", "rand::",
    )

    encontrados = []
    base = os.path.join(RAIZ, "crates", "arles-core", "src")
    for carpeta, _, ficheros in os.walk(base):
        for f in sorted(ficheros):
            if not f.endswith(".rs"):
                continue
            ruta = os.path.join(carpeta, f)
            with open(ruta, encoding="utf-8") as fh:
                lineas = fh.read().split("\n")
            dentro_de_pruebas = False
            for n, linea in enumerate(lineas, 1):
                # Las pruebas del núcleo sí pueden pedir un temporal o un
                # reloj: lo que no puede es el código que viaja al instalador.
                if "#[cfg(test)]" in linea:
                    dentro_de_pruebas = True
                if dentro_de_pruebas or linea.strip().startswith("//"):
                    continue
                for p in PROHIBIDO:
                    if p in linea:
                        rel = os.path.relpath(ruta, RAIZ)
                        encontrados.append(f"{rel}:{n} → {p}")

    check(1, "arles-core no toca el mundo: ni disco, ni red, ni reloj, ni azar",
          not encontrados,
          el_nucleo_no_toca_el_mundo.__doc__,
          detalle="\n      ".join(encontrados[:8]))


def el_grafo_de_crates_solo_apunta_hacia_dentro():
    """**Las dependencias entre crates van en una dirección y sólo en una.**

    ── LO QUE SE VIGILABA, Y LO QUE NO ──

    Hasta el 07/10/2026 se comprobaba **una sola arista**: que `arles-core` no
    dependiera de `arles-db`. El resto del grafo no lo miraba nadie, y el resto
    del grafo es donde vive la arquitectura:

    - `arles-db` **no conoce la IPC**: no depende de `arles-app` ni de `serde`.
    - `arles-motor` **no sabe que los adaptadores existen**: define el rasgo, y
      `arles-email` y `arles-whatsapp` lo implementan. La flecha va de los
      adaptadores al motor, nunca al revés.
    - **Nadie depende de `arles-app`**, que es la frontera: una dependencia
      hacia él convertiría un crate de dominio en parte de la interfaz.
    - `arles-banco` **no entra en el instalador**, así que nadie puede
      depender de él (eso ya lo vigila `el_banco_no_viaja_en_el_instalador`).

    El grafo se lee de los `Cargo.toml`, no de una lista escrita aquí: lo que
    se compara es lo que Cargo compila.

    Verificada el 07/10/2026 añadiendo `arles-app` a las dependencias de
    `arles-db`: falla nombrando la arista.
    """
    # Quién puede depender de quién. Vacío = de nadie.
    PERMITIDO = {
        "arles-core": set(),
        "arles-db": {"arles-core"},
        "arles-import": {"arles-core"},
        "arles-motor": {"arles-core", "arles-db"},
        "arles-email": {"arles-core", "arles-motor"},
        "arles-whatsapp": {"arles-core", "arles-motor"},
        "arles-app": {"arles-core", "arles-db", "arles-import", "arles-motor",
                      "arles-email", "arles-whatsapp"},
        # `arles-motor` entró el 07/10/2026 con la corrida larga de la
        # auditoría C: para medir si la memoria del motor se estabiliza hay que
        # hacerlo correr. La flecha apunta hacia dentro igual que las otras
        # tres, y el banco sigue sin que nadie dependa de él.
        #
        # Esta comprobación la cazó el mismo día que se añadió, que es
        # exactamente para lo que la escribió la auditoría A.
        "arles-banco": {"arles-core", "arles-db", "arles-import", "arles-motor"},
    }

    problemas = []
    crates_dir = os.path.join(RAIZ, "crates")
    presentes = sorted(d for d in os.listdir(crates_dir)
                       if os.path.isdir(os.path.join(crates_dir, d)))

    # Un crate nuevo sin su fila aquí es un crate cuya posición en el grafo no
    # ha decidido nadie. Eso también es un fallo, y es el que más barato sale
    # cazar el día que se añade.
    for c in presentes:
        if c not in PERMITIDO:
            problemas.append(f"{c}: crate nuevo sin sitio declarado en el grafo")

    for c in presentes:
        if c not in PERMITIDO:
            continue
        toml = leer("crates", c, "Cargo.toml") or ""
        usadas = set(re.findall(r"^\s*(arles-[a-z]+)\s*[=.]", toml, re.M))
        for d in sorted(usadas - PERMITIDO[c]):
            problemas.append(f"{c} → {d}: esta flecha no existe en la arquitectura")

    check(1, "el grafo de crates sólo apunta hacia dentro",
          not problemas,
          el_grafo_de_crates_solo_apunta_hacia_dentro.__doc__,
          detalle="\n      ".join(problemas))


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
    # ── SE LEEN LOS CRATES QUE HAY, NO DOS ESCRITOS A MANO ──
    #
    # Hasta el 07/10/2026 la lista decía `("arles-core", "arles-db")`: **dos de
    # siete**. Los otros cuatro de dominio —import, motor, email, whatsapp— lo
    # tenían puesto y nada lo sostenía, y `arles-banco` no lo tenía. Es el mismo
    # patrón que el §139 en la 7.7 y que el cero I/O de aquí al lado: una
    # comprobación que dice cubrir una regla y cubre una parte.
    #
    # `arles-app` queda fuera a propósito y con su motivo escrito arriba: es el
    # único con una excepción auditada, y tiene su propia comprobación debajo.
    crates_dir = os.path.join(RAIZ, "crates")
    de_dominio = sorted(
        d for d in os.listdir(crates_dir)
        if os.path.isdir(os.path.join(crates_dir, d, "src")) and d != "arles-app"
    )
    sin_prohibicion = []
    for crate in de_dominio:
        fuente = leer("crates", crate, "src", "lib.rs") or ""
        # ── SE QUITAN LOS COMENTARIOS ANTES DE MIRAR ──
        #
        # Sin esto, comentar el atributo —`// #![forbid(unsafe_code)]`— deja la
        # comprobación en verde: la cadena sigue ahí, dentro del comentario.
        # Se vio el 07/10/2026 haciendo justo esa ruptura en `arles-motor`: la
        # prohibición desaparecía del crate y nada cambiaba de color. **Quinta
        # vez** que este patrón aparece en el proyecto (5.3, 8.5 y dos veces en
        # la 8.3 bis), y la lección es siempre la misma: una comprobación que
        # busca una palabra en un archivo con comentarios la encuentra en los
        # comentarios.
        sin_comentarios = re.sub(r"//[^\n]*", "", fuente)
        if "#![forbid(unsafe_code)]" not in sin_comentarios:
            sin_prohibicion.append(crate)
    check(1, "todo crate que no sea la frontera prohíbe unsafe en su raíz",
          not sin_prohibicion,
          "Un crate de dominio que acepta `unsafe` ha dejado de ser de dominio. "
          "La lista sale de las carpetas que hay, no de dos nombres escritos aquí. "
          "Verificada el 07/10/2026 quitando el `forbid` de arles-motor, y "
          "**comentándolo**, que es la ruptura que la versión anterior no cazaba.",
          detalle=f"sin forbid: {', '.join(sin_prohibicion)}")

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

    las_tres_partes_de_un_error_no_salen_pegadas()

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
    # Los seis índices que la medición del 06/10/2026 demostró que faltaban.
    # No cambia ni un dato: sólo añade listas ordenadas. Entrega 8.6.
    "V11__indices_de_rendimiento.sql":
        "a97d3bddc184e9ba38cae4f3592c7242273999e9dda2bb3d33cb1f8f4c0db555",
    # Retirar una plantilla o una firma sin borrarla, para que lo que una
    # campaña envió siga pudiéndose consultar. Decisión de Dirección del
    # 21/10/2026.
    "V12__retirar_sin_borrar.sql":
        "c06782f9e32da0d970985fc80a09e345436dc4d5b62e36fd441d6e712960397f",
    # El cupo diario pasa a contarse por campaña y no por buzón, para que dos
    # campañas de 100 sobre el mismo remitente manden 200 y cada una cumpla el
    # plazo que prometió. Decisión de Dirección del 22/10/2026 (D-13).
    "V13__cupo_por_campana.sql":
        "1bccf390993d00f7f144a0100a9b57c9d7dd4d19394edf7efcd0db82e1261f36",
    # A quién va dirigida una campaña (D-15, 22/10/2026). Hasta aquí toda
    # campaña iba a todo el padrón, que es la vía más rápida a que marquen el
    # correo como no deseado. Las tablas de listas ya estaban en la V1: esto
    # sólo las ata a la campaña.
    "V14__lista_de_la_campana.sql":
        "eab9cd6f92c811847f2af47151cc8a1445a8435ffeb173e92914872552b23507",
    # El descanso entre campañas (D-16, 22/10/2026): el índice que permite
    # preguntar, antes de cada envío, cuándo se le escribió por última vez a
    # esta dirección **desde otra campaña**. Ninguno de los cuatro que había
    # empieza por `contact_address`.
    "V15__descanso_entre_campanas.sql":
        "4a0c07644ba40434a85f859f1e67c77177edef2ca3f6083faeb4ecb022482c20",
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


def _rutas_del_catalogo():
    """Todas las rutas del catálogo de textos, resueltas ENTERAS.

    La ruta y no la hoja, que es el defecto que la 7.7 arregló en la
    comprobación del §139 y que yo mismo volví a cometer el 26/10/2026
    midiendo con un `grep` de sangría: dio por duplicado un bloque
    `producto.contactos.supresion` contra `producto.supresion`, que son dos
    cosas distintas. **La sangría de este archivo no es de fiar** —hay bloques
    anidados escritos con dos espacios— así que lo que manda es el recuento de
    llaves.
    """
    texto = leer("app/src/app/locales", "es.ts") or ""
    rutas, pila, nivel = set(), [], 0
    for linea in texto.split("\n"):
        # Comentarios y textos fuera ANTES de contar llaves: un texto como
        # `'{fallan} por arreglar'` lleva una `}` dentro, y contándola el nivel
        # se descuadra a mitad del archivo (misma razón que `_esqueleto_del_catalogo`).
        s = re.sub(r"//.*$", "", linea)
        s = re.sub(r"'(?:\\.|[^'\\])*'", "''", s)
        s = re.sub(r'"(?:\\.|[^"\\])*"', '""', s)
        abre = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\{\s*$", s)
        if abre:
            pila.append(abre.group(1))
            rutas.add(".".join(pila))
            nivel += 1
            continue
        hoja = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)\s*:", s)
        if hoja and nivel:
            rutas.add(".".join(pila + [hoja.group(1)]))
        nivel += s.count("{") - s.count("}")
        while len(pila) > max(nivel, 0):
            pila.pop()
    # El catálogo entero cuelga de `producto`; las claves de Rust no lo nombran.
    return {r[len("producto.") :] if r.startswith("producto.") else r for r in rutas}


def toda_clave_de_pantalla_del_nucleo_tiene_texto():
    """Toda clave que devuelve un `clave_i18n()` del núcleo tiene su texto.

    ── EL HUECO QUE ESTO CIERRA ──

    `claves_de_error_sin_texto` cubre el §139 **sólo para las claves
    `error.*`**. Las de pantalla —`preflight.*`, `motor.*`, `intento.*`,
    `entregabilidad.*`— las devuelve el mismo mecanismo y no las miraba nadie.

    Es el patrón de siempre: una comprobación que dice cubrir una regla y cubre
    la parte que alguien rompió una vez. Y la consecuencia es la de siempre —
    una clave sin texto **no revienta nada**: `vue-i18n` enseña la cadena cruda
    y el usuario lee `entregabilidad.claveRetirada` donde debería leer una
    frase.

    Medido al escribirla el 26/10/2026: el núcleo devuelve **133** claves de
    pantalla y las únicas sin texto eran las diez de la 7.5 recién escritas.
    Nació en rojo sobre exactamente lo que faltaba, que es como se sabe que
    mide algo.
    """
    rutas = _rutas_del_catalogo()

    claves = set()
    directorio = os.path.join(RAIZ, "crates", "arles-core", "src")
    for archivo in sorted(os.listdir(directorio)):
        if not archivo.endswith(".rs"):
            continue
        texto = leer("crates/arles-core/src", archivo) or ""
        # Sin las pruebas: una clave inventada en un `assert_eq!` no es una
        # clave que el núcleo devuelva.
        texto = texto.split("#[cfg(test)]")[0]
        for cuerpo in re.findall(
            r"fn clave_i18n\([^)]*\)\s*->\s*&.*?\{(.*?)\n    \}", texto, re.S
        ):
            for k in re.findall(r'=>\s*"([a-zA-Z][\w.]*\.[\w.]+)"', cuerpo):
                claves.add((archivo, k))

    faltan = sorted(f"{a} → {k}" for a, k in claves if k not in rutas)
    check(
        7,
        f"toda clave de pantalla del núcleo tiene texto ({len(claves)} claves)",
        bool(claves) and not faltan,
        "Una clave sin texto no falla: se enseña cruda. El usuario lee "
        "«entregabilidad.claveRetirada» donde iba una frase (§139).",
        "sin texto:\n  " + "\n  ".join(faltan),
    )


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

    # ── Y EL TABLERO, QUE ES EL TERCER DESTINO DESDE EL 14/10/2026 ──
    #
    # Dirección pidió «una carpeta donde podamos graficar los avances». Un
    # tablero con colores es el sitio donde un número viejo hace más daño: se
    # mira de un vistazo, se cree entero y nadie va a comprobarlo contra el
    # JSON. Es exactamente lo que le pasó al §9 del CLAUDE.md, que estuvo
    # diciendo «entrega 3.3» con la Fase 4 cerrada.
    #
    # El tablero se genera, así que coincide **el día que se genera**. Esto
    # vigila que no se haya quedado atrás: cambiar `avance.json` y empujar sin
    # regenerar deja el tablero enseñando el porcentaje de antes.
    tablero = leer("tablero", "ARLES-tablero.html")
    if tablero is None:
        check(3, "el tablero existe y está al día", False,
              "Dirección pidió el tablero el 14/10/2026 y se genera con "
              "`python3 herramientas/tablero/generar.py`.",
              "no existe tablero/ARLES-tablero.html")
    else:
        m = re.search(r">(\d+)%</text>", tablero)
        en_tablero = int(m.group(1)) if m else None
        check(3, "el avance del tablero coincide con la fuente",
              en_tablero == esperado,
              "Un tablero se mira de un vistazo y se cree entero: un "
              "porcentaje viejo ahí hace más daño que en ningún otro sitio. "
              "Se regenera con `python3 herramientas/tablero/generar.py`.",
              f"tablero {en_tablero} vs calculado {esperado}")

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


def los_nombres_del_dominio_son_los_mismos_a_los_dos_lados():
    """**El H-05: un enum con dos nombres bloqueó el producto entero.**

    Dirección no podía dar de alta ni un contacto. El error era:

        unknown variant `email`, expected `correo` or `whats_app`

    `Canal` llevaba `rename_all = "snake_case"`, así que al cruzar la frontera
    se llamaba `correo` y `whats_app`. En la base, en las `CHECK` del esquema y
    en toda la interfaz se llama `email` y `whatsapp`. **Dos nombres para la
    misma cosa, y la junta entre ellos era justo la frontera.**

    Llevaba así desde la Fase 3 con las diez fases en verde, y ninguna prueba
    podía verlo: las de Rust construyen los valores sin pasar por JSON y las
    del frontend no cruzan la frontera. Lo mismo le pasaba a
    `MotivoDeSupresion` y a otros cuatro enums.

    ───────────────────────────────────────────────────────────────────────────
    POR QUÉ HACEN FALTA DOS COMPROBACIONES Y NO UNA

    `el_nombre_que_cruza_la_frontera_es_el_de_la_base` (una prueba de Rust) ata
    **serde al nombre de la base**, ejecutando serde de verdad. Esto ata **el
    nombre de la base a lo que la interfaz escribe**. Las dos juntas cierran el
    camino entero; cualquiera de las dos sola deja un lado suelto.

    Y los nombres se leen de `como_texto()`, no se derivan del nombre de la
    variante: derivarlos es lo que hizo fallar la primera versión de
    `las_listas_de_la_importacion_no_divergen`, que reproducía el fallo en vez
    de cazarlo.
    """
    import re as _re

    def del_nucleo(archivo, tipo):
        """Los literales de `como_texto()` de ese enum."""
        rust = leer("crates", "arles-core", "src", archivo) or ""
        bloque = _re.search(
            rf"impl {tipo} \{{.*?pub const fn como_texto\(self\) -> &'static str \{{(.*?)\n    \}}",
            rust,
            _re.S,
        )
        return set(_re.findall(r'=> "([^"]+)"', bloque.group(1))) if bloque else set()

    def de_la_interfaz(archivo, tipo):
        """Los literales de `export type X = 'a' | 'b'` del almacén."""
        ts = leer("app", "src", "app", "stores", archivo) or ""
        bloque = _re.search(rf"export type {tipo} =(.*?)\n\n", ts, _re.S)
        return set(_re.findall(r"'([^']+)'", bloque.group(1))) if bloque else set()

    for archivo_rs, tipo, archivo_ts in [
        ("canal.rs", "Canal", "contactos.ts"),
        ("canal.rs", "Canal", "supresion.ts"),
        ("supresion.rs", "MotivoDeSupresion", "supresion.ts"),
    ]:
        rust = del_nucleo(archivo_rs, tipo)
        ts = de_la_interfaz(archivo_ts, tipo)
        check(
            3,
            f"«{tipo}» se llama igual en el núcleo y en {archivo_ts}",
            bool(rust) and rust == ts,
            los_nombres_del_dominio_son_los_mismos_a_los_dos_lados.__doc__,
            detalle=f"núcleo: {sorted(rust)}\ninterfaz: {sorted(ts)}",
        )


def la_plantilla_dice_las_columnas_que_lleva():
    """El texto que enumera las columnas de la plantilla las nombra TODAS.

    Dirección pidió la plantilla descargable el 22/10/2026 con tres columnas, y
    hay **tres sitios** que las nombran: la constante del núcleo
    (`PLANTILLA_DE_CONTACTOS`), que es la que acaba dentro del XLSX; el texto
    del acuse, que le dice al usuario qué va a encontrar al abrirla; y la
    entradilla del paso 1, que le dice qué debe decir la primera fila de su
    archivo.

    Si se añade una cuarta columna, los dos textos siguen diciendo tres. No
    falla nada: el archivo sale con cuatro, la pantalla promete tres, y quien lo
    abre cree que la de más sobra. **Un texto que enumera y se queda corto es
    peor que uno que no enumera**, porque se lee como la lista completa.

    Se lee la constante y no una lista de aquí: con una lista a mano, esto sería
    el cuarto sitio que hay que acordarse de tocar. Es el defecto que las cuatro
    auditorías del §140 encontraron once veces.

    ───────────────────────────────────────────────────────────────────────────
    VERIFICADA AÑADIENDO UNA CUARTA COLUMNA al núcleo —`(Apellido, "Apellido")`—
    sin tocar los textos: en rojo, nombrando «Apellido» como la que falta en los
    dos.

    LO QUE NO CUBRE, Y SE DICE PORQUE LA PRIMERA VERSIÓN DE ESTE COMENTARIO
    AFIRMABA LO CONTRARIO: **la dirección inversa.** Quitar «Teléfono» de la
    constante deja los textos prometiendo una columna que el archivo no lleva, y
    esto **pasa en verde** — se probó. Mide que cada columna del núcleo esté
    nombrada, no que lo nombrado exista.

    Cubrirla exigiría sacar de una frase en prosa «qué columnas nombra», y las
    palabras «Correo» y «Nombre» aparecen en esos textos por otros motivos: el
    instrumento daría hallazgos falsos. La dirección que queda sin medir es
    además la improbable —se añaden columnas, no se quitan— y la que sí deja
    rastro: la que se quitara seguiría saliendo en el XLSX de la prueba de ida y
    vuelta de `arles-import`.

    Está escrito porque en este proyecto **las afirmaciones de verificación son
    las que más veces resultan falsas**, y ésta lo era: se dio por hecha una
    ruptura en vez de correrla.
    """
    import re as _re

    rust = leer("crates", "arles-core", "src", "importacion.rs") or ""
    bloque = _re.search(
        r"pub const PLANTILLA_DE_CONTACTOS: &\[\(CampoImportable, &str\)\] = &\[(.*?)\n\];",
        rust,
        _re.S,
    )
    columnas = _re.findall(r'"([^"]+)"', bloque.group(1)) if bloque else []

    check(
        3,
        "la plantilla declara sus columnas en el núcleo",
        len(columnas) >= 1,
        la_plantilla_dice_las_columnas_que_lleva.__doc__,
        detalle=f"columnas leídas: {columnas}",
    )

    # Los comentarios se quitan ANTES de buscar. Es el defecto que este proyecto
    # ha cometido nueve veces: una comprobación que se caza a sí misma en el
    # comentario que explica por qué existe — y aquí el bloque `plantilla:` de
    # `es.ts` lleva encima un comentario que nombra las tres columnas.
    textos = leer("app", "src", "app", "locales", "es.ts") or ""
    sin_comentarios = _re.sub(r"^\s*//.*$", "", textos, flags=_re.M)

    dentro = bloque_de(sin_comentarios, "importacion", 2)
    check(
        3,
        "el bloque de textos de la importación se encuentra",
        bool(dentro),
        la_plantilla_dice_las_columnas_que_lleva.__doc__,
        detalle="no se localizó `importacion: {` en es.ts",
    )

    # Dos textos y no uno: el acuse dice qué trae el archivo que acabas de
    # guardar, y la entradilla del paso 1 dice qué debe decir la primera fila
    # del tuyo. Los dos enumeran, y los dos se quedarían cortos igual.
    for sub, clave in [("plantilla", "guardadaDetalle"), ("elegir", "cuerpo")]:
        frase = bloque_de(dentro, sub, 4)
        faltan = [c for c in columnas if c not in frase]
        check(
            3,
            f"el texto «{sub}.{clave}» nombra las {len(columnas)} columnas de la plantilla",
            bool(frase) and not faltan,
            la_plantilla_dice_las_columnas_que_lleva.__doc__,
            detalle=(
                f"columnas del núcleo: {columnas}\n"
                f"no aparecen en «importacion.{sub}»: {faltan or 'ninguna'}"
            ),
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


def una_baja_no_exige_que_la_direccion_sea_un_contacto():
    """**Se puede dar de baja una dirección que no está en la lista.**

    ─────────────────────────────────────────────────────────────────────────
    EL HUECO QUE ESTO CIERRA, MEDIDO EL 23/10/2026

    `suprimir_contacto` recibe un `ContactId`, así que hasta hoy una baja sólo
    se podía apuntar **si la dirección ya era un contacto**. Y el caso que de
    verdad llega no es ése: alguien a quien le reenviaron el correo, alguien
    que contesta desde otra dirección suya, o un contacto que se borró después.
    Esas direcciones **no se podían dar de baja de ninguna forma** — y la
    siguiente campaña les volvía a escribir.

    `Db::suprimir_direccion` existe desde la Fase 4, con su prueba y su
    cabecera explicando por qué hace falta habiendo ya `suprimir`: «el motor no
    tiene ficha». Lo que no existía era **el comando**. Es el patrón del §9 por
    vigesimotercera vez, y aquí la consecuencia no es una función que falta: es
    una persona que pidió no recibir y recibe.

    ─────────────────────────────────────────────────────────────────────────
    POR QUÉ LOS TRES TRAMOS Y NO SÓLO EL COMANDO

    Porque los tres huecos son distintos y ninguno hace fallar al otro:

        Db::suprimir_direccion → comando → registrado → alguien que lo llame

    `ningun_comando_se_queda_sin_quien_lo_llame` (fase W) cubre los dos
    últimos. El primero no lo cubre nadie: un método de la base que ningún
    comando llama no es un comando sin quien lo llame — es un comando que no
    existe, y eso no deja rastro en `generate_handler!`.

    Escrita **antes** de construir nada y vista fallar nombrando los tres
    tramos el 23/10/2026.
    """
    faltan = []

    # Sin comentarios, siempre: la cabecera de `supresion.rs` nombra
    # `suprimir_direccion` al explicar por qué existe, y buscarlo en el archivo
    # entero se cazaría ahí. Es el defecto que este proyecto lleva once veces.
    app = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "crates", "arles-app", "src")):
        for f in ficheros:
            if f.endswith(".rs"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    app += fh.read()
    assert app, "no se leyó el código de arles-app"
    app_sin_comentarios = sin_comentarios_de_barras(app)

    if ".suprimir_direccion(" not in app_sin_comentarios:
        faltan.append(
            "Db::suprimir_direccion — nadie en arles-app la llama, así que una "
            "dirección que no sea un contacto no se puede dar de baja"
        )

    lib = sin_comentarios_de_barras(leer("crates", "arles-app", "src", "lib.rs") or "")
    if "suprimir_direccion," not in lib:
        faltan.append(
            "el comando suprimir_direccion no está en generate_handler!, "
            "así que la webview no lo puede pedir"
        )

    front = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "app", "src")):
        for f in ficheros:
            if f.endswith((".ts", ".vue")) and not f.endswith(".spec.ts"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    front += fh.read()
    assert front, "no se leyó el código de app/src"
    if "'suprimir_direccion'" not in sin_comentarios_de_barras(front):
        faltan.append(
            "ninguna pantalla invoca suprimir_direccion — el comando existiría "
            "sin botón, que es el hueco de la W.8"
        )

    check(
        3,
        "una baja no exige que la dirección sea un contacto",
        not faltan,
        una_baja_no_exige_que_la_direccion_sea_un_contacto.__doc__,
        detalle="; ".join(faltan),
    )


def el_nombre_repetido_se_compara_igual_en_las_tres_capas():
    """**Las tres capas que comparan nombres comparan lo mismo.**

    ─────────────────────────────────────────────────────────────────────────
    QUÉ SE PIERDE SI SE SEPARAN

    El aviso de nombres repetidos (23/10/2026) suma dos cifras que calculan
    sitios distintos: cuántos trae el archivo —`arles-core`— y cuántos había ya
    —`arles-db`—. Y la demostración lo calcula una tercera vez, en JavaScript.

    Si una compara **sólo el nombre de pila** y otra el nombre entero, las dos
    cifras no se refieren a lo mismo y el aviso sale con un número inventado:
    «serán 4» cuando son 2. Una cifra mal es peor que ninguna (§94).

    Y es un error fácil: **la primera versión de esto comparaba sólo el nombre
    de pila**, en las tres capas, y daba un falso positivo con cada «Ana». Se
    corrigió antes de entrar; esta comprobación existe para que no vuelva por
    una de las tres.

    Es el mismo riesgo que `AVISO_DIARIO`, duplicado entre Rust y TypeScript
    con la comprobación que lo mantiene igual.

    Escrita después de corregir el defecto y **vista en rojo en los tres
    tramos**, uno a uno.
    """
    faltan = []

    # ── 1 · El núcleo, que es quien define la forma ──
    core = sin_comentarios_de_barras(
        leer("crates", "arles-core", "src", "analisis.rs") or ""
    )
    if "fn nombre_entero" not in core:
        faltan.append(
            "arles-core no tiene `nombre_entero`: no hay una forma única que "
            "las otras capas puedan copiar"
        )
    else:
        # ── EL CUERPO, NO LA FIRMA ──
        #
        # La primera versión miraba los 300 caracteres tras el nombre de la
        # función, y **la ruptura salió en verde**: el parámetro se sigue
        # llamando `apellido`, así que la palabra estaba ahí aunque el cuerpo
        # la ignorara. Medir que una cadena aparece no es medir que el sitio
        # que importa la tenga, que es la lección del `cargo arles`.
        cuerpo_del_nucleo = core.split("fn nombre_entero", 1)[1]
        cuerpo_del_nucleo = cuerpo_del_nucleo[cuerpo_del_nucleo.find("{") :][:300]
        if "apellido" not in cuerpo_del_nucleo:
            faltan.append(
                "`nombre_entero` no usa el apellido en su cuerpo: «Ana Ruiz» y "
                "«Ana Pérez» saldrían como un nombre repetido"
            )

    # ── 2 · La frontera, que arma los nombres del archivo ──
    app = sin_comentarios_de_barras(leer("crates", "arles-app", "src", "comandos.rs") or "")
    if "nombre_entero" not in app:
        faltan.append(
            "arles-app no usa `nombre_entero` para armar los nombres del "
            "archivo: compararía de otra forma que el núcleo"
        )

    # ── 3 · La base, que cuenta los que ya estaban ──
    db = sin_comentarios_de_barras(leer("crates", "arles-db", "src", "contactos.rs") or "")
    cuerpo = db.split("fn cuantos_se_llaman_asi")
    if len(cuerpo) < 2:
        faltan.append("arles-db no tiene `cuantos_se_llaman_asi`")
    elif "last_name" not in cuerpo[1][:1200]:
        faltan.append(
            "la consulta de `cuantos_se_llaman_asi` no mira `last_name`: "
            "contaría por nombre de pila y la cifra del aviso no cuadraría"
        )

    # ── 4 · Y la demostración, que lo calcula una tercera vez ──
    sim = sin_comentarios_de_barras(leer("herramientas", "vista-previa", "simulador.js") or "")
    # Y aquí lo mismo: **el bloque que agrupa**, no una ventana de caracteres
    # hacia atrás. La primera versión miraba los 900 anteriores al `push` y la
    # ruptura salió en verde, porque `apellido: apellido` aparece antes, al
    # armar cada contacto. Segunda vez en la misma comprobación.
    if "nombresRepetidos.push" not in sim:
        faltan.append("el simulador no calcula los nombres repetidos")
    elif "var porNombre" not in sim:
        faltan.append("el simulador no agrupa por nombre")
    else:
        agrupa = sim.split("var porNombre", 1)[1].split("});", 1)[0]
        if "apellido" not in agrupa:
            faltan.append(
                "el simulador agrupa sin el apellido: la demostración "
                "enseñaría un aviso que la aplicación no da"
            )

    check(
        3,
        "el nombre repetido se compara igual en las tres capas",
        not faltan,
        el_nombre_repetido_se_compara_igual_en_las_tres_capas.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_simulador_rechaza_con_claves_que_existen():
    """Las claves de error del simulador tienen que estar en el catálogo.

    ─────────────────────────────────────────────────────────────────────────
    DOS DEFECTOS EN CUATRO LÍNEAS, ENCONTRADOS EL 22/10/2026

    El `case 'crear_campana'` del simulador rechazaba con
    `error.app.falta_aceptar_el_aviso`, y pasaba las dos cosas a la vez:

    1. **La condición no se cumplía nunca.** Preguntaba por `args.avisoAceptado`
       y el asistente manda `aceptoElRiesgoDelVolumen`, así que la demostración
       dejaba crear una campaña de alto volumen sin aceptar el aviso — con un
       comentario encima diciendo que lo exigía igual que el núcleo.
    2. **Y la clave no existía.** Si la condición se hubiera cumplido, la
       pantalla habría enseñado la cadena cruda en vez de un error en español.

    Esto caza el segundo, que es el que se puede medir leyendo. El primero no
    tiene comprobación: una condición que nunca es cierta no deja rastro. Lo
    que sí deja rastro es **rechazar con un nombre que nadie va a entender**,
    y eso es lo que se mira aquí.

    El §139 no lo cubría: su comprobación recorre los `clave_i18n()` de Rust, y
    el simulador es JavaScript — ninguna de las dos listas se mira entre sí.

    ───────────────────────────────────────────────────────────────────────────
    Verificada volviendo a poner `error.app.falta_aceptar_el_aviso`: en rojo,
    nombrándola. Y verificada al revés, con una clave buena: en verde.
    """
    import re as _re

    sim = leer("herramientas", "vista-previa", "simulador.js") or ""
    sim_sin_comentarios = _re.sub(r"^\s*//.*$", "", sim, flags=_re.M)
    claves = sorted(set(_re.findall(r"clave:\s*'(error\.[\w.]+)'", sim_sin_comentarios)))

    textos = leer("app", "src", "app", "locales", "es.ts") or ""

    def existe(clave):
        """La RUTA entera, no la hoja: `error.db.x` vive en `db: { x: {` ."""
        partes = clave.split(".")
        resto = textos
        for p in partes:
            m = _re.search(rf"\b{_re.escape(p)}:\s*\{{", resto)
            if not m:
                return False
            resto = resto[m.end():]
        return True

    faltan = [c for c in claves if not existe(c)]
    # Fase 3 y no 7: es donde se ejecuta —la llamada vive en ese bloque, al
    # lado de las demás del simulador—, y una etiqueta que no coincide con la
    # fase que la corre manda a buscarla donde no está.
    check(
        3,
        f"las {len(claves)} claves con las que rechaza el simulador existen en es.ts",
        bool(claves) and not faltan,
        el_simulador_rechaza_con_claves_que_existen.__doc__,
        detalle=(
            f"claves del simulador: {claves}\n"
            f"sin texto en el catálogo: {faltan or 'ninguna'}"
        ),
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
    los_nombres_del_dominio_son_los_mismos_a_los_dos_lados()
    la_bomba_se_mide_antes_de_abrir_la_hoja()
    la_plantilla_dice_las_columnas_que_lleva()
    una_supresion_pedida_no_se_levanta_con_un_boton()
    una_baja_no_exige_que_la_direccion_sea_un_contacto()
    el_informe_de_importacion_consulta_las_supresiones()
    la_vista_previa_no_miente_sobre_las_rutas()
    el_nombre_repetido_se_compara_igual_en_las_tres_capas()
    el_simulador_rechaza_con_claves_que_existen()
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

    nombre = "sonda: ningún texto se monta sobre otro ni se queda cortado"
    porque = (
        "Dirección abrió la aplicación el 13/10/2026 y escribió de Remitentes: "
        "«el texto se ve extremadamente desordenado y poco profesional». Las "
        "dos sondas que recorren el producto estaban EN VERDE ese día, y las "
        "dieciocho pantallas sin un hallazgo.\n\n"
        "Lo que pasaba es que nadie medía lo que Dirección estaba viendo. "
        "`sonda:auditoria` mide desbordes con `scrollWidth > clientWidth`, que "
        "caza lo que no cabe DE ANCHO: dos párrafos montados uno sobre otro "
        "caben perfectamente, su contenedor no se ensancha ni un píxel. Y "
        "`sonda:recorrido` compara el borde y el hueco DEL TÍTULO; lo que pase "
        "cien píxeles más abajo no lo mira.\n\n"
        "Esta sonda mide las cuatro cosas que quedaban fuera, las cuatro "
        "geométricas: dos textos cuyos rectángulos se cruzan, texto que su "
        "contenedor recorta de alto, texto que asoma de su padre, y un RÓTULO "
        "cortado por los puntos suspensivos —cortar un dato está bien, cortar "
        "una cabecera de columna deja «WhatsApp pri…», que es lo que Contactos "
        "enseñaba—.\n\n"
        "Las cuatro se vieron en rojo el 14/10/2026, rompiendo el código de "
        "verdad. Y la sonda nació con TRES hallazgos falsos, los tres "
        "comprobados antes de anotarlos: el contenido de un `<details>` cerrado "
        "sigue devolviendo rectángulos (nueve falsos por ventana), un padre con "
        "`text-overflow: ellipsis` no deja asomar nada, y un `.sr-only` de "
        "1×1 px tiene la forma exacta de un rótulo recortado y se lee entero "
        "por el único camino que lo usa."
    )
    if motivo:
        omitir(3, nombre, motivo)
    elif rapido:
        omitir(3, nombre, "--rapido")
    else:
        check_cmd(3, nombre, ["npm", "run", "sonda:encimado", "--silent"],
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
    # ── SE LEE LA FRONTERA ENTERA, NO UN ARCHIVO ──
    #
    # Hasta el 07/10/2026 esto leía **sólo `comandos.rs`**: 17 de las 52
    # estructuras serializables de `arles-app`. La frontera creció a once
    # módulos —respaldo, restauración, archivo, recuperación, whatsapp,
    # campañas, cifras, control, registro, error, motor— y ninguno se miraba.
    #
    # Verificado por la auditoría B del §140 metiendo un
    # `pub contrasena: String` en una estructura de `respaldo.rs`: **pasó en
    # verde**. La regla más cargada de la frontera —ningún comando devuelve un
    # secreto— se vigilaba en un tercio de su superficie.
    sospechosos = []
    base = os.path.join(RAIZ, "crates", "arles-app", "src")
    for carpeta, _, ficheros in os.walk(base):
        for f in sorted(ficheros):
            if not f.endswith(".rs"):
                continue
            ruta = os.path.join(carpeta, f)
            with open(ruta, encoding="utf-8") as fh:
                texto = fh.read()
            # Las pruebas pueden construir lo que quieran: lo que no puede
            # tener un secreto es lo que se serializa de verdad.
            corte = texto.find("#[cfg(test)]")
            if corte > 0:
                texto = texto[:corte]
            for bloque in re.finditer(
                r"#\[derive\(([^)]*)\)\][^{]*?pub struct (\w+)\s*\{([^}]*)\}",
                texto,
                re.S,
            ):
                derivados, nombre, cuerpo = bloque.group(1), bloque.group(2), bloque.group(3)
                if "Serialize" not in derivados:
                    continue
                # Más nombres que antes: el token de Meta y la clave de una API
                # son secretos igual que una contraseña, y no los buscaba nadie.
                # `lleva_contrasena: bool` NO entra: dice si hace falta una, no
                # cuál es, y el preflight de la 8.3 lo necesita.
                campo = re.search(
                    r"^\s*(pub )?(contrasena|password|secreto|token|api_key|credencial)\w*\s*:\s*(String|Secret|Vec<u8>)",
                    cuerpo,
                    re.M,
                )
                if campo:
                    rel = os.path.relpath(ruta, RAIZ)
                    sospechosos.append(f"{rel} · {nombre}.{campo.group(2)}")
    check(5, "nada que cruce la frontera lleva un secreto", not sospechosos,
          "Regla 3 de la frontera: la contraseña entra y no sale. "
          "Se lee `arles-app/src` entero desde el 07/10/2026, no sólo `comandos.rs`.",
          detalle="\n      ".join(sorted(sospechosos)))


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
    nada_sale_por_correo_sin_pasar_por_el_saneado()
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
    la_lista_de_la_campana_llega_desde_el_archivo_hasta_el_envio()


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

    ───────────────────────────────────────────────────────────────────────────
    SU TRAMO 1 VIGILABA **CERO** FUNCIONES, Y ASÍ PASÓ EL H-06 (17/10/2026)

    El patrón decía `async function (\\w+)\\(…\\)[^{]*\\{(?:[^{}]|\\{[^{}]*\\})*?invocar`:
    intentaba llegar de la llave de apertura hasta `invocar` admitiendo como
    mucho **un** nivel de anidamiento. Y en este proyecto **todas** las
    funciones de store envuelven la llamada en un `try { … }`, así que el
    `invocar` está dos niveles dentro y el patrón no llega nunca.

    Medido antes de cambiarlo: **0 funciones vigiladas de 52**. La comprobación
    que existe para cazar el patrón del §9 en el tramo store → pantalla llevaba
    desde la fase W afirmándolo y **no miraba ni una**.

    Por eso el H-06 pasó en verde: `guardarVentana` envolvía un comando
    probado, ninguna pantalla la llamaba, y lo encontró Dirección al quedarse
    sin horario de envío en el paso 5 del asistente.

    Ahora el texto se **trocea por `async function`** en vez de emparejar
    llaves con una expresión regular, que es lo que no se puede hacer con una.

    Y el uso se busca como `.nombre` y no como `nombre`: una función llamada
    `cargar` aparece en cualquier pantalla por otros motivos. Se acepta sin
    paréntesis a propósito —`@click="importacion.elegirArchivo"` pasa el
    manejador por referencia—, que fue el primer falso positivo al escribirlo.
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
    # Sin comentarios: nombrar una función al explicar algo no es llamarla. Es
    # la lección que este proyecto lleva nueve veces aprendiendo.
    pantallas = re.sub(r"/\*.*?\*/|//[^\n]*|<!--.*?-->", "", pantallas, flags=re.S)

    vigiladas = 0
    if os.path.isdir(dir_stores):
        for f in sorted(os.listdir(dir_stores)):
            if not f.endswith(".ts") or f.endswith(".spec.ts"):
                continue
            texto = leer("app/src/app/stores", f) or ""
            # Troceado, no emparejado de llaves: una expresión regular no puede
            # contar llaves, y la que lo intentaba se quedaba en el `try`.
            for trozo in re.split(r"\n  async function ", texto)[1:]:
                nombre = trozo[: trozo.index("(")]
                cuerpo = re.split(r"\n  (?:async )?function ", trozo)[0]
                # Sólo las que cruzan la frontera.
                if "invocar" not in cuerpo:
                    continue
                vigiladas += 1
                # Una función que solo usan otras del mismo store no es un hueco.
                usos_internos = len(re.findall(rf"\b{nombre}\b", texto))
                if f".{nombre}" not in pantallas and usos_internos <= 2:
                    sueltas.append(f"{f}: {nombre}() no se llama desde ninguna pantalla")

    # Que el troceado siga encontrando funciones. Sin esto, el día que los
    # stores cambien de forma —otra sangría, otra palabra— esta comprobación
    # volvería a vigilar cero y seguiría saliendo en verde, que es exactamente
    # lo que pasó hasta hoy. El umbral está medido: hay 52.
    if vigiladas < 40:
        sueltas.append(
            f"sólo se leyeron {vigiladas} funciones de store: el troceado dejó de encontrarlas"
        )

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
        # Etiquetas y campos propios: §40, declarado fuera del alcance de la
        # Fase 6 en su documento de cierre. Enseñar un filtro que no filtra
        # sería peor que no tenerlo.
        #
        # `contact_list` y `contact_list_member` SALIERON DE AQUÍ el
        # 23/10/2026: D-15 les dio escritor —la importación— y lector —la
        # audiencia de una campaña—. Llevaban desde la V1 en el esquema sin que
        # nadie las tocara, que es por lo que esta migración fue una columna y
        # no dos tablas.
        "tag",
        "contact_tag",
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
    prefijos: dict[str, set[str]] = {}
    for archivo in sorted(os.listdir(comp)):
        if not archivo.endswith(".vue"):
            continue
        fuente = open(os.path.join(comp, archivo), encoding="utf-8").read()
        declarados[archivo[:-4]] = set(_re.findall(r'<slot\s+name="([\w-]+)"', fuente))
        # ── Y LOS SLOTS CON NOMBRE CALCULADO ──────────────────────────────
        #
        # `ATabla` sirve uno por columna: `:name="`celda-${columna.id}`"`. El
        # nombre entero no se puede saber leyendo el componente —depende de
        # quién lo use— pero **el prefijo sí**, y es lo que hace falta para
        # distinguir un `#celda-acciones` legítimo de un `#pie` inventado.
        #
        # Se guarda el prefijo con su guion, así que `celda-` acepta
        # `#celda-loquesea` y **sigue rechazando** cualquier otro nombre. Sin
        # esto, un slot calculado saldría siempre como error y la salida
        # tentadora sería apagar la comprobación entera.
        prefijos[archivo[:-4]] = set(
            _re.findall(r'<slot\s+:name="`([\w-]+-)\$\{', fuente)
        )

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
                usado = m.group(1)
                vale = usado in declarados[componente] or any(
                    usado.startswith(pre) for pre in prefijos.get(componente, set())
                )
                if not vale:
                    sirve = sorted(declarados[componente]) + [
                        f"{pre}…" for pre in sorted(prefijos.get(componente, set()))
                    ]
                    malos.append(
                        f"{archivo}: `#{usado}` sobre <{componente}>, que sirve "
                        f"{sirve or 'ningún slot con nombre'}"
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


def las_tres_partes_de_un_error_no_salen_pegadas():
    """El slot de `AAviso` separa a sus hijos, o el §95 se lee como un bloque.

    ── EL FALLO QUE ESTO CAZA ──────────────────────────────────────────────

    Lo encontró Dirección el 14/10/2026 al fallar una conexión SMTP (H-02). Las
    pantallas escriben las tres partes del §95 como tres elementos:

        <AAviso><strong>{{ que }}</strong><span>{{ como }}</span>
                <span>{{ salvo }}</span></AAviso>

    `<strong>` y `<span>` son elementos **en línea**. Sin un contenedor que los
    separe el navegador los pega sin un espacio, y se lee:

        «No se pudo cifrar la conexión con el servidor.El servidor no ofrece
         cifrado…»

    El texto era correcto. Lo que fallaba era la maquetación, y daba impresión
    de producto sin terminar **justo en el momento en que algo ha fallado**, que
    es cuando más se lee.

    Eran cinco avisos en tres pantallas, y el arreglo va en el componente
    compartido para que el siguiente lo herede sin acordarse de nada.

    ── POR QUÉ NO LO MIDE UNA SONDA, QUE SERÍA LO FIEL ─────────────────────

    Se intentó. `auditoria.mjs` recorre las dieciocho pantallas en Chromium y
    mide píxeles, así que parecía su sitio. **Con el defecto puesto a propósito,
    la sonda no encontró nada** — y la razón es que en la demostración **los
    avisos de error no aparecen**: el simulador no falla, y el catálogo del
    design system está fuera de las rutas que se recorren.

    Una medida que no se puede ver fallar no vale, así que se retiró de la sonda
    en vez de dejarla dando falsa seguridad. Esto inspecciona el CSS, que es
    menos fiel que medir, pero **se puede romper y verlo en rojo** — y eso es lo
    que decide si una comprobación sirve (§2).

    Verificada el 14/10/2026 quitando el `flex-direction: column` de `.texto`:
    en rojo. Y quitando sólo el `gap`: en rojo también, porque sin separación
    dos párrafos seguidos siguen leyéndose como uno.
    """
    aviso = leer(os.path.join(RAIZ, "app/src/design/componentes/AAviso.vue"))
    # Sin comentarios: la explicación de esta regla cita las propiedades que la
    # regla exige, y sería la undécima vez que una comprobación se caza a sí
    # misma en el texto que la justifica.
    sin_comentarios = re.sub(r"/\*.*?\*/", "", aviso, flags=re.S)

    m = re.search(r"\.texto\s*\{([^}]*)\}", sin_comentarios)
    cuerpo = m.group(1) if m else ""

    check(
        2,
        "las tres partes de un error no salen pegadas",
        bool(m)
        and "flex-direction: column" in cuerpo
        and re.search(r"gap:\s*var\(--arles-space", cuerpo) is not None,
        "Sin separar, `<strong>` y `<span>` se pegan y el §95 se lee como un bloque corrido.",
        f"`.texto` de AAviso: {cuerpo.strip()[:120] or 'no encontrado'}",
    )


def ningun_aviso_sobrevive_a_la_recarga_que_lo_resuelve():
    """Un `cargar()` que escribe un aviso de error tiene que borrarlo al reintentar.

    ── EL FALLO QUE ESTO CAZA ──────────────────────────────────────────────

    Lo encontró Dirección el 14/10/2026 usando ARLES, y es el hallazgo H-01:

      1. Entra en Remitentes **antes** de configurar la empresa. La carga falla
         con `EmpresaNoConfigurada` y la pantalla pinta el aviso. Correcto.
      2. Va a Configuración y guarda los datos. Correcto.
      3. Vuelve a Remitentes. La carga **ahora sí funciona**… y el aviso viejo
         **sigue pintado**: «Todavía no has configurado los datos de tu
         empresa».

    La pantalla afirmaba algo **falso sobre el estado de su propia base**, y le
    hizo dudar de si el guardado había servido. Es el §95 al revés: un error
    que ya no existe sigue dando instrucciones.

    La causa es de una línea — `cargar()` escribe el aviso en su `catch` y nunca
    lo limpia al empezar — y **no era de Remitentes**: lo tenían dos de los diez
    stores, y un tercero lo limpiaba pasada su guarda de núcleo, que es otra
    forma de lo mismo.

    ── POR QUÉ UNA COMPROBACIÓN Y NO SÓLO LA PRUEBA ────────────────────────

    La prueba (`avisos-al-recargar.spec.ts`) cubre los tres stores que existen
    hoy. **Un store nuevo no estaría en ella**, y el patrón es tan fácil de
    repetir que ya se escribió tres veces sin querer. Esto mira *todos* los
    stores que haya, ahora y los que vengan.

    Se mide sobre el cuerpo de `cargar()` **sin sus comentarios**: ésta es la
    décima vez en el proyecto que una comprobación podría cazarse a sí misma en
    la explicación de la regla que vigila.

    Verificada el 14/10/2026 quitando el `errorGeneral.value = null` de
    `remitentes.ts` y de `empresa.ts`: en rojo las dos veces, nombrando el store.
    """
    import glob

    sospechosos = []
    for ruta in sorted(glob.glob(os.path.join(RAIZ, "app/src/app/stores/*.ts"))):
        if ruta.endswith(".spec.ts"):
            continue
        texto = leer(ruta)
        if not texto:
            continue

        # El cuerpo de `cargar()`: desde su firma hasta el cierre de la función.
        m = re.search(r"async function cargar\b[^\n]*\n(.*?)\n  \}", texto, re.S)
        if not m:
            continue
        cuerpo = re.sub(r"//[^\n]*", "", m.group(1))
        cuerpo = re.sub(r"/\*.*?\*/", "", cuerpo, flags=re.S)

        # Sólo importa si el `catch` deja un aviso escrito. Un `catch` vacío
        # —una preferencia de interfaz que no se puede leer no impide usar
        # ARLES— no tiene nada que limpiar, y exigírselo mandaría a arreglar
        # algo que funciona.
        tras_catch = cuerpo.split("catch", 1)
        if len(tras_catch) < 2:
            continue
        escribe = re.search(r"\b(\w*[eE]rror\w*)\.value\s*=\s*(?!null)", tras_catch[1])
        if not escribe:
            continue

        campo = escribe.group(1)
        # Y tiene que borrarlo ANTES del `try`, no después de que todo salga
        # bien: lo que hay que limpiar es el aviso del intento anterior.
        antes = cuerpo.split("try", 1)[0]
        if not re.search(rf"{re.escape(campo)}\.value\s*=\s*null", antes):
            sospechosos.append(f"{os.path.basename(ruta)} ({campo})")

    check(
        7,
        "ningún aviso de error sobrevive a la recarga que lo resuelve",
        not sospechosos,
        "Un error que ya no existe sigue dando instrucciones, y la pantalla miente sobre su propia base (§95 al revés).",
        f"Sin limpiar antes del try: {', '.join(sospechosos)}",
    )


def ningun_store_se_inventa_la_causa_de_un_error():
    """**Un error que no se conoce no se puede diagnosticar inventándole una causa.**

    Dirección no pudo dar de alta un contacto el 16/10/2026 y ARLES contestó
    «Ocurrió un error al acceder a los datos. Cierra ARLES y vuelve a abrirlo»
    (H-05). No se pudo arreglar, y el motivo no fue que el fallo fuera raro:
    fue que **el mensaje se tragó la causa**.

    Nueve stores escribían, para cualquier cosa no tipada que cayera en su
    `catch`:

        errorGeneral.value = esErrorIpc(e) ? e.clave : 'error.db.sqlite'

    Ese `: 'error.db.sqlite'` convierte un `TypeError` del propio JavaScript,
    un fallo al serializar o cualquier excepción de la webview en «falló la
    base de datos». Hace dos cosas malas a la vez:

      1. **Afirma una causa que no conoce.** Manda a arreglar lo que no está
         roto, y a cerrar y reabrir ARLES, que no arregla nada si la causa es
         otra — el mismo defecto que el fallo de migración tenía el 13/10.
      2. **Pierde la de verdad**, así que nadie puede diagnosticarla después.

    Esto exige que el fallback de un error no tipado sea `claveDelError(e)`,
    que devuelve `error.inesperado` y **guarda el detalle técnico**.

    ── LO QUE ESTA COMPROBACIÓN NO CUBRE, Y CONVIENE SABERLO ──

    Mira el **fallback**, no el camino entero. Un store que capture y no
    reporte nada se le escapa; para eso está
    `ningun_aviso_sobrevive_a_la_recarga_que_lo_resuelve`.

    Verificada el 17/10/2026 devolviendo el cajón de sastre a `contactos.ts`:
    falla nombrando el archivo y la clave inventada.
    """
    malos = []
    carpeta = os.path.join(RAIZ, "app", "src", "app", "stores")
    # Las claves que nombran una causa CONCRETA. Usarlas como red de seguridad
    # de un `catch` es afirmar algo que no se sabe. `error.inesperado` no está
    # aquí a propósito: es la que dice la verdad.
    INVENTADAS = ("error.db.sqlite", "error.db.", "error.correo.", "error.whatsapp.")
    for nombre in sorted(n for n in os.listdir(carpeta) if n.endswith(".ts")):
        if nombre.endswith(".spec.ts"):
            continue
        fuente = leer("app", "src", "app", "stores", nombre) or ""
        # Se quitan los comentarios antes de mirar: varios stores EXPLICAN en
        # los suyos qué clave usaban antes, y sin esto la comprobación se
        # cazaría en la explicación. Es la trampa del comentario, que en este
        # proyecto ya ha salido nueve veces.
        codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", fuente, flags=re.S))
        # El patrón busca el fallback de un ternario sobre `esErrorIpc`, y
        # también la rama `if (!esErrorIpc(e))` que hace lo mismo con otra
        # forma — las dos existían en el código del 16/10.
        for m in re.finditer(r"esErrorIpc\([^)]*\)\s*\?[^:]*:\s*'([^']+)'", codigo):
            if any(m.group(1).startswith(p) for p in INVENTADAS):
                malos.append(f"{nombre}: el fallback de un error no tipado es «{m.group(1)}»")
        if re.search(r"!esErrorIpc\([^)]*\)\s*\)\s*\{[^}]*?'error\.db\.", codigo, flags=re.S):
            malos.append(f"{nombre}: una rama `!esErrorIpc` reporta una causa de base de datos")

    check(
        7,
        "ningún store se inventa la causa de un error que no conoce",
        not malos,
        ningun_store_se_inventa_la_causa_de_un_error.__doc__,
        detalle="\n      ".join(malos),
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


def _entradas_del_manifiesto():
    """Las entradas del manifiesto como `(tabla, alcance, porque)`.

    El alcance puede llevar datos dentro —`RecortadaEnElEsencial { fecha: … }`
    desde la 8.2b—, así que el patrón admite la llave opcional. Sin ella, la
    regex dejaba de casar en las dos tablas que se recortan y las comprobaciones
    pasaban en verde **sin mirarlas**, que es el defecto que persiguen.
    """
    manifiesto = leer("crates/arles-core/src/respaldo.rs") or ""
    return re.findall(
        r'tabla:\s*"(\w+)",\s*(?://[^\n]*\n\s*)*'
        r"alcance:\s*Alcance::(\w+)\s*(?:\{[^}]*\})?,\s*"
        r'(?://[^\n]*\n\s*)*porque:\s*(.+?),?\n',
        manifiesto,
    )


def el_manifiesto_se_lee_entero():
    """**La comprobación de las comprobaciones.**

    Las tres del manifiesto leen `respaldo.rs` con expresiones regulares. El día
    que el patrón deje de casar —por una llave, un comentario en medio, un
    campo nuevo— no fallan: **encuentran menos entradas y aprueban las que
    encontraron**. Es el mismo defecto que en la 7.7 llevaba tiempo sin mirar el
    paso de Contactos sin decirlo, y en la 8.2b volvió a pasar: la llave de
    `RecortadaEnElEsencial { fecha: … }` dejó fuera dos tablas.

    Así que se compara el número de entradas leídas contra las tablas que el
    manifiesto declara con el patrón más simple que existe —`tabla: "…"`— y
    contra el esquema. Si las tres cifras no coinciden, el patrón está roto.

    Verificada el 05/10/2026 quitando el `(?:\\{[^}]*\\})?` del patrón: lee 27
    de 29 y falla nombrando las dos que se recortan.
    """
    manifiesto = leer("crates/arles-core/src/respaldo.rs") or ""
    declaradas = re.findall(r'tabla:\s*"(\w+)"', manifiesto)
    entendidas = [t for t, _, _ in _entradas_del_manifiesto()]
    perdidas = sorted(set(declaradas) - set(entendidas))

    check(
        8,
        "el patrón lee el manifiesto entero",
        not perdidas and len(declaradas) == len(entendidas),
        el_manifiesto_se_lee_entero.__doc__,
        detalle=(
            f"declaradas: {len(declaradas)} · entendidas: {len(entendidas)}\n"
            f"      el patrón no caza: {perdidas}"
        ),
    )


def lo_que_se_recorta_dice_por_que_columna():
    """**D-8B necesita saber por qué fecha se corta, y son dos distintas.**

    `message_attempt` guarda `created_at`; `whatsapp_reply` guarda
    `first_reply_at` y **no tiene `created_at`**. Dar por supuesto el nombre
    habría hecho que el recorte de las respuestas fallara en el `WHERE`, delante
    de quien pulsó exportar.

    Por eso la columna viaja dentro de la variante del alcance —el compilador la
    exige justo donde hace falta— y aquí se comprueba lo que el compilador no
    puede: que sea una frase y no una cadena vacía. Que **exista en el esquema**
    lo comprueba `toda_columna_de_corte_existe_de_verdad` en `arles-db`, contra
    una base migrada de verdad en vez de contra una regex.
    """
    manifiesto = leer("crates/arles-core/src/respaldo.rs") or ""
    cortes = re.findall(
        r'tabla:\s*"(\w+)",\s*(?://[^\n]*\n\s*)*'
        r'alcance:\s*Alcance::RecortadaEnElEsencial\s*\{\s*fecha:\s*"(\w*)"',
        manifiesto,
    )
    sin_columna = sorted(t for t, c in cortes if not c)

    check(
        8,
        "lo que se recorta dice por qué columna",
        len(cortes) == 2 and not sin_columna,
        lo_que_se_recorta_dice_por_que_columna.__doc__,
        detalle=f"recortadas: {cortes} · sin columna: {sin_columna}",
    )


def el_respaldo_no_se_lee_en_memoria():
    """**El exportador entrega las filas, no las devuelve. Y no pagina.**

    Dos regresiones concretas, las dos de las que «arreglan» algo:

    1. **Devolver las filas.** Un `-> Result<Vec<Fila>>` es lo cómodo y mete
       medio millón de filas en memoria: justo lo que el presupuesto de 500 MB
       no admite. Por eso hay un rasgo `Destino` con un método por fila.
    2. **Paginar con `LIMIT … OFFSET n`.** Parece la forma de leer por tandas y
       es cuadrática: SQLite no guarda un atajo a la fila n, así que recorre y
       descarta n filas en cada tanda. Y entre dos tandas la tabla puede
       cambiar, con lo que una fila se repite o se salta **sin que nada falle**.

    Lo que esta comprobación mira es el código. Cuánta memoria gasta de verdad
    se mide en la 8.5, con 500 000 filas sintéticas, y esto no lo sustituye: una
    comprobación de texto no pesa un proceso.

    Verificada el 05/10/2026 metiendo un `OFFSET` en la consulta del cuerpo:
    falla nombrando la línea.
    """
    fuente = leer("crates/arles-db/src/respaldo.rs") or ""
    assert fuente, "no existe crates/arles-db/src/respaldo.rs"

    # Sin comentarios: las cabeceras de ese módulo explican por qué NO se usa
    # `OFFSET`, y buscarlo en el texto entero se cazaría a sí misma — que es el
    # defecto del 29/09/2026, una comprobación que se encontraba en un
    # comentario y pasaba siempre.
    codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", fuente, flags=re.S))

    pagina = [
        l.strip() for l in codigo.splitlines() if re.search(r"\bOFFSET\b", l, re.I)
    ]
    devuelve = re.search(r"fn exportar[^\n]*->\s*Result<\s*Vec<", codigo)
    hay_rasgo = re.search(r"pub trait Destino", codigo) and re.search(
        r"fn fila\(", codigo
    )

    check(
        8,
        "el exportador entrega las filas y no pagina",
        not pagina and not devuelve and hay_rasgo,
        el_respaldo_no_se_lee_en_memoria.__doc__,
        detalle=(
            f"paginación con OFFSET: {pagina}\n"
            f"      devuelve las filas: {bool(devuelve)}\n"
            f"      rasgo Destino con un método por fila: {bool(hay_rasgo)}"
        ),
    )


def el_exportador_tiene_quien_lo_llame():
    """**El patrón del §9, vigilado antes de abrir el hueco y no después.**

    La 8.2b dejó `Db::exportar` construido, con trece pruebas, y **nadie lo
    llamaba**. Es exactamente lo que ha pasado trece veces en este proyecto:
    una pieza correcta, probada, que ningún otro módulo usa. Una prueba
    comprueba que la pieza funciona, no que alguien la llame.

    Así que esta comprobación se escribió **antes** de enchufarlo, y se vio
    fallar nombrando `Db::exportar` el 05/10/2026 — con las diez fases en verde
    y el exportador entero ya empujado.

    Mira el cable completo, que es lo que no miraba ninguna de las tres
    comprobaciones que ya perseguían este patrón:

        arles-db::exportar → arles-app → generate_handler! → app/src

    El último tramo —comando registrado y alguien que lo llame— ya lo vigila
    `ningun_comando_se_queda_sin_quien_lo_llame` en la fase W. Aquí falta el
    primero.
    """
    faltan = []

    app = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "crates", "arles-app", "src")):
        for f in ficheros:
            if f.endswith(".rs"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    app += fh.read()
    assert app, "no se leyó el código de arles-app"

    if ".exportar(" not in app:
        faltan.append("arles_db::Db::exportar — nadie en arles-app lo llama")

    # Y el comando tiene que estar registrado, o no hay frontera que cruzar.
    lib = leer("crates", "arles-app", "src", "lib.rs") or ""
    if "exportar_respaldo" not in lib:
        faltan.append("exportar_respaldo — no está en generate_handler!")

    check(
        8,
        "el exportador tiene quien lo llame",
        not faltan,
        el_exportador_tiene_quien_lo_llame.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_restaurador_tiene_quien_lo_llame():
    """**El mismo hueco del §9, en la pieza que más cara cuesta.**

    La 8.3b dejó `Db::restaurar` construido, con ocho pruebas, y **nadie lo
    llamaba** — igual que la 8.2b con `Db::exportar`, igual que el adaptador de
    WhatsApp, igual que el editor de plantillas. Catorce veces ya.

    Aquí el hueco es peor que en los otros trece casos: un respaldo que no se
    puede restaurar **no es un respaldo**. Escribirlo cada semana durante un año
    y descubrir el día del incendio que no hay botón es el fallo completo de la
    fase, con todas sus pruebas en verde.

    Así que esta comprobación se escribió **antes** de enchufar nada, y se vio
    fallar nombrando los tres tramos el 06/10/2026.

    Mira el cable entero, que es lo que ninguna prueba puede mirar:

        arles-db::restaurar → arles-app → generate_handler! → app/src

    Y vigila además que **el preflight se enseñe antes**: restaurar sin decir
    qué se va a perder es justo lo que D-8C y la 8.3a existen para impedir.
    """
    faltan = []

    app = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "crates", "arles-app", "src")):
        for f in ficheros:
            if f.endswith(".rs"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    app += fh.read()
    assert app, "no se leyó el código de arles-app"

    if ".restaurar(" not in app:
        faltan.append("arles_db::Db::restaurar — nadie en arles-app lo llama")
    if ".que_pasaria_al_restaurar(" not in app:
        faltan.append(
            "arles_db::Db::que_pasaria_al_restaurar — el preflight de la 8.3a no se usa"
        )

    lib = leer("crates", "arles-app", "src", "lib.rs") or ""
    for comando in ("revisar_respaldo", "restaurar_respaldo"):
        if comando not in lib:
            faltan.append(f"{comando} — no está en generate_handler!")

    # Y alguien en la interfaz que lo pulse.
    frontend = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "app", "src")):
        for f in ficheros:
            if f.endswith((".ts", ".vue")):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    frontend += fh.read()
    for comando in ("revisar_respaldo", "restaurar_respaldo"):
        if comando not in frontend:
            faltan.append(f"{comando} — ninguna pantalla lo llama")

    check(
        8,
        "el restaurador tiene quien lo llame",
        not faltan,
        el_restaurador_tiene_quien_lo_llame.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_archivado_tiene_quien_lo_llame():
    """**El §9 en la pieza destructiva, que es donde más duele.**

    La 8.3 bis a dejó `que_pasaria_al_archivar` y
    `borrar_el_historial_archivado` construidos, con ocho pruebas, y **nadie los
    llamaba**. Es la decimoquinta vez que este patrón aparece en el proyecto.

    Pero aquí hay una vuelta que no tenían los catorce anteriores: una de las
    dos piezas **borra datos**. Que falte el botón no es sólo una función que no
    existe — es que D-8D, la decisión que Dirección tomó para no saturar el
    disco, no sirve de nada, y el historial sigue creciendo sin techo mientras
    todo está en verde.

    Y al revés: enchufar **sólo el borrado** sin el archivo sería peor que no
    tener nada. Así que esta comprobación vigila **las cinco cosas en orden**,
    no sólo que existan:

        preflight → escritura del archivo → RELECTURA que lo verifica
        → borrado → bitácora

    La relectura es la segunda condición de D-8D y la que no se puede dar por
    hecha: escribir un archivo y borrar sin volver a abrirlo es exactamente el
    caso que convierte «archivar» en «borrar».

    Escrita **antes** de enchufar nada y vista fallar nombrando los siete
    tramos el 06/10/2026.

    ── Y UN DEFECTO DE ESTA COMPROBACIÓN, EL CUARTO DEL DÍA ──

    La primera versión medía el orden comparando **dónde aparece cada nombre en
    el archivo**, y salió en rojo sobre código correcto: la relectura vive en
    `comprobar_el_archivo`, escrita después de `archivar_historial` en el texto
    aunque se ejecute antes. **El orden del archivo no es el orden de
    ejecución.** Ahora se mide dentro del cuerpo de la función, que es donde esa
    pregunta tiene respuesta.
    """
    faltan = []

    app = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "crates", "arles-app", "src")):
        for f in ficheros:
            if f.endswith(".rs"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    app += fh.read()
    assert app, "no se leyó el código de arles-app"
    # Sin comentarios: los tramos se explican en comentarios que los nombran, y
    # buscarlos en el archivo entero se cazaría ahí. Mismo defecto que arriba.
    app_sin_comentarios = re.sub(r"//[^\n]*", "", app)

    if ".que_pasaria_al_archivar(" not in app_sin_comentarios:
        faltan.append("Db::que_pasaria_al_archivar — nadie en arles-app lo llama")
    if ".borrar_el_historial_archivado(" not in app_sin_comentarios:
        faltan.append("Db::borrar_el_historial_archivado — nadie en arles-app lo llama")
    if "ElHistorialAnteriorA" not in app_sin_comentarios:
        faltan.append(
            "QueSeLleva::ElHistorialAnteriorA — nadie escribe el archivo, así que "
            "el borrado no tendría de dónde venir"
        )

    # ── LA RELECTURA, QUE ES LA CONDICIÓN 2 DE D-8D ──
    #
    # El archivo se vuelve a abrir con el mismo lector que usa la restauración,
    # que es quien comprueba la suma. Si esto falta, se borra sin verificar.
    archivo_de_la_app = leer("crates", "arles-app", "src", "archivo.rs")
    if archivo_de_la_app is None:
        faltan.append("no existe crates/arles-app/src/archivo.rs")
    else:
        sin_comentarios = re.sub(r"//[^\n]*", "", archivo_de_la_app)
        if "LeeElArchivo" not in sin_comentarios:
            faltan.append(
                "el archivo no se vuelve a leer antes de borrar (condición 2 de D-8D)"
            )
        # ── EL ORDEN, MEDIDO DENTRO DE LA FUNCIÓN Y NO EN EL ARCHIVO ──
        #
        # La primera versión comparaba dónde aparece cada nombre en el archivo
        # entero, y salió en **rojo sobre código correcto**: la relectura vive
        # en `comprobar_el_archivo`, que está escrita DESPUÉS de
        # `archivar_historial` en el texto aunque se ejecute antes. El orden
        # del archivo no es el orden de ejecución.
        #
        # Lo que sí se puede leer es el cuerpo de `archivar_historial`: dentro
        # de él, la llamada que verifica tiene que estar antes de la que borra.
        cuerpo = re.search(
            r"pub fn archivar_historial\b.*?\n\}", sin_comentarios, re.S
        )
        if not cuerpo:
            faltan.append("no se encuentra el cuerpo de archivar_historial")
        else:
            texto = cuerpo.group(0)
            pos_verifica = texto.find("comprobar_el_archivo")
            pos_borra = texto.find("borrar_el_historial_archivado")
            if pos_verifica == -1:
                faltan.append("archivar_historial no verifica el archivo que escribió")
            elif pos_borra == -1:
                faltan.append("archivar_historial no borra: no archiva, sólo escribe")
            elif pos_verifica > pos_borra:
                faltan.append(
                    "se borra antes de verificar el archivo: el orden de D-8D es al revés"
                )

    lib = leer("crates", "arles-app", "src", "lib.rs") or ""
    for comando in ("revisar_archivado", "archivar_historial"):
        if comando not in lib:
            faltan.append(f"{comando} — no está en generate_handler!")

    frontend = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "app", "src")):
        for f in ficheros:
            if f.endswith((".ts", ".vue")):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    frontend += fh.read()
    for comando in ("revisar_archivado", "archivar_historial"):
        if comando not in frontend:
            faltan.append(f"{comando} — ninguna pantalla lo llama")

    check(
        8,
        "el archivado tiene quien lo llame, y en su orden",
        not faltan,
        el_archivado_tiene_quien_lo_llame.__doc__,
        detalle="\n      ".join(faltan),
    )


def cambiar_una_migracion_recompila():
    """**Sin esto, editar una migración no prueba nada.**

    `refinery::embed_migrations!` es una macro: lee los `.sql` y los incrusta en
    el binario al compilar. Cargo **no sabe** que esos archivos son una
    dependencia, así que al cambiar uno no recompila — y `cargo test` ejecuta
    el SQL anterior con todo en verde.

    Se descubrió el 06/10/2026 intentando verificar la V11 rompiéndola (§2): se
    quitó un índice, la prueba que lo exige **siguió pasando**, y no era que la
    prueba fuera mala: el índice quitado nunca llegó al binario.

    Es la peor clase de trampa, porque convierte el §2 en teatro: se rompe algo,
    se ve pasar la prueba, y se concluye que la comprobación no sirve — cuando
    lo que no sirve es el experimento.

    Lo cierra un `build.rs` con `rerun-if-changed` sobre la carpeta entera.
    """
    faltan = []

    build = leer("crates", "arles-db", "build.rs") or ""
    if not build:
        faltan.append("falta crates/arles-db/build.rs")
    elif "rerun-if-changed=migrations" not in build:
        faltan.append("build.rs no vigila la carpeta migrations")

    # Y que la prueba de los planes exista: es la que convierte un índice en una
    # garantía y no en una buena intención.
    planes = leer("crates", "arles-db", "tests", "planes_de_consulta.rs") or ""
    if "EXPLAIN QUERY PLAN" not in planes:
        faltan.append("nadie comprueba el plan de las consultas críticas")

    check(
        8,
        "cambiar una migración recompila, y los planes se vigilan",
        not faltan,
        cambiar_una_migracion_recompila.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_banco_no_viaja_en_el_instalador():
    """**El banco de pruebas mide el producto; no puede ser parte de él.**

    Uno de los trece presupuestos es el **tamaño del instalador** (< 20 MB), y
    un generador de medio millón de contactos dentro de `arles-app` lo
    engordaría justo para poder medirlo. Es la trampa más tonta posible: el
    instrumento falseando lo que mide.

    Vigila además que las cifras medidas lleven **su fecha**. Un número de
    rendimiento sin fecha no se puede comparar con el siguiente, y con el
    tiempo se lee como si fuera de hoy.

    Verificada el 06/10/2026 añadiendo `arles-banco` a las dependencias de
    `arles-app`: falla nombrándolo.
    """
    faltan = []

    app = leer("crates", "arles-app", "Cargo.toml") or ""
    if "arles-banco" in app:
        faltan.append("arles-app depende de arles-banco: viajaría en el instalador")

    # El generador tiene que ser reproducible, que es lo que permite repetir una
    # medición. Si alguien siembra con el reloj, deja de serlo.
    banco = leer("crates", "arles-banco", "src", "lib.rs") or ""
    if not banco:
        faltan.append("no existe crates/arles-banco/src/lib.rs")
    else:
        # ── SE QUITAN LOS COMENTARIOS, Y NO ES UN DETALLE ──
        #
        # Escrita sobre el texto crudo, esta comprobación salió en rojo el
        # 06/10/2026 señalando un generador perfectamente reproducible: lo que
        # casaba era **el comentario de la prueba**, que cuenta que se verificó
        # sembrando con `SystemTime::now()`. Es la misma trampa que la 5.3 ya
        # documentó —«una comprobación que se cazaba a sí misma en un
        # comentario»— y la que hace que una comprobación mienta en los dos
        # sentidos: aquí dio un falso rojo, y el día que alguien borre ese
        # comentario daría un falso verde sobre el mismo código.
        codigo_banco = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", banco, flags=re.S))
        if "SystemTime" in codigo_banco or "Instant::now" in codigo_banco:
            faltan.append("el generador usa el reloj: deja de ser reproducible")

    # Y las medidas, con su fecha.
    presupuesto = leer("documentacion", "06-calidad", "PRESUPUESTO_RENDIMIENTO.md") or ""
    # Sin distinguir mayúsculas: lo que importa es que el documento separe lo
    # medido de lo que no, no cómo lo escriba. Escrita primero exigiendo
    # «SIN MEDIR» en mayúsculas, y corregida el 06/10/2026 al verla señalar un
    # documento que decía exactamente eso en minúsculas.
    if "sin medir" not in presupuesto.lower():
        faltan.append(
            "el presupuesto no distingue lo medido de lo que no: «sin medir» no es «cumple»"
        )
    # ── EL PATRÓN ESTABA MAL, Y LA PRUEBA DE LA PRUEBA LO CAZÓ ──
    #
    # Se escribió `\b0[1-9]|[12]\d|3[01]/\d{2}/20\d{2}\b`, y la alternancia
    # ata antes que la concatenación: casaba «20» en cualquier parte del
    # documento. Daba por fechado un texto sin una sola fecha — la comprobación
    # que no puede fallar, otra vez (§2). Los paréntesis son el arreglo.
    if not re.search(r"\b(?:0[1-9]|[12]\d|3[01])/\d{2}/20\d{2}\b", presupuesto):
        faltan.append("las cifras medidas no llevan fecha")

    check(
        8,
        "el banco de pruebas no viaja en el instalador, y lo medido lleva fecha",
        not faltan,
        el_banco_no_viaja_en_el_instalador.__doc__,
        detalle="\n      ".join(faltan),
    )


def la_lista_de_la_campana_llega_desde_el_archivo_hasta_el_envio():
    """**El cable entero de D-15, de punta a punta.**

    ─────────────────────────────────────────────────────────────────────────
    QUÉ SE ESTÁ VIGILANDO, Y POR QUÉ NO BASTA CON UNA PRUEBA

    Que una campaña vaya sólo a su público atraviesa **cinco piezas** en tres
    capas: el archivo que se carga, la lista que se crea, la columna de la
    campaña, la consulta de la audiencia y el desplegable que la elige. Cada
    una tiene sus pruebas y todas pueden pasar con el cable suelto en medio —
    es el patrón que este proyecto lleva veintitrés veces, y la última, dos
    días antes de ésta, fue `Db::suprimir_direccion`: probada desde la Fase 4
    y sin ningún comando.

    Aquí lo que se perdería no es una función: es que una campaña de
    videovigilancia salga igualmente a todo el padrón, que es exactamente el
    riesgo que D-15 viene a quitar y lo que más rápido quema un dominio.

    ─────────────────────────────────────────────────────────────────────────
    LOS CINCO TRAMOS

      1. `importar_contactos` escribe en la lista.
      2. `crear_campana` guarda `contact_list_id`.
      3. `activar_campana` filtra la audiencia por ella.
      4. El comando que enseña las listas está registrado.
      5. **Alguien en la pantalla lo llama**, y el asistente manda la lista.

    El quinto es el único que el usuario pisa, y el que ninguna prueba de Rust
    alcanza.

    Escrita **antes** de la pantalla y vista en rojo nombrando sus tramos el
    23/10/2026.
    """
    faltan = []

    def solo_el_codigo(t):
        """Sin comentarios **y sin las pruebas**.

        ── LOS DOS DEFECTOS QUE ESTA FUNCIÓN ARREGLA, VISTOS EL 23/10/2026 ──

        La primera versión leía el archivo entero y **dos de las cinco
        rupturas salieron en verde**:

        1. Romper el filtro de la audiencia no la cazaba, porque las pruebas
           del mismo módulo siembran `INSERT INTO contact_list_member`. La
           cadena seguía estando, en código que no es el que corre en
           producción. Es «la hoja y no la ruta» con otra cara, y la enésima
           vez que una comprobación se caza a sí misma — aquí no en un
           comentario, sino en una siembra.
        2. Romper el `INSERT` de la campaña tampoco, y ése fue **error de la
           ruptura, no de la comprobación**: `contact_list_id` aparece también
           en el `SELECT` que lee la lista al activar, así que cambiar la
           primera aparición dejaba la segunda en pie. *Una ruptura que deja el
           síntoma en pie no prueba nada.*

        Se corta por `mod tests`, igual que `ninguna_tabla_se_queda_sin_escritor`.
        """
        t = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", t, flags=re.S))
        corte = t.find("mod tests")
        return t[:corte] if corte != -1 else t

    # ── 1 · la importación escribe en la lista ──
    cont = solo_el_codigo(leer("crates", "arles-db", "src", "contactos.rs") or "")
    if "meter_en_la_lista" not in cont:
        faltan.append(
            "importar_contactos no mete a nadie en la lista: el archivo se carga "
            "y la lista sale vacía"
        )

    # ── 2 y 3 · la campaña la guarda y la audiencia la respeta ──
    alta = solo_el_codigo(leer("crates", "arles-db", "src", "alta_de_campana.rs") or "")
    # Por el INSERT y no por el nombre de la columna a secas: `contact_list_id`
    # aparece también en el SELECT que la lee al activar, así que buscarlo
    # suelto daría por escrita una columna que sólo se lee.
    if "INSERT INTO campaign (" not in alta or "contact_list_id" not in alta.split(
        "INSERT INTO campaign ("
    )[1][:400]:
        faltan.append(
            "crear_campana no guarda `contact_list_id`: la lista elegida no se "
            "escribiría en ninguna parte"
        )
    if "contact_list_member" not in alta:
        faltan.append(
            "activar_campana no filtra la audiencia por la lista: toda campaña "
            "seguiría yendo a todo el padrón"
        )

    # ── 4 · el comando registrado ──
    lib = solo_el_codigo(leer("crates", "arles-app", "src", "lib.rs") or "")
    if "listas_de_contactos," not in lib:
        faltan.append(
            "`listas_de_contactos` no está en generate_handler!: la pantalla no "
            "puede saber qué listas hay"
        )

    # ── 5 · el tramo que el usuario pisa ──
    #
    # Sin comentarios otra vez: las pantallas explican en su cabecera para qué
    # sirve la lista, nombrando el comando. Es la trampa de siempre.
    front = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "app", "src")):
        for f in ficheros:
            if f.endswith((".ts", ".vue")) and not f.endswith(".spec.ts"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    front += fh.read()
    assert front, "no se leyó el código de app/src"
    front = sin_comentarios_de_barras(front)

    if "'listas_de_contactos'" not in front:
        faltan.append(
            "ninguna pantalla pide las listas: el desplegable de «a quién va "
            "dirigida» no existiría"
        )
    if "crear_campana" in front and "lista" not in front:
        faltan.append("el asistente no manda la lista al crear la campaña")

    check(
        6,
        "la lista de la campaña llega desde el archivo hasta el envío",
        not faltan,
        la_lista_de_la_campana_llega_desde_el_archivo_hasta_el_envio.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_umbral_de_rebotes_frena_de_verdad():
    """**El cable entero de R-3: del rebote al freno y a la pantalla.**

    ─────────────────────────────────────────────────────────────────────────
    QUÉ SE PIERDE SI UNO DE LOS CINCO TRAMOS SE SUELTA

      1. La **regla** vive en el núcleo (`rebotes.rs`): dos escalones y la
         muestra mínima. Sin ella no hay nada que obedecer.
      2. La **cuenta** vive en `arles-db`: cuántos rebotes duros lleva esta
         campaña y de cuántos envíos ya resueltos. Sin ella la regla no tiene
         con qué decidir.
      3. El **motor** las junta y **pausa**. Éste es el tramo que no caza
         nadie más: los dos de abajo pasan sus pruebas mientras la campaña
         sigue quemando el dominio, y nada falla. Es exactamente el estado en
         el que R-3 llevaba abierto desde la Fase 4: «ARLES los registra, no
         los frena».
      4. La **pantalla** lo enseña: una campaña pausada sin decir por qué se
         lee como una avería.
      5. Y los **textos**, o se lee `rebotes.frena` en crudo.

    Escrita **antes** de construir los tramos 2 a 5 y vista en rojo nombrando
    los cuatro que faltaban.
    """
    faltan = []

    nucleo = sin_comentarios_de_barras(
        leer("crates", "arles-core", "src", "rebotes.rs") or ""
    )
    # Con su paréntesis: sin él, renombrar la función a `mirar_XX` seguiría
    # casando. Es el defecto que la comprobación de la 7.5 tuvo el 26/10/2026.
    if "pub const fn mirar(" not in nucleo:
        faltan.append("arles-core no tiene la regla: no habría nada que obedecer")
    if "MUESTRA_MINIMA" not in nucleo:
        faltan.append(
            "no hay muestra mínima: con diez envíos un rebote es el 10 % y la "
            "campaña pararía por una dirección mal escrita"
        )

    # ── TODOS LOS NOMBRES VAN CON SU PARÉNTESIS ──
    #
    # Sin él, `rebotes_duros_de` sigue casando dentro de
    # `rebotes_duros_de_XX`, y renombrar la función —la forma natural de
    # romperla— pasa en verde.
    #
    # Es el MISMO defecto que la comprobación de la 7.5 tuvo el 26/10/2026, y
    # se repitió aquí unas horas después de arreglarlo allí: la lección se
    # escribió en un comentario de otro archivo y no sirvió de nada. Tres de
    # las siete rupturas de R-3 salieron en verde por esto.
    db = sin_comentarios_de_barras(leer("crates", "arles-db", "src", "cifras.rs") or "")
    if "rebotes_duros_de(" not in db:
        faltan.append(
            "arles-db no cuenta los rebotes duros de la campaña: la regla no "
            "tendría con qué decidir"
        )

    # ── El tramo que no caza nadie más ──
    motor = sin_comentarios_de_barras(
        leer("crates", "arles-app", "src", "motor.rs") or ""
    )
    if "rebotes::mirar(" not in motor:
        faltan.append(
            "el motor no mira los rebotes antes de seguir: las piezas de abajo "
            "pasarían sus pruebas y la campaña seguiría quemando el dominio"
        )
    if "hay_que_frenar()" not in motor:
        faltan.append(
            "el motor no pausa la campaña: contaría los rebotes y no haría "
            "nada con la cuenta, que es R-3 tal como estaba"
        )

    pantalla = leer("app", "src", "app", "pantallas", "PantallaActividad.vue") or ""
    if "rebotes" not in pantalla:
        faltan.append(
            "Actividad no enseña los rebotes: una campaña pausada sin decir "
            "por qué se lee como una avería (§95)"
        )

    catalogo = leer("app", "src", "app", "locales", "es.ts") or ""
    if "rebotes: {" not in catalogo:
        faltan.append("`rebotes.*` no tiene textos: se leería la clave en crudo")

    check(
        7,
        "el umbral de rebotes frena de verdad (R-3)",
        not faltan,
        "Contar los rebotes y no frenar es el estado en el que R-3 llevaba "
        "abierto desde la Fase 4. El precio lo paga el dominio, durante meses.",
        "\n  ".join(faltan),
    )


def los_registros_del_dominio_llegan_hasta_el_preflight():
    """**El cable entero de la 7.5: del DNS al punto 8 del preflight.**

    ─────────────────────────────────────────────────────────────────────────
    QUÉ SE PIERDE SI UNO DE LOS CINCO TRAMOS SE SUELTA

      1. La **lectura** vive en el núcleo (`entregabilidad.rs`): qué dice un
         SPF, un DKIM y un DMARC. Sin ella no hay nada que interpretar.
      2. La **consulta** vive en `arles-email/dns.rs`: ir a buscarlos. Sin
         ella, el núcleo sabe leer algo que nadie trae.
      3. **`arles-app` las junta** y contesta `registros_del_dominio`. Éste es
         el tramo que no caza nadie más: los dos de abajo pasan todas sus
         pruebas mientras el punto 8 sigue cableado a `None` y el preflight
         enseña «sin comprobar» para siempre. **Estuvo así desde la Fase 6**,
         con el comentario «P-16: ARLES no consulta DNS» al lado.
      4. La **pantalla** lo enseña: sin eso, se consulta el DNS y el resultado
         no se ve en ninguna parte.
      5. Y el **texto**: sin él se lee `entregabilidad.falta` en crudo.

    Es el patrón del §9, que este proyecto lleva más de veinte veces
    encontrando: cada pieza con sus pruebas y el hueco justo en la junta.

    Escrita **antes** de construir el tramo 3 y vista en rojo nombrando los
    cinco.
    """
    faltan = []

    # ── Los nombres se buscan CON SU PARÉNTESIS ──
    #
    # Sin él, `pub fn revisar_spf` sigue casando dentro de
    # `pub fn revisar_spf_XX`, así que renombrar la función —que es la forma
    # natural de romperla— pasaba en verde. Verificado el 26/10/2026: la
    # primera versión de esta comprobación no cazó esa ruptura. Es el defecto
    # de la subcadena, que en este proyecto lleva más de una docena de
    # apariciones.
    nucleo = sin_comentarios_de_barras(
        leer("crates", "arles-core", "src", "entregabilidad.rs") or ""
    )
    if "pub fn revisar_spf(" not in nucleo or "pub fn revisar_dmarc(" not in nucleo:
        faltan.append(
            "arles-core no sabe leer SPF/DMARC: no habría nada que interpretar"
        )

    adaptador = sin_comentarios_de_barras(
        leer("crates", "arles-email", "src", "dns.rs") or ""
    )
    if "pub fn revisar_dominio(" not in adaptador:
        faltan.append(
            "arles-email no consulta el DNS: el núcleo sabría leer algo que "
            "nadie trae"
        )
    # La regla que más caro sale equivocar, y que ya falló una vez de verdad:
    # una negativa sin firmar NO puede dar un registro por ausente.
    if "fn negativa" not in adaptador:
        faltan.append(
            "no se distingue una negativa firmada de un DNS filtrado: ARLES "
            "diría «te faltan los tres registros» a un dominio bien "
            "configurado (medido el 26/10/2026 contra telemetrymx.com)"
        )

    # ── El tramo que no caza nadie más ──
    app = sin_comentarios_de_barras(
        leer("crates", "arles-app", "src", "entregabilidad.rs") or ""
    )
    if "revisar_dominio(" not in app:
        faltan.append(
            "arles-app no junta las dos mitades: las de abajo pasarían sus "
            "pruebas y el punto 8 seguiría diciendo «sin comprobar» para "
            "siempre"
        )
    # Y que el preflight DEJE de estar cableado a None, que es lo que estuvo
    # desde la Fase 6. Se mira el argumento, no el nombre del campo.
    comandos = sin_comentarios_de_barras(
        leer("crates", "arles-app", "src", "comandos.rs") or ""
    )
    if re.search(r"registros_del_dominio:\s*None", comandos):
        faltan.append(
            "el preflight sigue cableado a `registros_del_dominio: None`: se "
            "consultaría el DNS y el punto 8 no se enteraría"
        )

    pantalla = leer("app", "src", "app", "pantallas", "PantallaRemitentes.vue") or ""
    if "entregabilidad" not in pantalla:
        faltan.append(
            "ninguna pantalla enseña los registros: se consultaría el DNS y "
            "el resultado no se vería en ninguna parte"
        )

    catalogo = leer("app", "src", "app", "locales", "es.ts") or ""
    if "entregabilidad: {" not in catalogo:
        faltan.append(
            "`entregabilidad.*` no tiene textos: se leería la clave en crudo"
        )

    check(
        7,
        "los registros del dominio llegan del DNS al preflight (7.5)",
        not faltan,
        "Cada pieza pasa sus pruebas y el hueco vive en la junta: el punto 8 "
        "lleva diciendo «sin comprobar» desde la Fase 6.",
        "\n  ".join(faltan),
    )


def el_descanso_entre_campanas_llega_hasta_la_pantalla():
    """**El cable entero de D-16, de la regla al texto que se lee.**

    ─────────────────────────────────────────────────────────────────────────
    QUÉ SE PIERDE SI UNO DE LOS CUATRO TRAMOS SE SUELTA

      1. La **regla** vive en el núcleo (`descanso.rs`): cuántos días y qué
         hacer. Sin ella no hay nada que obedecer.
      2. La **consulta** vive en `arles-db`: cuándo se le escribió por última
         vez desde otra campaña. Sin ella, la regla no tiene con qué decidir.
      3. El **motor** las junta antes de enviar. Si falta este tramo, los dos
         anteriores pasan sus pruebas y **ARLES escribe dos veces el mismo
         día** — que es justo lo que D-16 viene a impedir, y nada falla.
      4. Y el **texto**: sin él, la pantalla de Actividad enseña la clave
         cruda en la razón por la que la campaña está quieta.

    El tercero es el que no caza nadie más. Es el patrón del §9: cada pieza
    con sus pruebas y el hueco en la junta.

    Escrita después de construirlo y **vista en rojo en los cuatro tramos**,
    uno a uno.
    """
    faltan = []

    nucleo = sin_comentarios_de_barras(
        leer("crates", "arles-core", "src", "descanso.rs") or ""
    )
    if "DESCANSO_ENTRE_CAMPANAS_DIAS" not in nucleo:
        faltan.append(
            "arles-core no declara el descanso: no hay regla que obedecer"
        )

    db = sin_comentarios_de_barras(leer("crates", "arles-db", "src", "cola.rs") or "")
    if "ultimo_envio_de_otra_campana" not in db:
        faltan.append(
            "arles-db no sabe cuándo se le escribió por última vez desde otra "
            "campaña: la regla no tendría con qué decidir"
        )
    if "marcar_en_descanso" not in db:
        faltan.append(
            "no hay forma de devolver el envío a la cola con fecha: esperar "
            "dejaría el envío tomado para siempre"
        )

    motor = sin_comentarios_de_barras(leer("crates", "arles-motor", "src", "bucle.rs") or "")
    # El tramo que nadie más caza: que el BUCLE lo pregunte antes de enviar.
    if "ultimo_envio_de_otra_campana" not in motor:
        faltan.append(
            "el motor no consulta el descanso antes de enviar: las dos piezas "
            "de abajo pasarían sus pruebas y ARLES escribiría dos veces el "
            "mismo día"
        )
    if "marcar_en_descanso" not in motor:
        faltan.append(
            "el motor no aparca el envío: lo dejaría tomado o lo devolvería "
            "sin fecha, y volvería a salir en el acto"
        )

    # Y el texto, que es lo único que el usuario lee.
    catalogo = leer("app", "src", "app", "locales", "es.ts") or ""
    if "enDescanso:" not in catalogo:
        faltan.append(
            "`motor.enDescanso` no tiene texto: Actividad enseñaría la clave "
            "cruda como razón de que la campaña esté quieta"
        )

    check(
        7,
        "el descanso entre campañas llega del núcleo hasta la pantalla",
        not faltan,
        el_descanso_entre_campanas_llega_hasta_la_pantalla.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_desenlace_de_la_simulacion_llega_hasta_la_pantalla():
    """**ARLES ya sabe si una campaña puede terminar, y no lo decía.**

    El núcleo calcula tres desenlaces —`Termina`, `NoAvanza`,
    `NoCabeEnElHorizonte { faltan }`— y los tres textos están escritos en
    `es.ts` desde la Fase 4, incluido el que dice *«la campaña no termina en
    cinco años: {faltan} mensajes se quedarían sin enviar»*. El comentario de
    uno de ellos dice literalmente **«el caso que hay que cazar antes de
    activar»**.

    **No los llamaba nadie.** El paso 8 del asistente pintaba el tope diario y
    los días, y nada más; y `activar_campana` no consultaba la estimación. Con
    451 196 personas a 50 diarios el asistente enseñaba «1 303 días» —una cifra
    correcta— y **nunca decía que 386 046 personas no van a recibir un
    mensaje**, que es lo que el núcleo había calculado y entregado.

    Es el patrón del §9 número diecisiete: el cálculo bien, las variantes bien,
    los textos bien, y el cable sin conectar. Y las comprobaciones que persiguen
    ese patrón no lo cazaban, porque miran **comandos** registrados y quién los
    llama — un campo de una respuesta que la pantalla ignora no es un comando.

    Esto vigila los dos tramos del cable:

    1. La pantalla **usa** las tres claves de `duracion`.
    2. `activar_campana` **consulta** la estimación y rechaza lo imposible.

    Escrita **antes** del arreglo (8.6 E, 07/10/2026) y vista en rojo nombrando
    los dos tramos.
    """
    faltan = []

    # ── Tramo 1 · la pantalla enseña el desenlace ──
    #
    # Se miran las pantallas y los stores, no el catálogo: el catálogo es quien
    # TIENE los textos, y buscarlo ahí sería preguntarle a la pieza por sí
    # misma. Lo que hace falta saber es si alguien los pide.
    # ── Y AQUÍ TAMBIÉN SE QUITAN LOS COMENTARIOS. SÉPTIMA VEZ ──
    #
    # La primera versión de estas líneas miraba el texto crudo, y `termina`
    # **no salía en rojo**: casaba el comentario `/** Cuándo termina una
    # campaña, según el simulador. */` de `campanas.ts`. Dos de los tres
    # tramos se habrían dado por conectados sin estarlo.
    #
    # Es la misma trampa que la 5.3, la 8.5, las dos de la 8.3 bis, la 8.6 A y
    # la 8.6 D — y la segunda **en la misma sesión**, dentro de una
    # comprobación escrita horas después de arreglar la anterior. Saberlo no
    # basta: hay que quitar los comentarios siempre, por defecto.
    usados = ""
    for carpeta, nombre in (("pantallas", "AsistenteDeCampana.vue"),
                            ("stores", "campanas.ts")):
        bruto = leer("app", "src", "app", carpeta, nombre) or ""
        usados += re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", bruto, flags=re.S))
    if not usados:
        faltan.append("no se pudo leer el asistente ni el store de campañas")
    for clave in ("duracion.termina", "duracion.noAvanza",
                  "duracion.noCabeEnElHorizonte"):
        # La clave puede escribirse entera o por su hoja dentro de un `t()`
        # con prefijo. Se aceptan las dos formas y se exige la hoja, que es lo
        # que no puede faltar.
        hoja = clave.split(".")[1]
        if clave not in usados and hoja not in usados:
            faltan.append(
                f"la pantalla no enseña «{clave}»: el núcleo lo calcula y nadie lo dice"
            )

    # ── Tramo 2 · activar rechaza lo que no puede terminar ──
    alta = leer("crates", "arles-db", "src", "alta_de_campana.rs") or ""
    if not alta:
        faltan.append("no existe crates/arles-db/src/alta_de_campana.rs")
    else:
        # Se quitan los comentarios antes de mirar. Este módulo EXPLICA en su
        # cabecera por qué consulta la estimación, nombrando las variantes — y
        # sin esto la comprobación se cazaría en esa explicación. Es la trampa
        # que este proyecto ya ha tenido seis veces.
        codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", alta, flags=re.S))
        if "estimar" not in codigo:
            faltan.append(
                "activar_campana no llama a `estimar`: se puede activar una campaña "
                "que el núcleo ya sabe que no termina"
            )
        if "NoCabeEnElHorizonte" not in codigo:
            faltan.append("activar_campana no rechaza NoCabeEnElHorizonte")
        if "NoAvanza" not in codigo:
            faltan.append("activar_campana no rechaza NoAvanza")

    check(
        8,
        "el desenlace de la simulación llega hasta la pantalla y hasta activar",
        not faltan,
        el_desenlace_de_la_simulacion_llega_hasta_la_pantalla.__doc__,
        detalle="\n      ".join(faltan),
    )


def las_sondas_recorren_todas_las_pantallas():
    """**Una sonda que no recorre una pantalla no dice nada de esa pantalla.**

    La frase está escrita en `auditoria.mjs` desde el 29/09/2026, el día que se
    le añadieron tres rutas que faltaban. **Y volvió a pasar cinco veces**: la
    auditoría D del §140 encontró el 07/10/2026 que las dos sondas de recorrido
    llevaban cada una su lista de trece rutas a mano, y el producto tiene
    dieciocho.

    Registro, Respaldos, Restaurar, Archivar y Recuperar **nunca se habían
    medido**: ni su contraste, ni su foco, ni sus desbordes, ni la jerarquía de
    sus encabezados, ni si eran callejones sin salida. Y las dos sondas
    terminaban en verde.

    La lección se había escrito y no sirvió, porque iba en un comentario y la
    lista seguía siendo a mano. Ahora las rutas salen de `rutas.mjs`, que las
    lee del router, y esto vigila que siga siendo así: **ninguna sonda puede
    volver a llevar su propia lista**.

    Verificada el 07/10/2026 con cuatro rupturas, las cuatro vistas en rojo:

    1. Devolviendo a `auditoria.mjs` una constante `RUTAS = ['#/inicio', …]`:
       falla con «importa rutas.mjs y no lo llama».
    2. Borrando `rutas.mjs`: falla por no existir.
    3. Quitándole el `MINIMO` a `rutas.mjs`.
    4. Quitándole el `throw` a `recorrido.mjs`, que tiene tabla por ruta.

    **La tercera no cazó a la primera**, y es la lección del día: mirando el
    archivo entero, `MINIMO` seguía apareciendo en la cabecera del módulo que
    explica por qué existe. Es la trampa del comentario por **sexta vez** en
    este proyecto, y la primera dentro de una comprobación escrita el mismo día
    para arreglar otra igual. Se quitan los comentarios antes de mirar.

    ─────────────────────────────────────────────────────────────────────────
    Y EL 14/10/2026, EL DEFECTO DE ESTA COMPROBACIÓN DENTRO DE ELLA MISMA

    Decía vigilar «las sondas» y vigilaba **dos nombres escritos a mano**:
    `auditoria.mjs` y `recorrido.mjs`, las que existían el 07/10/2026. Ese día
    se escribió una tercera —`encimado.mjs`, para medir el texto montado que
    Dirección vio— y esta comprobación **no la habría mirado nunca**.

    Es el mismo defecto, dentro de la comprobación que existe para cazarlo. La
    lección volvió a ir en un comentario y la lista volvió a ser a mano.

    Ahora las sondas **salen de la carpeta**: son las que importan `rutas.mjs`.
    Y el hueco que eso deja —una sonda con su lista propia no importa
    `rutas.mjs`, así que no entraría— lo cierra un segundo criterio: ninguna
    sonda puede nombrar más de `MAX_RUTAS_PROPIAS` rutas por su cuenta. El
    umbral está **medido**: las sondas de una pantalla nombran 0 o 1,
    `vista-previa.mjs` nombra 3, y las que recorren el producto nombran 0
    porque las piden. Una lista a mano del producto entero son dieciocho.

    Verificada el 14/10/2026 con tres rupturas más, las tres en rojo:

    5. Quitándole a `encimado.mjs` el `import` de `rutas.mjs` y dándole una
       lista de cuatro rutas: falla nombrando el archivo y la cifra.
    6. Quitándole el `leerRutasConAlmohadilla()` a `encimado.mjs` dejando el
       import: falla con «importa rutas.mjs y no lo llama».
    7. Renombrando `rutas.mjs`: falla por no existir.
    8. Dejando `rutas.mjs` en su sitio y apuntando los tres imports a otro
       nombre: falla con «ninguna sonda importa rutas.mjs», que es la guarda
       que impide que esto pase en verde sin haber mirado ninguna sonda.

    **La séptima ruptura no hizo lo que yo había escrito que haría.** Anoté que
    fallaría «dos veces» —por no existir el módulo y porque entonces nadie lo
    importa—, y es falso: las tres sondas siguen llevando la cadena
    `./rutas.mjs` en su `import`, así que la segunda guarda no se dispara. Hubo
    que romperla aparte, y eso es la octava. Una afirmación de verificación es
    una afirmación como cualquier otra, y en este proyecto son las que más
    veces resultan falsas.
    """
    faltan = []

    rutas_mjs = leer("app", "pruebas", "sondas", "rutas.mjs")
    if not rutas_mjs:
        faltan.append("no existe app/pruebas/sondas/rutas.mjs: las rutas volverían a ser una lista")
    else:
        # ── Y AQUÍ TAMBIÉN SE QUITAN LOS COMENTARIOS, POR LO MISMO ──
        #
        # La primera versión de estas dos líneas miraba el archivo entero, y la
        # ruptura de «quitarle el mínimo» **no la cazó**: `MINIMO` y `router.ts`
        # están nombrados en la cabecera del módulo, que explica por qué
        # existen. Sexta vez en este proyecto, y la primera dentro de una
        # comprobación escrita el mismo día para arreglar otra igual.
        codigo_rutas = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", rutas_mjs, flags=re.S))
        # El módulo tiene que leer del router de verdad, no traer su propia
        # copia. Sin esto, «leer del código» se cumple de nombre.
        if "router.ts" not in codigo_rutas:
            faltan.append("rutas.mjs no lee router.ts: entonces su lista también es a mano")
        # Y tiene que reventar si el patrón deja de casar. Una sonda que
        # recorre cero pantallas pasa en verde en un segundo, y es la trampa
        # del §2 que este proyecto ya ha tenido cinco veces.
        if "MINIMO" not in codigo_rutas:
            faltan.append("rutas.mjs no exige un mínimo: un patrón roto daría cero rutas en verde")

    # ── LAS DOS SONDAS QUE RECORREN EL PRODUCTO ENTERO ──
    #
    # ── LA PRIMERA VERSIÓN DE ESTA COMPROBACIÓN ESTABA MAL ──
    #
    # Contaba cadenas con forma de ruta en el código y exigía como mucho una.
    # Salió en rojo el 07/10/2026 señalando `recorrido.mjs`, que **es
    # correcto**: lleva una tabla de cuántas acciones mínimas tiene que ofrecer
    # cada pantalla, y esa tabla va legítimamente indexada por ruta.
    #
    # Contar rutas era medir lo que se ve, no la propiedad que importa. La
    # propiedad es: **quién decide qué pantallas se recorren**. Y es el router,
    # si la sonda importa `rutas.mjs` y además —cuando tiene una tabla por
    # ruta— **revienta si al router le sobra una fila**. Una tabla que se
    # comprueba contra el router no puede quedarse corta; una lista que manda,
    # sí.
    # ── Y ESTA LISTA TAMBIÉN ERA A MANO, DENTRO DE LA COMPROBACIÓN QUE
    #    EXISTE PARA CAZAR LISTAS A MANO (14/10/2026) ──
    #
    # Decía `("auditoria.mjs", "recorrido.mjs")`, las dos sondas que recorrían
    # el producto el 07/10/2026. El 14/10/2026 se escribió una tercera
    # —`encimado.mjs`, para medir el texto montado que Dirección vio— y esta
    # comprobación **no la habría mirado nunca**: ni que importara `rutas.mjs`,
    # ni que lo llamara, ni que su tabla reventara.
    #
    # Es el defecto de esta comprobación repetido dentro de ella misma, que es
    # lo que lleva pasando todo el proyecto: la lección iba escrita en la
    # cabecera y la lista seguía siendo a mano. **Saberlo no basta.**
    #
    # Ahora las sondas salen de la carpeta, y son las que importan `rutas.mjs`.
    # Eso deja un hueco obvio —una sonda nueva con su lista propia no importa
    # `rutas.mjs`, así que se escaparía— y se cierra con lo de abajo: ninguna
    # sonda puede nombrar más de `MAX_RUTAS_PROPIAS` rutas por su cuenta.
    #
    # El umbral está **medido, no elegido**: las sondas de una pantalla nombran
    # 0 o 1 ruta, `vista-previa.mjs` nombra 3 —tres pantallas concretas que
    # comprueba— y las que recorren el producto nombran 0 porque las piden. Una
    # lista a mano del producto entero son dieciocho.
    MAX_RUTAS_PROPIAS = 3
    carpeta = os.path.join(RAIZ, "app", "pruebas", "sondas")
    # No son sondas: `rutas.mjs` es el módulo que se vigila, y los otros tres
    # son ayudas compartidas —lanzar Chromium, correr una orden, la lista de
    # temas— que no abren ninguna pantalla.
    NO_SON_SONDAS = {"rutas.mjs", "navegador.mjs", "ordenes.mjs", "temas.mjs"}

    que_recorren = []
    nombres = sorted(n for n in os.listdir(carpeta) if n.endswith(".mjs"))
    for nombre in nombres:
        if nombre in NO_SON_SONDAS:
            continue
        fuente = leer("app", "pruebas", "sondas", nombre) or ""
        codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", fuente, flags=re.S))
        if "rutas.mjs" in codigo:
            que_recorren.append(nombre)
            continue
        propias = set(re.findall(r"['\"`]#/[a-z0-9/_:-]+['\"`]", codigo))
        if len(propias) > MAX_RUTAS_PROPIAS:
            faltan.append(
                f"{nombre} nombra {len(propias)} rutas por su cuenta y no "
                "importa rutas.mjs: es una lista a mano, y se quedará corta"
            )

    if not que_recorren:
        faltan.append(
            "ninguna sonda importa rutas.mjs: o se borró el módulo, o esta "
            "comprobación dejó de encontrar las sondas y estaría en verde sin mirar"
        )

    for sonda in que_recorren:
        fuente = leer("app", "pruebas", "sondas", sonda)
        if not fuente:
            faltan.append(f"no existe app/pruebas/sondas/{sonda}")
            continue
        # Se quitan los comentarios antes de mirar. Las dos sondas EXPLICAN en
        # su cabecera qué rutas les faltaban, con sus nombres escritos — y sin
        # esto la comprobación se cazaría en esa explicación. Es la trampa del
        # comentario, que ya ha salido cinco veces en este proyecto.
        codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", fuente, flags=re.S))
        if "rutas.mjs" not in codigo:
            faltan.append(f"{sonda} no importa rutas.mjs: ¿volvió a tener su lista?")
            continue
        # Si la sonda tiene datos por ruta, el recorrido tiene que salir de
        # `leerRutas()` y no de las claves de esa tabla: al revés, añadir una
        # pantalla y no apuntarla la dejaría fuera en silencio, que es
        # exactamente lo que pasó.
        tiene_tabla = bool(re.search(r"'/[a-z][a-z/-]*'\s*:", codigo))
        if tiene_tabla and "throw new Error" not in codigo:
            faltan.append(
                f"{sonda} tiene una tabla por ruta y no revienta cuando al router "
                "le sobra una: una pantalla nueva se quedaría fuera en silencio"
            )
        if not re.search(r"leerRutas(ConAlmohadilla)?\(\)", codigo):
            faltan.append(f"{sonda} importa rutas.mjs y no lo llama")

    check(
        8,
        "las sondas recorren todas las pantallas del router",
        not faltan,
        las_sondas_recorren_todas_las_pantallas.__doc__,
        detalle="\n      ".join(faltan),
    )


def cada_caso_frontera_nombra_su_test():
    """**Una lista de casos frontera que nadie puede comprobar envejece sola.**

    `ESTRATEGIA_QA.md` §3 lleva desde la Fase 0 la lista de los fallos que
    convierten ARLES en un problema para el cliente, y afirmaba de ella que
    «cada uno tiene un test con nombre» — **sin decir cuál**. Así que la
    afirmación no se podía comprobar, y la auditoría C del §140 la comprobó a
    mano el 07/10/2026: a dos casos les faltaba el test, y tres describían
    caminos que el producto no tiene.

    Ahora cada fila nombra su test y esto lo verifica: lee los nombres entre
    comillas invertidas de las tablas del §3 y exige que cada uno exista como
    `fn <nombre>` en algún `.rs` de `crates/`. Un test que alguien renombre o
    borre deja la tabla mintiendo, y aquí se para.

    **Lo que esta comprobación NO hace, y conviene saberlo:** comprueba que el
    test *existe*, no que *cubra el caso*. Para eso no hay comprobación
    automática posible — es lo que hace la auditoría cada vez, y por eso la
    auditoría existe.

    Verificada el 07/10/2026 de dos formas: renombrando
    `quien_pidio_la_baja_no_recibe_nada` en el motor —falla nombrándolo— y
    quitando una columna «Su test» de una tabla del documento, que falla por
    tener menos casos nombrados que filas.
    """
    faltan = []

    doc = leer("documentacion", "06-calidad", "ESTRATEGIA_QA.md") or ""
    if not doc:
        faltan.append("no existe ESTRATEGIA_QA.md")
        check(8, "cada caso frontera nombra su test", False,
              cada_caso_frontera_nombra_su_test.__doc__, detalle="\n      ".join(faltan))
        return

    # Sólo el §3, no el documento entero: el §5 también nombra cosas entre
    # comillas y no son tests.
    try:
        trozo = doc[doc.index("## 3. Los casos que de verdad rompen"):doc.index("## 3 bis.")]
    except ValueError:
        faltan.append("el §3 no tiene la forma esperada: ¿se renombró la sección?")
        trozo = ""

    # ── SÓLO LAS FILAS DE TABLA, Y ESTO LO ENSEÑÓ UN ROJO ──
    #
    # Escrita sobre el §3 entero, esta comprobación salió en rojo el
    # 07/10/2026 señalando `cada_caso_frontera_nombra_su_test` — **su propio
    # nombre**, que la entradilla de la sección cita al explicar qué vigila.
    # Es la trampa que este proyecto ya ha visto cinco veces —una comprobación
    # que se caza a sí misma en un texto— y aquí apareció del revés: no un
    # falso verde, un falso rojo sobre un documento correcto.
    #
    # El arreglo no es una excepción con su nombre dentro, que se quedaría
    # obsoleta al renombrarla: es mirar **sólo las filas de tabla**, que es
    # donde viven los casos. Lo que el documento diga en prosa sobre sí mismo
    # no es un caso frontera.
    filas = [l for l in trozo.splitlines() if l.lstrip().startswith("|")]
    # Los nombres de test tienen forma de identificador de Rust y son largos:
    # el filtro de longitud deja fuera `.arles`, `.txt`, `company.timezone` y
    # los demás nombres cortos que las tablas citan.
    nombres = {
        n for n in re.findall(r"`([a-z][a-z0-9_]+)`", "\n".join(filas))
        if n.count("_") >= 3
    }

    # ── LA GUARDA QUE FALTABA, Y ES LA IMPORTANTE ──
    #
    # Sin esto, borrar la columna «Su test» de una tabla entera dejaría la
    # comprobación en verde: cero nombres, cero fallos. Es la forma de fallo
    # que esta familia de comprobaciones ha tenido cinco veces en este
    # proyecto — vigilar una lista y no vigilar que la lista esté llena.
    MINIMO = 25
    if len(nombres) < MINIMO:
        faltan.append(
            f"el §3 sólo nombra {len(nombres)} tests y tiene que nombrar al menos "
            f"{MINIMO}: ¿se quitó la columna «Su test» de alguna tabla?"
        )

    declaradas = set()
    for ruta, _, archivos in os.walk(os.path.join(RAIZ, "crates")):
        for archivo in archivos:
            if archivo.endswith(".rs"):
                try:
                    with open(os.path.join(ruta, archivo), encoding="utf-8") as f:
                        declaradas.update(re.findall(r"\bfn\s+([a-z][a-z0-9_]+)", f.read()))
                except OSError:
                    pass

    for n in sorted(nombres - declaradas):
        faltan.append(f"el §3 nombra el test `{n}` y no existe en crates/")

    check(
        8,
        f"cada caso frontera nombra su test ({len(nombres)} nombrados)",
        not faltan,
        cada_caso_frontera_nombra_su_test.__doc__,
        detalle="\n      ".join(faltan),
    )


def la_clave_perdida_tiene_salida():
    """**El consejo que el producto no puede cumplir.**

    Desde la Fase 1, perder la entrada del llavero produce un mensaje correcto
    con las tres partes del §95 — y la parte de «cómo se arregla» dice
    *«Restaura tu último respaldo .arles»*. **Y no había forma de hacerlo**: la
    aplicación se detenía con `std::process::exit(1)` antes de abrir ninguna
    ventana.

    Es el patrón del §9 en su forma más cruel: no es que falte un botón, es que
    el producto **da una instrucción que él mismo impide seguir**, y justo el
    día en que alguien ya ha perdido algo.

    Esta comprobación se escribió **antes** de construir la salida, y se vio
    fallar nombrando los cuatro tramos el 06/10/2026.

    Vigila además las dos cosas que no se pueden hacer mal aquí:

    - **No se crea una clave nueva al arrancar.** Sería lo cómodo y es el
      hallazgo F2 de la Fase 1: la base existente quedaría irrecuperable para
      siempre y nadie se enteraría. La clave nueva sólo se crea cuando alguien
      lo pide con todas las letras.
    - **La base ilegible no se borra.** Se aparta con otro nombre. Un llavero
      puede volver —un Keychain restaurado, un perfil que se recupera—, y el
      día que vuelva esos datos valen más que el respaldo.
    """
    faltan = []

    app = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "crates", "arles-app", "src")):
        for f in ficheros:
            if f.endswith(".rs"):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    app += fh.read()
    assert app, "no se leyó el código de arles-app"
    codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", app, flags=re.S))

    # 1 · Que exista el modo, y que el arranque no se limite a morirse.
    if "recuperacion" not in codigo:
        faltan.append("no hay modo de recuperación en arles-app")
    if "ClaveMaestraPerdida" in codigo and "modo_recuperacion" not in codigo:
        faltan.append(
            "ClaveMaestraPerdida sigue sin llevar a ninguna parte: falta `modo_recuperacion`"
        )

    # 2 · Los comandos de la salida, registrados en la frontera.
    #
    # Se buscan en TODOS los `generate_handler!` del crate y no sólo en el de
    # `lib.rs`: el modo de recuperación registra los suyos aparte a propósito
    # —ninguno de los normales puede funcionar sin base—, y una comprobación
    # que mire sólo un archivo daría por ausente un comando que sí está.
    # Escrita primero mirando `lib.rs`, y corregida el 06/10/2026 al verla
    # señalar dos comandos que estaban registrados donde tocaba.
    registrados = "\n".join(
        re.findall(r"generate_handler!\s*\[(.*?)\]", app, flags=re.S)
    )
    for comando in ("revisar_respaldo_para_recuperar", "recuperar_desde_respaldo"):
        if comando not in registrados:
            faltan.append(f"{comando} — no está en ningún generate_handler!")

    # 3 · Y alguien que los pulse.
    frontend = ""
    for carpeta, _, ficheros in os.walk(os.path.join(RAIZ, "app", "src")):
        for f in ficheros:
            if f.endswith((".ts", ".vue")):
                with open(os.path.join(carpeta, f), encoding="utf-8") as fh:
                    frontend += fh.read()
    for comando in ("revisar_respaldo_para_recuperar", "recuperar_desde_respaldo"):
        if comando not in frontend:
            faltan.append(f"{comando} — ninguna pantalla lo llama")

    # 4 · La base ilegible se aparta, no se borra.
    if "apartar" not in codigo:
        faltan.append("nada aparta la base ilegible: ¿se está borrando?")
    if re.search(r"remove_file\s*\(\s*&?base", codigo):
        faltan.append("se borra la base ilegible — tiene que apartarse, no borrarse")

    check(
        8,
        "la clave perdida tiene salida, y no se lleva nada por delante",
        not faltan,
        la_clave_perdida_tiene_salida.__doc__,
        detalle="\n      ".join(faltan),
    )


def el_preflight_dice_lo_que_no_vuelve():
    """**Restaurar se lleva cosas por cascada, y hay que decirlas.**

    D-8C dice «reemplazar, nunca mezclar», y suena a que cada tabla del respaldo
    sustituye a la suya y lo demás se queda igual. Medido el 06/10/2026 sobre el
    esquema V10, **no es lo que pasa**: `company` se reemplaza en los dos
    alcances y casi todo cuelga de ella con `ON DELETE CASCADE`, así que
    restaurar un respaldo esencial se lleva también `activity_event` y
    `whatsapp_quality_event` — que son `SoloCompleto` y **no vuelven**.

    Es el mismo fallo que en la Fase 1 costó la audiencia congelada y el
    registro de envíos (F1). La diferencia es que ahora se dice antes, con
    cifras, como el recorte del registro de la 7.4.

    Lo que esta comprobación vigila es que el cálculo siga leyendo **el
    esquema** —`PRAGMA foreign_key_list`— y no una lista a mano, que es lo que
    envejece en silencio. La exactitud del cálculo la comprueba
    `la_cascada_calculada_es_la_que_ejecuta_sqlite` en `arles-db`, borrando de
    verdad dentro de una transacción que se deshace.

    Verificada el 06/10/2026 sustituyendo el cierre por una lista fija: falla
    nombrando la lista.
    """
    fuente = leer("crates/arles-db/src/restauracion.rs") or ""
    assert fuente, "no existe crates/arles-db/src/restauracion.rs"

    codigo = re.sub(r"//[^\n]*", "", re.sub(r"/\*.*?\*/", "", fuente, flags=re.S))

    lee_el_esquema = "foreign_key_list" in codigo and "sqlite_master" in codigo
    # Una lista a mano de tablas arrastradas sería un arreglo de literales.
    a_mano = re.findall(
        r'(?:&?\[)\s*"(?:activity_event|whatsapp_quality_event|rate_budget|engine_control)"',
        codigo,
    )
    dice_el_minimo = "bajas_al_menos" in codigo and ".max(" in codigo

    check(
        8,
        "el preflight dice lo que no vuelve, leyendo el esquema",
        lee_el_esquema and not a_mano and dice_el_minimo,
        el_preflight_dice_lo_que_no_vuelve.__doc__,
        detalle=(
            f"lee las claves ajenas del esquema: {lee_el_esquema}\n"
            f"      lista de tablas escrita a mano: {a_mano}\n"
            f"      las bajas prometen un mínimo que no baja: {dice_el_minimo}"
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
    entradas = _entradas_del_manifiesto()
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
    alcance = next(
        (a for t, a, _ in _entradas_del_manifiesto() if t == "suppression_entry"),
        "(no declarada)",
    )
    check(
        8,
        "las bajas entran enteras en los dos respaldos",
        alcance == "SiempreEntera",
        las_bajas_nunca_se_recortan.__doc__,
        detalle=f"suppression_entry está como: {alcance}",
    )


def _tablas_del_archivo():
    """Las tablas que `EN_EL_ARCHIVO` declara, leídas del núcleo.

    Devuelve `None` si la constante no existe o el patrón no casa, que es lo que
    distingue «no hay archivado» de «el archivado no lleva nada» — y es la
    trampa de `el_manifiesto_se_lee_entero`: una regex que deja de casar no
    falla, aprueba una lista vacía.
    """
    fuente = leer("crates/arles-core/src/respaldo.rs") or ""
    m = re.search(r"EN_EL_ARCHIVO:\s*&\[&str\]\s*=\s*&\[(.*?)\];", fuente, re.S)
    if not m:
        return None
    # Se quitan los comentarios antes de buscar nombres: un `// message_attempt`
    # explicativo dentro de la lista contaría como una tabla declarada. Es el
    # defecto que la 8.5 encontró dos veces (una comprobación que se caza a sí
    # misma en un comentario).
    sin_comentarios = re.sub(r"//[^\n]*", "", m.group(1))
    return re.findall(r'"(\w+)"', sin_comentarios)


def las_bajas_y_el_consentimiento_nunca_se_archivan():
    """**La tercera condición de D-8D, hecha comprobación.**

    Archivar es mover datos a un disco que deja de ser responsabilidad de ARLES.
    Dirección puso cuatro condiciones el 05/10/2026, y la tercera es que **las
    bajas y el consentimiento no entran nunca**: son de `suppression_entry` y
    `consent_entry`, no del historial, y no se archivan ni se recortan jamás.

    Es la misma regla que `las_bajas_nunca_se_recortan` vigila en los respaldos,
    y hace falta por separado porque el archivado tiene **su propia lista de
    tablas**: una baja que entrara ahí se iría de la base al borrar, y volver a
    escribirle a quien pidió que no es lo peor que este producto puede hacer.

    También exige que cada tabla declarada exista en el manifiesto y no esté como
    `Nunca`: archivar una tabla que ningún respaldo se lleva sacaría datos de la
    base sin que ninguna copia los tenga.

    Verificada el 06/10/2026 añadiendo `"suppression_entry"` a `EN_EL_ARCHIVO`:
    falla nombrándola.
    """
    tablas = _tablas_del_archivo()
    if tablas is None:
        check(
            8,
            "las bajas y el consentimiento nunca se archivan",
            False,
            las_bajas_y_el_consentimiento_nunca_se_archivan.__doc__,
            detalle="no se encuentra EN_EL_ARCHIVO en arles-core/src/respaldo.rs",
        )
        return

    prohibidas = sorted(set(tablas) & {"suppression_entry", "consent_entry"})
    del_manifiesto = {t: a for t, a, _ in _entradas_del_manifiesto()}
    sin_clasificar = sorted(t for t in tablas if t not in del_manifiesto)
    marcadas_nunca = sorted(t for t in tablas if del_manifiesto.get(t) == "Nunca")

    problemas = []
    if prohibidas:
        problemas.append(f"se archivan y no deben: {', '.join(prohibidas)}")
    if sin_clasificar:
        problemas.append(f"no están en el manifiesto: {', '.join(sin_clasificar)}")
    if marcadas_nunca:
        problemas.append(f"están como Nunca: {', '.join(marcadas_nunca)}")

    check(
        8,
        "las bajas y el consentimiento nunca se archivan",
        not problemas and bool(tablas),
        las_bajas_y_el_consentimiento_nunca_se_archivan.__doc__,
        detalle="; ".join(problemas) or f"el archivo lleva: {', '.join(tablas)}",
    )


def un_archivo_de_historial_no_se_restaura():
    """**El archivo y el respaldo no se pueden confundir, y la confusión sería
    destructiva.**

    Un archivo de historial trae un año de envíos y nada más. Restaurarlo con
    D-8C —reemplazar, nunca mezclar— **vaciaría la base entera** para dejar
    dentro sólo esos envíos: los contactos, las bajas y las campañas se irían.
    No es un riesgo teórico; es lo que el restaurador hace con cualquier archivo
    que acepte.

    Por eso se vigilan las dos direcciones:

    1. El archivo lleva **su propia marca** en la primera línea, distinta a la
       del respaldo, así que el restaurador lo rechaza antes de leer una fila.
    2. `Db::restaurar` **nombra el alcance de historial** y lo rechaza con su
       error, en vez de dejarlo caer en un `_ =>` que mañana haría otra cosa.

    La marca sola no basta: el día que alguien arme una cabecera a mano —como
    hace el modo de recuperación— la marca la pone el escritor y el alcance la
    cabecera.

    ── LAS DOS PUERTAS SE VIGILAN POR SEPARADO, Y ES UN ARREGLO DEL §2 ──

    La primera versión de esta comprobación buscaba `Alcance::Historial` en todo
    `restauracion.rs`, y su comentario decía estar verificada quitando la rama
    de `Db::restaurar`. **Era falso.** Al quitarla, el `match` del preflight
    seguía nombrándola y la comprobación pasaba en verde: vigilaba una puerta de
    dos y afirmaba vigilar las dos. Lo cazó hacer la ruptura de verdad, el
    06/10/2026, en vez de darla por hecha — que es exactamente para lo que
    existe el §2.

    Ahora se busca cada puerta por lo que la distingue: el `match` del preflight
    por la variante, y el restaurador por la llamada a `se_puede_restaurar`.

    Verificada el 06/10/2026 quitando cada una por separado: falla nombrando la
    que falta.
    """
    formato = leer("crates/arles-core/src/formato_de_respaldo.rs") or ""
    restaurar = leer("crates/arles-db/src/restauracion.rs") or ""
    # Sin comentarios, por lo mismo que `el_archivado_cuenta_lo_que_arrastra`:
    # las dos puertas están explicadas en comentarios que las nombran.
    restaurar = re.sub(r"//[^\n]*", "", restaurar)

    marca_archivo = re.search(r'MARCA_ARCHIVO:\s*&str\s*=\s*"([^"]+)"', formato)
    marca = re.search(r'^pub const MARCA:\s*&str\s*=\s*"([^"]+)"', formato, re.M)

    problemas = []
    if not marca_archivo:
        problemas.append("el formato no declara MARCA_ARCHIVO")
    elif not marca or marca_archivo.group(1) == marca.group(1):
        problemas.append("MARCA_ARCHIVO no es distinta de MARCA")
    # Puerta 1 · el preflight nombra la variante en su `match`.
    if not re.search(r"Alcance::Historial\s*\{[^}]*\}\s*=>", restaurar):
        problemas.append("el preflight no nombra Alcance::Historial para rechazarlo")
    # Puerta 2 · el restaurador pregunta antes de tocar la base.
    if "se_puede_restaurar()" not in restaurar:
        problemas.append("Db::restaurar no pregunta si el alcance se puede restaurar")

    check(
        8,
        "un archivo de historial no se puede restaurar",
        not problemas,
        un_archivo_de_historial_no_se_restaura.__doc__,
        detalle="; ".join(problemas) or "marca propia y rechazo explícito",
    )


def el_archivado_cuenta_lo_que_arrastra():
    """**Borrar historial arrastra el registro de actividad, y hay que decirlo
    antes.**

    `activity_event.message_attempt_id` tiene clave ajena a `message_attempt`
    con `ON DELETE CASCADE`. Así que borrar un año de envíos **se lleva también**
    los sucesos del registro que apuntaban a ellos — sin aviso, sin error, y sin
    que el archivo los lleve, porque `activity_event` no entra en el archivado.

    Es exactamente el fallo de la Fase 1 (F1) y el de D-8C en la 8.3a, por
    tercera vez: la cascada protege de otra cosa y no se toca; lo que no puede
    pasar es que nadie la cuente. El preflight la calcula **del esquema**, con
    `PRAGMA foreign_key_list`, y no de una lista a mano que envejecería en
    silencio el día que otra tabla cuelgue del historial.

    Y exige la segunda condición de D-8D: el borrado recibe **cuántas filas se
    archivaron** y se niega si la base ya no tiene esas. Entre archivar y borrar
    el motor sigue escribiendo en su hilo.

    Verificada el 06/10/2026 sustituyendo el `PRAGMA` por la lista
    `["activity_event"]`: falla.

    ── Y UN DEFECTO DE ESTA COMPROBACIÓN, DEL 06/10/2026 ──

    La primera versión buscaba el nombre del error con una lista de tres
    nombres **inventados al escribir la comprobación** —`HistorialCambiado`,
    `FilasQueNoCuadran`, `NoCuadra`— y el del código es `ElHistorialCambio`. No
    casaba ninguno: salía en rojo sobre un código correcto, que es el gemelo del
    defecto de la 8.5 (la comprobación que se cazaba a sí misma en un
    comentario). Ahora lee **la forma del código**: que el borrado reciba cuántas
    filas se archivaron y que haya un `return Err` con ese dato delante.

    ── Y UN SEGUNDO DEFECTO, DE LA MISMA TARDE ──

    Buscar `foreign_key_list` en el archivo entero **se cazaba en la cabecera
    del módulo**, que lo nombra al explicar por qué la cascada se lee del
    esquema. Al cambiar el `PRAGMA` de verdad por otro, la comprobación seguía
    en verde. **Tercera vez** que este patrón aparece en el proyecto (5.3 y 8.5
    las otras dos), y la lección se repite: una comprobación que busca una
    palabra en un archivo con comentarios busca en los comentarios. Se quitan
    antes de mirar.
    """
    fuente = leer("crates/arles-db/src/archivo.rs")
    if fuente is not None:
        # Sin los comentarios, por lo de arriba. `//!` y `///` incluidos.
        fuente = re.sub(r"//[^\n]*", "", fuente)
    if fuente is None:
        check(
            8,
            "el archivado cuenta lo que la cascada se lleva",
            False,
            el_archivado_cuenta_lo_que_arrastra.__doc__,
            detalle="no existe crates/arles-db/src/archivo.rs",
        )
        return

    problemas = []
    if "foreign_key_list" not in fuente:
        problemas.append("la cascada no se calcula del esquema")
    if not re.search(r"fn\s+que_pasaria_al_archivar", fuente):
        problemas.append("no hay preflight que_pasaria_al_archivar")
    # El borrado tiene que recibir el recuento del archivo y negarse con él.
    borrado = re.search(
        r"fn\s+borrar_el_historial_archivado\s*\((.*?)\)\s*->", fuente, re.S
    )
    if not borrado:
        problemas.append("no hay borrar_el_historial_archivado")
    elif "archivadas" not in borrado.group(1):
        problemas.append("el borrado no recibe cuántas filas se archivaron")
    elif not re.search(r"archivadas[^\n]*\n[^\n]*return Err\(", fuente):
        problemas.append("el borrado no se niega si el recuento no cuadra")

    check(
        8,
        "el archivado cuenta lo que la cascada se lleva",
        not problemas,
        el_archivado_cuenta_lo_que_arrastra.__doc__,
        detalle="; ".join(problemas) or "cascada del esquema y recuento exigido",
    )


def fase_8(rapido=False):
    print(f"{NEGRITA}Fase 8 · Respaldos y endurecimiento{FIN}")

    print(f"{GRIS}  el manifiesto del respaldo (8.0){FIN}")
    el_manifiesto_se_lee_entero()
    ninguna_tabla_se_queda_fuera_del_respaldo()
    toda_tabla_que_no_entra_dice_por_que()
    las_bajas_nunca_se_recortan()

    print(f"{GRIS}  leer la base por tandas (8.2b){FIN}")
    lo_que_se_recorta_dice_por_que_columna()
    el_respaldo_no_se_lee_en_memoria()

    print(f"{GRIS}  el comando y la pantalla (8.2c){FIN}")
    el_exportador_tiene_quien_lo_llame()

    print(f"{GRIS}  leer un respaldo de vuelta (8.3a){FIN}")
    el_preflight_dice_lo_que_no_vuelve()

    print(f"{GRIS}  restaurar de verdad (8.3b y 8.3c){FIN}")
    el_restaurador_tiene_quien_lo_llame()

    print(f"{GRIS}  el llavero perdido (8.4){FIN}")
    la_clave_perdida_tiene_salida()

    print(f"{GRIS}  el banco de pruebas (8.5){FIN}")
    el_banco_no_viaja_en_el_instalador()

    print(f"{GRIS}  archivar el historial (8.3 bis){FIN}")
    las_bajas_y_el_consentimiento_nunca_se_archivan()
    un_archivo_de_historial_no_se_restaura()
    el_archivado_cuenta_lo_que_arrastra()
    el_archivado_tiene_quien_lo_llame()

    print(f"{GRIS}  los índices y su vigilancia (8.6){FIN}")
    cambiar_una_migracion_recompila()

    print(f"{GRIS}  la auditoría de QA (8.6 C){FIN}")
    cada_caso_frontera_nombra_su_test()

    print(f"{GRIS}  la auditoría de UX (8.6 D){FIN}")
    las_sondas_recorren_todas_las_pantallas()

    print(f"{GRIS}  el desenlace conectado (8.6 E){FIN}")
    el_desenlace_de_la_simulacion_llega_hasta_la_pantalla()


def fase_7(rapido=False):
    print(f"{NEGRITA}Fase 7 · Actividad y entregabilidad{FIN}")

    print(f"{GRIS}  el freno (7.0){FIN}")
    toda_capacidad_del_nucleo_tiene_puerta()
    el_freno_llega_hasta_la_pantalla()
    ningun_slot_se_escribe_a_ciegas()

    print(f"{GRIS}  el registro de actividad (7.1){FIN}")
    el_motor_no_tira_lo_que_sabe()

    print(f"{GRIS}  el descanso entre campañas (D-16){FIN}")
    el_descanso_entre_campanas_llega_hasta_la_pantalla()

    print(f"{GRIS}  las cifras (7.2){FIN}")
    las_cifras_salen_de_un_solo_sitio()

    print(f"{GRIS}  la pantalla de lo de ahora (7.3){FIN}")
    todo_comando_que_piden_las_pantallas_lo_contesta_el_simulador()

    print(f"{GRIS}  el registro con sus filtros (7.4){FIN}")
    ningun_texto_lleva_una_arroba_suelta()
    ningun_store_se_inventa_la_causa_de_un_error()
    ningun_texto_lleva_markdown()
    ningun_aviso_sobrevive_a_la_recarga_que_lo_resuelve()

    print(f"{GRIS}  el resumen exportable (7.6){FIN}")
    el_resumen_no_puede_llevar_direcciones()

    print(f"{GRIS}  SPF, DKIM y DMARC (7.5){FIN}")
    toda_clave_de_pantalla_del_nucleo_tiene_texto()
    los_registros_del_dominio_llegan_hasta_el_preflight()

    print(f"{GRIS}  el umbral de rebotes (R-3){FIN}")
    el_umbral_de_rebotes_frena_de_verdad()


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
