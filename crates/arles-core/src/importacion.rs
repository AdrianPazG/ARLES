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
    /// `whatsapp` en minúscula al cruzar la frontera, y **no `whatsApp`**.
    ///
    /// Sin el `rename`, serde produce `whatsApp` —trata `WhatsApp` como dos
    /// palabras— y la interfaz manda `whatsapp`, que es como se escribe el
    /// canal en el resto del sistema (`Canal::como_texto`). Dos grafías para lo
    /// mismo, y la importación falla al cruzar: **después** de que el usuario
    /// haya revisado los choques y aceptado la declaración.
    ///
    /// Lo cazó el validador comparando las dos listas. Nada más lo habría
    /// visto: el núcleo simulado de la vista previa es JavaScript y no
    /// comprueba tipos, y las pruebas de Rust usan el enum, no su JSON.
    #[serde(rename = "whatsapp")]
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

/// Las columnas de la plantilla que ARLES entrega para rellenar.
///
/// ─────────────────────────────────────────────────────────────────────────
/// PEDIDA POR DIRECCIÓN EL 22/10/2026, Y CON UNA CONDICIÓN QUE NO ES OBVIA
///
/// El caso que la justifica: la tabla la trae el cliente, con sus columnas y
/// en su orden. La plantilla es dónde pegar las que sirven.
///
/// Tres columnas y no siete: Dirección pidió **Nombre, Correo y Teléfono**, y
/// añadir «Apellido» o «Empresa» a una plantilla que existe para no perder a
/// nadie delante de una hoja ajena sería devolver el problema. El resto de los
/// campos siguen reconociéndose si la tabla los trae — esto es el suelo, no el
/// techo.
///
/// **La condición: cada encabezado de aquí tiene que volver a entrar.** Una
/// plantilla con una columna que [`adivinar_campo`] ignora es peor que no
/// tenerla: quien la rellena ve su dato escrito, importa, y ese dato no está —
/// sin un solo error. Lo vigila `la_plantilla_que_arles_da_entra_en_arles`,
/// y ya cazó una: **«Teléfono» a secas no se reconocía** (medido el
/// 22/10/2026), así que la tabla real de Dirección entraba sin un teléfono.
/// ─────────────────────────────────────────────────────────────────────────
pub const PLANTILLA_DE_CONTACTOS: &[(CampoImportable, &str)] = &[
    (CampoImportable::Nombre, "Nombre"),
    (CampoImportable::Correo, "Correo"),
    (CampoImportable::WhatsApp, "Teléfono"),
];

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

/// Huella criptográfica de un texto, en hexadecimal y con su algoritmo delante.
///
/// ─────────────────────────────────────────────────────────────────────────
/// PARA QUÉ SIRVE, Y PARA QUÉ NO
///
/// Sirve para **demostrar que un texto no ha cambiado**. Al importar se guarda
/// el texto de la afirmación de origen íntegro *y* su huella. Si la redacción
/// cambia en 2027, el texto viejo sigue ahí, pero sin huella nadie puede
/// distinguir «así estaba» de «así lo dejamos después» (ADR-0013 §1).
///
/// **No sirve para guardar secretos.** SHA-256 no es una función de derivación
/// de contraseñas: es rápida a propósito, que es justo lo que no se quiere ahí.
/// Ninguna credencial pasa por aquí — van al llavero del sistema (ADR-0011).
///
/// Lleva el prefijo `sha256:` porque un hexadecimal suelto no dice con qué se
/// calculó. El día que haya que cambiar de algoritmo, las huellas viejas siguen
/// siendo legibles y se sabe cuáles son cuáles.
/// ─────────────────────────────────────────────────────────────────────────
#[must_use]
pub fn huella(texto: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(texto.as_bytes());
    format!("sha256:{:x}", h.finalize())
}

/// Huella de un archivo entero, por sus bytes.
///
/// Es lo que permite decir «este archivo exacto ya se importó el 3 de marzo».
/// Se calcula sobre los bytes y no sobre el texto: un CSV en Windows-1252 y el
/// mismo en UTF-8 dicen lo mismo pero **no son el mismo archivo**, y para
/// responder a «¿qué se importó?» importa el archivo, no su lectura.
#[must_use]
pub fn huella_de_bytes(bytes: &[u8]) -> String {
    let mut h = HuellaEnCurso::nueva();
    h.añadir(bytes);
    h.cerrar()
}

