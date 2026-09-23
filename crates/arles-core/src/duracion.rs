//! Cuánto va a tardar esta campaña. La cifra más útil de todo el producto.
//!
//! `MOTOR_DE_EJECUCION.md` §11.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO IMPORTA MÁS DE LO QUE PARECE
//!
//! Alguien importa cinco mil contactos, configura cincuenta envíos diarios de
//! lunes a viernes, y pulsa activar. Sin este cálculo, se entera de que la
//! campaña dura **cuatro meses** cuando ya lleva tres semanas corriendo y el
//! lanzamiento del producto que anunciaba fue el mes pasado.
//!
//! Con este cálculo se entera **antes de activar**, que es el único momento en
//! que todavía puede decidir otra cosa: subir el límite, recortar la
//! audiencia, ampliar el horario, o partirla en dos.
//!
//! Y hay una respuesta peor que «cuatro meses», que este cálculo también da:
//! **«no termina nunca»**. Con un tope mal puesto —cero— o una audiencia
//! enorme, la campaña se quedaría enviando durante años sin que nada fallara.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ SE SIMULA HORA A HORA Y NO SE DIVIDE
//!
//! La tentación es `audiencia / limite_diario` y multiplicar por los días
//! laborables. Da un número plausible y está mal por tres motivos:
//!
//! **Hay dos topes, no uno.** Diez por hora en una ventana de nueve horas son
//! noventa, pero si el tope diario es cincuenta, se mandan cincuenta. Y al
//! revés: un tope diario de cien en una ventana de dos horas con cinco por
//! hora son diez. Manda el más pequeño, y **cuál manda cambia el consejo que
//! hay que dar**.
//!
//! **El primer día casi nunca está entero.** Quien activa a las 16:30 con
//! ventana hasta las 18:00 tiene hora y media, no nueve horas.
//!
//! **Los días no duran lo mismo.** El cambio de horario de verano hace días de
//! veintitrés y de veinticinco horas. México ya no lo aplica (D-10), pero una
//! campaña en horario de Madrid sí.
//!
//! Simular hora a hora sale gratis —cinco años son cuarenta y tres mil
//! ochocientas vueltas, milisegundos— y no hay que acordarse de ninguno de los
//! tres casos: salen solos.

use serde::Serialize;
use time::{Duration, OffsetDateTime};

use crate::ritmo::Limites;
use crate::ventana::{Ventana, clave_de_dia};

/// Hasta dónde se mira antes de decir «esto no termina».
///
/// Cinco años no es un número redondo cualquiera: es el punto a partir del
/// cual la respuesta útil deja de ser una fecha y pasa a ser «tu configuración
/// no sirve para esta audiencia». Dar «terminará el 3 de marzo de 2041» sería
/// técnicamente correcto y prácticamente inútil.
pub const HORIZONTE_DIAS: i64 = 365 * 5;

/// Cuántos días se detallan uno a uno.
///
/// Dos semanas: lo que cabe en una pantalla y lo que alguien mira de verdad.
/// El resto se resume en el total, porque una lista de mil doscientos días no
/// se lee — se cierra.
pub const DIAS_QUE_SE_DETALLAN: usize = 14;

/// Cuántos mensajes salen un día concreto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiaDeEnvio {
    /// El día en la zona de la campaña: `2026-09-23`.
    pub dia: String,
    pub cuantos: u32,
}

/// Cuál de los dos topes es el que de verdad limita.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTO ES LO QUE CONVIERTE UNA CIFRA EN UN CONSEJO
///
/// «Tu campaña tarda cuatro meses» deja al usuario sin saber qué tocar.
/// «Tu campaña tarda cuatro meses **porque el tope diario es 50**; el horario
/// daría para 90» le dice exactamente dónde está el freno.
///
/// Sin esto, lo normal es que alguien amplíe el horario de 9-18 a 8-20 y no
/// note ninguna diferencia, porque el que mandaba era el tope diario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QuienManda {
    /// El tope diario. Ampliar el horario no cambia nada.
    ElTopeDelDia,
    /// El horario, a través del tope por hora. Subir el diario no cambia nada.
    ElTopeDeLaHora,
    /// Los dos dan lo mismo: tocar cualquiera de los dos sirve.
    LosDos,
}

