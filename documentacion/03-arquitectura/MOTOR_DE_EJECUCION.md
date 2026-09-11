# Motor de ejecución

**Proyecto:** ARLES RELAY I · v1.2.0
**Crate:** `arles-engine`

> Este es el corazón del producto y donde se concentra el 70 % del riesgo técnico. Idempotencia, reanudación tras suspensión y respeto de ventanas horarias son problemas de sistemas distribuidos con ropa de aplicación de escritorio.
>
> Por eso el motor ocupa la **Fase 4** del roadmap, **antes** que la interfaz de campañas. Si el motor no es correcto, las pantallas bonitas sólo hacen más visible el fallo.

---

## 1. Máquina de estados

```
                    ┌──────────┐
                    │  queued  │◄──────────────┐
                    └────┬─────┘               │
                    CAS  │                     │ backoff
                    ┌────▼─────┐               │
                    │ claimed  │               │
                    └────┬─────┘               │
       supresión ◄───────┤                     │
                    ┌────▼─────┐               │
                    │ sending  │               │
                    └────┬─────┘               │
          ┌──────────────┼──────────────┐      │
     ┌────▼───┐    ┌─────▼────┐   ┌─────▼────┐ │
     │  sent  │    │  failed  ├───┘          │ │
     └────────┘    └──────────┘   │permanently│ │
                                  │  _failed  │ │
                                  └───────────┘ │
                                                │
   ┌───────────────┐   ┌───────────┐   ┌────────┴──────┐
   │  suppressed   │   │ cancelled │   │ presumed_sent │
   └───────────────┘   └───────────┘   └───────────────┘
                                        (la app murió en `sending`)
```

| Estado | Terminal | Significado |
|---|---|---|
| `queued` | no | En cola |
| `claimed` | no | Un worker lo tomó |
| `sending` | no | Llamada al proveedor en curso |
| `sent` | **sí** | El proveedor **aceptó** el mensaje |
| `failed` | no | Fallo transitorio; se reintentará |
| `permanently_failed` | **sí** | Fallo definitivo |
| `suppressed` | **sí** | Bloqueado por la lista de supresión |
| `cancelled` | **sí** | Cancelado por parada |
| `presumed_sent` | **sí** | Ambiguo — ver §3 |

`sent` significa **aceptado por el proveedor**, no entregado. La interfaz dice «Aceptado» (§65).

---

## 2. Idempotencia

Tres capas, de más fuerte a más débil.

### Capa 1 — El esquema

```sql
CREATE UNIQUE INDEX idx_attempt_unique
  ON message_attempt (campaign_id, contact_id);
```

Ningún fallo de lógica, ninguna condición de carrera y ningún doble clic pueden producir dos intentos para el mismo par campaña-contacto: **la base de datos lo rechaza**. Esto convierte el §55 en una propiedad estructural, no en una esperanza.

### Capa 2 — Compare-and-swap al tomar trabajo

```sql
UPDATE message_attempt
   SET state = 'claimed', claimed_at = ?, email_account_id = ?
 WHERE id = ? AND state = 'queued';
```

Se verifica el número de filas afectadas. **Si es 0, otro worker ganó la carrera** y este worker sigue adelante sin hacer nada. Nunca se lee-y-luego-escribe: la comprobación y la escritura son una sola operación atómica.

### Capa 3 — Clave de idempotencia en el propio mensaje

Un UUID generado y persistido **antes** de llamar al proveedor, que viaja como `Message-Id` del correo. Da trazabilidad y permitiría deduplicar en el lado del proveedor si algún día tuviéramos visibilidad allí.

---

## 3. El caso ambiguo — la decisión más importante del motor

Hay una ventana que **ninguna de las tres capas cierra**:

```
1. UPDATE state = 'sending'     ← persistido
2. provider.send(mensaje)       ← el proveedor ACEPTA
3. ✗ la aplicación muere aquí
4. UPDATE state = 'sent'        ← nunca ocurre
```

Al reiniciar, la fila está en `sending`. **El correo pudo salir o no, y no hay forma de saberlo.** Averiguarlo exigiría leer la bandeja de enviados, un permiso que v1.2.0 no solicita (§69) y que no queremos solicitar.

### Decisión: nunca se reenvía automáticamente

Toda fila que quede en `sending` al arrancar pasa a **`presumed_sent`**. Se muestra en la interfaz como «Envío no confirmado», con el contacto y la hora, y **el usuario decide** si reenviar manualmente.

**El razonamiento:** los dos errores posibles no son simétricos.

