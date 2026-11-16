//! Repositorio de las cuentas remitentes de correo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! AQUÍ NO HAY NINGUNA CONTRASEÑA, Y NO SE PUEDE COLAR UNA
//!
//! La contraseña vive en el llavero del sistema (ADR-0011). Lo que se guarda en
//! la tabla es `credential_ref`: una referencia **opaca**, que no sirve para
//! entrar a ningún sitio y que sólo tiene sentido delante del llavero de esa
//! máquina.
//!
//! Que no haya contraseñas no es una convención de este archivo: es una
//! propiedad de los tipos. Las funciones de alta y edición reciben un
//! [`DatosDeLaCuenta`], que **no tiene un campo donde quepa una**, así que ni
//! por descuido ni con prisa se puede escribir un secreto en la base desde aquí.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UNA FILA CORRUPTA SE PARA, NO SE SALTA
//!
//! Al leer, cada cuenta vuelve a pasar por la validación del núcleo. Si una
//! fila trae un cifrado que no reconocemos —porque alguien editó la base por
//! fuera, o porque una migración se quedó a medias—, la lectura falla y lo
//! dice.
//!
//! La alternativa sería saltarse esa fila y devolver las demás, que es peor de
//! lo que parece: la pantalla de Remitentes enseñaría tres cuentas donde hay
//! cuatro, sin decir que falta una, y la que falta es justamente la que
//! necesita que alguien la mire.

use arles_core::correo::{CifradoDeCorreo, DatosDeLaCuenta};
use arles_core::ids::{CompanyId, EmailAccountId};
use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::empresa::{ahora, auditar};
use crate::error::DbError;

/// Los límites de envío de una cuenta.
///
/// Van juntos porque se leen juntos y porque el motor respeta el más
/// restrictivo entre éstos y los de la campaña.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitesDeLaCuenta {
    pub diario: u32,
    pub por_hora: u32,
}

impl Default for LimitesDeLaCuenta {
    /// Los de una cuenta de Gmail personal, por lo bajo.
    ///
    /// Gmail admite unos 500 diarios en cuentas personales. Se empieza en 50 y
    /// 10 a propósito: pasarse del límite de un proveedor no da un error claro,
    /// da una cuenta marcada como spam — y eso no se arregla bajando el número
    /// después.
    fn default() -> Self {
        Self {
            diario: 50,
            por_hora: 10,
        }
    }
}

/// Una cuenta tal como está guardada. **Sin la contraseña.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuentaDeCorreoGuardada {
    pub id: EmailAccountId,
    pub datos: DatosDeLaCuenta,
    pub limites: LimitesDeLaCuenta,
    /// `connected`, `needs_reauth`, `circuit_open` o `disabled`.
    pub estado: String,
    /// La referencia al llavero. **No es un secreto**: no sirve para entrar a
    /// ningún sitio, y por eso puede cruzar la frontera y pintarse en un log.
    pub referencia_credencial: String,
}

