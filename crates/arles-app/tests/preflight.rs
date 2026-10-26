//! El preflight de punta a punta: de la base a los ocho puntos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE ARCHIVO COMPRUEBA, Y POR QUÉ NO BASTABAN LAS PRUEBAS DE ANTES
//!
//! Las reglas de los ocho puntos se probaron en la 4.5, sobre unas
//! observaciones escritas a mano. Lo que nadie había visto hasta ahora es que
//! **esas observaciones se rellenen desde la base de verdad**: que la audiencia
//! sea la que queda después de las bajas, que los huecos de las variables
//! salgan de la plantilla de esta campaña, y que el envío de prueba de la 5.5
//! —que llevaba tres entregas anotándose sin que nadie lo preguntara— llegue
//! por fin al punto que lo espera.
//!
//! Un preflight con las reglas bien y las observaciones mal da exactamente el
//! mismo resultado siempre, y nada falla.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ NUNCA SALE «LISTA», Y ESO NO ES UN AGUJERO DE LA PRUEBA
//!
//! El punto de la conexión sólo pasa si la sonda habla con un servidor de
//! verdad, y la fábrica —como debe— no acepta el certificado sin firmar del
//! servidor de mentira. Está explicado en `envio_de_prueba.rs`: la única forma
//! de que pasara sería que la fábrica admitiera una raíz de confianza elegida
//! desde fuera, que es la casilla de «aceptar cualquier certificado» por otro
//! camino.
//!
//! Así que lo que se comprueba aquí es lo que sí se puede comprobar sin
//! aflojar nada: **que cada observación llega a su punto**, y que los puntos
//! que no se pudieron mirar se dicen como lo que son. El caso en que todo pasa
//! está probado en `arles_core::preflight`, sobre las mismas reglas.
//!
//! R-24 sigue abierto por lo mismo, y se cierra en la máquina del usuario.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_app_lib::comandos::revision_de;
use arles_app_lib::fabrica::FuenteDeContrasenas;
use arles_core::ids::{CampaignId, CompanyId};
use arles_core::preflight::{Punto, Veredicto};
use arles_db::alta_de_campana::CampanaNueva;
use arles_db::plantillas::Redaccion;
use arles_db::{ClaveMaestra, Db};

const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";

/// Un llavero que contesta siempre lo mismo.
///
/// No hay llavero del sistema en un Linux sin escritorio, y tampoco hace falta:
/// lo que esta prueba mira es el preflight, no la custodia del secreto. Esa
/// tiene sus ocho casos en `fabrica.rs`.
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

/// Base con empresa, cuenta, ventana, plantilla con variable, y tres contactos
/// de los que uno está dado de baja y otro no tiene nombre.
///
/// El servidor de la cuenta apunta a un puerto cerrado a propósito: así la
/// sonda da «no se llegó al servidor», que es el caso que deja ver los otros
/// siete puntos sin que ninguno quede tapado por un fallo de red distinto.
///
/// `a.localhost` y no `localhost`: un servidor de correo sin punto no es un
/// nombre de dominio y no pasa la validación del núcleo.
fn base() -> (tempfile::TempDir, Db, CampaignId) {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at,
                                        smtp_host, smtp_port, smtp_security, smtp_username)
             VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@telemetry.mx', 'smtp',
                     'llavero://v', 'connected', 50, 10,
                     '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z',
                     'a.localhost', 1, 'starttls', 'ventas@telemetry.mx');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('vent-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                     'America/Mexico_City', '2026-09-29T00:00:00Z');"
        ))
    })
    .expect("siembra");

    contacto(&db, "c1", "uno@t.mx", Some("Ana"));
    contacto(&db, "c2", "dos@t.mx", None);
    contacto(&db, "c3", "tres@t.mx", Some("Caro"));

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                            scope, reason, origin, created_at)
             VALUES ('s1', '{EMPRESA}', 'email', 'tres@t.mx', 'global',
                     'unsubscribe', 'user', '2026-09-29T00:00:00Z');"
        ))
    })
    .expect("baja");

    let (_, version) = db
        .crear_plantilla(
            empresa(),
            "Bienvenida",
            &Redaccion {
                asunto: "Hola {{nombre}}".to_owned(),
                cuerpo_html: "<p>Gracias por escribirnos.</p>".to_owned(),
            },
        )
        .expect("plantilla");

    let campana = db
        .crear_campana(
            empresa(),
            &CampanaNueva {
                lista: None,
                nombre: "Septiembre",
                cuenta: CUENTA.parse().expect("id"),
                version_de_plantilla: version,
                firma: None,
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: None,
                // Sin tope propio no se pasa de 50: el de la cuenta manda.
                acepto_el_riesgo_del_volumen: false,
            },
        )
        .expect("campaña")
        .id;

    (dir, db, campana)
}

