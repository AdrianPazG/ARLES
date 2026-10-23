//! Las consultas críticas usan su índice. **Vigilado, no supuesto.**
//!
//! Entrega 8.6.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO ES UNA PRUEBA Y NO UN COMENTARIO EN LA MIGRACIÓN
//!
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Un índice se rompe sin tocarlo. Basta con que alguien añada un `AND` a la
//! consulta, cambie el `ORDER BY` o quite un `WHERE deleted_at IS NULL`, y
//! SQLite deja de poder usarlo **sin que nada falle**: la consulta sigue
//! devolviendo lo mismo, sólo que leyendo medio millón de filas en vez de
//! cien. Todas las pruebas siguen en verde. Es exactamente el defecto que la
//! 8.5 encontró midiendo, y la única forma de que no vuelva es comprobar el
//! plan, no el resultado.
//!
//! `EXPLAIN QUERY PLAN` dice qué va a hacer SQLite. Si dice `SCAN` sobre una
//! tabla que crece, alguien va a esperar.

use arles_db::{ClaveMaestra, abrir};
use rusqlite::Connection;

/// Las consultas que un usuario espera, con el índice que tienen que usar.
///
/// El texto es **el de la consulta real**, copiado del repositorio. Si alguien
/// cambia la de producción y no ésta, la prueba deja de cubrir lo que dice —
/// así que el comentario de cada una nombra dónde vive.
const CRITICAS: &[(&str, &str, &str)] = &[
    (
        // contactos.rs · Db::listar_contactos — la pantalla más usada
        "la lista de contactos",
        "SELECT id, first_name FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
          ORDER BY created_at DESC, id DESC
          LIMIT 100 OFFSET 0",
        "idx_contact_lista",
    ),
    (
        // contactos.rs · el total que la pantalla enseña arriba
        "el recuento de contactos",
        "SELECT count(*) FROM contact WHERE company_id = ?1 AND deleted_at IS NULL",
        "idx_contact_lista",
    ),
    (
        // contactos.rs · Db::duenos_de_direcciones — se ejecuta una vez por
        // fila al importar: con diez mil filas, diez mil veces
        "la búsqueda por dirección",
        "SELECT ch.value_normalized FROM contact_channel ch
           JOIN contact c ON c.id = ch.contact_id
          WHERE ch.company_id = ?1 AND ch.deleted_at IS NULL
            AND c.deleted_at IS NULL AND ch.value_normalized IN (?2)",
        "idx_channel_direccion",
    ),
    (
        // cola.rs · Db::recuperar_al_arrancar — en CADA arranque
        "la recuperación de envíos sin confirmar",
        "UPDATE message_attempt SET state = 'presumed_sent' WHERE state = 'sending'",
        "idx_attempt_enviando",
    ),
    (
        // cola.rs · la segunda mitad de la misma función
        "la recuperación de trabajo tomado",
        "UPDATE message_attempt SET state = 'queued' WHERE state = 'claimed'",
        "idx_attempt_tomado",
    ),
    (
        // alta_de_campana.rs · se consulta antes de activar una campaña
        "la pregunta a la bitácora",
        "SELECT count(*) FROM audit_log
          WHERE entity_id = ?1 AND action = 'aceptar_riesgo_de_volumen'",
        "idx_audit_accion",
    ),
    (
        // respuestas.rs · el derecho de cancelación
        "olvidar las respuestas de alguien",
        "DELETE FROM whatsapp_reply WHERE company_id = ?1 AND from_normalized = ?2",
        "idx_whatsapp_reply_de",
    ),
    (
        // supresion.rs · EL CAMINO DE ENVÍO. Ya usaba su índice antes de la
        // V11, y está aquí para que siga usándolo: es la consulta que se hace
        // antes de cada correo, y la que protege a quien pidió no recibir.
        "la supresión, antes de cada envío",
        "SELECT 1 FROM suppression_entry
          WHERE company_id = ?1 AND channel = ?2 AND address_normalized = ?3 LIMIT 1",
        "idx_suppression_unique",
    ),
    (
        // cola.rs · el bucle del motor pidiendo trabajo
        "tomar trabajo de la cola",
        "SELECT id FROM message_attempt
          WHERE campaign_id = ?1 AND state = 'queued' LIMIT 50",
        "idx_attempt_campaign_state",
    ),
    (
        // alta_de_campana.rs · la audiencia de una campaña CON LISTA (D-15).
        //
        // ── POR QUÉ ÉSTA Y NO LA DE SIN LISTA ────────────────────────────
        //
        // Porque son dos consultas distintas y sólo una se puede exigir hoy.
        // La de sin lista empieza con `SCAN ch` —recorre `contact_channel`
        // entera— y es parte de los 73.5 s que la auditoría C midió al activar
        // con 451 196 contactos: un problema declarado, con su cifra, que se
        // arregla sacando la activación al hilo del motor (Fase 9). Meterla
        // aquí dejaría esta prueba en rojo permanente, que es como se apaga
        // una comprobación.
        //
        // La de CON lista sí se puede exigir, y es la que importa: es la que
        // Dirección va a usar, y la que hace que una campaña de trescientas
        // personas cueste lo que cuestan trescientas. Medido el 23/10/2026
        // antes de escribirla: la forma obvia —`FROM contact` con un `EXISTS`
        // sobre la lista— daba `SCAN ch`; entrando por `contact_list_member`
        // no queda ningún `SCAN`.
        "la audiencia de una campaña con lista",
        "SELECT ch.value_normalized, c.id
           FROM contact_list_member m
           JOIN contact c ON c.id = m.contact_id
           JOIN contact_channel ch ON ch.contact_id = c.id
          WHERE m.list_id = ?4 AND c.company_id = ?2 AND c.status = 'active'
            AND ch.channel = 'email' AND ch.status = 'active'
            AND NOT EXISTS (SELECT 1 FROM suppression_entry s
                             WHERE s.company_id = ?2 AND s.channel = 'email'
                               AND s.address_normalized = ch.value_normalized)
          GROUP BY ch.value_normalized",
        "sqlite_autoindex_contact_list_member_1",
    ),
    (
        // cola.rs · el descanso entre campañas (D-16). EL CAMINO DE ENVÍO:
        // esto se pregunta **antes de cada correo**, así que un `SCAN` aquí no
        // cuesta una pantalla lenta — cuesta los 73.5 s de la auditoría C
        // multiplicados por cada destinatario.
        //
        // Ninguno de los cuatro índices que había empieza por
        // `contact_address`: el único que la lleva es `idx_attempt_unique`,
        // que es `(campaign_id, channel, contact_address)`, y esta consulta es
        // justo la que **no** filtra por campaña.
        "el descanso entre campañas",
        "SELECT MAX(COALESCE(sent_at, updated_at))
           FROM message_attempt
          WHERE contact_address = ?1 AND channel = ?2
            AND campaign_id <> ?3
            AND state IN ('sent', 'presumed_sent')",
        "idx_attempt_por_direccion",
    ),
];

