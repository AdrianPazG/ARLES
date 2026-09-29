//! Lo que hay que saber antes de activar una campaña de WhatsApp.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ HAY DOS CIFRAS QUE EN CORREO NO EXISTEN
//!
//! En correo, el preflight pregunta a cuánta gente se va a escribir. En
//! WhatsApp pregunta además **a cuánta de esa gente se le puede escribir
//! legítimamente**, y son dos números muy distintos:
//!
//! ```text
//!     847  contactos en la audiencia
//!   −   0  con permiso registrado
//!   ─────
//!       0  pueden recibir el mensaje legítimamente
//!     847  recibirían sin permiso
//! ```
//!
//! La diferencia importa porque el castigo es distinto. Un correo frío que
//! molesta acaba en la carpeta de no deseados; un WhatsApp frío que molesta
//! acaba con el número **inhabilitado**, y eso no se recupera — se pierde el
//! número, y con él la línea por la que la empresa atiende a sus clientes.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ CUENTA COMO PERMISO
//!
//! Una entrada en `consent_entry` de tipo `granted` para ese número, **sin una
//! retirada posterior**. Los cinco fundamentos de la V3 valen todos; lo que no
//! vale es suponerlo.
//!
//! Y el orden importa: se mira la última entrada de cada dirección, no si
//! existe alguna. Quien dio permiso en enero y lo retiró en marzo **no tiene
//! permiso**, y una consulta que sólo buscara `granted` diría que sí.

use arles_core::ids::{CampaignId, CompanyId};
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Db;
use crate::error::DbError;

/// Lo que la base sabe de una campaña de WhatsApp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoQueSabeDeWhatsApp {
    /// El número desde el que se enviaría.
    pub numero: String,
    /// Cuántos contactos hay con WhatsApp, tras aplicar las bajas.
    pub audiencia: u32,
    /// De ésos, a cuántos consta permiso vigente.
    pub con_permiso: u32,
    /// Y a cuántos **no**. Se guarda aparte en vez de restarlo en la pantalla:
    /// es la cifra que decide si el botón de activar cambia de color, y una
    /// resta hecha arriba es una resta que alguien puede hacer mal.
    pub sin_permiso: u32,
    /// ¿Está confirmada Coexistencia (L-11)?
    pub coexistencia: bool,
    /// ¿Se aceptó el riesgo del modo en frío?
    pub riesgo_aceptado: bool,
    /// ¿El modo de este número escribe en frío?
    pub escribe_en_frio: bool,
    /// Cuántas plantillas aprobadas hay para este número. **Sin al menos una,
    /// no hay con qué escribir**: Meta no acepta texto libre en una campaña.
    pub plantillas_aprobadas: u32,
}

impl LoQueSabeDeWhatsApp {
    /// ¿Se puede activar esta campaña?
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// ESCRIBIR SIN PERMISO **NO** LO IMPIDE, Y ESO ES DELIBERADO
    ///
    /// El §154 pide informar, no dictar, y la decisión de Dirección del
    /// 29/09/2026 fue construir los dos modos. Un número dado de alta en
    /// prospección directa, con su riesgo aceptado, puede escribir en frío:
    /// para eso se dio de alta así.
    ///
    /// Lo que ARLES hace es **que no se pueda hacer por inercia**: la cifra de
    /// quien no dio permiso se enseña siempre, y el botón de activar cambia de
    /// sitio y pide escribir una palabra.
    ///
    /// Lo que sí lo impide es lo que **no tiene arreglo escribiendo con
    /// cuidado**: sin Coexistencia nadie puede responder desde el móvil, sin
    /// plantilla aprobada no hay con qué escribir, y en frío sin riesgo
    /// aceptado la advertencia no consta.
    #[must_use]
    pub fn se_puede_activar(&self) -> bool {
        self.coexistencia
            && self.plantillas_aprobadas > 0
            && self.audiencia > 0
            && (!self.escribe_en_frio || self.riesgo_aceptado)
    }

    /// ¿Hace falta escribir una palabra para confirmar?
    ///
    /// Cuando alguien va a recibir un mensaje sin haber dado permiso. No es lo
    /// mismo que no poder activar: se puede, y hay que quererlo.
    #[must_use]
    pub const fn pide_palabra_de_confirmacion(&self) -> bool {
        self.sin_permiso > 0
    }
}

