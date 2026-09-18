//! Los invariantes del esquema, verificados contra una base real.
//!
//! Estos tests existen porque `MODELO_DE_DATOS.md` afirma que ciertas garantías
//! son **propiedades del esquema**, no del código. Si alguien relaja una
//! restricción, la afirmación deja de ser cierta y estos tests lo detectan.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_db::{ClaveMaestra, abrir};
use rusqlite::{Connection, params};

const EMPRESA: &str = "01900000-0000-7000-8000-000000000001";
const CAMPANA: &str = "01900000-0000-7000-8000-0000000000c1";
const CONTACTO_A: &str = "01900000-0000-7000-8000-00000000000a";
const CONTACTO_B: &str = "01900000-0000-7000-8000-00000000000b";
const AHORA: &str = "2026-09-11T00:00:00Z";
/// Móviles ya en forma canónica E.164, que es como los guarda
/// `arles_core::PhoneNumber`: +52 y diez dígitos, sin el «1» que WhatsApp
/// arrastra de antes de 2019.
const MOVIL_A: &str = "+528112345678";
const MOVIL_B: &str = "+528198765432";

fn base() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().expect("directorio temporal");
    let clave = ClaveMaestra::generar().expect("genera clave");
    let conn = abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    conn.execute(
        "INSERT INTO company (id, commercial_name, country, timezone,
                              corporate_email, created_at, updated_at)
         VALUES (?1, 'TELEMETRY INSIGHT', 'MX', 'America/Mexico_City',
                 'hola@telemetry.mx', ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("inserta empresa");

    for (id, correo, movil) in [
        (CONTACTO_A, "ana@empresa.com", MOVIL_A),
        (CONTACTO_B, "beto@empresa.com", MOVIL_B),
    ] {
        conn.execute(
            "INSERT INTO contact (id, company_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)",
            params![id, EMPRESA, AHORA],
        )
        .expect("inserta contacto");

        // Desde la V3 la dirección no vive en `contact`: es un canal (L-2).
        insertar_canal(&conn, &format!("ch-e-{id}"), id, "email", correo);
        insertar_canal(&conn, &format!("ch-w-{id}"), id, "whatsapp", movil);
    }

    conn.execute(
        "INSERT INTO campaign (id, company_id, name, created_at, updated_at)
         VALUES (?1, ?2, 'Clientes Q1', ?3, ?3)",
        params![CAMPANA, EMPRESA, AHORA],
    )
    .expect("inserta campaña");

    (dir, conn)
}

fn insertar_canal(conn: &Connection, id: &str, contacto: &str, canal: &str, valor: &str) {
    conn.execute(
        "INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                      value_raw, value_normalized,
                                      created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?6)",
        params![id, EMPRESA, contacto, canal, valor, AHORA],
    )
    .expect("inserta canal");
}

/// La dirección del contacto en un canal: la clave real de idempotencia.
fn direccion_de(contacto: &str, canal: &str) -> &'static str {
    match (contacto, canal) {
        (CONTACTO_A, "email") => "ana@empresa.com",
        (CONTACTO_B, "email") => "beto@empresa.com",
        (CONTACTO_A, "whatsapp") => MOVIL_A,
        (CONTACTO_B, "whatsapp") => MOVIL_B,
        _ => "desconocido@empresa.com",
    }
}

fn insertar_intento(
    conn: &Connection,
    id: &str,
    contacto: &str,
    clave: &str,
) -> rusqlite::Result<usize> {
    insertar_intento_por(conn, id, contacto, "email", clave)
}

fn insertar_intento_por(
    conn: &Connection,
    id: &str,
    contacto: &str,
    canal: &str,
    clave: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO message_attempt (id, campaign_id, channel, contact_id,
                                      contact_address, idempotency_key,
                                      created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            CAMPANA,
            canal,
            contacto,
            direccion_de(contacto, canal),
            clave,
            AHORA
        ],
    )
}

// ─── Idempotencia ───────────────────────────────────────────────────────────

/// §55 y ADR-0004: el duplicado debe ser imposible **a nivel de esquema**, no
/// una esperanza depositada en el código. Ningún fallo de lógica, ninguna
/// condición de carrera y ningún doble clic pueden crear dos intentos para el
/// mismo par campaña-contacto.
#[test]
fn no_puede_haber_dos_intentos_para_el_mismo_contacto_y_campana() {
    let (_d, conn) = base();

    insertar_intento(&conn, "i1", CONTACTO_A, "k1").expect("el primero entra");

    let segundo = insertar_intento(&conn, "i2", CONTACTO_A, "k2");
    assert!(
        segundo.is_err(),
        "el esquema permitió un intento duplicado: la garantía del §55 está rota"
    );
}

#[test]
fn contactos_distintos_de_la_misma_campana_si_conviven() {
    let (_d, conn) = base();
    insertar_intento(&conn, "i1", CONTACTO_A, "k1").expect("contacto A");
    insertar_intento(&conn, "i2", CONTACTO_B, "k2").expect("contacto B");
}

#[test]
fn la_clave_de_idempotencia_es_unica() {
    let (_d, conn) = base();
    insertar_intento(&conn, "i1", CONTACTO_A, "misma-clave").expect("el primero entra");
    assert!(
        insertar_intento(&conn, "i2", CONTACTO_B, "misma-clave").is_err(),
        "dos intentos comparten idempotency_key: el Message-Id dejaría de ser único"
    );
}

#[test]
fn el_estado_del_intento_solo_admite_los_valores_de_la_maquina_de_estados() {
    let (_d, conn) = base();
    insertar_intento(&conn, "i1", CONTACTO_A, "k1").expect("inserta");

    assert!(
        conn.execute(
            "UPDATE message_attempt SET state = 'inventado' WHERE id = 'i1'",
            [],
        )
        .is_err(),
        "el CHECK de state no está protegiendo la máquina de estados"
    );

    conn.execute(
        "UPDATE message_attempt SET state = 'presumed_sent' WHERE id = 'i1'",
        [],
    )
    .expect("presumed_sent debe ser un estado válido");
}

