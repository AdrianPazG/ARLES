//! El cliente de WhatsApp Cloud API.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL CONTENIDO SE FIJA AL CONSTRUIR EL PROVEEDOR, NO AL ENVIAR
//!
//! Igual que en `arles-email`: el `Mensaje` del motor sólo lleva el destino y
//! la clave de idempotencia, porque el motor no sabe qué se está mandando. La
//! plantilla y sus huecos se le dan a este proveedor cuando se construye, que
//! es donde sí se sabe.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA DIRECCIÓN DE LA API ENTRA POR PARÁMETRO, Y NO ES UN AGUJERO
//!
//! `base` existe para poder apuntar al servidor de mentira en las pruebas. No
//! es una puerta trasera: quien construye un proveedor ya tiene el token, que
//! es lo único que hay que proteger, y **la única forma de ponerla es desde
//! Rust** — no cruza la frontera de la IPC, no sale de un archivo de
//! configuración y no la puede tocar la webview.
//!
//! La alternativa, un `#[cfg(test)]`, dejaría el camino de producción sin
//! probar: el servidor de mentira vive en este crate pero lo usa `arles-app`
//! para probar el envío de punta a punta, y un `cfg(test)` no se ve desde otro
//! crate.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL TOKEN NO SE IMPRIME NUNCA
//!
//! Por eso este tipo no deriva `Debug`. Un `Debug` derivado lo sacaría en
//! cualquier `{:?}` — una traza, un `panic`, un informe de error— y de ahí a
//! una captura de pantalla hay un paso. Es la misma razón por la que
//! `ProveedorSmtp` tampoco lo deriva.

use std::time::Duration;

use arles_core::reintento::Fallo;
use arles_core::whatsapp::MensajeDePlantilla;
use arles_motor::proveedor::{Aceptado, Mensaje, Proveedor, recortar_respuesta};

use crate::codigos::{ErrorDeMeta, traducir};

/// La versión de la API contra la que se habla.
///
/// Fija y no «la última»: Meta cambia el comportamiento entre versiones, y una
/// campaña no puede empezar a fallar porque hoy salió una versión nueva. Se
/// sube a mano, leyendo qué cambió.
pub const VERSION_DE_LA_API: &str = "v21.0";

/// La dirección de verdad.
pub const GRAPH: &str = "https://graph.facebook.com";

/// Cuánto se espera a que Meta conteste.
///
/// Treinta segundos: la API contesta en menos de uno cuando está bien, y un
/// tiempo largo aquí deja al motor bloqueado en un envío mientras la cola no
/// avanza.
pub const ESPERA_MAXIMA: Duration = Duration::from_secs(30);

/// Habla con Cloud API.
pub struct ProveedorDeWhatsApp {
    base: String,
    phone_number_id: String,
    token: String,
    contenido: MensajeDePlantilla,
    agente: ureq::Agent,
}

/// `Debug` escrito a mano, y ésa es toda la razón de que exista.
///
/// Un `#[derive(Debug)]` sacaría el token en cualquier `{:?}` — una traza, un
/// `panic`, un informe de error— y de ahí a una captura de pantalla hay un
/// paso. Se enseña a dónde se llama y con qué plantilla, que es lo que sirve
/// para diagnosticar, y **nunca la credencial**.
///
/// `finish_non_exhaustive()` pone los puntos suspensivos: quien lea la salida
/// ve que hay campos que no se enseñan, en vez de creer que eso es todo.
impl std::fmt::Debug for ProveedorDeWhatsApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProveedorDeWhatsApp")
            .field("base", &self.base)
            .field("phone_number_id", &self.phone_number_id)
            .field("plantilla", &self.contenido.nombre)
            // El token no.
            .finish_non_exhaustive()
    }
}

