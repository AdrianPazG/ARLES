//! Sustituir `{{variable}}` en un mensaje. Y nada más.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! ESTE MOTOR NO SABE HACER NADA MÁS QUE SUSTITUIR, Y ES LO IMPORTANTE
//!
//! No hay condicionales. No hay bucles. No hay funciones, filtros, ni
//! aritmética. No se puede escribir `{{#if}}`, ni `{{contacto.nombre|mayus}}`,
//! ni nada que se parezca. **Sólo `{{clave}}` contra una lista cerrada de
//! claves conocidas.**
//!
//! La tentación de usar una biblioteca de plantillas hecha —Handlebars, Tera—
//! es real y se rechaza a propósito (ADR-0006). Esas bibliotecas evalúan
//! expresiones, y aquí las plantillas van a llevar **datos de contacto
//! importados de un XLSX que no escribimos nosotros**. Un motor con lógica
//! sobre datos ajenos es una vía de ejecución, y el precio de evitarla es
//! escribir estas ciento y pico líneas.
//!
//! También hay un precio de producto, y es pequeño: no se puede escribir «Hola
//! {{nombre}}» y que ponga «Hola» a secas cuando no hay nombre. Para eso está
//! el valor de reserva, que se elige al definir la variable y no dentro de la
//! plantilla.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LO QUE SE SUSTITUYE NO SE VUELVE A MIRAR
//!
//! El resultado de una sustitución **no se analiza otra vez**. Si el nombre de
//! un contacto fuera literalmente `{{empresa}}`, se queda así en el mensaje:
//! no se expande.
//!
//! Sin esta regla, un contacto importado de un archivo ajeno podría escribir
//! plantilla — y eso es exactamente la vía de ejecución que este módulo existe
//! para cerrar. Hay una prueba que lo sostiene.

use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;

/// Las variables que ARLES conoce. **La lista es cerrada.**
///
/// Cerrada y no abierta a propósito: una lista abierta —«cualquier columna del
/// archivo importado»— convertiría el nombre de una columna en el nombre de una
/// variable, y entonces el archivo de un tercero decidiría qué se puede escribir
/// en una plantilla.
///
/// Crecer está previsto: se añade una variante aquí, su texto en la interfaz, y
/// el `match` de [`Variable::de_clave`] deja de compilar hasta que alguien
/// decida de dónde sale el valor. Ese error de compilación es la característica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Variable {
    /// El nombre de pila del contacto.
    Nombre,
    /// Su apellido.
    Apellido,
    /// La empresa del contacto, no la que envía.
    Empresa,
    /// El nombre comercial de **quien envía**. Sale de la configuración, no
    /// del contacto: es el mismo para toda la campaña.
    MiEmpresa,
    /// **La salida de quien recibe**: cómo dejar de recibir estos correos.
    ///
    /// ── POR QUÉ ES UNA VARIABLE Y NO UN TEXTO QUE SE ESCRIBE ─────────────
    ///
    /// Porque la frase no la puede escribir quien redacta: lleva **la
    /// dirección del remitente de esta campaña**, y esa dirección la elige el
    /// asistente en un paso posterior al de la plantilla. Una plantilla con la
    /// dirección escrita a mano queda mintiendo en cuanto se use con otro
    /// remitente — y la baja llegaría a un buzón que nadie vacía.
    ///
    /// ── Y POR QUÉ IMPORTA MÁS QUE LAS OTRAS TRES ──────────────────────────
    ///
    /// Medido el 22/10/2026: la cabecera `List-Unsubscribe` sí se manda, pero
    /// **el cuerpo del correo no llevaba ninguna salida y nada lo exigía**. La
    /// cabecera la leen los filtros y los clientes que la soportan; **la
    /// persona que está leyendo el correo busca en el cuerpo**, y quien no la
    /// encuentra usa el botón de spam — que es la señal que más daño hace a la
    /// reputación del dominio.
    ///
    /// Así que dar una salida fácil no es ser amable: es lo que evita la queja.
    Baja,
}

