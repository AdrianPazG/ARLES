-- ARLES RELAY I · v1.2.0 · de dónde salió cada lista importada
--
-- Entrega 3.2, decisión ADR-0013 §1, con el origen concreto confirmado por
-- Dirección el 18/09/2026: las cinco opciones se aceptan tal cual.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ NO BASTA UNA CASILLA
--
-- `import_batch` ya guardaba `consent_affirmation`: el texto que el usuario
-- aceptó al importar. Eso responde a «¿afirmó algo?», y no responde a «¿QUÉ
-- afirmó?».
--
-- La diferencia importa el día que llegue una reclamación. «El cliente marcó
-- una casilla» es prueba débil: no dice de dónde salieron esos contactos, y
-- obliga a creer en la palabra de quien la marcó. «El cliente declaró que esta
-- lista salió de su propio formulario web, el 18 de septiembre, y aceptó este
-- texto, cuyo hash es éste» es otra cosa. Registra QUÉ se afirmó, no sólo QUE
-- se afirmó — y deja ver patrones entre clientes, que es lo que permite
-- detectar a tiempo a quien siempre importa «directorios públicos».
--
-- ── Las cinco opciones, y por qué son una lista cerrada ──
--
-- formulario_propio · clientes_existentes · evento_o_feria ·
-- directorio_publico · otro
--
-- Cerrada y no texto libre porque un campo libre acaba lleno de «varios», «de
-- siempre» y cadenas vacías, y entonces no hay nada que analizar. `otro` existe
-- para que nadie mienta por no encontrar su caso: es mejor un «otro» honesto
-- que un «clientes_existentes» falso.
--
-- ── Por qué el hash además del texto ──
--
-- El texto se guarda íntegro porque hay que poder enseñarlo. El hash se guarda
-- porque hay que poder demostrar que **no se ha tocado desde entonces**: si la
-- redacción cambia en 2027, el texto viejo sigue ahí, pero sin hash nadie puede
-- distinguir «así estaba» de «así lo dejamos después».
--
-- ── Lo que esta migración NO hace ──
--
-- No importa nada. Es dónde se guarda la declaración; quien la pide es la
-- pantalla de importación, y quien la escribe es el repositorio.
--
-- ⚠ El TEXTO exacto de la afirmación sigue **bloqueado por P-09** (revisión
-- jurídica). Hasta que llegue, la interfaz lo enseña marcado como provisional y
-- visible como tal, según ADR-0013. El esquema no bloquea nada: guarda el texto
-- que sea.
-- ═══════════════════════════════════════════════════════════════════════════

-- ─── El origen concreto de la lista ─────────────────────────────────────────

-- Se añade con DEFAULT porque la columna es NOT NULL y SQLite exige un valor
-- para las filas que ya existan. En la práctica no hay ninguna —nunca ha habido
-- importación—, pero una migración que asume que una tabla está vacía es una
-- migración que falla en el único equipo donde no lo estaba.
--
-- El DEFAULT es 'otro' y no una de las cinco reales: una fila anterior a esta
-- columna no declaró origen, y marcarla como 'formulario_propio' sería
-- inventarse una declaración que nadie hizo.
ALTER TABLE import_batch
    ADD COLUMN origin TEXT NOT NULL DEFAULT 'otro'
    CHECK (origin IN (
        'formulario_propio',
        'clientes_existentes',
        'evento_o_feria',
        'directorio_publico',
        'otro'
    ));

-- Huella del texto aceptado, para demostrar que no se ha tocado.
--
-- Admite NULL: las filas anteriores no tienen hash, y ponerles uno calculado
-- ahora afirmaría que ese texto es el que se aceptó entonces. Un NULL honesto
-- dice «no se sabe», que es la verdad.
ALTER TABLE import_batch ADD COLUMN consent_hash TEXT;

-- ─── De qué importación viene cada contacto ─────────────────────────────────

-- Sin cascada y sin SET NULL: **a propósito**.
--
-- Con ON DELETE CASCADE, borrar el registro de una importación borraría los
-- contactos que trajo — justo la prueba que se quiere conservar. Con SET NULL,
-- los contactos sobrevivirían pero se quedarían huérfanos, sin poder responder
-- a «¿de dónde salió esta persona?».
--
-- Sin nada de eso, SQLite impide borrar un lote que todavía tiene contactos, y
-- eso es lo correcto: el rastro de una importación no se borra porque estorbe.
ALTER TABLE contact ADD COLUMN import_batch_id TEXT REFERENCES import_batch(id);

CREATE INDEX idx_contact_import ON contact(import_batch_id)
    WHERE import_batch_id IS NOT NULL;
