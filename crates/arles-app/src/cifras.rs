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
    /// Cuántos salieron en cada una de las últimas seis horas.
    ///
    /// ── POR QUÉ VIAJAN LAS SEIS HORAS Y NO UN PROMEDIO ──
    ///
    /// Porque **un número sin su método es una promesa** (§94). La pantalla
    /// tiene que poder decir «al ritmo de las últimas 6 h» con los datos
    /// detrás, y con un promedio ya masticado no puede enseñar de dónde sale
    /// ni ver que las dos últimas horas fueron a cero.
    pub ritmo: Vec<RitmoDeHoraIpc>,
    /// El último suceso de bucle de esta campaña: **por qué no sale nada**.
    ///
    /// ── ES EL DATO DE MÁS VALOR DE LA PANTALLA, Y EL QUE FALTABA ──
    ///
    /// Un cero sin explicación es lo que hace que alguien reinicie el programa,
    /// y reiniciar no arregla «fuera de ventana» ni «sin cupo». Las seis razones
    /// del silencio están en `06-calidad/CATALOGO-DE-FALLOS.md` §1, y ARLES las
    /// conocía todas desde la Fase 4 sin decir ninguna.
    ///
    /// Es un suceso de bucle y **nunca un desenlace**: «se aceptó un correo» no
    /// explica por qué ahora no sale ninguno.
    ///
    /// `None` en una campaña que nunca dio una vuelta —recién creada, o
    /// terminada antes de que existiera el registro (7.1)—.
    ///
    /// **Quién decide si está quieta es la pantalla**, no esto: aquí viaja el
    /// último suceso, y el `ritmo` de arriba dice si además está avanzando. Una
    /// campaña puede haber pasado por «sin cupo» hace tres horas y estar
    /// enviando ahora mismo.
    pub razon: Option<RazonIpc>,
}

/// Lo último que le pasó al bucle de una campaña.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RazonIpc {
    /// La clave de i18n del suceso (§139), tal cual la dio `Paso::clave_i18n()`.
    pub clave: String,
    /// Las cifras del suceso: «vuelve a las **9:00**», «quedan **20** de **500**».
    pub numero_a: Option<i64>,
    pub numero_b: Option<i64>,
    pub cuando: String,
}

/// Lo que salió en una hora concreta.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RitmoDeHoraIpc {
    /// La hora, como «2026-09-30T10».
    pub hora: String,
    pub salieron: u32,
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

    // Una sola consulta para todas: la razón de cada campaña de golpe. Ver
    // `por_que_esta_quieta_cada_campana`, que agrupa por campaña con MAX(rowid).
    let mut razones = db
        .por_que_esta_quieta_cada_campana(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    // ── SIN CRUCE CON OTRA LISTA ───────────────────────────────────────────
    //
    // El nombre y el estado vienen en el propio resumen. La primera versión
    // los buscaba en `campanas()` con un `filter_map`, que **descarta en
    // silencio** lo que no encuentra: el día que esa lista filtre algo,
    // Actividad dejaría de enseñar esas campañas mientras el total seguiría
    // contándolas. Dos números que no cuadran, que es justo lo que esta
    // entrega existe para impedir.
    let campanas: Vec<CifrasDeCampanaIpc> = por_campana
        .into_iter()
        .map(|(quien, r)| CifrasDeCampanaIpc {
            // `remove` y no `get`: cada campaña sale una vez, y así lo que
            // quede en el mapa al final sería una razón de una campaña que el
            // resumen no trajo —imposible hoy, porque las dos consultas parten
            // de la misma empresa—.
            razon: razones.remove(&quien.id).map(|s| RazonIpc {
                clave: s.clave,
                numero_a: s.numero_a,
                numero_b: s.numero_b,
                cuando: s.cuando,
            }),
            // El estado crudo se traduce aquí, con el núcleo: la pantalla
            // recibe la clave y no el valor de la base (§139).
            estado: arles_core::campana::EstadoDeCampana::desde_texto(&quien.estado)
                .map_or("campana.estado.desconocido", |e| e.clave_i18n())
                .to_owned(),
            id: quien.id,
            nombre: quien.nombre,
            audiencia: r.audiencia,
            salio: r.salio,
            pendiente: r.pendiente,
            fallo: r.fallo,
            dado_de_baja: r.dado_de_baja,
            cancelado: r.cancelado,
            respondieron: r.respondieron,
            ritmo: Vec::new(),
        })
        .collect();

    // ── EL RITMO, SÓLO DE LO QUE ESTÁ ENVIANDO ─────────────────────────────
    //
    // `ritmo_por_hora` se construyó en la 7.2 y **ningún comando lo llamaba**:
    // el octavo caso del patrón del §9, creado en la misma entrega que lo
    // documenta. Sin esto, la pantalla no puede decir «al ritmo de las últimas
    // 6 h, termina el jueves».
    //
    // Se pide sólo para las campañas en marcha, y no para las treinta
    // terminadas del historial: una consulta más por campaña parada no dice
    // nada que alguien vaya a leer.
    let desde = hace_seis_horas();
    let mut campanas = campanas;
    for c in &mut campanas {
        if c.estado != "campana.estado.enviando" {
            continue;
        }
        c.ritmo = db
            .ritmo_por_hora(empresa, &c.id, &desde)
            .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
            .into_iter()
            .map(|(hora, salieron)| RitmoDeHoraIpc { hora, salieron })
            .collect();
    }

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

/// El instante de hace seis horas, en el formato de las marcas guardadas.
///
/// Seis y no una: con una hora, una campaña que acaba de arrancar o que lleva
/// un rato sin cupo daría ritmo cero, y la estimación diría «no termina
/// nunca». Seis horas suavizan los huecos sin llegar a esconder una parada.
fn hace_seis_horas() -> String {
    let t = time::OffsetDateTime::now_utc() - time::Duration::hours(6);
    t.format(&time::format_description::well_known::Rfc3339)
        // Un `OffsetDateTime` siempre cabe en RFC 3339. Si un día no cupiera,
        // pedir «desde el principio de los tiempos» da más datos de la cuenta,
        // que es preferible a no dar ninguno y enseñar un ritmo de cero.
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}