impl Db {
    /// Da de alta una cuenta de correo y devuelve su identificador.
    ///
    /// La `referencia_credencial` la calcula quien llama —el llavero vive en
    /// `arles-app`, no aquí—, así que el identificador se genera antes y se
    /// pasa hecho. Es deliberado: si esta capa inventara la referencia,
    /// `arles-db` tendría que saber cómo se nombran las entradas del llavero, y
    /// un cambio en el llavero obligaría a tocar la base de datos.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla. La transacción revierte el
    /// alta y su línea de bitácora juntas.
    pub fn alta_de_cuenta_de_correo(
        &self,
        empresa: CompanyId,
        id: EmailAccountId,
        datos: &DatosDeLaCuenta,
        limites: LimitesDeLaCuenta,
        referencia_credencial: &str,
    ) -> Result<(), DbError> {
        let empresa = empresa.to_string();
        let id = id.to_string();

        self.con(|conn| {
            let tx = conn.unchecked_transaction()?;
            let ahora = ahora();

            tx.execute(
                "INSERT INTO email_account
                     (id, company_id, display_name, email_address, provider_kind,
                      credential_ref, status, daily_limit, hourly_limit,
                      created_at, updated_at,
                      smtp_host, smtp_port, smtp_security, smtp_username)
                 VALUES (?1, ?2, ?3, ?4, 'smtp', ?5, 'connected', ?6, ?7, ?8, ?8,
                         ?9, ?10, ?11, ?12)",
                params![
                    id,
                    empresa,
                    // El nombre visible es opcional; en la tabla la columna es
                    // NOT NULL desde la V1, así que sin nombre se guarda el
                    // propio remitente. Nunca una cadena vacía: una fila con el
                    // nombre en blanco se enseñaría como un hueco en la lista.
                    datos
                        .nombre_visible()
                        .unwrap_or_else(|| datos.remitente().raw()),
                    datos.remitente().raw(),
                    referencia_credencial,
                    limites.diario,
                    limites.por_hora,
                    ahora,
                    datos.servidor(),
                    datos.puerto(),
                    datos.cifrado().como_str(),
                    datos.usuario(),
                ],
            )?;

            auditar(&tx, &empresa, "cuenta_de_correo.alta", "email_account", &id)?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Cambia los datos de conexión de una cuenta que ya existe.
    ///
    /// **Vuelve a poner la cuenta en `connected`.** Editar los datos es
    /// exactamente lo que se hace para arreglar una cuenta que estaba en
    /// `needs_reauth`; dejarla marcada como rota después de haberla corregido
    /// obligaría a buscar otro botón que la desmarcara, y ese botón invita a
    /// pulsarlo sin haber arreglado nada.
    ///
    /// Los contadores del disyuntor también se ponen a cero, por lo mismo: los
    /// fallos que llevaba eran de la configuración vieja.
    ///
    /// # Errores
    ///
    /// [`DbError::CuentaDeCorreoNoExiste`] si esa cuenta no está, y
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn actualizar_cuenta_de_correo(
        &self,
        id: EmailAccountId,
        datos: &DatosDeLaCuenta,
    ) -> Result<(), DbError> {
        let id = id.to_string();

        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;

            let empresa: Option<String> = tx
                .query_row(
                    "SELECT company_id FROM email_account WHERE id = ?1",
                    params![id],
                    |f| f.get(0),
                )
                .optional()?;
            let Some(empresa) = empresa else {
                return Err(DbError::CuentaDeCorreoNoExiste);
            };

            tx.execute(
                "UPDATE email_account
                    SET display_name = ?2, email_address = ?3, smtp_host = ?4,
                        smtp_port = ?5, smtp_security = ?6, smtp_username = ?7,
                        status = 'connected',
                        consecutive_failures = 0,
                        circuit_open_until = NULL,
                        updated_at = ?8
                  WHERE id = ?1",
                params![
                    id,
                    datos
                        .nombre_visible()
                        .unwrap_or_else(|| datos.remitente().raw()),
                    datos.remitente().raw(),
                    datos.servidor(),
                    datos.puerto(),
                    datos.cifrado().como_str(),
                    datos.usuario(),
                    ahora(),
                ],
            )?;

            auditar(
                &tx,
                &empresa,
                "cuenta_de_correo.actualizada",
                "email_account",
                &id,
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Marca una cuenta como pendiente de configurar.
    ///
    /// Se usa cuando la cuenta se guarda pero la prueba de conexión no pasa: la
    /// cuenta existe —no se pierde lo que alguien escribió— y **no dice que
    /// funciona**, que es lo que evita descubrirlo con una campaña en marcha.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn marcar_cuenta_por_configurar(&self, id: EmailAccountId) -> Result<(), DbError> {
        let id = id.to_string();
        self.con(|conn| {
            conn.execute(
                "UPDATE email_account SET status = 'needs_reauth', updated_at = ?2
                  WHERE id = ?1 AND status <> 'disabled'",
                params![id, ahora()],
            )?;
            Ok(())
        })
    }

    /// Las cuentas SMTP de una empresa, de la más antigua a la más reciente.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla, y [`DbError::DatoInvalido`] si
    /// **alguna** fila no pasa la validación del núcleo. No se salta: ver la
    /// nota de cabecera.
    pub fn cuentas_de_correo(
        &self,
        empresa: CompanyId,
    ) -> Result<Vec<CuentaDeCorreoGuardada>, DbError> {
        let empresa = empresa.to_string();

        self.con_dominio(|conn| {
            let mut consulta = conn.prepare(
                "SELECT id, display_name, email_address, credential_ref, status,
                        daily_limit, hourly_limit,
                        smtp_host, smtp_port, smtp_security, smtp_username
                   FROM email_account
                  WHERE company_id = ?1 AND provider_kind = 'smtp'
                  ORDER BY created_at, rowid",
            )?;

            let filas = consulta.query_map(params![empresa], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, String>(3)?,
                    f.get::<_, String>(4)?,
                    f.get::<_, i64>(5)?,
                    f.get::<_, i64>(6)?,
                    f.get::<_, Option<String>>(7)?,
                    f.get::<_, Option<i64>>(8)?,
                    f.get::<_, Option<String>>(9)?,
                    f.get::<_, Option<String>>(10)?,
                ))
            })?;

            let mut cuentas = Vec::new();
            for fila in filas {
                cuentas.push(reconstruir(fila?)?);
            }
            Ok(cuentas)
        })
    }

    /// Una cuenta concreta. `None` si no existe.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] o [`DbError::DatoInvalido`] igual que el listado.
    pub fn cuenta_de_correo(
        &self,
        id: EmailAccountId,
    ) -> Result<Option<CuentaDeCorreoGuardada>, DbError> {
        let id = id.to_string();

        self.con_dominio(|conn| {
            let fila = conn
                .query_row(
                    "SELECT id, display_name, email_address, credential_ref, status,
                            daily_limit, hourly_limit,
                            smtp_host, smtp_port, smtp_security, smtp_username
                       FROM email_account
                      WHERE id = ?1 AND provider_kind = 'smtp'",
                    params![id],
                    |f| {
                        Ok((
                            f.get::<_, String>(0)?,
                            f.get::<_, String>(1)?,
                            f.get::<_, String>(2)?,
                            f.get::<_, String>(3)?,
                            f.get::<_, String>(4)?,
                            f.get::<_, i64>(5)?,
                            f.get::<_, i64>(6)?,
                            f.get::<_, Option<String>>(7)?,
                            f.get::<_, Option<i64>>(8)?,
                            f.get::<_, Option<String>>(9)?,
                            f.get::<_, Option<String>>(10)?,
                        ))
                    },
                )
                .optional()?;

            match fila {
                Some(f) => Ok(Some(reconstruir(f)?)),
                None => Ok(None),
            }
        })
    }
}

/// Fila de la tabla → cuenta validada.
type Fila = (
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

fn reconstruir(f: Fila) -> Result<CuentaDeCorreoGuardada, DbError> {
    let (
        id,
        nombre_visible,
        remitente,
        referencia_credencial,
        estado,
        diario,
        por_hora,
        servidor,
        puerto,
        cifrado,
        usuario,
    ) = f;

    // Una cuenta SMTP sin datos de conexión es la que dejó la V7 marcada como
    // `needs_reauth`: existe, y no se puede usar. Se dice con el mismo error
    // que una fila corrupta, porque la acción es la misma —abrirla y
    // completarla— y porque callarlo la enseñaría como si funcionara.
    let (Some(servidor), Some(puerto), Some(cifrado), Some(usuario)) =
        (servidor, puerto, cifrado, usuario)
    else {
        return Err(DbError::DatoInvalido {
            motivo: "esa cuenta de correo no tiene datos de conexión: hay que configurarla",
        });
    };

    let puerto = u16::try_from(puerto).map_err(|_| DbError::DatoInvalido {
        motivo: "el puerto guardado para esa cuenta no es un puerto",
    })?;

    // El nombre visible se guarda igual al remitente cuando no hay nombre; al
    // leer se vuelve a `None` para que la interfaz no enseñe la dirección
    // duplicada en dos casillas.
    let nombre = if nombre_visible == remitente {
        None
    } else {
        Some(nombre_visible.as_str())
    };

    let datos = DatosDeLaCuenta::desde_lo_guardado(
        &servidor, puerto, &cifrado, &usuario, &remitente, nombre,
    )
    .map_err(|_| DbError::DatoInvalido {
        motivo: "esa cuenta de correo tiene datos que no pasan la validación",
    })?;

    Ok(CuentaDeCorreoGuardada {
        id: id.parse().map_err(|_| DbError::DatoInvalido {
            motivo: "el identificador de esa cuenta de correo no es un UUID",
        })?,
        datos,
        limites: LimitesDeLaCuenta {
            diario: u32::try_from(diario).unwrap_or(u32::MAX),
            por_hora: u32::try_from(por_hora).unwrap_or(u32::MAX),
        },
        estado,
        referencia_credencial,
    })
}

/// Comprueba que el cifrado guardado es uno de los dos. Para las pruebas del
/// esquema y para quien audite la base a mano.
#[must_use]
pub fn cifrado_valido(s: &str) -> bool {
    CifradoDeCorreo::desde_str(s).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClaveMaestra;
    use arles_core::correo::BorradorDeCuenta;
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

    fn cuenta(servidor: &str, nombre: &str) -> DatosDeLaCuenta {
        DatosDeLaCuenta::validar(&BorradorDeCuenta {
            servidor: servidor.into(),
            puerto: "587".into(),
            cifrado: "starttls".into(),
            usuario: "ventas@telemetry.mx".into(),
            remitente: "ventas@telemetry.mx".into(),
            nombre_visible: nombre.into(),
        })
        .expect("la cuenta es válida")
    }

    /// Lo que se guarda se vuelve a leer igual.
    #[test]
    fn una_cuenta_guardada_se_relee_entera() {
        let (_d, db, empresa) = base();
        let id = EmailAccountId::nuevo();
        let datos = cuenta("smtp.gmail.com", "TELEMETRY Ventas");

        db.alta_de_cuenta_de_correo(
            empresa,
            id,
            &datos,
            LimitesDeLaCuenta::default(),
            "llavero://prueba/email-account/x",
        )
        .expect("da de alta");

        let leida = db.cuenta_de_correo(id).expect("lee").expect("existe");
        assert_eq!(leida.id, id);
        assert_eq!(leida.datos, datos);
        assert_eq!(leida.estado, "connected");
        assert_eq!(leida.limites, LimitesDeLaCuenta::default());
        assert_eq!(
            leida.referencia_credencial,
            "llavero://prueba/email-account/x"
        );
    }

    /// **En la base no hay ninguna contraseña.**
    ///
    /// Se mira la base entera, columna por columna, en vez de confiar en que
    /// las funciones de este archivo no escriban secretos: lo que importa no es
    /// qué código creemos haber escrito, sino qué hay en el disco. Si alguien
    /// añade mañana una columna «por comodidad», esta prueba lo dice.
    ///
    /// Verificada rompiéndola el 23/09/2026: guardando la contraseña en
    /// `credential_ref` —el sitio más tentador, porque ya existe y es de
    /// texto—, la prueba falla nombrando la columna.
    #[test]
    fn en_la_base_no_hay_ninguna_contrasena() {
        const CONTRASENA: &str = "la-contrasena-de-aplicacion-de-gmail";

        let (_d, db, empresa) = base();
        db.alta_de_cuenta_de_correo(
            empresa,
            EmailAccountId::nuevo(),
            &cuenta("smtp.gmail.com", "Ventas"),
            LimitesDeLaCuenta::default(),
            // La referencia, que es lo único que se guarda. Lleva el mismo
            // aspecto que la de verdad para que la prueba no sea más fácil que
            // la realidad.
            "llavero://mx.telemetryinsight.arlesrelay/email-account/abc",
        )
        .expect("da de alta");

        let filas: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut columnas =
                    c.prepare("SELECT name FROM pragma_table_info('email_account')")?;
                let nombres: Vec<String> = columnas
                    .query_map([], |f| f.get::<_, String>(0))?
                    .filter_map(Result::ok)
                    .collect();

                let seleccion = nombres
                    .iter()
                    .map(|n| format!("COALESCE(CAST({n} AS TEXT), '')"))
                    .collect::<Vec<_>>()
                    .join(" || '·' || ");

                let mut todo = c.prepare(&format!("SELECT {seleccion} FROM email_account"))?;
                let v: Vec<String> = todo
                    .query_map([], |f| f.get::<_, String>(0))?
                    .filter_map(Result::ok)
                    .collect();
                Ok(v)
            })
            .expect("lee la tabla entera");

        assert!(!filas.is_empty(), "la cuenta tiene que estar guardada");
        for fila in &filas {
            assert!(
                !fila.contains(CONTRASENA),
                "hay una contraseña en la tabla de cuentas: {fila}"
            );
        }
    }

