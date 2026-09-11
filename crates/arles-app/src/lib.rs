//! Shell de escritorio de ARLES RELAY.
//!
//! Orquesta el arranque y expone los comandos Tauri. **Toda la autoridad de
//! negocio vive en los crates de dominio**: este crate conecta, no decide.

// §138: sin `unwrap()` ni `panic!()` indiscriminados en producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]

pub mod comandos;
pub mod error;
pub mod estado;
pub mod llavero;
pub mod rutas;

pub use error::{AppError, ErrorIpc};
pub use estado::EstadoApp;

/// Punto de entrada de la aplicación de escritorio.
///
/// # Pánico
///
/// Si el arranque falla —típicamente porque el llavero del sistema no está
/// disponible— la aplicación **se detiene con un mensaje explicativo**. Es
/// deliberado: arrancar sin poder descifrar la base de datos significaría
/// funcionar sin los datos del usuario, y eso es peor que no arrancar
/// (ADR-0011).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let estado = match EstadoApp::arrancar() {
        Ok(e) => e,
        Err(e) => {
            // §95: qué pasó, cómo arreglarlo, qué está a salvo.
            eprintln!("ARLES RELAY no pudo arrancar.");
            eprintln!();
            eprintln!("Qué pasó: {e}");
            eprintln!(
                "Qué hacer: verifica que el almacén de credenciales de tu sistema \
                 esté disponible y desbloqueado."
            );
            eprintln!("Tus datos están intactos: la base de datos no se ha modificado.");
            std::process::exit(1);
        }
    };

    let resultado = tauri::Builder::default()
        .manage(estado)
        .invoke_handler(tauri::generate_handler![
            comandos::info_app,
            comandos::estado_arranque,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = resultado {
        eprintln!("ARLES RELAY terminó de forma inesperada: {e}");
        std::process::exit(1);
    }
}
