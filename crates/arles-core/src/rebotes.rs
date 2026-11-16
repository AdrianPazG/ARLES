//! El umbral de rebotes · R-3, decidido por Dirección el 26/10/2026.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ PROBLEMA RESUELVE, Y POR QUÉ NO LO RESUELVE EL DISYUNTOR
//!
//! Una dirección que no existe devuelve un `550`. ARLES ya la da de baja en el
//! acto, y eso funciona desde la Fase 4. Lo que no hacía era **mirar cuántas
//! van**.
//!
//! Muchos rebotes duros seguidos le dicen al filtro de quien recibe «éste no
//! sabe a quién le escribe», que es la firma de una lista comprada. El precio
//! no lo paga la campaña: lo paga **el dominio**, durante meses, y no hay
//! forma de retirarlo.
//!
//! El disyuntor (§7) no sirve para esto, y es correcto que no sirva: cuenta
//! fallos **del canal** —red, saturación, credencial— y excluye los del
//! destinatario a propósito, porque «una lista importada con cinco direcciones
//! muertas seguidas no dice nada malo del remitente». Eso sigue siendo verdad
//! de cinco seguidas. Deja de serlo del 5 % de mil.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS TRES DECISIONES DE DIRECCIÓN, Y LO QUE CUESTA CADA UNA
//!
//! **1 · Dos escalones y no uno.** Al 2 % se avisa y la campaña sigue; al 5 %
//! se frena. `RIESGOS-DE-REPUTACION.md` tenía las dos cifras y se contradecía
//! —el título de R-3 decía 2 %, su mitigación 5 %—; ahora cada una tiene su
//! papel. El aviso existe para poder mirar **antes** de que algo pare.
//!
//! **2 · Se PAUSA, no se detiene.** Detener es irreversible en ARLES: cancela
//! lo que no ha salido, y hoy exige escribir el nombre de la campaña a mano
//! justamente para que nadie lo haga sin querer. Una acción irreversible
//! tomada por un umbral, sin preguntar, sería la única del producto. La pausa
//! se deshace: ARLES frena y **la decisión sigue siendo de quien mira**.
//!
//! **3 · Muestra mínima de 50.** Un porcentaje sobre pocos envíos es ruido:
//! con diez, un solo rebote ya es el 10 %, y parar ahí sería parar por una
//! dirección mal escrita.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! «DE CUÁNTOS» ES LO QUE MÁS FÁCIL SERÍA EQUIVOCAR
//!
//! El denominador son **los que ya se intentaron**, no la audiencia de la
//! campaña. Con la audiencia entera debajo, una campaña de 10 000 con 400
//! rebotes de los primeros 420 envíos daría un 4 % —por debajo del freno—
//! cuando lo que está pasando es que **casi todo lo que sale rebota**.
//!
//! Es el mismo error que haría inútil la regla entera, y no fallaría nada: la
//! cifra saldría, sería baja, y la campaña seguiría quemando el dominio.

use serde::Serialize;

/// A partir de aquí se avisa, en tanto por ciento. La campaña **sigue**.
pub const AVISO_POR_CIENTO: u32 = 2;

/// A partir de aquí se frena, en tanto por ciento.
pub const FRENO_POR_CIENTO: u32 = 5;

/// Cuántos envíos hacen falta antes de que un porcentaje signifique algo.
///
/// Cincuenta. Por debajo no se avisa ni se frena, pase lo que pase: con diez
/// envíos un solo rebote es el 10 %, y el 10 % de diez no dice nada de una
/// lista de cinco mil.
pub const MUESTRA_MINIMA: u32 = 50;

/// Qué pasa con los rebotes de una campaña.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "estado")]
pub enum Rebotes {
    /// Dentro de lo normal, o todavía sin muestra suficiente.
    Bien,
    /// Por encima del aviso. **La campaña sigue enviando.**
    Aviso {
        duros: u32,
        de: u32,
        /// Redondeado hacia arriba para enseñar. La **decisión** no sale de
        /// aquí: se toma con enteros exactos (ver [`mirar`]).
        por_ciento: u32,
    },
    /// Por encima del freno. **La campaña se pausa.**
    Frena {
        duros: u32,
        de: u32,
        por_ciento: u32,
    },
}

impl Rebotes {
    /// ¿Hay que pausar la campaña?
    #[must_use]
    pub const fn hay_que_frenar(self) -> bool {
        matches!(self, Self::Frena { .. })
    }

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Bien => "rebotes.bien",
            Self::Aviso { .. } => "rebotes.aviso",
            Self::Frena { .. } => "rebotes.frena",
        }
    }
}

/// El porcentaje **para enseñar**, redondeado hacia arriba.
///
/// Hacia arriba y no al más cercano: quedarse corto en una cifra de riesgo es
/// el único error que importa aquí. Un 4.6 % que se enseña como «4 %» hace
/// pensar que queda margen hasta el 5 cuando ya casi no queda.
const fn para_ensenar(duros: u32, de: u32) -> u32 {
    if de == 0 {
        return 0;
    }
    duros.saturating_mul(100).div_ceil(de)
}

