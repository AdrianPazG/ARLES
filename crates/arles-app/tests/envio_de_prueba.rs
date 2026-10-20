//! El envío de prueba, de punta a punta y contra un servidor de verdad.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE ARCHIVO EXISTE
//!
//! La entrega 5.4 dejó una cosa dicha y sin comprobar: **ARLES nunca se había
//! visto mandar un correo entero**. El adaptador estaba probado en
//! `arles-email`, el bucle del motor en `arles-motor`, y el pegamento de la
//! aplicación contra un puerto cerrado. Tres mitades, y ninguna prueba que las
//! recorriera juntas.
//!
//! Aquí se recorren: una base de verdad con su campaña y su etapa, un servidor
//! SMTP que habla el protocolo **cifrado**, y el comando de la frontera. Lo
//! único que se sustituye es el llavero del sistema, que no existe en la
//! integración continua.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! SE CIFRA DE VERDAD, Y POR ESO HIZO FALTA ABRIR EL SERVIDOR DE MENTIRA
//!
//! El adaptador no tiene ninguna forma de no exigir TLS (§29), así que un
//! servidor en claro no sirve para ver un envío completo: la conversación se
//! corta antes del `MAIL FROM`. El servidor cifrado vive en `arles-email`
//! detrás de la bandera `test-util`, y entra aquí por `dev-dependencies` — no
//! por el programa que se entrega. Medido con `cargo tree -p arles-app -e
//! normal`: `rcgen` no aparece.
//!
//! La verificación del certificado **no se desactiva**. Se confía en el
//! certificado concreto que el servidor presenta, y nada más.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE ARCHIVO NO RECORRE, Y POR QUÉ NO PUEDE
//!
//! El adaptador se construye aquí a mano y no con la fábrica de la 5.4a. No es
//! comodidad: **es imposible de otra forma**, y conviene tenerlo escrito.
//!
//! La fábrica lee la cuenta de la base y arma el adaptador con la verificación
//! de certificados normal — como debe. El servidor de mentira presenta un
//! certificado que ninguna autoridad ha firmado, así que un adaptador salido de
//! la fábrica se negaría a hablar con él, **y eso es exactamente lo que se
//! quiere que haga**. La única manera de que pasara sería que la fábrica
//! admitiera una raíz de confianza elegida desde fuera, que es la casilla de
//! «aceptar cualquier certificado» por otro camino.
//!
//! Así que lo que queda fuera de esta prueba es el tramo «de la fila de la base
//! y el llavero al adaptador». Está probado aparte, en `fabrica.rs`, con ocho
//! casos. Lo que se recorre aquí es lo que empieza en «tengo un adaptador»: el
//! envío, el cifrado, la conversación SMTP entera y la anotación.
//!
//! Dicho de otra forma: hay una costura, y está en un sitio elegido. No se
//! puede quitar sin aflojar la verificación de certificados, y no merece la
//! pena.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_app_lib::comandos::envio_de_prueba_para_tests as enviar;
use arles_core::EmailAddress;
use arles_core::ids::EmailAccountId;
use arles_db::{ClaveMaestra, Db};
use arles_email::servidor_cifrado::{ComoCifra, NOMBRE_DE_PRUEBA, ServidorCifrado};
use arles_email::{Cifrado, Contenido, Cuenta, Guion, ProveedorSmtp};

const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
const ETAPA: &str = "etapa-1";
const CONTRASENA: &str = "dieciseis-letras";

/// Base con una empresa, una cuenta, una campaña en borrador y su etapa.
///
/// La campaña está en **borrador** a propósito: el envío de prueba se hace
/// antes de activar, que es cuando sirve para algo.
fn base() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at,
                                        smtp_host, smtp_port, smtp_security, smtp_username)
             VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@telemetry.mx', 'smtp',
                     'llavero://v', 'connected', 50, 10,
                     '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z',
                     'smtp.gmail.com', 587, 'starttls', 'ventas@telemetry.mx');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('vent-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                     'America/Mexico_City', '2026-09-28T00:00:00Z');

             INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('camp-1', '{EMPRESA}', 'Prueba', 'draft',
                     '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z');

             INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                         email_account_id, execution_window_id,
                                         status, created_at, updated_at)
             VALUES ('{ETAPA}', 'camp-1', 1, 'email', '{CUENTA}', 'vent-1',
                     'draft', '2026-09-28T00:00:00Z', '2026-09-28T00:00:00Z');"
        ))
    })
    .expect("siembra");

    (dir, db)
}

