//! Los comandos de campañas, plantillas, firmas y horarios.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ HAY TIPOS AQUÍ QUE PARECEN COPIAS DE LOS DE `arles-db`
//!
//! `PlantillaIpc` se parece mucho a `PlantillaGuardada`, y no es descuido:
//! **`arles-db` no depende de `serde` a propósito** (`ARQUITECTURA.md` §4). La
//! capa de datos no sabe que existe una webview, y no debe saberlo — el día
//! que la interfaz cambie, o que haya dos, la base no se entera.
//!
//! El precio es traducir en la frontera. Se paga a gusto, porque la traducción
//! es además donde se decide **qué sale y qué no**, y eso conviene que sea una
//! decisión explícita y no lo que quedó al derivar `Serialize`.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LOS COMANDOS SON GRUESOS
//!
//! `guardar_plantilla` devuelve la lista entera, no sólo el id. Es la regla 1
//! de la frontera: cada cruce es una oportunidad de estado inconsistente, y
//! «guarda» seguido de «lista» son dos, con una ventana entre medias en la que
//! la pantalla enseña algo que ya no es cierto.

use serde::{Deserialize, Serialize};

use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Por dónde va a salir una campaña, tal como lo manda la pantalla (D-WA).
///
/// ─────────────────────────────────────────────────────────────────────────
/// UNA VARIANTE ETIQUETADA, Y NO SIETE CAMPOS OPCIONALES
///
/// `PorDondeSale` en `arles-db` hace imposible el estado imposible dentro de
/// Rust: no se puede construir una etapa de WhatsApp con una plantilla de
/// correo. **Pero si la frontera recibiera siete campos opcionales y los
/// armara aquí, el estado imposible volvería a caber** — en el único sitio
/// donde los datos vienen de fuera, que es justo donde más importa.
///
/// Con la etiqueta **y `deny_unknown_fields`**, `serde` rechaza la mezcla antes
/// de que ninguna línea de ARLES la vea: un `{ canal: 'whatsapp', cuenta: '…' }`
/// no deserializa y el comando no llega a correr. Es la misma razón por la que
/// un `CanalValidado` no se puede deserializar (§4 de la arquitectura): lo que
/// la webview puede mandar tiene que ser exactamente lo que es válido.
///
/// ── Y LAS DOS COSAS HACEN FALTA · 27/10/2026 ──
///
/// La primera versión de este comentario afirmaba que la etiqueta sola
/// bastaba. **Era falso, y lo demostró la prueba**: por defecto `serde`
/// **ignora** los campos que no conoce, así que
/// `{ canal: 'whatsapp', numero: '…', cuenta: '…' }` entraba perfectamente y
/// la `cuenta` se tiraba en silencio.
///
/// Eso no es inofensivo: quien se equivoque de campo en la pantalla vería la
/// campaña crearse como si todo estuviera bien, y el dato que escribió no
/// estaría. Es el §2 aplicado a mí mismo — una afirmación de verificación es
/// una afirmación como cualquier otra, y en este proyecto son las que más veces
/// resultan falsas. Lo caza `una_mezcla_de_canales_no_cruza_la_frontera`.
///
/// La etiqueta es `canal` y sus valores `email` y `whatsapp`, que es como el
/// núcleo serializa [`arles_core::canal::Canal`] y lo que el frontend ya usa
/// en todas partes. Dos nombres para lo mismo serían dos sitios donde
/// equivocarse.
///
/// Los identificadores llegan como texto y no como tipos del núcleo porque es
/// lo que la webview tiene: los que le dieron las listas. Quien comprueba que
/// existen son las claves ajenas de `campaign_stage`, y un valor inventado
/// hace fallar la creación entera en vez de crear una campaña que apunta a
/// nada.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "canal", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum PorDondeSaleIpc {
    #[serde(rename = "email")]
    Correo {
        cuenta: String,
        version_de_plantilla: String,
        /// La firma del pie, o `None` para la de por defecto de la empresa
        /// (H-12).
        firma: Option<String>,
    },
    /// Por WhatsApp. **No lleva firma**, y no por olvido: una firma es cosa del
    /// correo y el esquema lo exige con un `CHECK` desde la V4. Aquí ni se
    /// puede escribir.
    #[serde(rename = "whatsapp")]
    Whatsapp {
        numero: String,
        /// La plantilla aprobada de Meta. Opcional porque la etapa nace antes
        /// de elegirla —el asistente pregunta el canal y el número primero—, y
        /// **quien la exige es `activar_campana`**, que es donde importa: ahí
        /// el error se puede explicar con las tres partes del §95 en vez de
        /// salir como un fallo de la base. El motivo entero está en la V16.
        plantilla_de_meta: Option<String>,
    },
}

