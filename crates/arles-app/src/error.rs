//! Errores del shell, y su forma al cruzar la frontera IPC.

use serde::Serialize;
use thiserror::Error;

/// Error de arranque o de un comando.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    /// El almacén de credenciales del sistema no responde.
    ///
    /// **La aplicación no arranca.** Ver ADR-0011.
    #[error("el almacén de credenciales del sistema no está disponible: {0}")]
    LlaveroNoDisponible(String),

    /// Hay una base de datos en disco pero su clave ya no está en el llavero.
    ///
    /// Ocurre al reinstalar el sistema, cambiar de equipo o corromperse el
    /// perfil de usuario. **La base es irrecuperable sin esa clave** — es la
    /// propiedad que se buscaba al cifrarla (riesgo R-10).
    ///
    /// Se distingue de [`Self::LlaveroNoDisponible`] porque la acción del
    /// usuario es completamente distinta: allí hay que desbloquear el llavero;
    /// aquí hay que restaurar un respaldo.
    #[error(
        "la clave de la base de datos ya no está en el almacén de credenciales \
         del sistema"
    )]
    ClaveMaestraPerdida,

    #[error("no se pudo determinar el directorio de datos de la aplicación")]
    DirectorioDeDatos,

    /// No se pudo lanzar el navegador del sistema.
    ///
    /// No es grave —el usuario puede escribir la dirección— pero tiene que
    /// decirse: un enlace que no hace nada al pulsarlo se lee como una avería
    /// de la aplicación entera.
    #[error("no se pudo abrir el navegador del sistema: {0}")]
    SitioNoAbre(String),

    /// El formulario de empresa trae campos que no pasan la validación.
    ///
    /// Lleva **la lista de campos**, no un mensaje: la interfaz tiene que
    /// señalar cada campo malo. Un formulario que dice «hay un error» sin decir
    /// dónde obliga a revisarlo entero (§95).
    #[error("los datos de la empresa no son válidos")]
    EmpresaInvalida(Vec<arles_core::ErrorDeCampo>),

    #[error(transparent)]
    Db(#[from] arles_db::DbError),

    #[error(transparent)]
    Core(#[from] arles_core::CoreError),
}

impl AppError {
    /// Clave estable para que la interfaz traduzca el error con i18n (§139).
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::LlaveroNoDisponible(_) => "error.app.llavero_no_disponible",
            Self::ClaveMaestraPerdida => "error.app.clave_maestra_perdida",
            Self::DirectorioDeDatos => "error.app.directorio_de_datos",
            Self::SitioNoAbre(_) => "error.app.sitio_no_abre",
            Self::EmpresaInvalida(_) => "error.app.empresa_invalida",
            Self::Db(e) => e.clave_i18n(),
            Self::Core(e) => e.clave_i18n(),
        }
    }
}

/// Forma en que un error cruza la frontera IPC.
///
/// **Tipado, no una cadena.** El §95 exige que cada error diga qué pasó, cómo
/// arreglarlo y qué está a salvo, y eso es imposible de construir en la interfaz
/// a partir de un `Result<T, String>`: el frontend necesita distinguir variantes
/// para elegir el texto localizado correcto.
///
/// El campo `detalle` es para diagnóstico y **nunca** contiene secretos ni rutas
/// del sistema de archivos (THREAT_MODEL.md §4.1).
#[derive(Debug, Serialize)]
pub struct ErrorIpc {
    /// Clave de i18n: la interfaz resuelve con ella el texto de tres partes.
    pub clave: String,
    /// Texto técnico, para la bitácora de diagnóstico. No se muestra tal cual.
    pub detalle: String,
    /// Campos concretos que fallaron, cuando el error es de un formulario.
    ///
    /// Vacío en todo lo demás. Es lo que permite a la interfaz marcar el campo
    /// en vez de mostrar un aviso general que obliga a revisarlo todo.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub campos: Vec<arles_core::ErrorDeCampo>,
}

impl From<AppError> for ErrorIpc {
    fn from(e: AppError) -> Self {
        let campos = match &e {
            AppError::EmpresaInvalida(c) => c.clone(),
            _ => Vec::new(),
        };
        Self {
            clave: e.clave_i18n().to_owned(),
            detalle: redactar_rutas(&e.to_string()),
            campos,
        }
    }
}

