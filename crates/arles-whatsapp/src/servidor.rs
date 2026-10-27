//! Un servidor que habla como Cloud API, y al que se le puede ordenar fallar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO EXISTE
//!
//! La cuenta de Meta está bloqueada esperando a Operaciones (META), y cuando
//! llegue seguirá sin poder usarse desde la integración continua: cada mensaje
//! de prueba contra la API de verdad cuenta contra la calidad del número.
//!
//! Así que hay un servidor que habla el protocolo —HTTP con los cuerpos que
//! Meta devuelve— y que **contesta lo que se le mande contestar**. Es el mismo
//! oficio que el servidor SMTP de mentira de la Fase 5, y sirve para lo mismo:
//! ver el adaptador entero funcionando, incluidos los caminos que en producción
//! sólo ocurren cuando algo va mal.
//!
//! Lo que esto **no** prueba es que Meta se comporte como creemos. Eso sólo lo
//! cierra un envío real, en la máquina del usuario, con un número desechable.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! HABLA HTTP EN CLARO, Y ESO NO CONTRADICE AL §29
//!
//! El §29 dice que no hay forma de no cifrar, y es del correo: allí el cifrado
//! es una opción del protocolo que alguien podría querer apagar. Aquí no hay
//! nada que apagar — la dirección de producción es `https://graph.facebook.com`
//! y es una constante del código.
//!
//! Lo que este servidor escucha es `127.0.0.1` en un puerto que elige el
//! sistema, sólo mientras corre una prueba. No sale de la máquina.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Qué contesta el servidor al siguiente envío.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guion {
    /// Acepta y devuelve un identificador, como hace Meta.
    Acepta,
    /// Contesta con un error de Meta: estado HTTP y código.
    Falla { http: u16, codigo: u32 },
    /// Acepta la conexión y **se calla**. Reproduce el servidor que no
    /// contesta, que es distinto de uno que rechaza: aquí sí hay algo
    /// escuchando, así que la dirección y el puerto están bien.
    Enmudece,
    /// El token vale para mandar y **no para preguntar**: `debug_token`
    /// contesta 200 con el permiso de mensajería y sin el de gestión.
    ///
    /// Es el caso más probable de todos —el que la guía de configuración
    /// advierte en el paso 4— y el que no se puede reproducir con `Falla`,
    /// porque Meta no da ningún error: contesta bien y la lista viene vacía.
    TokenSinGestion,
    /// El token vale, autoriza un WABA, y ese WABA **no tiene ningún número**.
    ///
    /// Es el único de los tres que manda al panel de Meta a dar de alta un
    /// número en vez de a tocar el token.
    SinNumeros,
}

/// Lo que el servidor recibió, para poder comprobarlo.
#[derive(Debug, Clone, Default)]
pub struct LoQueLlego {
    /// La ruta pedida, tal cual: `/v21.0/pn-1/messages`.
    pub ruta: String,
    /// El valor de la cabecera `Authorization`.
    pub autorizacion: String,
    /// El cuerpo JSON, entero.
    pub cuerpo: String,
}

/// Un servidor de mentira corriendo en su hilo.
#[derive(Debug)]
pub struct ServidorDeMentira {
    direccion: SocketAddr,
    recibido: Arc<Mutex<Vec<LoQueLlego>>>,
    parar: Arc<AtomicBool>,
    hilo: Option<JoinHandle<()>>,
}

