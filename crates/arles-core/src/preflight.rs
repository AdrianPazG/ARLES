//! Las ocho comprobaciones de antes de activar.
//!
//! `MOTOR_DE_EJECUCION.md` §12, §63 del brief.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE ESTO EVITA
//!
//! Activar una campaña es, en la práctica, irreversible: la audiencia se
//! congela, los mensajes empiezan a salir, y lo que salió no vuelve. Los
//! fallos que este archivo caza son todos del mismo tipo — cosas que **no
//! fallan al activar**, sino tres días después:
//!
//! · La contraseña de la cuenta caducó → la campaña se activa y no sale nada.
//! · La audiencia se quedó vacía al aplicar las bajas → igual.
//! · Una variable de la plantilla no se resuelve para 340 contactos → salen
//!   340 correos que dicen «Hola {{nombre}}».
//! · El horario configurado no alcanza → la campaña tarda seis años.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA TERCERA RESPUESTA, Y ES LA IMPORTANTE
//!
//! Un punto no tiene dos respuestas sino **tres**: pasa, falla, y **no se ha
//! comprobado**.
//!
//! La tercera existe porque varias de estas comprobaciones necesitan piezas
//! que llegan después: los adaptadores de proveedor son de la Fase 5, las
//! plantillas de la Fase 6. Un preflight que devolviera «pasa» para lo que
//! todavía no sabe mirar sería peor que no tenerlo: daría por bueno
//! exactamente lo que no ha visto.
//!
//! Es la misma regla que el validador de fases aplica a las sondas: **omitida
//! no es pasada, y se dice**. Y de ahí sale la consecuencia que gobierna este
//! archivo: **un punto bloqueante sin comprobar impide activar**, igual que
//! uno que falla. La diferencia se enseña —el mensaje no es el mismo— pero el
//! botón está igual de apagado.

use serde::Serialize;

use crate::duracion::{Desenlace, Estimacion};

/// Qué se comprueba antes de activar.
///
/// Son los ocho del §63, en el orden en que se enseñan: lo que más
/// probablemente falle, primero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Punto {
    /// La cuenta remitente autentica contra su proveedor.
    LaCuentaAutentica,
    /// La credencial no ha caducado ni ha sido revocada.
    LaCredencialSirve,
    /// Queda alguien a quien escribir **después** de aplicar las bajas.
    LaAudienciaNoEstaVacia,
    /// Todas las variables de la plantilla se resuelven para todos.
    LasVariablesSeResuelven,
    /// El horario configurado alcanza para terminar.
    ElHorarioAlcanza,
    /// Hay conexión.
    HayConexion,
    /// Se hizo el envío de prueba (§45).
    SeHizoElEnvioDePrueba,
    /// SPF, DKIM y DMARC del dominio.
    ///
    /// **Este no bloquea**, y es el único que no lo hace. Un dominio sin DMARC
    /// entrega peor, pero entrega; bloquear por esto dejaría fuera a quien no
    /// controla su DNS —que en una empresa pequeña es casi todo el mundo— por
    /// algo que ARLES no puede arreglarle. §154: informar, no dictar.
    LosRegistrosDelDominio,
}

impl Punto {
    /// Los ocho, en orden de pantalla.
    pub const TODOS: &'static [Self] = &[
        Self::LaCuentaAutentica,
        Self::LaCredencialSirve,
        Self::LaAudienciaNoEstaVacia,
        Self::LasVariablesSeResuelven,
        Self::ElHorarioAlcanza,
        Self::HayConexion,
        Self::SeHizoElEnvioDePrueba,
        Self::LosRegistrosDelDominio,
    ];

    /// ¿Este punto impide activar?
    ///
    /// Siete de los ocho sí. El de los registros del dominio es una
    /// advertencia: se enseña y no cierra la puerta.
    #[must_use]
    pub const fn bloquea(self) -> bool {
        !matches!(self, Self::LosRegistrosDelDominio)
    }

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::LaCuentaAutentica => "preflight.laCuentaAutentica",
            Self::LaCredencialSirve => "preflight.laCredencialSirve",
            Self::LaAudienciaNoEstaVacia => "preflight.laAudienciaNoEstaVacia",
            Self::LasVariablesSeResuelven => "preflight.lasVariablesSeResuelven",
            Self::ElHorarioAlcanza => "preflight.elHorarioAlcanza",
            Self::HayConexion => "preflight.hayConexion",
            Self::SeHizoElEnvioDePrueba => "preflight.seHizoElEnvioDePrueba",
            Self::LosRegistrosDelDominio => "preflight.losRegistrosDelDominio",
        }
    }
}

