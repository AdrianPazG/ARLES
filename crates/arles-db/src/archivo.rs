//! Archivar el historial de envíos a un disco externo, y sólo entonces
//! borrarlo de la base (D-8D, entrega 8.3 bis).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ ES ESTO Y QUÉ NO ES
//!
//! No es borrar: es **mover**. `message_attempt` crece sin techo por diseño —un
//! renglón por persona y envío, y no se borra jamás (D-7C)—, así que en una
//! instalación de dos años a 50 000 contactos al mes son del orden de un millón
//! de renglones. Dirección preguntó el 05/10/2026 si se podía sacar lo viejo a
//! una SSD y quitarlo de aquí. La respuesta fue sí, con cuatro condiciones.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS CUATRO CONDICIONES, Y DÓNDE VIVE CADA UNA
//!
//! 1. **Nunca automático.** No hay nada en este módulo que corra solo: no hay
//!    temporizador, ni llamada desde el motor, ni un «si la base pasa de X».
//!    Lo pulsa una persona. Un proceso que borra historial solo es un proceso
//!    que un día borra el que hacía falta.
//! 2. **El archivo primero, el borrado después**, y sólo si el archivo se
//!    escribió entero y se volvió a leer. Eso lo orquesta `arles-app`, que es
//!    quien tiene disco; aquí lo que hay es la mitad que **se niega**:
//!    [`Db::borrar_el_historial_archivado`] exige cuántas filas se archivaron y
//!    falla si la base ya no tiene exactamente ésas.
//! 3. **Las bajas y el consentimiento no entran nunca.** Vive en
//!    `EN_EL_ARCHIVO` del núcleo, con su comprobación propia en el validador.
//! 4. **Queda en `audit_log`**: el periodo, cuántos renglones y cuándo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE EL PREFLIGHT TIENE QUE DECIR, Y POR QUÉ NO ES OBVIO
//!
//! Borrar un año de envíos **no se lleva sólo esos envíos**.
//! `activity_event.message_attempt_id` tiene clave ajena a `message_attempt`
//! con `ON DELETE CASCADE`, así que los sucesos del registro que apuntaban a
//! esos intentos se van con ellos — y `activity_event` **no viaja en el
//! archivo**, porque ya se recorta cada dos meses en la propia base (7.4). Así
//! que eso sí es una pérdida, y hay que contarla antes de que nadie confirme.
//!
//! Es el fallo de la Fase 1 (F1) por tercera vez: la cascada protege de otra
//! cosa y no se toca; lo que no puede pasar es que nadie la cuente. Por eso la
//! cascada **se calcula del esquema** con `PRAGMA foreign_key_list`, y hay una
//! prueba que borra de verdad dentro de una transacción que se deshace para
//! comprobar que lo calculado es lo que SQLite hace. Un cálculo corto callaría
//! una pérdida; uno largo asustaría con una que no ocurre.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::Connection;

use crate::db::Db;
use crate::error::DbError;

/// La tabla que se archiva. Las demás del archivo viajan enteras.
const HISTORIAL: &str = "message_attempt";

/// La columna por la que se corta el historial.
///
/// No se escribe aquí: se lee del manifiesto, que es donde la 8.2b la metió
/// dentro de la variante. Si algún día el historial cambia de columna de fecha
/// —como `whatsapp_reply`, que usa `first_reply_at`—, esto la sigue.
fn columna_de_corte() -> Result<&'static str, DbError> {
    arles_core::respaldo::alcance_de(HISTORIAL)
        .and_then(arles_core::respaldo::Alcance::columna_de_corte)
        .ok_or(DbError::EsquemaNoSeDejaRespaldar {
            motivo: "el manifiesto no dice por qué columna se recorta el historial, así que no se puede saber qué archivar",
        })
}

/// Lo que se pierde de otra tabla porque cuelga del historial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrastrada {
    pub tabla: String,
    /// Cuántas filas de ella se van con el borrado. **Filas, no «puede que
    /// alguna»**: es el número que se le enseña a quien confirma.
    pub filas: u64,
}

