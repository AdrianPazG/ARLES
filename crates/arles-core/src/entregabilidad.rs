//! SPF, DKIM y DMARC · la entrega 7.5, movida al final por Dirección el
//! 30/09/2026 y desbloqueada al cerrarse R-24 el 22/10/2026.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ ES ESTO Y QUÉ NO ES
//!
//! Esto **lee** tres registros del DNS y dice si están y si sirven. No los
//! pone, no los cambia y no consulta nada: el núcleo no tiene red (cero I/O),
//! así que aquí entran las cadenas TXT ya traídas y sale un veredicto.
//!
//! Lo que esto evita es el fallo más caro del correo masivo, y es un fallo que
//! **no se nota al enviar**: sin estos tres registros el correo sale, el
//! servidor lo acepta, ARLES lo cuenta como aceptado — y el filtro de quien
//! recibe lo manda a no deseados. Todas las cifras salen bien y el correo no
//! lo lee nadie. Está contado en `QUE-HACER-CON-EL-SPAM.md`: desde una
//! dirección de Gmail el correo caía en no deseados **con la autenticación
//! entera en verde**.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! TENER DMARC NO ES ESTAR PROTEGIDO, Y ÉSA ES LA DISTINCIÓN DE ESTE ARCHIVO
//!
//! Un DMARC en `p=none` dice literalmente «vigila y mándame informes, pero no
//! rechaces nada». Está bien puesto, se lee sin error, y **no detiene a nadie
//! que suplante el dominio**. Marcarlo como «en orden» a secas sería el mismo
//! engaño que decir «entregado» cuando lo que hubo fue un «aceptado» (§65):
//! una palabra correcta que se entiende como otra cosa.
//!
//! Pero marcarlo en rojo tampoco vale, y por un motivo concreto: **`p=none` es
//! lo correcto al empezar**. `PASO-A-PASO-DEL-CORREO.md` manda empezar ahí y
//! subir a `p=quarantine` tras unas semanas de informes, porque subirlo el
//! primer día es la forma más rápida de que el correo propio deje de
//! entregarse. Un rojo sobre la configuración que el propio proyecto recomienda
//! enseña a ignorar los rojos.
//!
//! Así que es un tercer estado —`EnObservacion`— que cuenta como correcto para
//! el preflight y **lleva su aviso**. §154: informar, no dictar.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ARLES NO SE INVENTA EL `include:` DEL SPF
//!
//! Cuando falta un registro, esto dice **qué poner**, que es lo que pedía el
//! plan: «el texto de qué poner en el DNS, no sólo un aspa roja». Con una
//! excepción deliberada: el SPF lleva un `include:` que **depende del
//! proveedor de correo** —Titan, Google y Microsoft tienen el suyo— y ARLES no
//! lo sabe.
//!
//! Inventarse uno daría un registro que se copia, se pega, parece correcto y
//! **deja de autorizar al servidor que de verdad envía**: el correo pasa de
//! llegar a rebotar, por culpa de la ayuda. Por eso `QuePoner::valor` es
//! `Option`, y en el SPF va a `None` con la forma explicada aparte. Es la
//! lección de la auditoría B escrita en un tipo: antes de afirmar algo que
//! falta, hay que saber cómo se llama de verdad.

use serde::Serialize;

/// Cuál de los tres registros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Registro {
    /// Quién puede enviar en nombre del dominio.
    Spf,
    /// La firma con la que se comprueba que el mensaje no se tocó.
    Dkim,
    /// Qué hacer con el correo que no pasa los dos anteriores.
    Dmarc,
}

impl Registro {
    /// Los tres, en el orden en que se enseñan: de lo más básico a lo que se
    /// configura al final.
    pub const TODOS: &'static [Self] = &[Self::Spf, Self::Dkim, Self::Dmarc];

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Spf => "entregabilidad.spf",
            Self::Dkim => "entregabilidad.dkim",
            Self::Dmarc => "entregabilidad.dmarc",
        }
    }
}

