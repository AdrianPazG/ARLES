//! Preguntarle al DNS por los registros del dominio · la 7.5, tramo 2.
//!
//! El núcleo sabe **leer** SPF, DKIM y DMARC (`arles_core::entregabilidad`) y no
//! tiene red. Este archivo es la otra mitad: va a buscarlos.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LA DISTINCIÓN QUE GOBIERNA TODO ESTE ARCHIVO
//!
//! **«No está» y «no se pudo preguntar» son cosas distintas**, y confundirlas
//! es el peor fallo que esta entrega puede tener.
//!
//! Si ARLES trata un DNS bloqueado como «faltan tus registros», le dice a
//! alguien con el dominio perfectamente configurado que lo tiene roto, y lo
//! manda a tocar el DNS — que es donde un cambio equivocado deja de entregar
//! el correo de la empresa entera. Un hallazgo falso cuesta más que uno
//! ausente (auditoría B), y aquí la factura la paga el dominio.
//!
//! Y pasa de verdad, no es teórico: **en el contenedor donde se construyó
//! esto, el DNS contesta vacío incluso para `google.com`** —medido el
//! 26/10/2026—. Una red corporativa que filtra el puerto 53 hace lo mismo.
//!
//! Por eso `Respuesta` tiene tres variantes y no un `Option`: lo que separa
//! `NoExiste` de `NoSeSabe` es que el servidor contestó «ese nombre no
//! existe», que es una respuesta, no un silencio.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ `hickory-resolver` Y NO LA MITAD DE ABAJO
//!
//! Se midieron las dos el 26/10/2026, y la primera medida apuntaba a la otra:
//!
//! | | crates | Tokio | ¿sabe a qué servidor preguntar? |
//! |---|---|---|---|
//! | `hickory-proto` y los sockets a mano | +22 | no | **en Windows, no** |
//! | `hickory-resolver` con `system-config` | +86 | sí | sí, en los tres |
//!
//! La primera es más barata y tiene un agujero **en la plataforma principal**:
//! sin `system-config` hay que averiguar a qué servidor DNS pregunta este
//! equipo, y en Windows eso se lee del registro —otra dependencia— mientras
//! que en Linux es un archivo de texto. La salida de clavar un servidor
//! público conocido se descartó: en la red que filtra el 53, que es justo
//! donde esto importa, fallaría más que el del sistema.
//!
//! **Tokio entra, y entra encerrado.** Vive dentro de `ResolutorDelSistema` y
//! en ningún otro sitio: un runtime de un solo hilo que se crea, resuelve tres
//! nombres y se tira. `arles-motor` sigue siendo síncrono y no se tocó ni una
//! línea — que es lo que la auditoría C dejó dicho cuando corrigió un
//! documento por afirmar que el motor corría sobre Tokio.
//!
//! **Lo que esto cuesta y no se esconde:** 86 crates más en el árbol de
//! `arles-email`, que `arles-app` arrastra. El tamaño del instalador es uno de
//! los trece presupuestos y **hoy no se puede medir** —necesita el empaquetador
//! y P-01—, así que queda declarado para la Fase 9 en vez de darlo por bueno.

use std::collections::HashMap;
use std::time::Duration;

use arles_core::entregabilidad::{Registro, Revision, revisar_dkim, revisar_dmarc, revisar_spf};

/// Cuánto se espera por el DNS antes de darlo por incontestable.
///
/// Cinco segundos **en total**, no por consulta: esto corre dentro del
/// preflight, delante de alguien que acaba de pulsar un botón. Un DNS que no
/// contesta en cinco segundos no va a contestar, y lo que pasa entonces —
/// `NoSeSabe`— no bloquea nada.
const ESPERA: Duration = Duration::from_secs(5);

/// Qué contestó el DNS a una pregunta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Respuesta {
    /// El servidor contestó. La lista puede venir **vacía**: el nombre existe
    /// y no tiene ningún TXT, que para SPF o DMARC significa que no está.
    Txt(Vec<String>),
    /// El servidor contestó «ese nombre no existe». El registro **no está**, y
    /// eso es un dato, no un fallo de la consulta.
    NoExiste,
    /// **No se pudo preguntar.** Ni sí ni no: sin red, con el puerto filtrado,
    /// o con el servidor callado. Ver la cabecera.
    NoSeSabe,
}

