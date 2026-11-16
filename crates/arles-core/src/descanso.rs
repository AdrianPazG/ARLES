//! El descanso entre campañas · D-16, pedido el 22/10/2026
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! LO QUE PIDIÓ DIRECCIÓN, CON SUS PALABRAS
//!
//!   «Ya si en ambas campañas o más se repite un contacto, como son dos
//!   plantillas diferentes de dos o más campañas diferentes, se le envía el
//!   correo de las dos campañas. Sólo que me gustaría tener cuidado que no se
//!   envíen en el mismo día; si es posible que pase un lapso de 1 o 2 semanas
//!   para no hostigar.»
//!
//! Las dos mitades importan y son distintas:
//!
//!   - **Recibe las dos**, porque son dos mensajes distintos y los dos le
//!     interesan. No se cancela ninguna.
//!   - **Pero no el mismo día.** Dos correos de la misma empresa en una tarde
//!     es lo que hace que alguien marque el segundo como no deseado, y la tasa
//!     de quejas es la cifra que más pesa en la reputación de un dominio.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! SE ESPERA, NO SE CANCELA — Y ESA ES LA DECISIÓN
//!
//! Lo cómodo sería sacar de la audiencia a quien recibió algo hace poco.
//! Sería **perder el envío**: esa persona no recibiría nunca la segunda
//! campaña, y nadie lo sabría, porque una audiencia más corta no se distingue
//! de una lista más corta.
//!
//! Lo que ocurre es que el envío **espera su turno** y sale cuando toca. La
//! campaña tarda más y lo dice; nadie se queda sin su mensaje.

/// Cuántos días tienen que pasar entre dos campañas para la misma persona.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ CATORCE Y NO SIETE
///
/// Dirección dijo «1 o 2 semanas», así que las dos cumplen lo que pidió. Se
/// toma la de arriba porque **el error de esperar de más es reversible y el de
/// esperar de menos no**: una campaña que tarda dos semanas más se explica;
/// una queja de spam se queda en el historial del dominio durante meses y no
/// hay forma de retirarla.
///
/// Y porque el coste real es pequeño: sólo afecta a **quien está en dos
/// listas**, no a la campaña entera.
pub const DESCANSO_ENTRE_CAMPANAS_DIAS: i64 = 14;

/// El mínimo que se puede configurar, si algún día se configura.
///
/// Está escrito aunque hoy no haya pantalla para cambiarlo, por la misma razón
/// que el mínimo de un año del archivado: el día que alguien añada el campo, la
/// frontera ya tiene con qué no fiarse de la pantalla. Siete porque es el
/// extremo bajo de lo que Dirección pidió.
pub const DESCANSO_MINIMO_DIAS: i64 = 7;

/// Qué hacer con un envío a alguien que ya recibió algo de otra campaña.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Descanso {
    /// Nadie le ha escrito hace poco: sale ahora.
    PuedeSalir,
    /// Le escribimos hace poco. **Espera**, y aquí está cuántos segundos.
    ///
    /// Se devuelven segundos y no una fecha porque el núcleo no tiene reloj
    /// (cero I/O): quien sabe qué hora es, y por tanto puede sumar, es
    /// `arles-db`. Aquí sólo se decide cuánto falta.
    Espera {
        /// Desde el último envío, cuánto queda para que se le pueda escribir.
        segundos: i64,
    },
}

/// Un día en segundos. No es una constante de conveniencia: es la unidad en la
/// que se mide el descanso, y escribirla una vez evita el `60 * 60 * 24`
/// repetido que algún día saldrá mal.
const UN_DIA: i64 = 86_400;

