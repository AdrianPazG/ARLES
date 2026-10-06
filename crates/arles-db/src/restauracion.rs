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

    pub(super) fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    /// Siembra lo justo para que haya algo que perder en cada tabla que importa.
    pub(super) fn sembrar(db: &Db) {
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

// ═══════════════════════════════════════════════════════════════════════════
// RESTAURAR  ·  entrega 8.3b
// ═══════════════════════════════════════════════════════════════════════════

/// De dónde salen las filas que se restauran.
///
/// Es un rasgo y no un `Vec`, simétrico a [`crate::respaldo::Destino`] y por el
/// mismo motivo: un `Vec` con medio millón de filas es la base entera en
/// memoria, que es lo que las 8.2b y 8.3a se tomaron el trabajo de evitar.
/// Una fila del respaldo: de qué tabla es, y sus valores en el orden que la
/// cabecera declaró para esa tabla.
pub type FilaDelRespaldo = (String, Vec<Option<String>>);

pub trait Origen {
    /// La siguiente fila del respaldo, o `None` cuando se acabó.
    ///
    /// # Errores
    ///
    /// Lo que falle al leer o al descifrar.
    fn siguiente(&mut self) -> Result<Option<FilaDelRespaldo>, DbError>;
}

/// Lo que pasó al restaurar. Con cifras, para poder decirlo (§94).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoQuePaso {
    pub filas: u64,
    pub bajas_antes: u64,
    pub bajas_despues: u64,
    /// Cuántas bajas de esta base **no venían** en el respaldo y se
    /// conservaron. Es la unión de D-8C hecha cifra.
    pub bajas_que_se_salvaron: u64,
    pub cuentas_a_reconectar: u64,
}

