//! Acceso a datos de ARLES RELAY.
//!
//! SQLite cifrado con SQLCipher (T-3), migraciones versionadas y repositorios
//! con consultas parametrizadas. Sin ORM: el SQL que se ejecuta se lee, que es
//! lo que permite auditarlo (ADR-0002).

// El §138 prohíbe `unwrap()` y `panic!()` indiscriminados. En los tests son
// legítimos; la denegación sigue activa en el código de producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
// SQLCipher se habla por FFI, pero la parte `unsafe` vive dentro de `rusqlite`,
// que la audita y la envuelve. Aquí no entra ninguna.
#![forbid(unsafe_code)]

pub mod conexion;
pub mod db;
pub mod empresa;
pub mod error;
pub mod migraciones;
pub mod preferencias;

pub use conexion::{ClaveMaestra, abrir};
pub use db::{Db, ResumenArranque};
pub use empresa::EmpresaGuardada;
pub use error::DbError;
pub use preferencias::CLAVES_DE_INTERFAZ;
