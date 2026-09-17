#!/usr/bin/env python3
"""Deja ARLES sin Mont, para poder distribuir una versión de prueba.

    python3 herramientas/marca/sin-mont.py           # avisa de qué haría
    python3 herramientas/marca/sin-mont.py --aplicar # lo hace

─────────────────────────────────────────────────────────────────────────────
POR QUÉ EXISTE ESTO

Mont es de Fontfabric. Las licencias Desktop y Web **no cubren incrustar el
binario de la fuente en una aplicación que se distribuye**; eso exige una App
License aparte, y TELEMETRY todavía no ha confirmado cuál tiene (P-01).

Mientras eso no se resuelva, cualquier instalador que salga del equipo de
desarrollo tiene que ir **sin Mont**. Es el plan B de `TIPOGRAFIA.md` §3, y
consiste en quitar las `@font-face`: `--arles-font-family` ya declara Figtree y
después las fuentes del sistema, así que la aplicación sigue funcionando entera
—sólo que con otra letra—.

!! **Lo que esto cambia y hay que saber al revisar:** la tipografía **no es la
definitiva**. Los tamaños, los pesos y el ritmo vertical sí lo son, porque salen
de los tokens; lo que cambia es la forma de las letras. Un hallazgo del tipo «la
tipografía no es la de la marca» en una versión generada con este script es
esperado, no un defecto.

Este script **modifica el árbol de trabajo**. Lo usa el flujo de entrega de
prueba sobre una copia recién clonada, nunca sobre el repositorio de nadie.
─────────────────────────────────────────────────────────────────────────────
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
TIPOGRAFIA = RAIZ / "app" / "src" / "design" / "tipografia.css"

SUSTITUTO = """/* GENERADO por herramientas/marca/sin-mont.py — NO COMMITEAR.
 *
 * Esta versión se construyó **sin Mont**, porque la licencia para incrustarla
 * en una aplicación distribuida sigue sin confirmarse (P-01). Es el plan B de
 * TIPOGRAFIA.md §3.
 *
 * No hay `@font-face`: `--arles-font-family` cae en Figtree y, si no está, en
 * la tipografía del sistema. Todo lo demás —tamaños, pesos, interlineado— sale
 * de los tokens y es idéntico a la versión con Mont.
 */
"""


def main() -> int:
    p = argparse.ArgumentParser(description="Quita Mont del bundle (plan B de P-01)")
    p.add_argument("--aplicar", action="store_true", help="escribe el cambio")
    args = p.parse_args()

    if not TIPOGRAFIA.exists():
        print(f"no encuentro {TIPOGRAFIA.relative_to(RAIZ)}", file=sys.stderr)
        return 1

    texto = TIPOGRAFIA.read_text(encoding="utf-8")
    caras = len(re.findall(r"@font-face", texto))
    if caras == 0:
        print("Ya estaba sin Mont: no hay ninguna @font-face.")
        return 0

    print(f"Se quitarían {caras} declaraciones @font-face de "
          f"{TIPOGRAFIA.relative_to(RAIZ)}.")
    print("La aplicación usará Figtree o la tipografía del sistema.")

    if not args.aplicar:
        print("\n(nada escrito; añade --aplicar)")
        return 0

    TIPOGRAFIA.write_text(SUSTITUTO, encoding="utf-8")
    print("\n✓ Aplicado. Este árbol ya NO sirve para commitear.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
