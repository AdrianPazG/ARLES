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

/// Una fila del archivo cuya dirección **ya pidió no recibir**.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ SE ENSEÑA Y NO SE SALTA · Dirección, 21/09/2026
///
/// Estas filas **sí se importan**. El nombre, la empresa y el resto de los
/// datos valen igual, y esa persona puede volver a darse de alta algún día;
/// tirarlos sería perder información por una razón que no lo pide.
///
/// Lo que no se puede hacer es **callarlo**. Un archivo de diez mil filas
/// donde trescientas personas pidieron la baja produce una lista que dice diez
/// mil y de la que sólo nueve mil setecientas van a recibir algo. Esa
/// diferencia aparece después, cuando una campaña «llega» a menos gente de la
/// esperada y nadie sabe por qué.
///
/// Es la misma decisión que Dirección tomó con los choques el 18/09/2026:
/// enseñar antes de importar. Una cifra incómoda es mejor que una cifra que
/// miente (§94).
/// ─────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilaSuprimida {
    pub fila: usize,
    /// La dirección suprimida, ya normalizada.
    pub direccion: String,
    /// Clave de i18n del motivo por el que dejó de recibir.
    pub motivo: String,
    /// Lo que se pueda enseñar de la fila para reconocerla.
    pub referencia: String,
    /// `true` si **todas** las direcciones de esa fila están suprimidas.
    ///
    /// Es la distinción que importa: alguien que pidió no recibir correos pero
    /// sí WhatsApp **sigue siendo alcanzable**. Contar las dos cosas igual
    /// daría una cifra tan falsa como la de callarlo.
    pub sin_ningun_canal_libre: bool,
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
    /// Filas que **sí entran** pero cuya dirección pidió no recibir.
    ///
    /// Una fila puede aparecer aquí más de una vez si tiene dos direcciones
    /// suprimidas: cada dirección es una negativa distinta y la pantalla las
    /// enseña por separado.
    pub suprimidas: Vec<FilaSuprimida>,
    /// Nombres que van a aparecer **más de una vez** en la lista de contactos.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// POR QUÉ ESTO NO ES UN CHOQUE, Y POR QUÉ AUN ASÍ SE DICE
    ///
    /// Lo preguntó Dirección el 23/10/2026 con su caso: la misma empresa en
    /// dos campañas, con correos y teléfonos distintos en cada una.
    ///
    /// **No es un duplicado y ARLES hace lo correcto**: lo que hace único a un
    /// contacto es su dirección, nunca su nombre, así que `apazg@…` y
    /// `veor@…` entran como dos contactos separados y cada uno conserva lo
    /// suyo. Medido el 23/10/2026.
    ///
    /// Lo que sí pasa son dos cosas, y las dos se ven después:
    ///
    ///   1. **En la lista hay dos filas con el mismo nombre.** Se distinguen
    ///      por su correo y su teléfono, que son columnas propias, pero de un
    ///      vistazo la columna «Nombre» dice lo mismo dos veces.
    ///   2. **Y el saludo del correo será el mismo para los dos**, porque
    ///      `{{nombre}}` sale de ahí. Si la columna «Nombre» del archivo traía
    ///      la EMPRESA y no la persona —que es el caso del ejemplo—, los dos
    ///      reciben «Hola Telemetry Insight».
    ///
    /// Lo segundo es lo que de verdad importa, y es **lo único que ARLES no
    /// puede arreglar solo**: no hay de dónde sacar el nombre de la persona si
    /// el archivo no lo trae. Inventarlo —«Telemetry Insight (2)»— sería
    /// escribir en el correo un dato que nadie escribió.
    ///
    /// Así que se cuenta y se dice, antes de importar.
    pub nombres_repetidos: Vec<NombreRepetido>,
}

/// Un nombre que va a quedar repetido, con su cifra (§94).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NombreRepetido {
    /// Tal y como lo escribió el cliente, con sus mayúsculas.
    ///
    /// Se enseña el primero que apareció y no una forma normalizada: lo que
    /// hay que poder reconocer es lo que está en el archivo.
    pub nombre: String,
    /// Cuántos contactos van a llamarse así **al terminar**: los de este
    /// archivo más los que ya estaban en ARLES.
    ///
    /// Los dos sumandos, y no sólo los del archivo, porque el caso que lo
    /// motivó son **dos archivos distintos** —la campaña de enero y la de
    /// marzo—, y mirar sólo el que se está cargando no vería nada.
    pub cuantos: usize,
    /// Cuántos de ésos ya estaban en ARLES antes de este archivo.
    ///
    /// Se separa porque cambia qué hacer: si ya había tres, el archivo no es
    /// el problema; si los dos salen de aquí, se corrige el archivo y se
    /// vuelve a cargar.
    pub ya_estaban: usize,
}

