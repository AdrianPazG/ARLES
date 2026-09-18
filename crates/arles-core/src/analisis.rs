//! Qué va a pasar si se importa este archivo (entrega 3.3).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NADA SE ESCRIBE AQUÍ, Y ESO ES LA DECISIÓN
//!
//! Dirección eligió que ARLES **enseñe los choques antes de importar**, frente a
//! saltarlos en silencio o fusionar solo. El motivo es que una importación de
//! diez mil filas es irreversible en la práctica: fusionar sobrescribe datos sin
//! preguntar, y saltar deja huecos de los que nadie se entera hasta mucho
//! después, cuando una campaña no llega a quien tenía que llegar.
//!
//! Así que este módulo **sólo mira**. Recorre el archivo, lo compara con lo que
//! ya hay y consigo mismo, y devuelve un informe. Escribir es otro paso, y sólo
//! ocurre si una persona lo pide después de ver esto.
//!
//! Y no toca la base: recibe las direcciones que ya existen en un mapa. Es lo
//! que permite probar la clasificación entera sin montar una base de datos —y
//! lo que mantiene este crate sin I/O—.
//! ─────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;

use serde::Serialize;

use crate::contacto::{DatosDeContacto, ErrorDeContacto};
use crate::importacion::{CampoImportable, fila_a_borrador};

/// Una fila que no se va a importar, y por qué.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilaRechazada {
    /// El número de fila **tal y como lo enseña Excel**: la primera fila de
    /// datos es la 2, porque la 1 son los encabezados.
    ///
    /// No es una coquetería: quien reciba este informe va a abrir su archivo
    /// para corregirlo, y un número que no coincide con el de su pantalla le
    /// hace contar filas a mano.
    pub fila: usize,
    /// Clave de i18n del motivo.
    pub motivo: String,
    /// Lo que se pueda enseñar de la fila para reconocerla: el nombre, o la
    /// primera celda con algo.
    pub referencia: String,
}

/// Una dirección del archivo que ya pertenece a alguien.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choque {
    pub fila: usize,
    /// La dirección en conflicto, ya normalizada. Es la que se compara.
    pub direccion: String,
    /// Con quién choca: el nombre del contacto que ya la tiene, o el número de
    /// la fila anterior del archivo si el choque es dentro del propio archivo.
    pub con: String,
    /// `true` si choca con otra fila del mismo archivo, `false` si con la base.
    ///
    /// Se distingue porque la acción es distinta: un duplicado dentro del
    /// archivo se arregla editando el archivo; uno contra la base significa que
    /// ese contacto ya está dado de alta.
    pub dentro_del_archivo: bool,
}

/// El informe completo de lo que pasaría al importar.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analisis {
    /// Las filas que se importarían tal cual.
    ///
    /// Van **validadas**, no en borrador: si el análisis dijera «lista» y luego
    /// el alta la rechazara, el informe estaría mintiendo.
    #[serde(skip)]
    pub listas: Vec<DatosDeContacto>,
    /// A qué fila del archivo corresponde cada una de [`Self::listas`].
    pub filas_de_las_listas: Vec<usize>,
    pub choques: Vec<Choque>,
    pub rechazadas: Vec<FilaRechazada>,
    /// Filas cuyas direcciones no pasan la validación del núcleo.
    pub invalidas: Vec<FilaRechazada>,
}

impl Analisis {
    /// Cuántas se importarían. Es una cifra, no un adjetivo (§94).
    #[must_use]
    pub fn cuantas_listas(&self) -> usize {
        self.listas.len()
    }

    /// ¿Hay algo que mirar antes de importar?
    #[must_use]
    pub fn hay_que_revisar(&self) -> bool {
        !self.choques.is_empty() || !self.rechazadas.is_empty() || !self.invalidas.is_empty()
    }
}

/// La primera fila de datos de una hoja de cálculo es la 2.
const PRIMERA_FILA_DE_DATOS: usize = 2;

/// Algo con lo que reconocer la fila en el informe.
fn referencia_de(fila: &[String]) -> String {
    fila.iter()
        .find(|c| !c.trim().is_empty())
        .map(|c| {
            let t = c.trim();
            // Recortado: una celda de mil caracteres no ayuda a reconocer nada
            // y sí puede romper la maquetación del informe.
            if t.chars().count() > 40 {
                t.chars().take(40).collect::<String>() + "…"
            } else {
                t.to_owned()
            }
        })
        .unwrap_or_default()
}

