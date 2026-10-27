//! Crear una campaña desde la frontera, por el canal que sea (D-WA, tramo 3b).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE ARCHIVO COMPRUEBA, Y POR QUÉ NO BASTA CON LO DE `arles-db`
//!
//! El tramo 3a probó que `Db::crear_campana` escribe la etapa del canal que se
//! le pide. Lo que nadie había visto es **que lo que la pantalla manda llegue a
//! ser eso**: entre las dos cosas está la frontera, y ahí los datos vienen de
//! fuera y en JSON.
//!
//! Es el hueco del §9 en su sitio de siempre — las dos capas pasan lo suyo y el
//! agujero vive en la junta—, y aquí tiene una forma concreta: `PorDondeSaleIpc`
//! podía haber sido siete campos opcionales, y entonces un `{ canal:
//! 'whatsapp', cuenta: '…' }` habría armado una etapa de WhatsApp con una
//! cuenta de correo dentro sin que nada protestara hasta el `CHECK` del
//! esquema, tres capas más abajo y como un fallo de SQLite.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ SE DESERIALIZA DE VERDAD Y NO SE CONSTRUYE LA VARIANTE
//!
//! Construir `PorDondeSaleIpc::Whatsapp { … }` en Rust probaría que el `match`
//! del estrechado funciona, que es la mitad fácil. **Lo que hay que poder
//! afirmar es que la forma que la webview manda es la única que entra**, y eso
//! sólo se ve pasando por `serde_json`: es el mismo camino exacto que recorre
//! un `invoke` de Tauri.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_app_lib::campanas::{PorDondeSaleIpc, crear_campana_en};
use arles_core::ids::CompanyId;
use arles_db::{ClaveMaestra, Db};

const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
const NUMERO: &str = "01900000-0000-7000-8000-0000000000b1";
const PLANTILLA: &str = "01900000-0000-7000-8000-0000000000d1";

fn empresa() -> CompanyId {
    EMPRESA.parse().expect("id")
}

/// Base con empresa, una cuenta de correo, un número de WhatsApp con su
/// plantilla aprobada, una ventana y una plantilla de correo.
fn base() -> (tempfile::TempDir, Db, String) {
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
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                     'llavero://v', 'connected', 50, 10,
                     '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

             INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                           waba_id, phone_number_id, credential_ref,
                                           modo, calidad, limite_diario_meta,
                                           status, created_at, updated_at)
             VALUES ('{NUMERO}', '{EMPRESA}', 'Ventas', '+528100000000',
                     'waba-1', 'pn-1', 'llavero://w', 'prospeccion_directa',
                     'green', 1000, 'connected',
                     '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

             INSERT INTO whatsapp_template (id, company_id, whatsapp_account_id,
                                            meta_name, language, status, body, slots,
                                            created_at, updated_at)
             VALUES ('{PLANTILLA}', '{EMPRESA}', '{NUMERO}', 'bienvenida', 'es_MX',
                     'approved', 'Hola {{{{1}}}}.', '[]',
                     '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('vent-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                     'America/Mexico_City', '2026-09-29T00:00:00Z');"
        ))
    })
    .expect("siembra");

    let (_, version) = db
        .crear_plantilla(
            empresa(),
            "Bienvenida",
            &arles_db::plantillas::Redaccion {
                asunto: "Hola {{nombre}}".to_owned(),
                cuerpo_html: "<p>Gracias</p><p>{{baja}}</p>".to_owned(),
            },
        )
        .expect("plantilla");

    (dir, db, version.to_string())
}

/// Lo que de verdad hace un `invoke`: JSON → la variante.
fn como_lo_manda_la_pantalla(json: &str) -> Result<PorDondeSaleIpc, serde_json::Error> {
    serde_json::from_str(json)
}