impl Analisis {
    /// Cuántas se importarían. Es una cifra, no un adjetivo (§94).
    #[must_use]
    pub fn cuantas_listas(&self) -> usize {
        self.listas.len()
    }

    /// A cuántas de las que entran **se les puede escribir de verdad**.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LA CIFRA QUE DIRECCIÓN PIDIÓ VER
    ///
    /// [`Self::cuantas_listas`] dice cuántos contactos se crean.
    /// Esto dice a cuántos se les puede escribir. La diferencia son las
    /// personas que pidieron la baja y siguen en el archivo.
    ///
    /// Se descuentan **sólo** las filas que se quedan sin ningún canal libre.
    /// Alguien que dijo «no me escribas por correo» pero tiene WhatsApp sigue
    /// contando: descontarlo daría una cifra tan falsa como no decir nada.
    /// ─────────────────────────────────────────────────────────────────────
    #[must_use]
    pub fn cuantas_se_pueden_alcanzar(&self) -> usize {
        let sin_salida: std::collections::HashSet<usize> = self
            .suprimidas
            .iter()
            .filter(|s| s.sin_ningun_canal_libre)
            .map(|s| s.fila)
            .collect();
        self.filas_de_las_listas
            .iter()
            .filter(|f| !sin_salida.contains(f))
            .count()
    }

