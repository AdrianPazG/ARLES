//! Lo que dice un servidor de correo, traducido a lo que el motor entiende.
//!
//! `ABSTRACCION_EMAIL_PROVIDER.md` §2.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTA TABLA ES TODO EL VALOR DE LA FASE 5
//!
//! El motor entiende **cinco fallos**. Un servidor de correo habla en números.
//! Traducir de lo uno a lo otro es el trabajo entero del adaptador, y una fila
//! mal puesta tiene consecuencias que no se pueden deshacer:
//!
//! **Confundir un `450` con un `550` da de baja a alguien que sí existe.** El
//! `450` significa «ahora no puedo, vuelve luego» y el `550` significa «ese
//! buzón no existe». Si se traduce el primero como el segundo, esa dirección
//! entra en la lista de bajas — y de la lista de bajas no se sale con un botón
//! cuando el motivo es un rebote.
//!
//! **Y al revés, confundir un `550` con un `450` insiste tres veces contra un
//! buzón que no existe**, que es exactamente lo que sube el contador que los
//! proveedores usan para marcar a un remitente como spam.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TRES NIVELES DE CERTEZA, Y SE USAN EN ORDEN
//!
//! Un servidor contesta con hasta tres cosas, de más precisa a menos:
//!
//! ```text
//! 550 5.1.1 The email account that you tried to reach does not exist
//! ^^^ ^^^^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//!  |    |    texto libre: NO se interpreta. Sólo se enseña.
//!  |    código extendido (RFC 3463): lo más preciso que hay
//!  código básico (RFC 5321)
//! ```
//!
//! **1 · El código extendido**, si viene. `5.1.1` dice «la dirección de destino
//! está mal» y `5.3.4` dice «tu mensaje es demasiado grande» — los dos son
//! `5xx`, y confundirlos suprimiría a alguien por culpa de un archivo adjunto.
//!
//! **2 · El código básico**, si el extendido no viene.
//!
//! **3 · La clase**, si el código no se reconoce: `4xx` es temporal y `5xx` es
//! permanente (RFC 5321 §4.2.1). Un código desconocido **nunca** se adivina al
//! revés: un `5xx` raro tratado como temporal reintentaría para siempre, y un
//! `4xx` raro tratado como permanente suprimiría a quien no debía.
//!
//! **El texto libre no se interpreta jamás.** Se recorta, se guarda y se
//! enseña. Buscar palabras en él —«full», «unknown», «try again»— funciona con
//! el servidor con el que se probó y falla con todos los demás, en idiomas que
//! no se leyeron nunca.

use arles_core::reintento::Fallo;

/// Lo que contestó el servidor, ya separado en sus partes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespuestaSmtp {
    /// El código básico de tres cifras: `250`, `550`, `421`…
    pub codigo: u16,
    /// El código extendido de RFC 3463, si vino: `(5, 1, 1)`.
    pub extendido: Option<(u8, u8, u8)>,
    /// El texto, tal cual. **No se interpreta.**
    pub texto: String,
}

impl RespuestaSmtp {
    /// ¿Aceptó el servidor?
    ///
    /// Sólo la clase `2xx`. La `3xx` es «sigue hablando» —la respuesta a
    /// `DATA`— y no significa que el mensaje esté aceptado: el `250` llega
    /// después del punto final. Tratar un `354` como éxito daría por enviado un
    /// mensaje cuyo cuerpo no se ha mandado todavía.
    #[must_use]
    pub const fn acepto(&self) -> bool {
        self.codigo >= 200 && self.codigo < 300
    }

    /// La clase: 2, 3, 4 o 5.
    #[must_use]
    pub const fn clase(&self) -> u8 {
        #[allow(clippy::cast_possible_truncation)]
        {
            (self.codigo / 100) as u8
        }
    }
}

