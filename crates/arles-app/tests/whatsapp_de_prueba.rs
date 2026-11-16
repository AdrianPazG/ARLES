//! El envío de prueba de WhatsApp, de punta a punta · entrega W.8
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTA PRUEBA TENÍA QUE EXISTIR, Y POR QUÉ NINGUNA DE LAS OTRAS VALÍA
//!
//! El canal llegó a la W.7 con setenta pruebas en verde y **sin poder mandar un
//! mensaje**. El adaptador estaba probado contra el servidor de mentira y
//! pasaba; el número se guardaba bien y pasaba; la pantalla enseñaba las cifras
//! y pasaba. Lo que no había era nadie que uniera las tres cosas.
//!
//! Esta prueba recorre **el camino entero**: lee el número de la base, pregunta
//! la puerta del canal, saca el token del llavero, arma el proveedor, manda, y
//! traduce lo que contestó Meta. Si mañana alguien vuelve a dejar una capa sin
//! conectar, esto falla.
//!
//! Va contra el servidor de mentira, no contra Meta. El bloqueo META sigue
//! abierto y se cierra en la máquina del usuario, con la guía de conexión real.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use arles_app_lib::fabrica::FuenteDeContrasenas;
use arles_app_lib::whatsapp::prueba_con;
use arles_core::ids::{CompanyId, WhatsappAccountId};
use arles_db::{ClaveMaestra, Db};
use arles_whatsapp::servidor::{Guion, ServidorDeMentira};

const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
const NUMERO: &str = "01900000-0000-7000-8000-0000000000a1";
const AHORA: &str = "2026-09-29T00:00:00Z";

/// Un llavero que contesta siempre lo mismo.
///
/// No hay llavero del sistema en un Linux sin escritorio, y aquí no hace falta:
/// lo que se mira es el camino del envío, no la custodia del secreto.
struct LlaveroDeMentira;

impl FuenteDeContrasenas for LlaveroDeMentira {
    fn contrasena_de(&self, _id: &str) -> Result<Option<String>, arles_app_lib::AppError> {
        Ok(Some("dieciseis-letras".to_owned()))
    }

    fn token_del_numero(&self, _id: &str) -> Result<Option<String>, arles_app_lib::AppError> {
        Ok(Some("EAAG-token-de-mentira".to_owned()))
    }
}

fn empresa() -> CompanyId {
    EMPRESA.parse().expect("id")
}

fn numero() -> WhatsappAccountId {
    NUMERO.parse().expect("id")
}

/// Una base con una empresa y un número, con o sin sus dos confirmaciones.
fn base(confirmado: bool) -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '{AHORA}', '{AHORA}');

             INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                           waba_id, phone_number_id, credential_ref,
                                           modo, status, created_at, updated_at)
             VALUES ('{NUMERO}', '{EMPRESA}', 'Ventas', '+528100000000', 'waba-1',
                     'pn-1', 'llavero://x/whatsapp-number/{NUMERO}', 'seguimiento',
                     'connected', '{AHORA}', '{AHORA}');"
        ))
    })
    .expect("siembra");

    if confirmado {
        db.confirmar_coexistencia(empresa(), numero(), "usuario_local")
            .expect("confirma");
    }

    (dir, db)
}

/// **El mensaje sale de verdad, y llega al servidor con su plantilla.**
///
/// Es la prueba que cierra el hueco de la W.7. No comprueba que el adaptador
/// funcione —eso ya estaba probado— sino que **alguien lo llama**, y que el
/// camino desde el comando hasta la red está completo.
///
/// Se mira lo que llegó al servidor, no solo que el comando devolviera
/// «aceptado»: un comando que dijera que sí sin mandar nada pasaría igual.
#[test]
fn un_mensaje_de_prueba_llega_al_servidor_con_su_plantilla() {
    let servidor = ServidorDeMentira::arrancar(Guion::Acepta).expect("arranca");
    let (_d, db) = base(true);

    let r = prueba_con(
        &db,
        &LlaveroDeMentira,
        empresa(),
        "MX",
        NUMERO,
        "81 1234 5678",
        "hello_world",
        "es_MX",
        &servidor.base(),
    )
    .expect("manda");

    assert!(r.aceptado, "Meta lo aceptó: {r:?}");
    assert_eq!(r.clave, "whatsapp.prueba.aceptado");

    let llegó = servidor.recibido();
    assert_eq!(llegó.len(), 1, "llegó exactamente un mensaje");
    assert!(
        llegó[0].cuerpo.contains("hello_world"),
        "con la plantilla que se pidió: {}",
        llegó[0].cuerpo
    );
    assert!(
        llegó[0].autorizacion.contains("EAAG-token-de-mentira"),
        "y con el token del llavero, no con otro"
    );
}

/// **Un número sin Coexistencia no manda nada, y no es un error de red.**
///
/// La puerta del canal se pregunta en la fábrica y no en la pantalla, porque
/// la fábrica es el único punto por el que pasa todo envío. Esta prueba entra
/// por el comando —no por la pantalla— y aun así no sale ningún mensaje.
///
/// Verificada rompiéndola el 29/09/2026: quitando el `if !numero.puede_enviar()`
/// de `proveedor_del_numero`, esta prueba falla y el servidor recibe el mensaje.
#[test]
fn sin_coexistencia_no_sale_ningun_mensaje() {
    let servidor = ServidorDeMentira::arrancar(Guion::Acepta).expect("arranca");
    let (_d, db) = base(false);

    let r = prueba_con(
        &db,
        &LlaveroDeMentira,
        empresa(),
        "MX",
        NUMERO,
        "81 1234 5678",
        "hello_world",
        "es_MX",
        &servidor.base(),
    );

    assert!(r.is_err(), "no se puede mandar todavía");
    assert!(
        servidor.recibido().is_empty(),
        "y al servidor no llegó nada: la puerta está antes de la red"
    );
}

/// **Un rechazo de Meta no es un error del comando: es el resultado.**
///
/// Es justo lo que la prueba venía a averiguar. Tratarlo como error dejaría al
/// usuario con «algo falló» en vez de «el token no vale», que es lo accionable
/// (§95).
#[test]
fn un_rechazo_de_meta_se_devuelve_traducido_y_no_como_error() {
    // 131049: Meta rechaza por límites de calidad. Es un fallo pasajero.
    let servidor = ServidorDeMentira::arrancar(Guion::Falla {
        http: 400,
        codigo: 131_049,
    })
    .expect("arranca");
    let (_d, db) = base(true);

    let r = prueba_con(
        &db,
        &LlaveroDeMentira,
        empresa(),
        "MX",
        NUMERO,
        "81 1234 5678",
        "hello_world",
        "es_MX",
        &servidor.base(),
    )
    .expect("el comando no falla: contesta");

    assert!(!r.aceptado);
    assert!(
        r.clave.starts_with("whatsapp.prueba."),
        "y con una clave de texto, no el inglés de Meta: {}",
        r.clave
    );
}
