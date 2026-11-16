//! Cómo se escribe una fecha en la base, y por qué la forma importa.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA BASE COMPARA FECHAS COMO SI FUERAN PALABRAS
//!
//! SQLite no tiene tipo de fecha. Las fechas se guardan como texto, y las
//! consultas de la cola las comparan **letra a letra**:
//!
//! ```text
//! WHERE next_retry_at <= '2026-09-23T12:05:00Z'
//! ```
//!
//! Eso funciona —y es la razón de usar RFC 3339 y no otra cosa— sólo mientras
//! **todas** las fechas se escriban con la misma forma. Dos formas distintas
//! del mismo instante dejan de compararse bien, y el fallo es silencioso.
//!
//! El caso concreto que hay que evitar es la fracción de segundo:
//!
//! ```text
//! "2026-09-23T12:05:00.123Z"   contra   "2026-09-23T12:05:00Z"
//! ```
//!
//! El punto vale menos que la letra Z en la tabla de caracteres, así que el
//! instante **con** fracción se ordena **antes** que el mismo instante sin
//! ella. Un reintento programado para las 12:05:00 en punto se consideraría
//! vencido a las 12:05:00.123 — y, al revés, uno guardado con fracción no
//! vencería a su hora en punto. Es un segundo de diferencia, y nada falla:
//! simplemente sale antes o después de lo que dice la pantalla.
//!
//! Por eso el motor escribe sus fechas **por aquí y sólo por aquí**: UTC,
//! segundos enteros, siempre la misma longitud.

use time::UtcOffset;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

/// RFC 3339 en UTC, con segundos enteros. Siempre veinte caracteres.
const FORMATO: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

/// Cuántos caracteres ocupa un sello. Todos ocupan lo mismo: es lo que hace
/// que compararlos como texto equivalga a compararlos como instantes.
pub const LARGO_DEL_SELLO: usize = 20;

/// Escribe un instante tal y como lo guarda la base.
///
/// Si el formateo fallara —no puede, con este formato y un `OffsetDateTime`—
/// devuelve el instante cero en vez de un texto a medias: un sello mal formado
/// se compararía mal, y eso es peor que uno visiblemente absurdo.
#[must_use]
pub fn sello(instante: time::OffsetDateTime) -> String {
    instante
        .to_offset(UtcOffset::UTC)
        .format(FORMATO)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    /// **La fracción de segundo rompe la comparación de la cola.**
    ///
    /// Esta prueba no comprueba nuestro código: comprueba el motivo por el que
    /// existe [`sello`]. Si alguien decide que la fracción «da más precisión»,
    /// aquí está por escrito lo que cuesta.
    #[test]
    fn un_instante_con_fraccion_se_ordena_antes_que_el_mismo_sin_ella() {
        assert!(
            "2026-09-23T12:05:00.123Z" < "2026-09-23T12:05:00Z",
            "el punto vale menos que la Z, así que el MISMO instante con \
             fracción se considera ANTERIOR. Mezclar las dos formas hace que \
             los reintentos salgan un segundo antes o después de su hora"
        );
    }

    #[test]
    fn el_sello_no_lleva_fraccion_y_mide_siempre_lo_mismo() {
        let con_nanos = datetime!(2026-09-23 12:05:00.987654321 UTC);
        let s = sello(con_nanos);
        assert_eq!(s, "2026-09-23T12:05:00Z");
        assert_eq!(s.len(), LARGO_DEL_SELLO);
    }

    /// Un instante en otro huso se guarda en UTC, no con su desfase.
    ///
    /// Guardarlo con el desfase daría textos de longitudes y prefijos
    /// distintos para el mismo momento, y la comparación literal volvería a
    /// mentir: «2026-09-23T07:05:00-06:00» es el mismo instante que
    /// «2026-09-23T13:05:00Z» y se ordena seis horas antes.
    #[test]
    fn un_instante_de_mexico_se_guarda_en_utc() {
        let en_mexico = datetime!(2026-09-23 07:05:00 -6);
        assert_eq!(sello(en_mexico), "2026-09-23T13:05:00Z");
    }

    /// El orden del texto tiene que ser el orden del tiempo. Ese es el trato.
    #[test]
    fn comparar_sellos_como_texto_es_comparar_instantes() {
        let antes = sello(datetime!(2026-09-23 12:04:59 UTC));
        let despues = sello(datetime!(2026-09-23 12:05:00 UTC));
        assert!(antes < despues);

        // Y a través del cambio de año, que es donde fallan los formatos
        // caseros.
        let fin = sello(datetime!(2026-12-31 23:59:59 UTC));
        let inicio = sello(datetime!(2027-01-01 00:00:00 UTC));
        assert!(fin < inicio);
    }
}
