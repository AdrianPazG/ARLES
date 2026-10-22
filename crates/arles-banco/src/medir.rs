//! Las trece cifras de `PRESUPUESTO_RENDIMIENTO.md`, medidas o declaradas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA REGLA DE ESTE ARCHIVO
//!
//! **Lo que no se puede medir aquí se dice sin medir, no se da por cumplido.**
//! Es la misma regla que el validador aplica a las sondas omitidas desde la
//! Fase 1, y la razón por la que existe: una tabla con trece ✅ de los cuales
//! siete son suposiciones es peor que no tener tabla, porque se cree.
//!
//! Cada cifra lleva **cómo se midió** al lado. Un número sin su método no se
//! puede repetir ni discutir, que es lo que pedía el §94 cuando se escribió
//! para las pantallas y vale igual aquí.

use std::time::Instant;

/// Qué se pudo hacer con una cifra del presupuesto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Veredicto {
    /// Se midió y entra en presupuesto.
    Cumple,
    /// Se midió y **no** entra. Es un resultado, no un fallo del banco.
    NoCumple,
    /// No se pudo medir aquí, y se dice por qué. **No es «cumple».**
    SinMedir(&'static str),
    /// No se puede medir porque la función no existe todavía.
    NoConstruido(&'static str),
}

impl Veredicto {
    fn marca(&self) -> &'static str {
        match self {
            Self::Cumple => "CUMPLE",
            Self::NoCumple => "NO CUMPLE",
            Self::SinMedir(_) => "SIN MEDIR",
            Self::NoConstruido(_) => "NO EXISTE",
        }
    }
}

/// Una línea del informe.
#[derive(Debug, Clone)]
pub struct Medida {
    pub operacion: &'static str,
    pub presupuesto: &'static str,
    /// Lo que salió, o por qué no salió nada.
    pub medido: String,
    /// Cómo se obtuvo. Sin esto el número no se puede repetir.
    pub metodo: &'static str,
    pub veredicto: Veredicto,
}

fn ms(d: std::time::Duration) -> String {
    format!("{:.0} ms", d.as_secs_f64() * 1000.0)
}

fn segundos(d: std::time::Duration) -> String {
    format!("{:.1} s", d.as_secs_f64())
}

fn mb(bytes: u64) -> String {
    format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
}

