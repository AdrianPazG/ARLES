//! Shell de escritorio de ARLES RELAY.
//!
//! Orquesta el arranque y expone los comandos Tauri. **Toda la autoridad de
//! negocio vive en los crates de dominio**: este crate conecta, no decide.

// §138: sin `unwrap()` ni `panic!()` indiscriminados en producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]

pub mod comandos;
pub mod error;
pub mod estado;
pub mod llavero;
pub mod rutas;

pub use error::{AppError, ErrorIpc};
pub use estado::EstadoApp;
pub use llavero::Llavero;

/// Punto de entrada de la aplicación de escritorio.
///
/// # Pánico
///
/// Si el arranque falla —típicamente porque el llavero del sistema no está
/// disponible— la aplicación **se detiene con un mensaje explicativo**. Es
/// deliberado: arrancar sin poder descifrar la base de datos significaría
/// funcionar sin los datos del usuario, y eso es peor que no arrancar
/// (ADR-0011).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let estado = match EstadoApp::arrancar() {
        Ok(e) => e,
        Err(e) => {
            mostrar_fallo_de_arranque(&e);
            std::process::exit(1);
        }
    };

    let resultado = tauri::Builder::default()
        // El diálogo de archivos. **La webview no recibe su permiso** —míralo
        // en `capabilities/principal.json`—: sólo `elegir_archivo_para_importar`
        // lo usa, desde Rust, y así la ruta del archivo no cruza la frontera
        // (regla 4).
        .plugin(tauri_plugin_dialog::init())
        .manage(estado)
        .invoke_handler(tauri::generate_handler![
            comandos::info_app,
            comandos::estado_arranque,
            comandos::configuracion_de_empresa,
            comandos::guardar_empresa,
            comandos::preferencias_de_interfaz,
            comandos::guardar_barra_plegada,
            comandos::guardar_tema,
            comandos::abrir_sitio_de_telemetry,
            comandos::listar_contactos,
            comandos::contacto,
            comandos::crear_contacto,
            comandos::editar_contacto,
            comandos::borrar_contacto,
            comandos::elegir_archivo_para_importar,
            comandos::analizar_importacion,
            comandos::confirmar_importacion,
            comandos::cancelar_importacion,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = resultado {
        eprintln!("ARLES RELAY terminó de forma inesperada: {e}");
        std::process::exit(1);
    }
}

/// Un fallo de arranque explicado en las tres partes que exige el §95.
///
/// Es una estructura y no una cadena para que la tercera parte —qué está a
/// salvo, la que falta en casi todo el software— **no se pueda omitir**: el
/// compilador obliga a rellenarla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FalloDeArranque {
    /// Qué pasó.
    pub que_paso: String,
    /// Cómo arreglarlo.
    pub que_hacer: &'static str,
    /// Qué está a salvo.
    pub a_salvo: &'static str,
}

impl std::fmt::Display for FalloDeArranque {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ARLES RELAY no pudo arrancar.\n\n\
             Qué pasó: {}.\n\n\
             Qué hacer: {}\n\n\
             {}",
            self.que_paso, self.que_hacer, self.a_salvo
        )
    }
}

/// Explica un fallo de arranque.
///
/// Es público para que los tests puedan comprobar que cada error produce un
/// mensaje accionable, sin tener que arrancar la aplicación.
#[must_use]
pub fn explicar_fallo(e: &AppError) -> FalloDeArranque {
    let (que_hacer, a_salvo) = match e {
        AppError::ClaveMaestraPerdida => (
            "Restaura tu último respaldo .arles. La clave que cifraba esta base \
             de datos ya no está en tu sistema, y sin ella los datos no se pueden \
             leer.",
            "El archivo no se ha modificado ni borrado: sigue donde estaba.",
        ),
        AppError::LlaveroNoDisponible(_) => (
            "Verifica que el almacén de credenciales de tu sistema esté \
             disponible y desbloqueado, y vuelve a abrir ARLES.",
            "Tus datos están intactos: la base de datos no se ha modificado.",
        ),
        AppError::DirectorioDeDatos => (
            "Comprueba que tu usuario tiene permiso de escritura en la carpeta \
             de datos de las aplicaciones.",
            "No se ha creado ni modificado ningún archivo.",
        ),
        _ => (
            "Cierra ARLES y vuelve a abrirlo. Si el problema persiste, restaura \
             tu último respaldo.",
            "Las operaciones incompletas se revirtieron por completo.",
        ),
    };

    FalloDeArranque {
        que_paso: e.to_string(),
        que_hacer,
        a_salvo,
    }
}

/// Comunica un fallo de arranque por los canales que el usuario pueda ver.
///
/// En Windows el binario se compila con `windows_subsystem = "windows"` para que
/// no aparezca una consola detrás de la ventana. El efecto secundario es que
/// `eprintln!` no va a ninguna parte: la aplicación moriría sin decir **nada**,
/// que es exactamente el fallo silencioso que ADR-0011 existe para evitar. Por
/// eso ahí se muestra además un cuadro de diálogo nativo.
fn mostrar_fallo_de_arranque(e: &AppError) {
    let texto = explicar_fallo(e).to_string();
    eprintln!("{texto}");

    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};

        let a_utf16 = |s: &str| {
            s.encode_utf16()
                .chain(std::iter::once(0))
                .collect::<Vec<u16>>()
        };
        let cuerpo = a_utf16(&texto);
        let titulo = a_utf16("ARLES RELAY");

        // SAFETY: ambos búferes terminan en NUL y viven hasta después de la
        // llamada. El workspace prohíbe `unsafe`, y esta es la única excepción:
        // sin ella la aplicación muere en silencio en Windows.
        #[allow(unsafe_code)]
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                cuerpo.as_ptr(),
                titulo.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §95: todo fallo de arranque dice qué pasó, cómo arreglarlo y qué está a
    /// salvo. La tercera parte es la que importa cuando alguien acaba de ver
    /// fallar la aplicación con sus datos dentro.
    #[test]
    fn todo_fallo_de_arranque_es_accionable() {
        let casos = [
            AppError::ClaveMaestraPerdida,
            AppError::LlaveroNoDisponible("prueba".into()),
            AppError::DirectorioDeDatos,
            AppError::Db(arles_db::DbError::ClaveIncorrecta),
        ];

        for e in casos {
            let f = explicar_fallo(&e);
            assert!(!f.que_paso.is_empty(), "falta qué pasó en {e}");
            assert!(!f.que_hacer.is_empty(), "falta qué hacer en {e}");
            assert!(!f.a_salvo.is_empty(), "falta qué está a salvo en {e}");

            let t = f.to_string();
            assert!(t.contains("Qué pasó:"));
            assert!(t.contains("Qué hacer:"));
            assert!(t.contains(f.a_salvo));
        }
    }

    /// El caso de la clave perdida tiene una acción propia: ningún otro mensaje
    /// sirve, porque lo único que recupera los datos es un respaldo.
    #[test]
    fn la_clave_perdida_manda_al_respaldo() {
        let f = explicar_fallo(&AppError::ClaveMaestraPerdida);
        assert!(f.que_hacer.contains("respaldo"), "{}", f.que_hacer);
    }
}
