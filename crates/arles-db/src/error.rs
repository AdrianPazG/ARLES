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

    /// Una fila almacenada ya no pasa la validación del dominio.
    ///
    /// Solo puede ocurrir si alguien editó la base por fuera de la aplicación o
    /// si una migración dejó datos a medias. **Se nombra en vez de repararse en
    /// silencio**: «arreglar» el dato aquí sería decidir por el usuario qué
    /// quiso escribir.
    #[error("un dato almacenado no es válido: {motivo}")]
    DatoInvalido { motivo: &'static str },

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
            Self::DatoInvalido { .. } => "error.db.dato_invalido",
            Self::Sqlite(_) => "error.db.sqlite",
        }
    }
}
