-- ARLES RELAY I · v1.2.0 · esquema inicial
-- Ver documentacion/03-arquitectura/MODELO_DE_DATOS.md
--
-- Convenciones:
--   · ids TEXT con UUID v7 (ordenables por tiempo, índices compactos)
--   · marcas de tiempo TEXT ISO-8601 UTC; la zona de la empresa va aparte
--   · booleanos INTEGER 0/1
--   · toda entidad de negocio lleva company_id, aunque hoy haya una sola:
--     es lo que no cierra la puerta a ARLES RELAY CLOUD (§8)

-- ─── Empresa ────────────────────────────────────────────────────────────────

CREATE TABLE company (
    id                   TEXT PRIMARY KEY NOT NULL,
    commercial_name      TEXT NOT NULL,
    logo_path            TEXT,
    website              TEXT,
    country              TEXT NOT NULL,
    -- Identificador IANA (America/Mexico_City). TODA la lógica de ventanas de
    -- ejecución usa este campo, nunca la zona del sistema operativo: un portátil
    -- que viaja no debe cambiar la política de envío de la empresa.
    timezone             TEXT NOT NULL,
    corporate_email      TEXT NOT NULL,
    default_signature_id TEXT,
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL
) STRICT;

-- ─── Cuentas remitentes ─────────────────────────────────────────────────────

CREATE TABLE email_account (
    id                    TEXT PRIMARY KEY NOT NULL,
    company_id            TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    display_name          TEXT NOT NULL,
    email_address         TEXT NOT NULL,
    provider_kind         TEXT NOT NULL CHECK (provider_kind IN ('smtp', 'google', 'microsoft')),
    -- Referencia OPACA al llavero del sistema operativo. NUNCA una credencial.
    -- Ver documentacion/04-seguridad/MODELO_DE_SECRETOS.md y ADR-0011.
    credential_ref        TEXT NOT NULL,
    status                TEXT NOT NULL DEFAULT 'connected'
                          CHECK (status IN ('connected', 'needs_reauth', 'circuit_open', 'disabled')),
    daily_limit           INTEGER NOT NULL,
    hourly_limit          INTEGER NOT NULL,
    consecutive_failures  INTEGER NOT NULL DEFAULT 0,
    circuit_open_until    TEXT,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
) STRICT;

CREATE INDEX idx_email_account_company ON email_account(company_id);

-- ─── Contactos ──────────────────────────────────────────────────────────────

CREATE TABLE contact (
    id               TEXT PRIMARY KEY NOT NULL,
    company_id       TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    -- Lo que escribió el usuario, para poder mostrarlo tal cual.
    email_raw        TEXT NOT NULL,
    -- trim + minúsculas, y nada más. NO se eliminan puntos ni sufijo '+':
    -- eso es específico de Gmail y fusionaría contactos distintos en otros
    -- dominios. Ver MODELO_DE_DATOS.md §3.1.
    email_normalized TEXT NOT NULL,
    first_name       TEXT,
    last_name        TEXT,
    source           TEXT,
    status           TEXT NOT NULL DEFAULT 'active'
                     CHECK (status IN ('active', 'archived')),
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT
) STRICT;

-- La deduplicación del §36 como propiedad del esquema, no del código.
CREATE UNIQUE INDEX idx_contact_unique
    ON contact(company_id, email_normalized)
    WHERE deleted_at IS NULL;

CREATE TABLE contact_field (
    contact_id  TEXT NOT NULL REFERENCES contact(id) ON DELETE CASCADE,
    field_key   TEXT NOT NULL,
    field_value TEXT,
    value_type  TEXT NOT NULL DEFAULT 'text'
                CHECK (value_type IN ('text', 'number', 'date', 'boolean')),
    PRIMARY KEY (contact_id, field_key)
) STRICT;

