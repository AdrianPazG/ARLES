-- ARLES RELAY I · v1.2.0 · una campaña tiene etapas
--
-- Decisión L-1, corregida el 16/09/2026 a petición de Dirección, e implementada
-- el 17/09/2026. Con la decisión L-11 (Coexistencia) de la misma semana.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- QUÉ CAMBIA
--
-- Hasta aquí una campaña tenía **una** cuenta remitente, **una** plantilla,
-- **una** ventana y **un** ritmo. Eso alcanza para un solo canal.
--
-- Lo que Dirección pidió es una secuencia sobre la misma tabla de contactos:
-- sale el correo, y quien cumpla la condición recibe después el WhatsApp. Eso
-- no es una campaña con dos remitentes: son **dos etapas**, cada una con su
-- canal, su remitente, su plantilla, su ritmo, su ventana y —lo que más
-- importa— **su propio estado**.
--
-- El estado por etapa es lo que hace posible la decisión L-6: apagar WhatsApp
-- sin apagar el correo. Con un solo `campaign.status`, «detener» sólo puede
-- significar detenerlo todo, y el día que el número se ponga en rojo habría que
-- elegir entre seguir arriesgando el número o parar también los correos, que no
-- tienen nada que ver.
--
-- ── Lo que esta migración NO hace ──
--
-- **No implementa el motor.** No hay nada que ejecute etapas todavía: esto es
-- el sitio donde se guardan. El motor es la Fase 4, y se escribe contra este
-- esquema — que es justo por lo que el esquema va antes (riesgo R-23).
-- ═══════════════════════════════════════════════════════════════════════════

-- ─── 1 · El número de WhatsApp como remitente ───────────────────────────────

-- Tabla aparte de `email_account`, y no una columna «tipo» dentro de ella.
--
-- Las dos cosas se parecen en el papel y no se parecen en nada real: una guarda
-- una referencia al llavero con credenciales SMTP, la otra guarda
-- identificadores de Meta y un token que caduca de otra forma; una tiene
-- límites que fija el usuario, la otra límites que fija Meta y **cambia sola**.
-- Una tabla con la mitad de las columnas nulas según el tipo es una tabla donde
-- el `CHECK` de coherencia se olvida un día.
CREATE TABLE whatsapp_account (
    id                    TEXT PRIMARY KEY NOT NULL,
    company_id            TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    display_name          TEXT NOT NULL,
    -- El número en forma canónica E.164, producida por `arles_core::PhoneNumber`
    -- — sin el «1» mexicano. Ver V3.
    phone_e164            TEXT NOT NULL,
    -- Identificadores de Meta. No son secretos: son direcciones.
    waba_id               TEXT NOT NULL,
    phone_number_id       TEXT NOT NULL,
    -- Referencia OPACA al llavero del sistema operativo. NUNCA una credencial.
    credential_ref        TEXT NOT NULL,
    -- 'seguimiento' sólo escribe a quien dio permiso o escribió primero, y no
    -- tiene riesgo. 'prospeccion_directa' es el frío (L-9). El modo vive en la
    -- cuenta y no en la campaña a propósito: es una propiedad del número y de
    -- lo que el cliente aceptó, no de lo que se le ocurra a cada campaña.
    modo                  TEXT NOT NULL DEFAULT 'seguimiento'
                          CHECK (modo IN ('seguimiento', 'prospeccion_directa')),

    -- ── L-11 · Coexistencia ──
    --
    -- Marca de tiempo, no un booleano: lo que hace falta poder enseñar es
    -- **cuándo** se confirmó y quién, no que alguien marcó una casilla alguna
    -- vez. NULL significa que no se ha confirmado, y entonces el canal no se
    -- ofrece.
    --
    -- Sin Coexistencia, el número sale de la app de WhatsApp Business: ARLES
    -- envía y **nadie puede responder desde el móvil**. Y, en segundo lugar,
    -- las respuestas no llegan, así que la cifra de «respondidos» sería siempre
    -- cero — justo la métrica que más protege al número, porque Meta premia que
    -- te contesten.
    --
    -- ARLES **no puede comprobarlo**: no ve dentro de Meta. Lo pregunta, lo
    -- registra aquí y en `audit_log`, y lo vuelve a preguntar en el preflight
    -- de cada campaña. Una casilla marcada hace tres meses no es una
    -- comprobación de hoy.
    coexistencia_confirmada_at TEXT,
    coexistencia_confirmada_by TEXT,
    -- Lo mismo para la advertencia de riesgo del modo en frío (paso 6.5).
    riesgo_aceptado_at    TEXT,
    riesgo_aceptado_by    TEXT,

    -- Lo que Meta decide y nosotros sólo leemos.
    calidad               TEXT CHECK (calidad IN ('green', 'yellow', 'red', 'unknown')),
    calidad_actualizada_at TEXT,
    limite_diario_meta    INTEGER,

    status                TEXT NOT NULL DEFAULT 'connected'
                          CHECK (status IN ('connected', 'needs_reauth', 'circuit_open', 'disabled')),
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
) STRICT;

