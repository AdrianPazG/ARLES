# ADR-0013 · Origen de contactos, purificación y envío canario

**Estado:** aceptado · **Fecha:** 2026-09-12 · **Decide:** Dirección + Tech Lead
**Resuelve:** P-03 · **Riesgos:** R-13, **R-20** · **Depende de:** P-09 (bloquea los textos)

---

## Contexto

Dirección planteó la política de origen de contactos para la Fase 3. Tres piezas:

1. TELEMETRY obtuvo sus contactos del **INEGI**, presumiblemente del DENUE.
2. Como es **técnicamente imposible** verificar si un cliente compró su lista,
   la responsabilidad se delega por EULA más una **casilla obligatoria**.
3. Para evitar que el disyuntor pause las campañas, se haría una
   **«purificación básica»** al importar: sintaxis y comprobación de MX.

Las tres son razonables. Dos de ellas descansan sobre premisas que al
verificarlas resultaron falsas o incompletas.

---

## Lo que se verificó, y con qué solidez

> ⚠️ La investigación se hizo en un entorno con el acceso bloqueado a
> `inegi.org.mx`, `diputados.gob.mx` y el DOF. **Nada de lo legal está
> verificado contra el texto primario.** Lo que sigue es orientación para la
> revisión jurídica de **P-09**, no una conclusión.

### Una licencia de datos abiertos no es una base legal de tratamiento

Los Términos de Libre Uso del INEGI permiten explotación comercial con
obligación de citar la fuente. **Eso autoriza redistribuir el dataset; no
autoriza usar esos correos para prospección comercial.** Son dos regímenes
distintos y la afirmación «lo obtuvimos legalmente» sólo cubre el primero.

### «Son negocios, no personas» no se sostiene para buena parte del DENUE

El DENUE está dominado por micronegocios cuyo titular es una **persona física
con actividad empresarial**. Ahí el correo es dato personal de una persona
física identificada. La exclusión del Reglamento alcanza a **personas morales**,
no a esto.

### El primer correo ya debe llevar aviso de privacidad

Cuando los datos se obtienen **indirectamente** de una fuente de acceso
público, subsiste la obligación de poner el aviso de privacidad a disposición
del titular **en el primer contacto**. Esto deja de ser una nota legal y pasa a
ser una **función del producto**.

### Hay un segundo eje legal que no estaba en el radar

El **art. 18 BIS de la Ley Federal de Protección al Consumidor** prohíbe enviar
publicidad a quien manifestó que no la desea, y hace **corresponsables a los
proveedores anunciados**. Aplica además de la ley de datos personales. El
REPEP de Profeco, en cambio, cubre teléfono y SMS — **no correo**: no hay
registro que consultar.

### El riesgo mayor no es el regulatorio

La **Acceptable Use Policy de Google Workspace prohíbe expresamente generar o
facilitar correo masivo no solicitado**, y las políticas de OAuth permiten a
Google revocar el acceso a las APIs. La verificación **no es permanente**.

Una revocación no afecta a un cliente: **apaga Gmail para todos a la vez**. Y
un EULA no protege de eso, porque Google no es parte del contrato. Registrado
como **R-20**.

---

## Decisión

### 1 · La responsabilidad se delega, y se documenta con prueba utilizable

Se acepta la propuesta de Dirección: ARLES **no audita ni supervisa** el
contenido del archivo que el cliente importa. No puede, y fingir que sí sería
peor.

Lo que sí hace es dejar **prueba de lo que el cliente afirmó**, en dos momentos
distintos y por un motivo concreto: una lista importada en marzo puede enviarse
en noviembre, y una sola afirmación cubriendo ambos momentos es prueba débil.

| Momento | Qué se afirma | Dónde queda |
|---|---|---|
| **Al importar** | Origen lícito de *este archivo*, más el **origen concreto** elegido de una lista cerrada | `import_batch` + `audit_log` |
| **Al publicar la campaña** | Que *este envío* se ajusta a lo declarado, con el número de destinatarios a la vista | `campaign` + `audit_log` |

El origen concreto —formulario propio · clientes existentes · evento o feria ·
directorio público · otro— **es mejor prueba que una casilla sola**: registra
*qué* afirmó, no sólo *que* afirmó. Y deja ver patrones de riesgo entre
clientes.

**Se guarda el texto íntegro y su hash, no un booleano.** Si la redacción
cambia en 2027, hay que poder demostrar qué decía cuando el cliente la aceptó.

