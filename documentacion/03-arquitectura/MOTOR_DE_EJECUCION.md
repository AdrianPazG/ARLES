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
  ON message_attempt (campaign_id, channel, contact_address);
```

Ningún fallo de lógica, ninguna condición de carrera y ningún doble clic pueden producir dos intentos para la misma campaña, la misma dirección y el mismo canal: **la base de datos lo rechaza**. Esto convierte el §55 en una propiedad estructural, no en una esperanza.

!! **Este bloque estuvo desactualizado dos veces.** Decía `contact_id`, que el
hallazgo F1 corrigió a `contact_email` en la Fase 1 sin actualizar aquí, y las
migraciones V3 y V4 lo cambiaron otra vez a `(campaign_id, channel,
contact_address)`. Detectado en la auditoría del 17/09/2026. La clave es la
**dirección**, no el `contact_id`: un contacto borrado y reimportado tiene id
nuevo, y con una clave basada en el id se le volvería a enviar.

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

Por cuenta remitente, con dos ventanas simultáneas —horaria y diaria— en la **zona horaria que declara la ventana de ejecución de la campaña**, nunca la del sistema operativo. Un portátil que cruza husos horarios no debe cambiar la política de envío.

!! **Este bloque decía «la zona horaria de la empresa»**, en singular, como si
sólo pudiera haber una. El esquema soportaba una por ventana desde la V1
—`execution_window.timezone` existe y `campaign.execution_window_id` la elige—,
así que lo equivocado era el texto, no el diseño. Corregido el 22/09/2026 al
contrastar una pregunta de Dirección con la base de datos (D-10).

El estado vive en `rate_budget`, así que sobrevive a cierres y reinicios. Un cubo en memoria se reiniciaría con la aplicación y permitiría enviar el límite diario varias veces al día.

### El reloj que se mueve hacia atrás

Una etiqueta **más vieja que la guardada no reinicia el contador** (corregido el 23/09/2026).

Sin esto bastaba con atrasar el reloj un día para que la etiqueta cambiara, el contador se reiniciara y la cuenta mandara el cupo diario otra vez. **Mover el reloj era una forma de saltarse el límite del proveedor**, y no hace falta mala fe: un ajuste de NTP tras una deriva hace exactamente eso.

Con la corrección se sigue gastando del presupuesto que había —el lado conservador: se envía de menos, nunca de más— y todo se recoloca solo en cuanto el reloj vuelve a su sitio. La misma regla se aplica al leer: si la fila guardada es de una ventana posterior a la que se pregunta, se devuelve lo que hay, no cero.

Es el caso «reloj del sistema ajustado hacia atrás» de `ESTRATEGIA_QA.md` §3, que estaba enunciado y sin cubrir.

### Ventana de ejecución

`execution_window` define días, horas operativas **y su zona horaria**. Fuera de ventana, los workers no toman trabajo. No se acumula nada: simplemente se espera.

**La ventana pertenece a la campaña, no a la empresa (D-10).** Una empresa puede
tener varias —«México · L-V 9-17 · `America/Mexico_City`» y «España · L-V 9-17 ·
`Europe/Madrid`»— y cada campaña elige la suya. Las dos pueden correr a la vez
desde el mismo equipo, esté donde esté.

La zona se guarda como **nombre IANA**, nunca como desfase. Un desfase cambia con
el horario de verano y un nombre no: guardar `-06:00` haría que la campaña se
desplazara una hora dos veces al año sin que nadie tocara nada.

**Lo que esto NO hace: la hora de cada destinatario.** Se difiere a la v1.3
(D-11). ARLES no sabe dónde vive la gente —una dirección de correo no lleva
ubicación, y un `+52` da el país, no la hora, porque México tiene cuatro zonas—
y adivinarlo mal sería peor que no hacerlo: se enviaría de madrugada mientras el
usuario cree que está resuelto.

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

**Se detecta y se informa. No se toca el presupuesto** (corregido el 23/09/2026 al auditar la fase).

Aquí había una regla que decía lo contrario: al detectar una suspensión, el presupuesto volvía a cero. Tenía su función, su prueba, y la prueba pasaba.

Estaba mal, y el caso que la tumba es una **siesta dentro del mismo día**: el equipo duerme de 10:00 a 18:00 con el cupo diario agotado, despierta, el contador diario vuelve a cero, y esa cuenta manda otros cincuenta. **Cien envíos en un día con un tope de cincuenta** — la ráfaga exacta que este apartado existe para impedir, escrita dentro de su propia solución.

El fallo era arquitectónico. Esa regla se escribió para un cubo de permisos **en memoria**, donde «lo gastado» es un número que hay que decidir qué hacer con él al despertar. En la entrega 4.2b ese cubo se sustituyó por el presupuesto **persistido por etiquetas** (§4), donde lo gastado no se arrastra: se consulta por la etiqueta de la hora y del día actuales. Al despertar, la hora es otra y su contador ya vale cero solo; si el día es el mismo, el contador del día sigue donde estaba. **El problema se disolvió y la regla se quedó, apuntando.**

La detección sigue, y sirve para lo que de verdad sirve: decirle al usuario que su equipo estuvo suspendido y que por eso su campaña va con retraso.

---

## 6. Reintentos

| Tipo de fallo | Respuesta |
|---|---|
| Red, timeout, 5xx transitorio | Reintento con backoff exponencial y jitter |
| **429 con `Retry-After`** | **Se respeta literalmente.** Nunca se ignora ni se acorta |
| 5xx permanente (buzón inexistente) | `permanently_failed` + entrada en supresión |
| 4xx de autenticación | Circuito abierto en la cuenta; **no se reintenta** |
| Supresión detectada | `suppressed`. No es un fallo |

**Esperas (D-12, 23/09/2026):** una **tabla**, no una fórmula — 5, 15 y 30
minutos. **Tres reintentos**, cincuenta minutos de margen total. Después,
`permanently_failed`.

!! **Este bloque decía «base 30 s, factor 2, techo 1 h, cinco intentos»**, que
da siete minutos y medio de margen. Dirección lo cambió el 23/09/2026 con el
argumento correcto: un apagón o una caída de proveedor duran más que eso, así
que los cinco intentos se gastarían dentro del mismo corte y todos los envíos
morirían por un problema que se resolvió solo veinte minutos después.

**El azar (jitter) es de ±20 % alrededor de la espera, no «entre cero y el
máximo».** El jitter importa —sin él, todos los intentos fallidos de un corte de
red reintentan a la vez cuando vuelve—, pero la receta habitual aplicada a estas
cifras haría que algunos reintentos salieran a los diez segundos, anulando D-12.
Cinco minutos pasa a ser entre cuatro y seis.

---

## 7. Disyuntor por cuenta

Tras **N fallos consecutivos** (por defecto 5) en una misma cuenta, el circuito se abre: `email_account.circuit_open_until` se fija y esa cuenta deja de tomar trabajo.

La interfaz lo dice explícitamente y con el motivo. Un fallo silencioso que simplemente ralentiza la campaña es peor que una parada visible.

Se cierra automáticamente al vencer el plazo, o manualmente cuando el usuario corrige el problema (p. ej. renueva la contraseña de aplicación).

### Cómo está construido (auditoría de la fase, 23/09/2026)

**Este apartado describía un disyuntor que no existía.** Las columnas `consecutive_failures` y `circuit_open_until` estaban en el esquema desde la V1, el §13 listaba la comprobación como un paso del bucle, y ningún código las leía ni las escribía. Un esquema que promete algo y un código que no lo cumple no falla: simplemente no protege de nada.

**No todo fallo cuenta, y ésa es la parte que se puede hacer mal.** Que una dirección no exista no es culpa de la cuenta. Una lista importada con cinco direcciones muertas seguidas no dice nada malo del remitente, y si contaran, esa campaña se pararía sola cada pocos minutos sin que el usuario entendiera por qué.

| Fallo | ¿Cuenta para el disyuntor? |
|---|---|
| Red caída, tiempo de espera agotado, servidor saturado | **Sí** — habla del canal |
| «No me mandes nada durante N» | **Sí** — el proveedor dice que este remitente va rápido |
| La credencial no sirve | **Sí, y abre a la primera** |
| Esa dirección no existe | No — habla del destinatario |
| Esa persona pidió la baja | No — ni siquiera es un fallo |

**La credencial no espera a cinco.** Ya sabemos que ningún envío de esa cuenta va a funcionar hasta que una persona la arregle; contar hasta cinco sólo gastaría los reintentos de otros cuatro envíos.

**Y se marca distinto**, porque se arregla distinto: `circuit_open` se cierra solo en media hora; `needs_reauth` no se cierra hasta que alguien renueve la credencial. Marcar los dos igual haría esperar media hora a que se arreglara sola una contraseña caducada.

**Un solo acierto reinicia el contador** — pero **no** borra el aviso de credencial. Un envío que salió no prueba que la contraseña esté renovada, y quitar el aviso escondería lo único que el usuario tiene que hacer.

Se cierra solo al vencer el plazo sin que nadie lo vigile: basta con comparar al leer, igual que hace el presupuesto con sus etiquetas.

---

## 8. Sin rotación de remitentes (T-1)

Al alcanzarse el límite de una cuenta, **la cola pausa**:

> «La cuenta [correo] alcanzó su límite diario de 50 envíos. La campaña continuará mañana a las 09:00.»

**No se salta a otra cuenta.** El usuario puede tener varias cuentas registradas y asignarlas a campañas distintas, pero una campaña tiene un remitente y lo respeta.

Esto es una decisión de producto, no una limitación técnica. Rotar remitentes para superar un límite es evasión, viola las políticas de Google y es el mecanismo exacto por el que se queman dominios. Ver ADR-0008.

**«La cola pausa» es una forma de hablar, no un cambio de estado** (aclarado el 23/09/2026 al auditar la fase). La campaña **sigue en `running`**: el trabajador simplemente deja de encontrar cupo y espera. Cambiarla a `paused` obligaría al usuario a reanudarla a mano cada día al agotar el límite diario, que es exactamente lo contrario de lo que promete la frase «la campaña continuará mañana a las 09:00».

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

### Cómo está construido (entrega 4.4, 23/09/2026)

Las **reglas** —qué acción procede desde qué estado— viven en `arles_core::campana`, no en el botón. Si «no se puede reanudar una campaña detenida» viviera en la pantalla, cualquier otro camino hasta la base se la saltaría sin enterarse.

**Detener toca tres tablas y va en una transacción**: el estado de la campaña, el de sus etapas y el de los envíos pendientes. Escritas por separado, una caída en medio dejaría estados que se contradicen — una campaña detenida con la cola intacta, o una cola cancelada con la campaña marcada como enviando, que diría «enviando» sobre una cola vacía sin fallar y sin enviar nada.

**Qué se cancela exactamente.** Los intentos en `queued`, `claimed` y `failed`. **`sending` no**, y no es un olvido: esa fila ya se le pasó al proveedor, y cancelarla diría que ese mensaje no salió cuando puede que sí. La termina el trabajador que la tiene en la mano; lo que impide que siga con la siguiente es el freno, no la cancelación.

**La cifra que se avisa es la que se cancela.** `cuantos_quedan_sin_enviar` y la cancelación miran la misma lista de estados, y hay una prueba que las enfrenta. Si miraran listas distintas, la confirmación prometería una cosa y ocurriría otra — sobre algo que no se puede deshacer.

**La parada de emergencia se guarda en la base** (migración V6, tabla `engine_control`), no en memoria. El caso concreto: alguien ve que la campaña recién activada lleva la plantilla equivocada, pulsa parar y cierra el programa. Al día siguiente ARLES arranca, no encuentra ninguna parada, y sigue enviando la plantilla equivocada a los cuatro mil que faltaban.

Se guarda **cuándo** se puso, no un sí/no, para que la pantalla pueda decir «detenido desde las 14:32» en vez de un «detenido» que invita a preguntarse si se quedó puesto de ayer (§94). Volver a pulsar no reescribe ese instante.

**Levantar la parada no reanuda nada.** Las campañas en pausa siguen en pausa. Una parada de emergencia se pulsa porque algo iba mal, y lo último que debe hacer el botón de quitarla es poner a enviar todo a la vez.

**Y ante la duda, se para.** El freno del bucle (`arles_motor::bucle::freno_de_la_base`) pregunta a la base, y si la base no puede contestar —un estado corrupto, una base editada por fuera— responde que sí hay que parar. Los dos errores no se parecen: parar de más se ve, porque la campaña no avanza y alguien pregunta; enviar de más no se ve hasta que los correos ya salieron.

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

### Cómo está construido (entrega 4.5, 23/09/2026)

Vive en `arles_core::duracion`, y **simula hora a hora** en vez de dividir. La tentación es `audiencia / limite_diario` por los días laborables: da un número plausible y está mal por tres motivos.

**Hay dos topes, no uno.** Diez por hora en una ventana de nueve horas son noventa, pero si el tope diario es cincuenta, se mandan cincuenta. Y al revés: un tope diario de cien en una ventana de dos horas con cinco por hora son diez. Manda el más pequeño.

**El primer día casi nunca está entero.** Quien activa a las 16:30 con ventana hasta las 18:00 tiene una hora útil, no nueve. La hora en curso **no cuenta**: se empieza en la siguiente en punto, porque una estimación que se queda corta se descubre tarde y mal.

**Los días no duran lo mismo.** El cambio de horario hace días de veintitrés y de veinticinco horas. México ya no lo aplica (D-10), pero una campaña en horario de Madrid sí.

Simular hora a hora sale gratis —cinco años son 43 800 vueltas— y los tres casos salen solos en vez de haber que acordarse de ellos.

**Se dice cuál de los dos topes frena.** «Tu campaña tarda cuatro meses» deja al usuario sin saber qué tocar; «tarda cuatro meses **porque el tope diario es 50**, cuando el horario daría para 90» le dice dónde está el freno. Sin esto, lo normal es ampliar el horario de 9-18 a 8-20 y no notar ninguna diferencia.

**Dos respuestas que no son una fecha:**

| Caso | Qué se dice |
|---|---|
| El tope efectivo es cero | «Con este tope no saldría ni un mensaje» |
| No cabe en cinco años | «{faltan} mensajes se quedarían sin enviar» |

El primero es el fallo más difícil de diagnosticar de todos: la campaña quedaría marcada como enviando y no enviaría nada, **sin nada roto que mirar**. El segundo evita dar «terminará el 3 de marzo de 2041», que es correcto e inútil.

**Un fallo que encontró su propia prueba.** El contador de días con envío se apoyaba en la lista recortada de catorce días; a partir del día quince esa lista deja de crecer, su último elemento se queda congelado, y cada hora de envío contaba como un día nuevo. Una campaña de cinco mil correos a cincuenta diarios decía **444 días** en vez de 100. Nadie lo habría notado: 444 es perfectamente plausible para una campaña larga, sólo que cuatro veces mayor que el real.

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

### La tercera respuesta (entrega 4.5, 23/09/2026)

Vive en `arles_core::preflight`. Cada punto tiene **tres** respuestas, no dos: pasa, falla, y **no se ha comprobado**.

La tercera existe porque varias comprobaciones necesitan piezas que llegan después —los adaptadores de proveedor son de la Fase 5, las plantillas de la Fase 6—. Un preflight que devolviera «pasa» para lo que todavía no sabe mirar sería peor que no tenerlo: daría por bueno exactamente lo que no ha visto.

Es la misma regla que el validador aplica a las sondas: **omitida no es pasada, y se dice**. De ahí la consecuencia que lo gobierna: **un punto bloqueante sin comprobar impide activar**, igual que uno que falla. La diferencia se enseña —el mensaje no es el mismo, y las cifras se cuentan aparte— pero el botón está igual de apagado. Un punto que falla se arregla; uno sin comprobar se comprueba.

**Siete de los ocho bloquean.** El único que no es el de SPF/DKIM/DMARC: un dominio sin DMARC entrega peor, pero entrega, y bloquear por eso dejaría fuera a quien no controla su DNS —en una empresa pequeña, casi todo el mundo— por algo que ARLES no le puede arreglar (§154: informar, no dictar).

**Se revisan los ocho aunque el primero falle.** Parar en el primero haría que el usuario arreglara la credencial, volviera, y descubriera que además la audiencia está vacía: tres viajes para tres problemas que caben en una pantalla.

**El simulador de duración deja de ser informativo y se convierte en una puerta.** Si el cálculo dice que no sale nada, o que no cabe en cinco años, el punto del horario **falla** y no se puede activar.

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

### Lo que se construyó, y las dos cosas que cambiaron al construirlo

El bucle vive en `crates/arles-motor/src/bucle.rs`, y sigue este §13 paso por paso. Dos detalles no estaban escritos aquí y se decidieron al implementarlo (entrega 4.3b, 23/09/2026):

**Una credencial caducada no gasta reintentos.** El esquema de arriba dice «AuthFallida → abrir circuito», y lo que no decía es qué pasa con el envío que estaba en curso. Contarlo como intento fallido gastaría uno de los tres reintentos de D-12 sin que ningún servidor de correo hubiera visto el mensaje. Con tres campañas paradas por una contraseña caducada, media audiencia se quedaría sin reintentos y, al arreglar la cuenta, esos envíos se rendirían enseguida sin haberlo intentado nunca de verdad.

El intento vuelve a la cola **intacto** (`attempt_count - 1`) y con hora de reintento *ahora*: esperar media hora no arregla una credencial, eso lo arregla una persona. Lo que impide que el motor gire mientras tanto no es la hora, es que se detiene y avisa.

**Un envío ya empezado no se cierra como «baja».** El proveedor puede contestar «esa persona pidió la baja» **mientras** se le manda —pasa sobre todo en WhatsApp, donde la baja la gestiona Meta—. Parecía que la fila debía quedar como `suppressed`, y no procede: ese estado no cuenta como envío consumido, así que una fila que llegó a `sending`, donde el mensaje **pudo** salir, quedaría fuera del presupuesto de la cuenta. Se cierra como `permanently_failed` con la clave que dice por qué, y la dirección entra en la lista de bajas, que es lo que de verdad protege a esa persona.

---

## 14. Una sola instancia sobre una sola base

El compare-and-swap del §2 impide que dos trabajadores se lleven el mismo envío. El daño de dos ARLES abiertos a la vez no está ahí: está en el **arranque**.

Al abrir, ARLES ordena la cola y pasa a `presumed_sent` todo lo que quedó en `sending`, porque sólo una caída deja filas así (§3). Si un segundo ARLES arranca mientras el primero está enviando, encuentra filas en `sending` que **no son restos de nada** —son los envíos en curso del otro— y las marca como no confirmadas.

El resultado no es un duplicado; es peor de leer: una lista de envíos dudosos que el usuario tiene que revisar uno a uno cuando todos salieron bien. Esa lista deja de servir para lo único que sirve.

**El mecanismo.** Un archivo `arles.lock` junto a la base, con un identificador y un **latido**: su fecha de modificación, reescrita cada pocos segundos por el propio bucle del trabajador. Por eso el cerrojo llegó con el bucle y no antes.

| Al arrancar se encuentra | Qué significa | Qué se hace |
|---|---|---|
| No hay archivo | No hay nadie | Se toma |
| Latido de hace menos de 90 s | Hay otro ARLES vivo | **No se abre**, y se dice hace cuántos segundos latió |
| Latido de hace más de 90 s | El otro murió sin soltarlo | Se toma, y se confirma volviendo a leer |

Los noventa segundos son holgados a propósito: el margen es para el equipo que va despacio —un disco lento, una copia de seguridad—, no para el que murió. Robarle el cerrojo a un ARLES vivo es exactamente el fallo que esto evita.

**Sin latido, el fallo es peor que el que se evita.** Un cerrojo que sólo comprueba si el archivo existe deja el programa inservible tras cualquier caída: el archivo se queda puesto y nadie puede volver a abrir ARLES sin saber que hay que borrar un archivo a mano.

**Lo que no se promete.** No es un cerrojo del sistema operativo. Dos ARLES que arrancaran en el mismo milisegundo sobre un cerrojo caducado podrían creerse ambos dueños; por eso al robar uno viejo se escribe, se espera y se vuelve a leer, y quien lea un identificador ajeno se retira. La ventana queda cerrada en la práctica, no por demostración. La garantía dura sigue siendo la de siempre: el `UNIQUE(campaign_id, channel, contact_address)` del esquema. **Nadie recibe dos correos aunque este cerrojo falle.**

Y el proceso que pierde el cerrojo —porque se quedó congelado más de noventa segundos y otro lo dio por muerto— se entera al latir, y deja de enviar.
