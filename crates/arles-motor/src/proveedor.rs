//! La frontera con el mundo: quién entrega un mensaje de verdad.
//!
//! El motor no sabe nada de SMTP ni de WhatsApp. Sabe pedir «entrega esto» y
//! entender cinco respuestas. Los adaptadores reales llegan en la Fase 5; hasta
//! entonces el único que existe es el simulador, y **toda la Fase 4 se
//! construye contra él**: al terminarla ARLES sabrá enviar perfectamente y no
//! habrá mandado ni un correo real.

use arles_core::reintento::Fallo;
use arles_core::texto::engana_al_que_mira;

/// Lo que se le pide al proveedor que entregue.
///
/// ─────────────────────────────────────────────────────────────────────────
/// EL CONTENIDO VIAJA AQUÍ, Y ANTES NO VIAJABA EN NINGUNA PARTE
///
/// Hasta el 20/10/2026 esta estructura llevaba sólo la dirección y la clave:
/// el asunto y el cuerpo los guardaba **el adaptador**, uno para toda la
/// campaña. Como una campaña no tiene un texto, sino un texto por persona,
/// `arles-app` le pasaba uno vacío y **ninguna campaña mandaba el cuerpo de su
/// plantilla** (H-15).
///
/// Ahora el contenido va por mensaje porque es lo que cambia en cada mensaje.
/// Lo redacta `arles_core::plantilla::redactar` antes de llamar aquí, con los
/// datos de esa persona ya escapados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mensaje<'a> {
    /// La dirección de destino, ya validada y normalizada por el núcleo.
    pub para: &'a str,
    /// El asunto, **ya con las variables puestas**.
    pub asunto: &'a str,
    /// El cuerpo en HTML, ya redactado para esta persona y con su firma.
    ///
    /// La versión en texto plano no viaja: la deriva el adaptador del mismo
    /// HTML. Mandar las dos desde aquí daría dos sitios donde pueden dejar de
    /// decir lo mismo.
    pub html: &'a str,
    /// ¿Este mensaje sale de una campaña, o es un envío suelto?
    ///
    /// ── LO QUE DECIDE: SI LLEVA LA CABECERA DE BAJA ───────────────────────
    ///
    /// `List-Unsubscribe` es obligatoria para quien manda en volumen, y **es
    /// también una declaración**: le dice al filtro «esto es correo de una
    /// lista». Para una campaña es lo correcto y suma. Para el envío de
    /// prueba —un mensaje suelto que alguien se manda a sí mismo— es falso y
    /// **resta**: lo clasifica como masivo, y a un correo masivo los filtros
    /// le exigen una reputación de dominio que una cuenta personal no tiene.
    ///
    /// Se puso siempre al escribir la cabecera (19/10/2026) y se corrigió al
    /// medir el correo entero (20/10/2026). Una prueba de conexión no
    /// pertenece a ninguna lista de la que darse de baja.
    pub de_una_campana: bool,
    /// El `Message-Id` del correo.
    ///
    /// Se generó y se guardó **antes** de esta llamada. Viaja con el mensaje
    /// para dar trazabilidad, y para poder deduplicar del lado del proveedor si
    /// algún día tenemos visibilidad allí (`MOTOR_DE_EJECUCION.md` §2, capa 3).
    pub clave_de_idempotencia: &'a str,
}

/// El proveedor se hizo cargo del mensaje.
///
/// **No significa «entregado»** (§65). Significa que aceptó la
/// responsabilidad de intentarlo. Si acaba en la bandeja, en spam o rebotando,
/// ARLES no lo sabe — y la interfaz no puede decir que lo sabe.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Aceptado {
    /// El identificador que devolvió el proveedor, si devolvió alguno.
    ///
    /// Va recortado por [`recortar_respuesta`]: es texto que viene de fuera.
    pub id_del_proveedor: Option<String>,
}

