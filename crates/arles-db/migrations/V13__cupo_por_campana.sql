-- ════════════════════════════════════════════════════════════════════════
-- V13 · El cupo diario se cuenta por campaña, no por buzón
-- ════════════════════════════════════════════════════════════════════════
--
-- Decisión de Dirección del 22/10/2026 (D-13), con sus palabras:
--
--   «si en una campaña coloco 100 mensajes por día, y en otra campaña coloco
--    100 también con el mismo remitente, tengo que respetar esa solicitud, ya
--    que el cliente está colocando que se envíen 100 en cada campaña, por lo
--    tanto sí se tienen que sumar… para respetar el tiempo que el cliente
--    calculó para su campaña de correos»
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ UNA COLUMNA Y NO UNA SUMA
--
-- La forma obvia de «que se sumen» es dejar el cubo donde está —uno por
-- buzón— y comparar el gasto contra la SUMA de los límites de las campañas
-- que usan ese buzón. Son 200, se gastan 200, el total cuadra.
--
-- **Y el plazo de una de las dos sigue roto.** Con un solo contador de 200, la
-- campaña que el motor atienda primero puede gastarse los 200 ella sola y
-- dejar a la otra en cero. El total sale bien y la promesa del paso 6 sigue
-- siendo falsa — que es exactamente el problema que esta decisión viene a
-- resolver, no el número agregado.
--
-- Para que sean «100 de una y 100 de otra» hace falta que cada campaña lleve
-- **su propio recuento**. Entonces el total es la suma por construcción, y
-- nadie tiene que calcularla para que se respete.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR ETAPA Y NO POR CAMPAÑA, AUNQUE DIRECCIÓN DIJERA «CAMPAÑA»
--
-- El límite no vive en `campaign`: la V4 lo movió a `campaign_stage`, junto
-- con la ventana, el remitente y la plantilla. Una campaña puede tener varias
-- etapas, cada una con su ritmo — así se separó el correo de WhatsApp, que
-- empieza mucho más lento (L-6).
--
-- Colgar el cubo de la campaña obligaría a sumar los límites de sus etapas
-- para compararlos con un contador único, y eso es el mismo defecto de arriba
-- un nivel más abajo. Se cuelga de la etapa, que es donde está el límite que
-- se compara. Hoy, con una etapa de correo por campaña, las dos cosas se ven
-- iguales; el día que no lo sean, esto ya estará bien.
--
-- ────────────────────────────────────────────────────────────────────────
-- QUÉ SE PIERDE AL MIGRAR, Y SE DICE EN VEZ DE CALLARLO
--
-- Los cubos que había cuelgan del buzón y **no se pueden repartir**: un
-- contador de 37 envíos de una cuenta que sirve a tres campañas no sabe
-- cuántos puso cada una. Inventar un reparto sería peor que vaciarlo.
--
-- Así que se vacían, y lo que eso significa es concreto: **si alguien
-- actualiza ARLES a media tarde habiendo enviado ya parte del cupo de hoy, el
-- contador vuelve a cero y ese día puede salir más correo del previsto.** Una
-- vez, el día de la actualización. Es el precio, está medido en lo que es, y
-- la alternativa —conservar un número que no se sabe de quién es— dejaría
-- campañas con un cupo gastado que nunca gastaron.
--
-- ────────────────────────────────────────────────────────────────────────
-- Y LO QUE ESTA MIGRACIÓN CUESTA FUERA DE AQUÍ
--
-- Sube el esquema a V13, y **un respaldo tomado con el V12 ya no se puede
-- restaurar**: es la limitación declarada en la 8.3b, y ésta es la segunda
-- migración que la dispara. Quien tenga un `.arles` de antes de hoy tiene que
-- volver a exportarlo.
--
-- `rate_budget` es `Alcance::Nunca` en el manifiesto del respaldo, así que la
-- columna nueva **no cambia lo que viaja** en un respaldo: este cupo es estado
-- de este equipo y de este momento, y restaurarlo daría permiso para enviar
-- algo que ya se envió.

-- ─── La columna ─────────────────────────────────────────────────────────
--
-- `ALTER TABLE … ADD COLUMN` y no la danza de tabla nueva, copia, DROP y
-- RENAME que usó la V4: aquí no cambia ninguna restricción de las que ya
-- están, sólo se añade una. Y `DROP TABLE` sobre una tabla con hijos es lo
-- que una vez se llevó por delante la audiencia congelada y el registro de
-- envíos.
--
-- Entra como NULL porque SQLite no admite otra cosa al añadir una columna con
-- clave ajena — y da igual, porque las filas que la tendrían en NULL se
-- borran tres líneas más abajo.
ALTER TABLE rate_budget
    ADD COLUMN stage_id TEXT REFERENCES campaign_stage(id) ON DELETE CASCADE;

-- ─── Los cubos viejos ───────────────────────────────────────────────────
--
-- Se vacían por lo dicho arriba: cuelgan del buzón y no se pueden repartir
-- entre las campañas que lo usan.
DELETE FROM rate_budget;

-- ─── Los índices ────────────────────────────────────────────────────────
--
-- Los de la V4 eran `(cuenta, ventana)` únicos, que es exactamente lo que
-- ahora impide que dos campañas tengan su propio cubo sobre el mismo buzón.
-- Se sustituyen por `(cuenta, etapa, ventana)`.
--
-- Siguen siendo PARCIALES y por canal, por la razón que explicó la V4: la
-- cuenta vive en una columna distinta según el canal, y una clave primaria no
-- puede cubrir las dos. Y llevan `stage_id IS NOT NULL` porque en un índice
-- único SQLite trata cada NULL como distinto: sin ese filtro, una fila sin
-- etapa no chocaría con otra sin etapa y volverían los cubos duplicados que
-- esto viene a ordenar.
DROP INDEX idx_rate_budget_email;
DROP INDEX idx_rate_budget_whatsapp;

CREATE UNIQUE INDEX idx_rate_budget_email
    ON rate_budget(email_account_id, stage_id, window_kind)
    WHERE email_account_id IS NOT NULL AND stage_id IS NOT NULL;

CREATE UNIQUE INDEX idx_rate_budget_whatsapp
    ON rate_budget(whatsapp_account_id, stage_id, window_kind)
    WHERE whatsapp_account_id IS NOT NULL AND stage_id IS NOT NULL;

-- Y uno más, que no es de unicidad: el semáforo de Remitentes (D-14) pregunta
-- «¿cuánto ha gastado hoy este buzón, entre todas sus campañas?», y eso
-- recorre todas las filas de la cuenta. Sin él es un barrido de la tabla.
CREATE INDEX idx_rate_budget_por_cuenta
    ON rate_budget(email_account_id, window_kind)
    WHERE email_account_id IS NOT NULL;