CREATE INDEX idx_whatsapp_account_company ON whatsapp_account(company_id);

-- Un número no se conecta dos veces a la misma empresa.
CREATE UNIQUE INDEX idx_whatsapp_account_unique
    ON whatsapp_account(company_id, phone_e164);

-- Historial de la calificación de Meta, para la gráfica de ACTIVIDAD.
--
-- Meta da **el resultado, no los ingredientes**: no publica cuántos te
-- bloquearon ni cuántos te reportaron, sólo un color calculado con esas señales
-- entre otras. Guardar cada cambio con su fecha es lo que permite responder a
-- la única pregunta que importa —«¿qué envío fue el que me quemó el número?»—,
-- que un contador de bloqueos no contestaría aunque existiera.
CREATE TABLE whatsapp_quality_event (
    id                  TEXT PRIMARY KEY NOT NULL,
    whatsapp_account_id TEXT NOT NULL REFERENCES whatsapp_account(id) ON DELETE CASCADE,
    calidad             TEXT NOT NULL CHECK (calidad IN ('green', 'yellow', 'red', 'unknown')),
    limite_diario_meta  INTEGER,
    -- El aviso tal y como llegó, para diagnóstico. Nunca una credencial.
    evento_crudo        TEXT,
    ocurrido_at         TEXT NOT NULL,
    created_at          TEXT NOT NULL
) STRICT;

CREATE INDEX idx_quality_cuenta_fecha
    ON whatsapp_quality_event(whatsapp_account_id, ocurrido_at);

-- ─── 2 · Las etapas ─────────────────────────────────────────────────────────

