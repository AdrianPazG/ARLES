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

/// El único sitio externo que ARLES sabe abrir.
///
/// Confirmado por Dirección para el logotipo de TELEMETRY del pie.
pub const SITIO_DE_TELEMETRY: &str = "https://telemetrymx.com";

/// Abre el sitio de TELEMETRY en el navegador del sistema.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTE COMANDO NO RECIBE UNA URL
///
/// Lo normal sería un plugin de shell con un permiso `allow-open` y un ámbito
/// que valide la URL contra una expresión regular. Eso deja **la webview
/// eligiendo el destino** y la seguridad dependiendo de que la expresión esté
/// bien escrita; una regular mal anclada convierte «abrir el sitio de la
/// empresa» en «abrir cualquier cosa», que es una de las rutas clásicas para
/// ejecutar algo en la máquina del usuario.
///
/// Aquí el destino es una **constante compilada**. El frontend no puede pasar
/// otra URL porque el comando no tiene parámetros. No hay ámbito que validar,
/// no hay expresión regular que revisar, y no hace falta un plugin más.
///
/// Tampoco se navega dentro de la ventana: una WebView que navega a internet
/// deja de ser una aplicación y pasa a ser un navegador sin barra de
/// direcciones, donde el usuario no puede saber dónde está.
/// ─────────────────────────────────────────────────────────────────────────
#[tauri::command]
pub fn abrir_sitio_de_telemetry() -> Result<(), ErrorIpc> {
    abrir_en_el_navegador(SITIO_DE_TELEMETRY)
}

/// Lanza el navegador predeterminado del sistema.
///
/// No interpola nada: `destino` es siempre [`SITIO_DE_TELEMETRY`], una
/// constante. Si algún día alguien le pasara algo dinámico, este comentario es
/// el sitio donde se dará cuenta de que hay que sanear antes.
fn abrir_en_el_navegador(destino: &str) -> Result<(), ErrorIpc> {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    let mut orden = {
        let mut c = Command::new("cmd");
        // El «» vacío es el TÍTULO de la ventana que espera `start`. Sin él,
        // `start` toma el primer argumento entrecomillado como título y no
        // abre nada — un fallo silencioso y difícil de ver.
        c.args(["/C", "start", "", destino]);
        c
    };

    #[cfg(target_os = "macos")]
    let mut orden = {
        let mut c = Command::new("open");
        c.arg(destino);
        c
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut orden = {
        let mut c = Command::new("xdg-open");
        c.arg(destino);
        c
    };

    orden
        .spawn()
        .map(|_| ())
        .map_err(|e| crate::error::AppError::SitioNoAbre(e.to_string()).into())
}

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

    /// **El motor está enviando.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ ESTO TIENE QUE CRUZAR LA FRONTERA
    ///
    /// `false` significa que hay otro ARLES abierto sobre esta misma base: el
    /// cerrojo de instancia única lo tiene el otro, y éste no envía.
    ///
    /// Sin este dato, la pantalla no puede distinguir «la campaña va despacio»
    /// de «esta ventana no está enviando nada». Las dos se ven igual —una
    /// campaña en marcha cuyo contador no sube— y la segunda no se arregla
    /// esperando. Una campaña que no avanza sin explicación se lee como una
    /// avería (§95).
    ///
    /// Va en este comando y no en uno propio por la regla 1 de la frontera:
    /// comandos gruesos. Es un dato del arranque como los otros tres.
    pub el_motor_envia: bool,
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
        el_motor_envia: estado.el_motor_esta_en_marcha(),
    })
}

/// Todo lo que la pantalla de configuración necesita, en **un solo cruce**.
///
/// Regla 1 de la frontera: comandos gruesos. Pedir la empresa, las zonas, los
/// países y la lista de alta por separado son cuatro viajes y cuatro momentos
/// en que la pantalla puede quedar a medias mostrando un estado que ya cambió.
///
/// Las listas de zonas y países las aporta **el núcleo**, no la interfaz. Es lo
/// que impide que el desplegable ofrezca una opción que el validador rechaza:
/// son el mismo dato, no dos copias.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguracionDeEmpresa {
    /// `None` en una instalación nueva: toca el alta (§25).
    pub empresa: Option<arles_core::DatosDeEmpresa>,
    pub zonas: &'static [&'static str],
    pub paises: &'static [&'static str],
    pub onboarding: arles_core::ListaDeOnboarding,
}

fn configuracion(estado: &EstadoApp) -> Result<ConfiguracionDeEmpresa, crate::AppError> {
    let empresa = estado.db().empresa()?.map(|e| e.datos);
    let recuento = estado.db().recuento_de_alta()?;
    Ok(ConfiguracionDeEmpresa {
        empresa,
        zonas: arles_core::ZONAS_SOPORTADAS,
        paises: arles_core::PAISES_SOPORTADOS,
        onboarding: arles_core::ListaDeOnboarding::desde(&recuento),
    })
}

/// Lee la configuración de empresa y la lista de alta.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde o si lo almacenado no pasa la
/// validación del dominio.
#[tauri::command]
pub fn configuracion_de_empresa(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<ConfiguracionDeEmpresa, ErrorIpc> {
    configuracion(&estado).map_err(ErrorIpc::from)
}

/// Valida y guarda los datos de la empresa.
///
/// Devuelve **la misma configuración que se lee al abrir**, no un `()`: la
/// lista de alta cambia al guardar, y con un comando que no devuelve nada la
/// interfaz tendría que pedirla otra vez —otro cruce, y un instante en que la
/// pantalla dice que el paso sigue pendiente.
///
/// # Errores
///
/// [`ErrorIpc`] con `campos` relleno si el formulario no es válido; con
/// `campos` vacío si el fallo es de la base.
#[tauri::command]
pub fn guardar_empresa(
    estado: tauri::State<'_, EstadoApp>,
    borrador: arles_core::BorradorDeEmpresa,
) -> Result<ConfiguracionDeEmpresa, ErrorIpc> {
    // La validación es del núcleo, no de aquí ni del formulario. El formulario
    // valida para avisar pronto; esta es la que decide.
    let datos = arles_core::DatosDeEmpresa::validar(&borrador)
        .map_err(|campos| ErrorIpc::from(crate::AppError::EmpresaInvalida(campos)))?;

    estado
        .db()
        .guardar_empresa(&datos)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    configuracion(&estado).map_err(ErrorIpc::from)
}

/// Preferencias de interfaz que se recuerdan entre sesiones (P-11).
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferenciasDeInterfaz {
    /// `true` si la barra lateral quedó plegada la última vez.
    pub barra_lateral_plegada: bool,
    /// Tema elegido: `auto`, `oscuro` o `claro`. Ver [`TEMAS`].
    pub tema: String,
}

/// Claves de las preferencias de v1.2.0. La lista cerrada vive en
/// `arles_db::CLAVES_DE_INTERFAZ`, y el test de abajo comprueba que estas
/// constantes siguen estando en ella.
const BARRA_PLEGADA: &str = "barra_lateral_plegada";
const TEMA: &str = "tema";

/// Los tres temas admitidos.
///
/// `auto` no es un tema: es «el que pida el sistema operativo». Quién lo
/// resuelve es la interfaz, que es la única que puede preguntárselo al sistema
/// (`prefers-color-scheme`). Aquí se guarda la palabra, no el resultado — si
/// guardáramos el resultado, alguien que cambia su sistema a claro por la noche
/// reabriría ARLES en oscuro sin entender por qué.
pub const TEMAS: &[&str] = &["auto", "oscuro", "claro"];

/// C-2: mientras el usuario no elija, ARLES hace lo que haga el sistema.
pub const TEMA_POR_DEFECTO: &str = "auto";

/// Lee las preferencias de interfaz.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde.
#[tauri::command]
pub fn preferencias_de_interfaz(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<PreferenciasDeInterfaz, ErrorIpc> {
    let db = estado.db();
    let plegada = db
        .preferencia(BARRA_PLEGADA)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;
    let tema = db
        .preferencia(TEMA)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(PreferenciasDeInterfaz {
        // Cualquier cosa que no sea «1» es desplegada. Una preferencia de
        // interfaz corrupta no es motivo para no abrir la aplicación.
        barra_lateral_plegada: plegada.as_deref() == Some("1"),
        // Y lo mismo con el tema: un valor que no reconocemos cae en el de por
        // defecto en vez de propagarse a la interfaz, donde acabaría escrito
        // tal cual en `data-tema` y dejaría la aplicación sin ningún tema.
        tema: match tema {
            Some(t) if TEMAS.contains(&t.as_str()) => t,
            _ => TEMA_POR_DEFECTO.to_owned(),
        },
    })
}

/// Recuerda si la barra lateral queda plegada.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde.
#[tauri::command]
pub fn guardar_barra_plegada(
    estado: tauri::State<'_, EstadoApp>,
    plegada: bool,
) -> Result<(), ErrorIpc> {
    estado
        .db()
        .guardar_preferencia(BARRA_PLEGADA, if plegada { "1" } else { "0" })
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

/// Recuerda el tema elegido (C-1).
///
/// # Errores
///
/// [`ErrorIpc`] si el tema no es uno de [`TEMAS`], o si la base no responde.
#[tauri::command]
pub fn guardar_tema(estado: tauri::State<'_, EstadoApp>, tema: String) -> Result<(), ErrorIpc> {
    // La webview manda la cadena, así que la cadena se comprueba aquí. Sin esto
    // el desplegable es de tres opciones pero el comando es de infinitas, y lo
    // que acabe en la base es lo que alguien escriba al otro lado de la IPC.
    if !TEMAS.contains(&tema.as_str()) {
        return Err(ErrorIpc::from(crate::AppError::TemaDesconocido));
    }

    estado
        .db()
        .guardar_preferencia(TEMA, &tema)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

// ─────────────────────────────────────────────────────────────────────────
// CONTACTOS (entrega 3.2)
//
// Los cuatro comandos que la pantalla de CONTACTOS necesita. Todos exigen que
// la empresa esté configurada: en v1.2.0 hay una sola (D-4), y el
// identificador **no** viaja desde la webview. Si lo hiciera, el frontend
// podría pedir los contactos de cualquier empresa pasando otro UUID; que hoy
// sólo haya una no es una defensa, es una coincidencia que dejará de serlo en
// v1.3.
// ─────────────────────────────────────────────────────────────────────────

/// Cuántos contactos pide la tabla por pantallazo.
///
/// La tabla es virtualizada: trae de más para que al desplazar no se vea el
/// hueco, y muy por debajo del tope de `arles_db::MAX_POR_PAGINA`.
pub const POR_PAGINA: u32 = 100;

/// La página que pide la tabla tiene que caber en el tope de la capa de datos.
///
/// Si alguien subiera `POR_PAGINA` por encima, `listar_contactos` lo recortaría
/// **en silencio** y la tabla creería que ha llegado al final de la lista.
///
/// Va como comprobación de compilación y no como prueba: los dos valores son
/// constantes, así que esto no puede depender de que alguien ejecute los tests.
/// Clippy lo señaló al escribirlo como `assert!` dentro de un `#[test]`, y
/// tenía razón.
const _: () = assert!(POR_PAGINA <= arles_db::MAX_POR_PAGINA);

/// Identificador de la empresa configurada.
///
/// # Errores
///
/// [`AppError::Db`] si la base no responde; [`AppError::EmpresaNoConfigurada`]
/// si todavía no se ha dado de alta.
pub(crate) fn empresa_actual(
    estado: &EstadoApp,
) -> Result<arles_core::ids::CompanyId, crate::AppError> {
    estado
        .db()
        .empresa()?
        .map(|e| e.id)
        .ok_or(crate::AppError::EmpresaNoConfigurada)
}

/// Un contacto tal y como lo ve la pantalla.
///
/// Es un tipo de esta capa y no el de `arles-db` a propósito: el crate de datos
/// no conoce el formato de la IPC, así que la conversión vive aquí. Es el punto
/// donde se decide qué sale hacia la webview —y, sobre todo, qué no.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactoParaLaPantalla {
    pub id: arles_core::ids::ContactId,
    pub nombre: String,
    pub apellido: String,
    pub empresa: String,
    pub canales: Vec<arles_core::contacto::CanalValidado>,
    pub creado_en: String,
    pub actualizado_en: String,
}

impl From<arles_db::ContactoGuardado> for ContactoParaLaPantalla {
    fn from(c: arles_db::ContactoGuardado) -> Self {
        Self {
            id: c.id,
            nombre: c.datos.nombre,
            apellido: c.datos.apellido,
            empresa: c.datos.empresa,
            canales: c.datos.canales,
            creado_en: c.creado_en,
            actualizado_en: c.actualizado_en,
        }
    }
}

/// Una página de contactos, con lo que la tabla necesita para paginar.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginaDeContactos {
    pub contactos: Vec<ContactoParaLaPantalla>,
    /// Cuántos hay en total, no cuántos vienen. La tabla necesita los dos para
    /// decir «100 de 12 480» sin inventarse nada (§65).
    pub total: u32,
    /// Direcciones de esta página que están en la lista de supresión.
    ///
    /// **Es un aviso, no un bloqueo** (L-13): el contacto se guarda igual. Quien
    /// decide si se le escribe es la campaña, en su comprobación previa, y
    /// duplicar aquí esa decisión sería tener dos reglas que mantener iguales.
    pub suprimidas: Vec<String>,
}

/// Una página de la lista de contactos.
///
/// # Errores
///
/// [`ErrorIpc`] si la empresa no está configurada o si la base no responde.
#[tauri::command]
pub fn listar_contactos(
    estado: tauri::State<'_, EstadoApp>,
    desde: u32,
) -> Result<PaginaDeContactos, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let pagina = estado
        .db()
        .listar_contactos(empresa, desde, POR_PAGINA)
        .map_err(crate::AppError::Db)?;

    // Los canales de toda la página en una consulta, no uno por contacto.
    let canales: Vec<_> = pagina
        .contactos
        .iter()
        .flat_map(|c| c.datos.canales.iter().cloned())
        .collect();
    let suprimidas = estado
        .db()
        .direcciones_suprimidas(empresa, &canales)
        .map_err(crate::AppError::Db)?;

    Ok(PaginaDeContactos {
        contactos: pagina.contactos.into_iter().map(Into::into).collect(),
        total: pagina.total,
        suprimidas,
    })
}

/// La ficha completa de un contacto, con **todos** sus canales (L-14).
///
/// Devuelve `None` si no existe, en vez de un error: pedir la ficha de algo que
/// alguien acaba de dar de baja no es una avería, y la pantalla tiene que poder
/// decir «ya no está» sin enseñar un mensaje de error.
///
/// # Errores
///
/// [`ErrorIpc`] si la empresa no está configurada o si la base no responde.
#[tauri::command]
pub fn contacto(
    estado: tauri::State<'_, EstadoApp>,
    id: arles_core::ids::ContactId,
) -> Result<Option<ContactoParaLaPantalla>, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    estado
        .db()
        .contacto(empresa, id)
        .map(|c| c.map(Into::into))
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

/// Da de alta un contacto escrito a mano (L-13).
///
/// # Errores
///
/// [`ErrorIpc`] con `canales` relleno si el formulario no es válido; con la
/// clave `error.db.direccion_en_uso` si alguna dirección ya es de otro
/// contacto.
#[tauri::command]
pub fn crear_contacto(
    estado: tauri::State<'_, EstadoApp>,
    borrador: arles_core::contacto::BorradorDeContacto,
) -> Result<arles_core::ids::ContactId, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let datos = validar(&estado, &borrador)?;
    estado
        .db()
        .crear_contacto(empresa, &datos)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

/// Cambia los datos o los canales de un contacto que ya existe (L-13).
///
/// # Errores
///
/// Las mismas que [`crear_contacto`], más `error.db.contacto_no_existe` si
/// alguien lo dio de baja mientras el formulario estaba abierto.
#[tauri::command]
pub fn editar_contacto(
    estado: tauri::State<'_, EstadoApp>,
    id: arles_core::ids::ContactId,
    borrador: arles_core::contacto::BorradorDeContacto,
) -> Result<(), ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let datos = validar(&estado, &borrador)?;
    estado
        .db()
        .editar_contacto(empresa, id, &datos)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

/// Da de baja un contacto.
///
/// # Errores
///
/// [`ErrorIpc`] con `error.db.contacto_no_existe` si ya no estaba.
#[tauri::command]
pub fn borrar_contacto(
    estado: tauri::State<'_, EstadoApp>,
    id: arles_core::ids::ContactId,
) -> Result<(), ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    estado
        .db()
        .borrar_contacto(empresa, id)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

/// Valida el borrador con el país de la empresa.
///
/// El país **sale de la empresa configurada**, no del formulario. Es lo que
/// completa un móvil escrito sin prefijo: «81 1234 5678» es mexicano porque la
/// empresa lo es. Si lo eligiera la pantalla, el mismo número escrito igual
/// acabaría en dos países según qué pantalla lo mandara.
fn validar(
    estado: &EstadoApp,
    borrador: &arles_core::contacto::BorradorDeContacto,
) -> Result<arles_core::contacto::DatosDeContacto, ErrorIpc> {
    let pais = estado
        .db()
        .empresa()
        .map_err(crate::AppError::Db)?
        .map(|e| e.datos.pais().to_owned())
        .ok_or(crate::AppError::EmpresaNoConfigurada)?;

    arles_core::contacto::DatosDeContacto::validar(borrador, &pais)
        .map_err(|errores| ErrorIpc::from(crate::AppError::ContactoInvalido(errores)))
}

// ─────────────────────────────────────────────────────────────────────────
// IMPORTAR UNA TABLA (entrega 3.3)
//
// Tres comandos y tres pasos, en este orden: leer, analizar, confirmar. Se
// parten así porque entre el segundo y el tercero hay **una persona
// decidiendo**: mira los choques y elige si sigue. Un solo comando que
// importara de golpe no podría enseñarle nada antes de escribir.
//
// ── POR QUÉ LA RUTA DEL ARCHIVO NO LLEGA DESDE LA PANTALLA ──
//
// La regla 4 de la frontera dice que **ninguna ruta del sistema de archivos la
// cruza**. Lo normal en Tauri sería que la webview abriera el diálogo con el
// plugin y le pasara la ruta al comando; eso deja a la webview eligiendo qué
// archivo se lee, y convierte «importar contactos» en «leer cualquier cosa del
// disco» si alguien consigue ejecutar algo ahí.
//
// Aquí el diálogo lo abre **Rust**, dentro del comando. La ruta nace y muere en
// esta capa. La webview no recibe el permiso `dialog:allow-open` —míralo en
// `capabilities/principal.json`: no está—, así que no puede pedir un archivo
// por su cuenta aunque quiera.
// ─────────────────────────────────────────────────────────────────────────

/// Lo que la pantalla necesita para enseñar la vista previa del archivo.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivoLeido {
    /// Nombre del archivo, **sólo para enseñarlo**. No es una ruta.
    pub nombre: String,
    pub encabezados: Vec<String>,
    /// Las primeras filas, para reconocer si el mapeo es el correcto.
    pub muestra: Vec<Vec<String>>,
    /// Qué campo parece ser cada columna. Es una **propuesta**: la pantalla la
    /// enseña y el usuario la puede cambiar.
    pub mapeo: Vec<arles_core::CampoImportable>,
    pub total_de_filas: usize,
}