/// Una versión de plantilla, tal como la ve la pantalla.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionIpc {
    pub id: String,
    pub numero: u32,
    pub asunto: String,
    /// Ya saneado en Rust. La pantalla lo pinta sin volver a tocarlo.
    pub cuerpo_html: String,
    /// Las claves de texto de las variables que usa (§139). **No los nombres**:
    /// la pantalla los traduce, y así «empresa» se puede llamar distinto en la
    /// interfaz sin romper ninguna plantilla guardada.
    pub variables: Vec<String>,
}

/// Una plantilla con su versión vigente.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantillaIpc {
    pub id: String,
    pub nombre: String,
    pub version: Option<VersionIpc>,
}

/// Una firma guardada.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmaIpc {
    pub id: String,
    pub nombre: String,
    pub cuerpo_html: String,
    pub por_defecto: bool,
}

/// Un horario de envío.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VentanaIpc {
    pub id: String,
    pub nombre: String,
    /// Máscara de bits: bit 0 = lunes … bit 6 = domingo.
    pub dias: u8,
    pub desde: u8,
    pub hasta: u8,
    pub zona: String,
}

/// Lo que queda al crear una campaña: su id y el de su etapa de correo.
///
/// La etapa viaja porque el envío de prueba del §45 cuelga de ella, y el paso 8
/// del asistente va justo después del 1. Sin esto, la pantalla tendría que
/// reconstruir ese identificador, que es un detalle de `arles-db`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampanaCreadaIpc {
    pub id: String,
    pub etapa: String,
}

/// Una campaña en la lista.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampanaIpc {
    pub id: String,
    pub nombre: String,
    /// La clave del rótulo de estado (§139).
    pub estado: String,
    pub audiencia: u32,
    /// Cuántos **aceptó el proveedor**. Nunca «entregados» (§65).
    pub aceptados: u32,
}

/// Lo que pasó al activar. Cifras, no adjetivos (§94).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivacionIpc {
    pub en_la_audiencia: u32,
    pub fuera_por_baja: u32,
}

fn a_version(v: arles_db::plantillas::VersionGuardada) -> VersionIpc {
    VersionIpc {
        id: v.id.to_string(),
        numero: v.numero,
        asunto: v.asunto,
        cuerpo_html: v.cuerpo_html,
        variables: v.variables.iter().map(|x| x.clave().to_owned()).collect(),
    }
}

/// Las plantillas de la empresa, con su versión vigente.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o la base no responde.
#[tauri::command]
pub fn plantillas(estado: tauri::State<'_, EstadoApp>) -> Result<Vec<PlantillaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let todas = estado
        .db()
        .plantillas(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(todas
        .into_iter()
        .map(|p| PlantillaIpc {
            id: p.id.to_string(),
            nombre: p.nombre,
            version: p.version_actual.map(a_version),
        })
        .collect())
}

/// Guarda una plantilla nueva y devuelve la lista entera.
///
/// **El cuerpo llega sin sanear y se sanea en Rust** (§6.2). Lo que la webview
/// manda no es de fiar ni cuando lo escribió el propio usuario: el HTML pudo
/// venir pegado de cualquier sitio.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o la escritura falla.
#[tauri::command]
pub fn guardar_plantilla(
    estado: tauri::State<'_, EstadoApp>,
    nombre: String,
    asunto: String,
    cuerpo_html: String,
) -> Result<Vec<PlantillaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    estado
        .db()
        .crear_plantilla(
            empresa,
            &nombre,
            &arles_db::plantillas::Redaccion {
                asunto,
                cuerpo_html,
            },
        )
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    plantillas(estado)
}

