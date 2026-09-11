//! Estado compartido de la aplicación.

use arles_db::Db;

use crate::error::AppError;
use crate::llavero;
use crate::rutas::Rutas;

/// Lo que los comandos comparten.
///
/// Nótese que **no aparece `rusqlite` por ninguna parte**: el shell habla con
/// `arles-db` mediante métodos tipados, no con el motor. Es lo que mantiene
/// acotadas las consultas y hace que la capa de datos sea auditable (ADR-0002).
#[derive(Debug)]
pub struct EstadoApp {
    db: Db,
}

impl EstadoApp {
    /// Arranca: resuelve rutas, obtiene la clave del llavero, abre y migra.
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén del sistema no responde.
    /// **La aplicación no arranca en ese caso**: seguir sin poder descifrar la
    /// base significaría funcionar sin los datos del usuario (ADR-0011).
    pub fn arrancar() -> Result<Self, AppError> {
        Self::arrancar_en(&Rutas::resolver()?)
    }

    /// Igual que [`Self::arrancar`], con rutas explícitas. Para tests.
    ///
    /// # Errores
    ///
    /// Ver [`Self::arrancar`].
    pub fn arrancar_en(rutas: &Rutas) -> Result<Self, AppError> {
        let clave = llavero::obtener_o_crear_clave_maestra()?;
        let db = Db::abrir(&rutas.base_de_datos(), &clave)?;
        Ok(Self { db })
    }

    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }
}