fn contacto(db: &Db, id: &str, correo: &str, nombre: Option<&str>) {
    let pila = nombre.map_or("NULL".to_owned(), |n| format!("'{n}'"));
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO contact (id, company_id, first_name, status, created_at, updated_at)
             VALUES ('{id}', '{EMPRESA}', {pila}, 'active',
                     '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
             INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                          value_raw, value_normalized, status,
                                          created_at, updated_at)
             VALUES ('ch-{id}', '{EMPRESA}', '{id}', 'email', '{correo}', '{correo}',
                     'active', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
        ))
    })
    .expect("contacto");
}

fn veredicto(revision: &arles_app_lib::comandos::RevisionDeCampana, punto: Punto) -> Veredicto {
    revision
        .preflight
        .puntos
        .iter()
        .find(|p| p.punto == punto)
        .map(|p| p.veredicto)
        .unwrap_or_else(|| panic!("el preflight tiene que traer {punto:?}"))
}

/// **Las cifras del preflight salen de la base, no de un valor por defecto.**
///
/// Tres contactos, uno dado de baja, uno sin nombre: dos destinatarios y un
/// hueco. Si las observaciones no se rellenaran de verdad, el preflight daría
/// el mismo veredicto —bloqueada, porque falta la conexión— con cifras
/// inventadas, y nadie lo notaría.
///
/// Verificada rompiéndola el 29/09/2026: pasando `audiencia: 0` fijo en
/// `revision_de`, esta prueba falla con «0 en vez de 2» y además el punto de
/// la audiencia pasa a fallar.
#[test]
fn las_cifras_del_preflight_salen_de_la_base() {
    let (_d, db, campana) = base();
    let r = revision_de(&db, &LlaveroDeMentira, empresa(), campana, None).expect("revisa");

    assert_eq!(r.audiencia, 2, "tres contactos menos la dada de baja");
    assert_eq!(r.contactos_con_huecos, 1, "el que no tiene nombre");
    assert_eq!(
        veredicto(&r, Punto::LaAudienciaNoEstaVacia),
        Veredicto::Pasa
    );
}

/// **El envío de prueba de la 5.5 llega por fin al punto que lo esperaba.**
///
/// Es lo que esta entrega venía a enchufar: la marca se anotaba desde la 5.5 y
/// nadie la leía. Sin esto, el punto diría «no se hizo» para siempre, con el
/// correo de prueba ya mandado y recibido.
///
/// Verificada rompiéndola el 29/09/2026: fijando
/// `se_hizo_el_envio_de_prueba: Some(false)` en `revision_de`, la segunda
/// mitad de esta prueba falla.
#[test]
fn el_envio_de_prueba_anotado_hace_pasar_su_punto() {
    let (_d, db, campana) = base();

    let antes = revision_de(&db, &LlaveroDeMentira, empresa(), campana, None).expect("revisa");
    assert_eq!(
        veredicto(&antes, Punto::SeHizoElEnvioDePrueba),
        Veredicto::Falla,
        "todavía no se ha mandado ninguno"
    );

    let etapa: String = db
        .ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT id FROM campaign_stage WHERE campaign_id = ?1",
                [campana.to_string()],
                |f| f.get(0),
            )
        })
        .expect("etapa");

    db.registrar_envio_de_prueba(&etapa, "yo@t.mx", CUENTA.parse().expect("id"))
        .expect("anota");

    let despues = revision_de(&db, &LlaveroDeMentira, empresa(), campana, None).expect("revisa");
    assert_eq!(
        veredicto(&despues, Punto::SeHizoElEnvioDePrueba),
        Veredicto::Pasa
    );
}

