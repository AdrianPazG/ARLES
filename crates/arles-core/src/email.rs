//! Direcciones de correo y su normalización.
//!
//! Ver `documentacion/03-arquitectura/MODELO_DE_DATOS.md` §3.1.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Límite defensivo. El RFC 5321 fija 320 octetos como máximo para una
/// dirección; cortamos ahí para que una celda gigante de un XLSX ajeno no se
/// convierta en un problema de memoria aguas abajo.
const MAX_LONGITUD: usize = 320;

/// Una dirección de correo validada, con su forma normalizada.
///
/// `raw` conserva lo que el usuario escribió, para poder mostrarlo tal cual.
/// `normalized` es la clave de deduplicación y de supresión.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailAddress {
    raw: String,
    normalized: String,
}

impl EmailAddress {
    /// Valida y normaliza una dirección.
    ///
    /// # Errores
    ///
    /// [`CoreError::EmailInvalido`] si está vacía, excede [`MAX_LONGITUD`], o no
    /// tiene una forma `local@dominio` mínimamente coherente.
    pub fn parse(entrada: &str) -> Result<Self, CoreError> {
        let raw = entrada.trim();

        if raw.is_empty() {
            return Err(CoreError::EmailInvalido {
                motivo: "la dirección está vacía",
            });
        }
        if raw.len() > MAX_LONGITUD {
            return Err(CoreError::EmailInvalido {
                motivo: "la dirección supera los 320 caracteres",
            });
        }

        let (local, dominio) = raw.split_once('@').ok_or(CoreError::EmailInvalido {
            motivo: "falta la arroba",
        })?;

        if local.is_empty() {
            return Err(CoreError::EmailInvalido {
                motivo: "falta la parte anterior a la arroba",
            });
        }
        if dominio.contains('@') {
            return Err(CoreError::EmailInvalido {
                motivo: "hay más de una arroba",
            });
        }
        if !dominio.contains('.') || dominio.starts_with('.') || dominio.ends_with('.') {
            return Err(CoreError::EmailInvalido {
                motivo: "el dominio no es válido",
            });
        }
        if raw.chars().any(char::is_whitespace) {
            return Err(CoreError::EmailInvalido {
                motivo: "la dirección contiene espacios",
            });
        }
        // Prevención de inyección de cabeceras: un valor con CR o LF que llegue
        // a una cabecera SMTP permite inyectar cabeceras arbitrarias.
        // Ver THREAT_MODEL.md §4.2. `is_whitespace` ya los cubre, pero lo
        // comprobamos explícitamente para que el motivo quede claro y para que
        // quitar la comprobación anterior no abra el agujero en silencio.
        if raw.contains(['\r', '\n']) {
            return Err(CoreError::EmailInvalido {
                motivo: "la dirección contiene saltos de línea",
            });
        }

        Ok(Self {
            raw: raw.to_owned(),
            normalized: raw.to_lowercase(),
        })
    }

