//! Lo que puede salir mal **antes** de hablar con el servidor.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTO NO ES LO MISMO QUE UN `Fallo`
//!
//! Son dos cosas distintas y conviene no mezclarlas:
//!
//! - [`ErrorDeCorreo`] es **la cuenta está mal configurada**. Ocurre al
//!   construir el adaptador, antes de que exista ningún envío, y no tiene nada
//!   que ver con ningún contacto. Se le enseña a quien configura la cuenta, en
//!   la pantalla de Remitentes.
//! - `arles_core::reintento::Fallo` es **este envío concreto no salió**. Lo
//!   produce el servidor, y el motor decide con él si reintenta, se rinde o
//!   para la cuenta.
//!
//! Si un problema de configuración llegara como `Fallo`, el motor lo trataría
//! como un envío fallido: gastaría reintentos, contaría para el disyuntor y
//! marcaría contactos. Un puerto mal escrito acabaría dando de baja a nadie —
//! pero sí dejaría la cuenta parada «por fallos seguidos», que es exactamente
//! el diagnóstico equivocado.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! CADA VARIANTE LLEVA SU CLAVE (§139)
//!
//! Nunca texto. El texto vive en `app/src/app/locales/es.ts` bajo
//! `error.correo.*`, con sus tres partes: qué pasó, cómo se arregla y qué está
//! a salvo (§95).

use std::fmt;

/// Un problema de configuración de la cuenta de correo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDeCorreo {
    /// No se pudo preparar el cifrado de la conexión.
    ///
    /// **No existe un camino que siga sin cifrar** (§29). Si esto falla, no hay
    /// adaptador; ver la nota de `tls.rs`.
    NoSePudoCifrar,
    /// El servidor que se escribió no es un nombre de máquina utilizable.
    ///
    /// Tiene que ser el **nombre** —`smtp.gmail.com`—, no una dirección IP: un
    /// certificado se emite para el nombre, y validarlo contra el número al que
    /// resuelve ese día no funciona.
    ElServidorNoVale,
    /// La dirección del remitente no es una dirección de correo.
    LaDireccionDelRemitenteNoVale,
    /// El mensaje no se pudo armar con los datos que hay.
    ///
    /// Es un fallo nuestro, no del servidor ni de la cuenta: llega antes de
    /// abrir ninguna conexión.
    NoSePudoArmarElMensaje,
}

impl ErrorDeCorreo {
    /// La clave del texto que se le enseña a quien configura la cuenta (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::NoSePudoCifrar => "error.correo.no_se_pudo_cifrar",
            Self::ElServidorNoVale => "error.correo.el_servidor_no_vale",
            Self::LaDireccionDelRemitenteNoVale => "error.correo.remitente_no_vale",
            Self::NoSePudoArmarElMensaje => "error.correo.no_se_pudo_armar",
        }
    }
}

impl fmt::Display for ErrorDeCorreo {
    /// Escribe **la clave**, no el texto.
    ///
    /// Es a propósito: si esto escribiera español, ese español acabaría en una
    /// pantalla sin pasar por el catálogo, y el día que haya una segunda lengua
    /// sería el único sitio que no se traduce. Además, un `Display` de un error
    /// acaba en bitácoras, y una clave es lo que se puede buscar.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.clave_i18n())
    }
}

impl std::error::Error for ErrorDeCorreo {}

#[cfg(test)]
mod tests {
    use super::*;

    const TODOS: [ErrorDeCorreo; 4] = [
        ErrorDeCorreo::NoSePudoCifrar,
        ErrorDeCorreo::ElServidorNoVale,
        ErrorDeCorreo::LaDireccionDelRemitenteNoVale,
        ErrorDeCorreo::NoSePudoArmarElMensaje,
    ];

    /// **Ningún error se queda sin clave, y ninguna clave se repite** (§139).
    ///
    /// Repetir una clave es el fallo silencioso de los dos: compila, no avisa,
    /// y en la pantalla dos problemas distintos dicen lo mismo — con lo cual el
    /// texto de «cómo se arregla» es el de otro problema.
    ///
    /// Verificada rompiéndola el 23/09/2026: dándole a `ElServidorNoVale` la
    /// clave de `NoSePudoCifrar`, esta prueba falla por el duplicado.
    #[test]
    fn cada_error_tiene_su_propia_clave() {
        let mut claves: Vec<&str> = TODOS.iter().map(ErrorDeCorreo::clave_i18n).collect();
        let cuantas = claves.len();
        claves.sort_unstable();
        claves.dedup();
        assert_eq!(claves.len(), cuantas, "hay dos errores con la misma clave");

        for e in TODOS {
            assert!(
                e.clave_i18n().starts_with("error.correo."),
                "{e:?} tiene que colgar de error.correo.*"
            );
        }
    }

    /// **El `Display` no escribe español.**
    ///
    /// Si lo escribiera, ese texto acabaría en una pantalla sin pasar por el
    /// catálogo de textos, y sería el único sitio que no se traduce.
    #[test]
    fn el_display_escribe_la_clave_y_no_el_texto() {
        for e in TODOS {
            assert_eq!(e.to_string(), e.clave_i18n());
        }
    }
}
