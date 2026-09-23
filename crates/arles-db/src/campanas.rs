//! Pausar, reanudar y detener una campaña, y la parada de emergencia.
//!
//! Las **reglas** —qué acción procede desde qué estado— viven en
//! `arles_core::campana`, no aquí. Este archivo las obedece y escribe.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE HACE QUE ESTO SEA DELICADO
//!
//! Detener toca **tres tablas a la vez**: el estado de la campaña, el de sus
//! etapas y el de los envíos pendientes. Si se escribieran por separado y el
//! programa muriera en medio, quedarían estados que se contradicen:
//!
//!   · campaña detenida con la cola intacta → al reabrir, ¿sigue o no sigue?
//!   · cola cancelada con la campaña enviando → la pantalla diría «enviando»
//!     sobre una cola vacía, sin fallar y sin enviar nada.
//!
//! Por eso todo va en **una transacción**. O se detiene entera, o no se
//! detiene.

use arles_core::attempt::AttemptState;
use arles_core::campana::{AccionDeCampana, EstadoDeCampana, ParadaDeEmergencia};
use arles_core::ids::CompanyId;
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Los estados de un envío que **todavía podían salir**.
///
/// Es la lista que detener cancela, y la misma que se cuenta para avisar
/// cuántos van a quedarse sin enviar. **Tienen que ser la misma lista**: si la
/// cuenta mirase más estados que la cancelación, la pantalla prometería
/// cancelar más de lo que cancela.
///
/// `sending` **no** está, y no es un olvido: esa fila ya se le pasó al
/// proveedor. Cancelarla diría que ese mensaje no salió, y puede que sí haya
/// salido. El trabajador que la tiene en la mano la termina y anota lo que
/// pasó de verdad; lo que impide que siga con la siguiente es el freno, no
/// esta cancelación.
const TODAVIA_PODIAN_SALIR: [AttemptState; 3] = [
    AttemptState::Queued,
    AttemptState::Claimed,
    AttemptState::Failed,
];

/// Qué pasó al pausar, reanudar o detener. **Cifras, no adjetivos** (§94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumenDeAccion {
    /// En qué estado queda la campaña.
    pub estado: EstadoDeCampana,
    /// Cuántos envíos se cancelaron. Cero salvo al detener.
    pub cancelados: u32,
    /// Cuántas etapas cambiaron de estado con la campaña.
    pub etapas: u32,
}

impl Db {
    /// En qué estado está una campaña.
    ///
    /// # Errores
    ///
    /// [`DbError::CampanaNoExiste`] si no hay tal campaña en esta empresa.
    /// [`DbError::Nucleo`] si el estado guardado no es uno de los siete.
    pub fn estado_de_campana(
        &self,
        empresa: CompanyId,
        campana: &str,
    ) -> Result<EstadoDeCampana, DbError> {
        self.con_dominio(|conn| leer_estado(conn, empresa, campana))
    }

