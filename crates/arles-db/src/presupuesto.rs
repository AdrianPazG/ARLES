//! El presupuesto de envíos, guardado en disco.
//!
//! Ver `MOTOR_DE_EJECUCION.md` §4. Las **reglas** viven en
//! `arles_core::ritmo`; aquí está sólo el recuento.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ NO PUEDE SER UN CONTADOR EN MEMORIA
//!
//! Un contador en memoria se reinicia al cerrar la aplicación. Con él, gastar
//! el límite diario varias veces el mismo día sería trivial y **ni siquiera
//! haría falta mala intención**: basta cerrar ARLES y volver a abrirlo.
//!
//! El usuario no vería nada raro —la aplicación diría que lleva cero— y el
//! proveedor sí: tres veces el límite diario desde la misma cuenta es
//! exactamente el patrón que marca una cuenta como spam.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! CÓMO SE REINICIA SIN GUARDAR CUÁNDO TOCA
//!
//! No hay ninguna tarea que borre contadores a media noche. En su lugar, cada
//! fila guarda **a qué hora o a qué día pertenece** (`window_start`), y al
//! consumir se compara esa etiqueta con la actual:
//!
//!   · Misma etiqueta → se suma uno.
//!   · Etiqueta distinta → empezó otra ventana: el contador vuelve a **uno**.
//!
//! Así el reinicio ocurre solo, sin relojes, sin tareas programadas y sin que
//! importe cuánto tiempo estuvo el programa cerrado. Un contador que dependiera
//! de que algo se ejecute a las 00:00 fallaría exactamente en el caso en que
//! más importa: con el equipo apagado.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS ETIQUETAS SE LEEN EN LA ZONA DE LA EMPRESA
//!
//! `arles_core::ventana::clave_de_dia` las calcula, y explica ahí por qué la
//! zona es la de la empresa y no la de cada campaña: si cada campaña midiera su
//! propio «hoy», el mismo buzón tendría dos días en curso a la vez y los cupos
//! se solaparían. Eso **no cambia** con la V13: el cubo pasó a ser de la
//! campaña, pero el calendario con el que se cuenta sigue siendo uno solo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UN CUBO POR CAMPAÑA, NO POR BUZÓN (D-13, 22/10/2026)
//!
//! Hasta la V13 el cubo colgaba de la **cuenta remitente**: dos campañas sobre
//! el mismo buzón compartían los cincuenta diarios. Era un techo real para el
//! dominio, **y escondía un plazo roto**: el paso 6 le promete al cliente los
//! días calculados con el límite que él escribió, y activar una segunda
//! campaña los duplicaba en silencio.
//!
//! Dirección decidió que se sumen, y la razón no es de volumen: *«el cliente
//! está colocando que se envíen 100 en cada campaña… para respetar el tiempo
//! que el cliente calculó»*. ARLES no puede prometer un plazo y entregar la
//! mitad.
//!
//! **Y no bastaba con sumar los límites.** Un cubo único de 200 deja que la
//! campaña que el motor atienda primero se gaste los 200 y la otra se quede en
//! cero: el total cuadra y el plazo de la segunda sigue roto, que es el
//! problema de verdad. Con un cubo por etapa, cada campaña respeta su número y
//! **el total es la suma por construcción** — nadie tiene que calcularla.
//!
//! Lo que esto quita está dicho en la V13 y en el reporte de reputación: el
//! agregado de un buzón **ya no tiene techo**. Lo que ocupa su sitio es el
//! semáforo de Remitentes (D-14), que para eso existe
//! [`Db::gastado_hoy_por_el_buzon`].

use arles_core::ritmo::Gastado;
use rusqlite::{Connection, OptionalExtension, params};

use crate::db::Db;
use crate::error::DbError;

/// Las dos ventanas que corren a la vez, tal como las nombra el esquema.
const HORARIA: &str = "hourly";
const DIARIA: &str = "daily";