/// Hallazgo F1 de la revisión de la Fase 1.
///
/// Borrar un contacto **no puede** borrar la prueba de que se le envió un
/// correo. Si lo hiciera, se perdería a la vez la auditoría y la garantía de
/// idempotencia: sin la fila, reimportar al contacto dejaría enviarle otra vez.
#[test]
fn borrar_el_contacto_no_borra_el_registro_de_envio() {
    let (_d, conn) = base();
    insertar_intento(&conn, "i1", CONTACTO_A, "k1").expect("inserta");

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("borra el contacto");

    let (quedan, con_identidad, correo): (i64, i64, String) = conn
        .query_row(
            "SELECT count(*), count(contact_id), max(contact_address) FROM message_attempt",
            [],
            |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
        )
        .expect("consulta");

    assert_eq!(
        quedan, 1,
        "el registro de envío desapareció al borrar el contacto: se pierden la \
         auditoría y la protección contra duplicados"
    );
    assert_eq!(
        con_identidad, 0,
        "el contact_id debería anularse: se conserva el hecho, no la identidad"
    );
    assert_eq!(
        correo, "ana@empresa.com",
        "la dirección debe conservarse: es la clave de idempotencia"
    );
}

/// El corolario que da sentido al cambio: reimportar a alguien ya contactado
/// **no** abre la puerta a un segundo envío dentro de la misma campaña.
#[test]
fn reimportar_un_contacto_no_permite_reenviarle() {
    let (_d, conn) = base();
    insertar_intento(&conn, "i1", CONTACTO_A, "k1").expect("primer envío");

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("borra");
    conn.execute(
        "INSERT INTO contact (id, company_id, created_at, updated_at)
         VALUES ('c-nuevo', ?1, ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("reimporta con un id nuevo");
    insertar_canal(&conn, "ch-nuevo", "c-nuevo", "email", "ana@empresa.com");

    let segundo = conn.execute(
        "INSERT INTO message_attempt (id, campaign_id, channel, contact_id,
                                      contact_address, idempotency_key,
                                      created_at, updated_at)
         VALUES ('i2', ?1, 'email', 'c-nuevo', 'ana@empresa.com', 'k2', ?2, ?2)",
        params![CAMPANA, AHORA],
    );

    assert!(
        segundo.is_err(),
        "se permitió un segundo envío a la misma dirección en la misma campaña \
         tras borrar y reimportar el contacto"
    );
}

/// La audiencia congelada tiene que seguir congelada.
#[test]
fn borrar_el_contacto_no_encoge_la_audiencia() {
    let (_d, conn) = base();
    conn.execute(
        "INSERT INTO campaign_audience (campaign_id, channel, contact_address,
                                        contact_id, added_at)
         VALUES (?1, 'email', 'ana@empresa.com', ?2, ?3)",
        params![CAMPANA, CONTACTO_A, AHORA],
    )
    .expect("congela la audiencia");

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("borra el contacto");

    let quedan: i64 = conn
        .query_row("SELECT count(*) FROM campaign_audience", [], |f| f.get(0))
        .expect("cuenta");
    assert_eq!(
        quedan, 1,
        "la audiencia encogió sola: si la instantánea cambia después, no es una \
         instantánea y «¿a quién le llegó esto?» se queda sin respuesta"
    );
}

// ─── Supresión ──────────────────────────────────────────────────────────────

/// §39: la clave de supresión es `email_normalized`, **no `contact_id`**. Si
/// alguien borra un contacto y lo vuelve a importar, la supresión sigue
/// aplicando: es sobre la dirección, no sobre el registro.
#[test]
fn la_supresion_es_unica_por_direccion() {
    let (_d, conn) = base();

    let insertar = |id: &str| {
        conn.execute(
            "INSERT INTO suppression_entry (id, company_id, channel,
                                            address_normalized,
                                            reason, origin, created_at)
             VALUES (?1, ?2, 'email', 'ana@empresa.com', 'unsubscribe', 'user', ?3)",
            params![id, EMPRESA, AHORA],
        )
    };

    insertar("s1").expect("la primera entra");
    assert!(
        insertar("s2").is_err(),
        "se permitió suprimir dos veces la misma dirección"
    );
}

#[test]
fn borrar_el_contacto_no_borra_su_supresion() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                        reason, origin, created_at)
         VALUES ('s1', ?1, 'email', 'ana@empresa.com', 'unsubscribe', 'user', ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("suprime");

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("borra el contacto");

    let quedan: i64 = conn
        .query_row("SELECT count(*) FROM suppression_entry", [], |f| f.get(0))
        .expect("cuenta");
    assert_eq!(
        quedan, 1,
        "la supresión desapareció al borrar el contacto: reimportarlo lo haría \
         recibir correos otra vez"
    );
}

// ─── Deduplicación de contactos ─────────────────────────────────────────────

/// §36: la deduplicación por dirección normalizada es del esquema. Desde la V3
/// vive en `contact_channel`, y es **por canal**.
#[test]
fn no_puede_haber_dos_contactos_con_la_misma_direccion_en_un_canal() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO contact (id, company_id, created_at, updated_at)
         VALUES ('c3', ?1, ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("inserta contacto");

    let r = conn.execute(
        "INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                      value_raw, value_normalized,
                                      created_at, updated_at)
         VALUES ('ch-x', ?1, 'c3', 'email', 'ANA@empresa.com', 'ana@empresa.com', ?2, ?2)",
        params![EMPRESA, AHORA],
    );
    assert!(r.is_err(), "se permitió duplicar una dirección de correo");
}

