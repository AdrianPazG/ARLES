//! Qué hay que enviar ahora mismo: las etapas en marcha, con sus reglas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTA CONSULTA EXISTE
//!
//! El trabajador de la Fase 4 necesita unos [`Ajustes`] completos para dar un
//! paso: la campaña, la cuenta, la ventana horaria y los límites. Hasta ahora
//! esos ajustes sólo los construían las pruebas, a mano. Para que el motor
//! corra de verdad dentro de la aplicación hace falta sacarlos de la base, y
//! ése es todo el trabajo de este módulo.
//!
//! Va en `arles-db` y no en `arles-motor` porque el motor **no escribe SQL**:
//! pide con métodos tipados. Es la regla que mantiene todas las consultas del
//! programa en un sitio donde se pueden leer de una sentada (ADR-0002).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LOS LÍMITES: SE TOMA EL MÁS BAJO, SIEMPRE
//!
//! Hay dos topes para el mismo envío y **no se suman ni se eligen**: se toma el
//! menor de los dos.
//!
//! - **La etapa** puede tener el suyo, y puede no tenerlo. Es donde vive el
//!   ritmo desde la V4, y existe ahí porque WhatsApp empieza mucho más lento
//!   que el correo: un tope compartido obligaría a elegir entre frenar el
//!   correo o quemar el número (L-6).
//! - **La cuenta** tiene el suyo siempre. Y es el que no se puede saltar: quien
//!   cuenta los correos de verdad es el proveedor, y pasarse de su límite no da
//!   un error claro — da una cuenta marcada como spam, que no se arregla
//!   bajando el número después.
//!
//! Dejar ganar al de la etapa cuando fuera mayor que el de la cuenta sería
//! exactamente ese error, y no fallaría nada: los correos saldrían.
//!
//! **La campaña no tiene límites propios.** Los tuvo hasta la V4, que los movió
//! a la etapa junto con la ventana, el remitente y la plantilla. Se dice aquí
//! porque el esquema de la V1 —que es el que se lee primero— todavía los
//! enseña, y buscarlos ahí cuesta un rato.

use arles_core::canal::Canal;
use arles_core::ids::{CompanyId, EmailAccountId};
use arles_core::ritmo::Limites;
use arles_core::ventana::{Dias, TimeZone, Ventana, zona};
use rusqlite::params;

use crate::db::Db;
use crate::error::DbError;

/// Una etapa que está enviando, con todo lo que el trabajador necesita saber.
///
/// Es deliberadamente **lo que cabe en unos `Ajustes`** y nada más. No lleva el
/// nombre de la campaña ni cuántos quedan: eso es para la pantalla, y meterlo
/// aquí obligaría a esta consulta a crecer cada vez que la pantalla cambie.
#[derive(Debug, Clone)]
pub struct EtapaEnMarcha {
    /// La etapa misma.
    ///
    /// Hace falta desde la 7.1: el registro de actividad recorta el ruido
    /// **por etapa**, no por campaña. Con dos etapas fuera de ventana a la vez,
    /// una comparación por campaña dejaría muda a la segunda.
    pub etapa: String,
    /// La campaña de la que se toma trabajo.
    pub campana: String,
    /// La cuenta remitente. **El presupuesto cuelga de ella.**
    pub cuenta: EmailAccountId,
    /// Por qué medio se manda.
    pub canal: Canal,
    /// Los días y horas en que puede enviar, con su zona (D-10).
    pub ventana: Ventana,
    /// El menor de los topes de la etapa y de la cuenta.
    pub limites: Limites,
    /// La zona de **la empresa**, donde se cuentan las horas y los días del
    /// presupuesto. No es necesariamente la de la ventana: una campaña dirigida
    /// a Madrid se envía en horario de Madrid y gasta del cupo diario de una
    /// cuenta que vive en México.
    pub zona_de_la_empresa: TimeZone,
}