impl Db {
    /// Cuánto lleva gastado una cuenta remitente en la hora y el día actuales.
    ///
    /// `clave_de_hora` y `clave_de_dia` salen de `arles_core::ventana`, ya
    /// calculadas en la zona de la empresa.
    ///
    /// Una fila cuya etiqueta no coincide con la actual **cuenta como cero**:
    /// pertenece a una ventana que ya pasó. No se borra —se sobreescribirá al
    /// primer envío de la nueva—, porque borrarla obligaría a escribir para
    /// poder leer.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn gastado_de(
        &self,
        cuenta: &str,
        etapa: &str,
        clave_de_hora: &str,
        clave_de_dia: &str,
    ) -> Result<Gastado, DbError> {
        self.con(|conn| {
            Ok(Gastado {
                en_esta_hora: leer(conn, cuenta, etapa, HORARIA, clave_de_hora)?,
                en_este_dia: leer(conn, cuenta, etapa, DIARIA, clave_de_dia)?,
            })
        })
    }

    /// Cuánto lleva gastado **un buzón hoy, entre todas sus campañas**.
    ///
    /// Es la cifra del semáforo de Remitentes (D-14), y es la suma de los cubos
    /// de sus etapas: desde la V13 cada campaña lleva el suyo, así que el gasto
    /// del buzón ya no se lee, se suma.
    ///
    /// Las filas de una ventana que ya pasó **no se cuentan**, con el mismo
    /// `>=` que usa [`leer`] y por la misma razón.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn gastado_hoy_por_el_buzon(
        &self,
        cuenta: &str,
        clave_de_dia: &str,
    ) -> Result<u32, DbError> {
        self.con(|conn| {
            let total: Option<i64> = conn.query_row(
                "SELECT SUM(consumed)
                   FROM rate_budget
                  WHERE email_account_id = ?1
                    AND window_kind = ?2
                    AND window_start >= ?3",
                params![cuenta, DIARIA, clave_de_dia],
                |f| f.get(0),
            )?;
            Ok(total
                .map(|c| u32::try_from(c).unwrap_or(u32::MAX))
                .unwrap_or(0))
        })
    }

    /// Anota un envío y devuelve cómo queda el presupuesto.
    ///
    /// **Se llama después de que el proveedor acepte**, no antes. Un permiso
    /// gastado por un envío que no llegó a salir hace que la campaña rinda
    /// menos de lo que el usuario configuró, y esa diferencia no se puede
    /// explicar mirando la pantalla.
    ///
    /// Las dos ventanas se suman **en la misma transacción**. Si se escribieran
    /// por separado y el proceso muriera en medio, quedaría la hora contada y
    /// el día no —o al revés—, y el presupuesto mentiría en la dirección
    /// peligrosa: permitir de más.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn anotar_un_envio(
        &self,
        cuenta: &str,
        etapa: &str,
        clave_de_hora: &str,
        clave_de_dia: &str,
        ahora: &str,
    ) -> Result<Gastado, DbError> {
        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let en_esta_hora = sumar_uno(&tx, cuenta, etapa, HORARIA, clave_de_hora, ahora)?;
            let en_este_dia = sumar_uno(&tx, cuenta, etapa, DIARIA, clave_de_dia, ahora)?;
            tx.commit()?;
            Ok(Gastado {
                en_esta_hora,
                en_este_dia,
            })
        })
    }
}

/// Lo gastado en una ventana, o cero si la fila es de una ventana pasada.
///
/// El `>=` acompaña al arreglo del reloj atrasado de [`sumar_uno`]: si la fila
/// guardada es de una ventana **posterior** a la que se pregunta —el reloj se
/// movió hacia atrás—, se devuelve lo que hay en vez de cero. Devolver cero
/// diría que el presupuesto está entero cuando no lo está, y sería la misma
/// puerta por el lado de la lectura.
fn leer(
    conn: &Connection,
    cuenta: &str,
    etapa: &str,
    clase: &str,
    clave: &str,
) -> Result<u32, rusqlite::Error> {
    let consumido: Option<i64> = conn
        .query_row(
            "SELECT consumed
               FROM rate_budget
              WHERE email_account_id = ?1
                AND stage_id = ?2
                AND window_kind = ?3
                AND window_start >= ?4",
            params![cuenta, etapa, clase, clave],
            |f| f.get(0),
        )
        .optional()?;
    Ok(consumido
        .map(|c| u32::try_from(c).unwrap_or(u32::MAX))
        .unwrap_or(0))
}

