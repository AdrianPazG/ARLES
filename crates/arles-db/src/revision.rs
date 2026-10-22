//! Lo que la base sabe de una campaña, para el preflight del §63.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ NO SE DECIDE NADA. SE MIRA.
//!
//! Quien decide si una campaña se puede activar es
//! `arles_core::preflight::revisar`, y lo hace sobre unas observaciones. Este
//! módulo reúne **las que están escritas en la base**, y no interpreta ninguna.
//!
//! La separación importa porque las reglas del preflight son de producto —qué
//! bloquea y qué sólo avisa, y por qué— y tienen que poder leerse y probarse
//! sin montar una base de datos. Lo que cambia entre una instalación y otra son
//! los datos, no las reglas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TRES DE LOS OCHO PUNTOS NO SE CONTESTAN DESDE AQUÍ, Y NO SE INVENTAN
//!
//! Si la cuenta autentica, si hay conexión y si el dominio tiene sus registros
//! exigen **hablar con alguien de fuera**, y `arles-db` no habla con nadie: no
//! conoce `arles-email` ni debe conocerlo.
//!
//! Por eso esto devuelve [`LoQueSabeLaBase`] y no un `Preflight` ya resuelto.
//! Quien complete lo que falta es `arles-app`, que sí tiene el adaptador
//! delante, y es quien llama a `revisar`.
//!
//! Rellenar esos tres con `Some(true)` aquí «para que pase» sería mentirle al
//! usuario sobre su propia campaña, y la mentira se descubriría en el primer
//! envío — con los correos ya saliendo.
//!
//! Hay una excepción, y está escrita: si el disyuntor marcó la cuenta como
//! `needs_reauth`, **eso sí consta en la base**. Lo resuelve
//! `SaludDeLaCuenta::credencial_para_el_preflight`, y decir «sin comprobar» de
//! algo que está escrito haría buscar el problema en otra parte.

use arles_core::duracion::{Estimacion, estimar};
use arles_core::ids::{CampaignId, CompanyId, EmailAccountId};
use arles_core::plantilla::{Plantilla, SiFalta, Valores, Variable};
use arles_core::ritmo::Limites;
use arles_core::ventana::{Dias, Ventana, zona};
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;
use crate::trabajo::el_limite_que_manda;

/// Los cinco puntos que se contestan sin salir del disco.
#[derive(Debug, Clone)]
pub struct LoQueSabeLaBase {
    /// La cuenta remitente de la etapa de correo. **Es a quien hay que
    /// sondear** para completar los tres puntos que faltan.
    pub cuenta: EmailAccountId,
    /// Lo que anota el preflight en `credencial_sirve`. `None` mientras no
    /// conste un fallo de credencial.
    pub credencial_sirve: Option<bool>,
    /// Cuánta gente queda **después** de aplicar las bajas.
    pub audiencia: u32,
    /// A cuántos contactos les falta alguna variable de la plantilla.
    ///
    /// Se guarda como cifra y no como sí/no: «a 34 de 1 300 les falta el
    /// nombre» deja decidir si compensa (§94). Un «hay variables sin resolver»
    /// obliga a revisar la lista entera a mano.
    pub contactos_con_huecos: u32,
    /// Cuánto va a tardar con lo que hay configurado. De aquí sale si el
    /// horario alcanza.
    pub estimacion: Estimacion,
    /// ¿Consta el envío de prueba del §45, hecho con **este** remitente?
    pub se_hizo_el_envio_de_prueba: bool,
    /// Las campañas que **ya están enviando** con este mismo remitente.
    ///
    /// ── POR QUÉ ESTO SALE DEL PREFLIGHT Y NO ES UN DETALLE ────────────────
    ///
    /// Dirección lo vio el 22/10/2026 antes que nadie: *«si coloco en la
    /// campaña uno un límite de 50 al día y otra campaña dos un límite de 50
    /// al día con el mismo remitente... no se mandan 50 en ambas»*.
    ///
    /// Tenía razón, y estaba medido: el presupuesto colgaba de la **cuenta**,
    /// así que dos campañas de 50 sobre un remitente sacaban **25 y 25**, y no
    /// por mitades sino por **orden de llegada**. Lo que rompía era la promesa
    /// de tiempo del paso 6.
    ///
    /// ── Y ESA MISMA TARDE DIRECCIÓN LO CAMBIÓ (D-13) ──────────────────
    ///
    /// El presupuesto pasó a colgar de la **etapa** y los topes **se suman**:
    /// las dos campañas sacan 50 cada una y el plazo se cumple. Lo que este
    /// campo avisa, entonces, ya no es que la estimación mienta — es que **de
    /// ese buzón van a salir 100 al día**, y lo que arriesga el dominio es el
    /// total y no cada campaña por separado.
    ///
    /// Sigue haciendo falta, y más que antes: antes el agregado tenía un techo
    /// y ahora no lo tiene. La cifra entera está en Remitentes con su semáforo
    /// (D-14); esto es lo que lo pone delante de quien está a punto de activar.
    ///
    /// Va vacío cuando el remitente es sólo de esta campaña.
    pub comparten_remitente: Vec<CampanaQueComparte>,
}

