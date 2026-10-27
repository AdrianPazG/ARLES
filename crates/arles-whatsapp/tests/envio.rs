//! El adaptador hablando con un servidor que contesta como Cloud API.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE SE COMPRUEBA ES LO QUE EL SERVIDOR RECIBIÓ
//!
//! No lo que el adaptador dice que mandó. Es la misma disciplina que la prueba
//! de envío de correo de la Fase 5: si sólo se mirara el valor de vuelta, un
//! adaptador que construyera mal el cuerpo pasaría igual, porque el servidor de
//! mentira contesta que sí a cualquier cosa.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTO NO PRUEBA, Y HAY QUE TENERLO ESCRITO
//!
//! Que Meta se comporte como creemos. El servidor de aquí contesta lo que le
//! decimos, así que confirma que **nuestro lado** está bien: la URL, la
//! cabecera de autorización, la forma del cuerpo y la traducción de cada
//! respuesta a una decisión del motor.
//!
//! El bloqueo META sigue abierto y es de Operaciones. Y aunque llegue, esta
//! prueba no se va a apuntar a la API de verdad: cada mensaje de prueba cuenta
//! contra la calidad del número.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_core::reintento::Fallo;
use arles_core::whatsapp::MensajeDePlantilla;
use arles_motor::proveedor::{Mensaje, Proveedor};
use arles_whatsapp::cloud::ProveedorDeWhatsApp;
use arles_whatsapp::servidor::{Guion, ServidorDeMentira};

const DESTINO: &str = "+528112345678";
const TOKEN: &str = "EAAG-de-mentira";

fn contenido() -> MensajeDePlantilla {
    MensajeDePlantilla {
        nombre: "bienvenida_distribuidores".to_owned(),
        idioma: "es_MX".to_owned(),
        valores: vec!["Ana".to_owned(), "TELEMETRY".to_owned()],
        faltaron: vec![],
    }
}

fn mandar(guion: Guion) -> (ServidorDeMentira, Result<String, Fallo>) {
    let servidor = ServidorDeMentira::arrancar(guion).expect("arranca");
    let proveedor = ProveedorDeWhatsApp::nuevo(&servidor.base(), "pn-1", TOKEN, contenido());

    let r = proveedor
        .enviar(&Mensaje {
            para: DESTINO,
            // En WhatsApp el contenido va con el proveedor: es una plantilla
            // aprobada por Meta, no texto libre.
            contenido: arles_motor::proveedor::Contenido::Whatsapp { valores: &[] },
            de_una_campana: true,
            clave_de_idempotencia: "k-1",
        })
        .map(|a| a.id_del_proveedor.unwrap_or_default());

    (servidor, r)
}

