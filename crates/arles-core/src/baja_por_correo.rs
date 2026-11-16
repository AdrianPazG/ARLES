//! Cuando alguien contesta pidiendo la baja (R-25).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA MITAD DE ESTO YA EXISTÍA, Y ES LO QUE HACE QUE EL MÓDULO SEA CORTO
//!
//! ARLES lleva desde el 19/10/2026 mandando dos salidas en cada campaña: la
//! cabecera `List-Unsubscribe` —que es lo que permite a Gmail y a Outlook
//! enseñar su propio botón de cancelar suscripción— y la frase del cuerpo que
//! el preflight exige, con la palabra [`PALABRA`] dentro.
//!
//! Las dos acaban en lo mismo: **un correo que llega al buzón de quien envía.**
//! Lo que faltaba no era darle una salida a nadie; era que ARLES mirara ese
//! buzón. Este módulo es la decisión de qué cuenta como una baja, y vive en el
//! núcleo porque es una regla y no una conexión: no abre nada, no lee nada, y
//! se puede ejercitar entera con cadenas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA DIRECCIÓN QUE SE DA DE BAJA ES LA DE QUIEN ESCRIBIÓ. SIEMPRE
//!
//! Y nunca una que venga escrita en el asunto o en el cuerpo. La diferencia no
//! es un matiz: si ARLES leyera la dirección del texto, cualquiera podría
//! mandar un correo con «BAJA juan@empresa.mx» y dar de baja a Juan — o, con un
//! archivo pegado, a la lista entera. Mandando el correo desde una dirección se
//! demuestra lo único que hay que demostrar: que es la suya.
//!
//! Por eso [`Veredicto`] no lleva ninguna dirección dentro. Quien llama ya
//! tiene la de `From`, y es la única que puede usar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TRES VEREDICTOS Y NO DOS, Y ES LA DECISIÓN DEL MÓDULO
//!
//! Lo cómodo sería `bool`. Y entonces habría que elegir entre dos males:
//!
//! - Dar de baja a quien dice **«no quiero darme de baja, me encanta lo que
//!   mandan»**, porque la frase lleva las palabras dentro.
//! - O exigir que el correo diga exactamente «baja» y nada más, y **perder** a
//!   quien escribe «quítame de la lista, por favor» — que es lo que escribe
//!   una persona que no está leyendo instrucciones.
//!
//! Los dos errores son caros y no son simétricos: el primero deja de escribir a
//! quien quería recibir, el segundo sigue escribiendo a quien pidió que no. Así
//! que lo inequívoco se aplica solo y **lo demás lo mira una persona**, que es
//! la misma decisión que Dirección tomó el 18/09/2026 con los choques de una
//! importación: enseñar antes de hacer algo que no tiene vuelta.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTE MÓDULO NO MIRA, DICHO EN VEZ DE DARSE POR CUBIERTO
//!
//! No mira el cuerpo entero: sólo **la primera línea que tenga algo**. Y es a
//! propósito, no por ahorrar. Quien responde a un correo arrastra el original
//! citado debajo, así que el cuerpo de una respuesta contiene la campaña
//! completa —la frase de baja incluida, que lleva la palabra dentro—. Mirando
//! el cuerpo entero, **toda respuesta a una campaña saldría dudosa**, y la
//! bandeja de lo que hay que mirar a mano sería la bandeja de todo.
//!
//! Quien escribe «BAJA» lo escribe arriba. Lo que viene después es la cita.

use crate::supresion::{MotivoDeSupresion, OrigenDeSupresion};

/// La palabra que ARLES pide, y **el único sitio donde está escrita**.
///
/// La piden tres cosas que tienen que decir lo mismo: el `subject` de la
/// cabecera `List-Unsubscribe` que arma `arles-email`, la frase del cuerpo que
/// escribe [`crate::plantilla::frase_de_baja`], y este módulo, que es quien la
/// lee de vuelta. Escrita tres veces, el día que una cambie las otras dos
/// seguirán pidiendo lo de antes y **nada fallará**: la cabecera mandaría un
/// asunto que el lector no reconoce, y la baja de quien pulse ese botón se
/// quedaría en la bandeja de lo dudoso para siempre.
///
/// En mayúsculas porque es **como se enseña**: así la escribe la frase del
/// cuerpo, así va en el asunto de la cabecera, y así la escribe de vuelta
/// quien contesta. La comparación de aquí no distingue mayúsculas.
pub const PALABRA: &str = "BAJA";

