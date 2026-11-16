//! D-19 medido: ¿cuesta el total exacto, o cuesta el recorrido? (11/11/2026)
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA PREGUNTA QUE ESTO CONTESTA, Y POR QUÉ NO SE PODÍA CONTESTAR LEYENDO
//!
//! D-19 llegó de C-2 planteada así: «el único cambio que baja el tiempo del
//! filtro es dejar de contar todas las coincidencias», y de ahí una decisión de
//! producto —renunciar al total exacto, que es D-7C—.
//!
//! La hipótesis con la que se escribió esto era otra, y **la medición la
//! refutó a medias**, que es la razón de medir. Las dos cifras de C-2 —una
//! coincidencia y 119 532 tardando lo mismo— hacían pensar que lo caro era el
//! recorrido y no el total, y por tanto que el total era casi gratis. Lo que
//! hay es más interesante.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE SALIÓ · 120 000 sucesos, tres corridas (11/11/2026)
//!
//! | Camino | 0 coincidencias | 120 000 coincidencias |
//! |---|---|---|
//! | `count(*)` de todo, el de hoy | 4 054 · 3 858 · 3 898 ms | 3 589 · 4 535 · 3 988 ms |
//! | traer la página (`LIMIT 500`) | 3 950 · 3 863 · 3 819 ms | **20 · 24 · 26 ms** |
//! | la página y una más (`LIMIT 501`) | 3 872 · 4 025 · 3 863 ms | **20 · 24 · 26 ms** |
//! | `count` acotado a 501 | 3 898 · 3 873 · 3 885 ms | **13 · 16 · 17 ms** |
//!
//! **Tres cosas, y ninguna es la que se esperaba:**
//!
//! 1. **Cuando el filtro encuentra, el total exacto es ~99 % del coste.** La
//!    página son 20-26 ms y el `count(*)` 3 589-4 535: no «cuesta otro tanto»
//!    como decía C-2, cuesta **unas 150 veces** lo que la página.
//! 2. **El recuento acotado lo arregla ahí**: 13-17 ms, unas 250× más rápido,
//!    y conserva lo único que el usuario necesita saber — que hay más de 500.
//! 3. **Cuando el filtro NO encuentra nada, no hay nada que optimizar.** Los
//!    cuatro caminos cuestan lo mismo, porque para afirmar que no hay ninguna
//!    coincidencia hay que visitar todas las filas. **D-19 no toca este caso**,
//!    y es el que el usuario más pisa mientras escribe: es lo que R-26 ya midió
//!    para el filtro de contactos —«el texto cuesta lo que NO encuentra»—.
//!
//! Así que D-19 no es «¿sacrificamos el total?»: es **«acotamos el total»**, y
//! resuelve la mitad del problema. La otra mitad es otra cosa y no se arregla
//! aquí.
//!
//! **Lo que estas cifras NO son:** las de C-2. Aquélla medía 451 196 sucesos y
//! daba ~1 200 ms donde esto da ~3 900 con menos de un tercio de las filas. No
//! se inventa la causa —puede ser el contenedor, la caché o que esta siembra no
//! aproveche los índices igual—: lo que se publica es **la proporción entre los
//! cuatro caminos de la misma corrida**, que es lo que la decisión necesita y
//! lo único que estas tres corridas sostienen.
//!
//! **Esto no es una prueba de regresión: es un experimento**, como
//! `banco lectora`. Se queda en el repositorio para que quien retome D-19 no
//! tenga que creerse esta cabecera. Se corre con:
//!
//! ```bash
//! cargo test -p arles-db d19 -- --ignored --nocapture
//! ```
//!
//! Vive en `src/` y no en `tests/` porque mide `Db::con`, que es `pub(crate)`:
//! un test de integración no la ve. Es el mismo caso que
//! `presupuesto_del_respaldo`, y como él **no entra en el binario**.

use std::time::Instant;

