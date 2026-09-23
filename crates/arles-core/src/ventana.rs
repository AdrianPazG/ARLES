//! La ventana de trabajo: qué días y a qué horas se envía, y **en qué reloj**.
//!
//! Ver `MOTOR_DE_EJECUCION.md` §4 y la decisión **D-10** del 22/09/2026.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA VENTANA ES DE LA CAMPAÑA, NO DE LA EMPRESA
//!
//! Dirección lo planteó con una frase que reordena el problema: **no importa a
//! qué hora lo envíe el remitente, sino a qué hora lo recibe el destinatario**.
//!
//! Una ventana de envío no existe para comodidad de quien manda: existe para
//! que el correo caiga en la bandeja de alguien a una hora en que esa persona
//! lo lea. De ahí que cada campaña lleve la suya, con su zona escrita:
//!
//!   · «Clientes México · L-V 9-17 · America/Mexico_City»
//!   · «Clientes España · L-V 9-17 · Europe/Madrid»
//!
//! Las dos pueden correr a la vez desde el mismo equipo, esté donde esté.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ UN NOMBRE DE ZONA Y NO UN DESFASE
//!
//! Guardar `-06:00` en vez de `America/Mexico_City` parece equivalente y no lo
//! es: **un desfase cambia dos veces al año** con el horario de verano y un
//! nombre no. Una ventana guardada como desfase se desplazaría una hora en
//! marzo y otra en octubre sin que nadie tocara nada, y nadie lo relacionaría
//! con el cambio de hora.
//!
//! La zona IANA va **incrustada en el binario**, no leída del sistema. Windows
//! no tiene base IANA, así que leerla del sistema dejaría a ARLES sin zonas en
//! su plataforma principal; y aun donde la hay, una base desactualizada daría
//! respuestas distintas en cada equipo. Incrustada, la misma campaña calcula
//! igual en las tres plataformas.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA BASE DE ZONAS NO ES UN DETALLE DE FONTANERÍA
//!
//! La primera implementación usaba `time-tz`, que extiende la biblioteca de
//! tiempo que el proyecto ya tenía. La tumbó la primera prueba: su base
//! **seguía aplicando horario de verano a México**, abolido en octubre de 2022.
//! Con ella, todas las campañas mexicanas habrían enviado una hora fuera de su
//! ventana medio año, en silencio. Ver la nota del `Cargo.toml` raíz.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NUNCA LA ZONA DEL SISTEMA OPERATIVO
//!
//! Es la regla que impide el fallo que Dirección quería evitar: llevarse el
//! portátil a Madrid y que la campaña mexicana empiece a disparar a las 2 de la
//! madrugada para sus clientes, **sin que nadie haya cambiado nada**.
//!
//! Este módulo no puede cometer ese fallo aunque alguien lo intente: no lee
//! ningún reloj ni ninguna zona. Recibe el instante y la zona como parámetros.

pub use jiff::tz::TimeZone;
use serde::Serialize;
use time::{OffsetDateTime, UtcOffset, Weekday};

/// Los días de la semana en los que se trabaja, como máscara de bits.
///
/// Bit 0 = lunes … bit 6 = domingo. Es lo que guarda
/// `execution_window.days_of_week`, y se conserva esa forma para que la tabla y
/// el dominio no tengan que traducirse el uno al otro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Dias(pub u8);

impl Dias {
    /// De lunes a viernes. El caso normal de una campaña de negocio.
    pub const LABORABLES: Self = Self(0b001_1111);
    /// Los siete.
    pub const TODOS: Self = Self(0b111_1111);

    /// ¿Se trabaja este día?
    #[must_use]
    pub const fn incluye(self, dia: Weekday) -> bool {
        // `number_days_from_monday` da 0 para lunes, que es el bit 0.
        self.0 & (1 << dia.number_days_from_monday()) != 0
    }