impl Respuesta {
    /// Las cadenas TXT, o `None` si no se pudo saber.
    ///
    /// `NoExiste` y `Txt(vec![])` dan lo mismo —una lista vacía— a propósito:
    /// para el núcleo los dos casos son «no hay ningún registro de este tipo»,
    /// y distinguirlos ahí no cambiaría ninguna respuesta.
    #[must_use]
    fn cadenas(self) -> Option<Vec<String>> {
        match self {
            Self::Txt(v) => Some(v),
            Self::NoExiste => Some(Vec::new()),
            Self::NoSeSabe => None,
        }
    }
}

/// Quien sabe preguntarle al DNS.
///
/// Es un rasgo y no una función para poder probar esto sin red — que en este
/// contenedor no es una comodidad, es la única forma.
pub trait ResuelveTxt {
    /// Los TXT de un nombre.
    fn txt(&self, nombre: &str) -> Respuesta;
}

/// Lo que se sabe del dominio después de preguntar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionDelDominio {
    /// Los registros que sí se pudieron mirar, con su veredicto.
    pub revisiones: Vec<Revision>,
    /// Los que no. **Omitido no es aprobado**, y por eso se devuelven aparte
    /// en vez de dejarlos fuera de la lista sin más.
    pub sin_comprobar: Vec<Registro>,
}

impl RevisionDelDominio {
    /// El veredicto para el punto 8 del preflight.
    ///
    /// ── EL ORDEN DE LAS TRES RESPUESTAS, QUE NO ES EL OBVIO ──
    ///
    /// 1. **Un fallo seguro manda sobre una duda.** Si el SPF falta y el DKIM
    ///    no se pudo mirar, la respuesta es «falla»: ya se sabe algo que está
    ///    mal, y esconderlo detrás de un «no se sabe» sería perder el dato que
    ///    sí se tiene.
    /// 2. **Una duda manda sobre un aprobado.** Si los dos que se miraron
    ///    están bien y el tercero no se pudo mirar, la respuesta es «no se
    ///    sabe». Devolver «bien» sería dar por bueno justo lo que no se vio,
    ///    que es la regla del preflight entera.
    /// 3. Y si se miraron todos y todos están bien, «bien».
    #[must_use]
    pub fn en_orden(&self) -> Option<bool> {
        if self.revisiones.iter().any(|r| !r.hallazgo.esta_bien()) {
            return Some(false);
        }
        if !self.sin_comprobar.is_empty() || self.revisiones.is_empty() {
            return None;
        }
        Some(true)
    }
}

