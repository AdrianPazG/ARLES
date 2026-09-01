# ADR-0006 — Sin seguimiento de aperturas ni de clics

**Estado:** Propuesto · **Fecha:** 2026-09-01

## Contexto
§67 y §68 piden cuestionar si aportan valor suficiente. Son funciones estándar en la categoría y su ausencia
será notada comercialmente.

## Decisión
**No implementar ninguna de las dos en v1.2.0**, dejar la arquitectura preparada, y **declarar la ausencia
como postura de producto** en lugar de omitirla.

## Razones

**Aperturas.** El píxel de seguimiento ya no distingue una lectura de una precarga automática: Apple Mail
Privacy Protection precarga las imágenes y los proxies corporativos también. El dato resultante no significa
lo que el usuario cree que significa, y §156 prohíbe mostrar métricas que no podemos verificar.

**Clics.** Exige reescribir las URLs por un redireccionador propio. Eso implica (a) alojar el
redireccionador — el backend que se quiere evitar; (b) **daña la entregabilidad**, porque el dominio del
enlace deja de coincidir con el del remitente, que es señal de spam; (c) contradice §146.

## Consecuencias

**Positivas.** Coherencia con el principio de honestidad, que es el diferenciador del producto. Menos
infraestructura, menos superficie de privacidad, menos riesgo de entregabilidad. "No fingimos métricas" es
argumento de venta.

**Negativas.** Ausencia visible frente a competidores en comparativas de funciones. Se mitiga explicándola:
el material comercial debe decir *por qué* no existen, no esconder que faltan.

**Preparación futura.** `MessageEvent` admite tipos de evento adicionales sin cambio de esquema. Si alguna
vez se implementa, deberá ser configurable, documentar sus límites y no presentarse como verdad absoluta.
