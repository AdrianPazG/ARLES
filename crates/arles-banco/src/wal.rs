//! ¿Cuánto crece el `-wal` con un lector abierto? · la incógnita 1 de ADR-0015
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ SE MIDE Y POR QUÉ NO SE PODÍA SUPONER
//!
//! ADR-0015 decide abrir una **segunda conexión de sólo lectura** para que las
//! pantallas no esperen al escritor. En WAL eso funciona —medido el 28/10/2026:
//! la lectora devuelve con una transacción de escritura abierta y no espera—,
//! pero tiene un coste que el ADR dejó **declarado y sin medir**: un lector
//! retrasa el *checkpoint*, y mientras lo retrasa el archivo `-wal` crece.
//!
//! Suponerlo no vale, porque el coste depende de **cómo** lea el lector, y los
//! dos casos que ARLES tiene son muy distintos:
//!
//!   A · **Lecturas de una en una** — lo que hacen las pantallas. Cada consulta
//!       abre y cierra su propia transacción de lectura implícita, así que el
//!       *checkpoint* sólo se retrasa el instante que dura la consulta.
//!
//!   B · **Una transacción de lectura larga** — lo que hace `Db::exportar`, que
//!       mantiene la suya abierta durante todo el respaldo (8.2b). Ahí el
//!       *checkpoint* no puede avanzar en ningún momento, y el `-wal` acumula
//!       **todo** lo que el escritor escriba mientras dure.
//!
//! El caso B es el que vale la pena conocer antes de mover un solo método,
//! porque es justamente el que ADR-0015 pone como el beneficio principal:
//! hacer un respaldo sin detener los envíos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y UN CUARTO CASO, QUE LA PRIMERA MEDICIÓN OBLIGÓ A AÑADIR
//!
//! Medidos los tres primeros a 30 000, el `-wal` salió **igual en los tres**:
//! 43.6 MB. La razón es que una importación es **una sola transacción**, así
//! que cabe entera en el `-wal` haya lector o no, y el *checkpoint* no puede
//! correr hasta que confirme. Lo que sí cambió fue **la base**: 42.9 MB sin
//! lector y **0.5 MB** con la transacción abierta, porque el *checkpoint* no
//! llegó a volcar nada.
//!
//! Es decir: un lector largo **no engorda el `-wal`**, impide que se vacíe. Y
//! para ver eso acumular hace falta la forma que de verdad lo acumula —
//! **muchas transacciones pequeñas**, que es lo que hace el motor envío a
//! envío—, no una grande. Ése es el caso D.
//!
//! Y se mide con una tercera corrida de referencia **sin ningún lector**, que
//! es la única forma de saber cuánto de lo que crece es por el lector y cuánto
//! crecería igual.

use std::path::Path;
use std::time::Instant;

use arles_core::ids::CompanyId;
use arles_db::{ClaveMaestra, Db};

/// Lo que deja una corrida.
#[derive(Debug, Clone, Copy)]
pub struct Medida {
    /// El pico del `-wal` durante la importación, en bytes.
    pub wal_maximo: u64,
    /// Lo que quedó al terminar, después de que el *checkpoint* pudiera correr.
    pub wal_final: u64,
    /// La base, para poder comparar el `-wal` con lo que protege.
    pub base: u64,
    /// Lo que tardó la importación.
    pub segundos: f64,
    /// Cuántas lecturas alcanzó a hacer el lector mientras duraba.
    pub lecturas: u64,
}

/// Cómo lee el lector mientras el escritor importa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lector {
    /// Ninguno: la referencia.
    Ninguno,
    /// Una consulta cada vez, como una pantalla.
    DeUnaEnUna,
    /// Una transacción abierta de principio a fin, como `Db::exportar`.
    ConLaTransaccionAbierta,
    /// Muchas transacciones pequeñas **sin** lector: la referencia del caso de
    /// abajo. Sin ella, los 373 MB no dirían nada — no se sabría cuánto es por
    /// el lector y cuánto crecería igual.
    PequenasSinLector,
    /// Como `ConLaTransaccionAbierta`, pero el escritor hace **muchas
    /// transacciones pequeñas** en vez de una grande — la forma del motor, que
    /// confirma cada envío—. Es la única en la que un *checkpoint* retrasado
    /// acumula de verdad, y la añadió haber medido las otras tres.
    PequenasConLaTransaccionAbierta,
}

