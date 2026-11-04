//! **El primer día entero, de la empresa vacía al envío contado.**
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! POR QUÉ ESTE ARCHIVO EXISTE
//!
//! Auditoría H, 04/11/2026. El taller tiene cuarenta y tres pruebas de
//! integración y **cada una recorre un tramo**: `empresa.rs` el alta,
//! `importacion.rs` la importación, `alta_por_canal.rs` la creación de la
//! campaña, `preflight.rs` las comprobaciones, `envio_de_prueba.rs` el SMTP,
//! `registro.rs` el historial. Cada eslabón probado, y **la cadena entera no**.
//!
//! Eso es exactamente donde este proyecto ha encontrado sus peores fallos. La
//! lista está en `CLAUDE.md` y no es corta: el adaptador de WhatsApp probado
//! con `arles-app` sin depender de él; `whatsapp_template` con escritor y sin
//! quien llamara al escritor; la firma de la campaña consultada meses antes de
//! que nada la escribiera; el límite diario que se escribía y no se aplicaba.
//! Todos vivían en una junta, y en una junta ninguna prueba de los dos lados
//! mira.
//!
//! Aquí se recorre la cadena: **empresa → contactos → plantilla → remitente y
//! horario → crear la campaña por la frontera → activarla → el motor envía →
//! el envío aparece contado**.
//!
//! ───────────────────────────────────────────────────────────────────────────
//! QUÉ CAZA ESTO QUE NINGUNA DE LAS OTRAS PUEDE
//!
//! Que lo que el asistente **crea** sea exactamente lo que el motor **sabe
//! recorrer**. Son dos módulos que no se conocen: `alta_de_campana` escribe
//! una campaña con su etapa, y `trabajo::etapas_en_marcha` la busca con tres
//! condiciones —campaña en marcha, etapa en marcha, y su ventana—. Si el alta
//! dejara cualquiera de las tres en un valor que la consulta no acepta, el
//! resultado sería **una campaña activada que no envía nada**: sin error, sin
//! fila en rojo, y con las dos capas pasando sus pruebas.
//!
//! Es el fallo más caro que este producto puede tener, porque no se nota hasta
//! que alguien pregunta por qué no ha llegado nada.
//!
//! ───────────────────────────────────────────────────────────────────────────
//! LO QUE SE SUSTITUYE, Y POR QUÉ ES LEGÍTIMO
//!
//! El **proveedor**: se usa el simulado del motor en vez de un servidor SMTP.
//! Lo que se mide aquí es la cadena de datos, no el protocolo — el protocolo
//! tiene su propia prueba contra un servidor que habla TLS de verdad
//! (`envio_de_prueba.rs`), y montarlo aquí haría esta prueba lenta sin añadir
//! una sola garantía nueva.
//!
//! El **remitente y el horario** se siembran con SQL: el alta de una cuenta
//! pide el llavero del sistema, que en la integración continua no existe. Esa
//! costura está probada aparte en `fabrica.rs`, con ocho casos.
//!
//! Todo lo demás pasa por las puertas de verdad.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use arles_app_lib::campanas::{PorDondeSaleIpc, crear_campana_en};
use arles_core::contacto::{BorradorDeCanal, BorradorDeContacto, DatosDeContacto};
use arles_core::{BorradorDeEmpresa, Canal, DatosDeEmpresa};
use arles_db::{ClaveMaestra, Db};
use arles_motor::{Ajustes, Entorno, ProveedorSimulado, Trabajador};
use time::OffsetDateTime;
use time::macros::datetime;

/// Un miércoles a las 10 de la mañana en Ciudad de México: dentro de la
/// ventana de lunes a viernes de 9 a 18 que siembra esta prueba.
const EN_HORARIO: OffsetDateTime = datetime!(2026-09-23 16:00:00 UTC);

/// Los identificadores se estrechan en la frontera —`EmailAccountId` es un
/// UUIDv7— así que una cadena cualquiera la rechaza `crear_campana_en` antes
/// de tocar la base. Se vio al escribir esta prueba, y es la frontera
/// funcionando: el error dice qué dato no vale en vez de salir tres capas más
/// abajo como un fallo de clave ajena.
const CUENTA: &str = "01900000-0000-7000-8000-00000000c411";
const VENTANA: &str = "01900000-0000-7000-8000-0000000076e4";

struct RelojFijo;

