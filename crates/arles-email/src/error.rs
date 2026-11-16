//! Lo que puede salir mal **antes** de hablar con el servidor.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTO NO ES LO MISMO QUE UN `Fallo`
//!
//! Son dos cosas distintas y conviene no mezclarlas:
//!
//! - [`ErrorDeCorreo`] es **la cuenta está mal configurada**. Ocurre al
//!   construir el adaptador, antes de que exista ningún envío, y no tiene nada
//!   que ver con ningún contacto. Se le enseña a quien configura la cuenta, en
//!   la pantalla de Remitentes.
//! - `arles_core::reintento::Fallo` es **este envío concreto no salió**. Lo
//!   produce el servidor, y el motor decide con él si reintenta, se rinde o
//!   para la cuenta.
//!
//! Si un problema de configuración llegara como `Fallo`, el motor lo trataría
//! como un envío fallido: gastaría reintentos, contaría para el disyuntor y
//! marcaría contactos. Un puerto mal escrito acabaría dando de baja a nadie —
//! pero sí dejaría la cuenta parada «por fallos seguidos», que es exactamente
//! el diagnóstico equivocado.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! CADA VARIANTE LLEVA SU CLAVE (§139)
//!
//! Nunca texto. El texto vive en `app/src/app/locales/es.ts` bajo
//! `error.correo.*`, con sus tres partes: qué pasó, cómo se arregla y qué está
//! a salvo (§95).

use std::fmt;

/// Un problema de configuración de la cuenta de correo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDeCorreo {
    /// No se pudo preparar el cifrado de la conexión.
    ///
    /// **No existe un camino que siga sin cifrar** (§29). Si esto falla, no hay
    /// adaptador; ver la nota de `tls.rs`.
    NoSePudoCifrar,
    /// El servidor que se escribió no es un nombre de máquina utilizable.
    ///
    /// Tiene que ser el **nombre** —`smtp.gmail.com`—, no una dirección IP: un
    /// certificado se emite para el nombre, y validarlo contra el número al que
    /// resuelve ese día no funciona.
    ElServidorNoVale,
    /// La dirección del remitente no es una dirección de correo.
    LaDireccionDelRemitenteNoVale,
    /// El mensaje no se pudo armar con los datos que hay.
    ///
    /// Es un fallo nuestro, no del servidor ni de la cuenta: llega antes de
    /// abrir ninguna conexión.
    NoSePudoArmarElMensaje,
    /// No se llegó al servidor de lectura del buzón (R-25).
    ///
    /// Es el de siempre: el nombre está mal, no hay red, o el puerto está
    /// cerrado. Lo que está a salvo es **todo**: las bajas que ya estaban
    /// aplicadas siguen, y leer el buzón no cambia nada en él.
    NoSePudoEntrarAlBuzon,
    /// El servidor del buzón dijo que esa contraseña no vale.
    ///
    /// **Variante propia y no la de conectar**, porque el arreglo es otro: aquí
    /// no se reintenta, se revisa la contraseña de aplicación. Y separada del
    /// envío a propósito: la contraseña es la misma, pero un proveedor puede
    /// tener **IMAP apagado** con el SMTP funcionando —en Gmail es una casilla
    /// de la cuenta—, así que «manda bien y no puede leer» es un estado real y
    /// su texto tiene que decir dónde se enciende.
    ElBuzonRechazoLaContrasena,
    /// El servidor del buzón contestó algo que no se entiende.
    ///
    /// Incluye los topes: un literal más grande de lo que se acepta, una línea
    /// interminable o más correos de los que caben en una pasada. Lo que está a
    /// salvo es todo, por lo mismo que arriba.
    ElBuzonContestoAlgoRaro,
}

impl ErrorDeCorreo {
    /// La clave del texto que se le enseña a quien configura la cuenta (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::NoSePudoCifrar => "error.correo.no_se_pudo_cifrar",
            Self::ElServidorNoVale => "error.correo.el_servidor_no_vale",
            Self::LaDireccionDelRemitenteNoVale => "error.correo.remitente_no_vale",
            Self::NoSePudoArmarElMensaje => "error.correo.no_se_pudo_armar",
            Self::NoSePudoEntrarAlBuzon => "error.correo.no_se_pudo_entrar_al_buzon",
            Self::ElBuzonRechazoLaContrasena => "error.correo.buzon_rechazo_contrasena",
            Self::ElBuzonContestoAlgoRaro => "error.correo.buzon_contesto_raro",
        }
    }
}

