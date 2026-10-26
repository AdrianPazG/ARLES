//! De una cuenta guardada a un adaptador listo para enviar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ RESUELVE ESTE MÓDULO
//!
//! El motor de la Fase 4 sabe enviar y pide un [`Proveedor`]. El adaptador de
//! la 5.2 sabe hablar SMTP y necesita una [`Cuenta`] y una contraseña. Entre
//! los dos falta alguien que sepa **de dónde salen esos datos**, y ése es este
//! módulo: lee la cuenta de la base, saca su contraseña del llavero del sistema
//! y arma el adaptador.
//!
//! Es un paso corto y tiene módulo propio a propósito: es el único sitio de
//! ARLES donde una contraseña de correo sale del llavero para entrar en un
//! objeto vivo. Reunirlo aquí hace que la regla 2 de la Fase 5 —*la contraseña
//! entra y no sale*— se pueda leer entera de un vistazo, en vez de repartida en
//! el medio de un comando de doscientas líneas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA PUERTA SE ABRE SÓLO CON `connected`
//!
//! Una cuenta tiene cuatro estados en la base, y **sólo uno de ellos envía**.
//! La comprobación está escrita al revés de como se escribe normalmente: no
//! enumera los que se rechazan, sino que exige el único que se acepta, y
//! cualquier otro valor —incluido uno que no exista todavía— se rechaza.
//!
//! La diferencia importa. Con una lista de rechazados, el día que una migración
//! añada un quinto estado —`suspended`, `quota_exceeded`, lo que sea— ese
//! estado **enviaría correo** hasta que alguien se acordara de añadirlo a la
//! lista. Y nadie se acuerda: el síntoma sería una campaña saliendo por una
//! cuenta que el sistema ya había marcado como problemática, que es exactamente
//! lo que el estado quería evitar.
//!
//! Hay una prueba que lo sostiene: `un_estado_que_no_existe_no_envia`.

use arles_core::correo::CifradoDeCorreo;
use arles_core::ids::EmailAccountId;
use arles_db::Db;
use arles_email::{Cifrado, Cuenta, ErrorDeCorreo, ProveedorSmtp};

use crate::error::AppError;
use crate::llavero::Llavero;

/// El único estado de cuenta que puede enviar.
const PUEDE_ENVIAR: &str = "connected";

/// De dónde salen las contraseñas de las cuentas.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTO ES UN TRAIT Y NO UNA LLAMADA DIRECTA AL LLAVERO
///
/// No es arquitectura por gusto: sin esto, **este módulo no se puede probar**.
/// El llavero del sistema no existe en el entorno donde corre la integración
/// continua —un Linux sin escritorio no tiene Keychain ni Credential Manager—,
/// así que una llamada directa dejaría estas rutas sin una sola prueba hasta
/// que alguien abriera ARLES en una máquina de verdad.
///
/// Y son precisamente las rutas donde un fallo es caro: enviar con la cuenta
/// equivocada, o enviar con una contraseña vacía y culpar al servidor.
///
/// En producción sólo hay una implementación —[`Llavero`]— y no hay ninguna
/// forma de configurar otra: no es un punto de extensión, es una costura para
/// poder mirar dentro.
pub trait FuenteDeContrasenas {
    /// La contraseña de una cuenta. `None` si no hay ninguna guardada.
    ///
    /// # Errores
    ///
    /// [`AppError::LlaveroNoDisponible`] si el almacén no responde. **Se
    /// distingue de `None` a propósito**: «no hay contraseña» se arregla
    /// escribiéndola otra vez, y «el llavero no contesta» se arregla
    /// desbloqueando el llavero. Fundirlos manda al usuario a hacer lo que no
    /// es (§95).
    fn contrasena_de(&self, id_cuenta: &str) -> Result<Option<String>, AppError>;

    /// El token de Meta de un número de WhatsApp.
    ///
    /// Método aparte y no el mismo con otro identificador, porque **el llavero
    /// usa espacios de nombres distintos**: `email-account/` y
    /// `whatsapp-number/`. Los dos tipos de cuenta usan UUID, así que una sola
    /// función entregaría el secreto equivocado al adaptador equivocado en
    /// cuanto dos identificadores coincidieran.
    ///
    /// # Errores
    ///
    /// Los mismos que [`Self::contrasena_de`], y por las mismas razones.
    fn token_del_numero(&self, id_numero: &str) -> Result<Option<String>, AppError>;
}

