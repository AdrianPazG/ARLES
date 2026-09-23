//! El adaptador de verdad: el que habla con Gmail.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ LA CONVERSACIÓN SE LLEVA PASO A PASO
//!
//! `lettre` trae una función que hace el envío entero de una vez. Es cómoda y
//! **no se usa**, por una razón concreta: cuando falla, dice «el servidor
//! contestó 550» y **no dice en qué momento**.
//!
//! Eso, aquí, no es un detalle. Un `550` contestando a `RCPT TO` significa «esa
//! dirección no existe» → esa persona acaba en la lista de bajas, para siempre
//! y en todas las campañas. El mismo `550` contestando a `MAIL FROM` significa
//! «tu cuenta no puede enviar desde esa dirección» → el problema es de la
//! cuenta, y no tiene nada que ver con quien iba a recibir el correo.
//!
//! Sin saber cuál de los dos fue, ARLES daría de baja a un contacto inocente
//! por un problema del remitente. Y una baja **no se deshace**: el contacto deja
//! de recibir y nadie vuelve a mirar por qué.
//!
//! Así que la conversación se lleva a mano —`MAIL FROM`, `RCPT TO`, `DATA`— y
//! cada respuesta se atribuye a quien le corresponde. Es más código; es la
//! diferencia entre una decisión correcta y una irreversible tomada a ciegas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y POR QUÉ ARLES COMPRUEBA EL CIFRADO POR SU CUENTA
//!
//! `tls.rs` configura `lettre` para exigir TLS, y `lettre` lo cumple. Pero eso
//! es confiar en que una biblioteca de terceros haga lo que promete en el
//! asunto donde más caro sale equivocarse: **la contraseña de correo de la
//! empresa**.
//!
//! Antes de mandar `AUTH`, este adaptador pregunta al socket si está cifrado y
//! se niega a continuar si no lo está (§29). No es desconfianza gratuita: es la
//! única comprobación que sigue valiendo si un día `lettre` cambia un valor por
//! defecto, si alguien toca la configuración, o si el servidor negocia algo
//! raro. La prueba `la_contrasena_nunca_sale_en_claro` la mira desde el otro
//! lado: revisa lo que el servidor recibió **antes** de cifrar.

use std::fmt;
use std::time::Duration;

use lettre::address::Address;
use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::client::{SmtpConnection, TlsParameters};
use lettre::transport::smtp::commands::{Data, Mail, Rcpt};
use lettre::transport::smtp::extension::ClientId;

use arles_core::reintento::Fallo;
use arles_motor::proveedor::{Aceptado, Mensaje, Proveedor, recortar_respuesta};

use crate::codigos::{leer_linea, traducir};
use crate::error::ErrorDeCorreo;
use crate::prueba::{Diagnostico, DondeSeRompio};
use crate::tls::{Cifrado, Modo};

/// Cuánto se espera a que el servidor conteste antes de darlo por perdido.
///
/// Sesenta segundos. Sin tope, un servidor que acepta la conexión y calla deja
/// al trabajador colgado para siempre: la campaña no avanza, no falla, y en la
/// pantalla no pasa nada — que es la peor de las tres cosas.
pub const ESPERA_MAXIMA: Duration = Duration::from_secs(60);

/// Los datos de la cuenta de correo. **Sin la contraseña.**
///
/// La contraseña vive en el llavero del sistema (ADR-0011) y se le pasa al
/// adaptador sólo en el momento de construirlo. No está aquí para que esta
/// estructura se pueda guardar, enseñar y registrar sin pensárselo dos veces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cuenta {
    /// El nombre del servidor: `smtp.gmail.com`.
    ///
    /// **El nombre, no una dirección IP.** El certificado se valida contra el
    /// nombre; con una IP la validación no puede cuadrar, y el arreglo fácil
    /// sería desactivarla.
    pub servidor: String,
    /// El puerto. Si se deja en `None`, el que le toque al cifrado elegido.
    pub puerto: Option<u16>,
    /// El usuario con el que se autentica.
    pub usuario: String,
    /// La dirección que verá quien reciba el correo.
    pub remitente: String,
    /// El nombre que acompaña a la dirección, si lo hay.
    pub nombre_visible: Option<String>,
    /// Cómo se cifra. No hay opción de no cifrar.
    pub cifrado: Cifrado,
}

/// Lo que lleva el correo dentro.
///
/// **Provisional hasta la Fase 6.** Ahí llegan las plantillas y el asunto y el
/// cuerpo pasan a construirse por contacto. Hoy son los mismos para toda la
/// campaña, que es lo que hace falta para el envío de prueba de la 5.5, y está
/// aquí en vez de inventado dentro del adaptador para que el cambio de la
/// Fase 6 sea sustituir esta estructura y nada más.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contenido {
    pub asunto: String,
    pub cuerpo: String,
}

/// El adaptador de SMTP.
pub struct ProveedorSmtp {
    servidor: String,
    puerto: u16,
    modo: Modo,
    tls: TlsParameters,
    credenciales: Credentials,
    de: Mailbox,
    dominio_del_remitente: String,
    contenido: Contenido,
    quien_dice_ser: ClientId,
}

