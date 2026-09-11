//! `Secret<T>`: valores que nunca deben aparecer en un registro.
//!
//! Ver `documentacion/04-seguridad/MODELO_DE_SECRETOS.md` §4 y ADR-0011.

use std::fmt;

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Envoltorio para credenciales, tokens y claves.
///
/// Tres propiedades deliberadas:
///
/// - **`Debug` y `Display` redactados.** Un `tracing::debug!("{account:?}")`
///   descuidado no puede filtrar el valor.
/// - **Sin `Deref`.** Exponer el valor exige llamar a [`Secret::expose_secret`],
///   que es visible en revisión de código y se encuentra con grep.
/// - **Se limpia al soltarse**, para reducir la ventana en que el secreto queda
///   en memoria liberada.
///
/// `Secret<T>` protege de lo que pasa por él. La capa de redacción de `tracing`
/// protege de lo que no. Se usan **las dos**, no una: una defensa que depende de
/// que todo esté envuelto correctamente se rompe con el tiempo.
///
/// ```
/// use arles_core::Secret;
///
/// let token = Secret::new(String::from("ya29.super-secreto"));
/// assert_eq!(format!("{token:?}"), "[REDACTADO]");
/// assert_eq!(format!("{token}"), "[REDACTADO]");
/// assert_eq!(token.expose_secret(), "ya29.super-secreto");
/// ```
#[derive(Clone, ZeroizeOnDrop)]
pub struct Secret<T: Zeroize>(T);

/// Lo que se imprime en lugar del valor real.
const REDACTADO: &str = "[REDACTADO]";

impl<T: Zeroize> Secret<T> {
    pub fn new(valor: T) -> Self {
        Self(valor)
    }

    /// Acceso explícito al valor. Que la llamada sea visible es el punto.
    pub fn expose_secret(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTADO)
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTADO)
    }
}

/// Nunca se serializa un secreto: un `Secret<T>` no debe poder acabar en JSON,
/// en la base de datos ni cruzando la frontera IPC (regla de frontera 3.4).
/// Por eso este tipo **no implementa `Serialize`** a propósito.
impl<T: Zeroize> From<T> for Secret<T> {
    fn from(valor: T) -> Self {
        Self::new(valor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_no_filtra_el_valor() {
        let s = Secret::new(String::from("contraseña-de-aplicacion"));
        let salida = format!("{s:?}");
        assert_eq!(salida, REDACTADO);
        assert!(!salida.contains("contraseña"));
    }

    #[test]
    fn display_no_filtra_el_valor() {
        let s = Secret::new(String::from("ya29.token"));
        assert!(!format!("{s}").contains("ya29"));
    }

    #[test]
    fn anidado_en_una_estructura_tampoco_filtra() {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Cuenta {
            correo: String,
            credencial: Secret<String>,
        }

        let c = Cuenta {
            correo: "ventas@empresa.com".into(),
            credencial: Secret::new("abcd efgh ijkl mnop".into()),
        };
        let salida = format!("{c:?}");

        assert!(salida.contains("ventas@empresa.com"));
        assert!(!salida.contains("abcd"));
        assert!(salida.contains(REDACTADO));
    }

    #[test]
    fn expose_secret_devuelve_el_valor() {
        let s = Secret::new(vec![1_u8, 2, 3]);
        assert_eq!(s.expose_secret(), &vec![1_u8, 2, 3]);
    }
}
