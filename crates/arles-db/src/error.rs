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

    /// Se intentó activar una campaña que **no termina nunca**.
    ///
    /// ─────────────────────────────────────────────────────────────────────
    /// EL NÚCLEO YA LO SABÍA, Y NADIE SE LO PREGUNTABA
    ///
    /// `arles_core::duracion::estimar` devuelve `NoCabeEnElHorizonte { faltan }`
    /// desde la Fase 4, y su texto está escrito desde entonces. Hasta la
    /// 8.6 E (07/10/2026) **`activar_campana` no consultaba la estimación**:
    /// se podía activar una campaña de 451 196 personas con 50 correos al día,
    /// y la pantalla decía «1 303 días» sin decir que **386 046 personas no
    /// recibirían un mensaje nunca**.
    ///
    /// Lo que lo hace grave es que el fallo es silencioso y largo: el motor
    /// envía correctamente durante años, el contador sube, y nada dice que no
    /// va a llegar al final. No hay ningún momento en que algo falle.
    ///
    /// `faltan` es la cifra (§94). Sin ella el mensaje sería «no termina», que
    /// no dice si sobra gente o sobra muchísima.
    #[error("esta campaña no termina en cinco años: faltarían {faltan} mensajes")]
    LaCampanaNoTermina { faltan: u32 },

    /// Se intentó activar una campaña con la que **no sale ni un mensaje**.
    ///
    /// Pasa con un tope a cero, o con una ventana que no contiene ninguna hora
    /// de ningún día permitido. El comentario del texto que lo acompaña lo
    /// llama, desde la Fase 4, «el caso que hay que cazar antes de activar» —
    /// y no se cazaba.
    ///
    /// Es peor que un error al activar: una campaña «enviando» que nunca manda
    /// nada es exactamente la razón número uno por la que alguien cree que
    /// ARLES se rompió (7.3).
    #[error("con estos topes no saldría ni un mensaje de esta campaña")]
    LaCampanaNoAvanza,

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

    /// Se pidió borrar una plantilla que alguna campaña usó.
    ///
    /// ── NO SE BORRA, Y NO ES UNA LIMITACIÓN TÉCNICA ──────────────────────
    ///
    /// `campaign_stage.template_version_id` es `ON DELETE RESTRICT`, así que
    /// el borrado fallaría igualmente — pero con un error de clave foránea,
    /// que para quien pulsa es «algo falló». Esto lo dice **antes de
    /// intentarlo** y con su cifra.
    ///
    /// Y la regla de fondo: una campaña apunta a la versión exacta que envió.
    /// Borrarla deja el registro de envíos apuntando a un texto que ya no
    /// existe, y la pregunta «¿qué decía el correo que recibió este cliente?»
    /// se queda sin respuesta — que es la mitad de lo que ARLES promete.
    #[error("esa plantilla la usan {campanas} campaña(s)")]
    LaPlantillaEstaEnUso { campanas: u32 },

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

    /// El respaldo no encaja con el esquema de esta base, así que **no se
    /// restaura**.
    ///
    /// Pasa cuando el respaldo es de otra versión del esquema, cuando declara
    /// una tabla o una columna que esta base no tiene, o cuando la cabecera
    /// promete unas filas y el cuerpo trae otras.
    ///
    /// **Se para en vez de restaurar lo que encaje.** Media base restaurada es
    /// peor que ninguna: la aplicación arranca, parece funcionar, y lo que
    /// falta se descubre semanas después sin saber desde cuándo.
    #[error("este respaldo no encaja con esta versión de ARLES: {motivo}")]
    RespaldoNoEncaja { motivo: String },

    /// El archivo elegido es un **archivo de historial**, no un respaldo, y por
    /// eso no se restaura (D-8D, entrega 8.3 bis).
    ///
    /// ── POR QUÉ ES UN ERROR APARTE Y NO UN `RespaldoNoEncaja` ──
    ///
    /// Los dos impiden restaurar, pero mandan a hacer cosas distintas, que es
    /// el mismo motivo por el que la 8.1 partió su error en tres. «No encaja»
    /// dice *busca otra copia o actualiza ARLES*. Esto dice *el archivo está
    /// bien y es de ARLES; lo que pasa es que un archivo de historial no sirve
    /// para recuperar la base, sirve para consultar envíos viejos*.
    ///
    /// Y lo que está en juego si se confundieran: restaurar un archivo de
    /// historial reemplazaría la base entera por un año de envíos — los
    /// contactos, las bajas y las campañas se irían. Mandar a alguien a
    /// «probar otra copia» ante este error sería mandarlo a repetir el intento.
    #[error("esto es un archivo de historial, no un respaldo: no se puede restaurar")]
    EsoEsUnArchivoDeHistorial,

    /// El historial de la base cambió entre archivarlo y borrarlo, así que
    /// **no se borra nada** (segunda condición de D-8D).
    ///
    /// El motor escribe en su propio hilo (§51): entre que se escribió el
    /// archivo y que alguien confirmó el borrado pueden haber entrado intentos
    /// nuevos con fecha anterior al corte —un envío que se quedó en cola—.
    /// Borrar «lo que haya ahora» se llevaría filas que el archivo no tiene.
    #[error(
        "el historial cambió desde que se escribió el archivo: había {archivadas} filas y ahora hay {ahora}"
    )]
    ElHistorialCambio { archivadas: u64, ahora: u64 },

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
            Self::LaCampanaNoTermina { .. } => "error.db.la_campana_no_termina",
            Self::LaCampanaNoAvanza => "error.db.la_campana_no_avanza",
            Self::CuentaDeCorreoNoExiste => "error.db.cuenta_de_correo_no_existe",
            Self::AccionDeCampanaImposible { clave } => clave,
            Self::FaltaAceptarElVolumen => "error.db.falta_aceptar_el_volumen",
            Self::LaPlantillaEstaEnUso { .. } => "error.db.la_plantilla_esta_en_uso",
            Self::VentanaInvalida(motivo) => motivo.clave_i18n(),
            Self::Nucleo(e) => e.clave_i18n(),
            Self::SupresionNoSeLevanta { .. } => "error.db.supresion_no_se_levanta",
            Self::EsquemaNoSeDejaRespaldar { .. } => "error.db.esquema_no_se_deja_respaldar",
            Self::RespaldoNoEncaja { .. } => "error.db.respaldo_no_encaja",
            Self::EsoEsUnArchivoDeHistorial => "error.db.eso_es_un_archivo_de_historial",
            Self::ElHistorialCambio { .. } => "error.db.el_historial_cambio",
            Self::Sqlite(_) => "error.db.sqlite",
        }
    }
}
