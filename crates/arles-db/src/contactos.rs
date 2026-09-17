//! Guardar, leer y editar contactos con sus canales.
//!
//! Entrega 3.2, tramo A. Es el **primer código que usa de verdad el esquema de
//! canales** de la migración V3: hasta aquí las tablas existían y nadie las
//! tocaba.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TRES DECISIONES QUE SE VEN EN TODO EL ARCHIVO
//!
//! **1 · Un contacto y sus canales se escriben en una sola transacción.** Un
//! contacto a medias —guardado sin sus direcciones— es una fila que engorda la
//! lista, cuenta en el total de la campaña y no recibe nada. Peor que no
//! guardarlo.
//!
//! **2 · Leer es siempre por páginas.** El objetivo son 500 000 contactos
//! (T-7). Una función que devuelva «todos» es una función que alguien llamará
//! con medio millón de filas detrás, y el problema aparecerá en el equipo de un
//! cliente y no aquí.
//!
//! **3 · Un choque de direcciones no se devuelve como error de SQLite.** El
//! índice único de `contact_channel` lo rechaza, pero «UNIQUE constraint
//! failed» no dice **qué dirección** ni **de quién** es. Se traduce a un error
//! con la dirección dentro, porque es lo único que permite a la pantalla decir
//! algo útil (§95).
//! ─────────────────────────────────────────────────────────────────────────

use arles_core::canal::Canal;
use arles_core::contacto::{CanalValidado, DatosDeContacto};
use arles_core::ids::{CompanyId, ContactId};
use rusqlite::{OptionalExtension, Transaction, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Tope de una página. Por encima, la consulta deja de caber cómodamente en
/// memoria y la tabla de la pantalla no puede dibujar tantas filas de golpe.
pub const MAX_POR_PAGINA: u32 = 1_000;

/// Un contacto tal y como está guardado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactoGuardado {
    pub id: ContactId,
    pub datos: DatosDeContacto,
    pub creado_en: String,
    pub actualizado_en: String,
}

/// Una página de la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginaDeContactos {
    pub contactos: Vec<ContactoGuardado>,
    /// Cuántos hay **en total**, no en esta página.
    ///
    /// Va aquí y no en otra llamada porque la pantalla necesita las dos cosas a
    /// la vez —«mostrando 50 de 12 400»— y pedirlas por separado deja un
    /// instante en que el total y la página no corresponden.
    pub total: u32,
}

impl Db {
    /// Da de alta un contacto con sus canales.
    ///
    /// # Errores
    ///
    /// [`DbError::DireccionEnUso`] si alguna dirección ya pertenece a otro
    /// contacto de la misma empresa, con la dirección dentro para que la
    /// pantalla pueda señalarla. [`DbError::Sqlite`] si la escritura falla.
    pub fn crear_contacto(
        &self,
        empresa: CompanyId,
        datos: &DatosDeContacto,
    ) -> Result<ContactId, DbError> {
        let id = ContactId::nuevo();
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO contact (id, company_id, first_name, last_name, source,
                                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    vacio_a_nulo(&datos.nombre),
                    vacio_a_nulo(&datos.apellido),
                    vacio_a_nulo(&datos.empresa),
                    cuando,
                ],
            )?;

