//! El registro de actividad, contra una base de verdad · entrega 7.4.
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! POR QUÉ ESTAS PRUEBAS ESTÁN AQUÍ Y NO EN `arles-db`
//!
//! Las de `arles-db` comprueban que la consulta filtra. Éstas comprueban lo que
//! pasa **al cruzar la frontera**, que es donde vive una clase de defecto que
//! ninguna de las dos capas puede ver sola:
//!
//!   - Un cuadro de búsqueda vacío manda `""`, no `null`. Si eso llega a la
//!     consulta como un filtro, `kind = ''` no devuelve **nada** y la pantalla
//!     se queda en blanco sin que nadie haya filtrado. La capa de datos hace lo
//!     correcto con lo que recibe; el defecto está en lo que se le manda.
//!   - El recorte tiene que decir cuánto va a borrar **antes** de borrarlo, y
//!     no tocar un solo desenlace de envío. D-7C: «se recorta, pero no se
//!     borra».

#![allow(clippy::expect_used)]

use arles_app_lib::registro::{FiltroIpc, que_se_recortaria_de, recortar_registro_de, registro_de};
use arles_core::ids::CompanyId;
use arles_db::actividad::SucesoNuevo;
use arles_db::{ClaveMaestra, Db};

const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
const ETAPA: &str = "01900000-0000-7000-8000-0000000000e1";
const AHORA: &str = "2026-09-30T12:00:00Z";
/// Hace cuatro meses: al otro lado del corte bimestral, se mire cuando se mire.
const VIEJO: &str = "2026-05-30T12:00:00Z";

/// El primer suceso de un registro, sin indexar.
fn primero(sucesos: &[arles_app_lib::registro::SucesoIpc]) -> &arles_app_lib::registro::SucesoIpc {
    sucesos
        .first()
        .expect("el registro tiene al menos un suceso")
}

fn empresa() -> CompanyId {
    EMPRESA.parse().expect("id")
}

fn base() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '{AHORA}', '{AHORA}');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('c-1', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                     'llavero://c', 'connected', 50, 10, '{AHORA}', '{AHORA}');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('v-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                     'America/Mexico_City', '{AHORA}');

             INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('k-1', '{EMPRESA}', 'Clientes de agosto', 'running', '{AHORA}', '{AHORA}');

             INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                         email_account_id, execution_window_id,
                                         status, created_at, updated_at)
             VALUES ('{ETAPA}', 'k-1', 1, 'email', 'c-1', 'v-1', 'running',
                     '{AHORA}', '{AHORA}');"
        ))
    })
    .expect("siembra");
    (dir, db)
}

/// Un desenlace de envío, con su intento y su dirección.
fn envio(db: &Db, id: &str, direccion: &str, clave: &'static str) {
    db.ejecutar_en_pruebas(|c| {
        c.execute(
            "INSERT INTO message_attempt
                 (id, campaign_id, stage_id, channel, contact_address,
                  idempotency_key, state, created_at, updated_at)
             VALUES (?1, 'k-1', ?2, 'email', ?3, ?1, 'sent', ?4, ?4)",
            rusqlite::params![id, ETAPA, direccion, AHORA],
        )
        .map(|_| ())
    })
    .expect("intento");

    db.anotar_suceso(
        empresa(),
        &SucesoNuevo {
            clave,
            es_desenlace: true,
            campana: Some("k-1"),
            etapa: Some(ETAPA),
            canal: Some("email"),
            intento: Some(id),
            numero_a: None,
            numero_b: None,
        },
    )
    .expect("anota");
}

/// Un desenlace de envío **viejo**, con su intento y su fecha elegida.
///
/// ── POR QUÉ HACE FALTA, Y POR QUÉ LA PRIMERA VERSIÓN NO PROBABA NADA ──
///
/// `recortar_no_toca_los_envios` usaba un envío de hoy. Y un envío de hoy
/// sobrevive al recorte **por su fecha**, no por ser un envío: el `WHERE` pide
/// `occurred_at < corte` además de `is_outcome = 0`. Al romper el `is_outcome`
/// a propósito, la prueba siguió pasando — la garantía que afirmaba no estaba
/// verificada (§2).
///
/// Con un envío de hace cuatro meses, lo único que puede salvarlo es la regla
/// de que un desenlace no se borra jamás. Que además es el caso real: los
/// envíos del año pasado siguen ahí.
fn envio_viejo(db: &Db, id: &str, direccion: &str, cuando: &str) {
    db.ejecutar_en_pruebas(|c| {
        c.execute(
            "INSERT INTO message_attempt
                 (id, campaign_id, stage_id, channel, contact_address,
                  idempotency_key, state, created_at, updated_at)
             VALUES (?1, 'k-1', ?2, 'email', ?3, ?1, 'sent', ?4, ?4)",
            rusqlite::params![id, ETAPA, direccion, cuando],
        )?;
        c.execute(
            "INSERT INTO activity_event
                 (id, company_id, campaign_id, kind, is_outcome,
                  message_attempt_id, occurred_at)
             VALUES (?1, ?2, 'k-1', 'motor.aceptadoPorElProveedor', 1, ?1, ?3)",
            rusqlite::params![id, EMPRESA, cuando],
        )
        .map(|_| ())
    })
    .expect("envío viejo");
}

