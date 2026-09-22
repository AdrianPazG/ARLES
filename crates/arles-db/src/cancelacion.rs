//! El derecho de **Cancelación**: borrar de verdad (entrega 3.4).
//!
//! La regla y su porqué viven en `arles_core::cancelacion`. Aquí está el
//! trabajo sucio: qué tablas se vacían, cuáles se anonimizan y en qué orden.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! O SE HACE TODO, O NO SE HACE NADA
//!
//! Una transacción, sin excepciones. Un borrado a medias es peor que ninguno:
//! deja la ficha borrada y los envíos con la dirección en claro, y entonces
//! se le ha dicho a alguien que se borraron sus datos mientras siguen ahí.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL ORDEN IMPORTA, Y NO ES EL INTUITIVO
//!
//! Lo natural sería borrar la ficha primero. Está **mal**: `contact_id` de
//! `message_attempt` y de `campaign_audience` es `ON DELETE SET NULL`, así que
//! al borrar el contacto esas filas pierden el único puntero que las ata a él
//! —y su `contact_address` se queda en claro, sin que nada diga ya de quién
//! era—. Habría que buscarlas por dirección, y eso sólo funciona si la
//! dirección todavía se conoce.
//!
//! Así que primero se anonimiza lo que se conserva, y sólo después se borra lo
//! que se va.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ NO SE TOCA, Y POR QUÉ
//!
//! - **`suppression_entry`**: permanece, y se crea si no estaba. Es lo que
//!   impide volver a escribirle cuando su dirección reaparezca en un CSV.
//! - **`audit_log`**: append-only por disparador. Se le añade una línea con la
//!   dirección **en huella**, nunca en claro.
//! - **`consent_entry`**: append-only por disparador, y guarda la dirección en
//!   claro. **Es un hueco conocido** y está abierto como P-17: hoy no lo
//!   escribe nadie —el consentimiento se registrará cuando existan las
//!   campañas—, así que no hay nada que borrar todavía. La prueba
//!   `no_queda_la_direccion_en_ninguna_tabla` recorre el esquema entero y
//!   **fallará el día que alguien escriba ahí**, que es cuando habrá que
//!   decidirlo de verdad.

use arles_core::cancelacion::{Cancelacion, marca_de_borrado};
use arles_core::ids::{CompanyId, SuppressionEntryId};
use rusqlite::{Transaction, params};

use crate::db::Db;
use crate::error::DbError;

impl Db {
    /// Borra a una persona, conservando el hecho de los envíos sin su nombre.
    ///
    /// `direccion` viene **ya normalizada** por `arles_core::ValorDeCanal`:
    /// aquí no se normaliza nada. Dos normalizaciones serían dos reglas que
    /// mantener iguales, y la que fallara dejaría a alguien sin borrar
    /// creyendo que se borró.
    ///
    /// Es **irreversible**. No hay deshacer, y la pantalla que lo dispara
    /// tiene que decirlo antes, no después.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si algo falla. Entonces **no se borra nada**: la
    /// transacción se deshace entera.
    pub fn cancelar_a(
        &self,
        empresa: CompanyId,
        canal: arles_core::canal::Canal,
        direccion: &str,
    ) -> Result<Cancelacion, DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let empresa_txt = empresa.to_string();
            let canal_txt = canal.como_texto();

            // ── 0 · ¿De quién es esta dirección? ──────────────────────────
            //
            // Se buscan los contactos por su canal, incluidos los que ya
            // estaban dados de baja lógicamente (`deleted_at` puesto): una
            // ficha en la papelera sigue siendo un dato de esa persona, y el
            // derecho de cancelación llega también hasta ahí.
            let contactos: Vec<String> = {
                let mut q = tx.prepare(
                    "SELECT DISTINCT contact_id FROM contact_channel
                      WHERE company_id = ?1 AND channel = ?2
                        AND value_normalized = ?3",
                )?;
                let filas = q.query_map(params![empresa_txt, canal_txt, direccion], |f| {
                    f.get::<_, String>(0)
                })?;
                filas.collect::<Result<_, _>>()?
            };

            // ── 1 · Lo que se conserva, sin dueño ─────────────────────────
            //
            // Cada fila recibe **su propia** marca. Una marca compartida
            // permitiría reagrupar los envíos de esa persona, que es lo que se
            // está destruyendo. Ver la nota de `arles_core::cancelacion`.
            let envios = anonimizar(
                &tx,
                Conserva::Intentos,
                &contactos,
                &empresa_txt,
                canal_txt,
                direccion,
            )?;
            let audiencias = anonimizar(
                &tx,
                Conserva::Audiencia,
                &contactos,
                &empresa_txt,
                canal_txt,
                direccion,
            )?;