/// Abre el diálogo del sistema, lee la tabla y la deja lista para analizar.
///
/// Devuelve `None` si el usuario cerró el diálogo sin elegir nada. No es un
/// error: cancelar es una decisión, y un error ahí pintaría un aviso rojo por
/// haber cambiado de opinión.
///
/// # Errores
///
/// [`ErrorIpc`] si el archivo no se puede leer, no es una tabla que ARLES
/// entienda, o pasa de los topes de `arles-import`.
#[tauri::command]
pub fn elegir_archivo_para_importar(
    estado: tauri::State<'_, EstadoApp>,
    app: tauri::AppHandle,
) -> Result<Option<ArchivoLeido>, ErrorIpc> {
    use tauri_plugin_dialog::DialogExt;

    let ruta = app
        .dialog()
        .file()
        // Los formatos que ARLES sabe leer, y sólo ésos. No es seguridad —el
        // usuario puede renombrar cualquier cosa—, es no ofrecerle elegir algo
        // que después se le va a rechazar.
        .add_filter(
            "Tablas de contactos",
            &["csv", "xlsx", "xls", "xlsm", "tsv"],
        )
        .blocking_pick_file();

    let Some(ruta) = ruta else {
        return Ok(None);
    };
    let ruta = ruta
        .into_path()
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

    let tabla =
        arles_import::leer(&ruta).map_err(|e| ErrorIpc::from(crate::AppError::Lectura(e)))?;

    let mapeo = arles_core::proponer_mapeo(&tabla.encabezados);
    let leido = ArchivoLeido {
        // Sólo el nombre, nunca la ruta: la carpeta de la que salió el archivo
        // dice dónde vive el usuario y no le aporta nada a la pantalla
        // (THREAT_MODEL.md §4.1).
        nombre: ruta
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
        encabezados: tabla.encabezados.clone(),
        muestra: tabla.muestra.clone(),
        mapeo,
        total_de_filas: tabla.total_de_filas,
    };

    estado.guardar_importacion(tabla);
    Ok(Some(leido))
}

/// Qué pasaría al importar, **sin escribir nada**.
///
/// Se puede llamar las veces que haga falta: el usuario corrige el mapeo, vuelve
/// a analizar, y mira otra vez. Mientras no confirme, la base no se toca.
///
/// # Errores
///
/// [`ErrorIpc`] con `error.app.sin_importacion_en_curso` si nadie ha elegido
/// archivo; con `error.app.empresa_no_configurada` si falta el alta.
#[tauri::command]
pub fn analizar_importacion(
    estado: tauri::State<'_, EstadoApp>,
    mapeo: Vec<arles_core::CampoImportable>,
) -> Result<arles_core::Analisis, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let pais = pais_de_la_empresa(&estado)?;

    // Las direcciones que trae el archivo, para preguntar de una vez de quién
    // son. Una consulta por fila serían medio millón de viajes a la base.
    let direcciones = estado
        .con_importacion(|t| direcciones_del_archivo(&mapeo, &t.filas, &pais))
        .ok_or_else(|| ErrorIpc::from(crate::AppError::SinImportacionEnCurso))?;

    let duenos = estado
        .db()
        .duenos_de_direcciones(empresa, &direcciones)
        .map_err(crate::AppError::Db)?;

    // Quién de este archivo ya había pedido no recibir. Dos consultas por lotes
    // en vez de una sola porque responden cosas distintas: una dice de quién es
    // la dirección, y la otra si esa dirección dijo que no. Juntarlas en un
    // `LEFT JOIN` ataría el informe de choques a la lista de supresión, y el
    // día que una cambie se llevaría la otra por delante.
    let suprimidas = estado
        .db()
        .motivos_de_supresion(empresa, &direcciones)
        .map_err(crate::AppError::Db)?;

    estado
        .con_importacion(|t| arles_core::analizar(&mapeo, &t.filas, &pais, &duenos, &suprimidas))
        .ok_or_else(|| ErrorIpc::from(crate::AppError::SinImportacionEnCurso))
}