    /// Cuántos envíos se quedarían sin salir si se detuviera ahora.
    ///
    /// Es la cifra que la pantalla enseña **antes** de pedir confirmación:
    /// «esto cancela 4 312 correos que no han salido». Un «¿seguro?» sin
    /// número no es una confirmación informada, y detener no se deshace.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn cuantos_quedan_sin_enviar(&self, campana: &str) -> Result<u32, DbError> {
        self.con(|conn| {
            let [a, b, c] = TODAVIA_PODIAN_SALIR;
            let cuantos: i64 = conn.query_row(
                "SELECT COUNT(*) FROM message_attempt
                  WHERE campaign_id = ?1 AND state IN (?2, ?3, ?4)",
                params![campana, a.as_str(), b.as_str(), c.as_str()],
                |f| f.get(0),
            )?;
            Ok(u32::try_from(cuantos).unwrap_or(u32::MAX))
        })
    }

    /// Pausa, reanuda o detiene una campaña.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LA DECISIÓN LA TOMA EL NÚCLEO, Y SE TOMA DENTRO DE LA TRANSACCIÓN
    ///
    /// El estado se lee y se comprueba **dentro** de la misma transacción que
    /// escribe. Leerlo fuera dejaría un hueco en el que otra acción —o la
    /// propia campaña al terminar— cambiara el estado entre la comprobación y
    /// la escritura: se validaría contra «enviando» y se escribiría sobre
    /// «terminada».
    ///
    /// Y el `UPDATE` **vuelve a exigir** el estado que se leyó, igual que hace
    /// la cola al tomar trabajo. Es el mismo seguro por el mismo motivo.
    ///
    /// Las etapas acompañan a la campaña: una campaña detenida con una etapa
    /// enviando es una contradicción que nadie sabría leer. Apagar un canal y
    /// dejar el otro (L-6) es otra acción, con su propio botón, y no ésta.
    ///
    /// # Errores
    ///
    /// - [`DbError::CampanaNoExiste`] si no hay tal campaña en esta empresa.
    /// - [`DbError::AccionDeCampanaImposible`] si la acción no procede, con la
    ///   clave del caso concreto.
    /// - [`DbError::Sqlite`] si alguna escritura falla.
    pub fn aplicar_a_la_campana(
        &self,
        empresa: CompanyId,
        campana: &str,
        accion: AccionDeCampana,
    ) -> Result<ResumenDeAccion, DbError> {
        let cuando = ahora();
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;

            let estaba = leer_estado(&tx, empresa, campana)?;
            let destino =
                accion
                    .aplicar(estaba)
                    .map_err(|e| DbError::AccionDeCampanaImposible {
                        clave: e.clave_i18n(),
                    })?;

            let filas = tx.execute(
                "UPDATE campaign SET status = ?1, updated_at = ?2
                  WHERE id = ?3 AND company_id = ?4 AND status = ?5",
                params![
                    destino.como_texto(),
                    cuando,
                    campana,
                    empresa.to_string(),
                    estaba.como_texto(),
                ],
            )?;
            if filas != 1 {
                // Otra cosa cambió el estado entre la lectura y esta escritura.
                // No se insiste: la acción se decidió sobre un estado que ya no
                // es el actual.
                return Err(DbError::AccionDeCampanaImposible {
                    clave: "campana.error.cambioMientrasTanto",
                });
            }

            // Las etapas que todavía no han terminado acompañan a la campaña.
            let etapas = tx.execute(
                "UPDATE campaign_stage SET status = ?1, updated_at = ?2
                  WHERE campaign_id = ?3 AND status IN (?4, ?5, ?6)",
                params![
                    destino.como_texto(),
                    cuando,
                    campana,
                    EstadoDeCampana::Programada.como_texto(),
                    EstadoDeCampana::Enviando.como_texto(),
                    EstadoDeCampana::EnPausa.como_texto(),
                ],
            )?;

            let cancelados = if accion.cancela_lo_pendiente() {
                let [a, b, c] = TODAVIA_PODIAN_SALIR;
                tx.execute(
                    "UPDATE message_attempt
                        SET state = ?1, updated_at = ?2, next_retry_at = NULL
                      WHERE campaign_id = ?3 AND state IN (?4, ?5, ?6)",
                    params![
                        AttemptState::Cancelled.as_str(),
                        cuando,
                        campana,
                        a.as_str(),
                        b.as_str(),
                        c.as_str(),
                    ],
                )?
            } else {
                0
            };

            tx.commit()?;
            Ok(ResumenDeAccion {
                estado: destino,
                cancelados: u32::try_from(cancelados).unwrap_or(u32::MAX),
                etapas: u32::try_from(etapas).unwrap_or(u32::MAX),
            })
        })
    }

    /// ¿Está puesta la parada de emergencia, y desde cuándo?
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn parada_de_emergencia(&self, empresa: CompanyId) -> Result<ParadaDeEmergencia, DbError> {
        self.con(|conn| {
            let fila: Option<(Option<String>, Option<String>)> = conn
                .query_row(
                    "SELECT emergency_stop_at, emergency_stop_by
                       FROM engine_control WHERE company_id = ?1",
                    params![empresa.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
                .optional()?;

            Ok(match fila {
                Some((Some(desde), por)) => ParadaDeEmergencia::Puesta { desde, por },
                // Sin fila, o con fila y sin instante: no hay parada. Son el
                // mismo caso a propósito — que la fila exista es un detalle de
                // cómo se guarda, no un estado del producto.
                _ => ParadaDeEmergencia::Levantada,
            })
        })
    }

    /// Pone la parada de emergencia: **que no salga nada más**.
    ///
    /// Es idempotente y **no refresca el instante** si ya estaba puesta.
    /// Refrescarlo haría que «detenido desde las 14:32» se convirtiera en
    /// «desde las 16:05» porque alguien volvió a pulsar, y ese dato es lo que
    /// después permite saber cuánto tiempo estuvo parado el envío.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn poner_parada_de_emergencia(
        &self,
        empresa: CompanyId,
        quien: Option<&str>,
    ) -> Result<ParadaDeEmergencia, DbError> {
        let cuando = ahora();
        self.con(|conn| {
            conn.execute(
                "INSERT INTO engine_control
                     (company_id, emergency_stop_at, emergency_stop_by, updated_at)
                 VALUES (?1, ?2, ?3, ?2)
                 ON CONFLICT(company_id) DO UPDATE SET
                     -- Si ya había una parada puesta, se respetan su instante y
                     -- su autor: los dos describen la MISMA parada, y quedarse
                     -- con el instante viejo y el autor nuevo daría una mezcla
                     -- que no ocurrió.
                     emergency_stop_at = COALESCE(engine_control.emergency_stop_at,
                                                  excluded.emergency_stop_at),
                     emergency_stop_by = CASE
                         WHEN engine_control.emergency_stop_at IS NOT NULL
                             THEN engine_control.emergency_stop_by
                         ELSE excluded.emergency_stop_by
                     END,
                     updated_at = excluded.updated_at",
                params![empresa.to_string(), cuando, quien],
            )?;
            Ok(())
        })?;
        self.parada_de_emergencia(empresa)
    }

    /// Levanta la parada. El motor vuelve a poder enviar.
    ///
    /// **No reanuda ninguna campaña**: las que estaban en pausa siguen en
    /// pausa, y las detenidas siguen detenidas. Levantar la parada quita el
    /// freno general; volver a arrancar cada campaña es una decisión aparte, y
    /// tiene que serlo — una parada de emergencia se pulsa porque algo iba mal,
    /// y lo último que debe hacer el botón de quitarla es poner a enviar todo a
    /// la vez.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn levantar_parada_de_emergencia(&self, empresa: CompanyId) -> Result<(), DbError> {
        let cuando = ahora();
        self.con(|conn| {
            conn.execute(
                "UPDATE engine_control
                    SET emergency_stop_at = NULL, emergency_stop_by = NULL, updated_at = ?2
                  WHERE company_id = ?1",
                params![empresa.to_string(), cuando],
            )?;
            Ok(())
        })
    }

    /// Lo que el motor pregunta antes de cada envío: **¿hay que parar?**
    ///
    /// Junta las dos cosas que pueden pararlo, porque al trabajador le da igual
    /// cuál de las dos sea:
    ///
    /// 1. La **parada de emergencia**, que vale para toda la empresa.
    /// 2. Que **esta campaña** haya dejado de estar en `enviando` — alguien la
    ///    pausó, la detuvo, o terminó.
    ///
    /// La segunda es lo que hace que pausar funcione de verdad. Sin ella,
    /// pausar cambiaría el estado en la pantalla y el motor seguiría enviando,
    /// que es exactamente la clase de mentira que este producto no se puede
    /// permitir.
    ///
    /// Una campaña que **no existe** cuenta como «hay que parar». Es el lado
    /// seguro: seguir enviando en nombre de algo que ya no está es peor que
    /// detenerse de más.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si alguna consulta falla.
    pub fn hay_que_parar(&self, empresa: CompanyId, campana: &str) -> Result<bool, DbError> {
        if self.parada_de_emergencia(empresa)?.hay_que_parar() {
            return Ok(true);
        }
        match self.estado_de_campana(empresa, campana) {
            Ok(estado) => Ok(estado != EstadoDeCampana::Enviando),
            Err(DbError::CampanaNoExiste) => Ok(true),
            Err(e) => Err(e),
        }
    }
}