/// Cómo salió un punto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Veredicto {
    Pasa,
    Falla,
    /// **No se ha podido mirar.** No es un aprobado.
    ///
    /// Puede ser porque la pieza que lo comprueba todavía no existe —los
    /// adaptadores de proveedor son de la Fase 5—, porque no hay conexión para
    /// preguntarlo, o porque el usuario no ha llegado a ese paso.
    SinComprobar,
}

impl Veredicto {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Pasa => "preflight.pasa",
            Self::Falla => "preflight.falla",
            Self::SinComprobar => "preflight.sinComprobar",
        }
    }

    /// Lo que se sabe de un `Option<bool>`: `None` es «no se miró».
    #[must_use]
    pub const fn de(observado: Option<bool>) -> Self {
        match observado {
            Some(true) => Self::Pasa,
            Some(false) => Self::Falla,
            None => Self::SinComprobar,
        }
    }
}

/// Lo que se ha podido averiguar sobre esta campaña.
///
/// ─────────────────────────────────────────────────────────────────────────
/// TODO ES `Option` MENOS LO QUE SIEMPRE SE SABE
///
/// Quien llama rellena lo que puede y deja en `None` lo que no. **No se
/// inventa nada**: un `None` se convierte en «sin comprobar» y bloquea igual
/// que un fallo, así que mentir aquí para que el preflight pase sería mentirle
/// al usuario sobre su propia campaña.
///
/// El tamaño de la audiencia y la estimación de duración no son opcionales:
/// se calculan con lo que ya hay en la base, sin hablar con nadie de fuera.
#[derive(Debug, Clone)]
pub struct Observaciones<'a> {
    /// ¿La cuenta remitente autentica? `None` hasta la Fase 5.
    pub cuenta_autentica: Option<bool>,
    /// ¿La credencial sigue viva? `None` hasta la Fase 5.
    pub credencial_sirve: Option<bool>,
    /// Cuánta gente queda **después** de aplicar las bajas.
    pub audiencia: u32,
    /// Cuántos contactos tienen alguna variable sin resolver. `None` hasta la
    /// Fase 6, que es cuando hay plantillas.
    pub contactos_con_variables_sin_resolver: Option<u32>,
    /// La estimación de duración. De aquí sale si el horario alcanza.
    pub estimacion: &'a Estimacion,
    /// ¿Hay conexión? `None` si no se llegó a mirar.
    pub hay_conexion: Option<bool>,
    /// ¿Se hizo el envío de prueba (§45)? `None` si no consta.
    pub se_hizo_el_envio_de_prueba: Option<bool>,
    /// ¿SPF/DKIM/DMARC en orden? `None` si no se pudo consultar el DNS.
    pub registros_del_dominio: Option<bool>,
}

/// Un punto con su veredicto, listo para enseñar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PuntoRevisado {
    pub punto: Punto,
    pub veredicto: Veredicto,
}

/// Si se puede activar o no, y por qué no.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "resultado")]
pub enum Resultado {
    /// Se puede activar. `avisos` cuenta lo que no bloquea pero conviene leer.
    Lista { avisos: u32 },
    /// No se puede activar. **Cifras, no adjetivos** (§94).
    Bloqueada {
        /// Puntos que fallaron de verdad.
        fallan: u32,
        /// Puntos que **no se pudieron mirar**. Bloquean igual, y se cuentan
        /// aparte porque lo que hay que hacer con ellos es distinto: uno que
        /// falla se arregla, uno sin comprobar se comprueba.
        sin_comprobar: u32,
    },
}

impl Resultado {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::Lista { .. } => "preflight.lista",
            Self::Bloqueada { .. } => "preflight.bloqueada",
        }
    }

    /// ¿Se puede pulsar activar?
    #[must_use]
    pub const fn se_puede_activar(&self) -> bool {
        matches!(self, Self::Lista { .. })
    }
}

/// El preflight completo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preflight {
    pub puntos: Vec<PuntoRevisado>,
    pub resultado: Resultado,
}