/// Un suceso de bucle con una fecha concreta, escrito a mano.
///
/// Directo al SQL porque `anotar_si_cambia` pone la fecha de ahora, y lo que
/// estas pruebas necesitan es ruido **viejo**, del otro lado del corte.
fn ruido_viejo(db: &Db, id: &str, cuando: &str) {
    db.ejecutar_en_pruebas(|c| {
        c.execute(
            "INSERT INTO activity_event
                 (id, company_id, campaign_id, kind, is_outcome, occurred_at)
             VALUES (?1, ?2, 'k-1', 'motor.fueraDeVentana', 0, ?3)",
            rusqlite::params![id, EMPRESA, cuando],
        )
        .map(|_| ())
    })
    .expect("ruido");
}

/// **Un cuadro de búsqueda vacío no filtra nada.**
///
/// ── EL DEFECTO QUE ESTO CAZA, Y POR QUÉ NO LO VE NINGUNA CAPA SOLA ──
///
/// La pantalla manda `""` cuando el cuadro está vacío —es lo que tiene un
/// `<input>`, no `null`—. Si eso llega a la consulta como filtro, `kind = ''`
/// **no devuelve nada**: la lista se queda en blanco sin que nadie haya
/// filtrado, y con el cuadro vacío delante no hay forma de entender por qué.
///
/// `arles-db` hace lo correcto con lo que recibe, y la pantalla hace lo normal
/// al mandar `""`. El hueco está justo en medio.
///
/// Verificada rompiéndola el 30/09/2026 quitando el `.filter(|s| !s.is_empty())`
/// de `registro_de`: llegan 0 de 2 y falla con «un filtro vacío no filtra».
#[test]
fn un_filtro_vacio_no_deja_la_pantalla_en_blanco() {
    let (_dir, db) = base();
    envio(&db, "a-1", "ana@t.mx", "motor.aceptadoPorElProveedor");
    envio(&db, "a-2", "beto@t.mx", "motor.aceptadoPorElProveedor");

    let r = registro_de(
        &db,
        empresa(),
        FiltroIpc {
            campana: Some(String::new()),
            clave: Some("   ".to_owned()),
            texto: Some(String::new()),
            desde: Some(String::new()),
            ..FiltroIpc::default()
        },
    )
    .expect("lee");

    assert_eq!(r.total, 2, "un filtro vacío no filtra");
    assert_eq!(r.sucesos.len(), 2);
}

/// **El registro dice cuántos hay, no cuántos enseña.**
///
/// Es lo que impide que el tope mienta: «500 de 3 412» sólo se puede escribir
/// si el total viaja al lado de la página, y `porPagina` evita que la pantalla
/// repita la cifra 500 por su cuenta.
#[test]
fn el_registro_lleva_el_total_y_el_tamano_de_pagina() {
    let (_dir, db) = base();
    envio(&db, "a-1", "ana@t.mx", "motor.aceptadoPorElProveedor");

    let r = registro_de(&db, empresa(), FiltroIpc::default()).expect("lee");
    assert_eq!(r.total, 1);
    assert_eq!(r.por_pagina, arles_db::actividad::TOPE_DE_PAGINA);
}

/// **El desplegable sigue ofreciendo las demás clases con un filtro puesto.**
///
/// Si las clases se calcularan sobre el filtro, elegir «rebotes» dejaría el
/// desplegable con una sola opción y **no habría forma de volver** sin recargar
/// la pantalla. Es un callejón sin salida de los que la auditoría A3 buscaba.
///
/// Verificada rompiéndola: pasando el filtro a `clases_de_suceso`, queda una
/// sola clase y la prueba falla.
#[test]
fn filtrar_por_una_clase_no_esconde_las_demas() {
    let (_dir, db) = base();
    envio(&db, "a-1", "ana@t.mx", "motor.aceptadoPorElProveedor");
    envio(&db, "a-2", "beto@t.mx", "reintento.direccionInexistente");

    let r = registro_de(
        &db,
        empresa(),
        FiltroIpc {
            clave: Some("reintento.direccionInexistente".to_owned()),
            ..FiltroIpc::default()
        },
    )
    .expect("lee");

    assert_eq!(r.total, 1, "la lista sí se filtra");
    assert_eq!(
        r.clases.len(),
        2,
        "el desplegable tiene que seguir ofreciendo las demás, o no hay vuelta"
    );
}

