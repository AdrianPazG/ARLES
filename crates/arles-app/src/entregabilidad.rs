//! Los registros del dominio, enchufados al preflight · la 7.5, tramo 3.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL HUECO QUE ESTO CIERRA, Y LLEVABA ABIERTO DESDE LA FASE 6
//!
//! El punto 8 del preflight —«SPF, DKIM y DMARC de tu dominio»— existe desde
//! la Fase 6 con su texto escrito y su veredicto previsto. Y estaba cableado
//! así:
//!
//! ```ignore
//! // P-16: ARLES no consulta DNS. No bloquea, y se dice.
//! registros_del_dominio: None,
//! ```
//!
//! Es el patrón del §9 en su forma más larga: la pieza correcta, con su texto,
//! su variante y su sitio en la pantalla — y nadie la contestaba. Cuatro fases
//! enseñando «sin comprobar» a todo el mundo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTE PUNTO NO BLOQUEA, Y POR ESO HAY QUE ACERTARLO
//!
//! Es el único de los ocho que no impide activar (§154: informar, no dictar),
//! y eso **sube** el listón en vez de bajarlo: un punto que no bloquea sólo
//! vale por lo que dice. Si dice algo falso, lo único que produce es que
//! alguien vaya a tocar el DNS de su empresa sin necesidad — y un DNS mal
//! tocado deja de entregar el correo de todo el mundo, no sólo el de ARLES.
//!
//! De ahí las dos reglas de abajo: la caché para no preguntar de más, y la de
//! los selectores, que es la que más fácil sería equivocar.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use arles_core::entregabilidad::Revision;
use arles_core::ids::EmailAccountId;
use arles_email::dns::{ResolutorDelSistema, ResuelveTxt, RevisionDelDominio, revisar_dominio};
use serde::Serialize;

/// Cuánto vale una consulta antes de volver a preguntar.
///
/// Una hora. El DNS **no cambia cada cinco minutos** y el preflight se abre
/// varias veces seguidas mientras alguien rellena el asistente: sin caché,
/// cada vuelta son tres consultas de red delante de un botón.
///
/// Y una hora es poco, no mucho: quien acaba de añadir su SPF quiere verlo
/// reconocido hoy, no mañana. El equilibrio se rompe por el lado de preguntar
/// de más, que sólo cuesta latencia.
const VALE_DURANTE: Duration = Duration::from_secs(60 * 60);

/// Los selectores de DKIM que se prueban, por proveedor.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTA TABLA NO CONTRADICE «ARLES NO SE INVENTA NADA»
///
/// El núcleo tiene escrito que no se inventa el `include:` del SPF ni la clave
/// pública del DKIM, porque **un valor inventado se copia y se pega**, y un
/// SPF mal copiado deja de autorizar al servidor que de verdad envía.
///
/// Un selector es otra cosa: no es un valor que nadie vaya a escribir en su
/// DNS, es **una pregunta**. Probar `titan1._domainkey.tudominio.com` y no
/// encontrar nada no estropea nada ni se le enseña a nadie como instrucción.
///
/// Lo que sí hay que no hacer —y es la regla de este módulo— es **tomar un
/// selector que no resuelve por prueba de que no hay DKIM**. Puede que el
/// proveedor use otro nombre, o que sea un proveedor que no está en esta
/// tabla. Por eso, cuando ninguno responde, el DKIM queda **sin comprobar** y
/// nunca como ausente.
///
/// Los dominios se comparan por sufijo del servidor SMTP configurado, que es
/// el dato que ARLES ya tiene de la cuenta.
const SELECTORES: &[(&str, &[&str])] = &[
    ("titan.email", &["titan1", "titan2"]),
    ("google.com", &["google"]),
    ("gmail.com", &["google"]),
    ("outlook.com", &["selector1", "selector2"]),
    ("office365.com", &["selector1", "selector2"]),
    ("zoho.com", &["zoho"]),
];

/// Lo que se guarda de una consulta, con cuándo se hizo.
struct Guardado {
    cuando: Instant,
    revision: RevisionDelDominio,
}

/// La caché de dominios ya consultados.
///
/// Vive en `EstadoApp` y la comparten el preflight y la pantalla: son la misma
/// pregunta hecha desde dos sitios, y preguntarla dos veces daría dos
/// respuestas distintas si el DNS cambia en medio.
#[derive(Default)]
pub struct Dominios(Mutex<HashMap<String, Guardado>>);

