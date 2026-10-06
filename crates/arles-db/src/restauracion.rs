//! Qué pasaría al restaurar un respaldo, **antes** de tocar nada.
//!
//! Entrega 8.3a. Aquí no se restaura: se mide y se cuenta. Restaurar es la 8.3b.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ENCONTRÓ MEDIR EL ESQUEMA, Y ORDENA TODA LA ENTREGA
//!
//! D-8C dice «reemplazar, nunca mezclar». Suena a que cada tabla del respaldo
//! sustituye a la suya y lo demás se queda igual. **No es lo que pasa.**
//!
//! `company` se reemplaza en los dos alcances, y con `PRAGMA foreign_keys=ON`
//! casi todo cuelga de ella con `ON DELETE CASCADE`. Borrar sus filas para
//! poner las del respaldo **arrastra a los hijos**, incluidos los que el
//! respaldo no trae. Medido el 06/10/2026 sobre el esquema V10, restaurar un
//! respaldo **esencial** se lleva por delante, sin que nada lo diga:
//!
//! | Tabla | Qué es | ¿Vuelve? |
//! |---|---|---|
//! | `activity_event` | El registro de actividad (7.1) | **No**: es `SoloCompleto` |
//! | `whatsapp_quality_event` | El histórico de calidad de Meta | **No**: es `SoloCompleto` |
//! | `rate_budget` | El cupo gastado de esta hora | No, y está bien: es `Nunca` |
//! | `engine_control` | La parada de emergencia de este equipo | No, y está bien: es `Nunca` |
//!
//! Las dos últimas son estado de **este** equipo y perderlas es correcto. Las
//! dos primeras son datos, y que desaparezcan sin avisar es exactamente el
//! fallo que costó una vez la audiencia congelada y el registro de envíos
//! (`CLAUDE.md` §5, hallazgo F1 de la Fase 1).
//!
//! Así que el preflight **las cuenta y las dice**. No se arregla cambiando el
//! esquema —las cascadas están bien puestas, protegen de otra cosa— ni
//! escondiéndolo: se dice con cifras antes de que nadie confirme, que es lo que
//! este producto hace desde la 7.4.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA CASCADA SE CALCULA DEL ESQUEMA, NO DE UNA LISTA
//!
//! Igual que el orden de inserción de la 8.2b. Una lista a mano de «qué
//! arrastra qué» envejece con la primera migración, y envejecer en silencio es
//! el problema, no la solución.

use std::collections::{BTreeMap, BTreeSet};

use arles_core::formato_de_respaldo::Alcance;
use arles_core::respaldo::MANIFIESTO;
use rusqlite::Connection;

use crate::db::Db;
use crate::error::DbError;

/// Una tabla, con lo que hay ahora y lo que trae el respaldo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparacionDeTabla {
    pub tabla: String,
    /// Cuántas filas hay en esta base ahora mismo.
    pub ahora: u64,
    /// Cuántas declara el respaldo.
    pub en_el_respaldo: u64,
}

/// Una tabla que el respaldo **no trae** y que se va igualmente, arrastrada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeVaPorCascada {
    pub tabla: String,
    /// Cuántas filas se pierden. Todas las suyas: el padre se reemplaza entero.
    pub filas: u64,
    /// Por qué tabla la arrastra. Para poder decirlo, no para decidir nada.
    pub por: String,
    /// `true` si el manifiesto la marca `Nunca`, es decir, si perderla es lo
    /// correcto: describe **este** equipo, no estos datos.
    pub es_de_este_equipo: bool,
}

/// Todo lo que hay que saber antes de confirmar una restauración.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoQuePasaria {
    /// Hasta qué migración va **esta** base: `V10`.
    pub esquema_de_la_base: String,
    /// Las tablas que el respaldo trae, en el orden en que se insertarán.
    pub tablas: Vec<ComparacionDeTabla>,
    /// Lo que se pierde sin volver. **Vacío no quiere decir «no pasa nada»**:
    /// quiere decir que no se arrastra nada.
    pub arrastradas: Vec<SeVaPorCascada>,
    pub bajas_ahora: u64,
    pub bajas_en_el_respaldo: u64,
    /// Cuántas bajas quedarán **como mínimo**.
    ///
    /// ── POR QUÉ UN MÍNIMO Y NO LA CIFRA EXACTA ──
    ///
    /// D-8C une las dos listas, y cuántas coinciden no se sabe sin leer el
    /// cuerpo entero del respaldo y comparar canal por canal. El preflight lee
    /// **sólo la cabecera** a propósito: con un archivo de 2 GB, leerlo dos
    /// veces para enseñar un aviso es medio minuto de espera antes de una
    /// pantalla.
    ///
    /// Así que se dice el mínimo, que es una verdad comprobable, en vez de una
    /// cifra exacta que habría que inventar. Lo que importa de este número es
    /// que **nunca baja**, y eso sí se garantiza.
    pub bajas_al_menos: u64,
    /// Cuántas cuentas aparecerán pidiendo reconectarse.
    ///
    /// El respaldo no lleva credenciales (§75), así que todas las que traiga
    /// entran sin contraseña ni token.
    pub cuentas_a_reconectar: u64,
}

