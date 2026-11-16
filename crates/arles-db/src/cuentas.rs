//! La salud de una cuenta remitente: su disyuntor y su estado.
//!
//! Las **reglas** —cuántos fallos abren el circuito, cuáles cuentan— viven en
//! `arles_core::disyuntor`. Aquí sólo se lee y se escribe.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS COLUMNAS ESTABAN DESDE LA V1 Y NADIE LAS TOCABA
//!
//! `email_account.consecutive_failures` y `circuit_open_until` se crearon con
//! el esquema inicial, y hasta esta auditoría (23/09/2026) **ningún código las
//! escribía ni las leía**. El §7 del motor describía un disyuntor que no
//! existía, y el §13 lo listaba como uno de los pasos del bucle.
//!
//! Es el patrón que este proyecto ya conoce: un esquema que promete algo y un
//! código que no lo cumple no falla — simplemente no protege de nada.

use arles_core::disyuntor::{Circuito, Efecto};
use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Lo que se sabe del disyuntor de una cuenta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaludDeLaCuenta {
    pub circuito: Circuito,
    /// Cuántos fallos del canal lleva seguidos.
    pub fallos_seguidos: u32,
    /// El estado de la cuenta tal como lo guarda el esquema: `connected`,
    /// `needs_reauth`, `circuit_open` o `disabled`.
    pub estado: String,
}

impl SaludDeLaCuenta {
    /// Lo que el preflight debe anotar en `credencial_sirve`.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LO QUE YA SE SABE NO SE MARCA COMO «SIN COMPROBAR»
    ///
    /// El preflight (§12) tiene tres respuestas, y la tercera —«sin
    /// comprobar»— existe para lo que de verdad no se puede mirar todavía:
    /// hablar con el proveedor es de la Fase 5.
    ///
    /// Pero si esta cuenta está en `needs_reauth`, **ya lo sabemos**, y lo
    /// sabemos sin hablar con nadie: se apuntó la última vez que falló. Dejarlo
    /// en «sin comprobar» sería decirle al usuario «no he podido mirar» cuando
    /// la respuesta estaba escrita en la base, y le haría buscar el problema en
    /// otra parte.
    ///
    /// Se detectó al revisar la fase como usuario el 23/09/2026: el disyuntor
    /// y el preflight se habían construido en entregas distintas y no se
    /// hablaban.
    ///
    /// | Estado de la cuenta | Qué anota el preflight |
    /// |---|---|
    /// | `needs_reauth` | `Some(false)` — **falla**, y se sabe por qué |
    /// | Cualquier otro | `None` — todavía no se puede comprobar (Fase 5) |
    ///
    /// Cuando los adaptadores existan, esta función devolverá `Some(true)`
    /// tras una autenticación correcta. Hasta entonces, lo único honesto que
    /// puede afirmar es el fallo que ya consta.
    #[must_use]
    pub fn credencial_para_el_preflight(&self) -> Option<bool> {
        if self.hay_que_renovar_la_credencial() {
            Some(false)
        } else {
            None
        }
    }

    /// ¿Hay que renovar la credencial?
    ///
    /// Se expone aparte del circuito porque **lo que hay que hacer es
    /// distinto**: un circuito abierto por la red se cierra solo en media
    /// hora; uno abierto por la credencial no se cierra hasta que una persona
    /// la renueve. Enseñar los dos igual haría esperar media hora para nada.
    #[must_use]
    pub fn hay_que_renovar_la_credencial(&self) -> bool {
        self.estado == "needs_reauth"
    }
}

/// La salud de una cuenta, sobre una conexión ya abierta.
///
/// Existe aparte de [`Db::salud_de_la_cuenta`] porque el cerrojo de la conexión
/// **no es reentrante**: quien ya esté dentro de un `con_dominio` —la revisión
/// de una campaña, sin ir más lejos— se quedaría colgado para siempre llamando
/// al método público. Un bloqueo no da error ni traza: la aplicación se queda
/// quieta, y eso es más difícil de diagnosticar que cualquier excepción.
///
/// Y se comparte en vez de repetir la consulta porque la regla de qué cuenta
/// como circuito abierto vive aquí, en un sitio, y no en dos que pueden
/// separarse sin que nada falle.
pub(crate) fn salud_en(
    conn: &rusqlite::Connection,
    cuenta: &str,
    ahora: &str,
) -> Result<SaludDeLaCuenta, DbError> {
    let fila: Option<(i64, Option<String>, String)> = conn
        .query_row(
            "SELECT consecutive_failures, circuit_open_until, status
               FROM email_account WHERE id = ?1",
            params![cuenta],
            |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
        )
        .optional()?;

    let Some((fallos, abierto_hasta, estado)) = fila else {
        return Err(DbError::DatoInvalido {
            motivo: "esa cuenta remitente no existe",
        });
    };

    // Los sellos son todos de la misma forma —UTC, segundos enteros—, así que
    // compararlos como texto es compararlos como instantes. Es el contrato de
    // `arles_motor::tiempo`.
    let circuito = match abierto_hasta {
        Some(hasta) if hasta.as_str() > ahora => Circuito::Abierto {
            segundos_restantes: segundos_entre(ahora, &hasta),
        },
        _ => Circuito::Cerrado,
    };

    Ok(SaludDeLaCuenta {
        circuito,
        fallos_seguidos: u32::try_from(fallos).unwrap_or(0),
        estado,
    })
}

