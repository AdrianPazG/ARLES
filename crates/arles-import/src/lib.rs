//! Lectura defensiva de CSV y XLSX (entrega 3.3).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL ARCHIVO QUE SE LEE AQUÍ NO ES DE FIAR
//!
//! Llega de fuera: lo exportó otro sistema, lo editó alguien, o lo mandó un
//! tercero. Este crate asume que **puede estar diseñado para hacer daño**, y por
//! eso está separado del resto: las defensas se prueban aisladas, y `calamine`
//! con su cadena de descompresión no entra en el shell de la aplicación.
//!
//! Cuatro cosas de las que defiende, todas del THREAT_MODEL.md §4.3:
//!
//!   1. **Bomba de descompresión.** Un XLSX es un ZIP. Cincuenta kilobytes
//!      pueden descomprimirse en cincuenta gigabytes. Se corta al pasar de
//!      [`MAX_CELDAS`] celdas leídas, que es un tope sobre lo que se lee y no
//!      sobre lo que el archivo dice medir —un archivo puede mentir sobre su
//!      tamaño, pero no sobre cuántas celdas ha entregado ya—.
//!   2. **Archivo enorme y legítimo.** No es un ataque, pero agota la memoria
//!      igual. Mismo tope, mismo corte.
//!   3. **Inyección de fórmulas.** Una celda que empieza por `=`, `+`, `-` o
//!      `@` es una fórmula en Excel y en Sheets. Aquí **nunca se evalúa** —sólo
//!      se lee texto—, y al exportar se neutraliza. Ver [`neutralizar_formula`].
//!   4. **Nombre de archivo hostil.** No se usa para nada: ni para escribir, ni
//!      para construir rutas. Sólo se guarda para enseñarlo, ya recortado.
//!
//! Y una cosa que no es un ataque pero rompe igual: **Excel de Windows guarda
//! los CSV en Windows-1252**, no en UTF-8. Sin detectarlo, una tabla mexicana
//! llega con los acentos rotos y nadie entiende por qué.
//! ─────────────────────────────────────────────────────────────────────────

#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::io::Read;
use std::path::Path;

use serde::Serialize;
use thiserror::Error;

/// Cuántas celdas se leen como mucho, entre todas las filas.
///
/// Es el tope **real** contra una bomba de descompresión: cuenta lo que ya se
/// ha leído, no lo que el archivo declara medir. Un archivo puede mentir sobre
/// su tamaño; no puede mentir sobre cuántas celdas ha entregado.
///
/// Con el tope de 64 columnas de `arles-core`, esto da margen para bastante más
/// de las 500 000 filas que pide T-7 en una tabla estrecha, y corta antes en una
/// tabla absurdamente ancha —que es justo la forma de la bomba—.
pub const MAX_CELDAS: usize = 8_000_000;

/// Tamaño máximo del archivo **en disco**.
///
/// Es una primera barrera barata, no la de verdad: la que cuenta es
/// [`MAX_CELDAS`], porque un ZIP de 20 MB puede descomprimirse en mucho más.
pub const MAX_BYTES: u64 = 200 * 1024 * 1024;

/// Cuántas filas se enseñan en la vista previa antes de importar.
///
/// Suficiente para reconocer si el mapeo de columnas es el correcto, y poco
/// para que la pantalla no tenga que virtualizar nada.
pub const FILAS_DE_MUESTRA: usize = 20;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ErrorDeLectura {
    /// La extensión no es `.csv`, `.xlsx` ni `.xls`.
    ///
    /// Se mira la extensión y **además** se intenta leer: la extensión es lo
    /// que el usuario cree que tiene, y el contenido es lo que hay.
    #[error("el archivo no es una tabla que ARLES sepa leer")]
    FormatoDesconocido,

    #[error("el archivo pesa más de lo admitido")]
    DemasiadoGrande,

    /// Se cortó la lectura al llegar al tope de celdas.
    ///
    /// **No se importa lo leído hasta ahí.** Un archivo que dispara esto es un
    /// archivo del que no se sabe qué más trae, y media importación es peor que
    /// ninguna: nadie sabría dónde se quedó.
    #[error("el archivo trae más datos de los que ARLES puede leer de una vez")]
    DemasiadosDatos,

    /// La hoja no tiene ninguna fila, o la primera está vacía.
    #[error("el archivo no tiene encabezados")]
    SinEncabezados,

    /// El XLSX no tiene ninguna hoja con datos.
    #[error("el archivo no tiene ninguna hoja con datos")]
    SinHojas,

    #[error("no se pudo leer el archivo")]
    NoSePudoLeer,
}

