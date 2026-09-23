//! El motor de ejecución de ARLES: quién manda los mensajes y con qué ritmo.
//!
//! Las **reglas** viven en `arles-core` —el ritmo, la ventana, los reintentos,
//! la máquina de estados— y **la cola** en `arles-db`. Aquí está lo que las une
//! y lo único que habla con el mundo exterior.
//!
//! La arquitectura lo prevé desde el discovery: `ARQUITECTURA.md` §2 lista
//! `arles-engine` y `arles-email` como crates propios, y da el motivo: con
//! módulos dentro de un solo crate, nada impide que un martes por la tarde el
//! código de contactos acabe importando el del motor. Con crates, el compilador
//! lo rechaza.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ EL MOTOR ES SÍNCRONO
//!
//! Lo esperable en Rust sería un motor asíncrono. Aquí no aporta y sí cuesta:
//!
//! **No hay nada que ganar.** ARLES manda cincuenta correos al día. El cuello
//! de botella es el límite que pone el proveedor, no el número de conexiones
//! que el programa pueda sostener a la vez. La asincronía resuelve un problema
//! que este producto no tiene.
//!
//! **Y sí hay algo que perder.** La base de datos ya es síncrona —`rusqlite`
//! detrás de un cerrojo—, así que un motor asíncrono estaría cruzando la
//! frontera en cada paso. Y un `trait` asíncrono convierte cada prueba del
//! simulador en algo que necesita un ejecutor para correr.
//!
//! El motor vive en su propio hilo, independiente de la ventana (§51). Cerrar
//! la ventana no detiene una campaña.

// El §138 prohíbe `unwrap()` y `panic!()` indiscriminados. En los tests son
// legítimos; la denegación sigue activa en el código de producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod bucle;
pub mod instancia;
pub mod proveedor;
pub mod simulador;
pub mod tiempo;

pub use bucle::{Ajustes, Entorno, EntornoDelSistema, Paso, Trabajador};
pub use instancia::{Cerrojo, NoSePudoTomar};
pub use proveedor::{Aceptado, Mensaje, Proveedor, RESPUESTA_MAXIMA, recortar_respuesta};
pub use simulador::{ProveedorSimulado, Respuesta};
pub use tiempo::sello;