/// Traduce la respuesta de un servidor al fallo que el motor entiende.
///
/// Devuelve `None` si el servidor **aceptó**: no hay nada que traducir.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LAS CUATRO FILAS QUE NO SON OBVIAS
///
/// **`552` (buzón lleno) es temporal, aunque sea `5xx`.** El número dice
/// permanente y la realidad dice que el martes que viene ese buzón tendrá
/// sitio. Suprimir a un cliente porque su buzón estaba lleno un día es un
/// error irreversible por una causa que se arregla sola. Se reintenta.
///
/// **`5.3.4` (mensaje demasiado grande) no es culpa del destinatario.** Es
/// culpa de nuestro mensaje, y esa dirección funciona perfectamente. Se rinde
/// **sin suprimir**: insistir con el mismo mensaje daría el mismo resultado,
/// pero la persona sigue siendo alcanzable.
///
/// **`5.7.x` (política y seguridad) habla de la CUENTA, no del destinatario.**
/// «Relay denegado», «tu IP está bloqueada», «no estás autorizado a enviar como
/// esa dirección». Suprimir al destinatario por esto sería castigar a la
/// persona equivocada, y encima escondería el problema real — que está en la
/// cuenta remitente y lo va a tener con todos los demás envíos.
///
/// **`535` no espera a cinco fallos.** La credencial no sirve, y el disyuntor
/// lo sabe: abre a la primera (`arles_core::disyuntor`).
#[must_use]
pub fn traducir(r: &RespuestaSmtp) -> Option<Fallo> {
    if r.acepto() {
        return None;
    }

    // ── 1 · El código extendido manda, si vino ──────────────────────────────
    if let Some((clase, sujeto, detalle)) = r.extendido
        && let Some(fallo) = por_extendido(clase, sujeto, detalle)
    {
        return Some(fallo);
    }

    // ── 2 · El código básico ────────────────────────────────────────────────
    let por_codigo = match r.codigo {
        // Autenticación. RFC 4954.
        530 | 534 | 535 | 538 => Some(Fallo::LaCuentaNoAutentica),

        // El buzón no existe, o el servidor lo rechaza para siempre.
        550 | 551 | 553 | 554 => Some(Fallo::LaDireccionNoExiste),

        // Buzón lleno. Codificado como permanente, temporal en la práctica.
        // Ver la nota de arriba: es el caso donde el número miente.
        552 => Some(Fallo::Pasajero),

        // Servicio no disponible, buzón ocupado, error local, sin espacio.
        421 | 450 | 451 | 452 => Some(Fallo::Pasajero),

        _ => None,
    };
    if por_codigo.is_some() {
        return por_codigo;
    }

    // ── 3 · La clase, para lo que no se reconoce ────────────────────────────
    //
    // Nunca al revés. Un 5xx desconocido tratado como temporal reintentaría
    // para siempre; un 4xx desconocido tratado como permanente suprimiría a
    // quien no debía.
    Some(match r.clase() {
        4 => Fallo::Pasajero,
        // Un 5xx que no se reconoce se rinde, pero **no suprime**: no hay nada
        // que diga que la dirección esté mal, y suprimir es irreversible.
        _ => Fallo::LaDireccionNoExiste,
    })
}

/// La traducción por código extendido (RFC 3463), que es la más precisa.
///
/// Devuelve `None` cuando el extendido no aporta nada que el código básico no
/// diga mejor.
const fn por_extendido(clase: u8, sujeto: u8, detalle: u8) -> Option<Fallo> {
    match (clase, sujeto, detalle) {
        // 5.1.x · la dirección de destino. Es el caso claro de supresión.
        // El 5.1.8 es la dirección del REMITENTE, no la del destinatario.
        (5, 1, 8) => Some(Fallo::LaCuentaNoAutentica),
        (5, 1, _) => Some(Fallo::LaDireccionNoExiste),

        // 4.2.2 y 5.2.2 · buzón lleno. Se arregla solo.
        (_, 2, 2) => Some(Fallo::Pasajero),
        // 5.2.3 · el mensaje excede el límite de ese buzón. No es la persona.
        (5, 2, 3) => Some(Fallo::Pasajero),

        // 5.3.4 · el mensaje es demasiado grande para el sistema. Es NUESTRO
        // mensaje el que está mal, no la dirección. Se rinde sin suprimir.
        (5, 3, 4) => Some(Fallo::Pasajero),

        // 5.5.x · error de protocolo. Nuestro, y no se arregla reintentando,
        // pero tampoco dice nada de la dirección.
        (5, 5, _) => Some(Fallo::Pasajero),

        // 5.7.x · política y seguridad: relay denegado, IP bloqueada, no
        // autorizado a enviar como esa dirección. Habla de la CUENTA.
        (5, 7, _) => Some(Fallo::LaCuentaNoAutentica),

        // 4.x.x · cualquier cosa temporal.
        (4, _, _) => Some(Fallo::Pasajero),

        _ => None,
    }
}

