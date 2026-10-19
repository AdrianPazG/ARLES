//! El servidor de mentira, pero hablando cifrado.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO VIVE SÓLO EN LAS PRUEBAS
//!
//! El módulo entero está detrás de `#[cfg(test)]`, y no es por orden: es para
//! que **`rustls` y `rcgen` no entren en el programa que se entrega**. Son un
//! motor de TLS y un generador de certificados; ARLES no necesita ninguno de
//! los dos para mandar correo —de eso se encarga `lettre`—, y meter un
//! generador de certificados en un instalador es superficie de ataque a cambio
//! de nada.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTO HACE POSIBLE
//!
//! El adaptador exige TLS y no tiene ninguna forma de no exigirlo (`tls.rs`).
//! Eso deja una pregunta incómoda: **¿cómo se prueba, entonces?** La respuesta
//! fácil es una llamada que acepte certificados inválidos «sólo en los tests»,
//! y esa llamada, una vez escrita, se acaba moviendo.
//!
//! Así que aquí se hace lo otro: se genera un certificado de verdad para
//! `localhost`, el servidor lo presenta, y al adaptador se le da **ese
//! certificado concreto** como raíz de confianza de más. La verificación sigue
//! entera —firma y nombre—; lo único que cambia es a quién se cree.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CONVERSACIÓN NO SE REESCRIBE
//!
//! [`crate::servidor::conversar`] trabaja sobre cualquier cosa legible y
//! escribible. Aquí se le pasa un flujo cifrado en vez de un socket, y ya está.
//! Ésa era la razón de escribirla así en la entrega 5.1.

use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

use crate::servidor::{Guion, conversar, conversar_ya_saludado};

/// El nombre para el que se emite el certificado de prueba.
///
/// **Un nombre, no una dirección IP.** Un certificado se valida contra el
/// nombre; si el adaptador se conectara a `127.0.0.1` la validación fallaría, y
/// el arreglo fácil sería desactivarla — que es justo lo que no se hace.
pub const NOMBRE_DE_PRUEBA: &str = "localhost";

/// Cómo cifra este servidor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComoCifra {
    /// Empieza en claro y se cifra cuando el cliente manda `STARTTLS`.
    /// Es lo que hace Gmail en el 587.
    Starttls,
    /// Cifrado desde el primer byte, como el 465.
    Directo,
}

/// Un servidor SMTP de mentira que habla cifrado.
#[derive(Debug)]
pub struct ServidorCifrado {
    direccion: SocketAddr,
    pem: Vec<u8>,
    dicho: Arc<Mutex<Vec<String>>>,
    en_claro: Arc<Mutex<Vec<String>>>,
    cabeceras: Arc<Mutex<Vec<String>>>,
    seguir: Arc<AtomicBool>,
    hilo: Option<JoinHandle<()>>,
}

impl ServidorCifrado {
    /// Levanta el servidor en un puerto libre de `localhost`, con un
    /// certificado recién generado para él.
    ///
    /// # Errores
    ///
    /// El error de red si no se puede escuchar, o uno de entrada/salida si el
    /// certificado o la configuración de TLS no se pueden construir.
    pub fn arrancar(guion: Guion, como: ComoCifra) -> io::Result<Self> {
        let (pem, config) = certificado_y_config()?;
        let config = Arc::new(config);

        let escucha = TcpListener::bind("127.0.0.1:0")?;
        let direccion = escucha.local_addr()?;

        let dicho = Arc::new(Mutex::new(Vec::new()));
        let en_claro = Arc::new(Mutex::new(Vec::new()));
        let cabeceras = Arc::new(Mutex::new(Vec::new()));
        let seguir = Arc::new(AtomicBool::new(true));

        let dicho_del_hilo = Arc::clone(&dicho);
        let en_claro_del_hilo = Arc::clone(&en_claro);
        let cabeceras_del_hilo = Arc::clone(&cabeceras);
        let seguir_del_hilo = Arc::clone(&seguir);
        let hilo = std::thread::spawn(move || {
            for conexion in escucha.incoming() {
                if !seguir_del_hilo.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(socket) = conexion else { return };
                atender(
                    socket,
                    &guion,
                    como,
                    &config,
                    &dicho_del_hilo,
                    &en_claro_del_hilo,
                    &cabeceras_del_hilo,
                );
            }
        });

        Ok(Self {
            direccion,
            pem,
            dicho,
            en_claro,
            cabeceras,
            seguir,
            hilo: Some(hilo),
        })
    }

    /// Dónde está escuchando.
    #[must_use]
    pub const fn direccion(&self) -> SocketAddr {
        self.direccion
    }

    /// El certificado que presenta, en PEM.
    ///
    /// Es lo que se le da al adaptador con `Cifrado::confiando_ademas_en`.
    #[must_use]
    pub fn pem(&self) -> Vec<u8> {
        self.pem.clone()
    }

