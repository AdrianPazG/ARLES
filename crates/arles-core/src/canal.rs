//! Canales de contacto, y el valor que identifica a alguien en cada uno.
//!
//! Decisión **L-2** de `documentacion/01-producto/LOGISTICA_DE_CAMPANAS.md`,
//! autorizada por Dirección el 16 de septiembre de 2026.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ CAMBIA, EN UNA FRASE
//!
//! Hasta aquí **un contacto era un correo**. A partir de aquí un contacto
//! **tiene canales**: un correo, un móvil, o los dos.
//!
//! Suena a matiz y no lo es. Antes de esto **un contacto no tenía dónde
//! guardar un móvil**: la dirección eran dos columnas de `contact` y sólo cabía
//! una. Ése era el bloqueo para WhatsApp, y no —como llegué a escribir— que el
//! segundo envío chocara con la clave de unicidad de los intentos; esa clave es
//! (campaña, dirección), y un correo y un móvil son direcciones distintas.
//!
//! El canal entra igualmente en esa clave, por una razón más modesta: hace que
//! diga lo que significa, y sostiene la garantía si dos canales llegaran a
//! compartir la misma cadena.
//! ─────────────────────────────────────────────────────────────────────────

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::email::EmailAddress;
use crate::error::CoreError;

/// Por dónde se le escribe a un contacto.
///
/// **Cerrado a propósito.** Cada canal que se añada obliga a revisar la
/// supresión, el consentimiento, los límites y la unicidad de los intentos; un
/// canal que entrara como cadena libre se colaría en la base sin pasar por
/// ninguna de esas cuatro decisiones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Canal {
    Correo,
    WhatsApp,
}

impl Canal {
    /// Cómo se escribe en la base de datos. Es un contrato con el esquema:
    /// las `CHECK` de las migraciones contienen literalmente estas cadenas.
    pub const fn como_texto(self) -> &'static str {
        match self {
            Self::Correo => "email",
            Self::WhatsApp => "whatsapp",
        }
    }

    /// Todos los canales. Lo usan las pruebas y cualquier recorrido exhaustivo.
    pub const TODOS: &'static [Self] = &[Self::Correo, Self::WhatsApp];

    /// Lee lo que hay guardado en la base.
    ///
    /// # Errores
    ///
    /// [`CoreError::CanalDesconocido`] si la cadena no es de las dos. Una fila
    /// con un canal que no reconocemos es un dato corrupto, no un caso a
    /// ignorar: decidiría a quién se le escribe.
    pub fn desde_texto(texto: &str) -> Result<Self, CoreError> {
        match texto {
            "email" => Ok(Self::Correo),
            "whatsapp" => Ok(Self::WhatsApp),
            _ => Err(CoreError::CanalDesconocido),
        }
    }
}

impl fmt::Display for Canal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_texto())
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Tope defensivo. Un número internacional no pasa de 15 dígitos (E.164), más
/// el «+». Lo demás es una celda de un XLSX ajeno que no es un teléfono.
const MAX_DIGITOS: usize = 15;

/// Mínimo razonable. Por debajo no es un número al que se pueda escribir; es
/// una extensión, un código o un resto de otra columna.
const MIN_DIGITOS: usize = 8;

/// Un número de móvil en forma canónica E.164.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA TRAMPA MEXICANA DEL «1», Y POR QUÉ SE GUARDA SIN ÉL
///
/// México unificó su marcación el 3 de agosto de 2019 y dejó de usar el «1»
/// que distinguía a los móviles. En E.164, un móvil mexicano es hoy
/// **+52 seguido de 10 dígitos**, doce en total.
///
/// WhatsApp, en cambio, arrastra ese «1» en algunos sitios: hay proveedores e
/// integraciones donde el mismo teléfono aparece como `+521…`, trece dígitos.
/// El resultado conocido es que **la misma persona entra dos veces** — con y
/// sin el «1»— y la deduplicación, la supresión y el «no le escribas dos
/// veces» se caen a la vez, porque las tres se apoyan en comparar este valor.
///
/// => Aquí se guarda **siempre la forma canónica sin el «1»**. Si algún día el
/// adaptador de WhatsApp necesita mandarlo con «1» por el cable, es asunto del
/// adaptador y de una sola función; lo que **no** puede pasar es que dos formas
/// del mismo teléfono convivan en la base.
///
/// Por el mismo motivo, cuando llegue un aviso de Meta con un identificador que
/// traiga el «1», hay que pasarlo por aquí antes de buscar a quién corresponde.
/// ─────────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct PhoneNumber {
    raw: String,
    normalized: String,
}

