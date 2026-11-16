# Plantilla · cuando hace falta que Dirección elija

**Cómo se usa:** se copia bajo la línea y se rellena. Si una sección sale vacía,
**la decisión no está lista para plantearse** — no se plantea a medias.

**Dónde va:** `10-requisitos/REGISTRO-DE-REQUISITOS.md` como una línea con
estado «espera decisión», y el desarrollo entero en `09-fases/` o
`01-producto/` según de qué sea.

---

## La forma, y por qué es ésta

Dirección ha pedido muchas veces no recibir opiniones sueltas, y la razón es
práctica: una opinión no se puede evaluar. Esta estructura se puede.

    Problema → Riesgo → Impacto → Alternativa → Trade-off → Recomendación

**Y lleva recomendación siempre.** Presentar tres alternativas sin decir cuál y
por qué es devolver el trabajo: quien conoce el código es quien puede estimar
el coste de cada una.

---

# [Qué hay que decidir] · para Dirección

**Fecha:** AAAA-MM-DD · **Identificador:** [D-xx / R-xx]

## El problema

*Qué pasa hoy, con una cifra si la hay. Sin interpretarlo todavía.*

## El riesgo

*Qué puede salir mal si se queda así. **Y con qué probabilidad**, si se puede
estimar: «se rompe alrededor de los 30 000 destinatarios, que es una lista
ordinaria» dice mucho más que «puede fallar con listas grandes».*

## El impacto

*A quién le pasa y qué cuesta. Si es dinero, en pesos. Si es tiempo, en horas.
Si es reputación de un número o de un dominio, se dice que **no se recupera**,
porque es la diferencia que decide.*

## Las alternativas

*Dos o tres. Cada una con lo que cuesta construirla —horas o días— y lo que
cuesta mantenerla.*

**A ·** *Qué sería.* Coste:
**B ·** *Qué sería.* Coste:
**C ·** *No hacer nada.* Coste: *casi siempre hay que ponerla, porque a veces es
la correcta y nombrarla evita que se elija por omisión.*

## El trade-off

*Qué se gana y qué se pierde con cada una, en la misma unidad. Si una es más
rápida y otra más segura, se dice cuánto de cada cosa.*

## Mi recomendación

*Una, y por qué. Si la recomendación es la que cuesta más, se dice qué pasa si
se elige la barata.*

## Lo que pasa si no se decide ahora

*Qué se queda bloqueado y qué se puede seguir construyendo. **Si no bloquea
nada, se dice**: una decisión que no corre prisa no debe presentarse como si
corriera.*

---

> **Una vez decidido**, se anota en el registro de requisitos con la fecha y en
> una sola línea, y si cambia cómo se construye se escribe también en
> `CLAUDE.md`. Una decisión que vive sólo en una conversación se vuelve a
> discutir dentro de tres semanas — es la razón por la que `CLAUDE.md` existe.