/// Sustituye cualquier fragmento con pinta de ruta por `[ruta]`.
///
/// `AppError` envuelve errores de `rusqlite` y del llavero, y esos **sí**
/// incluyen rutas del sistema de archivos en su mensaje. Sin esto, el campo
/// contradecía su propia documentación en cuanto el error venía de una capa
/// inferior, y el test que lo vigilaba solo cubría las dos variantes que nunca
/// podían contener una (THREAT_MODEL.md §4.1).
fn redactar_rutas(mensaje: &str) -> String {
    mensaje
        .split(' ')
        .map(|palabra| {
            let limpia = palabra.trim_matches(|c: char| !c.is_alphanumeric());
            if limpia.contains('/') || limpia.contains('\\') || limpia.contains(":\\") {
                "[ruta]"
            } else {
                palabra
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toda_variante_tiene_clave_i18n() {
        let casos = [
            AppError::LlaveroNoDisponible("prueba".into()),
            AppError::DirectorioDeDatos,
            AppError::Db(arles_db::DbError::ClaveIncorrecta),
        ];
        for e in casos {
            assert!(e.clave_i18n().starts_with("error."));
        }
    }

    /// El error de formulario llega con sus campos; los demás, sin ninguno.
    /// Si esto se rompiera, el formulario marcaría campos al azar o ninguno.
    #[test]
    fn solo_el_error_de_formulario_lleva_campos() {
        use arles_core::{CampoDeEmpresa, ErrorDeCampo};

        let con: ErrorIpc = AppError::EmpresaInvalida(vec![ErrorDeCampo {
            campo: CampoDeEmpresa::ZonaHoraria,
            clave: "empresa.error.zonaNoSoportada",
        }])
        .into();
        assert_eq!(con.campos.len(), 1);
        assert_eq!(
            con.campos.first().map(|c| c.campo),
            Some(CampoDeEmpresa::ZonaHoraria)
        );

        let sin: ErrorIpc = AppError::DirectorioDeDatos.into();
        assert!(sin.campos.is_empty());
        // Y no aparece en el JSON cuando está vacío: el frontend distingue
        // «no hay campos» de «hay una lista vacía» sin tener que mirar dentro.
        let json = serde_json::to_string(&sin).expect("serializa");
        assert!(!json.contains("campos"), "{json}");
    }

    #[test]
    fn el_error_ipc_lleva_clave_y_detalle() {
        let ipc: ErrorIpc = AppError::Db(arles_db::DbError::ClaveIncorrecta).into();
        assert_eq!(ipc.clave, "error.db.clave_incorrecta");
        assert!(!ipc.detalle.is_empty());
    }

    /// THREAT_MODEL.md §4.1: los mensajes no revelan la estructura del disco.
    #[test]
    fn los_errores_no_revelan_rutas() {
        for e in [
            AppError::DirectorioDeDatos,
            AppError::ClaveMaestraPerdida,
            AppError::Db(arles_db::DbError::ClaveIncorrecta),
        ] {
            let t = e.to_string();
            assert!(!t.contains('/'), "el error revela una ruta: {t}");
            assert!(!t.contains('\\'), "el error revela una ruta: {t}");
        }
    }

    /// Hallazgo F7. El test de arriba solo cubría variantes que **nunca** pueden
    /// contener una ruta. Las que envuelven a `rusqlite` o al llavero sí las
    /// llevan, y cruzaban la frontera IPC intactas.
    #[test]
    fn el_error_ipc_redacta_las_rutas_de_las_capas_inferiores() {
        let e = AppError::LlaveroNoDisponible(
            "no se pudo abrir /home/ana/.local/share/keyrings/login.keyring".into(),
        );
        let ipc: ErrorIpc = e.into();

        assert!(
            !ipc.detalle.contains("/home/ana"),
            "la ruta cruzó la frontera IPC: {}",
            ipc.detalle
        );
        assert!(ipc.detalle.contains("[ruta]"));
        // Sigue siendo útil para diagnosticar: el resto del mensaje se conserva.
        assert!(ipc.detalle.contains("no se pudo abrir"));
    }

    #[test]
    fn la_redaccion_tambien_cubre_rutas_de_windows() {
        let ipc: ErrorIpc = AppError::LlaveroNoDisponible(
            r"fallo en C:\Users\Ana\AppData\Roaming\ArlesRelay".into(),
        )
        .into();
        assert!(!ipc.detalle.contains("Users"), "{}", ipc.detalle);
        assert!(ipc.detalle.contains("[ruta]"));
    }

    #[test]
    fn la_redaccion_no_estropea_un_mensaje_sin_rutas() {
        let ipc: ErrorIpc = AppError::ClaveMaestraPerdida.into();
        assert!(!ipc.detalle.contains("[ruta]"));
        assert!(ipc.detalle.contains("clave"));
    }
}
