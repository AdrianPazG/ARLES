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
use arles_core::ids::{CompanyId, ContactId, ImportBatchId};
use rusqlite::{OptionalExtension, Transaction, params};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Tope de una página. Por encima, la consulta deja de caber cómodamente en
/// memoria y la tabla de la pantalla no puede dibujar tantas filas de golpe.
pub const MAX_POR_PAGINA: u32 = 1_000;

/// Un contacto tal y como está guardado.
///
/// **No lleva `serde`.** Este crate no conoce el formato en que viajan las
/// cosas por la IPC, y no debe: quien decide cómo se ve un contacto al otro
/// lado es `arles-app`, que lo convierte en su propio tipo. Si esta estructura
/// se serializara directamente, cambiar un nombre de columna aquí cambiaría el
/// JSON que recibe la pantalla.
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

/// Todo lo que hay que declarar para dejar constancia de una importación.
///
/// Va junto y no como ocho parámetros sueltos porque **ninguno es opcional**:
/// una importación sin origen declarado, o sin la huella del archivo, no deja
/// la prueba que la ley pide (ADR-0013 §1). Con una estructura, olvidarse de
/// uno no compila.
#[derive(Debug, Clone)]
pub struct DatosDelLote {
    /// Tal y como llegó. **Sólo metadato**: nunca se usa para escribir en disco.
    pub nombre_del_archivo: String,
    /// Huella de los bytes del archivo, para reconocerlo si vuelve.
    pub huella_del_archivo: String,
    /// El mapeo de columnas que se usó, en JSON. Sin él no se puede explicar
    /// por qué un contacto quedó con el nombre en el campo de la empresa.
    pub mapeo_en_json: String,
    /// El texto íntegro que el usuario aceptó. Su huella se calcula al guardar.
    pub texto_del_consentimiento: String,
    pub origen: arles_core::OrigenDeLaLista,
    /// Cifras del análisis, para que el registro cuadre con lo que se enseñó.
    pub total_de_filas: usize,
    pub choques: usize,
    pub invalidas: usize,
}

/// Lo que pasó al importar. Cifras, no adjetivos (§94).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenDeImportacion {
    pub lote: ImportBatchId,
    pub importados: usize,
    pub choques: usize,
    pub invalidas: usize,
    pub total_de_filas: usize,
}

