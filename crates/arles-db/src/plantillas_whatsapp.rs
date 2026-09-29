//! Registrar las plantillas que Meta aprobó.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ NO SE ESCRIBE UNA PLANTILLA: SE ANOTA UNA QUE YA EXISTE
//!
//! Y la diferencia no es de matiz. Una plantilla de correo se escribe entera
//! dentro de ARLES. Una de WhatsApp **se crea en el panel de Meta, Meta la
//! revisa y la aprueba**, y lo único que ARLES puede hacer es apuntar cuál es:
//! su nombre, su idioma, qué dice y qué variable va en cada hueco.
//!
//! Por eso este módulo no valida el texto ni lo sanea: no es texto que vaya a
//! salir de aquí. Lo que sale por la API es `meta_name` — el texto lo pone
//! Meta, desde su copia. El `body` que se guarda es **para poder leerlo en
//! pantalla** sin pedírselo a la red en mitad del asistente.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE MÓDULO NO EXISTÍA, Y QUÉ SIGNIFICABA
//!
//! La tabla está en el esquema desde la V9, y **nadie escribía en ella** fuera
//! de las pruebas. Solo se leía, para contar cuántas plantillas aprobadas hay.
//!
//! Como la cuenta siempre daba cero, y la puerta del canal exige al menos una,
//! **ninguna campaña de WhatsApp se podía activar nunca**. El canal entero
//! estaba construido y probado, y su puerta estaba cerrada con llave desde
//! dentro.
//!
//! Es el tercer hueco del mismo tipo en tres entregas —una pieza correcta que
//! nadie usa—, y el primero en forma de **tabla sin escritor**. La comprobación
//! que lo caza está en la fase W del validador.

use arles_core::ids::{CompanyId, WhatsappAccountId, WhatsappTemplateId};
use arles_core::plantilla::Variable;
use arles_core::whatsapp::EstadoEnMeta;
use rusqlite::params;

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Lo que hace falta para anotar una plantilla de Meta.
#[derive(Debug, Clone)]
pub struct PlantillaDeMetaNueva<'a> {
    /// El `name` de Meta. **Es el identificador real del mensaje**: es lo que
    /// viaja por la API, no el texto.
    pub meta_name: &'a str,
    /// El idioma en la forma de Meta: `es_MX`.
    pub idioma: &'a str,
    /// Qué dice Meta de ella.
    pub estado: EstadoEnMeta,
    /// El texto aprobado, para poder enseñarlo.
    pub cuerpo: &'a str,
    /// Qué variable de ARLES va en cada hueco, en orden.
    pub huecos: &'a [Variable],
}

/// Una plantilla anotada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantillaDeMetaGuardada {
    pub id: String,
    pub numero: WhatsappAccountId,
    pub meta_name: String,
    pub idioma: String,
    pub estado: EstadoEnMeta,
    pub cuerpo: String,
    pub huecos: Vec<Variable>,
}

impl PlantillaDeMetaGuardada {
    /// ¿Se puede mandar una campaña con ésta?
    ///
    /// La regla vive en el núcleo y está escrita **por lo que acepta**, no por
    /// lo que rechaza: un estado nuevo de Meta no debe empezar a enviar solo.
    #[must_use]
    pub fn se_puede_usar(&self) -> bool {
        self.estado.se_puede_usar()
    }
}