/// Enseña la forma canónica, no la original. Un `Debug` acaba en una bitácora,
/// y ahí la útil es la que se compara; duplicar el dato personal en dos formas
/// no ayuda a diagnosticar nada.
impl fmt::Debug for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.normalized)
    }
}

impl PartialEq for PhoneNumber {
    fn eq(&self, otro: &Self) -> bool {
        self.normalized == otro.normalized
    }
}

impl Eq for PhoneNumber {}

impl std::hash::Hash for PhoneNumber {
    fn hash<H: std::hash::Hasher>(&self, estado: &mut H) {
        self.normalized.hash(estado);
    }
}

/// Se serializa como la cadena original, igual que [`EmailAddress`]: así
/// `normalized` no puede llegar desde fuera contradiciendo a `raw`.
impl Serialize for PhoneNumber {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for PhoneNumber {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::parse(&s, "MX").map_err(serde::de::Error::custom)
    }
}

impl PhoneNumber {
    /// Valida y normaliza un número.
    ///
    /// `pais_por_defecto` es el de la empresa, y **sólo se usa cuando el número
    /// viene sin prefijo internacional**. Un número que ya trae `+` manda sobre
    /// él: una lista importada puede tener clientes de fuera.
    ///
    /// # Errores
    ///
    /// [`CoreError::TelefonoInvalido`] si está vacío, no tiene dígitos
    /// suficientes, se pasa de largo, trae caracteres que no son de un teléfono,
    /// o viene sin prefijo desde un país que todavía no sabemos completar.
    pub fn parse(entrada: &str, pais_por_defecto: &str) -> Result<Self, CoreError> {
        let raw = entrada.trim();
        if raw.is_empty() {
            return Err(CoreError::TelefonoInvalido {
                motivo: "el número está vacío",
            });
        }

        // Lo que la gente escribe de verdad: «+52 (81) 1234-5678», «81 1234
        // 5678», «5581234567». Se admite la puntuación y se tira; lo que NO se
        // admite es una letra, porque eso ya no es un número mal escrito sino
        // otra cosa metida en la columna del teléfono.
        let internacional = raw.starts_with('+') || raw.starts_with("00");
        let mut digitos = String::new();
        for c in raw.chars() {
            match c {
                '0'..='9' => digitos.push(c),
                '+' | ' ' | '-' | '(' | ')' | '.' | '\u{a0}' => {}
                _ => {
                    return Err(CoreError::TelefonoInvalido {
                        motivo: "el número contiene caracteres que no son de un teléfono",
                    });
                }
            }
        }

        // «00» es el prefijo internacional de marcación de media Europa y de
        // buena parte de Latinoamérica. Equivale al «+».
        if raw.starts_with("00") {
            digitos.drain(..2);
        }

        if digitos.len() < MIN_DIGITOS {
            return Err(CoreError::TelefonoInvalido {
                motivo: "el número tiene menos dígitos de los que necesita un teléfono",
            });
        }
        if digitos.len() > MAX_DIGITOS {
            return Err(CoreError::TelefonoInvalido {
                motivo: "el número supera los 15 dígitos que admite el formato internacional",
            });
        }

        let con_pais = if internacional {
            digitos
        } else {
            // Sin prefijo, hay que ponerlo, y el único que sabemos poner es el
            // de México. Adivinar el de otro país sería peor que negarse: un
            // número al que se escribe mal no da error, **le llega a otra
            // persona**.
            match pais_por_defecto {
                "MX" => format!("52{digitos}"),
                _ => {
                    return Err(CoreError::TelefonoInvalido {
                        motivo: "el número viene sin prefijo internacional y no sabemos cuál poner",
                    });
                }
            }
        };

        Ok(Self {
            raw: raw.to_owned(),
            normalized: format!("+{}", canonizar(&con_pais)),
        })
    }