/// Lee una línea de respuesta SMTP: `550 5.1.1 No such user`.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LO QUE NO SE HACE AQUÍ, Y ES DELIBERADO
///
/// **No se busca una espera en el texto.** SMTP no tiene un campo legible por
/// máquina que diga «vuelve en N segundos» — eso es del mundo HTTP, y es lo
/// que usará el adaptador de WhatsApp. Sacar el número de un texto libre es
/// tentador y peligroso: en `550 5.1.1 user 3600 not found` hay un `3600` que
/// no es una espera.
///
/// Por eso `Fallo::NoMeMandesNadaDurante` **no lo produce nunca este archivo**.
/// Existe para los proveedores que sí dan el número.
///
/// Si la línea no empieza por tres cifras, se devuelve `None`: una respuesta
/// que no se entiende no se adivina.
#[must_use]
pub fn leer_linea(linea: &str) -> Option<RespuestaSmtp> {
    let linea = linea.trim_end_matches(['\r', '\n']);
    let (cabeza, resto) = linea.split_at_checked(3)?;
    let codigo: u16 = cabeza.parse().ok()?;

    // Tras el código va un espacio —última línea— o un guion —hay más—. Si va
    // otra cosa, no es una respuesta SMTP.
    let resto = match resto.strip_prefix([' ', '-']) {
        Some(r) => r,
        None if resto.is_empty() => "",
        None => return None,
    };

    let (extendido, texto) = separar_extendido(resto);
    Some(RespuestaSmtp {
        codigo,
        extendido,
        texto: texto.to_owned(),
    })
}

/// Separa el código extendido del texto, si la primera palabra lo es.
///
/// El extendido tiene que **empezar con la misma clase** que el código básico
/// para ser creíble; aquí sólo se comprueba la forma, y quien traduce ya tiene
/// los dos. Lo que se evita es tomar por código extendido un texto que
/// casualmente empiece por números con puntos, como una versión.
fn separar_extendido(resto: &str) -> (Option<(u8, u8, u8)>, &str) {
    let Some((primera, cola)) = resto.split_once(' ') else {
        // Sin espacio: o es sólo el extendido, o es sólo texto.
        return match tres_numeros(resto) {
            Some(t) => (Some(t), ""),
            None => (None, resto),
        };
    };
    match tres_numeros(primera) {
        Some(t) => (Some(t), cola),
        None => (None, resto),
    }
}

