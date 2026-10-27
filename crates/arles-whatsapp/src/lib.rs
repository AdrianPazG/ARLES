//! El adaptador de WhatsApp Cloud API.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE CRATE HACE, Y LO QUE DELIBERADAMENTE NO
//!
//! Habla con la API de Meta y traduce lo que contesta a las cinco decisiones
//! que el motor entiende. **No sabe qué es una campaña**, igual que
//! `arles-email` no lo sabe: recibe un mensaje y lo entrega, o dice por qué no
//! pudo.
//!
//! Y **no compone texto**. No puede: el texto de una plantilla lo aprueba Meta
//! palabra por palabra, y lo que viaja por la API es su nombre más los valores
//! de sus huecos. Está explicado en `arles_core::whatsapp`.

#![forbid(unsafe_code)]

pub mod cloud;
pub mod codigos;
/// Preguntarle a Meta qué números tiene este token (D-WA).
pub mod descubrir;
pub mod entrante;
pub mod estado;
pub mod servidor;

pub use cloud::ProveedorDeWhatsApp;
pub use codigos::{ErrorDeMeta, traducir};
pub use descubrir::{NoSePudoPreguntar, NumeroDescubierto, numeros_del_token};
pub use entrante::{Aviso, AvisoRechazado, Respuesta};
pub use estado::{EstadoDelNumero, consultar};
pub use servidor::{Guion, ServidorDeMentira};