impl Variable {
    /// Todas, en el orden en que se ofrecen al escribir una plantilla.
    ///
    /// Primero las del contacto, que son las que se usan; la de la empresa
    /// propia al final, porque casi siempre ya está escrita en la firma.
    pub const TODAS: &'static [Self] = &[
        Self::Nombre,
        Self::Apellido,
        Self::Empresa,
        Self::MiEmpresa,
        Self::Baja,
    ];

    /// Cómo se escribe dentro de `{{ }}`.
    ///
    /// **Es un contrato con las plantillas ya guardadas.** Cambiar una de estas
    /// cadenas deja de resolver la variable en todas las plantillas escritas
    /// hasta hoy — y no falla: el texto saldría con el `{{nombre}}` en crudo o
    /// con el valor de reserva. Hay una prueba que las congela.
    #[must_use]
    pub const fn clave(self) -> &'static str {
        match self {
            Self::Nombre => "nombre",
            Self::Apellido => "apellido",
            Self::Empresa => "empresa",
            Self::MiEmpresa => "mi_empresa",
            Self::Baja => "baja",
        }
    }

    /// ¿Habla de **quien recibe** el correo?
    ///
    /// ── POR QUÉ ESTA PREGUNTA EXISTE (R-04, 16/10/2026) ──────────────────
    ///
    /// Dirección lo dijo así: *«sería una tragedia que apareciera en la firma
    /// el nombre del contacto a quien se le está haciendo llegar el correo y
    /// su empresa, en lugar de los datos del remitente»*. Y tiene razón: una
    /// firma con `{{nombre}}` no pone el de quien firma — pone **el de quien
    /// recibe**, en cada correo de la campaña, debajo del mensaje y donde más
    /// se mira.
    ///
    /// El formulario de plantillas y el de firmas son el mismo, y ofrecía las
    /// cuatro variables en los dos casos. Tres de ellas son del destinatario.
    ///
    /// `MiEmpresa` es la única que habla de quien envía, y por eso es la única
    /// que una firma puede usar.
    #[must_use]
    pub const fn es_de_quien_recibe(self) -> bool {
        match self {
            Self::Nombre | Self::Apellido | Self::Empresa => true,
            // `Baja` habla de quien recibe —es SU salida— pero el valor sale
            // del remitente, que es lo que esta pregunta decide: si una firma
            // puede usarla. Y debe poder, porque el pie de baja en la firma es
            // justo donde la gente lo espera.
            Self::MiEmpresa | Self::Baja => false,
        }
    }

    /// Las que una **firma** puede llevar.
    ///
    /// Se deriva de `es_de_quien_recibe` y no es otra lista: dos listas de lo
    /// mismo se separan en cuanto alguien añada una variable, y la que se
    /// quedaría atrás es justo la que protege.
    #[must_use]
    pub fn de_quien_firma() -> Vec<Self> {
        Self::TODAS
            .iter()
            .copied()
            .filter(|v| !v.es_de_quien_recibe())
            .collect()
    }

    /// La variable que corresponde a una clave, si existe.
    #[must_use]
    pub fn de_clave(clave: &str) -> Option<Self> {
        Self::TODAS.iter().copied().find(|v| v.clave() == clave)
    }

    /// La clave del texto que la describe en la interfaz (§139).
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// DECÍA `miEmpresa` Y EL CATÁLOGO DICE `mi_empresa`
    ///
    /// Encontrado el 01/10/2026, en la auditoría de la 7.7. La ruta
    /// `plantilla.variable.miEmpresa` **no existe**: el catálogo la tiene como
    /// `mi_empresa`, igual que [`Self::clave`].
    ///
    /// No se vio por tres razones a la vez, y las tres son el §9: nadie llama a
    /// este método —las pantallas construyen la clave con `clave()`, que sí
    /// valía—; la comprobación del §139 comparaba **la hoja** `mi_empresa`, que
    /// existe, en vez de la ruta; y su lista de prefijos no incluía `plantilla`.
    ///
    /// Ahora la prueba `la_clave_de_i18n_se_deriva_de_la_clave` ata las dos, así
    /// que no pueden volver a separarse sin que algo falle.
    #[must_use]
    pub const fn clave_i18n(self) -> &'static str {
        match self {
            Self::Nombre => "plantilla.variable.nombre",
            Self::Apellido => "plantilla.variable.apellido",
            Self::Empresa => "plantilla.variable.empresa",
            Self::MiEmpresa => "plantilla.variable.mi_empresa",
            Self::Baja => "plantilla.variable.baja",
        }
    }
}

/// La frase de baja que sustituye a `{{baja}}`, con la dirección a la que
/// escribir.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ES UNA RESPUESTA Y NO UN ENLACE
///
/// La baja de un clic (RFC 8058) necesita que alguien reciba un `POST` en una
/// dirección web, y **ARLES es una aplicación de escritorio sin servidor** —
/// la misma razón por la que `List-Unsubscribe` va como `mailto:` y no como
/// `https:` (ver `arles-email/src/smtp.rs`). Prometer un botón que no existe
/// lo comprueban los filtros, y quien lo pulsara no se daría de baja.
///
/// Así que se dice lo que de verdad funciona: **responder a este correo**. La
/// respuesta llega al buzón del remitente y alguien la apunta en Contactos.
///
/// ── Y ESO ES UNA OBLIGACIÓN QUE SE HEREDA ─────────────────────────────────
///
/// Si ese buzón no se vacía, la persona pidió la baja y ARLES le sigue
/// escribiendo — que es peor que no haber dado la salida, porque la pidió
/// creyendo que servía. Está declarado en `RIESGOS-DE-REPUTACION.md` §R-7 como
/// una tarea semanal de quien envía, y como **R-25** desde el 23/10/2026:
/// Dirección pidió que ARLES dé de baja solo, y la recomendación es leer el
/// buzón por IMAP. Hasta que exista, la tarea semanal es la única defensa.
///
/// ── EL TEXTO VIVE EN RUST, Y ESO NO CONTRADICE EL §139 ────────────────────
///
/// El §139 saca los textos de Rust porque **la interfaz** se traduce y se
/// revisa. Esto no es interfaz: es el cuerpo de un correo que sale en español
/// de México, igual que el `subject=Baja` de la cabecera. Ponerlo en el
/// catálogo de la webview obligaría a que un texto del correo cruzara la
/// frontera para volver, y el motor envía sin ventana abierta (§51).
#[must_use]
pub fn frase_de_baja(remitente: &str) -> String {
    format!(
        "Si no quieres recibir más correos nuestros, responde a este mensaje \
         con la palabra BAJA y te quitamos de la lista. También puedes \
         escribirnos a {}.",
        crate::sanear::escapar_para_html(remitente)
    )
}