/// La misma, en minúsculas, porque con eso se compara.
///
/// Una `const` no puede llamar a `to_lowercase`, así que son dos. Es la
/// duplicación con red que el proyecto sí admite —la misma forma que
/// `AVISO_DIARIO` en Rust y en TypeScript—: lo que la sostiene no es acordarse,
/// es [`pruebas::las_dos_formas_de_la_palabra_son_la_misma_palabra`].
const PALABRA_MINUSCULA: &str = "baja";

/// Las dos palabras que, **solas**, son una baja y no admiten otra lectura.
///
/// `unsubscribe` está porque es la que escribe cualquiera en inglés y la que
/// algunos clientes de correo ponen por su cuenta, y porque la comparación es
/// **exacta**: no existe la frase donde «unsubscribe» a secas signifique otra
/// cosa. No se añaden más «por si acaso» — cada palabra de esta lista es una
/// forma de dar de baja a alguien que no lo pidió.
const SOLAS_SON_UNA_BAJA: &[&str] = &[PALABRA_MINUSCULA, "unsubscribe"];

/// Los prefijos que los clientes de correo le pegan delante a un asunto.
///
/// En español y en inglés, porque el cliente de quien contesta no tiene por qué
/// estar en el mismo idioma que la campaña.
const PREFIJOS: &[&str] = &["re:", "rv:", "fwd:", "fw:", "res:", "ref:"];

/// Lo que ARLES mira de un correo que llegó de vuelta.
///
/// **No es un correo**: es lo poco que hace falta para decidir. No hay
/// cabeceras, ni adjuntos, ni fecha, ni identificador de mensaje, porque nada
/// de eso cambia el veredicto — y lo que no entra aquí no se puede usar por
/// descuido para decidir una baja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorreoDeVuelta<'a> {
    /// El asunto, tal como venía.
    pub asunto: &'a str,
    /// El cuerpo en texto, o lo que se haya leído de él.
    ///
    /// Sólo se mira su primera línea con algo: ver la nota de la cabecera del
    /// módulo. Puede venir recortado y el veredicto no cambia.
    pub cuerpo: &'a str,
    /// ¿Lo mandó una máquina?
    ///
    /// Lo deduce quien lee el buzón, de las cabeceras que lo declaran
    /// (`Auto-Submitted` del RFC 3834 y las suyas propias de cada proveedor).
    /// Aquí basta el sí o el no, y manda sobre todo lo demás.
    pub automatico: bool,
}

/// Qué es este correo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veredicto {
    /// Pidió la baja sin lugar a dudas. Se aplica sin preguntar.
    EsUnaBaja {
        /// Por dónde lo pidió. Va a la bitácora: «pulsó el botón de su correo»
        /// y «contestó BAJA» son la misma decisión por dos caminos, y el día
        /// que haya que revisar una lista importa cuál fue.
        como: ComoLoPidio,
    },
    /// Habla de la baja, pero dentro de una frase.
    ///
    /// No se toca nada: lo mira una persona **en su propio buzón**, que es
    /// donde está el correo. ARLES puede decir de quién es y con qué asunto
    /// llegó, y con eso se encuentra.
    MejorQueLoMireAlguien,
    /// No pide ninguna baja. Es una respuesta cualquiera, o nada.
    NoEsUnaBaja,
}

/// Por dónde llegó la petición.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComoLoPidio {
    /// El asunto era la palabra y nada más: el botón de cancelar suscripción
    /// de su cliente de correo, que es lo que arma la cabecera
    /// `List-Unsubscribe`.
    ElAsunto,
    /// Contestó al correo escribiendo la palabra arriba.
    ElCuerpo,
}

