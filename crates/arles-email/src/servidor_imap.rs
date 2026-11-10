//! Un servidor IMAP de mentira, para probar el lector del buzón sin un buzón.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO EXISTE, Y NO ES OPCIONAL
//!
//! Es la misma razón que el servidor SMTP de mentira de la 5.1: **no se le
//! puede pedir a Gmail que conteste `NO`**, ni que tenga en el buzón un correo
//! con el MIME partido de una forma concreta. Y probar el lector sólo contra
//! Gmail significa probarlo el día que alguien lo encienda, con el buzón de un
//! cliente — que aquí es peor que en el envío, porque lo que está en juego es
//! **a quién se da de baja**.
//!
//! Así que esto habla IMAP de verdad, cifrado de verdad, y se le dice qué
//! correos tiene dentro. El lector que hablará con Gmail es exactamente el
//! mismo que habla con éste.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE SERVIDOR SÍ HACE, Y ES LA MITAD DEL VALOR
//!
//! **Apunta todo lo que el cliente le dice**, y eso es lo que permite afirmar
//! la garantía del tramo desde el otro lado del cable: que ARLES mandó
//! `EXAMINE` y no `SELECT`, y `BODY.PEEK` y no `BODY`. Mirar el código del
//! lector comprueba que el lector está bien escrito; mirar esta lista
//! comprueba **lo que salió por el cable**, que es el patrón del §9 visto
//! veinte veces en este proyecto.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

use crate::servidor_cifrado::NOMBRE_DE_PRUEBA;

/// Un correo que este servidor dice tener en el buzón.
#[derive(Debug, Clone)]
pub struct CorreoGuardado {
    /// Su identificador dentro del buzón.
    pub uid: u32,
    /// Las cabeceras, tal como las devolvería un `FETCH`. Con `\r\n`.
    pub cabeceras: String,
    /// El cuerpo, tal como lo devolvería `BODY[TEXT]`.
    pub cuerpo: String,
}

impl CorreoGuardado {
    /// Un correo de una persona que contesta a una campaña.
    #[must_use]
    pub fn de(uid: u32, quien: &str, asunto: &str, cuerpo: &str) -> Self {
        Self {
            uid,
            cabeceras: format!("From: {quien}\r\nSubject: {asunto}\r\n"),
            cuerpo: cuerpo.to_owned(),
        }
    }

    /// Un aviso de ausencia, con la cabecera que lo declara.
    #[must_use]
    pub fn automatico(uid: u32, quien: &str, asunto: &str, cuerpo: &str) -> Self {
        Self {
            uid,
            cabeceras: format!(
                "From: {quien}\r\nSubject: {asunto}\r\nAuto-Submitted: auto-replied\r\n"
            ),
            cuerpo: cuerpo.to_owned(),
        }
    }
}

/// Qué va a contestar este servidor.
#[derive(Debug, Clone)]
pub struct GuionDelBuzon {
    /// Lo que hay en el buzón.
    pub correos: Vec<CorreoGuardado>,
    /// Si es `false`, el `LOGIN` se contesta con `NO`.
    pub acepta_la_contrasena: bool,
}

impl GuionDelBuzon {
    /// Un buzón con estos correos y una contraseña que vale.
    #[must_use]
    pub fn con(correos: Vec<CorreoGuardado>) -> Self {
        Self {
            correos,
            acepta_la_contrasena: true,
        }
    }

    /// Un servidor que rechaza la contraseña.
    #[must_use]
    pub fn contrasena_rechazada() -> Self {
        Self {
            correos: Vec::new(),
            acepta_la_contrasena: false,
        }
    }
}

/// El servidor de mentira.
#[derive(Debug)]
pub struct ServidorImap {
    direccion: SocketAddr,
    certificado: CertificateDer<'static>,
    dicho: Arc<Mutex<Vec<String>>>,
    seguir: Arc<AtomicBool>,
    hilo: Option<JoinHandle<()>>,
}

