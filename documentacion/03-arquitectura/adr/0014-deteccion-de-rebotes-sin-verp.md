# ADR-0014 · Detección de rebotes sin VERP

**Estado:** aceptado · **Fecha:** 2026-09-12 · **Decide:** Tech Lead + Dirección
**Sustituye** la sección «v1.3 — detección completa vía VERP» de [ADR-0009](0009-alcance-deteccion-rebotes.md)
**Resuelve:** P-04 · **Riesgos:** R-06 · **Relacionado:** D-1

---

## Contexto

ADR-0009 decidió que en v1.2.0 la detección de rebotes sería parcial —sólo
rechazos síncronos 5xx— y difirió a v1.3 la detección completa mediante **VERP**:
un `Return-Path` único por destinatario apuntando a un buzón de rebotes
dedicado, leído por IMAP.

Dirección pidió **adelantar esa detección a v1.2.0**, haciéndola opcional y con
degradación elegante: si el cliente no configura el buzón, la campaña corre con
detección sólo síncrona y el preflight lo advierte.

Antes de construirlo se verificó si el mecanismo es posible en los proveedores
previstos. **No lo es.**

---

## El hallazgo

### El VERP es inviable en los tres canales de v1.2.0

| Canal | Por qué no |
|---|---|
| **API de Gmail** (`messages.send`) | El sobre SMTP **no es un parámetro de la API**. Se entrega el mensaje RFC 5322 en `raw` y Google construye el sobre desde la cuenta autenticada o un alias `sendAs` verificado. No hay campo donde poner el Return-Path |
| **SMTP autenticado de Gmail** | Reescribe el Return-Path a la cuenta autenticada y rechaza cualquier `MAIL FROM` que no sea un alias verificado |
| **Microsoft 365** | Exchange Online exige que el `5321.MailFrom` coincida con el buzón autenticado o tenga permiso *Send As*, y encima aplica **SRS**, que lo reescribe en salida |

**Una dirección de retorno distinta por destinatario no es posible.** El plan de
v1.3 de ADR-0009 no habría funcionado, y lo habríamos descubierto construyéndolo.

### Y la vía alternativa por API está cerrada por coste, no por técnica

Leer rebotes con la API de Gmail requiere `users.watch`, que exige uno de
`gmail.readonly`, `gmail.modify`, `gmail.metadata` o `mail.google.com`. **Los
cuatro son scopes restringidos**, lo que implica verificación más **evaluación
CASA anual**, con coste y renovación.

Peor: **combinar `gmail.send` con cualquier scope restringido reclasifica la
aplicación entera como restringida.** Es exactamente el coste que D-1 se diseñó
para evitar, y lo pagaríamos aunque el cliente nunca active la función.

Y no existe atajo: **no hay un scope de «leer sólo una etiqueta»**. El filtro
`labelIds` de `watch` limita qué se notifica, no qué puede leer el token.

> ⚠️ Verificado en documentación de referencia y fuentes secundarias; el
> entorno de investigación tenía bloqueado `developers.google.com`. La
> coincidencia entre fuentes es alta y el diseño se apoya en ello, pero
> **conviene reconfirmarlo antes de escribir la primera línea de la Fase 7**.

---

## Decisión

### 1 · La detección asíncrona existe, pero por dos caminos distintos

| Camino | Cómo funciona | Para quién |
|---|---|---|
| **A · VERP completo** | El cliente controla el `MAIL FROM`. Return-Path único por destinatario hacia un buzón dedicado, leído por IMAP | **Sólo SMTP con servidor propio** |
| **B · Reenvío por filtro** | El cliente crea una regla en su proveedor que **reenvía los mensajes de rebote a un buzón externo** que él controla. ARLES lo lee con **IMAP normal, cero scopes de Google**, y correlaciona por `Message-Id` | **Gmail y Microsoft 365** |

El camino B es la aportación de este ADR y es lo que hace la función viable
para la mayoría. Funciona porque **el `Message-Id` ya lo generamos nosotros**
desde la clave de idempotencia (ADR-0004), y el DSN lo devuelve dentro del
cuerpo del reporte. No hace falta tocar el sobre.

Es más frágil que el VERP —depende de que el cliente configure bien el filtro—
y por eso el diseño no lo da por bueno: **el preflight comprueba que el buzón
recibe algo antes de prometer detección automática.**