            // ── 2 · Lo que se borra de verdad ─────────────────────────────
            //
            // Se cuenta ANTES de borrar: después no hay nada que contar, y la
            // pantalla tiene que poder decir qué desapareció (§94).
            let (canales, campos, pertenencias) = contar_lo_que_se_va(&tx, &contactos)?;

            let mut contactos_borrados = 0_u32;
            for id in &contactos {
                // DELETE de verdad, no `deleted_at`. Las tablas hijas se van
                // en cascada: `contact_field`, `contact_channel`,
                // `contact_list_member` y `contact_tag` cuelgan de `contact`
                // con ON DELETE CASCADE.
                contactos_borrados += u32::try_from(tx.execute(
                    "DELETE FROM contact WHERE id = ?1 AND company_id = ?2",
                    params![id, empresa_txt],
                )?)
                .unwrap_or(u32::MAX);
            }

            // ── 3 · La baja, que se queda o se crea ───────────────────────
            let ya_estaba: bool = tx
                .query_row(
                    "SELECT 1 FROM suppression_entry
                      WHERE company_id = ?1 AND channel = ?2
                        AND address_normalized = ?3 LIMIT 1",
                    params![empresa_txt, canal_txt, direccion],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if !ya_estaba {
                // Motivo `unsubscribe`: lo pidió la propia persona, así que
                // **no se puede levantar con un botón**. Es la misma regla que
                // gobierna la lista de bajas, y aquí importa todavía más:
                // levantarla dejaría escribir a alguien que pidió desaparecer.
                tx.execute(
                    "INSERT INTO suppression_entry
                        (id, company_id, channel, address_normalized, scope,
                         request_id, reason, origin, notes, created_at)
                     VALUES (?1, ?2, ?3, ?4, 'channel', ?1,
                             'unsubscribe', 'user',
                             'Creada al cancelar sus datos', ?5)",
                    params![
                        SuppressionEntryId::nuevo().to_string(),
                        empresa_txt,
                        canal_txt,
                        direccion,
                        crate::empresa::ahora(),
                    ],
                )?;
            }

            // ── 4 · La bitácora, con la dirección en huella ───────────────
            //
            // Al final y no al principio: anotar una cancelación que después
            // no ocurre convertiría la bitácora en una lista de intenciones.
            // Si algo falla antes, la transacción se deshace y esta línea
            // nunca llega a escribirse.
            crate::empresa::auditar(
                &tx,
                &empresa_txt,
                "arco_cancelacion",
                "direccion",
                &arles_core::huella(direccion),
            )?;

            tx.commit()?;

            Ok(Cancelacion {
                contactos_borrados,
                canales_borrados: canales,
                campos_borrados: campos,
                pertenencias_borradas: pertenencias,
                envios_anonimizados: envios,
                audiencias_anonimizadas: audiencias,
                la_baja_ya_existia: ya_estaba,
            })
        })
    }
}

/// Las dos tablas que conservan el hecho de un envío.
///
/// Un tipo cerrado y no una cadena: con un `&str` el SQL se elegiría por
/// comparación de texto, haría falta una rama de error imposible, y el día que
/// alguien pasara un nombre de tabla de fuera tendríamos una inyección. Aquí
/// no hay nada que validar porque no hay nada que pueda venir mal.
#[derive(Clone, Copy)]
enum Conserva {
    /// Cada intento de envío y cómo acabó.
    Intentos,
    /// La audiencia congelada de una campaña.
    Audiencia,
}