    /// ¿Hay algún día marcado?
    ///
    /// Una ventana sin días no se abre **nunca**, y una campaña con esa ventana
    /// se quedaría esperando para siempre sin decir por qué. El preflight de la
    /// entrega 4.5 la rechaza apoyándose en esto.
    #[must_use]
    pub const fn hay_alguno(self) -> bool {
        self.0 & Self::TODOS.0 != 0
    }
}

/// Cuándo se puede enviar, y en qué reloj se leen esas horas.
///
/// `desde` y `hasta` son horas enteras en la zona declarada. `hasta` es
/// **exclusiva**: `9..18` significa que a las 17:59 todavía se envía y a las
/// 18:00 ya no. Es como lo entiende cualquiera que escriba «de 9 a 18», y es lo
/// que impone la restricción `end_hour > start_hour` de la tabla.
// Sin `Copy`: la zona de `jiff` no lo es, porque por dentro puede llevar
// reglas de transición compartidas. Se clona explícitamente donde haga falta,
// que además es lo honesto — una ventana no es un entero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ventana {
    pub dias: Dias,
    pub desde: u8,
    pub hasta: u8,
    /// La zona IANA en la que se leen `desde` y `hasta`. Nunca la del sistema.
    ///
    /// Es un tipo de `jiff`, y es el ÚNICO sitio donde esa biblioteca asoma.
    /// Se guarda ya resuelta y no como texto para que una zona inventada se
    /// rechace al construir la ventana, no al primer envío de una campaña.
    pub zona: TimeZone,
}

/// Por qué una ventana no se puede usar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "motivo")]
pub enum VentanaInvalida {
    /// No hay ningún día marcado: no se abriría nunca.
    SinDias,
    /// La hora de fin no es posterior a la de inicio.
    HorasAlReves { desde: u8, hasta: u8 },
    /// Una hora fuera de 0..=24.
    HoraFueraDeRango { hora: u8 },
    /// El nombre de zona no está en la base IANA.
    ZonaDesconocida,
}

impl VentanaInvalida {
    /// La clave de texto del motivo (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::SinDias => "ventana.sinDias",
            Self::HorasAlReves { .. } => "ventana.horasAlReves",
            Self::HoraFueraDeRango { .. } => "ventana.horaFueraDeRango",
            Self::ZonaDesconocida => "ventana.zonaDesconocida",
        }
    }
}

/// Busca una zona IANA por su nombre.
///
/// # Errors
/// [`VentanaInvalida::ZonaDesconocida`] si el nombre no está en la base.
pub fn zona(nombre: &str) -> Result<TimeZone, VentanaInvalida> {
    TimeZone::get(nombre).map_err(|_| VentanaInvalida::ZonaDesconocida)
}

impl Ventana {
    /// Construye una ventana, comprobando que tiene sentido.
    ///
    /// Se valida **al construir** y no al usar, por la misma razón que
    /// `DatosDeContacto`: un tipo que sólo existe válido no se puede usar mal
    /// más abajo. Una ventana sin días o con las horas al revés no llegaría a
    /// dar error en ninguna parte — simplemente no se abriría nunca, y la
    /// campaña esperaría para siempre sin decir por qué.
    ///
    /// # Errors
    /// Devuelve qué tiene de malo, para que el preflight lo pueda decir.
    pub fn nueva(
        dias: Dias,
        desde: u8,
        hasta: u8,
        zona: TimeZone,
    ) -> Result<Self, VentanaInvalida> {
        if !dias.hay_alguno() {
            return Err(VentanaInvalida::SinDias);
        }
        for h in [desde, hasta] {
            if h > 24 {
                return Err(VentanaInvalida::HoraFueraDeRango { hora: h });
            }
        }
        if hasta <= desde {
            return Err(VentanaInvalida::HorasAlReves { desde, hasta });
        }
        Ok(Self {
            dias,
            desde,
            hasta,
            zona,
        })
    }