impl Veredicto {
    /// ¿Hay que escribir una supresión por esto?
    ///
    /// Un `match` y no un `matches!` para que un cuarto veredicto no se cuele
    /// como un «no»: lo que hay que decidir al añadirlo es precisamente esto.
    #[must_use]
    pub const fn se_aplica_solo(self) -> bool {
        match self {
            Self::EsUnaBaja { .. } => true,
            Self::MejorQueLoMireAlguien | Self::NoEsUnaBaja => false,
        }
    }

    /// El motivo y el origen con los que entra en la lista de supresión.
    ///
    /// Están aquí juntos porque son **una sola decisión** y no dos: una baja
    /// que llegó por correo es [`MotivoDeSupresion::Baja`] —la pidió la
    /// persona, así que no se puede levantar con un botón— y
    /// [`OrigenDeSupresion::PeticionPorCorreo`], que es lo que la distingue de
    /// la que alguien apuntó a mano desde ARLES.
    ///
    /// Devolver el origen `Usuario` habría ahorrado una migración y habría
    /// sido falso: es el defecto que DOC-14 dejó escrito el 03/11/2026 —un
    /// error diagnosticado con la variante de otro—, y aquí lo que se pierde
    /// es saber quién pidió cada baja.
    #[must_use]
    pub const fn como_entra_en_la_lista(self) -> Option<(MotivoDeSupresion, OrigenDeSupresion)> {
        match self {
            Self::EsUnaBaja { .. } => Some((
                MotivoDeSupresion::Baja,
                OrigenDeSupresion::PeticionPorCorreo,
            )),
            Self::MejorQueLoMireAlguien | Self::NoEsUnaBaja => None,
        }
    }
}

/// Lee un correo de vuelta y dice qué es.
///
/// El orden de las tres preguntas no es casual y está verificado rompiéndolo:
///
/// 1. **Lo automático no es nada.** Va primero porque un «estoy de vacaciones
///    hasta el lunes» puede tener la palabra dentro —una baja médica, una baja
///    por maternidad—, y dar de baja a alguien por su mensaje de ausencia es
///    el peor error de los dos posibles: nunca lo pidió.
/// 2. **El asunto exacto.** Es el botón del cliente de correo, y es el camino
///    que ARLES pide en su propia cabecera.
/// 3. **La primera línea del cuerpo.** Es quien contesta a mano.
#[must_use]
pub fn leer(correo: &CorreoDeVuelta<'_>) -> Veredicto {
    if correo.automatico {
        return Veredicto::NoEsUnaBaja;
    }

    let asunto = sin_prefijos(&normalizar(correo.asunto));
    if SOLAS_SON_UNA_BAJA.contains(&asunto.as_str()) {
        return Veredicto::EsUnaBaja {
            como: ComoLoPidio::ElAsunto,
        };
    }

    let Some(linea) = primera_linea_con_algo(correo.cuerpo) else {
        // Sin cuerpo que mirar, el asunto ya dijo lo que tenía que decir: si
        // además lo nombra, que lo mire alguien.
        return si_lo_nombra(&asunto);
    };

    let linea = normalizar(linea);
    if SOLAS_SON_UNA_BAJA.contains(&solo_letras_y_numeros(&linea).as_str()) {
        return Veredicto::EsUnaBaja {
            como: ComoLoPidio::ElCuerpo,
        };
    }

    // Ni el asunto ni la primera línea son la palabra sola. Si alguno la
    // nombra, hay algo que mirar; si no, es una respuesta cualquiera.
    match si_lo_nombra(&linea) {
        Veredicto::MejorQueLoMireAlguien => Veredicto::MejorQueLoMireAlguien,
        _ => si_lo_nombra(&asunto),
    }
}

/// ¿Este texto nombra la baja en alguna parte?
fn si_lo_nombra(texto: &str) -> Veredicto {
    if SOLAS_SON_UNA_BAJA.iter().any(|p| texto.contains(p)) {
        Veredicto::MejorQueLoMireAlguien
    } else {
        Veredicto::NoEsUnaBaja
    }
}

/// Minúsculas y sin los espacios de los bordes.
///
/// `to_lowercase` de Rust y no el `lower()` de SQLite, que **sólo baja ASCII**:
/// es el defecto que R-26 encontró el 03/11/2026 buscando a «TÉLLEZ». Aquí las
/// dos palabras que se comparan no llevan acentos, así que hoy no cambia nada
/// — y el día que alguna los lleve, cambiará sin que nadie se acuerde de esto.
fn normalizar(texto: &str) -> String {
    texto.trim().to_lowercase()
}