impl Entorno for RelojFijo {
    fn ahora(&self) -> OffsetDateTime {
        EN_HORARIO
    }
    fn azar(&self) -> f64 {
        0.5
    }
}

/// **El camino del primer día, entero.**
///
/// Cada paso es el que haría una persona, en su orden, y cada aserción
/// comprueba que el paso siguiente **puede** ocurrir — que es lo que distingue
/// esta prueba de las siete que recorren un tramo.
///
/// ── VERIFICADA CON DOS RUPTURAS, LAS DOS EN ROJO (04/11/2026) ───────────
///
/// Las dos son exactamente el fallo que esta prueba existe para cazar, y
/// ninguna de las otras cuarenta y tres lo vería:
///
/// 1. `activar_campana` deja la **etapa** en `ready` en vez de `running`.
/// 2. La deja en la **campaña**.
///
/// En los dos casos el alta funciona, la audiencia se congela, la cifra de
/// destinatarios es correcta y las pruebas de `alta_de_campana.rs` siguen en
/// verde — y el motor no encuentra nada que enviar. En producción eso es una
/// campaña activada que nunca manda un correo, sin un solo error. Aquí falla
/// en el paso 7, nombrándolo.
#[test]
fn del_alta_de_la_empresa_al_envio_contado() {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    // ── 1 · La empresa ────────────────────────────────────────────────────
    //
    // Una instalación nueva no tiene ninguna, y sin ella no hay a qué colgar
    // nada: es el primer paso del alta (§25).
    let empresa = db
        .guardar_empresa(
            &DatosDeEmpresa::validar(&BorradorDeEmpresa {
                nombre_comercial: "TELEMETRY INSIGHT".into(),
                pais: "mx".into(),
                zona_horaria: "America/Mexico_City".into(),
                correo_corporativo: "hola@telemetry.mx".into(),
                sitio_web: "https://telemetry.mx".into(),
            })
            .expect("el borrador es válido"),
        )
        .expect("guarda la empresa");

    // ── 2 · Dos contactos ─────────────────────────────────────────────────
    //
    // Dos y no uno, por la lección de la 8.6 F: con una fila no se ve si cada
    // quien recibe lo suyo, y una prueba que no puede distinguir eso no está
    // probando el reparto.
    for (nombre, correo) in [("Ana", "ana@empresa.mx"), ("Beto", "beto@otra.mx")] {
        db.crear_contacto(
            empresa,
            &DatosDeContacto::validar(
                &BorradorDeContacto {
                    nombre: nombre.into(),
                    apellido: "Pruebas".into(),
                    empresa: "Cliente SA".into(),
                    canales: vec![BorradorDeCanal {
                        canal: Canal::Correo,
                        valor: correo.to_owned(),
                        principal: true,
                    }],
                },
                "MX",
            )
            .expect("contacto válido"),
        )
        .expect("crea el contacto");
    }

    // ── 3 · La plantilla ──────────────────────────────────────────────────
    let (_plantilla, version) = db
        .crear_plantilla(
            empresa,
            "Bienvenida",
            &arles_db::plantillas::Redaccion {
                asunto: "Hola {{nombre}}".to_owned(),
                cuerpo_html: "<p>Gracias por tu interés.</p><p>{{baja}}</p>".to_owned(),
            },
        )
        .expect("crea la plantilla");

    // ── 4 · El remitente y el horario ─────────────────────────────────────
    //
    // Por SQL: el alta de una cuenta pide el llavero. Ver la cabecera.
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('{CUENTA}', '{empresa}', 'Ventas', 'ventas@telemetry.mx', 'smtp',
                     'llavero://ventas', 'connected', 50, 10,
                     '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z');

             INSERT INTO execution_window (id, company_id, name, days_of_week,
                                           start_hour, end_hour, timezone, created_at)
             VALUES ('{VENTANA}', '{empresa}', 'Laborables', 31, 9, 18,
                     'America/Mexico_City', '2026-09-01T00:00:00Z');"
        ))
    })
    .expect("siembra el remitente y el horario");

    // ── 5 · Crear la campaña, POR LA FRONTERA ─────────────────────────────
    //
    // `crear_campana_en` es lo que el asistente llama. Pasar por aquí y no por
    // `arles-db` es la mitad del valor de esta prueba: la frontera estrecha los
    // identificadores, y un desajuste entre lo que la pantalla manda y lo que
    // la base espera sale aquí.
    let creada = crear_campana_en(
        &db,
        empresa,
        "Campaña del primer día",
        PorDondeSaleIpc::Correo {
            cuenta: CUENTA.to_owned(),
            version_de_plantilla: version.to_string(),
            firma: None,
        },
        None,
        VENTANA,
        Some(10),
        Some(50),
        false,
    )
    .expect("crea la campaña");

    // ── 6 · Activarla ─────────────────────────────────────────────────────
    //
    // Aquí se congela la audiencia y se llena la cola. La cifra se comprueba:
    // una activación que no encola a nadie dejaría pasar el resto de la prueba
    // sin enviar nada, y el envío de abajo fallaría por el motivo equivocado.
    let activacion = db
        .activar_campana(
            empresa,
            creada.id.parse().expect("el id que acaba de devolver"),
        )
        .expect("activa la campaña");
    assert_eq!(
        activacion.en_la_audiencia, 2,
        "la audiencia congelada no son los dos contactos de arriba"
    );

    // ── 7 · EL TRAMO QUE NINGUNA OTRA PRUEBA RECORRE ──────────────────────
    //
    // ¿Encuentra el motor la campaña que el asistente acaba de crear?
    //
    // Son dos módulos que no se conocen: el alta escribe, y `etapas_en_marcha`
    // busca con tres condiciones. Si el alta dejara una en un valor que la
    // consulta no acepta, aquí saldría cero — y en producción sería una
    // campaña activada que no envía nada, sin un solo error.
    let etapas = db
        .etapas_en_marcha(empresa)
        .expect("consulta las etapas en marcha");
    assert_eq!(
        etapas.len(),
        1,
        "el motor NO encuentra la campaña que el asistente acaba de activar: \
         la crea una capa y la busca otra, y entre las dos no mira nadie"
    );
    let etapa = etapas.into_iter().next().expect("hay una");

    // ── 8 · El motor envía ────────────────────────────────────────────────
    let proveedor = ProveedorSimulado::que_siempre_acepta();
    let redaccion = db
        .redaccion_de_la_campana(&etapa.campana)
        .expect("lee la redacción");
    assert!(
        redaccion.is_some(),
        "la campaña creada por el asistente no tiene redacción que mandar: \
         el motor enviaría correos vacíos"
    );

    let sin_freno = || false;
    for _ in 0..2 {
        let trabajador = Trabajador::nuevo(
            &db,
            &proveedor,
            &RelojFijo,
            &sin_freno,
            Ajustes {
                empresa,
                campana: etapa.campana.clone(),
                cuenta: etapa.cuenta.como_texto(),
                etapa: etapa.etapa.clone(),
                canal: etapa.cuenta.canal(),
                ventana: etapa.ventana.clone(),
                limites: etapa.limites,
                zona_de_la_empresa: etapa.zona_de_la_empresa.clone(),
                redaccion: redaccion.clone(),
                plantilla_de_whatsapp: None,
            },
        );
        trabajador.un_paso().expect("un paso del motor");
    }

    // Los dos salieron, y **cada uno con su nombre dentro**: es lo que
    // comprueba que la plantilla del paso 3 llegó hasta el proveedor con los
    // datos del contacto del paso 2, cruzando los cinco módulos de en medio.
    let pedidos = proveedor.recibidos();
    assert_eq!(pedidos.len(), 2, "no salieron los dos correos");
    let asuntos: Vec<&str> = pedidos.iter().map(|p| p.asunto.as_str()).collect();
    assert!(
        asuntos.contains(&"Hola Ana") && asuntos.contains(&"Hola Beto"),
        "los correos no llevan el nombre de cada persona: {asuntos:?}"
    );

    // ── 9 · Y queda contado ───────────────────────────────────────────────
    //
    // El último eslabón: lo que salió tiene que verse en Actividad. Un envío
    // que ocurre y no se cuenta deja a quien mira la pantalla creyendo que la
    // campaña no ha arrancado.
    let enviados: i64 = db
        .ejecutar_en_pruebas(|c| {
            c.query_row(
                "SELECT count(*) FROM message_attempt
                  WHERE campaign_id = ?1 AND state = 'sent'",
                rusqlite::params![etapa.campana],
                |f| f.get(0),
            )
        })
        .expect("cuenta los envíos");
    assert_eq!(
        enviados, 2,
        "los correos salieron y el registro no los tiene: Actividad diría que \
         la campaña no ha enviado nada"
    );
}