impl ServidorImap {
    /// Levanta el servidor en un puerto libre de `localhost`, con un
    /// certificado recién generado para él.
    ///
    /// # Errores
    ///
    /// El error de red si no se puede escuchar, o uno de entrada/salida si el
    /// certificado o la configuración de TLS no se pueden construir.
    pub fn arrancar(guion: GuionDelBuzon) -> io::Result<Self> {
        let (certificado, config) = certificado_y_config()?;
        let config = Arc::new(config);

        let escucha = TcpListener::bind("127.0.0.1:0")?;
        let direccion = escucha.local_addr()?;

        let dicho = Arc::new(Mutex::new(Vec::new()));
        let seguir = Arc::new(AtomicBool::new(true));

        let dicho_del_hilo = Arc::clone(&dicho);
        let seguir_del_hilo = Arc::clone(&seguir);
        let hilo = std::thread::spawn(move || {
            for conexion in escucha.incoming() {
                if !seguir_del_hilo.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(socket) = conexion else { return };
                atender(socket, &guion, &config, &dicho_del_hilo);
            }
        });

        Ok(Self {
            direccion,
            certificado,
            dicho,
            seguir,
            hilo: Some(hilo),
        })
    }

    /// Dónde está escuchando.
    #[must_use]
    pub const fn direccion(&self) -> SocketAddr {
        self.direccion
    }

    /// El certificado que presenta, en DER.
    ///
    /// Es lo que se le da al lector con `correos_recientes_confiando_en`. En
    /// DER y no en PEM porque leer un PEM pedía una dependencia nueva, y esto
    /// ya lo tiene así.
    #[must_use]
    pub fn certificado(&self) -> CertificateDer<'static> {
        self.certificado.clone()
    }

    /// Todo lo que el cliente dijo.
    #[must_use]
    pub fn dicho(&self) -> Vec<String> {
        self.dicho.lock().map(|d| d.clone()).unwrap_or_default()
    }

    /// ¿Mandó algún comando que contenga esto? Sin distinguir mayúsculas.
    #[must_use]
    pub fn dijo(&self, trozo: &str) -> bool {
        let trozo = trozo.to_ascii_uppercase();
        self.dicho()
            .iter()
            .any(|l| l.to_ascii_uppercase().contains(&trozo))
    }
}

