-- ARLES RELAY I · v1.2.0 · la parada de emergencia, guardada
--
-- Entrega 4.4, MOTOR_DE_EJECUCION.md §9.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ UNA TABLA Y NO UNA VARIABLE EN MEMORIA
--
-- La parada de emergencia es el botón de «que no salga nada más». Guardada
-- sólo en memoria, cerrar ARLES la levanta — y quien la pulsó no está delante
-- para verlo.
--
-- El caso concreto: alguien se da cuenta de que la campaña que acaba de
-- activar lleva la plantilla equivocada, pulsa parar, y cierra el programa
-- para irse. Al día siguiente ARLES arranca, no encuentra ninguna parada, y
-- sigue enviando la plantilla equivocada a los cuatro mil que faltaban.
--
-- Guardada aquí, arrancar la encuentra puesta y lo dice.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ NO VA EN `app_preference`
--
-- Existe ya una tabla de preferencias, y habría bastado con una clave más.
-- No se hace, y el motivo no es estético: `app_preference` guarda **gustos de
-- la interfaz** —el tema, si la barra va plegada—, y su repositorio filtra por
-- una lista blanca de claves de interfaz precisamente para que nadie meta ahí
-- estado del producto.
--
-- Una parada de emergencia no es un gusto. Es una orden operativa que el motor
-- obedece antes de cada envío, y mezclarla con las preferencias significaría
-- que cualquier código que limpie o restablezca preferencias —restablecer la
-- interfaz, un borrado de ajustes, una importación de configuración— podría
-- levantar la parada sin saber lo que está haciendo.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ SE GUARDA UN INSTANTE Y NO UN SÍ/NO
--
-- `emergency_stop_at` es NULL cuando no hay parada, y un instante cuando la
-- hay. Un booleano respondería «¿está parado?»; un instante responde además
-- «¿desde cuándo?», que es lo que la pantalla necesita para decir «detenido
-- desde las 14:32» en vez de un «detenido» a secas que invita a preguntarse si
-- se quedó puesto de ayer (§94: cifras, no adjetivos).
--
-- `emergency_stop_by` es opcional a propósito: la v1.2.0 es de un solo usuario
-- (D-4), así que muchas veces no habrá nadie a quien nombrar. Existe ya para
-- que el día que haya varios, la columna no tenga que aparecer con las filas
-- viejas mintiendo.

CREATE TABLE engine_control (
    -- Una fila por empresa. La `PRIMARY KEY` lo impone: dos filas para la
    -- misma empresa darían dos respuestas a «¿hay que parar?», y el motor
    -- leería la que la consulta devolviera primero.
    company_id        TEXT PRIMARY KEY NOT NULL
                      REFERENCES company(id) ON DELETE CASCADE,

    -- NULL = el motor puede enviar. Un instante = parada puesta desde entonces.
    emergency_stop_at TEXT,

    -- Quién la puso, si consta.
    emergency_stop_by TEXT,

    updated_at        TEXT NOT NULL,

    -- Quien no está parado no tiene a quién nombrar. Sin esta comprobación
    -- podría quedar una fila con autor y sin instante, y cualquiera que la
    -- leyera pensaría que hay una parada puesta.
    CHECK (emergency_stop_at IS NOT NULL OR emergency_stop_by IS NULL)
) STRICT;
