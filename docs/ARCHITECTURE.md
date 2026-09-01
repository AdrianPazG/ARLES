# Arquitectura

Propuesta de Fase 1. **Pendiente de autorización de Dirección.**

## 1. Pila

| Capa | Decisión | ADR |
|---|---|---|
| Cascarón de escritorio | Tauri 2 | [0001](DECISIONS/ADR-0001-tauri.md) |
| Frontend | Vue 3 + TypeScript `strict` + Vite | — |
| Estado de UI | Pinia | — |
| Núcleo | Rust | [0001](DECISIONS/ADR-0001-tauri.md) |
| Almacenamiento | SQLite en modo WAL | [0002](DECISIONS/ADR-0002-sqlite.md) |
| Acceso a datos | `sqlx` con verificación en compilación | [0003](DECISIONS/ADR-0003-sqlx.md) |
| Secretos | Llavero del sistema vía `keyring` | [0005](DECISIONS/ADR-0005-secretos.md) |
| Correo | `lettre` para MIME y SMTP; HTTP tipado para Graph y Gmail | — |
| XLSX / CSV | `calamine` con topes duros | — |

**Regla de estado.** Pinia guarda estado de *interfaz*. La verdad vive en SQLite y llega por comandos
tipados. Nunca se replica la base en memoria: con 100 mil contactos eso es un fallo de memoria garantizado.

## 2. Arquitectura general

Monolito modular **en un solo proceso**. Sin microservicios, sin colas distribuidas, sin contenedores.
El motor de ejecución es una tarea asíncrona dentro del núcleo de Rust, no un proceso aparte
([ADR-0004](DECISIONS/ADR-0004-motor-en-proceso.md)).

### Dominios

| Anillo | Dominios | Regla de dependencia |
|---|---|---|
| Núcleo | `Identity` `Company` `Settings` `Audit` | No dependen de nadie |
| Datos | `Contacts` `Lists` `Imports` `Suppression` `Templates` | Solo del núcleo |
| Correo | `EmailAccounts` `Deliverability` | Tras el rasgo `EmailProvider` |
| Ejecución | `Campaigns` `Scheduling` `Sending` `Analytics` | Ver frontera crítica |
| Plataforma | `Licensing` `Updates` `Backups` `Diagnostics` `Device` | Nadie depende de ellos |

Dos ajustes sobre §10: `Providers` se absorbe en `EmailAccounts` (eran la misma cosa vista desde dos
lados) y se separa `Sending` de `Scheduling`, porque *cuándo* y *cómo* tienen ciclos de prueba distintos.

**Frontera crítica.** `Campaigns` **define**; `Sending` **ejecuta**. Nunca al revés. Esa separación es lo
que permite una futura edición Cloud sin reescribir el producto (§127).

**Regla verificable en CI:** ningún dominio fuera de `EmailAccounts` puede nombrar a Gmail, Graph ni SMTP.

## 3. Backend mínimo de TELEMETRY

Tres responsabilidades, ninguna toca datos de clientes:

1. Manifiesto de actualización firmado, en almacenamiento estático tras CDN. Sin lógica, sin base de datos.
2. Emisión y validación de licencias. API sin estado.
3. Recepción de paquetes de soporte, solo cuando el usuario los envía a propósito tras revisar su contenido.

**Invariante de privacidad.** Ningún contacto, campaña, mensaje ni credencial sale nunca del equipo del
cliente hacia infraestructura de TELEMETRY. Debe existir una prueba automatizada que lo verifique. Si
alguna petición lleva un dato de cliente, es un defecto de seguridad, no una función.

## 4. Modelo de datos

Se adoptan las 28 entidades del §32 con cuatro cambios estructurales, cada uno cerrando un riesgo concreto:

| Cambio | Riesgo que cierra |
|---|---|
| Eliminar `ApplicationUser`; usar `operator_label` en `AuditLog` | Elimina un almacén de contraseñas y sus flujos de recuperación |
| `Message.idempotency_key` con índice `UNIQUE` | §55 — el duplicado se vuelve imposible en la base |
| `CampaignRecipient` con `UNIQUE(campaign_id, contact_id)` | §108 — "la lista contiene duplicados" |
| `SuppressionEntry` con `UNIQUE(company_id, email_normalized)` | §39 — una importación no puede revivir una baja |

### Grupos

- **Organización** — `Company` `License` `Device` `ApplicationSetting`.
  `Company` es la raíz. Todas las tablas de datos llevan `company_id`: hoy siempre una, mañana no (§7, §127).
- **Contactos** — `Contact` `ContactList` `ContactListMember` `Tag` `ContactTag` `CustomField`
  `CustomFieldValue` `Import`. `Contact.email_normalized` es columna generada e indizada.
