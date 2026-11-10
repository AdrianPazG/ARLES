//! Aplicar las bajas que llegaron al buzón (R-25, tramo 3 · D-21).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE MÓDULO GUARDA, Y LO QUE DELIBERADAMENTE NO
//!
//! Guarda **la dirección, el motivo, el origen y la fecha** — exactamente lo
//! mismo que una baja apuntada a mano desde Contactos. No guarda el asunto, ni
//! el cuerpo, ni el nombre de quien escribió, ni nada que viniera dentro del
//! correo: `arles-email` lo lee, el núcleo lo juzga y lo que llega aquí es un
//! veredicto y una dirección.
//!
//! Lo único nuevo que se guarda es **hasta qué correo se leyó**
//! (`inbox_last_uid`), y eso no es un dato de nadie: es un número del buzón.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TODO VA EN UNA TRANSACCIÓN, Y NO ES POR PULCRITUD
//!
//! Las supresiones, su línea de bitácora y el avance del `inbox_last_uid`
//! entran o no entran juntos. Si el UID avanzara aparte y la escritura fallara,
//! esas bajas **no se volverían a mirar nunca**: el correo ya quedaría por
//! debajo de la marca, y la persona que pidió no recibir seguiría en la lista
//! de envío sin que nada protestara.
//!
//! Es el mismo razonamiento que `auditar` tiene escrito desde la Fase 1: la
//! bitácora va en la transacción del cambio que describe, porque registrarla
//! aparte deja cambios sin rastro.

use rusqlite::params;

use arles_core::canal::Canal;
use arles_core::ids::{CompanyId, EmailAccountId};
use arles_core::supresion::{MotivoDeSupresion, OrigenDeSupresion};

use crate::db::Db;
use crate::empresa::{ahora, auditar};
use crate::error::DbError;

/// Una cuenta con la lectura del buzón encendida.
///
/// **Sin la contraseña**, igual que [`crate::correo::CuentaDeCorreoGuardada`]:
/// aquí va la referencia al llavero, que no sirve para entrar a ningún sitio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuzonQueSeLee {
    pub cuenta: EmailAccountId,
    /// El nombre que se verá en la pantalla al decir qué buzón se revisó.
    pub nombre_visible: String,
    /// La dirección de la cuenta, que es el usuario en los cinco proveedores
    /// que ARLES reconoce.
    pub direccion: String,
    /// El servidor de lectura, tal como se guardó. **No se deduce aquí**: ver
    /// la cabecera de la migración V18.
    pub servidor: String,
    /// La referencia al llavero donde está la contraseña.
    pub referencia_credencial: String,
    /// El UID más alto ya procesado, o `None` si nunca se leyó este buzón.
    pub ultimo_uid: Option<u32>,
}

/// Cómo está la lectura del buzón de una cuenta, para enseñarla.
///
/// **Sin la referencia al llavero**: aquí no hace falta, y lo que no se pide no
/// se puede filtrar por descuido. La regla 3 de la frontera prohíbe devolver
/// secretos, y la referencia no lo es — pero tampoco sirve para nada en una
/// pantalla que sólo dice si el buzón se lee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComoEstaUnBuzon {
    pub cuenta: EmailAccountId,
    pub nombre_visible: String,
    pub direccion: String,
    /// El servidor de envío, que es de donde la pantalla **propone** el de
    /// lectura. ARLES no lo deduce por su cuenta: ver la V18.
    pub servidor_de_envio: String,
    /// El de lectura, o `None` si no se ha escrito ninguno.
    pub servidor_de_lectura: Option<String>,
    pub encendido: bool,
    /// ¿Se ha leído este buzón alguna vez? Lo que importa de
    /// `inbox_last_uid` en una pantalla es si hay marca, no cuál es: el número
    /// del UID no le dice nada a nadie.
    pub ya_se_leyo: bool,
}

/// Una baja que llegó por correo, lista para escribirse.
///
/// La construye `arles-app` **después** de que el núcleo haya dicho que ese
/// correo es una baja inequívoca. Que no haya forma de construirla con un
/// veredicto dudoso es cosa de quien llama; lo que este módulo garantiza es que
/// todo lo que entra por aquí se guarda con el origen correcto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BajaQueLlego {
    /// La dirección de **quien escribió**, ya normalizada. Nunca una del texto
    /// del correo: ver `arles_core::baja_por_correo`.
    pub direccion: String,
    /// El UID del correo que la pidió. Sirve para avanzar la marca, no se
    /// guarda con la baja.
    pub uid: u32,
    /// ¿Vino por el asunto —el botón de cancelar suscripción del cliente de
    /// correo— o por el cuerpo —contestó la palabra a mano—?
    ///
    /// **No se guarda en la supresión**: la tabla no tiene dónde, y no cambia
    /// nada de la baja. Va a la bitácora, y es lo que permite contestar
    /// «¿cómo pidió la baja esta persona?» meses después — que es distinto de
    /// «¿cuándo?». El núcleo lo distingue en `ComoLoPidio`.
    pub por_el_asunto: bool,
}

