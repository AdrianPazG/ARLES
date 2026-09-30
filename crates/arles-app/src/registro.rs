//! El registro de actividad, en la frontera.
//!
//! Entrega 7.4. Lo que D-7C define, con las palabras de Dirección del
//! 29/09/2026:
//!
//! > «Se recorta, pero no se borra, me gustaría que tuviera los registros
//! > bimestrales o en caso que sean demasiados, más de 20 000 correos y
//! > mensajes juntos, pues que solo sea posible visualizar los últimos 500
//! > registros, así que debería tener un filtro del registro, para poder buscar
//! > entre todos los registros, todo perfectamente estructurado y ordenado.»
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! LAS DOS MITADES, Y POR QUÉ VIAJAN JUNTAS
//!
//! La página y el total salen de la misma consulta (regla 1 de la frontera:
//! comandos gruesos). Pedirlos por separado abre una ventana en la que la
//! pantalla enseña 500 filas y un total que ya no corresponde — y el total es
//! precisamente lo que impide que el tope esconda algo: «500 de 3 412».
//!
//! Las clases de suceso —para el desplegable de «qué pasó»— viajan en el mismo
//! viaje por lo mismo: con un comando aparte, el desplegable puede ofrecer una
//! opción que la lista de al lado ya no tiene.

use serde::{Deserialize, Serialize};

use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Una fila del registro.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SucesoIpc {
    pub id: String,
    /// La clave de i18n (§139). La pantalla la traduce; aquí no viaja texto.
    pub clave: String,
    /// `true` si es el desenlace de un envío. **No se borra nunca** (D-7C).
    pub es_desenlace: bool,
    pub campana: Option<String>,
    pub canal: Option<String>,
    /// A quién se le escribió.
    ///
    /// Sale de `message_attempt` y no del registro, que no guarda ninguna
    /// dirección (P-17). `null` en los sucesos de bucle, que no son de nadie.
    pub direccion: Option<String>,
    /// Las cifras del suceso: «reintento 2 de 3», «tope de 100 por hora».
    pub numero_a: Option<i64>,
    pub numero_b: Option<i64>,
    pub cuando: String,
}

/// Una clase de suceso con cuántos hay, para el desplegable de «qué pasó».
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaseIpc {
    pub clave: String,
    pub cuantos: u32,
}

/// El registro, tal como lo pinta la pantalla.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistroIpc {
    pub sucesos: Vec<SucesoIpc>,
    /// **Cuántos hay de verdad**, no cuántos se sirven.
    ///
    /// Es la mitad que impide que el tope mienta. Sin él, 500 filas se leen
    /// como «esto es todo», y con 3 412 detrás eso es una conclusión falsa
    /// sobre una campaña.
    pub total: u32,
    /// Cuántos caben en una página. La pantalla lo necesita para poder decir
    /// «500 de 3 412» sin repetir la cifra en dos sitios.
    pub por_pagina: u32,
    pub clases: Vec<ClaseIpc>,
}

/// Lo que la pantalla pide filtrar.
///
/// Todos opcionales: un filtro que obliga a elegir algo antes de enseñar nada
/// convierte la primera visita en un formulario.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiltroIpc {
    pub campana: Option<String>,
    pub solo_desenlaces: Option<bool>,
    pub clave: Option<String>,
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub texto: Option<String>,
}

/// El registro de actividad, filtrado.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la base no responde.
#[tauri::command]
pub fn registro(
    estado: tauri::State<'_, EstadoApp>,
    filtro: Option<FiltroIpc>,
) -> Result<RegistroIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    registro_de(estado.db(), empresa, filtro.unwrap_or_default())
}