/// Pregunta por los tres registros del dominio y los revisa.
///
/// `selector` es el del DKIM, que **no se puede adivinar**: lo elige el
/// proveedor de correo. Sin él, el DKIM no se mira y se devuelve en
/// `sin_comprobar` — que no es lo mismo que decir que falta.
#[must_use]
pub fn revisar_dominio(
    resolutor: &dyn ResuelveTxt,
    dominio: &str,
    selector: Option<&str>,
) -> RevisionDelDominio {
    let mut revisiones = Vec::new();
    let mut sin_comprobar = Vec::new();

    // SPF: en la raíz del dominio.
    match resolutor.txt(dominio).cadenas() {
        Some(txt) => revisiones.push(revisar_spf(dominio, &txt)),
        None => sin_comprobar.push(Registro::Spf),
    }

    // DKIM: en `<selector>._domainkey.<dominio>`, y sólo si hay selector.
    match selector {
        Some(s) => match resolutor
            .txt(&format!("{s}._domainkey.{dominio}"))
            .cadenas()
        {
            Some(txt) => revisiones.push(revisar_dkim(dominio, s, &txt)),
            None => sin_comprobar.push(Registro::Dkim),
        },
        None => sin_comprobar.push(Registro::Dkim),
    }

    // DMARC: en `_dmarc.<dominio>`.
    match resolutor.txt(&format!("_dmarc.{dominio}")).cadenas() {
        Some(txt) => revisiones.push(revisar_dmarc(dominio, &txt)),
        None => sin_comprobar.push(Registro::Dmarc),
    }

    RevisionDelDominio {
        revisiones,
        sin_comprobar,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// El resolutor de verdad
// ───────────────────────────────────────────────────────────────────────────

/// Qué hacer con una negativa del DNS, según venga firmada o no.
///
/// Está aquí fuera y no dentro del `match` por la misma razón que
/// `juntar_trozos`: **es la regla que más caro sale equivocar y vivía donde
/// ninguna prueba la alcanzaba**, dentro del resolutor que necesita red. El
/// defecto que lo demostró está contado arriba, en el cuerpo de `txt`.
///
/// `con_soa` es si la respuesta trajo el registro SOA de la zona, que es lo
/// que convierte un «no hay» en una negativa autoritativa.
const fn negativa(con_soa: bool) -> Respuesta {
    if con_soa {
        // Alguien con autoridad sobre la zona dice que no está. Es un dato.
        Respuesta::NoExiste
    } else {
        // Nadie ha firmado esta negativa. No se acusa al dominio.
        Respuesta::NoSeSabe
    }
}

/// Vuelve a juntar los trozos de un TXT largo.
///
/// ── POR QUÉ ESTO ES UNA FUNCIÓN APARTE ──
///
/// Un TXT no puede pasar de 255 bytes, así que uno más largo **viaja partido**
/// y llega como varios trozos de un mismo registro. Una clave DKIM casi
/// siempre lo es.
///
/// Juntarlos mal es el hallazgo falso más caro que esta entrega puede
/// producir: tratar los trozos como registros distintos daría «hay 3 y por eso
/// no vale ninguno» sobre un DKIM perfectamente correcto, y mandaría a
/// rehacer una firma que funciona.
///
/// Estaba escrito dentro de `ResolutorDelSistema`, que necesita red, **así que
/// ninguna prueba lo tocaba**: la pieza correcta y la junta sin probar, que es
/// el patrón que este proyecto lleva más de veinte veces encontrando. Aquí
/// fuera se prueba sin DNS.
///
/// Se juntan **sin separador**: los trozos son un corte del texto, no una
/// lista. Un separador metería un carácter en mitad de la clave.
fn juntar_trozos(trozos: &[Box<[u8]>]) -> String {
    trozos
        .iter()
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .collect()
}

/// Le pregunta al DNS que tenga configurado este equipo.
///
/// Toda la dependencia de `hickory` y de Tokio vive aquí dentro. Fuera de este
/// bloque, el resto del archivo —y del proyecto— sólo conoce `ResuelveTxt`.
pub struct ResolutorDelSistema {
    runtime: tokio::runtime::Runtime,
    resolutor: hickory_resolver::TokioResolver,
}

/// `Debug` a mano y no derivado: el taller lo exige de todo tipo público, y
/// `TokioResolver` no lo implementa. Se escribe el nombre y nada más — lo que
/// hay dentro es la configuración de DNS del equipo, que no tiene por qué
/// acabar en una traza.
impl std::fmt::Debug for ResolutorDelSistema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ResolutorDelSistema")
    }
}

impl ResolutorDelSistema {
    /// Lee la configuración de DNS del equipo y prepara el resolutor.
    ///
    /// En Linux y macOS sale de `/etc/resolv.conf`; en Windows, del registro.
    /// Ésa es la razón de pagar esta dependencia: ver la cabecera.
    ///
    /// # Errors
    ///
    /// Si no se puede leer la configuración del sistema o no se puede crear el
    /// runtime. En los dos casos quien llama debe tratarlo como `NoSeSabe`, no
    /// como «faltan los registros».
    pub fn nuevo() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        let resolutor = hickory_resolver::TokioResolver::builder_tokio()
            .map_err(|e| e.to_string())?
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self { runtime, resolutor })
    }
}

