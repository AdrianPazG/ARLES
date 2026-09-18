//! Mapeo de columnas de un archivo importado (entrega 3.3).
//!
//! Este módulo **no lee archivos**: recibe encabezados y filas ya extraídos y
//! decide qué significa cada columna. Leer el XLSX o el CSV es I/O y vive en
//! otra capa; aquí sólo está la regla, que es lo que se puede probar sin montar
//! nada (regla de frontera 3.2).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! POR QUÉ EL MAPEO SE ADIVINA PERO NO SE IMPONE
//!
//! Dirección pidió que ARLES «organice las columnas en automático». Se hace, y
//! acierta en los encabezados normales. Pero la propuesta es **una propuesta**:
//! la pantalla la enseña y se puede cambiar antes de importar.
//!
//! La razón es que adivinar mal no es simétrico. Confundir «Empresa» con
//! «Nombre» produce contactos con el nombre mal y se ve enseguida. Confundir
//! una columna de teléfonos de oficina con la de móviles produce una lista a la
//! que se le manda WhatsApp a números que no lo tienen —y eso se paga en
//! calidad del número ante Meta, que es de lo poco que no se puede deshacer.
//!
//! Por eso la función se llama «adivinar»: dice lo que es, no lo que decide.
//! ─────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

use crate::canal::Canal;
use crate::contacto::{BorradorDeCanal, BorradorDeContacto};
use crate::error::CoreError;

/// Cuántas filas se admiten en un archivo.
///
/// T-7 pide operar con 500 000 contactos, así que el tope no puede ser menor.
/// Existe de todas formas porque un archivo sin tope es una forma de agotar la
/// memoria del equipo con un archivo perfectamente válido
/// (THREAT_MODEL.md §4.3).
pub const MAX_FILAS: usize = 500_000;

/// Cuántas columnas se miran.
///
/// Una hoja de cálculo puede declarar dieciséis mil columnas. Mirarlas todas
/// para adivinar su campo es trabajo por nada: ningún archivo de contactos real
/// pasa de unas pocas decenas, y más allá de aquí lo que hay es basura o un
/// intento de que ARLES se atragante.
pub const MAX_COLUMNAS: usize = 64;

/// Qué es cada columna del archivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampoImportable {
    Nombre,
    Apellido,
    /// Nombre y apellido en una sola columna. Se parte por el primer espacio.
    NombreCompleto,
    Empresa,
    Correo,
    WhatsApp,
    /// No se importa. Es el destino de todo lo que no se reconoce.
    ///
    /// **Ignorar es el valor por defecto, no un castigo.** Una columna que no
    /// se entiende no se adivina «a lo que más se parezca»: se deja fuera y se
    /// enseña para que una persona decida.
    Ignorar,
}

impl CampoImportable {
    /// Clave de i18n del nombre del campo, para el desplegable de la pantalla.
    #[must_use]
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::Nombre => "importacion.campo.nombre",
            Self::Apellido => "importacion.campo.apellido",
            Self::NombreCompleto => "importacion.campo.nombreCompleto",
            Self::Empresa => "importacion.campo.empresa",
            Self::Correo => "importacion.campo.correo",
            Self::WhatsApp => "importacion.campo.whatsapp",
            Self::Ignorar => "importacion.campo.ignorar",
        }
    }

    /// Todos, en el orden en que se ofrecen.
    pub const TODOS: &'static [Self] = &[
        Self::Nombre,
        Self::Apellido,
        Self::NombreCompleto,
        Self::Empresa,
        Self::Correo,
        Self::WhatsApp,
        Self::Ignorar,
    ];
}

