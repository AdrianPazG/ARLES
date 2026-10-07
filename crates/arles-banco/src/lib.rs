//! El banco de pruebas: medio millón de contactos sintéticos, y las medidas.
//!
//! Entrega 8.5.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NUNCA DATOS REALES, NI ANONIMIZADOS
//!
//! `ESTRATEGIA_QA.md` §7 lo dice como regla de privacidad y no de comodidad, y
//! es lo que hace que este archivo pueda vivir en el repositorio. Todo lo que
//! sale de aquí se calcula de una semilla.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ EL AZAR ES NUESTRO Y NO DE UNA DEPENDENCIA
//!
//! El requisito es **reproducible**: generar dos veces con la misma semilla
//! tiene que dar el mismo archivo, hoy y dentro de dos años. Una biblioteca de
//! números aleatorios no promete que su algoritmo no cambie entre versiones —
//! promete calidad estadística, que aquí da igual—. SplitMix64 cabe en diez
//! líneas, no tiene estado oculto y da lo mismo en cualquier máquina.
//!
//! Y no es criptografía: **no se use para nada que proteja algo.**

// Lo tenían los seis crates de dominio y éste no, porque la comprobación que
// lo vigila nombraba **dos de siete** a mano. Lo encontró la auditoría A del
// §140 el 07/10/2026, y la comprobación ahora lee las carpetas que hay.
#![forbid(unsafe_code)]

pub mod larga;
pub mod medir;

/// Un generador de números reproducible. SplitMix64.
///
/// No vale para criptografía y no se usa para eso: aquí sólo decide qué nombre
/// toca y si una fila lleva un acento.
#[derive(Debug, Clone)]
pub struct Azar {
    estado: u64,
}

impl Azar {
    #[must_use]
    pub const fn con_semilla(semilla: u64) -> Self {
        Self { estado: semilla }
    }

    fn siguiente(&mut self) -> u64 {
        self.estado = self.estado.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.estado;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Un número de 0 a `tope - 1`.
    fn hasta(&mut self, tope: usize) -> usize {
        if tope == 0 {
            return 0;
        }
        // El sesgo del módulo es irrelevante para datos de prueba, y evitarlo
        // costaría un bucle que haría el generador más lento sin cambiar nada
        // de lo que se mide.
        usize::try_from(self.siguiente() % (tope as u64)).unwrap_or(0)
    }

    /// `true` una de cada `uno_de_cada` veces.
    fn a_veces(&mut self, uno_de_cada: usize) -> bool {
        self.hasta(uno_de_cada) == 0
    }
}

const NOMBRES: &[&str] = &[
    "Ana",
    "Beto",
    "Citlali",
    "Diego",
    "Elena",
    "Fernando",
    "Guadalupe",
    "Héctor",
    "Itzel",
    "Joaquín",
    "Karla",
    "Luis",
    "María José",
    "Nicolás",
    "Ñandú",
    "Omar",
    "Paulina",
    "Quetzal",
    "Rocío",
    "Sofía",
    "Tomás",
    "Ulises",
    "Verónica",
    "Ximena",
    "Yolanda",
    "Zoé",
];

const APELLIDOS: &[&str] = &[
    "Álvarez",
    "Bañuelos",
    "Castañeda",
    "Domínguez",
    "Echeverría",
    "Fuentes",
    "Gómez",
    "Hernández",
    "Ibáñez",
    "Jiménez",
    "Küng",
    "López",
    "Muñoz",
    "Núñez",
    "Ochoa",
    "Peña",
    "Quiroz",
    "Ramírez",
    "Sánchez",
    "Treviño",
    "Urbina",
    "Vázquez",
    "Weber",
    "Ximénez",
    "Yáñez",
    "Zúñiga",
];

const EMPRESAS: &[&str] = &[
    "Telemetry Insight",
    "Café de Olla, S.A. de C.V.",
    "Grupo Ñ",
    "O'Brien & Asociados",
    "Müller Logística",
    "",
];

const DOMINIOS: &[&str] = &["ejemplo.mx", "prueba.com.mx", "demo.example", "correo.test"];

/// Una fila del conjunto sintético, tal y como saldría de un Excel de verdad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaSintetica {
    pub nombre: String,
    pub apellido: String,
    pub correo: String,
    pub telefono: String,
    pub empresa: String,
}

