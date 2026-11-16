//! Mide qué contesta el DNS de ESTE equipo a los tres registros de un dominio.
//!
//! No es una prueba: es el instrumento con el que se declara, con fecha y
//! cifra, si el resolutor de verdad pudo preguntar aquí. Se corre a mano:
//!
//!     cargo run -p arles-email --example medir_dns -- telemetrymx.com titan1
fn main() {
    let mut args = std::env::args().skip(1);
    let dominio = args.next().unwrap_or_else(|| "telemetrymx.com".to_string());
    let selector = args.next();

    match arles_email::dns::ResolutorDelSistema::nuevo() {
        Err(e) => println!("no se pudo preparar el resolutor: {e}"),
        Ok(r) => {
            let v = arles_email::dns::revisar_dominio(&r, &dominio, selector.as_deref());
            println!("dominio: {dominio}");
            for rev in &v.revisiones {
                println!("  {:?}: {:?}", rev.registro, rev.hallazgo);
            }
            for s in &v.sin_comprobar {
                println!("  {s:?}: SIN COMPROBAR");
            }
            println!("veredicto para el preflight: {:?}", v.en_orden());
        }
    }
}