impl ProveedorDeWhatsApp {
    /// Construye el proveedor.
    ///
    /// `base` es la dirección de la API: [`GRAPH`] en producción, la del
    /// servidor de mentira en las pruebas.
    #[must_use]
    pub fn nuevo(
        base: &str,
        phone_number_id: &str,
        token: &str,
        contenido: MensajeDePlantilla,
    ) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(ESPERA_MAXIMA))
            // ── UN 400 DE META NO ES UN ERROR DEL CLIENTE: ES LA RESPUESTA ──
            //
            // Por defecto `ureq` convierte cualquier 4xx en un error suyo y
            // **no deja leer el cuerpo**. Con eso, el código de Meta —que viaja
            // dentro del cuerpo de un 400— no se leería nunca, y todos los
            // rechazos caerían en `Pasajero`: ni una baja, ni una cuenta
            // parada, ni una espera respetada.
            //
            // Lo encontró la prueba de punta a punta de `tests/envio.rs`. Las
            // pruebas de `codigos.rs` pasaban perfectamente, porque
            // traducen bien un código que en producción nunca habría llegado.
            .http_status_as_error(false)
            // Sin reintentos del cliente: quien decide si se reintenta es el
            // motor, con la política del §53 y su espera con azar. Dos capas de
            // reintento se multiplican, y la de abajo no sabe nada de la
            // ventana horaria ni del presupuesto de la cuenta.
            .max_idle_connections(0)
            .build();

        Self {
            base: base.trim_end_matches('/').to_owned(),
            phone_number_id: phone_number_id.to_owned(),
            token: token.to_owned(),
            contenido,
            agente: config.into(),
        }
    }

    /// El cuerpo JSON de un envío de plantilla.
    ///
    /// Se escribe a mano y no con `serde_json` porque la forma es fija y
    /// pequeña, y porque así se ve de un vistazo **qué sale hacia Meta** — que
    /// en un canal con datos personales es justo lo que hay que poder auditar
    /// sin seguir tres capas de derivaciones.
    /// `valores` son los de **este contacto** (D-WA, 26/10/2026).
    ///
    /// ── LO QUE ESTO ARREGLA, Y NO FALLABA ──
    ///
    /// Antes se usaban los de `self.contenido`, puestos al construir el
    /// proveedor. Para el envío de prueba —un mensaje suelto— es correcto.
    /// Para una campaña era un defecto silencioso: el proveedor se arma una
    /// vez por etapa, así que **toda la lista habría recibido el mensaje con
    /// los datos de la primera persona**. Meta lo habría aceptado, ARLES lo
    /// habría contado como aceptado, y las cifras saldrían bien.
    ///
    /// El nombre y el idioma siguen saliendo del proveedor: son de la etapa y
    /// no cambian entre contactos. Lo que cambia son los valores.
    fn cuerpo(&self, para: &str, valores: &[String]) -> String {
        let parametros = valores
            .iter()
            .map(|v| format!(r#"{{"type":"text","text":{}}}"#, json_texto(v)))
            .collect::<Vec<_>>()
            .join(",");

        // `components` se omite entero cuando la plantilla no tiene huecos:
        // Meta rechaza un `components` con un `parameters` vacío.
        let componentes = if parametros.is_empty() {
            String::new()
        } else {
            format!(r#","components":[{{"type":"body","parameters":[{parametros}]}}]"#)
        };

        format!(
            r#"{{"messaging_product":"whatsapp","recipient_type":"individual","to":{},"type":"template","template":{{"name":{},"language":{{"code":{}}}{componentes}}}}}"#,
            json_texto(para),
            json_texto(&self.contenido.nombre),
            json_texto(&self.contenido.idioma),
        )
    }
}

/// Los valores que se mandan: los del mensaje si los trae, y si no los del
/// proveedor.
///
/// Las dos puertas hacen falta y no son redundantes. El **envío de prueba**
/// arma un proveedor con su contenido y manda un mensaje sin valores: ahí
/// mandan los del proveedor. Una **campaña** arma el proveedor una vez para
/// toda la etapa y manda los de cada contacto: ahí mandan los del mensaje.
///
/// Un `Contenido::Correo` aquí significaría que la fábrica armó el proveedor
/// equivocado; se cae del lado del proveedor, que es el único contenido que
/// este adaptador tiene y que Meta va a aceptar.
fn valores_de<'a>(
    contenido: &'a arles_motor::proveedor::Contenido<'a>,
    del_proveedor: &'a arles_core::whatsapp::MensajeDePlantilla,
) -> &'a [String] {
    match contenido {
        // **Vacío cae en los del proveedor, y la condición hace falta.**
        //
        // El comentario de arriba lo decía y el código no lo hacía: el envío
        // de prueba manda `valores: &[]` —su contenido va en el proveedor— y
        // llegaba a Meta una plantilla SIN SUS HUECOS. Lo cazaron las dos
        // pruebas de integración del adaptador, no una de módulo: el defecto
        // vivía en la junta.
        //
        // Una plantilla sin huecos tiene los dos lados vacíos, así que caer no
        // cambia nada; con huecos, los del mensaje son los de esta persona y
        // mandan.
        arles_motor::proveedor::Contenido::Whatsapp { valores } if !valores.is_empty() => valores,
        _ => &del_proveedor.valores,
    }
}

