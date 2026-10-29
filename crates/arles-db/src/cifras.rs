//! Las cifras de la pantalla de Actividad.
//!
//! Entrega 7.2.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UN SOLO SITIO QUE LAS CALCULA, Y EL TOTAL ES UNA SUMA
//!
//! Todo va **por campaña**, porque Dirección trabaja con hasta tres a la vez.
//! El total de la empresa no es una consulta aparte: es la suma de las
//! campañas, y hay una prueba que lo exige.
//!
//! No es purismo. Con dos consultas distintas —una por campaña y otra
//! general— llega el día en que discrepan, y entonces la pantalla enseña dos
//! números que no cuadran y **nadie sabe cuál creer**. Con una suma, si el
//! detalle está mal el total está igual de mal, y eso se nota y se arregla.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! DE DÓNDE SALE CADA NÚMERO
//!
//! De `message_attempt`, siempre. **No de `activity_event`**, que es la
//! narración (7.1) y puede perder una línea si el programa muere en el
//! instante justo. La prueba de que se escribió a alguien es el intento, y las
//! cifras se calculan sobre la prueba.
//!
//! La agrupación de los nueve estados en cinco grupos vive en
//! `arles_core::attempt::GrupoDeDesenlace`, no aquí: la necesitan la tarjeta
//! de campaña, el filtro del registro y el resumen exportable, y con tres
//! copias la que se olvidara de cambiar enseñaría otra cifra sin fallar.

use std::collections::HashMap;

use arles_core::attempt::{AttemptState, GrupoDeDesenlace};
use arles_core::ids::CompanyId;
use rusqlite::params;

use crate::db::Db;
use crate::error::DbError;

/// Quién es la campaña: lo justo para poder nombrarla sin volver a buscarla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentidadDeCampana {
    pub id: String,
    pub nombre: String,
    /// El estado **crudo de la base**. Lo traduce quien lo enseña, con
    /// `EstadoDeCampana::clave_i18n` — aquí no se interpreta nada.
    pub estado: String,
}

/// Cómo va una campaña.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResumenDeCampana {
    /// Cuánta gente tiene congelada. Cero mientras es borrador.
    pub audiencia: u32,
    /// Cuántos salieron. **No dice «entregados»** (§65).
    pub salio: u32,
    /// Cuántos quedan por intentar, incluidos los que se reintentarán.
    pub pendiente: u32,
    /// Cuántos no salieron y no saldrán.
    pub fallo: u32,
    /// Cuántos estaban en la lista de bajas. No es un fallo.
    pub dado_de_baja: u32,
    /// Cuántos se cancelaron al detener. Tampoco es un fallo.
    pub cancelado: u32,
    /// Cuánta gente contestó (D-7A). Sólo WhatsApp, y sólo lo que haya.
    pub respondieron: u32,
}

impl ResumenDeCampana {
    /// Cuántos intentos hay en total.
    ///
    /// **Puede no coincidir con `audiencia`**, y es correcto: la audiencia se
    /// congela al activar, y los intentos se crean a partir de ella. Entre las
    /// dos cosas, una supresión puede dejar a alguien fuera.
    #[must_use]
    pub const fn intentos(&self) -> u32 {
        self.salio + self.pendiente + self.fallo + self.dado_de_baja + self.cancelado
    }

    /// Suma otra campaña a ésta. Es como se calcula el total de la empresa.
    pub const fn sumar(&mut self, otra: &Self) {
        self.audiencia += otra.audiencia;
        self.salio += otra.salio;
        self.pendiente += otra.pendiente;
        self.fallo += otra.fallo;
        self.dado_de_baja += otra.dado_de_baja;
        self.cancelado += otra.cancelado;
        self.respondieron += otra.respondieron;
    }
}

/// Lo que queda de cupo en una cuenta, y **quién compite por él**.
///
/// ─────────────────────────────────────────────────────────────────────────
/// EL PROBLEMA QUE ESTO EXISTE PARA CONTAR
///
/// Tres campañas pueden compartir la misma cuenta de correo, y **el cupo es de
/// la cuenta, no de la campaña**. Si la de agosto se come 480 de 500, las
/// otras dos se paran —el motor hace exactamente lo que debe— y en la pantalla
/// se ven dos campañas quietas sin ninguna explicación.
///
/// Va a pasar el primer día que se lancen tres campañas con un remitente. Por
/// eso `campanas` no es un adorno: sin los nombres, la cifra «quedan 20 de
/// 500» no explica por qué se paró la campaña que el usuario está mirando.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CupoDeCuenta {
    pub cuenta: String,
    pub nombre: String,
    pub direccion: String,
    /// Lo gastado hoy por este buzón, **sumando todas sus campañas**.
    pub gastado_hoy: u32,
    /// Cuántos correos al día va a mandar este buzón: **la suma de los límites
    /// de las campañas en marcha que lo usan** (D-13, 22/10/2026).
    ///
    /// Hasta el 22/10 era el límite de la cuenta y era un techo compartido: dos
    /// campañas de 100 mandaban 100 entre las dos. Ahora cada una lleva su
    /// cuenta y se suman, así que dos campañas de 100 son **200 al día de este
    /// buzón** — y ésa es la cifra que hay que poder ver antes de activar nada,
    /// porque ya no hay ningún techo que la frene.
    ///
    /// `None` cuando ninguna campaña en marcha usa este buzón: no manda nada,
    /// que no es lo mismo que mandar cero por decisión.
    pub tope_diario: Option<u32>,
    pub gastado_esta_hora: u32,
    /// Lo mismo por hora.
    pub tope_por_hora: Option<u32>,
    /// Los nombres de las campañas en marcha que tiran de esta cuenta.
    pub campanas: Vec<String>,
}

impl CupoDeCuenta {
    /// Cuántos quedan hoy, si hay tope.
    #[must_use]
    pub fn quedan_hoy(&self) -> Option<u32> {
        self.tope_diario.map(|t| t.saturating_sub(self.gastado_hoy))
    }

    /// ¿Más de una campaña tira de esta cuenta?
    ///
    /// Es lo que decide si la pantalla tiene que explicar el reparto o puede
    /// callarse: con una sola campaña, nombrarla es ruido.
    #[must_use]
    pub fn hay_reparto(&self) -> bool {
        self.campanas.len() > 1
    }
}