/// La misma huella, calculada a trozos.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ HACE FALTA
///
/// [`huella_de_bytes`] necesita el archivo entero en memoria. Para un archivo
/// de importación —hasta 200 MB— eso significa reservar 200 MB **sólo para
/// calcular una huella**, encima de lo que ya cuesta leer la tabla.
///
/// Con esto, quien lee el archivo en trozos de 64 KB le va dando cada trozo y
/// nunca tiene más de un trozo en memoria. El resultado es idéntico —hay una
/// prueba que lo comprueba contra [`huella_de_bytes`]—, así que las huellas ya
/// guardadas siguen valiendo.
/// ─────────────────────────────────────────────────────────────────────────
pub struct HuellaEnCurso(sha2::Sha256);

impl core::fmt::Debug for HuellaEnCurso {
    /// No enseña el estado interno: no dice nada útil y sólo sirve para que
    /// medio archivo acabe en una bitácora si alguien depura con `{:?}`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("HuellaEnCurso(..)")
    }
}

impl HuellaEnCurso {
    #[must_use]
    pub fn nueva() -> Self {
        use sha2::Digest;
        Self(sha2::Sha256::new())
    }

    pub fn añadir(&mut self, bytes: &[u8]) {
        use sha2::Digest;
        self.0.update(bytes);
    }

    #[must_use]
    pub fn cerrar(self) -> String {
        use sha2::Digest;
        format!("sha256:{:x}", self.0.finalize())
    }
}

impl Default for HuellaEnCurso {
    fn default() -> Self {
        Self::nueva()
    }
}

/// Encabezados de teléfono que **no dicen si ese número recibe WhatsApp**.
///
/// ─────────────────────────────────────────────────────────────────────────
/// POR QUÉ ESTA LISTA EXISTE APARTE, Y NO DENTRO DE LOS SINÓNIMOS
///
/// Porque sirve para dos cosas, y separarlas en dos listas sería dejarlas
/// divergir: **se reconocen como WhatsApp** —una columna «Teléfono» ignorada
/// pierde el dato sin decirlo— **y se avisan en la pantalla**, porque un fijo
/// mandado a WhatsApp se paga en calidad del número ante Meta, que es de lo poco
/// que no se puede deshacer.
///
/// La cabecera de este módulo explica por qué el mapeo se adivina pero no se
/// impone; esto es el caso que más lo necesita. Hasta el 22/10/2026 el módulo
/// resolvía la ambigüedad **ignorando la columna**, y eso no era prudente: era
/// silencioso. La tabla real de Dirección —nueve columnas, una de ellas
/// «Teléfono»— entraba con ocho reconocidas y sin un solo número, sin que nada
/// lo dijera. Perder el dato callando es peor que acertar con un aviso.
/// ─────────────────────────────────────────────────────────────────────────
/// Encabezados que **sí** dicen que el número es un móvil.
///
/// Está aparte de [`TELEFONO_SIN_DECIR_CUAL`] porque la pregunta del aviso es
/// «¿se reconoció por uno de éstos?», no «¿se parece a un genérico?».
///
/// La diferencia no es cosmética, y lo cazó una prueba antes de entrar: la
/// primera versión del aviso miraba sólo los genéricos y «Teléfono móvil»
/// **empieza por** «telefono», así que avisaba de una columna que dice
/// exactamente de qué número habla. Un aviso que salta donde no hace falta
/// enseña a ignorar los que sí.
const SI_ES_MOVIL: &[&str] = &[
    "whatsapp",
    "wa",
    "movil",
    "celular",
    "celulares",
    "cel",
    "telefonomovil",
    "telefonocelular",
    "numerowhatsapp",
    "movilcontacto",
];

const TELEFONO_SIN_DECIR_CUAL: &[&str] = &[
    "telefono",
    "telefonos",
    "tel",
    "numerodetelefono",
    "numero",
    "numeros",
    "phone",
    "telephone",
];

