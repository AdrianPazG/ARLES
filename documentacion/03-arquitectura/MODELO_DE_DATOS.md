# Modelo de datos

**Proyecto:** ARLES RELAY I · v1.2.0
**Motor:** SQLite con SQLCipher · migraciones con `refinery`
**Escala objetivo:** 500 000 contactos (T-7)

---

## 1. Convenciones

- Identificadores: `TEXT` con UUID v7 — ordenables por tiempo, lo que mantiene los índices compactos frente a UUID v4.
- Marcas de tiempo: `TEXT` en ISO-8601 UTC. **La zona horaria de la empresa se guarda aparte** y se aplica al presentar y al calcular ventanas.
- Booleanos: `INTEGER` 0/1.
- Todas las entidades de negocio llevan `company_id`, incluso con una sola empresa: es lo que no cierra la puerta a ARLES RELAY CLOUD.
- Borrado lógico (`deleted_at`) donde la auditoría lo exige; borrado físico donde los derechos ARCO lo exigen (§3.4 de este documento).
- `PRAGMA journal_mode = WAL`, `PRAGMA foreign_keys = ON`.

---

## 2. Entidades

### `company`
La organización operadora.

```
id, commercial_name, logo_path, website, country, timezone,
corporate_email, default_signature_id, created_at, updated_at
```

`timezone` es el identificador IANA (`America/Mexico_City`). **Toda la lógica de ventanas de ejecución usa este campo, no la zona del sistema operativo** — un portátil que viaja no debe cambiar la política de envío de la empresa.

### `email_account`
Cuenta remitente.

```
id, company_id, display_name, email_address, provider_kind,
credential_ref, status, daily_limit, hourly_limit,
consecutive_failures, circuit_open_until, created_at, updated_at
```

**`credential_ref` es una referencia al llavero del sistema operativo, nunca una credencial.** Ver `04-seguridad/MODELO_DE_SECRETOS.md`.

`provider_kind`: `smtp` en v1.2.0; `google` y `microsoft` previstos.

### `contact`

> Cambiado en la **V3** (decisión L-2). La dirección ya no vive aquí.

```
id, company_id, first_name, last_name,
source, status, created_at, updated_at, deleted_at
```

Un contacto es **una persona**, no una dirección. Por dónde se le escribe está
en `contact_channel`.

### `contact_channel`

> Nueva en la **V3**.

```
id, company_id, contact_id, channel, value_raw, value_normalized,
status, created_at, updated_at, deleted_at
```

```sql
CREATE UNIQUE INDEX idx_channel_unique
  ON contact_channel (company_id, channel, value_normalized)
  WHERE deleted_at IS NULL;
```

`channel`: `email` · `whatsapp` — las mismas cadenas que produce
`arles_core::Canal::como_texto`, y hay un test que lo vigila en los dos lados.

`value_raw` preserva lo que el usuario escribió, para poder mostrarlo tal cual.
`value_normalized` es la clave de la deduplicación, de la supresión y de la
unicidad de los intentos.

!! **El «1» mexicano.** Un móvil se guarda en E.164 **sin** el `1` que WhatsApp
arrastra de antes de 2019: `+52` y diez dígitos. Con las dos formas conviviendo,
la misma persona entra dos veces y las tres comparaciones de arriba fallan a la
vez. Lo impone `arles_core::PhoneNumber`, no cada consulta. Si un aviso de Meta
llega con el `1`, hay que pasarlo por ahí antes de buscar a quién corresponde.

`company_id` se repite aunque ya esté en `contact`: el índice único es por
empresa y un índice no alcanza la columna de otra tabla.

### `consent_entry`

> Nueva en la **V3** (decisión L-3).

```
id, company_id, channel, address_normalized, kind, basis,
evidence, evidence_ref, recorded_at, recorded_by
```

`kind`: `granted` · `withdrawn`
`basis`: `public_source` · `existing_relationship` · `form_optin` ·
`import_affirmation` · `verbal`

