//! Acceso al almacén de credenciales del sistema operativo.
//!
//! Keychain en macOS, Credential Manager en Windows. Ver ADR-0011 y
//! `documentacion/04-seguridad/MODELO_DE_SECRETOS.md`.

use arles_db::ClaveMaestra;
use keyring::Entry;

use crate::error::AppError;

/// Servicio bajo el que se agrupan las entradas de ARLES en el llavero.
const SERVICIO: &str = "mx.telemetryinsight.arlesrelay";

/// Entrada de la clave maestra de la base de datos.
const CUENTA_CLAVE_MAESTRA: &str = "db-master-key";

/// Acceso al almacén de credenciales.
///
/// La cuenta es un parámetro y no una constante para que los tests puedan
/// aislarse: comparten el llavero real de la máquina, y con un nombre fijo un
/// test que borra la entrada rompe a otro que corre en paralelo. El bug existió
/// y produjo resultados que parecían correctos.
#[derive(Debug, Clone)]
pub struct Llavero {
    cuenta: String,
}

impl Llavero {
    /// El llavero de la aplicación.
    #[must_use]
    pub fn del_sistema() -> Self {
        Self {
            cuenta: CUENTA_CLAVE_MAESTRA.to_owned(),
        }
    }

    /// Un llavero con una cuenta propia. Para tests que necesitan aislamiento.
    #[must_use]
    pub fn con_cuenta(cuenta: impl Into<String>) -> Self {
        Self {
            cuenta: cuenta.into(),
        }
    }

    fn entrada(&self) -> Result<Entry, AppError> {
        Entry::new(SERVICIO, &self.cuenta).map_err(|e| AppError::LlaveroNoDisponible(e.to_string()))
    }

    /// Lee la clave maestra. `None` significa que no hay entrada.
    ///
    /// **Leer y crear están separados a propósito.** Una única función
    /// «obtener-o-crear» no puede distinguir el primer arranque de una entrada
    /// perdida, y ante la duda generaría una clave nueva: la base existente
    /// quedaría irrecuperable y el usuario solo vería «no se pudo descifrar»,
    /// sin enterarse de que lo que necesita es restaurar un respaldo. Quien
    /// llama sí sabe si hay base en disco, así que la decisión le corresponde.
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén no responde.
    pub fn leer(&self) -> Result<Option<ClaveMaestra>, AppError> {
        match self.entrada()?.get_secret() {
            Ok(bytes) => ClaveMaestra::desde_bytes(&bytes)
                .map(Some)
                .map_err(AppError::Db),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::LlaveroNoDisponible(e.to_string())),
        }
    }

    /// Genera una clave maestra y la guarda. Solo para el primer arranque.
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si no se puede escribir.
    ///
    /// **La aplicación no arranca sin llavero.** No hay degradación a un archivo
    /// ni modo compatibilidad: eso convertiría el cifrado en teatro, porque un
    /// atacante con acceso al sistema de archivos obtendría la base de datos y
    /// su clave del mismo directorio. Y el fallo sería silencioso (ADR-0011).
    pub fn crear(&self) -> Result<ClaveMaestra, AppError> {
        let clave = ClaveMaestra::generar().map_err(AppError::Db)?;
        self.entrada()?
            .set_secret(clave.exponer())
            .map_err(|e| AppError::LlaveroNoDisponible(e.to_string()))?;
        Ok(clave)
    }

    /// Borra la clave maestra.
    ///
    /// Solo para desinstalación y para los tests. **Borrarla hace la base de
    /// datos irrecuperable** (riesgo R-10).
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén no responde.
    pub fn borrar(&self) -> Result<(), AppError> {
        match self.entrada()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::LlaveroNoDisponible(e.to_string())),
        }
    }
}

/// Referencia opaca que se guarda en `email_account.credential_ref`.
///
/// La base de datos **nunca** guarda una credencial: guarda esta referencia, y
/// el secreto se resuelve contra el llavero en el momento del uso.
#[must_use]
pub fn referencia_de_cuenta(id_cuenta: &str) -> String {
    format!("llavero://{SERVICIO}/email-account/{id_cuenta}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_referencia_no_contiene_el_secreto() {
        let r = referencia_de_cuenta("01900000-0000-7000-8000-00000000000a");
        assert!(r.starts_with("llavero://"));
        assert!(r.contains("01900000-0000-7000-8000-00000000000a"));
    }

    /// El nombre del servicio se usa como clave en el llavero del sistema:
    /// cambiarlo deja huérfanas las credenciales de las instalaciones ya
    /// existentes, y el usuario tendría que reconectar todas sus cuentas.
    #[test]
    fn el_identificador_del_servicio_es_estable() {
        assert_eq!(SERVICIO, "mx.telemetryinsight.arlesrelay");
        assert_eq!(CUENTA_CLAVE_MAESTRA, "db-master-key");
        assert_eq!(Llavero::del_sistema().cuenta, CUENTA_CLAVE_MAESTRA);
    }

    #[test]
    fn dos_llaveros_con_cuentas_distintas_no_se_pisan() {
        let a = Llavero::con_cuenta("prueba-a");
        let b = Llavero::con_cuenta("prueba-b");
        assert_ne!(a.cuenta, b.cuenta);
    }
}
