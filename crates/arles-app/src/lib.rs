//! Shell de escritorio de ARLES RELAY.
//!
//! Orquesta el arranque y expone los comandos Tauri. **Toda la autoridad de
//! negocio vive en los crates de dominio**: este crate conecta, no decide.

// §138: sin `unwrap()` ni `panic!()` indiscriminados en producción.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]

pub mod archivo;
pub mod campanas;
mod cifras;
pub mod comandos;
mod control;
pub mod error;
pub mod estado;
pub mod fabrica;
pub mod llavero;
pub mod motor;
pub mod recuperacion;
pub mod registro;
pub mod respaldo;
pub mod restauracion;
pub mod rutas;
pub mod whatsapp;

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

        // ── LA CLAVE PERDIDA TIENE SALIDA DESDE LA 8.4 ────────────────────
        //
        // Hasta entonces, aquí se enseñaba un mensaje que decía «restaura tu
        // último respaldo» y se llamaba a `exit(1)`: una instrucción que el
        // producto impedía seguir, el día en que alguien ya ha perdido algo.
        //
        // Ahora arranca en modo de recuperación. **No se crea ninguna clave
        // aquí** —eso es el hallazgo F2 de la Fase 1—: la clave nueva sólo
        // aparece cuando alguien elige un respaldo y confirma.
        Err(AppError::ClaveMaestraPerdida) => {
            match (rutas::Rutas::resolver(), llavero::Llavero::del_sistema()) {
                (Ok(rutas), llavero) => {
                    recuperacion::modo_recuperacion(rutas, llavero);
                    return;
                }
                // Si ni siquiera hay carpeta de datos, no hay recuperación
                // posible y se dice lo de siempre.
                (Err(e), _) => {
                    mostrar_fallo_de_arranque(&e);
                    std::process::exit(1);
                }
            }
        }

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
            comandos::crear_contacto,
            comandos::editar_contacto,
            comandos::borrar_contacto,
            comandos::elegir_archivo_para_importar,
            comandos::analizar_importacion,
            comandos::confirmar_importacion,
            comandos::cancelar_importacion,
            comandos::listar_supresiones,
            comandos::suprimir_contacto,
            comandos::levantar_supresion,
            comandos::entregar_informe_de_acceso,
            comandos::que_se_borraria,
            comandos::cancelar_datos_de,
            comandos::remitentes,
            comandos::probar_remitente,
            comandos::guardar_remitente,
            comandos::enviar_correo_de_prueba,
            comandos::probar_envio_de_la_cuenta,
            comandos::revisar_campana,
            // ── `revisar_campana_de_whatsapp` NO está registrado, y es a
            //    propósito desde la 7.7 ──────────────────────────────────────
            //
            // Es el preflight del canal (W.5), con sus dos cifras de permiso.
            // Funciona y tiene sus pruebas, pero **es la puerta de un flujo que
            // no existe**: el motor sólo recorre `Canal::Correo`, así que no se
            // puede lanzar una campaña entera por WhatsApp y no hay pantalla
            // que pueda pedir esta revisión con sentido.
            //
            // Registrarlo era peor que no tenerlo: un comando en la frontera
            // que nadie llama es superficie de ataque que nadie vigila. Vuelve
            // aquí el día que el motor recorra WhatsApp, que es la decisión que
            // espera a Dirección en `FASE-05-PARA-DIRECCION.md`. La función y
            // sus pruebas se quedan donde están.
            whatsapp::numeros_de_whatsapp,
            whatsapp::guardar_numero_de_whatsapp,
            whatsapp::confirmar_coexistencia,
            whatsapp::aceptar_riesgo_del_canal,
            whatsapp::enviar_whatsapp_de_prueba,
            campanas::plantillas,
            campanas::guardar_plantilla,
            campanas::firmas,
            campanas::guardar_firma,
            campanas::ventanas,
            campanas::guardar_ventana,
            campanas::campanas,
            campanas::crear_campana,
            campanas::activar_campana,
            // El freno (7.0). Estos seis llevaban desde la Fase 4 sin existir,
            // con todo lo demás construido debajo: una campaña activada no se
            // podía parar desde la aplicación.
            control::pausar_campana,
            control::reanudar_campana,
            control::detener_campana,
            control::parar_todo,
            control::levantar_parada,
            control::estado_del_envio,
            // Las cifras de Actividad (7.2).
            cifras::actividad,
            // El registro con sus filtros, y el recorte bimestral (7.4, D-7C).
            registro::registro,
            registro::que_se_recortaria,
            registro::recortar_registro,
            // El resumen exportable (7.6).
            registro::exportar_resumen,
            // El respaldo (8.2c). Primero se dice con cifras qué se va a
            // llevar, y después se escribe: es el patrón del recorte del
            // registro (7.4) y del derecho de cancelación (3.4).
            respaldo::que_llevaria_el_respaldo,
            respaldo::exportar_respaldo,
            restauracion::revisar_respaldo,
            restauracion::restaurar_respaldo,
            // Archivar el historial (8.3 bis b, D-8D). El mismo orden, y aquí
            // no es una costumbre: el segundo **borra**, y lo único que
            // separa archivar de borrar es haber escrito y releído el archivo
            // antes.
            archivo::revisar_archivado,
            archivo::archivar_historial,
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
