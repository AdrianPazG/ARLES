//! Preferencias de interfaz.
//!
//! Ajustes del equipo que se recuerdan entre sesiones. **Nada de negocio vive
//! aquí**: ninguna preferencia de esta tabla decide a quién se le escribe, ni
//! cuándo, ni con qué límites.
//!
//! Ver la migración `V2__preferencias_de_interfaz.sql` para por qué están en la
//! base y no en `localStorage`.

use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Claves admitidas. **Lista cerrada.**
///
/// La webview puede pedir cualquier cosa, y un almacén clave/valor abierto al
/// otro lado de la frontera IPC es una tabla que crece con lo que a alguien se
/// le ocurra escribir. Con la lista cerrada, una clave desconocida es un error
/// visible, no una fila huérfana que nadie vuelve a leer.
pub const CLAVES_DE_INTERFAZ: &[&str] = &[
    // P-11: «Sí, se recuerda». Valor «1» plegada, «0» desplegada.
    "barra_lateral_plegada",
    // C-1: el tema que eligió el usuario. Valores «auto», «oscuro» o «claro».
    //
    // Qué significa cada uno lo decide `arles-app`, no esta capa: aquí sólo se
    // guarda una cadena corta. Lo que sí es de aquí es que la clave exista en
    // la lista, porque si no, guardarla se rechaza.
    "tema",
];

/// Tope del valor. Una preferencia de interfaz no necesita más, y sin tope la
/// frontera IPC admite que alguien engorde la base con un solo comando.
const MAX_VALOR: usize = 64;

impl Db {
    /// Lee una preferencia.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si la clave no está en [`CLAVES_DE_INTERFAZ`],
    /// y [`DbError::Sqlite`] si la consulta falla.
    pub fn preferencia(&self, clave: &str) -> Result<Option<String>, DbError> {
        exigir_clave(clave)?;
        self.con(|conn| {
            conn.query_row(
                "SELECT value FROM ui_preference WHERE key = ?1",
                params![clave],
                |f| f.get(0),
            )
            .optional()
        })
    }

    /// Guarda una preferencia.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si la clave no está admitida o el valor excede
    /// el tope, y [`DbError::Sqlite`] si la escritura falla.
    pub fn guardar_preferencia(&self, clave: &str, valor: &str) -> Result<(), DbError> {
        exigir_clave(clave)?;
        if valor.len() > MAX_VALOR {
            return Err(DbError::DatoInvalido {
                motivo: "el valor de la preferencia excede el tope",
            });
        }
        self.con(|conn| {
            conn.execute(
                "INSERT INTO ui_preference (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
                params![clave, valor, ahora()],
            )?;
            Ok(())
        })
    }
}

fn exigir_clave(clave: &str) -> Result<(), DbError> {
    if CLAVES_DE_INTERFAZ.contains(&clave) {
        Ok(())
    } else {
        Err(DbError::DatoInvalido {
            motivo: "preferencia de interfaz desconocida",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    #[test]
    fn una_preferencia_sin_escribir_no_existe() {
        let (_d, db) = base();
        assert_eq!(db.preferencia("barra_lateral_plegada").expect("lee"), None);
    }

    /// P-11: «Sí, se recuerda». Esto es esa decisión, comprobada.
    #[test]
    fn lo_guardado_se_recuerda() {
        let (_d, db) = base();
        db.guardar_preferencia("barra_lateral_plegada", "1")
            .expect("guarda");
        assert_eq!(
            db.preferencia("barra_lateral_plegada").expect("lee"),
            Some("1".to_owned())
        );
    }

    #[test]
    fn guardar_dos_veces_sustituye_el_valor() {
        let (_d, db) = base();
        db.guardar_preferencia("barra_lateral_plegada", "1")
            .expect("guarda");
        db.guardar_preferencia("barra_lateral_plegada", "0")
            .expect("guarda");
        assert_eq!(
            db.preferencia("barra_lateral_plegada").expect("lee"),
            Some("0".to_owned())
        );
        let filas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT count(*) FROM ui_preference", [], |f| f.get(0))
            })
            .expect("cuenta");
        assert_eq!(filas, 1, "debería actualizar, no acumular filas");
    }

    /// La frontera IPC no puede convertirse en un almacén abierto: una clave
    /// que nadie declaró es un error visible, no una fila huérfana.
    #[test]
    fn una_clave_desconocida_se_rechaza_al_escribir_y_al_leer() {
        let (_d, db) = base();
        assert!(matches!(
            db.guardar_preferencia("lo_que_sea", "1"),
            Err(DbError::DatoInvalido { .. })
        ));
        assert!(matches!(
            db.preferencia("lo_que_sea"),
            Err(DbError::DatoInvalido { .. })
        ));
    }

    #[test]
    fn un_valor_desmesurado_se_rechaza() {
        let (_d, db) = base();
        let enorme = "a".repeat(MAX_VALOR + 1);
        assert!(matches!(
            db.guardar_preferencia("barra_lateral_plegada", &enorme),
            Err(DbError::DatoInvalido { .. })
        ));
    }

    /// Las preferencias no son de la empresa: son del equipo. Si lo fueran,
    /// borrar la empresa dejaría la barra lateral en un estado indefinido.
    #[test]
    fn la_preferencia_no_depende_de_la_empresa() {
        let (_d, db) = base();
        let columnas: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut s = c.prepare("SELECT name FROM pragma_table_info('ui_preference')")?;
                let v = s
                    .query_map([], |f| f.get::<_, String>(0))?
                    .filter_map(Result::ok)
                    .collect::<Vec<_>>();
                Ok(v)
            })
            .expect("consulta");
        assert!(
            !columnas.iter().any(|c| c == "company_id"),
            "las preferencias de interfaz no llevan empresa: {columnas:?}"
        );
    }
}
