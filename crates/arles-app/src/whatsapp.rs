//! Los comandos del número de WhatsApp.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! LAS DOS MARCAS NO SE PUEDEN PONER AL DAR DE ALTA, Y ÉSE ES EL PUNTO
//!
//! Un número nace sin Coexistencia confirmada y sin riesgo aceptado, y por eso
//! **no puede enviar todavía**. Las dos marcas se ponen con dos comandos
//! aparte, cada uno con su propia pulsación.
//!
//! Si el alta las aceptara como dos casillas del mismo formulario, se marcarían
//! con el resto de los campos sin leerlas — que es exactamente lo que estas dos
//! marcas existen para impedir. Son la constancia de que a alguien se le avisó:
//! una que el número saldrá de la app de WhatsApp Business (L-11), y otra que
//! escribir a quien no lo pidió puede costar el número entero.
//!
//! Que cuesten un gesto más es el diseño, no un descuido.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! EL TOKEN ENTRA Y NO SALE
//!
//! Igual que la contraseña de correo. Entra por `guardar_numero_de_whatsapp`,
//! va al llavero, y en la base queda una referencia opaca. Ningún comando de
//! este módulo lo devuelve, y hay una prueba que serializa la salida y lo
//! busca en el JSON — porque lo que importa es lo que cruza la frontera, no lo
//! que creemos haber declarado.

use serde::Serialize;

use crate::comandos::empresa_actual;
use crate::error::ErrorIpc;
use crate::estado::EstadoApp;

/// Un número tal como lo ve la pantalla. **Sin token.**
///
/// Lleva `puede_enviar` ya resuelto en vez de dejar que la pantalla lo deduzca
/// de los tres campos. La regla —conectado, con Coexistencia, y con el riesgo
/// aceptado si escribe en frío— vive en el núcleo y en `NumeroGuardado`; que la
/// webview la reimplemente significa dos reglas que se separan en cuanto una de
/// las dos cambie, y la que decide si se envía no sería la que se ve.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NumeroIpc {
    pub id: String,
    pub nombre_visible: String,
    /// El número en E.164. No es un secreto: es la dirección desde la que se
    /// escribe, y quien recibe el mensaje la ve.
    pub telefono_e164: String,
    /// El identificador de la cuenta de empresa en Meta. Tampoco es secreto —
    /// Meta lo enseña en su propio panel—, y hace falta en la pantalla para
    /// que alguien pueda comprobar que dio de alta el número correcto.
    pub waba_id: String,
    pub modo: String,
    pub estado: String,
    /// Cuándo se confirmó Coexistencia. `null` = no se ha confirmado.
    pub coexistencia_confirmada: Option<String>,
    /// Cuándo se aceptó el riesgo del modo en frío. `null` = no se ha aceptado.
    pub riesgo_aceptado: Option<String>,
    /// Lo que Meta dice de la calidad, o `null` si no consta.
    ///
    /// **`null` es «no consta», no «está bien».** La pantalla tiene que poder
    /// distinguirlo: un semáforo en verde porque no ha podido mirar es peor que
    /// no tener semáforo.
    pub calidad: Option<String>,
    /// El tope diario que impone Meta. Lo decide Meta, no ARLES.
    pub limite_de_meta: Option<u32>,
    /// ¿Puede mandar una campaña hoy?
    pub puede_enviar: bool,
    /// ¿Este modo escribe a quien no lo pidió?
    pub escribe_en_frio: bool,
}

impl From<arles_db::numero::NumeroGuardado> for NumeroIpc {
    fn from(n: arles_db::numero::NumeroGuardado) -> Self {
        Self {
            puede_enviar: n.puede_enviar(),
            escribe_en_frio: n.modo.escribe_en_frio(),
            id: n.id.to_string(),
            nombre_visible: n.nombre_visible,
            telefono_e164: n.telefono_e164,
            waba_id: n.waba_id,
            modo: n.modo.como_se_guarda().to_owned(),
            estado: n.estado,
            coexistencia_confirmada: n.coexistencia_confirmada,
            riesgo_aceptado: n.riesgo_aceptado,
            calidad: n.calidad,
            limite_de_meta: n.limite_de_meta,
            // `referencia_de_credencial` no se copia. Es lo único del registro
            // que no cruza, y no cruza porque no le sirve a la pantalla para
            // nada: no es el token, pero dice dónde está.
        }
    }
}

