//! Restaurar un respaldo de un **esquema anterior**, poniéndolo al día.
//!
//! Cierra la limitación que la 8.3b dejó declarada y que la V11 disparó por
//! primera vez: *«un respaldo tomado antes de una migración futura no se podrá
//! restaurar: haría falta correr las migraciones sobre el propio respaldo, que
//! es otra entrega»*. Ésta es esa entrega.
//!
//! Hasta hoy, cada migración dejaba sin poder restaurar todos los `.arles`
//! anteriores, y eso **no es una molestia: es una copia de seguridad que muere
//! sin que nadie se entere**. El archivo sigue en el disco, se puede revisar, la
//! pantalla cuenta lo que trae — y el día que haga falta no entra.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! CÓMO, Y POR QUÉ ASÍ Y NO «LO QUE ENCAJE»
//!
//! Las dos salidas estaban escritas desde la 8.3b, y una está descartada por
//! escrito: insertar las columnas que coincidan y dejar las demás en su valor
//! por defecto. Eso es un apellido nulo y una fecha basura, en una base que
//! arranca y parece funcionar.
//!
//! La otra es correr las migraciones sobre los datos del respaldo. Y la forma de
//! hacerlo **sin escribir ni una línea de SQL nueva** es levantar una base
//! desechable en la versión que el respaldo declara, meterle sus filas, y
//! aplicarle las migraciones que faltan:
//!
//! ```text
//!   respaldo V11  ──┐
//!                   ├─> base desechable en V11  ──(V12…V16)──>  V16  ──┐
//!   cabecera V11  ──┘                                                  │
//!                                                                      v
//!                                         Db::restaurar(base real, …)
//! ```
//!
//! El SQL que transforma los datos es **exactamente el que ya corrió en la base
//! de quien actualizó ARLES**, con los mismos PRAGMA y el mismo orden. No hay
//! una segunda traducción que mantener al día, que es la parte que se habría
//! quedado atrás en la primera migración después de ésta.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA BASE DESECHABLE VA CIFRADA, Y NO ES CEREMONIA
//!
//! Lleva los contactos, las bajas y el historial de una empresa. Escribirla en
//! claro para un trámite interno sería el único sitio de ARLES donde esos datos
//! tocan el disco sin cifrar — y §29 no admite «menos en este caso».
//!
//! Su clave es **nueva y de usar y tirar**: nace de `getrandom`, vive en memoria
//! lo que dura la operación y no va al llavero. No hace falta recuperarla nunca:
//! si algo se interrumpe, lo que queda es un archivo que nadie puede abrir y que
//! el borrado de la carpeta se lleva. La contraseña del respaldo, que sí es del
//! usuario, no se guarda en ningún sitio (8.2a) y aquí tampoco se queda.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA SUMA SE COMPRUEBA ANTES, Y ESO MEJORA LA GARANTÍA DE LA 8.3c
//!
//! Restaurar sin poner al día comprueba la suma sha256 **dentro** de la
//! transacción, al llegar a la línea de cierre: un archivo alterado revienta
//! antes del `COMMIT`.
//!
//! Poniendo al día, el archivo se lee entero **antes** de tocar la base real —
//! para llenar la desechable—, así que la suma se comprueba antes, no después.
//! La garantía no se relaja: se refuerza. Un archivo cortado o manipulado falla
//! con la base real todavía intacta, y además falla sin haber tomado su cerrojo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTO **NO** ARREGLA, DICHO EN VEZ DE DARLO POR CUBIERTO
//!
//! - **Un respaldo del futuro no se puede poner al día.** Las migraciones que
//!   lo trajeron hasta ahí no están en este binario, y no se inventan. Se
//!   rechaza con [`DbError::RespaldoDeUnArlesPosterior`], que manda a actualizar
//!   ARLES — una instrucción que se puede seguir.
//! - **Un respaldo esencial puesto al día puede no dar lo mismo que la base
//!   completa migrada.** Una migración que rellene una columna leyendo una tabla
//!   que el esencial no lleva no tiene de dónde leer, y rellenará menos. Es
//!   inherente a que el esencial no lleve esas tablas, no de este módulo: la
//!   alternativa sería inventarse los datos que faltan.
//! - **Hace falta espacio en disco** para una base del tamaño del respaldo, a
//!   la vez que el respaldo y la base real. Un disco lleno a mitad falla con la
//!   base real intacta, que es el lado correcto.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use arles_core::formato_de_respaldo::Alcance;
use rusqlite::Connection;

use crate::conexion::ClaveMaestra;
use crate::error::DbError;
use crate::respaldo::ColumnasPorTabla;
use crate::restauracion::{FilaDelRespaldo, Origen};

