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

    /// Una fila trae un cifrado de correo que no reconocemos.
    ///
    /// **No se repara cayendo en un valor por defecto.** El «más parecido» a un
    /// valor corrupto no existe aquí: lo único que hay son dos formas de
    /// cifrar, y elegir una por su cuenta sería inventarse cómo viaja la
    /// contraseña de la empresa (§29).
    #[error("el cifrado de esa cuenta de correo no es uno de los admitidos")]
    CifradoDesconocido,

    /// Una fila de la base trae un canal que no reconocemos.
    ///
    /// No es un caso a ignorar: el canal decide **a quién se le escribe y por
    /// dónde**. Una fila con un canal corrupto se para, no se salta.
    #[error("el canal no es uno de los admitidos")]
    CanalDesconocido,

    /// Una fila trae un modo de canal que no reconocemos.
    ///
    /// **No se aproxima a «seguimiento»**, que es el seguro. Leer mal un número
    /// en frío como si fuera de seguimiento haría desaparecer todas las
    /// advertencias de la pantalla — y los mensajes saldrían igual, en frío, y
    /// sin que nadie lo hubiera dicho.
    #[error("el modo del canal no es uno de los admitidos")]
    ModoDeCanalDesconocido,

    /// Una plantilla de WhatsApp trae un estado que Meta no tenía cuando esto
    /// se escribió.
    ///
    /// **No se aproxima a «aprobada»**: eso mandaría mensajes con una plantilla
    /// que Meta no aceptó, y cada rechazo cuenta contra la calidad del número.
    #[error("el estado de la plantilla no es uno de los admitidos")]
    EstadoDePlantillaDesconocido,

    /// Se intentó mandar con una plantilla que Meta no ha aprobado.
    ///
    /// Es un error del dominio y no de la API a propósito: se caza **antes** de
    /// salir a la red, en `PlantillaDeMeta::preparar`, que es el único sitio por
    /// el que pasa todo mensaje de WhatsApp.
    #[error("esa plantilla no está aprobada por Meta")]
    PlantillaNoAprobada,

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

    /// El archivo que se eligió no es un respaldo de ARLES (8.1).
    ///
    /// Se distingue de un respaldo roto **a propósito**: los dos impiden
    /// restaurar, pero lo que hay que hacer es distinto. Aquí se elige otro
    /// archivo; allí se busca otra copia. Un único error para los dos casos
    /// mandaría a media gente a buscar donde no hay nada.
    #[error("el archivo elegido no es un respaldo de ARLES")]
    NoEsUnRespaldo,

    /// El respaldo lo hizo una versión de ARLES más nueva que ésta.
    ///
    /// **No se intenta leer igualmente.** Un formato posterior puede traer
    /// tablas o columnas que esta versión no conoce, y leerlo «lo que se pueda»
    /// restauraría una base incompleta sin que nada lo dijera — que es
    /// exactamente el fallo que toda la Fase 8 existe para evitar.
    #[error("el respaldo es del formato {formato} y esta versión lee hasta el {admitido}")]
    RespaldoDeVersionFutura { formato: u32, admitido: u32 },

    /// El respaldo está incompleto o alterado.
    ///
    /// Lo caza la suma de verificación o una línea que no se puede leer. En los
    /// dos casos **no se restaura nada**: una restauración a medias deja la base
    /// en un estado que nadie ha pensado.
    #[error("el respaldo está incompleto o alterado: {motivo}")]
    RespaldoCorrupto { motivo: &'static str },

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
            Self::CifradoDesconocido => "error.cifrado_desconocido",
            Self::CanalDesconocido => "error.canal_desconocido",
            Self::ModoDeCanalDesconocido => "error.modo_de_canal_desconocido",
            Self::EstadoDePlantillaDesconocido => "error.estado_de_plantilla_desconocido",
            Self::PlantillaNoAprobada => "error.plantilla_no_aprobada",
            Self::IdInvalido { .. } => "error.id_invalido",
            Self::OrigenDesconocido => "error.origen_desconocido",
            Self::MotivoDeSupresionDesconocido => "error.motivo_de_supresion_desconocido",
            Self::OrigenDeSupresionDesconocido => "error.origen_de_supresion_desconocido",
            Self::AlcanceDeSupresionDesconocido => "error.alcance_de_supresion_desconocido",
            Self::EstadoDeCampanaDesconocido => "error.estado_de_campana_desconocido",
            Self::SupresionSinCanales => "error.supresion_sin_canales",
            Self::AlcanceNoCoincideConLosCanales => "error.alcance_no_coincide",
            Self::NoEsUnRespaldo => "error.no_es_un_respaldo",
            Self::RespaldoDeVersionFutura { .. } => "error.respaldo_de_version_futura",
            Self::RespaldoCorrupto { .. } => "error.respaldo_corrupto",
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
            CoreError::ModoDeCanalDesconocido,
            CoreError::EstadoDePlantillaDesconocido,
            CoreError::PlantillaNoAprobada,
            CoreError::IdInvalido { tipo: "ContactId" },
            CoreError::NoEsUnRespaldo,
            CoreError::RespaldoDeVersionFutura {
                formato: 2,
                admitido: 1,
            },
            CoreError::RespaldoCorrupto { motivo: "prueba" },
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
