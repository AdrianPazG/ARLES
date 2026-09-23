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
//! zona es la de la empresa y no la de cada campaña: el presupuesto cuelga de
//! la **cuenta remitente**, y una cuenta puede servir a varias campañas. Si
//! cada una midiera su propio «hoy», la misma cuenta tendría dos días en curso
//! a la vez y el límite diario se podría gastar dos veces.

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
        clave_de_hora: &str,
        clave_de_dia: &str,
    ) -> Result<Gastado, DbError> {
        self.con(|conn| {
            Ok(Gastado {
                en_esta_hora: leer(conn, cuenta, HORARIA, clave_de_hora)?,
                en_este_dia: leer(conn, cuenta, DIARIA, clave_de_dia)?,
            })
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
        clave_de_hora: &str,
        clave_de_dia: &str,
        ahora: &str,
    ) -> Result<Gastado, DbError> {
        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let en_esta_hora = sumar_uno(&tx, cuenta, HORARIA, clave_de_hora, ahora)?;
            let en_este_dia = sumar_uno(&tx, cuenta, DIARIA, clave_de_dia, ahora)?;
            tx.commit()?;
            Ok(Gastado {
                en_esta_hora,
                en_este_dia,
            })
        })
    }
}

/// Lo gastado en una ventana, o cero si la fila es de una ventana pasada.
fn leer(conn: &Connection, cuenta: &str, clase: &str, clave: &str) -> Result<u32, rusqlite::Error> {
    let consumido: Option<i64> = conn
        .query_row(
            "SELECT consumed
               FROM rate_budget
              WHERE email_account_id = ?1
                AND window_kind = ?2
                AND window_start = ?3",
            params![cuenta, clase, clave],
            |f| f.get(0),
        )
        .optional()?;
    Ok(consumido
        .map(|c| u32::try_from(c).unwrap_or(u32::MAX))
        .unwrap_or(0))
}

/// Suma uno a una ventana, reiniciando si la etiqueta cambió.
///
/// El `CASE` es el reinicio entero, y merece leerse despacio:
///
///     consumed = CASE WHEN window_start = ?nueva THEN consumed + 1 ELSE 1 END
///
/// Si la fila ya era de esta hora (o de este día), se suma. Si era de otra,
/// **se empieza de nuevo en uno** — no en cero, porque este envío ya cuenta.
///
/// Un `consumed + 1` sin el `CASE` arrastraría el gasto de ayer al día de hoy
/// para siempre, y la cuenta se quedaría sin presupuesto el segundo día.
fn sumar_uno(
    conn: &Connection,
    cuenta: &str,
    clase: &str,
    clave: &str,
    ahora: &str,
) -> Result<u32, rusqlite::Error> {
    conn.execute(
        "INSERT INTO rate_budget
             (channel, email_account_id, whatsapp_account_id,
              window_kind, window_start, consumed, updated_at)
         VALUES ('email', ?1, NULL, ?2, ?3, 1, ?4)
         ON CONFLICT(email_account_id, window_kind)
            WHERE email_account_id IS NOT NULL
         DO UPDATE SET
             consumed     = CASE WHEN window_start = excluded.window_start
                                 THEN consumed + 1
                                 ELSE 1
                            END,
             window_start = excluded.window_start,
             updated_at   = excluded.updated_at",
        params![cuenta, clase, clave, ahora],
    )?;
    leer(conn, cuenta, clase, clave)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const AHORA: &str = "2026-09-23T20:00:00Z";
    const HORA: &str = "2026-09-23T14";
    const DIA: &str = "2026-09-23";

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
            )
        })
        .expect("siembra");
        (dir, db)
    }

    #[test]
    fn una_cuenta_nueva_no_ha_gastado_nada() {
        let (_d, db) = base();
        assert_eq!(
            db.gastado_de("cuenta", HORA, DIA).expect("lee"),
            Gastado::default()
        );
    }

    #[test]
    fn cada_envio_gasta_de_las_dos_ventanas() {
        let (_d, db) = base();
        let g = db
            .anotar_un_envio("cuenta", HORA, DIA, AHORA)
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
            .anotar_un_envio("cuenta", HORA, DIA, AHORA)
            .expect("anota");
        assert_eq!(g.en_esta_hora, 2);
        assert_eq!(g.en_este_dia, 2);

        // Y se lee igual desde fuera.
        assert_eq!(db.gastado_de("cuenta", HORA, DIA).expect("lee"), g);
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
                )
            })
            .expect("siembra");

            for _ in 0..7 {
                db.anotar_un_envio("cuenta", HORA, DIA, AHORA)
                    .expect("anota");
            }
        } // aquí se cierra la base, como al cerrar la ventana

        let db = Db::abrir(&ruta, &clave).expect("reabre");
        assert_eq!(
            db.gastado_de("cuenta", HORA, DIA).expect("lee"),
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
            db.anotar_un_envio("cuenta", HORA, DIA, AHORA)
                .expect("anota");
        }

        const HORA_SIGUIENTE: &str = "2026-09-23T15";
        let g = db
            .anotar_un_envio("cuenta", HORA_SIGUIENTE, DIA, AHORA)
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
            db.anotar_un_envio("cuenta", HORA, DIA, AHORA)
                .expect("anota");
        }

        let g = db
            .anotar_un_envio("cuenta", "2026-09-24T09", "2026-09-24", AHORA)
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
        db.anotar_un_envio("cuenta", HORA, DIA, AHORA)
            .expect("anota");

        assert_eq!(
            db.gastado_de("cuenta", "2026-09-23T15", "2026-09-24")
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
            )
        })
        .expect("otra cuenta");

        for _ in 0..4 {
            db.anotar_un_envio("cuenta", HORA, DIA, AHORA)
                .expect("anota");
        }

        assert_eq!(
            db.gastado_de("otra", HORA, DIA).expect("lee"),
            Gastado::default(),
            "el gasto de una cuenta no puede restarle permisos a otra"
        );
        assert_eq!(
            db.gastado_de("cuenta", HORA, DIA).expect("lee").en_este_dia,
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
                db.anotar_un_envio("cuenta", h, DIA, AHORA).expect("anota");
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
}
