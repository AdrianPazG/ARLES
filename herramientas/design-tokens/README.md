# Generador de tokens de diseño

Fuente única de los colores de ARLES RELAY. Se edita `tokens.json` y todo lo demás se regenera.

```bash
./generar.py --verificar   # comprueba contraste WCAG — para CI
./generar.py --css         # escribe dist/arles-tokens.css
./generar.py --lamina      # renderiza la lámina PNG
./generar.py --todo        # las tres cosas
```

Sin dependencias externas: solo stdlib de Python 3. Chromium hace falta únicamente para `--lamina`.

---

## Qué produce

| Artefacto | Destino | Para qué |
|---|---|---|
| CSS de tokens | `dist/arles-tokens.css` | Lo consume el frontend. Sustituye a escribir hex a mano (§17) |
| Lámina de la paleta | `/REFERENCIA_DE_COLOR/ARLES_RELAY-paleta-v1.2.0.png` | Referencia visual para diseño |
| Verificación WCAG | salida por consola, código de salida | Rompe la compilación ante una regresión de contraste |

La fuente de verdad **conceptual** sigue siendo [`COLOR_SYSTEM.md`](../../documentacion/05-diseno/COLOR_SYSTEM.md), que explica de dónde sale cada color y por qué. Este directorio es la fuente de verdad **operativa**: los valores que la aplicación consume.

---

## Por qué existe

`COLOR_SYSTEM.md` §6 y `ESTRATEGIA_QA.md` §6 prometían que los ratios de contraste se calcularían en CI y que una regresión rompería la compilación. Esta herramienta es ese compromiso hecho código, en vez de una buena intención en un documento.

Sin ella, cambiar un color significa: editar el markdown, recalcular los ratios a mano, actualizar tres tablas y volver a exportar la lámina. Cuatro oportunidades de que la documentación y la aplicación dejen de coincidir.

---

## Añadir o cambiar un color

1. Edita `tokens.json`.
2. Si el color aparece en texto o interfaz, añade su **contrato** en `contratos`.
3. `./generar.py --todo`.
4. Si algo falla, el color no entra hasta arreglarlo.
5. Actualiza `COLOR_SYSTEM.md` si cambia una regla, no solo un valor.

El alto de la lámina se recalcula solo según el número de colores por grupo, así que añadir uno no la deja cortada.

---

## Contratos y advertencias

Las dos listas de `tokens.json` que hacen que la verificación sea útil.

**`contratos`** — pares que **deben** pasar. Protegen contra regresiones: si alguien aclara `--arles-surface-raised` un poco «para que se vea mejor», el texto sobre tarjetas puede caer por debajo de 4.5:1 y la compilación lo detiene.

```json
{ "fondo": "--arles-surface-raised", "frente": "--arles-text",
  "min": 4.5, "por_que": "Tarjetas y modales llevan texto normal" }
```

**`advertencias`** — pares que **deben seguir fallando**. Esto es menos obvio y por eso importa: `COLOR_SYSTEM.md` afirma que el amarillo con texto blanco encima es ilegible (1.52:1) y que por eso ese patrón está prohibido. Si alguien oscureciera el amarillo lo suficiente, esa afirmación dejaría de ser cierta y **la documentación quedaría obsoleta sin que nadie se entere**.

```json
{ "fondo": "--arles-accent", "frente": "#FFFFFF", "max": 4.5,
  "regla": "COLOR_SYSTEM.md §7.2",
  "por_que": "El amarillo es luz, no señal..." }
```

La verificación comprueba la documentación **en las dos direcciones**: que lo que debe pasar pase, y que lo que se documenta como imposible siga siéndolo.

---

## Integración en CI

Corresponde al paso 5 del pipeline de `ESTRATEGIA_QA.md` §6:

```yaml
- name: Contraste de tokens de diseño
  run: python3 herramientas/design-tokens/generar.py --verificar
```

Código de salida `0` si todo pasa, `1` con el detalle de cada fallo.

---

## Chromium

`--lamina` lo busca en este orden: `$CHROMIUM_BIN` · la ruta de Playwright · `/usr/bin/chromium` · `chromium-browser` · `google-chrome` · Chrome de macOS.

```bash
CHROMIUM_BIN=/ruta/al/chrome ./generar.py --lamina
```

La lámina se renderiza a 2× (2800 px de ancho) con **Mont incrustada en base64**, así que el resultado es idéntico en cualquier máquina, tenga la fuente instalada o no.

> **Nota de licencia.** Aquí Mont se usa para **componer un documento**, que es uso cubierto por una licencia de escritorio. Es un supuesto distinto del de incrustar el binario de la fuente en la aplicación distribuida, que sigue pendiente en D-3.

---

## Lo que esta herramienta no hace

- **No decide colores.** Esos salen de la medición documentada en `COLOR_SYSTEM.md` §6 y de ADR-0005.
- **No genera un tema claro.** Fuera de alcance en v1.2.0.
- **No valida el uso de los tokens en los componentes.** Que exista `--arles-accent` no impide usarlo en veinte sitios de la misma pantalla; eso lo vigila la auditoría de UX (§140-D).
