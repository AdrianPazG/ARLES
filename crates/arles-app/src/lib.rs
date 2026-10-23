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
/// Las cuatro cifras del respaldo, que el presupuesto no tenía (hallazgo F-4
/// de la auditoría de la Fase 8). Sólo existe en pruebas: no entra en el
/// binario que se distribuye.
#[cfg(test)]
mod presupuesto_del_respaldo;
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
        // en `capabilities/principal.json`—: lo abren los comandos de Rust que
        // lo necesitan, y así la ruta del archivo no cruza la frontera
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
            comandos::descargar_plantilla_de_contactos,
            comandos::analizar_importacion,
            comandos::confirmar_importacion,
            comandos::listas_de_contactos,
            comandos::cancelar_importacion,
            comandos::listar_supresiones,
            comandos::suprimir_contacto,
            comandos::suprimir_direccion,
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
            // R-06 (17/10/2026). `nueva_version_de_plantilla` estaba en
            // `arles-db` desde la 6.3, probada, y sin registrar aquí: una
            // plantilla se creaba y no se podía corregir nunca.
            campanas::nueva_version_de_plantilla,
            // R-08 (21/10/2026): se creaban y no se podían quitar. Se
            // RETIRAN, no se borran — decisión de Dirección del mismo día.
            campanas::retirar_plantilla,
            campanas::firmas,
            campanas::guardar_firma,
            campanas::editar_firma,
            campanas::retirar_firma,
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
    ///
    /// Es `String` y no `&'static str` desde el 14/10/2026, porque el fallo de
    /// migración **tiene que decir dónde vive la base**: sin la ruta, la
    /// instrucción no se puede seguir.
    pub que_hacer: String,
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
/// `base` es dónde vive el archivo de la base de datos, cuando se sabe. Entra
/// como parámetro —y no se resuelve aquí dentro— para que esta función siga
/// siendo pura y los tests puedan fijarla en vez de depender de `%APPDATA%`.
///
/// **La regla 4 de la frontera no aplica a esta ruta.** «Ninguna ruta del
/// sistema de archivos cruza la frontera» protege la IPC con la webview; esto
/// es un cuadro de diálogo nativo que se muestra **antes de que exista
/// ninguna ventana**, y la ruta es justamente lo que el usuario necesita para
/// poder hacer algo.
///
/// Es público para que los tests puedan comprobar que cada error produce un
/// mensaje accionable, sin tener que arrancar la aplicación.
#[must_use]
pub fn explicar_fallo(e: &AppError, base: Option<&std::path::Path>) -> FalloDeArranque {
    // Si no se sabe la ruta exacta se nombra la carpeta por plataforma, que es
    // mejor que no decir nada: con eso el usuario llega igual.
    let donde = base.map_or_else(
        || {
            if cfg!(windows) {
                "%APPDATA%\\TelemetryInsight\\ArlesRelay\\data\\arles.db".to_owned()
            } else {
                "~/Library/Application Support/mx.TelemetryInsight.ArlesRelay/arles.db".to_owned()
            }
        },
        |p| p.display().to_string(),
    );

    let (que_hacer, a_salvo) = match e {
        AppError::ClaveMaestraPerdida => (
            "Restaura tu último respaldo .arles. La clave que cifraba esta base \
             de datos ya no está en tu sistema, y sin ella los datos no se pueden \
             leer."
                .to_owned(),
            "El archivo no se ha modificado ni borrado: sigue donde estaba.",
        ),
        AppError::LlaveroNoDisponible(_) => (
            "Verifica que el almacén de credenciales de tu sistema esté \
             disponible y desbloqueado, y vuelve a abrir ARLES."
                .to_owned(),
            "Tus datos están intactos: la base de datos no se ha modificado.",
        ),
        AppError::DirectorioDeDatos => (
            "Comprueba que tu usuario tiene permiso de escritura en la carpeta \
             de datos de las aplicaciones."
                .to_owned(),
            "No se ha creado ni modificado ningún archivo.",
        ),

        // ── EL CONSEJO QUE MANDABA A UN BUCLE ────────────────────────────────
        //
        // Hasta el 14/10/2026 este caso caía en el `_` de abajo y decía «cierra
        // ARLES y vuelve a abrirlo; si persiste, restaura tu último respaldo».
        // Las dos mitades están mal:
        //
        //   · Reabrir da **exactamente el mismo error**, siempre. El esquema de
        //     la base no cambia por volver a intentarlo, así que la primera
        //     instrucción es un bucle infinito.
        //   · «Restaura tu último respaldo» supone que hay uno. A quien abre
        //     ARLES por primera vez tras actualizar —que es justo cuando esto
        //     ocurre— lo manda a buscar un archivo que no existe.
        //
        // Y no decía **dónde** está la base, que es lo único que de verdad se
        // puede hacer: apartarla. Lo encontró Dirección el 13/10/2026, con el
        // primer instalador descargable y una base de la versión anterior.
        //
        // Se renombra y no se borra a propósito: la llave sigue en el llavero,
        // así que esos datos se pueden recuperar el día que exista la entrega
        // que migra un respaldo de otro esquema (declarada en la 8.3b).
        AppError::Db(arles_db::DbError::Migracion(_)) => (
            format!(
                "Esta base de datos es de una versión anterior de ARLES y su \
                 esquema ya no encaja, así que ARLES se niega a tocarla. Volver \
                 a abrirlo dará el mismo error.\n\n\
                 Cierra ARLES, aparta el archivo renombrándolo (no lo borres) y \
                 vuelve a abrirlo:\n\n\
                 {donde}\n\n\
                 Empezarás con una base nueva y vacía. Si tienes un respaldo \
                 .arles de esta misma versión, puedes restaurarlo después desde \
                 Ajustes."
            ),
            "El archivo no se ha tocado: ninguna migración llegó a aplicarse, y \
             la llave que lo descifra sigue en tu sistema.",
        ),

        _ => (
            "Cierra ARLES y vuelve a abrirlo. Si el problema persiste, restaura \
             tu último respaldo."
                .to_owned(),
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
    // Sin crear ninguna carpeta: un fallo de arranque no es momento de escribir
    // nada en disco, y el fallo de migración necesita esta ruta para que su
    // instrucción se pueda seguir.
    let base = rutas::Rutas::base_de_datos_probable();
    let texto = explicar_fallo(e, base.as_deref()).to_string();
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
            let f = explicar_fallo(&e, None);
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
        let f = explicar_fallo(&AppError::ClaveMaestraPerdida, None);
        assert!(f.que_hacer.contains("respaldo"), "{}", f.que_hacer);
    }

    /// ── El fallo que Dirección vio el 13/10/2026 ────────────────────────────
    ///
    /// Un instalador nuevo sobre la base de la versión anterior. El mensaje
    /// caía en el `_` genérico y decía dos cosas que no sirven: «cierra ARLES y
    /// vuelve a abrirlo» —que da el mismo error siempre— y «restaura tu último
    /// respaldo» —que puede no existir—. Y no decía **dónde** está el archivo,
    /// que es lo único que se puede hacer.
    ///
    /// Esta prueba se vio en rojo antes del arreglo, con el mensaje genérico:
    /// no contenía la ruta, no nombraba el renombrado, y sí decía «vuelve a
    /// abrirlo» como instrucción.
    #[test]
    fn el_fallo_de_migracion_dice_donde_esta_la_base_y_que_hacer_con_ella() {
        let base = std::path::Path::new("/tmp/prueba/ArlesRelay/data/arles.db");
        let e = AppError::Db(arles_db::DbError::Migracion(
            "applied migration V1__esquema_inicial is different than filesystem one".to_owned(),
        ));
        let f = explicar_fallo(&e, Some(base));

        // 1 · La ruta exacta, o la instrucción no se puede seguir.
        assert!(
            f.que_hacer.contains("arles.db"),
            "no dice dónde está la base: {}",
            f.que_hacer
        );
        assert!(
            f.que_hacer.contains("/tmp/prueba/ArlesRelay/data"),
            "no usa la ruta que se le pasó: {}",
            f.que_hacer
        );

        // 2 · La salida real es apartar el archivo, no reabrir.
        assert!(
            f.que_hacer.contains("renombr") && f.que_hacer.contains("no lo borres"),
            "no dice que se renombre sin borrar: {}",
            f.que_hacer
        );

        // 3 · Y dice que reabrir NO sirve, que es lo que evita el bucle.
        assert!(
            f.que_hacer.contains("mismo error"),
            "no avisa de que reabrir da el mismo error: {}",
            f.que_hacer
        );

        // 4 · El §95 entero: qué está a salvo, sin prometer de más.
        assert!(
            f.a_salvo.contains("no se ha tocado"),
            "la tercera parte del §95 no dice qué está a salvo: {}",
            f.a_salvo
        );
    }

    /// Sin ruta conocida el mensaje **sigue sirviendo**: nombra la carpeta por
    /// convención de la plataforma. Callar sería peor que aproximar.
    #[test]
    fn sin_ruta_el_mensaje_de_migracion_nombra_la_carpeta_de_la_plataforma() {
        let e = AppError::Db(arles_db::DbError::Migracion("lo que sea".to_owned()));
        let f = explicar_fallo(&e, None);
        let esperado = if cfg!(windows) {
            "%APPDATA%"
        } else {
            "Application Support"
        };
        assert!(f.que_hacer.contains(esperado), "{}", f.que_hacer);
        assert!(f.que_hacer.contains("arles.db"), "{}", f.que_hacer);
    }
}
