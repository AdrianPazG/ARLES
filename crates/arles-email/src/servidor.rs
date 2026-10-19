//! Un servidor de correo de mentira, en el propio proceso.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ HACE FALTA
//!
//! **A Gmail no se le puede pedir que conteste `550`.** Ni que se caiga a
//! mitad de la conversación, ni que rechace la contraseña, ni que tarde diez
//! minutos.
//!
//! Sin un servidor al que ordenarle eso, el adaptador SMTP sólo se ejercitaría
//! contra el camino feliz, y todas las ramas de error —que son la mitad del
//! código y la mitad de las decisiones difíciles— se probarían el día del
//! lanzamiento, con los contactos de un cliente.
//!
//! Es la misma idea que el proveedor simulado de la Fase 4, un piso más abajo:
//! allí se simulaba **el proveedor** y aquí **el servidor**. El adaptador que
//! hablará con Gmail es exactamente el mismo que habla con éste.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! HABLA EL PROTOCOLO DE VERDAD
//!
//! No es un doble que devuelve respuestas: acepta una conexión, saluda con su
//! `220`, entiende `EHLO`, `AUTH`, `MAIL FROM`, `RCPT TO`, `DATA` y `QUIT`, y
//! respeta el punto solo que cierra el cuerpo. Un doble que se saltara el
//! protocolo dejaría sin probar justo la parte que se puede equivocar.
//!
//! **Y guarda lo que le dijeron.** Eso permite afirmar cosas que de otro modo
//! habría que suponer: que el adaptador mandó `STARTTLS` antes de la
//! contraseña, o que a una persona dada de baja **no se le mandó ni un
//! `RCPT TO`**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CONVERSACIÓN NO SABE POR DÓNDE VIAJA
//!
//! [`conversar`] trabaja sobre cualquier cosa que se pueda leer y escribir, no
//! sobre un socket. Es deliberado: en la entrega 5.2 el adaptador exigirá TLS
//! —sin casilla para desactivarlo—, y entonces este mismo servidor tendrá que
//! hablar cifrado. Con la conversación atada al socket, eso sería reescribirlo;
//! así es envolverlo.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Qué contesta el servidor en cada paso de la conversación.
///
/// Cada campo es la línea entera, con su código: `"250 2.0.0 Ok"`. Se escribe
/// completa y no por partes para que una prueba diga literalmente lo que el
/// servidor va a decir, sin tener que reconstruirlo mentalmente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guion {
    /// Lo primero que se dice al conectar. `220` es «adelante».
    pub saludo: String,
    /// La respuesta a `AUTH`. `235` acepta, `535` rechaza.
    pub al_autenticar: String,
    /// La respuesta a `MAIL FROM`.
    pub al_remitente: String,
    /// La respuesta a `RCPT TO`. **Aquí es donde un servidor dice que un buzón
    /// no existe**, y por eso es el campo que más se cambia en las pruebas.
    pub al_destinatario: String,
    /// La respuesta al punto final del cuerpo. Es el acuse de verdad.
    pub al_cuerpo: String,
    /// Colgar sin decir nada, en cuanto alguien conecte.
    ///
    /// Es el «se cayó la red» de un servidor que sí está escuchando, que es
    /// distinto de que el puerto esté cerrado: la conexión se establece y
    /// muere después.
    pub cuelga_al_saludar: bool,
    /// ¿Anuncia `STARTTLS` en la respuesta al `EHLO`?
    ///
    /// Poniéndolo en `false` se consigue **un servidor que sólo habla en
    /// claro**, que es el caso que de verdad importa comprobar: ARLES tiene que
    /// negarse a hablar con él en vez de seguir sin cifrar (§29). Es también lo
    /// que haría un atacante capaz de tocar la respuesta al `EHLO`: borrar esa
    /// línea y esperar a que el cliente se conforme.
    pub ofrece_starttls: bool,
}

impl Default for Guion {
    fn default() -> Self {
        Self::todo_bien()
    }
}

