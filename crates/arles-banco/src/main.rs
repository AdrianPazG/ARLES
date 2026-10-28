//! El banco: genera el conjunto, mide, y escribe lo que midió.
//!
//! Se corre a mano, no en cada `cargo test`: medio millón de filas tardan
//! minutos y ocupan disco. El paso de CI lo invoca con una cifra más pequeña.
//!
//! ```text
//! cargo run --release -p arles-banco -- 500000
//! ```
//!
//! **Lo que no se puede medir aquí se dice sin medir**, que no es lo mismo que
//! cumplido. Es la regla que el validador ya aplica a las sondas omitidas.

fn main() {
    // `banco larga 100000` mide la corrida larga del motor en vez de las trece
    // cifras. Va aparte porque mide una **forma** —si la memoria se
    // estabiliza— y no un número, y porque tarda lo suyo.
    if std::env::args().nth(1).as_deref() == Some("larga") {
        let pasos: u32 = std::env::args()
            .nth(2)
            .and_then(|a| a.parse().ok())
            .unwrap_or(100_000);
        // `banco larga 20000 fuga` mete una fuga de verdad —un proveedor que
        // guarda cada pedido— para poder ver la comprobación en rojo. Sin
        // esto, «la memoria no crece» sería una afirmación que nadie ha visto
        // caer, que es lo que el §2 no admite.
        let con_fuga = std::env::args().nth(3).as_deref() == Some("fuga");
        match arles_banco::larga::correr(pasos, con_fuga) {
            Ok(c) => {
                let n = c.muestras.len();
                let cuarto = n / 4;
                println!(
                    "pasos: {} ({} enviados) en {:.1} s\nmuestras: {n}\n\
                     primer cuarto:  {:.1} MB\nsegundo cuarto: {:.1} MB\n\
                     tercer cuarto:  {:.1} MB\nultimo cuarto:  {:.1} MB\n\
                     crecimiento (ultimo - segundo): {:.2} MB",
                    c.pasos,
                    c.enviados,
                    c.tardo.as_secs_f64(),
                    c.tramo(0, cuarto) as f64 / 1_048_576.0,
                    c.tramo(cuarto, cuarto * 2) as f64 / 1_048_576.0,
                    c.tramo(cuarto * 2, cuarto * 3) as f64 / 1_048_576.0,
                    c.tramo(cuarto * 3, n) as f64 / 1_048_576.0,
                    c.crecimiento() as f64 / 1_048_576.0,
                );
            }
            Err(e) => {
                eprintln!("la corrida larga no pudo terminar: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // `banco wal 500000` mide cuánto crece el `-wal` con un lector abierto
    // durante una importación: la incógnita 1 de ADR-0015. Va aparte porque
    // corre la importación TRES VECES —sin lector, con lecturas de una en una
    // y con una transacción abierta— y sin la de referencia no se sabría
    // cuánto de lo que crece es por el lector.
    if std::env::args().nth(1).as_deref() == Some("wal") {
        let cuantas: usize = std::env::args()
            .nth(2)
            .and_then(|a| a.parse().ok())
            .unwrap_or(500_000);
        match arles_banco::wal::correr_los_tres(cuantas, 20_261_028) {
            Ok(tabla) => print!("{tabla}"),
            Err(e) => {
                eprintln!("la medición del wal no pudo terminar: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    let cuantas: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(500_000);
    let semilla: u64 = std::env::args()
        .nth(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or(20_261_006);

    match arles_banco::medir::correr(cuantas, semilla) {
        Ok(corrida) => {
            print!("{}", corrida.informe);
            // ── UNA MEDICIÓN ROTA SALE CON CÓDIGO DISTINTO DE CERO ───────
            //
            // Un presupuesto que **no se cumple** es un resultado y sale con
            // cero: es información, no un fallo del banco. Un presupuesto que
            // el banco **no pudo medir porque reventó** es otra cosa, y hasta
            // el 28/10/2026 salía igual: con cero y con un «SIN MEDIR» que se
            // lee como «todavía no toca». Así estuvo seis días sin medirse la
            // peor cifra del producto.
            if corrida.se_rompio {
                eprintln!(
                    "\nEL BANCO SE ROMPIÓ EN AL MENOS UNA MEDICIÓN. \
                     Busca «BANCO ROTO» en el informe: la cifra que falta no \
                     es una limitación declarada, es un instrumento averiado."
                );
                std::process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("el banco no pudo terminar: {e}");
            std::process::exit(1);
        }
    }
}