    /// ¿Hay algo que mirar antes de importar?
    #[must_use]
    pub fn hay_que_revisar(&self) -> bool {
        !self.choques.is_empty()
            || !self.rechazadas.is_empty()
            || !self.invalidas.is_empty()
            || !self.suprimidas.is_empty()
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
/// `suprimidas` mapea cada dirección normalizada que **pidió no recibir** a la
/// clave de i18n de su motivo. Va como mapa por lo mismo que `ya_existen`: este
/// crate no hace I/O.
///
/// `pais_por_defecto` es el de la empresa, y sólo completa móviles escritos sin
/// prefijo internacional.
#[must_use]
pub fn analizar(
    mapeo: &[CampoImportable],
    filas: &[Vec<String>],
    pais_por_defecto: &str,
    ya_existen: &HashMap<String, String>,
    suprimidas: &HashMap<String, String>,
    // Cuántos contactos de ARLES se llaman ya como cada nombre del archivo,
    // en minúsculas y recortado. Lo cuenta `arles-db` (el núcleo no toca la
    // base) y aquí sólo se suma.
    //
    // Sin esto, el caso que lo motivó —la misma empresa en dos archivos, con
    // correos distintos— no se vería: mirar sólo el archivo que se está
    // cargando no encuentra nada repetido.
    ya_se_llaman_asi: &HashMap<String, usize>,
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

        // Quién de esta fila ya había pedido no recibir.
        //
        // Va **después** del choque y antes de apuntar la fila como lista, y
        // ese orden importa: una fila que choca no entra, así que anunciar
        // además que está suprimida sería contarle al usuario dos problemas de
        // una fila que sólo tiene uno —y el que resuelve es el otro—.
        //
        // No se descarta la fila: entra igual (decisión de Dirección del
        // 21/09/2026, ver `FilaSuprimida`). Lo que cambia es que se dice.
        let cuantos_canales = datos.canales.len();
        let cuantos_suprimidos = datos
            .canales
            .iter()
            .filter(|c| suprimidas.contains_key(&c.valor_normalizado))
            .count();
        for canal in &datos.canales {
            if let Some(motivo) = suprimidas.get(&canal.valor_normalizado) {
                informe.suprimidas.push(FilaSuprimida {
                    fila: numero,
                    direccion: canal.valor_normalizado.clone(),
                    motivo: motivo.clone(),
                    referencia: referencia_de(fila),
                    // Sin canales libres sólo si **todos** están suprimidos.
                    // `cuantos_canales` nunca es cero: una fila sin ningún
                    // canal la rechaza el validador antes de llegar aquí.
                    sin_ningun_canal_libre: cuantos_suprimidos == cuantos_canales,
                });
            }
        }

        informe.filas_de_las_listas.push(numero);
        informe.listas.push(datos);
    }

    informe.nombres_repetidos = contar_nombres_repetidos(&informe.listas, ya_se_llaman_asi);
    informe
}

/// Nombre y apellido juntos, que es lo que se ve en la lista.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ NO BASTA CON EL NOMBRE DE PILA
///
/// Porque «Ana Ruiz» y «Ana Pérez» son dos personas distintas y comparar sólo
/// el nombre las daría por repetidas. Eso no es un detalle: en una tabla de
/// trescientas filas hay varias Anas y varios Luises, y un aviso que sale
/// siempre deja de leerse — que es peor que no tenerlo.
///
/// La primera versión de esto comparaba sólo `nombre`, y se corrigió antes de
/// entrar. Lo que hay que comparar es **lo que el usuario va a ver repetido**
/// en su lista, y ahí salen los dos campos.
///
/// Se usa aquí y en `arles-db`, con la misma forma: si una capa comparara de
/// otra manera, el recuento de los que ya estaban no cuadraría con el del
/// archivo y la cifra del aviso saldría mal.
pub fn nombre_entero(nombre: &str, apellido: &str) -> String {
    format!("{} {}", nombre.trim(), apellido.trim())
        .trim()
        .to_owned()
}

/// Qué nombres van a quedar repetidos al terminar.
///
/// ─────────────────────────────────────────────────────────────────────────
/// SE COMPARA SIN MAYÚSCULAS Y RECORTADO, Y NO ES UN DETALLE
///
/// «Telemetry Insight», «TELEMETRY INSIGHT» y « Telemetry Insight » son el
/// mismo nombre para quien lee la lista, y dos tablas de clientes distintos
/// casi nunca lo escriben igual. Comparando literalmente, el aviso callaría
/// justo en el caso más frecuente.
///
/// **Un nombre vacío no cuenta.** La columna «Nombre» puede no venir en el
/// archivo, y entonces todas las filas lo tendrían vacío: avisar de que
/// «quinientos contactos se llaman igual» cuando lo que pasa es que ninguno
/// tiene nombre es una cifra correcta sobre otra pregunta (§94), y además
/// taparía el aviso de verdad.
fn contar_nombres_repetidos(
    listas: &[DatosDeContacto],
    ya_se_llaman_asi: &HashMap<String, usize>,
) -> Vec<NombreRepetido> {
    let mut del_archivo: HashMap<String, (String, usize)> = HashMap::new();
    for datos in listas {
        let entero = nombre_entero(&datos.nombre, &datos.apellido);
        if entero.is_empty() {
            continue;
        }
        let clave = entero.to_lowercase();
        let entrada = del_archivo.entry(clave).or_insert_with(|| (entero, 0));
        entrada.1 += 1;
    }

    let mut salida: Vec<NombreRepetido> = del_archivo
        .into_iter()
        .filter_map(|(clave, (nombre, en_el_archivo))| {
            let ya_estaban = ya_se_llaman_asi.get(&clave).copied().unwrap_or(0);
            let cuantos = en_el_archivo + ya_estaban;
            // Uno solo no es una repetición. Avisar de cada nombre del archivo
            // sería un informe tan largo como la tabla.
            (cuantos > 1).then_some(NombreRepetido {
                nombre,
                cuantos,
                ya_estaban,
            })
        })
        .collect();

    // De más repetido a menos, y por nombre para desempatar: dos corridas
    // sobre el mismo archivo tienen que dar el mismo informe, y el orden de un
    // `HashMap` no lo garantiza.
    salida.sort_by(|a, b| {
        b.cuantos
            .cmp(&a.cuantos)
            .then_with(|| a.nombre.cmp(&b.nombre))
    });
    salida
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

    /// **El caso que preguntó Dirección el 23/10/2026, con sus datos.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// LO PRIMERO QUE ESTO COMPRUEBA ES QUE **NO** HAY PROBLEMA
    ///
    ///   «En caso que en dos o más campañas diferentes se repitan los
    ///   contactos, pero tengan correo o números diferentes…»
    ///
    /// Las dos filas entran como **dos contactos separados** y ninguna
    /// choca: lo que hace único a un contacto en ARLES es su dirección, nunca
    /// su nombre. Eso es lo correcto y es lo que ya hacía — medido el mismo
    /// día contra la base de verdad, no sólo aquí.
    ///
    /// Lo que sí pasa es que en la lista quedan dos filas con el mismo nombre,
    /// y **el `{{nombre}}` del correo será el mismo para los dos**. Eso es lo
    /// que se cuenta y se dice antes de importar.
    ///
    /// Verificada rompiéndola: sin la llamada a `contar_nombres_repetidos`,
    /// `nombres_repetidos` sale vacío y el assert da `left: 0, right: 1`.
    #[test]
    fn dos_correos_del_mismo_nombre_entran_los_dos_y_se_avisa() {
        let filas = vec![
            f("Telemetry Insight", "apazg@telemetrymx.com", "5611644230"),
            f("Telemetry Insight", "veor@telemetrymx.com", "5534003402"),
        ];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

        // Lo primero: los dos entran, y no hay ningún choque.
        assert_eq!(a.cuantas_listas(), 2, "son dos contactos distintos");
        assert_eq!(a.choques.len(), 0, "distinta dirección no es un choque");

        // Y lo segundo: se avisa de que van a llamarse igual.
        assert_eq!(a.nombres_repetidos.len(), 1);
        let r = a.nombres_repetidos.first().expect("hay uno");
        assert_eq!(r.nombre, "Telemetry Insight");
        assert_eq!(r.cuantos, 2);
        assert_eq!(r.ya_estaban, 0, "los dos salen de este archivo");
    }

    /// **Y el caso de verdad son DOS archivos, no uno.**
    ///
    /// Es como lo planteó Dirección: la campaña 1 y la campaña 2. Al cargar el
    /// segundo archivo, mirar sólo sus filas no encontraría nada repetido —hay
    /// una sola— y el aviso callaría justo cuando hace falta.
    ///
    /// Por eso `analizar` recibe cuántos se llaman ya así, contados en la base.
    ///
    /// Verificada rompiéndola: pasando un mapa vacío en vez del recuento, la
    /// lista de repetidos sale vacía.
    #[test]
    fn el_nombre_repetido_de_otro_archivo_tambien_cuenta() {
        // El segundo archivo trae una sola fila.
        let filas = vec![f("Telemetry Insight", "veor@telemetrymx.com", "")];
        // Y en ARLES ya hay uno que se llama así, del archivo anterior.
        let mut ya = HashMap::new();
        ya.insert("telemetry insight".to_owned(), 1_usize);

        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &ya,
        );

        assert_eq!(a.cuantas_listas(), 1);
        let r = a.nombres_repetidos.first().expect("se avisa");
        assert_eq!(r.cuantos, 2, "el del archivo y el que ya estaba");
        assert_eq!(r.ya_estaban, 1, "y se dice cuál de los dos ya estaba");
    }