/// De dónde salió una lista importada (ADR-0013 §1).
///
/// ─────────────────────────────────────────────────────────────────────────
/// LISTA CERRADA, Y CON «OTRO» DENTRO
///
/// Dirección confirmó estas cinco el 18/09/2026. Es cerrada y no texto libre
/// porque un campo libre acaba lleno de «varios», «de siempre» y cadenas
/// vacías, y entonces no hay nada que analizar el día que llegue una
/// reclamación.
///
/// Y `Otro` existe **a propósito**: sin él, quien no encuentre su caso elegiría
/// el que más se le parezca, y un «clientes existentes» falso es peor prueba
/// que un «otro» honesto.
/// ─────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrigenDeLaLista {
    /// Se dieron de alta ellos mismos en un formulario de la empresa.
    FormularioPropio,
    /// Ya son clientes: hay relación comercial previa.
    ClientesExistentes,
    /// Dejaron sus datos en un evento, una feria o un stand.
    EventoOFeria,
    /// Salieron de un directorio público. **El más delicado**: que un dato sea
    /// público no lo hace libre de usar para publicidad.
    DirectorioPublico,
    Otro,
}

impl OrigenDeLaLista {
    /// Los cinco, en el orden en que se ofrecen.
    ///
    /// `Otro` va el último a propósito: es la salida, no la primera opción.
    pub const TODOS: &'static [Self] = &[
        Self::FormularioPropio,
        Self::ClientesExistentes,
        Self::EventoOFeria,
        Self::DirectorioPublico,
        Self::Otro,
    ];

    /// Cómo se guarda en la base. **Tiene que coincidir con el CHECK de la
    /// migración V5**, y hay una prueba que lo comprueba.
    #[must_use]
    pub fn como_texto(&self) -> &'static str {
        match self {
            Self::FormularioPropio => "formulario_propio",
            Self::ClientesExistentes => "clientes_existentes",
            Self::EventoOFeria => "evento_o_feria",
            Self::DirectorioPublico => "directorio_publico",
            Self::Otro => "otro",
        }
    }

    /// Clave de i18n de su nombre visible.
    #[must_use]
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::FormularioPropio => "importacion.origen.formularioPropio",
            Self::ClientesExistentes => "importacion.origen.clientesExistentes",
            Self::EventoOFeria => "importacion.origen.eventoOFeria",
            Self::DirectorioPublico => "importacion.origen.directorioPublico",
            Self::Otro => "importacion.origen.otro",
        }
    }

    /// Lo lee de vuelta desde la base.
    ///
    /// # Errores
    ///
    /// [`CoreError::OrigenDesconocido`] si la fila guardada trae un origen que
    /// no reconocemos. **No se repara en silencio**: caer en `Otro` cambiaría
    /// lo que el usuario declaró, que es justo el dato que esto existe para
    /// conservar.
    pub fn desde_texto(texto: &str) -> Result<Self, CoreError> {
        Self::TODOS
            .iter()
            .copied()
            .find(|o| o.como_texto() == texto)
            .ok_or(CoreError::OrigenDesconocido)
    }
}

/// Encabezados que se reconocen, por campo.
///
/// Es una **lista cerrada y escrita a mano**, no una heurística de parecido.
/// Una distancia de edición aceptaría «nombre» para «nombres» y también
/// «hombre» para «nombre», y la segunda es la clase de acierto que nadie pidió.
///
/// Se comparan ya normalizados: sin acentos, sin mayúsculas y sin nada que no
/// sea letra o número. Así «Teléfono», «TELEFONO» y «tele-fono» son lo mismo.
const SINONIMOS: &[(CampoImportable, &[&str])] = &[
    (
        CampoImportable::Correo,
        &[
            "correo",
            "correoelectronico",
            "correos",
            "email",
            "emails",
            "mail",
            "emailaddress",
            "direccion",
            "direccioncorreo",
            "correodecontacto",
            "correoempresa",
            "e",
        ],
    ),
    (
        CampoImportable::WhatsApp,
        &[
            "whatsapp",
            "wa",
            "movil",
            "celular",
            "cel",
            "telefonomovil",
            "telefonocelular",
            "numerowhatsapp",
            "numero",
            "movilcontacto",
        ],
    ),
    (
        CampoImportable::NombreCompleto,
        &[
            "nombrecompleto",
            "nombreyapellido",
            "nombreyapellidos",
            "nombreapellidos",
            "contacto",
            "nombredelcontacto",
            "fullname",
            "name",
        ],
    ),
    (
        CampoImportable::Nombre,
        &[
            "nombre",
            "nombres",
            "nombredepila",
            "firstname",
            "primernombre",
        ],
    ),
    (
        CampoImportable::Apellido,
        &[
            "apellido",
            "apellidos",
            "apellidopaterno",
            "lastname",
            "surname",
        ],
    ),
    (
        CampoImportable::Empresa,
        &[
            "empresa",
            "compania",
            "company",
            "organizacion",
            "razonsocial",
            "negocio",
            "cliente",
        ],
    ),
];

