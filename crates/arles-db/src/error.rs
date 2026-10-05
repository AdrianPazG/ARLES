//! Errores de acceso a datos.

use thiserror::Error;

/// Error al abrir, migrar o consultar la base de datos.
///
/// Ningún mensaje incluye la clave maestra ni rutas completas del sistema de
/// archivos (THREAT_MODEL.md §4.1).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DbError {
    /// La clave no abre la base de datos.
    ///
    /// Puede significar dos cosas y la interfaz debe distinguirlas: que la clave
    /// del llavero no corresponda a este archivo, o que el archivo esté
    /// corrupto. En ninguno de los dos casos se debe ofrecer «continuar sin
    /// cifrado» (ADR-0011).
    #[error("no se pudo descifrar la base de datos con la clave proporcionada")]
    ClaveIncorrecta,

    /// La clave no tiene la longitud esperada.
    #[error("la clave maestra debe tener exactamente {esperado} bytes, tiene {recibido}")]
    ClaveMalFormada { esperado: usize, recibido: usize },

    #[error("fallo al migrar el esquema: {0}")]
    Migracion(String),

    /// Una fila almacenada ya no pasa la validación del dominio.
    ///
    /// Solo puede ocurrir si alguien editó la base por fuera de la aplicación o
    /// si una migración dejó datos a medias. **Se nombra en vez de repararse en
    /// silencio**: «arreglar» el dato aquí sería decidir por el usuario qué
    /// quiso escribir.
    #[error("un dato almacenado no es válido: {motivo}")]
    DatoInvalido { motivo: &'static str },

    /// Otro contacto de la misma empresa ya usa esa dirección.
    ///
    /// **Nombra la dirección en conflicto** a propósito: sin ella el usuario
    /// tendría que adivinar cuál de los hasta diez canales del formulario está
    /// repetido. Es la traducción del índice único parcial de `contact_channel`
    /// (§95: el error dice qué pasó y cómo arreglarlo).
    #[error("la dirección {direccion} ya está registrada en otro contacto ({canal})")]
    DireccionEnUso {
        canal: &'static str,
        direccion: String,
    },

    /// El contacto no existe en esta empresa, o ya fue borrado.
    ///
    /// No se distingue «nunca existió» de «era de otra empresa»: decirlo
    /// filtraría la existencia de datos ajenos.
    #[error("el contacto no existe o ya fue borrado")]
    ContactoNoExiste,

    /// No hay tal fila de supresión en esta empresa.
    #[error("esa supresión no existe")]
    SupresionNoExiste,

    /// No hay tal campaña en esta empresa, o ya no está.
    ///
    /// No se distingue «nunca existió» de «era de otra empresa», por el mismo
    /// motivo que en los contactos: decirlo filtraría la existencia de datos
    /// ajenos.
    #[error("esa campaña no existe")]
    CampanaNoExiste,

    /// No hay tal cuenta de correo.
    #[error("esa cuenta de correo no existe")]
    CuentaDeCorreoNoExiste,

    /// Se intentó activar una campaña sin nadie a quien escribir.
    ///
    /// **Es un error y no un éxito con cero.** Una campaña activada sin
    /// destinatarios se queda diciendo «enviando» para siempre, y nada explica
    /// por qué el contador no sube: ni la pantalla, ni el registro, ni el
    /// motor, que simplemente no encuentra trabajo.
    ///
    /// Pasa de verdad, y no por un descuido: una lista entera de contactos
    /// puede estar en la lista de bajas tras una importación mal hecha, y
    /// entonces lo único honesto es no activar y decirlo.
    #[error("no queda nadie a quien escribir en esta campaña")]
    AudienciaVacia,

    /// Esa acción no procede desde el estado en que está la campaña.
    ///
    /// **Lleva la clave i18n del caso concreto dentro**, no una genérica: la
    /// diferencia entre «no se puede reanudar desde ahí» y «detener no se
    /// deshace, y para volver a escribir a esa gente hay que crear una campaña
    /// nueva» es lo que decide si la persona entiende qué hacer (§95).
    #[error("esa acción no procede desde el estado actual de la campaña")]
    AccionDeCampanaImposible { clave: &'static str },

    /// Se pidió más de 50 diarios sin aceptar el aviso del §48.
    ///
    /// **No es un límite**: ARLES no bloquea el volumen (§154). Lo que no se
    /// permite es crearlo sin que conste que se avisó — y la constancia va a la
    /// bitácora, no a un campo que alguien pueda cambiar después.
    #[error("hace falta aceptar el aviso de volumen antes de crear esta campaña")]
    FaltaAceptarElVolumen,

    /// La ventana de ejecución no se abriría nunca.
    ///
    /// **Lleva el motivo dentro**, y no es adorno: sin días, con las horas al
    /// revés y con una zona inventada se arreglan en tres campos distintos del
    /// formulario, y un «la ventana no vale» a secas obliga a probar los tres
    /// (§95). La clave del texto sale del propio motivo, en el núcleo.
    #[error("esa ventana de ejecución no se abriría nunca")]
    VentanaInvalida(arles_core::ventana::VentanaInvalida),

    /// El núcleo no reconoce lo que hay guardado.
    ///
    /// Se convierte y no se envuelve en `DatoInvalido` para que la clave i18n
    /// que llega a la pantalla sea la del núcleo, que dice qué dato es.
    #[error(transparent)]
    Nucleo(#[from] arles_core::CoreError),

    /// La persona pidió la baja: no se levanta con un botón.
    ///
    /// **Lleva la clave i18n del motivo dentro** para que la pantalla pueda
    /// decir cuál fue —«pidió la baja» o «marcó el correo como no deseado»— en
    /// vez de un «no se puede» a secas. §95: el error dice qué pasó.
    #[error("esa supresión la pidió la persona y no se levanta desde aquí")]
    SupresionNoSeLevanta { motivo: &'static str },

    /// El esquema de esta instalación tiene algo que un respaldo no sabe
    /// llevar, y por eso no se escribe ninguno.
    ///
    /// Son tres casos, y los tres son **un defecto nuestro**, no del usuario: un
    /// dato binario en una tabla del respaldo —el formato es texto y guardarlo
    /// en hexadecimal devolvería otro dato—, tablas que dependen unas de otras
    /// en círculo —entonces no hay orden en que insertarlas— y un manifiesto
    /// que clasifica una tabla que la base no tiene.
    ///
    /// **Se para en vez de exportar lo que se pueda.** Un respaldo incompleto
    /// se descubre el día que hace falta, y ese día ya no hay de dónde sacarlo.
    #[error("el esquema de esta base tiene algo que un respaldo no puede llevar: {motivo}")]
    EsquemaNoSeDejaRespaldar { motivo: &'static str },

    #[error("error de SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

impl DbError {
    /// Clave estable para que la interfaz traduzca el error con i18n (§139).
    pub fn clave_i18n(&self) -> &'static str {
        match self {
            Self::ClaveIncorrecta => "error.db.clave_incorrecta",
            Self::ClaveMalFormada { .. } => "error.db.clave_mal_formada",
            Self::Migracion(_) => "error.db.migracion",
            Self::DatoInvalido { .. } => "error.db.dato_invalido",
            Self::DireccionEnUso { .. } => "error.db.direccion_en_uso",
            Self::ContactoNoExiste => "error.db.contacto_no_existe",
            Self::SupresionNoExiste => "error.db.supresion_no_existe",
            Self::CampanaNoExiste => "error.db.campana_no_existe",
            Self::AudienciaVacia => "error.db.audiencia_vacia",
            Self::CuentaDeCorreoNoExiste => "error.db.cuenta_de_correo_no_existe",
            Self::AccionDeCampanaImposible { clave } => clave,
            Self::FaltaAceptarElVolumen => "error.db.falta_aceptar_el_volumen",
            Self::VentanaInvalida(motivo) => motivo.clave_i18n(),
            Self::Nucleo(e) => e.clave_i18n(),
            Self::SupresionNoSeLevanta { .. } => "error.db.supresion_no_se_levanta",
            Self::EsquemaNoSeDejaRespaldar { .. } => "error.db.esquema_no_se_deja_respaldar",
            Self::Sqlite(_) => "error.db.sqlite",
        }
    }
}
