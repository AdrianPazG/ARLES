#!/usr/bin/env python3
"""Calcula el avance de ARLES y lo propaga a los dos sitios donde se enseña.

    python3 herramientas/avance/calcular.py              # sólo lo imprime
    python3 herramientas/avance/calcular.py --escribir    # actualiza README y app

─────────────────────────────────────────────────────────────────────────────
POR QUÉ ESTO ES UN GENERADOR Y NO DOS NÚMEROS ESCRITOS A MANO

Dirección pidió ver el porcentaje **en GitHub y en la pantalla de Inicio**. Son
dos sitios, y dos sitios con el mismo número escrito a mano son dos números que
divergen en la primera prisa: se actualiza el del README al cerrar una fase y el
de la pantalla se queda con el de hace un mes, sin que nadie se entere.

Aquí el número vive en `documentacion/07-entrega/avance.json` y **sólo ahí**. De
él salen la insignia del README y el archivo que lee la pantalla. El validador
recalcula y **falla** si los tres dejan de coincidir, así que la divergencia
deja de ser silenciosa.

Es la misma regla del §17 con los colores: una fuente, y lo demás generado.
─────────────────────────────────────────────────────────────────────────────
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
FUENTE = RAIZ / "documentacion" / "07-entrega" / "avance.json"
README = RAIZ / "README.md"
DESTINO_APP = RAIZ / "app" / "src" / "app" / "generado" / "avance.ts"

#: Marcas dentro del README. Se sustituye lo de en medio y nada más: así el
#: resto de la portada se edita a mano sin que este script la pise.
INICIO = "<!-- AVANCE:INICIO -->"
FIN = "<!-- AVANCE:FIN -->"


def cargar() -> dict:
    datos = json.loads(FUENTE.read_text(encoding="utf-8"))

    pesos = sum(f["peso"] for f in datos["fases"])
    if pesos != 100:
        raise SystemExit(
            f"Los pesos de las fases suman {pesos} y tienen que sumar 100. "
            "Si no, el porcentaje no es un porcentaje de nada."
        )
    for f in datos["fases"]:
        if not 0.0 <= f["hecho"] <= 1.0:
            raise SystemExit(f"{f['id']}: «hecho» tiene que estar entre 0 y 1.")
        if not f.get("porque", "").strip():
            raise SystemExit(
                f"{f['id']}: falta «porque». Una fracción sin nada que la "
                "sostenga es una opinión disfrazada de medición (§94)."
            )
    return datos


def porcentaje(datos: dict) -> int:
    return round(sum(f["peso"] * f["hecho"] for f in datos["fases"]))


def color(pct: int) -> str:
    """El color de la insignia. Tramos anchos: no pretende ser un semáforo."""
    if pct >= 80:
        return "brightgreen"
    if pct >= 50:
        return "yellow"
    return "orange"


def bloque_readme(datos: dict, pct: int) -> str:
    filas = "\n".join(
        f"| {f['id']} · {f['nombre']} | {f['peso']} % | "
        f"{'✅' if f['hecho'] >= 1 else ('🟨' if f['hecho'] > 0 else '⬜')} "
        f"{round(f['hecho'] * 100)} % |"
        for f in datos["fases"]
    )
    bloqueos = "\n".join(
        f"| **{b['id']}** | {b['que']} | {b['quien']} | {b['bloquea']} |"
        for b in datos["bloqueos"]
    )
    return f"""{INICIO}