/// El llavero del sistema. La única implementación que se usa de verdad.
#[derive(Debug, Clone, Copy, Default)]
pub struct ElLlaveroDelSistema;

impl FuenteDeContrasenas for ElLlaveroDelSistema {
    fn contrasena_de(&self, id_cuenta: &str) -> Result<Option<String>, AppError> {
        Llavero::de_la_cuenta(id_cuenta).leer_contrasena()
    }

    fn token_del_numero(&self, id_numero: &str) -> Result<Option<String>, AppError> {
        Llavero::del_numero(id_numero).leer_contrasena()
    }
}

/// Por qué una cuenta guardada no puede enviar ahora mismo.
///
/// Son seis motivos y **cada uno pide una acción distinta** de la persona que
/// lo lee: no existe → elige otra; hay que configurarla → escribe la
/// contraseña; está detenida → mira qué falló; está desactivada → actívala;
/// falta la contraseña → vuelve a guardarla; los datos no sirven → corrígelos.
/// Un único «la cuenta no sirve» los obligaría a probar los seis.
#[derive(Debug, thiserror::Error)]
pub enum LaCuentaNoSirve {
    /// El identificador no corresponde a ninguna cuenta de correo.
    ///
    /// Pasa si se borró la cuenta mientras había una campaña apuntando a ella.
    #[error("no hay ninguna cuenta de correo con ese identificador")]
    NoExiste,

    /// `needs_reauth`. Es lo que deja la migración V7 en las cuentas viejas, y
    /// lo que deja un guardado cuya prueba de conexión no pasó.
    #[error("la cuenta está pendiente de configurar")]
    HayQueConfigurarla,

    /// `circuit_open`. El disyuntor la paró tras varios fallos seguidos.
    #[error("la cuenta está detenida por fallos seguidos")]
    EstaDetenidaPorFallos,

    /// `disabled`. Alguien la apagó a mano.
    #[error("la cuenta está desactivada")]
    EstaDesactivada,

    /// La base trae un estado que este código no conoce.
    ///
    /// No se registra el valor en el mensaje: sale de la base, y el detalle de
    /// un error acaba en la bitácora. Es un fallo de programación —una
    /// migración que añadió un estado sin actualizar esto—, no algo que el
    /// usuario pueda arreglar, y el texto lo dice así.
    #[error("la cuenta está en un estado que esta versión de ARLES no conoce")]
    EstadoDesconocido,

    /// No hay contraseña en el llavero para esta cuenta.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// ESTO NO CONTINÚA CON LA CONTRASEÑA VACÍA
    ///
    /// Es la tentación obvia —`unwrap_or_default()`— y sería el peor error de
    /// este módulo. El adaptador se construiría, conectaría con el servidor,
    /// mandaría `AUTH` con una contraseña en blanco y el servidor contestaría
    /// «credenciales incorrectas». ARLES se lo creería y diría al usuario que
    /// su contraseña de aplicación no sirve, cuando el problema es que ARLES
    /// nunca la buscó bien.
    ///
    /// Peor todavía: eso gasta un intento de autenticación fallido contra
    /// Gmail, y varios seguidos marcan la cuenta.
    #[error("no hay ninguna contraseña guardada para esta cuenta")]
    SinContrasenaGuardada,

    /// Los datos guardados no producen un adaptador válido.
    ///
    /// Entre otros, el caso en que el cifrado no se puede preparar — que **no**
    /// continúa sin cifrar (§29).
    #[error("los datos de la cuenta no sirven para enviar: {0}")]
    NoSePudoPrepararElEnvio(#[from] ErrorDeCorreo),
}

impl LaCuentaNoSirve {
    /// La clave de i18n del texto que lee el usuario (§139).
    #[must_use]
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::NoExiste => "error.app.cuenta.no_existe",
            Self::HayQueConfigurarla => "error.app.cuenta.hay_que_configurarla",
            Self::EstaDetenidaPorFallos => "error.app.cuenta.detenida_por_fallos",
            Self::EstaDesactivada => "error.app.cuenta.desactivada",
            Self::EstadoDesconocido => "error.app.cuenta.estado_desconocido",
            Self::SinContrasenaGuardada => "error.app.cuenta.sin_contrasena",
            Self::NoSePudoPrepararElEnvio(e) => e.clave_i18n(),
        }
    }
}