/// Una campaña que ya está enviando con el mismo remitente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampanaQueComparte {
    pub nombre: String,
    /// Su límite diario efectivo, por el mismo camino que usa el motor.
    pub limite_diario: u32,
}

impl Db {
    /// Reúne lo que la base sabe de una campaña.
    ///
    /// Se puede llamar todas las veces que haga falta: **no escribe nada**.
    ///
    /// `ahora_utc` es el instante desde el que se estima. Entra por parámetro y
    /// no se lee del reloj aquí para que las pruebas puedan situarse un martes
    /// a las nueve y no depender del día en que corran.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si la campaña no existe, no es de esta
    /// empresa, no tiene etapa de correo, o su etapa guarda datos que no se
    /// pueden leer —una ventana con una zona que ya no está en la base IANA,
    /// por ejemplo—.
    ///
    /// **No se aproxima**: una revisión que se inventara la ventana diría que
    /// el horario alcanza para una campaña que no va a salir nunca.
    ///
    /// [`DbError::Sqlite`] si alguna consulta falla.
    pub fn lo_que_se_sabe_de(
        &self,
        empresa: CompanyId,
        campana: CampaignId,
        ahora_utc: OffsetDateTime,
    ) -> Result<LoQueSabeLaBase, DbError> {
        let id = campana.to_string();
        let sello = ahora();

        self.con_dominio(|conn| {
            let fila = leer_la_etapa(conn, &id, empresa)?;

            let cuenta =
                fila.cuenta
                    .parse::<EmailAccountId>()
                    .map_err(|_| DbError::DatoInvalido {
                        motivo: "campaign_stage.email_account_id",
                    })?;

            let ventana = fila.ventana()?;

            let limites = fila.limites();

            // ── La audiencia, y de dónde sale ──
            //
            // Si la campaña ya está activada, la **congelada**; si no, la que
            // habría hoy. Son dos cosas distintas y la diferencia importa: una
            // campaña en borrador todavía no tiene audiencia congelada, y
            // contar cero diría que no hay a quién escribir cuando sí lo hay.
            //
            // Los contactos que se cuentan para las variables salen de la misma
            // lista, por la misma razón: dos criterios distintos darían «1 300
            // destinatarios, 4 000 sin nombre».
            let contactos = contactos_de_la_audiencia(conn, &id, empresa)?;
            let audiencia = u32::try_from(contactos.len()).unwrap_or(u32::MAX);
            let contactos_con_huecos = cuantos_tienen_huecos(&fila, &contactos);

            let credencial_sirve = crate::cuentas::salud_en(conn, &fila.cuenta, &sello)?
                .credencial_para_el_preflight();

            // ── El envío de prueba (§45) ──
            //
            // Vale sólo si se hizo **con el remitente que la etapa usa hoy**.
            // Ver la nota de `prueba.rs`: una marca que sobreviviera a cambiar
            // de cuenta daría luz verde basándose en una prueba de otra cosa.
            let se_hizo_el_envio_de_prueba = match (&fila.probada_el, &fila.probada_con) {
                (Some(_), Some(con)) => con == &fila.cuenta,
                _ => false,
            };

            // Las que ya envían con este remitente. Se excluye la propia:
            // una campaña ya en marcha que se vuelve a revisar no comparte
            // consigo misma, y decirlo asustaría sin motivo.
            let comparten_remitente =
                otras_campanas_del_remitente(conn, &fila.cuenta, &campana.to_string())?;

            Ok(LoQueSabeLaBase {
                cuenta,
                credencial_sirve,
                audiencia,
                contactos_con_huecos,
                estimacion: estimar(audiencia, limites, &ventana, ahora_utc),
                se_hizo_el_envio_de_prueba,
                comparten_remitente,
            })
        })
    }
}

/// Las campañas **en marcha** que envían con este mismo remitente.
///
/// `running` y sólo `running`, en la campaña **y** en la etapa: es el mismo
/// filtro que usa `etapas_en_marcha` para decidir a quién le toca enviar, y
/// tiene que serlo. Contar una pausada diría que el cupo se reparte cuando no
/// se reparte; dejar fuera una que sí envía es el error que no se ve.
///
/// El límite se resuelve con `el_limite_que_manda`, el mismo que aplica el
/// motor. Si aquí se leyera `daily_limit` a secas, una etapa sin límite propio
/// saldría con cero y el aviso diría que la otra campaña no consume nada.
fn otras_campanas_del_remitente(
    conn: &Connection,
    cuenta: &str,
    excepto: &str,
) -> Result<Vec<CampanaQueComparte>, DbError> {
    let mut consulta = conn.prepare(
        "SELECT c.name, e.daily_limit, a.daily_limit
           FROM campaign_stage e
           JOIN campaign c ON c.id = e.campaign_id
           JOIN email_account a ON a.id = e.email_account_id
          WHERE e.email_account_id = ?1
            AND e.campaign_id <> ?2
            AND e.channel = 'email'
            AND c.status = 'running'
            AND e.status = 'running'
          ORDER BY c.name",
    )?;
    let filas = consulta.query_map(params![cuenta, excepto], |f| {
        Ok(CampanaQueComparte {
            nombre: f.get(0)?,
            limite_diario: el_limite_que_manda(f.get(1)?, f.get(2)?),
        })
    })?;
    let mut fuera = Vec::new();
    for fila in filas {
        fuera.push(fila?);
    }
    Ok(fuera)
}