impl Db {
    /// Cómo está el disyuntor de esta cuenta, ahora mismo.
    ///
    /// `ahora` es el sello de tiempo actual. El circuito se cierra **solo** al
    /// vencer el plazo: no hace falta una tarea que lo vaya mirando, igual que
    /// el presupuesto no necesita nadie que lo reinicie a medianoche. Basta con
    /// comparar al leer.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si no hay tal cuenta.
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn salud_de_la_cuenta(
        &self,
        cuenta: &str,
        ahora: &str,
    ) -> Result<SaludDeLaCuenta, DbError> {
        self.con_dominio(|conn| salud_en(conn, cuenta, ahora))
    }

    /// Aplica a la cuenta lo que decidió `arles_core::disyuntor`.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ `needs_reauth` Y `circuit_open` SON COSAS DISTINTAS
    ///
    /// El esquema tiene los dos estados, y la diferencia es lo que el usuario
    /// tiene que hacer:
    ///
    /// | Estado | Qué pasó | Qué hace el usuario |
    /// |---|---|---|
    /// | `circuit_open` | El servidor no responde | Esperar, o mirar su red |
    /// | `needs_reauth` | La contraseña no sirve | **Renovarla** |
    ///
    /// Marcar los dos igual haría que alguien esperase media hora a que se
    /// arreglara sola una contraseña caducada, que no se va a arreglar sola.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn aplicar_al_disyuntor(
        &self,
        cuenta: &str,
        efecto: Efecto,
        ahora: &str,
        abierto_hasta: Option<&str>,
    ) -> Result<(), DbError> {
        let cuando = ahora_o(ahora);
        self.con(|conn| {
            match efecto {
                // No toca el contador: este fallo no hablaba de la cuenta.
                Efecto::NoCuenta => {}

                Efecto::Reiniciar => {
                    // Se cierra el circuito **y** se limpia el contador, pero
                    // `needs_reauth` NO se toca: que un envío saliera no
                    // significa que la credencial esté renovada, y quitar ese
                    // aviso solo escondería lo que el usuario tiene que hacer.
                    conn.execute(
                        "UPDATE email_account
                            SET consecutive_failures = 0, circuit_open_until = NULL,
                                status = CASE WHEN status = 'circuit_open'
                                              THEN 'connected' ELSE status END,
                                updated_at = ?2
                          WHERE id = ?1",
                        params![cuenta, cuando],
                    )?;
                }

                Efecto::Contar { van, .. } => {
                    conn.execute(
                        "UPDATE email_account
                            SET consecutive_failures = ?2, updated_at = ?3
                          WHERE id = ?1",
                        params![cuenta, i64::from(van), cuando],
                    )?;
                }

                Efecto::Abrir { por_credencial, .. } => {
                    let estado = if por_credencial {
                        "needs_reauth"
                    } else {
                        "circuit_open"
                    };
                    conn.execute(
                        "UPDATE email_account
                            SET circuit_open_until = ?2, status = ?3, updated_at = ?4
                          WHERE id = ?1",
                        params![cuenta, abierto_hasta, estado, cuando],
                    )?;
                }
            }
            Ok(())
        })
    }

    /// Cierra el disyuntor a mano, porque el usuario arregló la causa.
    ///
    /// Es el «o manualmente cuando el usuario corrige el problema» del §7. Sin
    /// esto, quien renueva su contraseña tendría que esperar media hora a que
    /// ARLES se diera cuenta, y la espera no arregla nada que no esté ya
    /// arreglado.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn cerrar_el_disyuntor(&self, cuenta: &str) -> Result<(), DbError> {
        let cuando = ahora();
        self.con(|conn| {
            conn.execute(
                "UPDATE email_account
                    SET consecutive_failures = 0, circuit_open_until = NULL,
                        status = 'connected', updated_at = ?2
                  WHERE id = ?1",
                params![cuenta, cuando],
            )?;
            Ok(())
        })
    }
}