/// Qué le pasa a un registro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "hallazgo")]
pub enum Hallazgo {
    /// Está y sirve.
    EnOrden,

    /// **El DMARC está puesto y no rechaza nada todavía** (`p=none`).
    ///
    /// Cuenta como correcto —es lo que hay que poner al empezar— y lleva su
    /// aviso. Ver la cabecera de este archivo.
    EnObservacion,

    /// No hay ninguno.
    Falta,

    /// **Hay más de uno, así que no vale ninguno.**
    ///
    /// No es quisquillosidad: con dos registros SPF o dos DMARC, quien recibe
    /// no elige el más estricto ni el primero — **descarta los dos** y el
    /// dominio queda como si no tuviera nada. Es un fallo que no se ve
    /// mirando el DNS, porque los dos registros están ahí y los dos parecen
    /// correctos. `PASO-A-PASO-DEL-CORREO.md` ya lo encontró una vez.
    Duplicado {
        /// Cuántos hay. **Cifras, no adjetivos** (§94).
        cuantos: usize,
    },

    /// Está, pero no se entiende: le falta la versión o la parte obligatoria.
    NoSeEntiende,

    /// **El SPF autoriza a todo el mundo** (`+all`).
    ///
    /// Peor que no tenerlo: declara por escrito que cualquiera puede enviar en
    /// nombre del dominio, así que quien recibe no tiene con qué distinguir un
    /// envío legítimo de una suplantación.
    DemasiadoAbierto,

    /// **La clave DKIM está retirada** (`p=` vacío).
    ///
    /// Es la forma estándar de revocar una clave. El registro existe, se lee
    /// sin error, y ninguna firma hecha con ella vale.
    ClaveRetirada,
}

impl Hallazgo {
    /// ¿Cuenta como correcto para el preflight?
    ///
    /// `EnObservacion` **sí**, y es la única sutileza del archivo: el DMARC en
    /// `p=none` es lo que el proyecto manda poner al empezar.
    #[must_use]
    pub const fn esta_bien(&self) -> bool {
        matches!(self, Self::EnOrden | Self::EnObservacion)
    }

    /// La clave de texto (§139).
    #[must_use]
    pub const fn clave_i18n(&self) -> &'static str {
        match self {
            Self::EnOrden => "entregabilidad.enOrden",
            Self::EnObservacion => "entregabilidad.enObservacion",
            Self::Falta => "entregabilidad.falta",
            Self::Duplicado { .. } => "entregabilidad.duplicado",
            Self::NoSeEntiende => "entregabilidad.noSeEntiende",
            Self::DemasiadoAbierto => "entregabilidad.demasiadoAbierto",
            Self::ClaveRetirada => "entregabilidad.claveRetirada",
        }
    }
}

/// Qué hay que escribir en el DNS para arreglarlo.
///
/// `valor` es `Option` a propósito: ver la cabecera. Cuando ARLES no puede
/// saber el valor sin inventárselo, va a `None` y la pantalla explica la forma
/// en vez de dar algo que se copie y pegue mal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuePoner {
    /// El nombre del registro, entero. Ej. `_dmarc.telemetrymx.com`.
    pub nombre: String,
    /// Siempre `TXT`. Va explícito porque es la mitad de la instrucción: en el
    /// panel de un proveedor de DNS hay que elegir el tipo antes del valor.
    pub tipo: &'static str,
    /// El valor exacto, cuando ARLES puede saberlo sin inventárselo.
    pub valor: Option<String>,
}

/// Un registro revisado, listo para enseñar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    pub registro: Registro,
    pub hallazgo: Hallazgo,
    /// Qué poner. `None` cuando no hay nada que arreglar.
    pub que_poner: Option<QuePoner>,
}

/// El tipo de registro, para construir `QuePoner`.
const TXT: &str = "TXT";