/// **Una campaña de WhatsApp creada desde la frontera llega a la base como
/// etapa de WhatsApp.**
///
/// Es la prueba que este tramo existía para tener: lo que la pantalla manda,
/// deserializado por el mismo camino que un `invoke`, y comprobado en la fila
/// que el motor va a leer.
#[test]
fn lo_que_la_pantalla_manda_por_whatsapp_llega_como_etapa_de_whatsapp() {
    let (_d, db, _v) = base();

    let sale_por = como_lo_manda_la_pantalla(&format!(
        r#"{{"canal":"whatsapp","numero":"{NUMERO}","plantillaDeMeta":"{PLANTILLA}"}}"#
    ))
    .expect("la forma de WhatsApp deserializa");

    let creada = crear_campana_en(
        &db,
        empresa(),
        "WhatsApp de septiembre",
        sale_por,
        None,
        "vent-1",
        None,
        Some(100),
        true,
    )
    .expect("se crea");

    let (canal, correo, numero, plantilla): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = db
        .ejecutar_en_pruebas(|conn| {
            conn.query_row(
                "SELECT channel, email_account_id, whatsapp_account_id, whatsapp_template_id
                   FROM campaign_stage WHERE campaign_id = ?1",
                rusqlite::params![creada.id],
                |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?)),
            )
        })
        .expect("la etapa");

    assert_eq!(canal, "whatsapp");
    assert_eq!(correo, None, "ninguna cuenta de correo");
    assert_eq!(numero.as_deref(), Some(NUMERO));
    assert_eq!(plantilla.as_deref(), Some(PLANTILLA));
}

/// **Y el correo sigue entrando igual, con su firma.**
///
/// El canal nuevo no se lleva por delante el que había: es la mitad del valor
/// de este tramo y la que nadie mira cuando algo nuevo funciona.
#[test]
fn el_correo_sigue_entrando_por_su_variante() {
    let (_d, db, version) = base();

    let sale_por = como_lo_manda_la_pantalla(&format!(
        r#"{{"canal":"email","cuenta":"{CUENTA}","versionDePlantilla":"{version}","firma":null}}"#
    ))
    .expect("la forma de correo deserializa");

    let creada = crear_campana_en(
        &db,
        empresa(),
        "Septiembre",
        sale_por,
        None,
        "vent-1",
        None,
        None,
        false,
    )
    .expect("se crea");

    let (canal, correo, numero): (String, Option<String>, Option<String>) = db
        .ejecutar_en_pruebas(|conn| {
            conn.query_row(
                "SELECT channel, email_account_id, whatsapp_account_id
                   FROM campaign_stage WHERE campaign_id = ?1",
                rusqlite::params![creada.id],
                |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
            )
        })
        .expect("la etapa");

    assert_eq!(canal, "email");
    assert_eq!(correo.as_deref(), Some(CUENTA));
    assert_eq!(numero, None, "ningún número de WhatsApp");
}

/// **Una mezcla de canales NO deserializa, así que no llega a correr nada.**
///
/// Es la garantía por la que `PorDondeSaleIpc` es una variante etiquetada y no
/// siete campos opcionales. Con los campos sueltos, esto habría armado una
/// etapa de WhatsApp con una cuenta de correo dentro y el error habría
/// aparecido en el `CHECK` del esquema —tres capas más abajo, como un fallo de
/// SQLite y sin decir qué campo sobra—.
///
/// Se prueban las dos mezclas, porque son dos descuidos distintos: pegar un
/// campo de correo en WhatsApp, y al revés.
#[test]
fn una_mezcla_de_canales_no_cruza_la_frontera() {
    let de_mas = como_lo_manda_la_pantalla(&format!(
        r#"{{"canal":"whatsapp","numero":"{NUMERO}","cuenta":"{CUENTA}"}}"#
    ));
    assert!(
        de_mas.is_err(),
        "un canal de WhatsApp con una cuenta de correo dentro no puede entrar"
    );

    let al_reves = como_lo_manda_la_pantalla(&format!(
        r#"{{"canal":"email","cuenta":"{CUENTA}","versionDePlantilla":"v","numero":"{NUMERO}"}}"#
    ));
    assert!(
        al_reves.is_err(),
        "ni un canal de correo con un número de WhatsApp dentro"
    );
}

