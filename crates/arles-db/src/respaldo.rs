//! Leer la base para un respaldo, sin cargarla en memoria.
//!
//! Entrega 8.2b. Lo que falta después es escribir el archivo y cifrarlo (8.2c):
//! aquí sólo se **lee**, y se lee de forma que exportar medio millón de
//! contactos no necesite medio millón de contactos de RAM.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ NO HAY TANDAS, AUNQUE LA ENTREGA SE LLAME ASÍ
//!
//! El plan de la fase decía «se escribe por tandas, como lee `arles-import`».
//! Al escribirlo salió que **la tanda es la unidad equivocada para leer**, y
//! conviene dejar dicho por qué, porque la forma obvia es la mala:
//!
//! La tanda obvia es `LIMIT 500 OFFSET n`. SQLite no guarda un atajo a la fila
//! n: para servir `OFFSET 400000` recorre y descarta cuatrocientas mil filas.
//! Exportar medio millón en tandas de 500 son mil consultas que recorren, de
//! media, un cuarto de millón de filas cada una — unas **125 000 millones** de
//! filas visitadas para leer 500 000. Es cuadrático, y en una tabla de dos años
//! se nota en minutos.
//!
//! Y hay algo peor que la lentitud: entre dos tandas la tabla puede cambiar. Si
//! el motor inserta un intento mientras se exporta, las filas se corren y
//! **una fila se repite o se salta** sin que nada falle.
//!
//! Lo que sí es memoria acotada es un **cursor**: `SELECT … ORDER BY rowid`, y
//! se van consumiendo las filas de una en una. SQLite no materializa el
//! resultado, así que en memoria hay una fila; el `ORDER BY rowid` lo sirve el
//! índice interno de la tabla, sin ordenar nada.
//!
//! Las tandas sobreviven en su único sitio útil: **avisar del avance**. Una
//! barra que se mueve una vez por fila son 500 000 cruces de frontera; una que
//! se mueve cada [`FILAS_POR_AVISO`] son doscientos cincuenta.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! UNA SOLA TRANSACCIÓN DE LECTURA PARA TODO EL RESPALDO
//!
//! El motor escribe mientras el usuario exporta: es un hilo aparte (§51) que no
//! se detiene por esto. Sin una transacción, cada consulta vería un momento
//! distinto de la base, y entonces:
//!
//! - el recuento de la cabecera no cuadraría con las filas del cuerpo, y la
//!   8.3 enseñaría en su preflight una cifra que el archivo no cumple;
//! - se podría exportar una etapa cuya campaña se creó **después** de leer
//!   `campaign` — un hijo sin padre, que al restaurar no entra.
//!
//! Con la transacción abierta, todo el respaldo ve la base de un solo instante.
//! El coste es honesto y hay que decirlo: mientras se exporta, el motor no
//! puede escribir, porque el cerrojo de la conexión es uno solo (ADR-0002).
//! Exportar es una acción que alguien pulsa y espera mirando.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL ORDEN DE LAS TABLAS NO ES EL DEL MANIFIESTO, Y ES UN ARREGLO
//!
//! El manifiesto de la 8.0 ordena las tablas «como el esquema, agrupando lo que
//! se lee junto». Es un buen orden para leerlo una persona y **no se puede
//! restaurar**: `campaign_stage` va en el grupo de las campañas y apunta a
//! `email_account`, que está diez entradas más abajo; `message_attempt`, igual.
//! Insertar un hijo antes que su padre con `PRAGMA foreign_keys=ON` se rechaza.
//!
//! Así que el orden del cuerpo se **calcula**, leyendo las claves ajenas del
//! esquema con `PRAGMA foreign_key_list` y ordenando por dependencia. Leer del
//! esquema y no de una lista a mano es lo mismo que hace el manifiesto con las
//! tablas, y por el mismo motivo: una lista a mano envejece en silencio.

use std::collections::{BTreeMap, BTreeSet};

use arles_core::respaldo::MANIFIESTO;
use rusqlite::Connection;
use rusqlite::types::ValueRef;

use crate::db::Db;
use crate::error::DbError;

/// Cada cuántas filas leídas se avisa del avance.
///
/// Dos mil y no doscientas: con medio millón de filas son 250 avisos, que es
/// una barra que se mueve de forma visible sin convertir el avance en el coste
/// principal de exportar.
pub const FILAS_POR_AVISO: u64 = 2_000;

/// Qué se lleva este respaldo (D-8B).
///
/// **El esencial exige la fecha de corte dentro.** Un `Alcance::Esencial` a
/// secas se podría pedir sin ella, y entonces las dos tablas que se recortan
/// saldrían enteras: un respaldo «esencial» del tamaño del completo, sin que
/// nada fallara. Aquí no cabe pedirlo mal.
///
/// La fecha la calcula `arles-app`, que es quien tiene reloj: ni el núcleo ni
/// este crate lo tienen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueSeLleva<'a> {
    /// Todo lo que el manifiesto no marca `Nunca`, sin recortar nada.
    Todo,
    /// Lo esencial, con el historial desde esta fecha ISO-8601 inclusive.
    LoEsencialDesde(&'a str),
}

/// Cuántas filas trae cada tabla, en el orden del cuerpo.
pub type RecuentoPorTabla = Vec<(String, u64)>;