**Append-only**, con disparadores, como `audit_log`. Retirar el consentimiento
es una entrada **nueva**: si se pudiera editar la anterior, cualquiera podría
reescribir a posteriori con qué base se le escribió a alguien. El estado actual
es la última entrada por (empresa, canal, dirección).

!! **No lleva `contact_id`, y no es un olvido.** El primer borrador llevaba uno
con `ON DELETE SET NULL`. Un test lo tiró: `SET NULL` es un `UPDATE`, el
disparador de append-only lo aborta, y el resultado era que **un contacto con
consentimiento registrado ya no se podía borrar** — registrar la prueba de que
se le podía escribir impedía ejercer su derecho de cancelación (§3.4). La
entrada es sobre una dirección, no sobre un registro.

### `contact_field`
Campos personalizados (§33).

```
id, contact_id, field_key, field_value, value_type
```

Con índice sobre `(contact_id, field_key)`. Modelo clave-valor en vez de columnas dinámicas: con 500 k contactos y pocos campos por contacto la diferencia de rendimiento es despreciable, y evita migraciones de esquema cada vez que una empresa añade un campo.

### `contact_list`, `contact_list_member`, `tag`, `contact_tag`
Agrupaciones. `contact_list_member` y `contact_tag` son tablas de unión con clave primaria compuesta.

### `import_batch`
Registro de cada archivo importado (§35).

```
id, company_id, original_filename, stored_filename, file_hash,
column_mapping, total_rows, imported, duplicates, invalid, suppressed,
consent_affirmation, status, created_at, completed_at
```

**`original_filename` es sólo metadato: nunca se usa para escribir en disco.** `stored_filename` es un UUID generado por nosotros. Ver el vector «nombre de archivo malicioso» en el threat model.

`consent_affirmation` registra la afirmación de origen lícito del usuario (riesgo R-13).

### `suppression_entry`
La tabla con más autoridad del sistema (§39).

> Cambiada en la **V3**: distingue canales (decisión L-4).

```
id, company_id, channel, address_normalized, scope, request_id,
reason, origin, notes, created_at, created_by
```

```sql
CREATE UNIQUE INDEX idx_suppression_unique
  ON suppression_entry (company_id, channel, address_normalized);
```

`reason`: `hard_bounce` · `unsubscribe` · `manual` · `complaint` ·
`invalid_number` · `channel_failure`
`origin`: `smtp_5xx` · `user` · `import` · `whatsapp_webhook`
`scope`: `channel` · `global`

**«No me escribas por WhatsApp» no es «no me escribas nunca».** Sin el canal,
las dos frases se guardaban igual.

### El alcance global, y lo que el esquema no puede garantizar

Una baja global se guarda como **una fila por dirección**, todas con el mismo
`request_id`. No como una fila atada a la persona: ésa desaparecería al
ejercerse el derecho de cancelación, que es justo cuando más falta hace.

!i **Consecuencia que hay que conocer:** si a esa persona se le añade *después*
un canal nuevo, la fila de ese canal **la tiene que insertar el código** al
añadirlo. El esquema no puede hacerlo solo. Es una regla del repositorio de
contactos, no una garantía de la base, y se dice aquí en vez de aparentar que
está cubierta.

**Dos propiedades que definen su comportamiento:**

1. **Sobrevive a las importaciones.** Una importación **nunca** borra ni sobrescribe una entrada. Si un contacto suprimido reaparece en un archivo, se importa el contacto y la supresión permanece.
2. **Se consulta en el momento del envío**, no sólo al construir la audiencia.

Nótese que la clave es la **dirección normalizada**, **no `contact_id`**: si alguien borra un contacto y lo vuelve a importar, la supresión sigue aplicando. La supresión es sobre la dirección, no sobre el registro.

### `template` y `template_version`
Versionado de plantillas (§44).

```
template         id, company_id, name, current_version_id, created_at, updated_at
template_version id, template_id, version_number, subject, body_html,
                 body_text, variables_used, created_at
```

`template_version` es **inmutable**. Una campaña apunta a una versión concreta: editar la plantilla después no altera lo que ya se envió, ni lo que está en cola.

### `signature`

```
id, company_id, name, body_html, body_text, is_default
```

### `campaign`

