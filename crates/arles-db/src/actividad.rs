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
}

/// Cuántos sucesos se sirven de una vez (D-7C).
///
/// **Siempre, no a partir de un umbral.** Una campaña de 3 000 ya produce una
/// tabla inmanejable a ojo y no llegaría a ningún umbral razonable; y una
/// pantalla que cambia de comportamiento según el tamaño se siente rota. El
/// total va al lado para que el tope no esconda nada: «500 de 3 412».
pub const TOPE_DE_PAGINA: u32 = 500;

/// Qué se lee, y de dónde.
#[derive(Debug, Clone, Default)]
pub struct Filtro<'a> {
    /// Sólo esta campaña. Con tres a la vez, es el primer filtro que hace falta.
    pub campana: Option<&'a str>,
    /// Sólo desenlaces de envío, o sólo sucesos de bucle.
    pub solo_desenlaces: Option<bool>,
    /// Desde esta marca de tiempo, inclusive.
    pub desde: Option<&'a str>,
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
        self.con(|conn| {
            let empresa = empresa.to_string();
            // El mismo `WHERE` para contar y para leer, construido una sola
            // vez. Dos cadenas parecidas es como se acaba enseñando «500 de
            // 3 412» con un total que no corresponde a lo que se ve.
            let donde = "company_id = ?1
                     AND (?2 IS NULL OR campaign_id = ?2)
                     AND (?3 IS NULL OR is_outcome = ?3)
                     AND (?4 IS NULL OR occurred_at >= ?4)";
            let atados = params![
                empresa,
                f.campana,
                f.solo_desenlaces.map(i64::from),
                f.desde
            ];

            let total: u32 = conn.query_row(
                &format!("SELECT count(*) FROM activity_event WHERE {donde}"),
                atados,
                |r| r.get(0),
            )?;

            let mut consulta = conn.prepare(&format!(
                "SELECT id, kind, is_outcome, campaign_id, channel,
                        message_attempt_id, number_a, number_b, occurred_at
                   FROM activity_event
                  WHERE {donde}
                  ORDER BY occurred_at DESC, rowid DESC
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
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Pagina { sucesos, total })
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
}