/// Suma uno a una ventana, reiniciando **sólo si la etiqueta es más nueva**.
///
/// ```text
/// consumed = CASE WHEN ?nueva > window_start THEN 1 ELSE consumed + 1 END
/// ```
///
/// Tres casos, y los tres importan:
///
/// **Etiqueta más nueva** → empieza de nuevo en uno. No en cero, porque este
/// envío ya cuenta. Sin esto, el gasto de ayer se arrastraría al día de hoy
/// para siempre y la cuenta se quedaría sin presupuesto el segundo día.
///
/// **La misma etiqueta** → se suma. Es el caso normal.
///
/// **Etiqueta más VIEJA** → se suma igual, y la fila conserva la etiqueta
/// nueva. Esto es el arreglo del 23/09/2026, y tapa el caso «reloj del sistema
/// ajustado hacia atrás» de `ESTRATEGIA_QA.md` §3, que estaba sin cubrir.
///
/// Con la versión anterior —`window_start = excluded.window_start` a secas—
/// bastaba con **atrasar el reloj un día** para que la etiqueta cambiara, el
/// contador se reiniciara y la cuenta mandara el cupo diario otra vez. Mover el
/// reloj era una forma de saltarse el límite del proveedor, y no hace falta
/// mala fe: un ajuste de NTP tras una deriva hace exactamente eso.
///
/// Se sigue gastando del presupuesto que había, que es el lado conservador —se
/// envía de menos, nunca de más— y todo se recoloca solo en cuanto el reloj
/// vuelve a su sitio.
fn sumar_uno(
    conn: &Connection,
    cuenta: &str,
    etapa: &str,
    clase: &str,
    clave: &str,
    ahora: &str,
) -> Result<u32, rusqlite::Error> {
    conn.execute(
        "INSERT INTO rate_budget
             (channel, email_account_id, whatsapp_account_id, stage_id,
              window_kind, window_start, consumed, updated_at)
         VALUES ('email', ?1, NULL, ?2, ?3, ?4, 1, ?5)
         ON CONFLICT(email_account_id, stage_id, window_kind)
            WHERE email_account_id IS NOT NULL AND stage_id IS NOT NULL
         DO UPDATE SET
             consumed     = CASE WHEN excluded.window_start > window_start
                                 THEN 1
                                 ELSE consumed + 1
                            END,
             -- MAX y no `excluded` a secas: una etiqueta más vieja no puede
             -- hacer retroceder la ventana, o el siguiente envío la vería como
             -- «más nueva» y reiniciaría el contador.
             window_start = MAX(window_start, excluded.window_start),
             updated_at   = excluded.updated_at",
        params![cuenta, etapa, clase, clave, ahora],
    )?;
    leer(conn, cuenta, etapa, clase, clave)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const AHORA: &str = "2026-09-23T20:00:00Z";
    const HORA: &str = "2026-09-23T14";
    const DIA: &str = "2026-09-23";
    /// Las dos etapas que comparten el buzón `cuenta`, y la del buzón `otra`.
    /// Desde la V13 el presupuesto cuelga de la etapa (D-13).
    const ETAPA_A: &str = "etapa-a";
    const ETAPA_B: &str = "etapa-b";
    const ETAPA_C: &str = "etapa-c";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('emp', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', ?1, ?1)",
                params![AHORA],
            )?;
            c.execute(
                "INSERT INTO email_account
                     (id, company_id, display_name, email_address, provider_kind,
                      credential_ref, status, daily_limit, hourly_limit,
                      created_at, updated_at)
                 VALUES ('cuenta', 'emp', 'Envíos', 'envios@t.mx', 'smtp',
                         'llavero:cuenta', 'connected', 50, 10, ?1, ?1)",
                params![AHORA],
            )?;
            // ── DOS CAMPAÑAS SOBRE EL MISMO BUZÓN ─────────────────────────
            //
            // Desde la V13 el cubo cuelga de la etapa, con clave ajena, así que
            // sembrarlas no es decorado: sin ellas, anotar un envío falla con
            // `FOREIGN KEY constraint failed` y ninguna prueba de este módulo
            // correría. Y son DOS porque lo que hay que poder ver de la D-13 es
            // que cada campaña lleva su cuenta aparte sobre el mismo remitente.
            for (campana, etapa) in [("camp-a", ETAPA_A), ("camp-b", ETAPA_B)] {
                c.execute(
                    "INSERT INTO campaign (id, company_id, name, status,
                                           created_at, updated_at)
                     VALUES (?1, 'emp', ?1, 'running', ?2, ?2)",
                    params![campana, AHORA],
                )?;
                c.execute(
                    "INSERT INTO campaign_stage
                         (id, campaign_id, position, channel, email_account_id,
                          status, created_at, updated_at)
                     VALUES (?1, ?2, 1, 'email', 'cuenta', 'running', ?3, ?3)",
                    params![etapa, campana, AHORA],
                )?;
            }
            Ok(1)
        })
        .expect("siembra");
        (dir, db)
    }

    #[test]
    fn una_cuenta_nueva_no_ha_gastado_nada() {
        let (_d, db) = base();
        assert_eq!(
            db.gastado_de("cuenta", ETAPA_A, HORA, DIA).expect("lee"),
            Gastado::default()
        );
    }

    #[test]
    fn cada_envio_gasta_de_las_dos_ventanas() {
        let (_d, db) = base();
        let g = db
            .anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
            .expect("anota");
        assert_eq!(
            g,
            Gastado {
                en_esta_hora: 1,
                en_este_dia: 1
            },
            "un envío gasta un permiso de la hora Y uno del día"
        );

        let g = db
            .anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
            .expect("anota");
        assert_eq!(g.en_esta_hora, 2);
        assert_eq!(g.en_este_dia, 2);

        // Y se lee igual desde fuera.
        assert_eq!(db.gastado_de("cuenta", ETAPA_A, HORA, DIA).expect("lee"), g);
    }

    /// **Lo gastado sobrevive a cerrar y volver a abrir el programa.**
    ///
    /// Es la razón de existir del módulo. Con un contador en memoria, cerrar
    /// ARLES y reabrirlo dejaría el gasto en cero, y el límite diario se podría
    /// gastar varias veces el mismo día sin que nadie hiciera nada raro.
    ///
    /// Se cierra la base de verdad y se vuelve a abrir el mismo archivo: es lo
    /// que hace el usuario al cerrar la ventana.
    #[test]
    fn lo_gastado_sobrevive_a_cerrar_el_programa() {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join("arles.db");
        let clave = ClaveMaestra::generar().expect("genera");

        {
            let db = Db::abrir(&ruta, &clave).expect("abre");
            db.ejecutar_en_pruebas(|c| {
                c.execute(
                    "INSERT INTO company (id, commercial_name, country, timezone,
                                          corporate_email, created_at, updated_at)
                     VALUES ('emp', 'T', 'MX', 'America/Mexico_City', 'h@t.mx', ?1, ?1)",
                    params![AHORA],
                )?;
                c.execute(
                    "INSERT INTO email_account
                         (id, company_id, display_name, email_address, provider_kind,
                          credential_ref, status, daily_limit, hourly_limit,
                          created_at, updated_at)
                     VALUES ('cuenta', 'emp', 'E', 'e@t.mx', 'smtp',
                             'llavero:cuenta', 'connected', 50, 10, ?1, ?1)",
                    params![AHORA],
                )?;
                // Su campaña y su etapa: el cubo cuelga de la etapa (V13).
                c.execute(
                    "INSERT INTO campaign (id, company_id, name, status,
                                           created_at, updated_at)
                     VALUES ('camp-a', 'emp', 'camp-a', 'running', ?1, ?1)",
                    params![AHORA],
                )?;
                c.execute(
                    "INSERT INTO campaign_stage
                         (id, campaign_id, position, channel, email_account_id,
                          status, created_at, updated_at)
                     VALUES (?1, 'camp-a', 1, 'email', 'cuenta', 'running', ?2, ?2)",
                    params![ETAPA_A, AHORA],
                )
            })
            .expect("siembra");

            for _ in 0..7 {
                db.anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
                    .expect("anota");
            }
        } // aquí se cierra la base, como al cerrar la ventana

        let db = Db::abrir(&ruta, &clave).expect("reabre");
        assert_eq!(
            db.gastado_de("cuenta", ETAPA_A, HORA, DIA).expect("lee"),
            Gastado {
                en_esta_hora: 7,
                en_este_dia: 7
            },
            "al reabrir, el gasto tiene que seguir ahí. Si volviera a cero, el \
             límite diario se podría gastar tantas veces como veces se cierre \
             la aplicación"
        );
    }

    /// **Al cambiar de hora, el cubo horario se reinicia y el diario no.**
    ///
    /// Es la prueba que distingue las dos ventanas. Si el diario se reiniciara
    /// con la hora, el límite de 50 al día sería en realidad 50 por hora.
    #[test]
    fn al_cambiar_de_hora_se_reinicia_la_hora_pero_no_el_dia() {
        let (_d, db) = base();
        for _ in 0..5 {
            db.anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
                .expect("anota");
        }

        const HORA_SIGUIENTE: &str = "2026-09-23T15";
        let g = db
            .anotar_un_envio("cuenta", ETAPA_A, HORA_SIGUIENTE, DIA, AHORA)
            .expect("anota");

        assert_eq!(
            g.en_esta_hora, 1,
            "empezó otra hora: el cubo horario vuelve a uno, no a seis"
        );
        assert_eq!(
            g.en_este_dia, 6,
            "pero sigue siendo el mismo día: el diario SUMA. Si se reiniciara \
             aquí, un límite de 50 al día serían 50 cada hora"
        );
    }

    /// Al cambiar de día se reinician las dos.
    #[test]
    fn al_cambiar_de_dia_se_reinician_las_dos() {
        let (_d, db) = base();
        for _ in 0..9 {
            db.anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
                .expect("anota");
        }

        let g = db
            .anotar_un_envio("cuenta", ETAPA_A, "2026-09-24T09", "2026-09-24", AHORA)
            .expect("anota");
        assert_eq!(
            g,
            Gastado {
                en_esta_hora: 1,
                en_este_dia: 1
            },
            "día nuevo, cuentas nuevas"
        );
    }

    /// Una ventana pasada se lee como cero sin necesidad de borrarla.
    ///
    /// El reinicio ocurre **al comparar**, no porque nadie limpie a medianoche.
    /// Un contador que dependiera de una tarea a las 00:00 fallaría justo en el
    /// caso que más importa: con el equipo apagado.
    #[test]
    fn una_ventana_pasada_se_lee_como_cero() {
        let (_d, db) = base();
        db.anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
            .expect("anota");

        assert_eq!(
            db.gastado_de("cuenta", ETAPA_A, "2026-09-23T15", "2026-09-24")
                .expect("lee"),
            Gastado::default(),
            "las etiquetas no coinciden: esa hora y ese día empiezan de cero"
        );
    }

    /// Cada cuenta lleva su propio presupuesto.
    ///
    /// El límite lo pone el proveedor **por cuenta**. Un contador compartido
    /// haría que una cuenta agotara el permiso de la otra.
    #[test]
    fn cada_cuenta_lleva_su_propia_cuenta() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO email_account
                     (id, company_id, display_name, email_address, provider_kind,
                      credential_ref, status, daily_limit, hourly_limit,
                      created_at, updated_at)
                 VALUES ('otra', 'emp', 'Otra', 'otra@t.mx', 'smtp',
                         'llavero:otra', 'connected', 50, 10, ?1, ?1)",
                params![AHORA],
            )?;
            // Su etapa, porque desde la V13 el cubo cuelga de ella.
            c.execute(
                "INSERT INTO campaign (id, company_id, name, status,
                                       created_at, updated_at)
                 VALUES ('camp-c', 'emp', 'camp-c', 'running', ?1, ?1)",
                params![AHORA],
            )?;
            c.execute(
                "INSERT INTO campaign_stage
                     (id, campaign_id, position, channel, email_account_id,
                      status, created_at, updated_at)
                 VALUES (?1, 'camp-c', 1, 'email', 'otra', 'running', ?2, ?2)",
                params![ETAPA_C, AHORA],
            )
        })
        .expect("otra cuenta");

        for _ in 0..4 {
            db.anotar_un_envio("cuenta", ETAPA_A, HORA, DIA, AHORA)
                .expect("anota");
        }

        assert_eq!(
            db.gastado_de("otra", ETAPA_C, HORA, DIA).expect("lee"),
            Gastado::default(),
            "el gasto de una cuenta no puede restarle permisos a otra"
        );
        assert_eq!(
            db.gastado_de("cuenta", ETAPA_A, HORA, DIA)
                .expect("lee")
                .en_este_dia,
            4
        );
    }

    /// Sólo queda **una** fila por cuenta y ventana, pase lo que pase.
    ///
    /// Sin el índice único y el `ON CONFLICT`, cada envío insertaría una fila
    /// nueva: el contador leería la primera que encontrara y el presupuesto
    /// diría uno para siempre.
    #[test]
    fn no_se_acumulan_filas_por_cada_envio() {
        let (_d, db) = base();
        for h in ["2026-09-23T14", "2026-09-23T15", "2026-09-23T16"] {
            for _ in 0..3 {
                db.anotar_un_envio("cuenta", ETAPA_A, h, DIA, AHORA)
                    .expect("anota");
            }
        }

        let filas: i64 = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT count(*) FROM rate_budget WHERE email_account_id = 'cuenta'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("cuenta filas");
        assert_eq!(
            filas, 2,
            "una fila para la ventana horaria y otra para la diaria, y nada más"
        );
    }

    /// **La defensa que sustituye al `min` que se quitó el 22/10/2026.**
    ///
    /// Desde esa fecha el límite de la campaña manda y puede superar el valor
    /// de la cuenta (ver la cabecera de `trabajo.rs`). La pregunta que eso
    /// abre: si dos campañas de 300 comparten remitente, ¿salen 600 de esa
    /// cuenta en un día?
    ///
    /// **No, y es por esto:** el presupuesto cuelga del `email_account_id`, no
    /// de la campaña. Las dos escriben en el mismo contador, así que el total
    /// de una cuenta en un día es el mayor de los límites de sus campañas —
    /// nunca la suma.
    ///
    /// Se prueba desde los envíos y no leyendo el esquema: lo que hay que poder
    /// afirmar es que **el contador que las dos consultan es uno solo**, y eso
    /// se ve sumando por los dos lados y leyendo una vez.
    ///
    /// Verificada rompiéndola: dando a cada campaña su propia cuenta
    /// —`anotar_un_envio("cuenta-b", …)` en el segundo bucle—, el contador de
    /// «cuenta» se queda en 4 y la prueba cae con «el presupuesto no es
    /// compartido».
    #[test]
    fn dos_campanas_sobre_una_cuenta_no_suman_sus_topes() {
        let (_d, db) = base();

        // Cuatro envíos de la campaña A y tres de la B, misma cuenta.
        for _ in 0..4 {
            db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-23T14", DIA, AHORA)
                .expect("campaña A");
        }
        for _ in 0..3 {
            db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-23T14", DIA, AHORA)
                .expect("campaña B");
        }

        let gastado = db
            .gastado_de("cuenta", ETAPA_A, "2026-09-23T14", DIA)
            .expect("lectura");
        assert_eq!(
            gastado.en_este_dia, 7,
            "el presupuesto no es compartido: cada campaña estaría contando aparte"
        );
    }
}

