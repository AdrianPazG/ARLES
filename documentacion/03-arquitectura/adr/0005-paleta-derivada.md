# ADR-0005 · Paleta derivada: extraer el cian y el oro, construir los profundos

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Dirección (D-2)

---

## Contexto

El brief establece que la paleta debe extraerse de `/REFERENCIA_DE_COLOR` (§13), con *La noche estrellada* de Van Gogh como concepto inspirador (§14), y asigna roles concretos (§16): azul profundo para la estructura, azul medio para las superficies, cian para la interacción, amarillo girasol para los acentos.

La auditoría decodificó el archivo completo —que resultó ser un PNG de 2320×3080 renombrado a `.jpg`— y midió la distribución real:

| Familia | Rango | Presencia |
|---|---|---:|
| Cian / azur | H 185–210° | **54.35 %** |
| Ocre / naranja | H 15–38° | 19.48 % |
| Amarillo / oro | H 38–62° | 13.44 % |
| **Azul profundo** | **H 210–250°** | **0.73 %** |

Luminosidad: el 76 % de los píxeles entre L 20 y L 60; sólo el 1 % por debajo de L 10.

**No es una paleta de *La noche estrellada*.** El azul profundo, que es el color que define el cuadro, está ausente. La imagen es una composición complementaria cian-contra-cálido, más cercana a un paisaje al atardecer que a un cielo nocturno. Además está marcada como generada por IA (`trainedAlgorithmicMedia`) y procede de un banco de imágenes, no de TELEMETRY.

Y el dato de accesibilidad que cierra la discusión:

| Superficie | `#F4ECE4` | `#FCCC0C` | Blanco |
|---|---|---|---|
| `#2CA4D4` cian vivo | 2.44:1 ❌ | 1.87:1 ❌ | 2.85:1 ❌ |

## Decisión

**Se extraen el cian, el amarillo y los cremas reales. Se construyen por rampa los azules profundos que faltan.**

La imagen queda como **referencia interna de estudio**: nunca se distribuye dentro del producto ni se presenta como activo de marca.

## Justificación

**Extraer literalmente los tres roles del §16 los colapsa.** El 54 % de la imagen cabe en una banda de ±12°. Asignar «estructura», «superficie» e «interacción» a tres puntos de esa banda produce una interfaz monocromática cian donde **nada distingue visualmente el chasis de lo pulsable**. La jerarquía desaparece.

**El fondo profundo no existe en la imagen.** El §18 pide dark-first construido por capas de azul. Los oscuros disponibles son o bien cian apagado (`#041C2C`) o bien tierras cálidas turbias (`#341C14`). Ninguno sirve tal cual.

**Construir las rampas no traiciona la referencia: es lo único que la hace usable.** La referencia aporta el carácter cromático —ese cian concreto, ese amarillo concreto, esos cremas— y la rampa aporta la estructura que el sistema necesita para tener jerarquía y pasar AA.

**Legalmente es limpio.** Los colores y sus combinaciones no son protegibles por derecho de autor. Extraer una paleta de una imagen de stock generada por IA no crea ninguna obligación. Lo que no se puede hacer —y no se hará— es distribuir el archivo o presentarlo como activo propio.

## Las tres reglas que salen de los datos

1. **`#2CA4D4` nunca es relleno de una superficie con texto.** Falla WCAG contra todo, blanco incluido (2.85:1). Es color de **trazo, borde y foco**.

2. **El amarillo es luz, no señal.** `#FCCC0C` tiene luminancia relativa **0.639** — más cerca del blanco (1.0) que del negro. Brilla sobre `#041C2C` (11.41:1) y es inservible con texto blanco encima (1.56:1). Invierte la intuición habitual: no es «color fuerte sobre claro», es **luz sobre oscuro**. Encaja perfectamente con el dark-first y prohíbe de raíz el patrón «botón amarillo, texto blanco».

3. **La información nunca se comunica sólo por color** (§19). Todo estado lleva forma, icono o texto además del color.

## Consecuencias

**Positivas.** Jerarquía real entre estructura, superficie e interacción · WCAG 2.2 AA alcanzable y verificado por cálculo, no por impresión · sin dependencia legal de un asset de terceros · la paleta queda **reproducible**: el método de extracción está documentado y cualquiera puede recalcular las cifras.

**Negativas.**
- Los azules profundos **no aparecen literalmente en la referencia**. Alguien que compare la imagen con la interfaz notará que el fondo es más oscuro. Es deliberado y está documentado aquí.
- Requiere disciplina de tokens: ningún color se escribe a mano en un componente (§17).

## Alternativas descartadas

**Usar la referencia literalmente.** Fiel al brief y contradicho por los datos: UI monocromática sin jerarquía, y `#2CA4D4` fallando AA como superficie. Habría que elegir entre la fidelidad a la referencia y la accesibilidad, y el §19 ya dice cuál gana.

**Entregar un brandbook completo antes de la Fase 2.** Lo más sólido comercialmente —identidad genuinamente propiedad de TELEMETRY, sin stock de IA en el linaje de la marca— pero bloquea el Design System indefinidamente. **Sigue siendo la recomendación para v1.3** y está registrada en P-02.

**Encargar un asset cromático original.** Elimina la contradicción con T-9 sin frenar tanto como un brandbook. Buena opción intermedia si Dirección quiere limpiar el linaje del asset más adelante.

**Usar la paleta real de *La noche estrellada*.** Contradice el §14, que es explícito: el cuadro aporta concepto y atmósfera; la **referencia** define los colores.

## Referencias

`05-diseno/COLOR_SYSTEM.md` (método, rampas y tabla completa de contraste) · `02-auditoria/INVENTARIO_DE_ASSETS.md` (hallazgos A-02 a A-05)
