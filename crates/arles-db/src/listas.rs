//! A quién va dirigida una campaña · D-15, 23/10/2026
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! LO QUE PIDIÓ DIRECCIÓN, Y POR QUÉ NO ES UNA COMODIDAD
//!
//!   «Cuando creen una campaña por ejemplo "Videovigilancia" y carguen todos
//!   los contactos donde podrían tener interés… pero si se crea otra campaña de
//!   "Desarrollo de aplicaciones" puedan cargar su tabla de contactos… y no
//!   mezclar contactos donde no sean de interés del público y así arriesgar el
//!   dominio de que le den spam.»
//!
//! Hasta hoy **toda campaña iba a todo el padrón**. Mandar una campaña de
//! videovigilancia a quien pidió una aplicación no es ruido: es la vía más
//! rápida a que marquen el correo como no deseado, y la tasa de quejas es la
//! cifra que más pesa en la reputación de un dominio.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! LAS TABLAS YA ESTABAN, Y ESO NO ES CASUALIDAD NI SUERTE
//!
//! `contact_list` y `contact_list_member` existen desde la V1 con la forma
//! exacta que hacía falta, y llevaban desde entonces en la lista de
//! `PENDIENTES_CONOCIDOS` del validador: tablas sin nadie que las escriba, con
//! su motivo escrito —«§40, declarado fuera del alcance de la Fase 6»—.
//!
//! Lo que este módulo hace es enchufarlas. La migración V14 sólo añade la
//! columna que las ata a la campaña.

use rusqlite::{Connection, OptionalExtension, params};

use arles_core::ids::{CompanyId, ContactListId};

use crate::db::Db;
use crate::empresa::ahora;
use crate::error::DbError;

/// Una lista, con lo que la pantalla necesita para elegirla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListaGuardada {
    pub id: ContactListId,
    pub nombre: String,
    /// **Cuánta gente tiene dentro.** Es la cifra que decide si esa lista es la
    /// que se quería (§94): un nombre sin su número deja elegir «Videovigilancia»
    /// creyendo que son trescientos cuando se quedó en doce.
    pub cuantos: u32,
    /// Cuántas campañas la usan. Es lo que impide borrarla, y lo que hay que
    /// poder decir antes de intentarlo.
    pub campanas: u32,
    pub creada: String,
}

impl Db {
    /// Crea una lista vacía, o devuelve la que ya se llamaba así.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ DEVUELVE LA EXISTENTE EN VEZ DE FALLAR
    ///
    /// Porque el camino real es importar dos archivos a la misma lista: la
    /// tabla de videovigilancia de enero y la de marzo. Fallar con «ya existe»
    /// obligaría a inventar «Videovigilancia 2», y dos listas con el mismo
    /// público es exactamente lo que esto viene a evitar.
    ///
    /// El índice único de la V14 —`(company_id, name)`— es el que lo hace
    /// seguro: sin él, dos llamadas a la vez crearían dos listas iguales y el
    /// desplegable enseñaría dos opciones idénticas.
    ///
    /// # Errores
    ///
    /// - [`DbError::DatoInvalido`] si el nombre viene vacío. Una lista sin
    ///   nombre no se puede elegir en un desplegable.
    /// - [`DbError::Sqlite`] si la escritura falla.
    pub fn crear_lista(&self, empresa: CompanyId, nombre: &str) -> Result<ContactListId, DbError> {
        let nombre = nombre.trim();
        if nombre.is_empty() {
            return Err(DbError::DatoInvalido {
                motivo: "contact_list.name",
            });
        }
        let cuando = ahora();
        self.con_dominio(|conn| crear_lista_en(conn, empresa, nombre, &cuando))
    }

    /// Las listas de la empresa, con su gente y sus campañas.
    ///
    /// Ordenadas por nombre y no por fecha: en un desplegable se busca con la
    /// vista, y una lista alfabética se recorre; una por fecha hay que leerla
    /// entera.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la consulta falla.
    pub fn listar_listas(&self, empresa: CompanyId) -> Result<Vec<ListaGuardada>, DbError> {
        self.con(|conn| {
            let mut s = conn.prepare(
                "SELECT l.id, l.name, l.created_at,
                        (SELECT count(*) FROM contact_list_member m
                          JOIN contact c ON c.id = m.contact_id
                         WHERE m.list_id = l.id AND c.deleted_at IS NULL),
                        (SELECT count(*) FROM campaign k WHERE k.contact_list_id = l.id)
                   FROM contact_list l
                  WHERE l.company_id = ?1
                  ORDER BY l.name COLLATE NOCASE",
            )?;
            let filas = s.query_map(params![empresa.to_string()], |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, i64>(3)?,
                    f.get::<_, i64>(4)?,
                ))
            })?;