/// `Debug` escrito a mano **para que no se escape la contraseña**.
///
/// El derivado imprimiría `credenciales`, y con él la contraseña de correo de
/// la empresa, en cualquier `{:?}` que alguien ponga depurando — y de ahí a un
/// archivo de bitácora o a la salida de una prueba en CI hay un paso. Las
/// credenciales jamás en texto plano ni en bitácoras (ADR-0011).
impl fmt::Debug for ProveedorSmtp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProveedorSmtp")
            .field("servidor", &self.servidor)
            .field("puerto", &self.puerto)
            .field("modo", &self.modo)
            .field("de", &self.de)
            // Ni el usuario ni la contraseña.
            .finish_non_exhaustive()
    }
}

impl ProveedorSmtp {
    /// Prepara el adaptador. **No conecta todavía.**
    ///
    /// La contraseña se recibe aquí, se guarda dentro de las credenciales de
    /// `lettre` y no sale de este objeto: ningún método la devuelve y el
    /// `Debug` no la escribe.
    ///
    /// # Errores
    ///
    /// [`ErrorDeCorreo`] si la cuenta está mal configurada: el servidor no es
    /// un nombre, la dirección del remitente no es una dirección, o el cifrado
    /// no se puede preparar. **Ninguno de estos casos continúa sin cifrar.**
    pub fn nuevo(
        cuenta: &Cuenta,
        contrasena: &str,
        contenido: Contenido,
    ) -> Result<Self, ErrorDeCorreo> {
        let servidor = cuenta.servidor.trim();
        if servidor.is_empty() || servidor.contains(char::is_whitespace) {
            return Err(ErrorDeCorreo::ElServidorNoVale);
        }

        let direccion: Address = cuenta
            .remitente
            .parse()
            .map_err(|_| ErrorDeCorreo::LaDireccionDelRemitenteNoVale)?;
        let dominio_del_remitente = direccion.domain().to_owned();
        let de = Mailbox::new(cuenta.nombre_visible.clone(), direccion);

        // El certificado se valida contra el NOMBRE del servidor. Aquí es donde
        // ese nombre entra en la comprobación.
        let tls = match cuenta.cifrado.parametros(servidor)? {
            lettre::transport::smtp::client::Tls::Required(p)
            | lettre::transport::smtp::client::Tls::Wrapper(p) => p,
            // `Cifrado` no puede producir ningún otro caso: no tiene una
            // variante sin cifrar. Si algún día la tuviera, esto es lo que hay
            // que arreglar, y por eso es un error y no un `_ => sin cifrar`.
            _ => return Err(ErrorDeCorreo::NoSePudoCifrar),
        };

        Ok(Self {
            servidor: servidor.to_owned(),
            puerto: cuenta
                .puerto
                .unwrap_or_else(|| cuenta.cifrado.puerto_habitual()),
            modo: cuenta.cifrado.modo(),
            tls,
            credenciales: Credentials::new(cuenta.usuario.clone(), contrasena.to_owned()),
            de,
            dominio_del_remitente,
            contenido,
            // Quién dice ser ARLES al saludar. El nombre de la máquina, que es
            // lo que espera cualquier servidor; `lettre` lo resuelve solo.
            quien_dice_ser: ClientId::default(),
        })
    }

    /// Conecta, cifra y autentica. **El mismo camino para enviar y para
    /// probar.**
    ///
    /// Está extraído a propósito. Una prueba de conexión con su propio código
    /// podría decir «correcto» mientras los envíos fallan —o al revés—, y en
    /// cuanto eso pasa una vez, la prueba deja de servir para diagnosticar
    /// nada: manda a buscar el problema al lado equivocado.
    ///
    /// Lo que sigue después es lo único que cambia: enviar continúa con
    /// `MAIL FROM`; probar dice adiós y cuelga.
    fn conectar_cifrar_y_autenticar(&self) -> Result<SmtpConnection, DondeSeRompio> {
        // Con TLS directo, el cifrado va desde el primer byte: los parámetros
        // entran ya en la conexión. Con STARTTLS se empieza en claro y se cifra
        // en el paso siguiente — y hasta entonces no sale nada sensible.
        let envoltorio = match self.modo {
            Modo::Directo => Some(&self.tls),
            Modo::Starttls => None,
        };

        let mut conexion = SmtpConnection::connect(
            (self.servidor.as_str(), self.puerto),
            Some(ESPERA_MAXIMA),
            &self.quien_dice_ser,
            envoltorio,
            None,
        )
        .map_err(DondeSeRompio::AlConectar)?;

        if self.modo == Modo::Starttls {
            // Si el servidor no ofrece STARTTLS, esto falla — y eso es lo
            // correcto. **No hay una rama que siga en claro** (§29).
            if let Err(e) = conexion.starttls(&self.tls, &self.quien_dice_ser) {
                conexion.abort();
                return Err(DondeSeRompio::AlCifrar(e));
            }
        }

        // LA COMPROBACIÓN QUE NO SE DELEGA.
        //
        // Se le pregunta al socket, no a la configuración. Si por lo que sea el
        // canal no está cifrado, aquí se corta **antes** de mandar `AUTH`, que
        // es el comando que lleva la contraseña.
        if !conexion.is_encrypted() {
            conexion.abort();
            return Err(DondeSeRompio::ElCanalNoEstabaCifrado);
        }

        if let Err(e) = conexion.auth(&[Mechanism::Plain, Mechanism::Login], &self.credenciales) {
            conexion.abort();
            return Err(DondeSeRompio::AlAutenticar(e));
        }

        Ok(conexion)
    }