impl QuienManda {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::ElTopeDelDia => "duracion.mandaElTopeDelDia",
            Self::ElTopeDeLaHora => "duracion.mandaElTopeDeLaHora",
            Self::LosDos => "duracion.mandanLosDos",
        }
    }
}

/// En qué acaba la simulación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "desenlace")]
pub enum Desenlace {
    /// Termina, y se sabe cuándo.
    ///
    /// `cuando` es el **comienzo de la hora** en que sale el último mensaje.
    /// Se enseña como «alrededor del 24 de enero», nunca con minutos: la
    /// precisión que este cálculo tiene es de horas, y escribir «a las 14:37»
    /// prometería una que no existe (§94).
    Termina {
        #[serde(skip)]
        cuando: OffsetDateTime,
    },
    /// **No sale ni un mensaje.** El tope efectivo es cero.
    ///
    /// Pasa con un límite puesto a cero, y es un caso que hay que cazar
    /// **antes** de activar: si no, la campaña quedaría marcada como enviando
    /// y no enviaría nada, sin fallar y sin explicar por qué.
    NoAvanza,
    /// No cabe en [`HORIZONTE_DIAS`]. Ésa es la respuesta útil.
    NoCabeEnElHorizonte {
        /// Cuántos se quedarían sin salir en cinco años. Es una cifra (§94).
        faltan: u32,
    },
}

impl Desenlace {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::Termina { .. } => "duracion.termina",
            Self::NoAvanza => "duracion.noAvanza",
            Self::NoCabeEnElHorizonte { .. } => "duracion.noCabeEnElHorizonte",
        }
    }
}

/// Lo que se le enseña a quien va a activar una campaña.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimacion {
    pub desenlace: Desenlace,
    /// Cuántos mensajes hay que mandar.
    pub audiencia: u32,
    /// Cuántos caben en un día completo. Es el techo real, ya cruzados los dos
    /// topes con las horas de la ventana.
    pub techo_diario: u32,
    /// Cuál de los dos topes es el freno.
    pub quien_manda: QuienManda,
    /// En cuántos días distintos sale al menos un mensaje.
    pub dias_con_envio: u32,
    /// Los primeros días, uno a uno. Como mucho [`DIAS_QUE_SE_DETALLAN`].
    pub primeros_dias: Vec<DiaDeEnvio>,
}

/// Cuántos mensajes caben en un día completo, y quién pone el freno.
///
/// Las horas de la ventana son `hasta - desde`: una ventana de 9 a 18 son
/// nueve horas, porque las 18:00 ya no entran.
///
/// **El día del cambio de horario tiene una hora más o una menos**, y este
/// cálculo no lo refleja: da el techo de un día normal. Es una diferencia de
/// una hora al año en los husos que aplican horario de verano, y cero en
/// México (D-10). La simulación de abajo sí lo tiene en cuenta, porque
/// pregunta a la ventana hora por hora; esto es el techo teórico que se
/// **enseña**, y redondearlo a un día normal es lo que hace que la frase
/// «caben 50 al día» se entienda.
#[must_use]
pub fn techo_diario(limites: Limites, ventana: &Ventana) -> (u32, QuienManda) {
    let horas = u32::from(ventana.hasta.saturating_sub(ventana.desde));
    let por_el_horario = limites.por_hora.saturating_mul(horas);
    let por_el_dia = limites.por_dia;

    let quien = match por_el_horario.cmp(&por_el_dia) {
        std::cmp::Ordering::Less => QuienManda::ElTopeDeLaHora,
        std::cmp::Ordering::Greater => QuienManda::ElTopeDelDia,
        std::cmp::Ordering::Equal => QuienManda::LosDos,
    };
    (por_el_horario.min(por_el_dia), quien)
}