/// Deja un encabezado en su forma comparable.
///
/// Minúsculas, sin acentos y sólo letras y números. Es lo que hace que
/// «Teléfono móvil», «TELEFONO MOVIL» y «telefono_movil» acaben iguales.
///
/// La `ñ` **se conserva** y no se convierte en `n`: «año» y «ano» no son la
/// misma palabra, y una tabla de contactos mexicana lleva eñes.
#[must_use]
pub fn normalizar_encabezado(texto: &str) -> String {
    texto
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => Some('a'),
            'é' | 'è' | 'ë' | 'ê' => Some('e'),
            'í' | 'ì' | 'ï' | 'î' => Some('i'),
            'ó' | 'ò' | 'ö' | 'ô' => Some('o'),
            'ú' | 'ù' | 'ü' | 'û' => Some('u'),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

/// Qué campo parece ser una columna, por su encabezado.
///
/// Devuelve [`CampoImportable::Ignorar`] cuando no lo reconoce. **No se
/// arriesga**: una columna mal adivinada es peor que una columna que la persona
/// tiene que elegir, porque la primera no se ve y la segunda sí.
#[must_use]
pub fn adivinar_campo(encabezado: &str) -> CampoImportable {
    let n = normalizar_encabezado(encabezado);
    if n.is_empty() {
        return CampoImportable::Ignorar;
    }

    // Coincidencia exacta primero. El orden de `SINONIMOS` importa: el correo
    // va antes que el nombre, y «nombre completo» antes que «nombre», porque
    // «nombrecompleto» contiene «nombre» y con la búsqueda parcial de abajo
    // ganaría el segundo.
    for (campo, nombres) in SINONIMOS {
        if nombres.contains(&n.as_str()) {
            return *campo;
        }
    }

    // Y si no, que el encabezado **empiece** por uno de los sinónimos: cubre
    // «correo electronico principal» o «whatsapp del contacto» sin aceptar
    // cualquier cosa que los mencione de pasada.
    for (campo, nombres) in SINONIMOS {
        if nombres.iter().any(|s| s.len() >= 4 && n.starts_with(s)) {
            return *campo;
        }
    }

    CampoImportable::Ignorar
}

/// El mapeo propuesto para un archivo: un campo por columna.
///
/// Se devuelve completo —incluidas las columnas que se ignoran— para que la
/// pantalla pueda enseñarlas todas. Una columna que desaparece de la propuesta
/// es una columna que la persona no sabe que existe.
#[must_use]
pub fn proponer_mapeo(encabezados: &[String]) -> Vec<CampoImportable> {
    let mut propuesta: Vec<CampoImportable> = encabezados
        .iter()
        .take(MAX_COLUMNAS)
        .map(|h| adivinar_campo(h))
        .collect();

    // Un campo de identidad repetido no tiene sentido: dos columnas «Nombre»
    // no se pueden fusionar sin inventarse una regla. Se queda la primera y las
    // demás pasan a ignorarse, que es lo que la persona verá y podrá corregir.
    //
    // El correo y el WhatsApp **sí** pueden repetirse: es lo normal en una
    // tabla con «Correo 1» y «Correo 2», y el contacto admite varios canales.
    let unicos = [
        CampoImportable::Nombre,
        CampoImportable::Apellido,
        CampoImportable::NombreCompleto,
        CampoImportable::Empresa,
    ];
    for campo in unicos {
        let mut visto = false;
        for c in &mut propuesta {
            if *c != campo {
                continue;
            }
            if visto {
                *c = CampoImportable::Ignorar;
            }
            visto = true;
        }
    }

    propuesta
}

/// Por qué una fila del archivo no se pudo convertir en contacto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MotivoDeRechazo {
    /// No trae ni correo ni móvil: no hay a quién escribir.
    SinCanales,
    /// Trae más canales de los que admite un contacto.
    DemasiadosCanales,
}

