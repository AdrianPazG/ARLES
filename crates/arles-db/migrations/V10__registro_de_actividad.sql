-- ARLES RELAY I · v1.2.0 · el registro de actividad
--
-- Entrega 7.1.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- QUÉ SE PERDÍA
--
-- El motor sabe once cosas distintas y las tira todas.
--
-- `Trabajador::un_paso` devuelve un `Paso`: aceptado, reintentará, se rindió,
-- dio de baja, la cuenta se paró, fuera de ventana, sin cupo, el disyuntor
-- abierto… **cada uno con su clave de texto en español ya escrita** y pensada
-- desde la Fase 4 para esta pantalla. Y `una_vuelta()` en `arles-app` hacía:
--
--     .map(|ultimo| ultimo.hay_mas_que_hacer())
--
-- Lo convertía en un sí/no. De que a las 10:04 la cuenta de ventas se paró no
-- quedaba nada: sólo el estado final de cada intento, sin cuándo ni por qué.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- ESTA TABLA NO ES LA FUENTE DE VERDAD, Y ESO ES DELIBERADO
--
-- La prueba de que se escribió a alguien es `message_attempt`, y lo seguirá
-- siendo: se escribe en la transacción del envío, con su `idempotency_key`, y
-- es lo que sostiene el control anti-abuso (§37) y lo que contesta un derecho
-- ARCO.
--
-- `activity_event` es **derivada y de conveniencia**: cuenta lo que fue
-- pasando. Se escribe FUERA de la transacción del intento, justo después, y
-- por lo tanto **se puede perder una línea si el programa muere en ese
-- instante**. Es un coste aceptado a cambio de no meter una segunda escritura
-- dentro de la transacción que decide si un correo sale.
--
-- La consecuencia práctica hay que tenerla clara: si alguna vez el registro y
-- los intentos discrepan, **gana `message_attempt`**. Ninguna cifra de la
-- pantalla de Actividad se calcula desde aquí; las cifras salen de los
-- intentos (7.2). De aquí sale la narración.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- AQUÍ NO HAY NI UNA DIRECCIÓN DE CORREO, Y ES A PROPÓSITO
--
-- El intento se referencia por su id. La dirección se lee de `message_attempt`
-- al pintarla, no se copia aquí.
--
-- El motivo es P-17, que está abierto justo por esto: `consent_entry` guarda
-- direcciones en claro en una tabla de sólo-añadir, y el día que alguien
-- ejerza su derecho de cancelación esa fila no se podrá tocar. Repetir aquí el
-- mismo error, en una tabla que va a tener millones de filas, sería crear el
-- problema a sabiendas.
--
-- Con este diseño, borrar un contacto o anonimizar su intento deja el registro
-- de actividad **sin nada que identifique a nadie**, sin tocar una sola fila.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- EL VOLUMEN: POR QUÉ HAY UNA COLUMNA `is_outcome`
--
-- Una campaña de 500 000 con reintentos produce cientos de miles de
-- desenlaces. Eso es inevitable y es lo que hay que guardar.
--
-- Lo que NO es inevitable es el ruido: el motor da una vuelta cada cinco
-- segundos, y fuera de horario devuelve `FueraDeVentana` **17 280 veces al
-- día** por etapa. Anotar eso tal cual llena la base de nada.
--
-- Se resuelve en dos sitios, y hacen falta los dos:
--
--   1. **Al escribir** (`anotar_si_cambia` en `actividad.rs`): un suceso de
--      bucle sólo se anota si es DISTINTO del último de esa etapa. «Fuera de
--      ventana» se anota una vez, no 17 280. Ahí está el recorte de verdad.
--   2. **Al recortar** (D-7C): lo que no es desenlace se borra a los dos
--      meses. Los desenlaces **no se borran jamás**.
--
-- `is_outcome` es lo que distingue las dos cosas, y es una columna y no una
-- lista de claves en el código porque el día que se añada un `Paso` nuevo hay
-- que decidir a cuál de los dos grupos pertenece, y esa decisión se toma una
-- vez, al insertarlo.