/// El proveedor que toca para una etapa, sea del canal que sea (D-WA).
///
/// ── POR QUÉ `Box<dyn Proveedor>` Y NO UN ENUM ──
///
/// Porque el motor ya habla con un rasgo: `Trabajador` recibe
/// `&dyn Proveedor` y no sabe si detrás hay SMTP o Meta. Devolver un enum
/// obligaría a hacer un `match` aquí y otro en cada sitio que lo use, para
/// acabar llamando al mismo método.
///
/// La decisión la toma la **cuenta de la etapa**, que ya lleva el canal
/// dentro: no hay forma de pedir el proveedor equivocado sin cambiar el tipo.
///
/// `contenido` es el de la etapa —nombre e idioma de la plantilla de Meta—.
/// Los valores de cada contacto no van aquí: viajan en el mensaje, que es lo
/// que D-WA vino a arreglar.
///
/// # Errores
///
/// Los de [`proveedor_de_la_cuenta`] y [`proveedor_del_numero`].
pub fn proveedor_de_la_etapa(
    db: &Db,
    secretos: &dyn FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    cuenta: &arles_db::trabajo::CuentaDeLaEtapa,
    contenido: arles_core::whatsapp::MensajeDePlantilla,
) -> Result<Box<dyn arles_motor::proveedor::Proveedor>, AppError> {
    match cuenta {
        arles_db::trabajo::CuentaDeLaEtapa::Correo(id) => {
            Ok(Box::new(proveedor_de_la_cuenta(db, secretos, *id)?))
        }
        arles_db::trabajo::CuentaDeLaEtapa::Whatsapp(id) => Ok(Box::new(proveedor_del_numero(
            db,
            secretos,
            empresa,
            *id,
            contenido,
            arles_whatsapp::cloud::GRAPH,
        )?)),
    }
}

/// Arma el adaptador de correo de una cuenta guardada.
///
/// Los tres pasos, en este orden y no en otro: se lee la cuenta, se comprueba
/// que **puede** enviar, y sólo entonces se toca el llavero. Leer la contraseña
/// de una cuenta desactivada no serviría de nada y, en macOS, puede hacer que
/// el sistema pregunte al usuario si autoriza el acceso — un cuadro de diálogo
/// por una cuenta que no se iba a usar.
///
/// **El contenido ya no entra aquí.** Hasta el 20/10/2026 la fábrica recibía un
/// `Contenido` y se lo guardaba el adaptador: uno para toda la campaña. Como
/// una campaña tiene un texto por persona, el motor le pasaba uno vacío y el
/// cuerpo de la plantilla no llegaba a ningún correo (H-15). Ahora viaja con
/// cada mensaje.
///
/// # Errores
///
/// - [`AppError::Db`] si la base no responde o la cuenta guardada no pasa la
///   validación del núcleo.
/// - [`AppError::LlaveroNoDisponible`] si el llavero no contesta.
/// - [`AppError::CuentaNoSirve`] con el motivo concreto, si la cuenta existe
///   pero no puede enviar.
pub fn proveedor_de_la_cuenta(
    db: &Db,
    secretos: &dyn FuenteDeContrasenas,
    id: EmailAccountId,
) -> Result<ProveedorSmtp, AppError> {
    let cuenta = db
        .cuenta_de_correo(id)
        .map_err(AppError::Db)?
        .ok_or(LaCuentaNoSirve::NoExiste)?;

    if cuenta.estado != PUEDE_ENVIAR {
        return Err(porque_no_envia(&cuenta.estado).into());
    }

    let contrasena = secretos
        .contrasena_de(&cuenta.id.to_string())?
        .ok_or(LaCuentaNoSirve::SinContrasenaGuardada)?;

    let datos = &cuenta.datos;
    let proveedor = ProveedorSmtp::nuevo(
        &Cuenta {
            servidor: datos.servidor().to_owned(),
            puerto: Some(datos.puerto()),
            usuario: datos.usuario().to_owned(),
            remitente: datos.remitente().raw().to_owned(),
            nombre_visible: datos.nombre_visible().map(ToOwned::to_owned),
            cifrado: match datos.cifrado() {
                CifradoDeCorreo::Starttls => Cifrado::starttls(),
                CifradoDeCorreo::Directo => Cifrado::directo(),
            },
        },
        &contrasena,
    )
    .map_err(LaCuentaNoSirve::NoSePudoPrepararElEnvio)?;

    Ok(proveedor)
}