use crate::{ClaveMaestra, Db};

use arles_core::empresa::{BorradorDeEmpresa, DatosDeEmpresa};

/// Cuántos sucesos se siembran. No son los 451 196 de C-2: lo que se mide aquí
/// es una **forma** —si el coste sigue al recorrido o a las coincidencias— y
/// esa forma se ve igual con menos filas y en un minuto en vez de en veinte.
const SUCESOS: usize = 120_000;

/// El tope de página del registro (D-7C).
const TOPE: usize = 500;

#[test]
#[ignore = "experimento: siembra 120 000 sucesos y cronometra. Se corre a mano"]
fn el_total_cuesta_un_recorrido_y_no_las_coincidencias() {
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
        .expect("guarda");

    sembrar(&db, &empresa.to_string(), SUCESOS);

    println!("\n  D-19 · ¿el total o el recorrido? · {SUCESOS} sucesos\n");
    println!("  {:<46} {:>10}  resultado", "camino", "ms");
    println!("  {}", "-".repeat(72));

    // Dos filtros con el MISMO coste de recorrido y un número de coincidencias
    // muy distinto. Es lo único que distingue las dos hipótesis.
    let raro = "%zzz-no-existe-nadie-asi%";
    let comun = "%@ejemplo.mx%";

    for (nombre, patron) in [
        ("una aguja (0 coincidencias)", raro),
        ("un dominio (todas)", comun),
    ] {
        println!("  · {nombre}");

        let (ms, total) = cronometrar(&db, &empresa.to_string(), patron, Medida::Total);
        println!(
            "  {:<46} {ms:>10.0}  {total}",
            "    count(*) de todo, el de hoy"
        );

        let (ms, filas) = cronometrar(&db, &empresa.to_string(), patron, Medida::Pagina);
        println!(
            "  {:<46} {ms:>10.0}  {filas} filas",
            "    traer la página (LIMIT 500)"
        );

        let (ms, filas) = cronometrar(&db, &empresa.to_string(), patron, Medida::PaginaYUna);
        println!(
            "  {:<46} {ms:>10.0}  {filas} filas",
            "    la página y UNA MÁS (LIMIT 501)"
        );

        let (ms, total) = cronometrar(&db, &empresa.to_string(), patron, Medida::TotalAcotado);
        println!(
            "  {:<46} {ms:>10.0}  {total}\n",
            "    count acotado a 501 (la idea obvia)"
        );
    }

    println!(
        "  Cómo se lee, con lo que salió el 11/11/2026 (ver la cabecera):\n\n  \
         · Con coincidencias, la página es gratis (20-26 ms) y el total exacto\n    \
           cuesta 150 veces más. Acotarlo a 501 lo deja en 13-17 ms.\n  \
         · Sin coincidencias, los cuatro caminos cuestan lo mismo: para decir\n    \
           «no hay nada» hay que visitar todas las filas. D-19 NO toca este\n    \
           caso, y es el que se pisa mientras se escribe en el campo.\n"
    );
}

enum Medida {
    /// El `count(*)` de hoy: recorre todas las coincidencias.
    Total,
    /// Traer la página, que es el otro recorrido de hoy.
    Pagina,
    /// La propuesta: una fila más que la página, en un solo recorrido.
    PaginaYUna,
    /// La idea obvia que **no** debería ayudar: acotar el recuento. Se mide
    /// para poder descartarla con una cifra en vez de con un argumento.
    TotalAcotado,
}