/// El plan que SQLite va a seguir para esta consulta.
///
/// Los parámetros van como nulos: el plan no depende del valor, sólo de la
/// forma de la consulta y de los índices que haya.
///
/// ── CUÁNTOS PARÁMETROS, Y POR QUÉ NO SE CUENTAN LAS INTERROGACIONES ─────
///
/// Porque SQLite no cuenta huecos: reserva **hasta el índice más alto**. Una
/// consulta que usa `?2` dos veces y `?4` una tiene tres interrogaciones y
/// necesita cuatro parámetros, y la versión anterior de esto fallaba con
/// `InvalidParameterCount(3, 4)`.
///
/// No se vio nunca porque las nueve consultas de la lista usaban `?1 ?2 ?3`
/// seguidos y sin repetir. La primera que no lo hizo —la audiencia con lista,
/// 23/10/2026— rompió el instrumento, no la consulta. Es la forma más barata
/// del defecto de siempre: la comprobación cubría la parte que nadie había
/// roto.
fn plan(conn: &Connection, consulta: &str) -> Vec<String> {
    let mut s = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {consulta}"))
        .expect("prepara");
    let huecos = consulta
        .match_indices('?')
        .filter_map(|(i, _)| {
            consulta[i + 1..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<usize>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    s.query_map(
        rusqlite::params_from_iter(std::iter::repeat_n(rusqlite::types::Null, huecos)),
        |f| f.get::<_, String>(3),
    )
    .expect("consulta")
    .collect::<Result<Vec<_>, _>>()
    .expect("filas")
}

fn base() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let conn = abrir(&dir.path().join("arles.db"), &clave).expect("abre");
    (dir, conn)
}

/// **Ninguna consulta crítica barre una tabla que crece.**
///
/// Verificada el 06/10/2026 quitando `idx_contact_lista` de la V11: falla
/// nombrando «la lista de contactos» y enseñando el `SCAN contact`.
#[test]
fn las_consultas_criticas_usan_su_indice() {
    let (_d, conn) = base();
    let mut fallos = Vec::new();

    for (nombre, consulta, indice) in CRITICAS {
        let p = plan(&conn, consulta);
        let texto = p.join(" | ");

        if texto.contains("SCAN ") && !texto.contains("SCAN sqlite_") {
            fallos.push(format!(
                "{nombre}: barre la tabla en vez de usar {indice}\n      {texto}"
            ));
        } else if !texto.contains(indice) {
            fallos.push(format!(
                "{nombre}: no usa {indice}, que es el que se creó para ella\n      {texto}"
            ));
        }
    }

    assert!(
        fallos.is_empty(),
        "consultas que harán esperar a alguien:\n  - {}",
        fallos.join("\n  - ")
    );
}

/// **La lista de contactos no ordena en memoria.**
///
/// Es la mitad del defecto que se midió, y se comprueba aparte porque un
/// índice puede acotar la búsqueda y aun así dejar la ordenación fuera: el
/// plan diría `SEARCH` —parecería bien— y SQLite seguiría metiendo 455 864
/// filas en un árbol temporal para sacar cien.
///
/// ── LO QUE SE APRENDIÓ AL VERIFICARLA (§2) ──
///
/// El primer intento fue invertir el índice a `created_at ASC`, dando por
/// hecho que un `ORDER BY ... DESC` necesita un índice descendente. **La
/// prueba siguió pasando, y tenía razón**: SQLite recorre un índice al revés
/// sin coste, así que ascendente y descendente sirven igual mientras las dos
/// columnas vayan en la misma dirección.
///
/// Lo que sí la rompe —y es lo que esta prueba vigila— es que `created_at`
/// desaparezca de la clave del índice. Verificada el 06/10/2026 dejándolo en
/// `contact(company_id)` a secas: vuelve el `USE TEMP B-TREE FOR ORDER BY`.
///
/// Y hay una tercera forma, más sutil, que también se midió: un índice
/// **mezclado** `(company_id, created_at ASC, id DESC)` deja un
/// `USE TEMP B-TREE FOR RIGHT PART OF ORDER BY`. No importa la dirección;
/// importa que las dos columnas vayan igual.
#[test]
fn la_lista_de_contactos_no_ordena_en_memoria() {
    let (_d, conn) = base();
    let p = plan(
        &conn,
        "SELECT id FROM contact
          WHERE company_id = ?1 AND deleted_at IS NULL
          ORDER BY created_at DESC, id DESC
          LIMIT 100",
    );
    let texto = p.join(" | ");
    assert!(
        !texto.contains("TEMP B-TREE"),
        "la lista ordena en memoria: para enseñar cien filas las lee todas\n      {texto}"
    );
}

/// Los índices de la V11 existen, y con el nombre que la migración les dio.
///
/// Es la red de abajo: si alguien renombra uno, las pruebas de arriba fallan
/// por «no usa el índice» y esta dice por qué.
#[test]
fn los_indices_de_la_v11_estan_puestos() {
    let (_d, conn) = base();
    let mut s = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index'")
        .expect("prepara");
    let puestos: Vec<String> = s
        .query_map([], |f| f.get(0))
        .expect("consulta")
        .collect::<Result<Vec<_>, _>>()
        .expect("índices");

    for esperado in [
        "idx_contact_lista",
        "idx_channel_direccion",
        "idx_attempt_enviando",
        "idx_attempt_tomado",
        "idx_audit_accion",
        "idx_whatsapp_reply_de",
    ] {
        assert!(
            puestos.iter().any(|n| n == esperado),
            "falta el índice {esperado} de la V11"
        );
    }
}
