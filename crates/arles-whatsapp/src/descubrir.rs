//! Preguntarle a Meta qué números tiene esta cuenta, con sólo el token.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO EXISTE
//!
//! El alta de un número pedía **seis datos**: teléfono, nombre visible, WABA
//! ID, Phone Number ID, token y modo. Cuatro de los seis son identificadores
//! que sólo existen en el panel de Meta, en tres pantallas distintas, y que
//! hay que copiar sin equivocarse — el WABA ID y el Phone Number ID son dos
//! números largos y parecidos, y copiar uno en el campo del otro da un error
//! que no dice cuál de los dos está mal.
//!
//! El alta de correo pasó de siete campos a dos el 14/10/2026 deduciendo el
//! servidor del dominio (R-01). Aquí no se puede deducir nada de la nada —está
//! escrito en `CLAUDE.md`— pero sí se puede **preguntar**: el token ya
//! autoriza a Meta a decir qué números cuelgan de él.
//!
//! Así que quedan **un dato y una elección**: se pega el token, ARLES trae la
//! lista, y se elige el número. Lo demás viene relleno y verificado por la
//! fuente, que es mejor que verificado por quien copia.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTO NO HACE, Y NO ES UN OLVIDO
//!
//! **No crea nada en Meta.** No da de alta una cuenta de WhatsApp Business, no
//! verifica un número, no pide plantillas. Eso se hace en el panel de Meta y
//! ARLES no tiene por qué poder hacerlo: una aplicación de escritorio que
//! pudiera crear cuentas con el token de alguien es una superficie de ataque
//! que no compensa.
//!
//! **Y no guarda el token.** Entra, se usa para preguntar, y sale. Quien lo
//! guarda es el llavero, al dar de alta el número.

use std::time::Duration;

/// Cuánto se espera a Meta antes de darlo por incontestable.
///
/// Diez segundos: esto corre delante de alguien que acaba de pegar un token y
/// está mirando. Más que eso se lee como que la aplicación se colgó.
const ESPERA: Duration = Duration::from_secs(10);

/// Un número que Meta dice que existe, con todo lo que ARLES necesita.
///
/// Viene entero de Meta. Ningún campo lo teclea nadie, y ésa es la razón de
/// este módulo: un identificador copiado a mano puede estar mal y uno traído
/// de la fuente no.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumeroDescubierto {
    /// El `phone_number_id` de Meta. Es con el que se manda.
    pub phone_number_id: String,
    /// El WABA del que cuelga.
    pub waba_id: String,
    /// El número tal como Meta lo enseña: `+52 55 1234 5678`.
    ///
    /// **Con los espacios de Meta**, no normalizado: lo que se enseña en la
    /// lista tiene que parecerse a lo que se ve en el panel, o quien elija no
    /// reconocerá el suyo. La normalización a E.164 la hace el núcleo al
    /// guardar.
    pub telefono: String,
    /// El nombre verificado que Meta enseña a quien recibe.
    pub nombre: String,
    /// `GREEN`, `YELLOW`, `RED` o `UNKNOWN`, tal como lo manda Meta.
    pub calidad: String,
    /// Cuántos mensajes al día deja mandar Meta. `None` si no lo dice.
    pub tope_diario: Option<u32>,
}

/// Por qué no se pudo traer la lista.
///
/// Son tres y no uno a propósito, por la misma razón que los tres errores del
/// respaldo: los tres impiden seguir, pero **cada uno manda a hacer algo
/// distinto**. Uno a revisar el token, otro a esperar, y el tercero a ir al
/// panel de Meta a dar de alta un número.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoSePudoPreguntar {
    /// Meta no acepta el token: caducado, revocado, o mal pegado.
    ElTokenNoVale,
    /// No se llegó a Meta: sin red, o Meta caída.
    NoSeLlego,
    /// Meta contestó, y este token no tiene ningún número detrás.
    ///
    /// **No es un fallo del token**: es una cuenta sin número de WhatsApp dado
    /// de alta todavía, que es lo que le pasa a casi todo el mundo la primera
    /// vez. Por eso se distingue: mandar a revisar el token a quien lo tiene
    /// bien es una tarde perdida.
    NingunNumero,
}