/// Cuándo terminará esta campaña.
///
/// `desde` es cuándo empieza a enviar: ahora, si se activa ya, o la fecha
/// programada.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA HORA DE ARRANQUE SE REDONDEA HACIA ARRIBA, Y ESO ES DELIBERADO
///
/// Quien activa a las 16:30 no tiene la hora de las 16:00 entera: le queda
/// media. Contar la hora empezada como completa haría que la estimación
/// prometiera **antes** de lo que puede cumplir, y una estimación que se queda
/// corta es peor que una que se pasa: la primera se descubre tarde y mal.
///
/// Así que la hora en curso no cuenta y se empieza en la siguiente en punto.
/// La estimación puede quedarse larga hasta en una hora. Se prefiere.
#[must_use]
pub fn estimar(
    audiencia: u32,
    limites: Limites,
    ventana: &Ventana,
    desde: OffsetDateTime,
) -> Estimacion {
    let (techo, quien_manda) = techo_diario(limites, ventana);

    let base = Estimacion {
        desenlace: Desenlace::NoAvanza,
        audiencia,
        techo_diario: techo,
        quien_manda,
        dias_con_envio: 0,
        primeros_dias: Vec::new(),
    };

    // Sin nada que mandar, termina en el momento en que empieza. No es un caso
    // raro: una audiencia puede quedarse vacía entera al aplicar la lista de
    // bajas, y entonces la respuesta correcta no es «no avanza» —que suena a
    // avería— sino «no hay nadie a quien escribir».
    if audiencia == 0 {
        return Estimacion {
            desenlace: Desenlace::Termina { cuando: desde },
            ..base
        };
    }

    // Un techo de cero se caza **aquí y no en el bucle**. En el bucle daría
    // cuarenta y tres mil vueltas sin mandar nada para acabar diciendo lo
    // mismo, y la respuesta es distinta: no es «tardaría demasiado», es «así
    // no sale ni uno».
    if techo == 0 {
        return base;
    }

    let mut quedan = audiencia;
    let mut instante = siguiente_hora_en_punto(desde);
    let limite = desde.saturating_add(Duration::days(HORIZONTE_DIAS));

    let mut dia = clave_de_dia(&ventana.zona, instante);
    let mut gastado_hoy: u32 = 0;
    let mut anotador = Anotador::nuevo();
    let mut cuando: Option<OffsetDateTime> = None;

    while quedan > 0 && instante < limite {
        let dia_de_ahora = clave_de_dia(&ventana.zona, instante);
        if dia_de_ahora != dia {
            dia = dia_de_ahora;
            gastado_hoy = 0;
        }

        if ventana.contiene(instante) {
            let cabe_hoy = limites.por_dia.saturating_sub(gastado_hoy);
            let salen = limites.por_hora.min(cabe_hoy).min(quedan);
            if salen > 0 {
                quedan -= salen;
                gastado_hoy += salen;
                anotador.anotar(&dia, salen);
                if quedan == 0 {
                    cuando = Some(instante);
                }
            }
        }

        instante = instante.saturating_add(Duration::hours(1));
    }

    Estimacion {
        desenlace: match cuando {
            Some(cuando) => Desenlace::Termina { cuando },
            None => Desenlace::NoCabeEnElHorizonte { faltan: quedan },
        },
        dias_con_envio: anotador.cuantos_dias,
        primeros_dias: anotador.dias,
        ..base
    }
}

/// La siguiente hora en punto. Si ya es en punto, la siguiente igualmente.
///
/// Que una hora exacta salte a la siguiente parece de más y no lo es: las
/// 09:00:00 clavadas sólo ocurren en las pruebas y en una campaña programada,
/// y en los dos casos perder una hora de estimación no engaña a nadie. El caso
/// contrario —conceder una hora que no existe— sí.
fn siguiente_hora_en_punto(t: OffsetDateTime) -> OffsetDateTime {
    let t = t.replace_nanosecond(0).unwrap_or(t);
    let t = t.replace_second(0).unwrap_or(t);
    let t = t.replace_minute(0).unwrap_or(t);
    t.saturating_add(Duration::hours(1))
}