fn cronometrar(db: &Db, empresa: &str, patron: &str, que: Medida) -> (f64, String) {
    let empieza = Instant::now();
    let salida = db
        .con(|conn| {
            let de = "activity_event e
                      LEFT JOIN message_attempt a ON a.id = e.message_attempt_id";
            let donde = "e.company_id = ?1 AND a.contact_address LIKE ?2 ESCAPE '\\'";
            Ok(match que {
                Medida::Total => {
                    let n: i64 = conn.query_row(
                        &format!("SELECT count(*) FROM {de} WHERE {donde}"),
                        rusqlite::params![empresa, patron],
                        |r| r.get(0),
                    )?;
                    format!("total {n}")
                }
                Medida::TotalAcotado => {
                    let n: i64 = conn.query_row(
                        &format!(
                            "SELECT count(*) FROM (SELECT 1 FROM {de} WHERE {donde} LIMIT {})",
                            TOPE + 1
                        ),
                        rusqlite::params![empresa, patron],
                        |r| r.get(0),
                    )?;
                    format!("hasta {n}")
                }
                Medida::Pagina | Medida::PaginaYUna => {
                    let cuantas = if matches!(que, Medida::PaginaYUna) {
                        TOPE + 1
                    } else {
                        TOPE
                    };
                    let mut c = conn.prepare(&format!(
                        "SELECT e.id, a.contact_address FROM {de} WHERE {donde}
                          ORDER BY e.occurred_at DESC, e.rowid DESC LIMIT {cuantas}"
                    ))?;
                    let n = c
                        .query_map(rusqlite::params![empresa, patron], |_| Ok(()))?
                        .count();
                    n.to_string()
                }
            })
        })
        .expect("la consulta corre");
    (empieza.elapsed().as_secs_f64() * 1000.0, salida)
}

/// Siembra sucesos con su intento, que es donde vive la dirección (P-17).
///
/// En una transacción y con una sentencia preparada: sembrar 120 000 filas de
/// una en una con `execute` tarda minutos y lo que se mide después no cambia.
fn sembrar(db: &Db, empresa: &str, cuantos: usize) {
    let empieza = Instant::now();
    db.con(|conn| {
        conn.execute_batch("BEGIN")?;
        // Una campaña y una etapa de las que colgar los intentos. Los nombres
        // de columna salen del esquema, no de la memoria: una columna
        // inventada aquí daría un error dos pasos después.
        conn.execute(
            "INSERT INTO campaign (id, company_id, name, created_at, updated_at)
             VALUES ('camp-1', ?1, 'Medición D-19', ?2, ?2)",
            rusqlite::params![empresa, "2026-11-11T00:00:00Z"],
        )?;

        let mut intento = conn.prepare(
            "INSERT INTO message_attempt
               (id, campaign_id, channel, contact_address, idempotency_key,
                state, attempt_count, created_at, updated_at)
             VALUES (?1, 'camp-1', 'email', ?2, ?3, 'sent', 1, ?4, ?4)",
        )?;
        let mut suceso = conn.prepare(
            "INSERT INTO activity_event
               (id, company_id, kind, is_outcome, campaign_id, channel,
                message_attempt_id, occurred_at)
             VALUES (?1, ?2, 'envio.aceptado', 1, 'camp-1', 'email', ?3, ?4)",
        )?;

        for i in 0..cuantos {
            let id_intento = format!("int-{i:07}");
            // Todas en el mismo dominio: así el filtro «común» coincide con
            // todas y el «raro» con ninguna, que es el par que distingue las
            // dos hipótesis.
            let direccion = format!("persona{i:07}@ejemplo.mx");
            let cuando = format!("2026-11-{:02}T{:02}:00:00Z", 1 + (i % 28), i % 24);
            intento.execute(rusqlite::params![
                id_intento,
                direccion,
                format!("idem-{i:07}"),
                cuando
            ])?;
            suceso.execute(rusqlite::params![
                format!("suc-{i:07}"),
                empresa,
                id_intento,
                cuando
            ])?;
        }
        drop(intento);
        drop(suceso);
        conn.execute_batch("COMMIT")?;
        Ok(())
    })
    .expect("siembra");
    println!(
        "\n  sembrados {cuantos} sucesos en {:.1} s",
        empieza.elapsed().as_secs_f64()
    );
}