CREATE TABLE contact_list (
    id         TEXT PRIMARY KEY NOT NULL,
    company_id TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE contact_list_member (
    list_id    TEXT NOT NULL REFERENCES contact_list(id) ON DELETE CASCADE,
    contact_id TEXT NOT NULL REFERENCES contact(id) ON DELETE CASCADE,
    added_at   TEXT NOT NULL,
    PRIMARY KEY (list_id, contact_id)
) STRICT;

CREATE INDEX idx_list_member_contact ON contact_list_member(contact_id);

CREATE TABLE tag (
    id         TEXT PRIMARY KEY NOT NULL,
    company_id TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_tag_unique ON tag(company_id, name);

CREATE TABLE contact_tag (
    tag_id     TEXT NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
    contact_id TEXT NOT NULL REFERENCES contact(id) ON DELETE CASCADE,
    PRIMARY KEY (tag_id, contact_id)
) STRICT;

CREATE INDEX idx_contact_tag_contact ON contact_tag(contact_id);

-- ─── Importaciones ──────────────────────────────────────────────────────────

CREATE TABLE import_batch (
    id                  TEXT PRIMARY KEY NOT NULL,
    company_id          TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    -- SOLO metadato. El nombre suministrado NUNCA se usa para escribir en disco:
    -- stored_filename es un UUID nuestro. Ver THREAT_MODEL.md §4.1.
    original_filename   TEXT NOT NULL,
    stored_filename     TEXT NOT NULL,
    file_hash           TEXT NOT NULL,
    column_mapping      TEXT NOT NULL,
    total_rows          INTEGER NOT NULL DEFAULT 0,
    imported            INTEGER NOT NULL DEFAULT 0,
    duplicates          INTEGER NOT NULL DEFAULT 0,
    invalid             INTEGER NOT NULL DEFAULT 0,
    suppressed          INTEGER NOT NULL DEFAULT 0,
    -- Afirmación de origen lícito del usuario (LFPDPPP, riesgo R-13).
    consent_affirmation TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    created_at          TEXT NOT NULL,
    completed_at        TEXT
) STRICT;

CREATE INDEX idx_import_company ON import_batch(company_id, created_at);

-- ─── Supresión ──────────────────────────────────────────────────────────────

-- La tabla con más autoridad del sistema (§39).
--
-- La clave es email_normalized, NO contact_id: si alguien borra un contacto y
-- lo vuelve a importar, la supresión sigue aplicando. La supresión es sobre la
-- dirección, no sobre el registro.
--
-- Una importación NUNCA elimina ni sobrescribe una entrada de esta tabla.
CREATE TABLE suppression_entry (
    id               TEXT PRIMARY KEY NOT NULL,
    company_id       TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    email_normalized TEXT NOT NULL,
    reason           TEXT NOT NULL
                     CHECK (reason IN ('hard_bounce', 'unsubscribe', 'manual', 'complaint')),
    origin           TEXT NOT NULL
                     CHECK (origin IN ('smtp_5xx', 'user', 'import')),
    notes            TEXT,
    created_at       TEXT NOT NULL,
    created_by       TEXT
) STRICT;

CREATE UNIQUE INDEX idx_suppression_unique
    ON suppression_entry(company_id, email_normalized);

-- ─── Plantillas y firmas ────────────────────────────────────────────────────

CREATE TABLE template (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    name               TEXT NOT NULL,
    current_version_id TEXT,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
) STRICT;

-- Inmutable (§44). Una campaña apunta a una VERSIÓN concreta: editar la
-- plantilla después no altera lo que ya se envió ni lo que está en cola.
CREATE TABLE template_version (
    id             TEXT PRIMARY KEY NOT NULL,
    template_id    TEXT NOT NULL REFERENCES template(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    subject        TEXT NOT NULL,
    body_html      TEXT NOT NULL,
    body_text      TEXT NOT NULL,
    variables_used TEXT NOT NULL,
    created_at     TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_template_version_unique
    ON template_version(template_id, version_number);

CREATE TABLE signature (
    id         TEXT PRIMARY KEY NOT NULL,
    company_id TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    body_html  TEXT NOT NULL,
    body_text  TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

-- ─── Ejecución ──────────────────────────────────────────────────────────────

CREATE TABLE execution_window (
    id           TEXT PRIMARY KEY NOT NULL,
    company_id   TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    -- Máscara de bits: bit 0 = lunes … bit 6 = domingo.
    days_of_week INTEGER NOT NULL,
    start_hour   INTEGER NOT NULL CHECK (start_hour BETWEEN 0 AND 23),
    end_hour     INTEGER NOT NULL CHECK (end_hour BETWEEN 1 AND 24),
    timezone     TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    CHECK (end_hour > start_hour)
) STRICT;

CREATE TABLE campaign (
    id                          TEXT PRIMARY KEY NOT NULL,
    company_id                  TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    name                        TEXT NOT NULL,
    status                      TEXT NOT NULL DEFAULT 'draft'
                                CHECK (status IN ('draft', 'ready', 'scheduled', 'running',
                                                  'paused', 'stopped', 'completed')),
    email_account_id            TEXT REFERENCES email_account(id) ON DELETE RESTRICT,
    template_version_id         TEXT REFERENCES template_version(id) ON DELETE RESTRICT,
    signature_id                TEXT REFERENCES signature(id) ON DELETE SET NULL,
    execution_window_id         TEXT REFERENCES execution_window(id) ON DELETE RESTRICT,
    daily_limit                 INTEGER,
    hourly_limit                INTEGER,
    scheduled_start             TEXT,
    activated_at                TEXT,
    completed_at                TEXT,
    -- Aceptación explícita del aviso de más de 50 diarios (§48). Se duplica en
    -- audit_log: aquí para consultarlo rápido, allí para que sea imborrable.
    over_50_warning_accepted_at TEXT,
    created_at                  TEXT NOT NULL,
    updated_at                  TEXT NOT NULL
) STRICT;

CREATE INDEX idx_campaign_company_status ON campaign(company_id, status);

-- Audiencia CONGELADA al activar. Es lo que hace la campaña reproducible y
-- auditable: sin esto, «¿a quién le llegó esto?» no tendría respuesta.
--
-- Congelada significa que sobrevive a lo que pase después con el contacto. Por
-- eso la clave es la dirección y el contact_id se anula al borrar, en vez de
-- arrastrar la fila: si la instantánea encogiera sola, no sería una instantánea.
CREATE TABLE campaign_audience (
    campaign_id   TEXT NOT NULL REFERENCES campaign(id) ON DELETE CASCADE,
    contact_email TEXT NOT NULL,
    contact_id    TEXT REFERENCES contact(id) ON DELETE SET NULL,
    added_at      TEXT NOT NULL,
    PRIMARY KEY (campaign_id, contact_email)
) STRICT;

CREATE INDEX idx_audiencia_contacto ON campaign_audience(contact_id);

-- ─── La tabla crítica ───────────────────────────────────────────────────────

-- Un intento por contacto y campaña. Es un OUTBOX, no un log.
--
-- La restricción única de abajo es lo que convierte el §55 (idempotencia) en
-- una propiedad del esquema y no en una esperanza depositada en el código:
-- ningún fallo de lógica, ninguna condición de carrera y ningún doble clic
-- pueden crear dos intentos para el mismo par.
CREATE TABLE message_attempt (
    id                  TEXT PRIMARY KEY NOT NULL,
    campaign_id         TEXT NOT NULL REFERENCES campaign(id) ON DELETE CASCADE,
    -- SET NULL, no CASCADE: borrar un contacto NO puede borrar la prueba de que
    -- se le envió un correo. Se conserva el intento y su resultado, se elimina
    -- la identidad — que es exactamente lo que MODELO_DE_DATOS.md §3.4 describe
    -- para el derecho de cancelación.
    contact_id          TEXT REFERENCES contact(id) ON DELETE SET NULL,
    -- La clave REAL de idempotencia. Por el mismo motivo que la supresión se
    -- indexa por dirección y no por contact_id (§39): si alguien borra un
    -- contacto y lo vuelve a importar, obtiene un id nuevo — y con una
    -- restricción basada en contact_id, la campaña le enviaría OTRA VEZ.
    contact_email       TEXT NOT NULL,
    email_account_id    TEXT REFERENCES email_account(id) ON DELETE SET NULL,
    -- UUID generado y persistido ANTES de llamar al proveedor. Viaja como
    -- Message-Id del correo. Ver ADR-0004.
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

-- Un intento por campaña y DIRECCIÓN, no por campaña y contact_id.
-- Es lo que hace que borrar y reimportar un contacto no abra la puerta a un
-- segundo envío (§55, ADR-0004).
CREATE UNIQUE INDEX idx_attempt_unique
    ON message_attempt(campaign_id, contact_email);

CREATE UNIQUE INDEX idx_attempt_idempotency
    ON message_attempt(idempotency_key);

CREATE INDEX idx_attempt_contacto ON message_attempt(contact_id);

-- Índice PARCIAL: solo cubre filas accionables. En una campaña de 500 000 con
-- 499 000 completadas, este índice tiene 1 000 entradas, no 500 000.
CREATE INDEX idx_attempt_work
    ON message_attempt(state, scheduled_for)
    WHERE state IN ('queued', 'failed');

CREATE INDEX idx_attempt_campaign_state
    ON message_attempt(campaign_id, state);

-- Estado persistido del cubo de tokens. Persistirlo es lo que permite que los
-- límites sobrevivan a cierres y reinicios: un cubo en memoria se reiniciaría
-- con la aplicación y dejaría enviar el límite diario varias veces al día.
CREATE TABLE rate_budget (
    email_account_id TEXT NOT NULL REFERENCES email_account(id) ON DELETE CASCADE,
    window_kind      TEXT NOT NULL CHECK (window_kind IN ('hourly', 'daily')),
    window_start     TEXT NOT NULL,
    consumed         INTEGER NOT NULL DEFAULT 0,
    updated_at       TEXT NOT NULL,
    PRIMARY KEY (email_account_id, window_kind)
) STRICT;

-- ─── Auditoría ──────────────────────────────────────────────────────────────

-- Append-only (§91). Sin UPDATE ni DELETE: los triggers de abajo lo imponen a
-- nivel de motor, no de convención.
CREATE TABLE audit_log (
    id          TEXT PRIMARY KEY NOT NULL,
    company_id  TEXT NOT NULL,
    actor       TEXT NOT NULL,
    action      TEXT NOT NULL,
    entity_kind TEXT,
    entity_id   TEXT,
    details     TEXT,
    created_at  TEXT NOT NULL
) STRICT;

CREATE INDEX idx_audit_company_fecha ON audit_log(company_id, created_at);

CREATE TRIGGER audit_log_sin_update
BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit_log es append-only: no admite UPDATE');
END;

CREATE TRIGGER audit_log_sin_delete
BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit_log es append-only: no admite DELETE');
END;