/// Encabezados que dicen a las claras que el número **no** es un móvil.
///
/// Se comprueba **antes** que los sinónimos, porque «telefonofijo» empieza por
/// «telefono» y si no ganaría la lista de arriba. Van a [`CampoImportable::Ignorar`]:
/// un conmutador no recibe WhatsApp y nadie quiere que lo intente.
const NO_RECIBE_WHATSAPP: &[&str] = &[
    "telefonofijo",
    "fijo",
    "telefonooficina",
    "telefonodeoficina",
    "oficina",
    "conmutador",
    "lada",
    "extension",
    "ext",
    "fax",
];

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
    (CampoImportable::WhatsApp, SI_ES_MOVIL),
    // Los genéricos van **después** de los que dicen «móvil»: los dos acaban en
    // WhatsApp, pero así el que se reconoce por su nombre exacto nunca depende
    // del prefijo de otro.
    (CampoImportable::WhatsApp, TELEFONO_SIN_DECIR_CUAL),
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

    // Lo que dice no ser un móvil se ignora antes de mirar nada más:
    // «telefonofijo» empieza por «telefono» y la búsqueda por prefijo de abajo
    // lo daría por WhatsApp.
    if NO_RECIBE_WHATSAPP.iter().any(|s| n.starts_with(s)) {
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

/// Si un encabezado normalizado coincide con alguno de **una** lista: exacto, o
/// empezando por un sinónimo de cuatro letras o más.
///
/// No es toda la regla de [`adivinar_campo`], y conviene no creerlo: aquélla
/// hace **dos pasadas** sobre todas las listas —primero los exactos de todas,
/// luego los prefijos— porque el orden entre campos decide los empates
/// («nombrecompleto» contra «nombre»). Aquí no hay empate que resolver: la
/// pregunta es si este encabezado está en esta lista.
fn casa_con(n: &str, lista: &[&str]) -> bool {
    lista
        .iter()
        .any(|s| n == *s || (s.len() >= 4 && n.starts_with(s)))
}

/// Si una columna se va a mandar por WhatsApp **sin que su encabezado lo diga**.
///
/// Es lo que la pantalla necesita para avisar. Devuelve `true` cuando el campo
/// adivinado es WhatsApp y el encabezado **no** es uno de los que dicen que el
/// número es un móvil ([`SI_ES_MOVIL`]). Una columna que dice «Celular» o
/// «WhatsApp» no se avisa — avisar de todo es no avisar de nada.
#[must_use]
pub fn el_telefono_no_dice_si_es_movil(encabezado: &str) -> bool {
    if adivinar_campo(encabezado) != CampoImportable::WhatsApp {
        return false;
    }
    !casa_con(&normalizar_encabezado(encabezado), SI_ES_MOVIL)
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

    /// La huella lleva su algoritmo delante: un hexadecimal suelto no dice con
    /// qué se calculó, y el día que haya que cambiarlo no se sabría cuál es cuál.
    #[test]
    fn la_huella_dice_con_que_se_calculo() {
        let h = huella("Declaro que esta lista tiene origen lícito.");
        assert!(h.starts_with("sha256:"), "{h}");
        // 7 del prefijo + 64 del hexadecimal.
        assert_eq!(h.len(), 71, "{h}");
    }

    /// El mismo texto da la misma huella; uno distinto, otra. Es lo único que
    /// se le pide, y es lo que permite demostrar que no se tocó.
    #[test]
    fn la_huella_cambia_si_el_texto_cambia() {
        let a = huella("Declaro que esta lista tiene origen lícito.");
        assert_eq!(a, huella("Declaro que esta lista tiene origen lícito."));

        // Un punto de diferencia basta.
        assert_ne!(a, huella("Declaro que esta lista tiene origen lícito"));
        // Y un espacio invisible al final también, que es el caso que se cuela.
        assert_ne!(a, huella("Declaro que esta lista tiene origen lícito. "));
    }

    /// La huella del archivo va por **bytes**, no por texto. Un CSV en
    /// Windows-1252 y el mismo en UTF-8 dicen lo mismo pero no son el mismo
    /// archivo, y para responder a «¿qué se importó?» importa el archivo.
    #[test]
    fn la_huella_del_archivo_distingue_la_codificacion() {
        let utf8 = "Nombre,Compañía\n".as_bytes().to_vec();
        let mut latino = b"Nombre,Compa".to_vec();
        latino.push(0xF1); // ñ en Windows-1252
        latino.extend_from_slice(b"\xEDa\n");

        assert_ne!(
            huella_de_bytes(&utf8),
            huella_de_bytes(&latino),
            "dos archivos distintos dieron la misma huella"
        );
    }

    /// **La plantilla que ARLES entrega tiene que entrar en ARLES.**
    ///
    /// Es el patrón del §9 —dos piezas correctas y el cable sin conectar—
    /// aplicado al único sitio donde el producto escribe un archivo que después
    /// él mismo lee. Si una columna de la plantilla cae en `Ignorar`, quien la
    /// rellena ve su dato en la hoja, importa, y el dato no está: ningún error,
    /// ninguna cifra en rojo, nada.
    ///
    /// **Se escribió antes del arreglo y se vio en rojo**: «Teléfono» a secas no
    /// estaba en ningún sinónimo —sólo «teléfono móvil» y «teléfono celular»—,
    /// así que la plantilla habría salido con una columna muerta. El mensaje fue
    /// «la columna «Teléfono» de la plantilla se ignora al importar».
    #[test]
    fn la_plantilla_que_arles_da_entra_en_arles() {
        for (campo, encabezado) in PLANTILLA_DE_CONTACTOS {
            assert_eq!(
                adivinar_campo(encabezado),
                *campo,
                "la columna «{encabezado}» de la plantilla no vuelve a entrar como {campo:?}"
            );
        }

        // Y el mapeo completo de la plantilla no ignora ninguna columna: lo
        // anterior comprueba cada encabezado por su cuenta, y `proponer_mapeo`
        // además descarta los campos de identidad repetidos. Una plantilla con
        // dos columnas que adivinan lo mismo pasaría la primera mitad.
        let encabezados: Vec<String> = PLANTILLA_DE_CONTACTOS
            .iter()
            .map(|(_, h)| (*h).to_owned())
            .collect();
        let propuesta = proponer_mapeo(&encabezados);
        assert!(
            !propuesta.contains(&CampoImportable::Ignorar),
            "el mapeo de la plantilla ignora alguna columna: {propuesta:?}"
        );
    }

    /// Las tres columnas que Dirección pidió el 22/10/2026, por su nombre.
    ///
    /// Escrita a mano a propósito: derivarla de la constante haría que la prueba
    /// se moviera con el cambio, y lo que se está fijando es un acuerdo con
    /// Dirección, no una propiedad del código.
    #[test]
    fn la_plantilla_lleva_las_tres_columnas_acordadas() {
        let nombres: Vec<&str> = PLANTILLA_DE_CONTACTOS.iter().map(|(_, h)| *h).collect();
        assert_eq!(nombres, vec!["Nombre", "Correo", "Teléfono"]);
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

    /// **El defecto que encontró Dirección el 22/10/2026.** Su tabla real traía
    /// nueve columnas y ARLES reconocía ocho: «Teléfono» caía en `Ignorar`, así
    /// que los contactos entraban sin un solo número y nada lo decía.
    ///
    /// La prueba que existía probaba «Teléfono móvil», que es el caso que no
    /// falla. Es el patrón de este proyecto: la comprobación cubría la parte que
    /// nadie había roto.
    #[test]
    fn un_telefono_a_secas_es_un_telefono() {
        for h in [
            "Teléfono",
            "TELEFONO",
            "Telefono",
            "teléfonos",
            "Tel",
            "Teléfono de contacto",
            "Número",
            "Phone",
            "Telephone",
        ] {
            assert_eq!(
                adivinar_campo(h),
                CampoImportable::WhatsApp,
                "«{h}» se ignoró, y con ella el número de cada contacto"
            );
        }
    }

    /// Y lo que dice a las claras **no** ser un móvil se sigue ignorando: un
    /// conmutador mandado a WhatsApp se paga en calidad del número ante Meta.
    ///
    /// Van por prefijo y antes que los sinónimos a propósito: «telefonofijo»
    /// empieza por «telefono». Verificada quitando la comprobación de
    /// [`NO_RECIBE_WHATSAPP`] de `adivinar_campo`: «Teléfono fijo» pasó a
    /// `WhatsApp` y esta prueba cayó nombrándolo.
    #[test]
    fn un_fijo_declarado_no_se_manda_por_whatsapp() {
        for h in [
            "Teléfono fijo",
            "Fijo",
            "Teléfono de oficina",
            "Conmutador",
            "Lada",
            "Extensión",
            "Fax",
        ] {
            assert_eq!(
                adivinar_campo(h),
                CampoImportable::Ignorar,
                "«{h}» se dio por móvil"
            );
        }
    }

    /// El aviso se da **sólo** cuando el encabezado no dice de qué número habla.
    /// Avisar de una columna que ya dice «WhatsApp» es ruido, y el ruido enseña
    /// a ignorar los avisos de verdad.
    #[test]
    fn solo_se_avisa_del_telefono_que_no_se_sabe_que_es() {
        for h in ["Teléfono", "Tel", "Número", "teléfonos", "Phone"] {
            assert!(el_telefono_no_dice_si_es_movil(h), "«{h}» no avisó");
        }
        for h in [
            "WhatsApp",
            "Celular",
            "Teléfono móvil",
            "Móvil",
            // Ni de lo que no es un teléfono, ni de lo que se ignora.
            "Correo",
            "Nombre",
            "Teléfono fijo",
            "Notas",
        ] {
            assert!(
                !el_telefono_no_dice_si_es_movil(h),
                "«{h}» avisó sin hacer falta"
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

#[cfg(test)]
mod prueba_de_serializacion {
    use super::*;

    /// **Cómo viaja cada campo por la IPC, fijado uno a uno.**
    ///
    /// No es decorativo: la interfaz manda estas cadenas exactas y Rust las
    /// lee. Si no coinciden, la importación falla al cruzar la frontera —
    /// **después** de que el usuario haya revisado los choques y aceptado la
    /// declaración de origen.
    ///
    /// Se escriben a mano, no se derivan: derivarlas de la misma regla que
    /// produce el JSON haría que la prueba se moviera con el fallo. El caso que
    /// lo justifica ya ocurrió — `WhatsApp` salía como `whatsApp`.
    #[test]
    fn cada_campo_viaja_con_su_nombre_exacto() {
        for (campo, esperado) in [
            (CampoImportable::Nombre, "\"nombre\""),
            (CampoImportable::Apellido, "\"apellido\""),
            (CampoImportable::NombreCompleto, "\"nombreCompleto\""),
            (CampoImportable::Empresa, "\"empresa\""),
            (CampoImportable::Correo, "\"correo\""),
            (CampoImportable::WhatsApp, "\"whatsapp\""),
            (CampoImportable::Ignorar, "\"ignorar\""),
        ] {
            assert_eq!(
                serde_json::to_string(&campo).expect("serializa"),
                esperado,
                "{campo:?} viaja con otro nombre del que espera la interfaz"
            );
        }
    }

    /// Y vuelve: lo que la interfaz manda, Rust lo entiende.
    #[test]
    fn lo_que_manda_la_interfaz_se_entiende() {
        for texto in [
            "nombre",
            "apellido",
            "nombreCompleto",
            "empresa",
            "correo",
            "whatsapp",
            "ignorar",
        ] {
            let json = format!("\"{texto}\"");
            assert!(
                serde_json::from_str::<CampoImportable>(&json).is_ok(),
                "la interfaz manda «{texto}» y el núcleo no sabe leerlo"
            );
        }
    }

    /// El canal de la importación y el del resto del sistema se escriben
    /// **igual**. Dos grafías para lo mismo es el fallo que esto evita.
    #[test]
    fn el_canal_se_escribe_igual_en_todo_el_sistema() {
        assert_eq!(
            serde_json::to_string(&CampoImportable::WhatsApp).expect("serializa"),
            format!("\"{}\"", crate::Canal::WhatsApp.como_texto())
        );
    }

    /// La huella a trozos tiene que dar **exactamente** la misma que la de
    /// golpe, o las huellas ya guardadas dejan de valer y «este archivo ya se
    /// importó» empieza a mentir.
    ///
    /// Se prueba con trozos de tamaños que no dividen el total y que caen justo
    /// en los bordes: un trozo vacío, uno de un byte, y uno más grande que todo
    /// lo que queda. Un troceado que sólo se pruebe con trozos que encajan
    /// redondos no prueba el troceado, prueba el caso fácil.
    #[test]
    fn la_huella_a_trozos_es_la_misma_que_la_de_golpe() {
        let datos: Vec<u8> = (0..=255_u8).cycle().take(5_000).collect();
        let de_golpe = huella_de_bytes(&datos);

        for corte in [1_usize, 7, 64, 999, 5_000, 9_999] {
            let mut h = HuellaEnCurso::nueva();
            h.añadir(&[]); // un trozo vacío no puede cambiar nada
            for pedazo in datos.chunks(corte) {
                h.añadir(pedazo);
            }
            assert_eq!(
                h.cerrar(),
                de_golpe,
                "con trozos de {corte} bytes sale otra huella"
            );
        }
    }

    /// Y una huella vacía sigue siendo una huella: no es cadena vacía ni panica.
    #[test]
    fn una_huella_sin_datos_tiene_forma_de_huella() {
        let vacia = HuellaEnCurso::nueva().cerrar();
        assert_eq!(vacia, huella_de_bytes(&[]));
        assert!(vacia.starts_with("sha256:"), "{vacia}");
    }
}
