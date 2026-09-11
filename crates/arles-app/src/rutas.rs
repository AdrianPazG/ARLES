//! Directorios de datos, según la convención de cada plataforma.
//!
//! §7: nada de rutas absolutas atadas a una máquina.

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::error::AppError;

const ORGANIZACION: &str = "TelemetryInsight";
const APLICACION: &str = "ArlesRelay";

/// Directorios de la aplicación.
///
/// | | Windows | macOS |
/// |---|---|---|
/// | Datos | `%APPDATA%\TelemetryInsight\ArlesRelay` | `~/Library/Application Support/…` |
/// | Registros | `%LOCALAPPDATA%\…\logs` | `~/Library/Logs/…` |
#[derive(Debug, Clone)]
pub struct Rutas {
    datos: PathBuf,
}

impl Rutas {
    /// Resuelve los directorios y se asegura de que existen.
    ///
    /// # Errores
    ///
    /// [`AppError::DirectorioDeDatos`] si la plataforma no expone un directorio
    /// de datos o no se puede crear.
    pub fn resolver() -> Result<Self, AppError> {
        let dirs =
            ProjectDirs::from("mx", ORGANIZACION, APLICACION).ok_or(AppError::DirectorioDeDatos)?;
        let datos = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&datos).map_err(|_| AppError::DirectorioDeDatos)?;
        Ok(Self { datos })
    }

    /// Construye rutas bajo un directorio concreto. Para tests.
    #[must_use]
    pub fn en(datos: PathBuf) -> Self {
        Self { datos }
    }

    /// Archivo de la base de datos cifrada.
    #[must_use]
    pub fn base_de_datos(&self) -> PathBuf {
        self.datos.join("arles.db")
    }

    #[must_use]
    pub fn datos(&self) -> &PathBuf {
        &self.datos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_base_de_datos_vive_bajo_el_directorio_de_datos() {
        let dir = tempfile::tempdir().expect("temporal");
        let r = Rutas::en(dir.path().to_path_buf());
        let db = r.base_de_datos();
        assert!(db.starts_with(dir.path()));
        assert_eq!(db.file_name().and_then(|s| s.to_str()), Some("arles.db"));
    }

    /// §7: el nombre de la base no depende de la empresa ni del usuario, así que
    /// una instalación es portable entre máquinas del mismo sistema.
    #[test]
    fn el_nombre_del_archivo_es_estable() {
        let a = Rutas::en(PathBuf::from("/a")).base_de_datos();
        let b = Rutas::en(PathBuf::from("/b")).base_de_datos();
        assert_eq!(a.file_name(), b.file_name());
    }
}
