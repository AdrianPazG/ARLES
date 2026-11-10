//! Leer el buzón del remitente para aplicar las bajas (R-25, D-21).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! SÓLO LECTURA, Y ESO NO ES UNA PROMESA: ES EL PROTOCOLO
//!
//! Dirección autorizó el 10/11/2026 leer su buzón con cuatro límites: sólo
//! lectura, sólo las bajas, sólo la dirección de quien escribió, nunca el
//! cuerpo. Los cuatro se podían escribir como un comentario y confiar en que
//! nadie los tocara. Tres de ellos están en algo mejor:
//!
//! 1. **`EXAMINE` y no `SELECT`.** Son el mismo comando con una diferencia: el
//!    primero abre el buzón en modo *read-only* y el servidor **rechaza**
//!    cualquier cambio (RFC 3501 §6.3.2). No es que ARLES no pida borrar nada:
//!    es que si lo pidiera, el servidor diría no.
//! 2. **`BODY.PEEK[…]` y no `BODY[…]`.** La segunda forma **marca el correo
//!    como leído** como efecto secundario de leerlo. Con `PEEK`, el buzón
//!    queda exactamente como estaba, y quien lo abra después verá sus correos
//!    sin leer igual que antes.
//! 3. **No existe ninguna función que escriba.** No hay `STORE`, ni `COPY`, ni
//!    `MOVE`, ni `APPEND`, ni `EXPUNGE` en este módulo, y lo vigila
//!    `el_lector_del_buzon_no_sabe_escribir` en `--fase 1`: busca esas
//!    palabras en el archivo y falla si aparecen.
//!
//! El cuarto —no guardar el cuerpo— no se puede poner en el protocolo, porque
//! hay que leerlo para saber si dice «BAJA». Lo que sí se puede es que **no
//! haya dónde guardarlo**: [`CorreoDelBuzon`] se construye, se le pregunta al
//! núcleo y se tira. Nada de este módulo escribe en disco ni en la base.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE MÓDULO Y NO UN CRATE NUEVO
//!
//! El análisis del 23/10/2026 proponía «un adaptador IMAP en un crate propio
//! con su servidor de mentira, como `arles-email`». Vive aquí dentro, y la
//! razón es que **es el mismo crate**: habla con el servidor de correo de la
//! misma cuenta, con la misma contraseña del llavero, exigiendo el mismo
//! cifrado y traduciendo a los mismos errores. Un crate aparte habría
//! duplicado `tls.rs` y `error.rs` para separar «mandar» de «leer», que es una
//! frontera que no existe en ningún otro sitio del producto.
//!
//! Lo que sí es verdad del plan es el servidor de mentira: está en
//! [`crate::servidor_imap`], y es lo que permite probar esto sin un buzón.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ SE HABLA IMAP A MANO Y NO CON UNA BIBLIOTECA
//!
//! Medido el 10/11/2026: la única biblioteca de IMAP de Rust con uso real
//! —`imap`— está en **`3.0.0-alpha.15`**, y su TLS por defecto es
//! `native-tls`, que validaría con el almacén del sistema y no con el mismo
//! que usa el envío. Meter una dependencia alfa en el camino que **da de baja
//! a personas** no se sostiene.
//!
//! Y lo que hace falta de IMAP es diminuto: cuatro comandos. `rustls` y
//! `webpki-roots` **ya estaban en el árbol** —los trae `lettre` para hablar
//! TLS—, así que esto no añade ni una dependencia ni un byte al instalador.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE LLEGA DE FUERA NO ES DE FIAR
//!
//! Es la regla de `arles-import` aplicada aquí: lo que contesta el servidor
//! viene de la red. Un literal IMAP declara su tamaño —`{2048}`— y nada impide
//! que diga `{4294967295}`, así que hay topes en todo: [`TOPE_DEL_LITERAL`],
//! [`TOPE_DE_LINEA`] y [`TOPE_DE_CORREOS`]. Sin ellos, un servidor hostil
//! —o roto— reserva memoria hasta que la aplicación muere.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::{CertificateDer, ServerName};
use rustls::{ClientConnection, RootCertStore, StreamOwned};

use arles_core::baja_por_correo::CorreoDeVuelta;

use crate::error::ErrorDeCorreo;

/// El puerto de IMAP sobre TLS.
///
/// **Sólo éste.** El 143 es IMAP en claro, y no se ofrece: la contraseña del
/// llavero no sale de este equipo sin cifrar (§29). No hay un `Modo` con dos
/// variantes como en `tls.rs` porque aquí no hay nada que elegir — IMAP no
/// tiene un `STARTTLS` universal que valga la pena, y ofrecerlo sería crear la
/// opción mala.
pub const PUERTO_IMAP_TLS: u16 = 993;

/// Lo más grande que se acepta en un literal del servidor: 64 KiB.
///
/// Un correo de baja son unos cientos de bytes. El tope es generoso a
/// propósito —una respuesta con el original citado puede ser larga— y lo que
/// impide es que `{4294967295}` reserve cuatro gigas.
pub const TOPE_DEL_LITERAL: usize = 64 * 1024;

/// Lo más larga que se acepta una línea de respuesta: 8 KiB.
pub const TOPE_DE_LINEA: usize = 8 * 1024;

/// Cuántos correos se miran en una pasada.
///
/// Si hay más, se miran los más recientes y el resto queda para la pasada
/// siguiente. Mejor que fallar: un buzón con mil correos nuevos no puede dejar
/// sin aplicar las bajas que sí caben.
pub const TOPE_DE_CORREOS: usize = 200;

/// Cuánto se espera a que el servidor conteste.
///
/// El mismo minuto que el envío. Sin tope, un servidor que acepta la conexión
/// y calla deja el hilo colgado para siempre.
pub const ESPERA_MAXIMA: Duration = Duration::from_secs(60);

/// Cuántos bytes del cuerpo se piden.
///
/// Dos kilobytes. Lo que decide una baja está en la **primera línea** del
/// texto, y pedir el correo entero sería traerse el original citado —que puede
/// ser la campaña completa— para tirarlo.
const BYTES_DEL_CUERPO: usize = 2048;

/// Los datos para entrar a leer. **Sin la contraseña.**
///
/// Igual que [`crate::smtp::Cuenta`]: la contraseña vive en el llavero
/// (ADR-0011) y se le pasa al lector sólo al abrir la conexión, así que esta
/// estructura se puede guardar, enseñar y registrar sin pensárselo dos veces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuentaDeLectura {
    /// El nombre del servidor: `imap.gmail.com`.
    ///
    /// **El nombre, no una dirección IP**, por lo mismo que en el envío: el
    /// certificado se valida contra el nombre.
    pub servidor: String,
    /// El puerto, o [`PUERTO_IMAP_TLS`] si no se dice.
    pub puerto: Option<u16>,
    /// El usuario con el que se entra. En todos los proveedores que ARLES
    /// reconoce es la propia dirección de correo.
    pub usuario: String,
}