#[cfg(test)]
mod tests_de_la_auditoria_de_la_fase_4 {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const AHORA: &str = "2026-09-23T20:00:00Z";
    /// Las mismas dos etapas del buzón compartido que el módulo de arriba:
    /// desde la V13 el presupuesto cuelga de la etapa (D-13).
    const ETAPA_A: &str = "etapa-a";
    const ETAPA_B: &str = "etapa-b";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES ('emp', 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@t.mx', ?1, ?1)",
                params![AHORA],
            )?;
            c.execute(
                "INSERT INTO email_account
                     (id, company_id, display_name, email_address, provider_kind,
                      credential_ref, status, daily_limit, hourly_limit,
                      created_at, updated_at)
                 VALUES ('cuenta', 'emp', 'Envíos', 'envios@t.mx', 'smtp',
                         'llavero:cuenta', 'connected', 50, 10, ?1, ?1)",
                params![AHORA],
            )?;
            // ── DOS CAMPAÑAS SOBRE EL MISMO BUZÓN ─────────────────────────
            //
            // Desde la V13 el cubo cuelga de la etapa, con clave ajena, así que
            // sembrarlas no es decorado: sin ellas, anotar un envío falla con
            // `FOREIGN KEY constraint failed` y ninguna prueba de este módulo
            // correría. Y son DOS porque lo que hay que poder ver de la D-13 es
            // que cada campaña lleva su cuenta aparte sobre el mismo remitente.
            for (campana, etapa) in [("camp-a", ETAPA_A), ("camp-b", ETAPA_B)] {
                c.execute(
                    "INSERT INTO campaign (id, company_id, name, status,
                                           created_at, updated_at)
                     VALUES (?1, 'emp', ?1, 'running', ?2, ?2)",
                    params![campana, AHORA],
                )?;
                c.execute(
                    "INSERT INTO campaign_stage
                         (id, campaign_id, position, channel, email_account_id,
                          status, created_at, updated_at)
                     VALUES (?1, ?2, 1, 'email', 'cuenta', 'running', ?3, ?3)",
                    params![etapa, campana, AHORA],
                )?;
            }
            Ok(1)
        })
        .expect("siembra");
        (dir, db)
    }

    /// **LA PUERTA DE SALIDA DE LA FASE 4: dormir no produce una ráfaga.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// ESTA PRUEBA EXISTE PORQUE LA ANTERIOR CONFIRMABA EL FALLO
    ///
    /// La garantía estaba «probada» en `arles_core::ritmo`, con una función
    /// que al detectar una suspensión devolvía el presupuesto a cero. Su
    /// prueba se llamaba `ocho_horas_dormido_no_producen_una_rafaga` y pasaba.
    ///
    /// Se descubrió el 23/09/2026 auditando la fase, con el caso que nadie
    /// había puesto: **una siesta dentro del mismo día**. El equipo duerme de
    /// 10:00 a 18:00 con el cupo diario agotado, despierta, el contador vuelve
    /// a cero, y esa cuenta manda otros cincuenta. **Cien envíos en un día con
    /// un tope de cincuenta** — la ráfaga exacta que la fase existe para
    /// impedir, escrita dentro de su propia solución.
    ///
    /// Aquí se mide donde la garantía vive de verdad: el presupuesto
    /// persistido por etiquetas. El día no se arrastra en memoria, se consulta
    /// por su etiqueta, y la etiqueta de un día no cambia porque alguien cierre
    /// la tapa del portátil.
    #[test]
    fn dormir_ocho_horas_no_devuelve_el_cupo_del_dia() {
        let (_d, db) = base();

        // A las 10:00 de México se agota el cupo del día.
        for _ in 0..50 {
            db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-23T10", "2026-09-23", AHORA)
                .expect("gasta");
        }
        let antes = db
            .gastado_de("cuenta", ETAPA_A, "2026-09-23T10", "2026-09-23")
            .expect("lee");
        assert_eq!(antes.en_este_dia, 50);

        // El equipo duerme ocho horas. Al despertar son las 18:00 **del mismo
        // día**: hora nueva, día viejo.
        let despues = db
            .gastado_de("cuenta", ETAPA_A, "2026-09-23T18", "2026-09-23")
            .expect("lee");

        assert_eq!(
            despues.en_esta_hora, 0,
            "la hora es otra, así que su contador empieza en cero. Eso es \
             correcto y no es la ráfaga"
        );
        assert_eq!(
            despues.en_este_dia, 50,
            "SIGUE SIENDO EL MISMO DÍA. Si esto fuera 0, esa cuenta podría \
             mandar otros cincuenta correos: cien en un día con un tope de \
             cincuenta, que es exactamente lo que el §53 prohíbe"
        );
    }

    /// Y al día siguiente sí empieza de cero, que es lo que debe pasar.
    #[test]
    fn al_dia_siguiente_el_cupo_vuelve_entero() {
        let (_d, db) = base();
        for _ in 0..50 {
            db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-23T10", "2026-09-23", AHORA)
                .expect("gasta");
        }
        let manana = db
            .gastado_de("cuenta", ETAPA_A, "2026-09-24T09", "2026-09-24")
            .expect("lee");
        assert_eq!(manana.en_este_dia, 0);
    }

    /// **Atrasar el reloj no regala permisos.**
    ///
    /// Es el caso «reloj del sistema ajustado hacia atrás» de
    /// `ESTRATEGIA_QA.md` §3, y estaba sin cubrir. Pasa de verdad: un ajuste de
    /// NTP tras una deriva, o alguien corrigiendo la hora a mano.
    ///
    /// Sin esta protección, retrasar el reloj un día cambia la etiqueta del
    /// día, el contador se reinicia, y la cuenta manda el cupo entero otra vez.
    /// Basta con mover el reloj para saltarse el límite del proveedor.
    ///
    /// La regla: **una etiqueta más vieja que la guardada no reinicia nada.**
    /// Se sigue gastando del presupuesto que había, que es el lado conservador,
    /// y todo vuelve a la normalidad solo en cuanto el reloj se recoloca.
    ///
    /// Verificada rompiéndola el 23/09/2026: sin la comparación de etiquetas,
    /// el contador del día vuelve a 1 y esta prueba falla.
    #[test]
    fn atrasar_el_reloj_no_regala_el_cupo_del_dia() {
        let (_d, db) = base();
        for _ in 0..50 {
            db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-23T10", "2026-09-23", AHORA)
                .expect("gasta");
        }

        // Alguien mueve el reloj un día hacia atrás.
        let gastado = db
            .anotar_un_envio("cuenta", ETAPA_A, "2026-09-22T10", "2026-09-22", AHORA)
            .expect("anota");

        assert_eq!(
            gastado.en_este_dia, 51,
            "una etiqueta más vieja no reinicia el contador: mover el reloj \
             hacia atrás no puede ser una forma de saltarse el límite diario"
        );
    }

    /// Y con el reloj recolocado, todo sigue como debía.
    #[test]
    fn recolocar_el_reloj_no_deja_el_presupuesto_roto() {
        let (_d, db) = base();
        db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-23T10", "2026-09-23", AHORA)
            .expect("gasta");
        db.anotar_un_envio("cuenta", ETAPA_A, "2026-09-22T10", "2026-09-22", AHORA)
            .expect("con el reloj atrasado");

        let hoy = db
            .gastado_de("cuenta", ETAPA_A, "2026-09-23T10", "2026-09-23")
            .expect("lee");
        assert_eq!(
            hoy.en_este_dia, 2,
            "la fila sigue etiquetada con el día bueno y con todo lo gastado"
        );
    }
}
