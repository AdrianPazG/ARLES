//! Traducir lo que contesta Meta a una de las cinco decisiones del motor.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTE ARCHIVO DECIDE QUIÉN SE QUEDA SIN RECIBIR NUNCA MÁS
//!
//! De los cinco fallos, uno —[`Fallo::LaDireccionNoExiste`]— lleva a
//! `RendirseYDarDeBaja`, y una baja **no se deshace con un botón**. Así que la
//! lista de códigos que llevan ahí es cerrada, corta y está escrita a mano,
//! y todo lo demás cae en el lado seguro.
//!
//! Es la misma disciplina que `arles-email::codigos` aplica a los `5xx` de
//! SMTP, y por la misma razón: confundir «ahora no puedo» con «esta persona no
//! existe» borra a alguien alcanzable de todas las campañas futuras.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE NO RECONOCEMOS ES PASAJERO, Y ESO ES UNA DECISIÓN
//!
//! Meta añade códigos sin avisar. Un código nuevo cae en [`Fallo::Pasajero`],
//! que reintenta unas cuantas veces y se rinde: **no da de baja a nadie y no
//! para la cuenta**.
//!
//! El coste es unos reintentos inútiles. La alternativa —tratar lo desconocido
//! como definitivo— convertiría un código nuevo de Meta en una purga silenciosa
//! de la lista de contactos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL TEXTO DE META NO DECIDE NADA
//!
//! Sólo el código numérico. El `title` y el `message` que acompañan al error
//! son texto libre que Meta cambia cuando quiere, y traducidos a inglés o a
//! español según la cuenta. Mirarlos haría que la decisión de dar de baja a
//! alguien dependiera de una cadena que nadie nos garantiza.

use std::time::Duration;

use arles_core::reintento::Fallo;

/// Un error tal como lo devuelve Cloud API.
///
/// Meta anida el detalle interesante en `error.error_data.details` y el código
/// fino en `error_subcode`; aquí llegan ya extraídos por [`crate::cloud`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDeMeta {
    /// El código de la API. Es lo único que decide.
    pub codigo: u32,
    /// El subcódigo, cuando lo hay. Afina dos casos y ninguno más.
    pub subcodigo: Option<u32>,
    /// El estado HTTP. Sirve para el caso en que Meta ni siquiera devuelve un
    /// cuerpo con forma de error — un 502 del balanceador, por ejemplo.
    pub http: u16,
}

// ── Los códigos que importan, agrupados por lo que hay que hacer ───────────

/// El número no puede recibir este mensaje **por ser ese número**.
///
/// `131026` es lo más parecido que tiene WhatsApp a un `550` de SMTP: el
/// destino no tiene WhatsApp, o su cuenta no puede recibir mensajes de
/// empresa. Insistir no lo arregla y sube el contador que Meta usa para medir
/// la calidad del número que envía.
///
/// **Es el único código que da de baja.** Y la baja es del canal de WhatsApp,
/// no de la persona: su correo sigue valiendo (V3).
const NO_SE_PUEDE_ENTREGAR: u32 = 131_026;

/// La cuenta no puede enviar, y no es cosa de este mensaje.
///
/// `190` es el token caducado o revocado. `133_010` es el número sin registrar
/// en Cloud API. `131_031` es la cuenta inhabilitada por Meta, que es terminal
/// y lo que más duele: se pierde el número. `368` es un bloqueo temporal por
/// políticas — se levanta solo, pero mientras tanto no sale nada.
///
/// Los cuatro paran la cuenta entera. Seguir intentando con una cuenta así sólo
/// añade fallos al historial que Meta mira.
const CUENTA_INHABILITADA: [u32; 4] = [190, 133_010, 131_031, 368];

/// Meta pide bajar el ritmo.
///
/// `130_429` es el límite de mensajes por segundo. `4` y `80_007` son límites
/// de la aplicación y de la cuenta de empresa. Los tres se arreglan solos
/// esperando, y por eso llevan una espera concreta en vez de un reintento
/// inmediato: el motor la respeta y deja de golpear.
const DEMASIADO_DEPRISA: [u32; 3] = [130_429, 4, 80_007];

/// Lo que Meta llama «límite de ecosistema sano».
///
/// `131_049` no es un fallo del mensaje: Meta decidió que a **esa persona** ya
/// le llegó bastante publicidad hoy. Se levanta al día siguiente.
///
/// Va con una espera de un día y no con un reintento normal porque reintentar
/// dentro de cinco minutos da exactamente el mismo error, tres veces, y luego
/// se rinde — gastando tres intentos fallidos del historial para nada.
const YA_RECIBIO_BASTANTE_HOY: u32 = 131_049;

