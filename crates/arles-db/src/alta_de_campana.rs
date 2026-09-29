//! Crear una campaña, listarla, y **activarla**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ACTIVAR ES EL ACTO IRREVERSIBLE DE ARLES
//!
//! Todo lo demás se deshace: una plantilla se versiona, un contacto se edita,
//! una cuenta se reconfigura. Activar una campaña **no**. A partir de ese
//! instante hay correos en una cola que van a salir, y un correo que salió no
//! vuelve.
//!
//! Por eso activar hace tres cosas **en una sola transacción**, y o pasan las
//! tres o no pasa ninguna:
//!
//! 1. **Se congela la audiencia.** Quién entra en esta campaña se decide ahora
//!    y no se vuelve a calcular.
//! 2. **Se llena la cola**, una fila por destinatario, con su clave de
//!    idempotencia ya escrita.
//! 3. **La campaña pasa a «enviando».**
//!
//! Si se hicieran por separado, una caída entre la primera y la segunda dejaría
//! una campaña con audiencia y sin cola —enviando a nadie, para siempre— y una
//! entre la segunda y la tercera dejaría correos en cola de una campaña que
//! sigue diciendo «borrador». Las dos cosas son silenciosas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ SE CONGELA LA AUDIENCIA
//!
//! Sin instantánea, «¿a quién le llegó esta campaña?» se contestaría
//! recalculando la lista hoy — y hoy hay contactos que se han dado de baja,
//! otros que se importaron después y otros que se borraron.
//!
//! La respuesta cambiaría cada vez que se hace la pregunta, y eso no es una
//! respuesta. La instantánea guarda **la dirección**, no sólo el contacto: si
//! el contacto se borra, la fila se queda con el `contact_id` a nulo y la
//! dirección intacta. Una instantánea que encoge sola no es una instantánea.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS BAJAS SE MIRAN DOS VECES, Y NO SOBRA NINGUNA
//!
//! Aquí, al congelar, para no meter en la cola a quien ya había pedido no
//! recibir. Y otra vez en el motor, antes de cada envío (`bucle.rs`): entre que
//! una campaña se activa y un correo concreto sale pueden pasar semanas, y
//! alguien puede haber pedido la baja ayer.
//!
//! Quitar la de aquí llenaría la cola de gente a la que no hay que escribir —el
//! motor los descartaría uno a uno, pero las cifras de la campaña mentirían
//! desde el primer minuto. Quitar la del motor mandaría correos a quien pidió
//! no recibirlos.

use arles_core::campana::EstadoDeCampana;
use arles_core::ids::{
    CampaignId, CompanyId, EmailAccountId, IdempotencyKey, MessageAttemptId, TemplateVersionId,
};
use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::{ahora, auditar};
use crate::error::DbError;

/// Lo que hace falta para crear una campaña de correo.
///
/// Una campaña **nace con su etapa**: sin remitente, sin plantilla y sin
/// horario no se puede activar, y una campaña que no se puede activar es una
/// fila que la interfaz enseña y que no lleva a ninguna parte.
#[derive(Debug, Clone)]
pub struct CampanaNueva<'a> {
    pub nombre: &'a str,
    pub cuenta: EmailAccountId,
    pub version_de_plantilla: TemplateVersionId,
    /// La ventana de ejecución ya guardada. Es de la empresa, y se reutiliza
    /// entre campañas.
    pub ventana: &'a str,
    /// Los topes de esta etapa. `None` deja mandar el de la cuenta.
    pub limite_por_hora: Option<u32>,
    pub limite_diario: Option<u32>,
}

/// Una campaña tal como la enseña la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampanaGuardada {
    pub id: CampaignId,
    pub nombre: String,
    pub estado: EstadoDeCampana,
    /// Cuántos destinatarios tiene congelados. Cero mientras es borrador.
    pub audiencia: u32,
    /// Cuántos han salido ya. **No dice «entregados»** (§65): dice que el
    /// proveedor se hizo cargo.
    pub aceptados: u32,
}