/// Escribe lo que el análisis marcó como listo, con su declaración de origen.
///
/// **Vuelve a analizar antes de escribir**, con el mismo mapeo. No es trabajo
/// repetido: entre que el usuario miró el informe y pulsó confirmar pueden
/// haber pasado minutos, y en ese rato alguien pudo dar de alta a mano una de
/// las direcciones. Importar lo que decía el informe viejo escribiría algo que
/// ya no es cierto.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay importación en curso, si la empresa no está
/// configurada, o si la escritura falla. **Si falla, no se escribe nada**.
#[tauri::command]
pub fn confirmar_importacion(
    estado: tauri::State<'_, EstadoApp>,
    mapeo: Vec<arles_core::CampoImportable>,
    origen: arles_core::OrigenDeLaLista,
    texto_del_consentimiento: String,
) -> Result<ResumenParaLaPantalla, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let pais = pais_de_la_empresa(&estado)?;

    let direcciones = estado
        .con_importacion(|t| direcciones_del_archivo(&mapeo, &t.filas, &pais))
        .ok_or_else(|| ErrorIpc::from(crate::AppError::SinImportacionEnCurso))?;
    let duenos = estado
        .db()
        .duenos_de_direcciones(empresa, &direcciones)
        .map_err(crate::AppError::Db)?;
    // También al confirmar, y no sólo al analizar: entre que el usuario miró el
    // informe y pulsó el botón alguien pudo dar de baja a una de esas
    // direcciones. Importar con el informe viejo la contaría como alcanzable.
    let suprimidas = estado
        .db()
        .motivos_de_supresion(empresa, &direcciones)
        .map_err(crate::AppError::Db)?;

    let datos = estado
        .con_importacion(|t| {
            let informe = arles_core::analizar(&mapeo, &t.filas, &pais, &duenos, &suprimidas);
            let lote = arles_db::DatosDelLote {
                nombre_del_archivo: t.nombre.clone(),
                huella_del_archivo: t.huella.clone(),
                // El mapeo se guarda para poder explicar, dentro de un año, por
                // qué un contacto quedó con el nombre en el campo de la empresa.
                mapeo_en_json: serde_json::to_string(&mapeo).unwrap_or_default(),
                texto_del_consentimiento: texto_del_consentimiento.clone(),
                origen,
                total_de_filas: t.total_de_filas,
                choques: informe.choques.len(),
                invalidas: informe.invalidas.len() + informe.rechazadas.len(),
            };
            (lote, informe)
        })
        .ok_or_else(|| ErrorIpc::from(crate::AppError::SinImportacionEnCurso))?;

    let (lote, informe) = datos;
    let resumen = estado
        .db()
        .importar_contactos(empresa, &lote, &informe.listas)
        .map_err(crate::AppError::Db)?;

    // Se suelta el archivo: medio millón de filas no se quedan en memoria
    // hasta que alguien cierre ARLES.
    estado.soltar_importacion();

    Ok(ResumenParaLaPantalla {
        importados: resumen.importados,
        choques: resumen.choques,
        invalidas: resumen.invalidas,
        total_de_filas: resumen.total_de_filas,
    })
}

/// Descarta la importación en curso.
///
/// La llama la pantalla al cerrar el asistente. Sin esto, el archivo se queda
/// en memoria hasta que alguien cierre la aplicación.
#[tauri::command]
pub fn cancelar_importacion(estado: tauri::State<'_, EstadoApp>) {
    estado.soltar_importacion();
}

/// Lo que pasó al importar, para la pantalla.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumenParaLaPantalla {
    pub importados: usize,
    pub choques: usize,
    pub invalidas: usize,
    pub total_de_filas: usize,
}

/// Todas las direcciones normalizadas que trae el archivo, sin repetir.
///
/// Se normalizan aquí con la misma función del núcleo que usa el análisis: si
/// se consultaran sin normalizar, «Ana@Empresa.MX» no encontraría a su dueño y
/// el choque se detectaría tarde — al escribir, con el lote ya empezado.
fn direcciones_del_archivo(
    mapeo: &[arles_core::CampoImportable],
    filas: &[Vec<String>],
    pais: &str,
) -> Vec<String> {
    let mut vistas = std::collections::HashSet::new();
    for fila in filas {
        let Ok(borrador) = arles_core::fila_a_borrador(mapeo, fila) else {
            continue;
        };
        let Ok(datos) = arles_core::contacto::DatosDeContacto::validar(&borrador, pais) else {
            continue;
        };
        for canal in datos.canales {
            vistas.insert(canal.valor_normalizado);
        }
    }
    vistas.into_iter().collect()
}

/// El país de la empresa configurada, que completa los móviles sin prefijo.
pub(crate) fn pais_de_la_empresa(estado: &EstadoApp) -> Result<String, ErrorIpc> {
    estado
        .db()
        .empresa()
        .map_err(crate::AppError::Db)?
        .map(|e| e.datos.pais().to_owned())
        .ok_or_else(|| ErrorIpc::from(crate::AppError::EmpresaNoConfigurada))
}

// ─── Entrega 3.4 · la lista de supresión ────────────────────────────────────

/// Cuántas filas de la lista de supresión trae una página.
///
/// Más pequeña que la de contactos a propósito: esta lista se lee para
/// **revisarla**, no para recorrerla. Quien la abre busca una dirección
/// concreta o quiere ver las bajas recientes.
pub const SUPRESIONES_POR_PAGINA: u32 = 50;

const _: () = assert!(SUPRESIONES_POR_PAGINA as usize <= arles_db::MAX_SUPRESIONES_POR_PAGINA);

/// Una fila de la lista, ya lista para pintarse.
///
/// Todo lo que la pantalla necesita decidir **viene resuelto desde Rust**:
/// si se puede levantar, qué motivo enseñar, de quién es la dirección. La
/// webview no vuelve a razonar ninguna de esas reglas — es la razón de que los
/// comandos sean gruesos (regla de frontera 1).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupresionParaLaPantalla {
    pub id: arles_core::ids::SuppressionEntryId,
    pub canal: arles_core::Canal,
    pub direccion: String,
    /// Clave de i18n del motivo (§139).
    pub motivo: String,
    /// **Resuelto aquí, no en la pantalla.** Si la interfaz decidiera esto por
    /// su cuenta, el botón de «volver a permitir» aparecería o no según su
    /// propia copia de la regla — y el día que divergieran, enseñaría un botón
    /// que el núcleo rechaza, o escondería uno que sí valía.
    pub se_puede_levantar: bool,
    /// Si hay un contacto vivo con esa dirección, su nombre.
    pub de_quien: String,
    pub notas: String,
    pub creada: String,
    /// `true` si la baja fue para todos los canales de esa persona.
    pub global: bool,
}

impl From<arles_db::SupresionGuardada> for SupresionParaLaPantalla {
    fn from(s: arles_db::SupresionGuardada) -> Self {
        Self {
            id: s.id,
            canal: s.canal,
            direccion: s.direccion,
            motivo: s.motivo.clave_i18n().to_owned(),
            se_puede_levantar: s.motivo.se_puede_levantar(),
            // Cadena vacía y no `null`: el frontend tiene
            // `exactOptionalPropertyTypes`, y un `undefined` que cruza la
            // frontera obliga a cada componente a contemplar tres casos donde
            // hay dos.
            de_quien: s.de_quien.unwrap_or_default(),
            notas: s.notas,
            creada: s.creada,
            global: s.alcance == arles_core::AlcanceDeSupresion::Global,
        }
    }
}

/// Una página de la lista de supresión.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginaDeSupresiones {
    pub supresiones: Vec<SupresionParaLaPantalla>,
    /// Cuántas hay en total, no cuántas vienen. Es lo que permite decir
    /// «50 de 1 284» sin inventarse nada (§94).
    pub total: usize,
}

/// La lista de quién pidió no recibir, de lo más reciente a lo más viejo.
///
/// Es lo que Dirección pidió el 21/09/2026: poder ver quién canceló, por qué
/// canal y por qué motivo.
///
/// # Errores
///
/// [`ErrorIpc`] si la empresa no está configurada o si la base no responde.
#[tauri::command]
pub fn listar_supresiones(
    estado: tauri::State<'_, EstadoApp>,
    desde: u32,
) -> Result<PaginaDeSupresiones, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let db = estado.db();

    let filas = db
        .listar_supresiones(empresa, desde as usize, SUPRESIONES_POR_PAGINA as usize)
        .map_err(crate::AppError::Db)?;
    let total = db
        .contar_supresiones(empresa)
        .map_err(crate::AppError::Db)?;

    Ok(PaginaDeSupresiones {
        supresiones: filas.into_iter().map(Into::into).collect(),
        total,
    })
}

/// Da de baja a un contacto entero: todos sus canales de una vez.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA WEBVIEW NO DICE QUÉ DIRECCIONES SON
///
/// Sólo manda el identificador del contacto. Las direcciones **se leen de la
/// base aquí**, por dos razones:
///
///   1. La pantalla enseña lo que cargó al abrir la ficha, y entre ese momento
///      y el clic puede haberse añadido un canal. Dar de baja «todo» con una
///      lista vieja dejaría ese canal abierto — justo el que la persona pidió
///      cerrar.
///   2. Es la quinta regla de frontera: un tipo validado no entra por JSON. Si
///      la webview mandara las direcciones, mandaría su propia normalización, y
///      la supresión compara justo esa forma.
/// ─────────────────────────────────────────────────────────────────────────
///
/// # Errores
///
/// [`ErrorIpc`] con `error.db.contacto_no_existe` si ya no está; con
/// `error.supresion_sin_canales` si el contacto no tiene ninguna dirección.
#[tauri::command]
pub fn suprimir_contacto(
    estado: tauri::State<'_, EstadoApp>,
    id: arles_core::ids::ContactId,
    motivo: arles_core::MotivoDeSupresion,
    notas: String,
) -> Result<ResumenDeSupresionParaLaPantalla, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let db = estado.db();

    let canales = db
        .canales_para_suprimir(empresa, id)
        .map_err(crate::AppError::Db)?;

    let peticion = arles_core::PeticionDeSupresion::nueva(
        canales,
        arles_core::AlcanceDeSupresion::Global,
        motivo,
        // Lo hizo una persona desde ARLES. El origen no llega de la webview:
        // sería la webview afirmando quién tomó la decisión.
        arles_core::OrigenDeSupresion::Usuario,
        &notas,
    )
    .map_err(crate::AppError::Core)?;

    let resumen = db
        .suprimir(empresa, &peticion)
        .map_err(crate::AppError::Db)?;
    Ok(ResumenDeSupresionParaLaPantalla {
        nuevas: resumen.nuevas,
        ya_estaban: resumen.ya_estaban,
    })
}

/// Qué pasó al dar de baja. Cifras, no adjetivos (§94).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumenDeSupresionParaLaPantalla {
    pub nuevas: usize,
    pub ya_estaban: usize,
}

/// Saca una dirección de la lista, si su motivo lo permite.
///
/// **No se comprueba aquí si se puede.** Se pide, y el repositorio lee el
/// motivo guardado y decide. Comprobarlo también en este lado sería una segunda
/// copia de la regla, y dos copias divergen.
///
/// # Errores
///
/// [`ErrorIpc`] con `error.db.supresion_no_existe` si no está; con
/// `error.db.supresion_no_se_levanta` si la persona la pidió ella misma.
#[tauri::command]
pub fn levantar_supresion(
    estado: tauri::State<'_, EstadoApp>,
    id: arles_core::ids::SuppressionEntryId,
) -> Result<(), ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    estado
        .db()
        .levantar_supresion(empresa, id)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

// ─── Entrega 3.4 · el derecho de Acceso ─────────────────────────────────────

/// Qué pasó al entregar el informe. La pantalla lo necesita para decirlo.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InformeEntregado {
    /// **Sólo el nombre del archivo, nunca la carpeta.** Regla de frontera 4:
    /// ninguna ruta del sistema cruza hacia la webview. La carpeta donde el
    /// usuario guarda sus cosas dice dónde vive, y no le aporta nada a la
    /// pantalla (THREAT_MODEL.md §4.1).
    pub nombre_del_archivo: String,
    /// `false` si no consta ningún dato de esa dirección.
    ///
    /// **No es un error.** «No tenemos nada sobre usted» es exactamente lo que
    /// alguien puede estar preguntando, y el informe se entrega igual.
    pub habia_datos: bool,
    /// Si sigue dada de alta como contacto, o sólo queda su rastro.
    pub sigue_siendo_contacto: bool,
}

