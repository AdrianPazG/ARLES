//! El envío de prueba de una etapa: registrarlo y saber si todavía vale.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE UNA PRUEBA PRUEBA, Y HASTA CUÁNDO
//!
//! Un envío de prueba dice una frase concreta: **«este remitente mandó un
//! correo que salió»**. No dice que la plantilla esté bien, ni que la audiencia
//! sea la correcta, ni que el correo no acabe en spam.
//!
//! Y deja de ser cierta en cuanto se cambia el remitente de la etapa. La cuenta
//! nueva puede tener la contraseña mal, el dominio sin autorizar o el servidor
//! equivocado, y una marca de «ya se probó» que sobreviviera a ese cambio haría
//! que el preflight diera luz verde **basándose en una prueba de otra cosa**.
//!
//! Por eso se guarda con qué cuenta se probó, y [`EnvioDePrueba::sigue_valiendo`]
//! compara. La marca no se borra al cambiar de cuenta: se conserva y se
//! compara, porque «se probó el 28/09 con otra cuenta» es justo lo que hay que
//! poder enseñar cuando alguien pregunta por qué el preflight se puso en rojo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UN ENVÍO DE PRUEBA NO TOCA NADA MÁS
//!
//! No entra en la cola, no gasta presupuesto, no aparece en el registro de
//! envíos y **no puede dar de baja a nadie**. Es la regla 3 de la fase: las
//! únicas puertas a una supresión son el `RCPT TO` de un envío real y el acuse
//! final.
//!
//! Esto es lo que hace que este módulo sea tan corto: escribe tres columnas de
//! una fila y no habla con nadie más. Si algún día necesitara una transacción
//! con otra tabla, la pregunta que hay que hacerse es qué se ha colado.

use arles_core::ids::EmailAccountId;
use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Un envío de prueba que consta en la base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvioDePrueba {
    /// Cuándo se hizo, en UTC.
    pub cuando: String,
    /// A qué dirección se mandó. Se enseña: «probado el 28/09 a hola@…» dice
    /// mucho más que «probado».
    pub a_donde: String,
    /// Con qué cuenta se mandó.
    ///
    /// Se guarda como texto y no como [`EmailAccountId`] a propósito: es un dato
    /// histórico, y la cuenta puede haberse borrado. Un identificador que ya no
    /// corresponde a nada sigue sirviendo para comparar —que es lo único que se
    /// hace con él— mientras que exigir que siga siendo válido convertiría el
    /// borrado de una cuenta en un error al leer una etapa vieja.
    pub con_la_cuenta: String,
}

impl EnvioDePrueba {
    /// ¿Esta prueba dice algo del remitente que la etapa usa **hoy**?
    ///
    /// Verificado rompiéndolo el 28/09/2026: devolviendo siempre `true`, falla
    /// `cambiar_de_remitente_invalida_la_prueba`.
    #[must_use]
    pub fn sigue_valiendo(&self, cuenta_de_la_etapa: EmailAccountId) -> bool {
        self.con_la_cuenta == cuenta_de_la_etapa.to_string()
    }
}

impl Db {
    /// Deja constancia de que se mandó un correo de prueba.
    ///
    /// Se llama **después** de que el proveedor aceptara el mensaje, nunca
    /// antes: una marca escrita antes de mandar diría que se probó algo que
    /// quizá falló, y el preflight daría luz verde a una cuenta que no envía.
    ///
    /// Las tres columnas se escriben en la misma sentencia. No hay un `CHECK`
    /// que lo exija —SQLite revalida todas las filas al añadir una columna con
    /// `CHECK`, y eso haría imposible migrar una base con etapas—, así que la
    /// coherencia la mantiene esta función siendo la única que las escribe.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla, o
    /// [`DbError::DatoInvalido`] si la etapa no existe. Lo segundo importa:
    /// sin esa comprobación, un `UPDATE` que no toca ninguna fila **no es un
    /// error en SQL**, y registrar la prueba de una etapa borrada se daría por
    /// bueno en silencio.
    pub fn registrar_envio_de_prueba(
        &self,
        etapa: &str,
        a_donde: &str,
        con_la_cuenta: EmailAccountId,
    ) -> Result<(), DbError> {
        let filas = self.con(|conn| {
            conn.execute(
                "UPDATE campaign_stage
                    SET test_sent_at = ?2,
                        test_sent_to = ?3,
                        test_sent_with_account_id = ?4,
                        updated_at = ?2
                  WHERE id = ?1",
                params![etapa, ahora(), a_donde, con_la_cuenta.to_string()],
            )
        })?;

        if filas == 0 {
            return Err(DbError::DatoInvalido {
                motivo: "campaign_stage.id",
            });
        }
        Ok(())
    }