/// ¿Este texto le da a quien lo recibe una forma de darse de baja?
///
/// Mira si usa la variable [`Variable::Baja`], que es la única forma de que la
/// frase lleve la dirección correcta del remitente de **esta** campaña.
///
/// ── NO BUSCA PALABRAS, Y ES LA DIFERENCIA ENTRE MEDIR Y CONFIAR ───────────
///
/// La otra forma de escribir esto sería buscar «baja», «unsubscribe» o un
/// `mailto:` en el cuerpo. Y pasaría en verde con una plantilla que dice «este
/// mes damos de baja los precios» — sin ninguna salida, con una comprobación
/// contenta. Lo que se puede afirmar de una variable es que **ARLES la
/// sustituye por una salida que funciona**; de una palabra no se puede afirmar
/// nada.
#[must_use]
pub fn tiene_salida_de_baja(cuerpo_html: &str) -> bool {
    Plantilla::nueva(cuerpo_html, SiFalta::Nada)
        .variables()
        .contains(&Variable::Baja)
}

impl fmt::Display for Variable {
    /// Cómo se escribe entera, lista para pegar en una plantilla.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{{{}}}}}", self.clave())
    }
}

/// Lo que se sabe de un contacto a la hora de escribirle.
///
/// Es un mapa cerrado y no un diccionario libre: ver la nota de [`Variable`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Valores(BTreeMap<Variable, String>);

impl Valores {
    #[must_use]
    pub fn nuevos() -> Self {
        Self::default()
    }

    /// Da valor a una variable. Un valor en blanco cuenta como **ausente**.
    ///
    /// Que el blanco cuente como ausente no es pereza: en un archivo importado,
    /// una celda vacía y una celda con un espacio son lo mismo para quien la
    /// escribió, y tratarlas distinto haría que la mitad de los contactos
    /// recibieran «Hola  ,» y la otra mitad el valor de reserva.
    #[must_use]
    pub fn con(mut self, variable: Variable, valor: &str) -> Self {
        let valor = valor.trim();
        if !valor.is_empty() {
            self.0.insert(variable, valor.to_owned());
        }
        self
    }

    /// El valor de una variable, si lo hay.
    #[must_use]
    pub fn de(&self, variable: Variable) -> Option<&str> {
        self.0.get(&variable).map(String::as_str)
    }
}

/// Qué poner cuando una variable no tiene valor para este contacto.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTO ES UNA DECISIÓN Y NO UN DETALLE
///
/// Es el caso más común de una importación real: una columna a medio rellenar.
/// Y las dos salidas fáciles son malas.
///
/// **Dejar el hueco** produce «Hola ,» — que se lee como un error del programa,
/// y encima delata que era un correo automático.
///
/// **Dejar el `{{nombre}}` en crudo** es peor: se manda a un cliente el nombre
/// de una variable, y ahí ya no hay duda de que nadie lo revisó.
///
/// Así que el valor de reserva se elige **al escribir la plantilla** y el
/// preflight cuenta a cuántos contactos les va a tocar, para que se pueda
/// decidir con una cifra delante y no a ciegas (§94).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SiFalta {
    /// Se pone lo que se haya escrito. Vacío es una opción legítima.
    #[default]
    Nada,
    /// Un texto fijo: «cliente», «equipo», lo que se elija.
    Esto(String),
}

/// Una plantilla lista para rellenar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plantilla {
    texto: String,
    si_falta: SiFalta,
}

/// Lo que quedó al rellenar una plantilla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rellenada {
    /// El texto, con las variables ya sustituidas.
    pub texto: String,
    /// Qué variables no tenían valor y se rellenaron con la reserva.
    ///
    /// Se devuelve en vez de tragársela porque es lo que el preflight cuenta
    /// para decir «a 34 contactos les falta el nombre» antes de activar.
    pub faltaron: Vec<Variable>,
}

/// Comprueba que un texto sirve como **firma**.
///
/// ── LA GUARDA DE R-04, Y POR QUÉ VIVE AQUÍ ──────────────────────────────
///
/// Una firma con `{{nombre}}` pone el nombre de **quien recibe** debajo del
/// mensaje, en cada correo de la campaña. Dirección lo llamó «una tragedia»
/// el 16/10/2026, y lo es: no se nota al escribirla, se nota cuando ya salió.
///
/// El formulario de plantillas y el de firmas son el mismo, y ofrecía las
/// cuatro variables en los dos casos. **La pantalla ya no las ofrece**, pero
/// eso sólo cubre el camino de quien pulsa un botón: quedan las escritas a
/// mano, las que entren en un respaldo restaurado, y cualquier formulario
/// futuro que olvide filtrar.
///
/// Por eso la comprobación está en el núcleo, donde no se puede rodear.
///
/// # Errores
///
/// [`CoreError::FirmaConVariablesDelContacto`] nombrando **todas** las que
/// encontró, no la primera: con una lista de una, corregirla y volver a
/// guardar enseñaría la siguiente, y así una por intento.
pub fn revisar_firma(texto: &str) -> Result<(), crate::CoreError> {
    let ajenas: Vec<&str> = Plantilla::nueva(texto, SiFalta::Nada)
        .variables()
        .into_iter()
        .filter(|v| v.es_de_quien_recibe())
        .map(Variable::clave)
        .collect();

    if ajenas.is_empty() {
        return Ok(());
    }
    Err(crate::CoreError::FirmaConVariablesDelContacto {
        cuales: ajenas.join(", "),
    })
}