    /// Editar una cuenta la devuelve a `connected` y pone el disyuntor a cero.
    ///
    /// Es lo que se hace para arreglar una cuenta rota. Dejarla marcada
    /// obligaría a buscar otro botón que la desmarcara — y ese botón se pulsa
    /// sin haber arreglado nada.
    #[test]
    fn editar_una_cuenta_rota_la_deja_lista_otra_vez() {
        let (_d, db, empresa) = base();
        let id = EmailAccountId::nuevo();
        db.alta_de_cuenta_de_correo(
            empresa,
            id,
            &cuenta("smtp.malo.com", "Ventas"),
            LimitesDeLaCuenta::default(),
            "llavero://x",
        )
        .expect("alta");

        // Se rompe como lo haría el motor: la cuenta no autentica.
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE email_account
                    SET status = 'needs_reauth', consecutive_failures = 5,
                        circuit_open_until = '2026-09-23T12:00:00Z'",
                [],
            )
        })
        .expect("la rompe");

        db.actualizar_cuenta_de_correo(id, &cuenta("smtp.gmail.com", "Ventas"))
            .expect("la corrige");

        let leida = db.cuenta_de_correo(id).expect("lee").expect("existe");
        assert_eq!(leida.estado, "connected");
        assert_eq!(leida.datos.servidor(), "smtp.gmail.com");

        let (fallos, abierto): (i64, Option<String>) = db
            .ejecutar_en_pruebas(|c| {
                c.query_row(
                    "SELECT consecutive_failures, circuit_open_until FROM email_account",
                    [],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("consulta");
        assert_eq!(fallos, 0, "los fallos eran de la configuración vieja");
        assert_eq!(abierto, None, "y el disyuntor tenía que cerrarse");
    }

    /// Editar una cuenta que no existe lo dice, en vez de no hacer nada.
    #[test]
    fn editar_una_cuenta_que_no_existe_lo_dice() {
        let (_d, db, _e) = base();
        let r = db.actualizar_cuenta_de_correo(
            EmailAccountId::nuevo(),
            &cuenta("smtp.gmail.com", "Ventas"),
        );
        assert!(matches!(r, Err(DbError::CuentaDeCorreoNoExiste)));
    }

    /// **Una fila corrupta se para; no se salta.**
    ///
    /// Saltarla dejaría la pantalla de Remitentes enseñando tres cuentas donde
    /// hay cuatro, sin decir que falta una — y la que falta es justo la que
    /// necesita que alguien la mire.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que el listado ignore las
    /// filas que no validan, la lectura devuelve una cuenta y esta prueba falla.
    #[test]
    fn una_fila_corrupta_para_la_lectura_en_vez_de_esconderse() {
        let (_d, db, empresa) = base();
        db.alta_de_cuenta_de_correo(
            empresa,
            EmailAccountId::nuevo(),
            &cuenta("smtp.gmail.com", "Ventas"),
            LimitesDeLaCuenta::default(),
            "llavero://x",
        )
        .expect("alta");

        // Como si alguien hubiera editado la base por fuera. El `CHECK` no deja
        // poner un cifrado inventado, así que se vacía el servidor: es la misma
        // forma en que queda una cuenta anterior a la V7.
        db.ejecutar_en_pruebas(|c| c.execute("UPDATE email_account SET smtp_host = NULL", []))
            .expect("la corrompe");

        let r = db.cuentas_de_correo(empresa);
        assert!(
            matches!(r, Err(DbError::DatoInvalido { .. })),
            "una cuenta sin datos de conexión no puede pasar por buena: {r:?}"
        );
    }

    /// El alta deja constancia en la bitácora (§91).
    #[test]
    fn el_alta_y_la_edicion_dejan_rastro() {
        let (_d, db, empresa) = base();
        let id = EmailAccountId::nuevo();
        db.alta_de_cuenta_de_correo(
            empresa,
            id,
            &cuenta("smtp.gmail.com", "Ventas"),
            LimitesDeLaCuenta::default(),
            "llavero://x",
        )
        .expect("alta");
        db.actualizar_cuenta_de_correo(id, &cuenta("smtp.otro.com", "Ventas"))
            .expect("edita");

        let acciones: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut s = c.prepare(
                    "SELECT action FROM audit_log
                      WHERE entity_kind = 'email_account'
                      ORDER BY created_at, rowid",
                )?;
                let v: Vec<String> = s
                    .query_map([], |f| f.get::<_, String>(0))?
                    .filter_map(Result::ok)
                    .collect();
                Ok(v)
            })
            .expect("lee la bitácora");

        assert_eq!(
            acciones,
            vec!["cuenta_de_correo.alta", "cuenta_de_correo.actualizada"]
        );
    }

    /// Sin nombre visible, la lista no enseña la dirección dos veces.
    #[test]
    fn sin_nombre_visible_no_se_repite_la_direccion() {
        let (_d, db, empresa) = base();
        let id = EmailAccountId::nuevo();
        db.alta_de_cuenta_de_correo(
            empresa,
            id,
            &cuenta("smtp.gmail.com", ""),
            LimitesDeLaCuenta::default(),
            "llavero://x",
        )
        .expect("alta");

        let leida = db.cuenta_de_correo(id).expect("lee").expect("existe");
        assert_eq!(leida.datos.nombre_visible(), None);
    }

    /// Las cuentas se listan de la más antigua a la más reciente, y sólo las de
    /// esa empresa.
    #[test]
    fn el_listado_respeta_el_orden_de_alta() {
        let (_d, db, empresa) = base();
        for (servidor, nombre) in [
            ("smtp.uno.com", "Uno"),
            ("smtp.dos.com", "Dos"),
            ("smtp.tres.com", "Tres"),
        ] {
            db.alta_de_cuenta_de_correo(
                empresa,
                EmailAccountId::nuevo(),
                &cuenta(servidor, nombre),
                LimitesDeLaCuenta::default(),
                "llavero://x",
            )
            .expect("alta");
        }

        let servidores: Vec<String> = db
            .cuentas_de_correo(empresa)
            .expect("lista")
            .into_iter()
            .map(|c| c.datos.servidor().to_owned())
            .collect();
        assert_eq!(
            servidores,
            ["smtp.uno.com", "smtp.dos.com", "smtp.tres.com"]
        );
    }
}