/// El índice único es parcial (`WHERE deleted_at IS NULL`): un contacto
/// archivado no debe impedir volver a dar de alta la misma dirección.
#[test]
fn un_canal_borrado_libera_su_direccion() {
    let (_d, conn) = base();

    conn.execute(
        "UPDATE contact_channel SET deleted_at = ?1 WHERE contact_id = ?2",
        params![AHORA, CONTACTO_A],
    )
    .expect("marca el canal como borrado");

    conn.execute(
        "INSERT INTO contact (id, company_id, created_at, updated_at)
         VALUES ('c3', ?1, ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("inserta contacto");

    conn.execute(
        "INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                      value_raw, value_normalized,
                                      created_at, updated_at)
         VALUES ('ch-x', ?1, 'c3', 'email', 'ana@empresa.com', 'ana@empresa.com', ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("debería poder reinsertarse tras el borrado lógico");
}

// ─── Bitácora de auditoría ──────────────────────────────────────────────────

/// §91: `audit_log` es append-only. Los triggers lo imponen a nivel de motor,
/// no de convención — una convención se rompe en un commit de viernes.
#[test]
fn la_bitacora_no_admite_update_ni_delete() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO audit_log (id, company_id, actor, action, created_at)
         VALUES ('a1', ?1, 'usuario', 'campaign.activated', ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("inserta en la bitácora");

    assert!(
        conn.execute("UPDATE audit_log SET action = 'otra' WHERE id = 'a1'", [])
            .is_err(),
        "se pudo modificar la bitácora de auditoría"
    );
    assert!(
        conn.execute("DELETE FROM audit_log WHERE id = 'a1'", [])
            .is_err(),
        "se pudo borrar de la bitácora de auditoría"
    );

    let quedan: i64 = conn
        .query_row("SELECT count(*) FROM audit_log", [], |f| f.get(0))
        .expect("cuenta");
    assert_eq!(quedan, 1);
}

// ─── Integridad referencial ─────────────────────────────────────────────────

#[test]
fn las_claves_foraneas_se_hacen_cumplir() {
    let (_d, conn) = base();

    let r = insertar_intento(&conn, "i1", "contacto-que-no-existe", "k1");
    assert!(
        r.is_err(),
        "se insertó un intento apuntando a un contacto inexistente: \
         PRAGMA foreign_keys no está activo"
    );
}

/// Una cuenta remitente con campañas asociadas no se borra en silencio: el
/// `ON DELETE RESTRICT` obliga a que el usuario resuelva qué pasa con ellas.
#[test]
fn no_se_puede_borrar_una_cuenta_remitente_en_uso() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO email_account (id, company_id, display_name, email_address,
                                    provider_kind, credential_ref, daily_limit,
                                    hourly_limit, created_at, updated_at)
         VALUES ('ea1', ?1, 'Ventas', 'ventas@empresa.com', 'smtp',
                 'llavero://ea1', 50, 10, ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("inserta cuenta");

    // Desde la V4 el remitente vive en la ETAPA, no en la campaña: es lo que
    // permite que una campaña tenga una etapa de correo y otra de WhatsApp.
    conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     email_account_id, created_at, updated_at)
         VALUES ('st1', ?1, 1, 'email', 'ea1', ?2, ?2)",
        params![CAMPANA, AHORA],
    )
    .expect("asigna la cuenta a la etapa");

    assert!(
        conn.execute("DELETE FROM email_account WHERE id = 'ea1'", [])
            .is_err(),
        "se borró una cuenta remitente que una campaña sigue usando"
    );
}

#[test]
fn la_ventana_de_ejecucion_rechaza_horas_incoherentes() {
    let (_d, conn) = base();

    let insertar = |id: &str, inicio: i64, fin: i64| {
        conn.execute(
            "INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES (?1, ?2, 'Laboral', 31, ?3, ?4, 'America/Mexico_City', ?5)",
            params![id, EMPRESA, inicio, fin, AHORA],
        )
    };

    insertar("w1", 9, 18).expect("9 a 18 es válido");
    assert!(
        insertar("w2", 18, 9).is_err(),
        "aceptó una ventana que termina antes de empezar"
    );
    assert!(
        insertar("w3", 9, 30).is_err(),
        "aceptó una hora fuera de rango"
    );
}

// ─── L-2 · Un contacto tiene canales ────────────────────────────────────────

/// Lo que Dirección pidió, escrito como prueba: primero el correo y después el
/// WhatsApp, dentro de la misma campaña.
///
/// !! **Este test pasaría también con el esquema anterior**, y conviene que
/// quede dicho en vez de dejarlo aparentando más de lo que comprueba. La
/// unicidad vieja era (campaña, dirección), y el correo y el móvil son
/// direcciones distintas, así que no chocaban. Lo que de verdad impedía esto
/// antes de la V3 era que **un contacto no tenía dónde guardar un móvil**: la
/// dirección eran dos columnas de `contact` y sólo cabía una.
///
/// Se conserva porque documenta el comportamiento del producto. Lo que ejerce
/// el canal en la clave de unicidad es el test de abajo.
#[test]
fn una_campana_puede_escribir_al_mismo_contacto_por_los_dos_canales() {
    let (_d, conn) = base();

    insertar_intento_por(&conn, "i1", CONTACTO_A, "email", "k1").expect("primero el correo");
    insertar_intento_por(&conn, "i2", CONTACTO_A, "whatsapp", "k2")
        .expect("y después el WhatsApp, que es justo lo que L-2 hace posible");

    let cuantos: i64 = conn
        .query_row(
            "SELECT count(*) FROM message_attempt WHERE contact_id = ?1",
            params![CONTACTO_A],
            |f| f.get(0),
        )
        .expect("cuenta");
    assert_eq!(cuantos, 2);
}

/// Y lo que NO cambia: dos veces por el mismo canal sigue siendo imposible.
/// Si esto se relajara, L-2 habría comprado el segundo canal al precio de la
/// garantía del §55, que es un precio que nadie aceptó pagar.
#[test]
fn dos_veces_por_el_mismo_canal_sigue_siendo_imposible() {
    let (_d, conn) = base();

    insertar_intento_por(&conn, "i1", CONTACTO_A, "whatsapp", "k1").expect("el primero entra");
    assert!(
        insertar_intento_por(&conn, "i2", CONTACTO_A, "whatsapp", "k2").is_err(),
        "se permitió un segundo WhatsApp al mismo contacto en la misma campaña"
    );
}

/// **El test que sí ejerce el canal dentro de la clave de unicidad.**
///
/// Dos intentos de la misma campaña con **la misma cadena** como dirección, uno
/// por cada canal. Con la clave vieja —(campaña, dirección)— el segundo se
/// rechaza; con la nueva entra, porque el canal forma parte de la identidad.
///
/// Comprobado quitando el canal del índice: este test falla y el de arriba no.
/// Ese contraste es exactamente por qué los dos existen.
#[test]
fn el_mismo_valor_en_canales_distintos_no_es_un_duplicado() {
    let (_d, conn) = base();

    let insertar = |id: &str, canal: &str, clave: &str| {
        conn.execute(
            "INSERT INTO message_attempt (id, campaign_id, channel, contact_id,
                                          contact_address, idempotency_key,
                                          created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'mismo-texto', ?5, ?6, ?6)",
            params![id, CAMPANA, canal, CONTACTO_A, clave, AHORA],
        )
    };

    insertar("i1", "email", "k1").expect("el primero entra");
    insertar("i2", "whatsapp", "k2").expect("la misma cadena por otro canal no es el mismo envío");
}