impl Drop for ServidorImap {
    fn drop(&mut self) {
        self.seguir.store(false, Ordering::SeqCst);
        // El hilo está dormido en `accept` y no se entera del cambio de
        // bandera: se le despierta con una conexión que no dice nada. Es lo
        // mismo que hacen los otros dos servidores de mentira.
        let _ = TcpStream::connect(self.direccion);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

fn atender(
    socket: TcpStream,
    guion: &GuionDelBuzon,
    config: &Arc<ServerConfig>,
    dicho: &Mutex<Vec<String>>,
) {
    let Ok(conexion) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let cifrado = StreamOwned::new(conexion, socket);
    let mut flujo = BufReader::new(cifrado);

    // IMAP saluda primero (RFC 3501 §7.1.1).
    if escribir(&mut flujo, "* OK ARLES de mentira listo\r\n").is_err() {
        return;
    }

    loop {
        let mut linea = String::new();
        match flujo.read_line(&mut linea) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let linea = linea.trim_end_matches(['\r', '\n']).to_owned();
        if linea.is_empty() {
            continue;
        }
        if let Ok(mut d) = dicho.lock() {
            d.push(linea.clone());
        }

        let (etiqueta, orden) = linea.split_once(' ').unwrap_or((linea.as_str(), ""));
        let arriba = orden.to_ascii_uppercase();

        let salida = if arriba.starts_with("LOGIN") {
            if guion.acepta_la_contrasena {
                format!("{etiqueta} OK entró\r\n")
            } else {
                format!("{etiqueta} NO esa contraseña no vale\r\n")
            }
        } else if arriba.starts_with("EXAMINE") {
            // `[READ-ONLY]` es lo que de verdad contesta un servidor a
            // `EXAMINE`, y lo que contestaría `[READ-WRITE]` a `SELECT`.
            format!(
                "* {} EXISTS\r\n{etiqueta} OK [READ-ONLY] abierto\r\n",
                guion.correos.len()
            )
        } else if arriba.starts_with("SELECT") {
            // Nunca debería llegar aquí: lo contesta igual para que, si algún
            // día llegara, lo que falle sea la comprobación del cable y no la
            // conexión.
            format!("{etiqueta} OK [READ-WRITE] abierto\r\n")
        } else if arriba.starts_with("UID SEARCH") {
            let uids: Vec<String> = guion.correos.iter().map(|c| c.uid.to_string()).collect();
            format!("* SEARCH {}\r\n{etiqueta} OK buscado\r\n", uids.join(" "))
        } else if arriba.starts_with("UID FETCH") {
            respuesta_al_fetch(etiqueta, &arriba, guion)
        } else if arriba.starts_with("LOGOUT") {
            let _ = escribir(&mut flujo, &format!("* BYE\r\n{etiqueta} OK adiós\r\n"));
            return;
        } else {
            format!("{etiqueta} BAD no sé qué es eso\r\n")
        };

        if escribir(&mut flujo, &salida).is_err() {
            return;
        }
    }
}

/// Arma la respuesta a un `UID FETCH`, con sus literales.
fn respuesta_al_fetch(etiqueta: &str, arriba: &str, guion: &GuionDelBuzon) -> String {
    // Qué pidió: las cabeceras o el texto. Se mira `HEADER` porque es lo que
    // distingue los dos viajes del lector.
    let quiere_cabeceras = arriba.contains("HEADER");
    let mut salida = String::new();

    for (n, correo) in guion.correos.iter().enumerate() {
        // Sólo los UID que pidió. Se buscan como palabra entre comas para que
        // el 1 no case con el 12.
        let pedidos = arriba
            .split_whitespace()
            .nth(2)
            .unwrap_or_default()
            .to_owned();
        if !pedidos
            .split(',')
            .any(|p| p.trim() == correo.uid.to_string())
        {
            continue;
        }

        let (etiqueta_del_dato, datos) = if quiere_cabeceras {
            (
                "BODY[HEADER.FIELDS (FROM SUBJECT)]".to_owned(),
                // Un `FETCH` de cabeceras acaba con la línea vacía que las
                // cierra, y el lector tiene que soportarlo.
                format!("{}\r\n", correo.cabeceras),
            )
        } else {
            ("BODY[TEXT]<0>".to_owned(), correo.cuerpo.clone())
        };

        salida.push_str(&format!(
            "* {} FETCH (UID {} {etiqueta_del_dato} {{{}}}\r\n{datos})\r\n",
            n + 1,
            correo.uid,
            datos.len()
        ));
    }

    salida.push_str(&format!("{etiqueta} OK traído\r\n"));
    salida
}

fn escribir<S: Read + Write>(flujo: &mut BufReader<S>, texto: &str) -> io::Result<()> {
    flujo.get_mut().write_all(texto.as_bytes())?;
    flujo.get_mut().flush()
}

/// Genera un certificado para `localhost` y la configuración de TLS que lo usa.
fn certificado_y_config() -> io::Result<(CertificateDer<'static>, ServerConfig)> {
    let emitido = rcgen::generate_simple_self_signed([NOMBRE_DE_PRUEBA.to_owned()])
        .map_err(|e| io::Error::other(format!("no se pudo emitir el certificado: {e}")))?;

    let cert: CertificateDer<'static> = emitido.cert.der().clone();
    let clave: PrivateKeyDer<'static> =
        PrivatePkcs8KeyDer::from(emitido.signing_key.serialize_der()).into();

    let config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| io::Error::other(format!("versiones de TLS: {e}")))?
            .with_no_client_auth()
            .with_single_cert(vec![cert.clone()], clave)
            .map_err(|e| io::Error::other(format!("configuración de TLS: {e}")))?;

    Ok((cert, config))
}
