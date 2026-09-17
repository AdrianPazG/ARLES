//! Repositorio de la empresa operadora y del recuento de alta.
//!
//! Dos operaciones y una consulta. Las consultas son de las que se leen enteras
//! —nada de ORM— porque es la propiedad que hace auditable esta capa (ADR-0002).

use arles_core::empresa::{BorradorDeEmpresa, DatosDeEmpresa};
use arles_core::ids::CompanyId;
use arles_core::onboarding::RecuentoDeAlta;
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::db::Db;
use crate::error::DbError;

/// La empresa tal como está guardada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmpresaGuardada {
    pub id: CompanyId,
    pub datos: DatosDeEmpresa,
}

/// Marca de tiempo ISO-8601 en UTC.
///
/// La zona de la empresa **no** entra aquí: lo almacenado es siempre UTC y la
/// zona se aplica al mostrar y al calcular ventanas. Mezclar las dos cosas en
/// la columna es cómo se acaba con marcas de tiempo que significan algo
/// distinto según quién las escribió.
pub(crate) fn ahora() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        // `Rfc3339` no falla sobre un `OffsetDateTime`, pero el §138 deniega
        // `expect`: ante lo imposible, un valor que se nota.
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

/// Escribe una línea en la bitácora inmutable (§91).
///
/// Va **en la misma transacción** que el cambio que describe. Si se registrara
/// aparte, un fallo entre las dos escrituras dejaría un cambio sin rastro, que
/// es justo lo que una bitácora existe para impedir.
pub(crate) fn auditar(
    conn: &Connection,
    empresa: &str,
    accion: &str,
    entidad: &str,
    entidad_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO audit_log (id, company_id, actor, action, entity_kind, entity_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            uuid_nuevo(),
            empresa,
            // v1.2.0 es monousuario en un equipo (D-4). Cuando haya cuentas,
            // este valor pasa a ser quién estaba dentro.
            "usuario_local",
            accion,
            entidad,
            entidad_id,
            ahora(),
        ],
    )?;
    Ok(())
}

fn uuid_nuevo() -> String {
    // Los identificadores los genera el dominio; aquí solo hace falta uno para
    // la bitácora, y `CompanyId` sirve de generador de UUID v7 sin añadir una
    // dependencia más a este crate.
    CompanyId::nuevo().to_string()
}

impl Db {
    /// La empresa configurada, si la hay.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si lo almacenado ya no pasa la validación del
    /// dominio —solo posible editando la base por fuera—, y [`DbError::Sqlite`]
    /// si la consulta falla.
    pub fn empresa(&self) -> Result<Option<EmpresaGuardada>, DbError> {
        let fila = self.con(|conn| {
            conn.query_row(
                "SELECT id, commercial_name, country, timezone, corporate_email, website
                 FROM company ORDER BY created_at LIMIT 1",
                [],
                |f| {
                    Ok((
                        f.get::<_, String>(0)?,
                        BorradorDeEmpresa {
                            nombre_comercial: f.get(1)?,
                            pais: f.get(2)?,
                            zona_horaria: f.get(3)?,
                            correo_corporativo: f.get(4)?,
                            sitio_web: f.get::<_, Option<String>>(5)?.unwrap_or_default(),
                        },
                    ))
                },
            )
            .optional()
        })?;

        let Some((id, borrador)) = fila else {
            return Ok(None);
        };

        // Se valida **al leer**, no solo al escribir. Una fila editada a mano en
        // la base con una zona horaria inexistente rompería el cálculo de
        // ventanas mucho más tarde y muy lejos de aquí.
        let datos = DatosDeEmpresa::validar(&borrador).map_err(|_| DbError::DatoInvalido {
            motivo: "la empresa almacenada no pasa la validación del dominio",
        })?;

        Ok(Some(EmpresaGuardada {
            id: id.parse().map_err(|_| DbError::DatoInvalido {
                motivo: "el identificador de la empresa no es un UUID",
            })?,
            datos,
        }))
    }

