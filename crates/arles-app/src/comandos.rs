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
fn empresa_actual(estado: &EstadoApp) -> Result<arles_core::ids::CompanyId, crate::AppError> {
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
fn pais_de_la_empresa(estado: &EstadoApp) -> Result<String, ErrorIpc> {
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
fn nombre_seguro(direccion: &str) -> String {
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
fn hoy_legible() -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

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