/// `5.1.1` → `(5, 1, 1)`. Sólo si las tres partes son números y la clase es
/// 2, 4 o 5: es lo que RFC 3463 admite.
fn tres_numeros(s: &str) -> Option<(u8, u8, u8)> {
    let mut partes = s.split('.');
    let a: u8 = partes.next()?.parse().ok()?;
    let b: u8 = partes.next()?.parse().ok()?;
    let c: u8 = partes.next()?.parse().ok()?;
    if partes.next().is_some() {
        return None;
    }
    matches!(a, 2 | 4 | 5).then_some((a, b, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn respuesta(linea: &str) -> RespuestaSmtp {
        leer_linea(linea).expect("es una respuesta SMTP")
    }

    fn fallo(linea: &str) -> Option<Fallo> {
        traducir(&respuesta(linea))
    }

    // ─── La tabla del §2, fila por fila ─────────────────────────────────────

    /// **La fila que más caro cuesta equivocar.**
    ///
    /// `450` es «ahora no puedo» y `550` es «ese buzón no existe». Traducir el
    /// primero como el segundo mete en la lista de bajas a alguien que sí
    /// existe — y de ahí no se sale con un botón.
    ///
    /// Verificada rompiéndola el 23/09/2026: moviendo el `450` a la fila de los
    /// permanentes, esta prueba falla y dice exactamente qué se rompió.
    #[test]
    fn un_no_puedo_ahora_no_es_un_no_existe() {
        assert_eq!(
            fallo("450 4.2.1 Mailbox busy"),
            Some(Fallo::Pasajero),
            "un 450 es temporal: suprimir a esta persona sería irreversible \
             por algo que se arregla solo"
        );
        assert_eq!(
            fallo("550 5.1.1 No such user"),
            Some(Fallo::LaDireccionNoExiste),
            "un 550 sí es definitivo: insistir sube el contador de spam"
        );
    }

    /// Los temporales del §2, los cuatro.
    #[test]
    fn los_codigos_temporales_se_reintentan() {
        for linea in [
            "421 Service not available",
            "450 Requested action not taken",
            "451 Local error in processing",
            "452 Insufficient system storage",
        ] {
            assert_eq!(
                fallo(linea),
                Some(Fallo::Pasajero),
                "«{linea}» debería reintentarse"
            );
        }
    }

    /// Los permanentes del §2 que sí son culpa de la dirección.
    #[test]
    fn los_buzones_inexistentes_se_dan_de_baja() {
        for linea in [
            "550 No such user here",
            "551 User not local",
            "553 Mailbox name not allowed",
            "554 Transaction failed",
        ] {
            assert_eq!(
                fallo(linea),
                Some(Fallo::LaDireccionNoExiste),
                "«{linea}» debería dar de baja la dirección"
            );
        }
    }

    /// La autenticación abre el disyuntor, no toca al destinatario.
    #[test]
    fn la_credencial_que_no_sirve_habla_de_la_cuenta() {
        for linea in [
            "535 5.7.8 Username and Password not accepted",
            "530 5.7.0 Authentication required",
            "534 5.7.9 Application-specific password required",
        ] {
            assert_eq!(
                fallo(linea),
                Some(Fallo::LaCuentaNoAutentica),
                "«{linea}» es de la cuenta, no del destinatario"
            );
        }
    }

    // ─── Las cuatro filas que no son obvias ─────────────────────────────────

    /// **Un buzón lleno no da de baja a nadie.**
    ///
    /// El `552` está codificado como permanente y en la práctica es temporal:
    /// el martes que viene ese buzón tendrá sitio. Suprimir a un cliente por
    /// esto es un error irreversible por una causa que se arregla sola.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando el `552` de los
    /// temporales cae en la clase `5xx` y esta prueba falla.
    #[test]
    fn un_buzon_lleno_no_da_de_baja_a_nadie() {
        assert_eq!(
            fallo("552 Requested mail action aborted: exceeded storage allocation"),
            Some(Fallo::Pasajero),
            "el número dice permanente y la realidad dice que se arregla solo"
        );
        assert_eq!(fallo("452 4.2.2 Over quota"), Some(Fallo::Pasajero));
        assert_eq!(
            fallo("550 5.2.2 Mailbox full"),
            Some(Fallo::Pasajero),
            "ni siquiera con un 550 delante: el extendido 5.2.2 manda"
        );
    }

    /// **Un mensaje demasiado grande no es culpa del destinatario.**
    ///
    /// Esa dirección funciona perfectamente; el que está mal es nuestro
    /// mensaje. Suprimirla sería castigar a la persona equivocada y, además,
    /// esconder el problema real.
    #[test]
    fn un_mensaje_demasiado_grande_no_suprime_al_destinatario() {
        assert_eq!(
            fallo("552 5.3.4 Message size exceeds fixed maximum message size"),
            Some(Fallo::Pasajero)
        );
        assert_ne!(
            fallo("552 5.3.4 Message too big"),
            Some(Fallo::LaDireccionNoExiste),
            "la dirección está bien: el archivo adjunto es el que sobra"
        );
    }

    /// **«Relay denegado» habla de la cuenta, no de quien iba a recibir.**
    ///
    /// Es un `5xx` con pinta de rechazo del destinatario, y si se tradujera así
    /// se suprimiría a una persona alcanzable **y** se escondería el problema
    /// real, que la cuenta va a tener con todos los demás envíos.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando la fila `5.7.x`, el
    /// `550` de abajo cae en «no existe» y esta prueba falla.
    #[test]
    fn un_relay_denegado_no_suprime_a_nadie() {
        assert_eq!(
            fallo("550 5.7.1 Relaying denied"),
            Some(Fallo::LaCuentaNoAutentica),
            "el problema está en la cuenta remitente y lo tendrá con todos"
        );
        assert_eq!(
            fallo("550 5.7.0 Message rejected: sender address not authorized"),
            Some(Fallo::LaCuentaNoAutentica)
        );
    }

    /// `5.1.8` es la dirección del **remitente**, no la del destinatario.
    ///
    /// Está dentro de `5.1.x`, que es la familia de «la dirección está mal», y
    /// es la única de esa familia que habla de nosotros.
    #[test]
    fn el_remitente_mal_puesto_no_es_culpa_del_destinatario() {
        assert_eq!(
            fallo("550 5.1.8 Bad sender address syntax"),
            Some(Fallo::LaCuentaNoAutentica)
        );
        assert_eq!(
            fallo("550 5.1.1 Bad destination mailbox address"),
            Some(Fallo::LaDireccionNoExiste),
            "y el resto de la familia sí es del destinatario"
        );
    }

    // ─── Lo desconocido ─────────────────────────────────────────────────────

    /// **Un código que no se reconoce cae del lado de su clase, nunca al
    /// revés.**
    ///
    /// Un `5xx` raro tratado como temporal reintentaría para siempre; un `4xx`
    /// raro tratado como permanente suprimiría a quien no debía.
    #[test]
    fn lo_desconocido_cae_del_lado_de_su_clase() {
        assert_eq!(
            fallo("471 Something nobody has ever seen"),
            Some(Fallo::Pasajero),
            "un 4xx desconocido es temporal"
        );
        assert_eq!(
            fallo("569 Something nobody has ever seen"),
            Some(Fallo::LaDireccionNoExiste),
            "un 5xx desconocido es definitivo"
        );
    }

    /// Y lo que acepta no es un fallo.
    #[test]
    fn lo_que_el_servidor_acepta_no_es_un_fallo() {
        assert_eq!(fallo("250 2.0.0 OK: queued as 4X7v2k1Qm"), None);
        assert_eq!(fallo("220 smtp.gmail.com ESMTP ready"), None);
    }

    /// **Un `354` no es un éxito.**
    ///
    /// Es «sigue hablando, mándame el cuerpo». Tratarlo como aceptación daría
    /// por enviado un mensaje cuyo contenido no se ha mandado todavía: el
    /// intento quedaría como «aceptado» y nadie recibiría nada.
    #[test]
    fn el_adelante_mandame_el_cuerpo_no_es_una_aceptacion() {
        let r = respuesta("354 Start mail input; end with <CRLF>.<CRLF>");
        assert!(!r.acepto(), "un 354 es una invitación, no un acuse");
    }

    // ─── Leer la línea ──────────────────────────────────────────────────────

    #[test]
    fn se_leen_las_tres_partes_de_una_respuesta() {
        let r = respuesta("550 5.1.1 The email account does not exist");
        assert_eq!(r.codigo, 550);
        assert_eq!(r.extendido, Some((5, 1, 1)));
        assert_eq!(r.texto, "The email account does not exist");
    }

    /// Una respuesta multilínea lleva guion en vez de espacio.
    #[test]
    fn una_respuesta_multilinea_se_lee_igual() {
        let r = respuesta("250-smtp.gmail.com at your service");
        assert_eq!(r.codigo, 250);
        assert_eq!(r.texto, "smtp.gmail.com at your service");
    }

    /// Sin código extendido, el texto no se parte.
    #[test]
    fn sin_codigo_extendido_el_texto_queda_entero() {
        let r = respuesta("421 Service not available, closing channel");
        assert_eq!(r.extendido, None);
        assert_eq!(r.texto, "Service not available, closing channel");
    }

    /// **Un número con puntos en el texto no es un código extendido.**
    ///
    /// «Postfix 3.5.6 ready» empieza por algo con la forma de un extendido, y
    /// no lo es. Tomarlo por uno haría que la versión del servidor decidiera
    /// qué hacer con el envío.
    #[test]
    fn una_version_en_el_texto_no_se_confunde_con_un_codigo() {
        let r = respuesta("220 3.5.6 ready");
        assert_eq!(
            r.extendido, None,
            "la clase 3 no existe en RFC 3463: es texto"
        );
        assert_eq!(r.texto, "3.5.6 ready");
    }

    /// Lo que no es una respuesta SMTP **no se adivina**.
    #[test]
    fn lo_que_no_es_una_respuesta_no_se_interpreta() {
        assert!(leer_linea("").is_none());
        assert!(leer_linea("hola").is_none());
        assert!(leer_linea("5501 too long").is_none());
        assert!(
            leer_linea("250OK").is_none(),
            "tras el código va un espacio o un guion, no una letra"
        );
    }

    /// **El texto libre no decide nada.**
    ///
    /// Buscar palabras —«full», «unknown», «try again»— funciona con el
    /// servidor con el que se probó y falla con todos los demás, en idiomas que
    /// nadie leyó. Aquí se comprueba que el mismo código con textos opuestos da
    /// el mismo resultado.
    #[test]
    fn el_texto_libre_no_cambia_la_decision() {
        let a = fallo("450 mailbox temporarily unavailable");
        let b = fallo("450 usuario desconocido, no existe, jamás existió");
        assert_eq!(
            a, b,
            "el número decide; el texto se guarda y se enseña, no se interpreta"
        );
    }

    /// SMTP no tiene un campo de espera, así que aquí **nunca** sale un
    /// «espera N segundos».
    ///
    /// Sacar el número de un texto libre es tentador y peligroso: en
    /// `550 5.1.1 user 3600 not found` hay un 3600 que no es una espera. Ese
    /// fallo existe para los proveedores que sí dan el número, como Meta.
    #[test]
    fn de_smtp_nunca_sale_una_espera_concreta() {
        for linea in [
            "421 4.7.0 Try again in 3600 seconds",
            "450 please retry in 60s",
            "550 5.1.1 user 3600 not found",
        ] {
            assert!(
                !matches!(fallo(linea), Some(Fallo::NoMeMandesNadaDurante(_))),
                "«{linea}»: SMTP no da esperas legibles por máquina"
            );
        }
    }
}