impl std::fmt::Debug for Dominios {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dominios")
    }
}

impl Dominios {
    /// Lo que se sepa del dominio, preguntando sólo si hace falta.
    ///
    /// `resolutor` entra como parámetro para poder probar esto sin red: es el
    /// mismo motivo por el que `ResuelveTxt` es un rasgo.
    pub fn de(
        &self,
        resolutor: &dyn ResuelveTxt,
        dominio: &str,
        selector: Option<&str>,
    ) -> RevisionDelDominio {
        if let Ok(cache) = self.0.lock()
            && let Some(g) = cache.get(dominio)
            && g.cuando.elapsed() < VALE_DURANTE
        {
            return g.revision.clone();
        }

        let revision = revisar_dominio(resolutor, dominio, selector);

        if let Ok(mut cache) = self.0.lock() {
            cache.insert(
                dominio.to_string(),
                Guardado {
                    cuando: Instant::now(),
                    revision: revision.clone(),
                },
            );
        }
        revision
    }

    /// Olvida lo guardado de un dominio.
    ///
    /// La pantalla lo usa en «Volver a comprobar»: quien acaba de añadir su
    /// SPF pulsa eso, y una caché de una hora le diría que sigue faltando. Un
    /// botón de recargar que no recarga es peor que no tenerlo.
    pub fn olvidar(&self, dominio: &str) {
        if let Ok(mut cache) = self.0.lock() {
            cache.remove(dominio);
        }
    }
}

/// Busca el selector de DKIM probando los que se conocen del proveedor.
///
/// Devuelve `None` cuando ninguno responde, que es **«no sabemos cuál es»** y
/// nunca «no hay DKIM». Ver la nota de [`SELECTORES`].
fn selector_que_responde(
    resolutor: &dyn ResuelveTxt,
    servidor: &str,
    dominio: &str,
) -> Option<String> {
    let servidor = servidor.to_ascii_lowercase();
    let candidatos = SELECTORES
        .iter()
        .find(|(proveedor, _)| servidor.ends_with(proveedor))
        .map(|(_, sels)| *sels)?;

    candidatos.iter().find_map(|s| {
        match resolutor.txt(&format!("{s}._domainkey.{dominio}")) {
            // Sólo cuenta si contestó ALGO. Un `NoExiste` significa que ese
            // selector no es el suyo; un `NoSeSabe`, que no se pudo preguntar.
            // Ninguno de los dos autoriza a dar el DKIM por ausente.
            arles_email::dns::Respuesta::Txt(t) if !t.is_empty() => Some((*s).to_string()),
            _ => None,
        }
    })
}

/// Lo que cruza la frontera hacia la pantalla.
///
/// Se redacta aquí y no se reenvía `RevisionDelDominio` tal cual porque los
/// tipos de `arles-email` no son de la frontera. **No lleva ningún secreto**
/// (regla 3) ni ninguna ruta (regla 4): nombres de registro, veredictos y el
/// texto de qué poner, que es público por definición — está en el DNS.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrosDelDominio {
    /// El dominio mirado, para que la pantalla pueda decir cuál.
    pub dominio: String,
    /// El selector que contestó, si alguno.
    pub selector: Option<String>,
    pub revisiones: Vec<Revision>,
    /// Los que no se pudieron mirar, por su clave de texto (§139).
    pub sin_comprobar: Vec<String>,
    /// El veredicto del punto 8: `true`, `false` o `null` si no se sabe.
    pub en_orden: Option<bool>,
}

impl RegistrosDelDominio {
    fn de(dominio: &str, selector: Option<String>, r: &RevisionDelDominio) -> Self {
        Self {
            dominio: dominio.to_string(),
            selector,
            revisiones: r.revisiones.clone(),
            sin_comprobar: r
                .sin_comprobar
                .iter()
                .map(|x| x.clave_i18n().to_string())
                .collect(),
            en_orden: r.en_orden(),
        }
    }
}