/// Quita los `Re:` y `Fwd:` que se van acumulando, sean los que sean.
///
/// En bucle porque se apilan: un asunto reenviado dos veces llega como
/// `Fwd: Re: Baja`, y mirar sólo el primero dejaría fuera el resto.
fn sin_prefijos(asunto: &str) -> String {
    let mut queda = asunto.trim();
    'otra_vez: loop {
        for prefijo in PREFIJOS {
            if let Some(resto) = queda.strip_prefix(prefijo) {
                queda = resto.trim_start();
                continue 'otra_vez;
            }
        }
        return queda.to_owned();
    }
}

/// La primera línea que tenga algo más que espacios, si hay alguna.
fn primera_linea_con_algo(cuerpo: &str) -> Option<&str> {
    cuerpo.lines().map(str::trim).find(|l| !l.is_empty())
}

/// Sólo las letras y los números, para comparar sin puntuación.
///
/// Lo que entra es «BAJA.», «baja!», «— baja —», y las tres son la palabra
/// sola. Lo que NO se puede hacer es quitar los espacios **y** dar por buena
/// la coincidencia: «de baja» quedaría en «debaja», que no es la palabra, y es
/// lo correcto — esa frase la mira una persona.
fn solo_letras_y_numeros(texto: &str) -> String {
    texto.chars().filter(|c| c.is_alphanumeric()).collect()
}

// ═════════════════════════════════════════════════════════════════════════
// CADA CUÁNTO SE MIRA EL BUZÓN (R-30, 11/11/2026)
// ═════════════════════════════════════════════════════════════════════════
//
// Esto vive aquí y no en un módulo propio porque es la otra mitad de la misma
// decisión de producto: D-21 dijo que las bajas se aplican solas, y «solas»
// necesita un ritmo. Quien venga a leer qué cuenta como una baja es quien se
// pregunta cada cuánto se mira, y separarlos dejaría la segunda respuesta en
// un archivo que nadie abriría.
//
// Y vive en el núcleo por la razón de siempre: es una regla. No abre nada, no
// mira el reloj —el reloj entra como parámetro— y se ejercita entera con
// fechas escritas a mano.

/// Cada cuántos minutos se mira el buzón.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ QUINCE, CON SUS DOS CIFRAS
///
/// Lo que se arriesga esperando es que alguien que pidió la baja reciba un
/// correo más. A 300 envíos diarios —el techo que D-13 permite por campaña— en
/// una ventana de ocho horas salen unos 37 por hora, así que un cuarto de hora
/// son **unos nueve correos** en el peor caso, y sólo si la baja llegó justo
/// después de una revisión.
///
/// Lo que se arriesga mirando más a menudo es la cuenta del usuario. Cada
/// revisión es una conexión IMAP con su autenticación, y los proveedores
/// cuentan los intentos: mirar cada minuto son 1 440 al día contra 96. Ninguno
/// de los cinco proveedores que ARLES reconoce publica ese umbral, así que lo
/// prudente no es acercarse a él.
///
/// **No es configurable**, y es deliberado: un ajuste más en una pantalla que
/// ya tiene casilla y servidor, para elegir entre dos números que casi nadie
/// puede razonar, es complejidad que se paga sin comprar nada. El botón de
/// «Revisar el buzón ahora» sigue ahí para quien no quiera esperar, y eso cubre
/// el caso de verdad —«acabo de pedir la baja, compruébalo»— mejor que un
/// desplegable.
pub const REVISAR_CADA_MINUTOS: u16 = 15;