    /// **Las mayúsculas no hacen dos nombres distintos.**
    ///
    /// Dos tablas de clientes distintos casi nunca escriben un nombre igual, y
    /// comparando literalmente el aviso callaría en el caso más frecuente.
    #[test]
    fn el_mismo_nombre_con_otras_mayusculas_cuenta_igual() {
        let filas = vec![
            f("Telemetry Insight", "a@t.mx", ""),
            f("TELEMETRY INSIGHT", "b@t.mx", ""),
            f("  telemetry insight  ", "c@t.mx", ""),
        ];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(a.nombres_repetidos.len(), 1, "es un solo nombre, no tres");
        assert_eq!(a.nombres_repetidos.first().expect("uno").cuantos, 3);
    }

    /// **Dos «Ana» con apellidos distintos NO son un nombre repetido.**
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL FALSO POSITIVO QUE ESTA PRUEBA EXISTE PARA QUE NO VUELVA
    ///
    /// La primera versión comparaba sólo el nombre de pila, y se corrigió
    /// antes de entrar. En una tabla de trescientas filas hay varias Anas y
    /// varios Luises: el aviso habría salido en casi todas las importaciones,
    /// y **un aviso que sale siempre deja de leerse** — que es peor que no
    /// tenerlo, porque el día que avise de verdad nadie lo mirará.
    ///
    /// Lo que se compara es lo que el usuario va a ver repetido en su lista, y
    /// ahí sale el nombre entero.
    ///
    /// Verificada rompiéndola: comparando sólo `datos.nombre`, sale
    /// `left: 1, right: 0`.
    #[test]
    fn dos_personas_con_el_mismo_nombre_de_pila_no_se_avisan() {
        let mapeo = vec![
            CampoImportable::Nombre,
            CampoImportable::Apellido,
            CampoImportable::Correo,
        ];
        let filas = vec![
            vec!["Ana".into(), "Ruiz".into(), "ana.r@empresa.mx".into()],
            vec!["Ana".into(), "Pérez".into(), "ana.p@empresa.mx".into()],
        ];
        let a = analizar(
            &mapeo,
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(a.cuantas_listas(), 2);
        assert_eq!(
            a.nombres_repetidos.len(),
            0,
            "«Ana Ruiz» y «Ana Pérez» son dos personas, no un nombre repetido"
        );
    }

    /// Y con el mismo apellido sí se avisa, que es la otra mitad.
    #[test]
    fn dos_personas_con_el_mismo_nombre_entero_si_se_avisan() {
        let mapeo = vec![
            CampoImportable::Nombre,
            CampoImportable::Apellido,
            CampoImportable::Correo,
        ];
        let filas = vec![
            vec!["Ana".into(), "Ruiz".into(), "ana1@empresa.mx".into()],
            vec!["Ana".into(), "Ruiz".into(), "ana2@empresa.mx".into()],
        ];
        let a = analizar(
            &mapeo,
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(
            a.nombres_repetidos.first().expect("se avisa").nombre,
            "Ana Ruiz"
        );
    }

    /// **Un nombre que sólo aparece una vez no es una repetición.**
    ///
    /// Sin esto, el informe tendría una línea por cada fila del archivo, y un
    /// aviso que sale siempre deja de leerse.
    #[test]
    fn un_nombre_que_no_se_repite_no_se_avisa() {
        let filas = vec![
            f("Ana", "ana@empresa.mx", ""),
            f("Luis", "luis@empresa.mx", ""),
        ];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(a.nombres_repetidos.is_empty());
    }

    /// **Y las filas sin nombre no cuentan como «todas se llaman igual».**
    ///
    /// La columna «Nombre» puede no venir en el archivo. Decir «quinientos
    /// contactos se llaman igual» cuando lo que pasa es que ninguno tiene
    /// nombre es una cifra correcta sobre otra pregunta (§94), y taparía el
    /// aviso de verdad.
    #[test]
    fn las_filas_sin_nombre_no_cuentan_como_repetidas() {
        let filas = vec![f("", "a@t.mx", ""), f("  ", "b@t.mx", "")];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(
            a.cuantas_listas(),
            2,
            "entran igual: el nombre no es obligatorio"
        );
        assert!(a.nombres_repetidos.is_empty(), "pero no se avisa de nada");
    }

    #[test]
    fn un_archivo_limpio_sale_entero() {
        let filas = vec![
            f("Ana", "ana@empresa.mx", ""),
            f("Luis", "luis@empresa.mx", "8110000001"),
        ];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(a.cuantas_listas(), 2);
        assert!(!a.hay_que_revisar());
    }

    /// Los números de fila son los de **Excel**: la primera de datos es la 2.
    /// Quien reciba el informe va a abrir su archivo para corregirlo.
    #[test]
    fn las_filas_se_numeran_como_en_excel() {
        let filas = vec![f("Ana", "no-es-un-correo", "")];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

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
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
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
        let a = analizar(
            &mapeo_normal(),
            &[],
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(a.cuantas_listas(), 0);
        assert!(!a.hay_que_revisar());
    }
    // ── Entrega 3.4 · quién ya había pedido no recibir ──────────────────

    /// Ayudante: el mapa de supresiones, por dirección normalizada.
    fn suprimida(direccion: &str) -> HashMap<String, String> {
        HashMap::from([(direccion.to_owned(), "supresion.motivo.baja".to_owned())])
    }

    /// **La decisión de Dirección del 21/09/2026.** La fila entra igual —el
    /// nombre y la empresa valen— pero el informe lo dice.
    #[test]
    fn una_direccion_suprimida_entra_pero_se_anuncia() {
        let filas = vec![f("Ana", "ana@empresa.mx", "")];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &suprimida("ana@empresa.mx"),
            &HashMap::new(),
        );

        assert_eq!(a.cuantas_listas(), 1, "la fila tenía que entrar igual");
        assert_eq!(a.suprimidas.len(), 1, "y tenía que anunciarse");
        let s = a.suprimidas.first().expect("hay una");
        assert_eq!(s.direccion, "ana@empresa.mx");
        assert_eq!(s.motivo, "supresion.motivo.baja");
        assert_eq!(s.fila, 2, "los números de fila son los de Excel");
        assert!(s.sin_ningun_canal_libre, "no le queda ningún canal abierto");
    }

    /// **La distinción que hace honesta a la cifra.** Quien dijo «no me
    /// escribas por correo» pero tiene WhatsApp **sigue siendo alcanzable**.
    ///
    /// Se verificó rompiendo la guarda: contando como sin salida a cualquiera
    /// con una dirección suprimida, esta prueba falla y dice que Ana se
    /// descontó teniendo WhatsApp libre.
    #[test]
    fn cerrar_un_canal_no_deja_a_nadie_incomunicado_si_le_queda_otro() {
        let filas = vec![f("Ana", "ana@empresa.mx", "8112345678")];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &suprimida("ana@empresa.mx"),
            &HashMap::new(),
        );

        assert_eq!(a.suprimidas.len(), 1);
        let s = a.suprimidas.first().expect("hay una");
        assert!(
            !s.sin_ningun_canal_libre,
            "le queda el WhatsApp y se la dio por incomunicada"
        );
        assert_eq!(
            a.cuantas_se_pueden_alcanzar(),
            1,
            "Ana se descontó del alcance teniendo un canal libre"
        );
    }

    /// Y si están cerrados los dos, sí se descuenta.
    #[test]
    fn quien_cerro_todos_sus_canales_no_cuenta_como_alcanzable() {
        let filas = vec![
            f("Ana", "ana@empresa.mx", "8112345678"),
            f("Beto", "beto@empresa.mx", ""),
        ];
        let mut sup = HashMap::new();
        sup.insert(
            "ana@empresa.mx".to_owned(),
            "supresion.motivo.baja".to_owned(),
        );
        sup.insert(
            "+528112345678".to_owned(),
            "supresion.motivo.queja".to_owned(),
        );

        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &sup,
            &HashMap::new(),
        );

        assert_eq!(a.cuantas_listas(), 2, "las dos entran");
        assert_eq!(
            a.cuantas_se_pueden_alcanzar(),
            1,
            "sólo Beto puede recibir, y la cifra tiene que decirlo"
        );
        assert_eq!(
            a.suprimidas.len(),
            2,
            "las dos direcciones de Ana son dos negativas distintas"
        );
        assert!(a.suprimidas.iter().all(|s| s.sin_ningun_canal_libre));
    }

