//! El llavero perdido: la otra mitad del ADR-0011.
//!
//! Entrega 8.4.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL CONSEJO QUE EL PRODUCTO NO PODÍA CUMPLIR
//!
//! Desde la Fase 1, perder la entrada del llavero produce un mensaje con las
//! tres partes del §95, y la de «cómo se arregla» dice *«Restaura tu último
//! respaldo .arles»*. **Y no había forma de hacerlo**: `run()` llamaba a
//! `std::process::exit(1)` antes de abrir ninguna ventana.
//!
//! No es que faltara un botón. Es que el producto **daba una instrucción que él
//! mismo impedía seguir**, el día en que alguien ya ha perdido algo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS DOS COSAS QUE AQUÍ NO SE PUEDEN HACER MAL
//!
//! **1 · No se crea una clave nueva al arrancar.** Sería lo cómodo y es el
//! hallazgo F2 de la Fase 1: la base existente quedaría ilegible para siempre y
//! nadie se enteraría. El llavero puede estar vacío por motivos pasajeros —un
//! Keychain bloqueado, un perfil que todavía no ha montado, un gestor de
//! credenciales que tarda—. La clave nueva se crea **sólo** cuando alguien pide
//! empezar de cero, con un respaldo elegido y confirmado.
//!
//! **2 · La base ilegible no se borra. Se aparta.** Un llavero puede volver: un
//! Keychain restaurado de una copia, un perfil de usuario que se recupera, un
//! disco que se vuelve a montar. El día que vuelva, esos datos valen más que
//! cualquier respaldo, porque son los de hoy y el respaldo es de la semana
//! pasada. Borrarla sería cambiar un problema recuperable por uno que no lo es.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LÍMITE DECLARADO
//!
//! Esto se prueba entero en Linux, donde el llavero del sistema se simula. En
//! Windows y macOS se prueba el día que se cierre **R-07**. Se dice así, y no
//! «probado» a secas.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use arles_db::Db;

use crate::error::{AppError, ErrorIpc};
use crate::llavero::Llavero;
use crate::rutas::Rutas;

/// Lo que sabe la aplicación cuando arranca sin poder abrir su base.
///
/// No tiene `Db` —no hay ninguna que abrir— y es justo lo que la distingue de
/// [`crate::EstadoApp`]. Por eso el modo de recuperación registra sus propios
/// comandos: ninguno de los ciento y pico normales puede funcionar aquí, y
/// ofrecerlos sería ofrecer puertas pintadas.
#[derive(Debug)]
pub struct EstadoDeRecuperacion {
    rutas: Rutas,
    llavero: Llavero,
    /// El respaldo que se está revisando, entre que se elige y se confirma.
    /// Mismo criterio que en la restauración normal: la ruta no cruza la
    /// frontera (regla 4).
    respaldo_elegido: Mutex<Option<PathBuf>>,
}

impl EstadoDeRecuperacion {
    #[must_use]
    pub fn nuevo(rutas: Rutas, llavero: Llavero) -> Self {
        Self {
            rutas,
            llavero,
            respaldo_elegido: Mutex::new(None),
        }
    }
}

/// Lo que la pantalla de recuperación necesita saber. Con cifras (§94).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuePaso {
    /// Cuántos bytes ocupa la base que no se puede leer. Es la forma honesta de
    /// decir «tus datos siguen ahí»: un número que se puede comprobar.
    pub bytes_de_la_base: u64,
    /// El nombre del archivo, **nunca su ruta** (regla 4).
    pub nombre_de_la_base: String,
}

/// Qué pasó, para poder decirlo con las tres partes del §95.
///
/// # Errores
///
/// Nunca falla: si no se puede leer el tamaño del archivo se devuelve 0, que es
/// menos informativo pero no impide enseñar la pantalla. Quedarse sin salida
/// por no poder leer unos metadatos sería el mismo fallo otra vez.
#[tauri::command]
pub fn que_paso_con_la_clave(estado: tauri::State<'_, EstadoDeRecuperacion>) -> QuePaso {
    let base = estado.rutas.base_de_datos();
    QuePaso {
        bytes_de_la_base: std::fs::metadata(&base).map(|m| m.len()).unwrap_or(0),
        nombre_de_la_base: base
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arles.db")
            .to_owned(),
    }
}

