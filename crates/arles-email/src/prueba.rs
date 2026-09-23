//! El resultado de «Probar conexión» (§29).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO NO ES UN `Fallo`
//!
//! Es el hueco que quedó anotado al cerrar la 5.2, y conviene entender por qué
//! importa.
//!
//! El motor entiende cinco `Fallo`, y todos significan lo mismo en el fondo:
//! **este envío concreto no salió**. Con ese vocabulario, una cuenta mal
//! configurada —un puerto equivocado, una contraseña que caducó— sólo se puede
//! decir de una forma: «pasajero». El motor entonces reintenta a los 5, 15 y 30
//! minutos, falla las tres veces, y a los cinco fallos seguidos el disyuntor
//! para la cuenta y dice «cinco fallos seguidos».
//!
//! Eso es cierto y es inútil. Quien lo lee no sabe si el problema es su
//! contraseña, su red o el servidor del otro lado; y ha tardado media hora en
//! enterarse de que hay un problema.
//!
//! Este diagnóstico existe para decirlo **antes de que haya una campaña**, y
//! para decir algo que se pueda accionar. Cada variante corresponde a una cosa
//! distinta que hacer, y ninguna es «vuelve a intentarlo».
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE LA PRUEBA NO HACE
//!
//! **No manda ningún correo, ni un `MAIL FROM`, ni un `RCPT TO`.**
//!
//! La tentación es real: un `MAIL FROM` seguido de `RSET` verificaría además
//! que el servidor acepta esa dirección de remitente, que es información útil.
//! No se hace, por dos razones. La primera es que algunos servidores cuentan
//! ese diálogo como un intento de envío a efectos de reputación, y el usuario
//! puede pulsar «Probar conexión» veinte veces mientras configura. La segunda
//! es la que cierra la discusión: un `RCPT TO` de prueba apuntaría a **una
//! persona real**, y la respuesta del servidor a ese comando es, en todo ARLES,
//! el único camino que da de baja a alguien.
//!
//! Hay una prueba que mira el servidor de mentira desde fuera y exige que esos
//! tres comandos no aparezcan.

use lettre::transport::smtp::Error as ErrorSmtp;

/// Lo que se averiguó al probar la cuenta.
///
/// Cada variante lleva a una acción distinta. Si dos llevaran a la misma, una
/// de las dos sobra: dar a elegir entre dos textos que dicen lo mismo es peor
/// que dar uno.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diagnostico {
    /// Conectó, cifró y la contraseña sirve. La cuenta se puede usar.
    Correcto,
    /// No se llegó a hablar con el servidor.
    ///
    /// El nombre no resuelve, el puerto está cerrado, o la red de la empresa
    /// bloquea la salida — que es más común de lo que parece: muchas redes
    /// corporativas cierran el 587 y el 465 precisamente para impedir que se
    /// mande correo desde dentro.
    NoSeLlegoAlServidor,
    /// El servidor aceptó la conexión y luego se quedó callado.
    ///
    /// Distinto del anterior a propósito: aquí **sí** hay algo escuchando en
    /// ese puerto, así que el nombre y el puerto están bien y el problema es
    /// otro. Decir «revisa el puerto» aquí manda a buscar donde no es.
    ElServidorTardoDemasiado,
    /// No se pudo establecer el cifrado.
    ///
    /// El servidor no lo ofrece, o su certificado no es válido para el nombre
    /// con el que se le llamó.
    ///
    /// **Aquí es donde alguien pediría una casilla para seguir sin cifrar**, y
    /// es exactamente donde no la va a haber (§29). El texto lo dice sin
    /// rodeos, porque dejarlo ambiguo es invitar a que lo pidan otra vez.
    ElCifradoNoSePudoEstablecer,
    /// El servidor rechazó la contraseña, y lo dijo de forma definitiva.
    ///
    /// En Gmail esto casi siempre significa lo mismo: se escribió la contraseña
    /// de la cuenta de Google en vez de una **contraseña de aplicación**, que
    /// es otra cosa y exige tener la verificación en dos pasos activada. Es el
    /// punto donde más gente se atasca, así que el texto lleva el camino
    /// exacto.
    LaContrasenaNoSirve,
    /// El servidor contestó que ahora no puede.
    ///
    /// Un rechazo temporal —4xx— mientras se autenticaba. No dice que la
    /// contraseña esté mal: decirlo mandaría a alguien a generar una
    /// contraseña nueva por un problema que se arregla solo.
    ElServidorNoEstaDisponible,
}

impl Diagnostico {
    /// La clave del texto que se enseña (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::Correcto => "correo.prueba.correcto",
            Self::NoSeLlegoAlServidor => "correo.prueba.no_se_llego",
            Self::ElServidorTardoDemasiado => "correo.prueba.tardo_demasiado",
            Self::ElCifradoNoSePudoEstablecer => "correo.prueba.sin_cifrado",
            Self::LaContrasenaNoSirve => "correo.prueba.contrasena_rechazada",
            Self::ElServidorNoEstaDisponible => "correo.prueba.no_disponible",
        }
    }

    /// ¿La cuenta se puede guardar y usar?
    #[must_use]
    pub const fn sirve(&self) -> bool {
        matches!(self, Self::Correcto)
    }
}

/// En qué paso de la conversación se rompió.
///
/// Existe para que **el envío y la prueba recorran el mismo camino**. Si la
/// prueba tuviera su propio código de conexión, podría dar «correcto» mientras
/// los envíos fallan —o al revés—, y una prueba de conexión en la que no se
/// puede confiar es peor que no tenerla: manda a buscar el problema al otro
/// lado.
///
/// El paso importa porque el mismo error de red significa cosas distintas según
/// dónde ocurra: al conectar es «no llegué», al cifrar es «llegué y no pudimos
/// entendernos en privado».
pub(crate) enum DondeSeRompio {
    AlConectar(ErrorSmtp),
    AlCifrar(ErrorSmtp),
    /// El socket dijo que el canal no estaba cifrado. La comprobación que ARLES
    /// hace por su cuenta, y que nunca debería saltar.
    ElCanalNoEstabaCifrado,
    AlAutenticar(ErrorSmtp),
}