impl Lector {
    const fn nombre(self) -> &'static str {
        match self {
            Self::Ninguno => "sin lector (referencia)",
            Self::DeUnaEnUna => "lector de una en una (pantallas)",
            Self::ConLaTransaccionAbierta => "lector con transacción abierta (respaldo)",
            Self::PequenasSinLector => "transacciones pequeñas, sin lector (referencia)",
            Self::PequenasConLaTransaccionAbierta => {
                "transacciones pequeñas + lector abierto (motor + respaldo)"
            }
        }
    }
}

fn tamano(p: &Path) -> u64 {
    std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}

/// Importa `cuantas` filas con el lector que se diga, midiendo el `-wal`.
///
/// # Errores
///
/// Lo que falle al generar el conjunto, abrir la base o importar.
pub fn correr(
    cuantas: usize,
    semilla: u64,
    lector: Lector,
) -> Result<Medida, Box<dyn std::error::Error>> {
    const EMPRESA: &str = "00000000-0000-4000-8000-000000000001";

    let dir = tempfile::tempdir()?;
    let csv = dir.path().join("sintetico.csv");
    crate::escribir_csv(&csv, cuantas, semilla)?;

    let ruta = dir.path().join("arles.db");
    let wal = dir.path().join("arles.db-wal");
    let clave = ClaveMaestra::generar()?;
    let db = Db::abrir(&ruta, &clave)?;
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'Banco', 'MX', 'America/Mexico_City',
                     'banco@ejemplo.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');"
        ))
    })?;

    let tabla = arles_import::leer(&csv)?;
    let empresa: CompanyId = EMPRESA.parse().map_err(|e| format!("empresa: {e:?}"))?;

    // ── LA SEGUNDA CONEXIÓN, CON `query_only` ────────────────────────────
    //
    // Es la de ADR-0015, abierta como la abriría el producto: la misma clave,
    // y `query_only = 1` para que una escritura por aquí no falle a veces sino
    // siempre. Medido el 28/10/2026: devuelve `Err(ReadOnly, 8)`.
    let lectora = if matches!(lector, Lector::Ninguno | Lector::PequenasSinLector) {
        None
    } else {
        let c = arles_db::conexion::abrir(&ruta, &clave)?;
        c.pragma_update(None, "query_only", true)?;
        Some(c)
    };

    // El hilo que lee mientras el otro importa. Se le dice que pare con un
    // booleano compartido, no con un `join` a destiempo: lo que se mide es el
    // `-wal` **durante** la importación.
    let parar = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let pico = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let leidas = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let vigilante = {
        let parar = std::sync::Arc::clone(&parar);
        let pico = std::sync::Arc::clone(&pico);
        let leidas = std::sync::Arc::clone(&leidas);
        let wal = wal.clone();
        std::thread::spawn(move || {
            use std::sync::atomic::Ordering;
            // La transacción larga se abre **una vez**, aquí, y no se cierra
            // hasta el final: es lo que hace el caso B.
            let tx = match (&lectora, lector) {
                (
                    Some(c),
                    Lector::ConLaTransaccionAbierta | Lector::PequenasConLaTransaccionAbierta,
                ) => {
                    c.execute_batch("BEGIN").ok();
                    // Una consulta de verdad, porque `BEGIN` a secas en SQLite
                    // es perezoso: la transacción de lectura **no empieza**
                    // hasta la primera lectura, así que sin esto no se estaría
                    // reteniendo nada y el caso B mediría el caso A.
                    c.query_row("SELECT count(*) FROM contact", [], |f| f.get::<_, i64>(0))
                        .ok();
                    true
                }
                _ => false,
            };

            while !parar.load(Ordering::Relaxed) {
                pico.fetch_max(tamano(&wal), Ordering::Relaxed);
                if let (Some(c), Lector::DeUnaEnUna) = (&lectora, lector)
                    && c.query_row("SELECT count(*) FROM contact", [], |f| f.get::<_, i64>(0))
                        .is_ok()
                {
                    leidas.fetch_add(1, Ordering::Relaxed);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            pico.fetch_max(tamano(&wal), Ordering::Relaxed);
            if tx && let Some(c) = &lectora {
                c.execute_batch("COMMIT").ok();
            }
        })
    };

    let t = Instant::now();
    if matches!(
        lector,
        Lector::PequenasSinLector | Lector::PequenasConLaTransaccionAbierta
    ) {
        // Una transacción por fila, que es la forma del motor: confirma cada
        // envío. `cuantas` aquí son transacciones, no contactos, y se mantiene
        // modesto a propósito — lo que se mide es la FORMA de la curva, no el
        // tiempo.
        let cuantas_tx = cuantas.min(20_000);
        db.ejecutar_en_pruebas(|c| {
            for i in 0..cuantas_tx {
                // Las columnas son las de la V1 y no las que uno recuerda:
                // `entity_kind`, `created_at` y un `id` que no es autoincremental.
                // La primera versión escribió `entity` y `at`, y reventó con
                // «table audit_log has no column named entity» — es el tercer
                // esquema que me invento en esta sesión en vez de mirarlo.
                c.execute(
                    "INSERT INTO audit_log (id, company_id, actor, action,
                                            entity_kind, entity_id, created_at)
                     VALUES (?1, ?2, 'banco', 'medir', 'wal', ?1,
                             '2026-10-28T00:00:00Z')",
                    rusqlite::params![i.to_string(), EMPRESA],
                )?;
            }
            Ok(())
        })?;
    } else {
        crate::medir::importar_para_el_wal(&db, empresa, &tabla)?;
    }
    let segundos = t.elapsed().as_secs_f64();

    parar.store(true, std::sync::atomic::Ordering::Relaxed);
    vigilante.join().map_err(|_| "el vigilante del wal murió")?;

    // El final se mide **después** de soltar la lectora: con ella viva, el
    // `-wal` no se puede recortar y la cifra diría lo del pico. Lo que esta
    // línea contesta es «¿se recupera el espacio cuando el lector se va?».
    let wal_final = tamano(&wal);

    Ok(Medida {
        wal_maximo: pico.load(std::sync::atomic::Ordering::Relaxed),
        wal_final,
        base: tamano(&ruta),
        segundos,
        lecturas: leidas.load(std::sync::atomic::Ordering::Relaxed),
    })
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}