/// Clasifica cada fila del archivo **sin escribir nada**.
///
/// `ya_existen` mapea cada dirección normalizada que ya está en la base al
/// nombre de quien la tiene. Se pasa como mapa y no se consulta aquí: este
/// crate no hace I/O, y así la clasificación entera se prueba sin base de datos.
///
/// `pais_por_defecto` es el de la empresa, y sólo completa móviles escritos sin
/// prefijo internacional.
#[must_use]
pub fn analizar(
    mapeo: &[CampoImportable],
    filas: &[Vec<String>],
    pais_por_defecto: &str,
    ya_existen: &HashMap<String, String>,
) -> Analisis {
    let mut informe = Analisis::default();

    // Direcciones vistas en este archivo, con la fila donde aparecieron. Es lo
    // que detecta el duplicado **dentro** del archivo: sin esto, dos filas con
    // el mismo correo pasarían el análisis y la segunda reventaría al escribir,
    // ya con la primera dentro.
    let mut vistas: HashMap<String, usize> = HashMap::new();

    for (i, fila) in filas.iter().enumerate() {
        let numero = i + PRIMERA_FILA_DE_DATOS;

        let borrador = match fila_a_borrador(mapeo, fila) {
            Ok(b) => b,
            Err(motivo) => {
                informe.rechazadas.push(FilaRechazada {
                    fila: numero,
                    motivo: motivo.clave_i18n().to_owned(),
                    referencia: referencia_de(fila),
                });
                continue;
            }
        };

        // La validación es la del núcleo, la misma que usa el alta a mano. Dos
        // validaciones serían dos reglas que mantener iguales, y el día que
        // divergieran la importación aceptaría lo que el formulario rechaza.
        let datos = match DatosDeContacto::validar(&borrador, pais_por_defecto) {
            Ok(d) => d,
            Err(errores) => {
                informe.invalidas.push(FilaRechazada {
                    fila: numero,
                    motivo: clave_del_primer_error(&errores),
                    referencia: referencia_de(fila),
                });
                continue;
            }
        };

        // Choques. Se miran **todas** las direcciones de la fila, no sólo la
        // principal: basta una repetida para que el alta falle entera.
        let mut choco = false;
        for canal in &datos.canales {
            let direccion = &canal.valor_normalizado;
            if let Some(dueno) = ya_existen.get(direccion) {
                informe.choques.push(Choque {
                    fila: numero,
                    direccion: direccion.clone(),
                    con: dueno.clone(),
                    dentro_del_archivo: false,
                });
                choco = true;
            } else if let Some(antes) = vistas.get(direccion) {
                informe.choques.push(Choque {
                    fila: numero,
                    direccion: direccion.clone(),
                    con: antes.to_string(),
                    dentro_del_archivo: true,
                });
                choco = true;
            }
        }

        if choco {
            continue;
        }

        // Sólo se apuntan como vistas las direcciones de una fila que **sí** se
        // va a importar. Si se apuntaran las de una fila con choque, la
        // siguiente que trajera la misma dirección chocaría contra una fila que
        // tampoco va a entrar, y el informe culparía a la fila equivocada.
        for canal in &datos.canales {
            vistas.insert(canal.valor_normalizado.clone(), numero);
        }

        informe.filas_de_las_listas.push(numero);
        informe.listas.push(datos);
    }

    informe
}

