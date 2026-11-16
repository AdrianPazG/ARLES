-- ════════════════════════════════════════════════════════════════════════
-- V20 · Una baja puede venir de quien recibe el WhatsApp
-- ════════════════════════════════════════════════════════════════════════
--
-- Petición de Dirección del 11/11/2026: «también estuvimos trabajando para
-- que la baja en whatsapp sea automatica tambien, eso hay que solucionarlo».
--
-- Es la gemela de la V17, tres días después y para el otro canal. Y la mitad
-- del trabajo estaba hecha: el núcleo ya tenía `Fallo::EsaPersonaPidioLaBaja`
-- —«no es un fallo: es el sistema funcionando»—, el motor ya suprime al
-- recibirlo, y `entrante.rs` ya sabe verificar un aviso de Meta. Lo que no
-- había era **la señal**.
--
-- ────────────────────────────────────────────────────────────────────────
-- DE DÓNDE SALE LA SEÑAL, Y POR QUÉ NO HACE FALTA UN SERVIDOR
--
-- WhatsApp enseña a quien recibe un mensaje de empresa un control para dejar
-- de recibir promociones. Cuando alguien lo usa, Meta **no nos avisa**: eso
-- iría por el webhook `user_preferences`, y ARLES no tiene ninguna URL
-- pública a la que Meta pueda llamar (`arles-whatsapp/src/entrante.rs` lo
-- tiene escrito desde el 29/09/2026).
--
-- Pero **sí contesta `131050` al siguiente envío**, y eso es una respuesta que
-- ARLES ya recibe. Verificado en la documentación de Meta, no de memoria:
-- «Recipient stopped receiving marketing messages from your business. Don't
-- retry.»
--
-- Así que la baja automática por WhatsApp no necesitaba el relay ciego. Lo que
-- necesitaba era **leer un código que caía en el cajón de lo desconocido**:
-- hasta hoy, `131050` se trataba como un fallo pasajero, se reintentaba tres
-- veces y se rendía — y la persona que había pedido no recibir **seguía en la
-- lista** y volvía a recibir en la campaña siguiente.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ UN ORIGEN NUEVO Y NO `whatsapp_webhook`, QUE YA EXISTE
--
-- Porque sería falso, y de la misma forma que la V17 explicó para `user`.
-- `whatsapp_webhook` significa «lo avisó Meta por webhook», y esto no viene de
-- ningún webhook: viene de la respuesta de un envío. Pero sobre todo, el
-- origen no dice **cómo nos enteramos**, dice **quién lo decidió** —así está
-- escrito en el propio enum— y aquí lo decidió la persona.
--
-- La simetría es la que importa: `email_unsubscribe` y `whatsapp_optout` son
-- la misma decisión tomada en los dos canales, y el día que haya que defender
-- por qué un número está en la lista, «lo pidió ella» se sostiene y «nos lo
-- avisó Meta» obliga a ir a buscar qué significaba eso.
--
-- ────────────────────────────────────────────────────────────────────────
-- Y EL SÉPTIMO ORIGEN ARREGLA UN DEFECTO QUE YA ESTABA
--
-- Medido al llegar aquí, y no es de esta petición: el motor escribe el origen
-- **fijo en `smtp_5xx`** en los dos sitios donde suprime. Así que desde que el
-- motor recorre WhatsApp —D-WA, 26/10/2026— un `131026` de Meta guarda la baja
-- de un número de móvil diciendo que **lo dijo un servidor de correo**.
--
-- Nada falla por eso: la fila entra, el número deja de recibir y la protección
-- funciona. Lo que queda mal es justo aquello para lo que existe el campo —
-- poder defender de dónde salió cada baja—, y es el mismo defecto que las dos
-- secciones de arriba argumentan para no cometer. Habría sido incoherente
-- arreglar la mitad nueva y dejar la vieja, así que el origen sale ahora **del
-- canal** y no de una constante.
--
-- `whatsapp_api` es el gemelo de `smtp_5xx`: lo dijo el proveedor en la
-- respuesta del envío, hablando de ese destinatario.
--
-- ────────────────────────────────────────────────────────────────────────
-- POR QUÉ SE RECREA LA TABLA, Y POR QUÉ AQUÍ SÍ SE PUEDE
--
-- Las mismas dos razones de la V17, y siguen siendo verdad medidas hoy:
-- SQLite no sabe ampliar un `CHECK`, y `suppression_entry` **no es padre de
-- nadie** —ninguna tabla del esquema la nombra en un `REFERENCES`—, así que
-- recrearla no dispara el fallo F1. Es hoja, con una clave ajena hacia
-- `company` en el otro sentido, que se vuelve a declarar abajo.
--
-- Aun así es la tabla más delicada del producto, y el `INSERT … SELECT` copia
-- **columna por columna y por nombre**, nunca por posición.
--
-- ────────────────────────────────────────────────────────────────────────
-- LO QUE ESTA MIGRACIÓN CUESTA
--
-- Nada que no se pueda deshacer: desde el 04/11/2026 un respaldo de un esquema
-- anterior se pone al día antes de restaurarse, así que un `.arles` tomado con
-- la V19 sigue entrando.

