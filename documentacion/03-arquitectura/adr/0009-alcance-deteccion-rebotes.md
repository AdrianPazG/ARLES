# ADR-0009 · Alcance de la detección de rebotes

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Product + Dirección
**Resuelve:** contradicción del brief entre §37 y §69

---

## Contexto

El brief se contradice consigo mismo:

- **§37** exige que los rebotes duros alimenten automáticamente la lista de supresión. La lista de supresión es el control anti-abuso central del producto.
- **§69** prohíbe solicitar permisos de lectura de bandeja de entrada en v1.2.0.

**Ambas cosas no pueden ser ciertas a la vez.** Los rebotes duros llegan como **correo asíncrono** a la bandeja del remitente, minutos u horas después del envío, en forma de mensaje DSN (RFC 3464). Sin leer un buzón, no hay forma de verlos.

No es un detalle menor. Sin supresión automática, los contactos muertos se reintentan campaña tras campaña, la tasa de rebote se sostiene por encima del umbral que los filtros vigilan, y el cliente acaba dañando la reputación de su propio dominio **usando el producto correctamente**.

## Decisión

### v1.2.0 — detección parcial, declarada como tal

**1 · Rechazos síncronos 5xx del diálogo SMTP.**

El diálogo SMTP devuelve códigos en el momento del envío. Cuando el servidor de destino rechaza al destinatario durante la transacción, lo sabemos inmediatamente:

| Código | Acción |
|---|---|
| `550`, `553`, `551`, `554` | `PermanentRecipient` → supresión automática |
| `450`, `451`, `452`, `421` | `Transient` → reintento |

Esto atrapa **buzones inexistentes en servidores que validan durante el diálogo**, que es un subconjunto real y útil.

**2 · Supresión manual.** El usuario puede suprimir direcciones a mano, individualmente o por lote.

**3 · Y la condición innegociable: la interfaz lo declara.**

El centro de entregabilidad y la pantalla de supresiones muestran, de forma visible y permanente:

> **La detección automática de rebotes es parcial en esta versión.** ARLES detecta los rechazos que el servidor de destino comunica durante el envío, pero no los que llegan después como correo a tu bandeja. Revisa tu bandeja periódicamente y suprime a mano las direcciones que reboten.

### v1.3 — detección completa vía VERP

`Return-Path` único por destinatario apuntando a un **buzón de rebotes dedicado** que el cliente configura, leído por IMAP. Incluye parseo de DSN según RFC 3464 y clasificación duro/blando.

**Clave:** es una cuenta **separada que el cliente crea para ese fin**, no su bandeja personal. Eso respeta el espíritu del §69 —no fisgar el correo del usuario— cumpliendo el §37.

## Justificación

### Por qué no se hace VERP ya

Añade un dominio completo al alcance: configuración IMAP, gestión de conexión persistente, parseo de DSN con sus variantes reales (que se desvían del RFC con frecuencia), clasificación duro/blando, y correlación del rebote con el intento original. Y pide al cliente que configure un buzón extra en el onboarding.

En v1.2.0 —que es despliegue interno de TELEMETRY (D-4) y con SMTP como único proveedor— el coste no se justifica frente a la alternativa de declarar la limitación.

### Por qué la declaración es innegociable

Esta es la parte que no se puede negociar por conveniencia.

**Un cliente informado puede compensarlo.** Revisa su bandeja una vez por semana, suprime lo que rebotó, y sigue adelante. Cuesta diez minutos.

**Un cliente que se cree protegido descubre el problema cuando ya quemó su dominio.** Y lo descubre porque sus correos dejan de llegar, no porque ARLES le avisara.

El §65 exige honestidad en las métricas. **La honestidad sobre las capacidades es el mismo principio.** Un producto que calla lo que no sabe hacer es tan deshonesto como uno que dice «entregado» cuando sólo sabe «aceptado».

## Consecuencias

**Positivas.** Alcance de v1.2.0 acotado y entregable · el subconjunto que sí se detecta funciona de verdad y es automático · la limitación es visible, así que el usuario puede actuar · la arquitectura ya prevé VERP (`Capabilities::custom_return_path` existe en el trait desde v1.2.0).

**Negativas.**
- **La lista de supresión está incompleta durante v1.2.0.** Es el coste directo de la decisión (riesgo R-06).
- Trabajo manual recurrente para el usuario.
- La entregabilidad se degrada con el tiempo si el usuario no hace ese trabajo. **Mitigación:** un recordatorio periódico en el panel de inicio si hace más de N días que no se revisan supresiones.

## Alternativas descartadas

**Solicitar `gmail.readonly` para leer los rebotes.** Es un scope **restringido**: dispararía la auditoría CASA Tier 2 con auditor externo aprobado por Google, coste significativo y renovación anual. Y contradice el §69 de forma directa: pediría acceso a **toda** la bandeja del usuario para leer unos pocos mensajes de sistema. Desproporcionado y contrario al principio de mínimo privilegio (§87).

**Ignorar la contradicción y no implementar nada.** Deja la lista de supresión sin alimentación automática **y** sin que el usuario lo sepa. Es la peor opción de todas.

**Implementar VERP completo en v1.2.0.** Correcto pero caro. Diferido, no descartado.

**Inferir rebotes por ausencia de respuesta.** No se puede: el correo no confirma entregas. Sería adivinar, y adivinar contradice el §65.

## Nota para v1.3

Cuando se implemente VERP, dos puntos de diseño que conviene decidir pronto:

1. **El buzón de rebotes debe poder ser del mismo dominio pero cuenta distinta** (`bounces@empresa.com`). Un dominio distinto complica SPF y DMARC.
2. **La clasificación duro/blando debe ser conservadora.** Ante la duda, blando: suprimir por error a un contacto válido es un fallo silencioso que nadie detecta, y el producto pierde un destinatario legítimo para siempre.