/// La etapa de correo de la campaña, cruda.
///
/// Existe como estructura para que los errores de dominio se den por campo
/// **fuera** del `query_row`: dentro sólo caben errores de `rusqlite`, y decir
/// «error de SQLite» ante una zona horaria inventada manda a buscar el problema
/// al disco. Es el mismo patrón que `FilaDeEtapa` en `trabajo.rs`.
pub(crate) struct FilaDeRevision {
    cuenta: String,
    dias: i64,
    desde: i64,
    hasta: i64,
    zona: String,
    hora_etapa: Option<i64>,
    dia_etapa: Option<i64>,
    hora_cuenta: i64,
    dia_cuenta: i64,
    probada_el: Option<String>,
    probada_con: Option<String>,
    /// El asunto y el cuerpo de la versión de plantilla enganchada, si la hay.
    asunto: Option<String>,
    cuerpo: Option<String>,
}

impl FilaDeRevision {
    /// Los topes con los que de verdad se va a enviar.
    ///
    /// **El más bajo de la etapa y la cuenta**, igual que en el motor: quien
    /// cuenta los correos es el proveedor, y pasarse de su límite no da un
    /// error claro — da una cuenta marcada como spam.
    ///
    /// Vive aquí y no en cada llamador desde la 8.6 E: lo necesitan el
    /// preflight y `activar_campana`, y dos copias del mismo «el más bajo de
    /// los dos» son dos sitios donde uno puede quedarse con el de la etapa y
    /// prometer un ritmo que el proveedor no acepta.
    pub(crate) fn limites(&self) -> Limites {
        Limites::nuevos(
            el_limite_que_manda(self.hora_etapa, self.hora_cuenta),
            el_limite_que_manda(self.dia_etapa, self.dia_cuenta),
        )
    }

    pub(crate) fn ventana(&self) -> Result<Ventana, DbError> {
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

        Ventana::nueva(dias, desde, hasta, zona_ventana).map_err(|_| DbError::DatoInvalido {
            motivo: "execution_window",
        })
    }
}

pub(crate) fn leer_la_etapa(
    conn: &Connection,
    campana: &str,
    empresa: CompanyId,
) -> Result<FilaDeRevision, DbError> {
    conn.query_row(
        "SELECT e.email_account_id,
                v.days_of_week, v.start_hour, v.end_hour, v.timezone,
                e.hourly_limit, e.daily_limit,
                a.hourly_limit, a.daily_limit,
                e.test_sent_at, e.test_sent_with_account_id,
                tv.subject, tv.body_text
           FROM campaign c
           JOIN campaign_stage e ON e.campaign_id = c.id AND e.channel = 'email'
           JOIN email_account   a ON a.id = e.email_account_id
           JOIN execution_window v ON v.id = e.execution_window_id
           LEFT JOIN template_version tv ON tv.id = e.template_version_id
          WHERE c.id = ?1 AND c.company_id = ?2",
        params![campana, empresa.to_string()],
        |f| {
            Ok(FilaDeRevision {
                cuenta: f.get(0)?,
                dias: f.get(1)?,
                desde: f.get(2)?,
                hasta: f.get(3)?,
                zona: f.get(4)?,
                hora_etapa: f.get(5)?,
                dia_etapa: f.get(6)?,
                hora_cuenta: f.get(7)?,
                dia_cuenta: f.get(8)?,
                probada_el: f.get(9)?,
                probada_con: f.get(10)?,
                asunto: f.get(11)?,
                cuerpo: f.get(12)?,
            })
        },
    )
    .optional()?
    .ok_or(DbError::DatoInvalido {
        motivo: "esa campaña no tiene etapa de correo",
    })
}

/// Lo que hace falta saber de cada destinatario para rellenar la plantilla.
///
/// La «empresa» del contacto vive en la columna `source`: es donde la escribe
/// `contactos.rs` desde la 3.2. El nombre viene del esquema V1 y no se ha
/// renombrado porque una migración que sólo renombra una columna en uso arregla
/// algo que nadie ve y arriesga algo que todo el mundo usa.
struct Destinatario {
    nombre: String,
    apellido: String,
    empresa: String,
}

