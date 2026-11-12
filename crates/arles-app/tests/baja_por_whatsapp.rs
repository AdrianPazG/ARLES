//! La baja por WhatsApp, de punta a punta · 12/11/2026
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTA PRUEBA TENÍA QUE EXISTIR
//!
//! Petición de Dirección del 11/11/2026: que la baja por WhatsApp también se
//! aplique sola. Y como pasó con R-25 en el correo, **la mitad del camino
//! estaba construida y nadie lo sabía**:
//!
//!   · el núcleo tenía `Fallo::EsaPersonaPidioLaBaja` desde la Fase 4,
//!   · el motor ya suprimía al recibirlo,
//!   · `arles-whatsapp::codigos` ya traducía los errores de Meta,
//!   · y `entrante.rs` ya sabía verificar un aviso firmado.
//!
//! Lo que faltaba era **la señal**: el código `131050` —«el destinatario dejó
//! de recibir mensajes de marketing de tu negocio, no reintentes»— caía en el
//! cajón de lo desconocido, se reintentaba tres veces y se rendía. La persona
//! que había pedido no recibir **seguía en la lista**, y volvía a recibir en
//! la campaña siguiente.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y POR QUÉ NINGUNA DE LAS OTRAS PRUEBAS VALÍA
//!
//! Cada capa tiene las suyas y **ninguna comprueba que lo que sale de una
//! entre en la siguiente**, que es el patrón que este proyecto lleva
//! dieciocho veces persiguiendo:
//!
//!   · `codigos.rs` comprueba que 131050 se traduce bien — con un `ErrorDeMeta`
//!     construido a mano, no con uno que haya viajado por un cable.
//!   · `bucle.rs` comprueba que el motor suprime con el origen de su canal —
//!     con el proveedor **simulado**, que no habla con nadie.
//!   · `cloud.rs` comprueba que el adaptador lee la respuesta de Meta — sin
//!     motor detrás que haga nada con ella.
//!
//! Esto recorre el camino entero: **el servidor contesta 131050**, el
//! adaptador lo extrae, `codigos` lo traduce, el núcleo decide, el motor
//! suprime, y la fila queda en la base con el canal y el origen correctos. Si
//! mañana alguien quita un eslabón, esto falla.
//!
//! Va contra el servidor de mentira, no contra Meta: el bloqueo META sigue
//! abierto y se cierra en la máquina del usuario.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use arles_app_lib::fabrica::{FuenteDeContrasenas, proveedor_del_numero};
use arles_core::ids::{CompanyId, WhatsappAccountId};
use arles_core::supresion::OrigenDeSupresion;
use arles_core::{Canal, MotivoDeSupresion};
use arles_db::{ClaveMaestra, Db};
use arles_motor::{Ajustes, Entorno, Paso, Trabajador};
use arles_whatsapp::servidor::{Guion, ServidorDeMentira};
use time::OffsetDateTime;

const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
const NUMERO: &str = "01900000-0000-7000-8000-0000000000a1";
const CAMPANA: &str = "camp-wa";
/// El motor consulta la salud del remitente en `email_account` **también en
/// WhatsApp**: el disyuntor vive en esa tabla desde la Fase 4 y no se ha
/// movido. Así que la cuenta de los `Ajustes` tiene que existir ahí, igual que
/// en las pruebas de WhatsApp del propio motor. Queda dicho porque es una
/// costura del diseño actual, no una conveniencia de esta prueba.
const CUENTA: &str = "cuenta-para-la-salud";
const ETAPA: &str = "etapa-wa";
const AHORA: &str = "2026-11-12T18:00:00Z";
/// El móvil de quien pidió dejar de recibir promociones.
const MOVIL: &str = "+5215512345678";

struct LlaveroDeMentira;

impl FuenteDeContrasenas for LlaveroDeMentira {
    fn contrasena_de(&self, _id: &str) -> Result<Option<String>, arles_app_lib::AppError> {
        Ok(Some("dieciseis-letras".to_owned()))
    }

    fn token_del_numero(&self, _id: &str) -> Result<Option<String>, arles_app_lib::AppError> {
        Ok(Some("EAAG-token-de-mentira".to_owned()))
    }
}

/// Un reloj fijo, en horario de envío.
struct RelojFijo;

impl Entorno for RelojFijo {
    fn ahora(&self) -> OffsetDateTime {
        OffsetDateTime::parse(AHORA, &time::format_description::well_known::Rfc3339)
            .expect("fecha válida")
    }

    fn azar(&self) -> f64 {
        0.5
    }
}

fn empresa() -> CompanyId {
    EMPRESA.parse().expect("id")
}

fn numero() -> WhatsappAccountId {
    NUMERO.parse().expect("id")
}