/// Qué columnas trae cada tabla, en el mismo orden.
pub type ColumnasPorTabla = Vec<(String, Vec<String>)>;

/// Una tabla del respaldo: cómo se llama, qué columnas trae y cuántas filas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablaDelRespaldo {
    pub tabla: String,
    /// En el orden en que se escriben los campos de cada fila.
    pub columnas: Vec<String>,
    pub filas: u64,
}

/// Lo que se sabe del respaldo **antes** de escribir una sola fila.
///
/// Es lo que va a la cabecera, y por eso lleva el orden ya resuelto: el orden
/// de este vector es el orden del cuerpo y el orden en que la 8.3 insertará.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventario {
    /// Hasta qué migración llegó esta base: `V10`.
    pub esquema: String,
    pub tablas: Vec<TablaDelRespaldo>,
}

impl Inventario {
    /// Cuántas filas trae el respaldo entero.
    #[must_use]
    pub fn total_de_filas(&self) -> u64 {
        self.tablas.iter().map(|t| t.filas).sum()
    }

    /// Los dos vectores que la cabecera del formato necesita, en este orden.
    ///
    /// Existe para que 8.2c **no pueda** armar la cabecera con un orden
    /// distinto al del cuerpo: los dos salen de aquí.
    #[must_use]
    pub fn para_la_cabecera(&self) -> (RecuentoPorTabla, ColumnasPorTabla) {
        (
            self.tablas
                .iter()
                .map(|t| (t.tabla.clone(), t.filas))
                .collect(),
            self.tablas
                .iter()
                .map(|t| (t.tabla.clone(), t.columnas.clone()))
                .collect(),
        )
    }
}

/// Dónde van las filas que se leen.
///
/// Es un rasgo y no un `Vec` devuelto a propósito: devolver las filas sería
/// tenerlas todas en memoria, que es justo lo que esta entrega evita. Quien lo
/// implementa en la 8.2c escribe al archivo y va cifrando por trozos.
pub trait Destino {
    /// Se llama una vez, antes de la primera fila, con todo lo que hace falta
    /// para escribir la cabecera.
    ///
    /// # Errores
    ///
    /// Lo que falle al escribir. Un error aquí **aborta el respaldo**: un
    /// archivo sin cabecera no es un respaldo.
    fn inventario(&mut self, inventario: &Inventario) -> Result<(), DbError>;

    /// Una fila, con sus valores en el orden de las columnas declaradas.
    ///
    /// # Errores
    ///
    /// Lo que falle al escribir: típicamente un disco lleno a mitad.
    fn fila(&mut self, tabla: &str, valores: &[Option<String>]) -> Result<(), DbError>;

    /// Cuántas filas van de cuántas. Se llama cada [`FILAS_POR_AVISO`] y una
    /// vez al terminar cada tabla.
    ///
    /// No puede fallar: avisar del avance no es motivo para abortar un respaldo.
    fn avance(&mut self, _leidas: u64, _total: u64) {}
}

impl Db {
    /// Lee la base entera para un respaldo y se la va dando al destino.
    ///
    /// Devuelve cuántas filas se leyeron, para que quien escriba pueda
    /// comprobar que coinciden con las que la cabecera declaró.
    ///
    /// # Errores
    ///
    /// - [`DbError::Sqlite`] si alguna consulta falla.
    /// - [`DbError::EsquemaNoSeDejaRespaldar`] si el esquema tiene algo que el
    ///   formato no sabe llevar —un dato binario— o un ciclo de claves ajenas.
    /// - Lo que devuelva el destino al escribir.
    pub fn exportar<D: Destino>(
        &self,
        que: QueSeLleva<'_>,
        destino: &mut D,
    ) -> Result<u64, DbError> {
        self.con_dominio(|conn| {
            // La transacción abarca el inventario Y el cuerpo. Ver la cabecera:
            // sin ella la cifra de la cabecera y las filas del archivo pueden
            // no ser del mismo instante.
            let tx = conn.unchecked_transaction()?;

            let inventario = inventario_en(&tx, que)?;
            destino.inventario(&inventario)?;

            let total = inventario.total_de_filas();
            let mut leidas = 0_u64;
            for t in &inventario.tablas {
                leidas += volcar_tabla(&tx, t, que, destino, leidas, total)?;
                destino.avance(leidas, total);
            }

            // Sólo se ha leído, así que da igual confirmar o deshacer. Se
            // deshace para dejar claro que esta transacción no escribe nada.
            tx.rollback()?;
            Ok(leidas)
        })
    }
}

impl Db {
    /// Deja constancia en `audit_log` de que se escribió un respaldo.
    ///
    /// **Se llama después de escribir el archivo, no antes.** Anotar un
    /// respaldo que no llegó a existir convertiría la bitácora en una lista de
    /// intenciones — es la misma regla que el informe de acceso (3.4).
    ///
    /// Lo que se anota es el alcance, si iba cifrado y cuántas filas llevaba.
    /// **No el nombre del archivo ni su carpeta**: una ruta en la bitácora
    /// contaría de qué máquina salió y a dónde fue, que es la regla 4 de la
    /// frontera aplicada al disco. Si alguien pregunta «¿dónde está mi
    /// respaldo?», la respuesta no puede salir de aquí, y es correcto.
    ///
    /// # Errores
    ///
    /// [`DbError::Sqlite`] si la escritura falla.
    pub fn registrar_respaldo(
        &self,
        empresa: &str,
        alcance: &str,
        cifrado: bool,
        filas: u64,
    ) -> Result<(), DbError> {
        let resumen = format!(
            "{alcance}/{}/{filas}",
            if cifrado { "cifrado" } else { "sin-cifrar" }
        );
        self.con_dominio(|conn| {
            crate::empresa::auditar(conn, empresa, "exportar_respaldo", "respaldo", &resumen)?;
            Ok(())
        })
    }
}

