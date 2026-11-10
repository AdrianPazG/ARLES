-- ════════════════════════════════════════════════════════════════════════
-- V18 · Dónde se lee el buzón, y hasta dónde se leyó
-- ════════════════════════════════════════════════════════════════════════
--
-- Tercer tramo de R-25 (D-21, 10/11/2026). Los dos anteriores dejaron la regla
-- —qué cuenta como una baja— y el lector del buzón. Esto es lo que falta para
-- que alguien pueda encenderlo: tres columnas en `email_account`.
--
-- `ALTER TABLE ADD COLUMN` y no recrear la tabla: aquí no hay ningún `CHECK`
-- que ampliar, así que no hace falta el camino caro de la V17. Las filas que ya
-- existen no se tocan.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ EL SERVIDOR DE LECTURA SE GUARDA Y NO SE DEDUCE
--
-- Lo cómodo sería calcularlo: cambiar `smtp.` por `imap.` acierta en Gmail
-- (`smtp.gmail.com` → `imap.gmail.com`), en Titan y en Zoho. Y falla en cuanto
-- un proveedor no sigue esa convención — y entonces ARLES **se inventa un
-- servidor**, intenta conectarse a un nombre que no existe y lo que el usuario
-- lee es «no se pudo entrar al buzón», que apunta a su red o a su contraseña.
--
-- Es el mismo defecto que la V7 evitó con `smtp_host`: la cabecera de esa
-- migración lo dice con otras palabras —una columna `NOT NULL` obligaría a
-- inventarse un dato falso, y ese dato falso acaba en algún sitio—.
--
-- Así que la columna es `NULL`able: `NULL` es «no lo sé y no lo adivino». La
-- pantalla propone el valor deducido **como texto editable**, que es distinto
-- de que el código lo dé por bueno.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ NACE APAGADO
--
-- `inbox_scan_enabled` por defecto `0`. Leer el buzón de alguien es lo que
-- Dirección autorizó el 10/11/2026 **para su cuenta**, no una opción que ARLES
-- active por su cuenta en la cuenta de nadie. Una actualización de ARLES que
-- empezara a leer buzones sin que nadie lo pidiera sería exactamente lo que
-- ningún programa debería hacer.
--
-- ────────────────────────────────────────────────────────────────────────
-- Y POR QUÉ HACE FALTA GUARDAR EL ÚLTIMO UID
--
-- El lector busca por fecha y no por «sin leer», porque `EXAMINE` no marca nada
-- como leído (es la garantía del tramo 2) y un correo seguiría saliendo en
-- `UNSEEN` para siempre. La fecha acota, pero no impide volver a mirar los
-- mismos correos en la pasada siguiente.
--
-- `inbox_last_uid` es lo que cierra eso: el UID más alto que ya se procesó. No
-- es una optimización — sin él, **cada pasada volvería a registrar en la
-- bitácora las mismas bajas**, y el registro de por qué alguien está en la
-- lista de supresión se llenaría de copias del mismo correo.
--
-- Los UID de IMAP crecen dentro de un buzón y no se reutilizan (RFC 3501
-- §2.3.1.1), así que «mayor que el último» es una pregunta que se puede hacer.
-- Si el servidor reiniciara su `UIDVALIDITY` —se borra y se recrea la carpeta—
-- los UID podrían volver a empezar: entonces lo que pasa es que se vuelven a
-- mirar correos ya vistos, y las bajas ya aplicadas no cambian nada porque la
-- supresión es idempotente por (empresa, canal, dirección). **Cae del lado
-- seguro.**

ALTER TABLE email_account ADD COLUMN imap_host TEXT;

-- Apagado, y es una decisión. Ver arriba.
ALTER TABLE email_account ADD COLUMN inbox_scan_enabled INTEGER NOT NULL DEFAULT 0
    CHECK (inbox_scan_enabled IN (0, 1));

-- `NULL` es «nunca se ha leído este buzón», que no es lo mismo que 0 — el UID 0
-- no existe en IMAP, pero un 0 guardado se leería como «ya se procesó algo» y
-- aquí la diferencia decide si la primera pasada mira los correos de los
-- últimos días o ninguno.
ALTER TABLE email_account ADD COLUMN inbox_last_uid INTEGER
    CHECK (inbox_last_uid IS NULL OR inbox_last_uid > 0);