    /// ¿Este instante cae dentro de la ventana?
    ///
    /// El instante llega en UTC y se convierte **a la zona de la ventana**
    /// antes de mirar el día y la hora. Ese orden es todo lo que hace falta
    /// para que el cambio de horario de verano funcione solo: la conversión ya
    /// aplica el desfase que tocaba ese día concreto.
    #[must_use]
    pub fn contiene(&self, instante: OffsetDateTime) -> bool {
        let local = self.local(instante);
        self.dias.incluye(dia_de(&local)) && {
            let h = u8::try_from(local.hour()).unwrap_or(u8::MAX);
            h >= self.desde && h < self.hasta
        }
    }

    /// El instante, leído en la zona de la ventana.
    ///
    /// Aquí es donde se cruzan las dos bibliotecas de tiempo, y es el único
    /// sitio. El puente son los **segundos desde 1970**, que las dos entienden
    /// igual y que no dependen de zona ni de calendario: convertir por texto
    /// —formatear y volver a leer— habría metido un formato en medio y con él
    /// una forma más de equivocarse.
    fn local(&self, instante: OffsetDateTime) -> jiff::Zoned {
        let ts = jiff::Timestamp::from_second(instante.unix_timestamp())
            .unwrap_or(jiff::Timestamp::UNIX_EPOCH);
        ts.to_zoned(self.zona.clone())
    }

    /// El desfase que tiene la zona en ese instante.
    ///
    /// Expuesto porque es lo que permite **enseñar** por qué una campaña está
    /// esperando: «son las 22:00 en Ciudad de México (UTC-06:00)». Un «fuera de
    /// horario» sin decir de qué horario se lee como una avería (§95).
    #[must_use]
    pub fn desfase_en(&self, instante: OffsetDateTime) -> UtcOffset {
        let segundos = self.local(instante).offset().seconds();
        UtcOffset::from_whole_seconds(segundos).unwrap_or(UtcOffset::UTC)
    }
}

/// En qué hora local cae un instante: `2026-09-23T14`.
///
/// Es la etiqueta del cubo horario de una cuenta remitente. Dos envíos con la
/// misma etiqueta gastan del mismo permiso; en cuanto la etiqueta cambia, el
/// contador vuelve a cero porque **empezó otra hora**.
///
/// Se guarda como texto y no como un número de horas desde 1970 a propósito:
/// una etiqueta que se lee —`2026-09-23T14`— se puede mirar en la base de datos
/// y entender, y eso es lo que permite auditar por qué una campaña se detuvo.
#[must_use]
pub fn clave_de_hora(zona: &TimeZone, instante: OffsetDateTime) -> String {
    let z = en_zona(zona, instante);
    format!("{}T{:02}", z.date(), z.hour())
}

/// En qué día local cae un instante: `2026-09-23`.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTO ES LO QUE HACE QUE «HOY» SIGNIFIQUE ALGO
///
/// El cubo horario cambia a la misma vez en todo el mundo —las horas encajan—,
/// pero **el día no**. A las 23:00 de Ciudad de México ya es el día siguiente
/// en UTC, y un contador diario que usara UTC se reiniciaría a las 18:00 hora
/// de México: el usuario vería su límite diario renovarse a media tarde, y al
/// día siguiente se quedaría corto.
///
/// Es el caso «campaña que cruza medianoche» de `ESTRATEGIA_QA.md` §3.
///
/// ─────────────────────────────────────────────────────────────────────────
/// Y POR QUÉ LA ZONA DE LA EMPRESA, NO LA DE LA CAMPAÑA
///
/// D-10 puso el **horario de envío** en la campaña, y está bien ahí: decide a
/// qué hora recibe el destinatario.
///
/// El presupuesto es otra cosa. Cuelga de la **cuenta remitente**, y una misma
/// cuenta puede servir a varias campañas con husos distintos. Si cada una
/// midiera su propio «hoy», la misma cuenta tendría dos días en curso a la vez
/// y el límite diario dejaría de significar nada — se gastaría dos veces.
///
/// Así que el día del presupuesto es el de la empresa: uno solo, estable, y el
/// mismo para todas sus campañas.
#[must_use]
pub fn clave_de_dia(zona: &TimeZone, instante: OffsetDateTime) -> String {
    en_zona(zona, instante).date().to_string()
}