/// Un adaptador apuntando al servidor de mentira, confiando en **su**
/// certificado y en ninguno más. La verificación sigue entera.
fn adaptador(s: &ServidorCifrado) -> ProveedorSmtp {
    ProveedorSmtp::nuevo(
        &Cuenta {
            // El NOMBRE, no la IP: el certificado se emitió para él.
            servidor: NOMBRE_DE_PRUEBA.to_owned(),
            puerto: Some(s.direccion().port()),
            usuario: "ventas@telemetry.mx".to_owned(),
            remitente: "ventas@telemetry.mx".to_owned(),
            nombre_visible: Some("TELEMETRY".to_owned()),
            cifrado: Cifrado::starttls().confiando_ademas_en(s.pem()),
        },
        CONTRASENA,
        Contenido::de_html(
            "Correo de prueba de ARLES RELAY",
            "<p>Si estás leyendo esto, la cuenta puede enviar.</p>",
        ),
    )
    .expect("la cuenta está bien formada")
}

fn cuenta() -> EmailAccountId {
    CUENTA.parse().expect("id")
}

fn destino(d: &str) -> EmailAddress {
    EmailAddress::parse(d).expect("dirección válida")
}

/// **ARLES manda un correo entero, y queda anotado.**
///
/// Es la prueba que la 5.4 dejó pendiente: la cadena completa —base, cuenta,
/// adaptador, TLS, conversación SMTP— recorrida de una vez.
///
/// Se comprueba lo que el servidor **recibió**, no lo que el adaptador dice que
/// mandó: `MAIL FROM`, `RCPT TO` y `DATA`. Sin eso, la prueba pasaría con un
/// adaptador que se inventara el acuse sin hablar con nadie.
///
/// Verificada rompiéndola el 28/09/2026 de dos formas: apagando el servidor
/// antes de llamar, y quitando la llamada a `registrar_envio_de_prueba`.
#[test]
fn arles_manda_un_correo_de_verdad_y_lo_deja_anotado() {
    let (_dir, db) = base();
    let servidor =
        ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");

    let r = enviar(
        &db,
        &adaptador(&servidor),
        ETAPA,
        &destino("adrian@telemetry.mx"),
        cuenta(),
    )
    .expect("el comando no falla");

    assert!(r.aceptado, "el servidor tenía que aceptarlo: {r:?}");
    assert_eq!(r.a_donde, "adrian@telemetry.mx");

    // Lo que el servidor recibió de verdad, no lo que el adaptador afirma.
    assert!(servidor.dijo("MAIL FROM"), "llegó a mandar el remitente");
    assert!(servidor.dijo("RCPT TO"), "y el destinatario");
    assert!(servidor.dijo("DATA"), "y el mensaje entero");

    // Y quedó anotado, con las tres cosas.
    let anotado = db
        .envio_de_prueba(ETAPA)
        .expect("consulta")
        .expect("tiene que constar");
    assert_eq!(anotado.a_donde, "adrian@telemetry.mx");
    assert_eq!(anotado.con_la_cuenta, CUENTA);
    assert!(anotado.sigue_valiendo(cuenta()));
}

/// **La contraseña no viaja en claro, ni siquiera en un envío de prueba.**
///
/// Se mira lo que el servidor recibió **antes** de cifrar. La comprobación es
/// por el comando `AUTH` y no por la contraseña literal: `AUTH PLAIN` la manda
/// codificada en base64, así que buscarla tal cual no fallaría nunca — es un
/// error que ya se cometió en esta fase y que costó una prueba inútil.
#[test]
fn la_contrasena_no_sale_en_claro_en_un_envio_de_prueba() {
    let (_dir, db) = base();
    let servidor =
        ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");

    enviar(
        &db,
        &adaptador(&servidor),
        ETAPA,
        &destino("adrian@telemetry.mx"),
        cuenta(),
    )
    .expect("manda");

    assert!(
        !servidor.dijo_en_claro("AUTH"),
        "la autenticación empezó antes de cifrar: {:?}",
        servidor.dicho_en_claro()
    );
}

/// **Si el servidor rechaza, no se anota nada.**
///
/// Escribir la marca antes de saber el resultado haría que el preflight diera
/// luz verde a una cuenta que no envía — que es exactamente lo que ese punto
/// existe para impedir.
///
/// Y no es un error del comando: la pantalla tiene que poder enseñar la
/// dirección y el motivo, y dejar reintentar sin salir de ahí.
///
/// Verificada rompiéndola el 28/09/2026: anotando antes de mandar, falla.
#[test]
fn si_el_servidor_rechaza_no_queda_anotado_nada() {
    let (_dir, db) = base();
    let servidor = ServidorCifrado::arrancar(Guion::credencial_invalida(), ComoCifra::Starttls)
        .expect("arranca");

    let r = enviar(
        &db,
        &adaptador(&servidor),
        ETAPA,
        &destino("adrian@telemetry.mx"),
        cuenta(),
    )
    .expect("un rechazo del servidor no es un error del comando");

    assert!(!r.aceptado);
    assert!(r.problema.is_some(), "tiene que decir qué pasó");
    assert_eq!(
        r.a_donde, "adrian@telemetry.mx",
        "la dirección se devuelve igualmente, para poder reintentar"
    );
    assert_eq!(
        db.envio_de_prueba(ETAPA).expect("consulta"),
        None,
        "una prueba que no salió no puede quedar como hecha"
    );
}

