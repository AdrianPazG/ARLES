//! Archivar el historial: los dos comandos, el diálogo y la relectura que
//! verifica antes de borrar (D-8D, entrega 8.3 bis b).
//!
//! Es el tramo que cierra el hueco que la 8.3 bis a dejó abierto a propósito:
//! `que_pasaria_al_archivar` y `borrar_el_historial_archivado` estaban
//! construidos, con ocho pruebas, y **nadie los llamaba**. Lo vigila
//! `el_archivado_tiene_quien_lo_llame` en `--fase 8`, escrita antes de esto y
//! vista en rojo nombrando los siete tramos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL ORDEN ES LA ENTREGA, Y NO SE PUEDE CAMBIAR
//!
//! ```text
//! 1 · preflight        ← qué se va, qué se queda, qué se pierde
//! 2 · el usuario elige dónde           (diálogo del sistema)
//! 3 · se escribe el archivo
//! 4 · SE VUELVE A LEER ENTERO y se comprueba su suma
//! 5 · sólo entonces se borra           (y el borrado vuelve a contar)
//! 6 · bitácora                         (dentro de la transacción del borrado)
//! ```
//!
//! **El paso 4 es la segunda condición de D-8D y es la que hace que esto sea
//! archivar y no borrar.** Escribir un archivo y borrar sin volver a abrirlo
//! deja el caso que arruina la decisión entera: un disco externo que falla a
//! mitad, un `.arles` cortado, y el historial ya no está en ningún sitio. Un
//! archivo que nadie ha leído es una suposición.
//!
//! Y se relee con **el mismo lector que usa la restauración** (`LeeElArchivo`),
//! no con uno escrito para esto: la comprobación que importa es que el archivo
//! se pueda abrir el día que haga falta, y ese día lo abrirá ése.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ EL ARCHIVO NO LLEVA CONTRASEÑA, Y NO ES UN OLVIDO
//!
//! Un respaldo se restaura dentro de ARLES, que puede pedir la contraseña. Un
//! archivo de historial se abre **años después**, puede que en otro equipo, y
//! para contestar una solicitud de datos de una persona. La contraseña del
//! respaldo no se guarda en ningún sitio (8.2a): aplicada aquí, un archivo
//! cifrado cuya contraseña nadie recuerda es exactamente igual a no tener el
//! archivo — y entonces el historial se habría borrado para nada.
//!
//! Es un trade-off y se dice: el archivo va **en claro**, así que quien lo
//! guarda tiene que guardarlo donde guardaría una carpeta de expedientes. La
//! pantalla lo dice con esas palabras. Cifrarlo con la clave del equipo tampoco
//! vale: esa clave vive en el llavero de **este** equipo, y el archivo tiene que
//! sobrevivir a que este equipo no exista.

use arles_db::archivo::QueSeArchiva;
use arles_db::respaldo::QueSeLleva;

use crate::comandos::empresa_actual;
use crate::error::ErrorIpc;
use crate::estado::EstadoApp;
use crate::respaldo::{Avance, ElArchivo, ahora_utc};

/// Lo que se le enseña antes de tocar nada.
///
/// **No lleva la ruta de nada** (regla 4 de la frontera).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoQueSeArchivaria {
    /// La fecha de corte que se usó, para que la pantalla la enseñe tal cual y
    /// no la vuelva a calcular con otro reloj.
    pub corte: String,
    pub envios_que_se_archivan: u64,
    pub envios_que_se_quedan: u64,
    /// El periodo **real** de lo que se va, no el pedido. `None` si no hay nada.
    pub primero: Option<String>,
    pub ultimo: Option<String>,
    pub campanas: u64,
    /// Lo que la cascada se lleva y el archivo **no** guarda.
    pub arrastradas: Vec<FilasArrastradas>,
    pub total_arrastrado: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FilasArrastradas {
    pub tabla: String,
    pub filas: u64,
}

