//! Qué hacer cuando un envío falla: reintentar, rendirse, o parar la cuenta.
//!
//! Ver `MOTOR_DE_EJECUCION.md` §6 y la decisión **D-12** del 23/09/2026.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO DIFÍCIL DE ESTO NO ES REINTENTAR: ES SABER CUÁNDO PARAR
//!
//! Reintentar un fallo pasajero es obvio y barato. Reintentar uno definitivo
//! hace daño activamente:
//!
//!   · Insistir con una dirección que no existe sube el contador que los
//!     proveedores usan para decidir si alguien es spam. Mandar a direcciones
//!     inexistentes es precisamente lo que hacen las listas compradas.
//!
//!   · Insistir con una contraseña caducada es la forma más rápida de que un
//!     proveedor bloquee la cuenta entera por seguridad.
//!
//! Por eso este módulo no tiene una política de reintentos: tiene **cinco
//! respuestas distintas**, y elegir la correcta es todo el trabajo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS ESPERAS LAS FIJÓ DIRECCIÓN, Y SON UNA TABLA, NO UNA FÓRMULA
//!
//! La primera propuesta era la de manual: empezar en 30 segundos y doblar
//! —30 s, 1 min, 2 min, 4 min—, cinco intentos. Dirección la cambió el
//! 23/09/2026 con un argumento que es correcto:
//!
//!   «Es común que los proveedores de correo o de internet, o un apagón,
//!    demoren bastantito.»
//!
//! Aquella tabla daba **siete minutos y medio** de margen total. Un corte de
//! luz o una caída de proveedor duran mucho más, así que los cinco intentos se
//! habrían gastado enteros dentro del mismo corte y todos los envíos habrían
//! terminado en fallo definitivo por un problema que se resolvió solo veinte
//! minutos después.
//!
//! Las de ahora —5, 15 y 30 minutos— dan **cincuenta minutos**. Y se escriben
//! como tabla y no como fórmula a propósito: una fórmula hay que ejecutarla
//! mentalmente para saber qué hace, y ésta la puede leer Dirección de un
//! vistazo y cambiarla sin tocar nada más.

use std::time::Duration;

use serde::Serialize;

/// Cuánto se espera antes de cada reintento (D-12).
///
/// El largo de esta lista **es** el número de reintentos: añadir una espera
/// añade un intento, y quitarla lo quita. No hay un tope escrito aparte que se
/// pueda desincronizar de la tabla — que es el defecto clásico de tener las dos
/// cosas: alguien añade una espera, olvida subir el tope, y la última nunca se
/// usa sin que nada falle.
pub const ESPERAS: [Duration; 3] = [
    Duration::from_secs(5 * 60),
    Duration::from_secs(15 * 60),
    Duration::from_secs(30 * 60),
];

/// Cuánto se permite que el azar mueva una espera, en tanto por uno.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ HAY AZAR
///
/// Se cae el internet de la oficina con mil envíos en marcha. Los mil fallan en
/// el mismo segundo. Si los mil esperasen exactamente cinco minutos, los mil
/// reintentarían **en el mismo instante**: el servidor, que acaba de volver,
/// recibe mil peticiones de golpe y se vuelve a caer. Y otra vez. Es un sistema
/// saboteándose a sí mismo al intentar recuperarse.
///
/// POR QUÉ ±20 % Y NO «ENTRE CERO Y LA ESPERA»
///
/// La receta habitual para esto es el *jitter* completo: esperar un rato al
/// azar entre cero y el máximo. Con las esperas antiguas —30 segundos— era
/// razonable.
///
/// Con las de Dirección **rompería su decisión**: «entre 0 y 5 minutos» haría
/// que algunos reintentos salieran a los diez segundos, que es exactamente lo
/// que esas cifras existen para evitar. Así que el azar se mueve *alrededor*
/// del número elegido, no desde cero: 5 minutos pasa a ser entre 4 y 6.
///
/// Reparte la carga igual de bien y respeta lo que se pidió.
pub const MARGEN_DE_AZAR: f64 = 0.20;

/// Qué contestó el proveedor, traducido a lo único que importa aquí.
///
/// No son códigos de error: son **decisiones**. La traducción de un `550` o un
/// `421` a una de estas variantes vive en el adaptador del proveedor (Fase 5),
/// porque cada uno los usa a su manera y el núcleo no tiene por qué saberlo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fallo {
    /// Se cayó la red, saltó un tiempo de espera, el servidor está saturado.
    /// No tiene que ver con este correo: dentro de un rato funcionará.
    Pasajero,
    /// El proveedor dijo **cuánto** esperar, con un número concreto.
    NoMeMandesNadaDurante(Duration),
    /// Esa dirección no existe. Insistir sube el contador que usan los
    /// proveedores para marcar a alguien como spam.
    LaDireccionNoExiste,
    /// La credencial no sirve. Afecta a **toda** la cuenta, no a este envío.
    LaCuentaNoAutentica,
    /// Esa persona pidió no recibir nada. No es un fallo: es el sistema
    /// funcionando.
    EsaPersonaPidioLaBaja,
}