/// Traduce el estado guardado al motivo por el que no envía.
///
/// Sólo se llama cuando ya se sabe que el estado **no** es `connected`.
fn porque_no_envia(estado: &str) -> LaCuentaNoSirve {
    match estado {
        "needs_reauth" => LaCuentaNoSirve::HayQueConfigurarla,
        "circuit_open" => LaCuentaNoSirve::EstaDetenidaPorFallos,
        "disabled" => LaCuentaNoSirve::EstaDesactivada,
        _ => LaCuentaNoSirve::EstadoDesconocido,
    }
}

// ── El proveedor de WhatsApp · entrega W.8 ───────────────────────────────

/// Arma el proveedor de WhatsApp de un número guardado.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTA FUNCIÓN NO EXISTÍA, Y QUÉ COSTÓ
///
/// El adaptador de la W.2 hablaba con Cloud API y tenía su prueba de punta a
/// punta contra el servidor de mentira. Pasaba. Lo que no había era **nadie
/// que lo construyera**: `arles-app` ni siquiera dependía de `arles-whatsapp`.
///
/// Así que el canal entero —seis entregas, setenta pruebas en verde— no podía
/// mandar un solo mensaje, y ninguna prueba podía decirlo: una prueba comprueba
/// que la pieza funciona, no que otro módulo la use. El hueco vivía justo entre
/// dos capas que cada una pasaba lo suyo.
///
/// Lo cazó el validador del canal, en la comprobación «todo lo que se
/// construye lo usa alguien», que existe por esto y por el editor de plantillas
/// de la Fase 6 — el mismo patrón, dos fases seguidas.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA PUERTA DEL CANAL SE PREGUNTA AQUÍ, NO EN LA PANTALLA
///
/// `puede_enviar()` resuelve las tres condiciones —conectado, con Coexistencia,
/// y con el riesgo aceptado si escribe en frío—. Se pregunta en la fábrica
/// porque **es el único punto por el que pasa todo envío**: puesta solo en la
/// pantalla, cualquier camino que no fuera la pantalla la saltaría.
///
/// # Errores
///
/// - [`AppError::Db`] si la lectura falla.
/// - [`LaCuentaNoSirve::NoExiste`] si el número no está.
/// - [`LaCuentaNoSirve::SinContrasenaGuardada`] si el llavero no tiene su token.
/// - [`AppError::CuentaNoSirve`] si el número existe pero todavía no puede
///   enviar — le falta una de las dos confirmaciones, o no está conectado.
pub fn proveedor_del_numero(
    db: &Db,
    secretos: &dyn FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    id: arles_core::ids::WhatsappAccountId,
    contenido: arles_core::whatsapp::MensajeDePlantilla,
    base: &str,
) -> Result<arles_whatsapp::ProveedorDeWhatsApp, AppError> {
    let numero = db
        .numero(empresa, id)
        .map_err(AppError::Db)?
        .ok_or(LaCuentaNoSirve::NoExiste)?;

    if !numero.puede_enviar() {
        return Err(porque_no_envia(&numero.estado).into());
    }

    let token = secretos
        .token_del_numero(&id.to_string())?
        .ok_or(LaCuentaNoSirve::SinContrasenaGuardada)?;

    Ok(arles_whatsapp::ProveedorDeWhatsApp::nuevo(
        base,
        &numero.phone_number_id,
        &token,
        contenido,
    ))
}

#[cfg(test)]
mod tests {
    use arles_core::correo::{BorradorDeCuenta, DatosDeLaCuenta};
    use arles_core::ids::CompanyId;
    use arles_db::ClaveMaestra;
    use arles_db::correo::LimitesDeLaCuenta;
    use rusqlite::params;

    use super::*;