/// Y el canal también forma parte de la identidad de un canal de contacto.
#[test]
fn el_mismo_valor_en_canales_distintos_no_es_un_contacto_duplicado() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO contact (id, company_id, created_at, updated_at)
         VALUES ('c-raro', ?1, ?2, ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("inserta contacto");

    insertar_canal(&conn, "ch-1", "c-raro", "email", "mismo-texto");
    insertar_canal(&conn, "ch-2", "c-raro", "whatsapp", "mismo-texto");
}

/// La deduplicación es por canal, y dentro del canal es estricta.
#[test]
fn dos_contactos_no_comparten_el_mismo_movil() {
    let (_d, conn) = base();

    let r = conn.execute(
        "INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                      value_raw, value_normalized,
                                      created_at, updated_at)
         VALUES ('ch-choque', ?1, ?2, 'whatsapp', ?3, ?3, ?4, ?4)",
        params![EMPRESA, CONTACTO_B, MOVIL_A, AHORA],
    );
    assert!(
        r.is_err(),
        "dos contactos comparten el mismo móvil: la misma persona recibiría dos veces"
    );
}

/// Borrar un contacto se lleva sus canales —son suyos— pero **no** los intentos,
/// que son la prueba de lo que se envió. Si los canales sobrevivieran al
/// contacto, quedarían direcciones sin dueño que nadie volvería a mirar.
#[test]
fn borrar_el_contacto_borra_sus_canales_pero_no_los_intentos() {
    let (_d, conn) = base();
    insertar_intento_por(&conn, "i1", CONTACTO_A, "whatsapp", "k1").expect("inserta");

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("borra el contacto");

    let canales: i64 = conn
        .query_row(
            "SELECT count(*) FROM contact_channel WHERE contact_id = ?1",
            params![CONTACTO_A],
            |f| f.get(0),
        )
        .expect("cuenta");
    assert_eq!(canales, 0, "quedaron canales sin contacto");

    let (intentos, direccion): (i64, String) = conn
        .query_row(
            "SELECT count(*), max(contact_address) FROM message_attempt",
            [],
            |f| Ok((f.get(0)?, f.get(1)?)),
        )
        .expect("consulta");
    assert_eq!(intentos, 1, "se perdió la prueba del envío");
    assert_eq!(
        direccion, MOVIL_A,
        "la dirección debe conservarse: es la clave de idempotencia"
    );
}

// ─── L-4 · La supresión distingue canales ───────────────────────────────────

/// «No me escribas por WhatsApp» **no es** «no me escribas nunca». Antes de la
/// V3 el esquema no sabía distinguirlos, así que una baja de un canal habría
/// apagado los dos — o, peor, se habría guardado como si fuera del otro.
#[test]
fn suprimir_un_canal_no_suprime_el_otro() {
    let (_d, conn) = base();

    let suprimir = |id: &str, canal: &str, direccion: &str| {
        conn.execute(
            "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                            reason, origin, created_at)
             VALUES (?1, ?2, ?3, ?4, 'unsubscribe', 'user', ?5)",
            params![id, EMPRESA, canal, direccion, AHORA],
        )
    };

    suprimir("s1", "whatsapp", MOVIL_A).expect("se da de baja de WhatsApp");

    let mut consulta = conn
        .prepare("SELECT channel FROM suppression_entry ORDER BY channel")
        .expect("prepara");
    let por_canal: Vec<String> = consulta
        .query_map([], |f| f.get::<_, String>(0))
        .expect("consulta")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(
        por_canal,
        vec!["whatsapp".to_owned()],
        "la baja de un canal alcanzó a otro"
    );

    // Y el correo del mismo contacto se puede suprimir aparte, sin chocar.
    suprimir("s2", "email", "ana@empresa.com").expect("la del correo es otra entrada");
}

/// El alcance global existe, y se guarda como **una fila por dirección** unidas
/// por `request_id`. No como una fila atada a la persona: esa desaparecería al
/// ejercerse el derecho de cancelación, justo cuando más falta hace.
#[test]
fn una_baja_global_deja_una_fila_por_canal_unidas_por_su_peticion() {
    let (_d, conn) = base();

    for (id, canal, direccion) in [
        ("s1", "email", "ana@empresa.com"),
        ("s2", "whatsapp", MOVIL_A),
    ] {
        conn.execute(
            "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                            scope, request_id, reason, origin, created_at)
             VALUES (?1, ?2, ?3, ?4, 'global', 'pet-1', 'unsubscribe', 'user', ?5)",
            params![id, EMPRESA, canal, direccion, AHORA],
        )
        .expect("inserta");
    }

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("ejerce la cancelación y borra el contacto");

    let quedan: i64 = conn
        .query_row(
            "SELECT count(*) FROM suppression_entry WHERE request_id = 'pet-1'",
            [],
            |f| f.get(0),
        )
        .expect("cuenta");
    assert_eq!(
        quedan, 2,
        "la baja global se evaporó al borrar el contacto: reimportarlo lo haría \
         recibir por los dos canales otra vez"
    );
}

