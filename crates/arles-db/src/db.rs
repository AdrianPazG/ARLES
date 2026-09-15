//! El asa de la base de datos.
//!
//! Encapsula la conexión para que **`rusqlite` no salga de este crate**. Los
//! crates de dominio y el shell hablan con métodos tipados, no con SQL ni con
//! tipos del motor: así cambiar el motor —o simplemente auditar qué consultas
//! existen— es un trabajo acotado a `arles-db`.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use rusqlite::Connection;

use crate::conexion::{self, ClaveMaestra};
use crate::error::DbError;

/// Conexión a la base de datos cifrada, lista para usarse desde varios hilos.
///
/// **Un solo escritor.** SQLite con WAL admite lectores concurrentes pero un
/// único escritor; serializar aquí **elimina de raíz** la clase de errores
/// `SQLITE_BUSY` en vez de gestionarlos con reintentos que funcionan el 99 % de
/// las veces (ADR-0002).
#[derive(Debug)]
pub struct Db {
    conexion: Mutex<Connection>,
}

/// Qué se sabe de la base al arrancar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenArranque {
    /// Versión del esquema aplicada por las migraciones.
    pub version_esquema: Option<i32>,
    /// Cuántas empresas hay configuradas. Cero significa que toca el
    /// onboarding del §25.
    pub empresas: i64,
}

impl Db {
    /// Abre la base cifrada, aplica los PRAGMA y corre las migraciones.
    ///
    /// # Errores
    ///
    /// Ver [`crate::conexion::abrir`].
    pub fn abrir(ruta: &Path, clave: &ClaveMaestra) -> Result<Self, DbError> {
        Ok(Self {
            conexion: Mutex::new(conexion::abrir(ruta, clave)?),
        })
    }

    /// Toma la conexión.
    ///
    /// Es `pub(crate)` para que los repositorios de este crate la usen; **no
    /// sale de `arles-db`**, que es lo que mantiene `rusqlite` dentro (ADR-0002).
    ///
    /// Un mutex envenenado significa que otro hilo entró en pánico teniéndola
    /// tomada. Se recupera el guardia en vez de propagar el pánico: la conexión
    /// sigue siendo válida y tumbar la aplicación sería peor para el usuario.
    pub(crate) fn con<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        let guardia = self.conexion.lock().unwrap_or_else(PoisonError::into_inner);
        f(&guardia).map_err(DbError::Sqlite)
    }

    /// Estado de la base al arrancar.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn resumen_arranque(&self) -> Result<ResumenArranque, DbError> {
        self.con(|conn| {
            let empresas: i64 =
                conn.query_row("SELECT count(*) FROM company", [], |f| f.get::<_, i64>(0))?;

            // `refinery` crea esta tabla; si no existe, la consulta falla y
            // devolvemos `None` en vez de tumbar el arranque.
            let version_esquema = conn
                .query_row(
                    "SELECT max(version) FROM refinery_schema_history",
                    [],
                    |f| f.get::<_, Option<i32>>(0),
                )
                .ok()
                .flatten();

            Ok(ResumenArranque {
                version_esquema,
                empresas,
            })
        })
    }

    /// Ejecuta SQL arbitrario. **Solo para tests**: el código de producción usa
    /// métodos tipados para que las consultas sean auditables (ADR-0002).
    #[cfg(any(test, feature = "test-util"))]
    pub fn ejecutar_en_pruebas<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        self.con(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    #[test]
    fn una_base_nueva_no_tiene_empresa_configurada() {
        let (_d, db) = base();
        let r = db.resumen_arranque().expect("resumen");
        assert_eq!(
            r.empresas, 0,
            "una instalación nueva debe llevar al onboarding"
        );
    }

    #[test]
    fn el_resumen_reporta_la_version_del_esquema() {
        let (_d, db) = base();
        let r = db.resumen_arranque().expect("resumen");
        assert_eq!(
            r.version_esquema,
            Some(2),
            "las migraciones deberían haber dejado el esquema en la última versión"
        );
    }

    #[test]
    fn el_resumen_ve_la_empresa_una_vez_creada() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('c1', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-11T00:00:00Z', '2026-09-11T00:00:00Z')",
                [],
            )
        })
        .expect("inserta");

        assert_eq!(db.resumen_arranque().expect("resumen").empresas, 1);
    }

    /// El asa debe poder compartirse entre hilos: el motor de ejecución correrá
    /// en su propio conjunto de tareas, independiente de la ventana (§51).
    #[test]
    fn el_asa_es_compartible_entre_hilos() {
        fn exige_sync_send<T: Sync + Send>() {}
        exige_sync_send::<Db>();
    }
}