/// Lo que se va anotando mientras se simula.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ EL CONTADOR DE DÍAS NO SALE DE LA LISTA
///
/// La primera versión no tenía `ultimo_dia`: miraba el último elemento de
/// `primeros_dias` para saber si el día había cambiado. Funcionaba durante
/// catorce días y **se rompía en el quince**, cuando la lista deja de crecer:
/// a partir de ahí su último elemento se queda congelado en el día catorce,
/// nunca coincide con el día en curso, y cada hora de envío se contaba como un
/// día nuevo.
///
/// El resultado que dio la prueba: una campaña de cinco mil correos a
/// cincuenta diarios decía **444 días** en vez de 100. Y no habría saltado
/// ninguna alarma — 444 es un número perfectamente plausible para una campaña
/// larga, sólo que cuatro veces mayor que el real.
///
/// El contador lleva su propia memoria del último día contado, que no depende
/// de cuántos se enseñen.
struct Anotador {
    dias: Vec<DiaDeEnvio>,
    cuantos_dias: u32,
    ultimo_dia: Option<String>,
}

impl Anotador {
    const fn nuevo() -> Self {
        Self {
            dias: Vec::new(),
            cuantos_dias: 0,
            ultimo_dia: None,
        }
    }

    /// Suma lo que sale a su día, y cuenta el día si es el primero que ve.
    fn anotar(&mut self, dia: &str, salen: u32) {
        if self.ultimo_dia.as_deref() == Some(dia) {
            // La comprobación del día **también aquí**, y no sólo en el
            // contador: pasado el día catorce la lista no crece, así que su
            // último elemento ya no es el día en curso. Sumarle sin mirar le
            // echaba encima todo lo que se enviaba después — el día catorce
            // acababa con 3 490 correos en una campaña de cincuenta diarios.
            // Lo cazó `el_desglose_cuadra_con_el_techo_diario`.
            if let Some(ultimo) = self.dias.last_mut()
                && ultimo.dia == dia
            {
                ultimo.cuantos += salen;
            }
            return;
        }
        self.ultimo_dia = Some(dia.to_owned());
        self.cuantos_dias += 1;
        if self.dias.len() < DIAS_QUE_SE_DETALLAN {
            self.dias.push(DiaDeEnvio {
                dia: dia.to_owned(),
                cuantos: salen,
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ventana::{Dias, zona};
    use time::macros::datetime;

    fn mexico() -> crate::ventana::TimeZone {
        zona("America/Mexico_City").expect("zona conocida")
    }

    /// De lunes a viernes, de 9 a 18, en Ciudad de México.
    fn laborable() -> Ventana {
        Ventana::nueva(Dias::LABORABLES, 9, 18, mexico()).expect("ventana")
    }

    /// Un miércoles a las 8:00 de México (14:00 UTC), una hora antes de abrir.
    ///
    /// **No son las 9:00 clavadas a propósito.** `estimar` redondea la hora de
    /// arranque hacia arriba —quien activa a las 16:30 no tiene la hora de las
    /// 16:00 entera—, así que empezar a las 9:00 exactas perdería la primera
    /// hora de la ventana y todas las cifras de estas pruebas saldrían
    /// descuadradas por una hora sin que se viera por qué.
    const AL_ABRIR: OffsetDateTime = datetime!(2026-09-23 14:00:00 UTC);

    fn dia_de(e: &Estimacion) -> Option<String> {
        match e.desenlace {
            Desenlace::Termina { cuando } => Some(clave_de_dia(&mexico(), cuando)),
            _ => None,
        }
    }

    // ─── El techo: cuál de los dos topes manda ──────────────────────────────

    /// **Manda el más pequeño de los dos, y se dice cuál.**
    ///
    /// Es lo que convierte la cifra en un consejo: sin saber quién frena, lo
    /// normal es ampliar el horario de 9-18 a 8-20 y no notar ninguna
    /// diferencia porque el freno era el tope diario.
    #[test]
    fn el_techo_del_dia_es_el_menor_de_los_dos_topes() {
        // 9 horas × 10 por hora = 90, pero el tope diario es 50.
        let (techo, quien) = techo_diario(Limites::nuevos(10, 50), &laborable());
        assert_eq!(techo, 50);
        assert_eq!(quien, QuienManda::ElTopeDelDia);

        // Al revés: 9 horas × 5 = 45, y el tope diario de 100 no llega a usarse.
        let (techo, quien) = techo_diario(Limites::nuevos(5, 100), &laborable());
        assert_eq!(techo, 45);
        assert_eq!(quien, QuienManda::ElTopeDeLaHora);
    }

    /// Cuando los dos dan lo mismo, se dice: tocar cualquiera sirve.
    #[test]
    fn cuando_los_dos_topes_coinciden_se_dice() {
        // 9 horas × 5 = 45, y el tope diario también 45.
        let (_, quien) = techo_diario(Limites::nuevos(5, 45), &laborable());
        assert_eq!(quien, QuienManda::LosDos);
    }

    /// **Ampliar el horario no sirve de nada si el freno es el tope diario.**
    ///
    /// Es la prueba del consejo: el mismo cálculo con una ventana el doble de
    /// larga da exactamente el mismo techo.
    #[test]
    fn ampliar_el_horario_no_cambia_nada_si_manda_el_tope_del_dia() {
        let corta = Ventana::nueva(Dias::LABORABLES, 9, 18, mexico()).expect("ventana");
        let larga = Ventana::nueva(Dias::LABORABLES, 6, 22, mexico()).expect("ventana");
        let limites = Limites::nuevos(10, 50);

        assert_eq!(
            techo_diario(limites, &corta).0,
            techo_diario(limites, &larga).0,
            "si el freno es el tope diario, alargar el horario no da un solo \
             envío más — y el usuario tiene que saberlo antes de perder el día \
             reconfigurando"
        );
    }

    // ─── La simulación ──────────────────────────────────────────────────────

    /// El caso de `MOTOR_DE_EJECUCION.md` §11, medido.
    ///
    /// Cinco mil correos, cincuenta al día, de lunes a viernes. Son cien días
    /// de envío, que a cinco por semana son veinte semanas.
    #[test]
    fn cinco_mil_correos_a_cincuenta_diarios_duran_cien_dias_habiles() {
        let e = estimar(5_000, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);

        assert_eq!(e.techo_diario, 50);
        assert_eq!(
            e.dias_con_envio, 100,
            "5 000 entre 50 son 100 días con envío"
        );
        // Cien días hábiles desde el miércoles 23/09/2026 son veinte semanas.
        assert_eq!(dia_de(&e).as_deref(), Some("2027-02-09"));
    }

    /// **Los fines de semana no cuentan, y por eso tarda lo que tarda.**
    ///
    /// Con la misma audiencia y el mismo tope, enviando todos los días se
    /// termina bastante antes. Es la comparación que hace entender el coste de
    /// la ventana.
    #[test]
    fn enviar_solo_en_laborables_alarga_la_campana() {
        let todos = Ventana::nueva(Dias::TODOS, 9, 18, mexico()).expect("ventana");
        let limites = Limites::nuevos(10, 50);

        let solo_laborables = estimar(5_000, limites, &laborable(), AL_ABRIR);
        let todos_los_dias = estimar(5_000, limites, &todos, AL_ABRIR);

        assert_eq!(solo_laborables.dias_con_envio, 100);
        assert_eq!(todos_los_dias.dias_con_envio, 100);
        let (Desenlace::Termina { cuando: a }, Desenlace::Termina { cuando: b }) =
            (solo_laborables.desenlace, todos_los_dias.desenlace)
        else {
            panic!("las dos terminan");
        };
        assert!(
            a > b,
            "los mismos 100 días de envío caen en 140 días de calendario si se \
             saltan los fines de semana"
        );
    }

    /// **El tope por hora frena de verdad**, no sólo el diario.
    ///
    /// Una ventana de dos horas con cinco por hora da diez al día, por mucho
    /// que el tope diario diga cincuenta.
    #[test]
    fn el_tope_por_hora_frena_aunque_el_diario_sea_alto() {
        let corta = Ventana::nueva(Dias::TODOS, 9, 11, mexico()).expect("ventana");
        let e = estimar(100, Limites::nuevos(5, 50), &corta, AL_ABRIR);

        assert_eq!(e.techo_diario, 10, "dos horas a cinco por hora son diez");
        assert_eq!(e.dias_con_envio, 10, "cien entre diez al día");
    }

    /// **El día en que se activa casi nunca está entero.**
    ///
    /// Activar a las 16:30 con ventana hasta las 18:00 deja una hora útil —la
    /// de 17:00—, no nueve. Contar la hora empezada como completa haría que la
    /// estimación prometiera antes de lo que puede cumplir.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando `siguiente_hora_en_punto`
    /// y empezando en `desde`, el primer día pasa a llevar veinte y esta
    /// prueba falla.
    #[test]
    fn activar_a_media_tarde_no_regala_el_dia_entero() {
        // 16:30 en México = 22:30 UTC. Quedan las horas 17 (la 16 va empezada).
        let tarde = datetime!(2026-09-23 22:30:00 UTC);
        let e = estimar(100, Limites::nuevos(10, 50), &laborable(), tarde);

        let primero = e.primeros_dias.first().expect("hay un primer día");
        assert_eq!(primero.dia, "2026-09-23");
        assert_eq!(
            primero.cuantos, 10,
            "de 16:30 a 18:00 sólo queda la hora de las 17: diez envíos, no \
             veinte ni cincuenta"
        );
    }

    /// Fuera de la ventana no sale nada, y el desglose lo enseña.
    #[test]
    fn activar_de_noche_empieza_a_la_manana_siguiente() {
        // 23:00 de México del miércoles = 05:00 UTC del jueves.
        let de_noche = datetime!(2026-09-24 05:00:00 UTC);
        let e = estimar(100, Limites::nuevos(10, 50), &laborable(), de_noche);

        assert_eq!(
            e.primeros_dias.first().expect("hay día").dia,
            "2026-09-24",
            "activar a las once de la noche del miércoles empieza el jueves"
        );
    }

    // ─── Las dos respuestas que no son una fecha ────────────────────────────

    /// **Un tope en cero no envía nada, y hay que decirlo antes de activar.**
    ///
    /// Sin este caso, la campaña quedaría marcada como enviando y no enviaría
    /// ni un mensaje, sin fallar y sin explicar por qué. Es el fallo más
    /// difícil de diagnosticar de todos: no hay nada roto que mirar.
    ///
    /// Verificada rompiéndola el 23/09/2026: quitando el atajo del techo cero,
    /// el bucle da cuarenta y tres mil vueltas y acaba en
    /// `NoCabeEnElHorizonte`, que le diría al usuario que es cuestión de
    /// tiempo cuando no lo es.
    #[test]
    fn un_tope_en_cero_dice_que_no_sale_nada() {
        let e = estimar(100, Limites::nuevos(0, 50), &laborable(), AL_ABRIR);
        assert_eq!(e.desenlace, Desenlace::NoAvanza);
        assert_eq!(e.techo_diario, 0);

        let e = estimar(100, Limites::nuevos(10, 0), &laborable(), AL_ABRIR);
        assert_eq!(e.desenlace, Desenlace::NoAvanza);
    }

    /// **«No termina en cinco años» es la respuesta útil**, no una fecha
    /// absurda.
    ///
    /// Medio millón de contactos a cincuenta diarios son diez mil días de
    /// envío: veintisiete años. Decir «terminará en 2053» sería correcto y no
    /// serviría de nada; decir cuántos se quedarían fuera, sí.
    #[test]
    fn medio_millon_a_cincuenta_diarios_no_cabe_en_cinco_anios() {
        let e = estimar(500_000, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);
        let Desenlace::NoCabeEnElHorizonte { faltan } = e.desenlace else {
            panic!("medio millón a cincuenta diarios no cabe en cinco años");
        };
        assert!(
            faltan > 400_000,
            "en cinco años de laborables caben unos 65 000, así que faltarían \
             más de 400 000: {faltan}"
        );
    }

    /// Una audiencia vacía termina al empezar. **No es una avería.**
    ///
    /// Pasa de verdad: una audiencia puede quedarse vacía entera al aplicar la
    /// lista de bajas. La respuesta correcta no es «no avanza» —que suena a
    /// que algo está roto— sino que no hay nadie a quien escribir, y de eso se
    /// encarga el preflight.
    #[test]
    fn una_audiencia_vacia_termina_al_empezar() {
        let e = estimar(0, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);
        assert_eq!(e.desenlace, Desenlace::Termina { cuando: AL_ABRIR });
        assert_eq!(e.dias_con_envio, 0);
    }

    // ─── El desglose ────────────────────────────────────────────────────────

    /// Se detallan como mucho catorce días, pero se cuentan todos.
    ///
    /// Una lista de mil doscientos días no se lee: se cierra. El total sí se
    /// dice entero, porque es la cifra que importa.
    ///
    /// **Esta prueba encontró un fallo al escribirla** (23/09/2026): el
    /// contador de días miraba el último elemento de la lista recortada, que a
    /// partir del día quince se queda congelado. Cada hora de envío contaba
    /// como un día nuevo, y esta campaña decía **444 días** en vez de 100.
    /// Nadie lo habría notado: 444 es un número plausible para una campaña
    /// larga, sólo que cuatro veces mayor que el real.
    #[test]
    fn el_desglose_se_recorta_pero_la_cuenta_no() {
        let e = estimar(5_000, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);
        assert_eq!(e.primeros_dias.len(), DIAS_QUE_SE_DETALLAN);
        assert_eq!(e.dias_con_envio, 100, "se cuentan los cien, se enseñan 14");
    }

    /// Cada día del desglose lleva su cifra, y suman lo que debe.
    ///
    /// **La otra mitad del mismo fallo** (23/09/2026): al arreglar el contador,
    /// la suma seguía cayendo sobre el último elemento de la lista aunque ya no
    /// fuera el día en curso. El día catorce acababa con 3 490 correos en una
    /// campaña de cincuenta diarios.
    #[test]
    fn el_desglose_cuadra_con_el_techo_diario() {
        let e = estimar(5_000, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);
        for d in &e.primeros_dias {
            assert_eq!(
                d.cuantos, 50,
                "el día {} debería llevar el techo entero",
                d.dia
            );
        }
    }

    /// Un día que se queda a medias lleva su cifra real, no el techo.
    #[test]
    fn el_ultimo_dia_lleva_lo_que_queda_y_no_el_techo() {
        // 120 a 50 diarios: 50, 50 y 20.
        let e = estimar(120, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);
        let cifras: Vec<u32> = e.primeros_dias.iter().map(|d| d.cuantos).collect();
        assert_eq!(cifras, vec![50, 50, 20]);
        assert_eq!(e.dias_con_envio, 3);
    }

    // ─── La zona ────────────────────────────────────────────────────────────

    /// **La misma campaña en dos husos no termina el mismo día.**
    ///
    /// Es la comprobación de que la zona de la ventana se usa de verdad (D-10)
    /// y no se cuela la del equipo por algún lado.
    #[test]
    fn dos_husos_distintos_dan_resultados_distintos() {
        let madrid = Ventana::nueva(
            Dias::LABORABLES,
            9,
            18,
            zona("Europe/Madrid").expect("zona conocida"),
        )
        .expect("ventana");

        // Las 15:00 UTC son las 9:00 en México y las 17:00 en Madrid: en
        // México acaba de abrir la ventana y en Madrid queda una hora.
        let en_mexico = estimar(100, Limites::nuevos(10, 50), &laborable(), AL_ABRIR);
        let en_madrid = estimar(100, Limites::nuevos(10, 50), &madrid, AL_ABRIR);

        assert_eq!(
            en_mexico.primeros_dias.first().expect("día").cuantos,
            50,
            "en México quedan las nueve horas de la ventana"
        );
        assert_eq!(
            en_madrid.primeros_dias.first().expect("día").cuantos,
            10,
            "en Madrid ya son las 17:00 y sólo queda una hora"
        );
    }

    /// El horizonte y el detalle son constantes, no números sueltos repartidos
    /// por el código. Si alguien cambia uno, esto dice cuál era.
    #[test]
    fn el_horizonte_son_cinco_anios_y_el_detalle_dos_semanas() {
        assert_eq!(HORIZONTE_DIAS, 1_825);
        assert_eq!(DIAS_QUE_SE_DETALLAN, 14);
    }
}