impl Db {
    /// Anota una plantilla aprobada de Meta, o actualiza la que ya estaba.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// SE ACTUALIZA EN VEZ DE FALLAR, Y ES DELIBERADO
    ///
    /// Meta cambia el estado de una plantilla por su cuenta: una aprobada puede
    /// pasar a `paused` si la gente la reporta. Quien vuelva a anotarla está
    /// poniendo al día lo que Meta dice, no creando una segunda.
    ///
    /// La clave única de la V9 —número, nombre e idioma— es la misma que usa
    /// Meta, así que «la misma plantilla» significa lo mismo en los dos sitios.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si el nombre o el idioma vienen vacíos.
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn anotar_plantilla_de_meta(
        &self,
        empresa: CompanyId,
        numero: WhatsappAccountId,
        nueva: &PlantillaDeMetaNueva<'_>,
    ) -> Result<(), DbError> {
        if nueva.meta_name.trim().is_empty() {
            return Err(DbError::DatoInvalido {
                motivo: "la plantilla necesita el nombre que le puso Meta",
            });
        }
        if nueva.idioma.trim().is_empty() {
            return Err(DbError::DatoInvalido {
                motivo: "la plantilla necesita su idioma, en la forma de Meta",
            });
        }

        let huecos = nueva
            .huecos
            .iter()
            .map(|v| v.clave())
            .collect::<Vec<_>>()
            .join(",");

        self.con_dominio(|conn| {
            let cuando = ahora();
            conn.execute(
                "INSERT INTO whatsapp_template (id, company_id, whatsapp_account_id,
                                                meta_name, language, status, body,
                                                slots, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
                 ON CONFLICT (whatsapp_account_id, meta_name, language)
                 DO UPDATE SET status = excluded.status,
                               body = excluded.body,
                               slots = excluded.slots,
                               status_checked_at = excluded.updated_at,
                               updated_at = excluded.updated_at",
                params![
                    WhatsappTemplateId::nuevo().to_string(),
                    empresa.to_string(),
                    numero.to_string(),
                    nueva.meta_name.trim(),
                    nueva.idioma.trim(),
                    nueva.estado.como_se_guarda(),
                    nueva.cuerpo,
                    huecos,
                    cuando
                ],
            )?;
            Ok(())
        })
    }

    /// Las plantillas anotadas de un número.
    ///
    /// Devuelve **todas**, no solo las aprobadas: la pantalla necesita poder
    /// decir «ésta está en revisión» y no hacerla desaparecer, que es como se
    /// pierde media hora buscando algo que sí se dio de alta.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la lectura falla, o [`DbError::Nucleo`] si la base
    /// trae un estado o una variable que el núcleo no reconoce.
    pub fn plantillas_de_meta(
        &self,
        empresa: CompanyId,
        numero: WhatsappAccountId,
    ) -> Result<Vec<PlantillaDeMetaGuardada>, DbError> {
        self.con_dominio(|conn| {
            let mut consulta = conn.prepare(
                "SELECT id, meta_name, language, status, body, slots
                   FROM whatsapp_template
                  WHERE company_id = ?1 AND whatsapp_account_id = ?2
                  ORDER BY meta_name, language",
            )?;

            let filas =
                consulta.query_map(params![empresa.to_string(), numero.to_string()], |f| {
                    Ok((
                        f.get::<_, String>(0)?,
                        f.get::<_, String>(1)?,
                        f.get::<_, String>(2)?,
                        f.get::<_, String>(3)?,
                        f.get::<_, String>(4)?,
                        f.get::<_, String>(5)?,
                    ))
                })?;

            let mut todas = Vec::new();
            for fila in filas {
                let (id, meta_name, idioma, estado, cuerpo, huecos) = fila?;
                todas.push(PlantillaDeMetaGuardada {
                    id,
                    numero,
                    meta_name,
                    idioma,
                    estado: EstadoEnMeta::de_guardado(&estado).map_err(DbError::Nucleo)?,
                    cuerpo,
                    huecos: leer_huecos(&huecos)?,
                });
            }
            Ok(todas)
        })
    }
}