![Avance de la v1.2.0](https://img.shields.io/badge/avance_v1.2.0-{pct}%25-{color(pct)}?style=flat-square)

**{pct} % de la versión 1.2.0 construido y verificado**, a {datos['actualizado']}.

<details>
<summary>Cómo sale ese número, y qué no mide</summary>

Suma de **peso × hecho** sobre las fases de abajo. Los **pesos** son una
estimación del tamaño relativo de cada fase y suman 100; se dice que son una
estimación porque presentarlos como medición exacta sería mentir. Lo que **no**
es estimación es la fracción hecha: cada una se apoya en algo comprobable, y esa
comprobación está escrita en
[`avance.json`](documentacion/07-entrega/avance.json).

**No mide si el producto sirve**, sino cuánto del alcance de la v1.2.0 está
construido *y verificado*. Los bloqueos externos no restan porcentaje, porque no
son trabajo pendiente nuestro: van aparte.

| Fase | Peso | Hecho |
|---|---|---|
{filas}

**Esperando a alguien de fuera:**

| | Qué | Quién | Qué frena |
|---|---|---|---|
{bloqueos}

El número se genera; no se escribe a mano en dos sitios:

```bash
python3 herramientas/avance/calcular.py --escribir
```

</details>
{FIN}"""


def escribir_readme(datos: dict, pct: int) -> bool:
    texto = README.read_text(encoding="utf-8")
    nuevo = bloque_readme(datos, pct)
    if INICIO in texto and FIN in texto:
        texto2 = re.sub(
            re.escape(INICIO) + r".*?" + re.escape(FIN),
            lambda _: nuevo,
            texto,
            flags=re.S,
        )
    else:
        raise SystemExit(
            f"No encuentro las marcas {INICIO} … {FIN} en el README. "
            "Sin ellas no sé dónde poner la insignia sin pisar la portada."
        )
    if texto2 == texto:
        return False
    README.write_text(texto2, encoding="utf-8")
    return True


def escribir_app(datos: dict, pct: int) -> bool:
    fases = ",\n".join(
        "  {{ id: {id!r}, nombre: {nombre!r}, hecho: {hecho} }}".format(
            id=f["id"], nombre=f["nombre"], hecho=round(f["hecho"], 3)
        ).replace("'", "'")
        for f in datos["fases"]
    )
    # Comillas simples, que es lo que impone el lint del frontend.
    fases = fases.replace('"', "'")
    contenido = f"""/**
 * GENERADO. No editar a mano.
 *
 * Sale de `documentacion/07-entrega/avance.json` con
 * `python3 herramientas/avance/calcular.py --escribir`.
 *
 * Dirección pidió ver el avance en GitHub y en la pantalla de Inicio. Son dos
 * sitios; dos números escritos a mano divergen en la primera prisa. El
 * validador recalcula y falla si este archivo, el README y la fuente dejan de
 * coincidir.
 */
export const AVANCE_PORCENTAJE = {pct}

/** La fecha del dato, no la de hoy: un porcentaje sin fecha no dice nada. */
export const AVANCE_ACTUALIZADO = '{datos["actualizado"]}'

export interface FaseDeAvance {{
  id: string
  nombre: string
  /** Fracción de 0 a 1. */
  hecho: number
}}

export const AVANCE_FASES: readonly FaseDeAvance[] = [
{fases},
]
"""
    DESTINO_APP.parent.mkdir(parents=True, exist_ok=True)
    if DESTINO_APP.exists() and DESTINO_APP.read_text(encoding="utf-8") == contenido:
        return False
    DESTINO_APP.write_text(contenido, encoding="utf-8")
    return True


def main() -> int:
    p = argparse.ArgumentParser(description="Avance de ARLES RELAY I")
    p.add_argument("--escribir", action="store_true", help="actualiza README y app")
    args = p.parse_args()

    datos = cargar()
    pct = porcentaje(datos)

    print(f"ARLES RELAY I v{datos['version']} · avance {pct} %  ({datos['actualizado']})")
    for f in datos["fases"]:
        barra = "█" * round(f["hecho"] * 20) + "·" * (20 - round(f["hecho"] * 20))
        print(f"  {f['id']:<4} {barra} {round(f['hecho'] * 100):>3} %  "
              f"peso {f['peso']:>2}  {f['nombre']}")
    if datos["bloqueos"]:
        print("\n  esperando a alguien de fuera:")
        for b in datos["bloqueos"]:
            print(f"    {b['id']:<6} {b['que']}  →  {b['quien']}")

    if args.escribir:
        cambios = []
        if escribir_readme(datos, pct):
            cambios.append("README.md")
        if escribir_app(datos, pct):
            cambios.append(str(DESTINO_APP.relative_to(RAIZ)))
        print("\n  " + ("actualizado: " + ", ".join(cambios) if cambios
                        else "nada que actualizar: ya coincidían"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