impl MotivoDeRechazo {
    #[must_use]
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::SinCanales => "importacion.rechazo.sinCanales",
            Self::DemasiadosCanales => "importacion.rechazo.demasiadosCanales",
        }
    }
}

/// Convierte una fila del archivo en un borrador de contacto.
///
/// **No valida las direcciones**: eso es de [`crate::contacto::DatosDeContacto::validar`],
/// que es quien decide. Aquí sólo se reparte cada celda en su sitio.
///
/// Se rechaza antes de validar en dos casos, porque no dependen del formato de
/// ninguna dirección: la fila no trae ninguna forma de contacto, o trae más de
/// las que caben.
///
/// # Errores
///
/// [`MotivoDeRechazo`] con el motivo, para el informe de rechazados.
pub fn fila_a_borrador(
    mapeo: &[CampoImportable],
    fila: &[String],
) -> Result<BorradorDeContacto, MotivoDeRechazo> {
    let mut nombre = String::new();
    let mut apellido = String::new();
    let mut empresa = String::new();
    let mut canales: Vec<BorradorDeCanal> = Vec::new();

    for (i, campo) in mapeo.iter().enumerate() {
        // Una fila puede traer menos celdas que encabezados: las hojas de
        // cálculo recortan las celdas vacías del final. No es un error de la
        // fila, así que se trata como celda vacía y no como rechazo.
        let celda = fila.get(i).map(|c| c.trim()).unwrap_or("");
        if celda.is_empty() {
            continue;
        }

        match campo {
            CampoImportable::Nombre => nombre = celda.to_owned(),
            CampoImportable::Apellido => apellido = celda.to_owned(),
            CampoImportable::Empresa => empresa = celda.to_owned(),
            CampoImportable::NombreCompleto => {
                // Por el **primer** espacio, no por el último: en México lo
                // habitual son dos apellidos, así que «Ana María Ruiz Pérez»
                // parte mejor como «Ana» + «María Ruiz Pérez» que dejando
                // «Ana María Ruiz» de nombre. Ninguna de las dos acierta
                // siempre; ésta falla de forma más obvia, y por eso la
                // pantalla ofrece las columnas separadas cuando existen.
                let (n, a) = celda.split_once(' ').unwrap_or((celda, ""));
                nombre = n.to_owned();
                apellido = a.trim().to_owned();
            }
            CampoImportable::Correo | CampoImportable::WhatsApp => {
                let canal = if *campo == CampoImportable::Correo {
                    Canal::Correo
                } else {
                    Canal::WhatsApp
                };
                canales.push(BorradorDeCanal {
                    canal,
                    valor: celda.to_owned(),
                    // El primero de cada tipo que aparece en el archivo es el
                    // principal. Es el único criterio que el archivo ofrece: no
                    // trae ninguna marca, y el orden de las columnas es lo que
                    // quien preparó la tabla decidió poner primero.
                    principal: !canales.iter().any(|c| c.canal == canal),
                });
            }
            CampoImportable::Ignorar => {}
        }
    }

    if canales.is_empty() {
        return Err(MotivoDeRechazo::SinCanales);
    }
    if canales.len() > crate::contacto::MAX_CANALES {
        return Err(MotivoDeRechazo::DemasiadosCanales);
    }

    Ok(BorradorDeContacto {
        nombre,
        apellido,
        empresa,
        canales,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El lint del workspace prohíbe indexar, también en tests.
    fn en(v: &[CampoImportable], i: usize) -> CampoImportable {
        *v.get(i).expect("la columna existe")
    }

    /// Los cinco orígenes que Dirección confirmó el 18/09/2026.
    #[test]
    fn hay_cinco_origenes_y_otro_va_el_ultimo() {
        assert_eq!(OrigenDeLaLista::TODOS.len(), 5);
        assert_eq!(
            OrigenDeLaLista::TODOS.last(),
            Some(&OrigenDeLaLista::Otro),
            "«Otro» es la salida, no la primera opción"
        );
    }

    /// Ida y vuelta por texto. Es lo que se guarda en la base.
    #[test]
    fn el_origen_va_y_vuelve_por_texto() {
        for o in OrigenDeLaLista::TODOS {
            assert_eq!(
                OrigenDeLaLista::desde_texto(o.como_texto()),
                Ok(*o),
                "{o:?} no vuelve"
            );
        }
    }

    /// Un origen que no reconocemos **no cae en «Otro»**: eso cambiaría lo que
    /// el usuario declaró, que es el dato que esto existe para conservar.
    #[test]
    fn un_origen_desconocido_no_se_repara_en_silencio() {
        for malo in ["comprada", "", "FORMULARIO_PROPIO", "varios"] {
            assert!(
                OrigenDeLaLista::desde_texto(malo).is_err(),
                "«{malo}» se aceptó"
            );
        }
    }

    #[test]
    fn todos_los_origenes_tienen_clave_de_texto() {
        for o in OrigenDeLaLista::TODOS {
            assert!(o.clave_i18n().starts_with("importacion.origen."), "{o:?}");
        }
    }

    #[test]
    fn reconoce_los_encabezados_normales() {
        for (encabezado, esperado) in [
            ("Correo", CampoImportable::Correo),
            ("correo electrónico", CampoImportable::Correo),
            ("E-mail", CampoImportable::Correo),
            ("EMAIL", CampoImportable::Correo),
            ("WhatsApp", CampoImportable::WhatsApp),
            ("Teléfono móvil", CampoImportable::WhatsApp),
            ("Celular", CampoImportable::WhatsApp),
            ("Nombre", CampoImportable::Nombre),
            ("Apellidos", CampoImportable::Apellido),
            ("Razón social", CampoImportable::Empresa),
            ("Nombre completo", CampoImportable::NombreCompleto),
        ] {
            assert_eq!(
                adivinar_campo(encabezado),
                esperado,
                "«{encabezado}» no se reconoció"
            );
        }
    }

    /// Acentos, mayúsculas y separadores no cambian nada.
    #[test]
    fn la_forma_de_escribirlo_no_importa() {
        for variante in [
            "Teléfono Móvil",
            "TELEFONO MOVIL",
            "telefono_movil",
            " Teléfono-Móvil ",
        ] {
            assert_eq!(
                adivinar_campo(variante),
                CampoImportable::WhatsApp,
                "«{variante}»"
            );
        }
    }

    /// La `ñ` se conserva: «año» y «ano» no son la misma palabra, y una tabla
    /// mexicana lleva eñes.
    #[test]
    fn la_ene_no_se_convierte_en_n() {
        assert_eq!(normalizar_encabezado("Año de alta"), "añodealta");
        assert_ne!(normalizar_encabezado("Año"), normalizar_encabezado("Ano"));
    }

    /// Lo que no se reconoce **se ignora**, no se adivina a lo que más se
    /// parezca. Una columna mal adivinada no se ve; una ignorada sí.
    #[test]
    fn lo_desconocido_se_ignora_en_vez_de_arriesgar() {
        for raro in [
            "Notas",
            "RFC",
            "Fecha de alta",
            "Segmento",
            "",
            "   ",
            "xyz123",
        ] {
            assert_eq!(
                adivinar_campo(raro),
                CampoImportable::Ignorar,
                "«{raro}» se adivinó cuando no debía"
            );
        }
    }

    /// El caso que obliga a que «nombre completo» vaya antes que «nombre» en la
    /// lista: el segundo es prefijo del primero, y con la búsqueda por prefijo
    /// ganaría el equivocado.
    #[test]
    fn nombre_completo_gana_a_nombre() {
        assert_eq!(
            adivinar_campo("Nombre completo"),
            CampoImportable::NombreCompleto
        );
        assert_eq!(adivinar_campo("Nombre"), CampoImportable::Nombre);
    }

    /// Un encabezado que **menciona** un sinónimo de pasada no cuenta: sólo
    /// vale si empieza por él. «Sin correo» no es una columna de correos.
    #[test]
    fn mencionar_no_es_empezar_por() {
        assert_eq!(adivinar_campo("Sin correo"), CampoImportable::Ignorar);
        assert_eq!(adivinar_campo("Tiene whatsapp"), CampoImportable::Ignorar);
    }

    #[test]
    fn el_correo_y_el_whatsapp_si_se_pueden_repetir() {
        let m = proponer_mapeo(&[
            "Correo 1".into(),
            "Correo 2".into(),
            "WhatsApp".into(),
            "Celular".into(),
        ]);
        assert_eq!(en(&m, 0), CampoImportable::Correo);
        assert_eq!(en(&m, 1), CampoImportable::Correo);
        assert_eq!(en(&m, 2), CampoImportable::WhatsApp);
        assert_eq!(en(&m, 3), CampoImportable::WhatsApp);
    }

    /// Un campo de identidad repetido se queda con la primera columna. Dos
    /// «Nombre» no se pueden fusionar sin inventarse una regla.
    #[test]
    fn un_campo_de_identidad_repetido_se_queda_con_el_primero() {
        let m = proponer_mapeo(&["Nombre".into(), "Nombres".into(), "Empresa".into()]);
        assert_eq!(en(&m, 0), CampoImportable::Nombre);
        assert_eq!(en(&m, 1), CampoImportable::Ignorar);
        assert_eq!(en(&m, 2), CampoImportable::Empresa);
    }

    /// El mapeo devuelve **todas** las columnas, también las ignoradas: una
    /// columna que desaparece de la propuesta es una que nadie sabe que existe.
    #[test]
    fn la_propuesta_incluye_las_columnas_ignoradas() {
        let m = proponer_mapeo(&["Nombre".into(), "Notas".into(), "Correo".into()]);
        assert_eq!(m.len(), 3);
        assert_eq!(en(&m, 1), CampoImportable::Ignorar);
    }

    #[test]
    fn no_se_miran_mas_columnas_que_el_tope() {
        let muchas: Vec<String> = (0..MAX_COLUMNAS + 20).map(|i| format!("col{i}")).collect();
        assert_eq!(proponer_mapeo(&muchas).len(), MAX_COLUMNAS);
    }

    #[test]
    fn una_fila_normal_se_convierte() {
        let mapeo = [
            CampoImportable::Nombre,
            CampoImportable::Apellido,
            CampoImportable::Empresa,
            CampoImportable::Correo,
            CampoImportable::WhatsApp,
        ];
        let fila = [
            "Ana".to_owned(),
            "Ruiz".to_owned(),
            "Empresa SA".to_owned(),
            "ana@empresa.mx".to_owned(),
            "81 1234 5678".to_owned(),
        ];
        let b = fila_a_borrador(&mapeo, &fila).expect("fila válida");
        assert_eq!(b.nombre, "Ana");
        assert_eq!(b.apellido, "Ruiz");
        assert_eq!(b.empresa, "Empresa SA");
        assert_eq!(b.canales.len(), 2);
        assert!(b.canales.iter().all(|c| c.principal), "uno de cada tipo");
    }

    /// El nombre completo se parte por el **primer** espacio: en México lo
    /// habitual son dos apellidos.
    #[test]
    fn el_nombre_completo_se_parte_por_el_primer_espacio() {
        let mapeo = [CampoImportable::NombreCompleto, CampoImportable::Correo];
        let fila = ["Ana María Ruiz Pérez".to_owned(), "a@b.mx".to_owned()];
        let b = fila_a_borrador(&mapeo, &fila).expect("válida");
        assert_eq!(b.nombre, "Ana");
        assert_eq!(b.apellido, "María Ruiz Pérez");
    }

    #[test]
    fn un_nombre_de_una_sola_palabra_no_deja_apellido() {
        let mapeo = [CampoImportable::NombreCompleto, CampoImportable::Correo];
        let fila = ["Ana".to_owned(), "a@b.mx".to_owned()];
        let b = fila_a_borrador(&mapeo, &fila).expect("válida");
        assert_eq!(b.nombre, "Ana");
        assert_eq!(b.apellido, "");
    }

    /// Sólo el primero de cada tipo es principal. Si todos lo fueran, el núcleo
    /// tendría que elegir por su cuenta a cuál se escribe.
    #[test]
    fn el_primero_de_cada_tipo_es_el_principal() {
        let mapeo = [
            CampoImportable::Correo,
            CampoImportable::Correo,
            CampoImportable::WhatsApp,
            CampoImportable::WhatsApp,
        ];
        let fila = [
            "uno@b.mx".to_owned(),
            "dos@b.mx".to_owned(),
            "8110000001".to_owned(),
            "8110000002".to_owned(),
        ];
        let b = fila_a_borrador(&mapeo, &fila).expect("válida");
        let principales: Vec<_> = b
            .canales
            .iter()
            .filter(|c| c.principal)
            .map(|c| c.valor.as_str())
            .collect();
        assert_eq!(principales, vec!["uno@b.mx", "8110000001"]);
    }

    /// Una fila sin ninguna forma de contacto se rechaza **antes** de validar:
    /// no hay a quién escribir, y el motivo es del informe, no de un campo.
    #[test]
    fn una_fila_sin_canales_se_rechaza() {
        let mapeo = [CampoImportable::Nombre, CampoImportable::Correo];
        let fila = ["Ana".to_owned(), "   ".to_owned()];
        assert_eq!(
            fila_a_borrador(&mapeo, &fila),
            Err(MotivoDeRechazo::SinCanales)
        );
    }

    /// Una hoja de cálculo recorta las celdas vacías del final. Eso no es un
    /// error de la fila: se trata como celda vacía.
    #[test]
    fn una_fila_mas_corta_que_los_encabezados_no_es_un_error() {
        let mapeo = [
            CampoImportable::Nombre,
            CampoImportable::Correo,
            CampoImportable::WhatsApp,
        ];
        let fila = ["Ana".to_owned(), "ana@b.mx".to_owned()];
        let b = fila_a_borrador(&mapeo, &fila).expect("válida");
        assert_eq!(b.canales.len(), 1);
    }

    #[test]
    fn una_fila_con_demasiados_canales_se_rechaza() {
        let mapeo: Vec<_> = (0..crate::contacto::MAX_CANALES + 1)
            .map(|_| CampoImportable::Correo)
            .collect();
        let fila: Vec<_> = (0..crate::contacto::MAX_CANALES + 1)
            .map(|i| format!("c{i}@b.mx"))
            .collect();
        assert_eq!(
            fila_a_borrador(&mapeo, &fila),
            Err(MotivoDeRechazo::DemasiadosCanales)
        );
    }

    /// Las columnas ignoradas no aportan nada al contacto. Si aportaran, una
    /// columna de notas acabaría dentro del nombre.
    #[test]
    fn las_columnas_ignoradas_no_entran() {
        let mapeo = [
            CampoImportable::Ignorar,
            CampoImportable::Nombre,
            CampoImportable::Ignorar,
            CampoImportable::Correo,
        ];
        let fila = [
            "basura".to_owned(),
            "Ana".to_owned(),
            "más basura".to_owned(),
            "ana@b.mx".to_owned(),
        ];
        let b = fila_a_borrador(&mapeo, &fila).expect("válida");
        assert_eq!(b.nombre, "Ana");
        assert_eq!(b.empresa, "");
        assert_eq!(b.canales.len(), 1);
    }

    /// Todo campo tiene clave de i18n, y todas empiezan igual. Sin esto, un
    /// campo nuevo saldría en el desplegable con la clave cruda.
    #[test]
    fn todos_los_campos_tienen_clave_de_texto() {
        for c in CampoImportable::TODOS {
            assert!(
                c.clave_i18n().starts_with("importacion.campo."),
                "{c:?} no tiene clave"
            );
        }
        assert_eq!(CampoImportable::TODOS.len(), 7);
    }
}
