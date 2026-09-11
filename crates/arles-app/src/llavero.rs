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

/// Obtiene la clave maestra, generándola en el primer arranque.
///
/// # Errores
///
/// [`AppError::LlaveroNoDisponible`] si el almacén del sistema no responde.
///
/// **La aplicación no arranca sin llavero.** No hay degradación a un archivo ni
/// modo compatibilidad: eso convertiría el cifrado en teatro, porque un atacante
/// con acceso al sistema de archivos obtendría la base de datos y su clave del
/// mismo directorio. Y el fallo sería silencioso (ADR-0011).
pub fn obtener_o_crear_clave_maestra() -> Result<ClaveMaestra, AppError> {
    let entrada = Entry::new(SERVICIO, CUENTA_CLAVE_MAESTRA)
        .map_err(|e| AppError::LlaveroNoDisponible(e.to_string()))?;

    match entrada.get_secret() {
        Ok(bytes) => ClaveMaestra::desde_bytes(&bytes).map_err(AppError::Db),

        Err(keyring::Error::NoEntry) => {
            // Primer arranque: se genera y se guarda.
            let clave = ClaveMaestra::generar().map_err(AppError::Db)?;
            entrada
                .set_secret(clave.exponer())
                .map_err(|e| AppError::LlaveroNoDisponible(e.to_string()))?;
            Ok(clave)
        }

        Err(e) => Err(AppError::LlaveroNoDisponible(e.to_string())),
    }
}

/// Borra la clave maestra del llavero.
///
/// Solo para desinstalación y para los tests. **Borrarla hace la base de datos
/// irrecuperable** (riesgo R-10).
pub fn borrar_clave_maestra() -> Result<(), AppError> {
    let entrada = Entry::new(SERVICIO, CUENTA_CLAVE_MAESTRA)
        .map_err(|e| AppError::LlaveroNoDisponible(e.to_string()))?;
    match entrada.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::LlaveroNoDisponible(e.to_string())),
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
    }
}