/// Guarda **una versión nueva** de una plantilla que ya existe, y devuelve la
/// lista entera.
///
/// ── POR QUÉ ESTO NO ES «EDITAR» AUNQUE EL BOTÓN DIGA EDITAR ───────────────
///
/// Una versión no se reescribe nunca (§44): la campaña que ya se activó apunta
/// a la que eligió, y lo que está en cola sale con ese texto. Lo que hace este
/// comando es **dejar nacer la siguiente**, que es lo que Dirección pedía al
/// decir que no se podían editar las plantillas (R-06).
///
/// La diferencia hay que decirla en la pantalla, porque es contraintuitiva:
/// quien corrige una errata y ve «versión 3» puede creer que arregló el correo
/// que está saliendo, y es justo lo contrario.
///
/// `arles-db` tenía `nueva_version_de_plantilla` **desde la 6.3, probada, y
/// sin nadie que la llamara**: el patrón del §9 otra vez, esta vez en el tramo
/// base → frontera.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada, la plantilla no existe o la
/// escritura falla.
#[tauri::command]
pub fn nueva_version_de_plantilla(
    estado: tauri::State<'_, EstadoApp>,
    plantilla: String,
    asunto: String,
    cuerpo_html: String,
) -> Result<Vec<PlantillaIpc>, ErrorIpc> {
    crate::comandos::empresa_actual(&estado)?;
    let id = plantilla.parse().map_err(|_| {
        ErrorIpc::from(crate::AppError::Db(arles_db::DbError::DatoInvalido {
            motivo: "template.id",
        }))
    })?;

    estado
        .db()
        .nueva_version_de_plantilla(
            id,
            &arles_db::plantillas::Redaccion {
                asunto,
                cuerpo_html,
            },
        )
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    plantillas(estado)
}

/// Las firmas de la empresa.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o la base no responde.
#[tauri::command]
pub fn firmas(estado: tauri::State<'_, EstadoApp>) -> Result<Vec<FirmaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let todas = estado
        .db()
        .firmas(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(todas
        .into_iter()
        .map(|f| FirmaIpc {
            id: f.id.to_string(),
            nombre: f.nombre,
            cuerpo_html: f.cuerpo_html,
            por_defecto: f.por_defecto,
        })
        .collect())
}

/// Guarda una firma y devuelve la lista entera.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o la escritura falla.
#[tauri::command]
pub fn guardar_firma(
    estado: tauri::State<'_, EstadoApp>,
    nombre: String,
    cuerpo_html: String,
    por_defecto: bool,
) -> Result<Vec<FirmaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    estado
        .db()
        .crear_firma(empresa, &nombre, &cuerpo_html, por_defecto)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    firmas(estado)
}

/// Reescribe una firma que ya existe, y devuelve la lista entera.
///
/// A diferencia de una plantilla, una firma **se edita encima**: no tiene
/// versiones, así que no hay nada congelado que reescribir. El por qué está en
/// `arles_db::plantillas::Db::actualizar_firma`, incluido lo que habrá que
/// decidir el día que el contenido llegue al envío (H-12).
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada, la firma no es de esta empresa,
/// el cuerpo lleva variables de quien recibe (R-04) o la escritura falla.
#[tauri::command]
pub fn editar_firma(
    estado: tauri::State<'_, EstadoApp>,
    firma: String,
    nombre: String,
    cuerpo_html: String,
    por_defecto: bool,
) -> Result<Vec<FirmaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let id = firma.parse().map_err(|_| {
        ErrorIpc::from(crate::AppError::Db(arles_db::DbError::DatoInvalido {
            motivo: "signature.id",
        }))
    })?;

    estado
        .db()
        .actualizar_firma(empresa, id, &nombre, &cuerpo_html, por_defecto)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    firmas(estado)
}

/// Retira una plantilla —deja de ofrecerse, no se borra— y devuelve la lista.
///
/// El porqué está en `arles_db::plantillas::Db::retirar_plantilla`: lo que una
/// campaña envió tiene que poder consultarse, así que el texto se queda.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa, la plantilla no es suya, o la escritura
/// falla.
#[tauri::command]
pub fn retirar_plantilla(
    estado: tauri::State<'_, EstadoApp>,
    plantilla: String,
) -> Result<Vec<PlantillaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let id = plantilla.parse().map_err(|_| {
        ErrorIpc::from(crate::AppError::Db(arles_db::DbError::DatoInvalido {
            motivo: "template.id",
        }))
    })?;

    estado
        .db()
        .retirar_plantilla(empresa, id)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    plantillas(estado)
}

/// Retira una firma —y con ella la marca de «por defecto»— y devuelve la lista.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa, la firma no es suya, o la escritura falla.
#[tauri::command]
pub fn retirar_firma(
    estado: tauri::State<'_, EstadoApp>,
    firma: String,
) -> Result<Vec<FirmaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let id = firma.parse().map_err(|_| {
        ErrorIpc::from(crate::AppError::Db(arles_db::DbError::DatoInvalido {
            motivo: "signature.id",
        }))
    })?;

    estado
        .db()
        .retirar_firma(empresa, id)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    firmas(estado)
}

