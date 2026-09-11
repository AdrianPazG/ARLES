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

    for (id, correo) in [
        (CONTACTO_A, "ana@empresa.com"),
        (CONTACTO_B, "beto@empresa.com"),
    ] {
        conn.execute(
            "INSERT INTO contact (id, company_id, email_raw, email_normalized,
                                  created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3, ?4, ?4)",
            params![id, EMPRESA, correo, AHORA],
        )
        .expect("inserta contacto");
    }

    conn.execute(
        "INSERT INTO campaign (id, company_id, name, created_at, updated_at)
         VALUES (?1, ?2, 'Clientes Q1', ?3, ?3)",
        params![CAMPANA, EMPRESA, AHORA],
    )
    .expect("inserta campaña");

    (dir, conn)
}

fn insertar_intento(
    conn: &Connection,
    id: &str,
    contacto: &str,
    clave: &str,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO message_attempt (id, campaign_id, contact_id, idempotency_key,
                                      created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, CAMPANA, contacto, clave, AHORA],
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

// ─── Supresión ──────────────────────────────────────────────────────────────

/// §39: la clave de supresión es `email_normalized`, **no `contact_id`**. Si
/// alguien borra un contacto y lo vuelve a importar, la supresión sigue
/// aplicando: es sobre la dirección, no sobre el registro.
#[test]
fn la_supresion_es_unica_por_direccion() {
    let (_d, conn) = base();

    let insertar = |id: &str| {
        conn.execute(
            "INSERT INTO suppression_entry (id, company_id, email_normalized,
                                            reason, origin, created_at)
             VALUES (?1, ?2, 'ana@empresa.com', 'unsubscribe', 'user', ?3)",
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
        "INSERT INTO suppression_entry (id, company_id, email_normalized,
                                        reason, origin, created_at)
         VALUES ('s1', ?1, 'ana@empresa.com', 'unsubscribe', 'user', ?2)",
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

/// §36: la deduplicación por correo normalizado es del esquema.
#[test]
fn no_puede_haber_dos_contactos_con_el_mismo_correo_normalizado() {
    let (_d, conn) = base();

    let r = conn.execute(
        "INSERT INTO contact (id, company_id, email_raw, email_normalized,
                              created_at, updated_at)
         VALUES ('c3', ?1, 'ANA@empresa.com', 'ana@empresa.com', ?2, ?2)",
        params![EMPRESA, AHORA],
    );
    assert!(r.is_err(), "se permitió duplicar un contacto por correo");
}

/// El índice único es parcial (`WHERE deleted_at IS NULL`): un contacto
/// archivado no debe impedir volver a dar de alta la misma dirección.
#[test]
fn un_contacto_borrado_libera_su_direccion() {
    let (_d, conn) = base();

    conn.execute(
        "UPDATE contact SET deleted_at = ?1 WHERE id = ?2",
        params![AHORA, CONTACTO_A],
    )
    .expect("marca como borrado");

    conn.execute(
        "INSERT INTO contact (id, company_id, email_raw, email_normalized,
                              created_at, updated_at)
         VALUES ('c3', ?1, 'ana@empresa.com', 'ana@empresa.com', ?2, ?2)",
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

    conn.execute(
        "UPDATE campaign SET email_account_id = 'ea1' WHERE id = ?1",
        params![CAMPANA],
    )
    .expect("asigna la cuenta a la campaña");

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