/// Qué trae el respaldo, con el orden del cuerpo ya resuelto.
///
/// # Errores
///
/// [`DbError::Sqlite`] o [`DbError::EsquemaNoSeDejaRespaldar`] si hay un ciclo.
pub(crate) fn inventario_en(conn: &Connection, que: QueSeLleva<'_>) -> Result<Inventario, DbError> {
    let incluidas: Vec<&'static str> = MANIFIESTO
        .iter()
        .filter(|e| match que {
            QueSeLleva::Todo => e.alcance.entra_en_el_completo(),
            QueSeLleva::LoEsencialDesde(_) => e.alcance.entra_en_el_esencial(),
        })
        .map(|e| e.tabla)
        .collect();

    let mut tablas = Vec::with_capacity(incluidas.len());
    for tabla in orden_de_insercion(conn, &incluidas)? {
        tablas.push(TablaDelRespaldo {
            tabla: tabla.to_owned(),
            columnas: columnas_de(conn, tabla)?,
            filas: cuantas_filas(conn, tabla, corte_de(tabla, que))?,
        });
    }

    Ok(Inventario {
        esquema: version_del_esquema(conn)?,
        tablas,
    })
}

/// La condición de recorte de una tabla, si la hay: `(columna, desde)`.
fn corte_de<'a>(tabla: &str, que: QueSeLleva<'a>) -> Option<(&'static str, &'a str)> {
    let QueSeLleva::LoEsencialDesde(desde) = que else {
        return None;
    };
    MANIFIESTO
        .iter()
        .find(|e| e.tabla == tabla)
        .and_then(|e| e.alcance.columna_de_corte())
        .map(|columna| (columna, desde))
}

/// Hasta qué migración llegó esta base, en la forma `V10`.
///
/// # Errores
///
/// [`DbError::Sqlite`] si la tabla de `refinery` no se puede leer. **Y no se
/// aproxima a «desconocido»**: un respaldo sin versión de esquema no se puede
/// restaurar con criterio, así que es mejor no escribirlo.
fn version_del_esquema(conn: &Connection) -> Result<String, DbError> {
    let n: Option<i32> = conn.query_row(
        "SELECT max(version) FROM refinery_schema_history",
        [],
        |f| f.get(0),
    )?;
    let n = n.ok_or(DbError::EsquemaNoSeDejaRespaldar {
        motivo: "la base no dice por qué migración va, así que un respaldo suyo no se podría restaurar con criterio",
    })?;
    Ok(format!("V{n}"))
}

/// Las columnas de una tabla, en el orden del esquema.
///
/// # Errores
///
/// [`DbError::Sqlite`], o [`DbError::EsquemaNoSeDejaRespaldar`] si la tabla no
/// tiene columnas — que sólo puede significar que no existe.
fn columnas_de(conn: &Connection, tabla: &str) -> Result<Vec<String>, DbError> {
    // El nombre se interpola porque SQLite no deja parametrizar un
    // identificador. No es una inyección: `tabla` sale de `MANIFIESTO`, que es
    // una constante de `arles-core`, y nunca de nada que escriba nadie. Lo
    // vigila `ninguna_tabla_se_queda_fuera_del_respaldo` en el validador, que
    // compara el manifiesto contra el esquema en los dos sentidos.
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{tabla}\")"))?;
    let columnas = stmt
        .query_map([], |f| f.get::<_, String>("name"))?
        .collect::<Result<Vec<_>, _>>()?;

    if columnas.is_empty() {
        return Err(DbError::EsquemaNoSeDejaRespaldar {
            motivo: "el manifiesto clasifica una tabla que esta base no tiene",
        });
    }
    Ok(columnas)
}

/// Cuántas filas se van a escribir de esta tabla.
fn cuantas_filas(
    conn: &Connection,
    tabla: &str,
    corte: Option<(&str, &str)>,
) -> Result<u64, DbError> {
    let n: i64 = match corte {
        None => conn.query_row(&format!("SELECT count(*) FROM \"{tabla}\""), [], |f| {
            f.get(0)
        })?,
        Some((columna, desde)) => conn.query_row(
            &format!("SELECT count(*) FROM \"{tabla}\" WHERE \"{columna}\" >= ?1"),
            [desde],
            |f| f.get(0),
        )?,
    };
    Ok(n.max(0).unsigned_abs())
}