impl fmt::Display for ErrorDeCorreo {
    /// Escribe **la clave**, no el texto.
    ///
    /// Es a propósito: si esto escribiera español, ese español acabaría en una
    /// pantalla sin pasar por el catálogo, y el día que haya una segunda lengua
    /// sería el único sitio que no se traduce. Además, un `Display` de un error
    /// acaba en bitácoras, y una clave es lo que se puede buscar.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.clave_i18n())
    }
}

impl std::error::Error for ErrorDeCorreo {}

#[cfg(test)]
mod tests {
    use super::*;

    const TODOS: [ErrorDeCorreo; 7] = [
        ErrorDeCorreo::NoSePudoCifrar,
        ErrorDeCorreo::ElServidorNoVale,
        ErrorDeCorreo::LaDireccionDelRemitenteNoVale,
        ErrorDeCorreo::NoSePudoArmarElMensaje,
        ErrorDeCorreo::NoSePudoEntrarAlBuzon,
        ErrorDeCorreo::ElBuzonRechazoLaContrasena,
        ErrorDeCorreo::ElBuzonContestoAlgoRaro,
    ];

    /// **`TODOS` es una lista a mano, y esto es lo que impide que se quede
    /// atrás.**
    ///
    /// Es el defecto que este proyecto lleva once veces arreglando: una lista
    /// escrita a mano que deja de cubrir lo que dice cubrir. Aquí no se puede
    /// leer el enum de ninguna carpeta —Rust no tiene reflexión sobre
    /// variantes—, así que lo que hace el trabajo es el **compilador**: este
    /// `match` es exhaustivo, de modo que una variante nueva no compila hasta
    /// que alguien la nombre aquí, y al nombrarla el `assert` exige que esté
    /// también en `TODOS`.
    ///
    /// Verificada añadiendo una variante de prueba al enum: **no compila**,
    /// que es antes todavía que el rojo de una prueba.
    #[test]
    fn la_lista_de_todos_los_errores_los_tiene_a_todos() {
        for e in TODOS {
            // El `match` no se usa para nada más que forzar la exhaustividad.
            let nombrada = match e {
                ErrorDeCorreo::NoSePudoCifrar
                | ErrorDeCorreo::ElServidorNoVale
                | ErrorDeCorreo::LaDireccionDelRemitenteNoVale
                | ErrorDeCorreo::NoSePudoArmarElMensaje
                | ErrorDeCorreo::NoSePudoEntrarAlBuzon
                | ErrorDeCorreo::ElBuzonRechazoLaContrasena
                | ErrorDeCorreo::ElBuzonContestoAlgoRaro => true,
            };
            assert!(nombrada && TODOS.contains(&e), "{e:?} no está en TODOS");
        }
    }

    /// **Ningún error se queda sin clave, y ninguna clave se repite** (§139).
    ///
    /// Repetir una clave es el fallo silencioso de los dos: compila, no avisa,
    /// y en la pantalla dos problemas distintos dicen lo mismo — con lo cual el
    /// texto de «cómo se arregla» es el de otro problema.
    ///
    /// Verificada rompiéndola el 23/09/2026: dándole a `ElServidorNoVale` la
    /// clave de `NoSePudoCifrar`, esta prueba falla por el duplicado.
    #[test]
    fn cada_error_tiene_su_propia_clave() {
        let mut claves: Vec<&str> = TODOS.iter().map(ErrorDeCorreo::clave_i18n).collect();
        let cuantas = claves.len();
        claves.sort_unstable();
        claves.dedup();
        assert_eq!(claves.len(), cuantas, "hay dos errores con la misma clave");

        for e in TODOS {
            assert!(
                e.clave_i18n().starts_with("error.correo."),
                "{e:?} tiene que colgar de error.correo.*"
            );
        }
    }

    /// **El `Display` no escribe español.**
    ///
    /// Si lo escribiera, ese texto acabaría en una pantalla sin pasar por el
    /// catálogo de textos, y sería el único sitio que no se traduce.
    #[test]
    fn el_display_escribe_la_clave_y_no_el_texto() {
        for e in TODOS {
            assert_eq!(e.to_string(), e.clave_i18n());
        }
    }
}