| Si elegimos… | Y el correo sí había salido | Y el correo no había salido |
|---|---|---|
| **Reenviar** | **Duplicado.** Irreversible. Molesta al destinatario, alimenta la tasa de queja y contribuye a quemar la reputación del dominio | Correcto |
| **No reenviar** | Correcto | Un contacto no recibió el correo. Reversible: el usuario lo ve y lo reenvía |

Un no-envío es un problema visible y recuperable. Un duplicado es invisible para nosotros, irreversible para el destinatario y acumulativo para el dominio. **Preferimos fallar del lado recuperable.**

Ver ADR-0004.

> **Nota de producto:** en la práctica esta ventana es de milisegundos y `presumed_sent` debería ser raro. Si se vuelve frecuente, es señal de que la aplicación está cayendo, y eso es el bug de verdad.

---

## 4. Límites y throttling

### Cubo de tokens persistido

Por cuenta remitente, con dos ventanas simultáneas —horaria y diaria— en la **zona horaria de la empresa**, no la del sistema operativo. Un portátil que cruza husos horarios no debe cambiar la política de envío.

El estado vive en `rate_budget`, así que sobrevive a cierres y reinicios. Un cubo en memoria se reiniciaría con la aplicación y permitiría enviar el límite diario varias veces al día.

### Ventana de ejecución

`execution_window` define días y horas operativas. Fuera de ventana, los workers no toman trabajo. No se acumula nada: simplemente se espera.

### Aviso de más de 50 diarios (§48)

Si el usuario configura más de 50 correos diarios por remitente, se muestra un aviso claro sobre riesgo de entregabilidad, rebotes y reputación de dominio, **y se registra su aceptación** en `campaign.over_50_warning_accepted_at` y en `audit_log`.

No se bloquea. El §154 pide autonomía: informar y advertir, no dictar.

---

## 5. Suspensión del equipo — el problema de la ráfaga

El §53 prohíbe que al despertar el equipo se dispare una ráfaga de correos acumulados. Esto es más sutil de lo que parece.

**El fallo que hay que evitar.** Un cubo de tokens ingenuo se recarga en función del tiempo transcurrido. Si el equipo duerme ocho horas, al despertar el cubo calcula «han pasado 8 horas, me corresponden 8 horas de permisos» y los libera **todos de golpe**. El resultado es exactamente la ráfaga prohibida, y es la forma más rápida de que un proveedor marque la cuenta.

### Detección

```rust
// Al arrancar y en cada tick del planificador
let salto_monotono = Instant::now()      - ultimo_instant;
let salto_pared    = SystemTime::now()   - ultimo_systemtime;

// El reloj monótono NO avanza durante la suspensión; el de pared SÍ.
if salto_pared > salto_monotono + UMBRAL {
    // Hubo suspensión. Duración ≈ salto_pared - salto_monotono
    reiniciar_presupuestos();
}
```

El reloj monótono no avanza mientras el sistema está suspendido; el reloj de pared sí. La diferencia entre ambos **es** la duración de la suspensión.

### Respuesta

**El cubo se reinicia, no se acumula.** Los presupuestos horario y diario se recalculan desde cero contra la ventana actual.

Consecuencia, que es la correcta: **un correo no enviado ayer no se recupera hoy.** Se queda sin enviar y la campaña dura más. El simulador ya avisó de que la campaña tardaría 100 días; que tarde 103 porque el equipo estuvo apagado es el comportamiento esperado, no un fallo.

**Verificación obligatoria:** test que simula un salto del reloj de pared y comprueba que no hay ráfaga (riesgo R-04).

---

## 6. Reintentos

| Tipo de fallo | Respuesta |
|---|---|
| Red, timeout, 5xx transitorio | Reintento con backoff exponencial y jitter |
| **429 con `Retry-After`** | **Se respeta literalmente.** Nunca se ignora ni se acorta |
| 5xx permanente (buzón inexistente) | `permanently_failed` + entrada en supresión |
| 4xx de autenticación | Circuito abierto en la cuenta; **no se reintenta** |
| Supresión detectada | `suppressed`. No es un fallo |

**Backoff:** base 30 s, factor 2, techo 1 h, con jitter completo. El jitter importa: sin él, todos los intentos fallidos de un corte de red reintentan exactamente a la vez cuando vuelve.

**Tope de intentos:** 5. Después, `permanently_failed`.

---

## 7. Disyuntor por cuenta

Tras **N fallos consecutivos** (por defecto 5) en una misma cuenta, el circuito se abre: `email_account.circuit_open_until` se fija y esa cuenta deja de tomar trabajo.

La interfaz lo dice explícitamente y con el motivo. Un fallo silencioso que simplemente ralentiza la campaña es peor que una parada visible.

Se cierra automáticamente al vencer el plazo, o manualmente cuando el usuario corrige el problema (p. ej. renueva la contraseña de aplicación).

