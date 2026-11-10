-- ════════════════════════════════════════════════════════════════════════
-- V17 · Una baja puede venir de quien recibe el correo
-- ════════════════════════════════════════════════════════════════════════
--
-- Decisión de Dirección del 10/11/2026, con sus palabras:
--
--   «Necesito que sea posible darse de baja de manera automatica, busca una
--    alternativa mas sencilla para el administrador de la aplicacion y para
--    el usuario»
--
-- R-25, que estaba pendiente desde el 23/10/2026. La alternativa elegida no
-- monta ningún servidor: ARLES lee el buzón del remitente —sólo lectura— y
-- aplica la baja de quien contestó, bien pulsando el botón de cancelar
-- suscripción de su cliente de correo (lo habilita la cabecera
-- `List-Unsubscribe` que ARLES manda desde el 19/10/2026), bien escribiendo
-- la palabra de vuelta.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ UN ORIGEN NUEVO Y NO `user`, QUE NO HABRÍA COSTADO UNA MIGRACIÓN
--
-- Porque sería falso, y de la forma concreta que este proyecto ya se ha
-- cobrado una vez. `user` significa «lo hizo una persona desde ARLES»; esto
-- lo hizo **quien recibe el correo**, desde su bandeja. Son dos manos
-- distintas, y guardarlas con la misma etiqueta es diagnosticar con la
-- variante de otro: el defecto DOC-14 del 03/11/2026, donde una base que no
-- podía dar aleatoriedad decía al usuario que su base era de una versión
-- anterior.
--
-- Lo que se pierde al ahorrarse la migración no es cosmético. El día que haya
-- que defender por qué una dirección está en la lista de supresión —y para
-- eso existe la lista—, «la apuntó alguien de la empresa» y «la pidió ella
-- misma» no valen lo mismo delante de nadie.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ SE RECREA LA TABLA, Y POR QUÉ AQUÍ SÍ SE PUEDE
--
-- SQLite no sabe ampliar un `CHECK`: hay que recrear. Y el CLAUDE.md prohíbe
-- `DROP TABLE` de una tabla **padre**, porque con `PRAGMA foreign_keys=ON`
-- borra en cascada a sus hijos — así se perdieron una vez la audiencia
-- congelada y el registro de envíos (el fallo F1, que ha vuelto cuatro
-- veces).
--
-- `suppression_entry` **no es padre de nadie**, y está medido y no supuesto:
-- ninguna tabla del esquema la nombra en un `REFERENCES`. Es hoja — tiene una
-- clave ajena hacia `company`, en el otro sentido, que es la que se vuelve a
-- declarar abajo. Es además el mismo camino que recorrió la V3 sobre esta
-- misma tabla al partir el correo en canales.
--
-- Aun así, es la tabla más delicada del producto: «escribirle a alguien que
-- pidió que no le escribieran no tiene arreglo». El `INSERT … SELECT` copia
-- **columna por columna y por nombre**, nunca por posición, y hay una prueba
-- que puebla la lista antes de migrar y comprueba fila a fila que está entera
-- después.
--
-- ────────────────────────────────────────────────────────────────────────
-- LO QUE ESTA MIGRACIÓN CUESTA
--
-- Nada que no se pueda deshacer: desde el 04/11/2026, un respaldo de un
-- esquema anterior **se pone al día** antes de restaurarse
-- (`arles-db/src/puesta_al_dia.rs`), así que un `.arles` tomado con la V16
-- sigue entrando. Es la primera migración que se escribe con esa red puesta.

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
    -- El quinto es el de esta migración: lo pidió quien recibe el correo.
    origin             TEXT NOT NULL
                       CHECK (origin IN ('smtp_5xx', 'user', 'import',
                                         'whatsapp_webhook', 'email_unsubscribe')),
    notes              TEXT,
    created_at         TEXT NOT NULL,
    created_by         TEXT
) STRICT;

-- Por nombre de columna y no por posición: una migración futura que mueva una
-- columna metería los campos corridos sin que una sola fila fallara. Es el
-- arreglo que el formato del respaldo necesitó en la 8.2b, aplicado antes de
-- necesitarlo.
INSERT INTO suppression_entry_nueva (
    id, company_id, channel, address_normalized, scope, request_id,
    reason, origin, notes, created_at, created_by
)
SELECT id, company_id, channel, address_normalized, scope, request_id,
       reason, origin, notes, created_at, created_by
FROM suppression_entry;

DROP TABLE suppression_entry;
ALTER TABLE suppression_entry_nueva RENAME TO suppression_entry;

-- Los índices se van con la tabla y hay que volver a crearlos. Son los mismos
-- de la V3: el único que impide duplicar una baja, y el que agrupa las filas
-- de una petición global.
CREATE UNIQUE INDEX idx_suppression_unique
    ON suppression_entry(company_id, channel, address_normalized);

CREATE INDEX idx_suppression_peticion
    ON suppression_entry(request_id)
    WHERE request_id IS NOT NULL;
