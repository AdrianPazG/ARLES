//! Migraciones de esquema.
//!
//! SQL versionado y embebido en el binario (ADR-0002). Reglas:
//!
//! - **Nunca se edita una migración ya publicada.** Hay bases de clientes con
//!   ella aplicada; cambiarla produce divergencias silenciosas.
//! - Toda migración se prueba con una base **poblada**, no vacía.
//! - Antes de migrar, la aplicación hace una copia de la base: una migración
//!   fallida sobre los datos de un cliente sin copia previa es una pérdida
//!   irrecuperable.

use rusqlite::Connection;

use crate::error::DbError;

mod embebidas {
    refinery::embed_migrations!("migrations");
}

/// La última versión de esquema que este binario sabe aplicar.
///
/// Se deriva de las migraciones embebidas en vez de escribirse a mano. El
/// número estaba puesto a mano en dos pruebas, y al añadir la `V2` una de
/// ellas —la del arranque— siguió esperando `Some(1)`: no falló en Linux
/// porque esa rama sólo corre donde hay llavero, así que el fallo apareció en
/// Windows y macOS, en CI, después de dar el trabajo por bueno.
#[must_use]
pub fn ultima_version() -> Option<i32> {
    embebidas::migrations::runner()
        .get_migrations()
        .iter()
        // `refinery` ya devuelve i32 aquí; no hay conversión que hacer.
        .map(refinery::Migration::version)
        .max()
}

