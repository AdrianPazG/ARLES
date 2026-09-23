//! El disyuntor por cuenta: cuándo dejar de insistir.
//!
//! `MOTOR_DE_EJECUCION.md` §7.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ PROBLEMA RESUELVE, Y POR QUÉ NO LO RESUELVEN LOS REINTENTOS
//!
//! La política de reintentos (D-12) mira **un envío**: falló, espera cinco
//! minutos, vuelve. Está bien para un fallo de red pasajero.
//!
//! El disyuntor mira **la cuenta**. Hay fallos que no son de un envío sino del
//! remitente entero: el servidor de correo caído, el proveedor limitando por
//! reputación, una contraseña que dejó de servir. En esos casos los reintentos
//! empeoran las cosas: cada envío gasta sus tres intentos contra una puerta
//! cerrada, y lo único que consigue la insistencia es que el proveedor tome
//! nota de que ese remitente sigue llamando.
//!
//! Tras [`FALLOS_PARA_ABRIR`] fallos seguidos, la cuenta **para**. El §7 lo
//! dice con claridad: «un fallo silencioso que simplemente ralentiza la
//! campaña es peor que una parada visible».
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NO TODO FALLO CUENTA, Y ÉSA ES LA PARTE QUE SE PUEDE HACER MAL
//!
//! Que una dirección no exista **no es culpa de la cuenta**. Una lista
//! importada con cinco direcciones muertas seguidas no dice nada malo del
//! remitente, y si contaran, esa campaña se pararía sola cada pocos minutos y
//! el usuario no entendería por qué.
//!
//! Cuentan los fallos que hablan del **canal**: red, tiempos de espera,
//! saturación, «no me mandes nada durante». No cuentan los que hablan del
//! **destinatario**: dirección inexistente, persona dada de baja.
//!
//! Y la credencial no espera a cinco: abre el disyuntor **a la primera**.
//! Contar hasta cinco cuando ya sabemos que la contraseña no sirve sólo gasta
//! los reintentos de otros cuatro envíos.

use std::time::Duration;

use serde::Serialize;

use crate::reintento::Fallo;

/// Cuántos fallos seguidos del canal abren el disyuntor.
///
/// Cinco es el valor del §7. No es un número mágico: es lo bastante alto para
/// que un par de tropiezos de red no paren una campaña, y lo bastante bajo
/// para no gastar quince reintentos —cinco envíos por tres intentos cada uno—
/// contra un servidor que no está.
pub const FALLOS_PARA_ABRIR: u32 = 5;

/// Cuánto permanece abierto antes de volver a probar solo.
///
/// Media hora. Un plazo corto convierte el disyuntor en un reintento más caro;
/// uno largo deja parada una campaña por una caída de servidor que duró dos
/// minutos. Media hora es el orden de magnitud de una incidencia de proveedor
/// de correo, y el usuario puede cerrarlo a mano antes si arregla la causa.
pub const ESPERA_DEL_DISYUNTOR: Duration = Duration::from_secs(30 * 60);

/// En qué estado está el disyuntor de una cuenta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "estado")]
pub enum Circuito {
    /// La cuenta puede enviar.
    Cerrado,
    /// La cuenta está parada. `segundos_restantes` es una cifra (§94).
    Abierto { segundos_restantes: u64 },
}

impl Circuito {
    /// ¿Puede esta cuenta tomar trabajo ahora?
    #[must_use]
    pub const fn puede_enviar(self) -> bool {
        matches!(self, Self::Cerrado)
    }

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Cerrado => "disyuntor.cerrado",
            Self::Abierto { .. } => "disyuntor.abierto",
        }
    }
}

/// Qué hacer con la cuenta tras el desenlace de un envío.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "efecto")]
pub enum Efecto {
    /// Todo bien: el contador de fallos seguidos vuelve a cero.
    ///
    /// **Se reinicia con un solo acierto**, no poco a poco. Un envío que sale
    /// prueba que la cuenta funciona, y arrastrar fallos viejos haría que una
    /// campaña larga acabara abriendo el disyuntor por tropiezos repartidos a
    /// lo largo de semanas.
    Reiniciar,
    /// Un fallo del canal más. Todavía no se abre.
    Contar { van: u32, de: u32 },
    /// Se abre el disyuntor: la cuenta deja de enviar.
    Abrir {
        durante: Duration,
        por_credencial: bool,
    },
    /// Este fallo no dice nada de la cuenta. El contador no se toca.
    NoCuenta,
}

impl Efecto {
    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Reiniciar => "disyuntor.reiniciar",
            Self::Contar { .. } => "disyuntor.contar",
            Self::Abrir { .. } => "disyuntor.abrir",
            Self::NoCuenta => "disyuntor.noCuenta",
        }
    }
}

/// ¿Este fallo habla de la cuenta o del destinatario?
///
/// Es la distinción que hace que el disyuntor sirva. Ver la nota de cabecera.
#[must_use]
pub const fn habla_de_la_cuenta(fallo: Fallo) -> bool {
    match fallo {
        // Del canal: la red, el servidor, su ritmo.
        Fallo::Pasajero | Fallo::NoMeMandesNadaDurante(_) | Fallo::LaCuentaNoAutentica => true,
        // Del destinatario: esa dirección, esa persona.
        Fallo::LaDireccionNoExiste | Fallo::EsaPersonaPidioLaBaja => false,
    }
}

/// Qué le pasa al disyuntor tras un envío aceptado.
#[must_use]
pub const fn tras_un_acierto() -> Efecto {
    Efecto::Reiniciar
}