/// Lo que pasó en una revisión del buzón.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResumenDeLaRevision {
    /// Cuántas direcciones entraron en la lista por primera vez.
    pub dadas_de_baja: u32,
    /// Cuántas ya estaban. **No es un error**: suprimir es idempotente, y esta
    /// cifra existe para que la pantalla no diga «3 bajas nuevas» cuando son
    /// las mismas tres de ayer.
    pub ya_estaban: u32,
    /// Cuántos correos nombraban la baja sin pedirla claramente. **No se toca
    /// ninguno**: los mira una persona en su buzón.
    pub para_mirar: u32,
}

impl Db {
    /// Los buzones de esta empresa con la lectura encendida.
    ///
    /// Sólo los que tienen servidor **y** están encendidos: una cuenta con la
    /// casilla marcada y sin servidor no se puede leer, y devolverla aquí
    /// obligaría a quien llama a repetir la comprobación.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la lectura falla.
    pub fn buzones_que_se_leen(&self, empresa: CompanyId) -> Result<Vec<BuzonQueSeLee>, DbError> {
        self.leer_dominio(|conn| {
            let mut consulta = conn.prepare(
                "SELECT id, display_name, email_address, imap_host,
                        credential_ref, inbox_last_uid
                   FROM email_account
                  WHERE company_id = ?1
                    AND provider_kind = 'smtp'
                    AND inbox_scan_enabled = 1
                    AND imap_host IS NOT NULL
                    AND status = 'connected'
                  ORDER BY created_at, rowid",
            )?;
            let filas = consulta.query_map([empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, String>(3)?,
                    f.get::<_, String>(4)?,
                    f.get::<_, Option<i64>>(5)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, direccion, servidor, referencia, uid) = fila?;
                salida.push(BuzonQueSeLee {
                    cuenta: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "el identificador de esa cuenta de correo no es un UUID",
                    })?,
                    nombre_visible: nombre,
                    direccion,
                    servidor,
                    referencia_credencial: referencia,
                    // Un UID no cabe en negativo y no pasa de `u32` (RFC 3501):
                    // lo que no quepa se trata como «nunca se leyó», que hace
                    // mirar de más y nunca de menos.
                    ultimo_uid: uid.and_then(|n| u32::try_from(n).ok()),
                });
            }
            Ok(salida)
        })
    }

    /// Cómo está la lectura del buzón en **todas** las cuentas de correo.
    ///
    /// Distinto de [`Self::buzones_que_se_leen`], que devuelve sólo las que se
    /// pueden leer ahora: esto es lo que la pantalla necesita para enseñar el
    /// estado de cada una, incluidas las apagadas y las que no tienen servidor.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la lectura falla.
    pub fn como_estan_los_buzones(
        &self,
        empresa: CompanyId,
    ) -> Result<Vec<ComoEstaUnBuzon>, DbError> {
        self.leer_dominio(|conn| {
            let mut consulta = conn.prepare(
                "SELECT id, display_name, email_address, smtp_host,
                        imap_host, inbox_scan_enabled, inbox_last_uid
                   FROM email_account
                  WHERE company_id = ?1 AND provider_kind = 'smtp'
                  ORDER BY created_at, rowid",
            )?;
            let filas = consulta.query_map([empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, Option<String>>(3)?,
                    f.get::<_, Option<String>>(4)?,
                    f.get::<_, i64>(5)?,
                    f.get::<_, Option<i64>>(6)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, direccion, envio, lectura, encendido, uid) = fila?;
                salida.push(ComoEstaUnBuzon {
                    cuenta: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "el identificador de esa cuenta de correo no es un UUID",
                    })?,
                    nombre_visible: nombre,
                    direccion,
                    servidor_de_envio: envio.unwrap_or_default(),
                    servidor_de_lectura: lectura,
                    encendido: encendido == 1,
                    ya_se_leyo: uid.is_some(),
                });
            }
            Ok(salida)
        })
    }

    /// Enciende o apaga la lectura del buzón de una cuenta, con su servidor.
    ///
    /// Queda en la bitácora: encender la lectura del buzón de alguien es una
    /// decisión, no una preferencia de interfaz.
    ///
    /// # Errores
    ///
    /// [`DbError::CuentaDeCorreoNoExiste`] si esa cuenta no es de esta empresa.
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn configurar_la_lectura_del_buzon(
        &self,
        empresa: CompanyId,
        cuenta: EmailAccountId,
        servidor: Option<&str>,
        encendido: bool,
    ) -> Result<(), DbError> {
        self.con_dominio(|tx| {
            let filas = tx.execute(
                "UPDATE email_account
                    SET imap_host = ?3,
                        inbox_scan_enabled = ?4,
                        updated_at = ?5
                  WHERE id = ?1 AND company_id = ?2",
                params![
                    cuenta.to_string(),
                    empresa.to_string(),
                    servidor,
                    i32::from(encendido),
                    ahora(),
                ],
            )?;
            if filas == 0 {
                return Err(DbError::CuentaDeCorreoNoExiste);
            }
            auditar(
                tx,
                &empresa.to_string(),
                if encendido {
                    "inbox_scan_enabled"
                } else {
                    "inbox_scan_disabled"
                },
                "email_account",
                &cuenta.to_string(),
            )?;
            Ok(())
        })
    }

    /// Escribe las bajas que llegaron y avanza la marca del buzón.
    ///
    /// `hasta_uid` es el UID más alto que se **miró**, no el más alto que dio
    /// una baja: si no se guardara así, los correos que no eran bajas se
    /// volverían a leer en cada pasada para nada.
    ///
    /// `para_mirar` entra sólo para el resumen y para la bitácora. **No escribe
    /// ninguna supresión**: lo dudoso lo mira una persona.
    ///
    /// # Errores
    ///
    /// [`DbError::CuentaDeCorreoNoExiste`] si esa cuenta no es de esta empresa.
    /// [`DbError::Sqlite`] si algo falla; entonces **no entra nada**, ni las
    /// bajas ni la marca.
    pub fn aplicar_bajas_del_buzon(
        &self,
        empresa: CompanyId,
        cuenta: EmailAccountId,
        bajas: &[BajaQueLlego],
        para_mirar: u32,
        hasta_uid: Option<u32>,
    ) -> Result<ResumenDeLaRevision, DbError> {
        let cuando = ahora();
        self.con_dominio(|tx| {
            // La cuenta primero: si no es de esta empresa, no se escribe nada.
            let suya: i64 = tx.query_row(
                "SELECT count(*) FROM email_account WHERE id = ?1 AND company_id = ?2",
                params![cuenta.to_string(), empresa.to_string()],
                |f| f.get(0),
            )?;
            if suya == 0 {
                return Err(DbError::CuentaDeCorreoNoExiste);
            }

            let mut resumen = ResumenDeLaRevision {
                para_mirar,
                ..ResumenDeLaRevision::default()
            };

            for baja in bajas {
                // Las mismas columnas y los mismos valores que
                // `suprimir_direccion`, y escritas aquí y no llamando a ese
                // método **porque el cerrojo de la conexión no es reentrante**:
                // llamar a un método público de `Db` desde dentro de un
                // `con_dominio` cuelga el programa sin error ni traza. Está en
                // el CLAUDE.md como la segunda trampa de la Fase 6.
                let filas = tx.execute(
                    "INSERT OR IGNORE INTO suppression_entry
                         (id, company_id, channel, address_normalized, scope,
                          request_id, reason, origin, notes, created_at, created_by)
                     VALUES (?1, ?2, ?3, ?4, 'channel', NULL, ?5, ?6, NULL, ?7, NULL)",
                    params![
                        arles_core::ids::SuppressionEntryId::nuevo().to_string(),
                        empresa.to_string(),
                        Canal::Correo.como_texto(),
                        baja.direccion,
                        MotivoDeSupresion::Baja.como_texto(),
                        OrigenDeSupresion::PeticionPorCorreo.como_texto(),
                        cuando,
                    ],
                )?;
                if filas == 1 {
                    resumen.dadas_de_baja += 1;
                    // Una línea por baja, con el UID del correo que la pidió:
                    // es lo único que permite, meses después, contestar «por
                    // qué está esta dirección en la lista».
                    auditar(
                        tx,
                        &empresa.to_string(),
                        "unsubscribe_from_inbox",
                        "suppression_entry",
                        &format!(
                            "{} (uid {}, {})",
                            baja.direccion,
                            baja.uid,
                            if baja.por_el_asunto {
                                "boton"
                            } else {
                                "respuesta"
                            }
                        ),
                    )?;
                } else {
                    resumen.ya_estaban += 1;
                }
            }

            // La marca avanza **sólo hacia arriba**. Un `hasta_uid` menor que
            // el guardado dejaría volver a mirar correos ya procesados y a
            // registrar las mismas bajas otra vez en la bitácora.
            if let Some(uid) = hasta_uid {
                tx.execute(
                    "UPDATE email_account
                        SET inbox_last_uid = max(coalesce(inbox_last_uid, 0), ?3),
                            updated_at = ?4
                      WHERE id = ?1 AND company_id = ?2",
                    params![
                        cuenta.to_string(),
                        empresa.to_string(),
                        i64::from(uid),
                        cuando,
                    ],
                )?;
            }

            Ok(resumen)
        })
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::ClaveMaestra;
    use crate::correo::LimitesDeLaCuenta;
    use arles_core::correo::BorradorDeCuenta;
    use arles_core::correo::DatosDeLaCuenta;
    use arles_core::empresa::{BorradorDeEmpresa, DatosDeEmpresa};

    fn base() -> (tempfile::TempDir, Db, CompanyId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(
                &DatosDeEmpresa::validar(&BorradorDeEmpresa {
                    nombre_comercial: "TELEMETRY".into(),
                    pais: "MX".into(),
                    zona_horaria: "America/Mexico_City".into(),
                    correo_corporativo: "hola@telemetry.mx".into(),
                    sitio_web: String::new(),
                })
                .expect("válidos"),
            )
            .expect("guarda la empresa");
        (dir, db, empresa)
    }

    fn alta(db: &Db, empresa: CompanyId) -> EmailAccountId {
        let id = EmailAccountId::nuevo();
        let datos = DatosDeLaCuenta::validar(&BorradorDeCuenta {
            servidor: "smtp.telemetrymx.com".into(),
            puerto: "587".into(),
            cifrado: "starttls".into(),
            usuario: "enlace@telemetrymx.com".into(),
            remitente: "enlace@telemetrymx.com".into(),
            nombre_visible: "Enlace".into(),
        })
        .expect("la cuenta es válida");
        db.alta_de_cuenta_de_correo(
            empresa,
            id,
            &datos,
            LimitesDeLaCuenta::default(),
            "llavero://cuenta-1",
        )
        .expect("alta");
        id
    }

    #[test]
    fn un_buzon_apagado_no_se_lee() {
        let (_d, db, empresa) = base();
        let cuenta = alta(&db, empresa);

        // Nace apagado: es la decisión de la V18.
        assert!(db.buzones_que_se_leen(empresa).expect("lee").is_empty());

        // Encendido pero sin servidor: tampoco, porque no se puede leer.
        db.configurar_la_lectura_del_buzon(empresa, cuenta, None, true)
            .expect("configura");
        assert!(
            db.buzones_que_se_leen(empresa).expect("lee").is_empty(),
            "sin servidor no hay buzón que leer"
        );

        // Con servidor y encendido, sí.
        db.configurar_la_lectura_del_buzon(empresa, cuenta, Some("imap.telemetrymx.com"), true)
            .expect("configura");
        let buzones = db.buzones_que_se_leen(empresa).expect("lee");
        assert_eq!(buzones.len(), 1);
        let uno = buzones.first().expect("uno");
        assert_eq!(uno.servidor, "imap.telemetrymx.com");
        assert_eq!(uno.direccion, "enlace@telemetrymx.com");
        assert_eq!(uno.referencia_credencial, "llavero://cuenta-1");
        assert_eq!(uno.ultimo_uid, None, "nunca se ha leído");

        // Y apagarlo lo saca de la lista sin perder el servidor.
        db.configurar_la_lectura_del_buzon(empresa, cuenta, Some("imap.telemetrymx.com"), false)
            .expect("apaga");
        assert!(db.buzones_que_se_leen(empresa).expect("lee").is_empty());
    }

    #[test]
    fn una_baja_del_buzon_entra_con_su_origen_y_no_se_puede_levantar() {
        let (_d, db, empresa) = base();
        let cuenta = alta(&db, empresa);

        let resumen = db
            .aplicar_bajas_del_buzon(
                empresa,
                cuenta,
                &[BajaQueLlego {
                    direccion: "ana@ejemplo.mx".to_owned(),
                    uid: 7,
                    por_el_asunto: true,
                }],
                0,
                Some(9),
            )
            .expect("aplica");
        assert_eq!(resumen.dadas_de_baja, 1);
        assert_eq!(resumen.ya_estaban, 0);

        // Está en la lista, con el origen que la distingue de una apuntada a
        // mano, y con el motivo que no se puede levantar con un botón.
        let (motivo, origen): (String, String) = db
            .con(|conn| {
                conn.query_row(
                    "SELECT reason, origin FROM suppression_entry
                      WHERE address_normalized = 'ana@ejemplo.mx'",
                    [],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("consulta");
        assert_eq!(motivo, MotivoDeSupresion::Baja.como_texto());
        assert_eq!(origen, OrigenDeSupresion::PeticionPorCorreo.como_texto());
        assert!(!MotivoDeSupresion::Baja.se_puede_levantar());
    }

    /// **Suprimir es idempotente, y la cifra lo dice.**
    ///
    /// Una segunda pasada sobre el mismo correo no es un error, pero la
    /// pantalla no puede decir «1 baja nueva» cuando es la de ayer.
    #[test]
    fn la_segunda_vez_cuenta_como_ya_estaba() {
        let (_d, db, empresa) = base();
        let cuenta = alta(&db, empresa);
        let baja = [BajaQueLlego {
            direccion: "ana@ejemplo.mx".to_owned(),
            uid: 7,
            por_el_asunto: true,
        }];

        let primera = db
            .aplicar_bajas_del_buzon(empresa, cuenta, &baja, 0, Some(7))
            .expect("primera");
        let segunda = db
            .aplicar_bajas_del_buzon(empresa, cuenta, &baja, 0, Some(7))
            .expect("segunda");

        assert_eq!((primera.dadas_de_baja, primera.ya_estaban), (1, 0));
        assert_eq!((segunda.dadas_de_baja, segunda.ya_estaban), (0, 1));

        // Y la bitácora tiene **una** línea, no dos: la segunda pasada no
        // apuntó nada porque no cambió nada.
        let lineas: i64 = db
            .con(|conn| {
                conn.query_row(
                    "SELECT count(*) FROM audit_log WHERE action = 'unsubscribe_from_inbox'",
                    [],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(lineas, 1);
    }

    /// ── LA MARCA SÓLO AVANZA, Y ESTO ES LO QUE LO MIDE ────────────────────
    ///
    /// Un `hasta_uid` menor que el guardado dejaría volver a mirar correos ya
    /// procesados, y cada pasada registraría las mismas bajas otra vez.
    #[test]
    fn la_marca_del_buzon_no_retrocede() {
        let (_d, db, empresa) = base();
        let cuenta = alta(&db, empresa);
        db.configurar_la_lectura_del_buzon(empresa, cuenta, Some("imap.x.mx"), true)
            .expect("configura");

        db.aplicar_bajas_del_buzon(empresa, cuenta, &[], 0, Some(50))
            .expect("avanza");
        db.aplicar_bajas_del_buzon(empresa, cuenta, &[], 0, Some(20))
            .expect("no retrocede");

        let uno = db
            .buzones_que_se_leen(empresa)
            .expect("lee")
            .first()
            .cloned()
            .expect("hay buzón");
        assert_eq!(uno.ultimo_uid, Some(50));
    }

    /// Lo dudoso **no escribe ninguna supresión**, y sale en el resumen.
    #[test]
    fn lo_que_hay_que_mirar_a_mano_no_da_de_baja_a_nadie() {
        let (_d, db, empresa) = base();
        let cuenta = alta(&db, empresa);

        let resumen = db
            .aplicar_bajas_del_buzon(empresa, cuenta, &[], 3, Some(9))
            .expect("aplica");
        assert_eq!(resumen.para_mirar, 3);
        assert_eq!(resumen.dadas_de_baja, 0);

        let cuantas: i64 = db
            .con(|conn| conn.query_row("SELECT count(*) FROM suppression_entry", [], |f| f.get(0)))
            .expect("cuenta");
        assert_eq!(cuantas, 0, "un dudoso no da de baja a nadie");
    }

    /// Una cuenta de otra empresa no se toca, y **no se escribe nada** — ni la
    /// baja ni la marca. Es la regla de ADR-0008 aplicada aquí.
    #[test]
    fn una_cuenta_de_otra_empresa_no_deja_escribir_nada() {
        let (_d, db, empresa) = base();
        let ajena = EmailAccountId::nuevo();

        let fallo = db.aplicar_bajas_del_buzon(
            empresa,
            ajena,
            &[BajaQueLlego {
                direccion: "ana@ejemplo.mx".to_owned(),
                uid: 7,
                por_el_asunto: true,
            }],
            0,
            Some(9),
        );
        assert!(matches!(fallo, Err(DbError::CuentaDeCorreoNoExiste)));

        let cuantas: i64 = db
            .con(|conn| conn.query_row("SELECT count(*) FROM suppression_entry", [], |f| f.get(0)))
            .expect("cuenta");
        assert_eq!(cuantas, 0, "la transacción entera se deshizo");
    }
}