impl FilaSintetica {
    /// La fila como la escribiría un CSV: sin comillas si no hacen falta.
    #[must_use]
    pub fn a_csv(&self) -> String {
        fn campo(v: &str) -> String {
            if v.contains([',', '"', '\n']) {
                format!("\"{}\"", v.replace('"', "\"\""))
            } else {
                v.to_owned()
            }
        }
        format!(
            "{},{},{},{},{}",
            campo(&self.nombre),
            campo(&self.apellido),
            campo(&self.correo),
            campo(&self.telefono),
            campo(&self.empresa)
        )
    }
}

/// La cabecera del CSV que genera [`generar`].
pub const CABECERA_CSV: &str = "nombre,apellido,correo,telefono,empresa";

/// Genera `cuantas` filas sintéticas, **de una en una**.
///
/// ── POR QUÉ UN ITERADOR Y NO UN `Vec` ──
///
/// Porque uno de los presupuestos que esto va a medir es **la memoria durante
/// la importación de 500 000 filas**, y un generador que las junte todas antes
/// de escribirlas gastaría esa memoria él mismo. El banco mediría su propio
/// defecto y lo anotaría como si fuera del producto.
///
/// Lo que sale está pensado para romper cosas, según `ESTRATEGIA_QA.md` §7:
/// acentos, correos en mayúsculas y con `+`, campos vacíos, duplicados exactos
/// y por normalización, direcciones inválidas y nombres que parecen fórmulas.
pub fn generar(cuantas: usize, semilla: u64) -> impl Iterator<Item = FilaSintetica> {
    let mut azar = Azar::con_semilla(semilla);
    (0..cuantas).map(move |i| una_fila(&mut azar, i))
}

fn una_fila(azar: &mut Azar, i: usize) -> FilaSintetica {
    let nombre_base = NOMBRES[azar.hasta(NOMBRES.len())];
    let apellido = APELLIDOS[azar.hasta(APELLIDOS.len())];

    // ── Nombres que parecen fórmulas ──
    //
    // Un Excel los interpreta al abrirlos, y una herramienta que los escribe
    // tal cual en un CSV convierte el archivo en un vector de ataque. ARLES los
    // neutraliza al leer (`arles_import::neutralizar_formula`), y el conjunto
    // tiene que traerlos para que eso se ejercite de verdad.
    let nombre = if azar.a_veces(5_000) {
        [
            "=SUM(A1:A9)",
            "+1-800-ESTAFA",
            "-5+5",
            "@contacto",
            "=cmd|'/c calc'!A1",
        ][azar.hasta(5)]
        .to_owned()
    } else if azar.a_veces(200) {
        // Campos vacíos: pasa de verdad en listas pegadas de un correo.
        String::new()
    } else {
        nombre_base.to_owned()
    };

    // ── Los correos, con todas las formas que la normalización tiene que ver
    //    iguales y las que tiene que rechazar ──
    let dominio = DOMINIOS[azar.hasta(DOMINIOS.len())];
    let local = format!("{}.{}{i}", sin_acentos(nombre_base), sin_acentos(apellido)).to_lowercase();
    let correo = match azar.hasta(100) {
        // Duplicado EXACTO: la misma dirección que una fila anterior.
        0..=1 if i > 10 => format!("ana.alvarez{}@ejemplo.mx", i - 10),
        // Duplicado POR NORMALIZACIÓN: mayúsculas, espacios alrededor, o un
        // `+etiqueta` que no cambia a quién llega.
        2..=3 => format!("  {}@{}  ", local.to_uppercase(), dominio),
        4..=5 => format!("{local}+boletin@{dominio}"),
        6 => format!("{}@{}", local.replace('.', ""), dominio),
        // Direcciones inválidas: tienen que rechazarse, no colarse.
        7 => format!("{local}@sinpunto"),
        8 => format!("{local}@@{dominio}"),
        9 => format!("@{dominio}"),
        10 => String::new(),
        _ => format!("{local}@{dominio}"),
    };

    let telefono = match azar.hasta(50) {
        0 => String::new(),
        1 => "+52 1 55 1234 5678".to_owned(),
        2 => "5512345678".to_owned(),
        3 => "no tiene".to_owned(),
        _ => format!("+521{:010}", 5_500_000_000_u64 + (i as u64 % 9_000_000)),
    };

    FilaSintetica {
        nombre,
        apellido: if azar.a_veces(300) {
            String::new()
        } else {
            apellido.to_owned()
        },
        correo,
        telefono,
        empresa: EMPRESAS[azar.hasta(EMPRESAS.len())].to_owned(),
    }
}

