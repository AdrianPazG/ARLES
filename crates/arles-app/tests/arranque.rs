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
use arles_app_lib::{AppError, EstadoApp, Llavero, rutas::Rutas};

/// Cada test usa su propia cuenta del llavero.
///
/// Todos comparten el almacén real de la máquina, así que con una cuenta fija un
/// test que borra la entrada rompe a otro que corre en paralelo. Ocurrió, y los
/// resultados parecían correctos.
fn entorno(nombre: &str) -> (tempfile::TempDir, Rutas, Llavero) {
    let dir = tempfile::tempdir().expect("directorio temporal");
    let rutas = Rutas::en(dir.path().to_path_buf());
    let llavero = Llavero::con_cuenta(format!("prueba-{nombre}-{}", std::process::id()));
    (dir, rutas, llavero)
}

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
    let (_d, rutas, llavero) = entorno("arranque");

    match EstadoApp::arrancar_con(&rutas, &llavero) {
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
                arles_db::ultima_version(),
                "el arranque debe dejar el esquema en la última versión"
            );
            assert_eq!(
                resumen.empresas, 0,
                "una instalación nueva no tiene empresa: toca el onboarding (§25)"
            );
            assert!(
                rutas.base_de_datos().exists(),
                "la base debería existir en el directorio de datos"
            );

            let _ = llavero.borrar();
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

/// Hallazgo F2 de la revisión de la Fase 1.
///
/// Si la entrada del llavero desaparece —reinstalación del sistema, cambio de
/// equipo, perfil corrupto— pero el archivo de base de datos sigue ahí, la
/// aplicación **no puede generar una clave nueva en silencio**.
///
/// Hacerlo era lo cómodo y lo peor posible: la base quedaba irrecuperable y el
/// usuario solo veía «no se pudo descifrar», sin enterarse de que lo que
/// necesitaba era restaurar un respaldo (riesgo R-10).
#[test]
fn una_clave_perdida_con_base_existente_se_nombra_en_vez_de_taparse() {
    let (_d, rutas, llavero) = entorno("clave-perdida");

    if EstadoApp::arrancar_con(&rutas, &llavero).is_err() {
        println!("RAMA: sin llavero, escenario no aplicable");
        return;
    }
    println!("RAMA: llavero disponible, escenario ejercitado");
    assert!(rutas.base_de_datos().exists());

    // Se pierde la entrada del llavero; la base permanece.
    llavero.borrar().expect("borra la entrada");

    match EstadoApp::arrancar_con(&rutas, &llavero) {
        Err(AppError::ClaveMaestraPerdida) => {}
        Err(otro) => panic!(
            "se reportó «{otro}», que no le dice al usuario que debe restaurar \
             un respaldo"
        ),
        Ok(_) => panic!("arrancó con una base que no puede descifrar"),
    }

    // Y lo más importante: NO se guardó una clave nueva encima.
    assert!(
        llavero.leer().expect("lee el llavero").is_none(),
        "se generó una clave nueva pese a existir una base: si el usuario \
         recuperase la entrada original, esta la habría sobrescrito"
    );
}

/// Los mensajes de error se muestran al usuario; no deben revelar la estructura
/// del disco ni el contenido de un secreto (THREAT_MODEL.md §4.1).
#[test]
fn los_errores_de_arranque_no_filtran_rutas_ni_secretos() {
    let (dir, rutas, llavero) = entorno("errores");

    if let Err(e) = EstadoApp::arrancar_con(&rutas, &llavero) {
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
    let _ = llavero.borrar();
}