            escribir_canales(&tx, empresa, id, &datos.canales, &cuando)?;
            tx.commit()?;
            Ok(())
        })?;
        Ok(id)
    }

    /// Cambia los datos y los canales de un contacto que ya existe.
    ///
    /// Los canales se reemplazan por completo por los que lleguen. Los que
    /// desaparecen **no se borran**: se marcan con `deleted_at`, y el índice
    /// único es parcial (`WHERE deleted_at IS NULL`), así que la dirección
    /// queda libre para volver a usarse. Borrarlos de verdad perdería la
    /// respuesta a «¿a qué dirección se le escribió en marzo?».
    ///
    /// # Errores
    ///
    /// [`DbError::ContactoNoExiste`] si el id no corresponde a ningún contacto
    /// vivo de esa empresa; [`DbError::DireccionEnUso`] si una dirección nueva
    /// ya es de otro contacto.
    pub fn editar_contacto(
        &self,
        empresa: CompanyId,
        id: ContactId,
        datos: &DatosDeContacto,
    ) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            let filas = tx.execute(
                "UPDATE contact
                    SET first_name = ?3, last_name = ?4, source = ?5, updated_at = ?6
                  WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                params![
                    id.to_string(),
                    empresa.to_string(),
                    vacio_a_nulo(&datos.nombre),
                    vacio_a_nulo(&datos.apellido),
                    vacio_a_nulo(&datos.empresa),
                    cuando,
                ],
            )?;
            if filas == 0 {
                return Err(DbError::ContactoNoExiste);
            }

            // Se retiran todos y se vuelven a escribir. Comparar cuáles
            // siguen, cuáles cambiaron y cuáles se fueron daría el mismo
            // resultado con tres veces el código y un caso frontera por rama.
            tx.execute(
                "UPDATE contact_channel SET deleted_at = ?2, updated_at = ?2
                  WHERE contact_id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), cuando],
            )?;

            escribir_canales(&tx, empresa, id, &datos.canales, &cuando)?;
            tx.commit()?;
            Ok(())
        })
    }

    /// La ficha de un contacto, con **todos** sus canales ordenados (L-14).
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si una fila guardada trae un canal que no
    /// reconocemos; [`DbError::Sqlite`] si la consulta falla.
    pub fn contacto(
        &self,
        empresa: CompanyId,
        id: ContactId,
    ) -> Result<Option<ContactoGuardado>, DbError> {
        self.con_dominio(|conn| {
            let cabecera: Option<(String, String, String, String, String)> = conn
                .query_row(
                    "SELECT COALESCE(first_name, ''), COALESCE(last_name, ''),
                            COALESCE(source, ''), created_at, updated_at
                       FROM contact
                      WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                    params![id.to_string(), empresa.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?, f.get(4)?)),
                )
                .optional()?;

            let Some((nombre, apellido, empresa_del_contacto, creado, actualizado)) = cabecera
            else {
                return Ok(None);
            };

            let canales = leer_canales(conn, &[id.to_string()])?
                .into_iter()
                .next()
                .map(|(_, c)| c)
                .unwrap_or_default();

            Ok(Some(ContactoGuardado {
                id,
                datos: DatosDeContacto {
                    nombre,
                    apellido,
                    empresa: empresa_del_contacto,
                    canales,
                },
                creado_en: creado,
                actualizado_en: actualizado,
            }))
        })
    }

    /// Una página de la lista, ordenada por fecha de alta descendente.
    ///
    /// `tamano` se recorta a [`MAX_POR_PAGINA`] en vez de rechazarse: una
    /// pantalla que pide de más no es un error del usuario, y devolverle un
    /// fallo la dejaría vacía en lugar de con las primeras mil filas.
    ///
    /// # Errores
    ///
    /// [`DbError::DatoInvalido`] si una fila guardada trae un canal
    /// desconocido; [`DbError::Sqlite`] si la consulta falla.
    pub fn listar_contactos(
        &self,
        empresa: CompanyId,
        desde: u32,
        tamano: u32,
    ) -> Result<PaginaDeContactos, DbError> {
        let tamano = tamano.clamp(1, MAX_POR_PAGINA);
        self.con_dominio(|conn| {
            let total: u32 = conn.query_row(
                "SELECT count(*) FROM contact WHERE company_id = ?1 AND deleted_at IS NULL",
                params![empresa.to_string()],
                |f| f.get(0),
            )?;

            let mut consulta = conn.prepare(
                "SELECT id, COALESCE(first_name, ''), COALESCE(last_name, ''),
                        COALESCE(source, ''), created_at, updated_at
                   FROM contact
                  WHERE company_id = ?1 AND deleted_at IS NULL
                  ORDER BY created_at DESC, id DESC
                  LIMIT ?2 OFFSET ?3",
            )?;
            let filas: Vec<(String, String, String, String, String, String)> = consulta
                .query_map(params![empresa.to_string(), tamano, desde], |f| {
                    Ok((
                        f.get(0)?,
                        f.get(1)?,
                        f.get(2)?,
                        f.get(3)?,
                        f.get(4)?,
                        f.get(5)?,
                    ))
                })?
                .collect::<Result<_, _>>()?;

            // Los canales de toda la página en **una** consulta. Uno por
            // contacto serían mil viajes a la base por página, que es la forma
            // habitual de que una lista vaya bien con diez filas y mal con mil.
            let ids: Vec<String> = filas.iter().map(|f| f.0.clone()).collect();
            let mut por_contacto = leer_canales(conn, &ids)?;

            let mut contactos = Vec::with_capacity(filas.len());
            for (id, nombre, apellido, empresa_del_contacto, creado, actualizado) in filas {
                let canales = por_contacto.remove(&id).unwrap_or_default();
                contactos.push(ContactoGuardado {
                    id: id.parse().map_err(|_| DbError::DatoInvalido {
                        motivo: "el identificador de un contacto no es un UUID",
                    })?,
                    datos: DatosDeContacto {
                        nombre,
                        apellido,
                        empresa: empresa_del_contacto,
                        canales,
                    },
                    creado_en: creado,
                    actualizado_en: actualizado,
                });
            }

            Ok(PaginaDeContactos { contactos, total })
        })
    }

    /// Da de baja un contacto.
    ///
    /// **Borrado lógico.** El borrado físico existe y es otra cosa: lo exige el
    /// derecho de cancelación, tiene su propio flujo en la entrega 3.4 y no se
    /// dispara desde el botón de una lista.
    ///
    /// # Errores
    ///
    /// [`DbError::ContactoNoExiste`] si no había nada que dar de baja.
    pub fn borrar_contacto(&self, empresa: CompanyId, id: ContactId) -> Result<(), DbError> {
        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();
            let filas = tx.execute(
                "UPDATE contact SET deleted_at = ?3, updated_at = ?3
                  WHERE id = ?1 AND company_id = ?2 AND deleted_at IS NULL",
                params![id.to_string(), empresa.to_string(), cuando],
            )?;
            if filas == 0 {
                return Err(DbError::ContactoNoExiste);
            }
            // Los canales se van con él: si sobrevivieran, sus direcciones
            // seguirían ocupadas y no se podría volver a dar de alta a la misma
            // persona.
            tx.execute(
                "UPDATE contact_channel SET deleted_at = ?2, updated_at = ?2
                  WHERE contact_id = ?1 AND deleted_at IS NULL",
                params![id.to_string(), cuando],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    /// Cuáles de esas direcciones están en la lista de supresión.
    ///
    /// Es el aviso de **L-13**: al cambiar el correo o el móvil de alguien, la
    /// dirección vieja puede estar dada de baja y **la nueva no hereda esa
    /// baja** — la supresión es de la dirección, no de la persona (§39).
    /// Callarlo convertiría corregir un dato en una forma silenciosa de
    /// saltarse una baja.
    ///
    /// Devuelve lo que hay, **no bloquea**: quien edita decide.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn direcciones_suprimidas(
        &self,
        empresa: CompanyId,
        canales: &[CanalValidado],
    ) -> Result<Vec<String>, DbError> {
        if canales.is_empty() {
            return Ok(Vec::new());
        }
        self.con_dominio(|conn| {
            let mut suprimidas = Vec::new();
            let mut consulta = conn.prepare(
                "SELECT 1 FROM suppression_entry
                  WHERE company_id = ?1 AND channel = ?2 AND address_normalized = ?3",
            )?;
            for c in canales {
                let hay: Option<i64> = consulta
                    .query_row(
                        params![
                            empresa.to_string(),
                            c.canal.como_texto(),
                            c.valor_normalizado,
                        ],
                        |f| f.get(0),
                    )
                    .optional()?;
                if hay.is_some() {
                    suprimidas.push(c.valor_normalizado.clone());
                }
            }
            Ok(suprimidas)
        })
    }
}