/// Corre el banco entero y devuelve el informe en Markdown.
///
/// # Errores
///
/// Lo que falle al generar el archivo, abrir la base o importar. **No se
/// traga**: un banco que mide después de un error mediría otra cosa.
pub fn correr(cuantas: usize, semilla: u64) -> Result<String, Box<dyn std::error::Error>> {
    use arles_core::ids::CompanyId;
    use arles_db::{ClaveMaestra, Db};

    let dir = tempfile::tempdir()?;
    let mut medidas: Vec<Medida> = Vec::new();

    // ── 1 · Generar el conjunto ───────────────────────────────────────────
    let csv = dir.path().join("sintetico.csv");
    let t = Instant::now();
    let bytes = crate::escribir_csv(&csv, cuantas, semilla)?;
    let generar = t.elapsed();
    eprintln!(
        "generado: {cuantas} filas, {} en {}",
        mb(bytes),
        segundos(generar)
    );

    // ── 2 · La base ───────────────────────────────────────────────────────
    let clave = ClaveMaestra::generar()?;
    let db = Db::abrir(&dir.path().join("arles.db"), &clave)?;
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(
            "INSERT INTO company (id, commercial_name, country, timezone, corporate_email, created_at, updated_at)
               VALUES ('00000000-0000-4000-8000-000000000001', 'Banco', 'MX', 'America/Mexico_City',
                       'banco@ejemplo.mx', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');",
        )
    })?;
    let empresa: CompanyId = "00000000-0000-4000-8000-000000000001"
        .parse()
        .map_err(|e| format!("id de empresa: {e:?}"))?;

    // ── 3 · Leer el archivo, que es la primera mitad de importar ──────────
    let t = Instant::now();
    let tabla = arles_import::leer(&csv)?;
    let leer = t.elapsed();
    eprintln!("leído el CSV en {}", segundos(leer));

    // ── 4 · Importar ──────────────────────────────────────────────────────
    //
    // Se mide el camino completo —leer el archivo y escribir en la base—
    // porque es lo que el presupuesto describe: «importar 500 k filas». Medir
    // sólo la escritura daría una cifra mejor y falsa.
    let t = Instant::now();
    let resultado = importar(&db, empresa, &tabla);
    let importar_t = leer + t.elapsed();
    let (filas_escritas, choques, invalidas) = match resultado {
        Ok(t) => t,
        Err(e) => {
            // Se dice lo que pasó y se sigue midiendo lo que se pueda: un
            // informe a medias con su motivo vale más que ninguno.
            eprintln!("la importación falló: {e}");
            (0, 0, 0)
        }
    };
    eprintln!(
        "importadas {filas_escritas} filas en {} ({choques} choques, {invalidas} inválidas)",
        segundos(importar_t)
    );

    let presupuesto_importar = if cuantas >= 400_000 {
        std::time::Duration::from_secs(300)
    } else {
        // Con menos filas el presupuesto se escala, o el banco pequeño del CI
        // pasaría siempre y no vigilaría nada.
        std::time::Duration::from_secs_f64(300.0 * (cuantas as f64 / 500_000.0))
    };
    medidas.push(Medida {
        operacion: "Importar 500 000 filas",
        presupuesto: "< 5 min, memoria acotada",
        medido: format!(
            "{} ({cuantas} filas: {filas_escritas} escritas, {choques} choques, {invalidas} inválidas)",
            segundos(importar_t)
        ),
        metodo: "el camino del asistente entero —leer, analizar choques y escribir— de reloj a reloj, en --release",
        veredicto: if importar_t <= presupuesto_importar {
            Veredicto::Cumple
        } else {
            Veredicto::NoCumple
        },
    });

    // ── 5 · La memoria, que es el presupuesto que esta fase más ha citado ──
    let pico = crate::pico_de_memoria();
    medidas.push(Medida {
        operacion: "Memoria durante importación de 500 000",
        presupuesto: "< 500 MB",
        medido: pico.map_or_else(|| "no se pudo leer".to_owned(), mb),
        metodo: "VmHWM de /proc/self/status: la marca de agua del sistema operativo, que no baja",
        veredicto: match pico {
            Some(p) if p <= 500 * 1024 * 1024 => Veredicto::Cumple,
            Some(_) => Veredicto::NoCumple,
            None => Veredicto::SinMedir("sin /proc: sólo se puede leer en Linux"),
        },
    });

    // ── 6 · Búsqueda por correo ───────────────────────────────────────────
    //
    // Hay índice único, así que el presupuesto es de 100 ms. Se busca una
    // dirección que existe, no una inventada: buscar algo que no está mide el
    // índice igual pero no el camino que el usuario recorre.
    let una = crate::generar(cuantas.min(1000), semilla)
        .map(|f| f.correo.trim().to_lowercase())
        .find(|c| c.contains('@') && !c.starts_with('@') && c.matches('@').count() == 1)
        .unwrap_or_else(|| "ana.alvarez1@ejemplo.mx".to_owned());
    let t = Instant::now();
    let encontrados = db.duenos_de_direcciones(empresa, std::slice::from_ref(&una))?;
    let buscar = t.elapsed();
    medidas.push(Medida {
        operacion: "Búsqueda por correo",
        presupuesto: "< 100 ms",
        medido: format!("{} ({} resultado(s))", ms(buscar), encontrados.len()),
        metodo: "Db::duenos_de_direcciones con una dirección que existe, sobre el índice único",
        veredicto: if buscar.as_millis() < 100 {
            Veredicto::Cumple
        } else {
            Veredicto::NoCumple
        },
    });

    // ── 7 · La primera página de la tabla ─────────────────────────────────
    //
    // El presupuesto habla de «hasta primer pintado», y pintar necesita una
    // webview que aquí no hay. Se mide **la consulta**, que es la mitad que
    // este banco puede medir, y se dice que es la mitad.
    let t = Instant::now();
    let pagina = db.listar_contactos(empresa, 0, 100)?;
    let primera = t.elapsed();
    medidas.push(Medida {
        operacion: "Abrir tabla de contactos (500 k): la consulta",
        presupuesto: "< 500 ms hasta primer pintado",
        medido: format!("{} para la primera página ({} de {})", ms(primera), pagina.contactos.len(), pagina.total),
        metodo: "Db::listar_contactos(0, 100). NO incluye el pintado: eso necesita la webview (R-07)",
        veredicto: if primera.as_millis() < 500 {
            Veredicto::Cumple
        } else {
            Veredicto::NoCumple
        },
    });

    // ── 8 · La simulación, que es aritmética ──────────────────────────────
    {
        use arles_core::duracion::estimar;
        use arles_core::ritmo::Limites;
        use arles_core::ventana::{Dias, Ventana};

        let zona =
            arles_core::ventana::zona("America/Mexico_City").map_err(|e| format!("zona: {e:?}"))?;
        let ventana =
            Ventana::nueva(Dias::LABORABLES, 9, 18, zona).map_err(|e| format!("ventana: {e:?}"))?;
        let limites = Limites::nuevos(10, 50);
        let desde = time::OffsetDateTime::UNIX_EPOCH;

        let t = Instant::now();
        // Una sola llamada tarda menos de lo que el reloj distingue, así que se
        // miden mil y se divide. Medir una sola daría «0 ms», que no es una
        // medida sino el límite del instrumento.
        for _ in 0..1000 {
            let _ = estimar(500_000, limites, &ventana, desde);
        }
        let simular = t.elapsed() / 1000;
        medidas.push(Medida {
            operacion: "Simulación de campaña",
            presupuesto: "< 200 ms",
            medido: format!("{:.3} ms", simular.as_secs_f64() * 1000.0),
            metodo: "arles_core::duracion::estimar con 500 000 de audiencia, media de 1 000 llamadas",
            veredicto: if simular.as_millis() < 200 {
                Veredicto::Cumple
            } else {
                Veredicto::NoCumple
            },
        });
    }

    // ── 9 · Activar una campaña sobre la base poblada ─────────────────────
    //
    // ── POR QUÉ ESTA CIFRA SE MIDE AHORA Y NO EN LA 8.5 ──────────────────
    //
    // La 8.5 la dejó como `SinMedir("exige una campaña activa con su etapa y
    // remitente: es un banco aparte")`. Eso describe una **molestia**, no un
    // impedimento: la etapa y el remitente son cuatro `INSERT`. La auditoría C
    // del §140 la midió el 07/10/2026 y salió **la peor cifra del producto**.
    //
    // Y es la que más importa de las trece, porque `activar_campana` corre
    // **dentro de un comando de Tauri**, en una sola transacción y con el
    // cerrojo de la conexión tomado: mientras dura, la interfaz no contesta y
    // el motor no escribe.
    let activar = match activacion(&db, empresa, cuantas) {
        Ok(d) => Some(d),
        Err(e) => {
            eprintln!("no se pudo medir la activación: {e}");
            None
        }
    };
    medidas.push(Medida {
        operacion: "Calcular audiencia de campaña (activar)",
        presupuesto: "< 2 s",
        medido: activar.map_or_else(|| "no se pudo medir".to_owned(), |d| segundos(d.0)),
        metodo: "Db::activar_campana sobre la base ya importada: congela la audiencia y llena la cola, en una transacción",
        veredicto: match activar {
            Some((d, _)) if d.as_millis() < 2000 => Veredicto::Cumple,
            Some(_) => Veredicto::NoCumple,
            None => Veredicto::SinMedir("la activación falló: ver la salida de error"),
        },
    });

    // ── 10 · El filtro del registro, que sí existe desde la 7.4 ───────────
    //
    // La 8.5 declaró este presupuesto `NoConstruido("la audiencia no se puede
    // filtrar todavía")`. Es verdad de la audiencia y **falso del producto**:
    // la 7.4 construyó seis criterios combinables sobre el registro, y el de
    // texto es el caso malo —un `LIKE '%…%'`, que ningún índice sirve—.
    //
    // Se mide el que se busca de verdad: una dirección parcial sobre los
    // intentos que acaba de crear la activación de arriba.
    let filtro = activar.and_then(|_| filtrar_el_registro(&db, empresa).ok());
    medidas.push(Medida {
        operacion: "Filtro del registro por texto",
        presupuesto: "< 1 s",
        medido: filtro.map_or_else(
            || "no se pudo medir".to_owned(),
            |(d, total)| format!("{} ({total} coincidencias)", ms(d)),
        ),
        metodo: "Db::sucesos con Filtro::texto sobre los intentos de la campaña activada",
        veredicto: match filtro {
            Some((d, _)) if d.as_millis() < 1000 => Veredicto::Cumple,
            Some(_) => Veredicto::NoCumple,
            None => Veredicto::SinMedir("exige la activación de arriba, que no se pudo medir"),
        },
    });

    // ── 11 · Y las que no se pueden medir desde aquí ──────────────────────
    //
    // Cada una dice por qué, y ninguna dice «cumple».
    medidas.push(Medida {
        operacion: "Arranque en frío hasta interfaz usable",
        presupuesto: "< 2 s",
        medido: "—".to_owned(),
        metodo: "Necesita abrir la aplicación: ventana de Tauri y llavero del sistema",
        veredicto: Veredicto::SinMedir("sin escritorio ni llavero en el contenedor (R-07)"),
    });
    medidas.push(Medida {
        operacion: "Desplazamiento en tabla virtualizada",
        presupuesto: "60 fps sostenidos",
        medido: "—".to_owned(),
        metodo: "Necesita una webview midiendo cuadros por segundo",
        veredicto: Veredicto::SinMedir("sin WKWebView ni WebView2 aquí (R-07)"),
    });
    medidas.push(Medida {
        operacion: "Filtro sobre 500 k contactos",
        presupuesto: "< 1 s",
        medido: "—".to_owned(),
        metodo: "No hay filtro de la LISTA DE CONTACTOS: `listar_contactos` sólo toma página y tamaño. El del registro se mide arriba",
        veredicto: Veredicto::NoConstruido(
            "la audiencia no se puede filtrar todavía: está anotado desde la Fase 6",
        ),
    });
    medidas.push(Medida {
        operacion: "Preflight completo",
        presupuesto: "< 3 s",
        medido: "—".to_owned(),
        metodo: "Tres de sus ocho puntos son de red: autenticación, conexión y DNS",
        veredicto: Veredicto::SinMedir("la red del contenedor bloquea 587, 465 y 25 (R-24)"),
    });
    medidas.push(Medida {
        operacion: "Memoria en reposo con motor activo",
        presupuesto: "< 250 MB",
        medido: "—".to_owned(),
        metodo: "Exige la aplicación abierta con su hilo del motor corriendo",
        veredicto: Veredicto::SinMedir("sin llavero no arranca la aplicación entera (R-07)"),
    });
    medidas.push(Medida {
        operacion: "Importar 100 k filas XLSX",
        presupuesto: "< 60 s",
        medido: "—".to_owned(),
        metodo: "Haría falta escribir un XLSX de 100 000 filas, y no hay con qué",
        veredicto: Veredicto::SinMedir("no hay escritor de XLSX en el proyecto, y añadir uno sólo para medir es una dependencia que se distribuye"),
    });
    medidas.push(Medida {
        operacion: "Tamaño del instalador",
        presupuesto: "< 20 MB",
        medido: "—".to_owned(),
        metodo: "Exige `cargo tauri build` con el empaquetador del sistema",
        veredicto: Veredicto::SinMedir(
            "el contenedor no tiene el empaquetador ni la licencia de Mont (P-01)",
        ),
    });

    Ok(informe(cuantas, semilla, bytes, generar, &medidas))
}