/// Lo que pasaría al archivar, **antes de tocar nada**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueSeArchiva {
    /// Cuántos envíos se van al archivo y se borran de aquí.
    pub envios_que_se_archivan: u64,
    /// Cuántos se quedan en la base.
    pub envios_que_se_quedan: u64,
    /// El envío más viejo de los que se van, si hay alguno.
    ///
    /// Es el periodo **real**, no el pedido: alguien que pide archivar «lo de
    /// antes de 2026» con la base empezada en marzo tiene que ver «de marzo a
    /// diciembre de 2025» y no «todo lo anterior a 2026», que no dice nada.
    pub primero: Option<String>,
    /// El más reciente de los que se van.
    pub ultimo: Option<String>,
    /// Cuántas campañas viajan en el archivo (van enteras: ver `EN_EL_ARCHIVO`).
    pub campanas: u64,
    /// Lo que la cascada se lleva además, y que el archivo **no** tiene.
    pub arrastradas: Vec<Arrastrada>,
}

impl QueSeArchiva {
    /// ¿Hay algo que archivar?
    ///
    /// Existe para que la pantalla no tenga que comparar con cero: «no hay nada
    /// anterior a esa fecha» es una respuesta, y enseñar un botón que escribiría
    /// un archivo vacío no lo es.
    #[must_use]
    pub fn hay_algo(&self) -> bool {
        self.envios_que_se_archivan > 0
    }

    /// Cuántas filas se pierden por la cascada, en total.
    #[must_use]
    pub fn total_arrastrado(&self) -> u64 {
        self.arrastradas.iter().map(|a| a.filas).sum()
    }
}

/// Lo que de verdad pasó al borrar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoQueSeBorro {
    pub envios: u64,
    pub arrastradas: Vec<Arrastrada>,
}

impl Db {
    /// Qué se archivaría y qué se perdería, con el corte en esta fecha.
    ///
    /// `corte` es una fecha ISO-8601 **exclusiva**: entra lo anterior, no lo de
    /// ese día.
    ///
    /// # Errores
    ///
    /// - [`DbError::Sqlite`] si alguna consulta falla.
    /// - [`DbError::EsquemaNoSeDejaRespaldar`] si el manifiesto no dice por qué
    ///   columna se corta el historial.
    pub fn que_pasaria_al_archivar(&self, corte: &str) -> Result<QueSeArchiva, DbError> {
        let columna = columna_de_corte()?;
        self.con_dominio(|conn| {
            // Una transacción de lectura, igual que al exportar y al restaurar:
            // todas las cifras que se le enseñan a una persona tienen que ser
            // del mismo instante, o la suma no cuadra con los sumandos.
            let tx = conn.unchecked_transaction()?;

            let envios_que_se_archivan = cuantos(
                &tx,
                &format!("SELECT count(*) FROM \"{HISTORIAL}\" WHERE \"{columna}\" < ?1"),
                corte,
            )?;
            let envios_que_se_quedan = cuantos(
                &tx,
                &format!("SELECT count(*) FROM \"{HISTORIAL}\" WHERE \"{columna}\" >= ?1"),
                corte,
            )?;

            let (primero, ultimo): (Option<String>, Option<String>) = tx.query_row(
                &format!(
                    "SELECT min(\"{columna}\"), max(\"{columna}\") FROM \"{HISTORIAL}\" WHERE \"{columna}\" < ?1"
                ),
                [corte],
                |f| Ok((f.get(0)?, f.get(1)?)),
            )?;

            let campanas = cuantos_sin_parametro(&tx, "SELECT count(*) FROM campaign")?;

            let arrastradas = arrastradas_por_el_borrado(&tx, columna, corte)?;

            tx.rollback()?;
            Ok(QueSeArchiva {
                envios_que_se_archivan,
                envios_que_se_quedan,
                primero,
                ultimo,
                campanas,
                arrastradas,
            })
        })
    }