/// **La dirección cruza la frontera, y viene del intento.**
///
/// Es lo que hace legible el registro: «se aceptó un correo» sin saber para
/// quién no sirve para nada. Y sale de `message_attempt`, no de una copia en el
/// registro (P-17).
#[test]
fn el_registro_dice_a_quien_se_le_escribio() {
    let (_dir, db) = base();
    envio(&db, "a-1", "ana@empresa.mx", "motor.aceptadoPorElProveedor");

    let r = registro_de(&db, empresa(), FiltroIpc::default()).expect("lee");
    assert_eq!(
        primero(&r.sucesos).direccion.as_deref(),
        Some("ana@empresa.mx")
    );
}

/// **El recorte dice cuánto se va y cuánto se queda, antes de tocar nada.**
///
/// Borrar sin decir cuánto es lo que hace que nadie pulse el botón. Y el «se
/// quedan» es la mitad del §95 que tranquiliza: lo que se va es ruido de bucle,
/// lo que se queda son los envíos.
///
/// Verificada rompiéndola el 30/09/2026 haciendo que `que_se_recortaria_de`
/// llame a `recortar_sucesos`: la segunda consulta da 0 y falla con «consultar
/// no puede borrar».
#[test]
fn consultar_el_recorte_no_borra_nada() {
    let (_dir, db) = base();
    ruido_viejo(&db, "s-1", VIEJO);
    ruido_viejo(&db, "s-2", VIEJO);
    envio(&db, "a-1", "ana@t.mx", "motor.aceptadoPorElProveedor");

    let antes = que_se_recortaria_de(&db, empresa()).expect("consulta");
    assert_eq!(antes.se_irian, 2, "los dos sucesos de bucle viejos");
    assert_eq!(antes.se_quedan, 1, "el envío no se cuenta como recortable");

    let otra_vez = que_se_recortaria_de(&db, empresa()).expect("consulta");
    assert_eq!(
        otra_vez.se_irian, 2,
        "consultar no puede borrar: la segunda vez tiene que decir lo mismo"
    );
}

/// **Recortar borra el ruido viejo y NO toca un solo envío.**
///
/// D-7C con las palabras de Dirección: «se recorta, pero no se borra». Lo que
/// se recorta es ruido de bucle de hace más de dos meses; lo que prueba que se
/// le escribió a alguien no se toca jamás.
///
/// Verificada rompiéndola quitando el `is_outcome = 0` de `recortar_sucesos`:
/// el envío desaparece y falla con «un desenlace de envío no se borra nunca».
#[test]
fn recortar_no_toca_los_envios() {
    let (_dir, db) = base();
    ruido_viejo(&db, "s-1", VIEJO);
    // **Viejo**, del otro lado del corte: si se borra, es porque nada lo
    // protege. Con un envío de hoy lo protegería la fecha y esta prueba pasaría
    // con la regla rota.
    envio_viejo(&db, "a-1", "ana@t.mx", VIEJO);

    let despues = recortar_registro_de(&db, empresa()).expect("recorta");
    assert_eq!(
        despues.se_irian, 0,
        "después de recortar no queda nada por recortar"
    );

    let r = registro_de(&db, empresa(), FiltroIpc::default()).expect("lee");
    assert_eq!(r.total, 1, "un desenlace de envío no se borra nunca");
    assert!(primero(&r.sucesos).es_desenlace);
    assert_eq!(primero(&r.sucesos).direccion.as_deref(), Some("ana@t.mx"));
}

/// **Y el ruido reciente tampoco se toca.**
///
/// Sin esta segunda prueba, un recorte que borrara *todos* los sucesos de bucle
/// —ignorando la fecha— pasaría la de arriba: el envío seguiría ahí y el
/// «se_irian: 0» también. Y se habría perdido la explicación de por qué una
/// campaña estuvo parada esta mañana.
#[test]
fn recortar_respeta_lo_de_los_ultimos_dos_meses() {
    let (_dir, db) = base();
    ruido_viejo(&db, "s-1", VIEJO);
    // De hoy: dentro de la ventana de dos meses, se mire cuando se mire.
    ruido_viejo(&db, "s-2", AHORA);

    recortar_registro_de(&db, empresa()).expect("recorta");

    let r = registro_de(&db, empresa(), FiltroIpc::default()).expect("lee");
    assert_eq!(
        r.total, 1,
        "el ruido reciente explica lo que pasó esta semana"
    );
    assert_eq!(primero(&r.sucesos).id, "s-2");
}