/// La clave del primer error, que es la que se enseña en el informe.
///
/// Se queda con uno y no con todos a propósito: el informe de una importación
/// de diez mil filas con cinco errores cada una es ilegible. Quien quiera el
/// detalle abre la fila en su archivo, que es donde va a corregirla.
fn clave_del_primer_error(errores: &[ErrorDeContacto]) -> String {
    errores
        .first()
        .map(|e| e.clave.to_owned())
        .unwrap_or_else(|| "contacto.error.invalido".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canal::Canal;
    use crate::importacion::MotivoDeRechazo;

    fn mapeo_normal() -> Vec<CampoImportable> {
        vec![
            CampoImportable::Nombre,
            CampoImportable::Correo,
            CampoImportable::WhatsApp,
        ]
    }

    fn f(nombre: &str, correo: &str, movil: &str) -> Vec<String> {
        vec![nombre.to_owned(), correo.to_owned(), movil.to_owned()]
    }

    fn choque(a: &Analisis, i: usize) -> &Choque {
        a.choques.get(i).expect("el choque existe")
    }

    fn rechazada(a: &Analisis, i: usize) -> &FilaRechazada {
        a.rechazadas.get(i).expect("la fila existe")
    }

    #[test]
    fn un_archivo_limpio_sale_entero() {
        let filas = vec![
            f("Ana", "ana@empresa.mx", ""),
            f("Luis", "luis@empresa.mx", "8110000001"),
        ];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());
        assert_eq!(a.cuantas_listas(), 2);
        assert!(!a.hay_que_revisar());
    }

    /// Los números de fila son los de **Excel**: la primera de datos es la 2.
    /// Quien reciba el informe va a abrir su archivo para corregirlo.
    #[test]
    fn las_filas_se_numeran_como_en_excel() {
        let filas = vec![f("Ana", "no-es-un-correo", "")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());
        assert_eq!(
            a.invalidas.first().map(|r| r.fila),
            Some(2),
            "la primera fila de datos tiene que ser la 2, que es la que ve el usuario"
        );
    }

    /// El choque contra la base dice **con quién**, no sólo que lo hay.
    #[test]
    fn un_choque_con_la_base_dice_de_quien_es_la_direccion() {
        let mut ya = HashMap::new();
        ya.insert("ana@empresa.mx".to_owned(), "Ana Ruiz".to_owned());

        let filas = vec![f("Ana", "ana@empresa.mx", "")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &ya);

        assert_eq!(a.cuantas_listas(), 0);
        assert_eq!(a.choques.len(), 1);
        assert_eq!(choque(&a, 0).con, "Ana Ruiz");
        assert_eq!(choque(&a, 0).direccion, "ana@empresa.mx");
        assert!(!choque(&a, 0).dentro_del_archivo);
    }

    /// Un duplicado **dentro del archivo** también se detecta. Sin esto, la
    /// primera entraría y la segunda reventaría al escribir, con media
    /// importación ya hecha.
    #[test]
    fn dos_filas_del_archivo_con_la_misma_direccion_chocan_entre_si() {
        let filas = vec![
            f("Ana", "ana@empresa.mx", ""),
            f("Ana de nuevo", "ana@empresa.mx", ""),
        ];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());

        assert_eq!(a.cuantas_listas(), 1, "la primera sí entra");
        assert_eq!(a.choques.len(), 1);
        assert!(choque(&a, 0).dentro_del_archivo);
        assert_eq!(choque(&a, 0).fila, 3);
        assert_eq!(choque(&a, 0).con, "2", "señala la fila con la que choca");
    }

    /// La comparación es sobre la forma **normalizada**. «Ana@Empresa.MX» y
    /// «ana@empresa.mx» son la misma dirección, y si no se detectara el choque,
    /// entrarían las dos y la deduplicación quedaría rota desde el primer día.
    #[test]
    fn el_choque_se_detecta_aunque_se_escriba_distinto() {
        let mut ya = HashMap::new();
        ya.insert("ana@empresa.mx".to_owned(), "Ana Ruiz".to_owned());

        let filas = vec![f("Ana", "  Ana@Empresa.MX  ", "")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &ya);

        assert_eq!(a.choques.len(), 1, "no se detectó: se comparó lo escrito");
        assert_eq!(a.cuantas_listas(), 0);
    }

    /// Y lo mismo con el «1» del móvil mexicano: «+52 1 81…» y «81…» son el
    /// mismo número. Es la trampa que rompe la deduplicación entera.
    #[test]
    fn el_choque_detecta_el_movil_con_y_sin_el_uno() {
        let mut ya = HashMap::new();
        ya.insert("+528112345678".to_owned(), "Luis".to_owned());

        let filas = vec![f("Luis", "", "+52 1 81 1234 5678")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &ya);

        assert_eq!(
            a.choques.len(),
            1,
            "el móvil con el «1» no se reconoció como el mismo número"
        );
    }

    /// Se miran **todas** las direcciones de la fila, no sólo la principal:
    /// basta una repetida para que el alta falle entera.
    #[test]
    fn el_choque_se_busca_en_todos_los_canales_de_la_fila() {
        let mut ya = HashMap::new();
        ya.insert("+528112345678".to_owned(), "Luis".to_owned());

        // El correo es nuevo; el móvil no.
        let filas = vec![f("Ana", "nueva@empresa.mx", "8112345678")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &ya);

        assert_eq!(a.choques.len(), 1);
        assert_eq!(a.cuantas_listas(), 0, "la fila entera se queda fuera");
    }

    /// Una fila que choca **no** deja sus direcciones apuntadas. Si lo hiciera,
    /// la siguiente con la misma dirección chocaría contra una fila que tampoco
    /// va a entrar, y el informe culparía a la fila equivocada.
    #[test]
    fn una_fila_que_choca_no_arrastra_a_la_siguiente() {
        let mut ya = HashMap::new();
        ya.insert("ana@empresa.mx".to_owned(), "Ana Ruiz".to_owned());

        let filas = vec![
            f("Ana", "ana@empresa.mx", "8110000001"),
            f("Otra", "otra@empresa.mx", "8110000001"),
        ];
        let a = analizar(&mapeo_normal(), &filas, "MX", &ya);

        // La fila 3 choca por el móvil, pero contra la BASE no —el móvil no
        // está— sino contra la fila 2, que tampoco entra. Eso sería culpar a la
        // fila equivocada, así que la 3 entra.
        assert_eq!(a.choques.len(), 1, "sólo la fila 2 choca");
        assert_eq!(choque(&a, 0).fila, 2);
        assert_eq!(a.cuantas_listas(), 1);
        assert_eq!(a.filas_de_las_listas, vec![3]);
    }

    /// Una fila sin ninguna forma de contacto se rechaza con su motivo, no se
    /// cuela vacía.
    #[test]
    fn una_fila_sin_canales_se_rechaza_con_su_motivo() {
        let filas = vec![f("Ana", "", "")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());

        assert_eq!(a.rechazadas.len(), 1);
        assert_eq!(
            rechazada(&a, 0).motivo,
            MotivoDeRechazo::SinCanales.clave_i18n()
        );
        assert_eq!(rechazada(&a, 0).referencia, "Ana");
        assert_eq!(a.cuantas_listas(), 0);
    }

    /// Una dirección mal escrita va a «inválidas», no a «rechazadas»: son dos
    /// cosas distintas y se corrigen distinto. Una fila sin canales le falta un
    /// dato; una inválida lo tiene mal escrito.
    #[test]
    fn una_direccion_mal_escrita_es_invalida_no_rechazada() {
        let filas = vec![f("Ana", "esto-no-es-un-correo", "")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());

        assert_eq!(a.invalidas.len(), 1);
        assert!(a.rechazadas.is_empty());
        // La familia es `contacto.error.*`, la misma que usa el formulario de
        // alta a mano. Que sean la misma no es casualidad: es que la validación
        // también es la misma, y un motivo de otra familia querría decir que la
        // importación se ha traído su propia validación por detrás.
        assert!(
            a.invalidas
                .first()
                .is_some_and(|r| r.motivo.starts_with("contacto.error.")),
            "el motivo salió de otra familia: {:?}",
            a.invalidas.first().map(|r| &r.motivo)
        );
    }

    /// El informe distingue los cuatro destinos. Si los mezclara, la pantalla no
    /// podría decir qué hacer con cada grupo.
    #[test]
    fn el_informe_separa_los_cuatro_destinos() {
        let mut ya = HashMap::new();
        ya.insert("existe@empresa.mx".to_owned(), "Quien sea".to_owned());

        let filas = vec![
            f("Bien", "bien@empresa.mx", ""),    // 2 · lista
            f("Choca", "existe@empresa.mx", ""), // 3 · choque
            f("Mal", "no-es-correo", ""),        // 4 · inválida
            f("Vacía", "", ""),                  // 5 · rechazada
        ];
        let a = analizar(&mapeo_normal(), &filas, "MX", &ya);

        assert_eq!(a.cuantas_listas(), 1);
        assert_eq!(a.choques.len(), 1);
        assert_eq!(a.invalidas.len(), 1);
        assert_eq!(a.rechazadas.len(), 1);
        assert!(a.hay_que_revisar());
        assert_eq!(a.filas_de_las_listas, vec![2]);
    }

    /// Lo que sale listo está **validado**, no en borrador. Si el análisis
    /// dijera «lista» y el alta la rechazara después, el informe mentiría.
    #[test]
    fn lo_que_sale_listo_ya_esta_validado_y_normalizado() {
        let filas = vec![f("Ana", "  Ana@Empresa.MX  ", "+52 1 81 1234 5678")];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());

        let c = a.listas.first().expect("una lista");
        let correo = c
            .canales
            .iter()
            .find(|k| k.canal == Canal::Correo)
            .expect("hay correo");
        assert_eq!(correo.valor_normalizado, "ana@empresa.mx");
        // Y lo escrito se conserva, para poder enseñarlo tal cual.
        assert_eq!(correo.valor_raw, "Ana@Empresa.MX");

        let movil = c
            .canales
            .iter()
            .find(|k| k.canal == Canal::WhatsApp)
            .expect("hay móvil");
        assert_eq!(
            movil.valor_normalizado, "+528112345678",
            "el «1» mexicano sigue dentro"
        );
    }

    /// La referencia sirve para reconocer la fila, y se recorta: una celda de
    /// mil caracteres no ayuda y rompe la maquetación del informe.
    #[test]
    fn la_referencia_se_recorta() {
        let largo = "x".repeat(200);
        let filas = vec![vec![largo, String::new(), String::new()]];
        let a = analizar(&mapeo_normal(), &filas, "MX", &HashMap::new());
        let r = &rechazada(&a, 0).referencia;
        assert!(
            r.chars().count() <= 41,
            "no se recortó: {} caracteres",
            r.chars().count()
        );
        assert!(r.ends_with('…'));
    }

    #[test]
    fn un_archivo_vacio_no_produce_nada() {
        let a = analizar(&mapeo_normal(), &[], "MX", &HashMap::new());
        assert_eq!(a.cuantas_listas(), 0);
        assert!(!a.hay_que_revisar());
    }
}
