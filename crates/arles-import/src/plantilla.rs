//! La plantilla de contactos que ARLES entrega para rellenar (22/10/2026).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO EXISTE
//!
//! Dirección lo pidió con el caso delante: la tabla la trae el cliente, con sus
//! columnas, en su orden y con nueve campos de los que ARLES usa tres. La
//! plantilla es dónde pegar las que sirven.
//!
//! Y es el **único** archivo que ARLES escribe para volver a leerlo él mismo,
//! así que lleva su propia comprobación: `la_plantilla_que_arles_da_entra_en_arles`
//! en el núcleo, más la ida y vuelta de aquí, que la genera y la lee con
//! [`crate::leer`] — el mismo camino que recorre el archivo del cliente.
//!
//! POR QUÉ XLSX Y NO CSV
//!
//! Porque un CSV no dice con qué separador se escribió, y **Excel con la
//! configuración regional de México espera punto y coma**. Una plantilla con
//! comas se abre con las tres columnas dentro de la A, y quien la recibe no
//! tiene forma de saber que eso no es la plantilla. El XLSX no tiene esa
//! ambigüedad, ni la de la codificación: los acentos de «Teléfono» van en UTF-8
//! dentro del XML y no dependen de la máquina que lo abra.
//!
//! Y no cuesta una dependencia: `zip` ya estaba aquí para medir la
//! descompresión de los XLSX que se leen.
//!
//! POR QUÉ NO LLEVA NINGUNA FILA DE EJEMPLO
//!
//! Porque una fila de ejemplo que nadie borra **se importa**. Sería un contacto
//! inventado dentro de la primera campaña, con su correo falso recibiendo un
//! envío real — y el rastro quedaría en el registro como un fallo de entrega que
//! nadie sabría explicar. Tres columnas con nombres en español no necesitan
//! ejemplo; un contacto fantasma sí necesita explicación.
//! ─────────────────────────────────────────────────────────────────────────

use std::io::Write as _;
use std::path::Path;

use arles_core::importacion::PLANTILLA_DE_CONTACTOS;

use crate::ErrorDeLectura;

/// Cómo se llama la hoja dentro del archivo.
///
/// En español y sin espacios: es lo que Excel enseña en la pestaña de abajo, y
/// `calamine` lo usa para encontrarla.
pub const NOMBRE_DE_LA_HOJA: &str = "Contactos";

/// Nombre propuesto al guardar. La extensión importa: es lo que hace que el
/// sistema la abra con Excel y lo que [`crate::leer`] mira al reabrirla.
pub const NOMBRE_SUGERIDO: &str = "ARLES-plantilla-de-contactos.xlsx";

/// Escribe la plantilla vacía en `ruta`.
///
/// Las columnas salen de [`PLANTILLA_DE_CONTACTOS`], en el núcleo, y no de una
/// lista de aquí: una segunda lista es un segundo sitio donde añadir una columna
/// y olvidar la otra, y entonces la plantilla trae un encabezado que la
/// importación no reconoce.
///
/// # Errores
///
/// [`ErrorDeLectura::NoSePudoLeer`] si no se puede escribir donde se pidió —
/// carpeta de sólo lectura, disco lleno, o un archivo abierto en Excel, que en
/// Windows lo deja bloqueado.
pub fn escribir(ruta: &Path) -> Result<(), ErrorDeLectura> {
    let bytes = armar()?;
    let mut f = std::fs::File::create(ruta).map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
    f.write_all(&bytes)
        .map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
    // Sin esto, un error de escritura aparece al cerrar el archivo y se pierde:
    // `File` ignora el fallo de su propio `Drop`. Un disco lleno daría una
    // plantilla a medias y un «listo» en pantalla.
    f.flush().map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
    Ok(())
}