impl Proveedor for ProveedorDeWhatsApp {
    fn enviar(&self, mensaje: &Mensaje<'_>) -> Result<Aceptado, Fallo> {
        let url = format!(
            "{}/{}/{}/messages",
            self.base, VERSION_DE_LA_API, self.phone_number_id
        );

        let respuesta = self
            .agente
            .post(&url)
            .header("Authorization", &format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send(
                self.cuerpo(
                    mensaje.para,
                    valores_de(&mensaje.contenido, &self.contenido),
                )
                .as_bytes(),
            );

        let mut r = match respuesta {
            Ok(r) => r,
            Err(ureq::Error::StatusCode(codigo)) => {
                // Sin cuerpo que leer: se decide sólo con el estado HTTP.
                return Err(traducir(&ErrorDeMeta {
                    codigo: 0,
                    subcodigo: None,
                    http: codigo,
                }));
            }
            // No se llegó a hablar con Meta: red caída, DNS, tiempo agotado.
            // **Pasajero, y nunca da de baja**: no sabemos nada del destino.
            Err(_) => return Err(Fallo::Pasajero),
        };

        let http = r.status().as_u16();
        let texto = r.body_mut().read_to_string().unwrap_or_default();

        if (200..300).contains(&http) {
            return Ok(Aceptado {
                id_del_proveedor: extraer(&texto, "\"id\":\"").map(|v| recortar_respuesta(&v)),
            });
        }

        Err(traducir(&ErrorDeMeta {
            codigo: extraer_numero(&texto, "\"code\":").unwrap_or(0),
            subcodigo: extraer_numero(&texto, "\"error_subcode\":"),
            http,
        }))
    }
}

/// Escapa un texto como cadena JSON.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTO ES LO QUE IMPIDE QUE UN NOMBRE ROMPA EL MENSAJE
///
/// Los valores que entran aquí vienen de la lista de contactos, que a su vez
/// viene de un XLSX que no escribimos nosotros. Un contacto llamado
/// `Ana"},"x":{"` sin escapar produciría un JSON con otra forma — y lo que
/// Meta recibiría ya no sería el mensaje que ARLES quiso mandar.
///
/// Se escapa lo que el estándar JSON exige y nada más: comillas, barra
/// invertida y los caracteres de control por debajo del espacio.
fn json_texto(valor: &str) -> String {
    let mut salida = String::with_capacity(valor.len() + 2);
    salida.push('"');
    for c in valor.chars() {
        match c {
            '"' => salida.push_str("\\\""),
            '\\' => salida.push_str("\\\\"),
            '\n' => salida.push_str("\\n"),
            '\r' => salida.push_str("\\r"),
            '\t' => salida.push_str("\\t"),
            c if (c as u32) < 0x20 => salida.push_str(&format!("\\u{:04x}", c as u32)),
            c => salida.push(c),
        }
    }
    salida.push('"');
    salida
}

/// Saca el primer valor de texto que sigue a una marca.
///
/// Lectura a mano y deliberadamente tonta: lo único que se busca en la
/// respuesta de Meta son **un número y un identificador**, y ninguno de los dos
/// decide nada delicado por sí solo —la decisión la toma `traducir` con el
/// código—. Añadir un analizador de JSON completo para eso sería traer una
/// dependencia que procesa texto hostil para leer dos campos.
fn extraer(texto: &str, marca: &str) -> Option<String> {
    let desde = texto.find(marca)? + marca.len();
    let resto = texto.get(desde..)?;
    let hasta = resto.find('"')?;
    resto.get(..hasta).map(str::to_owned)
}

/// Igual, para un número.
fn extraer_numero(texto: &str, marca: &str) -> Option<u32> {
    let desde = texto.find(marca)? + marca.len();
    let resto = texto.get(desde..)?.trim_start();
    let hasta = resto.find(|c: char| !c.is_ascii_digit())?;
    resto.get(..hasta)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contenido() -> MensajeDePlantilla {
        MensajeDePlantilla {
            nombre: "bienvenida".to_owned(),
            idioma: "es_MX".to_owned(),
            valores: vec!["Ana".to_owned(), "TELEMETRY".to_owned()],
            faltaron: vec![],
        }
    }

    fn proveedor(contenido: MensajeDePlantilla) -> ProveedorDeWhatsApp {
        ProveedorDeWhatsApp::nuevo("http://a.localhost:1", "pn-1", "secreto", contenido)
    }

    /// **Un nombre con comillas no cambia la forma del mensaje.**
    ///
    /// Los valores vienen de una lista importada de un archivo ajeno. Sin
    /// escapar, un contacto llamado `Ana"},"x":{"` produciría un JSON distinto
    /// del que ARLES quiso mandar — y **no fallaría aquí**: fallaría en Meta,
    /// o peor, sería aceptado con otro contenido.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `format!("\"{valor}\"")`
    /// en `json_texto`, esta prueba falla.
    #[test]
    fn un_nombre_con_comillas_no_rompe_el_json() {
        let malicioso = MensajeDePlantilla {
            valores: vec![r#"Ana"},"to":"+520000000000"#.to_owned()],
            ..contenido()
        };
        let cuerpo = proveedor(malicioso.clone()).cuerpo("+528112345678", &malicioso.valores);

        // El destino sigue siendo uno solo, y es el que se pidió.
        assert_eq!(
            cuerpo.matches(r#""to":"#).count(),
            1,
            "el nombre del contacto no puede introducir otro destinatario"
        );
        assert!(cuerpo.contains(r#""to":"+528112345678""#));
        // Y las comillas del nombre van escapadas.
        assert!(cuerpo.contains(r#"Ana\"}"#));
    }

    /// **Los saltos de línea y los caracteres de control también se escapan.**
    ///
    /// Una celda de Excel puede traerlos, y un `\n` crudo dentro de una cadena
    /// JSON hace que Meta rechace la petición entera — o sea, un contacto con
    /// un salto de línea en el nombre tumbaría su propio envío.
    #[test]
    fn un_salto_de_linea_no_invalida_el_json() {
        let con_salto = MensajeDePlantilla {
            valores: vec!["Ana\nMaría\u{1}".to_owned()],
            ..contenido()
        };
        let cuerpo = proveedor(con_salto.clone()).cuerpo("+528112345678", &con_salto.valores);
        assert!(cuerpo.contains("Ana\\nMar"), "el salto va escapado");
        assert!(cuerpo.contains("\\u0001"), "y los de control también");
        assert!(!cuerpo.contains('\n'), "no queda ningún salto crudo");
    }

    /// **Una plantilla sin huecos no manda un `components` vacío.**
    ///
    /// Meta rechaza un `components` con `parameters: []`, así que el mensaje se
    /// queda sin enviar por añadir algo que no hacía falta. Y un mensaje fijo
    /// —sin huecos— es perfectamente legítimo.
    #[test]
    fn una_plantilla_sin_huecos_no_manda_componentes() {
        let sin_huecos = MensajeDePlantilla {
            valores: vec![],
            ..contenido()
        };
        let cuerpo = proveedor(sin_huecos.clone()).cuerpo("+528112345678", &sin_huecos.valores);
        assert!(!cuerpo.contains("components"), "{cuerpo}");
        assert!(cuerpo.contains(r#""name":"bienvenida""#));
    }

    /// El cuerpo lleva lo que Meta exige, y el nombre de la plantilla —no su
    /// texto, que ARLES ni siquiera manda.
    #[test]
    fn el_cuerpo_manda_el_nombre_de_la_plantilla_y_no_su_texto() {
        let cuerpo = proveedor(contenido()).cuerpo("+528112345678", &contenido().valores);
        assert!(cuerpo.contains(r#""messaging_product":"whatsapp""#));
        assert!(cuerpo.contains(r#""type":"template""#));
        assert!(cuerpo.contains(r#""name":"bienvenida""#));
        assert!(cuerpo.contains(r#""code":"es_MX""#));
        assert!(cuerpo.contains(r#""text":"Ana""#));
        assert!(cuerpo.contains(r#""text":"TELEMETRY""#));
    }

    /// La lectura de la respuesta saca lo que tiene que sacar, y nada más.
    #[test]
    fn de_la_respuesta_se_sacan_el_id_y_el_codigo() {
        let ok = r#"{"messaging_product":"whatsapp","messages":[{"id":"wamid.ABC"}]}"#;
        assert_eq!(extraer(ok, "\"id\":\""), Some("wamid.ABC".to_owned()));

        let error = r#"{"error":{"message":"x","code":131026,"error_subcode":2494010}}"#;
        assert_eq!(extraer_numero(error, "\"code\":"), Some(131_026));
        assert_eq!(extraer_numero(error, "\"error_subcode\":"), Some(2_494_010));

        // Y ante algo que no tiene la forma esperada, no se inventa nada.
        assert_eq!(extraer_numero("{}", "\"code\":"), None);
        assert_eq!(extraer("no es json", "\"id\":\""), None);
    }
}
