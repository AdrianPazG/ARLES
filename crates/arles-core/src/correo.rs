//! La cuenta de correo con la que se envía, validada.
//!
//! Sigue el patrón del resto del núcleo: un borrador con lo que escribió el
//! usuario, y unos datos validados que **sólo salen de aquí**. Así no hay forma
//! de que un servidor sin revisar llegue a la base de datos ni al adaptador.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE NO ESTÁ AQUÍ: LA CONTRASEÑA
//!
//! No es un olvido. La contraseña vive en el llavero del sistema (ADR-0011) y
//! nunca pasa por estos tipos, para que se puedan guardar, enseñar, registrar y
//! pintar en un `{:?}` sin tener que pensárselo. Lo que se guarda en la base es
//! una referencia opaca al llavero, no un secreto.

use serde::{Deserialize, Serialize};

use crate::email::EmailAddress;
use crate::error::CoreError;

/// Longitud máxima de un nombre de servidor. Es el tope del DNS.
pub const MAX_SERVIDOR: usize = 253;

/// Longitud máxima del usuario y del nombre visible.
pub const MAX_TEXTO: usize = 320;

/// Cómo viaja la conexión con el servidor de correo.
///
/// ─────────────────────────────────────────────────────────────────────────
/// **DOS VARIANTES, Y LAS DOS CIFRAN. NO HAY UNA TERCERA.**
///
/// Esto no es una lista de opciones a la que un día se le añade «ninguno»: es
/// el §29 escrito como tipo. Mientras este `enum` tenga sólo estas dos
/// variantes, **no existe ningún valor que represente una conexión sin
/// cifrar**, y por tanto no hay nada que un formulario, una migración o un
/// archivo de configuración puedan poner para desactivarla.
///
/// La misma regla está escrita otras dos veces, a propósito: en el `CHECK` de
/// la columna `smtp_security` (migración V7) y en `arles_email::tls`, que no
/// tiene forma de construir parámetros sin cifrado. Tres sitios para una regla
/// parece mucho hasta que se piensa en el día en que alguien «arregle» uno.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CifradoDeCorreo {
    /// Se conecta en claro y se cifra antes de decir nada sensible. Puerto 587.
    /// Es lo que usa Gmail.
    Starttls,
    /// Cifrado desde el primer byte. Puerto 465.
    Directo,
}

impl CifradoDeCorreo {
    /// El puerto que usa casi todo el mundo con este cifrado.
    #[must_use]
    pub const fn puerto_habitual(self) -> u16 {
        match self {
            Self::Starttls => 587,
            Self::Directo => 465,
        }
    }

    /// Cómo se guarda en la base. **Es el valor del `CHECK` de la V7.**
    #[must_use]
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Starttls => "starttls",
            Self::Directo => "directo",
        }
    }

    /// Lee lo que había guardado.
    ///
    /// # Errores
    ///
    /// [`CoreError::CifradoDesconocido`] si la fila trae otra cosa. **No se
    /// repara cayendo en un valor por defecto**: una fila corrupta que dijera
    /// «ninguno» acabaría eligiendo el «más parecido», y el más parecido a
    /// «ninguno» no existe. Se para y se dice.
    pub fn desde_str(s: &str) -> Result<Self, CoreError> {
        match s {
            "starttls" => Ok(Self::Starttls),
            "directo" => Ok(Self::Directo),
            _ => Err(CoreError::CifradoDesconocido),
        }
    }

    /// La clave del texto que se enseña (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Starttls => "correo.cifrado.starttls",
            Self::Directo => "correo.cifrado.directo",
        }
    }
}

/// Los campos del formulario de una cuenta de correo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampoDeCuenta {
    Servidor,
    Puerto,
    Cifrado,
    Usuario,
    Remitente,
    NombreVisible,
}

impl CampoDeCuenta {
    /// Nombre del campo tal como lo conoce el formulario.
    #[must_use]
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Servidor => "servidor",
            Self::Puerto => "puerto",
            Self::Cifrado => "cifrado",
            Self::Usuario => "usuario",
            Self::Remitente => "remitente",
            Self::NombreVisible => "nombreVisible",
        }
    }
}