            let mut salida = Vec::new();
            for fila in filas {
                let (id, nombre, creada, cuantos, campanas) = fila?;
                let Ok(id) = id.parse::<ContactListId>() else {
                    // Una fila con un id ilegible se salta en vez de reventar
                    // la pantalla entera: las demás listas siguen sirviendo.
                    continue;
                };
                salida.push(ListaGuardada {
                    id,
                    nombre,
                    cuantos: u32::try_from(cuantos).unwrap_or(u32::MAX),
                    campanas: u32::try_from(campanas).unwrap_or(u32::MAX),
                    creada,
                });
            }
            Ok(salida)
        })
    }
}

/// Crea la lista con la conexión ya abierta.
///
/// Existe aparte porque **el cerrojo de la conexión no es reentrante**: la
/// importación necesita crear la lista dentro de su propia transacción, y
/// llamar a `Db::crear_lista` desde ahí colgaría el programa sin error ni
/// traza. Es la misma razón por la que existe `cuentas::salud_en`.
pub(crate) fn crear_lista_en(
    conn: &Connection,
    empresa: CompanyId,
    nombre: &str,
    cuando: &str,
) -> Result<ContactListId, DbError> {
    let ya: Option<String> = conn
        .query_row(
            "SELECT id FROM contact_list WHERE company_id = ?1 AND name = ?2",
            params![empresa.to_string(), nombre],
            |f| f.get(0),
        )
        .optional()?;
    if let Some(Ok(id)) = ya.map(|t| t.parse::<ContactListId>()) {
        return Ok(id);
    }

    let id = ContactListId::nuevo();
    conn.execute(
        "INSERT INTO contact_list (id, company_id, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id.to_string(), empresa.to_string(), nombre, cuando],
    )?;
    Ok(id)
}