impl Conserva {
    /// Busca por `rowid`, no por la clave primaria.
    ///
    /// `message_attempt` la tiene de texto y `campaign_audience` la tiene
    /// compuesta; `rowid` existe en las dos y vale lo mismo para lo único que
    /// hace falta aquí, que es volver a esa fila exacta para marcarla.
    ///
    /// Ninguna de las dos tiene `company_id`: cuelgan de `campaign`, que sí lo
    /// tiene. Se llega por ahí en vez de confiar en que una dirección sea
    /// única entre empresas — que no lo es.
    const fn buscar(self) -> &'static str {
        match self {
            Self::Intentos => {
                "SELECT a.rowid FROM message_attempt a
                   JOIN campaign c ON c.id = a.campaign_id
                  WHERE c.company_id = ?1 AND a.channel = ?2
                    AND (a.contact_address = ?3 OR a.contact_id = ?4)"
            }
            Self::Audiencia => {
                "SELECT a.rowid FROM campaign_audience a
                   JOIN campaign c ON c.id = a.campaign_id
                  WHERE c.company_id = ?1 AND a.channel = ?2
                    AND (a.contact_address = ?3 OR a.contact_id = ?4)"
            }
        }
    }

    const fn marcar(self) -> &'static str {
        match self {
            Self::Intentos => {
                "UPDATE message_attempt SET contact_address = ?2, contact_id = NULL
                  WHERE rowid = ?1"
            }
            Self::Audiencia => {
                "UPDATE campaign_audience SET contact_address = ?2, contact_id = NULL
                  WHERE rowid = ?1"
            }
        }
    }
}

/// Sustituye la dirección por una marca **distinta en cada fila**.
///
/// Lo de «distinta en cada fila» no es un detalle: una marca compartida
/// permitiría volver a agrupar los envíos de esa persona, y entonces no se
/// habría borrado nada. Por eso hay un `UPDATE` por fila en vez de uno solo
/// con `WHERE`, que sería más corto y escribiría el mismo valor en todas.
fn anonimizar(
    tx: &Transaction<'_>,
    tabla: Conserva,
    contactos: &[String],
    empresa: &str,
    canal: &str,
    direccion: &str,
) -> Result<u32, DbError> {
    // Una consulta por contacto, más una por la dirección suelta.
    //
    // La versión corta sería una lista `IN (…)` construida con `json_each`,
    // y para eso haría falta serializar los identificadores a JSON. **`arles-db`
    // no depende de `serde` a propósito** —es la regla de `ARQUITECTURA.md §4`
    // que lo mantiene fuera de la IPC, y el validador la comprueba—, así que
    // la lista se recorre a mano.
    //
    // No es un problema de rendimiento: una cancelación afecta a una persona,
    // y una persona tiene uno o dos contactos. Esto no es el camino del envío.
    let mut filas: Vec<i64> = Vec::new();
    {
        let mut q = tx.prepare(tabla.buscar())?;
        // Primero por dirección, que es lo que encuentra a quien ya no tiene
        // ficha: alguien borrado antes cuyos envíos siguen ahí con su correo.
        // `""` como contacto no casa con ninguno, porque los identificadores
        // son UUID.
        let it = q.query_map(params![empresa, canal, direccion, ""], |f| f.get(0))?;
        for fila in it {
            filas.push(fila?);
        }
        for contacto in contactos {
            let it = q.query_map(params![empresa, canal, direccion, contacto], |f| f.get(0))?;
            for fila in it {
                filas.push(fila?);
            }
        }
    }
    // Una fila puede haber salido por las dos vías —su dirección Y su
    // contacto—. Sin esto se marcaría dos veces: no rompe nada, pero contaría
    // de más, y la cifra que se le enseña a Dirección dejaría de ser cierta.
    filas.sort_unstable();
    filas.dedup();

    let mut cuantas = 0_u32;
    for fila in filas {
        cuantas += u32::try_from(tx.execute(tabla.marcar(), params![fila, marca_de_borrado()])?)
            .unwrap_or(0);
    }
    Ok(cuantas)
}

