-- ════════════════════════════════════════════════════════════════════════
-- V11 · Los índices que la medición del 06/10/2026 demostró que faltaban
-- ════════════════════════════════════════════════════════════════════════
--
-- Entrega 8.6. Esta migración **no cambia ni un dato**: no crea tablas, no
-- añade columnas, no mueve nada. Añade seis listas ordenadas —índices— para
-- que seis consultas que hoy recorren la tabla entera vayan directas.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ EXISTE, Y POR QUÉ NO EXISTÍA ANTES
--
-- Hasta la 8.5, las trece cifras de `PRESUPUESTO_RENDIMIENTO.md` eran
-- estimaciones de la Fase 0 que nadie había comprobado. El banco de pruebas
-- las midió con 500 000 contactos sintéticos y **dos no se cumplían**:
--
--   · Abrir la tabla de contactos: 1 424 ms de un presupuesto de 500.
--   · Buscar por correo:             331 ms de un presupuesto de 100.
--
-- La causa no se supuso: se sacó de `EXPLAIN QUERY PLAN`. Y la auditoría que
-- siguió no se quedó en esas dos — recorrió **las 295 consultas** de
-- `arles-db` y encontró 23 que barren una tabla que crece.
--
-- ────────────────────────────────────────────────────────────────────────
-- CADA ÍNDICE SE VERIFICÓ ANTES DE ESCRIBIRLO
--
-- Comparando el plan de su consulta antes y después de crearlo. **Uno de los
-- seis que se habían diseñado no servía**: un índice parcial
-- `(state) WHERE state IN ('sending','claimed')` no lo usa SQLite para un
-- `WHERE state = 'sending'`, y se habría quedado ahí costando escrituras sin
-- acelerar nada. Los dos que lo sustituyen llevan el predicado exacto.
--
-- Un índice no es gratis: ocupa disco y hay que mantenerlo en cada escritura.
-- Por eso aquí no hay ninguno «por si acaso»: los seis sirven a una consulta
-- concreta que alguien espera, y se comprobó que la cambian.
--
-- ────────────────────────────────────────────────────────────────────────
-- LO QUE ESTA MIGRACIÓN CUESTA, DICHO ANTES DE QUE PASE
--
-- **Un respaldo `.arles` tomado con el esquema V10 no se podrá restaurar en
-- una base V11.** Es la limitación que la 8.3b documentó: ARLES se niega a
-- restaurar un respaldo de otro esquema antes que dejar media base sin
-- avisar. El 06/10/2026 no existe ningún `.arles` fuera de las pruebas, así
-- que el coste de hacerlo hoy es cero — y por eso se hace hoy.