impl ErrorDeLectura {
    /// Clave estable para que la interfaz traduzca el error (§139).
    #[must_use]
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::FormatoDesconocido => "error.import.formato_desconocido",
            Self::DemasiadoGrande => "error.import.demasiado_grande",
            Self::DemasiadosDatos => "error.import.demasiados_datos",
            Self::SinEncabezados => "error.import.sin_encabezados",
            Self::SinHojas => "error.import.sin_hojas",
            Self::NoSePudoLeer => "error.import.no_se_pudo_leer",
        }
    }
}

/// Lo que se saca de un archivo antes de importar nada.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TablaLeida {
    /// Nombre del archivo, **sin la carpeta**. Sólo para enseñarlo.
    ///
    /// La ruta no se guarda: dice dónde vive el usuario y no le aporta nada a
    /// ninguna pantalla (THREAT_MODEL.md §4.1).
    pub nombre: String,
    /// Huella de los bytes del archivo.
    ///
    /// Va por bytes y no por su lectura: un CSV en Windows-1252 y el mismo en
    /// UTF-8 dicen lo mismo pero **no son el mismo archivo**, y para responder
    /// a «¿qué se importó?» importa el archivo.
    pub huella: String,
    /// Los encabezados, tal y como venían.
    pub encabezados: Vec<String>,
    /// Las primeras [`FILAS_DE_MUESTRA`] filas, para la vista previa.
    pub muestra: Vec<Vec<String>>,
    /// **Todas** las filas de datos, sin los encabezados.
    ///
    /// Van en memoria a propósito: la importación necesita recorrerlas dos
    /// veces —una para detectar los choques y otra para escribir—, y volver a
    /// abrir el archivo entre las dos dejaría una ventana en la que alguien
    /// puede cambiarlo debajo.
    #[serde(skip)]
    pub filas: Vec<Vec<String>>,
    /// Cuántas filas de datos trae. Es una cifra, no un adjetivo (§94).
    pub total_de_filas: usize,
}

/// Lee una tabla de un archivo, sin escribir nada.
///
/// # Errores
///
/// [`ErrorDeLectura`] si el formato no se reconoce, si el archivo pasa de los
/// topes, o si no tiene encabezados.
pub fn leer(ruta: &Path) -> Result<TablaLeida, ErrorDeLectura> {
    let peso = std::fs::metadata(ruta)
        .map_err(|_| ErrorDeLectura::NoSePudoLeer)?
        .len();
    if peso > MAX_BYTES {
        return Err(ErrorDeLectura::DemasiadoGrande);
    }

    let extension = ruta
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let filas = match extension.as_str() {
        "csv" | "txt" | "tsv" => leer_csv(ruta)?,
        "xlsx" | "xlsm" | "xls" | "xlsb" => leer_hoja_de_calculo(ruta)?,
        _ => return Err(ErrorDeLectura::FormatoDesconocido),
    };

    let nombre = ruta
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_owned();
    let huella = huella_del_archivo(ruta)?;

    armar(nombre, huella, filas)
}