impl NoSePudoPreguntar {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::ElTokenNoVale => "numeroDeWhatsapp.descubrir.tokenNoVale",
            Self::NoSeLlego => "numeroDeWhatsapp.descubrir.noSeLlego",
            Self::NingunNumero => "numeroDeWhatsapp.descubrir.ningunNumero",
        }
    }
}

/// Pregunta a Meta qué números tiene este token.
///
/// `base` es la raíz de la API: la de Meta en producción, o la del servidor de
/// mentira en las pruebas. Entra como parámetro por la misma razón que en el
/// resto del adaptador — aquí no hay Meta con la que probar.
///
/// # Errores
///
/// [`NoSePudoPreguntar`], con la distinción entre un token que no vale, una
/// red que no llega y una cuenta sin números.
pub fn numeros_del_token(
    base: &str,
    token: &str,
) -> Result<Vec<NumeroDescubierto>, NoSePudoPreguntar> {
    let agente = ureq::Agent::config_builder()
        .timeout_global(Some(ESPERA))
        .build()
        .new_agent();

    // ── PASO 1 · QUÉ WABA AUTORIZA ESTE TOKEN ──
    //
    // `debug_token` es la forma documentada de preguntarle a Meta «¿qué puede
    // hacer este token?». Entre lo que devuelve están los `target_ids` del
    // permiso de gestión de WhatsApp, que son los WABA.
    //
    // La alternativa era pedirle al usuario el WABA ID, que es justo lo que
    // este módulo existe para no pedir.
    let url = format!(
        "{base}/{}/debug_token?input_token={}",
        crate::cloud::VERSION_DE_LA_API,
        token
    );
    let texto = pedir(&agente, &url, token)?;
    let wabas = extraer_ids(&texto);

    if wabas.is_empty() {
        // Meta contestó y el token no autoriza ningún WABA. Es un token que no
        // sirve para esto, no una cuenta sin números: la diferencia se nota en
        // que ni siquiera hay dónde mirar.
        return Err(NoSePudoPreguntar::ElTokenNoVale);
    }

    // ── PASO 2 · QUÉ NÚMEROS CUELGAN DE CADA WABA ──
    let mut todos = Vec::new();
    for waba in wabas {
        let url = format!(
            "{base}/{}/{waba}/phone_numbers?fields=id,display_phone_number,verified_name,quality_rating,messaging_limit_tier",
            crate::cloud::VERSION_DE_LA_API
        );
        // Un WABA que no contesta no tumba los demás: con dos cuentas, que una
        // falle no es razón para no enseñar los números de la otra.
        let Ok(texto) = pedir(&agente, &url, token) else {
            continue;
        };
        todos.extend(leer_numeros(&texto, &waba));
    }

    if todos.is_empty() {
        return Err(NoSePudoPreguntar::NingunNumero);
    }
    Ok(todos)
}

/// Una llamada GET a Meta, con el token en la cabecera.
fn pedir(agente: &ureq::Agent, url: &str, token: &str) -> Result<String, NoSePudoPreguntar> {
    let respuesta = agente
        .get(url)
        .header("Authorization", &format!("Bearer {token}"))
        .call();

    let mut r = match respuesta {
        Ok(r) => r,
        // 400 y 401 son «este token no vale». Cualquier otro estado es un
        // problema de Meta o del camino, y mandar a revisar el token por un
        // 502 es mandar a buscar donde no está.
        Err(ureq::Error::StatusCode(401 | 400)) => return Err(NoSePudoPreguntar::ElTokenNoVale),
        Err(_) => return Err(NoSePudoPreguntar::NoSeLlego),
    };

    r.body_mut()
        .read_to_string()
        .map_err(|_| NoSePudoPreguntar::NoSeLlego)
}