/// Qué hace el motor a continuación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "decision")]
pub enum Decision {
    /// Volver a intentarlo dentro de `espera`.
    Reintentar {
        #[serde(skip)]
        espera: Duration,
        /// Cuál de los reintentos es, contando desde 1. Se enseña al usuario:
        /// «reintento 2 de 3» es una cifra, no un adjetivo (§94).
        cual: u8,
        de: u8,
    },
    /// Se agotaron los reintentos. Queda como fallido, con su motivo.
    Rendirse,
    /// Rendirse **y** meter la dirección en la lista de bajas.
    ///
    /// No se le vuelve a escribir nunca, ni en ésta ni en campañas futuras. Es
    /// una decisión con consecuencias, y por eso es visible en la pantalla de
    /// Bajas en vez de quedarse en una bitácora.
    RendirseYDarDeBaja,
    /// Parar la cuenta entera y decírselo al usuario con el motivo.
    ///
    /// Una campaña que se ralentiza en silencio es peor que una que se detiene
    /// y explica por qué.
    PararLaCuenta,
    /// No es un fallo. No se reintenta, no cuenta como error.
    NoEsUnFallo,
}

impl Decision {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Reintentar { .. } => "reintento.seVolveraAIntentar",
            Self::Rendirse => "reintento.seAgotaronLosIntentos",
            Self::RendirseYDarDeBaja => "reintento.direccionInexistente",
            Self::PararLaCuenta => "reintento.laCuentaNoAutentica",
            Self::NoEsUnFallo => "reintento.pidioLaBaja",
        }
    }
}

/// Cuánto esperar antes del reintento número `cual` (contando desde 1).
///
/// `azar` va de 0.0 a 1.0 y **lo aporta quien llama**, igual que en `ritmo`:
/// este módulo no tiene generador de números aleatorios, porque un módulo que
/// se sortea sus propios números no se puede probar — cada ejecución daría algo
/// distinto y habría que aceptar el resultado sin comprobarlo.
///
/// Devuelve `None` si ya no quedan reintentos.
#[must_use]
pub fn espera_del_reintento(cual: u8, azar: f64) -> Option<Duration> {
    let base = *ESPERAS.get(usize::from(cual.checked_sub(1)?))?;
    // Se sanea el azar en vez de confiar en quien llama: un valor fuera de
    // rango daría una espera disparatada, y aquí eso significa un reintento
    // inmediato contra un servidor que acaba de fallar.
    //
    // El `is_finite` no es adorno y lo encontró la prueba de este mismo
    // archivo: `f64::clamp` **devuelve NaN si recibe NaN** —está en su
    // documentación— y `Duration::mul_f64` revienta con un multiplicador no
    // finito. Recortar sin comprobar era una defensa que terminaba en pánico,
    // que es peor que no defenderse: el proceso se lleva la campaña por
    // delante en vez de enviar un correo un minuto antes.
    //
    // Un número inservible se trata como «sin azar»: la espera exacta de la
    // tabla. Es lo único que no requiere inventarse nada.
    let azar = if azar.is_finite() {
        azar.clamp(0.0, 1.0)
    } else {
        0.5
    };
    let factor = 1.0 - MARGEN_DE_AZAR + 2.0 * MARGEN_DE_AZAR * azar;
    Some(base.mul_f64(factor))
}

/// Qué hacer ante un fallo.
///
/// `reintentos_ya_hechos` es cuántas veces se reintentó **ya** este envío, sin
/// contar el original: 0 la primera vez que falla.
#[must_use]
pub fn que_hacer(fallo: Fallo, reintentos_ya_hechos: u8, azar: f64) -> Decision {
    match fallo {
        // El orden importa: los definitivos se resuelven ANTES de mirar
        // cuántos reintentos quedan. Preguntar primero por los reintentos haría
        // que una dirección inexistente se reintentara tres veces antes de
        // darse de baja, que es justo el daño que se quiere evitar.
        Fallo::EsaPersonaPidioLaBaja => Decision::NoEsUnFallo,
        Fallo::LaCuentaNoAutentica => Decision::PararLaCuenta,
        Fallo::LaDireccionNoExiste => Decision::RendirseYDarDeBaja,

        // El proveedor dijo un número. Se respeta **literalmente**: ni se
        // acorta por prisa ni se alarga por precaución.
        //
        // Cuando un servidor se toma la molestia de decir cuánto esperar,
        // está dando la única forma de no ser bloqueado. Acortarlo se
        // interpreta como intentar saltarse el límite, que es lo que hace el
        // software de spam. Y **no gasta reintento**: no es un fallo nuestro,
        // es una instrucción suya.
        Fallo::NoMeMandesNadaDurante(cuanto) => Decision::Reintentar {
            espera: cuanto,
            cual: reintentos_ya_hechos.saturating_add(1),
            de: total_de_reintentos(),
        },

        Fallo::Pasajero => {
            let cual = reintentos_ya_hechos.saturating_add(1);
            match espera_del_reintento(cual, azar) {
                Some(espera) => Decision::Reintentar {
                    espera,
                    cual,
                    de: total_de_reintentos(),
                },
                None => Decision::Rendirse,
            }
        }
    }
}