/// **Un canal que no existe tampoco entra.**
///
/// `serde` rechaza la etiqueta desconocida, así que no hace falta un `_ =>` en
/// ningún `match` de abajo — y por eso no lo hay: un comodín ahí dejaría pasar
/// en silencio lo que esta prueba exige que se rechace.
#[test]
fn un_canal_inventado_no_entra() {
    assert!(
        como_lo_manda_la_pantalla(r#"{"canal":"paloma","numero":"x"}"#).is_err(),
        "sólo existen los dos canales del núcleo"
    );
}

/// **Una etapa de WhatsApp sin plantilla elegida SÍ se crea.**
///
/// Es lo que la V16 decidió a propósito: la etapa nace al elegir canal y
/// número, y la plantilla se elige dos pasos después. Un `CHECK` en la columna
/// obligaría a preguntar en el orden contrario al que la pantalla pregunta, y
/// el primero que tropieza con eso es quien vuelve atrás a cambiar algo.
///
/// Quien la exige es `activar_campana`, probado en el tramo 3a.
#[test]
fn una_etapa_de_whatsapp_puede_nacer_sin_plantilla() {
    let (_d, db, _v) = base();

    // Y la cadena en blanco cuenta como «no se eligió», que es lo que manda un
    // desplegable vacío: tratarla como identificador daría un error de dato
    // inválido en vez de dejar la etapa a medias, que es un estado legítimo.
    let sale_por = como_lo_manda_la_pantalla(&format!(
        r#"{{"canal":"whatsapp","numero":"{NUMERO}","plantillaDeMeta":""}}"#
    ))
    .expect("deserializa");

    let creada = crear_campana_en(
        &db,
        empresa(),
        "Sin plantilla todavía",
        sale_por,
        None,
        "vent-1",
        None,
        Some(100),
        true,
    )
    .expect("se crea en borrador");

    let plantilla: Option<String> = db
        .ejecutar_en_pruebas(|conn| {
            conn.query_row(
                "SELECT whatsapp_template_id FROM campaign_stage WHERE campaign_id = ?1",
                rusqlite::params![creada.id],
                |f| f.get(0),
            )
        })
        .expect("la etapa");

    assert_eq!(plantilla, None, "nace sin plantilla, y eso es correcto");
}

/// **Un identificador inventado falla en la frontera, con la clave del dato.**
///
/// No llega a la base: el estrechado va antes de tocarla. Así el error dice qué
/// dato no vale, en vez de salir como un fallo de clave ajena de SQLite.
///
/// ── LO QUE SE COMPRUEBA ES LA CLAVE, Y NO QUE «FALLE» · 27/10/2026 ──
///
/// La primera versión de esta prueba sólo miraba que fallara y que no quedara
/// ninguna campaña escrita. **Y eso pasa igual sin el estrechado**: lo
/// descubrió una ruptura que cambiaba el `?` por un identificador al azar y
/// salió en verde, porque entonces falla la clave ajena de `campaign_stage` y
/// la transacción se deshace igual.
///
/// Las dos cosas acaban en error, y la diferencia es todo lo que importa (§95):
/// una dice **qué dato está mal** y la otra es «falló la base». Así que lo que
/// se compara es la clave que llega a la pantalla.
#[test]
fn un_numero_que_no_es_un_identificador_falla_antes_de_la_base() {
    let (_d, db, _v) = base();

    let sale_por = como_lo_manda_la_pantalla(r#"{"canal":"whatsapp","numero":"no-soy-un-uuid"}"#)
        .expect("deserializa: la forma es válida, el valor no");

    let r = crear_campana_en(
        &db,
        empresa(),
        "Con un número inventado",
        sale_por,
        None,
        "vent-1",
        None,
        Some(100),
        true,
    );

    let e = r.expect_err("un identificador que no vale no crea nada");
    assert_eq!(
        e.clave, "error.id_invalido",
        "tiene que decir qué dato no vale, no «falló la base»: {e:?}"
    );

    let cuantas: i64 = db
        .ejecutar_en_pruebas(|conn| {
            conn.query_row("SELECT count(*) FROM campaign", [], |f| f.get(0))
        })
        .expect("cuenta");
    assert_eq!(cuantas, 0, "y no deja media campaña escrita");
}
