//! El camino completo de la configuración de empresa, de punta a punta.
//!
//! Por qué existe este archivo, dicho sin adornos: hasta la auditoría de la
//! 3.1, cada capa estaba probada por su lado —el dominio valida, la base
//! guarda, el comando conecta— y **nada recorría el camino entero**. Una
//! frontera mal puesta entre dos capas probadas pasa desapercibida: cada test
//! dice que su lado funciona.
//!
//! Esto arranca la aplicación como lo hace de verdad —llavero, base cifrada,
//! migraciones— y hace lo que hace un usuario. Lo único que queda fuera es
//! Tauri, que sólo transporta.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

// El crate librería se llama `arles_app_lib`: Tauri 2 exige separar librería y
// binario para poder construir también objetivos móviles.
use arles_app_lib::{EstadoApp, Llavero, rutas::Rutas};
use arles_core::onboarding::{ListaDeOnboarding, PasoDeOnboarding};
use arles_core::{BorradorDeEmpresa, DatosDeEmpresa};

fn entorno(nombre: &str) -> (tempfile::TempDir, Rutas, Llavero) {
    let dir = tempfile::tempdir().expect("directorio temporal");
    let rutas = Rutas::en(dir.path().to_path_buf());
    // Cuenta propia por test: el almacén es el de la máquina, y con una cuenta
    // fija un test que borra la entrada rompe a otro que corre en paralelo.
    let llavero = Llavero::con_cuenta(format!("prueba-empresa-{nombre}-{}", std::process::id()));
    (dir, rutas, llavero)
}

fn borrador() -> BorradorDeEmpresa {
    BorradorDeEmpresa {
        nombre_comercial: "  TELEMETRY INSIGHT  ".into(),
        pais: "mx".into(),
        zona_horaria: "America/Mexico_City".into(),
        correo_corporativo: " Hola@Telemetry.MX ".into(),
        sitio_web: "https://telemetry.mx".into(),
    }
}

/// El recorrido entero: instalación nueva → alta pendiente → configurar →
/// alta avanzada → reabrir y seguir ahí.
#[test]
fn configurar_la_empresa_avanza_el_alta_y_sobrevive_al_reinicio() {
    let (_d, rutas, llavero) = entorno("camino");

    let Ok(estado) = EstadoApp::arrancar_con(&rutas, &llavero) else {
        // Sin llavero en la máquina no se puede ejercitar: se dice y se sale,
        // en vez de dar por buena una comprobación que no se hizo.
        println!("RAMA: sin llavero, no se puede ejercitar el camino completo");
        return;
    };
    println!("RAMA: llavero disponible, camino completo ejercitado");

    // Instalación nueva: nada configurado, y el alta lo refleja.
    let lista = ListaDeOnboarding::desde(&estado.db().recuento_de_alta().expect("recuento"));
    assert_eq!(lista.completados, 0);
    assert_eq!(lista.siguiente, Some(PasoDeOnboarding::Empresa));
    assert!(estado.db().empresa().expect("consulta").is_none());

    // Se configura.
    let datos = DatosDeEmpresa::validar(&borrador()).expect("el borrador es válido");
    estado.db().guardar_empresa(&datos).expect("guarda");

    // Lo que se lee es lo que se escribió, ya normalizado.
    let guardada = estado
        .db()
        .empresa()
        .expect("consulta")
        .expect("hay empresa");
    assert_eq!(guardada.datos.nombre_comercial(), "TELEMETRY INSIGHT");
    assert_eq!(guardada.datos.pais(), "MX");
    assert_eq!(
        guardada.datos.correo_corporativo().normalized(),
        "hola@telemetry.mx",
        "la normalización del correo tiene que llegar hasta la base"
    );

    // Y el alta avanza sola, porque se deriva del dato.
    let lista = ListaDeOnboarding::desde(&estado.db().recuento_de_alta().expect("recuento"));
    assert_eq!(lista.completados, 1);
    assert!(!lista.terminada(), "queda mucho para poder enviar");

    // Se cierra la aplicación y se vuelve a abrir.
    drop(estado);
    let estado = EstadoApp::arrancar_con(&rutas, &llavero).expect("segundo arranque");
    let otra_vez = estado.db().empresa().expect("consulta").expect("sigue ahí");
    assert_eq!(otra_vez.id, guardada.id, "debe ser la misma empresa");
    assert_eq!(otra_vez.datos, guardada.datos);
}

/// La preferencia de la barra lateral sobrevive al cierre.
///
/// Es la mitad de P-11 que no se ve en una captura: «se recuerda» sólo se
/// puede comprobar cerrando y volviendo a abrir.
#[test]
fn la_barra_plegada_se_recuerda_entre_arranques() {
    let (_d, rutas, llavero) = entorno("preferencia");

    let Ok(estado) = EstadoApp::arrancar_con(&rutas, &llavero) else {
        println!("RAMA: sin llavero, no se puede ejercitar la preferencia");
        return;
    };
    println!("RAMA: llavero disponible, preferencia ejercitada");

    assert_eq!(
        estado
            .db()
            .preferencia("barra_lateral_plegada")
            .expect("lee"),
        None,
        "una instalación nueva abre con la barra desplegada"
    );

    estado
        .db()
        .guardar_preferencia("barra_lateral_plegada", "1")
        .expect("guarda");

    drop(estado);
    let estado = EstadoApp::arrancar_con(&rutas, &llavero).expect("segundo arranque");
    assert_eq!(
        estado
            .db()
            .preferencia("barra_lateral_plegada")
            .expect("lee"),
        Some("1".to_owned()),
        "P-11: si la dejaste plegada, abre plegada"
    );
}

/// Los datos malos no llegan a la base.
///
/// El comando valida antes de escribir; esto comprueba que **no hay otra
/// puerta**: con el borrador inválido no existe forma de construir los datos,
/// y sin ellos `guardar_empresa` ni siquiera se puede llamar.
#[test]
fn un_borrador_invalido_no_produce_datos_que_guardar() {
    let malo = BorradorDeEmpresa {
        nombre_comercial: "TELEMETRY\r\nBcc: victima@otra.com".into(),
        pais: "MX".into(),
        zona_horaria: "America/Mexico".into(),
        correo_corporativo: "no-es-correo".into(),
        sitio_web: "javascript:alert(1)".into(),
    };
    let errores = DatosDeEmpresa::validar(&malo).expect_err("debería rechazarse");
    assert_eq!(errores.len(), 4, "faltan campos por reportar: {errores:?}");
}
