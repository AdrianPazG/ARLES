//! Cómo se cifra la conexión, y por qué no hay forma de no cifrarla.
//!
//! `ABSTRACCION_EMAIL_PROVIDER.md` §2, §29 del brief.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NO HAY CASILLA PARA DESACTIVAR TLS. TAMPOCO AVANZADA. TAMPOCO CON AVISO
//!
//! El brief pedía «no permitir desactivar TLS **por defecto**». Aquí se va más
//! lejos: **no se permite en absoluto**, y el motivo es que una casilla así
//! siempre acaba marcada.
//!
//! El camino es conocido. Alguien conecta un servidor de correo interno mal
//! configurado, falla, encuentra la casilla, la marca, y funciona. A partir de
//! ese momento **la contraseña de correo de esa empresa viaja en claro por su
//! red, para siempre**, y nadie lo recuerda: el problema que llevó a marcarla
//! se resolvió hace meses y la casilla sigue puesta.
//!
//! Por eso este módulo no tiene ninguna función que devuelva «sin cifrar». No
//! es que esté desaconsejada: **no existe**. [`Modo`] tiene dos variantes y las
//! dos cifran.
//!
//! Lo mismo con los certificados: no hay opción de aceptar uno inválido ni uno
//! autofirmado. Si un servidor presenta un certificado que no valida, el
//! mensaje explica el problema en vez de ofrecer una salida insegura.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ENTONCES ¿CÓMO SE PRUEBA?
//!
//! Ésta es la pregunta que hunde a la mayoría de los proyectos que se ponen
//! esta regla, porque la respuesta fácil es exactamente la casilla que se acaba
//! de prohibir — `dangerous_accept_invalid_certs`, «sólo para los tests». Y una
//! vez que esa llamada está escrita en alguna parte, alguien la encuentra.
//!
//! La respuesta correcta es otra: en las pruebas **se confía en un certificado
//! concreto**, el del servidor de mentira, en vez de desactivar la
//! comprobación. La verificación sigue siendo estricta; lo único que cambia es
//! la lista de a quién se cree.
//!
//! La diferencia importa: [`Cifrado::confiando_ademas_en`] **añade** una raíz,
//! no quita ninguna comprobación. El certificado del servidor tiene que estar
//! firmado por esa raíz y el nombre tiene que cuadrar. Un atacante en medio
//! seguiría sin poder hacer nada, porque su certificado no es ése.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS RAÍCES SON LAS MISMAS EN LOS TRES SISTEMAS
//!
//! `lettre` va con `rustls` y con la lista de raíces de `webpki-roots`, no con
//! la del sistema operativo. Es deliberado: con la del sistema, un equipo con
//! el almacén de raíces tocado —una empresa que inspecciona el tráfico, un
//! programa que se instaló una raíz propia— validaría distinto que el de al
//! lado, y ARLES no podría decir qué comprobó.

use lettre::transport::smtp::client::{Certificate, Tls, TlsParameters};

use crate::error::ErrorDeCorreo;

/// El puerto de STARTTLS. Es el que usa Gmail.
///
/// La conexión empieza en claro y se cifra con el comando `STARTTLS` antes de
/// mandar la contraseña. **Nunca sale nada sensible antes de ese comando**: de
/// eso se encarga `lettre`, y hay una prueba que lo comprueba mirando lo que el
/// servidor llegó a recibir en claro.
pub const PUERTO_STARTTLS: u16 = 587;

/// El puerto de TLS desde el primer byte.
///
/// Aquí no hay un momento en claro: el apretón de manos va antes que el saludo
/// del servidor. Es más viejo que STARTTLS y sigue siendo perfectamente válido.
pub const PUERTO_TLS_DIRECTO: u16 = 465;

/// El principio de un certificado en PEM.
///
/// Se comprueba a mano porque `lettre` acepta sin rechistar un PEM que no
/// contiene ningún certificado: la lista sale vacía, la configuración se
/// construye, y el fallo aparece mucho después, al conectar, disfrazado de
/// «el servidor no valida». Ver la prueba del final.
const CABECERA_PEM: &str = "-----BEGIN CERTIFICATE-----";

/// Cómo viaja el cifrado. **No hay una variante sin cifrar.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modo {
    /// Se conecta en claro y se cifra con `STARTTLS` antes de autenticar.
    /// Puerto 587. Es lo que pide Gmail.
    Starttls,
    /// Cifrado desde el primer byte. Puerto 465.
    Directo,
}

impl Modo {
    /// El puerto que le corresponde, si no se dice otro.
    #[must_use]
    pub const fn puerto_habitual(self) -> u16 {
        match self {
            Self::Starttls => PUERTO_STARTTLS,
            Self::Directo => PUERTO_TLS_DIRECTO,
        }
    }

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Starttls => "correo.cifrado.starttls",
            Self::Directo => "correo.cifrado.directo",
        }
    }
}

