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
    // ── EL VEREDICTO MIRA DOS COSAS, Y ANTES MIRABA UNA ──────────────────
    //
    // Hasta la auditoría de la Fase 8 (07/10/2026) esto era sólo la comparación
    // de esquemas, y el campo se llama `se_puede_restaurar`. Así que ante un
    // **archivo de historial** con el esquema de hoy la pantalla decía que sí
    // se podía: quien hubiera perdido el llavero y eligiera por error el
    // archivo del disco externo recibía luz verde, confirmaba, y el comando
    // apartaba la base y creaba la clave nueva antes de que `restaurar`
    // rechazara el archivo por su alcance.
    //
    // Un archivo de historial no se puede restaurar **nunca** (D-8C y la 8.3
    // bis a): reemplazar con él vaciaría la base entera. El restaurador ya lo
    // rechaza; lo que faltaba era decirlo **antes** de que alguien confirme.
    let se_puede = cabecera.esquema == esquema_de_la_base && cabecera.alcance.se_puede_restaurar();

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
        alcance: cabecera.alcance.as_str(),
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

    // ── 1 · PRIMERO SE COMPRUEBA EL RESPALDO. NADA SE TOCA TODAVÍA ───────
    //
    // ── POR QUÉ ESTE ORDEN, Y QUÉ PASABA CON EL ANTERIOR ────────────────
    //
    // Hasta la auditoría de la Fase 8 (07/10/2026) este comando apartaba la
    // base y **creaba la clave nueva** antes de mirar el respaldo. Con una
    // contraseña mal escrita —y el respaldo va cifrado por defecto (D-8A)— el
    // resultado era:
    //
    //   · la base ilegible renombrada, y su nombre NO se decía, porque el
    //     comando devolvía `Err` y `nombre_de_la_apartada` sólo viaja en el
    //     `Ok`: quien se equivocara al teclear se quedaba sin saber dónde
    //     estaba su archivo;
    //   · **una clave nueva escrita en el llavero**, que es exactamente lo que
    //     el F2 de la Fase 1 prohíbe. El llavero puede estar vacío por motivos
    //     pasajeros —un Keychain bloqueado, un D-Bus que no levantó— y la
    //     regla dice que la clave sólo nace cuando alguien elige un respaldo
    //     **y la recuperación sale bien**. Una contraseña mal teclada
    //     convertía un problema reversible en uno definitivo.
    //
    // El sobre **no puede validar la contraseña al abrirse**: hace falta
    // descifrar un trozo y comprobar su etiqueta. Así que se descifra el
    // primero aquí, a propósito, y sólo entonces se toca algo.
    let (cabecera, salta) = crate::restauracion::leer_la_cabecera(&ruta)?;
    let contrasena = contrasena.filter(|c| !c.is_empty());
    comprobar_el_respaldo_antes_de_tocar_nada(&ruta, &cabecera, salta, contrasena.as_deref())?;

    // ── 2 · Y AHORA SÍ: apartar, crear la clave y restaurar ─────────────
    let base = estado.rutas.base_de_datos();
    let apartada = apartar_la_base_ilegible(&base, &ahora_para_nombre())?;

    let clave = estado.llavero.crear()?;
    let db = Db::abrir(&base, &clave).map_err(AppError::Db)?;

    // Se vuelve a abrir desde el principio: el que se usó para comprobar ya
    // gastó el primer trozo.
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