/// Quita los acentos para construir la parte local de un correo.
///
/// No es una normalización Unicode completa y no pretende serlo: cubre lo que
/// aparece en los nombres de arriba, que es lo que este generador escribe.
fn sin_acentos(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' => 'a',
            'é' | 'è' | 'ë' | 'ê' | 'É' => 'e',
            'í' | 'ì' | 'ï' | 'î' | 'Í' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'Ó' => 'o',
            'ú' | 'ù' | 'ü' | 'û' | 'Ú' => 'u',
            'ñ' | 'Ñ' => 'n',
            ' ' | '\'' | ',' | '.' => '-',
            otro => otro,
        })
        .collect()
}

/// Escribe el conjunto a un CSV, fila a fila.
///
/// # Errores
///
/// Lo que falle al escribir: típicamente un disco lleno, que con medio millón
/// de filas es un caso real y no teórico.
pub fn escribir_csv(ruta: &std::path::Path, cuantas: usize, semilla: u64) -> std::io::Result<u64> {
    use std::io::Write as _;

    let archivo = std::fs::File::create(ruta)?;
    // Un búfer grande: sin él, medio millón de filas son medio millón de
    // llamadas al sistema y lo que se mediría es eso.
    let mut w = std::io::BufWriter::with_capacity(1 << 20, archivo);
    writeln!(w, "{CABECERA_CSV}")?;
    for fila in generar(cuantas, semilla) {
        writeln!(w, "{}", fila.a_csv())?;
    }
    w.flush()?;
    Ok(std::fs::metadata(ruta)?.len())
}