/// Pone el esquema al día.
///
/// # Errores
///
/// [`DbError::Migracion`] con el detalle del fallo.
pub fn aplicar(conn: &mut Connection) -> Result<(), DbError> {
    embebidas::migrations::runner()
        .run(conn)
        .map(|_| ())
        .map_err(|e| DbError::Migracion(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::{ClaveMaestra, abrir};

    fn base_de_prueba() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let conn = abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, conn)
    }

    fn tablas(conn: &Connection) -> Vec<String> {
        let mut s = conn
            .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
            .expect("prepara");
        s.query_map([], |f| f.get::<_, String>(0))
            .expect("consulta")
            .filter_map(Result::ok)
            .collect()
    }

    #[test]
    fn la_ultima_version_sale_de_las_migraciones_embebidas() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let archivos = std::fs::read_dir(dir)
            .expect("hay migraciones")
            .filter_map(Result::ok)
            .count();
        assert_eq!(
            ultima_version(),
            Some(i32::try_from(archivos).expect("caben")),
            "la última versión debe corresponder al número de migraciones"
        );
    }

    /// `refinery` guarda en cada base una huella del texto SQL de cada
    /// migración y, al abrirla, **se niega a arrancar** si no coincide con la
    /// del binario (`abort_divergent`). Un retorno de carro cambia la huella.
    ///
    /// Git para Windows convierte a CRLF al sacar los archivos, y así lo hace
    /// también `windows-latest` en CI: los instaladores de Windows llevaban
    /// una huella y los de macOS otra, para el mismo SQL. El validador no lo
    /// veía porque corre en Ubuntu. Lo fija `.gitattributes` (`eol=lf`).
    ///
    /// Vista fallar: en Windows, con los archivos sacados antes de añadir la
    /// regla a `.gitattributes`, falla nombrando las cinco migraciones.
    #[test]
    fn las_migraciones_embebidas_no_llevan_retornos_de_carro() {
        let con_cr: Vec<String> = embebidas::migrations::runner()
            .get_migrations()
            .iter()
            .filter(|m| m.sql().is_some_and(|s| s.contains('\r')))
            .map(ToString::to_string)
            .collect();
        assert!(
            con_cr.is_empty(),
            "migraciones con CRLF (su huella cambia según dónde se compile): {con_cr:?}"
        );
    }

    #[test]
    fn crea_las_tablas_del_modelo_de_datos() {
        let (_d, conn) = base_de_prueba();
        let t = tablas(&conn);
        for esperada in [
            "company",
            "email_account",
            "campaign_stage",
            "whatsapp_account",
            "whatsapp_quality_event",
            "contact",
            "contact_channel",
            "consent_entry",
            "contact_field",
            "contact_list",
            "contact_list_member",
            "tag",
            "contact_tag",
            "import_batch",
            "suppression_entry",
            "template",
            "template_version",
            "signature",
            "execution_window",
            "campaign",
            "campaign_audience",
            "message_attempt",
            "rate_budget",
            "audit_log",
            "ui_preference",
        ] {
            assert!(
                t.iter().any(|x| x == esperada),
                "falta la tabla {esperada}; hay: {t:?}"
            );
        }
    }

    /// La regla del módulo: toda migración se prueba con una base **poblada**,
    /// no vacía. Una migración que solo se ha visto correr sobre una base
    /// recién creada no se ha visto correr sobre la de nadie.
    #[test]
    fn migrar_una_base_con_datos_no_los_toca() {
        let (_d, mut conn) = base_de_prueba();
        conn.execute(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-15T00:00:00Z', '2026-09-15T00:00:00Z')",
            [],
        )
        .expect("inserta");

        aplicar(&mut conn).expect("vuelve a migrar");

        let nombre: String = conn
            .query_row("SELECT commercial_name FROM company", [], |f| f.get(0))
            .expect("sigue ahí");
        assert_eq!(nombre, "TELEMETRY");
    }

    /// **La V8 sobre una base que ya tiene etapas.**
    ///
    /// Es la prueba que el primer borrador de la V7 no pasaba, y por eso se
    /// escribe igual aquí: SQLite **revalida todas las filas existentes** al
    /// añadir una columna con `CHECK`, así que una restricción en un `ALTER
    /// TABLE … ADD COLUMN` puede hacer la migración imposible para cualquiera
    /// que ya tenga datos — y no para quien la prueba sobre una base recién
    /// creada, donde no hay filas que revalidar.
    ///
    /// Aquí se para el runner en la V7, se puebla con una campaña y su etapa, y
    /// sólo entonces se aplica la V8. Lo que se exige es lo único que le
    /// importa a quien ya tiene datos: que la migración pase y que la etapa
    /// siga ahí, sin prueba de envío, que es la verdad — nunca se hizo una.
    ///
    /// Verificada rompiéndola el 28/09/2026: añadiendo
    /// `CHECK (test_sent_at IS NOT NULL)` a la primera columna de la V8, esta
    /// prueba falla y `migrar_una_base_con_datos_no_los_toca` sigue pasando,
    /// porque aquélla no crea ninguna etapa.
    #[test]
    fn la_v8_migra_una_base_que_ya_tiene_etapas() {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn = crate::conexion::abrir_crudo(&dir.path().join("vieja.db"), &clave)
            .expect("abre sin migrar");

        embebidas::migrations::runner()
            .set_target(refinery::Target::Version(7))
            .run(&mut conn)
            .expect("migra hasta la V7");

        conn.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at,
                                        smtp_host, smtp_port, smtp_security, smtp_username)
             VALUES ('a1', 'c1', 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://v',
                     'connected', 50, 10, '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z',
                     'smtp.gmail.com', 587, 'starttls', 'ventas@t.mx');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('w1', 'c1', 'Laborables', 31, 9, 18, 'America/Mexico_City',
                     '2026-09-28T00:00:00Z');

             INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('k1', 'c1', 'Una campaña de antes', 'running',
                     '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z');

             INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                         email_account_id, execution_window_id,
                                         status, created_at, updated_at)
             VALUES ('e1', 'k1', 1, 'email', 'a1', 'w1', 'running',
                     '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z');",
        )
        .expect("puebla como estaba en la V7");

        aplicar(&mut conn).expect("la V8 tiene que poder correr sobre una base con etapas");

        let (id, probada): (String, Option<String>) = conn
            .query_row(
                "SELECT id, test_sent_at FROM campaign_stage WHERE id = 'e1'",
                [],
                |f| Ok((f.get(0)?, f.get(1)?)),
            )
            .expect("la etapa sigue ahí");
        assert_eq!(id, "e1");
        assert_eq!(
            probada, None,
            "una etapa que existía antes de la V8 no ha hecho ningún envío de \
             prueba, y el esquema no puede decir lo contrario"
        );
    }

    /// **La prueba que de verdad ejerce la V3.**
    ///
    /// Sobre una base recién creada, el traslado de datos de la V3 no mueve
    /// nada: `contact` está vacío, así que el `INSERT … SELECT` no copia ni una
    /// fila y la migración parece correcta sin haberse ejecutado sobre nada.
    ///
    /// Aquí se para el runner **en la V2**, se puebla la base como estaba antes
    /// —con el correo dentro de `contact`— y sólo entonces se aplica la V3. Es
    /// la única forma de comprobar lo único que le importa a quien ya tiene
    /// datos: que no los pierde.
    #[test]
    fn la_v3_traslada_los_correos_existentes_a_sus_canales() {
        // `base_de_prueba` migra del todo, así que aquí se abre a mano y se
        // para el runner en la V2.
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn = crate::conexion::abrir_crudo(&dir.path().join("vieja.db"), &clave)
            .expect("abre sin migrar");

        embebidas::migrations::runner()
            .set_target(refinery::Target::Version(2))
            .run(&mut conn)
            .expect("migra hasta la V2");

        conn.execute(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-16T00:00:00Z', '2026-09-16T00:00:00Z')",
            [],
        )
        .expect("inserta empresa");

        // Dos contactos como los guardaba la V1: la dirección dentro de la
        // propia fila del contacto. Uno de ellos, archivado.
        conn.execute(
            "INSERT INTO contact (id, company_id, email_raw, email_normalized,
                                  status, created_at, updated_at)
             VALUES ('k1', 'c1', 'Ana@Empresa.com', 'ana@empresa.com', 'active',
                     '2026-09-16T00:00:00Z', '2026-09-16T00:00:00Z'),
                    ('k2', 'c1', 'beto@empresa.com', 'beto@empresa.com', 'archived',
                     '2026-09-16T00:00:00Z', '2026-09-16T00:00:00Z')",
            [],
        )
        .expect("inserta contactos");

        conn.execute(
            "INSERT INTO suppression_entry (id, company_id, email_normalized,
                                            reason, origin, created_at)
             VALUES ('s1', 'c1', 'beto@empresa.com', 'unsubscribe', 'user',
                     '2026-09-16T00:00:00Z')",
            [],
        )
        .expect("inserta supresión");

        aplicar(&mut conn).expect("aplica la V3 sobre datos reales");

        // Los dos contactos siguen ahí, y cada uno tiene su canal de correo.
        let contactos: i64 = conn
            .query_row("SELECT count(*) FROM contact", [], |f| f.get(0))
            .expect("cuenta");
        assert_eq!(contactos, 2, "se perdieron contactos al migrar");

        let (canales, crudo, normalizado): (i64, String, String) = conn
            .query_row(
                "SELECT count(*), max(value_raw), max(value_normalized)
                 FROM contact_channel WHERE channel = 'email' AND contact_id = 'k1'",
                [],
                |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
            )
            .expect("consulta");
        assert_eq!(canales, 1, "el correo de k1 no llegó a contact_channel");
        assert_eq!(
            crudo, "Ana@Empresa.com",
            "se perdió lo que escribió el usuario"
        );
        assert_eq!(
            normalizado, "ana@empresa.com",
            "se perdió la forma normalizada"
        );

        // El archivado llega archivado: si llegara activo, una campaña volvería
        // a escribirle a alguien que se había dado de baja de la lista.
        let estado: String = conn
            .query_row(
                "SELECT status FROM contact_channel WHERE contact_id = 'k2'",
                [],
                |f| f.get(0),
            )
            .expect("consulta");
        assert_eq!(estado, "archived");

        // La supresión sobrevive y ahora dice por qué canal era.
        let (canal, direccion, alcance): (String, String, String) = conn
            .query_row(
                "SELECT channel, address_normalized, scope FROM suppression_entry",
                [],
                |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
            )
            .expect("la supresión sobrevive");
        assert_eq!(canal, "email");
        assert_eq!(direccion, "beto@empresa.com");
        assert_eq!(alcance, "channel");

        // Y el correo ya no vive en `contact`: tenerlo en dos sitios es cómo
        // acaban diciendo cosas distintas.
        let mut consulta = conn
            .prepare("SELECT name FROM pragma_table_info('contact')")
            .expect("prepara");
        let columnas: Vec<String> = consulta
            .query_map([], |f| f.get::<_, String>(0))
            .expect("consulta")
            .filter_map(Result::ok)
            .collect();
        assert!(
            !columnas.iter().any(|c| c.starts_with("email")),
            "`contact` conserva la dirección: {columnas:?}"
        );
    }

    /// **El §29 escrito como esquema: no cabe un cifrado que no cifre.**
    ///
    /// Es la garantía que justifica la V7. La misma regla está en el núcleo
    /// —`CifradoDeCorreo` no tiene una tercera variante— y en el adaptador, que
    /// no sabe construir una conexión en claro. Ésta es la tercera, y es la
    /// única que sigue valiendo si un día el código deja pasar el valor: aunque
    /// lo dejara, no entra en la tabla.
    ///
    /// Verificada rompiéndola el 23/09/2026: añadiendo `'none'` a la lista del
    /// `CHECK`, el primer `INSERT` pasa y esta prueba falla.
    #[test]
    fn la_v7_no_admite_una_conexion_sin_cifrar() {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn = crate::conexion::abrir_crudo(&dir.path().join("v7.db"), &clave)
            .expect("abre sin migrar");
        aplicar(&mut conn).expect("migra hasta la última");

        conn.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z');",
        )
        .expect("una empresa");

        let alta = |id: &str, cifrado: &str| -> Result<usize, rusqlite::Error> {
            conn.execute(
                "INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, daily_limit,
                                            hourly_limit, created_at, updated_at,
                                            smtp_host, smtp_port, smtp_security,
                                            smtp_username)
                 VALUES (?1, 'c1', 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://x',
                         50, 10, '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z',
                         'smtp.gmail.com', 587, ?2, 'ventas@t.mx')",
                rusqlite::params![id, cifrado],
            )
        };

        // Las formas en las que alguien intentaría apagar el cifrado.
        for (i, intento) in ["none", "ninguno", "plain", "opportunistic", "", "NONE"]
            .into_iter()
            .enumerate()
        {
            assert!(
                alta(&format!("malo{i}"), intento).is_err(),
                "«{intento}» no puede caber en la base: es una conexión sin cifrar"
            );
        }

        // Y los dos que sí valen entran.
        alta("bueno1", "starttls").expect("STARTTLS entra");
        alta("bueno2", "directo").expect("TLS directo entra");
    }

    /// Un puerto imposible no entra.
    ///
    /// Sin esto, un `0` guardado daría un «no se llegó al servidor» eterno, y
    /// el sitio donde mirar sería una columna que nadie abre.
    #[test]
    fn la_v7_rechaza_los_puertos_que_no_existen() {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn = crate::conexion::abrir_crudo(&dir.path().join("v7p.db"), &clave)
            .expect("abre sin migrar");
        aplicar(&mut conn).expect("migra hasta la última");

        conn.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z');",
        )
        .expect("una empresa");

        let alta = |id: &str, puerto: i64| -> Result<usize, rusqlite::Error> {
            conn.execute(
                "INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, daily_limit,
                                            hourly_limit, created_at, updated_at,
                                            smtp_host, smtp_port, smtp_security,
                                            smtp_username)
                 VALUES (?1, 'c1', 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://x',
                         50, 10, '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z',
                         'smtp.gmail.com', ?2, 'starttls', 'ventas@t.mx')",
                rusqlite::params![id, puerto],
            )
        };

        assert!(alta("p0", 0).is_err(), "el 0 no es un puerto");
        assert!(alta("pneg", -1).is_err(), "un puerto negativo no existe");
        assert!(
            alta("pmax", 65_536).is_err(),
            "por encima de 65535 no existe"
        );
        alta("pok", 587).expect("el 587 sí");
        alta("pok2", 65_535).expect("el último puerto también");
    }

    /// **Una cuenta SMTP de antes de la V7 no se queda diciendo que funciona.**
    ///
    /// No tiene servidor, así que no puede enviar. Dejarla como `connected`
    /// haría que la pantalla la enseñara lista y que el fallo apareciera al
    /// arrancar una campaña, con los contactos ya en cola.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando el `UPDATE` del final de
    /// la V7, la cuenta sigue en `connected` y esta prueba falla.
    #[test]
    fn la_v7_marca_las_cuentas_smtp_que_venian_sin_servidor() {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn = crate::conexion::abrir_crudo(&dir.path().join("v7v.db"), &clave)
            .expect("abre sin migrar");

        embebidas::migrations::runner()
            .set_target(refinery::Target::Version(6))
            .run(&mut conn)
            .expect("migra hasta la V6");

        conn.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('vieja', 'c1', 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://v',
                     'connected', 50, 10, '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');",
        )
        .expect("una cuenta como las de antes");

        aplicar(&mut conn).expect("la V7 se aplica sobre una base con cuentas");

        let estado: String = conn
            .query_row(
                "SELECT status FROM email_account WHERE id = 'vieja'",
                [],
                |f| f.get(0),
            )
            .expect("la cuenta sigue ahí");
        assert_eq!(
            estado, "needs_reauth",
            "una cuenta sin servidor no puede seguir diciendo que está conectada"
        );
    }

    /// La V4 sobre datos reales: una campaña con su cuenta, su plantilla y sus
    /// intentos tiene que salir convertida en una campaña **con una etapa de
    /// correo**, sin perder nada y sin dejar los intentos huérfanos.
    ///
    /// Igual que con la V3, sobre una base recién creada el traslado no mueve
    /// ni una fila.
    #[test]
    fn la_v4_convierte_las_campanas_existentes_en_campanas_con_una_etapa() {
        let dir = tempfile::tempdir().expect("directorio temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn = crate::conexion::abrir_crudo(&dir.path().join("vieja.db"), &clave)
            .expect("abre sin migrar");

        embebidas::migrations::runner()
            .set_target(refinery::Target::Version(3))
            .run(&mut conn)
            .expect("migra hasta la V3");

        conn.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, daily_limit,
                                        hourly_limit, created_at, updated_at)
             VALUES ('ea1', 'c1', 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://ea1',
                     50, 10, '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');

             INSERT INTO campaign (id, company_id, name, status, email_account_id,
                                   daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('cam1', 'c1', 'Clientes Q1', 'running', 'ea1', 40, 8,
                     '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');

             -- Un borrador sin remitente: no puede producir etapa, y tampoco
             -- puede perderse.
             INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('cam2', 'c1', 'Sin remitente aún', 'draft',
                     '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');

             INSERT INTO contact (id, company_id, created_at, updated_at)
             VALUES ('k1', 'c1', '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');

             INSERT INTO message_attempt (id, campaign_id, channel, contact_id,
                                          contact_address, email_account_id,
                                          idempotency_key, state, created_at, updated_at)
             VALUES ('i1', 'cam1', 'email', 'k1', 'ana@empresa.com', 'ea1', 'k-1',
                     'sent', '2026-09-17T00:00:00Z', '2026-09-17T00:00:00Z');

             INSERT INTO campaign_audience (campaign_id, channel, contact_address,
                                            contact_id, added_at)
             VALUES ('cam1', 'email', 'ana@empresa.com', 'k1',
                     '2026-09-17T00:00:00Z');

             INSERT INTO rate_budget (email_account_id, window_kind, window_start,
                                      consumed, updated_at)
             VALUES ('ea1', 'daily', '2026-09-17T00:00:00Z', 7,
                     '2026-09-17T00:00:00Z');",
        )
        .expect("puebla como la V3");

        aplicar(&mut conn).expect("aplica la V4 sobre datos reales");

        // Las dos campañas siguen ahí.
        let campanas: i64 = conn
            .query_row("SELECT count(*) FROM campaign", [], |f| f.get(0))
            .expect("cuenta");
        assert_eq!(campanas, 2, "se perdieron campañas al migrar");

        // La que tenía remitente tiene ahora una etapa de correo con todo lo
        // que antes colgaba de la campaña.
        let (id, posicion, canal, cuenta, diario, estado): (
            String,
            i64,
            String,
            String,
            i64,
            String,
        ) = conn
            .query_row(
                "SELECT id, position, channel, email_account_id, daily_limit, status
                   FROM campaign_stage WHERE campaign_id = 'cam1'",
                [],
                |f| {
                    Ok((
                        f.get(0)?,
                        f.get(1)?,
                        f.get(2)?,
                        f.get(3)?,
                        f.get(4)?,
                        f.get(5)?,
                    ))
                },
            )
            .expect("la campaña con remitente tiene etapa");
        assert_eq!(posicion, 1);
        assert_eq!(canal, "email");
        assert_eq!(cuenta, "ea1");
        assert_eq!(diario, 40, "el ritmo tenía que bajar a la etapa");
        assert_eq!(estado, "running", "la etapa hereda el estado de la campaña");

        // El borrador sin remitente no produce etapa —no pasaría el CHECK— pero
        // tampoco se pierde.
        let sin_etapa: i64 = conn
            .query_row(
                "SELECT count(*) FROM campaign_stage WHERE campaign_id = 'cam2'",
                [],
                |f| f.get(0),
            )
            .expect("cuenta");
        assert_eq!(sin_etapa, 0);

        // ── Lo que la primera versión de la V4 borraba en silencio ──
        //
        // Reconstruía `campaign` con DROP + RENAME. Con las claves foráneas
        // activas, soltar una tabla PADRE ejecuta un borrado implícito que
        // **cascadea**: la audiencia congelada y el registro de envíos cuelgan
        // de `campaign` con ON DELETE CASCADE. La migración terminaba sin un
        // solo error, con la instantánea de a quién se le escribió y la prueba
        // de que se le escribió en cero filas.
        let audiencia: i64 = conn
            .query_row("SELECT count(*) FROM campaign_audience", [], |f| f.get(0))
            .expect("cuenta");
        assert_eq!(
            audiencia, 1,
            "la audiencia congelada desapareció al migrar: «¿a quién le llegó \
             esto?» se queda sin respuesta para todo lo ya enviado"
        );

        let intentos: i64 = conn
            .query_row("SELECT count(*) FROM message_attempt", [], |f| f.get(0))
            .expect("cuenta");
        assert_eq!(intentos, 1, "se perdió el registro de envíos al migrar");

        // El intento queda enganchado a esa etapa, no huérfano.
        let etapa_del_intento: String = conn
            .query_row(
                "SELECT stage_id FROM message_attempt WHERE id = 'i1'",
                [],
                |f| f.get(0),
            )
            .expect("el intento conserva su etapa");
        assert_eq!(etapa_del_intento, id);

        // ── EL CUBO DE RITMO, Y POR QUÉ ESTA PRUEBA CAMBIÓ EL 22/10/2026 ──
        //
        // Hasta la V13, aquí se comprobaba que la V4 **conserva lo consumido**
        // —siete envíos antes de migrar, siete después— con este motivo
        // escrito: «se reinició el cubo al migrar: la campaña enviaría de más
        // hoy». Y era correcto.
        //
        // La V13 vacía `rate_budget` a propósito, y por eso esta prueba salió
        // en rojo al escribirla: los cubos viejos cuelgan del BUZÓN y no se
        // pueden repartir entre las campañas que lo usan. Un contador de 37 de
        // una cuenta que sirve a tres campañas no sabe cuántos puso cada una, y
        // repartirlo a ojo dejaría campañas con un cupo gastado que nunca
        // gastaron.
        //
        // Así que lo que esta prueba afirma ahora son dos cosas distintas, y
        // las dos hacen falta:
        //
        //   · que la V4 hace su trabajo —el canal, la etapa, la audiencia y el
        //     registro de envíos siguen ahí—, que es de lo que esta prueba va;
        //   · y que el cubo queda vacío **por la V13**, no por un descuido.
        //
        // El precio está declarado en la cabecera de la V13: quien actualice a
        // media tarde habiendo gastado parte del cupo de hoy puede mandar más
        // de lo previsto ESE día. Una vez, y se dice en vez de callarlo.
        // ── EL CUPO GASTADO SE HEREDA, NO SE PIERDE ──────────────────
        //
        // La primera versión de la V13, del mismo día, los vaciaba — y eso
        // hacía que quien actualizara a media tarde mandara DE MÁS ese día, que
        // es justo lo que Dirección señaló. Ahora cada etapa del buzón hereda
        // el contador entero: falso en el reparto y verdadero en lo que
        // importa, porque el total del día no sube. El error cae del lado de
        // enviar de menos.
        let (cubos, consumido): (i64, i64) = conn
            .query_row(
                "SELECT count(*), COALESCE(MAX(consumed), -1) FROM rate_budget
                  WHERE window_kind = 'daily'",
                [],
                |f| Ok((f.get(0)?, f.get(1)?)),
            )
            .expect("cuenta");
        assert_eq!(
            cubos, 1,
            "la campaña que la V4 creó hereda el cubo diario de su buzón"
        );
        assert_eq!(
            consumido, 7,
            "y hereda los SIETE que el buzón llevaba gastados: con cero, quien \
             actualice a media tarde manda de más ese día"
        );

        // Y ningún cubo se queda colgando del buzón sin etapa: esos son los que
        // la lectura ya no mira, así que dejarlos sería guardar un número que
        // nadie consulta y que el respaldo tendría que explicar.
        let huerfanos: i64 = conn
            .query_row(
                "SELECT count(*) FROM rate_budget WHERE stage_id IS NULL",
                [],
                |f| f.get(0),
            )
            .expect("cuenta");
        assert_eq!(huerfanos, 0, "no quedan cubos sin etapa");

        // Y la columna que la V13 añade existe, porque es de lo que cuelga el
        // presupuesto desde la D-13. Sin ella, `anotar_un_envio` fallaría en
        // cada envío y no al migrar — un fallo que llegaría al usuario.
        let tiene_etapa: i64 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('rate_budget')
                  WHERE name = 'stage_id'",
                [],
                |f| f.get(0),
            )
            .expect("cuenta");
        assert_eq!(tiene_etapa, 1, "la V13 añade `stage_id` a `rate_budget`");
    }

    #[test]
    fn aplicar_dos_veces_es_idempotente() {
        let (_d, mut conn) = base_de_prueba();
        let antes = tablas(&conn).len();
        aplicar(&mut conn).expect("segunda aplicación");
        assert_eq!(tablas(&conn).len(), antes);
    }
}