/// Mira los registros del dominio de una cuenta.
///
/// Es el camino único: lo llaman el preflight y la pantalla, para que los dos
/// vean lo mismo.
///
/// `resolutor` entra como parámetro por la misma razón de siempre: aquí no hay
/// DNS con el que probar.
///
/// # Errores
///
/// Si la cuenta no se puede leer de la base.
pub fn registros_de_la_cuenta(
    db: &arles_db::Db,
    dominios: &Dominios,
    resolutor: &dyn ResuelveTxt,
    cuenta: EmailAccountId,
) -> Result<RegistrosDelDominio, crate::ErrorIpc> {
    let guardada = db
        .cuenta_de_correo(cuenta)
        .map_err(|e| crate::ErrorIpc::from(crate::AppError::Db(e)))?
        .ok_or_else(|| {
            crate::ErrorIpc::from(crate::AppError::Db(
                arles_db::error::DbError::CuentaDeCorreoNoExiste,
            ))
        })?;

    // El dominio que hay que autorizar es el del REMITENTE, no el del
    // servidor: lo que SPF y DMARC protegen es el `From:` que lee quien
    // recibe. Mandar desde `smtp.titan.email` en nombre de `telemetrymx.com`
    // es exactamente el caso normal.
    let dominio = guardada.datos.remitente().dominio().to_string();
    let servidor = guardada.datos.servidor().to_string();

    let selector = selector_que_responde(resolutor, &servidor, &dominio);
    let revision = dominios.de(resolutor, &dominio, selector.as_deref());

    Ok(RegistrosDelDominio::de(&dominio, selector, &revision))
}

/// El veredicto para el punto 8 del preflight.
///
/// Devuelve `None` ante cualquier tropiezo —cuenta ilegible, resolutor que no
/// arranca— y **nunca `Some(false)`**: no haber podido mirar no es un
/// suspenso. Es la misma regla que gobierna `dns.rs`, aplicada un piso más
/// arriba, porque aquí hay dos motivos nuevos para no saber.
#[must_use]
pub fn veredicto_del_dominio(
    db: &arles_db::Db,
    dominios: &Dominios,
    cuenta: EmailAccountId,
) -> Option<bool> {
    let resolutor = ResolutorDelSistema::nuevo().ok()?;
    registros_de_la_cuenta(db, dominios, &resolutor, cuenta)
        .ok()
        .and_then(|r| r.en_orden)
}