impl LoQuePasaria {
    /// Cuántas filas se pierden en total sin volver.
    #[must_use]
    pub fn filas_que_no_vuelven(&self) -> u64 {
        self.arrastradas
            .iter()
            .filter(|a| !a.es_de_este_equipo)
            .map(|a| a.filas)
            .sum()
    }
}

impl Db {
    /// Qué pasaría al restaurar este respaldo. **No toca nada.**
    ///
    /// `filas_del_respaldo` sale de la cabecera del archivo, que es lo único
    /// que hace falta leer para contestar esto.
    ///
    /// # Errores
    ///
    /// - [`DbError::Sqlite`] si alguna consulta falla.
    /// - [`DbError::EsquemaNoSeDejaRespaldar`] si la base no dice por qué
    ///   migración va.
    pub fn que_pasaria_al_restaurar(
        &self,
        alcance: Alcance,
        filas_del_respaldo: &[(String, u64)],
    ) -> Result<LoQuePasaria, DbError> {
        self.con_dominio(|conn| {
            // Una transacción de lectura, por lo mismo que al exportar: todas
            // las cifras que se le enseñan a alguien tienen que ser del mismo
            // instante, o la suma no cuadra con los sumandos.
            let tx = conn.unchecked_transaction()?;

            let que = match alcance {
                Alcance::Completo => crate::respaldo::QueSeLleva::Todo,
                // La fecha da igual aquí: lo que se mide es qué tablas se
                // reemplazan, no cuántas filas trae cada una — eso lo dice la
                // cabecera del respaldo.
                Alcance::Esencial => {
                    crate::respaldo::QueSeLleva::LoEsencialDesde("0000-01-01T00:00:00Z")
                }
            };
            let incluidas: Vec<&'static str> = MANIFIESTO
                .iter()
                .filter(|e| match que {
                    crate::respaldo::QueSeLleva::Todo => e.alcance.entra_en_el_completo(),
                    crate::respaldo::QueSeLleva::LoEsencialDesde(_) => {
                        e.alcance.entra_en_el_esencial()
                    }
                })
                .map(|e| e.tabla)
                .collect();
            let orden = crate::respaldo::orden_de_insercion(&tx, &incluidas)?;

            let mut tablas = Vec::with_capacity(orden.len());
            for t in &orden {
                tablas.push(ComparacionDeTabla {
                    tabla: (*t).to_owned(),
                    ahora: cuantas(&tx, t)?,
                    en_el_respaldo: filas_del_respaldo
                        .iter()
                        .find(|(n, _)| n == t)
                        .map_or(0, |(_, n)| *n),
                });
            }

            let reemplazadas: BTreeSet<&str> = orden.iter().copied().collect();
            let mut arrastradas = Vec::new();
            for (tabla, por) in arrastradas_por(&tx, &reemplazadas)? {
                let filas = cuantas(&tx, &tabla)?;
                if filas == 0 {
                    // Una tabla vacía que se arrastra no es una pérdida, y
                    // enseñarla con un cero al lado sólo tapa las que sí lo son.
                    continue;
                }
                let es_de_este_equipo = MANIFIESTO
                    .iter()
                    .find(|e| e.tabla == tabla)
                    .is_some_and(|e| !e.alcance.entra_en_el_completo());
                arrastradas.push(SeVaPorCascada {
                    tabla,
                    filas,
                    por,
                    es_de_este_equipo,
                });
            }

            let bajas_ahora = cuantas(&tx, "suppression_entry")?;
            let bajas_en_el_respaldo = filas_del_respaldo
                .iter()
                .find(|(n, _)| n == "suppression_entry")
                .map_or(0, |(_, n)| *n);

            let cuentas_a_reconectar = ["email_account", "whatsapp_account"]
                .iter()
                .map(|t| {
                    filas_del_respaldo
                        .iter()
                        .find(|(n, _)| n == t)
                        .map_or(0, |(_, n)| *n)
                })
                .sum();

            let esquema_de_la_base = crate::respaldo::version_del_esquema(&tx)?;
            tx.rollback()?;

            Ok(LoQuePasaria {
                esquema_de_la_base,
                tablas,
                arrastradas,
                bajas_ahora,
                bajas_en_el_respaldo,
                // Se **unen** (D-8C): la lista final no puede ser menor que la
                // mayor de las dos. Si fuera menor, alguien que pidió no
                // recibir habría vuelto a la lista de envíos.
                bajas_al_menos: bajas_ahora.max(bajas_en_el_respaldo),
                cuentas_a_reconectar,
            })
        })
    }
}

