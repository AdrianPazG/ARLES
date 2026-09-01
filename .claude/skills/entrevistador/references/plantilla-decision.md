# Plantilla de registro de decisión

Un archivo por decisión. Nombre sugerido: `DECISIONES/NNNN-titulo-corto.md`, numerado en orden y sin
renumerar nunca (los números son identificadores, no posiciones).

El objetivo del formato es que alguien que no estuvo en la conversación entienda en dos minutos **qué se
decidió y por qué era razonable decidirlo así con la información de ese momento**. Eso último importa:
una decisión que hoy parece equivocada pudo ser correcta con lo que se sabía. Registrar el contexto evita
que se juzgue el pasado con datos del futuro.

---

## Plantilla

```markdown
# NNNN · [Título en una línea, en forma de decisión]

- **Estado:** Propuesta | Aceptada | Rechazada | Sustituida por NNNN | Pendiente
- **Fecha:** AAAA-MM-DD
- **Decide:** [Persona o rol con autoridad sobre esta decisión]
- **Consultó:** [Quién más participó]

## Decisión

[Una o dos frases en voz activa. "Usaremos X." "La licencia pertenece a la empresa."
No "se evaluó", no "se considera". Qué se hará.]

## Contexto

[Qué situación obligó a decidir. Qué se sabía en ese momento y qué no.
Incluye las restricciones reales: plazo, presupuesto, personas, contratos, dependencias externas.]

## Qué se recomendó

[La recomendación que se llevó a la mesa, con su motivo. Si la decisión final fue distinta,
esto queda igual: el registro documenta el proceso, no solo el resultado.]

## Alternativas consideradas

| Opción | A favor | En contra | Por qué no |
|---|---|---|---|
| | | | |

## Consecuencias

**Se desbloquea:** [Qué se puede empezar ahora.]

**Se asume:** [Deuda técnica, costo, riesgo o limitación que se acepta conscientemente.]

**Se cierra:** [Qué deja de ser posible, o qué se vuelve caro de revertir.]

## Qué haría revisar esta decisión

[Condición concreta y observable, no "si cambian las circunstancias".
Ejemplo: "si superamos 500 000 contactos por instalación" o "si un cliente exige ejecución con el equipo apagado".]

## Objeciones registradas

[Si alguien —incluido tú— estuvo en desacuerdo, aquí queda su argumento.
No es un reproche: es información para quien revise esto después.
Si no hubo objeciones, escribe "Ninguna".]
```

---

## Ejemplo trabajado

```markdown
# 0007 · La licencia pertenece a la empresa, no al dispositivo

- **Estado:** Aceptada
- **Fecha:** 2026-09-01
- **Decide:** Dirección
- **Consultó:** Arquitectura, Comercial

## Decisión

Una licencia se emite a nombre de la empresa cliente e incluye un máximo configurable de dispositivos
activos. Los equipos se registran y se dan de baja contra esa licencia sin renegociar el contrato.

## Contexto

El producto se distribuye como software instalable, cada organización lo despliega en uno o varios equipos,
y aún no existe servidor de activación. Había que fijar la unidad de licenciamiento antes de definir el
esquema de base de datos, porque cambiar la unidad después implica migrar licencias ya emitidas.

## Qué se recomendó

Licencia por empresa con conteo de dispositivos. Es como compran las organizaciones, permite reemplazar un
equipo averiado sin fricción comercial, y hace natural una edición Enterprise que solo sube el tope de
equipos sin cambiar el modelo.

## Alternativas consideradas

| Opción | A favor | En contra | Por qué no |
|---|---|---|---|
| Por dispositivo | Fácil de aplicar técnicamente | Genera un ticket de soporte cada vez que alguien cambia de laptop; difícil de vender por volumen | El costo de soporte previsto supera la simplicidad técnica |
| Sin límite | Cero fricción para el cliente | No hay palanca de precio entre ediciones | No es defendible comercialmente |

## Consecuencias

**Se desbloquea:** el esquema de `License` y `Device`, y el discurso comercial de las tres ediciones.

**Se asume:** hay que construir registro y baja de dispositivos, que por dispositivo no haría falta.

**Se cierra:** el precio por puesto individual deja de ser una opción sin rediseñar el modelo.

## Qué haría revisar esta decisión

Si aparece demanda real de uso individual por profesional independiente, o si el conteo de dispositivos
resulta evadible de forma trivial en producción.

## Objeciones registradas

Comercial señaló que el conteo por dispositivo es más fácil de auditar en una disputa contractual.
Se aceptó el riesgo: en esta etapa pesa más la facilidad de venta que la auditabilidad.
```

---

## Notas de uso

- **Estado «Pendiente»** es legítimo y útil. Una decisión que no se pudo tomar, registrada con su dueño y su
  fecha límite, evita que se olvide. Vale más que un archivo que no existe.
- **No reescribas decisiones aceptadas.** Si cambia el criterio, crea una nueva y marca la anterior como
  *Sustituida por NNNN*. El historial es el valor.
- **Sé breve.** Si el registro pasa de una pantalla, probablemente está mezclando varias decisiones.
  Sepáralas.