CREATE TABLE campaign_stage (
    id                  TEXT PRIMARY KEY NOT NULL,
    campaign_id         TEXT NOT NULL REFERENCES campaign(id) ON DELETE CASCADE,
    -- 1 la primera, 2 la segunda. El tope está aquí y en
    -- `arles_core::MAX_ETAPAS`, y hay un test que lo vigila en los dos lados.
    position            INTEGER NOT NULL CHECK (position IN (1, 2)),
    channel             TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),

    -- El remitente: exactamente uno de los dos, y el que corresponde al canal.
    -- Un `CHECK` y no una regla del código, porque una etapa de correo colgada
    -- de un número de WhatsApp no falla al guardarse: falla al enviar, con la
    -- campaña ya activada.
    email_account_id    TEXT REFERENCES email_account(id) ON DELETE RESTRICT,
    whatsapp_account_id TEXT REFERENCES whatsapp_account(id) ON DELETE RESTRICT,

    template_version_id TEXT REFERENCES template_version(id) ON DELETE RESTRICT,
    signature_id        TEXT REFERENCES signature(id) ON DELETE SET NULL,
    execution_window_id TEXT REFERENCES execution_window(id) ON DELETE RESTRICT,

    -- El ritmo es **de la etapa**. WhatsApp empieza mucho más lento que el
    -- correo —5 a 10 diarios frente a decenas—, así que un límite compartido
    -- obligaría a elegir entre frenar el correo o quemar el número.
    daily_limit         INTEGER,
    hourly_limit        INTEGER,

    -- Cuándo entra esta etapa.
    condition           TEXT NOT NULL DEFAULT 'always'
                        CHECK (condition IN ('always', 'previous_not_failed')),
    -- Horas desde que termina la etapa anterior. Tope de 30 días, el mismo que
    -- `arles_core::MAX_ESPERA_HORAS`.
    wait_hours          INTEGER NOT NULL DEFAULT 0
                        CHECK (wait_hours >= 0 AND wait_hours <= 720),

    -- **Su propio estado.** Es lo que permite apagar un canal sin apagar el
    -- otro (L-6).
    status              TEXT NOT NULL DEFAULT 'draft'
                        CHECK (status IN ('draft', 'ready', 'scheduled', 'running',
                                          'paused', 'stopped', 'completed')),
    activated_at        TEXT,
    completed_at        TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,

    -- El remitente corresponde al canal, y es exactamente uno.
    CHECK (
        (channel = 'email'
            AND email_account_id IS NOT NULL AND whatsapp_account_id IS NULL)
        OR (channel = 'whatsapp'
            AND whatsapp_account_id IS NOT NULL AND email_account_id IS NULL)
    ),
    -- Una firma es cosa del correo. En WhatsApp no existe tal cosa, y una firma
    -- colgada de una etapa de WhatsApp es un campo que alguien rellenaría
    -- esperando que saliera en el mensaje.
    CHECK (channel = 'email' OR signature_id IS NULL),
    -- La primera etapa no espera a nadie. Si esperara, la campaña quedaría
    -- activada sin enviar nada y sin que la pantalla pudiera explicar por qué.
    CHECK (position > 1 OR (wait_hours = 0 AND condition = 'always'))
) STRICT;

-- Las posiciones no se repiten dentro de una campaña.
CREATE UNIQUE INDEX idx_stage_posicion
    ON campaign_stage(campaign_id, position);

-- Y los canales tampoco. Dos etapas del mismo canal no son una secuencia: son
-- dos envíos iguales, y la unicidad de los intentos rechazaría el segundo
-- contacto a contacto — la campaña se activaría y la segunda etapa no enviaría
-- ni un mensaje, sin ningún error a la vista.
CREATE UNIQUE INDEX idx_stage_canal
    ON campaign_stage(campaign_id, channel);

CREATE INDEX idx_stage_trabajo
    ON campaign_stage(status, campaign_id)
    WHERE status IN ('scheduled', 'running');

-- ─── 3 · La campaña se queda con lo que es suyo ─────────────────────────────

-- Lo que decide CÓMO se envía baja a la etapa. Lo que decide QUÉ es la campaña
-- —su nombre, su estado general, su audiencia— se queda aquí.
--
-- `campaign.status` no desaparece: sigue siendo el estado de la campaña como
-- conjunto (borrador, activada, terminada). Lo que ya no puede es decidir si un
-- canal concreto está enviando; eso lo dice la etapa.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ ESTA TABLA **NO** SE RECONSTRUYE, Y LAS OTRAS SÍ
--
-- La primera versión de esta migración hacía lo de siempre: crear
-- `campaign_nueva`, copiar, `DROP TABLE campaign` y renombrar.
--
-- **Eso borraba la audiencia congelada y el registro de envíos.** Con
-- `PRAGMA foreign_keys = ON`, soltar una tabla PADRE ejecuta un borrado
-- implícito de sus filas, y ese borrado **cascadea**: `campaign_audience` y
-- `message_attempt` cuelgan de `campaign` con `ON DELETE CASCADE`. La migración
-- terminaba «bien», sin un solo error, con la instantánea de a quién se le
-- escribió y la prueba de que se le escribió convertidas en cero filas.
--
-- Lo destapó la prueba sobre base poblada: las etapas que se acababan de crear
-- desaparecían con el mismo `DROP`. Sobre una base vacía —que es como se ve
-- correr una migración si uno no se molesta en poblarla— no habría pasado nada.
--
-- => Aquí se quitan las columnas **en su sitio**, con `ALTER TABLE … DROP
-- COLUMN`. La tabla nunca se suelta, así que no hay borrado implícito ni
-- cascada. Las tablas que sí se reconstruyen más abajo —`message_attempt` y
-- `rate_budget`— son HIJAS: nadie cuelga de ellas, y soltarlas no arrastra nada.
-- ═══════════════════════════════════════════════════════════════════════════

