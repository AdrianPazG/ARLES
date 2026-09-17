#!/usr/bin/env python3
"""Calcula el avance de ARLES y lo escribe en la portada del repositorio.

    python3 herramientas/avance/calcular.py              # sólo lo imprime
    python3 herramientas/avance/calcular.py --escribir    # actualiza el README

─────────────────────────────────────────────────────────────────────────────
POR QUÉ ESTO ES UN GENERADOR Y NO DOS NÚMEROS ESCRITOS A MANO

Dirección lo quiere **sólo en la portada del repositorio en GitHub**, no dentro
de la aplicación (decidido el 17/09/2026; la primera versión lo puso en los dos
sitios y se retiró de la pantalla).

Aun siendo un solo destino, el número vive en
`documentacion/07-entrega/avance.json` y la insignia se **genera**: escrito a
mano en el README, se edita ahí y el desglose de fases de debajo se queda con
los valores del mes pasado, contradiciendo a la propia insignia. El validador
recalcula y **falla** si el README y la fuente dejan de coincidir.

Es la misma regla del §17 con los colores: una fuente, y lo demás generado.

Y hay un motivo para que el porcentaje NO esté dentro de la aplicación: es un
dato **del proyecto**, no del producto. A quien use ARLES no le sirve saber que
está al 31 %; le sirve saber qué puede hacer hoy, que es lo que dice la lista de
alta. En el repositorio, en cambio, es exactamente la pregunta que se viene a
responder.
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


def main() -> int:
    p = argparse.ArgumentParser(description="Avance de ARLES RELAY I")
    p.add_argument("--escribir", action="store_true", help="actualiza el README")
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
        print("\n  " + ("actualizado: " + ", ".join(cambios) if cambios
                        else "nada que actualizar: ya coincidían"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