/// Los números dados de alta.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa configurada o si la lectura falla.
#[tauri::command]
pub fn numeros_de_whatsapp(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<Vec<NumeroIpc>, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    Ok(estado
        .db()
        .numeros(empresa)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?
        .into_iter()
        .map(NumeroIpc::from)
        .collect())
}

/// Da de alta un número de WhatsApp.
///
/// El orden es el mismo que el de una cuenta de correo, y por la misma razón:
/// **el secreto primero, la base después, y si la base falla el secreto se
/// retira**. Al revés quedaría una fila apuntando a un llavero vacío, que es
/// una cuenta que no puede enviar y que nadie sabe por qué.
///
/// # Errores
///
/// [`ErrorIpc`] si el número no es un E.164 válido, si el modo no se reconoce,
/// si el llavero no está disponible o si la escritura falla.
#[tauri::command]
pub fn guardar_numero_de_whatsapp(
    estado: tauri::State<'_, EstadoApp>,
    nombre_visible: String,
    telefono: String,
    waba_id: String,
    phone_number_id: String,
    modo: String,
    token: String,
) -> Result<Vec<NumeroIpc>, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;

    // El número se valida en el núcleo, no aquí. Un E.164 mal formado guardado
    // ahora es un envío rechazado por Meta dentro de tres semanas. Se completa
    // con el país de la empresa, igual que los teléfonos de los contactos: quien
    // da de alta su propio número lo escribe como lo tiene en el móvil.
    let pais = crate::comandos::pais_de_la_empresa(&estado)?;
    let telefono = arles_core::canal::PhoneNumber::parse(&telefono, &pais)
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    let modo = arles_core::whatsapp::ModoDelCanal::de_guardado(&modo)
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    if token.trim().is_empty() {
        return Err(ErrorIpc::from(crate::AppError::Db(
            arles_db::DbError::DatoInvalido {
                motivo: "el token de Meta no puede estar vacío",
            },
        )));
    }

    let id = arles_core::ids::WhatsappAccountId::nuevo();
    let texto_del_id = id.to_string();

    // 1 · El secreto.
    let llavero = crate::llavero::Llavero::del_numero(&texto_del_id);
    llavero.guardar_contrasena(&token).map_err(ErrorIpc::from)?;

    // 2 · La base. Si falla, se retira lo que se acaba de escribir.
    let escritura = estado.db().guardar_numero(
        empresa,
        id,
        &arles_db::numero::NumeroNuevo {
            nombre_visible: &nombre_visible,
            telefono_e164: telefono.normalized(),
            waba_id: &waba_id,
            phone_number_id: &phone_number_id,
            modo,
        },
        &crate::llavero::referencia_de_numero(&texto_del_id),
    );

    if let Err(e) = escritura {
        let _ = llavero.borrar();
        return Err(ErrorIpc::from(crate::AppError::Db(e)));
    }

    numeros_de_whatsapp(estado)
}

/// Confirma que el número está en Coexistencia (L-11).
///
/// Se guarda **quién y cuándo**, no un simple «sí». Una marca sin autor no
/// sirve de constancia el día que alguien pregunte quién dijo que este número
/// podía seguir atendiéndose desde el móvil.
///
/// # Errores
///
/// [`ErrorIpc`] si el identificador no vale o si la escritura falla.
#[tauri::command]
pub fn confirmar_coexistencia(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<Vec<NumeroIpc>, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let numero = id
        .parse::<arles_core::ids::WhatsappAccountId>()
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    estado
        .db()
        .confirmar_coexistencia(empresa, numero, USUARIO_LOCAL)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    numeros_de_whatsapp(estado)
}