impl CuentaDeLectura {
    /// El puerto que toca.
    #[must_use]
    pub const fn puerto(&self) -> u16 {
        match self.puerto {
            Some(p) => p,
            None => PUERTO_IMAP_TLS,
        }
    }
}

/// Un correo del buzón, reducido a lo que decide una baja.
///
/// **No es un correo.** No hay destinatarios, ni fecha, ni adjuntos, ni
/// identificador de mensaje, ni el cuerpo entero: sólo lo que
/// `arles_core::baja_por_correo` necesita, más el `uid` para que quien llame
/// pueda no volver a mirarlo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorreoDelBuzon {
    /// El identificador que el servidor le da dentro de este buzón.
    pub uid: u32,
    /// La dirección de quien escribió, ya sin el nombre visible.
    ///
    /// **Es la única que se puede dar de baja.** Ver la nota de
    /// `arles_core::baja_por_correo`.
    pub de: String,
    /// El asunto, tal como venía.
    pub asunto: String,
    /// El principio del texto, desenvuelto de su MIME. Ver [`texto_legible`].
    pub texto: String,
    /// ¿Lo mandó una máquina? Sale de las cabeceras que lo declaran.
    pub automatico: bool,
}

impl CorreoDelBuzon {
    /// Lo que se le pregunta al núcleo.
    #[must_use]
    pub fn como_lo_lee_el_nucleo(&self) -> CorreoDeVuelta<'_> {
        CorreoDeVuelta {
            asunto: &self.asunto,
            cuerpo: &self.texto,
            automatico: self.automatico,
        }
    }
}

/// Las cabeceras que se piden, y ninguna más.
///
/// `From` para saber a quién dar de baja, `Subject` porque es el camino del
/// botón de cancelar suscripción, y las tres de abajo para no dar de baja a
/// nadie por su mensaje de ausencia:
///
/// - `Auto-Submitted` es la del RFC 3834, la que debería bastar.
/// - `X-Autoreply` y `X-Autorespond` las ponen servidores que no la
///   implementan, y en 2026 siguen siendo mayoría de los avisos de vacaciones.
/// - `Precedence: auto_reply` es la convención vieja, y la siguen poniendo
///   listas y robots.
///
/// **No se piden `To`, `Cc`, `Date` ni `Message-ID`**: ninguna cambia el
/// veredicto, y lo que no se pide no se puede usar por descuido.
const CABECERAS: &str = "FROM SUBJECT AUTO-SUBMITTED X-AUTOREPLY X-AUTORESPOND PRECEDENCE";

/// Entra al buzón, lee lo que llegó desde hace `dias` y sale.
///
/// La conexión se abre y se cierra en la misma llamada: un buzón abierto
/// mantiene un cerrojo en el servidor y, en Gmail, cuenta para el tope de
/// conexiones simultáneas de la cuenta.
///
/// # Errores
///
/// - [`ErrorDeCorreo::NoSePudoEntrarAlBuzon`] si no se llega al servidor.
/// - [`ErrorDeCorreo::NoSePudoCifrar`] si el certificado no valida.
/// - [`ErrorDeCorreo::ElBuzonRechazoLaContrasena`] si el servidor dice que no al
///   `LOGIN`. Es la misma que el envío, porque es la misma contraseña y el
///   mismo arreglo.
/// - [`ErrorDeCorreo::ElBuzonContestoAlgoRaro`] si el servidor contesta algo que
///   no se entiende, o más de lo que cabe en los topes.
pub fn correos_recientes(
    cuenta: &CuentaDeLectura,
    contrasena: &str,
    dias: u16,
) -> Result<Vec<CorreoDelBuzon>, ErrorDeCorreo> {
    let socket = conectar(cuenta)?;
    let cifrado = cifrar(socket, &cuenta.servidor, None)?;
    let mut sesion = Sesion::nueva(cifrado)?;
    let correos = sesion.leer(cuenta, contrasena, dias);
    // El `LOGOUT` se manda pase lo que pase: dejar la conexión abierta tras un
    // fallo gasta una de las que el servidor concede.
    sesion.despedirse();
    correos
}

/// Lo mismo, confiando además en un certificado concreto.
///
/// **Sólo lo usan las pruebas**, y la forma importa: no desactiva la
/// verificación, **añade una raíz**. El certificado del servidor tiene que
/// estar firmado por ésta y el nombre tiene que cuadrar, igual que con Gmail.
/// Es la misma decisión que `Cifrado::confiando_ademas_en` para el envío —ahí
/// está explicado por qué una llamada que acepte certificados inválidos «sólo
/// en los tests» acaba moviéndose de sitio—.
///
/// El certificado se recibe **en DER y no en PEM** a propósito: el PEM es
/// texto con base64 dentro, y leerlo pedía una dependencia nueva
/// —`rustls-pemfile`, que no está en el árbol— para un camino que sólo
/// recorren las pruebas. El servidor de mentira ya tiene el DER a mano porque
/// es lo que `rcgen` le da.
///
/// # Errores
///
/// Los mismos que [`correos_recientes`].
#[cfg(any(test, feature = "test-util"))]
pub fn correos_recientes_confiando_en(
    cuenta: &CuentaDeLectura,
    contrasena: &str,
    dias: u16,
    certificado: CertificateDer<'static>,
) -> Result<Vec<CorreoDelBuzon>, ErrorDeCorreo> {
    let socket = conectar(cuenta)?;
    let cifrado = cifrar(socket, &cuenta.servidor, Some(certificado))?;
    let mut sesion = Sesion::nueva(cifrado)?;
    let correos = sesion.leer(cuenta, contrasena, dias);
    sesion.despedirse();
    correos
}

fn conectar(cuenta: &CuentaDeLectura) -> Result<TcpStream, ErrorDeCorreo> {
    let socket = TcpStream::connect((cuenta.servidor.as_str(), cuenta.puerto()))
        .map_err(|_| ErrorDeCorreo::NoSePudoEntrarAlBuzon)?;
    socket
        .set_read_timeout(Some(ESPERA_MAXIMA))
        .and_then(|()| socket.set_write_timeout(Some(ESPERA_MAXIMA)))
        .map_err(|_| ErrorDeCorreo::NoSePudoEntrarAlBuzon)?;
    Ok(socket)
}

/// El flujo cifrado. Alias para que las firmas de abajo se lean.
type Cifrado = StreamOwned<ClientConnection, TcpStream>;

