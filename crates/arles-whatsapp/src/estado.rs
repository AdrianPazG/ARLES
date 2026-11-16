//! Preguntarle a Meta cómo está el número, en vez de esperar a que Meta llame.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTO EXISTE PORQUE ARLES NO PUEDE RECIBIR AVISOS
//!
//! El diseño de `LOGISTICA_DE_CAMPANAS.md` da por hecho que la calificación de
//! calidad llega sola, como aviso de Meta. No puede: los avisos son *webhooks*
//! y ARLES es una aplicación de escritorio sin componente de red
//! (`ARQUITECTURA.md` §11). Está explicado en [`crate::entrante`].
//!
//! Pero la calidad **sí se puede consultar**: Cloud API la devuelve al pedir el
//! número. Así que el semáforo del §—el que decide si hay que bajar el ritmo o
//! parar el canal— no depende de resolver el problema del alojamiento. Se
//! pregunta al abrir la pantalla y antes de cada campaña.
//!
//! **Lo que no se puede consultar son las respuestas.** No hay forma de pedirle
//! a Cloud API los mensajes entrantes: sólo llegan por aviso. Esa mitad sigue
//! esperando una decisión.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! PREGUNTAR TIENE UN COSTE, Y NO ES EL QUE PARECE
//!
//! No es el tráfico: es que **consultar da una foto y un aviso da una
//! película**. Entre dos consultas, la calidad pudo bajar y volver a subir sin
//! que nos enteremos. Para el semáforo basta —lo que importa es cómo está hoy—
//! pero para reaccionar a una caída en mitad de una campaña de mil mensajes,
//! no.
//!
//! Se dice aquí para que nadie lo descubra creyendo que está resuelto.

use std::time::Duration;

/// Cómo está el número, según Meta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstadoDelNumero {
    /// `GREEN`, `YELLOW`, `RED` o `UNKNOWN`.
    ///
    /// Se guarda tal como lo manda Meta y se traduce en la pantalla: es un dato
    /// de Meta, y reinterpretarlo aquí sería inventarse una escala propia sobre
    /// la suya.
    pub calidad: String,
    /// Cuántos mensajes al día deja mandar Meta a este número.
    ///
    /// **No es el límite que pone el usuario**: es el techo de Meta, y manda
    /// sobre el nuestro igual que el de la cuenta de correo manda sobre el de
    /// la etapa. `None` si Meta no lo dice.
    pub tope_diario: Option<u32>,
    /// Si Meta ha verificado el nombre que se enseña.
    pub nombre_verificado: bool,
}

/// Pregunta a Cloud API por el número.
///
/// # Errores
///
/// Devuelve `None` si no se pudo preguntar o si la respuesta no trae la
/// calidad. **No se inventa un `GREEN` por defecto**: un semáforo que se pone
/// en verde cuando no ha podido mirar es peor que no tener semáforo, porque se
/// mira y se cree.
#[must_use]
pub fn consultar(base: &str, phone_number_id: &str, token: &str) -> Option<EstadoDelNumero> {
    let agente: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(false)
        .build()
        .into();

    let url = format!(
        "{}/{}/{}?fields=quality_rating,messaging_limit_tier,name_status",
        base.trim_end_matches('/'),
        crate::cloud::VERSION_DE_LA_API,
        phone_number_id
    );

    let mut r = agente
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .ok()?;

    if !(200..300).contains(&r.status().as_u16()) {
        return None;
    }

    let texto = r.body_mut().read_to_string().ok()?;
    leer_estado(&texto)
}

/// Saca el estado de la respuesta de Meta.
///
/// Aparte de `consultar` para poder probarlo sin red — y porque es donde está
/// la única decisión: qué hacer cuando falta un campo.
#[must_use]
pub fn leer_estado(texto: &str) -> Option<EstadoDelNumero> {
    let calidad = entre(texto, "\"quality_rating\":\"", "\"")?;

    Some(EstadoDelNumero {
        calidad,
        // El «tier» de Meta llega como `TIER_1K`, `TIER_10K`… Se traduce a la
        // cifra, que es lo que se puede comparar con el ritmo configurado.
        tope_diario: entre(texto, "\"messaging_limit_tier\":\"", "\"").and_then(|t| tope(&t)),
        nombre_verificado: entre(texto, "\"name_status\":\"", "\"")
            .is_some_and(|s| s == "APPROVED"),
    })
}

