---
name: entrevistador
description: Conduce entrevistas estructuradas de requisitos y decisiones antes de construir algo. Investiga primero, luego pregunta de una en una, y cada pregunta llega con recomendación propia, alternativas y trade-off — nunca como pregunta abierta vacía. Cierra con un registro de decisiones. Úsala siempre que aparezca cualquiera de estas situaciones, aunque nadie diga la palabra "entrevista": el usuario pide levantar requisitos, definir alcance, aclarar un brief ambiguo, hacer discovery o kickoff, "entrevístame", "pregúntame lo que necesites", "hazme las preguntas que falten", "¿qué necesitas saber para empezar?"; hay que decidir entre varias opciones técnicas o de producto y falta contexto del negocio; un documento o brief tiene huecos, contradicciones o supuestos sin confirmar; o hay que consultar a Dirección, a un stakeholder o a un cliente antes de escribir código. También aplica en inglés: interview, requirements gathering, stakeholder interview, discovery session, scoping, clarifying questions, product decisions.
---

# Entrevistador

## Por qué existe esta skill

Una entrevista mal hecha es peor que no hacerla. Los dos fracasos típicos:

1. **La ráfaga de veinte preguntas.** Se manda un cuestionario enorme, la persona contesta las tres primeras
   con cuidado, las demás con desgana, y las respuestas no sirven. Peor: varias de esas preguntas tenían
   respuesta en el repositorio, y preguntarlas comunica que no se revisó nada.
2. **La pregunta abierta vacía.** «¿Qué tecnología prefieres?», «¿Cómo te gustaría que funcionara?».
   Traslada el trabajo de decidir a quien esperaba criterio técnico. Si tienes elementos para recomendar,
   preguntar sin recomendar es abdicar.

Esta skill existe para lo contrario: **llegar preparado, preguntar poco, y que cada pregunta traiga
una recomendación defendible**. La persona entrevistada debería poder responder «sí, adelante» a la mitad
de las preguntas y concentrar su energía en la otra mitad.

## El flujo

```
Investigar  →  Clasificar  →  Preguntar  →  Escuchar  →  Registrar
```

---

## 1 · Investigar antes de preguntar

Esto no es opcional y no es una formalidad. **Cada pregunta que hagas cuya respuesta estaba disponible
te cuesta credibilidad en las que sí importan.** Antes de formular nada:

- Lee el material que ya te dieron: brief, documentos, tickets, correos, el historial de esta conversación.
- Explora el repositorio de verdad: estructura, README, configuración, código existente, historial de git,
  decisiones ya tomadas. Lo que ya está construido es una respuesta.
- Busca convenciones internas: `CLAUDE.md`, ADRs, documentos de arquitectura, guías de estilo.
- Si la decisión depende de un hecho externo verificable (una política de un proveedor, un límite de una
  API, el estado de una biblioteca), **verifícalo tú**. No conviertas un dato comprobable en una pregunta.

Al terminar la investigación deberías poder decir en voz alta: *«de las quince cosas que no sabía,
resolví once. Estas cuatro requieren a una persona.»* Si tu lista de preguntas no bajó durante la
investigación, la investigación no ocurrió.

**Señal de alarma:** si estás a punto de preguntar algo porque analizarlo tú sería laborioso, no es una
pregunta de entrevista — es trabajo que estás delegando. Haz el análisis.

---

## 2 · Clasificar: qué merece una pregunta

No todas las incógnitas son iguales. Ordénalas en tres cubetas y trata cada una distinto:

| Tipo | Qué es | Qué hacer |
|---|---|---|
| **Bloqueante** | No puedes avanzar sin la respuesta, o avanzar con el supuesto equivocado invalida el trabajo. Suele ser legal, contractual, de acceso, o una restricción del negocio que no puedes deducir. | **Pregunta.** Va primero. |
| **Estructural** | Puedes avanzar, pero cambiar de opinión después sale caro: modelo de datos, contrato de API, identidad de marca, modelo de licenciamiento. | **Pregunta, con recomendación fuerte.** |
| **Reversible** | Nombres de variables, orden de una lista, detalles de estilo, cosas que se cambian en una tarde. | **No preguntes.** Decide, avanza y menciónalo al pasar. |

Regla práctica: **si el costo de equivocarse es menor que el costo de interrumpir a la persona, no preguntes.**
Interrumpir tiene un costo real y se agota.

Apunta a **entre tres y seis preguntas** en una entrevista. Si te salen quince, casi siempre significa que
faltó investigación o que estás preguntando cosas reversibles. Vuelve al paso 1.

---

## 3 · El formato de cada pregunta

Cada pregunta lleva cuatro partes. Sin ellas es una pregunta abierta vacía disfrazada.

```
PREGUNTA        Concreta, cerrada donde se pueda, una sola decisión.
POR QUÉ IMPORTA Qué cambia en el trabajo según la respuesta. Sé específico:
                qué módulo, qué fecha, qué costo.
RECOMENDACIÓN   Tu postura, con el motivo. No "depende". Si de verdad depende,
                di de qué depende y recomienda para el caso más probable.
ALTERNATIVAS    Las opciones reales descartadas y por qué, incluyendo el
                trade-off que asume quien elija distinto.
```