/// Mete en la lista a **todo el que tenga una de estas direcciones**.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ NO BASTA CON LOS QUE SE ACABAN DE IMPORTAR
///
/// Porque los repetidos no se importan. Una importación de trescientas filas
/// donde cincuenta ya estaban en la base —de otra campaña, de otro archivo—
/// crea doscientos cincuenta contactos, y los cincuenta repetidos **no serían
/// de la lista**: la campaña saldría a 250 personas de las 300 que se
/// cargaron, sin un solo error.
///
/// Y son justo las cincuenta que más importan: alguien que ya está en la base
/// es alguien con quien ya hay relación.
///
/// Así que la pertenencia se resuelve **por dirección y no por quién se acaba
/// de crear**: el archivo dice a quién se quiere escribir, no de dónde salió
/// cada ficha. `INSERT OR IGNORE` porque meter dos veces al mismo no es un
/// error —la clave primaria es `(list_id, contact_id)`— y porque un contacto
/// con dos correos del archivo aparecería dos veces.
///
/// Va en lotes de 400 por el tope de variables por sentencia de SQLite, igual
/// que `duenos_de_direcciones`: **medido en este build en 32 766**, no las 999
/// de la documentación vieja, y aun así se pasa con un archivo grande.
///
/// # Errores
///
/// [`DbError::Sqlite`] si la escritura falla.
pub(crate) fn meter_en_la_lista(
    conn: &Connection,
    empresa: CompanyId,
    lista: ContactListId,
    direcciones: &[String],
    cuando: &str,
) -> Result<usize, DbError> {
    const POR_LOTE: usize = 400;
    let mut metidos = 0;

    for trozo in direcciones.chunks(POR_LOTE) {
        let huecos = std::iter::repeat_n("?", trozo.len())
            .collect::<Vec<_>>()
            .join(",");
        // El `format!` interpola **interrogantes**, nunca un dato: las
        // direcciones van como parámetros. Es lo único que SQLite no sabe
        // parametrizar.
        let sql = format!(
            "INSERT OR IGNORE INTO contact_list_member (list_id, contact_id, added_at)
             SELECT ?1, c.id, ?2
               FROM contact c
               JOIN contact_channel ch ON ch.contact_id = c.id
              WHERE c.company_id = ?3 AND c.deleted_at IS NULL
                AND ch.value_normalized IN ({huecos})"
        );
        let mut valores: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(lista.to_string()),
            Box::new(cuando.to_owned()),
            Box::new(empresa.to_string()),
        ];
        for d in trozo {
            valores.push(Box::new(d.clone()));
        }
        metidos += conn.execute(&sql, rusqlite::params_from_iter(valores.iter()))?;
    }

    Ok(metidos)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;
    use crate::conexion::ClaveMaestra;

    const EMPRESA: &str = "00000000-0000-4000-8000-000000000001";

    fn empresa() -> CompanyId {
        EMPRESA.parse().expect("id")
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
                         'hola@t.mx', '2026-10-23T10:00:00Z', '2026-10-23T10:00:00Z');"
            ))
        })
        .expect("siembra la empresa");
        (dir, db)
    }

    /// Siembra un contacto con un correo.
    fn contacto(db: &Db, id: &str, correo: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO contact (id, company_id, first_name, created_at, updated_at)
                 VALUES ('{id}', '{EMPRESA}', 'N', '2026-10-23T10:00:00Z', '2026-10-23T10:00:00Z');
                 INSERT INTO contact_channel (id, company_id, contact_id, channel,
                                              value_raw, value_normalized, status,
                                              created_at, updated_at)
                 VALUES ('ch-{id}', '{EMPRESA}', '{id}', 'email', '{correo}', '{correo}',
                         'active', '2026-10-23T10:00:00Z', '2026-10-23T10:00:00Z');"
            ))
        })
        .expect("siembra el contacto");
    }

    /// Crear la misma lista dos veces devuelve la misma, no dos.
    ///
    /// Es el camino real: dos archivos del mismo público, en enero y en marzo.
    /// Fallar obligaría a llamarla «Videovigilancia 2», y dos listas con la
    /// misma gente es justo lo que esto viene a evitar.
    #[test]
    fn crear_dos_veces_la_misma_lista_devuelve_la_misma() {
        let (_d, db) = base();
        let a = db.crear_lista(empresa(), "Videovigilancia").expect("crea");
        let b = db.crear_lista(empresa(), "Videovigilancia").expect("crea");
        assert_eq!(a, b);
        assert_eq!(db.listar_listas(empresa()).expect("lista").len(), 1);
    }

    /// Y el nombre se recorta, así que « Videovigilancia » es la misma.
    #[test]
    fn el_nombre_de_la_lista_se_recorta() {
        let (_d, db) = base();
        let a = db.crear_lista(empresa(), "Videovigilancia").expect("crea");
        let b = db
            .crear_lista(empresa(), "  Videovigilancia  ")
            .expect("crea");
        assert_eq!(a, b, "un espacio de más no es otra lista");
    }

    /// Una lista sin nombre no se puede elegir en un desplegable.
    #[test]
    fn una_lista_sin_nombre_no_se_crea() {
        let (_d, db) = base();
        let r = db.crear_lista(empresa(), "   ");
        assert!(matches!(r, Err(DbError::DatoInvalido { .. })), "{r:?}");
    }

    /// **Los que ya estaban en la base también entran en la lista.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL DEFECTO QUE ESTA PRUEBA EXISTE PARA QUE NO VUELVA
    ///
    /// La forma obvia de armar la lista es «los contactos que acabo de
    /// importar». Y los repetidos **no se importan**: una tabla de trescientas
    /// filas con cincuenta que ya estaban deja la lista en doscientos
    /// cincuenta, la campaña sale a 250, y no falla nada.
    ///
    /// Las cincuenta que faltan son las que más importan: ya hay relación con
    /// ellas. Por eso la pertenencia se resuelve **por dirección**.
    ///
    /// ── LO QUE ESTA PRUEBA **NO** PUEDE COMPROBAR, Y DÓNDE SE COMPRUEBA ──
    ///
    /// Su primer comentario decía estar verificada «metiendo sólo los recién
    /// creados». **Es imposible desde aquí**: `meter_en_la_lista` no sabe
    /// quién se acaba de crear — recibe direcciones y ya. Esa garantía vive
    /// un piso más arriba, y se comprueba en
    /// `contactos::tests::importar_a_una_lista_mete_tambien_a_los_que_ya_estaban`,
    /// verificada quitando la llamada a esta función de `importar_contactos`:
    /// sale `left: 0, right: 2`.
    ///
    /// Se deja escrito porque es el defecto que este proyecto lleva más veces
    /// cometiendo: **una afirmación de verificación es una afirmación como
    /// cualquier otra**, y ésta era falsa.
    ///
    /// Lo que sí comprueba esto es lo suyo: que la pertenencia se resuelve por
    /// dirección, contra contactos que ya existían antes de la llamada.
    #[test]
    fn los_repetidos_tambien_entran_en_la_lista() {
        let (_d, db) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");

        let lista = db.crear_lista(empresa(), "Videovigilancia").expect("crea");
        let metidos = db
            .ejecutar_en_pruebas(|conn| {
                meter_en_la_lista(
                    conn,
                    empresa(),
                    lista,
                    &["ana@x.mx".to_owned(), "beto@x.mx".to_owned()],
                    "2026-10-23T11:00:00Z",
                )
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{e:?}")))
            })
            .expect("mete");

        assert_eq!(metidos, 2, "los dos ya estaban y los dos entran");
        let listas = db.listar_listas(empresa()).expect("lista");
        assert_eq!(listas.first().expect("hay una lista").cuantos, 2);
    }

    /// Meter dos veces al mismo no duplica ni falla.
    ///
    /// Pasa de verdad: dos importaciones a la misma lista con gente repetida
    /// entre los dos archivos. Sin el `OR IGNORE`, la segunda reventaría contra
    /// la clave primaria y se llevaría por delante la importación entera.
    #[test]
    fn meter_dos_veces_al_mismo_no_duplica() {
        let (_d, db) = base();
        contacto(&db, "k1", "ana@x.mx");
        let lista = db.crear_lista(empresa(), "Videovigilancia").expect("crea");

        for _ in 0..2 {
            db.ejecutar_en_pruebas(|conn| {
                meter_en_la_lista(
                    conn,
                    empresa(),
                    lista,
                    &["ana@x.mx".to_owned()],
                    "2026-10-23T11:00:00Z",
                )
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{e:?}")))
            })
            .expect("mete");
        }

        assert_eq!(
            db.listar_listas(empresa())
                .expect("l")
                .first()
                .expect("hay una lista")
                .cuantos,
            1
        );
    }

    /// La lista dice cuántas campañas la usan, que es lo que impide borrarla.
    #[test]
    fn la_lista_dice_cuantas_campanas_la_usan() {
        let (_d, db) = base();
        let lista = db.crear_lista(empresa(), "Videovigilancia").expect("crea");
        assert_eq!(
            db.listar_listas(empresa())
                .expect("l")
                .first()
                .expect("hay una lista")
                .campanas,
            0
        );

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(&format!(
                "INSERT INTO campaign (id, company_id, name, status, contact_list_id,
                                       created_at, updated_at)
                 VALUES ('cmp-1', '{EMPRESA}', 'Vigilancia', 'draft', '{lista}',
                         '2026-10-23T10:00:00Z', '2026-10-23T10:00:00Z');"
            ))
        })
        .expect("siembra la campaña");

        assert_eq!(
            db.listar_listas(empresa())
                .expect("l")
                .first()
                .expect("hay una lista")
                .campanas,
            1
        );
    }

    /// **Y un contacto borrado no cuenta en su lista.**
    ///
    /// La pertenencia no se borra —`contact_list_member` cuelga de `contact`
    /// con `ON DELETE CASCADE`, pero ARLES archiva en vez de borrar (§12)—, así
    /// que sin el `deleted_at IS NULL` la lista diría trescientos con doscientos
    /// vivos. Una cifra que no cuadra con la audiencia que sale después es peor
    /// que no dar cifra (§94).
    #[test]
    fn un_contacto_borrado_no_cuenta_en_la_lista() {
        let (_d, db) = base();
        contacto(&db, "k1", "ana@x.mx");
        contacto(&db, "k2", "beto@x.mx");
        let lista = db.crear_lista(empresa(), "Videovigilancia").expect("crea");
        db.ejecutar_en_pruebas(|conn| {
            meter_en_la_lista(
                conn,
                empresa(),
                lista,
                &["ana@x.mx".to_owned(), "beto@x.mx".to_owned()],
                "2026-10-23T11:00:00Z",
            )
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{e:?}")))
        })
        .expect("mete");

        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "UPDATE contact SET deleted_at = '2026-10-23T12:00:00Z' WHERE id = 'k2'",
            )
        })
        .expect("borra");

        assert_eq!(
            db.listar_listas(empresa())
                .expect("l")
                .first()
                .expect("hay una lista")
                .cuantos,
            1
        );
    }

    /// Y las listas de otra empresa no se ven.
    #[test]
    fn las_listas_no_cruzan_de_una_empresa_a_otra() {
        let (_d, db) = base();
        db.crear_lista(empresa(), "Videovigilancia").expect("crea");

        let otra: CompanyId = "00000000-0000-4000-8000-000000000002".parse().expect("id");
        assert!(db.listar_listas(otra).expect("lista").is_empty());
    }
}
