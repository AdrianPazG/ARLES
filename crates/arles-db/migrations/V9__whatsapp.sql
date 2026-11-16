-- ARLES RELAY I · v1.2.0 · las plantillas de Meta y el recuento de respuestas
--
-- Entrega W.1.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- QUÉ FALTABA
--
-- La V4 dejó `whatsapp_account` entera: el número, el modo, Coexistencia, la
-- calidad que Meta decide. Lo que no dejó es **con qué se escribe**.
--
-- En correo, la etapa apunta a una `template_version` que ARLES escribió y
-- versionó. En WhatsApp eso no vale, y no por comodidad: **el texto lo aprueba
-- Meta**, palabra por palabra, y lo que viaja por la API es el NOMBRE de una
-- plantilla suya más los valores de sus huecos. ARLES no manda texto.
--
-- Así que hace falta una tabla distinta, que no versiona nada porque no hay
-- nada que versionar: lo que Meta tiene aprobado es lo que hay.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ ESTO ES UNA TABLA APARTE Y NO UNAS COLUMNAS EN `template`
--
-- Se probó a reusar `template` con una columna de canal, y no encaja en tres
-- sitios a la vez:
--
--   · `template_version` guarda `subject` y `body_html`, que en WhatsApp no
--     existen — un mensaje de plantilla no tiene asunto ni HTML.
--   · Las versiones de `template` son nuestras y son inmutables por decisión
--     nuestra. Las de Meta cambian de estado **sin que nosotros hagamos nada**:
--     una plantilla aprobada puede quedar pausada mañana por cómo reacciona la
--     gente. Una tabla de versiones inmutables no puede representar eso.
--   · Los huecos de Meta son POSICIONALES (`{{1}}`) y los nuestros llevan
--     nombre (`{{nombre}}`). Guardar unos donde van los otros perdería el
--     orden, y el orden es el significado.
--
-- Mezclarlas habría obligado a que la mitad de las columnas fueran nulas según
-- el canal, que es el olor de dos tablas dentro de una.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- Y POR QUÉ EL RECUENTO DE RESPUESTAS NO GUARDA LO QUE DICE LA GENTE
--
-- Dirección decidió el 29/09/2026 contar las respuestas y **no** guardarlas.
-- Esta tabla es esa decisión hecha esquema: hay un contador y una marca de
-- tiempo, y no hay ninguna columna donde quepa un texto.
--
-- No es sólo higiene de datos personales, aunque también: lo que protege al
-- número ante Meta es **que te contesten**, y para eso basta saber que hubo
-- respuesta. Guardar el contenido añadiría datos personales nuevos —que el
-- abogado no ha revisado (P-09)— a cambio de nada que ARLES vaya a usar.
--
-- Si algún día se guardan, será con una migración que lo diga en su nombre y
-- con el aviso de privacidad actualizado, no añadiendo una columna aquí.

-- ═══════════════════════════════════════════════════════════════════════════
-- Las plantillas que existen en Meta
-- ═══════════════════════════════════════════════════════════════════════════