---

## 8. Sin rotación de remitentes (T-1)

Al alcanzarse el límite de una cuenta, **la cola pausa**:

> «La cuenta [correo] alcanzó su límite diario de 50 envíos. La campaña continuará mañana a las 09:00.»

**No se salta a otra cuenta.** El usuario puede tener varias cuentas registradas y asignarlas a campañas distintas, pero una campaña tiene un remitente y lo respeta.

Esto es una decisión de producto, no una limitación técnica. Rotar remitentes para superar un límite es evasión, viola las políticas de Google y es el mecanismo exacto por el que se queman dominios. Ver ADR-0008.

---

## 9. Pausar, Reanudar, Detener

Tres acciones **distintas** que la interfaz nunca debe confundir (§62):

| Acción | Efecto en la cola | Reversible |
|---|---|---|
| **Pausar** | Se conserva intacta | Sí |
| **Reanudar** | Continúa donde estaba, respetando límites | — |
| **Detener** | Los pendientes pasan a `cancelled` | **No** |

**Detener exige confirmación explícita** y dice cuántos mensajes quedarán sin enviar.

### Parada de emergencia

Bandera en base de datos más `CancellationToken`. Los workers la comprueban **antes de cada envío**, no entre lotes.

Esto último importa: si se comprobara entre lotes de 50, «parar» significaría «parar dentro de unos minutos y unos cuantos correos». Comprobar antes de cada envío hace que la parada sea inmediata en el único sentido que le importa al usuario: **ningún correo más sale**.

Lo que ya fue aceptado por el proveedor no se puede recuperar, y la interfaz debe decirlo.

---

## 10. Independencia de la ventana

El motor corre en su propio conjunto de tareas de Tokio, independiente de la ventana principal (§51). Cerrar la ventana **no** detiene una campaña; se mantiene el icono en la bandeja del sistema o la barra de menús.

**Y la interfaz debe decirlo.** Cerrar una ventana y no saber si la campaña sigue corriendo es exactamente la clase de ambigüedad que erosiona la confianza en un producto que maneja envíos reales. Al cerrar con una campaña activa, se avisa.

---

## 11. Simulador (§50)

Antes de activar, se calcula la duración real:

```
Entradas: tamaño de audiencia (tras supresión)
          límite diario y horario
          ventana de ejecución (días y horas)
          zona horaria de la empresa
Salida:   fecha y hora estimadas de finalización
          desglose por día
```

Se presenta en lenguaje llano: «Esta campaña enviará 5 000 correos. Con tu límite de 50 diarios y enviando de lunes a viernes de 9 a 18, terminará alrededor del 24 de enero de 2027.»

Ese número es frecuentemente la información más valiosa de todo el producto: es lo que hace que alguien reconsidere su configuración **antes** de lanzar, no después.

---

## 12. Preflight (§63)

Validación previa a la activación. Devuelve **LISTA** o bloquea con errores accionables:

- [ ] La cuenta remitente autentica correctamente
- [ ] La credencial no ha caducado ni ha sido revocada
- [ ] La audiencia no está vacía tras aplicar supresiones
- [ ] Todas las variables de la plantilla se resuelven para todos los contactos
- [ ] La ventana de ejecución es coherente y alcanzable
- [ ] Hay conectividad
- [ ] Se realizó el envío de prueba (§45)
- [ ] SPF/DKIM/DMARC comprobados (advertencia, no bloqueo)

Los errores siguen el §95: **qué pasó, cómo arreglarlo, y qué está a salvo.**

---

## 13. Bucle del worker

```
mientras no cancelado:
    si parada_de_emergencia:                  → salir
    si fuera_de_ventana:                      → esperar
    si circuito_abierto(cuenta):              → esperar
    si presupuesto_agotado(cuenta):           → pausar cola + avisar (T-1)

    intento = tomar_siguiente()               ← CAS
    si ninguno:                               → esperar

    si suprimido(intento.email):              → suppressed; continuar
    si parada_de_emergencia:                  → cancelled; salir   ← segunda comprobación

    persistir(state = 'sending')              ← ANTES de llamar al proveedor
    resultado = proveedor.enviar(mensaje)

    según resultado:
        Aceptado    → sent, consumir token
        Transitorio → failed, programar reintento
        Permanente  → permanently_failed, suprimir
        429         → respetar Retry-After
        AuthFallida → abrir circuito
```

La **doble comprobación de la parada de emergencia** —antes de tomar trabajo y otra vez justo antes de enviar— es deliberada: entre ambas puede haber pasado una consulta a la base de datos y una resolución de credencial, tiempo más que suficiente para que el usuario pulse el botón.