    /// Lo que escribió el usuario, para poder mostrarlo tal cual.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// La forma canónica E.164, que es la clave de todo lo que compara.
    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

/// Quita el «1» mexicano cuando sobra. Ver la nota grande de [`PhoneNumber`].
///
/// Sólo actúa sobre `52` + `1` + 10 dígitos, que es la forma exacta que produce
/// la trampa. Un `+521…` de otra longitud no se toca: no sabemos qué es, y
/// recortarlo a ciegas sería inventarse un teléfono.
fn canonizar(digitos: &str) -> String {
    if let Some(resto) = digitos.strip_prefix("521")
        && resto.len() == 10
    {
        return format!("52{resto}");
    }
    digitos.to_owned()
}

// ─────────────────────────────────────────────────────────────────────────────

/// Un canal con su valor: por dónde se le escribe a alguien, y a qué dirección.
///
/// Es lo que se guarda en `contact_channel`. El tipo junta las dos cosas a
/// propósito: un canal sin valor no sirve para nada, y un valor sin canal no se
/// sabe cómo validar — un correo y un móvil no se normalizan igual, y hasta
/// ahora el esquema tenía una sola columna que daba por hecho que era correo.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ValorDeCanal {
    Correo(EmailAddress),
    WhatsApp(PhoneNumber),
}

impl ValorDeCanal {
    /// Valida `valor` **según el canal que se diga**.
    ///
    /// # Errores
    ///
    /// Lo que devuelva el validador del canal correspondiente.
    pub fn parse(canal: Canal, valor: &str, pais_por_defecto: &str) -> Result<Self, CoreError> {
        match canal {
            Canal::Correo => EmailAddress::parse(valor).map(Self::Correo),
            Canal::WhatsApp => PhoneNumber::parse(valor, pais_por_defecto).map(Self::WhatsApp),
        }
    }

    pub fn canal(&self) -> Canal {
        match self {
            Self::Correo(_) => Canal::Correo,
            Self::WhatsApp(_) => Canal::WhatsApp,
        }
    }

    /// Lo que escribió el usuario.
    pub fn raw(&self) -> &str {
        match self {
            Self::Correo(e) => e.raw(),
            Self::WhatsApp(t) => t.raw(),
        }
    }

    /// La forma normalizada. **Es la clave de la deduplicación, de la supresión
    /// y de la unicidad de los intentos**, y por eso vive en el tipo y no en
    /// cada consulta que quiera compararla.
    pub fn normalized(&self) -> &str {
        match self {
            Self::Correo(e) => e.normalized(),
            Self::WhatsApp(t) => t.normalized(),
        }
    }
}