impl Db {
    /// Reúne lo que la base sabe de una campaña de WhatsApp.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si la campaña no existe, no es de esta empresa
    /// o no tiene etapa de WhatsApp. [`DbError::Sqlite`] si algo falla.
    pub fn lo_que_se_sabe_de_whatsapp(
        &self,
        empresa: CompanyId,
        campana: CampaignId,
    ) -> Result<LoQueSabeDeWhatsApp, DbError> {
        let id = campana.to_string();

        self.con_dominio(|conn| {
            let fila: Option<(String, String, Option<String>, Option<String>)> = conn
                .query_row(
                    "SELECT a.id, a.modo,
                            a.coexistencia_confirmada_at, a.riesgo_aceptado_at
                       FROM campaign c
                       JOIN campaign_stage e ON e.campaign_id = c.id AND e.channel = 'whatsapp'
                       JOIN whatsapp_account a ON a.id = e.whatsapp_account_id
                      WHERE c.id = ?1 AND c.company_id = ?2",
                    params![id, empresa.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?)),
                )
                .optional()?;

            let Some((numero, modo, coex, riesgo)) = fila else {
                return Err(DbError::DatoInvalido {
                    motivo: "esa campaña no tiene etapa de WhatsApp",
                });
            };

            let escribe_en_frio =
                arles_core::whatsapp::ModoDelCanal::de_guardado(&modo).map_err(DbError::Nucleo)?;

            let (audiencia, con_permiso) = contar_audiencia(conn, empresa)?;

            let plantillas: i64 = conn.query_row(
                "SELECT count(*) FROM whatsapp_template
                  WHERE whatsapp_account_id = ?1 AND status = 'approved'",
                params![numero],
                |f| f.get(0),
            )?;

            Ok(LoQueSabeDeWhatsApp {
                numero,
                audiencia,
                con_permiso,
                sin_permiso: audiencia.saturating_sub(con_permiso),
                coexistencia: coex.is_some(),
                riesgo_aceptado: riesgo.is_some(),
                escribe_en_frio: escribe_en_frio.escribe_en_frio(),
                plantillas_aprobadas: u32::try_from(plantillas).unwrap_or(u32::MAX),
            })
        })
    }
}