impl Plantilla {
    /// Prepara una plantilla.
    #[must_use]
    pub fn nueva(texto: &str, si_falta: SiFalta) -> Self {
        Self {
            texto: texto.to_owned(),
            si_falta,
        }
    }

    /// Las variables que esta plantilla usa, sin repetir y en orden.
    ///
    /// Lo que **no** reconoce —`{{fecha}}`, `{{ nombre }}` con espacios, un
    /// `{{` sin cerrar— no aparece aquí y tampoco se sustituye: sale tal cual.
    /// Es deliberado: una plantilla que se «arregla» sola esconde la errata
    /// hasta que el correo ya salió.
    #[must_use]
    pub fn variables(&self) -> Vec<Variable> {
        let mut vistas = Vec::new();
        for v in recorrer(&self.texto).filter_map(|t| match t {
            Trozo::Variable(v) => Some(v),
            Trozo::Texto(_) => None,
        }) {
            if !vistas.contains(&v) {
                vistas.push(v);
            }
        }
        vistas
    }

    /// Rellena la plantilla con lo que se sepa de un contacto.
    ///
    /// **Lo sustituido no se vuelve a mirar**: si un valor contiene
    /// `{{empresa}}`, se queda así. Ver la nota de la cabecera.
    #[must_use]
    pub fn rellenar(&self, valores: &Valores) -> Rellenada {
        let mut texto = String::with_capacity(self.texto.len());
        let mut faltaron = Vec::new();

        for trozo in recorrer(&self.texto) {
            match trozo {
                // El texto de alrededor se copia tal cual.
                Trozo::Texto(t) => texto.push_str(t),
                Trozo::Variable(v) => match valores.de(v) {
                    Some(valor) => texto.push_str(valor),
                    None => {
                        if !faltaron.contains(&v) {
                            faltaron.push(v);
                        }
                        if let SiFalta::Esto(reserva) = &self.si_falta {
                            texto.push_str(reserva);
                        }
                    }
                },
            }
        }

        Rellenada { texto, faltaron }
    }
}

/// Un trozo del texto: o es texto, o es una variable reconocida.
#[derive(Debug, PartialEq, Eq)]
enum Trozo<'a> {
    Texto(&'a str),
    Variable(Variable),
}

/// Parte el texto en trozos. **Es todo el análisis que se hace.**
///
/// Recorre buscando `{{`, y sólo si lo que sigue es una clave conocida cerrada
/// por `}}` lo trata como variable. Cualquier otra cosa es texto.
///
/// No hay estado, no hay pila, no hay recursión: por eso no puede evaluar nada.
fn recorrer(texto: &str) -> impl Iterator<Item = Trozo<'_>> {
    let mut resto = texto;
    std::iter::from_fn(move || {
        if resto.is_empty() {
            return None;
        }

        let Some(abre) = resto.find("{{") else {
            let todo = resto;
            resto = "";
            return Some(Trozo::Texto(todo));
        };

        // Hay texto antes de la llave: se devuelve primero.
        if abre > 0 {
            let (antes, despues) = resto.split_at(abre);
            resto = despues;
            return Some(Trozo::Texto(antes));
        }

        // `resto` empieza por `{{`. ¿Cierra, y con una clave conocida?
        let dentro = &resto[2..];
        match dentro
            .find("}}")
            .and_then(|cierra| Variable::de_clave(&dentro[..cierra]).map(|v| (v, cierra)))
        {
            Some((variable, cierra)) => {
                resto = &dentro[cierra + 2..];
                Some(Trozo::Variable(variable))
            }
            // No cierra, o la clave no existe: las dos llaves son texto. Se
            // devuelven solas para que la búsqueda siga después de ellas y un
            // `{{{{nombre}}` no se quede dando vueltas.
            None => {
                resto = dentro;
                Some(Trozo::Texto("{{"))
            }
        }
    })
}

/// Un correo ya redactado para una persona concreta.
///
/// Las dos piezas que necesita el adaptador, y nada más: lo que lleva dentro
/// el sobre. **No lleva la dirección** — ésa viene del intento de la cola, y
/// mezclarlas aquí daría dos sitios de donde sacar a quién se escribe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorreoRedactado {
    pub asunto: String,
    /// Ya con las variables puestas y la firma pegada. **Los datos de la
    /// persona van escapados**: ver [`redactar`].
    pub html: String,
    /// Qué variables no tenían valor para esta persona.
    ///
    /// Viaja para poder decirlo, no para decidir nada: quien quiera no
    /// escribir a quien le falten datos lo decide antes, en el preflight.
    pub faltaron: Vec<Variable>,
}