/// Un instante leído en una zona. El puente entre las dos bibliotecas de
/// tiempo, igual que en [`Ventana::local`].
fn en_zona(zona: &TimeZone, instante: OffsetDateTime) -> jiff::Zoned {
    let ts = jiff::Timestamp::from_second(instante.unix_timestamp())
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH);
    ts.to_zoned(zona.clone())
}

/// El día de la semana, traducido al vocabulario de `time`.
///
/// Las dos bibliotecas numeran la semana distinto, así que se traduce por
/// nombre y no por número: un desplazamiento de uno movería la semana entera
/// un día, y el síntoma sería «los lunes no envía y los domingos sí», que
/// nadie relacionaría con una conversión.
fn dia_de(z: &jiff::Zoned) -> Weekday {
    match z.weekday() {
        jiff::civil::Weekday::Monday => Weekday::Monday,
        jiff::civil::Weekday::Tuesday => Weekday::Tuesday,
        jiff::civil::Weekday::Wednesday => Weekday::Wednesday,
        jiff::civil::Weekday::Thursday => Weekday::Thursday,
        jiff::civil::Weekday::Friday => Weekday::Friday,
        jiff::civil::Weekday::Saturday => Weekday::Saturday,
        jiff::civil::Weekday::Sunday => Weekday::Sunday,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn mexico() -> TimeZone {
        zona("America/Mexico_City").expect("la zona de México existe")
    }
    fn madrid() -> TimeZone {
        zona("Europe/Madrid").expect("la zona de Madrid existe")
    }

    fn laborable_9_18(z: TimeZone) -> Ventana {
        Ventana::nueva(Dias::LABORABLES, 9, 18, z).expect("ventana válida")
    }

    /// El caso normal: un miércoles a media mañana en México.
    #[test]
    fn un_miercoles_a_las_diez_esta_dentro() {
        let v = laborable_9_18(mexico());
        // 2026-09-23 16:00 UTC = 10:00 en Ciudad de México (UTC-6).
        assert!(v.contiene(datetime!(2026-09-23 16:00 UTC)));
    }

    /// La hora de fin es exclusiva: a las 18:00 ya no se envía.
    ///
    /// Un `<=` en vez de `<` dejaría enviar durante la hora 18 entera, que es
    /// una hora de más cada día — y precisamente la que el usuario excluyó al
    /// escribir «hasta las 18».
    #[test]
    fn la_hora_de_fin_es_exclusiva() {
        let v = laborable_9_18(mexico());
        // 23:59 UTC = 17:59 en México: dentro.
        assert!(v.contiene(datetime!(2026-09-23 23:59 UTC)));
        // 00:00 UTC del día siguiente = 18:00 en México: fuera.
        assert!(!v.contiene(datetime!(2026-09-24 00:00 UTC)));
    }

    /// El sábado no se trabaja con `LABORABLES`.
    #[test]
    fn el_fin_de_semana_queda_fuera() {
        let v = laborable_9_18(mexico());
        // 2026-09-26 es sábado. 16:00 UTC = 10:00 en México.
        assert!(!v.contiene(datetime!(2026-09-26 16:00 UTC)));
        // Y con los siete días marcados, sí.
        let siempre = Ventana::nueva(Dias::TODOS, 9, 18, mexico()).expect("válida");
        assert!(siempre.contiene(datetime!(2026-09-26 16:00 UTC)));
    }

    // ── D-10 · la prueba que da sentido a toda la decisión ───────────────

    /// **Dos campañas, dos husos, el mismo instante.**
    ///
    /// Es la puerta de salida de D-10: una campaña mexicana y una española
    /// corriendo a la vez desde el mismo equipo. A las 16:00 UTC en México son
    /// las 10 de la mañana —se envía— y en España las 18:00 —ya no—.
    ///
    /// Si la zona se leyera del sistema operativo, las dos darían lo mismo, y
    /// ese «lo mismo» cambiaría al llevarse el portátil de viaje. Ese es
    /// exactamente el fallo que D-10 impide.
    #[test]
    fn dos_campanas_en_husos_distintos_no_se_pisan() {
        let mx = laborable_9_18(mexico());
        let es = laborable_9_18(madrid());
        let instante = datetime!(2026-09-23 16:00 UTC);

        assert!(
            mx.contiene(instante),
            "en México son las 10:00 de un miércoles: se envía"
        );
        assert!(
            !es.contiene(instante),
            "en España son las 18:00 del mismo miércoles: ya no. Si esto fallara, \
             la zona no se estaría aplicando y las dos campañas usarían el mismo reloj"
        );
    }

    /// **El cambio de horario de verano, sin hueco ni doble envío.**
    ///
    /// Es la puerta de salida declarada de la entrega 4.1.
    ///
    /// España pasa a horario de invierno el último domingo de octubre: el
    /// 25/10/2026 el desfase va de UTC+2 a UTC+1. Las 08:30 UTC son las 10:30
    /// el viernes anterior y las 09:30 el lunes siguiente — **las dos dentro**
    /// de una ventana de 9 a 18, que es lo correcto: el usuario dijo «de 9 a
    /// 18 hora de España» y eso es lo que sigue significando.
    ///
    /// El fallo que esto caza es guardar un desfase en vez de un nombre. Con
    /// `+02:00` escrito a mano, el lunes 26 las 08:30 UTC se leerían como las
    /// 10:30 cuando en España son las 9:30: la ventana entera se habría
    /// desplazado una hora sin que nadie tocara nada.
    #[test]
    fn el_cambio_de_horario_no_mueve_la_ventana() {
        let v = laborable_9_18(madrid());

        // Viernes 23/10/2026, todavía en horario de verano (UTC+2).
        let antes = datetime!(2026-10-23 08:30 UTC);
        assert_eq!(v.desfase_en(antes), UtcOffset::from_hms(2, 0, 0).unwrap());
        assert!(v.contiene(antes), "son las 10:30 en Madrid: dentro");

        // Lunes 26/10/2026, ya en horario de invierno (UTC+1).
        let despues = datetime!(2026-10-26 08:30 UTC);
        assert_eq!(v.desfase_en(despues), UtcOffset::from_hms(1, 0, 0).unwrap());
        assert!(
            v.contiene(despues),
            "son las 9:30 en Madrid: también dentro"
        );

        // Y el borde: 07:30 UTC es 09:30 en verano (dentro) y 08:30 en
        // invierno (fuera). La ventana no se movió; el reloj de España sí.
        assert!(v.contiene(datetime!(2026-10-23 07:30 UTC)));
        assert!(!v.contiene(datetime!(2026-10-26 07:30 UTC)));
    }

    /// **México no tiene horario de verano, y esto lo fija para siempre.**
    ///
    /// Es la prueba que cazó el fallo de la primera dependencia y la que impide
    /// que vuelva.
    ///
    /// México abolió el horario de verano en **octubre de 2022**. Desde
    /// entonces Ciudad de México es UTC-6 los doce meses del año.
    ///
    /// La primera base de zonas que se probó —la que incrusta `time-tz` 2.0.0—
    /// **seguía aplicando el horario abolido**: daba UTC-5 en junio y en
    /// septiembre de 2026. Con esa base, una campaña mexicana de 9 a 18 habría
    /// enviado de 10 a 19 durante medio año, en silencio, sin que ninguna
    /// prueba del producto fallara.
    ///
    /// El impacto no es teórico: México es el mercado principal de ARLES
    /// (D-4). Eso convierte una hora de desfase en el defecto más caro posible
    /// de esta entrega, porque nadie lo notaría hasta que un cliente
    /// preguntara por qué sus correos llegan a las siete de la tarde.
    ///
    /// Esta prueba no comprueba nuestro código: comprueba **la dependencia**.
    /// Es deliberado. El día que alguien suba la versión de la base de zonas,
    /// esto falla si la nueva trae las reglas viejas.
    #[test]
    fn mexico_no_tiene_horario_de_verano_en_ningun_mes() {
        let mx = mexico();
        let v = laborable_9_18(mx);
        let seis_horas = UtcOffset::from_hms(-6, 0, 0).unwrap();

        // Un día de cada mes de 2026, al mediodía UTC.
        let meses = [
            datetime!(2026-01-15 18:00 UTC),
            datetime!(2026-02-15 18:00 UTC),
            datetime!(2026-03-15 18:00 UTC),
            datetime!(2026-04-15 18:00 UTC),
            datetime!(2026-05-15 18:00 UTC),
            datetime!(2026-06-15 18:00 UTC),
            datetime!(2026-07-15 18:00 UTC),
            datetime!(2026-08-15 18:00 UTC),
            datetime!(2026-09-15 18:00 UTC),
            datetime!(2026-10-15 18:00 UTC),
            datetime!(2026-11-15 18:00 UTC),
            datetime!(2026-12-15 18:00 UTC),
        ];
        for instante in meses {
            assert_eq!(
                v.desfase_en(instante),
                seis_horas,
                "{instante} da un desfase que no es UTC-6. México abolió el \
                 horario de verano en octubre de 2022: si esto falla, la base \
                 de zonas trae reglas anteriores a ese cambio y TODAS las \
                 campañas mexicanas enviarían una hora fuera de su ventana"
            );
        }

        // Y el borde que lo hace visible: 18:00 UTC son las 12:00 en México
        // todo el año. Con el horario abolido serían las 13:00 medio año.
        for instante in meses {
            assert!(
                v.contiene(instante) || !v.dias.incluye(dia_de(&v.local(instante))),
                "{instante} debería caer dentro de la ventana en un día laborable"
            );
        }
    }

    /// Una ventana sin días no se puede construir.
    ///
    /// No se abriría nunca, y la campaña esperaría para siempre sin decir por
    /// qué. Es peor que un error, porque parece que está funcionando.
    #[test]
    fn una_ventana_sin_dias_no_se_puede_construir() {
        assert_eq!(
            Ventana::nueva(Dias(0), 9, 18, mexico()),
            Err(VentanaInvalida::SinDias)
        );
    }

    /// Las horas al revés tampoco.
    #[test]
    fn las_horas_al_reves_no_se_pueden_construir() {
        assert_eq!(
            Ventana::nueva(Dias::LABORABLES, 18, 9, mexico()),
            Err(VentanaInvalida::HorasAlReves {
                desde: 18,
                hasta: 9
            })
        );
        assert_eq!(
            Ventana::nueva(Dias::LABORABLES, 9, 9, mexico()),
            Err(VentanaInvalida::HorasAlReves { desde: 9, hasta: 9 }),
            "una ventana de cero horas tampoco sirve"
        );
    }

    /// Una hora imposible se nombra en vez de recortarse.
    #[test]
    fn una_hora_fuera_de_rango_se_nombra() {
        assert_eq!(
            Ventana::nueva(Dias::LABORABLES, 9, 25, mexico()),
            Err(VentanaInvalida::HoraFueraDeRango { hora: 25 })
        );
    }

    /// Un nombre de zona inventado no cuela.
    ///
    /// Es el mismo criterio que las zonas de la empresa en la entrega 3.1:
    /// comprobar sólo la forma dejaría entrar `America/Mexico`, que no existe,
    /// y el fallo aparecería meses después al calcular una ventana.
    #[test]
    fn una_zona_inventada_no_cuela() {
        assert_eq!(
            zona("America/Mexico"),
            Err(VentanaInvalida::ZonaDesconocida)
        );
        assert_eq!(zona("Europa/Madrid"), Err(VentanaInvalida::ZonaDesconocida));
        assert!(zona("America/Mexico_City").is_ok());
    }

    /// Los días se leen bien, de lunes a domingo.
    ///
    /// Un desplazamiento de un bit aquí movería la semana entera un día, y el
    /// síntoma sería «los lunes no envía y los domingos sí» — que nadie
    /// relacionaría con una máscara de bits.
    #[test]
    fn la_mascara_de_dias_empieza_en_lunes() {
        assert!(Dias::LABORABLES.incluye(Weekday::Monday));
        assert!(Dias::LABORABLES.incluye(Weekday::Friday));
        assert!(!Dias::LABORABLES.incluye(Weekday::Saturday));
        assert!(!Dias::LABORABLES.incluye(Weekday::Sunday));
        assert!(Dias::TODOS.incluye(Weekday::Sunday));
    }

    /// **El día del presupuesto cambia a medianoche EN MÉXICO, no en UTC.**
    ///
    /// Es el caso «campaña que cruza medianoche» de `ESTRATEGIA_QA.md` §3.
    ///
    /// A las 23:00 de Ciudad de México ya es el día siguiente en UTC. Un
    /// contador diario que usara UTC se reiniciaría a las 18:00 hora de
    /// México: el usuario vería su límite renovarse a media tarde y al día
    /// siguiente se quedaría corto, sin ninguna forma de relacionarlo con el
    /// huso horario.
    #[test]
    fn el_dia_del_presupuesto_cambia_a_medianoche_local() {
        let mx = mexico();

        // 2026-09-24 05:00 UTC = 2026-09-23 23:00 en México. Todavía es 23.
        assert_eq!(
            clave_de_dia(&mx, datetime!(2026-09-24 05:00 UTC)),
            "2026-09-23",
            "en UTC ya es día 24, pero en México son las 23:00 del 23: el \
             presupuesto diario NO puede haberse reiniciado todavía"
        );

        // 2026-09-24 06:00 UTC = 2026-09-24 00:00 en México. Ahora sí.
        assert_eq!(
            clave_de_dia(&mx, datetime!(2026-09-24 06:00 UTC)),
            "2026-09-24",
            "medianoche en México: aquí es donde empieza el día nuevo"
        );
    }

    /// La hora también se lee en la zona, y se ve a simple vista.
    #[test]
    fn la_clave_de_hora_se_lee_en_la_zona_y_es_legible() {
        let mx = mexico();
        assert_eq!(
            clave_de_hora(&mx, datetime!(2026-09-23 20:30 UTC)),
            "2026-09-23T14",
            "20:30 UTC son las 14:30 en México"
        );
        // La hora siguiente es otra etiqueta: ahí se reinicia el cubo horario.
        assert_ne!(
            clave_de_hora(&mx, datetime!(2026-09-23 20:59 UTC)),
            clave_de_hora(&mx, datetime!(2026-09-23 21:00 UTC)),
        );
    }

    /// Dos husos distintos dan días distintos en el mismo instante.
    ///
    /// Es la razón de que el presupuesto use la zona de la EMPRESA y no la de
    /// cada campaña: si cada campaña midiera su propio «hoy», una misma cuenta
    /// remitente tendría dos días en curso a la vez y el límite diario se
    /// podría gastar dos veces.
    #[test]
    fn el_mismo_instante_cae_en_dias_distintos_segun_la_zona() {
        let instante = datetime!(2026-09-24 05:00 UTC);
        assert_eq!(clave_de_dia(&mexico(), instante), "2026-09-23");
        assert_eq!(clave_de_dia(&madrid(), instante), "2026-09-24");
    }

    /// Todo motivo tiene clave de texto (§139), y ninguna se repite.
    #[test]
    fn cada_motivo_tiene_su_clave_y_ninguna_se_repite() {
        let motivos = [
            VentanaInvalida::SinDias,
            VentanaInvalida::HorasAlReves { desde: 1, hasta: 1 },
            VentanaInvalida::HoraFueraDeRango { hora: 99 },
            VentanaInvalida::ZonaDesconocida,
        ];
        let claves: std::collections::HashSet<_> = motivos.iter().map(|m| m.clave_i18n()).collect();
        assert_eq!(claves.len(), motivos.len());
        for m in motivos {
            assert!(m.clave_i18n().starts_with("ventana."), "{m:?} sin prefijo");
        }
    }
}