**Ejemplo de pregunta bien hecha:**

> **¿La licencia se limita por dispositivo o por empresa?**
>
> **Por qué importa:** define el modelo de datos, el discurso comercial y qué pasa cuando un cliente
> cambia de computadora un viernes por la tarde. Cambiarlo después de emitir licencias es migración, no ajuste.
>
> **Recomiendo** que la licencia pertenezca a la empresa y los dispositivos se cuenten contra ella
> («hasta 3 equipos»). Encaja con cómo compran las empresas, permite reemplazar un equipo sin renegociar,
> y hace natural una edición Enterprise con más equipos.
>
> **Alternativas:** por dispositivo es más fácil de aplicar pero peor de vender y genera soporte cada vez
> que alguien cambia de laptop. Sin límite no es defendible comercialmente.

**El mismo caso, mal hecho:** «¿Cómo quieres manejar las licencias?»

---

## 4 · Conducir la conversación

**De una en una, o en grupos pequeños.** Una pregunta bien contestada vale más que seis a medias. Si
varias son independientes y cortas, agruparlas está bien; si una depende de la respuesta de otra, espera.

**Usa la herramienta adecuada al tipo de pregunta.** Cuando existe `AskUserQuestion` y la pregunta tiene
opciones discretas, úsala: pon tu recomendación como primera opción marcada como tal, y que cada opción
explique su consecuencia. Cuando la respuesta es un dato, un texto o un matiz («¿cuál es el correo oficial?»,
«¿qué te preocupa de este enfoque?»), pregunta en prosa — encajar eso en botones lo empobrece.

**No bloquees todo por una pregunta.** Mientras esperas, avanza en todo lo que no dependa de esa respuesta.
Volver con «mientras tanto dejé listo X, Y y Z» respeta el tiempo de quien contesta.

**Adapta el registro a quien tienes enfrente.** A una persona de negocio háblale de costo, plazo y riesgo
comercial; a una técnica, de arquitectura y deuda. La misma decisión se explica distinto sin cambiar de
recomendación.

---

## 5 · Escuchar de verdad

Aquí es donde una entrevista se gana o se pierde.

**Si la respuesta es vaga**, no la des por buena. «Lo que sea más rápido» no es una decisión: es confianza
depositada en ti. Conviértela en una propuesta concreta y confírmala: *«entiendo que decides por velocidad;
entonces hago X, que cuesta Y de deuda técnica. ¿Correcto?»*

**Si la respuesta contradice algo anterior**, dilo en el momento, sin dramatismo. Las contradicciones en
los briefs son normales — casi siempre se escribieron en sesiones distintas. Nombrarlas es un servicio,
no una corrección.

**Si no están de acuerdo con tu recomendación**, expón tu argumento **una vez más**, con claridad y con lo
que crees que se pierde. Si insisten, **es su decisión**: acátala, regístrala junto con la objeción que
planteaste, y sigue adelante sin resentimiento ni recordatorios pasivo-agresivos después. Insistir una
segunda vez es asesorar; una tercera es obstruir.

**Si responden algo que abre un problema nuevo**, no lo guardes para el final. Dilo, y evalúa si cambia
alguna pregunta que aún no has hecho.

**Si no lo saben**, ayúdalos a acotar en vez de dejar el hueco abierto: *«¿lo puedes averiguar esta semana,
o avanzo con el supuesto A y lo marcamos como pendiente de confirmar?»* Un supuesto explícito y fechado es
infinitamente mejor que una incógnita silenciosa.

---

## 6 · Cerrar con un registro de decisiones

Una entrevista cuyo resultado vive solo en el chat se pierde. El cierre es un documento breve — ése es el
verdadero producto de la sesión, porque es lo que consultará dentro de seis meses alguien que no estuvo.

Guarda un registro por decisión. La plantilla completa, con un ejemplo trabajado, está en
`references/plantilla-decision.md` — léela cuando vayas a escribir el cierre.

En resumen, cada decisión registra: **qué se decidió · quién y cuándo · el contexto que la motivó · qué se
recomendó · las alternativas descartadas · qué se desbloquea o se bloquea · qué la haría revisarse.**

Registra también lo que **no** se decidió. Un pendiente explícito con su dueño y su fecha es información;
un pendiente olvidado es una bomba de tiempo.

Al final, cierra el ciclo en la conversación: qué quedó decidido, qué sigue pendiente y qué vas a hacer
ahora. Que la persona no tenga que releer nada para saber en qué quedaron.

---

## Cuándo *no* usar esta skill

- **Cuando puedes decidir tú.** La mayoría de las cosas son reversibles. Decide y avanza.
- **En tareas de ejecución con instrucciones claras.** Si te dijeron qué hacer y cómo, hazlo.
- **Cuando la investigación aún no está hecha.** Preguntar antes de investigar es el error que esta skill
  existe para evitar. Vuelve al paso 1.
- **Como sustituto de un análisis difícil.** Si la pregunta es «¿qué arquitectura uso?» y no has estudiado
  el problema, la respuesta no es entrevistar: es estudiar el problema y luego presentar una recomendación.