/// Los contactos a los que va a llegar esto.
///
/// Si hay audiencia congelada, es **ésa** —una campaña activada ya no mira la
/// lista de contactos: mira la suya—. Si no, la que habría hoy, con las bajas
/// ya aplicadas.
fn contactos_de_la_audiencia(
    conn: &Connection,
    campana: &str,
    empresa: CompanyId,
) -> Result<Vec<Destinatario>, DbError> {
    let congelada: i64 = conn.query_row(
        "SELECT count(*) FROM campaign_audience
          WHERE campaign_id = ?1 AND channel = 'email'",
        params![campana],
        |f| f.get(0),
    )?;

    let (sql, parametro) = if congelada > 0 {
        (
            "SELECT COALESCE(c.first_name, ''), COALESCE(c.last_name, ''),
                    COALESCE(c.source, '')
               FROM campaign_audience au
               JOIN contact c ON c.id = au.contact_id
              WHERE au.campaign_id = ?1 AND au.channel = 'email'",
            campana.to_owned(),
        )
    } else {
        (
            "SELECT COALESCE(c.first_name, ''), COALESCE(c.last_name, ''),
                    COALESCE(c.source, '')
               FROM contact c
               JOIN contact_channel ch ON ch.contact_id = c.id
              WHERE c.company_id = ?1 AND c.status = 'active'
                AND c.deleted_at IS NULL
                AND ch.channel = 'email' AND ch.status = 'active'
                AND NOT EXISTS (SELECT 1 FROM suppression_entry s
                                 WHERE s.company_id = ?1 AND s.channel = 'email'
                                   AND s.address_normalized = ch.value_normalized)",
            empresa.to_string(),
        )
    };

    let mut consulta = conn.prepare(sql)?;
    let filas = consulta.query_map(params![parametro], |f| {
        Ok(Destinatario {
            nombre: f.get(0)?,
            apellido: f.get(1)?,
            empresa: f.get(2)?,
        })
    })?;

    let mut todos = Vec::new();
    for fila in filas {
        todos.push(fila?);
    }
    Ok(todos)
}