// ─── L-3 · El consentimiento es un registro, no una casilla ─────────────────

/// Retirar el consentimiento es una entrada NUEVA. Si se pudiera editar la
/// anterior, el registro dejaría de ser prueba de nada: cualquiera podría
/// reescribir a posteriori con qué base se le escribió a alguien.
#[test]
fn el_consentimiento_no_admite_update_ni_delete() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO consent_entry (id, company_id, channel, address_normalized,
                                    kind, basis, evidence, recorded_at)
         VALUES ('n1', ?1, 'whatsapp', ?2, 'granted', 'public_source',
                 'directorio público de la cámara', ?3)",
        params![EMPRESA, MOVIL_A, AHORA],
    )
    .expect("registra el consentimiento");

    assert!(
        conn.execute(
            "UPDATE consent_entry SET kind = 'withdrawn' WHERE id = 'n1'",
            [],
        )
        .is_err(),
        "se pudo editar un consentimiento ya registrado"
    );
    assert!(
        conn.execute("DELETE FROM consent_entry WHERE id = 'n1'", [])
            .is_err(),
        "se pudo borrar un consentimiento ya registrado"
    );

    // La retirada sí entra, como lo que es: otra entrada.
    conn.execute(
        "INSERT INTO consent_entry (id, company_id, channel, address_normalized,
                                    kind, basis, recorded_at)
         VALUES ('n2', ?1, 'whatsapp', ?2, 'withdrawn', 'verbal', ?3)",
        params![EMPRESA, MOVIL_A, AHORA],
    )
    .expect("la retirada es una entrada nueva");
}

/// La prueba sobrevive al contacto, igual que la supresión: si se fuera con él,
/// reimportarlo dejaría el envío sin nada que lo respalde.
///
/// **Y el contacto se tiene que poder borrar.** Este test encontró que no se
/// podía: el primer borrador de `consent_entry` llevaba un `contact_id` con
/// `ON DELETE SET NULL`, y `SET NULL` es un UPDATE que el disparador de
/// append-only aborta. Registrar la prueba de que se le podía escribir a
/// alguien impedía ejercer su derecho de cancelación. La columna se quitó: la
/// entrada es sobre una dirección, no sobre un registro.
#[test]
fn borrar_el_contacto_no_borra_la_prueba_del_consentimiento() {
    let (_d, conn) = base();

    conn.execute(
        "INSERT INTO consent_entry (id, company_id, channel, address_normalized,
                                    kind, basis, recorded_at)
         VALUES ('n1', ?1, 'email', 'ana@empresa.com', 'granted',
                 'import_affirmation', ?2)",
        params![EMPRESA, AHORA],
    )
    .expect("registra");

    conn.execute("DELETE FROM contact WHERE id = ?1", params![CONTACTO_A])
        .expect("el derecho de cancelación no puede quedar bloqueado por la prueba");

    let quedan: i64 = conn
        .query_row("SELECT count(*) FROM consent_entry", [], |f| f.get(0))
        .expect("consulta");
    assert_eq!(quedan, 1, "la prueba desapareció con el contacto");
}

/// El canal no admite cualquier cosa. Una fila con un canal inventado decidiría
/// a quién se le escribe y por dónde, y nadie la miraría hasta que fuera tarde.
#[test]
fn el_canal_solo_admite_los_dos_que_existen() {
    let (_d, conn) = base();

    for tabla_y_sql in [
        "INSERT INTO contact_channel (id, company_id, contact_id, channel, value_raw,
                                      value_normalized, created_at, updated_at)
         VALUES ('x', '01900000-0000-7000-8000-000000000001',
                 '01900000-0000-7000-8000-00000000000a', 'sms', 'a', 'a',
                 '2026-09-11T00:00:00Z', '2026-09-11T00:00:00Z')",
        "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                        reason, origin, created_at)
         VALUES ('x', '01900000-0000-7000-8000-000000000001', 'sms', 'a',
                 'manual', 'user', '2026-09-11T00:00:00Z')",
    ] {
        assert!(
            conn.execute(tabla_y_sql, []).is_err(),
            "se admitió un canal que no existe"
        );
    }
}

// ─── L-1 · Una campaña tiene etapas ─────────────────────────────────────────

const CUENTA_CORREO: &str = "ea-ventas";
const CUENTA_WHATSAPP: &str = "wa-ventas";

/// Deja las dos cuentas remitentes listas: una de correo y un número.
fn con_remitentes(conn: &Connection) {
    conn.execute(
        "INSERT INTO email_account (id, company_id, display_name, email_address,
                                    provider_kind, credential_ref, daily_limit,
                                    hourly_limit, created_at, updated_at)
         VALUES (?1, ?2, 'Ventas', 'ventas@empresa.com', 'smtp',
                 'llavero://ea1', 50, 10, ?3, ?3)",
        params![CUENTA_CORREO, EMPRESA, AHORA],
    )
    .expect("inserta cuenta de correo");

    conn.execute(
        "INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                       waba_id, phone_number_id, credential_ref,
                                       created_at, updated_at)
         VALUES (?1, ?2, 'Ventas WA', '+528100000000', 'waba-1', 'pn-1',
                 'llavero://wa1', ?3, ?3)",
        params![CUENTA_WHATSAPP, EMPRESA, AHORA],
    )
    .expect("inserta número de WhatsApp");
}