/// Un campo inválido, con la clave del texto que la interfaz debe mostrar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDeCampoDeCuenta {
    pub campo: CampoDeCuenta,
    pub clave: &'static str,
}

/// Lo que el formulario envía: texto sin validar.
///
/// **El puerto llega como texto**, y es a propósito. Si fuera un número, un
/// «58a» rompería la deserialización entera y la respuesta sería un error de
/// formato sin campo: el formulario no podría marcar en rojo la casilla del
/// puerto, que es justo lo que hay que hacer. Como texto, un puerto mal escrito
/// es un error de **ese** campo, como cualquier otro.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorradorDeCuenta {
    pub servidor: String,
    pub puerto: String,
    pub cifrado: String,
    pub usuario: String,
    pub remitente: String,
    /// Opcional. Cadena vacía y ausente son lo mismo.
    #[serde(default)]
    pub nombre_visible: String,
}

/// Los datos de la cuenta, ya validados. **Sin la contraseña.**
///
/// Los campos son privados: la única puerta de entrada es [`Self::validar`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosDeLaCuenta {
    servidor: String,
    puerto: u16,
    cifrado: CifradoDeCorreo,
    usuario: String,
    remitente: EmailAddress,
    nombre_visible: Option<String>,
}

impl<'de> Deserialize<'de> for DatosDeLaCuenta {
    /// **Un tipo validado no se deserializa directamente.**
    ///
    /// Pasa por el borrador y por `validar`. Si se pudiera construir desde
    /// JSON, la webview podría mandar una cuenta con un servidor que nadie
    /// revisó —o un cifrado que no es ninguno de los dos— y llegaría a la base
    /// sin que nada fallara.
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let borrador = BorradorDeCuenta::deserialize(d)?;
        Self::validar(&borrador).map_err(|errores| {
            let campos = errores
                .iter()
                .map(|e| e.campo.como_str())
                .collect::<Vec<_>>()
                .join(", ");
            serde::de::Error::custom(format!("cuenta de correo inválida: {campos}"))
        })
    }
}