### 2 · La disponibilidad la decide el proveedor, no una casilla

La propuesta original lo planteaba como elección del cliente. En realidad es una
**capacidad del canal**, y presentarla como casilla haría que alguien perdiera
una tarde configurando algo que nunca iba a funcionar.

`Capabilities` ya tiene `custom_return_path` desde la Fase 0 — se amplía:

```rust
pub struct Capabilities {
    pub custom_return_path: bool,   // camino A
    pub bounce_forwarding: bool,    // camino B: el proveedor admite reenvío
    // …
}
```

El preflight muestra **tres estados, no dos**:

| Estado | Qué dice la interfaz |
|---|---|
| Buzón configurado y verificado | *Detección automática de rebotes activa.* |
| El canal lo admite, sin configurar | *Puedes activar la detección automática configurando un buzón de rebotes. Sin ella, los rebotes llegarán a tu bandeja y tendrás que suprimirlos a mano.* |
| **El canal no lo admite** | *Tu proveedor no permite dirigir los rebotes a un buzón dedicado. Con esta cuenta la detección automática no es posible; los rebotes llegarán a tu bandeja.* |

El tercero es el que la propuesta no contemplaba, y es el que evita prometer lo
imposible.

### 3 · La degradación elegante se mantiene, tal como pedía Dirección

Sin buzón configurado, **la campaña corre igual**, con detección síncrona y la
advertencia del preflight. Nunca se bloquea por esto.

### 4 · Se secuencia por fases, no todo en v1.2.0

Adelantarlo entero no es gratis: trae configuración y credenciales IMAP —otro
secreto en el llavero—, conexión en segundo plano, parseo de DSN con sus
variantes reales, clasificación duro/blando y correlación con el intento
original. Es un subsistema, no una función.

| Fase | Qué entra |
|---|---|
| **3** | Sólo esquema: el nuevo origen `'dsn'` en `suppression_entry` y el sitio para la configuración en `email_account`. Cero IMAP |
| **5** | Las capacidades por proveedor y **los tres estados del preflight**. Aquí ya se nota |
| **7** | El lector IMAP y el parseo de DSN, **detrás de un interruptor**. Si llega, va en v1.2.0; si se complica, sale en v1.2.1 sin bloquear el lanzamiento |

Así la arquitectura queda hecha ahora —que es lo que Dirección pidió y lo que
evita reescrituras— y la parte cara no se lleva por delante la fecha.

### 5 · Lo que NO cambia de ADR-0009

La decisión de v1.2.0 sigue intacta, y ahora con más razón:

- Rechazos síncronos 5xx → supresión automática.
- Supresión manual.
- **Y la condición innegociable: la interfaz declara la limitación.**

Sobre el parseo de DSN: el RFC 3464 define `multipart/report` y los MTA grandes
lo cumplen la mayor parte del tiempo, pero no existe una biblioteca de Rust
consolidada. **Se asume implementación propia con respaldo heurístico**, y ese
coste está contado en la Fase 7.

---

## Consecuencias

- **ADR-0009 queda corregido**, no derogado: su decisión de alcance para v1.2.0
  era correcta; lo que no funcionaba era su plan de v1.3.
- **La detección completa depende del proveedor del cliente**, y eso hay que
  decirlo en el material comercial. Un cliente en Gmail sin dominio propio
  tendrá siempre detección parcial o dependiente de un filtro que él mantiene.
- **Refuerza ADR-0013 §8**: SMTP con dominio propio no sólo es mejor para la
  reputación, es el único camino con detección de rebotes completa.
- El `Message-Id` derivado de la clave de idempotencia pasa de ser un detalle de
  trazabilidad a ser **la pieza que hace posible el camino B**. No se cambia sin
  revisar este ADR.

---

## Alternativas descartadas

**Pedir `gmail.readonly` y asumir la auditoría CASA.** Reclasifica la
aplicación entera como restringida, con evaluación anual y coste recurrente,
para una función opcional. Contradice D-1 de frente.

**Leer la bandeja principal del usuario.** Prohibido por §69, y con razón: el
producto promete no fisgar el correo del cliente.

**Renunciar a la detección asíncrona.** Era la posición de ADR-0009 y es
defendible, pero el camino B existe, no cuesta scopes y resuelve el caso de la
mayoría. Renunciar ahora sería no haber mirado.