/// Revisa los ocho puntos.
///
/// ─────────────────────────────────────────────────────────────────────────
/// SE REVISAN LOS OCHO, SIEMPRE, Y NO SE PARA EN EL PRIMERO QUE FALLA
///
/// Parar en el primero ahorraría trabajo y haría perder tiempo al usuario:
/// arregla la credencial, vuelve, y descubre que además la audiencia está
/// vacía. Tres viajes para tres problemas que se podían enseñar juntos.
///
/// Aquí no hay nada caro que ahorrar —las observaciones ya vienen hechas—, así
/// que se enseñan los ocho de una vez.
#[must_use]
pub fn revisar(obs: &Observaciones<'_>) -> Preflight {
    let puntos: Vec<PuntoRevisado> = Punto::TODOS
        .iter()
        .copied()
        .map(|punto| PuntoRevisado {
            punto,
            veredicto: veredicto_de(punto, obs),
        })
        .collect();

    let mut fallan = 0;
    let mut sin_comprobar = 0;
    let mut avisos = 0;
    for p in &puntos {
        match (p.punto.bloquea(), p.veredicto) {
            (true, Veredicto::Falla) => fallan += 1,
            (true, Veredicto::SinComprobar) => sin_comprobar += 1,
            // Lo que no bloquea, cuando no pasa, es un aviso que se lee.
            (false, Veredicto::Falla | Veredicto::SinComprobar) => avisos += 1,
            _ => {}
        }
    }

    let resultado = if fallan == 0 && sin_comprobar == 0 {
        Resultado::Lista { avisos }
    } else {
        Resultado::Bloqueada {
            fallan,
            sin_comprobar,
        }
    };

    Preflight { puntos, resultado }
}