impl Guion {
    /// El camino feliz: todo se acepta.
    #[must_use]
    pub fn todo_bien() -> Self {
        Self {
            saludo: "220 mentira.arles ESMTP listo".to_owned(),
            al_autenticar: "235 2.7.0 Authentication successful".to_owned(),
            al_remitente: "250 2.1.0 Ok".to_owned(),
            al_destinatario: "250 2.1.5 Ok".to_owned(),
            al_cuerpo: "250 2.0.0 Ok: queued as MENTIRA1".to_owned(),
            cuelga_al_saludar: false,
            ofrece_starttls: true,
        }
    }

    /// Un servidor que **sólo habla en claro**: no anuncia `STARTTLS`.
    ///
    /// Es el caso que decide si el §29 es una regla o una intención. ARLES tiene
    /// que negarse a hablar con él, no seguir sin cifrar.
    #[must_use]
    pub fn sin_cifrado() -> Self {
        Self {
            ofrece_starttls: false,
            ..Self::todo_bien()
        }
    }

    /// El servidor rechaza al destinatario con la línea que se le diga.
    ///
    /// Es el caso que decide si esa persona entra en la lista de bajas, así que
    /// la línea se escribe entera a propósito: `"550 5.1.1 No such user"` dice
    /// algo muy distinto de `"450 4.2.1 Mailbox busy"`, y la diferencia entre
    /// las dos es irreversible.
    #[must_use]
    pub fn rechaza_al_destinatario(linea: &str) -> Self {
        Self {
            al_destinatario: linea.to_owned(),
            ..Self::todo_bien()
        }
    }

    /// La contraseña no sirve.
    #[must_use]
    pub fn credencial_invalida() -> Self {
        Self {
            al_autenticar: "535 5.7.8 Username and Password not accepted".to_owned(),
            ..Self::todo_bien()
        }
    }

    /// El servidor está saturado y lo dice **de forma temporal**.
    ///
    /// Un `4xx` autenticando no significa que la contraseña esté mal. La
    /// diferencia importa: confundirlo con un `535` manda a alguien a generar
    /// una contraseña de aplicación nueva —y a tirar la que funcionaba— para
    /// arreglar algo que se arregla solo en diez minutos.
    #[must_use]
    pub fn ocupado_al_autenticar() -> Self {
        Self {
            al_autenticar: "421 4.7.0 Too many connections, try again later".to_owned(),
            ..Self::todo_bien()
        }
    }

    /// Acepta la conexión y cuelga sin saludar.
    #[must_use]
    pub fn se_cae_al_conectar() -> Self {
        Self {
            cuelga_al_saludar: true,
            ..Self::todo_bien()
        }
    }
}

/// El servidor levantado y escuchando.
///
/// Al soltarlo deja de escuchar. **No hace falta acordarse de apagarlo**: una
/// prueba que se olvide dejaría un puerto abierto y un hilo vivo, y la
/// siguiente heredaría la conversación de la anterior.
#[derive(Debug)]
pub struct ServidorDeMentira {
    direccion: SocketAddr,
    dicho: Arc<Mutex<Vec<String>>>,
    seguir: Arc<AtomicBool>,
    hilo: Option<JoinHandle<()>>,
}

