//! Por qué la conexión de lectura es 12× más lenta. **Medido, no supuesto.**
//!
//! 03/11/2026. Lo encontró R-26: la misma consulta sobre la misma base de
//! 182 261 contactos tarda **302 ms por el escritor y 3 742 por la lectora**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE YA ESTABA DESCARTADO ANTES DE ESCRIBIR ESTO
//!
//! Con las cifras de R-26, sin medir nada nuevo:
//!
//! 1. **No es el `ESCAPE` del patrón del `LIKE`.** La misma consulta sin él
//!    tarda 315 ms contra 302: un 4 %.
//! 2. **No es el recuento sin tope.** Medido aparte, 335 ms; con la página
//!    suman ~640, no 7 000.
//! 3. **No es el calentamiento de la caché.** Y ésta es la que importa, porque
//!    era la explicación cómoda: las **tres corridas de la lectora son
//!    planas** —3 742, 3 741, 3 576— igual que las tres del escritor. Una
//!    caché fría mejora en la segunda pasada. Esta no mejora, así que la
//!    diferencia es estructural y por corrida.
//! 4. **No es una diferencia de PRAGMA.** Las dos conexiones salen de
//!    `abrir_crudo`, así que llevan los mismos cuatro; lo único que la lectora
//!    añade es `query_only`.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS DOS QUE QUEDAN, Y CÓMO SE DISTINGUEN
//!
//! **A · `query_only`.** Se distingue abriendo una **segunda conexión sin
//! él** —la del escritor, que es la misma `abrir`— y midiendo lo mismo. Si
//! también tarda, `query_only` no es la causa y lo que cuesta es *ser la
//! segunda conexión*.
//!
//! **B · El `-wal`.** En WAL, una conexión que no escribió tiene que mirar el
//! registro antes que el archivo principal para cada página: lo que el
//! escritor tiene ya resuelto, la otra lo busca. Y aquí el `-wal` está en su
//! pico, porque **acabamos de importar** — que es, además, exactamente cuando
//! alguien abre Contactos.
//!
//! Se distingue con un `wal_checkpoint(TRUNCATE)`: si tras volcar el registro
//! la lectora baja al tiempo del escritor, la causa es el `-wal` y el arreglo
//! es un checkpoint, no un pragma.
//!
//! **Se mide el tamaño del `-wal` antes y después**, porque una explicación sin
//! la cifra que la sostiene es una conjetura con buena prensa.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y EL PRIMER INTENTO DE ESTE EXPERIMENTO ESTABA MAL DISEÑADO
//!
//! Medía los cinco caminos **en bloques**: tres corridas del escritor, luego
//! tres de la lectora, luego tres de la segunda conexión, luego el checkpoint
//! y otras seis. Y dio esto:
//!
//! ```text
//! 1 · El escritor, tal cual                        96, 91, 90 ms
//! 2 · La lectora (query_only)                     995, 1117, 1165
//! 3 · Una SEGUNDA conexión sin query_only         127, 131, 99
//! 4 · La lectora, tras el checkpoint              1217, 1138, 1455
//! 5 · El escritor, tras el checkpoint             1281, 1114, 1073
//! ```
//!
//! **El escritor pasó de 90 ms a 1 100.** Eso no lo explica ninguna hipótesis
//! sobre conexiones: lo explica el **orden**. El escritor medía primero, con
//! todo caliente de la importación; la segunda conexión medía después de que
//! la lectora hubiera recorrido la tabla tres veces, así que heredaba su
//! calor; y el checkpoint reescribe el archivo principal, enfriando a todos.
//!
//! Medir dos caminos en bloques no los compara: compara **el primero con el
//! segundo**. Así que las rondas se **intercalan** —A, B, C, A, B, C…— y
//! cualquier calentamiento les toca a los tres por igual. Es la misma lección
//! que la corrida larga del motor (auditoría C), que compara el último cuarto
//! con el **segundo** y no con el primero para no meter dentro el arranque de
//! la caché.

use std::time::Instant;

/// Lo que tarda un camino, con su nombre.
#[derive(Debug)]
pub struct Tiempo {
    pub nombre: &'static str,
    pub ms: Vec<f64>,
}

impl Tiempo {
    fn mediana(&self) -> f64 {
        let mut v = self.ms.clone();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }

