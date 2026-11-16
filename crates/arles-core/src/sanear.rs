//! Dejar el HTML de un mensaje en algo que se puede mandar sin miedo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LISTA BLANCA, NO LISTA NEGRA
//!
//! No se quitan las etiquetas peligrosas: **se conservan sólo las permitidas**,
//! y todo lo demás se va. La diferencia es la que hay entre una puerta con
//! cerradura y una puerta con un cartel de «no pasar».
//!
//! Una lista negra hay que mantenerla al día con cada truco nuevo —`<svg>` con
//! un `onload`, `<math>` con un `href`, una entidad HTML que se decodifica
//! tarde—, y lo que no esté en la lista pasa. Con lista blanca, lo que no esté
//! previsto **no pasa**, y el coste de equivocarse es que alguien no pueda usar
//! una etiqueta que quería, no que un correo lleve un script.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Y ESTO CORRE EN RUST, NUNCA EN LA INTERFAZ
//!
//! Sanear en el navegador sería sanear **después** de que el texto ya está
//! donde alguien puede cambiarlo. Todo lo que la webview manda es un dato de
//! entrada, incluido el resultado de su propio saneado.
//!
//! Se sanea **al guardar y otra vez al enviar**, y no es redundante: entre las
//! dos cosas hay un archivo de base de datos en el disco del usuario, y confiar
//! en lo guardado es confiar en que nadie lo tocó. La segunda pasada cuesta
//! microsegundos por correo.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! QUÉ SE PERMITE, Y POR QUÉ TAN POCO
//!
//! Lo que cabe en un correo escrito por una persona: párrafos, saltos, negrita,
//! cursiva, subrayado, listas y enlaces. Ni imágenes, ni tablas, ni estilos.
//!
//! No es minimalismo por gusto. Cada etiqueta de más es superficie que hay que
//! entender: una `<img>` con un `src` remoto es un rastreador que delata cuándo
//! se abrió el correo —lo que ARLES promete no hacer (§65: no se sabe si
//! llegó)—, y un `<style>` es un vector de fuga de datos por CSS que ningún
//! cliente de correo filtra igual.
//!
//! Si algún día hace falta una imagen, se añade aquí con su decisión escrita al
//! lado, no se cuela por una lista negra que se olvidó de nombrarla.

use std::collections::HashSet;
use std::sync::OnceLock;

/// Las etiquetas que sobreviven. Todo lo demás se va.
///
/// **El texto de dentro se conserva** aunque la etiqueta desaparezca: quitar
/// un `<div>` no puede borrar el párrafo que envolvía. Lo que se borra entero,
/// con su contenido, es lo que nunca es texto para leer — un `<script>`.
pub const ETIQUETAS: &[&str] = &[
    "p", "br", "strong", "b", "em", "i", "u", "ul", "ol", "li", "a",
];

/// Los atributos que sobreviven, y sólo en `<a>`.
///
/// `href` porque un enlace sin destino no es un enlace. `title` porque es texto
/// que el usuario escribe y que algunos lectores de pantalla anuncian.
///
/// **Nada de `target`, `rel`, `style`, `class` ni `id`.** Un `style` en línea
/// es la puerta trasera del CSS, y los eventos —`onclick` y compañía— no están
/// aquí ni pueden estarlo: no son atributos permitidos, así que se van con todo
/// lo que no está en esta lista.
pub const ATRIBUTOS_DE_ENLACE: &[&str] = &["href", "title"];

/// Los esquemas de enlace admitidos.
///
/// `javascript:` **no está**, y ése es el motivo de que exista esta lista. Un
/// `<a href="javascript:...">` tiene la etiqueta permitida y el atributo
/// permitido: lo único que lo para es que su esquema no esté aquí.
///
/// `mailto` y `tel` están porque en un correo comercial son normales y no
/// ejecutan nada.
pub const ESQUEMAS: &[&str] = &["http", "https", "mailto", "tel"];

/// El saneador, construido una vez.
///
/// `ammonia` compila su configuración al construirse, y hacerlo por cada correo
/// de una campaña de diez mil sería trabajo repetido diez mil veces para llegar
/// siempre al mismo sitio.
fn saneador() -> &'static ammonia::Builder<'static> {
    static SANEADOR: OnceLock<ammonia::Builder<'static>> = OnceLock::new();
    SANEADOR.get_or_init(|| {
        let mut b = ammonia::Builder::default();
        b.tags(ETIQUETAS.iter().copied().collect::<HashSet<_>>());
        b.url_schemes(ESQUEMAS.iter().copied().collect::<HashSet<_>>());
        b.generic_attributes(HashSet::new());
        b.tag_attributes(std::collections::HashMap::from([(
            "a",
            ATRIBUTOS_DE_ENLACE.iter().copied().collect::<HashSet<_>>(),
        )]));
        // Sin `rel="noopener"` añadido: ningún enlace lleva `target`, así que no
        // hay ventana nueva que proteger, y un atributo que ARLES inventa es un
        // atributo que alguien va a ver en el correo y no va a entender.
        b.link_rel(None);
        b
    })
}