/// Quita las comillas y los espacios de los extremos de una cadena TXT.
///
/// Un registro TXT largo viaja partido en trozos entrecomillados, y según de
/// dónde se lea puede llegar con las comillas puestas. Sin esto, un
/// `"v=spf1 ..."` entrecomillado no empezaría por `v=spf1` y se daría por
/// ausente un registro que está.
fn limpiar(t: &str) -> String {
    t.trim().trim_matches('"').trim().to_string()
}

/// Se queda sólo con los TXT que son de este tipo de registro.
///
/// Mirar el prefijo —y no quedarse con todos los TXT del nombre— importa:
/// en la raíz de un dominio conviven el SPF con los TXT de verificación de
/// media docena de servicios. Contar «cuántos TXT hay» daría `Duplicado`
/// siempre.
fn los_que_son(txt: &[String], prefijo: &str) -> Vec<String> {
    txt.iter()
        .map(|t| limpiar(t))
        .filter(|t| {
            t.to_ascii_lowercase()
                .replace(' ', "")
                .starts_with(&prefijo.to_ascii_lowercase().replace(' ', ""))
        })
        .collect()
}

/// Lee las etiquetas `clave=valor` separadas por `;` de un registro.
///
/// Devuelve el valor de `clave`, en minúsculas y sin espacios. DMARC y DKIM se
/// escriben los dos así, y los dos admiten espacios alrededor del `;` y del
/// `=`.
fn etiqueta(registro: &str, clave: &str) -> Option<String> {
    registro.split(';').find_map(|parte| {
        let (k, v) = parte.split_once('=')?;
        if k.trim().eq_ignore_ascii_case(clave) {
            Some(v.trim().to_ascii_lowercase())
        } else {
            None
        }
    })
}

/// Lo común a los tres: ¿falta, hay más de uno, o hay exactamente uno?
///
/// Escrito una vez porque el caso de «más de uno invalida» es idéntico para
/// SPF y DMARC, y repetirlo tres veces es tres sitios donde arreglarlo a
/// medias.
/// Se escribe con un patrón de slice y no indexando: `clippy::indexing_slicing`
/// está prohibido en el taller, y con razón — aquí el `[0]` sólo es seguro por
/// el `len()` de la línea de arriba, que es exactamente la clase de garantía
/// que se rompe al editar.
fn el_unico(encontrados: &[String]) -> Result<&String, Hallazgo> {
    match encontrados {
        [] => Err(Hallazgo::Falta),
        [unico] => Ok(unico),
        varios => Err(Hallazgo::Duplicado {
            cuantos: varios.len(),
        }),
    }
}

/// Revisa el SPF del dominio.
///
/// `txt` son los TXT de la raíz del dominio, tal como los devolvió el DNS.
#[must_use]
pub fn revisar_spf(dominio: &str, txt: &[String]) -> Revision {
    let encontrados = los_que_son(txt, "v=spf1");

    let hallazgo = match el_unico(&encontrados) {
        Err(h) => h,
        Ok(registro) => {
            let bajo = registro.to_ascii_lowercase();
            // `+all` y `all` a secas son lo mismo: el `+` es el calificador por
            // defecto. Se buscan los dos, y al final, que es donde manda.
            let palabras: Vec<&str> = bajo.split_whitespace().collect();
            match palabras.last() {
                Some(&"+all" | &"all") => Hallazgo::DemasiadoAbierto,
                // `~all`, `-all` y `?all` cierran el registro como es debido.
                Some(u) if u.ends_with("all") => Hallazgo::EnOrden,
                // Un SPF sin ningún `all` deja el final indefinido.
                _ => Hallazgo::NoSeEntiende,
            }
        }
    };

    Revision {
        que_poner: si_hace_falta(&hallazgo, || QuePoner {
            nombre: dominio.to_string(),
            tipo: TXT,
            // Deliberadamente `None`: ver la cabecera del archivo.
            valor: None,
        }),
        registro: Registro::Spf,
        hallazgo,
    }
}