/// A cuántos contactos les falta alguna variable que la plantilla usa.
///
/// Se rellena de verdad, contacto a contacto, con el motor del núcleo, en vez
/// de contar columnas vacías en SQL. Es más lento y es lo correcto: lo que
/// importa no es qué columnas están vacías, sino **qué variables usa esta
/// plantilla concreta**, y eso sólo lo sabe el motor.
///
/// Sin plantilla enganchada devuelve 0, y no es un aprobado disimulado: una
/// etapa sin plantilla no tiene variables que dejar sin resolver. Lo que impide
/// activarla es otra cosa, y la 6.6 lo pone donde se ve.
fn cuantos_tienen_huecos(fila: &FilaDeRevision, contactos: &[Destinatario]) -> u32 {
    let (Some(asunto), Some(cuerpo)) = (&fila.asunto, &fila.cuerpo) else {
        return 0;
    };

    // `SiFalta::Nada` a propósito: lo que se cuenta es a quién le **falta** el
    // dato, y eso no cambia porque haya valor de reserva. Con el de la
    // plantilla, la cifra dependería de si alguien se molestó en escribir un
    // «cliente» de reserva, y entonces no contaría lo que dice contar.
    let del_asunto = Plantilla::nueva(asunto, SiFalta::Nada);
    let del_cuerpo = Plantilla::nueva(cuerpo, SiFalta::Nada);
    if del_asunto.variables().is_empty() && del_cuerpo.variables().is_empty() {
        return 0;
    }

    let mut con_huecos = 0u32;
    for c in contactos {
        let valores = Valores::nuevos()
            .con(Variable::Nombre, &c.nombre)
            .con(Variable::Apellido, &c.apellido)
            .con(Variable::Empresa, &c.empresa)
            // La empresa propia no falta nunca: es la que envía, y sin ella
            // configurada no se llega a crear una campaña. Se da por presente
            // para que no cuente como hueco de todos los contactos a la vez.
            .con(Variable::MiEmpresa, "la empresa");

        if !del_asunto.rellenar(&valores).faltaron.is_empty()
            || !del_cuerpo.rellenar(&valores).faltaron.is_empty()
        {
            con_huecos = con_huecos.saturating_add(1);
        }
    }
    con_huecos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alta_de_campana::CampanaNueva;
    use crate::conexion::ClaveMaestra;
    use crate::plantillas::Redaccion;
    use arles_core::ids::TemplateVersionId;
    use time::macros::datetime;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
    const OTRA_CUENTA: &str = "01900000-0000-7000-8000-00000000000b";

    /// Un martes a las nueve de la mañana, hora de Ciudad de México.
    ///
    /// Fijo y no `now_utc()`: una estimación que arranca en domingo da días
    /// distintos, y la prueba fallaría o pasaría según el día en que corriera.
    const MARTES: OffsetDateTime = datetime!(2026-09-29 15:00 UTC);

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    /// Base con empresa, dos cuentas, ventana y una plantilla que usa
    /// `{{nombre}}`.
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
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z'),
                        ('{OTRA_CUENTA}', '{EMPRESA}', 'Soporte', 'soporte@t.mx', 'smtp',
                         'llavero://s', 'connected', 50, 10,
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

    /// Un contacto activo con su correo. `nombre` vacío deja la columna nula.
    fn contacto(db: &Db, id: &str, correo: &str, nombre: &str) {
        let pila = if nombre.is_empty() {
            "NULL".to_owned()
        } else {
            format!("'{nombre}'")
        };
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT OR IGNORE INTO contact (id, company_id, first_name, status,
                                                created_at, updated_at)
                 VALUES ('{id}', '{EMPRESA}', {pila}, 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-{id}', '{EMPRESA}', '{id}', 'email', '{correo}', '{correo}',
                         'active', '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("contacto");
    }

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
                firma: None,
                ventana: "vent-1",
                limite_por_hora: None,
                limite_diario: None,
                // Sin tope propio no se pasa de 50: el de la cuenta manda.
                acepto_el_riesgo_del_volumen: false,
            },
        )
        .expect("crea")
        .id
    }

    fn etapa_de(db: &Db, campana: CampaignId) -> String {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT id FROM campaign_stage WHERE campaign_id = ?1",
                params![campana.to_string()],
                |f| f.get(0),
            )
        })
        .expect("etapa")
    }

    fn revisar(db: &Db, campana: CampaignId) -> LoQueSabeLaBase {
        db.lo_que_se_sabe_de(empresa(), campana, MARTES)
            .expect("revisa")
    }

    /// **La audiencia es la que queda después de las bajas, no el total.**
    ///
    /// Es el punto entero del preflight: una campaña puede tener tres mil
    /// contactos y cero destinatarios, y activarla sin saberlo congela una
    /// audiencia vacía que no se puede deshacer.
    ///
    /// Verificada rompiéndola el 29/09/2026: quitando el `NOT EXISTS` de la
    /// supresión en `contactos_de_la_audiencia`, esta prueba falla con
    /// «3 en vez de 2».
    #[test]
    fn la_audiencia_descuenta_las_bajas() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        contacto(&db, "c2", "dos@t.mx", "Beto");
        contacto(&db, "c3", "tres@t.mx", "Caro");
        suprimir(&db, "tres@t.mx");

        let c = una_campana(&db, v);
        assert_eq!(revisar(&db, c).audiencia, 2, "la dada de baja no cuenta");
    }

    /// **Una campaña activada mira su audiencia congelada, no la lista de hoy.**
    ///
    /// Si mirara la lista, el preflight de una campaña en marcha cambiaría cada
    /// vez que alguien importa contactos, y la cifra que se enseña dejaría de
    /// tener que ver con los correos que van a salir.
    ///
    /// Verificada rompiéndola el 29/09/2026: haciendo que
    /// `contactos_de_la_audiencia` use siempre la consulta de hoy, esta prueba
    /// falla con «3 en vez de 2».
    #[test]
    fn una_campana_activada_mira_su_audiencia_congelada() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        contacto(&db, "c2", "dos@t.mx", "Beto");

        let c = una_campana(&db, v);
        db.activar_campana(empresa(), c).expect("activa");

        // Llega alguien nuevo después de activar. No entra.
        contacto(&db, "c3", "tres@t.mx", "Caro");

        assert_eq!(revisar(&db, c).audiencia, 2, "la congelada, no la de hoy");
    }

    /// **Los huecos se cuentan por contacto, con la plantilla de esta campaña.**
    ///
    /// La cifra es lo que deja decidir: «a 1 de 3 le falta el nombre» se puede
    /// aceptar; «hay variables sin resolver» obliga a revisar la lista entera.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `0` sin mirar en
    /// `cuantos_tienen_huecos`, esta prueba falla con «0 en vez de 1».
    #[test]
    fn se_cuenta_a_cuantos_les_falta_una_variable() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        contacto(&db, "c2", "dos@t.mx", "Beto");
        contacto(&db, "c3", "tres@t.mx", ""); // sin nombre

        let c = una_campana(&db, v);
        let sabido = revisar(&db, c);
        assert_eq!(sabido.audiencia, 3);
        assert_eq!(
            sabido.contactos_con_huecos, 1,
            "sólo al que no tiene nombre"
        );
    }

    /// **Un espacio en blanco es un nombre que falta.**
    ///
    /// Viene de `Valores::con`, y se comprueba aquí porque el camino real pasa
    /// por la base: una celda con un espacio en un XLSX llega como texto no
    /// vacío y se guardaría tal cual. Contarla como nombre produciría «Hola  ,»
    /// en el correo, que es justo lo que la cifra existe para evitar.
    #[test]
    fn un_nombre_en_blanco_cuenta_como_hueco() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "   ");

        let c = una_campana(&db, v);
        assert_eq!(revisar(&db, c).contactos_con_huecos, 1);
    }

    /// **El envío de prueba hecho con otra cuenta no vale.**
    ///
    /// Es el caso que no falla si se hace mal: la marca está, el punto pasa, y
    /// la campaña se activa apoyada en una prueba que demostró que **otra**
    /// cuenta podía enviar.
    ///
    /// Verificada rompiéndola el 29/09/2026: comparando sólo
    /// `probada_el.is_some()`, esta prueba falla — el punto pasaría con la
    /// prueba de la otra cuenta.
    #[test]
    fn la_prueba_de_otra_cuenta_no_cuenta() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        let c = una_campana(&db, v);
        let etapa = etapa_de(&db, c);

        db.registrar_envio_de_prueba(&etapa, "yo@t.mx", OTRA_CUENTA.parse().expect("id"))
            .expect("registra");
        assert!(
            !revisar(&db, c).se_hizo_el_envio_de_prueba,
            "se probó con otra cuenta"
        );

        db.registrar_envio_de_prueba(&etapa, "yo@t.mx", CUENTA.parse().expect("id"))
            .expect("registra");
        assert!(
            revisar(&db, c).se_hizo_el_envio_de_prueba,
            "y con la suya sí vale"
        );
    }

    /// **Las campañas que comparten remitente se nombran, con su límite.**
    ///
    /// ── LO QUE DIRECCIÓN VIO ANTES QUE NADIE (22/10/2026) ─────────────────
    ///
    /// *«si coloco en la campaña uno un límite de 50 al día y otra campaña dos
    /// un límite de 50 al día con el mismo remitente... no se mandan 50 en
    /// ambas campañas»*. Medido entonces: **25 y 25**, por orden de llegada.
    ///
    /// Esa misma tarde Dirección decidió que los topes **se sumen** (D-13), y
    /// ahora salen 50 y 50. Así que lo que esta lista sostiene ya no es «el
    /// plazo miente» sino «el buzón va a mandar la suma», que es el riesgo que
    /// la D-13 abrió al quitar el techo del agregado.
    ///
    /// Esta prueba fija las tres cosas que el aviso necesita para no mentir:
    /// que sólo cuenten las que **de verdad están enviando**, que no se cuente
    /// a sí misma, y que el límite salga por el **mismo camino que el motor**.
    ///
    /// Verificada rompiéndola por los tres lados: quitando el filtro de
    /// `status = 'running'` aparece la pausada; quitando `e.campaign_id <> ?2`
    /// se nombra a sí misma; y leyendo `e.daily_limit` a secas en vez de
    /// `el_limite_que_manda`, la que no fija límite propio sale con 0.
    #[test]
    fn se_nombra_a_las_campanas_que_comparten_remitente() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");

        let mia = db
            .crear_campana(
                empresa(),
                &CampanaNueva {
                    nombre: "La mía",
                    cuenta: CUENTA.parse().expect("id"),
                    firma: None,
                    version_de_plantilla: v,
                    ventana: "vent-1",
                    limite_por_hora: None,
                    limite_diario: Some(50),
                    acepto_el_riesgo_del_volumen: false,
                },
            )
            .expect("crea")
            .id;

        // Sin nadie más enviando, el remitente es sólo suyo.
        assert!(
            revisar(&db, mia).comparten_remitente.is_empty(),
            "no hay con quién compartir todavía"
        );

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                // Una que SÍ envía, con su propio límite.
                "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-env', '{EMPRESA}', 'La de octubre', 'running',
                         '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');
                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, template_version_id,
                                             execution_window_id, daily_limit,
                                             status, created_at, updated_at)
                 VALUES ('st-env', 'k-env', 1, 'email', '{CUENTA}', '{v}', 'vent-1', 50,
                         'running', '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');

                 -- Una PAUSADA: no consume cupo, así que no se nombra.
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-pau', '{EMPRESA}', 'La pausada', 'paused',
                         '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');
                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, template_version_id,
                                             execution_window_id, daily_limit,
                                             status, created_at, updated_at)
                 VALUES ('st-pau', 'k-pau', 1, 'email', '{CUENTA}', '{v}', 'vent-1', 50,
                         'paused', '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');

                 -- Una de OTRO remitente: no comparte nada.
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-otr', '{EMPRESA}', 'La de soporte', 'running',
                         '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');
                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, template_version_id,
                                             execution_window_id, daily_limit,
                                             status, created_at, updated_at)
                 VALUES ('st-otr', 'k-otr', 1, 'email', '{OTRA_CUENTA}', '{v}', 'vent-1', 50,
                         'running', '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');

                 -- Y una que NO fija límite propio: hereda el de la cuenta.
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-her', '{EMPRESA}', 'La heredada', 'running',
                         '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');
                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, template_version_id,
                                             execution_window_id, daily_limit,
                                             status, created_at, updated_at)
                 VALUES ('st-her', 'k-her', 1, 'email', '{CUENTA}', '{v}', 'vent-1', NULL,
                         'running', '2026-10-22T00:00:00Z', '2026-10-22T00:00:00Z');"
            ))
        })
        .expect("siembra");

        let comparten = revisar(&db, mia).comparten_remitente;
        let nombres: Vec<&str> = comparten.iter().map(|c| c.nombre.as_str()).collect();
        assert_eq!(
            nombres,
            vec!["La de octubre", "La heredada"],
            "sólo las que de verdad están enviando con este remitente"
        );
        assert_eq!(
            comparten
                .iter()
                .map(|c| c.limite_diario)
                .collect::<Vec<_>>(),
            vec![50, 50],
            "la heredada tiene que traer el de la cuenta, no un cero"
        );
    }

    /// **La estimación usa el mismo límite que el envío, sea cual sea.**
    ///
    /// Lo que esta prueba protege no es un número concreto: es que **las dos
    /// respuestas no se separen**. Una estima lo que va a tardar y la otra
    /// decide cuánto sale de verdad; si divergieran, el paso 6 prometería
    /// terminar en un plazo que el motor no va a cumplir, y la diferencia se
    /// descubriría contando correos semanas después. Por eso las dos llaman a
    /// `el_limite_que_manda` y no a dos copias de la misma regla.
    ///
    /// **La cifra cambió el 22/10/2026** y la prueba con ella: con la cuenta a
    /// 50 y la etapa a 500, antes se esperaba 50 —mandaba el más bajo— y ahora
    /// se esperan 500, porque el límite de la campaña manda (ver la cabecera de
    /// `trabajo.rs`). Lo que no cambió es lo que se está comprobando.
    ///
    /// Verificada rompiéndola: haciendo que la estimación lea `self.dia_cuenta`
    /// directamente en vez de pasar por `el_limite_que_manda` —que es
    /// exactamente cómo se separarían las dos—, esta prueba cae con «50 en vez
    /// de 500».
    #[test]
    fn la_estimacion_usa_el_mismo_limite_que_el_envio() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");

        let c = db
            .crear_campana(
                empresa(),
                &CampanaNueva {
                    nombre: "Con prisa",
                    cuenta: CUENTA.parse().expect("id"),
                    version_de_plantilla: v,
                    firma: None,
                    ventana: "vent-1",
                    limite_por_hora: Some(500),
                    limite_diario: Some(500),
                    // 500 diarios pasan del aviso del §48, así que sin esto
                    // `crear_campana` la rechaza — y es lo que se quiere.
                    acepto_el_riesgo_del_volumen: true,
                },
            )
            .expect("crea")
            .id;

        assert_eq!(
            revisar(&db, c).estimacion.techo_diario,
            500,
            "la estimación tiene que decir lo mismo que va a salir"
        );
    }

    /// **La cifra que enseña el preflight es la que se congela al activar.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// AUDITORÍA DE LA FASE 6 · 29/09/2026
    ///
    /// Son **dos consultas distintas, escritas en dos entregas distintas**: la
    /// de `contactos_de_la_audiencia` (6.5) y la de `activar_campana` (6.4).
    /// Cada una tiene sus filtros —contacto activo, canal activo, sin baja— y
    /// nadie las había comparado.
    ///
    /// Si divergieran no fallaría nada: el paso 6 del asistente diría «1 342
    /// correos», alguien lo leería, decidiría con esa cifra, pulsaría activar,
    /// y se congelarían otros tantos. La diferencia se descubriría días
    /// después, contando correos, y para entonces ya salieron.
    ///
    /// Un contacto con **dos direcciones** recibe dos correos, y las dos
    /// cuentan: es lo que de verdad va a salir.
    #[test]
    fn lo_que_promete_el_preflight_es_lo_que_se_congela() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        contacto(&db, "c2", "dos@t.mx", "Beto");
        contacto(&db, "c3", "tres@t.mx", "Caro");
        suprimir(&db, "tres@t.mx");

        // Beto tiene una segunda dirección: son dos correos, no uno.
        //
        // Y una tercera, archivada. Está aquí porque **es el caso que hace que
        // esta prueba sirva de algo**: sin un canal archivado, las dos
        // consultas dan lo mismo aunque una se deje el filtro de `status`, y la
        // prueba pasaría sin comprobar nada. Se descubrió al romperla el
        // 29/09/2026 — quitar `ch.status = 'active'` de la revisión no la hacía
        // fallar. Con esta fila, sí.
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-c2b', '{EMPRESA}', 'c2', 'email', 'beto@otra.mx',
                         'beto@otra.mx', 'active',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z'),
                        ('ch-c2c', '{EMPRESA}', 'c2', 'email', 'beto@vieja.mx',
                         'beto@vieja.mx', 'archived',
                         '2026-09-29T00:00:00Z', '2026-09-29T00:00:00Z');"
            ))
        })
        .expect("las otras dos direcciones");

        let c = una_campana(&db, v);
        let prometido = revisar(&db, c).audiencia;
        let activacion = db.activar_campana(empresa(), c).expect("activa");

        assert_eq!(prometido, 3, "Ana, y Beto por partida doble");
        assert_eq!(
            activacion.en_la_audiencia, prometido,
            "la cifra que se leyó antes de pulsar es la que salió"
        );
        assert_eq!(activacion.fuera_por_baja, 1, "Caro pidió no recibir");
    }

    /// **Un tope de cero no deja una campaña colgada en silencio.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// AUDITORÍA DE LA FASE 6 · 29/09/2026
    ///
    /// El asistente deja escribir el tope diario a mano, así que un `0` puede
    /// llegar hasta aquí. Y un tope de cero **no da error en ninguna parte**:
    /// la campaña se crea, se activa, el motor la mira cada cinco segundos, no
    /// le toca enviar nada, y vuelve a mirar. Para siempre, con la pantalla
    /// diciendo «enviando».
    ///
    /// Lo que lo caza no es una validación del formulario —que se salta por
    /// cualquier otro camino— sino la cadena entera: el tope es 0, el techo
    /// diario es 0, la estimación dice que no avanza, y el punto «el horario
    /// alcanza» del preflight falla y bloquea la activación.
    ///
    /// Esta prueba existe porque **esa cadena nunca se había visto funcionar
    /// entera**, y son cuatro piezas de cuatro entregas distintas.
    ///
    /// ── EL CERO ENTRA AHORA POR OTRA PUERTA, Y POR ESO SIGUE HACIENDO FALTA ─
    ///
    /// Desde el 22/10/2026 `crear_campana` rechaza un cero con
    /// `LimiteDeEnviosEnCero`, así que esta prueba **ya no puede sembrarlo
    /// creando la campaña**: lo escribe en la etapa después, que es el camino
    /// que queda abierto —una base de antes de esa guarda, o una fila tocada a
    /// mano—.
    ///
    /// Haberla borrado al añadir la guarda habría sido el error: una
    /// validación de entrada no protege a las filas que ya estaban dentro, y
    /// ésta es la red que las caza.
    #[test]
    fn un_tope_de_cero_lo_caza_la_estimacion() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");

        let c = db
            .crear_campana(
                empresa(),
                &CampanaNueva {
                    nombre: "Parada",
                    cuenta: CUENTA.parse().expect("id"),
                    version_de_plantilla: v,
                    firma: None,
                    ventana: "vent-1",
                    limite_por_hora: Some(1),
                    limite_diario: Some(1),
                    acepto_el_riesgo_del_volumen: false,
                },
            )
            .expect("crea")
            .id;

        // El cero se siembra en la etapa, que es por donde puede llegar hoy.
        db.ejecutar_en_pruebas(|conn| {
            conn.execute(
                "UPDATE campaign_stage SET hourly_limit = 0, daily_limit = 0
                  WHERE campaign_id = ?1",
                rusqlite::params![c.to_string()],
            )
        })
        .expect("siembra el cero");

        let sabido = revisar(&db, c);
        assert_eq!(sabido.estimacion.techo_diario, 0, "no cabe ni uno");
        assert!(
            !matches!(
                sabido.estimacion.desenlace,
                arles_core::duracion::Desenlace::Termina { .. }
            ),
            "y por eso el punto del horario del preflight no pasa"
        );
    }

    /// **Una ventana que ya no se puede leer detiene la revisión.**
    ///
    /// Aproximarla —tomar UTC, o los días de la semana enteros— daría un
    /// preflight que dice «el horario alcanza» para una campaña que no va a
    /// enviar nunca.
    ///
    /// Verificada rompiéndola el 29/09/2026: cayendo a `TimeZone::UTC` cuando
    /// `zona()` falla, esta prueba falla — la revisión devuelve `Ok`.
    #[test]
    fn una_zona_que_ya_no_existe_detiene_la_revision() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        let c = una_campana(&db, v);

        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch(
                "UPDATE execution_window SET timezone = 'America/Atlantida'
                  WHERE id = 'vent-1';",
            )
        })
        .expect("estropea");

        let error = db
            .lo_que_se_sabe_de(empresa(), c, MARTES)
            .err()
            .unwrap_or_else(|| panic!("una zona inventada tiene que doler"));
        assert!(matches!(
            error,
            DbError::DatoInvalido {
                motivo: "execution_window.timezone"
            }
        ));
    }

    /// **Una credencial que consta caducada no se enseña como «sin comprobar».**
    ///
    /// Es la conexión entre el disyuntor y el preflight: la respuesta está
    /// escrita en la base desde la última vez que falló, y decir «no he podido
    /// mirar» mandaría a buscar el problema a otra parte.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `None` fijo en vez de
    /// llamar a `credencial_para_el_preflight`, esta prueba falla.
    #[test]
    fn la_credencial_caducada_consta_y_se_dice() {
        let (_d, db, v) = base();
        contacto(&db, "c1", "uno@t.mx", "Ana");
        let c = una_campana(&db, v);

        assert_eq!(
            revisar(&db, c).credencial_sirve,
            None,
            "con la cuenta sana no se puede afirmar nada todavía"
        );

        db.ejecutar_en_pruebas(|conn| {
            conn.execute_batch(&format!(
                "UPDATE email_account SET status = 'needs_reauth' WHERE id = '{CUENTA}';"
            ))
        })
        .expect("marca");

        assert_eq!(
            revisar(&db, c).credencial_sirve,
            Some(false),
            "esto sí consta"
        );
    }

    /// **Una campaña de otra empresa no se revisa.**
    ///
    /// El aislamiento por empresa no falla solo: sin el filtro, la revisión
    /// devuelve cifras de contactos ajenos y nada lo delata.
    #[test]
    fn una_campana_de_otra_empresa_no_se_revisa() {
        let (_d, db, v) = base();
        let c = una_campana(&db, v);

        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");
        assert!(db.lo_que_se_sabe_de(otra, c, MARTES).is_err());
    }
}