CREATE TABLE whatsapp_template (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    -- De qué número cuelga. Una plantilla vive en una cuenta de Meta concreta,
    -- y la misma empresa puede tener dos números en dos cuentas distintas —que
    -- es justo lo que se pidió para separar el número de pruebas del de
    -- seguimiento—. RESTRICT y no CASCADE: borrar un número no debe llevarse
    -- por delante el registro de qué se tenía aprobado.
    whatsapp_account_id TEXT NOT NULL
                        REFERENCES whatsapp_account(id) ON DELETE RESTRICT,

    -- El `name` de Meta. **Es el identificador real del mensaje**: es lo que se
    -- manda por la API, no el texto.
    meta_name          TEXT NOT NULL,
    -- El idioma en la forma de Meta: 'es_MX'. La misma plantilla en dos idiomas
    -- son dos plantillas para Meta, y por eso entra en la clave única.
    language           TEXT NOT NULL,

    -- Lo que Meta dice, y nosotros sólo leemos. Los cinco estados son los de la
    -- API. **Sólo 'approved' puede enviar**, y esa regla vive en
    -- `arles_core::whatsapp::EstadoEnMeta::se_puede_usar`, escrita por lo que
    -- acepta y no por lo que rechaza: un estado nuevo de Meta no debe empezar a
    -- enviar solo.
    status             TEXT NOT NULL
                       CHECK (status IN ('approved', 'pending', 'rejected',
                                         'paused', 'disabled')),
    status_checked_at  TEXT,

    -- El texto aprobado, **para poder enseñarlo en pantalla**. No se manda: lo
    -- tiene Meta. Se guarda porque quien elige una plantilla en el asistente
    -- necesita leer lo que va a salir, y pedírselo a Meta en ese momento
    -- pondría la pantalla a depender de la red.
    body               TEXT NOT NULL,

    -- Qué variable de ARLES va en cada hueco, **en orden**, separadas por comas:
    -- 'nombre,mi_empresa' quiere decir que {{1}} es el nombre y {{2}} la
    -- empresa propia.
    --
    -- Texto y no una tabla hija a propósito: son dos o tres valores, siempre se
    -- leen juntos y completos, y nunca se consultan por separado. Una tabla
    -- aparte añadiría una junta a cada envío para no ganar nada.
    --
    -- Vacío es legítimo: una plantilla sin huecos es un mensaje fijo.
    slots              TEXT NOT NULL DEFAULT '',

    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
) STRICT;

-- Meta no deja dos plantillas con el mismo nombre e idioma en la misma cuenta,
-- así que la base tampoco. Sin esto, dar de alta la misma plantilla dos veces
-- —al sincronizar con Meta, por ejemplo— dejaría dos filas y el asistente
-- ofrecería la misma opción repetida, con dos estados que podrían discrepar.
CREATE UNIQUE INDEX idx_whatsapp_template_unica
    ON whatsapp_template(whatsapp_account_id, meta_name, language);

CREATE INDEX idx_whatsapp_template_empresa
    ON whatsapp_template(company_id, status);

-- ═══════════════════════════════════════════════════════════════════════════
-- Cuánta gente contestó
-- ═══════════════════════════════════════════════════════════════════════════
--
-- Una fila por etapa y por persona, sin el texto de lo que dijo.
--
-- Por persona y no un simple contador porque la pregunta que hay que poder
-- contestar es «a cuánta GENTE le interesó», no «cuántos mensajes entraron».
-- Quien conteste cinco veces cuenta una, y el `UNIQUE` lo impone en vez de
-- confiar en que el código se acuerde.

CREATE TABLE whatsapp_reply (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    -- De qué etapa venía el mensaje al que contestaron. CASCADE porque esto es
    -- una métrica de esa etapa: sin ella no significa nada.
    campaign_stage_id  TEXT NOT NULL
                       REFERENCES campaign_stage(id) ON DELETE CASCADE,
    -- El número que contestó, ya normalizado. Es un dato personal y por eso es
    -- lo ÚNICO personal que hay aquí: hace falta para no contar dos veces a la
    -- misma persona, y para poder borrarlo si ejerce su derecho de cancelación.
    from_normalized    TEXT NOT NULL,
    -- Cuándo llegó la primera respuesta de esta persona.
    first_reply_at     TEXT NOT NULL,

    -- ── AQUÍ NO HAY UNA COLUMNA CON EL MENSAJE, Y NO ES UN OLVIDO ──
    --
    -- Ver la cabecera. El texto se descarta al recibirlo, antes de escribir
    -- nada. Lo que ARLES necesita saber es que hubo respuesta.

    UNIQUE (campaign_stage_id, from_normalized)
) STRICT;

CREATE INDEX idx_whatsapp_reply_etapa ON whatsapp_reply(campaign_stage_id);
