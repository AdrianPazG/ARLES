//! El adaptador de correo de ARLES.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ HACE ESTE CRATE, Y QUÉ NO
//!
//! Hace **una sola cosa**: traducir. El motor de la Fase 4 sabe enviar y
//! entiende cinco fallos; un servidor de correo habla en números de tres
//! cifras y texto libre. Este crate es el puente, y nada más.
//!
//! **El motor no cambia.** Ésa era la razón de construirlo contra un proveedor
//! simulado: si al enchufar SMTP hubiera que tocar el bucle, la abstracción
//! estaría rota.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ HAY UN SERVIDOR DE MENTIRA AQUÍ DENTRO
//!
//! No se le puede pedir a Gmail que conteste `550`. Y probar el adaptador sólo
//! contra Gmail significa probarlo el día del lanzamiento, con los contactos de
//! un cliente.
//!
//! Así que [`servidor`] levanta un servidor de correo **en el propio proceso**,
//! que habla el protocolo de verdad y al que se le puede ordenar qué contestar
//! en cada paso de la conversación. Es la misma idea que el simulador de la
//! Fase 4, un piso más abajo: allí se simulaba «el proveedor», aquí «el
//! servidor», y el adaptador que hablará con Gmail es exactamente el mismo que
//! habla con éste.

// El §138 prohíbe `unwrap()` y `panic!()` indiscriminados. En los tests son
// legítimos; la denegación sigue activa en el código de producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod codigos;
pub mod servidor;

pub use codigos::{RespuestaSmtp, leer_linea, traducir};
pub use servidor::{Guion, ServidorDeMentira};