/// Traduce el escalón de Meta a mensajes por día.
///
/// `TIER_UNLIMITED` devuelve `None` y no un número enorme: un tope inventado se
/// compararía con el ritmo configurado y podría enseñar un aviso que no tiene
/// sentido. «Sin tope» y «un tope muy grande» no son lo mismo.
fn tope(tier: &str) -> Option<u32> {
    match tier {
        "TIER_50" => Some(50),
        "TIER_250" => Some(250),
        "TIER_1K" => Some(1_000),
        "TIER_10K" => Some(10_000),
        "TIER_100K" => Some(100_000),
        _ => None,
    }
}

fn entre(texto: &str, desde: &str, hasta: &str) -> Option<String> {
    let inicio = texto.find(desde)? + desde.len();
    let resto = texto.get(inicio..)?;
    let fin = resto.find(hasta)?;
    resto.get(..fin).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// De la respuesta de Meta salen las tres cosas que la pantalla enseña.
    #[test]
    fn de_la_respuesta_sale_el_estado() {
        let r = r#"{"quality_rating":"GREEN","messaging_limit_tier":"TIER_1K","name_status":"APPROVED","id":"pn-1"}"#;
        assert_eq!(
            leer_estado(r),
            Some(EstadoDelNumero {
                calidad: "GREEN".to_owned(),
                tope_diario: Some(1_000),
                nombre_verificado: true,
            })
        );
    }

    /// **Sin calidad no hay estado, y no se inventa un verde.**
    ///
    /// Un semáforo que se pone en verde cuando no ha podido mirar es peor que
    /// no tener semáforo: se mira, se cree, y se manda una campaña con un
    /// número que podría estar en rojo.
    ///
    /// Es la misma regla que el «sin comprobar» del preflight: lo que no se ha
    /// visto no se da por bueno.
    ///
    /// Verificada rompiéndola el 29/09/2026: devolviendo `"GREEN"` cuando falta
    /// el campo, esta prueba falla.
    #[test]
    fn sin_calidad_no_se_inventa_un_verde() {
        for sin in [
            r#"{"messaging_limit_tier":"TIER_1K"}"#,
            "{}",
            "no es json",
            r#"{"error":{"code":190}}"#,
        ] {
            assert_eq!(leer_estado(sin), None, "«{sin}»");
        }
    }

    /// **«Sin tope» no es «un tope enorme».**
    ///
    /// `TIER_UNLIMITED` devuelve `None`. Traducirlo a un número grande haría que
    /// la pantalla comparara el ritmo configurado contra un tope inventado, y
    /// podría enseñar un aviso que no significa nada.
    #[test]
    fn sin_tope_no_es_un_tope_grande() {
        assert_eq!(tope("TIER_UNLIMITED"), None);
        assert_eq!(tope("TIER_QUE_META_INVENTE_MANANA"), None);
        assert_eq!(tope("TIER_50"), Some(50));
        assert_eq!(tope("TIER_100K"), Some(100_000));
    }

    /// El nombre sólo está verificado si Meta dice `APPROVED`.
    ///
    /// Escrito por lo que se acepta: cualquier otro valor —incluido uno que
    /// Meta estrene mañana— es «no verificado», que es el lado seguro.
    #[test]
    fn el_nombre_solo_esta_verificado_si_meta_lo_dice() {
        for (estado, esperado) in [
            ("APPROVED", true),
            ("PENDING_REVIEW", false),
            ("DECLINED", false),
            ("ALGO_NUEVO", false),
        ] {
            let r = format!(r#"{{"quality_rating":"GREEN","name_status":"{estado}"}}"#);
            assert_eq!(
                leer_estado(&r).map(|e| e.nombre_verificado),
                Some(esperado),
                "«{estado}»"
            );
        }
    }
}