impl DatosDeLaCuenta {
    /// Valida el borrador entero y devuelve **todos** los campos malos.
    ///
    /// Todos, no el primero: corregir un formulario de seis campos de uno en
    /// uno, con un viaje a la validación por cada intento, es la clase de cosa
    /// que hace que alguien se rinda y escriba la cuenta en un papel.
    ///
    /// # Errores
    ///
    /// La lista de campos inválidos, cada uno con su clave de texto.
    pub fn validar(b: &BorradorDeCuenta) -> Result<Self, Vec<ErrorDeCampoDeCuenta>> {
        let mut errores = Vec::new();

        let servidor = b.servidor.trim().to_lowercase();
        if servidor.is_empty() {
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::Servidor,
                clave: "correo.error.servidorVacio",
            });
        } else if servidor.chars().count() > MAX_SERVIDOR {
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::Servidor,
                clave: "correo.error.servidorLargo",
            });
        } else if servidor.contains(char::is_whitespace) || !servidor.contains('.') {
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::Servidor,
                clave: "correo.error.servidorNoVale",
            });
        } else if parece_una_ip(&servidor) {
            // LA REGLA QUE NO ES OBVIA.
            //
            // Un certificado se emite para un NOMBRE. Llamando por IP, la
            // validación no puede cuadrar nunca, y el arreglo que encuentra
            // todo el mundo es desactivar la validación — que es exactamente lo
            // que este producto no permite (§29). Así que se corta aquí, donde
            // todavía se puede explicar.
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::Servidor,
                clave: "correo.error.servidorEsUnaIp",
            });
        }

        // El puerto: 0 no es un puerto, y por encima de 65535 no existe.
        let puerto = match b.puerto.trim().parse::<u16>() {
            Ok(p) if p > 0 => Some(p),
            _ => {
                errores.push(ErrorDeCampoDeCuenta {
                    campo: CampoDeCuenta::Puerto,
                    clave: "correo.error.puertoNoVale",
                });
                None
            }
        };

        let cifrado = match CifradoDeCorreo::desde_str(b.cifrado.trim()) {
            Ok(c) => Some(c),
            Err(_) => {
                errores.push(ErrorDeCampoDeCuenta {
                    campo: CampoDeCuenta::Cifrado,
                    clave: "correo.error.cifradoNoVale",
                });
                None
            }
        };

        let usuario = b.usuario.trim().to_owned();
        if usuario.is_empty() {
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::Usuario,
                clave: "correo.error.usuarioVacio",
            });
        } else if usuario.chars().count() > MAX_TEXTO {
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::Usuario,
                clave: "correo.error.usuarioLargo",
            });
        }

        let remitente = match EmailAddress::parse(&b.remitente) {
            Ok(d) => Some(d),
            Err(_) => {
                errores.push(ErrorDeCampoDeCuenta {
                    campo: CampoDeCuenta::Remitente,
                    clave: "correo.error.remitenteNoVale",
                });
                None
            }
        };

        let nombre = b.nombre_visible.trim();
        if nombre.chars().count() > MAX_TEXTO {
            errores.push(ErrorDeCampoDeCuenta {
                campo: CampoDeCuenta::NombreVisible,
                clave: "correo.error.nombreLargo",
            });
        }

        // Se desempaquetan los tres opcionales SIN `expect`.
        //
        // Si `errores` está vacío, los tres son `Some` — pero eso es un
        // razonamiento sobre el código de arriba, no algo que el compilador
        // garantice, y un `expect` convierte un cambio futuro en un programa
        // que se cierra solo. El `else` es inalcanzable hoy y no entra en
        // pánico: deja un error con su campo, para que el formulario marque
        // algo en vez de quedarse sin decir nada.
        match (puerto, cifrado, remitente) {
            (Some(puerto), Some(cifrado), Some(remitente)) if errores.is_empty() => Ok(Self {
                servidor,
                puerto,
                cifrado,
                usuario,
                remitente,
                nombre_visible: if nombre.is_empty() {
                    None
                } else {
                    Some(nombre.to_owned())
                },
            }),
            _ => {
                if errores.is_empty() {
                    errores.push(ErrorDeCampoDeCuenta {
                        campo: CampoDeCuenta::Servidor,
                        clave: "correo.error.generico",
                    });
                }
                Err(errores)
            }
        }
    }

    /// Reconstruye lo que ya está guardado, sin pasar por el formulario.
    ///
    /// Es la puerta de la base de datos, y valida igual: una fila editada por
    /// fuera —o dejada a medias por una migración— no entra sin revisar.
    ///
    /// # Errores
    ///
    /// La lista de campos inválidos.
    pub fn desde_lo_guardado(
        servidor: &str,
        puerto: u16,
        cifrado: &str,
        usuario: &str,
        remitente: &str,
        nombre_visible: Option<&str>,
    ) -> Result<Self, Vec<ErrorDeCampoDeCuenta>> {
        Self::validar(&BorradorDeCuenta {
            servidor: servidor.to_owned(),
            puerto: puerto.to_string(),
            cifrado: cifrado.to_owned(),
            usuario: usuario.to_owned(),
            remitente: remitente.to_owned(),
            nombre_visible: nombre_visible.unwrap_or_default().to_owned(),
        })
    }

    #[must_use]
    pub fn servidor(&self) -> &str {
        &self.servidor
    }

    #[must_use]
    pub const fn puerto(&self) -> u16 {
        self.puerto
    }

    #[must_use]
    pub const fn cifrado(&self) -> CifradoDeCorreo {
        self.cifrado
    }

    #[must_use]
    pub fn usuario(&self) -> &str {
        &self.usuario
    }

    #[must_use]
    pub const fn remitente(&self) -> &EmailAddress {
        &self.remitente
    }

    #[must_use]
    pub fn nombre_visible(&self) -> Option<&str> {
        self.nombre_visible.as_deref()
    }
}