/// Cuánta gente hay, y de ésa, a cuánta consta permiso vigente.
///
/// Las dos cifras salen de **una sola consulta** a propósito: con dos, la
/// segunda podría contar sobre un conjunto distinto —otro filtro, otro
/// momento— y la resta daría un número que no significa nada. Aquí la resta
/// siempre cuadra porque el denominador es el mismo.
///
/// **El permiso es la ÚLTIMA entrada de cada dirección, no cualquiera.** Quien
/// dio permiso en enero y lo retiró en marzo no tiene permiso, y una consulta
/// que sólo buscara `granted` diría que sí.
fn contar_audiencia(conn: &Connection, empresa: CompanyId) -> Result<(u32, u32), DbError> {
    let (total, con): (i64, i64) = conn.query_row(
        "SELECT count(*),
                sum(CASE WHEN (
                    SELECT ce.kind FROM consent_entry ce
                     WHERE ce.company_id = ch.company_id
                       AND ce.channel = 'whatsapp'
                       AND ce.address_normalized = ch.value_normalized
                     ORDER BY ce.recorded_at DESC, ce.id DESC
                     LIMIT 1
                ) = 'granted' THEN 1 ELSE 0 END)
           FROM contact c
           JOIN contact_channel ch ON ch.contact_id = c.id
          WHERE c.company_id = ?1 AND c.status = 'active'
            AND c.deleted_at IS NULL
            AND ch.channel = 'whatsapp' AND ch.status = 'active'
            AND NOT EXISTS (SELECT 1 FROM suppression_entry s
                             WHERE s.company_id = ?1 AND s.channel = 'whatsapp'
                               AND s.address_normalized = ch.value_normalized)",
        params![empresa.to_string()],
        |f| Ok((f.get(0)?, f.get::<_, Option<i64>>(1)?.unwrap_or(0))),
    )?;

    Ok((
        u32::try_from(total).unwrap_or(u32::MAX),
        u32::try_from(con).unwrap_or(0),
    ))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const AHORA: &str = "2026-09-29T00:00:00Z";

    fn base(modo: &str) -> (tempfile::TempDir, Db, CampaignId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', '{AHORA}', '{AHORA}');

                 INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                               waba_id, phone_number_id, credential_ref,
                                               modo, status, created_at, updated_at)
                 VALUES ('01900000-0000-7000-8000-0000000000a1', '{EMPRESA}', 'Ventas', '+528100000000', 'w', 'p',
                         'llavero://w', '{modo}', 'connected', '{AHORA}', '{AHORA}');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('v-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '{AHORA}');

                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('01900000-0000-7000-8000-0000000000c1', '{EMPRESA}', 'WA',
                         'draft', '{AHORA}', '{AHORA}');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             whatsapp_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('et-1', '01900000-0000-7000-8000-0000000000c1', 1, 'whatsapp',
                         '01900000-0000-7000-8000-0000000000a1', 'v-1', 'draft', '{AHORA}', '{AHORA}');

                 INSERT INTO whatsapp_template (id, company_id, whatsapp_account_id,
                                                meta_name, language, status, body,
                                                slots, created_at, updated_at)
                 VALUES ('pl-1', '{EMPRESA}', '01900000-0000-7000-8000-0000000000a1', 'bienvenida', 'es_MX',
                         'approved', 'Hola', '', '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("siembra");

        let campana: CampaignId = "01900000-0000-7000-8000-0000000000c1".parse().expect("id");
        (dir, db, campana)
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    fn numero_id() -> arles_core::ids::WhatsappAccountId {
        "01900000-0000-7000-8000-0000000000a1".parse().expect("id")
    }

    /// Añade un contacto con WhatsApp.
    fn contacto(db: &Db, id: &str, movil: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('{id}', '{EMPRESA}', 'active', '{AHORA}', '{AHORA}');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-{id}', '{EMPRESA}', '{id}', 'whatsapp', '{movil}', '{movil}',
                         'active', '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("contacto");
    }

    /// Y una entrada de consentimiento, con su fecha.
    fn consentimiento(db: &Db, id: &str, movil: &str, tipo: &str, cuando: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO consent_entry (id, company_id, channel, address_normalized,
                                            kind, basis, recorded_at)
                 VALUES ('{id}', '{EMPRESA}', 'whatsapp', '{movil}', '{tipo}',
                         'form_optin', '{cuando}');"
            ))
        })
        .expect("consentimiento");
    }

    fn revisar(db: &Db, c: CampaignId) -> LoQueSabeDeWhatsApp {
        db.lo_que_se_sabe_de_whatsapp(empresa(), c).expect("revisa")
    }

    /// **Las dos cifras: a cuántos se escribe y a cuántos con permiso.**
    ///
    /// Es la diferencia que el preflight de WhatsApp tiene y el de correo no.
    /// Tres contactos, uno con permiso: se escribe a tres, y dos lo reciben sin
    /// haberlo pedido.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `total` también como
    /// `con_permiso`, esta prueba falla — y el aviso de campaña en frío
    /// desaparecería de la pantalla.
    #[test]
    fn se_cuenta_a_quien_dio_permiso_y_a_quien_no() {
        let (_d, db, c) = base("seguimiento");
        contacto(&db, "c1", "+528111111111");
        contacto(&db, "c2", "+528122222222");
        contacto(&db, "c3", "+528133333333");
        consentimiento(&db, "co-1", "+528111111111", "granted", AHORA);

        let r = revisar(&db, c);
        assert_eq!(r.audiencia, 3);
        assert_eq!(r.con_permiso, 1);
        assert_eq!(r.sin_permiso, 2);
        assert!(r.pide_palabra_de_confirmacion());
    }

    /// **Quien retiró el permiso no tiene permiso.**
    ///
    /// Es el caso que una consulta ingenua se come: buscar si existe una
    /// entrada `granted` diría que sí, aunque en marzo la persona dijera que
    /// no. `consent_entry` es de sólo-añadir, así que la retirada es una fila
    /// nueva y no un borrado — y hay que mirar **la última**.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `ORDER BY … DESC` de
    /// la subconsulta, se lee la primera entrada y esta prueba falla.
    #[test]
    fn quien_retiro_el_permiso_no_cuenta_como_que_lo_dio() {
        let (_d, db, c) = base("seguimiento");
        contacto(&db, "c1", "+528111111111");
        consentimiento(
            &db,
            "co-1",
            "+528111111111",
            "granted",
            "2026-01-01T00:00:00Z",
        );
        consentimiento(
            &db,
            "co-2",
            "+528111111111",
            "withdrawn",
            "2026-03-01T00:00:00Z",
        );

        let r = revisar(&db, c);
        assert_eq!(r.audiencia, 1);
        assert_eq!(r.con_permiso, 0, "lo retiró en marzo");
    }

    /// **Sin Coexistencia no se activa, aunque todo lo demás esté bien.**
    ///
    /// Sin ella el número sale de la app de WhatsApp Business: ARLES envía y
    /// nadie puede responder desde el móvil. Eso no se arregla escribiendo con
    /// cuidado, así que sí bloquea.
    #[test]
    fn sin_coexistencia_no_se_activa() {
        let (_d, db, c) = base("seguimiento");
        contacto(&db, "c1", "+528111111111");
        consentimiento(&db, "co-1", "+528111111111", "granted", AHORA);

        assert!(!revisar(&db, c).se_puede_activar(), "falta Coexistencia");

        db.confirmar_coexistencia(empresa(), numero_id(), "usuario_local")
            .expect("confirma");
        assert!(revisar(&db, c).se_puede_activar());
    }

    /// **Sin plantilla aprobada no hay con qué escribir.**
    ///
    /// Meta no acepta texto libre en una campaña: o hay una plantilla aprobada
    /// o no hay mensaje. Es distinto de los demás bloqueos —no es un riesgo,
    /// es una imposibilidad— y por eso también impide activar.
    #[test]
    fn sin_plantilla_aprobada_no_se_activa() {
        let (_d, db, c) = base("seguimiento");
        contacto(&db, "c1", "+528111111111");
        db.confirmar_coexistencia(empresa(), numero_id(), "usuario_local")
            .expect("confirma");
        assert!(revisar(&db, c).se_puede_activar());

        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch("UPDATE whatsapp_template SET status = 'pending';")
        })
        .expect("la deja en revisión");

        let r = revisar(&db, c);
        assert_eq!(r.plantillas_aprobadas, 0);
        assert!(!r.se_puede_activar());
    }

    /// **Escribir sin permiso no impide activar. Pide quererlo.**
    ///
    /// Es la decisión de Dirección del 29/09/2026 y el §154: informar, no
    /// dictar. Un número dado de alta en prospección directa, con su riesgo
    /// aceptado, puede escribir en frío — para eso se dio de alta así.
    ///
    /// Lo que ARLES impide es hacerlo **por inercia**: la cifra se enseña
    /// siempre y el botón pide escribir una palabra.
    ///
    /// Verificada rompiéndola el 29/09/2026: añadiendo `self.sin_permiso == 0`
    /// a `se_puede_activar`, esta prueba falla — y el modo en frío quedaría
    /// construido y sin poder usarse nunca.
    #[test]
    fn escribir_en_frio_se_puede_pero_no_por_inercia() {
        let (_d, db, c) = base("prospeccion_directa");
        contacto(&db, "c1", "+528111111111");
        let numero = numero_id();
        db.confirmar_coexistencia(empresa(), numero, "usuario_local")
            .expect("confirma");

        let sin_aceptar = revisar(&db, c);
        assert!(
            !sin_aceptar.se_puede_activar(),
            "en frío sin el riesgo aceptado, no"
        );

        db.aceptar_el_riesgo(empresa(), numero, "usuario_local")
            .expect("acepta");

        let r = revisar(&db, c);
        assert_eq!(r.sin_permiso, 1, "nadie dio permiso");
        assert!(r.se_puede_activar(), "y aun así se puede activar");
        assert!(
            r.pide_palabra_de_confirmacion(),
            "pero hay que escribir una palabra"
        );
    }

    /// Una audiencia vacía no se activa, igual que en correo.
    #[test]
    fn sin_nadie_a_quien_escribir_no_se_activa() {
        let (_d, db, c) = base("seguimiento");
        db.confirmar_coexistencia(empresa(), numero_id(), "usuario_local")
            .expect("confirma");

        let r = revisar(&db, c);
        assert_eq!(r.audiencia, 0);
        assert!(!r.se_puede_activar());
        assert!(
            !r.pide_palabra_de_confirmacion(),
            "sin nadie a quien escribir no hay a quien escribir sin permiso"
        );
    }

    /// Una campaña que sólo tiene etapa de correo no se revisa como WhatsApp.
    #[test]
    fn una_campana_sin_etapa_de_whatsapp_da_error() {
        let (_d, db, c) = base("seguimiento");
        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch("DELETE FROM campaign_stage WHERE id = 'et-1';")
        })
        .expect("quita la etapa");

        assert!(db.lo_que_se_sabe_de_whatsapp(empresa(), c).is_err());
    }
}
