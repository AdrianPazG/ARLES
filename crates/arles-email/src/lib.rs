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
/// Preguntarle al DNS por SPF, DKIM y DMARC (7.5).
pub mod dns;
pub mod error;
pub mod prueba;
pub mod servidor;
pub mod smtp;
pub mod tls;

// El servidor de mentira que habla cifrado no entra en el programa que se
// entrega: `rustls` y `rcgen` —un motor de TLS y un generador de certificados—
// sólo se compilan con la bandera `test-util`, que va en las
// `dev-dependencies` de quien la use.
//
// Es público desde la entrega 5.5 porque `arles-app` necesita probar el envío
// de prueba **de punta a punta**, y eso exige un servidor que hable cifrado:
// el adaptador no tiene ninguna forma de no exigir TLS (§29), así que sin esto
// no hay manera de verlo mandar un correo entero.
#[cfg(any(test, feature = "test-util"))]
pub mod servidor_cifrado;

pub use codigos::{RespuestaSmtp, leer_linea, traducir};
pub use error::ErrorDeCorreo;
pub use prueba::Diagnostico;
pub use servidor::{Guion, ServidorDeMentira};
pub use smtp::{Contenido, Cuenta, ProveedorSmtp};
pub use tls::{Cifrado, Modo};
