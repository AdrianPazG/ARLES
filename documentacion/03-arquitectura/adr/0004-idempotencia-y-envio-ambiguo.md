# ADR-0004 · Idempotencia y política de envío ambiguo

**Estado:** aceptado · **Fecha:** 2026-09-11 · **Decide:** Tech Lead + Product

---

## Contexto

El §55 exige que caídas, reinicios y dobles clics **nunca** produzcan correos duplicados.

Un duplicado es el peor fallo posible del producto: es irreversible —no se puede des-enviar—, molesta al destinatario, alimenta la tasa de queja y contribuye a quemar la reputación del dominio del cliente. Y es invisible para nosotros: nadie nos avisa.

## Decisión

### Tres capas de idempotencia

**1 · Restricción única en el esquema**

```sql
CREATE UNIQUE INDEX idx_attempt_unique
  ON message_attempt (campaign_id, channel, contact_address);
```

> **Ampliado en la V4 (17/09/2026).** La columna pasó a llamarse
> `contact_address` —una columna llamada «email» que guarda un teléfono es una
> trampa— y la clave ganó el canal. Lo que el canal compra es modesto y conviene
> no inflarlo: hace que la clave diga lo que significa y sostiene la garantía si
> dos canales llegaran a compartir la misma cadena. **No** es lo que permite
> escribir por dos canales: eso ya funcionaba, porque la clave es la dirección.
>
> **Corregido en la Fase 1 (hallazgo F1).** La clave era `contact_id`, y con
> ella borrar un contacto y reimportarlo permitía **volver a enviarle**: el id
> es nuevo, así que la fila única ya no colisionaba. La clave es la
> **dirección**, igual que en la supresión (§39): un contacto borrado y
> reimportado es un id nuevo; una dirección es la misma persona.

Ningún fallo de lógica, ninguna condición de carrera y ningún doble clic pueden crear dos intentos para el mismo par. **La base de datos lo rechaza.** El §55 pasa a ser una propiedad estructural en vez de una esperanza depositada en el código.

**2 · Compare-and-swap al tomar trabajo**

```sql
UPDATE message_attempt
   SET state = 'claimed', claimed_at = ?
 WHERE id = ? AND state = 'queued';
```

Se verifica el número de filas afectadas. Si es 0, otro worker ganó. Nunca se lee-y-luego-escribe.

**3 · Clave de idempotencia en el mensaje**

UUID persistido **antes** de llamar al proveedor, que viaja como `Message-Id`.

### Y la decisión que de verdad importa: el caso ambiguo

Hay una ventana que las tres capas **no** cierran:

```
1. UPDATE state = 'sending'   ← persistido
2. provider.send(...)         ← el proveedor ACEPTA
3. ✗ la aplicación muere
4. UPDATE state = 'sent'      ← nunca ocurre
```

Al reiniciar, la fila está en `sending` y **es genuinamente imposible saber si el correo salió**. Averiguarlo exigiría leer la bandeja de enviados, permiso que v1.2.0 no solicita (§69) y que no queremos solicitar.

**Decisión: toda fila que quede en `sending` al arrancar pasa a `presumed_sent`. Nunca se reenvía automáticamente.**

Se muestra en la interfaz como «Envío no confirmado», con el contacto y la hora, y el usuario decide.

## Justificación

Los dos errores posibles **no son simétricos**:

| Si elegimos… | Y el correo sí salió | Y el correo no salió |
|---|---|---|
| **Reenviar** | **Duplicado.** Irreversible, invisible para nosotros, acumulativo para el dominio | Correcto |
| **No reenviar** | Correcto | Un contacto no recibió. **Visible y recuperable** |

Un no-envío es un problema que el usuario ve y puede arreglar en treinta segundos. Un duplicado no lo ve nadie hasta que la reputación del dominio ya está dañada.

**Fallamos del lado recuperable.**

## Consecuencias

**Positivas.** El duplicado es imposible por esquema · el caso ambiguo es explícito en vez de resuelto en silencio con la opción equivocada · el usuario conserva el control sobre una decisión que sólo él puede tomar con contexto.

**Negativas.**
- Un estado más en la interfaz, que hay que explicar bien. El §94 obliga a que el texto diga qué pasó, qué hacer y qué está a salvo.
- Trabajo manual para el usuario cuando ocurre.

**Mitigación del segundo punto:** en la práctica la ventana es de milisegundos y `presumed_sent` debería ser raro. **Si se vuelve frecuente, es señal de que la aplicación está cayendo** — y ése es el bug de verdad, no el estado. Conviene instrumentarlo para detectarlo.

## Alternativas descartadas

**Reenviar automáticamente.** Optimiza la entrega a costa del riesgo de duplicado. Inaceptable por la asimetría de arriba.

**Escribir `sent` antes de llamar al proveedor.** Invierte el problema: ahora el fallo silencioso es el no-envío, que nadie ve porque el sistema cree que envió. Peor: es un fallo invisible.

**Two-phase commit con el proveedor.** Ni SMTP ni Gmail API lo soportan. No existe.

**Consultar la bandeja de enviados al arrancar.** Resolvería la ambigüedad de verdad, pero exige `gmail.readonly` — un scope **restringido** que dispararía la auditoría CASA Tier 2 con su auditor externo y su coste anual. Pagar eso para resolver un caso que ocurre en milisegundos es desproporcionado. Y contradice el §69.

**Heurística por tiempo** («si lleva más de N segundos en `sending`, asumir que falló»). Es adivinar, y adivinar mal produce el duplicado que intentamos evitar.