-- ── Traslado ──
--
-- Cada campaña que ya tenía remitente se convierte en una campaña con **una
-- etapa de correo**, que es lo único que podía ser: hasta aquí no existía otro
-- canal. La etapa hereda el estado de la campaña, porque hasta ahora ése era el
-- único estado que había y no hay forma de saber otra cosa.
--
-- Las campañas en borrador **sin remitente** no producen etapa: una etapa sin
-- remitente no pasa el `CHECK`, y dejarla a medias sería peor que no tenerla.
-- Se recuperan en la pantalla de campañas eligiendo cuenta, que es lo que su
-- autor tenía pendiente de todos modos.
--
-- El id de la etapa se deriva del de la campaña: determinista, para que volver
-- a correr esto dé lo mismo y se pueda comparar.
INSERT INTO campaign_stage (
    id, campaign_id, position, channel, email_account_id, whatsapp_account_id,
    template_version_id, signature_id, execution_window_id, daily_limit,
    hourly_limit, condition, wait_hours, status, activated_at, completed_at,
    created_at, updated_at
)
SELECT
    'st-1-' || id,
    id,
    1,
    'email',
    email_account_id,
    NULL,
    template_version_id,
    signature_id,
    execution_window_id,
    daily_limit,
    hourly_limit,
    'always',
    0,
    status,
    activated_at,
    completed_at,
    created_at,
    updated_at
FROM campaign
WHERE email_account_id IS NOT NULL;

ALTER TABLE campaign DROP COLUMN email_account_id;
ALTER TABLE campaign DROP COLUMN template_version_id;
ALTER TABLE campaign DROP COLUMN signature_id;
ALTER TABLE campaign DROP COLUMN execution_window_id;
ALTER TABLE campaign DROP COLUMN daily_limit;
ALTER TABLE campaign DROP COLUMN hourly_limit;

-- ─── 4 · El intento sabe de qué etapa salió ─────────────────────────────────

-- Sin esto, un intento de WhatsApp y uno de correo de la misma campaña son
-- indistinguibles salvo por el canal — que coincide con la etapa **hoy**, con
-- una etapa por canal. Guardar la etapa y no deducirla del canal es lo que
-- permite contar por etapa, reintentar una etapa sin tocar la otra, y que la
-- cifra de ACTIVIDAD siga significando algo si algún día hay dos etapas del
-- mismo canal.
--
-- `ON DELETE SET NULL`, no CASCADE: borrar una etapa no puede borrar la prueba
-- de lo que se envió desde ella. Es la misma razón que en la V1 con el contacto.
CREATE TABLE message_attempt_nueva (
    id                  TEXT PRIMARY KEY NOT NULL,
    campaign_id         TEXT NOT NULL REFERENCES campaign(id) ON DELETE CASCADE,
    stage_id            TEXT REFERENCES campaign_stage(id) ON DELETE SET NULL,
    channel             TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    contact_id          TEXT REFERENCES contact(id) ON DELETE SET NULL,
    contact_address     TEXT NOT NULL,
    email_account_id    TEXT REFERENCES email_account(id) ON DELETE SET NULL,
    whatsapp_account_id TEXT REFERENCES whatsapp_account(id) ON DELETE SET NULL,
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
    id, campaign_id, stage_id, channel, contact_id, contact_address,
    email_account_id, whatsapp_account_id, idempotency_key, state, attempt_count,
    scheduled_for, claimed_at, sent_at, provider_message_id, provider_response,
    error_kind, error_detail, next_retry_at, created_at, updated_at
)
SELECT
    a.id, a.campaign_id,
    -- La etapa que se acaba de crear para esa campaña, si la hubo.
    (SELECT s.id FROM campaign_stage s
      WHERE s.campaign_id = a.campaign_id AND s.channel = a.channel),
    a.channel, a.contact_id, a.contact_address,
    a.email_account_id, NULL, a.idempotency_key, a.state, a.attempt_count,
    a.scheduled_for, a.claimed_at, a.sent_at, a.provider_message_id,
    a.provider_response, a.error_kind, a.error_detail, a.next_retry_at,
    a.created_at, a.updated_at