```
id, company_id, name, status, email_account_id, template_version_id,
signature_id, execution_window_id, daily_limit, hourly_limit,
scheduled_start, activated_at, completed_at,
over_50_warning_accepted_at, created_at, updated_at
```

`status`: `draft` · `ready` · `scheduled` · `running` · `paused` · `stopped` · `completed`

`over_50_warning_accepted_at` registra la aceptación explícita del aviso del §48. También se duplica en `audit_log`: aquí para consultarlo rápido, allí para que sea imborrable.

### `campaign_audience`
La audiencia **congelada** al activar.

```
campaign_id, contact_id, added_at
```

Congelarla es lo que hace la campaña reproducible y auditable. Sin esto, «¿a quién le llegó esto?» no tendría respuesta.

### `message_attempt` — la tabla crítica

> Cambiada en la **V3**: lleva canal, y `contact_email` pasó a llamarse
> `contact_address`. Una columna llamada «email» que guarda un teléfono es una
> trampa para quien lea la consulta dentro de un año.

```
id, campaign_id, channel, contact_id, contact_address, email_account_id,
idempotency_key, state, attempt_count,
scheduled_for, claimed_at, sent_at,
provider_message_id, provider_response,
error_kind, error_detail, next_retry_at,
created_at, updated_at
```

```sql
CREATE UNIQUE INDEX idx_attempt_unique
  ON message_attempt (campaign_id, channel, contact_address);

CREATE INDEX idx_attempt_work
  ON message_attempt (state, scheduled_for)
  WHERE state IN ('queued', 'failed');

CREATE INDEX idx_attempt_campaign_state
  ON message_attempt (campaign_id, state);
```

**La restricción única es lo que hace que el §55 (idempotencia) sea una propiedad del esquema y no una esperanza del código.** Ningún fallo de lógica, ninguna condición de carrera, ningún doble clic puede producir dos intentos para la misma campaña, la misma dirección y el mismo canal: la base de datos lo rechaza.

!! **Qué compró exactamente el canal en esta clave, y qué no.** No compró que
una campaña pueda escribirle a alguien por correo y después por WhatsApp: eso ya
funcionaba, porque la clave es la **dirección** y un correo y un móvil son
direcciones distintas. Lo que impedía el doble canal antes de la V3 era que **un
contacto no tenía dónde guardar un móvil**. El canal en la clave hace que ésta
*diga* lo que significa y sostiene la garantía si dos canales llegaran a
compartir la misma cadena; sin él, «un envío por canal» sería una coincidencia
de que las direcciones tienen formas distintas.

Esto se descubrió rompiendo el índice a propósito: la prueba que supuestamente
lo vigilaba siguió pasando. Hay dos tests ahora, y el contraste entre ellos es
el punto.

`idempotency_key` se genera y persiste **antes** de llamar al proveedor, y viaja como `Message-Id` del correo.

Estados y su significado: ver `MOTOR_DE_EJECUCION.md` y el glosario.

### `execution_window`
Días y horas operativas (§47).

```
id, company_id, name, days_of_week, start_hour, end_hour, timezone
```

### `rate_budget`
Estado persistido del cubo de tokens.

```
email_account_id, window_kind, window_start, consumed, updated_at
```

`window_kind`: `hourly` · `daily`. Persistirlo es lo que permite que los límites sobrevivan a cierres y reinicios.

### `audit_log`
Append-only (§91).

```
id, company_id, actor, action, entity_kind, entity_id,
details, created_at
```

**Sin `UPDATE` ni `DELETE`.** Acciones registradas: activar/pausar/detener campaña, conectar/desconectar cuenta, importar, suprimir, restaurar respaldo, aceptar el aviso de más de 50 diarios, exportar datos.

### `schema_migration`
Gestionada por `refinery`.

---

## 3. Decisiones que importan

### 3.1 Normalización de correo

```
email_normalized = trim(email).to_lowercase()
```

**Y nada más.** En particular, **no** se eliminan los puntos ni se recorta el sufijo `+`.

