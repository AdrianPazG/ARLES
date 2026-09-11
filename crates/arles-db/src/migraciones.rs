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
        ] {
            assert!(
                t.iter().any(|x| x == esperada),
                "falta la tabla {esperada}; hay: {t:?}"
            );
        }
    }

    #[test]
    fn aplicar_dos_veces_es_idempotente() {
        let (_d, mut conn) = base_de_prueba();
        let antes = tablas(&conn).len();
        aplicar(&mut conn).expect("segunda aplicación");
        assert_eq!(tablas(&conn).len(), antes);
    }
}