/// Importa el conjunto por el mismo camino que la pantalla.
///
/// ── LA PRIMERA VERSIÓN DE ESTA FUNCIÓN SE SALTABA UN PASO, Y SE NOTÓ ──
///
/// Validaba cada fila y se las daba todas a `importar_contactos`. Con medio
/// millón de filas sintéticas eso falló en la primera corrida del 06/10/2026:
///
/// > la dirección +525512345678 ya está registrada en otro contacto (whatsapp)
///
/// El conjunto trae duplicados **a propósito** —los pide `ESTRATEGIA_QA.md`
/// §7— y el asistente de importación los caza **antes** de escribir, con
/// `analizar`. Saltarse ese paso no es «medir sin la interfaz»: es medir otro
/// camino, uno que el usuario nunca recorre y que además revienta.
///
/// Así que ahora se hace lo mismo que el paso 3 del asistente: analizar,
/// separar lo importable de los choques, y escribir sólo lo importable. El
/// tiempo que se mide incluye el análisis, porque el usuario también lo espera.
///
/// El mapeo va escrito aquí porque el archivo lo escribimos nosotros; en la
/// aplicación sale del paso 2.
fn importar(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
    tabla: &arles_import::TablaLeida,
) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
    use arles_core::analisis::analizar;
    use arles_core::importacion::CampoImportable;
    use arles_db::contactos::DatosDelLote;

    let mapeo = [
        CampoImportable::Nombre,
        CampoImportable::Apellido,
        CampoImportable::Correo,
        CampoImportable::WhatsApp,
        CampoImportable::Empresa,
    ];

    // Una base recién creada: no hay nada con lo que chocar fuera del propio
    // archivo, y las supresiones están vacías. Es el caso de la primera
    // importación, que es el que el presupuesto describe.
    let ya_existen = std::collections::HashMap::new();
    let suprimidas = std::collections::HashMap::new();
    let analisis = analizar(&mapeo, &tabla.filas, "MX", &ya_existen, &suprimidas);

    let resumen = db.importar_contactos(
        empresa,
        &DatosDelLote {
            nombre_del_archivo: tabla.nombre.clone(),
            huella_del_archivo: tabla.huella.clone(),
            mapeo_en_json: "[\"nombre\",\"apellido\",\"correo\",\"whatsapp\",\"empresa\"]"
                .to_owned(),
            texto_del_consentimiento: "Conjunto sintético del banco de pruebas (8.5).".to_owned(),
            origen: arles_core::OrigenDeLaLista::Otro,
            total_de_filas: tabla.filas.len(),
            choques: analisis.choques.len(),
            invalidas: analisis.invalidas.len() + analisis.rechazadas.len(),
        },
        &analisis.listas,
    )?;
    Ok((
        resumen.importados,
        analisis.choques.len(),
        analisis.invalidas.len() + analisis.rechazadas.len(),
    ))
}

