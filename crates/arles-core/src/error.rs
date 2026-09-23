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
    /// Una fila guardada trae un origen de importación que no reconocemos.
    ///
    /// Sólo puede pasar si alguien editó la base por fuera o si una migración
    /// dejó datos a medias. **No se repara cayendo en «otro»**: eso cambiaría
    /// lo que el usuario declaró, que es justo el dato que se guarda para poder
    /// responder a una reclamación.
    #[error("el origen declarado de esa importación no es uno de los admitidos")]
    OrigenDesconocido,

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

    /// Una fila de supresión trae un motivo que no reconocemos.
    ///
    /// **No se repara cayendo en «manual»**: el motivo decide si esa supresión
    /// se puede levantar con un botón. Adivinarlo mal convertiría una baja que
    /// pidió una persona en algo que cualquiera puede deshacer.
    #[error("el motivo de esa supresión no es uno de los admitidos")]
    MotivoDeSupresionDesconocido,

    /// Una fila de supresión trae un origen que no reconocemos.
    #[error("el origen de esa supresión no es uno de los admitidos")]
    OrigenDeSupresionDesconocido,

    /// Una fila de supresión trae un alcance que no reconocemos.
    #[error("el alcance de esa supresión no es uno de los admitidos")]
    AlcanceDeSupresionDesconocido,

    /// Una campaña guardada trae un estado que no reconocemos.
    ///
    /// **No se repara suponiendo «borrador»**: el estado decide qué acciones
    /// se admiten, y suponer el más inofensivo dejaría pasar un «reanudar»
    /// sobre una campaña que en realidad estaba detenida.
    #[error("el estado de esa campaña no es uno de los admitidos")]
    EstadoDeCampanaDesconocido,

    /// Se pidió suprimir sin decir qué dirección.
    ///
    /// Una supresión sin dirección no bloquea a nadie: dejaría una fila en la
    /// pantalla y la sensación de que algo se hizo.
    #[error("no se puede suprimir sin al menos una dirección")]
    SupresionSinCanales,

    /// Se pidió suprimir «este canal» pero vinieron varios.
    ///
    /// No se amplía a global en silencio: quien pidió una cosa no pidió la
    /// otra, y ampliar una supresión sin decirlo cambia lo que alguien decidió.
    #[error("el alcance de la supresión no coincide con las direcciones dadas")]
    AlcanceNoCoincideConLosCanales,

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
            Self::OrigenDesconocido => "error.origen_desconocido",
            Self::MotivoDeSupresionDesconocido => "error.motivo_de_supresion_desconocido",
            Self::OrigenDeSupresionDesconocido => "error.origen_de_supresion_desconocido",
            Self::AlcanceDeSupresionDesconocido => "error.alcance_de_supresion_desconocido",
            Self::EstadoDeCampanaDesconocido => "error.estado_de_campana_desconocido",
            Self::SupresionSinCanales => "error.supresion_sin_canales",
            Self::AlcanceNoCoincideConLosCanales => "error.alcance_no_coincide",
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
