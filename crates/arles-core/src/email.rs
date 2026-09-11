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
///
/// **La igualdad se decide por `normalized`, no por `raw`.** Con la comparación
/// derivada, `VENTAS@empresa.com` y `ventas@EMPRESA.com` eran valores distintos,
/// así que meterlos en un `HashSet` daba dos entradas y la deduplicación fallaba
/// justo donde el documento afirma que funciona.
#[derive(Clone)]
pub struct EmailAddress {
    raw: String,
    normalized: String,
}

impl PartialEq for EmailAddress {
    fn eq(&self, otro: &Self) -> bool {
        self.normalized == otro.normalized
    }
}

impl Eq for EmailAddress {}

impl std::hash::Hash for EmailAddress {
    fn hash<H: std::hash::Hasher>(&self, estado: &mut H) {
        self.normalized.hash(estado);
    }
}

/// Se serializa como la cadena original, no como una estructura de dos campos.
///
/// Así `normalized` no puede llegar desde fuera contradiciendo a `raw`.
impl Serialize for EmailAddress {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.raw)
    }
}

/// Toda deserialización pasa por [`EmailAddress::parse`].
///
/// Con la implementación derivada, una entrada JSON podía construir directamente
/// una dirección cuyo `raw` contuviera CRLF —la carga de inyección de cabeceras
/// que los tests daban por imposible— o cuyo `normalized` no correspondiera a
/// `raw`. Validar solo en el constructor no sirve si hay otra puerta de entrada.
impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
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

    /// La parte del dominio, ya validada.
    ///
    /// Se expone para construir cabeceras como `Message-Id`, donde interpolar un
    /// dominio sin validar permitiría inyectar cabeceras SMTP.
    pub fn dominio(&self) -> &str {
        // `parse` garantizó que hay exactamente una arroba.
        self.normalized
            .split_once('@')
            .map_or(&self.normalized, |(_, d)| d)
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

    /// Hallazgo F3 de la revisión de la Fase 1.
    ///
    /// Con la igualdad derivada sobre `raw`, dos escrituras de la misma
    /// dirección eran valores distintos y la deduplicación fallaba en cualquier
    /// `HashSet` o `HashMap` — justo donde el documento afirma que funciona.
    #[test]
    fn la_igualdad_se_decide_por_la_forma_normalizada() {
        use std::collections::HashSet;

        let a = EmailAddress::parse("VENTAS@empresa.com").expect("válida");
        let b = EmailAddress::parse("ventas@EMPRESA.com").expect("válida");

        assert_eq!(
            a, b,
            "dos escrituras de la misma dirección deben ser iguales"
        );

        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(
            set.len(),
            1,
            "el HashSet no dedujo que son la misma dirección"
        );
    }

    #[test]
    fn direcciones_distintas_siguen_siendo_distintas() {
        let a = EmailAddress::parse("ana@empresa.com").expect("válida");
        let b = EmailAddress::parse("beto@empresa.com").expect("válida");
        assert_ne!(a, b);
    }

    /// Hallazgo F4. Validar solo en el constructor no sirve si serde abre otra
    /// puerta: la implementación derivada aceptaba `raw` con CRLF y un
    /// `normalized` que no correspondía.
    #[test]
    fn deserializar_pasa_por_la_validacion() {
        // La forma antigua —estructura de dos campos— ya no se acepta.
        let estructura: Result<EmailAddress, _> = serde_json::from_str(
            r#"{"raw":"juan@x.com\r\nBcc: victima@y.com","normalized":"otra@cosa.com"}"#,
        );
        assert!(
            estructura.is_err(),
            "serde aceptó una dirección sin validar"
        );

        // Una cadena con inyección tampoco pasa.
        let inyeccion: Result<EmailAddress, _> =
            serde_json::from_str(r#""juan@x.com\r\nBcc: victima@y.com""#);
        assert!(inyeccion.is_err(), "serde aceptó CRLF en la dirección");
    }

    #[test]
    fn la_serializacion_va_y_vuelve() {
        let original = EmailAddress::parse("Ana.Perez@Empresa.com").expect("válida");
        let json = serde_json::to_string(&original).expect("serializa");
        assert_eq!(json, "\"Ana.Perez@Empresa.com\"");

        let vuelta: EmailAddress = serde_json::from_str(&json).expect("deserializa");
        assert_eq!(vuelta.raw(), original.raw());
        assert_eq!(vuelta.normalized(), original.normalized());
    }

    #[test]
    fn el_dominio_se_extrae_normalizado() {
        let e = EmailAddress::parse("Ana@Empresa.COM").expect("válida");
        assert_eq!(e.dominio(), "empresa.com");
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