/// Las tablas de las que depende ésta, sin contarse a sí misma.
fn padres_de(conn: &Connection, tabla: &str) -> Result<BTreeSet<String>, DbError> {
    let mut stmt = conn.prepare(&format!("PRAGMA foreign_key_list(\"{tabla}\")"))?;
    let padres = stmt
        .query_map([], |f| f.get::<_, String>("table"))?
        .collect::<Result<BTreeSet<_>, _>>()?;
    // Una tabla que se refiere a sí misma no impone orden **entre tablas**: lo
    // que ordena es la inserción de sus filas, que no es asunto de aquí. Hoy no
    // hay ninguna; dejarla dentro colgaría el orden sin decir por qué.
    Ok(padres.into_iter().filter(|p| p != tabla).collect())
}

/// Las tablas ordenadas para que ningún hijo vaya antes que su padre.
///
/// Es un orden topológico con el orden del manifiesto como desempate, para que
/// dos exportaciones de la misma base den **el mismo archivo**: un orden que
/// cambiara entre ejecuciones haría imposible comparar dos respaldos.
///
/// Un padre que no esté en el respaldo —porque el alcance lo deja fuera— no
/// impone nada aquí: ese caso es un defecto de clasificación, y lo caza
/// `ningun_hijo_viaja_sin_su_padre` leyendo el esquema de verdad.
///
/// # Errores
///
/// [`DbError::EsquemaNoSeDejaRespaldar`] si hay un ciclo. **No se rompe por
/// donde sea**: elegir un orden cualquiera produciría un respaldo que no se
/// puede restaurar, y eso hay que saberlo al exportar, no al necesitarlo.
pub(crate) fn orden_de_insercion<'t>(
    conn: &Connection,
    tablas: &[&'t str],
) -> Result<Vec<&'t str>, DbError> {
    let dentro: BTreeSet<&str> = tablas.iter().copied().collect();

    let mut pendientes: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for t in tablas {
        let padres = padres_de(conn, t)?
            .into_iter()
            .filter(|p| dentro.contains(p.as_str()))
            .collect();
        pendientes.insert(t, padres);
    }

    let mut orden: Vec<&'t str> = Vec::with_capacity(tablas.len());
    let mut colocadas: BTreeSet<&str> = BTreeSet::new();

    while orden.len() < tablas.len() {
        // Se recorre `tablas` —el orden del manifiesto— y no el mapa, que está
        // ordenado alfabéticamente: el desempate tiene que ser el que una
        // persona escribió, no el del abecedario.
        let siguiente = tablas.iter().find(|t| {
            !colocadas.contains(**t)
                && pendientes
                    .get(**t)
                    .is_some_and(|ps| ps.iter().all(|p| colocadas.contains(p.as_str())))
        });

        let Some(t) = siguiente else {
            let atascadas: Vec<&str> = tablas
                .iter()
                .copied()
                .filter(|t| !colocadas.contains(t))
                .collect();
            debug_assert!(!atascadas.is_empty(), "sin atascadas no habría ciclo");
            return Err(DbError::EsquemaNoSeDejaRespaldar {
                motivo: "hay tablas que dependen unas de otras en círculo, y no existe un orden en que insertarlas",
            });
        };

        colocadas.insert(t);
        orden.push(t);
    }

    Ok(orden)
}

