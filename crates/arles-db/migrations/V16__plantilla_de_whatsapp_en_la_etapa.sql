-- ─────────────────────────────────────────────────────────────────────────
-- D-WA · qué plantilla de Meta manda una etapa de WhatsApp
--
-- Decisión de Dirección del 26/10/2026: el canal de WhatsApp entra en la
-- v1.2.0, con campañas enteras y no sólo el envío de prueba.
--
-- QUÉ FALTABA
--
-- `campaign_stage` sabe desde la V4 de qué canal es y de qué cuenta sale —con
-- su `CHECK` exigiendo exactamente una—, y tiene `template_version_id` para la
-- plantilla de CORREO. Lo que no tenía es dónde apuntar a una plantilla
-- aprobada de Meta, que es lo único que se puede mandar por WhatsApp.
--
-- Sin esta columna, una etapa de WhatsApp activada no sabe QUÉ mandar: el
-- motor la recorrería, pediría el contenido, y no habría ninguno. La campaña
-- se quedaría quieta sin que nada fallara.
--
-- POR QUÉ NO SE REUSA `template_version_id`
--
-- Porque son dos cosas distintas y sólo se parecen en el nombre. Una plantilla
-- de correo es texto nuestro, con sus versiones congeladas, que ARLES redacta.
-- Una de Meta es un identificador que **Meta aprobó**: el texto vive en su
-- panel, no aquí, y lo único que se manda es su nombre, su idioma y los
-- valores de sus huecos. Meterlas en la misma columna obligaría a mirar el
-- canal para saber a qué tabla apunta — un `REFERENCES` que no se puede
-- declarar y que SQLite no podría comprobar.
-- ─────────────────────────────────────────────────────────────────────────

ALTER TABLE campaign_stage
    ADD COLUMN whatsapp_template_id TEXT
        REFERENCES whatsapp_template(id) ON DELETE RESTRICT;

-- Para encontrar las etapas que usan una plantilla antes de retirarla.
--
-- Parcial —sólo las que la tienen— porque las etapas de correo son la mayoría
-- y ninguna va a aparecer nunca en esta consulta: un índice que las incluyera
-- sería más grande y no más rápido.
CREATE INDEX idx_campaign_stage_plantilla_whatsapp
    ON campaign_stage(whatsapp_template_id)
    WHERE whatsapp_template_id IS NOT NULL;

-- ─────────────────────────────────────────────────────────────────────────
-- NO SE AÑADE UN `CHECK` QUE EXIJA LA PLANTILLA SEGÚN EL CANAL, Y ES A
-- PROPÓSITO
--
-- Lo natural sería exigir que una etapa de WhatsApp tenga
-- `whatsapp_template_id` y una de correo no. No se hace por una razón
-- concreta: **una etapa nace antes de elegir su plantilla**. El asistente la
-- crea al elegir canal y número, y la plantilla se elige dos pasos después.
--
-- Un `CHECK` así obligaría a construir la fila entera de una vez o a escribirla
-- en el orden contrario al que la pantalla pregunta, y el primero que tropieza
-- con eso es quien vuelve atrás en el asistente a cambiar algo.
--
-- Quien SÍ lo exige es `activar_campana`, que es donde importa: una etapa de
-- WhatsApp sin plantilla no se puede activar, y ahí el error se puede explicar
-- con las tres partes del §95 en vez de salir como un fallo de la base.
-- ─────────────────────────────────────────────────────────────────────────