CREATE TABLE suppression_entry_nueva (
    id                 TEXT PRIMARY KEY NOT NULL,
    company_id         TEXT NOT NULL REFERENCES company(id) ON DELETE CASCADE,
    channel            TEXT NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    address_normalized TEXT NOT NULL,
    -- 'channel': pidió no recibir por este medio.
    -- 'global':  pidió no recibir por ninguno; esta fila es una de varias.
    scope              TEXT NOT NULL DEFAULT 'channel'
                       CHECK (scope IN ('channel', 'global')),
    -- Une las filas que salieron de la misma petición. Sin esto, deshacer una
    -- petición global obligaría a adivinar cuáles eran sus filas.
    request_id         TEXT,
    reason             TEXT NOT NULL
                       CHECK (reason IN ('hard_bounce', 'unsubscribe', 'manual',
                                         'complaint', 'invalid_number', 'channel_failure')),
    -- Los dos nuevos de esta migración completan los dos pares simétricos:
    --   'smtp_5xx'          ← lo dijo el servidor de correo
    --   'whatsapp_api'      ← lo dijo la API de WhatsApp (131026: no recibe)
    --   'email_unsubscribe' ← lo pidió quien recibe el correo
    --   'whatsapp_optout'   ← lo pidió quien recibe el WhatsApp (131050)
    origin             TEXT NOT NULL
                       CHECK (origin IN ('smtp_5xx', 'user', 'import',
                                         'whatsapp_webhook', 'email_unsubscribe',
                                         'whatsapp_optout', 'whatsapp_api')),
    notes              TEXT,
    created_at         TEXT NOT NULL,
    created_by         TEXT
) STRICT;

-- Por nombre de columna y no por posición: una migración futura que mueva una
-- columna metería los campos corridos sin que una sola fila fallara.
INSERT INTO suppression_entry_nueva (
    id, company_id, channel, address_normalized, scope, request_id,
    reason, origin, notes, created_at, created_by
)
SELECT id, company_id, channel, address_normalized, scope, request_id,
       reason, origin, notes, created_at, created_by
FROM suppression_entry;

DROP TABLE suppression_entry;
ALTER TABLE suppression_entry_nueva RENAME TO suppression_entry;

-- Los índices se van con la tabla y hay que volver a crearlos. Son los mismos
-- de la V3 y la V17: el único que impide duplicar una baja, y el que agrupa
-- las filas de una petición global.
CREATE UNIQUE INDEX idx_suppression_unique
    ON suppression_entry(company_id, channel, address_normalized);

CREATE INDEX idx_suppression_peticion
    ON suppression_entry(request_id)
    WHERE request_id IS NOT NULL;