/// Qué le pasa al disyuntor tras un fallo.
///
/// `fallos_seguidos` es cuántos llevaba **antes** de éste.
#[must_use]
pub const fn tras_un_fallo(fallo: Fallo, fallos_seguidos: u32) -> Efecto {
    if !habla_de_la_cuenta(fallo) {
        return Efecto::NoCuenta;
    }

    // La credencial no espera a cinco. Ya sabemos que ningún envío de esta
    // cuenta va a funcionar hasta que una persona la arregle; contar hasta
    // cinco sólo gastaría los reintentos de otros cuatro envíos.
    if matches!(fallo, Fallo::LaCuentaNoAutentica) {
        return Efecto::Abrir {
            durante: ESPERA_DEL_DISYUNTOR,
            por_credencial: true,
        };
    }

    let van = fallos_seguidos.saturating_add(1);
    if van >= FALLOS_PARA_ABRIR {
        Efecto::Abrir {
            durante: ESPERA_DEL_DISYUNTOR,
            por_credencial: false,
        }
    } else {
        Efecto::Contar {
            van,
            de: FALLOS_PARA_ABRIR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Cinco fallos de red seguidos paran la cuenta.**
    ///
    /// Es el §7 medido: lo que se quiere evitar es que cada envío gaste sus
    /// tres reintentos contra un servidor que no está.
    #[test]
    fn cinco_fallos_de_red_seguidos_abren_el_disyuntor() {
        for van in 0..FALLOS_PARA_ABRIR - 1 {
            assert_eq!(
                tras_un_fallo(Fallo::Pasajero, van),
                Efecto::Contar {
                    van: van + 1,
                    de: FALLOS_PARA_ABRIR
                },
                "con {van} fallos previos todavía no se abre"
            );
        }
        assert_eq!(
            tras_un_fallo(Fallo::Pasajero, FALLOS_PARA_ABRIR - 1),
            Efecto::Abrir {
                durante: ESPERA_DEL_DISYUNTOR,
                por_credencial: false
            }
        );
    }

    /// **Cinco direcciones muertas NO paran la cuenta.**
    ///
    /// Es la mitad que se puede hacer mal, y la que rompería el producto: una
    /// lista importada con direcciones viejas tiene rachas de rebotes, y si
    /// contaran, la campaña se pararía sola cada pocos minutos sin que el
    /// usuario entendiera por qué.
    ///
    /// Verificada rompiéndola el 23/09/2026: haciendo que `habla_de_la_cuenta`
    /// devuelva `true` para todo, esta prueba falla en la primera vuelta.
    #[test]
    fn una_racha_de_direcciones_muertas_no_para_la_cuenta() {
        for van in 0..FALLOS_PARA_ABRIR + 5 {
            assert_eq!(
                tras_un_fallo(Fallo::LaDireccionNoExiste, van),
                Efecto::NoCuenta,
                "que una dirección no exista no dice nada malo del remitente"
            );
        }
    }

    /// Ni una racha de bajas, que ni siquiera son fallos.
    #[test]
    fn una_racha_de_bajas_tampoco_para_la_cuenta() {
        assert_eq!(
            tras_un_fallo(Fallo::EsaPersonaPidioLaBaja, 4),
            Efecto::NoCuenta
        );
    }

    /// **La credencial abre el disyuntor a la primera.**
    ///
    /// Esperar a cinco cuando ya sabemos que la contraseña no sirve sólo gasta
    /// los reintentos de otros cuatro envíos.
    #[test]
    fn una_credencial_que_no_sirve_no_espera_a_cinco() {
        assert_eq!(
            tras_un_fallo(Fallo::LaCuentaNoAutentica, 0),
            Efecto::Abrir {
                durante: ESPERA_DEL_DISYUNTOR,
                por_credencial: true
            },
            "y se marca que fue la credencial: lo que hay que hacer es \
             renovarla, no esperar media hora"
        );
    }

    /// **Un solo acierto reinicia el contador.**
    ///
    /// Arrastrar fallos viejos haría que una campaña de cuatro meses acabara
    /// abriendo el disyuntor por cinco tropiezos repartidos en semanas.
    #[test]
    fn un_envio_que_sale_borra_los_fallos_anteriores() {
        assert_eq!(tras_un_acierto(), Efecto::Reiniciar);

        // Cuatro fallos, un acierto, y vuelta a empezar desde cero.
        assert_eq!(
            tras_un_fallo(Fallo::Pasajero, 0),
            Efecto::Contar {
                van: 1,
                de: FALLOS_PARA_ABRIR
            },
            "tras reiniciar, el siguiente fallo es el primero otra vez"
        );
    }

    /// Un servidor que pide esperar también habla de la cuenta.
    ///
    /// Cinco «no me mandes nada durante diez minutos» seguidos no son un
    /// tropiezo: son un proveedor diciendo que este remitente va demasiado
    /// rápido.
    #[test]
    fn el_que_pide_esperar_cuenta_para_el_disyuntor() {
        assert!(habla_de_la_cuenta(Fallo::NoMeMandesNadaDurante(
            Duration::from_secs(600)
        )));
    }

    /// Un circuito abierto no deja enviar, y dice cuánto falta.
    #[test]
    fn un_circuito_abierto_dice_cuanto_falta() {
        assert!(Circuito::Cerrado.puede_enviar());
        let abierto = Circuito::Abierto {
            segundos_restantes: 900,
        };
        assert!(!abierto.puede_enviar());
        assert_eq!(abierto.clave_i18n(), "disyuntor.abierto");
    }

    /// Media hora, y cinco fallos. Si alguien los cambia, esto dice cuáles
    /// eran.
    #[test]
    fn los_umbrales_son_los_del_septimo_apartado() {
        assert_eq!(FALLOS_PARA_ABRIR, 5);
        assert_eq!(ESPERA_DEL_DISYUNTOR, Duration::from_secs(1_800));
    }
}