/// ¿Esto es una dirección IP en vez de un nombre?
///
/// No hace falta un analizador completo: basta con reconocer las dos formas con
/// las que alguien escribe una IP a mano. Un IPv4 son cuatro números separados
/// por puntos; un IPv6 lleva dos puntos, que en un nombre de máquina no
/// aparecen nunca.
fn parece_una_ip(s: &str) -> bool {
    if s.contains(':') {
        return true;
    }
    let trozos: Vec<&str> = s.split('.').collect();
    trozos.len() == 4 && trozos.iter().all(|t| t.parse::<u8>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn borrador_bueno() -> BorradorDeCuenta {
        BorradorDeCuenta {
            servidor: "smtp.gmail.com".to_owned(),
            puerto: "587".to_owned(),
            cifrado: "starttls".to_owned(),
            usuario: "ventas@telemetry.mx".to_owned(),
            remitente: "ventas@telemetry.mx".to_owned(),
            nombre_visible: "TELEMETRY".to_owned(),
        }
    }

    #[test]
    fn una_cuenta_bien_escrita_se_valida() {
        let d = DatosDeLaCuenta::validar(&borrador_bueno()).expect("es válida");
        assert_eq!(d.servidor(), "smtp.gmail.com");
        assert_eq!(d.puerto(), 587);
        assert_eq!(d.cifrado(), CifradoDeCorreo::Starttls);
        assert_eq!(d.nombre_visible(), Some("TELEMETRY"));
    }

    /// **El servidor no puede ser una IP.**
    ///
    /// Es la regla menos obvia del módulo y la que más importa: un certificado
    /// se emite para un nombre, así que con una IP la validación no cuadra
    /// nunca — y el arreglo que encuentra cualquiera es desactivarla.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando la rama de `parece_una_ip`,
    /// `64.233.160.1` se acepta como servidor y esta prueba falla.
    #[test]
    fn una_ip_no_es_un_servidor() {
        for ip in ["64.233.160.1", "192.168.1.1", "2001:4860:4860::8888", "::1"] {
            let b = BorradorDeCuenta {
                servidor: ip.to_owned(),
                ..borrador_bueno()
            };
            let errores = DatosDeLaCuenta::validar(&b).expect_err(&format!("{ip} no vale"));
            assert!(
                errores.iter().any(|e| e.campo == CampoDeCuenta::Servidor),
                "{ip} tendría que marcar el campo del servidor: {errores:?}"
            );
        }
    }

    /// **No hay ningún texto que signifique «sin cifrar»** (§29).
    ///
    /// Es la regla escrita como tipo. Si algún día `CifradoDeCorreo` ganara una
    /// tercera variante, esta prueba no la caza sola — por eso la misma regla
    /// está también en el `CHECK` de la V7 y en el adaptador.
    #[test]
    fn ningun_valor_significa_sin_cifrar() {
        for intento in [
            "", "none", "ninguno", "sin", "plain", "insecure", "no", "tls",
        ] {
            assert!(
                CifradoDeCorreo::desde_str(intento).is_err(),
                "«{intento}» no puede ser un cifrado válido"
            );
        }
        // Y los dos que sí valen, cifran los dos.
        assert_eq!(
            CifradoDeCorreo::desde_str("starttls").expect("vale"),
            CifradoDeCorreo::Starttls
        );
        assert_eq!(
            CifradoDeCorreo::desde_str("directo").expect("vale"),
            CifradoDeCorreo::Directo
        );
    }

    /// Los puertos imposibles se marcan en **su** campo, no revientan la
    /// petición.
    #[test]
    fn el_puerto_se_valida_como_un_campo_mas() {
        for malo in ["0", "-1", "65536", "99999", "", "587a", "  ", "5 8 7"] {
            let b = BorradorDeCuenta {
                puerto: malo.to_owned(),
                ..borrador_bueno()
            };
            let errores = DatosDeLaCuenta::validar(&b).expect_err(&format!("«{malo}» no vale"));
            assert!(
                errores.iter().any(|e| e.campo == CampoDeCuenta::Puerto),
                "«{malo}» tendría que marcar el puerto: {errores:?}"
            );
        }
    }

    /// **Se devuelven todos los campos malos de una vez.**
    ///
    /// Corregir seis campos de uno en uno, con un viaje por cada intento, es
    /// cómo se consigue que alguien apunte la cuenta en un papel y se rinda.
    #[test]
    fn se_marcan_todos_los_campos_malos_a_la_vez() {
        let b = BorradorDeCuenta {
            servidor: String::new(),
            puerto: "0".to_owned(),
            cifrado: "ninguno".to_owned(),
            usuario: String::new(),
            remitente: "esto no es una dirección".to_owned(),
            nombre_visible: "x".repeat(MAX_TEXTO + 1),
        };
        let errores = DatosDeLaCuenta::validar(&b).expect_err("todo está mal");
        let campos: Vec<_> = errores.iter().map(|e| e.campo).collect();
        for esperado in [
            CampoDeCuenta::Servidor,
            CampoDeCuenta::Puerto,
            CampoDeCuenta::Cifrado,
            CampoDeCuenta::Usuario,
            CampoDeCuenta::Remitente,
            CampoDeCuenta::NombreVisible,
        ] {
            assert!(campos.contains(&esperado), "falta {esperado:?}: {campos:?}");
        }
    }

    /// **Un tipo validado no entra por JSON: pasa por `validar`.**
    ///
    /// Si no pasara, la webview podría mandar una cuenta que nadie revisó —un
    /// servidor que es una IP, con el que la validación del certificado no
    /// puede cuadrar— y llegaría a la base.
    ///
    /// ── POR QUÉ SE MIRA EL MENSAJE Y NO SÓLO QUE FALLE ──
    ///
    /// La primera versión sólo comprobaba `is_err()`, y **no cazaba nada**. Se
    /// vio al romperla: poniéndole un `#[derive(Deserialize)]` a
    /// `DatosDeLaCuenta` —la rotura exacta contra la que se escribió— la prueba
    /// seguía pasando, porque el JSON fallaba igual por los tipos de los campos
    /// (el borrador manda el puerto como texto y el derivado lo espera número).
    /// Fallaba, sí, pero por el motivo equivocado.
    ///
    /// Mirando el mensaje se distingue: sólo el camino que pasa por `validar`
    /// dice qué campo está mal, que es además lo que el formulario necesita
    /// para marcarlo. Verificada rompiéndola el 23/09/2026 en los dos sentidos.
    #[test]
    fn no_se_puede_deserializar_una_cuenta_sin_validar() {
        // Con la forma del borrador —todo texto— y una IP por servidor: encaja
        // en el JSON y la validación tiene que rechazarlo igualmente.
        let json = r#"{
            "servidor": "64.233.160.1",
            "puerto": "587",
            "cifrado": "starttls",
            "usuario": "ventas@telemetry.mx",
            "remitente": "ventas@telemetry.mx",
            "nombreVisible": ""
        }"#;
        let error = serde_json::from_str::<DatosDeLaCuenta>(json)
            .expect_err("una IP por servidor no puede entrar");
        let dicho = error.to_string();
        assert!(
            dicho.contains("cuenta de correo inválida") && dicho.contains("servidor"),
            "tiene que fallar EN LA VALIDACIÓN y nombrar el campo, no por el \
             tipo del JSON: {dicho}"
        );
    }

    /// Lo que se guardó se puede volver a leer, y sigue pasando la validación.
    #[test]
    fn lo_guardado_se_relee_igual() {
        let d = DatosDeLaCuenta::validar(&borrador_bueno()).expect("válida");
        let otra = DatosDeLaCuenta::desde_lo_guardado(
            d.servidor(),
            d.puerto(),
            d.cifrado().como_str(),
            d.usuario(),
            d.remitente().raw(),
            d.nombre_visible(),
        )
        .expect("lo guardado se relee");
        assert_eq!(d, otra);
    }

    /// Una fila con un cifrado corrupto **se para**, no se repara.
    #[test]
    fn una_fila_con_un_cifrado_corrupto_no_se_repara_sola() {
        let r = DatosDeLaCuenta::desde_lo_guardado(
            "smtp.gmail.com",
            587,
            "ninguno",
            "ventas@telemetry.mx",
            "ventas@telemetry.mx",
            None,
        );
        assert!(r.is_err(), "no se puede elegir «el más parecido» a ninguno");
    }

    #[test]
    fn cada_cifrado_tiene_su_clave_y_su_puerto() {
        assert_eq!(CifradoDeCorreo::Starttls.puerto_habitual(), 587);
        assert_eq!(CifradoDeCorreo::Directo.puerto_habitual(), 465);
        assert_ne!(
            CifradoDeCorreo::Starttls.clave_i18n(),
            CifradoDeCorreo::Directo.clave_i18n()
        );
    }
}