/// El cifrado de una conexión: cómo viaja, y a quién se cree.
///
/// Se construye con [`Cifrado::starttls`] o [`Cifrado::directo`]. **Ninguno de
/// los dos admite no cifrar**, y no hay un tercero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cifrado {
    modo: Modo,
    /// Una raíz de confianza **de más**, nunca una comprobación de menos.
    ///
    /// Sólo lo rellenan las pruebas, con el certificado del servidor de
    /// mentira. Hay una comprobación del validador de fase que falla si algo
    /// fuera de un módulo de pruebas llama a `confiando_ademas_en`.
    raiz_de_mas: Option<Vec<u8>>,
}

impl Cifrado {
    /// Cifrado con `STARTTLS`, exigido. Es el de Gmail.
    #[must_use]
    pub const fn starttls() -> Self {
        Self {
            modo: Modo::Starttls,
            raiz_de_mas: None,
        }
    }

    /// Cifrado desde el primer byte.
    #[must_use]
    pub const fn directo() -> Self {
        Self {
            modo: Modo::Directo,
            raiz_de_mas: None,
        }
    }

    /// **Añade** una raíz de confianza a las de siempre. No quita ninguna
    /// comprobación.
    ///
    /// Existe para poder probar contra el servidor de mentira sin escribir en
    /// ninguna parte una llamada que acepte certificados inválidos. Ver la nota
    /// de arriba: es la diferencia entera entre «confío también en éste» y «no
    /// compruebo nada».
    #[must_use]
    pub fn confiando_ademas_en(mut self, pem: Vec<u8>) -> Self {
        self.raiz_de_mas = Some(pem);
        self
    }

    /// Cómo viaja.
    #[must_use]
    pub const fn modo(&self) -> Modo {
        self.modo
    }

    /// El puerto que le corresponde, si no se dice otro.
    #[must_use]
    pub const fn puerto_habitual(&self) -> u16 {
        self.modo.puerto_habitual()
    }