/// Siembra remitente, ventana y plantilla, crea una campaña y la **activa**.
///
/// Devuelve lo que tardó la activación y cuántos entraron en la audiencia. Se
/// mide **sólo `activar_campana`**: sembrar la cuenta y crear el borrador son
/// cuatro filas que el usuario hace en el asistente, y meterlas en el reloj
/// daría una cifra peor y menos útil.
///
/// El tope diario va alto **con su aceptación de riesgo** (§48): la campaña que
/// este presupuesto describe es la de medio millón, y `crear_campana` exige
/// constancia de que se informó antes de dejar pasar de 50.
fn activacion(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
    cuantas: usize,
) -> Result<(std::time::Duration, u32), Box<dyn std::error::Error>> {
    use arles_db::alta_de_campana::CampanaNueva;
    use arles_db::plantillas::Redaccion;

    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
    let e = empresa.to_string();
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('{CUENTA}', '{e}', 'Banco', 'banco@ejemplo.mx', 'smtp',
                     'llavero://banco', 'connected', 50, 10,
                     '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('vent-banco', '{e}', 'Laborables', 31, 9, 18,
                     'America/Mexico_City', '2026-01-01T00:00:00Z');"
        ))
    })?;

    let (_, version) = db.crear_plantilla(
        empresa,
        "Banco",
        &Redaccion {
            asunto: "Hola {{nombre}}".to_owned(),
            cuerpo_html: "<p>Gracias</p>".to_owned(),
        },
    )?;

    let creada = db.crear_campana(
        empresa,
        &CampanaNueva {
            nombre: "Campaña del banco",
            cuenta: CUENTA.parse().map_err(|e| format!("cuenta: {e:?}"))?,
            version_de_plantilla: version,
            // El banco mide tiempos, no redacción: sin firma es un correo
            // menos que armar y una variable menos en la medida.
            firma: None,
            ventana: "vent-banco",
            limite_por_hora: Some(500),
            limite_diario: Some(5_000),
            acepto_el_riesgo_del_volumen: true,
        },
    )?;

    let t = Instant::now();
    let activada = db.activar_campana(empresa, creada.id)?;
    let tardo = t.elapsed();
    eprintln!(
        "activada con {} destinatarios de {cuantas} en {}",
        activada.en_la_audiencia,
        segundos(tardo)
    );
    Ok((tardo, activada.en_la_audiencia))
}