/// Cuántas filas se traen de la base desechable de una vez.
///
/// Es un **cursor por clave** (`rowid > el último`), no un `LIMIT … OFFSET`: la
/// lección de la 8.2b es que SQLite no guarda un atajo a la fila n, así que
/// medio millón en tandas con `OFFSET` visita unas 125 000 millones de filas.
/// Aquí cada tanda arranca donde acabó la anterior y el índice de la clave la
/// encuentra de un salto.
///
/// Quinientas y no todas: quinientas filas son memoria acotada —el cuerpo entero
/// sería la base en RAM, que es lo que las 8.2b, 8.2c y 8.3a evitaron cada una
/// por su lado— y suficientes para que el coste por tanda no se note.
const FILAS_POR_TANDA: i64 = 500;

/// Qué se puede hacer con un respaldo de un esquema dado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Veredicto {
    /// Es del esquema de hoy: se restaura tal cual, sin pasar por aquí.
    YaEstaAlDia,
    /// Es anterior y se puede poner al día.
    SePuedePonerAlDia {
        desde: i32,
        hasta: i32,
        /// Cuántas migraciones habrá que aplicarle. Es una cifra para decirla
        /// (§94), no para decidir nada.
        migraciones: usize,
    },
    /// Lo escribió un ARLES posterior. **No se puede**, y la salida es
    /// actualizar ARLES.
    EsDeUnArlesPosterior { version: i32, el_de_aqui: i32 },
    /// La cabecera no dice una versión que se entienda.
    NoSeEntiende,
}

/// Qué hacer con un respaldo que declara este esquema.
///
/// Se contesta **sin abrir la base**: la versión de referencia es la última
/// migración que este binario conoce, que es hasta donde se puede migrar. La
/// base de este equipo siempre está ahí — [`crate::conexion::abrir`] migra al
/// abrirla y no hay forma de quedarse a medias—, así que compararse con el
/// binario y compararse con la base dan lo mismo, y esto además contesta antes
/// de que haya base.
#[must_use]
pub fn que_se_puede_hacer(esquema_del_respaldo: &str) -> Veredicto {
    let Some(desde) = version_de(esquema_del_respaldo) else {
        return Veredicto::NoSeEntiende;
    };
    let conocidas = crate::migraciones::versiones();
    let Some(&hasta) = conocidas.last() else {
        return Veredicto::NoSeEntiende;
    };
    if desde == hasta {
        return Veredicto::YaEstaAlDia;
    }
    if desde > hasta {
        return Veredicto::EsDeUnArlesPosterior {
            version: desde,
            el_de_aqui: hasta,
        };
    }
    // Las que faltan, contadas de las que hay y no restando números: un hueco
    // en la numeración —una migración retirada antes de publicarse, como estuvo
    // a punto de pasar con la V17— haría que `hasta - desde` dijera una más de
    // las que se van a aplicar.
    Veredicto::SePuedePonerAlDia {
        desde,
        hasta,
        migraciones: conocidas.iter().filter(|v| **v > desde).count(),
    }
}

/// `"V11"` → `11`. Cualquier otra cosa, `None`.
///
/// ── EL CERO Y LOS NEGATIVOS SE RECHAZAN, Y NO ES CELO ──
///
/// `parse::<i32>` acepta `-3`, así que la primera versión de esta función daba
/// `SePuedePonerAlDia { desde: -3 }` para una cabecera que dijera `V-3` — lo
/// cazó su propia prueba. Con eso, `aplicar_hasta` no aplicaría ninguna
/// migración, la base desechable nacería vacía, y el fallo llegaría dos pasos
/// después diciendo que *el respaldo trae una tabla que esta base no tiene*: un
/// diagnóstico equivocado, que manda a hacer algo (DOC-14). La primera
/// migración es la V1; por debajo de ahí no hay esquema que levantar.
fn version_de(esquema: &str) -> Option<i32> {
    let n: i32 = esquema.strip_prefix('V')?.parse().ok()?;
    (n >= 1).then_some(n)
}

/// Una carpeta que se borra al soltarla, con todo lo que tenga dentro.
///
/// ── POR QUÉ ES UN CAMPO Y NO UN `impl Drop for RespaldoAlDia` ──
///
/// Porque el orden importa y es sutil: un `Drop` del struct entero corre
/// **antes** de que se suelten sus campos, así que la conexión seguiría abierta
/// al borrar el archivo. En Linux eso funciona por accidente; en Windows,
/// borrar un archivo abierto falla y la carpeta se queda. Como campo declarado
/// **después** de `conn`, el orden de los campos lo garantiza el compilador.
#[derive(Debug)]
struct CarpetaQueSeBorra(PathBuf);