/// Cómo sale un punto concreto.
fn veredicto_de(punto: Punto, obs: &Observaciones<'_>) -> Veredicto {
    match punto {
        Punto::LaCuentaAutentica => Veredicto::de(obs.cuenta_autentica),
        Punto::LaCredencialSirve => Veredicto::de(obs.credencial_sirve),

        // La audiencia siempre se sabe: sale de contar filas en la base.
        Punto::LaAudienciaNoEstaVacia => Veredicto::de(Some(obs.audiencia > 0)),

        // Cero contactos con variables sin resolver es pasar. Uno solo ya
        // falla: un correo que dice «Hola {{nombre}}» no se puede retirar.
        Punto::LasVariablesSeResuelven => {
            Veredicto::de(obs.contactos_con_variables_sin_resolver.map(|n| n == 0))
        }

        // Aquí es donde el simulador de duración deja de ser informativo y se
        // convierte en una puerta: si el cálculo dice que no sale nada, o que
        // no cabe en cinco años, el horario **no alcanza** y activar sería
        // dejar una campaña colgada sin que nada falle.
        Punto::ElHorarioAlcanza => Veredicto::de(Some(matches!(
            obs.estimacion.desenlace,
            Desenlace::Termina { .. }
        ))),

        Punto::HayConexion => Veredicto::de(obs.hay_conexion),
        Punto::SeHizoElEnvioDePrueba => Veredicto::de(obs.se_hizo_el_envio_de_prueba),
        Punto::LosRegistrosDelDominio => Veredicto::de(obs.registros_del_dominio),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::duracion::{QuienManda, estimar};
    use crate::ritmo::Limites;
    use crate::ventana::{Dias, Ventana, zona};
    use time::macros::datetime;

    const AL_ABRIR: time::OffsetDateTime = datetime!(2026-09-23 14:00:00 UTC);

    fn ventana() -> Ventana {
        Ventana::nueva(
            Dias::LABORABLES,
            9,
            18,
            zona("America/Mexico_City").expect("zona conocida"),
        )
        .expect("ventana")
    }

    /// Una campaña que termina en un plazo razonable.
    fn estimacion_buena() -> Estimacion {
        estimar(500, Limites::nuevos(10, 50), &ventana(), AL_ABRIR)
    }

    /// Todo comprobado y todo bien.
    fn todo_en_orden(e: &Estimacion) -> Observaciones<'_> {
        Observaciones {
            cuenta_autentica: Some(true),
            credencial_sirve: Some(true),
            audiencia: 500,
            contactos_con_variables_sin_resolver: Some(0),
            estimacion: e,
            hay_conexion: Some(true),
            se_hizo_el_envio_de_prueba: Some(true),
            registros_del_dominio: Some(true),
        }
    }

    fn veredicto(p: &Preflight, punto: Punto) -> Veredicto {
        p.puntos
            .iter()
            .find(|r| r.punto == punto)
            .expect("el punto está en la lista")
            .veredicto
    }

    #[test]
    fn con_todo_en_orden_se_puede_activar() {
        let e = estimacion_buena();
        let r = revisar(&todo_en_orden(&e));
        assert_eq!(r.resultado, Resultado::Lista { avisos: 0 });
        assert!(r.resultado.se_puede_activar());
        assert_eq!(r.puntos.len(), 8, "los ocho del §63");
    }

    // ─── La tercera respuesta ───────────────────────────────────────────────

    /// **«No se ha comprobado» no es «pasa».**
    ///
    /// Es la regla que gobierna este archivo, y la misma que el validador
    /// aplica a las sondas: omitida no es pasada. Un preflight que diera por
    /// bueno lo que no ha mirado sería peor que no tenerlo.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que `Veredicto::de`
    /// devuelva `Pasa` ante `None`, esta prueba deja de bloquear y la campaña
    /// se activa con la credencial sin mirar.
    #[test]
    fn lo_que_no_se_ha_comprobado_no_deja_activar() {
        let e = estimacion_buena();
        let obs = Observaciones {
            credencial_sirve: None,
            ..todo_en_orden(&e)
        };
        let r = revisar(&obs);

        assert_eq!(
            veredicto(&r, Punto::LaCredencialSirve),
            Veredicto::SinComprobar
        );
        assert_eq!(
            r.resultado,
            Resultado::Bloqueada {
                fallan: 0,
                sin_comprobar: 1
            },
            "sin mirar la credencial no se activa, y se dice que es «sin \
             comprobar» y no «falla»: lo que hay que hacer es distinto"
        );
    }

    /// Fallar y no haber mirado se cuentan **por separado**.
    ///
    /// Un punto que falla se arregla; uno sin comprobar se comprueba. Mezclar
    /// las dos cifras convertiría «arregla la credencial y conéctate» en «hay
    /// dos problemas», que no dice qué hacer con ninguno.
    #[test]
    fn lo_que_falla_y_lo_que_no_se_miro_se_cuentan_aparte() {
        let e = estimacion_buena();
        let obs = Observaciones {
            cuenta_autentica: Some(false),
            hay_conexion: None,
            se_hizo_el_envio_de_prueba: None,
            ..todo_en_orden(&e)
        };
        assert_eq!(
            revisar(&obs).resultado,
            Resultado::Bloqueada {
                fallan: 1,
                sin_comprobar: 2
            }
        );
    }

    // ─── Lo que bloquea y lo que sólo avisa ─────────────────────────────────

    /// **Los registros del dominio avisan, no bloquean** (§154).
    ///
    /// Un dominio sin DMARC entrega peor, pero entrega. Bloquear por esto
    /// dejaría fuera a quien no controla su DNS —en una empresa pequeña, casi
    /// todo el mundo— por algo que ARLES no le puede arreglar.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que `bloquea` devuelva
    /// `true` para los ocho, esta campaña deja de poderse activar.
    #[test]
    fn un_dominio_sin_dmarc_avisa_pero_deja_activar() {
        let e = estimacion_buena();
        let obs = Observaciones {
            registros_del_dominio: Some(false),
            ..todo_en_orden(&e)
        };
        let r = revisar(&obs);

        assert_eq!(r.resultado, Resultado::Lista { avisos: 1 });
        assert!(
            r.resultado.se_puede_activar(),
            "informar, no dictar: esto no es un motivo para no dejar enviar"
        );
    }

    /// Y los otros siete sí bloquean, todos.
    #[test]
    fn los_otros_siete_bloquean() {
        let bloqueantes = Punto::TODOS.iter().filter(|p| p.bloquea()).count();
        assert_eq!(bloqueantes, 7);
        assert!(!Punto::LosRegistrosDelDominio.bloquea());
    }

    // ─── Los casos que se descubren tres días tarde ─────────────────────────

    /// **Una audiencia vacía no se activa.**
    ///
    /// Pasa de verdad: se importa una lista, se aplican las bajas, y no queda
    /// nadie. Sin este punto la campaña se activaría, no enviaría nada, y no
    /// habría nada roto que mirar.
    #[test]
    fn una_audiencia_vacia_no_se_activa() {
        let e = estimar(0, Limites::nuevos(10, 50), &ventana(), AL_ABRIR);
        let obs = Observaciones {
            audiencia: 0,
            ..todo_en_orden(&e)
        };
        let r = revisar(&obs);

        assert_eq!(
            veredicto(&r, Punto::LaAudienciaNoEstaVacia),
            Veredicto::Falla
        );
        assert!(!r.resultado.se_puede_activar());
    }

    /// **Una sola variable sin resolver ya bloquea.**
    ///
    /// No hay margen de tolerancia, y es deliberado: un correo que dice «Hola
    /// {{nombre}}» no se puede retirar, y con 340 contactos afectados son 340
    /// correos.
    #[test]
    fn un_solo_contacto_con_una_variable_sin_resolver_bloquea() {
        let e = estimacion_buena();
        let obs = Observaciones {
            contactos_con_variables_sin_resolver: Some(1),
            ..todo_en_orden(&e)
        };
        assert_eq!(
            veredicto(&revisar(&obs), Punto::LasVariablesSeResuelven),
            Veredicto::Falla
        );
    }

    /// **El simulador de duración deja de ser informativo y cierra la puerta.**
    ///
    /// Un tope en cero hace que la campaña no envíe nada. Con el cálculo
    /// enchufado al preflight, eso se ve **antes** de activar en vez de
    /// descubrirse cuando alguien pregunta por qué no ha llegado ningún correo.
    #[test]
    fn un_horario_que_no_alcanza_bloquea_la_activacion() {
        let no_avanza = estimar(500, Limites::nuevos(0, 50), &ventana(), AL_ABRIR);
        assert_eq!(no_avanza.desenlace, Desenlace::NoAvanza);

        let obs = todo_en_orden(&no_avanza);
        let r = revisar(&obs);
        assert_eq!(veredicto(&r, Punto::ElHorarioAlcanza), Veredicto::Falla);
        assert!(!r.resultado.se_puede_activar());
    }

    /// Y lo mismo si tardaría más de cinco años.
    #[test]
    fn una_campana_que_tardaria_anios_tampoco_se_activa() {
        let enorme = estimar(500_000, Limites::nuevos(10, 50), &ventana(), AL_ABRIR);
        assert!(matches!(
            enorme.desenlace,
            Desenlace::NoCabeEnElHorizonte { .. }
        ));

        let obs = Observaciones {
            audiencia: 500_000,
            ..todo_en_orden(&enorme)
        };
        assert_eq!(
            veredicto(&revisar(&obs), Punto::ElHorarioAlcanza),
            Veredicto::Falla
        );
    }

    // ─── La forma de la respuesta ───────────────────────────────────────────

    /// **Se revisan los ocho aunque el primero falle.**
    ///
    /// Parar en el primero haría que el usuario arreglara la credencial,
    /// volviera, y descubriera que además la audiencia está vacía. Tres viajes
    /// para tres problemas que caben en una pantalla.
    #[test]
    fn se_enseñan_los_ocho_aunque_el_primero_falle() {
        // Audiencia con gente y topes en cero: así falla también el horario.
        // Con la audiencia vacía el horario «alcanza» trivialmente —no hay a
        // quién escribir—, que es correcto y no es lo que esta prueba mide.
        let e = estimar(500, Limites::nuevos(0, 0), &ventana(), AL_ABRIR);
        let obs = Observaciones {
            cuenta_autentica: Some(false),
            credencial_sirve: Some(false),
            audiencia: 0,
            contactos_con_variables_sin_resolver: Some(9),
            estimacion: &e,
            hay_conexion: Some(false),
            se_hizo_el_envio_de_prueba: Some(false),
            registros_del_dominio: Some(false),
        };
        let r = revisar(&obs);

        assert_eq!(r.puntos.len(), 8);
        assert_eq!(
            r.resultado,
            Resultado::Bloqueada {
                fallan: 7,
                sin_comprobar: 0
            },
            "los siete bloqueantes fallan y el octavo es un aviso"
        );
    }

    /// Cada punto y cada veredicto llevan su clave de texto (§139).
    #[test]
    fn todo_lo_que_se_enseña_tiene_clave_de_texto() {
        for p in Punto::TODOS {
            assert!(p.clave_i18n().starts_with("preflight."));
        }
        for v in [Veredicto::Pasa, Veredicto::Falla, Veredicto::SinComprobar] {
            assert!(v.clave_i18n().starts_with("preflight."));
        }
    }

    /// El orden de los puntos es el de pantalla y no cambia solo.
    ///
    /// Una lista de comprobaciones que se reordena entre ejecuciones obliga a
    /// releerla entera cada vez.
    #[test]
    fn el_orden_de_los_puntos_es_estable() {
        let e = estimacion_buena();
        let r = revisar(&todo_en_orden(&e));
        let orden: Vec<Punto> = r.puntos.iter().map(|p| p.punto).collect();
        assert_eq!(orden, Punto::TODOS.to_vec());
    }

    /// La estimación que entra en el preflight es la misma que se enseña.
    ///
    /// No se recalcula por dentro: si el preflight hiciera su propio cálculo,
    /// podría bloquear por un motivo que la pantalla no está enseñando.
    #[test]
    fn el_preflight_usa_la_estimacion_que_se_le_da() {
        let e = estimacion_buena();
        assert_eq!(e.quien_manda, QuienManda::ElTopeDelDia);
        let r = revisar(&todo_en_orden(&e));
        assert_eq!(veredicto(&r, Punto::ElHorarioAlcanza), Veredicto::Pasa);
    }
}