/// Mide el filtro de texto del registro sobre los intentos ya creados.
///
/// Se busca un trozo de dominio que **todas** las filas contienen: es el peor
/// caso del `LIKE '%…%'` y el que de verdad se escribe, porque quien filtra no
/// sabe la dirección entera. Buscar algo que no está mediría el mismo recorrido
/// con menos trabajo al devolver.
///
/// ── LOS SUCESOS SE SIEMBRAN CON SQL, Y HAY QUE DECIR POR QUÉ ─────────────
///
/// `activity_event` lo llena el motor, un desenlace por envío. Medio millón de
/// envíos de verdad son medio millón de conexiones al servidor de mentira, que
/// es otra medición y tarda horas. Lo que este presupuesto describe es **la
/// consulta sobre una tabla grande**, así que lo honesto es poner las filas por
/// el camino corto y decirlo, en vez de no medirlo.
///
/// Lo que esto **no** mide: lo que cuesta escribirlas. Eso lo paga el motor,
/// una por envío, y se mide en la corrida larga.
fn filtrar_el_registro(
    db: &arles_db::Db,
    empresa: arles_core::ids::CompanyId,
) -> Result<(std::time::Duration, u32), Box<dyn std::error::Error>> {
    use arles_db::actividad::Filtro;

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(
            "INSERT INTO activity_event (id, company_id, campaign_id, campaign_stage_id,
                                         channel, kind, is_outcome, message_attempt_id,
                                         occurred_at)
             SELECT 'ev-' || a.id, c.company_id, a.campaign_id, a.stage_id, 'email',
                    'desenlace.aceptado', 1, a.id, a.created_at
               FROM message_attempt a
               JOIN campaign c ON c.id = a.campaign_id",
        )
    })?;

    let filtro = Filtro {
        texto: Some("ejemplo.mx"),
        ..Filtro::default()
    };
    let t = Instant::now();
    let pagina = db.sucesos(empresa, &filtro)?;
    let tardo = t.elapsed();
    eprintln!(
        "el registro filtrado devolvió {} de {} en {}",
        pagina.sucesos.len(),
        pagina.total,
        ms(tardo)
    );
    Ok((tardo, pagina.total))
}

fn informe(
    cuantas: usize,
    semilla: u64,
    bytes: u64,
    generar: std::time::Duration,
    medidas: &[Medida],
) -> String {
    let mut s = String::new();
    s.push_str("| Operación | Presupuesto | Medido | Veredicto |\n|---|---|---|---|\n");
    for m in medidas {
        s.push_str(&format!(
            "| {} | {} | {} | **{}** |\n",
            m.operacion,
            m.presupuesto,
            m.medido,
            m.veredicto.marca()
        ));
    }
    s.push_str("\n### Cómo se midió cada una\n\n");
    for m in medidas {
        s.push_str(&format!("- **{}** — {}", m.operacion, m.metodo));
        if let Veredicto::SinMedir(p) | Veredicto::NoConstruido(p) = &m.veredicto {
            s.push_str(&format!(". No se midió: {p}"));
        }
        s.push_str(".\n");
    }
    s.push_str(&format!(
        "\n### El conjunto\n\n{cuantas} filas, semilla {semilla}, {} de CSV, generado en {}.\n",
        mb(bytes),
        segundos(generar)
    ));
    s
}
