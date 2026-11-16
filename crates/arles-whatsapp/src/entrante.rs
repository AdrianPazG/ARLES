//! Lo que Meta manda de vuelta: respuestas y cambios de calidad.
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! LEE ESTO ANTES DE USAR NADA DE AQUÍ
//!
//! **ARLES no tiene por dónde recibir un aviso de Meta, y esto no lo arregla.**
//!
//! Los avisos de Cloud API son *webhooks*: Meta llama a una URL pública con
//! HTTPS. ARLES es una aplicación de escritorio de un solo proceso, sin ningún
//! componente de red propio (`ARQUITECTURA.md` §11), así que **no hay ninguna
//! URL a la que Meta pueda llamar**.
//!
//! `LOGISTICA_DE_CAMPANAS.md` da por hecho que los avisos llegan —para contar
//! respuestas, para bajar el ritmo cuando cae la calidad y para parar el canal
//! ante una restricción—. Es una contradicción del diseño, del mismo tipo que
//! la de §37 contra §69 en los rebotes de correo, y está pendiente de una
//! decisión de Dirección.
//!
//! Lo que hay en este archivo es la pieza que hará falta **venga por donde
//! venga** el aviso: verificar que de verdad lo manda Meta, sacar lo mínimo, y
//! tirar el resto. Escribirla ahora no es adelantarse: es la parte con reglas
//! de privacidad y de seguridad, y es la que no conviene hacer con prisa el día
//! que se decida el transporte.
//!
//! **Lo que sí funciona hoy desde un escritorio está en [`crate::estado`]**:
//! preguntarle a Meta por el número en vez de esperar a que Meta llame. Eso
//! cubre el semáforo de calidad sin necesidad de servidor.
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! EL TEXTO DEL MENSAJE SE TIRA. AQUÍ. ANTES DE QUE LLEGUE A NADIE.
//!
//! Dirección decidió el 29/09/2026 **contar** las respuestas y no guardarlas.
//! [`Respuesta`] no tiene dónde meter un texto, y la V9 tampoco tiene columna
//! donde quepa.
//!
//! El sitio donde se descarta importa: si el texto viajara hasta `arles-db` y
//! allí se ignorara, pasaría por una capa más —y por cualquier traza, informe
//! de error o volcado que haya en medio—. Aquí muere en la misma función que lo
//! lee.

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Por qué un aviso no se acepta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AvisoRechazado {
    /// La firma no corresponde al cuerpo, o no venía firma.
    ///
    /// **Es lo único que separa un aviso de Meta de uno que se inventó
    /// cualquiera.** Un aviso falso podría decir que un número contestó —lo que
    /// falsea la métrica— o que la calidad se hundió, lo que pararía el canal.
    #[error("la firma del aviso no corresponde")]
    FirmaQueNoCuadra,
    /// El cuerpo no tiene la forma que Meta manda.
    #[error("el aviso no tiene la forma esperada")]
    FormaDesconocida,
}

/// Una respuesta de alguien, **sin lo que dijo**.
///
/// Tiene dos campos y ninguno es el mensaje. Ver la nota de la cabecera: es la
/// decisión de Dirección hecha tipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Respuesta {
    /// El número que contestó, en la forma que manda Meta (sin el `+`).
    ///
    /// Es el único dato personal que sale de aquí, y hace falta para dos cosas:
    /// no contar dos veces a la misma persona, y poder borrarlo si ejerce su
    /// derecho de cancelación.
    pub de: String,
    /// Cuándo, según Meta. Segundos desde 1970.
    pub cuando: u64,
}

/// Lo que ARLES entiende de un aviso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aviso {
    /// Alguien contestó. Se cuenta; no se guarda lo que dijo.
    Contestaron(Vec<Respuesta>),
    /// Meta cambió la calificación de calidad del número.
    CalidadNueva {
        /// `GREEN`, `YELLOW`, `RED` o `UNKNOWN`, tal como lo manda Meta.
        calidad: String,
    },
    /// Un aviso de un tipo que ARLES todavía no usa.
    ///
    /// **No es un error**: Meta manda avisos de cosas que no hemos pedido, y
    /// tratarlos como error llenaría la pantalla de fallos que no lo son. Se
    /// reconocen y se ignoran.
    DeAlgoQueNoUsamos,
}