/// El número está mandando más de lo que su calidad aguanta.
///
/// `131_048` es el aviso serio: Meta ha visto bloqueos o reportes y ha bajado
/// el límite. **No da de baja a nadie** —el problema es del número que envía,
/// no de quien recibe— pero tampoco es un reintento cualquiera.
///
/// Se traduce a una espera larga para que la campaña frene de verdad. La otra
/// mitad de la respuesta es la pantalla: el semáforo de calidad tiene que
/// enseñarlo, porque lo que hay que hacer con esto es mirar **a quién se está
/// escribiendo**, no esperar.
const RITMO_POR_ENCIMA_DE_LA_CALIDAD: u32 = 131_048;

/// Cuánto esperar cuando Meta dice que el destinatario ya recibió bastante hoy.
///
/// Un día y un poco: el contador de Meta va por día natural en su zona, que no
/// conocemos, y volver a intentar justo al filo devuelve el mismo error.
pub const ESPERA_DE_UN_DIA: Duration = Duration::from_secs(25 * 60 * 60);

/// Cuánto frenar cuando la calidad del número no da para el ritmo actual.
pub const ESPERA_POR_CALIDAD: Duration = Duration::from_secs(60 * 60);

/// Cuánto esperar ante un límite de velocidad.
///
/// Corto a propósito: es el caso que de verdad se arregla solo en seguida, y
/// una espera larga aquí dejaría la campaña parada media hora por un pico de
/// tres segundos.
pub const ESPERA_POR_RITMO: Duration = Duration::from_secs(60);