    /// «Probar conexión» (§29). **No manda ningún correo.**
    ///
    /// Llega hasta donde llega un envío antes de tocar a nadie: conecta, cifra,
    /// autentica y cuelga. Ni `MAIL FROM`, ni `RCPT TO`, ni `DATA` — el porqué
    /// está en `prueba.rs`, y hay una prueba que lo mira desde el servidor.
    #[must_use]
    pub fn probar(&self) -> Diagnostico {
        match self.conectar_cifrar_y_autenticar() {
            Ok(mut conexion) => {
                // Que el adiós falle no cambia el resultado: ya sabemos todo lo
                // que queríamos saber.
                let _ = conexion.quit();
                Diagnostico::Correcto
            }
            Err(donde) => donde.como_diagnostico(),
        }
    }

    /// Arma el correo en bytes, listo para mandar tras el `DATA`.
    fn armar(&self, mensaje: &Mensaje<'_>, para: Mailbox) -> Result<Vec<u8>, Fallo> {
        let correo = Message::builder()
            .from(self.de.clone())
            .to(para)
            .subject(self.contenido.asunto.clone())
            // El `Message-Id` es la clave de idempotencia que se guardó ANTES
            // de esta llamada (`MOTOR_DE_EJECUCION.md` §2, capa 3). Viaja con
            // el correo para poder rastrearlo, y para poder deduplicar del lado
            // del proveedor el día que tengamos visibilidad allí.
            .message_id(Some(format!(
                "<{}@{}>",
                limpiar_la_clave(mensaje.clave_de_idempotencia),
                self.dominio_del_remitente
            )))
            .body(self.contenido.cuerpo.clone())
            // Que el correo no se pueda armar es un fallo nuestro, no del
            // servidor ni de esa persona. Pasajero: se reintenta y, si sigue,
            // se da por fallido — pero **no da de baja a nadie**, que es lo que
            // hay que evitar a toda costa cuando el problema es de casa.
            .map_err(|_| Fallo::Pasajero)?;

        Ok(correo.formatted())
    }
}

/// Deja la clave de idempotencia en algo que cabe en un `Message-Id`.
///
/// La generamos nosotros y es un UUID, así que en la práctica no cambia nada.
/// Está porque el `Message-Id` va dentro del correo entre `<` y `>`: un
/// carácter raro ahí rompería la cabecera del mensaje para todo el que lo
/// reciba, y confiar en que el valor «ya viene bien» es la clase de suposición
/// que se rompe cuando alguien cambia cómo se generan las claves.
fn limpiar_la_clave(clave: &str) -> String {
    let limpia: String = clave
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'))
        .take(120)
        .collect();
    if limpia.is_empty() {
        "sin-clave".to_owned()
    } else {
        limpia
    }
}

impl Proveedor for ProveedorSmtp {
    fn enviar(&self, mensaje: &Mensaje<'_>) -> Result<Aceptado, Fallo> {
        // La dirección ya viene validada por el núcleo; esto es el cinturón.
        let destino: Address = mensaje
            .para
            .parse()
            .map_err(|_| Fallo::LaDireccionNoExiste)?;
        let correo = self.armar(mensaje, Mailbox::new(None, destino.clone()))?;

        let mut conexion = self
            .conectar_cifrar_y_autenticar()
            .map_err(|donde| donde.como_fallo())?;

        // `MAIL FROM` habla del REMITENTE. Un rechazo aquí no dice nada de
        // quien iba a recibir el correo, y confundirlo lo daría de baja.
        conexion
            .command(Mail::new(Some(self.de.email.clone()), vec![]))
            .map_err(|e| {
                conexion.abort();
                culpa_de_la_cuenta(&e)
            })?;

        // `RCPT TO` habla del DESTINATARIO. **Éste es el único punto donde un
        // rechazo puede acabar en una baja**, y por eso es el único que traduce
        // sin reinterpretar.
        let respuesta_al_destinatario = conexion.command(Rcpt::new(destino, vec![]));
        if let Err(e) = respuesta_al_destinatario {
            conexion.abort();
            return Err(culpa_del_destinatario(&e));
        }

        conexion.command(Data).map_err(|e| {
            conexion.abort();
            culpa_de_la_cuenta(&e)
        })?;

        let acuse = conexion.message(&correo).map_err(|e| {
            conexion.abort();
            // El acuse final es del mensaje entero. Puede traer un rechazo del
            // destinatario que el servidor se guardó hasta aquí —algunos lo
            // hacen—, así que se traduce igual que el `RCPT TO`.
            culpa_del_destinatario(&e)
        })?;

        // Que el adiós falle no cambia nada: el servidor ya aceptó el mensaje.
        let _ = conexion.quit();

        Ok(Aceptado {
            id_del_proveedor: acuse.first_line().map(recortar_respuesta),
        })
    }
}

