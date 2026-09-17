//! Errores del dominio.
//!
//! Los errores son **tipados, no cadenas**. El §95 exige que cada error diga
//! qué pasó, cómo arreglarlo y qué está a salvo — y eso es imposible de
//! construir en la interfaz a partir de un `Result<T, String>`.
//!
//! Ver `documentacion/03-arquitectura/ARQUITECTURA.md` §5.

use thiserror::Error;

/// Error de validación o de invariante del dominio.
///
/// Ningún mensaje de error incluye rutas del sistema de archivos ni valores de
/// secretos (THREAT_MODEL.md §4.1).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CoreError {
    #[error("dirección de correo inválida: {motivo}")]
    EmailInvalido { motivo: &'static str },

    #[error("número de teléfono inválido: {motivo}")]
    TelefonoInvalido { motivo: &'static str },

    /// Una fila de la base trae un canal que no reconocemos.
    ///
    /// No es un caso a ignorar: el canal decide **a quién se le escribe y por
    /// dónde**. Una fila con un canal corrupto se para, no se salta.
    #[error("el canal no es uno de los admitidos")]
    CanalDesconocido,

    #[error("identificador de {tipo} inválido")]
    IdInvalido { tipo: &'static str },

    #[error(transparent)]
    Transicion(#[from] crate::attempt::TransitionError),
}

impl CoreError {
    /// Clave estable para que la interfaz traduzca el error con i18n (§139).
    ///
    /// La interfaz **no** muestra `Display`: muestra el texto localizado que
    /// corresponde a esta clave, con sus tres partes (§95).
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::EmailInvalido { .. } => "error.email_invalido",
            Self::TelefonoInvalido { .. } => "error.telefono_invalido",
            Self::CanalDesconocido => "error.canal_desconocido",
            Self::IdInvalido { .. } => "error.id_invalido",
            Self::Transicion(_) => "error.transicion_invalida",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attempt::AttemptState;

    #[test]
    fn cada_variante_tiene_clave_i18n() {
        let casos = [
            CoreError::EmailInvalido { motivo: "prueba" },
            CoreError::TelefonoInvalido { motivo: "prueba" },
            CoreError::CanalDesconocido,
            CoreError::IdInvalido { tipo: "ContactId" },
            CoreError::Transicion(
                AttemptState::Sent
                    .transicionar(AttemptState::Queued)
                    .expect_err("debería fallar"),
            ),
        ];
        for e in casos {
            assert!(e.clave_i18n().starts_with("error."));
        }
    }

    #[test]
    fn los_mensajes_no_revelan_rutas() {
        let e = CoreError::EmailInvalido {
            motivo: "falta la arroba",
        };
        let texto = e.to_string();
        assert!(!texto.contains('/'));
        assert!(!texto.contains('\\'));
    }
}
