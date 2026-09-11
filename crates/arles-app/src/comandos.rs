//! Comandos Tauri: la única frontera entre la webview y el dominio.
//!
//! Cuatro reglas, de `documentacion/03-arquitectura/ARQUITECTURA.md` §5:
//!
//! 1. **Comandos gruesos, no finos.** Cada cruce de la frontera es una
//!    oportunidad de estado inconsistente.
//! 2. **Errores tipados, no cadenas** (§95).
//! 3. **Ningún comando devuelve un secreto** (regla de frontera 3.4).
//! 4. **Ninguna ruta del sistema de archivos cruza la frontera.**

use serde::Serialize;

use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Información de la aplicación para el pie y el diálogo «Acerca de».
///
/// ADR-0010: el numeral «I» y el número de versión nunca van en el mismo
/// renglón, así que se entregan por separado y la interfaz decide dónde ponerlos.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoApp {
    /// Nombre visible, sin numeral: «ARLES RELAY».
    pub nombre: String,
    /// Nombre comercial, con numeral: «ARLES RELAY I».
    pub nombre_comercial: String,
    pub version: String,
    pub atribucion: String,
}

#[tauri::command]
pub fn info_app() -> InfoApp {
    InfoApp {
        nombre: "ARLES RELAY".to_owned(),
        nombre_comercial: "ARLES RELAY I".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        atribucion: "Software desarrollado por TELEMETRY INSIGHT".to_owned(),
    }
}

/// Estado de salud del arranque, para la pantalla de inicio.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoArranque {
    /// La base de datos cifrada está abierta y migrada.
    pub base_de_datos_lista: bool,
    /// Versión del esquema aplicada.
    pub version_esquema: Option<i32>,
    /// Hay una empresa configurada (si no, toca el onboarding del §25).
    pub empresa_configurada: bool,
}

/// Comprueba el estado del arranque.
///
/// # Errores
///
/// [`ErrorIpc`] si la base de datos no responde. Nunca incluye rutas ni secretos.
#[tauri::command]
pub fn estado_arranque(estado: tauri::State<'_, EstadoApp>) -> Result<EstadoArranque, ErrorIpc> {
    let resumen = estado
        .db()
        .resumen_arranque()
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(EstadoArranque {
        base_de_datos_lista: true,
        version_esquema: resumen.version_esquema,
        empresa_configurada: resumen.empresas > 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR-0010: la interfaz no puede poner «I» junto al número porque los
    /// recibe en campos distintos. El tipo hace cumplir la regla.
    #[test]
    fn la_info_separa_el_numeral_de_la_version() {
        let i = info_app();
        assert_eq!(i.nombre, "ARLES RELAY");
        assert_eq!(i.nombre_comercial, "ARLES RELAY I");
        assert!(!i.nombre.contains(&i.version));
        assert!(!i.nombre_comercial.contains(&i.version));
    }

    #[test]
    fn la_version_coincide_con_la_del_paquete() {
        assert_eq!(info_app().version, "1.2.0");
    }

    /// Regla de frontera 3.4: si algún día alguien añade un campo con una
    /// credencial, este test lo detecta al serializar.
    #[test]
    fn la_info_no_lleva_secretos() {
        let json = serde_json::to_string(&info_app()).expect("serializa");
        for prohibido in ["password", "token", "secret", "key", "credencial"] {
            assert!(
                !json.to_lowercase().contains(prohibido),
                "el comando expone «{prohibido}» al frontend"
            );
        }
    }
}