/// Corre los tres casos y los imprime comparados.
///
/// # Errores
///
/// Lo que falle en cualquiera de las tres corridas.
pub fn correr_los_tres(cuantas: usize, semilla: u64) -> Result<String, Box<dyn std::error::Error>> {
    let mut s = String::new();
    s.push_str(&format!(
        "Crecimiento del `-wal` con {cuantas} contactos · incógnita 1 de ADR-0015\n\n\
         | Caso | `-wal` máximo | `-wal` al soltar | Base | Importar | Lecturas |\n\
         |---|---|---|---|---|---|\n"
    ));
    for lector in [
        Lector::Ninguno,
        Lector::DeUnaEnUna,
        Lector::ConLaTransaccionAbierta,
        Lector::PequenasSinLector,
        Lector::PequenasConLaTransaccionAbierta,
    ] {
        let m = correr(cuantas, semilla, lector)?;
        eprintln!(
            "{}: wal max {} · final {} · base {} · {:.1} s · {} lecturas",
            lector.nombre(),
            mb(m.wal_maximo),
            mb(m.wal_final),
            mb(m.base),
            m.segundos,
            m.lecturas
        );
        s.push_str(&format!(
            "| {} | {} | {} | {} | {:.1} s | {} |\n",
            lector.nombre(),
            mb(m.wal_maximo),
            mb(m.wal_final),
            mb(m.base),
            m.segundos,
            m.lecturas
        ));
    }
    Ok(s)
}
