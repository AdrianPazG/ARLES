//! El arranque, verificado de extremo a extremo.
//!
//! Comprueba la decisión más incómoda del producto: **si el llavero del sistema
//! no está disponible, la aplicación no arranca** (ADR-0011). No hay modo
//! compatibilidad ni archivo de claves de respaldo, porque eso convertiría el
//! cifrado en teatro: un atacante con acceso al disco obtendría la base de datos
//! y su clave del mismo directorio, y el fallo sería silencioso.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

// El crate librería se llama `arles_app_lib`: Tauri 2 exige separar librería y
// binario para poder construir también objetivos móviles.
use arles_app_lib::{AppError, EstadoApp, rutas::Rutas};

/// El invariante, en las dos ramas posibles.
///
/// En una máquina con llavero (macOS, Windows, o Linux con servicio de secretos)
/// el arranque funciona y deja la base migrada. Donde no lo hay —un contenedor
/// de CI sin D-Bus, por ejemplo— **falla de forma explícita**.
///
/// Lo que nunca puede ocurrir es la tercera opción: arrancar sin llavero
/// escribiendo la clave en un archivo.
#[test]
fn sin_llavero_no_arranca_y_con_llavero_queda_migrada() {
    let dir = tempfile::tempdir().expect("directorio temporal");
    let rutas = Rutas::en(dir.path().to_path_buf());

    match EstadoApp::arrancar_en(&rutas) {
        Ok(estado) => {
            // Se declara la rama para que la validación sepa qué se ejercitó:
            // un «ok» no dice si se probó el arranque o el rechazo.
            println!("RAMA: llavero disponible, arranque completo");

            let resumen = estado
                .db()
                .resumen_arranque()
                .expect("la base debe responder tras un arranque correcto");

            assert_eq!(
                resumen.version_esquema,
                Some(1),
                "el arranque debe dejar el esquema migrado"
            );
            assert_eq!(
                resumen.empresas, 0,
                "una instalación nueva no tiene empresa: toca el onboarding (§25)"
            );

            assert!(
                rutas.base_de_datos().exists(),
                "la base debería existir en el directorio de datos"
            );
        }

        Err(AppError::LlaveroNoDisponible(_)) => {
            // La rama correcta cuando no hay almacén de credenciales.
            println!("RAMA: sin llavero, arranque rechazado");
            assert!(
                !rutas.base_de_datos().exists(),
                "sin llavero no debe quedar NINGUNA base en disco: si existiera, \
                 se habría creado sin cifrado real o con una clave improvisada"
            );
        }

        Err(otro) => panic!(
            "el arranque falló por un motivo inesperado: {otro}. Sin llavero, el \
             único fallo admisible es LlaveroNoDisponible"
        ),
    }
}

/// Los mensajes de error se muestran al usuario; no deben revelar la estructura
/// del disco ni el contenido de un secreto (THREAT_MODEL.md §4.1).
#[test]
fn los_errores_de_arranque_no_filtran_rutas_ni_secretos() {
    let dir = tempfile::tempdir().expect("directorio temporal");
    let rutas = Rutas::en(dir.path().to_path_buf());

    if let Err(e) = EstadoApp::arrancar_en(&rutas) {
        let texto = e.to_string();
        assert!(
            !texto.contains(dir.path().to_string_lossy().as_ref()),
            "el error revela la ruta del directorio de datos: {texto}"
        );
        assert!(
            e.clave_i18n().starts_with("error."),
            "todo error debe tener clave de i18n para que la interfaz lo traduzca (§139)"
        );
    }
}