- **Correo** — `EmailAccount` `SenderIdentity` `DomainHealthCheck`.
  **`EmailAccount` no guarda credenciales**: guarda un `secret_ref` al llavero del sistema.
- **Contenido** — `Template` `TemplateVersion`. `Message` apunta a `template_version_id`, no a
  `template_id`: editar una plantilla no puede reescribir el historial (§43).
- **Ejecución** — `Campaign` `CampaignRecipient` `Message` `MessageAttempt` `MessageEvent` `ExecutionJob`.
  `MessageAttempt` es la bitácora inmutable y la fuente de los contadores de límite.
- **Protección** — `SuppressionEntry` `AuditLog` `BackupRecord` `UpdateRecord`.
  `AuditLog` es solo-inserción: sin `UPDATE`, sin `DELETE`.

### Integridad (§113)

`PRAGMA foreign_keys = ON` en **cada** conexión — SQLite lo trae apagado por omisión. `journal_mode = WAL`.
`busy_timeout` configurado. Toda escritura de varias tablas dentro de una transacción.

**Fechas.** Todo en UTC ISO-8601, salvo las *reglas* de ventana horaria, que guardan hora local más
identificador IANA. Una ventana "09:00–18:00 America/Mexico_City" debe guardarse como regla, no como un
UTC ya calculado: si no, un cambio de horario de verano desplaza la campaña sin que nadie lo note. Los
eventos ya ocurridos sí se guardan como instantes UTC. Esta distinción produce la mayoría de los errores
de planificador.

### Borrado (§148)

| Acción | Efecto | Recuperable | Aplica a |
|---|---|---|---|
| Archivar | `archived_at`. Sale de vistas, conserva historial | Sí | Listas, campañas, plantillas |
| Borrado suave | `deleted_at`. Invisible, excluido de envíos | Sí, 30 días | Contactos |
| Borrado definitivo | Se elimina la fila | No | Contactos, por derecho de cancelación |

**Excepción documentada.** Borrar definitivamente un contacto **no** borra su entrada de supresión: si se
borrara, un reimporte volvería a contactar a quien pidió no serlo. La entrada conserva solo el correo
normalizado y el motivo. Esta tensión entre derecho de cancelación y derecho de oposición debe explicarse
en el aviso de privacidad y **revisarse con un abogado**.

## 5. Motor de ejecución

### 5.1 Un solo escritor

El motor vive dentro del proceso de la aplicación, con la **única conexión de escritura** del sistema.
Cerrar la ventana oculta la interfaz; la aplicación sigue en bandeja o barra de menú (§51). Salir de verdad
detiene el motor, y ARLES lo dice si hay campaña activa.

Consecuencia: *"dos procesos intentan enviar el mismo mensaje"* (§108) deja de ser un caso de prueba y pasa
a ser imposible.

### 5.2 Idempotencia (§55)

1. Al **preparar**, cada destinatario genera un `Message` con su `idempotency_key`. Preparar dos veces:
   el índice `UNIQUE` rechaza la segunda.
2. Al **tomar el trabajo**: `UPDATE message SET state='processing' WHERE id=? AND state='queued'`.
   Si afecta cero filas, otro lo tomó y se aborta. La carrera se resuelve en el motor de la base.
3. Al **aceptar el proveedor**, se escriben `provider_message_id`, estado e intento en la misma transacción.
4. Doble clic en la interfaz: el botón se deshabilita, pero eso es cosmético. La defensa real es que
   activar dos veces la misma campaña es una transición de estado inválida.

**Ventana irreducible.** Entre que el proveedor acepta y ARLES lo escribe en disco existe un instante en
que un corte de energía deja el mensaje enviado y no registrado. No hay arquitectura sin transacción
distribuida con el proveedor que lo elimine, y los proveedores no la ofrecen.

Tratamiento: al arrancar, toda fila en `processing` más de N minutos pasa a `unknown`. **ARLES no reintenta
automáticamente** — reintentar es la forma de duplicar. Muestra la lista y pide decisión humana, indicando
que el proveedor pudo haberlo enviado (§156, §167).

### 5.3 Contadores derivados (§57)

Los contadores **no se almacenan: se derivan**.
`SELECT COUNT(*) FROM message_attempt WHERE account_id=? AND accepted_at >= ?` sobre un índice.

No pueden divergir de la realidad, sobreviven al reinicio sin código de recuperación, son auditables y
eliminan toda una clase de condiciones de carrera. Cuesta una consulta por envío: irrelevante a estos
volúmenes.

### 5.4 Estados de la cola (§56)

