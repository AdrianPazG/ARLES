-- V14 · A quién va dirigida una campaña (D-15)
--
-- ─────────────────────────────────────────────────────────────────────────────
-- LO QUE PIDIÓ DIRECCIÓN, CON SUS PALABRAS (22/10/2026)
--
--   «Al iniciar una campaña podemos colocar una opción para cargar los
--   contactos en un archivo xls o csv… así cuando creen una campaña por
--   ejemplo "Videovigilancia" y carguen todos los contactos donde podrían
--   tener interés de contratar videovigilancia. Pero si se crea otra campaña
--   de "Desarrollo de aplicaciones" puedan cargar su tabla de contactos… y no
--   mezclar contactos donde no sean de interés del público y así arriesgar el
--   dominio de que le den spam.»
--
-- Hasta hoy **toda campaña va a todo el padrón**: `activar_campana` recorre
-- `contact` por `company_id` y nada más. Mandar una campaña de videovigilancia
-- a quien pidió una aplicación no es sólo ruido: es la vía más rápida a que
-- marquen el correo como no deseado, y la tasa de quejas es la cifra que más
-- pesa en la reputación del dominio.
--
-- ─────────────────────────────────────────────────────────────────────────────
-- POR QUÉ ESTA MIGRACIÓN ES UNA COLUMNA Y NO DOS TABLAS
--
-- Porque `contact_list` y `contact_list_member` **ya están en el esquema desde
-- la V1**, con la forma exacta que hace falta. Lo que nunca existió es quien
-- las escriba y quien las lea: están en la lista de `PENDIENTES_CONOCIDOS` del
-- validador desde la Fase 6, con su motivo —«§40, declarado fuera del alcance».
--
-- Así que esto no inventa un modelo: enchufa el que ya había.
-- ─────────────────────────────────────────────────────────────────────────────

-- ── A quién va dirigida ──────────────────────────────────────────────────────
--
-- NULL significa «toda la empresa», que es lo que hacían todas las campañas
-- hasta hoy y lo que seguirán haciendo las ya creadas. Es una opción legítima
-- —un aviso que va a todo el padrón existe— pero **la pantalla la ofrece con
-- esas palabras**, elegida a propósito y nunca por omisión.
--
-- `ON DELETE RESTRICT` y no las otras dos, y la diferencia no es de estilo:
--
--   - `CASCADE` borraría la campaña al borrar su lista. Es el fallo que en la
--     Fase 1 costó la audiencia congelada y el registro de envíos (F1), por
--     cuarta vez.
--   - `SET NULL` dejaría la campaña apuntando a «toda la empresa» **sin que
--     nada avise**: una campaña de 300 personas pasaría a 450 000 en silencio,
--     que es exactamente el riesgo que esta migración viene a quitar.
--   - `RESTRICT` impide borrar una lista que una campaña usa, y el día que
--     haya un botón de borrar listas tendrá que decir por qué no puede. Eso es
--     un mensaje que escribir, no un dato que perder.
ALTER TABLE campaign ADD COLUMN contact_list_id TEXT
    REFERENCES contact_list(id) ON DELETE RESTRICT;

-- Para preguntar «¿qué campañas usan esta lista?» sin recorrer la tabla, que
-- es lo que hace falta antes de borrar una lista y lo que enseña la pantalla.
CREATE INDEX idx_campaign_lista ON campaign(contact_list_id);

-- ── Dos listas no se pueden llamar igual ─────────────────────────────────────
--
-- `tag` tiene este índice desde la V1 y `contact_list` no, y se nota en cuanto
-- algo las escribe: el desplegable del asistente enseña **el nombre**, así que
-- dos «Videovigilancia» serían dos opciones idénticas y elegir la equivocada
-- no daría ningún error — mandaría la campaña a la gente de la otra.
--
-- Hoy no hay ni una fila en la tabla, así que crear el índice no puede chocar
-- con nada. Dentro de un mes de uso sí podría, y entonces esta migración
-- fallaría al arrancar.
CREATE UNIQUE INDEX idx_contact_list_unique ON contact_list(company_id, name);
