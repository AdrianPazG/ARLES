//! Estado compartido de la aplicación.

use arles_db::Db;

use crate::error::AppError;
use crate::llavero::Llavero;
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
        Self::arrancar_con(&Rutas::resolver()?, &Llavero::del_sistema())
    }

    /// Igual que [`Self::arrancar`], con rutas y llavero explícitos.
    ///
    /// Los tests inyectan un llavero con cuenta propia: comparten el almacén
    /// real de la máquina, y con una cuenta fija un test que borra la entrada
    /// rompe a otro que corre en paralelo.
    ///
    /// # Errores
    ///
    /// Ver [`Self::arrancar`].
    pub fn arrancar_con(rutas: &Rutas, llavero: &Llavero) -> Result<Self, AppError> {
        let base = rutas.base_de_datos();

        let clave = match llavero.leer()? {
            Some(c) => c,

            // No hay clave en el llavero pero **sí hay una base en disco**.
            //
            // Generar una clave nueva aquí sería lo cómodo y lo peor posible: la
            // base existente quedaría irrecuperable y el usuario solo vería «no
            // se pudo descifrar», sin enterarse de que lo que necesita es
            // restaurar un respaldo. Se nombra la situación en vez de taparla
            // (riesgo R-10).
            None if base.exists() => return Err(AppError::ClaveMaestraPerdida),

            // Primer arranque de verdad: no hay clave y tampoco hay base.
            None => llavero.crear()?,
        };

        Ok(Self {
            db: Db::abrir(&base, &clave)?,
        })
    }

    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }
}