impl ServidorDeMentira {
    /// Levanta el servidor en un puerto libre de `localhost`.
    ///
    /// **Puerto cero**: lo elige el sistema. Un puerto fijo haría que dos
    /// pruebas que corren a la vez se pelearan por él, y el fallo aparecería
    /// una de cada diez veces — que es peor que fallar siempre.
    ///
    /// # Errores
    ///
    /// El error de red si no se puede escuchar en `localhost`.
    pub fn arrancar(guion: Guion) -> std::io::Result<Self> {
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
                let Ok(mut flujo) = conexion else { return };
                // Este servidor no sabe cifrar: si el cliente pide `STARTTLS`
                // se le contesta que adelante y ahí se acaba la conexión, que
                // es lo que un servidor roto haría. El que sí sabe cifrar es
                // `ServidorCifrado`, y vive en las pruebas para que `rustls` no
                // entre en el binario que se entrega.
                let conversacion = conversar(&mut flujo, &guion);
                if let Ok(mut d) = dicho_del_hilo.lock() {
                    d.extend(conversacion.dicho);
                }
            }
        });

        Ok(Self {
            direccion,
            dicho,
            seguir,
            hilo: Some(hilo),
        })
    }

    /// Dónde está escuchando. Es lo que se le pasa al adaptador.
    #[must_use]
    pub const fn direccion(&self) -> SocketAddr {
        self.direccion
    }

    /// Todo lo que el cliente dijo, línea a línea y en orden.
    ///
    /// Es lo que permite afirmar **lo que no se pidió**: mirar el resultado
    /// diría «no consta un envío», y mirar esto dice «no se mandó ni un
    /// `RCPT TO`», que es lo que de verdad importa cuando alguien está en la
    /// lista de bajas.
    #[must_use]
    pub fn dicho(&self) -> Vec<String> {
        self.dicho.lock().map(|d| d.clone()).unwrap_or_default()
    }

    /// ¿El cliente llegó a mandar un comando que empiece así?
    ///
    /// Se compara **sin distinguir mayúsculas**: el protocolo no las distingue
    /// y los clientes las escriben como quieren.
    #[must_use]
    pub fn dijo(&self, comando: &str) -> bool {
        let comando = comando.to_ascii_uppercase();
        self.dicho()
            .iter()
            .any(|l| l.to_ascii_uppercase().starts_with(&comando))
    }
}