    fn escrito(&self) -> String {
        self.ms
            .iter()
            .map(|m| format!("{m:.1}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// El resultado entero, con el veredicto que las cifras sostienen.
#[derive(Debug)]
pub struct Corrida {
    pub contactos: u32,
    pub wal_antes: u64,
    pub wal_despues: u64,
    pub tiempos: Vec<Tiempo>,
}

impl std::fmt::Display for Corrida {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\nPor qué la lectora es más lenta · {} contactos\n",
            self.contactos
        )?;
        writeln!(
            f,
            "El `-wal` tras importar: {:.1} MB. Tras el checkpoint: {:.1} MB.\n",
            self.wal_antes as f64 / 1_048_576.0,
            self.wal_despues as f64 / 1_048_576.0
        )?;
        writeln!(f, "| Camino | Tres corridas (ms) |")?;
        writeln!(f, "|---|---|")?;
        for t in &self.tiempos {
            writeln!(f, "| {} | {} |", t.nombre, t.escrito())?;
        }

        // ── EL VEREDICTO SE NIEGA A CONCLUIR, Y ESO ES EL ARREGLO ─────
        //
        // La primera versión de este bloque imprimía «ES `query_only`» y «ES
        // el `-wal`» calculando proporciones de una sola corrida. Dijo las dos
        // cosas, y las dos resultaron falsas al repetir el experimento: el
        // signo de la diferencia cambió cinco veces.
        //
        // **Un veredicto que una sola corrida puede dar es un veredicto que no
        // vale**, y escribirlo en la salida lo convierte en una cita. Así que
        // ahora la salida enseña las proporciones y dice **qué haría falta para
        // concluir**, en vez de concluir.
        let busca = |n: &str| {
            self.tiempos
                .iter()
                .find(|t| t.nombre == n)
                .map(Tiempo::mediana)
        };
        writeln!(f, "\nLas proporciones de ESTA corrida:\n")?;
        if let (Some(esc), Some(lec), Some(sin)) = (
            busca("escritor · con el -wal lleno"),
            busca("LECTORA  · con el -wal lleno"),
            busca("2.ª conexión sin query_only, -wal lleno"),
        ) {
            writeln!(
                f,
                "  la lectora contra el escritor         {:.1}×",
                lec / esc.max(0.01)
            )?;
            writeln!(
                f,
                "  la 2.ª sin query_only contra él       {:.1}×",
                sin / esc.max(0.01)
            )?;
        }
        if let (Some(l1), Some(e1), Some(l2)) = (
            busca("SOLO la lectora, nueve seguidas · PRIMERA"),
            busca("SOLO el escritor, nueve seguidas · DESPUÉS"),
            busca("SOLO la lectora otra vez · ÚLTIMA"),
        ) {
            writeln!(
                f,
                "  aisladas: lectora {l1:.0} → escritor {e1:.0} → lectora {l2:.0} ms"
            )?;
        }
        writeln!(
            f,
            "\n  ── NO SE CONCLUYE NADA DE UNA CORRIDA ──\n\
             \n  El 03/11/2026 se publicó «la lectora es 12× más lenta» de una\n\
             \x20 corrida como ésta, y cinco repeticiones dieron el signo en los dos\n\
             \x20 sentidos: 12× más lenta, 10× más rápida, 11× más rápida, 11× más\n\
             \x20 rápida y 11× más lenta. El signo cambió al añadir dos count(*)\n\
             \x20 antes de cronometrar.\n\
             \n  Para concluir algo de este experimento hace falta correrlo varias\n\
             \x20 veces cambiando lo que NO debería importar —el orden, qué toca la\n\
             \x20 tabla primero, el tamaño— y ver que la proporción se mantiene. En\n\
             \x20 este contenedor no se mantiene: lo que domina es la cache.\n\
             \n  Las cinco corridas estan en PRESUPUESTO_RENDIMIENTO.md §8."
        )?;
        Ok(())
    }
}

/// La consulta que se mide: la del filtro de contactos por texto, que es la
/// que destapó el problema. Recorre la tabla porque no encuentra nada.
const CONSULTA: &str = "SELECT c.id FROM contact c
  WHERE c.company_id = ?1 AND c.deleted_at IS NULL
    AND (lower(COALESCE(c.first_name,'')) LIKE ?2 ESCAPE '\\'
      OR lower(COALESCE(c.last_name,''))  LIKE ?2 ESCAPE '\\'
      OR lower(COALESCE(c.source,''))     LIKE ?2 ESCAPE '\\')
  ORDER BY c.created_at DESC, c.id DESC LIMIT 100";

/// Cuántas rondas. **Múltiplo de tres**, porque el orden rota entre los tres
/// caminos y cada uno tiene que ocupar cada posición el mismo número de veces:
/// si no, se vuelve a medir la posición en vez de la conexión.
const RONDAS: usize = 9;

/// Mide los caminos **intercalados** sobre una base recién importada.
///
/// # Errores
///
/// Lo que falle al generar, abrir la base o importar.
pub fn correr(cuantas: usize, semilla: u64) -> Result<Corrida, Box<dyn std::error::Error>> {
    use arles_core::ids::CompanyId;
    use arles_db::{ClaveMaestra, Db};

    let dir = tempfile::tempdir()?;
    let csv = dir.path().join("sintetico.csv");
    eprintln!("generando {cuantas} filas…");
    crate::escribir_csv(&csv, cuantas, semilla)?;

    let ruta = dir.path().join("arles.db");
    let clave = ClaveMaestra::generar()?;
    let db = Db::abrir(&ruta, &clave)?;
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
               VALUES ('00000000-0000-4000-8000-000000000001', 'Banco', 'MX', 'America/Mexico_City',
                       'banco@ejemplo.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');",
        )
    })?;
    let empresa: CompanyId = "00000000-0000-4000-8000-000000000001"
        .parse()
        .map_err(|e| format!("id de empresa: {e:?}"))?;

    eprintln!("importando…");
    let tabla = arles_import::leer(&csv)?;
    crate::medir::importar_para_el_wal(&db, empresa, &tabla)?;

    let emp = empresa.to_string();
    let patron = "%zzqx%".to_owned();
    let wal = dir.path().join("arles.db-wal");
    let tamano = |p: &std::path::Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let wal_antes = tamano(&wal);
    eprintln!(
        "el -wal tras importar: {:.1} MB",
        wal_antes as f64 / 1_048_576.0
    );

    // La tercera conexión, sin `query_only`. Es `abrir`, la misma del
    // escritor, sobre el mismo archivo: lo único que la distingue de la
    // lectora es ese pragma.
    let segunda = arles_db::abrir(&ruta, &clave)?;

    let contar = |c: &rusqlite::Connection| -> Result<(), rusqlite::Error> {
        let mut q = c.prepare(CONSULTA)?;
        let _ = q
            .query_map(rusqlite::params![&emp, &patron], |_| Ok(()))?
            .count();
        Ok(())
    };
    let cronometrar = |f: &dyn Fn() -> Result<(), Box<dyn std::error::Error>>| -> Result<f64, Box<dyn std::error::Error>> {
        let t = Instant::now();
        f()?;
        Ok(t.elapsed().as_secs_f64() * 1000.0)
    };

    // ── ANTES DE CRONOMETRAR NADA: ¿LAS DOS VEN LOS MISMOS DATOS? ────────
    //
    // Una lectora que fuera rápida **porque no ve las filas** daría
    // exactamente las mismas cifras que una lectora rápida de verdad, y sería
    // un defecto de datos y no de rendimiento: la pantalla enseñaría una lista
    // vacía con todo en verde.
    //
    // No es paranoia: la lectora se abre en `Db::abrir`, **antes** de esta
    // importación, y en WAL un lector que retenga su transacción ve el
    // instantáneo de cuando la abrió. Que `leer` abra y cierre la suya en cada
    // llamada es justo lo que lo evita — y eso es una afirmación, así que se
    // comprueba.
    let filas_escritor: i64 =
        db.ejecutar_en_pruebas(|c| c.query_row("SELECT count(*) FROM contact", [], |f| f.get(0)))?;
    let filas_lectora: i64 =
        db.leer_en_pruebas(|c| c.query_row("SELECT count(*) FROM contact", [], |f| f.get(0)))?;
    eprintln!("filas que ve el escritor: {filas_escritor} · que ve la lectora: {filas_lectora}");
    if filas_escritor != filas_lectora {
        return Err(format!(
            "la lectora ve {filas_lectora} filas y el escritor {filas_escritor}: \
             cualquier cifra de rendimiento de abajo compararía dos tablas distintas"
        )
        .into());
    }

    // ── LAS TRES INTERCALADAS **Y ROTANDO EL ORDEN** ──────────────────────
    //
    // ── POR QUÉ ROTAR, Y NO SÓLO INTERCALAR ──
    //
    // Intercalar en orden fijo tampoco compara los caminos: compara las
    // **posiciones**. Medido con A, B, C fijo sobre 54 638 contactos, seis
    // rondas: el que iba segundo daba ~105 ms en las seis y los otros dos
    // ~1 100, fuese quien fuese. Lo que domina no es la conexión: es que el
    // recorrido anterior deja las páginas en la caché del sistema, y en este
    // contenedor no caben las tres.
    //
    // **Y por eso la primera versión de este experimento dio lo contrario que
    // la segunda** —la lectora 12× más lenta, y luego 10× más rápida—. Dos
    // corridas del mismo experimento que se contradicen no miden lo que dicen
    // medir.
    //
    // Rotando, cada camino ocupa cada posición el mismo número de veces, así
    // que el calentamiento se reparte por igual y lo que quede de diferencia
    // es de la conexión. Las rondas son múltiplo de tres por eso.
    let mut escritor = Vec::with_capacity(RONDAS);
    let mut lectora = Vec::with_capacity(RONDAS);
    let mut sin_pragma = Vec::with_capacity(RONDAS);
    for ronda in 0..RONDAS {
        let del_escritor = || {
            cronometrar(&|| {
                db.ejecutar_en_pruebas(contar)?;
                Ok(())
            })
        };
        let de_la_lectora = || {
            cronometrar(&|| {
                db.leer_en_pruebas(contar)?;
                Ok(())
            })
        };
        let de_la_segunda = || {
            cronometrar(&|| {
                contar(&segunda)?;
                Ok(())
            })
        };
        match ronda % 3 {
            0 => {
                escritor.push(del_escritor()?);
                lectora.push(de_la_lectora()?);
                sin_pragma.push(de_la_segunda()?);
            }
            1 => {
                lectora.push(de_la_lectora()?);
                sin_pragma.push(de_la_segunda()?);
                escritor.push(del_escritor()?);
            }
            _ => {
                sin_pragma.push(de_la_segunda()?);
                escritor.push(del_escritor()?);
                lectora.push(de_la_lectora()?);
            }
        }
    }

    // ── Y LAS MISMAS TRES TRAS VOLCAR EL `-wal` ───────────────────────────
    //
    // `TRUNCATE` deja el registro a cero. Se hace por el escritor, que es
    // quien puede.
    db.ejecutar_en_pruebas(|c| c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);"))?;
    let wal_despues = tamano(&wal);
    eprintln!(
        "el -wal tras el checkpoint: {:.1} MB",
        wal_despues as f64 / 1_048_576.0
    );

    // Y aquí también se alterna quién va primero, por el mismo motivo.
    let mut escritor2 = Vec::with_capacity(RONDAS);
    let mut lectora2 = Vec::with_capacity(RONDAS);
    for ronda in 0..RONDAS {
        let del_escritor = || {
            cronometrar(&|| {
                db.ejecutar_en_pruebas(contar)?;
                Ok(())
            })
        };
        let de_la_lectora = || {
            cronometrar(&|| {
                db.leer_en_pruebas(contar)?;
                Ok(())
            })
        };
        if ronda % 2 == 0 {
            escritor2.push(del_escritor()?);
            lectora2.push(de_la_lectora()?);
        } else {
            lectora2.push(de_la_lectora()?);
            escritor2.push(del_escritor()?);
        }
    }

    // ── Y LA PRUEBA LIMPIA: CADA UNA SOLA, SIN LA OTRA EN MEDIO ──────────
    //
    // Rotar reparte las posiciones, pero sigue habiendo una ronda donde la
    // lectora corre detrás del escritor. Nueve corridas **seguidas** de cada
    // una, sin la otra entre medias, es lo único que no tiene posición que
    // repartir. Si aquí se mantiene la diferencia, es de la conexión.
    // Y **en los dos órdenes**: la lectora primero y el escritor primero. Si al
    // invertirlo se invierte el resultado, lo que se mide sigue siendo la
    // posición. Si no, es la conexión — y es lo único que lo demuestra.
    let mut solo_lectora = Vec::with_capacity(RONDAS);
    for _ in 0..RONDAS {
        solo_lectora.push(cronometrar(&|| {
            db.leer_en_pruebas(contar)?;
            Ok(())
        })?);
    }
    let mut solo_escritor = Vec::with_capacity(RONDAS);
    for _ in 0..RONDAS {
        solo_escritor.push(cronometrar(&|| {
            db.ejecutar_en_pruebas(contar)?;
            Ok(())
        })?);
    }
    let mut lectora_despues = Vec::with_capacity(RONDAS);
    for _ in 0..RONDAS {
        lectora_despues.push(cronometrar(&|| {
            db.leer_en_pruebas(contar)?;
            Ok(())
        })?);
    }

    let contactos: u32 = db.ejecutar_en_pruebas(|c| {
        c.query_row(
            "SELECT count(*) FROM contact WHERE deleted_at IS NULL",
            [],
            |f| f.get(0),
        )
    })?;

    Ok(Corrida {
        contactos,
        wal_antes,
        wal_despues,
        tiempos: vec![
            Tiempo {
                nombre: "escritor · con el -wal lleno",
                ms: escritor,
            },
            Tiempo {
                nombre: "LECTORA  · con el -wal lleno",
                ms: lectora,
            },
            Tiempo {
                nombre: "2.ª conexión sin query_only, -wal lleno",
                ms: sin_pragma,
            },
            Tiempo {
                nombre: "escritor · tras el checkpoint",
                ms: escritor2,
            },
            Tiempo {
                nombre: "LECTORA  · tras el checkpoint",
                ms: lectora2,
            },
            Tiempo {
                nombre: "SOLO la lectora, nueve seguidas · PRIMERA",
                ms: solo_lectora,
            },
            Tiempo {
                nombre: "SOLO el escritor, nueve seguidas · DESPUÉS",
                ms: solo_escritor,
            },
            Tiempo {
                nombre: "SOLO la lectora otra vez · ÚLTIMA",
                ms: lectora_despues,
            },
        ],
    })
}