impl Db {
    /// Las etapas de correo que están enviando ahora mismo.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LAS TRES CONDICIONES, Y POR QUÉ SON TRES
    ///
    /// **La campaña está en marcha** y **la etapa también**. Son dos estados
    /// distintos a propósito desde la L-6: apagar el canal de WhatsApp de una
    /// campaña no apaga el de correo. Mirar sólo el de la campaña haría que una
    /// etapa pausada siguiera enviando, y el rótulo de la pantalla diría
    /// «pausada» mientras salen los correos.
    ///
    /// **Y el canal es `email`.** Hoy no hay ningún adaptador de WhatsApp: la
    /// Fase 3W va por el esquema. Devolver una etapa de WhatsApp aquí no
    /// produciría un error, produciría un trabajador sin proveedor — o, peor,
    /// uno al que alguien le enchufara el de correo por descuido.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// UNA ETAPA MAL CONFIGURADA DETIENE LA LECTURA. NO SE SALTA.
    ///
    /// Si una ventana guardada no vale —una zona que ya no existe en la base
    /// IANA, unas horas al revés—, esta función devuelve error en vez de
    /// devolver las demás etapas.
    ///
    /// Saltársela sería lo cómodo y dejaría una campaña activada que **no envía
    /// y no lo dice**: en la pantalla pone «en marcha» y el motor la ignora en
    /// silencio para siempre. Un error se ve; una campaña muda, no.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla, o [`DbError::DatoInvalido`] si
    /// una etapa guardada no produce unos ajustes válidos.
    pub fn etapas_en_marcha(&self, empresa: CompanyId) -> Result<Vec<EtapaEnMarcha>, DbError> {
        let empresa_txt = empresa.to_string();

        self.con_dominio(|conn| {
            // La zona de la empresa se lee una vez: es la misma para todas.
            let zona_empresa_txt: String = conn.query_row(
                "SELECT timezone FROM company WHERE id = ?1",
                params![empresa_txt],
                |f| f.get(0),
            )?;
            let zona_de_la_empresa =
                zona(&zona_empresa_txt).map_err(|_| DbError::DatoInvalido {
                    motivo: "company.timezone",
                })?;

            let mut consulta = conn.prepare(
                "SELECT e.id,
                        e.campaign_id,
                        e.email_account_id,
                        v.days_of_week, v.start_hour, v.end_hour, v.timezone,
                        e.hourly_limit, e.daily_limit,
                        a.hourly_limit, a.daily_limit
                   FROM campaign_stage e
                   JOIN campaign        c ON c.id = e.campaign_id
                   JOIN email_account   a ON a.id = e.email_account_id
                   JOIN execution_window v ON v.id = e.execution_window_id
                  WHERE c.company_id = ?1
                    AND c.status = 'running'
                    AND e.status = 'running'
                    AND e.channel = 'email'
                  ORDER BY e.campaign_id, e.position",
            )?;

            let filas = consulta.query_map(params![empresa_txt], |f| {
                Ok(FilaDeEtapa {
                    etapa: f.get(0)?,
                    campana: f.get(1)?,
                    cuenta: f.get(2)?,
                    dias: f.get(3)?,
                    desde: f.get(4)?,
                    hasta: f.get(5)?,
                    zona: f.get(6)?,
                    hora_etapa: f.get(7)?,
                    dia_etapa: f.get(8)?,
                    hora_cuenta: f.get(9)?,
                    dia_cuenta: f.get(10)?,
                })
            })?;

            let mut etapas = Vec::new();
            for fila in filas {
                etapas.push(fila?.a_etapa(&zona_de_la_empresa)?);
            }
            Ok(etapas)
        })
    }
}