fn insertar_etapa(
    conn: &Connection,
    id: &str,
    posicion: i64,
    canal: &str,
) -> rusqlite::Result<usize> {
    let (correo, whatsapp) = match canal {
        "email" => (Some(CUENTA_CORREO), None),
        _ => (None, Some(CUENTA_WHATSAPP)),
    };
    conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     email_account_id, whatsapp_account_id,
                                     condition, wait_hours, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            id,
            CAMPANA,
            posicion,
            canal,
            correo,
            whatsapp,
            if posicion == 1 {
                "always"
            } else {
                "previous_not_failed"
            },
            if posicion == 1 { 0 } else { 48 },
            AHORA
        ],
    )
}

/// **Lo que L-1 hace posible.** Correo primero, WhatsApp después, en la misma
/// campaña y sobre la misma tabla de contactos.
#[test]
fn una_campana_admite_una_etapa_de_correo_y_otra_de_whatsapp() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    insertar_etapa(&conn, "st1", 1, "email").expect("primera etapa: correo");
    insertar_etapa(&conn, "st2", 2, "whatsapp").expect("segunda etapa: WhatsApp");
}

/// Dos etapas del mismo canal no son una secuencia: son dos envíos iguales, y
/// la unicidad de los intentos rechazaría el segundo contacto a contacto. Sin
/// esta restricción la campaña se activaría y la segunda etapa no enviaría ni
/// un mensaje, **sin ningún error a la vista**.
#[test]
fn una_campana_no_admite_dos_etapas_del_mismo_canal() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    insertar_etapa(&conn, "st1", 1, "email").expect("la primera entra");
    assert!(
        insertar_etapa(&conn, "st2", 2, "email").is_err(),
        "se admitieron dos etapas de correo en la misma campaña"
    );
}

#[test]
fn las_posiciones_no_se_repiten_dentro_de_una_campana() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    insertar_etapa(&conn, "st1", 1, "email").expect("la primera entra");
    assert!(
        insertar_etapa(&conn, "st2", 1, "whatsapp").is_err(),
        "se admitieron dos etapas en la posición 1"
    );
}

/// El remitente tiene que corresponder al canal. Una etapa de correo colgada de
/// un número de WhatsApp **no falla al guardarse**: falla al enviar, con la
/// campaña ya activada. Por eso es un CHECK y no una regla del código.
#[test]
fn el_remitente_de_una_etapa_corresponde_a_su_canal() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    let cruzada = conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     whatsapp_account_id, created_at, updated_at)
         VALUES ('st-mal', ?1, 1, 'email', ?2, ?3, ?3)",
        params![CAMPANA, CUENTA_WHATSAPP, AHORA],
    );
    assert!(
        cruzada.is_err(),
        "una etapa de correo aceptó un número de WhatsApp"
    );

    let sin_remitente = conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     created_at, updated_at)
         VALUES ('st-vacia', ?1, 1, 'email', ?2, ?2)",
        params![CAMPANA, AHORA],
    );
    assert!(sin_remitente.is_err(), "se admitió una etapa sin remitente");

    let con_los_dos = conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     email_account_id, whatsapp_account_id,
                                     created_at, updated_at)
         VALUES ('st-dos', ?1, 1, 'email', ?2, ?3, ?4, ?4)",
        params![CAMPANA, CUENTA_CORREO, CUENTA_WHATSAPP, AHORA],
    );
    assert!(
        con_los_dos.is_err(),
        "se admitió una etapa con dos remitentes"
    );
}

/// La primera etapa no espera a nadie. Si esperara, la campaña quedaría
/// activada sin enviar nada y sin que la pantalla pudiera explicar por qué.
#[test]
fn la_primera_etapa_no_puede_esperar_ni_depender() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    let con_espera = conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     email_account_id, wait_hours,
                                     created_at, updated_at)
         VALUES ('st-mal', ?1, 1, 'email', ?2, 24, ?3, ?3)",
        params![CAMPANA, CUENTA_CORREO, AHORA],
    );
    assert!(con_espera.is_err(), "la primera etapa aceptó una espera");

    let con_condicion = conn.execute(
        "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                     email_account_id, condition,
                                     created_at, updated_at)
         VALUES ('st-mal2', ?1, 1, 'email', ?2, 'previous_not_failed', ?3, ?3)",
        params![CAMPANA, CUENTA_CORREO, AHORA],
    );
    assert!(
        con_condicion.is_err(),
        "la primera etapa aceptó una condición"
    );
}

/// El tope de espera del esquema es el mismo que `arles_core::MAX_ESPERA_HORAS`.
/// Si divergieran, el núcleo aprobaría una secuencia que la base rechaza.
#[test]
fn la_espera_entre_etapas_tiene_el_mismo_tope_que_el_nucleo() {
    let (_d, conn) = base();
    con_remitentes(&conn);
    insertar_etapa(&conn, "st1", 1, "email").expect("primera");

    let meter = |id: &str, horas: u32| {
        conn.execute(
            "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                         whatsapp_account_id, condition, wait_hours,
                                         created_at, updated_at)
             VALUES (?1, ?2, 2, 'whatsapp', ?3, 'always', ?4, ?5, ?5)",
            params![id, CAMPANA, CUENTA_WHATSAPP, horas, AHORA],
        )
    };

    meter("st-tope", arles_core::MAX_ESPERA_HORAS).expect("el tope exacto entra");
    conn.execute("DELETE FROM campaign_stage WHERE id = 'st-tope'", [])
        .expect("limpia");
    assert!(
        meter("st-pasada", arles_core::MAX_ESPERA_HORAS + 1).is_err(),
        "el esquema admitió una espera que el núcleo rechaza"
    );
}

/// **L-6 escrito como prueba.** Parar WhatsApp no para el correo. Con un solo
/// estado en la campaña, «detener» sólo podía significar detenerlo todo.
#[test]
fn parar_una_etapa_no_para_la_otra() {
    let (_d, conn) = base();
    con_remitentes(&conn);
    insertar_etapa(&conn, "st1", 1, "email").expect("correo");
    insertar_etapa(&conn, "st2", 2, "whatsapp").expect("whatsapp");

    conn.execute(
        "UPDATE campaign_stage SET status = 'running' WHERE campaign_id = ?1",
        params![CAMPANA],
    )
    .expect("las dos en marcha");

    conn.execute(
        "UPDATE campaign_stage SET status = 'stopped'
          WHERE campaign_id = ?1 AND channel = 'whatsapp'",
        params![CAMPANA],
    )
    .expect("se detiene WhatsApp");

    let correo: String = conn
        .query_row(
            "SELECT status FROM campaign_stage
              WHERE campaign_id = ?1 AND channel = 'email'",
            params![CAMPANA],
            |f| f.get(0),
        )
        .expect("consulta");
    assert_eq!(
        correo, "running",
        "detener WhatsApp detuvo también el correo: L-6 no se sostiene"
    );
}

