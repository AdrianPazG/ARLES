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

    /// El llavero de una cuenta de correo concreta.
    ///
    /// Cada cuenta tiene su propia entrada, nombrada por su identificador. No
    /// se comparte una entrada entre cuentas ni se guardan varias contraseñas
    /// juntas: borrar una cuenta tiene que poder borrar **su** secreto sin
    /// tocar los de las demás, y una entrada compartida obligaría a leer,
    /// modificar y reescribir el conjunto — con dos ARLES abiertos, eso pierde
    /// contraseñas.
    #[must_use]
    pub fn de_la_cuenta(id: &str) -> Self {
        Self {
            cuenta: format!("email-account/{id}"),
        }
    }

    /// El llavero de un número de WhatsApp.
    ///
    /// Un espacio de nombres distinto del de correo —`whatsapp-number/`— y no
    /// el mismo con otro identificador. Los dos tipos de cuenta usan UUID, así
    /// que compartir prefijo haría que un choque de identificadores entregara
    /// el secreto equivocado al adaptador equivocado: el token de Meta a SMTP,
    /// o al revés. Improbable, y gratis de impedir.
    #[must_use]
    pub fn del_numero(id: &str) -> Self {
        Self {
            cuenta: format!("whatsapp-number/{id}"),
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

    /// Guarda la contraseña de una cuenta de correo.
    ///
    /// **Es la única forma de que una contraseña de correo entre en ARLES.** No
    /// hay un camino que la escriba en la base de datos, ni en un archivo de
    /// configuración, ni en una preferencia: `DatosDeLaCuenta` no tiene un campo
    /// donde quepa (ADR-0011).
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén no responde. **No hay
    /// respaldo a un archivo**: si el llavero no está, la contraseña no se
    /// guarda y se dice. Guardarla «mientras tanto» en texto plano es cómo se
    /// acaba con la contraseña de correo de una empresa en un JSON que nadie
    /// recuerda.
    pub fn guardar_contrasena(&self, contrasena: &str) -> Result<(), AppError> {
        self.entrada()?
            .set_password(contrasena)
            .map_err(|e| AppError::LlaveroNoDisponible(e.to_string()))
    }

    /// Lee la contraseña de una cuenta. `None` si no hay entrada.
    ///
    /// **Este valor no cruza la frontera de la IPC.** Lo usa el adaptador de
    /// correo en el momento de conectar y no sale de Rust: ningún comando lo
    /// devuelve (regla 3 de la frontera, `ARQUITECTURA.md` §5).
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén no responde.
    pub fn leer_contrasena(&self) -> Result<Option<String>, AppError> {
        match self.entrada()?.get_password() {
            Ok(c) => Ok(Some(c)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::LlaveroNoDisponible(e.to_string())),
        }
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

/// Referencia opaca que se guarda en `whatsapp_account.credential_ref`.
///
/// Misma regla que la de correo: en la base va la referencia, nunca el token
/// de Meta.
#[must_use]
pub fn referencia_de_numero(id_numero: &str) -> String {
    format!("llavero://{SERVICIO}/whatsapp-number/{id_numero}")
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

    /// **Cada cuenta de correo tiene su propia entrada en el llavero.**
    ///
    /// Con una entrada compartida, borrar una cuenta obligaría a leer, quitar
    /// una contraseña y reescribir el resto — y con dos ARLES abiertos eso
    /// pierde contraseñas de cuentas que nadie tocó.
    #[test]
    fn cada_cuenta_de_correo_tiene_su_propia_entrada() {
        let a = Llavero::de_la_cuenta("01900000-0000-7000-8000-00000000000a");
        let b = Llavero::de_la_cuenta("01900000-0000-7000-8000-00000000000b");
        assert_ne!(a.cuenta, b.cuenta);
        assert!(a.cuenta.starts_with("email-account/"));
        // Y ninguna se pisa con la de la clave maestra, que dejaría la base
        // irrecuperable al borrar una cuenta.
        assert_ne!(a.cuenta, CUENTA_CLAVE_MAESTRA);
        assert_ne!(b.cuenta, CUENTA_CLAVE_MAESTRA);
    }

    /// La referencia que se guarda en la base **nombra la misma entrada** que
    /// se usa para leer la contraseña.
    ///
    /// Si se separaran —porque alguien cambie una de las dos y no la otra—, la
    /// base apuntaría a un sitio del llavero donde no hay nada, y el síntoma
    /// sería «la contraseña no sirve» en una cuenta perfectamente configurada.
    #[test]
    fn la_referencia_guardada_y_la_entrada_del_llavero_no_se_separan() {
        let id = "01900000-0000-7000-8000-00000000000a";
        let referencia = referencia_de_cuenta(id);
        let entrada = Llavero::de_la_cuenta(id).cuenta;
        assert!(
            referencia.ends_with(&entrada),
            "la referencia «{referencia}» tiene que terminar en la entrada «{entrada}»"
        );
    }

    #[test]
    fn dos_llaveros_con_cuentas_distintas_no_se_pisan() {
        let a = Llavero::con_cuenta("prueba-a");
        let b = Llavero::con_cuenta("prueba-b");
        assert_ne!(a.cuenta, b.cuenta);
    }
}