/// La fila cruda, antes de validarla.
///
/// Existe para que `a_etapa` pueda dar un error de dominio por campo: dentro
/// del `query_map` sólo caben errores de `rusqlite`, y decir «error de SQLite»
/// ante una zona horaria inventada mandaría a buscar el problema al disco.
struct FilaDeEtapa {
    etapa: String,
    campana: String,
    cuenta: String,
    dias: i64,
    desde: i64,
    hasta: i64,
    zona: String,
    hora_etapa: Option<i64>,
    dia_etapa: Option<i64>,
    hora_cuenta: i64,
    dia_cuenta: i64,
}

impl FilaDeEtapa {
    fn a_etapa(self, zona_de_la_empresa: &TimeZone) -> Result<EtapaEnMarcha, DbError> {
        let cuenta = self
            .cuenta
            .parse::<EmailAccountId>()
            .map_err(|_| DbError::DatoInvalido {
                motivo: "campaign_stage.email_account_id",
            })?;

        let dias = u8::try_from(self.dias)
            .map(Dias)
            .map_err(|_| DbError::DatoInvalido {
                motivo: "execution_window.days_of_week",
            })?;
        let desde = u8::try_from(self.desde).map_err(|_| DbError::DatoInvalido {
            motivo: "execution_window.start_hour",
        })?;
        let hasta = u8::try_from(self.hasta).map_err(|_| DbError::DatoInvalido {
            motivo: "execution_window.end_hour",
        })?;
        let zona_ventana = zona(&self.zona).map_err(|_| DbError::DatoInvalido {
            motivo: "execution_window.timezone",
        })?;

        let ventana = Ventana::nueva(dias, desde, hasta, zona_ventana).map_err(|_| {
            DbError::DatoInvalido {
                motivo: "execution_window",
            }
        })?;

        Ok(EtapaEnMarcha {
            etapa: self.etapa,
            campana: self.campana,
            cuenta,
            // La consulta filtra por `channel = 'email'`, así que no hay otro
            // caso que traducir. El día que haya adaptador de WhatsApp, esto y
            // el `WHERE` cambian juntos.
            canal: Canal::Correo,
            ventana,
            limites: Limites::nuevos(
                el_mas_bajo(self.hora_etapa, self.hora_cuenta),
                el_mas_bajo(self.dia_etapa, self.dia_cuenta),
            ),
            zona_de_la_empresa: zona_de_la_empresa.clone(),
        })
    }
}

/// El menor de los topes que haya. El de la cuenta no es opcional y siempre
/// entra en la comparación: ver la nota de la cabecera.
///
/// Lo comparte la revisión de una campaña (`revision.rs`), y se comparte en vez
/// de copiarse porque las dos respuestas tienen que ser la misma: una estima lo
/// que va a tardar y la otra decide cuánto sale de verdad. Con dos copias, una
/// podría cambiar sin la otra y la estimación mentiría sin que nada fallara.
///
/// Un valor negativo o desmesurado en la base se trata como «sin tope propio»
/// en vez de reventar: el de la cuenta sigue ahí, y es el que importa.
pub(crate) fn el_mas_bajo(etapa: Option<i64>, cuenta: i64) -> u32 {
    [etapa, Some(cuenta)]
        .into_iter()
        .flatten()
        .filter_map(|v| u32::try_from(v).ok())
        .min()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dos topes para el mismo envío: manda el más bajo, venga de donde venga.
    #[test]
    fn de_dos_topes_manda_el_mas_bajo() {
        assert_eq!(el_mas_bajo(Some(5), 50), 5, "el de la etapa");
        assert_eq!(
            el_mas_bajo(None, 50),
            50,
            "el de la cuenta, si la etapa no fija"
        );
    }

    /// **Y el de la cuenta no se puede superar aunque la etapa pida más.**
    ///
    /// Es el caso que no falla si se hace mal: los correos salen, y lo que se
    /// rompe es la reputación de la cuenta ante el proveedor.
    ///
    /// Verificada rompiéndola el 28/09/2026: dejando que la etapa gane cuando
    /// existe —`etapa.unwrap_or(cuenta)`—, esta prueba falla y la de arriba
    /// sigue pasando.
    #[test]
    fn la_etapa_no_puede_pedir_mas_de_lo_que_admite_la_cuenta() {
        assert_eq!(el_mas_bajo(Some(500), 50), 50);
    }

    /// Un número imposible en la base no revienta: queda el tope de la cuenta.
    #[test]
    fn un_tope_imposible_no_revienta_y_deja_el_de_la_cuenta() {
        assert_eq!(el_mas_bajo(Some(-1), 50), 50);
    }
}