/// Entrega el informe de **Acceso**: todo lo que ARLES tiene sobre alguien.
///
/// ─────────────────────────────────────────────────────────────────────────
/// EL DIÁLOGO DE GUARDADO SE ABRE DESDE RUST
///
/// La webview manda una dirección y recibe el nombre del archivo que se
/// escribió. **Nunca ve una ruta**, ni la elige, ni la construye: es la cuarta
/// regla de frontera de `ARQUITECTURA.md §5`.
///
/// Además, así la webview no necesita el permiso `dialog:allow-save` ni
/// `fs:allow-write`. Un permiso que no se concede no se puede abusar, y esta
/// aplicación deniega por defecto.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA DIRECCIÓN SE NORMALIZA CON EL VALIDADOR DEL NÚCLEO
///
/// Quien atiende la solicitud escribe lo que le llegó por correo:
/// `Ana@Empresa.MX`. Las tablas guardan `ana@empresa.mx`. Buscar por lo
/// escrito no encontraría nada, y el informe diría «no hay datos» sobre
/// alguien de quien sí los hay — que es la peor respuesta posible a un
/// derecho de acceso.
/// ─────────────────────────────────────────────────────────────────────────
///
/// # Errores
///
/// - [`ErrorIpc`] con `error.email_invalido` o `error.telefono_invalido` si lo
///   escrito no es una dirección.
/// - Con `error.app.archivo_no_se_pudo_leer` si no se pudo escribir el archivo.
///
/// Devuelve `None` si quien atiende cerró el diálogo sin guardar. Cancelar es
/// una decisión, no un fallo.
#[tauri::command]
pub fn entregar_informe_de_acceso(
    estado: tauri::State<'_, EstadoApp>,
    app: tauri::AppHandle,
    canal: arles_core::Canal,
    direccion: String,
) -> Result<Option<InformeEntregado>, ErrorIpc> {
    use tauri_plugin_dialog::DialogExt;

    let empresa = empresa_actual(&estado)?;
    let pais = pais_de_la_empresa(&estado)?;

    // Se valida y se normaliza con el mismo validador que usa el alta. Dos
    // normalizaciones serían dos reglas que mantener iguales.
    let valor =
        arles_core::ValorDeCanal::parse(canal, &direccion, &pais).map_err(crate::AppError::Core)?;
    let normalizada = valor.normalized().to_owned();

    let db = estado.db();
    let nombre_de_la_empresa = db
        .empresa()
        .map_err(crate::AppError::Db)?
        .map(|e| e.datos.nombre_comercial().to_owned())
        .unwrap_or_default();

    let expediente = db
        .expediente_de(empresa, &normalizada, &nombre_de_la_empresa, &hoy_legible())
        .map_err(crate::AppError::Db)?;

    let texto = arles_core::arco::redactar(&expediente);

    // El nombre propuesto lleva la dirección para que quien atienda varias
    // solicitudes no acabe con cinco «informe.txt» en la misma carpeta.
    let sugerido = format!("datos-de-{}.txt", nombre_seguro(&normalizada));

    let Some(destino) = app
        .dialog()
        .file()
        .set_file_name(&sugerido)
        .add_filter("Texto", &["txt"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let destino = destino
        .into_path()
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

    std::fs::write(&destino, texto.as_bytes())
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

    // Se registra **después** de escribir, no antes: anotar una entrega que no
    // llegó a ocurrir convertiría la bitácora en una lista de intenciones.
    db.registrar_acceso(empresa, &normalizada)
        .map_err(crate::AppError::Db)?;

    Ok(Some(InformeEntregado {
        nombre_del_archivo: destino
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
        habia_datos: expediente.hay_algo(),
        sigue_siendo_contacto: expediente.sigue_siendo_contacto(),
    }))
}

/// Una dirección convertida en algo que se puede poner en un nombre de archivo.
///
/// Se quedan letras, números, punto, guion y guion bajo; **todo lo demás pasa a
/// guion**. Una arroba o un `+` en un nombre de archivo son legales en Linux y
/// problemáticos en otros sitios, y una barra convertiría el nombre en una
/// ruta.
pub(crate) fn nombre_seguro(direccion: &str) -> String {
    let limpio: String = direccion
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    limpio.chars().take(60).collect()
}

/// La fecha de hoy, escrita como la lee una persona.
///
/// Sin dependencias de formato: el informe lo lee alguien que preguntó por sus
/// datos, y `2026-09-21T14:03:22Z` no es una fecha para esa persona.
pub(crate) fn hoy_legible() -> String {
    const MESES: [&str; 12] = [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ];
    let ahora = time::OffsetDateTime::now_utc();
    let mes = MESES
        .get(usize::from(u8::from(ahora.month())).saturating_sub(1))
        .copied()
        .unwrap_or("");
    format!("{} de {} de {}", ahora.day(), mes, ahora.year())
}

// ─── El derecho de Cancelación · borrar de verdad ───────────────────────────

/// Qué se borraría si se cancelara a esa persona. **No borra nada.**
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ HAY DOS COMANDOS Y NO UNO
///
/// Un borrado irreversible que sólo pregunta «¿seguro?» no pregunta nada:
/// nadie puede decidir sobre algo que no ha visto. Este comando existe para
/// que la pantalla pueda **enseñar las cifras antes** de dejar pulsar.
///
/// Se apoya en el mismo expediente que arma el derecho de Acceso. Contar por
/// separado daría dos recuentos de lo mismo, y el día que divergieran la
/// pantalla prometería borrar algo que no borra.
///
/// # Errores
///
/// - `error.email_invalido` o `error.telefono_invalido` si lo escrito no es
///   una dirección válida.
#[tauri::command]
pub fn que_se_borraria(
    estado: tauri::State<'_, EstadoApp>,
    canal: arles_core::Canal,
    direccion: String,
) -> Result<arles_core::cancelacion::VistaPrevia, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let pais = pais_de_la_empresa(&estado)?;
    let valor =
        arles_core::ValorDeCanal::parse(canal, &direccion, &pais).map_err(crate::AppError::Core)?;

    let db = estado.db();
    let expediente = db
        .expediente_de(empresa, valor.normalized(), "", &hoy_legible())
        .map_err(crate::AppError::Db)?;

    Ok(arles_core::cancelacion::VistaPrevia::desde_expediente(
        &expediente,
    ))
}

/// Borra a esa persona. **Irreversible.**
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA CONFIRMACIÓN NO VIVE AQUÍ, Y ES A PROPÓSITO
///
/// Este comando **no pregunta nada**: recibe una dirección y borra. Quien
/// confirma es la pantalla, y lo hace obligando a escribir la dirección
/// completa — no con una casilla, que se marca sin leer.
///
/// Meter la confirmación en el comando parecería más seguro y sería peor:
/// una comprobación que sólo puede fallar con un valor que el propio programa
/// acaba de generar no comprueba nada, y además la pantalla tendría que
/// mandar dos veces la misma dirección para que la segunda «confirmara» la
/// primera. La defensa real está en **lo que el núcleo hace**, que está
/// probado: la baja se crea, los envíos se anonimizan, la ficha desaparece.
///
/// # Errores
///
/// - `error.email_invalido` o `error.telefono_invalido` si lo escrito no es
///   una dirección válida.
/// - `error.db.*` si algo falla al borrar. Entonces **no se borra nada**: la
///   transacción se deshace entera.
#[tauri::command]
pub fn cancelar_datos_de(
    estado: tauri::State<'_, EstadoApp>,
    canal: arles_core::Canal,
    direccion: String,
) -> Result<arles_core::cancelacion::Cancelacion, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let pais = pais_de_la_empresa(&estado)?;
    // Se valida con el MISMO validador que el alta y que la vista previa. Con
    // dos normalizaciones distintas, la vista previa podría enseñar lo de una
    // persona y el borrado llevarse lo de otra.
    let valor =
        arles_core::ValorDeCanal::parse(canal, &direccion, &pais).map_err(crate::AppError::Core)?;

    estado
        .db()
        .cancelar_a(empresa, canal, valor.normalized())
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))
}

// ─── Remitentes · entrega 5.3 ───────────────────────────────────────────────

/// Una cuenta remitente tal como la ve la pantalla. **Sin la contraseña.**
///
/// No es que se omita al serializar: **no está en la estructura**. La regla 3
/// de la frontera —ningún comando devuelve un secreto— se cumple aquí por la
/// forma del tipo, no por acordarse de no rellenar un campo.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemitenteEnPantalla {
    pub id: String,
    pub servidor: String,
    pub puerto: u16,
    /// `starttls` o `directo`. La interfaz lo traduce con `correo.cifrado.*`.
    pub cifrado: String,
    pub usuario: String,
    pub remitente: String,
    pub nombre_visible: Option<String>,
    /// `connected`, `needs_reauth`, `circuit_open` o `disabled`.
    pub estado: String,
    pub limite_diario: u32,
    pub limite_por_hora: u32,
}

/// Lo que devuelve guardar o probar: la lista al día y qué se averiguó.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoDeRemitente {
    /// La clave del diagnóstico (`correo.prueba.*`), o `None` si no se probó.
    pub diagnostico: Option<String>,
    /// `true` si la cuenta quedó lista para enviar.
    pub sirve: bool,
    /// La lista completa, para que la pantalla no tenga que volver a pedirla.
    pub remitentes: Vec<RemitenteEnPantalla>,
}

fn a_pantalla(c: arles_db::correo::CuentaDeCorreoGuardada) -> RemitenteEnPantalla {
    RemitenteEnPantalla {
        id: c.id.to_string(),
        servidor: c.datos.servidor().to_owned(),
        puerto: c.datos.puerto(),
        cifrado: c.datos.cifrado().como_str().to_owned(),
        usuario: c.datos.usuario().to_owned(),
        remitente: c.datos.remitente().raw().to_owned(),
        nombre_visible: c.datos.nombre_visible().map(ToOwned::to_owned),
        estado: c.estado,
        limite_diario: c.limites.diario,
        limite_por_hora: c.limites.por_hora,
    }
}

