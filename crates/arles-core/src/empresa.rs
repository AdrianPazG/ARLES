//! Datos de la empresa operadora y su validación.
//!
//! Es lo primero que el usuario configura (§25) y lo que gobierna después toda
//! la ejecución: la **zona horaria** de aquí es la que decide si una ventana de
//! envío está abierta, nunca la del sistema operativo. Un portátil que viaja no
//! puede cambiar la política de envío de la empresa.
//!
//! Ver `documentacion/03-arquitectura/MODELO_DE_DATOS.md` y
//! `documentacion/05-diseno/UX_NAVEGACION.md` §4.

use serde::{Deserialize, Serialize};

use crate::email::EmailAddress;

/// Longitud máxima del nombre comercial.
///
/// No es un capricho: el nombre viaja a cabeceras de correo y a la interfaz.
/// Un valor sin tope convierte un campo de texto en un problema de otra capa.
const MAX_NOMBRE: usize = 120;

/// Longitud máxima del sitio web.
const MAX_SITIO: usize = 255;

/// Zonas horarias que ARLES admite en v1.2.0.
///
/// **Es una lista cerrada, y eso es deliberado.** Validar un identificador IANA
/// de verdad exige una base de datos de zonas horarias; aceptar cualquier
/// cadena con pinta de zona dejaría entrar `America/Mexico` —que no existe— y
/// el fallo aparecería meses después, al calcular una ventana de ejecución.
///
/// La interfaz ofrece exactamente esta lista, así que lo que se ve y lo que se
/// acepta **no pueden divergir**: son el mismo dato.
///
/// Cubre las cuatro zonas de México, las de la frontera con horario propio y
/// UTC para pruebas. Añadir países es añadir entradas aquí (v1.3, §8).
pub const ZONAS_SOPORTADAS: &[&str] = &[
    "America/Mexico_City",
    "America/Cancun",
    "America/Merida",
    "America/Monterrey",
    "America/Matamoros",
    "America/Chihuahua",
    "America/Ojinaga",
    "America/Mazatlan",
    "America/Bahia_Banderas",
    "America/Hermosillo",
    "America/Tijuana",
    "UTC",
];

/// Países admitidos en v1.2.0.
///
/// v1.2.0 es despliegue interno de TELEMETRY (D-4), y el marco legal que la
/// aplicación cita —LFPDPPP— es el mexicano. Ofrecer una lista de doscientos
/// países sugeriría un soporte legal que no existe.
pub const PAISES_SOPORTADOS: &[&str] = &["MX"];

/// Qué campo del formulario falló.
///
/// Es un enumerado y no una cadena porque la interfaz tiene que **señalar el
/// campo concreto**: un formulario que dice «hay un error» sin decir dónde
/// obliga a revisarlo entero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampoDeEmpresa {
    NombreComercial,
    Pais,
    ZonaHoraria,
    CorreoCorporativo,
    SitioWeb,
}

impl CampoDeEmpresa {
    /// Nombre del campo tal como lo conoce el formulario.
    #[must_use]
    pub fn como_str(self) -> &'static str {
        match self {
            Self::NombreComercial => "nombreComercial",
            Self::Pais => "pais",
            Self::ZonaHoraria => "zonaHoraria",
            Self::CorreoCorporativo => "correoCorporativo",
            Self::SitioWeb => "sitioWeb",
        }
    }
}

/// Un campo inválido, con la clave del texto que la interfaz debe mostrar.
///
/// **No lleva el mensaje**: lleva la clave (§139). El texto vive en el catálogo
/// de i18n, con sus tres partes (§95).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDeCampo {
    pub campo: CampoDeEmpresa,
    pub clave: &'static str,
}

/// Lo que el formulario envía: texto sin validar.
///
/// Existe como tipo propio para que **no se pueda confundir con los datos
/// buenos**. `DatosDeEmpresa` solo se construye validando esto, así que no hay
/// forma de que un valor sin revisar llegue a la base de datos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorradorDeEmpresa {
    pub nombre_comercial: String,
    pub pais: String,
    pub zona_horaria: String,
    pub correo_corporativo: String,
    /// Opcional. Cadena vacía y ausente son lo mismo.
    #[serde(default)]
    pub sitio_web: String,
}