/// Lee una tabla de punta a punta y le da cada fila al destino.
///
/// Devuelve cuántas filas leyó.
fn volcar_tabla<D: Destino>(
    conn: &Connection,
    t: &TablaDelRespaldo,
    que: QueSeLleva<'_>,
    destino: &mut D,
    ya_leidas: u64,
    total: u64,
) -> Result<u64, DbError> {
    let columnas = t
        .columnas
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<_>>()
        .join(", ");

    // `ORDER BY rowid` no ordena nada: es el orden físico de la tabla, servido
    // por su índice interno. Sirve para que dos exportaciones de una base que no
    // cambió den el mismo archivo, que es lo que permite compararlas.
    let (sql, corte) = match corte_de(&t.tabla, que) {
        None => (
            format!("SELECT {columnas} FROM \"{}\" ORDER BY rowid", t.tabla),
            None,
        ),
        Some((columna, desde)) => (
            format!(
                "SELECT {columnas} FROM \"{}\" WHERE \"{columna}\" >= ?1 ORDER BY rowid",
                t.tabla
            ),
            Some(desde),
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let mut filas = match corte {
        None => stmt.query([])?,
        Some(desde) => stmt.query([desde])?,
    };

    // El búfer se reutiliza fila a fila. Con medio millón de filas y veinte
    // columnas, una `Vec` nueva por fila son diez millones de reservas que no
    // hacen falta: los `String` se vacían y se vuelven a llenar.
    let mut valores: Vec<Option<String>> = vec![None; t.columnas.len()];
    let mut leidas = 0_u64;

    while let Some(fila) = filas.next()? {
        for (i, hueco) in valores.iter_mut().enumerate() {
            fijar(hueco, &valor_como_texto(&t.tabla, fila.get_ref(i)?)?);
        }
        destino.fila(&t.tabla, &valores)?;
        leidas += 1;
        if leidas.is_multiple_of(FILAS_POR_AVISO) {
            destino.avance(ya_leidas + leidas, total);
        }
    }

    Ok(leidas)
}

/// Mete un valor en el hueco del búfer **sin tirar la reserva** si cabe.
fn fijar(hueco: &mut Option<String>, nuevo: &Option<String>) {
    match (hueco.as_mut(), nuevo) {
        (Some(s), Some(n)) => {
            s.clear();
            s.push_str(n);
        }
        (_, otro) => *hueco = otro.clone(),
    }
}

/// Un valor de SQLite como el texto que va al archivo.
///
/// # Errores
///
/// [`DbError::EsquemaNoSeDejaRespaldar`] si es un dato binario. **No se guarda
/// en hexadecimal por si acaso**: al restaurar no habría forma de distinguir un
/// blob de un texto que casualmente parece hexadecimal, y el dato volvería
/// cambiado sin que nada fallara. Hoy el esquema no tiene ni una columna
/// binaria, y `ninguna_columna_del_respaldo_guarda_binario` lo comprueba contra
/// una base migrada de verdad: si alguien añade una, se entera ahí.
fn valor_como_texto(tabla: &str, v: ValueRef<'_>) -> Result<Option<String>, DbError> {
    Ok(match v {
        ValueRef::Null => None,
        ValueRef::Integer(n) => Some(n.to_string()),
        // `Display` de `f64` da la forma más corta que vuelve al mismo número,
        // así que la ida y la vuelta no pierden nada.
        ValueRef::Real(x) => Some(x.to_string()),
        ValueRef::Text(bytes) => Some(
            std::str::from_utf8(bytes)
                .map_err(|_| DbError::DatoInvalido {
                    motivo: "una fila guarda texto que no es UTF-8 válido",
                })?
                .to_owned(),
        ),
        ValueRef::Blob(_) => {
            let _ = tabla;
            return Err(DbError::EsquemaNoSeDejaRespaldar {
                motivo: "una tabla guarda datos binarios, y el formato del respaldo no sabe llevarlos sin cambiarlos",
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::respaldo::Alcance as AlcanceDeTabla;

    use crate::conexion::ClaveMaestra;

    /// Un destino de pruebas que se lo queda todo en memoria.
    #[derive(Debug, Default)]
    struct EnMemoria {
        inventario: Option<Inventario>,
        filas: Vec<(String, Vec<Option<String>>)>,
        avances: Vec<(u64, u64)>,
    }

    impl Destino for EnMemoria {
        fn inventario(&mut self, inv: &Inventario) -> Result<(), DbError> {
            self.inventario = Some(inv.clone());
            Ok(())
        }
        fn fila(&mut self, tabla: &str, valores: &[Option<String>]) -> Result<(), DbError> {
            self.filas.push((tabla.to_owned(), valores.to_vec()));
            Ok(())
        }
        fn avance(&mut self, leidas: u64, total: u64) {
            self.avances.push((leidas, total));
        }
    }

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        (dir, db)
    }

    /// Siembra una empresa, un lote, dos contactos y dos intentos: uno viejo y
    /// uno reciente, para ver el recorte.
    fn sembrar(db: &Db) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
                   VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO import_batch
                   (id, company_id, original_filename, stored_filename, file_hash, column_mapping,
                    total_rows, consent_affirmation, created_at)
                   VALUES ('L1', 'E1', 'lista.csv', 'L1.csv', 'abc', '{}', 2, 'sí', '2026-01-02T00:00:00Z');
                 INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, created_at, updated_at)
                   VALUES ('C1', 'E1', 'L1', 'Ana', 'Ruiz', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');
                 INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, created_at, updated_at)
                   VALUES ('C2', 'E1', 'L1', 'Beto', NULL, '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                   VALUES ('K1', 'E1', 'Primera', 'draft', '2026-01-03T00:00:00Z', '2026-01-03T00:00:00Z');
                 INSERT INTO message_attempt
                   (id, campaign_id, channel, contact_id, contact_address, idempotency_key, state, created_at, updated_at)
                   VALUES ('A1', 'K1', 'email', 'C1', 'ana@ejemplo.mx', 'k1', 'sent', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z');
                 INSERT INTO message_attempt
                   (id, campaign_id, channel, contact_id, contact_address, idempotency_key, state, created_at, updated_at)
                   VALUES ('A2', 'K1', 'email', 'C2', 'beto@ejemplo.mx', 'k2', 'sent', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');",
            )
        })
        .expect("siembra");
    }

    /// **Ningún hijo viaja sin su padre, en los dos alcances.**
    ///
    /// Es la comprobación que encontró el defecto de la 8.0: `import_batch`
    /// estaba como `SoloCompleto` y `contact` —que entra siempre— tiene una
    /// clave ajena a ella. Un respaldo esencial traía los contactos y no los
    /// lotes, así que **no se podía restaurar**: con `foreign_keys=ON` cada
    /// `INSERT` de contacto se rechaza.
    ///
    /// Lee las claves ajenas del esquema, no una lista: el día que una migración
    /// añada una tabla con un padre mal clasificado, falla aquí.
    ///
    /// Verificada el 05/10/2026 devolviendo `import_batch` a `SoloCompleto`:
    /// falla nombrando `contact -> import_batch`.
    #[test]
    fn ningun_hijo_viaja_sin_su_padre() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|conn| {
            for (etiqueta, que) in [
                ("esencial", QueSeLleva::LoEsencialDesde("2020-01-01T00:00:00Z")),
                ("completo", QueSeLleva::Todo),
            ] {
                let incluidas: Vec<&str> = MANIFIESTO
                    .iter()
                    .filter(|e| match que {
                        QueSeLleva::Todo => e.alcance.entra_en_el_completo(),
                        QueSeLleva::LoEsencialDesde(_) => e.alcance.entra_en_el_esencial(),
                    })
                    .map(|e| e.tabla)
                    .collect();

                for hija in &incluidas {
                    for padre in padres_de(conn, hija).expect("padres") {
                        assert!(
                            incluidas.contains(&padre.as_str()),
                            "en el respaldo {etiqueta}, {hija} -> {padre}: el hijo viaja y el padre no, \
                             así que restaurarlo fallaría en el primer INSERT"
                        );

                        // Y el otro lado del mismo agujero: un padre que SÍ va
                        // pero recortado por fecha deja huérfanas las filas de
                        // la hija que apunten a lo recortado. Hoy no pasa.
                        let recortado = MANIFIESTO
                            .iter()
                            .find(|e| e.tabla == padre)
                            .is_some_and(|e| e.alcance.columna_de_corte().is_some());
                        let hija_entera = MANIFIESTO
                            .iter()
                            .find(|e| e.tabla == *hija)
                            .is_some_and(|e| e.alcance == AlcanceDeTabla::SiempreEntera);
                        assert!(
                            !(recortado && hija_entera && matches!(que, QueSeLleva::LoEsencialDesde(_))),
                            "en el respaldo {etiqueta}, {hija} va entera y su padre {padre} se recorta: \
                             las filas que apunten a lo recortado quedarían huérfanas"
                        );
                    }
                }
            }
            Ok(())
        })
        .expect("recorre el esquema");
    }

    /// **El orden del manifiesto no sirve para restaurar, y el calculado sí.**
    ///
    /// Las dos mitades a la vez, porque una sin la otra no dice nada: que el
    /// orden calculado valga sólo importa si el obvio no valía.
    ///
    /// Verificada el 05/10/2026 de la única forma que vale: se le pasó al
    /// verificador el orden tal cual lo declara el manifiesto y señaló tres
    /// tablas antes que sus padres —`contact`, `campaign_stage` y
    /// `message_attempt`—.
    #[test]
    fn el_orden_calculado_pone_a_cada_padre_antes_que_sus_hijos() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|conn| {
            let declarado: Vec<&str> = MANIFIESTO
                .iter()
                .filter(|e| e.alcance.entra_en_el_completo())
                .map(|e| e.tabla)
                .collect();

            // Devuelve las parejas (hija, padre) que están al revés.
            let al_reves = |orden: &[&str]| -> Vec<String> {
                let mut malas = Vec::new();
                for (i, hija) in orden.iter().enumerate() {
                    for padre in padres_de(conn, hija).expect("padres") {
                        if let Some(j) = orden.iter().position(|t| *t == padre.as_str())
                            && j > i
                        {
                            malas.push(format!("{hija} antes de {padre}"));
                        }
                    }
                }
                malas
            };

            assert!(
                !al_reves(&declarado).is_empty(),
                "si el orden del manifiesto ya valiera, calcularlo no haría falta: \
                 revisa si alguien lo reordenó a mano, porque entonces esta prueba \
                 dejó de proteger nada"
            );

            let calculado = orden_de_insercion(conn, &declarado).expect("orden");
            assert_eq!(
                calculado.len(),
                declarado.len(),
                "el orden calculado pierde o repite tablas"
            );
            assert!(
                al_reves(&calculado).is_empty(),
                "el orden calculado deja hijos antes de sus padres: {:?}",
                al_reves(&calculado)
            );
            Ok(())
        })
        .expect("recorre el esquema");
    }

    /// El orden calculado es **el mismo** dos veces.
    ///
    /// Si no lo fuera, dos respaldos de la misma base saldrían distintos y no se
    /// podrían comparar — ni comprobar que exportar no cambió nada.
    #[test]
    fn el_orden_no_cambia_entre_dos_llamadas() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|conn| {
            let tablas: Vec<&str> = MANIFIESTO.iter().map(|e| e.tabla).collect();
            let a = orden_de_insercion(conn, &tablas).expect("orden");
            let b = orden_de_insercion(conn, &tablas).expect("orden");
            assert_eq!(a, b);
            Ok(())
        })
        .expect("ok");
    }

    /// **Lo que la cabecera declara es lo que el cuerpo trae.**
    ///
    /// La cifra del recuento es lo que la 8.3 enseñará en su preflight antes de
    /// que alguien confirme una restauración. Si no cuadrara con las filas del
    /// archivo, el preflight mentiría — y mentiría con cifras, que es peor.
    ///
    /// Verificada el 05/10/2026 contando `message_attempt` sin la condición del
    /// recorte: declara 2 y el cuerpo trae 1, y la prueba falla nombrando la
    /// tabla.
    #[test]
    fn el_recuento_de_la_cabecera_cuadra_con_las_filas_escritas() {
        let (_d, db) = base();
        sembrar(&db);

        for que in [
            QueSeLleva::Todo,
            QueSeLleva::LoEsencialDesde("2026-01-01T00:00:00Z"),
        ] {
            let mut d = EnMemoria::default();
            let leidas = db.exportar(que, &mut d).expect("exporta");
            let inv = d.inventario.clone().expect("inventario");

            // Primero tabla por tabla, y después el total: al revés, un
            // descuadre se anuncia como «6 no es 7» y hay que ir a buscar de
            // qué tabla sobra o falta.
            for t in &inv.tablas {
                let escritas = d.filas.iter().filter(|(n, _)| *n == t.tabla).count() as u64;
                assert_eq!(
                    escritas, t.filas,
                    "{que:?}: la cabecera declara {} filas de {} y el cuerpo trae {escritas}",
                    t.filas, t.tabla
                );
            }
            assert_eq!(leidas, inv.total_de_filas(), "{que:?}");
        }
    }

    /// **El recorte del esencial recorta de verdad.**
    ///
    /// Es D-8B hecha prueba: si el esencial se llevara el historial entero, la
    /// elección de alcance sería decorativa y el respaldo del día a día volvería
    /// a tardar lo que D-8B quería evitar.
    ///
    /// Verificada el 05/10/2026 quitando el `WHERE` del recorte: el esencial se
    /// lleva los dos intentos y la prueba falla.
    #[test]
    fn el_esencial_deja_fuera_el_historial_viejo() {
        let (_d, db) = base();
        sembrar(&db);

        let intentos = |que| {
            let mut d = EnMemoria::default();
            db.exportar(que, &mut d).expect("exporta");
            d.filas
                .iter()
                .filter(|(t, _)| t == "message_attempt")
                .count()
        };

        assert_eq!(intentos(QueSeLleva::Todo), 2, "el completo no recorta nada");
        assert_eq!(
            intentos(QueSeLleva::LoEsencialDesde("2026-01-01T00:00:00Z")),
            1,
            "el esencial tenía que dejar fuera el intento de 2024"
        );

        // Y las bajas no se recortan nunca, ni cuando el corte las dejaría
        // fuera: es la regla que no se negocia de la fase.
        let mut d = EnMemoria::default();
        db.exportar(QueSeLleva::LoEsencialDesde("2026-01-01T00:00:00Z"), &mut d)
            .expect("exporta");
        let inv = d.inventario.expect("inventario");
        assert!(
            inv.tablas.iter().any(|t| t.tabla == "suppression_entry"),
            "las bajas tienen que estar en el esencial"
        );
    }

    /// **El vacío y el nulo siguen siendo dos cosas al salir de la base.**
    ///
    /// El formato los distingue (8.1), pero sólo sirve si quien lee la base los
    /// trae distintos. `Beto` no tiene apellido —NULL— y eso tiene que llegar
    /// como `None`, no como `Some("")`.
    #[test]
    fn un_nulo_de_la_base_llega_como_nulo() {
        let (_d, db) = base();
        sembrar(&db);

        let mut d = EnMemoria::default();
        db.exportar(QueSeLleva::Todo, &mut d).expect("exporta");
        let inv = d.inventario.expect("inventario");
        let columnas = inv
            .tablas
            .iter()
            .find(|t| t.tabla == "contact")
            .map(|t| t.columnas.clone())
            .expect("contact va en el respaldo");
        let i = columnas
            .iter()
            .position(|c| c == "last_name")
            .expect("contact tiene last_name");

        let apellidos: Vec<Option<String>> = d
            .filas
            .iter()
            .filter(|(t, _)| t == "contact")
            .map(|(_, v)| v.get(i).cloned().flatten())
            .collect();
        assert!(
            apellidos.contains(&None),
            "el apellido nulo de Beto llegó como {apellidos:?}"
        );
        assert!(apellidos.contains(&Some("Ruiz".to_owned())));
    }

    /// Los números no se pierden al pasar por texto.
    #[test]
    fn un_entero_viaja_como_su_texto() {
        let (_d, db) = base();
        sembrar(&db);

        let mut d = EnMemoria::default();
        db.exportar(QueSeLleva::Todo, &mut d).expect("exporta");
        let inv = d.inventario.expect("inventario");
        let columnas = inv
            .tablas
            .iter()
            .find(|t| t.tabla == "import_batch")
            .map(|t| t.columnas.clone())
            .expect("import_batch va en el respaldo");
        let i = columnas
            .iter()
            .position(|c| c == "total_rows")
            .expect("import_batch tiene total_rows");
        let fila = d
            .filas
            .iter()
            .find(|(t, _)| t == "import_batch")
            .map(|(_, v)| v.clone())
            .expect("hay un lote");
        assert_eq!(fila.get(i).cloned().flatten(), Some("2".to_owned()));
    }

    /// **Ninguna columna del respaldo guarda datos binarios.**
    ///
    /// Si alguna los guardara, el formato —que es texto— no podría llevarla sin
    /// cambiar el dato, y `valor_como_texto` aborta. La prueba lo comprueba
    /// contra el esquema migrado de verdad para que el día que alguien añada
    /// una columna `BLOB` se entere aquí y no exportando.
    #[test]
    fn ninguna_columna_del_respaldo_guarda_binario() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|conn| {
            let mut malas = Vec::new();
            for e in MANIFIESTO
                .iter()
                .filter(|e| e.alcance.entra_en_el_completo())
            {
                let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", e.tabla))?;
                let tipos = stmt
                    .query_map([], |f| {
                        Ok((f.get::<_, String>("name")?, f.get::<_, String>("type")?))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                for (columna, tipo) in tipos {
                    if tipo.to_uppercase().contains("BLOB") {
                        malas.push(format!("{}.{columna}", e.tabla));
                    }
                }
            }
            assert!(
                malas.is_empty(),
                "el formato del respaldo es texto y estas columnas son binarias: {malas:?}"
            );
            Ok(())
        })
        .expect("recorre el esquema");
    }

    /// **Toda columna de corte existe de verdad en su tabla.**
    ///
    /// El manifiesto declara por qué columna se recorta cada tabla, y es un
    /// nombre escrito a mano. Si no existiera, el `WHERE` fallaría — al
    /// exportar, delante del usuario, y no aquí.
    ///
    /// Verificada el 05/10/2026 poniendo `created_at` en `whatsapp_reply`, que
    /// no la tiene: falla nombrando `whatsapp_reply.created_at`.
    #[test]
    fn toda_columna_de_corte_existe_de_verdad() {
        let (_d, db) = base();
        db.ejecutar_en_pruebas(|conn| {
            for e in MANIFIESTO {
                let Some(columna) = e.alcance.columna_de_corte() else {
                    continue;
                };
                let existe = columnas_de(conn, e.tabla)
                    .expect("columnas")
                    .iter()
                    .any(|c| c == columna);
                assert!(
                    existe,
                    "el manifiesto recorta {} por {columna}, y esa columna no existe",
                    e.tabla
                );
            }
            Ok(())
        })
        .expect("recorre el esquema");
    }

    /// **Lo que el manifiesto marca `Nunca` no sale, ni de rebote.**
    ///
    /// Las tres describen ESTE equipo: el tema, el cupo gastado y la parada de
    /// emergencia. Restaurar un `engine_control` de otra máquina arrancaría
    /// envíos que nadie pidió aquí.
    #[test]
    fn el_estado_de_este_equipo_no_sale_en_ningun_alcance() {
        let (_d, db) = base();
        sembrar(&db);

        for que in [
            QueSeLleva::Todo,
            QueSeLleva::LoEsencialDesde("2026-01-01T00:00:00Z"),
        ] {
            let mut d = EnMemoria::default();
            db.exportar(que, &mut d).expect("exporta");
            let inv = d.inventario.expect("inventario");
            for prohibida in ["ui_preference", "rate_budget", "engine_control"] {
                assert!(
                    !inv.tablas.iter().any(|t| t.tabla == prohibida),
                    "{que:?} se lleva {prohibida}"
                );
                assert!(!d.filas.iter().any(|(t, _)| t == prohibida));
            }
        }
    }

    /// El avance avisa, y la última cifra es el total.
    ///
    /// Una barra que no llega al final deja a alguien mirando un 98 % que ya
    /// terminó (§94).
    #[test]
    fn el_avance_termina_en_el_total() {
        let (_d, db) = base();
        sembrar(&db);

        let mut d = EnMemoria::default();
        let leidas = db.exportar(QueSeLleva::Todo, &mut d).expect("exporta");
        let ultimo = d.avances.last().copied().expect("avisó al menos una vez");
        assert_eq!(ultimo, (leidas, leidas), "el último aviso no es el total");
    }

    /// **Un destino que falla aborta el respaldo, no lo deja a medias.**
    ///
    /// Es el caso del disco que se llena a mitad de escribir. Lo que no puede
    /// pasar es que el error se tape y quede un archivo corto que parece bueno.
    #[test]
    fn si_el_destino_falla_el_respaldo_falla() {
        #[derive(Default)]
        struct SeQuedaSinDisco(u32);
        impl Destino for SeQuedaSinDisco {
            fn inventario(&mut self, _: &Inventario) -> Result<(), DbError> {
                Ok(())
            }
            fn fila(&mut self, _: &str, _: &[Option<String>]) -> Result<(), DbError> {
                self.0 += 1;
                if self.0 > 2 {
                    return Err(DbError::DatoInvalido {
                        motivo: "no queda espacio en el disco",
                    });
                }
                Ok(())
            }
        }

        let (_d, db) = base();
        sembrar(&db);
        let mut d = SeQuedaSinDisco::default();
        assert!(
            db.exportar(QueSeLleva::Todo, &mut d).is_err(),
            "el fallo al escribir tiene que llegar hasta arriba"
        );
    }

    /// Exportar **no escribe nada** en la base.
    ///
    /// Suena obvio y por eso se comprueba: la transacción se abre con
    /// `unchecked_transaction`, que confirmaría al soltarse si nadie dijera lo
    /// contrario.
    #[test]
    fn exportar_no_toca_la_base() {
        let (_d, db) = base();
        sembrar(&db);

        let huella = |db: &Db| -> Vec<(String, i64)> {
            db.ejecutar_en_pruebas(|c| {
                let mut stmt =
                    c.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
                let tablas = stmt
                    .query_map([], |f| f.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                let mut salida = Vec::new();
                for t in tablas {
                    let n: i64 =
                        c.query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |f| f.get(0))?;
                    salida.push((t, n));
                }
                Ok(salida)
            })
            .expect("huella")
        };

        let antes = huella(&db);
        let mut d = EnMemoria::default();
        db.exportar(QueSeLleva::Todo, &mut d).expect("exporta");
        assert_eq!(antes, huella(&db), "exportar cambió algo en la base");
    }
}