/// Lista las cuentas remitentes.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde, o si **alguna** cuenta guardada no pasa
/// la validación: una cuenta rota se dice, no se esconde de la lista.
#[tauri::command]
pub fn remitentes(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<Vec<RemitenteEnPantalla>, ErrorIpc> {
    let empresa = empresa_actual(&estado).map_err(ErrorIpc::from)?;
    let cuentas = estado
        .db()
        .cuentas_de_correo(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;
    Ok(cuentas.into_iter().map(a_pantalla).collect())
}

/// Construye el adaptador a partir de un borrador ya validado.
fn adaptador(
    datos: &arles_core::correo::DatosDeLaCuenta,
    contrasena: &str,
) -> Result<arles_email::ProveedorSmtp, ErrorIpc> {
    use arles_core::correo::CifradoDeCorreo;

    let cifrado = match datos.cifrado() {
        CifradoDeCorreo::Starttls => arles_email::Cifrado::starttls(),
        CifradoDeCorreo::Directo => arles_email::Cifrado::directo(),
    };

    arles_email::ProveedorSmtp::nuevo(
        &arles_email::Cuenta {
            servidor: datos.servidor().to_owned(),
            puerto: Some(datos.puerto()),
            usuario: datos.usuario().to_owned(),
            remitente: datos.remitente().raw().to_owned(),
            nombre_visible: datos.nombre_visible().map(ToOwned::to_owned),
            cifrado,
        },
        contrasena,
        // La prueba de conexión no manda ningún correo, así que este contenido
        // no llega a usarse. Está porque el adaptador es el mismo que envía, y
        // que sea el mismo es lo que hace que la prueba signifique algo.
        arles_email::Contenido {
            asunto: String::new(),
            cuerpo: String::new(),
        },
    )
    .map_err(|e| ErrorIpc {
        clave: e.clave_i18n().to_owned(),
        detalle: e.to_string(),
        campos: Vec::new(),
        canales: Vec::new(),
        campos_de_cuenta: Vec::new(),
    })
}

/// Prueba una cuenta **sin guardar nada** (§29).
///
/// La contraseña entra por aquí porque el usuario acaba de escribirla y todavía
/// no está en el llavero. Entra; no sale: ningún comando la devuelve.
///
/// Si `id` viene relleno y `contrasena` está vacía, se prueba la contraseña ya
/// guardada. Es el caso de «esta cuenta ha dejado de funcionar, ¿qué le pasa?»,
/// donde obligar a reescribir la contraseña para poder diagnosticar es la forma
/// segura de que nadie use el botón.
///
/// # Errores
///
/// [`ErrorIpc`] con `camposDeCuenta` relleno si el formulario no es válido.
#[tauri::command]
pub fn probar_remitente(
    estado: tauri::State<'_, EstadoApp>,
    borrador: arles_core::correo::BorradorDeCuenta,
    contrasena: String,
    id: Option<String>,
) -> Result<ResultadoDeRemitente, ErrorIpc> {
    let datos = arles_core::correo::DatosDeLaCuenta::validar(&borrador)
        .map_err(|campos| ErrorIpc::from(crate::AppError::CuentaDeCorreoInvalida(campos)))?;

    let contrasena = resolver_contrasena(contrasena, id.as_deref())?;
    let diagnostico = adaptador(&datos, &contrasena)?.probar();

    Ok(ResultadoDeRemitente {
        diagnostico: Some(diagnostico.clave_i18n().to_owned()),
        sirve: diagnostico.sirve(),
        remitentes: remitentes(estado)?,
    })
}

/// La contraseña que toca usar: la escrita, o la del llavero.
///
/// **El valor que sale de aquí no cruza la frontera de vuelta.** Se usa para
/// construir el adaptador y muere con él.
fn resolver_contrasena(escrita: String, id: Option<&str>) -> Result<String, ErrorIpc> {
    if !escrita.is_empty() {
        return Ok(escrita);
    }

    let Some(id) = id else {
        // Sin contraseña escrita y sin cuenta guardada no hay nada que probar.
        // Se dice como un campo del formulario para que la pantalla marque la
        // casilla, en vez de soltar un aviso general.
        return Err(ErrorIpc::from(crate::AppError::CuentaDeCorreoInvalida(
            vec![arles_core::correo::ErrorDeCampoDeCuenta {
                campo: arles_core::correo::CampoDeCuenta::Usuario,
                clave: "correo.error.contrasenaVacia",
            }],
        )));
    };

    crate::llavero::Llavero::de_la_cuenta(id)
        .leer_contrasena()
        .map_err(ErrorIpc::from)?
        .ok_or_else(|| {
            ErrorIpc::from(crate::AppError::CuentaDeCorreoInvalida(vec![
                arles_core::correo::ErrorDeCampoDeCuenta {
                    campo: arles_core::correo::CampoDeCuenta::Usuario,
                    clave: "correo.error.contrasenaVacia",
                },
            ]))
        })
}

/// Guarda una cuenta remitente: la valida, **la prueba** y la escribe.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ GUARDAR PRUEBA, Y POR QUÉ GUARDA IGUAL SI LA PRUEBA FALLA
///
/// Son dos decisiones, y las dos tienen su motivo.
///
/// **Prueba** porque una cuenta guardada sin probar se queda diciendo que está
/// conectada, y el primer sitio donde se descubre que no lo está es una campaña
/// en marcha, con los contactos ya en cola. Es un comando grueso (regla 1 de la
/// frontera): validar, probar y guardar en un solo cruce, porque partirlo deja
/// estados intermedios donde la pantalla y la base no dicen lo mismo.
///
/// **Y guarda aunque la prueba falle** porque lo contrario pierde el trabajo de
/// quien lo escribió. Un servidor que no responde ahora mismo, una red de
/// oficina que bloquea el puerto, un corte de dos minutos: si el guardado se
/// rechazara, esa persona tendría que volver a teclear seis campos y una
/// contraseña de aplicación de dieciséis caracteres. Lo que se guarda en ese
/// caso queda marcado como `needs_reauth` —hay que configurarla—, así que la
/// cuenta existe y no miente.
///
/// El diagnóstico vuelve con el resultado para que la pantalla lo diga.
///
/// ─────────────────────────────────────────────────────────────────────────
/// EL ORDEN DE ESCRITURA, QUE NO ES CASUAL
///
/// Primero el llavero, después la base. Si se hiciera al revés y el llavero
/// fallara, quedaría una cuenta en la base apuntando a una contraseña que no
/// existe: la pantalla la enseñaría como configurada y todos sus envíos
/// fallarían con «la contraseña no sirve».
///
/// Al revés, lo peor que queda si la base falla es una entrada de llavero que
/// nadie usa — y aun así se borra, para no ir dejando secretos huérfanos por la
/// máquina.
///
/// # Errores
///
/// [`ErrorIpc`] con `camposDeCuenta` si el formulario no vale, o si el llavero
/// no está disponible. **Sin llavero no se guarda nada** (ADR-0011).
#[tauri::command]
pub fn guardar_remitente(
    estado: tauri::State<'_, EstadoApp>,
    borrador: arles_core::correo::BorradorDeCuenta,
    contrasena: String,
    id: Option<String>,
) -> Result<ResultadoDeRemitente, ErrorIpc> {
    let datos = arles_core::correo::DatosDeLaCuenta::validar(&borrador)
        .map_err(|campos| ErrorIpc::from(crate::AppError::CuentaDeCorreoInvalida(campos)))?;

    let empresa = empresa_actual(&estado).map_err(ErrorIpc::from)?;

    // El identificador se genera aquí, antes de escribir nada: la referencia
    // del llavero lo lleva dentro, y la entrada del llavero se nombra con él.
    let id_cuenta = match &id {
        Some(existente) => existente.parse().map_err(|_| {
            ErrorIpc::from(crate::AppError::Db(arles_db::DbError::DatoInvalido {
                motivo: "ese identificador de cuenta no es válido",
            }))
        })?,
        None => arles_core::ids::EmailAccountId::nuevo(),
    };
    let texto_del_id = id_cuenta.to_string();

    let contrasena = resolver_contrasena(contrasena, id.as_deref())?;

    // 1 · El secreto, primero.
    let llavero = crate::llavero::Llavero::de_la_cuenta(&texto_del_id);
    llavero
        .guardar_contrasena(&contrasena)
        .map_err(ErrorIpc::from)?;

    // 2 · La prueba, con el adaptador que de verdad envía.
    let diagnostico = adaptador(&datos, &contrasena)?.probar();

    // 3 · La base. Si falla, el secreto que se acaba de escribir se retira.
    let escritura = match &id {
        Some(_) => estado.db().actualizar_cuenta_de_correo(id_cuenta, &datos),
        None => estado.db().alta_de_cuenta_de_correo(
            empresa,
            id_cuenta,
            &datos,
            arles_db::correo::LimitesDeLaCuenta::default(),
            &crate::llavero::referencia_de_cuenta(&texto_del_id),
        ),
    };

    if let Err(e) = escritura {
        // Sólo se retira el secreto de una cuenta que se acaba de crear. En una
        // edición, la contraseña vieja ya fue sustituida por la nueva y borrarla
        // dejaría sin contraseña a una cuenta que sigue existiendo.
        if id.is_none() {
            let _ = llavero.borrar();
        }
        return Err(ErrorIpc::from(crate::AppError::Db(e)));
    }

    // 4 · Y si la prueba no pasó, la cuenta lo dice.
    if !diagnostico.sirve() {
        estado
            .db()
            .marcar_cuenta_por_configurar(id_cuenta)
            .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;
    }

    Ok(ResultadoDeRemitente {
        diagnostico: Some(diagnostico.clave_i18n().to_owned()),
        sirve: diagnostico.sirve(),
        remitentes: remitentes(estado)?,
    })
}

/// Cómo salió el envío de prueba.
///
/// **No dice «entregado»** (§65). ARLES no puede saberlo: lo que sabe es que el
/// servidor se hizo cargo del mensaje. Un «entregado» aquí sería una afirmación
/// que el producto entero está construido para no hacer.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoDelEnvioDePrueba {
    /// El servidor aceptó el mensaje.
    pub aceptado: bool,
    /// La clave del texto que explica qué pasó, cuando no se aceptó (§139).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problema: Option<String>,
    /// A dónde se mandó, ya normalizada. Se enseña: confirma al usuario que la
    /// dirección que escribió es la que se usó.
    pub a_donde: String,
}

/// Manda un correo de prueba por la cuenta de una etapa, y lo deja anotado.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTO NO PASA POR LA COLA, Y ES LO QUE MÁS IMPORTA DE ESTA FUNCIÓN
///
/// Un envío de prueba **no toca nada del aparato de campañas**: no entra en la
/// cola, no gasta presupuesto, no aparece en el registro de envíos, no consulta
/// la lista de bajas y **no puede dar de baja a nadie**.
///
/// Es la regla 3 de la fase escrita como código: las únicas puertas a una
/// supresión son el `RCPT TO` de un envío real y el acuse final. Si un `550` de
/// esta función pudiera suprimir, probar una cuenta mal configurada contra la
/// dirección propia del usuario **lo daría de baja a él** — y una baja no se
/// deshace.
///
/// Por eso llama al adaptador directamente y no al trabajador. La tentación de
/// reutilizar el bucle «para no duplicar» es real, y traería consigo todo lo
/// que el bucle hace.
///
/// ─────────────────────────────────────────────────────────────────────────
/// SE ANOTA DESPUÉS, NUNCA ANTES
///
/// La marca se escribe sólo si el servidor aceptó. Escrita antes, el preflight
/// daría luz verde a una cuenta que no envía, que es exactamente lo que este
/// punto existe para impedir.
///
/// # Errores
///
/// [`ErrorIpc`] si la dirección no vale, si la etapa no existe o no es de
/// correo, si la cuenta no puede enviar, o si la base falla. **Que el servidor
/// rechace el mensaje no es un error de este comando**: es un resultado con su
/// `problema`, porque quien lo lee tiene que poder ver la dirección que usó y
/// volver a intentarlo sin salir de la pantalla.
#[tauri::command]
pub fn enviar_correo_de_prueba(
    estado: tauri::State<'_, EstadoApp>,
    etapa: String,
    a_donde: String,
) -> Result<ResultadoDelEnvioDePrueba, ErrorIpc> {
    let destino = arles_core::EmailAddress::parse(&a_donde)
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    let cuenta = estado
        .db()
        .remitente_de_la_etapa(&etapa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
        .ok_or_else(|| {
            ErrorIpc::from(crate::AppError::CuentaNoSirve(
                crate::fabrica::LaCuentaNoSirve::NoExiste,
            ))
        })?;

    let proveedor = crate::fabrica::proveedor_de_la_cuenta(
        estado.db(),
        &crate::fabrica::ElLlaveroDelSistema,
        cuenta,
        contenido_de_prueba(),
    )?;

    enviar_la_prueba(estado.db(), &proveedor, &etapa, &destino, cuenta)
}

/// Lo que lleva dentro el correo de prueba.
///
/// **Provisional hasta la Fase 6**, donde el asunto y el cuerpo los produce la
/// plantilla de la campaña. Hasta entonces dice lo único que puede decir con
/// verdad: que ARLES lo mandó, y para qué sirve haberlo recibido.
///
/// El texto está aquí y no en los archivos de idioma a propósito: es el
/// contenido de un correo, no una cadena de la interfaz. Traducirlo exigiría
/// saber en qué idioma lee quien lo recibe, y quien lo recibe es el propio
/// usuario — que ya está viendo la aplicación en español.
fn contenido_de_prueba() -> arles_email::Contenido {
    arles_email::Contenido {
        asunto: "Correo de prueba de ARLES RELAY".to_owned(),
        cuerpo: "Si estás leyendo esto, la cuenta de correo configurada en ARLES \
                 puede enviar.\n\n\
                 Este mensaje se mandó a mano desde ARLES RELAY para comprobar la \
                 conexión antes de activar una campaña. No forma parte de ninguna \
                 campaña y nadie más lo ha recibido."
            .to_owned(),
    }
}

/// La misma función, para las pruebas de integración.
///
/// Existe porque la prueba de punta a punta necesita darle **su** proveedor
/// —uno que apunta al servidor de mentira— y el comando real construye el suyo
/// a partir del llavero del sistema, que no existe en la integración continua.
///
/// Lo que se salta es sólo la resolución de la cuenta. Todo lo demás —el
/// envío, la conversación SMTP, el cifrado y la anotación— es el mismo código
/// que corre en producción; si no lo fuera, la prueba no diría nada.
#[doc(hidden)]
pub fn envio_de_prueba_para_tests(
    db: &arles_db::Db,
    proveedor: &dyn arles_motor::proveedor::Proveedor,
    etapa: &str,
    destino: &arles_core::EmailAddress,
    cuenta: arles_core::ids::EmailAccountId,
) -> Result<ResultadoDelEnvioDePrueba, ErrorIpc> {
    enviar_la_prueba(db, proveedor, etapa, destino, cuenta)
}

/// Qué se le dice al usuario cuando la prueba no salió.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LOS CINCO FALLOS DEL MOTOR NO SIGNIFICAN LO MISMO EN UNA PRUEBA
///
/// El motor los interpreta para decidir si reintenta; aquí no hay reintento que
/// decidir, hay una persona mirando la pantalla. Y dos de ellos cambian de
/// significado por completo:
///
/// **«La dirección no existe»** en una campaña señala a un contacto, y es la
/// puerta a una baja. Aquí señala a **la dirección que el usuario acaba de
/// escribir**, casi siempre la suya, y no da de baja a nadie — esta función no
/// toca la lista de supresión ni puede hacerlo. El texto tiene que decirlo, o
/// alguien va a creer que se acaba de dar de baja a sí mismo.
///
/// **«Pidió la baja»** no puede llegar aquí: eso lo produce el motor al
/// consultar la lista de supresión, y un envío de prueba no la consulta. Se
/// mapea igualmente, porque un `match` que no cubriera el caso obligaría a un
/// comodín — y un comodín es lo que hace que el día que aparezca un fallo nuevo
/// el usuario vea el mensaje de otra cosa.
fn por_que_no_salio(fallo: &arles_core::reintento::Fallo) -> &'static str {
    use arles_core::reintento::Fallo;
    match fallo {
        Fallo::Pasajero => "correo.prueba.envio.pasajero",
        Fallo::NoMeMandesNadaDurante(_) => "correo.prueba.envio.elServidorPideEsperar",
        Fallo::LaDireccionNoExiste => "correo.prueba.envio.esaDireccionNoExiste",
        Fallo::LaCuentaNoAutentica => "correo.prueba.envio.laCuentaNoAutentica",
        Fallo::EsaPersonaPidioLaBaja => "correo.prueba.envio.pidioLaBaja",
    }
}

/// El envío en sí, separado para poder probarlo con un proveedor cualquiera.
///
/// Sin esta separación, el envío de prueba sólo se podría ejercitar con el
/// llavero del sistema por medio — que no existe en la integración continua—, y
/// se quedaría sin una sola comprobación.
///
/// Se expone bajo otro nombre en [`envio_de_prueba_para_tests`]: lo que hace
/// falta probar de punta a punta es esto, y no el trozo que resuelve la cuenta.
fn enviar_la_prueba(
    db: &arles_db::Db,
    proveedor: &dyn arles_motor::proveedor::Proveedor,
    etapa: &str,
    destino: &arles_core::EmailAddress,
    cuenta: arles_core::ids::EmailAccountId,
) -> Result<ResultadoDelEnvioDePrueba, ErrorIpc> {
    let resultado = mandar_el_correo(proveedor, destino);

    // **La marca sólo se escribe si el servidor aceptó.** Escrita antes, el
    // preflight daría luz verde a una cuenta que no envía.
    if resultado.aceptado {
        db.registrar_envio_de_prueba(etapa, destino.raw(), cuenta)
            .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;
    }
    Ok(resultado)
}

/// Manda el correo y traduce lo que pasó. **No escribe nada en la base.**
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTÁ SEPARADO DE LA ANOTACIÓN
///
/// Hay dos preguntas distintas y las dos son legítimas:
///
/// **«¿Esta cuenta puede mandar un correo?»** es una pregunta de la cuenta. Se
/// contesta desde Remitentes, no depende de ninguna campaña, y **no se anota**:
/// no hay etapa a la que atar la respuesta.
///
/// **«¿Se hizo el envío de prueba de esta etapa?»** es lo que pregunta el
/// preflight (§45), y ésa sí se anota, atada a la etapa y al remitente con el
/// que se probó.
///
/// Son la misma conversación SMTP y dos cosas distintas encima. Fundirlas
/// obligaría a inventar una etapa para poder probar una cuenta, o a dejar la
/// marca sin etapa — y entonces cambiar de remitente no la invalidaría, que es
/// justamente la garantía que hace que el punto del preflight signifique algo.
fn mandar_el_correo(
    proveedor: &dyn arles_motor::proveedor::Proveedor,
    destino: &arles_core::EmailAddress,
) -> ResultadoDelEnvioDePrueba {
    // Una clave propia, que no sale de la cola: un envío de prueba no tiene
    // fila en `message_attempt` y no puede reutilizar la de nadie.
    let clave = format!("prueba-{}", arles_core::ids::MessageAttemptId::nuevo());

    match proveedor.enviar(&arles_motor::proveedor::Mensaje {
        para: destino.raw(),
        clave_de_idempotencia: &clave,
    }) {
        Ok(_) => ResultadoDelEnvioDePrueba {
            aceptado: true,
            problema: None,
            a_donde: destino.raw().to_owned(),
        },
        // Un fallo del servidor no es un error: la pantalla tiene que poder
        // enseñar la dirección y el motivo, y dejar reintentar.
        Err(fallo) => ResultadoDelEnvioDePrueba {
            aceptado: false,
            problema: Some(por_que_no_salio(&fallo).to_owned()),
            a_donde: destino.raw().to_owned(),
        },
    }
}

/// Manda un correo de prueba **por una cuenta**, sin campaña de por medio.
///
/// Es el botón de Remitentes, y contesta la pregunta que «Probar conexión» no
/// contesta: aquélla dice que la cuenta autentica; ésta dice que **puede
/// enviar**. No son lo mismo — una cuenta puede autenticar perfectamente y que
/// el servidor rechace el remitente, o que la dirección no esté autorizada para
/// enviar desde ese dominio.
///
/// **No anota nada**, y es deliberado: la marca del preflight cuelga de una
/// etapa de campaña, y aquí no hay ninguna. Anotarla sin etapa la haría
/// sobrevivir a un cambio de remitente, que es lo que la invalida.
///
/// # Errores
///
/// [`ErrorIpc`] si la dirección no vale, si la cuenta no existe o no puede
/// enviar, o si el llavero no responde. Que el servidor rechace el mensaje **no
/// es un error**: es un resultado con su `problema`.
#[tauri::command]
pub fn probar_envio_de_la_cuenta(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
    a_donde: String,
) -> Result<ResultadoDelEnvioDePrueba, ErrorIpc> {
    let destino = arles_core::EmailAddress::parse(&a_donde)
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    let cuenta = id.parse::<arles_core::ids::EmailAccountId>().map_err(|_| {
        ErrorIpc::from(crate::AppError::CuentaNoSirve(
            crate::fabrica::LaCuentaNoSirve::NoExiste,
        ))
    })?;

    let proveedor = crate::fabrica::proveedor_de_la_cuenta(
        estado.db(),
        &crate::fabrica::ElLlaveroDelSistema,
        cuenta,
        contenido_de_prueba(),
    )?;

    Ok(mandar_el_correo(&proveedor, &destino))
}

/// [`mandar_el_correo`], para las pruebas de integración. Ver
/// [`envio_de_prueba_para_tests`].
#[doc(hidden)]
pub fn mandar_el_correo_para_tests(
    proveedor: &dyn arles_motor::proveedor::Proveedor,
    destino: &arles_core::EmailAddress,
) -> ResultadoDelEnvioDePrueba {
    mandar_el_correo(proveedor, destino)
}

// ── El preflight, enchufado · entrega 6.5 ────────────────────────────────

/// Lo que la pantalla de preflight enseña.
///
/// Lleva el veredicto **y las cifras**, porque los ocho puntos dicen si pasa y
/// no dicen cuánto: «la audiencia no está vacía» no es lo mismo que «1 342
/// destinatarios», y §94 pide lo segundo.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionDeCampana {
    /// Los ocho puntos y si se puede activar.
    pub preflight: arles_core::preflight::Preflight,
    /// Cuánta gente recibiría.
    pub audiencia: u32,
    /// A cuántos de ellos les falta alguna variable.
    pub contactos_con_huecos: u32,
    /// Cuánto tardaría con el ritmo y el horario configurados.
    pub estimacion: arles_core::duracion::Estimacion,
    /// El diagnóstico de la sonda a la cuenta, para poder explicar **por qué**
    /// fallan los puntos de conexión y autenticación. `None` si no se llegó a
    /// sondear porque la cuenta ya estaba descartada.
    pub diagnostico: Option<String>,
}

