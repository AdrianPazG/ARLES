-- ARLES RELAY I · v1.2.0 · un contacto tiene canales
--
-- Decisión L-2 / D-6, autorizada por Dirección el 16 de septiembre de 2026.
-- Ver documentacion/01-producto/LOGISTICA_DE_CAMPANAS.md y MODELO_DE_DATOS.md.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- QUÉ CAMBIA, Y POR QUÉ NO PUEDE ESPERAR A LA FASE 4
--
-- Hasta aquí **un contacto era un correo**: la dirección vivía como dos
-- columnas de `contact`, la unicidad de los envíos era (campaña, dirección) y
-- la supresión era (empresa, dirección). Las tres daban por hecho que sólo
-- existe un canal.
--
-- Con WhatsApp dentro de v1.2.0 (P-15), eso rompe tres cosas:
--
--   1. **Un contacto no tiene dónde guardar un móvil.** La dirección son dos
--      columnas de `contact` y sólo cabe una. Éste es el bloqueo de verdad, y
--      conviene decirlo bien: la primera redacción de esta nota afirmaba que
--      el segundo envío chocaría contra `idx_attempt_unique`, y **no es
--      cierto** — esa clave es (campaña, dirección), y el correo y el móvil
--      son direcciones distintas, así que nunca habrían chocado. El error se
--      destapó al romper el índice a propósito y ver que la prueba que
--      supuestamente lo vigilaba seguía pasando.
--   2. «No me escribas por WhatsApp» se guardaría como «no me escribas nunca»,
--      porque la supresión no sabe distinguir canales (L-4).
--   3. El consentimiento sería una casilla sin prueba ni fecha, y eso no
--      sostiene una revisión (L-3, LFPDPPP).
--
-- El canal entra igualmente en la clave de unicidad de los intentos, pero por
-- otra razón, más modesta y que conviene no inflar: hace que la clave **diga**
-- lo que significa, y sostiene la garantía si alguna vez dos canales llegaran
-- a compartir la misma cadena. Sin él, «un envío por canal» sería una
-- coincidencia de que las direcciones tienen formas distintas.
--
-- Se hace **antes** del motor de ejecución porque el motor se escribe contra
-- este esquema: cambiar la clave de unicidad con el motor ya escrito toca la
-- cola, los estados y la idempotencia enteros. Sería una reescritura, no una
-- migración (riesgo R-23).
--
-- ── Lo que esta migración NO hace ──
--
-- **No parte las campañas en etapas por canal** (decisión L-1). Eso es el paso
-- siguiente y toca `campaign`, no los contactos. Aquí se prepara el terreno:
-- audiencia e intentos ya saben por qué canal van.
-- ═══════════════════════════════════════════════════════════════════════════

-- ─── 1 · Los canales de un contacto ─────────────────────────────────────────

-- Un contacto puede tener un correo, un móvil, o los dos. También dos correos,
-- que es lo normal en una empresa (el genérico y el de la persona).
--
-- `company_id` se repite aquí aunque ya esté en `contact`. No es un descuido:
-- el índice único de abajo es por empresa, y un índice no puede alcanzar la
-- columna de otra tabla. La alternativa —comprobarlo en el código— convierte
-- una propiedad del esquema en una esperanza depositada en cada consulta.
CREATE TABLE contact_channel (
    id               TEXT PRIMARY KEY NOT NULL,
    company_id       TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    contact_id       TEXT NOT NULL REFERENCES contact(id) ON DELETE CASCADE,
    channel          TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    -- Lo que escribió el usuario, para poder mostrarlo tal cual.
    value_raw        TEXT NOT NULL,
    -- La clave de TODO lo que compara: deduplicación, supresión y unicidad de
    -- los intentos. La produce `arles_core::ValorDeCanal`, que es quien sabe
    -- que un correo se normaliza a minúsculas y un móvil mexicano se guarda
    -- SIN el «1» que WhatsApp arrastra de antes de 2019. Con las dos formas
    -- conviviendo, la misma persona entra dos veces y las tres comparaciones
    -- de arriba fallan a la vez.
    value_normalized TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'active'
                     CHECK (status IN ('active', 'archived')),
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT
) STRICT;