#[cfg(test)]
mod contra_la_base {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
    const CAMPANA: &str = "camp-1";

    /// Siembra una empresa, una cuenta, una ventana y una campaña con su etapa,
    /// todo en marcha. Cada prueba tuerce después lo que quiere mirar.
    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit,
                                            created_at, updated_at)
                 VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                         'llavero://v', 'connected', 50, 10,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('vent-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign (id, company_id, name, status,
                                       created_at, updated_at)
                 VALUES ('{CAMPANA}', '{EMPRESA}', 'Prueba', 'running',
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('etapa-1', '{CAMPANA}', 1, 'email', '{CUENTA}', 'vent-1',
                         'running', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');"
            ))
        })
        .expect("siembra");

        (dir, db)
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id de empresa")
    }

    fn sql(db: &Db, orden: &str) {
        db.ejecutar_en_pruebas(|c| c.execute_batch(orden))
            .expect("orden de prueba");
    }

    /// Una etapa en marcha llega entera: campaña, cuenta, ventana y zonas.
    #[test]
    fn una_etapa_en_marcha_llega_con_todos_sus_ajustes() {
        let (_dir, db) = base();
        let etapas = db.etapas_en_marcha(empresa()).expect("consulta");

        assert_eq!(etapas.len(), 1, "hay una etapa enviando");
        let e = etapas.first().expect("la etapa sembrada");
        assert_eq!(e.campana, CAMPANA);
        assert_eq!(e.cuenta.to_string(), CUENTA);
        assert_eq!(e.canal, Canal::Correo);
        assert_eq!(e.ventana.desde, 9);
        assert_eq!(e.ventana.hasta, 18);
        assert_eq!(e.ventana.dias, Dias(31));
        // Sin topes propios de etapa ni de campaña, manda la cuenta.
        assert_eq!(e.limites, Limites::nuevos(10, 50));
    }

    /// **Una campaña pausada no envía, aunque su etapa siga en marcha.**
    #[test]
    fn una_campana_pausada_no_da_trabajo() {
        let (_dir, db) = base();
        sql(
            &db,
            &format!("UPDATE campaign SET status='paused' WHERE id='{CAMPANA}'"),
        );
        assert!(db.etapas_en_marcha(empresa()).expect("consulta").is_empty());
    }

    /// **Y una etapa pausada tampoco, aunque la campaña siga en marcha.**
    ///
    /// Es la garantía de la L-6: apagar un canal no apaga el otro. Mirando sólo
    /// el estado de la campaña, la pantalla diría «pausada» mientras los
    /// correos siguen saliendo.
    ///
    /// Verificada rompiéndola el 28/09/2026: quitando `AND e.status='running'`
    /// de la consulta, esta prueba falla y `una_campana_pausada_no_da_trabajo`
    /// sigue pasando — que es justo el par que hay que distinguir.
    #[test]
    fn una_etapa_pausada_no_da_trabajo_aunque_la_campana_siga() {
        let (_dir, db) = base();
        sql(
            &db,
            "UPDATE campaign_stage SET status='paused' WHERE id='etapa-1'",
        );
        assert!(db.etapas_en_marcha(empresa()).expect("consulta").is_empty());
    }

    /// **Una etapa de WhatsApp no llega aquí.**
    ///
    /// Hoy no hay adaptador de WhatsApp. Devolverla produciría un trabajador
    /// sin proveedor — o uno al que alguien le enchufara el de correo.
    #[test]
    fn una_etapa_de_whatsapp_no_se_devuelve() {
        let (_dir, db) = base();
        sql(
            &db,
            &format!(
                "INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                               waba_id, phone_number_id, credential_ref,
                                               status, created_at, updated_at)
                 VALUES ('wa-1', '{EMPRESA}', 'Ventas', '+525555555555',
                         'waba-1', 'pn-1', 'llavero://w', 'connected',
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 UPDATE campaign_stage SET channel='whatsapp', email_account_id=NULL,
                        whatsapp_account_id='wa-1' WHERE id='etapa-1';"
            ),
        );
        assert!(db.etapas_en_marcha(empresa()).expect("consulta").is_empty());
    }

    /// **Los dos topes, de punta a punta: manda el más bajo de cada uno.**
    ///
    /// Con la etapa pidiendo 3 por hora y 500 al día, y la cuenta admitiendo 10
    /// y 50: por hora gana la etapa, por día gana la cuenta. Un envío no puede
    /// heredar el tope alto de ninguno de los dos lados.
    #[test]
    fn el_tope_mas_bajo_de_cada_lado_es_el_que_llega() {
        let (_dir, db) = base();
        sql(
            &db,
            "UPDATE campaign_stage SET hourly_limit=3, daily_limit=500 WHERE id='etapa-1';",
        );
        let etapas = db.etapas_en_marcha(empresa()).expect("consulta");
        let e = etapas.first().expect("la etapa sembrada");
        assert_eq!(e.limites, Limites::nuevos(3, 50));
    }

    /// **Una ventana mal guardada detiene la lectura; no se salta la etapa.**
    ///
    /// Saltársela dejaría una campaña que pone «en marcha» en pantalla y que el
    /// motor ignora en silencio para siempre. Un error se ve; una campaña muda,
    /// no.
    ///
    /// Verificada rompiéndola el 28/09/2026: cambiando el bucle por un
    /// `filter_map` que descarta las filas que no validan, esta prueba falla y
    /// todas las demás siguen pasando.
    #[test]
    fn una_ventana_con_una_zona_que_no_existe_da_error_y_no_se_salta() {
        let (_dir, db) = base();
        sql(
            &db,
            "UPDATE execution_window SET timezone='Marte/Olympus' WHERE id='vent-1'",
        );

        let error = db
            .etapas_en_marcha(empresa())
            .expect_err("una ventana inválida no puede pasar desapercibida");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");
    }

    /// **Las etapas de una empresa no salen en la consulta de otra.**
    ///
    /// La primera versión de esta prueba estaba mal escrita: aceptaba un error
    /// **o** una lista vacía, con lo que pasaba dijera lo que dijera la
    /// consulta. Se deja escrito porque es el fallo del §2 en su forma más
    /// común — una comprobación que no puede fallar.
    ///
    /// Ahora se siembra una segunda empresa de verdad, con su propia zona, y se
    /// exige que su consulta devuelva cero etapas sin dar error.
    #[test]
    fn las_etapas_de_otra_empresa_no_aparecen() {
        let (_dir, db) = base();
        let otra = "22222222-2222-4222-8222-222222222222";
        sql(
            &db,
            &format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{otra}', 'OTRA', 'MX', 'America/Monterrey',
                         'hola@otra.mx', '2026-09-01T00:00:00Z',
                         '2026-09-01T00:00:00Z');"
            ),
        );

        let suyas = db
            .etapas_en_marcha(otra.parse().expect("id"))
            .expect("la consulta de una empresa sin campañas no es un error");
        assert!(
            suyas.is_empty(),
            "la etapa de una empresa no puede salir en la consulta de otra"
        );
        // Y la primera sigue viendo la suya: el filtro acota, no vacía.
        assert_eq!(db.etapas_en_marcha(empresa()).expect("consulta").len(), 1);
    }
}