impl DondeSeRompio {
    /// Traduce a lo que el motor entiende, para un envío de verdad.
    ///
    /// Vive aquí y no junto al diagnóstico porque necesita `culpa_de_la_cuenta`,
    /// que es la regla de este adaptador: **nada de lo que pasa antes del
    /// `RCPT TO` puede dar de baja a nadie**.
    pub(crate) fn como_fallo(&self) -> Fallo {
        match self {
            Self::AlConectar(e) | Self::AlCifrar(e) => culpa_de_la_cuenta(e),
            // El canal no estaba cifrado: se cortó antes de mandar la
            // contraseña. Pasajero, y nunca una baja.
            Self::ElCanalNoEstabaCifrado => Fallo::Pasajero,
            Self::AlAutenticar(e) => match culpa_de_la_cuenta(e) {
                // Un fallo autenticando es de la cuenta por definición, aunque
                // el servidor conteste algo raro: el motor para la cuenta y le
                // pide a quien la configuró que renueve la contraseña.
                Fallo::Pasajero => Fallo::Pasajero,
                _ => Fallo::LaCuentaNoAutentica,
            },
        }
    }
}

/// Traduce un fallo que **no** habla del destinatario.
///
/// Nunca devuelve [`Fallo::LaDireccionNoExiste`]: ése es el que da de baja a una
/// persona, y ningún error de conexión, de cifrado ni de `MAIL FROM` puede
/// justificarlo. Lo peor que sale de aquí es parar la cuenta.
fn culpa_de_la_cuenta(e: &lettre::transport::smtp::Error) -> Fallo {
    match traducir_lo_que_dijo_el_servidor(e) {
        // El servidor habló de la cuenta: se para y se avisa.
        Some(Fallo::LaCuentaNoAutentica) => Fallo::LaCuentaNoAutentica,
        // Cualquier otra cosa —incluido un `550`, que aquí NO es una dirección
        // inexistente— se trata como pasajera. Se reintenta, y si sigue
        // fallando el disyuntor para la cuenta a los cinco seguidos y lo dice.
        _ => Fallo::Pasajero,
    }
}

/// Traduce un fallo que sí habla del destinatario.
///
/// Es el único camino que puede acabar en una baja.
fn culpa_del_destinatario(e: &lettre::transport::smtp::Error) -> Fallo {
    traducir_lo_que_dijo_el_servidor(e).unwrap_or(Fallo::Pasajero)
}