-- La deduplicación del §36, ahora por canal. Dos contactos no pueden compartir
-- la misma dirección en el mismo canal; el mismo teléfono y el mismo correo
-- **sí** pueden convivir, porque no son el mismo valor ni el mismo canal.
CREATE UNIQUE INDEX idx_channel_unique
    ON contact_channel(company_id, channel, value_normalized)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_channel_contact ON contact_channel(contact_id, channel);

-- ── Traslado de lo que ya hay ──
--
-- Cada contacto existente se convierte en un contacto con un canal de correo.
-- Nadie pierde nada y nadie se duplica: la dirección viaja tal cual, en sus dos
-- formas, porque ya estaba normalizada con la misma regla.
--
-- El id del canal se deriva del id del contacto. Es determinista a propósito:
-- volver a correr esto sobre la misma base daría los mismos ids, y una
-- migración que produce ids nuevos cada vez no se puede comparar con nada.
INSERT INTO contact_channel (
    id, company_id, contact_id, channel, value_raw, value_normalized,
    status, created_at, updated_at, deleted_at
)
SELECT
    'ch-email-' || id,
    company_id,
    id,
    'email',
    email_raw,
    email_normalized,
    CASE status WHEN 'archived' THEN 'archived' ELSE 'active' END,
    created_at,
    updated_at,
    deleted_at
FROM contact;

-- Y el correo deja de vivir en `contact`. Dejarlo ahí además de en
-- `contact_channel` sería tener el mismo dato en dos sitios con dos reglas de
-- actualización, que es la forma habitual de que acaben diciendo cosas
-- distintas. El índice se borra primero porque usa las columnas.
DROP INDEX idx_contact_unique;
ALTER TABLE contact DROP COLUMN email_raw;
ALTER TABLE contact DROP COLUMN email_normalized;

-- ─── 2 · El consentimiento es un registro, no una casilla (L-3) ──────────────

-- Una casilla dice «sí» y nada más. No dice quién lo dijo, cuándo, con qué
-- base ni dónde consta — y eso es exactamente lo que hay que poder enseñar si
-- alguien pregunta por qué se le escribió.
--
-- **Append-only**, como `audit_log`. Retirar el consentimiento es una entrada
-- NUEVA, no una edición de la anterior: si se pudiera editar, el registro
-- dejaría de ser prueba de nada. El estado actual es la última entrada por
-- (empresa, canal, dirección).
--
-- Se indexa por DIRECCIÓN y no por `contact_id`, por el mismo motivo que la
-- supresión (§39): si alguien borra un contacto y lo vuelve a importar, obtiene
-- un id nuevo, y una prueba atada al id se habría evaporado.
CREATE TABLE consent_entry (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    channel            TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    address_normalized TEXT NOT NULL,
    -- ── Aquí NO hay contact_id, y es deliberado ──
    --
    -- El primer borrador llevaba uno con `ON DELETE SET NULL`. Un test lo tiró:
    -- `SET NULL` es un UPDATE, el disparador de append-only lo aborta, y el
    -- resultado era que **un contacto con consentimiento registrado ya no se
    -- podía borrar**. O sea, registrar la prueba de que se le podía escribir
    -- impedía ejercer el derecho de cancelación de la LFPDPPP, que es lo
    -- contrario de lo que esta tabla existe para sostener.
    --
    -- Y no hacía falta: la entrada es sobre una DIRECCIÓN, no sobre un
    -- registro. A quién correspondía se averigua por `contact_channel`, y
    -- cuando ese contacto ya no existe, la respuesta correcta es que no
    -- corresponde a nadie.
    kind               TEXT NOT NULL CHECK (kind IN ('granted', 'withdrawn')),
    -- En qué se apoya. No es adorno: decide si el envío es defendible.
    --   public_source          fuente de acceso público (LFPDPPP art. 10 II)
    --   existing_relationship  relación comercial previa
    --   form_optin             la persona lo pidió en un formulario
    --   import_affirmation     lo afirmó quien importó la lista
    --   verbal                 lo dijo de viva voz; lo más débil de los cinco
    basis              TEXT NOT NULL
                       CHECK (basis IN ('public_source', 'existing_relationship',
                                        'form_optin', 'import_affirmation', 'verbal')),
    -- Dónde consta: la dirección de la fuente pública, la nota de quien lo
    -- registró, el texto del formulario. **Nunca** una credencial.
    evidence           TEXT,
    -- A qué registro nuestro apunta la prueba, cuando lo hay: el lote de
    -- importación, normalmente.
    evidence_ref       TEXT,
    recorded_at        TEXT NOT NULL,
    recorded_by        TEXT
) STRICT;