FROM message_attempt a;

DROP TABLE message_attempt;
ALTER TABLE message_attempt_nueva RENAME TO message_attempt;

CREATE UNIQUE INDEX idx_attempt_unique
    ON message_attempt(campaign_id, channel, contact_address);

CREATE UNIQUE INDEX idx_attempt_idempotency
    ON message_attempt(idempotency_key);

CREATE INDEX idx_attempt_contacto ON message_attempt(contact_id);

CREATE INDEX idx_attempt_work
    ON message_attempt(state, scheduled_for)
    WHERE state IN ('queued', 'failed');

CREATE INDEX idx_attempt_campaign_state
    ON message_attempt(campaign_id, state);

-- Contar por etapa es la consulta de ACTIVIDAD, y con dos etapas por campaña
-- la de campaña sola ya no basta.
CREATE INDEX idx_attempt_stage_state
    ON message_attempt(stage_id, state);

-- ─── 5 · El ritmo, también por canal ────────────────────────────────────────

-- `rate_budget` colgaba de `email_account`. Un número de WhatsApp necesita su
-- propio cubo —con límites que además fija Meta y cambia sola—, y meterlo en la
-- misma columna obligaría a una clave foránea que apunta a dos tablas, que no
-- existe.
CREATE TABLE rate_budget_nueva (
    channel             TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    email_account_id    TEXT REFERENCES email_account(id) ON DELETE CASCADE,
    whatsapp_account_id TEXT REFERENCES whatsapp_account(id) ON DELETE CASCADE,
    window_kind         TEXT NOT NULL CHECK (window_kind IN ('hourly', 'daily')),
    window_start        TEXT NOT NULL,
    consumed            INTEGER NOT NULL DEFAULT 0,
    updated_at          TEXT NOT NULL,
    CHECK (
        (channel = 'email'
            AND email_account_id IS NOT NULL AND whatsapp_account_id IS NULL)
        OR (channel = 'whatsapp'
            AND whatsapp_account_id IS NOT NULL AND email_account_id IS NULL)
    )
) STRICT;

INSERT INTO rate_budget_nueva (
    channel, email_account_id, whatsapp_account_id, window_kind, window_start,
    consumed, updated_at
)
SELECT 'email', email_account_id, NULL, window_kind, window_start, consumed, updated_at
FROM rate_budget;

DROP TABLE rate_budget;
ALTER TABLE rate_budget_nueva RENAME TO rate_budget;

-- Un cubo por cuenta y ventana. Dos índices parciales en vez de una clave
-- primaria, porque la cuenta vive en una columna distinta según el canal.
CREATE UNIQUE INDEX idx_rate_budget_email
    ON rate_budget(email_account_id, window_kind)
    WHERE email_account_id IS NOT NULL;

CREATE UNIQUE INDEX idx_rate_budget_whatsapp
    ON rate_budget(whatsapp_account_id, window_kind)
    WHERE whatsapp_account_id IS NOT NULL;