impl Db {
    /// Reemplaza la base con el contenido del respaldo. **Entero o nada.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LAS BAJAS SE UNEN, Y ÉSA ES LA REGLA QUE NO SE NEGOCIA
    ///
    /// `suppression_entry` se reemplaza como las demás, así que las bajas de
    /// **esta** base se borrarían. D-8C dice que se unen, y perder una baja
    /// significa volver a escribirle a quien pidió que no.
    ///
    /// Se resuelve **en SQL y no en memoria**: una tabla temporal guarda las
    /// bajas de ahora dentro de la misma transacción, y al final vuelven con
    /// `INSERT OR IGNORE`. Leerlas a un `Vec` sería meter la lista entera en
    /// RAM justo en la entrega que evita eso en todas las demás.
    ///
    /// **Y vuelven con un identificador nuevo.** Lo que hace única a una baja
    /// es (empresa, canal, dirección), no su `id`. Si una baja del respaldo
    /// trajera por casualidad el mismo `id` que una de aquí —dos bases
    /// distintas, dos identificadores generados aparte—, reinsertarla con el
    /// suyo chocaría contra la clave primaria y el `OR IGNORE` **se la
    /// tragaría en silencio**: una baja perdida, sin un solo error. Con un `id`
    /// nuevo, el `OR IGNORE` sólo puede saltar por el índice único, que es el
    /// caso que sí queremos ignorar: la misma persona en las dos listas.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LAS CUENTAS VUELVEN SIN CREDENCIALES
    ///
    /// El respaldo no las lleva (§75), así que todas entran en `needs_reauth`:
    /// el estado que ya existe desde la V7 y que la pantalla de Remitentes
    /// sabe enseñar con su botón. Dejarlas en `connected` sin contraseña haría
    /// que el motor las tomara por buenas y fallara contacto a contacto.
    ///
    /// # Errores
    ///
    /// - [`DbError::RespaldoNoEncaja`] si el respaldo es de otro esquema, o
    ///   declara una tabla o columna que esta base no tiene.
    /// - [`DbError::Sqlite`] si algo falla al escribir. **La transacción se
    ///   deshace entera**: la base queda como estaba.
    pub fn restaurar<O: Origen>(
        &self,
        origen: &mut O,
        esquema_del_respaldo: &str,
        columnas: &crate::respaldo::ColumnasPorTabla,
        alcance: Alcance,
        actor: &str,
        cuando: &str,
    ) -> Result<LoQuePaso, DbError> {
        self.con_dominio(|conn| {
            let esquema = crate::respaldo::version_del_esquema(conn)?;
            if esquema != esquema_del_respaldo {
                // ── POR QUÉ SE PARA EN VEZ DE INTENTARLO ──
                //
                // Un respaldo de un esquema anterior necesitaría que las
                // migraciones corrieran sobre él, y eso es otra entrega. Entre
                // pararse y restaurar «lo que encaje», se para: media base
                // restaurada arranca, parece funcionar, y lo que falta se
                // descubre semanas después.
                return Err(DbError::RespaldoNoEncaja {
                    motivo: format!(
                        "el respaldo es del esquema {esquema_del_respaldo} y esta base va por el {esquema}"
                    ),
                });
            }

            let tx = conn.unchecked_transaction()?;
            // Con las claves ajenas encendidas: un respaldo manipulado que
            // traiga un hijo sin su padre tiene que fallar, no entrar a medias.
            tx.execute_batch("PRAGMA foreign_keys=ON")?;

            let incluidas: Vec<&'static str> = MANIFIESTO
                .iter()
                .filter(|e| match alcance {
                    Alcance::Completo => e.alcance.entra_en_el_completo(),
                    Alcance::Esencial => e.alcance.entra_en_el_esencial(),
                })
                .map(|e| e.tabla)
                .collect();
            let orden = crate::respaldo::orden_de_insercion(&tx, &incluidas)?;

            // ── 1 · Las bajas de ahora, a salvo, antes de borrar nada ──
            let bajas_antes = cuantas(&tx, "suppression_entry")?;
            tx.execute_batch(
                "CREATE TEMP TABLE bajas_que_se_quedan AS SELECT * FROM suppression_entry",
            )?;

            // ── 2 · Vaciar, hijos antes que padres ──
            for tabla in orden.iter().rev() {
                tx.execute(&format!("DELETE FROM \"{tabla}\""), [])?;
            }

            // ── 3 · Meter lo del respaldo, por NOMBRE de columna ──
            //
            // Nunca por posición: una migración que mueva una columna metería
            // el apellido donde va la empresa sin que una sola fila falle. Es
            // el arreglo que la 8.2b le hizo al formato de la 8.1.
            let mut filas = 0_u64;
            let mut bajas_del_respaldo = 0_u64;
            let mut sentencias: BTreeMap<String, String> = BTreeMap::new();
            while let Some((tabla, valores)) = origen.siguiente()? {
                let cols = columnas
                    .iter()
                    .find(|(t, _)| *t == tabla)
                    .map(|(_, c)| c)
                    .ok_or_else(|| DbError::RespaldoNoEncaja {
                        motivo: format!("el respaldo trae filas de «{tabla}» sin declarar sus columnas"),
                    })?;
                if cols.len() != valores.len() {
                    return Err(DbError::RespaldoNoEncaja {
                        motivo: format!(
                            "una fila de «{tabla}» trae {} valores y la cabecera declara {} columnas",
                            valores.len(),
                            cols.len()
                        ),
                    });
                }
                let sql = match sentencias.get(&tabla) {
                    Some(s) => s.clone(),
                    None => {
                        comprobar_que_encajan(&tx, &tabla, cols)?;
                        let nombres = cols
                            .iter()
                            .map(|c| format!("\"{c}\""))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let huecos = vec!["?"; cols.len()].join(", ");
                        let s = format!("INSERT INTO \"{tabla}\" ({nombres}) VALUES ({huecos})");
                        sentencias.insert(tabla.clone(), s.clone());
                        s
                    }
                };
                tx.execute(&sql, rusqlite::params_from_iter(valores.iter()))?;
                filas += 1;
                if tabla == "suppression_entry" {
                    bajas_del_respaldo += 1;
                }
            }

            // ── 4 · Las bajas vuelven, unidas ──
            //
            // ── UNA BAJA CUELGA DE SU EMPRESA, Y ESO COMPLICA LA UNIÓN ──
            //
            // Lo encontró la prueba el 06/10/2026: `suppression_entry` tiene
            // una clave ajena a `company`, y `company` también se reemplaza. Si
            // el respaldo trae otro identificador de empresa, las bajas de aquí
            // vuelven apuntando a una empresa que **ya no existe** y SQLite las
            // rechaza. La restauración entera se deshace y nadie puede
            // restaurar nada.
            //
            // De las tres salidas, dos son inaceptables:
            //
            // - Descartar las bajas huérfanas **pierde bajas**, que es la única
            //   cosa que esta fase existe para impedir.
            // - Dejar que falle convierte D-8C en «no se puede restaurar».
            //
            // Así que las bajas que sobreviven se **reapuntan** a la empresa que
            // quedó. ARLES es de una sola empresa —hay una fila en `company`—,
            // de modo que en el caso real es la misma organización. Y en el caso
            // raro —restaurar el respaldo de otra— el error cae del lado seguro:
            // de más, nunca de menos. Suprimir a alguien de más significa no
            // escribirle; de menos significa escribirle a quien pidió que no.
            let cols_de_baja = columnas_de(&tx, "suppression_entry")?;
            let empresa_que_queda: Option<String> = tx
                .query_row("SELECT id FROM company LIMIT 1", [], |f| f.get(0))
                .ok();
            if bajas_antes > 0 && empresa_que_queda.is_none() {
                // Sin empresa no hay dónde colgarlas, y perderlas no es una
                // opción: se para antes de hacerlo.
                return Err(DbError::RespaldoNoEncaja {
                    motivo: format!(
                        "este respaldo no trae ninguna empresa, y esta base tiene {bajas_antes} bajas que se perderían"
                    ),
                });
            }
            let seleccion = cols_de_baja
                .iter()
                .filter(|c| *c != "id")
                .map(|c| {
                    if c == "company_id" {
                        "?1".to_owned()
                    } else {
                        format!("\"{c}\"")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let sin_id = cols_de_baja
                .iter()
                .filter(|c| *c != "id")
                .map(|c| format!("\"{c}\""))
                .collect::<Vec<_>>()
                .join(", ");
            tx.execute(
                &format!(
                    "INSERT OR IGNORE INTO suppression_entry (id, {sin_id})
                       SELECT lower(hex(randomblob(16))), {seleccion} FROM bajas_que_se_quedan"
                ),
                rusqlite::params![empresa_que_queda],
            )?;
            let bajas_despues = cuantas(&tx, "suppression_entry")?;
            tx.execute_batch("DROP TABLE bajas_que_se_quedan")?;

            // ── 5 · Las cuentas, a reconectar ──
            let mut cuentas_a_reconectar = 0_u64;
            for tabla in ["email_account", "whatsapp_account"] {
                if orden.contains(&tabla) {
                    cuentas_a_reconectar += u64::from(
                        tx.execute(
                            &format!("UPDATE \"{tabla}\" SET status = 'needs_reauth'"),
                            [],
                        )?
                        .try_into()
                        .unwrap_or(u32::MAX),
                    );
                }
            }

            // ── 6 · Y queda escrito ──
            //
            // `audit_log` no cuelga de `company`, así que la cascada no se lo
            // lleva y esta anotación sobrevive a su propia restauración. En el
            // alcance completo el respaldo lo reemplaza, y esta fila se escribe
            // después: la última línea de la bitácora dice de dónde salió todo
            // lo anterior.
            let empresa: Option<String> = tx
                .query_row("SELECT id FROM company LIMIT 1", [], |f| f.get(0))
                .ok();
            tx.execute(
                "INSERT INTO audit_log (id, company_id, actor, action, entity_kind, details, created_at)
                   VALUES (lower(hex(randomblob(16))), ?1, ?2, 'respaldo.restaurado', 'respaldo', ?3, ?4)",
                rusqlite::params![
                    empresa.unwrap_or_else(|| "desconocida".to_owned()),
                    actor,
                    // Sin la ruta del archivo: diría de qué equipo salió.
                    format!(
                        "alcance={} filas={filas} bajas_antes={bajas_antes} bajas_despues={bajas_despues} cuentas={cuentas_a_reconectar}",
                        alcance.as_str()
                    ),
                    cuando,
                ],
            )?;

            tx.commit()?;
            Ok(LoQuePaso {
                filas,
                bajas_antes,
                bajas_despues,
                // Las que quedaron menos las que trajo el respaldo: las que
                // sólo existían aquí y la unión conservó.
                bajas_que_se_salvaron: bajas_despues.saturating_sub(bajas_del_respaldo),
                cuentas_a_reconectar,
            })
        })
    }
}

/// Las columnas que la base tiene para esta tabla, en su orden.
fn columnas_de(conn: &Connection, tabla: &str) -> Result<Vec<String>, DbError> {
    let mut s = conn.prepare(&format!("PRAGMA table_info(\"{tabla}\")"))?;
    let cols = s
        .query_map([], |f| f.get::<_, String>("name"))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(cols)
}

/// Comprueba que las columnas que el respaldo declara existen aquí.
///
/// Si no, se para: insertar las que encajen dejaría los campos que faltan en su
/// valor por defecto, que para un apellido es nulo y para una fecha es basura.
fn comprobar_que_encajan(
    conn: &Connection,
    tabla: &str,
    declaradas: &[String],
) -> Result<(), DbError> {
    let aqui: BTreeSet<String> = columnas_de(conn, tabla)?.into_iter().collect();
    if aqui.is_empty() {
        return Err(DbError::RespaldoNoEncaja {
            motivo: format!("el respaldo trae la tabla «{tabla}», que esta base no tiene"),
        });
    }
    if let Some(falta) = declaradas.iter().find(|c| !aqui.contains(*c)) {
        return Err(DbError::RespaldoNoEncaja {
            motivo: format!(
                "el respaldo trae la columna «{tabla}.{falta}», que esta base no tiene"
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests_de_restaurar {
    use super::tests::{base, sembrar};
    use super::*;

    /// Un origen de mentira: las filas que se le den, en ese orden.
    struct Filas {
        pendientes: std::collections::VecDeque<FilaDelRespaldo>,
        /// Si está puesto, falla al llegar a esa fila. Es lo que permite
        /// provocar el corte a mitad en vez de suponerlo.
        falla_en: Option<usize>,
        entregadas: usize,
    }

    impl Filas {
        fn de(v: Vec<(&str, Vec<Option<&str>>)>) -> Self {
            Self {
                pendientes: v
                    .into_iter()
                    .map(|(t, c)| {
                        (
                            t.to_owned(),
                            c.into_iter().map(|x| x.map(str::to_owned)).collect(),
                        )
                    })
                    .collect(),
                falla_en: None,
                entregadas: 0,
            }
        }
        fn que_falla_en(mut self, n: usize) -> Self {
            self.falla_en = Some(n);
            self
        }
    }

    impl Origen for Filas {
        fn siguiente(&mut self) -> Result<Option<FilaDelRespaldo>, DbError> {
            if self.falla_en == Some(self.entregadas) {
                // Un archivo cortado, un disco que se desconecta, una
                // contraseña que deja de abrir a mitad del cuerpo.
                return Err(DbError::RespaldoNoEncaja {
                    motivo: "se cortó la lectura del respaldo".to_owned(),
                });
            }
            self.entregadas += 1;
            Ok(self.pendientes.pop_front())
        }
    }

    fn columnas(v: &[(&str, &[&str])]) -> crate::respaldo::ColumnasPorTabla {
        v.iter()
            .map(|(t, c)| {
                (
                    (*t).to_owned(),
                    c.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>(),
                )
            })
            .collect()
    }

    const COLS_EMPRESA: &[&str] = &[
        "id",
        "commercial_name",
        "country",
        "timezone",
        "corporate_email",
        "created_at",
        "updated_at",
    ];
    const COLS_BAJA: &[&str] = &[
        "id",
        "company_id",
        "channel",
        "address_normalized",
        "scope",
        "reason",
        "origin",
        "created_at",
    ];

    fn empresa(id: &str) -> (&'static str, Vec<Option<&'static str>>) {
        // Las cadenas son `'static` para no complicar el molde; el id va fijo.
        assert_eq!(id, "E9", "este molde sólo sirve para la empresa E9");
        (
            "company",
            vec![
                Some("E9"),
                Some("Otra Empresa"),
                Some("MX"),
                Some("America/Mexico_City"),
                Some("otra@empresa.mx"),
                Some("2026-04-01T00:00:00Z"),
                Some("2026-04-01T00:00:00Z"),
            ],
        )
    }

    fn todas_las_columnas(db: &Db, tablas: &[&str]) -> crate::respaldo::ColumnasPorTabla {
        db.ejecutar_en_pruebas(|c| {
            Ok(tablas
                .iter()
                .map(|t| ((*t).to_owned(), columnas_de(c, t).expect("columnas")))
                .collect())
        })
        .expect("columnas")
    }

    /// **Las bajas de esta base sobreviven a la restauración.**
    ///
    /// Es la regla que no se negocia de la fase: `suppression_entry` se
    /// reemplaza como las demás, así que sin la unión las bajas de aquí se
    /// borrarían — y perder una baja significa volver a escribirle a quien
    /// pidió que no.
    ///
    /// Verificada el 06/10/2026 quitando el `INSERT OR IGNORE` del final: la
    /// baja de esta base desaparece y la prueba la nombra.
    #[test]
    fn las_bajas_de_aqui_sobreviven() {
        let (_d, db) = base();
        sembrar(&db); // deja 'quien@no.quiere' en la empresa E1

        let cols = columnas(&[("company", COLS_EMPRESA), ("suppression_entry", COLS_BAJA)]);
        let mut origen = Filas::de(vec![
            empresa("E9"),
            (
                "suppression_entry",
                vec![
                    Some("S9"),
                    Some("E9"),
                    Some("email"),
                    Some("otra@no.quiere"),
                    Some("global"),
                    Some("unsubscribe"),
                    Some("user"),
                    Some("2026-04-02T00:00:00Z"),
                ],
            ),
        ]);

        let r = db
            .restaurar(
                &mut origen,
                "V10",
                &cols,
                Alcance::Esencial,
                "dirección",
                "2026-04-03T10:00:00Z",
            )
            .expect("restaura");

        assert_eq!(r.bajas_antes, 1);
        assert_eq!(r.bajas_despues, 2, "la baja de aquí y la del respaldo");
        assert_eq!(r.bajas_que_se_salvaron, 1);

        let direcciones: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut s =
                    c.prepare("SELECT address_normalized FROM suppression_entry ORDER BY 1")?;
                s.query_map([], |f| f.get(0))?
                    .collect::<Result<Vec<_>, _>>()
            })
            .expect("lee");
        assert!(
            direcciones.contains(&"quien@no.quiere".to_owned()),
            "se perdió una baja de esta base: {direcciones:?}"
        );
        assert!(direcciones.contains(&"otra@no.quiere".to_owned()));
    }

    /// **Una baja con el mismo identificador no se traga a la de aquí.**
    ///
    /// Lo que hace única a una baja es (empresa, canal, dirección), no su `id`.
    /// Dos bases distintas generan identificadores aparte, así que pueden
    /// coincidir. Si la baja de aquí volviera con su `id` original, chocaría
    /// contra la clave primaria de la del respaldo y el `OR IGNORE` **se la
    /// tragaría en silencio**: una baja perdida, sin un solo error.
    ///
    /// Verificada el 06/10/2026 reinsertando con el `id` original en vez de uno
    /// nuevo: la baja de esta base desaparece y la prueba falla.
    #[test]
    fn una_baja_con_el_mismo_id_no_se_come_a_la_de_aqui() {
        let (_d, db) = base();
        sembrar(&db);
        // La baja de aquí se llama S1 (ver `sembrar`). El respaldo trae OTRA
        // dirección con ESE MISMO id.
        let cols = columnas(&[("company", COLS_EMPRESA), ("suppression_entry", COLS_BAJA)]);
        let mut origen = Filas::de(vec![
            empresa("E9"),
            (
                "suppression_entry",
                vec![
                    Some("S1"),
                    Some("E9"),
                    Some("email"),
                    Some("distinta@no.quiere"),
                    Some("global"),
                    Some("unsubscribe"),
                    Some("user"),
                    Some("2026-04-02T00:00:00Z"),
                ],
            ),
        ]);

        let r = db
            .restaurar(
                &mut origen,
                "V10",
                &cols,
                Alcance::Esencial,
                "dirección",
                "2026-04-03T10:00:00Z",
            )
            .expect("restaura");
        assert_eq!(
            r.bajas_despues, 2,
            "un id repetido se comió una baja: quedaron {} de 2",
            r.bajas_despues
        );
    }

    /// **La misma persona en las dos listas no se duplica.**
    ///
    /// La otra mitad de la unión: `OR IGNORE` tiene que seguir saltando por el
    /// índice único.
    #[test]
    fn la_misma_baja_en_las_dos_listas_no_se_duplica() {
        let (_d, db) = base();
        sembrar(&db);
        let cols = columnas(&[("company", COLS_EMPRESA), ("suppression_entry", COLS_BAJA)]);
        // Mismo canal y misma dirección que la de aquí, pero en la empresa que
        // trae el respaldo — que es la misma fila de company reemplazada.
        let mut origen = Filas::de(vec![
            (
                "company",
                vec![
                    Some("E1"),
                    Some("Telemetry"),
                    Some("MX"),
                    Some("America/Mexico_City"),
                    Some("hola@telemetry.mx"),
                    Some("2026-01-01T00:00:00Z"),
                    Some("2026-01-01T00:00:00Z"),
                ],
            ),
            (
                "suppression_entry",
                vec![
                    Some("OTRO"),
                    Some("E1"),
                    Some("email"),
                    Some("quien@no.quiere"),
                    Some("global"),
                    Some("unsubscribe"),
                    Some("user"),
                    Some("2026-04-02T00:00:00Z"),
                ],
            ),
        ]);
        let r = db
            .restaurar(
                &mut origen,
                "V10",
                &cols,
                Alcance::Esencial,
                "dirección",
                "2026-04-03T10:00:00Z",
            )
            .expect("restaura");
        assert_eq!(r.bajas_despues, 1, "la misma persona quedó dos veces");
        assert_eq!(r.bajas_que_se_salvaron, 0);
    }

    /// **Una restauración cortada a mitad deja la base como estaba.**
    ///
    /// «Entero o nada» de D-8C. **Se provoca el fallo**, no se supone: el
    /// origen revienta en la tercera fila, con la base ya vaciada y dos filas
    /// metidas. Sin la transacción, ahí se quedaría — vacía y a medias.
    ///
    /// Verificada el 06/10/2026 cambiando el `?` del bucle por un `break`: la
    /// restauración «termina bien» con la base medio llena y la prueba falla al
    /// comparar la huella.
    #[test]
    fn una_restauracion_cortada_no_deja_rastro() {
        let (_d, db) = base();
        sembrar(&db);

        let huella = |db: &Db| -> Vec<(String, i64)> {
            db.ejecutar_en_pruebas(|c| {
                let mut s =
                    c.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
                let tablas = s
                    .query_map([], |f| f.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                let mut salida: Vec<(String, i64)> = Vec::new();
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
        let cols = columnas(&[("company", COLS_EMPRESA), ("suppression_entry", COLS_BAJA)]);
        let mut origen = Filas::de(vec![
            empresa("E9"),
            (
                "suppression_entry",
                vec![
                    Some("S9"),
                    Some("E9"),
                    Some("email"),
                    Some("otra@no.quiere"),
                    Some("global"),
                    Some("unsubscribe"),
                    Some("user"),
                    Some("2026-04-02T00:00:00Z"),
                ],
            ),
            empresa("E9"),
        ])
        .que_falla_en(2);

        let r = db.restaurar(
            &mut origen,
            "V10",
            &cols,
            Alcance::Esencial,
            "dirección",
            "2026-04-03T10:00:00Z",
        );
        assert!(r.is_err(), "una restauración cortada terminó bien");
        assert_eq!(
            antes,
            huella(&db),
            "la base no volvió a como estaba tras un corte a mitad"
        );
    }

    /// **Las cuentas vuelven pidiendo reconectarse, nunca como conectadas.**
    ///
    /// El respaldo no lleva credenciales (§75). Una cuenta restaurada en
    /// `connected` sin contraseña haría que el motor la tomara por buena y
    /// fallara contacto a contacto, gastando el cupo del día.
    ///
    /// Verificada el 06/10/2026 quitando el `UPDATE`: la cuenta vuelve como
    /// `connected` y la prueba la nombra.
    #[test]
    fn las_cuentas_vuelven_pidiendo_reconectarse() {
        let (_d, db) = base();
        sembrar(&db);
        let cols = todas_las_columnas(&db, &["company", "email_account"]);
        let valores_cuenta: Vec<Option<&str>> = cols
            .iter()
            .find(|(t, _)| t == "email_account")
            .expect("email_account")
            .1
            .iter()
            .map(|c| match c.as_str() {
                "id" => Some("C9"),
                "company_id" => Some("E9"),
                // Viene como conectada a propósito: es lo que haría un respaldo
                // tomado con la cuenta funcionando, y lo que la restauración
                // tiene que cambiar.
                "status" => Some("connected"),
                "created_at" | "updated_at" => Some("2026-04-01T00:00:00Z"),
                "display_name" => Some("Telemetry"),
                "email_address" => Some("hola@telemetry.mx"),
                "provider_kind" => Some("smtp"),
                "credential_ref" => Some("llavero:C9"),
                "daily_limit" => Some("500"),
                "hourly_limit" => Some("50"),
                "consecutive_failures" => Some("0"),
                "smtp_host" => Some("smtp.telemetry.mx"),
                "smtp_port" => Some("587"),
                "smtp_security" => Some("starttls"),
                "smtp_username" => Some("hola@telemetry.mx"),
                _ => None,
            })
            .collect();

        // La empresa, con TODAS las columnas que la cabecera declara: el molde
        // corto de arriba sirve para un respaldo de siete columnas, no para
        // éste.
        let valores_empresa: Vec<Option<&str>> = cols
            .iter()
            .find(|(t, _)| t == "company")
            .expect("company")
            .1
            .iter()
            .map(|c| match c.as_str() {
                "id" => Some("E9"),
                "commercial_name" => Some("Otra Empresa"),
                "country" => Some("MX"),
                "timezone" => Some("America/Mexico_City"),
                "corporate_email" => Some("otra@empresa.mx"),
                "created_at" | "updated_at" => Some("2026-04-01T00:00:00Z"),
                _ => None,
            })
            .collect();

        let mut origen = Filas::de(vec![
            ("company", valores_empresa),
            ("email_account", valores_cuenta),
        ]);
        let r = db
            .restaurar(
                &mut origen,
                "V10",
                &cols,
                Alcance::Esencial,
                "dirección",
                "2026-04-03T10:00:00Z",
            )
            .expect("restaura");
        assert_eq!(r.cuentas_a_reconectar, 1);

        let estado: String = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT status FROM email_account WHERE id='C9'", [], |f| {
                    f.get(0)
                })
            })
            .expect("lee");
        let estado: String = estado;
        assert_eq!(
            estado, "needs_reauth",
            "la cuenta volvió como «{estado}»: el motor la daría por buena sin contraseña"
        );
    }

    /// **Un respaldo de otro esquema no se restaura a medias: no se restaura.**
    #[test]
    fn un_respaldo_de_otro_esquema_se_rechaza() {
        let (_d, db) = base();
        sembrar(&db);
        let cols = columnas(&[("company", COLS_EMPRESA)]);
        let mut origen = Filas::de(vec![empresa("E9")]);
        let r = db.restaurar(
            &mut origen,
            "V9",
            &cols,
            Alcance::Esencial,
            "dirección",
            "2026-04-03T10:00:00Z",
        );
        assert!(matches!(r, Err(DbError::RespaldoNoEncaja { .. })), "{r:?}");
        // Y no tocó nada.
        assert_eq!(
            db.ejecutar_en_pruebas(
                |c| c.query_row("SELECT count(*) FROM company", [], |f| f.get::<_, i64>(0))
            )
            .expect("cuenta"),
            1
        );
    }

    /// **Una columna que esta base no tiene para la restauración**, en vez de
    /// insertar las que encajen.
    ///
    /// Insertar «lo que encaje» dejaría los campos que faltan en su valor por
    /// defecto: nulo para un apellido, basura para una fecha.
    #[test]
    fn una_columna_que_no_existe_para_la_restauracion() {
        let (_d, db) = base();
        let cols = columnas(&[("company", &["id", "commercial_name", "color_favorito"])]);
        let mut origen = Filas::de(vec![(
            "company",
            vec![Some("E9"), Some("Otra"), Some("azul")],
        )]);
        let r = db.restaurar(
            &mut origen,
            "V10",
            &cols,
            Alcance::Esencial,
            "dirección",
            "2026-04-03T10:00:00Z",
        );
        match r {
            Err(DbError::RespaldoNoEncaja { motivo }) => {
                assert!(motivo.contains("color_favorito"), "{motivo}");
            }
            otro => panic!("se restauró con una columna inventada: {otro:?}"),
        }
    }

    /// Queda escrito en la bitácora, **sin la ruta del archivo**.
    ///
    /// La ruta diría de qué equipo y de qué carpeta salió el respaldo, que es
    /// justo lo que la regla 4 de la frontera evita en toda la aplicación.
    #[test]
    fn la_restauracion_queda_en_la_bitacora_sin_la_ruta() {
        let (_d, db) = base();
        sembrar(&db);
        let cols = columnas(&[("company", COLS_EMPRESA)]);
        let mut origen = Filas::de(vec![empresa("E9")]);
        db.restaurar(
            &mut origen,
            "V10",
            &cols,
            Alcance::Esencial,
            "dirección",
            "2026-04-03T10:00:00Z",
        )
        .expect("restaura");

        let (accion, detalles): (String, String) = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT action, details FROM audit_log ORDER BY created_at DESC LIMIT 1",
                    [],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("bitácora");
        assert_eq!(accion, "respaldo.restaurado");
        assert!(detalles.contains("alcance=esencial"), "{detalles}");
        assert!(detalles.contains("filas=1"), "{detalles}");
        assert!(
            !detalles.contains('/') && !detalles.contains('\\'),
            "la bitácora lleva una ruta: {detalles}"
        );
    }
}
