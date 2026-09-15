-- ARLES RELAY I · v1.2.0 · preferencias de interfaz
--
-- Dirección decidió (P-11) que la barra lateral se pliega a mano, se queda en
-- iconos y **se recuerda al reabrir**. Eso último es lo que obliga a esta tabla.
--
-- Por qué en la base de datos y no en `localStorage`:
--
--   · `localStorage` vive en el perfil de la WebView, que no es el perfil de la
--     aplicación. Se borra con la caché del sistema, no viaja con el respaldo
--     `.arles` y en Windows depende del directorio de WebView2. Una preferencia
--     que se pierde al limpiar la caché es una preferencia que no se recuerda.
--   · Aquí queda dentro de la base cifrada, entra en el respaldo y se restaura
--     con él.
--
-- Por qué clave/valor y no una columna por preferencia: son ajustes de interfaz
-- que van a crecer pantalla a pantalla, y una migración por casilla plegada es
-- coste sin contrapartida. Las reglas son del código que las lee, no del
-- esquema: ninguna preferencia de interfaz gobierna nada de negocio.
--
-- Deliberadamente **sin company_id**: es del equipo y del usuario que se sienta
-- delante, no de la empresa. Si mañana hay varias empresas en una instalación
-- (§8), la barra lateral sigue plegada igual.

CREATE TABLE ui_preference (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;