/// Segundos entre dos sellos, para poder decir cuánto falta (§94).
///
/// Se calcula restando los caracteres que ya se sabe que están —el formato es
/// fijo— en vez de traer una biblioteca de fechas a `arles-db`, que no la
/// tiene y no debería tenerla. Si algo no encaja, se devuelve cero: una cifra
/// visiblemente absurda es mejor que una inventada.
fn segundos_entre(desde: &str, hasta: &str) -> u64 {
    let a = time::OffsetDateTime::parse(desde, &time::format_description::well_known::Rfc3339);
    let b = time::OffsetDateTime::parse(hasta, &time::format_description::well_known::Rfc3339);
    match (a, b) {
        (Ok(a), Ok(b)) => u64::try_from((b - a).whole_seconds()).unwrap_or(0),
        _ => 0,
    }
}

/// El sello que se recibe, o el del reloj si viene vacío.
fn ahora_o(dado: &str) -> String {
    if dado.is_empty() {
        ahora()
    } else {
        dado.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::disyuntor::{ESPERA_DEL_DISYUNTOR, FALLOS_PARA_ABRIR, tras_un_fallo};
    use arles_core::reintento::Fallo;

    use crate::conexion::ClaveMaestra;

    const AHORA: &str = "2026-09-23T12:00:00Z";
    const MEDIA_HORA_DESPUES: &str = "2026-09-23T12:30:00Z";
    const UNA_HORA_DESPUES: &str = "2026-09-23T13:00:00Z";

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

    /// Simula una racha de fallos como la haría el bucle.
    fn fallar(db: &Db, fallo: Fallo, veces: u32) {
        for _ in 0..veces {
            let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
            let efecto = tras_un_fallo(fallo, salud.fallos_seguidos);
            db.aplicar_al_disyuntor("cuenta", efecto, AHORA, Some(MEDIA_HORA_DESPUES))
                .expect("aplica");
        }
    }

    /// **Cinco fallos de red seguidos paran la cuenta de verdad.**
    ///
    /// No es lo mismo que la prueba del núcleo: aquí se comprueba que el
    /// contador se guarda entre envíos. Un contador que no persistiera haría
    /// que el disyuntor no se abriera nunca, porque cada envío empezaría de
    /// cero.
    #[test]
    fn cinco_fallos_de_red_abren_el_disyuntor_de_verdad() {
        let (_d, db) = base();
        fallar(&db, Fallo::Pasajero, FALLOS_PARA_ABRIR - 1);

        let a_medias = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert_eq!(a_medias.fallos_seguidos, FALLOS_PARA_ABRIR - 1);
        assert!(
            a_medias.circuito.puede_enviar(),
            "con cuatro fallos todavía se envía"
        );

        fallar(&db, Fallo::Pasajero, 1);
        let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert!(!salud.circuito.puede_enviar());
        assert_eq!(salud.estado, "circuit_open");
    }

    /// **Y cinco direcciones muertas no.**
    ///
    /// Verificada rompiéndola el 23/09/2026: si `NoCuenta` escribiera el
    /// contador, esta prueba falla y una lista con direcciones viejas pararía
    /// la campaña cada pocos minutos.
    #[test]
    fn una_racha_de_direcciones_muertas_no_para_la_cuenta() {
        let (_d, db) = base();
        fallar(&db, Fallo::LaDireccionNoExiste, FALLOS_PARA_ABRIR + 3);

        let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert_eq!(salud.fallos_seguidos, 0);
        assert!(salud.circuito.puede_enviar());
        assert_eq!(salud.estado, "connected");
    }

    /// **El disyuntor se cierra solo al vencer el plazo.**
    ///
    /// Sin tarea que lo vigile: basta con comparar al leer, igual que hace el
    /// presupuesto con sus etiquetas.
    #[test]
    fn el_disyuntor_se_cierra_solo_al_vencer_el_plazo() {
        let (_d, db) = base();
        fallar(&db, Fallo::Pasajero, FALLOS_PARA_ABRIR);

        assert!(
            !db.salud_de_la_cuenta("cuenta", AHORA)
                .expect("lee")
                .circuito
                .puede_enviar(),
            "recién abierto, no envía"
        );
        assert!(
            db.salud_de_la_cuenta("cuenta", UNA_HORA_DESPUES)
                .expect("lee")
                .circuito
                .puede_enviar(),
            "una hora después, el plazo venció y vuelve a enviar"
        );
    }

    /// Y mientras está abierto, dice cuánto falta. Cifras, no adjetivos (§94).
    #[test]
    fn un_circuito_abierto_dice_cuantos_segundos_faltan() {
        let (_d, db) = base();
        fallar(&db, Fallo::Pasajero, FALLOS_PARA_ABRIR);

        assert_eq!(
            db.salud_de_la_cuenta("cuenta", AHORA)
                .expect("lee")
                .circuito,
            Circuito::Abierto {
                segundos_restantes: ESPERA_DEL_DISYUNTOR.as_secs()
            }
        );
    }

    /// **Una credencial caducada se marca distinto, porque se arregla
    /// distinto.**
    ///
    /// Un circuito abierto por la red se cierra solo; uno abierto por la
    /// contraseña no se cierra hasta que alguien la renueve. Marcarlos igual
    /// haría esperar media hora para nada.
    #[test]
    fn una_credencial_caducada_pide_renovarla_y_no_esperar() {
        let (_d, db) = base();
        fallar(&db, Fallo::LaCuentaNoAutentica, 1);

        let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert_eq!(salud.estado, "needs_reauth");
        assert!(salud.hay_que_renovar_la_credencial());
        assert!(!salud.circuito.puede_enviar(), "y mientras tanto no envía");
    }

    /// **Un envío que sale borra la racha, pero no borra el aviso de
    /// credencial.**
    ///
    /// Es la parte contraintuitiva: `needs_reauth` sobrevive a un acierto
    /// porque un envío que salió no prueba que la contraseña esté renovada, y
    /// quitar el aviso escondería lo único que el usuario tiene que hacer.
    #[test]
    fn un_acierto_borra_la_racha_pero_no_el_aviso_de_credencial() {
        let (_d, db) = base();

        // Racha de red: el acierto la borra entera.
        fallar(&db, Fallo::Pasajero, 3);
        db.aplicar_al_disyuntor("cuenta", Efecto::Reiniciar, AHORA, None)
            .expect("acierta");
        let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert_eq!(salud.fallos_seguidos, 0);
        assert_eq!(salud.estado, "connected");

        // Credencial: el aviso se queda.
        fallar(&db, Fallo::LaCuentaNoAutentica, 1);
        db.aplicar_al_disyuntor("cuenta", Efecto::Reiniciar, AHORA, None)
            .expect("acierta");
        assert!(
            db.salud_de_la_cuenta("cuenta", AHORA)
                .expect("lee")
                .hay_que_renovar_la_credencial(),
            "un envío que salió no renueva ninguna contraseña"
        );
    }

    /// El usuario puede cerrarlo a mano cuando arregla la causa (§7).
    #[test]
    fn el_usuario_puede_cerrar_el_disyuntor_a_mano() {
        let (_d, db) = base();
        fallar(&db, Fallo::LaCuentaNoAutentica, 1);

        db.cerrar_el_disyuntor("cuenta").expect("cierra");

        let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert!(salud.circuito.puede_enviar());
        assert_eq!(salud.estado, "connected");
        assert_eq!(salud.fallos_seguidos, 0);
    }

    /// **El preflight no dice «sin comprobar» de algo que ya consta.**
    ///
    /// Si la cuenta está marcada como que hay que renovar la credencial, el
    /// preflight lo anota como un fallo, no como algo que no se pudo mirar. La
    /// diferencia es lo que el usuario hace después: buscar el problema en otra
    /// parte, o renovar su contraseña.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que devuelva siempre
    /// `None`, el preflight de una cuenta con la contraseña caducada dice «sin
    /// comprobar» teniendo la respuesta escrita en la base.
    #[test]
    fn el_preflight_sabe_que_la_credencial_no_sirve_sin_preguntar_a_nadie() {
        let (_d, db) = base();

        assert_eq!(
            db.salud_de_la_cuenta("cuenta", AHORA)
                .expect("lee")
                .credencial_para_el_preflight(),
            None,
            "con la cuenta sana no se puede afirmar nada hasta la Fase 5"
        );

        fallar(&db, Fallo::LaCuentaNoAutentica, 1);

        assert_eq!(
            db.salud_de_la_cuenta("cuenta", AHORA)
                .expect("lee")
                .credencial_para_el_preflight(),
            Some(false),
            "esto no es «no he podido mirar»: consta que no sirve"
        );
    }

    /// Y un circuito abierto por la red **no** marca la credencial como mala.
    ///
    /// Son dos cosas distintas: la red se arregla sola en media hora, la
    /// contraseña no. Confundirlas mandaría al usuario a renovar una
    /// credencial que estaba perfecta.
    #[test]
    fn un_circuito_abierto_por_la_red_no_acusa_a_la_credencial() {
        let (_d, db) = base();
        fallar(&db, Fallo::Pasajero, FALLOS_PARA_ABRIR);

        let salud = db.salud_de_la_cuenta("cuenta", AHORA).expect("lee");
        assert!(
            !salud.circuito.puede_enviar(),
            "el circuito sí está abierto"
        );
        assert_eq!(
            salud.credencial_para_el_preflight(),
            None,
            "pero la contraseña no tiene la culpa"
        );
    }

    /// Preguntar por una cuenta que no existe es un error, no un «está bien».
    #[test]
    fn una_cuenta_que_no_existe_no_se_da_por_sana() {
        let (_d, db) = base();
        assert!(db.salud_de_la_cuenta("no-existe", AHORA).is_err());
    }
}