/// ¿Toca mirar este buzón?
///
/// `ultimo_intento` es **cuándo se intentó**, con éxito o sin él, y no hasta
/// dónde se leyó. La diferencia está escrita en la cabecera de la migración
/// V19: si esto recibiera la marca del éxito, un servidor caído se reintentaría
/// en cada vuelta del programador.
///
/// `None` es «nunca se ha intentado» y **sí toca**: es lo que hace que encender
/// la casilla se note enseguida en vez de dentro de un cuarto de hora.
///
/// Una marca **en el futuro** —el reloj del equipo se movió— cuenta como recién
/// revisada y por tanto *no* toca. Es el mismo lado por el que falla
/// `arles_motor::instancia::edad_del_latido` ante una fecha rara: esperar de
/// más no rompe nada, y mirar de más sería conectarse a un servidor ajeno en
/// cada vuelta hasta que alguien arreglara el reloj.
#[must_use]
pub fn toca_revisar(
    ultimo_intento: Option<time::OffsetDateTime>,
    ahora: time::OffsetDateTime,
) -> bool {
    let Some(entonces) = ultimo_intento else {
        return true;
    };
    // `checked_sub` no hace falta: `ahora - entonces` sobre `OffsetDateTime` da
    // una `Duration` con signo, y una negativa es la fecha en el futuro.
    let pasado = ahora - entonces;
    pasado >= time::Duration::minutes(i64::from(REVISAR_CADA_MINUTOS))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Lo que llega cuando alguien pulsa el botón de su cliente de correo.
    fn del_boton(asunto: &str) -> CorreoDeVuelta<'_> {
        CorreoDeVuelta {
            asunto,
            cuerpo: "",
            automatico: false,
        }
    }

    /// Lo que llega cuando alguien contesta a una campaña.
    fn contestando(cuerpo: &str) -> CorreoDeVuelta<'_> {
        CorreoDeVuelta {
            asunto: "Re: Oferta de noviembre",
            cuerpo,
            automatico: false,
        }
    }

    #[test]
    fn el_boton_del_cliente_de_correo_es_una_baja() {
        // El asunto exacto que arma la cabecera `List-Unsubscribe`.
        assert_eq!(
            leer(&del_boton("Baja")),
            Veredicto::EsUnaBaja {
                como: ComoLoPidio::ElAsunto
            }
        );
        // Y como quiera que lo escriba o lo reenvíe su cliente.
        for asunto in ["baja", "BAJA", "  Baja  ", "Re: Baja", "Fwd: Re: baja"] {
            assert!(
                leer(&del_boton(asunto)).se_aplica_solo(),
                "«{asunto}» tenía que ser una baja"
            );
        }
    }

    #[test]
    fn unsubscribe_sola_tambien() {
        assert!(leer(&del_boton("Unsubscribe")).se_aplica_solo());
        assert!(leer(&contestando("unsubscribe")).se_aplica_solo());
    }

    #[test]
    fn contestar_la_palabra_arriba_es_una_baja() {
        for cuerpo in ["BAJA", "baja", "Baja.", "  baja  ", "¡BAJA!"] {
            assert_eq!(
                leer(&contestando(cuerpo)),
                Veredicto::EsUnaBaja {
                    como: ComoLoPidio::ElCuerpo
                },
                "«{cuerpo}» tenía que ser una baja"
            );
        }
    }

    /// ── EL CASO QUE JUSTIFICA QUE HAYA TRES VEREDICTOS Y NO DOS ───────────
    ///
    /// Con un `bool`, estas dos frases dan el mismo resultado y una de las dos
    /// está mal: la primera pide la baja con sus palabras y la segunda dice lo
    /// contrario. Ninguna se puede decidir sin leerla.
    #[test]
    fn una_frase_con_la_palabra_dentro_la_mira_una_persona() {
        for cuerpo in [
            "Quítame de la lista, no quiero más correos. Baja por favor",
            "No quiero darme de baja, me encanta lo que mandan",
            "Estoy de baja hasta el lunes, lo veo a la vuelta",
        ] {
            assert_eq!(
                leer(&contestando(cuerpo)),
                Veredicto::MejorQueLoMireAlguien,
                "«{cuerpo}» tenía que ir a la bandeja de lo dudoso"
            );
        }
    }

    #[test]
    fn una_respuesta_cualquiera_no_es_nada() {
        for cuerpo in [
            "Gracias, me interesa. ¿Me llamas mañana?",
            "¿Tienen catálogo en PDF?",
            "",
        ] {
            assert_eq!(leer(&contestando(cuerpo)), Veredicto::NoEsUnaBaja);
        }
    }

    /// ── LO AUTOMÁTICO VA PRIMERO, Y SE VE AQUÍ ────────────────────────────
    ///
    /// Un «fuera de la oficina» con la palabra dentro es el peor error
    /// posible: nadie pidió nada. Y el caso no es raro en español, donde una
    /// ausencia se anuncia con esa misma palabra.
    #[test]
    fn un_aviso_automatico_nunca_da_de_baja_a_nadie() {
        let vacaciones = CorreoDeVuelta {
            asunto: "Automatic reply: Oferta de noviembre",
            cuerpo: "Estoy de baja por maternidad hasta marzo.",
            automatico: true,
        };
        assert_eq!(leer(&vacaciones), Veredicto::NoEsUnaBaja);

        // Y manda incluso sobre el camino inequívoco: un robot que conteste
        // con el asunto exacto tampoco da de baja a nadie.
        let robot = CorreoDeVuelta {
            asunto: "Baja",
            cuerpo: "",
            automatico: true,
        };
        assert_eq!(leer(&robot), Veredicto::NoEsUnaBaja);
    }

    /// ── LA CITA DEL CORREO ORIGINAL NO CUENTA ─────────────────────────────
    ///
    /// Es el caso que decidió mirar **sólo la primera línea**. Una respuesta
    /// arrastra la campaña entera citada debajo, y la campaña lleva la frase de
    /// baja dentro — con la palabra. Mirando el cuerpo entero, toda respuesta
    /// saldría dudosa y la bandeja de lo que hay que mirar sería la de todo.
    #[test]
    fn la_campana_citada_debajo_no_convierte_una_respuesta_en_dudosa() {
        let con_cita = contestando(
            "Gracias, me interesa.\n\
             \n\
             El 10 de noviembre escribió:\n\
             > Oferta de noviembre\n\
             > Si no quieres recibir más correos nuestros, responde a este\n\
             > mensaje con la palabra BAJA y te quitamos de la lista.\n",
        );
        assert_eq!(leer(&con_cita), Veredicto::NoEsUnaBaja);
    }

    #[test]
    fn la_palabra_arriba_y_la_cita_debajo_si_es_una_baja() {
        let con_cita = contestando(
            "BAJA\n\
             \n\
             El 10 de noviembre escribió:\n\
             > Oferta de noviembre\n",
        );
        assert_eq!(
            leer(&con_cita),
            Veredicto::EsUnaBaja {
                como: ComoLoPidio::ElCuerpo
            }
        );
    }

    /// Las líneas en blanco de arriba las pone el cliente de correo, no quien
    /// escribe.
    #[test]
    fn las_lineas_vacias_de_arriba_no_tapan_la_palabra() {
        assert!(leer(&contestando("\n\n   \nBAJA\n")).se_aplica_solo());
    }

    /// ── LA DIRECCIÓN DEL TEXTO NO SE USA PARA NADA ────────────────────────
    ///
    /// No hay forma de comprobarlo con un `assert` sobre el veredicto, porque
    /// el veredicto **no lleva ninguna dirección**: eso es la garantía, y es
    /// del tipo. Lo que sí se puede comprobar es que pedir la baja de otro no
    /// se aplique solo.
    #[test]
    fn pedir_la_baja_de_otra_persona_no_se_aplica_solo() {
        let ajena = contestando("BAJA juan@empresa.mx");
        assert_eq!(leer(&ajena), Veredicto::MejorQueLoMireAlguien);
        assert!(!leer(&ajena).se_aplica_solo());
    }

    #[test]
    fn una_baja_entra_como_baja_y_no_se_puede_levantar() {
        let (motivo, origen) = leer(&del_boton("Baja"))
            .como_entra_en_la_lista()
            .expect("una baja tiene que entrar en la lista");
        assert_eq!(motivo, MotivoDeSupresion::Baja);
        assert_eq!(origen, OrigenDeSupresion::PeticionPorCorreo);
        // La pidió la persona, así que no sale con un botón.
        assert!(!motivo.se_puede_levantar());
        assert!(motivo.lo_pidio_la_persona());
    }

    #[test]
    fn lo_que_no_es_una_baja_no_entra_en_la_lista() {
        assert!(
            leer(&contestando("Gracias"))
                .como_entra_en_la_lista()
                .is_none()
        );
        assert!(
            leer(&contestando("no quiero darme de baja"))
                .como_entra_en_la_lista()
                .is_none()
        );
    }

    /// ── LA PALABRA ESTÁ EN UN SOLO SITIO, Y ESTO LO COMPRUEBA ─────────────
    ///
    /// La frase que lee quien recibe el correo y la palabra que reconoce el
    /// lector tienen que ser la misma. Están las dos en el núcleo, así que esta
    /// prueba las ve; la tercera copia —el `subject` de la cabecera, que vive
    /// en `arles-email`— la comprueba una prueba de ese crate, que es la única
    /// que ve las dos cosas.
    #[test]
    fn la_frase_del_correo_pide_la_palabra_que_este_modulo_reconoce() {
        let frase = crate::plantilla::frase_de_baja("enlace@empresa.mx");
        assert!(
            frase.contains(PALABRA),
            "la frase de baja no pide «{PALABRA}»: {frase}"
        );
    }

    /// La red de las dos copias de la palabra. Ver la nota de
    /// [`PALABRA_MINUSCULA`].
    #[test]
    fn las_dos_formas_de_la_palabra_son_la_misma_palabra() {
        assert_eq!(PALABRA.to_lowercase(), PALABRA_MINUSCULA);
        // Y la que se compara tiene que estar en la lista de las inequívocas,
        // o el botón del cliente de correo no daría de baja a nadie.
        assert!(SOLAS_SON_UNA_BAJA.contains(&PALABRA_MINUSCULA));
    }

    /// Un instante fijo, para que la prueba no dependa de cuándo se corra.
    fn un_martes() -> time::OffsetDateTime {
        time::OffsetDateTime::from_unix_timestamp(1_762_800_000).expect("fecha válida")
    }

    /// **Nunca revisado toca enseguida**, y es lo que hace que encender la
    /// casilla se note.
    ///
    /// Sin esto, el primer arranque esperaría el intervalo entero y quien
    /// acabara de encenderlo pensaría que no funciona.
    #[test]
    fn un_buzon_que_nunca_se_reviso_toca_ya() {
        assert!(toca_revisar(None, un_martes()));
    }

    /// Antes del intervalo no toca; cumplido, sí. **El límite exacto cuenta
    /// como cumplido**, que es el único caso que distingue `>=` de `>`.
    #[test]
    fn el_intervalo_se_respeta_y_el_limite_exacto_toca() {
        let ahora = un_martes();
        let cada = i64::from(REVISAR_CADA_MINUTOS);

        let recien = ahora - time::Duration::minutes(cada - 1);
        assert!(
            !toca_revisar(Some(recien), ahora),
            "a un minuto de cumplirse el intervalo no toca: mirar antes es \
             una conexión IMAP más contra la cuenta del usuario"
        );

        let justo = ahora - time::Duration::minutes(cada);
        assert!(
            toca_revisar(Some(justo), ahora),
            "cumplido el intervalo exacto toca: con `>` en vez de `>=` habría \
             que esperar un minuto más del declarado"
        );

        let viejo = ahora - time::Duration::hours(9);
        assert!(toca_revisar(Some(viejo), ahora));
    }

    /// **Una marca en el futuro no hace mirar**: el reloj del equipo se movió.
    ///
    /// Es el mismo lado por el que falla el latido del cerrojo ante una fecha
    /// rara. Lo contrario —darla por caducada— conectaría con un servidor ajeno
    /// en cada vuelta del programador hasta que alguien arreglara el reloj, que
    /// es justo lo que [`REVISAR_CADA_MINUTOS`] existe para no hacer.
    ///
    /// Verificada rompiéndola: con el cálculo en valor absoluto, esta prueba
    /// falla y las otras tres siguen pasando.
    #[test]
    fn un_reloj_movido_no_dispara_una_revision() {
        let ahora = un_martes();
        let futuro = ahora + time::Duration::hours(3);
        assert!(
            !toca_revisar(Some(futuro), ahora),
            "ante una fecha rara, esperar es el lado que no rompe nada"
        );
    }
}