/// Borrar una etapa no puede borrar la prueba de lo que se envió desde ella.
/// Misma razón que con el contacto en la V1.
#[test]
fn borrar_una_etapa_no_borra_sus_intentos() {
    let (_d, conn) = base();
    con_remitentes(&conn);
    insertar_etapa(&conn, "st1", 1, "email").expect("etapa");

    conn.execute(
        "INSERT INTO message_attempt (id, campaign_id, stage_id, channel, contact_id,
                                      contact_address, idempotency_key,
                                      created_at, updated_at)
         VALUES ('i1', ?1, 'st1', 'email', ?2, 'ana@empresa.com', 'k1', ?3, ?3)",
        params![CAMPANA, CONTACTO_A, AHORA],
    )
    .expect("inserta intento");

    conn.execute("DELETE FROM campaign_stage WHERE id = 'st1'", [])
        .expect("borra la etapa");

    let (quedan, con_etapa): (i64, i64) = conn
        .query_row(
            "SELECT count(*), count(stage_id) FROM message_attempt",
            [],
            |f| Ok((f.get(0)?, f.get(1)?)),
        )
        .expect("consulta");
    assert_eq!(
        quedan, 1,
        "se perdió la prueba del envío al borrar la etapa"
    );
    assert_eq!(con_etapa, 0, "el stage_id debería anularse");
}

/// Una cuenta remitente con una etapa que la usa no se borra en silencio.
#[test]
fn no_se_puede_borrar_un_numero_de_whatsapp_en_uso() {
    let (_d, conn) = base();
    con_remitentes(&conn);
    insertar_etapa(&conn, "st1", 1, "whatsapp").expect("etapa de WhatsApp");

    assert!(
        conn.execute(
            "DELETE FROM whatsapp_account WHERE id = ?1",
            params![CUENTA_WHATSAPP]
        )
        .is_err(),
        "se borró un número que una campaña sigue usando"
    );
}

// ─── L-11 · Coexistencia ────────────────────────────────────────────────────

/// La confirmación de Coexistencia se guarda como **fecha**, no como casilla.
/// Lo que hay que poder enseñar es cuándo se confirmó y quién, no que alguien
/// marcó algo alguna vez.
#[test]
fn la_coexistencia_se_registra_con_fecha_y_autor() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    let sin_confirmar: Option<String> = conn
        .query_row(
            "SELECT coexistencia_confirmada_at FROM whatsapp_account WHERE id = ?1",
            params![CUENTA_WHATSAPP],
            |f| f.get(0),
        )
        .expect("consulta");
    assert!(
        sin_confirmar.is_none(),
        "un número recién conectado no puede nacer con la Coexistencia confirmada"
    );

    conn.execute(
        "UPDATE whatsapp_account
            SET coexistencia_confirmada_at = ?1, coexistencia_confirmada_by = 'direccion'
          WHERE id = ?2",
        params![AHORA, CUENTA_WHATSAPP],
    )
    .expect("confirma");

    let (cuando, quien): (String, String) = conn
        .query_row(
            "SELECT coexistencia_confirmada_at, coexistencia_confirmada_by
               FROM whatsapp_account WHERE id = ?1",
            params![CUENTA_WHATSAPP],
            |f| Ok((f.get(0)?, f.get(1)?)),
        )
        .expect("consulta");
    assert_eq!(cuando, AHORA);
    assert_eq!(quien, "direccion");
}

/// La calificación que da Meta se guarda con su historial. Meta da el
/// resultado, no los ingredientes: no publica cuántos bloquearon ni cuántos
/// reportaron. La línea de tiempo es lo que contesta «¿qué envío quemó el
/// número?», que un contador de bloqueos no contestaría aunque existiera.
#[test]
fn la_calificacion_de_meta_guarda_su_historial() {
    let (_d, conn) = base();
    con_remitentes(&conn);

    for (id, calidad, cuando) in [
        ("q1", "green", "2026-09-01T00:00:00Z"),
        ("q2", "yellow", "2026-09-15T00:00:00Z"),
        ("q3", "red", "2026-09-17T00:00:00Z"),
    ] {
        conn.execute(
            "INSERT INTO whatsapp_quality_event (id, whatsapp_account_id, calidad,
                                                 ocurrido_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, CUENTA_WHATSAPP, calidad, cuando],
        )
        .expect("inserta evento");
    }

    let cuantos: i64 = conn
        .query_row("SELECT count(*) FROM whatsapp_quality_event", [], |f| {
            f.get(0)
        })
        .expect("cuenta");
    assert_eq!(
        cuantos, 3,
        "el historial no puede quedarse sólo con el último"
    );

    assert!(
        conn.execute(
            "INSERT INTO whatsapp_quality_event (id, whatsapp_account_id, calidad,
                                                 ocurrido_at, created_at)
             VALUES ('q4', ?1, 'morado', ?2, ?2)",
            params![CUENTA_WHATSAPP, AHORA],
        )
        .is_err(),
        "se admitió una calificación que Meta no da"
    );
}

// ─── V5 · el origen declarado de una importación ────────────────────────────

/// Inserta un lote de importación con el origen que se le diga.
fn insertar_lote(conn: &Connection, id: &str, origen: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO import_batch
             (id, company_id, original_filename, stored_filename, file_hash,
              column_mapping, consent_affirmation, origin, created_at)
         VALUES (?1, ?2, 'contactos.xlsx', ?1, 'sha256:x', '[]',
                 'Declaro que esta lista tiene origen lícito.', ?3, ?4)",
        params![id, EMPRESA, origen, AHORA],
    )
}