/// Revisa una campaña contra los ocho puntos, hablando con el servidor.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTE COMANDO SE CONECTA, Y POR QUÉ ESO NO ES OPCIONAL
///
/// Cinco de los ocho puntos salen de la base y los reúne
/// `arles_db::Db::lo_que_se_sabe_de`. Los otros tres —«la cuenta autentica»,
/// «hay conexión» y los registros del dominio— exigen salir a la red.
///
/// Y **no se pueden dejar en «sin comprobar»**: un punto bloqueante sin
/// comprobar impide activar igual que uno que falla, así que una campaña
/// nunca llegaría a activarse. El preflight dejaría de ser una revisión y
/// pasaría a ser un muro.
///
/// Así que aquí se sondea de verdad, con el mismo adaptador que envía —el de
/// «Probar conexión»—, y el diagnóstico se traduce a los dos puntos con
/// `Diagnostico::autentica` y `Diagnostico::hay_conexion`, que distinguen «no
/// autentica» de «no se llegó a intentar».
///
/// **Los registros del dominio siguen en `None`** y seguirán: ARLES no
/// consulta DNS todavía (P-16). Es el único de los ocho que no bloquea, así
/// que aparece como «sin comprobar» y no impide nada — que es exactamente lo
/// que es.
///
/// ─────────────────────────────────────────────────────────────────────────
/// UNA CUENTA QUE NO PUEDE ENVIAR NO SE SONDEA
///
/// Si la cuenta está desactivada, detenida por fallos o sin contraseña
/// guardada, no hay nada que preguntarle al servidor: el punto de
/// autenticación falla y el de conexión se queda sin comprobar, porque no se
/// llegó a mirar. Sondear igualmente daría «no se llegó al servidor» y
/// mandaría a revisar la red por una cuenta que alguien desactivó.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada, si el identificador no vale, o
/// si la campaña no existe o no tiene etapa de correo. Que la cuenta no pueda
/// enviar **no es un error**: es un punto que falla.
#[tauri::command]
pub fn revisar_campana(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<RevisionDeCampana, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let campana = id
        .parse::<arles_core::ids::CampaignId>()
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    revision_de(
        estado.db(),
        &crate::fabrica::ElLlaveroDelSistema,
        empresa,
        campana,
    )
}

/// El preflight completo, con la fuente de contraseñas que se le diga.
///
/// El comando de arriba le pasa el llavero del sistema. Las pruebas de
/// integración le pasan el suyo y apuntan la cuenta al servidor de mentira, y
/// así recorren **este mismo código**: la lectura de la base, la sonda, la
/// traducción del diagnóstico y las reglas del núcleo. Si la prueba tuviera su
/// propio camino, no diría nada de éste.
///
/// # Errores
///
/// Los de [`revisar_campana`].
pub fn revision_de(
    db: &arles_db::Db,
    secretos: &dyn crate::fabrica::FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    campana: arles_core::ids::CampaignId,
) -> Result<RevisionDeCampana, ErrorIpc> {
    let sabido = db
        .lo_que_se_sabe_de(empresa, campana, time::OffsetDateTime::now_utc())
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    let sonda = sondear_la_cuenta(db, secretos, sabido.cuenta);

    let preflight = arles_core::preflight::revisar(&arles_core::preflight::Observaciones {
        cuenta_autentica: sonda.autentica,
        hay_conexion: sonda.hay_conexion,
        credencial_sirve: sabido.credencial_sirve,
        audiencia: sabido.audiencia,
        contactos_con_variables_sin_resolver: Some(sabido.contactos_con_huecos),
        estimacion: &sabido.estimacion,
        se_hizo_el_envio_de_prueba: Some(sabido.se_hizo_el_envio_de_prueba),
        // P-16: ARLES no consulta DNS. No bloquea, y se dice.
        registros_del_dominio: None,
    });

    Ok(RevisionDeCampana {
        preflight,
        audiencia: sabido.audiencia,
        contactos_con_huecos: sabido.contactos_con_huecos,
        estimacion: sabido.estimacion,
        diagnostico: sonda.clave_i18n,
    })
}

/// Lo que la sonda averiguó de la cuenta.
struct Sonda {
    autentica: Option<bool>,
    hay_conexion: Option<bool>,
    clave_i18n: Option<String>,
}

/// Pregunta al servidor si esta cuenta autentica **ahora**.
///
/// No devuelve error: todo lo que puede salir mal aquí es un punto del
/// preflight que no pasa, y un preflight que se cae con un error deja al
/// usuario sin los otros siete puntos — que son los que le dicen qué arreglar.
fn sondear_la_cuenta(
    db: &arles_db::Db,
    secretos: &dyn crate::fabrica::FuenteDeContrasenas,
    cuenta: arles_core::ids::EmailAccountId,
) -> Sonda {
    let proveedor =
        crate::fabrica::proveedor_de_la_cuenta(db, secretos, cuenta, contenido_de_prueba());

    match proveedor {
        Ok(p) => {
            let d = p.probar();
            Sonda {
                autentica: d.autentica(),
                hay_conexion: d.hay_conexion(),
                clave_i18n: Some(d.clave_i18n().to_owned()),
            }
        }
        // Ver la nota del comando: una cuenta que no puede enviar no se
        // sondea, y su punto de conexión se queda sin comprobar porque no se
        // llegó a mirar. Decir `Some(false)` ahí mandaría a revisar la red.
        Err(_) => Sonda {
            autentica: Some(false),
            hay_conexion: None,
            clave_i18n: None,
        },
    }
}