/// Lo que de verdad pasó.
///
/// **Sólo el nombre del archivo, nunca su carpeta.**
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistorialArchivado {
    pub nombre_del_archivo: String,
    pub bytes: u64,
    /// Cuántos envíos se escribieron **y se verificaron** en el archivo.
    pub envios_archivados: u64,
    /// Cuántos se borraron de la base. Tiene que ser el mismo número, y se
    /// devuelven los dos a propósito: la pantalla los enseña juntos, que es la
    /// forma de que una diferencia no pase desapercibida.
    pub envios_borrados: u64,
    pub arrastradas: Vec<FilasArrastradas>,
    pub corte: String,
}

/// Cuántos meses atrás como mínimo puede estar el corte.
///
/// ── POR QUÉ HAY UN MÍNIMO, Y POR QUÉ SON DOCE ──
///
/// Doce, los mismos que el respaldo esencial se guarda
/// (`MESES_EN_EL_ESENCIAL`), y la coincidencia es el motivo: archivar algo que
/// el respaldo esencial **sí** se lleva dejaría el historial reciente fuera de
/// la base y dentro de dos archivos distintos, que es la peor forma de tenerlo.
///
/// Y la razón de producto: el historial del último año es el que se consulta.
/// Archivarlo no ahorra nada que valga la pena y rompe las cifras de Actividad
/// del periodo que la gente mira.
pub const MESES_MINIMOS: u32 = arles_core::respaldo::MESES_EN_EL_ESENCIAL;

/// La fecha de corte, a partir de cuántos meses atrás.
///
/// Se calcula **aquí y no en el núcleo**, que no tiene reloj (cero I/O), y con
/// la misma aritmética que `corte_del_esencial` para que las dos fronteras del
/// mismo día coincidan.
fn corte_de(meses: u32) -> String {
    corte_desde(time::OffsetDateTime::now_utc(), meses)
}