impl Drop for ServidorDeMentira {
    fn drop(&mut self) {
        self.seguir.store(false, Ordering::SeqCst);
        // El hilo está parado en `accept`, así que no se entera del cambio de
        // bandera. Se le despierta con una conexión que no dice nada: mira la
        // bandera, la ve apagada y sale. Sin esto, el hilo viviría hasta el
        // final del proceso y las pruebas irían acumulando hilos dormidos.
        let _ = TcpStream::connect(self.direccion);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

/// Cómo terminó una vuelta de conversación.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Conversacion {
    /// Todo lo que dijo el cliente, línea a línea y en orden.
    pub dicho: Vec<String>,
    /// Las cabeceras del correo que llegó tras el `DATA`, si llegó alguno.
    ///
    /// **Sólo las cabeceras**: el texto del mensaje se sigue tirando. Están
    /// aquí porque hay garantías —la cabecera de baja— que no se pueden
    /// comprobar desde ningún otro sitio. Ver `tragar_el_cuerpo`.
    pub cabeceras_del_correo: Vec<String>,
    /// El cliente pidió `STARTTLS` y ya se le contestó que adelante.
    ///
    /// Quien llama tiene que envolver el flujo en TLS y volver a entrar con
    /// [`conversar_ya_saludado`]: a partir de aquí la conversación empieza de
    /// cero —el cliente repite el `EHLO`— pero **sin saludo nuevo**, que es lo
    /// que dice el RFC 3207.
    pub hay_que_cifrar: bool,
}

/// Lleva una conversación SMTP entera y devuelve lo que dijo el cliente.
///
/// Trabaja sobre cualquier cosa legible y escribible, **no sobre un socket**.
/// Es lo que permite que el mismo código lleve la conversación cifrada sin
/// tocarla: se envuelve el flujo en TLS y se vuelve a entrar.
pub fn conversar<S: Read + Write>(flujo: &mut S, guion: &Guion) -> Conversacion {
    conversar_desde(flujo, guion, true)
}

/// Como [`conversar`], pero **sin saludar**.
///
/// Es la vuelta de después de `STARTTLS`: el canal ya está cifrado y el cliente
/// va a mandar un `EHLO` nuevo, pero el servidor no vuelve a decir `220` — eso
/// sólo pasa al abrir la conexión.
pub fn conversar_ya_saludado<S: Read + Write>(flujo: &mut S, guion: &Guion) -> Conversacion {
    conversar_desde(flujo, guion, false)
}

fn conversar_desde<S: Read + Write>(flujo: &mut S, guion: &Guion, saludar: bool) -> Conversacion {
    let mut dicho = Vec::new();
    let mut cabeceras_del_correo: Vec<String> = Vec::new();

    if guion.cuelga_al_saludar {
        return Conversacion {
            dicho,
            cabeceras_del_correo,
            hay_que_cifrar: false,
        };
    }
    if saludar && decir(flujo, &guion.saludo).is_err() {
        return Conversacion {
            dicho,
            cabeceras_del_correo,
            hay_que_cifrar: false,
        };
    }

    // El lector se queda con su propio buffer, así que el flujo se lee a través
    // de él de aquí en adelante. Escribir sigue yendo al flujo directamente.
    let mut lector = BufReader::new(Lectura(flujo));
    let mut linea = String::new();

    let hay_que_cifrar = 'conversacion: {
        loop {
            linea.clear();
            match lector.read_line(&mut linea) {
                Ok(0) | Err(_) => break 'conversacion false,
                Ok(_) => {}
            }
            let comando = linea.trim_end_matches(['\r', '\n']).to_owned();
            if comando.is_empty() {
                continue;
            }
            let mayusculas = comando.to_ascii_uppercase();
            dicho.push(comando);

            let flujo = &mut lector.get_mut().0;
            let respuesta = if mayusculas.starts_with("EHLO") || mayusculas.starts_with("HELO") {
                // Las capacidades van en varias líneas; sólo la última lleva
                // espacio tras el código. Es lo que un cliente usa para saber
                // que la lista terminó.
                if decir(flujo, "250-mentira.arles").is_err() {
                    break 'conversacion false;
                }
                if guion.ofrece_starttls {
                    if decir(flujo, "250-AUTH PLAIN LOGIN").is_err() {
                        break 'conversacion false;
                    }
                    "250 STARTTLS".to_owned()
                } else {
                    // Un servidor que sólo habla en claro. No anuncia STARTTLS,
                    // y ARLES tiene que negarse a seguir.
                    "250 AUTH PLAIN LOGIN".to_owned()
                }
            } else if mayusculas.starts_with("STARTTLS") {
                if !guion.ofrece_starttls {
                    "454 4.7.0 TLS no disponible".to_owned()
                } else {
                    let _ = decir(flujo, "220 2.0.0 Adelante con TLS");
                    // Se sale **soltando el lector**, y con él lo que tuviera
                    // en el buffer. No es un descuido: un cliente correcto no
                    // manda nada entre el `STARTTLS` y el apretón de manos, así
                    // que si hay bytes ahí los puso alguien en medio para que
                    // se ejecutaran como si hubieran llegado cifrados. Es la
                    // inyección de comandos de STARTTLS, y la defensa es
                    // exactamente ésta: tirar el buffer.
                    break 'conversacion true;
                }
            } else if mayusculas.starts_with("AUTH") {
                guion.al_autenticar.clone()
            } else if mayusculas.starts_with("MAIL FROM") {
                guion.al_remitente.clone()
            } else if mayusculas.starts_with("RCPT TO") {
                guion.al_destinatario.clone()
            } else if mayusculas.starts_with("DATA") {
                if decir(flujo, "354 Adelante, termina con un punto solo").is_err() {
                    break 'conversacion false;
                }
                // El cuerpo se traga entero hasta el punto solo. **Sus líneas
                // no se guardan**: son el correo del cliente, y esto es una
                // herramienta de pruebas, no un buzón.
                match tragar_el_cuerpo(&mut lector) {
                    Ok(cs) => cabeceras_del_correo.extend(cs),
                    Err(_) => break 'conversacion false,
                }
                guion.al_cuerpo.clone()
            } else if mayusculas.starts_with("QUIT") {
                let _ = decir(&mut lector.get_mut().0, "221 2.0.0 Adiós");
                break 'conversacion false;
            } else {
                "250 2.0.0 Ok".to_owned()
            };

            if decir(&mut lector.get_mut().0, &respuesta).is_err() {
                break 'conversacion false;
            }
        }
    };

