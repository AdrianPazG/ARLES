//! Estado compartido de la aplicación.

use std::sync::Mutex;

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
    /// El archivo que se está importando, entre que se lee y se confirma.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ SE QUEDA AQUÍ Y NO SE VUELVE A LEER
    ///
    /// Importar son tres pasos: leer, analizar y confirmar. Entre el segundo y
    /// el tercero el usuario está mirando el informe de choques, y eso puede
    /// durar minutos.
    ///
    /// Volver a abrir el archivo en cada paso dejaría una ventana en la que
    /// alguien —o el propio Excel, que reescribe al guardar— puede cambiarlo
    /// debajo. Entonces el usuario aprobaría un informe de un archivo y ARLES
    /// importaría otro, sin que nada fallara.
    ///
    /// Se guarda **uno solo**: empezar una importación descarta la anterior.
    /// Dos a la vez no tienen sentido —hay una sola pantalla— y guardarlas
    /// todas sería una fuga de memoria con nombre propio.
    /// ─────────────────────────────────────────────────────────────────────
    importacion: Mutex<Option<arles_import::TablaLeida>>,
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
            importacion: Mutex::new(None),
        })
    }

    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Guarda el archivo leído, descartando el anterior si lo hubiera.
    pub(crate) fn guardar_importacion(&self, tabla: arles_import::TablaLeida) {
        let mut hueco = self
            .importacion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *hueco = Some(tabla);
    }

    /// Hace algo con el archivo en curso, sin sacarlo del estado.
    ///
    /// Se pasa un cierre en vez de devolver una copia: el archivo puede tener
    /// medio millón de filas, y clonarlo en cada paso multiplicaría por tres la
    /// memoria de la importación.
    pub(crate) fn con_importacion<T>(
        &self,
        f: impl FnOnce(&arles_import::TablaLeida) -> T,
    ) -> Option<T> {
        let hueco = self
            .importacion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        hueco.as_ref().map(f)
    }

    /// Suelta el archivo en curso.
    ///
    /// Se llama al confirmar y al cancelar. Sin esto, medio millón de filas se
    /// quedarían en memoria hasta cerrar ARLES.
    pub(crate) fn soltar_importacion(&self) {
        let mut hueco = self
            .importacion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *hueco = None;
    }
}