fn cifrar(
    socket: TcpStream,
    servidor: &str,
    raiz_de_mas: Option<CertificateDer<'static>>,
) -> Result<Cifrado, ErrorDeCorreo> {
    let mut raices = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    if let Some(cert) = raiz_de_mas {
        // `add` AÑADE una raíz. No desactiva nada: el certificado del servidor
        // tiene que estar firmado por ésta o por una de las de siempre, y el
        // nombre tiene que cuadrar. Es la misma decisión que
        // `Cifrado::confiando_ademas_en` para el envío.
        raices
            .add(cert)
            .map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?;
    }

    // `builder_with_provider` y no `builder()`, por lo mismo que el servidor de
    // mentira: el segundo depende de que alguien haya instalado antes un
    // proveedor de criptografía en una variable global del proceso, y en una
    // batería de pruebas en paralelo eso es una carrera. `ring` es el mismo que
    // usa `lettre`, así que no se compila una segunda biblioteca.
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?
    .with_root_certificates(raices)
    .with_no_client_auth();

    let nombre =
        ServerName::try_from(servidor.to_owned()).map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?;
    let conexion = ClientConnection::new(Arc::new(config), nombre)
        .map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?;
    Ok(StreamOwned::new(conexion, socket))
}

/// Una conversación IMAP abierta.
struct Sesion {
    flujo: BufReader<Cifrado>,
    etiqueta: u32,
}

impl Sesion {
    /// Abre la sesión y se traga el saludo del servidor.
    fn nueva(cifrado: Cifrado) -> Result<Self, ErrorDeCorreo> {
        let mut sesion = Self {
            flujo: BufReader::new(cifrado),
            etiqueta: 0,
        };
        // El saludo es `* OK …`. Si el servidor dice `* BYE`, no hay sesión.
        let saludo = sesion.leer_linea()?;
        if !saludo.starts_with("* OK") && !saludo.starts_with("* PREAUTH") {
            return Err(ErrorDeCorreo::NoSePudoEntrarAlBuzon);
        }
        Ok(sesion)
    }

    /// El recorrido entero: entrar, abrir en sólo lectura, buscar y traer.
    fn leer(
        &mut self,
        cuenta: &CuentaDeLectura,
        contrasena: &str,
        dias: u16,
    ) -> Result<Vec<CorreoDelBuzon>, ErrorDeCorreo> {
        // ── 1 · Entrar ──
        //
        // El usuario y la contraseña van entre comillas y con lo que haya que
        // escapar: una contraseña con una comilla o una barra rompería el
        // comando y el servidor contestaría `BAD`, que se leería como una
        // credencial rechazada. Es el defecto de los espacios de la contraseña
        // de Google (15/10/2026) en otra forma.
        let orden = format!(
            "LOGIN {} {}",
            entrecomillar(&cuenta.usuario),
            entrecomillar(contrasena)
        );
        self.mandar(&orden).map_err(|e| match e {
            // Un `NO` al `LOGIN` es exactamente «esa contraseña no vale», y
            // tiene su propio error porque el arreglo es otro: ahí no se
            // reintenta, se cambia la contraseña.
            ErrorDeCorreo::ElBuzonContestoAlgoRaro => ErrorDeCorreo::ElBuzonRechazoLaContrasena,
            otro => otro,
        })?;

        // ── 2 · Abrir el buzón en SÓLO LECTURA ──
        //
        // `EXAMINE` y no `SELECT`. Ver la cabecera del módulo: con esto, el
        // servidor rechaza cualquier cambio aunque alguien lo pidiera.
        self.mandar("EXAMINE INBOX")?;

        // ── 3 · Qué correos ──
        //
        // Por fecha y no `UNSEEN`. Con `EXAMINE` nada se marca como leído
        // —que es lo que se quiere—, así que un correo seguiría saliendo en
        // `UNSEEN` para siempre y se procesaría en cada pasada. La fecha acota
        // de verdad; de no repetir trabajo se encarga quien llama, con el
        // `uid`.
        let respuestas = self.mandar(&format!("UID SEARCH SINCE {}", hace_dias(dias)))?;
        let mut uids = uids_de_la_busqueda(&respuestas);
        if uids.len() > TOPE_DE_CORREOS {
            // Los más recientes, que son los que importan. Ver
            // `TOPE_DE_CORREOS`.
            uids.sort_unstable();
            uids = uids.split_off(uids.len() - TOPE_DE_CORREOS);
        }
        if uids.is_empty() {
            return Ok(Vec::new());
        }

        // ── 4 · Traerlos, en dos viajes ──
        //
        // Las cabeceras en uno y el texto en otro, **a propósito**: pedir los
        // dos en el mismo `FETCH` mete dos literales en una respuesta, y
        // partir eso bien exige un analizador de IMAP entero. Con un literal
        // por respuesta, lo que hay que entender cabe en `recoger_fetch`.
        let lista = uids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");

        let cabeceras = self.recoger_fetch(&format!(
            "UID FETCH {lista} BODY.PEEK[HEADER.FIELDS ({CABECERAS})]"
        ))?;
        let cuerpos = self.recoger_fetch(&format!(
            "UID FETCH {lista} BODY.PEEK[TEXT]<0.{BYTES_DEL_CUERPO}>"
        ))?;

        Ok(armar(&uids, &cabeceras, &cuerpos))
    }

