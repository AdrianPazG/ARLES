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
    let cuantas: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(500_000);
    let semilla: u64 = std::env::args()
        .nth(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or(20_261_006);

    match arles_banco::medir::correr(cuantas, semilla) {
        Ok(informe) => print!("{informe}"),
        Err(e) => {
            eprintln!("el banco no pudo terminar: {e}");
            std::process::exit(1);
        }
    }
}