    /// Construye los parámetros que `lettre` necesita.
    ///
    /// `dominio` es el nombre contra el que se valida el certificado. **Tiene
    /// que ser el nombre, no la dirección IP**: un certificado se emite para
    /// `smtp.gmail.com`, no para el número al que resuelve ese día.
    ///
    /// # Errores
    ///
    /// [`ErrorDeCorreo::NoSePudoCifrar`] si la raíz de más no se puede leer o
    /// si la configuración de TLS falla. **No hay un camino que siga sin
    /// cifrar.**
    pub fn parametros(&self, dominio: &str) -> Result<Tls, ErrorDeCorreo> {
        let mut constructor = TlsParameters::builder(dominio.to_owned());

        if let Some(pem) = &self.raiz_de_mas {
            // `lettre` acepta un PEM sin ningún certificado dentro: se queda
            // con una lista vacía y devuelve `Ok`. Entonces el error aparece al
            // conectar, con la forma de «el servidor no valida», que apunta al
            // sitio equivocado. Se comprueba aquí.
            let texto = std::str::from_utf8(pem).map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?;
            if !texto.contains(CABECERA_PEM) {
                return Err(ErrorDeCorreo::NoSePudoCifrar);
            }
            let cert = Certificate::from_pem(pem).map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?;
            // `add_root_certificate` AÑADE una raíz. No desactiva nada: el
            // certificado del servidor tiene que estar firmado por ésta o por
            // una de las de siempre, y el nombre tiene que cuadrar.
            constructor = constructor.add_root_certificate(cert);
        }

        let parametros = constructor
            .build()
            .map_err(|_| ErrorDeCorreo::NoSePudoCifrar)?;

        Ok(match self.modo {
            Modo::Directo => Tls::Wrapper(parametros),
            // `Required`, no `Opportunistic`. La diferencia es exactamente el
            // agujero que este módulo existe para cerrar: con `Opportunistic`,
            // un servidor que no ofrezca STARTTLS —o un atacante que borre esa
            // línea de la respuesta al `EHLO`— deja la conversación en claro, y
            // la contraseña sale sin cifrar sin que nada avise.
            Modo::Starttls => Tls::Required(parametros),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **No hay ninguna forma de pedir una conexión sin cifrar.**
    ///
    /// Es la comprobación de que la regla vive en los tipos y no en la
    /// disciplina de quien escribe el código: `Modo` tiene dos variantes y las
    /// dos cifran. No hay una tercera que alguien pueda elegir un martes por la
    /// tarde para hacer funcionar un servidor mal configurado.
    #[test]
    fn no_existe_una_conexion_sin_cifrar() {
        for cifrado in [Cifrado::starttls(), Cifrado::directo()] {
            let tls = cifrado.parametros("smtp.gmail.com").expect("parámetros");
            assert!(
                matches!(tls, Tls::Required(_) | Tls::Wrapper(_)),
                "ni «ninguno» ni «oportunista»: las dos formas cifran"
            );
        }
    }

    /// **STARTTLS se exige, no se ofrece.**
    ///
    /// Con `Opportunistic`, un servidor que no anuncie STARTTLS —o un atacante
    /// que borre esa línea— dejaría la conversación en claro, y la contraseña
    /// saldría sin cifrar sin que nada avisara. Es el agujero clásico de este
    /// protocolo.
    ///
    /// Verificada rompiéndola el 23/09/2026: cambiando `Tls::Required` por
    /// `Tls::Opportunistic`, esta prueba falla. La de extremo a extremo de
    /// `smtp.rs` **no** —la salva la comprobación `is_encrypted()` del
    /// adaptador—, que es justo para lo que esa comprobación está.
    #[test]
    fn el_cifrado_se_exige_y_no_se_negocia() {
        let tls = Cifrado::starttls()
            .parametros("smtp.gmail.com")
            .expect("parámetros");
        assert!(
            matches!(tls, Tls::Required(_)),
            "«oportunista» significa «en claro si el otro lado lo pide», y eso \
             es exactamente lo que no se puede permitir"
        );
    }

    /// El puerto directo envuelve la conexión entera desde el primer byte.
    #[test]
    fn el_puerto_directo_cifra_desde_el_principio() {
        let tls = Cifrado::directo()
            .parametros("smtp.gmail.com")
            .expect("parámetros");
        assert!(matches!(tls, Tls::Wrapper(_)));
    }

    /// Los puertos de siempre, por si alguien los cambia sin querer.
    #[test]
    fn los_puertos_son_los_de_siempre() {
        assert_eq!(Cifrado::starttls().puerto_habitual(), 587);
        assert_eq!(Cifrado::directo().puerto_habitual(), 465);
        assert_eq!(PUERTO_STARTTLS, 587, "el de Gmail");
    }

    /// **Confiar en una raíz de más no relaja el cifrado.**
    ///
    /// Si lo relajara, habríamos reintroducido por la puerta de atrás justo lo
    /// que este módulo existe para cerrar.
    #[test]
    fn confiar_en_una_raiz_de_mas_no_relaja_el_cifrado() {
        let pem = format!("{CABECERA_PEM}\nMIIB\n-----END CERTIFICATE-----\n");
        let cifrado = Cifrado::starttls().confiando_ademas_en(pem.into_bytes());
        // El PEM de mentira no llega a validar como certificado, pero lo que se
        // comprueba aquí es el camino: o sale `Required`, o sale error. Nunca
        // «pues entonces sin cifrar».
        match cifrado.parametros("localhost") {
            Ok(tls) => assert!(matches!(tls, Tls::Required(_))),
            Err(e) => assert_eq!(e, ErrorDeCorreo::NoSePudoCifrar),
        }
    }

    /// **Un PEM que no contiene ningún certificado es un error aquí, no un
    /// misterio dentro de tres pasos.**
    ///
    /// Éste es un fallo de verdad que apareció al escribir la entrega:
    /// `Certificate::from_pem` con basura dentro **devuelve `Ok`** con una
    /// lista vacía de certificados. Sin esta comprobación, el adaptador se
    /// construía tan tranquilo y el problema salía después, al conectar, con la
    /// forma de «el certificado del servidor no valida» — que manda a mirar al
    /// servidor cuando el problema estaba en el archivo que se pegó.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando el `contains`, esta
    /// prueba falla porque `parametros` devuelve `Ok`.
    #[test]
    fn un_pem_sin_certificados_se_caza_aqui() {
        let malo = Cifrado::starttls().confiando_ademas_en(b"esto no es un PEM".to_vec());
        assert_eq!(
            malo.parametros("localhost").err(),
            Some(ErrorDeCorreo::NoSePudoCifrar),
            "un PEM vacío tiene que fallar donde se escribió, no al conectar"
        );
    }

    /// **La llamada que desactiva la verificación no está escrita.**
    ///
    /// `dangerous_accept_invalid_certs` es la función de `lettre` que apaga la
    /// comprobación de certificados. Existe, es una línea, y resuelve cualquier
    /// problema al instante — por eso la prueba no es «no la usamos», sino «no
    /// está escrita en ninguna parte».
    ///
    /// Una vez que esa llamada existe en el código, aunque sea tras un
    /// `cfg(test)`, alguien la encuentra y la mueve. El validador de fase
    /// comprueba lo mismo sobre el árbol entero; esto lo comprueba sobre el
    /// archivo donde la tentación es mayor.
    #[test]
    fn la_llamada_que_desactiva_la_verificacion_no_esta_escrita() {
        let fuente = include_str!("tls.rs");
        // El texto que se busca se arma por trozos a propósito. Escribirlo
        // entero aquí haría que la prueba se encontrara a sí misma y fallara
        // siempre — que es justo lo que pasó al escribirla, el 23/09/2026: la
        // comprobación funcionaba, sólo que apuntándose a sí misma.
        let llamada = |que: &str| format!(".dangerous_accept_invalid_{que}(");
        assert!(
            !fuente.contains(&llamada("certs")),
            "si esta llamada aparece, la regla del §29 se ha perdido"
        );
        assert!(
            !fuente.contains(&llamada("hostnames")),
            "y la del nombre del servidor, igual"
        );
    }
}