    /// Manda un comando y devuelve las respuestas sin etiqueta.
    ///
    /// # Errores
    ///
    /// [`ErrorDeCorreo::ElBuzonContestoAlgoRaro`] si el servidor contesta `NO` o
    /// `BAD`, o si se pasa de los topes.
    fn mandar(&mut self, orden: &str) -> Result<Vec<String>, ErrorDeCorreo> {
        self.etiqueta += 1;
        let etiqueta = format!("a{}", self.etiqueta);
        let linea = format!("{etiqueta} {orden}\r\n");
        self.flujo
            .get_mut()
            .write_all(linea.as_bytes())
            .and_then(|()| self.flujo.get_mut().flush())
            .map_err(|_| ErrorDeCorreo::NoSePudoEntrarAlBuzon)?;

        let mut sin_etiqueta = Vec::new();
        loop {
            let linea = self.leer_linea()?;
            if let Some(resto) = linea.strip_prefix(&format!("{etiqueta} ")) {
                return if resto.starts_with("OK") {
                    Ok(sin_etiqueta)
                } else {
                    Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro)
                };
            }
            if sin_etiqueta.len() >= TOPE_DE_CORREOS * 4 {
                return Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro);
            }
            sin_etiqueta.push(linea);
        }
    }

    /// Manda un `FETCH` y devuelve, por cada respuesta, su `uid` y lo que vino
    /// en el literal.
    ///
    /// ── LO QUE ESTO NO ENTIENDE, DICHO EN VEZ DE DARSE POR CUBIERTO ───────
    ///
    /// Espera que el `UID` y el `{tamaño}` vengan **en la misma línea**, que es
    /// como lo manda todo servidor real: `* 3 FETCH (UID 12 BODY[…] {57}`. Un
    /// servidor que partiera eso en dos líneas devolvería ese correo sin datos
    /// —y el correo se descartaría— en vez de dar un dato equivocado, que es el
    /// lado por el que hay que fallar.
    ///
    /// Escribir un analizador de IMAP completo para cubrir ese caso sería
    /// cambiar una limitación conocida por mucho código sin ejercitar en el
    /// camino que **da de baja a personas**.
    fn recoger_fetch(&mut self, orden: &str) -> Result<Vec<(u32, String)>, ErrorDeCorreo> {
        self.etiqueta += 1;
        let etiqueta = format!("a{}", self.etiqueta);
        let linea = format!("{etiqueta} {orden}\r\n");
        self.flujo
            .get_mut()
            .write_all(linea.as_bytes())
            .and_then(|()| self.flujo.get_mut().flush())
            .map_err(|_| ErrorDeCorreo::NoSePudoEntrarAlBuzon)?;

        let mut recogido = Vec::new();
        loop {
            let linea = self.leer_linea()?;
            if let Some(resto) = linea.strip_prefix(&format!("{etiqueta} ")) {
                return if resto.starts_with("OK") {
                    Ok(recogido)
                } else {
                    Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro)
                };
            }

            // `* 3 FETCH (UID 12 BODY[…] {57}` — el literal viene detrás.
            if let (Some(uid), Some(cuantos)) = (uid_de(&linea), literal_de(&linea)?) {
                let datos = self.leer_exactamente(cuantos)?;
                recogido.push((uid, datos));
                if recogido.len() > TOPE_DE_CORREOS {
                    return Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro);
                }
            }
        }
    }

    /// Una línea, sin el `\r\n` y con su tope.
    fn leer_linea(&mut self) -> Result<String, ErrorDeCorreo> {
        let mut bruta = Vec::new();
        let leidos = (&mut self.flujo)
            .take(TOPE_DE_LINEA as u64)
            .read_until(b'\n', &mut bruta)
            .map_err(|_| ErrorDeCorreo::NoSePudoEntrarAlBuzon)?;
        if leidos == 0 {
            return Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro);
        }
        if leidos >= TOPE_DE_LINEA && !bruta.ends_with(b"\n") {
            return Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro);
        }
        // `from_utf8_lossy` y no `from_utf8`: un asunto mal codificado no puede
        // tumbar la lectura entera del buzón. Lo que salga raro no casará con
        // la palabra, y eso es un dudoso — que es el lado seguro.
        Ok(String::from_utf8_lossy(&bruta)
            .trim_end_matches(['\r', '\n'])
            .to_owned())
    }

    /// Exactamente `cuantos` bytes, que es lo que el literal prometió.
    fn leer_exactamente(&mut self, cuantos: usize) -> Result<String, ErrorDeCorreo> {
        let mut buzon = vec![0_u8; cuantos];
        self.flujo
            .read_exact(&mut buzon)
            .map_err(|_| ErrorDeCorreo::ElBuzonContestoAlgoRaro)?;
        Ok(String::from_utf8_lossy(&buzon).into_owned())
    }

    /// Se despide, y no le importa si falla.
    fn despedirse(&mut self) {
        let _ = self.mandar("LOGOUT");
    }
}

/// Entre comillas, escapando lo que IMAP exige escapar (RFC 3501 §4.3).
fn entrecomillar(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len() + 2);
    salida.push('"');
    for c in texto.chars() {
        if c == '"' || c == '\\' {
            salida.push('\\');
        }
        salida.push(c);
    }
    salida.push('"');
    salida
}

/// La fecha de hace `dias`, como la quiere `SEARCH SINCE`: `10-Nov-2026`.
///
/// **Los meses van en inglés y con esas tres letras** (RFC 3501 §9): es un
/// formato de protocolo, no un texto que alguien lea, así que no pasa por el
/// §139 ni se traduce. Escrito con un mes en español, el servidor contesta
/// `BAD` y la lectura entera falla.
/// ── Y LO ESCRIBE `time`, NO UNA TABLA DE MESES A MANO ─────────────────────
///
/// La primera versión llevaba un `[&str; 12]` con los meses y lo indexaba por
/// el número del mes menos uno. Dos defectos en una línea: es la **lista a
/// mano** que este proyecto lleva once veces arreglando, y clippy la rechazó
/// con razón —`indexing may panic`, y el §138 no admite panics en producción—.
/// `[month repr:short]` da exactamente las tres letras que pide el RFC, y lo
/// mantiene quien mantiene la biblioteca.
fn hace_dias(dias: u16) -> String {
    const COMO: &[time::format_description::FormatItem<'_>] =
        time::macros::format_description!("[day]-[month repr:short]-[year]");
    let cuando = time::OffsetDateTime::now_utc() - time::Duration::days(i64::from(dias));
    // Un formato fijo sobre una fecha válida no puede fallar; si alguna vez
    // fallara, una cadena vacía haría que el servidor contestara `BAD` y la
    // lectura fallaría con su error — nunca daría de baja a nadie de más.
    cuando.format(COMO).unwrap_or_default()
}

/// Los UID de una respuesta `* SEARCH 3 7 12`.
fn uids_de_la_busqueda(respuestas: &[String]) -> Vec<u32> {
    respuestas
        .iter()
        .filter_map(|l| l.strip_prefix("* SEARCH"))
        .flat_map(|resto| resto.split_whitespace().filter_map(|n| n.parse().ok()))
        .collect()
}

/// El `UID 12` de una línea de `FETCH`.
fn uid_de(linea: &str) -> Option<u32> {
    let resto = linea.split("UID ").nth(1)?;
    let cifras: String = resto.chars().take_while(char::is_ascii_digit).collect();
    cifras.parse().ok()
}

/// El `{57}` del final de una línea, si lo hay.
///
/// # Errores
///
/// [`ErrorDeCorreo::ElBuzonContestoAlgoRaro`] si el tamaño pasa de
/// [`TOPE_DEL_LITERAL`]. **Esto es la defensa**, no un detalle: un `{4294967295}`
/// sin tope reserva cuatro gigas antes de leer un solo byte.
fn literal_de(linea: &str) -> Result<Option<usize>, ErrorDeCorreo> {
    let Some(abre) = linea.rfind('{') else {
        return Ok(None);
    };
    if !linea.ends_with('}') {
        return Ok(None);
    }
    let dentro = &linea[abre + 1..linea.len() - 1];
    // Un literal «no sincronizado» acaba en `+}`; el tamaño es lo de delante.
    let dentro = dentro.trim_end_matches('+');
    let Ok(cuantos) = dentro.parse::<usize>() else {
        return Ok(None);
    };
    if cuantos > TOPE_DEL_LITERAL {
        return Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro);
    }
    Ok(Some(cuantos))
}