/// Datos de la empresa, ya validados.
///
/// Los campos son privados: la única puerta de entrada es [`Self::validar`].
/// Es la misma lección de [`crate::EmailAddress`] — validar solo en el
/// constructor no sirve de nada si serde abre una segunda puerta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosDeEmpresa {
    nombre_comercial: String,
    pais: String,
    zona_horaria: String,
    correo_corporativo: EmailAddress,
    sitio_web: Option<String>,
}

impl<'de> Deserialize<'de> for DatosDeEmpresa {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let borrador = BorradorDeEmpresa::deserialize(d)?;
        Self::validar(&borrador).map_err(|errores| {
            let campos = errores
                .iter()
                .map(|e| e.campo.como_str())
                .collect::<Vec<_>>()
                .join(", ");
            serde::de::Error::custom(format!("datos de empresa inválidos: {campos}"))
        })
    }
}

impl DatosDeEmpresa {
    /// Valida un borrador del formulario.
    ///
    /// Devuelve **todos** los campos inválidos, no el primero: un formulario
    /// que reporta un error por intento obliga a enviarlo cinco veces para
    /// enterarse de que hay cinco problemas (UX_WRITING.md §95).
    ///
    /// # Errores
    ///
    /// La lista de campos que no pasan, cada uno con su clave de i18n.
    pub fn validar(b: &BorradorDeEmpresa) -> Result<Self, Vec<ErrorDeCampo>> {
        let mut errores = Vec::new();

        let nombre = b.nombre_comercial.trim().to_owned();
        if nombre.is_empty() {
            errores.push(ErrorDeCampo {
                campo: CampoDeEmpresa::NombreComercial,
                clave: "empresa.error.nombreVacio",
            });
        } else if nombre.chars().count() > MAX_NOMBRE {
            errores.push(ErrorDeCampo {
                campo: CampoDeEmpresa::NombreComercial,
                clave: "empresa.error.nombreLargo",
            });
        } else if nombre.contains(['\r', '\n']) {
            // El nombre acaba en la cabecera `From` de los correos. Un salto de
            // línea ahí es inyección de cabeceras, igual que en la dirección
            // (THREAT_MODEL.md §4.2).
            errores.push(ErrorDeCampo {
                campo: CampoDeEmpresa::NombreComercial,
                clave: "empresa.error.nombreConSaltos",
            });
        }

        let pais = b.pais.trim().to_uppercase();
        if !PAISES_SOPORTADOS.contains(&pais.as_str()) {
            errores.push(ErrorDeCampo {
                campo: CampoDeEmpresa::Pais,
                clave: "empresa.error.paisNoSoportado",
            });
        }

        let zona = b.zona_horaria.trim().to_owned();
        if !ZONAS_SOPORTADAS.contains(&zona.as_str()) {
            errores.push(ErrorDeCampo {
                campo: CampoDeEmpresa::ZonaHoraria,
                clave: "empresa.error.zonaNoSoportada",
            });
        }

        let correo = match EmailAddress::parse(&b.correo_corporativo) {
            Ok(c) => Some(c),
            Err(_) => {
                errores.push(ErrorDeCampo {
                    campo: CampoDeEmpresa::CorreoCorporativo,
                    clave: "empresa.error.correoInvalido",
                });
                None
            }
        };

        let sitio = match validar_sitio(&b.sitio_web) {
            Ok(s) => s,
            Err(clave) => {
                errores.push(ErrorDeCampo {
                    campo: CampoDeEmpresa::SitioWeb,
                    clave,
                });
                None
            }
        };

        // El `match` en vez de un `expect` sobre `correo`: el §138 deniega
        // `expect`, y aquí además no hace falta. Si el correo falló, `errores`
        // no está vacío, así que las dos condiciones son la misma.
        match correo {
            Some(correo_corporativo) if errores.is_empty() => Ok(Self {
                nombre_comercial: nombre,
                pais,
                zona_horaria: zona,
                correo_corporativo,
                sitio_web: sitio,
            }),
            _ => Err(errores),
        }
    }