/// **La lista de orígenes es cerrada, y el esquema lo impone.**
///
/// Si no lo hiciera, un origen escrito a mano —«varios», «de siempre», una
/// cadena vacía— entraría sin más, y entonces el campo deja de servir para lo
/// único que existe: poder analizar de dónde salen las listas cuando llegue una
/// reclamación.
#[test]
fn el_origen_de_una_importacion_sale_de_la_lista_cerrada() {
    let (_d, conn) = base();

    for (i, bueno) in [
        "formulario_propio",
        "clientes_existentes",
        "evento_o_feria",
        "directorio_publico",
        "otro",
    ]
    .iter()
    .enumerate()
    {
        let id = format!("01900000-0000-7000-8000-00000000b{i:03}");
        insertar_lote(&conn, &id, bueno)
            .unwrap_or_else(|e| panic!("«{bueno}» debería admitirse: {e}"));
    }

    for malo in ["comprada", "", "FORMULARIO_PROPIO", "varios"] {
        let id = "01900000-0000-7000-8000-00000000bfff";
        assert!(
            insertar_lote(&conn, id, malo).is_err(),
            "«{malo}» entró como origen: la lista no es cerrada de verdad"
        );
    }
}

/// **El rastro de una importación no se borra porque estorbe.**
///
/// Sin esta restricción —con CASCADE— borrar el registro de una importación se
/// llevaría por delante los contactos que trajo, que son justo la prueba de lo
/// que se afirmó al importarlos. Con SET NULL sobrevivirían huérfanos, sin poder
/// responder a «¿de dónde salió esta persona?».
#[test]
fn no_se_puede_borrar_un_lote_que_todavia_tiene_contactos() {
    let (_d, conn) = base();
    let lote = "01900000-0000-7000-8000-00000000c001";
    insertar_lote(&conn, lote, "formulario_propio").expect("inserta el lote");

    conn.execute(
        "UPDATE contact SET import_batch_id = ?1 WHERE id = ?2",
        params![lote, CONTACTO_A],
    )
    .expect("enlaza el contacto con su importación");

    assert!(
        conn.execute("DELETE FROM import_batch WHERE id = ?1", params![lote])
            .is_err(),
        "se borró un lote que todavía tenía contactos: el rastro de la \
         importación se puede perder"
    );

    // Y el contacto sigue ahí, con su origen.
    let sigue: i64 = conn
        .query_row(
            "SELECT count(*) FROM contact WHERE id = ?1 AND import_batch_id = ?2",
            params![CONTACTO_A, lote],
            |f| f.get(0),
        )
        .expect("cuenta");
    assert_eq!(sigue, 1);
}

/// Un contacto dado de alta a mano **no** tiene lote, y eso es correcto: no
/// vino de ninguna importación. Si la columna fuera obligatoria, el alta a mano
/// tendría que inventarse un lote falso.
#[test]
fn un_contacto_dado_de_alta_a_mano_no_tiene_lote() {
    let (_d, conn) = base();
    let sin_lote: i64 = conn
        .query_row(
            "SELECT count(*) FROM contact WHERE import_batch_id IS NULL",
            [],
            |f| f.get(0),
        )
        .expect("cuenta");
    assert!(
        sin_lote >= 2,
        "los contactos de prueba deberían no tener lote"
    );
}

/// El hash admite NULL para las filas anteriores a la columna. Poner uno
/// calculado ahora afirmaría que ese texto es el que se aceptó entonces, y no
/// se sabe. Un NULL honesto dice «no se sabe», que es la verdad.
#[test]
fn el_hash_del_consentimiento_admite_no_saberse() {
    let (_d, conn) = base();
    let lote = "01900000-0000-7000-8000-00000000c002";
    insertar_lote(&conn, lote, "otro").expect("inserta");

    let hash: Option<String> = conn
        .query_row(
            "SELECT consent_hash FROM import_batch WHERE id = ?1",
            params![lote],
            |f| f.get(0),
        )
        .expect("lee");
    assert_eq!(hash, None);
}

/// **El enum del núcleo y el CHECK de la migración son la misma lista.**
///
/// Si divergieran, el fallo sería silencioso de la peor manera: el desplegable
/// ofrecería un origen que la base rechaza —y la importación entera se caería
/// al guardar, después de que el usuario haya revisado los choques—, o la base
/// admitiría uno que el núcleo no sabe leer de vuelta.
///
/// Se comprueba **insertando cada valor del enum**, no comparando dos listas de
/// texto: comparar listas prueba que dos constantes coinciden; insertar prueba
/// que la base los acepta.
#[test]
fn los_origenes_del_nucleo_son_los_que_la_base_admite() {
    let (_d, conn) = base();

    for (i, origen) in arles_core::OrigenDeLaLista::TODOS.iter().enumerate() {
        let id = format!("01900000-0000-7000-8000-00000000d{i:03}");
        insertar_lote(&conn, &id, origen.como_texto()).unwrap_or_else(|e| {
            panic!(
                "el núcleo ofrece «{}» pero la base lo rechaza: {e}",
                origen.como_texto()
            )
        });
    }

    // Y al revés: todo lo que la base guarda, el núcleo lo sabe leer.
    let mut consulta = conn
        .prepare("SELECT origin FROM import_batch")
        .expect("prepara");
    let guardados: Vec<String> = consulta
        .query_map([], |f| f.get::<_, String>(0))
        .expect("consulta")
        .collect::<Result<_, _>>()
        .expect("filas");

    for g in &guardados {
        assert!(
            arles_core::OrigenDeLaLista::desde_texto(g).is_ok(),
            "la base guardó «{g}» y el núcleo no sabe leerlo"
        );
    }
    assert_eq!(guardados.len(), arles_core::OrigenDeLaLista::TODOS.len());
}