-- ── 1 · La lista de contactos, y su recuento ────────────────────────────
--
-- Es la pantalla donde el usuario pasa más tiempo, y `contact` no tenía
-- ningún índice para su propia consulta: sólo el de la clave primaria y uno
-- sobre `import_batch_id`.
--
-- Antes:   SCAN contact  +  USE TEMP B-TREE FOR ORDER BY
-- Después: SEARCH contact USING INDEX idx_contact_lista (company_id=?)
--
-- Léase: para enseñar **cien** filas, SQLite leía las 455 864 y las ordenaba
-- en memoria. Y el total de arriba hacía un segundo barrido completo.
--
-- Las columnas son las del `ORDER BY` de la consulta, y eso no es decorativo:
-- medido el 06/10/2026, un índice sobre `contact(company_id)` a secas deja el
-- `USE TEMP B-TREE FOR ORDER BY` y no arregla nada.
--
-- ── LA DIRECCIÓN NO IMPORTA; QUE SEA LA MISMA, SÍ ──
--
-- El `DESC` de aquí es por legibilidad —se lee igual que la consulta— y no
-- por necesidad: SQLite recorre un índice al revés sin coste, así que uno
-- ascendente serviría igual. Lo que **no** sirve es mezclarlos: un
-- `(company_id, created_at ASC, id DESC)` deja un
-- `USE TEMP B-TREE FOR RIGHT PART OF ORDER BY`. Las dos columnas tienen que
-- ir en la misma dirección, sea cual sea.
--
-- Y es **parcial** (`WHERE deleted_at IS NULL`) porque la pantalla nunca
-- enseña los borrados: así el índice no carga con filas que nadie pide.
CREATE INDEX idx_contact_lista
    ON contact(company_id, created_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- ── 2 · Buscar a quién pertenece una dirección ──────────────────────────
--
-- `idx_channel_unique` existe y está bien puesto, pero su orden es
-- `(company_id, channel, value_normalized)`. La consulta que busca una
-- dirección **no sabe de qué canal es** —ni tiene por qué—, así que se salta
-- la columna del medio y SQLite sólo puede aprovechar la primera: recorre
-- todos los canales de la empresa, correos y WhatsApp juntos.
--
-- Antes:   SEARCH ... idx_channel_unique (company_id=?)
-- Después: SEARCH ... idx_channel_direccion (company_id=? AND value_normalized=?)
--
-- Esto es lo que se ejecuta al importar, para saber qué filas chocan con
-- contactos que ya existen: con una lista de diez mil, diez mil veces.
CREATE INDEX idx_channel_direccion
    ON contact_channel(company_id, value_normalized)
    WHERE deleted_at IS NULL;

-- ── 3 y 4 · La recuperación al arrancar ─────────────────────────────────
--
-- `recuperar_al_arrancar` hace dos `UPDATE ... WHERE state = ?` sobre
-- `message_attempt`, y los dos barrían la tabla entera. Con medio millón de
-- intentos eso es un barrido completo **en cada arranque**, y el presupuesto
-- de arranque es de 2 segundos.
--
-- ── POR QUÉ DOS ÍNDICES PARCIALES Y NO UNO COMPLETO ──
--
-- Un índice sobre `state` a secas funcionaría, y costaría mantenerse en cada
-- una de las 500 000 escrituras de una importación. Estos dos sólo contienen
-- **las filas que estaban en vuelo cuando se cerró el programa**: unas
-- decenas, casi siempre cero. Son minúsculos y su mantenimiento es gratis.
--
-- El predicado tiene que ser EXACTAMENTE el de la consulta. Se probó primero
-- con `WHERE state IN ('sending','claimed')` y SQLite **no lo usaba**: se
-- habría quedado ahí sin acelerar nada.
CREATE INDEX idx_attempt_enviando
    ON message_attempt(state)
    WHERE state = 'sending';

CREATE INDEX idx_attempt_tomado
    ON message_attempt(state)
    WHERE state = 'claimed';

-- ── 5 · Las preguntas a la bitácora ─────────────────────────────────────
--
-- `audit_log` es append-only y **crece para siempre**: una fila por cada
-- acción crítica, sin recorte (§91, y a propósito). Cuatro consultas la
-- barrían entera para responder cosas como «¿aceptó el riesgo de volumen de
-- esta campaña?».
--
-- Hoy con mil filas no se nota. Con tres años de uso, sí — y es justo el tipo
-- de lentitud que aparece despacio y nadie sabe de dónde salió.
--
-- `entity_id` va en la clave para que la pregunta más frecuente —acción Y
-- entidad— se conteste sin tocar la tabla: el plan dice COVERING INDEX.
CREATE INDEX idx_audit_accion
    ON audit_log(action, entity_id);

-- ── 6 · Olvidar las respuestas de alguien ───────────────────────────────
--
-- El derecho de cancelación borra las respuestas de WhatsApp de una persona,
-- y lo hacía barriendo la tabla. No es frecuente, pero es una operación que
-- ARLES promete en un plazo legal: lo que no puede es tardar más cuanto más
-- vieja sea la base.
CREATE INDEX idx_whatsapp_reply_de
    ON whatsapp_reply(company_id, from_normalized);

-- ── Y las estadísticas, para que el optimizador use todo esto ───────────
--
-- Sin `ANALYZE`, SQLite elige el plan por reglas generales en vez de por lo
-- que hay de verdad en cada tabla. Es barato aquí —se corre una vez, sobre
-- índices recién creados— y es lo que hace que la elección sea la buena
-- cuando una consulta tiene dos índices entre los que escoger.
ANALYZE;