    /// **Una fila que choca no se anuncia además como suprimida.**
    ///
    /// No entra, así que contarle dos problemas al usuario cuando sólo tiene
    /// uno —y el que resuelve es el otro— alarga el informe sin ayudar.
    #[test]
    fn una_fila_que_choca_no_se_cuenta_tambien_como_suprimida() {
        let filas = vec![f("Ana", "ana@empresa.mx", "")];
        let ya = HashMap::from([("ana@empresa.mx".to_owned(), "Ana Ruiz".to_owned())]);

        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &ya,
            &suprimida("ana@empresa.mx"),
            &HashMap::new(),
        );

        assert_eq!(a.choques.len(), 1);
        assert_eq!(a.cuantas_listas(), 0, "una fila con choque no entra");
        assert!(
            a.suprimidas.is_empty(),
            "se le anunciaron dos problemas a una fila que sólo tiene uno"
        );
    }

    /// Sin supresiones, nada cambia: el informe de la 3.3 sigue igual.
    #[test]
    fn sin_supresiones_el_informe_no_menciona_ninguna() {
        let filas = vec![f("Ana", "ana@empresa.mx", "")];
        let a = analizar(
            &mapeo_normal(),
            &filas,
            "MX",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

        assert!(a.suprimidas.is_empty());
        assert_eq!(a.cuantas_se_pueden_alcanzar(), a.cuantas_listas());
    }
}