    /// Guarda la empresa: la crea si no existe, la actualiza si ya está.
    ///
    /// v1.2.0 opera **una sola empresa** por instalación (D-4). El método lo
    /// impone en vez de confiar en que la interfaz no llame dos veces: si ya
    /// hay una fila, se actualiza esa.
    ///
    /// Devuelve el identificador y deja constancia en `audit_log`.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla. En ese caso **no se escribe
    /// nada**: la transacción revierte el cambio y su línea de bitácora juntos.
    pub fn guardar_empresa(&self, datos: &DatosDeEmpresa) -> Result<CompanyId, DbError> {
        let id = self.con(|conn| {
            let tx = conn.unchecked_transaction()?;

            let existente: Option<String> = tx
                .query_row(
                    "SELECT id FROM company ORDER BY created_at LIMIT 1",
                    [],
                    |f| f.get(0),
                )
                .optional()?;

            let ahora = ahora();
            let sitio = datos.sitio_web();

            let (id, accion) = match existente {
                Some(id) => {
                    tx.execute(
                        "UPDATE company
                         SET commercial_name = ?2, country = ?3, timezone = ?4,
                             corporate_email = ?5, website = ?6, updated_at = ?7
                         WHERE id = ?1",
                        params![
                            id,
                            datos.nombre_comercial(),
                            datos.pais(),
                            datos.zona_horaria(),
                            datos.correo_corporativo().raw(),
                            sitio,
                            ahora,
                        ],
                    )?;
                    (id, "empresa.actualizada")
                }
                None => {
                    let id = CompanyId::nuevo().to_string();
                    tx.execute(
                        "INSERT INTO company
                             (id, commercial_name, country, timezone, corporate_email,
                              website, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                        params![
                            id,
                            datos.nombre_comercial(),
                            datos.pais(),
                            datos.zona_horaria(),
                            datos.correo_corporativo().raw(),
                            sitio,
                            ahora,
                        ],
                    )?;
                    (id, "empresa.configurada")
                }
            };

            auditar(&tx, &id, accion, "company", &id)?;
            tx.commit()?;
            Ok(id)
        })?;

        id.parse().map_err(|_| DbError::DatoInvalido {
            motivo: "el identificador generado para la empresa no es un UUID",
        })
    }

    /// Cuántas cosas de cada tipo hay, para la lista de alta.
    ///
    /// Una sola consulta con subconsultas en vez de seis viajes: es una
    /// pantalla que se pinta en cada arranque.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn recuento_de_alta(&self) -> Result<RecuentoDeAlta, DbError> {
        self.con(|conn| {
            conn.query_row(
                "SELECT (SELECT count(*) FROM company),
                        (SELECT count(*) FROM email_account),
                        -- Los contactos borrados no cuentan: el paso pregunta
                        -- si hay a quién escribir, no cuántas filas hay.
                        (SELECT count(*) FROM contact WHERE deleted_at IS NULL),
                        (SELECT count(*) FROM template),
                        (SELECT count(*) FROM execution_window),
                        (SELECT count(*) FROM campaign)",
                [],
                |f| {
                    Ok(RecuentoDeAlta {
                        empresas: f.get(0)?,
                        remitentes: f.get(1)?,
                        contactos: f.get(2)?,
                        plantillas: f.get(3)?,
                        ventanas: f.get(4)?,
                        campanas: f.get(5)?,
                    })
                },
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conexion::ClaveMaestra;

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    fn datos(nombre: &str) -> DatosDeEmpresa {
        DatosDeEmpresa::validar(&BorradorDeEmpresa {
            nombre_comercial: nombre.into(),
            pais: "MX".into(),
            zona_horaria: "America/Mexico_City".into(),
            correo_corporativo: "hola@telemetry.mx".into(),
            sitio_web: "https://telemetry.mx".into(),
        })
        .expect("válidos")
    }

    #[test]
    fn una_base_nueva_no_tiene_empresa() {
        let (_d, db) = base();
        assert_eq!(db.empresa().expect("consulta"), None);
    }

    #[test]
    fn guardar_y_leer_devuelve_lo_mismo() {
        let (_d, db) = base();
        let d = datos("TELEMETRY INSIGHT");
        let id = db.guardar_empresa(&d).expect("guarda");

        let leida = db.empresa().expect("consulta").expect("hay empresa");
        assert_eq!(leida.id, id);
        assert_eq!(leida.datos, d);
    }

    /// v1.2.0 opera una sola empresa. Guardar dos veces actualiza; no crea una
    /// segunda fila que después nadie sabría cuál es la buena.
    #[test]
    fn guardar_dos_veces_actualiza_en_vez_de_duplicar() {
        let (_d, db) = base();
        let primero = db.guardar_empresa(&datos("PRIMERO")).expect("guarda");
        let segundo = db.guardar_empresa(&datos("SEGUNDO")).expect("guarda");

        assert_eq!(primero, segundo, "debería ser la misma empresa");
        assert_eq!(db.recuento_de_alta().expect("recuento").empresas, 1);
        assert_eq!(
            db.empresa()
                .expect("consulta")
                .expect("hay")
                .datos
                .nombre_comercial(),
            "SEGUNDO"
        );
    }

    #[test]
    fn el_sitio_web_ausente_se_guarda_como_nulo() {
        let (_d, db) = base();
        let d = DatosDeEmpresa::validar(&BorradorDeEmpresa {
            nombre_comercial: "SIN SITIO".into(),
            pais: "MX".into(),
            zona_horaria: "UTC".into(),
            correo_corporativo: "hola@telemetry.mx".into(),
            sitio_web: String::new(),
        })
        .expect("válidos");
        db.guardar_empresa(&d).expect("guarda");

        let nulo: bool = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT website IS NULL FROM company", [], |f| f.get(0))
            })
            .expect("consulta");
        assert!(nulo, "un sitio vacío debe guardarse como NULL, no como ''");
        assert_eq!(
            db.empresa().expect("lee").expect("hay").datos.sitio_web(),
            None
        );
    }

    /// §91: configurar la empresa deja rastro, y el rastro no se puede borrar.
    #[test]
    fn guardar_deja_constancia_en_la_bitacora() {
        let (_d, db) = base();
        db.guardar_empresa(&datos("TELEMETRY")).expect("guarda");
        db.guardar_empresa(&datos("TELEMETRY II")).expect("guarda");

        let acciones: Vec<String> = db
            .ejecutar_en_pruebas(|c| {
                let mut s = c.prepare("SELECT action FROM audit_log ORDER BY created_at, rowid")?;
                let v = s
                    .query_map([], |f| f.get::<_, String>(0))?
                    .filter_map(Result::ok)
                    .collect::<Vec<_>>();
                Ok(v)
            })
            .expect("consulta");

        assert_eq!(acciones, ["empresa.configurada", "empresa.actualizada"]);
    }

    /// Una fila editada por fuera con una zona horaria inexistente rompería el
    /// cálculo de ventanas mucho más tarde. Se detecta al leer.
    #[test]
    fn una_empresa_corrupta_se_nombra_en_vez_de_devolverse() {
        let (_d, db) = base();
        db.guardar_empresa(&datos("TELEMETRY")).expect("guarda");
        db.ejecutar_en_pruebas(|c| c.execute("UPDATE company SET timezone = 'America/Mexico'", []))
            .expect("estropea");

        assert!(
            matches!(db.empresa(), Err(DbError::DatoInvalido { .. })),
            "una empresa con zona inexistente no debería devolverse como válida"
        );
    }

    #[test]
    fn el_recuento_arranca_en_cero() {
        let (_d, db) = base();
        assert_eq!(
            db.recuento_de_alta().expect("recuento"),
            RecuentoDeAlta::default()
        );
    }

    #[test]
    fn el_recuento_ve_la_empresa_configurada() {
        let (_d, db) = base();
        db.guardar_empresa(&datos("TELEMETRY")).expect("guarda");
        let r = db.recuento_de_alta().expect("recuento");
        assert_eq!(r.empresas, 1);
        assert_eq!(r.contactos, 0);
    }

    /// El paso de contactos pregunta si hay a quién escribir. Un contacto
    /// borrado no cuenta, o la lista diría que está hecho con la agenda vacía.
    #[test]
    fn el_recuento_no_cuenta_contactos_borrados() {
        let (_d, db) = base();
        let id = db.guardar_empresa(&datos("TELEMETRY")).expect("guarda");
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                // Sin dirección: desde la V3 el correo vive en
                // `contact_channel`, no en `contact` (L-2).
                "INSERT INTO contact (id, company_id, created_at, updated_at, deleted_at)
                 VALUES ('x1', ?1, '2026-09-15T00:00:00Z', '2026-09-15T00:00:00Z',
                         '2026-09-15T00:00:00Z')",
                params![id.to_string()],
            )
        })
        .expect("inserta");

        assert_eq!(db.recuento_de_alta().expect("recuento").contactos, 0);
    }
}