    Conversacion {
        dicho,
        cabeceras_del_correo,
        hay_que_cifrar,
    }
}

/// Lee hasta el punto solo que cierra el cuerpo (RFC 5321 §4.1.1.4).
/// Se traga el cuerpo hasta el punto solo, **quedándose con las cabeceras**.
///
/// ── POR QUÉ AHORA SÍ SE GUARDA ALGO, SI ANTES NO ──────────────────────────
///
/// Este servidor tiraba el cuerpo entero, y el motivo escrito era bueno: «son
/// el correo del cliente, y esto es una herramienta de pruebas, no un buzón».
///
/// Lo que cambió es que hay una garantía que **sólo se puede comprobar desde
/// este lado**: que el correo lleva su `List-Unsubscribe`. Mirar lo que produce
/// la función que arma el mensaje comprobaría que esa función está bien, no que
/// lo que sale por el cable la use — el patrón del §9, que en este proyecto ha
/// aparecido veinte veces.
///
/// **Se queda con las cabeceras y nada más.** Terminan en la primera línea
/// vacía, que es donde empieza el texto que alguien escribió: eso se sigue
/// tirando, así que esto sigue sin ser un buzón.
fn tragar_el_cuerpo<S: Read>(lector: &mut BufReader<S>) -> std::io::Result<Vec<String>> {
    let mut linea = String::new();
    let mut cabeceras = Vec::new();
    let mut en_cabeceras = true;
    loop {
        linea.clear();
        if lector.read_line(&mut linea)? == 0 {
            return Ok(cabeceras);
        }
        let limpia = linea.trim_end_matches(['\r', '\n']);
        if limpia == "." {
            return Ok(cabeceras);
        }
        if en_cabeceras {
            if limpia.is_empty() {
                en_cabeceras = false;
            } else {
                cabeceras.push(limpia.to_owned());
            }
        }
    }
}

/// Escribe una línea con el final que exige el protocolo: `\r\n`, siempre.
///
/// Un `\n` solo funciona con muchos servidores y con otros no, y el fallo
/// aparece en el que no se probó.
fn decir<S: Write>(flujo: &mut S, linea: &str) -> std::io::Result<()> {
    flujo.write_all(linea.as_bytes())?;
    flujo.write_all(b"\r\n")?;
    flujo.flush()
}

/// Envoltorio para poder leer de un `&mut S` a través de un `BufReader` sin
/// perder la referencia para escribir.
struct Lectura<'a, S>(&'a mut S);

