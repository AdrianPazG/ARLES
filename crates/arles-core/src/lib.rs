//! Tipos de dominio de ARLES RELAY.
//!
//! Este crate **no hace I/O**: ni base de datos, ni red, ni sistema de archivos.
//! Es la regla de frontera 3.2 de `documentacion/03-arquitectura/ARQUITECTURA.md`,
//! y lo que permite testear la lógica de dominio sin levantar nada.

// El §138 prohíbe `unwrap()` y `panic!()` indiscriminados, y el workspace los
// deniega. En los tests son legítimos: un test que no puede fallar ruidosamente
// no sirve de nada. La denegación sigue activa en el código de producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
// `forbid`, no `deny`: aquí no hay excepción posible ni la habrá. Este crate no
// toca el sistema operativo, así que un `unsafe` sería un error, nunca una
// necesidad. El workspace deniega; estas dos raíces prohíben.
#![forbid(unsafe_code)]

pub mod attempt;
pub mod email;
pub mod error;
pub mod ids;
pub mod secret;

pub use attempt::{AttemptState, TransitionError};
pub use email::EmailAddress;
pub use error::CoreError;
pub use secret::Secret;