/// Los horarios de envío guardados.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o la base no responde.
#[tauri::command]
pub fn ventanas(estado: tauri::State<'_, EstadoApp>) -> Result<Vec<VentanaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let todas = estado
        .db()
        .ventanas(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(todas
        .into_iter()
        .map(|v| VentanaIpc {
            id: v.id,
            nombre: v.nombre,
            dias: v.dias,
            desde: v.desde,
            hasta: v.hasta,
            zona: v.zona,
        })
        .collect())
}

/// Guarda un horario de envío y devuelve la lista entera.
///
/// # Errores
///
/// [`ErrorIpc`] con la clave del motivo si la ventana no se abriría nunca: sin
/// días, con las horas al revés, o con una zona que no está en la base IANA.
/// Los tres se arreglan en campos distintos del formulario (§95).
#[tauri::command]
pub fn guardar_ventana(
    estado: tauri::State<'_, EstadoApp>,
    nombre: String,
    dias: u8,
    desde: u8,
    hasta: u8,
    zona: String,
) -> Result<Vec<VentanaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    estado
        .db()
        .guardar_ventana(empresa, &nombre, dias, desde, hasta, &zona)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    ventanas(estado)
}

/// Las campañas de la empresa.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o la base no responde.
#[tauri::command]
pub fn campanas(estado: tauri::State<'_, EstadoApp>) -> Result<Vec<CampanaIpc>, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let todas = estado
        .db()
        .campanas(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(todas
        .into_iter()
        .map(|c| CampanaIpc {
            id: c.id.to_string(),
            nombre: c.nombre,
            estado: c.estado.clave_i18n().to_owned(),
            audiencia: c.audiencia,
            aceptados: c.aceptados,
        })
        .collect())
}

/// Crea una campaña en borrador, con su etapa de correo.
///
/// Nace **en borrador**: crear no envía nada. Lo que empieza a mandar correos
/// es activar, y eso es otro comando y otra confirmación.
///
/// **El aviso de más de 50 diarios no se comprueba aquí**, y es a propósito: lo
/// exige `arles-db`, un piso más abajo, y deja su línea en la bitácora en la
/// misma transacción. Si la regla viviera en este comando, cualquier otro
/// camino hacia `crear_campana` se la saltaría sin que nada fallara.
///
/// # Errores
///
/// [`ErrorIpc`] si algún identificador no vale, si el remitente, la plantilla
/// o el horario no existen, o `error.db.falta_aceptar_el_volumen` si se piden
/// más de 50 diarios sin aceptar el aviso (§48).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn crear_campana(
    estado: tauri::State<'_, EstadoApp>,
    nombre: String,
    // Por dónde sale, con lo que ese canal necesita y nada más. Ver
    // [`PorDondeSaleIpc`]: la etiqueta impide que llegue una mezcla.
    sale_por: PorDondeSaleIpc,
    // ── A quién va dirigida (D-15) ───────────────────────────────────────
    //
    // Llega el identificador de una lista que ya existe, no un archivo: la
    // carga del XLSX vive en Contactos › Importar, con su revisión de choques
    // —que es la decisión de Dirección del 18/09/2026 y no se puede saltar—.
    // Aquí sólo se elige a cuál de las listas cargadas va esta campaña.
    //
    // `None` es «toda la empresa», y la pantalla lo ofrece con esas palabras.
    lista: Option<String>,
    ventana: String,
    limite_por_hora: Option<u32>,
    limite_diario: Option<u32>,
    acepto_el_riesgo_del_volumen: bool,
) -> Result<CampanaCreadaIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    crear_campana_en(
        estado.db(),
        empresa,
        &nombre,
        sale_por,
        lista.as_deref(),
        &ventana,
        limite_por_hora,
        limite_diario,
        acepto_el_riesgo_del_volumen,
    )
}