impl Drop for CarpetaQueSeBorra {
    fn drop(&mut self) {
        // Si no se puede borrar, no hay nada que hacer y menos que decir: lo
        // que queda es un archivo cifrado con una clave que ya no existe.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Un respaldo de un esquema anterior, ya puesto al día y listo para
/// [`crate::db::Db::restaurar`].
///
/// Implementa [`Origen`], así que el restaurador no sabe —ni tiene por qué—
/// que estas filas vienen de una base desechable y no de un archivo.
#[derive(Debug)]
pub struct RespaldoAlDia {
    /// **Declarada antes que la carpeta a propósito**: ver `CarpetaQueSeBorra`.
    conn: Connection,
    /// Nadie la lee: existe para que su `Drop` borre la carpeta, y de ahí el
    /// subrayado. Lo que hace falta de ella es **dónde** está declarada.
    _carpeta: CarpetaQueSeBorra,

    esquema: String,
    columnas: ColumnasPorTabla,
    filas_del_respaldo: u64,
    filas_al_dia: u64,

    /// Las tablas que quedan por entregar, en orden de inserción.
    pendientes: VecDeque<&'static str>,
    /// La tabla en curso y el último `rowid` que se entregó de ella.
    en_curso: Option<(&'static str, i64)>,
    tanda: VecDeque<FilaDelRespaldo>,
}

impl RespaldoAlDia {
    /// Pone el respaldo al día. **No toca la base real** y no la necesita.
    ///
    /// `origen` se consume entero aquí: es lo que adelanta la comprobación de la
    /// suma (ver la cabecera del módulo). `junto_a` es la carpeta donde se crea
    /// la base desechable — la de la base real, no la temporal del sistema, que
    /// puede estar en otro volumen y con otro tamaño.
    ///
    /// # Errores
    ///
    /// - [`DbError::RespaldoDeUnArlesPosterior`] si es de un ARLES más nuevo.
    /// - [`DbError::EsoEsUnArchivoDeHistorial`] si no es un respaldo. Se
    ///   comprueba aquí **además** de en el preflight y en el restaurador: ésta
    ///   es una tercera puerta a lo irreversible, y poner al día un archivo de
    ///   historial produciría un respaldo del esquema de hoy con un año de
    ///   envíos y nada más, que es justo lo que D-8C convertiría en una base
    ///   vacía.
    /// - [`DbError::NoSePudoPonerAlDia`] si una migración falla sobre sus datos.
    /// - [`DbError::RespaldoNoEncaja`] si el respaldo declara una tabla o
    ///   columna que su propio esquema no tiene — un archivo manipulado.
    pub fn nuevo<O: Origen>(
        origen: &mut O,
        esquema_del_respaldo: &str,
        columnas: &ColumnasPorTabla,
        alcance: &Alcance,
        junto_a: &Path,
    ) -> Result<Self, DbError> {
        if !alcance.se_puede_restaurar() {
            return Err(DbError::EsoEsUnArchivoDeHistorial);
        }
        let desde = match que_se_puede_hacer(esquema_del_respaldo) {
            Veredicto::SePuedePonerAlDia { desde, .. } => desde,
            Veredicto::YaEstaAlDia => {
                return Err(DbError::NoSePudoPonerAlDia {
                    motivo: "este respaldo ya es del esquema de hoy, no hay nada que poner al día"
                        .to_owned(),
                });
            }
            Veredicto::EsDeUnArlesPosterior {
                version,
                el_de_aqui,
            } => {
                return Err(DbError::RespaldoDeUnArlesPosterior {
                    version,
                    el_de_aqui,
                });
            }
            Veredicto::NoSeEntiende => {
                return Err(DbError::RespaldoNoEncaja {
                    motivo: format!(
                        "la cabecera dice que el esquema es «{esquema_del_respaldo}», que no es una versión que ARLES sepa leer"
                    ),
                });
            }
        };

        // ── 1 · La base desechable, en la versión del respaldo ──
        let carpeta = CarpetaQueSeBorra(crear_la_carpeta(junto_a)?);
        let ruta_de_la_base = carpeta.0.join("al-dia.db");
        let clave = ClaveMaestra::generar()?;
        let mut conn = crate::conexion::abrir_crudo(&ruta_de_la_base, &clave)?;
        crate::migraciones::aplicar_hasta(&mut conn, desde).map_err(|e| {
            DbError::NoSePudoPonerAlDia {
                motivo: format!("no se pudo levantar el esquema V{desde} para leerlo: {e}"),
            }
        })?;

        // ── 2 · Sus filas, en el orden en que el archivo las trae ──
        //
        // Que ya es un orden de inserción válido para SU esquema: lo escribió
        // el exportador calculándolo de las claves ajenas. Aquí no hay nada que
        // vaciar —la base acaba de nacer— y nada que reordenar.
        let filas_del_respaldo = {
            let tx = conn.unchecked_transaction()?;
            let (filas, _) = crate::restauracion::meter_las_filas(&tx, origen, columnas)?;
            tx.commit()?;
            filas
        };

        // ── 3 · Y las migraciones que faltan, sobre los datos ──
        //
        // Lo mismo que le pasó a la base de quien actualizó ARLES. Si una falla,
        // falla **aquí**: la base real no se ha abierto todavía.
        crate::migraciones::aplicar(&mut conn).map_err(|e| DbError::NoSePudoPonerAlDia {
            motivo: format!(
                "una de las migraciones posteriores a la V{desde} no se pudo aplicar a sus datos: {e}"
            ),
        })?;

        // ── 4 · Lo que ha quedado, para poder entregarlo y contarlo ──
        let esquema = crate::respaldo::version_del_esquema(&conn)?;
        let existen = tablas_que_existen(&conn)?;
        let del_alcance: Vec<&'static str> = crate::respaldo::tablas_del_alcance(alcance)
            .into_iter()
            .filter(|t| existen.iter().any(|e| e == t))
            .collect();
        let pendientes = crate::respaldo::orden_de_insercion(&conn, &del_alcance)?;

        let mut columnas_de_hoy: ColumnasPorTabla = Vec::with_capacity(pendientes.len());
        let mut filas_al_dia = 0_u64;
        for tabla in &pendientes {
            columnas_de_hoy.push((
                (*tabla).to_owned(),
                crate::respaldo::columnas_de(&conn, tabla)?,
            ));
            filas_al_dia += cuantas(&conn, tabla)?;
        }

        Ok(Self {
            conn,
            _carpeta: carpeta,
            esquema,
            columnas: columnas_de_hoy,
            filas_del_respaldo,
            filas_al_dia,
            pendientes: pendientes.into_iter().collect(),
            en_curso: None,
            tanda: VecDeque::new(),
        })
    }

    /// El esquema que tienen ya estas filas: el de hoy.
    #[must_use]
    pub fn esquema(&self) -> &str {
        &self.esquema
    }

    /// Las columnas de cada tabla **en el esquema de hoy**, para que el
    /// restaurador inserte por nombre y no por posición.
    #[must_use]
    pub fn columnas(&self) -> &ColumnasPorTabla {
        &self.columnas
    }

    /// Cuántas filas traía el respaldo tal como estaba.
    #[must_use]
    pub fn filas_del_respaldo(&self) -> u64 {
        self.filas_del_respaldo
    }

    /// Cuántas filas hay después de ponerlo al día.
    ///
    /// **No tiene por qué ser la misma cifra, y la diferencia se dice.** La V13,
    /// por ejemplo, borra los cubos de ritmo que se quedan sin etapa. Una
    /// migración puede quitar filas con todo el derecho; lo que no puede es
    /// quitarlas sin que nadie lo cuente.
    #[must_use]
    pub fn filas_al_dia(&self) -> u64 {
        self.filas_al_dia
    }

    /// Trae la siguiente tanda de la tabla en curso, o pasa a la siguiente
    /// tabla. Devuelve `false` cuando ya no queda nada.
    fn traer_otra_tanda(&mut self) -> Result<bool, DbError> {
        loop {
            let (tabla, ultimo) = match self.en_curso {
                Some(x) => x,
                None => match self.pendientes.pop_front() {
                    Some(t) => (t, 0),
                    None => return Ok(false),
                },
            };
            let cols = self
                .columnas
                .iter()
                .find(|(t, _)| t == tabla)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let lista = cols
                .iter()
                .map(|c| format!("\"{c}\""))
                .collect::<Vec<_>>()
                .join(", ");

            // `rowid` va primero para recordar dónde se quedó. Ninguna tabla del
            // esquema es `WITHOUT ROWID` —lo comprueba
            // `ninguna_tabla_del_esquema_se_queda_sin_rowid`—, que es lo que
            // permite este cursor sin inventar una clave por tabla.
            let sql = format!(
                "SELECT rowid, {lista} FROM \"{tabla}\" WHERE rowid > ?1 ORDER BY rowid LIMIT ?2"
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let mut filas = stmt.query(rusqlite::params![ultimo, FILAS_POR_TANDA])?;
            let mut nuevo_ultimo = ultimo;
            let mut cuantas_trajo = 0_i64;
            while let Some(fila) = filas.next()? {
                nuevo_ultimo = fila.get(0)?;
                let mut valores = Vec::with_capacity(cols.len());
                for i in 0..cols.len() {
                    valores.push(crate::respaldo::valor_como_texto(
                        tabla,
                        fila.get_ref(i + 1)?,
                    )?);
                }
                self.tanda.push_back((tabla.to_owned(), valores));
                cuantas_trajo += 1;
            }

            if cuantas_trajo == 0 {
                // Esta tabla se acabó: a la siguiente, y si no hay, se acabó
                // todo.
                self.en_curso = None;
                if self.pendientes.is_empty() {
                    return Ok(false);
                }
                continue;
            }
            self.en_curso = Some((tabla, nuevo_ultimo));
            return Ok(true);
        }
    }
}

impl Origen for RespaldoAlDia {
    fn siguiente(&mut self) -> Result<Option<FilaDelRespaldo>, DbError> {
        loop {
            if let Some(fila) = self.tanda.pop_front() {
                return Ok(Some(fila));
            }
            if !self.traer_otra_tanda()? {
                return Ok(None);
            }
        }
    }
}

/// Las tablas que existen en esta base, sin las del motor de migraciones.
fn tablas_que_existen(conn: &Connection) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_schema
          WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
            AND name <> 'refinery_schema_history'
          ORDER BY name",
    )?;
    let tablas = stmt
        .query_map([], |f| f.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tablas)
}

fn cuantas(conn: &Connection, tabla: &str) -> Result<u64, DbError> {
    let n: i64 = conn.query_row(&format!("SELECT count(*) FROM \"{tabla}\""), [], |f| {
        f.get(0)
    })?;
    Ok(u64::try_from(n).unwrap_or(0))
}

/// Crea la carpeta de la base desechable, con un nombre que no puede chocar.
///
/// El nombre lleva aleatoriedad y no la hora: dos puestas al día en el mismo
/// segundo —dos ventanas, o una repetida tras un fallo— compartirían carpeta y
/// la segunda se encontraría la base de la primera a medio llenar.
fn crear_la_carpeta(junto_a: &Path) -> Result<PathBuf, DbError> {
    let mut azar = [0_u8; 8];
    getrandom::fill(&mut azar).map_err(|_| DbError::SinAleatoriedad)?;
    let nombre: String = azar.iter().map(|b| format!("{b:02x}")).collect();
    let ruta = junto_a.join(format!("arles-poner-al-dia-{nombre}"));
    std::fs::create_dir_all(&ruta).map_err(|e| DbError::NoSePudoPonerAlDia {
        motivo: format!("no se pudo crear la carpeta de trabajo: {e}"),
    })?;
    Ok(ruta)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un respaldo en memoria: las filas que una base vieja tenía, con las
    /// columnas que tenía entonces.
    ///
    /// ── POR QUÉ NO SE FABRICAN LAS FILAS A MANO ──
    ///
    /// Porque entonces la prueba compararía mi idea de cómo era el esquema
    /// antiguo con mi idea de cómo se migra. Las filas y las columnas se **leen
    /// de una base de verdad** levantada por las mismas migraciones, así que lo
    /// único inventado aquí es el envoltorio.
    struct ElRespaldoEnMemoria {
        filas: VecDeque<FilaDelRespaldo>,
    }

    impl Origen for ElRespaldoEnMemoria {
        fn siguiente(&mut self) -> Result<Option<FilaDelRespaldo>, DbError> {
            Ok(self.filas.pop_front())
        }
    }

    /// Levanta una base en `version`, la deja poblar, y devuelve lo que un
    /// respaldo suyo habría llevado: sus filas y sus columnas.
    fn respaldo_de_una_base_en_la_version(
        version: i32,
        poblar: &str,
    ) -> (ElRespaldoEnMemoria, ColumnasPorTabla) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let mut conn =
            crate::conexion::abrir_crudo(&dir.path().join("vieja.db"), &clave).expect("abre");
        crate::migraciones::aplicar_hasta(&mut conn, version).expect("migra hasta ahí");
        conn.execute_batch(poblar).expect("puebla");

        let nombres = tablas_que_existen(&conn).expect("tablas");
        let refs: Vec<&str> = nombres.iter().map(String::as_str).collect();
        let orden = crate::respaldo::orden_de_insercion(&conn, &refs).expect("orden");

        let mut columnas: ColumnasPorTabla = Vec::new();
        let mut filas: VecDeque<FilaDelRespaldo> = VecDeque::new();
        for tabla in &orden {
            let cols = crate::respaldo::columnas_de(&conn, tabla).expect("columnas");
            let lista = cols
                .iter()
                .map(|c| format!("\"{c}\""))
                .collect::<Vec<_>>()
                .join(", ");
            let mut stmt = conn
                .prepare(&format!("SELECT {lista} FROM \"{tabla}\""))
                .expect("prepara");
            let mut rs = stmt.query([]).expect("consulta");
            while let Some(f) = rs.next().expect("fila") {
                let mut valores = Vec::with_capacity(cols.len());
                for i in 0..cols.len() {
                    valores.push(
                        crate::respaldo::valor_como_texto(tabla, f.get_ref(i).expect("valor"))
                            .expect("texto"),
                    );
                }
                filas.push_back(((*tabla).to_owned(), valores));
            }
            columnas.push(((*tabla).to_owned(), cols));
        }
        (ElRespaldoEnMemoria { filas }, columnas)
    }

    fn hoy() -> i32 {
        crate::migraciones::ultima_version().expect("hay migraciones")
    }

    fn recoger(al_dia: &mut RespaldoAlDia) -> Vec<FilaDelRespaldo> {
        let mut todo = Vec::new();
        while let Some(f) = al_dia.siguiente().expect("entrega") {
            todo.push(f);
        }
        todo
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EL VEREDICTO
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn el_esquema_de_hoy_no_hay_que_ponerlo_al_dia() {
        assert_eq!(
            que_se_puede_hacer(&format!("V{}", hoy())),
            Veredicto::YaEstaAlDia
        );
    }

    #[test]
    fn un_esquema_anterior_se_puede_poner_al_dia_y_dice_cuantas_faltan() {
        let hasta = hoy();
        match que_se_puede_hacer("V11") {
            Veredicto::SePuedePonerAlDia {
                desde,
                hasta: h,
                migraciones,
            } => {
                assert_eq!(desde, 11);
                assert_eq!(h, hasta);
                assert_eq!(
                    migraciones,
                    usize::try_from(hasta - 11).expect("caben"),
                    "las migraciones que faltan se cuentan de las que hay, no restando números"
                );
            }
            otro => panic!("un respaldo V11 dio {otro:?}"),
        }
    }

    #[test]
    fn un_esquema_del_futuro_no_se_pone_al_dia() {
        let hasta = hoy();
        assert_eq!(
            que_se_puede_hacer(&format!("V{}", hasta + 1)),
            Veredicto::EsDeUnArlesPosterior {
                version: hasta + 1,
                el_de_aqui: hasta,
            }
        );
    }

    /// **Una cabecera que no dice una versión no se interpreta.**
    ///
    /// Caer en «la más antigua» o en «la de hoy» serían las dos formas de
    /// adivinar, y las dos acabarían metiendo filas en un esquema que no es el
    /// suyo.
    #[test]
    fn una_version_que_no_se_entiende_no_se_adivina() {
        for escrito in ["", "V", "11", "Vonce", "V1.1", "v11", "V-3", "V0"] {
            assert_eq!(
                que_se_puede_hacer(escrito),
                Veredicto::NoSeEntiende,
                "«{escrito}» se interpretó como una versión"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EL CURSOR
    // ═══════════════════════════════════════════════════════════════════════

    /// El cursor de la entrega va por `rowid`, así que **ninguna tabla puede
    /// ser `WITHOUT ROWID`**.
    ///
    /// Se lee del esquema y no de una lista: una tabla nueva declarada así
    /// entregaría cero filas —`rowid` no existe y la consulta falla— y lo haría
    /// justo cuando alguien restaura un respaldo viejo. Marcarla `Nunca` en el
    /// manifiesto es válido; que exista sin que nadie lo sepa, no.
    #[test]
    fn ninguna_tabla_del_esquema_se_queda_sin_rowid() {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let conn = crate::conexion::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_schema
                  WHERE type = 'table' AND sql LIKE '%WITHOUT ROWID%'",
            )
            .expect("prepara");
        let sin_rowid: Vec<String> = stmt
            .query_map([], |f| f.get(0))
            .expect("consulta")
            .filter_map(Result::ok)
            .collect();
        assert!(
            sin_rowid.is_empty(),
            "estas tablas son WITHOUT ROWID y el cursor de la puesta al día no \
             las puede recorrer: {sin_rowid:?}"
        );
    }

    /// **El cursor cruza varias tandas sin saltarse ni repetir una fila.**
    ///
    /// Es la lección de la 8.6 F, donde un respaldo cifrado de 30 000 contactos
    /// no se podía restaurar porque **todas** las pruebas de ida y vuelta usaban
    /// dos filas: un solo trozo, ninguna frontera, y el único trozo que se
    /// ejercitaba era el único que no puede fallar.
    ///
    /// Así que esta prueba **declara su margen**: con 1 250 contactos y tandas
    /// de 500 se cruzan dos fronteras en esa tabla (500 · 500 · 250). El día que
    /// alguien suba `FILAS_POR_TANDA` por encima de 625 esta prueba deja de
    /// cruzar ninguna, y por eso la cuenta está escrita como una comprobación y
    /// no como un comentario.
    #[test]
    fn el_cursor_entrega_todas_las_filas_cruzando_varias_tandas() {
        const CUANTOS: usize = 1_250;
        assert!(
            CUANTOS as i64 > FILAS_POR_TANDA * 2,
            "con {CUANTOS} filas y tandas de {FILAS_POR_TANDA} no se cruzan dos fronteras: \
             esta prueba dejaría de probar lo que dice"
        );

        let mut siembra = String::from(
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email,
                                  created_at, updated_at)
               VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'h@t.mx',
                       '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');\n",
        );
        for i in 0..CUANTOS {
            siembra.push_str(&format!(
                "INSERT INTO contact (id, company_id, first_name, created_at, updated_at)
                   VALUES ('C{i}', 'E1', 'Persona {i}', '2026-01-02T00:00:00Z',
                           '2026-01-02T00:00:00Z');\n"
            ));
        }

        let (mut origen, columnas) = respaldo_de_una_base_en_la_version(11, &siembra);
        let dir = tempfile::tempdir().expect("temporal");
        let mut al_dia = RespaldoAlDia::nuevo(
            &mut origen,
            "V11",
            &columnas,
            &Alcance::Completo,
            dir.path(),
        )
        .expect("pone al día");

        let entregadas = recoger(&mut al_dia);
        let contactos: Vec<&FilaDelRespaldo> =
            entregadas.iter().filter(|(t, _)| t == "contact").collect();
        assert_eq!(
            contactos.len(),
            CUANTOS,
            "el cursor entregó otra cantidad de contactos: se saltó o repitió filas"
        );

        // Y son los mismos, no 1 250 copias de uno: el `rowid > el último` mal
        // escrito devolvería la primera tanda una y otra vez.
        let distintos: std::collections::BTreeSet<&Option<String>> =
            contactos.iter().filter_map(|(_, v)| v.first()).collect();
        assert_eq!(
            distintos.len(),
            CUANTOS,
            "el cursor repitió identificadores"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LAS MIGRACIONES CORREN SOBRE LOS DATOS
    // ═══════════════════════════════════════════════════════════════════════

    /// **La prueba que da sentido a todo el módulo: una migración que
    /// transforma datos los transforma también en el respaldo.**
    ///
    /// De la V11 a la V16 ninguna migración mueve datos que viajen en un
    /// respaldo —son columnas nuevas e índices—, así que una prueba V11→V16
    /// pasaría en verde con un módulo que sólo copiara filas. Medido antes de
    /// escribir esto, y por eso la prueba arranca en la **V2**, donde sí pasan
    /// dos cosas:
    ///
    /// - la **V3** crea `contact_channel` y traslada ahí el correo que vivía en
    ///   `contact.email_normalized`;
    /// - la **V4** crea `campaign_stage` y convierte cada campaña con remitente
    ///   en una campaña con una etapa de correo.
    ///
    /// Las dos tablas **no existían** cuando se tomó ese respaldo, así que no
    /// venían en el archivo: salen de migrar sus datos. Eso es exactamente lo
    /// que distingue poner al día de «restaurar lo que encaje».
    #[test]
    fn una_migracion_que_mueve_datos_los_mueve_tambien_en_el_respaldo() {
        let (mut origen, columnas) = respaldo_de_una_base_en_la_version(
            2,
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email,
                                  created_at, updated_at)
               VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'h@t.mx',
                       '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, daily_limit,
                                        hourly_limit, created_at, updated_at)
               VALUES ('EA1', 'E1', 'Ventas', 'ventas@t.mx', 'smtp', 'llavero://ea1',
                       50, 10, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO contact (id, company_id, email_raw, email_normalized,
                                  first_name, created_at, updated_at)
               VALUES ('C1', 'E1', 'Ana@Empresa.com', 'ana@empresa.com', 'Ana',
                       '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');
             INSERT INTO campaign (id, company_id, name, status, email_account_id,
                                   daily_limit, hourly_limit, created_at, updated_at)
               VALUES ('CAM1', 'E1', 'Clientes', 'running', 'EA1', 40, 8,
                       '2026-01-03T00:00:00Z', '2026-01-03T00:00:00Z');",
        );

        // El respaldo V2 no podía traer estas dos tablas: no existían.
        assert!(
            !columnas.iter().any(|(t, _)| t == "contact_channel"),
            "la base V2 no debería tener contact_channel"
        );
        assert!(
            !columnas.iter().any(|(t, _)| t == "campaign_stage"),
            "la base V2 no debería tener campaign_stage"
        );

        let dir = tempfile::tempdir().expect("temporal");
        let mut al_dia =
            RespaldoAlDia::nuevo(&mut origen, "V2", &columnas, &Alcance::Completo, dir.path())
                .expect("pone al día");
        assert_eq!(al_dia.esquema(), format!("V{}", hoy()));

        let entregadas = recoger(&mut al_dia);

        // El correo de Ana está ahora en su canal, trasladado por la V3.
        let canales: Vec<&FilaDelRespaldo> = entregadas
            .iter()
            .filter(|(t, _)| t == "contact_channel")
            .collect();
        assert_eq!(
            canales.len(),
            1,
            "la V3 no trasladó el correo del contacto a su canal"
        );
        let canal = canales.first().expect("hay un canal");
        let lleva_el_correo = canal
            .1
            .iter()
            .any(|v| v.as_deref() == Some("ana@empresa.com"));
        assert!(
            lleva_el_correo,
            "el canal entregado no lleva el correo que tenía el contacto: {:?}",
            canal.1
        );

        // Y la campaña tiene su etapa, creada por la V4.
        let etapas = entregadas
            .iter()
            .filter(|(t, _)| t == "campaign_stage")
            .count();
        assert_eq!(etapas, 1, "la V4 no convirtió la campaña en etapa");
    }

    /// **La diferencia de filas se cuenta, no se esconde.**
    ///
    /// Una migración puede quitar filas con todo el derecho —la V13 borra los
    /// cubos de ritmo que se quedan sin etapa—, pero una copia de seguridad que
    /// entra con menos filas de las que decía su cabecera tiene que decir
    /// cuántas y por qué, igual que el preflight dice lo que se pierde por
    /// cascada desde la 8.3a.
    #[test]
    fn las_filas_de_antes_y_de_despues_se_pueden_comparar() {
        let (mut origen, columnas) = respaldo_de_una_base_en_la_version(
            11,
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email,
                                  created_at, updated_at)
               VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'h@t.mx',
                       '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO contact (id, company_id, first_name, created_at, updated_at)
               VALUES ('C1', 'E1', 'Ana', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');",
        );
        let dir = tempfile::tempdir().expect("temporal");
        let al_dia = RespaldoAlDia::nuevo(
            &mut origen,
            "V11",
            &columnas,
            &Alcance::Completo,
            dir.path(),
        )
        .expect("pone al día");

        assert_eq!(al_dia.filas_del_respaldo(), 2, "entraron otras filas");
        assert_eq!(
            al_dia.filas_al_dia(),
            2,
            "de la V11 a hoy ninguna migración debería quitar estas dos filas"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LAS PUERTAS
    // ═══════════════════════════════════════════════════════════════════════

    /// **Un archivo de historial no se pone al día**, que sería la forma de
    /// colarlo por la puerta de atrás: puesto al día sería un respaldo del
    /// esquema de hoy con un año de envíos y nada más, y D-8C lo convertiría en
    /// una base vacía.
    #[test]
    fn un_archivo_de_historial_no_se_pone_al_dia() {
        let (mut origen, columnas) = respaldo_de_una_base_en_la_version(11, "");
        let dir = tempfile::tempdir().expect("temporal");
        let r = RespaldoAlDia::nuevo(
            &mut origen,
            "V11",
            &columnas,
            &Alcance::Historial {
                hasta: "2026-01-01T00:00:00Z".to_owned(),
            },
            dir.path(),
        );
        assert!(
            matches!(r, Err(DbError::EsoEsUnArchivoDeHistorial)),
            "un archivo de historial se puso al día: {r:?}"
        );
    }

    #[test]
    fn un_respaldo_del_futuro_no_se_pone_al_dia() {
        let (mut origen, columnas) = respaldo_de_una_base_en_la_version(11, "");
        let dir = tempfile::tempdir().expect("temporal");
        let r = RespaldoAlDia::nuevo(
            &mut origen,
            &format!("V{}", hoy() + 5),
            &columnas,
            &Alcance::Completo,
            dir.path(),
        );
        assert!(
            matches!(r, Err(DbError::RespaldoDeUnArlesPosterior { .. })),
            "un respaldo del futuro se puso al día: {r:?}"
        );
    }

    /// **La base desechable se borra al soltar el respaldo puesto al día.**
    ///
    /// Lleva los contactos y las bajas de una empresa. Que se quede en el disco
    /// sería una copia de la base que nadie pidió y que nadie va a borrar — y
    /// aunque va cifrada con una clave de usar y tirar, lo que no se borra se
    /// acumula: restaurar cinco veces dejaría cinco bases.
    #[test]
    fn la_base_desechable_no_se_queda_en_el_disco() {
        let (mut origen, columnas) = respaldo_de_una_base_en_la_version(11, "");
        let dir = tempfile::tempdir().expect("temporal");
        let cuantas_antes = std::fs::read_dir(dir.path()).expect("lee").count();

        let al_dia = RespaldoAlDia::nuevo(
            &mut origen,
            "V11",
            &columnas,
            &Alcance::Completo,
            dir.path(),
        )
        .expect("pone al día");
        assert_eq!(
            std::fs::read_dir(dir.path()).expect("lee").count(),
            cuantas_antes + 1,
            "la base desechable tenía que estar ahí mientras se usa"
        );

        drop(al_dia);
        assert_eq!(
            std::fs::read_dir(dir.path()).expect("lee").count(),
            cuantas_antes,
            "la base desechable se quedó en el disco"
        );
    }
}