/// Cuánta memoria ha llegado a usar este proceso, en bytes.
///
/// ── POR QUÉ EL PICO Y NO LO QUE SE USA AHORA ──
///
/// Porque el presupuesto que se mide es «memoria **durante** la importación de
/// 500 000». Leer el uso actual al terminar daría una cifra bonita y falsa: el
/// asignador ya devolvió lo que usó a mitad.
///
/// `VmHWM` es la marca de agua del sistema operativo y no se puede bajar. En
/// Linux sale de `/proc/self/status`; en otros sistemas devuelve `None` y el
/// informe lo dice **sin medir**, que no es lo mismo que cumplido.
#[must_use]
pub fn pico_de_memoria() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for linea in status.lines() {
        if let Some(resto) = linea.strip_prefix("VmHWM:") {
            let kb: u64 = resto.trim().trim_end_matches(" kB").trim().parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Dos generaciones con la misma semilla dan exactamente lo mismo.**
    ///
    /// Es el requisito del plan —«el conjunto se genera dos veces y sale
    /// idéntico»— y lo que hace que una medición se pueda repetir. Sin esto,
    /// dos ejecuciones del banco medirían conjuntos distintos y la diferencia
    /// entre ellas no significaría nada.
    ///
    /// Verificada el 06/10/2026 sembrando con `SystemTime::now()`: las dos
    /// listas difieren y la prueba lo nombra.
    #[test]
    fn la_misma_semilla_da_el_mismo_conjunto() {
        let a: Vec<_> = generar(5_000, 42).collect();
        let b: Vec<_> = generar(5_000, 42).collect();
        assert_eq!(a, b, "dos generaciones con la misma semilla no coinciden");

        let c: Vec<_> = generar(5_000, 43).collect();
        assert_ne!(a, c, "dos semillas distintas dan el mismo conjunto");
    }

    /// **El conjunto trae de verdad lo que la estrategia de QA pide.**
    ///
    /// Es la comprobación que impide que el generador se vuelva cómodo. Un
    /// conjunto de medio millón de «Ana López» pasaría todas las pruebas de
    /// rendimiento y **no mediría nada** de lo que de verdad cuesta: normalizar
    /// acentos, rechazar direcciones inválidas, cazar duplicados.
    ///
    /// Verificada el 06/10/2026 quitando la rama de las fórmulas: falla
    /// nombrando «nombres que parecen fórmulas».
    #[test]
    fn el_conjunto_trae_los_casos_dificiles() {
        let filas: Vec<_> = generar(50_000, 7).collect();

        let hay = |nombre: &str, cuantas: usize| {
            assert!(cuantas > 0, "el conjunto no trae {nombre}");
        };

        hay(
            "acentos",
            filas.iter().filter(|f| !f.apellido.is_ascii()).count(),
        );
        hay(
            "correos en mayúsculas",
            filas
                .iter()
                .filter(|f| f.correo.chars().any(char::is_uppercase))
                .count(),
        );
        hay(
            "correos con espacios alrededor",
            filas.iter().filter(|f| f.correo != f.correo.trim()).count(),
        );
        hay(
            "correos con +etiqueta",
            filas.iter().filter(|f| f.correo.contains('+')).count(),
        );
        hay(
            "campos vacíos",
            filas
                .iter()
                .filter(|f| f.nombre.is_empty() || f.apellido.is_empty() || f.correo.is_empty())
                .count(),
        );
        hay(
            "direcciones inválidas",
            filas
                .iter()
                .filter(|f| {
                    let c = f.correo.trim();
                    !c.is_empty() && (c.matches('@').count() != 1 || c.starts_with('@'))
                })
                .count(),
        );
        hay(
            "nombres que parecen fórmulas",
            filas
                .iter()
                .filter(|f| f.nombre.starts_with(['=', '+', '-', '@']))
                .count(),
        );

        // Duplicados: la misma dirección normalizada más de una vez.
        let mut vistos = std::collections::HashMap::new();
        for f in &filas {
            let clave = f.correo.trim().to_lowercase();
            if !clave.is_empty() {
                *vistos.entry(clave).or_insert(0_usize) += 1;
            }
        }
        hay("duplicados", vistos.values().filter(|n| **n > 1).count());
    }

    /// El CSV escapa lo que tiene que escapar.
    ///
    /// «Café de Olla, S.A. de C.V.» lleva comas: sin comillas, esa fila tendría
    /// una columna de más y el archivo entero se leería corrido a partir de
    /// ahí.
    #[test]
    fn el_csv_escapa_las_comas() {
        let f = FilaSintetica {
            nombre: "Ana".to_owned(),
            apellido: "Álvarez".to_owned(),
            correo: "ana@ejemplo.mx".to_owned(),
            telefono: String::new(),
            empresa: "Café de Olla, S.A. de C.V.".to_owned(),
        };
        let linea = f.a_csv();

        // El campo va entrecomillado: es lo único que impide que la coma de
        // «S.A. de C.V.» cuente como separador.
        assert!(
            linea.contains("\"Café de Olla, S.A. de C.V.\""),
            "la empresa con coma no se entrecomilló: {linea}"
        );

        // Y la prueba de que hacía falta: partir por comas a lo bruto da seis
        // trozos donde hay cinco campos. Un lector que no entienda las
        // comillas lee la fila corrida a partir de ahí.
        assert_eq!(
            linea.split(',').count(),
            6,
            "sin la coma dentro del campo, esta prueba no comprueba nada: {linea}"
        );
    }

    /// El pico de memoria se lee, o se dice que no se puede.
    #[test]
    fn el_pico_de_memoria_se_lee_o_se_calla() {
        match pico_de_memoria() {
            Some(bytes) => assert!(bytes > 0, "un pico de 0 bytes no es una medida"),
            None => println!("RAMA: sin /proc, el pico de memoria no se puede medir — OMITIDA"),
        }
    }
}