/// Una base con una campaña de WhatsApp en marcha y un envío encolado.
fn base() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("temporal");
    let clave = ClaveMaestra::generar().expect("genera");
    let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO company (id, commercial_name, country, timezone,
                                  corporate_email, created_at, updated_at)
             VALUES ('{EMPRESA}', 'TELEMETRY', 'MX', 'America/Mexico_City',
                     'hola@t.mx', '{AHORA}', '{AHORA}');

             INSERT INTO whatsapp_account (id, company_id, display_name, phone_e164,
                                           waba_id, phone_number_id, credential_ref,
                                           modo, status, created_at, updated_at)
             VALUES ('{NUMERO}', '{EMPRESA}', 'Ventas', '+528100000000', 'waba-1',
                     'pn-1', 'llavero://x/whatsapp-number/{NUMERO}', 'seguimiento',
                     'connected', '{AHORA}', '{AHORA}');

             INSERT INTO email_account (id, company_id, display_name, email_address,
                                        provider_kind, credential_ref, status,
                                        daily_limit, hourly_limit, created_at, updated_at)
             VALUES ('{CUENTA}', '{EMPRESA}', 'Ventas', 'ventas@t.mx', 'smtp',
                     'llavero://ventas', 'connected', 50, 10, '{AHORA}', '{AHORA}');

             INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('{CAMPANA}', '{EMPRESA}', 'Promoción', 'running', '{AHORA}', '{AHORA}');

             INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                         whatsapp_account_id, status,
                                         created_at, updated_at)
             VALUES ('{ETAPA}', '{CAMPANA}', 1, 'whatsapp', '{NUMERO}', 'running',
                     '{AHORA}', '{AHORA}');

             INSERT INTO message_attempt
                 (id, campaign_id, channel, contact_address, idempotency_key,
                  state, attempt_count, created_at, updated_at)
             VALUES ('wa-1', '{CAMPANA}', 'whatsapp', '{MOVIL}', 'idem-wa-1',
                     'queued', 0, '{AHORA}', '{AHORA}');"
        ))
    })
    .expect("siembra");

    // La puerta del canal: sin las dos confirmaciones no sale ningún mensaje,
    // y lo que aquí se mide es lo que pasa **después** de que salga.
    db.confirmar_coexistencia(empresa(), numero(), "usuario_local")
        .expect("confirma");

    (dir, db)
}

fn ajustes() -> Ajustes {
    let mx = arles_core::ventana::zona("America/Mexico_City").expect("zona conocida");
    Ajustes {
        empresa: empresa(),
        campana: CAMPANA.to_owned(),
        cuenta: CUENTA.to_owned(),
        etapa: ETAPA.to_owned(),
        canal: Canal::WhatsApp,
        ventana: arles_core::ventana::Ventana::nueva(
            arles_core::ventana::Dias::TODOS,
            0,
            23,
            mx.clone(),
        )
        .expect("ventana"),
        limites: arles_core::ritmo::Limites::nuevos(10, 50),
        zona_de_la_empresa: mx,
        redaccion: None,
        plantilla_de_whatsapp: Some(plantilla()),
    }
}

fn plantilla() -> arles_core::whatsapp::PlantillaDeMeta {
    arles_core::whatsapp::PlantillaDeMeta {
        nombre: "promo_noviembre".to_owned(),
        idioma: "es_MX".to_owned(),
        estado: arles_core::whatsapp::EstadoEnMeta::Aprobada,
        huecos: vec![],
        cuerpo: "Tenemos una promoción para ti".to_owned(),
    }
}

/// Lo que el proveedor manda. Sin huecos: lo que esta prueba mide es lo que
/// pasa con la RESPUESTA, no cómo se arma el mensaje.
fn contenido() -> arles_core::whatsapp::MensajeDePlantilla {
    arles_core::whatsapp::MensajeDePlantilla {
        nombre: "promo_noviembre".to_owned(),
        idioma: "es_MX".to_owned(),
        valores: vec![],
        faltaron: vec![],
    }
}

