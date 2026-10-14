# Bitácora de pruebas de funcionamiento

**Abierta el 14 de octubre de 2026**, el día que Dirección abrió ARLES por
primera vez como aplicación de verdad.

Aquí entra **todo defecto encontrado usando ARLES**: qué pasó, qué debería pasar,
cuánto estorba y en qué estado está. Un hallazgo no se arregla hasta que está
escrito aquí — así ninguno se pierde entre dos mensajes.

> **Esto no es el [registro de tropiezos](../07-entrega/REGISTRO-DE-TROPIEZOS.md).**
> Aquél es para **Dirección**: «me ha salido esto, qué hago». Éste es para
> **corregir el producto**: «esto está mal y hay que arreglarlo».

---

## El método de trabajo

Acordado el 14/10/2026. **Hay dos vías, y la decide una sola pregunta:**

### ¿Puedes seguir probando?

| | | Qué haces | Qué hago |
|---|---|---|---|
| **NO** · te bloquea | 🔴 | **Dímelo al momento** | Lo arreglo y publico un instalador nuevo. Paramos la prueba hasta tenerlo |
| **SÍ** · es feo, confuso o está mal escrito | 🟡 | **Sigue probando** y acumula | Los registro todos. Los arreglo **en tanda** cuando me digas «corrige» |

### Por qué así, y no de uno en uno

**Cada instalador tarda unos veinte minutos en compilarse.** Arreglar de uno en
uno significa veinte minutos de espera por cada defecto, y te rompe el ritmo de
la prueba justo cuando estás dentro del flujo.

En tanda es **un solo ciclo** para diez defectos. Pero un bloqueante no puede
esperar, porque sin él no hay prueba que seguir.

### Cómo me das un hallazgo

No hace falta que sea formal. Con esto me basta:

1. **En qué pantalla estabas**
2. **Qué hiciste** (el botón que pulsaste, lo que escribiste)
3. **Qué pasó**
4. **Qué esperabas que pasara**
5. **Una captura**, si se ve

Si falta algo te lo pregunto. **Nunca des por supuesto que es poco importante** —
en este proyecto, siete de cada diez defectos graves los ha encontrado alguien
mirando, no una comprobación automática.

### Qué pasa después

- Lo registro aquí con un número (**H-01**, **H-02**…) y su gravedad.
- Cuando lo arreglo, escribo **qué lo causaba** y la comprobación que impide que
  vuelva — porque en este proyecto la regla es que **una comprobación que nunca
  se ha visto fallar no está verificada** (§2).
- La fila queda aquí para siempre, con su estado.

---

## Gravedad

| | Qué significa |
|---|---|
| 🔴 **Bloqueante** | No se puede seguir. O hace perder datos. Se arregla ya |
| 🟠 **Grave** | Se puede seguir, pero el resultado es incorrecto o engañoso |
| 🟡 **Molesto** | Funciona, pero confunde, se ve mal o está mal escrito |
| 🔵 **Propuesta** | No está mal; podría estar mejor |

---

## Estado

| | |
|---|---|
| **Abierto** | Registrado, sin arreglar |
| **En curso** | Lo estoy arreglando |
| **Arreglado** | Corregido y verificado, con la versión en que entra |
| **Declarado** | No se arregla ahora, y aquí está el motivo y su cifra |
| **No era un defecto** | Se comprobó y el comportamiento es correcto. **Se queda escrito**, con por qué |

---

# Hallazgos

> Vacía por ahora. El primero entra en cuanto Dirección lo describa.

| # | Gravedad | Pantalla | Qué pasa | Estado |
|---|---|---|---|---|
| — | — | — | *(sin hallazgos todavía)* | — |

---

## Lo que ya sabemos que falta, y no hace falta reportar

Esto **no son defectos**: son cosas no construidas, ya declaradas. Si te las
encuentras, no hace falta que las anotes.

| Qué | Por qué |
|---|---|
| Una campaña entera por WhatsApp | El motor sólo recorre correo. De WhatsApp sale el mensaje de prueba |
| Filtrar la audiencia de una campaña | No está construido |
| Editar una campaña ya creada | No está construido |
| La letra no es Mont | Las versiones descargables van sin ella mientras P-01 siga abierto |
| Windows dice «editor desconocido» | Falta el certificado de firma (Fase 9) |
| Activar una campaña grande congela la ventana | Medido: 73,5 s con 451 196 destinatarios; se nota desde unos 30 000. Está en la Fase 9 |
| Buscar texto en el registro tarda | Medido: 3 709 ms. Declarado como C-2 |
