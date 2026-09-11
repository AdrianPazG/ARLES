//! Errores del shell, y su forma al cruzar la frontera IPC.

use serde::Serialize;
use thiserror::Error;

/// Error de arranque o de un comando.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    /// El almacén de credenciales del sistema no responde.
    ///
    /// **La aplicación no arranca.** Ver ADR-0011.
    #[error("el almacén de credenciales del sistema no está disponible: {0}")]
    LlaveroNoDisponible(String),

    #[error("no se pudo determinar el directorio de datos de la aplicación")]
    DirectorioDeDatos,

    #[error(transparent)]
    Db(#[from] arles_db::DbError),

    #[error(transparent)]
    Core(#[from] arles_core::CoreError),
}

impl AppError {
    /// Clave estable para que la interfaz traduzca el error con i18n (§139).
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::LlaveroNoDisponible(_) => "error.app.llavero_no_disponible",
            Self::DirectorioDeDatos => "error.app.directorio_de_datos",
            Self::Db(e) => e.clave_i18n(),
            Self::Core(e) => e.clave_i18n(),
        }
    }
}

/// Forma en que un error cruza la frontera IPC.
///
/// **Tipado, no una cadena.** El §95 exige que cada error diga qué pasó, cómo
/// arreglarlo y qué está a salvo, y eso es imposible de construir en la interfaz
/// a partir de un `Result<T, String>`: el frontend necesita distinguir variantes
/// para elegir el texto localizado correcto.
///
/// El campo `detalle` es para diagnóstico y **nunca** contiene secretos ni rutas
/// del sistema de archivos (THREAT_MODEL.md §4.1).
#[derive(Debug, Serialize)]
pub struct ErrorIpc {
    /// Clave de i18n: la interfaz resuelve con ella el texto de tres partes.
    pub clave: String,
    /// Texto técnico, para la bitácora de diagnóstico. No se muestra tal cual.
    pub detalle: String,
}

impl From<AppError> for ErrorIpc {
    fn from(e: AppError) -> Self {
        Self {
            clave: e.clave_i18n().to_owned(),
            detalle: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toda_variante_tiene_clave_i18n() {
        let casos = [
            AppError::LlaveroNoDisponible("prueba".into()),
            AppError::DirectorioDeDatos,
            AppError::Db(arles_db::DbError::ClaveIncorrecta),
        ];
        for e in casos {
            assert!(e.clave_i18n().starts_with("error."));
        }
    }

    #[test]
    fn el_error_ipc_lleva_clave_y_detalle() {
        let ipc: ErrorIpc = AppError::Db(arles_db::DbError::ClaveIncorrecta).into();
        assert_eq!(ipc.clave, "error.db.clave_incorrecta");
        assert!(!ipc.detalle.is_empty());
    }

    /// THREAT_MODEL.md §4.1: los mensajes no revelan la estructura del disco.
    #[test]
    fn los_errores_no_revelan_rutas() {
        for e in [
            AppError::DirectorioDeDatos,
            AppError::Db(arles_db::DbError::ClaveIncorrecta),
        ] {
            let t = e.to_string();
            assert!(!t.contains('/'), "el error revela una ruta: {t}");
            assert!(!t.contains('\\'), "el error revela una ruta: {t}");
        }
    }
}