> **El texto exacto queda bloqueado por P-09.** Un aviso que cita una ley
> abrogada es peor que no citar ninguna: aparenta rigor. Hasta que haya
> revisión jurídica, la Fase 3 construye el mecanismo con texto marcado como
> provisional y visible como tal en la interfaz.

### 2 · El aviso de privacidad en el primer contacto es una función, no una nota

El preflight comprueba que la plantilla incluye aviso de privacidad y mecanismo
de ejercicio de derechos ARCO. Es **advertencia, no bloqueo** —§154, autonomía
del cliente—, pero visible y explicando el riesgo.

### 3 · La purificación se acepta corregida, y se le quita el papel protagonista

La propuesta esperaba que la purificación evitara las pausas. **No lo va a
hacer**, y conviene decirlo antes de construirla:

> La comprobación de MX detecta **dominios muertos**. No detecta **buzones
> muertos**. En una lista de directorio, el grueso de los rebotes son buzones
> que ya no existen en dominios que funcionan perfectamente —
> `ventas@empresa-que-sigue-viva.mx`, donde la persona se fue hace tres años.

Se queda porque es barata y quita basura evidente, con tres correcciones:

**a) La regla es MX, y si no hay, A/AAAA.** El RFC 5321 §5 establece el MX
implícito: si un dominio no tiene MX pero sí registro de dirección, el correo
se entrega ahí. Descartar por ausencia de MX produce **falsos positivos** — y
un falso positivo es peor que un rebote, porque el cliente nunca se entera de
que no le escribiste a alguien real.

**b) Deduplicada por dominio, con tope, caché y timeout.** 20 000 contactos no
son 20 000 consultas: son tantas como dominios únicos. Sin tope, un CSV con
20 000 dominios aleatorios convierte a ARLES en un generador de tráfico DNS y
agota el resolver del cliente. **Va al modelo de amenazas.**

**c) El resultado caduca y se revalida en el preflight.** Un dominio válido en
marzo puede estar muerto en noviembre.

Y una consecuencia de privacidad que hay que declarar: **el resolver DNS del
cliente ve el conjunto de dominios objetivo de su campaña.** Es una revelación
de su lista de prospectos a un tercero, aunque sea su propio ISP.

### 4 · Lo que sí evita las pausas: el envío canario

**Esta es la pieza que la propuesta no tenía y que resuelve lo que buscaba.**

Por encima de un umbral de destinatarios, la campaña **no sale entera**:

1. Se envían los primeros **N** (por defecto 200).
2. La campaña **se detiene sola** y mide la tasa de rebote real.
3. Por debajo del umbral, continúa automáticamente.
4. Por encima, para y lo dice con cifras:

> **La campaña se detuvo tras los primeros 200 envíos.**
> Rebotaron 34 (17 %). A ese ritmo, esta campaña generaría unos 3 400 rebotes y
> pondría en riesgo la reputación de tu dominio.
> **Los 19 800 restantes siguen en cola, sin enviar.** Puedes depurar la lista
> y reanudar.

Mide **el comportamiento real** en vez de adivinarlo, que es lo único que
protege de verdad el dominio. Y convierte el disyuntor de «algo que te
interrumpe» en «algo que te avisó a tiempo».

### 5 · Dos disyuntores, no uno

Conviene no mezclarlos, porque tienen umbrales y significados distintos:

| Disyuntor | Dispara por | Alcance | Estado |
|---|---|---|---|
| **Por cuenta** | N fallos **consecutivos** de autenticación o conexión | La cuenta deja de tomar trabajo | Ya especificado (MOTOR §7) |
| **Por campaña** | **Tasa de rebote** sobre una ventana | La campaña pasa a `paused` | **Nuevo en este ADR** |

El segundo exige **muestra mínima**: pausar por 3 rebotes de 5 envíos sería
pausar siempre. Y la pausa **nunca es un bloqueo definitivo** — el cliente
depura y reanuda, y la idempotencia garantiza que nadie recibe dos veces.

### 6 · El informe de importación es un entregable, no un contador

Si el cliente sube 20 000 y entran 18 000, **tiene derecho a saber qué pasó con
los 2 000**. Hoy el esquema guarda sólo los totales; hace falta el detalle por
fila.