    #[must_use]
    pub fn nombre_comercial(&self) -> &str {
        &self.nombre_comercial
    }

    #[must_use]
    pub fn pais(&self) -> &str {
        &self.pais
    }

    /// Zona IANA que gobierna **todas** las ventanas de ejecución.
    #[must_use]
    pub fn zona_horaria(&self) -> &str {
        &self.zona_horaria
    }

    #[must_use]
    pub fn correo_corporativo(&self) -> &EmailAddress {
        &self.correo_corporativo
    }

    #[must_use]
    pub fn sitio_web(&self) -> Option<&str> {
        self.sitio_web.as_deref()
    }
}

/// Valida el sitio web opcional.
///
/// Se exige `http://` o `https://` explícito en vez de aceptar `empresa.com` y
/// completarlo: un esquema adivinado es un enlace que algún día abre algo que
/// el usuario no escribió. Y se rechaza cualquier otro esquema —`javascript:`,
/// `file:`, `data:`— porque este valor acaba en un `href`.
fn validar_sitio(entrada: &str) -> Result<Option<String>, &'static str> {
    let s = entrada.trim();
    if s.is_empty() {
        return Ok(None);
    }
    if s.chars().count() > MAX_SITIO {
        return Err("empresa.error.sitioLargo");
    }
    if s.contains(['\r', '\n']) || s.chars().any(char::is_whitespace) {
        return Err("empresa.error.sitioInvalido");
    }
    let resto = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .ok_or("empresa.error.sitioSinEsquema")?;
    // `https://` a secas no es una dirección, y un host sin punto tampoco lo es
    // fuera de una red local.
    if !resto.contains('.') || resto.starts_with('.') || resto.starts_with('/') {
        return Err("empresa.error.sitioInvalido");
    }
    Ok(Some(s.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn borrador_valido() -> BorradorDeEmpresa {
        BorradorDeEmpresa {
            nombre_comercial: "TELEMETRY INSIGHT".into(),
            pais: "MX".into(),
            zona_horaria: "America/Mexico_City".into(),
            correo_corporativo: "hola@telemetry.mx".into(),
            sitio_web: "https://telemetry.mx".into(),
        }
    }

    fn campos_con_error(b: &BorradorDeEmpresa) -> Vec<CampoDeEmpresa> {
        DatosDeEmpresa::validar(b)
            .err()
            .unwrap_or_default()
            .iter()
            .map(|e| e.campo)
            .collect()
    }

    #[test]
    fn un_borrador_correcto_se_valida() {
        let d = DatosDeEmpresa::validar(&borrador_valido()).expect("válido");
        assert_eq!(d.nombre_comercial(), "TELEMETRY INSIGHT");
        assert_eq!(d.zona_horaria(), "America/Mexico_City");
        assert_eq!(d.correo_corporativo().normalized(), "hola@telemetry.mx");
        assert_eq!(d.sitio_web(), Some("https://telemetry.mx"));
    }

    #[test]
    fn el_sitio_web_es_opcional() {
        let mut b = borrador_valido();
        b.sitio_web = "   ".into();
        let d = DatosDeEmpresa::validar(&b).expect("válido sin sitio");
        assert_eq!(d.sitio_web(), None);
    }

    /// Un formulario que reporta un error por intento obliga a enviarlo cinco
    /// veces para enterarse de que hay cinco problemas.
    #[test]
    fn reporta_todos_los_campos_malos_de_una_vez() {
        let b = BorradorDeEmpresa {
            nombre_comercial: "  ".into(),
            pais: "ES".into(),
            zona_horaria: "Europe/Madrid".into(),
            correo_corporativo: "no-es-un-correo".into(),
            sitio_web: "telemetry.mx".into(),
        };
        let campos = campos_con_error(&b);
        assert_eq!(campos.len(), 5, "faltan campos por reportar: {campos:?}");
    }

    /// `America/Mexico` no existe. Una comprobación por forma la aceptaría, y
    /// el fallo aparecería meses después al calcular una ventana de ejecución.
    #[test]
    fn rechaza_zonas_que_parecen_validas_pero_no_existen() {
        for zona in ["America/Mexico", "America/CDMX", "GMT-6", "", "UTC+0"] {
            let mut b = borrador_valido();
            b.zona_horaria = zona.into();
            assert!(
                campos_con_error(&b).contains(&CampoDeEmpresa::ZonaHoraria),
                "debería rechazarse la zona {zona:?}"
            );
        }
    }

    /// THREAT_MODEL.md §4.2: el nombre comercial acaba en la cabecera `From`.
    #[test]
    fn el_nombre_no_admite_saltos_de_linea() {
        let mut b = borrador_valido();
        b.nombre_comercial = "TELEMETRY\r\nBcc: victima@otra.com".into();
        assert!(campos_con_error(&b).contains(&CampoDeEmpresa::NombreComercial));
    }

    /// El sitio acaba en un `href`: un esquema ejecutable ahí es XSS servido
    /// por la propia configuración del usuario.
    #[test]
    fn el_sitio_solo_admite_http_y_https() {
        for sitio in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,<script>",
            "JavaScript:alert(1)",
            "//telemetry.mx",
            "telemetry.mx",
            "https://",
            "https:// telemetry.mx",
        ] {
            let mut b = borrador_valido();
            b.sitio_web = sitio.into();
            assert!(
                campos_con_error(&b).contains(&CampoDeEmpresa::SitioWeb),
                "debería rechazarse el sitio {sitio:?}"
            );
        }
    }

    #[test]
    fn el_nombre_tiene_tope() {
        let mut b = borrador_valido();
        b.nombre_comercial = "a".repeat(MAX_NOMBRE + 1);
        assert!(campos_con_error(&b).contains(&CampoDeEmpresa::NombreComercial));
    }

    #[test]
    fn el_pais_se_acepta_en_minusculas() {
        let mut b = borrador_valido();
        b.pais = "mx".into();
        let d = DatosDeEmpresa::validar(&b).expect("válido");
        assert_eq!(d.pais(), "MX");
    }

    /// La misma lección que `EmailAddress`: validar solo en el constructor no
    /// sirve si serde abre una segunda puerta.
    #[test]
    fn deserializar_pasa_por_la_validacion() {
        let json = r#"{
            "nombreComercial": "TELEMETRY\r\nBcc: victima@otra.com",
            "pais": "MX",
            "zonaHoraria": "Europe/Madrid",
            "correoCorporativo": "hola@telemetry.mx",
            "sitioWeb": ""
        }"#;
        assert!(
            serde_json::from_str::<DatosDeEmpresa>(json).is_err(),
            "serde aceptó datos de empresa sin validar"
        );
    }

    #[test]
    fn la_serializacion_usa_los_nombres_del_formulario() {
        let d = DatosDeEmpresa::validar(&borrador_valido()).expect("válido");
        let json = serde_json::to_string(&d).expect("serializa");
        for campo in [
            "nombreComercial",
            "pais",
            "zonaHoraria",
            "correoCorporativo",
            "sitioWeb",
        ] {
            assert!(json.contains(campo), "falta {campo} en {json}");
        }
    }

    /// Lo que la interfaz ofrece y lo que el núcleo acepta tienen que ser el
    /// mismo dato: si divergen, el usuario elige una zona de la lista y el
    /// núcleo se la rechaza.
    #[test]
    fn toda_zona_soportada_se_acepta() {
        for zona in ZONAS_SOPORTADAS {
            let mut b = borrador_valido();
            b.zona_horaria = (*zona).to_owned();
            assert!(
                DatosDeEmpresa::validar(&b).is_ok(),
                "la zona ofrecida {zona} no se acepta"
            );
        }
    }

    #[test]
    fn cada_campo_tiene_nombre_estable_para_el_formulario() {
        let todos = [
            CampoDeEmpresa::NombreComercial,
            CampoDeEmpresa::Pais,
            CampoDeEmpresa::ZonaHoraria,
            CampoDeEmpresa::CorreoCorporativo,
            CampoDeEmpresa::SitioWeb,
        ];
        for c in todos {
            assert!(!c.como_str().is_empty());
        }
    }
}