/// Acepta el riesgo de escribir a quien no dio permiso.
///
/// Sólo tiene sentido en el modo de prospección directa. Se deja llamar igual
/// en el otro —la marca queda puesta y no hace daño—, porque rechazarlo
/// obligaría a la pantalla a conocer la regla del modo, y ésa vive en el
/// núcleo.
///
/// # Errores
///
/// [`ErrorIpc`] si el identificador no vale o si la escritura falla.
#[tauri::command]
pub fn aceptar_riesgo_del_canal(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
) -> Result<Vec<NumeroIpc>, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let numero = id
        .parse::<arles_core::ids::WhatsappAccountId>()
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    estado
        .db()
        .aceptar_el_riesgo(empresa, numero, USUARIO_LOCAL)
        .map_err(|e| ErrorIpc::from(crate::AppError::Db(e)))?;

    numeros_de_whatsapp(estado)
}

/// El texto que explica un fallo de envío (§139).
///
/// `Fallo` vive en el núcleo y **no tiene `clave_i18n`**: es un tipo del motor,
/// pensado para decidir si se reintenta, no para enseñarse. La traducción a
/// algo que una persona pueda leer se hace aquí, en la frontera, que es donde
/// hace falta.
///
/// Se escribe con un `match` exhaustivo a propósito: si mañana el núcleo añade
/// una forma de fallar, esto **no compila** hasta que alguien decida qué se le
/// dice al usuario. Un `_ =>` genérico la enseñaría como «error desconocido»
/// para siempre, sin que nadie se enterara.
const fn clave_del_fallo(fallo: &arles_core::reintento::Fallo) -> &'static str {
    use arles_core::reintento::Fallo;
    match fallo {
        Fallo::Pasajero => "whatsapp.prueba.pasajero",
        Fallo::NoMeMandesNadaDurante(_) => "whatsapp.prueba.demasiadoRapido",
        Fallo::LaDireccionNoExiste => "whatsapp.prueba.numeroNoExiste",
        Fallo::LaCuentaNoAutentica => "whatsapp.prueba.tokenNoVale",
        Fallo::EsaPersonaPidioLaBaja => "whatsapp.prueba.pidioLaBaja",
    }
}

/// Quién queda anotado en las dos marcas.
///
/// ARLES no tiene cuentas de usuario (está fuera de alcance en la v1.2), así
/// que la constancia dice «el usuario de este equipo» y no inventa un nombre.
/// Escribir aquí un identificador que no existe sería peor que esta cadena
/// honesta: parecería una auditoría real.
const USUARIO_LOCAL: &str = "usuario_local";

/// Cómo salió el envío de prueba de WhatsApp.
///
/// **No dice «entregado»** (§65), igual que el de correo. Lo que ARLES sabe es
/// que Meta se hizo cargo del mensaje. Si llegó al teléfono, y si la persona lo
/// vio, son cosas que solo dirían los avisos — y los avisos todavía no tienen
/// por dónde llegar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoDeLaPruebaDeWhatsApp {
    /// ¿Meta lo aceptó?
    pub aceptado: bool,
    /// La clave del texto que explica qué pasó (§139). Nunca el texto crudo de
    /// Meta: viene en inglés y a veces trae el número dentro.
    pub clave: String,
}

