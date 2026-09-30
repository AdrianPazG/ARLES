//! Las cifras de Actividad, en la frontera.
//!
//! Entrega 7.2. Un solo comando, grueso (regla 1 de la frontera): la pantalla
//! pide «cómo va todo» y recibe las campañas, el total y el cupo de cada
//! remitente de una vez.
//!
//! Tres llamadas separadas —campañas, total, cupos— serían tres cruces con una
//! ventana entre medias en la que la pantalla enseña un total que ya no
//! corresponde a las campañas de al lado. Con tres campañas enviando a la vez,
//! esa ventana se nota.

use serde::Serialize;

use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Cómo va una campaña, con su nombre.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CifrasDeCampanaIpc {
    pub id: String,
    pub nombre: String,
    /// La clave del rótulo de estado (§139), no el estado crudo.
    pub estado: String,
    pub audiencia: u32,
    pub salio: u32,
    pub pendiente: u32,
    pub fallo: u32,
    pub dado_de_baja: u32,
    pub cancelado: u32,
    /// Cuánta gente contestó (D-7A). **Nunca lo que dijeron.**
    pub respondieron: u32,
}

/// El cupo de un remitente, y quién compite por él.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CupoIpc {
    pub nombre: String,
    pub direccion: String,
    pub gastado_hoy: u32,
    pub tope_diario: Option<u32>,
    pub quedan_hoy: Option<u32>,
    pub gastado_esta_hora: u32,
    pub tope_por_hora: Option<u32>,
    /// Las campañas en marcha que tiran de esta cuenta, por nombre.
    ///
    /// Es lo que explica por qué dos campañas están quietas cuando una tercera
    /// se comió el cupo: sin los nombres, «quedan 20 de 500» no dice de quién.
    pub campanas: Vec<String>,
    /// `true` si más de una campaña tira de ella.
    pub hay_reparto: bool,
}

/// Todo lo que la pantalla de Actividad necesita para pintarse.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActividadIpc {
    pub campanas: Vec<CifrasDeCampanaIpc>,
    /// El total, que es **la suma de las campañas de arriba**, no otra cuenta.
    pub total_salio: u32,
    pub total_pendiente: u32,
    pub total_fallo: u32,
    pub remitentes: Vec<CupoIpc>,
}

/// Cómo va todo ahora mismo.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la base no responde.
#[tauri::command]
pub fn actividad(estado: tauri::State<'_, EstadoApp>) -> Result<ActividadIpc, ErrorIpc> {
    let empresa = crate::comandos::empresa_actual(&estado)?;
    let db = estado.db();

    let (por_campana, total) = db
        .resumen_de_la_empresa(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    // Los nombres y estados vienen de la misma lista que pinta Campañas, para
    // que las dos pantallas no puedan llamar distinto a la misma campaña.
    let guardadas = db
        .campanas(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    let campanas = por_campana
        .into_iter()
        .filter_map(|(id, r)| {
            let g = guardadas.iter().find(|c| c.id.to_string() == id)?;
            Some(CifrasDeCampanaIpc {
                id,
                nombre: g.nombre.clone(),
                estado: g.estado.clave_i18n().to_owned(),
                audiencia: r.audiencia,
                salio: r.salio,
                pendiente: r.pendiente,
                fallo: r.fallo,
                dado_de_baja: r.dado_de_baja,
                cancelado: r.cancelado,
                respondieron: r.respondieron,
            })
        })
        .collect();

    // La zona de la empresa, que es donde se cuentan las horas del presupuesto.
    // Sin empresa no se habría llegado hasta aquí: `empresa_actual` ya falló.
    let zona = db
        .empresa()
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
        .map(|e| e.datos.zona_horaria().to_owned())
        .unwrap_or_default();
    // Una zona que no se puede interpretar no puede pasar: la validación del
    // núcleo la rechaza al guardar la empresa, así que llegar aquí con una
    // rota significaría que alguien tocó la base a mano. En ese caso se enseña
    // el cupo en UTC —mejor que no enseñar nada— y la pantalla sigue viva.
    let Ok(zona) = arles_core::ventana::zona(&zona) else {
        // Sin zona interpretable no se puede decir de qué día es el cupo, y
        // enseñar el de otro día sería peor que no enseñarlo: la lista de
        // remitentes se queda vacía y las cifras de campaña, que no dependen
        // de la hora, siguen saliendo.
        return Ok(ActividadIpc {
            campanas,
            total_salio: total.salio,
            total_pendiente: total.pendiente,
            total_fallo: total.fallo,
            remitentes: Vec::new(),
        });
    };
    let (hora, dia) = claves_de_ahora(&zona);
    let remitentes = db
        .cupo_de_los_remitentes(empresa, &hora, &dia)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
        .into_iter()
        .map(|c| CupoIpc {
            quedan_hoy: c.quedan_hoy(),
            hay_reparto: c.hay_reparto(),
            nombre: c.nombre,
            direccion: c.direccion,
            gastado_hoy: c.gastado_hoy,
            tope_diario: c.tope_diario,
            gastado_esta_hora: c.gastado_esta_hora,
            tope_por_hora: c.tope_por_hora,
            campanas: c.campanas,
        })
        .collect();

    Ok(ActividadIpc {
        campanas,
        total_salio: total.salio,
        total_pendiente: total.pendiente,
        total_fallo: total.fallo,
        remitentes,
    })
}

/// Las claves de hora y día de ahora, **en la zona de la empresa**.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ NO VALE UTC, Y ESTUVO MAL ESCRITO UN RATO
///
/// El presupuesto se cuenta en la zona de la empresa: es lo que hace el motor
/// con `clave_de_hora` y `clave_de_dia` del núcleo. Leerlo con claves en UTC
/// da el cupo correcto **casi siempre**, y el «casi» es lo que hace daño:
/// entre las 18:00 y las 00:00 de México, UTC ya va en el día siguiente, así
/// que la pantalla enseñaría el cupo de mañana —a cero— mientras el motor
/// sigue gastando el de hoy. Seis horas al día diciendo «quedan 500» con el
/// cupo agotado.
///
/// Se usan las mismas funciones del núcleo que usa el motor, y no una copia:
/// dos formas de calcular la misma clave es como se acaba leyendo un cubo que
/// nadie está llenando.
fn claves_de_ahora(zona: &arles_core::ventana::TimeZone) -> (String, String) {
    let ahora = time::OffsetDateTime::now_utc();
    (
        arles_core::ventana::clave_de_hora(zona, ahora),
        arles_core::ventana::clave_de_dia(zona, ahora),
    )
}