    const EMPRESA: &str = "11111111-1111-4111-8111-111111111111";
    const CUENTA: &str = "01900000-0000-7000-8000-00000000000a";
    const CONTRASENA: &str = "dieciseis-letras";

    /// Un llavero de mentira, que contesta lo que se le diga.
    enum LlaveroDeMentira {
        Con(&'static str),
        Vacio,
        NoResponde,
    }

    impl FuenteDeContrasenas for LlaveroDeMentira {
        fn contrasena_de(&self, _id: &str) -> Result<Option<String>, AppError> {
            match self {
                Self::Con(c) => Ok(Some((*c).to_owned())),
                Self::Vacio => Ok(None),
                Self::NoResponde => Err(AppError::LlaveroNoDisponible("de mentira".to_owned())),
            }
        }

        // El token sigue el mismo estado que la contraseña: lo que se prueba es
        // cómo reacciona quien llama a un llavero vacío o caído, y eso no
        // depende de qué secreto se pidiera.
        fn token_del_numero(&self, id: &str) -> Result<Option<String>, AppError> {
            self.contrasena_de(id)
        }
    }

    fn datos() -> DatosDeLaCuenta {
        DatosDeLaCuenta::validar(&BorradorDeCuenta {
            servidor: "smtp.gmail.com".to_owned(),
            puerto: "587".to_owned(),
            cifrado: "starttls".to_owned(),
            usuario: "ventas@telemetry.mx".to_owned(),
            remitente: "ventas@telemetry.mx".to_owned(),
            nombre_visible: "TELEMETRY".to_owned(),
        })
        .expect("el borrador de prueba es válido")
    }