impl Db {
    /// Cómo va una campaña, por grupos.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si alguna consulta falla.
    pub fn resumen_de_campana(
        &self,
        empresa: CompanyId,
        campana: &str,
    ) -> Result<ResumenDeCampana, DbError> {
        self.leer(|conn| {
            let audiencia = conn.query_row(
                "SELECT count(*) FROM campaign_audience a
                  JOIN campaign c ON c.id = a.campaign_id
                 WHERE a.campaign_id = ?1 AND c.company_id = ?2",
                params![campana, empresa.to_string()],
                |f| f.get(0),
            )?;

            let mut r = ResumenDeCampana {
                audiencia,
                ..ResumenDeCampana::default()
            };

            // Los nueve estados en una sola consulta, y se agrupan en Rust con
            // `GrupoDeDesenlace`. Agruparlos en SQL con un `CASE` sería una
            // segunda copia de la misma decisión, en otro lenguaje, y el día
            // que cambie una de las dos nadie se enteraría.
            let mut q = conn.prepare(
                "SELECT m.state, count(*)
                   FROM message_attempt m
                   JOIN campaign c ON c.id = m.campaign_id
                  WHERE m.campaign_id = ?1 AND c.company_id = ?2
                  GROUP BY m.state",
            )?;

            for fila in q.query_map(params![campana, empresa.to_string()], |f| {
                Ok((f.get::<_, String>(0)?, f.get::<_, u32>(1)?))
            })? {
                let (estado, cuantos) = fila?;
                // Un estado que el núcleo no conoce **no se reparte al azar ni
                // se suma a «pendiente»**: se ignora y se dice en la prueba de
                // invariantes. Sumarlo a un grupo cualquiera sería inventar.
                let Some(grupo) = de_texto(&estado).map(AttemptState::grupo) else {
                    continue;
                };
                match grupo {
                    GrupoDeDesenlace::Salio => r.salio += cuantos,
                    GrupoDeDesenlace::Pendiente => r.pendiente += cuantos,
                    GrupoDeDesenlace::Fallo => r.fallo += cuantos,
                    GrupoDeDesenlace::DadoDeBaja => r.dado_de_baja += cuantos,
                    GrupoDeDesenlace::Cancelado => r.cancelado += cuantos,
                }
            }

            // Las respuestas (D-7A): la cifra, nunca el contenido. Se suman las
            // de todas sus etapas — una campaña puede tener correo y WhatsApp.
            r.respondieron = conn.query_row(
                "SELECT count(*) FROM whatsapp_reply w
                  JOIN campaign_stage e ON e.id = w.campaign_stage_id
                 WHERE e.campaign_id = ?1 AND w.company_id = ?2",
                params![campana, empresa.to_string()],
                |f| f.get(0),
            )?;

            Ok(r)
        })
    }