/// El estado guardado de una campaña, ya convertido.
fn leer_estado(
    conn: &Connection,
    empresa: CompanyId,
    campana: &str,
) -> Result<EstadoDeCampana, DbError> {
    let texto: Option<String> = conn
        .query_row(
            "SELECT status FROM campaign WHERE id = ?1 AND company_id = ?2",
            params![campana, empresa.to_string()],
            |f| f.get(0),
        )
        .optional()?;
    let texto = texto.ok_or(DbError::CampanaNoExiste)?;
    Ok(EstadoDeCampana::desde_texto(&texto)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::campana::AccionDeCampana::{Detener, Pausar, Reanudar};
    use arles_core::campana::EstadoDeCampana as E;

    use crate::conexion::ClaveMaestra;

    const AHORA: &str = "2026-09-23T12:00:00Z";
    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const OTRA: &str = "22222222-2222-4222-8222-222222222222";

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id válido")
    }

    /// Una base con una empresa, una campaña **enviando** y una etapa suya.
    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            for id in [EMPRESA, OTRA] {
                c.execute(
                    "INSERT INTO company (id, commercial_name, country, timezone,
                                          corporate_email, created_at, updated_at)
                     VALUES (?1, 'TELEMETRY', 'MX', 'America/Mexico_City',
                             'hola@t.mx', ?2, ?2)",
                    params![id, AHORA],
                )?;
            }
            c.execute(
                "INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit, created_at, updated_at)
                 VALUES ('cuenta-1', ?1, 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://v',
                         'connected', 50, 10, ?2, ?2)",
                params![EMPRESA, AHORA],
            )?;
            c.execute(
                "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('camp', ?1, 'Prueba', 'running', ?2, ?2)",
                params![EMPRESA, AHORA],
            )?;
            c.execute(
                "INSERT INTO campaign_stage
                     (id, campaign_id, position, channel, email_account_id,
                      status, created_at, updated_at)
                 VALUES ('etapa-1', 'camp', 1, 'email', 'cuenta-1', 'running', ?1, ?1)",
                params![AHORA],
            )
        })
        .expect("siembra");
        (dir, db)
    }

    fn encolar(db: &Db, id: &str, direccion: &str, estado: AttemptState) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO message_attempt
                     (id, campaign_id, channel, contact_address, idempotency_key,
                      state, attempt_count, created_at, updated_at)
                 VALUES (?1, 'camp', 'email', ?2, ?3, ?4, 0, ?5, ?5)",
                params![id, direccion, format!("msgid-{id}"), estado.as_str(), AHORA],
            )
        })
        .expect("encola");
    }

    fn estado_del_intento(db: &Db, id: &str) -> String {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT state FROM message_attempt WHERE id = ?1",
                params![id],
                |f| f.get(0),
            )
        })
        .expect("lee")
    }

    fn estado_de_la_etapa(db: &Db) -> String {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT status FROM campaign_stage WHERE id = 'etapa-1'",
                [],
                |f| f.get(0),
            )
        })
        .expect("lee")
    }

    // ─── Pausar y detener son cosas distintas ───────────────────────────────

    /// **Pausar deja la cola intacta.** Es la diferencia entera con detener.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que `Pausar` cancele lo
    /// pendiente, los dos intentos de abajo pasan a `cancelled` y esta prueba
    /// falla.
    #[test]
    fn pausar_no_toca_un_solo_envio() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx", AttemptState::Queued);
        encolar(&db, "b", "beto@x.mx", AttemptState::Failed);

        let r = db
            .aplicar_a_la_campana(empresa(), "camp", Pausar)
            .expect("pausa");

        assert_eq!(r.estado, E::EnPausa);
        assert_eq!(r.cancelados, 0, "pausar es «espera», no «se acabó»");
        assert_eq!(estado_del_intento(&db, "a"), "queued");
        assert_eq!(estado_del_intento(&db, "b"), "failed");
    }

    /// Y reanudar devuelve la campaña a enviando con su cola donde estaba.
    #[test]
    fn reanudar_recupera_la_cola_tal_como_estaba() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx", AttemptState::Queued);
        db.aplicar_a_la_campana(empresa(), "camp", Pausar)
            .expect("pausa");

        let r = db
            .aplicar_a_la_campana(empresa(), "camp", Reanudar)
            .expect("reanuda");

        assert_eq!(r.estado, E::Enviando);
        assert_eq!(estado_del_intento(&db, "a"), "queued");
        assert_eq!(estado_de_la_etapa(&db), "running");
    }

    /// **Detener cancela lo que no había salido, y dice cuántos.**
    #[test]
    fn detener_cancela_lo_pendiente_y_da_la_cifra() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx", AttemptState::Queued);
        encolar(&db, "b", "beto@x.mx", AttemptState::Claimed);
        encolar(&db, "c", "cris@x.mx", AttemptState::Failed);

        let r = db
            .aplicar_a_la_campana(empresa(), "camp", Detener)
            .expect("detiene");

        assert_eq!(r.estado, E::Detenida);
        assert_eq!(r.cancelados, 3);
        for id in ["a", "b", "c"] {
            assert_eq!(estado_del_intento(&db, id), "cancelled");
        }
    }

    /// **Lo que ya salió no se toca, ni lo que se le pasó al proveedor.**
    ///
    /// Un `sent` cancelado diría que ese correo no salió, y salió. Un `sending`
    /// cancelado diría lo mismo sobre un mensaje que puede estar saliendo ahora
    /// mismo: esa fila la termina el trabajador que la tiene en la mano.
    ///
    /// Verificada rompiéndola el 23/09/2026: añadiendo `sent` y `sending` a la
    /// lista que se cancela, las dos comprobaciones de abajo fallan.
    #[test]
    fn detener_no_reescribe_la_historia_de_lo_que_ya_salio() {
        let (_d, db) = base();
        encolar(&db, "salio", "ana@x.mx", AttemptState::Sent);
        encolar(&db, "saliendo", "beto@x.mx", AttemptState::Sending);
        encolar(&db, "pendiente", "cris@x.mx", AttemptState::Queued);

        let r = db
            .aplicar_a_la_campana(empresa(), "camp", Detener)
            .expect("detiene");

        assert_eq!(r.cancelados, 1, "sólo el que todavía podía salir");
        assert_eq!(estado_del_intento(&db, "salio"), "sent");
        assert_eq!(
            estado_del_intento(&db, "saliendo"),
            "sending",
            "esa fila ya está en manos del proveedor: cancelarla diría que no \
             salió, y puede que sí"
        );
    }

    /// **La cifra que se enseña antes es la que se cancela después.**
    ///
    /// Si la cuenta mirase estados distintos de los que cancela la acción, la
    /// confirmación prometería una cosa y ocurriría otra — sobre algo que no se
    /// puede deshacer.
    #[test]
    fn el_aviso_promete_exactamente_lo_que_detener_cancela() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx", AttemptState::Queued);
        encolar(&db, "b", "beto@x.mx", AttemptState::Claimed);
        encolar(&db, "c", "cris@x.mx", AttemptState::Failed);
        encolar(&db, "d", "dani@x.mx", AttemptState::Sent);
        encolar(&db, "e", "eva@x.mx", AttemptState::Suppressed);

        let avisados = db.cuantos_quedan_sin_enviar("camp").expect("cuenta");
        let r = db
            .aplicar_a_la_campana(empresa(), "camp", Detener)
            .expect("detiene");

        assert_eq!(
            avisados, r.cancelados,
            "la pantalla avisó de {avisados} y se cancelaron {}",
            r.cancelados
        );
    }

    /// Las etapas acompañan a la campaña: no queda un canal enviando solo.
    #[test]
    fn las_etapas_se_paran_con_la_campana() {
        let (_d, db) = base();
        let r = db
            .aplicar_a_la_campana(empresa(), "camp", Detener)
            .expect("detiene");
        assert_eq!(r.etapas, 1);
        assert_eq!(estado_de_la_etapa(&db), "stopped");
    }

    /// **Detener no se deshace**, y el error lo dice con su clave propia.
    #[test]
    fn una_campana_detenida_no_se_reanuda() {
        let (_d, db) = base();
        db.aplicar_a_la_campana(empresa(), "camp", Detener)
            .expect("detiene");

        let error = db
            .aplicar_a_la_campana(empresa(), "camp", Reanudar)
            .expect_err("no procede");
        assert_eq!(error.clave_i18n(), "campana.error.detenerNoSeDeshace");
    }

    /// Y una acción que no procede **no escribe nada**.
    #[test]
    fn una_accion_que_no_procede_deja_todo_como_estaba() {
        let (_d, db) = base();
        encolar(&db, "a", "ana@x.mx", AttemptState::Queued);

        assert!(
            db.aplicar_a_la_campana(empresa(), "camp", Reanudar)
                .is_err(),
            "no se reanuda algo que ya está enviando"
        );
        assert_eq!(
            db.estado_de_campana(empresa(), "camp").expect("lee"),
            E::Enviando
        );
        assert_eq!(estado_del_intento(&db, "a"), "queued");
        assert_eq!(estado_de_la_etapa(&db), "running");
    }

    /// Una campaña de otra empresa no se toca, ni se admite que existe.
    #[test]
    fn no_se_puede_detener_la_campana_de_otra_empresa() {
        let (_d, db) = base();
        let ajena: CompanyId = OTRA.parse().expect("id");
        assert!(matches!(
            db.aplicar_a_la_campana(ajena, "camp", Detener),
            Err(DbError::CampanaNoExiste)
        ));
        assert_eq!(
            db.estado_de_campana(empresa(), "camp").expect("lee"),
            E::Enviando
        );
    }

    /// Un estado guardado que el núcleo no reconoce **no se adivina**.
    #[test]
    fn un_estado_corrupto_no_se_interpreta() {
        let (_d, db) = base();
        // Se salta la `CHECK` de la tabla a propósito, para simular una base
        // tocada por fuera o una migración a medias.
        db.ejecutar_en_pruebas(|c| {
            // La `CHECK` de la tabla rechaza este valor, y es bueno que lo
            // haga: es la primera barrera. Se desactiva a propósito para poder
            // comprobar la SEGUNDA — qué hace el código ante una base que ya
            // llegó corrupta, por una edición externa o una migración a medias.
            c.execute_batch("PRAGMA ignore_check_constraints = ON")?;
            let r = c.execute(
                "UPDATE campaign SET status = 'halted' WHERE id = 'camp'",
                [],
            );
            c.execute_batch("PRAGMA ignore_check_constraints = OFF")?;
            r
        })
        .expect("corrompe");

        assert!(db.estado_de_campana(empresa(), "camp").is_err());
    }

    // ─── La parada de emergencia ────────────────────────────────────────────

    /// **La parada sobrevive a cerrar el programa.**
    ///
    /// Es el motivo entero de la migración V6. Guardada en memoria, arrancar
    /// ARLES la levantaría sola y seguiría saliendo lo que alguien paró.
    ///
    /// Verificada rompiéndola el 23/09/2026: sin la escritura en
    /// `engine_control`, la base reabierta dice que no hay parada.
    #[test]
    fn la_parada_de_emergencia_sobrevive_a_cerrar_el_programa() {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("arles.db");
        let clave = ClaveMaestra::generar().expect("genera");
        {
            let db = Db::abrir(&ruta, &clave).expect("abre");
            db.ejecutar_en_pruebas(|c| {
                c.execute(
                    "INSERT INTO company (id, commercial_name, country, timezone,
                                          corporate_email, created_at, updated_at)
                     VALUES (?1, 'T', 'MX', 'America/Mexico_City', 'h@t.mx', ?2, ?2)",
                    params![EMPRESA, AHORA],
                )
            })
            .expect("siembra");
            db.poner_parada_de_emergencia(empresa(), Some("adrian"))
                .expect("para");
        }

        let db = Db::abrir(&ruta, &clave).expect("reabre");
        let parada = db.parada_de_emergencia(empresa()).expect("lee");
        assert!(
            parada.hay_que_parar(),
            "quien pulsó parar y cerró el programa no está delante para volver \
             a pulsarlo"
        );
        assert!(matches!(
            parada,
            ParadaDeEmergencia::Puesta { por: Some(ref q), .. } if q == "adrian"
        ));
    }

    /// Pulsar parar dos veces no cambia desde cuándo está parado.
    ///
    /// Ese instante es lo que después permite saber cuánto tiempo estuvo
    /// detenido el envío; refrescarlo lo perdería.
    #[test]
    fn volver_a_pulsar_parar_no_reescribe_la_hora() {
        let (_d, db) = base();
        let primera = db
            .poner_parada_de_emergencia(empresa(), Some("adrian"))
            .expect("para");
        let segunda = db
            .poner_parada_de_emergencia(empresa(), Some("otro"))
            .expect("vuelve a pulsar");
        assert_eq!(primera, segunda);
    }

    /// **Levantar la parada no reanuda nada por su cuenta.**
    ///
    /// Una parada de emergencia se pulsa porque algo iba mal. Lo último que
    /// debe hacer el botón de quitarla es poner a enviar todo a la vez.
    #[test]
    fn levantar_la_parada_no_reanuda_las_campanas() {
        let (_d, db) = base();
        db.aplicar_a_la_campana(empresa(), "camp", Pausar)
            .expect("pausa");
        db.poner_parada_de_emergencia(empresa(), None)
            .expect("para");

        db.levantar_parada_de_emergencia(empresa())
            .expect("levanta");

        assert!(
            !db.parada_de_emergencia(empresa())
                .expect("lee")
                .hay_que_parar()
        );
        assert_eq!(
            db.estado_de_campana(empresa(), "camp").expect("lee"),
            E::EnPausa,
            "quitar el freno general no decide por el usuario qué campañas \
             vuelven a arrancar"
        );
    }

    // ─── Lo que el motor pregunta ───────────────────────────────────────────

    /// **Pausar para el motor de verdad, no sólo en la pantalla.**
    ///
    /// Sin esta comprobación, pausar cambiaría el rótulo y los correos
    /// seguirían saliendo. Es la mentira que este producto no se puede
    /// permitir.
    #[test]
    fn pausar_hace_que_el_motor_pare() {
        let (_d, db) = base();
        assert!(
            !db.hay_que_parar(empresa(), "camp").expect("consulta"),
            "una campaña enviando no tiene por qué pararse"
        );

        db.aplicar_a_la_campana(empresa(), "camp", Pausar)
            .expect("pausa");

        assert!(
            db.hay_que_parar(empresa(), "camp").expect("consulta"),
            "si pausar no para el motor, el rótulo dice una cosa y el programa \
             hace otra"
        );
    }

    /// La parada de emergencia para cualquier campaña, esté como esté.
    #[test]
    fn la_parada_de_emergencia_para_una_campana_que_estaba_enviando() {
        let (_d, db) = base();
        db.poner_parada_de_emergencia(empresa(), None)
            .expect("para");
        assert!(db.hay_que_parar(empresa(), "camp").expect("consulta"));

        db.levantar_parada_de_emergencia(empresa())
            .expect("levanta");
        assert!(!db.hay_que_parar(empresa(), "camp").expect("consulta"));
    }

    /// Una campaña que no existe cuenta como «hay que parar».
    ///
    /// Es el lado seguro: seguir enviando en nombre de algo que ya no está es
    /// peor que detenerse de más.
    #[test]
    fn una_campana_que_no_existe_para_el_motor() {
        let (_d, db) = base();
        assert!(
            db.hay_que_parar(empresa(), "no-existe").expect("consulta"),
            "ante la duda, no se envía"
        );
    }
}