    /// Una base con una empresa y una cuenta `connected`.
    fn base() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("temporal");
        let clave = ClaveMaestra::generar().expect("genera");
        let db = Db::abrir(&dir.path().join("arles.db"), &clave).expect("abre");

        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "INSERT INTO company (id, commercial_name, country, timezone,
                                      corporate_email, created_at, updated_at)
                 VALUES (?1, 'TELEMETRY', 'MX', 'America/Mexico_City',
                         'hola@telemetry.mx', '2026-09-01T00:00:00Z',
                         '2026-09-01T00:00:00Z')",
                params![EMPRESA],
            )
        })
        .expect("siembra la empresa");

        db.alta_de_cuenta_de_correo(
            EMPRESA.parse::<CompanyId>().expect("id de empresa"),
            CUENTA.parse::<EmailAccountId>().expect("id de cuenta"),
            &datos(),
            LimitesDeLaCuenta::default(),
            "llavero://prueba",
        )
        .expect("da de alta la cuenta");

        (dir, db)
    }

    fn con_estado(db: &Db, estado: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute(
                "UPDATE email_account SET status = ?2 WHERE id = ?1",
                params![CUENTA, estado],
            )
        })
        .expect("cambia el estado");
    }

    /// Escribe un estado que **la base no admite**, saltándose su `CHECK`.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ HACE FALTA ESTE RODEO, Y QUÉ DEMUESTRA QUE HAGA FALTA
    ///
    /// La primera versión de la prueba del estado desconocido escribía
    /// `'quota_exceeded'` con un `UPDATE` normal, y **falló**: la V1 tiene un
    /// `CHECK (status IN ('connected', 'needs_reauth', 'circuit_open',
    /// 'disabled'))`. Eso es una buena noticia y hay que decirla — la base es
    /// la primera barrera, y hoy no deja entrar un estado inventado.
    ///
    /// No convierte la comprobación de la fábrica en algo redundante, por dos
    /// razones concretas. Una: el `CHECK` sólo mira lo que se escribe desde
    /// aquí, y el día que una migración amplíe esa lista —que es exactamente
    /// cuando aparece un quinto estado— dejará de proteger. Dos: el archivo de
    /// base de datos es de disco, y ARLES lo abre tal como está; una base
    /// escrita por una versión más nueva ya trae estados que ésta no conoce.
    ///
    /// `ignore_check_constraints` desactiva los `CHECK` durante el `UPDATE` y
    /// se vuelve a encender al salir: es la única forma de poner la fábrica
    /// **en el escenario de esa segunda barrera** sin inventarse una migración
    /// falsa en mitad de una prueba.
    fn con_estado_que_la_base_no_admite(db: &Db, estado: &str) {
        db.ejecutar_en_pruebas(|c| {
            c.execute_batch("PRAGMA ignore_check_constraints = ON")?;
            let escrito = c.execute(
                "UPDATE email_account SET status = ?2 WHERE id = ?1",
                params![CUENTA, estado],
            );
            c.execute_batch("PRAGMA ignore_check_constraints = OFF")?;
            escrito
        })
        .expect("escribe el estado desconocido");
    }

    fn armar(db: &Db, llavero: &LlaveroDeMentira) -> Result<ProveedorSmtp, AppError> {
        proveedor_de_la_cuenta(
            db,
            llavero,
            CUENTA.parse::<EmailAccountId>().expect("id de cuenta"),
        )
    }

    /// El camino feliz: cuenta lista y contraseña en el llavero.
    #[test]
    fn una_cuenta_lista_con_su_contrasena_produce_un_adaptador() {
        let (_dir, db) = base();
        armar(&db, &LlaveroDeMentira::Con(CONTRASENA)).expect("arma el adaptador");
    }

    /// **La contraseña no se escribe en ningún sitio.**
    ///
    /// El adaptador lleva dentro la contraseña de correo de la empresa. Si su
    /// `Debug` la imprimiera, aparecería en cualquier `{:?}` puesto depurando,
    /// y de ahí a un archivo de bitácora o a la salida de una prueba en CI hay
    /// un paso (ADR-0011).
    ///
    /// ── LO QUE ESTA PRUEBA CAZA, Y LO QUE NO ──
    ///
    /// Verificada rompiéndola el 28/09/2026, y con dos correcciones que vale la
    /// pena dejar escritas, porque las dos contradicen lo que parecía obvio.
    ///
    /// **La rotura evidente no es posible.** Cambiar el `impl Debug` a mano de
    /// `ProveedorSmtp` por un `#[derive(Debug)]` **no compila**: `TlsParameters`
    /// de `lettre` no implementa `Debug`. O sea que el `impl` escrito a mano no
    /// se puede perder por descuido — el compilador lo impide.
    ///
    /// **Y las credenciales de `lettre` ya se protegen solas.** Añadir
    /// `.field("credenciales", &self.credenciales)` al `Debug` compila, y esta
    /// prueba **sigue pasando**: el `Debug` de `Credentials` imprime la palabra
    /// `Credentials` y nada más. Medido, no supuesto.
    ///
    /// Lo que sí caza, y es la fuga realista: que alguien añada a la estructura
    /// un campo propio con la contraseña —«por si hay que reconectar»— y lo
    /// imprima. Con esa rotura la prueba falla, y el mensaje enseña la
    /// contraseña en claro dentro de la estructura escrita.
    #[test]
    fn la_contrasena_no_aparece_al_escribir_el_adaptador() {
        let (_dir, db) = base();
        let proveedor = armar(&db, &LlaveroDeMentira::Con(CONTRASENA)).expect("arma");

        let escrito = format!("{proveedor:?}");
        assert!(
            !escrito.contains(CONTRASENA),
            "la contraseña salió al escribir el adaptador: {escrito}"
        );
    }

    /// Una cuenta que ya no está no envía, y lo dice con su propio motivo.
    #[test]
    fn una_cuenta_que_no_existe_no_envia() {
        let (_dir, db) = base();
        db.ejecutar_en_pruebas(|c| {
            c.execute("DELETE FROM email_account WHERE id = ?1", params![CUENTA])
        })
        .expect("borra la cuenta");

        let error = armar(&db, &LlaveroDeMentira::Con(CONTRASENA)).expect_err("no debería armar");
        assert_eq!(error.clave_i18n(), "error.app.cuenta.no_existe");
    }

    /// **Cada estado que no envía dice cuál es su motivo.**
    ///
    /// Van los tres juntos porque lo que se comprueba es que **no se funden**:
    /// la acción del usuario es distinta en cada uno, y un mensaje común los
    /// obligaría a probar los tres.
    #[test]
    fn cada_estado_que_no_envia_da_su_propio_motivo() {
        for (estado, clave) in [
            ("needs_reauth", "error.app.cuenta.hay_que_configurarla"),
            ("circuit_open", "error.app.cuenta.detenida_por_fallos"),
            ("disabled", "error.app.cuenta.desactivada"),
        ] {
            let (_dir, db) = base();
            con_estado(&db, estado);

            let error = armar(&db, &LlaveroDeMentira::Con(CONTRASENA))
                .err()
                .unwrap_or_else(|| panic!("«{estado}» no debería poder enviar"));
            assert_eq!(error.clave_i18n(), clave, "el motivo de «{estado}»");
        }
    }

    /// **Un estado que todavía no existe tampoco envía.**
    ///
    /// Es la garantía de la cabecera del módulo: la puerta se abre sólo con
    /// `connected`, no se cierra ante una lista de estados conocidos. El día
    /// que una migración añada un quinto estado, esa cuenta **no** manda correo
    /// hasta que alguien decida que puede.
    ///
    /// Verificada rompiéndola el 28/09/2026: escribiendo la comprobación al
    /// revés —rechazar los tres estados conocidos en vez de exigir
    /// `connected`—, esta prueba falla y las otras tres siguen pasando. Que
    /// sólo falle ésta es justamente lo que la hace valer.
    ///
    /// Sobre por qué hace falta saltarse el `CHECK` de la base para montar el
    /// escenario, y por qué ese `CHECK` no vuelve redundante esta prueba, está
    /// la nota de [`con_estado_que_la_base_no_admite`].
    #[test]
    fn un_estado_que_no_existe_no_envia() {
        let (_dir, db) = base();
        con_estado_que_la_base_no_admite(&db, "quota_exceeded");

        let error = armar(&db, &LlaveroDeMentira::Con(CONTRASENA))
            .expect_err("un estado desconocido no puede enviar");
        assert_eq!(error.clave_i18n(), "error.app.cuenta.estado_desconocido");
    }

    /// **Sin contraseña en el llavero no se envía con la contraseña vacía.**
    ///
    /// La alternativa —`unwrap_or_default()`— construiría el adaptador, gastaría
    /// un intento de autenticación fallido contra el proveedor y le diría al
    /// usuario que su contraseña de aplicación no sirve. Varios de ésos seguidos
    /// marcan la cuenta.
    #[test]
    fn sin_contrasena_guardada_no_se_arma_nada() {
        let (_dir, db) = base();
        let error = armar(&db, &LlaveroDeMentira::Vacio).expect_err("no hay contraseña");
        assert_eq!(error.clave_i18n(), "error.app.cuenta.sin_contrasena");
    }

    /// **«No hay contraseña» y «el llavero no contesta» no son lo mismo.**
    ///
    /// Se arreglan de formas distintas: una escribiendo la contraseña otra vez,
    /// la otra desbloqueando el llavero del sistema. Fundirlas manda al usuario
    /// a hacer lo que no es (§95).
    #[test]
    fn un_llavero_que_no_responde_no_se_confunde_con_uno_vacio() {
        let (_dir, db) = base();
        let error = armar(&db, &LlaveroDeMentira::NoResponde).expect_err("el llavero falla");
        assert_eq!(error.clave_i18n(), "error.app.llavero_no_disponible");
        assert_ne!(error.clave_i18n(), "error.app.cuenta.sin_contrasena");
    }

    /// **El llavero no se toca si la cuenta no puede enviar.**
    ///
    /// En macOS, leer una entrada del llavero puede hacer que el sistema
    /// pregunte al usuario si autoriza el acceso. Un cuadro de diálogo por una
    /// cuenta desactivada, que no se iba a usar, es una avería de cara al
    /// usuario aunque el código «funcione».
    ///
    /// Se comprueba con un llavero que falla siempre: si el orden fuera el
    /// contrario, el error sería el del llavero y no el de la cuenta.
    #[test]
    fn una_cuenta_que_no_envia_ni_siquiera_abre_el_llavero() {
        let (_dir, db) = base();
        con_estado(&db, "disabled");

        let error = armar(&db, &LlaveroDeMentira::NoResponde).expect_err("no debería armar");
        assert_eq!(
            error.clave_i18n(),
            "error.app.cuenta.desactivada",
            "se tocó el llavero antes de mirar si la cuenta podía enviar"
        );
    }
}