/// Los `target_ids` del permiso de gestión de WhatsApp.
///
/// Se lee a mano y no con `serde_json` por la misma razón que el cuerpo del
/// envío: la forma es fija y pequeña, y así se ve de un vistazo qué se está
/// leyendo de una respuesta que viene de fuera.
fn extraer_ids(texto: &str) -> Vec<String> {
    let Some(desde) = texto.find("whatsapp_business_management") else {
        return Vec::new();
    };
    let Some(resto) = texto.get(desde..) else {
        return Vec::new();
    };
    let Some(inicio) = resto.find("target_ids") else {
        return Vec::new();
    };
    let Some(lista) = resto.get(inicio..).and_then(|t| {
        let a = t.find('[')?;
        let b = t.find(']')?;
        t.get(a + 1..b)
    }) else {
        return Vec::new();
    };

    lista
        .split(',')
        .map(|v| v.trim().trim_matches('"').to_owned())
        .filter(|v| !v.is_empty())
        .collect()
}

/// Los números de la respuesta de `phone_numbers`.
fn leer_numeros(texto: &str, waba: &str) -> Vec<NumeroDescubierto> {
    // Cada número es un objeto dentro de `data`. Se parten por `{` y se lee
    // cada trozo: la respuesta no anida objetos dentro de cada número, así que
    // no hace falta un analizador de verdad.
    texto
        .split('{')
        .filter_map(|trozo| {
            let id = campo(trozo, "id")?;
            Some(NumeroDescubierto {
                phone_number_id: id,
                waba_id: waba.to_owned(),
                telefono: campo(trozo, "display_phone_number").unwrap_or_default(),
                nombre: campo(trozo, "verified_name").unwrap_or_default(),
                // Sin calidad declarada, `UNKNOWN` y no `GREEN`: un semáforo
                // que se pone en verde cuando no ha podido mirar es peor que
                // no tenerlo (misma regla que `estado.rs`).
                calidad: campo(trozo, "quality_rating").unwrap_or_else(|| "UNKNOWN".to_owned()),
                tope_diario: campo(trozo, "messaging_limit_tier")
                    .and_then(|t| tope_del_escalon(&t)),
            })
        })
        .collect()
}

/// El valor de un campo de texto de la respuesta.
fn campo(trozo: &str, nombre: &str) -> Option<String> {
    let patron = format!("\"{nombre}\":\"");
    let desde = trozo.find(&patron)? + patron.len();
    let resto = trozo.get(desde..)?;
    let hasta = resto.find('"')?;
    let v = resto.get(..hasta)?;
    if v.is_empty() {
        None
    } else {
        Some(v.to_owned())
    }
}