/// Quien sabe entregar un mensaje.
///
/// `Send + Sync` porque el motor corre en su propio hilo, aparte de la ventana
/// (§51): cerrar la ventana no detiene una campaña.
///
/// **Síncrono a propósito.** Ver la nota del `lib.rs`: a cincuenta correos
/// diarios la asincronía no aporta nada y complica cada prueba.
pub trait Proveedor: Send + Sync {
    /// Entrega el mensaje, o dice por qué no pudo.
    ///
    /// # Errores
    ///
    /// Un [`Fallo`], que el motor traduce a una decisión con
    /// `arles_core::reintento::que_hacer`. Los códigos concretos de cada
    /// proveedor —un `550` de SMTP, un `131049` de Meta— se traducen **aquí**,
    /// en cada adaptador: el núcleo no tiene por qué conocerlos.
    fn enviar(&self, mensaje: &Mensaje<'_>) -> Result<Aceptado, Fallo>;
}

/// Cuánto texto del proveedor se guarda.
///
/// Quinientos caracteres bastan para diagnosticar cualquier respuesta real de
/// SMTP, que rara vez pasa de dos líneas.
pub const RESPUESTA_MAXIMA: usize = 500;

/// Deja una respuesta del proveedor en algo seguro de guardar y de enseñar.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTO ES TEXTO DE FUERA, Y SE TRATA COMO TAL
///
/// Es el caso «el servidor devuelve 5 000 caracteres de basura» de
/// `ESTRATEGIA_QA.md` §3: **se trunca, no se interpola en ningún sitio, y no
/// se cae**. Un servidor de correo mal configurado, o uno hostil, contesta lo
/// que le da la gana.
///
/// Tres cosas y por qué cada una:
///
/// **Se recorta.** Sin tope, una respuesta de cinco mil caracteres acaba en la
/// base de datos y después en una pantalla, empujando todo lo demás fuera de
/// la vista. Multiplicado por los intentos de una campaña, engorda el archivo
/// sin aportar nada: la información útil está en las dos primeras líneas.
///
/// **Se recorta por CARACTERES, no por bytes.** Cortar por bytes puede partir
/// una letra acentuada por la mitad y dejar una cadena que ya no es texto
/// válido. Una respuesta en español lo tiene garantizado.
///
/// **Se quitan los caracteres que engañan.** Un `\r` de más rompe el formato de
/// la bitácora; un carácter de cambio de dirección de escritura —`U+202E`—
/// **dibuja el texto al revés** y puede hacer que un mensaje de error parezca
/// otra cosa. Es el mismo ataque que se encontró en los nombres de archivo de
/// la importación, y por eso la lista es **la misma**:
/// `arles_core::texto::engana_al_que_mira`.
///
/// Ojo con lo que parece bastar y no basta: `U+202E` **no es un carácter de
/// control** —Unicode lo clasifica como de formato—, así que un filtro escrito
/// con `is_control` lo deja pasar. La primera versión de esta función lo hacía,
/// y la prueba del texto al revés la tumbó.
///
/// Los saltos de línea sí se conservan: son la estructura de una respuesta
/// SMTP.
#[must_use]
pub fn recortar_respuesta(bruto: &str) -> String {
    bruto
        .chars()
        .filter(|c| *c == '\n' || !engana_al_que_mira(*c))
        .take(RESPUESTA_MAXIMA)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cinco mil caracteres de basura se quedan en quinientos.
    ///
    /// Es el caso literal de `ESTRATEGIA_QA.md` §3.
    #[test]
    fn cinco_mil_caracteres_de_basura_se_recortan() {
        let basura = "x".repeat(5_000);
        let limpio = recortar_respuesta(&basura);
        assert_eq!(limpio.chars().count(), RESPUESTA_MAXIMA);
    }

    /// Se recorta por caracteres, no por bytes.
    ///
    /// Cortar por bytes partiría una `ñ` por la mitad y dejaría algo que ya no
    /// es texto. En español eso no es un caso raro: es el caso normal.
    #[test]
    fn se_recorta_por_caracteres_y_no_parte_los_acentos() {
        let con_acentos = "ñ".repeat(5_000);
        let limpio = recortar_respuesta(&con_acentos);
        assert_eq!(limpio.chars().count(), RESPUESTA_MAXIMA);
        assert!(
            limpio.chars().all(|c| c == 'ñ'),
            "alguna letra se partió por la mitad"
        );
    }

    /// **El carácter que dibuja el texto al revés no pasa.**
    ///
    /// `U+202E` invierte la dirección de escritura de todo lo que viene
    /// después. Un servidor hostil puede usarlo para que un mensaje de error
    /// parezca decir algo distinto de lo que dice.
    ///
    /// Es el mismo ataque que apareció en los nombres de archivo de la
    /// importación en la entrega 3.3, y por eso se vigila aquí también: todo
    /// texto que viene de fuera lo tiene.
    #[test]
    fn el_caracter_que_invierte_el_texto_no_pasa() {
        let hostil = "550 error\u{202E}odaivne on";
        let limpio = recortar_respuesta(hostil);
        assert!(
            !limpio.contains('\u{202E}'),
            "un servidor no puede dibujar su respuesta al revés en nuestra pantalla"
        );
        assert!(limpio.starts_with("550 error"));
    }

    /// Los saltos de línea se conservan: son la forma de una respuesta SMTP.
    #[test]
    fn los_saltos_de_linea_sobreviven_y_los_retornos_no() {
        let respuesta = "250-OK\r\n250 Listo";
        let limpio = recortar_respuesta(respuesta);
        assert_eq!(limpio, "250-OK\n250 Listo");
    }

    /// Una respuesta normal no se toca.
    #[test]
    fn una_respuesta_normal_pasa_entera() {
        let normal = "250 2.0.0 Ok: queued as 4X7v2k1Qm";
        assert_eq!(recortar_respuesta(normal), normal);
    }

    /// Recortar una cadena vacía no revienta.
    #[test]
    fn una_respuesta_vacia_no_revienta() {
        assert_eq!(recortar_respuesta(""), "");
    }
}