/// Lo que pasó al activar. Cifras, no adjetivos (§94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activacion {
    /// Cuántos entraron en la audiencia congelada.
    pub en_la_audiencia: u32,
    /// Cuántos se dejaron fuera por estar en la lista de bajas.
    ///
    /// Se devuelve para poder decirlo: «se activó para 1 240 de 1 300 — 60
    /// habían pedido no recibir» es una cifra con la que se puede vivir. «Se
    /// activó» a secas deja a alguien preguntándose por qué el contador no
    /// coincide con su lista.
    pub fuera_por_baja: u32,
}

impl Db {
    /// Crea una campaña en borrador, con su etapa de correo.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla — entre otras cosas, si el
    /// remitente, la plantilla o la ventana no existen: son claves foráneas.
    pub fn crear_campana(
        &self,
        empresa: CompanyId,
        nueva: &CampanaNueva<'_>,
    ) -> Result<CampaignId, DbError> {
        let id = CampaignId::nuevo();
        let etapa = format!("st-1-{id}");

        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'draft', ?4, ?4)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    nueva.nombre.trim(),
                    cuando
                ],
            )?;

            // La etapa 1, de correo. El día que haya adaptador de WhatsApp,
            // aquí entra la etapa 2 — y el `CHECK` del esquema ya impide que
            // una etapa de correo cuelgue de un número de WhatsApp.
            tx.execute(
                "INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, template_version_id,
                                             execution_window_id, hourly_limit, daily_limit,
                                             status, created_at, updated_at)
                 VALUES (?1, ?2, 1, 'email', ?3, ?4, ?5, ?6, ?7, 'draft', ?8, ?8)",
                params![
                    etapa,
                    id.to_string(),
                    nueva.cuenta.to_string(),
                    nueva.version_de_plantilla.to_string(),
                    nueva.ventana,
                    nueva.limite_por_hora,
                    nueva.limite_diario,
                    cuando
                ],
            )?;

            auditar(
                &tx,
                &empresa.to_string(),
                "crear",
                "campaign",
                &id.to_string(),
            )?;
            tx.commit()?;
            Ok(id)
        })
    }

    /// Las campañas de una empresa, con sus cifras.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla, o [`DbError::DatoInvalido`] si
    /// una fila guarda un estado que el núcleo no conoce. **No se salta**: una
    /// campaña que desaparece de la lista sin decir por qué es peor que un
    /// error, porque nadie la va a echar de menos hasta que pregunte por ella.
    pub fn campanas(&self, empresa: CompanyId) -> Result<Vec<CampanaGuardada>, DbError> {
        self.con_dominio(|conn| {
            let mut c = conn.prepare(
                "SELECT c.id, c.name, c.status,
                        (SELECT count(*) FROM campaign_audience a
                          WHERE a.campaign_id = c.id),
                        (SELECT count(*) FROM message_attempt m
                          WHERE m.campaign_id = c.id AND m.state = 'sent')
                   FROM campaign c
                  WHERE c.company_id = ?1
                  ORDER BY c.updated_at DESC",
            )?;

            let filas = c.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, i64>(3)?,
                    f.get::<_, i64>(4)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, estado, audiencia, aceptados) = fila?;
                salida.push(CampanaGuardada {
                    id: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "campaign.id",
                    })?,
                    nombre,
                    estado: EstadoDeCampana::desde_texto(&estado).map_err(|_| {
                        DbError::DatoInvalido {
                            motivo: "campaign.status",
                        }
                    })?,
                    audiencia: u32::try_from(audiencia).unwrap_or(u32::MAX),
                    aceptados: u32::try_from(aceptados).unwrap_or(u32::MAX),
                });
            }
            Ok(salida)
        })
    }

    /// Activa una campaña: congela la audiencia, llena la cola y la pone a
    /// enviar. **En una sola transacción.**
    ///
    /// La audiencia son los contactos activos de la empresa con una dirección
    /// de correo, **menos los que están en la lista de bajas**.
    ///
    /// # Errores
    ///
    /// - [`DbError::DatoInvalido`] si la campaña no existe, si no está en
    ///   borrador o lista, o si no tiene etapa de correo.
    /// - [`DbError::AudienciaVacia`] si no queda nadie a quien escribir.
    ///   **Es un error y no un éxito con cero**: una campaña activada sin
    ///   destinatarios se queda diciendo «enviando» para siempre sin que nada
    ///   explique por qué el contador no sube.
    /// - [`DbError::Sqlite`] si la escritura falla.
    pub fn activar_campana(
        &self,
        empresa: CompanyId,
        campana: CampaignId,
    ) -> Result<Activacion, DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();
            let id = campana.to_string();

            // 1 · La campaña existe, es de esta empresa y se puede activar.
            let estado: Option<String> = tx
                .query_row(
                    "SELECT status FROM campaign WHERE id = ?1 AND company_id = ?2",
                    params![id, empresa.to_string()],
                    |f| f.get(0),
                )
                .optional()?;
            let Some(estado) = estado else {
                return Err(DbError::DatoInvalido {
                    motivo: "campaign.id",
                });
            };
            if !matches!(estado.as_str(), "draft" | "ready") {
                // Activar dos veces duplicaría la cola entera. El
                // `UNIQUE(campaign_id, channel, contact_address)` lo impediría,
                // pero fallaría con un error de SQLite en vez de decir lo que
                // pasa.
                return Err(DbError::DatoInvalido {
                    motivo: "campaign.status",
                });
            }

            // 2 · La etapa de correo, con su remitente.
            let etapa: Option<(String, String)> = tx
                .query_row(
                    "SELECT id, email_account_id FROM campaign_stage
                      WHERE campaign_id = ?1 AND channel = 'email'",
                    params![id],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
                .optional()?;
            let Some((etapa, cuenta)) = etapa else {
                return Err(DbError::DatoInvalido {
                    motivo: "campaign_stage.email_account_id",
                });
            };

            // 3 · La audiencia. Se congela con la dirección NORMALIZADA, que es
            // la forma con la que se compara todo lo demás —la deduplicación y
            // la supresión—, y la que el motor usa para escribir.
            //
            // Tres filtros, y los tres hacen falta: el contacto activo, **el
            // canal activo** y la dirección no suprimida. El del canal se
            // olvidó al escribir esto y lo cazó la prueba
            // `un_canal_dado_de_baja_no_entra`: un contacto puede seguir activo
            // y haber borrado uno de sus correos, y escribir a una dirección
            // que su dueño quitó es exactamente lo que ARLES promete no hacer.
            let fuera: i64 = tx.query_row(
                "SELECT count(*)
                   FROM contact c
                   JOIN contact_channel ch ON ch.contact_id = c.id
                  WHERE c.company_id = ?1 AND c.status = 'active'
                    AND ch.channel = 'email' AND ch.status = 'active'
                    AND EXISTS (SELECT 1 FROM suppression_entry s
                                 WHERE s.company_id = ?1 AND s.channel = 'email'
                                   AND s.address_normalized = ch.value_normalized)",
                params![empresa.to_string()],
                |f| f.get(0),
            )?;

            let metidos = tx.execute(
                "INSERT INTO campaign_audience (campaign_id, channel, contact_address,
                                                contact_id, added_at)
                 SELECT ?1, 'email', ch.value_normalized, c.id, ?3
                   FROM contact c
                   JOIN contact_channel ch ON ch.contact_id = c.id
                  WHERE c.company_id = ?2 AND c.status = 'active'
                    AND ch.channel = 'email' AND ch.status = 'active'
                    AND NOT EXISTS (SELECT 1 FROM suppression_entry s
                                     WHERE s.company_id = ?2 AND s.channel = 'email'
                                       AND s.address_normalized = ch.value_normalized)
                  GROUP BY ch.value_normalized",
                params![id, empresa.to_string(), cuando],
            )?;

            if metidos == 0 {
                return Err(DbError::AudienciaVacia);
            }

            // 4 · La cola. Una fila por destinatario, con su clave de
            // idempotencia escrita ANTES de que nadie llame a un proveedor
            // (ADR-0004). Es lo único que permite reconocer después un envío
            // ambiguo.
            {
                let mut insertar = tx.prepare(
                    "INSERT INTO message_attempt (id, campaign_id, stage_id, channel,
                                                  contact_id, contact_address,
                                                  email_account_id, idempotency_key,
                                                  state, attempt_count, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 'email', ?4, ?5, ?6, ?7, 'queued', 0, ?8, ?8)",
                )?;

                let mut destinatarios = tx.prepare(
                    "SELECT contact_address, contact_id FROM campaign_audience
                      WHERE campaign_id = ?1 AND channel = 'email'",
                )?;
                let filas = destinatarios.query_map(params![id], |f| {
                    Ok((f.get::<_, String>(0)?, f.get::<_, Option<String>>(1)?))
                })?;

                for fila in filas {
                    let (direccion, contacto) = fila?;
                    insertar.execute(params![
                        MessageAttemptId::nuevo().to_string(),
                        id,
                        etapa,
                        contacto,
                        direccion,
                        cuenta,
                        IdempotencyKey::nueva().to_string(),
                        cuando
                    ])?;
                }
            }

            // 5 · Y ahora sí, la campaña envía. La etapa también: son dos
            // estados desde la L-6, y dejar la etapa en borrador haría que el
            // motor no la viera nunca.
            tx.execute(
                "UPDATE campaign SET status = 'running', activated_at = ?2, updated_at = ?2
                  WHERE id = ?1",
                params![id, cuando],
            )?;
            tx.execute(
                "UPDATE campaign_stage SET status = 'running', activated_at = ?2,
                        updated_at = ?2
                  WHERE id = ?1",
                params![etapa, cuando],
            )?;

            auditar(&tx, &empresa.to_string(), "activar", "campaign", &id)?;
            tx.commit()?;

            Ok(Activacion {
                en_la_audiencia: u32::try_from(metidos).unwrap_or(u32::MAX),
                fuera_por_baja: u32::try_from(fuera).unwrap_or(u32::MAX),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;
    use crate::plantillas::Redaccion;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    /// Base con empresa, cuenta, ventana y una plantilla lista.
    fn base() -> (tempfile::TempDir, Db, TemplateVersionId) {
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
                &Redaccion {
                    asunto: "Hola {{nombre}}".to_owned(),
                    cuerpo_html: "<p>Gracias</p>".to_owned(),
                },
            )
            .expect("plantilla");

        (dir, db, version)
    }

    /// Añade un contacto activo con su correo.
    fn contacto(db: &Db, id: &str, correo: &str) {
        canal(db, id, correo, "active");
    }

    /// Un contacto con su canal en el estado que se le diga.
    fn canal(db: &Db, id: &str, correo: &str, estado: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT OR IGNORE INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('{id}', '{EMPRESA}', 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-{id}', '{EMPRESA}', '{id}', 'email', '{correo}', '{correo}',
                         '{estado}', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("contacto");
    }

    /// Y da de baja una dirección.
    fn suprimir(db: &Db, correo: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO suppression_entry (id, company_id, channel,
                                                address_normalized, scope, reason,
                                                origin, created_at)
                 VALUES ('sup-{correo}', '{EMPRESA}', 'email', '{correo}', 'global',
                         'unsubscribe', 'user', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("supresión");
    }

    fn una_campana(db: &Db, version: TemplateVersionId) -> CampaignId {
        db.crear_campana(
            empresa(),
            &CampanaNueva {
                nombre: "Septiembre",
                cuenta: CUENTA.parse().expect("id"),
                version_de_plantilla: version,
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: None,
            },
        )
        .expect("crea")
    }

    fn cuantos(db: &Db, tabla: &str, campana: CampaignId) -> i64 {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                &format!("SELECT count(*) FROM {tabla} WHERE campaign_id = ?1"),
                params![campana.to_string()],
                |f| f.get(0),
            )
        })
        .expect("cuenta")
    }

    /// Una campaña nace en borrador, con su etapa y sin audiencia.
    #[test]
    fn una_campana_nace_en_borrador_con_su_etapa() {
        let (_d, db, v) = base();
        let c = una_campana(&db, v);

        let lista = db.campanas(empresa()).expect("lista");
        let guardada = lista.first().expect("hay una");
        assert_eq!(guardada.id, c);
        assert_eq!(guardada.estado, EstadoDeCampana::Borrador);
        assert_eq!(guardada.audiencia, 0, "en borrador no hay audiencia");
        assert_eq!(cuantos(&db, "campaign_stage", c), 1, "nace con su etapa");
    }

    /// **Activar congela la audiencia y llena la cola, las dos cosas.**
    ///
    /// Es el corazón de la entrega. Una sin la otra deja una campaña que dice
    /// «enviando» y no envía, o correos en cola de una campaña que dice
    /// «borrador» — las dos cosas, en silencio.
    #[test]
    fn activar_congela_la_audiencia_y_llena_la_cola() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        let c = una_campana(&db, v);

        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 2);
        assert_eq!(r.fuera_por_baja, 0);

        assert_eq!(
            cuantos(&db, "campaign_audience", c),
            2,
            "audiencia congelada"
        );
        assert_eq!(cuantos(&db, "message_attempt", c), 2, "cola llena");

        let guardada = db.campanas(empresa()).expect("lista");
        assert_eq!(
            guardada.first().expect("hay una").estado,
            EstadoDeCampana::Enviando
        );
    }

    /// **Quien pidió la baja no entra en la cola, y se dice cuántos fueron.**
    ///
    /// La cifra importa: «se activó para 1 de 2 — uno había pedido no recibir»
    /// es algo con lo que se puede vivir. «Se activó» a secas deja a alguien
    /// preguntándose por qué el contador no coincide con su lista (§94).
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `NOT EXISTS` del
    /// `INSERT`, esta prueba falla.
    #[test]
    fn quien_pidio_la_baja_no_entra_en_la_campana() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        suprimir(&db, "beto@x.mx");
        let c = una_campana(&db, v);

        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 1);
        assert_eq!(r.fuera_por_baja, 1, "y se dice cuántos quedaron fuera");

        let en_cola: String = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT contact_address FROM message_attempt WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(en_cola, "ana@x.mx", "el suprimido no está en la cola");
    }

    /// **Sin nadie a quien escribir no se activa.**
    ///
    /// Es un error y no un éxito con cero: una campaña activada sin
    /// destinatarios se queda diciendo «enviando» para siempre y nada explica
    /// por qué el contador no sube.
    ///
    /// Y no deja rastro: la transacción revierte la audiencia que se hubiera
    /// escrito.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `if metidos == 0`,
    /// esta prueba falla.
    #[test]
    fn sin_nadie_a_quien_escribir_no_se_activa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        suprimir(&db, "ana@x.mx");
        let c = una_campana(&db, v);

        let error = db.activar_campana(empresa(), c).expect_err("no hay nadie");
        assert_eq!(error.clave_i18n(), "error.db.audiencia_vacia");

        assert_eq!(cuantos(&db, "campaign_audience", c), 0, "no queda rastro");
        assert_eq!(cuantos(&db, "message_attempt", c), 0);
        assert_eq!(
            db.campanas(empresa())
                .expect("lista")
                .first()
                .expect("una")
                .estado,
            EstadoDeCampana::Borrador,
            "y sigue en borrador"
        );
    }

    /// **Activar dos veces no duplica la cola.**
    ///
    /// Es el peor fallo posible: cada fila de más es un correo de más a una
    /// persona real. El `UNIQUE` del esquema lo impediría, pero fallaría con un
    /// error de SQLite; aquí se dice lo que pasa.
    #[test]
    fn activar_dos_veces_no_duplica_nada() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        let c = una_campana(&db, v);

        db.activar_campana(empresa(), c).expect("primera");
        let error = db.activar_campana(empresa(), c).expect_err("la segunda no");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");

        assert_eq!(
            cuantos(&db, "message_attempt", c),
            1,
            "una sola vez en la cola"
        );
    }

    /// **El esquema impide que dos contactos compartan una dirección.**
    ///
    /// ── LO QUE ESTA PRUEBA IBA A SER, Y LO QUE ES ──
    ///
    /// Se escribió para comprobar que una dirección repetida entra una sola vez
    /// en la audiencia — el caso de una importación con duplicados. **No se
    /// puede llegar a ese estado**: la V3 puso un
    /// `UNIQUE(company_id, channel, value_normalized)`, así que la segunda
    /// inserción falla antes.
    ///
    /// El `GROUP BY` de la consulta de activación se queda igualmente, como
    /// defensa en profundidad, y se dice aquí que **hoy no puede dispararse**:
    /// una defensa que nadie sabe que es redundante se convierte en la razón
    /// por la que alguien no busca el problema donde está.
    ///
    /// Lo que la prueba comprueba ahora es la garantía de verdad, que es más
    /// fuerte: la base no admite el duplicado.
    #[test]
    fn el_esquema_impide_dos_contactos_con_la_misma_direccion() {
        let (_d, db, _v) = base();
        contacto(&db, "k1", "ana@x.mx");

        let choque = db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('k2', '{EMPRESA}', 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-k2b', '{EMPRESA}', 'k2', 'email', 'Ana@X.MX', 'ana@x.mx',
                         'active', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        });

        assert!(
            choque.is_err(),
            "la base tiene que impedir que dos contactos compartan una dirección: \
             si no, la misma persona recibiría dos correos iguales"
        );
    }

    /// Un contacto archivado no recibe.
    #[test]
    fn un_contacto_archivado_no_entra() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("UPDATE contact SET status = 'archived' WHERE id = 'k2'")
        })
        .expect("archiva");

        let c = una_campana(&db, v);
        assert_eq!(
            db.activar_campana(empresa(), c)
                .expect("activa")
                .en_la_audiencia,
            1
        );
    }

    /// **Un canal dado de baja no entra, aunque el contacto siga activo.**
    ///
    /// ── ESTA PRUEBA ENCONTRÓ UN FALLO DE VERDAD ──
    ///
    /// La primera versión de la consulta filtraba por contacto activo y por
    /// dirección no suprimida, y **se olvidaba del estado del canal**. Un
    /// contacto puede seguir siendo cliente y haber borrado uno de sus correos;
    /// escribir a una dirección que su dueño quitó es exactamente lo que ARLES
    /// promete no hacer, y no habría fallado nada.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando `ch.status = 'active'` de
    /// la consulta —que es como estaba escrita— esta prueba falla y las demás
    /// siguen pasando.
    #[test]
    fn un_canal_dado_de_baja_no_entra() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("UPDATE contact_channel SET status = 'archived' WHERE id = 'ch-k2'")
        })
        .expect("da de baja el canal");

        let c = una_campana(&db, v);
        let r = db.activar_campana(empresa(), c).expect("activa");
        assert_eq!(r.en_la_audiencia, 1, "sólo el que sigue teniendo correo");

        let quien: String = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT contact_address FROM message_attempt WHERE campaign_id = ?1",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("lee");
        assert_eq!(quien, "ana@x.mx");
    }

    /// **Cada envío de la cola nace con su clave de idempotencia, y todas son
    /// distintas.**
    ///
    /// Se escribe **antes** de que nadie llame a un proveedor (ADR-0004): es lo
    /// único que permite reconocer después un envío ambiguo. Dos iguales harían
    /// que un reintento se confundiera con otro envío.
    #[test]
    fn cada_envio_nace_con_su_clave_de_idempotencia() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        let c = una_campana(&db, v);
        db.activar_campana(empresa(), c).expect("activa");

        let distintas: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT count(DISTINCT idempotency_key) FROM message_attempt
                      WHERE campaign_id = ?1 AND idempotency_key <> ''",
                    params![c.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(distintas, 2, "dos claves, y las dos distintas y no vacías");
    }

    /// La campaña de otra empresa no se puede activar desde aquí.
    #[test]
    fn no_se_puede_activar_la_campana_de_otra_empresa() {
        let (_d, db, v) = base();
        contacto(&db, "k1", "ana@x.mx");
        let c = una_campana(&db, v);

        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");
        let error = db.activar_campana(otra, c).expect_err("no es suya");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");
        assert_eq!(cuantos(&db, "message_attempt", c), 0);
    }
}