/// Los bytes del XLSX, sin tocar el disco.
///
/// Separado de [`escribir`] para poder probar el archivo sin un `tempfile`, y
/// para que el comando de la aplicación pueda contar cuánto pesa antes de
/// escribirlo.
///
/// # Errores
///
/// [`ErrorDeLectura::NoSePudoLeer`]. **Hoy no puede ocurrir**: se escribe en un
/// `Vec` en memoria, con cinco nombres de parte que son constantes, así que lo
/// único que fallaría es quedarse sin memoria. Devuelve un `Result` en vez de
/// un `expect` porque ARLES no tiene **ningún** panic en producción desde la
/// auditoría A del 07/10/2026 —el único que había, un `unreachable!()`
/// igual de imposible, se quitó—, y un `expect` que «no puede saltar» es la
/// forma en que vuelven.
pub fn armar() -> Result<Vec<u8>, ErrorDeLectura> {
    // Un XLSX es un ZIP con cinco partes. Son las mínimas que Excel acepta:
    // quitar cualquiera da «archivo dañado», no una hoja incompleta.
    let partes: [(&str, String); 5] = [
        ("[Content_Types].xml", tipos_de_contenido()),
        ("_rels/.rels", relaciones_del_paquete()),
        ("xl/workbook.xml", libro()),
        ("xl/_rels/workbook.xml.rels", relaciones_del_libro()),
        ("xl/worksheets/sheet1.xml", hoja()),
    ];

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    // `SimpleFileOptions` desde zip 9: el tipo ganó un segundo parámetro de
    // vida y el alias lo deja escrito en un sitio. Es el único cambio de API
    // que trajo la subida de la 2 a la 9.
    let opciones = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (nombre, contenido) in &partes {
        zip.start_file(*nombre, opciones)
            .map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
        zip.write_all(contenido.as_bytes())
            .map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
    }

    Ok(zip
        .finish()
        .map_err(|_| ErrorDeLectura::NoSePudoLeer)?
        .into_inner())
}

fn tipos_de_contenido() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
        r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
        r#"<Default Extension="xml" ContentType="application/xml"/>"#,
        r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#,
        r#"<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
        "</Types>"
    )
    .to_owned()
}

fn relaciones_del_paquete() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>"#,
        "</Relationships>"
    )
    .to_owned()
}

fn libro() -> String {
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main""#,
            r#" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<sheets><sheet name="{}" sheetId="1" r:id="rId1"/></sheets>"#,
            "</workbook>"
        ),
        NOMBRE_DE_LA_HOJA
    )
}

fn relaciones_del_libro() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>"#,
        "</Relationships>"
    )
    .to_owned()
}

/// La hoja: una sola fila, la de los encabezados.
///
/// Las celdas van como `inlineStr` y no por la tabla de cadenas compartidas.
/// Con tres textos, la tabla sería una parte más del ZIP y una indirección más
/// que puede quedar descolgada; en línea, lo que se lee es lo que está escrito.
fn hoja() -> String {
    let mut celdas = String::new();
    for (i, (_, encabezado)) in PLANTILLA_DE_CONTACTOS.iter().enumerate() {
        celdas.push_str(&format!(
            r#"<c r="{}1" t="inlineStr"><is><t>{}</t></is></c>"#,
            columna_de_excel(i),
            escapar_xml(encabezado)
        ));
    }
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#,
            r#"<sheetData><row r="1">{}</row></sheetData>"#,
            "</worksheet>"
        ),
        celdas
    )
}

/// La letra de la columna número `i`, contando desde cero: `A`, `B`, … `Z`, `AA`.
///
/// Pasa de la Z porque el tope de columnas del núcleo son 64, y una plantilla
/// futura con más de veintiséis columnas produciría referencias como `[1` sin
/// que Excel diga otra cosa que «archivo dañado».
fn columna_de_excel(i: usize) -> String {
    let mut n = i + 1;
    let mut letras = Vec::new();
    while n > 0 {
        let resto = (n - 1) % 26;
        letras.push(b'A' + u8::try_from(resto).unwrap_or(0));
        n = (n - 1) / 26;
    }
    letras.reverse();
    String::from_utf8(letras).unwrap_or_default()
}