CREATE TABLE activity_event (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,

    -- De qué campaña y etapa. Nulos en los sucesos que no son de ninguna
    -- —el motor arrancó, la parada de emergencia—, que por eso no llevan
    -- `NOT NULL`.
    campaign_id        TEXT REFERENCES campaign(id) ON DELETE CASCADE,
    campaign_stage_id  TEXT REFERENCES campaign_stage(id) ON DELETE CASCADE,
    channel            TEXT CHECK (channel IS NULL OR channel IN ('email', 'whatsapp')),

    -- **La clave de i18n del suceso** (§139), no un texto y no un código
    -- propio: es exactamente lo que devuelve `Paso::clave_i18n()`, así que la
    -- pantalla la pasa a `t()` y no hay una segunda tabla de traducción que
    -- pueda divergir. Ya nos pasó con el estado de las campañas.
    kind               TEXT NOT NULL,

    -- 1 si es el desenlace de un intento de envío. Ver la cabecera.
    is_outcome         INTEGER NOT NULL DEFAULT 0 CHECK (is_outcome IN (0, 1)),

    -- El intento, si lo hay. **Sin la dirección**: se lee de `message_attempt`.
    message_attempt_id TEXT REFERENCES message_attempt(id) ON DELETE CASCADE,

    -- Lo que la pantalla necesita para decir una cifra sin volver a consultar:
    -- «reintento 2 de 3», «vuelve en 23 minutos». Nulo cuando no aplica.
    number_a           INTEGER,
    number_b           INTEGER,

    occurred_at        TEXT NOT NULL
) STRICT;

-- Lo que pide la pantalla: lo último de esta empresa, y lo último de una
-- campaña. Las dos consultas ordenan por fecha descendente y recortan a 500
-- (D-7C), así que el índice lleva la fecha.
CREATE INDEX idx_activity_empresa_fecha
    ON activity_event(company_id, occurred_at DESC);

CREATE INDEX idx_activity_campana_fecha
    ON activity_event(campaign_id, occurred_at DESC);

-- Para `anotar_si_cambia`: cuál fue el último suceso de esta etapa.
--
-- **Sólo la etapa, sin la fecha.** La primera versión llevaba
-- `(campaign_stage_id, occurred_at DESC)`, que parece mejor y es peor: dentro
-- de una etapa deja las filas ordenadas por fecha, y la consulta pide «la
-- última insertada», que es el `rowid` más alto. Resultado medido con
-- `EXPLAIN QUERY PLAN` sobre 20 000 filas: **«USE TEMP B-TREE FOR ORDER BY»**,
-- un árbol temporal en una consulta que corre cada cinco segundos por etapa.
--
-- Con el índice a secas, SQLite añade el `rowid` como desempate ascendente, y
-- `ORDER BY rowid DESC LIMIT 1` se sirve recorriendo el índice hacia atrás: una
-- sola lectura, sin ordenar nada.
CREATE INDEX idx_activity_etapa
    ON activity_event(campaign_stage_id);

-- Para el recorte bimestral, que sólo mira lo que NO es desenlace.
CREATE INDEX idx_activity_recorte
    ON activity_event(occurred_at)
    WHERE is_outcome = 0;

-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ ESTA TABLA SÍ ADMITE DELETE
--
-- `audit_log` tiene disparadores que prohíben `UPDATE` y `DELETE`: ahí consta
-- quién decidió qué, y eso no se toca nunca.
--
-- Aquí es al revés a propósito. El recorte de D-7C **necesita** borrar, y lo
-- que se borra es ruido de bucle, que no es prueba de nada. Lo que sí es
-- prueba —que se envió a alguien— no vive aquí: vive en `message_attempt`.
--
-- Lo que no se puede hacer es `UPDATE`: un suceso pasó como pasó.
CREATE TRIGGER activity_event_sin_update
BEFORE UPDATE ON activity_event
BEGIN
    SELECT RAISE(ABORT, 'activity_event no se corrige: un suceso pasó como pasó');
END;
