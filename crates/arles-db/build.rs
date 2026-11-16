//! Hace que cambiar una migración recompile el crate.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE ARCHIVO EXISTE, Y QUÉ PASABA SIN ÉL
//!
//! `refinery::embed_migrations!("migrations")` es una macro: lee los `.sql` y
//! los incrusta en el binario **en tiempo de compilación**. Pero cargo no sabe
//! que esos archivos son una dependencia, así que al cambiar uno **no
//! recompila nada**.
//!
//! El efecto es el peor posible: alguien edita una migración, corre
//! `cargo test`, todo pasa en verde… y lo que se ha ejecutado es el SQL
//! anterior. La prueba no comprueba el cambio: comprueba lo de antes.
//!
//! Se descubrió el 06/10/2026 al intentar verificar la V11 rompiéndola a
//! propósito (§2): se quitó un índice, la prueba que lo exige **siguió
//! pasando**, y no era que la prueba fuera mala — era que el índice quitado
//! nunca llegó al binario.
//!
//! `rerun-if-changed` sobre la carpeta entera lo cierra: cualquier `.sql` que
//! se añada, cambie o borre fuerza la recompilación.

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