/// Comprueba **todo lo que se puede saber** antes de tocar la base y el llavero.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA REGLA QUE ESTA FUNCIÓN EXISTE PARA CUMPLIR
///
/// **Nada irreversible ocurre hasta que todo lo conocible está comprobado.**
///
/// En esta pantalla lo irreversible son dos cosas: renombrar la base ilegible y
/// **escribir una clave nueva en el llavero**. La segunda es la que no tiene
/// vuelta: el llavero puede estar vacío por un motivo pasajero y arreglarse
/// solo, pero no después de que le escribamos encima (F2 de la Fase 1).
///
/// Así que aquí se comprueban las tres cosas que `Db::restaurar` rechazaría, en
/// orden de lo más barato a lo más caro:
///
/// 1. **El alcance.** Un archivo de historial no se restaura nunca (D-8C): con
///    él, reemplazar vaciaría la base entera. Se sabe de la cabecera.
/// 2. **El esquema.** La base que se va a crear es nueva, así que su esquema es
///    el último; un respaldo de otro esquema no entra, y eso también se sabe de
///    la cabecera.
/// 3. **Que se abra y se descifre.** Pide la primera fila, porque el cifrado
///    autenticado **no se puede comprobar sin descifrar** (8.2a): el sobre no
///    sabe si la contraseña es buena hasta que una etiqueta cuadra.
///
/// Un respaldo de cero filas vale: lo que se comprueba es que el archivo se
/// abre, no que traiga datos. Quien exportó una base recién creada tiene
/// derecho a restaurarla.
///
/// # Errores
///
/// Lo que falle en cualquiera de los tres pasos. **Y entonces no se sigue**,
/// que es todo el punto.
fn comprobar_el_respaldo_antes_de_tocar_nada(
    ruta: &Path,
    cabecera: &arles_core::formato_de_respaldo::Cabecera,
    salta: usize,
    contrasena: Option<&str>,
) -> Result<(), ErrorIpc> {
    use arles_db::restauracion::Origen as _;

    if !cabecera.alcance.se_puede_restaurar() {
        return Err(AppError::Db(arles_db::DbError::EsoEsUnArchivoDeHistorial).into());
    }

    let ultimo = arles_db::ultima_version()
        .map(|v| format!("V{v}"))
        .unwrap_or_default();
    if cabecera.esquema != ultimo {
        let del_respaldo = &cabecera.esquema;
        return Err(AppError::Db(arles_db::DbError::RespaldoNoEncaja {
            motivo: format!(
                "el respaldo es del esquema {del_respaldo} y esta base va por el {ultimo}"
            ),
        })
        .into());
    }

    let mut origen = crate::restauracion::LeeElArchivo::nuevo(ruta, cabecera, salta, contrasena)?;
    origen.siguiente().map_err(AppError::Db)?;
    Ok(())
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

    /// **El camino que D-8A hace el más probable: el respaldo va cifrado.**
    ///
    /// ── POR QUÉ ESTA PRUEBA ES LA QUE FALTABA ────────────────────────────
    ///
    /// Tres decisiones se cruzan justo aquí, y ninguna prueba cruzaba las tres:
    ///
    ///   · **D-8A** · el respaldo va cifrado, y es el valor por defecto.
    ///   · **8.4** · perder el llavero se arregla restaurando un respaldo.
    ///   · **8.2a** · la contraseña del respaldo **no se guarda en ningún
    ///     sitio**.
    ///
    /// Así que el desastre realista no es «perdí el llavero»: es «perdí el
    /// llavero y mi único respaldo está cifrado». El comando aceptaba una
    /// contraseña desde la 8.4 y **nunca se había probado con una**: el camino
    /// existía y nadie lo había recorrido.
    ///
    /// Encontrada por la auditoría de la Fase 8, 07/10/2026.
    #[test]
    fn se_recupera_de_un_respaldo_cifrado_con_su_contrasena() {
        use arles_db::{ClaveMaestra, Db, QueSeLleva};

        let dir = tempfile::tempdir().expect("temporal");
        let base = dir.path().join("arles.db");
        let respaldo = dir.path().join("copia.arles");
        const CLAVE_DEL_RESPALDO: &str = "una contraseña larga de verdad";

        let clave = ClaveMaestra::generar().expect("genera");
        {
            let db = Db::abrir(&base, &clave).expect("abre");
            sembrar(&db);
            let mut destino = crate::respaldo::ElArchivo::nuevo(
                &respaldo,
                "completo",
                Some(CLAVE_DEL_RESPALDO),
                Box::new(|_| {}),
                "2026-10-07T12:00:00Z".to_owned(),
            )
            .expect("abre el archivo");
            db.exportar(QueSeLleva::Todo, &mut destino)
                .expect("exporta");
            destino.cerrar().expect("cierra");
        }

        // Que esté cifrado de verdad, no que lo diga la cabecera: si el cuerpo
        // saliera en claro, el resto de la prueba pasaría igual y no
        // demostraría nada.
        let crudo = std::fs::read(&respaldo).expect("lee");
        assert!(
            !String::from_utf8_lossy(&crudo).contains("Ana"),
            "el respaldo no está cifrado: el nombre se lee en el archivo"
        );

        let (cabecera, salta) = crate::restauracion::leer_la_cabecera(&respaldo).expect("cabecera");
        assert!(
            cabecera.cifrado.lleva_contrasena(),
            "la cabecera tiene que decir que hace falta contraseña, en claro"
        );

        // La comprobación previa pasa con la buena…
        comprobar_el_respaldo_antes_de_tocar_nada(
            &respaldo,
            &cabecera,
            salta,
            Some(CLAVE_DEL_RESPALDO),
        )
        .expect("con la contraseña buena el respaldo se abre");

        // …y restaura.
        let apartada = apartar_la_base_ilegible(&base, "20261007").expect("aparta");
        let clave_nueva = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&base, &clave_nueva).expect("abre la base nueva");
        let mut origen = crate::restauracion::LeeElArchivo::nuevo(
            &respaldo,
            &cabecera,
            salta,
            Some(CLAVE_DEL_RESPALDO),
        )
        .expect("lector");
        db.restaurar(
            &mut origen,
            &cabecera.esquema,
            &cabecera.columnas,
            cabecera.alcance,
            "recuperación",
            "2026-10-07T13:00:00Z",
        )
        .expect("recupera del respaldo cifrado");

        let nombre: String = db
            .ejecutar_en_pruebas(|c| {
                c.query_row("SELECT first_name FROM contact WHERE id='C1'", [], |f| {
                    f.get(0)
                })
            })
            .expect("lee");
        assert_eq!(nombre, "Ana");
        assert!(apartada.exists(), "la base ilegible se perdió");
    }

    /// **Una contraseña mal escrita no toca la base ni el llavero.**
    ///
    /// ── EL FALLO QUE ESTO ARREGLA, Y POR QUÉ ERA GRAVE ───────────────────
    ///
    /// Hasta el 07/10/2026 `recuperar_desde_respaldo` apartaba la base y
    /// **creaba la clave nueva** antes de mirar el respaldo. El sobre no puede
    /// validar la contraseña al abrirse —hace falta descifrar un trozo— así que
    /// una contraseña mal teclada fallaba **después** de:
    ///
    ///   · renombrar la base ilegible, sin decir cómo se llama ahora: el nombre
    ///     sólo viaja en el `Ok`, y esto devolvía `Err`;
    ///   · **escribir una clave nueva en el llavero**, que es justo lo que el
    ///     F2 de la Fase 1 prohíbe. Un llavero vacío por un motivo pasajero
    ///     —Keychain bloqueado, D-Bus que no levantó— se arregla solo; una
    ///     clave nueva encima, no.
    ///
    /// Es decir: **un error de tecleo convertía un problema reversible en uno
    /// definitivo**, y en la pantalla a la que se llega el día que ya se ha
    /// perdido algo.
    ///
    /// Esto comprueba la garantía nueva: con la contraseña mala, la
    /// comprobación previa falla y **nada se ha movido**.
    ///
    /// Verificada quitando la llamada a `comprobar_el_respaldo_antes_de_tocar_nada` del
    /// comando: la contraseña mala pasa el filtro y revienta dentro de
    /// `restaurar`, con la base ya apartada.
    #[test]
    fn con_la_contrasena_mal_escrita_no_se_toca_nada() {
        use arles_db::{ClaveMaestra, Db, QueSeLleva};

        let dir = tempfile::tempdir().expect("temporal");
        let base = dir.path().join("arles.db");
        let respaldo = dir.path().join("copia.arles");

        let clave = ClaveMaestra::generar().expect("genera");
        {
            let db = Db::abrir(&base, &clave).expect("abre");
            sembrar(&db);
            let mut destino = crate::respaldo::ElArchivo::nuevo(
                &respaldo,
                "completo",
                Some("la buena"),
                Box::new(|_| {}),
                "2026-10-07T12:00:00Z".to_owned(),
            )
            .expect("abre el archivo");
            db.exportar(QueSeLleva::Todo, &mut destino)
                .expect("exporta");
            destino.cerrar().expect("cierra");
        }
        let bytes_antes = std::fs::metadata(&base).expect("metadatos").len();

        let (cabecera, salta) = crate::restauracion::leer_la_cabecera(&respaldo).expect("cabecera");

        // La comprobación previa rechaza la contraseña mala.
        assert!(
            comprobar_el_respaldo_antes_de_tocar_nada(&respaldo, &cabecera, salta, Some("la mala"))
                .is_err(),
            "una contraseña equivocada tiene que rechazarse ANTES de tocar nada"
        );

        // Y sin contraseña también, que es el otro descuido posible.
        assert!(
            comprobar_el_respaldo_antes_de_tocar_nada(&respaldo, &cabecera, salta, None).is_err(),
            "un respaldo cifrado sin contraseña tiene que rechazarse"
        );

        // Lo que importa: la base sigue donde estaba, con su tamaño.
        assert!(base.exists(), "la base se apartó con una contraseña mala");
        assert_eq!(
            std::fs::metadata(&base).expect("metadatos").len(),
            bytes_antes,
            "la base cambió de tamaño con una contraseña mala"
        );
        // Y no quedó ninguna base apartada rondando.
        let apartadas = std::fs::read_dir(dir.path())
            .expect("lee la carpeta")
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("arles-ilegible-")
            })
            .count();
        assert_eq!(apartadas, 0, "se apartó una base con la contraseña mala");
    }

    /// Siembra una empresa, un lote y un contacto. Lo mismo en tres pruebas.
    fn sembrar(db: &arles_db::Db) {
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

    /// Una cabecera cualquiera, para ejercitar las dos puertas que se contestan
    /// **sin abrir el archivo**.
    fn cabecera_de(
        alcance: arles_core::formato_de_respaldo::Alcance,
        esquema: &str,
    ) -> arles_core::formato_de_respaldo::Cabecera {
        use arles_core::formato_de_respaldo::{Cabecera, Cifrado};
        Cabecera {
            formato: arles_core::formato_de_respaldo::VERSION_DEL_FORMATO,
            esquema: esquema.to_owned(),
            alcance,
            creado_en: "2026-10-07T12:00:00Z".to_owned(),
            cifrado: Cifrado::No,
            filas: Vec::new(),
            columnas: Vec::new(),
        }
    }

    /// **Un archivo de historial dado a la recuperación no llega a tocar nada.**
    ///
    /// ── EL SEGUNDO DISPARADOR DEL MISMO FALLO ────────────────────────────
    ///
    /// `Db::restaurar` rechaza tres cosas, y las tres se saben **de la cabecera**
    /// antes de tocar la base: el alcance, el esquema y que el archivo se abra.
    /// La primera versión de la comprobación previa sólo cubría la tercera, así
    /// que elegir por error el archivo del disco externo —el escenario realista:
    /// quien perdió el llavero busca «el archivo de ARLES» y encuentra dos—
    /// apartaba la base y **escribía una clave nueva en el llavero** antes de
    /// que `restaurar` lo rechazara. Eso es el F2 de la Fase 1, que esta misma
    /// pantalla existe para no cometer.
    ///
    /// La ruta **no existe a propósito**: si la comprobación abriera el archivo
    /// primero, el error sería de E/S y no éste. Que salga
    /// `EsoEsUnArchivoDeHistorial` sobre una ruta inexistente demuestra que la
    /// puerta está antes de cualquier apertura.
    ///
    /// Verificada quitando la puerta del alcance: el error pasa a ser de E/S y
    /// la prueba cae.
    ///
    /// **Se compara la clave de i18n, no el nombre de la variante.** La primera
    /// versión buscaba `EsoEsUnArchivoDeHistorial` en el `Debug` del error y
    /// salió en rojo sobre código correcto: `ErrorIpc` no lleva nombres de Rust
    /// al otro lado de la frontera —lleva la clave y el detalle traducido—, que
    /// es precisamente lo que el §139 pide. La clave es además lo que el usuario
    /// acaba leyendo, así que es la garantía que importa.
    ///
    /// Encontrada por la auditoría de la Fase 8, 07/10/2026.
    #[test]
    fn un_archivo_de_historial_no_llega_a_tocar_nada() {
        use arles_core::formato_de_respaldo::Alcance;

        let cabecera = cabecera_de(
            Alcance::Historial {
                hasta: "2025-10-07T00:00:00Z".to_owned(),
            },
            &arles_db::ultima_version()
                .map(|v| format!("V{v}"))
                .unwrap_or_default(),
        );
        let error = comprobar_el_respaldo_antes_de_tocar_nada(
            Path::new("/no/existe/archivo-de-historial.arles"),
            &cabecera,
            0,
            None,
        )
        .expect_err("un archivo de historial tiene que rechazarse");

        assert!(
            error.clave == "error.db.eso_es_un_archivo_de_historial",
            "se rechazó, pero por otro motivo — la puerta del alcance no está antes de abrir: {error:?}"
        );
    }

    /// **Un respaldo de otro esquema tampoco llega a tocar nada.**
    ///
    /// La base que la recuperación va a crear es **nueva**, así que su esquema
    /// es siempre el último; un respaldo anterior no entra (8.3b) y hoy no hay
    /// forma de migrarlo. Saberlo cuesta una comparación de cadenas, y no
    /// saberlo cuesta una clave nueva en el llavero.
    ///
    /// El `"V1"` es deliberado: cualquier migración futura lo mantiene distinto
    /// del último, que es lo que la prueba necesita. Verificada quitando la
    /// comparación de esquemas.
    #[test]
    fn un_respaldo_de_otro_esquema_no_llega_a_tocar_nada() {
        use arles_core::formato_de_respaldo::Alcance;

        let cabecera = cabecera_de(Alcance::Completo, "V1");
        let error = comprobar_el_respaldo_antes_de_tocar_nada(
            Path::new("/no/existe/respaldo-viejo.arles"),
            &cabecera,
            0,
            None,
        )
        .expect_err("un respaldo de otro esquema tiene que rechazarse");

        assert!(
            error.clave == "error.db.respaldo_no_encaja",
            "se rechazó, pero por otro motivo — la comparación de esquemas no está antes de abrir: {error:?}"
        );
    }
}