/// ¿Puede salir este envío, o tiene que esperar?
///
/// `desde_el_ultimo_envio` son los segundos transcurridos desde que se le
/// escribió por última vez **desde otra campaña**. Negativo o cero significa
/// «acaba de recibir»: se trata igual que cero, porque un reloj que va hacia
/// atrás —cambio de hora, ajuste del sistema— no puede convertirse en permiso
/// para escribir dos veces.
///
/// `dias_de_descanso` se **sube al mínimo** si viene por debajo, igual que el
/// corte del archivado: la frontera no se fía de la pantalla.
#[must_use]
pub const fn puede_salir(desde_el_ultimo_envio: i64, dias_de_descanso: i64) -> Descanso {
    let dias = if dias_de_descanso < DESCANSO_MINIMO_DIAS {
        DESCANSO_MINIMO_DIAS
    } else {
        dias_de_descanso
    };
    let descanso = dias * UN_DIA;

    // Un reloj hacia atrás no da permiso: se trata como «acaba de recibir».
    let transcurrido = if desde_el_ultimo_envio < 0 {
        0
    } else {
        desde_el_ultimo_envio
    };

    if transcurrido >= descanso {
        Descanso::PuedeSalir
    } else {
        Descanso::Espera {
            segundos: descanso - transcurrido,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Quien no ha recibido nada sale ahora.
    #[test]
    fn sin_envio_anterior_sale() {
        // Catorce días y un segundo: hace mucho que no se le escribe.
        assert_eq!(
            puede_salir(14 * UN_DIA + 1, DESCANSO_ENTRE_CAMPANAS_DIAS),
            Descanso::PuedeSalir
        );
    }

    /// **El caso que motivó esto: dos campañas el mismo día.**
    ///
    /// Es la frase de Dirección —«que no se envíen en el mismo día»— y es el
    /// único caso que hay que acertar seguro.
    #[test]
    fn el_mismo_dia_espera() {
        let r = puede_salir(2 * 3_600, DESCANSO_ENTRE_CAMPANAS_DIAS);
        match r {
            Descanso::Espera { segundos } => {
                assert_eq!(segundos, 14 * UN_DIA - 2 * 3_600);
            }
            Descanso::PuedeSalir => panic!("dos correos el mismo día es lo que esto impide"),
        }
    }

    /// **La frontera exacta sale, y el segundo anterior espera.**
    ///
    /// Es el único par que distingue `>=` de `>`. Escrito con los dos lados
    /// porque cambiar el operador no hace fallar nada más: adelanta o retrasa
    /// exactamente un segundo, y eso no se ve en ninguna otra prueba.
    #[test]
    fn la_frontera_del_descanso_se_mide_en_los_dos_lados() {
        assert_eq!(
            puede_salir(14 * UN_DIA, DESCANSO_ENTRE_CAMPANAS_DIAS),
            Descanso::PuedeSalir,
            "a los catorce días exactos ya se puede"
        );
        assert_eq!(
            puede_salir(14 * UN_DIA - 1, DESCANSO_ENTRE_CAMPANAS_DIAS),
            Descanso::Espera { segundos: 1 },
            "un segundo antes, todavía no"
        );
    }

    /// **Un descanso por debajo del mínimo se sube, no se obedece.**
    ///
    /// La misma regla que el corte del archivado: si algún día hay un campo en
    /// la pantalla, la frontera no se fía de lo que le manden.
    #[test]
    fn un_descanso_demasiado_corto_se_sube_al_minimo() {
        // Piden 1 día; con 6 días transcurridos, el mínimo de 7 todavía manda.
        assert_eq!(
            puede_salir(6 * UN_DIA, 1),
            Descanso::Espera { segundos: UN_DIA },
            "un día de descanso no vale: el suelo son siete"
        );
    }

    /// **Un reloj que va hacia atrás no da permiso para escribir.**
    ///
    /// Pasa de verdad: el cambio de hora, o alguien ajustando el reloj del
    /// equipo. Con un transcurrido negativo y una resta sin proteger, el
    /// descanso saldría **más largo** —inofensivo— pero con la comparación al
    /// revés daría `PuedeSalir`, que es escribir dos veces el mismo día.
    #[test]
    fn un_reloj_hacia_atras_no_deja_escribir() {
        assert_eq!(
            puede_salir(-3_600, DESCANSO_ENTRE_CAMPANAS_DIAS),
            Descanso::Espera {
                segundos: 14 * UN_DIA
            }
        );
    }
}
