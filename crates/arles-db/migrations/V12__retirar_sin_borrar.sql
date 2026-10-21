-- ════════════════════════════════════════════════════════════════════════
-- V12 · Retirar una plantilla o una firma, sin borrarla
-- ════════════════════════════════════════════════════════════════════════
--
-- Decisión de Dirección del 21/10/2026, con sus palabras:
--
--   «me gustaría que quede registrado en algo que no podamos ver al momento
--    de elegir ya no seguir utilizando esa plantilla, así que prácticamente
--    no quedaría borrada, sólo no se muestra porque se decidió "borrar" la
--    plantilla que ya no se quiere utilizar»
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ ESTO ES MEJOR QUE EL `DELETE` QUE SUSTITUYE
--
-- La primera versión de «eliminar» (R-08, el mismo día) borraba de verdad, y
-- por eso tenía que **negarse** cuando una campaña había usado la plantilla:
-- el registro de envíos apunta a una versión concreta, y sin ella la pregunta
-- «¿qué decía el correo que recibió este cliente?» se queda sin respuesta.
--
-- Retirando en vez de borrar, esa negativa **desaparece**: se puede retirar
-- cualquier plantilla, la haya usado una campaña o no, porque el texto sigue
-- ahí para quien tenga que consultarlo. Lo único que cambia es que deja de
-- ofrecerse al crear una campaña nueva.
--
-- Es la misma forma que el contacto dado de baja (`contact.status`) y que la
-- cuenta deshabilitada: **en ARLES lo que se deja de usar no se destruye**.
-- Y es lo que exige el §65 llevado a su conclusión: si no podemos afirmar que
-- un correo llegó, menos podemos destruir la prueba de qué decía.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ UNA FECHA Y NO UNA BANDERA
--
-- `retired_at TEXT NULL` y no `is_retired INTEGER`. Una bandera dice «sí o
-- no»; una fecha dice además **cuándo**, que es lo que convierte esto en algo
-- que se puede auditar — y el `audit_log` guarda la acción, pero una consulta
-- directa a la tabla no tendría por qué cruzarse con él para contestar.
--
-- `NULL` es «en uso», que es el estado de todo lo que ya existe: la migración
-- no toca ni una fila.
--
-- ────────────────────────────────────────────────────────────────────────
-- LO QUE ESTA MIGRACIÓN CUESTA
--
-- Un respaldo tomado con el esquema V11 **ya no se puede restaurar** — la
-- limitación que la 8.3b dejó declarada y que la V11 disparó por primera vez.
-- Hoy no hay ningún `.arles` fuera de las pruebas, así que el coste real
-- vuelve a ser cero; el día que lo haya, no lo será.

ALTER TABLE template ADD COLUMN retired_at TEXT;
ALTER TABLE signature ADD COLUMN retired_at TEXT;

-- Las listas que la interfaz pide son siempre «lo que está en uso», así que
-- el índice lleva el predicado dentro: un índice sobre la columna entera
-- costaría escrituras para distinguir dos valores, y uno de ellos es el 100 %
-- de las filas hasta que alguien retire la primera.
CREATE INDEX idx_template_en_uso
    ON template(company_id, name)
    WHERE retired_at IS NULL;

CREATE INDEX idx_signature_en_uso
    ON signature(company_id, is_default DESC, name)
    WHERE retired_at IS NULL;