impl fmt::Debug for ValorDeCanal {
    /// Enseña el canal y la forma normalizada, nunca en bruto: un `Debug` de
    /// estos acaba en una bitácora, y ahí no hace falta el original.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.canal(), self.normalized())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_canal_va_y_vuelve_de_su_texto() {
        for canal in Canal::TODOS {
            assert_eq!(Canal::desde_texto(canal.como_texto()), Ok(*canal));
        }
    }

    /// Las cadenas de `como_texto` están escritas dentro de las `CHECK` de la
    /// migración V3. Si alguien las cambia aquí, la base rechaza las filas y el
    /// fallo aparece lejísimos de la causa.
    #[test]
    fn los_textos_del_canal_son_los_del_esquema() {
        assert_eq!(Canal::Correo.como_texto(), "email");
        assert_eq!(Canal::WhatsApp.como_texto(), "whatsapp");
    }

    #[test]
    fn un_canal_que_no_existe_se_rechaza() {
        assert_eq!(Canal::desde_texto("sms"), Err(CoreError::CanalDesconocido));
        assert_eq!(Canal::desde_texto(""), Err(CoreError::CanalDesconocido));
        // Mayúsculas incluidas: la base guarda minúsculas y nada más.
        assert_eq!(
            Canal::desde_texto("EMAIL"),
            Err(CoreError::CanalDesconocido)
        );
    }

    // ── El teléfono ──

    #[test]
    fn un_movil_mexicano_de_diez_digitos_recibe_su_prefijo() {
        let t = PhoneNumber::parse("8112345678", "MX").expect("válido");
        assert_eq!(t.normalized(), "+528112345678");
        assert_eq!(t.raw(), "8112345678");
    }

    #[test]
    fn la_puntuacion_que_escribe_la_gente_se_admite_y_se_tira() {
        for escrito in [
            "+52 (81) 1234-5678",
            "+52 81 1234 5678",
            "+52-81-1234-5678",
            "0052 81 1234 5678",
            "81 1234 5678",
            "81.1234.5678",
        ] {
            let t = PhoneNumber::parse(escrito, "MX").expect(escrito);
            assert_eq!(t.normalized(), "+528112345678", "escrito como «{escrito}»");
        }
    }

    /// **La prueba que justifica todo este módulo.**
    ///
    /// Con y sin el «1» son la misma persona. Si no lo fueran, la misma persona
    /// entraría dos veces en la lista, recibiría dos veces la misma campaña, y
    /// una supresión pedida sobre una de las dos formas no protegería de la
    /// otra.
    #[test]
    fn el_uno_mexicano_no_crea_una_segunda_persona() {
        let con = PhoneNumber::parse("+5218112345678", "MX").expect("válido");
        let sin = PhoneNumber::parse("+528112345678", "MX").expect("válido");

        assert_eq!(con.normalized(), "+528112345678");
        assert_eq!(con, sin, "con «1» y sin «1» tienen que ser el mismo número");

        use std::collections::HashSet;
        let conjunto: HashSet<_> = [con, sin].into_iter().collect();
        assert_eq!(conjunto.len(), 1, "la deduplicación los cuenta como uno");
    }

    /// El recorte es quirúrgico: sólo `52` + `1` + diez dígitos. Un número que
    /// empiece por 521 y tenga otra longitud no se toca, porque no sabemos qué
    /// es y recortarlo sería inventarse un teléfono al que le llega a alguien.
    #[test]
    fn el_recorte_del_uno_no_se_aplica_a_cualquier_cosa() {
        let otro = PhoneNumber::parse("+5211234567", "MX").expect("válido");
        assert_eq!(otro.normalized(), "+5211234567");
    }

    #[test]
    fn un_numero_de_fuera_conserva_su_prefijo() {
        let t = PhoneNumber::parse("+1 415 555 0132", "MX").expect("válido");
        assert_eq!(t.normalized(), "+14155550132");
    }

    /// El país de la empresa **no pisa** un prefijo escrito. Una lista importada
    /// puede traer clientes de fuera, y ponerles 52 delante los mandaría a un
    /// número mexicano que existe y es de otra persona.
    #[test]
    fn el_pais_por_defecto_no_pisa_un_prefijo_escrito() {
        let t = PhoneNumber::parse("+14155550132", "MX").expect("válido");
        assert!(t.normalized().starts_with("+1"));
    }

    #[test]
    fn sin_prefijo_y_sin_pais_conocido_se_rechaza() {
        let e = PhoneNumber::parse("8112345678", "AR");
        assert!(matches!(e, Err(CoreError::TelefonoInvalido { .. })));
    }

    #[test]
    fn lo_que_no_es_un_telefono_se_rechaza() {
        for malo in ["", "   ", "no tiene", "1234", "ext. 4021"] {
            assert!(
                matches!(
                    PhoneNumber::parse(malo, "MX"),
                    Err(CoreError::TelefonoInvalido { .. })
                ),
                "«{malo}» debería rechazarse"
            );
        }
    }

    #[test]
    fn un_numero_desmesurado_se_rechaza() {
        let largo = format!("+{}", "9".repeat(MAX_DIGITOS + 1));
        assert!(matches!(
            PhoneNumber::parse(&largo, "MX"),
            Err(CoreError::TelefonoInvalido { .. })
        ));
    }

    // ── El valor de canal ──

    #[test]
    fn cada_canal_valida_con_su_propia_regla() {
        let correo = ValorDeCanal::parse(Canal::Correo, "Ana@Empresa.MX", "MX").expect("válido");
        assert_eq!(correo.normalized(), "ana@empresa.mx");
        assert_eq!(correo.canal(), Canal::Correo);

        let wa = ValorDeCanal::parse(Canal::WhatsApp, "81 1234 5678", "MX").expect("válido");
        assert_eq!(wa.normalized(), "+528112345678");
        assert_eq!(wa.canal(), Canal::WhatsApp);
    }

    /// Un teléfono en la columna del correo no cuela, y un correo en la del
    /// teléfono tampoco. Con una sola columna de texto los dos pasaban.
    #[test]
    fn un_valor_no_vale_para_cualquier_canal() {
        assert!(ValorDeCanal::parse(Canal::Correo, "8112345678", "MX").is_err());
        assert!(ValorDeCanal::parse(Canal::WhatsApp, "ana@empresa.mx", "MX").is_err());
    }

    /// El `Debug` acaba en bitácoras. Que enseñe la forma normalizada y el
    /// canal basta para diagnosticar, y evita duplicar el dato personal.
    #[test]
    fn el_debug_dice_el_canal_y_el_valor_normalizado() {
        let wa = ValorDeCanal::parse(Canal::WhatsApp, "+52 81 1234 5678", "MX").expect("válido");
        assert_eq!(format!("{wa:?}"), "whatsapp:+528112345678");
    }
}
