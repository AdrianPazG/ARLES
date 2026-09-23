-- ARLES RELAY I · v1.2.0 · los datos de conexión de la cuenta de correo
--
-- Entrega 5.3.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- LA TABLA EXISTÍA DESDE LA V1 Y NO SE PODÍA CONECTAR A NADA
--
-- `email_account` se creó con el esquema inicial: nombre visible, dirección
-- del remitente, tipo de proveedor, referencia al llavero, límites y las
-- columnas del disyuntor. Todo menos lo único que hace falta para conectarse a
-- un servidor SMTP: **el servidor, el puerto, el cifrado y el usuario**.
--
-- No es una casualidad. Cuando se escribió la V1, la conexión de verdad estaba
-- a cuatro fases de distancia y el esquema se diseñó contra un documento, no
-- contra un adaptador funcionando. Es el mismo patrón que dejó las columnas del
-- disyuntor sin que nadie las escribiera durante cuatro entregas: un esquema
-- que promete algo y un código que no lo cumple no falla — simplemente no hace
-- nada.
--
-- Aquí pasa al revés: el adaptador de la 5.2 ya funciona y pide estos cuatro
-- datos, así que las columnas nacen con quien las usa.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- EL §29 ESCRITO COMO ESQUEMA
--
-- `smtp_security` sólo admite dos valores, y **los dos cifran**. No hay un
-- 'none', ni un 'plain', ni un 'opportunistic'.
--
-- Esto no es redundante con el código: es la tercera vez que la misma regla se
-- escribe, y a propósito. Está en `arles_core::correo::CifradoDeCorreo`, que no
-- tiene una variante sin cifrar; está en `arles_email::tls`, que no puede
-- construir parámetros sin cifrado; y está aquí. Tres sitios para una regla
-- parece mucho hasta que se piensa en el día en que alguien «arregle» uno de
-- los tres para hacer funcionar un servidor mal configurado. Con esta columna,
-- el valor no cabe en la base aunque el código lo dejara pasar.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- POR QUÉ LAS COLUMNAS SON OPCIONALES
--
-- `email_account` guarda también las cuentas de Google y de Microsoft, que
-- llegan en v1.2.x y v1.3 (D-1) y **no tienen servidor ni contraseña**: se
-- conectan por OAuth. Una columna `NOT NULL` obligaría a inventarse un
-- `smtp.google.com` falso para esas filas, y ese dato falso acabaría en algún
-- sitio pareciendo verdadero.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- LO QUE SE INTENTÓ PRIMERO, Y POR QUÉ NO SE PUEDE
--
-- La primera versión de esta migración obligaba a las cuentas SMTP a traer los
-- cuatro datos, con un `CHECK (provider_kind <> 'smtp' OR smtp_host IS NOT
-- NULL)` en cada columna. Era mejor: la regla vivía en el esquema.
--
-- **No se puede.** La prueba que aplica las migraciones sobre datos sembrados
-- falló, y al comprobarlo salió el motivo: SQLite **revalida las filas que ya
-- existen** al añadir una columna con `CHECK`. Una base con una cuenta SMTP ya
-- guardada —que por definición no tiene estas columnas todavía— no puede
-- migrar: el `ALTER TABLE` falla y ARLES no arranca.
--
-- Rellenar esas filas con un servidor inventado para que pasen el `CHECK` sería
-- lo fácil y es justo lo que no se hace: un `smtp.gmail.com` puesto por una
-- migración es indistinguible de uno que escribió el usuario, y el primer envío
-- iría a un servidor que nadie eligió.
--
-- Así que la obligatoriedad se queda en el código —el repositorio escribe los
-- cuatro datos juntos o ninguno, y al leer se revalidan— y aquí se queda lo que
-- el esquema sí puede garantizar sin romper nada: **el dominio de los valores**.
-- Que es, además, la parte que importa: un `CHECK` de rango no impide migrar,
-- porque las filas viejas traen NULL y NULL pasa.
--
-- ═══════════════════════════════════════════════════════════════════════════
-- LAS CUENTAS VIEJAS NO SE QUEDAN CALLADAS
--
-- Una cuenta SMTP de antes de esta migración no tiene servidor, así que no
-- puede enviar. Dejarla con `status = 'connected'` sería mentir: la pantalla la
-- enseñaría como lista y el fallo aparecería al arrancar una campaña.
--
-- El `UPDATE` del final las marca como `needs_reauth`, que es el estado que ya
-- existía desde la V1 para «hay que volver a configurarla». La cuenta no se
-- pierde, no se inventa nada, y quien la abra ve que le falta algo.

ALTER TABLE email_account ADD COLUMN smtp_host TEXT;

-- El puerto, con su rango. El 0 no es un puerto y por encima de 65535 no
-- existe: sin esto, un 0 guardado daría un «no se llegó al servidor» eterno y
-- nadie miraría la columna.
ALTER TABLE email_account ADD COLUMN smtp_port INTEGER
    CHECK (smtp_port IS NULL OR (smtp_port > 0 AND smtp_port <= 65535));

-- LOS DOS VALORES QUE CIFRAN. NO HAY UN TERCERO.
--
-- Es el §29 escrito como esquema, y la parte de esta migración que de verdad
-- protege algo: aunque un día el código dejara pasar un 'none', aquí no cabe.
ALTER TABLE email_account ADD COLUMN smtp_security TEXT
    CHECK (smtp_security IS NULL OR smtp_security IN ('starttls', 'directo'));

ALTER TABLE email_account ADD COLUMN smtp_username TEXT;

-- Las cuentas SMTP que venían de antes no pueden enviar: lo dicen.
UPDATE email_account
   SET status = 'needs_reauth',
       updated_at = updated_at
 WHERE provider_kind = 'smtp'
   AND smtp_host IS NULL
   AND status <> 'disabled';