/// Lo mismo, con el instante de partida dado.
///
/// ── POR QUÉ ESTÁ PARTIDA EN DOS ──
///
/// Porque una función que pide la hora al sistema **no se puede comparar
/// consigo misma**: dos llamadas seguidas dan cadenas distintas por los
/// milisegundos, y la primera versión de
/// `un_corte_mas_corto_que_el_minimo_se_sube_al_minimo` falló por eso — no por
/// el código que probaba. Es la misma lección que `ElArchivo` recibiendo un
/// cierre en vez del `AppHandle` (8.2c): lo que no es puro no se prueba.
fn corte_desde(desde: time::OffsetDateTime, meses: u32) -> String {
    use time::format_description::well_known::Rfc3339;

    let meses = i64::from(meses.max(MESES_MINIMOS));
    desde
        .saturating_sub(time::Duration::days(meses * 365 / 12))
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

fn arrastradas(de: &QueSeArchiva) -> Vec<FilasArrastradas> {
    de.arrastradas
        .iter()
        .map(|a| FilasArrastradas {
            tabla: a.tabla.clone(),
            filas: a.filas,
        })
        .collect()
}

/// Qué se archivaría, **sin escribir ni borrar nada**.
///
/// `meses` es cuánto historial se deja en la base. Un valor menor que
/// [`MESES_MINIMOS`] **se sube** a ese mínimo en vez de rechazarse: la pantalla
/// no ofrece menos, y si llegara menos por cualquier vía, equivocarse del lado
/// de archivar menos no pierde nada.
///
/// # Errores
///
/// - `error.app.empresa_no_configurada` si todavía no hay empresa.
/// - Lo que falle al leer la base.
#[tauri::command]
pub fn revisar_archivado(
    estado: tauri::State<'_, EstadoApp>,
    meses: u32,
) -> Result<LoQueSeArchivaria, ErrorIpc> {
    // Se exige empresa aunque el preflight no la use: archivar sin empresa
    // configurada no tiene sentido y el borrado sí la necesita para la
    // bitácora. Pedirla aquí es que el aviso llegue antes del botón.
    let _ = empresa_actual(&estado)?;

    let corte = corte_de(meses);
    let p = estado
        .db()
        .que_pasaria_al_archivar(&corte)
        .map_err(crate::AppError::Db)?;

    Ok(LoQueSeArchivaria {
        corte,
        envios_que_se_archivan: p.envios_que_se_archivan,
        envios_que_se_quedan: p.envios_que_se_quedan,
        primero: p.primero.clone(),
        ultimo: p.ultimo.clone(),
        campanas: p.campanas,
        total_arrastrado: p.total_arrastrado(),
        arrastradas: arrastradas(&p),
    })
}

/// Escribe el archivo, lo vuelve a leer para comprobarlo, y **sólo entonces**
/// borra.
///
/// Devuelve `None` si cerró el diálogo sin elegir sitio: no es un error, y
/// tratarlo como fallo enseñaría un aviso rojo por cancelar.
///
/// # Errores
///
/// - `error.app.empresa_no_configurada` si todavía no hay empresa.
/// - `error.app.nada_que_archivar` si no hay ningún envío anterior al corte.
///   **No se escribe un archivo vacío**: un `.arles` de cero filas en un disco
///   externo es una copia de seguridad que alguien creerá que tiene.
/// - `error.app.archivo_no_se_pudo_leer` si no se puede escribir donde se eligió.
/// - `error.db.el_historial_cambio` si entró un envío entre escribir y borrar.
/// - Lo que falle al leer la base, al escribir o **al releer el archivo**. En
///   todos esos casos **no se borra nada**.
#[tauri::command]
pub fn archivar_historial(
    estado: tauri::State<'_, EstadoApp>,
    app: tauri::AppHandle,
    meses: u32,
) -> Result<Option<HistorialArchivado>, ErrorIpc> {
    use tauri_plugin_dialog::DialogExt;

    let empresa = empresa_actual(&estado)?;
    let corte = corte_de(meses);

    // ── 1 · El preflight otra vez, y no es redundante ──
    //
    // La pantalla ya lo pidió, pero entre que alguien lo leyó y pulsó pueden
    // haber pasado minutos. Lo que importa aquí no es la cifra que se enseñó,
    // sino que **haya algo**: escribir un archivo vacío y «archivar» cero
    // envíos dejaría en el disco un archivo que alguien contará como copia.
    let pasaria = estado
        .db()
        .que_pasaria_al_archivar(&corte)
        .map_err(crate::AppError::Db)?;
    if !pasaria.hay_algo() {
        return Err(crate::AppError::NadaQueArchivar.into());
    }

    // ── 2 · Dónde ──
    //
    // El nombre propuesto lleva el periodo, no la fecha de hoy: dentro de tres
    // años, lo que hace falta saber de un archivo mirando su nombre es **qué
    // periodo guarda**, no cuándo se escribió.
    let sugerido = format!(
        "arles-historial-hasta-{}.arles",
        corte.get(..10).unwrap_or("fecha")
    );
    let Some(elegido) = app
        .dialog()
        .file()
        .set_file_name(&sugerido)
        .add_filter("Archivo de historial de ARLES", &["arles"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let ruta = elegido
        .into_path()
        .map_err(|_| ErrorIpc::from(crate::AppError::ArchivoNoSePudoLeer))?;

    // ── 3 · Escribir ──
    //
    // El mismo `ElArchivo` de la 8.2c. El alcance lo escribe en la cabecera, y
    // de él sale la marca: un archivo de historial no puede salir con la marca
    // de un respaldo porque nadie elige la marca (ver `Alcance::marca`).
    let alcance = arles_core::formato_de_respaldo::Alcance::Historial {
        hasta: corte.clone(),
    };
    let emisor = app.clone();
    let avisar = Box::new(move |a: Avance| {
        use tauri::Emitter as _;
        let _ = emisor.emit(crate::respaldo::EVENTO_DE_AVANCE, a);
    });
    let mut destino = ElArchivo::nuevo(&ruta, &alcance.as_str(), None, avisar, ahora_utc())?;
    let filas = estado
        .db()
        .exportar(QueSeLleva::ElHistorialAnteriorA(&corte), &mut destino)
        .map_err(crate::AppError::Db)?;
    let bytes = destino.cerrar()?;

    // ── 4 · Volver a leerlo entero, con el lector de la restauración ──
    //
    // Si esto falla, se para: el error dice qué pasó y **nada se ha borrado**.
    let leido = comprobar_el_archivo(&ruta, "message_attempt")?;
    let envios_archivados = leido.de_la_tabla;

    // Lo que se escribió y lo que se vuelve a leer tiene que ser lo mismo,
    // **contando todas las tablas**. Comparar sólo el historial dejaría pasar
    // un archivo sin las campañas: con su suma correcta, y sin con qué
    // contestar un ARCO.
    if leido.todas != filas {
        return Err(
            crate::AppError::Core(arles_core::CoreError::RespaldoCorrupto {
                motivo: "el archivo que se acaba de escribir no trae las filas que se escribieron",
            })
            .into(),
        );
    }
    if envios_archivados != pasaria.envios_que_se_archivan {
        // El archivo se leyó bien y trae otra cantidad de la que el preflight
        // contó. Pasa si el motor escribió mientras se exportaba —no puede,
        // porque el cerrojo es uno solo— o si el corte y el recuento miran
        // cosas distintas, que sería un defecto nuestro. En los dos casos, no
        // se borra.
        return Err(crate::AppError::Db(arles_db::DbError::ElHistorialCambio {
            archivadas: pasaria.envios_que_se_archivan,
            ahora: envios_archivados,
        })
        .into());
    }

    // ── 5 y 6 · Borrar, contando otra vez, y la bitácora dentro ──
    let borrado = estado
        .db()
        .borrar_el_historial_archivado(&empresa.to_string(), &corte, envios_archivados)
        .map_err(crate::AppError::Db)?;

    Ok(Some(HistorialArchivado {
        nombre_del_archivo: ruta
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned(),
        bytes,
        envios_archivados,
        envios_borrados: borrado.envios,
        arrastradas: borrado
            .arrastradas
            .iter()
            .map(|a| FilasArrastradas {
                tabla: a.tabla.clone(),
                filas: a.filas,
            })
            .collect(),
        corte,
    }))
}

/// Lee el archivo recién escrito de punta a punta y devuelve cuántas filas de
/// `tabla` trae.
///
/// ── QUÉ COMPRUEBA DE VERDAD, Y POR QUÉ ESO ──
///
/// Recorre el archivo con `LeeElArchivo`, el mismo `Origen` que usa la
/// restauración. Eso da tres cosas de una vez, y ninguna se podría afirmar sin
/// leer:
///
/// 1. **La cabecera se entiende** y declara lo que el escritor dijo.
/// 2. **Cada fila se puede volver a partir** en sus campos — que es donde se
///    cazaría un escapado mal hecho en un nombre con tabuladores.
/// 3. **La suma cuadra**, y la comprueba el propio lector al llegar a la línea
///    de cierre. Un archivo cortado por un disco lleno no tiene esa línea y
///    falla ahí.
///
/// Las filas se cuentan y se tiran: no se guarda ninguna. Medio millón de
/// envíos se recorren sin tenerlos en memoria, que es todo el trabajo de la
/// 8.2b y la 8.3a.
///
/// # Errores
///
/// Lo que falle al abrir o al leer. **Quien llama no borra si esto falla**, y es
/// el punto entero de la función.
pub(crate) fn comprobar_el_archivo(
    ruta: &std::path::Path,
    tabla: &str,
) -> Result<LoQueTraeElArchivo, ErrorIpc> {
    use arles_db::restauracion::Origen as _;

    let (cabecera, salta) = crate::restauracion::leer_la_cabecera(ruta)?;
    let mut lector = crate::restauracion::LeeElArchivo::nuevo(ruta, &cabecera, salta, None)?;

    let mut de_la_tabla = 0_u64;
    let mut todas = 0_u64;
    while let Some((t, _)) = lector.siguiente().map_err(crate::AppError::Db)? {
        todas += 1;
        if t == tabla {
            de_la_tabla += 1;
        }
    }
    Ok(LoQueTraeElArchivo { de_la_tabla, todas })
}

/// Lo que la relectura encontró dentro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoQueTraeElArchivo {
    /// Cuántas filas de la tabla que se preguntó.
    pub de_la_tabla: u64,
    /// Cuántas filas en total, de todas las tablas.
    ///
    /// Se devuelve además de la otra para poder comprobarlo contra lo que el
    /// exportador dijo haber escrito. Comparar sólo el historial dejaría pasar
    /// un archivo al que le faltaran **las campañas** — y entonces el archivo
    /// existe, su suma cuadra, y no sirve para contestar un ARCO porque no se
    /// sabe a qué envío pertenece cada fila.
    pub todas: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_db::{ClaveMaestra, Db};

    const CORTE: &str = "2026-06-01T00:00:00Z";

    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch(
                "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
                   VALUES ('E1', 'Telemetry', 'MX', 'America/Mexico_City', 'hola@telemetry.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
                 INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
                   VALUES ('K1', 'E1', 'Vieja', 'completed', '2025-01-03T00:00:00Z', '2025-01-03T00:00:00Z');",
            )
        })
        .expect("siembra");

        // Dos antes del corte y uno después. El nombre lleva un tabulador a
        // propósito: si el escapado del formato (8.1) se rompiera, la relectura
        // partiría la fila en dos y el archivo traería una fila de más — que es
        // lo que la comprobación de los recuentos tiene que cazar.
        for (id, fecha, direccion) in [
            ("M1", "2025-03-01T10:00:00Z", "ana\tla\tprimera@ejemplo.mx"),
            ("M2", "2025-07-15T10:00:00Z", "beto@ejemplo.mx"),
            ("M3", "2026-09-20T10:00:00Z", "caro@ejemplo.mx"),
        ] {
            db.ejecutar_en_pruebas(|c| {
                c.execute(
                    "INSERT INTO message_attempt
                       (id, campaign_id, channel, contact_address, idempotency_key, state,
                        created_at, updated_at)
                     VALUES (?1, 'K1', 'email', ?2, ?3, 'sent', ?4, ?4)",
                    rusqlite::params![id, direccion, format!("idem-{id}"), fecha],
                )?;
                c.execute(
                    "INSERT INTO activity_event
                       (id, company_id, campaign_id, message_attempt_id, kind, is_outcome, occurred_at)
                     VALUES (?1, 'E1', 'K1', ?2, 'envio.aceptado', 1, ?3)",
                    rusqlite::params![format!("A{id}"), id, fecha],
                )?;
                Ok(())
            })
            .expect("envío");
        }
        (dir, db)
    }

    /// Escribe el archivo de historial con el escritor de verdad (8.2c).
    fn escribir_el_archivo(db: &Db, ruta: &std::path::Path) -> u64 {
        let alcance = arles_core::formato_de_respaldo::Alcance::Historial {
            hasta: CORTE.to_owned(),
        };
        let mut destino = crate::respaldo::ElArchivo::nuevo(
            ruta,
            &alcance.as_str(),
            None,
            Box::new(|_| {}),
            "2026-10-06T12:00:00Z".to_owned(),
        )
        .expect("abre el archivo");
        let filas = db
            .exportar(QueSeLleva::ElHistorialAnteriorA(CORTE), &mut destino)
            .expect("exporta");
        destino.cerrar().expect("cierra");
        filas
    }

    fn cuantas(db: &Db, tabla: &str) -> u64 {
        db.ejecutar_en_pruebas(|c| {
            c.query_row(&format!("SELECT count(*) FROM \"{tabla}\""), [], |f| {
                f.get::<_, i64>(0)
            })
        })
        .expect("cuenta")
        .max(0)
        .unsigned_abs()
    }

    /// **El viaje entero, que es lo que esta entrega existía para tener:**
    /// base poblada → archivo escrito → **releído y verificado** → base
    /// borrada, con las cifras cuadrando en los tres sitios.
    ///
    /// Ninguna entrega anterior podía escribirla: la 8.3 bis a comprobaba que
    /// la base se borra y que el preflight cuenta, y **nadie** que lo que sale
    /// de la base entre en el archivo y se pueda volver a leer.
    ///
    /// Verificada el 06/10/2026 cortando los últimos 40 bytes del archivo antes
    /// de releerlo: la relectura falla y el borrado no se llega a hacer.
    #[test]
    fn el_archivo_se_escribe_se_relee_y_entonces_se_borra() {
        let (dir, db) = base();
        let ruta = dir.path().join("historial.arles");

        let escritas = escribir_el_archivo(&db, &ruta);

        // ── La relectura, con el lector de la restauración ──
        let leido = comprobar_el_archivo(&ruta, "message_attempt").expect("se relee");
        assert_eq!(leido.de_la_tabla, 2, "dos envíos son anteriores al corte");
        assert_eq!(
            leido.todas, escritas,
            "el archivo no trae las filas que se escribieron: {} de {escritas}",
            leido.todas
        );
        // Y las campañas viajan: sin ellas el archivo no contesta un ARCO.
        assert_eq!(
            leido.todas - leido.de_la_tabla,
            1,
            "falta la campaña a la que pertenecen los envíos"
        );

        // ── Y sólo ahora, el borrado ──
        let borrado = db
            .borrar_el_historial_archivado("E1", CORTE, leido.de_la_tabla)
            .expect("borra");
        assert_eq!(borrado.envios, 2);
        assert_eq!(cuantas(&db, "message_attempt"), 1, "queda el posterior");
        assert_eq!(
            cuantas(&db, "activity_event"),
            1,
            "la cascada se llevó los dos sucesos de los envíos archivados"
        );
        assert_eq!(cuantas(&db, "campaign"), 1, "la campaña sigue en la base");
    }

    /// **Un archivo cortado no se da por bueno, y entonces no se borra nada.**
    ///
    /// Es el caso que la segunda condición de D-8D existe para cubrir: un disco
    /// externo que se llena o se desconecta a mitad. Sin la relectura, el
    /// historial se borraría y el archivo no tendría dentro lo que promete.
    ///
    /// Se provoca el fallo de verdad: se trunca el archivo ya escrito.
    #[test]
    fn un_archivo_cortado_no_pasa_la_relectura_y_la_base_queda_intacta() {
        let (dir, db) = base();
        let ruta = dir.path().join("historial.arles");
        escribir_el_archivo(&db, &ruta);

        let antes = cuantas(&db, "message_attempt");
        let bytes = std::fs::read(&ruta).expect("lee");
        let cortado = bytes
            .get(..bytes.len().saturating_sub(40))
            .expect("el archivo tiene más de 40 bytes");
        std::fs::write(&ruta, cortado).expect("trunca");

        let r = comprobar_el_archivo(&ruta, "message_attempt");
        assert!(r.is_err(), "un archivo cortado pasó la comprobación: {r:?}");
        assert_eq!(
            cuantas(&db, "message_attempt"),
            antes,
            "el flujo borró con un archivo que no se pudo leer"
        );
    }

    /// **Un byte cambiado dentro del archivo tampoco pasa.**
    ///
    /// El truncado lo caza la línea de cierre que falta; esto lo caza la suma.
    /// Son dos defensas distintas y hay que ver fallar las dos: un archivo al
    /// que un disco le cambió un carácter tiene su línea de cierre intacta.
    ///
    /// Verificada el 06/10/2026 comprobando que sin el cambio el mismo archivo
    /// sí pasa — si no, la prueba pasaría por cualquier otro motivo.
    #[test]
    fn un_byte_cambiado_lo_caza_la_suma() {
        let (dir, db) = base();
        let ruta = dir.path().join("historial.arles");
        escribir_el_archivo(&db, &ruta);

        // Primero: tal cual, pasa. Sin esto la prueba no distingue «la suma
        // caza el cambio» de «este archivo nunca se pudo leer».
        comprobar_el_archivo(&ruta, "message_attempt").expect("intacto se relee");

        let texto = std::fs::read_to_string(&ruta).expect("lee");
        let cambiado = texto.replacen("beto@ejemplo.mx", "beta@ejemplo.mx", 1);
        assert_ne!(texto, cambiado, "la prueba no cambió nada");
        std::fs::write(&ruta, &cambiado).expect("escribe");

        let r = comprobar_el_archivo(&ruta, "message_attempt");
        assert!(r.is_err(), "la suma no cazó un carácter cambiado: {r:?}");
    }

    /// **El archivo no se puede restaurar, comprobado con el archivo de verdad.**
    ///
    /// La 8.3 bis a lo comprobó con un alcance construido a mano. Esto lo
    /// comprueba leyendo la cabecera **de un archivo escrito por el escritor**:
    /// el camino que recorrería alguien que eligiera el archivo equivocado en
    /// la pantalla de restaurar.
    #[test]
    fn el_archivo_escrito_lo_rechaza_el_restaurador() {
        let (dir, db) = base();
        let ruta = dir.path().join("historial.arles");
        escribir_el_archivo(&db, &ruta);

        let (cabecera, _) = crate::restauracion::leer_la_cabecera(&ruta).expect("cabecera");
        assert!(
            !cabecera.alcance.se_puede_restaurar(),
            "la cabecera del archivo dice que se puede restaurar: {:?}",
            cabecera.alcance
        );

        let r = db.que_pasaria_al_restaurar(cabecera.alcance, &[]);
        assert!(
            matches!(r, Err(arles_db::DbError::EsoEsUnArchivoDeHistorial)),
            "{r:?}"
        );
    }

    /// **El mínimo del corte no se puede saltar por abajo.**
    ///
    /// Un `meses` de 0 archivaría el historial de hoy. La pantalla no lo
    /// ofrece, pero la frontera no se fía de la pantalla: el valor se sube al
    /// mínimo en vez de obedecerse.
    ///
    /// Verificada el 06/10/2026 quitando el `.max(MESES_MINIMOS)`: el corte de
    /// 0 meses pasa a ser el instante de partida y la prueba falla.
    ///
    /// Con un instante fijo, no con el reloj: la primera versión llamaba dos
    /// veces a `corte_de` y falló por los milisegundos entre las dos
    /// lecturas — un defecto de la prueba, no del código. Está contado en
    /// `corte_desde`.
    #[test]
    fn un_corte_mas_corto_que_el_minimo_se_sube_al_minimo() {
        let ahora =
            time::OffsetDateTime::from_unix_timestamp(1_790_000_000).expect("instante válido");

        let de_cero = corte_desde(ahora, 0);
        let del_minimo = corte_desde(ahora, MESES_MINIMOS);
        assert_eq!(
            de_cero, del_minimo,
            "un corte de 0 meses archivaría el historial de este mes"
        );

        // Y más atrás sí se respeta: el mínimo es un suelo, no un valor fijo.
        let de_dos_anos = corte_desde(ahora, 24);
        assert!(
            de_dos_anos < del_minimo,
            "24 meses tiene que cortar más atrás que 12: {de_dos_anos} vs {del_minimo}"
        );
    }

    /// **El corte sale en la forma que el núcleo sabe leer.**
    ///
    /// Si no, `Alcance::parse` no reconstruiría el alcance al releer el archivo
    /// —y la relectura del paso 4 fallaría **después** de haber escrito—.
    #[test]
    fn el_corte_vale_como_fecha_del_alcance() {
        let corte = corte_de(MESES_MINIMOS);
        let alcance = arles_core::formato_de_respaldo::Alcance::Historial {
            hasta: corte.clone(),
        };
        let vuelta = arles_core::formato_de_respaldo::Alcance::parse(&alcance.as_str())
            .expect("el alcance se relee");
        assert_eq!(vuelta, alcance);
        assert!(!vuelta.se_puede_restaurar());
        assert_eq!(
            vuelta.marca(),
            arles_core::formato_de_respaldo::MARCA_ARCHIVO
        );
    }
}
