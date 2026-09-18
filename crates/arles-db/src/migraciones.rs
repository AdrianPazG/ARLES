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

        // Y el cubo de ritmo conserva lo consumido, ahora sabiendo de qué canal es.
        let (canal_cubo, consumido): (String, i64) = conn
            .query_row("SELECT channel, consumed FROM rate_budget", [], |f| {
                Ok((f.get(0)?, f.get(1)?))
            })
            .expect("consulta");
        assert_eq!(canal_cubo, "email");
        assert_eq!(
            consumido, 7,
            "se reinició el cubo al migrar: la campaña enviaría de más hoy"
        );
    }

    #[test]
    fn aplicar_dos_veces_es_idempotente() {
        let (_d, mut conn) = base_de_prueba();
        let antes = tablas(&conn).len();
        aplicar(&mut conn).expect("segunda aplicación");
        assert_eq!(tablas(&conn).len(), antes);
    }
}
