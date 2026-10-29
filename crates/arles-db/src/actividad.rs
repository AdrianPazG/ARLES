//! El registro de actividad: qué fue pasando, y cuándo.
//!
//! Entrega 7.1. La migración `V10__registro_de_actividad.sql` explica por qué
//! esta tabla **no es la fuente de verdad** y por qué no guarda ni una
//! dirección. Aquí está lo que el resto del programa usa.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA FUNCIÓN QUE DECIDE SI ESTO SIRVE O LLENA LA BASE DE NADA
//!
//! `anotar_si_cambia`. El motor da una vuelta cada cinco segundos: fuera de
//! horario, una etapa devuelve «fuera de ventana» **17 280 veces al día**.
//! Anotarlas todas no informa de nada y hace inútil la pantalla.
//!
//! Con el recorte por cambio, un fin de semana entero son **dos filas**: la del
//! viernes a las 18:00 diciendo que se acabó el horario, y la del lunes a las
//! 9:00 diciendo que volvió. Que es exactamente lo que alguien querría leer.

use std::collections::HashMap;

use arles_core::ids::{ActivityEventId, CompanyId};
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Un suceso a punto de anotarse.
///
/// Se pasa entero y no como siete argumentos sueltos: son siete y cuatro son
/// opcionales, y una llamada con cuatro `None` seguidos es una invitación a
/// ponerlos en el orden equivocado sin que el compilador diga nada.
#[derive(Debug, Clone)]
pub struct SucesoNuevo<'a> {
    /// La clave de i18n, tal cual la da `Paso::clave_i18n()` (§139).
    pub clave: &'a str,
    /// `true` si es el desenlace de un intento de envío. **No se borra nunca.**
    pub es_desenlace: bool,
    pub campana: Option<&'a str>,
    pub etapa: Option<&'a str>,
    /// `"email"` o `"whatsapp"`. Lo comprueba un `CHECK` del esquema.
    pub canal: Option<&'a str>,
    pub intento: Option<&'a str>,
    /// Las cifras del suceso: «reintento **2** de **3**», «vuelve en **23**».
    pub numero_a: Option<i64>,
    pub numero_b: Option<i64>,
}

/// Un suceso leído, listo para la pantalla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suceso {
    pub id: String,
    pub clave: String,
    pub es_desenlace: bool,
    pub campana: Option<String>,
    pub canal: Option<String>,
    pub intento: Option<String>,
    pub numero_a: Option<i64>,
    pub numero_b: Option<i64>,
    pub cuando: String,
    /// A quién se le escribió, **leída de `message_attempt` al consultar**.
    ///
    /// ── POR QUÉ NO ESTÁ GUARDADA AQUÍ, Y POR QUÉ ESO IMPORTA ──
    ///
    /// `activity_event` no tiene ni una dirección de correo: sólo el id del
    /// intento (ver la cabecera de la V10). El motivo es P-17, y la
    /// consecuencia es la que se quería: **el día que alguien ejerza su
    /// derecho de cancelación y su intento se anonimice, el registro deja de
    /// enseñar su dirección sin tocar una sola fila de esta tabla** — que es
    /// de sólo-añadir y no se podría corregir.
    ///
    /// `None` en los sucesos de bucle, que no son de nadie.
    pub direccion: Option<String>,
}

/// Cuántos sucesos se sirven de una vez (D-7C).
///
/// **Siempre, no a partir de un umbral.** Una campaña de 3 000 ya produce una
/// tabla inmanejable a ojo y no llegaría a ningún umbral razonable; y una
/// pantalla que cambia de comportamiento según el tamaño se siente rota. El
/// total va al lado para que el tope no esconda nada: «500 de 3 412».
pub const TOPE_DE_PAGINA: u32 = 500;

/// Qué se lee, y de dónde.
///
/// ───────────────────────────────────────────────────────────────────────────
/// LOS CUATRO CRITERIOS DE D-7C, Y POR QUÉ SON ESTOS
///
/// Dirección lo pidió así el 29/09/2026: «debería tener un filtro del registro,
/// para poder buscar entre todos los registros». Con 20 000 correos y mensajes
/// juntos, una lista de 500 sin forma de buscar no es un registro: es una
/// muestra. Los cuatro responden a las cuatro preguntas que alguien se hace
/// delante de esta pantalla:
///
///   - **campaña** → «¿qué pasó con la de octubre?»
///   - **clave** → «enséñame sólo los rebotes»
///   - **desde / hasta** → «¿qué pasó el martes?»
///   - **texto** → «¿le llegó a Ana?»
///
/// Todos combinables, y **todos opcionales**: un filtro que obliga a elegir
/// algo antes de enseñar nada convierte la primera visita en un formulario.
#[derive(Debug, Clone, Default)]
pub struct Filtro<'a> {
    /// Sólo esta campaña. Con tres a la vez, es el primer filtro que hace falta.
    pub campana: Option<&'a str>,
    /// Sólo desenlaces de envío, o sólo sucesos de bucle.
    pub solo_desenlaces: Option<bool>,
    /// Sólo esta clase de suceso, por su clave de i18n: `reintento.pidioLaBaja`.
    ///
    /// La clave y no un grupo propio: es lo que la tabla guarda y lo que la
    /// pantalla traduce, así que no hay una segunda lista que pueda divergir.
    pub clave: Option<&'a str>,
    /// Desde esta marca de tiempo, inclusive.
    pub desde: Option<&'a str>,
    /// Hasta esta marca de tiempo, **inclusive**.
    ///
    /// Inclusive a propósito: quien filtra «hasta el martes» quiere el martes
    /// entero. Quien llama pasa el final del día, no su principio.
    pub hasta: Option<&'a str>,
    /// Una dirección o un número, o un trozo de ellos.
    ///
    /// Busca en `message_attempt`, que es donde vive la dirección: aquí no hay
    /// ninguna (P-17). Por eso **sólo encuentra desenlaces de envío**: un
    /// suceso de bucle no es de nadie, y buscar «ana@» no puede devolver «fuera
    /// de horario».
    pub texto: Option<&'a str>,
}