/// Cuántos reintentos hay, según la tabla.
#[must_use]
pub const fn total_de_reintentos() -> u8 {
    // `as` en un `const fn` sobre una longitud conocida de 3: no puede perder
    // información, y `try_into` no es `const`.
    #[allow(clippy::cast_possible_truncation)]
    {
        ESPERAS.len() as u8
    }
}

/// Cuánto margen total da la tabla, sin contar el azar.
///
/// Es la cifra que de verdad decide si la política sirve: es el tiempo que
/// puede durar un corte sin que los envíos terminen en fallo definitivo.
#[must_use]
pub fn margen_total() -> Duration {
    ESPERAS.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sin azar (0.5 = justo en medio), las esperas son las que fijó Dirección.
    #[test]
    fn las_esperas_son_cinco_quince_y_treinta_minutos() {
        assert_eq!(
            espera_del_reintento(1, 0.5),
            Some(Duration::from_secs(5 * 60))
        );
        assert_eq!(
            espera_del_reintento(2, 0.5),
            Some(Duration::from_secs(15 * 60))
        );
        assert_eq!(
            espera_del_reintento(3, 0.5),
            Some(Duration::from_secs(30 * 60))
        );
        assert_eq!(espera_del_reintento(4, 0.5), None, "sólo hay tres");
        assert_eq!(
            espera_del_reintento(0, 0.5),
            None,
            "no existe el reintento 0"
        );
    }

    /// **El margen total es de cincuenta minutos.**
    ///
    /// Es la cifra que justifica la decisión de Dirección del 23/09/2026. La
    /// tabla anterior —30 s, 1, 2 y 4 minutos— daba siete minutos y medio, y un
    /// apagón o una caída de proveedor duran más que eso: los intentos se
    /// habrían gastado enteros dentro del mismo corte.
    ///
    /// Esta prueba existe para que nadie recorte las esperas sin ver lo que
    /// pierde.
    #[test]
    fn el_margen_total_cubre_un_corte_de_verdad() {
        assert_eq!(
            margen_total(),
            Duration::from_secs(50 * 60),
            "las tres esperas tienen que sumar cincuenta minutos: es el tiempo \
             que puede durar un corte sin que los envíos mueran por él"
        );
        assert_eq!(total_de_reintentos(), 3);
    }

    /// El azar mueve la espera un ±20 %, no desde cero.
    ///
    /// ES LA PRUEBA QUE PROTEGE LA DECISIÓN DE DIRECCIÓN. La receta habitual
    /// —esperar entre cero y el máximo— haría que algunos reintentos salieran a
    /// los diez segundos, que es exactamente lo que las cifras de Dirección
    /// existen para evitar.
    #[test]
    fn el_azar_rodea_la_espera_en_vez_de_partir_de_cero() {
        let minimo = espera_del_reintento(1, 0.0).unwrap();
        let maximo = espera_del_reintento(1, 1.0).unwrap();

        assert_eq!(minimo, Duration::from_secs(4 * 60), "5 min menos un 20 %");
        assert_eq!(maximo, Duration::from_secs(6 * 60), "5 min más un 20 %");

        assert!(
            minimo >= Duration::from_secs(4 * 60),
            "ningún reintento puede salir antes de cuatro minutos. Si esto \
             falla, el azar parte de cero y la decisión de Dirección del \
             23/09/2026 queda anulada"
        );
    }

    /// Un azar fuera de rango no produce una espera disparatada.
    ///
    /// Aquí una espera negativa o de cero significa reintentar de inmediato
    /// contra un servidor que acaba de fallar.
    #[test]
    fn un_azar_fuera_de_rango_se_recorta() {
        assert_eq!(espera_del_reintento(1, -5.0), espera_del_reintento(1, 0.0));
        assert_eq!(espera_del_reintento(1, 99.0), espera_del_reintento(1, 1.0));
        assert_eq!(
            espera_del_reintento(1, f64::NAN),
            espera_del_reintento(1, 0.5),
            "un número inservible da la espera exacta de la tabla, sin azar"
        );
        assert_eq!(
            espera_del_reintento(1, f64::INFINITY),
            espera_del_reintento(1, 0.5)
        );
    }

    /// El recorrido completo de un fallo pasajero: tres reintentos y se rinde.
    #[test]
    fn un_fallo_pasajero_se_reintenta_tres_veces_y_se_rinde() {
        for ya in 0..3u8 {
            let d = que_hacer(Fallo::Pasajero, ya, 0.5);
            assert!(
                matches!(d, Decision::Reintentar { cual, de, .. } if cual == ya + 1 && de == 3),
                "tras {ya} reintentos debería tocar el {}: {d:?}",
                ya + 1
            );
        }
        assert_eq!(
            que_hacer(Fallo::Pasajero, 3, 0.5),
            Decision::Rendirse,
            "tras los tres, se rinde"
        );
    }

    // ── Los definitivos · donde está el valor de este módulo ─────────────

    /// **Una dirección inexistente no se reintenta ni una vez.**
    ///
    /// Es la prueba que más importa del módulo. Insistir con una dirección que
    /// no existe sube el contador que los proveedores usan para decidir si
    /// alguien manda listas compradas, y eso quema la reputación del dominio —
    /// lo único de ARLES que no se puede reparar después.
    ///
    /// Se comprueba con **cero** reintentos hechos a propósito: es el caso en
    /// que una política escrita en el orden equivocado sí reintentaría.
    #[test]
    fn una_direccion_inexistente_no_se_reintenta_jamas() {
        for ya in 0..5u8 {
            assert_eq!(
                que_hacer(Fallo::LaDireccionNoExiste, ya, 0.5),
                Decision::RendirseYDarDeBaja,
                "con {ya} reintentos hechos sigue sin reintentarse: insistir con \
                 una dirección que no existe es lo que quema el dominio"
            );
        }
    }

    /// Una credencial mala para la cuenta entera, no sólo este envío.
    #[test]
    fn una_credencial_mala_para_la_cuenta_entera() {
        assert_eq!(
            que_hacer(Fallo::LaCuentaNoAutentica, 0, 0.5),
            Decision::PararLaCuenta
        );
    }

    /// Quien pidió la baja no es un fallo.
    #[test]
    fn la_baja_no_cuenta_como_fallo() {
        assert_eq!(
            que_hacer(Fallo::EsaPersonaPidioLaBaja, 0, 0.5),
            Decision::NoEsUnFallo
        );
    }

    /// **«Espera diez minutos» significa diez minutos.**
    ///
    /// Ni nueve por prisa ni once por precaución. Cuando un servidor se toma la
    /// molestia de decir cuánto esperar, está dando la única forma de no ser
    /// bloqueado; acortarlo se lee como intentar saltarse el límite, que es lo
    /// que hace el software de spam.
    #[test]
    fn el_tiempo_que_pide_el_proveedor_se_respeta_al_pie_de_la_letra() {
        let diez = Duration::from_secs(10 * 60);
        let d = que_hacer(Fallo::NoMeMandesNadaDurante(diez), 0, 0.0);
        match d {
            Decision::Reintentar { espera, .. } => assert_eq!(
                espera, diez,
                "se pidieron diez minutos y hay que esperar diez, sin azar ni \
                 tabla de por medio"
            ),
            otra => panic!("debería reintentarse: {otra:?}"),
        }

        // Y con el azar en los extremos sigue siendo exacto: la tabla no toca
        // esto.
        for azar in [0.0, 0.5, 1.0] {
            assert!(matches!(
                que_hacer(Fallo::NoMeMandesNadaDurante(diez), 1, azar),
                Decision::Reintentar { espera, .. } if espera == diez
            ));
        }
    }

    /// Toda decisión tiene clave de texto (§139), y ninguna se repite.
    #[test]
    fn cada_decision_tiene_su_clave_y_ninguna_se_repite() {
        let decisiones = [
            Decision::Reintentar {
                espera: Duration::ZERO,
                cual: 1,
                de: 3,
            },
            Decision::Rendirse,
            Decision::RendirseYDarDeBaja,
            Decision::PararLaCuenta,
            Decision::NoEsUnFallo,
        ];
        let claves: std::collections::HashSet<_> =
            decisiones.iter().map(|d| d.clave_i18n()).collect();
        assert_eq!(claves.len(), decisiones.len());
        for d in decisiones {
            assert!(
                d.clave_i18n().starts_with("reintento."),
                "{d:?} sin prefijo"
            );
        }
    }
}