/// ¿Cómo van los rebotes de esta campaña?
///
/// `duros` son los rebotes definitivos de dirección inexistente. `intentados`
/// son **los envíos ya resueltos** —salieron o rebotaron—, nunca la audiencia
/// entera: ver la cabecera, es el error que haría inútil la regla.
///
/// La comparación es con enteros y sin dividir: `duros * 100 >= umbral * de`.
/// Con coma flotante, un 5 % exacto podría caer del lado de abajo según cómo
/// se redondee, y el borde es justo donde esto tiene que acertar.
#[must_use]
pub const fn mirar(duros: u32, intentados: u32) -> Rebotes {
    // Sin muestra no hay juicio. Va primero: con `intentados` pequeño
    // cualquier umbral se dispara, y ése es el caso que esto evita.
    if intentados < MUESTRA_MINIMA {
        return Rebotes::Bien;
    }

    let cien = duros.saturating_mul(100);
    let por_ciento = para_ensenar(duros, intentados);

    if cien >= FRENO_POR_CIENTO.saturating_mul(intentados) {
        Rebotes::Frena {
            duros,
            de: intentados,
            por_ciento,
        }
    } else if cien >= AVISO_POR_CIENTO.saturating_mul(intentados) {
        Rebotes::Aviso {
            duros,
            de: intentados,
            por_ciento,
        }
    } else {
        Rebotes::Bien
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Por debajo de la muestra mínima no se frena nunca**, aunque el
    /// porcentaje sea escandaloso.
    ///
    /// Es el caso que motiva `MUESTRA_MINIMA`: cuatro rebotes de diez es el
    /// 40 %, y no dice nada de una lista de cinco mil. Parar ahí sería parar
    /// por una dirección mal escrita.
    #[test]
    fn con_pocos_envios_no_se_juzga() {
        assert_eq!(mirar(4, 10), Rebotes::Bien);
        assert_eq!(mirar(49, 49), Rebotes::Bien, "ni con todo rebotando");
    }

    /// **La frontera de la muestra, en los dos lados.**
    ///
    /// Es el único par que distingue `<` de `<=` en `MUESTRA_MINIMA`, y
    /// cambiar ese operador no hace fallar nada más.
    #[test]
    fn la_muestra_minima_se_mide_en_los_dos_lados() {
        // 3 de 49 es el 6 %, pero no hay muestra.
        assert_eq!(mirar(3, 49), Rebotes::Bien);
        // 3 de 50 es el 6 %, y ya hay muestra.
        assert!(mirar(3, 50).hay_que_frenar());
    }

    /// **El 2 % exacto avisa y no frena.**
    ///
    /// Un rebote de cincuenta es exactamente el 2 %. Con `>` en vez de `>=` no
    /// avisaría, y con el umbral del freno mal puesto pararía: las dos cosas
    /// se comprueban en la misma prueba porque el borde es el mismo.
    #[test]
    fn el_dos_por_ciento_exacto_avisa_sin_frenar() {
        let r = mirar(1, 50);
        assert_eq!(
            r,
            Rebotes::Aviso {
                duros: 1,
                de: 50,
                por_ciento: 2
            }
        );
        assert!(!r.hay_que_frenar(), "el aviso deja seguir enviando");
    }

    /// **El 5 % exacto frena.**
    ///
    /// Cinco de cien. Es el borde que decide si el freno es `>=` o `>`, y
    /// equivocarlo deja pasar justo el caso para el que se construyó.
    #[test]
    fn el_cinco_por_ciento_exacto_frena() {
        let r = mirar(5, 100);
        assert!(r.hay_que_frenar());
        assert_eq!(
            r,
            Rebotes::Frena {
                duros: 5,
                de: 100,
                por_ciento: 5
            }
        );
    }

    /// Justo por debajo de cada umbral no pasa nada todavía.
    #[test]
    fn justo_por_debajo_de_cada_umbral_no_salta() {
        // 1 de 100 es el 1 %: ni aviso.
        assert_eq!(mirar(1, 100), Rebotes::Bien);
        // 4 de 100 es el 4 %: avisa, no frena.
        assert!(matches!(mirar(4, 100), Rebotes::Aviso { .. }));
    }

    /// **El porcentaje que se enseña redondea HACIA ARRIBA.**
    ///
    /// 46 de 1 000 es el 4.6 %. Enseñado como «4 %» hace pensar que queda un
    /// punto entero de margen cuando queda menos de medio. En una cifra de
    /// riesgo, quedarse corto es el único error que importa.
    #[test]
    fn el_porcentaje_que_se_ensena_no_se_queda_corto() {
        match mirar(46, 1_000) {
            Rebotes::Aviso { por_ciento, .. } => assert_eq!(por_ciento, 5),
            otro => panic!("tenía que avisar: {otro:?}"),
        }
    }

    /// **Una campaña limpia no avisa de nada**, por grande que sea.
    #[test]
    fn una_campana_sana_no_molesta() {
        assert_eq!(mirar(0, 10_000), Rebotes::Bien);
        assert_eq!(mirar(9, 1_000), Rebotes::Bien, "0.9 %");
    }

    /// **Medio millón de envíos no desborda la cuenta.**
    ///
    /// `duros * 100` con el techo declarado del producto —500 000— pasa de
    /// cincuenta millones, que cabe de sobra en `u32`; pero la multiplicación
    /// está saturada y esto lo deja escrito. Sin ella, un desbordamiento daría
    /// un número pequeño y **el freno dejaría de dispararse** justo en la
    /// campaña más grande.
    #[test]
    fn con_el_techo_del_producto_la_cuenta_no_desborda() {
        let r = mirar(25_000, 500_000);
        assert!(r.hay_que_frenar(), "25 000 de 500 000 es el 5 %");
    }
}