CREATE INDEX idx_consent_direccion
    ON consent_entry(company_id, channel, address_normalized, recorded_at);

CREATE TRIGGER consent_entry_sin_update
BEFORE UPDATE ON consent_entry
BEGIN
    SELECT RAISE(ABORT, 'consent_entry es append-only: retirar el consentimiento es una entrada nueva');
END;

CREATE TRIGGER consent_entry_sin_delete
BEFORE DELETE ON consent_entry
BEGIN
    SELECT RAISE(ABORT, 'consent_entry es append-only: no admite DELETE');
END;

-- ─── 3 · La supresión distingue canales (L-4) ───────────────────────────────

-- Sigue siendo la tabla con más autoridad del sistema (§39) y sigue indexándose
-- por dirección y no por contacto. Lo que gana es el canal.
--
-- ── Sobre el alcance global ──
--
-- «No me contactes por ningún medio» **no se puede guardar como una sola fila
-- atada a la persona**, y conviene decir por qué en vez de aparentar que sí.
-- Una fila atada a `contact_id` desaparece —o deja de identificar a nadie— en
-- cuanto se ejerce el derecho de cancelación y se borra el contacto, que es
-- justo cuando más falta hace. Por eso se guarda **una fila por dirección**,
-- todas con el mismo `request_id` y con `scope = 'global'`.
--
-- Consecuencia que hay que conocer: si a esa persona se le añade DESPUÉS un
-- canal nuevo, la fila de ese canal la tiene que insertar el código al añadirlo.
-- El esquema no puede hacerlo solo. Queda anotado en MODELO_DE_DATOS.md como
-- regla del repositorio de contactos, no como algo que la base garantice.
CREATE TABLE suppression_entry_nueva (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    channel            TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    address_normalized TEXT NOT NULL,
    -- 'channel': pidió no recibir por este medio.
    -- 'global':  pidió no recibir por ninguno; esta fila es una de varias.
    scope              TEXT NOT NULL DEFAULT 'channel'
                       CHECK (scope IN ('channel', 'global')),
    -- Une las filas que salieron de la misma petición. Sin esto, deshacer una
    -- petición global obligaría a adivinar cuáles eran sus filas.
    request_id         TEXT,
    reason             TEXT NOT NULL
                       CHECK (reason IN ('hard_bounce', 'unsubscribe', 'manual',
                                         'complaint', 'invalid_number', 'channel_failure')),
    origin             TEXT NOT NULL
                       CHECK (origin IN ('smtp_5xx', 'user', 'import', 'whatsapp_webhook')),
    notes              TEXT,
    created_at         TEXT NOT NULL,
    created_by         TEXT
) STRICT;

-- Lo que había era todo de correo, porque no había otro canal.
INSERT INTO suppression_entry_nueva (
    id, company_id, channel, address_normalized, scope, request_id,
    reason, origin, notes, created_at, created_by
)
SELECT id, company_id, 'email', email_normalized, 'channel', NULL,
       reason, origin, notes, created_at, created_by
FROM suppression_entry;

DROP TABLE suppression_entry;
ALTER TABLE suppression_entry_nueva RENAME TO suppression_entry;

CREATE UNIQUE INDEX idx_suppression_unique
    ON suppression_entry(company_id, channel, address_normalized);

CREATE INDEX idx_suppression_peticion
    ON suppression_entry(request_id)
    WHERE request_id IS NOT NULL;

-- ─── 4 · La audiencia congelada, por canal ──────────────────────────────────