/// Mira los registros del dominio de un remitente, para la pantalla.
///
/// `recomprobar` olvida lo guardado antes de preguntar: es el botón de «volver
/// a comprobar» de quien acaba de añadir su SPF y no quiere esperar una hora.
///
/// # Errores
///
/// [`crate::ErrorIpc`] si el identificador no vale o la cuenta no existe. Que
/// el DNS no conteste **no es un error**: es un registro sin comprobar.
#[tauri::command]
pub fn registros_del_dominio(
    estado: tauri::State<'_, crate::estado::EstadoApp>,
    cuenta: String,
    recomprobar: bool,
) -> Result<RegistrosDelDominio, crate::ErrorIpc> {
    let id = cuenta
        .parse::<EmailAccountId>()
        .map_err(|e| crate::ErrorIpc::from(crate::AppError::Core(e)))?;

    // Si el resolutor no arranca no se acusa a nadie: se devuelven los tres
    // sin comprobar, que es lo que de verdad se sabe.
    let Ok(resolutor) = ResolutorDelSistema::nuevo() else {
        return Ok(RegistrosDelDominio {
            dominio: String::new(),
            selector: None,
            revisiones: Vec::new(),
            sin_comprobar: arles_core::entregabilidad::Registro::TODOS
                .iter()
                .map(|r| r.clave_i18n().to_string())
                .collect(),
            en_orden: None,
        });
    };

    if recomprobar && let Ok(Some(c)) = estado.db().cuenta_de_correo(id) {
        estado.dominios().olvidar(c.datos.remitente().dominio());
    }

    registros_de_la_cuenta(estado.db(), estado.dominios(), &resolutor, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    // `Registro` se lee del núcleo, que es de donde sale: `dns.rs` sólo lo
    // importa para su uso, no lo reexporta.
    use arles_core::entregabilidad::Registro;
    use arles_email::dns::ResolutorDeMentira;

    const DOMINIO: &str = "telemetrymx.com";

    fn bien() -> ResolutorDeMentira {
        ResolutorDeMentira::nuevo()
            .con(DOMINIO, &["v=spf1 include:spf.titan.email ~all"])
            .con(
                &format!("titan1._domainkey.{DOMINIO}"),
                &["v=DKIM1; k=rsa; p=MIIBIjANBgkq"],
            )
            .con(&format!("_dmarc.{DOMINIO}"), &["v=DMARC1; p=none"])
    }

    /// **El selector se encuentra probando los del proveedor.**
    #[test]
    fn el_selector_de_titan_se_encuentra_probando() {
        let s = selector_que_responde(&bien(), "smtp.titan.email", DOMINIO);
        assert_eq!(s, Some("titan1".to_string()));
    }

    /// **Un proveedor que no está en la tabla no da un selector inventado.**
    ///
    /// Y lo importante: tampoco da el DKIM por ausente. Es la regla de
    /// `SELECTORES`.
    #[test]
    fn un_proveedor_desconocido_no_inventa_selector() {
        let s = selector_que_responde(&bien(), "mail.loquesea.mx", DOMINIO);
        assert_eq!(s, None);
    }

    /// **Si ningún selector conocido responde, el DKIM queda SIN COMPROBAR.**
    ///
    /// Nunca `Falta`. Es la diferencia entre «no tienes firma» —que manda a
    /// crear una que quizá ya existe— y «no sabemos cuál es tu selector».
    #[test]
    fn sin_selector_que_responda_el_dkim_no_se_da_por_ausente() {
        let solo_spf_y_dmarc = ResolutorDeMentira::nuevo()
            .con(DOMINIO, &["v=spf1 include:otro.proveedor ~all"])
            .con(&format!("_dmarc.{DOMINIO}"), &["v=DMARC1; p=quarantine"]);

        let selector = selector_que_responde(&solo_spf_y_dmarc, "smtp.titan.email", DOMINIO);
        assert_eq!(selector, None);

        let r = revisar_dominio(&solo_spf_y_dmarc, DOMINIO, selector.as_deref());
        assert_eq!(r.sin_comprobar, vec![Registro::Dkim]);
        assert_eq!(r.en_orden(), None, "sin mirar el DKIM no hay aprobado");
    }

    /// **La caché contesta sin volver a preguntar.**
    ///
    /// Se mide con un espía que cuenta consultas: sin caché, dos llamadas son
    /// seis consultas de red delante de un botón.
    #[test]
    fn la_cache_no_vuelve_a_preguntar() {
        use std::cell::RefCell;

        struct Espia(ResolutorDeMentira, RefCell<usize>);
        impl ResuelveTxt for Espia {
            fn txt(&self, nombre: &str) -> arles_email::dns::Respuesta {
                *self.1.borrow_mut() += 1;
                self.0.txt(nombre)
            }
        }

        let espia = Espia(bien(), RefCell::new(0));
        let dominios = Dominios::default();

        dominios.de(&espia, DOMINIO, Some("titan1"));
        let tras_la_primera = *espia.1.borrow();
        assert_eq!(tras_la_primera, 3, "tres registros, tres consultas");

        dominios.de(&espia, DOMINIO, Some("titan1"));
        assert_eq!(
            *espia.1.borrow(),
            tras_la_primera,
            "la segunda vuelta sale de la caché"
        );

        // Y «Volver a comprobar» sí vuelve a preguntar: un botón de recargar
        // que no recarga es peor que no tenerlo.
        dominios.olvidar(DOMINIO);
        dominios.de(&espia, DOMINIO, Some("titan1"));
        assert_eq!(*espia.1.borrow(), tras_la_primera * 2);
    }

    /// Lo que cruza la frontera lleva el veredicto y las claves de texto, y
    /// **ninguna dirección ni ninguna ruta** (reglas 3 y 4).
    #[test]
    fn lo_que_cruza_la_frontera_no_lleva_nada_de_nadie() {
        let dominios = Dominios::default();
        let r = dominios.de(&bien(), DOMINIO, Some("titan1"));
        let ipc = RegistrosDelDominio::de(DOMINIO, Some("titan1".into()), &r);

        let json = serde_json::to_string(&ipc).expect("serializa");
        assert!(!json.contains('@'), "ninguna dirección: {json}");
        assert!(!json.contains('/'), "ninguna ruta: {json}");
        assert_eq!(ipc.en_orden, Some(true));
    }
}