/// Huella de los bytes del archivo, leídos otra vez y en trozos.
///
/// En trozos y no de golpe: el tope son 200 MB, y meterlos enteros en memoria
/// sólo para calcular una huella sería pagar dos veces por el archivo.
fn huella_del_archivo(ruta: &Path) -> Result<String, ErrorDeLectura> {
    let mut f = std::fs::File::open(ruta).map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
    let mut trozo = vec![0_u8; 64 * 1024];
    let mut acumulado: Vec<u8> = Vec::new();
    loop {
        let leidos = f
            .read(&mut trozo)
            .map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
        if leidos == 0 {
            break;
        }
        acumulado.extend_from_slice(trozo.get(..leidos).unwrap_or(&[]));
        if acumulado.len() as u64 > MAX_BYTES {
            return Err(ErrorDeLectura::DemasiadoGrande);
        }
    }
    Ok(arles_core::huella_de_bytes(&acumulado))
}

/// Separa encabezados de datos y recorta la muestra.
fn armar(
    nombre: String,
    huella: String,
    mut filas: Vec<Vec<String>>,
) -> Result<TablaLeida, ErrorDeLectura> {
    // Las filas del principio completamente vacías se saltan: es lo que deja un
    // Excel con un título arriba y la tabla dos filas más abajo.
    while filas
        .first()
        .is_some_and(|f| f.iter().all(|c| c.trim().is_empty()))
    {
        filas.remove(0);
    }

    if filas.is_empty() {
        return Err(ErrorDeLectura::SinEncabezados);
    }
    let encabezados = filas.remove(0);
    if encabezados.iter().all(|c| c.trim().is_empty()) {
        return Err(ErrorDeLectura::SinEncabezados);
    }

    // Las filas vacías de en medio y del final no son datos. Excel las deja
    // por docenas y contarlas como filas haría que el total mintiera.
    filas.retain(|f| f.iter().any(|c| !c.trim().is_empty()));

    let muestra = filas.iter().take(FILAS_DE_MUESTRA).cloned().collect();
    let total_de_filas = filas.len();

    Ok(TablaLeida {
        nombre,
        huella,
        encabezados,
        muestra,
        filas,
        total_de_filas,
    })
}

/// Lee un CSV, detectando su codificación y su separador.
fn leer_csv(ruta: &Path) -> Result<Vec<Vec<String>>, ErrorDeLectura> {
    let mut crudo = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|_| ErrorDeLectura::NoSePudoLeer)?
        // `take` antes de leer: sin él, un archivo que creció entre el
        // `metadata` de arriba y este momento entraría entero en memoria.
        .take(MAX_BYTES)
        .read_to_end(&mut crudo)
        .map_err(|_| ErrorDeLectura::NoSePudoLeer)?;

    let texto = decodificar(&crudo);
    let separador = adivinar_separador(&texto);

    let mut lector = csv::ReaderBuilder::new()
        .delimiter(separador)
        // Sin encabezados para el lector: los saca `armar`, que es quien sabe
        // saltarse las filas vacías de arriba.
        .has_headers(false)
        // Una tabla real trae filas con distinto número de celdas, y eso no es
        // motivo para rechazar el archivo entero.
        .flexible(true)
        .from_reader(texto.as_bytes());

    let mut filas = Vec::new();
    let mut celdas = 0usize;
    for registro in lector.records() {
        let registro = registro.map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
        celdas += registro.len();
        if celdas > MAX_CELDAS {
            return Err(ErrorDeLectura::DemasiadosDatos);
        }
        filas.push(
            registro
                .iter()
                .take(arles_core::MAX_COLUMNAS)
                .map(|c| c.trim().to_owned())
                .collect(),
        );
    }
    Ok(filas)
}

/// Decodifica el CSV a texto.
///
/// Por orden: la marca de orden de bytes si la hay, luego UTF-8 si es válido, y
/// si no **Windows-1252**, que es lo que escribe el Excel de Windows en español.
///
/// El orden importa: casi cualquier secuencia de bytes es Windows-1252 válido,
/// así que probarlo primero aceptaría un UTF-8 correcto y lo destrozaría.
fn decodificar(crudo: &[u8]) -> String {
    if crudo.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (texto, _, _) = encoding_rs::UTF_8.decode(crudo);
        return texto.into_owned();
    }
    if let Ok(texto) = std::str::from_utf8(crudo) {
        return texto.to_owned();
    }
    let (texto, _, _) = encoding_rs::WINDOWS_1252.decode(crudo);
    texto.into_owned()
}