/// Manda un mensaje de prueba a un número, con una plantilla aprobada.
///
/// ─────────────────────────────────────────────────────────────────────────
/// ESTE COMANDO ES EL QUE FALTABA PARA QUE EL CANAL EXISTIERA DE VERDAD
///
/// Hasta la W.8, ARLES sabía dar de alta un número, revisar una campaña y
/// enseñar todas las cifras — y no tenía **ningún** camino por el que saliera
/// un mensaje. El adaptador estaba probado contra el servidor de mentira y nadie
/// lo llamaba.
///
/// ─────────────────────────────────────────────────────────────────────────
/// LA PLANTILLA LA ELIGE QUIEN LLAMA, Y TIENE QUE ESTAR APROBADA
///
/// Meta **no acepta texto libre** para iniciar una conversación. Para la
/// demostración sirve `hello_world`, que viene aprobada con el número de prueba
/// que Meta regala — y que por eso no depende del editor de plantillas, que
/// sigue pendiente (entrega 6.8).
///
/// `base` permite apuntar al servidor de mentira en las pruebas. En producción
/// es la constante `GRAPH`, y **no se deja elegir desde la webview**: una
/// dirección que cruzara la frontera sería un sitio al que mandar el token.
///
/// # Errores
///
/// [`ErrorIpc`] si no hay empresa, si el identificador no vale, si el número no
/// puede enviar todavía —le falta una de las dos confirmaciones— o si el
/// llavero no tiene su token. **Que Meta rechace el mensaje no es un error del
/// comando**: es un resultado con `aceptado: false` y su clave, porque eso es
/// justo lo que la prueba venía a averiguar.
#[tauri::command]
pub fn enviar_whatsapp_de_prueba(
    estado: tauri::State<'_, EstadoApp>,
    id: String,
    para: String,
    plantilla: String,
    idioma: String,
) -> Result<ResultadoDeLaPruebaDeWhatsApp, ErrorIpc> {
    let empresa = empresa_actual(&estado)?;
    let pais = crate::comandos::pais_de_la_empresa(&estado)?;
    prueba_con(
        estado.db(),
        &crate::fabrica::ElLlaveroDelSistema,
        empresa,
        &pais,
        &id,
        &para,
        &plantilla,
        &idioma,
        arles_whatsapp::cloud::GRAPH,
    )
}