impl ServidorDeMentira {
    /// Arranca el servidor con un guion fijo.
    ///
    /// # Errores
    ///
    /// Devuelve el error de red si no puede abrir el puerto.
    pub fn arrancar(guion: Guion) -> std::io::Result<Self> {
        // Puerto 0: lo elige el sistema. Fijar uno haría que dos pruebas en
        // paralelo se pelearan por él, y eso aparece como un fallo aleatorio
        // que nadie sabe reproducir.
        let escucha = TcpListener::bind("127.0.0.1:0")?;
        let direccion = escucha.local_addr()?;
        escucha.set_nonblocking(true)?;

        let recibido = Arc::new(Mutex::new(Vec::new()));
        let parar = Arc::new(AtomicBool::new(false));

        let recibido_hilo = Arc::clone(&recibido);
        let parar_hilo = Arc::clone(&parar);

        let hilo = std::thread::spawn(move || {
            while !parar_hilo.load(Ordering::Relaxed) {
                match escucha.accept() {
                    Ok((flujo, _)) => {
                        atender(flujo, &guion, &recibido_hilo);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            direccion,
            recibido,
            parar,
            hilo: Some(hilo),
        })
    }

    /// La dirección a la que apuntar el proveedor.
    #[must_use]
    pub fn base(&self) -> String {
        format!("http://{}", self.direccion)
    }

    /// Lo que le llegó, en orden.
    #[must_use]
    pub fn recibido(&self) -> Vec<LoQueLlego> {
        self.recibido.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl Drop for ServidorDeMentira {
    fn drop(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

/// Lee una petición, la apunta, y contesta según el guion.
fn atender(mut flujo: TcpStream, guion: &Guion, recibido: &Arc<Mutex<Vec<LoQueLlego>>>) {
    let mut lector = BufReader::new(match flujo.try_clone() {
        Ok(f) => f,
        Err(_) => return,
    });

    let mut peticion = LoQueLlego::default();
    let mut largo = 0usize;

    // La línea de petición y las cabeceras, hasta la línea en blanco.
    let mut linea = String::new();
    if lector.read_line(&mut linea).is_err() {
        return;
    }
    peticion.ruta = linea.split_whitespace().nth(1).unwrap_or("").to_owned();

    loop {
        let mut cabecera = String::new();
        match lector.read_line(&mut cabecera) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => return,
        }
        let limpia = cabecera.trim_end();
        if limpia.is_empty() {
            break;
        }
        if let Some(v) = sin_prefijo(limpia, "authorization:") {
            peticion.autorizacion = v.trim().to_owned();
        }
        if let Some(v) = sin_prefijo(limpia, "content-length:") {
            largo = v.trim().parse().unwrap_or(0);
        }
    }

    let mut cuerpo = vec![0u8; largo];
    if largo > 0 && lector.read_exact(&mut cuerpo).is_err() {
        return;
    }
    peticion.cuerpo = String::from_utf8_lossy(&cuerpo).into_owned();

    // La ruta se copia ANTES de guardar la petición: el guardado la mueve, y
    // las dos rutas del descubrimiento la necesitan para decidir qué contestar.
    let ruta = peticion.ruta.clone();

    if let Ok(mut g) = recibido.lock() {
        g.push(peticion);
    }

    let respuesta = match guion {
        // ── LAS DOS RUTAS DEL DESCUBRIMIENTO (D-WA) ──────────────────────
        //
        // Van ANTES del guion y sólo en `Acepta`: son consultas, no envíos, y
        // mezclarlas con el guion haría que una prueba de «Meta rechaza el
        // mensaje» tampoco pudiera listar los números — dos cosas distintas
        // con un solo interruptor.
        Guion::Acepta if ruta.contains("debug_token") => http(
            200,
            r#"{"data":{"app_id":"1","granular_scopes":[
                {"scope":"whatsapp_business_messaging","target_ids":["waba-1"]},
                {"scope":"whatsapp_business_management","target_ids":["waba-1"]}]}}"#,
        ),
        Guion::Acepta if ruta.contains("phone_numbers") => http(
            200,
            r#"{"data":[{"verified_name":"Telemetry","display_phone_number":"+52 55 1234 5678",
                "id":"pn-1","quality_rating":"GREEN","messaging_limit_tier":"TIER_1K"}]}"#,
        ),
        Guion::Acepta => http(
            200,
            r#"{"messaging_product":"whatsapp","contacts":[{"wa_id":"5218112345678"}],"messages":[{"id":"wamid.DEMENTIRA"}]}"#,
        ),
        Guion::Falla {
            http: estado,
            codigo,
        } => http(
            *estado,
            &format!(
                r#"{{"error":{{"message":"de mentira","type":"OAuthException","code":{codigo},"fbtrace_id":"x"}}}}"#
            ),
        ),
        // Se cierra sin contestar: el cliente se queda esperando hasta su
        // tiempo máximo. Es lo que hace un servidor saturado.
        Guion::Enmudece => return,

        // Las dos caras del descubrimiento que Meta contesta **con un 200**.
        // Son las que ningún `Falla` puede reproducir, y son justo las que
        // mandan a dos sitios distintos: a generar otro token, o al panel.
        Guion::TokenSinGestion if ruta.contains("debug_token") => http(
            200,
            r#"{"data":{"app_id":"1","granular_scopes":[
                {"scope":"whatsapp_business_messaging","target_ids":["waba-1"]}]}}"#,
        ),
        Guion::SinNumeros if ruta.contains("debug_token") => http(
            200,
            r#"{"data":{"app_id":"1","granular_scopes":[
                {"scope":"whatsapp_business_management","target_ids":["waba-1"]}]}}"#,
        ),
        Guion::SinNumeros if ruta.contains("phone_numbers") => http(200, r#"{"data":[]}"#),

        // Fuera de sus dos rutas, los dos guiones nuevos no tienen nada que
        // decir: un 500 y no un 200, para que una prueba que llegue aquí por
        // error falle en vez de creerse una respuesta inventada.
        Guion::TokenSinGestion | Guion::SinNumeros => http(
            500,
            r#"{"error":{"message":"este guion solo contesta al descubrimiento","type":"OAuthException","code":1,"fbtrace_id":"x"}}"#,
        ),
    };

    let _ = flujo.write_all(respuesta.as_bytes());
    let _ = flujo.flush();
}

/// Compara sin distinguir mayúsculas, que es como se comparan las cabeceras
/// HTTP. Un `Authorization:` y un `authorization:` son la misma.
fn sin_prefijo<'a>(linea: &'a str, prefijo: &str) -> Option<&'a str> {
    let (nombre, valor) = linea.split_once(':')?;
    let esperado = prefijo.trim_end_matches(':');
    if nombre.eq_ignore_ascii_case(esperado) {
        Some(valor)
    } else {
        None
    }
}

fn http(estado: u16, cuerpo: &str) -> String {
    format!(
        "HTTP/1.1 {estado} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
        cuerpo.len()
    )
}