/// Revisa el DMARC del dominio.
///
/// `txt` son los TXT de `_dmarc.<dominio>`.
#[must_use]
pub fn revisar_dmarc(dominio: &str, txt: &[String]) -> Revision {
    let encontrados = los_que_son(txt, "v=DMARC1");

    let hallazgo = match el_unico(&encontrados) {
        Err(h) => h,
        Ok(registro) => match etiqueta(registro, "p").as_deref() {
            Some("none") => Hallazgo::EnObservacion,
            Some("quarantine" | "reject") => Hallazgo::EnOrden,
            // Sin `p=` no hay DMARC: es la etiqueta obligatoria, y sin ella
            // quien recibe no sabe qué hacer con lo que no pasa.
            _ => Hallazgo::NoSeEntiende,
        },
    };

    Revision {
        que_poner: si_hace_falta(&hallazgo, || QuePoner {
            nombre: format!("_dmarc.{dominio}"),
            tipo: TXT,
            // `p=none` y no `quarantine`, y no es por timidez: subirlo el
            // primer día manda a no deseados el correo propio. Se sube cuando
            // haya unas semanas de informes (`PASO-A-PASO-DEL-CORREO.md`).
            valor: Some(format!("v=DMARC1; p=none; rua=mailto:dmarc@{dominio}")),
        }),
        registro: Registro::Dmarc,
        hallazgo,
    }
}

/// Revisa la clave DKIM de un selector.
///
/// `txt` son los TXT de `<selector>._domainkey.<dominio>`.
///
/// **El selector no se puede adivinar**, y por eso entra como parámetro en vez
/// de salir de aquí: lo elige el proveedor de correo al dar de alta la firma.
/// Quien no lo sepa no tiene un DKIM roto: tiene un DKIM que no se ha podido
/// mirar, y eso lo decide quien llama, no este archivo.
#[must_use]
pub fn revisar_dkim(dominio: &str, selector: &str, txt: &[String]) -> Revision {
    let encontrados = los_que_son(txt, "v=DKIM1");

    let hallazgo = match el_unico(&encontrados) {
        Err(h) => h,
        Ok(registro) => match etiqueta(registro, "p") {
            // `p=` vacío es la forma estándar de revocar una clave: el
            // registro está y ninguna firma suya vale.
            Some(clave) if clave.is_empty() => Hallazgo::ClaveRetirada,
            Some(_) => Hallazgo::EnOrden,
            None => Hallazgo::NoSeEntiende,
        },
    };

    Revision {
        que_poner: si_hace_falta(&hallazgo, || QuePoner {
            nombre: format!("{selector}._domainkey.{dominio}"),
            tipo: TXT,
            // La clave pública la genera el proveedor y es distinta cada vez.
            // Misma razón que el `include:` del SPF.
            valor: None,
        }),
        registro: Registro::Dkim,
        hallazgo,
    }
}

/// `QuePoner` sólo cuando hay algo que arreglar.
///
/// Un «qué poner» junto a un registro correcto invita a cambiarlo, y cambiar
/// un SPF que funciona es exactamente el accidente que esto existe para
/// evitar.
fn si_hace_falta(hallazgo: &Hallazgo, construir: impl FnOnce() -> QuePoner) -> Option<QuePoner> {
    if hallazgo.esta_bien() {
        None
    } else {
        Some(construir())
    }
}

