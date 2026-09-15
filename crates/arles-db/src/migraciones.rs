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

    #[test]
    fn crea_las_tablas_del_modelo_de_datos() {
        let (_d, conn) = base_de_prueba();
        let t = tablas(&conn);
        for esperada in [
            "company",
            "email_account",
            "contact",
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

    #[test]
    fn aplicar_dos_veces_es_idempotente() {
        let (_d, mut conn) = base_de_prueba();
        let antes = tablas(&conn).len();
        aplicar(&mut conn).expect("segunda aplicación");
        assert_eq!(tablas(&conn).len(), antes);
    }
}
