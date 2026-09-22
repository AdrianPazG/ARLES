//! El ritmo de envío: cuántos correos caben, cuándo, y qué pasa al despertar.
//!
//! Ver `documentacion/03-arquitectura/MOTOR_DE_EJECUCION.md` §4 y §5.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO NO TIENE NI RELOJ NI BASE DE DATOS
//!
//! `arles-core` no hace I/O, y aquí eso no es purismo: es lo que permite
//! **probar el paso del tiempo**. El módulo recibe los instantes ya leídos y
//! devuelve decisiones. Quien los lee es el motor.
//!
//! Si este módulo llamara a `SystemTime::now()`, la prueba del caso que más
//! importa —que el equipo duerma ocho horas y al despertar no salga una
//! ráfaga— exigiría dormir ocho horas de verdad, o parchear el reloj del
//! sistema. Con los instantes como parámetro, esa prueba es aritmética.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL FALLO QUE ESTE MÓDULO EXISTE PARA EVITAR (§53, riesgo R-04)
//!
//! Un cubo de tokens normal se recarga según el tiempo transcurrido: «han
//! pasado diez minutos, me tocan diez minutos de permisos». Es correcto
//! mientras el equipo esté encendido.
//!
//! Si el equipo duerme ocho horas, ese mismo cálculo dice «me corresponden
//! ocho horas de permisos» y los suelta **todos de golpe**. El resultado es
//! una ráfaga de cientos de correos en un minuto, que es la forma más rápida
//! que existe de que un proveedor marque la cuenta como spam.
//!
//! La respuesta de ARLES es **reiniciar el presupuesto, no acumularlo**. La
//! consecuencia, que es la correcta y hay que decirla: **un correo que no
//! salió ayer no se recupera hoy**. La campaña dura más. Si el simulador dijo
//! cien días y el equipo estuvo apagado tres, dura ciento tres.

use serde::Serialize;

/// Cuántos correos caben por hora y por día en una cuenta remitente.
///
/// Las dos ventanas corren **a la vez**: un envío gasta un permiso de cada
/// una. Con sólo la diaria, quinientos correos podrían salir en un minuto; con
/// sólo la horaria, no habría tope real al cabo del día.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limites {
    pub por_hora: u32,
    pub por_dia: u32,
}

impl Limites {
    /// El umbral del §48: por encima de esto se avisa del riesgo de
    /// entregabilidad, **y no se bloquea**. El §154 pide autonomía: informar,
    /// no dictar.
    pub const AVISO_DIARIO: u32 = 50;

    #[must_use]
    pub const fn nuevos(por_hora: u32, por_dia: u32) -> Self {
        Self { por_hora, por_dia }
    }

    /// ¿Hay que enseñar el aviso de entregabilidad antes de aceptar esto?
    #[must_use]
    pub const fn necesita_aviso(self) -> bool {
        self.por_dia > Self::AVISO_DIARIO
    }
}

/// Lo gastado hasta ahora en cada ventana.
///
/// Esto es lo que vive en `rate_budget` y sobrevive a cerrar la aplicación. Un
/// contador en memoria se reiniciaría al abrirla otra vez, y el límite diario
/// se podría gastar varias veces el mismo día sencillamente cerrando y
/// volviendo a abrir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gastado {
    pub en_esta_hora: u32,
    pub en_este_dia: u32,
}

/// Por qué no se puede enviar ahora mismo.
///
/// Cada variante lleva su clave de texto (§139) y **dice qué hacer**: un
/// «no se puede» sin motivo se lee como una avería (§95).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "motivo")]
pub enum NoSePuede {
    /// Se agotó el permiso de esta hora. Se reanuda solo a la hora siguiente.
    TopeDeLaHora { cabian: u32 },
    /// Se agotó el permiso de hoy. Se reanuda solo mañana.
    TopeDelDia { cabian: u32 },
    /// Estamos fuera del horario de trabajo que configuró la empresa.
    FueraDeVentana,
}

impl NoSePuede {
    /// La clave de i18n del motivo (§139: los textos no viven en Rust).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::TopeDeLaHora { .. } => "ritmo.topeDeLaHora",
            Self::TopeDelDia { .. } => "ritmo.topeDelDia",
            Self::FueraDeVentana => "ritmo.fueraDeVentana",
        }
    }
}