impl ResuelveTxt for ResolutorDelSistema {
    fn txt(&self, nombre: &str) -> Respuesta {
        let consulta =
            async { tokio::time::timeout(ESPERA, self.resolutor.txt_lookup(nombre)).await };

        match self.runtime.block_on(consulta) {
            // Se acabó el tiempo: ni sí ni no.
            Err(_) => Respuesta::NoSeSabe,
            Ok(Ok(respuesta)) => Respuesta::Txt(
                respuesta
                    .answers()
                    .iter()
                    // `data` es un campo público en hickory 0.26, no un método.
                    .filter_map(|registro| match &registro.data {
                        hickory_resolver::proto::rr::RData::TXT(txt) => Some(txt),
                        // En la respuesta vienen también los CNAME de la
                        // cadena, si el nombre era un alias. No son TXT y no
                        // se cuentan: contarlos daría «hay 2» sobre un
                        // registro único.
                        _ => None,
                    })
                    .map(|txt| juntar_trozos(&txt.txt_data))
                    .collect(),
            ),
            Ok(Err(e)) => {
                // ── LA LÍNEA QUE SEPARA «NO ESTÁ» DE «NO SE PUDO PREGUNTAR» ──
                //
                // `is_no_records_found()` NO basta, y creerlo costó el único
                // defecto grave de esta entrega. Medido el 26/10/2026 contra
                // `telemetrymx.com`, que tiene los tres registros bien puestos
                // desde el 22/10: ARLES contestó **«faltan los tres»**.
                //
                // La red de este contenedor devuelve respuestas vacías con
                // código NOERROR, y eso es «no hay registros» a ojos del
                // resolutor. Una red corporativa que filtra el DNS hace lo
                // mismo. Ninguna de las nueve pruebas podía cazarlo: el
                // resolutor de mentira no sabe mentir como miente una red.
                //
                // Lo que distingue un «no existe» de verdad es el **SOA**: una
                // negativa autoritativa viene firmada por quien manda en la
                // zona, y así lo dice la documentación del propio tipo. Una
                // respuesta fabricada por un filtro, o un referral a medias,
                // no lo trae.
                //
                // Sin SOA no se acusa a nadie: `NoSeSabe`. El coste del error
                // es asimétrico y por eso se cae de este lado — decir «no se
                // sabe» de un dominio pelado sólo pierde un aviso que no
                // bloquea, y decir «te faltan los registros» manda a tocar el
                // DNS de una empresa que lo tiene bien.
                negativa(e.into_soa().is_some())
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// El resolutor de mentira
// ───────────────────────────────────────────────────────────────────────────

/// Un DNS de mentira, para probar sin red.
///
/// Existe por la misma razón que el servidor SMTP de mentira de este crate: a
/// Gmail no se le puede pedir que conteste un `550`, y al DNS de verdad no se
/// le puede pedir que se quede callado a voluntad. **Y aquí además no hay
/// alternativa**: este contenedor no resuelve TXT.
/// Sin `Default`: lo que se contesta por defecto a un nombre desconocido es
/// una decisión —`NoExiste` o `NoSeSabe`— y son las dos mitades de lo que este
/// archivo distingue. Un `Default` derivado la tomaría en silencio.
#[derive(Debug, Clone)]
pub struct ResolutorDeMentira {
    respuestas: HashMap<String, Respuesta>,
    /// Lo que se contesta a un nombre del que nadie dijo nada.
    ///
    /// `NoExiste` por defecto, que es lo que hace un DNS sano con un nombre
    /// que no está. Se cambia a `NoSeSabe` con `todo_callado()` para simular
    /// una red que filtra el puerto 53.
    por_defecto: Respuesta,
}

impl ResolutorDeMentira {
    #[must_use]
    pub fn nuevo() -> Self {
        Self {
            respuestas: HashMap::new(),
            por_defecto: Respuesta::NoExiste,
        }
    }

    /// Este nombre contesta estos TXT.
    #[must_use]
    pub fn con(mut self, nombre: &str, txt: &[&str]) -> Self {
        self.respuestas.insert(
            nombre.to_string(),
            Respuesta::Txt(txt.iter().map(|t| (*t).to_string()).collect()),
        );
        self
    }

    /// Este nombre no contesta. Simula el puerto 53 filtrado para un nombre.
    #[must_use]
    pub fn callado(mut self, nombre: &str) -> Self {
        self.respuestas
            .insert(nombre.to_string(), Respuesta::NoSeSabe);
        self
    }

    /// **Nada contesta**: la red filtra el DNS entero.
    #[must_use]
    pub fn todo_callado(mut self) -> Self {
        self.por_defecto = Respuesta::NoSeSabe;
        self
    }
}

impl ResuelveTxt for ResolutorDeMentira {
    fn txt(&self, nombre: &str) -> Respuesta {
        self.respuestas
            .get(nombre)
            .cloned()
            .unwrap_or_else(|| self.por_defecto.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::entregabilidad::Hallazgo;

    const DOMINIO: &str = "telemetrymx.com";
    const SELECTOR: &str = "titan1";

    /// Un dominio con los tres registros puestos como los tiene
    /// `telemetrymx.com` desde el 22/10/2026.
    fn bien_configurado() -> ResolutorDeMentira {
        ResolutorDeMentira::nuevo()
            .con(DOMINIO, &["v=spf1 include:spf.titan.email ~all"])
            .con(
                &format!("{SELECTOR}._domainkey.{DOMINIO}"),
                &["v=DKIM1; k=rsa; p=MIIBIjANBgkq"],
            )
            .con(
                &format!("_dmarc.{DOMINIO}"),
                &["v=DMARC1; p=none; rua=mailto:a@b.com"],
            )
    }

    #[test]
    fn un_dominio_bien_configurado_sale_en_orden() {
        let r = revisar_dominio(&bien_configurado(), DOMINIO, Some(SELECTOR));
        assert_eq!(r.revisiones.len(), 3);
        assert!(r.sin_comprobar.is_empty());
        assert_eq!(r.en_orden(), Some(true), "p=none cuenta como correcto");
    }

    /// **El caso que esta entrega existe para no equivocar.**
    ///
    /// Con la red filtrando el DNS entero —lo que hace este contenedor, medido
    /// el 26/10/2026 incluso para `google.com`— la respuesta tiene que ser «no
    /// se sabe» y **nunca** «te faltan los registros». Lo segundo mandaría a
    /// tocar el DNS de un dominio que está perfectamente.
    #[test]
    fn con_el_dns_filtrado_no_se_sabe_y_no_se_acusa() {
        let r = revisar_dominio(
            &ResolutorDeMentira::nuevo().todo_callado(),
            DOMINIO,
            Some(SELECTOR),
        );
        assert!(r.revisiones.is_empty());
        assert_eq!(r.sin_comprobar.len(), 3);
        assert_eq!(
            r.en_orden(),
            None,
            "sin poder preguntar no se acusa a nadie de nada"
        );
    }

    /// **Un nombre que no existe SÍ es una respuesta**, y se distingue del
    /// silencio. Sin esta diferencia, un dominio sin DMARC y uno que no se
    /// pudo consultar darían lo mismo, y las acciones son opuestas: en un caso
    /// hay que añadir un registro, en el otro no hay que tocar nada.
    #[test]
    fn un_nombre_que_no_existe_no_es_lo_mismo_que_un_dns_callado() {
        // El DMARC no está; el resto sí.
        let falta = ResolutorDeMentira::nuevo()
            .con(DOMINIO, &["v=spf1 include:spf.titan.email ~all"])
            .con(
                &format!("{SELECTOR}._domainkey.{DOMINIO}"),
                &["v=DKIM1; p=MIIBIjANBgkq"],
            );
        let r = revisar_dominio(&falta, DOMINIO, Some(SELECTOR));
        assert_eq!(r.en_orden(), Some(false), "sabemos que falta el DMARC");
        assert!(r.sin_comprobar.is_empty());

        // Ahora el DMARC no contesta, y lo demás igual.
        let callado = falta.clone().callado(&format!("_dmarc.{DOMINIO}"));
        let r2 = revisar_dominio(&callado, DOMINIO, Some(SELECTOR));
        assert_eq!(r2.en_orden(), None, "no se sabe no es falla");
        assert_eq!(r2.sin_comprobar, vec![Registro::Dmarc]);
    }

    /// **Un fallo seguro manda sobre una duda**, que es el punto 1 del orden
    /// de `en_orden`. Se prueba con el SPF roto y el DKIM mudo a la vez, que
    /// es el único caso que distingue ese orden del contrario.
    #[test]
    fn un_fallo_seguro_pesa_mas_que_una_duda() {
        let r = revisar_dominio(
            &ResolutorDeMentira::nuevo()
                .con(DOMINIO, &["v=spf1 +all"])
                .callado(&format!("{SELECTOR}._domainkey.{DOMINIO}"))
                .con(&format!("_dmarc.{DOMINIO}"), &["v=DMARC1; p=reject"]),
            DOMINIO,
            Some(SELECTOR),
        );
        assert_eq!(
            r.en_orden(),
            Some(false),
            "el SPF abierto se sabe, y se dice"
        );
        assert_eq!(r.sin_comprobar, vec![Registro::Dkim]);
    }

    /// **Una duda manda sobre un aprobado**, que es el punto 2. Dos correctos
    /// y uno sin mirar no son un dominio en orden.
    #[test]
    fn dos_correctos_y_uno_sin_mirar_no_son_un_aprobado() {
        let r = revisar_dominio(
            &bien_configurado().callado(&format!("{SELECTOR}._domainkey.{DOMINIO}")),
            DOMINIO,
            Some(SELECTOR),
        );
        assert_eq!(r.revisiones.len(), 2);
        assert!(r.revisiones.iter().all(|x| x.hallazgo.esta_bien()));
        assert_eq!(r.en_orden(), None, "omitido no es aprobado");
    }

    /// **Sin selector el DKIM no se mira, y no se da por ausente.**
    ///
    /// Quien no sepa su selector no tiene un DKIM roto: tiene un DKIM que
    /// nadie ha podido mirar. Marcarlo como `Falta` lo mandaría a crear una
    /// firma que probablemente ya existe.
    #[test]
    fn sin_selector_el_dkim_queda_sin_comprobar_y_no_como_ausente() {
        let r = revisar_dominio(&bien_configurado(), DOMINIO, None);
        assert_eq!(r.sin_comprobar, vec![Registro::Dkim]);
        assert!(
            !r.revisiones.iter().any(|x| x.registro == Registro::Dkim),
            "no se inventa un veredicto del DKIM"
        );
        assert_eq!(r.en_orden(), None);
    }

    /// **Un nombre que existe sin ningún TXT cuenta como que falta.**
    ///
    /// Es distinto de `NoExiste` en el cable y lo mismo para el usuario: no
    /// hay registro. Se comprueba que las dos formas dan el mismo veredicto,
    /// porque tratarlas distinto sería una diferencia que nadie puede explicar.
    #[test]
    fn un_nombre_sin_txt_y_uno_que_no_existe_dan_lo_mismo() {
        let vacio = ResolutorDeMentira::nuevo().con(&format!("_dmarc.{DOMINIO}"), &[]);
        let ausente = ResolutorDeMentira::nuevo();

        let a = revisar_dominio(&vacio, DOMINIO, Some(SELECTOR));
        let b = revisar_dominio(&ausente, DOMINIO, Some(SELECTOR));

        let dmarc = |r: &RevisionDelDominio| {
            r.revisiones
                .iter()
                .find(|x| x.registro == Registro::Dmarc)
                .map(|x| x.hallazgo.clone())
        };
        assert_eq!(dmarc(&a), Some(Hallazgo::Falta));
        assert_eq!(dmarc(&a), dmarc(&b));
    }

    /// **Una negativa sin firmar no acusa al dominio.**
    ///
    /// Es el defecto más grave de esta entrega, y lo encontró MEDIR, no leer:
    /// el 26/10/2026, contra `telemetrymx.com` —con los tres registros bien
    /// puestos desde el 22/10—, ARLES contestó «faltan los tres» y
    /// `Some(false)`. La red de este contenedor devuelve respuestas vacías con
    /// código NOERROR, que para el resolutor es «no hay registros».
    ///
    /// Las nueve pruebas estaban en verde: el resolutor de mentira contesta lo
    /// que se le dice, y **no sabe mentir como miente una red**.
    ///
    /// Lo que distingue una negativa de verdad es el SOA de la zona. Sin él no
    /// se da nada por ausente. Medido otra vez tras el arreglo, mismo dominio
    /// y misma red: los tres salen «sin comprobar» y el veredicto es `None`.
    #[test]
    fn una_negativa_sin_firmar_no_da_por_ausente_ningun_registro() {
        assert_eq!(
            negativa(false),
            Respuesta::NoSeSabe,
            "sin SOA nadie ha firmado ese «no está»"
        );
        assert_eq!(
            negativa(true),
            Respuesta::NoExiste,
            "con SOA, la zona dice que no está y eso es un dato"
        );
    }

    /// **Una clave DKIM larga llega partida y se vuelve a juntar entera.**
    ///
    /// Es la junta que ninguna prueba tocaba hasta que se sacó de dentro del
    /// resolutor de verdad. Se comprueba con las dos mitades que importan: que
    /// el texto sale completo, y que **no se cuela ningún separador** — un
    /// espacio o una coma en mitad de la clave la invalida sin que nada falle.
    #[test]
    fn una_clave_dkim_partida_en_trozos_se_vuelve_a_juntar_entera() {
        // Un DKIM real pasa de 255 bytes y llega en dos trozos.
        let trozos: Vec<Box<[u8]>> = vec![
            b"v=DKIM1; k=rsa; p=MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA"
                .to_vec()
                .into(),
            b"abcdefghijklmnopqrstuvwxyz0123456789IDAQAB"
                .to_vec()
                .into(),
        ];
        let entero = juntar_trozos(&trozos);

        assert_eq!(
            entero,
            "v=DKIM1; k=rsa; p=MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA\
             abcdefghijklmnopqrstuvwxyz0123456789IDAQAB"
        );

        // Y lo que de verdad importa: juntado así, el núcleo lo lee como UN
        // registro correcto. Con un separador entre trozos, la clave quedaría
        // partida y el DKIM saldría roto.
        let r = revisar_dominio(
            &ResolutorDeMentira::nuevo()
                .con(&format!("{SELECTOR}._domainkey.{DOMINIO}"), &[&entero]),
            DOMINIO,
            Some(SELECTOR),
        );
        let dkim = r
            .revisiones
            .iter()
            .find(|x| x.registro == Registro::Dkim)
            .map(|x| x.hallazgo.clone());
        assert_eq!(dkim, Some(Hallazgo::EnOrden));
    }

    /// Los tres nombres que se consultan son los del estándar, y se comprueban
    /// porque un `_dmarc` mal escrito —o un `_domainkey` sin el guion bajo—
    /// daría «falta el registro» sobre un dominio correcto, sin que nada más
    /// lo cazara.
    #[test]
    fn se_pregunta_por_los_tres_nombres_del_estandar() {
        use std::cell::RefCell;

        struct Espia(RefCell<Vec<String>>);
        impl ResuelveTxt for Espia {
            fn txt(&self, nombre: &str) -> Respuesta {
                self.0.borrow_mut().push(nombre.to_string());
                Respuesta::NoExiste
            }
        }

        let espia = Espia(RefCell::new(Vec::new()));
        // Lo que se mide aquí no es el veredicto sino POR QUÉ NOMBRES se
        // preguntó, así que el resultado se descarta a propósito.
        let _ = revisar_dominio(&espia, DOMINIO, Some(SELECTOR));

        assert_eq!(
            *espia.0.borrow(),
            vec![
                "telemetrymx.com".to_string(),
                "titan1._domainkey.telemetrymx.com".to_string(),
                "_dmarc.telemetrymx.com".to_string(),
            ]
        );
    }
}