> **Importación terminada.** 18 000 de 20 000 contactos añadidos.
>
> | Motivo | Filas |
> |---|---|
> | Duplicados en tu archivo | 1 240 |
> | Formato inválido | 410 |
> | Dominio inexistente | 290 |
> | Ya estaban suprimidos | 60 |
>
> `[Descargar los 2 000 rechazados]`

Dos condiciones sobre ese archivo: contiene datos personales —necesita política
de retención— y **pasa por la neutralización de fórmulas** del THREAT_MODEL
§4.1. Sin eso le entregamos al cliente un CSV que ejecuta fórmulas al abrirlo
en Excel.

### 7 · Un rechazo de importación NO es una supresión

`suppression_entry` es la tabla con más autoridad del sistema (§39). Una
dirección con sintaxis inválida o dominio muerto **no entra ahí**: es un
rechazo de importación, no un bloqueo deliberado y permanente. Mezclarlos
ensucia el único registro que nunca debe tener ruido.

### 8 · Gmail no se posiciona para prospección en frío

Decisión de producto derivada de R-20. **SMTP con dominio propio del cliente**
para prospección; **Gmail** para relación con clientes existentes. La
documentación y el onboarding lo dicen.

Dato que lo hace viable: las exigencias de Google para remitentes masivos
—baja de un clic, RFC 8058— aplican **a partir de 5 000 mensajes diarios a
Gmail**. Con los límites reales de Gmail (500/día en cuenta personal, 2 000/día
en Workspace), un cliente enviando por Gmail **no puede alcanzar ese umbral**.
La limitación que PRIVACIDAD_LFPDPPP.md §5 documenta honestamente **no le
afecta**. Sí afecta al camino de SMTP propio a volumen.

---

## Lo que se descarta, y por qué

**Validación por SMTP (`RCPT TO` sin enviar).** Gmail, Microsoft 365,
Proofpoint y Mimecast responden 250 a cualquier dirección o 4xx a lo que huela
a verificador. Y miles de conexiones `RCPT` sin entregar es **conductualmente
idéntico a un ataque de cosecha de directorio**: lleva a throttling, tarpitting
y listas negras. Se descarta por completo.

**Validadores externos (ZeroBounce, NeverBounce) en v1.2.0.** Anuncian 99,6 %;
las pruebas independientes los sitúan en **93–99 %**, y dejan **20–40 % de una
lista B2B sin resolver** como «catch-all». Además, mandarles la lista es una
**transferencia a un tercero** normalmente fuera de México: exige constar en el
aviso de privacidad y contrato de encargado. Se difiere, como proponía
Dirección.

---

## Consecuencias

### Cambios de esquema que la Fase 3 debe traer

| Qué | Dónde |
|---|---|
| Detalle por fila de cada rechazo | Tabla nueva `import_rejection` |
| Quién aceptó, cuándo y con qué versión del texto | `import_batch`: falta `created_by` y la versión |
| Afirmación al publicar la campaña | `campaign`: no existe el campo |
| Caché de comprobación de dominio con caducidad | Tabla nueva |
| Estado y umbral del canario | `campaign` |

### Lo que hay que aceptar

- **La purificación no evitará todas las pausas.** Quita quizá un 5–15 % en una
  lista de directorio. El resto sólo se revela al enviar, y para eso está el
  canario.
- **El EULA no protege de Google.** Lo que protege es lo ya construido: cuentas
  del cliente, sin rotación de remitentes (ADR-0008), límites inevadibles.
- **Los textos legales están bloqueados por P-09** hasta revisión jurídica.

### Pendiente que Dirección debe mover

- **P-09** — qué ley rige hoy. Bloquea los textos visibles.
- **P-10** — cobertura real de correo en el DENUE. Se resuelve descargando el
  CSV y contando no nulos; decide cuánto vale invertir en validación.

---

## Alternativas descartadas

**Bloquear la importación si la tasa de dominios muertos supera un umbral.**
Paternalista y evitable: el cliente exporta, limpia a mano y vuelve. El canario
mide lo que de verdad importa y llega en el momento en que importa.

**No pedir afirmación ninguna y confiar sólo en el EULA.** La afirmación en el
momento del acto tiene un valor probatorio que un contrato firmado una vez no
tiene, y cuesta una casilla.

**Auditar el contenido del archivo.** Imposible y contraproducente: implicaría
que ARLES lee y juzga los contactos del cliente, que es exactamente lo que el
modelo «los datos no salen de tu máquina» promete que no ocurre.