/// Junta lo de los dos viajes en un correo por UID.
fn armar(
    uids: &[u32],
    cabeceras: &[(u32, String)],
    cuerpos: &[(u32, String)],
) -> Vec<CorreoDelBuzon> {
    uids.iter()
        .filter_map(|uid| {
            let cab = cabeceras.iter().find(|(u, _)| u == uid)?;
            // **Un correo sin su cabecera se descarta; sin su cuerpo, no.** El
            // del botón de cancelar suscripción llega con el cuerpo vacío, y
            // ése es justo el camino que no se puede perder.
            let texto = cuerpos
                .iter()
                .find(|(u, _)| u == uid)
                .map_or("", |(_, t)| t.as_str());
            let de = direccion_de(&cabecera(&cab.1, "from")?)?;
            Some(CorreoDelBuzon {
                uid: *uid,
                de,
                asunto: cabecera(&cab.1, "subject").unwrap_or_default(),
                texto: texto_legible(texto),
                automatico: parece_automatico(&cab.1),
            })
        })
        .collect()
}

/// El valor de una cabecera, uniendo sus líneas continuadas.
///
/// Una cabecera larga se parte en varias líneas y las siguientes empiezan por
/// un espacio o un tabulador (RFC 5322 §2.2.3). Un asunto de más de 78
/// caracteres llega así **siempre**, así que sin esto el asunto quedaría
/// cortado — y «Baja» no se cortaría nunca, pero una frase que lo nombre sí.
fn cabecera(crudo: &str, nombre: &str) -> Option<String> {
    let mut valor: Option<String> = None;
    for linea in crudo.lines() {
        if let Some(v) = &mut valor {
            if linea.starts_with([' ', '\t']) {
                v.push(' ');
                v.push_str(linea.trim());
                continue;
            }
            return Some(v.trim().to_owned());
        }
        let Some((izq, der)) = linea.split_once(':') else {
            continue;
        };
        if izq.trim().eq_ignore_ascii_case(nombre) {
            valor = Some(der.trim().to_owned());
        }
    }
    valor.map(|v| v.trim().to_owned())
}

/// La dirección de un `From`, sin el nombre visible.
///
/// `"Ana Pérez" <ana@ejemplo.mx>` da `ana@ejemplo.mx`. Sin ángulos, el valor
/// entero. Se devuelve `None` si no hay una `@`: lo que no es una dirección no
/// se puede dar de baja, y adivinar aquí sería dar de baja a quien no es.
fn direccion_de(from: &str) -> Option<String> {
    let dentro = match (from.rfind('<'), from.rfind('>')) {
        (Some(a), Some(b)) if a < b => &from[a + 1..b],
        _ => from,
    };
    let limpia = dentro.trim().trim_matches('"').trim().to_lowercase();
    limpia.contains('@').then_some(limpia)
}

/// ¿Las cabeceras dicen que lo mandó una máquina?
fn parece_automatico(crudo: &str) -> bool {
    // `Auto-Submitted: no` es lo que pone un correo de una persona, y existe:
    // comprobar sólo que la cabecera esté daría por automático un correo
    // humano que la declare.
    if cabecera(crudo, "auto-submitted").is_some_and(|v| !v.trim().eq_ignore_ascii_case("no")) {
        return true;
    }
    if cabecera(crudo, "x-autoreply").is_some() || cabecera(crudo, "x-autorespond").is_some() {
        return true;
    }
    cabecera(crudo, "precedence").is_some_and(|v| {
        matches!(
            v.trim().to_lowercase().as_str(),
            "auto_reply" | "bulk" | "junk"
        )
    })
}

/// Saca el texto que escribió una persona de lo que vino en `BODY[TEXT]`.
///
/// ── POR QUÉ ESTO NO PUEDE VIVIR EN EL NÚCLEO ──────────────────────────────
///
/// El núcleo decide **qué cuenta como una baja** y para eso mira la primera
/// línea con algo. Lo que llega del servidor no empieza donde escribió la
/// persona: un correo de Gmail es `multipart/alternative`, así que
/// `BODY[TEXT]` arranca con un delimitador —`--000000000000abc`— y las
/// cabeceras de la primera parte. Dárselo tal cual al núcleo haría que la
/// primera línea con algo fuera **siempre** el delimitador, y entonces ni una
/// respuesta a mano daría de baja a nadie.
///
/// Desenvolver MIME es saber de correo, que es lo que este crate sabe y el
/// núcleo no (cero I/O, cero formatos de transporte). Así que se desenvuelve
/// aquí y el núcleo sigue viendo lo que vería de un correo de texto simple.
///
/// ── Y SU LÍMITE, DICHO EN VEZ DE DARSE POR CUBIERTO ───────────────────────
///
/// Un correo **sólo HTML**, sin su parte de texto, llega con etiquetas. Se les
/// quitan las marcas, y lo que quede puede no ser exactamente la palabra: eso
/// sale **dudoso** y lo mira una persona, que es el lado seguro. No se escribe
/// un intérprete de HTML aquí para ganar ese caso.
#[must_use]
pub fn texto_legible(crudo: &str) -> String {
    let mut lineas = crudo.lines().peekable();

    // Si empieza por un delimitador MIME, saltar hasta pasada la línea vacía
    // que cierra las cabeceras de esa parte.
    if lineas
        .peek()
        .is_some_and(|l| l.trim_start().starts_with("--"))
    {
        for linea in lineas.by_ref() {
            if linea.trim().is_empty() {
                break;
            }
        }
    }

    let mut salida = String::new();
    for linea in lineas {
        // Un delimitador cierra la parte: lo que venga después es otra
        // versión del mismo mensaje (el HTML), y no se mira.
        if linea.trim_start().starts_with("--") && linea.trim().len() > 2 {
            break;
        }
        let limpia = sin_etiquetas(linea);
        if !limpia.trim().is_empty() {
            salida.push_str(&limpia);
            salida.push('\n');
        } else if !salida.is_empty() {
            salida.push('\n');
        }
    }
    salida
}

