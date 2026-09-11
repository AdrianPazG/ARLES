//! Acceso a datos de ARLES RELAY.
//!
//! SQLite cifrado con SQLCipher (T-3), migraciones versionadas y repositorios
//! con consultas parametrizadas. Sin ORM: el SQL que se ejecuta se lee, que es
//! lo que permite auditarlo (ADR-0002).

// El §138 prohíbe `unwrap()` y `panic!()` indiscriminados. En los tests son
// legítimos; la denegación sigue activa en el código de producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]

pub mod conexion;
pub mod db;
pub mod error;
pub mod migraciones;

pub use conexion::{ClaveMaestra, abrir};
pub use db::{Db, ResumenArranque};
pub use error::DbError;