/// Lee la lista de huecos guardada como texto.
///
/// **Una variable que el núcleo no reconoce es un error, no un hueco que se
/// salta.** Saltarla correría todas las demás una posición: `{{2}}` recibiría
/// lo que va en `{{3}}`, y el mensaje saldría con los datos cambiados de sitio
/// sin que nada fallara.
fn leer_huecos(guardado: &str) -> Result<Vec<Variable>, DbError> {
    guardado
        .split(',')
        .filter(|v| !v.trim().is_empty())
        .map(|v| {
            Variable::de_clave(v.trim()).ok_or(DbError::DatoInvalido {
                motivo: "la plantilla trae una variable que el núcleo no reconoce",
            })
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const NUMERO: &str = "01900000-0000-7000-8000-0000000000a1";
    const AHORA: &str = "2026-09-29T00:00:00Z";

    fn base() -> (tempfile::TempDir, Db) {
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
                 VALUES ('{NUMERO}', '{EMPRESA}', 'Ventas', '+528100000000', 'w', 'p',
                         'llavero://w', 'seguimiento', 'connected', '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("siembra");

        (dir, db)
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    fn numero() -> WhatsappAccountId {
        NUMERO.parse().expect("id")
    }

    fn anotar(db: &Db, nombre: &str, estado: EstadoEnMeta, huecos: &[Variable]) {
        db.anotar_plantilla_de_meta(
            empresa(),
            numero(),
            &PlantillaDeMetaNueva {
                meta_name: nombre,
                idioma: "es_MX",
                estado,
                cuerpo: "Hola {{1}}",
                huecos,
            },
        )
        .expect("anota");
    }

    /// **Lo que se anota se puede volver a leer, con sus huecos en orden.**
    ///
    /// El orden es lo que decide qué dato va en `{{1}}` y cuál en `{{2}}`. Si se
    /// perdiera, el mensaje saldría con el nombre donde va la empresa.
    #[test]
    fn una_plantilla_anotada_se_lee_con_sus_huecos_en_orden() {
        let (_d, db) = base();
        anotar(
            &db,
            "bienvenida",
            EstadoEnMeta::Aprobada,
            &[Variable::Nombre, Variable::MiEmpresa],
        );

        let todas = db.plantillas_de_meta(empresa(), numero()).expect("lee");
        assert_eq!(todas.len(), 1);
        assert_eq!(todas[0].meta_name, "bienvenida");
        assert_eq!(todas[0].huecos, vec![Variable::Nombre, Variable::MiEmpresa]);
        assert!(todas[0].se_puede_usar());
    }

    /// **Anotar la misma plantilla dos veces la pone al día, no la duplica.**
    ///
    /// Meta cambia el estado por su cuenta: una aprobada puede pasar a `paused`
    /// si la reportan. Quien la vuelve a anotar está poniendo al día lo que Meta
    /// dice. Duplicarla dejaría el asistente ofreciendo la misma opción dos
    /// veces, con dos estados que podrían discrepar.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `ON CONFLICT … DO
    /// UPDATE`, la segunda llamada falla por la clave única y esta prueba cae.
    #[test]
    fn anotarla_otra_vez_la_pone_al_dia_y_no_la_duplica() {
        let (_d, db) = base();
        anotar(
            &db,
            "bienvenida",
            EstadoEnMeta::Aprobada,
            &[Variable::Nombre],
        );
        anotar(
            &db,
            "bienvenida",
            EstadoEnMeta::Pausada,
            &[Variable::Nombre],
        );

        let todas = db.plantillas_de_meta(empresa(), numero()).expect("lee");
        assert_eq!(todas.len(), 1, "sigue habiendo una");
        assert_eq!(
            todas[0].estado,
            EstadoEnMeta::Pausada,
            "con el estado nuevo"
        );
        assert!(!todas[0].se_puede_usar());
    }

    /// **Las que no están aprobadas se leen igual.**
    ///
    /// La pantalla tiene que poder decir «ésta está en revisión». Esconderlas es
    /// como se pierde media hora buscando algo que sí se dio de alta.
    #[test]
    fn una_plantilla_en_revision_no_desaparece_de_la_lista() {
        let (_d, db) = base();
        anotar(&db, "en-revision", EstadoEnMeta::EnRevision, &[]);

        let todas = db.plantillas_de_meta(empresa(), numero()).expect("lee");
        assert_eq!(todas.len(), 1);
        assert!(
            !todas[0].se_puede_usar(),
            "pero no se puede enviar con ella"
        );
    }

    /// **Una plantilla sin huecos es legítima.** Un mensaje fijo, sin variables.
    #[test]
    fn una_plantilla_sin_huecos_se_guarda_y_se_lee_vacia() {
        let (_d, db) = base();
        anotar(&db, "aviso", EstadoEnMeta::Aprobada, &[]);

        let todas = db.plantillas_de_meta(empresa(), numero()).expect("lee");
        assert!(todas[0].huecos.is_empty());
    }

    /// **Y ahora la puerta del canal deja pasar.**
    ///
    /// Es la prueba que cierra el hueco: antes del 29/09/2026 nadie escribía en
    /// esta tabla, así que `plantillas_aprobadas` era siempre 0 y
    /// `se_puede_activar()` devolvía siempre `false`. **Ninguna campaña de
    /// WhatsApp se podía activar nunca.**
    ///
    /// Se comprueba de punta a punta —anotar aquí, contar allí— porque el hueco
    /// vivía justo entre los dos módulos, y cada uno pasaba lo suyo.
    #[test]
    fn anotar_una_aprobada_desbloquea_la_puerta_del_canal() {
        let (_d, db) = base();

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO execution_window (id, company_id, name, days_of_week,
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
                         '{NUMERO}', 'v-1', 'draft', '{AHORA}', '{AHORA}');
                 INSERT INTO contact (id, company_id, status, created_at, updated_at)
                 VALUES ('c1', '{EMPRESA}', 'active', '{AHORA}', '{AHORA}');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-1', '{EMPRESA}', 'c1', 'whatsapp', '+528111111111',
                         '+528111111111', 'active', '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("siembra la campaña");

        db.confirmar_coexistencia(empresa(), numero(), "usuario_local")
            .expect("confirma");

        let campana = "01900000-0000-7000-8000-0000000000c1".parse().expect("id");

        let antes = db
            .lo_que_se_sabe_de_whatsapp(empresa(), campana)
            .expect("revisa");
        assert_eq!(antes.plantillas_aprobadas, 0);
        assert!(
            !antes.se_puede_activar(),
            "sin plantilla no se podía activar"
        );

        anotar(
            &db,
            "bienvenida",
            EstadoEnMeta::Aprobada,
            &[Variable::Nombre],
        );

        let despues = db
            .lo_que_se_sabe_de_whatsapp(empresa(), campana)
            .expect("revisa");
        assert_eq!(despues.plantillas_aprobadas, 1);
        assert!(despues.se_puede_activar(), "y ahora sí");
    }

    /// **Una plantilla en revisión no cuenta para la puerta.**
    ///
    /// El contrapeso: si contaran todas, anotar una que Meta todavía no ha
    /// aprobado dejaría activar una campaña que Meta va a rechazar entera.
    #[test]
    fn una_plantilla_en_revision_no_cuenta_como_aprobada() {
        let (_d, db) = base();
        anotar(&db, "en-revision", EstadoEnMeta::EnRevision, &[]);
        anotar(&db, "rechazada", EstadoEnMeta::Rechazada, &[]);

        let aprobadas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(*) FROM whatsapp_template WHERE status = 'approved'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");

        assert_eq!(aprobadas, 0);
    }

    /// Sin nombre de Meta no hay plantilla: es el identificador que viaja.
    #[test]
    fn sin_el_nombre_de_meta_no_se_anota() {
        let (_d, db) = base();
        let r = db.anotar_plantilla_de_meta(
            empresa(),
            numero(),
            &PlantillaDeMetaNueva {
                meta_name: "   ",
                idioma: "es_MX",
                estado: EstadoEnMeta::Aprobada,
                cuerpo: "Hola",
                huecos: &[],
            },
        );
        assert!(r.is_err());
    }
}