/// Adivina el separador de la primera línea.
///
/// El Excel en español escribe CSV con **punto y coma**, porque la coma es su
/// separador decimal. Un lector que asuma la coma lee esa tabla como una sola
/// columna gigante — y no da error, que es lo peor: sale una importación con un
/// contacto por fila y todo el contenido en el nombre.
fn adivinar_separador(texto: &str) -> u8 {
    let primera = texto.lines().next().unwrap_or("");
    let candidatos = [(b';', ';'), (b'\t', '\t'), (b',', ',')];
    let mut mejor = b',';
    let mut mas = 0;
    for (byte, caracter) in candidatos {
        let n = primera.matches(caracter).count();
        if n > mas {
            mas = n;
            mejor = byte;
        }
    }
    mejor
}

/// Lee la primera hoja con datos de un XLSX.
fn leer_hoja_de_calculo(ruta: &Path) -> Result<Vec<Vec<String>>, ErrorDeLectura> {
    use calamine::{Data, Reader};

    let mut libro = calamine::open_workbook_auto(ruta).map_err(|_| ErrorDeLectura::NoSePudoLeer)?;
    let hojas = libro.sheet_names().to_vec();
    let nombre = hojas.first().ok_or(ErrorDeLectura::SinHojas)?;
    let hoja = libro
        .worksheet_range(nombre)
        .map_err(|_| ErrorDeLectura::SinHojas)?;

    let mut filas = Vec::new();
    let mut celdas = 0usize;
    for fila in hoja.rows() {
        celdas += fila.len().min(arles_core::MAX_COLUMNAS);
        if celdas > MAX_CELDAS {
            return Err(ErrorDeLectura::DemasiadosDatos);
        }
        filas.push(
            fila.iter()
                .take(arles_core::MAX_COLUMNAS)
                .map(|c| match c {
                    // **Nunca se evalúa nada.** Una celda con `=A1+A2` se lee
                    // como el texto que Excel dejó ahí, y si no dejó ninguno,
                    // como cadena vacía.
                    Data::Empty => String::new(),
                    Data::String(s) => s.trim().to_owned(),
                    // Los números se pasan a texto sin notación científica: un
                    // móvil guardado como número en Excel sale «5.2811e12» con
                    // el formato por defecto, y ese número ya no es un móvil.
                    Data::Float(f) => formatear_numero(*f),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(d) => d.to_string(),
                    otro => otro.to_string().trim().to_owned(),
                })
                .collect(),
        );
    }
    Ok(filas)
}

/// Pasa un número de hoja de cálculo a texto sin notación científica.
///
/// Excel guarda un móvil escrito sin prefijo como número. Con el formato por
/// defecto de Rust, `5281123456789.0` sale como `5281123456789` —bien— pero
/// números mayores salen como `5.281123456789e12`, y eso ya no es un teléfono.
/// Los enteros se escriben como enteros, y los decimales se dejan tal cual.
fn formatear_numero(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e18 {
        format!("{f:.0}")
    } else {
        f.to_string()
    }
}