/// Elige un respaldo y dice qué trae. **No toca nada.**
///
/// Es la hermana de `revisar_respaldo` (8.3c) y no la misma: aquí no hay base
/// contra la que comparar, así que no hay preflight de pérdidas — no se pierde
/// nada, porque no hay nada que leer. Lo que sí se comprueba es que el archivo
/// sea un respaldo y de un esquema que esta versión sepa leer, **antes** de
/// crear ninguna clave.
///
/// # Errores
///
/// - `error.core.no_es_un_respaldo` si el archivo no lo es.
/// - `error.app.archivo_no_se_pudo_leer` si no se puede abrir.
#[tauri::command]
pub fn revisar_respaldo_para_recuperar(
    estado: tauri::State<'_, EstadoDeRecuperacion>,
    app: tauri::AppHandle,
) -> Result<Option<crate::restauracion::LoQueTraeElRespaldo>, ErrorIpc> {
    use tauri_plugin_dialog::DialogExt;

    let Some(elegido) = app
        .dialog()
        .file()
        .add_filter("Respaldo de ARLES", &["arles"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let ruta = elegido
        .into_path()
        .map_err(|_| ErrorIpc::from(AppError::ArchivoNoSePudoLeer))?;

    let (cabecera, _) = crate::restauracion::leer_la_cabecera(&ruta)?;
    // La misma forma que escribe el exportador: «V11».
    let esquema_de_la_base = arles_db::ultima_version()
        .map(|v| format!("V{v}"))
        .unwrap_or_default();
    let se_puede = cabecera.esquema == esquema_de_la_base;

    estado
        .respaldo_elegido
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .replace(ruta.clone());

    Ok(Some(crate::restauracion::LoQueTraeElRespaldo {
        nombre_del_archivo: ruta
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
        creado_en: cabecera.creado_en.clone(),
        alcance: cabecera.alcance.as_str().to_owned(),
        esquema_del_respaldo: cabecera.esquema.clone(),
        esquema_de_la_base,
        lleva_contrasena: cabecera.cifrado.lleva_contrasena(),
        se_puede_restaurar: se_puede,
        // Sin base no hay con qué comparar, y **no se inventa**: una tabla con
        // «ahora: 0» al lado daría a entender que se comprobó algo.
        tablas: Vec::new(),
        total_de_filas: cabecera.total_de_filas(),
        // No se pierde nada porque no hay nada legible que perder.
        no_vuelven: Vec::new(),
        filas_que_no_vuelven: 0,
        bajas_ahora: 0,
        bajas_en_el_respaldo: cabecera
            .filas
            .iter()
            .find(|(t, _)| t == "suppression_entry")
            .map_or(0, |(_, n)| *n),
        bajas_al_menos: cabecera
            .filas
            .iter()
            .find(|(t, _)| t == "suppression_entry")
            .map_or(0, |(_, n)| *n),
        cuentas_a_reconectar: ["email_account", "whatsapp_account"]
            .iter()
            .map(|t| {
                cabecera
                    .filas
                    .iter()
                    .find(|(n, _)| n == t)
                    .map_or(0, |(_, n)| *n)
            })
            .sum(),
    }))
}

/// Empieza de cero con el respaldo revisado.
///
/// En este orden exacto, y cada paso está donde está por un motivo:
///
/// 1. **Se aparta la base ilegible**, con fecha en el nombre. No se borra.
/// 2. **Se crea la clave nueva**, y sólo ahora: hasta aquí nadie había pedido
///    nada irreversible.
/// 3. Se abre una base nueva y se restaura el respaldo dentro.
///
/// Si algo falla después de apartar, la base apartada **sigue ahí** con su
/// nombre nuevo, y la pantalla lo dice.
///
/// # Errores
///
/// - `error.app.respaldo_sin_revisar` si no se revisó uno antes.
/// - `error.core.respaldo_corrupto` si la contraseña no abre el archivo.
/// - `error.app.archivo_no_se_pudo_leer` si no se puede apartar la base vieja.
#[tauri::command]
pub fn recuperar_desde_respaldo(
    estado: tauri::State<'_, EstadoDeRecuperacion>,
    contrasena: Option<String>,
) -> Result<Recuperado, ErrorIpc> {
    let ruta = estado
        .respaldo_elegido
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let Some(ruta) = ruta else {
        return Err(AppError::RespaldoSinRevisar.into());
    };

    let base = estado.rutas.base_de_datos();
    let apartada = apartar_la_base_ilegible(&base, &ahora_para_nombre())?;

    let clave = estado.llavero.crear()?;
    let db = Db::abrir(&base, &clave).map_err(AppError::Db)?;

    let (cabecera, salta) = crate::restauracion::leer_la_cabecera(&ruta)?;
    let contrasena = contrasena.filter(|c| !c.is_empty());
    let mut origen =
        crate::restauracion::LeeElArchivo::nuevo(&ruta, &cabecera, salta, contrasena.as_deref())?;

    let paso = db
        .restaurar(
            &mut origen,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance,
            "recuperación",
            crate::respaldo::ahora_utc().as_str(),
        )
        .map_err(AppError::Db)?;

    estado
        .respaldo_elegido
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();

    Ok(Recuperado {
        filas: paso.filas,
        bajas: paso.bajas_despues,
        cuentas_a_reconectar: paso.cuentas_a_reconectar,
        // Sólo el nombre: la ruta no cruza la frontera, y además decir en qué
        // carpeta está diría de qué equipo se trata.
        nombre_de_la_apartada: apartada
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
    })
}

/// Lo que quedó después de recuperar.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recuperado {
    pub filas: u64,
    pub bajas: u64,
    pub cuentas_a_reconectar: u64,
    /// Cómo se llama ahora la base vieja. **Se dice**, para que quien recupere
    /// su llavero más tarde sepa qué archivo buscar.
    pub nombre_de_la_apartada: String,
}

/// Aparta la base que no se puede leer, con fecha en el nombre.
///
/// **No la borra, y eso no es prudencia de más.** Un llavero puede volver —un
/// Keychain restaurado, un perfil de usuario que se recupera— y el día que
/// vuelva esos datos valen más que el respaldo, porque son los de hoy.
///
/// Si ya existe un archivo con ese nombre se le añade un sufijo: recuperar dos
/// veces el mismo día no puede pisar la copia de la primera.
///
/// # Errores
///
/// [`AppError::ArchivoNoSePudoLeer`] si no se puede renombrar. **Y entonces no
/// se sigue**: crear la clave nueva con la base vieja todavía en su sitio la
/// dejaría ilegible y sin nombre que la distinga.
pub fn apartar_la_base_ilegible(base: &Path, marca: &str) -> Result<PathBuf, AppError> {
    if !base.exists() {
        // No había base: es el caso de un llavero vaciado sin datos detrás.
        // No hay nada que apartar y tampoco es un error.
        return Ok(base.to_path_buf());
    }
    let carpeta = base.parent().unwrap_or(Path::new("."));
    let mut destino = carpeta.join(format!("arles-ilegible-{marca}.db"));
    let mut n = 2;
    while destino.exists() {
        destino = carpeta.join(format!("arles-ilegible-{marca}-{n}.db"));
        n += 1;
        if n > 100 {
            return Err(AppError::ArchivoNoSePudoLeer);
        }
    }
    std::fs::rename(base, &destino).map_err(|_| AppError::ArchivoNoSePudoLeer)?;
    Ok(destino)
}

/// La marca de tiempo que va en el nombre del archivo apartado.
fn ahora_para_nombre() -> String {
    crate::respaldo::ahora_utc()
        .get(..10)
        .unwrap_or("sin-fecha")
        .replace('-', "")
}

/// Arranca ARLES en modo de recuperación: una ventana, tres comandos y nada más.
///
/// Los comandos normales **no se registran**. No es una omisión: ninguno puede
/// funcionar sin base, y un comando registrado que siempre falla es una puerta
/// que nadie vigila (§9).
pub fn modo_recuperacion(rutas: Rutas, llavero: Llavero) {
    let estado = EstadoDeRecuperacion::nuevo(rutas, llavero);

    let resultado = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // ── CÓMO SABE LA INTERFAZ QUE ESTÁ AQUÍ ───────────────────────────
        //
        // Con una marca puesta **antes** de que cargue nada, y no preguntando.
        // La alternativa —que el frontend llame a un comando y vea si falla—
        // deja un error en la consola en el arranque normal, y este proyecto
        // tiene sondas que exigen la consola limpia: una comprobación que se
        // acostumbra a ver rojo deja de servir.
        .setup(|app| {
            use tauri::Manager as _;
            if let Some(v) = app.get_webview_window("main") {
                let _ = v.eval("window.__ARLES_RECUPERACION__ = true");
            }
            Ok(())
        })
        .manage(estado)
        .invoke_handler(tauri::generate_handler![
            que_paso_con_la_clave,
            revisar_respaldo_para_recuperar,
            recuperar_desde_respaldo,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = resultado {
        crate::mostrar_fallo_de_arranque(&AppError::Db(arles_db::DbError::Migracion(
            e.to_string(),
        )));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **La base ilegible se aparta, nunca se borra.**
    ///
    /// Es la garantía que separa un problema recuperable de uno que no lo es.
    /// Un llavero puede volver —un Keychain restaurado de una copia, un perfil
    /// de usuario que se recupera—, y el día que vuelva esos datos valen más
    /// que el respaldo, porque son los de hoy.
    ///
    /// Verificada el 06/10/2026 cambiando el `rename` por un `remove_file`: el
    /// contenido desaparece y la prueba lo nombra.
    #[test]
    fn la_base_ilegible_se_aparta_y_no_se_pierde() {
        let dir = tempfile::tempdir().expect("temporal");
        let base = dir.path().join("arles.db");
        std::fs::write(&base, b"cifrado con una clave que ya no existe").expect("escribe");

        let apartada = apartar_la_base_ilegible(&base, "20261006").expect("aparta");

        assert!(!base.exists(), "la base tenía que dejar su sitio libre");
        assert!(apartada.exists(), "la base apartada no existe: se perdió");
        assert_eq!(
            std::fs::read(&apartada).expect("lee"),
            b"cifrado con una clave que ya no existe",
            "el contenido de la base apartada cambió"
        );
        assert!(
            apartada
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("ilegible")),
            "el nombre no dice que es la base que no se pudo leer"
        );
    }

    /// **Recuperar dos veces el mismo día no pisa la copia de la primera.**
    ///
    /// Suena a detalle y es la diferencia entre conservar los datos y
    /// conservarlos una vez. Alguien que lo intenta, se equivoca de respaldo y
    /// lo vuelve a intentar borraría con el segundo intento lo que el primero
    /// había salvado.
    ///
    /// Verificada el 06/10/2026 quitando el bucle del sufijo: el segundo
    /// `rename` machaca el archivo del primero y la prueba lo nombra.
    #[test]
    fn apartar_dos_veces_el_mismo_dia_no_pisa_nada() {
        let dir = tempfile::tempdir().expect("temporal");
        let base = dir.path().join("arles.db");

        std::fs::write(&base, b"la primera").expect("escribe");
        let primera = apartar_la_base_ilegible(&base, "20261006").expect("aparta");

        std::fs::write(&base, b"la segunda").expect("escribe");
        let segunda = apartar_la_base_ilegible(&base, "20261006").expect("aparta");

        assert_ne!(primera, segunda, "las dos copias fueron al mismo nombre");
        assert_eq!(std::fs::read(&primera).expect("lee"), b"la primera");
        assert_eq!(std::fs::read(&segunda).expect("lee"), b"la segunda");
    }

    /// Sin base que apartar, no es un error.
    ///
    /// Es el caso de un llavero vaciado sin datos detrás: no hay nada que
    /// salvar y la recuperación tiene que poder seguir.
    #[test]
    fn sin_base_no_hay_nada_que_apartar() {
        let dir = tempfile::tempdir().expect("temporal");
        let base = dir.path().join("arles.db");
        assert!(apartar_la_base_ilegible(&base, "20261006").is_ok());
    }

    /// **El camino entero de la recuperación, sin depender del llavero.**
    ///
    /// Es la prueba que el plan de la 8.4 pide: se pierde el acceso a la base,
    /// se recupera desde un respaldo, y **lo viejo no se destruye**.
    ///
    /// ── POR QUÉ NO USA EL LLAVERO DEL SISTEMA ──
    ///
    /// Porque entonces, aquí, no probaría nada. El contenedor de integración no
    /// tiene D-Bus, así que `Llavero::crear` falla y la prueba se iría por la
    /// rama de «sin llavero» — pasando en verde sin haber ejercitado una línea.
    /// Es el patrón de «omitida no es pasada» aplicado a sí mismo.
    ///
    /// Así que la clave se maneja directamente: lo que se comprueba es la
    /// mecánica de la recuperación, que es lo que esta entrega añade. Que
    /// `arrancar_con` no invente una clave lo comprueba
    /// `arrancar_sin_clave_y_con_base_no_improvisa`, que sí declara su rama.
    ///
    /// Verificada el 06/10/2026 apartando la base con un `remove_file`: los
    /// contactos vuelven igual —el respaldo los trae— y la prueba falla en el
    /// paso 5, que es el único que vigila lo irrecuperable.
    #[test]
    fn recuperar_devuelve_los_datos_y_no_destruye_la_base_vieja() {
        use arles_db::{ClaveMaestra, Db, QueSeLleva};

        let dir = tempfile::tempdir().expect("temporal");
        let base = dir.path().join("arles.db");
        let respaldo = dir.path().join("copia.arles");

        // ── 1 · Una base con datos, y un respaldo de ella ──
        let clave = ClaveMaestra::generar().expect("genera");
        {
            let db = Db::abrir(&base, &clave).expect("abre");
            db.ejecutar_en_pruebas(|c| {
                c.execute_batch(
                    "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
                       VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                     INSERT INTO import_batch
                       (id, company_id, original_filename, stored_filename, file_hash, column_mapping,
                        total_rows, consent_affirmation, created_at)
                       VALUES ('L1', 'E1', 'lista.csv', 'L1.csv', 'abc', '{}', 1, 'sí', '2026-01-02T00:00:00Z');
                     INSERT INTO contact (id, company_id, import_batch_id, first_name, last_name, created_at, updated_at)
                       VALUES ('C1', 'E1', 'L1', 'Ana', 'Ruiz', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');",
                )
            })
            .expect("siembra");

            let mut destino = crate::respaldo::ElArchivo::nuevo(
                &respaldo,
                "completo",
                None,
                Box::new(|_| {}),
                "2026-10-06T12:00:00Z".to_owned(),
            )
            .expect("abre el archivo");
            db.exportar(QueSeLleva::Todo, &mut destino)
                .expect("exporta");
            destino.cerrar().expect("cierra");
        }
        let bytes_antes = std::fs::metadata(&base).expect("metadatos").len();

        // ── 2 · La clave ya no sirve: la base es ilegible ──
        let otra = ClaveMaestra::generar().expect("genera");
        assert!(
            Db::abrir(&base, &otra).is_err(),
            "la base se abrió con otra clave: no estaba cifrada"
        );

        // ── 3 · La salida: apartar, clave nueva, restaurar ──
        let apartada = apartar_la_base_ilegible(&base, "20261006").expect("aparta");
        let clave_nueva = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&base, &clave_nueva).expect("abre la base nueva");

        let (cabecera, salta) = crate::restauracion::leer_la_cabecera(&respaldo).expect("cabecera");
        let mut origen =
            crate::restauracion::LeeElArchivo::nuevo(&respaldo, &cabecera, salta, None)
                .expect("lector");
        db.restaurar(
            &mut origen,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance,
            "recuperación",
            "2026-10-06T13:00:00Z",
        )
        .expect("recupera");

        // ── 4 · Los datos volvieron ──
        let nombre: String = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT first_name FROM contact WHERE id='C1'", [], |f| {
                    f.get(0)
                })
            })
            .expect("lee");
        assert_eq!(nombre, "Ana");

        // ── 5 · Y lo viejo sigue ahí, intacto ──
        //
        // Es el único paso que vigila algo irrecuperable: si la llave vieja
        // aparece algún día, este archivo es lo único que la aprovecha.
        assert!(apartada.exists(), "la base ilegible se perdió al recuperar");
        assert_eq!(
            std::fs::metadata(&apartada).expect("metadatos").len(),
            bytes_antes,
            "la base apartada cambió de tamaño: algo la tocó"
        );
    }

    /// **Arrancar sin clave y con una base en disco no improvisa una nueva.**
    ///
    /// Es el hallazgo F2 de la Fase 1 y el error que más caro saldría aquí:
    /// una clave nueva dejaría la base existente ilegible **para siempre**, y
    /// nadie se enteraría hasta mucho después.
    ///
    /// Esta sí necesita el llavero del sistema, así que **declara su rama**. En
    /// el contenedor de integración no hay D-Bus y se salta: eso es «omitida»,
    /// que no es lo mismo que «pasada», y por eso lo dice en voz alta. En
    /// Windows y macOS se prueba el día que se cierre R-07.
    #[test]
    fn arrancar_sin_clave_y_con_base_no_improvisa() {
        use crate::estado::EstadoApp;

        let dir = tempfile::tempdir().expect("temporal");
        let rutas = Rutas::en(dir.path().to_path_buf());
        let llavero = Llavero::con_cuenta(format!("prueba-8-4-{}", std::process::id()));

        let Ok(clave) = llavero.crear() else {
            println!("RAMA: sin llavero, no se puede ejercitar el arranque — OMITIDA");
            return;
        };
        println!("RAMA: llavero disponible, arranque sin clave ejercitado");

        let base = rutas.base_de_datos();
        drop(arles_db::Db::abrir(&base, &clave).expect("abre"));
        llavero.borrar().expect("borra la entrada");

        let intento = EstadoApp::arrancar_con(&rutas, &llavero);
        assert!(
            matches!(intento, Err(AppError::ClaveMaestraPerdida)),
            "arrancar sin clave y con base en disco tiene que parar: {intento:?}"
        );
        assert!(
            llavero.leer().expect("lee").is_none(),
            "el arranque creó una clave nueva: la base vieja queda ilegible para siempre (F2)"
        );
        let _ = llavero.borrar();
    }
}