/// **Un puerto cerrado es «no hay conexión», no «la cuenta no autentica».**
///
/// Es la distinción que `Diagnostico::autentica` existe para mantener, y aquí
/// se ve de punta a punta: el punto de la conexión falla, el de la
/// autenticación queda **sin comprobar** porque no se llegó a intentar, y el
/// diagnóstico dice cuál de los dos.
///
/// El error cómodo sería marcar los dos como fallidos: el botón quedaría
/// apagado igual, y el usuario se pasaría la tarde generando contraseñas de
/// aplicación nuevas por un puerto cerrado.
#[test]
fn un_puerto_cerrado_no_acusa_a_la_contrasena() {
    let (_d, db, campana) = base();
    let r = revision_de(&db, &LlaveroDeMentira, empresa(), campana, None).expect("revisa");

    assert_eq!(veredicto(&r, Punto::HayConexion), Veredicto::Falla);
    assert_eq!(
        veredicto(&r, Punto::LaCuentaAutentica),
        Veredicto::SinComprobar,
        "no se llegó a probar la contraseña"
    );
    assert_eq!(
        r.diagnostico.as_deref(),
        Some("correo.prueba.no_se_llego"),
        "y se dice cuál de los dos fue"
    );
    assert!(!r.preflight.resultado.se_puede_activar());
}

/// **Una cuenta que no puede enviar no se sondea, y el texto no culpa a la red.**
///
/// Con la cuenta desactivada no hay nada que preguntarle al servidor. Sondear
/// igualmente daría «no se llegó al servidor» y mandaría a revisar el cortafuegos
/// por una cuenta que alguien apagó a propósito.
#[test]
fn una_cuenta_desactivada_no_se_sondea() {
    let (_d, db, campana) = base();
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "UPDATE email_account SET status = 'disabled' WHERE id = '{CUENTA}';"
        ))
    })
    .expect("desactiva");

    let r = revision_de(&db, &LlaveroDeMentira, empresa(), campana, None).expect("revisa");

    assert_eq!(veredicto(&r, Punto::LaCuentaAutentica), Veredicto::Falla);
    assert_eq!(
        veredicto(&r, Punto::HayConexion),
        Veredicto::SinComprobar,
        "no se llegó a mirar la red"
    );
    assert_eq!(r.diagnostico, None, "no hubo sonda que contar");
}

/// **Los registros del dominio se dicen «sin comprobar», y no bloquean.**
///
/// ARLES no consulta DNS todavía (P-16). Es el único de los ocho que avisa en
/// vez de bloquear, así que lo que hay que ver es que **no** aparezca entre lo
/// que impide activar: si contara, ninguna campaña se activaría nunca y la
/// causa sería invisible — el punto no falla, es que no se pudo mirar.
#[test]
fn los_registros_del_dominio_avisan_y_no_bloquean() {
    let (_d, db, campana) = base();
    let r = revision_de(&db, &LlaveroDeMentira, empresa(), campana, None).expect("revisa");

    assert_eq!(
        veredicto(&r, Punto::LosRegistrosDelDominio),
        Veredicto::SinComprobar
    );
    assert!(
        !Punto::LosRegistrosDelDominio.bloquea(),
        "y no cuenta para el veredicto"
    );
}