/// Cuántos mensajes al día permite cada escalón de Meta.
///
/// Meta no devuelve el número sino el nombre del escalón. La tabla está aquí
/// escrita y no adivinada, y **un escalón que no conocemos devuelve `None`**:
/// inventar un techo alto dejaría mandar de más y Meta rechazaría; inventar
/// uno bajo frenaría una campaña sin motivo.
fn tope_del_escalon(escalon: &str) -> Option<u32> {
    match escalon {
        "TIER_50" => Some(50),
        "TIER_250" => Some(250),
        "TIER_1K" => Some(1_000),
        "TIER_10K" => Some(10_000),
        "TIER_100K" => Some(100_000),
        // `TIER_UNLIMITED` entra aquí a propósito: no hay techo que anotar, y
        // poner un número grande sería inventarse un límite que Meta no pone.
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
mod tests {
    use super::*;

    const DEBUG: &str = r#"{"data":{"app_id":"1","granular_scopes":[
        {"scope":"whatsapp_business_messaging","target_ids":["111"]},
        {"scope":"whatsapp_business_management","target_ids":["111","222"]}]}}"#;

    const NUMEROS: &str = r#"{"data":[
      {"verified_name":"Telemetry","display_phone_number":"+52 55 1234 5678",
       "id":"pn-1","quality_rating":"GREEN","messaging_limit_tier":"TIER_1K"},
      {"verified_name":"Soporte","display_phone_number":"+52 81 8765 4321",
       "id":"pn-2","quality_rating":"YELLOW"}]}"#;

    /// **Los WABA salen del permiso de GESTIÓN, no del de mensajería.**
    ///
    /// Los dos permisos traen `target_ids` y no son los mismos: el de
    /// mensajería puede traer un subconjunto. Leer el primero que aparezca
    /// dejaría números fuera de la lista sin que nada fallara — y quien no vea
    /// el suyo creerá que ARLES no lo soporta.
    #[test]
    fn los_wabas_salen_del_permiso_de_gestion() {
        assert_eq!(extraer_ids(DEBUG), vec!["111".to_owned(), "222".to_owned()]);
    }

    #[test]
    fn un_token_sin_permisos_no_da_ningun_waba() {
        assert!(extraer_ids(r#"{"data":{"app_id":"1"}}"#).is_empty());
    }

    /// **Cada número llega entero y con su WABA.**
    #[test]
    fn los_numeros_llegan_con_todo_lo_que_hace_falta() {
        let v = leer_numeros(NUMEROS, "111");
        assert_eq!(v.len(), 2);

        let [primero, _] = v.as_slice() else {
            panic!("tienen que llegar dos números");
        };
        assert_eq!(primero.phone_number_id, "pn-1");
        assert_eq!(primero.waba_id, "111");
        assert_eq!(primero.telefono, "+52 55 1234 5678");
        assert_eq!(primero.nombre, "Telemetry");
        assert_eq!(primero.calidad, "GREEN");
        assert_eq!(primero.tope_diario, Some(1_000));
    }

    /// **Sin calidad declarada se dice `UNKNOWN`, nunca `GREEN`.**
    ///
    /// Es la misma regla que `estado.rs`: un semáforo que se pone en verde
    /// cuando no ha podido mirar es peor que no tener semáforo, porque se mira
    /// y se cree.
    #[test]
    fn un_numero_sin_calidad_no_se_da_por_bueno() {
        let v = leer_numeros(
            r#"{"data":[{"id":"pn-9","display_phone_number":"+52 1","verified_name":"X"}]}"#,
            "111",
        );
        let [n] = v.as_slice() else {
            panic!("tiene que llegar uno");
        };
        assert_eq!(n.calidad, "UNKNOWN");
        assert_eq!(n.tope_diario, None, "sin escalón no se inventa un techo");
    }

    /// **Un escalón que no conocemos no se convierte en un número.**
    ///
    /// Meta puede añadir escalones. Inventar un techo alto dejaría mandar de
    /// más —y Meta rechazaría—; inventar uno bajo frenaría una campaña sin
    /// motivo. `None` deja que mande el límite de la campaña, que es lo que
    /// alguien escribió a propósito.
    #[test]
    fn un_escalon_desconocido_no_se_adivina() {
        assert_eq!(tope_del_escalon("TIER_1K"), Some(1_000));
        assert_eq!(tope_del_escalon("TIER_DE_MARTE"), None);
        assert_eq!(
            tope_del_escalon("TIER_UNLIMITED"),
            None,
            "sin techo no es un techo grande"
        );
    }

    /// Una respuesta sin números no produce ninguno, y no revienta.
    #[test]
    fn una_respuesta_vacia_no_inventa_numeros() {
        assert!(leer_numeros(r#"{"data":[]}"#, "111").is_empty());
    }

    /// **Los tres motivos mandan a hacer tres cosas distintas**, y por eso son
    /// tres. Si fueran uno, a quien tiene el token bien y no ha dado de alta
    /// ningún número se le mandaría a revisar el token.
    #[test]
    fn los_tres_motivos_tienen_su_texto_y_son_distintos() {
        let claves = [
            NoSePudoPreguntar::ElTokenNoVale.clave_i18n(),
            NoSePudoPreguntar::NoSeLlego.clave_i18n(),
            NoSePudoPreguntar::NingunNumero.clave_i18n(),
        ];
        let distintas: std::collections::HashSet<_> = claves.iter().collect();
        assert_eq!(distintas.len(), 3, "tres motivos, tres textos");
    }
}