/// Lo que devuelve una consulta: la página y **cuántos hay en total**.
///
/// Las dos cosas juntas y no dos métodos: enseñar 500 filas sin decir de
/// cuántas es exactamente el defecto que D-7C quiere evitar, y con dos métodos
/// separados es cuestión de tiempo que alguien llame sólo al primero.
#[derive(Debug, Clone)]
pub struct Pagina {
    pub sucesos: Vec<Suceso>,
    pub total: u32,
}

impl Db {
    /// Anota un suceso, **sin mirar si se repite**.
    ///
    /// Es la puerta de los desenlaces de envío: cada uno cuenta, aunque sean
    /// cien iguales seguidos, porque cada uno es un correo distinto.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn anotar_suceso(&self, empresa: CompanyId, s: &SucesoNuevo<'_>) -> Result<(), DbError> {
        let cuando = ahora();
        self.con(|conn| insertar(conn, empresa, s, &cuando))?;
        Ok(())
    }

    /// Anota un suceso **sólo si es distinto del último de esa etapa**.
    ///
    /// Es la puerta de los sucesos de bucle. Ver la cabecera del módulo: sin
    /// esto, un fin de semana son 34 560 filas que dicen lo mismo.
    ///
    /// Devuelve `true` si se anotó.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la lectura o la escritura fallan.
    pub fn anotar_si_cambia(
        &self,
        empresa: CompanyId,
        s: &SucesoNuevo<'_>,
    ) -> Result<bool, DbError> {
        let cuando = ahora();
        self.con(|conn| {
            // La comparación es por CLAVE, no por fila entera: «vuelve en 23
            // minutos» y «vuelve en 22 minutos» son el mismo suceso contado dos
            // veces, y anotar los dos es volver al problema del volumen por la
            // puerta de atrás.
            // ── SE ORDENA POR `rowid`, Y NO POR FECHA ────────────────────
            //
            // «El último de esta etapa» es, literalmente, el insertado más
            // tarde: el `rowid` más alto. Ordenar por `occurred_at DESC,
            // rowid DESC` daba el mismo resultado y obligaba a SQLite a montar
            // **un árbol temporal en cada consulta** —lo dijo
            // `EXPLAIN QUERY PLAN`: «USE TEMP B-TREE FOR LAST TERM OF ORDER
            // BY»—, porque un índice ordena su rowid ascendente y no se puede
            // pedir descendente en el último término.
            //
            // Esta consulta corre **cada cinco segundos, para siempre**, por
            // etapa. Un árbol temporal ahí es un motor que se frena solo a
            // medida que trabaja: el peor tipo de defecto, porque aparece
            // tarde, en producción, y parece otra cosa.
            //
            // Se encontró midiendo el plan con 20 000 filas, no leyendo el
            // código. La prueba `preguntar_por_el_ultimo_no_recorre_la_tabla_entera`
            // lo vigila, y exige que no haya árbol temporal — no sólo que se
            // use un índice, que es lo que afirmaba su primera versión y se
            // cumplía igual con el índice equivocado.
            let ultima: Option<String> = conn
                .query_row(
                    "SELECT kind FROM activity_event
                      WHERE company_id = ?1
                        AND campaign_stage_id IS ?2
                      ORDER BY rowid DESC
                      LIMIT 1",
                    params![empresa.to_string(), s.etapa],
                    |f| f.get(0),
                )
                .optional()?;

            if ultima.as_deref() == Some(s.clave) {
                return Ok(false);
            }
            insertar(conn, empresa, s, &cuando)?;
            Ok(true)
        })
    }

    /// Lee una página del registro, con su total.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn sucesos(&self, empresa: CompanyId, f: &Filtro<'_>) -> Result<Pagina, DbError> {
        self.leer(|conn| {
            let empresa = empresa.to_string();
            // El mismo `WHERE` para contar y para leer, construido una sola
            // vez. Dos cadenas parecidas es como se acaba enseñando «500 de
            // 3 412» con un total que no corresponde a lo que se ve.
            // ── EL MISMO `WHERE` PARA CONTAR Y PARA LEER ──────────────────
            //
            // Construido una sola vez. Dos cadenas parecidas es como se acaba
            // enseñando «500 de 3 412» con un total que no corresponde a lo
            // que se ve, que es justo lo que D-7C quiere impedir.
            //
            // El JOIN es **LEFT** y no interno: un suceso de bucle no tiene
            // intento, y con un JOIN interno desaparecerían todos —la mitad
            // del registro, en silencio—.
            //
            // `?7` compara contra la dirección del intento, que vive en
            // `message_attempt` porque aquí no hay ninguna (P-17). Es un
            // `LIKE` con comodines a los dos lados, así que **no usa índice**:
            // medido sobre 20 000 filas, 11 ms. Con el filtro de campaña
            // puesto —que es como se usa— el índice recorta antes y baja a 2.
            // Si algún día hiciera falta más, la respuesta es FTS5, no un
            // índice sobre un LIKE que no lo puede aprovechar.
            let donde = "e.company_id = ?1
                     AND (?2 IS NULL OR e.campaign_id = ?2)
                     AND (?3 IS NULL OR e.is_outcome = ?3)
                     AND (?4 IS NULL OR e.kind = ?4)
                     AND (?5 IS NULL OR e.occurred_at >= ?5)
                     AND (?6 IS NULL OR e.occurred_at <= ?6)
                     AND (?7 IS NULL OR a.contact_address LIKE '%' || ?7 || '%')";
            let de = "activity_event e
                      LEFT JOIN message_attempt a ON a.id = e.message_attempt_id";
            let atados = params![
                empresa,
                f.campana,
                f.solo_desenlaces.map(i64::from),
                f.clave,
                f.desde,
                f.hasta,
                f.texto,
            ];

            let total: u32 = conn.query_row(
                &format!("SELECT count(*) FROM {de} WHERE {donde}"),
                atados,
                |r| r.get(0),
            )?;

            let mut consulta = conn.prepare(&format!(
                "SELECT e.id, e.kind, e.is_outcome, e.campaign_id, e.channel,
                        e.message_attempt_id, e.number_a, e.number_b,
                        e.occurred_at, a.contact_address
                   FROM {de}
                  WHERE {donde}
                  ORDER BY e.occurred_at DESC, e.rowid DESC
                  LIMIT {TOPE_DE_PAGINA}"
            ))?;

            let sucesos = consulta
                .query_map(atados, |r| {
                    Ok(Suceso {
                        id: r.get(0)?,
                        clave: r.get(1)?,
                        es_desenlace: r.get::<_, i64>(2)? == 1,
                        campana: r.get(3)?,
                        canal: r.get(4)?,
                        intento: r.get(5)?,
                        numero_a: r.get(6)?,
                        numero_b: r.get(7)?,
                        cuando: r.get(8)?,
                        direccion: r.get(9)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Pagina { sucesos, total })
        })
    }

    /// **Por qué está quieta cada campaña.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL FALLO MÁS FRECUENTE DE UN PRODUCTO ASÍ NO ES UN ERROR: ES EL SILENCIO
    ///
    /// ARLES puede estar completamente quieto por **seis razones distintas**,
    /// todas correctas: fuera de horario, sin cupo, disyuntor abierto, campaña
    /// en pausa, parada de emergencia, o una cuenta sin contraseña. Quien mira
    /// la pantalla no ve ninguna: ve que no pasa nada, y concluye que ARLES se
    /// rompió.
    ///
    /// El motor ya las calcula las seis —son variantes de `Paso`, cada una con
    /// su texto en español—, y desde la 7.1 quedan escritas. Esto las lee.
    ///
    /// Devuelve el **último suceso de bucle** de cada campaña, que es
    /// literalmente la última razón por la que el motor no envió. Los
    /// desenlaces no cuentan: «se aceptó un correo» no explica por qué ahora
    /// no sale ninguno.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn por_que_esta_quieta_cada_campana(
        &self,
        empresa: CompanyId,
    ) -> Result<HashMap<String, Suceso>, DbError> {
        self.con(|conn| {
            // Una sola consulta para todas las campañas, y no una por campaña:
            // con tres enviando y treinta en el historial serían treinta y tres
            // viajes cada cinco segundos.
            //
            // `MAX(rowid)` por campaña es el último insertado, por el mismo
            // motivo que en `anotar_si_cambia`: el rowid es monótono y no hace
            // falta ordenar por fecha.
            let mut q = conn.prepare(
                "SELECT e.campaign_id, e.id, e.kind, e.number_a, e.number_b, e.occurred_at
                   FROM activity_event e
                   JOIN (SELECT campaign_id, MAX(rowid) AS ultimo
                           FROM activity_event
                          WHERE company_id = ?1
                            AND is_outcome = 0
                            AND campaign_id IS NOT NULL
                          GROUP BY campaign_id) u
                     ON u.ultimo = e.rowid",
            )?;

            let mut por_campana = HashMap::new();
            for fila in q.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    Suceso {
                        id: f.get(1)?,
                        clave: f.get(2)?,
                        es_desenlace: false,
                        campana: None,
                        canal: None,
                        intento: None,
                        // Una razón es un suceso de bucle: no es de nadie, así
                        // que no hay dirección que enseñar.
                        direccion: None,
                        numero_a: f.get(3)?,
                        numero_b: f.get(4)?,
                        cuando: f.get(5)?,
                    },
                ))
            })? {
                let (campana, suceso) = fila?;
                por_campana.insert(campana, suceso);
            }
            Ok(por_campana)
        })
    }

    /// Cuántos sucesos de bucle se borrarían al recortar (D-7C).
    ///
    /// Existe para poder **preguntar antes**: «se van a quitar 42 100 líneas de
    /// espera de hace más de dos meses. ¿Guardar una copia?». Un borrado que no
    /// dice cuánto borra no se puede autorizar.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn cuanto_se_recortaria(&self, empresa: CompanyId, antes_de: &str) -> Result<u32, DbError> {
        self.con(|conn| {
            conn.query_row(
                "SELECT count(*) FROM activity_event
                  WHERE company_id = ?1 AND is_outcome = 0 AND occurred_at < ?2",
                params![empresa.to_string(), antes_de],
                |r| r.get(0),
            )
        })
    }

    /// Las clases de suceso que **hay de verdad** en el registro, con cuántos.
    ///
    /// ───────────────────────────────────────────────────────────────────────
    /// LEÍDAS DE LOS DATOS, NO DE UNA LISTA
    ///
    /// El desplegable de «qué pasó» podría llenarse con las catorce variantes
    /// de `Paso`. Sería peor por dos motivos, y los dos se notan el primer día:
    ///
    ///   1. **Doce de las catorce estarían vacías.** Quien filtra por «la
    ///      cuenta no autentica» y no ve nada no sabe si es que no pasó o si el
    ///      filtro está roto. Un desplegable que sólo ofrece lo que existe no
    ///      tiene esa duda: cada opción lleva su cifra al lado (§94).
    ///   2. Una lista escrita a mano se queda vieja el día que se añada un
    ///      `Paso`, y nadie se entera hasta que alguien lo busca.
    ///
    /// Se ordenan por frecuencia y no por nombre: lo que más pasa es lo que
    /// más se busca.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn clases_de_suceso(&self, empresa: CompanyId) -> Result<Vec<(String, u32)>, DbError> {
        self.con(|conn| {
            let mut q = conn.prepare(
                "SELECT kind, count(*) AS n
                   FROM activity_event
                  WHERE company_id = ?1
                  GROUP BY kind
                  ORDER BY n DESC, kind",
            )?;
            let filas = q
                .query_map(params![empresa.to_string()], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(filas)
        })
    }

    /// Borra los sucesos de bucle anteriores a una fecha. Devuelve cuántos.
    ///
    /// **Los desenlaces de envío no se tocan nunca**, y no porque esta función
    /// se acuerde: el `WHERE is_outcome = 0` es la única forma de llamar a
    /// `DELETE` sobre esta tabla en todo el programa, y hay una prueba que lo
    /// verifica pidiéndole que borre un desenlace y comprobando que sigue ahí.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si el borrado falla.
    pub fn recortar_sucesos(&self, empresa: CompanyId, antes_de: &str) -> Result<u32, DbError> {
        self.con(|conn| {
            let n = conn.execute(
                "DELETE FROM activity_event
                  WHERE company_id = ?1 AND is_outcome = 0 AND occurred_at < ?2",
                params![empresa.to_string(), antes_de],
            )?;
            Ok(u32::try_from(n).unwrap_or(u32::MAX))
        })
    }
}