/// **Un envío de prueba no da de baja a nadie.**
///
/// Es la regla 3 de la fase. Un `550` contra la dirección que el usuario acaba
/// de escribir —casi siempre la suya— no puede acabar en la lista de bajas: se
/// suprimiría a sí mismo, y una baja no se deshace.
///
/// Verificada rompiéndola el 28/09/2026: haciendo que el comando llame a
/// `suprimir` ante `LaDireccionNoExiste`, esta prueba falla.
#[test]
fn un_envio_de_prueba_rechazado_no_da_de_baja_a_nadie() {
    let (_dir, db) = base();
    let servidor = ServidorCifrado::arrancar(
        Guion::rechaza_al_destinatario("550 5.1.1 No such user"),
        ComoCifra::Starttls,
    )
    .expect("arranca");

    let r = enviar(
        &db,
        &adaptador(&servidor),
        ETAPA,
        &destino("adrian@telemetry.mx"),
        cuenta(),
    )
    .expect("no es un error del comando");

    assert!(!r.aceptado);

    let bajas: i64 = db
        .ejecutar_en_pruebas(|c| {
            c.query_row("SELECT count(*) FROM suppression_entry", [], |f| f.get(0))
        })
        .expect("cuenta las bajas");
    assert_eq!(
        bajas, 0,
        "probar una cuenta mal configurada contra la dirección propia no puede \
         dar de baja a quien la escribió"
    );
}

/// Y la cola tampoco se toca: un envío de prueba no es un envío de campaña.
#[test]
fn un_envio_de_prueba_no_entra_en_la_cola() {
    let (_dir, db) = base();
    let servidor =
        ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");

    enviar(
        &db,
        &adaptador(&servidor),
        ETAPA,
        &destino("adrian@telemetry.mx"),
        cuenta(),
    )
    .expect("manda");

    let intentos: i64 = db
        .ejecutar_en_pruebas(|c| {
            c.query_row("SELECT count(*) FROM message_attempt", [], |f| f.get(0))
        })
        .expect("cuenta los intentos");
    assert_eq!(
        intentos, 0,
        "un envío de prueba no gasta presupuesto ni aparece en el registro de \
         envíos de ninguna campaña"
    );
}

/// El destino se usa **tal como lo normalizó el núcleo**, no como se escribió.
#[test]
fn la_direccion_se_normaliza_antes_de_mandar() {
    let (_dir, db) = base();
    let servidor =
        ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");

    let r = enviar(
        &db,
        &adaptador(&servidor),
        ETAPA,
        &destino("  Adrian@Telemetry.MX  "),
        cuenta(),
    )
    .expect("manda");

    assert!(r.aceptado);
    assert!(
        !r.a_donde.contains(' '),
        "la dirección llegó con espacios: {:?}",
        r.a_donde
    );
}

/// **Probar una cuenta desde Remitentes manda el correo y NO anota nada.**
///
/// Son dos preguntas distintas y las dos son legítimas: «¿esta cuenta puede
/// mandar?» es de la cuenta y se contesta sin campaña de por medio; «¿se hizo
/// el envío de prueba de esta etapa?» es lo que pregunta el preflight.
///
/// Anotar aquí sería el atajo cómodo y rompería la garantía de la 5.5a: una
/// marca sin etapa sobreviviría a un cambio de remitente, y entonces el punto
/// del preflight daría luz verde basándose en una prueba de otra cuenta.
///
/// Verificada rompiéndola el 29/09/2026: haciendo que
/// `probar_envio_de_la_cuenta` llame a `registrar_envio_de_prueba`, esta prueba
/// falla y `arles_manda_un_correo_de_verdad_y_lo_deja_anotado` sigue pasando —
/// que es justo el par que hay que distinguir.
#[test]
fn probar_la_cuenta_manda_el_correo_pero_no_anota_la_prueba_de_ninguna_etapa() {
    let (_dir, db) = base();
    let servidor =
        ServidorCifrado::arrancar(Guion::todo_bien(), ComoCifra::Starttls).expect("arranca");

    let r = arles_app_lib::comandos::mandar_el_correo_para_tests(
        &adaptador(&servidor),
        &destino("adrian@telemetry.mx"),
    );

    assert!(r.aceptado, "el servidor tenía que aceptarlo");
    assert!(servidor.dijo("RCPT TO"), "mandó el correo de verdad");
    assert_eq!(
        db.envio_de_prueba(ETAPA).expect("consulta"),
        None,
        "probar una cuenta no puede dar por probada una etapa: la marca del \
         preflight cuelga de la etapa y del remitente con el que se probó"
    );
}