/// Cuenta lo que va a desaparecer, **antes** de que desaparezca.
///
/// Después no habría nada que contar, y la pantalla tiene que poder decir
/// exactamente qué se borró: un borrado irreversible que sólo dice «listo» no
/// le sirve a quien tiene que responderle a una persona (§94).
fn contar_lo_que_se_va(
    tx: &Transaction<'_>,
    contactos: &[String],
) -> Result<(u32, u32, u32), DbError> {
    let mut canales = 0_u32;
    let mut campos = 0_u32;
    let mut pertenencias = 0_u32;

    for contacto in contactos {
        let contar = |sql: &str| -> Result<u32, DbError> {
            let n: i64 = tx.query_row(sql, params![contacto], |f| f.get(0))?;
            Ok(u32::try_from(n).unwrap_or(u32::MAX))
        };
        canales += contar("SELECT COUNT(*) FROM contact_channel WHERE contact_id = ?1")?;
        campos += contar("SELECT COUNT(*) FROM contact_field WHERE contact_id = ?1")?;
        pertenencias += contar(
            "SELECT (SELECT COUNT(*) FROM contact_list_member WHERE contact_id = ?1)
                  + (SELECT COUNT(*) FROM contact_tag WHERE contact_id = ?1)",
        )?;
    }
    Ok((canales, campos, pertenencias))
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use arles_core::canal::{Canal, ValorDeCanal};
    use arles_core::cancelacion::es_marca_de_borrado;
    use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto, DatosDeContacto};
    use arles_core::supresion::{
        AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion, PeticionDeSupresion,
    };

    use crate::conexion::ClaveMaestra;

    fn base() -> (tempfile::TempDir, Db, CompanyId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(
                &arles_core::DatosDeEmpresa::validar(&arles_core::BorradorDeEmpresa {
                    nombre_comercial: "TELEMETRY INSIGHT".into(),
                    pais: "MX".into(),
                    zona_horaria: "America/Mexico_City".into(),
                    correo_corporativo: "hola@telemetrymx.com".into(),
                    sitio_web: String::new(),
                })
                .expect("empresa válida"),
            )
            .expect("guarda empresa");
        (dir, db, empresa)
    }

    fn contacto(canales: &[(Canal, &str)]) -> DatosDeContacto {
        DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: "Ana".into(),
                apellido: "Ruiz".into(),
                empresa: "Empresa SA".into(),
                canales: canales
                    .iter()
                    .map(|(c, v)| BorradorDeCanal {
                        canal: *c,
                        valor: (*v).to_owned(),
                        principal: false,
                    })
                    .collect(),
            },
            "MX",
        )
        .expect("contacto válido")
    }

    fn cv(canal: Canal, valor: &str) -> arles_core::contacto::CanalValidado {
        let v = ValorDeCanal::parse(canal, valor, "MX").expect("valor válido");
        arles_core::contacto::CanalValidado {
            canal: v.canal(),
            valor_raw: v.raw().to_owned(),
            valor_normalizado: v.normalized().to_owned(),
            principal: false,
        }
    }

    /// Deja una campaña con un envío y una audiencia para esa dirección.
    ///
    /// Se escribe con SQL crudo porque las campañas llegan en la entrega 6 y
    /// todavía no hay repositorio. Lo que se está probando es el borrado, no
    /// cómo se crea una campaña.
    fn con_envio(db: &Db, empresa: CompanyId, contacto_id: &str, direccion: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('camp-1', ?1, 'Septiembre', 'draft', '2026-09-01', '2026-09-01')",
                rusqlite::params![empresa.to_string()],
            )?;
            c.execute(
                "INSERT INTO campaign_audience
                    (campaign_id, channel, contact_address, contact_id, added_at)
                 VALUES ('camp-1', 'email', ?1, ?2, '2026-09-01')",
                rusqlite::params![direccion, contacto_id],
            )?;
            c.execute(
                "INSERT INTO message_attempt
                    (id, campaign_id, channel, contact_id, contact_address,
                     idempotency_key, state, created_at, updated_at)
                 VALUES ('int-1', 'camp-1', 'email', ?2, ?1, 'idem-1', 'sent',
                         '2026-09-02', '2026-09-02')",
                rusqlite::params![direccion, contacto_id],
            )?;
            Ok(())
        })
        .expect("siembra");
    }

    /// Cuántas filas de TODO el esquema contienen esa cadena en algún texto.
    ///
    /// Recorre las tablas y columnas de verdad, preguntándoselas a SQLite: una
    /// lista escrita a mano se queda vieja en cuanto alguien añade una tabla,
    /// y esta prueba dejaría de mirar justo donde estaría el fallo nuevo.
    fn donde_aparece(db: &Db, aguja: &str) -> Vec<String> {
        db.ejecutar_en_pruebas(|c| {
            let tablas: Vec<String> = {
                let mut q = c.prepare(
                    "SELECT name FROM sqlite_master
                      WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
                )?;
                q.query_map([], |f| f.get(0))?.collect::<Result<_, _>>()?
            };
            let mut encontrado = Vec::new();
            for tabla in tablas {
                let columnas: Vec<String> = {
                    let mut q = c.prepare(&format!("PRAGMA table_info({tabla})"))?;
                    q.query_map([], |f| f.get::<_, String>(1))?
                        .collect::<Result<_, _>>()?
                };
                for columna in columnas {
                    let sql = format!(
                        "SELECT COUNT(*) FROM \"{tabla}\" WHERE CAST(\"{columna}\" AS TEXT) = ?1"
                    );
                    let n: i64 = c.query_row(&sql, rusqlite::params![aguja], |f| f.get(0))?;
                    if n > 0 {
                        encontrado.push(format!("{tabla}.{columna} ({n})"));
                    }
                }
            }
            Ok(encontrado)
        })
        .expect("recorre el esquema")
    }

    /// ── LA PUERTA DE SALIDA DE LA CANCELACIÓN ──────────────────────────────
    ///
    /// Después de cancelar, la dirección **no puede quedar en claro en ninguna
    /// parte** salvo en la lista de bajas, que sobrevive a propósito porque es
    /// lo que impide volver a escribirle.
    ///
    /// Se recorre el ESQUEMA ENTERO preguntándoselo a SQLite, tabla por tabla
    /// y columna por columna. Una lista escrita a mano se quedaría vieja en
    /// cuanto alguien añadiera una tabla —y ése es justo el día en que esta
    /// prueba tendría que avisar—.
    ///
    /// Es además el guardián del hueco conocido de `consent_entry` (P-17): esa
    /// tabla es append-only y guarda la dirección en claro. Hoy no la escribe
    /// nadie, así que la prueba pasa. **El día que alguien registre un
    /// consentimiento, esta prueba fallará** y habrá que decidirlo de verdad
    /// en vez de descubrirlo en producción.
    ///
    /// VERIFICADO ROMPIÉNDOLO: quitando la anonimización de `message_attempt`
    /// falla nombrando `message_attempt.contact_address`.
    #[test]
    fn no_queda_la_direccion_en_ninguna_tabla() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";

        let guardado = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");
        con_envio(&db, empresa, &guardado.to_string(), direccion);

        // Antes: la dirección está por todas partes.
        let antes = donde_aparece(&db, direccion);
        assert!(
            antes.len() >= 3,
            "la siembra no dejó rastro suficiente para que la prueba signifique algo: {antes:?}"
        );

        db.cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");

        let despues = donde_aparece(&db, direccion);
        assert_eq!(
            despues,
            vec!["suppression_entry.address_normalized (1)".to_owned()],
            "la dirección sigue en claro donde no debería. Antes estaba en {antes:?}"
        );
    }

    /// El hecho del envío SOBREVIVE, sin dueño.
    ///
    /// Es la otra mitad y se prueba aparte: borrar la fila entera también
    /// dejaría la prueba de arriba en verde, y destruiría la auditoría que el
    /// §91 exige.
    #[test]
    fn el_envio_sobrevive_sin_dueño() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";
        let guardado = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");
        con_envio(&db, empresa, &guardado.to_string(), direccion);

        let resumen = db
            .cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");

        assert_eq!(resumen.envios_anonimizados, 1);
        assert_eq!(resumen.audiencias_anonimizadas, 1);

        let (cuantos, estado, quien, direccion_guardada): (i64, String, Option<String>, String) =
            db.ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT (SELECT COUNT(*) FROM message_attempt),
                            state, contact_id, contact_address
                       FROM message_attempt WHERE id = 'int-1'",
                    [],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?)),
                )
            })
            .expect("lee el intento");

        assert_eq!(
            cuantos, 1,
            "el intento se borró: la auditoría del §91 pierde el hecho"
        );
        assert_eq!(estado, "sent", "el resultado del envío cambió");
        assert_eq!(quien, None, "el intento sigue apuntando a un contacto");
        assert!(
            es_marca_de_borrado(&direccion_guardada),
            "la dirección no se sustituyó por una marca: {direccion_guardada}"
        );
    }

    /// Dos filas de la misma persona reciben marcas DISTINTAS.
    ///
    /// Con una marca compartida, cualquiera que abriera la base podría
    /// reagrupar los envíos de esa persona — y el vínculo que se pretendía
    /// cortar seguiría ahí, sólo que con otro nombre.
    #[test]
    fn cada_fila_recibe_su_propia_marca() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";
        let guardado = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");
        con_envio(&db, empresa, &guardado.to_string(), direccion);

        db.cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");

        let (del_intento, de_la_audiencia): (String, String) = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT (SELECT contact_address FROM message_attempt WHERE id = 'int-1'),
                            (SELECT contact_address FROM campaign_audience
                              WHERE campaign_id = 'camp-1')",
                    [],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("lee las dos");

        assert_ne!(
            del_intento, de_la_audiencia,
            "las dos filas llevan la misma marca: se pueden volver a agrupar"
        );
    }

    /// Cancelar a alguien que NO tenía baja **se la crea**.
    ///
    /// Sin esto, su dirección en un CSV la semana que viene volvería a recibir
    /// correos — de alguien que pidió desaparecer. Sería el peor fallo posible
    /// del producto, y no lo detectaría nada.
    #[test]
    fn cancelar_crea_la_baja_si_no_la_habia() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");

        let resumen = db
            .cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");
        assert!(!resumen.la_baja_ya_existia);

        let (cuantas, motivo): (i64, String) = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT COUNT(*), MAX(reason) FROM suppression_entry
                      WHERE address_normalized = ?1",
                    rusqlite::params![direccion],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("lee la baja");

        assert_eq!(
            cuantas, 1,
            "cancelar no dejó baja: se le puede volver a escribir"
        );
        assert_eq!(
            motivo, "unsubscribe",
            "la baja no consta como pedida por la persona, así que se podría levantar con un botón"
        );
    }

    /// Y si ya la tenía, **no se duplica** y se dice que ya estaba.
    #[test]
    fn cancelar_respeta_la_baja_que_ya_existia() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");
        db.suprimir(
            empresa,
            &PeticionDeSupresion {
                canales: vec![cv(Canal::Correo, direccion)],
                alcance: AlcanceDeSupresion::Canal,
                motivo: MotivoDeSupresion::Baja,
                origen: OrigenDeSupresion::Usuario,
                notas: String::new(),
            },
        )
        .expect("suprime");

        let resumen = db
            .cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");
        assert!(resumen.la_baja_ya_existia);

        let cuantas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM suppression_entry WHERE address_normalized = ?1",
                    rusqlite::params![direccion],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(cuantas, 1, "la baja se duplicó");
    }

    /// Cancelar a quien no está no es un error: es «no teníamos nada suyo».
    ///
    /// Y la baja se crea igual, para que siga sin tenerse.
    #[test]
    fn cancelar_a_quien_no_esta_no_falla_y_deja_la_baja() {
        let (_d, db, empresa) = base();

        let resumen = db
            .cancelar_a(empresa, Canal::Correo, "nadie@empresa.mx")
            .expect("no es un error");

        assert!(!resumen.habia_algo());
        assert_eq!(resumen.contactos_borrados, 0);
        assert!(!resumen.la_baja_ya_existia);

        let cuantas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM suppression_entry WHERE address_normalized = ?1",
                    rusqlite::params!["nadie@empresa.mx"],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(cuantas, 1);
    }

    /// La bitácora anota la cancelación **con la dirección en huella**.
    ///
    /// En claro sería absurdo: la bitácora no se puede borrar, así que borrar
    /// a alguien le dejaría el correo en la única tabla intocable del sistema
    /// — deshaciendo el borrado que acaba de pedir.
    #[test]
    fn la_bitacora_guarda_la_huella_y_no_la_direccion() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");

        db.cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");

        let guardado: String = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT entity_id FROM audit_log WHERE action = 'arco_cancelacion'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("hay anotación");

        assert_ne!(guardado, direccion, "la bitácora guardó el correo en claro");
        assert_eq!(guardado, arles_core::huella(direccion));
    }

    /// La ficha se borra **de verdad**, no con `deleted_at`.
    ///
    /// Un borrado lógico satisfaría a cualquiera que mirase la interfaz y
    /// dejaría los datos íntegros en el disco, que es exactamente lo que la
    /// ley no permite aquí.
    #[test]
    fn la_ficha_desaparece_de_la_tabla() {
        let (_d, db, empresa) = base();
        let direccion = "ana@empresa.mx";
        let guardado = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, direccion)]))
            .expect("crea");

        let resumen = db
            .cancelar_a(empresa, Canal::Correo, direccion)
            .expect("cancela");
        assert_eq!(resumen.contactos_borrados, 1);
        assert_eq!(resumen.canales_borrados, 1);

        let (fichas, canales): (i64, i64) = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT (SELECT COUNT(*) FROM contact WHERE id = ?1),
                            (SELECT COUNT(*) FROM contact_channel WHERE contact_id = ?1)",
                    rusqlite::params![guardado.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("cuenta");

        assert_eq!(
            fichas, 0,
            "la ficha sigue en la tabla: fue un borrado lógico"
        );
        assert_eq!(
            canales, 0,
            "sus canales sobrevivieron al borrado de la ficha"
        );
    }
}