/// **Meta dice que esa persona dejó de recibir promociones, y ARLES la da de
/// baja sola.**
///
/// Lo que se mira no es que el envío falle —eso ya pasaba— sino que la fila
/// quede en la lista de bajas, **con el canal y el origen que le corresponden**.
///
/// El origen es la mitad del valor de esta prueba: el motor lo escribía fijo en
/// `RespuestaSmtp`, así que hasta hoy una baja de un número de móvil decía que
/// la había dicho un servidor de correo.
#[test]
fn un_131050_de_meta_da_de_baja_al_numero() {
    let servidor = ServidorDeMentira::arrancar(Guion::Falla {
        http: 400,
        codigo: 131_050,
    })
    .expect("arranca");
    let (_d, db) = base();

    let proveedor = proveedor_del_numero(
        &db,
        &LlaveroDeMentira,
        empresa(),
        numero(),
        contenido(),
        &servidor.base(),
    )
    .expect("monta el proveedor");

    let reloj = RelojFijo;
    let sin_freno = || false;
    let paso = Trabajador::nuevo(&db, &proveedor, &reloj, &sin_freno, ajustes())
        .un_paso()
        .expect("paso");

    assert!(
        matches!(paso, Paso::Baja { .. }),
        "el motor no cerró el intento como una baja: {paso:?}"
    );

    // Y el mensaje salió de verdad: un comando que dijera «baja» sin haber
    // intentado nada pasaría la aserción de arriba igual.
    assert_eq!(
        servidor.recibido().len(),
        1,
        "no se intentó mandar nada, así que Meta no pudo contestar nada"
    );

    let bajas = db.listar_supresiones(empresa(), 0, 10).expect("listar");
    assert_eq!(bajas.len(), 1, "el número no quedó en la lista de bajas");
    assert_eq!(bajas[0].direccion, MOVIL, "se dio de baja a otro número");
    assert_eq!(bajas[0].canal, Canal::WhatsApp);
    assert_eq!(
        bajas[0].motivo,
        MotivoDeSupresion::Baja,
        "la pidió la persona: no es un rebote ni un número inválido"
    );
    assert_eq!(
        bajas[0].origen,
        OrigenDeSupresion::PeticionPorWhatsApp,
        "la baja dice que la pidió otro canal, o que la dijo Meta por un \
         webhook que no existe"
    );
}

/// **Y la baja que el propio motor acaba de escribir protege a ese número
/// después.**
///
/// Es la otra mitad de lo que hay que afirmar. Que la fila esté en la tabla no
/// sirve de nada si el camino del envío no la consulta — y «escribirle a
/// alguien que pidió que no le escribieran no tiene arreglo».
///
/// Hay una prueba del motor que ya mide esto en correo
/// (`quien_pidio_la_baja_no_recibe_nada`), pero con una supresión **sembrada a
/// mano**. Lo que aquí se recorre es el ciclo entero: la baja la escribe el
/// motor al leer la respuesta de Meta, y es ésa la que protege.
///
/// ── POR QUÉ SE AFIRMA EL PASO Y NO SÓLO QUE NO SALIÓ NADA ──
///
/// Porque un envío puede no salir por media docena de razones —la ventana, el
/// cupo, el descanso de catorce días entre campañas, el freno— y una prueba
/// que sólo mirara el servidor **pasaría en verde por cualquiera de ellas**,
/// incluso con la supresión borrada. `Paso::Baja` es inequívoco.
#[test]
fn y_despues_ese_numero_ya_no_recibe() {
    let servidor = ServidorDeMentira::arrancar(Guion::Falla {
        http: 400,
        codigo: 131_050,
    })
    .expect("arranca");
    let (_d, db) = base();

    let proveedor = proveedor_del_numero(
        &db,
        &LlaveroDeMentira,
        empresa(),
        numero(),
        contenido(),
        &servidor.base(),
    )
    .expect("monta el proveedor");

    let reloj = RelojFijo;
    let sin_freno = || false;
    Trabajador::nuevo(&db, &proveedor, &reloj, &sin_freno, ajustes())
        .un_paso()
        .expect("paso");

    // Otra campaña al mismo número, como la semana que viene. Tiene que ser
    // otra: `message_attempt` es única por (campaña, canal, dirección), que es
    // la protección contra encolar dos veces a la misma persona.
    db.ejecutar_en_pruebas(|c| {
        c.execute_batch(&format!(
            "INSERT INTO campaign (id, company_id, name, status, created_at, updated_at)
             VALUES ('otra', '{EMPRESA}', 'La de la semana que viene', 'running',
                     '{AHORA}', '{AHORA}');

             INSERT INTO campaign_stage (id, campaign_id, position, channel,
                                         whatsapp_account_id, status,
                                         created_at, updated_at)
             VALUES ('otra-etapa', 'otra', 1, 'whatsapp', '{NUMERO}', 'running',
                     '{AHORA}', '{AHORA}');

             INSERT INTO message_attempt
                 (id, campaign_id, channel, contact_address, idempotency_key,
                  state, attempt_count, created_at, updated_at)
             VALUES ('wa-2', 'otra', 'whatsapp', '{MOVIL}', 'idem-wa-2',
                     'queued', 0, '{AHORA}', '{AHORA}');"
        ))
    })
    .expect("encola el de la campaña nueva");

    let antes = servidor.recibido().len();
    let paso = Trabajador::nuevo(
        &db,
        &proveedor,
        &reloj,
        &sin_freno,
        Ajustes {
            campana: "otra".to_owned(),
            etapa: "otra-etapa".to_owned(),
            ..ajustes()
        },
    )
    .un_paso()
    .expect("segundo paso");

    assert!(
        matches!(paso, Paso::Baja { .. }),
        "el motor no vio la baja que él mismo escribió: {paso:?}"
    );
    assert_eq!(
        servidor.recibido().len(),
        antes,
        "se volvió a escribir a alguien que había pedido no recibir"
    );
}