/// Quita las marcas `<…>` de una línea.
fn sin_etiquetas(linea: &str) -> String {
    let mut salida = String::with_capacity(linea.len());
    let mut dentro = false;
    for c in linea.chars() {
        match c {
            '<' => dentro = true,
            '>' => dentro = false,
            _ if !dentro => salida.push(c),
            _ => {}
        }
    }
    salida
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_puerto_es_el_cifrado_y_no_hay_otro() {
        let cuenta = CuentaDeLectura {
            servidor: "imap.gmail.com".to_owned(),
            puerto: None,
            usuario: "ana@gmail.com".to_owned(),
        };
        assert_eq!(cuenta.puerto(), 993);
        // El 143 es IMAP en claro. Que se pueda escribir a mano es inevitable
        // —es un `Option<u16>`—, pero no hay ninguna constante ni ningún camino
        // del producto que lo ofrezca.
        assert_ne!(PUERTO_IMAP_TLS, 143);
    }

    #[test]
    fn una_contrasena_con_comillas_no_rompe_el_comando() {
        assert_eq!(entrecomillar("abcd efgh"), "\"abcd efgh\"");
        assert_eq!(entrecomillar("con\"comilla"), "\"con\\\"comilla\"");
        assert_eq!(entrecomillar("con\\barra"), "\"con\\\\barra\"");
    }

    /// ── EL TOPE DEL LITERAL ES LA DEFENSA, NO UN DETALLE ──────────────────
    #[test]
    fn un_literal_enorme_se_rechaza_antes_de_reservar_nada() {
        assert_eq!(literal_de("* 1 FETCH (UID 2 BODY[] {57}"), Ok(Some(57)));
        assert_eq!(
            literal_de("* 1 FETCH (UID 2 BODY[] {4294967295}"),
            Err(ErrorDeCorreo::ElBuzonContestoAlgoRaro)
        );
        // Sin literal, no hay nada que leer.
        assert_eq!(literal_de("* 1 FETCH (UID 2 FLAGS ())"), Ok(None));
        // Un literal no sincronizado acaba en `+}`.
        assert_eq!(literal_de("* 1 FETCH (UID 2 BODY[] {12+}"), Ok(Some(12)));
    }

    #[test]
    fn los_uid_salen_de_la_busqueda_y_del_fetch() {
        let respuestas = vec!["* SEARCH 3 7 12".to_owned(), "* 4 EXISTS".to_owned()];
        assert_eq!(uids_de_la_busqueda(&respuestas), vec![3, 7, 12]);
        assert_eq!(uids_de_la_busqueda(&[]), Vec::<u32>::new());
        assert_eq!(uid_de("* 1 FETCH (UID 12 BODY[] {5}"), Some(12));
        assert_eq!(uid_de("* 1 FETCH (FLAGS ())"), None);
    }

    #[test]
    fn la_fecha_de_la_busqueda_lleva_el_mes_en_ingles() {
        let hoy = hace_dias(0);
        // `10-Nov-2026`: dos cifras, tres letras, cuatro cifras.
        let trozos: Vec<&str> = hoy.split('-').collect();
        let [dia, mes, anio] = trozos.as_slice() else {
            panic!("formato inesperado: {hoy}")
        };
        assert_eq!(dia.len(), 2, "el día va con dos cifras: {hoy}");
        assert_eq!(mes.len(), 3, "el mes va con tres letras: {hoy}");
        assert!(
            mes.chars().all(|c| c.is_ascii_alphabetic()),
            "el mes tiene que ir en letras, no en cifras: {hoy}"
        );
        assert_eq!(anio.len(), 4, "el año va con cuatro cifras: {hoy}");
    }

    #[test]
    fn la_direccion_sale_de_entre_los_angulos() {
        assert_eq!(
            direccion_de("\"Ana Pérez\" <ana@ejemplo.mx>").as_deref(),
            Some("ana@ejemplo.mx")
        );
        assert_eq!(
            direccion_de("beto@ejemplo.mx").as_deref(),
            Some("beto@ejemplo.mx")
        );
        // En minúsculas, porque es con lo que se compara una supresión.
        assert_eq!(
            direccion_de("<ANA@Ejemplo.MX>").as_deref(),
            Some("ana@ejemplo.mx")
        );
        // Lo que no es una dirección no se da de baja.
        assert_eq!(direccion_de("Mailer Daemon"), None);
        assert_eq!(direccion_de(""), None);
    }

    /// Un asunto de más de 78 caracteres llega partido **siempre**.
    #[test]
    fn una_cabecera_partida_en_varias_lineas_se_une() {
        let crudo = "Subject: esto es un asunto muy largo que el cliente\r\n \
                     de correo partió en dos\r\nFrom: ana@ejemplo.mx\r\n";
        assert_eq!(
            cabecera(crudo, "subject").as_deref(),
            Some("esto es un asunto muy largo que el cliente de correo partió en dos")
        );
        assert_eq!(cabecera(crudo, "from").as_deref(), Some("ana@ejemplo.mx"));
        assert_eq!(cabecera(crudo, "to"), None);
    }

    #[test]
    fn el_nombre_de_la_cabecera_no_distingue_mayusculas() {
        let crudo = "SUBJECT: Baja\r\nFROM: ana@ejemplo.mx\r\n";
        assert_eq!(cabecera(crudo, "subject").as_deref(), Some("Baja"));
    }

    /// ── LAS CUATRO FORMAS DE DECLARARSE AUTOMÁTICO ────────────────────────
    #[test]
    fn un_aviso_automatico_se_reconoce_por_sus_cuatro_cabeceras() {
        for crudo in [
            "Auto-Submitted: auto-replied\r\n",
            "X-Autoreply: yes\r\n",
            "X-Autorespond: vacation\r\n",
            "Precedence: auto_reply\r\n",
        ] {
            assert!(parece_automatico(crudo), "no lo cazó: {crudo}");
        }
    }

    /// **`Auto-Submitted: no` es lo que pone un correo de una persona.**
    ///
    /// Comprobar sólo que la cabecera esté daría por automático —y por tanto
    /// perdería— la baja de quien escribe con un cliente que la declara.
    #[test]
    fn auto_submitted_no_es_una_persona() {
        assert!(!parece_automatico("Auto-Submitted: no\r\n"));
        assert!(!parece_automatico("From: ana@ejemplo.mx\r\n"));
        assert!(!parece_automatico(""));
    }

    /// ── EL CASO QUE OBLIGÓ A ESCRIBIR `texto_legible` ──────────────────────
    ///
    /// Un correo de Gmail es `multipart/alternative`. Sin desenvolver, la
    /// primera línea con algo es el delimitador y **ninguna respuesta a mano
    /// daría de baja a nadie**.
    #[test]
    fn el_mime_de_una_respuesta_de_gmail_se_desenvuelve() {
        let crudo = "--000000000000abc123\r\n\
                     Content-Type: text/plain; charset=\"UTF-8\"\r\n\
                     \r\n\
                     BAJA\r\n\
                     \r\n\
                     --000000000000abc123\r\n\
                     Content-Type: text/html; charset=\"UTF-8\"\r\n\
                     \r\n\
                     <div dir=\"ltr\">BAJA</div>\r\n\
                     --000000000000abc123--\r\n";
        let texto = texto_legible(crudo);
        assert_eq!(
            texto.lines().find(|l| !l.trim().is_empty()),
            Some("BAJA"),
            "lo que llegó al núcleo: {texto:?}"
        );
    }

    #[test]
    fn un_correo_de_texto_simple_pasa_tal_cual() {
        assert_eq!(texto_legible("BAJA\r\n").trim(), "BAJA");
        assert_eq!(texto_legible("").trim(), "");
    }

    /// El límite declarado: un correo sólo HTML pierde sus etiquetas, y lo que
    /// queda puede no ser la palabra sola. Sale dudoso, que es el lado seguro.
    #[test]
    fn un_correo_solo_html_se_queda_sin_etiquetas() {
        let texto = texto_legible("<div dir=\"ltr\">BAJA</div>\r\n");
        assert_eq!(texto.trim(), "BAJA");
        assert_eq!(sin_etiquetas("<p>hola</p>"), "hola");
    }

    /// Lo que de verdad importa de este módulo: que lo que saca se lea como
    /// una baja. Con el MIME de Gmail delante, que es el caso real.
    #[test]
    fn lo_que_sale_de_aqui_el_nucleo_lo_lee_como_una_baja() {
        let correo = CorreoDelBuzon {
            uid: 7,
            de: "ana@ejemplo.mx".to_owned(),
            asunto: "Re: Oferta de noviembre".to_owned(),
            texto: texto_legible("--abc\r\nContent-Type: text/plain\r\n\r\nBAJA\r\n--abc--\r\n"),
            automatico: false,
        };
        let veredicto = arles_core::baja_por_correo::leer(&correo.como_lo_lee_el_nucleo());
        assert!(
            veredicto.se_aplica_solo(),
            "el núcleo no lo leyó como baja: {veredicto:?}"
        );
    }

    #[test]
    fn un_correo_sin_cabecera_se_descarta_y_sin_cuerpo_no() {
        let cabeceras = vec![(
            1_u32,
            "From: ana@ejemplo.mx\r\nSubject: BAJA\r\n".to_owned(),
        )];
        // El del botón llega sin cuerpo, y es el camino que no se puede perder.
        let correos = armar(&[1, 2], &cabeceras, &[]);
        assert_eq!(correos.len(), 1, "el que no tiene cabecera se va");
        let uno = correos.first().expect("queda uno");
        assert_eq!(uno.de, "ana@ejemplo.mx");
        assert_eq!(uno.texto, "");
    }

    #[test]
    fn un_from_que_no_es_una_direccion_no_entra() {
        let cabeceras = vec![(1_u32, "From: Mailer Daemon\r\nSubject: BAJA\r\n".to_owned())];
        assert!(armar(&[1], &cabeceras, &[]).is_empty());
    }

    // ══════════════════════════════════════════════════════════════════════
    // DE PUNTA A PUNTA, CONTRA UN SERVIDOR IMAP QUE HABLA CIFRADO
    // ══════════════════════════════════════════════════════════════════════
    //
    // Lo de arriba comprueba las piezas; esto comprueba **lo que sale por el
    // cable**, que es lo único que no se puede afirmar leyendo el código. Es la
    // misma razón por la que el envío tiene su servidor de mentira desde la
    // 5.1.

    use crate::servidor_imap::{CorreoGuardado, GuionDelBuzon, ServidorImap};

    /// La cuenta que apunta al servidor de mentira.
    ///
    /// **`localhost` y no `127.0.0.1`**: el certificado se emite para el
    /// nombre, y con la dirección IP la validación no podría cuadrar — y el
    /// arreglo fácil sería desactivarla, que es justo lo que no se hace.
    fn cuenta_de(servidor: &ServidorImap) -> CuentaDeLectura {
        CuentaDeLectura {
            servidor: crate::servidor_cifrado::NOMBRE_DE_PRUEBA.to_owned(),
            puerto: Some(servidor.direccion().port()),
            usuario: "enlace@telemetrymx.com".to_owned(),
        }
    }

    fn leer_de(servidor: &ServidorImap) -> Result<Vec<CorreoDelBuzon>, ErrorDeCorreo> {
        correos_recientes_confiando_en(
            &cuenta_de(servidor),
            "la-contrasena",
            7,
            servidor.certificado(),
        )
    }

    /// ── LA GARANTÍA DEL TRAMO, MEDIDA DESDE EL OTRO LADO ──────────────────
    ///
    /// Dirección autorizó leer su buzón **sólo lectura**. Que el código diga
    /// `EXAMINE` se comprueba leyéndolo; que lo que llegue al servidor sea
    /// `EXAMINE` y no `SELECT`, sólo desde aquí.
    ///
    /// Las dos mitades importan, y es la lección del 09/10 con `cargo arles`:
    /// **exigir que el bueno esté no impide que el malo también esté.**
    #[test]
    fn el_buzon_se_abre_en_solo_lectura_y_no_se_marca_nada_como_leido() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            7,
            "ana@ejemplo.mx",
            "Baja",
            "",
        )]))
        .expect("arranca");
        leer_de(&s).expect("lee");

        assert!(
            s.dijo("EXAMINE"),
            "tiene que abrir con EXAMINE: {:?}",
            s.dicho()
        );
        assert!(
            !s.dijo("SELECT"),
            "SELECT abre en lectura y escritura: {:?}",
            s.dicho()
        );
        assert!(
            s.dijo("BODY.PEEK"),
            "tiene que pedir con PEEK: {:?}",
            s.dicho()
        );
        assert!(
            !s.dijo("BODY["),
            "un BODY[ sin PEEK marca el correo como leído: {:?}",
            s.dicho()
        );
        // Y ninguno de los comandos que cambian el buzón, aunque `EXAMINE` los
        // rechazaría: las dos defensas, no una.
        for prohibido in ["STORE", "EXPUNGE", "COPY ", "MOVE ", "APPEND", "DELETE"] {
            assert!(
                !s.dijo(prohibido),
                "mandó un comando que escribe ({prohibido}): {:?}",
                s.dicho()
            );
        }
    }

    /// El camino del botón de cancelar suscripción: asunto exacto, cuerpo
    /// vacío.
    #[test]
    fn el_correo_del_boton_se_lee_y_el_nucleo_lo_aplica() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            7,
            "\"Ana Pérez\" <ana@ejemplo.mx>",
            "BAJA",
            "",
        )]))
        .expect("arranca");

        let correos = leer_de(&s).expect("lee");
        assert_eq!(correos.len(), 1);
        let uno = correos.first().expect("llegó el correo del botón");
        assert_eq!(uno.uid, 7);
        assert_eq!(uno.de, "ana@ejemplo.mx");
        let veredicto = arles_core::baja_por_correo::leer(&uno.como_lo_lee_el_nucleo());
        assert!(veredicto.se_aplica_solo(), "veredicto: {veredicto:?}");
    }

    /// ── EL CAMINO QUE OBLIGÓ A DESENVOLVER EL MIME ────────────────────────
    ///
    /// Quien contesta a mano desde Gmail manda `multipart/alternative` con el
    /// original citado debajo. Es el correo real, y el que ninguna prueba de
    /// una pieza sola puede comprobar.
    #[test]
    fn una_respuesta_de_gmail_con_la_campana_citada_se_aplica() {
        let cuerpo = "--000000000000abc\r\n\
                      Content-Type: text/plain; charset=\"UTF-8\"\r\n\
                      \r\n\
                      BAJA\r\n\
                      \r\n\
                      El 10 de noviembre escribió:\r\n\
                      > Oferta de noviembre\r\n\
                      > Si no quieres recibir más correos nuestros, responde a\r\n\
                      > este mensaje con la palabra BAJA.\r\n\
                      \r\n\
                      --000000000000abc--\r\n";
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            12,
            "beto@ejemplo.mx",
            "Re: Oferta de noviembre",
            cuerpo,
        )]))
        .expect("arranca");

        let correos = leer_de(&s).expect("lee");
        assert_eq!(correos.len(), 1);
        let uno = correos.first().expect("llegó la respuesta");
        let veredicto = arles_core::baja_por_correo::leer(&uno.como_lo_lee_el_nucleo());
        assert!(
            veredicto.se_aplica_solo(),
            "una respuesta real de Gmail no se leyó como baja. Texto: {:?}",
            uno.texto
        );
    }

    /// Y la vuelta de la misma moneda: una respuesta que NO pide la baja, con
    /// la campaña citada debajo —que lleva la palabra dentro—.
    #[test]
    fn una_respuesta_interesada_no_da_de_baja_a_nadie() {
        let cuerpo = "--abc\r\n\
                      Content-Type: text/plain\r\n\
                      \r\n\
                      Gracias, me interesa. ¿Me llamas mañana?\r\n\
                      \r\n\
                      El 10 de noviembre escribió:\r\n\
                      > responde a este mensaje con la palabra BAJA\r\n\
                      --abc--\r\n";
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::de(
            3,
            "caro@ejemplo.mx",
            "Re: Oferta",
            cuerpo,
        )]))
        .expect("arranca");

        let correos = leer_de(&s).expect("lee");
        let uno = correos.first().expect("llegó la respuesta");
        let veredicto = arles_core::baja_por_correo::leer(&uno.como_lo_lee_el_nucleo());
        assert_eq!(
            veredicto,
            arles_core::baja_por_correo::Veredicto::NoEsUnaBaja,
            "texto que llegó: {:?}",
            uno.texto
        );
    }

    /// Un aviso de ausencia con la palabra dentro. El peor error posible es
    /// darle de baja: nunca pidió nada.
    #[test]
    fn un_aviso_de_ausencia_llega_marcado_como_automatico() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![CorreoGuardado::automatico(
            4,
            "dani@ejemplo.mx",
            "Automatic reply: Oferta",
            "Estoy de baja por maternidad hasta marzo.\r\n",
        )]))
        .expect("arranca");

        let correos = leer_de(&s).expect("lee");
        assert_eq!(correos.len(), 1);
        let uno = correos.first().expect("llegó el aviso");
        assert!(uno.automatico, "no se marcó como automático");
        let veredicto = arles_core::baja_por_correo::leer(&uno.como_lo_lee_el_nucleo());
        assert_eq!(
            veredicto,
            arles_core::baja_por_correo::Veredicto::NoEsUnaBaja
        );
    }

    #[test]
    fn un_buzon_vacio_devuelve_una_lista_vacia() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(Vec::new())).expect("arranca");
        assert!(leer_de(&s).expect("lee").is_empty());
        // Y no pide ningún FETCH: no hay nada que traer.
        assert!(!s.dijo("FETCH"), "pidió correos sin tener ninguno");
    }

    /// Una contraseña rechazada tiene su propio error porque el arreglo es
    /// otro: no se reintenta, se revisa la contraseña.
    #[test]
    fn una_contrasena_rechazada_lo_dice_con_su_error() {
        let s = ServidorImap::arrancar(GuionDelBuzon::contrasena_rechazada()).expect("arranca");
        assert_eq!(leer_de(&s), Err(ErrorDeCorreo::ElBuzonRechazoLaContrasena));
    }

    /// Varios correos a la vez, y cada uno con su UID.
    ///
    /// **Dos y no uno a propósito**: con uno solo no se ve que las cabeceras y
    /// el cuerpo de cada cual se junten con los suyos — y los dos viajes del
    /// `FETCH` son exactamente donde eso se puede cruzar. Es la lección de
    /// D-WA, cuya primera prueba de WhatsApp manda a dos contactos por lo
    /// mismo.
    #[test]
    fn cada_correo_se_junta_con_sus_propios_datos() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(vec![
            CorreoGuardado::de(10, "ana@ejemplo.mx", "Re: Oferta", "BAJA\r\n"),
            CorreoGuardado::de(
                20,
                "beto@ejemplo.mx",
                "Re: Oferta",
                "Gracias, me interesa\r\n",
            ),
        ]))
        .expect("arranca");

        let correos = leer_de(&s).expect("lee");
        assert_eq!(correos.len(), 2);

        let ana = correos.iter().find(|c| c.uid == 10).expect("ana");
        let beto = correos.iter().find(|c| c.uid == 20).expect("beto");
        assert_eq!(ana.de, "ana@ejemplo.mx");
        assert_eq!(beto.de, "beto@ejemplo.mx");
        assert!(
            arles_core::baja_por_correo::leer(&ana.como_lo_lee_el_nucleo()).se_aplica_solo(),
            "a Ana le tocaba baja"
        );
        assert!(
            !arles_core::baja_por_correo::leer(&beto.como_lo_lee_el_nucleo()).se_aplica_solo(),
            "a Beto no: su cuerpo era de Ana"
        );
    }

    /// La contraseña viaja cifrada, y se mide desde el servidor.
    ///
    /// El servidor de mentira sólo habla TLS desde el primer byte, así que una
    /// conexión en claro no llegaría a decir `LOGIN`. Lo que esto comprueba es
    /// que **no existe un camino sin cifrar**: no hay un `Modo` que elegir.
    #[test]
    fn no_hay_forma_de_leer_el_buzon_sin_cifrar() {
        let s = ServidorImap::arrancar(GuionDelBuzon::con(Vec::new())).expect("arranca");
        leer_de(&s).expect("lee");
        // Llegó el LOGIN, y sólo pudo llegar por dentro del túnel.
        assert!(s.dijo("LOGIN"), "no entró: {:?}", s.dicho());
    }
}