/// Redacta el correo de una persona: la plantilla con sus datos, y la firma
/// detrás.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LOS DATOS DE LA PERSONA SE ESCAPAN, Y EL ASUNTO NO
///
/// Y es a propósito en los dos casos. El cuerpo es HTML, así que un nombre con
/// un `<` tiene que entrar como `&lt;` — si no, una empresa llamada
/// `<script>…` se convierte en etiqueta dentro de un HTML que ya pasó su
/// saneado (ver [`crate::sanear::escapar_para_html`]).
///
/// El asunto **no es HTML**: viaja en una cabecera del correo y se enseña tal
/// cual. Escaparlo ahí haría que una empresa llamada `Pérez & Hijos` apareciera
/// como `Pérez &amp; Hijos` en la bandeja de quien recibe.
///
/// Son los mismos valores rellenando dos cosas distintas, y por eso se rellena
/// dos veces con dos juegos: hacerlo con uno solo rompe una de las dos, y la
/// que se rompe no falla en ninguna parte.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA FIRMA SE PEGA DETRÁS, Y NO PASA POR LAS VARIABLES DEL CONTACTO
///
/// Una firma no puede llevar variables de quien recibe —es R-04, y lo impide
/// `revisar_firma` antes de guardarla— así que aquí se rellena con **los
/// mismos valores**, que es lo correcto: `{{mi_empresa}}` es de quien firma y
/// tiene que funcionar también en la firma.
///
/// Va dentro de un `<p>` y separada por una regla: una firma pegada al último
/// párrafo se lee como parte del mensaje.
#[must_use]
pub fn redactar(
    asunto: &str,
    cuerpo_html: &str,
    firma_html: Option<&str>,
    valores: &Valores,
    si_falta: SiFalta,
) -> CorreoRedactado {
    // Los mismos valores, escapados para lo que va dentro del HTML.
    let mut para_html = Valores::nuevos();
    for v in Variable::TODAS {
        if let Some(valor) = valores.de(*v) {
            para_html = para_html.con(*v, &crate::sanear::escapar_para_html(valor));
        }
    }

    let cuerpo = Plantilla::nueva(cuerpo_html, si_falta.clone()).rellenar(&para_html);
    let asunto = Plantilla::nueva(asunto, si_falta.clone()).rellenar(valores);

    let mut html = cuerpo.texto;
    if let Some(firma) = firma_html.filter(|f| !f.trim().is_empty()) {
        let firma = Plantilla::nueva(firma, si_falta).rellenar(&para_html);
        html.push_str("<p>—</p>");
        html.push_str(&firma.texto);
    }

    // Lo que faltó es de las dos: una variable que sólo está en el asunto
    // también falta. Sin unirlas, un asunto con `{{nombre}}` y un cuerpo sin
    // él diría que no faltó nada.
    let mut faltaron = cuerpo.faltaron;
    for v in asunto.faltaron {
        if !faltaron.contains(&v) {
            faltaron.push(v);
        }
    }

    CorreoRedactado {
        asunto: asunto.texto,
        html,
        faltaron,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ana() -> Valores {
        Valores::nuevos()
            .con(Variable::Nombre, "Ana")
            .con(Variable::Empresa, "Panadería Rosa")
    }

    #[test]
    fn sustituye_lo_que_conoce_y_deja_el_resto() {
        let p = Plantilla::nueva("Hola {{nombre}}, ¿cómo va {{empresa}}?", SiFalta::Nada);
        assert_eq!(
            p.rellenar(&ana()).texto,
            "Hola Ana, ¿cómo va Panadería Rosa?"
        );
    }

    /// **Lo sustituido no se vuelve a analizar.**
    ///
    /// Es la garantía que cierra la vía de ejecución. Los datos de contacto
    /// vienen de un archivo que no escribimos nosotros; si el valor de una
    /// variable pudiera contener plantilla, quien escribiera ese archivo
    /// escribiría el mensaje.
    ///
    /// Verificada rompiéndola el 29/09/2026: haciendo que `rellenar` se llame a
    /// sí misma sobre el resultado, esta prueba falla.
    #[test]
    fn un_valor_que_parece_plantilla_no_se_expande() {
        let travieso = Valores::nuevos()
            .con(Variable::Nombre, "{{empresa}}")
            .con(Variable::Empresa, "ACME");

        let p = Plantilla::nueva("Hola {{nombre}}", SiFalta::Nada);
        assert_eq!(
            p.rellenar(&travieso).texto,
            "Hola {{empresa}}",
            "el valor de un contacto no puede escribir plantilla"
        );
    }

    /// **Una variable que no existe se queda tal cual.**
    ///
    /// No se borra ni se «arregla»: una plantilla que se corrige sola esconde
    /// la errata hasta que el correo ya salió, y entonces el cliente recibe un
    /// hueco sin que nadie sepa por qué.
    #[test]
    fn una_variable_que_no_existe_sale_en_crudo() {
        let p = Plantilla::nueva("Hola {{apodo}} de {{empresa}}", SiFalta::Nada);
        assert_eq!(p.rellenar(&ana()).texto, "Hola {{apodo}} de Panadería Rosa");
        assert_eq!(p.variables(), vec![Variable::Empresa]);
    }

    /// **Y nada que se parezca a lógica se ejecuta.**
    ///
    /// Es la razón de no usar una biblioteca de plantillas hecha (ADR-0006).
    /// Cada una de estas líneas es sintaxis válida en Handlebars o en Tera.
    #[test]
    fn nada_que_parezca_logica_se_ejecuta() {
        for intento in [
            "{{#if nombre}}hola{{/if}}",
            "{{#each contactos}}x{{/each}}",
            "{{nombre|mayusculas}}",
            "{{ nombre }}",
            "{% for x in y %}",
            "{{7*7}}",
            "{{> otra_plantilla}}",
        ] {
            let p = Plantilla::nueva(intento, SiFalta::Nada);
            assert_eq!(
                p.rellenar(&ana()).texto,
                intento,
                "esto no se puede interpretar: {intento}"
            );
            assert!(p.variables().is_empty(), "no hay variables en {intento}");
        }
    }

    /// **Una llave suelta no cuelga, no se come texto y no se inventa nada.**
    ///
    /// Se comprueba carácter por carácter, con la salida exacta de cada caso.
    /// La primera versión de esta prueba aceptaba «que contenga "nombre" **o**
    /// que sea "{{"», y con ese `o` casi no podía fallar: es el fallo del §2 en
    /// su forma más común, una comprobación que pasa dijera lo que dijera el
    /// código. Se deja escrito porque volvió a aparecer aquí.
    ///
    /// El tercer caso es el que importa: `{{{{nombre}}` son dos llaves sueltas
    /// **seguidas de una variable de verdad**, y las dos cosas tienen que salir
    /// bien. Si el recorrido no avanzara tras las llaves que no cierran, se
    /// quedaría dando vueltas sobre ellas para siempre.
    #[test]
    fn una_llave_sin_cerrar_no_rompe_nada() {
        for (entra, sale) in [
            ("{{", "{{"),
            ("{{nombre", "{{nombre"),
            ("{{{{nombre}}", "{{Ana"),
            ("}}nombre{{", "}}nombre{{"),
            ("{{}}", "{{}}"),
            ("a{{b}}c", "a{{b}}c"),
        ] {
            let p = Plantilla::nueva(entra, SiFalta::Nada);
            assert_eq!(p.rellenar(&ana()).texto, sale, "con la entrada «{entra}»");
        }
    }

    /// **Lo que falta se cuenta, no se traga.**
    ///
    /// Es lo que el preflight enseña antes de activar: «a 34 contactos les
    /// falta el nombre» es una cifra con la que se puede decidir (§94).
    #[test]
    fn lo_que_falta_se_devuelve_para_poder_contarlo() {
        let p = Plantilla::nueva("Hola {{nombre}} {{apellido}}", SiFalta::Nada);
        let r = p.rellenar(&ana());
        assert_eq!(r.faltaron, vec![Variable::Apellido]);
        assert_eq!(r.texto, "Hola Ana ");
    }

    /// El valor de reserva se usa donde falta, y sólo ahí.
    #[test]
    fn el_valor_de_reserva_tapa_el_hueco() {
        let p = Plantilla::nueva(
            "Hola {{nombre}} {{apellido}}",
            SiFalta::Esto("cliente".to_owned()),
        );
        assert_eq!(p.rellenar(&ana()).texto, "Hola Ana cliente");
    }

    /// **Un valor en blanco cuenta como ausente.**
    ///
    /// En un archivo importado, una celda vacía y una con un espacio son lo
    /// mismo para quien la escribió. Tratarlas distinto haría que la mitad de
    /// los contactos recibieran «Hola  ,» y la otra mitad el valor de reserva.
    #[test]
    fn un_valor_en_blanco_cuenta_como_que_falta() {
        let v = Valores::nuevos().con(Variable::Nombre, "   ");
        let p = Plantilla::nueva("Hola {{nombre}}", SiFalta::Esto("cliente".to_owned()));
        let r = p.rellenar(&v);
        assert_eq!(r.texto, "Hola cliente");
        assert_eq!(r.faltaron, vec![Variable::Nombre]);
    }

    /// Las variables se listan sin repetir, en el orden en que aparecen.
    #[test]
    fn las_variables_se_listan_una_vez_y_en_orden() {
        let p = Plantilla::nueva("{{empresa}} y {{nombre}} y {{empresa}}", SiFalta::Nada);
        assert_eq!(p.variables(), vec![Variable::Empresa, Variable::Nombre]);
    }

    /// **Las claves son un contrato con las plantillas ya guardadas.**
    ///
    /// Cambiar una deja de resolver esa variable en todo lo escrito hasta hoy,
    /// y **no falla**: el mensaje saldría con el `{{nombre}}` en crudo o con el
    /// valor de reserva. Por eso se congelan aquí.
    #[test]
    fn las_claves_de_las_variables_son_estables() {
        assert_eq!(Variable::Nombre.clave(), "nombre");
        assert_eq!(Variable::Apellido.clave(), "apellido");
        assert_eq!(Variable::Empresa.clave(), "empresa");
        assert_eq!(Variable::MiEmpresa.clave(), "mi_empresa");
        assert_eq!(Variable::Baja.clave(), "baja");
        assert_eq!(Variable::TODAS.len(), 5);
    }

    /// La clave de i18n se deriva de la clave, y no se escribe por separado.
    ///
    /// ── POR QUÉ EXISTE ESTA PRUEBA ────────────────────────────────────────
    ///
    /// Porque las dos estaban escritas a mano y **se habían separado**:
    /// `clave()` decía `mi_empresa` y `clave_i18n()` decía `miEmpresa`. El
    /// catálogo sólo tiene la primera, así que la segunda apuntaba a un texto
    /// inexistente y la interfaz habría enseñado la clave cruda.
    ///
    /// Nadie lo vio porque las pantallas no usan `clave_i18n()`: construyen la
    /// clave con `clave()`, que era la correcta. El método estaba mal **y no lo
    /// llamaba nadie** — el patrón del §9, esta vez en un valor y no en una
    /// función.
    ///
    /// Verificada rompiéndola el 01/10/2026: volviendo `miEmpresa`, falla
    /// nombrando la variable. (§2)
    #[test]
    fn la_clave_de_i18n_se_deriva_de_la_clave() {
        for v in Variable::TODAS {
            assert_eq!(
                v.clave_i18n(),
                format!("plantilla.variable.{}", v.clave()),
                "la clave de i18n de {v:?} no deriva de su clave"
            );
        }
    }

    /// Cada variable se escribe entera igual que se reconoce.
    ///
    /// Sin esto, el botón que pega `{{nombre}}` en el editor podría escribir
    /// algo que el motor no reconoce, y nadie lo vería hasta el primer envío.
    #[test]
    fn como_se_escribe_es_como_se_reconoce() {
        for v in Variable::TODAS {
            let escrita = v.to_string();
            let p = Plantilla::nueva(&escrita, SiFalta::Nada);
            assert_eq!(p.variables(), vec![*v], "no se reconoce «{escrita}»");
        }
    }

    /// Un texto sin variables sale idéntico, y no se pierde un carácter.
    #[test]
    fn un_texto_sin_variables_sale_igual() {
        let texto = "Estimado cliente:\n\nGracias por su compra.\n\n— El equipo";
        let p = Plantilla::nueva(texto, SiFalta::Nada);
        assert_eq!(p.rellenar(&ana()).texto, texto);
        assert!(p.variables().is_empty());
    }
}

#[cfg(test)]
mod firmas_sin_datos_ajenos {
    //! R-04 · una firma no puede llevar variables de quien recibe
    //!
    //! Dirección lo dijo el 16/10/2026: *«sería una tragedia que apareciera en
    //! la firma el nombre del contacto a quien se le está haciendo llegar el
    //! correo y su empresa, en lugar de los datos del remitente»*.
    //!
    //! Y es exacto: el formulario de plantillas y el de firmas son el mismo, y
    //! ofrecía las cuatro variables en los dos casos. Tres son del
    //! destinatario, y una firma con `{{nombre}}` las habría puesto **debajo
    //! del mensaje, en cada correo de la campaña**.
    use super::*;

    #[test]
    fn una_firma_limpia_pasa() {
        assert!(revisar_firma("Ana Ruiz · Distribuidora del Norte").is_ok());
    }

    #[test]
    fn la_variable_de_quien_envia_si_vale() {
        // `mi_empresa` es la única que habla de quien firma, y es justo la que
        // una firma necesita: así el día que cambie el nombre comercial, las
        // firmas lo recogen solas.
        assert!(revisar_firma("Ana Ruiz\n{{mi_empresa}}").is_ok());
    }

    #[test]
    fn las_tres_del_contacto_se_rechazan() {
        for ajena in ["{{nombre}}", "{{apellido}}", "{{empresa}}"] {
            let r = revisar_firma(&format!("Ana Ruiz\n{ajena}"));
            assert!(r.is_err(), "{ajena} tendría que rechazarse en una firma");
        }
    }

    /// **Nombra todas, no la primera.**
    ///
    /// Con una lista de una, corregirla y volver a guardar enseñaría la
    /// siguiente, y así una por intento: es el defecto que
    /// `reporta_todos_los_campos_malos_de_una_vez` ya corrigió en el
    /// formulario de empresa.
    #[test]
    fn dice_todas_las_que_encontro() {
        let r = revisar_firma("{{nombre}} de {{empresa}}, saludos de {{mi_empresa}}");
        let crate::CoreError::FirmaConVariablesDelContacto { cuales } =
            r.expect_err("tiene que fallar")
        else {
            panic!("el error tiene que ser el de la firma")
        };
        assert!(cuales.contains("nombre"), "{cuales}");
        assert!(cuales.contains("empresa"), "{cuales}");
        // Y no acusa a la que sí vale.
        assert!(!cuales.contains("mi_empresa"), "{cuales}");
    }

    /// **La lista de las que una firma puede usar se deriva, no se repite.**
    ///
    /// Dos listas de lo mismo se separan en cuanto alguien añada una variable,
    /// y la que se quedaría atrás es justo la que protege.
    /// Las que una firma puede usar son **las que no hablan del destinatario**.
    ///
    /// ── Y DESDE EL 22/10/2026 SON DOS, NO UNA ─────────────────────────────
    ///
    /// `Baja` entró con el pie de baja obligatorio (R-7), y entra aquí **a
    /// propósito**: el pie de baja en la firma es justo donde la gente lo
    /// espera, y `activar_campana` acepta que la salida venga de la plantilla o
    /// de la firma. Dejarla fuera habría obligado a repetirla en todas las
    /// plantillas aunque la firma ya la trajera.
    ///
    /// Su valor sale del remitente —como el de `MiEmpresa`—, que es lo que esta
    /// pregunta decide. Que la frase hable de quien recibe no la convierte en un
    /// dato suyo: R-04 existe para que una firma no ponga el nombre del
    /// destinatario donde debería ir el de quien firma.
    #[test]
    fn de_quien_firma_sale_de_es_de_quien_recibe() {
        let permitidas = Variable::de_quien_firma();
        assert_eq!(permitidas, vec![Variable::MiEmpresa, Variable::Baja]);
        for v in Variable::TODAS {
            assert_eq!(
                permitidas.contains(v),
                !v.es_de_quien_recibe(),
                "{v:?} no cuadra entre las dos"
            );
        }
    }
}

#[cfg(test)]
mod redactar_un_correo {
    //! R-07 paso 2b · el correo que se manda a una persona concreta.
    //!
    //! Hasta hoy el cuerpo de una campaña **no llegaba al correo**: el único
    //! `Contenido` que veía el adaptador era un texto fijo (H-15). Esto es la
    //! pieza pura de ese camino, y lo que se vigila aquí son las cosas que la
    //! sustitución puede hacer mal — todas silenciosas.
    //!
    //! **Verificadas rompiéndolas el 20/10/2026**, cinco rupturas y cada una
    //! cazada sólo por su prueba: no escapar los datos, escapar también el
    //! asunto, pegar la firma sin rellenarla, contar sólo lo que falta en el
    //! cuerpo, y dejar la raya de la firma cuando no hay firma.
    use super::*;

    fn ana() -> Valores {
        Valores::nuevos()
            .con(Variable::Nombre, "Ana")
            .con(Variable::Empresa, "Fábrica")
            .con(Variable::MiEmpresa, "TELEMETRY")
    }

    #[test]
    fn la_plantilla_sale_con_los_datos_de_la_persona() {
        let r = redactar(
            "Hola {{nombre}}",
            "<p>Buenas, {{nombre}} de {{empresa}}.</p>",
            None,
            &ana(),
            SiFalta::Nada,
        );
        assert_eq!(r.asunto, "Hola Ana");
        assert_eq!(r.html, "<p>Buenas, Ana de Fábrica.</p>");
    }

    /// **El dato de una persona no puede meter etiquetas en el correo.**
    ///
    /// Es el hallazgo que la auditoría B dejó escrito el 07/10/2026 antes de
    /// que este camino existiera: la plantilla se sanea **al guardarla**, y la
    /// sustitución ocurre después. Una empresa llamada `<script>…` —que pudo
    /// venir de un XLSX ajeno— entraría como etiqueta en un HTML que ya pasó
    /// su revisión, y el saneado no lo vio porque entonces no estaba.
    #[test]
    fn un_dato_con_etiquetas_no_se_convierte_en_etiquetas() {
        let valores = Valores::nuevos()
            .con(Variable::Nombre, "<script>alert(1)</script>")
            .con(Variable::Empresa, "Pérez & Hijos");

        let r = redactar(
            "Para {{empresa}}",
            "<p>Hola {{nombre}}, de {{empresa}}.</p>",
            None,
            &valores,
            SiFalta::Nada,
        );

        assert!(
            !r.html.contains("<script>"),
            "entró una etiqueta en el correo: {}",
            r.html
        );
        assert!(r.html.contains("&lt;script&gt;"), "{}", r.html);
        // Y el nombre se sigue leyendo entero: escapar no es recortar.
        assert!(r.html.contains("Pérez &amp; Hijos"), "{}", r.html);

        // **El asunto no se escapa**: no es HTML, viaja en una cabecera y se
        // enseña tal cual. Con el escapado, quien lo recibe leería
        // «Pérez &amp; Hijos» en su bandeja.
        assert_eq!(r.asunto, "Para Pérez & Hijos");
    }

    /// La firma va detrás, separada, y **con `{{mi_empresa}}` resuelto**.
    ///
    /// Una firma no puede llevar variables de quien recibe (R-04), pero sí la
    /// de quien firma — y si no se rellenara, saldría `{{mi_empresa}}` impreso
    /// al pie de cada correo de la campaña.
    #[test]
    fn la_firma_va_detras_y_con_sus_variables_puestas() {
        let r = redactar(
            "Hola",
            "<p>El mensaje.</p>",
            Some("<p>Ana Ruiz · {{mi_empresa}}</p>"),
            &ana(),
            SiFalta::Nada,
        );
        assert!(r.html.starts_with("<p>El mensaje.</p>"), "{}", r.html);
        assert!(r.html.contains("TELEMETRY"), "{}", r.html);
        assert!(
            !r.html.contains("{{mi_empresa}}"),
            "la firma salió sin rellenar: {}",
            r.html
        );
        assert!(
            r.html.contains("<p>—</p>"),
            "sin separación, la firma se lee como parte del mensaje: {}",
            r.html
        );
    }

    /// Sin firma no se pega nada, **ni la separación**.
    ///
    /// Una firma vacía que deja su raya detrás del mensaje es una firma que
    /// parece haberse perdido.
    #[test]
    fn sin_firma_no_queda_ninguna_raya() {
        for firma in [None, Some(""), Some("   ")] {
            let r = redactar("Hola", "<p>El mensaje.</p>", firma, &ana(), SiFalta::Nada);
            assert_eq!(r.html, "<p>El mensaje.</p>", "con firma {firma:?}");
        }
    }

    /// **Lo que faltó se cuenta del asunto Y del cuerpo.**
    ///
    /// Un asunto con `{{apellido}}` y un cuerpo sin él diría que no faltó
    /// nada: la cifra de «a cuántos les falta un dato» saldría más baja de lo
    /// que es, que es la que decide si una campaña se manda.
    #[test]
    fn lo_que_falta_en_el_asunto_tambien_cuenta() {
        let r = redactar(
            "Hola {{apellido}}",
            "<p>Buenas, {{nombre}}.</p>",
            None,
            &ana(),
            SiFalta::Nada,
        );
        assert!(
            r.faltaron.contains(&Variable::Apellido),
            "faltaron: {:?}",
            r.faltaron
        );
    }

    /// Y el texto de reserva se aplica en los dos sitios.
    #[test]
    fn el_texto_de_reserva_vale_para_el_asunto_y_para_el_cuerpo() {
        let r = redactar(
            "Hola {{apellido}}",
            "<p>Buenas, {{apellido}}.</p>",
            None,
            &ana(),
            SiFalta::Esto("cliente".to_owned()),
        );
        assert_eq!(r.asunto, "Hola cliente");
        assert_eq!(r.html, "<p>Buenas, cliente.</p>");
    }
}