Eso es tentador porque `juan.perez@gmail.com` y `juanperez@gmail.com` son la misma persona en Gmail. Pero es **específico de Gmail**: en la mayoría de dominios corporativos son buzones distintos de personas distintas. Aplicar la normalización de Gmail a todo el mundo fusionaría contactos que no son el mismo, y eso es un error silencioso y difícil de deshacer.

Preferimos un duplicado ocasional a una fusión incorrecta. El duplicado el usuario lo ve y lo arregla; la fusión no la ve nadie.

Sobre el dominio en minúsculas: es correcto siempre, porque el dominio es insensible a mayúsculas por RFC. La parte local técnicamente **sí** puede ser sensible a mayúsculas, pero ningún proveedor real lo explota, y tratarla como sensible generaría duplicados constantes a partir de errores de captura. Aquí sí ganamos más de lo que perdemos.

### 3.2 La supresión gana siempre

Comprobación en tres puntos, de menos a más autoritativo:

1. **Al construir la audiencia** — para que el conteo previsto sea honesto.
2. **Al encolar** — para no llenar la cola de trabajo que se va a descartar.
3. **Inmediatamente antes de entregar al proveedor** — **esta es la que cuenta.**

Sólo la tercera es autoritativa. Las dos primeras son optimizaciones y cortesía informativa.

### 3.3 Escala: 500 000 contactos

SQLite aguanta esto sin dificultad. Lo que hay que hacer bien:

**Paginación por keyset, nunca `OFFSET`.**

```sql
-- Mal: O(n) en el desplazamiento
SELECT … ORDER BY created_at LIMIT 50 OFFSET 400000;

-- Bien: O(log n) siempre
SELECT … WHERE (created_at, id) > (?, ?) ORDER BY created_at, id LIMIT 50;
```

Con 500 k filas, la diferencia entre ambas al final de la tabla es de milisegundos a segundos.

**Importación por lotes en transacción.** Una transacción por cada ~1 000 filas. Una sola transacción gigante consume memoria sin límite; una transacción por fila hace 500 000 fsync.

**Índices sólo sobre lo que de verdad se filtra.** Cada índice cuesta en escritura, y la importación es la operación de escritura más pesada del producto.

**Sobrecoste de SQLCipher:** del orden de 5–15 %. Asumible y no negociable (T-3).

### 3.4 Derechos ARCO y borrado

La LFPDPPP (T-8) da derecho a cancelación. Eso significa **borrado físico real** de los datos del titular, no un `deleted_at`.

La tensión: el §91 exige bitácora de auditoría, y la auditoría exige no borrar.

**Resolución:** al ejercerse el derecho de cancelación se borran físicamente `contact`, `contact_field`, sus pertenencias a listas y etiquetas, y se **anonimizan** las referencias en `message_attempt` (se conserva el intento y su resultado, se elimina la identidad). La entrada de `audit_log` registra que hubo una cancelación, con el correo **hasheado**, no en claro.

Así la auditoría conserva el hecho —«el 12 de marzo se canceló un titular»— sin conservar al titular. Y `suppression_entry` permanece, porque suprimir es precisamente lo que protege al titular de futuros envíos.

### 3.5 Migraciones

`refinery`, con migraciones SQL versionadas y embebidas en el binario.

- Nunca se edita una migración ya publicada.
- Toda migración se prueba con una base de datos poblada, no vacía.
- Antes de migrar, la aplicación hace una copia de la base de datos. Una migración fallida sobre datos de un cliente sin copia previa es una pérdida irrecuperable.

---

## 4. Diagrama de relaciones

```
company ─┬─ email_account ──────────┐
         ├─ contact ─┬─ contact_field│
         │           ├─ contact_list_member ── contact_list
         │           └─ contact_tag ── tag
         ├─ import_batch
         ├─ suppression_entry        (clave: email_normalized)
         ├─ template ── template_version ─┐
         ├─ signature                     │
         ├─ execution_window ─────────┐   │
         ├─ audit_log                 │   │
         └─ campaign ─────────────────┴───┘
                │
                ├─ campaign_audience ── contact
                └─ message_attempt ──── contact
                                    └── email_account ── rate_budget
```