`prepared` → `approved` → `scheduled` → `queued` → `processing` → `accepted` | `failed` | `unknown`,
más `suppressed` y `cancelled` como terminales.

`accepted` significa **aceptado por el proveedor**, nunca "entregado".

**La supresión se comprueba dos veces:** al preparar y otra vez inmediatamente antes de enviar. Eso cubre
el caso del §108 *"un contacto se suprime mientras la campaña está activa"*.

### 5.5 Suspensión y horario (§53, §54)

- **Detección:** comparación de reloj monótono contra reloj de pared en cada ciclo.
- **Reacción:** se descarta el plan y se recalcula. **Nunca se acumula ni se compensa.** Si la ventana pasó,
  esos envíos van al siguiente día hábil configurado.
- **Estado visible:** el motor pasa a `SUSPENDIDO` y explica qué pasó y cuándo retoma.
- **Mantener despierto:** opción explícita, apagada por omisión.

### 5.6 Errores y disyuntor (§58–§60)

| Clase | Ejemplo | Reintento | Acción |
|---|---|---|---|
| `auth` | 401 | Una vez tras refrescar | Cuenta a *Requiere reconexión*, campaña en pausa |
| `permission` | 403 | No | Pausar cuenta. Mostrar el mensaje del proveedor sin traducirlo |
| `rate_limit` | 429 | Sí | **Obedecer `Retry-After` literalmente.** Si no viene, retroceso exponencial |
| `provider` | 5xx | Sí, máx. 5 | Retroceso exponencial |
| `network` | DNS, TLS, timeout | Sí | Motor a `SIN CONEXIÓN` |
| `recipient` | 550 | No | `failed` y proponer supresión. Nunca suprimir en automático |
| `content` | Rechazo por contenido | No | Detener la campaña completa |

**Disyuntor por cuenta:** N fallos consecutivos abren el circuito; enfría con retroceso exponencial; cierra
con un mensaje de prueba. Estado visible en Salud del Remitente.

### 5.7 Distribución en la ventana

Intervalo objetivo = ventana ÷ cupo restante, con variación acotada de ±15 %. Activada por omisión,
desactivable, documentada.

**No es simulación de comportamiento humano** (§93 lo prohíbe y se respeta). Es limitación de caudal: la
misma técnica que cualquier cliente de API usa para no saturar un servidor. Reduce los 429 y las pausas
del disyuntor.

### 5.8 Límites de proveedor

**ARLES no codifica ningún límite de proveedor.** Varían por tipo de cuenta, antigüedad, reputación y
decisiones internas que no se publican con precisión; cualquier número codificado estaría mal para algún
cliente y quedaría obsoleto.

En su lugar: la organización configura sus máximos; el motor observa las respuestas reales y obedece 429 y
`Retry-After`; Salud del Remitente muestra la tasa de limitación **observada**, que es un dato medido.

## 6. Abstracción de proveedor

El rasgo `EmailProvider` se escribe **antes** que cualquier adaptador. Operaciones:
`authenticate` `disconnect` `validate` `send` `send_test` `refresh_authentication` `capabilities`
`limits` `provider_status`.

Adaptadores: `SmtpProvider`, `MicrosoftProvider`, `GoogleProvider` (tras bandera de función).

## 7. Rendimiento — objetivos

**Todos son objetivos, ninguno está medido.** Se marcan NO VERIFICADO hasta tener cifras reales (§168).

| Operación | Objetivo | Estado |
|---|---|---|
| Arranque en frío hasta interactivo | < 2.0 s | NO VERIFICADO |
| Abrir base y pintar panel | < 400 ms | NO VERIFICADO |
| Filtrar 100 mil contactos | < 300 ms | NO VERIFICADO |
| Importar 50 mil filas XLSX | < 90 s | NO VERIFICADO |
| Memoria durante esa importación | < 400 MB | NO VERIFICADO |
| Respaldo de 100 mil contactos | < 30 s | NO VERIFICADO |
| CPU del motor en reposo | < 1 % | NO VERIFICADO |
| Bloqueo del hilo de interfaz | < 50 ms | NO VERIFICADO |

Hasta que exista medición reproducible sobre hardware real, **ARLES no debe afirmar en ningún material que
soporta 100 mil contactos** (§102). Se publica el número medido, con máquina y fecha.

## 8. Requisito de testabilidad

**El motor no debe llamar nunca a `SystemTime::now()` ni a la red directamente.** Recibe una fuente de
tiempo y un cliente de red como dependencias inyectables. Sin eso, la mitad de los casos obligatorios del
§108 —suspensión, cambio de reloj, caída a media transacción— son imposibles de probar de forma
automatizada.

Es una decisión de arquitectura, no de QA, y debe tomarse en la Fase 3.