/// Lo mismo, con la base ya abierta.
///
/// ── POR QUÉ ESTÁ SEPARADO DEL COMANDO ──
///
/// Para poder probarlo contra una base de verdad. Un `tauri::State` no se puede
/// construir en una prueba, así que un comando que hace el trabajo dentro **no
/// se puede probar**: sólo queda mirarlo, que es como llegaron a producción los
/// huecos del §9. Es el mismo patrón que `revision_de` en el preflight.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde.
pub fn registro_de(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
    f: FiltroIpc,
) -> Result<RegistroIpc, ErrorIpc> {
    // ── LOS VACÍOS NO SON FILTROS ──────────────────────────────────────────
    //
    // La pantalla manda «» cuando el cuadro de búsqueda está vacío, y un
    // `Some("")` filtraría por la cadena vacía: con un `LIKE '%%'` da igual,
    // pero con `kind = ''` **no devuelve nada** y la pantalla se queda en
    // blanco sin que nadie haya filtrado. Se normaliza aquí, una vez, y no en
    // cada sitio que construya un filtro.
    let limpio = |v: &Option<String>| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    let (campana, clave, desde, hasta, texto) = (
        limpio(&f.campana),
        limpio(&f.clave),
        limpio(&f.desde),
        limpio(&f.hasta),
        limpio(&f.texto),
    );

    let pagina = db
        .sucesos(
            empresa,
            &arles_db::actividad::Filtro {
                campana: campana.as_deref(),
                solo_desenlaces: f.solo_desenlaces,
                clave: clave.as_deref(),
                desde: desde.as_deref(),
                hasta: hasta.as_deref(),
                texto: texto.as_deref(),
            },
        )
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    // Las clases **sin filtrar**, a propósito: el desplegable tiene que seguir
    // ofreciendo las demás después de elegir una. Calculadas sobre el filtro
    // puesto, elegir «rebotes» dejaría el desplegable con una sola opción y no
    // habría forma de volver sin recargar.
    let clases = db
        .clases_de_suceso(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
        .into_iter()
        .map(|(clave, cuantos)| ClaseIpc { clave, cuantos })
        .collect();

    Ok(RegistroIpc {
        sucesos: pagina
            .sucesos
            .into_iter()
            .map(|s| SucesoIpc {
                id: s.id,
                clave: s.clave,
                es_desenlace: s.es_desenlace,
                campana: s.campana,
                canal: s.canal,
                direccion: s.direccion,
                numero_a: s.numero_a,
                numero_b: s.numero_b,
                cuando: s.cuando,
            })
            .collect(),
        total: pagina.total,
        por_pagina: arles_db::actividad::TOPE_DE_PAGINA,
        clases,
    })
}

/// Cuántos meses de ruido se guardan antes de recortar (D-7C).
///
/// «Registros bimestrales», dijo Dirección. Dos meses es lo que se guarda de lo
/// que **no** es un envío: fuera de horario, sin cupo, el motor arrancó. Los
/// desenlaces de envío no se recortan jamás, ni a los dos meses ni a los diez
/// años: son la prueba de a quién se le escribió.
pub const MESES_QUE_SE_GUARDAN: i64 = 2;

/// Lo que pasaría al recortar, **antes** de recortar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecorteIpc {
    /// Cuántos sucesos de bucle se irían.
    pub se_irian: u32,
    /// La fecha de corte, para poder decirla: «anteriores al 31 de julio».
    pub antes_de: String,
    /// Cuántos **no** se tocan. Es la mitad tranquilizadora del §95.
    pub se_quedan: u32,
}

/// Qué se recortaría si se recortara ahora.
///
/// ── POR QUÉ ES UN COMANDO APARTE Y NO UN PARÁMETRO DE `recortar` ──
///
/// Porque borrar sin decir cuánto es lo que hace que alguien no pulse nunca el
/// botón. Esto contesta «se van 3 412 líneas de bucle, se quedan los 18 900
/// envíos», y con eso la decisión se toma sola.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la base no responde.
#[tauri::command]
pub fn que_se_recortaria(estado: tauri::State<'_, EstadoApp>) -> Result<RecorteIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    que_se_recortaria_de(estado.db(), empresa)
}

/// Lo mismo, con la base ya abierta. Ver `registro_de`.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde.
pub fn que_se_recortaria_de(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
) -> Result<RecorteIpc, ErrorIpc> {
    let antes_de = corte_bimestral();

    let se_irian = db
        .cuanto_se_recortaria(empresa, &antes_de)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;
    let todos = db
        .sucesos(empresa, &arles_db::actividad::Filtro::default())
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
        .total;

    Ok(RecorteIpc {
        se_quedan: todos.saturating_sub(se_irian),
        se_irian,
        antes_de,
    })
}

/// Recorta el ruido de bucle anterior al corte. Devuelve el estado posterior.
///
/// **Los desenlaces de envío no se tocan**, y no porque este comando se
/// acuerde: el `WHERE is_outcome = 0` vive en `arles-db` y es la única forma de
/// llamar a `DELETE` sobre esa tabla en todo el programa.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la base no responde.
#[tauri::command]
pub fn recortar_registro(estado: tauri::State<'_, EstadoApp>) -> Result<RecorteIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    recortar_registro_de(estado.db(), empresa)
}

/// Lo mismo, con la base ya abierta. Ver `registro_de`.
///
/// # Errores
///
/// [`ErrorIpc`] si la base no responde.
pub fn recortar_registro_de(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
) -> Result<RecorteIpc, ErrorIpc> {
    db.recortar_sucesos(empresa, &corte_bimestral())
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    // Se devuelve el estado **después**, releído: «se irían 0» es lo que
    // confirma que el recorte pasó. Devolver lo que se acaba de borrar
    // obligaría a la pantalla a suponer el resto.
    que_se_recortaria_de(db, empresa)
}

/// La fecha de corte: hace dos meses, en UTC.
///
/// UTC y no la zona de la empresa, a diferencia del cupo: aquí el corte es de
/// dos meses, y unas horas arriba o abajo no cambian qué se borra. En el cupo
/// sí cambiaban —el día de México contra el de UTC—, y por eso allí se usa la
/// zona. Dos criterios distintos porque los dos casos son distintos.
fn corte_bimestral() -> String {
    let t = time::OffsetDateTime::now_utc() - time::Duration::days(30 * MESES_QUE_SE_GUARDAN);
    t.format(&time::format_description::well_known::Rfc3339)
        // Una fecha que no se puede formatear no puede convertirse en «borra
        // desde el principio de los tiempos»: se devuelve el instante cero, que
        // no recorta nada. Ante la duda, no se borra.
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}