/// **ARLES manda un mensaje entero, y el servidor recibe lo que tenía que
/// recibir.**
///
/// Es lo que esta entrega venía a poder decir. Se comprueban las cuatro cosas
/// que Meta mira: a dónde se llama, con qué credencial, a quién va y qué
/// plantilla se usa.
#[test]
fn arles_manda_un_mensaje_de_plantilla_completo() {
    let (servidor, r) = mandar(Guion::Acepta);

    assert_eq!(
        r,
        Ok("wamid.DEMENTIRA".to_owned()),
        "el identificador que devuelve Meta es lo que se guarda"
    );

    let llego = servidor.recibido();
    let peticion = llego.first().expect("llegó una petición");

    assert_eq!(
        peticion.ruta, "/v21.0/pn-1/messages",
        "la ruta de Cloud API"
    );
    assert_eq!(peticion.autorizacion, format!("Bearer {TOKEN}"));
    assert!(peticion.cuerpo.contains(r#""to":"+528112345678""#));
    assert!(
        peticion
            .cuerpo
            .contains(r#""name":"bienvenida_distribuidores""#)
    );
    assert!(peticion.cuerpo.contains(r#""code":"es_MX""#));
    assert!(peticion.cuerpo.contains(r#""text":"Ana""#));
}

/// **ARLES no manda el texto de la plantilla. Manda su nombre.**
///
/// Es la diferencia de fondo con el correo, y la que cierra por construcción la
/// puerta de la evasión: desde ARLES no se puede alterar lo que la gente lee en
/// un mensaje de WhatsApp.
///
/// Se comprueba **contra el cuerpo que salió por el cable**, que es el único
/// sitio donde la afirmación se puede sostener: una prueba sobre el tipo diría
/// que el campo no existe, y esto dice que no viajó.
///
/// Verificada rompiéndola el 29/09/2026: añadiendo `"body":"Hola Ana"` al
/// cuerpo en `cloud::cuerpo`, esta prueba falla.
#[test]
fn el_texto_del_mensaje_no_viaja_nunca() {
    let (servidor, _) = mandar(Guion::Acepta);
    let llego = servidor.recibido();
    let cuerpo = &llego.first().expect("llegó").cuerpo;

    // Ojo con la forma de buscarlo: `"type":"body"` sí aparece, y es legítimo
    // —«body» es el nombre del componente de la plantilla en la API de Meta—.
    // Lo que no puede aparecer es `body` como **campo con texto dentro**.
    //
    // La primera versión de esta prueba buscaba `"body"` a secas y fallaba
    // contra el cuerpo correcto: cazaba el nombre del componente en vez del
    // texto. Una prueba que falla con el código bien es tan inútil como una
    // que pasa con el código mal.
    for prohibido in [r#""body":""#, r#""text_body""#, r#""preview_url""#] {
        assert!(
            !cuerpo.contains(prohibido),
            "el cuerpo lleva «{prohibido}», y el texto lo pone Meta: {cuerpo}"
        );
    }

    // Y los únicos textos que viajan son los valores de los huecos: dos.
    assert_eq!(
        cuerpo.matches(r#""type":"text""#).count(),
        2,
        "sólo viajan los huecos, no el mensaje: {cuerpo}"
    );
}

/// **Un rechazo del destinatario da de baja. Los demás, no.**
///
/// El recorrido entero: Meta contesta con un código, el adaptador lo traduce, y
/// el motor recibe la decisión. Es donde se ve que la traducción de
/// `codigos.rs` llega hasta el final sin perderse por el camino.
///
/// `131026` es el único que lleva a una baja, y la baja no se deshace con un
/// botón. Los otros tres son igual de definitivos para **este** envío y no
/// tocan a nadie más.
#[test]
fn cada_respuesta_de_meta_llega_al_motor_como_su_decision() {
    let casos = [
        (400, 131_026, Fallo::LaDireccionNoExiste),
        (401, 190, Fallo::LaCuentaNoAutentica),
        (400, 131_031, Fallo::LaCuentaNoAutentica),
        (400, 999_999, Fallo::Pasajero),
    ];

    for (http, codigo, esperado) in casos {
        let (_s, r) = mandar(Guion::Falla { http, codigo });
        assert_eq!(r, Err(esperado), "código {codigo} con HTTP {http}");
    }
}

/// **Los límites de Meta llegan con su espera, no como un reintento cualquiera.**
///
/// Y las tres esperas son distintas. Con una sola, o la campaña frena media
/// hora por un pico de tres segundos, o gasta tres intentos fallidos contra un
/// límite que sólo se levanta mañana — y los intentos fallidos cuentan contra
/// la calidad del número.
#[test]
fn los_limites_llegan_con_su_espera() {
    for (codigo, espera) in [
        (130_429, arles_whatsapp::codigos::ESPERA_POR_RITMO),
        (131_048, arles_whatsapp::codigos::ESPERA_POR_CALIDAD),
        (131_049, arles_whatsapp::codigos::ESPERA_DE_UN_DIA),
    ] {
        let (_s, r) = mandar(Guion::Falla { http: 400, codigo });
        assert_eq!(
            r,
            Err(Fallo::NoMeMandesNadaDurante(espera)),
            "código {codigo}"
        );
    }
}

/// **Un servidor que se calla es pasajero, y no da de baja a nadie.**
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTA PRUEBA SUSTITUYE A UNA QUE NO VALÍA
///
/// La primera versión apuntaba el adaptador a un puerto cerrado para simular un
/// corte de red. Pasaba, y **no probaba nada**: el proxy del entorno de
/// desarrollo intercepta las salidas y devuelve un 403, así que lo que se
/// estaba ejerciendo era la rama del estado HTTP, no la del fallo de red. Se
/// descubrió rompiendo la rama a propósito y viendo que la prueba seguía en
/// verde (§2).
///
/// Ésta usa un servidor de verdad que acepta la conexión y no contesta. Eso sí
/// agota el tiempo de espera del cliente, que es el camino que se quería
/// ejercer. Lo importante: sin haber hablado con Meta no sabemos **nada** del
/// destino, así que un corte de red no puede borrar a nadie de las campañas
/// futuras.
#[test]
fn un_servidor_que_se_calla_no_da_de_baja_a_nadie() {
    let servidor = ServidorDeMentira::arrancar(Guion::Enmudece).expect("arranca");
    let proveedor = ProveedorDeWhatsApp::nuevo(&servidor.base(), "pn-1", TOKEN, contenido());

    let r = proveedor.enviar(&Mensaje {
        para: DESTINO,
        // En WhatsApp el contenido va con el proveedor: es una plantilla
        // aprobada por Meta, no texto libre.
        contenido: arles_motor::proveedor::Contenido::Whatsapp { valores: &[] },
        de_una_campana: true,
        clave_de_idempotencia: "k-1",
    });

    assert_eq!(
        r,
        Err(Fallo::Pasajero),
        "sin respuesta no se sabe nada del destino, así que no se le da de baja"
    );
}

/// **El token viaja en la cabecera, y no en la ruta ni en el cuerpo.**
///
/// Las rutas y los cuerpos acaban en registros de servidor, en trazas y en
/// informes de error. La cabecera de autorización también puede acabar ahí,
/// pero es el sitio donde todo el mundo sabe que hay que buscar un secreto para
/// taparlo — una URL con el token dentro se copia y se pega sin pensar.
#[test]
fn el_token_va_en_la_cabecera_y_en_ningun_otro_sitio() {
    let (servidor, _) = mandar(Guion::Acepta);
    let llego = servidor.recibido();
    let p = llego.first().expect("llegó");

    assert!(p.autorizacion.contains(TOKEN), "va en la cabecera");
    assert!(!p.ruta.contains(TOKEN), "y no en la ruta: {}", p.ruta);
    assert!(!p.cuerpo.contains(TOKEN), "ni en el cuerpo");
}

// ═══════════════════════════════════════════════════════════════════════════
// EL DESCUBRIMIENTO · D-WA, 26/10/2026
//
// El alta pedía SEIS datos, cuatro de ellos identificadores que sólo existen
// en el panel de Meta y hay que copiar sin equivocarse. Con el token, Meta
// dice el resto: queda un dato y una elección.
//
// Estas pruebas van contra el servidor de mentira y no contra las funciones
// sueltas —que ya tienen las suyas en el módulo— porque lo que hay que poder
// afirmar es que **las dos llamadas encadenan**: la primera saca el WABA y la
// segunda lo usa. Cada mitad puede estar bien y la junta no.
// ═══════════════════════════════════════════════════════════════════════════

/// **Con sólo el token, Meta dice el número entero.**
///
/// Es la prueba de que el alta se puede reducir: todo lo que el formulario
/// pedía a mano —WABA, Phone Number ID, teléfono y nombre— llega de la fuente.
#[test]
fn con_el_token_se_descubre_el_numero_entero() {
    let servidor = ServidorDeMentira::arrancar(Guion::Acepta).expect("arranca");

    let numeros =
        arles_whatsapp::numeros_del_token(&servidor.base(), TOKEN).expect("Meta contesta");

    assert_eq!(numeros.len(), 1, "el servidor de mentira tiene uno");
    let n = numeros.first().expect("uno");
    assert_eq!(n.phone_number_id, "pn-1");
    assert_eq!(n.waba_id, "waba-1", "el WABA salió de debug_token");
    assert_eq!(n.telefono, "+52 55 1234 5678");
    assert_eq!(n.nombre, "Telemetry");
    assert_eq!(n.calidad, "GREEN");
    assert_eq!(n.tope_diario, Some(1_000), "TIER_1K");
}

/// **Las dos llamadas encadenan, y se comprueba que la segunda usa el WABA de
/// la primera.**
///
/// Es la junta: con el WABA mal pasado, la segunda llamada iría a otra ruta y
/// la lista saldría vacía — que se leería como «no tienes números» delante de
/// alguien que sí los tiene.
#[test]
fn la_segunda_llamada_usa_el_waba_de_la_primera() {
    let servidor = ServidorDeMentira::arrancar(Guion::Acepta).expect("arranca");

    let _ = arles_whatsapp::numeros_del_token(&servidor.base(), TOKEN).expect("Meta contesta");

    let pedidos = servidor.recibido();
    assert_eq!(pedidos.len(), 2, "una por paso");
    assert!(
        pedidos
            .first()
            .expect("la primera")
            .ruta
            .contains("debug_token"),
        "primero se pregunta qué autoriza el token"
    );
    assert!(
        pedidos
            .get(1)
            .expect("la segunda")
            .ruta
            .contains("waba-1/phone_numbers"),
        "y después se piden los números DE ESE WABA"
    );
}

/// **Un token que Meta rechaza manda a revisar el token, no a esperar.**
#[test]
fn un_token_rechazado_se_distingue_de_una_red_caida() {
    let servidor = ServidorDeMentira::arrancar(Guion::Falla {
        http: 401,
        codigo: 190,
    })
    .expect("arranca");

    let r = arles_whatsapp::numeros_del_token(&servidor.base(), TOKEN);
    assert_eq!(r, Err(arles_whatsapp::NoSePudoPreguntar::ElTokenNoVale));
}

/// **Un token que sirve para mandar y no para preguntar manda a cambiar el
/// token, no al panel de Meta.**
///
/// Es el fallo más probable del alta —la guía lo advierte en el paso 4— y el
/// que ningún error de Meta delata: `debug_token` contesta **200**, con el
/// permiso de mensajería y sin el de gestión, y la lista de WABA viene vacía.
///
/// Los dos desenlaces posibles mandan a sitios opuestos: «genera otro token
/// con los dos permisos» o «ve al panel a dar de alta un número». Confundirlos
/// es media tarde buscando un número que sí existe.
///
/// **Esta prueba existe porque su ruptura salió en verde** (§2, 27/10/2026):
/// cambiar este desenlace a `NingunNumero` no hacía fallar nada, así que la
/// distinción entre los dos motivos no estaba verificada por ninguna de las
/// once pruebas del módulo. Verificada ahora rompiéndola **en los dos
/// sentidos**, con `una_cuenta_sin_numeros_no_acusa_al_token`: cada una caza
/// la suya y ninguna caza las dos.
#[test]
fn un_token_sin_el_permiso_de_gestion_no_parece_una_cuenta_sin_numeros() {
    let servidor = ServidorDeMentira::arrancar(Guion::TokenSinGestion).expect("arranca");

    let r = arles_whatsapp::numeros_del_token(&servidor.base(), TOKEN);
    assert_eq!(
        r,
        Err(arles_whatsapp::NoSePudoPreguntar::ElTokenNoVale),
        "sin el permiso de gestión no hay dónde mirar: el token es lo que falta"
    );
}

/// **Y al revés: una cuenta de verdad sin ningún número manda al panel.**
///
/// Aquí el token está perfecto. Decirle a quien lo acaba de generar con los dos
/// permisos que «el token no vale» le haría generar otro, y otro.
#[test]
fn una_cuenta_sin_numeros_no_acusa_al_token() {
    let servidor = ServidorDeMentira::arrancar(Guion::SinNumeros).expect("arranca");

    let r = arles_whatsapp::numeros_del_token(&servidor.base(), TOKEN);
    assert_eq!(r, Err(arles_whatsapp::NoSePudoPreguntar::NingunNumero));
}

/// **Meta callada no es un token malo**, y mandar a revisar el token por eso
/// es una tarde perdida buscando donde no está.
#[test]
fn meta_callada_no_acusa_al_token() {
    let servidor = ServidorDeMentira::arrancar(Guion::Enmudece).expect("arranca");

    let r = arles_whatsapp::numeros_del_token(&servidor.base(), TOKEN);
    assert_eq!(r, Err(arles_whatsapp::NoSePudoPreguntar::NoSeLlego));
}