/// Deja el HTML en lo que se puede mandar.
///
/// Se llama **al guardar y otra vez al enviar**. Ver la nota de la cabecera.
#[must_use]
pub fn html(sucio: &str) -> String {
    saneador().clean(sucio).to_string()
}

/// Convierte un dato de una persona en algo que se puede meter dentro de un
/// HTML sin cambiar su significado.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTO NO ES LO MISMO QUE [`html`], Y POR QUÉ HACEN FALTA LAS DOS
///
/// [`html`] limpia **un texto que es HTML**: se queda con las etiquetas de la
/// lista blanca y tira el resto. Esto trata un texto que **no es HTML** —el
/// nombre de alguien, su empresa— y que va a quedar dentro de uno.
///
/// La diferencia importa porque la plantilla se sanea **al guardarla**, y la
/// sustitución de variables ocurre **después**, al enviar: mete un dato que
/// vino de un XLSX ajeno dentro de un HTML que ya pasó su revisión. Sin
/// escapar, una empresa llamada `<script>…` entra en el correo como etiqueta,
/// y el saneado de la plantilla no lo vio porque entonces no estaba.
///
/// Es el hallazgo que la auditoría B del 07/10/2026 dejó escrito antes de que
/// existiera este camino: *«entre guardar y enviar está la sustitución de
/// variables, que mete datos de un XLSX ajeno dentro del HTML ya saneado»*.
///
/// **Y no vale pasar el dato por [`html`] en su lugar.** Ése lo dejaría
/// intacto si no tiene etiquetas y le quitaría los `<` sueltos: una empresa
/// llamada `Pérez < Hijos` perdería parte de su nombre, en silencio. Lo que
/// hace falta aquí es que el dato **se vea tal como se escribió**.
///
/// Se escapan cinco: `&` primero —si no, reescribiría los `&` que las otras
/// cuatro acaban de meter—, los dos ángulos, y las dos clases de comilla,
/// porque un dato puede acabar dentro de un atributo.
#[must_use]
pub fn escapar_para_html(dato: &str) -> String {
    let mut salida = String::with_capacity(dato.len());
    for c in dato.chars() {
        match c {
            '&' => salida.push_str("&amp;"),
            '<' => salida.push_str("&lt;"),
            '>' => salida.push_str("&gt;"),
            '"' => salida.push_str("&quot;"),
            '\'' => salida.push_str("&#39;"),
            otro => salida.push(otro),
        }
    }
    salida
}

/// La versión en texto plano de un HTML ya saneado.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ARLES ESCRIBE LAS DOS VERSIONES
///
/// Un correo sólo-HTML entra peor: los filtros de spam lo puntúan peor, y hay
/// clientes —y relojes, y lectores de pantalla— que enseñan la alternativa de
/// texto. `template_version` tiene las dos columnas desde la V1 por eso.
///
/// **No es un conversor de HTML a Markdown.** Quita las etiquetas y respeta los
/// saltos que la estructura implica: un párrafo son dos saltos, un `<br>` es
/// uno, y cada elemento de lista empieza en su línea con un guion. Lo demás
/// —negritas, enlaces— desaparece dejando su texto, porque en texto plano no
/// hay forma honesta de enseñarlo.
///
/// El destino del enlace se conserva entre paréntesis: un texto plano que dice
/// «haz clic aquí» sin decir dónde es inútil, y en un correo comercial es
/// además sospechoso.
#[must_use]
pub fn a_texto_plano(html: &str) -> String {
    let limpio = self::html(html);
    let mut texto = String::with_capacity(limpio.len());
    let mut dentro_de_etiqueta = false;
    let mut etiqueta = String::new();

    for c in limpio.chars() {
        match c {
            '<' => {
                dentro_de_etiqueta = true;
                etiqueta.clear();
            }
            '>' if dentro_de_etiqueta => {
                dentro_de_etiqueta = false;
                texto.push_str(&separador_de(&etiqueta));
                if let Some(destino) = destino_de(&etiqueta) {
                    texto.push_str(&destino);
                }
            }
            _ if dentro_de_etiqueta => etiqueta.push(c),
            _ => texto.push(c),
        }
    }

    // Las entidades que `ammonia` deja escritas vuelven a ser caracteres: en
    // texto plano, un `&amp;` se lee como «&amp;» y no como «&».
    let texto = texto
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Nunca más de dos saltos seguidos: un HTML con párrafos vacíos dejaría un
    // texto lleno de huecos.
    let mut apretado = String::with_capacity(texto.len());
    let mut saltos = 0;
    for c in texto.chars() {
        if c == '\n' {
            saltos += 1;
            if saltos > 2 {
                continue;
            }
        } else {
            saltos = 0;
        }
        apretado.push(c);
    }
    apretado.trim().to_owned()
}

