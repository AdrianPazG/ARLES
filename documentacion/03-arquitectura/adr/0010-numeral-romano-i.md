# ADR-0010 · Retención del numeral «I» y su presentación

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Dirección (T-5)

---

## Contexto

El nombre oficial del producto es **ARLES RELAY I** y la versión objetivo es **v1.2.0**. El brief pide cuestionar si «I» es el mejor uso de ese numeral (§2), y T-5 lo confirma como identificador comercial.

## El problema

«ARLES RELAY I / v1.2.0» pone **dos significantes de generación en la misma línea**.

El lector natural —un cliente, un comercial, alguien que ve una captura— asume que «I» significa «versión 1» y entonces se pregunta por qué el número dice 1.2.0. La pregunta no tiene respuesta satisfactoria, porque «I» y «1» efectivamente compiten por significar lo mismo.

El riesgo es bajo pero **permanente**: aparece en cada pantalla, en cada instalador y en cada documento comercial durante toda la vida del producto.

## Decisión

**Se retiene «I»** como identificador comercial, según T-5.

**Pero no se muestra adyacente al número de versión.**

| Contexto | Qué se muestra |
|---|---|
| Marca, empaque, material comercial | **ARLES RELAY I** |
| Logotipo | **ARLES RELAY** (§21 — el logotipo es sólo tipográfico) |
| Título de ventana, encabezado de la aplicación | **ARLES RELAY** |
| Pie de la aplicación | `v1.2.0` |
| Diálogo «Acerca de» | ARLES RELAY I<br>Versión 1.2.0 (build 2026.09.11)<br>Software desarrollado por TELEMETRY INSIGHT |
| Nombre del instalador | `ArlesRelay-1.2.0-x64.exe` |

La regla, en una línea: **«I» y el número de versión nunca aparecen en el mismo renglón.** Separados por una línea o por un contexto distinto, cada uno significa lo suyo sin competir.

## Justificación

**Retenerlo respeta T-5**, que es una decisión comercial de Dirección y no técnica.

**Separarlos elimina el conflicto sin cambiar nada.** «I» funciona bien como identificador de edición o generación cuando está solo. El conflicto no es el numeral: es la adyacencia.

**El §3 exige que la aplicación muestre claramente su versión.** Lo hace, en el pie y en «Acerca de», con el número completo.

## Consecuencias

**Positivas.** Se mantiene la identidad comercial · desaparece la ambigüedad · la versión sigue siendo visible e inequívoca.

**Negativas.** Requiere disciplina: cada vez que alguien escriba el nombre en una plantilla, un documento o una captura, hay que recordar la regla. **Mitigación:** la regla está en `05-diseno/UX_WRITING.md` y el nombre visible sale de una única constante en el código, no escrito a mano en cada sitio.

## Alternativas descartadas

**Eliminar «I».** La opción más limpia tipográfica y conceptualmente. Descartada por T-5: es una decisión comercial de Dirección.

**Renumerar a v1.0.0 para que «I» coincida con la versión.** Coherente, pero contradice T-4 —el primer lanzamiento es v1.2.0— y perdería el historial de versionado interno.

**Usar «ARLES RELAY ONE».** Deshace la ambigüedad visual con el número, pero sigue sonando a «versión uno» y además cambia la marca, lo que T-5 no autoriza.

**Reinterpretar «I» como inicial de otra cosa** (Intelligence, Infrastructure…). Sería inventar significado a posteriori para justificar una letra. No mejora nada.

## Recomendación para v2

Cuando llegue la siguiente generación mayor del producto, conviene decidir explícitamente si el numeral avanza a «II» o si se abandona. **Un numeral de generación sólo aporta valor si de verdad avanza**; si ARLES RELAY I sigue llamándose así en v3.4.0, el «I» habrá dejado de significar algo y será simplemente ruido en el nombre.