    /// Borra el historial que ya está archivado, y deja constancia.
    ///
    /// ── LO QUE ESTA FUNCIÓN SE NIEGA A HACER ──
    ///
    /// `archivadas` es cuántas filas escribió el archivo. Si la base ya no
    /// tiene exactamente ésas, **no se borra nada** y se devuelve
    /// [`DbError::ElHistorialCambio`].
    ///
    /// No es paranoia: el motor escribe en su propio hilo (§51), así que entre
    /// escribir el archivo y confirmar el borrado puede haber entrado un
    /// intento con fecha anterior al corte — un envío que se quedó en cola y se
    /// resolvió. Borrar «lo que haya ahora» se llevaría una fila que el archivo
    /// no tiene, y nadie se enteraría: el recuento final cuadraría consigo
    /// mismo.
    ///
    /// # Errores
    ///
    /// - [`DbError::ElHistorialCambio`] si el recuento no coincide.
    /// - [`DbError::Sqlite`] si algo falla al borrar. La transacción se deshace
    ///   entera: o se va el periodo completo, o no se va nada.
    pub fn borrar_el_historial_archivado(
        &self,
        empresa: &str,
        corte: &str,
        archivadas: u64,
    ) -> Result<LoQueSeBorro, DbError> {
        let columna = columna_de_corte()?;
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            // Encendidas para que la cascada ocurra de verdad. Apagadas, los
            // sucesos del registro quedarían apuntando a intentos que ya no
            // existen — una base con claves ajenas roídas y ninguna queja.
            tx.execute_batch("PRAGMA foreign_keys=ON")?;

            let ahora = cuantos(
                &tx,
                &format!("SELECT count(*) FROM \"{HISTORIAL}\" WHERE \"{columna}\" < ?1"),
                corte,
            )?;
            if ahora != archivadas {
                return Err(DbError::ElHistorialCambio { archivadas, ahora });
            }

            // Se cuenta **antes** de borrar: después, las filas que la cascada
            // se llevó ya no están y no hay forma de saber cuántas eran. Es lo
            // que permite escribirlo en la bitácora con una cifra y no con un
            // «algunas».
            let arrastradas = arrastradas_por_el_borrado(&tx, columna, corte)?;

            let borradas = tx.execute(
                &format!("DELETE FROM \"{HISTORIAL}\" WHERE \"{columna}\" < ?1"),
                [corte],
            )?;

            // ── LA BITÁCORA VA DENTRO DE LA TRANSACCIÓN ──
            //
            // Si se escribiera después y el proceso muriera en medio, habría un
            // borrado sin constancia: justo el caso que la cuarta condición de
            // D-8D existe para que no pase. Y si la bitácora falla, el borrado
            // se deshace — que es el lado correcto del que fallar.
            let resumen = format!(
                "hasta={corte}/envios={borradas}/arrastradas={}",
                arrastradas.iter().map(|a| a.filas).sum::<u64>()
            );
            crate::empresa::auditar(&tx, empresa, "archivar_historial", "historial", &resumen)?;

            tx.commit()?;
            Ok(LoQueSeBorro {
                envios: u64::try_from(borradas).unwrap_or(archivadas),
                arrastradas,
            })
        })
    }
}

fn cuantos(conn: &Connection, sql: &str, p: &str) -> Result<u64, DbError> {
    let n: i64 = conn.query_row(sql, [p], |f| f.get(0))?;
    Ok(n.max(0).unsigned_abs())
}

fn cuantos_sin_parametro(conn: &Connection, sql: &str) -> Result<u64, DbError> {
    let n: i64 = conn.query_row(sql, [], |f| f.get(0))?;
    Ok(n.max(0).unsigned_abs())
}

