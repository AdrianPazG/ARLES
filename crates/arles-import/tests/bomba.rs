//! La bomba de descompresión, sola en su proceso.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ ESTE ATAQUE NO VIVE CON LOS DEMÁS
//!
//! Lo que hay que demostrar aquí no es que la lectura **devuelva un error** —eso
//! lo puede devolver después de haberse comido dos gigabytes—, sino que lo
//! devuelve **a tiempo**. Y eso sólo se ve midiendo la memoria del proceso.
//!
//! El pico de memoria (`VmHWM`) es del proceso entero y no baja nunca. Un
//! binario de pruebas corre todos sus tests en el mismo proceso y en paralelo,
//! así que cualquier otro test que reserve medio giga deja el pico arriba y la
//! medición pasa a ser un adorno que siempre dice que sí.
//!
//! Por eso este ataque tiene su propio binario de pruebas: es el único test del
//! proceso, y el pico que se mide es suyo.
//!
//! Por la misma razón el archivo se escribe **en trozos directamente al disco**.
//! Armar el XML de la bomba en una cadena de Rust subiría el pico del proceso a
//! lo que mide la bomba antes de leerla, y la comprobación volvería a ser vacua
//! —midiendo el generador en vez de la defensa—.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::io::Write as _;

use arles_import::{ErrorDeLectura, MAX_CELDAS, leer};

/// Cuántas filas de ocho celdas trae la bomba.
///
/// Dos millones por ocho son dieciséis millones de celdas: el doble del tope.
/// Sin comprimir, el XML pasa de **mil megabytes**; comprimido son menos de
/// dos, porque es la misma cadena repetida. Ése es exactamente el truco de una
/// bomba de descompresión.
const FILAS: usize = 2_000_000;

/// Pico de memoria del proceso, en kilobytes.
///
/// `VmHWM` es la marca de agua: la mayor cantidad de memoria física que el
/// proceso ha tenido en uso desde que arrancó. Se lee antes y después porque lo
/// que interesa es el salto, no el valor.
///
/// Sólo existe en Linux, que es donde corre la integración continua. En otros
/// sistemas devuelve `None` y la prueba lo dice en vez de callárselo: una
/// comprobación que se salta no es una comprobación que pasa.
fn pico_de_memoria_kb() -> Option<u64> {
    let estado = std::fs::read_to_string("/proc/self/status").ok()?;
    estado
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|r| r.trim().trim_end_matches("kB").trim().parse().ok())
}

/// Escribe la bomba al disco sin tenerla nunca entera en memoria.
fn escribir_la_bomba(ruta: &std::path::Path) -> u64 {
    let archivo = std::fs::File::create(ruta).expect("crea");
    let mut zip = zip::ZipWriter::new(archivo);
    let opciones: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let armazon: [(&str, &str); 4] = [
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
    for (nombre, cuerpo) in armazon {
        zip.start_file(nombre, opciones).expect("entrada");
        zip.write_all(cuerpo.as_bytes()).expect("escribe");
    }

    zip.start_file("xl/worksheets/sheet1.xml", opciones)
        .expect("entrada");
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#,
    )
    .expect("escribe");

    // De mil en mil filas: el trozo que vive en memoria son unos cien
    // kilobytes, no mil megabytes.
    let mut trozo = String::with_capacity(1000 * 500);
    for bloque in 0..(FILAS / 1000) {
        trozo.clear();
        for i in 0..1000 {
            let fila = bloque * 1000 + i + 1;
            trozo.push_str(&format!("<row r=\"{fila}\">"));
            for c in 0..8u8 {
                let columna = char::from(b'A' + c);
                trozo.push_str(&format!(
                    "<c r=\"{columna}{fila}\" t=\"inlineStr\"><is><t>relleno</t></is></c>"
                ));
            }
            trozo.push_str("</row>");
        }
        zip.write_all(trozo.as_bytes()).expect("escribe");
    }
    zip.write_all(b"</sheetData></worksheet>").expect("escribe");
    zip.finish().expect("cierra");

    std::fs::metadata(ruta).expect("mide").len()
}

/// **El ataque clásico contra un XLSX: es un ZIP.**
///
/// Un archivo de menos de dos megabytes en disco que declara dieciséis millones
/// de celdas. Se exigen dos cosas, y la segunda es la que importa:
///
///   1. Que la lectura se corte con [`ErrorDeLectura::DemasiadosDatos`].
///   2. Que se corte **antes de haberse comido la hoja descomprimida**. Un tope
///      que se comprueba después de materializar el archivo no es una defensa:
///      la memoria ya se gastó, y el error llega cuando el daño está hecho.
///
/// Se verificó rompiendo la guarda a propósito —leyendo la hoja entera antes de
/// contar—: la prueba falla, y falla por el pico de memoria, no por el error.
#[test]
fn una_bomba_de_descompresion_se_corta_antes_de_agotar_la_memoria() {
    let celdas = FILAS * 8;
    assert!(
        celdas > MAX_CELDAS,
        "la bomba trae {celdas} celdas, por debajo del tope de {MAX_CELDAS}: no prueba nada"
    );

    let dir = tempfile::tempdir().expect("temporal");
    let ruta = dir.path().join("bomba.xlsx");
    let en_disco = escribir_la_bomba(&ruta);

    let antes = pico_de_memoria_kb();
    let resultado = leer(&ruta);
    let despues = pico_de_memoria_kb();

    assert!(
        matches!(resultado, Err(ErrorDeLectura::DemasiadosDatos)),
        "una bomba de {en_disco} bytes que declara {celdas} celdas no se cortó: {resultado:?}"
    );

    match (antes, despues) {
        (Some(antes), Some(despues)) => {
            let salto_mb = despues.saturating_sub(antes) / 1024;
            println!(
                "· bomba: {en_disco} bytes en disco, {celdas} celdas declaradas, \
                 pico +{salto_mb} MB al leerla"
            );
            assert!(
                salto_mb < 400,
                "la lectura se cortó, pero sólo después de comerse {salto_mb} MB: \
                 el tope se comprueba demasiado tarde para servir de defensa"
            );
        }
        _ => println!("· sin /proc/self/status: el pico de memoria no se pudo medir"),
    }
}
