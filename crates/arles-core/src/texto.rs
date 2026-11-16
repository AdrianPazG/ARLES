//! Texto que viene de fuera, y cómo dejarlo seguro de enseñar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL ATAQUE QUE ESTO IMPIDE
//!
//! Unicode tiene caracteres que **no se dibujan**, pero cambian cómo se dibuja
//! todo lo que viene detrás. El más conocido es `U+202E`, que da la vuelta al
//! texto:
//!
//! ```text
//! factura<U+202E>gnp.exe   se ve en pantalla como   facturaexe.png
//! ```
//!
//! Quien mira lee una cosa y tiene otra. Es el truco con el que se distribuye
//! software disfrazado de imagen, y es legal en un nombre de archivo, en una
//! respuesta de servidor y en un campo de un CSV.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO VIVE EN EL NÚCLEO Y NO EN CADA SITIO
//!
//! La lista estaba escrita en `arles-import`, para los nombres de archivo de la
//! entrega 3.3. Al llegar el motor hizo falta otra vez, para las respuestas de
//! los proveedores — y copiarla habría dejado **dos listas que se separan**:
//! alguien añade un carácter nuevo a una, nadie se acuerda de la otra, y el
//! agujero reaparece en el sitio que no se tocó.
//!
//! Una lista, un sitio, y las dos la usan. Es una regla sobre texto, sin I/O:
//! el núcleo es su lugar.

/// Caracteres que no se dibujan pero cambian cómo se dibuja lo demás.
///
/// Se enumeran a mano en vez de tirar de una tabla de categorías de Unicode:
/// son pocos, no cambian, y una lista escrita se puede leer y discutir. Los dos
/// primeros bloques son las marcas y anulaciones bidireccionales; el resto, los
/// de ancho cero y el espacio que no se ve.
#[must_use]
pub const fn es_de_formato(c: char) -> bool {
    matches!(c,
        '\u{200B}'..='\u{200F}'   // ancho cero, marcas de dirección
        | '\u{202A}'..='\u{202E}' // incrustaciones y anulaciones bidireccionales
        | '\u{2066}'..='\u{2069}' // aislamientos bidireccionales
        | '\u{00A0}'              // espacio que no se parte ni se ve distinto
        | '\u{FEFF}'              // marca de orden de bytes suelta
    )
}

/// ¿Este carácter puede engañar a quien mire la pantalla?
///
/// Junta los dos casos: los de control —que rompen el formato de una bitácora
/// o de una tabla— y los de formato, que disfrazan sin verse.
///
/// **`is_control` no basta**, y es fácil creer que sí. `U+202E` no es un
/// carácter de control: Unicode lo clasifica como de **formato**, así que
/// `char::is_control()` devuelve `false` y lo deja pasar. Se descubrió el
/// 23/09/2026 al escribir el recorte de respuestas del motor: la prueba del
/// texto al revés falló con un filtro que sólo miraba `is_control`.
///
/// No es `const`: `char::is_control` no lo es todavía. `es_de_formato` sí, y se
/// deja así para que la lista se pueda usar en un contexto constante.
#[must_use]
pub fn engana_al_que_mira(c: char) -> bool {
    c.is_control() || es_de_formato(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`is_control` no caza el carácter que da la vuelta al texto.**
    ///
    /// Es el motivo de que exista `engana_al_que_mira`, y la prueba que lo
    /// documenta: si alguien «simplifica» el filtro a `is_control`, esto falla
    /// y dice por qué.
    #[test]
    fn el_caracter_que_da_la_vuelta_al_texto_no_es_de_control() {
        assert!(
            !'\u{202E}'.is_control(),
            "Unicode clasifica U+202E como de FORMATO, no de control. Filtrar \
             sólo por `is_control` lo deja pasar, y con él el texto al revés"
        );
        assert!(es_de_formato('\u{202E}'));
        assert!(engana_al_que_mira('\u{202E}'));
    }

    /// Los cuatro bloques de la lista, con un representante cada uno.
    #[test]
    fn los_cuatro_bloques_de_la_lista_se_cazan() {
        for c in [
            '\u{200B}', // ancho cero
            '\u{200E}', // marca de dirección
            '\u{202A}', // incrustación bidireccional
            '\u{2066}', // aislamiento bidireccional
            '\u{00A0}', // espacio que no se ve distinto
            '\u{FEFF}', // marca de orden de bytes
        ] {
            assert!(
                es_de_formato(c),
                "U+{:04X} debería estar en la lista",
                c as u32
            );
        }
    }

    /// El texto normal no se toca. Los acentos y la eñe **no** engañan a nadie.
    #[test]
    fn el_texto_normal_pasa_entero() {
        for c in "Señor Peña — año 2026, 50 % (ok)".chars() {
            assert!(
                !engana_al_que_mira(c),
                "«{c}» es texto corriente y no puede filtrarse"
            );
        }
    }

    /// Los de control sí, incluido el retorno de carro.
    #[test]
    fn los_de_control_tambien_se_cazan() {
        assert!(engana_al_que_mira('\r'));
        assert!(engana_al_que_mira('\u{0000}'));
        assert!(
            engana_al_que_mira('\n'),
            "el salto también: quien lo quiera, lo exime"
        );
    }
}