/// Qué salto de línea implica una etiqueta.
fn separador_de(etiqueta: &str) -> String {
    let nombre = etiqueta
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let cierra = etiqueta.starts_with('/');

    match nombre.as_str() {
        "br" => "\n".to_owned(),
        "p" | "ul" | "ol" if cierra => "\n\n".to_owned(),
        "li" if !cierra => "\n- ".to_owned(),
        _ => String::new(),
    }
}

/// El destino de un enlace, entre paréntesis, si la etiqueta lo lleva.
fn destino_de(etiqueta: &str) -> Option<String> {
    if etiqueta.starts_with('/') || !etiqueta.to_ascii_lowercase().starts_with('a') {
        return None;
    }
    let (_, resto) = etiqueta.split_once("href=\"")?;
    let (destino, _) = resto.split_once('"')?;
    if destino.is_empty() {
        return None;
    }
    Some(format!(" ({destino})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Un script no sobrevive, ni con su texto dentro.**
    ///
    /// Es la razón de que este módulo exista. Un `<script>` no es texto para
    /// leer, así que se va entero — a diferencia de un `<div>`, que se va
    /// dejando su contenido.
    #[test]
    fn un_script_desaparece_entero() {
        let limpio = html("<p>Hola</p><script>alert('x')</script>");
        assert!(!limpio.contains("script"), "{limpio}");
        assert!(
            !limpio.contains("alert"),
            "y tampoco su contenido: {limpio}"
        );
        assert!(limpio.contains("Hola"), "el texto de al lado se queda");
    }

    /// **Un enlace que ejecuta código no pasa, aunque la etiqueta esté
    /// permitida.**
    ///
    /// `<a>` está en la lista blanca y `href` también. Lo único que para esto
    /// es que el esquema `javascript` no esté entre los admitidos — por eso esa
    /// lista existe, y por eso no se puede quitar «porque los enlaces ya están
    /// permitidos».
    ///
    /// Verificada rompiéndola el 29/09/2026: añadiendo `javascript` a
    /// `ESQUEMAS`, esta prueba falla.
    #[test]
    fn un_enlace_que_ejecuta_codigo_no_pasa() {
        for sucio in [
            r#"<a href="javascript:alert(1)">clic</a>"#,
            r#"<a href="JaVaScRiPt:alert(1)">clic</a>"#,
            r#"<a href="data:text/html,<script>alert(1)</script>">clic</a>"#,
        ] {
            let limpio = html(sucio);
            assert!(
                !limpio.to_ascii_lowercase().contains("javascript")
                    && !limpio.to_ascii_lowercase().contains("data:"),
                "pasó un enlace ejecutable: {limpio}"
            );
            assert!(limpio.contains("clic"), "el texto del enlace se conserva");
        }
    }

    /// **Los manejadores de eventos se van.**
    ///
    /// No hay una lista de eventos prohibidos: se van porque **no están entre
    /// los atributos permitidos**, que es justo la diferencia entre lista
    /// blanca y lista negra. Un `onpointerrawupdate` que nadie ha oído nombrar
    /// también se va.
    #[test]
    fn los_manejadores_de_eventos_se_van() {
        for sucio in [
            r#"<p onclick="alert(1)">Hola</p>"#,
            r#"<a href="https://x.mx" onmouseover="alert(1)">clic</a>"#,
            r#"<p onpointerrawupdate="alert(1)">Hola</p>"#,
        ] {
            let limpio = html(sucio);
            assert!(!limpio.contains("alert"), "{limpio}");
            assert!(!limpio.contains("on"), "quedó un manejador: {limpio}");
        }
    }

    /// **Ni imágenes ni estilos**, y las dos por un motivo concreto.
    ///
    /// Una `<img>` remota es un rastreador que delata cuándo se abrió el correo
    /// —lo que ARLES promete no saber (§65)— y un `<style>` es un vector de
    /// fuga por CSS que ningún cliente de correo filtra igual.
    #[test]
    fn ni_imagenes_ni_estilos() {
        let limpio =
            html(r#"<p>Hola</p><img src="https://rastreador.mx/1.gif"><style>p{x:y}</style>"#);
        assert!(!limpio.contains("img"), "{limpio}");
        assert!(!limpio.contains("rastreador"), "{limpio}");
        assert!(!limpio.contains("style"), "{limpio}");
        assert!(limpio.contains("Hola"));
    }

    /// Lo que sí se permite se conserva entero.
    ///
    /// Sin esta prueba, las de arriba pasarían con un saneador que devolviera
    /// la cadena vacía.
    #[test]
    fn lo_permitido_sobrevive() {
        let bueno = r#"<p>Hola <strong>Ana</strong>, mira <a href="https://telemetry.mx">esto</a>.</p><ul><li>Uno</li><li>Dos</li></ul>"#;
        let limpio = html(bueno);
        for trozo in [
            "<p>",
            "<strong>",
            "Ana",
            "<a href=",
            "telemetry.mx",
            "<ul>",
            "<li>",
            "Dos",
        ] {
            assert!(limpio.contains(trozo), "se perdió «{trozo}»: {limpio}");
        }
    }

    /// **Sanear dos veces da lo mismo que sanear una.**
    ///
    /// Es lo que permite hacerlo al guardar **y** al enviar sin que el mensaje
    /// se degrade en cada pasada. Sin esta garantía, el segundo saneado sería
    /// un riesgo en vez de una defensa.
    #[test]
    fn sanear_dos_veces_no_cambia_nada() {
        for entrada in [
            r#"<p>Hola <strong>Ana</strong></p>"#,
            r#"<p onclick="x">Hola</p><script>y</script>"#,
            r#"<a href="https://x.mx" target="_blank">clic</a>"#,
            "texto pelado sin etiquetas",
            "",
        ] {
            let una = html(entrada);
            assert_eq!(html(&una), una, "no es estable con «{entrada}»");
        }
    }

    /// El texto plano dice lo mismo, con los saltos que la estructura implica.
    #[test]
    fn el_texto_plano_conserva_la_estructura() {
        let plano = a_texto_plano("<p>Hola Ana</p><p>Gracias por todo</p>");
        assert_eq!(plano, "Hola Ana\n\nGracias por todo");
    }

    /// **Y conserva a dónde lleva cada enlace.**
    ///
    /// Un texto plano que dice «haz clic aquí» sin decir dónde es inútil, y en
    /// un correo comercial es además sospechoso.
    #[test]
    fn el_texto_plano_dice_a_donde_lleva_un_enlace() {
        let plano = a_texto_plano(r#"<p>Mira <a href="https://telemetry.mx">esto</a></p>"#);
        assert!(plano.contains("esto"), "{plano}");
        assert!(plano.contains("https://telemetry.mx"), "{plano}");
    }

    /// Las listas se leen como listas.
    #[test]
    fn el_texto_plano_marca_los_elementos_de_una_lista() {
        let plano = a_texto_plano("<ul><li>Uno</li><li>Dos</li></ul>");
        assert!(plano.contains("- Uno"), "{plano}");
        assert!(plano.contains("- Dos"), "{plano}");
    }

    /// **Y del texto plano no sale ni una etiqueta.**
    ///
    /// Es la garantía que importa: si una `<` sobreviviera, el cliente de
    /// correo del destinatario podría interpretarla.
    #[test]
    fn del_texto_plano_no_sale_ninguna_etiqueta() {
        let plano = a_texto_plano(
            r#"<p>Hola</p><script>alert(1)</script><p onclick="x">Adiós</p><img src="y">"#,
        );
        assert!(!plano.contains('<'), "quedó una etiqueta: {plano}");
        assert!(!plano.contains('>'), "quedó una etiqueta: {plano}");
        assert!(!plano.contains("alert"), "{plano}");
        assert!(plano.contains("Hola") && plano.contains("Adiós"));
    }

    /// Las entidades vuelven a ser caracteres: en texto plano, `&amp;` se lee
    /// como «&amp;» y no como «&».
    #[test]
    fn el_texto_plano_deshace_las_entidades() {
        assert_eq!(a_texto_plano("<p>Uno &amp; dos</p>"), "Uno & dos");
    }

    /// Nada de lo que entra vacío o roto revienta.
    #[test]
    fn lo_vacio_y_lo_roto_no_revientan() {
        for entrada in ["", "   ", "<", "<p", "<<<>>>", "</p>", "<p><p><p>"] {
            let _ = html(entrada);
            let _ = a_texto_plano(entrada);
        }
    }
}