    /// El total de la empresa: **la suma de sus campañas**, no otra consulta.
    ///
    /// Devuelve el resumen de cada campaña y el total, para que la pantalla no
    /// tenga que sumar por su cuenta y arriesgarse a sumar distinto.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ CADA FILA TRAE SU NOMBRE Y SU ESTADO
    ///
    /// La primera versión devolvía sólo el id, y el comando cruzaba esa lista
    /// con `campanas()` para ponerle nombre — con un `filter_map`, que
    /// **descarta en silencio** lo que no encuentra.
    ///
    /// Hoy las dos consultas leen lo mismo, así que no se pierde nada. Pero el
    /// día que `campanas()` filtre algo —las archivadas, por ejemplo—,
    /// Actividad dejaría de enseñar esas campañas **sin que nada fallara**, y
    /// el total seguiría incluyéndolas: dos números que no cuadran, otra vez.
    ///
    /// Con el nombre aquí no hay cruce que pueda fallar.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si alguna consulta falla.
    /// Los fallos de una campaña, **agrupados por su causa**.
    ///
    /// ───────────────────────────────────────────────────────────────────────
    /// POR QUÉ SALE DE `message_attempt` Y NO DEL REGISTRO
    ///
    /// Porque el registro de actividad es derivado y se puede recortar (D-7C),
    /// y los intentos son la prueba. Un resumen que contara los fallos desde
    /// `activity_event` daría cifras distintas antes y después del recorte
    /// bimestral, sobre una campaña que no cambió.
    ///
    /// Se cuentan los estados **definitivos**: `permanently_failed` y
    /// `suppressed`. Un `failed` a secas va a volver a intentarse, así que
    /// contarlo como fallo diría que se perdió algo que todavía está en camino.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    /// Cuántos rebotes duros lleva esta campaña, y de cuántos envíos.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LAS DOS DECISIONES DE ESTA CONSULTA, Y LAS DOS SE PUEDEN HACER MAL
    ///
    /// **1 · Qué es un rebote duro.** Sólo `permanently_failed` con la causa
    /// de dirección inexistente. No vale contar todos los `permanently_failed`
    /// — ahí entran las credenciales caducadas y los intentos agotados, que no
    /// dicen nada de la lista y harían frenar campañas por un problema del
    /// remitente.
    ///
    /// **2 · De cuántos.** De los **ya resueltos**: salió o rebotó. No entra
    /// la audiencia entera, ni lo que sigue en cola, ni los `suppressed`.
    ///
    /// Los `suppressed` fuera es la parte sutil: esa gente estaba en la lista
    /// de bajas y **nunca se le llamó**, así que meterlos en el denominador
    /// diluiría el porcentaje con envíos que no ocurrieron. Una campaña con
    /// mil bajas y cincuenta rebotes de cincuenta envíos daría un 5 % mal
    /// contado —parecería el 4.8 % de 1 050— y el freno no saltaría.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn rebotes_duros_de(
        &self,
        empresa: CompanyId,
        campana: &str,
    ) -> Result<(u32, u32), DbError> {
        self.con(|conn| {
            let fila = conn.query_row(
                "SELECT
                    COALESCE(SUM(CASE
                        WHEN a.state = 'permanently_failed'
                         AND a.error_kind = 'reintento.direccionInexistente'
                        THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE
                        WHEN a.state IN ('sent', 'presumed_sent', 'permanently_failed')
                        THEN 1 ELSE 0 END), 0)
                   FROM message_attempt a
                   JOIN campaign c ON c.id = a.campaign_id
                  WHERE c.company_id = ?1
                    AND a.campaign_id = ?2",
                params![empresa.to_string(), campana],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
            )?;
            Ok((
                u32::try_from(fila.0).unwrap_or(u32::MAX),
                u32::try_from(fila.1).unwrap_or(u32::MAX),
            ))
        })
    }

    pub fn fallos_por_causa(
        &self,
        empresa: CompanyId,
        campana: &str,
    ) -> Result<Vec<(String, u32)>, DbError> {
        self.con(|conn| {
            let mut q = conn.prepare(
                "SELECT COALESCE(a.error_kind, ''), count(*) AS n
                   FROM message_attempt a
                   JOIN campaign c ON c.id = a.campaign_id
                  WHERE c.company_id = ?1
                    AND a.campaign_id = ?2
                    AND a.state IN ('permanently_failed', 'suppressed')
                  GROUP BY a.error_kind
                  ORDER BY n DESC, a.error_kind",
            )?;
            let filas = q
                .query_map(params![empresa.to_string(), campana], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(filas)
        })
    }

    pub fn resumen_de_la_empresa(
        &self,
        empresa: CompanyId,
    ) -> Result<
        (
            Vec<(IdentidadDeCampana, ResumenDeCampana)>,
            ResumenDeCampana,
        ),
        DbError,
    > {
        let cuales: Vec<IdentidadDeCampana> = self.con(|conn| {
            let mut q = conn.prepare(
                "SELECT id, name, status FROM campaign
                  WHERE company_id = ?1
                  ORDER BY created_at DESC",
            )?;
            let v = q
                .query_map(params![empresa.to_string()], |f| {
                    Ok(IdentidadDeCampana {
                        id: f.get(0)?,
                        nombre: f.get(1)?,
                        estado: f.get(2)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(v)
        })?;

        let mut por_campana = Vec::with_capacity(cuales.len());
        let mut total = ResumenDeCampana::default();
        for cual in cuales {
            let r = self.resumen_de_campana(empresa, &cual.id)?;
            total.sumar(&r);
            por_campana.push((cual, r));
        }
        Ok((por_campana, total))
    }

    /// El cupo de cada remitente, y qué campañas compiten por él.
    ///
    /// Ver [`CupoDeCuenta`]: es lo que explica por qué dos campañas están
    /// quietas cuando una tercera se comió el cupo.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si alguna consulta falla.
    pub fn cupo_de_los_remitentes(
        &self,
        empresa: CompanyId,
        clave_de_hora: &str,
        clave_de_dia: &str,
    ) -> Result<Vec<CupoDeCuenta>, DbError> {
        self.con(|conn| {
            // Qué campañas en marcha usan cada cuenta. Se lee de una vez y no
            // por cuenta: con tres campañas y dos cuentas serían seis
            // consultas para responder algo que cabe en una.
            let mut q = conn.prepare(
                "SELECT e.email_account_id, c.name
                   FROM campaign_stage e
                   JOIN campaign c ON c.id = e.campaign_id
                  WHERE c.company_id = ?1
                    AND c.status = 'running'
                    AND e.email_account_id IS NOT NULL
                  ORDER BY c.name",
            )?;
            let mut tiran: HashMap<String, Vec<String>> = HashMap::new();
            for fila in q.query_map(params![empresa.to_string()], |f| {
                Ok((f.get::<_, String>(0)?, f.get::<_, String>(1)?))
            })? {
                let (cuenta, nombre) = fila?;
                let suyas = tiran.entry(cuenta).or_default();
                // Una campaña con dos etapas sobre la misma cuenta se nombraría
                // dos veces, y «Agosto, Agosto y Reactivación» se lee como un
                // defecto de la pantalla.
                if !suyas.contains(&nombre) {
                    suyas.push(nombre);
                }
            }

            // ── `SUM` Y NO `SELECT consumed`, DESDE LA V13 ────────────────
            //
            // El cubo pasó a colgar de la etapa (D-13), así que una cuenta con
            // tres campañas tiene TRES filas en `rate_budget` por ventana. El
            // `SELECT consumed` que había aquí devolvía **la primera que
            // encontrara**: con 80, 60 y 40 enviados habría dicho «80 de 250»
            // en la pantalla del semáforo, sin un error y sin una fila de más.
            // Es el defecto más silencioso que podía dejar este cambio, porque
            // el número es plausible.
            //
            // Y el TOPE también se suma: es lo que Dirección pidió ver —«que se
            // vea en realidad cuántos correos está enviando por día»—, y es la
            // suma de los límites de las etapas en marcha, con el de la cuenta
            // como caída cuando la etapa no fija ninguno. `a.daily_limit` solo
            // ya no es el tope de nada: desde el 22/10 es la propuesta que el
            // asistente enseña al crear la campaña.
            //
            // `MAX(…, 0)` sobre cada término porque un límite negativo guardado
            // en la base cuenta como «no fija» en `el_limite_que_manda`, y
            // sumarlo en crudo restaría del total del buzón.
            let mut cuentas = conn.prepare(
                "SELECT a.id, a.display_name, a.email_address,
                        (SELECT SUM(MAX(COALESCE(e.daily_limit, a.daily_limit, 0), 0))
                           FROM campaign_stage e
                           JOIN campaign c ON c.id = e.campaign_id
                          WHERE e.email_account_id = a.id
                            AND c.status = 'running'
                            AND e.status = 'running'
                            AND e.channel = 'email'),
                        (SELECT SUM(MAX(COALESCE(e.hourly_limit, a.hourly_limit, 0), 0))
                           FROM campaign_stage e
                           JOIN campaign c ON c.id = e.campaign_id
                          WHERE e.email_account_id = a.id
                            AND c.status = 'running'
                            AND e.status = 'running'
                            AND e.channel = 'email'),
                        (SELECT SUM(b.consumed) FROM rate_budget b
                          WHERE b.email_account_id = a.id
                            AND b.window_kind = 'daily'
                            AND b.window_start >= ?2),
                        (SELECT SUM(b.consumed) FROM rate_budget b
                          WHERE b.email_account_id = a.id
                            AND b.window_kind = 'hourly'
                            AND b.window_start >= ?3)
                   FROM email_account a
                  WHERE a.company_id = ?1
                  ORDER BY a.display_name",
            )?;

            let filas = cuentas.query_map(
                params![empresa.to_string(), clave_de_dia, clave_de_hora],
                |f| {
                    Ok(CupoDeCuenta {
                        cuenta: f.get(0)?,
                        nombre: f.get(1)?,
                        direccion: f.get(2)?,
                        tope_diario: f.get(3)?,
                        tope_por_hora: f.get(4)?,
                        gastado_hoy: f.get::<_, Option<u32>>(5)?.unwrap_or(0),
                        gastado_esta_hora: f.get::<_, Option<u32>>(6)?.unwrap_or(0),
                        // Un buzón sin ninguna campaña en marcha no manda
                        // nada, y las dos sumas de arriba vienen en NULL. Eso
                        // es `None` —«no hay tope porque no hay nada»— y no
                        // cero, que se leería como «parado a propósito».
                        campanas: Vec::new(),
                    })
                },
            )?;

            let mut v = Vec::new();
            for fila in filas {
                let mut c = fila?;
                c.campanas = tiran.remove(&c.cuenta).unwrap_or_default();
                v.push(c);
            }
            Ok(v)
        })
    }

    /// Cuántos salieron en cada una de las últimas horas.
    ///
    /// Es lo que sostiene la estimación de término, y por eso devuelve las
    /// horas y no un promedio ya calculado: **un número sin su método es una
    /// promesa**, y la pantalla tiene que poder decir «al ritmo de las últimas
    /// 6 h» con los datos a la vista.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn ritmo_por_hora(
        &self,
        empresa: CompanyId,
        campana: &str,
        desde: &str,
    ) -> Result<Vec<(String, u32)>, DbError> {
        self.con(|conn| {
            let mut q = conn.prepare(
                // `substr(sent_at, 1, 13)` recorta a «2026-09-30T10», que es la
                // hora. Las marcas se guardan en UTC con formato fijo, así que
                // el recorte es estable; `strftime` haría lo mismo más despacio
                // y con una dependencia del formato igual de fuerte.
                "SELECT substr(m.sent_at, 1, 13) AS hora, count(*)
                   FROM message_attempt m
                   JOIN campaign c ON c.id = m.campaign_id
                  WHERE m.campaign_id = ?1
                    AND c.company_id = ?2
                    AND m.state = 'sent'
                    AND m.sent_at IS NOT NULL
                    AND m.sent_at >= ?3
                  GROUP BY hora
                  ORDER BY hora",
            )?;
            let v = q
                .query_map(params![campana, empresa.to_string(), desde], |f| {
                    Ok((f.get::<_, String>(0)?, f.get::<_, u32>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(v)
        })
    }
}

/// El texto guardado, de vuelta al estado del núcleo.
///
/// Devuelve `None` para un estado que el núcleo no conoce, en vez de adivinar.
/// Es la única forma de que una fila con un valor imposible —de una migración
/// futura mal aplicada, por ejemplo— no acabe sumada al grupo equivocado.
fn de_texto(estado: &str) -> Option<AttemptState> {
    AttemptState::TODOS
        .iter()
        .copied()
        .find(|e: &AttemptState| e.as_str() == estado)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const AHORA: &str = "2026-09-30T10:00:00Z";
    const DIA: &str = "2026-09-30";
    const HORA: &str = "2026-09-30T10";

    /// Tres campañas y dos cuentas, con **una cuenta compartida**.
    ///
    /// No es decorado: es el escenario de Dirección, y es el único en el que
    /// aparecen los dos defectos que estas pruebas buscan — un `GROUP BY` al
    /// que le falte una columna, y el reparto de cupo que deja dos campañas
    /// quietas sin explicación.
    ///
    /// Con una sola campaña y una sola cuenta, **las dos cosas pasan igual
    /// estando mal**. Es la misma trampa que la prueba de contactos con un
    /// solo contacto (§2).
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

                 INSERT INTO email_account (id, company_id, display_name, email_address,
                                            provider_kind, credential_ref, status,
                                            daily_limit, hourly_limit, created_at, updated_at)
                 VALUES ('c-1', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                         'llavero://v', 'connected', 500, 60, '{AHORA}', '{AHORA}'),
                        ('c-2', '{EMPRESA}', 'Soporte', 'soporte@t.mx', 'smtp',
                         'llavero://s', 'connected', 100, 20, '{AHORA}', '{AHORA}');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('v-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '{AHORA}');

                 -- Tres campañas: dos en marcha sobre la MISMA cuenta, una aparte.
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-1', '{EMPRESA}', 'Clientes de agosto', 'running',
                         '2026-09-30T09:00:00Z', '{AHORA}'),
                        ('k-2', '{EMPRESA}', 'Reactivación', 'running',
                         '2026-09-30T08:00:00Z', '{AHORA}'),
                        ('k-3', '{EMPRESA}', 'Bienvenida', 'running',
                         '2026-09-30T07:00:00Z', '{AHORA}'),
                        -- Detenida, y sobre una cuenta que sí se usa. Sin ella,
                        -- la prueba del reparto pasaba igual al quitar el filtro
                        -- por `running` — afirmaba algo que no verificaba, que
                        -- es el defecto que el §2 existe para cazar. Se vio
                        -- rompiéndola el 30/09/2026.
                        ('k-4', '{EMPRESA}', 'Verano (detenida)', 'stopped',
                         '2026-09-30T06:00:00Z', '{AHORA}');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('e-1', 'k-1', 1, 'email', 'c-1', 'v-1', 'running', '{AHORA}', '{AHORA}'),
                        ('e-2', 'k-2', 1, 'email', 'c-1', 'v-1', 'running', '{AHORA}', '{AHORA}'),
                        ('e-3', 'k-3', 1, 'email', 'c-2', 'v-1', 'running', '{AHORA}', '{AHORA}'),
                        ('e-4', 'k-4', 1, 'email', 'c-2', 'v-1', 'stopped', '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("siembra");
        (dir, db)
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    /// Mete un intento en el estado que se pida.
    fn intento(db: &Db, campana: &str, n: u32, estado: &str, cuando: Option<&str>) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO message_attempt
                     (id, campaign_id, contact_address, idempotency_key, state,
                      channel, sent_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'email', ?6, ?7, ?7)",
                params![
                    format!("m-{campana}-{n}"),
                    campana,
                    format!("p{n}@t.mx"),
                    format!("k-{campana}-{n}"),
                    estado,
                    cuando,
                    AHORA
                ],
            )
            .map(|_| ())
        })
        .expect("intento");
    }

    /// **Los nueve estados, uno de cada, y cada grupo cuenta lo suyo.**
    ///
    /// Con un solo intento esta prueba pasaría con casi cualquier agregación.
    /// Con los nueve, un `GROUP BY` mal escrito o un grupo olvidado falla.
    ///
    /// Verificada rompiéndola: haciendo que `PresumedSent` caiga en
    /// `Pendiente`, falla con «salio» a 1 en vez de 2.
    #[test]
    fn los_nueve_estados_caen_cada_uno_en_su_grupo() {
        let (_d, db) = base();

        for (n, e) in AttemptState::TODOS.iter().enumerate() {
            let n = u32::try_from(n).expect("cabe");
            intento(&db, "k-1", n, e.as_str(), Some(AHORA));
        }

        let r = db.resumen_de_campana(empresa(), "k-1").expect("resumen");

        // queued, claimed, sending, failed
        assert_eq!(
            r.pendiente, 4,
            "pendientes: los cuatro que aún se pueden intentar"
        );
        // sent, presumed_sent
        assert_eq!(r.salio, 2, "salió: incluye el envío ambiguo del ADR-0004");
        assert_eq!(r.fallo, 1, "falló: sólo el definitivo");
        assert_eq!(r.dado_de_baja, 1);
        assert_eq!(r.cancelado, 1);
        assert_eq!(
            r.intentos(),
            9,
            "los nueve tienen que estar contados en algún grupo: si uno no \
             cuadra, hay un estado sin sitio"
        );
    }

    /// **El total es la suma de las campañas, no otra consulta.**
    ///
    /// Es la garantía que impide que la pantalla enseñe dos números que no
    /// cuadran sin que nadie sepa cuál creer.
    ///
    /// Verificada rompiéndola: haciendo que `resumen_de_la_empresa` cuente con
    /// un `SELECT` propio sobre `message_attempt` sin filtrar por campaña, las
    /// dos cifras dejan de coincidir y esta prueba lo dice.
    #[test]
    fn el_total_de_la_empresa_es_la_suma_de_sus_campanas() {
        let (_d, db) = base();

        intento(&db, "k-1", 1, "sent", Some(AHORA));
        intento(&db, "k-1", 2, "sent", Some(AHORA));
        intento(&db, "k-2", 1, "sent", Some(AHORA));
        intento(&db, "k-2", 2, "permanently_failed", None);
        intento(&db, "k-3", 1, "suppressed", None);

        let (por_campana, total) = db.resumen_de_la_empresa(empresa()).expect("resumen");

        assert_eq!(
            por_campana.len(),
            4,
            "las cuatro campañas: el resumen las cuenta todas, también las \
             detenidas, porque sus envíos ocurrieron"
        );

        let sumado = por_campana
            .iter()
            .fold(ResumenDeCampana::default(), |mut acc, (_, r)| {
                acc.sumar(r);
                acc
            });
        assert_eq!(
            total, sumado,
            "el total no coincide con la suma de las campañas: hay dos formas \
             de calcular lo mismo y ya discrepan"
        );
        assert_eq!(total.salio, 3);
        assert_eq!(total.fallo, 1);
        assert_eq!(total.dado_de_baja, 1);
    }

    /// **Dos campañas que comparten cuenta se ven compartiéndola.**
    ///
    /// Es el problema que va a salir el primer día: el cupo es de la cuenta, y
    /// si una campaña se lo come las otras se paran sin explicación visible.
    ///
    /// Verificada rompiéndola: quitando `AND c.status = 'running'` de la
    /// consulta de campañas, la cuenta de Soporte aparece con campañas que no
    /// están enviando y el reparto que se enseña es falso.
    #[test]
    fn el_cupo_dice_que_campanas_compiten_por_cada_cuenta() {
        let (_d, db) = base();

        let cupos = db
            .cupo_de_los_remitentes(empresa(), HORA, DIA)
            .expect("cupos");

        assert_eq!(cupos.len(), 2, "las dos cuentas");

        let ventas = cupos.iter().find(|c| c.cuenta == "c-1").expect("ventas");
        assert_eq!(
            ventas.campanas,
            vec!["Clientes de agosto".to_owned(), "Reactivación".to_owned()],
            "las dos campañas que tiran de Ventas, por nombre: sin ellos, «quedan \
             20 de 500» no explica por qué se paró la que el usuario mira"
        );
        assert!(ventas.hay_reparto());

        let soporte = cupos.iter().find(|c| c.cuenta == "c-2").expect("soporte");
        assert_eq!(
            soporte.campanas,
            vec!["Bienvenida".to_owned()],
            "«Verano» está detenida y usa esta cuenta: no compite por su cupo, \
             y enseñarla haría creer que el reparto es peor de lo que es"
        );
        assert!(
            !soporte.hay_reparto(),
            "con una sola campaña, nombrarla es ruido"
        );
    }

    /// El cupo gastado se lee de verdad, y lo que queda se calcula.
    ///
    /// ── ESTA PRUEBA DECÍA `Some(500)` HASTA LA D-13 ──────────────────────
    ///
    /// Y el cambio de cifra **es** la decisión del 22/10/2026, así que vale la
    /// pena que quede escrito aquí y no sólo en la cabecera del presupuesto.
    ///
    /// La base de pruebas tiene dos campañas en marcha —`e-1` y `e-2`— sobre el
    /// mismo buzón `c-1`, y ninguna de las dos fija límite propio, así que las
    /// dos caen al de la cuenta: 500. Antes eso era **un techo compartido** y el
    /// buzón mandaba 500 entre las dos. Ahora cada una lleva su recuento y
    /// **se suman: 1 000 al día**.
    ///
    /// Es la cifra que el semáforo de Remitentes (D-14) tiene que enseñar en
    /// rojo, porque ya no hay nada que la frene.
    #[test]
    fn el_cupo_gastado_sale_del_presupuesto_persistido() {
        let (_d, db) = base();

        for _ in 0..480 {
            db.anotar_un_envio("c-1", "e-1", HORA, DIA, AHORA)
                .expect("gasta");
        }

        let cupos = db
            .cupo_de_los_remitentes(empresa(), HORA, DIA)
            .expect("cupos");
        let ventas = cupos.iter().find(|c| c.cuenta == "c-1").expect("ventas");

        assert_eq!(ventas.gastado_hoy, 480);
        assert_eq!(
            ventas.tope_diario,
            Some(1_000),
            "dos campañas de 500 sobre un buzón son 1 000 al día desde la D-13, \
             no 500 compartidos"
        );
        assert_eq!(
            ventas.quedan_hoy(),
            Some(520),
            "y lo que queda es del buzón entero: es la cifra que explica por qué \
             una campaña se va a parar"
        );
    }

    /// **Lo gastado por un buzón SUMA sus campañas, y esto es lo que una
    /// `SELECT consumed` habría dejado pasar.**
    ///
    /// Desde la V13 hay una fila de `rate_budget` por etapa, así que la cifra
    /// del buzón es una suma. La consulta que había aquí leía **la primera fila
    /// que encontrara**: con 300 enviados por una campaña y 100 por la otra
    /// habría dicho «300 de 1 000» — un número plausible, sin error, sin una
    /// fila de más y sin ninguna prueba en rojo.
    ///
    /// Por eso las dos campañas gastan cantidades **distintas**: con 200 y 200,
    /// un `SELECT consumed` daría 200 y la suma 400, y ninguna de las dos se
    /// podría confundir con la otra por casualidad. Con 100 y 100 un error de
    /// «leer una y doblarla» pasaría en verde.
    #[test]
    fn lo_gastado_por_un_buzon_suma_sus_campanas() {
        let (_d, db) = base();

        for _ in 0..300 {
            db.anotar_un_envio("c-1", "e-1", HORA, DIA, AHORA)
                .expect("gasta la primera");
        }
        for _ in 0..100 {
            db.anotar_un_envio("c-1", "e-2", HORA, DIA, AHORA)
                .expect("gasta la segunda");
        }

        let cupos = db
            .cupo_de_los_remitentes(empresa(), HORA, DIA)
            .expect("cupos");
        let ventas = cupos.iter().find(|c| c.cuenta == "c-1").expect("ventas");

        assert_eq!(
            ventas.gastado_hoy, 400,
            "300 de una campaña y 100 de la otra son 400 del buzón"
        );
        assert_eq!(
            ventas.gastado_esta_hora, 400,
            "y la hora se suma igual, que es la ventana que primero se llena"
        );
    }

    /// **Y cada campaña respeta SU número, que es el motivo de la D-13.**
    ///
    /// Sumar los topes no bastaba: con un cubo único de 1 000, la campaña que el
    /// motor atendiera primero podía gastárselo entero y dejar a la otra en
    /// cero. El total cuadraría y el plazo que el paso 6 prometió a la segunda
    /// seguiría siendo falso — que es el problema que esta decisión resuelve,
    /// no el número agregado.
    ///
    /// Así que lo que hay que poder ver es que los dos recuentos **no se
    /// contaminan**: 300 en una campaña dejan la otra a cero.
    #[test]
    fn una_campana_no_gasta_del_recuento_de_la_otra() {
        let (_d, db) = base();

        for _ in 0..300 {
            db.anotar_un_envio("c-1", "e-1", HORA, DIA, AHORA)
                .expect("gasta");
        }

        assert_eq!(
            db.gastado_de("c-1", "e-1", HORA, DIA)
                .expect("lee")
                .en_este_dia,
            300
        );
        assert_eq!(
            db.gastado_de("c-1", "e-2", HORA, DIA)
                .expect("lee")
                .en_este_dia,
            0,
            "la segunda campaña no ha gastado nada, aunque comparta el buzón"
        );
    }

    /// **El ritmo va por horas, no como un promedio ya masticado.**
    ///
    /// Un número sin su método es una promesa: la pantalla tiene que poder
    /// decir «al ritmo de las últimas 6 h» con los datos delante.
    #[test]
    fn el_ritmo_se_agrupa_por_hora_y_solo_cuenta_lo_que_salio() {
        let (_d, db) = base();

        intento(&db, "k-1", 1, "sent", Some("2026-09-30T09:15:00Z"));
        intento(&db, "k-1", 2, "sent", Some("2026-09-30T09:47:00Z"));
        intento(&db, "k-1", 3, "sent", Some("2026-09-30T10:02:00Z"));
        // Un fallo con fecha: no cuenta para el ritmo, porque el ritmo es de
        // lo que sale, no de lo que se intenta.
        intento(
            &db,
            "k-1",
            4,
            "permanently_failed",
            Some("2026-09-30T10:05:00Z"),
        );

        let r = db
            .ritmo_por_hora(empresa(), "k-1", "2026-09-30T00:00:00Z")
            .expect("ritmo");

        assert_eq!(
            r,
            vec![
                ("2026-09-30T09".to_owned(), 2),
                ("2026-09-30T10".to_owned(), 1),
            ],
            "dos horas con lo que salió en cada una, y el fallo fuera"
        );
    }

    /// Las cifras de una campaña **no se mezclan con las de otra**.
    ///
    /// Con tres a la vez es el error más fácil de cometer y el más difícil de
    /// ver: todas las cifras parecen razonables.
    #[test]
    fn las_cifras_de_una_campana_no_traen_las_de_las_otras() {
        let (_d, db) = base();

        intento(&db, "k-1", 1, "sent", Some(AHORA));
        for n in 1..=5 {
            intento(&db, "k-2", n, "sent", Some(AHORA));
        }

        let uno = db.resumen_de_campana(empresa(), "k-1").expect("r");
        let dos = db.resumen_de_campana(empresa(), "k-2").expect("r");

        assert_eq!(uno.salio, 1);
        assert_eq!(dos.salio, 5);
    }

    /// **El resumen no se arrastra con una campaña grande.**
    ///
    /// `resumen_de_campana` cuenta con un `GROUP BY` sobre `message_attempt`,
    /// que con 500 000 filas es un recorrido si no hay índice por campaña. Y
    /// la pantalla lo pide **cada cinco segundos** mientras está abierta.
    ///
    /// Aquí se mide el plan de verdad con 50 000 intentos. La primera versión
    /// sólo exigía que no hubiera «SCAN», y **pasaba igual con el índice
    /// reducido a `(campaign_id)`**: no verificaba lo que decía verificar.
    ///
    /// Medido, la diferencia es real y está en el plan: con
    /// `(campaign_id, state)` sale «USING COVERING INDEX» —SQLite lee sólo el
    /// índice—, y con `(campaign_id)` sale «USE TEMP B-TREE FOR GROUP BY».
    /// Eso es lo que ahora se exige.
    #[test]
    fn contar_los_estados_no_recorre_la_tabla_entera() {
        let (_d, db) = base();

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("BEGIN;")?;
            for n in 0..50_000u32 {
                c.execute(
                    "INSERT INTO message_attempt
                         (id, campaign_id, contact_address, idempotency_key, state,
                          channel, created_at, updated_at)
                     VALUES (?1, 'k-1', ?2, ?1, 'sent', 'email', ?3, ?3)",
                    params![format!("g-{n}"), format!("g{n}@t.mx"), AHORA],
                )?;
            }
            c.execute_batch("COMMIT;")
        })
        .expect("siembra grande");

        let plan: String = db
            .ejecutar_en_pruebas(|c| {
                let mut q = c.prepare(
                    "EXPLAIN QUERY PLAN
                     SELECT m.state, count(*)
                       FROM message_attempt m
                       JOIN campaign c ON c.id = m.campaign_id
                      WHERE m.campaign_id = ?1 AND c.company_id = ?2
                      GROUP BY m.state",
                )?;
                let v = q
                    .query_map(params!["k-1", EMPRESA], |r| r.get::<_, String>(3))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(v.join(" · "))
            })
            .expect("plan");

        assert!(
            plan.contains("COVERING INDEX") && !plan.contains("TEMP B-TREE"),
            "contar los estados no se sirve entero del índice. Con \
             `(campaign_id, state)` SQLite lee sólo el índice y agrupa sobre la \
             marcha; con `(campaign_id)` a secas monta un árbol temporal para el \
             GROUP BY — medido, no supuesto. Y la pantalla lo pide cada cinco \
             segundos con la campaña entera detrás. Plan: {plan}"
        );

        // Y la cifra sale bien con la tabla llena, que es lo que de verdad
        // importa: un plan bonito que devuelva mal el número no vale nada.
        let r = db.resumen_de_campana(empresa(), "k-1").expect("resumen");
        assert_eq!(r.salio, 50_000);
    }

    /// **Cuántas consultas cuesta pintar la pantalla.**
    ///
    /// `resumen_de_la_empresa` llama a `resumen_de_campana` por cada campaña,
    /// y cada una abre la conexión y hace tres consultas. Con veinte campañas
    /// son sesenta consultas y sesenta tomas del cerrojo **cada cinco
    /// segundos**, que es lo que la pantalla va a pedir.
    ///
    /// Esta prueba no adivina: cuenta las consultas de verdad con el contador
    /// de SQLite, y falla si el coste crece más de lo que se declara aquí.
    #[test]
    fn pintar_la_pantalla_no_cuesta_una_consulta_por_campana() {
        let (_d, db) = base();

        let antes = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT count(*) FROM campaign", [], |r| r.get::<_, i64>(0))
            })
            .expect("cuenta");
        assert_eq!(antes, 4, "cuatro campañas en el decorado");

        // Treinta campañas más: no es un número de fantasía. Dirección lanza
        // tres a la vez, pero las campañas **no se borran** —son el registro
        // de lo que se envió—, así que en un año de uso hay decenas.
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("BEGIN;")?;
            for n in 0..30u32 {
                c.execute(
                    "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 'completed', ?4, ?4)",
                    params![format!("v-{n}"), EMPRESA, format!("Vieja {n}"), AHORA],
                )?;
                for m in 0..20u32 {
                    c.execute(
                        "INSERT INTO message_attempt
                             (id, campaign_id, contact_address, idempotency_key, state,
                              channel, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?1, 'sent', 'email', ?4, ?4)",
                        params![
                            format!("vm-{n}-{m}"),
                            format!("v-{n}"),
                            format!("v{n}x{m}@t.mx"),
                            AHORA
                        ],
                    )?;
                }
            }
            c.execute_batch("COMMIT;")
        })
        .expect("campañas viejas");

        let t = std::time::Instant::now();
        for _ in 0..20 {
            db.resumen_de_la_empresa(empresa()).expect("resumen");
        }
        let cada = t.elapsed() / 20;

        // El presupuesto es generoso a propósito: no se busca el milisegundo,
        // se busca que **no crezca sin control**. Si un día esto tarda cien
        // milisegundos con treinta campañas, con trescientas tarda un segundo
        // y la pantalla se siente rota.
        assert!(
            cada < std::time::Duration::from_millis(50),
            "pintar la pantalla cuesta {cada:?} con 34 campañas, y se pide cada \
             cinco segundos. Cada campaña añade tres consultas y una toma del \
             cerrojo: es un N+1 que crece con el historial, no con lo que se \
             está enviando"
        );
    }

    /// Una empresa no ve las cifras de otra.
    ///
    /// Hoy ARLES es de una sola empresa, así que esto no puede fallar en
    /// producción — y por eso mismo es fácil escribir la consulta sin el
    /// filtro y no enterarse hasta que deje de serlo.
    #[test]
    fn el_resumen_filtra_por_empresa() {
        let (_d, db) = base();
        intento(&db, "k-1", 1, "sent", Some(AHORA));

        let otra: CompanyId = "22222222-2222-4222-8222-222222222222".parse().expect("id");
        let r = db.resumen_de_campana(otra, "k-1").expect("resumen");

        assert_eq!(r.salio, 0, "la campaña es de otra empresa: no se cuenta");
        assert_eq!(r.audiencia, 0);
    }
    /// Mete un intento fallido con su causa.
    fn fallo_con_causa(db: &Db, campana: &str, n: u32, estado: &str, causa: Option<&str>) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO message_attempt
                     (id, campaign_id, contact_address, idempotency_key, state,
                      channel, error_kind, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'email', ?6, ?7, ?7)",
                params![
                    format!("f-{campana}-{n}"),
                    campana,
                    format!("f{n}@t.mx"),
                    format!("kf-{campana}-{n}"),
                    estado,
                    causa,
                    AHORA
                ],
            )
            .map(|_| ())
        })
        .expect("fallo");
    }

    /// **Los fallos se agrupan por causa, de más a menos.**
    ///
    /// Es lo que el resumen exportable necesita para decir «30 · la dirección
    /// no existe» en vez de «30 fallaron» a secas. Un resumen que dice cuántos
    /// fallaron sin decir por qué no sirve para arreglar nada.
    ///
    /// Verificada rompiéndola el 30/09/2026 quitando el `GROUP BY`: llega una
    /// sola fila con todo junto y falla.
    #[test]
    fn los_fallos_se_agrupan_por_causa() {
        let (_dir, db) = base();
        fallo_con_causa(
            &db,
            "k-1",
            1,
            "permanently_failed",
            Some("reintento.direccionInexistente"),
        );
        fallo_con_causa(
            &db,
            "k-1",
            2,
            "permanently_failed",
            Some("reintento.direccionInexistente"),
        );
        fallo_con_causa(
            &db,
            "k-1",
            3,
            "permanently_failed",
            Some("reintento.seAgotaronLosIntentos"),
        );

        let causas = db.fallos_por_causa(empresa(), "k-1").expect("lee");
        assert_eq!(
            causas,
            vec![
                ("reintento.direccionInexistente".to_owned(), 2),
                ("reintento.seAgotaronLosIntentos".to_owned(), 1),
            ],
            "de más a menos: la causa más frecuente es la que hay que atacar"
        );
    }

    /// **Sólo la dirección inexistente cuenta como rebote duro** (R-3).
    ///
    /// Es la primera de las dos decisiones de `rebotes_duros_de`. Contar todos
    /// los `permanently_failed` metería dentro las credenciales caducadas y
    /// los intentos agotados, que no dicen nada de la lista: una cuenta con la
    /// contraseña mal frenaría la campaña acusando a la lista.
    #[test]
    fn solo_la_direccion_inexistente_cuenta_como_rebote_duro() {
        let (_dir, db) = base();
        fallo_con_causa(
            &db,
            "k-1",
            1,
            "permanently_failed",
            Some("reintento.direccionInexistente"),
        );
        fallo_con_causa(
            &db,
            "k-1",
            2,
            "permanently_failed",
            Some("reintento.seAgotaronLosIntentos"),
        );
        fallo_con_causa(
            &db,
            "k-1",
            3,
            "permanently_failed",
            Some("reintento.laCuentaNoAutentica"),
        );

        let (duros, de) = db.rebotes_duros_de(empresa(), "k-1").expect("lee");
        assert_eq!(duros, 1, "de los tres fallos, uno es rebote de verdad");
        assert_eq!(de, 3, "los tres están resueltos, los tres cuentan de base");
    }

    /// **Los `suppressed` NO entran en el denominador**, y es la parte sutil.
    ///
    /// Esa gente estaba en la lista de bajas y **nunca se le llamó**: no hubo
    /// `RCPT TO`. Meterlos diluiría el porcentaje con envíos que no
    /// ocurrieron, y el freno no saltaría en el caso que más lo necesita —una
    /// lista vieja, llena de bajas y de direcciones muertas.
    ///
    /// Se mide con la cuenta que lo distingue: 3 rebotes de 50 resueltos es el
    /// 6 % y frena; con 50 suprimidos dentro serían 3 de 100, el 3 %, y no
    /// frenaría. La misma campaña, el mismo daño, y la red callada.
    #[test]
    fn los_suprimidos_no_diluyen_el_porcentaje() {
        let (_dir, db) = base();
        for n in 0..3 {
            fallo_con_causa(
                &db,
                "k-1",
                n,
                "permanently_failed",
                Some("reintento.direccionInexistente"),
            );
        }
        for n in 3..50 {
            fallo_con_causa(&db, "k-1", n, "sent", None);
        }
        // Y cincuenta que nunca se intentaron.
        for n in 100..150 {
            fallo_con_causa(&db, "k-1", n, "suppressed", None);
        }

        let (duros, de) = db.rebotes_duros_de(empresa(), "k-1").expect("lee");
        assert_eq!(duros, 3);
        assert_eq!(de, 50, "los 50 suprimidos no se intentaron: fuera");
        assert!(
            arles_core::rebotes::mirar(duros, de).hay_que_frenar(),
            "3 de 50 es el 6 % y tiene que frenar"
        );
    }

    /// **Lo que sigue en cola no cuenta todavía.**
    ///
    /// Si la audiencia entera fuera el denominador, una campaña de 10 000 con
    /// 400 rebotes de los primeros 420 envíos daría un 4 % —por debajo del
    /// freno— cuando lo que pasa es que casi todo lo que sale rebota.
    #[test]
    fn lo_que_sigue_en_cola_no_cuenta() {
        let (_dir, db) = base();
        for n in 0..60 {
            fallo_con_causa(
                &db,
                "k-1",
                n,
                "permanently_failed",
                Some("reintento.direccionInexistente"),
            );
        }
        for n in 100..1_000 {
            fallo_con_causa(&db, "k-1", n, "queued", None);
        }

        let (duros, de) = db.rebotes_duros_de(empresa(), "k-1").expect("lee");
        assert_eq!((duros, de), (60, 60), "sólo lo resuelto");
        assert!(arles_core::rebotes::mirar(duros, de).hay_que_frenar());
    }

    /// **Un fallo que se va a reintentar NO cuenta como fallo.**
    ///
    /// ── EL DEFECTO QUE ESTO CAZA ──
    ///
    /// `failed` significa «falló esta vez y se reintentará». Contarlo en el
    /// resumen diría que se perdieron mensajes que todavía están en camino, y
    /// alguien tomaría una decisión —parar la campaña, cambiar de remitente—
    /// sobre un problema que no existe.
    ///
    /// Sólo cuentan los estados definitivos: `permanently_failed` y
    /// `suppressed`.
    ///
    /// Verificada rompiéndola el 30/09/2026 añadiendo `'failed'` a la lista de
    /// estados: llegan 2 causas en vez de 1 y falla.
    #[test]
    fn un_reintento_pendiente_no_es_un_fallo() {
        let (_dir, db) = base();
        fallo_con_causa(
            &db,
            "k-1",
            1,
            "permanently_failed",
            Some("reintento.seAgotaronLosIntentos"),
        );
        // Éste va a volver a intentarse: todavía no se ha perdido nada.
        fallo_con_causa(
            &db,
            "k-1",
            2,
            "failed",
            Some("reintento.seVolveraAIntentar"),
        );

        let causas = db.fallos_por_causa(empresa(), "k-1").expect("lee");
        assert_eq!(causas.len(), 1, "un reintento pendiente no es un fallo");
        let (clave, _) = causas.first().expect("hay una causa");
        assert_eq!(clave, "reintento.seAgotaronLosIntentos");
    }

    /// **Y los fallos son de la campaña que se pide, no de todas.**
    ///
    /// Sin el filtro por campaña, el resumen de la campaña de octubre llevaría
    /// los rebotes de la de agosto — cifras correctas bajo un título falso.
    #[test]
    fn los_fallos_son_de_una_sola_campana() {
        let (_dir, db) = base();
        fallo_con_causa(
            &db,
            "k-1",
            1,
            "permanently_failed",
            Some("reintento.direccionInexistente"),
        );
        fallo_con_causa(
            &db,
            "k-2",
            2,
            "permanently_failed",
            Some("reintento.laCuentaNoAutentica"),
        );

        let causas = db.fallos_por_causa(empresa(), "k-1").expect("lee");
        assert_eq!(causas.len(), 1);
        let (clave, _) = causas.first().expect("hay una causa");
        assert_eq!(clave, "reintento.direccionInexistente");
    }
}