/// El envío de prueba, con la fuente de secretos y la dirección que se le diga.
///
/// El comando de arriba le pasa el llavero del sistema y la dirección de Meta.
/// Las pruebas le pasan las suyas y **recorren este mismo código**: la lectura
/// del número, la puerta del canal, el armado del proveedor y la traducción del
/// resultado. Una prueba con su propio camino no diría nada de éste — que es
/// exactamente cómo el canal llegó a la W.7 sin poder enviar.
///
/// # Errores
///
/// Los de [`enviar_whatsapp_de_prueba`].
#[allow(clippy::too_many_arguments)]
pub fn prueba_con(
    db: &arles_db::Db,
    secretos: &dyn crate::fabrica::FuenteDeContrasenas,
    empresa: arles_core::ids::CompanyId,
    pais: &str,
    id: &str,
    para: &str,
    plantilla: &str,
    idioma: &str,
    base: &str,
) -> Result<ResultadoDeLaPruebaDeWhatsApp, ErrorIpc> {
    use arles_motor::proveedor::{Mensaje, Proveedor};

    let numero = id
        .parse::<arles_core::ids::WhatsappAccountId>()
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    // El destino se valida en el núcleo, con el país de la empresa: quien
    // escribe su propio móvil para probar lo escribe como lo tiene en el móvil.
    let destino = arles_core::canal::PhoneNumber::parse(para, pais)
        .map_err(|e| ErrorIpc::from(crate::AppError::Core(e)))?;

    let contenido = arles_core::whatsapp::MensajeDePlantilla {
        nombre: plantilla.to_owned(),
        idioma: idioma.to_owned(),
        // `hello_world` no tiene huecos. Una plantilla con huecos necesitaría
        // los valores, y ésos los elige la campaña — no una prueba.
        valores: Vec::new(),
        faltaron: Vec::new(),
    };

    let proveedor =
        crate::fabrica::proveedor_del_numero(db, secretos, empresa, numero, contenido, base)
            .map_err(ErrorIpc::from)?;

    // La clave de idempotencia de una prueba no se guarda en ningún sitio: no
    // hay envío que deduplicar, y reutilizar la de un envío real haría que el
    // motor creyera que ese mensaje ya salió.
    let clave = format!("prueba-{}", numero);
    let mensaje = Mensaje {
        para: destino.normalized(),
        // **En WhatsApp el contenido no viaja por aquí**, y no es un olvido:
        // Meta no acepta texto libre en un primer mensaje. Lo que se manda es
        // una plantilla que Meta aprobó, con sus valores, y eso va con el
        // proveedor —arriba, en `contenido`— porque es lo que la API pide.
        //
        // El `Mensaje` es el mismo para los dos canales, así que estos dos
        // campos van vacíos en este camino.
        asunto: "",
        html: "",
        de_una_campana: false,
        clave_de_idempotencia: &clave,
    };

    Ok(match proveedor.enviar(&mensaje) {
        Ok(_) => ResultadoDeLaPruebaDeWhatsApp {
            aceptado: true,
            clave: "whatsapp.prueba.aceptado".to_owned(),
        },
        Err(fallo) => ResultadoDeLaPruebaDeWhatsApp {
            aceptado: false,
            clave: clave_del_fallo(&fallo).to_owned(),
        },
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn numero_de_muestra(coexistencia: Option<&str>, riesgo: Option<&str>) -> NumeroIpc {
        NumeroIpc {
            id: "01900000-0000-7000-8000-0000000000a1".to_owned(),
            nombre_visible: "Ventas".to_owned(),
            telefono_e164: "+528100000000".to_owned(),
            waba_id: "123456".to_owned(),
            modo: "prospeccion_directa".to_owned(),
            estado: "connected".to_owned(),
            coexistencia_confirmada: coexistencia.map(ToOwned::to_owned),
            riesgo_aceptado: riesgo.map(ToOwned::to_owned),
            calidad: None,
            limite_de_meta: Some(250),
            puede_enviar: coexistencia.is_some() && riesgo.is_some(),
            escribe_en_frio: true,
        }
    }

    /// **Lo que cruza la frontera no lleva el token ni dónde encontrarlo.**
    ///
    /// Se serializa de verdad y se busca en el JSON, en vez de leer la
    /// definición del `struct`: si alguien añade un campo mañana, esta prueba
    /// lo dice. Se busca también `llavero://`, que es la referencia: no es el
    /// secreto, pero decir dónde está tampoco le sirve a la pantalla.
    ///
    /// Verificada rompiéndola el 29/09/2026: copiando
    /// `referencia_de_credencial` al `From`, esta prueba falla.
    #[test]
    fn lo_que_sale_no_lleva_el_token_ni_la_referencia() {
        let json = serde_json::to_string(&numero_de_muestra(None, None)).expect("serializa");

        assert!(!json.contains("llavero://"), "ni la referencia: {json}");
        assert!(!json.contains("token"), "ni la palabra token: {json}");
        assert!(
            json.contains("+528100000000"),
            "pero sí el número, que no es secreto: {json}"
        );
    }

    /// **Las dos marcas llegan como nulas cuando no constan, y no desaparecen.**
    ///
    /// La pantalla tiene que poder enseñar «sin confirmar» y no un hueco. Un
    /// campo que desaparece del JSON se lee en la webview como `undefined`, y
    /// `undefined` pinta igual que vacío — que es como una confirmación que
    /// nadie dio pasaría por una que sí.
    #[test]
    fn las_marcas_sin_poner_llegan_nulas_y_no_desaparecen() {
        let json = serde_json::to_string(&numero_de_muestra(None, None)).expect("serializa");

        assert!(json.contains("\"coexistenciaConfirmada\":null"), "{json}");
        assert!(json.contains("\"riesgoAceptado\":null"), "{json}");
        assert!(json.contains("\"calidad\":null"), "{json}");
    }

    /// **`puedeEnviar` viaja resuelto, no se deduce en la pantalla.**
    ///
    /// Un número en frío con Coexistencia pero sin el riesgo aceptado **no**
    /// puede enviar, y es el caso que una webview que mirara sólo la
    /// Coexistencia daría por bueno.
    #[test]
    fn un_numero_en_frio_sin_riesgo_aceptado_no_puede_enviar() {
        let solo_coexistencia = numero_de_muestra(Some("2026-09-29T00:00:00Z"), None);
        assert!(!solo_coexistencia.puede_enviar);

        let json = serde_json::to_string(&solo_coexistencia).expect("serializa");
        assert!(json.contains("\"puedeEnviar\":false"), "{json}");

        let ambas = numero_de_muestra(Some("2026-09-29T00:00:00Z"), Some("2026-09-29T00:01:00Z"));
        assert!(ambas.puede_enviar);
    }
}