/// Comprueba que el aviso viene de Meta, y lo traduce.
///
/// `firma` es el valor de la cabecera `X-Hub-Signature-256`, con su prefijo
/// `sha256=`. `secreto` es el de la aplicación de Meta, que vive en el llavero.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA FIRMA SE COMPRUEBA ANTES DE MIRAR EL CUERPO, Y NO AL REVÉS
///
/// El orden no es estético. El cuerpo es texto de fuera; analizarlo primero
/// significaría procesar lo que mande cualquiera que descubra la dirección, y
/// sólo después preguntarse si tenía derecho a mandarlo.
///
/// # Errores
///
/// [`AvisoRechazado::FirmaQueNoCuadra`] si no la manda Meta.
/// [`AvisoRechazado::FormaDesconocida`] si el cuerpo no se parece a un aviso.
pub fn leer(cuerpo: &[u8], firma: &str, secreto: &[u8]) -> Result<Aviso, AvisoRechazado> {
    if !la_firma_cuadra(cuerpo, firma, secreto) {
        return Err(AvisoRechazado::FirmaQueNoCuadra);
    }

    let texto = std::str::from_utf8(cuerpo).map_err(|_| AvisoRechazado::FormaDesconocida)?;

    if !texto.contains("\"object\":\"whatsapp_business_account\"") {
        return Err(AvisoRechazado::FormaDesconocida);
    }

    if texto.contains("\"field\":\"phone_number_quality_update\"") {
        return Ok(Aviso::CalidadNueva {
            calidad: entre(texto, "\"current_limit\":\"", "\"")
                .or_else(|| entre(texto, "\"event\":\"", "\""))
                .unwrap_or_else(|| "UNKNOWN".to_owned()),
        });
    }

    if texto.contains("\"messages\":[") {
        let respuestas = quien_contesto(texto);
        if respuestas.is_empty() {
            // Un aviso de mensajes sin ningún remitente legible: se ignora en
            // vez de contar una respuesta fantasma. La métrica que protege al
            // número tiene que poder confiarse.
            return Ok(Aviso::DeAlgoQueNoUsamos);
        }
        return Ok(Aviso::Contestaron(respuestas));
    }

    Ok(Aviso::DeAlgoQueNoUsamos)
}

/// HMAC-SHA256 del cuerpo contra el secreto, comparado **en tiempo constante**.
///
/// La comparación la hace `verify_slice` de la propia biblioteca. Un `==` sobre
/// los bytes se para en la primera diferencia, y ese tiempo —medible a lo largo
/// de muchos intentos— deja adivinar la firma byte a byte.
fn la_firma_cuadra(cuerpo: &[u8], firma: &str, secreto: &[u8]) -> bool {
    let Some(hex) = firma.strip_prefix("sha256=") else {
        return false;
    };

    let Some(esperada) = de_hex(hex) else {
        return false;
    };

    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secreto) else {
        return false;
    };
    mac.update(cuerpo);
    mac.verify_slice(&esperada).is_ok()
}

fn de_hex(texto: &str) -> Option<Vec<u8>> {
    if !texto.len().is_multiple_of(2) {
        return None;
    }
    let bytes = texto.as_bytes();
    let mut salida = Vec::with_capacity(texto.len() / 2);
    for par in bytes.chunks(2) {
        let s = std::str::from_utf8(par).ok()?;
        salida.push(u8::from_str_radix(s, 16).ok()?);
    }
    Some(salida)
}