/// Lo mismo, con la base ya abierta.
///
/// ── POR QUÉ ESTÁ SEPARADO DEL COMANDO ──
///
/// Para poder probarlo contra una base de verdad. Un `tauri::State` no se
/// puede construir en una prueba, así que un comando que hace el trabajo
/// dentro **no se puede probar**: sólo queda mirarlo, que es como llegaron a
/// producción los huecos del §9. Es el mismo patrón que `registro_de` y que
/// `revision_de`.
///
/// # Errores
///
/// [`ErrorIpc`] si algún identificador no vale o si la base rechaza la
/// creación.
#[allow(clippy::too_many_arguments)]
pub fn crear_campana_en(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
    nombre: &str,
    sale_por: PorDondeSaleIpc,
    lista: Option<&str>,
    ventana: &str,
    limite_por_hora: Option<u32>,
    limite_diario: Option<u32>,
    acepto_el_riesgo_del_volumen: bool,
) -> Result<CampanaCreadaIpc, ErrorIpc> {
    // Los identificadores se estrechan ANTES de tocar la base. Un valor
    // inventado hace fallar aquí, con la clave de qué dato no vale, en vez de
    // llegar como un fallo de clave ajena de SQLite tres capas más abajo.
    //
    // El `match` no tiene `_ =>`: añadir un canal mañana no compila hasta
    // decidir qué se estrecha, que es exactamente donde la decisión importa.
    let sale_por = match &sale_por {
        PorDondeSaleIpc::Correo {
            cuenta,
            version_de_plantilla,
            firma,
        } => arles_db::alta_de_campana::PorDondeSale::Correo {
            cuenta: cuenta
                .parse::<arles_core::ids::EmailAccountId>()
                .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?,
            version_de_plantilla: version_de_plantilla
                .parse::<arles_core::ids::TemplateVersionId>()
                .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?,
            firma: firma.as_deref(),
        },
        PorDondeSaleIpc::Whatsapp {
            numero,
            plantilla_de_meta,
        } => arles_db::alta_de_campana::PorDondeSale::Whatsapp {
            numero: numero
                .parse::<arles_core::ids::WhatsappAccountId>()
                .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?,
            // Una cadena en blanco es «no se eligió», igual que la lista: la
            // webview manda `''` cuando un desplegable está vacío, y tratarlo
            // como un identificador daría un error de dato inválido en vez de
            // dejar la etapa sin plantilla — que es un estado legítimo hasta
            // que se active.
            plantilla_de_meta: match plantilla_de_meta
                .as_deref()
                .map(str::trim)
                .filter(|p| !p.is_empty())
            {
                Some(p) => Some(
                    p.parse::<arles_core::ids::WhatsappTemplateId>()
                        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?,
                ),
                None => None,
            },
        },
    };

    // Una lista inventada hace fallar la creación entera —es una clave ajena—
    // en vez de crear una campaña que apunta a nada. Mismo trato que la firma.
    let lista = match lista.map(str::trim).filter(|l| !l.is_empty()) {
        Some(l) => Some(
            l.parse::<arles_core::ids::ContactListId>()
                .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?,
        ),
        None => None,
    };

    let creada = db
        .crear_campana(
            empresa,
            &arles_db::alta_de_campana::CampanaNueva {
                lista,
                nombre,
                sale_por,
                ventana,
                limite_por_hora,
                limite_diario,
                acepto_el_riesgo_del_volumen,
            },
        )
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(CampanaCreadaIpc {
        id: creada.id.to_string(),
        etapa: creada.etapa,
    })
}

/// Activa una campaña: congela la audiencia y empieza a enviar.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTE COMANDO NO SE PUEDE DESHACER, Y NO COMPRUEBA EL PREFLIGHT
///
/// Lo segundo es deliberado y conviene que esté escrito. El preflight vive en
/// `revisar_campana`, y la pantalla no deja pulsar «activar» hasta que sale
/// «lista».
///
/// Repetirlo aquí sería una segunda comprobación con su propia sonda a la red,
/// que puede dar un resultado distinto al de hace dos segundos y dejar al
/// usuario ante un botón encendido que no hace nada. Y el preflight **no es un
/// control de seguridad**: es una ayuda para no equivocarse. Lo que sí es
/// invariante —que la audiencia se congele entera o no se congele— lo garantiza
/// la transacción de `activar_campana`, un piso más abajo.
///
/// # Errores
///
/// [`ErrorIpc`] si la campaña no existe, no está en un estado desde el que se
/// pueda activar, o si al aplicar las bajas no queda nadie a quien escribir
/// —`error.db.audiencia_vacia`—, que es el caso que más duele descubrir después.
#[tauri::command]
pub fn activar_campana(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<ActivacionIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let campana = id
        .parse::<arles_core::ids::CampaignId>()
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    let a = estado
        .db()
        .activar_campana(empresa, campana)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    Ok(ActivacionIpc {
        en_la_audiencia: a.en_la_audiencia,
        fuera_por_baja: a.fuera_por_baja,
    })
}