/// Recorta un nombre de archivo para guardarlo.
///
/// Un nombre de 4 000 caracteres no aporta nada y sí puede reventar la
/// maquetación de cualquier pantalla que lo enseñe.
fn recortar(nombre: &str) -> String {
    nombre.chars().take(255).collect()
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
    /// Importa una tanda de contactos con su registro de origen.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// O ENTRA TODO, O NO ENTRA NADA
    ///
    /// Una transacción para el lote entero. No es la opción cómoda —confirmar
    /// cada mil sería más rápido en un archivo grande— y se elige igual:
    ///
    /// Una importación a medias deja una lista de la que **nadie sabe dónde se
    /// quedó**. Reintentarla duplica lo que ya entró; no reintentarla deja
    /// fuera a gente que el usuario cree tener. Y como los choques ya se
    /// descartaron en el análisis, que esto falle a mitad sólo puede ser por
    /// algo raro —disco lleno, base corrupta—. Ante algo raro, prefiero no
    /// dejar nada a dejar la mitad.
    ///
    /// El precio está medido y aceptado: con 500 000 contactos la transacción
    /// es larga y ARLES no responde mientras dura. Es una operación que el
    /// usuario lanza a propósito y espera a que termine, no algo que ocurra de
    /// fondo.
    /// ─────────────────────────────────────────────────────────────────────
    ///
    /// `contactos` son los que el análisis marcó como **listos**. Aquí no se
    /// vuelve a analizar: si llega uno que choca, la restricción de la base lo
    /// rechaza y **se cae el lote entero**, que es lo correcto —significa que
    /// la base cambió entre el análisis y esto, y entonces el informe que el
    /// usuario aprobó ya no describe la realidad—.
    ///
    /// # Errores
    ///
    /// [`DbError::DireccionEnUso`] si alguna dirección se ocupó entre el
    /// análisis y ahora; [`DbError::Sqlite`] si la escritura falla. En los dos
    /// casos **no se escribe nada**: ni los contactos ni el registro del lote.
    pub fn importar_contactos(
        &self,
        empresa: CompanyId,
        lote: &DatosDelLote,
        contactos: &[DatosDeContacto],
    ) -> Result<ResumenDeImportacion, DbError> {
        let id_lote = ImportBatchId::nuevo();

        self.con_dominio(|conn| {
            let tx = conn.unchecked_transaction()?;
            let cuando = ahora();

            tx.execute(
                "INSERT INTO import_batch
                     (id, company_id, original_filename, stored_filename, file_hash,
                      column_mapping, total_rows, imported, duplicates, invalid,
                      suppressed, consent_affirmation, consent_hash, origin,
                      status, created_at, completed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11, ?12, ?13,
                         'completed', ?14, ?14)",
                params![
                    id_lote.to_string(),
                    empresa.to_string(),
                    recortar(&lote.nombre_del_archivo),
                    // El nombre con el que se guarda es NUESTRO, no el
                    // suministrado. El de fuera nunca toca el disco
                    // (THREAT_MODEL.md §4.1).
                    id_lote.to_string(),
                    lote.huella_del_archivo,
                    lote.mapeo_en_json,
                    lote.total_de_filas,
                    contactos.len(),
                    lote.choques,
                    lote.invalidas,
                    lote.texto_del_consentimiento,
                    arles_core::huella(&lote.texto_del_consentimiento),
                    lote.origen.como_texto(),
                    cuando,
                ],
            )?;

            for datos in contactos {
                let id = ContactId::nuevo();
                tx.execute(
                    "INSERT INTO contact (id, company_id, first_name, last_name,
                                          source, import_batch_id, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![
                        id.to_string(),
                        empresa.to_string(),
                        vacio_a_nulo(&datos.nombre),
                        vacio_a_nulo(&datos.apellido),
                        vacio_a_nulo(&datos.empresa),
                        id_lote.to_string(),
                        cuando,
                    ],
                )?;
                escribir_canales(&tx, empresa, id, &datos.canales, &cuando)?;
            }

            tx.commit()?;
            Ok(())
        })?;

        Ok(ResumenDeImportacion {
            lote: id_lote,
            importados: contactos.len(),
            choques: lote.choques,
            invalidas: lote.invalidas,
            total_de_filas: lote.total_de_filas,
        })
    }

    /// Quién es el dueño de cada dirección que ya está registrada.
    ///
    /// Es lo que alimenta la detección de choques de una importación: el
    /// análisis necesita saber **con quién** choca cada dirección, no sólo que
    /// choca. Decir «esta dirección ya existe» sin decir de quién obliga a
    /// buscarla a mano entre miles de contactos.
    ///
    /// Sólo mira las direcciones que se le pasan, en lotes: traerse las 500 000
    /// de la base para comparar contra un archivo de cien filas sería gastar
    /// memoria por nada, y con un archivo de 500 000 sería gastarla dos veces.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn duenos_de_direcciones(
        &self,
        empresa: CompanyId,
        direcciones: &[String],
    ) -> Result<std::collections::HashMap<String, String>, DbError> {
        let mut duenos = std::collections::HashMap::new();
        if direcciones.is_empty() {
            return Ok(duenos);
        }

        self.con_dominio(|conn| {
            // En lotes, con tantos interrogantes como direcciones lleve el
            // lote.
            //
            // SQLite tiene un tope de variables por sentencia. **Medido en este
            // build: 32 766** —ver `el_tope_de_variables_esta_por_encima_del_lote`—,
            // no las 999 que dice la documentación vieja y que este comentario
            // repetía antes de comprobarlo. Con 500 000 contactos (T-7) se pasa
            // igual, y no «a veces»: siempre, y justo en el archivo grande.
            //
            // 400 es holgado por debajo del tope de cualquier build razonable,
            // incluido uno compilado con el límite antiguo.
            const POR_LOTE: usize = 400;

            for lote in direcciones.chunks(POR_LOTE) {
                let huecos = std::iter::repeat_n("?", lote.len())
                    .collect::<Vec<_>>()
                    .join(",");
                let sql = format!(
                    "SELECT ch.value_normalized,
                            COALESCE(
                                TRIM(COALESCE(c.first_name, '') || ' ' || COALESCE(c.last_name, '')),
                                ''
                            )
                       FROM contact_channel ch
                       JOIN contact c ON c.id = ch.contact_id
                      WHERE ch.company_id = ?1
                        AND ch.deleted_at IS NULL
                        AND c.deleted_at IS NULL
                        AND ch.value_normalized IN ({huecos})"
                );

                let mut consulta = conn.prepare(&sql)?;
                let mut parametros: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(lote.len() + 1);
                let empresa_txt = empresa.to_string();
                parametros.push(&empresa_txt);
                for d in lote {
                    parametros.push(d);
                }

                let filas = consulta.query_map(parametros.as_slice(), |f| {
                    Ok((f.get::<_, String>(0)?, f.get::<_, String>(1)?))
                })?;
                for fila in filas {
                    let (direccion, nombre) = fila?;
                    duenos.insert(direccion, nombre);
                }
            }
            Ok(duenos)
        })
    }

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

    fn lote_de_prueba() -> DatosDelLote {
        DatosDelLote {
            nombre_del_archivo: "contactos-marzo.xlsx".into(),
            huella_del_archivo: "sha256:abc".into(),
            mapeo_en_json: r#"["nombre","correo"]"#.into(),
            texto_del_consentimiento:
                "Declaro que esta lista tiene origen lícito. [TEXTO PROVISIONAL]".into(),
            origen: arles_core::OrigenDeLaLista::FormularioPropio,
            total_de_filas: 3,
            choques: 0,
            invalidas: 0,
        }
    }

    #[test]
    fn una_importacion_deja_los_contactos_y_su_registro() {
        let (_d, db, empresa) = base();
        let gente = [
            contacto(&[(Canal::Correo, "uno@empresa.mx")]),
            contacto(&[(Canal::Correo, "dos@empresa.mx")]),
        ];

        let resumen = db
            .importar_contactos(empresa, &lote_de_prueba(), &gente)
            .expect("importa");

        assert_eq!(resumen.importados, 2);
        assert_eq!(
            db.listar_contactos(empresa, 0, 100).expect("lista").total,
            2
        );

        // Y cada contacto sabe de qué importación viene.
        let con_lote: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT count(*) FROM contact WHERE import_batch_id = ?1",
                    params![resumen.lote.to_string()],
                    |f| f.get(0),
                )
            })
            .expect("cuenta");
        assert_eq!(con_lote, 2);
    }

    /// **La garantía de esta entrega: o entra todo, o no entra nada.**
    ///
    /// Una importación a medias deja una lista de la que nadie sabe dónde se
    /// quedó: reintentarla duplica lo ya entrado, y no reintentarla deja fuera a
    /// gente que el usuario cree tener.
    ///
    /// Se provoca con una dirección que se ocupó **entre el análisis y la
    /// escritura**, que es el caso real: el análisis dijo que estaba libre y
    /// alguien la dio de alta mientras el usuario revisaba el informe.
    #[test]
    fn si_una_fila_falla_no_entra_ninguna() {
        let (_d, db, empresa) = base();
        // Alguien dio de alta esta dirección después del análisis.
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ocupada@empresa.mx")]))
            .expect("alta previa");

        let gente = [
            contacto(&[(Canal::Correo, "primera@empresa.mx")]),
            contacto(&[(Canal::Correo, "ocupada@empresa.mx")]), // choca
            contacto(&[(Canal::Correo, "tercera@empresa.mx")]),
        ];

        let fallo = db.importar_contactos(empresa, &lote_de_prueba(), &gente);
        assert!(matches!(fallo, Err(DbError::DireccionEnUso { .. })));

        // Sólo queda el contacto previo: ni la primera fila, que iba bien y se
        // escribió antes del choque, ni la tercera.
        let pagina = db.listar_contactos(empresa, 0, 100).expect("lista");
        assert_eq!(
            pagina.total, 1,
            "quedaron {} contactos: la importación entró a medias",
            pagina.total
        );

        // Y tampoco queda el registro del lote: un lote «completado» sin sus
        // contactos sería una prueba falsa de una importación que no ocurrió.
        let lotes: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row("SELECT count(*) FROM import_batch", [], |f| f.get(0))
            })
            .expect("cuenta");
        assert_eq!(lotes, 0, "quedó el registro de un lote que no se escribió");
    }

    /// El registro guarda **qué** se declaró, no sólo que se declaró algo: el
    /// origen concreto, el texto íntegro y su huella (ADR-0013 §1).
    #[test]
    fn el_registro_guarda_el_origen_el_texto_y_su_huella() {
        let (_d, db, empresa) = base();
        let lote = lote_de_prueba();
        let resumen = db
            .importar_contactos(
                empresa,
                &lote,
                &[contacto(&[(Canal::Correo, "uno@empresa.mx")])],
            )
            .expect("importa");

        let (origen, texto, hash): (String, String, String) = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT origin, consent_affirmation, consent_hash
                       FROM import_batch WHERE id = ?1",
                    params![resumen.lote.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
                )
            })
            .expect("lee el lote");

        assert_eq!(origen, "formulario_propio");
        assert_eq!(texto, lote.texto_del_consentimiento);
        assert_eq!(
            hash,
            arles_core::huella(&lote.texto_del_consentimiento),
            "la huella no corresponde al texto guardado"
        );
    }

    /// El nombre del archivo se guarda **recortado y como metadato**. Nunca
    /// toca el disco: el archivo se guarda con un identificador nuestro
    /// (THREAT_MODEL.md §4.1).
    #[test]
    fn el_nombre_del_archivo_no_se_usa_para_escribir() {
        let (_d, db, empresa) = base();
        let mut lote = lote_de_prueba();
        lote.nombre_del_archivo = format!("../../{}.xlsx", "a".repeat(400));

        let resumen = db
            .importar_contactos(
                empresa,
                &lote,
                &[contacto(&[(Canal::Correo, "uno@empresa.mx")])],
            )
            .expect("importa");

        let (original, guardado): (String, String) = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row(
                    "SELECT original_filename, stored_filename
                       FROM import_batch WHERE id = ?1",
                    params![resumen.lote.to_string()],
                    |f| Ok((f.get(0)?, f.get(1)?)),
                )
            })
            .expect("lee");

        assert!(original.chars().count() <= 255, "no se recortó");
        assert_eq!(
            guardado,
            resumen.lote.to_string(),
            "el nombre con el que se guarda tiene que ser nuestro, no el de fuera"
        );
        assert!(
            !guardado.contains(".."),
            "el nombre de fuera llegó al disco"
        );
    }

    /// Las cifras del resumen son las del análisis: si no cuadraran, el
    /// registro contaría una historia distinta de la que el usuario aprobó.
    #[test]
    fn las_cifras_del_resumen_cuadran_con_las_del_analisis() {
        let (_d, db, empresa) = base();
        let mut lote = lote_de_prueba();
        lote.total_de_filas = 10;
        lote.choques = 3;
        lote.invalidas = 2;

        let gente: Vec<_> = (0..5)
            .map(|i| contacto(&[(Canal::Correo, &format!("c{i}@empresa.mx"))]))
            .collect();
        let resumen = db
            .importar_contactos(empresa, &lote, &gente)
            .expect("importa");

        assert_eq!(resumen.importados, 5);
        assert_eq!(resumen.choques, 3);
        assert_eq!(resumen.invalidas, 2);
        assert_eq!(resumen.total_de_filas, 10);
        assert_eq!(
            resumen.importados + resumen.choques + resumen.invalidas,
            resumen.total_de_filas,
            "las cifras no suman: el informe y el registro dirían cosas distintas"
        );
    }

    /// Importar cero contactos deja el registro igual: es una importación que
    /// ocurrió y en la que no entró nadie, y eso también hay que poder contarlo.
    #[test]
    fn una_importacion_sin_nada_que_importar_deja_su_registro() {
        let (_d, db, empresa) = base();
        let mut lote = lote_de_prueba();
        lote.total_de_filas = 2;
        lote.invalidas = 2;

        let resumen = db.importar_contactos(empresa, &lote, &[]).expect("importa");
        assert_eq!(resumen.importados, 0);

        let lotes: i64 = db
            .ejecutar_en_pruebas(|conn| {
                conn.query_row("SELECT count(*) FROM import_batch", [], |f| f.get(0))
            })
            .expect("cuenta");
        assert_eq!(lotes, 1);
    }

    /// La consulta de dueños dice **con quién** choca cada dirección, no sólo
    /// que choca. Es lo que permite al informe de una importación decir «ya la
    /// tiene Ana Ruiz» en vez de «ya existe».
    #[test]
    fn los_duenos_dicen_de_quien_es_cada_direccion() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let duenos = db
            .duenos_de_direcciones(empresa, &["ana@empresa.mx".to_owned()])
            .expect("consulta");

        assert_eq!(duenos.len(), 1);
        assert_eq!(
            duenos.get("ana@empresa.mx").map(String::as_str),
            Some("Ana Ruiz")
        );
    }

    /// Una dirección que no está no aparece en el mapa. Si apareciera con el
    /// nombre vacío, el análisis la contaría como choque y la importación
    /// dejaría fuera filas perfectamente buenas.
    #[test]
    fn una_direccion_que_no_existe_no_sale_en_el_mapa() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let duenos = db
            .duenos_de_direcciones(
                empresa,
                &["ana@empresa.mx".to_owned(), "nadie@empresa.mx".to_owned()],
            )
            .expect("consulta");

        assert_eq!(duenos.len(), 1);
        assert!(!duenos.contains_key("nadie@empresa.mx"));
    }

    /// Un contacto dado de baja **libera** su dirección, así que no puede salir
    /// como dueño: si saliera, reimportar a alguien que se dio de baja sería
    /// imposible y el informe culparía a un contacto que ya no está.
    #[test]
    fn un_contacto_dado_de_baja_no_es_dueno_de_nada() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");
        db.borrar_contacto(empresa, id).expect("baja");

        let duenos = db
            .duenos_de_direcciones(empresa, &["ana@empresa.mx".to_owned()])
            .expect("consulta");
        assert!(
            duenos.is_empty(),
            "una dirección liberada salió como ocupada"
        );
    }

    /// El filtro por `contact.deleted_at` es **cinturón y tirantes**, y aquí se
    /// prueba contra el caso para el que existe.
    ///
    /// Hoy `borrar_contacto` marca el contacto **y** sus canales, así que
    /// filtrar por el canal ya bastaría — el cebo que quitaba este filtro no
    /// rompía nada—. Pero una base editada por fuera, o una migración a medias,
    /// puede dejar un contacto de baja con sus canales vivos. Sin el filtro, ese
    /// contacto fantasma saldría como dueño y bloquearía una importación
    /// perfectamente legítima.
    ///
    /// Se construye ese estado a mano, que es la única forma de llegar a él.
    #[test]
    fn un_contacto_de_baja_con_canales_vivos_tampoco_es_dueno() {
        let (_d, db, empresa) = base();
        let id = db
            .crear_contacto(
                empresa,
                &contacto(&[(Canal::Correo, "fantasma@empresa.mx")]),
            )
            .expect("crea");

        db.ejecutar_en_pruebas(|conn| {
            // Sólo el contacto, dejando sus canales vivos: el estado
            // inconsistente que el filtro existe para tolerar.
            conn.execute(
                "UPDATE contact SET deleted_at = ?2 WHERE id = ?1",
                params![id.to_string(), ahora()],
            )?;
            Ok(())
        })
        .expect("deja el contacto a medio borrar");

        let duenos = db
            .duenos_de_direcciones(empresa, &["fantasma@empresa.mx".to_owned()])
            .expect("consulta");

        assert!(
            duenos.is_empty(),
            "un contacto de baja con canales vivos salió como dueño y bloquearía la importación"
        );
    }

    /// Las direcciones de otra empresa no cuentan. Sin esto, una importación
    /// rechazaría filas por chocar con datos que no son suyos — y de paso
    /// filtraría que existen.
    #[test]
    fn los_duenos_no_cruzan_empresas() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "ana@empresa.mx")]))
            .expect("crea");

        let otra = db
            .ejecutar_en_pruebas(|conn| {
                let id = CompanyId::nuevo();
                conn.execute(
                    "INSERT INTO company (id, commercial_name, country, timezone,
                                          corporate_email, created_at, updated_at)
                     VALUES (?1, 'Otra', 'MX', 'UTC', 'otra@otra.mx', ?2, ?2)",
                    params![id.to_string(), ahora()],
                )?;
                Ok(id)
            })
            .expect("otra empresa");

        let duenos = db
            .duenos_de_direcciones(otra, &["ana@empresa.mx".to_owned()])
            .expect("consulta");
        assert!(duenos.is_empty());
    }

    /// **El lote.** SQLite tiene un tope de variables por sentencia —999 por
    /// defecto—, así que una consulta con una dirección por interrogante falla
    /// en cuanto el archivo pasa de ahí. Y falla en el archivo grande, que es
    /// justo donde importa.
    ///
    /// Se piden más del doble del tamaño de lote para que se recorra más de una
    /// vez, y una de ellas es real: si el troceado perdiera lotes, no saldría.
    #[test]
    fn se_consultan_en_lotes_para_no_pasar_del_tope_de_sqlite() {
        let (_d, db, empresa) = base();
        db.crear_contacto(empresa, &contacto(&[(Canal::Correo, "aguja@empresa.mx")]))
            .expect("crea");

        // Por encima del tope medido (32 766). Con 1 500 —lo que pedía la
        // primera versión de esta prueba— el cebo que quitaba el troceado NO
        // rompía nada, porque 1 500 variables caben de sobra. No medía nada.
        let mut direcciones: Vec<String> = (0..40_000)
            .map(|i| format!("pajar{i}@empresa.mx"))
            .collect();
        // La real, al final del todo: si sólo se consultara el primer lote,
        // esto no la encontraría.
        direcciones.push("aguja@empresa.mx".to_owned());

        let duenos = db
            .duenos_de_direcciones(empresa, &direcciones)
            .expect("la consulta no debería pasarse del tope de variables");

        assert_eq!(duenos.len(), 1);
        assert!(duenos.contains_key("aguja@empresa.mx"));
    }

    /// Un contacto sin nombre sale con cadena vacía, no con un espacio suelto
    /// del `nombre || ' ' || apellido`. La pantalla decide qué poner en su
    /// lugar; lo que no puede es recibir « » y creer que hay nombre.
    #[test]
    fn un_contacto_sin_nombre_sale_con_cadena_vacia() {
        let (_d, db, empresa) = base();
        let datos = DatosDeContacto::validar(
            &BorradorDeContacto {
                nombre: String::new(),
                apellido: String::new(),
                empresa: String::new(),
                canales: vec![BorradorDeCanal {
                    canal: Canal::Correo,
                    valor: "anonimo@empresa.mx".into(),
                    principal: true,
                }],
            },
            "MX",
        )
        .expect("válido");
        db.crear_contacto(empresa, &datos).expect("crea");

        let duenos = db
            .duenos_de_direcciones(empresa, &["anonimo@empresa.mx".to_owned()])
            .expect("consulta");
        assert_eq!(
            duenos.get("anonimo@empresa.mx").map(String::as_str),
            Some("")
        );
    }

    /// Dónde está de verdad el tope de variables de este SQLite.
    ///
    /// Se mide preparando sentencias cada vez más anchas hasta que una falla.
    /// No es ceremonia: el comentario del troceado afirmaba «999 por defecto»,
    /// y el cebo que quitaba el troceado **no rompió nada**, así que o la
    /// afirmación era vieja o la prueba no llegaba al tope. Había que medirlo
    /// en vez de repetirlo.
    #[test]
    fn el_tope_de_variables_esta_por_encima_del_lote() {
        let (_d, db, _e) = base();
        let mut tope = 0;
        for n in [
            900_usize, 999, 1_000, 2_000, 10_000, 32_766, 32_767, 100_000,
        ] {
            let sql = format!(
                "SELECT 1 WHERE 1 IN ({})",
                std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",")
            );
            let cabe = db
                .ejecutar_en_pruebas(|conn| Ok(conn.prepare(&sql).is_ok()))
                .unwrap_or(false);
            if cabe {
                tope = n;
            } else {
                break;
            }
        }
        println!("el tope real de variables está entre {tope} y el siguiente escalón");

        assert!(
            tope >= 999,
            "este SQLite no admite ni 999 variables: el troceado tendría que ser más pequeño"
        );
    }

    /// Pedir nada no consulta nada.
    #[test]
    fn pedir_cero_direcciones_no_consulta() {
        let (_d, db, empresa) = base();
        assert!(
            db.duenos_de_direcciones(empresa, &[])
                .expect("consulta")
                .is_empty()
        );
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