    /// Todo lo que dijo el cliente, cifrado o no.
    #[must_use]
    pub fn dicho(&self) -> Vec<String> {
        self.dicho.lock().map(|d| d.clone()).unwrap_or_default()
    }

    /// **Lo que el cliente dijo antes de cifrar.**
    ///
    /// Es la lista que importa para el §29: lo que un vecino de red habría
    /// podido leer. La contraseña no puede estar aquí.
    #[must_use]
    pub fn dicho_en_claro(&self) -> Vec<String> {
        self.en_claro.lock().map(|d| d.clone()).unwrap_or_default()
    }

    /// ¿Llegó a mandar un comando que empiece así? Sin distinguir mayúsculas.
    /// Las cabeceras del correo que llegó, línea a línea.
    ///
    /// **Es el único sitio desde donde se puede afirmar que una cabecera
    /// viaja de verdad.** Mirar lo que produce la función que arma el mensaje
    /// comprueba que esa función está bien, no que el envío la use — que es el
    /// patrón del §9, veinte veces visto en este proyecto.
    #[must_use]
    pub fn cabeceras_del_correo(&self) -> Vec<String> {
        self.cabeceras.lock().map(|d| d.clone()).unwrap_or_default()
    }

    #[must_use]
    pub fn dijo(&self, comando: &str) -> bool {
        empieza_por(&self.dicho(), comando)
    }

    /// ¿Lo dijo **en claro**, antes de cifrar?
    #[must_use]
    pub fn dijo_en_claro(&self, comando: &str) -> bool {
        empieza_por(&self.dicho_en_claro(), comando)
    }
}

impl Drop for ServidorCifrado {
    fn drop(&mut self) {
        self.seguir.store(false, Ordering::SeqCst);
        // Igual que el servidor en claro: el hilo está dormido en `accept` y no
        // se entera del cambio de bandera. Se le despierta con una conexión que
        // no dice nada.
        let _ = TcpStream::connect(self.direccion);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

fn empieza_por(lineas: &[String], comando: &str) -> bool {
    let comando = comando.to_ascii_uppercase();
    lineas
        .iter()
        .any(|l| l.to_ascii_uppercase().starts_with(&comando))
}

/// Lleva una conexión entera, cifrándola donde toque.
fn atender(
    socket: TcpStream,
    guion: &Guion,
    como: ComoCifra,
    config: &Arc<ServerConfig>,
    dicho: &Mutex<Vec<String>>,
    en_claro: &Mutex<Vec<String>>,
    cabeceras: &Mutex<Vec<String>>,
) {
    let mut socket = socket;

    if como == ComoCifra::Starttls {
        // Primera vuelta en claro: saludo, `EHLO`, `STARTTLS`. Aquí es donde un
        // cliente mal hecho mandaría la contraseña.
        let primera = conversar(&mut socket, guion);
        apuntar(en_claro, primera.dicho.clone());
        apuntar(dicho, primera.dicho);
        if !primera.hay_que_cifrar {
            return;
        }
    }

    let Ok(conexion) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut cifrado = StreamOwned::new(conexion, socket);

    // El mismo código de siempre, ahora sobre un flujo cifrado. Con STARTTLS no
    // se vuelve a saludar: el `220` es de la apertura de la conexión, no del
    // cifrado (RFC 3207).
    let segunda = match como {
        ComoCifra::Starttls => conversar_ya_saludado(&mut cifrado, guion),
        ComoCifra::Directo => conversar(&mut cifrado, guion),
    };
    apuntar(dicho, segunda.dicho);
    apuntar(cabeceras, segunda.cabeceras_del_correo);
}

fn apuntar(donde: &Mutex<Vec<String>>, lineas: Vec<String>) {
    if let Ok(mut d) = donde.lock() {
        d.extend(lineas);
    }
}

/// Genera un certificado para `localhost` y la configuración de TLS que lo usa.
fn certificado_y_config() -> io::Result<(Vec<u8>, ServerConfig)> {
    let emitido = rcgen::generate_simple_self_signed([NOMBRE_DE_PRUEBA.to_owned()])
        .map_err(|e| io::Error::other(format!("no se pudo emitir el certificado: {e}")))?;

    let pem = emitido.cert.pem().into_bytes();
    let cert: CertificateDer<'static> = emitido.cert.der().clone();
    let clave: PrivateKeyDer<'static> =
        PrivatePkcs8KeyDer::from(emitido.signing_key.serialize_der()).into();

    // `builder_with_provider` y no `builder()`: el segundo depende de que
    // alguien haya instalado antes un proveedor de criptografía en una variable
    // global del proceso, y en una batería de pruebas que corren en paralelo
    // eso es una carrera. Aquí se dice cuál se usa —`ring`, el mismo que
    // `lettre`— y no hay estado compartido que pueda faltar.
    let config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| io::Error::other(format!("versiones de TLS: {e}")))?
            .with_no_client_auth()
            .with_single_cert(vec![cert], clave)
            .map_err(|e| io::Error::other(format!("configuración de TLS: {e}")))?;

    Ok((pem, config))
}