/// ¿Cabe un envío más?
///
/// El orden de las comprobaciones **importa y está elegido**: primero la
/// ventana, después el día, después la hora. Se informa siempre del motivo
/// que más tiempo va a durar, porque es el que de verdad explica la espera.
/// Decir «espera a la hora siguiente» cuando además se acabó el día haría que
/// el usuario volviera en una hora a encontrarse lo mismo.
///
/// # Errors
/// Devuelve el motivo por el que no cabe.
pub fn cabe_uno_mas(
    limites: Limites,
    gastado: Gastado,
    dentro_de_ventana: bool,
) -> Result<(), NoSePuede> {
    if !dentro_de_ventana {
        return Err(NoSePuede::FueraDeVentana);
    }
    if gastado.en_este_dia >= limites.por_dia {
        return Err(NoSePuede::TopeDelDia {
            cabian: limites.por_dia,
        });
    }
    if gastado.en_esta_hora >= limites.por_hora {
        return Err(NoSePuede::TopeDeLaHora {
            cabian: limites.por_hora,
        });
    }
    Ok(())
}

/// Qué hacer con el presupuesto al volver de una pausa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "clase")]
pub enum Despertar {
    /// No pasó nada raro: el tiempo corrió como debía.
    SinNovedad,
    /// El equipo estuvo suspendido. **El presupuesto se reinicia.**
    ///
    /// `segundos_dormido` es aproximado a propósito: sale de restar dos
    /// relojes que no se leen en el mismo instante, así que lleva el error de
    /// esa distancia. Sirve para explicárselo al usuario y para la bitácora,
    /// nunca para calcular permisos — si se usara para eso, volveríamos al
    /// cubo que suelta la ráfaga.
    HuboSuspension { segundos_dormido: u64 },
}

/// Cuánto desajuste entre los dos relojes se considera normal.
///
/// Los dos no se leen en el mismo instante y ninguno es perfecto, así que
/// siempre hay unas décimas de diferencia. Dos segundos es holgado para el
/// ruido y muy inferior a cualquier suspensión real: nadie suspende un equipo
/// dos segundos.
///
/// Un umbral demasiado bajo haría saltar la detección constantemente y el
/// presupuesto se reiniciaría solo, todo el rato — un fallo peor que el que se
/// quiere evitar, porque haría que ARLES nunca alcanzara su propio límite.
pub const UMBRAL_DE_SUSPENSION_S: u64 = 2;

/// ¿El equipo estuvo suspendido entre dos observaciones?
///
/// Se le pasan los dos saltos **ya medidos**: el del reloj monótono —que no
/// avanza mientras el equipo duerme— y el del reloj de pared, que sí. La
/// diferencia entre los dos **es** lo que duró la suspensión.
///
/// Por qué hacen falta los dos: el reloj de pared solo no distingue «el equipo
/// durmió ocho horas» de «alguien cambió la hora del sistema», y el monótono
/// solo no puede ver la suspensión en absoluto, que es precisamente su
/// definición. Ninguno de los dos basta; juntos, la respuesta es una resta.
#[must_use]
pub fn al_despertar(salto_monotono_s: u64, salto_de_pared_s: u64) -> Despertar {
    // La resta va saturada, no con `-`: si el reloj de pared se atrasa —un
    // ajuste de NTP hacia atrás, alguien corrigiendo la hora— el salto de
    // pared queda por debajo del monótono y una resta normal desbordaría. Un
    // desbordamiento aquí se llevaría el proceso por delante en mitad de una
    // campaña, y por un reloj que se movió doscientos milisegundos.
    let dormido = salto_de_pared_s.saturating_sub(salto_monotono_s);
    if dormido > UMBRAL_DE_SUSPENSION_S {
        Despertar::HuboSuspension {
            segundos_dormido: dormido,
        }
    } else {
        Despertar::SinNovedad
    }
}