    /// El envío de prueba de una etapa, si consta.
    ///
    /// `None` significa que nunca se hizo uno. **No** significa que no valga:
    /// para eso está [`EnvioDePrueba::sigue_valiendo`], y son dos cosas que la
    /// pantalla cuenta distinto —«todavía no lo has probado» frente a «lo
    /// probaste con otro remitente»—.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn envio_de_prueba(&self, etapa: &str) -> Result<Option<EnvioDePrueba>, DbError> {
        self.con(|conn| {
            conn.query_row(
                "SELECT test_sent_at, test_sent_to, test_sent_with_account_id
                   FROM campaign_stage
                  WHERE id = ?1",
                params![etapa],
                |f| {
                    Ok((
                        f.get::<_, Option<String>>(0)?,
                        f.get::<_, Option<String>>(1)?,
                        f.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()
            .map(|fila| match fila {
                // Las tres se escriben juntas. Si alguna falta, la fila no
                // describe una prueba y se dice que no hay ninguna, en vez de
                // inventarse los huecos.
                Some((Some(cuando), Some(a_donde), Some(con_la_cuenta))) => Some(EnvioDePrueba {
                    cuando,
                    a_donde,
                    con_la_cuenta,
                }),
                _ => None,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
    const OTRA_CUENTA: &str = "01900000-0000-7000-8000-00000000000b";
    const ETAPA: &str = "etapa-1";

    fn cuenta(id: &str) -> EmailAccountId {
        id.parse().expect("id de cuenta")
    }

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
                                            daily_limit, hourly_limit, created_at, updated_at)
                 VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                         'llavero://v', 'connected', 50, 10,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z'),
                        ('{OTRA_CUENTA}', '{EMPRESA}', 'Soporte', 'soporte@t.mx', 'smtp',
                         'llavero://s', 'connected', 50, 10,
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO execution_window (id, company_id, name, days_of_week,
                                               start_hour, end_hour, timezone, created_at)
                 VALUES ('vent-1', '{EMPRESA}', 'Laborables', 31, 9, 18,
                         'America/Mexico_City', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                 VALUES ('camp-1', '{EMPRESA}', 'Prueba', 'draft',
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             email_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('{ETAPA}', 'camp-1', 1, 'email', '{CUENTA}', 'vent-1',
                         'draft', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');"
            ))
        })
        .expect("siembra");

        (dir, db)
    }

    /// Una etapa recién creada **no está probada**, y se nota.
    ///
    /// Es lo que el preflight tiene que poder decir en una instalación nueva.
    /// Un valor por defecto que dijera lo contrario marcaría como probadas
    /// todas las campañas que ya existen.
    #[test]
    fn una_etapa_nueva_no_tiene_ninguna_prueba() {
        let (_dir, db) = base();
        assert_eq!(db.envio_de_prueba(ETAPA).expect("consulta"), None);
    }

    /// Registrada, se lee entera: cuándo, a dónde y con qué cuenta.
    #[test]
    fn una_prueba_registrada_se_lee_con_sus_tres_datos() {
        let (_dir, db) = base();
        db.registrar_envio_de_prueba(ETAPA, "adrian@telemetry.mx", cuenta(CUENTA))
            .expect("registra");

        let p = db
            .envio_de_prueba(ETAPA)
            .expect("consulta")
            .expect("consta la prueba");
        assert_eq!(p.a_donde, "adrian@telemetry.mx");
        assert_eq!(p.con_la_cuenta, CUENTA);
        assert!(p.cuando.starts_with("20"), "una fecha ISO: {}", p.cuando);
        assert!(p.sigue_valiendo(cuenta(CUENTA)));
    }

    /// **Cambiar el remitente invalida la prueba, y no la borra.**
    ///
    /// Una marca de «ya se probó» que sobreviviera al cambio de cuenta haría
    /// que el preflight diera luz verde basándose en una prueba de otra cosa:
    /// la cuenta nueva puede tener la contraseña mal o el dominio sin
    /// autorizar, y nadie lo sabría hasta el primer envío de verdad.
    ///
    /// Y se conserva a propósito, porque «se probó el 28/09 con otro remitente»
    /// es exactamente lo que hay que enseñar cuando alguien pregunta por qué el
    /// preflight se puso en rojo. Un disparador que la borrara destruiría esa
    /// explicación.
    ///
    /// Verificada rompiéndola el 28/09/2026: haciendo que `sigue_valiendo`
    /// devuelva siempre `true`, esta prueba falla.
    #[test]
    fn cambiar_de_remitente_invalida_la_prueba() {
        let (_dir, db) = base();
        db.registrar_envio_de_prueba(ETAPA, "adrian@telemetry.mx", cuenta(CUENTA))
            .expect("registra");

        let p = db
            .envio_de_prueba(ETAPA)
            .expect("consulta")
            .expect("consta");

        assert!(
            !p.sigue_valiendo(cuenta(OTRA_CUENTA)),
            "una prueba de un remitente no dice nada de otro"
        );
        // Y sigue ahí para poder explicarlo.
        assert_eq!(p.con_la_cuenta, CUENTA);
        assert_eq!(p.a_donde, "adrian@telemetry.mx");
    }

    /// Volver a probar sustituye la prueba anterior.
    ///
    /// No se acumulan: lo que el preflight pregunta es si la etapa está probada
    /// **ahora**, y un historial de pruebas es una pantalla que nadie pidió.
    #[test]
    fn volver_a_probar_sustituye_la_prueba_anterior() {
        let (_dir, db) = base();
        db.registrar_envio_de_prueba(ETAPA, "uno@telemetry.mx", cuenta(CUENTA))
            .expect("primera");
        db.registrar_envio_de_prueba(ETAPA, "dos@telemetry.mx", cuenta(OTRA_CUENTA))
            .expect("segunda");

        let p = db
            .envio_de_prueba(ETAPA)
            .expect("consulta")
            .expect("consta");
        assert_eq!(p.a_donde, "dos@telemetry.mx");
        assert_eq!(p.con_la_cuenta, OTRA_CUENTA);
    }

    /// **Registrar la prueba de una etapa que no existe es un error.**
    ///
    /// Un `UPDATE` que no toca ninguna fila no falla en SQL. Sin la
    /// comprobación del recuento, esto se daría por bueno en silencio y quien
    /// llama creería que la prueba quedó registrada.
    ///
    /// Verificada rompiéndola el 28/09/2026: quitando el `if filas == 0`, esta
    /// prueba falla.
    #[test]
    fn registrar_la_prueba_de_una_etapa_que_no_existe_da_error() {
        let (_dir, db) = base();
        let error = db
            .registrar_envio_de_prueba("no-existe", "a@b.mx", cuenta(CUENTA))
            .expect_err("no se puede probar una etapa que no está");
        assert_eq!(error.clave_i18n(), "error.db.dato_invalido");
    }

    /// **Una fila a medias no se lee como una prueba.**
    ///
    /// No lo puede producir el repositorio —las tres columnas se escriben
    /// juntas—, pero sí una base tocada a mano o una restauración a medias.
    /// Inventarse los huecos daría una prueba con fecha y sin destinatario, que
    /// el preflight aceptaría.
    #[test]
    fn una_prueba_a_medias_no_cuenta_como_prueba() {
        let (_dir, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "UPDATE campaign_stage
                    SET test_sent_at = '2026-09-28T00:00:00Z',
                        test_sent_to = NULL,
                        test_sent_with_account_id = NULL
                  WHERE id = 'etapa-1'",
            )
        })
        .expect("deja la fila a medias");

        assert_eq!(db.envio_de_prueba(ETAPA).expect("consulta"), None);
    }

    /// La prueba de una etapa no marca a las demás.
    ///
    /// Desde la V4 una campaña puede tener dos etapas por dos canales, y una
    /// prueba por correo no dice nada de la de WhatsApp: otra cuenta, otro
    /// proveedor, otro formato de mensaje.
    #[test]
    fn probar_una_etapa_no_marca_a_la_otra() {
        let (_dir, db) = base();
        // La segunda etapa es de WhatsApp, y no por adorno: el esquema impide
        // dos etapas del mismo canal en una campaña (L-1), y además es el
        // escenario que importa — otra cuenta, otro proveedor, otro formato de
        // mensaje. Una prueba por correo no dice nada de esto.
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                               waba_id, phone_number_id, credential_ref,
                                               status, created_at, updated_at)
                 VALUES ('wa-1', '{EMPRESA}', 'Ventas', '+525555555555',
                         'waba-1', 'pn-1', 'llavero://w', 'connected',
                         '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

                 INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                             whatsapp_account_id, execution_window_id,
                                             status, created_at, updated_at)
                 VALUES ('etapa-2', 'camp-1', 2, 'whatsapp', 'wa-1', 'vent-1',
                         'draft', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');"
            ))
        })
        .expect("segunda etapa");

        db.registrar_envio_de_prueba(ETAPA, "uno@telemetry.mx", cuenta(CUENTA))
            .expect("registra");

        assert!(db.envio_de_prueba(ETAPA).expect("consulta").is_some());
        assert_eq!(
            db.envio_de_prueba("etapa-2").expect("consulta"),
            None,
            "probar una etapa no puede dar por probada otra"
        );
    }
}