/// `""` se guarda como `NULL`, no como cadena vacía.
///
/// Son dos valores distintos para la base y significan lo mismo para el
/// usuario; dejar que convivan obliga a que cada consulta contemple los dos.
fn vacio_a_nulo(texto: &str) -> Option<&str> {
    let t = texto.trim();
    if t.is_empty() { None } else { Some(t) }
}

/// Inserta los canales de un contacto, traduciendo el choque de direcciones.
fn escribir_canales(
    tx: &Transaction<'_>,
    empresa: CompanyId,
    contacto: ContactId,
    canales: &[CanalValidado],
    cuando: &str,
) -> Result<(), DbError> {
    for c in canales {
        let r = tx.execute(
            "INSERT INTO contact_channel
                 (id, company_id, contact_id, channel, value_raw, value_normalized,
                  created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                arles_core::ids::ContactChannelId::nuevo().to_string(),
                empresa.to_string(),
                contacto.to_string(),
                c.canal.como_texto(),
                c.valor_raw,
                c.valor_normalizado,
                cuando,
            ],
        );
        match r {
            Ok(_) => {}
            // El índice único de `contact_channel` rechaza la fila, pero
            // «UNIQUE constraint failed» no dice qué dirección ni de quién. La
            // pantalla no puede señalar nada con eso.
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                return Err(DbError::DireccionEnUso {
                    canal: c.canal.como_texto(),
                    direccion: c.valor_normalizado.clone(),
                });
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Los canales de varios contactos, en una sola consulta y ya ordenados.
///
/// El orden sale de **la base**, no de reordenar en Rust: es el mismo criterio
/// de L-14 —correos primero, móviles después, el principal arriba— y hacerlo
/// aquí evita que la lista y la ficha puedan acabar ordenando distinto.
fn leer_canales(
    conn: &rusqlite::Connection,
    ids: &[String],
) -> Result<std::collections::HashMap<String, Vec<CanalValidado>>, DbError> {
    let mut por_contacto: std::collections::HashMap<String, Vec<CanalValidado>> =
        std::collections::HashMap::new();
    if ids.is_empty() {
        return Ok(por_contacto);
    }

    // `IN` con tantos interrogantes como ids. Interpolar los ids en el SQL
    // sería inyección aunque hoy vengan de nuestra propia consulta.
    let huecos = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT contact_id, channel, value_raw, value_normalized,
                CASE WHEN id = (
                    SELECT p.id FROM contact_channel p
                     WHERE p.contact_id = contact_channel.contact_id
                       AND p.channel = contact_channel.channel
                       AND p.deleted_at IS NULL
                     ORDER BY p.created_at, p.id LIMIT 1
                ) THEN 1 ELSE 0 END AS es_primero
           FROM contact_channel
          WHERE contact_id IN ({huecos}) AND deleted_at IS NULL
          ORDER BY contact_id,
                   CASE channel WHEN 'email' THEN 0 ELSE 1 END,
                   created_at, id"
    );

    let mut consulta = conn.prepare(&sql)?;
    let filas = consulta.query_map(rusqlite::params_from_iter(ids), |f| {
        Ok((
            f.get::<_, String>(0)?,
            f.get::<_, String>(1)?,
            f.get::<_, String>(2)?,
            f.get::<_, String>(3)?,
            f.get::<_, i64>(4)?,
        ))
    })?;

    for fila in filas {
        let (contacto, canal, raw, normalizado, primero) = fila?;
        let canal = Canal::desde_texto(&canal).map_err(|_| DbError::DatoInvalido {
            motivo: "un canal guardado no es ni correo ni WhatsApp",
        })?;
        por_contacto
            .entry(contacto)
            .or_default()
            .push(CanalValidado {
                canal,
                valor_raw: raw,
                valor_normalizado: normalizado,
                principal: primero == 1,
            });
    }
    Ok(por_contacto)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto};

    use crate::conexion::ClaveMaestra;

    fn base() -> (tempfile::TempDir, Db, CompanyId) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        let empresa = db
            .guardar_empresa(&empresa_de_prueba())
            .expect("guarda empresa");
        (dir, db, empresa)
    }

    fn empresa_de_prueba() -> arles_core::DatosDeEmpresa {
        arles_core::DatosDeEmpresa::validar(&arles_core::BorradorDeEmpresa {
            nombre_comercial: "TELEMETRY INSIGHT".into(),
            pais: "MX".into(),
            zona_horaria: "America/Mexico_City".into(),
            correo_corporativo: "hola@telemetrymx.com".into(),
            sitio_web: String::new(),
        })
        .expect("empresa válida")
    }

    fn contacto(canales: &[(Canal, &str)]) -> DatosDeContacto {
        DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: "Ana".into(),
                apellido: "Ruiz".into(),
                empresa: "Empresa SA".into(),
                canales: canales
                    .iter()
                    .map(|(c, v)| BorradorDeCanal {
                        canal: *c,
                        valor: (*v).to_owned(),
                        principal: false,
                    })
                    .collect(),
            },
            "MX",
        )
        .expect("contacto válido")
    }

    /// Mete `cuantos` contactos en **una** transacción y por SQL directo.
    ///
    /// Mil llamadas a `crear_contacto` serían mil transacciones y la prueba
    /// tardaría más de lo que nadie espera de un `cargo test`. Aquí lo que se
    /// mide es el recorte de la página, no el alta.
    fn sembrar_contactos(db: &Db, empresa: CompanyId, cuantos: u32) {
        db.ejecutar_en_pruebas(|conn| {
            let tx = conn.unchecked_transaction()?;
            for i in 0..cuantos {
                tx.execute(
                    "INSERT INTO contact (id, company_id, first_name, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![
                        ContactId::nuevo().to_string(),
                        empresa.to_string(),
                        format!("Contacto {i}"),
                        ahora(),
                    ],
                )?;
            }
            tx.commit()
        })
        .expect("siembra");
    }

    #[test]
    fn lo_guardado_se_lee_igual() {
        let (_d, db, empresa) = base();
        let datos = contacto(&[
            (Canal::Correo, "Ana@Empresa.MX"),
            (Canal::WhatsApp, "81 1234 5678"),
        ]);
        let id = db.crear_contacto(empresa, &datos).expect("crea");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        assert_eq!(leido.datos.nombre, "Ana");
        assert_eq!(leido.datos.canales.len(), 2);
        assert_eq!(
            leido
                .datos
                .canales
                .first()
                .map(|c| c.valor_normalizado.as_str()),
            Some("ana@empresa.mx")
        );
        // El original se conserva, no sólo la forma canónica.
        assert_eq!(
            leido.datos.canales.first().map(|c| c.valor_raw.as_str()),
            Some("Ana@Empresa.MX")
        );
    }

    /// **L-14 leído de la base.** El orden no se reconstruye en Rust: sale de
    /// la consulta, para que la lista y la ficha no puedan ordenar distinto.
    ///
    /// Se entra con los cuatro canales **desordenados a propósito**: mezclados
    /// los dos tipos y con el que acabará siendo principal en tercer lugar. Lo
    /// que se exige es doble:
    ///
    /// 1. que salgan agrupados —correos, luego móviles— con el principal
    ///    arriba de su grupo, y
    /// 2. que ese orden sea **exactamente** el que el núcleo dejó al validar.
    ///
    /// El punto 2 es el que importa: el núcleo ordena en memoria y la base
    /// vuelve a ordenar al leer. Si los dos criterios se separaran algún día,
    /// la ficha y la lista enseñarían los mismos canales en distinto orden sin
    /// que nada fallara. Aquí falla.
    #[test]
    fn la_ficha_devuelve_los_canales_agrupados_y_con_el_principal_arriba() {
        let (_d, db, empresa) = base();
        let datos = contacto(&[
            (Canal::WhatsApp, "+528100000002"),
            (Canal::Correo, "alta@empresa.mx"),
            (Canal::WhatsApp, "+528100000001"),
            (Canal::Correo, "beta@empresa.mx"),
        ]);
        let id = db.crear_contacto(empresa, &datos).expect("crea");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        let orden: Vec<_> = leido
            .datos
            .canales
            .iter()
            .map(|c| (c.canal, c.valor_normalizado.as_str(), c.principal))
            .collect();

        // El principal de cada tipo es el **primero que el usuario escribió de
        // ese tipo**, no el alfabético ni el primero del formulario: aquí el
        // móvil …002 venía antes que el …001.
        assert_eq!(
            orden,
            vec![
                (Canal::Correo, "alta@empresa.mx", true),
                (Canal::Correo, "beta@empresa.mx", false),
                (Canal::WhatsApp, "+528100000002", true),
                (Canal::WhatsApp, "+528100000001", false),
            ]
        );

        // Y lo mismo que dejó el núcleo, canal por canal.
        let del_nucleo: Vec<_> = datos
            .canales
            .iter()
            .map(|c| (c.canal, c.valor_normalizado.as_str(), c.principal))
            .collect();
        assert_eq!(
            orden, del_nucleo,
            "la base y el núcleo ordenan los canales con criterios distintos"
        );
    }

    /// El agrupado por tipo lo hace **la consulta**, no el orden en que se
    /// escribieron las filas.
    ///
    /// La prueba de arriba no lo demuestra, y se vio: quitando el
    /// `CASE channel WHEN 'email' …` del `ORDER BY`, seguía en verde. La razón
    /// es que el núcleo ya ordena antes de insertar, así que `created_at` sola
    /// reproduce el mismo resultado y la cláusula sobraba sin que se notara.
    ///
    /// Aquí los canales se escriben **intercalados a mano**, que es lo que
    /// pasará en cuanto exista un camino que añada un canal suelto a un
    /// contacto que ya existe, sin reescribir los demás. Si el agrupado
    /// dependiera del orden de inserción, esto saldría mezclado.
    #[test]
    fn el_agrupado_por_tipo_no_depende_del_orden_en_que_se_escribieron() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "uno@empresa.mx")]))
            .expect("crea");

        // …y ahora, uno a uno y alternando tipo, como si se hubieran ido
        // añadiendo con el tiempo.
        let sueltos = [
            (Canal::WhatsApp, "+528100000001"),
            (Canal::Correo, "dos@empresa.mx"),
            (Canal::WhatsApp, "+528100000002"),
            (Canal::Correo, "tres@empresa.mx"),
        ];
        db.ejecutar_en_pruebas(|conn| {
            // El canal que ya existe se ancla a una fecha fija, porque el alta
            // lo escribió con la hora real: si se dejara, «antes» y «después»
            // dependerían de a qué hora se corre la prueba.
            conn.execute(
                "UPDATE contact_channel SET created_at = '2026-09-17T10:00:00Z'
                  WHERE contact_id = ?1",
                params![id.to_string()],
            )?;

            for (n, (canal, valor)) in sueltos.iter().enumerate() {
                conn.execute(
                    "INSERT INTO contact_channel
                         (id, company_id, contact_id, channel, value_raw,
                          value_normalized, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?6)",
                    params![
                        arles_core::ids::ContactChannelId::nuevo().to_string(),
                        empresa.to_string(),
                        id.to_string(),
                        canal.como_texto(),
                        valor,
                        // Fechas crecientes y distintas: el orden de escritura
                        // queda fijado sin ambigüedad.
                        format!("2026-09-17T10:0{n}:00Z"),
                    ],
                )?;
            }
            Ok(())
        })
        .expect("escribe los canales sueltos");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        let tipos: Vec<_> = leido.datos.canales.iter().map(|c| c.canal).collect();

        assert_eq!(
            tipos,
            vec![
                Canal::Correo,
                Canal::Correo,
                Canal::Correo,
                Canal::WhatsApp,
                Canal::WhatsApp,
            ],
            "los canales salen intercalados: el agrupado depende del orden de inserción"
        );

        // Y dentro del grupo, el más antiguo arriba y marcado como principal.
        assert_eq!(
            leido
                .datos
                .canales
                .first()
                .map(|c| (c.valor_normalizado.as_str(), c.principal)),
            Some(("uno@empresa.mx", true))
        );
        assert_eq!(
            leido
                .datos
                .canales
                .get(3)
                .map(|c| (c.valor_normalizado.as_str(), c.principal)),
            Some(("+528100000001", true))
        );
    }

    /// Dos contactos no pueden compartir dirección, y el error dice **cuál**.
    /// Con «UNIQUE constraint failed» la pantalla no puede señalar nada.
    #[test]
    fn una_direccion_repetida_da_un_error_que_dice_cual_es() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("el primero entra");

        let e = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ANA@empresa.mx")]))
            .expect_err("debe fallar");
        match e {
            DbError::DireccionEnUso { canal, direccion } => {
                assert_eq!(canal, "email");
                assert_eq!(direccion, "ana@empresa.mx");
            }
            otro => panic!("se esperaba DireccionEnUso, llegó {otro:?}"),
        }
    }

    /// **Lo que protege la transacción.** Si el segundo canal choca, el
    /// contacto **no puede quedarse guardado sin sus direcciones**: sería una
    /// fila que engorda la lista y no recibe nada.
    #[test]
    fn un_choque_en_el_segundo_canal_no_deja_el_contacto_a_medias() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ocupado@empresa.mx")]))
            .expect("el primero entra");

        let antes = db.listar_contactos(empresa, 0, 100).expect("lista").total;
        let e = db.crear_contacto(
            empresa,
            &contacto(&[
                (Canal::WhatsApp, "+528112345678"),
                (Canal::Correo, "ocupado@empresa.mx"),
            ]),
        );
        assert!(e.is_err());

        let despues = db.listar_contactos(empresa, 0, 100).expect("lista").total;
        assert_eq!(
            antes, despues,
            "quedó un contacto a medias: guardado y sin poder recibir nada"
        );
    }

    #[test]
    fn editar_cambia_los_datos_y_los_canales() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "viejo@empresa.mx")]))
            .expect("crea");

        let nuevos = contacto(&[
            (Canal::Correo, "nuevo@empresa.mx"),
            (Canal::WhatsApp, "+528112345678"),
        ]);
        db.editar_contacto(empresa, id, &nuevos).expect("edita");

        let leido = db.contacto(empresa, id).expect("lee").expect("existe");
        assert_eq!(leido.datos.canales.len(), 2);
        assert!(
            !leido
                .datos
                .canales
                .iter()
                .any(|c| c.valor_normalizado == "viejo@empresa.mx"),
            "el canal retirado sigue apareciendo"
        );
    }

    /// El canal retirado deja su dirección **libre**: el índice único es
    /// parcial. Sin esto, corregir una errata dejaría la dirección buena
    /// bloqueada por la mala para siempre.
    #[test]
    fn la_direccion_de_un_canal_retirado_se_puede_volver_a_usar() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "aana@empresa.mx")]))
            .expect("crea");

        db.editar_contacto(empresa, id, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("corrige la errata");

        // Y otra persona puede quedarse con la dirección vieja.
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "aana@empresa.mx")]))
            .expect("la dirección retirada quedó libre");
    }

    #[test]
    fn editar_un_contacto_que_no_existe_lo_dice() {
        let (_d, db, empresa) = base();
        let e = db.editar_contacto(
            empresa,
            ContactId::nuevo(),
            &contacto(&[(Canal::Correo, "ana@empresa.mx")]),
        );
        assert!(matches!(e, Err(DbError::ContactoNoExiste)));
    }

    /// Borrar es lógico y se lleva los canales: si sobrevivieran, la dirección
    /// seguiría ocupada y no se podría volver a dar de alta a esa persona.
    #[test]
    fn borrar_libera_las_direcciones_y_saca_de_la_lista() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        db.borrar_contacto(empresa, id).expect("borra");
        assert_eq!(db.listar_contactos(empresa, 0, 10).expect("lista").total, 0);
        assert!(db.contacto(empresa, id).expect("lee").is_none());

        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("la dirección quedó libre");
    }

    /// El objetivo son 500 000 contactos. Una lectura que devuelva «todos» es
    /// una lectura que alguien llamará con medio millón de filas detrás.
    #[test]
    fn la_lista_va_por_paginas_y_dice_el_total() {
        let (_d, db, empresa) = base();
        for i in 0..25 {
            db.crear_contacto(
                empresa,
                &contacto(&[(Canal::Correo, &format!("c{i}@empresa.mx"))]),
            )
            .expect("crea");
        }

        let primera = db.listar_contactos(empresa, 0, 10).expect("lista");
        assert_eq!(primera.contactos.len(), 10);
        assert_eq!(
            primera.total, 25,
            "el total es el de la empresa, no el de la página"
        );

        let ultima = db.listar_contactos(empresa, 20, 10).expect("lista");
        assert_eq!(ultima.contactos.len(), 5);

        // Sin solaparse: la primera y la última no comparten ningún contacto.
        let ids_primera: Vec<_> = primera.contactos.iter().map(|c| c.id).collect();
        assert!(!ultima.contactos.iter().any(|c| ids_primera.contains(&c.id)));
    }

    /// Pedir de más no es un error del usuario: se recorta. Devolver un fallo
    /// dejaría la pantalla vacía en vez de con las primeras mil filas.
    ///
    /// Hacen falta **más contactos que el tope** para que esto mida algo. La
    /// primera versión de esta prueba tenía uno solo: con un contacto, pedir
    /// diez mil también devuelve uno, y el recorte se podía quitar entero sin
    /// que nada se pusiera rojo. Se vio quitándolo.
    #[test]
    fn pedir_una_pagina_desmesurada_se_recorta_en_vez_de_fallar() {
        let (_d, db, empresa) = base();
        sembrar_contactos(&db, empresa, MAX_POR_PAGINA + 5);

        let p = db
            .listar_contactos(empresa, 0, MAX_POR_PAGINA * 10)
            .expect("no falla");

        assert_eq!(
            p.contactos.len() as u32,
            MAX_POR_PAGINA,
            "se pidieron {} filas y volvieron {}: el tope no está recortando",
            MAX_POR_PAGINA * 10,
            p.contactos.len()
        );
        // El total sí es el de verdad: el recorte limita lo que se trae, no lo
        // que se dice que hay. Si no, la paginación de la pantalla mentiría.
        assert_eq!(p.total, MAX_POR_PAGINA + 5);
    }

    /// La lista trae los canales de todos sus contactos, no sólo los del
    /// primero: es lo que la tabla necesita para enseñar el principal de cada
    /// uno sin una consulta por fila.
    #[test]
    fn la_lista_trae_los_canales_de_cada_contacto() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "uno@empresa.mx")]))
            .expect("crea");
        db.crear_contacto(
            empresa,
            &contacto(&[
                (Canal::Correo, "dos@empresa.mx"),
                (Canal::WhatsApp, "+528112345678"),
            ]),
        )
        .expect("crea");

        let p = db.listar_contactos(empresa, 0, 10).expect("lista");
        assert!(
            p.contactos.iter().all(|c| !c.datos.canales.is_empty()),
            "algún contacto llegó sin canales"
        );
        assert_eq!(
            p.contactos
                .iter()
                .map(|c| c.datos.canales.len())
                .sum::<usize>(),
            3
        );
    }

    // ── L-13 · el aviso de la dirección suprimida ──

    /// La supresión es **de la dirección, no de la persona**. Al cambiar el
    /// correo de alguien, la dirección nueva no hereda la baja de la vieja, y
    /// callarlo convierte corregir un dato en saltarse una baja.
    #[test]
    fn se_avisa_si_una_direccion_esta_suprimida() {
        let (_d, db, empresa) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                                reason, origin, created_at)
                 VALUES ('s1', ?1, 'email', 'baja@empresa.mx', 'unsubscribe', 'user', ?2)",
                params![empresa.to_string(), ahora()],
            )
        })
        .expect("suprime");

        let datos = contacto(&[
            (Canal::Correo, "baja@empresa.mx"),
            (Canal::WhatsApp, "+528112345678"),
        ]);
        let avisos = db
            .direcciones_suprimidas(empresa, &datos.canales)
            .expect("consulta");
        assert_eq!(avisos, vec!["baja@empresa.mx".to_owned()]);
    }

    /// El aviso **no bloquea**. Quien edita decide, y la supresión sigue
    /// haciendo su trabajo en el momento del envío (§39).
    #[test]
    fn el_aviso_no_impide_guardar() {
        let (_d, db, empresa) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO suppression_entry (id, company_id, channel, address_normalized,
                                                reason, origin, created_at)
                 VALUES ('s1', ?1, 'email', 'baja@empresa.mx', 'unsubscribe', 'user', ?2)",
                params![empresa.to_string(), ahora()],
            )
        })
        .expect("suprime");

        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "baja@empresa.mx")]))
            .expect("se puede guardar: el aviso informa, no prohíbe");
    }

    #[test]
    fn sin_canales_que_mirar_no_se_consulta_nada() {
        let (_d, db, empresa) = base();
        assert!(
            db.direcciones_suprimidas(empresa, &[])
                .expect("no falla")
                .is_empty()
        );
    }

    /// Los contactos de una empresa no se ven desde otra. Hoy hay una sola,
    /// pero el §8 deja la puerta abierta a varias y el aislamiento no se añade
    /// después: se comprueba desde el principio.
    #[test]
    fn un_contacto_de_otra_empresa_no_se_lee() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let otra = CompanyId::nuevo();
        assert!(db.contacto(otra, id).expect("lee").is_none());
        assert_eq!(db.listar_contactos(otra, 0, 10).expect("lista").total, 0);
        assert!(matches!(
            db.editar_contacto(otra, id, &contacto(&[(Canal::Correo, "x@empresa.mx")])),
            Err(DbError::ContactoNoExiste)
        ));
        assert!(matches!(
            db.borrar_contacto(otra, id),
            Err(DbError::ContactoNoExiste)
        ));
    }
}