impl Despertar {
    /// Lo gastado que le corresponde al motor después de despertar.
    ///
    /// **Aquí es donde se impide la ráfaga.** Tras una suspensión el
    /// presupuesto vuelve a cero, y eso no significa «te devuelvo lo que no
    /// gastaste»: significa que la hora y el día que estaban en curso ya no
    /// existen, y empieza el cómputo de los actuales.
    ///
    /// Un cubo que sumara lo dormido tendría aquí, en esta única función, el
    /// fallo entero: ocho horas dormidas darían ocho horas de permisos de
    /// golpe.
    #[must_use]
    pub const fn presupuesto(self, antes: Gastado) -> Gastado {
        match self {
            Self::SinNovedad => antes,
            Self::HuboSuspension { .. } => Gastado {
                en_esta_hora: 0,
                en_este_dia: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El caso normal: hay sitio y se envía.
    #[test]
    fn con_sitio_de_sobra_cabe() {
        let l = Limites::nuevos(10, 50);
        let g = Gastado {
            en_esta_hora: 3,
            en_este_dia: 20,
        };
        assert_eq!(cabe_uno_mas(l, g, true), Ok(()));
    }

    /// Fuera del horario de la empresa no se envía, aunque quede presupuesto.
    ///
    /// ROTO A PROPÓSITO: se quitó la comprobación de ventana y esta prueba
    /// falló diciendo que cabía. Restaurada.
    #[test]
    fn fuera_de_ventana_no_se_envia_aunque_sobre_presupuesto() {
        let l = Limites::nuevos(100, 1000);
        let g = Gastado::default();
        assert_eq!(
            cabe_uno_mas(l, g, false),
            Err(NoSePuede::FueraDeVentana),
            "fuera del horario configurado no se manda nada, por mucho sitio que haya"
        );
    }

    /// Cuando se acaban las dos ventanas, se nombra **la diaria**.
    ///
    /// Es la que más dura. Decirle a alguien «espera a la hora siguiente»
    /// cuando además se acabó el día lo haría volver en una hora a
    /// encontrarse exactamente lo mismo.
    #[test]
    fn si_se_acaban_las_dos_se_nombra_la_que_mas_dura() {
        let l = Limites::nuevos(10, 50);
        let g = Gastado {
            en_esta_hora: 10,
            en_este_dia: 50,
        };
        assert_eq!(
            cabe_uno_mas(l, g, true),
            Err(NoSePuede::TopeDelDia { cabian: 50 }),
            "con las dos agotadas se informa de la diaria, que es la que explica la espera"
        );
    }

    /// El tope horario se nombra cuando es el único agotado.
    #[test]
    fn el_tope_de_la_hora_se_nombra_cuando_es_el_unico() {
        let l = Limites::nuevos(10, 500);
        let g = Gastado {
            en_esta_hora: 10,
            en_este_dia: 10,
        };
        assert_eq!(
            cabe_uno_mas(l, g, true),
            Err(NoSePuede::TopeDeLaHora { cabian: 10 })
        );
    }

    /// El límite es «hasta», no «hasta e incluyendo uno más».
    ///
    /// Un `>` en vez de `>=` dejaría enviar exactamente **uno de más** en cada
    /// ventana. Con un tope de 50 diarios serían 51, y ese uno de más es justo
    /// el que cruza el umbral que el proveedor vigila.
    #[test]
    fn el_ultimo_permiso_se_usa_y_el_siguiente_ya_no() {
        let l = Limites::nuevos(3, 100);
        let casi = Gastado {
            en_esta_hora: 2,
            en_este_dia: 2,
        };
        assert_eq!(
            cabe_uno_mas(l, casi, true),
            Ok(()),
            "el tercero todavía cabe"
        );

        let lleno = Gastado {
            en_esta_hora: 3,
            en_este_dia: 3,
        };
        assert!(
            cabe_uno_mas(l, lleno, true).is_err(),
            "el cuarto no: el tope es 3, no 4"
        );
    }

    /// El aviso del §48 salta por encima de cincuenta, no en cincuenta.
    #[test]
    fn el_aviso_de_entregabilidad_salta_pasado_el_umbral() {
        assert!(
            !Limites::nuevos(10, 50).necesita_aviso(),
            "50 es el umbral, no lo pasa"
        );
        assert!(Limites::nuevos(10, 51).necesita_aviso());
    }

    // ── La ráfaga · el caso que este módulo existe para evitar ───────────

    /// **La prueba de la puerta de salida de la Fase 4.**
    ///
    /// El equipo duerme ocho horas. Al despertar, el presupuesto vuelve a
    /// cero: ni se acumulan las ocho horas de permisos ni se «recuperan» los
    /// correos que no salieron.
    ///
    /// ROTO A PROPÓSITO: se cambió `presupuesto` para que sumara lo dormido en
    /// vez de reiniciar —que es exactamente el cubo ingenuo del §5— y la
    /// prueba falló nombrando la ráfaga. Restaurado.
    #[test]
    fn ocho_horas_dormido_no_producen_una_rafaga() {
        const OCHO_HORAS: u64 = 8 * 3600;

        // El reloj monótono no avanzó durante la suspensión: sólo el segundo
        // que tardó el planificador en volver a mirar.
        let despertar = al_despertar(1, OCHO_HORAS);
        assert_eq!(
            despertar,
            Despertar::HuboSuspension {
                segundos_dormido: OCHO_HORAS - 1
            },
            "la diferencia entre los dos relojes ES la suspensión"
        );

        let antes = Gastado {
            en_esta_hora: 9,
            en_este_dia: 40,
        };
        let despues = despertar.presupuesto(antes);

        assert_eq!(
            despues,
            Gastado::default(),
            "tras dormir, el presupuesto se REINICIA. Acumularlo soltaría de \
             golpe ocho horas de permisos, que es la ráfaga que prohíbe el §53"
        );

        // Y la consecuencia, que es la correcta: lo que no salió, no se
        // recupera. Con el tope horario de 10 sólo cabe uno más, no los 31 que
        // «sobraban» del día anterior.
        let l = Limites::nuevos(10, 50);
        assert_eq!(cabe_uno_mas(l, despues, true), Ok(()));
    }

    /// Un tic normal del planificador no se confunde con una suspensión.
    ///
    /// Si se confundiera, el presupuesto se reiniciaría en cada vuelta y ARLES
    /// no alcanzaría nunca su propio límite: un fallo más silencioso que la
    /// ráfaga y más difícil de ver.
    #[test]
    fn un_tic_normal_no_se_confunde_con_dormir() {
        assert_eq!(al_despertar(30, 30), Despertar::SinNovedad);
        assert_eq!(
            al_despertar(30, 31),
            Despertar::SinNovedad,
            "un segundo de deriva entre relojes es ruido, no una suspensión"
        );
        let antes = Gastado {
            en_esta_hora: 7,
            en_este_dia: 7,
        };
        assert_eq!(
            Despertar::SinNovedad.presupuesto(antes),
            antes,
            "sin suspensión, lo gastado sigue gastado"
        );
    }

    /// Justo en el umbral todavía no cuenta como suspensión.
    #[test]
    fn el_umbral_es_estricto() {
        assert_eq!(
            al_despertar(0, UMBRAL_DE_SUSPENSION_S),
            Despertar::SinNovedad
        );
        assert!(matches!(
            al_despertar(0, UMBRAL_DE_SUSPENSION_S + 1),
            Despertar::HuboSuspension { .. }
        ));
    }

    /// Si el reloj de pared va POR DETRÁS del monótono, no se revienta.
    ///
    /// Pasa de verdad: un ajuste de NTP hacia atrás, o alguien corrigiendo la
    /// hora del equipo. Con una resta normal esto desbordaría y se llevaría el
    /// proceso por delante en mitad de una campaña.
    ///
    /// ROTO A PROPÓSITO: se cambió `saturating_sub` por `-` y esta prueba
    /// reventó con «attempt to subtract with overflow». Restaurado.
    #[test]
    fn un_reloj_que_va_hacia_atras_no_revienta_el_motor() {
        assert_eq!(
            al_despertar(100, 40),
            Despertar::SinNovedad,
            "un reloj de pared atrasado no es una suspensión, y no puede tirar el proceso"
        );
    }

    /// Todo motivo tiene clave de texto (§139), y ninguna se repite.
    #[test]
    fn cada_motivo_tiene_su_clave_y_ninguna_se_repite() {
        let motivos = [
            NoSePuede::TopeDeLaHora { cabian: 1 },
            NoSePuede::TopeDelDia { cabian: 1 },
            NoSePuede::FueraDeVentana,
        ];
        let claves: std::collections::HashSet<_> = motivos.iter().map(|m| m.clave_i18n()).collect();
        assert_eq!(
            claves.len(),
            motivos.len(),
            "hay dos motivos con la misma clave"
        );
        for m in motivos {
            assert!(
                m.clave_i18n().starts_with("ritmo."),
                "{m:?} no lleva su prefijo"
            );
        }
    }
}