fn insertar(
    conn: &Connection,
    empresa: CompanyId,
    s: &SucesoNuevo<'_>,
    cuando: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO activity_event
             (id, company_id, campaign_id, campaign_stage_id, channel,
              kind, is_outcome, message_attempt_id, number_a, number_b, occurred_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            ActivityEventId::nuevo().to_string(),
            empresa.to_string(),
            s.campana,
            s.etapa,
            s.canal,
            s.clave,
            i64::from(s.es_desenlace),
            s.intento,
            s.numero_a,
            s.numero_b,
            cuando,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const AHORA: &str = "2026-09-30T10:00:00Z";
    const ETAPA: &str = "et-1";
    const OTRA_ETAPA: &str = "et-2";

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
                                            daily_limit, hourly_limit,
                                            created_at, updated_at)
                 VALUES ('c-1', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                         'llavero://c', 'connected', 50, 10, '{AHORA}', '{AHORA}');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('v-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '{AHORA}');

                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-1', '{EMPRESA}', 'Clientes de agosto', 'running',
                         '{AHORA}', '{AHORA}');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('k-2', '{EMPRESA}', 'Reactivación', 'running',
                         '{AHORA}', '{AHORA}');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('{ETAPA}', 'k-1', 1, 'email', 'c-1', 'v-1', 'running',
                         '{AHORA}', '{AHORA}');
                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('{OTRA_ETAPA}', 'k-2', 1, 'email', 'c-1', 'v-1', 'running',
                         '{AHORA}', '{AHORA}');"
            ))
        })
        .expect("siembra");
        (dir, db)
    }

    /// El primer suceso de una página, sin indexar.
    ///
    /// `sucesos[0]` entra en pánico con la lista vacía, y una prueba que entra
    /// en pánico no dice cuál era la expectativa: dice «index out of bounds».
    fn primero(sucesos: &[Suceso]) -> &Suceso {
        sucesos.first().expect("la página tiene al menos un suceso")
    }

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
    }

    fn de_bucle(clave: &'static str, etapa: &'static str) -> SucesoNuevo<'static> {
        SucesoNuevo {
            clave,
            es_desenlace: false,
            campana: Some("k-1"),
            etapa: Some(etapa),
            canal: Some("email"),
            intento: None,
            numero_a: None,
            numero_b: None,
        }
    }

    fn desenlace(clave: &'static str) -> SucesoNuevo<'static> {
        SucesoNuevo {
            clave,
            es_desenlace: true,
            campana: Some("k-1"),
            etapa: Some(ETAPA),
            canal: Some("email"),
            intento: None,
            numero_a: None,
            numero_b: None,
        }
    }

    /// **La prueba que decide si esta tabla sirve o llena la base de nada.**
    ///
    /// El motor da una vuelta cada cinco segundos. Fuera de horario, una etapa
    /// devuelve «fuera de ventana» 17 280 veces al día; un fin de semana son
    /// 34 560 filas que dicen lo mismo, por etapa.
    ///
    /// Aquí se piden mil de golpe y se exige que quede **una**.
    ///
    /// Verificada rompiéndola: sustituyendo `anotar_si_cambia` por
    /// `anotar_suceso` en el cuerpo del bucle, falla con
    /// «se anotaron 1000 filas iguales: la tabla se llena de ruido».
    #[test]
    fn mil_vueltas_iguales_dejan_una_sola_linea() {
        let (_d, db) = base();

        for _ in 0..1_000 {
            db.anotar_si_cambia(empresa(), &de_bucle("motor.fueraDeVentana", ETAPA))
                .expect("anota");
        }

        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(
            p.total, 1,
            "se anotaron {} filas iguales: la tabla se llena de ruido",
            p.total
        );
    }

    /// Y cuando de verdad cambia algo, se nota.
    ///
    /// Es la otra mitad: un recorte que no dejara pasar los cambios convertiría
    /// la pantalla en una línea congelada del primer día.
    #[test]
    fn el_fin_de_semana_entero_son_dos_lineas() {
        let (_d, db) = base();

        // Viernes 18:00: se acabó el horario. Todo el sábado y el domingo.
        for _ in 0..500 {
            db.anotar_si_cambia(empresa(), &de_bucle("motor.fueraDeVentana", ETAPA))
                .expect("anota");
        }
        // Lunes 9:00: vuelve a haber trabajo.
        let cambio = db
            .anotar_si_cambia(empresa(), &de_bucle("motor.noHayTrabajo", ETAPA))
            .expect("anota");

        assert!(cambio, "el cambio de razón tiene que anotarse");
        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(p.total, 2, "un fin de semana son dos líneas, no 501");
    }

    /// El recorte es **por etapa**, no global.
    ///
    /// Con tres campañas a la vez —el escenario de Dirección— dos etapas pueden
    /// estar fuera de ventana al mismo tiempo. Si la comparación fuera global,
    /// la segunda no se anotaría nunca y su pantalla diría que no pasa nada.
    ///
    /// Verificada rompiéndola: quitando `AND campaign_stage_id IS ?2` de la
    /// consulta, esta prueba falla con 1 en vez de 2.
    #[test]
    fn dos_etapas_fuera_de_ventana_se_anotan_las_dos() {
        let (_d, db) = base();

        db.anotar_si_cambia(empresa(), &de_bucle("motor.fueraDeVentana", ETAPA))
            .expect("anota");
        let segunda = db
            .anotar_si_cambia(empresa(), &de_bucle("motor.fueraDeVentana", OTRA_ETAPA))
            .expect("anota");

        assert!(
            segunda,
            "la segunda etapa tiene su propia historia: sin esto, su pantalla \
             se quedaría muda"
        );
        assert_eq!(
            db.sucesos(empresa(), &Filtro::default())
                .expect("lee")
                .total,
            2
        );
    }

    /// Cien envíos son cien líneas, aunque acaben igual.
    ///
    /// El recorte por repetición **no** aplica a los desenlaces: cada uno es
    /// un correo distinto a una persona distinta, y contarlos como uno sería
    /// perder justo lo que hay que guardar.
    #[test]
    fn cien_envios_aceptados_son_cien_lineas() {
        let (_d, db) = base();

        for _ in 0..100 {
            db.anotar_suceso(empresa(), &desenlace("envio.aceptado"))
                .expect("anota");
        }

        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(p.total, 100);
    }

    /// **Nunca más de 500 en pantalla, y el total dice la verdad** (D-7C).
    ///
    /// Se piden 501 y no uno: una prueba de paginación con un solo registro
    /// pasa aunque el `LIMIT` esté mal, y eso ya costó una vez en la página de
    /// contactos — pedir diez mil devolvía uno y el recorte se podía quitar
    /// entero (§2).
    ///
    /// Verificada rompiéndola: quitando el `LIMIT`, falla con 501 servidos.
    #[test]
    fn se_sirven_500_de_501_y_el_total_lo_dice() {
        let (_d, db) = base();

        for _ in 0..501 {
            db.anotar_suceso(empresa(), &desenlace("envio.aceptado"))
                .expect("anota");
        }

        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(p.sucesos.len(), 500, "en pantalla, nunca más de 500");
        assert_eq!(p.total, 501, "y el total dice cuántos hay de verdad");
    }

    /// El filtro por campaña, que con tres a la vez es el primero que hace falta.
    #[test]
    fn el_filtro_por_campana_separa_las_tres() {
        let (_d, db) = base();

        db.anotar_suceso(empresa(), &desenlace("envio.aceptado"))
            .expect("anota");
        db.anotar_suceso(
            empresa(),
            &SucesoNuevo {
                campana: Some("k-2"),
                etapa: Some(OTRA_ETAPA),
                ..desenlace("envio.rendido")
            },
        )
        .expect("anota");

        let solo_k1 = db
            .sucesos(
                empresa(),
                &Filtro {
                    campana: Some("k-1"),
                    ..Filtro::default()
                },
            )
            .expect("lee");

        assert_eq!(solo_k1.total, 1);
        assert_eq!(
            solo_k1.sucesos.first().map(|s| s.clave.as_str()),
            Some("envio.aceptado")
        );
    }

    /// **El recorte bimestral no toca un solo envío** (D-7C).
    ///
    /// Es la garantía que separa «se recorta» de «se borra»: el ruido de bucle
    /// se va, la prueba de que se escribió a alguien se queda. Borrarla nos
    /// dejaría sin respuesta ante una reclamación.
    ///
    /// Verificada rompiéndola: quitando `AND is_outcome = 0` del `DELETE`, el
    /// desenlace desaparece y esta prueba falla.
    #[test]
    fn el_recorte_se_lleva_el_ruido_y_deja_los_envios() {
        let (_d, db) = base();

        db.anotar_si_cambia(empresa(), &de_bucle("motor.fueraDeVentana", ETAPA))
            .expect("anota");
        db.anotar_suceso(empresa(), &desenlace("envio.aceptado"))
            .expect("anota");

        // Una fecha posterior a las dos: todo es «viejo».
        let manana = "2999-01-01T00:00:00Z";

        let cuantos = db.cuanto_se_recortaria(empresa(), manana).expect("cuenta");
        assert_eq!(cuantos, 1, "sólo el suceso de bucle se puede recortar");

        let borrados = db.recortar_sucesos(empresa(), manana).expect("recorta");
        assert_eq!(borrados, 1);

        let quedan = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(quedan.total, 1);
        assert!(
            quedan.sucesos.first().is_some_and(|s| s.es_desenlace),
            "lo que queda tiene que ser el envío: es la prueba de que se escribió \
             a alguien, y no se borra jamás"
        );
    }

    /// **El recorte no se vuelve lento cuando la tabla crece.**
    ///
    /// `anotar_si_cambia` consulta «cuál fue el último de esta etapa» en cada
    /// vuelta del motor, o sea **cada cinco segundos, para siempre**. Con una
    /// campaña de 500 000 detrás, esa consulta corre sobre una tabla enorme.
    ///
    /// Sin el índice `idx_activity_etapa` sería un recorrido completo, y
    /// el motor se frenaría solo a medida que trabaja — el peor tipo de
    /// defecto: aparece en producción, tarde, y parece otra cosa.
    ///
    /// Aquí se mide de verdad, con 20 000 filas: si el plan de SQLite dejara
    /// de usar el índice, esta prueba lo dice.
    #[test]
    fn preguntar_por_el_ultimo_no_recorre_la_tabla_entera() {
        let (_d, db) = base();

        for _ in 0..20_000 {
            db.anotar_suceso(empresa(), &desenlace("envio.aceptado"))
                .expect("anota");
        }

        let plan: String = db
            .ejecutar_en_pruebas(|c| {
                let mut q = c.prepare(
                    "EXPLAIN QUERY PLAN
                     SELECT kind FROM activity_event
                      WHERE company_id = ?1 AND campaign_stage_id IS ?2
                      ORDER BY rowid DESC
                      LIMIT 1",
                )?;
                let v = q
                    .query_map(params!["x", "y"], |r| r.get::<_, String>(3))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(v.join(" · "))
            })
            .expect("plan");

        assert!(
            plan.contains("idx_activity_etapa") && !plan.contains("TEMP B-TREE"),
            "la consulta del recorte no va directa al índice, o monta un árbol \
             temporal. Corre cada cinco segundos por etapa: con la tabla llena, \
             el motor se frena solo. Plan: {plan}"
        );
    }

    /// **Cada campaña dice por qué está quieta, y no la de al lado.**
    ///
    /// Es la propuesta P-A del plan: el fallo más frecuente de un producto así
    /// no es un error, es el silencio. Con tres campañas a la vez, cada una
    /// puede estar parada por un motivo distinto —una fuera de horario, otra
    /// sin cupo—, y darles a las tres la misma explicación sería peor que no
    /// dar ninguna.
    ///
    /// Verificada rompiéndola: quitando `GROUP BY campaign_id` de la subconsulta,
    /// las dos campañas reciben el mismo suceso y esta prueba lo dice.
    #[test]
    fn cada_campana_dice_su_propia_razon() {
        let (_d, db) = base();

        // k-1 se quedó sin cupo; k-2 está fuera de su horario.
        db.anotar_si_cambia(empresa(), &de_bucle("ritmo.topeDelDia", ETAPA))
            .expect("anota");
        db.anotar_si_cambia(
            empresa(),
            &SucesoNuevo {
                campana: Some("k-2"),
                etapa: Some(OTRA_ETAPA),
                ..de_bucle("motor.fueraDeVentana", OTRA_ETAPA)
            },
        )
        .expect("anota");

        let razones = db
            .por_que_esta_quieta_cada_campana(empresa())
            .expect("razones");

        assert_eq!(
            razones.get("k-1").map(|s| s.clave.as_str()),
            Some("ritmo.topeDelDia")
        );
        assert_eq!(
            razones.get("k-2").map(|s| s.clave.as_str()),
            Some("motor.fueraDeVentana"),
            "cada campaña tiene su motivo: darles el mismo sería peor que \
             no dar ninguno"
        );
    }

    /// **Un envío aceptado no explica por qué ahora no sale nada.**
    ///
    /// La razón se busca entre los sucesos de bucle. Si se colaran los
    /// desenlaces, una campaña que acaba de mandar un correo y luego se quedó
    /// sin cupo diría «se aceptó un correo» — que es cierto y no responde a la
    /// pregunta.
    #[test]
    fn la_razon_no_es_un_desenlace_de_envio() {
        let (_d, db) = base();

        db.anotar_si_cambia(empresa(), &de_bucle("ritmo.topeDelDia", ETAPA))
            .expect("anota");
        // Después, un envío. Es más reciente, pero no es una razón.
        db.anotar_suceso(empresa(), &desenlace("motor.aceptadoPorElProveedor"))
            .expect("anota");

        let razones = db
            .por_que_esta_quieta_cada_campana(empresa())
            .expect("razones");

        assert_eq!(
            razones.get("k-1").map(|s| s.clave.as_str()),
            Some("ritmo.topeDelDia"),
            "la razón tiene que ser el último suceso de BUCLE, no el último de \
             todos: «se aceptó un correo» no explica por qué no sale ninguno"
        );
    }

    /// Un suceso pasó como pasó: la tabla no admite `UPDATE`.
    #[test]
    fn un_suceso_no_se_puede_corregir() {
        let (_d, db) = base();
        db.anotar_suceso(empresa(), &desenlace("envio.aceptado"))
            .expect("anota");

        let r = db.ejecutar_en_pruebas(|c| {
            c.execute("UPDATE activity_event SET kind = 'mentira'", [])
                .map(|_| ())
        });

        assert!(r.is_err(), "el disparador tiene que impedir el UPDATE");
    }

    /// **Aquí no hay ni una dirección de correo.**
    ///
    /// Se comprueba sobre el esquema y no sobre una fila: lo que hay que
    /// impedir es que alguien **añada la columna**, y eso no lo ve una prueba
    /// que sólo mire los datos de hoy. Es el mismo problema que P-17 tiene
    /// abierto sobre `consent_entry`, y repetirlo aquí —en una tabla que va a
    /// tener millones de filas— sería crearlo a sabiendas.
    #[test]
    fn el_registro_no_tiene_donde_guardar_una_direccion() {
        let (_d, db) = base();

        let columnas: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut q = c.prepare("SELECT name FROM pragma_table_info('activity_event')")?;
                let v = q
                    .query_map([], |r| r.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(v)
            })
            .expect("lee el esquema");

        for sospechosa in ["email", "address", "phone", "contact_email", "to_address"] {
            assert!(
                !columnas.iter().any(|c| c.contains(sospechosa)),
                "`activity_event` tiene una columna «{sospechosa}»: el registro \
                 de actividad no guarda a quién se escribió, sólo el id del \
                 intento. Ver P-17."
            );
        }
    }
    // ═══════════════════════════════════════════════════════════════════════
    // LOS FILTROS DEL REGISTRO · entrega 7.4
    // ═══════════════════════════════════════════════════════════════════════

    /// Mete un intento con su dirección, y devuelve su id.
    ///
    /// Hace falta porque **`activity_event` no guarda direcciones** (P-17): la
    /// del registro sale de `message_attempt` por un JOIN, así que sin intentos
    /// de verdad no se puede probar ni la columna ni el filtro de texto.
    fn intento_para(db: &Db, id: &str, direccion: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                // `stage_id` y no `campaign_stage_id`: la V4 lo llamó así, y el
                // nombre largo es el de `activity_event`. Dos tablas vecinas con
                // el mismo dato y distinto nombre; se escribió mal a la primera.
                "INSERT INTO message_attempt
                     (id, campaign_id, stage_id, channel, contact_address,
                      idempotency_key, state, created_at, updated_at)
                 VALUES (?1, 'k-1', ?2, 'email', ?3, ?1, 'sent', ?4, ?4)",
                params![id, ETAPA, direccion, AHORA],
            )
            .map(|_| ())
        })
        .expect("intento");
    }

    fn desenlace_de<'a>(clave: &'static str, intento: &'a str) -> SucesoNuevo<'a> {
        SucesoNuevo {
            clave,
            es_desenlace: true,
            campana: Some("k-1"),
            etapa: Some(ETAPA),
            canal: Some("email"),
            intento: Some(intento),
            numero_a: None,
            numero_b: None,
        }
    }

    /// **El tope no esconde nada: 500 se sirven, y el total dice 501.**
    ///
    /// ── POR QUÉ 501 Y NO UNO ──
    ///
    /// Porque con un solo registro la prueba pasa aunque el `LIMIT` no exista,
    /// y eso ya nos costó una vez: la prueba de la página de contactos usaba un
    /// contacto, pedir diez mil devolvía uno igualmente, y el recorte de página
    /// se podía quitar entero sin que nada fallara.
    ///
    /// Con 501 las dos mitades se comprueban de verdad: que **llegan 500** —el
    /// `LIMIT` existe— y que el total dice **501** —el tope no esconde el
    /// tamaño real, que es lo que D-7C pide enseñar: «500 de 501»—.
    ///
    /// Verificada rompiéndola el 30/09/2026 quitando el `LIMIT`: llegan 501 y
    /// falla con «el tope de página no se aplicó».
    #[test]
    fn el_tope_sirve_500_y_el_total_dice_cuantos_hay() {
        let (_dir, db) = base();

        for i in 0..501u32 {
            let id = format!("a-{i}");
            intento_para(&db, &id, &format!("p{i}@t.mx"));
            db.anotar_suceso(
                empresa(),
                &desenlace_de("motor.aceptadoPorElProveedor", &id),
            )
            .expect("anota");
        }

        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(
            p.sucesos.len(),
            TOPE_DE_PAGINA as usize,
            "el tope de página no se aplicó"
        );
        assert_eq!(
            p.total, 501,
            "el total tiene que decir cuántos hay de verdad"
        );
    }

    /// **La dirección sale del intento, y no de esta tabla.**
    ///
    /// Es lo que hace que P-17 no se repita aquí: el registro puede enseñar a
    /// quién se le escribió **sin guardar ni una dirección** en una tabla de
    /// sólo-añadir de millones de filas.
    ///
    /// Verificada rompiéndola quitando el `LEFT JOIN`: falla al no poder leer
    /// la columna.
    #[test]
    fn la_direccion_se_lee_del_intento() {
        let (_dir, db) = base();
        intento_para(&db, "a-1", "ana@empresa.mx");
        db.anotar_suceso(
            empresa(),
            &desenlace_de("motor.aceptadoPorElProveedor", "a-1"),
        )
        .expect("anota");

        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(
            primero(&p.sucesos).direccion.as_deref(),
            Some("ana@empresa.mx")
        );

        // Y en la tabla del registro no hay ni rastro de ella.
        let hay: u32 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(*) FROM pragma_table_info('activity_event')
                      WHERE name LIKE '%address%' OR name LIKE '%email%'",
                    [],
                    |r| r.get(0),
                )
            })
            .expect("consulta");
        assert_eq!(
            hay, 0,
            "activity_event no puede tener columnas de dirección"
        );
    }

    /// **Anonimizar el intento borra la dirección del registro, sin tocarlo.**
    ///
    /// ── LA PROMESA DE LA V10, COMPROBADA ──
    ///
    /// `activity_event` prohíbe `UPDATE` con un disparador: un suceso pasó como
    /// pasó. Si la dirección estuviera guardada ahí, el día que alguien ejerza
    /// su derecho de cancelación **esa fila no se podría tocar** — el problema
    /// que P-17 tiene abierto por `consent_entry`.
    ///
    /// Con la dirección leída del intento, anonimizar el intento basta. Esta
    /// prueba lo hace de verdad y comprueba que el suceso sigue ahí, con su
    /// fecha y su clave, y sin la dirección.
    #[test]
    fn anonimizar_el_intento_quita_la_direccion_del_registro() {
        let (_dir, db) = base();
        intento_para(&db, "a-1", "ana@empresa.mx");
        db.anotar_suceso(
            empresa(),
            &desenlace_de("motor.aceptadoPorElProveedor", "a-1"),
        )
        .expect("anota");

        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE message_attempt SET contact_address = '' WHERE id = 'a-1'",
                [],
            )
            .map(|_| ())
        })
        .expect("anonimiza");

        let p = db.sucesos(empresa(), &Filtro::default()).expect("lee");
        assert_eq!(p.total, 1, "el suceso sigue en el registro");
        assert_eq!(
            primero(&p.sucesos).direccion.as_deref(),
            Some(""),
            "la dirección desaparece con el intento, sin tocar activity_event"
        );
        assert_eq!(primero(&p.sucesos).clave, "motor.aceptadoPorElProveedor");
    }

    /// **Buscar una dirección encuentra sus envíos, y sólo los suyos.**
    ///
    /// Verificada rompiéndola quitando la condición de `?7`: devuelve los tres
    /// y falla con el recuento.
    #[test]
    fn el_texto_busca_en_la_direccion() {
        let (_dir, db) = base();
        for (id, dir) in [
            ("a-1", "ana@empresa.mx"),
            ("a-2", "beto@empresa.mx"),
            ("a-3", "ana.lopez@otra.mx"),
        ] {
            intento_para(&db, id, dir);
            db.anotar_suceso(empresa(), &desenlace_de("motor.aceptadoPorElProveedor", id))
                .expect("anota");
        }

        let p = db
            .sucesos(
                empresa(),
                &Filtro {
                    texto: Some("ana"),
                    ..Filtro::default()
                },
            )
            .expect("lee");
        assert_eq!(
            p.total, 2,
            "«ana» casa con las dos direcciones que la llevan"
        );

        let exacto = db
            .sucesos(
                empresa(),
                &Filtro {
                    texto: Some("beto@empresa.mx"),
                    ..Filtro::default()
                },
            )
            .expect("lee");
        assert_eq!(exacto.total, 1);
    }

    /// **Y buscar un texto no devuelve sucesos de bucle.**
    ///
    /// Un «fuera de horario» no es de nadie, así que no puede salir al buscar
    /// una dirección. Sin esto, el `LEFT JOIN` los dejaría pasar —su
    /// `contact_address` es nulo, y un `NULL LIKE '%ana%'` es nulo, no falso—,
    /// que es el error clásico de un JOIN externo con filtro.
    ///
    /// Verificada rompiéndola: poniendo `(a.contact_address IS NULL OR …)` en
    /// la condición, el suceso de bucle aparece y la prueba falla.
    #[test]
    fn buscar_una_direccion_no_devuelve_sucesos_de_bucle() {
        let (_dir, db) = base();
        intento_para(&db, "a-1", "ana@empresa.mx");
        db.anotar_suceso(
            empresa(),
            &desenlace_de("motor.aceptadoPorElProveedor", "a-1"),
        )
        .expect("anota");
        db.anotar_suceso(empresa(), &de_bucle("motor.fueraDeVentana", ETAPA))
            .expect("anota");

        let p = db
            .sucesos(
                empresa(),
                &Filtro {
                    texto: Some("ana"),
                    ..Filtro::default()
                },
            )
            .expect("lee");
        assert_eq!(p.total, 1);
        assert!(p.sucesos.iter().all(|s| s.es_desenlace));
    }

    /// **El filtro de clase devuelve sólo esa clase.**
    #[test]
    fn el_filtro_de_clase_separa_los_rebotes_de_lo_demas() {
        let (_dir, db) = base();
        // Direcciones distintas: `UNIQUE(campaign_id, channel, contact_address)`
        // impide escribirle dos veces a la misma persona en la misma campaña
        // (§55), y está bien que lo impida. La prueba se escribió repitiendo
        // una dirección y fue la invariante la que avisó.
        for (id, clave) in [
            ("a-1", "motor.aceptadoPorElProveedor"),
            ("a-2", "reintento.direccionInexistente"),
            ("a-3", "motor.aceptadoPorElProveedor"),
        ] {
            intento_para(&db, id, &format!("{id}@t.mx"));
            db.anotar_suceso(empresa(), &desenlace_de(clave, id))
                .expect("anota");
        }

        let p = db
            .sucesos(
                empresa(),
                &Filtro {
                    clave: Some("reintento.direccionInexistente"),
                    ..Filtro::default()
                },
            )
            .expect("lee");
        assert_eq!(p.total, 1);
        assert_eq!(primero(&p.sucesos).clave, "reintento.direccionInexistente");
    }

    /// **«Hasta» incluye su propio instante.**
    ///
    /// Quien filtra «hasta el martes» quiere el martes entero. Con un `<` en
    /// vez de un `<=`, el último suceso del día elegido desaparece — y es
    /// justo el que se estaba buscando.
    ///
    /// Verificada rompiéndola: cambiando `<=` por `<`, llegan 1 en vez de 2.
    #[test]
    fn el_hasta_incluye_su_instante() {
        let (_dir, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO activity_event
                     (id, company_id, kind, is_outcome, occurred_at)
                 VALUES ('s-1', '{EMPRESA}', 'motor.parado', 0, '2026-09-28T10:00:00Z'),
                        ('s-2', '{EMPRESA}', 'motor.parado', 0, '2026-09-29T23:59:59Z'),
                        ('s-3', '{EMPRESA}', 'motor.parado', 0, '2026-09-30T00:00:01Z');"
            ))
        })
        .expect("siembra");

        let p = db
            .sucesos(
                empresa(),
                &Filtro {
                    hasta: Some("2026-09-29T23:59:59Z"),
                    ..Filtro::default()
                },
            )
            .expect("lee");
        assert_eq!(p.total, 2, "el instante de «hasta» cuenta");
    }

    /// **Los cuatro criterios se combinan, y el total corresponde a lo servido.**
    ///
    /// Es la garantía que sostiene «500 de 3 412»: el mismo `WHERE` para contar
    /// y para leer. Con dos cadenas parecidas, el total acaba diciendo una cosa
    /// y la tabla otra, que es exactamente lo que D-7C quiere impedir.
    #[test]
    fn los_filtros_se_combinan_y_el_total_corresponde() {
        let (_dir, db) = base();
        // Dos direcciones que contienen «ana» y una que no. No se repite
        // ninguna: la misma dirección no puede tener dos intentos en la misma
        // campaña (§55), así que buscar una persona dentro de una campaña
        // devuelve como mucho un intento — y todos los sucesos de ese intento.
        for (id, dir, clave) in [
            ("a-1", "ana@t.mx", "motor.aceptadoPorElProveedor"),
            ("a-2", "ana.lopez@t.mx", "reintento.direccionInexistente"),
            ("a-3", "beto@t.mx", "motor.aceptadoPorElProveedor"),
        ] {
            intento_para(&db, id, dir);
            db.anotar_suceso(empresa(), &desenlace_de(clave, id))
                .expect("anota");
        }
        db.anotar_suceso(empresa(), &de_bucle("motor.fueraDeVentana", ETAPA))
            .expect("anota");

        let p = db
            .sucesos(
                empresa(),
                &Filtro {
                    campana: Some("k-1"),
                    texto: Some("ana"),
                    clave: Some("motor.aceptadoPorElProveedor"),
                    solo_desenlaces: Some(true),
                    ..Filtro::default()
                },
            )
            .expect("lee");
        assert_eq!(p.total, 1, "sólo el aceptado de ana");
        assert_eq!(
            p.total as usize,
            p.sucesos.len(),
            "el total y lo servido tienen que salir del mismo WHERE"
        );
    }

    /// **El desplegable ofrece lo que hay, con su cifra, y lo más frecuente
    /// primero.**
    ///
    /// Una opción vacía en un filtro es peor que no tenerla: quien la elige y
    /// no ve nada no sabe si es que no pasó o si el filtro está roto.
    #[test]
    fn las_clases_salen_de_los_datos_y_no_de_una_lista() {
        let (_dir, db) = base();
        // ── LA FRECUENTE ES LA SEGUNDA POR ORDEN ALFABÉTICO, A PROPÓSITO ──
        //
        // Con `motor.*` como la más repetida, ordenar por frecuencia y ordenar
        // por nombre dan **el mismo resultado**, y la prueba pasaba con las
        // dos: no verificaba nada del orden. Se vio al romperla (§2). Con
        // `reintento.*` como la frecuente, los dos órdenes difieren y la
        // aserción distingue.
        for _ in 0..3 {
            db.anotar_suceso(empresa(), &desenlace("reintento.direccionInexistente"))
                .expect("anota");
        }
        db.anotar_suceso(empresa(), &desenlace("motor.aceptadoPorElProveedor"))
            .expect("anota");

        let clases = db.clases_de_suceso(empresa()).expect("lee");
        assert_eq!(
            clases,
            vec![
                ("reintento.direccionInexistente".to_owned(), 3),
                ("motor.aceptadoPorElProveedor".to_owned(), 1),
            ],
            "ordenadas por frecuencia: lo que más pasa es lo que más se busca"
        );
        assert!(
            !clases.iter().any(|(_, n)| *n == 0),
            "no puede ofrecerse una clase sin sucesos"
        );
    }
}