impl DondeSeRompio {
    /// Traduce a un diagnóstico accionable.
    pub(crate) fn como_diagnostico(&self) -> Diagnostico {
        match self {
            // Un tiempo agotado, en cualquier paso, dice lo mismo: hay algo al
            // otro lado y no contesta.
            Self::AlConectar(e) | Self::AlCifrar(e) | Self::AlAutenticar(e) if e.is_timeout() => {
                Diagnostico::ElServidorTardoDemasiado
            }

            // Con TLS directo el cifrado ocurre DENTRO de la conexión, así que
            // un certificado que no cuadra llega por este camino. Sin esta
            // rama diría «no se llegó al servidor», que es falso y manda a
            // revisar el puerto cuando el problema es el certificado.
            Self::AlConectar(e) if fallo_de_certificado(e) => {
                Diagnostico::ElCifradoNoSePudoEstablecer
            }
            Self::AlConectar(_) => Diagnostico::NoSeLlegoAlServidor,

            Self::AlCifrar(_) | Self::ElCanalNoEstabaCifrado => {
                Diagnostico::ElCifradoNoSePudoEstablecer
            }

            // Sólo un rechazo DEFINITIVO culpa a la contraseña. Un 4xx es «ahora
            // no puedo», y decirle a alguien que su contraseña está mal por un
            // 4xx le hace generar una nueva —y perder la que funcionaba— para
            // arreglar algo que no estaba roto.
            Self::AlAutenticar(e) if e.is_permanent() => Diagnostico::LaContrasenaNoSirve,
            Self::AlAutenticar(e) if e.is_transient() => Diagnostico::ElServidorNoEstaDisponible,
            // Sin código: el servidor cortó, o contestó algo ilegible, mientras
            // se autenticaba. No se puede afirmar que la contraseña esté mal.
            Self::AlAutenticar(_) => Diagnostico::ElServidorNoEstaDisponible,
        }
    }
}

/// ¿Este fallo de conexión es en realidad un certificado que no cuadra?
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ NO SE USA `is_tls()`, QUE ES LO QUE PARECE
///
/// `lettre` tiene un `Error::is_tls()`, y la primera versión de esto lo usaba.
/// **No funciona**, y se supo porque la prueba del certificado ajeno falló:
/// ante un certificado que no valida, `lettre` devuelve `Kind::Network` con el
/// error de `rustls` dentro, y `is_tls()` contesta `false` en los dos modos de
/// cifrado. La rama no se ejecutaba nunca.
///
/// Lo que sí distingue es el error de E/S que hay debajo. Un handshake que
/// falla produce `InvalidData` —«hablé con alguien y lo que me dijo no vale»—
/// mientras que no llegar produce `ConnectionRefused`, `TimedOut` o
/// `HostUnreachable`. Es una clasificación por tipo, no por el texto del
/// mensaje: los textos cambian con la versión y con el idioma del sistema, y
/// este proyecto ya decidió en la 5.1 que el texto libre no decide nada.
fn fallo_de_certificado(e: &ErrorSmtp) -> bool {
    let mut fuente: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(e);
    while let Some(actual) = fuente {
        if let Some(io) = actual.downcast_ref::<std::io::Error>() {
            return io.kind() == std::io::ErrorKind::InvalidData;
        }
        fuente = actual.source();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODOS: [Diagnostico; 6] = [
        Diagnostico::Correcto,
        Diagnostico::NoSeLlegoAlServidor,
        Diagnostico::ElServidorTardoDemasiado,
        Diagnostico::ElCifradoNoSePudoEstablecer,
        Diagnostico::LaContrasenaNoSirve,
        Diagnostico::ElServidorNoEstaDisponible,
    ];

    /// **Ningún diagnóstico repite la clave de otro** (§139).
    ///
    /// Repetirla es el fallo silencioso: compila, no avisa, y dos problemas
    /// distintos enseñan el mismo «cómo se arregla» — o sea, a uno de los dos
    /// se le da la solución del otro.
    ///
    /// Verificada rompiéndola el 23/09/2026: dándole a `LaContrasenaNoSirve` la
    /// clave de `NoSeLlegoAlServidor`, falla por el duplicado.
    #[test]
    fn cada_diagnostico_tiene_su_propia_clave() {
        let mut claves: Vec<&str> = TODOS.iter().map(Diagnostico::clave_i18n).collect();
        let cuantos = claves.len();
        claves.sort_unstable();
        claves.dedup();
        assert_eq!(
            claves.len(),
            cuantos,
            "hay dos diagnósticos con la misma clave"
        );

        for d in TODOS {
            assert!(
                d.clave_i18n().starts_with("correo.prueba."),
                "{d:?} tiene que colgar de correo.prueba.*"
            );
        }
    }

    /// **Sólo uno de los seis deja usar la cuenta.**
    ///
    /// Si `sirve()` fuera cierto para alguno más, la pantalla dejaría guardar
    /// una cuenta que no puede enviar, y el problema aparecería la primera vez
    /// que corra una campaña de verdad — con contactos reales delante.
    #[test]
    fn solo_el_correcto_deja_usar_la_cuenta() {
        let sirven: Vec<_> = TODOS.into_iter().filter(Diagnostico::sirve).collect();
        assert_eq!(sirven, vec![Diagnostico::Correcto]);
    }
}