/// Traduce un error de Meta a la decisión del motor.
///
/// El orden de las ramas es deliberado: primero lo que **para la cuenta**,
/// luego lo que **da de baja**, y sólo al final lo que espera. Escrito al
/// revés, un código que estuviera en dos listas por error se trataría como el
/// caso más benigno, que es justo el que no queremos que gane por accidente.
#[must_use]
pub fn traducir(e: &ErrorDeMeta) -> Fallo {
    // El token caducado llega con código 190, y también como un 401 sin
    // cuerpo útil cuando Meta está de mal humor. Los dos son lo mismo.
    if CUENTA_INHABILITADA.contains(&e.codigo) || e.http == 401 {
        return Fallo::LaCuentaNoAutentica;
    }

    if e.codigo == NO_SE_PUEDE_ENTREGAR {
        return Fallo::LaDireccionNoExiste;
    }

    if e.codigo == YA_RECIBIO_BASTANTE_HOY {
        return Fallo::NoMeMandesNadaDurante(ESPERA_DE_UN_DIA);
    }

    if e.codigo == RITMO_POR_ENCIMA_DE_LA_CALIDAD {
        return Fallo::NoMeMandesNadaDurante(ESPERA_POR_CALIDAD);
    }

    if DEMASIADO_DEPRISA.contains(&e.codigo) || e.http == 429 {
        return Fallo::NoMeMandesNadaDurante(ESPERA_POR_RITMO);
    }

    // Todo lo demás. Ver la nota de la cabecera: lo desconocido no da de baja
    // a nadie y no para la cuenta.
    Fallo::Pasajero
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de(codigo: u32) -> ErrorDeMeta {
        ErrorDeMeta {
            codigo,
            subcodigo: None,
            http: 400,
        }
    }

    /// **Sólo un código da de baja, y hay que enumerarlos para verlo.**
    ///
    /// Una baja no se deshace con un botón: borra a alguien de todas las
    /// campañas futuras. Esta prueba recorre todos los códigos que este archivo
    /// conoce y exige que **exactamente uno** lleve ahí.
    ///
    /// Es la prueba que caza el error más caro de este módulo, que no es
    /// escribir mal una traducción sino **añadir un código a la lista
    /// equivocada** meses después, cuando ya nadie recuerda cuál era cuál.
    ///
    /// Verificada rompiéndola el 29/09/2026: añadiendo `131_047` a la rama de
    /// `NO_SE_PUEDE_ENTREGAR`, esta prueba falla nombrando el código.
    #[test]
    fn solo_un_codigo_da_de_baja() {
        let conocidos = [
            190, 368, 133_010, 131_031, 131_026, 131_048, 131_049, 130_429, 4, 80_007, 131_047, 0,
        ];

        let que_dan_de_baja: Vec<u32> = conocidos
            .into_iter()
            .filter(|c| traducir(&de(*c)) == Fallo::LaDireccionNoExiste)
            .collect();

        assert_eq!(
            que_dan_de_baja,
            vec![131_026],
            "sólo «no se puede entregar» puede dar de baja a alguien"
        );
    }

    /// **Un código que no conocemos no da de baja ni para la cuenta.**
    ///
    /// Meta añade códigos sin avisar. El coste de tratarlos como pasajeros son
    /// unos reintentos inútiles; el de tratarlos como definitivos sería una
    /// purga silenciosa de la lista de contactos el día que Meta estrene un
    /// error.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo
    /// `Fallo::LaDireccionNoExiste` en la rama final, esta prueba falla.
    #[test]
    fn lo_que_no_conocemos_cae_del_lado_seguro() {
        for inventado in [999_999, 1, 42, 131_099] {
            assert_eq!(
                traducir(&de(inventado)),
                Fallo::Pasajero,
                "el código {inventado} no debería decidir nada grave"
            );
        }
    }

    /// **La cuenta inhabilitada para la cuenta, no el mensaje.**
    ///
    /// Los cuatro casos son distintos para quien los lee —token caducado,
    /// número sin registrar, cuenta inhabilitada, bloqueo por políticas— y el
    /// mismo para el motor: dejar de intentar con esa cuenta. Seguir sólo añade
    /// fallos al historial que Meta mira para decidir si te inhabilita del todo.
    #[test]
    fn lo_que_rompe_la_cuenta_para_la_cuenta() {
        for codigo in CUENTA_INHABILITADA {
            assert_eq!(
                traducir(&de(codigo)),
                Fallo::LaCuentaNoAutentica,
                "{codigo}"
            );
        }
        // Y un 401 sin cuerpo útil es lo mismo, aunque no traiga código.
        assert_eq!(
            traducir(&ErrorDeMeta {
                codigo: 0,
                subcodigo: None,
                http: 401
            }),
            Fallo::LaCuentaNoAutentica
        );
    }

    /// **Lo que se arregla esperando lleva una espera concreta, no un reintento.**
    ///
    /// Y las tres esperas son distintas porque los tres casos lo son. Un pico
    /// de velocidad se pasa en un minuto; que Meta haya bajado el límite por
    /// calidad no se arregla en un minuto; y que una persona ya haya recibido
    /// su cupo de publicidad del día no se arregla hasta mañana.
    ///
    /// Con una sola espera para los tres, o se frena media hora por un pico de
    /// tres segundos, o se gastan tres intentos fallidos contra un límite
    /// diario — y los intentos fallidos cuentan.
    #[test]
    fn cada_espera_es_la_de_su_caso() {
        assert_eq!(
            traducir(&de(130_429)),
            Fallo::NoMeMandesNadaDurante(ESPERA_POR_RITMO)
        );
        assert_eq!(
            traducir(&de(131_048)),
            Fallo::NoMeMandesNadaDurante(ESPERA_POR_CALIDAD)
        );
        assert_eq!(
            traducir(&de(131_049)),
            Fallo::NoMeMandesNadaDurante(ESPERA_DE_UN_DIA)
        );

        assert!(
            ESPERA_POR_RITMO < ESPERA_POR_CALIDAD && ESPERA_POR_CALIDAD < ESPERA_DE_UN_DIA,
            "las tres esperas tienen que ir de menos a más, o dos de ellas sobran"
        );
    }

    /// **El texto de Meta no decide nada.**
    ///
    /// La estructura no tiene dónde guardar el mensaje, y esta prueba existe
    /// para que eso sea una decisión visible y no un descuido: el `title` de
    /// Meta es texto libre que cambia cuando quieren y llega traducido según la
    /// cuenta. Si decidiera, dar de baja a alguien dependería de una cadena que
    /// nadie nos garantiza.
    ///
    /// Dos errores con el mismo código son el mismo error, venga como venga.
    #[test]
    fn dos_errores_con_el_mismo_codigo_deciden_igual() {
        let uno = ErrorDeMeta {
            codigo: 131_026,
            subcodigo: Some(1),
            http: 400,
        };
        let otro = ErrorDeMeta {
            codigo: 131_026,
            subcodigo: Some(99),
            http: 404,
        };
        assert_eq!(traducir(&uno), traducir(&otro));
    }

    /// Un 429 sin código reconocible también es «baja el ritmo».
    ///
    /// Pasa cuando quien contesta es el balanceador de Meta y no la API: no hay
    /// cuerpo con forma de error, sólo el estado HTTP.
    #[test]
    fn un_429_pelado_tambien_frena() {
        assert_eq!(
            traducir(&ErrorDeMeta {
                codigo: 0,
                subcodigo: None,
                http: 429
            }),
            Fallo::NoMeMandesNadaDurante(ESPERA_POR_RITMO)
        );
    }
}