/// Saca de quién viene cada mensaje, **y nada más**.
///
/// Recorre los `"from":"…"` y sus `"timestamp":"…"`. No toca `"text"`, ni
/// `"body"`, ni ningún campo con contenido: ver la nota de la cabecera.
fn quien_contesto(texto: &str) -> Vec<Respuesta> {
    let mut salida = Vec::new();
    let mut resto = texto;

    while let Some(pos) = resto.find("\"from\":\"") {
        let Some(tras_marca) = resto.get(pos + "\"from\":\"".len()..) else {
            break;
        };
        let Some(fin) = tras_marca.find('"') else {
            break;
        };
        let Some(de) = tras_marca.get(..fin) else {
            break;
        };

        // El sello del mismo mensaje: se busca a partir de aquí, no desde el
        // principio, para no coger el de otro.
        let cuando = entre(tras_marca, "\"timestamp\":\"", "\"")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if !de.is_empty() && !salida.iter().any(|r: &Respuesta| r.de == de) {
            salida.push(Respuesta {
                de: de.to_owned(),
                cuando,
            });
        }

        let Some(siguiente) = tras_marca.get(fin..) else {
            break;
        };
        resto = siguiente;
    }

    salida
}

fn entre(texto: &str, desde: &str, hasta: &str) -> Option<String> {
    let inicio = texto.find(desde)? + desde.len();
    let resto = texto.get(inicio..)?;
    let fin = resto.find(hasta)?;
    resto.get(..fin).map(str::to_owned)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
mod tests {
    use super::*;

    const SECRETO: &[u8] = b"secreto-de-la-app";

    fn firmar(cuerpo: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(SECRETO).expect("clave");
        mac.update(cuerpo);
        let bytes = mac.finalize().into_bytes();
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        format!("sha256={hex}")
    }

    const UNA_RESPUESTA: &str = r#"{"object":"whatsapp_business_account","entry":[{"changes":[{"field":"messages","value":{"messages":[{"from":"5218112345678","id":"wamid.X","timestamp":"1790000000","type":"text","text":{"body":"me interesa, mándame precios"}}]}}]}]}"#;

    /// **Un aviso sin firma válida no se mira siquiera.**
    ///
    /// Es lo único que separa un aviso de Meta de uno que se inventó
    /// cualquiera. Y un aviso falso no es inocuo: puede decir que alguien
    /// contestó —falseando la métrica que decide si el número va bien— o que la
    /// calidad se hundió, lo que pararía el canal.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `true` en
    /// `la_firma_cuadra`, esta prueba falla en los tres casos.
    #[test]
    fn sin_firma_valida_no_se_acepta_nada() {
        let cuerpo = UNA_RESPUESTA.as_bytes();

        for firma in [
            "",
            "sha256=00",
            "sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            // Con el prefijo correcto pero firmado con otro secreto.
            &firmar(b"otro cuerpo"),
        ] {
            assert_eq!(
                leer(cuerpo, firma, SECRETO),
                Err(AvisoRechazado::FirmaQueNoCuadra),
                "firma «{firma}»"
            );
        }
    }

    /// **Y con la firma correcta, sí.**
    ///
    /// Sin esto, la prueba de arriba pasaría con una función que rechaza
    /// siempre — que no protege nada, sólo rompe el canal entero.
    #[test]
    fn con_la_firma_correcta_se_lee() {
        let cuerpo = UNA_RESPUESTA.as_bytes();
        let r = leer(cuerpo, &firmar(cuerpo), SECRETO).expect("firma buena");
        assert!(matches!(r, Aviso::Contestaron(_)));
    }

    /// **De una respuesta sale quién contestó, y nunca qué dijo.**
    ///
    /// El cuerpo del aviso lleva el texto del mensaje —«me interesa, mándame
    /// precios»— y ese texto **no puede salir de esta función**. Es la decisión
    /// de Dirección del 29/09/2026 hecha código, y se comprueba mirando lo que
    /// devuelve, no lo que se guarda después: si saliera de aquí, ya habría
    /// pasado por todas las capas de en medio.
    ///
    /// Verificada rompiéndola el 29/09/2026: no hay forma de romperla sin
    /// añadir un campo a `Respuesta`, y ése es justo el punto — el tipo no
    /// tiene dónde meter el texto. Lo que sí se rompió: devolver el cuerpo
    /// entero en `de`, y entonces esta prueba falla.
    #[test]
    fn de_una_respuesta_sale_quien_y_nunca_que_dijo() {
        let cuerpo = UNA_RESPUESTA.as_bytes();
        let r = leer(cuerpo, &firmar(cuerpo), SECRETO).expect("lee");

        let Aviso::Contestaron(respuestas) = r else {
            panic!("tenía que ser una respuesta");
        };
        assert_eq!(respuestas.len(), 1);
        assert_eq!(respuestas[0].de, "5218112345678");
        assert_eq!(respuestas[0].cuando, 1_790_000_000);

        // Y lo que dijo no está por ninguna parte de lo que devolvimos.
        let vuelto = format!("{respuestas:?}");
        assert!(
            !vuelto.contains("interesa") && !vuelto.contains("precios"),
            "el texto del mensaje salió de la función: {vuelto}"
        );
    }

    /// **La misma persona contestando dos veces cuenta una.**
    ///
    /// Meta puede mandar varios mensajes en un solo aviso, y reenviar avisos
    /// que ya mandó. La pregunta que hay que poder contestar es a cuánta
    /// **gente** le interesó, no cuántos mensajes entraron.
    ///
    /// El esquema también lo impone con un `UNIQUE`, y las dos cosas hacen
    /// falta: ésta evita contar de más dentro de un aviso, y aquélla entre
    /// avisos distintos.
    #[test]
    fn la_misma_persona_cuenta_una_vez() {
        let dos = r#"{"object":"whatsapp_business_account","entry":[{"changes":[{"field":"messages","value":{"messages":[{"from":"5218112345678","timestamp":"1790000000"},{"from":"5218112345678","timestamp":"1790000009"},{"from":"5218199999999","timestamp":"1790000010"}]}}]}]}"#;
        let cuerpo = dos.as_bytes();

        let Aviso::Contestaron(r) = leer(cuerpo, &firmar(cuerpo), SECRETO).expect("lee") else {
            panic!("tenían que ser respuestas");
        };
        assert_eq!(r.len(), 2, "dos personas, tres mensajes");
    }

    /// Un aviso de algo que no usamos se reconoce y se ignora. **No es un
    /// error**: Meta manda avisos de cosas que no hemos pedido, y tratarlos
    /// como fallo llenaría la pantalla de errores que no lo son.
    #[test]
    fn lo_que_no_usamos_no_es_un_error() {
        let otro = r#"{"object":"whatsapp_business_account","entry":[{"changes":[{"field":"message_template_status_update","value":{}}]}]}"#;
        let cuerpo = otro.as_bytes();
        assert_eq!(
            leer(cuerpo, &firmar(cuerpo), SECRETO),
            Ok(Aviso::DeAlgoQueNoUsamos)
        );
    }

    /// Un cuerpo que no es de WhatsApp se rechaza, aunque venga firmado.
    ///
    /// La firma dice que lo manda quien tiene el secreto; no dice que sea lo
    /// que esperamos.
    #[test]
    fn un_cuerpo_que_no_es_de_whatsapp_se_rechaza() {
        for basura in [
            r#"{"object":"page","entry":[]}"#,
            "{}",
            "no es json siquiera",
        ] {
            let cuerpo = basura.as_bytes();
            assert_eq!(
                leer(cuerpo, &firmar(cuerpo), SECRETO),
                Err(AvisoRechazado::FormaDesconocida),
                "«{basura}»"
            );
        }
    }

    /// Un aviso de mensajes sin remitente legible no cuenta una respuesta
    /// fantasma. La métrica que protege al número tiene que poder confiarse.
    #[test]
    fn un_aviso_de_mensajes_sin_remitente_no_cuenta() {
        let raro = r#"{"object":"whatsapp_business_account","entry":[{"changes":[{"value":{"messages":[{"id":"wamid.X"}]}}]}]}"#;
        let cuerpo = raro.as_bytes();
        assert_eq!(
            leer(cuerpo, &firmar(cuerpo), SECRETO),
            Ok(Aviso::DeAlgoQueNoUsamos)
        );
    }
}