// ── La puerta del canal de WhatsApp · entrega W.5 ────────────────────────

/// Lo que la pantalla enseña antes de activar una campaña de WhatsApp.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ NO ES `RevisionDeCampana` CON UN CAMPO MÁS
///
/// Porque los ocho puntos del preflight de correo hablan de SMTP —autentica,
/// hay conexión, los registros del dominio—, y en WhatsApp **ninguno de los
/// tres significa nada**: no hay servidor al que conectarse con credenciales
/// propias, ni dominio que publicar. Meterlos aquí como «sin comprobar»
/// llenaría la pantalla de puntos que nunca se van a poder contestar.
///
/// Y al revés: las dos cifras de permiso, Coexistencia y la aceptación de
/// riesgo no existen en correo. Son revisiones distintas porque los canales
/// son distintos, y forzarlas al mismo tipo obligaría a que la mitad de los
/// campos fueran `Option` sin significado.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionDeWhatsApp {
    /// Cuánta gente recibiría el mensaje.
    pub audiencia: u32,
    /// De ésos, a cuántos consta permiso vigente.
    pub con_permiso: u32,
    /// Y a cuántos no. Es la cifra que cambia el botón de activar.
    pub sin_permiso: u32,
    /// ¿Está confirmada Coexistencia (L-11)?
    pub coexistencia: bool,
    /// ¿Se aceptó el riesgo del modo en frío?
    pub riesgo_aceptado: bool,
    /// ¿Este número escribe en frío?
    pub escribe_en_frio: bool,
    /// Cuántas plantillas aprobadas hay.
    pub plantillas_aprobadas: u32,
    /// ¿Se puede activar?
    pub se_puede_activar: bool,
    /// ¿Hay que escribir una palabra para confirmar?
    pub pide_palabra_de_confirmacion: bool,
    /// La calidad que Meta asigna al número, si se pudo consultar.
    ///
    /// **`None` significa «no se pudo preguntar», no «está bien».** Un semáforo
    /// que se pone en verde cuando no ha podido mirar es peor que no tener
    /// semáforo: es la misma regla del «sin comprobar» del preflight de correo.
    pub calidad: Option<String>,
}