/// Deja un texto listo para ir dentro de un XML.
///
/// Los encabezados de hoy son tres palabras en español, así que esto no hace
/// nada visible. Existe porque el día que una columna lleve un `&` —«Nombre &
/// apellido»— el archivo dejaría de ser XML válido y Excel diría «dañado», que
/// no apunta a ninguna parte. El `&` va primero, o escaparía los de los demás.
fn escapar_xml(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use arles_core::importacion::{CampoImportable, proponer_mapeo};

    /// **La prueba que esta entrega existía para tener.** La plantilla se
    /// genera, se escribe en disco y se lee con el **mismo** [`crate::leer`] que
    /// lee el archivo del cliente.
    ///
    /// Ninguna prueba del núcleo puede escribirla: allí se comprueba que los
    /// encabezados *de la constante* se reconocen, no que el archivo los lleve.
    /// Entre las dos piezas está el generador, que es donde cabe el error —
    /// escribir `Telefono` sin acento, perder una columna, o producir un XLSX
    /// que `calamine` no abra. Es el patrón del §9 en la única junta nueva.
    #[test]
    fn la_plantilla_se_escribe_y_se_vuelve_a_leer() {
        let dir = tempfile::tempdir().expect("carpeta temporal");
        let ruta = dir.path().join(NOMBRE_SUGERIDO);
        escribir(&ruta).expect("se escribe");

        let leida = crate::leer(&ruta).expect("ARLES lee su propia plantilla");

        assert_eq!(
            leida.encabezados,
            vec!["Nombre", "Correo", "Teléfono"],
            "los encabezados no sobrevivieron la ida y vuelta"
        );
        assert_eq!(leida.total_de_filas, 0, "la plantilla no lleva filas");
    }

    /// Y lo que se lee de vuelta se reconoce: ninguna columna cae en `Ignorar`.
    ///
    /// Es la mitad que importa. Una plantilla que se relee pero cuyas columnas
    /// no se adivinan es una plantilla que el cliente rellena y que entra sin
    /// sus datos, sin un solo error.
    #[test]
    fn las_columnas_de_la_plantilla_se_reconocen_al_releerla() {
        let dir = tempfile::tempdir().expect("carpeta temporal");
        let ruta = dir.path().join(NOMBRE_SUGERIDO);
        escribir(&ruta).expect("se escribe");

        let leida = crate::leer(&ruta).expect("se lee");
        let mapeo = proponer_mapeo(&leida.encabezados);

        assert_eq!(
            mapeo,
            vec![
                CampoImportable::Nombre,
                CampoImportable::Correo,
                CampoImportable::WhatsApp
            ],
            "la plantilla no se mapea a los tres campos que declara"
        );
    }

    /// El acento de «Teléfono» es el caso que distingue un XLSX bien escrito de
    /// uno que se abre con la codificación de la máquina. Va aparte porque es la
    /// razón por la que esto no es un CSV.
    #[test]
    fn el_acento_sobrevive() {
        let dir = tempfile::tempdir().expect("carpeta temporal");
        let ruta = dir.path().join("p.xlsx");
        escribir(&ruta).expect("se escribe");
        let leida = crate::leer(&ruta).expect("se lee");
        assert!(
            leida.encabezados.iter().any(|h| h == "Teléfono"),
            "el acento se perdió: {:?}",
            leida.encabezados
        );
    }

    /// Las letras de columna pasan de la Z. Con tres columnas no se nota; con
    /// veintisiete, una cuenta mal escrita produce `[1` y Excel dice «dañado».
    #[test]
    fn las_columnas_siguen_contando_despues_de_la_z() {
        assert_eq!(columna_de_excel(0), "A");
        assert_eq!(columna_de_excel(25), "Z");
        assert_eq!(columna_de_excel(26), "AA");
        assert_eq!(columna_de_excel(27), "AB");
        assert_eq!(columna_de_excel(51), "AZ");
        assert_eq!(columna_de_excel(52), "BA");
    }

    /// El escapado existe para un encabezado futuro con `&`. Se prueba con uno,
    /// porque una defensa que nunca se ha ejercitado no está verificada.
    #[test]
    fn un_ampersand_no_rompe_el_xml() {
        assert_eq!(escapar_xml("Nombre & apellido"), "Nombre &amp; apellido");
        assert_eq!(escapar_xml("<script>"), "&lt;script&gt;");
        // El `&` va primero: al revés, el `&lt;` del `<` se convertiría en
        // `&amp;lt;` y el archivo enseñaría «&lt;» como texto.
        assert_eq!(escapar_xml("a<b&c"), "a&lt;b&amp;c");
    }

    /// Un XLSX es un ZIP con cinco partes, y cada una tiene que estar. Quitar
    /// cualquiera no da una hoja incompleta: da «archivo dañado».
    #[test]
    fn el_archivo_lleva_las_cinco_partes() {
        let bytes = armar().expect("se arma en memoria");
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("es un ZIP legible");
        // Desde zip 9, `name()` devuelve `Result<Cow<str>, ZipError>`: un
        // nombre dentro de un ZIP puede no ser texto válido. Aquí se descartan
        // los que no lo sean, que es lo mismo que hacía el `.ok()` de la línea
        // de al lado — lo que se comprueba es que las cinco partes ESTÉN, y
        // una parte con el nombre ilegible no está.
        let dentro: Vec<String> = (0..zip.len())
            .filter_map(|i| {
                let f = zip.by_index(i).ok()?;
                Some(f.name().ok()?.into_owned())
            })
            .collect();
        for parte in [
            "[Content_Types].xml",
            "_rels/.rels",
            "xl/workbook.xml",
            "xl/_rels/workbook.xml.rels",
            "xl/worksheets/sheet1.xml",
        ] {
            assert!(
                dentro.iter().any(|n| n == parte),
                "falta {parte}, dentro hay {dentro:?}"
            );
        }
    }
}