    /// Tal y como lo escribió el usuario (sin espacios al principio ni al final).
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Clave de deduplicación y de supresión.
    ///
    /// Es `trim` + minúsculas, **y nada más**. En particular **no** se eliminan
    /// los puntos ni se recorta el sufijo `+`.
    ///
    /// Es tentador hacerlo porque `juan.perez@gmail.com` y `juanperez@gmail.com`
    /// son la misma persona en Gmail. Pero eso es **específico de Gmail**: en la
    /// mayoría de dominios corporativos son buzones distintos de personas
    /// distintas. Aplicarlo a todo el mundo fusionaría contactos que no son el
    /// mismo, y esa es una pérdida de datos silenciosa e irreversible.
    ///
    /// Preferimos un duplicado ocasional a una fusión incorrecta: el duplicado
    /// el usuario lo ve y lo arregla; la fusión no la ve nadie.
    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl fmt::Debug for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmailAddress({})", self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(s: &str) -> String {
        EmailAddress::parse(s)
            .map(|e| e.normalized().to_owned())
            .unwrap_or_else(|_| panic!("debería ser válido: {s}"))
    }

    #[test]
    fn normaliza_mayusculas_y_espacios() {
        assert_eq!(norm("  Juan.Perez@Empresa.COM  "), "juan.perez@empresa.com");
    }

    #[test]
    fn conserva_la_forma_original_para_mostrarla() {
        let e = EmailAddress::parse(" Juan.Perez@Empresa.COM ").expect("válido");
        assert_eq!(e.raw(), "Juan.Perez@Empresa.COM");
        assert_eq!(e.normalized(), "juan.perez@empresa.com");
    }

    /// La decisión de MODELO_DE_DATOS.md §3.1, escrita como test para que nadie
    /// "mejore" la normalización sin darse cuenta de lo que rompe.
    #[test]
    fn no_elimina_puntos_porque_eso_es_especifico_de_gmail() {
        assert_ne!(
            norm("juan.perez@empresa.com"),
            norm("juanperez@empresa.com")
        );
    }

    #[test]
    fn no_recorta_el_sufijo_mas() {
        assert_ne!(norm("juan+ofertas@empresa.com"), norm("juan@empresa.com"));
    }

    #[test]
    fn detecta_duplicados_reales_por_mayusculas() {
        assert_eq!(norm("VENTAS@empresa.com"), norm("ventas@EMPRESA.com"));
    }

    #[test]
    fn rechaza_direcciones_mal_formadas() {
        for entrada in [
            "",
            "   ",
            "sinarroba.com",
            "@empresa.com",
            "doble@@empresa.com",
            "juan@sinpunto",
            "juan@.empresa.com",
            "juan@empresa.com.",
            "juan perez@empresa.com",
        ] {
            assert!(
                EmailAddress::parse(entrada).is_err(),
                "debería rechazarse: {entrada:?}"
            );
        }
    }

    /// THREAT_MODEL.md §4.2: un valor con CRLF que llegue a una cabecera SMTP
    /// permite inyectar cabeceras arbitrarias.
    #[test]
    fn rechaza_inyeccion_de_cabeceras() {
        for entrada in [
            "juan@empresa.com\r\nBcc: victima@otra.com",
            "juan@empresa.com\nSubject: secuestrado",
            "juan\r\n@empresa.com",
            "\r\nBcc: victima@otra.com",
        ] {
            assert!(
                EmailAddress::parse(entrada).is_err(),
                "debería rechazarse: {entrada:?}"
            );
        }
    }

    /// Un CR o LF **rodeando** la dirección no es un ataque: es un archivo con
    /// finales de línea CRLF, que es el caso normal al importar un CSV de
    /// Windows. Se recorta y la dirección se acepta limpia.
    ///
    /// Lo que importa es que el valor almacenado no contenga el carácter, que
    /// es lo que la comprobación de inyección protege de verdad.
    #[test]
    fn recorta_saltos_de_linea_alrededor_sin_rechazar() {
        for entrada in [
            "juan@empresa.com\r",
            "\r\njuan@empresa.com\r\n",
            "\tjuan@empresa.com \n",
        ] {
            let e = EmailAddress::parse(entrada)
                .unwrap_or_else(|_| panic!("debería aceptarse: {entrada:?}"));
            assert_eq!(e.normalized(), "juan@empresa.com");
            assert!(!e.raw().contains(['\r', '\n']));
        }
    }

    #[test]
    fn rechaza_direcciones_desmesuradas() {
        let larga = format!("{}@empresa.com", "a".repeat(400));
        assert!(EmailAddress::parse(&larga).is_err());
    }

    #[test]
    fn acepta_direcciones_validas_poco_comunes() {
        for entrada in [
            "juan+etiqueta@empresa.com.mx",
            "j@a.io",
            "nombre_apellido-123@sub.dominio.com",
        ] {
            assert!(
                EmailAddress::parse(entrada).is_ok(),
                "debería aceptarse: {entrada:?}"
            );
        }
    }
}