/// El veredicto de los tres juntos, para el punto 8 del preflight.
///
/// Devuelve lo que `Observaciones::registros_del_dominio` espera:
///
/// - `Some(true)` — los tres están y sirven.
/// - `Some(false)` — alguno falta o no sirve.
/// - `None` — **no se pudo mirar**, que no es un aprobado (§ del preflight:
///   «omitida no es pasada»). Es lo que devuelve una lista vacía: sin haber
///   consultado nada, decir que falla acusaría a un dominio bien configurado.
#[must_use]
pub fn en_orden(revisiones: &[Revision]) -> Option<bool> {
    if revisiones.is_empty() {
        return None;
    }
    Some(revisiones.iter().all(|r| r.hallazgo.esta_bien()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOMINIO: &str = "telemetrymx.com";

    fn txt(valores: &[&str]) -> Vec<String> {
        valores.iter().map(|v| (*v).to_string()).collect()
    }

    // ── SPF ────────────────────────────────────────────────────────────────

    #[test]
    fn un_spf_que_cierra_con_tilde_all_esta_en_orden() {
        let r = revisar_spf(DOMINIO, &txt(&["v=spf1 include:spf.titan.email ~all"]));
        assert_eq!(r.hallazgo, Hallazgo::EnOrden);
        assert!(
            r.que_poner.is_none(),
            "un registro correcto no invita a tocarlo"
        );
    }

    /// **El SPF convive con los TXT de verificación de otros servicios.**
    ///
    /// Es el caso normal de una raíz de dominio, y la razón por la que no se
    /// cuentan «los TXT» sino los que empiezan por `v=spf1`. Verificada
    /// quitando el filtro de prefijo: con él fuera, esto da
    /// `Duplicado { cuantos: 3 }` sobre un dominio perfectamente configurado.
    #[test]
    fn los_txt_de_otros_servicios_no_cuentan_como_spf() {
        let r = revisar_spf(
            DOMINIO,
            &txt(&[
                "google-site-verification=abc123",
                "v=spf1 include:spf.titan.email ~all",
                "MS=ms12345678",
            ]),
        );
        assert_eq!(r.hallazgo, Hallazgo::EnOrden);
    }

    /// **Dos SPF no son un SPF más estricto: no son ninguno.**
    ///
    /// Quien recibe descarta los dos, así que el dominio queda sin SPF
    /// mientras en el panel del DNS se ven dos registros correctos.
    #[test]
    fn dos_spf_invalidan_el_dominio() {
        let r = revisar_spf(
            DOMINIO,
            &txt(&[
                "v=spf1 include:spf.titan.email ~all",
                "v=spf1 include:_spf.google.com ~all",
            ]),
        );
        assert_eq!(r.hallazgo, Hallazgo::Duplicado { cuantos: 2 });
        assert!(r.que_poner.is_some());
    }

    /// **`+all` es peor que no tener SPF**, y por eso no es `EnOrden`.
    ///
    /// Declara por escrito que cualquiera puede enviar en nombre del dominio.
    /// Se prueban las dos formas porque el `+` es el calificador por defecto y
    /// `all` a secas significa lo mismo — tratarlas distinto dejaría pasar la
    /// más corta, que es la que se escribe por descuido.
    #[test]
    fn un_spf_que_autoriza_a_todo_el_mundo_no_esta_en_orden() {
        for abierto in ["v=spf1 include:spf.titan.email +all", "v=spf1 all"] {
            assert_eq!(
                revisar_spf(DOMINIO, &txt(&[abierto])).hallazgo,
                Hallazgo::DemasiadoAbierto,
                "{abierto}"
            );
        }
    }

    #[test]
    fn un_spf_sin_all_no_se_entiende() {
        let r = revisar_spf(DOMINIO, &txt(&["v=spf1 include:spf.titan.email"]));
        assert_eq!(r.hallazgo, Hallazgo::NoSeEntiende);
    }

    /// **Lo que se ofrece arreglar no lleva el `include:` inventado.**
    ///
    /// Es la decisión de la cabecera, y se comprueba en vez de confiarla al
    /// comentario: un valor sugerido aquí se copia, se pega, y deja de
    /// autorizar al servidor que de verdad envía.
    #[test]
    fn al_faltar_el_spf_arles_no_se_inventa_el_include() {
        let r = revisar_spf(DOMINIO, &txt(&[]));
        assert_eq!(r.hallazgo, Hallazgo::Falta);
        let q = r.que_poner.expect("falta: tiene que decir qué poner");
        assert_eq!(q.nombre, DOMINIO, "el SPF va en la raíz del dominio");
        assert_eq!(q.tipo, "TXT");
        assert!(q.valor.is_none(), "ARLES no sabe el include del proveedor");
    }

    // ── DMARC ──────────────────────────────────────────────────────────────

    /// **La distinción que motiva este archivo.**
    ///
    /// `p=none` cuenta como correcto —es lo que hay que poner al empezar— pero
    /// no es `EnOrden`: no rechaza nada. Las dos mitades se comprueban juntas
    /// porque cada una sin la otra es un error distinto: sólo la primera lo
    /// pintaría en rojo sobre la configuración que el proyecto recomienda, y
    /// sólo la segunda diría que el dominio está protegido cuando no lo está.
    #[test]
    fn un_dmarc_en_none_cuenta_como_correcto_pero_no_como_en_orden() {
        let r = revisar_dmarc(DOMINIO, &txt(&["v=DMARC1; p=none; rua=mailto:a@b.com"]));
        assert_eq!(r.hallazgo, Hallazgo::EnObservacion);
        assert!(r.hallazgo.esta_bien(), "p=none es lo correcto al empezar");
        assert_ne!(r.hallazgo, Hallazgo::EnOrden, "p=none no rechaza nada");
    }

    #[test]
    fn un_dmarc_que_rechaza_o_pone_en_cuarentena_esta_en_orden() {
        for politica in ["v=DMARC1; p=quarantine", "v=DMARC1; p=reject; pct=100"] {
            assert_eq!(
                revisar_dmarc(DOMINIO, &txt(&[politica])).hallazgo,
                Hallazgo::EnOrden,
                "{politica}"
            );
        }
    }

    /// **Un DMARC sin `p=` no es un DMARC.**
    ///
    /// Es la etiqueta obligatoria: sin ella quien recibe no sabe qué hacer con
    /// lo que no pasa, así que el registro existe y no sirve de nada.
    #[test]
    fn un_dmarc_sin_politica_no_se_entiende() {
        let r = revisar_dmarc(DOMINIO, &txt(&["v=DMARC1; rua=mailto:a@b.com"]));
        assert_eq!(r.hallazgo, Hallazgo::NoSeEntiende);
    }

    /// **El valor sugerido empieza en `p=none`, y es a propósito.**
    ///
    /// Sugerir `p=quarantine` de entrada es la forma más rápida de que el
    /// correo propio deje de entregarse, que es lo que
    /// `PASO-A-PASO-DEL-CORREO.md` manda evitar.
    #[test]
    fn al_faltar_el_dmarc_lo_que_se_sugiere_empieza_en_observacion() {
        let r = revisar_dmarc(DOMINIO, &txt(&[]));
        assert_eq!(r.hallazgo, Hallazgo::Falta);
        let q = r.que_poner.expect("falta: tiene que decir qué poner");
        assert_eq!(q.nombre, format!("_dmarc.{DOMINIO}"));
        let v = q.valor.expect("el DMARC sí se puede escribir entero");
        assert!(v.contains("p=none"), "{v}");
        assert!(!v.contains("quarantine") && !v.contains("reject"), "{v}");
    }

    /// Dos DMARC se anulan igual que dos SPF, y es el caso que
    /// `PASO-A-PASO-DEL-CORREO.md` ya encontró una vez de verdad.
    #[test]
    fn dos_dmarc_invalidan_el_dominio() {
        let r = revisar_dmarc(
            DOMINIO,
            &txt(&["v=DMARC1; p=none", "v=DMARC1; p=quarantine"]),
        );
        assert_eq!(r.hallazgo, Hallazgo::Duplicado { cuantos: 2 });
    }

    // ── DKIM ───────────────────────────────────────────────────────────────

    #[test]
    fn una_clave_dkim_presente_esta_en_orden() {
        let r = revisar_dkim(DOMINIO, "titan1", &txt(&["v=DKIM1; k=rsa; p=MIIBIjANBgkq"]));
        assert_eq!(r.hallazgo, Hallazgo::EnOrden);
    }

    /// **Una clave retirada no es una clave que falta**, y se distinguen
    /// porque la acción es otra: una se pone, la otra se vuelve a generar en
    /// el panel del proveedor.
    #[test]
    fn una_clave_dkim_retirada_se_distingue_de_una_que_falta() {
        let retirada = revisar_dkim(DOMINIO, "titan1", &txt(&["v=DKIM1; k=rsa; p="]));
        assert_eq!(retirada.hallazgo, Hallazgo::ClaveRetirada);

        let ausente = revisar_dkim(DOMINIO, "titan1", &txt(&[]));
        assert_eq!(ausente.hallazgo, Hallazgo::Falta);

        assert_ne!(retirada.hallazgo, ausente.hallazgo);
    }

    #[test]
    fn el_nombre_del_dkim_lleva_el_selector_y_domainkey() {
        let r = revisar_dkim(DOMINIO, "titan1", &txt(&[]));
        let q = r.que_poner.expect("falta: tiene que decir qué poner");
        assert_eq!(q.nombre, format!("titan1._domainkey.{DOMINIO}"));
    }

    // ── Cosas de formato que llegan del DNS ────────────────────────────────

    /// **Un TXT puede llegar entrecomillado**, según de dónde se lea.
    ///
    /// Sin quitar las comillas, `"v=spf1 ..."` no empieza por `v=spf1` y un
    /// dominio bien configurado saldría como «falta el SPF» — un hallazgo
    /// falso sobre la configuración correcta, que es el error que más caro
    /// sale (auditoría B).
    #[test]
    fn un_txt_entrecomillado_se_lee_igual() {
        let r = revisar_spf(DOMINIO, &txt(&["\"v=spf1 include:spf.titan.email ~all\""]));
        assert_eq!(r.hallazgo, Hallazgo::EnOrden);
    }

    /// Las etiquetas admiten espacios y mayúsculas: `V=DMARC1 ; P = None` es
    /// un DMARC válido y hay paneles de DNS que lo escriben así.
    #[test]
    fn las_etiquetas_se_leen_con_espacios_y_mayusculas() {
        let r = revisar_dmarc(DOMINIO, &txt(&["V=DMARC1 ; P = None ; rua=mailto:a@b.com"]));
        assert_eq!(r.hallazgo, Hallazgo::EnObservacion);
    }

    // ── El veredicto de los tres juntos ────────────────────────────────────

    /// **Sin haber consultado nada, la respuesta es «no se sabe»**, no «falla».
    ///
    /// Es la regla del preflight —omitida no es pasada, y tampoco es
    /// suspensa—: devolver `Some(false)` acusaría de un fallo a un dominio
    /// bien configurado cuando lo que pasó es que no hubo DNS.
    #[test]
    fn sin_revisiones_no_se_sabe() {
        assert_eq!(en_orden(&[]), None);
    }

    #[test]
    fn los_tres_bien_dan_un_si() {
        let revisiones = vec![
            revisar_spf(DOMINIO, &txt(&["v=spf1 include:spf.titan.email ~all"])),
            revisar_dkim(DOMINIO, "titan1", &txt(&["v=DKIM1; p=MIIBIjANBgkq"])),
            revisar_dmarc(DOMINIO, &txt(&["v=DMARC1; p=none"])),
        ];
        assert_eq!(
            en_orden(&revisiones),
            Some(true),
            "p=none cuenta como correcto"
        );
    }

    /// Uno roto basta para que el conjunto no esté en orden. Se prueba con los
    /// otros dos correctos, que es el caso que distingue «mira todos» de
    /// «mira el primero».
    #[test]
    fn uno_roto_tumba_el_conjunto() {
        let revisiones = vec![
            revisar_spf(DOMINIO, &txt(&["v=spf1 include:spf.titan.email ~all"])),
            revisar_dkim(DOMINIO, "titan1", &txt(&["v=DKIM1; p=MIIBIjANBgkq"])),
            revisar_dmarc(DOMINIO, &txt(&[])),
        ];
        assert_eq!(en_orden(&revisiones), Some(false));
    }
}