/// Revisa una campaña de WhatsApp antes de activarla.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA CALIDAD SE PREGUNTA, LAS RESPUESTAS NO SE PUEDEN PREGUNTAR
///
/// De las tres cosas que `LOGISTICA_DE_CAMPANAS.md` esperaba de los avisos de
/// Meta, **ésta sí se puede consultar**: Cloud API expone la calidad del
/// número, así que el semáforo no depende de resolver dónde se alojan los
/// avisos. Se pregunta aquí, antes de cada campaña.
///
/// Lo que no se puede consultar son las respuestas de la gente: no hay forma
/// de pedirle a Cloud API los mensajes entrantes, y ésa es la parte que queda
/// pendiente de la decisión de Dirección (W.4).
///
/// Que la consulta falle **no es un error de este comando**: la calidad queda
/// en `None` y la pantalla dice «no se pudo consultar». Tratarlo como error
/// impediría revisar una campaña por una red caída, que es justo cuando más
/// falta hace ver las cifras de permiso.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada, si el identificador no vale, o
/// si la campaña no existe o no tiene etapa de WhatsApp.
#[tauri::command]
pub fn revisar_campana_de_whatsapp(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<RevisionDeWhatsApp, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let campana = id
        .parse::<arles_core::ids::CampaignId>()
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    let sabido = estado
        .db()
        .lo_que_se_sabe_de_whatsapp(empresa, campana)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(RevisionDeWhatsApp {
        audiencia: sabido.audiencia,
        con_permiso: sabido.con_permiso,
        sin_permiso: sabido.sin_permiso,
        coexistencia: sabido.coexistencia,
        riesgo_aceptado: sabido.riesgo_aceptado,
        escribe_en_frio: sabido.escribe_en_frio,
        plantillas_aprobadas: sabido.plantillas_aprobadas,
        se_puede_activar: sabido.se_puede_activar(),
        pide_palabra_de_confirmacion: sabido.pide_palabra_de_confirmacion(),
        // Todavía no se sondea a Meta desde aquí: queda en «no se pudo
        // consultar», que es lo que es. Enchufarlo es la entrega W.6.
        calidad: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── La puerta del canal de WhatsApp · entrega W.5 ─────────────────────

    /// **Lo que cruza la frontera no lleva el token de Meta ni el número.**
    ///
    /// La revisión se construye a partir de un registro que sí tiene
    /// `numero` —la pantalla necesita las cifras, no la credencial—, y se
    /// busca en el JSON. La regla 3 de la frontera IPC dice que ningún comando
    /// devuelve un secreto, y el identificador del número de teléfono de Meta
    /// es con lo que se envía en su nombre.
    ///
    /// Se serializa de verdad en vez de leer la definición del `struct`:
    /// si alguien añade un campo mañana, esta prueba lo dice.
    ///
    /// Verificada rompiéndola el 29/09/2026: añadiendo `numero` a
    /// `RevisionDeWhatsApp` y rellenándolo, esta prueba falla.
    #[test]
    fn la_revision_de_whatsapp_no_lleva_el_numero_ni_el_token() {
        let r = RevisionDeWhatsApp {
            audiencia: 847,
            con_permiso: 12,
            sin_permiso: 835,
            coexistencia: true,
            riesgo_aceptado: false,
            escribe_en_frio: true,
            plantillas_aprobadas: 2,
            se_puede_activar: false,
            pide_palabra_de_confirmacion: true,
            calidad: None,
        };

        let json = serde_json::to_string(&r).expect("serializa");
        assert!(!json.contains("+52"), "no sale el número: {json}");
        assert!(!json.contains("llavero://"), "ni la referencia: {json}");
        assert!(
            !json.contains("phone_number_id"),
            "ni el id de Meta: {json}"
        );
        assert!(json.contains("835"), "pero sí las cifras: {json}");
    }

    /// **Sin calidad consultada, el campo va nulo — no «bien».**
    ///
    /// Un semáforo que se pone en verde cuando no ha podido mirar es peor que
    /// no tener semáforo. La pantalla tiene que poder distinguir «la calidad
    /// es alta» de «no se pudo preguntar», y para eso el `None` tiene que
    /// llegar como `null` y no desaparecer del JSON.
    #[test]
    fn la_calidad_sin_consultar_llega_como_nula_y_no_desaparece() {
        let r = RevisionDeWhatsApp {
            audiencia: 1,
            con_permiso: 1,
            sin_permiso: 0,
            coexistencia: true,
            riesgo_aceptado: false,
            escribe_en_frio: false,
            plantillas_aprobadas: 1,
            se_puede_activar: true,
            pide_palabra_de_confirmacion: false,
            calidad: None,
        };

        let json = serde_json::to_string(&r).expect("serializa");
        assert!(
            json.contains("\"calidad\":null"),
            "el campo tiene que estar, y nulo: {json}"
        );
    }

    // ── Remitentes · entrega 5.3 ─────────────────────────────────────────

    /// **Lo que sale hacia la webview no lleva la contraseña.**
    ///
    /// Se serializa una cuenta de verdad y se busca el secreto en el JSON, en
    /// vez de leer la definición del `struct`: lo que importa es lo que cruza
    /// la frontera, no lo que creemos haber declarado. Si alguien añade un
    /// campo mañana, esta prueba lo dice.
    ///
    /// Verificada rompiéndola el 23/09/2026, y con algo que merece anotarse:
    /// el campo que se añadió para romperla se llamaba `clave_de_acceso`, no
    /// «contraseña», y esta prueba lo cazó igual **porque busca el valor**. La
    /// invariante del validador busca el nombre del campo, así que ésa no lo
    /// habría visto. Las dos hacen falta: una caza al que guarda un secreto con
    /// un nombre inocente, la otra al que lo declara sin rellenarlo todavía.
    #[test]
    fn lo_que_cruza_la_frontera_no_lleva_la_contrasena() {
        const CONTRASENA: &str = "la-contrasena-de-aplicacion";

        let cuenta = RemitenteEnPantalla {
            id: "01900000-0000-7000-8000-00000000000a".to_owned(),
            servidor: "smtp.gmail.com".to_owned(),
            puerto: 587,
            cifrado: "starttls".to_owned(),
            usuario: "ventas@telemetry.mx".to_owned(),
            remitente: "ventas@telemetry.mx".to_owned(),
            nombre_visible: Some("TELEMETRY".to_owned()),
            estado: "connected".to_owned(),
            limite_diario: 50,
            limite_por_hora: 10,
        };

        let json = serde_json::to_string(&cuenta).expect("serializa");
        assert!(
            !json.contains(CONTRASENA) && !json.contains("contrasena"),
            "la cuenta que va a la pantalla no puede llevar contraseña: {json}"
        );
        // Y sí lleva lo que la pantalla necesita para pintarse.
        assert!(json.contains("smtp.gmail.com") && json.contains("starttls"));
    }

    /// El resultado de guardar o probar tampoco.
    #[test]
    fn el_resultado_de_probar_no_lleva_la_contrasena() {
        let r = ResultadoDeRemitente {
            diagnostico: Some("correo.prueba.contrasena_rechazada".to_owned()),
            sirve: false,
            remitentes: Vec::new(),
        };
        let json = serde_json::to_string(&r).expect("serializa");
        assert!(!json.contains("contrasena\":"), "{json}");
        // La clave del diagnóstico sí va: es lo que la pantalla traduce.
        assert!(json.contains("correo.prueba.contrasena_rechazada"));
    }

    /// **Una cuenta que sale no puede volver a entrar.**
    ///
    /// La misma regla que ya rige para los contactos: si `RemitenteEnPantalla`
    /// se pudiera deserializar, la webview podría mandar una cuenta «ya
    /// formada» —con el estado que quisiera— y saltarse la validación del
    /// núcleo. Lo que entra es un `BorradorDeCuenta` y nada más.
    #[test]
    fn el_remitente_que_sale_no_puede_volver_a_entrar() {
        fn solo_si_deserializa<T: serde::de::DeserializeOwned>() {}
        // Esta línea NO compila si alguien le pone `Deserialize` al tipo, que
        // es exactamente la señal que se busca:
        // solo_si_deserializa::<RemitenteEnPantalla>();
        solo_si_deserializa::<arles_core::correo::BorradorDeCuenta>();
    }

    /// Los seis diagnósticos que puede devolver la prueba tienen texto.
    ///
    /// La pantalla resuelve la clave contra el catálogo; una clave sin texto
    /// enseñaría «correo.prueba.no_se_llego» en la cara del usuario (§139).
    #[test]
    fn el_diagnostico_que_cruza_es_una_clave_y_no_un_texto() {
        use arles_email::Diagnostico;
        for d in [
            Diagnostico::Correcto,
            Diagnostico::NoSeLlegoAlServidor,
            Diagnostico::ElServidorTardoDemasiado,
            Diagnostico::ElCifradoNoSePudoEstablecer,
            Diagnostico::LaContrasenaNoSirve,
            Diagnostico::ElServidorNoEstaDisponible,
        ] {
            let clave = d.clave_i18n();
            assert!(
                clave.starts_with("correo.prueba."),
                "lo que cruza tiene que ser una clave: {clave}"
            );
            assert!(
                !clave.contains(' '),
                "una clave con espacios es un texto disfrazado: {clave}"
            );
        }
    }

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

    /// La constante de la preferencia tiene que estar en la lista cerrada de
    /// `arles-db`. Si alguien la renombra en un sitio y no en el otro, la barra
    /// lateral deja de recordarse **en silencio**: el comando falla y la
    /// interfaz se limita a no plegar nada.
    #[test]
    fn la_clave_de_la_barra_esta_declarada_en_la_capa_de_datos() {
        assert!(
            arles_db::CLAVES_DE_INTERFAZ.contains(&BARRA_PLEGADA),
            "«{BARRA_PLEGADA}» no está en las claves admitidas: {:?}",
            arles_db::CLAVES_DE_INTERFAZ
        );
    }

    /// Lo mismo para el tema. Aquí el fallo silencioso sería peor: el usuario
    /// elige «claro», la pantalla se pone clara, y al reabrir vuelve a oscuro
    /// sin ningún mensaje.
    #[test]
    fn la_clave_del_tema_esta_declarada_en_la_capa_de_datos() {
        assert!(
            arles_db::CLAVES_DE_INTERFAZ.contains(&TEMA),
            "«{TEMA}» no está en las claves admitidas: {:?}",
            arles_db::CLAVES_DE_INTERFAZ
        );
    }

    /// C-2: sin haber elegido nada, el tema es «auto», que es lo que hace que
    /// ARLES siga al sistema operativo. Si el primero fuera «oscuro», una
    /// instalación nueva ignoraría el sistema y nadie lo notaría hasta abrirla
    /// en un equipo configurado en claro.
    #[test]
    fn el_tema_por_defecto_sigue_al_sistema() {
        assert_eq!(TEMA_POR_DEFECTO, "auto");
        assert!(TEMAS.contains(&TEMA_POR_DEFECTO));
        assert_eq!(TEMAS, ["auto", "oscuro", "claro"]);
    }

    /// El valor guardado acaba escrito en `data-tema`. Si admitiéramos
    /// cualquier cadena, el atributo podría llevar cualquier cosa.
    #[test]
    fn un_tema_inventado_no_es_admisible() {
        assert!(!TEMAS.contains(&"rosa"));
        assert!(!TEMAS.contains(&""));
        let ipc: ErrorIpc = crate::AppError::TemaDesconocido.into();
        assert_eq!(ipc.clave, "error.app.tema_desconocido");
    }

    /// Lo que el desplegable ofrece y lo que el núcleo acepta son el mismo
    /// dato. El comando no puede copiar la lista: la reexporta.
    #[test]
    fn la_configuracion_ofrece_las_zonas_del_nucleo() {
        assert!(std::ptr::eq(
            arles_core::ZONAS_SOPORTADAS,
            arles_core::empresa::ZONAS_SOPORTADAS
        ));
        assert!(arles_core::ZONAS_SOPORTADAS.contains(&"America/Mexico_City"));
    }

    /// El error del formulario de contacto llega con **el índice** del canal.
    ///
    /// Sin él, la pantalla sabe que «un canal es inválido» pero no cuál, y con
    /// diez canales eso obliga a repasarlos todos (§95).
    #[test]
    fn el_error_de_contacto_dice_que_canal_fallo() {
        use arles_core::contacto::{CampoDeContacto, ErrorDeContacto};

        let ipc: ErrorIpc = crate::AppError::ContactoInvalido(vec![ErrorDeContacto {
            campo: CampoDeContacto::Canal,
            indice: Some(3),
            clave: "error.telefono_invalido",
        }])
        .into();

        assert_eq!(ipc.clave, "error.app.contacto_invalido");
        assert_eq!(ipc.canales.first().and_then(|c| c.indice), Some(3));
        // Y no se cuela en la lista del otro formulario.
        assert!(ipc.campos.is_empty());

        let json = serde_json::to_string(&ipc).expect("serializa");
        assert!(json.contains("\"indice\":3"), "{json}");
    }

    /// Un contacto guardado sale hacia la pantalla; **no entra**.
    ///
    /// Lo que entra es un borrador, que se valida. Si `ContactoParaLaPantalla`
    /// se pudiera deserializar, la webview podría mandar canales ya
    /// «normalizados» a su gusto, y la deduplicación y la supresión —que
    /// comparan esa forma— dejarían de funcionar sin que nada fallara.
    #[test]
    /// Se comprobó descomentando la línea: no compila. Y añadiendo
    /// `Deserialize` a la estructura tampoco, porque `CanalValidado` tampoco lo
    /// tiene — la barrera está puesta dos veces, en el núcleo y aquí.
    fn el_contacto_que_sale_no_puede_volver_a_entrar() {
        fn solo_si_deserializa<T: serde::de::DeserializeOwned>() {}
        // Esta línea no compila:
        //     solo_si_deserializa::<ContactoParaLaPantalla>();
        //
        // Y ésta sí, que es la puerta de entrada buena: un borrador, que hay
        // que validar antes de que sea un contacto.
        solo_si_deserializa::<arles_core::contacto::BorradorDeContacto>();
    }

    /// Los canales del contacto salen en el JSON con su forma normalizada
    /// **y** con lo que el usuario escribió. La tabla enseña lo segundo; la
    /// supresión compara lo primero.
    #[test]
    fn el_contacto_lleva_las_dos_formas_de_cada_canal() {
        let datos = arles_core::contacto::DatosDeContacto::validar(
            &arles_core::contacto::BorradorDeContacto {
                nombre: "Ana".into(),
                apellido: String::new(),
                empresa: String::new(),
                canales: vec![arles_core::contacto::BorradorDeCanal {
                    canal: arles_core::Canal::Correo,
                    valor: "Ana@Empresa.MX".into(),
                    principal: true,
                }],
            },
            "MX",
        )
        .expect("válido");

        let para_pantalla = ContactoParaLaPantalla::from(arles_db::ContactoGuardado {
            id: arles_core::ids::ContactId::nuevo(),
            datos,
            creado_en: "2026-09-17T10:00:00Z".into(),
            actualizado_en: "2026-09-17T10:00:00Z".into(),
        });

        let json = serde_json::to_string(&para_pantalla).expect("serializa");
        assert!(json.contains("Ana@Empresa.MX"), "falta lo que se escribió");
        assert!(
            json.contains("ana@empresa.mx"),
            "falta la forma normalizada"
        );
        // camelCase en la frontera, como el resto de comandos.
        assert!(json.contains("valorNormalizado"), "{json}");
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
    // ── Entrega 3.4 · la frontera de la supresión ──────────────────────────

    /// **La regla viaja resuelta, no como dato en bruto.**
    ///
    /// La pantalla recibe `sePuedeLevantar` ya decidido por el núcleo. Si en
    /// vez de eso recibiera el motivo y decidiera ella, tendría una segunda
    /// copia de la regla — y el día que divergieran enseñaría un botón que el
    /// núcleo rechaza, o escondería uno que sí valía.
    #[test]
    fn la_fila_de_supresion_viaja_con_la_regla_ya_resuelta() {
        use arles_core::supresion::{AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion};

        for (motivo, esperado) in [
            (MotivoDeSupresion::Baja, false),
            (MotivoDeSupresion::Queja, false),
            (MotivoDeSupresion::RebotePermanente, true),
            (MotivoDeSupresion::Manual, true),
        ] {
            let fila = SupresionParaLaPantalla::from(arles_db::SupresionGuardada {
                id: arles_core::ids::SuppressionEntryId::nuevo(),
                canal: arles_core::Canal::Correo,
                direccion: "ana@empresa.mx".into(),
                alcance: AlcanceDeSupresion::Canal,
                motivo,
                origen: OrigenDeSupresion::Usuario,
                notas: String::new(),
                creada: "2026-09-21T00:00:00Z".into(),
                peticion: None,
                de_quien: None,
            });
            assert_eq!(
                fila.se_puede_levantar,
                esperado,
                "«{}» cruzó la frontera con la regla equivocada",
                motivo.como_texto()
            );
            assert_eq!(fila.motivo, motivo.clave_i18n());
        }
    }

    /// `de_quien` cruza como cadena vacía, nunca como `null`.
    ///
    /// El frontend tiene `exactOptionalPropertyTypes`: un `undefined` que cruza
    /// obliga a cada componente a contemplar tres casos donde hay dos.
    #[test]
    fn una_direccion_sin_dueno_cruza_como_cadena_vacia() {
        use arles_core::supresion::{AlcanceDeSupresion, MotivoDeSupresion, OrigenDeSupresion};

        let fila = SupresionParaLaPantalla::from(arles_db::SupresionGuardada {
            id: arles_core::ids::SuppressionEntryId::nuevo(),
            canal: arles_core::Canal::WhatsApp,
            direccion: "+528112345678".into(),
            alcance: AlcanceDeSupresion::Global,
            motivo: MotivoDeSupresion::Manual,
            origen: OrigenDeSupresion::Usuario,
            notas: String::new(),
            creada: "2026-09-21T00:00:00Z".into(),
            peticion: Some("p1".into()),
            de_quien: None,
        });

        let json = serde_json::to_string(&fila).expect("serializa");
        assert!(!json.contains("null"), "cruzó un null: {json}");
        assert!(json.contains("\"deQuien\":\"\""), "{json}");
        assert!(json.contains("\"global\":true"), "{json}");
    }

    /// La página de la frontera no puede pedir más de lo que el repositorio da.
    ///
    /// Es una comprobación **en tiempo de compilación** junto a la constante;
    /// esto sólo deja constancia de por qué existe.
    #[test]
    fn la_pagina_de_supresiones_cabe_en_la_del_repositorio() {
        assert!(
            SUPRESIONES_POR_PAGINA as usize <= arles_db::MAX_SUPRESIONES_POR_PAGINA,
            "la pantalla pediría más filas de las que el repositorio devuelve, \
             y la paginación saltaría filas sin decirlo"
        );
    }
    // ── Entrega 3.4 · el derecho de Acceso ─────────────────────────────────

    /// El nombre propuesto **no puede ser una ruta**.
    ///
    /// Una barra en un nombre de archivo lo convierte en una ruta, y una
    /// arroba o un `+` son legales en Linux y problemáticos en otros sitios.
    /// La dirección viene de fuera: puede ser cualquier cosa que pase el
    /// validador.
    #[test]
    fn el_nombre_propuesto_nunca_es_una_ruta() {
        for direccion in [
            "ana@empresa.mx",
            "+528112345678",
            "a.b-c_d@sub.dominio.mx",
            "raro+etiqueta@empresa.mx",
        ] {
            let n = nombre_seguro(direccion);
            assert!(!n.contains('/'), "«{direccion}» salió con barra: {n}");
            assert!(!n.contains('\\'), "«{direccion}» salió con barra: {n}");
            assert!(!n.contains(".."), "«{direccion}» salió con ascenso: {n}");
            assert!(!n.contains('@'), "«{direccion}» salió con arroba: {n}");
            assert!(!n.is_empty(), "«{direccion}» se quedó sin nombre");
        }
        // Y lo que sí vale se conserva, o el nombre no diría de quién es.
        assert_eq!(nombre_seguro("ana.ruiz@empresa.mx"), "ana.ruiz-empresa.mx");
    }

    /// Un nombre interminable se recorta: los sistemas de archivos tienen un
    /// tope, y pasarse deja el informe sin escribir justo cuando hace falta.
    #[test]
    fn el_nombre_propuesto_se_recorta() {
        let largo = format!("{}@empresa.mx", "a".repeat(500));
        assert!(nombre_seguro(&largo).chars().count() <= 60);
    }

    /// La fecha del informe se escribe **para una persona**.
    ///
    /// `2026-09-21T14:03:22Z` es correcto para una base de datos y no es una
    /// fecha para quien preguntó por sus datos.
    #[test]
    fn la_fecha_del_informe_se_lee_como_una_fecha() {
        let f = hoy_legible();
        assert!(f.contains(" de "), "no parece una fecha en palabras: {f}");
        assert!(!f.contains('T'), "salió en formato de máquina: {f}");
        assert!(!f.contains('Z'), "salió en formato de máquina: {f}");
        assert!(f.split(" de ").count() == 3, "faltan día, mes o año: {f}");
        // El mes es una palabra, no un número.
        let mes = f.split(" de ").nth(1).unwrap_or("");
        assert!(
            mes.chars().all(|c| c.is_alphabetic()),
            "el mes salió como número: {f}"
        );
    }

    /// Lo que cruza la frontera al entregar el informe **no lleva ninguna
    /// ruta**: sólo el nombre del archivo.
    #[test]
    fn el_informe_entregado_no_cruza_ninguna_ruta() {
        let json = serde_json::to_string(&InformeEntregado {
            nombre_del_archivo: "datos-de-ana-empresa.mx.txt".to_owned(),
            habia_datos: true,
            sigue_siendo_contacto: false,
        })
        .expect("serializa");

        assert!(!json.contains('/'), "cruzó una ruta: {json}");
        assert!(!json.contains("\\"), "cruzó una ruta: {json}");
        assert!(!json.contains("home"), "{json}");
        assert!(!json.contains("Users"), "{json}");
        assert!(json.contains("nombreDelArchivo"), "{json}");
    }
}