/// Qué tablas cuelgan de cuál con `ON DELETE CASCADE`, leído del esquema.
///
/// Devuelve `padre -> [(hija, columna_de_la_hija)]`. La columna hace falta para
/// poder **contar filas** y no sólo nombrar tablas: lo que se le enseña a quien
/// confirma es un número.
fn cascada_del_esquema(
    conn: &Connection,
) -> Result<BTreeMap<String, Vec<(String, String)>>, DbError> {
    let mut todas = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let nombres = todas
        .query_map([], |f| f.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut hijos: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for hija in &nombres {
        let mut fk = conn.prepare(&format!("PRAGMA foreign_key_list(\"{hija}\")"))?;
        let filas = fk
            .query_map([], |f| {
                Ok((
                    f.get::<_, String>("table")?,
                    f.get::<_, String>("from")?,
                    f.get::<_, String>("on_delete")?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (padre, columna, al_borrar) in filas {
            // Una tabla que se refiere a sí misma no se cuenta: el borrado de
            // sus propias filas ya está contado como el borrado principal.
            if al_borrar.eq_ignore_ascii_case("CASCADE") && padre != *hija {
                hijos
                    .entry(padre)
                    .or_default()
                    .push((hija.clone(), columna));
            }
        }
    }
    Ok(hijos)
}

/// Cuántas filas de cada otra tabla se va a llevar la cascada.
///
/// ── POR QUÉ SE ARMA UNA SUBCONSULTA ANIDADA Y NO SE CUENTA A OJO ──
///
/// Hoy la cascada del historial tiene **un solo nivel**: `activity_event` y
/// nada más. Contar ese caso a mano sería una línea. El problema es el día en
/// que otra tabla cuelgue de `activity_event`: la cuenta de un nivel seguiría
/// pasando en verde y callaría la pérdida del segundo. Así que se recorre lo
/// que el esquema diga, a la profundidad que diga.
fn arrastradas_por_el_borrado(
    conn: &Connection,
    columna: &str,
    corte: &str,
) -> Result<Vec<Arrastrada>, DbError> {
    let hijos = cascada_del_esquema(conn)?;

    // El conjunto de filas del historial que se va, como subconsulta. Cada
    // nivel la envuelve con la suya.
    let raiz = format!("SELECT id FROM \"{HISTORIAL}\" WHERE \"{columna}\" < ?1");

    let mut arrastradas: Vec<Arrastrada> = Vec::new();
    let mut vistas: BTreeSet<String> = BTreeSet::new();
    vistas.insert(HISTORIAL.to_owned());
    // (tabla, subconsulta que devuelve los ids de sus filas condenadas)
    let mut cola: Vec<(String, String)> = vec![(HISTORIAL.to_owned(), raiz)];

    while let Some((padre, ids_del_padre)) = cola.pop() {
        for (hija, columna_hija) in hijos.get(&padre).into_iter().flatten() {
            if !vistas.insert(hija.clone()) {
                continue;
            }
            let condenadas =
                format!("SELECT id FROM \"{hija}\" WHERE \"{columna_hija}\" IN ({ids_del_padre})");
            let n = cuantos(conn, &format!("SELECT count(*) FROM ({condenadas})"), corte)?;
            if n > 0 {
                arrastradas.push(Arrastrada {
                    tabla: hija.clone(),
                    filas: n,
                });
            }
            cola.push((hija.clone(), condenadas));
        }
    }

    // Orden estable por nombre: la pantalla enseña esta lista y un orden que
    // cambiara entre dos aperturas haría dudar de las cifras.
    arrastradas.sort_by(|a, b| a.tabla.cmp(&b.tabla));
    Ok(arrastradas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    /// Dos campañas, una baja, y seis envíos repartidos a los dos lados del
    /// corte del 2026-06-01, cada uno con un suceso del registro colgado.
    ///
    /// Los sucesos son lo que hace útil esta siembra: sin ellos la cascada no
    /// tendría nada que llevarse y las pruebas pasarían sin ejercitarla.
    fn sembrar(db: &Db) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
                   VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO suppression_entry (id, company_id, channel, address_normalized, scope, reason, origin, created_at)
                   VALUES ('S1', 'E1', 'email', 'quien@no.quiere', 'global', 'unsubscribe', 'user', '2026-02-01T00:00:00Z');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                   VALUES ('K1', 'E1', 'Vieja', 'completed', '2025-01-03T00:00:00Z', '2025-01-03T00:00:00Z');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                   VALUES ('K2', 'E1', 'Nueva', 'draft', '2026-09-03T00:00:00Z', '2026-09-03T00:00:00Z');",
            )
        })
        .expect("siembra");

        // Tres antes del corte y tres después.
        for (id, campana, fecha) in [
            ("M1", "K1", "2025-03-01T10:00:00Z"),
            ("M2", "K1", "2025-07-15T10:00:00Z"),
            ("M3", "K1", "2026-05-31T23:59:59Z"),
            ("M4", "K2", "2026-06-01T00:00:00Z"),
            ("M5", "K2", "2026-08-10T10:00:00Z"),
            ("M6", "K2", "2026-09-20T10:00:00Z"),
        ] {
            db.ejecutar_en_pruebas(|c| {
                c.execute(
                    "INSERT INTO message_attempt
                       (id, campaign_id, channel, contact_address, idempotency_key, state,
                        created_at, updated_at)
                     VALUES (?1, ?2, 'email', ?3, ?4, 'sent', ?5, ?5)",
                    rusqlite::params![
                        id,
                        campana,
                        format!("{id}@ejemplo.mx"),
                        format!("idem-{id}"),
                        fecha
                    ],
                )?;
                c.execute(
                    "INSERT INTO activity_event
                       (id, company_id, campaign_id, message_attempt_id, kind, is_outcome, occurred_at)
                     VALUES (?1, 'E1', ?2, ?3, 'envio.aceptado', 1, ?4)",
                    rusqlite::params![format!("A{id}"), campana, id, fecha],
                )?;
                Ok(())
            })
            .expect("envío");
        }
    }

    const CORTE: &str = "2026-06-01T00:00:00Z";

    fn cuantas(db: &Db, tabla: &str) -> u64 {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(&format!("SELECT count(*) FROM \"{tabla}\""), [], |f| {
                f.get::<_, i64>(0)
            })
        })
        .expect("cuenta")
        .max(0)
        .unsigned_abs()
    }

    /// **El preflight cuenta los dos lados, y el periodo real.**
    ///
    /// Verificada el 06/10/2026 cambiando el `<` del recuento por `<=`: pasa a
    /// decir 4 y falla. Es el día de más que no haría fallar nada.
    #[test]
    fn el_preflight_dice_que_se_va_y_que_se_queda() {
        let (_d, db) = base();
        sembrar(&db);

        let p = db.que_pasaria_al_archivar(CORTE).expect("preflight");
        assert_eq!(p.envios_que_se_archivan, 3, "{p:?}");
        assert_eq!(p.envios_que_se_quedan, 3, "{p:?}");
        assert_eq!(p.primero.as_deref(), Some("2025-03-01T10:00:00Z"));
        assert_eq!(p.ultimo.as_deref(), Some("2026-05-31T23:59:59Z"));
        assert!(p.hay_algo());
        // Las dos campañas viajan enteras: un intento sin su campaña es
        // ilegible, y el archivo tiene que servir dentro de tres años.
        assert_eq!(p.campanas, 2);
    }

    /// **El corte parte el historial sin solaparse y sin dejar hueco.**
    ///
    /// Lo que se archiva (`<`) y lo que se queda (`>=`) están escritos en dos
    /// funciones distintas —el exportador y el borrado—, así que cambiar uno a
    /// `<=` no hace fallar nada: duplica o pierde exactamente un día. Esta
    /// prueba pone un envío **justo en el instante del corte**, que es el único
    /// que distingue las dos fronteras.
    ///
    /// Verificada el 06/10/2026 poniendo `<=` en el recuento de lo que se
    /// archiva: la suma pasa a 7 sobre 6 filas y falla.
    #[test]
    fn el_corte_parte_el_historial_sin_solaparse_ni_dejar_hueco() {
        let (_d, db) = base();
        sembrar(&db);

        let total = cuantas(&db, "message_attempt");
        let p = db.que_pasaria_al_archivar(CORTE).expect("preflight");
        assert_eq!(
            p.envios_que_se_archivan + p.envios_que_se_quedan,
            total,
            "el corte deja un hueco o solapa: {} + {} != {total}",
            p.envios_que_se_archivan,
            p.envios_que_se_quedan
        );
        // Y el del instante exacto se queda: «hasta el 1 de junio» deja el 1.
        let p2 = db
            .que_pasaria_al_archivar("2026-06-01T00:00:00Z")
            .expect("preflight");
        assert_eq!(p2.envios_que_se_quedan, 3, "M4 es del instante del corte");
    }

    /// **Lo que el exportador se lleva es exactamente lo que el borrado quita.**
    ///
    /// Son dos caminos distintos —`QueSeLleva::ElHistorialAnteriorA` por un
    /// lado y el `DELETE` por otro— y si no coincidieran, el archivo tendría
    /// filas que siguen en la base, o faltarían filas que ya se fueron. Lo
    /// segundo es pérdida de datos sin un solo error.
    ///
    /// Verificada el 06/10/2026 cambiando el operador del archivado a `<=`:
    /// el inventario dice 4 y el borrado 3, y falla.
    #[test]
    fn el_inventario_del_archivo_cuadra_con_el_borrado() {
        let (_d, db) = base();
        sembrar(&db);

        let del_inventario = db
            .ejecutar_en_pruebas(|conn| {
                let inv = crate::respaldo::inventario_en(
                    conn,
                    crate::respaldo::QueSeLleva::ElHistorialAnteriorA(CORTE),
                )
                .expect("inventario");
                Ok(inv
                    .tablas
                    .iter()
                    .find(|t| t.tabla == "message_attempt")
                    .map_or(0, |t| t.filas))
            })
            .expect("inventario");

        let borrado = db
            .borrar_el_historial_archivado("E1", CORTE, del_inventario)
            .expect("borra");
        assert_eq!(borrado.envios, del_inventario);
        assert_eq!(cuantas(&db, "message_attempt"), 3);
    }

    /// **El archivo lleva las dos tablas de `EN_EL_ARCHIVO` y ninguna más.**
    ///
    /// Es lo que impide que una tabla se cuele en el archivado y se vaya de la
    /// base al borrar. La lista se compara contra la del núcleo, no contra una
    /// escrita aquí.
    ///
    /// Verificada el 06/10/2026 añadiendo `suppression_entry` a
    /// `EN_EL_ARCHIVO`: el inventario la trae y falla.
    #[test]
    fn el_archivo_lleva_lo_declarado_y_nada_mas() {
        let (_d, db) = base();
        sembrar(&db);

        let tablas = db
            .ejecutar_en_pruebas(|conn| {
                let inv = crate::respaldo::inventario_en(
                    conn,
                    crate::respaldo::QueSeLleva::ElHistorialAnteriorA(CORTE),
                )
                .expect("inventario");
                Ok(inv
                    .tablas
                    .iter()
                    .map(|t| t.tabla.clone())
                    .collect::<BTreeSet<_>>())
            })
            .expect("inventario");

        let declaradas: BTreeSet<String> = arles_core::respaldo::EN_EL_ARCHIVO
            .iter()
            .map(|t| (*t).to_owned())
            .collect();
        assert_eq!(tablas, declaradas, "el archivo no lleva lo declarado");
        assert!(
            !tablas.contains("suppression_entry") && !tablas.contains("consent_entry"),
            "tercera condición de D-8D: las bajas y el consentimiento no se archivan"
        );
    }

    /// **La cascada calculada es la que borra SQLite de verdad.**
    ///
    /// El cierre se lee de `PRAGMA foreign_key_list`; esta prueba no se fía y
    /// lo compara con lo que pasa: borra de verdad dentro de una transacción
    /// que se deshace, y cuenta qué desapareció de las demás tablas.
    ///
    /// Un cálculo corto callaría una pérdida; uno largo asustaría con una que
    /// no ocurre. Las dos son mentiras con cifras.
    ///
    /// Verificada el 06/10/2026 sustituyendo el recorrido del esquema por la
    /// lista `[("activity_event", "message_attempt_id")]`: sigue coincidiendo
    /// **hoy**, porque hoy la cascada es de un nivel. Así que además se
    /// comprueba que lo calculado trae `activity_event` con su cifra exacta, y
    /// no sólo que el conjunto de nombres cuadre: una lista a mano con el
    /// número mal pasaría la comparación de nombres.
    #[test]
    fn la_cascada_calculada_es_la_que_borra_sqlite() {
        let (_d, db) = base();
        sembrar(&db);

        let columna = columna_de_corte().expect("columna");

        db.ejecutar_en_pruebas(|conn| {
            let calculadas: BTreeMap<String, u64> =
                arrastradas_por_el_borrado(conn, columna, CORTE)
                    .expect("cascada")
                    .into_iter()
                    .map(|a| (a.tabla, a.filas))
                    .collect();

            // Todas las tablas del esquema menos el motor de migraciones.
            let mut s = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'refinery%'",
            )?;
            let nombres = s
                .query_map([], |f| f.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;

            let contar = |c: &Connection, t: &str| -> u64 {
                c.query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |f| {
                    f.get::<_, i64>(0)
                })
                .unwrap_or(0)
                .max(0)
                .unsigned_abs()
            };
            let antes: BTreeMap<String, u64> = nombres
                .iter()
                .map(|t| (t.clone(), contar(conn, t)))
                .collect();

            let tx = conn.unchecked_transaction()?;
            tx.execute_batch("PRAGMA foreign_keys=ON")?;
            tx.execute(
                &format!("DELETE FROM \"message_attempt\" WHERE \"{columna}\" < ?1"),
                [CORTE],
            )?;
            let medidas: BTreeMap<String, u64> = nombres
                .iter()
                .filter(|t| *t != "message_attempt")
                .filter_map(|t| {
                    let fue = antes.get(t).copied().unwrap_or(0);
                    let queda = contar(&tx, t);
                    (fue > queda).then(|| (t.clone(), fue - queda))
                })
                .collect();
            tx.rollback()?;

            assert_eq!(
                calculadas, medidas,
                "lo que el preflight dice que la cascada se lleva no es lo que SQLite se lleva"
            );
            // Y la cifra concreta, para que una lista a mano con el número mal
            // no pase por coincidir los nombres.
            assert_eq!(
                calculadas.get("activity_event").copied(),
                Some(3),
                "tres envíos archivados llevan sus tres sucesos"
            );
            Ok(())
        })
        .expect("cascada");
    }

    /// **El borrado se niega si el historial cambió, y no borra nada.**
    ///
    /// El motor escribe en su propio hilo: entre escribir el archivo y
    /// confirmar el borrado puede entrar un intento con fecha anterior al
    /// corte. Borrar «lo que haya» se llevaría una fila que el archivo no
    /// tiene.
    ///
    /// Verificada el 06/10/2026 quitando la comparación: borra las 3 y la
    /// prueba falla al comprobar que la base quedó intacta.
    #[test]
    fn el_borrado_se_niega_si_el_historial_cambio() {
        let (_d, db) = base();
        sembrar(&db);

        let antes = cuantas(&db, "message_attempt");
        let sucesos_antes = cuantas(&db, "activity_event");

        // Se le dice que archivó 2 cuando hay 3.
        let r = db.borrar_el_historial_archivado("E1", CORTE, 2);
        assert!(
            matches!(
                r,
                Err(DbError::ElHistorialCambio {
                    archivadas: 2,
                    ahora: 3
                })
            ),
            "{r:?}"
        );
        assert_eq!(cuantas(&db, "message_attempt"), antes, "no se borró nada");
        assert_eq!(cuantas(&db, "activity_event"), sucesos_antes);
    }

    /// **Las bajas y el consentimiento siguen ahí después de archivar.**
    ///
    /// La tercera condición de D-8D, comprobada contra la base y no contra una
    /// lista: archivar y borrar de verdad, y mirar que la baja está.
    ///
    /// Verificada el 06/10/2026 añadiendo `suppression_entry` a
    /// `EN_EL_ARCHIVO`: el archivo se la lleva, pero el borrado NO la borra
    /// —sólo borra el historial—, así que esta prueba seguiría pasando. Es
    /// exactamente el hueco que el validador cubre leyendo la lista, y por eso
    /// hacen falta las dos: aquí se comprueba el borrado, allí la lista.
    #[test]
    fn archivar_no_toca_las_bajas_ni_el_consentimiento() {
        let (_d, db) = base();
        sembrar(&db);

        db.borrar_el_historial_archivado("E1", CORTE, 3)
            .expect("borra");

        assert_eq!(cuantas(&db, "suppression_entry"), 1, "la baja sigue ahí");
        assert_eq!(cuantas(&db, "campaign"), 2, "las campañas siguen ahí");
    }

    /// **Queda en `audit_log`, con el periodo y las dos cifras.**
    ///
    /// Cuarta condición de D-8D: si un día hay que explicar por qué falta un
    /// año de historial, que esté escrito.
    ///
    /// Verificada el 06/10/2026 quitando la llamada a `auditar`: falla al no
    /// encontrar la fila.
    #[test]
    fn archivar_queda_en_la_bitacora() {
        let (_d, db) = base();
        sembrar(&db);

        db.borrar_el_historial_archivado("E1", CORTE, 3)
            .expect("borra");

        let resumen: String = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT entity_id FROM audit_log WHERE action = 'archivar_historial'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("bitácora");
        assert_eq!(resumen, format!("hasta={CORTE}/envios=3/arrastradas=3"));
    }

    /// **Un corte sin nada que archivar lo dice, en vez de escribir un archivo
    /// vacío.**
    #[test]
    fn un_corte_sin_nada_detras_no_tiene_nada_que_archivar() {
        let (_d, db) = base();
        sembrar(&db);

        let p = db
            .que_pasaria_al_archivar("2020-01-01T00:00:00Z")
            .expect("preflight");
        assert!(!p.hay_algo());
        assert_eq!(p.envios_que_se_archivan, 0);
        assert_eq!(p.primero, None, "sin filas no hay periodo que enseñar");
        assert_eq!(p.total_arrastrado(), 0);
    }
}
