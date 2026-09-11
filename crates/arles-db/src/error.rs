//! Errores de acceso a datos.

use thiserror::Error;

/// Error al abrir, migrar o consultar la base de datos.
///
/// Ningún mensaje incluye la clave maestra ni rutas completas del sistema de
/// archivos (THREAT_MODEL.md §4.1).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DbError {
    /// La clave no abre la base de datos.
    ///
    /// Puede significar dos cosas y la interfaz debe distinguirlas: que la clave
    /// del llavero no corresponda a este archivo, o que el archivo esté
    /// corrupto. En ninguno de los dos casos se debe ofrecer «continuar sin
    /// cifrado» (ADR-0011).
    #[error("no se pudo descifrar la base de datos con la clave proporcionada")]
    ClaveIncorrecta,

    /// La clave no tiene la longitud esperada.
    #[error("la clave maestra debe tener exactamente {esperado} bytes, tiene {recibido}")]
    ClaveMalFormada { esperado: usize, recibido: usize },

    #[error("fallo al migrar el esquema: {0}")]
    Migracion(String),

    #[error("error de SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

impl DbError {
    /// Clave estable para que la interfaz traduzca el error con i18n (§139).
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::ClaveIncorrecta => "error.db.clave_incorrecta",
            Self::ClaveMalFormada { .. } => "error.db.clave_mal_formada",
            Self::Migracion(_) => "error.db.migracion",
            Self::Sqlite(_) => "error.db.sqlite",
        }
    }
}