-- `contact_email` pasa a llamarse `contact_address`. El nombre viejo describía
-- el contenido de la columna cuando sólo había un canal; con dos, una columna
-- llamada «email» que guarda un teléfono es una trampa para quien lea la
-- consulta dentro de un año.
CREATE TABLE campaign_audience_nueva (
    campaign_id     TEXT NOT NULL REFERENCES campaign(id) ON DELETE CASCADE,
    channel         TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    contact_address TEXT NOT NULL,
    contact_id      TEXT REFERENCES contact(id) ON DELETE SET NULL,
    added_at        TEXT NOT NULL,
    PRIMARY KEY (campaign_id, channel, contact_address)
) STRICT;

INSERT INTO campaign_audience_nueva (campaign_id, channel, contact_address, contact_id, added_at)
SELECT campaign_id, 'email', contact_email, contact_id, added_at
FROM campaign_audience;

DROP TABLE campaign_audience;
ALTER TABLE campaign_audience_nueva RENAME TO campaign_audience;

CREATE INDEX idx_audiencia_contacto ON campaign_audience(contact_id);

-- ─── 5 · La tabla crítica: un intento por canal ─────────────────────────────

-- El cambio que motiva toda la migración.
--
-- Antes: UNIQUE(campaign_id, contact_email).
-- Ahora: UNIQUE(campaign_id, channel, contact_address).
--
-- Sigue siendo imposible enviarle dos veces lo mismo a la misma persona por el
-- mismo medio —que es lo que el §55 protege—, y pasa a ser posible enviarle por
-- correo y después por WhatsApp, que es lo que Dirección pidió.
CREATE TABLE message_attempt_nueva (
    id                  TEXT PRIMARY KEY NOT NULL,
    campaign_id         TEXT NOT NULL REFERENCES campaign(id) ON DELETE CASCADE,
    channel             TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    contact_id          TEXT REFERENCES contact(id) ON DELETE SET NULL,
    contact_address     TEXT NOT NULL,
    email_account_id    TEXT REFERENCES email_account(id) ON DELETE SET NULL,
    idempotency_key     TEXT NOT NULL,
    state               TEXT NOT NULL DEFAULT 'queued'
                        CHECK (state IN ('queued', 'claimed', 'sending', 'sent', 'failed',
                                         'permanently_failed', 'suppressed', 'cancelled',
                                         'presumed_sent')),
    attempt_count       INTEGER NOT NULL DEFAULT 0,
    scheduled_for       TEXT,
    claimed_at          TEXT,
    sent_at             TEXT,
    provider_message_id TEXT,
    provider_response   TEXT,
    error_kind          TEXT,
    error_detail        TEXT,
    next_retry_at       TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
) STRICT;

INSERT INTO message_attempt_nueva (
    id, campaign_id, channel, contact_id, contact_address, email_account_id,
    idempotency_key, state, attempt_count, scheduled_for, claimed_at, sent_at,
    provider_message_id, provider_response, error_kind, error_detail,
    next_retry_at, created_at, updated_at
)
SELECT
    id, campaign_id, 'email', contact_id, contact_email, email_account_id,
    idempotency_key, state, attempt_count, scheduled_for, claimed_at, sent_at,
    provider_message_id, provider_response, error_kind, error_detail,
    next_retry_at, created_at, updated_at
FROM message_attempt;

DROP TABLE message_attempt;
ALTER TABLE message_attempt_nueva RENAME TO message_attempt;

CREATE UNIQUE INDEX idx_attempt_unique
    ON message_attempt(campaign_id, channel, contact_address);

CREATE UNIQUE INDEX idx_attempt_idempotency
    ON message_attempt(idempotency_key);

CREATE INDEX idx_attempt_contacto ON message_attempt(contact_id);

-- Índice PARCIAL: sólo cubre filas accionables, igual que antes.
CREATE INDEX idx_attempt_work
    ON message_attempt(state, scheduled_for)
    WHERE state IN ('queued', 'failed');

CREATE INDEX idx_attempt_campaign_state
    ON message_attempt(campaign_id, state);