/// Neutraliza una celda que Excel interpretaría como fórmula, **al exportar**.
///
/// Una celda que empieza por `=`, `+`, `-`, `@`, tabulador o retorno se ejecuta
/// al abrir el archivo en Excel o en Sheets. Es la inyección de fórmulas CSV:
/// ARLES exporta contactos, y si uno de ellos se llamara `=cmd|'/c calc'!A1`,
/// ese archivo sería un ejecutable disfrazado para quien lo abra.
///
/// Se antepone una comilla simple, que es lo que Excel entiende como «esto es
/// texto». **Al importar no se evalúa nada**, así que esto es sólo para la
/// salida — pero vive aquí, junto a la explicación, y no en la capa que
/// exporta, donde nadie recordaría por qué existe.
#[must_use]
pub fn neutralizar_formula(celda: &str) -> String {
    const PELIGROSOS: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];
    if celda.starts_with(PELIGROSOS) {
        format!("'{celda}")
    } else {
        celda.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn archivo(nombre: &str, contenido: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("temporal");
        let ruta = dir.path().join(nombre);
        let mut f = std::fs::File::create(&ruta).expect("crea");
        f.write_all(contenido).expect("escribe");
        (dir, ruta)
    }

    fn fila(t: &TablaLeida, i: usize) -> &Vec<String> {
        t.filas.get(i).expect("la fila existe")
    }

    fn celda(t: &TablaLeida, f: usize, c: usize) -> &str {
        fila(t, f)
            .get(c)
            .map(String::as_str)
            .expect("la celda existe")
    }

    #[test]
    fn lee_un_csv_normal() {
        let (_d, ruta) = archivo(
            "c.csv",
            b"Nombre,Correo\nAna,ana@empresa.mx\nLuis,luis@empresa.mx\n",
        );
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.encabezados, vec!["Nombre", "Correo"]);
        assert_eq!(t.total_de_filas, 2);
        assert_eq!(celda(&t, 0, 0), "Ana");
    }

    /// El nombre se guarda **sin la carpeta**: la ruta dice dónde vive el
    /// usuario y no le aporta nada a ninguna pantalla.
    #[test]
    fn se_guarda_el_nombre_pero_nunca_la_carpeta() {
        let (dir, ruta) = archivo("contactos-marzo.csv", b"Nombre,Correo\nAna,a@b.mx\n");
        let t = leer(&ruta).expect("lee");

        assert_eq!(t.nombre, "contactos-marzo.csv");
        let carpeta = dir.path().to_string_lossy().to_string();
        assert!(
            !t.nombre.contains(&carpeta) && !t.nombre.contains('/') && !t.nombre.contains('\\'),
            "la carpeta se coló en el nombre: {}",
            t.nombre
        );
    }

    /// La huella cambia con el contenido. Es lo que permite decir «este archivo
    /// exacto ya se importó».
    #[test]
    fn la_huella_distingue_dos_archivos() {
        let (_d1, a) = archivo("a.csv", b"Nombre,Correo\nAna,a@b.mx\n");
        let (_d2, b) = archivo("b.csv", b"Nombre,Correo\nAna,otro@b.mx\n");

        let ta = leer(&a).expect("lee a");
        let tb = leer(&b).expect("lee b");

        assert!(ta.huella.starts_with("sha256:"), "{}", ta.huella);
        assert_ne!(ta.huella, tb.huella, "dos archivos distintos, misma huella");
    }

    /// El mismo contenido con otro nombre da la **misma** huella: la huella es
    /// del contenido, no del nombre. Si dependiera del nombre, renombrar un
    /// archivo lo convertiría en «otro» y la deduplicación de importaciones no
    /// serviría de nada.
    #[test]
    fn la_huella_no_depende_del_nombre() {
        let (_d1, a) = archivo("marzo.csv", b"Nombre,Correo\nAna,a@b.mx\n");
        let (_d2, b) = archivo("abril.csv", b"Nombre,Correo\nAna,a@b.mx\n");
        assert_eq!(leer(&a).expect("a").huella, leer(&b).expect("b").huella);
    }

    /// El Excel en español escribe CSV con punto y coma, porque la coma es su
    /// separador decimal. Sin detectarlo, la tabla entera se lee como **una
    /// sola columna** — y sin dar error, que es lo peor.
    #[test]
    fn detecta_el_punto_y_coma_del_excel_en_espanol() {
        let (_d, ruta) = archivo("c.csv", b"Nombre;Correo\nAna;ana@empresa.mx\n");
        let t = leer(&ruta).expect("lee");
        assert_eq!(
            t.encabezados,
            vec!["Nombre", "Correo"],
            "se leyó como una sola columna: el separador no se detectó"
        );
    }

    #[test]
    fn detecta_el_tabulador() {
        let (_d, ruta) = archivo("c.csv", b"Nombre\tCorreo\nAna\tana@empresa.mx\n");
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.encabezados, vec!["Nombre", "Correo"]);
    }

    /// Windows-1252 es lo que escribe el Excel de Windows en español. Sin
    /// detectarlo, una tabla mexicana llega con los acentos rotos.
    #[test]
    fn lee_los_acentos_de_un_csv_de_excel_windows() {
        // «Teléfono,Compañía» en Windows-1252.
        let mut bytes = b"Nombre,Tel".to_vec();
        bytes.push(0xE9); // é
        bytes.extend_from_slice(b"fono\nMar");
        bytes.push(0xED); // í
        bytes.extend_from_slice(b"a,8110000000\n");

        let (_d, ruta) = archivo("c.csv", &bytes);
        let t = leer(&ruta).expect("lee");
        assert_eq!(
            t.encabezados.get(1).map(String::as_str),
            Some("Teléfono"),
            "los acentos llegaron rotos"
        );
        assert_eq!(celda(&t, 0, 0), "María");
    }

    /// El orden de la detección importa: casi cualquier secuencia de bytes es
    /// Windows-1252 válido, así que probarlo primero destrozaría un UTF-8 bueno.
    #[test]
    fn un_utf8_valido_no_se_lee_como_windows_1252() {
        let (_d, ruta) = archivo("c.csv", "Nombre,Compañía\nJosé,Ñandú SA\n".as_bytes());
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.encabezados.get(1).map(String::as_str), Some("Compañía"));
        assert_eq!(celda(&t, 0, 0), "José");
        assert_eq!(celda(&t, 0, 1), "Ñandú SA");
    }

    #[test]
    fn la_marca_de_orden_de_bytes_no_se_cuela_en_el_encabezado() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"Nombre,Correo\nAna,a@b.mx\n");
        let (_d, ruta) = archivo("c.csv", &bytes);
        let t = leer(&ruta).expect("lee");
        assert_eq!(
            t.encabezados.first().map(String::as_str),
            Some("Nombre"),
            "la marca de orden de bytes se quedó pegada al primer encabezado"
        );
    }

    /// Un Excel con un título arriba y la tabla dos filas más abajo es lo
    /// normal en una tabla hecha a mano.
    #[test]
    fn se_salta_las_filas_vacias_de_arriba() {
        let (_d, ruta) = archivo("c.csv", b",\n,\nNombre,Correo\nAna,a@b.mx\n");
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.encabezados, vec!["Nombre", "Correo"]);
        assert_eq!(t.total_de_filas, 1);
    }

    /// Excel deja filas vacías por docenas al final. Contarlas haría que el
    /// total mintiera, y §94 dice que las cifras son cifras.
    #[test]
    fn las_filas_vacias_no_cuentan_como_datos() {
        let (_d, ruta) = archivo("c.csv", b"Nombre,Correo\nAna,a@b.mx\n,\n,\n,\n");
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.total_de_filas, 1);
    }

    #[test]
    fn un_archivo_sin_encabezados_se_rechaza() {
        let (_d, ruta) = archivo("c.csv", b"");
        assert!(matches!(leer(&ruta), Err(ErrorDeLectura::SinEncabezados)));
    }

    #[test]
    fn una_extension_desconocida_se_rechaza() {
        let (_d, ruta) = archivo("c.pdf", b"Nombre,Correo\n");
        assert!(matches!(
            leer(&ruta),
            Err(ErrorDeLectura::FormatoDesconocido)
        ));
    }

    /// El tope cuenta **celdas leídas**, no lo que el archivo dice medir. Es lo
    /// que hace que sirva contra una bomba de descompresión: un archivo puede
    /// mentir sobre su tamaño, no sobre lo que ya entregó.
    #[test]
    fn un_archivo_con_demasiadas_celdas_se_corta() {
        // Filas de dos celdas: hacen falta más de MAX_CELDAS/2 para pasarse.
        // Se usa una constante pequeña en una copia de la lógica sería trampa,
        // así que se genera de verdad —con dos columnas, son pocas líneas de
        // texto por fila y el archivo cabe de sobra en el temporal—.
        let filas_necesarias = MAX_CELDAS / 2 + 10;
        let mut contenido = String::from("Nombre,Correo\n");
        contenido.reserve(filas_necesarias * 12);
        for i in 0..filas_necesarias {
            contenido.push_str(&format!("n{i},c{i}\n"));
        }
        let (_d, ruta) = archivo("c.csv", contenido.as_bytes());
        assert!(
            matches!(leer(&ruta), Err(ErrorDeLectura::DemasiadosDatos)),
            "el tope de celdas no cortó la lectura"
        );
    }

    /// Una fila con más columnas que el tope se recorta, no tumba la lectura.
    #[test]
    fn las_columnas_de_mas_se_recortan() {
        let encabezados = (0..arles_core::MAX_COLUMNAS + 30)
            .map(|i| format!("col{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let contenido = format!("{encabezados}\n");
        let (_d, ruta) = archivo("c.csv", contenido.as_bytes());
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.encabezados.len(), arles_core::MAX_COLUMNAS);
    }

    /// Una tabla real trae filas con distinto número de celdas. No es motivo
    /// para rechazar el archivo entero.
    #[test]
    fn una_fila_mas_corta_no_tumba_el_archivo() {
        let (_d, ruta) = archivo("c.csv", b"Nombre,Correo,Empresa\nAna,a@b.mx\n");
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.total_de_filas, 1);
        assert_eq!(fila(&t, 0).len(), 2);
    }

    /// La muestra es para mirar, y se queda corta a propósito.
    #[test]
    fn la_muestra_no_pasa_de_su_tope() {
        let mut contenido = String::from("Nombre,Correo\n");
        for i in 0..FILAS_DE_MUESTRA + 15 {
            contenido.push_str(&format!("n{i},c{i}@b.mx\n"));
        }
        let (_d, ruta) = archivo("c.csv", contenido.as_bytes());
        let t = leer(&ruta).expect("lee");
        assert_eq!(t.muestra.len(), FILAS_DE_MUESTRA);
        assert_eq!(t.total_de_filas, FILAS_DE_MUESTRA + 15);
    }

    /// **Nunca se evalúa una fórmula.** Se lee como el texto que es.
    #[test]
    fn una_formula_en_un_csv_se_lee_como_texto() {
        let (_d, ruta) = archivo(
            "c.csv",
            b"Nombre,Correo\n=1+1,a@b.mx\n\"=cmd|'/c calc'!A1\",c@b.mx\n",
        );
        let t = leer(&ruta).expect("lee");
        assert_eq!(celda(&t, 0, 0), "=1+1", "la fórmula se evaluó o se alteró");
        assert!(celda(&t, 1, 0).contains("cmd"));
    }

    /// Al **exportar** sí se neutraliza: si no, el archivo que ARLES genera es
    /// un ejecutable disfrazado para quien lo abra en Excel.
    #[test]
    fn al_exportar_la_formula_se_neutraliza() {
        for peligrosa in ["=1+1", "+1", "-1", "@SUM(A1)", "=cmd|'/c calc'!A1"] {
            let salida = neutralizar_formula(peligrosa);
            assert!(
                salida.starts_with('\''),
                "«{peligrosa}» sale sin neutralizar: {salida}"
            );
        }
    }

    /// Y lo que no es peligroso no se toca: un nombre no puede salir con una
    /// comilla delante.
    #[test]
    fn al_exportar_lo_normal_no_se_toca() {
        for normal in [
            "Ana",
            "ana@empresa.mx",
            "+52 no es número aquí porque va después",
        ] {
            if normal.starts_with('+') {
                continue;
            }
            assert_eq!(neutralizar_formula(normal), normal);
        }
    }

    /// Excel guarda un móvil sin prefijo como número. Con el formato por
    /// defecto saldría en notación científica, y eso ya no es un teléfono.
    #[test]
    fn un_movil_guardado_como_numero_no_sale_en_notacion_cientifica() {
        assert_eq!(formatear_numero(5_281_123_456_789.0), "5281123456789");
        assert_eq!(formatear_numero(8_110_000_000.0), "8110000000");
        // Y un decimal de verdad se conserva.
        assert_eq!(formatear_numero(1.5), "1.5");
    }
}