impl<S: Read> Read for Lectura<'_, S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codigos::{leer_linea, traducir};
    use arles_core::reintento::Fallo;
    use std::io::BufWriter;

    /// Un cliente SMTP mínimo, escrito a mano.
    ///
    /// Existe porque en la entrega 5.1 **todavía no hay adaptador**: sin él,
    /// este servidor sería código sin nadie que lo use, y un servidor de
    /// pruebas que nunca se ha visto funcionar no vale más que no tenerlo. En
    /// la 5.2 lo sustituye el adaptador de verdad.
    struct ClienteDeMano {
        lector: BufReader<TcpStream>,
        escritor: BufWriter<TcpStream>,
    }

    impl ClienteDeMano {
        fn conecta(a: SocketAddr) -> std::io::Result<Self> {
            let flujo = TcpStream::connect(a)?;
            Ok(Self {
                lector: BufReader::new(flujo.try_clone()?),
                escritor: BufWriter::new(flujo),
            })
        }

        fn lee(&mut self) -> String {
            let mut linea = String::new();
            let _ = self.lector.read_line(&mut linea);
            linea.trim_end_matches(['\r', '\n']).to_owned()
        }

        /// Lee saltándose las líneas intermedias de una respuesta multilínea.
        fn lee_final(&mut self) -> String {
            loop {
                let l = self.lee();
                if l.len() < 4 || l.as_bytes().get(3) != Some(&b'-') {
                    return l;
                }
            }
        }

        fn dice(&mut self, linea: &str) {
            let _ = self.escritor.write_all(linea.as_bytes());
            let _ = self.escritor.write_all(b"\r\n");
            let _ = self.escritor.flush();
        }
    }

    /// **El servidor habla SMTP de verdad, de punta a punta.**
    ///
    /// Si sólo devolviera respuestas sin entender el protocolo, la mitad de lo
    /// que el adaptador de la 5.2 puede equivocar quedaría sin probar.
    #[test]
    fn una_conversacion_entera_llega_hasta_el_acuse() {
        let s = ServidorDeMentira::arrancar(Guion::todo_bien()).expect("arranca");
        let mut c = ClienteDeMano::conecta(s.direccion()).expect("conecta");

        assert!(c.lee().starts_with("220"), "el servidor saluda primero");
        c.dice("EHLO arles");
        assert!(c.lee_final().starts_with("250"));
        c.dice("AUTH PLAIN abc");
        assert!(c.lee().starts_with("235"));
        c.dice("MAIL FROM:<ventas@t.mx>");
        assert!(c.lee().starts_with("250"));
        c.dice("RCPT TO:<ana@x.mx>");
        assert!(c.lee().starts_with("250"));
        c.dice("DATA");
        assert!(c.lee().starts_with("354"), "invita a mandar el cuerpo");
        c.dice("Subject: hola");
        c.dice("");
        c.dice("qué tal");
        c.dice(".");
        assert_eq!(c.lee(), "250 2.0.0 Ok: queued as MENTIRA1");
        c.dice("QUIT");
        assert!(c.lee().starts_with("221"));
    }

    /// **Guarda lo que le dijeron**, que es lo que permite afirmar lo que
    /// **no** se pidió.
    #[test]
    fn el_servidor_recuerda_lo_que_le_dijeron() {
        let s = ServidorDeMentira::arrancar(Guion::todo_bien()).expect("arranca");
        {
            let mut c = ClienteDeMano::conecta(s.direccion()).expect("conecta");
            c.lee();
            c.dice("EHLO arles");
            c.lee_final();
            c.dice("RCPT TO:<ana@x.mx>");
            c.lee();
            c.dice("QUIT");
            c.lee();
        }

        assert!(s.dijo("EHLO"));
        assert!(s.dijo("rcpt to"), "y sin distinguir mayúsculas");
        assert!(
            !s.dijo("AUTH"),
            "esto es lo que de verdad importa: poder afirmar que algo NO se \
             pidió"
        );
    }

    /// **El cuerpo del correo no se guarda.**
    ///
    /// Esto es una herramienta de pruebas, no un buzón. Guardar el cuerpo
    /// pondría el contenido de los correos en la salida de los tests, y de ahí
    /// a una bitácora de CI hay un paso.
    #[test]
    fn el_cuerpo_del_correo_no_se_guarda() {
        let s = ServidorDeMentira::arrancar(Guion::todo_bien()).expect("arranca");
        {
            let mut c = ClienteDeMano::conecta(s.direccion()).expect("conecta");
            c.lee();
            c.dice("DATA");
            c.lee();
            c.dice("Estimada Ana, le escribo por el asunto confidencial");
            c.dice(".");
            c.lee();
            c.dice("QUIT");
            c.lee();
        }

        let dicho = s.dicho().join("\n");
        assert!(dicho.contains("DATA"));
        assert!(
            !dicho.contains("confidencial"),
            "el cuerpo se traga y no se apunta: esto no es un buzón"
        );
    }

    // ─── Lo que se le puede ordenar ─────────────────────────────────────────

    /// **El servidor rechaza al destinatario cuando se le manda**, y lo que
    /// contesta es lo que decide si esa persona entra en la lista de bajas.
    ///
    /// Es la prueba de que el guion sirve para lo que existe: aquí se junta el
    /// servidor con la traducción de códigos, que son las dos mitades de esta
    /// entrega.
    #[test]
    fn lo_que_contesta_el_servidor_decide_el_destino_de_esa_persona() {
        for (linea, esperado) in [
            ("550 5.1.1 No such user", Fallo::LaDireccionNoExiste),
            ("450 4.2.1 Mailbox busy", Fallo::Pasajero),
            ("552 Mailbox full", Fallo::Pasajero),
        ] {
            let s = ServidorDeMentira::arrancar(Guion::rechaza_al_destinatario(linea))
                .expect("arranca");
            let mut c = ClienteDeMano::conecta(s.direccion()).expect("conecta");
            c.lee();
            c.dice("RCPT TO:<ana@x.mx>");

            let contestado = c.lee();
            assert_eq!(contestado, linea, "el servidor dice lo que se le ordenó");

            let r = leer_linea(&contestado).expect("es una respuesta");
            assert_eq!(
                traducir(&r),
                Some(esperado),
                "«{linea}» tiene que acabar en {esperado:?}"
            );
        }
    }

    /// Una contraseña rechazada se dice con `535`, que es lo que abre el
    /// disyuntor de la cuenta.
    #[test]
    fn se_le_puede_ordenar_que_rechace_la_contrasena() {
        let s = ServidorDeMentira::arrancar(Guion::credencial_invalida()).expect("arranca");
        let mut c = ClienteDeMano::conecta(s.direccion()).expect("conecta");
        c.lee();
        c.dice("AUTH PLAIN abc");

        let r = leer_linea(&c.lee()).expect("es una respuesta");
        assert_eq!(traducir(&r), Some(Fallo::LaCuentaNoAutentica));
    }

    /// **Un servidor que acepta la conexión y cuelga no es lo mismo que un
    /// puerto cerrado.**
    ///
    /// Es el caso «se cayó la red» de un servidor que sí está: la conexión se
    /// establece y muere después. Un adaptador que sólo contemple el puerto
    /// cerrado se queda esperando aquí.
    #[test]
    fn se_le_puede_ordenar_que_cuelgue_sin_saludar() {
        let s = ServidorDeMentira::arrancar(Guion::se_cae_al_conectar()).expect("arranca");
        let mut c = ClienteDeMano::conecta(s.direccion()).expect("la conexión sí se establece");
        assert_eq!(c.lee(), "", "y muere sin decir nada");
    }

    // ─── Que no se quede nada encendido ─────────────────────────────────────

    /// **Al soltarlo deja de escuchar.**
    ///
    /// Una prueba que se olvidara de apagarlo dejaría un puerto abierto y un
    /// hilo vivo, y la siguiente heredaría la conversación de la anterior.
    #[test]
    fn al_soltarlo_deja_de_escuchar() {
        let direccion = {
            let s = ServidorDeMentira::arrancar(Guion::todo_bien()).expect("arranca");
            let d = s.direccion();
            // Se usa de verdad antes de soltarlo, para que el hilo esté en
            // marcha y no dormido en el primer `accept`.
            let mut c = ClienteDeMano::conecta(d).expect("conecta");
            c.lee();
            c.dice("QUIT");
            c.lee();
            d
        };

        // Tras soltarlo, o no hay nadie escuchando, o quien conteste no saluda.
        let apagado = match ClienteDeMano::conecta(direccion) {
            // No hay nadie escuchando: el puerto se cerró.
            Err(_) => true,
            // O alguien acepta —el sistema puede tardar en soltar el puerto—
            // pero no saluda, porque ya no hay servidor detrás.
            Ok(mut c) => c.lee().is_empty(),
        };
        assert!(
            apagado,
            "el servidor tenía que haberse apagado con su valor"
        );
    }

    /// Dos servidores a la vez no se pelean por un puerto.
    ///
    /// Se piden en el puerto cero para que los elija el sistema. Con un puerto
    /// fijo, dos pruebas en paralelo fallarían una de cada diez veces — que es
    /// peor que fallar siempre.
    #[test]
    fn dos_servidores_a_la_vez_no_chocan() {
        let a = ServidorDeMentira::arrancar(Guion::todo_bien()).expect("arranca el primero");
        let b = ServidorDeMentira::arrancar(Guion::todo_bien()).expect("arranca el segundo");
        assert_ne!(a.direccion().port(), b.direccion().port());
    }
}
