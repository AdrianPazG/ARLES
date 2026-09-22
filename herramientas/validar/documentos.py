#!/usr/bin/env python3
"""La documentación dice la verdad · 22/09/2026

    python3 herramientas/validar/documentos.py

─────────────────────────────────────────────────────────────────────────────
POR QUÉ ESTO EXISTE

`documentacion/` son 66 archivos y 40 MB. Nadie los relee enteros, y la
documentación no falla ruidosamente: falla quedándose atrás. Un enlace que
apunta a un archivo que se renombró, una cifra que era cierta hace una semana,
un documento que dice «ocho preguntas» cuando ya son dieciocho.

Ninguna de esas cosas rompe nada. Todas hacen que Dirección lea algo que no es
cierto, que es exactamente el daño que hizo la vista previa el 22/09: el
problema no fue el defecto, fue **revisar sobre una copia que mentía**.

QUÉ COMPRUEBA, Y POR QUÉ CADA COSA

1. **Todo enlace interno resuelve.** Un enlace roto en un índice convierte el
   mapa en adivinanza. Se comprueban también las anclas de sección (`#algo`).

2. **Las cifras que se afirman coinciden con la realidad.** Número de
   migraciones, de preguntas jurídicas, de pruebas. Se cuenta la realidad y se
   compara con lo escrito, en vez de confiar en que alguien lo actualizó.

3. **El avance del README sale de su fuente única.** Ya lo vigila el
   validador; aquí se comprueba además que la FECHA no se quede atrás.

4. **Ningún documento promete un anexo que no existe.** Es el defecto que tuvo
   la consulta jurídica: ofrecía dos anexos que nunca se escribieron, y el
   abogado los habría pedido.

5. **Ningún identificador de modelo** en nada que acabe en el repositorio.
   Es una regla dura del proyecto (CLAUDE.md §1) y no la vigilaba nadie.

SE HA VISTO FALLAR · 22/09/2026
Cada comprobación se rompió a propósito antes de darla por buena. Está anotado
en cada función.
─────────────────────────────────────────────────────────────────────────────
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
DOCS = RAIZ / "documentacion"

VERDE, ROJO, GRIS, FIN = "\033[32m", "\033[31m", "\033[90m", "\033[0m"

fallos: list[str] = []


def mal(que: str, detalle: str = "") -> None:
    fallos.append(que if not detalle else f"{que}\n      {detalle}")


def bien(que: str) -> None:
    print(f"  {VERDE}✓{FIN} {que}")


def documentos() -> list[Path]:
    """Todo el Markdown del repositorio, no sólo el de `documentacion/`."""
    salida = []
    for base in (DOCS, RAIZ):
        for p in base.glob("*.md" if base is RAIZ else "**/*.md"):
            if ".git" in p.parts or "node_modules" in p.parts:
                continue
            salida.append(p)
    return sorted(set(salida))


# ── 1 · Los enlaces ──────────────────────────────────────────────────────────

RE_ENLACE = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")
RE_CODIGO = re.compile(r"```.*?```", re.S)


def titulos_de(texto: str) -> set[str]:
    """Las anclas que GitHub genera para los encabezados de un documento.

    GitHub baja a minúsculas, quita lo que no sea letra, número, guion o
    espacio, y cambia los espacios por guiones. Se replica esa regla: si se
    inventara otra, el enlace pasaría aquí y fallaría en GitHub, que es donde
    Dirección lee.
    """
    anclas = set()
    for linea in texto.splitlines():
        if not linea.startswith("#"):
            continue
        t = linea.lstrip("#").strip()
        t = re.sub(r"[`*_\[\]()]", "", t)
        t = re.sub(r"[^\w\s-]", "", t, flags=re.UNICODE)
        anclas.add(t.strip().lower().replace(" ", "-"))
    return anclas


def los_enlaces_llevan_a_alguna_parte() -> None:
    """Todo enlace interno apunta a algo que existe.

    ROTO A PROPÓSITO: se cambió un enlace de `00-INDICE.md` a un archivo
    inexistente y a un ancla inexistente. Los dos se cazaron.
    """
    rotos = []
    for doc in documentos():
        texto = RE_CODIGO.sub("", doc.read_text(encoding="utf-8"))
        for etiqueta, destino in RE_ENLACE.findall(texto):
            destino = destino.strip()
            if destino.startswith(("http://", "https://", "mailto:", "#!")):
                continue
            if destino.startswith("#"):
                # Ancla dentro del mismo documento.
                if destino[1:].lower() not in titulos_de(doc.read_text(encoding="utf-8")):
                    rotos.append(f"{doc.relative_to(RAIZ)} → «{etiqueta}» {destino}")
                continue
            archivo, _, ancla = destino.partition("#")
            objetivo = (doc.parent / archivo).resolve()
            if not objetivo.exists():
                rotos.append(f"{doc.relative_to(RAIZ)} → «{etiqueta}» {destino} (no existe)")
                continue
            if ancla and objetivo.suffix == ".md":
                if ancla.lower() not in titulos_de(objetivo.read_text(encoding="utf-8")):
                    rotos.append(
                        f"{doc.relative_to(RAIZ)} → «{etiqueta}» {destino} (sin esa sección)"
                    )
    if rotos:
        mal(f"{len(rotos)} enlace(s) interno(s) roto(s)", "\n      ".join(rotos[:15]))
    else:
        bien("todos los enlaces internos llevan a alguna parte")


# ── 2 · Las cifras ───────────────────────────────────────────────────────────


def contar_migraciones() -> int:
    d = RAIZ / "crates/arles-db/migrations"
    return len(list(d.glob("V*.sql"))) if d.exists() else 0


def contar_preguntas_juridicas() -> int:
    """Las preguntas C-n de la consulta, contadas sin repetir."""
    f = DOCS / "08-legal/CONSULTA-JURIDICA.md"
    if not f.exists():
        return 0
    return len(set(re.findall(r"\bC-(\d+)\b", f.read_text(encoding="utf-8"))))


def contar_pruebas_de_rust() -> int:
    r = subprocess.run(
        ["cargo", "test", "--workspace", "--no-fail-fast"],
        cwd=RAIZ, capture_output=True, text=True, timeout=1800,
    )
    return sum(int(m) for m in re.findall(r"^test result: ok\. (\d+) passed", r.stdout, re.M))


def citada(original: str, i: int) -> bool:
    """¿La cifra que hay en la posición `i` está entrecomillada con «»?

    Citar un error no es cometerlo. El informe de la Fase 3 cuenta que tres
    documentos decían «las ocho preguntas», y esa frase tiene que poder
    escribirse sin que la comprobación la señale — si no, contar un error se
    vuelve imposible y el remedio sería callarlo.

    Se mira SÓLO la línea: unas comillas abiertas veinte párrafos más arriba no
    protegen nada, y así un texto sin cerrar no puede silenciar el resto del
    documento.

    El riesgo de que alguien esconda una cifra falsa entrecomillándola existe y
    se acepta: nadie escribe una afirmación propia entre «». La consulta
    jurídica decía **Nueve preguntas nuevas** en negrita y sin comillas, que es
    como se escribe lo que uno afirma.
    """
    inicio = original.rfind("\n", 0, i) + 1
    fin = original.find("\n", i)
    linea = original[inicio : fin if fin != -1 else len(original)]
    rel = i - inicio
    return linea.count("«", 0, rel) > linea.count("»", 0, rel)


def las_cifras_afirmadas_son_ciertas(rapido: bool) -> None:
    """Lo que la documentación afirma coincide con lo que hay.

    ROTO A PROPÓSITO: se escribió «cuatro migraciones» en un documento cuando
    hay cinco, y «ocho preguntas» cuando hay dieciocho. Los dos se cazaron
    nombrando el archivo y la línea.

    Se comprueba sólo lo que se puede contar sin ambigüedad. Una afirmación
    como «la tabla aguanta 500 000 contactos» no se cuenta aquí: ésa la mide
    una sonda, que es su sitio.
    """
    migraciones = contar_migraciones()
    preguntas = contar_preguntas_juridicas()

    NUMEROS = {
        "uno": 1, "dos": 2, "tres": 3, "cuatro": 4, "cinco": 5, "seis": 6,
        "siete": 7, "ocho": 8, "nueve": 9, "diez": 10, "once": 11, "doce": 12,
        "trece": 13, "catorce": 14, "quince": 15, "dieciséis": 16,
        "diecisiete": 17, "dieciocho": 18, "diecinueve": 19, "veinte": 20,
    }
    palabra = "|".join(NUMEROS)

    def leer(m: re.Match) -> int | None:
        """El número, venga en cifra o en palabra. `None` si no se reconoce."""
        bruto = m.group(1).lower()
        if bruto.isdigit():
            return int(bruto)
        # `.get(bruto, int(bruto))` parecía equivalente y no lo era: Python
        # evalúa el valor por defecto SIEMPRE, así que «ocho» reventaba en el
        # `int()` antes de llegar al diccionario.
        return NUMEROS.get(bruto)

    # Cada patrón: (nombre legible, expresión, valor real, pide_contexto)
    #
    # `pide_contexto` existe porque la primera versión era demasiado glotona:
    # cazaba «responde a tres preguntas en dos segundos» de UX_NAVEGACION y
    # «las cuatro preguntas que siguen» de un bloque de la consulta, que son
    # ciertas las dos. Un aviso falso repetido enseña a ignorar la
    # comprobación, que es la forma más rápida de que deje de servir.
    #
    # Así que una cifra de preguntas sólo cuenta como afirmación sobre el TOTAL
    # de la consulta cuando alrededor se habla de ella.
    CONTEXTO_JURIDICO = re.compile(r"jurídic|abogado|CONSULTA-JURIDICA", re.I)

    AFIRMACIONES = [
        (
            "migraciones",
            re.compile(rf"\b(\d+|{palabra})\s+migraciones\b", re.I),
            migraciones,
            None,
        ),
        (
            "preguntas de la consulta jurídica",
            re.compile(rf"\b(?:las\s+)?(\d+|{palabra})\s+preguntas\b", re.I),
            preguntas,
            CONTEXTO_JURIDICO,
        ),
    ]

    errados = []
    for doc in documentos():
        # El propio CHANGELOG narra la historia: «eran ocho, ahora son
        # dieciocho» es cierto como relato y falso como cifra actual. Un
        # registro de cambios que se reescribe para que cuadre con el presente
        # deja de ser un registro.
        if doc.name in ("CHANGELOG.md",):
            continue
        original = doc.read_text(encoding="utf-8")
        # Se busca sobre el texto ORIGINAL y no sobre el que tiene los bloques
        # de código quitados: quitarlos desplaza todas las líneas de abajo, y
        # el número que se imprimía no llevaba a ninguna parte. Los bloques se
        # saltan comprobando si la posición cae dentro de uno.
        bloques = [(m.start(), m.end()) for m in RE_CODIGO.finditer(original)]
        def en_bloque(i: int) -> bool:
            return any(a <= i < b for a, b in bloques)

        for nombre, patron, real, contexto in AFIRMACIONES:
            for m in patron.finditer(original):
                if en_bloque(m.start()):
                    continue
                dicho = leer(m)
                if dicho is None or dicho == real:
                    continue
                if citada(original, m.start()):
                    continue
                if contexto is not None:
                    # Una ventana de una frase a cada lado: lo justo para saber
                    # de qué se está hablando sin arrastrar el párrafo entero.
                    ventana = original[max(0, m.start() - 200) : m.end() + 200]
                    if not contexto.search(ventana):
                        continue
                linea = original[: m.start()].count("\n") + 1
                errados.append(
                    f"{doc.relative_to(RAIZ)}:{linea} dice «{m.group(0)}» "
                    f"y hay {real} {nombre}"
                )

    if errados:
        mal(f"{len(errados)} cifra(s) que ya no son ciertas", "\n      ".join(errados[:15]))
    else:
        bien(f"las cifras afirmadas coinciden ({migraciones} migraciones, {preguntas} preguntas)")

    # ── Los rangos de preguntas cuadran con su recuento ──────────────────
    #
    # «Nueve preguntas nuevas: C-9 a C-18» es falso: C-9 a C-18 son DIEZ. Es un
    # error de los que nadie ve, porque las dos mitades de la frase son
    # plausibles por separado y contarlas a mano es justo lo que nadie hace.
    rangos = []
    # La primera versión de esto pedía que el rango viniera pegado al número,
    # con una expresión que no sabía de asteriscos. Contra la frase real
    # —«**Nueve preguntas nuevas:** C-9 a C-18»— **no casaba**, así que la
    # comprobación decía «cuadran» sin haber mirado ninguno. Se descubrió
    # porque el error ya se conocía: si no, habría pasado por buena.
    #
    # Ahora se busca el número por un lado, el rango por otro, y se exige sólo
    # que estén cerca. Menos elegante y, a diferencia de la anterior, funciona.
    patron_cifra = re.compile(rf"\b(\d+|{palabra})\s+preguntas\b", re.I)
    patron_rango = re.compile(r"C-(\d+)\s+a\s+(?:la\s+)?C-(\d+)", re.I)
    for doc in documentos():
        original = doc.read_text(encoding="utf-8")
        for m in patron_cifra.finditer(original):
            dicho = leer(m)
            if dicho is None or citada(original, m.start()):
                continue
            # Ochenta caracteres: lo que cabe entre «N preguntas» y el rango en
            # la misma frase, sin alcanzar la siguiente.
            cerca = patron_rango.search(original[m.end() : m.end() + 80])
            if not cerca:
                continue
            cuantas = int(cerca.group(2)) - int(cerca.group(1)) + 1
            if dicho != cuantas:
                linea = original[: m.start()].count("\n") + 1
                rangos.append(
                    f"{doc.relative_to(RAIZ)}:{linea} dice «{m.group(0)}» pero "
                    f"de C-{cerca.group(1)} a C-{cerca.group(2)} hay {cuantas}"
                )
    if rangos:
        mal("hay rangos de preguntas que no cuadran con su recuento", "\n      ".join(rangos))
    else:
        bien("los rangos de preguntas cuadran con su recuento")

    if rapido:
        print(f"  {GRIS}–{FIN} recuento de pruebas de Rust: --rapido")
        return

    reales = contar_pruebas_de_rust()
    dichas = []
    patron = re.compile(r"\b(\d+)\s+(?:pruebas|tests)\s+de\s+Rust\b", re.I)
    # Una cifra ANCLADA a un momento es un registro, no una afirmación sobre
    # hoy: «133 pruebas de Rust al cerrar la 3.1» era cierto entonces y seguirá
    # siéndolo siempre. Reescribirla para que cuadre con el presente destruiría
    # justo lo que un documento de fase sirve para guardar.
    #
    # El ancla tiene que estar ESCRITA. Una cifra sin fecha ni «al cerrar» se
    # lee como el estado actual, y por eso se exige.
    ANCLADA = re.compile(
        r"al cerrar|en ese momento|entonces|hasta la (?:entrega|fase)|\b20\d\d\b",
        re.I,
    )
    for doc in documentos():
        if doc.name == "CHANGELOG.md":
            continue
        original = doc.read_text(encoding="utf-8")
        for m in patron.finditer(RE_CODIGO.sub("", original)):
            if int(m.group(1)) == reales or citada(original, m.start()):
                continue
            ventana = original[max(0, m.start() - 120) : m.end() + 120]
            if ANCLADA.search(ventana):
                continue
            dichas.append(f"{doc.relative_to(RAIZ)} dice {m.group(1)} y son {reales}")
    if dichas:
        mal("el recuento de pruebas de Rust está desfasado", "\n      ".join(dichas))
    else:
        bien(f"el recuento de pruebas de Rust es correcto ({reales})")


# ── 3 · Los anexos prometidos ────────────────────────────────────────────────


def no_se_promete_nada_que_no_exista() -> None:
    """Ningún documento ofrece un anexo que no se escribió.

    Es un defecto REAL de este proyecto: la consulta jurídica ofrecía dos
    anexos que no existían, y el abogado los habría pedido en la primera
    reunión. Se descubrió leyendo, que es la forma cara de descubrirlo.

    ROTO A PROPÓSITO: se añadió «Anexo C · el borrador que no existe» a la
    consulta jurídica y se cazó.
    """
    sospechosos = []
    patron = re.compile(r"^#{1,4}\s*Anexo\s+([A-Z0-9]+)\s*[·:—-]\s*(.+)$", re.M | re.I)
    for doc in documentos():
        texto = doc.read_text(encoding="utf-8")
        for m in patron.finditer(texto):
            cuerpo = texto[m.end():]
            # Un anexo de verdad trae contenido debajo, no sólo su título.
            siguiente = re.search(r"^#{1,4}\s", cuerpo, re.M)
            contenido = cuerpo[: siguiente.start()] if siguiente else cuerpo
            if len(contenido.strip()) < 120:
                sospechosos.append(
                    f"{doc.relative_to(RAIZ)} · «Anexo {m.group(1)} · {m.group(2)[:50]}» "
                    f"tiene {len(contenido.strip())} caracteres debajo"
                )
    if sospechosos:
        mal("hay anexos anunciados y vacíos", "\n      ".join(sospechosos))
    else:
        bien("ningún documento promete un anexo que no exista")


# ── 4 · Ningún identificador de modelo ───────────────────────────────────────


def ningun_identificador_de_modelo() -> None:
    """CLAUDE.md §1: ningún identificador de modelo en nada del repositorio.

    En el chat sí; en commits, cuerpos de PR, comentarios del código y
    documentación, no. Es una regla dura que hasta hoy no vigilaba nadie.

    ROTO A PROPÓSITO: se escribió un identificador de modelo en un comentario
    de `base.css` y en un documento, y se cazaron los dos.

    La firma de coautoría de los commits es la excepción escrita del propio
    CLAUDE.md, y los commits no son archivos: esto mira el árbol de trabajo.
    """
    # Se buscan familias y versiones, no la palabra suelta: «Claude Code» es el
    # nombre de la herramienta y está permitido en la atribución.
    patron = re.compile(
        r"\b(?:claude-(?:opus|sonnet|haiku|fable)[\w.-]*"
        r"|(?:opus|sonnet|haiku)\s*\d+(?:\.\d+)?)\b",
        re.I,
    )
    EXENTOS = {"CLAUDE.md"}
    encontrados = []
    r = subprocess.run(
        ["git", "ls-files"], cwd=RAIZ, capture_output=True, text=True, timeout=120
    )
    for relativo in r.stdout.splitlines():
        p = RAIZ / relativo
        if p.name in EXENTOS or p.suffix in (".png", ".jpg", ".pdf", ".woff2", ".ttf",
                                             ".eot", ".woff", ".zip", ".mp4", ".sha256"):
            continue
        try:
            texto = p.read_text(encoding="utf-8")
        except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
            continue
        for m in patron.finditer(texto):
            linea = texto[: m.start()].count("\n") + 1
            encontrados.append(f"{relativo}:{linea} · «{m.group(0)}»")
    if encontrados:
        mal(
            "hay identificadores de modelo en el repositorio (CLAUDE.md §1)",
            "\n      ".join(encontrados[:15]),
        )
    else:
        bien("ningún identificador de modelo en el repositorio")


# ── 5 · El índice cubre lo que hay ───────────────────────────────────────────


def el_indice_no_se_queda_atras() -> None:
    """Todo documento de `documentacion/` aparece en el índice.

    Un mapa que no nombra media biblioteca es peor que no tener mapa: da la
    impresión de completo. Los ADR se cuentan por su carpeta, que el índice
    nombra entera.

    ROTO A PROPÓSITO: se creó un documento nuevo sin añadirlo al índice y se
    cazó por su nombre.
    """
    indice = DOCS / "00-INDICE.md"
    if not indice.exists():
        mal("no existe documentacion/00-INDICE.md")
        return
    texto = indice.read_text(encoding="utf-8")
    ausentes = []
    for doc in sorted(DOCS.glob("**/*.md")):
        rel = doc.relative_to(DOCS).as_posix()
        if rel == "00-INDICE.md":
            continue
        # Los ADR y los informes de fase se nombran por carpeta.
        if rel.startswith("03-arquitectura/adr/") and "03-arquitectura/adr/" in texto:
            continue
        if doc.name in texto or rel in texto:
            continue
        ausentes.append(rel)
    if ausentes:
        mal(
            f"{len(ausentes)} documento(s) que el índice no nombra",
            "\n      ".join(ausentes),
        )
    else:
        bien("el índice nombra todos los documentos")


def main() -> int:
    rapido = "--rapido" in sys.argv
    print(f"\n{GRIS}La documentación dice la verdad{FIN}\n")
    los_enlaces_llevan_a_alguna_parte()
    las_cifras_afirmadas_son_ciertas(rapido)
    no_se_promete_nada_que_no_exista()
    ningun_identificador_de_modelo()
    el_indice_no_se_queda_atras()

    print()
    if fallos:
        for f in fallos:
            print(f"  {ROJO}✗{FIN} {f}")
        print(f"\n{len(fallos)} problema(s) en la documentación.\n")
        return 1
    print("La documentación dice la verdad.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