/// Reconstruye la línea que mandó el servidor y la pasa por la traducción.
///
/// `lettre` parte la respuesta en dos: el código, en `status()`, y el texto, en
/// el error de origen. La traducción de `codigos.rs` trabaja sobre la línea
/// entera —porque el código extendido de RFC 3463 vive en el texto—, así que se
/// vuelve a juntar. Es también lo que hace que la tabla de traducción se pruebe
/// una sola vez y valga para los dos caminos.
fn traducir_lo_que_dijo_el_servidor(e: &lettre::transport::smtp::Error) -> Option<Fallo> {
    let codigo = e.status()?;
    let texto = std::error::Error::source(e).map(ToString::to_string);
    let linea = match texto {
        Some(t) => format!("{codigo} {t}"),
        None => codigo.to_string(),
    };
    traducir(&leer_linea(&linea)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prueba::Diagnostico;
    use crate::servidor::Guion;
    use crate::servidor_cifrado::{ComoCifra, NOMBRE_DE_PRUEBA, ServidorCifrado};

    const CONTRASENA: &str = "la-contrasena-de-la-empresa";

    fn contenido() -> Contenido {
        Contenido {
            asunto: "Aviso de ARLES".to_owned(),
            cuerpo: "Buenas tardes.".to_owned(),
        }
    }

    /// Un adaptador apuntando al servidor de mentira, confiando en **su**
    /// certificado y en ninguno más.
    fn adaptador(s: &ServidorCifrado, como: ComoCifra) -> ProveedorSmtp {
        let cifrado = match como {
            ComoCifra::Starttls => Cifrado::starttls(),
            ComoCifra::Directo => Cifrado::directo(),
        }
        .confiando_ademas_en(s.pem());

        ProveedorSmtp::nuevo(
            &Cuenta {
                // El NOMBRE, no la IP: el certificado se emitió para él.
                servidor: NOMBRE_DE_PRUEBA.to_owned(),
                puerto: Some(s.direccion().port()),
                usuario: "ventas@telemetry.mx".to_owned(),
                remitente: "ventas@telemetry.mx".to_owned(),
                nombre_visible: Some("TELEMETRY".to_owned()),
                cifrado,
            },
            CONTRASENA,
            contenido(),
        )
        .expect("la cuenta está bien formada")
    }

    fn mensaje<'a>(para: &'a str, clave: &'a str) -> Mensaje<'a> {
        Mensaje {
            para,
            clave_de_idempotencia: clave,
        }
    }

    // ─── El camino feliz, cifrado ───────────────────────────────────────────

    /// **Un correo entero por STARTTLS, contra un servidor que exige
    /// certificado válido.**
    ///
    /// Es la prueba de que el adaptador funciona sin ninguna concesión: el
    /// servidor presenta un certificado de verdad, ARLES lo valida de verdad, y
    /// la única diferencia con Gmail es de quién está firmado.
    #[test]
    fn un_correo_entero_por_starttls() {
        let s =
            ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);

        let acuse = p
            .enviar(&mensaje("ana@ejemplo.mx", "clave-1"))
            .expect("el servidor lo acepta");

        assert!(
            acuse
                .id_del_proveedor
                .as_deref()
                .is_some_and(|i| i.contains("MENTIRA1")),
            "se guarda lo que contestó el servidor: {:?}",
            acuse.id_del_proveedor
        );
        assert!(s.dijo("MAIL FROM"), "llegó a mandar el remitente");
        assert!(s.dijo("RCPT TO"), "y el destinatario");
    }

    /// El otro puerto: cifrado desde el primer byte.
    #[test]
    fn un_correo_entero_por_tls_directo() {
        let s = ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Directo).expect("arranca");
        let p = adaptador(&s, ComoCifra::Directo);

        p.enviar(&mensaje("ana@ejemplo.mx", "clave-2"))
            .expect("el servidor lo acepta");
        assert!(s.dijo("RCPT TO"));
    }

    // ─── El §29, mirado desde los dos lados ─────────────────────────────────

    /// **La contraseña no sale en claro. Nunca.**
    ///
    /// No se comprueba leyendo la configuración —eso sería creerse lo que
    /// decimos que hacemos—, sino mirando **lo que el servidor recibió antes de
    /// cifrar**. Si `AUTH` apareciera ahí, la contraseña de correo de la
    /// empresa habría viajado legible por la red.
    ///
    /// Verificada rompiéndola el 23/09/2026: adelantando el `auth` delante del
    /// `starttls`, esta prueba falla nombrando la línea `AUTH PLAIN`.
    #[test]
    fn la_contrasena_nunca_sale_en_claro() {
        let s =
            ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);
        p.enviar(&mensaje("ana@ejemplo.mx", "clave-3")).expect("va");

        let en_claro = s.dicho_en_claro();
        assert!(
            en_claro.iter().any(|l| l.eq_ignore_ascii_case("STARTTLS")),
            "en claro sólo se habla hasta pedir cifrado: {en_claro:?}"
        );
        assert!(
            !s.dijo_en_claro("AUTH"),
            "la contraseña salió sin cifrar: {en_claro:?}"
        );
        assert!(
            !s.dijo_en_claro("MAIL FROM") && !s.dijo_en_claro("RCPT TO"),
            "ni las direcciones de los contactos: {en_claro:?}"
        );
        // Y del otro lado: cifrado, sí se autenticó.
        assert!(s.dijo("AUTH"), "pero autenticarse, se autenticó");
    }

    /// **Contra un servidor que no ofrece cifrado, ARLES no envía.**
    ///
    /// Es el caso que decide si el §29 es una regla o una buena intención. Un
    /// servidor mal configurado —o alguien en medio que borra la línea de
    /// `STARTTLS` de la respuesta al `EHLO`— no consigue que la contraseña
    /// salga en claro: consigue que no se mande nada.
    ///
    /// Verificada rompiéndola el 23/09/2026, y con una sorpresa: poniendo
    /// `Tls::Opportunistic` en `tls.rs` esta prueba **sigue pasando**, porque la
    /// comprobación `is_encrypted()` de arriba ataja el envío igualmente. Son
    /// dos defensas independientes, y hace falta tumbar las dos: quitando
    /// además esa comprobación —o adelantando el `auth` delante de ella— esta
    /// prueba falla y la contraseña aparece en `dicho_en_claro()`.
    ///
    /// Que la primera rotura no bastara es la razón de que la segunda
    /// comprobación exista.
    #[test]
    fn contra_un_servidor_sin_cifrado_no_se_envia() {
        let s =
            ServidorCifrado::arrancar(Guion::sin_cifrado(), ComoCifra::Starttls).expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);

        let resultado = p.enviar(&mensaje("ana@ejemplo.mx", "clave-4"));
        assert!(resultado.is_err(), "no se puede enviar sin cifrar");
        assert_eq!(
            resultado.unwrap_err(),
            Fallo::Pasajero,
            "y el fallo no da de baja a nadie: el problema no es de esa persona"
        );
        assert!(
            !s.dijo("AUTH"),
            "y sobre todo: la contraseña no llegó a salir"
        );
        assert!(!s.dijo("RCPT TO"), "ni la dirección del contacto");
    }

    /// **Un certificado que no está en la lista de confianza no vale.**
    ///
    /// Es la otra mitad: no basta con cifrar, tiene que ser quien dice ser. Aquí
    /// se le da al adaptador el certificado de **otro** servidor de mentira, y
    /// tiene que negarse.
    ///
    /// Verificada rompiéndola el 23/09/2026: dándole el PEM del servidor bueno,
    /// el envío pasa y esta prueba falla.
    #[test]
    fn un_certificado_que_no_es_el_suyo_no_vale() {
        let bueno =
            ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");
        let otro =
            ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");

        // Se apunta al servidor «bueno» pero se confía en el certificado del
        // otro. Es, en pequeño, exactamente lo que haría alguien en medio.
        let p = ProveedorSmtp::nuevo(
            &Cuenta {
                servidor: NOMBRE_DE_PRUEBA.to_owned(),
                puerto: Some(bueno.direccion().port()),
                usuario: "ventas@telemetry.mx".to_owned(),
                remitente: "ventas@telemetry.mx".to_owned(),
                nombre_visible: None,
                cifrado: Cifrado::starttls().confiando_ademas_en(otro.pem()),
            },
            CONTRASENA,
            contenido(),
        )
        .expect("la cuenta está bien formada");

        assert!(
            p.enviar(&mensaje("ana@ejemplo.mx", "clave-5")).is_err(),
            "el certificado no lo firma nadie en quien confiemos"
        );
        assert!(!bueno.dijo("AUTH"), "y la contraseña no salió");
    }

    // ─── De quién es la culpa ───────────────────────────────────────────────

    /// **Un `550` al destinatario da de baja. Un `550` al remitente, no.**
    ///
    /// Éste es el motivo entero de llevar la conversación paso a paso, y el
    /// fallo que se evita es irreversible: dar de baja a un contacto sano por
    /// un problema de la cuenta.
    ///
    /// Verificada rompiéndola el 23/09/2026: usando `culpa_del_destinatario`
    /// también para `MAIL FROM`, el segundo caso devuelve `LaDireccionNoExiste`
    /// y esta prueba falla.
    ///
    /// Al hacerlo salió un fallo de la propia prueba: con el guion original
    /// —`550 5.7.1 Sender address rejected`— **seguía pasando** con la
    /// atribución rota, porque el código extendido `5.7.x` ya se traduce solo a
    /// «problema de la cuenta». Es decir, comprobaba la tabla de códigos, no la
    /// atribución. Por eso el guion es ahora un `550` pelado.
    #[test]
    fn un_rechazo_del_remitente_no_da_de_baja_al_contacto() {
        // Al destinatario: esa dirección no existe.
        let s = ServidorCifrado::arrancar(
            Guion {
                al_destinatario: "550 5.1.1 No such user here".to_owned(),
                ..Guion::todo_bien()
            },
            ComoCifra::Starttls,
        )
        .expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);
        assert_eq!(
            p.enviar(&mensaje("nadie@ejemplo.mx", "clave-6")),
            Err(Fallo::LaDireccionNoExiste),
            "un 550 contestando al RCPT TO sí habla de esa persona"
        );

        // El mismo 550, pero al remitente: el problema es de la cuenta.
        let s2 = ServidorCifrado::arrancar(
            Guion {
                // **Sin código extendido a propósito.** Con un `5.7.1` la
                // traducción ya acierta sola —habla de política de la cuenta—,
                // y la prueba pasaría aunque la atribución estuviera rota. Un
                // `550` pelado es lo que contestan de verdad los relés mal
                // configurados, y es el único que distingue las dos cosas.
                al_remitente: "550 Relaying denied".to_owned(),
                ..Guion::todo_bien()
            },
            ComoCifra::Starttls,
        )
        .expect("arranca");
        let p2 = adaptador(&s2, ComoCifra::Starttls);
        let fallo = p2
            .enviar(&mensaje("ana@ejemplo.mx", "clave-7"))
            .expect_err("el servidor rechaza el remitente");
        assert_ne!(
            fallo,
            Fallo::LaDireccionNoExiste,
            "el mismo código en otro momento NO puede dar de baja a nadie"
        );
        assert!(!s2.dijo("RCPT TO"), "ni se llegó a nombrar al contacto");
    }

    /// Un rechazo pasajero del destinatario se reintenta, no se da de baja.
    #[test]
    fn un_450_al_destinatario_es_pasajero() {
        let s = ServidorCifrado::arrancar(
            Guion::rechaza_al_destinatario("450 4.2.1 Mailbox busy, try again"),
            ComoCifra::Starttls,
        )
        .expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);
        assert_eq!(
            p.enviar(&mensaje("ana@ejemplo.mx", "clave-8")),
            Err(Fallo::Pasajero)
        );
    }

    /// Una contraseña rechazada para la cuenta entera, no este envío.
    #[test]
    fn una_contrasena_rechazada_para_la_cuenta() {
        let s = ServidorCifrado::arrancar(Guion::credencial_invalida(), ComoCifra::Starttls)
            .expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);
        assert_eq!(
            p.enviar(&mensaje("ana@ejemplo.mx", "clave-9")),
            Err(Fallo::LaCuentaNoAutentica),
            "esperar no arregla una contraseña: hay que renovarla"
        );
        assert!(
            !s.dijo("RCPT TO"),
            "y no se llegó a nombrar al contacto, así que no gasta su reintento \
             por un problema de la cuenta"
        );
    }

    /// Un servidor que acepta la conexión y cuelga es pasajero, no una baja.
    #[test]
    fn un_servidor_que_cuelga_es_pasajero() {
        let s = ServidorCifrado::arrancar(Guion::se_cae_al_conectar(), ComoCifra::Starttls)
            .expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);
        assert_eq!(
            p.enviar(&mensaje("ana@ejemplo.mx", "clave-10")),
            Err(Fallo::Pasajero),
            "que se caiga la red no dice nada de esa dirección"
        );
    }

    // ─── Configuración ──────────────────────────────────────────────────────

    /// Una cuenta mal configurada falla al construirla, no al enviar.
    #[test]
    fn una_cuenta_mal_configurada_no_llega_a_enviar() {
        let base = Cuenta {
            servidor: "smtp.gmail.com".to_owned(),
            puerto: None,
            usuario: "u".to_owned(),
            remitente: "ventas@telemetry.mx".to_owned(),
            nombre_visible: None,
            cifrado: Cifrado::starttls(),
        };

        assert_eq!(
            ProveedorSmtp::nuevo(
                &Cuenta {
                    servidor: "   ".to_owned(),
                    ..base.clone()
                },
                "x",
                contenido()
            )
            .map(|_| ())
            .unwrap_err(),
            ErrorDeCorreo::ElServidorNoVale
        );

        assert_eq!(
            ProveedorSmtp::nuevo(
                &Cuenta {
                    remitente: "esto no es un correo".to_owned(),
                    ..base.clone()
                },
                "x",
                contenido()
            )
            .map(|_| ())
            .unwrap_err(),
            ErrorDeCorreo::LaDireccionDelRemitenteNoVale
        );

        // Y sin puerto, el que le toca al cifrado.
        let p = ProveedorSmtp::nuevo(&base, "x", contenido()).expect("bien formada");
        assert_eq!(p.puerto, 587, "el de STARTTLS, que es el de Gmail");
    }

    /// **La contraseña no aparece al depurar.**
    ///
    /// Un `{:?}` puesto al depurar acaba en un archivo de bitácora o en la
    /// salida de una prueba en CI. El `Debug` derivado habría impreso las
    /// credenciales enteras (ADR-0011).
    ///
    /// Verificada rompiéndola el 23/09/2026, y conviene decir qué salió: el
    /// `#[derive(Debug)]` **ni siquiera compila** —`TlsParameters` no implementa
    /// `Debug`—, y pintar las credenciales a mano tampoco enseña nada, porque
    /// el `Debug` de `lettre` ya las tapa. O sea que hoy la contraseña la
    /// protegen dos cosas de `lettre`, no este archivo.
    ///
    /// Lo que esta prueba sí caza, y se comprobó rompiéndolo: un campo propio
    /// que guarde la contraseña —lo más fácil del mundo de añadir al ampliar el
    /// adaptador— pintado en el `Debug`. Ahí falla, y ése es el riesgo real.
    #[test]
    fn la_contrasena_no_aparece_al_depurar() {
        let s =
            ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");
        let p = adaptador(&s, ComoCifra::Starttls);
        let pintado = format!("{p:?}");
        assert!(
            !pintado.contains(CONTRASENA),
            "la contraseña se está imprimiendo: {pintado}"
        );
        assert!(
            pintado.contains("smtp") || pintado.contains(NOMBRE_DE_PRUEBA),
            "pero sigue diciendo lo útil para depurar: {pintado}"
        );
    }

    // ── «Probar conexión» · entrega 5.3 ──────────────────────────────────

    /// Una cuenta buena pasa la prueba, por los dos caminos de cifrado.
    #[test]
    fn una_cuenta_buena_pasa_la_prueba() {
        for como in [ComoCifra::Starttls, ComoCifra::Directo] {
            let s = ServidorCifrado::arrancar(Guion::todo_bien(), como).expect("arranca");
            assert_eq!(
                adaptador(&s, como).probar(),
                Diagnostico::Correcto,
                "{como:?}"
            );
        }
    }

    /// **La prueba no toca a ninguna persona.**
    ///
    /// Es la garantía que justifica el módulo entero. Un `MAIL FROM` de prueba
    /// cuenta como intento de envío para algunos servidores, y un `RCPT TO` de
    /// prueba apuntaría a una dirección real — y la respuesta a ese comando es,
    /// en todo ARLES, **el único camino que da de baja a alguien**. Alguien que
    /// configura una cuenta pulsa este botón muchas veces.
    ///
    /// Se mira desde el servidor, no desde el adaptador: lo que importa no es
    /// qué código creemos haber escrito, sino qué comandos llegaron.
    ///
    /// Verificada rompiéndola el 23/09/2026: añadiendo un `MAIL FROM` a
    /// `probar()` —el atajo natural para comprobar también el remitente—, esta
    /// prueba falla nombrando el comando.
    #[test]
    fn probar_no_manda_ningun_correo() {
        let s =
            ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");
        assert_eq!(
            adaptador(&s, ComoCifra::Starttls).probar(),
            Diagnostico::Correcto
        );

        for comando in ["MAIL", "RCPT", "DATA"] {
            assert!(
                !s.dijo(comando),
                "probar la conexión mandó «{comando}», y eso ya es tocar a alguien: {:?}",
                s.dicho()
            );
        }
        // Y sí hizo lo que tenía que hacer: saludar, cifrar y autenticarse.
        assert!(s.dijo("AUTH"), "no llegó a autenticar: {:?}", s.dicho());
    }

    /// **Una contraseña rechazada y un servidor ocupado no dicen lo mismo.**
    ///
    /// Es la distinción que hace útil el botón. Un `535` es «esa contraseña no
    /// sirve» y se arregla generando una contraseña de aplicación; un `421` es
    /// «ahora no puedo» y se arregla esperando. Confundirlos hace que alguien
    /// tire la contraseña que funcionaba para arreglar un problema que no
    /// existía.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que cualquier fallo de
    /// `AUTH` devuelva `LaContrasenaNoSirve` —que es lo que hace el envío, y
    /// ahí está bien—, el caso del `421` falla.
    #[test]
    fn la_contrasena_rechazada_no_se_confunde_con_el_servidor_ocupado() {
        let malo = ServidorCifrado::arrancar(Guion::credencial_invalida(), ComoCifra::Starttls)
            .expect("arranca");
        assert_eq!(
            adaptador(&malo, ComoCifra::Starttls).probar(),
            Diagnostico::LaContrasenaNoSirve
        );

        let ocupado =
            ServidorCifrado::arrancar(Guion::ocupado_al_autenticar(), ComoCifra::Starttls)
                .expect("arranca");
        assert_eq!(
            adaptador(&ocupado, ComoCifra::Starttls).probar(),
            Diagnostico::ElServidorNoEstaDisponible
        );
    }

    /// Un servidor que no ofrece cifrado no pasa la prueba, y la contraseña no
    /// llega a salir.
    ///
    /// El diagnóstico tiene que ser el del cifrado y no otro: si dijera «no se
    /// llegó al servidor», quien lo lee revisaría el puerto para siempre.
    #[test]
    fn un_servidor_sin_cifrado_no_pasa_la_prueba() {
        let s =
            ServidorCifrado::arrancar(Guion::sin_cifrado(), ComoCifra::Starttls).expect("arranca");
        assert_eq!(
            adaptador(&s, ComoCifra::Starttls).probar(),
            Diagnostico::ElCifradoNoSePudoEstablecer
        );
        // Se busca el COMANDO `AUTH`, no el texto de la contraseña.
        //
        // Buscar la contraseña literal es lo que se escribió primero, y era una
        // aserción que no podía fallar nunca: `AUTH PLAIN` manda la credencial
        // en base64, así que el texto no aparece ni cuando el fallo ocurre. Es
        // el mismo patrón que la sonda que buscaba `+521` en una pantalla donde
        // el número se enseña con espacios.
        assert!(
            !s.dijo_en_claro("AUTH"),
            "la contraseña salió sin cifrar al probar: {:?}",
            s.dicho_en_claro()
        );
    }

    /// Un certificado que no es el suyo no pasa la prueba.
    ///
    /// Y el diagnóstico dice que el problema es el cifrado, que es lo que
    /// evita la pregunta «¿puedo desactivarlo?» y su respuesta equivocada.
    #[test]
    fn un_certificado_que_no_cuadra_no_pasa_la_prueba() {
        for como in [ComoCifra::Starttls, ComoCifra::Directo] {
            let suyo = ServidorCifrado::arrancar(Guion::todo_bien(), como).expect("arranca");
            let otro = ServidorCifrado::arrancar(Guion::todo_bien(), como).expect("arranca");

            // Se le da el certificado del OTRO servidor: bien formado, válido,
            // y de alguien que no es con quien se está hablando.
            let cifrado = match como {
                ComoCifra::Starttls => Cifrado::starttls(),
                ComoCifra::Directo => Cifrado::directo(),
            }
            .confiando_ademas_en(otro.pem());

            let p = ProveedorSmtp::nuevo(
                &Cuenta {
                    servidor: NOMBRE_DE_PRUEBA.to_owned(),
                    puerto: Some(suyo.direccion().port()),
                    usuario: "ventas@telemetry.mx".to_owned(),
                    remitente: "ventas@telemetry.mx".to_owned(),
                    nombre_visible: None,
                    cifrado,
                },
                CONTRASENA,
                contenido(),
            )
            .expect("la cuenta está bien formada");

            assert_eq!(
                p.probar(),
                Diagnostico::ElCifradoNoSePudoEstablecer,
                "con {como:?}, un certificado ajeno tiene que salir como problema de cifrado"
            );
        }
    }

    /// **Y no llegar al servidor se sigue diciendo como lo que es.**
    ///
    /// Es la otra mitad de la prueba anterior, y hace falta: la detección del
    /// certificado mira el error de E/S que hay debajo, así que si se pasara
    /// de lista diría «problema de cifrado» ante un puerto cerrado — y quien
    /// lo lee se pondría a mirar certificados teniendo el puerto mal escrito.
    ///
    /// El puerto se toma de un servidor que se apaga justo antes de probar, que
    /// es la forma fiable de tener un puerto donde seguro que no hay nadie.
    #[test]
    fn un_puerto_donde_no_hay_nadie_no_es_un_problema_de_cifrado() {
        let puerto = {
            let s = ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls)
                .expect("arranca");
            s.direccion().port()
        };

        let p = ProveedorSmtp::nuevo(
            &Cuenta {
                servidor: NOMBRE_DE_PRUEBA.to_owned(),
                puerto: Some(puerto),
                usuario: "ventas@telemetry.mx".to_owned(),
                remitente: "ventas@telemetry.mx".to_owned(),
                nombre_visible: None,
                cifrado: Cifrado::starttls(),
            },
            CONTRASENA,
            contenido(),
        )
        .expect("la cuenta está bien formada");

        assert_eq!(p.probar(), Diagnostico::NoSeLlegoAlServidor);
    }

    /// La clave de idempotencia se limpia antes de entrar en el `Message-Id`.
    #[test]
    fn la_clave_se_limpia_antes_de_entrar_en_la_cabecera() {
        assert_eq!(limpiar_la_clave("a1b2-c3.d4_e5"), "a1b2-c3.d4_e5");
        assert_eq!(
            limpiar_la_clave("mala>\r\nSubject: otra cosa"),
            "malaSubjectotracosa",
            "un carácter raro aquí rompería la cabecera para quien reciba el correo"
        );
        assert_eq!(limpiar_la_clave(""), "sin-clave");
        assert_eq!(limpiar_la_clave("<<<>>>"), "sin-clave");
    }
}