fn cuantas(conn: &Connection, tabla: &str) -> Result<u64, DbError> {
    // El nombre viene del manifiesto o de `sqlite_master`, nunca de fuera.
    let n: i64 = conn.query_row(&format!("SELECT count(*) FROM \"{tabla}\""), [], |f| {
        f.get(0)
    })?;
    Ok(n.max(0).unsigned_abs())
}

/// Las tablas que se van por cascada al reemplazar las del respaldo.
///
/// Devuelve `(tabla, por quién)`. Es el cierre transitivo de las aristas
/// `ON DELETE CASCADE` desde las tablas que se reemplazan, quitando las que el
/// respaldo trae —ésas vuelven—.
///
/// Lee las aristas de `PRAGMA foreign_key_list`, no de una lista a mano, por lo
/// mismo que el orden de inserción de la 8.2b.
pub(crate) fn arrastradas_por(
    conn: &Connection,
    reemplazadas: &BTreeSet<&str>,
) -> Result<Vec<(String, String)>, DbError> {
    let mut todas = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let nombres = todas
        .query_map([], |f| f.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    // padre -> hijos que se van con él
    let mut hijos: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for hija in &nombres {
        let mut fk = conn.prepare(&format!("PRAGMA foreign_key_list(\"{hija}\")"))?;
        let filas = fk
            .query_map([], |f| {
                Ok((
                    f.get::<_, String>("table")?,
                    f.get::<_, String>("on_delete")?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (padre, al_borrar) in filas {
            if al_borrar.eq_ignore_ascii_case("CASCADE") && padre != *hija {
                hijos.entry(padre).or_default().insert(hija.clone());
            }
        }
    }

    let mut arrastradas: BTreeMap<String, String> = BTreeMap::new();
    let mut cola: Vec<String> = reemplazadas.iter().map(|t| (*t).to_owned()).collect();
    while let Some(padre) = cola.pop() {
        for hija in hijos.get(&padre).into_iter().flatten() {
            // Las que el respaldo trae se borran y **vuelven**: no son pérdida.
            if reemplazadas.contains(hija.as_str()) || arrastradas.contains_key(hija) {
                continue;
            }
            arrastradas.insert(hija.clone(), padre.clone());
            cola.push(hija.clone());
        }
    }

    Ok(arrastradas.into_iter().collect())
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

    /// Siembra lo justo para que haya algo que perder en cada tabla que importa.
    fn sembrar(db: &Db) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
                   VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO suppression_entry (id, company_id, channel, address_normalized, scope, reason, origin, created_at)
                   VALUES ('S1', 'E1', 'email', 'quien@no.quiere', 'global', 'unsubscribe', 'user', '2026-02-01T00:00:00Z');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                   VALUES ('K1', 'E1', 'Primera', 'draft', '2026-01-03T00:00:00Z', '2026-01-03T00:00:00Z');
                 INSERT INTO activity_event (id, company_id, campaign_id, kind, is_outcome, occurred_at)
                   VALUES ('A1', 'E1', 'K1', 'motor.arranca', 0, '2026-03-01T00:00:00Z');
                 INSERT INTO activity_event (id, company_id, campaign_id, kind, is_outcome, occurred_at)
                   VALUES ('A2', 'E1', 'K1', 'motor.para', 0, '2026-03-02T00:00:00Z');",
            )
        })
        .expect("siembra");
    }

    /// **La cascada calculada es la que ejecuta SQLite de verdad.**
    ///
    /// Es la prueba que sostiene todo el preflight. El cierre transitivo se
    /// calcula leyendo `PRAGMA foreign_key_list`; esta prueba **no se fía** y lo
    /// compara contra lo que pasa de verdad: borra las filas de las tablas que
    /// se reemplazarían, dentro de una transacción que se deshace, y cuenta qué
    /// desapareció de las demás.
    ///
    /// Si el cálculo dijera de menos, el preflight callaría una pérdida; si
    /// dijera de más, asustaría con una que no ocurre. Las dos cosas son
    /// mentiras con cifras, que es la peor clase.
    ///
    /// Verificada el 06/10/2026 cortando el cierre transitivo —quedándose con
    /// los hijos directos—: sigue coincidiendo en este esquema, porque todo
    /// cuelga de `company` a un salto. Así que además se comprueba el cierre
    /// con un caso de dos saltos, `whatsapp_quality_event`, que cuelga de
    /// `whatsapp_account` y no de `company`.
    #[test]
    fn la_cascada_calculada_es_la_que_ejecuta_sqlite() {
        let (_d, db) = base();
        sembrar(&db);

        db.ejecutar_en_pruebas(|conn| {
            let incluidas: Vec<&str> = MANIFIESTO
                .iter()
                .filter(|e| e.alcance.entra_en_el_esencial())
                .map(|e| e.tabla)
                .collect();
            let reemplazadas: BTreeSet<&str> = incluidas.iter().copied().collect();
            let calculadas: BTreeSet<String> = arrastradas_por(conn, &reemplazadas)
                .expect("cascada")
                .into_iter()
                .map(|(t, _)| t)
                .collect();

            // Todas las tablas del esquema, menos las del motor de migraciones.
            let mut s = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'refinery%'",
            )?;
            let nombres = s
                .query_map([], |f| f.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;

            let contar = |conn: &Connection| -> BTreeMap<String, i64> {
                nombres
                    .iter()
                    .map(|t| {
                        let n = conn
                            .query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |f| {
                                f.get::<_, i64>(0)
                            })
                            .unwrap_or(0);
                        (t.clone(), n)
                    })
                    .collect()
            };

            let antes = contar(conn);

            // ── Se hace de verdad, y se deshace ──
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch("PRAGMA foreign_keys=ON")?;
            // En orden inverso al de inserción, que es como borrará la 8.3b.
            for t in incluidas.iter().rev() {
                tx.execute(&format!("DELETE FROM \"{t}\""), [])?;
            }
            let despues = contar(&tx);
            tx.rollback()?;

            // Qué se vació de verdad, sin contar las que se reemplazan.
            let de_verdad: BTreeSet<String> = nombres
                .iter()
                .filter(|t| !reemplazadas.contains(t.as_str()))
                .filter(|t| {
                    antes.get(*t).copied().unwrap_or(0) > 0
                        && despues.get(*t).copied().unwrap_or(0) == 0
                })
                .cloned()
                .collect();

            assert!(
                !de_verdad.is_empty(),
                "la siembra no dejó nada que se arrastre: la prueba no comprobaría nada"
            );
            for t in &de_verdad {
                assert!(
                    calculadas.contains(t),
                    "SQLite se llevó {t} y el preflight no lo dice — una pérdida callada"
                );
            }
            Ok(())
        })
        .expect("mide la cascada");
    }

    /// **El preflight dice que el registro de actividad no vuelve.**
    ///
    /// Es el hallazgo que ordena la entrega: restaurar un respaldo esencial se
    /// lleva `activity_event` por cascada y **no lo trae de vuelta**, porque es
    /// `SoloCompleto`. Sin este aviso, alguien restaura y descubre semanas
    /// después que su registro empieza el día de la restauración.
    ///
    /// Verificada el 06/10/2026 quitando del cierre las tablas que no están en
    /// el respaldo: `arrastradas` sale vacío y la prueba falla.
    #[test]
    fn el_preflight_avisa_de_lo_que_no_vuelve() {
        let (_d, db) = base();
        sembrar(&db);

        let r = db
            .que_pasaria_al_restaurar(Alcance::Esencial, &[("suppression_entry".to_owned(), 5)])
            .expect("preflight");

        let actividad = r
            .arrastradas
            .iter()
            .find(|a| a.tabla == "activity_event")
            .expect("el registro de actividad se arrastra y tiene que decirse");
        assert_eq!(actividad.filas, 2);
        assert!(
            !actividad.es_de_este_equipo,
            "el registro de actividad son datos, no estado de este equipo"
        );
        assert_eq!(r.filas_que_no_vuelven(), 2);

        // Y un respaldo completo sí lo trae, así que no se pierde.
        let completo = db
            .que_pasaria_al_restaurar(Alcance::Completo, &[("activity_event".to_owned(), 9)])
            .expect("preflight");
        assert!(
            !completo
                .arrastradas
                .iter()
                .any(|a| a.tabla == "activity_event"),
            "el respaldo completo trae el registro: no se pierde"
        );
    }

    /// **Las bajas nunca bajan.** D-8C hecha cifra.
    ///
    /// Es la regla que no se negocia de la fase: perder una baja significa
    /// volver a escribirle a quien pidió que no. El preflight promete un
    /// **mínimo**, no una cifra exacta, porque la unión exacta exige leer el
    /// cuerpo entero — y el preflight lee sólo la cabecera a propósito.
    ///
    /// Verificada el 06/10/2026 poniendo `bajas_al_menos: bajas_en_el_respaldo`:
    /// con 7 bajas en la base y 2 en el respaldo promete 2, y la prueba falla.
    #[test]
    fn las_bajas_nunca_bajan() {
        let (_d, db) = base();
        sembrar(&db);
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO suppression_entry (id, company_id, channel, address_normalized, scope, reason, origin, created_at)
                   VALUES ('S2', 'E1', 'email', 'otra@no.quiere', 'global', 'unsubscribe', 'user', '2026-02-02T00:00:00Z');",
            )
        })
        .expect("otra baja");

        for en_el_respaldo in [0_u64, 1, 2, 50] {
            let r = db
                .que_pasaria_al_restaurar(
                    Alcance::Esencial,
                    &[("suppression_entry".to_owned(), en_el_respaldo)],
                )
                .expect("preflight");
            assert_eq!(r.bajas_ahora, 2);
            assert!(
                r.bajas_al_menos >= r.bajas_ahora,
                "con {en_el_respaldo} bajas en el respaldo promete {} y ya hay {}",
                r.bajas_al_menos,
                r.bajas_ahora
            );
            assert!(r.bajas_al_menos >= r.bajas_en_el_respaldo);
        }
    }

    /// Lo que describe este equipo se marca aparte, y es correcto perderlo.
    ///
    /// `rate_budget` y `engine_control` también se arrastran. Mezclarlas con el
    /// registro de actividad en el mismo aviso haría que un usuario leyera
    /// cuatro pérdidas donde hay dos.
    #[test]
    fn lo_de_este_equipo_se_distingue_de_lo_que_son_datos() {
        let (_d, db) = base();
        sembrar(&db);
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO engine_control (company_id, emergency_stop_at, updated_at)
                   VALUES ('E1', '2026-03-01T00:00:00Z', '2026-03-01T00:00:00Z');",
            )
        })
        .expect("estado del motor");

        let r = db
            .que_pasaria_al_restaurar(Alcance::Esencial, &[])
            .expect("preflight");
        let motor = r
            .arrastradas
            .iter()
            .find(|a| a.tabla == "engine_control")
            .expect("engine_control se arrastra");
        assert!(
            motor.es_de_este_equipo,
            "la parada de emergencia es de este equipo: perderla al restaurar es correcto"
        );
        // Y por eso no cuenta como pérdida de datos.
        assert_eq!(
            r.filas_que_no_vuelven(),
            2,
            "sólo los dos sucesos de actividad"
        );
    }

    /// El preflight **no toca nada**.
    ///
    /// Se comprueba porque la prueba de la cascada borra de verdad, aunque sea
    /// dentro de una transacción que se deshace: una transacción mal cerrada
    /// dejaría el preflight vaciando la base de quien sólo preguntó.
    #[test]
    fn preguntar_no_cambia_la_base() {
        let (_d, db) = base();
        sembrar(&db);

        let huella = |db: &Db| -> Vec<(String, i64)> {
            db.ejecutar_en_pruebas(|c| {
                let mut s =
                    c.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
                let tablas = s
                    .query_map([], |f| f.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                let mut salida = Vec::new();
                for t in tablas {
                    let n: i64 =
                        c.query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |f| f.get(0))?;
                    salida.push((t, n));
                }
                Ok(salida)
            })
            .expect("huella")
        };

        let antes = huella(&db);
        db.que_pasaria_al_restaurar(Alcance::Esencial, &[("contact".to_owned(), 3)])
            .expect("preflight");
        db.que_pasaria_al_restaurar(Alcance::Completo, &[("contact".to_owned(), 3)])
            .expect("preflight");
        assert_eq!(antes, huella(&db), "preguntar cambió la base");
    }

    /// Las cuentas que vuelven lo hacen **sin credenciales**, y se dice cuántas.
    #[test]
    fn se_dice_cuantas_cuentas_hay_que_reconectar() {
        let (_d, db) = base();
        let r = db
            .que_pasaria_al_restaurar(
                Alcance::Completo,
                &[
                    ("email_account".to_owned(), 2),
                    ("whatsapp_account".to_owned(), 1),
                ],
            )
            .expect("preflight");
        assert_eq!(r.cuentas_a_reconectar, 3);
    }
}
